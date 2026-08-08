#[doc(hidden)]
pub mod agent_dev_support;
pub mod agent_events;
pub(crate) mod agent_inventory;
pub(crate) mod agent_runtime;
mod agents_cmd;
mod auto_sync;
mod cli;
mod config_cmd;
pub(crate) mod config_model;
mod env_scripts;
mod home_bootstrap;
#[cfg(any(target_os = "linux", test))]
pub(crate) mod host_inbox_materialization;
pub(crate) mod install_bootstrap;
mod invocation;
pub mod lock;
pub mod managed_lifecycle;
mod manager;
pub mod manager_init;
pub(crate) mod orchestrator_world_dispatch;
mod platform;
#[cfg(any(target_os = "macos", target_os = "windows"))]
pub(crate) mod platform_world;
mod policy_cmd;
pub(crate) mod policy_model;
pub(crate) mod policy_snapshot;
pub(crate) mod prompt_fulfillment;
mod pty;
pub mod repl_persistent_session;
mod routing;
mod settings;
pub(crate) use settings::resolve_world_root;
pub(crate) use settings::WorldRootSettings;
pub mod shim_deploy;
#[cfg(target_os = "linux")]
pub(crate) mod socket_activation; // Made public for integration tests
mod value_parse;
mod workspace;
mod workspace_cmd;

pub(crate) use agents_cmd::handle_agent_command;
pub(crate) use agents_cmd::handle_agents_command;
pub(crate) use auto_sync::run_auto_sync_if_enabled;
pub use cli::*;
pub(crate) use config_cmd::handle_config_command;
pub(crate) use env_scripts::{
    env_sh_path, export_runtime_config_env, write_env_sh, write_env_sh_at,
};
pub use invocation::{needs_shell, ShellConfig, ShellMode};
pub(crate) use policy_cmd::handle_policy_command;
#[cfg(unix)]
pub(crate) use pty::get_terminal_size;
pub(crate) use pty::MinimalTerminalGuard;
#[cfg(test)]
pub(crate) use routing::test_utils::ProcessCwdTestGuard;
pub use routing::*;
pub(crate) use workspace::find_workspace_root;
pub(crate) use workspace_cmd::handle_workspace_command;

pub use managed_lifecycle::{
    compare_and_swap_head_v1, complete_mac_lima_stage_one_transition_v1,
    deliver_retained_publisher_bootstrap_authorization_v1, derive_lifecycle_capsule_locator_v1,
    load_manifest_v1, open_publisher_bootstrap_channel_v1, open_trusted_lifecycle_capsule_v1,
    publish_action_receipt_index_v1, publish_manifest_v1, resume_action_receipt_commit_v1,
    submit_post_pm_managed_action_v1, submit_stage_one_absent_instance_create_v1,
    transition_manifest_v1, update_shared_claims_v1, validate_mapped_lifecycle_control_request_v1,
    validate_publisher_response_v1, LifecyclePublisherClientV1, ManagedLifecycleControlRequestV1,
    MappedLifecycleTagV1,
};
pub(crate) use manager::{
    configure_child_shell_env, configure_manager_init, current_platform, log_manager_init_event,
    manager_manifest_base_path, write_manager_env_script,
};
pub(crate) use platform::{
    handle_health_command, handle_host_command, handle_world_command, update_world_env,
};
#[cfg(any(target_os = "macos", target_os = "windows"))]
pub(crate) use platform_world as pw;

#[cfg(test)]
use parking_lot::{ReentrantMutex, ReentrantMutexGuard};
#[cfg(test)]
use std::ffi::{OsStr, OsString};
#[cfg(test)]
use std::sync::OnceLock;

#[cfg(test)]
// Deliberately non-poisoning: panic unwinding drops the guard, restores the exact
// authority-environment snapshot, and permits a later test to acquire the boundary.
static WORLD_ENV_LOCK: OnceLock<ReentrantMutex<()>> = OnceLock::new();

