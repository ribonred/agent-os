//! Thin client for the local Hermes Agent gateway's HTTP API.
//!
//! The webview never talks to the gateway directly: these commands proxy
//! it, so the bearer token never crosses into the web context. `agent_chat`
//! holds one gateway session for the app's lifetime (the gateway owns the
//! conversation history server-side) and streams each turn's SSE events to
//! the frontend through a Tauri channel, translated to the UI's own
//! event shape: token, done, and error.

use http_body_util::{BodyExt, Full};
use hyper::body::Bytes;
use hyper_util::client::legacy::{connect::HttpConnector, Client};
use hyper_util::rt::TokioExecutor;
use tauri::ipc::Channel;
use tokio::sync::Mutex;

// Mirrors the gateway's default bind; the env override serves dev setups
// pointing at a non-default port.
fn base_url() -> String {
    std::env::var("AGENTIC_OS_HERMES_URL").unwrap_or_else(|_| "http://127.0.0.1:8642".to_string())
}

/// Scopes the agent's long-term memory independently of transcript
/// sessions -- stable across app restarts, so what the device learns
/// about its owner persists.
const MEMORY_SCOPE: &str = "agentic-os:device:main";

/// Reads `API_SERVER_KEY=...` out of an env-format file.
fn key_from_env_file(path: &std::path::Path) -> Option<String> {
    let content = std::fs::read_to_string(path).ok()?;
    content.lines().find_map(|line| {
        let value = line.trim().strip_prefix("API_SERVER_KEY=")?.trim();
        (!value.is_empty()).then(|| value.trim_matches('"').to_string())
    })
}

/// The gateway bearer token, checked loudest-override first:
/// 1. AGENTIC_OS_HERMES_KEY -- explicit, for scratch setups
/// 2. /etc/agentic-os/hermes.env -- the device: written at factory time
///    by the installer, handed to this user via tmpfiles
/// 3. ~/.hermes/.env -- dev machines: the local Hermes install's own
///    config, so `make dev` needs no key copying at all
fn api_key() -> Result<String, String> {
    if let Ok(key) = std::env::var("AGENTIC_OS_HERMES_KEY") {
        if !key.is_empty() {
            return Ok(key);
        }
    }
    let device_file = std::path::PathBuf::from("/etc/agentic-os/hermes.env");
    if let Some(key) = key_from_env_file(&device_file) {
        return Ok(key);
    }
    if let Some(home) = std::env::var_os("HOME") {
        let dev_file = std::path::PathBuf::from(home).join(".hermes/.env");
        if let Some(key) = key_from_env_file(&dev_file) {
            return Ok(key);
        }
    }
    Err("no Hermes API key found: set AGENTIC_OS_HERMES_KEY, or provide \
         API_SERVER_KEY in /etc/agentic-os/hermes.env (device) or ~/.hermes/.env (dev)"
        .to_string())
}

fn http_client() -> Client<HttpConnector, Full<Bytes>> {
    Client::builder(TokioExecutor::new()).build_http()
}

/// One gateway session per app run, created lazily on first chat.
#[derive(Default)]
pub struct AgentSession(Mutex<Option<String>>);

#[tauri::command]
pub async fn agent_status() -> Result<serde_json::Value, String> {
    let uri: hyper::Uri = format!("{}/health", base_url())
        .parse()
        .map_err(|e| format!("bad gateway URL: {e}"))?;
    let response = http_client()
        .get(uri)
        .await
        .map_err(|e| format!("agent gateway unreachable: {e}"))?;

    let status = response.status();
    let body = response
        .into_body()
        .collect()
        .await
        .map_err(|e| format!("agent gateway response failed: {e}"))?
        .to_bytes();

    if !status.is_success() {
        return Err(format!(
            "agent gateway returned {status}: {}",
            String::from_utf8_lossy(&body)
        ));
    }
    serde_json::from_slice(&body).map_err(|e| format!("agent gateway sent invalid JSON: {e}"))
}

