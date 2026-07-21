#![cfg(unix)]
#![allow(dead_code, unused_imports)]

#[path = "../common.rs"]
pub mod common;

use assert_cmd::Command;
use parking_lot::{ReentrantMutex, ReentrantMutexGuard};
use std::env;
use std::ffi::{OsStr, OsString};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, OnceLock};
use std::time::{Duration, Instant};
use tempfile::{Builder, TempDir};

pub use common::{binary_path, ensure_substrate_built, substrate_shell_driver, temp_dir};
pub use substrate_common::dedupe_path;
mod socket;
pub use socket::{AgentSocket, PendingDiffAckError, PendingDiffAckState, SocketResponse};
mod repl_world_service;
pub use repl_world_service::{
    MemberDispatchStreamScript, PersistentExecRecord, PersistentExecStdoutOverride,
    PersistentStartSessionRecord, ReplWorldAgentRecords, ReplWorldAgentStub, StreamBehavior,
};

pub const PAYLOAD_MARKER: &str = "__SUBSTRATE_PAYLOAD__";

static AUTHORITY_ENV_LOCK: OnceLock<ReentrantMutex<()>> = OnceLock::new();

struct AuthorityEnvGuard {
    previous_home: Option<OsString>,
    previous_world_socket: Option<OsString>,
    _lock: ReentrantMutexGuard<'static, ()>,
}

impl AuthorityEnvGuard {
    fn set_home(value: impl AsRef<OsStr>) -> Self {
        let lock = AUTHORITY_ENV_LOCK
            .get_or_init(|| ReentrantMutex::new(()))
            .lock();
        let previous_home = env::var_os("SUBSTRATE_HOME");
        let previous_world_socket = env::var_os("SUBSTRATE_WORLD_SOCKET");
        env::set_var("SUBSTRATE_HOME", value);
        Self {
            previous_home,
            previous_world_socket,
            _lock: lock,
        }
    }
}

impl Drop for AuthorityEnvGuard {
    fn drop(&mut self) {
        match self.previous_home.as_deref() {
            Some(value) => env::set_var("SUBSTRATE_HOME", value),
            None => env::remove_var("SUBSTRATE_HOME"),
        }
        match self.previous_world_socket.as_deref() {
            Some(value) => env::set_var("SUBSTRATE_WORLD_SOCKET", value),
            None => env::remove_var("SUBSTRATE_WORLD_SOCKET"),
        }
    }
}

pub fn get_substrate_binary() -> Command {
    substrate_shell_driver()
}

pub struct ShellEnvFixture {
    _temp: TempDir,
    home: PathBuf,
}

impl ShellEnvFixture {
    pub fn new() -> Self {
        let temp = Builder::new()
            .prefix("substrate-test-")
            .tempdir_in("/tmp")
            .expect("failed to allocate integration test temp dir");
        let home = temp.path().join("home");
        fs::create_dir_all(home.join(".substrate/shims"))
            .expect("failed to create shims directory");
        Self { _temp: temp, home }
    }

    pub fn home(&self) -> &Path {
        &self.home
    }

    pub fn shim_dir(&self) -> PathBuf {
        self.home.join(".substrate/shims")
    }

    pub fn manager_env_path(&self) -> PathBuf {
        self.home.join(".substrate/manager_env.sh")
    }

    pub fn manager_init_path(&self) -> PathBuf {
        self.home.join(".substrate/manager_init.sh")
    }

    pub fn preexec_path(&self) -> PathBuf {
        self.home.join(".substrate_preexec")
    }

    pub fn overlay_path(&self) -> PathBuf {
        self.home.join(".substrate/manager_hooks.local.yaml")
    }

    pub fn write_manifest(&self, contents: &str) -> PathBuf {
        let path = self.home.join("manager_hooks.yaml");
        fs::write(&path, contents).expect("failed to write manager manifest");
        path
    }
}

