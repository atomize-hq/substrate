//! Path, cwd, and environment helpers used during routing.

use crate::execution::settings;
use anyhow::{Context, Result};
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
            "substrate: info: caged root guard: blocked cd to {} (outside {}); returning to {}",
            requested.display(),
            anchor_clean.display(),
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

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;
    use tempfile::tempdir;

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
}
