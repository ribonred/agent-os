//! The owner's earlier conversations: listing them, reopening one, and
//! the few things they can do to one.
//!
//! Almost nothing is stored here. The gateway already keeps every
//! conversation, names them itself, and orders them by when they were
//! last used, so this module is a proxy with one piece of real logic --
//! turning a stored transcript back into the turns the pane renders.
//! Keeping the bearer token out of the webview is the same rule as
//! everywhere else: the frontend asks this process, never the gateway.

use http_body_util::{BodyExt, Full};
use hyper::body::Bytes;

use crate::agent::{
    api_key, base_url, http_client, save_session, stored_session, AgentSession, CHAT_SESSION_KEY,
    CHAT_SOURCE,
};

/// How much of a conversation is brought back on screen. The gateway
/// caps a page at 500; a device whose owner scrolls back further than
/// this is asking for a history feature that does not exist yet, and
/// rendering thousands of turns to make that point would be slow.
const TRANSCRIPT_LIMIT: usize = 200;

/// One request to the gateway with no body to send. The bodied calls the
/// shell makes elsewhere go through `agent::post_json`; these are the
/// shapes it does not cover -- reading a list, and the two verbs that
/// change or remove a conversation.
async fn request(
    method: hyper::Method,
    path: &str,
    key: &str,
    payload: Option<&serde_json::Value>,
) -> Result<(hyper::StatusCode, Vec<u8>), String> {
    let uri: hyper::Uri = format!("{}{path}", base_url())
        .parse()
        .map_err(|e| format!("bad gateway URL: {e}"))?;
    let mut builder = hyper::Request::builder()
        .method(method)
        .uri(uri)
        .header("authorization", format!("Bearer {key}"));
    let body = match payload {
        Some(payload) => {
            builder = builder.header("content-type", "application/json");
            Bytes::from(
                serde_json::to_vec(payload).map_err(|e| format!("could not encode request: {e}"))?,
            )
        }
        None => Bytes::new(),
    };
    let request = builder
        .body(Full::new(body))
        .map_err(|e| format!("could not build request: {e}"))?;

    let response = http_client()
        .request(request)
        .await
        .map_err(|e| format!("agent gateway unreachable: {e}"))?;
    let status = response.status();
    let body = response
        .into_body()
        .collect()
        .await
        .map_err(|e| format!("gateway response failed: {e}"))?
        .to_bytes();
    Ok((status, body.to_vec()))
}

async fn json(
    method: hyper::Method,
    path: &str,
    payload: Option<&serde_json::Value>,
    what: &str,
) -> Result<serde_json::Value, String> {
    let key = api_key()?;
    let (status, body) = request(method, path, &key, payload).await?;
    if !status.is_success() {
        return Err(format!(
            "agent gateway refused {what} ({status}): {}",
            String::from_utf8_lossy(&body)
        ));
    }
    serde_json::from_slice(&body).map_err(|e| format!("invalid {what} JSON: {e}"))
}

/// One row of the list, in the shell's own words rather than the
/// gateway's. Translating here rather than passing the gateway's
/// object through means the frontend depends on this shape and not on
/// a column set the runtime is free to change under us.
fn row(session: &serde_json::Value) -> serde_json::Value {
    serde_json::json!({
        "id": session["id"].as_str().unwrap_or_default(),
        "title": session["title"].as_str().unwrap_or_default(),
        "preview": session["preview"].as_str().unwrap_or_default(),
        // Seconds since the epoch, as a float. The frontend decides what
        // "yesterday" means, because that is a question about the
        // owner's day rather than about storage.
        "lastActive": session["last_active"].as_f64().unwrap_or(0.0),
        "messageCount": session["message_count"].as_u64().unwrap_or(0),
        "kept": session["pinned"].as_bool().unwrap_or(false),
    })
}

/// Rebuilds the turns of a stored conversation.
///
/// The gateway stores a transcript the way a model consumes it, which is
/// not the way the pane draws it: an assistant message carrying only
/// tool calls is machinery, and the row saying what the device did is
/// the tool's own result message a moment later. Dropping the former and
/// keeping the latter is what makes a reopened conversation read like
/// the one the owner watched arrive.
///
/// Permission requests are deliberately not here. They are not stored,
/// and a question that was already answered is not a question -- a card
/// with live buttons on a conversation from last week would be offering
/// a decision that has already been made.
fn transcript_turns(transcript: &serde_json::Value) -> Vec<serde_json::Value> {
    let Some(messages) = transcript["data"].as_array() else {
        return Vec::new();
    };
    messages
        .iter()
        .filter_map(|message| {
            let role = message["role"].as_str().unwrap_or_default();
            let content = message["content"].as_str().unwrap_or_default();
            match role {
                "user" => (!content.trim().is_empty())
                    .then(|| serde_json::json!({ "kind": "user", "content": content })),
                "assistant" => (!content.trim().is_empty())
                    .then(|| serde_json::json!({ "kind": "assistant", "content": content })),
                // The name goes over as the runtime spells it; the
                // frontend has the one table that turns a tool name into
                // something the owner reads, and a replayed row must
                // come out of it identically to a live one.
                "tool" => Some(serde_json::json!({
                    "kind": "tool",
                    "name": message["tool_name"].as_str().unwrap_or_default(),
                })),
                _ => None,
            }
        })
        .collect()
}