#[cfg(test)]
const AUTHORITY_ENVIRONMENT_NAMES: &[&str] = &[
    "API_TOKEN",
    "CODEX_HOME",
    "COLUMNS",
    "EXPORT_COMPLEX",
    "HOME",
    "LIMA_VM_NAME",
    "LINES",
    "OLDPWD",
    "OPENAI_API_KEY",
    "PATH",
    "PLAIN_VALUE",
    "PWD",
    "SHIM_ACTIVE",
    "SHIM_ORIGINAL_PATH",
    "SHIM_PARENT_SPAN",
    "SHIM_SESSION_ID",
    "SHIM_TRACE_LOG",
    "SUBSTRATE_ANCHOR_MODE",
    "SUBSTRATE_ANCHOR_PATH",
    "SUBSTRATE_CAGED",
    "SUBSTRATE_COMMAND_SUCCESS_EVENTS",
    "SUBSTRATE_DISABLE_PTY",
    "SUBSTRATE_FORCE_PTY",
    "SUBSTRATE_HOME",
    "SUBSTRATE_INSTALL_BOOTSTRAP_CONTEXT_V1",
    "SUBSTRATE_INSTALL_HOST_CONTEXT_COMMITMENT",
    "SUBSTRATE_INSTALL_PRIMARY_UID",
    "SUBSTRATE_INSTALL_PRIMARY_USER",
    "SUBSTRATE_INTERNAL_CODEX_AUTH_SEED_HOME",
    "SUBSTRATE_LIMA_VM_NAME",
    "SUBSTRATE_LLM_BACKEND_AUTH_CLI_CODEX_ACCESS_TOKEN",
    "SUBSTRATE_LLM_BACKEND_AUTH_CLI_CODEX_ACCOUNT_ID",
    "SUBSTRATE_MANAGER_ENV",
    "SUBSTRATE_MANAGER_INIT",
    "SUBSTRATE_MANAGER_INIT_DEBUG",
    "SUBSTRATE_MANAGER_INIT_POWERSHELL",
    "SUBSTRATE_MANAGER_MANIFEST",
    "SUBSTRATE_NO_SHIMS",
    "SUBSTRATE_OVERRIDE_ANCHOR_MODE",
    "SUBSTRATE_OVERRIDE_ANCHOR_PATH",
    "SUBSTRATE_OVERRIDE_CAGED",
    "SUBSTRATE_OVERRIDE_WORLD",
    "SUBSTRATE_POLICY_MODE",
    "SUBSTRATE_PTY_PIPELINE_LAST",
    "SUBSTRATE_REPLAY_USE_WORLD",
    "SUBSTRATE_REPLAY_VERBOSE",
    "SUBSTRATE_ROOT",
    "SUBSTRATE_SHELL",
    "SUBSTRATE_SKIP_MANAGER_INIT",
    "SUBSTRATE_SKIP_MANAGER_INIT_LIST",
    "SUBSTRATE_SOCKET_ACTIVATION_OVERRIDE",
    "SUBSTRATE_SYSTEMCTL_TIMEOUT_MS",
    "SUBSTRATE_TEST_LOCAL_WORLD_ID",
    "SUBSTRATE_TEST_SHARED_WORLD_METADATA_ROOT",
    "SUBSTRATE_WORLD",
    "SUBSTRATE_WORLD_DEPS_GUEST_BIN_DIR",
    "SUBSTRATE_WORLD_DEPS_SKIP_APT",
    "SUBSTRATE_WORLD_DEPS_SKIP_PACMAN",
    "SUBSTRATE_WORLD_ENABLED",
    "SUBSTRATE_WORLD_FAIL_CLOSED_ROUTING",
    "SUBSTRATE_WORLD_FS_ISOLATION",
    "SUBSTRATE_WORLD_FS_MODE",
    "SUBSTRATE_WORLD_ID",
    "SUBSTRATE_WORLD_NET_FILTER",
    "SUBSTRATE_WORLD_REQUEST_PROFILE",
    "SUBSTRATE_WORLD_REQUIRE_WORLD",
    "SUBSTRATE_WORLD_SOCKET",
    "TEST_ENV_KEY",
    "TEST_MODE",
    "UNSET_ME",
    "USERPROFILE",
    "XDG_CONFIG_HOME",
    "XDG_DATA_HOME",
    "XDG_STATE_HOME",
];

#[cfg(test)]
pub(crate) fn world_env_guard() -> ReentrantMutexGuard<'static, ()> {
    WORLD_ENV_LOCK
        .get_or_init(|| ReentrantMutex::new(()))
        .lock()
}

#[cfg(test)]
pub(crate) fn run_in_bounded_test_subprocess(test_name: &str, role: &str) -> bool {
    use std::io::{Read, Write};
    use std::process::Stdio;
    use std::time::{Duration, Instant};

    let test_name = test_name
        .strip_prefix(concat!(env!("CARGO_CRATE_NAME"), "::"))
        .unwrap_or(test_name);
    let marker = format!("__substrate_harness_child__{role}__{test_name}");
    let sentinel = format!("SUBSTRATE_HARNESS_READY {role} {test_name}\n");
    if std::env::args_os().any(|arg| arg == OsStr::new(&marker)) {
        std::io::stdout()
            .write_all(sentinel.as_bytes())
            .expect("publish harness child readiness");
        std::io::stdout()
            .flush()
            .expect("flush harness child readiness");
        return false;
    }

    let process_state = ProcessCwdTestGuard::preserve();
    let mut child = std::process::Command::new(
        std::env::current_exe().expect("resolve shell test binary for child isolation"),
    )
    .arg(test_name)
    .arg("--exact")
    .arg("--nocapture")
    .arg("--skip")
    .arg(&marker)
    .stdin(Stdio::null())
    .stdout(Stdio::piped())
    .stderr(Stdio::piped())
    .spawn()
    .expect("spawn isolated shell test child");
    drop(process_state);

    let mut stdout = child.stdout.take().expect("isolated child stdout");
    let mut stderr = child.stderr.take().expect("isolated child stderr");
    let stdout_reader = std::thread::spawn(move || {
        let mut bytes = Vec::new();
        stdout
            .read_to_end(&mut bytes)
            .expect("drain isolated child stdout");
        bytes
    });
    let stderr_reader = std::thread::spawn(move || {
        let mut bytes = Vec::new();
        stderr
            .read_to_end(&mut bytes)
            .expect("drain isolated child stderr");
        bytes
    });

    let deadline = Instant::now() + Duration::from_secs(60);
    let status = loop {
        if let Some(status) = child.try_wait().expect("poll isolated shell test child") {
            break status;
        }
        if Instant::now() >= deadline {
            child
                .kill()
                .expect("kill timed-out isolated shell test child");
            let status = child
                .wait()
                .expect("reap timed-out isolated shell test child");
            let stdout = stdout_reader.join().expect("join isolated stdout drainer");
            let stderr = stderr_reader.join().expect("join isolated stderr drainer");
            panic!(
                "isolated test child timed out and was reaped: test={test_name} role={role} status={status} stdout_bytes={} stderr_bytes={}",
                stdout.len(),
                stderr.len()
            );
        }
        std::thread::yield_now();
    };

    let stdout = stdout_reader.join().expect("join isolated stdout drainer");
    let stderr = stderr_reader.join().expect("join isolated stderr drainer");
    let sentinel_count = stdout
        .windows(sentinel.len())
        .filter(|window| *window == sentinel.as_bytes())
        .count()
        + stderr
            .windows(sentinel.len())
            .filter(|window| *window == sentinel.as_bytes())
            .count();
    assert_eq!(
        sentinel_count,
        1,
        "isolated child readiness mismatch: test={test_name} role={role} status={status} stdout_bytes={} stderr_bytes={}",
        stdout.len(),
        stderr.len()
    );
    assert!(
        status.success(),
        "isolated test child failed: test={test_name} role={role} status={status} stdout_bytes={} stderr_bytes={}",
        stdout.len(),
        stderr.len()
    );
    true
}

