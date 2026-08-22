//! Cloud API key storage: OS keyring for user-entered keys, plus an
//! optional vendor-provisioned key file.
//!
//! One implementation, shared by every consumer: the Tauri UI shell
//! (save/delete/status commands -- the key itself never crosses into the
//! webview) and any Rust-side process that needs the actual key (via
//! `resolve_key`) to call the provider.
//!
//! Two sources, one clear precedence:
//!
//! 1. **Keyring** (Secret Service on Linux) -- the user's own key,
//!    entered through the UI. An explicit user action, so it wins.
//! 2. **Provisioned file** -- `/etc/agentic-os/cloud-keys.toml`, written
//!    at deployment/factory time (root-owned, mode 0600) so a device can
//!    ship with cloud access already working and the buyer never has to
//!    know what an API key is. Fallback when no keyring entry exists;
//!    "disconnect" in the UI removes the keyring entry and falls back
//!    here, it cannot delete the file.
//!
//! One section per provider, so a device can be provisioned for some and
//! not others, and each is resolved entirely independently:
//!
//! ```toml
//! [openrouter]
//! api_key = "sk-or-v1-..."
//!
//! [elevenlabs]
//! api_key = "sk_..."
//! ```
//!
//! Dev override: `AGENTIC_OS_CLOUD_KEYS_FILE` points at an alternative
//! file path (useful where no keyring daemon runs, e.g. bare WSL).

use std::collections::HashMap;
use std::path::PathBuf;

use keyring::{Entry, Error as KeyringError};
use serde::Deserialize;

const SERVICE: &str = "com.agenticos.shell";
const PROVISIONED_KEYS_PATH: &str = "/etc/agentic-os/cloud-keys.toml";
const PROVISIONED_KEYS_ENV: &str = "AGENTIC_OS_CLOUD_KEYS_FILE";

/// Which service a key belongs to. Adding one is a variant plus its two
/// names -- everything below is written against this rather than against
/// a particular provider, so a second provider cannot arrive with its
/// own subtly different precedence rules.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Provider {
    /// The model aggregator the device thinks through when it thinks
    /// off-device.
    OpenRouter,
    /// Speech: what the device hears with and what it speaks with.
    ElevenLabs,
}

impl Provider {
    /// The section name in the provisioned file.
    fn section(self) -> &'static str {
        match self {
            Provider::OpenRouter => "openrouter",
            Provider::ElevenLabs => "elevenlabs",
        }
    }

    /// The keyring entry's username. Stable across releases: changing one
    /// would silently orphan a key the owner already stored, and look
    /// exactly like the device forgetting it.
    fn keyring_user(self) -> &'static str {
        match self {
            Provider::OpenRouter => "openrouter-api-key",
            Provider::ElevenLabs => "elevenlabs-api-key",
        }
    }
}

/// Which source is currently supplying the key. This -- never the key
/// itself -- is all the UI layer is allowed to learn.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeySource {
    Keyring,
    Provisioned,
}

/// Deserialized as a map rather than named fields so a file carrying a
/// provider this build has never heard of still parses. A device
/// provisioned for a later release must not fail to read the key it
/// *does* understand because of a section it does not.
#[derive(Deserialize)]
struct ProviderKey {
    api_key: String,
}

fn entry(provider: Provider) -> Result<Entry, String> {
    Entry::new(SERVICE, provider.keyring_user()).map_err(|e| format!("keyring unavailable: {e}"))
}

fn provisioned_keys_path() -> PathBuf {
    std::env::var(PROVISIONED_KEYS_ENV)
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from(PROVISIONED_KEYS_PATH))
}

/// Pure parsing, split out so it's testable without touching the
/// filesystem. Empty/whitespace keys are treated as absent, not valid --
/// a provisioned file with a blank value is a provisioning mistake and
/// must not count as "cloud is configured".
fn parse_provisioned_key(contents: &str, provider: Provider) -> Result<Option<String>, String> {
    let parsed: HashMap<String, ProviderKey> =
        toml::from_str(contents).map_err(|e| format!("provisioned key file is invalid: {e}"))?;
    Ok(parsed
        .get(provider.section())
        .map(|p| p.api_key.trim().to_string())
        .filter(|k| !k.is_empty()))
}