/// The owner's conversations, most recently used first.
///
/// Only the ones they had: the source tag keeps setup, and anything else
/// on this device that talks to the same runtime, out of the list.
#[tauri::command]
pub async fn sessions_list(limit: u32, offset: u32) -> Result<serde_json::Value, String> {
    let path =
        format!("/api/sessions?source={CHAT_SOURCE}&limit={}&offset={offset}", limit.clamp(1, 200));
    let body = json(hyper::Method::GET, &path, None, "the conversation list").await?;
    let rows: Vec<serde_json::Value> = body["data"]
        .as_array()
        .map(|sessions| sessions.iter().map(row).collect())
        .unwrap_or_default();
    Ok(serde_json::json!({
        "sessions": rows,
        "hasMore": body["has_more"].as_bool().unwrap_or(false),
    }))
}

/// Which conversation the pane should be showing. None on a device whose
/// owner has not said anything yet.
#[tauri::command]
pub async fn sessions_active(
    app: tauri::AppHandle,
    session: tauri::State<'_, AgentSession>,
) -> Result<Option<String>, String> {
    let mut active = session.chat.lock().await;
    if active.is_none() {
        *active = stored_session(&app, CHAT_SESSION_KEY);
    }
    Ok(active.clone())
}

/// Reopens a conversation: makes it the one the next turn continues, and
/// hands back what was said in it.
#[tauri::command]
pub async fn sessions_open(
    session_id: String,
    app: tauri::AppHandle,
    session: tauri::State<'_, AgentSession>,
) -> Result<Vec<serde_json::Value>, String> {
    // The transcript is fetched before anything is committed, so a
    // conversation the gateway has lost leaves the owner where they
    // were rather than in an empty pane pointed at nothing.
    let transcript = json(
        hyper::Method::GET,
        &format!("/api/sessions/{session_id}/messages?order=latest&limit={TRANSCRIPT_LIMIT}"),
        None,
        "the conversation",
    )
    .await?;

    // The gateway resolves a conversation that has been compressed and
    // continued to whichever one is live now. Continuing the id it
    // reports rather than the one asked for is what keeps a long
    // conversation from forking on reopen.
    let resolved = transcript["session_id"]
        .as_str()
        .unwrap_or(&session_id)
        .to_string();

    let mut active = session.chat.lock().await;
    *active = Some(resolved.clone());
    save_session(&app, CHAT_SESSION_KEY, Some(&resolved))?;
    Ok(transcript_turns(&transcript))
}

/// Starts a fresh conversation. Nothing is created here: the next turn
/// opens one, exactly as it does on a device that has never been used.
#[tauri::command]
pub async fn sessions_new(
    app: tauri::AppHandle,
    session: tauri::State<'_, AgentSession>,
) -> Result<(), String> {
    let mut active = session.chat.lock().await;
    *active = None;
    save_session(&app, CHAT_SESSION_KEY, None)
}

/// The owner's own name for a conversation, replacing the one the device
/// wrote for it.
#[tauri::command]
pub async fn sessions_rename(session_id: String, title: String) -> Result<(), String> {
    let payload = serde_json::json!({ "title": title.trim() });
    json(
        hyper::Method::PATCH,
        &format!("/api/sessions/{session_id}"),
        Some(&payload),
        "the new name",
    )
    .await
    .map(|_| ())
}

/// "Keep this one" -- it stays at the top of the list and is exempt from
/// aging out of it.
#[tauri::command]
pub async fn sessions_keep(session_id: String, kept: bool) -> Result<(), String> {
    let payload = serde_json::json!({ "pinned": kept });
    json(
        hyper::Method::PATCH,
        &format!("/api/sessions/{session_id}"),
        Some(&payload),
        "the change",
    )
    .await
    .map(|_| ())
}

