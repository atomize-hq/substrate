//! Path resolution helpers for world enable.

use anyhow::{anyhow, bail, Context, Result};
use chrono::Utc;
use std::env;
use std::path::{Component, Path, PathBuf};
use substrate_common::paths as substrate_paths;

pub(super) fn resolve_prefix(explicit: Option<&Path>) -> Result<PathBuf> {
    if let Some(prefix) = explicit {
        return Ok(prefix.to_path_buf());
    }
    if let Ok(prefix) = env::var("SUBSTRATE_PREFIX") {
        return Ok(PathBuf::from(prefix));
    }
    substrate_paths::substrate_home()
        .context("failed to locate Substrate home (override via --prefix or SUBSTRATE_HOME)")
}

pub(super) fn resolve_manager_env_path() -> Result<PathBuf> {
    if let Ok(path) = env::var("SUBSTRATE_MANAGER_ENV") {
        return Ok(PathBuf::from(path));
    }
    Ok(substrate_paths::substrate_home()?.join("manager_env.sh"))
}

pub(super) fn resolve_version_dir(prefix: &Path) -> Result<PathBuf> {
    let bin_name = if cfg!(target_os = "windows") {
        "substrate.exe"
    } else {
        "substrate"
    };
    let bin_path = prefix.join("bin").join(bin_name);
    if !bin_path.exists() {
        bail!(
            "Substrate binary not found at {}. Reinstall or pass --prefix to an existing install.",
            bin_path.display()
        );
    }
    let canonical = bin_path
        .canonicalize()
        .with_context(|| format!("failed to resolve {}", bin_path.display()))?;
    let bin_dir = canonical
        .parent()
        .ok_or_else(|| anyhow!("{} has no parent directory", canonical.display()))?;
    let version_dir = bin_dir
        .parent()
        .ok_or_else(|| anyhow!("{} has no parent directory", bin_dir.display()))?;
    Ok(version_dir.to_path_buf())
}

pub(super) fn locate_helper_script(
    prefix: &Path,
    version_dir: Option<&Path>,
    override_path: Option<PathBuf>,
) -> Result<PathBuf> {
    if let Some(path) = override_path {
        if path.exists() {
            return Ok(path);
        }
        bail!(
            "SUBSTRATE_WORLD_ENABLE_SCRIPT={} does not exist",
            path.display()
        );
    }

    let version_dir =
        version_dir.ok_or_else(|| anyhow!("missing version directory for helper discovery"))?;
    let candidates = [
        version_dir.join("scripts/substrate/world-enable.sh"),
        prefix.join("scripts/substrate/world-enable.sh"),
    ];

    for candidate in candidates {
        if candidate.exists() {
            return Ok(candidate);
        }
    }

    bail!(
        "world-enable helper script not found under {}. Reinstall Substrate to refresh scripts.",
        version_dir.display()
    )
}

pub(super) fn next_log_path(prefix: &Path) -> Result<PathBuf> {
    let log_dir = prefix.join("logs");
    let stamp = Utc::now().format("%Y%m%d-%H%M%S");
    Ok(log_dir.join(format!("world-enable-{}.log", stamp)))
}

pub(super) fn resolve_world_socket_path() -> Option<PathBuf> {
    env::var_os("SUBSTRATE_WORLD_SOCKET")
        .map(PathBuf::from)
        .map(|path| normalize_path(&path))
}

