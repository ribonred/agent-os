//! Tauri command surface over the shared cloud-key crate
//! (agent-core/cloud-key -- storage logic, precedence rules, and tests
//! live there, shared with any Rust-side consumer).
//!
//! Deliberate security posture: keys never come back out to the webview.
//! There is no command that returns a stored secret -- the frontend can
//! only save, check status, and delete. Consumers that actually need a
//! key call cloud_key::resolve_key on the Rust side, never through JS.

use cloud_key::{KeySource, Provider};

/// What the UI is allowed to know: which source is active, never the key.
fn status_word(source: Option<KeySource>) -> String {
    match source {
        Some(KeySource::Keyring) => "keyring",
        Some(KeySource::Provisioned) => "provisioned",
        None => "none",
    }
    .to_string()
}

#[tauri::command]
pub fn cloud_key_save(key: String) -> Result<(), String> {
    cloud_key::save_key_for(Provider::OpenRouter, &key)
}

#[tauri::command]
pub fn cloud_key_status() -> Result<String, String> {
    Ok(status_word(cloud_key::key_status_for(Provider::OpenRouter)?))
}

#[tauri::command]
pub fn cloud_key_delete() -> Result<(), String> {
    cloud_key::delete_key_for(Provider::OpenRouter)
}

// Speech is a separate service from the one the device thinks with, and
// a separate decision for the owner: a device can perfectly well think
// on its own hardware and still speak, or think in the cloud and stay
// silent. Same three commands, same posture, its own key.

#[tauri::command]
pub fn voice_key_save(key: String) -> Result<(), String> {
    cloud_key::save_key_for(Provider::ElevenLabs, &key)
}

#[tauri::command]
pub fn voice_key_status() -> Result<String, String> {
    Ok(status_word(cloud_key::key_status_for(Provider::ElevenLabs)?))
}

#[tauri::command]
pub fn voice_key_delete() -> Result<(), String> {
    cloud_key::delete_key_for(Provider::ElevenLabs)
}