async fn create_session(key: &str) -> Result<String, String> {
    let uri: hyper::Uri = format!("{}/api/sessions", base_url())
        .parse()
        .map_err(|e| format!("bad gateway URL: {e}"))?;
    let request = hyper::Request::post(uri)
        .header("authorization", format!("Bearer {key}"))
        .header("content-type", "application/json")
        .body(Full::new(Bytes::from_static(b"{}")))
        .map_err(|e| format!("could not build session request: {e}"))?;

    let response = http_client()
        .request(request)
        .await
        .map_err(|e| format!("agent gateway unreachable: {e}"))?;

    let status = response.status();
    let body = response
        .into_body()
        .collect()
        .await
        .map_err(|e| format!("session response failed: {e}"))?
        .to_bytes();
    if !status.is_success() {
        return Err(format!(
            "agent gateway refused session ({status}): {}",
            String::from_utf8_lossy(&body)
        ));
    }

    let parsed: serde_json::Value =
        serde_json::from_slice(&body).map_err(|e| format!("invalid session JSON: {e}"))?;
    parsed["session"]["id"]
        .as_str()
        .map(str::to_string)
        .ok_or_else(|| "session response carried no session.id".to_string())
}

/// One complete SSE record ("event:"/"data:" lines up to a blank line),
/// translated to the UI's stream-event shape. Returns None for records
/// the UI has no use for (lifecycle bookkeeping, tool progress -- the
/// orb's thinking rhythm already covers the pre-token wait).
fn translate_sse_record(record: &str) -> Option<serde_json::Value> {
    let mut event_name = "";
    let mut data = String::new();
    for line in record.lines() {
        let line = line.trim_end_matches('\r');
        if let Some(rest) = line.strip_prefix("event:") {
            event_name = rest.trim();
        } else if let Some(rest) = line.strip_prefix("data:") {
            data.push_str(rest.trim_start());
        }
    }

    match event_name {
        "assistant.delta" => {
            let parsed: serde_json::Value = serde_json::from_str(&data).ok()?;
            let delta = parsed["delta"].as_str()?.to_string();
            Some(serde_json::json!({ "type": "token", "content": delta }))
        }
        "run.completed" => Some(serde_json::json!({ "type": "done" })),
        name if name.contains("error") || name.contains("failed") => {
            let message = serde_json::from_str::<serde_json::Value>(&data)
                .ok()
                .and_then(|v| {
                    v["message"]
                        .as_str()
                        .or(v["error"].as_str())
                        .map(str::to_string)
                })
                .unwrap_or_else(|| format!("{event_name}: {data}"));
            Some(serde_json::json!({ "type": "error", "message": message }))
        }
        _ => None,
    }
}

