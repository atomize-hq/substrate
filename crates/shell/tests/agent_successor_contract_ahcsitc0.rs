#![cfg(unix)]

mod common;

use common::{substrate_shell_driver, temp_dir};
use serde_json::{json, Value};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Output;
use tempfile::TempDir;

const PURE_AGENT_PROTOCOL: &str = "uaa.agent.session";

#[derive(Clone, Copy)]
enum CapabilityOverride<'a> {
    ForceFalse(&'a str),
    Omit(&'a str),
}

#[derive(Clone, Copy)]
struct SessionContractOptions<'a> {
    protocol: Option<&'a str>,
    capability_override: Option<CapabilityOverride<'a>>,
    cli_mode: &'a str,
    binary: &'a str,
}

impl SessionContractOptions<'_> {
    const fn default() -> Self {
        Self {
            protocol: Some(PURE_AGENT_PROTOCOL),
            capability_override: None,
            cli_mode: "persistent",
            binary: "sh",
        }
    }
}

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

    fn seed_inventory_for_toolbox_contracts(&self, transport: &str) {
        self.write_global_config_patch(&format!(
            r#"agents:
  enabled: true
  hub:
    orchestrator_agent_id: claude_code
  toolbox:
    enabled: true
    bind:
      transport: {transport}
"#
        ));
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
        SessionContractOptions::default(),
    )
}

fn cli_agent_file_with_session_contract<'a>(
    agent_id: &str,
    scope: &str,
    llm: bool,
    mcp_client: bool,
    enabled: bool,
    options: SessionContractOptions<'a>,
) -> String {
    let mut body =
        format!("version: 1\nid: {agent_id}\nconfig:\n  kind: cli\n  enabled: {enabled}\n");
    if let Some(protocol) = options.protocol {
        body.push_str(&format!("  protocol: {protocol}\n"));
    }
    body.push_str(&format!(
        "  execution:\n    scope: {scope}\n  cli:\n    binary: {}\n    mode: {}\n  capabilities:\n",
        options.binary, options.cli_mode
    ));
    for capability in [
        "session_start",
        "session_resume",
        "session_fork",
        "session_stop",
        "status_snapshot",
        "event_stream",
    ] {
        if matches!(
            options.capability_override,
            Some(CapabilityOverride::Omit(omitted)) if omitted == capability
        ) {
            continue;
        }
        let value = !matches!(
            options.capability_override,
            Some(CapabilityOverride::ForceFalse(false_capability))
                if false_capability == capability
        );
        body.push_str(&format!("    {capability}: {value}\n"));
    }
    body.push_str(&format!("    llm: {llm}\n    mcp_client: {mcp_client}\n"));
    body
}

fn write_live_runtime_manifest(
    fixture: &AgentSuccessorFixture,
    agent_id: &str,
    orchestration_session_id: &str,
    participant_id: &str,
    ts: &str,
) {
    write_runtime_participant(
        fixture,
        participant_id,
        agent_id,
        orchestration_session_id,
        RuntimeParticipantOptions::host_orchestrator("running", true, ts),
    );
}

fn write_invalidated_runtime_manifest(
    fixture: &AgentSuccessorFixture,
    agent_id: &str,
    orchestration_session_id: &str,
    participant_id: &str,
    ts: &str,
) {
    write_runtime_participant(
        fixture,
        participant_id,
        agent_id,
        orchestration_session_id,
        RuntimeParticipantOptions::host_orchestrator("invalidated", false, ts),
    );
}

#[allow(clippy::too_many_arguments)]
fn write_invalidated_world_member_manifest(
    fixture: &AgentSuccessorFixture,
    agent_id: &str,
    orchestration_session_id: &str,
    participant_id: &str,
    orchestrator_participant_id: &str,
    world_id: &str,
    world_generation: u64,
    resumed_from_participant_id: Option<&str>,
    ts: &str,
) {
    write_runtime_participant(
        fixture,
        participant_id,
        agent_id,
        orchestration_session_id,
        RuntimeParticipantOptions::world_member(
            "invalidated",
            false,
            ts,
            world_id,
            world_generation,
            orchestrator_participant_id,
        )
        .with_resumed_from_participant_id(resumed_from_participant_id),
    );
}

#[allow(clippy::too_many_arguments)]
fn write_replacement_world_member_manifest(
    fixture: &AgentSuccessorFixture,
    agent_id: &str,
    orchestration_session_id: &str,
    participant_id: &str,
    orchestrator_participant_id: &str,
    world_id: &str,
    world_generation: u64,
    resumed_from_participant_id: &str,
    ts: &str,
) {
    write_runtime_participant(
        fixture,
        participant_id,
        agent_id,
        orchestration_session_id,
        RuntimeParticipantOptions::world_member(
            "ready",
            true,
            ts,
            world_id,
            world_generation,
            orchestrator_participant_id,
        )
        .with_resumed_from_participant_id(Some(resumed_from_participant_id)),
    );
}

#[derive(Clone, Copy)]
struct RuntimeParticipantOptions<'a> {
    role: &'a str,
    scope: &'a str,
    state: &'a str,
    ownership_mode: &'a str,
    ownership_valid: bool,
    ts: &'a str,
    world_binding: Option<(&'a str, u64)>,
    parent_participant_id: Option<&'a str>,
    resumed_from_participant_id: Option<&'a str>,
    orchestrator_participant_id: Option<&'a str>,
}

impl<'a> RuntimeParticipantOptions<'a> {
    fn host_orchestrator(state: &'a str, ownership_valid: bool, ts: &'a str) -> Self {
        Self {
            role: "orchestrator",
            scope: "host",
            state,
            ownership_mode: "attached_control",
            ownership_valid,
            ts,
            world_binding: None,
            parent_participant_id: None,
            resumed_from_participant_id: None,
            orchestrator_participant_id: None,
        }
    }

    fn world_member(
        state: &'a str,
        ownership_valid: bool,
        ts: &'a str,
        world_id: &'a str,
        world_generation: u64,
        orchestrator_participant_id: &'a str,
    ) -> Self {
        Self {
            role: "member",
            scope: "world",
            state,
            ownership_mode: "member_runtime",
            ownership_valid,
            ts,
            world_binding: Some((world_id, world_generation)),
            parent_participant_id: None,
            resumed_from_participant_id: None,
            orchestrator_participant_id: Some(orchestrator_participant_id),
        }
    }

    fn with_resumed_from_participant_id(
        mut self,
        resumed_from_participant_id: Option<&'a str>,
    ) -> Self {
        self.resumed_from_participant_id = resumed_from_participant_id;
        self
    }
}

fn write_runtime_participant(
    fixture: &AgentSuccessorFixture,
    participant_id: &str,
    agent_id: &str,
    orchestration_session_id: &str,
    options: RuntimeParticipantOptions<'_>,
) {
    write_json_file(
        &canonical_participant_manifest_path(fixture, orchestration_session_id, participant_id),
        &runtime_participant_manifest(
            fixture,
            participant_id,
            agent_id,
            orchestration_session_id,
            options,
        ),
    );
}

fn write_flat_runtime_participant_compatibility(
    fixture: &AgentSuccessorFixture,
    participant_id: &str,
    agent_id: &str,
    orchestration_session_id: &str,
    options: RuntimeParticipantOptions<'_>,
) {
    write_json_file(
        &flat_participant_manifest_path(fixture, participant_id),
        &runtime_participant_manifest(
            fixture,
            participant_id,
            agent_id,
            orchestration_session_id,
            options,
        ),
    );
}

fn runtime_participant_manifest(
    _fixture: &AgentSuccessorFixture,
    participant_id: &str,
    agent_id: &str,
    orchestration_session_id: &str,
    options: RuntimeParticipantOptions<'_>,
) -> Value {
    let mut manifest = serde_json::Map::new();
    manifest.insert("participant_id".to_string(), json!(participant_id));
    manifest.insert(
        "orchestration_session_id".to_string(),
        json!(orchestration_session_id),
    );
    manifest.insert("agent_id".to_string(), json!(agent_id));
    manifest.insert("backend_id".to_string(), json!(format!("cli:{agent_id}")));
    manifest.insert("role".to_string(), json!(options.role));
    manifest.insert("protocol".to_string(), json!(PURE_AGENT_PROTOCOL));
    manifest.insert("execution".to_string(), json!({ "scope": options.scope }));
    manifest.insert("state".to_string(), json!(options.state));
    manifest.insert("opened_at".to_string(), json!(options.ts));
    manifest.insert("last_transition_at".to_string(), json!(options.ts));
    if let Some((world_id, world_generation)) = options.world_binding {
        manifest.insert("world_id".to_string(), json!(world_id));
        manifest.insert("world_generation".to_string(), json!(world_generation));
    }
    if let Some(parent_participant_id) = options.parent_participant_id {
        manifest.insert(
            "parent_participant_id".to_string(),
            json!(parent_participant_id),
        );
    }
    if let Some(resumed_from_participant_id) = options.resumed_from_participant_id {
        manifest.insert(
            "resumed_from_participant_id".to_string(),
            json!(resumed_from_participant_id),
        );
    }
    if let Some(orchestrator_participant_id) = options.orchestrator_participant_id {
        manifest.insert(
            "orchestrator_participant_id".to_string(),
            json!(orchestrator_participant_id),
        );
    }
    manifest.insert(
        "internal".to_string(),
        json!({
            "resolved_agent_kind": agent_id,
            "resolved_binary_path": "sh",
            "shell_owner_pid": std::process::id(),
            "lease_token": format!("lease-{participant_id}"),
            "uaa_session_id": if options.ownership_valid { Value::String("external-session-1".to_string()) } else { Value::Null },
            "latest_run_id": "0195f8f1-7a35-7b7f-9c4d-9a7c2f5d6fab",
            "cancel_supported": true,
            "control_owner_retained": options.ownership_valid,
            "event_stream_active": options.ownership_valid,
            "completion_observer_retained": options.ownership_valid,
            "ownership_mode": options.ownership_mode,
            "ownership_valid": options.ownership_valid,
            "ownership_verified_at": options.ts,
            "last_heartbeat_at": options.ts,
            "last_event_at": options.ts,
            "terminal_observed_at": if options.ownership_valid { Value::Null } else { json!(options.ts) },
            "termination_reason": if options.ownership_valid { Value::Null } else { json!("attached control exited") },
            "last_error_bucket": null,
            "last_error_message": null
        }),
    );
    Value::Object(manifest)
}

