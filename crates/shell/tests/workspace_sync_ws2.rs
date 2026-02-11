#![cfg(unix)]

mod support;

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
        let temp = temp_dir("substrate-workspace-sync-ws2-");
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

fn pending_diff_record(
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
    pending_diff_record(session_started_at, diff_id, Vec::new())
}

#[test]
fn workspace_sync_apply_from_world_applies_deletes_respects_excludes_and_clears_pending_diff() {
    let fixture = WorkspaceSyncFixture::new();
    fixture.init_workspace();
    fixture.write_workspace_yaml_patch(
        "sync:\n  auto_sync: false\n  direction: from_world\n  conflict_policy: prefer_host\n  exclude: []\n",
    );

    fs::write(fixture.workspace_root.join("deleted.txt"), "goodbye\n").expect("seed deleted.txt");
    fs::create_dir_all(fixture.workspace_root.join("excluded")).expect("seed excluded dir");
    fs::write(fixture.workspace_root.join("excluded/skip.txt"), "keep\n")
        .expect("seed excluded/skip.txt");

    let socket_dir = Builder::new()
        .prefix("substrate-ws2-sock-")
        .tempdir_in("/tmp")
        .expect("create ws2 socket tempdir");
    let socket_path = socket_dir.path().join("world-agent.sock");

    let state = PendingDiffAckState {
        features: vec![
            "execute".to_string(),
            "pending_diff_v1".to_string(),
            "pending_diff_clear_v1".to_string(),
            "world_fs_read_v1".to_string(),
        ],
        current_pending_diff: pending_diff_record(
            "2100-01-01T00:00:00Z",
            "diff_ws2_01",
            vec!["deleted.txt".to_string(), "excluded/skip.txt".to_string()],
        ),
        cleared_pending_diff: empty_pending_diff_record("2100-01-01T00:00:00Z", "diff_ws2_empty"),
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
        .args([
            "workspace",
            "sync",
            "--direction",
            "from_world",
            "--exclude",
            "excluded/**",
        ]);
    let output = cmd.output().expect("run workspace sync apply");

    assert_eq!(
        output.status.code(),
        Some(0),
        "workspace sync apply must exit 0: {}",
        combined_output(&output)
    );
    assert!(
        !fixture.workspace_root.join("deleted.txt").exists(),
        "sync apply must delete deleted.txt"
    );
    assert!(
        fixture.workspace_root.join("excluded/skip.txt").exists(),
        "sync apply must respect excludes (excluded/skip.txt must remain)"
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
            "--verbose",
        ]);
    let output = cmd
        .output()
        .expect("run workspace sync --dry-run after apply");
    assert_eq!(
        output.status.code(),
        Some(0),
        "workspace sync --dry-run after apply must exit 0: {}",
        combined_output(&output)
    );
    let combined = combined_output(&output);
    assert!(
        combined.contains("total_paths: 0"),
        "cleared pending diff must produce an empty summary: {combined}"
    );
}