#[tauri::command]
pub async fn agent_chat(
    input: String,
    session: tauri::State<'_, AgentSession>,
    on_event: Channel<serde_json::Value>,
) -> Result<(), String> {
    let key = api_key()?;

    // Hold the lock across the whole turn: the gateway serializes turns
    // per session anyway, and this keeps a second send from racing
    // session creation.
    let mut session_id = session.0.lock().await;
    if session_id.is_none() {
        *session_id = Some(create_session(&key).await?);
    }
    let id = session_id.as_ref().expect("session id set above").clone();

    let uri: hyper::Uri = format!("{}/api/sessions/{id}/chat/stream", base_url())
        .parse()
        .map_err(|e| format!("bad gateway URL: {e}"))?;
    let payload = serde_json::to_vec(&serde_json::json!({ "input": input }))
        .map_err(|e| format!("could not encode chat request: {e}"))?;
    let request = hyper::Request::post(uri)
        .header("authorization", format!("Bearer {key}"))
        .header("content-type", "application/json")
        .header("x-hermes-session-key", MEMORY_SCOPE)
        .body(Full::new(Bytes::from(payload)))
        .map_err(|e| format!("could not build chat request: {e}"))?;

    let response = http_client()
        .request(request)
        .await
        .map_err(|e| format!("agent gateway unreachable: {e}"))?;

    let status = response.status();
    if !status.is_success() {
        let body = response
            .into_body()
            .collect()
            .await
            .map(|b| String::from_utf8_lossy(&b.to_bytes()).into_owned())
            .unwrap_or_default();
        // A dead session (gateway restarted, session evicted) shouldn't
        // wedge the app until relaunch -- drop it so the next send
        // starts fresh.
        if status == hyper::StatusCode::NOT_FOUND {
            *session_id = None;
        }
        return Err(format!("agent gateway rejected chat ({status}): {body}"));
    }

    // Reassemble SSE records across frame boundaries; records are
    // separated by a blank line.
    let mut body = response.into_body();
    let mut buf: Vec<u8> = Vec::new();
    while let Some(frame) = body.frame().await {
        let frame = frame.map_err(|e| format!("chat stream failed mid-response: {e}"))?;
        let Some(data) = frame.data_ref() else {
            continue;
        };
        buf.extend_from_slice(data);
        while let Some(boundary) = buf.windows(2).position(|w| w == b"\n\n") {
            let record: Vec<u8> = buf.drain(..boundary + 2).collect();
            let record = String::from_utf8_lossy(&record);
            if let Some(event) = translate_sse_record(&record) {
                on_event
                    .send(event)
                    .map_err(|e| format!("ui channel closed: {e}"))?;
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn delta_record_becomes_a_token() {
        let record = "event: assistant.delta\ndata: {\"delta\": \"Hi\", \"seq\": 3}\n";
        assert_eq!(
            translate_sse_record(record),
            Some(serde_json::json!({ "type": "token", "content": "Hi" }))
        );
    }

    #[test]
    fn run_completed_becomes_done() {
        let record = "event: run.completed\ndata: {\"completed\": true}\n";
        assert_eq!(
            translate_sse_record(record),
            Some(serde_json::json!({ "type": "done" }))
        );
    }

    #[test]
    fn bookkeeping_records_are_dropped() {
        for name in ["run.started", "message.started", "tool.progress", "tool.started"] {
            let record = format!("event: {name}\ndata: {{}}\n");
            assert_eq!(translate_sse_record(&record), None, "{name}");
        }
    }

    #[test]
    fn error_records_surface_their_message() {
        let record = "event: run.error\ndata: {\"message\": \"model unavailable\"}\n";
        assert_eq!(
            translate_sse_record(record),
            Some(serde_json::json!({ "type": "error", "message": "model unavailable" }))
        );
    }

    /// Full round-trip against a live gateway; opt-in because it needs
    /// one running (and a resolvable key): cargo test -- --ignored
    #[tokio::test]
    #[ignore]
    async fn live_gateway_roundtrip() {
        let key = api_key().expect("no key resolvable");
        let session = create_session(&key).await.expect("session create failed");
        assert!(session.starts_with("api_"), "unexpected session id: {session}");

        let uri: hyper::Uri = format!("{}/api/sessions/{session}/chat/stream", base_url())
            .parse()
            .unwrap();
        let body = serde_json::json!({ "input": "Reply with the single word: pong" });
        let request = hyper::Request::post(uri)
            .header("authorization", format!("Bearer {key}"))
            .header("content-type", "application/json")
            .body(Full::new(Bytes::from(serde_json::to_vec(&body).unwrap())))
            .unwrap();
        let response = http_client().request(request).await.expect("stream failed");
        assert!(response.status().is_success(), "{}", response.status());

        let mut body = response.into_body();
        let mut buf: Vec<u8> = Vec::new();
        let mut tokens = String::new();
        let mut saw_done = false;
        while let Some(frame) = body.frame().await {
            let frame = frame.expect("frame error");
            if let Some(data) = frame.data_ref() {
                buf.extend_from_slice(data);
                while let Some(boundary) = buf.windows(2).position(|w| w == b"\n\n") {
                    let record: Vec<u8> = buf.drain(..boundary + 2).collect();
                    match translate_sse_record(&String::from_utf8_lossy(&record)) {
                        Some(ev) if ev["type"] == "token" => {
                            tokens.push_str(ev["content"].as_str().unwrap())
                        }
                        Some(ev) if ev["type"] == "done" => saw_done = true,
                        Some(ev) => panic!("stream error event: {ev}"),
                        None => {}
                    }
                }
            }
        }
        assert!(saw_done, "stream ended without run.completed");
        assert!(!tokens.is_empty(), "no tokens streamed");
        println!("assistant said: {tokens}");
    }

    #[test]
    fn env_file_key_is_extracted() {
        let dir = std::env::temp_dir().join("aos-agent-test");
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("hermes.env");
        std::fs::write(&path, "OPENROUTER_API_KEY=or-xyz\nAPI_SERVER_KEY=sekrit\n").unwrap();
        assert_eq!(key_from_env_file(&path), Some("sekrit".to_string()));
        std::fs::remove_file(&path).unwrap();
    }
}
