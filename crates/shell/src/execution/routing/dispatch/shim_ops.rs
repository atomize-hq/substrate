//! Shim and install helpers for routing.

use crate::execution::config_model::{self, CliConfigOverrides};
use crate::execution::routing::path_env::canonicalize_or;
use crate::execution::ShellConfig;
use anyhow::Context;
use std::env;
use std::path::{Path, PathBuf};
use substrate_common::paths as substrate_paths;

pub(crate) fn build_world_env_map_for_cwd(
    cwd: &Path,
) -> anyhow::Result<(std::collections::HashMap<String, String>, bool)> {
    use std::collections::HashMap;

    const DEFAULT_WORLD_DEPS_BIN: &str = "/var/lib/substrate/world-deps/bin";
    const BASELINE_PATH: &str = "/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin";

    let effective = config_model::resolve_effective_config(cwd, &CliConfigOverrides::default())
        .with_context(|| {
            format!(
                "failed to resolve effective config for world env (cwd={})",
                cwd.display()
            )
        })?;
    let inherit_from_host = effective.world.env.inherit_from_host;

    let world_deps_bin = env::var("SUBSTRATE_WORLD_DEPS_GUEST_BIN_DIR")
        .ok()
        .map(|v| v.trim().to_string())
        .filter(|v| !v.is_empty())
        .unwrap_or_else(|| DEFAULT_WORLD_DEPS_BIN.to_string());

    let mut env_map: HashMap<String, String> = HashMap::new();
    env_map.insert(
        "SUBSTRATE_WORLD_DEPS_GUEST_BIN_DIR".to_string(),
        world_deps_bin.clone(),
    );
    env_map.insert(
        "PATH".to_string(),
        format!("{}:{}", world_deps_bin.trim_end_matches('/'), BASELINE_PATH),
    );
    env_map.insert("HOME".to_string(), "/root".to_string());
    env_map.insert("XDG_CONFIG_HOME".to_string(), "/root/.config".to_string());
    env_map.insert(
        "XDG_DATA_HOME".to_string(),
        "/root/.local/share".to_string(),
    );
    env_map.insert("XDG_CACHE_HOME".to_string(), "/root/.cache".to_string());
    env_map.insert("TERM".to_string(), "xterm-256color".to_string());

    // Preserve Substrate/WORLD control-plane env so policy/config state is visible to world-agent
    // and downstream shims, without inheriting arbitrary host user environment.
    for (key, value) in std::env::vars() {
        let keep = key.starts_with("SUBSTRATE_") || key.starts_with("WORLD_");
        if !keep || value.is_empty() {
            continue;
        }
        match key.as_str() {
            // Reserved keys are owned by the deterministic world env contract above.
            "PATH" | "HOME" | "TERM" | "SUBSTRATE_WORLD_DEPS_GUEST_BIN_DIR" => continue,
            _ if key.starts_with("XDG_") => continue,
            // Shim bookkeeping must never leak into the world environment.
            "SHIM_ACTIVE" | "SHIM_CALLER" | "SHIM_CALL_STACK" | "SHIM_DEPTH" => continue,
            _ => {}
        }
        env_map.insert(key, value);
    }

    if inherit_from_host {
        for (key, value) in std::env::vars() {
            let forward =
                key == "LANG" || key == "TZ" || key == "NO_COLOR" || key.starts_with("LC_");
            if forward && !value.is_empty() {
                env_map.insert(key, value);
            }
        }
    }

    // Always ensure shim/runtime book-keeping does not leak into the world environment.
    for key in [
        "SHIM_ACTIVE",
        "SHIM_CALLER",
        "SHIM_CALL_STACK",
        "SHIM_DEPTH",
    ] {
        env_map.remove(key);
    }

    // Best-effort: keep substrate shims out of PATH if a caller injected them.
    if let Ok(shim_dir) = substrate_paths::shims_dir() {
        if let Some(current_path) = env_map.get("PATH").cloned() {
            let separator = ':';
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

    Ok((env_map, inherit_from_host))
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
