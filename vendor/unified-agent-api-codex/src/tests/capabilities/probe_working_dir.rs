use super::*;

use std::{env, ffi::OsString, path::Path, path::PathBuf, time::Duration};

struct RestoreCurrentDir {
    original: PathBuf,
}

impl RestoreCurrentDir {
    fn capture() -> Self {
        Self {
            original: env::current_dir().unwrap(),
        }
    }
}

impl Drop for RestoreCurrentDir {
    fn drop(&mut self) {
        env::set_current_dir(&self.original).unwrap();
    }
}

struct RestoreEnvVar {
    key: &'static str,
    original: Option<OsString>,
}

impl RestoreEnvVar {
    fn capture(key: &'static str) -> Self {
        Self {
            key,
            original: env::var_os(key),
        }
    }

    fn set(&self, value: impl Into<OsString>) {
        env::set_var(self.key, value.into());
    }

    fn clear(&self) {
        env::remove_var(self.key);
    }
}

impl Drop for RestoreEnvVar {
    fn drop(&mut self) {
        match self.original.take() {
            Some(value) => env::set_var(self.key, value),
            None => env::remove_var(self.key),
        }
    }
}

fn prepend_path(path: &Path) -> OsString {
    match env::var_os("PATH") {
        Some(current) => {
            let mut entries = vec![path.to_path_buf()];
            entries.extend(env::split_paths(&current));
            env::join_paths(entries).expect("join PATH entries")
        }
        None => path.as_os_str().to_os_string(),
    }
}

fn write_probe_script(root: &Path, supports_add_dir: bool) {
    let bin_dir = root.join("bin");
    std_fs::create_dir_all(&bin_dir).unwrap();
    let features_json = if supports_add_dir {
        r#"{"features":["add_dir"]}"#
    } else {
        r#"{"features":[]}"#
    };
    let features_text = if supports_add_dir { "add_dir" } else { "" };
    let help_text = if supports_add_dir {
        "Usage: codex --add-dir"
    } else {
        "Usage: codex exec"
    };

    write_fake_codex(
        &bin_dir,
        &format!(
            r#"#!/usr/bin/env bash
set -euo pipefail

if [[ "$1" == "--version" ]]; then
  echo "codex 1.0.0"
elif [[ "$1" == "features" && "$2" == "list" && "$3" == "--json" ]]; then
  echo '{features_json}'
elif [[ "$1" == "features" && "$2" == "list" ]]; then
  echo "{features_text}"
elif [[ "$1" == "--help" ]]; then
  echo "{help_text}"
fi
"#,
            features_json = features_json,
            features_text = features_text,
            help_text = help_text
        ),
    );
}

#[cfg(unix)]
#[tokio::test]
async fn probe_cache_key_uses_effective_working_dir_for_relative_binary() {
    let _guard = env_guard_async().await;
    clear_capability_cache();

    let ambient = tempfile::tempdir().unwrap();
    let alpha = tempfile::tempdir().unwrap();
    let beta = tempfile::tempdir().unwrap();
    let _restore_cwd = RestoreCurrentDir::capture();

    write_probe_script(ambient.path(), false);
    write_probe_script(alpha.path(), true);
    write_probe_script(beta.path(), false);
    env::set_current_dir(ambient.path()).unwrap();

    let alpha_client = CodexClient::builder()
        .binary("./bin/codex")
        .working_dir(alpha.path())
        .timeout(Duration::from_secs(5))
        .build();
    let alpha_capabilities = alpha_client
        .probe_capabilities_for_current_dir(alpha.path())
        .await;
    assert!(alpha_capabilities.features.supports_add_dir);
    assert_eq!(
        alpha_capabilities.cache_key.binary_path,
        std_fs::canonicalize(alpha.path().join("bin/codex")).unwrap()
    );

    let beta_client = CodexClient::builder()
        .binary("./bin/codex")
        .working_dir(beta.path())
        .timeout(Duration::from_secs(5))
        .build();
    let beta_capabilities = beta_client
        .probe_capabilities_for_current_dir(beta.path())
        .await;
    assert!(!beta_capabilities.features.supports_add_dir);
    assert_eq!(
        beta_capabilities.cache_key.binary_path,
        std_fs::canonicalize(beta.path().join("bin/codex")).unwrap()
    );

    let cache_paths: Vec<_> = capability_cache_entries()
        .into_iter()
        .map(|entry| entry.cache_key.binary_path)
        .collect();
    assert!(cache_paths.contains(&std_fs::canonicalize(alpha.path().join("bin/codex")).unwrap()));
    assert!(cache_paths.contains(&std_fs::canonicalize(beta.path().join("bin/codex")).unwrap()));
}

#[cfg(unix)]
#[tokio::test]
async fn probe_cache_key_resolves_default_bare_binary_from_path() {
    let _guard = env_guard_async().await;
    clear_capability_cache();

    let path_root = tempfile::tempdir().unwrap();
    let working = tempfile::tempdir().unwrap();
    let ambient = tempfile::tempdir().unwrap();
    let _restore_cwd = RestoreCurrentDir::capture();
    let path_restore = RestoreEnvVar::capture("PATH");
    let binary_restore = RestoreEnvVar::capture("CODEX_BINARY");

    write_probe_script(path_root.path(), true);
    binary_restore.clear();
    path_restore.set(prepend_path(&path_root.path().join("bin")));
    env::set_current_dir(ambient.path()).unwrap();

    let client = CodexClient::builder()
        .working_dir(working.path())
        .timeout(Duration::from_secs(5))
        .build();
    let capabilities = client
        .probe_capabilities_for_current_dir(working.path())
        .await;

    assert!(capabilities.features.supports_add_dir);
    assert_eq!(
        capabilities.cache_key.binary_path,
        std_fs::canonicalize(path_root.path().join("bin/codex")).unwrap()
    );
}
