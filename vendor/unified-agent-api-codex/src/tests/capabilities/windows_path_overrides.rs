use super::*;

use std::collections::BTreeMap;
use std::ffi::OsString;
use std::path::{Path, PathBuf};

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

fn write_windows_probe_codex(dir: &Path, supports_add_dir: bool) -> PathBuf {
    let path = dir.join("codex.cmd");
    let script = if supports_add_dir {
        r#"@echo off
if "%~1"=="--version" (
  echo codex 1.0.0
  exit /b 0
)
if "%~1"=="features" if "%~2"=="list" if "%~3"=="--json" (
  echo {"features":["add_dir"]}
  exit /b 0
)
if "%~1"=="features" if "%~2"=="list" (
  echo add_dir
  exit /b 0
)
if "%~1"=="--help" (
  echo Usage: codex --add-dir
  exit /b 0
)
echo unexpected args: %*
exit /b 1
"#
    } else {
        r#"@echo off
if "%~1"=="--version" (
  echo codex 1.0.0
  exit /b 0
)
if "%~1"=="features" if "%~2"=="list" if "%~3"=="--json" (
  echo {"features":[]}
  exit /b 0
)
if "%~1"=="features" if "%~2"=="list" (
  exit /b 0
)
if "%~1"=="--help" (
  echo Usage: codex exec
  exit /b 0
)
echo unexpected args: %*
exit /b 1
"#
    };

    std_fs::write(&path, script).unwrap();
    path
}

#[tokio::test]
async fn capability_probe_with_env_overrides_uses_effective_path_case_insensitively_on_windows() {
    let _guard = env_guard_async().await;
    clear_capability_cache();

    let ambient = tempfile::tempdir().unwrap();
    let override_dir = tempfile::tempdir().unwrap();
    let path_restore = RestoreEnvVar::capture("PATH");
    let binary_restore = RestoreEnvVar::capture("CODEX_BINARY");

    let ambient_binary = write_windows_probe_codex(ambient.path(), false);
    let override_binary = write_windows_probe_codex(override_dir.path(), true);
    binary_restore.clear();
    path_restore.set(ambient.path().as_os_str().to_os_string());

    let client = CodexClient::builder()
        .timeout(Duration::from_secs(5))
        .build();

    let base = client.probe_capabilities().await;
    assert!(!base.features.supports_add_dir);
    assert_eq!(
        base.cache_key.binary_path,
        std_fs::canonicalize(&ambient_binary).unwrap()
    );

    let env_overrides = BTreeMap::from([(
        "Path".to_string(),
        override_dir.path().to_string_lossy().to_string(),
    )]);
    let env_sensitive = client
        .probe_capabilities_with_env_overrides(&env_overrides)
        .await;

    assert!(env_sensitive.features.supports_add_dir);
    assert_eq!(
        env_sensitive.cache_key.binary_path,
        std_fs::canonicalize(&override_binary).unwrap()
    );
}
