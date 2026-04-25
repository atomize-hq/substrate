#![cfg(unix)]

mod common;

use common::{substrate_shell_driver, temp_dir};
use serde_json::{json, Value};
use std::fs;
use std::path::PathBuf;
use std::process::Output;
use tempfile::TempDir;

const PURE_AGENT_PROTOCOL: &str = "uaa.agent.session";

struct AgentSuccessorFixture {
    _temp: TempDir,
    home: PathBuf,
    substrate_home: PathBuf,
    workspace_root: PathBuf,
}

impl AgentSuccessorFixture {
    fn new() -> Self {
        let temp = temp_dir("substrate-agent-successor-");
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
            .env("SUBSTRATE_HOME", &self.substrate_home);
        cmd
    }

    fn init_workspace(&self) {
        let output = self
            .command()
            .arg("workspace")
            .arg("init")
            .arg(&self.workspace_root)
            .arg("--force")
            .output()
            .expect("failed to run workspace init");
        assert!(
            output.status.success(),
            "workspace init should succeed: {output:?}"
        );
    }

    fn write_global_config_patch(&self, contents: &str) {
        fs::write(self.substrate_home.join("config.yaml"), contents)
            .expect("failed to write config.yaml");
    }

    fn write_global_policy_patch(&self, contents: &str) {
        fs::write(self.substrate_home.join("policy.yaml"), contents)
            .expect("failed to write policy.yaml");
    }

    fn write_agent_file(&self, file_name: &str, contents: &str) {
        let agents_dir = self.substrate_home.join("agents");
        fs::create_dir_all(&agents_dir).expect("failed to create agents directory");
        fs::write(agents_dir.join(file_name), contents).expect("failed to write agent file");
    }

    fn write_trace_events(&self, events: &[Value]) {
        let trace = self.substrate_home.join("trace.jsonl");
        let body = events
            .iter()
            .map(|event| serde_json::to_string(event).expect("serialize trace event"))
            .collect::<Vec<_>>()
            .join("\n");
        fs::write(trace, format!("{body}\n")).expect("failed to write trace.jsonl");
    }

    fn run(&self, args: &[&str]) -> Output {
        self.command()
            .current_dir(&self.workspace_root)
            .args(args)
            .output()
            .expect("failed to run substrate command")
    }

    fn seed_inventory_for_list_and_status_contracts(&self) {
        self.write_global_config_patch(
            r#"agents:
  enabled: true
  hub:
    orchestrator_agent_id: claude_code
"#,
        );
        self.write_global_policy_patch(
            r#"agents:
  allowed_backends:
    - cli:claude_code
    - cli:codex
"#,
        );
        self.write_agent_file(
            "claude_code.yaml",
            &cli_agent_file("claude_code", "host", true, true, true),
        );
        self.write_agent_file(
            "codex.yaml",
            &cli_agent_file("codex", "world", true, false, true),
        );
    }

    fn seed_inventory_for_doctor_contract(&self) {
        self.seed_doctor_prereqs();
        self.write_agent_file(
            "claude_code.yaml",
            &cli_agent_file("claude_code", "host", true, true, true),
        );
        self.write_agent_file(
            "helper.yaml",
            &cli_agent_file("helper", "host", false, true, true),
        );
    }

    fn seed_doctor_prereqs(&self) {
        self.write_global_config_patch(
            r#"agents:
  enabled: true
  hub:
    orchestrator_agent_id: claude_code
"#,
        );
        self.write_global_policy_patch(
            r#"agents:
  allowed_backends:
    - cli:claude_code
    - cli:helper
"#,
        );
    }
}

fn cli_agent_file(
    agent_id: &str,
    scope: &str,
    llm: bool,
    mcp_client: bool,
    enabled: bool,
) -> String {
    cli_agent_file_with_session_contract(
        agent_id,
        scope,
        llm,
        mcp_client,
        enabled,
        Some(PURE_AGENT_PROTOCOL),
        None,
        None,
    )
}

fn cli_agent_file_with_session_contract(
    agent_id: &str,
    scope: &str,
    llm: bool,
    mcp_client: bool,
    enabled: bool,
    protocol: Option<&str>,
    false_capability: Option<&str>,
    omitted_capability: Option<&str>,
) -> String {
    assert!(
        false_capability.is_none()
            || omitted_capability.is_none()
            || false_capability != omitted_capability,
        "a capability cannot be both explicit-false and omitted"
    );

    let mut body =
        format!("version: 1\nid: {agent_id}\nconfig:\n  kind: cli\n  enabled: {enabled}\n");
    if let Some(protocol) = protocol {
        body.push_str(&format!("  protocol: {protocol}\n"));
    }
    body.push_str(&format!(
        "  execution:\n    scope: {scope}\n  cli:\n    binary: {agent_id}\n    mode: persistent\n  capabilities:\n"
    ));
    for capability in [
        "session_start",
        "session_resume",
        "session_fork",
        "session_stop",
        "status_snapshot",
        "event_stream",
    ] {
        if omitted_capability == Some(capability) {
            continue;
        }
        let value = false_capability != Some(capability);
        body.push_str(&format!("    {capability}: {value}\n"));
    }
    body.push_str(&format!("    llm: {llm}\n    mcp_client: {mcp_client}\n"));
    body
}

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("repo root should exist")
}

fn read_repo_file(path: &str) -> String {
    fs::read_to_string(repo_root().join(path))
        .unwrap_or_else(|err| panic!("failed to read {path}: {err}"))
}

fn parse_json_output(output: &Output) -> Value {
    serde_json::from_slice(&output.stdout).expect("stdout should be valid JSON")
}

fn assert_malformed_world_identity_failure(
    output: &Output,
    agent_id: &str,
    orchestration_session_id: &str,
    run_id: &str,
    ts: &str,
) {
    assert_eq!(
        output.status.code(),
        Some(2),
        "status should fail closed on malformed world identity: {output:?}"
    );
    assert!(
        String::from_utf8_lossy(&output.stdout).trim().is_empty(),
        "malformed world identity failures should not print stdout: {output:?}"
    );

    let stderr = String::from_utf8_lossy(&output.stderr);
    for needle in [
        "malformed world identity",
        &format!("agent_id={agent_id}"),
        &format!("orchestration_session_id={orchestration_session_id}"),
        &format!("run_id={run_id}"),
        &format!("ts={ts}"),
    ] {
        assert!(
            stderr.contains(needle),
            "stderr must contain `{needle}` for malformed world identity failures: {stderr}"
        );
    }
}

fn assert_doctor_fails_at_orchestrator_selection(output: &Output, expected_reason: &str) {
    assert_eq!(
        output.status.code(),
        Some(2),
        "doctor should fail closed at orchestrator_selection: {output:?}"
    );

    let json = parse_json_output(output);
    assert_eq!(json.get("healthy").and_then(Value::as_bool), Some(false));
    assert_eq!(json.get("fail_closed").and_then(Value::as_bool), Some(true));

    let checks = json["checks"]
        .as_array()
        .expect("checks should be an array");
    let observed: Vec<&str> = checks
        .iter()
        .map(|check| {
            check
                .pointer("/check")
                .and_then(Value::as_str)
                .expect("check id should be a string")
        })
        .collect();
    assert_eq!(
        observed,
        vec!["inventory_scan", "orchestrator_selection"],
        "doctor must stop before policy_allowlist/world_boundary on orchestrator failures: {json}"
    );
    assert_eq!(
        checks[1].pointer("/status").and_then(Value::as_str),
        Some("fail")
    );
    assert_eq!(
        checks[1].pointer("/reason").and_then(Value::as_str),
        Some(expected_reason),
        "doctor must publish the exact orchestrator denial reason: {json}"
    );
    assert!(
        json.get("orchestrator").is_none() || json["orchestrator"].is_null(),
        "failed orchestrator selection must not publish an orchestrator summary: {json}"
    );
}

