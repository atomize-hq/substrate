use crate::execution::settings::WorldRootSettings;
use crate::execution::world_env_guard;
use crate::execution::{ShellConfig, ShellMode};
use parking_lot::{ReentrantMutex, ReentrantMutexGuard};
use std::collections::HashMap;
use std::env;
use std::path::PathBuf;
use substrate_common::WorldRootMode;
use substrate_trace::{init_trace, set_global_trace_context, TraceContext};
use tempfile::TempDir;

pub(crate) fn test_shell_config(temp: &TempDir) -> ShellConfig {
    let trace_log_file = temp.path().join("trace.jsonl");
    env::set_var("SHIM_TRACE_LOG", &trace_log_file);
    let _ = set_global_trace_context(TraceContext::default());
    let _ = init_trace(Some(trace_log_file.clone()));

    ShellConfig {
        mode: ShellMode::Interactive { use_pty: false },
        session_id: "test-session".to_string(),
        trace_log_file,
        original_path: env::var("PATH").unwrap_or_default(),
        shim_dir: temp.path().join("shims"),
        shell_path: if cfg!(windows) {
            "cmd.exe".to_string()
        } else {
            "/bin/sh".to_string()
        },
        ci_mode: false,
        no_exit_on_error: false,
        skip_shims: false,
        no_world: false,
        world_root: WorldRootSettings {
            mode: WorldRootMode::Project,
            path: temp.path().to_path_buf(),
            caged: true,
        },
        async_repl: false,
        env_vars: HashMap::new(),
        manager_init_path: temp.path().join("manager_init.sh"),
        manager_env_path: temp.path().join("manager_env.sh"),
        shimmed_path: Some(temp.path().join("shims").display().to_string()),
        host_bash_env: None,
        bash_preexec_path: temp.path().join(".substrate_preexec"),
        preexec_available: true,
    }
}

pub(crate) fn set_env(key: &str, value: &str) -> Option<String> {
    let _guard = world_env_guard();
    let previous = env::var(key).ok();
    env::set_var(key, value);
    previous
}

pub(crate) fn restore_env(key: &str, previous: Option<String>) {
    let _guard = world_env_guard();
    if let Some(value) = previous {
        env::set_var(key, value);
    } else {
        env::remove_var(key);
    }
}

pub(crate) struct DirGuard {
    original: PathBuf,
    _lock: ReentrantMutexGuard<'static, ()>,
}

impl DirGuard {
    pub(crate) fn new() -> Self {
        let lock = cwd_lock().lock();
        let original = env::current_dir().expect("capture cwd");
        Self {
            original,
            _lock: lock,
        }
    }
}

impl Drop for DirGuard {
    fn drop(&mut self) {
        let _ = env::set_current_dir(&self.original);
    }
}

fn cwd_lock() -> &'static ReentrantMutex<()> {
    use std::sync::OnceLock;
    static LOCK: OnceLock<ReentrantMutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| ReentrantMutex::new(()))
}