#[cfg(test)]
#[derive(Clone, Copy)]
pub(crate) enum TestSubprocessExpectation {
    Success,
    ExitCode(i32),
    #[cfg(windows)]
    Failure,
    #[cfg(unix)]
    Signal(i32),
    Timeout(std::time::Duration),
}

#[cfg(test)]
fn normalized_test_name(test_name: &str) -> &str {
    test_name
        .strip_prefix(concat!(env!("CARGO_CRATE_NAME"), "::"))
        .unwrap_or(test_name)
}

#[cfg(test)]
pub(crate) fn is_bounded_test_subprocess_child(test_name: &str, role: &str) -> bool {
    use std::io::Write;

    let test_name = normalized_test_name(test_name);
    let marker = format!("__substrate_harness_child__{role}__{test_name}");
    let is_child = std::env::args_os().any(|arg| arg == OsStr::new(&marker));
    if is_child {
        let sentinel = format!("SUBSTRATE_HARNESS_READY {role} {test_name}\n");
        std::io::stdout()
            .write_all(sentinel.as_bytes())
            .expect("publish isolated role readiness");
        std::io::stdout()
            .flush()
            .expect("flush isolated role readiness");
    }
    is_child
}

#[cfg(test)]
pub(crate) fn publish_test_subprocess_state_ready(test_name: &str, role: &str) {
    use std::io::Write;

    let test_name = normalized_test_name(test_name);
    let sentinel = format!("SUBSTRATE_HARNESS_STATE_READY {role} {test_name}\n");
    std::io::stdout()
        .write_all(sentinel.as_bytes())
        .expect("publish isolated child state readiness");
    std::io::stdout()
        .flush()
        .expect("flush isolated child state readiness");
}

#[cfg(test)]
pub(crate) fn run_bounded_test_subprocess_role(
    test_name: &str,
    role: &str,
    stdin: std::process::Stdio,
    expectation: TestSubprocessExpectation,
    require_state_ready: bool,
) {
    use std::io::Read;
    use std::process::Stdio;
    use std::time::{Duration, Instant};

    let test_name = normalized_test_name(test_name);
    let marker = format!("__substrate_harness_child__{role}__{test_name}");
    let ready_sentinel = format!("SUBSTRATE_HARNESS_READY {role} {test_name}\n");
    let state_sentinel = format!("SUBSTRATE_HARNESS_STATE_READY {role} {test_name}\n");
    let process_state = ProcessCwdTestGuard::preserve();
    let mut command = std::process::Command::new(
        std::env::current_exe().expect("resolve shell test binary for isolated role"),
    );
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;

        const CREATE_NEW_CONSOLE: u32 = 0x0000_0010;
        command.creation_flags(CREATE_NEW_CONSOLE);
    }
    let mut child = command
        .arg(test_name)
        .arg("--exact")
        .arg("--nocapture")
        .arg("--skip")
        .arg(&marker)
        .stdin(stdin)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn isolated shell test role");
    drop(process_state);

    let mut stdout = child.stdout.take().expect("isolated role stdout");
    let mut stderr = child.stderr.take().expect("isolated role stderr");
    let stdout_reader = std::thread::spawn(move || {
        let mut bytes = Vec::new();
        stdout
            .read_to_end(&mut bytes)
            .expect("drain isolated role stdout");
        bytes
    });
    let stderr_reader = std::thread::spawn(move || {
        let mut bytes = Vec::new();
        stderr
            .read_to_end(&mut bytes)
            .expect("drain isolated role stderr");
        bytes
    });

    let (deadline, expect_timeout) = match expectation {
        TestSubprocessExpectation::Timeout(duration) => (Instant::now() + duration, true),
        _ => (Instant::now() + Duration::from_secs(60), false),
    };
    let mut timed_out = false;
    let status = loop {
        if let Some(status) = child.try_wait().expect("poll isolated shell test role") {
            break status;
        }
        if Instant::now() >= deadline {
            timed_out = true;
            child
                .kill()
                .expect("kill timed-out isolated shell test role");
            break child
                .wait()
                .expect("reap timed-out isolated shell test role");
        }
        std::thread::yield_now();
    };

    let stdout = stdout_reader.join().expect("join isolated role stdout");
    let stderr = stderr_reader.join().expect("join isolated role stderr");
    let count = |needle: &[u8]| {
        stdout
            .windows(needle.len())
            .filter(|window| *window == needle)
            .count()
            + stderr
                .windows(needle.len())
                .filter(|window| *window == needle)
                .count()
    };
    assert_eq!(
        count(ready_sentinel.as_bytes()),
        1,
        "isolated role readiness mismatch: test={test_name} role={role} status={status} stdout_bytes={} stderr_bytes={}",
        stdout.len(),
        stderr.len()
    );
    if require_state_ready {
        assert_eq!(
            count(state_sentinel.as_bytes()),
            1,
            "isolated role state-readiness mismatch: test={test_name} role={role} status={status} stdout_bytes={} stderr_bytes={}",
            stdout.len(),
            stderr.len()
        );
    }

    match expectation {
        TestSubprocessExpectation::Success => assert!(
            status.success() && !timed_out,
            "isolated role did not succeed: test={test_name} role={role} status={status}"
        ),
        TestSubprocessExpectation::ExitCode(expected) => assert_eq!(
            status.code(),
            Some(expected),
            "isolated role exit mismatch: test={test_name} role={role} status={status}"
        ),
        #[cfg(windows)]
        TestSubprocessExpectation::Failure => assert!(
            !status.success() && !timed_out,
            "isolated role did not fail as required: test={test_name} role={role} status={status}"
        ),
        #[cfg(unix)]
        TestSubprocessExpectation::Signal(expected) => {
            use std::os::unix::process::ExitStatusExt;
            assert_eq!(
                status.signal(),
                Some(expected),
                "isolated role signal mismatch: test={test_name} role={role} status={status}"
            );
        }
        TestSubprocessExpectation::Timeout(_) => assert!(
            expect_timeout && timed_out && !status.success(),
            "isolated role timeout mismatch: test={test_name} role={role} status={status}"
        ),
    }
}

