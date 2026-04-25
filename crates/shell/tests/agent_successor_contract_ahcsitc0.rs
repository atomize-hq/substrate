#![cfg(unix)]

mod common;

use common::{substrate_shell_driver, temp_dir};
use serde_json::Value;
use std::fs;
use std::path::PathBuf;
use std::process::Output;
use tempfile::TempDir;

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
        self.write_agent_file(
            "claude_code.yaml",
            &cli_agent_file("claude_code", "host", true, true, true),
        );
        self.write_agent_file(
            "helper.yaml",
            &cli_agent_file("helper", "host", false, true, true),
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
    format!(
        "version: 1\nid: {agent_id}\nconfig:\n  kind: cli\n  enabled: {enabled}\n  execution:\n    scope: {scope}\n  cli:\n    binary: {agent_id}\n    mode: persistent\n  capabilities:\n    llm: {llm}\n    mcp_client: {mcp_client}\n"
    )
}

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("repo root should exist")
}

fn parse_json_output(output: &Output) -> Value {
    serde_json::from_slice(&output.stdout).expect("stdout should be valid JSON")
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
fn docs_usage_and_repo_boundary_match_the_successor_contract() {
    let usage = fs::read_to_string(repo_root().join("docs/USAGE.md"))
        .expect("docs/USAGE.md should be readable");
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
