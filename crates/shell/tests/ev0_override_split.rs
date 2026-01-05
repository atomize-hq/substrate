#![cfg(unix)]

mod common;

use assert_cmd::Command;
use common::{substrate_shell_driver, temp_dir};
use serde_json::Value as JsonValue;
use std::fs;
use std::path::{Path, PathBuf};
use tempfile::TempDir;

struct OverrideSplitFixture {
    _temp: TempDir,
    home: PathBuf,
    substrate_home: PathBuf,
    project: PathBuf,
    trace: PathBuf,
}

impl OverrideSplitFixture {
    fn new(prefix: &str) -> Self {
        let temp = temp_dir(prefix);
        let home = temp.path().join("home");
        fs::create_dir_all(&home).expect("create HOME");
        let substrate_home = temp.path().join("substrate-home");
        fs::create_dir_all(&substrate_home).expect("create SUBSTRATE_HOME");
        let project = temp.path().join("project");
        fs::create_dir_all(&project).expect("create project dir");
        let trace = temp.path().join("trace.jsonl");
        fs::write(&trace, "").expect("seed trace.jsonl");
        Self {
            _temp: temp,
            home,
            substrate_home,
            project,
            trace,
        }
    }

    fn write_global_config(
        &self,
        policy_mode: &str,
        anchor_mode: &str,
        anchor_path: &str,
        caged: bool,
    ) {
        let caged = if caged { "true" } else { "false" };
        let contents = format!(
            "world:\n  enabled: false\n  anchor_mode: {anchor_mode}\n  anchor_path: \"{anchor_path}\"\n  caged: {caged}\n\npolicy:\n  mode: {policy_mode}\n\nsync:\n  auto_sync: false\n  direction: from_world\n  conflict_policy: prefer_host\n  exclude: []\n"
        );
        fs::write(self.substrate_home.join("config.yaml"), contents).expect("write config.yaml");
    }

    fn write_global_policy_deny_echo(&self) {
        self.write_global_policy(&["echo*"]);
    }

    fn write_global_policy(&self, cmd_denied: &[&str]) {
        let denied = if cmd_denied.is_empty() {
            "cmd_denied: []\n".to_string()
        } else {
            let items = cmd_denied
                .iter()
                .map(|v| format!("  - \"{}\"\n", v))
                .collect::<String>();
            format!("cmd_denied:\n{items}")
        };
        let contents = format!(
            r#"id: "ev0-override-split-test"
name: "EV0 Override Split Test Policy"

world_fs:
  mode: writable
  isolation: workspace
  require_world: false
  read_allowlist: ["*"]
  write_allowlist: []

net_allowed: []
cmd_allowed: []
{denied}cmd_isolated: []

require_approval: false
allow_shell_operators: true

limits:
  max_memory_mb: null
  max_cpu_percent: null
  max_runtime_ms: null
  max_egress_bytes: null

metadata: {{}}
"#
        );
        fs::write(self.substrate_home.join("policy.yaml"), contents).expect("write policy.yaml");
    }

    fn write_workspace_config(
        &self,
        workspace_root: &Path,
        policy_mode: &str,
        anchor_mode: &str,
        anchor_path: &str,
        caged: bool,
    ) {
        let caged = if caged { "true" } else { "false" };
        let contents = format!(
            "world:\n  enabled: false\n  anchor_mode: {anchor_mode}\n  anchor_path: \"{anchor_path}\"\n  caged: {caged}\n\npolicy:\n  mode: {policy_mode}\n\nsync:\n  auto_sync: false\n  direction: from_world\n  conflict_policy: prefer_host\n  exclude: []\n"
        );
        let marker = workspace_root.join(".substrate").join("workspace.yaml");
        fs::create_dir_all(marker.parent().expect("workspace .substrate parent"))
            .expect("create .substrate");
        fs::write(marker, contents).expect("write workspace.yaml");
    }

