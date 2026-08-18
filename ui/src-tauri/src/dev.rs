//! Putting a development device back to the state it shipped in.
//!
//! Setup is meant to happen once, and everything about it is built to be
//! hard to undo: the shell records that it finished, and the agent writes
//! the owner's profile into its own long-term memory. That is right for a
//! customer and useless for anyone working on the setup conversation
//! itself, who needs to see the first-boot experience more than once.
//!
//! This is compiled out of a release build entirely -- there is no
//! shipped code path to reach it, not merely an unreachable button -- and
//! is refused even in a debug build unless the environment asks for it.
//! A "start over" the owner can reach is a different feature with
//! different consequences (it discards everything the device has learned)
//! and is not this.

/// Everything the shell records about setup. Kept as one list so a new
/// setup key cannot be added without this being the obvious place it also
/// has to appear.
#[cfg(debug_assertions)]
const SETUP_KEYS: &[&str] = &[
    "language",
    "persona",
    "agentName",
    "onboardingStarted",
    "onboardingQuestionCount",
    "onboardingComplete",
    "onboardingSessionId",
];

/// Clears the shell's half of setup so the next launch opens on the
/// language screen.
///
/// The agent's half -- the profile it committed to its own memory -- is
/// not reachable from here and is not cleared: a reset that clears only
/// the shell leaves a device that interviews an owner it still remembers,
/// which is a worse state than either end of the switch. The Makefile
/// target clears both, and is the supported way to do this.
#[cfg(debug_assertions)]
#[tauri::command]
pub fn dev_reset_setup(app: tauri::AppHandle) -> Result<(), String> {
    use tauri_plugin_store::StoreExt;

    if std::env::var("AGENTIC_OS_DEV").as_deref() != Ok("1") {
        return Err("development reset is not enabled".to_string());
    }

    let store = app
        .store("settings.json")
        .map_err(|e| format!("could not open the settings store: {e}"))?;
    for key in SETUP_KEYS {
        store.delete(*key);
    }
    store
        .save()
        .map_err(|e| format!("could not save the settings store: {e}"))?;

    log::warn!("setup state cleared by development reset");
    Ok(())
}

/// The release build's stand-in: the command exists in the handler list
/// either way, so the frontend has one shape to call, and this one always
/// refuses. Nothing about a shipped device can clear the owner's setup.
#[cfg(not(debug_assertions))]
#[tauri::command]
pub fn dev_reset_setup(_app: tauri::AppHandle) -> Result<(), String> {
    Err("development reset is not enabled".to_string())
}

#[cfg(all(test, debug_assertions))]
mod tests {
    use super::*;

    #[test]
    fn every_key_setup_writes_is_cleared() {
        // The gate in setupStore.ts reads exactly these; one missing key
        // leaves a device that skips a screen it should show again.
        for key in [
            "language",
            "persona",
            "agentName",
            "onboardingStarted",
            "onboardingQuestionCount",
            "onboardingComplete",
            "onboardingSessionId",
        ] {
            assert!(SETUP_KEYS.contains(&key), "{key} would survive a reset");
        }
    }
}
