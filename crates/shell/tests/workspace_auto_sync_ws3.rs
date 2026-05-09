#![cfg(unix)]

mod support;

use serde_json::json;
use std::fs;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use support::{substrate_shell_driver, AgentSocket, PendingDiffAckState, SocketResponse};
use tempfile::{Builder, TempDir};

struct WorkspaceAutoSyncFixture {
    _temp: TempDir,
    home: PathBuf,
    substrate_home: PathBuf,
    workspace_root: PathBuf,
}

impl WorkspaceAutoSyncFixture {
    fn new() -> Self {
        let temp = Builder::new()
            .prefix("substrate-workspace-auto-sync-ws3-")
            .tempdir_in("/tmp")
            .expect("failed to allocate ws3 temp dir");
        let home = temp.path().join("home");
        fs::create_dir_all(&home).expect("failed to create HOME fixture");
        let substrate_home = temp.path().join("substrate-home");
        fs::create_dir_all(&substrate_home).expect("failed to create SUBSTRATE_HOME fixture");
        let workspace_root = temp.path().join("workspace");
        fs::create_dir_all(&workspace_root).expect("failed to create workspace root");
        Self {
            _temp: temp,
            home,
            substrate_home,
            workspace_root,
        }
    }

    fn command(&self) -> assert_cmd::Command {
        let mut cmd = substrate_shell_driver();
        cmd.env("HOME", &self.home)
            .env("USERPROFILE", &self.home)
            .env("SUBSTRATE_HOME", &self.substrate_home)
            .env_remove("SUBSTRATE_WORLD")
            .env_remove("SUBSTRATE_WORLD_ENABLED")
            .env_remove("SUBSTRATE_WORLD_ID");
        cmd
    }

    fn init_workspace(&self) {
        let out = self
            .command()
            .arg("workspace")
            .arg("init")
            .arg(&self.workspace_root)
            .output()
            .expect("failed to run workspace init");
        assert!(out.status.success(), "workspace init must succeed: {out:?}");
    }

    fn workspace_yaml_path(&self) -> PathBuf {
        self.workspace_root
            .join(".substrate")
            .join("workspace.yaml")
    }

    fn write_workspace_yaml_patch(&self, body: &str) {
        fs::write(self.workspace_yaml_path(), body).expect("write workspace.yaml patch");
    }
}

fn combined_output(output: &std::process::Output) -> String {
    let mut out = String::new();
    out.push_str(&String::from_utf8_lossy(&output.stdout));
    if !out.ends_with('\n') {
        out.push('\n');
    }
    out.push_str(&String::from_utf8_lossy(&output.stderr));
    out
}

fn pending_diff_record_deletes(
    session_started_at: &str,
    diff_id: &str,
    deletes: Vec<String>,
) -> serde_json::Value {
    json!({
        "session_started_at": session_started_at,
        "diff_id": diff_id,
        "non_pty": {
            "writes": [],
            "mods": [],
            "deletes": deletes
        }
    })
}

fn empty_pending_diff_record(session_started_at: &str, diff_id: &str) -> serde_json::Value {
    pending_diff_record_deletes(session_started_at, diff_id, Vec::new())
}

fn ws3_socket_state(current_pending_diff: serde_json::Value) -> PendingDiffAckState {
    PendingDiffAckState {
        features: vec![
            "execute".to_string(),
            "pending_diff_v1".to_string(),
            "pending_diff_clear_v1".to_string(),
            "world_fs_read_v1".to_string(),
        ],
        current_pending_diff,
        cleared_pending_diff: empty_pending_diff_record("2100-01-01T00:00:00Z", "diff_ws3_empty"),
        flip_after_first_pending_diff: None,
        ack_error: None,
        pending_diff_calls: 0,
        ack_calls: 0,
    }
}