fn normalize_path(path: &Path) -> PathBuf {
    let mut prefix_component: Option<std::ffi::OsString> = None;
    let mut has_root = false;
    let mut parts: Vec<std::ffi::OsString> = Vec::new();

    for component in path.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                if let Some(last) = parts.last() {
                    if last != ".." {
                        parts.pop();
                        continue;
                    }
                }
                if has_root || prefix_component.is_some() {
                    continue;
                }
                parts.push(std::ffi::OsString::from(".."));
            }
            Component::RootDir => {
                has_root = true;
                parts.clear();
            }
            Component::Prefix(prefix) => {
                prefix_component = Some(prefix.as_os_str().to_os_string());
                parts.clear();
            }
            Component::Normal(part) => parts.push(part.to_os_string()),
        }
    }

    let mut normalized = PathBuf::new();
    if let Some(prefix) = prefix_component {
        normalized.push(prefix);
    }
    if has_root {
        normalized.push(Path::new("/"));
    }
    for part in parts {
        normalized.push(part);
    }
    if normalized.as_os_str().is_empty() {
        normalized.push(".");
    }
    normalized
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;
    use std::env;
    use tempfile::tempdir;

    fn set_env(key: &str, value: &Path) -> Option<std::ffi::OsString> {
        let previous = env::var_os(key);
        env::set_var(key, value);
        previous
    }

    fn restore_env(key: &str, value: Option<std::ffi::OsString>) {
        if let Some(val) = value {
            env::set_var(key, val);
        } else {
            env::remove_var(key);
        }
    }

    #[test]
    #[serial]
    fn resolve_prefix_prefers_explicit_then_env_then_home() {
        let temp = tempdir().unwrap();
        let explicit = temp.path().join("explicit");
        let env_prefix = temp.path().join("from-env");
        let home = temp.path().join("home").join(".substrate");

        let prev_prefix = set_env("SUBSTRATE_PREFIX", &env_prefix);
        let prev_home = set_env("SUBSTRATE_HOME", &home);

        assert_eq!(resolve_prefix(Some(explicit.as_ref())).unwrap(), explicit);

        assert_eq!(resolve_prefix(None).unwrap(), env_prefix);

        env::remove_var("SUBSTRATE_PREFIX");
        assert_eq!(resolve_prefix(None).unwrap(), home);

        restore_env("SUBSTRATE_PREFIX", prev_prefix);
        restore_env("SUBSTRATE_HOME", prev_home);
    }

    #[test]
    #[serial]
    fn resolve_manager_env_path_uses_env_or_home() {
        let temp = tempdir().unwrap();
        let custom = temp.path().join("custom/env.sh");
        let prev_manager_env = set_env("SUBSTRATE_MANAGER_ENV", &custom);
        let prev_home = set_env("SUBSTRATE_HOME", temp.path());

        assert_eq!(resolve_manager_env_path().unwrap(), custom);

        restore_env("SUBSTRATE_MANAGER_ENV", prev_manager_env);
        let expected_default = temp.path().join("manager_env.sh");
        assert_eq!(resolve_manager_env_path().unwrap(), expected_default);

        restore_env("SUBSTRATE_HOME", prev_home);
    }

    #[test]
    fn next_log_path_is_namespaced_under_logs_dir() {
        let temp = tempdir().unwrap();
        let log_path = next_log_path(temp.path()).unwrap();
        assert!(log_path.starts_with(temp.path().join("logs")));
        let file_name = log_path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or_default();
        assert!(file_name.starts_with("world-enable-"));
        assert!(file_name.ends_with(".log"));
    }

    #[test]
    #[serial]
    fn resolve_world_socket_path_normalizes_relative_components() {
        let temp = tempdir().unwrap();
        let raw = temp.path().join("socket").join("..").join("sock");
        let prev_socket = set_env("SUBSTRATE_WORLD_SOCKET", &raw);

        let normalized = resolve_world_socket_path().unwrap();
        assert_eq!(normalized, temp.path().join("sock"));

        restore_env("SUBSTRATE_WORLD_SOCKET", prev_socket);
    }

    #[test]
    fn locate_helper_script_prefers_version_dir() {
        let temp = tempdir().unwrap();
        let version_dir = temp.path().join("version");
        let script = version_dir.join("scripts/substrate/world-enable.sh");
        std::fs::create_dir_all(script.parent().unwrap()).unwrap();
        std::fs::write(&script, "#!/bin/sh\necho helper").unwrap();

        let resolved =
            locate_helper_script(temp.path(), Some(version_dir.as_ref()), None).expect("script");
        assert_eq!(resolved, script);
    }
}
