//! Path, cwd, and environment helpers used during routing.

use crate::execution::settings;
use anyhow::{Context, Result};
use std::env;
use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitStatus};
use substrate_common::WorldRootMode;

pub(crate) fn ok_status() -> Result<ExitStatus> {
    if cfg!(windows) {
        Command::new("cmd").arg("/C").arg("exit 0").status()
    } else {
        Command::new("true").status()
    }
    .context("Failed to create success status")
}

pub(crate) fn canonicalize_cd_target(current_dir: &Path, target: &str) -> Result<PathBuf> {
    let requested = Path::new(target);
    let absolute = if requested.is_absolute() {
        requested.to_path_buf()
    } else {
        current_dir.join(requested)
    };
    fs::canonicalize(&absolute)
        .with_context(|| format!("failed to resolve directory {}", absolute.display()))
}

pub(crate) fn enforce_caged_destination(
    settings: &settings::WorldRootSettings,
    current_dir: &Path,
    requested: PathBuf,
) -> (PathBuf, Option<String>) {
    if !settings.caged || settings.mode == WorldRootMode::FollowCwd {
        return (requested, None);
    }

    let anchor = settings.anchor_root(current_dir);
    let anchor_clean = canonicalize_or(&anchor);
    if path_within_root(&anchor_clean, &requested) {
        (requested, None)
    } else {
        let message = format!(
            "substrate: info: caged root guard: returning to {}",
            anchor_clean.display()
        );
        (anchor_clean, Some(message))
    }
}

pub(crate) fn canonicalize_or(path: &Path) -> PathBuf {
    fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf())
}

pub(crate) fn path_within_root(anchor: &Path, candidate: &Path) -> bool {
    candidate == anchor || candidate.starts_with(anchor)
}

pub(crate) fn world_deps_manifest_base_path() -> PathBuf {
    if let Ok(override_path) = env::var("SUBSTRATE_WORLD_DEPS_MANIFEST") {
        return PathBuf::from(override_path);
    }

    if let Some(path) = installed_world_deps_manifest_base_path() {
        return path;
    }

    repo_world_deps_manifest_base_path()
}

fn installed_world_deps_manifest_base_path() -> Option<PathBuf> {
    let exe_path = env::current_exe().ok()?;
    let canonical = canonicalize_or(&exe_path);

    let bin_dir = canonical.parent()?;
    if bin_dir.file_name() != Some(OsStr::new("bin")) {
        return None;
    }

    let version_dir = bin_dir.parent()?;
    let versions_dir = version_dir.parent()?;
    if versions_dir.file_name() != Some(OsStr::new("versions")) {
        return None;
    }

    Some(version_dir.join("config").join("world-deps.yaml"))
}

fn repo_world_deps_manifest_base_path() -> PathBuf {
    let crate_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    crate_dir
        .parent()
        .and_then(|dir| dir.parent())
        .map(|root| {
            root.join("scripts")
                .join("substrate")
                .join("world-deps.yaml")
        })
        .unwrap_or_else(|| PathBuf::from("scripts/substrate/world-deps.yaml"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;
    use tempfile::tempdir;

    fn set_env(key: &str, value: &str) -> Option<String> {
        let previous = env::var(key).ok();
        env::set_var(key, value);
        previous
    }

    fn restore_env(key: &str, previous: Option<String>) {
        if let Some(value) = previous {
            env::set_var(key, value);
        } else {
            env::remove_var(key);
        }
    }

    #[test]
    #[serial]
    fn enforce_caged_destination_bounces_outside_anchor() {
        let temp = tempdir().unwrap();
        let root = temp.path().join("root");
        let outside = temp.path().join("outside");
        fs::create_dir_all(&root).unwrap();
        fs::create_dir_all(&outside).unwrap();
        let settings = settings::WorldRootSettings {
            mode: WorldRootMode::Project,
            path: fs::canonicalize(&root).unwrap(),
            caged: true,
        };
        let requested = fs::canonicalize(&outside).unwrap();

        let (destination, warning) =
            enforce_caged_destination(&settings, &settings.path, requested);
        assert_eq!(destination, settings.path);
        let message = warning.expect("expected caged warning");
        assert!(message.contains("caged root guard"));
        assert!(message.contains(settings.path.to_str().unwrap()));
    }

    #[test]
    #[serial]
    fn world_deps_manifest_base_path_prefers_env_override() {
        let temp = tempdir().unwrap();
        let override_path = temp.path().join("deps.yaml");
        let previous = set_env(
            "SUBSTRATE_WORLD_DEPS_MANIFEST",
            &override_path.display().to_string(),
        );
        let resolved = world_deps_manifest_base_path();
        assert_eq!(resolved, override_path);
        restore_env("SUBSTRATE_WORLD_DEPS_MANIFEST", previous);
    }

    #[test]
    #[serial]
    fn world_deps_manifest_base_path_defaults_to_repo_location() {
        let previous = env::var("SUBSTRATE_WORLD_DEPS_MANIFEST").ok();
        env::remove_var("SUBSTRATE_WORLD_DEPS_MANIFEST");
        let resolved = world_deps_manifest_base_path();
        assert!(
            resolved
                .components()
                .any(|c| c.as_os_str() == "world-deps.yaml"),
            "path should point to scripts/substrate/world-deps.yaml (found {})",
            resolved.display()
        );
        restore_env("SUBSTRATE_WORLD_DEPS_MANIFEST", previous);
    }
}