fn participant_manifest_path(fixture: &AgentSuccessorFixture, participant_id: &str) -> PathBuf {
    let participant_dirs = fixture
        .substrate_home
        .join("run")
        .join("agent-hub")
        .join("sessions");
    fs::read_dir(&participant_dirs)
        .expect("canonical sessions dir should exist")
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .find_map(|path| {
            let candidate = path
                .join("participants")
                .join(format!("{participant_id}.json"));
            candidate.is_file().then_some(candidate)
        })
        .unwrap_or_else(|| {
            panic!("expected canonical participant manifest for `{participant_id}` to exist")
        })
}

fn canonical_participant_manifest_path(
    fixture: &AgentSuccessorFixture,
    orchestration_session_id: &str,
    participant_id: &str,
) -> PathBuf {
    fixture
        .substrate_home
        .join("run")
        .join("agent-hub")
        .join("sessions")
        .join(orchestration_session_id)
        .join("participants")
        .join(format!("{participant_id}.json"))
}

fn flat_participant_manifest_path(
    fixture: &AgentSuccessorFixture,
    participant_id: &str,
) -> PathBuf {
    fixture
        .substrate_home
        .join("run")
        .join("agent-hub")
        .join("participants")
        .join(format!("{participant_id}.json"))
}

fn canonical_orchestration_session_path(
    fixture: &AgentSuccessorFixture,
    orchestration_session_id: &str,
) -> PathBuf {
    fixture
        .substrate_home
        .join("run")
        .join("agent-hub")
        .join("sessions")
        .join(orchestration_session_id)
        .join("session.json")
}

fn flat_orchestration_session_path(
    fixture: &AgentSuccessorFixture,
    orchestration_session_id: &str,
) -> PathBuf {
    fixture
        .substrate_home
        .join("run")
        .join("agent-hub")
        .join("sessions")
        .join(format!("{orchestration_session_id}.json"))
}

fn write_active_orchestration_session(
    fixture: &AgentSuccessorFixture,
    agent_id: &str,
    orchestration_session_id: &str,
    active_session_handle_id: Option<&str>,
    ts: &str,
) {
    write_orchestration_session(
        fixture,
        agent_id,
        orchestration_session_id,
        active_session_handle_id,
        "active",
        ts,
    );
}

fn write_active_orchestration_session_with_world_binding(
    fixture: &AgentSuccessorFixture,
    agent_id: &str,
    orchestration_session_id: &str,
    active_session_handle_id: Option<&str>,
    ts: &str,
    world_id: &str,
    world_generation: u64,
) {
    write_orchestration_session_with_world_binding(
        fixture,
        agent_id,
        orchestration_session_id,
        active_session_handle_id,
        "active",
        ts,
        (world_id, world_generation),
    );
}

fn write_inactive_orchestration_session(
    fixture: &AgentSuccessorFixture,
    agent_id: &str,
    orchestration_session_id: &str,
    active_session_handle_id: Option<&str>,
    ts: &str,
) {
    write_orchestration_session(
        fixture,
        agent_id,
        orchestration_session_id,
        active_session_handle_id,
        "stopped",
        ts,
    );
}

fn write_orchestration_session_with_world_binding(
    fixture: &AgentSuccessorFixture,
    agent_id: &str,
    orchestration_session_id: &str,
    active_session_handle_id: Option<&str>,
    state: &str,
    ts: &str,
    world_binding: (&str, u64),
) {
    write_orchestration_session_impl(
        fixture,
        agent_id,
        orchestration_session_id,
        active_session_handle_id,
        state,
        ts,
        Some(world_binding),
    );
}

fn write_orchestration_session(
    fixture: &AgentSuccessorFixture,
    agent_id: &str,
    orchestration_session_id: &str,
    active_session_handle_id: Option<&str>,
    state: &str,
    ts: &str,
) {
    write_orchestration_session_impl(
        fixture,
        agent_id,
        orchestration_session_id,
        active_session_handle_id,
        state,
        ts,
        None,
    );
}

fn write_orchestration_session_impl(
    fixture: &AgentSuccessorFixture,
    agent_id: &str,
    orchestration_session_id: &str,
    active_session_handle_id: Option<&str>,
    state: &str,
    ts: &str,
    world_binding: Option<(&str, u64)>,
) {
    write_json_file(
        &canonical_orchestration_session_path(fixture, orchestration_session_id),
        &orchestration_session_manifest(
            fixture,
            agent_id,
            orchestration_session_id,
            active_session_handle_id,
            state,
            ts,
            world_binding,
        ),
    );
}

fn write_flat_orchestration_session_compatibility(
    fixture: &AgentSuccessorFixture,
    agent_id: &str,
    orchestration_session_id: &str,
    active_session_handle_id: Option<&str>,
    state: &str,
    ts: &str,
    world_binding: Option<(&str, u64)>,
) {
    write_json_file(
        &flat_orchestration_session_path(fixture, orchestration_session_id),
        &orchestration_session_manifest(
            fixture,
            agent_id,
            orchestration_session_id,
            active_session_handle_id,
            state,
            ts,
            world_binding,
        ),
    );
}

fn orchestration_session_manifest(
    fixture: &AgentSuccessorFixture,
    agent_id: &str,
    orchestration_session_id: &str,
    active_session_handle_id: Option<&str>,
    state: &str,
    ts: &str,
    world_binding: Option<(&str, u64)>,
) -> Value {
    let (world_id, world_generation) = match world_binding {
        Some((world_id, world_generation)) => (json!(world_id), json!(world_generation)),
        None => (Value::Null, Value::Null),
    };
    json!({
        "orchestration_session_id": orchestration_session_id,
        "shell_trace_session_id": "ses_agent_hub",
        "workspace_root": fixture.workspace_root.display().to_string(),
        "shell_owner_pid": std::process::id(),
        "state": state,
        "opened_at": ts,
        "last_active_at": ts,
        "orchestrator_agent_id": agent_id,
        "orchestrator_backend_id": format!("cli:{agent_id}"),
        "orchestrator_protocol": PURE_AGENT_PROTOCOL,
        "active_session_handle_id": active_session_handle_id,
        "latest_run_id": "0195f8f1-7a35-7b7f-9c4d-9a7c2f5d6fab",
        "world_id": world_id,
        "world_generation": world_generation,
        "invalidation_reason": if state == "active" { Value::Null } else { json!("fixture stopped parent") },
        "closed_at": if state == "active" { Value::Null } else { json!(ts) }
    })
}