#[cfg(test)]
pub(crate) struct AuthorityEnvTestGuard {
    previous_home: Option<OsString>,
    previous_world_socket: Option<OsString>,
    previous_other: Vec<(&'static str, Option<OsString>)>,
    restored: bool,
    lock: Option<ReentrantMutexGuard<'static, ()>>,
}

#[cfg(test)]
impl AuthorityEnvTestGuard {
    pub(crate) fn preserve() -> Self {
        Self::apply(None, None)
    }

    pub(crate) fn try_preserve() -> Option<Self> {
        let lock = WORLD_ENV_LOCK
            .get_or_init(|| ReentrantMutex::new(()))
            .try_lock()?;
        let previous_home = std::env::var_os("SUBSTRATE_HOME");
        let previous_world_socket = std::env::var_os("SUBSTRATE_WORLD_SOCKET");
        let previous_other = AUTHORITY_ENVIRONMENT_NAMES
            .iter()
            .copied()
            .filter(|key| !matches!(*key, "SUBSTRATE_HOME" | "SUBSTRATE_WORLD_SOCKET"))
            .map(|key| (key, std::env::var_os(key)))
            .collect();
        Some(Self {
            previous_home,
            previous_world_socket,
            previous_other,
            restored: false,
            lock: Some(lock),
        })
    }

    pub(crate) fn set_home(value: impl AsRef<OsStr>) -> Self {
        Self::apply(Some(Some(value.as_ref())), None)
    }

    pub(crate) fn remove_home() -> Self {
        Self::apply(Some(None), None)
    }

    pub(crate) fn set_world_socket(value: impl AsRef<OsStr>) -> Self {
        Self::apply(None, Some(Some(value.as_ref())))
    }

    pub(crate) fn remove_world_socket() -> Self {
        Self::apply(None, Some(None))
    }

    pub(crate) fn set_both(home: impl AsRef<OsStr>, world_socket: impl AsRef<OsStr>) -> Self {
        Self::apply(Some(Some(home.as_ref())), Some(Some(world_socket.as_ref())))
    }

    pub(crate) fn remove_both() -> Self {
        Self::apply(Some(None), Some(None))
    }

    pub(crate) fn install_home(&self, value: impl AsRef<OsStr>) {
        std::env::set_var("SUBSTRATE_HOME", value);
    }

    fn apply(home: Option<Option<&OsStr>>, world_socket: Option<Option<&OsStr>>) -> Self {
        let lock = world_env_guard();
        let previous_home = std::env::var_os("SUBSTRATE_HOME");
        let previous_world_socket = std::env::var_os("SUBSTRATE_WORLD_SOCKET");
        let previous_other = AUTHORITY_ENVIRONMENT_NAMES
            .iter()
            .copied()
            .filter(|key| !matches!(*key, "SUBSTRATE_HOME" | "SUBSTRATE_WORLD_SOCKET"))
            .map(|key| (key, std::env::var_os(key)))
            .collect();
        Self::apply_value("SUBSTRATE_HOME", home);
        Self::apply_value("SUBSTRATE_WORLD_SOCKET", world_socket);
        Self {
            previous_home,
            previous_world_socket,
            previous_other,
            restored: false,
            lock: Some(lock),
        }
    }

    fn apply_value(key: &str, value: Option<Option<&OsStr>>) {
        match value {
            None => {}
            Some(Some(value)) => std::env::set_var(key, value),
            Some(None) => std::env::remove_var(key),
        }
    }

    fn restore_value(key: &str, value: Option<&OsStr>) {
        match value {
            Some(value) => std::env::set_var(key, value),
            None => std::env::remove_var(key),
        }
    }

    pub(crate) fn restore(&mut self) {
        if self.restored {
            return;
        }
        for (key, value) in self.previous_other.iter().rev() {
            Self::restore_value(key, value.as_deref());
        }
        Self::restore_value("SUBSTRATE_HOME", self.previous_home.as_deref());
        Self::restore_value(
            "SUBSTRATE_WORLD_SOCKET",
            self.previous_world_socket.as_deref(),
        );
        self.restored = true;
    }
}

#[cfg(test)]
impl Drop for AuthorityEnvTestGuard {
    fn drop(&mut self) {
        self.restore();
        if let Some(lock) = self.lock.take() {
            ReentrantMutexGuard::unlock_fair(lock);
        }
    }
}

#[cfg(test)]
pub(crate) struct WorldSocketTestGuard {
    previous: Option<OsString>,
    _authority: AuthorityEnvTestGuard,
}

#[cfg(test)]
pub(crate) struct AuthorityEnvTestTempDir {
    temp: Option<tempfile::TempDir>,
    authority: Option<AuthorityEnvTestGuard>,
}

#[cfg(test)]
impl AuthorityEnvTestTempDir {
    pub(crate) fn new(temp: tempfile::TempDir) -> Self {
        Self {
            temp: Some(temp),
            authority: Some(AuthorityEnvTestGuard::preserve()),
        }
    }

    pub(crate) fn with_authority(
        temp: tempfile::TempDir,
        authority: AuthorityEnvTestGuard,
    ) -> Self {
        Self {
            temp: Some(temp),
            authority: Some(authority),
        }
    }

    pub(crate) fn install_as_home(&self) {
        self.authority
            .as_ref()
            .expect("authority tempdir guard")
            .install_home(self.path());
    }
}

#[cfg(test)]
impl std::ops::Deref for AuthorityEnvTestTempDir {
    type Target = tempfile::TempDir;

    fn deref(&self) -> &Self::Target {
        self.temp.as_ref().expect("authority tempdir")
    }
}

#[cfg(test)]
impl Drop for AuthorityEnvTestTempDir {
    fn drop(&mut self) {
        drop(self.temp.take());
        drop(self.authority.take());
    }
}

#[cfg(test)]
impl WorldSocketTestGuard {
    pub(crate) fn set(value: impl AsRef<OsStr>) -> Self {
        Self::set_optional(Some(value.as_ref()))
    }

