//! Helpers for locating and executing `limactl`.

use anyhow::Result;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

static LIMACTL_PATH: OnceLock<PathBuf> = OnceLock::new();

pub(crate) fn path() -> Result<PathBuf> {
    if let Some(cached) = LIMACTL_PATH.get() {
        return Ok(cached.clone());
    }

    let resolved = which::which("limactl").or_else(|_| {
        // GitHub Actions self-hosted runners (and some local setups) may not include Homebrew
        // in PATH. Fall back to the common install prefixes.
        for candidate in [
            "/opt/homebrew/bin/limactl",
            "/usr/local/bin/limactl",
            "/opt/homebrew/sbin/limactl",
            "/usr/local/sbin/limactl",
        ] {
            let candidate = Path::new(candidate);
            if candidate.is_file() {
                return Ok(candidate.to_path_buf());
            }
        }

        Err(anyhow::anyhow!(
            "limactl not found. Install Lima with: brew install lima"
        ))
    })?;

    let _ = LIMACTL_PATH.set(resolved.clone());
    Ok(resolved)
}

pub(crate) fn command() -> Result<std::process::Command> {
    Ok(std::process::Command::new(path()?))
}
