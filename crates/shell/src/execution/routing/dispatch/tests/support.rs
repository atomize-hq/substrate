use std::env;
use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};

// Global mutex to ensure tests that modify environment run sequentially
pub(crate) static TEST_ENV_MUTEX: Mutex<()> = Mutex::new(());
static TEST_POLICY_PATH: OnceLock<PathBuf> = OnceLock::new();

// Helper to run tests with TEST_MODE set
pub(crate) fn with_test_mode<F: FnOnce()>(f: F) {
    // Lock the mutex to ensure exclusive access to environment
    let _guard = TEST_ENV_MUTEX.lock().unwrap_or_else(|err| err.into_inner());

    // Save original value if it exists
    let original = env::var("TEST_MODE").ok();

    env::set_var("TEST_MODE", "1");
    let policy_path = TEST_POLICY_PATH.get_or_init(|| {
        let path = env::temp_dir().join(format!(
            "substrate-shell-test-policy-{}.yaml",
            std::process::id()
        ));
        std::fs::write(
            &path,
            r#"
id: shell-tests
name: Shell Tests
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
"#,
        )
        .expect("write shell test policy");
        path
    });
    substrate_broker::init(Some(policy_path)).expect("initialize broker for shell tests");
    f();

    // Restore original value or remove
    match original {
        Some(val) => env::set_var("TEST_MODE", val),
        None => env::remove_var("TEST_MODE"),
    }
}