fn write_json_file(path: &Path, value: &Value) {
    let parent = path
        .parent()
        .expect("json fixture path should have a parent");
    fs::create_dir_all(parent)
        .unwrap_or_else(|err| panic!("failed to create fixture dir {}: {err}", parent.display()));
    fs::write(
        path,
        serde_json::to_vec_pretty(value).expect("serialize fixture json"),
    )
    .unwrap_or_else(|err| panic!("failed to write fixture json {}: {err}", path.display()));
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

fn read_production_shell_source_without_inline_tests(path: &Path) -> String {
    let source = fs::read_to_string(path)
        .unwrap_or_else(|err| panic!("failed to read production source {}: {err}", path.display()));
    let cutoff = source.find("#[cfg(test)]").unwrap_or(source.len());
    source[..cutoff].to_string()
}

fn collect_rust_files(root: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    let mut stack = vec![root.to_path_buf()];
    while let Some(path) = stack.pop() {
        let entries = fs::read_dir(&path)
            .unwrap_or_else(|err| panic!("failed to read directory {}: {err}", path.display()));
        for entry in entries {
            let entry = entry.unwrap_or_else(|err| {
                panic!("failed to enumerate directory {}: {err}", path.display())
            });
            let entry_path = entry.path();
            if entry_path.is_dir() {
                stack.push(entry_path);
            } else if entry_path.extension().and_then(|ext| ext.to_str()) == Some("rs") {
                files.push(entry_path);
            }
        }
    }
    files.sort();
    files
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

fn assert_malformed_nested_parent_correlation_failure(
    output: &Output,
    agent_id: &str,
    orchestration_session_id: &str,
    run_id: &str,
    parent_run_id: &str,
) {
    assert_eq!(
        output.status.code(),
        Some(2),
        "status should fail closed on malformed nested parent correlation: {output:?}"
    );
    assert!(
        String::from_utf8_lossy(&output.stdout).trim().is_empty(),
        "malformed nested parent correlation failures should not print stdout: {output:?}"
    );

    let stderr = String::from_utf8_lossy(&output.stderr);
    for needle in [
        "malformed nested parent correlation on selected status surface",
        &format!("agent_id={agent_id}"),
        &format!("orchestration_session_id={orchestration_session_id}"),
        &format!("run_id={run_id}"),
        &format!("parent_run_id={parent_run_id}"),
    ] {
        assert!(
            stderr.contains(needle),
            "stderr must contain `{needle}` for malformed nested parent correlation failures: {stderr}"
        );
    }
}

fn assert_malformed_nested_required_fields_failure(
    output: &Output,
    agent_id: &str,
    orchestration_session_id: &str,
    run_id: &str,
    missing_fields: &str,
) {
    assert_eq!(
        output.status.code(),
        Some(2),
        "status should fail closed on malformed selected nested tuple fields: {output:?}"
    );
    assert!(
        String::from_utf8_lossy(&output.stdout).trim().is_empty(),
        "malformed selected nested tuple field failures should not print stdout: {output:?}"
    );

    let stderr = String::from_utf8_lossy(&output.stderr);
    for needle in [
        "malformed nested tuple on selected status surface",
        &format!("agent_id={agent_id}"),
        &format!("orchestration_session_id={orchestration_session_id}"),
        &format!("run_id={run_id}"),
        &format!("missing_fields={missing_fields}"),
    ] {
        assert!(
            stderr.contains(needle),
            "stderr must contain `{needle}` for malformed selected nested tuple field failures: {stderr}"
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

fn assert_parent_resolution_fail_closed(
    output: &Output,
    command: &str,
    expected_stderr_fragments: &[&str],
) {
    assert_eq!(
        output.status.code(),
        Some(2),
        "{command} must fail closed on broken orchestrator parent resolution: {output:?}"
    );
    assert!(
        String::from_utf8_lossy(&output.stdout).trim().is_empty(),
        "{command} failures must not emit stdout: {output:?}"
    );

    let stderr = String::from_utf8_lossy(&output.stderr);
    for needle in expected_stderr_fragments {
        assert!(
            stderr.contains(needle),
            "{command} stderr must contain `{needle}`: {stderr}"
        );
    }
}

fn assert_parent_resolution_fail_closed_across_operator_surfaces(
    fixture: &AgentSuccessorFixture,
    expected_stderr_fragments: &[&str],
) {
    for args in [
        vec!["agent", "status", "--json"],
        vec!["agent", "toolbox", "status", "--json"],
        vec!["agent", "toolbox", "env", "--json"],
    ] {
        let output = fixture.run(&args);
        assert_parent_resolution_fail_closed(&output, &args.join(" "), expected_stderr_fragments);
    }
}

fn find_session_by_agent<'a>(sessions: &'a [Value], agent_id: &str) -> &'a Value {
    sessions
        .iter()
        .find(|session| session.pointer("/agent_id").and_then(Value::as_str) == Some(agent_id))
        .unwrap_or_else(|| panic!("expected session row for agent `{agent_id}`"))
}

fn find_session_by_agent_and_orchestration_session<'a>(
    sessions: &'a [Value],
    agent_id: &str,
    orchestration_session_id: &str,
) -> &'a Value {
    sessions
        .iter()
        .find(|session| {
            session.pointer("/agent_id").and_then(Value::as_str) == Some(agent_id)
                && session
                    .pointer("/orchestration_session_id")
                    .and_then(Value::as_str)
                    == Some(orchestration_session_id)
        })
        .unwrap_or_else(|| {
            panic!(
                "expected session row for agent `{agent_id}` in orchestration session `{orchestration_session_id}`"
            )
        })
}

fn seed_nested_gateway_status_fixture(fixture: &AgentSuccessorFixture) {
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
fn agent_toolbox_status_json_reports_template_when_no_active_orchestrator_session_exists() {
    let fixture = AgentSuccessorFixture::new();
    fixture.init_workspace();
    fixture.seed_inventory_for_toolbox_contracts("uds");
    let expected_template = format!(
        "unix://{}/run/agent-toolbox/<orchestration_session_id>.sock",
        fixture.substrate_home.display()
    );

    let output = fixture.run(&["agent", "toolbox", "status", "--json"]);
    assert!(
        output.status.success(),
        "toolbox status should succeed when the surface is enabled but no session is active: {output:?}"
    );

    let json = parse_json_output(&output);
    assert_eq!(
        json.pointer("/toolbox_enabled").and_then(Value::as_bool),
        Some(true)
    );
    assert_eq!(
        json.pointer("/toolbox_version").and_then(Value::as_u64),
        Some(1)
    );
    assert_eq!(
        json.pointer("/transport").and_then(Value::as_str),
        Some("uds")
    );
    assert_eq!(
        json.pointer("/eligibility/state").and_then(Value::as_str),
        Some("dependency_unavailable")
    );
    assert!(
        json.pointer("/active_orchestration_session_id")
            .is_some_and(Value::is_null),
        "status must publish a null active session id when no orchestrator session is active: {json}"
    );
    assert!(
        json.pointer("/endpoint").is_none() || json["endpoint"].is_null(),
        "status must omit a concrete endpoint when no session exists: {json}"
    );
    assert_eq!(
        json.pointer("/endpoint_template").and_then(Value::as_str),
        Some(expected_template.as_str()),
        "UDS status must expose the deterministic endpoint template: {json}"
    );
    assert_eq!(
        json.pointer("/orchestrator/agent_id")
            .and_then(Value::as_str),
        Some("claude_code")
    );
    assert_eq!(
        json.pointer("/orchestrator/backend_id")
            .and_then(Value::as_str),
        Some("cli:claude_code")
    );
    assert_eq!(
        json.pointer("/orchestrator/role").and_then(Value::as_str),
        Some("orchestrator")
    );
    assert_eq!(
        json.pointer("/orchestrator/execution/scope")
            .and_then(Value::as_str),
        Some("host")
    );
    assert!(
        json.get("active_world_binding").is_none(),
        "toolbox status must omit active_world_binding when no active session is live: {json}"
    );
}

#[test]
fn agent_toolbox_env_requires_an_active_orchestrator_session() {
    let fixture = AgentSuccessorFixture::new();
    fixture.init_workspace();
    fixture.seed_inventory_for_toolbox_contracts("uds");

    let output = fixture.run(&["agent", "toolbox", "env", "--json"]);
    assert_eq!(
        output.status.code(),
        Some(3),
        "toolbox env must fail with dependency-unavailable when no orchestrator session is active: {output:?}"
    );
    assert!(
        String::from_utf8_lossy(&output.stdout).trim().is_empty(),
        "failed toolbox env commands must not emit stdout: {output:?}"
    );

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("no live host-scoped orchestrator participant found"),
        "stderr must explain the missing active session: {stderr}"
    );
}

#[test]
fn agent_toolbox_env_trace_history_does_not_authorize_active_session() {
    let fixture = AgentSuccessorFixture::new();
    fixture.init_workspace();
    fixture.seed_inventory_for_toolbox_contracts("uds");
    fixture.write_trace_events(&[
        json!({
            "ts": "2026-04-06T00:00:00Z",
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
            "ts": "2026-04-06T00:00:01Z",
            "event_type": "agent_event",
            "session_id": "ses_agent_hub",
            "component": "agent-hub",
            "kind": "status",
            "agent_id": "claude_code",
            "orchestration_session_id": "0195f8f1-7a34-7b7f-9c4d-9a7c2f5d6f99",
            "run_id": "0195f8f1-7a35-7b7f-9c4d-9a7c2f5d6f14",
            "backend_id": "cli:claude_code",
            "client": "claude_code",
            "router": "agent_hub",
            "protocol": "uaa.agent.session",
            "role": "orchestrator",
            "data": { "message": "newer orchestrator session is live" }
        }),
    ]);

    let output = fixture.run(&["agent", "toolbox", "env", "--json"]);
    assert_eq!(
        output.status.code(),
        Some(3),
        "trace history must not authorize an active toolbox session: {output:?}"
    );

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("no live host-scoped orchestrator participant found"),
        "trace-only active history must fail closed at the authoritative live manifest boundary: {stderr}"
    );
}

#[test]
fn agent_toolbox_env_prefers_live_manifest_over_trace_fallback() {
    let fixture = AgentSuccessorFixture::new();
    fixture.init_workspace();
    fixture.seed_inventory_for_toolbox_contracts("uds");
    write_live_runtime_manifest(
        &fixture,
        "claude_code",
        "0195f8f1-7a34-7b7f-9c4d-9a7c2f5d6faa",
        "ash_live_toolbox",
        "2026-04-06T00:00:02Z",
    );
    write_active_orchestration_session(
        &fixture,
        "claude_code",
        "0195f8f1-7a34-7b7f-9c4d-9a7c2f5d6faa",
        Some("ash_live_toolbox"),
        "2026-04-06T00:00:02Z",
    );
    fixture.write_trace_events(&[json!({
        "ts": "2026-04-06T00:00:03Z",
        "event_type": "agent_event",
        "session_id": "ses_agent_hub",
        "component": "agent-hub",
        "kind": "status",
        "agent_id": "claude_code",
        "orchestration_session_id": "0195f8f1-7a34-7b7f-9c4d-9a7c2f5d6fab",
        "run_id": "0195f8f1-7a35-7b7f-9c4d-9a7c2f5d6fac",
        "backend_id": "cli:claude_code",
        "client": "claude_code",
        "router": "agent_hub",
        "protocol": "uaa.agent.session",
        "role": "orchestrator",
        "data": { "message": "trace fallback should lose to live manifest" }
    })]);

    let output = fixture.run(&["agent", "toolbox", "env", "--json"]);
    assert!(
        output.status.success(),
        "toolbox env should prefer the live manifest: {output:?}"
    );

    let json = parse_json_output(&output);
    let expected = format!(
        "unix://{}/run/agent-toolbox/0195f8f1-7a34-7b7f-9c4d-9a7c2f5d6faa.sock",
        fixture.substrate_home.display()
    );
    assert_eq!(
        json.pointer("/SUBSTRATE_AGENT_TOOLBOX_ENDPOINT")
            .and_then(Value::as_str),
        Some(expected.as_str()),
        "toolbox env must use the authoritative live manifest and ignore trace history for active-session authorization: {json}"
    );
}

#[test]
fn operator_surfaces_fail_closed_when_live_orchestrator_child_has_no_parent_session() {
    let fixture = AgentSuccessorFixture::new();
    fixture.init_workspace();
    fixture.seed_inventory_for_toolbox_contracts("uds");
    write_live_runtime_manifest(
        &fixture,
        "claude_code",
        "0195f8f1-7a34-7b7f-9c4d-9a7c2f5d6fac",
        "ash_missing_parent",
        "2026-04-06T00:00:02Z",
    );

    assert_parent_resolution_fail_closed_across_operator_surfaces(
        &fixture,
        &["live host-scoped orchestrator participant exists for agent claude_code without an active parent session"],
    );
}