    pub(crate) fn remove() -> Self {
        Self::set_optional(None)
    }

    pub(crate) fn set_optional(value: Option<&OsStr>) -> Self {
        let authority = match value {
            Some(value) => AuthorityEnvTestGuard::set_world_socket(value),
            None => AuthorityEnvTestGuard::remove_world_socket(),
        };
        let previous = authority.previous_world_socket.clone();
        Self {
            previous,
            _authority: authority,
        }
    }
}

#[cfg(test)]
mod world_socket_test_guard_tests {
    use super::{
        AuthorityEnvTestGuard, AuthorityEnvTestTempDir, WorldSocketTestGuard,
        AUTHORITY_ENVIRONMENT_NAMES,
    };
    use std::ffi::{OsStr, OsString};
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::{mpsc, Arc};
    use std::time::Duration;

    #[test]
    fn world_socket_test_guard_restores_prior_absence() {
        let outer = WorldSocketTestGuard::remove();
        {
            let _inner = WorldSocketTestGuard::set(OsStr::new("/tmp/f0-inner.sock"));
            assert_eq!(
                std::env::var_os("SUBSTRATE_WORLD_SOCKET"),
                Some(OsString::from("/tmp/f0-inner.sock"))
            );
        }
        assert_eq!(std::env::var_os("SUBSTRATE_WORLD_SOCKET"), None);
        drop(outer);
    }

    #[test]
    fn authority_env_test_guard_restores_home_and_socket_as_one_snapshot() {
        let outer = AuthorityEnvTestGuard::remove_both();
        {
            let _inner = AuthorityEnvTestGuard::set_both(
                OsStr::new("/tmp/f0a-home"),
                OsStr::new("/tmp/f0a-world.sock"),
            );
            assert_eq!(
                std::env::var_os("SUBSTRATE_HOME"),
                Some(OsString::from("/tmp/f0a-home"))
            );
            assert_eq!(
                std::env::var_os("SUBSTRATE_WORLD_SOCKET"),
                Some(OsString::from("/tmp/f0a-world.sock"))
            );
        }
        assert_eq!(std::env::var_os("SUBSTRATE_HOME"), None);
        assert_eq!(std::env::var_os("SUBSTRATE_WORLD_SOCKET"), None);
        drop(outer);
    }

    #[test]
    fn authority_env_test_guard_preserve_restores_direct_mutations() {
        let outer = AuthorityEnvTestGuard::remove_both();
        {
            let _inner = AuthorityEnvTestGuard::preserve();
            std::env::set_var("SUBSTRATE_HOME", "/tmp/f0a-direct-home");
            std::env::set_var("SUBSTRATE_WORLD_SOCKET", "/tmp/f0a-direct.sock");
        }
        assert_eq!(std::env::var_os("SUBSTRATE_HOME"), None);
        assert_eq!(std::env::var_os("SUBSTRATE_WORLD_SOCKET"), None);
        drop(outer);
    }

    #[test]
    fn authority_env_test_guard_preserves_unicode_and_empty_values() {
        let outer =
            AuthorityEnvTestGuard::set_both(OsStr::new("/tmp/authority-雪"), OsStr::new(""));
        {
            let _inner = AuthorityEnvTestGuard::remove_both();
        }
        assert_eq!(
            std::env::var_os("SUBSTRATE_HOME"),
            Some(OsString::from("/tmp/authority-雪"))
        );
        assert_eq!(
            std::env::var_os("SUBSTRATE_WORLD_SOCKET"),
            Some(OsString::new())
        );
        drop(outer);
    }

    #[cfg(unix)]
    #[test]
    fn authority_env_test_guard_restores_exact_non_unicode_pair() {
        use std::os::unix::ffi::OsStringExt;

        assert_eq!(AUTHORITY_ENVIRONMENT_NAMES.len(), 74);
        for key in [
            "XDG_CONFIG_HOME",
            "XDG_DATA_HOME",
            "XDG_STATE_HOME",
            "SUBSTRATE_OVERRIDE_ANCHOR_MODE",
            "SUBSTRATE_OVERRIDE_ANCHOR_PATH",
            "SUBSTRATE_OVERRIDE_CAGED",
        ] {
            assert!(
                AUTHORITY_ENVIRONMENT_NAMES.contains(&key),
                "parent-mutated environment key is not coordinated: {key}"
            );
        }
        for key in [
            "ANTHROPIC_API_KEY",
            "BASH_ENV",
            "SUBSTRATE_A1_REPLACEMENT_CHILD_ROOT",
            "SUBSTRATE_AGENT_TOOLBOX_ENDPOINT",
            "SUBSTRATE_AGENT_TOOLBOX_VERSION",
            "SUBSTRATE_ENABLE_PREEXEC",
            "SUBSTRATE_MANAGER_ENV_ACTIVE",
            "SUBSTRATE_ORIGINAL_BASH_ENV",
            "SUBSTRATE_PARENT_SPAN_ID",
            "SUBSTRATE_R0_RETAINED_SUBPROCESS_VARIANT",
            "SUBSTRATE_WORLD_FS_ENFORCEMENT_PLAN_B64",
            "SUBSTRATE_WORLD_PROJECT_DIR",
        ] {
            assert!(
                !AUTHORITY_ENVIRONMENT_NAMES.contains(&key),
                "non-parent environment key entered the parent coordinator: {key}"
            );
        }

        let prior_home = OsString::from_vec(b"/tmp/f0a-home-\xff".to_vec());
        let prior_socket = OsString::from_vec(b"/tmp/f0-socket-\xfe".to_vec());
        let prior_force_pty = OsString::from_vec(b"f0-harness-\xfd".to_vec());
        let outer = AuthorityEnvTestGuard::set_both(&prior_home, &prior_socket);
        std::env::set_var("SUBSTRATE_FORCE_PTY", &prior_force_pty);
        {
            let _inner = AuthorityEnvTestGuard::set_both(
                OsStr::new("/tmp/f0a-inner-home"),
                OsStr::new("/tmp/f0-inner.sock"),
            );
            std::env::remove_var("SUBSTRATE_FORCE_PTY");
        }
        assert_eq!(std::env::var_os("SUBSTRATE_HOME"), Some(prior_home));
        assert_eq!(
            std::env::var_os("SUBSTRATE_WORLD_SOCKET"),
            Some(prior_socket)
        );
        assert_eq!(
            std::env::var_os("SUBSTRATE_FORCE_PTY"),
            Some(prior_force_pty)
        );
        drop(outer);
    }

