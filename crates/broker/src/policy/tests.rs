use super::*;

const PCM1_BASE_POLICY_YAML: &str = r#"
id: "test-policy"
name: "Test Policy"

world_fs:
  mode: writable
  isolation: full
  require_world: false
  read:
    allow_list: ["."]
  write:
    allow_list: ["dist"]

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
"#;

fn pcm1_policy_yaml_with_backend_entries(
    llm_backend: &str,
    agents_backend: &str,
    host_credentials_backend: &str,
) -> String {
    format!(
        r#"
id: "test-policy"
name: "Test Policy"

world_fs:
  mode: writable
  isolation: full
  require_world: false
  read:
    allow_list: ["."]
  write:
    allow_list: ["dist"]

llm:
  fail_closed:
    routing: false
  require_approval: true
  allowed_backends:
    - "{llm_backend}"
  secrets:
    env_allowed:
      - OPENAI_API_KEY

agents:
  allowed_backends:
    - "{agents_backend}"
  fail_closed:
    routing: false
  host_credentials:
    read:
      allowed_backends:
        - "{host_credentials_backend}"

workflow:
  router:
    enabled: true
    allow_cross_workspace: true
    allowed_rule_ids:
      - "rule-1"
    allowed_workflow_ids:
      - "workflow-1"
    allowed_target_workspace_ids:
      - "workspace-1"

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

metadata: {{}}
"#
    )
}

#[test]
fn test_default_policy() {
    let policy = Policy::default();
    assert_eq!(policy.id, "default");
    assert_eq!(policy.fs_read, vec!["*".to_string()]);
    assert_eq!(policy.world_fs_mode, WorldFsMode::Writable);
    assert!(!policy.cmd_denied.is_empty());
}

#[test]
fn test_policy_merge() {
    let mut base = Policy {
        fs_write: vec!["/tmp/*".to_string()],
        ..Default::default()
    };

    let addon = Policy {
        id: "addon".to_string(),
        name: "Addon Policy".to_string(),
        fs_write: vec!["/var/tmp/*".to_string()],
        cmd_denied: vec!["sudo rm -rf /".to_string()],
        ..Default::default()
    };

    base.merge(&addon);

    assert_eq!(base.fs_write.len(), 2);
    assert!(base.fs_write.contains(&"/tmp/*".to_string()));
    assert!(base.fs_write.contains(&"/var/tmp/*".to_string()));
    assert!(base.cmd_denied.contains(&"sudo rm -rf /".to_string()));
}

#[test]
fn test_path_checks() {
    let policy = Policy {
        fs_read: vec!["/home/*".to_string(), "/tmp/*".to_string()],
        fs_write: vec!["/tmp/*".to_string()],
        ..Default::default()
    };

    assert!(policy.is_path_readable("/home/user/file.txt"));
    assert!(policy.is_path_readable("/tmp/test.txt"));
    assert!(!policy.is_path_readable("/etc/passwd"));

    assert!(policy.is_path_writable("/tmp/test.txt"));
    assert!(!policy.is_path_writable("/home/user/file.txt"));
}

#[test]
fn test_host_allowed() {
    let policy = Policy {
        net_allowed: vec!["github.com".to_string(), "*.example.com".to_string()],
        ..Default::default()
    };

    assert!(policy.is_host_allowed("github.com"));
    assert!(policy.is_host_allowed("api.github.com"));
    assert!(policy.is_host_allowed("test.example.com"));
    assert!(!policy.is_host_allowed("evil.com"));
}

#[test]
fn pcm1_policy_yaml_parses_with_strict_schema() {
    let policy: Policy = serde_yaml::from_str(PCM1_BASE_POLICY_YAML).expect("PCM1 policy parses");
    assert_eq!(policy.id, "test-policy");
}

#[test]
fn pcm1_policy_yaml_rejects_unknown_keys() {
    let raw = format!("{PCM1_BASE_POLICY_YAML}\nunexpected: 1\n");
    let err = serde_yaml::from_str::<Policy>(&raw).expect_err("unknown key should be rejected");
    let msg = err.to_string();
    assert!(
        msg.contains("unknown field") || msg.contains("unknown key"),
        "expected unknown-key error, got: {msg}"
    );
}

