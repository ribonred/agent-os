//! Thin client for the orchestrator daemon's Unix-socket API.
//!
//! The webview never talks to the daemon directly (a browser can't reach
//! a Unix socket, and shouldn't): these commands proxy it. `agent_chat`
//! streams the daemon's ndjson events to the frontend through a Tauri
//! channel, one event per line, exactly as the daemon emitted them --
//! token, done, and error events pass through unaltered so the UI sees
//! the same truth the daemon logged.

use http_body_util::{BodyExt, Full};
use hyper::body::Bytes;
use hyper_util::client::legacy::Client;
use hyperlocal::{UnixClientExt, UnixConnector, Uri};
use serde::{Deserialize, Serialize};
use tauri::ipc::Channel;

// Mirrors the daemon's default; the env override serves dev setups where
// the daemon runs on a scratch socket instead of the systemd path.
fn socket_path() -> String {
    std::env::var("AGENTIC_OS_SOCKET")
        .unwrap_or_else(|_| "/run/agentic-os/orchestrator.sock".to_string())
}

fn unix_client() -> Client<UnixConnector, Full<Bytes>> {
    Client::unix()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

#[tauri::command]
pub async fn agent_status() -> Result<serde_json::Value, String> {
    let uri: hyper::Uri = Uri::new(socket_path(), "/status").into();
    let response = unix_client()
        .get(uri)
        .await
        .map_err(|e| format!("agent daemon unreachable: {e}"))?;

    let status = response.status();
    let body = response
        .into_body()
        .collect()
        .await
        .map_err(|e| format!("agent daemon response failed: {e}"))?
        .to_bytes();

    if !status.is_success() {
        return Err(format!(
            "agent daemon returned {status}: {}",
            String::from_utf8_lossy(&body)
        ));
    }
    serde_json::from_slice(&body).map_err(|e| format!("agent daemon sent invalid JSON: {e}"))
}

#[tauri::command]
pub async fn agent_chat(
    messages: Vec<ChatMessage>,
    on_event: Channel<serde_json::Value>,
) -> Result<(), String> {
    let uri: hyper::Uri = Uri::new(socket_path(), "/chat").into();
    let payload = serde_json::to_vec(&serde_json::json!({ "messages": messages }))
        .map_err(|e| format!("could not encode chat request: {e}"))?;
    let request = hyper::Request::post(uri)
        .header("content-type", "application/json")
        .body(Full::new(Bytes::from(payload)))
        .map_err(|e| format!("could not build chat request: {e}"))?;

    let response = unix_client()
        .request(request)
        .await
        .map_err(|e| format!("agent daemon unreachable: {e}"))?;

    let status = response.status();
    if !status.is_success() {
        let body = response
            .into_body()
            .collect()
            .await
            .map(|b| String::from_utf8_lossy(&b.to_bytes()).into_owned())
            .unwrap_or_default();
        return Err(format!("agent daemon rejected chat ({status}): {body}"));
    }

    // Reassemble ndjson lines across frame boundaries and forward each
    // complete one to the webview.
    let mut body = response.into_body();
    let mut buf: Vec<u8> = Vec::new();
    while let Some(frame) = body.frame().await {
        let frame = frame.map_err(|e| format!("chat stream failed mid-response: {e}"))?;
        let Some(data) = frame.data_ref() else {
            continue;
        };
        buf.extend_from_slice(data);
        while let Some(newline_at) = buf.iter().position(|&b| b == b'\n') {
            let line: Vec<u8> = buf.drain(..=newline_at).collect();
            let event: serde_json::Value = match serde_json::from_slice(&line) {
                Ok(event) => event,
                // A malformed line is a daemon bug worth hearing about,
                // not something to skip silently.
                Err(e) => return Err(format!("agent daemon sent an invalid stream line: {e}")),
            };
            on_event
                .send(event)
                .map_err(|e| format!("ui channel closed: {e}"))?;
        }
    }
    Ok(())
}
