//! LLM backend clients: local Ollama and cloud OpenRouter, both streamed.
//!
//! Both backends speak "chat messages in, token stream out", so the rest
//! of the daemon deals in one `StreamEvent` shape and doesn't care which
//! side produced it. Line/SSE parsing is split into pure functions so the
//! protocol handling is unit-testable without a live upstream.

use futures_util::StreamExt;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

#[derive(Debug, PartialEq)]
pub enum StreamEvent {
    Token(String),
    Done,
}

/// One line of Ollama's /api/chat ndjson stream:
/// `{"message":{"role":"assistant","content":"..."},"done":false}` with a
/// final `"done":true` line. Unknown/empty lines yield None rather than
/// erroring -- the stream also carries timing metadata lines we don't use.
pub fn parse_ollama_line(line: &str) -> Option<StreamEvent> {
    #[derive(Deserialize)]
    struct OllamaChunk {
        message: Option<OllamaMessage>,
        done: bool,
    }
    #[derive(Deserialize)]
    struct OllamaMessage {
        content: String,
    }

    let line = line.trim();
    if line.is_empty() {
        return None;
    }
    let chunk: OllamaChunk = serde_json::from_str(line).ok()?;
    if chunk.done {
        return Some(StreamEvent::Done);
    }
    chunk
        .message
        .filter(|m| !m.content.is_empty())
        .map(|m| StreamEvent::Token(m.content))
}

/// One line of an OpenAI-style SSE stream (OpenRouter):
/// `data: {"choices":[{"delta":{"content":"..."}}]}` ending with
/// `data: [DONE]`. Non-data lines (comments, event names, blanks) yield
/// None.
pub fn parse_openrouter_sse_line(line: &str) -> Option<StreamEvent> {
    #[derive(Deserialize)]
    struct SseChunk {
        choices: Vec<SseChoice>,
    }
    #[derive(Deserialize)]
    struct SseChoice {
        delta: SseDelta,
    }
    #[derive(Deserialize)]
    struct SseDelta {
        content: Option<String>,
    }

    let data = line.trim().strip_prefix("data:")?.trim();
    if data == "[DONE]" {
        return Some(StreamEvent::Done);
    }
    let chunk: SseChunk = serde_json::from_str(data).ok()?;
    chunk
        .choices
        .into_iter()
        .next()
        .and_then(|c| c.delta.content)
        .filter(|content| !content.is_empty())
        .map(StreamEvent::Token)
}

/// Reads an HTTP byte stream, reassembles complete lines across chunk
/// boundaries, parses each with `parse`, and forwards events to `tx`.
/// Returns Err with a human-readable message on transport failure --
/// callers surface that to the client, never swallow it.
async fn pump_lines<F>(
    response: reqwest::Response,
    parse: F,
    tx: &mpsc::Sender<Result<StreamEvent, String>>,
) -> Result<(), String>
where
    F: Fn(&str) -> Option<StreamEvent>,
{
    let mut buf: Vec<u8> = Vec::new();
    let mut stream = response.bytes_stream();

    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(|e| format!("upstream stream failed mid-response: {e}"))?;
        buf.extend_from_slice(&chunk);

        while let Some(newline_at) = buf.iter().position(|&b| b == b'\n') {
            let line: Vec<u8> = buf.drain(..=newline_at).collect();
            let line = String::from_utf8_lossy(&line);
            if let Some(event) = parse(&line) {
                let done = event == StreamEvent::Done;
                if tx.send(Ok(event)).await.is_err() {
                    return Ok(()); // client hung up; nothing left to do
                }
                if done {
                    return Ok(());
                }
            }
        }
    }
    // Upstream closed without a done marker -- still report done so the
    // client isn't left hanging, but log it: a well-behaved backend ends
    // with an explicit terminator.
    log::warn!("upstream closed stream without a done marker");
    let _ = tx.send(Ok(StreamEvent::Done)).await;
    Ok(())
}

pub struct OllamaClient {
    pub base_url: String,
    pub model: String,
}

impl OllamaClient {
    pub async fn chat(
        &self,
        http: &reqwest::Client,
        messages: &[ChatMessage],
        tx: &mpsc::Sender<Result<StreamEvent, String>>,
    ) -> Result<(), String> {
        let response = http
            .post(format!("{}/api/chat", self.base_url))
            .json(&serde_json::json!({
                "model": self.model,
                "messages": messages,
                "stream": true,
            }))
            .send()
            .await
            .map_err(|e| format!("could not reach local model server: {e}"))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(format!("local model server returned {status}: {body}"));
        }
        pump_lines(response, parse_ollama_line, tx).await
    }
}

pub struct OpenRouterClient {
    pub base_url: String,
    pub model: String,
}

impl OpenRouterClient {
    pub async fn chat(
        &self,
        http: &reqwest::Client,
        api_key: &str,
        messages: &[ChatMessage],
        tx: &mpsc::Sender<Result<StreamEvent, String>>,
    ) -> Result<(), String> {
        let response = http
            .post(format!("{}/chat/completions", self.base_url))
            .bearer_auth(api_key)
            .json(&serde_json::json!({
                "model": self.model,
                "messages": messages,
                "stream": true,
            }))
            .send()
            .await
            .map_err(|e| format!("could not reach cloud provider: {e}"))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(format!("cloud provider returned {status}: {body}"));
        }
        pump_lines(response, parse_openrouter_sse_line, tx).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ollama_content_line_is_a_token() {
        let line = r#"{"message":{"role":"assistant","content":"Hi"},"done":false}"#;
        assert_eq!(
            parse_ollama_line(line),
            Some(StreamEvent::Token("Hi".to_string()))
        );
    }

    #[test]
    fn ollama_done_line_ends_the_stream() {
        // Real final lines carry timing stats alongside done:true.
        let line = r#"{"done":true,"total_duration":123,"eval_count":5}"#;
        assert_eq!(parse_ollama_line(line), Some(StreamEvent::Done));
    }

    #[test]
    fn ollama_blank_and_garbage_lines_are_skipped_not_fatal() {
        assert_eq!(parse_ollama_line(""), None);
        assert_eq!(parse_ollama_line("not json"), None);
    }

    #[test]
    fn openrouter_delta_line_is_a_token() {
        let line = r#"data: {"choices":[{"delta":{"content":"Hello"}}]}"#;
        assert_eq!(
            parse_openrouter_sse_line(line),
            Some(StreamEvent::Token("Hello".to_string()))
        );
    }

    #[test]
    fn openrouter_done_marker_ends_the_stream() {
        assert_eq!(
            parse_openrouter_sse_line("data: [DONE]"),
            Some(StreamEvent::Done)
        );
    }

    #[test]
    fn openrouter_comment_and_role_only_lines_are_skipped() {
        // OpenRouter sends ": OPENROUTER PROCESSING" keep-alive comments,
        // and the first delta often carries only a role, no content.
        assert_eq!(parse_openrouter_sse_line(": OPENROUTER PROCESSING"), None);
        assert_eq!(
            parse_openrouter_sse_line(r#"data: {"choices":[{"delta":{"role":"assistant"}}]}"#),
            None
        );
    }
}