#[test]
fn operator_surfaces_fail_closed_when_live_orchestrator_parent_is_inactive() {
    let fixture = AgentSuccessorFixture::new();
    fixture.init_workspace();
    fixture.seed_inventory_for_toolbox_contracts("uds");
    write_live_runtime_manifest(
        &fixture,
        "claude_code",
        "0195f8f1-7a34-7b7f-9c4d-9a7c2f5d6fad",
        "ash_inactive_parent",
        "2026-04-06T00:00:02Z",
    );
    write_inactive_orchestration_session(
        &fixture,
        "claude_code",
        "0195f8f1-7a34-7b7f-9c4d-9a7c2f5d6fad",
        Some("ash_inactive_parent"),
        "2026-04-06T00:00:02Z",
    );

    assert_parent_resolution_fail_closed_across_operator_surfaces(
        &fixture,
        &["live host-scoped orchestrator participant exists for agent claude_code without an active parent session"],
    );
}

#[test]
fn operator_surfaces_fail_closed_when_active_parent_omits_active_session_handle_id() {
    let fixture = AgentSuccessorFixture::new();
    fixture.init_workspace();
    fixture.seed_inventory_for_toolbox_contracts("uds");
    write_live_runtime_manifest(
        &fixture,
        "claude_code",
        "0195f8f1-7a34-7b7f-9c4d-9a7c2f5d6fae",
        "ash_missing_active_handle",
        "2026-04-06T00:00:02Z",
    );
    write_active_orchestration_session(
        &fixture,
        "claude_code",
        "0195f8f1-7a34-7b7f-9c4d-9a7c2f5d6fae",
        None,
        "2026-04-06T00:00:02Z",
    );

    assert_parent_resolution_fail_closed_across_operator_surfaces(
        &fixture,
        &[
            "active orchestration session 0195f8f1-7a34-7b7f-9c4d-9a7c2f5d6fae is missing active_session_handle_id",
        ],
    );
}

#[test]
fn operator_surfaces_fail_closed_when_active_parent_points_to_different_live_handle() {
    let fixture = AgentSuccessorFixture::new();
    fixture.init_workspace();
    fixture.seed_inventory_for_toolbox_contracts("uds");
    write_live_runtime_manifest(
        &fixture,
        "claude_code",
        "0195f8f1-7a34-7b7f-9c4d-9a7c2f5d6faf",
        "ash_live_handle",
        "2026-04-06T00:00:02Z",
    );
    write_active_orchestration_session(
        &fixture,
        "claude_code",
        "0195f8f1-7a34-7b7f-9c4d-9a7c2f5d6faf",
        Some("ash_other_handle"),
        "2026-04-06T00:00:02Z",
    );

    assert_parent_resolution_fail_closed_across_operator_surfaces(
        &fixture,
        &[
            "active orchestration session 0195f8f1-7a34-7b7f-9c4d-9a7c2f5d6faf references missing participant ash_other_handle",
        ],
    );
}

#[test]
fn operator_surfaces_fail_closed_when_active_parent_selects_inactive_participant() {
    let fixture = AgentSuccessorFixture::new();
    fixture.init_workspace();
    fixture.seed_inventory_for_toolbox_contracts("uds");
    write_invalidated_runtime_manifest(
        &fixture,
        "claude_code",
        "0195f8f1-7a34-7b7f-9c4d-9a7c2f5d6fb0",
        "ash_inactive_selected",
        "2026-04-06T00:00:02Z",
    );
    write_active_orchestration_session(
        &fixture,
        "claude_code",
        "0195f8f1-7a34-7b7f-9c4d-9a7c2f5d6fb0",
        Some("ash_inactive_selected"),
        "2026-04-06T00:00:02Z",
    );

    assert_parent_resolution_fail_closed_across_operator_surfaces(
        &fixture,
        &[ "active orchestration session 0195f8f1-7a34-7b7f-9c4d-9a7c2f5d6fb0 references inactive participant ash_inactive_selected" ],
    );
}

#[test]
fn operator_surfaces_fail_closed_when_multiple_active_parent_candidates_exist() {
    let fixture = AgentSuccessorFixture::new();
    fixture.init_workspace();
    fixture.seed_inventory_for_toolbox_contracts("uds");
    write_active_orchestration_session(
        &fixture,
        "claude_code",
        "0195f8f1-7a34-7b7f-9c4d-9a7c2f5d6fb0",
        Some("ash_parent_one"),
        "2026-04-06T00:00:02Z",
    );
    write_active_orchestration_session(
        &fixture,
        "claude_code",
        "0195f8f1-7a34-7b7f-9c4d-9a7c2f5d6fb1",
        Some("ash_parent_two"),
        "2026-04-06T00:00:03Z",
    );

    assert_parent_resolution_fail_closed_across_operator_surfaces(
        &fixture,
        &["multiple active orchestration session candidates found for agent claude_code"],
    );
}

#[test]
fn agent_toolbox_env_invalidated_manifest_and_trace_still_fail_closed() {
    let fixture = AgentSuccessorFixture::new();
    fixture.init_workspace();
    fixture.seed_inventory_for_toolbox_contracts("uds");
    write_invalidated_runtime_manifest(
        &fixture,
        "claude_code",
        "0195f8f1-7a34-7b7f-9c4d-9a7c2f5d6faa",
        "ash_invalidated_toolbox",
        "2026-04-06T00:00:02Z",
    );
    fixture.write_trace_events(&[json!({
        "ts": "2026-04-06T00:00:03Z",
        "event_type": "agent_event",
        "session_id": "ses_agent_hub",
        "component": "agent-hub",
        "kind": "status",
        "agent_id": "claude_code",
        "orchestration_session_id": "0195f8f1-7a34-7b7f-9c4d-9a7c2f5d6fab",
        "run_id": "0195f8f1-7a35-7b7f-9c4d-9a7c2f5d6fac",
        "backend_id": "cli:claude_code",
        "client": "claude_code",
        "router": "agent_hub",
        "protocol": "uaa.agent.session",
        "role": "orchestrator",
        "data": { "message": "historical trace must not resurrect an invalidated session" }
    })]);

    let output = fixture.run(&["agent", "toolbox", "env", "--json"]);
    assert_eq!(
        output.status.code(),
        Some(3),
        "invalidated manifests must not be resurrected by trace fallback: {output:?}"
    );
    assert!(
        String::from_utf8_lossy(&output.stderr)
            .contains("no live host-scoped orchestrator participant found"),
        "invalidated manifests must keep toolbox env behind the authoritative live-manifest contract"
    );
}

#[test]
fn agent_toolbox_status_json_omits_active_world_binding_non_fatally_for_live_host_session() {
    let fixture = AgentSuccessorFixture::new();
    fixture.init_workspace();
    fixture.seed_inventory_for_toolbox_contracts("uds");
    write_live_runtime_manifest(
        &fixture,
        "claude_code",
        "0195f8f1-7a34-7b7f-9c4d-9a7c2f5d6fb2",
        "ash_live_no_binding",
        "2026-04-06T00:00:02Z",
    );
    write_active_orchestration_session(
        &fixture,
        "claude_code",
        "0195f8f1-7a34-7b7f-9c4d-9a7c2f5d6fb2",
        Some("ash_live_no_binding"),
        "2026-04-06T00:00:02Z",
    );

    let output = fixture.run(&["agent", "toolbox", "status", "--json"]);
    assert!(
        output.status.success(),
        "toolbox status should remain readable when the live host session has no binding proof: {output:?}"
    );

    let json = parse_json_output(&output);
    let expected = format!(
        "unix://{}/run/agent-toolbox/0195f8f1-7a34-7b7f-9c4d-9a7c2f5d6fb2.sock",
        fixture.substrate_home.display()
    );
    assert_eq!(
        json.pointer("/eligibility/state").and_then(Value::as_str),
        Some("allowed")
    );
    assert_eq!(
        json.pointer("/active_orchestration_session_id")
            .and_then(Value::as_str),
        Some("0195f8f1-7a34-7b7f-9c4d-9a7c2f5d6fb2")
    );
    assert_eq!(
        json.pointer("/endpoint").and_then(Value::as_str),
        Some(expected.as_str())
    );
    assert!(
        json.get("active_world_binding").is_none(),
        "toolbox status must keep active_world_binding optional for live host sessions without projected binding state: {json}"
    );
}

#[test]
fn agent_toolbox_status_json_reports_active_world_binding_from_live_parent_session() {
    let fixture = AgentSuccessorFixture::new();
    fixture.init_workspace();
    fixture.seed_inventory_for_toolbox_contracts("uds");
    write_live_runtime_manifest(
        &fixture,
        "claude_code",
        "0195f8f1-7a34-7b7f-9c4d-9a7c2f5d6fb3",
        "ash_live_with_binding",
        "2026-04-06T00:00:02Z",
    );
    write_active_orchestration_session_with_world_binding(
        &fixture,
        "claude_code",
        "0195f8f1-7a34-7b7f-9c4d-9a7c2f5d6fb3",
        Some("ash_live_with_binding"),
        "2026-04-06T00:00:02Z",
        "wld_active_0002",
        7,
    );

    let toolbox_output = fixture.run(&["agent", "toolbox", "status", "--json"]);
    assert!(
        toolbox_output.status.success(),
        "toolbox status should surface the optional active binding proof without changing transport semantics: {toolbox_output:?}"
    );

    let toolbox_json = parse_json_output(&toolbox_output);
    assert_eq!(
        toolbox_json
            .pointer("/active_world_binding/world_id")
            .and_then(Value::as_str),
        Some("wld_active_0002"),
        "toolbox status must publish the active world_id proof from the authoritative parent session: {toolbox_json}"
    );
    assert_eq!(
        toolbox_json
            .pointer("/active_world_binding/world_generation")
            .and_then(Value::as_u64),
        Some(7),
        "toolbox status must publish the active world_generation proof from the authoritative parent session: {toolbox_json}"
    );
}

