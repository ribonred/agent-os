//! Whether the runtime stops the agent and asks the owner before running
//! a command it considers dangerous.
//!
//! The setting lives in the agent runtime's own config file, not in the
//! shell's store, because the runtime is what enforces it -- a copy here
//! would be a second answer to the same question and the two would drift.
//! The file is machine-generated at build time with a known shape and
//! carries comments explaining the posture, so this edits the one line it
//! owns rather than parsing and re-serialising the document.
//!
//! The gateway keys its config cache on the file's modification time and
//! size, so a write here is picked up on the next turn. Nothing restarts.

use std::path::PathBuf;

/// The two values this device offers. "smart" also exists upstream and
/// is deliberately not offered: it runs an auxiliary model judgement per
/// flagged command, which is a second inference on the critical path and
/// behaves differently depending on whether the device routed local or
/// cloud. An appliance should be predictable about when it interrupts.
const ASKING: &str = "manual";
const NOT_ASKING: &str = "off";

/// The agent runtime's config, resolved the same way its credentials
/// are: an explicit override first, then the device layout, then a
/// developer's own install.
fn config_path() -> Result<PathBuf, String> {
    if let Some(explicit) = std::env::var_os("AGENTIC_OS_HERMES_CONFIG") {
        return Ok(PathBuf::from(explicit));
    }
    let home = std::env::var_os("HOME").ok_or_else(|| "no home directory".to_string())?;
    Ok(PathBuf::from(home).join(".hermes/config.yaml"))
}

/// Finds the `mode:` line inside the `approvals:` block.
///
/// Returns the line's index and its current value. A top-level key is any
/// line starting in column zero, so the block ends at the next one --
/// which is how a `mode:` belonging to some other section is never
/// mistaken for this one.
fn find_mode_line(config: &str) -> Option<(usize, String)> {
    let mut in_approvals = false;
    for (index, line) in config.lines().enumerate() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        let is_top_level = !line.starts_with([' ', '\t']);
        if is_top_level {
            in_approvals = trimmed.starts_with("approvals:");
            continue;
        }
        if in_approvals {
            if let Some(value) = trimmed.strip_prefix("mode:") {
                let value = value
                    .split('#')
                    .next()
                    .unwrap_or("")
                    .trim()
                    .trim_matches(['"', '\''])
                    .to_string();
                return Some((index, value));
            }
        }
    }
    None
}

/// The mode with one line rewritten, preserving that line's indentation
/// and everything else in the file byte for byte.
fn with_mode(config: &str, mode: &str) -> Result<String, String> {
    let (target, _) = find_mode_line(config).ok_or_else(|| {
        "the agent's configuration has no approvals mode to change".to_string()
    })?;

    let mut lines: Vec<String> = config.lines().map(str::to_string).collect();
    let indent: String = lines[target]
        .chars()
        .take_while(|c| c.is_whitespace())
        .collect();
    // Quoted on purpose: YAML 1.1 reads a bare `off` as the boolean
    // false, and the runtime then has to guess what was meant.
    lines[target] = format!("{indent}mode: \"{mode}\"");

    let mut rewritten = lines.join("\n");
    if config.ends_with('\n') {
        rewritten.push('\n');
    }
    Ok(rewritten)
}

/// Whether the owner has asked to be consulted before risky commands.
///
/// A config that cannot be read is reported as not asking rather than as
/// an error: the setting screen should show the device's actual posture,
/// and a device whose runtime config is unreadable is not asking anyone
/// anything.
#[tauri::command]
pub fn approval_mode_get() -> bool {
    let Ok(path) = config_path() else {
        return false;
    };
    let Ok(config) = std::fs::read_to_string(path) else {
        return false;
    };
    find_mode_line(&config).is_some_and(|(_, mode)| mode == ASKING)
}

