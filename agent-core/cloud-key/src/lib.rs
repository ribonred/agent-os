//! Cloud API key storage: OS keyring for user-entered keys, plus an
//! optional vendor-provisioned key file.
//!
//! One implementation, two consumers: the Tauri UI shell (save/delete/
//! status commands -- the key itself never crosses into the webview) and
//! the orchestrator daemon (the actual key, via `resolve_openrouter_key`,
//! to call the cloud provider).
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
//! File format is per-provider so future providers slot in without a
//! format change:
//!
//! ```toml
//! [openrouter]
//! api_key = "sk-or-v1-..."
//! ```
//!
//! Dev override: `AGENTIC_OS_CLOUD_KEYS_FILE` points at an alternative
//! file path (useful where no keyring daemon runs, e.g. bare WSL).

use std::path::PathBuf;

use keyring::{Entry, Error as KeyringError};
use serde::Deserialize;

const SERVICE: &str = "com.agenticos.shell";
const USER: &str = "openrouter-api-key";
const PROVISIONED_KEYS_PATH: &str = "/etc/agentic-os/cloud-keys.toml";
const PROVISIONED_KEYS_ENV: &str = "AGENTIC_OS_CLOUD_KEYS_FILE";

/// Which source is currently supplying the key. This -- never the key
/// itself -- is all the UI layer is allowed to learn.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeySource {
    Keyring,
    Provisioned,
}

#[derive(Deserialize)]
struct ProvisionedKeys {
    openrouter: Option<ProviderKey>,
}

#[derive(Deserialize)]
struct ProviderKey {
    api_key: String,
}

fn entry() -> Result<Entry, String> {
    Entry::new(SERVICE, USER).map_err(|e| format!("keyring unavailable: {e}"))
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
fn parse_provisioned_openrouter_key(contents: &str) -> Result<Option<String>, String> {
    let parsed: ProvisionedKeys =
        toml::from_str(contents).map_err(|e| format!("provisioned key file is invalid: {e}"))?;
    Ok(parsed
        .openrouter
        .map(|p| p.api_key.trim().to_string())
        .filter(|k| !k.is_empty()))
}

/// Reads the provisioned key, if the file exists and has one. A missing
/// file is the normal case (nothing provisioned), not an error. A file
/// that exists but cannot be read or parsed IS an error -- that's a
/// broken provisioning that someone needs to hear about, not silently
/// treat as "no key".
fn provisioned_openrouter_key() -> Result<Option<String>, String> {
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
    parse_provisioned_openrouter_key(&contents)
}

// Read posture differs from write posture on purpose. A dead or missing
// keyring daemon must not brick a device whose key was factory-
// provisioned -- reads log the failure loudly and fall through to the
// provisioned file. Writes (save/delete below) still fail hard: the user
// is actively storing a secret and must hear that it didn't happen.
fn keyring_openrouter_key() -> Option<String> {
    let entry = match Entry::new(SERVICE, USER) {
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

/// The actual key, for consumers that call the cloud provider (the
/// orchestrator). Keyring wins over provisioned file.
pub fn resolve_openrouter_key() -> Result<Option<String>, String> {
    if let Some(key) = keyring_openrouter_key() {
        return Ok(Some(key));
    }
    provisioned_openrouter_key()
}

/// Which source is active, never the key. Safe to expose to UI layers.
pub fn key_status() -> Result<Option<KeySource>, String> {
    if keyring_openrouter_key().is_some() {
        return Ok(Some(KeySource::Keyring));
    }
    if provisioned_openrouter_key()?.is_some() {
        return Ok(Some(KeySource::Provisioned));
    }
    Ok(None)
}

/// Stores a user-entered key in the OS keyring. Rejects empty input.
pub fn save_key(key: &str) -> Result<(), String> {
    let key = key.trim();
    if key.is_empty() {
        return Err("API key must not be empty".to_string());
    }
    entry()?
        .set_password(key)
        .map_err(|e| format!("could not store the key: {e}"))
}

/// Removes the user's keyring entry. Idempotent: deleting a key that
/// isn't there is not an error -- the end state the caller asked for
/// already holds. Never touches the provisioned file.
pub fn delete_key() -> Result<(), String> {
    match entry()?.delete_credential() {
        Ok(()) | Err(KeyringError::NoEntry) => Ok(()),
        Err(e) => Err(format!("could not delete the key: {e}")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_a_valid_provisioned_key() {
        let contents = "[openrouter]\napi_key = \"sk-or-v1-abc123\"\n";
        assert_eq!(
            parse_provisioned_openrouter_key(contents).unwrap(),
            Some("sk-or-v1-abc123".to_string())
        );
    }

    #[test]
    fn missing_provider_section_is_no_key_not_an_error() {
        assert_eq!(parse_provisioned_openrouter_key("").unwrap(), None);
        assert_eq!(
            parse_provisioned_openrouter_key("[other_provider]\napi_key = \"x\"\n").unwrap(),
            None
        );
    }

    #[test]
    fn blank_key_counts_as_absent_not_configured() {
        let contents = "[openrouter]\napi_key = \"   \"\n";
        assert_eq!(parse_provisioned_openrouter_key(contents).unwrap(), None);
    }

    #[test]
    fn malformed_toml_is_a_loud_error_not_silently_no_key() {
        let result = parse_provisioned_openrouter_key("[openrouter\napi_key =");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("invalid"));
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
        let got = provisioned_openrouter_key().unwrap();
        unsafe { std::env::remove_var(PROVISIONED_KEYS_ENV) };
        std::fs::remove_dir_all(&dir).ok();

        assert_eq!(got, Some("sk-test-999".to_string()));
    }
}
