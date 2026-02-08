#![cfg(all(unix, target_os = "linux"))]

mod support;

use support::{AgentSocket, ShellEnvFixture, SocketResponse};

use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tempfile::{Builder, TempDir};

struct Waps0008Fixture {
    shell: ShellEnvFixture,
    _socket_dir: TempDir,
    project_dir: PathBuf,
    socket_path: PathBuf,
}

impl Waps0008Fixture {
    fn new() -> Self {
        let shell = ShellEnvFixture::new();
        let socket_dir = Builder::new()
            .prefix("substrate-waps-0008-sock-")
            .tempdir_in("/tmp")
            .expect("failed to create socket tempdir");
        let socket_path = socket_dir.path().join("substrate.sock");
        let project_dir = shell.home().join("project");
        fs::create_dir_all(&project_dir).expect("failed to create project dir");

        let workspace_substrate_dir = project_dir.join(".substrate");
        fs::create_dir_all(&workspace_substrate_dir)
            .expect("failed to create workspace .substrate");

        Self {
            shell,
            _socket_dir: socket_dir,
            project_dir,
            socket_path,
        }
    }

    fn substrate_home(&self) -> PathBuf {
        self.shell.home().join(".substrate")
    }

    fn workspace_config_path(&self) -> PathBuf {
        self.project_dir.join(".substrate").join("workspace.yaml")
    }

    fn workspace_policy_path(&self) -> PathBuf {
        self.project_dir.join(".substrate").join("policy.yaml")
    }

    fn socket_path(&self) -> &Path {
        &self.socket_path
    }

    fn write_global_config_patch(&self, contents: &str) {
        let substrate_home = self.substrate_home();
        fs::create_dir_all(&substrate_home).expect("failed to create SUBSTRATE_HOME");
        fs::write(substrate_home.join("config.yaml"), contents).expect("write config.yaml");
    }

    fn write_workspace_config_patch(&self, contents: &str) {
        fs::write(self.workspace_config_path(), contents).expect("write workspace.yaml");
    }

    fn write_workspace_policy_patch(&self, contents: &str) {
        fs::write(self.workspace_policy_path(), contents).expect("write workspace policy.yaml");
    }

    fn spawn_pipe_shell(&self, trace_log: &Path) -> std::process::Child {
        support::ensure_substrate_built();

        let mut cmd = Command::new(support::binary_path());
        cmd.stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .current_dir(&self.project_dir)
            .env("TMPDIR", support::common::shared_tmpdir())
            .env("HOME", self.shell.home())
            .env("USERPROFILE", self.shell.home())
            .env("SHELL", "/bin/bash")
            .env("SUBSTRATE_HOME", self.substrate_home())
            .env("SHIM_TRACE_LOG", trace_log)
            .env("SUBSTRATE_WORLD_SOCKET", self.socket_path())
            .env_remove("SUBSTRATE_WORLD")
            .env_remove("SUBSTRATE_WORLD_ENABLED")
            .env_remove("SUBSTRATE_WORLD_ID")
            .env("SUBSTRATE_OVERRIDE_WORLD", "enabled")
            .env_remove("SUBSTRATE_NO_SHIMS")
            .env_remove("SUBSTRATE_SHIM_PATH")
            .env_remove("SUBSTRATE_SHIM_ORIGINAL_PATH")
            .env_remove("SUBSTRATE_SHIM_DEPLOY_DIR")
            .env_remove("SHIM_ORIGINAL_PATH")
            .env_remove("PATH_BEFORE_SUBSTRATE_SHIM")
            .arg("--shim-skip");

        cmd.spawn().expect("spawn substrate pipe shell")
    }
}

fn wait_for_records(
    records: &Arc<Mutex<Vec<serde_json::Value>>>,
    expected: usize,
    timeout: Duration,
) {
    let start = Instant::now();
    while start.elapsed() < timeout {
        let len = records.lock().map(|guard| guard.len()).unwrap_or(0);
        if len >= expected {
            return;
        }
        std::thread::sleep(Duration::from_millis(10));
    }
    let len = records.lock().map(|guard| guard.len()).unwrap_or(0);
    panic!("timed out waiting for {expected} recorded execute requests; got {len}");
}