fn find_session_by_agent<'a>(sessions: &'a [Value], agent_id: &str) -> &'a Value {
    sessions
        .iter()
        .find(|session| session.pointer("/agent_id").and_then(Value::as_str) == Some(agent_id))
        .unwrap_or_else(|| panic!("expected session row for agent `{agent_id}`"))
}

#[test]
fn plural_agents_namespace_keeps_validate_as_the_only_compatibility_leaf() {
    let fixture = AgentSuccessorFixture::new();
    fixture.init_workspace();
    fixture.write_agent_file(
        "codex.yaml",
        &cli_agent_file("codex", "world", true, false, true),
    );

    let validate = fixture.run(&["agents", "validate"]);
    assert!(
        validate.status.success(),
        "substrate agents validate must remain supported: {validate:?}"
    );

    for alias in [
        ["agents", "list"],
        ["agents", "status"],
        ["agents", "doctor"],
    ] {
        let output = fixture.run(&alias);
        assert_eq!(
            output.status.code(),
            Some(2),
            "`substrate {}` must remain an invalid plural alias: {output:?}",
            alias.join(" ")
        );
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(
            stderr.contains("unrecognized subcommand")
                || stderr.contains("unexpected argument")
                || stderr.contains("Usage:"),
            "plural alias failure should be a CLI usage error\nstderr: {stderr}"
        );
    }
}

#[test]
fn agent_list_json_locks_backend_id_derivation_role_and_omission_rules() {
    let fixture = AgentSuccessorFixture::new();
    fixture.init_workspace();
    fixture.seed_inventory_for_list_and_status_contracts();

    let output = fixture.run(&["agent", "list", "--json"]);
    assert!(
        output.status.success(),
        "substrate agent list --json should succeed for a valid successor fixture: {output:?}"
    );

    let json = parse_json_output(&output);
    assert_eq!(
        json.get("disabled").and_then(Value::as_bool),
        Some(false),
        "list output must report agents enabled: {json}"
    );
    assert_eq!(
        json.pointer("/scope_filter").and_then(Value::as_str),
        Some("any"),
        "default list scope filter must be `any`: {json}"
    );
    assert!(
        json.pointer("/role_filter")
            .is_some_and(serde_json::Value::is_null),
        "list output must publish a null role filter when --role is absent: {json}"
    );

    let agents = json["agents"]
        .as_array()
        .expect("agents should be an array");
    assert_eq!(
        agents.len(),
        2,
        "fixture should produce exactly two agents: {json}"
    );

    let orchestrator = agents
        .iter()
        .find(|agent| agent.pointer("/agent_id").and_then(Value::as_str) == Some("claude_code"))
        .expect("orchestrator row should exist");
    assert_eq!(
        orchestrator.pointer("/backend_id").and_then(Value::as_str),
        Some("cli:claude_code"),
        "backend_id must be derived as <kind>:<agent_id>: {orchestrator}"
    );
    assert_eq!(
        orchestrator.pointer("/role").and_then(Value::as_str),
        Some("orchestrator"),
        "role must remain a separate field from backend_id: {orchestrator}"
    );
    assert_eq!(
        orchestrator
            .pointer("/execution/scope")
            .and_then(Value::as_str),
        Some("host")
    );
    assert_eq!(
        orchestrator
            .pointer("/capabilities_summary/llm")
            .and_then(Value::as_bool),
        Some(true)
    );
    assert_eq!(
        orchestrator
            .pointer("/capabilities_summary/mcp_client")
            .and_then(Value::as_bool),
        Some(true)
    );
    assert_eq!(
        orchestrator.pointer("/protocol").and_then(Value::as_str),
        Some("uaa.agent.session")
    );

    let member = agents
        .iter()
        .find(|agent| agent.pointer("/agent_id").and_then(Value::as_str) == Some("codex"))
        .expect("member row should exist");
    assert_eq!(
        member.pointer("/backend_id").and_then(Value::as_str),
        Some("cli:codex"),
        "member backend_id must be derived as <kind>:<agent_id>: {member}"
    );
    assert!(
        member
            .pointer("/role")
            .is_some_and(serde_json::Value::is_null),
        "non-orchestrator rows must keep role separate and unassigned: {member}"
    );
    assert_eq!(
        member.pointer("/execution/scope").and_then(Value::as_str),
        Some("world")
    );

    for agent in agents {
        for forbidden in ["provider", "auth_authority", "world_id", "world_generation"] {
            assert!(
                agent.get(forbidden).is_none(),
                "agent list rows must omit `{forbidden}`: {agent}"
            );
        }
    }
}

#[test]
fn agent_status_json_uses_locked_top_level_field_names() {
    let fixture = AgentSuccessorFixture::new();
    fixture.init_workspace();
    fixture.seed_inventory_for_list_and_status_contracts();

    let output = fixture.run(&["agent", "status", "--json"]);
    assert!(
        output.status.success(),
        "substrate agent status --json should succeed for a valid successor fixture: {output:?}"
    );

    let json = parse_json_output(&output);
    assert_eq!(
        json.get("disabled").and_then(Value::as_bool),
        Some(false),
        "status output must report agents enabled: {json}"
    );
    assert_eq!(
        json.pointer("/scope_filter").and_then(Value::as_str),
        Some("any"),
        "default status scope filter must be `any`: {json}"
    );
    assert!(
        json.pointer("/role_filter")
            .is_some_and(serde_json::Value::is_null),
        "status output must publish a null role filter when --role is absent: {json}"
    );
    assert_eq!(
        json.pointer("/orchestrator_agent_id")
            .and_then(Value::as_str),
        Some("claude_code"),
        "status output must report orchestrator_agent_id as the selected inventory id: {json}"
    );
    assert!(
        json.get("sessions").is_some_and(Value::is_array),
        "status output must expose `sessions` as an array: {json}"
    );
    assert!(
        json.get("nested_llm_records").is_some_and(Value::is_array),
        "status output must expose `nested_llm_records` as an array: {json}"
    );
}