#[tauri::command]
pub fn approval_mode_set(enabled: bool) -> Result<(), String> {
    let path = config_path()?;
    let config = std::fs::read_to_string(&path)
        .map_err(|e| format!("could not read the agent's configuration: {e}"))?;
    let rewritten = with_mode(&config, if enabled { ASKING } else { NOT_ASKING })?;
    // Fail before writing rather than leaving the runtime a config it
    // cannot parse: a device whose agent will not start is unusable in a
    // way the setting it was toggling never was.
    if find_mode_line(&rewritten).is_none() {
        return Err("the rewritten configuration lost its approvals mode".to_string());
    }
    std::fs::write(&path, rewritten)
        .map_err(|e| format!("could not save the agent's configuration: {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    const CONFIG: &str = "terminal:\n  cwd: /home/owner\n\n\
                          # Commentary that has to survive.\n\
                          approvals:\n  mode: \"off\"\n  deny:\n    - \"*secret*\"\n";

    #[test]
    fn the_mode_is_found_inside_its_own_block() {
        assert_eq!(find_mode_line(CONFIG), Some((5, "off".to_string())));
    }

    #[test]
    fn another_sections_mode_is_never_mistaken_for_this_one() {
        let config = "model:\n  mode: chat\n\napprovals:\n  mode: \"off\"\n";
        assert_eq!(find_mode_line(config), Some((4, "off".to_string())));

        // And a file with no approvals block at all has nothing to change.
        assert_eq!(find_mode_line("model:\n  mode: chat\n"), None);
        assert!(with_mode("model:\n  mode: chat\n", ASKING).is_err());
    }

    #[test]
    fn turning_it_on_rewrites_one_line_and_nothing_else() {
        let rewritten = with_mode(CONFIG, ASKING).expect("should rewrite");
        assert!(rewritten.contains("  mode: \"manual\""));
        assert!(!rewritten.contains("\"off\""));
        // Everything the file said about why stays said.
        assert!(rewritten.contains("# Commentary that has to survive."));
        assert!(rewritten.contains("    - \"*secret*\""));
        assert!(rewritten.contains("  cwd: /home/owner"));
        assert!(rewritten.ends_with('\n'));

        // And it round-trips: what was written reads back as what was set.
        assert_eq!(
            find_mode_line(&rewritten),
            Some((5, "manual".to_string()))
        );
        let back = with_mode(&rewritten, NOT_ASKING).expect("should rewrite");
        assert_eq!(find_mode_line(&back), Some((5, "off".to_string())));
    }

    #[test]
    fn the_value_stays_quoted() {
        // Unquoted, YAML 1.1 reads `off` as the boolean false and the
        // runtime is left guessing what the owner meant.
        let rewritten = with_mode(CONFIG, NOT_ASKING).expect("should rewrite");
        assert!(rewritten.contains("mode: \"off\""), "{rewritten}");
    }

    /// Against the agent runtime's real config on this machine, which
    /// is written by the installer and then rewritten by the runtime
    /// itself -- neither of which is the shape a fixture guesses at.
    /// Restores whatever it found. Opt-in: cargo test -- --ignored
    #[test]
    #[ignore]
    fn the_real_config_toggles_and_comes_back() {
        let path = config_path().expect("no config path");
        let original = std::fs::read_to_string(&path).expect("no config to read");
        let was_asking = approval_mode_get();

        approval_mode_set(true).expect("could not turn it on");
        assert!(approval_mode_get(), "turning it on did not take");
        approval_mode_set(false).expect("could not turn it off");
        assert!(!approval_mode_get(), "turning it off did not take");

        // Only the one line may ever have moved.
        approval_mode_set(was_asking).expect("could not restore");
        let restored = std::fs::read_to_string(&path).expect("read back");
        let differing = original
            .lines()
            .zip(restored.lines())
            .filter(|(a, b)| a != b)
            .count();
        assert_eq!(
            original.lines().count(),
            restored.lines().count(),
            "the config changed length"
        );
        assert!(differing <= 1, "{differing} lines differ after a round trip");
    }

    #[test]
    fn an_already_unquoted_or_commented_value_is_still_read() {
        let config = "approvals:\n  mode: manual # set from the settings screen\n";
        assert_eq!(find_mode_line(config), Some((1, "manual".to_string())));
    }
}
