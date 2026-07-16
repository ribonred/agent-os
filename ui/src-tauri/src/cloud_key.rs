//! Tauri command surface over the shared cloud-key crate
//! (agent-core/cloud-key -- storage logic, precedence rules, and tests
//! live there, shared with the orchestrator daemon).
//!
//! Deliberate security posture: keys never come back out to the webview.
//! There is no command that returns a stored secret -- the frontend can
//! only save, check status, and delete. Consumers that actually need the
//! key (the orchestrator) call cloud_key::resolve_openrouter_key on the
//! Rust side, never through JS.

use cloud_key::KeySource;

#[tauri::command]
pub fn cloud_key_save(key: String) -> Result<(), String> {
    cloud_key::save_key(&key)
}

/// What the UI is allowed to know: which source is active, never the key.
#[tauri::command]
pub fn cloud_key_status() -> Result<String, String> {
    Ok(match cloud_key::key_status()? {
        Some(KeySource::Keyring) => "keyring",
        Some(KeySource::Provisioned) => "provisioned",
        None => "none",
    }
    .to_string())
}

#[tauri::command]
pub fn cloud_key_delete() -> Result<(), String> {
    cloud_key::delete_key()
}