#[test]
fn agent_status_preserves_member_roles_and_filters_them_by_contract_label() {
    let fixture = AgentSuccessorFixture::new();
    fixture.init_workspace();
    fixture.seed_inventory_for_list_and_status_contracts();
    fixture.write_trace_events(&[
        json!({
            "ts": "2026-04-05T00:00:00Z",
            "event_type": "agent_event",
            "session_id": "ses_agent_hub",
            "component": "agent-hub",
            "kind": "status",
            "agent_id": "claude_code",
            "orchestration_session_id": "0195f8f1-7a34-7b7f-9c4d-9a7c2f5d6f12",
            "run_id": "0195f8f1-7a35-7b7f-9c4d-9a7c2f5d6f13",
            "backend_id": "cli:claude_code",
            "client": "claude_code",
            "router": "agent_hub",
            "protocol": "uaa.agent.session",
            "role": "orchestrator",
            "data": { "message": "orchestrator session is live" }
        }),
        json!({
            "ts": "2026-04-05T00:00:01Z",
            "event_type": "agent_event",
            "session_id": "ses_agent_hub",
            "component": "agent-hub",
            "kind": "status",
            "agent_id": "codex",
            "orchestration_session_id": "0195f8f1-7a34-7b7f-9c4d-9a7c2f5d6f12",
            "run_id": "0195f8f1-7a35-7b7f-9c4d-9a7c2f5d6f14",
            "backend_id": "cli:codex",
            "client": "codex",
            "router": "agent_hub",
            "protocol": "uaa.agent.session",
            "role": "member",
            "world_id": "wld_active_0002",
            "world_generation": 7,
            "data": { "message": "member session is live" }
        }),
    ]);

    let output = fixture.run(&["agent", "status", "--json"]);
    assert!(
        output.status.success(),
        "unfiltered agent status should succeed for member-role fixtures: {output:?}"
    );

    let json = parse_json_output(&output);
    assert!(
        json.pointer("/role_filter")
            .is_some_and(serde_json::Value::is_null),
        "unfiltered status must publish a null role_filter: {json}"
    );
    let sessions = json["sessions"]
        .as_array()
        .expect("sessions should be an array");
    assert_eq!(
        sessions.len(),
        2,
        "unfiltered status should keep both orchestrator and member sessions: {json}"
    );

    let orchestrator = find_session_by_agent(sessions, "claude_code");
    assert_eq!(
        orchestrator.pointer("/role").and_then(Value::as_str),
        Some("orchestrator"),
        "orchestrator session must preserve the orchestrator role label: {json}"
    );

    let member = find_session_by_agent(sessions, "codex");
    assert_eq!(
        member.pointer("/role").and_then(Value::as_str),
        Some("member"),
        "member session must preserve the member role label: {json}"
    );
    assert_eq!(
        member.pointer("/world_id").and_then(Value::as_str),
        Some("wld_active_0002"),
        "world-scoped member status rows must keep world_id: {json}"
    );
    assert_eq!(
        member.pointer("/world_generation").and_then(Value::as_u64),
        Some(7),
        "world-scoped member status rows must keep world_generation: {json}"
    );

    let member_output = fixture.run(&["agent", "status", "--role", "member", "--json"]);
    assert!(
        member_output.status.success(),
        "member-filtered agent status should succeed: {member_output:?}"
    );
    let member_json = parse_json_output(&member_output);
    assert_eq!(
        member_json.pointer("/role_filter").and_then(Value::as_str),
        Some("member"),
        "member-filtered status must report role_filter=member: {member_json}"
    );
    let member_sessions = member_json["sessions"]
        .as_array()
        .expect("sessions should be an array");
    assert_eq!(
        member_sessions.len(),
        1,
        "--role member should return exactly one member session: {member_json}"
    );
    let member_only = find_session_by_agent(member_sessions, "codex");
    assert_eq!(
        member_only.pointer("/role").and_then(Value::as_str),
        Some("member")
    );
    assert_eq!(
        member_only.pointer("/world_id").and_then(Value::as_str),
        Some("wld_active_0002")
    );
    assert_eq!(
        member_only
            .pointer("/world_generation")
            .and_then(Value::as_u64),
        Some(7)
    );

    let orchestrator_output = fixture.run(&["agent", "status", "--role", "orchestrator", "--json"]);
    assert!(
        orchestrator_output.status.success(),
        "orchestrator-filtered agent status should succeed: {orchestrator_output:?}"
    );
    let orchestrator_json = parse_json_output(&orchestrator_output);
    assert_eq!(
        orchestrator_json
            .pointer("/role_filter")
            .and_then(Value::as_str),
        Some("orchestrator"),
        "orchestrator-filtered status must report role_filter=orchestrator: {orchestrator_json}"
    );
    let orchestrator_sessions = orchestrator_json["sessions"]
        .as_array()
        .expect("sessions should be an array");
    assert_eq!(
        orchestrator_sessions.len(),
        1,
        "--role orchestrator should return exactly one orchestrator session: {orchestrator_json}"
    );
    let orchestrator_only = find_session_by_agent(orchestrator_sessions, "claude_code");
    assert_eq!(
        orchestrator_only.pointer("/role").and_then(Value::as_str),
        Some("orchestrator")
    );
}

#[test]
fn agent_status_prefers_newest_pure_session_event_when_trace_lines_are_out_of_order() {
    let fixture = AgentSuccessorFixture::new();
    fixture.init_workspace();
    fixture.seed_inventory_for_list_and_status_contracts();
    let orchestration_session_id = "0195f8f1-7a34-7b7f-9c4d-9a7c2f5d6f12";
    let run_id = "0195f8f1-7a35-7b7f-9c4d-9a7c2f5d6f14";
    let ts = "2026-04-05T00:00:02+00:00";
    fixture.write_trace_events(&[
        json!({
            "ts": "2026-04-05T00:00:02Z",
            "event_type": "agent_event",
            "session_id": "ses_agent_hub",
            "component": "agent-hub",
            "kind": "status",
            "agent_id": "codex",
            "orchestration_session_id": orchestration_session_id,
            "run_id": run_id,
            "backend_id": "cli:codex",
            "client": "codex",
            "router": "agent_hub",
            "protocol": "uaa.agent.session",
            "role": "member",
            "data": { "message": "newest member session event intentionally omits world fields" }
        }),
        json!({
            "ts": "2026-04-05T00:00:01Z",
            "event_type": "agent_event",
            "session_id": "ses_agent_hub",
            "component": "agent-hub",
            "kind": "status",
            "agent_id": "codex",
            "orchestration_session_id": orchestration_session_id,
            "run_id": "0195f8f1-7a35-7b7f-9c4d-9a7c2f5d6f15",
            "backend_id": "cli:codex",
            "client": "codex",
            "router": "agent_hub",
            "protocol": "uaa.agent.session",
            "role": "member",
            "world_id": "wld_stale_0001",
            "world_generation": 6,
            "data": { "message": "older stale member session event arrives later in file order" }
        }),
    ]);

    let output = fixture.run(&["agent", "status", "--json"]);
    assert_malformed_world_identity_failure(&output, "codex", orchestration_session_id, run_id, ts);
}

#[test]
fn agent_status_fails_when_newest_world_scoped_event_omits_top_level_world_id() {
    let fixture = AgentSuccessorFixture::new();
    fixture.init_workspace();
    fixture.seed_inventory_for_list_and_status_contracts();
    let orchestration_session_id = "0195f8f1-7a34-7b7f-9c4d-9a7c2f5d6f12";
    let run_id = "0195f8f1-7a35-7b7f-9c4d-9a7c2f5d6f16";
    let ts = "2026-04-05T00:00:03+00:00";
    fixture.write_trace_events(&[
        json!({
            "ts": "2026-04-05T00:00:03Z",
            "event_type": "agent_event",
            "session_id": "ses_agent_hub",
            "component": "agent-hub",
            "kind": "status",
            "agent_id": "codex",
            "orchestration_session_id": orchestration_session_id,
            "run_id": run_id,
            "backend_id": "cli:codex",
            "client": "codex",
            "router": "agent_hub",
            "protocol": "uaa.agent.session",
            "role": "member",
            "world_generation": 8,
            "data": { "message": "newest member session event omits top-level world_id" }
        }),
        json!({
            "ts": "2026-04-05T00:00:02Z",
            "event_type": "agent_event",
            "session_id": "ses_agent_hub",
            "component": "agent-hub",
            "kind": "status",
            "agent_id": "codex",
            "orchestration_session_id": orchestration_session_id,
            "run_id": "0195f8f1-7a35-7b7f-9c4d-9a7c2f5d6f15",
            "backend_id": "cli:codex",
            "client": "codex",
            "router": "agent_hub",
            "protocol": "uaa.agent.session",
            "role": "member",
            "world_id": "wld_stale_0001",
            "world_generation": 7,
            "data": { "message": "older event remains fully formed" }
        }),
    ]);

    let output = fixture.run(&["agent", "status", "--json"]);
    assert_malformed_world_identity_failure(&output, "codex", orchestration_session_id, run_id, ts);
}