#[test]
fn pcm1_policy_yaml_rejects_type_mismatches() {
    let raw = PCM1_BASE_POLICY_YAML.replace(r#"id: "test-policy""#, "id: 123");
    let err = serde_yaml::from_str::<Policy>(&raw).expect_err("type mismatch should be rejected");
    let msg = err.to_string();
    assert!(
        msg.contains("invalid type") || msg.contains("expected"),
        "expected type error, got: {msg}"
    );
}

#[test]
fn pcm1_policy_yaml_accepts_llm_agents_and_workflow_router_families() {
    let raw = pcm1_policy_yaml_with_backend_entries("cli:codex", "cli:codex", "cli:codex");
    let policy: Policy = serde_yaml::from_str(&raw).expect("policy with new key families parses");

    assert!(!policy.llm_fail_closed_routing);
    assert!(policy.llm_require_approval);
    assert_eq!(policy.llm_allowed_backends, vec!["cli:codex".to_string()]);
    assert_eq!(
        policy.llm_secrets_env_allowed,
        vec!["OPENAI_API_KEY".to_string()]
    );
    assert_eq!(
        policy.agents_allowed_backends,
        vec!["cli:codex".to_string()]
    );
    assert!(!policy.agents_fail_closed_routing);
    assert_eq!(
        policy.agents_host_credentials_read_allowed_backends,
        vec!["cli:codex".to_string()]
    );
    assert!(policy.workflow_router_enabled);
    assert!(policy.workflow_router_allow_cross_workspace);
    assert_eq!(
        policy.workflow_router_allowed_rule_ids,
        vec!["rule-1".to_string()]
    );
    assert_eq!(
        policy.workflow_router_allowed_workflow_ids,
        vec!["workflow-1".to_string()]
    );
    assert_eq!(
        policy.workflow_router_allowed_target_workspace_ids,
        vec!["workspace-1".to_string()]
    );
}

#[test]
fn pcm1_policy_yaml_rejects_malformed_llm_allowed_backends_ids() {
    let raw = pcm1_policy_yaml_with_backend_entries("CLI:Codex", "cli:codex", "cli:codex");
    let err = serde_yaml::from_str::<Policy>(&raw)
        .expect_err("invalid llm.allowed_backends entry should be rejected");
    let msg = err.to_string();
    assert!(
        msg.contains("llm.allowed_backends"),
        "expected llm.allowed_backends diagnostic, got: {msg}"
    );
}

#[test]
fn pcm1_policy_yaml_rejects_malformed_agents_allowed_backends_ids() {
    let raw = pcm1_policy_yaml_with_backend_entries("cli:codex", "bad backend", "cli:codex");
    let err = serde_yaml::from_str::<Policy>(&raw)
        .expect_err("invalid agents.allowed_backends entry should be rejected");
    let msg = err.to_string();
    assert!(
        msg.contains("agents.allowed_backends"),
        "expected agents.allowed_backends diagnostic, got: {msg}"
    );
}

#[test]
fn pcm1_policy_yaml_rejects_malformed_agents_host_credentials_backend_ids() {
    let raw = pcm1_policy_yaml_with_backend_entries("cli:codex", "cli:codex", "api:openai:extra");
    let err = serde_yaml::from_str::<Policy>(&raw)
        .expect_err("invalid agents.host_credentials.read.allowed_backends entry should fail");
    let msg = err.to_string();
    assert!(
        msg.contains("agents.host_credentials.read.allowed_backends"),
        "expected host credentials backend diagnostic, got: {msg}"
    );
}

#[test]
fn pcm1_policy_rejects_legacy_world_fs_cage_key() {
    let raw = PCM1_BASE_POLICY_YAML.replace(
        "require_world: false",
        "require_world: false\n  cage: workspace",
    );
    let err = serde_yaml::from_str::<Policy>(&raw).expect_err("legacy key should be rejected");
    let msg = err.to_string();
    assert!(
        msg.contains("unknown field") || msg.contains("legacy"),
        "expected legacy-key rejection, got: {msg}"
    );
}