    fn command(&self, cwd: &Path) -> Command {
        let mut cmd = substrate_shell_driver();
        cmd.current_dir(cwd)
            .env("HOME", &self.home)
            .env("USERPROFILE", &self.home)
            .env("SUBSTRATE_HOME", &self.substrate_home)
            .env("SHIM_TRACE_LOG", &self.trace)
            .env("SHELL", "/bin/bash");

        for key in [
            "SUBSTRATE_POLICY_MODE",
            "SUBSTRATE_ANCHOR_MODE",
            "SUBSTRATE_ANCHOR_PATH",
            "SUBSTRATE_CAGED",
            "SUBSTRATE_SYNC_AUTO_SYNC",
            "SUBSTRATE_SYNC_DIRECTION",
            "SUBSTRATE_SYNC_CONFLICT_POLICY",
            "SUBSTRATE_SYNC_EXCLUDE",
            "SUBSTRATE_OVERRIDE_POLICY_MODE",
            "SUBSTRATE_OVERRIDE_ANCHOR_MODE",
            "SUBSTRATE_OVERRIDE_ANCHOR_PATH",
            "SUBSTRATE_OVERRIDE_CAGED",
            "SUBSTRATE_OVERRIDE_SYNC_AUTO_SYNC",
            "SUBSTRATE_OVERRIDE_SYNC_DIRECTION",
            "SUBSTRATE_OVERRIDE_SYNC_CONFLICT_POLICY",
            "SUBSTRATE_OVERRIDE_SYNC_EXCLUDE",
        ] {
            cmd.env_remove(key);
        }

        cmd
    }
}

fn read_trace(path: &Path) -> Vec<JsonValue> {
    let log_content = fs::read_to_string(path).expect("read trace.jsonl");
    log_content
        .lines()
        .filter(|line| !line.trim().is_empty())
        .filter_map(|line| serde_json::from_str::<JsonValue>(line).ok())
        .collect()
}

fn events_containing_marker<'a>(events: &'a [JsonValue], marker: &str) -> Vec<&'a JsonValue> {
    events
        .iter()
        .filter(|event| {
            event
                .get("command")
                .and_then(|v| v.as_str())
                .is_some_and(|cmd| cmd.contains(marker))
        })
        .collect()
}

fn policy_decisions_for_marker(events: &[JsonValue], marker: &str) -> Vec<JsonValue> {
    events_containing_marker(events, marker)
        .into_iter()
        .filter_map(|event| event.get("policy_decision").cloned())
        .filter(|value| !value.is_null())
        .collect()
}

#[test]
fn ev0_legacy_substrate_env_does_not_override_effective_config() {
    let fixture = OverrideSplitFixture::new("ev0-legacy-ignored-");
    fixture.write_global_config("disabled", "follow-cwd", "", false);
    fixture.write_global_policy_deny_echo();

    let anchor = fixture._temp.path().join("legacy-anchor");
    fs::create_dir_all(&anchor).expect("create legacy anchor");

    let marker = "__ev0_legacy_ignored__";
    let output = fixture
        .command(&fixture.project)
        .env("SUBSTRATE_POLICY_MODE", "observe")
        .env("SUBSTRATE_ANCHOR_MODE", "custom")
        .env("SUBSTRATE_ANCHOR_PATH", &anchor)
        .env("SUBSTRATE_CAGED", "1")
        .arg("-c")
        .arg(format!(
            "echo {marker} mode=$SUBSTRATE_ANCHOR_MODE caged=$SUBSTRATE_CAGED; :"
        ))
        .output()
        .expect("run substrate command");

    assert!(
        output.status.success(),
        "expected command to succeed (policy.mode=disabled): {output:?}"
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains(marker),
        "expected marker in stdout, got: {stdout}"
    );
    assert!(
        stdout.contains("mode=follow-cwd caged=0"),
        "expected world exports to reflect config (not legacy env): {stdout}"
    );

    let events = read_trace(&fixture.trace);
    assert!(
        policy_decisions_for_marker(&events, marker).is_empty(),
        "expected disabled mode to omit policy decisions"
    );
}