#[test]
fn agent_status_scope_host_ignores_filtered_out_malformed_world_rows() {
    let fixture = AgentSuccessorFixture::new();
    fixture.init_workspace();
    fixture.seed_inventory_for_list_and_status_contracts();
    fixture.write_trace_events(&[
        json!({
            "ts": "2026-04-05T00:00:03Z",
            "event_type": "agent_event",
            "session_id": "ses_agent_hub",
            "component": "agent-hub",
            "kind": "status",
            "agent_id": "codex",
            "orchestration_session_id": "0195f8f1-7a34-7b7f-9c4d-9a7c2f5d6f12",
            "run_id": "0195f8f1-7a35-7b7f-9c4d-9a7c2f5d6f17",
            "backend_id": "cli:codex",
            "client": "codex",
            "router": "agent_hub",
            "protocol": "uaa.agent.session",
            "role": "member",
            "data": { "message": "filtered-out world row is malformed" }
        }),
        json!({
            "ts": "2026-04-05T00:00:02Z",
            "event_type": "agent_event",
            "session_id": "ses_agent_hub",
            "component": "agent-hub",
            "kind": "status",
            "agent_id": "claude_code",
            "orchestration_session_id": "0195f8f1-7a34-7b7f-9c4d-9a7c2f5d6f12",
            "run_id": "0195f8f1-7a35-7b7f-9c4d-9a7c2f5d6f18",
            "backend_id": "cli:claude_code",
            "client": "claude_code",
            "router": "agent_hub",
            "protocol": "uaa.agent.session",
            "role": "orchestrator",
            "data": { "message": "host-scoped orchestrator row remains valid" }
        }),
    ]);

    let output = fixture.run(&["agent", "status", "--scope", "host", "--json"]);
    assert!(
        output.status.success(),
        "host scope should ignore malformed world-scoped rows that are filtered out: {output:?}"
    );

    let json = parse_json_output(&output);
    let sessions = json["sessions"]
        .as_array()
        .expect("sessions should be an array");
    assert_eq!(
        sessions.len(),
        1,
        "--scope host should only emit the host-scoped session row: {json}"
    );
    let session = find_session_by_agent(sessions, "claude_code");
    assert_eq!(
        session.pointer("/execution/scope").and_then(Value::as_str),
        Some("host")
    );
}

#[test]
fn agent_status_ignores_non_selected_trace_orchestrator_roles() {
    let fixture = AgentSuccessorFixture::new();
    fixture.init_workspace();
    fixture.seed_inventory_for_list_and_status_contracts();
    fixture.write_trace_events(&[
        json!({
            "ts": "2026-04-05T00:00:00Z",
            "event_type": "agent_event",
            "session_id": "ses_agent_hub",
            "component": "agent-hub",
            "kind": "status",
            "agent_id": "claude_code",
            "orchestration_session_id": "0195f8f1-7a34-7b7f-9c4d-9a7c2f5d6f12",
            "run_id": "0195f8f1-7a35-7b7f-9c4d-9a7c2f5d6f13",
            "backend_id": "cli:claude_code",
            "client": "claude_code",
            "router": "agent_hub",
            "protocol": "uaa.agent.session",
            "role": "orchestrator",
            "data": { "message": "selected orchestrator session is live" }
        }),
        json!({
            "ts": "2026-04-05T00:00:01Z",
            "event_type": "agent_event",
            "session_id": "ses_agent_hub",
            "component": "agent-hub",
            "kind": "status",
            "agent_id": "codex",
            "orchestration_session_id": "0195f8f1-7a34-7b7f-9c4d-9a7c2f5d6f12",
            "run_id": "0195f8f1-7a35-7b7f-9c4d-9a7c2f5d6f14",
            "backend_id": "cli:codex",
            "client": "codex",
            "router": "agent_hub",
            "protocol": "uaa.agent.session",
            "role": "orchestrator",
            "world_id": "wld_active_0002",
            "world_generation": 7,
            "data": { "message": "stale member row claims orchestrator" }
        }),
    ]);

    let output = fixture.run(&["agent", "status", "--json"]);
    assert!(
        output.status.success(),
        "status should succeed when a non-selected agent claims orchestrator: {output:?}"
    );
    let json = parse_json_output(&output);
    let sessions = json["sessions"]
        .as_array()
        .expect("sessions should be an array");

    let orchestrator = find_session_by_agent(sessions, "claude_code");
    assert_eq!(
        orchestrator.pointer("/role").and_then(Value::as_str),
        Some("orchestrator"),
        "selected orchestrator must remain the only orchestrator session: {json}"
    );

    let codex = find_session_by_agent(sessions, "codex");
    assert!(
        codex
            .pointer("/role")
            .is_some_and(serde_json::Value::is_null),
        "non-selected stale orchestrator rows must collapse to null role: {json}"
    );

    let orchestrator_output = fixture.run(&["agent", "status", "--role", "orchestrator", "--json"]);
    assert!(
        orchestrator_output.status.success(),
        "orchestrator-filtered status should succeed: {orchestrator_output:?}"
    );
    let orchestrator_json = parse_json_output(&orchestrator_output);
    let orchestrator_sessions = orchestrator_json["sessions"]
        .as_array()
        .expect("sessions should be an array");
    assert_eq!(
        orchestrator_sessions.len(),
        1,
        "only the configured orchestrator should match --role orchestrator: {orchestrator_json}"
    );
    assert_eq!(
        find_session_by_agent(orchestrator_sessions, "claude_code")
            .pointer("/role")
            .and_then(Value::as_str),
        Some("orchestrator")
    );

    let member_output = fixture.run(&["agent", "status", "--role", "member", "--json"]);
    assert!(
        member_output.status.success(),
        "member-filtered status should succeed: {member_output:?}"
    );
    let member_json = parse_json_output(&member_output);
    assert_eq!(
        member_json.pointer("/role_filter").and_then(Value::as_str),
        Some("member"),
        "member-filtered status must echo the requested role filter: {member_json}"
    );
    assert_eq!(
        member_json["sessions"]
            .as_array()
            .expect("sessions should be an array")
            .len(),
        0,
        "stale non-selected orchestrator roles must not become filterable member rows: {member_json}"
    );
}

#[test]
fn agent_status_keeps_selected_orchestrator_host_scoped_when_trace_posture_says_in_world() {
    let fixture = AgentSuccessorFixture::new();
    fixture.init_workspace();
    fixture.write_global_config_patch(
        r#"agents:
  enabled: true
  hub:
    orchestrator_agent_id: claude_code
"#,
    );
    fixture.write_global_policy_patch(
        r#"agents:
  allowed_backends:
    - cli:claude_code
"#,
    );
    fixture.write_agent_file(
        "claude_code.yaml",
        &cli_agent_file("claude_code", "host", true, true, true),
    );
    fixture.write_trace_events(&[json!({
        "ts": "2026-04-05T00:00:00Z",
        "event_type": "agent_event",
        "session_id": "ses_agent_hub",
        "component": "agent-hub",
        "kind": "status",
        "agent_id": "claude_code",
        "orchestration_session_id": "0195f8f1-7a34-7b7f-9c4d-9a7c2f5d6f12",
        "run_id": "0195f8f1-7a35-7b7f-9c4d-9a7c2f5d6f13",
        "backend_id": "cli:claude_code",
        "client": "claude_code",
        "router": "agent_hub",
        "protocol": "uaa.agent.session",
        "role": "orchestrator",
        "placement_posture": {
            "execution": "in_world"
        },
        "world_id": "wld_active_0002",
        "world_generation": 7,
        "data": { "message": "trace posture should not move the selected orchestrator into world scope" }
    })]);

    let output = fixture.run(&["agent", "status", "--json"]);
    assert!(
        output.status.success(),
        "status should succeed when the selected orchestrator trace posture says in_world: {output:?}"
    );
    let json = parse_json_output(&output);
    let sessions = json["sessions"]
        .as_array()
        .expect("sessions should be an array");
    assert_eq!(
        sessions.len(),
        1,
        "fixture should project exactly one selected orchestrator session: {json}"
    );

    let orchestrator = &sessions[0];
    assert_eq!(
        orchestrator
            .pointer("/execution/scope")
            .and_then(Value::as_str),
        Some("host"),
        "selected orchestrator status rows must stay host-scoped despite trace posture: {json}"
    );
    assert!(
        orchestrator.get("world_id").is_none(),
        "host-scoped selected orchestrator rows must omit world_id: {json}"
    );
    assert!(
        orchestrator.get("world_generation").is_none(),
        "host-scoped selected orchestrator rows must omit world_generation: {json}"
    );

    let world_output = fixture.run(&["agent", "status", "--scope", "world", "--json"]);
    assert!(
        world_output.status.success(),
        "world-filtered status should succeed: {world_output:?}"
    );
    let world_json = parse_json_output(&world_output);
    assert_eq!(
        world_json["sessions"]
            .as_array()
            .expect("sessions should be an array")
            .len(),
        0,
        "world-filtered status must exclude the selected orchestrator row: {world_json}"
    );
}