#[test]
fn waps_0008_policy_patch_edits_visible_to_next_command() {
    let fixture = Waps0008Fixture::new();

    // Workspace marker file + initial config. Keep policy.mode non-disabled so policy resolution
    // always runs before sending a world-agent request.
    fixture.write_workspace_config_patch("{}\n");
    fixture.write_global_config_patch(
        "world:\n  enabled: true\n  anchor_mode: follow-cwd\n  anchor_path: \"\"\n  caged: false\n\npolicy:\n  mode: observe\n\nsync:\n  auto_sync: false\n  direction: from_world\n  conflict_policy: prefer_host\n  exclude: []\n",
    );

    // Start with a writable world_fs policy so the snapshot is `writable`.
    fixture.write_workspace_policy_patch(
        "world_fs:\n  host_visible: true\n  fail_closed:\n    routing: false\n  write:\n    enabled: true\n",
    );

    let records: Arc<Mutex<Vec<serde_json::Value>>> = Arc::new(Mutex::new(Vec::new()));
    let _socket = AgentSocket::start(
        fixture.socket_path(),
        SocketResponse::CapabilitiesAndHostExecuteRecord {
            scopes: vec![],
            records: records.clone(),
        },
    );

    let trace_log = fixture.shell.home().join(".substrate/trace.jsonl");
    let mut child = fixture.spawn_pipe_shell(&trace_log);
    let mut stdin = child.stdin.take().expect("child stdin");

    writeln!(stdin, "echo first").expect("write first command");
    stdin.flush().expect("flush first command");
    wait_for_records(&records, 1, Duration::from_secs(5));

    // Flip to read_only + require_world. Change file size to guarantee cache invalidation even if
    // filesystem mtime is coarse.
    fixture.write_workspace_policy_patch(
        "world_fs:\n  host_visible: true\n  write:\n    enabled: false\n  fail_closed:\n    routing: true\n\nmetadata:\n  test_marker: waps-0008\n",
    );

    writeln!(stdin, "echo second").expect("write second command");
    stdin.flush().expect("flush second command");
    wait_for_records(&records, 2, Duration::from_secs(5));

    drop(stdin);
    let output = child.wait_with_output().expect("wait for substrate");
    assert!(
        output.status.success(),
        "expected substrate pipe shell to exit successfully: {output:?}"
    );

    let records = records.lock().expect("records lock");
    assert_eq!(
        records.len(),
        2,
        "expected exactly two world-agent execute requests"
    );
    let mode_1 = records[0]
        .pointer("/policy_snapshot/world_fs/mode")
        .and_then(|v| v.as_str())
        .unwrap_or("<missing>");
    let mode_2 = records[1]
        .pointer("/policy_snapshot/world_fs/mode")
        .and_then(|v| v.as_str())
        .unwrap_or("<missing>");
    assert_eq!(
        mode_1, "writable",
        "first command should use writable snapshot"
    );
    assert_eq!(
        mode_2, "read_only",
        "second command should reflect edited policy patch"
    );
}

#[test]
fn waps_0008_config_edits_visible_to_next_command() {
    let fixture = Waps0008Fixture::new();

    // Start with observe mode from global config; workspace patch is empty.
    fixture.write_workspace_config_patch("{}\n");
    fixture.write_global_config_patch(
        "world:\n  enabled: true\n  anchor_mode: follow-cwd\n  anchor_path: \"\"\n  caged: false\n\npolicy:\n  mode: observe\n\nsync:\n  auto_sync: false\n  direction: from_world\n  conflict_policy: prefer_host\n  exclude: []\n",
    );
    fixture.write_workspace_policy_patch(
        "world_fs:\n  host_visible: true\n  fail_closed:\n    routing: false\n  write:\n    enabled: true\n",
    );

    let records: Arc<Mutex<Vec<serde_json::Value>>> = Arc::new(Mutex::new(Vec::new()));
    let _socket = AgentSocket::start(
        fixture.socket_path(),
        SocketResponse::CapabilitiesAndHostExecuteRecord {
            scopes: vec![],
            records: records.clone(),
        },
    );

    let trace_log = fixture.shell.home().join(".substrate/trace.jsonl");
    let mut child = fixture.spawn_pipe_shell(&trace_log);
    let mut stdin = child.stdin.take().expect("child stdin");

    writeln!(stdin, "echo first").expect("write first command");
    stdin.flush().expect("flush first command");
    wait_for_records(&records, 1, Duration::from_secs(5));

    // Edit both config.yaml and workspace.yaml; workspace patch should take precedence.
    fixture.write_global_config_patch(
        "world:\n  enabled: true\n  anchor_mode: follow-cwd\n  anchor_path: \"\"\n  caged: false\n\npolicy:\n  mode: disabled\n\nsync:\n  auto_sync: false\n  direction: from_world\n  conflict_policy: prefer_host\n  exclude: []\n\n# waps-0008-global\n",
    );
    fixture.write_workspace_config_patch("policy:\n  mode: enforce\n\n# waps-0008-workspace\n");

    writeln!(stdin, "echo second").expect("write second command");
    stdin.flush().expect("flush second command");
    wait_for_records(&records, 2, Duration::from_secs(5));

    drop(stdin);
    let output = child.wait_with_output().expect("wait for substrate");
    assert!(
        output.status.success(),
        "expected substrate pipe shell to exit successfully: {output:?}"
    );

    let records = records.lock().expect("records lock");
    assert_eq!(
        records.len(),
        2,
        "expected exactly two world-agent execute requests"
    );

    let mode_1 = records[0]
        .pointer("/env/SUBSTRATE_POLICY_MODE")
        .and_then(|v| v.as_str())
        .unwrap_or("<missing>");
    let mode_2 = records[1]
        .pointer("/env/SUBSTRATE_POLICY_MODE")
        .and_then(|v| v.as_str())
        .unwrap_or("<missing>");

    assert_eq!(
        mode_1, "observe",
        "first command should use policy.mode=observe from config.yaml"
    );
    assert_eq!(
        mode_2, "enforce",
        "second command should reflect edited config.yaml + workspace.yaml"
    );
}