pub fn substrate_command_for_home(fixture: &ShellEnvFixture) -> Command {
    let mut cmd = get_substrate_binary();
    cmd.env("HOME", fixture.home())
        .env("USERPROFILE", fixture.home())
        .current_dir(fixture.home())
        .env("SHELL", "/bin/bash")
        .env("SUBSTRATE_HOME", fixture.home().join(".substrate"))
        .env_remove("SUBSTRATE_WORLD")
        .env_remove("SUBSTRATE_WORLD_ENABLED")
        .env("SUBSTRATE_OVERRIDE_WORLD", "enabled")
        .env_remove("SUBSTRATE_NO_SHIMS")
        // Ensure host-installed shims do not affect PATH injection in tests.
        .env_remove("SUBSTRATE_SHIM_PATH")
        .env_remove("SUBSTRATE_SHIM_ORIGINAL_PATH")
        .env_remove("SUBSTRATE_SHIM_DEPLOY_DIR")
        .env_remove("SHIM_ORIGINAL_PATH")
        .env_remove("PATH_BEFORE_SUBSTRATE_SHIM");
    cmd
}

pub fn path_str(path: &Path) -> String {
    path.to_string_lossy().into_owned()
}

pub fn wait_for_min_member_dispatch_requests(
    records: &Arc<Mutex<ReplWorldAgentRecords>>,
    min_requests: usize,
    timeout: Duration,
) {
    let deadline = Instant::now() + timeout;
    while Instant::now() < deadline {
        let guard = records.lock().expect("lock records");
        if guard.member_dispatch_requests.len() >= min_requests {
            return;
        }
        drop(guard);
        std::thread::sleep(Duration::from_millis(25));
    }

    let guard = records.lock().expect("lock records");
    panic!(
        "timed out waiting for member_dispatch requests >= {min_requests}; got {}; records: {guard:#?}",
        guard.member_dispatch_requests.len(),
    );
}

pub fn wait_for_min_member_turn_submit_requests(
    records: &Arc<Mutex<ReplWorldAgentRecords>>,
    min_requests: usize,
    timeout: Duration,
) {
    let deadline = Instant::now() + timeout;
    while Instant::now() < deadline {
        let guard = records.lock().expect("lock records");
        if guard.member_turn_submit_requests.len() >= min_requests {
            return;
        }
        drop(guard);
        std::thread::sleep(Duration::from_millis(25));
    }

    let guard = records.lock().expect("lock records");
    panic!(
        "timed out waiting for member_turn_submit requests >= {min_requests}; got {}; records: {guard:#?}",
        guard.member_turn_submit_requests.len(),
    );
}

pub fn wait_for_min_execute_cancel_requests(
    records: &Arc<Mutex<ReplWorldAgentRecords>>,
    min_requests: usize,
    timeout: Duration,
) {
    let deadline = Instant::now() + timeout;
    while Instant::now() < deadline {
        let guard = records.lock().expect("lock records");
        if guard.execute_cancel_requests.len() >= min_requests {
            return;
        }
        drop(guard);
        std::thread::sleep(Duration::from_millis(25));
    }

    let guard = records.lock().expect("lock records");
    panic!(
        "timed out waiting for execute_cancel requests >= {min_requests}; got {}; records: {guard:#?}",
        guard.execute_cancel_requests.len(),
    );
}

pub fn payload_lines(stdout: &[u8]) -> Vec<String> {
    let data = String::from_utf8_lossy(stdout);
    let mut marker_found = false;
    let mut lines = Vec::new();
    for line in data.lines() {
        if marker_found {
            let trimmed = line.trim_end();
            if trimmed.is_empty() || trimmed == PAYLOAD_MARKER {
                continue;
            }
            lines.push(trimmed.to_string());
        } else if line.trim() == PAYLOAD_MARKER {
            marker_found = true;
        }
    }
    assert!(
        marker_found,
        "payload marker `{}` not found in output: {}",
        PAYLOAD_MARKER, data
    );
    lines
}

pub fn persist_runtime_alert_for_substrate_home(
    substrate_home: &Path,
    orchestration_session_id: &str,
    item_id: &str,
    message: Option<String>,
) {
    let _authority_env = AuthorityEnvGuard::set_home(substrate_home);
    let result =
        substrate_shell::execution::agent_dev_support::persist_runtime_alert_for_dev_support(
            orchestration_session_id,
            item_id,
            message,
        );
    result.expect("persist runtime alert through authoritative state store");
}