#[test]
fn agent_status_unsupported_event_roles_fall_back_to_contract_roles() {
    let fixture = AgentSuccessorFixture::new();
    fixture.init_workspace();
    fixture.seed_inventory_for_list_and_status_contracts();
    fixture.write_trace_events(&[
        json!({
            "ts": "2026-04-05T00:00:00Z",
            "event_type": "agent_event",
            "session_id": "ses_agent_hub",
            "component": "agent-hub",
            "kind": "status",
            "agent_id": "claude_code",
            "orchestration_session_id": "0195f8f1-7a34-7b7f-9c4d-9a7c2f5d6f12",
            "run_id": "0195f8f1-7a35-7b7f-9c4d-9a7c2f5d6f13",
            "backend_id": "cli:claude_code",
            "client": "claude_code",
            "router": "agent_hub",
            "protocol": "uaa.agent.session",
            "role": "unexpected",
            "data": { "message": "unsupported role should not leak" }
        }),
        json!({
            "ts": "2026-04-05T00:00:01Z",
            "event_type": "agent_event",
            "session_id": "ses_agent_hub",
            "component": "agent-hub",
            "kind": "status",
            "agent_id": "codex",
            "orchestration_session_id": "0195f8f1-7a34-7b7f-9c4d-9a7c2f5d6f12",
            "run_id": "0195f8f1-7a35-7b7f-9c4d-9a7c2f5d6f14",
            "backend_id": "cli:codex",
            "client": "codex",
            "router": "agent_hub",
            "protocol": "uaa.agent.session",
            "role": "unexpected",
            "world_id": "wld_active_0002",
            "world_generation": 7,
            "data": { "message": "unsupported role should collapse to null" }
        }),
    ]);

    let output = fixture.run(&["agent", "status", "--json"]);
    assert!(
        output.status.success(),
        "status should succeed when unsupported event roles are present: {output:?}"
    );
    let json = parse_json_output(&output);
    let sessions = json["sessions"]
        .as_array()
        .expect("sessions should be an array");

    let orchestrator = find_session_by_agent(sessions, "claude_code");
    assert_eq!(
        orchestrator.pointer("/role").and_then(Value::as_str),
        Some("orchestrator"),
        "unsupported explicit roles for the configured orchestrator must fall back to orchestrator: {json}"
    );

    let member = find_session_by_agent(sessions, "codex");
    assert!(
        member
            .pointer("/role")
            .is_some_and(serde_json::Value::is_null),
        "unsupported explicit roles for non-orchestrators must fall back to null: {json}"
    );

    let unexpected_output = fixture.run(&["agent", "status", "--role", "unexpected", "--json"]);
    assert!(
        unexpected_output.status.success(),
        "status should not reject unknown role filters even when no rows match: {unexpected_output:?}"
    );
    let unexpected_json = parse_json_output(&unexpected_output);
    assert_eq!(
        unexpected_json
            .pointer("/role_filter")
            .and_then(Value::as_str),
        Some("unexpected"),
        "status should echo the requested unexpected role filter: {unexpected_json}"
    );
    assert_eq!(
        unexpected_json["sessions"]
            .as_array()
            .expect("sessions should be an array")
            .len(),
        0,
        "unsupported event roles must not become filterable session roles: {unexpected_json}"
    );
}

#[test]
fn agent_doctor_json_locks_field_names_omissions_and_check_order() {
    let fixture = AgentSuccessorFixture::new();
    fixture.init_workspace();
    fixture.seed_inventory_for_doctor_contract();

    let output = fixture.run(&["agent", "doctor", "--json"]);
    assert!(
        output.status.success(),
        "substrate agent doctor --json should succeed for a valid host-only successor fixture: {output:?}"
    );

    let json = parse_json_output(&output);
    assert_eq!(
        json.get("healthy").and_then(Value::as_bool),
        Some(true),
        "doctor should report a healthy control plane for the host-only fixture: {json}"
    );
    assert_eq!(
        json.get("fail_closed").and_then(Value::as_bool),
        Some(false),
        "healthy doctor output must not mark fail_closed: {json}"
    );
    assert_eq!(
        json.pointer("/orchestrator/agent_id")
            .and_then(Value::as_str),
        Some("claude_code")
    );
    assert_eq!(
        json.pointer("/orchestrator/backend_id")
            .and_then(Value::as_str),
        Some("cli:claude_code"),
        "doctor orchestrator summary must publish the derived backend_id: {json}"
    );
    assert_eq!(
        json.pointer("/orchestrator/execution/scope")
            .and_then(Value::as_str),
        Some("host"),
        "doctor orchestrator summary must preserve execution.scope: {json}"
    );
    assert!(
        json.pointer("/orchestrator/provider").is_none(),
        "doctor orchestrator summary must omit provider: {json}"
    );
    assert!(
        json.pointer("/orchestrator/auth_authority").is_none(),
        "doctor orchestrator summary must omit auth_authority: {json}"
    );

    let checks = json["checks"]
        .as_array()
        .expect("checks should be an array");
    let observed: Vec<&str> = checks
        .iter()
        .map(|check| {
            check
                .pointer("/check")
                .and_then(Value::as_str)
                .expect("check id should be a string")
        })
        .collect();
    assert_eq!(
        observed,
        vec![
            "inventory_scan",
            "orchestrator_selection",
            "policy_allowlist",
            "world_boundary",
        ],
        "doctor checks must stay in the contract-locked order: {json}"
    );

    let statuses: Vec<&str> = checks
        .iter()
        .map(|check| {
            check
                .pointer("/status")
                .and_then(Value::as_str)
                .expect("check status should be a string")
        })
        .collect();
    assert_eq!(
        statuses,
        vec!["pass", "pass", "pass", "not_applicable"],
        "host-only doctor fixture should report a not_applicable world boundary after the three required passes: {json}"
    );
}

#[test]
fn agent_doctor_fails_at_orchestrator_selection_when_protocol_is_missing() {
    let fixture = AgentSuccessorFixture::new();
    fixture.init_workspace();
    fixture.seed_doctor_prereqs();
    fixture.write_agent_file(
        "claude_code.yaml",
        &cli_agent_file_with_session_contract(
            "claude_code",
            "host",
            true,
            true,
            true,
            None,
            None,
            None,
        ),
    );
    fixture.write_agent_file(
        "helper.yaml",
        &cli_agent_file("helper", "host", false, true, true),
    );

    let output = fixture.run(&["agent", "doctor", "--json"]);
    assert_doctor_fails_at_orchestrator_selection(
        &output,
        "orchestrator agent 'claude_code' does not advertise protocol 'uaa.agent.session'",
    );
}

#[test]
fn agent_doctor_fails_at_orchestrator_selection_when_protocol_is_wrong() {
    let fixture = AgentSuccessorFixture::new();
    fixture.init_workspace();
    fixture.seed_doctor_prereqs();
    fixture.write_agent_file(
        "claude_code.yaml",
        &cli_agent_file_with_session_contract(
            "claude_code",
            "host",
            true,
            true,
            true,
            Some("openai.responses"),
            None,
            None,
        ),
    );
    fixture.write_agent_file(
        "helper.yaml",
        &cli_agent_file("helper", "host", false, true, true),
    );

    let output = fixture.run(&["agent", "doctor", "--json"]);
    assert_doctor_fails_at_orchestrator_selection(
        &output,
        "orchestrator agent 'claude_code' does not advertise protocol 'uaa.agent.session'",
    );
}

#[test]
fn agent_doctor_fails_at_orchestrator_selection_when_required_capability_is_false() {
    let fixture = AgentSuccessorFixture::new();
    fixture.init_workspace();
    fixture.seed_doctor_prereqs();
    fixture.write_agent_file(
        "claude_code.yaml",
        &cli_agent_file_with_session_contract(
            "claude_code",
            "host",
            true,
            true,
            true,
            Some(PURE_AGENT_PROTOCOL),
            Some("event_stream"),
            None,
        ),
    );
    fixture.write_agent_file(
        "helper.yaml",
        &cli_agent_file("helper", "host", false, true, true),
    );

    let output = fixture.run(&["agent", "doctor", "--json"]);
    assert_doctor_fails_at_orchestrator_selection(
        &output,
        "orchestrator agent 'claude_code' is missing required capability 'event_stream'",
    );
}