    #[test]
    fn authority_env_test_guard_nested_home_override_preserves_outer_pair() {
        let outer = AuthorityEnvTestGuard::remove_both();
        {
            let _middle = AuthorityEnvTestGuard::set_both(
                OsStr::new("/tmp/f0a-outer-home"),
                OsStr::new("/tmp/f0-outer.sock"),
            );
            {
                let _inner = AuthorityEnvTestGuard::set_home(OsStr::new("/tmp/f0a-inner-home"));
                assert_eq!(
                    std::env::var_os("SUBSTRATE_WORLD_SOCKET"),
                    Some(OsString::from("/tmp/f0-outer.sock"))
                );
            }
            assert_eq!(
                std::env::var_os("SUBSTRATE_HOME"),
                Some(OsString::from("/tmp/f0a-outer-home"))
            );
            assert_eq!(
                std::env::var_os("SUBSTRATE_WORLD_SOCKET"),
                Some(OsString::from("/tmp/f0-outer.sock"))
            );
        }
        assert_eq!(std::env::var_os("SUBSTRATE_HOME"), None);
        assert_eq!(std::env::var_os("SUBSTRATE_WORLD_SOCKET"), None);
        drop(outer);
    }

    #[test]
    fn authority_env_test_guard_restores_pair_after_panic_without_poisoning() {
        let outer = AuthorityEnvTestGuard::set_both(
            OsStr::new("/tmp/f0a-prior-home"),
            OsStr::new("/tmp/f0-prior.sock"),
        );
        let panic_result = std::panic::catch_unwind(|| {
            let _inner = AuthorityEnvTestGuard::remove_both();
            panic!("intentional authority-environment guard panic");
        });
        assert!(panic_result.is_err());
        assert_eq!(
            std::env::var_os("SUBSTRATE_HOME"),
            Some(OsString::from("/tmp/f0a-prior-home"))
        );
        assert_eq!(
            std::env::var_os("SUBSTRATE_WORLD_SOCKET"),
            Some(OsString::from("/tmp/f0-prior.sock"))
        );

        {
            let _later = AuthorityEnvTestGuard::remove_home();
            assert_eq!(std::env::var_os("SUBSTRATE_HOME"), None);
        }
        assert_eq!(
            std::env::var_os("SUBSTRATE_HOME"),
            Some(OsString::from("/tmp/f0a-prior-home"))
        );
        drop(outer);
    }

    #[test]
    fn authority_env_test_guard_panic_releases_boundary_for_cross_thread_reacquisition() {
        let (prior_tx, prior_rx) = mpsc::channel();
        let owner = std::thread::spawn(move || {
            let guard = AuthorityEnvTestGuard::set_both(
                OsStr::new("/tmp/f0a-panicking-owner-home"),
                OsStr::new("/tmp/f0-panicking-owner.sock"),
            );
            prior_tx
                .send((
                    guard.previous_home.clone(),
                    guard.previous_world_socket.clone(),
                ))
                .expect("send panicking owner prior pair");
            panic!("intentional cross-thread authority-environment panic");
        });
        let prior = prior_rx.recv().expect("receive panicking owner prior pair");
        assert!(owner.join().is_err());

        let later = std::thread::spawn(move || {
            let _guard = AuthorityEnvTestGuard::preserve();
            assert_eq!(std::env::var_os("SUBSTRATE_HOME"), prior.0);
            assert_eq!(std::env::var_os("SUBSTRATE_WORLD_SOCKET"), prior.1);
        });
        later
            .join()
            .expect("cross-thread reacquisition after owner panic");
    }

    #[cfg(unix)]
    #[test]
    fn authority_env_test_guard_bounds_subprocess_inheritance_through_wait_and_output() {
        let outer = AuthorityEnvTestGuard::remove_both();
        {
            let _inner = AuthorityEnvTestGuard::set_both(
                OsStr::new("/tmp/f0a-child-home"),
                OsStr::new("/tmp/f0-child.sock"),
            );
            let output = std::process::Command::new("sh")
                .args([
                    "-c",
                    "printf '%s\\n%s\\n' \"$SUBSTRATE_HOME\" \"$SUBSTRATE_WORLD_SOCKET\"",
                ])
                .output()
                .expect("run authority environment child");
            assert!(output.status.success());
            assert_eq!(output.stdout, b"/tmp/f0a-child-home\n/tmp/f0-child.sock\n");
        }
        assert_eq!(std::env::var_os("SUBSTRATE_HOME"), None);
        assert_eq!(std::env::var_os("SUBSTRATE_WORLD_SOCKET"), None);
        drop(outer);
    }

