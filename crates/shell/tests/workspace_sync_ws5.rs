#![cfg(unix)]

mod support;

use chrono::{SecondsFormat, Utc};
use serde_json::json;
use std::fs;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use support::{substrate_shell_driver, temp_dir, AgentSocket, PendingDiffAckState, SocketResponse};
use tempfile::{Builder, TempDir};

struct WorkspaceSyncFixture {
    _temp: TempDir,
    home: PathBuf,
    substrate_home: PathBuf,
    workspace_root: PathBuf,
}

impl WorkspaceSyncFixture {
    fn new() -> Self {
        let temp = temp_dir("substrate-workspace-sync-ws5-");
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

fn assert_line_contains(haystack: &str, path: &str, needles: &[&str]) {
    let line = haystack
        .lines()
        .find(|line| line.contains(path))
        .unwrap_or_else(|| panic!("expected output to contain a line for {path}: {haystack}"));
    for needle in needles {
        assert!(
            line.to_ascii_lowercase()
                .contains(&needle.to_ascii_lowercase()),
            "expected line for {path} to contain {needle}: {line}"
        );
    }
}

#[test]
fn workspace_sync_apply_from_world_includes_pty_deletes() {
    let fixture = WorkspaceSyncFixture::new();
    fixture.init_workspace();
    fixture.write_workspace_yaml_patch(
        "sync:\n  auto_sync: false\n  direction: from_world\n  conflict_policy: prefer_host\n  exclude: []\n",
    );

    fs::write(fixture.workspace_root.join("nonpty_deleted.txt"), "bye\n")
        .expect("seed nonpty_deleted.txt");
    fs::write(fixture.workspace_root.join("pty_deleted.txt"), "bye\n")
        .expect("seed pty_deleted.txt");

    let socket_dir = Builder::new()
        .prefix("substrate-ws5-sock-")
        .tempdir_in("/tmp")
        .expect("create ws5 socket tempdir");
    let socket_path = socket_dir.path().join("world-service.sock");

    let state = PendingDiffAckState {
        features: vec![
            "execute".to_string(),
            "pending_diff_v1".to_string(),
            "pending_diff_clear_v1".to_string(),
            "world_fs_read_v1".to_string(),
        ],
        current_pending_diff: json!({
            "schema_version": 1,
            "session_started_at": "2100-01-01T00:00:00Z",
            "diff_id": "diff_ws5_from_world_pty_01",
            "non_pty": {
                "writes": [],
                "mods": [],
                "deletes": ["nonpty_deleted.txt"]
            },
            "pty": {
                "writes": [],
                "mods": [],
                "deletes": ["pty_deleted.txt"]
            }
        }),
        cleared_pending_diff: json!({
            "schema_version": 1,
            "session_started_at": "2100-01-01T00:00:00Z",
            "diff_id": "diff_ws5_empty",
            "non_pty": {
                "writes": [],
                "mods": [],
                "deletes": []
            }
        }),
        flip_after_first_pending_diff: None,
        ack_error: None,
        pending_diff_calls: 0,
        ack_calls: 0,
    };
    let _socket = AgentSocket::start(
        &socket_path,
        SocketResponse::CapabilitiesPendingDiffAndAck {
            state: Arc::new(Mutex::new(state)),
        },
    );

    let mut cmd = fixture.command();
    cmd.current_dir(&fixture.workspace_root)
        .env("SUBSTRATE_OVERRIDE_WORLD", "enabled")
        .env("SUBSTRATE_WORLD_SOCKET", &socket_path)
        .args(["workspace", "sync", "--direction", "from_world"]);
    let output = cmd.output().expect("run workspace sync apply (ws5 pty)");

    assert_eq!(
        output.status.code(),
        Some(0),
        "workspace sync apply must exit 0: {}",
        combined_output(&output)
    );
    assert!(
        !fixture.workspace_root.join("nonpty_deleted.txt").exists(),
        "sync apply must include non_pty deletes"
    );
    assert!(
        !fixture.workspace_root.join("pty_deleted.txt").exists(),
        "sync apply must include pty deletes"
    );
}

#[test]
fn workspace_sync_dry_run_from_host_reports_conflicts_and_per_path_decisions() {
    let fixture = WorkspaceSyncFixture::new();
    fixture.init_workspace();
    fixture.write_workspace_yaml_patch(
        "sync:\n  auto_sync: false\n  direction: from_host\n  conflict_policy: prefer_host\n  exclude: []\n",
    );

    fs::write(fixture.workspace_root.join("keep.txt"), "host\n").expect("seed keep.txt");
    std::thread::sleep(std::time::Duration::from_secs(2));
    let baseline = Utc::now().to_rfc3339_opts(SecondsFormat::Secs, true);
    fs::write(fixture.workspace_root.join("discard.txt"), "host\n").expect("seed discard.txt");

    let socket_dir = Builder::new()
        .prefix("substrate-ws5-sock-")
        .tempdir_in("/tmp")
        .expect("create ws5 socket tempdir");
    let socket_path = socket_dir.path().join("world-service.sock");

    let pending = json!({
        "schema_version": 1,
        "session_started_at": baseline,
        "diff_id": "diff_ws5_from_host_01",
        "non_pty": {
            "writes": [],
            "mods": [],
            "deletes": ["keep.txt", "discard.txt"]
        },
        "pty": {
            "writes": [],
            "mods": [],
            "deletes": ["pty_discard.txt"]
        }
    });
    let _socket = AgentSocket::start(
        &socket_path,
        SocketResponse::CapabilitiesAndPendingDiff {
            features: vec![
                "execute".to_string(),
                "pending_diff_v1".to_string(),
                "pending_diff_reconcile_v1".to_string(),
            ],
            pending_diff: pending,
        },
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
            "from_host",
            "--verbose",
        ]);
    let output = cmd
        .output()
        .expect("run workspace sync --dry-run --direction from_host (ws5)");

    assert_eq!(
        output.status.code(),
        Some(0),
        "workspace sync --dry-run --direction from_host must exit 0: {}",
        combined_output(&output)
    );

    let combined = combined_output(&output);
    assert!(
        combined.to_ascii_lowercase().contains("reconcil"),
        "dry-run must report a reconciliation plan: {combined}"
    );
    assert!(
        combined.to_ascii_lowercase().contains("conflict"),
        "dry-run must report whether conflicts exist: {combined}"
    );

    assert_line_contains(&combined, "keep.txt", &["keep"]);
    assert_line_contains(&combined, "discard.txt", &["discard"]);
    assert!(
        combined.contains("pty_discard.txt"),
        "verbose output must include PTY bucket paths: {combined}"
    );
}

#[test]
fn workspace_sync_from_host_mutates_world_but_not_host() {
    let fixture = WorkspaceSyncFixture::new();
    fixture.init_workspace();
    fixture.write_workspace_yaml_patch(
        "sync:\n  auto_sync: false\n  direction: from_host\n  conflict_policy: prefer_host\n  exclude: []\n",
    );

    fs::write(fixture.workspace_root.join("shadowed.txt"), "host\n").expect("seed shadowed.txt");
    let before = fs::read_to_string(fixture.workspace_root.join("shadowed.txt"))
        .expect("read shadowed.txt before");

    let socket_dir = Builder::new()
        .prefix("substrate-ws5-sock-")
        .tempdir_in("/tmp")
        .expect("create ws5 socket tempdir");
    let socket_path = socket_dir.path().join("world-service.sock");

    let state = PendingDiffAckState {
        features: vec![
            "execute".to_string(),
            "pending_diff_v1".to_string(),
            "pending_diff_reconcile_v1".to_string(),
            "pending_diff_clear_v1".to_string(),
            "world_fs_read_v1".to_string(),
        ],
        current_pending_diff: json!({
            "schema_version": 1,
            "session_started_at": "1970-01-01T00:00:00Z",
            "diff_id": "diff_ws5_from_host_apply_01",
            "non_pty": {
                "writes": [],
                "mods": [],
                "deletes": ["shadowed.txt"]
            }
        }),
        cleared_pending_diff: json!({
            "schema_version": 1,
            "session_started_at": "1970-01-01T00:00:00Z",
            "diff_id": "diff_ws5_empty",
            "non_pty": {
                "writes": [],
                "mods": [],
                "deletes": []
            }
        }),
        flip_after_first_pending_diff: None,
        ack_error: None,
        pending_diff_calls: 0,
        ack_calls: 0,
    };
    let socket = AgentSocket::start(
        &socket_path,
        SocketResponse::CapabilitiesExecutePendingDiffAndAck {
            stdout: "".to_string(),
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
        .args([
            "workspace",
            "sync",
            "--direction",
            "from_host",
            "--conflict-policy",
            "prefer_host",
            "--verbose",
        ]);
    let output = cmd
        .output()
        .expect("run workspace sync --direction from_host (ws5)");

    assert_eq!(
        output.status.code(),
        Some(0),
        "workspace sync --direction from_host must exit 0: {}",
        combined_output(&output)
    );

    let after = fs::read_to_string(fixture.workspace_root.join("shadowed.txt"))
        .expect("read shadowed.txt after");
    assert_eq!(
        before, after,
        "direction=from_host must not mutate the host workspace"
    );

    assert!(
        socket.reconcile_request_count() > 0,
        "direction=from_host must reconcile pending diff state in world (expected at least one reconcile request)"
    );
    assert!(
        socket
            .last_reconcile_discard_paths()
            .contains(&"shadowed.txt".to_string()),
        "direction=from_host must request discard for the conflicting path"
    );
}

#[test]
fn workspace_sync_direction_both_applies_non_pty_and_pty_deletes() {
    let fixture = WorkspaceSyncFixture::new();
    fixture.init_workspace();
    fixture.write_workspace_yaml_patch(
        "sync:\n  auto_sync: false\n  direction: both\n  conflict_policy: prefer_world\n  exclude: []\n",
    );

    fs::write(
        fixture.workspace_root.join("both_nonpty_deleted.txt"),
        "bye\n",
    )
    .expect("seed both_nonpty_deleted.txt");
    fs::write(fixture.workspace_root.join("both_pty_deleted.txt"), "bye\n")
        .expect("seed both_pty_deleted.txt");

    let socket_dir = Builder::new()
        .prefix("substrate-ws5-sock-")
        .tempdir_in("/tmp")
        .expect("create ws5 socket tempdir");
    let socket_path = socket_dir.path().join("world-service.sock");

    let state = PendingDiffAckState {
        features: vec![
            "execute".to_string(),
            "pending_diff_v1".to_string(),
            "pending_diff_reconcile_v1".to_string(),
            "pending_diff_clear_v1".to_string(),
            "world_fs_read_v1".to_string(),
        ],
        current_pending_diff: json!({
            "schema_version": 1,
            "session_started_at": "2100-01-01T00:00:00Z",
            "diff_id": "diff_ws5_both_01",
            "non_pty": {
                "writes": [],
                "mods": [],
                "deletes": ["both_nonpty_deleted.txt"]
            },
            "pty": {
                "writes": [],
                "mods": [],
                "deletes": ["both_pty_deleted.txt"]
            }
        }),
        cleared_pending_diff: json!({
            "schema_version": 1,
            "session_started_at": "2100-01-01T00:00:00Z",
            "diff_id": "diff_ws5_empty",
            "non_pty": {
                "writes": [],
                "mods": [],
                "deletes": []
            }
        }),
        flip_after_first_pending_diff: None,
        ack_error: None,
        pending_diff_calls: 0,
        ack_calls: 0,
    };
    let _socket = AgentSocket::start(
        &socket_path,
        SocketResponse::CapabilitiesExecutePendingDiffAndAck {
            stdout: "".to_string(),
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
        .args(["workspace", "sync", "--direction", "both"]);
    let output = cmd
        .output()
        .expect("run workspace sync --direction both (ws5)");

    assert_eq!(
        output.status.code(),
        Some(0),
        "workspace sync --direction both must exit 0: {}",
        combined_output(&output)
    );
    assert!(
        !fixture
            .workspace_root
            .join("both_nonpty_deleted.txt")
            .exists(),
        "direction=both must apply non_pty deletes"
    );
    assert!(
        !fixture.workspace_root.join("both_pty_deleted.txt").exists(),
        "direction=both must apply pty deletes"
    );
}
