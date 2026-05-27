//! Helpers for enforcing caged root guards inside the world backend.

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use substrate_common::WorldRootMode;

const ANCHOR_MODE_ENV: &str = "SUBSTRATE_ANCHOR_MODE";
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
    let display_name = shell_escape_literal(display_name);
    let mut guarded = format!(
        "__substrate_anchor_root={anchor}; \
         __substrate_anchor_display={display}; \
         substrate_anchor_builtin_cd() {{ if builtin cd \"$@\" 2>/dev/null; then :; else command cd \"$@\"; fi; }}; \
         substrate_anchor_cd() {{ substrate_anchor_builtin_cd \"$@\" || return $?; dest=$(pwd -P); case \"$dest\" in \"$__substrate_anchor_root\"|\"$__substrate_anchor_root\"/*) ;; *) printf 'substrate: info: caged root guard: blocked cd to %s (outside %s); returning to %s (%s)\\n' \"$dest\" \"$__substrate_anchor_root\" \"$__substrate_anchor_root\" \"$__substrate_anchor_display\" >&2; substrate_anchor_builtin_cd \"$__substrate_anchor_root\" || return $?;; esac; unset dest; }}; \
         cd() {{ substrate_anchor_cd \"$@\"; }}; \
         substrate_anchor_cd .; ",
        anchor = display_anchor,
        display = display_name
    );
    guarded.push_str(command);
    guarded
}

/// Wrap a shell command with the deterministic world environment contract.
///
/// This is defense-in-depth against shells or service environments that mutate PATH/HOME/XDG/TERM
/// despite the caller providing an explicit env map. The stable contract reference is
/// `docs/reference/config/world.md`.
pub fn wrap_with_world_env_contract(command: &str, env: &HashMap<String, String>) -> String {
    const DEFAULT_WORLD_DEPS_BIN: &str = "/var/lib/substrate/world-deps/bin";
    const BASELINE_PATH: &str = "/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin";

    let world_deps_bin = env
        .get("SUBSTRATE_WORLD_DEPS_GUEST_BIN_DIR")
        .map(|v| v.trim().to_string())
        .filter(|v| !v.is_empty())
        .unwrap_or_else(|| DEFAULT_WORLD_DEPS_BIN.to_string());
    let world_deps_bin_norm = world_deps_bin.trim_end_matches('/').to_string();
    let path = format!("{world_deps_bin_norm}:{BASELINE_PATH}");

    // Keep this as plain POSIX sh so it works uniformly across backends.
    let mut guarded = String::new();
    guarded.push_str("export SUBSTRATE_WORLD_DEPS_GUEST_BIN_DIR=");
    guarded.push_str(&shell_escape_literal(&world_deps_bin_norm));
    guarded.push_str("; ");
    guarded.push_str("export PATH=");
    guarded.push_str(&shell_escape_literal(&path));
    guarded.push_str("; ");
    guarded.push_str("export HOME='/root'; ");
    guarded.push_str("export XDG_CONFIG_HOME='/root/.config'; ");
    guarded.push_str("export XDG_DATA_HOME='/root/.local/share'; ");
    guarded.push_str("export XDG_CACHE_HOME='/root/.cache'; ");
    guarded.push_str("export TERM='xterm-256color'; ");
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

fn shell_escape_literal(raw: &str) -> String {
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
        let expected_anchor = shell_escape_for_sh(&canonicalize_or(temp.path()));
        assert!(
            wrapped.contains(&expected_anchor),
            "wrapped command missing anchor path: {wrapped}"
        );
        assert!(wrapped.ends_with(command));
    }

    #[test]
    fn guard_display_value_is_quoted() {
        let temp = tempdir().unwrap();
        let wrapped = wrap_with_anchor_guard("true", temp.path());
        assert!(
            wrapped.contains("__substrate_anchor_display='[Substrate Host]'"),
            "display string should be single-quoted for safe evaluation: {wrapped}"
        );
    }
}