#[test]
fn agent_doctor_fails_at_orchestrator_selection_when_required_capability_is_omitted() {
    let fixture = AgentSuccessorFixture::new();
    fixture.init_workspace();
    fixture.seed_doctor_prereqs();
    fixture.write_agent_file(
        "claude_code.yaml",
        &cli_agent_file_with_session_contract(
            "claude_code",
            "host",
            true,
            true,
            true,
            Some(PURE_AGENT_PROTOCOL),
            None,
            Some("event_stream"),
        ),
    );
    fixture.write_agent_file(
        "helper.yaml",
        &cli_agent_file("helper", "host", false, true, true),
    );

    let output = fixture.run(&["agent", "doctor", "--json"]);
    assert_doctor_fails_at_orchestrator_selection(
        &output,
        "orchestrator agent 'claude_code' is missing required capability 'event_stream'",
    );
}

#[test]
fn docs_usage_and_repo_boundary_match_the_successor_contract() {
    let usage = read_repo_file("docs/USAGE.md");
    assert!(
        usage.contains("substrate agent list"),
        "docs/USAGE.md must document the canonical singular list command"
    );
    assert!(
        usage.contains("substrate agent status"),
        "docs/USAGE.md must document the canonical singular status command"
    );
    assert!(
        usage.contains("substrate agent doctor"),
        "docs/USAGE.md must document the canonical singular doctor command"
    );
    for forbidden in [
        "substrate agents list",
        "substrate agents status",
        "substrate agents doctor",
    ] {
        assert!(
            !usage.contains(forbidden),
            "docs/USAGE.md must not advertise plural successor aliases: {forbidden}"
        );
    }

    assert!(
        !repo_root().join("crates/agent-hub").exists(),
        "AHCSITC0 must not introduce a new crates/agent-hub package"
    );
}

#[test]
fn ahcsitc3_specs_lock_supersession_parity_and_validation_boundaries() {
    let compatibility = read_repo_file(
        "docs/project_management/packs/draft/agent-hub-core-successor-identity-tuple-compatible/compatibility-spec.md",
    );
    for required in [
        "ADR-0025 is historical evidence only",
        "ADR-0025 may be cited only as superseded historical evidence.",
        "Existing `agents.allowed_backends` entries remain valid without rewriting because `backend_id` stays derived as `<kind>:<agent_id>`.",
        "`substrate agents validate` remains supported as an additive compatibility leaf for inventory validation only.",
        "`backend_id` remains the agent-side adapter identifier and allowlist token",
    ] {
        assert!(
            compatibility.contains(required),
            "compatibility-spec.md must lock AHCSITC3 closeout rule `{required}`"
        );
    }

    let parity = read_repo_file(
        "docs/project_management/packs/draft/agent-hub-core-successor-identity-tuple-compatible/platform-parity-spec.md",
    );
    for required in [
        "| Linux |",
        "| macOS |",
        "| Windows |",
        "`substrate agents validate` remains a compatibility leaf on every platform and never becomes an alias for list, status, or doctor.",
        "Nested gateway-backed LLM records always omit `world_id` and `world_generation` on every platform.",
        "If the effective command path requires a world-scoped member posture and the required world boundary is temporarily unavailable, `substrate agent doctor` returns exit `3`.",
        "If the current build or platform cannot satisfy the required world posture at all, `substrate agent doctor` returns exit `4`.",
        "`crates/shell/tests/agents_validate.rs`",
        "`crates/shell/tests/agent_hub_trace_persistence.rs`",
        "`crates/shell/tests/repl_world_first_routing_v1.rs`",
        "`scripts/linux/world-provision.sh`",
        "`scripts/mac/lima-warm.sh`",
        "`scripts/mac/smoke.sh`",
        "`scripts/windows/wsl-warm.ps1`",
        "`scripts/windows/wsl-smoke.ps1`",
    ] {
        assert!(
            parity.contains(required),
            "platform-parity-spec.md must lock AHCSITC3 parity evidence `{required}`"
        );
    }

    let playbook = read_repo_file(
        "docs/project_management/packs/draft/agent-hub-core-successor-identity-tuple-compatible/manual_testing_playbook.md",
    );
    for required in [
        "### Case 1 — `substrate agent list --json` keeps adapter identity and omission rules",
        "### Case 2 — `substrate agent status --json` proves a host-scoped orchestrator",
        "### Case 3 — world-scoped members publish `world_id` and `world_generation`",
        "### Case 4 — nested gateway-backed records publish `run_id`, `provider`, and `auth_authority` on the nested record only",
        "### Case 5 — canonical trace keeps the same pure-agent versus nested-record split",
        "### Case 6 — `substrate agent doctor --json` proves healthy ordered checks",
        "### Case 7 — `substrate agent doctor` fails closed for invalid orchestrator state",
        "### Case 8 — `substrate agent doctor` fails closed for world-boundary loss",
        "`crates/shell/tests/agents_validate.rs`",
        "`crates/shell/tests/agent_hub_trace_persistence.rs`",
        "`crates/shell/tests/repl_world_first_routing_v1.rs`",
    ] {
        assert!(
            playbook.contains(required),
            "manual_testing_playbook.md must keep AHCSITC3 validation coverage `{required}`"
        );
    }
}

#[test]
fn ahcsitc3_configuration_doc_locks_successor_config_surface() {
    let configuration = read_repo_file("docs/CONFIGURATION.md");
    for required in [
        "agents.hub.orchestrator_agent_id",
        "agents.allowed_backends",
    ] {
        assert!(
            configuration.contains(required),
            "docs/CONFIGURATION.md must document successor config surface `{required}`"
        );
    }
}

#[test]
fn ahcsitc3_trace_doc_locks_tuple_compatible_fields() {
    let trace = read_repo_file("docs/TRACE.md");
    for required in [
        "`backend_id`",
        "`client`",
        "`router`",
        "`protocol`",
        "`provider`",
        "`auth_authority`",
        "`world_id`",
        "`world_generation`",
    ] {
        assert!(
            trace.contains(required),
            "docs/TRACE.md must document tuple-compatible trace field `{required}`"
        );
    }
}

#[test]
fn agent_doctor_fails_closed_on_world_member_allowlist_before_world_boundary() {
    let fixture = AgentSuccessorFixture::new();
    fixture.init_workspace();
    fixture.write_global_config_patch(
        r#"world:
  enabled: false
agents:
  enabled: true
  hub:
    orchestrator_agent_id: claude_code
"#,
    );
    fixture.write_global_policy_patch(
        r#"id: "ahcsitc2-policy"
name: "ahcsitc2-policy"

world_fs:
  host_visible: true
  fail_closed:
    routing: true
  write:
    enabled: true

agents:
  allowed_backends:
    - "cli:claude_code"

net_allowed: []
cmd_allowed: []
cmd_denied: []
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
    );
    fixture.write_agent_file(
        "claude_code.yaml",
        &cli_agent_file("claude_code", "host", true, true, true),
    );
    fixture.write_agent_file(
        "codex.yaml",
        &cli_agent_file("codex", "world", true, false, true),
    );

    let output = fixture.run(&["agent", "doctor", "--json"]);
    assert_eq!(
        output.status.code(),
        Some(5),
        "doctor should fail on the member backend allowlist before world boundary checks: {output:?}"
    );

    let json = parse_json_output(&output);
    assert_eq!(json.get("healthy").and_then(Value::as_bool), Some(false));
    assert_eq!(json.get("fail_closed").and_then(Value::as_bool), Some(true));

    let checks = json["checks"]
        .as_array()
        .expect("checks should be an array");
    let observed: Vec<&str> = checks
        .iter()
        .map(|check| {
            check
                .pointer("/check")
                .and_then(Value::as_str)
                .expect("check id should be a string")
        })
        .collect();
    assert_eq!(
        observed,
        vec!["inventory_scan", "orchestrator_selection", "policy_allowlist"],
        "fail-closed routing must stop at the member backend allowlist before world_boundary: {json}"
    );
    assert_eq!(
        checks[2].pointer("/reason").and_then(Value::as_str),
        Some(
            "required world-scoped member backend 'cli:codex' is not allowlisted by effective policy agents.allowed_backends"
        ),
        "member dispatch must be gated by the derived backend_id before world boundary handling: {json}"
    );
}

