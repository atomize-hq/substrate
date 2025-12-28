use super::*;

const PCM1_BASE_POLICY_YAML: &str = r#"
id: "test-policy"
name: "Test Policy"

world_fs:
  mode: writable
  isolation: project
  require_world: false
  read_allowlist: ["*"]
  write_allowlist: []

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
fn pcm1_policy_invariant_full_isolation_requires_world() {
    let raw = PCM1_BASE_POLICY_YAML.replace("isolation: project", "isolation: full");
    assert!(
        serde_yaml::from_str::<Policy>(&raw).is_err(),
        "full isolation with require_world=false must fail to load"
    );
}

#[test]
fn pcm1_policy_invariant_read_only_requires_world() {
    let raw = PCM1_BASE_POLICY_YAML.replace("mode: writable", "mode: read_only");
    assert!(
        serde_yaml::from_str::<Policy>(&raw).is_err(),
        "read_only with require_world=false must fail to load"
    );
}

#[test]
fn pcm1_policy_rejects_legacy_world_fs_cage_key() {
    let raw = PCM1_BASE_POLICY_YAML.replace("isolation: project", "cage: project");
    let err = serde_yaml::from_str::<Policy>(&raw).expect_err("legacy key should be rejected");
    let msg = err.to_string();
    assert!(
        msg.contains("unknown field") || msg.contains("legacy"),
        "expected legacy-key rejection, got: {msg}"
    );
}