#[test]
fn agent_toolbox_surfaces_prefer_canonical_session_roots_over_flat_compatibility_files() {
    let fixture = AgentSuccessorFixture::new();
    fixture.init_workspace();
    fixture.seed_inventory_for_toolbox_contracts("uds");
    write_live_runtime_manifest(
        &fixture,
        "claude_code",
        "0195f8f1-7a34-7b7f-9c4d-9a7c2f5d6fb4",
        "ash_canonical_toolbox",
        "2026-04-06T00:00:02Z",
    );
    write_active_orchestration_session(
        &fixture,
        "claude_code",
        "0195f8f1-7a34-7b7f-9c4d-9a7c2f5d6fb4",
        Some("ash_canonical_toolbox"),
        "2026-04-06T00:00:02Z",
    );
    write_flat_runtime_participant_compatibility(
        &fixture,
        "ash_canonical_toolbox",
        "claude_code",
        "0195f8f1-7a34-7b7f-9c4d-9a7c2f5d6f99",
        RuntimeParticipantOptions::host_orchestrator("invalidated", false, "2026-04-06T00:00:01Z"),
    );
    write_flat_orchestration_session_compatibility(
        &fixture,
        "claude_code",
        "0195f8f1-7a34-7b7f-9c4d-9a7c2f5d6fb4",
        Some("ash_flat_conflict"),
        "stopped",
        "2026-04-06T00:00:01Z",
        None,
    );

    let status_output = fixture.run(&["agent", "toolbox", "status", "--json"]);
    assert!(
        status_output.status.success(),
        "toolbox status must prefer the canonical session-root record over conflicting flat compatibility files: {status_output:?}"
    );
    let status_json = parse_json_output(&status_output);
    assert_eq!(
        status_json
            .pointer("/active_orchestration_session_id")
            .and_then(Value::as_str),
        Some("0195f8f1-7a34-7b7f-9c4d-9a7c2f5d6fb4")
    );

    let env_output = fixture.run(&["agent", "toolbox", "env", "--json"]);
    assert!(
        env_output.status.success(),
        "toolbox env must resolve from the same canonical session-root record chosen by toolbox status: {env_output:?}"
    );
    let env_json = parse_json_output(&env_output);
    let expected_endpoint = format!(
        "unix://{}/run/agent-toolbox/0195f8f1-7a34-7b7f-9c4d-9a7c2f5d6fb4.sock",
        fixture.substrate_home.display()
    );
    assert_eq!(
        env_json
            .pointer("/SUBSTRATE_AGENT_TOOLBOX_ENDPOINT")
            .and_then(Value::as_str),
        Some(expected_endpoint.as_str()),
        "canonical participant and parent records must outrank conflicting flat compatibility files on both toolbox surfaces: {env_json}"
    );
}

#[test]
fn agent_toolbox_surfaces_fall_back_to_flat_participant_when_canonical_root_is_incomplete() {
    let fixture = AgentSuccessorFixture::new();
    fixture.init_workspace();
    fixture.seed_inventory_for_toolbox_contracts("uds");
    write_active_orchestration_session(
        &fixture,
        "claude_code",
        "0195f8f1-7a34-7b7f-9c4d-9a7c2f5d6fb5",
        Some("ash_flat_fallback"),
        "2026-04-06T00:00:02Z",
    );
    write_flat_runtime_participant_compatibility(
        &fixture,
        "ash_flat_fallback",
        "claude_code",
        "0195f8f1-7a34-7b7f-9c4d-9a7c2f5d6fb5",
        RuntimeParticipantOptions::host_orchestrator("running", true, "2026-04-06T00:00:02Z"),
    );

    let status_output = fixture.run(&["agent", "toolbox", "status", "--json"]);
    assert!(
        status_output.status.success(),
        "toolbox status must keep the flat compatibility participant fallback when the canonical session root is parent-only: {status_output:?}"
    );
    let status_json = parse_json_output(&status_output);
    assert_eq!(
        status_json
            .pointer("/active_orchestration_session_id")
            .and_then(Value::as_str),
        Some("0195f8f1-7a34-7b7f-9c4d-9a7c2f5d6fb5")
    );

    let env_output = fixture.run(&["agent", "toolbox", "env", "--json"]);
    assert!(
        env_output.status.success(),
        "toolbox env must share the same flat compatibility fallback when the canonical participant is missing: {env_output:?}"
    );
    let env_json = parse_json_output(&env_output);
    let expected_endpoint = format!(
        "unix://{}/run/agent-toolbox/0195f8f1-7a34-7b7f-9c4d-9a7c2f5d6fb5.sock",
        fixture.substrate_home.display()
    );
    assert_eq!(
        env_json
            .pointer("/SUBSTRATE_AGENT_TOOLBOX_ENDPOINT")
            .and_then(Value::as_str),
        Some(expected_endpoint.as_str()),
        "flat compatibility participant fallback must stay readable when the canonical root is incomplete: {env_json}"
    );
}

#[test]
fn agent_status_selected_host_row_stays_unchanged_when_parent_session_has_world_binding() {
    let fixture = AgentSuccessorFixture::new();
    fixture.init_workspace();
    fixture.seed_inventory_for_toolbox_contracts("uds");
    write_live_runtime_manifest(
        &fixture,
        "claude_code",
        "0195f8f1-7a34-7b7f-9c4d-9a7c2f5d6fb3",
        "ash_live_with_binding",
        "2026-04-06T00:00:02Z",
    );
    write_active_orchestration_session_with_world_binding(
        &fixture,
        "claude_code",
        "0195f8f1-7a34-7b7f-9c4d-9a7c2f5d6fb3",
        Some("ash_live_with_binding"),
        "2026-04-06T00:00:02Z",
        "wld_active_0002",
        7,
    );

    let status_output = fixture.run(&["agent", "status", "--json"]);
    assert!(
        status_output.status.success(),
        "selected host status rows must remain readable when the parent session has world binding state: {status_output:?}"
    );

    let status_json = parse_json_output(&status_output);
    let sessions = status_json["sessions"]
        .as_array()
        .expect("sessions should be an array");
    let orchestrator = find_session_by_agent(sessions, "claude_code");
    assert_eq!(
        orchestrator.pointer("/execution/scope").and_then(Value::as_str),
        Some("host"),
        "selected orchestrator rows must stay host-scoped even when the parent session carries world binding state: {status_json}"
    );
    assert!(
        orchestrator.get("world_id").is_none(),
        "selected host status rows must keep omitting world_id: {status_json}"
    );
    assert!(
        orchestrator.get("world_generation").is_none(),
        "selected host status rows must keep omitting world_generation: {status_json}"
    );
}