/// Reads the provisioned key, if the file exists and has one. A missing
/// file is the normal case (nothing provisioned), not an error. A file
/// that exists but cannot be read or parsed IS an error -- that's a
/// broken provisioning that someone needs to hear about, not silently
/// treat as "no key".
fn provisioned_key(provider: Provider) -> Result<Option<String>, String> {
    let path = provisioned_keys_path();
    if !path.exists() {
        return Ok(None);
    }

    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        if let Ok(meta) = std::fs::metadata(&path) {
            if meta.mode() & 0o044 != 0 {
                log::warn!(
                    "provisioned key file {} is readable by group/other -- it should be mode 0600",
                    path.display()
                );
            }
        }
    }

    let contents = std::fs::read_to_string(&path)
        .map_err(|e| format!("could not read provisioned key file: {e}"))?;
    parse_provisioned_key(&contents, provider)
}

// Read posture differs from write posture on purpose. A dead or missing
// keyring daemon must not brick a device whose key was factory-
// provisioned -- reads log the failure loudly and fall through to the
// provisioned file. Writes (save/delete below) still fail hard: the user
// is actively storing a secret and must hear that it didn't happen.
fn keyring_key(provider: Provider) -> Option<String> {
    let entry = match Entry::new(SERVICE, provider.keyring_user()) {
        Ok(entry) => entry,
        Err(e) => {
            log::error!("keyring unavailable, falling back to provisioned key file: {e}");
            return None;
        }
    };
    match entry.get_password() {
        Ok(key) => Some(key),
        Err(KeyringError::NoEntry) => None,
        Err(e) => {
            log::error!("keyring read failed, falling back to provisioned key file: {e}");
            None
        }
    }
}

/// The actual key, for consumers that call the provider.
/// Keyring wins over provisioned file.
pub fn resolve_key(provider: Provider) -> Result<Option<String>, String> {
    if let Some(key) = keyring_key(provider) {
        return Ok(Some(key));
    }
    provisioned_key(provider)
}

/// Which source is active, never the key. Safe to expose to UI layers.
pub fn key_status_for(provider: Provider) -> Result<Option<KeySource>, String> {
    if keyring_key(provider).is_some() {
        return Ok(Some(KeySource::Keyring));
    }
    if provisioned_key(provider)?.is_some() {
        return Ok(Some(KeySource::Provisioned));
    }
    Ok(None)
}

/// Stores a user-entered key in the OS keyring. Rejects empty input.
pub fn save_key_for(provider: Provider, key: &str) -> Result<(), String> {
    let key = key.trim();
    if key.is_empty() {
        return Err("API key must not be empty".to_string());
    }
    entry(provider)?
        .set_password(key)
        .map_err(|e| format!("could not store the key: {e}"))
}

/// Removes the user's keyring entry. Idempotent: deleting a key that
/// isn't there is not an error -- the end state the caller asked for
/// already holds. Never touches the provisioned file.
pub fn delete_key_for(provider: Provider) -> Result<(), String> {
    match entry(provider)?.delete_credential() {
        Ok(()) | Err(KeyringError::NoEntry) => Ok(()),
        Err(e) => Err(format!("could not delete the key: {e}")),
    }
}

// The original single-provider surface, kept so callers that only ever
// mean the aggregator read as if there were nothing to choose.

pub fn resolve_openrouter_key() -> Result<Option<String>, String> {
    resolve_key(Provider::OpenRouter)
}

pub fn key_status() -> Result<Option<KeySource>, String> {
    key_status_for(Provider::OpenRouter)
}

pub fn save_key(key: &str) -> Result<(), String> {
    save_key_for(Provider::OpenRouter, key)
}