/// Removes a conversation for good. Deleting the one on screen leaves
/// the pane on a fresh conversation rather than on a transcript that no
/// longer exists anywhere.
#[tauri::command]
pub async fn sessions_delete(
    session_id: String,
    app: tauri::AppHandle,
    session: tauri::State<'_, AgentSession>,
) -> Result<(), String> {
    json(
        hyper::Method::DELETE,
        &format!("/api/sessions/{session_id}"),
        None,
        "the deletion",
    )
    .await?;
    let mut active = session.chat.lock().await;
    if active.as_deref() == Some(session_id.as_str()) {
        *active = None;
        save_session(&app, CHAT_SESSION_KEY, None)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Shaped exactly like a transcript read back from the running
    /// gateway: the owner asks, the device makes three tool calls whose
    /// assistant messages carry no text, and only then says something.
    fn exchange() -> serde_json::Value {
        serde_json::json!({
            "object": "list",
            "session_id": "api_1_a",
            "data": [
                { "role": "user", "content": "Save this note for me." },
                { "role": "assistant", "content": "", "tool_calls": [{ "id": "c1" }] },
                { "role": "tool", "tool_name": "search_files", "content": "{\"total_count\": 50}" },
                { "role": "assistant", "content": "", "tool_calls": [{ "id": "c2" }] },
                { "role": "tool", "tool_name": "read_file", "content": "{\"content\": \"x\"}" },
                { "role": "assistant", "content": "Where would you like it?" }
            ]
        })
    }

    #[test]
    fn a_stored_exchange_comes_back_as_the_turns_that_were_drawn() {
        assert_eq!(
            transcript_turns(&exchange()),
            vec![
                serde_json::json!({ "kind": "user", "content": "Save this note for me." }),
                serde_json::json!({ "kind": "tool", "name": "search_files" }),
                serde_json::json!({ "kind": "tool", "name": "read_file" }),
                serde_json::json!({ "kind": "assistant", "content": "Where would you like it?" }),
            ]
        );
    }

    #[test]
    fn an_assistant_message_that_only_carried_tool_calls_is_not_a_turn() {
        // It renders as nothing, so replaying it would insert blank
        // gaps between the rows saying what the device did.
        let turns = transcript_turns(&exchange());
        assert!(turns
            .iter()
            .all(|turn| turn["kind"] != "assistant" || turn["content"] != ""));
    }

    #[test]
    fn a_conversation_with_nothing_in_it_replays_as_nothing() {
        let empty = serde_json::json!({ "object": "list", "data": [] });
        assert!(transcript_turns(&empty).is_empty());
        // A response that isn't shaped like one at all must not panic
        // the pane on reopen.
        assert!(transcript_turns(&serde_json::json!({})).is_empty());
    }

    #[test]
    fn a_tool_the_shell_has_never_heard_of_still_becomes_a_row() {
        // Tools arrive when the runtime is upgraded. The row is what
        // tells the owner the device did something; dropping it because
        // the name is unfamiliar would hide the work, and the frontend
        // already has a truthful sentence for a name it doesn't know.
        let transcript = serde_json::json!({
            "data": [{ "role": "tool", "tool_name": "acp_bridge", "content": "{}" }]
        });
        assert_eq!(
            transcript_turns(&transcript),
            vec![serde_json::json!({ "kind": "tool", "name": "acp_bridge" })]
        );
    }

    #[test]
    fn roles_the_pane_has_no_place_for_are_dropped() {
        // A stored system message is the shell's own overlay coming back
        // at us. It was never on screen and must not appear now.
        let transcript = serde_json::json!({
            "data": [
                { "role": "system", "content": "Your owner has named you Ada." },
                { "role": "user", "content": "Hello" }
            ]
        });
        assert_eq!(
            transcript_turns(&transcript),
            vec![serde_json::json!({ "kind": "user", "content": "Hello" })]
        );
    }

    #[test]
    fn a_listed_conversation_is_translated_out_of_the_gateways_vocabulary() {
        let session = serde_json::json!({
            "id": "api_1_a",
            "title": "Track invoices",
            "preview": "I want help keeping track of my invoices.",
            "last_active": 1787039991.9215083_f64,
            "message_count": 2,
            "pinned": true,
            "archived": false,
            "estimated_cost_usd": 0.004
        });
        let row = row(&session);
        assert_eq!(row["title"], "Track invoices");
        assert_eq!(row["messageCount"], 2);
        assert_eq!(row["kept"], true);
        // Nothing about spend, tokens or storage reaches the webview:
        // the list is a list of conversations, not of runs.
        assert!(row.get("estimated_cost_usd").is_none());
        assert!(row.get("archived").is_none());
    }

    #[test]
    fn a_conversation_the_device_has_not_named_yet_carries_no_name() {
        // A brand-new conversation has no title for a second or two.
        // Empty rather than absent, so the frontend has one case to
        // handle instead of two.
        let row = row(&serde_json::json!({ "id": "api_1_a", "message_count": 0 }));
        assert_eq!(row["title"], "");
        assert_eq!(row["kept"], false);
        assert_eq!(row["lastActive"], 0.0);
    }
}