#[test]
fn agent_toolbox_status_json_reports_tcp_as_unsupported_pre_runtime() {
    let fixture = AgentSuccessorFixture::new();
    fixture.init_workspace();
    fixture.seed_inventory_for_toolbox_contracts("tcp");

    let output = fixture.run(&["agent", "toolbox", "status", "--json"]);
    assert!(
        output.status.success(),
        "toolbox status should stay readable when tcp transport is selected: {output:?}"
    );

    let json = parse_json_output(&output);
    assert_eq!(
        json.pointer("/transport").and_then(Value::as_str),
        Some("tcp")
    );
    assert_eq!(
        json.pointer("/eligibility/state").and_then(Value::as_str),
        Some("unsupported")
    );
    let reason = json
        .pointer("/eligibility/reason")
        .and_then(Value::as_str)
        .expect("unsupported tcp posture must publish a reason");
    assert!(
        reason.contains("deterministic pre-runtime loopback port contract"),
        "unsupported tcp status must explain the pre-runtime port-allocation gap: {json}"
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
fn agent_status_prefers_live_manifest_over_trace_fallback_for_selected_orchestrator() {
    let fixture = AgentSuccessorFixture::new();
    fixture.init_workspace();
    fixture.seed_inventory_for_list_and_status_contracts();
    write_live_runtime_manifest(
        &fixture,
        "claude_code",
        "0195f8f1-7a34-7b7f-9c4d-9a7c2f5d6faa",
        "ash_live_status",
        "2026-04-05T00:00:02Z",
    );
    write_active_orchestration_session(
        &fixture,
        "claude_code",
        "0195f8f1-7a34-7b7f-9c4d-9a7c2f5d6faa",
        Some("ash_live_status"),
        "2026-04-05T00:00:02Z",
    );
    fixture.write_trace_events(&[json!({
        "ts": "2026-04-05T00:00:03Z",
        "event_type": "agent_event",
        "session_id": "ses_agent_hub",
        "component": "agent-hub",
        "kind": "status",
        "agent_id": "claude_code",
        "orchestration_session_id": "0195f8f1-7a34-7b7f-9c4d-9a7c2f5d6fab",
        "run_id": "0195f8f1-7a35-7b7f-9c4d-9a7c2f5d6fac",
        "backend_id": "cli:claude_code",
        "client": "claude_code",
        "router": "agent_hub",
        "protocol": "uaa.agent.session",
        "role": "orchestrator",
        "data": { "message": "trace fallback should lose to live manifest" }
    })]);

    let output = fixture.run(&["agent", "status", "--json"]);
    assert!(
        output.status.success(),
        "status should prefer the live manifest: {output:?}"
    );

    let json = parse_json_output(&output);
    let sessions = json["sessions"]
        .as_array()
        .expect("sessions should be an array");
    let orchestrator = find_session_by_agent(sessions, "claude_code");
    assert_eq!(
        orchestrator
            .pointer("/orchestration_session_id")
            .and_then(Value::as_str),
        Some("0195f8f1-7a34-7b7f-9c4d-9a7c2f5d6faa"),
        "status must prefer live manifest state over trace fallback for the selected orchestrator: {json}"
    );
}

#[test]
fn agent_status_tombstone_suppression_beats_stale_trace_fallback_for_world_member() {
    let fixture = AgentSuccessorFixture::new();
    fixture.init_workspace();
    fixture.seed_inventory_for_list_and_status_contracts();
    write_invalidated_world_member_manifest(
        &fixture,
        "codex",
        "0195f8f1-7a34-7b7f-9c4d-9a7c2f5d6fb0",
        "ash_codex_old",
        "ash_orchestrator",
        "wld_old_0001",
        6,
        None,
        "2026-04-05T00:00:02Z",
    );
    fixture.write_trace_events(&[json!({
        "ts": "2026-04-05T00:00:03Z",
        "event_type": "agent_event",
        "session_id": "ses_agent_hub",
        "component": "agent-hub",
        "kind": "status",
        "agent_id": "codex",
        "orchestration_session_id": "0195f8f1-7a34-7b7f-9c4d-9a7c2f5d6fb0",
        "run_id": "0195f8f1-7a35-7b7f-9c4d-9a7c2f5d6fb1",
        "backend_id": "cli:codex",
        "client": "codex",
        "router": "agent_hub",
        "protocol": "uaa.agent.session",
        "role": "member",
        "world_id": "wld_old_0001",
        "world_generation": 6,
        "data": { "message": "stale trace row must not beat the authoritative participant tombstone" }
    })]);

    let output = fixture.run(&["agent", "status", "--scope", "world", "--json"]);
    assert!(
        output.status.success(),
        "world status should succeed when tombstones suppress stale trace fallback: {output:?}"
    );

    let json = parse_json_output(&output);
    let sessions = json["sessions"]
        .as_array()
        .expect("sessions should be an array");
    assert!(
        sessions.is_empty(),
        "authoritative participant tombstones must suppress stale trace fallback for the same (orchestration_session_id, agent_id, execution.scope) tuple: {json}"
    );
}

#[test]
fn agent_status_omits_invalidated_world_member_until_replacement_persists() {
    let fixture = AgentSuccessorFixture::new();
    fixture.init_workspace();
    fixture.seed_inventory_for_list_and_status_contracts();
    write_live_runtime_manifest(
        &fixture,
        "claude_code",
        "0195f8f1-7a34-7b7f-9c4d-9a7c2f5d6fb2",
        "ash_orchestrator_live",
        "2026-04-05T00:00:01Z",
    );
    write_active_orchestration_session(
        &fixture,
        "claude_code",
        "0195f8f1-7a34-7b7f-9c4d-9a7c2f5d6fb2",
        Some("ash_orchestrator_live"),
        "2026-04-05T00:00:01Z",
    );
    write_invalidated_world_member_manifest(
        &fixture,
        "codex",
        "0195f8f1-7a34-7b7f-9c4d-9a7c2f5d6fb2",
        "ash_codex_missing_replacement",
        "ash_orchestrator_live",
        "wld_old_0002",
        6,
        None,
        "2026-04-05T00:00:02Z",
    );

    let output = fixture.run(&["agent", "status", "--json"]);
    assert!(
        output.status.success(),
        "status should succeed when a replacement participant has not been persisted yet: {output:?}"
    );

    let json = parse_json_output(&output);
    let sessions = json["sessions"]
        .as_array()
        .expect("sessions should be an array");
    assert_eq!(
        sessions.len(),
        1,
        "missing replacements must leave omission rather than stale liveness on the selected status surface: {json}"
    );
    let orchestrator = find_session_by_agent(sessions, "claude_code");
    assert_eq!(
        orchestrator
            .pointer("/participant_id")
            .and_then(Value::as_str),
        Some("ash_orchestrator_live")
    );
    assert!(
        sessions
            .iter()
            .all(|session| session.pointer("/agent_id").and_then(Value::as_str) != Some("codex")),
        "invalidated members must stay absent until a replacement participant is persisted: {json}"
    );
}

#[test]
fn agent_status_keeps_same_agent_concurrent_sessions_visible_across_orchestration_sessions() {
    let fixture = AgentSuccessorFixture::new();
    fixture.init_workspace();
    fixture.seed_inventory_for_list_and_status_contracts();
    write_live_runtime_manifest(
        &fixture,
        "claude_code",
        "0195f8f1-7a34-7b7f-9c4d-9a7c2f5d6faa",
        "ash_live_status_suppression",
        "2026-04-05T00:00:02Z",
    );
    write_active_orchestration_session(
        &fixture,
        "claude_code",
        "0195f8f1-7a34-7b7f-9c4d-9a7c2f5d6faa",
        Some("ash_live_status_suppression"),
        "2026-04-05T00:00:02Z",
    );
    fixture.write_trace_events(&[json!({
        "ts": "2026-04-05T00:00:01Z",
        "event_type": "agent_event",
        "session_id": "ses_agent_hub",
        "component": "agent-hub",
        "kind": "status",
        "agent_id": "claude_code",
        "orchestration_session_id": "0195f8f1-7a34-7b7f-9c4d-9a7c2f5d6fab",
        "run_id": "0195f8f1-7a35-7b7f-9c4d-9a7c2f5d6fac",
        "backend_id": "cli:claude_code",
        "client": "claude_code",
        "router": "agent_hub",
        "protocol": "uaa.agent.session",
        "role": "orchestrator",
        "data": { "message": "distinct orchestration_session_id values must remain independently visible" }
    })]);

    let output = fixture.run(&["agent", "status", "--json"]);
    assert!(
        output.status.success(),
        "status should succeed when the same agent is visible in concurrent sessions: {output:?}"
    );

    let json = parse_json_output(&output);
    let sessions = json["sessions"]
        .as_array()
        .expect("sessions should be an array");
    let claude_rows: Vec<&Value> = sessions
        .iter()
        .filter(|session| {
            session.pointer("/agent_id").and_then(Value::as_str) == Some("claude_code")
        })
        .collect();
    assert_eq!(
        claude_rows.len(),
        2,
        "suppression identity must stay scoped to (orchestration_session_id, agent_id, execution.scope) so same-agent concurrent sessions remain independently visible: {json}"
    );
    assert_eq!(
        find_session_by_agent_and_orchestration_session(
            sessions,
            "claude_code",
            "0195f8f1-7a34-7b7f-9c4d-9a7c2f5d6faa",
        )
        .pointer("/orchestration_session_id")
        .and_then(Value::as_str),
        Some("0195f8f1-7a34-7b7f-9c4d-9a7c2f5d6faa")
    );
    assert_eq!(
        find_session_by_agent_and_orchestration_session(
            sessions,
            "claude_code",
            "0195f8f1-7a34-7b7f-9c4d-9a7c2f5d6fab",
        )
        .pointer("/orchestration_session_id")
        .and_then(Value::as_str),
        Some("0195f8f1-7a34-7b7f-9c4d-9a7c2f5d6fab")
    );
}

#[test]
fn agent_status_keeps_same_agent_concurrent_live_sessions_visible_from_session_records() {
    let fixture = AgentSuccessorFixture::new();
    fixture.init_workspace();
    fixture.seed_inventory_for_list_and_status_contracts();
    fixture.write_global_policy_patch(
        r#"agents:
  allowed_backends:
    - cli:claude_code
    - cli:codex
    - cli:helper
"#,
    );
    fixture.write_agent_file(
        "helper.yaml",
        &cli_agent_file("helper", "host", true, false, true),
    );
    write_live_runtime_manifest(
        &fixture,
        "claude_code",
        "0195f8f1-7a34-7b7f-9c4d-9a7c2f5d6fc1",
        "ash_orchestrator_session_one",
        "2026-04-05T00:00:02Z",
    );
    write_active_orchestration_session(
        &fixture,
        "claude_code",
        "0195f8f1-7a34-7b7f-9c4d-9a7c2f5d6fc1",
        Some("ash_orchestrator_session_one"),
        "2026-04-05T00:00:02Z",
    );
    write_runtime_participant(
        &fixture,
        "ash_codex_session_one",
        "codex",
        "0195f8f1-7a34-7b7f-9c4d-9a7c2f5d6fc1",
        RuntimeParticipantOptions::world_member(
            "running",
            true,
            "2026-04-05T00:00:02Z",
            "wld_live_member_0001",
            7,
            "ash_orchestrator_session_one",
        ),
    );
    write_live_runtime_manifest(
        &fixture,
        "helper",
        "0195f8f1-7a34-7b7f-9c4d-9a7c2f5d6fc2",
        "ash_orchestrator_session_two",
        "2026-04-05T00:00:03Z",
    );
    write_active_orchestration_session(
        &fixture,
        "helper",
        "0195f8f1-7a34-7b7f-9c4d-9a7c2f5d6fc2",
        Some("ash_orchestrator_session_two"),
        "2026-04-05T00:00:03Z",
    );
    write_runtime_participant(
        &fixture,
        "ash_codex_session_two",
        "codex",
        "0195f8f1-7a34-7b7f-9c4d-9a7c2f5d6fc2",
        RuntimeParticipantOptions::world_member(
            "running",
            true,
            "2026-04-05T00:00:03Z",
            "wld_live_member_0002",
            8,
            "ash_orchestrator_session_two",
        ),
    );

    let output = fixture.run(&["agent", "status", "--scope", "world", "--json"]);
    assert!(
        output.status.success(),
        "agent status must keep same-agent concurrent live sessions visible when both rows come from store-owned session records: {output:?}"
    );

    let json = parse_json_output(&output);
    let sessions = json["sessions"]
        .as_array()
        .expect("sessions should be an array");
    let codex_rows: Vec<&Value> = sessions
        .iter()
        .filter(|session| session.pointer("/agent_id").and_then(Value::as_str) == Some("codex"))
        .collect();
    assert_eq!(
        codex_rows.len(),
        2,
        "same-agent concurrent visibility must survive the session-record regrouping cutover: {json}"
    );
    assert_eq!(
        find_session_by_agent_and_orchestration_session(
            sessions,
            "codex",
            "0195f8f1-7a34-7b7f-9c4d-9a7c2f5d6fc1",
        )
        .pointer("/participant_id")
        .and_then(Value::as_str),
        Some("ash_codex_session_one")
    );
    assert_eq!(
        find_session_by_agent_and_orchestration_session(
            sessions,
            "codex",
            "0195f8f1-7a34-7b7f-9c4d-9a7c2f5d6fc2",
        )
        .pointer("/participant_id")
        .and_then(Value::as_str),
        Some("ash_codex_session_two")
    );
}

#[test]
fn agent_status_persists_resumed_from_participant_id_for_replacement_members() {
    let fixture = AgentSuccessorFixture::new();
    fixture.init_workspace();
    fixture.seed_inventory_for_list_and_status_contracts();
    write_live_runtime_manifest(
        &fixture,
        "claude_code",
        "0195f8f1-7a34-7b7f-9c4d-9a7c2f5d6fb3",
        "ash_orchestrator_lineage",
        "2026-04-05T00:00:01Z",
    );
    write_active_orchestration_session(
        &fixture,
        "claude_code",
        "0195f8f1-7a34-7b7f-9c4d-9a7c2f5d6fb3",
        Some("ash_orchestrator_lineage"),
        "2026-04-05T00:00:01Z",
    );
    write_invalidated_world_member_manifest(
        &fixture,
        "codex",
        "0195f8f1-7a34-7b7f-9c4d-9a7c2f5d6fb3",
        "ash_codex_old_lineage",
        "ash_orchestrator_lineage",
        "wld_old_0003",
        6,
        None,
        "2026-04-05T00:00:02Z",
    );
    write_replacement_world_member_manifest(
        &fixture,
        "codex",
        "0195f8f1-7a34-7b7f-9c4d-9a7c2f5d6fb3",
        "ash_codex_new_lineage",
        "ash_orchestrator_lineage",
        "wld_new_0003",
        7,
        "ash_codex_old_lineage",
        "2026-04-05T00:00:03Z",
    );

    let output = fixture.run(&["agent", "status", "--scope", "world", "--json"]);
    assert!(
        output.status.success(),
        "world status should succeed for replacement lineage fixtures: {output:?}"
    );

    let json = parse_json_output(&output);
    let sessions = json["sessions"]
        .as_array()
        .expect("sessions should be an array");
    assert_eq!(
        sessions.len(),
        1,
        "replacement member should win world status selection: {json}"
    );
    let replacement = find_session_by_agent(sessions, "codex");
    assert_eq!(
        replacement
            .pointer("/participant_id")
            .and_then(Value::as_str),
        Some("ash_codex_new_lineage")
    );
    assert_eq!(
        replacement
            .pointer("/world_generation")
            .and_then(Value::as_u64),
        Some(7)
    );

    let persisted = serde_json::from_str::<Value>(
        &fs::read_to_string(participant_manifest_path(&fixture, "ash_codex_new_lineage"))
            .expect("replacement participant manifest should be readable"),
    )
    .expect("replacement participant manifest should be valid JSON");
    assert_eq!(
        persisted
            .pointer("/resumed_from_participant_id")
            .and_then(Value::as_str),
        Some("ash_codex_old_lineage"),
        "replacement participant persistence must keep resumed_from_participant_id as the lineage field name"
    );
    assert!(
        persisted.get("resumed_from_session_handle_id").is_none(),
        "replacement participant persistence must not revert to the legacy resumed_from_session_handle_id field name"
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
            "runtime_realizability",
            "participant_store",
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
        vec!["pass", "pass", "pass", "pass", "pass", "not_applicable"],
        "host-only doctor fixture should report a not_applicable world boundary after the four required passes: {json}"
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
            SessionContractOptions {
                protocol: None,
                capability_override: None,
                ..SessionContractOptions::default()
            },
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
            SessionContractOptions {
                protocol: Some("openai.responses"),
                capability_override: None,
                ..SessionContractOptions::default()
            },
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
            SessionContractOptions {
                protocol: Some(PURE_AGENT_PROTOCOL),
                capability_override: Some(CapabilityOverride::ForceFalse("event_stream")),
                ..SessionContractOptions::default()
            },
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
            SessionContractOptions {
                protocol: Some(PURE_AGENT_PROTOCOL),
                capability_override: Some(CapabilityOverride::Omit("event_stream")),
                ..SessionContractOptions::default()
            },
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
fn agent_doctor_fails_at_runtime_realizability_when_selected_binary_is_missing() {
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
            SessionContractOptions {
                binary: "definitely_missing_substrate_agent_binary",
                ..SessionContractOptions::default()
            },
        ),
    );

    let output = fixture.run(&["agent", "doctor", "--json"]);
    assert_eq!(
        output.status.code(),
        Some(4),
        "doctor should classify missing orchestrator binaries as host-prereq failures: {output:?}"
    );

    let json = parse_json_output(&output);
    let checks = json["checks"]
        .as_array()
        .expect("checks should be an array");
    let observed: Vec<&str> = checks
        .iter()
        .map(|check| check.pointer("/check").and_then(Value::as_str).unwrap())
        .collect();
    assert_eq!(
        observed,
        vec![
            "inventory_scan",
            "orchestrator_selection",
            "runtime_realizability",
        ],
        "doctor must stop at runtime_realizability before policy/world checks on binary resolution failures: {json}"
    );
    assert!(
        checks[2]
            .pointer("/reason")
            .and_then(Value::as_str)
            .is_some_and(|reason| reason.contains("did not resolve on the host")),
        "runtime_realizability must explain the missing binary: {json}"
    );
}

#[test]
fn agent_doctor_fails_at_runtime_realizability_when_selected_cli_mode_is_per_request() {
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
            SessionContractOptions {
                cli_mode: "per_request",
                ..SessionContractOptions::default()
            },
        ),
    );

    let output = fixture.run(&["agent", "doctor", "--json"]);
    assert_eq!(
        output.status.code(),
        Some(2),
        "doctor should classify unsupported cli.mode values as runtime contract failures: {output:?}"
    );

    let json = parse_json_output(&output);
    let checks = json["checks"]
        .as_array()
        .expect("checks should be an array");
    let observed: Vec<&str> = checks
        .iter()
        .map(|check| check.pointer("/check").and_then(Value::as_str).unwrap())
        .collect();
    assert_eq!(
        observed,
        vec![
            "inventory_scan",
            "orchestrator_selection",
            "runtime_realizability",
        ],
        "doctor must stop at runtime_realizability before policy/world checks on unsupported cli.mode values: {json}"
    );
    assert_eq!(
        checks[2].pointer("/reason").and_then(Value::as_str),
        Some(
            "selected orchestrator 'claude_code' is not runtime-realizable because cli.mode=per_request is unsupported; only cli.mode=persistent is supported for the first caller path"
        )
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
    assert!(
        usage.contains("substrate agent toolbox status"),
        "docs/USAGE.md must document the canonical singular toolbox status command"
    );
    assert!(
        usage.contains("substrate agent toolbox env"),
        "docs/USAGE.md must document the canonical singular toolbox env command"
    );
    for forbidden in [
        "substrate agents list",
        "substrate agents status",
        "substrate agents doctor",
        "live session discovery is backed by persisted manifests under `~/.substrate/run/agent-hub/handles/`",
    ] {
        assert!(
            !usage.contains(forbidden),
            "docs/USAGE.md must not advertise plural successor aliases: {forbidden}"
        );
    }
    for required in [
        "`~/.substrate/run/agent-hub/sessions/<orchestration_session_id>/session.json`",
        "`~/.substrate/run/agent-hub/sessions/<orchestration_session_id>/participants/<participant_id>.json`",
        "`~/.substrate/run/agent-hub/handles/*.json` remains compatibility input only",
    ] {
        assert!(
            usage.contains(required),
            "docs/USAGE.md must describe the session-root live-state authority boundary `{required}`"
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
        "`~/.substrate/run/agent-hub/sessions/<orchestration_session_id>/session.json`",
        "`~/.substrate/run/agent-hub/sessions/<orchestration_session_id>/participants/<participant_id>.json`",
        "`~/.substrate/run/agent-hub/sessions/<orchestration_session_id>.json`",
        "`~/.substrate/run/agent-hub/participants/*.json`",
        "`~/.substrate/run/agent-hub/handles/*.json` remains legacy compatibility input only",
        "Invalidated participant tombstones in canonical or flat compatibility participant records beat stale trace fallback rows",
    ] {
        assert!(
            trace.contains(required),
            "docs/TRACE.md must document tuple-compatible trace field `{required}`"
        );
    }
}

#[test]
fn production_runtime_snapshot_writes_remain_centralized_in_state_store() {
    let src_root = repo_root().join("crates/shell/src");
    let forbidden_markers = [
        "join(\"agent-hub\")",
        "join(\"handles\")",
        "join(\"participants\")",
        "join(\"leases\")",
        "handles_dir()",
        "participants_dir()",
        "sessions_dir()",
    ];

    let offenders = collect_rust_files(&src_root)
        .into_iter()
        .filter(|path| path.file_name().and_then(|name| name.to_str()) != Some("state_store.rs"))
        .filter_map(|path| {
            let source = read_production_shell_source_without_inline_tests(&path);
            let hits = forbidden_markers
                .iter()
                .copied()
                .filter(|marker| source.contains(marker))
                .collect::<Vec<_>>();
            (!hits.is_empty()).then_some(format!("{} => {}", path.display(), hits.join(", ")))
        })
        .collect::<Vec<_>>();

    assert!(
        offenders.is_empty(),
        "production callers outside state_store.rs must not reach flat compatibility parent/participant/lease or legacy handle paths directly: {}",
        offenders.join(" | ")
    );
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
        vec![
            "inventory_scan",
            "orchestrator_selection",
            "runtime_realizability",
            "participant_store",
            "policy_allowlist",
        ],
        "fail-closed routing must stop at the member backend allowlist before world_boundary: {json}"
    );
    assert_eq!(
        checks[4].pointer("/reason").and_then(Value::as_str),
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
fn agent_status_ignores_stale_nested_rows_from_historical_parent_runs() {
    let fixture = AgentSuccessorFixture::new();
    seed_nested_gateway_status_fixture(&fixture);
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
            "world_id": "wld_active_0001",
            "world_generation": 6,
            "data": { "message": "older pure-agent session is live" }
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
            "data": { "summary": "stale nested gateway request completed" }
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
            "backend_id": "cli:claude_code",
            "client": "claude_code",
            "router": "agent_hub",
            "protocol": "uaa.agent.session",
            "role": "orchestrator",
            "world_id": "wld_active_0002",
            "world_generation": 7,
            "data": { "message": "newest pure-agent session is live" }
        }),
        json!({
            "ts": "2026-04-05T00:00:03Z",
            "event_type": "agent_event",
            "session_id": "ses_agent_hub",
            "component": "agent-hub",
            "kind": "status",
            "agent_id": "claude_code",
            "orchestration_session_id": "0195f8f1-7a34-7b7f-9c4d-9a7c2f5d6f12",
            "run_id": "0195f8f1-7a35-7b7f-9c4d-9a7c2f5d6f16",
            "parent_run_id": "0195f8f1-7a35-7b7f-9c4d-9a7c2f5d6f15",
            "backend_id": "cli:claude_code",
            "client": "claude_code",
            "router": "substrate_gateway",
            "protocol": "openai.responses",
            "provider": "openai",
            "auth_authority": "codex_subscription",
            "data": { "summary": "current nested gateway request completed" }
        }),
    ]);

    let output = fixture.run(&["agent", "status", "--json"]);
    assert!(
        output.status.success(),
        "status should ignore stale nested rows tied to historical pure-agent runs: {output:?}"
    );

    let json = parse_json_output(&output);
    let nested = json["nested_llm_records"]
        .as_array()
        .expect("nested_llm_records should be an array");
    assert_eq!(
        nested.len(),
        1,
        "only nested rows for the winning selected parent run should remain: {json}"
    );
    assert_eq!(
        nested[0].pointer("/run_id").and_then(Value::as_str),
        Some("0195f8f1-7a35-7b7f-9c4d-9a7c2f5d6f16")
    );
}