#[test]
fn workspace_sync_apply_refuses_when_pending_diff_contains_protected_paths_and_does_not_mutate() {
    let fixture = WorkspaceSyncFixture::new();
    fixture.init_workspace();
    fixture.write_workspace_yaml_patch(
        "sync:\n  auto_sync: false\n  direction: from_world\n  conflict_policy: prefer_host\n  exclude: []\n",
    );

    fs::write(fixture.workspace_root.join("kept.txt"), "stay\n").expect("seed kept.txt");

    let socket_dir = Builder::new()
        .prefix("substrate-ws2-sock-")
        .tempdir_in("/tmp")
        .expect("create ws2 socket tempdir");
    let socket_path = socket_dir.path().join("world-agent.sock");

    let state = PendingDiffAckState {
        features: vec!["execute".to_string(), "pending_diff_v1".to_string()],
        current_pending_diff: json!({
            "session_started_at": "2100-01-01T00:00:00Z",
            "diff_id": "diff_protected",
            "non_pty": {
                "writes": [".git/config"],
                "mods": [],
                "deletes": ["kept.txt"]
            }
        }),
        cleared_pending_diff: empty_pending_diff_record("2100-01-01T00:00:00Z", "diff_empty"),
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
    let output = cmd
        .output()
        .expect("run workspace sync apply with protected path");

    assert_eq!(
        output.status.code(),
        Some(5),
        "protected path refusal must exit 5: {}",
        combined_output(&output)
    );
    assert!(
        fixture.workspace_root.join("kept.txt").exists(),
        "protected path refusal must not mutate the workspace"
    );
}

#[test]
fn workspace_sync_apply_refuses_when_max_paths_guard_exceeded_and_includes_threshold_and_observed_values(
) {
    let fixture = WorkspaceSyncFixture::new();
    fixture.init_workspace();
    fixture.write_workspace_yaml_patch(
        "sync:\n  auto_sync: false\n  direction: from_world\n  conflict_policy: prefer_host\n  exclude: []\n",
    );

    let socket_dir = Builder::new()
        .prefix("substrate-ws2-sock-")
        .tempdir_in("/tmp")
        .expect("create ws2 socket tempdir");
    let socket_path = socket_dir.path().join("world-agent.sock");

    let deletes = (0..10001)
        .map(|idx| format!("too_many_{idx:05}.txt"))
        .collect::<Vec<_>>();
    let state = PendingDiffAckState {
        features: vec!["execute".to_string(), "pending_diff_v1".to_string()],
        current_pending_diff: pending_diff_record("2100-01-01T00:00:00Z", "diff_big", deletes),
        cleared_pending_diff: empty_pending_diff_record("2100-01-01T00:00:00Z", "diff_empty"),
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
    let output = cmd
        .output()
        .expect("run workspace sync apply with max paths exceeded");

    assert_eq!(
        output.status.code(),
        Some(5),
        "max paths guard must exit 5: {}",
        combined_output(&output)
    );
    let combined = combined_output(&output);
    assert!(
        combined.contains("10000") && combined.contains("10001"),
        "size guard refusal must include both threshold and observed values: {combined}"
    );
}

#[test]
fn workspace_sync_apply_conflict_policy_abort_refuses_and_does_not_clear_pending_diff() {
    let fixture = WorkspaceSyncFixture::new();
    fixture.init_workspace();
    fixture.write_workspace_yaml_patch(
        "sync:\n  auto_sync: false\n  direction: from_world\n  conflict_policy: abort\n  exclude: []\n",
    );

    fs::write(fixture.workspace_root.join("conflict.txt"), "host\n").expect("seed conflict.txt");

    let socket_dir = Builder::new()
        .prefix("substrate-ws2-sock-")
        .tempdir_in("/tmp")
        .expect("create ws2 socket tempdir");
    let socket_path = socket_dir.path().join("world-agent.sock");

    let state = PendingDiffAckState {
        features: vec![
            "execute".to_string(),
            "pending_diff_v1".to_string(),
            "pending_diff_clear_v1".to_string(),
            "world_fs_read_v1".to_string(),
        ],
        current_pending_diff: pending_diff_record(
            "1970-01-01T00:00:00Z",
            "diff_conflict_abort",
            vec!["conflict.txt".to_string()],
        ),
        cleared_pending_diff: empty_pending_diff_record("1970-01-01T00:00:00Z", "diff_empty"),
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
        .args([
            "workspace",
            "sync",
            "--direction",
            "from_world",
            "--conflict-policy",
            "abort",
        ]);
    let output = cmd.output().expect("run workspace sync apply (abort)");

    assert_eq!(
        output.status.code(),
        Some(5),
        "abort-on-conflict must exit 5: {}",
        combined_output(&output)
    );
    assert!(
        fixture.workspace_root.join("conflict.txt").exists(),
        "abort-on-conflict must not mutate the workspace"
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
        .expect("run workspace sync --dry-run after abort refusal");
    assert_eq!(
        output.status.code(),
        Some(0),
        "dry-run after abort refusal must exit 0: {}",
        combined_output(&output)
    );
    let combined = combined_output(&output);
    assert!(
        combined.contains("total_paths: 1"),
        "abort refusal must not clear pending diffs: {combined}"
    );
}

#[test]
fn workspace_sync_apply_conflict_policy_prefer_host_skips_conflicts_exits_0_and_clears_pending_diff(
) {
    let fixture = WorkspaceSyncFixture::new();
    fixture.init_workspace();
    fixture.write_workspace_yaml_patch(
        "sync:\n  auto_sync: false\n  direction: from_world\n  conflict_policy: prefer_host\n  exclude: []\n",
    );

    fs::write(fixture.workspace_root.join("conflict.txt"), "host\n").expect("seed conflict.txt");

    let socket_dir = Builder::new()
        .prefix("substrate-ws2-sock-")
        .tempdir_in("/tmp")
        .expect("create ws2 socket tempdir");
    let socket_path = socket_dir.path().join("world-agent.sock");

    let state = PendingDiffAckState {
        features: vec![
            "execute".to_string(),
            "pending_diff_v1".to_string(),
            "pending_diff_clear_v1".to_string(),
            "world_fs_read_v1".to_string(),
        ],
        current_pending_diff: pending_diff_record(
            "1970-01-01T00:00:00Z",
            "diff_conflict_prefer_host",
            vec!["conflict.txt".to_string()],
        ),
        cleared_pending_diff: empty_pending_diff_record("1970-01-01T00:00:00Z", "diff_empty"),
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
        .args([
            "workspace",
            "sync",
            "--direction",
            "from_world",
            "--conflict-policy",
            "prefer_host",
        ]);
    let output = cmd
        .output()
        .expect("run workspace sync apply (prefer_host)");

    assert_eq!(
        output.status.code(),
        Some(0),
        "prefer_host must exit 0: {}",
        combined_output(&output)
    );
    assert!(
        fixture.workspace_root.join("conflict.txt").exists(),
        "prefer_host must not overwrite conflicting host paths"
    );

    let combined = combined_output(&output);
    assert!(
        combined.to_ascii_lowercase().contains("skipped")
            && combined.to_ascii_lowercase().contains("conflict")
            && combined.contains('1'),
        "prefer_host summary must include skipped-by-conflict count: {combined}"
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
        .expect("run workspace sync --dry-run after prefer_host");
    assert_eq!(
        output.status.code(),
        Some(0),
        "dry-run after prefer_host apply must exit 0: {}",
        combined_output(&output)
    );
    let combined = combined_output(&output);
    assert!(
        combined.contains("total_paths: 0"),
        "prefer_host apply must clear pending diffs: {combined}"
    );
}

#[test]
fn workspace_sync_apply_exits_1_when_applied_but_pending_diffs_not_cleared_and_does_not_clear_whatever_is_current(
) {
    let fixture = WorkspaceSyncFixture::new();
    fixture.init_workspace();
    fixture.write_workspace_yaml_patch(
        "sync:\n  auto_sync: false\n  direction: from_world\n  conflict_policy: prefer_host\n  exclude: []\n",
    );

    fs::write(fixture.workspace_root.join("deleted.txt"), "bye\n").expect("seed deleted.txt");

    let socket_dir = Builder::new()
        .prefix("substrate-ws2-sock-")
        .tempdir_in("/tmp")
        .expect("create ws2 socket tempdir");
    let socket_path = socket_dir.path().join("world-agent.sock");

    let initial = pending_diff_record(
        "2100-01-01T00:00:00Z",
        "diff_old",
        vec!["deleted.txt".to_string()],
    );
    let new_current = pending_diff_record(
        "2100-01-01T00:00:00Z",
        "diff_new",
        vec!["newer.txt".to_string()],
    );

    let state = PendingDiffAckState {
        features: vec![
            "execute".to_string(),
            "pending_diff_v1".to_string(),
            "pending_diff_clear_v1".to_string(),
            "world_fs_read_v1".to_string(),
        ],
        current_pending_diff: initial,
        cleared_pending_diff: empty_pending_diff_record("2100-01-01T00:00:00Z", "diff_empty"),
        flip_after_first_pending_diff: Some(new_current),
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
    let output = cmd
        .output()
        .expect("run workspace sync apply with ack mismatch");

    assert_eq!(
        output.status.code(),
        Some(1),
        "clear/ack failure after apply must exit 1: {}",
        combined_output(&output)
    );
    assert!(
        !fixture.workspace_root.join("deleted.txt").exists(),
        "clear/ack failure happens after apply; applied mutations must be present"
    );
    let combined = combined_output(&output).to_ascii_lowercase();
    assert!(
        combined.contains("applied but pending diffs were not cleared"),
        "clear/ack failure message must be actionable: {combined}"
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
            "--verbose",
        ]);
    let output = cmd
        .output()
        .expect("run workspace sync --dry-run after clear failure");
    assert_eq!(
        output.status.code(),
        Some(0),
        "dry-run after clear failure must still succeed: {}",
        combined_output(&output)
    );
    let combined = combined_output(&output);
    assert!(
        combined.contains("diff_new"),
        "Substrate must not clear whatever is current after diff_id mismatch: {combined}"
    );
}