#[test]
fn auto_sync_runs_after_successful_non_pty_world_command_and_applies_pending_diff() {
    let fixture = WorkspaceAutoSyncFixture::new();
    fixture.init_workspace();
    fixture.write_workspace_yaml_patch(
        "sync:\n  auto_sync: true\n  direction: from_world\n  conflict_policy: prefer_host\n  exclude: []\n",
    );

    fs::write(fixture.workspace_root.join("deleted.txt"), "goodbye\n").expect("seed deleted.txt");

    let socket_dir = Builder::new()
        .prefix("substrate-ws3-sock-")
        .tempdir_in("/tmp")
        .expect("create ws3 socket tempdir");
    let socket_path = socket_dir.path().join("world-agent.sock");
    let state = ws3_socket_state(pending_diff_record_deletes(
        "2100-01-01T00:00:00Z",
        "diff_ws3_01",
        vec!["deleted.txt".to_string()],
    ));
    let _socket = AgentSocket::start(
        &socket_path,
        SocketResponse::CapabilitiesExecutePendingDiffAndAck {
            stdout: "ok\n".to_string(),
            stderr: "".to_string(),
            exit: 0,
            scopes: vec![],
            state: Arc::new(Mutex::new(state)),
        },
    );

    let mut cmd = fixture.command();
    cmd.current_dir(&fixture.workspace_root)
        .env("SUBSTRATE_OVERRIDE_WORLD", "enabled")
        .env("SUBSTRATE_WORLD_SOCKET", &socket_path)
        .args(["--world", "-c", "echo ws3"]);
    let output = cmd.output().expect("run substrate -c");

    assert_eq!(
        output.status.code(),
        Some(0),
        "successful command must exit 0: {}",
        combined_output(&output)
    );
    assert!(
        _socket.execute_request_count() > 0,
        "command must execute in the world (expected /v1/execute requests)"
    );
    assert!(
        !fixture.workspace_root.join("deleted.txt").exists(),
        "auto-sync must apply the pending diff (deleted.txt removed)"
    );

    let mut cmd = fixture.command();
    cmd.current_dir(&fixture.workspace_root)
        .env("SUBSTRATE_OVERRIDE_WORLD", "enabled")
        .env("SUBSTRATE_WORLD_SOCKET", &socket_path)
        .args([
            "workspace",
            "sync",
            "--dry-run",
            "--direction",
            "from_world",
        ]);
    let output = cmd
        .output()
        .expect("run workspace sync --dry-run after auto-sync");
    assert_eq!(
        output.status.code(),
        Some(0),
        "workspace sync --dry-run must succeed: {}",
        combined_output(&output)
    );
    let combined = combined_output(&output);
    assert!(
        combined.contains("total_paths: 0"),
        "auto-sync must clear the pending diff: {combined}"
    );
}

#[test]
fn auto_sync_does_not_run_when_non_pty_world_command_exits_nonzero() {
    let fixture = WorkspaceAutoSyncFixture::new();
    fixture.init_workspace();
    fixture.write_workspace_yaml_patch(
        "sync:\n  auto_sync: true\n  direction: from_world\n  conflict_policy: prefer_host\n  exclude: []\n",
    );

    fs::write(fixture.workspace_root.join("deleted.txt"), "goodbye\n").expect("seed deleted.txt");

    let socket_dir = Builder::new()
        .prefix("substrate-ws3-sock-")
        .tempdir_in("/tmp")
        .expect("create ws3 socket tempdir");
    let socket_path = socket_dir.path().join("world-agent.sock");
    let state = ws3_socket_state(pending_diff_record_deletes(
        "2100-01-01T00:00:00Z",
        "diff_ws3_nonzero",
        vec!["deleted.txt".to_string()],
    ));
    let _socket = AgentSocket::start(
        &socket_path,
        SocketResponse::CapabilitiesExecutePendingDiffAndAck {
            stdout: "".to_string(),
            stderr: "boom\n".to_string(),
            exit: 7,
            scopes: vec![],
            state: Arc::new(Mutex::new(state)),
        },
    );

    let mut cmd = fixture.command();
    cmd.current_dir(&fixture.workspace_root)
        .env("SUBSTRATE_OVERRIDE_WORLD", "enabled")
        .env("SUBSTRATE_WORLD_SOCKET", &socket_path)
        .args(["--world", "-c", "echo ws3"]);
    let output = cmd.output().expect("run substrate -c failing");

    assert_eq!(
        output.status.code(),
        Some(7),
        "non-zero command exit must propagate: {}",
        combined_output(&output)
    );
    assert!(
        fixture.workspace_root.join("deleted.txt").exists(),
        "auto-sync must not run on non-zero command exit"
    );

    let mut cmd = fixture.command();
    cmd.current_dir(&fixture.workspace_root)
        .env("SUBSTRATE_OVERRIDE_WORLD", "enabled")
        .env("SUBSTRATE_WORLD_SOCKET", &socket_path)
        .args([
            "workspace",
            "sync",
            "--dry-run",
            "--direction",
            "from_world",
        ]);
    let output = cmd
        .output()
        .expect("run workspace sync --dry-run after non-zero command");
    assert_eq!(
        output.status.code(),
        Some(0),
        "workspace sync --dry-run must succeed: {}",
        combined_output(&output)
    );
    let combined = combined_output(&output);
    assert!(
        combined.contains("total_paths: 1") && combined.contains("deletes: 1"),
        "pending diff must remain when auto-sync does not run: {combined}"
    );
}