#[test]
fn agent_status_fails_closed_when_selected_nested_row_omits_parent_run_id() {
    let fixture = AgentSuccessorFixture::new();
    seed_nested_gateway_status_fixture(&fixture);
    let orchestration_session_id = "0195f8f1-7a34-7b7f-9c4d-9a7c2f5d6f12";
    fixture.write_trace_events(&[
        json!({
            "ts": "2026-04-05T00:00:00Z",
            "event_type": "agent_event",
            "session_id": "ses_agent_hub",
            "component": "agent-hub",
            "kind": "status",
            "agent_id": "claude_code",
            "orchestration_session_id": orchestration_session_id,
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
            "orchestration_session_id": orchestration_session_id,
            "run_id": "0195f8f1-7a35-7b7f-9c4d-9a7c2f5d6f14",
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
    assert_malformed_nested_parent_correlation_failure(
        &output,
        "claude_code",
        orchestration_session_id,
        "0195f8f1-7a35-7b7f-9c4d-9a7c2f5d6f14",
        "<missing>",
    );
}

#[test]
fn agent_status_fails_closed_when_selected_nested_row_has_empty_parent_run_id() {
    let fixture = AgentSuccessorFixture::new();
    seed_nested_gateway_status_fixture(&fixture);
    let orchestration_session_id = "0195f8f1-7a34-7b7f-9c4d-9a7c2f5d6f12";
    fixture.write_trace_events(&[
        json!({
            "ts": "2026-04-05T00:00:00Z",
            "event_type": "agent_event",
            "session_id": "ses_agent_hub",
            "component": "agent-hub",
            "kind": "status",
            "agent_id": "claude_code",
            "orchestration_session_id": orchestration_session_id,
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
            "orchestration_session_id": orchestration_session_id,
            "run_id": "0195f8f1-7a35-7b7f-9c4d-9a7c2f5d6f14",
            "parent_run_id": "",
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
    assert_malformed_nested_parent_correlation_failure(
        &output,
        "claude_code",
        orchestration_session_id,
        "0195f8f1-7a35-7b7f-9c4d-9a7c2f5d6f14",
        "<empty>",
    );
}

#[test]
fn agent_status_fails_closed_when_selected_nested_row_has_unknown_parent_run_id() {
    let fixture = AgentSuccessorFixture::new();
    seed_nested_gateway_status_fixture(&fixture);
    let orchestration_session_id = "0195f8f1-7a34-7b7f-9c4d-9a7c2f5d6f12";
    let bad_parent_run_id = "0195f8f1-7a35-7b7f-9c4d-9a7c2f5d6faa";
    fixture.write_trace_events(&[
        json!({
            "ts": "2026-04-05T00:00:00Z",
            "event_type": "agent_event",
            "session_id": "ses_agent_hub",
            "component": "agent-hub",
            "kind": "status",
            "agent_id": "claude_code",
            "orchestration_session_id": orchestration_session_id,
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
            "orchestration_session_id": orchestration_session_id,
            "run_id": "0195f8f1-7a35-7b7f-9c4d-9a7c2f5d6f14",
            "parent_run_id": bad_parent_run_id,
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
    assert_malformed_nested_parent_correlation_failure(
        &output,
        "claude_code",
        orchestration_session_id,
        "0195f8f1-7a35-7b7f-9c4d-9a7c2f5d6f14",
        bad_parent_run_id,
    );
}

#[test]
fn agent_status_fails_closed_when_selected_nested_row_omits_provider() {
    let fixture = AgentSuccessorFixture::new();
    seed_nested_gateway_status_fixture(&fixture);
    let orchestration_session_id = "0195f8f1-7a34-7b7f-9c4d-9a7c2f5d6f12";
    fixture.write_trace_events(&[
        json!({
            "ts": "2026-04-05T00:00:00Z",
            "event_type": "agent_event",
            "session_id": "ses_agent_hub",
            "component": "agent-hub",
            "kind": "status",
            "agent_id": "claude_code",
            "orchestration_session_id": orchestration_session_id,
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
            "orchestration_session_id": orchestration_session_id,
            "run_id": "0195f8f1-7a35-7b7f-9c4d-9a7c2f5d6f14",
            "parent_run_id": "0195f8f1-7a35-7b7f-9c4d-9a7c2f5d6f13",
            "backend_id": "cli:claude_code",
            "client": "claude_code",
            "router": "substrate_gateway",
            "protocol": "openai.responses",
            "auth_authority": "codex_subscription",
            "data": { "summary": "nested gateway request completed" }
        }),
    ]);

    let output = fixture.run(&["agent", "status", "--json"]);
    assert_malformed_nested_required_fields_failure(
        &output,
        "claude_code",
        orchestration_session_id,
        "0195f8f1-7a35-7b7f-9c4d-9a7c2f5d6f14",
        "provider",
    );
}

#[test]
fn agent_status_fails_closed_when_selected_nested_row_omits_auth_authority() {
    let fixture = AgentSuccessorFixture::new();
    seed_nested_gateway_status_fixture(&fixture);
    let orchestration_session_id = "0195f8f1-7a34-7b7f-9c4d-9a7c2f5d6f12";
    fixture.write_trace_events(&[
        json!({
            "ts": "2026-04-05T00:00:00Z",
            "event_type": "agent_event",
            "session_id": "ses_agent_hub",
            "component": "agent-hub",
            "kind": "status",
            "agent_id": "claude_code",
            "orchestration_session_id": orchestration_session_id,
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
            "orchestration_session_id": orchestration_session_id,
            "run_id": "0195f8f1-7a35-7b7f-9c4d-9a7c2f5d6f14",
            "parent_run_id": "0195f8f1-7a35-7b7f-9c4d-9a7c2f5d6f13",
            "backend_id": "cli:claude_code",
            "client": "claude_code",
            "router": "substrate_gateway",
            "protocol": "openai.responses",
            "provider": "openai",
            "data": { "summary": "nested gateway request completed" }
        }),
    ]);

    let output = fixture.run(&["agent", "status", "--json"]);
    assert_malformed_nested_required_fields_failure(
        &output,
        "claude_code",
        orchestration_session_id,
        "0195f8f1-7a35-7b7f-9c4d-9a7c2f5d6f14",
        "auth_authority",
    );
}

#[test]
fn agent_status_fails_closed_when_selected_nested_row_omits_provider_and_auth_authority() {
    let fixture = AgentSuccessorFixture::new();
    seed_nested_gateway_status_fixture(&fixture);
    let orchestration_session_id = "0195f8f1-7a34-7b7f-9c4d-9a7c2f5d6f12";
    fixture.write_trace_events(&[
        json!({
            "ts": "2026-04-05T00:00:00Z",
            "event_type": "agent_event",
            "session_id": "ses_agent_hub",
            "component": "agent-hub",
            "kind": "status",
            "agent_id": "claude_code",
            "orchestration_session_id": orchestration_session_id,
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
            "orchestration_session_id": orchestration_session_id,
            "run_id": "0195f8f1-7a35-7b7f-9c4d-9a7c2f5d6f14",
            "parent_run_id": "0195f8f1-7a35-7b7f-9c4d-9a7c2f5d6f13",
            "backend_id": "cli:claude_code",
            "client": "claude_code",
            "router": "substrate_gateway",
            "protocol": "openai.responses",
            "data": { "summary": "nested gateway request completed" }
        }),
    ]);

    let output = fixture.run(&["agent", "status", "--json"]);
    assert_malformed_nested_required_fields_failure(
        &output,
        "claude_code",
        orchestration_session_id,
        "0195f8f1-7a35-7b7f-9c4d-9a7c2f5d6f14",
        "provider,auth_authority",
    );
}

#[test]
fn agent_status_ignores_malformed_nested_rows_when_parent_surface_is_filtered_out() {
    let fixture = AgentSuccessorFixture::new();
    seed_nested_gateway_status_fixture(&fixture);
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
            "backend_id": "cli:claude_code",
            "client": "claude_code",
            "router": "substrate_gateway",
            "protocol": "openai.responses",
            "data": { "summary": "nested gateway request completed" }
        }),
    ]);

    let output = fixture.run(&["agent", "status", "--role", "member", "--json"]);
    assert!(
        output.status.success(),
        "status should ignore malformed nested rows when the parent pure-agent row is filtered out: {output:?}"
    );

    let json = parse_json_output(&output);
    assert_eq!(
        json.pointer("/role_filter").and_then(Value::as_str),
        Some("member")
    );
    assert_eq!(
        json["sessions"]
            .as_array()
            .expect("sessions should be an array")
            .len(),
        0,
        "filtered-out parent rows should not remain in the selected output surface: {json}"
    );
    assert_eq!(
        json["nested_llm_records"]
            .as_array()
            .expect("nested_llm_records should be an array")
            .len(),
        0,
        "nested rows under filtered-out parents should be ignored rather than failing or emitting output: {json}"
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