#[test]
fn agent_doctor_does_not_treat_trace_records_as_control_plane_authorization() {
    let fixture = AgentSuccessorFixture::new();
    fixture.init_workspace();
    fixture.write_global_config_patch(
        r#"agents:
  enabled: true
  hub:
    orchestrator_agent_id: ""
"#,
    );
    fixture.write_global_policy_patch(
        r#"id: "ahcsitc2-policy"
name: "ahcsitc2-policy"

world_fs:
  host_visible: true
  fail_closed:
    routing: true
  write:
    enabled: true

agents:
  allowed_backends:
    - "cli:claude_code"

net_allowed: []
cmd_allowed: []
cmd_denied: []
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
    );
    fixture.write_trace_events(&[json!({
        "ts": "2026-04-05T00:00:00Z",
        "event_type": "agent_event",
        "session_id": "ses_trace_only",
        "component": "agent-hub",
        "kind": "status",
        "agent_id": "claude_code",
        "orchestration_session_id": "0195f8f1-7a34-7b7f-9c4d-9a7c2f5d6f12",
        "run_id": "0195f8f1-7a35-7b7f-9c4d-9a7c2f5d6f13",
        "backend_id": "cli:claude_code",
        "client": "claude_code",
        "router": "agent_hub",
        "protocol": "uaa.agent.session",
        "data": { "message": "trace says the orchestrator is healthy" }
    })]);

    let output = fixture.run(&["agent", "doctor", "--json"]);
    assert_eq!(
        output.status.code(),
        Some(2),
        "trace-only observation must not authorize orchestrator selection: {output:?}"
    );

    let json = parse_json_output(&output);
    assert_eq!(json.get("healthy").and_then(Value::as_bool), Some(false));
    let checks = json["checks"]
        .as_array()
        .expect("checks should be an array");
    assert_eq!(
        checks[1].pointer("/check").and_then(Value::as_str),
        Some("orchestrator_selection"),
        "doctor must still fail at orchestrator selection even when trace records look healthy: {json}"
    );
    assert_eq!(
        checks[1].pointer("/status").and_then(Value::as_str),
        Some("fail")
    );
    assert_eq!(
        checks[1].pointer("/reason").and_then(Value::as_str),
        Some("agents.hub.orchestrator_agent_id must select an orchestrator"),
        "event-plane or trace-plane records must not back-authorize control-plane actions: {json}"
    );
}

#[test]
fn agent_status_uses_top_level_tuple_fields_for_pure_and_nested_records() {
    let fixture = AgentSuccessorFixture::new();
    fixture.init_workspace();
    fixture.write_global_config_patch(
        r#"agents:
  enabled: true
  hub:
    orchestrator_agent_id: claude_code
"#,
    );
    fixture.write_global_policy_patch(
        r#"id: "ahcsitc2-policy"
name: "ahcsitc2-policy"

world_fs:
  host_visible: true
  fail_closed:
    routing: true
  write:
    enabled: true

agents:
  allowed_backends:
    - "cli:claude_code"

net_allowed: []
cmd_allowed: []
cmd_denied: []
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
    );
    fixture.write_agent_file(
        "claude_code.yaml",
        &cli_agent_file("claude_code", "world", true, true, true),
    );
    fixture.write_trace_events(&[
        json!({
            "ts": "2026-04-05T00:00:00Z",
            "event_type": "agent_event",
            "session_id": "ses_agent_hub",
            "component": "agent-hub",
            "kind": "status",
            "agent_id": "claude_code",
            "orchestration_session_id": "0195f8f1-7a34-7b7f-9c4d-9a7c2f5d6f12",
            "run_id": "0195f8f1-7a35-7b7f-9c4d-9a7c2f5d6f13",
            "backend_id": "cli:claude_code",
            "client": "claude_code",
            "router": "agent_hub",
            "protocol": "uaa.agent.session",
            "role": "orchestrator",
            "world_id": "wld_active_0002",
            "world_generation": 7,
            "data": { "message": "pure-agent session is live" }
        }),
        json!({
            "ts": "2026-04-05T00:00:01Z",
            "event_type": "agent_event",
            "session_id": "ses_agent_hub",
            "component": "agent-hub",
            "kind": "status",
            "agent_id": "claude_code",
            "orchestration_session_id": "0195f8f1-7a34-7b7f-9c4d-9a7c2f5d6f12",
            "run_id": "0195f8f1-7a35-7b7f-9c4d-9a7c2f5d6f14",
            "parent_run_id": "0195f8f1-7a35-7b7f-9c4d-9a7c2f5d6f13",
            "backend_id": "cli:claude_code",
            "client": "claude_code",
            "router": "substrate_gateway",
            "protocol": "openai.responses",
            "provider": "openai",
            "auth_authority": "codex_subscription",
            "data": { "summary": "nested gateway request completed" }
        }),
    ]);

    let output = fixture.run(&["agent", "status", "--json"]);
    assert!(
        output.status.success(),
        "agent status should stay readable with tuple-compatible trace records: {output:?}"
    );

    let json = parse_json_output(&output);
    let sessions = json["sessions"]
        .as_array()
        .expect("sessions should be an array");
    assert_eq!(
        sessions.len(),
        1,
        "pure-agent records must still project into the status session surface from top-level tuple fields: {json}"
    );
    let session = &sessions[0];
    assert_eq!(
        session.pointer("/client").and_then(Value::as_str),
        Some("claude_code")
    );
    assert_eq!(
        session.pointer("/router").and_then(Value::as_str),
        Some("agent_hub")
    );
    assert_eq!(
        session.pointer("/protocol").and_then(Value::as_str),
        Some("uaa.agent.session")
    );
    assert_eq!(
        session.pointer("/world_id").and_then(Value::as_str),
        Some("wld_active_0002"),
        "world-scoped pure-agent records must publish world_id at the top level: {json}"
    );
    assert_eq!(
        session.pointer("/world_generation").and_then(Value::as_u64),
        Some(7),
        "world_generation must stay top-level on world-scoped pure-agent records: {json}"
    );
    assert!(
        session.get("provider").is_none() && session.get("auth_authority").is_none(),
        "pure-agent status sessions must omit nested gateway tuple fields: {session}"
    );

    let nested = json["nested_llm_records"]
        .as_array()
        .expect("nested_llm_records should be an array");
    assert_eq!(
        nested.len(),
        1,
        "nested gateway-backed records must remain distinct from pure-agent session rows: {json}"
    );
    let record = &nested[0];
    assert_eq!(
        record.pointer("/client").and_then(Value::as_str),
        Some("claude_code")
    );
    assert_eq!(
        record.pointer("/run_id").and_then(Value::as_str),
        Some("0195f8f1-7a35-7b7f-9c4d-9a7c2f5d6f14")
    );
    assert_eq!(
        record.pointer("/router").and_then(Value::as_str),
        Some("substrate_gateway")
    );
    assert_eq!(
        record.pointer("/protocol").and_then(Value::as_str),
        Some("openai.responses")
    );
    assert_eq!(
        record.pointer("/provider").and_then(Value::as_str),
        Some("openai")
    );
    assert_eq!(
        record.pointer("/auth_authority").and_then(Value::as_str),
        Some("codex_subscription")
    );
    assert!(
        record.get("world_id").is_none() && record.get("world_generation").is_none(),
        "nested gateway-backed records must omit world scope fields: {record}"
    );
}

