//! The agent runtime's own config file, and the one-line edits the shell
//! makes to it.
//!
//! Settings the runtime enforces live there rather than in the shell's
//! store: a copy here would be a second answer to the same question, and
//! the two would drift. That is not hypothetical -- the model the shell
//! stamped on every session drifted away from the model this file names,
//! and nothing reported it for as long as the shell kept its own copy.
//!
//! The file is machine-generated at build time with a known shape and
//! carries comments explaining the posture, so this edits the one line it
//! owns rather than parsing and re-serialising the document. Round-tripping
//! it through a YAML library would silently drop every one of those
//! comments, which are the only explanation the next person gets.
//!
//! The gateway keys its config cache on the file's modification time and
//! size, so a write here is picked up on the next turn. Nothing restarts.

use std::path::PathBuf;

/// The agent runtime's config, resolved the same way its credentials
/// are: an explicit override first, then the device layout, then a
/// developer's own install.
pub(crate) fn config_path() -> Result<PathBuf, String> {
    if let Some(explicit) = std::env::var_os("AGENTIC_OS_HERMES_CONFIG") {
        return Ok(PathBuf::from(explicit));
    }
    let home = std::env::var_os("HOME").ok_or_else(|| "no home directory".to_string())?;
    Ok(PathBuf::from(home).join(".hermes/config.yaml"))
}

/// Finds `key:` inside the named top-level block.
///
/// Returns the line's index and its current value. A top-level key is any
/// line starting in column zero, so the block ends at the next one --
/// which is how a key belonging to some other section is never mistaken
/// for this one. `model:` and `approvals:` both contain short keys that
/// would otherwise collide.
pub(crate) fn find_key(config: &str, block: &str, key: &str) -> Option<(usize, String)> {
    let block_header = format!("{block}:");
    let key_prefix = format!("{key}:");
    let mut inside = false;

    for (index, line) in config.lines().enumerate() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        let is_top_level = !line.starts_with([' ', '\t']);
        if is_top_level {
            inside = trimmed.starts_with(&block_header);
            continue;
        }
        if inside {
            if let Some(value) = trimmed.strip_prefix(&key_prefix) {
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

/// The config with one line rewritten, preserving that line's indentation
/// and everything else in the file byte for byte.
///
/// The value is always quoted. YAML 1.1 reads a bare `off` as the boolean
/// false, and a model id is full of characters -- slashes, colons, dots --
/// that are better not left to a parser's judgement.
pub(crate) fn with_key(
    config: &str,
    block: &str,
    key: &str,
    value: &str,
) -> Result<String, String> {
    let (target, _) = find_key(config, block, key)
        .ok_or_else(|| format!("the agent's configuration has no {block} {key} to change"))?;

    let mut lines: Vec<String> = config.lines().map(str::to_string).collect();
    let indent: String = lines[target]
        .chars()
        .take_while(|c| c.is_whitespace())
        .collect();
    lines[target] = format!("{indent}{key}: \"{value}\"");

    let mut rewritten = lines.join("\n");
    if config.ends_with('\n') {
        rewritten.push('\n');
    }
    Ok(rewritten)
}

/// Read one value out of the runtime's config on disk.
pub(crate) fn read_key(block: &str, key: &str) -> Result<String, String> {
    let path = config_path()?;
    let config = std::fs::read_to_string(&path)
        .map_err(|e| format!("could not read the agent's configuration: {e}"))?;
    find_key(&config, block, key)
        .map(|(_, value)| value)
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| format!("the agent's configuration does not set {block}.{key}"))
}

/// Write one value into the runtime's config on disk.
///
/// Fails before writing rather than leaving the runtime a config it
/// cannot parse: a device whose agent will not start is unusable in a way
/// the setting it was changing never was.
pub(crate) fn write_key(block: &str, key: &str, value: &str) -> Result<(), String> {
    let path = config_path()?;
    let config = std::fs::read_to_string(&path)
        .map_err(|e| format!("could not read the agent's configuration: {e}"))?;
    let rewritten = with_key(&config, block, key, value)?;
    if find_key(&rewritten, block, key).map(|(_, v)| v).as_deref() != Some(value) {
        return Err(format!(
            "the rewritten configuration did not keep {block}.{key}"
        ));
    }
    std::fs::write(&path, rewritten)
        .map_err(|e| format!("could not save the agent's configuration: {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    const CONFIG: &str = "model:\n  provider: openrouter\n  default: z-ai/glm-5.3\n\n\
                          # Commentary that has to survive.\n\
                          approvals:\n  mode: \"off\"\n";

    #[test]
    fn a_key_is_found_only_inside_its_own_block() {
        assert_eq!(
            find_key(CONFIG, "model", "default"),
            Some((2, "z-ai/glm-5.3".to_string()))
        );
        assert_eq!(find_key(CONFIG, "approvals", "mode"), Some((6, "off".to_string())));
        // The same key name under a different block is a different key.
        assert_eq!(find_key(CONFIG, "approvals", "default"), None);
        assert_eq!(find_key(CONFIG, "model", "mode"), None);
    }

    #[test]
    fn rewriting_touches_one_line_and_leaves_the_comments() {
        let out = with_key(CONFIG, "model", "default", "anthropic/claude-opus-5")
            .expect("should rewrite");
        assert!(out.contains("  default: \"anthropic/claude-opus-5\""));
        assert!(out.contains("# Commentary that has to survive."));
        assert!(out.contains("  provider: openrouter"));
        assert!(out.contains("  mode: \"off\""));
        assert!(out.ends_with('\n'));
        assert_eq!(out.lines().count(), CONFIG.lines().count());
    }

    #[test]
    fn a_model_id_survives_the_round_trip_intact() {
        // Slashes, dots, dashes and a colon variant suffix -- all of which
        // a bare YAML scalar would leave to the parser's judgement.
        for id in [
            "z-ai/glm-5.3",
            "anthropic/claude-opus-5-fast",
            "nvidia/nemotron-3-ultra-550b-a55b:free",
        ] {
            let out = with_key(CONFIG, "model", "default", id).expect("should rewrite");
            assert_eq!(
                find_key(&out, "model", "default"),
                Some((2, id.to_string())),
                "{id} did not survive"
            );
        }
    }

    #[test]
    fn a_config_without_the_block_is_an_error_not_a_guess() {
        assert!(with_key("terminal:\n  cwd: /home/owner\n", "model", "default", "x").is_err());
    }
}
