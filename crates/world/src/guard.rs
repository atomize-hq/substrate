//! Helpers for enforcing caged root guards inside the world backend.

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use substrate_common::WorldRootMode;

const ANCHOR_MODE_ENV: &str = "SUBSTRATE_ANCHOR_MODE";
const LEGACY_ROOT_MODE_ENV: &str = "SUBSTRATE_WORLD_ROOT_MODE";
const CAGED_ENV: &str = "SUBSTRATE_CAGED";

/// Return true when the current execution should enforce the caged guard.
pub fn should_guard_anchor(env: &HashMap<String, String>) -> bool {
    let caged = env
        .get(CAGED_ENV)
        .and_then(|raw| parse_bool(raw))
        .unwrap_or(true);
    if !caged {
        return false;
    }

    let mode = env
        .get(ANCHOR_MODE_ENV)
        .or_else(|| env.get(LEGACY_ROOT_MODE_ENV))
        .and_then(|value| WorldRootMode::parse(value))
        .unwrap_or(WorldRootMode::Project);

    mode != WorldRootMode::FollowCwd
}

/// Wrap a shell command with guards that prevent leaving `anchor_root`.
pub fn wrap_with_anchor_guard(command: &str, anchor_root: &Path) -> String {
    let anchor = canonicalize_or(anchor_root);
    let display_anchor = shell_escape_for_sh(&anchor);
    let display_name = if anchor.starts_with("/var/lib/substrate/overlay") {
        "[Substrate World]"
    } else {
        "[Substrate Host]"
    };
    let mut guarded = format!(
        "__substrate_anchor_root={anchor}; \
         __substrate_anchor_display={display}; \
         substrate_anchor_builtin_cd() {{ if builtin cd \"$@\" 2>/dev/null; then :; else command cd \"$@\"; fi; }}; \
         substrate_anchor_cd() {{ substrate_anchor_builtin_cd \"$@\" || return $?; dest=$(pwd -P); case \"$dest\" in \"$__substrate_anchor_root\"|\"$__substrate_anchor_root\"/*) ;; *) printf 'substrate: info: caged root guard: returning to %s (%s)\\n' \"$__substrate_anchor_root\" \"$__substrate_anchor_display\" >&2; substrate_anchor_builtin_cd \"$__substrate_anchor_root\" || return $?;; esac; unset dest; }}; \
         cd() {{ substrate_anchor_cd \"$@\"; }}; \
         substrate_anchor_cd .; ",
        anchor = display_anchor,
        display = display_name
    );
    guarded.push_str(command);
    guarded
}

fn canonicalize_or(path: &Path) -> PathBuf {
    fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf())
}

fn parse_bool(raw: &str) -> Option<bool> {
    match raw.trim().to_ascii_lowercase().as_str() {
        "1" | "true" | "yes" | "on" => Some(true),
        "0" | "false" | "no" | "off" => Some(false),
        _ => None,
    }
}

fn shell_escape_for_sh(path: &Path) -> String {
    let raw = path.to_string_lossy();
    if raw.contains('\'') {
        format!("'{}'", raw.replace('\'', "'\"'\"'"))
    } else {
        format!("'{raw}'")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn guard_enabled_by_default() {
        let env = HashMap::new();
        assert!(should_guard_anchor(&env));
    }

    #[test]
    fn guard_disabled_when_uncaged() {
        let mut env = HashMap::new();
        env.insert(CAGED_ENV.to_string(), "0".to_string());
        assert!(!should_guard_anchor(&env));
    }

    #[test]
    fn guard_disabled_for_follow_cwd() {
        let mut env = HashMap::new();
        env.insert(ANCHOR_MODE_ENV.to_string(), "follow-cwd".to_string());
        assert!(!should_guard_anchor(&env));
    }

    #[test]
    fn guard_wrapping_includes_anchor_path() {
        let temp = tempdir().unwrap();
        let command = "pwd && cd ..";
        let wrapped = wrap_with_anchor_guard(command, temp.path());
        assert!(
            wrapped.contains(temp.path().to_str().unwrap()),
            "wrapped command missing anchor path: {wrapped}"
        );
        assert!(wrapped.ends_with(command));
    }
}
