#![cfg(unix)]

mod common;

use assert_cmd::Command;
use common::{substrate_shell_driver, temp_dir};
use serde_json::Value as JsonValue;
use std::fs;
use std::path::{Path, PathBuf};
use tempfile::TempDir;

struct Pcm2Fixture {
    _temp: TempDir,
    home: PathBuf,
    substrate_home: PathBuf,
    project: PathBuf,
    trace: PathBuf,
}

impl Pcm2Fixture {
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

    fn write_global_config(&self, policy_mode: &str, world_enabled: bool) {
        let world_enabled = if world_enabled { "true" } else { "false" };
        let contents = format!(
            "world:\n  enabled: {world_enabled}\n  anchor_mode: follow-cwd\n  anchor_path: \"\"\n  caged: false\n\npolicy:\n  mode: {policy_mode}\n\nsync:\n  auto_sync: false\n  direction: from_world\n  conflict_policy: prefer_host\n  exclude: []\n"
        );
        fs::write(self.substrate_home.join("config.yaml"), contents)
            .expect("write global config.yaml");
    }

    fn write_global_policy(&self, cmd_denied: &[&str], require_world: bool) {
        let denied = if cmd_denied.is_empty() {
            "cmd_denied: []\n".to_string()
        } else {
            let items = cmd_denied
                .iter()
                .map(|v| format!("  - \"{}\"\n", v))
                .collect::<String>();
            format!("cmd_denied:\n{items}")
        };
        let require_world = if require_world { "true" } else { "false" };
        let contents = format!(
            r#"id: "pcm2-test"
name: "PCM2 Test Policy"

world_fs:
  mode: writable
  isolation: project
  require_world: {require_world}
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

    fn command(&self) -> Command {
        let mut cmd = substrate_shell_driver();
        cmd.env_remove("SUBSTRATE_POLICY_MODE");
        cmd.env_remove("SUBSTRATE_ANCHOR_MODE");
        cmd.env_remove("SUBSTRATE_ANCHOR_PATH");
        cmd.env_remove("SUBSTRATE_CAGED");
        cmd.env_remove("SUBSTRATE_SYNC_AUTO_SYNC");
        cmd.env_remove("SUBSTRATE_SYNC_DIRECTION");
        cmd.env_remove("SUBSTRATE_SYNC_CONFLICT_POLICY");
        cmd.env_remove("SUBSTRATE_SYNC_EXCLUDE");
        cmd.current_dir(&self.project)
            .env("HOME", &self.home)
            .env("USERPROFILE", &self.home)
            .env("SUBSTRATE_HOME", &self.substrate_home)
            .env("SHIM_TRACE_LOG", &self.trace)
            .env("SHELL", "/bin/bash");
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
fn pcm2_disabled_does_not_evaluate_policy_decisions() {
    let fixture = Pcm2Fixture::new("pcm2-disabled-");
    fixture.write_global_config("disabled", false);
    fixture.write_global_policy(&["echo*"], false);

    let marker = "__pcm2_disabled__";
    let output = fixture
        .command()
        .arg("-c")
        .arg(format!("echo {marker}"))
        .output()
        .expect("run substrate command");

    assert!(
        output.status.success(),
        "expected disabled mode to allow execution: {output:?}"
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains(marker),
        "expected command output, got stdout: {stdout}"
    );

    let events = read_trace(&fixture.trace);
    let matching = events_containing_marker(&events, marker);
    assert!(
        !matching.is_empty(),
        "expected command events to be logged for disabled mode"
    );
    assert!(
        policy_decisions_for_marker(&events, marker).is_empty(),
        "expected disabled mode to omit policy decisions"
    );
}

#[test]
fn pcm2_observe_allows_execution_and_records_would_deny() {
    let fixture = Pcm2Fixture::new("pcm2-observe-");
    fixture.write_global_config("observe", false);
    fixture.write_global_policy(&["echo*"], false);

    let marker = "__pcm2_observe__";
    let output = fixture
        .command()
        .arg("-c")
        .arg(format!("echo {marker}"))
        .output()
        .expect("run substrate command");

    assert!(
        output.status.success(),
        "expected observe mode to allow execution: {output:?}"
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains(marker),
        "expected command output, got stdout: {stdout}"
    );

    let events = read_trace(&fixture.trace);
    let decisions = policy_decisions_for_marker(&events, marker);
    assert!(
        !decisions.is_empty(),
        "expected observe mode to record policy decision"
    );

    let deny = decisions
        .iter()
        .find(|value| value.get("action").and_then(|v| v.as_str()) == Some("deny"));
    assert!(
        deny.is_some(),
        "expected observe mode to record a would-deny decision for denied commands: {decisions:?}"
    );
}

#[test]
fn pcm2_enforce_denies_cmd_denied() {
    let fixture = Pcm2Fixture::new("pcm2-enforce-deny-");
    fixture.write_global_config("enforce", false);
    fixture.write_global_policy(&["echo*"], false);

    let marker = "__pcm2_enforce_deny__";
    let output = fixture
        .command()
        .arg("-c")
        .arg(format!("echo {marker}"))
        .output()
        .expect("run substrate command");

    assert!(
        !output.status.success(),
        "expected enforce mode to deny cmd_denied matches: {output:?}"
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        !stdout.contains(marker),
        "expected denied command to not execute, got stdout: {stdout}"
    );

    let events = read_trace(&fixture.trace);
    let decisions = policy_decisions_for_marker(&events, marker);
    assert!(
        decisions
            .iter()
            .any(|value| value.get("action").and_then(|v| v.as_str()) == Some("deny")),
        "expected enforce mode to record deny decision: {decisions:?}"
    );
}

#[test]
#[cfg(target_os = "linux")]
fn pcm2_enforce_fails_closed_when_world_required_and_unavailable() {
    let fixture = Pcm2Fixture::new("pcm2-enforce-world-missing-");
    fixture.write_global_config("enforce", true);
    fixture.write_global_policy(&[], true);

    let marker = "__pcm2_requires_world__";
    let missing_socket = fixture.project.join("missing.sock");

    let output = fixture
        .command()
        .env("SUBSTRATE_WORLD", "enabled")
        .env("SUBSTRATE_WORLD_ENABLED", "1")
        .env("SUBSTRATE_WORLD_SOCKET", &missing_socket)
        .arg("-c")
        .arg(format!("echo {marker}"))
        .output()
        .expect("run substrate command");

    assert!(
        !output.status.success(),
        "expected enforce mode to fail closed when world is required but unavailable: {output:?}"
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        !stdout.contains(marker),
        "expected command to not run when world required and unavailable, got stdout: {stdout}"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("world") && (stderr.contains("unavailable") || stderr.contains("socket")),
        "expected stderr to mention world backend availability: {stderr}"
    );
}