#[test]
fn agent_status_preserves_same_tuple_nested_rows_and_sorts_them_by_run_id() {
    let fixture = AgentSuccessorFixture::new();
    fixture.init_workspace();
    fixture.write_global_config_patch(
        r#"agents:
  enabled: true
  hub:
    orchestrator_agent_id: claude_code
"#,
    );
    fixture.write_global_policy_patch(
        r#"id: "ahcsitc2-policy"
name: "ahcsitc2-policy"

world_fs:
  host_visible: true
  fail_closed:
    routing: true
  write:
    enabled: true

agents:
  allowed_backends:
    - "cli:claude_code"

net_allowed: []
cmd_allowed: []
cmd_denied: []
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
    );
    fixture.write_agent_file(
        "claude_code.yaml",
        &cli_agent_file("claude_code", "world", true, true, true),
    );
    fixture.write_trace_events(&[
        json!({
            "ts": "2026-04-05T00:00:00Z",
            "event_type": "agent_event",
            "session_id": "ses_agent_hub",
            "component": "agent-hub",
            "kind": "status",
            "agent_id": "claude_code",
            "orchestration_session_id": "0195f8f1-7a34-7b7f-9c4d-9a7c2f5d6f12",
            "run_id": "0195f8f1-7a35-7b7f-9c4d-9a7c2f5d6f13",
            "backend_id": "cli:claude_code",
            "client": "claude_code",
            "router": "agent_hub",
            "protocol": "uaa.agent.session",
            "role": "orchestrator",
            "world_id": "wld_active_0002",
            "world_generation": 7,
            "data": { "message": "pure-agent session is live" }
        }),
        json!({
            "ts": "2026-04-05T00:00:02Z",
            "event_type": "agent_event",
            "session_id": "ses_agent_hub",
            "component": "agent-hub",
            "kind": "status",
            "agent_id": "claude_code",
            "orchestration_session_id": "0195f8f1-7a34-7b7f-9c4d-9a7c2f5d6f12",
            "run_id": "0195f8f1-7a35-7b7f-9c4d-9a7c2f5d6f15",
            "parent_run_id": "0195f8f1-7a35-7b7f-9c4d-9a7c2f5d6f13",
            "backend_id": "cli:claude_code",
            "client": "claude_code",
            "router": "substrate_gateway",
            "protocol": "openai.responses",
            "provider": "openai",
            "auth_authority": "codex_subscription",
            "data": { "summary": "second nested gateway request completed" }
        }),
        json!({
            "ts": "2026-04-05T00:00:01Z",
            "event_type": "agent_event",
            "session_id": "ses_agent_hub",
            "component": "agent-hub",
            "kind": "status",
            "agent_id": "claude_code",
            "orchestration_session_id": "0195f8f1-7a34-7b7f-9c4d-9a7c2f5d6f12",
            "run_id": "0195f8f1-7a35-7b7f-9c4d-9a7c2f5d6f14",
            "parent_run_id": "0195f8f1-7a35-7b7f-9c4d-9a7c2f5d6f13",
            "backend_id": "cli:claude_code",
            "client": "claude_code",
            "router": "substrate_gateway",
            "protocol": "openai.responses",
            "provider": "openai",
            "auth_authority": "codex_subscription",
            "data": { "summary": "first nested gateway request completed" }
        }),
    ]);

    let output = fixture.run(&["agent", "status", "--json"]);
    assert!(
        output.status.success(),
        "agent status should preserve multiple same-tuple nested rows in JSON mode: {output:?}"
    );

    let json = parse_json_output(&output);
    let nested = json["nested_llm_records"]
        .as_array()
        .expect("nested_llm_records should be an array");
    assert_eq!(
        nested.len(),
        2,
        "same-tuple nested gateway records with different run_id values must stay distinct: {json}"
    );
    assert_eq!(
        nested[0].pointer("/run_id").and_then(Value::as_str),
        Some("0195f8f1-7a35-7b7f-9c4d-9a7c2f5d6f14")
    );
    assert_eq!(
        nested[1].pointer("/run_id").and_then(Value::as_str),
        Some("0195f8f1-7a35-7b7f-9c4d-9a7c2f5d6f15")
    );
    for record in nested {
        assert_eq!(
            record.pointer("/router").and_then(Value::as_str),
            Some("substrate_gateway")
        );
        assert_eq!(
            record.pointer("/provider").and_then(Value::as_str),
            Some("openai")
        );
        assert_eq!(
            record.pointer("/auth_authority").and_then(Value::as_str),
            Some("codex_subscription")
        );
        assert_eq!(
            record.pointer("/protocol").and_then(Value::as_str),
            Some("openai.responses")
        );
        assert!(
            record.get("world_id").is_none() && record.get("world_generation").is_none(),
            "nested gateway-backed records must omit world scope fields: {record}"
        );
    }

    let text_output = fixture.run(&["agent", "status"]);
    assert!(
        text_output.status.success(),
        "agent status should preserve multiple same-tuple nested rows in text mode: {text_output:?}"
    );
    let stdout = String::from_utf8_lossy(&text_output.stdout);
    assert!(
        stdout.contains("nested_llm_records"),
        "text mode should render a nested_llm_records section when nested records exist\nstdout: {stdout}"
    );
    let first_idx = stdout
        .find("run_id=0195f8f1-7a35-7b7f-9c4d-9a7c2f5d6f14")
        .expect("text mode should include the lexically first nested run_id");
    let second_idx = stdout
        .find("run_id=0195f8f1-7a35-7b7f-9c4d-9a7c2f5d6f15")
        .expect("text mode should include the lexically second nested run_id");
    assert!(
        first_idx < second_idx,
        "text mode should sort nested rows by run_id rather than insertion order\nstdout: {stdout}"
    );
}

#[test]
fn nested_gateway_policy_does_not_inherit_agent_hub_success_by_implication() {
    let fixture = AgentSuccessorFixture::new();
    fixture.init_workspace();
    fixture.write_global_config_patch(
        r#"agents:
  enabled: true
  hub:
    orchestrator_agent_id: claude_code
llm:
  enabled: true
  gateway:
    enabled: true
  routing:
    default_backend: cli:codex
"#,
    );
    fixture.write_global_policy_patch(
        r#"id: "ahcsitc2-policy"
name: "ahcsitc2-policy"

world_fs:
  host_visible: true
  fail_closed:
    routing: true
  write:
    enabled: true

llm:
  allowed_backends:
    - "api:openai"

agents:
  allowed_backends:
    - "cli:claude_code"

net_allowed: []
cmd_allowed: []
cmd_denied: []
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
    );
    fixture.write_agent_file(
        "claude_code.yaml",
        &cli_agent_file("claude_code", "host", true, true, true),
    );
    fixture.write_agent_file(
        "codex.yaml",
        &cli_agent_file("codex", "host", true, false, true),
    );

    let doctor = fixture.run(&["agent", "doctor", "--json"]);
    assert!(
        doctor.status.success(),
        "agent doctor must succeed for the host-scoped orchestrator fixture: {doctor:?}"
    );
    let doctor_json = parse_json_output(&doctor);
    assert_eq!(
        doctor_json.get("healthy").and_then(Value::as_bool),
        Some(true)
    );

    let missing_socket = fixture.workspace_root.join("missing-world.sock");
    let output = fixture
        .command()
        .current_dir(&fixture.workspace_root)
        .env_remove("SUBSTRATE_OVERRIDE_WORLD")
        .env("SUBSTRATE_WORLD_ENABLED", "1")
        .env("SUBSTRATE_WORLD", "enabled")
        .env("SUBSTRATE_WORLD_SOCKET", &missing_socket)
        .args(["world", "gateway", "status"])
        .output()
        .expect("failed to run world gateway status");

    assert_eq!(
        output.status.code(),
        Some(5),
        "nested gateway approval must remain an independent policy surface after agent-hub success: {output:?}"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("cli:codex is not allowlisted by effective policy llm.allowed_backends"),
        "nested gateway denial must come from llm.allowed_backends, not implied success: {stderr}"
    );
    assert!(
        !stderr.contains("required gateway/world component unavailable"),
        "gateway allowlist denial must happen before socket/runtime checks: {stderr}"
    );
}
