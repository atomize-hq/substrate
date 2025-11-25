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