pub fn delete_key() -> Result<(), String> {
    delete_key_for(Provider::OpenRouter)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_a_valid_provisioned_key() {
        let contents = "[openrouter]\napi_key = \"sk-or-v1-abc123\"\n";
        assert_eq!(
            parse_provisioned_key(contents, Provider::OpenRouter).unwrap(),
            Some("sk-or-v1-abc123".to_string())
        );
    }

    #[test]
    fn missing_provider_section_is_no_key_not_an_error() {
        assert_eq!(
            parse_provisioned_key("", Provider::OpenRouter).unwrap(),
            None
        );
        assert_eq!(
            parse_provisioned_key("[other_provider]\napi_key = \"x\"\n", Provider::OpenRouter)
                .unwrap(),
            None
        );
    }

    #[test]
    fn blank_key_counts_as_absent_not_configured() {
        let contents = "[openrouter]\napi_key = \"   \"\n";
        assert_eq!(
            parse_provisioned_key(contents, Provider::OpenRouter).unwrap(),
            None
        );
    }

    #[test]
    fn malformed_toml_is_a_loud_error_not_silently_no_key() {
        let result = parse_provisioned_key("[openrouter\napi_key =", Provider::OpenRouter);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("invalid"));
    }

    #[test]
    fn each_provider_reads_only_its_own_section() {
        let contents = "[openrouter]\napi_key = \"sk-or-v1-abc\"\n\n\
                        [elevenlabs]\napi_key = \"sk_speech\"\n";
        assert_eq!(
            parse_provisioned_key(contents, Provider::OpenRouter).unwrap(),
            Some("sk-or-v1-abc".to_string())
        );
        assert_eq!(
            parse_provisioned_key(contents, Provider::ElevenLabs).unwrap(),
            Some("sk_speech".to_string())
        );
    }

    /// A unit provisioned for one service and not the other is a normal
    /// unit, not a broken one: voice is optional and so is the cloud.
    #[test]
    fn one_provider_provisioned_leaves_the_other_simply_absent() {
        let contents = "[elevenlabs]\napi_key = \"sk_speech\"\n";
        assert_eq!(
            parse_provisioned_key(contents, Provider::OpenRouter).unwrap(),
            None
        );
        assert_eq!(
            parse_provisioned_key(contents, Provider::ElevenLabs).unwrap(),
            Some("sk_speech".to_string())
        );
    }

    /// A device flashed from an older image must keep working when a
    /// later release adds a section it has never heard of.
    #[test]
    fn an_unknown_section_does_not_hide_a_key_this_build_understands() {
        let contents = "[openrouter]\napi_key = \"sk-or-v1-abc\"\n\n\
                        [some_future_service]\napi_key = \"whatever\"\n";
        assert_eq!(
            parse_provisioned_key(contents, Provider::OpenRouter).unwrap(),
            Some("sk-or-v1-abc".to_string())
        );
    }

    #[test]
    fn providers_do_not_share_a_keyring_entry() {
        assert_ne!(
            Provider::OpenRouter.keyring_user(),
            Provider::ElevenLabs.keyring_user()
        );
    }

    #[test]
    fn provisioned_file_read_respects_env_override_and_full_flow() {
        let dir = std::env::temp_dir().join("agentic-os-cloud-key-test");
        std::fs::create_dir_all(&dir).unwrap();
        let file = dir.join("cloud-keys.toml");
        std::fs::write(&file, "[openrouter]\napi_key = \"sk-test-999\"\n").unwrap();

        // SAFETY: tests in this module that touch this env var run in one
        // process; no other test reads it.
        unsafe { std::env::set_var(PROVISIONED_KEYS_ENV, &file) };
        let got = provisioned_key(Provider::OpenRouter).unwrap();
        unsafe { std::env::remove_var(PROVISIONED_KEYS_ENV) };
        std::fs::remove_dir_all(&dir).ok();

        assert_eq!(got, Some("sk-test-999".to_string()));
    }
}
