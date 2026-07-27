//! Helpers for locating and executing `limactl`.

use anyhow::{anyhow, Result};
use std::path::{Path, PathBuf};
use std::sync::OnceLock;
use transport_api_types::normalize_unix_install_bootstrap_path;

static LIMACTL_PATH: OnceLock<PathBuf> = OnceLock::new();

pub(crate) fn path() -> Result<PathBuf> {
    #[cfg(test)]
    if let Some(test_override) = std::env::var_os("SUBSTRATE_TEST_LIMACTL_PATH") {
        return Ok(PathBuf::from(test_override));
    }

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

pub(crate) fn command_for_control_root(
    host_account_home: &Path,
    control_root: &Path,
) -> Result<std::process::Command> {
    let mut command = std::process::Command::new(path()?);
    configure_command_for_control_root(&mut command, host_account_home, control_root)?;
    Ok(command)
}

fn configure_command_for_control_root(
    command: &mut std::process::Command,
    host_account_home: &Path,
    control_root: &Path,
) -> Result<()> {
    let host_account_home = normalized_unix_path(host_account_home, "host account home")?;
    let control_root = normalized_unix_path(control_root, "Lima control root")?;
    let expected_control_root = PathBuf::from(&host_account_home).join(".lima");
    let expected_control_root = normalized_unix_path(
        &expected_control_root,
        "expected Lima control root derived from host account home",
    )?;

    if control_root != expected_control_root {
        return Err(anyhow!(
            "Lima control root must equal <host account home>/.lima"
        ));
    }

    command.env_remove("HOME");
    command.env_remove("LIMA_HOME");
    command.env("HOME", &host_account_home);
    command.env("LIMA_HOME", &control_root);
    Ok(())
}

fn normalized_unix_path(path: &Path, field: &str) -> Result<String> {
    let raw = path
        .to_str()
        .ok_or_else(|| anyhow!("{field} is not valid UTF-8"))?;
    normalize_unix_install_bootstrap_path(raw)
        .map_err(|_| anyhow!("{field} is not a normalized absolute Unix path"))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn command_env_value(command: &std::process::Command, key: &str) -> Option<String> {
        command
            .get_envs()
            .find_map(|(name, value)| (name == std::ffi::OsStr::new(key)).then_some(value))
            .flatten()
            .map(|value| value.to_string_lossy().into_owned())
    }

    #[test]
    fn test_command_for_control_root_overwrites_home_and_lima_home() {
        let mut command = std::process::Command::new("/usr/bin/true");
        configure_command_for_control_root(
            &mut command,
            Path::new("/Users/alice"),
            Path::new("/Users/alice/.lima"),
        )
        .unwrap();

        assert_eq!(
            command_env_value(&command, "HOME").as_deref(),
            Some("/Users/alice")
        );
        assert_eq!(
            command_env_value(&command, "LIMA_HOME").as_deref(),
            Some("/Users/alice/.lima")
        );
    }

    #[test]
    fn test_command_for_control_root_rejects_conflicting_root() {
        let mut command = std::process::Command::new("/usr/bin/true");
        let err = configure_command_for_control_root(
            &mut command,
            Path::new("/Users/alice"),
            Path::new("/Users/alice/custom-lima"),
        )
        .unwrap_err();

        assert!(
            err.to_string().contains("<host account home>/.lima"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn test_command_for_control_root_rejects_non_normalized_paths() {
        let mut command = std::process::Command::new("/usr/bin/true");
        assert!(configure_command_for_control_root(
            &mut command,
            Path::new("Users/alice"),
            Path::new("/Users/alice/.lima"),
        )
        .is_err());
        assert!(configure_command_for_control_root(
            &mut command,
            Path::new("/Users/alice"),
            Path::new("/Users/alice/../alice/.lima"),
        )
        .is_err());
    }
}