#[test]
fn auto_sync_failure_propagates_exit_code_and_prefixes_error() {
    let fixture = WorkspaceAutoSyncFixture::new();
    fixture.init_workspace();
    fixture.write_workspace_yaml_patch(
        "sync:\n  auto_sync: true\n  direction: from_world\n  conflict_policy: prefer_host\n  exclude: []\n",
    );

    fs::write(fixture.workspace_root.join("kept.txt"), "stay\n").expect("seed kept.txt");

    let socket_dir = Builder::new()
        .prefix("substrate-ws3-sock-")
        .tempdir_in("/tmp")
        .expect("create ws3 socket tempdir");
    let socket_path = socket_dir.path().join("world-agent.sock");
    let state = ws3_socket_state(json!({
        "session_started_at": "2100-01-01T00:00:00Z",
        "diff_id": "diff_ws3_protected",
        "non_pty": {
            "writes": [".git/config"],
            "mods": [],
            "deletes": ["kept.txt"]
        }
    }));
    let _socket = AgentSocket::start(
        &socket_path,
        SocketResponse::CapabilitiesExecutePendingDiffAndAck {
            stdout: "ok\n".to_string(),
            stderr: "".to_string(),
            exit: 0,
            scopes: vec![],
            state: Arc::new(Mutex::new(state)),
        },
    );

    let mut cmd = fixture.command();
    cmd.current_dir(&fixture.workspace_root)
        .env("SUBSTRATE_OVERRIDE_WORLD", "enabled")
        .env("SUBSTRATE_WORLD_SOCKET", &socket_path)
        .args(["--world", "-c", "echo ws3"]);
    let output = cmd.output().expect("run substrate -c with protected diff");

    assert_eq!(
        output.status.code(),
        Some(5),
        "auto-sync failure must propagate exit code: {}",
        combined_output(&output)
    );
    let combined = combined_output(&output);
    assert!(
        combined.contains("auto-sync failed:"),
        "output must include required prefix line: {combined}"
    );
    assert!(
        combined.contains("protected paths"),
        "output must include the underlying failure reason: {combined}"
    );
    assert!(
        fixture.workspace_root.join("kept.txt").exists(),
        "auto-sync failure must not mutate the workspace"
    );
}

#[test]
fn auto_sync_noops_when_effective_direction_is_from_host() {
    let fixture = WorkspaceAutoSyncFixture::new();
    fixture.init_workspace();
    fixture.write_workspace_yaml_patch(
        "sync:\n  auto_sync: true\n  direction: from_host\n  conflict_policy: prefer_host\n  exclude: []\n",
    );

    fs::write(fixture.workspace_root.join("deleted.txt"), "goodbye\n").expect("seed deleted.txt");

    let socket_dir = Builder::new()
        .prefix("substrate-ws3-sock-")
        .tempdir_in("/tmp")
        .expect("create ws3 socket tempdir");
    let socket_path = socket_dir.path().join("world-agent.sock");
    let state = ws3_socket_state(pending_diff_record_deletes(
        "2100-01-01T00:00:00Z",
        "diff_ws3_from_host",
        vec!["deleted.txt".to_string()],
    ));
    let _socket = AgentSocket::start(
        &socket_path,
        SocketResponse::CapabilitiesExecutePendingDiffAndAck {
            stdout: "ok\n".to_string(),
            stderr: "".to_string(),
            exit: 0,
            scopes: vec![],
            state: Arc::new(Mutex::new(state)),
        },
    );

    let mut cmd = fixture.command();
    cmd.current_dir(&fixture.workspace_root)
        .env("SUBSTRATE_OVERRIDE_WORLD", "enabled")
        .env("SUBSTRATE_WORLD_SOCKET", &socket_path)
        .args(["--world", "-c", "echo ws3"]);
    let output = cmd.output().expect("run substrate -c direction from_host");

    assert_eq!(
        output.status.code(),
        Some(0),
        "command exit code must be preserved: {}",
        combined_output(&output)
    );
    assert!(
        fixture.workspace_root.join("deleted.txt").exists(),
        "auto-sync must no-op when effective direction is from_host"
    );
}