    #[test]
    fn authority_env_tempdir_cleanup_precedes_unlock_during_panic() {
        let fixture = AuthorityEnvTestTempDir::new(tempfile::tempdir().expect("authority tempdir"));
        let fixture_path = fixture.path().to_path_buf();
        fixture.install_as_home();
        let (competitor_started_tx, competitor_started_rx) = mpsc::channel();
        let (competitor_acquired_tx, competitor_acquired_rx) = mpsc::channel();
        let path_for_competitor = fixture_path.clone();
        let competitor = std::thread::spawn(move || {
            competitor_started_tx.send(()).expect("competitor started");
            let _guard = AuthorityEnvTestGuard::preserve();
            assert!(
                !path_for_competitor.exists(),
                "competitor acquired before authority tempdir cleanup"
            );
            competitor_acquired_tx
                .send(())
                .expect("competitor acquired");
        });
        competitor_started_rx.recv().expect("competitor started");
        assert!(
            competitor_acquired_rx
                .recv_timeout(Duration::from_millis(100))
                .is_err(),
            "competitor acquired while authority tempdir boundary was live"
        );

        let panic_result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(move || {
            let _fixture = fixture;
            panic!("intentional authority tempdir panic");
        }));
        assert!(panic_result.is_err());
        competitor_acquired_rx
            .recv_timeout(Duration::from_secs(5))
            .expect("competitor acquired after authority tempdir cleanup");
        competitor.join().expect("authority tempdir competitor");
        assert!(!fixture_path.exists());
    }

    #[cfg(unix)]
    #[tokio::test(flavor = "current_thread")]
    async fn authority_env_test_guard_holds_boundary_through_aborted_server_cleanup() {
        struct ServerTerminationFlag(Arc<AtomicBool>);

        impl Drop for ServerTerminationFlag {
            fn drop(&mut self) {
                self.0.store(true, Ordering::SeqCst);
            }
        }

        let server_terminated = Arc::new(AtomicBool::new(false));
        let fixture_cleaned = Arc::new(AtomicBool::new(false));
        let boundary = AuthorityEnvTestGuard::set_both(
            OsStr::new("/tmp/f0a-async-owner-home"),
            OsStr::new("/tmp/f0-async-owner.sock"),
        );
        let socket_home = tempfile::tempdir().expect("async socket tempdir");
        let socket_path = socket_home.path().join("world.sock");
        let listener = tokio::net::UnixListener::bind(&socket_path).expect("bind async socket");
        let (started_tx, started_rx) = tokio::sync::oneshot::channel();
        let termination_for_server = Arc::clone(&server_terminated);
        let server = tokio::spawn(async move {
            let _termination = ServerTerminationFlag(termination_for_server);
            let _listener = listener;
            let _ = started_tx.send(());
            std::future::pending::<()>().await;
        });
        started_rx.await.expect("async server started");

        let (competitor_started_tx, competitor_started_rx) = mpsc::channel();
        let (competitor_acquired_tx, competitor_acquired_rx) = mpsc::channel();
        let termination_for_competitor = Arc::clone(&server_terminated);
        let cleanup_for_competitor = Arc::clone(&fixture_cleaned);
        let competitor = std::thread::spawn(move || {
            competitor_started_tx.send(()).expect("competitor started");
            let _guard = AuthorityEnvTestGuard::set_both(
                OsStr::new("/tmp/f0a-async-competitor-home"),
                OsStr::new("/tmp/f0-async-competitor.sock"),
            );
            assert!(termination_for_competitor.load(Ordering::SeqCst));
            assert!(cleanup_for_competitor.load(Ordering::SeqCst));
            competitor_acquired_tx
                .send(())
                .expect("competitor acquisition signal");
        });
        competitor_started_rx.recv().expect("competitor started");

        server.abort();
        assert!(
            competitor_acquired_rx
                .recv_timeout(Duration::from_millis(100))
                .is_err(),
            "competitor acquired after abort but before awaited server termination"
        );
        match server.await {
            Err(error) if error.is_cancelled() => {}
            Ok(()) => panic!("async server completed normally instead of confirming cancellation"),
            Err(error) => panic!("async server join failed unexpectedly: {error}"),
        }
        assert!(server_terminated.load(Ordering::SeqCst));
        assert!(
            std::os::unix::net::UnixStream::connect(&socket_path).is_err(),
            "terminated server still accepted socket connections"
        );
        drop(socket_home);
        assert!(!socket_path.exists());
        fixture_cleaned.store(true, Ordering::SeqCst);
        assert!(
            competitor_acquired_rx
                .recv_timeout(Duration::from_millis(100))
                .is_err(),
            "competitor acquired before authority restoration and unlock"
        );
        drop(boundary);
        competitor_acquired_rx
            .recv_timeout(Duration::from_secs(5))
            .expect("competitor acquired after async cleanup and unlock");
        competitor.join().expect("async cleanup competitor");
    }

    #[cfg(unix)]
    #[test]
    fn world_socket_test_guard_restores_exact_non_unicode_prior_value() {
        use std::os::unix::ffi::OsStringExt;

        let prior = OsString::from_vec(b"/tmp/f0-prior-\xff.sock".to_vec());
        let outer = WorldSocketTestGuard::set(prior.as_os_str());
        {
            let _inner = WorldSocketTestGuard::set(OsStr::new("/tmp/f0-inner.sock"));
        }
        assert_eq!(std::env::var_os("SUBSTRATE_WORLD_SOCKET"), Some(prior));
        drop(outer);
    }

    #[test]
    fn world_socket_test_guard_restores_after_panic_and_allows_reacquisition() {
        let outer = WorldSocketTestGuard::remove();
        let panic_result = std::panic::catch_unwind(|| {
            let _inner = WorldSocketTestGuard::set(OsStr::new("/tmp/f0-panic.sock"));
            panic!("intentional world-socket guard panic");
        });
        assert!(panic_result.is_err());
        assert_eq!(std::env::var_os("SUBSTRATE_WORLD_SOCKET"), None);

        {
            let _later = WorldSocketTestGuard::set(OsStr::new("/tmp/f0-reacquired.sock"));
            assert_eq!(
                std::env::var_os("SUBSTRATE_WORLD_SOCKET"),
                Some(OsString::from("/tmp/f0-reacquired.sock"))
            );
        }
        assert_eq!(std::env::var_os("SUBSTRATE_WORLD_SOCKET"), None);
        drop(outer);
    }