#[test]
fn ev0_override_env_overrides_effective_config_when_no_workspace_exists() {
    let fixture = OverrideSplitFixture::new("ev0-override-no-workspace-");
    fixture.write_global_config("disabled", "follow-cwd", "", false);
    fixture.write_global_policy_deny_echo();

    let anchor = fixture._temp.path().join("override-anchor");
    fs::create_dir_all(&anchor).expect("create override anchor");

    let marker = "__ev0_override_applies__";
    let output = fixture
        .command(&fixture.project)
        .env("SUBSTRATE_OVERRIDE_POLICY_MODE", "observe")
        .env("SUBSTRATE_OVERRIDE_ANCHOR_MODE", "custom")
        .env("SUBSTRATE_OVERRIDE_ANCHOR_PATH", &anchor)
        .env("SUBSTRATE_OVERRIDE_CAGED", "1")
        .arg("-c")
        .arg(format!(
            "echo {marker} mode=$SUBSTRATE_ANCHOR_MODE caged=$SUBSTRATE_CAGED; :"
        ))
        .output()
        .expect("run substrate command");

    assert!(
        output.status.success(),
        "expected command to succeed (policy.mode=observe): {output:?}"
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains(marker),
        "expected marker in stdout, got: {stdout}"
    );
    assert!(
        stdout.contains("mode=custom caged=1"),
        "expected world exports to reflect overrides: {stdout}"
    );

    let events = read_trace(&fixture.trace);
    let decisions = policy_decisions_for_marker(&events, marker);
    assert!(
        decisions
            .iter()
            .any(|value| value.get("action").and_then(|v| v.as_str()) == Some("deny")),
        "expected observe mode to record a would-deny decision: {decisions:?}"
    );
}

#[test]
fn ev0_workspace_config_wins_over_override_env() {
    let fixture = OverrideSplitFixture::new("ev0-workspace-wins-");
    fixture.write_global_config("observe", "follow-cwd", "", false);
    fixture.write_global_policy_deny_echo();

    let workspace_anchor = fixture._temp.path().join("workspace-anchor");
    fs::create_dir_all(&workspace_anchor).expect("create workspace anchor");

    fixture.write_workspace_config(
        &fixture.project,
        "disabled",
        "custom",
        &workspace_anchor.display().to_string(),
        true,
    );

    let cwd = fixture.project.join("nested");
    fs::create_dir_all(&cwd).expect("create nested cwd");

    let override_anchor = fixture._temp.path().join("override-anchor");
    fs::create_dir_all(&override_anchor).expect("create override anchor");

    let marker = "__ev0_workspace_wins__";
    let output = fixture
        .command(&cwd)
        .env("SUBSTRATE_OVERRIDE_POLICY_MODE", "observe")
        .env("SUBSTRATE_OVERRIDE_ANCHOR_MODE", "follow-cwd")
        .env("SUBSTRATE_OVERRIDE_ANCHOR_PATH", &override_anchor)
        .env("SUBSTRATE_OVERRIDE_CAGED", "0")
        .arg("-c")
        .arg(format!(
            "echo {marker} mode=$SUBSTRATE_ANCHOR_MODE caged=$SUBSTRATE_CAGED; :"
        ))
        .output()
        .expect("run substrate command");

    assert!(
        output.status.success(),
        "expected command to succeed (policy.mode=disabled): {output:?}"
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains(marker),
        "expected marker in stdout, got: {stdout}"
    );
    assert!(
        stdout.contains("mode=custom caged=1"),
        "expected world exports to reflect workspace config: {stdout}"
    );

    let events = read_trace(&fixture.trace);
    assert!(
        policy_decisions_for_marker(&events, marker).is_empty(),
        "expected workspace policy.mode=disabled to omit policy decisions"
    );
}
