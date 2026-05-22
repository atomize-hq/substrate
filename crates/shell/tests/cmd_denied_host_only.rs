#![cfg(all(unix, target_os = "linux"))]

mod support;

use support::{substrate_command_for_home, AgentSocket, ShellEnvFixture, SocketResponse};

use assert_cmd::Command;
use std::fs;
use std::path::{Path, PathBuf};
use tempfile::{Builder, TempDir};

struct CmdDeniedHostOnlyFixture {
    shell: ShellEnvFixture,
    _socket_dir: TempDir,
    project_dir: PathBuf,
    socket_path: PathBuf,
}

impl CmdDeniedHostOnlyFixture {
    fn new() -> Self {
        let shell = ShellEnvFixture::new();
        let socket_dir = Builder::new()
            .prefix("substrate-cmd-denied-sock-")
            .tempdir_in("/tmp")
            .expect("failed to create socket tempdir");
        let socket_path = socket_dir.path().join("substrate.sock");
        let project_dir = shell.home().join("project");
        fs::create_dir_all(&project_dir).expect("failed to create project dir");
        fs::create_dir_all(project_dir.join(".substrate"))
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

    fn socket_path(&self) -> &Path {
        &self.socket_path
    }

    fn write_config(&self) {
        let substrate_home = self.substrate_home();
        fs::create_dir_all(&substrate_home).expect("failed to create SUBSTRATE_HOME");
        fs::write(
            substrate_home.join("config.yaml"),
            "world:\n  enabled: true\n  anchor_mode: follow-cwd\n  anchor_path: \"\"\n  caged: false\n\npolicy:\n  mode: enforce\n\nsync:\n  auto_sync: false\n  direction: from_world\n  conflict_policy: prefer_host\n  exclude: []\n",
        )
        .expect("failed to write config.yaml");
        fs::write(self.workspace_config_path(), "world:\n  enabled: true\n")
            .expect("failed to write workspace.yaml");
    }

    fn write_global_policy_patch_cmd_denied(&self) {
        let substrate_home = self.substrate_home();
        fs::create_dir_all(&substrate_home).expect("failed to create SUBSTRATE_HOME");
        fs::write(
            substrate_home.join("policy.yaml"),
            r#"id: "waps-0007-test"
name: "WAPS-0007 cmd_denied host-only regression"

world_fs:
  host_visible: true
  fail_closed:
    routing: true
  write:
    enabled: true

net_allowed: []
cmd_allowed: []
cmd_denied:
  - "echo*"
cmd_isolated: []

require_approval: false
allow_shell_operators: true

limits:
  max_memory_mb: null
  max_cpu_percent: null
  max_runtime_ms: null
  max_egress_bytes: null

metadata: {}
"#,
        )
        .expect("failed to write policy.yaml");
    }

    fn command(&self) -> Command {
        let mut cmd = substrate_command_for_home(&self.shell);
        cmd.current_dir(&self.project_dir)
            .env("SUBSTRATE_HOME", self.substrate_home())
            .env("SUBSTRATE_WORLD_SOCKET", self.socket_path())
            .env("SUBSTRATE_WORLD", "enabled")
            .env("SUBSTRATE_WORLD_ENABLED", "1")
            .env("SUBSTRATE_OVERRIDE_POLICY_MODE", "enforce");
        cmd
    }
}

#[test]
fn cmd_denied_enforcement_is_host_only_even_when_world_enabled() {
    let fixture = CmdDeniedHostOnlyFixture::new();
    fixture.write_config();
    fixture.write_global_policy_patch_cmd_denied();

    let socket = AgentSocket::start(fixture.socket_path(), SocketResponse::Capabilities);

    let marker = "__cmd_denied_host_only__";
    let output = fixture
        .command()
        .arg("-c")
        .arg(format!("echo {marker}"))
        .output()
        .expect("run substrate command");

    assert_eq!(
        output.status.code(),
        Some(126),
        "expected cmd_denied enforcement to exit 126: {output:?}"
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        !stdout.contains(marker),
        "expected denied command to not execute, got stdout: {stdout}"
    );

    assert_eq!(
        socket.execute_request_count(),
        0,
        "expected cmd_denied enforcement to occur before any world-service execution request"
    );
}