    #[test]
    fn world_socket_test_guard_nested_acquisition_restores_in_stack_order() {
        let outer = WorldSocketTestGuard::set(OsStr::new("/tmp/f0-outer.sock"));
        {
            let _inner = WorldSocketTestGuard::set(OsStr::new("/tmp/f0-inner.sock"));
            assert_eq!(
                std::env::var_os("SUBSTRATE_WORLD_SOCKET"),
                Some(OsString::from("/tmp/f0-inner.sock"))
            );
        }
        assert_eq!(
            std::env::var_os("SUBSTRATE_WORLD_SOCKET"),
            Some(OsString::from("/tmp/f0-outer.sock"))
        );
        drop(outer);
    }

    #[test]
    fn world_socket_test_guard_blocks_competitor_until_cleanup_and_restoration() {
        let cleanup_complete = Arc::new(AtomicBool::new(false));
        let (owner_ready_tx, owner_ready_rx) = mpsc::channel();
        let (release_owner_tx, release_owner_rx) = mpsc::channel();
        let owner_cleanup = Arc::clone(&cleanup_complete);
        let owner = std::thread::spawn(move || {
            let guard = WorldSocketTestGuard::set(OsStr::new("/tmp/f0-owner.sock"));
            owner_ready_tx
                .send(guard.previous.clone())
                .expect("signal owner ready");
            release_owner_rx.recv().expect("release owner");
            owner_cleanup.store(true, Ordering::SeqCst);
            drop(guard);
        });
        let original = owner_ready_rx.recv().expect("owner prior state");

        let (competitor_started_tx, competitor_started_rx) = mpsc::channel();
        let (competitor_acquired_tx, competitor_acquired_rx) = mpsc::channel();
        let competitor_cleanup = Arc::clone(&cleanup_complete);
        let competitor = std::thread::spawn(move || {
            competitor_started_tx.send(()).expect("signal competitor");
            let guard = WorldSocketTestGuard::set(OsStr::new("/tmp/f0-competitor.sock"));
            assert!(
                competitor_cleanup.load(Ordering::SeqCst),
                "competitor acquired before owner cleanup"
            );
            assert_eq!(
                std::env::var_os("SUBSTRATE_WORLD_SOCKET"),
                Some(OsString::from("/tmp/f0-competitor.sock"))
            );
            competitor_acquired_tx
                .send(guard.previous.clone())
                .expect("signal competitor acquisition");
            drop(guard);
        });

        competitor_started_rx.recv().expect("competitor started");
        assert!(
            competitor_acquired_rx
                .recv_timeout(Duration::from_millis(100))
                .is_err(),
            "competitor acquired while owner held the guard"
        );
        release_owner_tx.send(()).expect("release owner");
        assert_eq!(
            competitor_acquired_rx
                .recv_timeout(Duration::from_secs(5))
                .expect("competitor eventually acquired"),
            original
        );
        owner.join().expect("owner thread");
        competitor.join().expect("competitor thread");

        let probe = WorldSocketTestGuard::remove();
        assert_eq!(probe.previous, original);
        drop(probe);
    }

    #[test]
    fn authority_env_test_guard_blocks_competitor_until_pair_cleanup_and_restoration() {
        let cleanup_complete = Arc::new(AtomicBool::new(false));
        let (owner_ready_tx, owner_ready_rx) = mpsc::channel();
        let (release_owner_tx, release_owner_rx) = mpsc::channel();
        let owner_cleanup = Arc::clone(&cleanup_complete);
        let owner = std::thread::spawn(move || {
            let guard = AuthorityEnvTestGuard::set_both(
                OsStr::new("/tmp/f0a-owner-home"),
                OsStr::new("/tmp/f0-owner.sock"),
            );
            owner_ready_tx
                .send((
                    guard.previous_home.clone(),
                    guard.previous_world_socket.clone(),
                ))
                .expect("signal authority owner ready");
            release_owner_rx.recv().expect("release authority owner");
            owner_cleanup.store(true, Ordering::SeqCst);
            drop(guard);
        });
        let original = owner_ready_rx.recv().expect("owner prior authority pair");

        let (competitor_started_tx, competitor_started_rx) = mpsc::channel();
        let (competitor_acquired_tx, competitor_acquired_rx) = mpsc::channel();
        let competitor_cleanup = Arc::clone(&cleanup_complete);
        let competitor = std::thread::spawn(move || {
            competitor_started_tx.send(()).expect("signal competitor");
            let guard = AuthorityEnvTestGuard::set_both(
                OsStr::new("/tmp/f0a-competitor-home"),
                OsStr::new("/tmp/f0-competitor.sock"),
            );
            assert!(
                competitor_cleanup.load(Ordering::SeqCst),
                "competitor acquired before authority owner cleanup"
            );
            assert_eq!(
                std::env::var_os("SUBSTRATE_HOME"),
                Some(OsString::from("/tmp/f0a-competitor-home"))
            );
            assert_eq!(
                std::env::var_os("SUBSTRATE_WORLD_SOCKET"),
                Some(OsString::from("/tmp/f0-competitor.sock"))
            );
            competitor_acquired_tx
                .send((
                    guard.previous_home.clone(),
                    guard.previous_world_socket.clone(),
                ))
                .expect("signal authority competitor acquisition");
            drop(guard);
        });

        competitor_started_rx.recv().expect("competitor started");
        assert!(
            competitor_acquired_rx
                .recv_timeout(Duration::from_millis(100))
                .is_err(),
            "competitor acquired while owner held the authority boundary"
        );
        release_owner_tx.send(()).expect("release authority owner");
        assert_eq!(
            competitor_acquired_rx
                .recv_timeout(Duration::from_secs(5))
                .expect("authority competitor eventually acquired"),
            original
        );
        owner.join().expect("authority owner thread");
        competitor.join().expect("authority competitor thread");

        let probe = AuthorityEnvTestGuard::preserve();
        assert_eq!(
            (
                probe.previous_home.clone(),
                probe.previous_world_socket.clone()
            ),
            original
        );
        drop(probe);
    }
}
