//! Shim and install helpers for routing.

use crate::execution::routing::path_env::canonicalize_or;
use crate::execution::ShellConfig;
use std::env;
use std::path::{Path, PathBuf};
use substrate_common::paths as substrate_paths;

pub(crate) fn build_world_env_map() -> std::collections::HashMap<String, String> {
    use std::collections::HashMap;

    let mut env_map: HashMap<String, String> = std::env::vars().collect();

    if let Ok(original_path) = std::env::var("SHIM_ORIGINAL_PATH") {
        env_map.insert("PATH".to_string(), original_path.clone());
        #[cfg(windows)]
        {
            env_map.insert("Path".to_string(), original_path);
        }
    } else if let Ok(shim_dir) = substrate_paths::shims_dir() {
        if let Some(current_path) = env_map.get("PATH").cloned() {
            let separator = if cfg!(windows) { ';' } else { ':' };
            let shim_str = shim_dir.to_string_lossy();
            let filtered: String = current_path
                .split(separator)
                .filter(|segment| segment != &shim_str)
                .filter(|segment| !segment.is_empty())
                .collect::<Vec<&str>>()
                .join(&separator.to_string());
            env_map.insert("PATH".to_string(), filtered);
        }
    }

    for key in [
        "SHIM_ACTIVE",
        "SHIM_CALLER",
        "SHIM_CALL_STACK",
        "SHIM_DEPTH",
    ] {
        env_map.remove(key);
    }

    env_map
}

pub(crate) fn wrap_with_anchor_guard(command: &str, config: &ShellConfig) -> String {
    let current_dir = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let anchor = canonicalize_or(&config.world_root.anchor_root(&current_dir));
    let anchor_escaped = shell_escape_for_shell(&anchor);
    let mut guarded = format!(
        "__substrate_anchor_root={anchor}; \
         substrate_anchor_builtin_cd() {{ if builtin cd \"$@\" 2>/dev/null; then :; else command cd \"$@\"; fi; }}; \
         substrate_anchor_cd() {{ substrate_anchor_builtin_cd \"$@\" || return $?; dest=$(pwd -P); case \"$dest\" in \"$__substrate_anchor_root\"|\"$__substrate_anchor_root\"/*) ;; *) printf 'substrate: info: caged root guard: returning to %s\\n' \"$__substrate_anchor_root\" >&2; substrate_anchor_builtin_cd \"$__substrate_anchor_root\" || return $?;; esac; unset dest; }}; \
         cd() {{ substrate_anchor_cd \"$@\"; }}; \
         substrate_anchor_cd .; ",
        anchor = anchor_escaped,
    );
    guarded.push_str(command);
    guarded
}

fn shell_escape_for_shell(path: &Path) -> String {
    let raw = path.to_string_lossy();
    if raw.contains('\'') {
        format!("'{}'", raw.replace('\'', "'\"'\"'"))
    } else {
        format!("'{raw}'")
    }
}
