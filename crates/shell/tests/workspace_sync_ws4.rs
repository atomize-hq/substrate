#![cfg(unix)]

mod support;

use serde_json::json;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use support::{substrate_shell_driver, temp_dir, AgentSocket, SocketResponse};
use tempfile::{Builder, TempDir};

struct WorkspaceSyncFixture {
    _temp: TempDir,
    home: PathBuf,
    substrate_home: PathBuf,
    workspace_root: PathBuf,
}

impl WorkspaceSyncFixture {
    fn new() -> Self {
        let temp = temp_dir("substrate-workspace-sync-ws4-");
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
        assert!(
            self.workspace_yaml_path().is_file(),
            "workspace init must create workspace.yaml"
        );
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

fn parse_pending_diff_summaries(combined: &str) -> HashMap<String, HashMap<String, String>> {
    let mut sections: HashMap<String, HashMap<String, String>> = HashMap::new();
    let mut current: Option<String> = None;

    for line in combined.lines() {
        let trimmed = line.trim_end();
        if let Some(rest) = trimmed.strip_prefix("substrate: pending diff summary (") {
            if let Some(section) = rest.strip_suffix(')') {
                current = Some(section.to_string());
                sections.entry(section.to_string()).or_default();
                continue;
            }
        }

        let Some(section) = current.clone() else {
            continue;
        };

        let Some(kv) = trimmed.strip_prefix("  ") else {
            continue;
        };
        let Some((key, value)) = kv.split_once(':') else {
            continue;
        };
        sections
            .entry(section)
            .or_default()
            .insert(key.trim().to_string(), value.trim().to_string());
    }

    sections
}

fn parse_leading_usize(value: &str) -> Option<usize> {
    let digits = value
        .chars()
        .take_while(|c| c.is_ascii_digit())
        .collect::<String>();
    if digits.is_empty() {
        None
    } else {
        digits.parse().ok()
    }
}

#[test]
fn workspace_sync_dry_run_reports_pty_and_non_pty_pending_diff_summaries_when_supported() {
    let fixture = WorkspaceSyncFixture::new();
    fixture.init_workspace();
    fixture.write_workspace_yaml_patch(
        "sync:\n  auto_sync: false\n  direction: from_world\n  conflict_policy: prefer_host\n  exclude: []\n",
    );

    let socket_dir = Builder::new()
        .prefix("substrate-ws4-sock-")
        .tempdir_in("/tmp")
        .expect("create ws4 socket tempdir");
    let socket_path = socket_dir.path().join("world-agent.sock");
    let pending = json!({
        "schema_version": 1,
        "session_started_at": "2026-02-10T18:38:23Z",
        "diff_id": "diff_test_ws4_pty_01",
        "non_pty": {
            "writes": ["added.txt"],
            "mods": [],
            "deletes": []
        },
        "pty": {
            "writes": ["pty_added.txt"],
            "mods": ["pty_changed.txt"],
            "deletes": []
        }
    });
    let _socket = AgentSocket::start(
        &socket_path,
        SocketResponse::CapabilitiesAndPendingDiff {
            features: vec!["execute".to_string(), "pending_diff_v1".to_string()],
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
            "from_world",
        ]);
    let output = cmd.output().expect("run workspace sync --dry-run (ws4)");

    assert_eq!(
        output.status.code(),
        Some(0),
        "workspace sync --dry-run must exit 0: {}",
        combined_output(&output)
    );

    let combined = combined_output(&output);
    let summaries = parse_pending_diff_summaries(&combined);

    assert!(
        summaries.contains_key("non_pty"),
        "dry-run must include non_pty summary section: {combined}"
    );
    assert!(
        summaries.contains_key("pty"),
        "dry-run must include pty summary section when supported: {combined}"
    );
    assert!(
        summaries.contains_key("combined"),
        "dry-run must include combined summary section when pty is supported: {combined}"
    );

    let non_pty = &summaries["non_pty"];
    assert_eq!(
        parse_leading_usize(non_pty.get("total_paths").map(String::as_str).unwrap_or("")),
        Some(1),
        "non_pty total_paths must reflect the non_pty bucket: {combined}"
    );

    let pty = &summaries["pty"];
    assert_eq!(
        parse_leading_usize(pty.get("total_paths").map(String::as_str).unwrap_or("")),
        Some(2),
        "pty total_paths must reflect the pty bucket: {combined}"
    );

    let combined_section = &summaries["combined"];
    assert_eq!(
        parse_leading_usize(
            combined_section
                .get("total_paths")
                .map(String::as_str)
                .unwrap_or("")
        ),
        Some(3),
        "combined total_paths must include both buckets: {combined}"
    );
}

#[test]
fn workspace_sync_dry_run_reports_pty_pending_diffs_unsupported_explicitly() {
    let fixture = WorkspaceSyncFixture::new();
    fixture.init_workspace();
    fixture.write_workspace_yaml_patch(
        "sync:\n  auto_sync: false\n  direction: from_world\n  conflict_policy: prefer_host\n  exclude: []\n",
    );

    let socket_dir = Builder::new()
        .prefix("substrate-ws4-sock-")
        .tempdir_in("/tmp")
        .expect("create ws4 socket tempdir");
    let socket_path = socket_dir.path().join("world-agent.sock");
    let pending = json!({
        "schema_version": 1,
        "session_started_at": "2026-02-10T18:38:23Z",
        "diff_id": "diff_test_ws4_no_pty_01",
        "non_pty": {
            "writes": ["added.txt", "changed.txt"],
            "mods": [],
            "deletes": []
        }
    });
    let _socket = AgentSocket::start(
        &socket_path,
        SocketResponse::CapabilitiesAndPendingDiff {
            features: vec!["execute".to_string(), "pending_diff_v1".to_string()],
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
            "from_world",
        ]);
    let output = cmd.output().expect("run workspace sync --dry-run (ws4)");

    assert_eq!(
        output.status.code(),
        Some(0),
        "workspace sync --dry-run must exit 0 when PTY diffs are unsupported: {}",
        combined_output(&output)
    );

    let combined = combined_output(&output);
    assert!(
        combined.contains("PTY pending diffs unsupported"),
        "dry-run must print an explicit PTY unsupported line: {combined}"
    );

    let summaries = parse_pending_diff_summaries(&combined);
    assert!(
        summaries.contains_key("non_pty"),
        "dry-run must still include the non_pty summary section: {combined}"
    );
    let non_pty = &summaries["non_pty"];
    assert_eq!(
        parse_leading_usize(non_pty.get("total_paths").map(String::as_str).unwrap_or("")),
        Some(2),
        "non_pty total_paths must still be reported: {combined}"
    );
}
