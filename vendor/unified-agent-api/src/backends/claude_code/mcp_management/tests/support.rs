use std::{
    collections::BTreeMap,
    env,
    ffi::OsString,
    path::{Path, PathBuf},
    process::ExitStatus,
    time::Duration,
};

use tokio::io::{AsyncWriteExt, DuplexStream};

use crate::{mcp::AgentWrapperMcpCommandContext, AgentWrapperError};

use super::super::PINNED_SPAWN_FAILURE;

#[cfg(unix)]
use std::{
    fs,
    os::unix::fs::PermissionsExt,
    time::{SystemTime, UNIX_EPOCH},
};

pub(super) fn success_exit_status() -> ExitStatus {
    #[cfg(unix)]
    {
        use std::os::unix::process::ExitStatusExt;
        ExitStatus::from_raw(0)
    }
    #[cfg(windows)]
    {
        use std::os::windows::process::ExitStatusExt;
        ExitStatus::from_raw(0)
    }
}

pub(super) fn exit_status_with_code(code: i32) -> ExitStatus {
    #[cfg(unix)]
    {
        use std::os::unix::process::ExitStatusExt;
        ExitStatus::from_raw(code << 8)
    }
    #[cfg(windows)]
    {
        use std::os::windows::process::ExitStatusExt;
        ExitStatus::from_raw(code as u32)
    }
}

pub(super) fn sample_config() -> super::super::super::ClaudeCodeBackendConfig {
    super::super::super::ClaudeCodeBackendConfig {
        binary: Some(PathBuf::from("/tmp/fake-claude")),
        claude_home: Some(PathBuf::from("/tmp/claude-home")),
        default_timeout: Some(Duration::from_secs(30)),
        default_working_dir: Some(PathBuf::from("default/workdir")),
        env: BTreeMap::from([
            ("CONFIG_ONLY".to_string(), "config-only".to_string()),
            ("OVERRIDE_ME".to_string(), "config".to_string()),
        ]),
        ..Default::default()
    }
}

pub(super) fn sample_config_without_home() -> super::super::super::ClaudeCodeBackendConfig {
    super::super::super::ClaudeCodeBackendConfig {
        binary: Some(PathBuf::from("/tmp/fake-claude")),
        claude_home: None,
        default_timeout: Some(Duration::from_secs(30)),
        default_working_dir: Some(PathBuf::from("default/workdir")),
        env: BTreeMap::from([
            ("CONFIG_ONLY".to_string(), "config-only".to_string()),
            ("OVERRIDE_ME".to_string(), "config".to_string()),
        ]),
        ..Default::default()
    }
}

pub(super) fn sample_context() -> AgentWrapperMcpCommandContext {
    AgentWrapperMcpCommandContext {
        working_dir: Some(PathBuf::from("request/workdir")),
        timeout: Some(Duration::from_secs(5)),
        env: BTreeMap::from([
            ("OVERRIDE_ME".to_string(), "request".to_string()),
            ("REQUEST_ONLY".to_string(), "request-only".to_string()),
        ]),
    }
}

pub(super) fn test_env_lock() -> crate::backends::test_support::TestEnvLockGuard {
    crate::backends::test_support::test_env_lock()
}

pub(super) struct EnvGuard {
    key: &'static str,
    previous: Option<OsString>,
}

impl EnvGuard {
    pub(super) fn set(key: &'static str, value: impl Into<OsString>) -> Self {
        let previous = env::var_os(key);
        env::set_var(key, value.into());
        Self { key, previous }
    }

    pub(super) fn unset(key: &'static str) -> Self {
        let previous = env::var_os(key);
        env::remove_var(key);
        Self { key, previous }
    }
}

impl Drop for EnvGuard {
    fn drop(&mut self) {
        if let Some(value) = self.previous.take() {
            env::set_var(self.key, value);
        } else {
            env::remove_var(self.key);
        }
    }
}

pub(super) struct CurrentDirGuard {
    previous: PathBuf,
}

impl CurrentDirGuard {
    pub(super) fn set(path: &Path) -> Self {
        let previous = env::current_dir().unwrap_or_else(|_| env::temp_dir());
        env::set_current_dir(path).expect("current dir should be set");
        Self { previous }
    }
}

impl Drop for CurrentDirGuard {
    fn drop(&mut self) {
        if env::set_current_dir(&self.previous).is_err() {
            env::set_current_dir(env::temp_dir()).expect("fallback current dir should be restored");
        }
    }
}

pub(super) fn assert_backend_spawn_failure(err: AgentWrapperError) {
    match err {
        AgentWrapperError::Backend { message } => {
            assert_eq!(message, PINNED_SPAWN_FAILURE);
        }
        other => panic!("expected Backend error, got: {other:?}"),
    }
}

pub(super) async fn write_all_and_close(mut writer: DuplexStream, bytes: Vec<u8>) {
    writer.write_all(&bytes).await.expect("write succeeds");
    writer.shutdown().await.expect("shutdown succeeds");
}

#[cfg(unix)]
pub(super) fn temp_test_dir(label: &str) -> PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock should be after epoch")
        .as_nanos();
    let dir = std::env::temp_dir().join(format!(
        "agent-api-claude-mcp-{label}-{}-{unique}",
        std::process::id()
    ));
    fs::create_dir_all(&dir).expect("temp dir should be created");
    dir
}

#[cfg(unix)]
pub(super) fn write_fake_claude(dir: &std::path::Path, script: &str) -> PathBuf {
    fs::create_dir_all(dir).expect("script directory should be created");
    let path = dir.join("claude");
    fs::write(&path, script).expect("script should be written");
    let mut permissions = fs::metadata(&path)
        .expect("script metadata should exist")
        .permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(&path, permissions).expect("script should be executable");
    path
}
