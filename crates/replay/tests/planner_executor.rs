#![cfg(unix)]

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Mutex;

use substrate_replay::replay::{
    execute_direct, execute_in_world, parse_command, replay_sequence, world_isolation_available,
    ExecutionResult, ExecutionState,
};

static ENV_LOCK: Mutex<()> = Mutex::new(());

struct EnvGuard {
    previous: Vec<(String, Option<std::ffi::OsString>)>,
}

impl EnvGuard {
    fn set(vars: &[(&str, Option<&str>)]) -> Self {
        let previous = vars
            .iter()
            .map(|(key, _)| (key.to_string(), std::env::var_os(key)))
            .collect::<Vec<_>>();
        for (key, value) in vars {
            match value {
                Some(v) => std::env::set_var(key, v),
                None => std::env::remove_var(key),
            }
        }
        Self { previous }
    }
}

impl Drop for EnvGuard {
    fn drop(&mut self) {
        for (key, value) in self.previous.drain(..) {
            match value {
                Some(v) => std::env::set_var(&key, v),
                None => std::env::remove_var(&key),
            }
        }
    }
}

fn make_state(raw_cmd: &str) -> ExecutionState {
    let (command, args) = parse_command(raw_cmd);
    ExecutionState {
        raw_cmd: raw_cmd.to_string(),
        command,
        args,
        cwd: std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")),
        env: HashMap::new(),
        stdin: None,
        session_id: "session".to_string(),
        span_id: "span".to_string(),
    }
}

#[tokio::test]
async fn execute_direct_injects_replay_env() {
    let state = make_state("printf '%s' \"$SUBSTRATE_REPLAY\"");
    let result = execute_direct(&state, 5).await.unwrap();
    assert_eq!(result.exit_code, 0);
    assert_eq!(String::from_utf8_lossy(&result.stdout), "1");
    assert!(result.fs_diff.is_none());
}

#[tokio::test]
async fn execute_direct_streams_stdin() {
    let mut state = make_state("cat");
    state.stdin = Some(b"hello world".to_vec());

    let result = execute_direct(&state, 5).await.unwrap();
    assert_eq!(result.exit_code, 0);
    assert_eq!(result.stdout, b"hello world");
}

#[tokio::test]
async fn execute_in_world_respects_disable_env() {
    let _lock = ENV_LOCK.lock().unwrap();
    let _guard = EnvGuard::set(&[
        ("SUBSTRATE_WORLD", Some("disabled")),
        ("SUBSTRATE_WORLD_ENABLED", Some("0")),
    ]);

    let state = make_state("echo direct-only");
    let result = execute_in_world(&state, 5).await.unwrap();
    assert_eq!(result.exit_code, 0);
    assert_eq!(
        String::from_utf8_lossy(&result.stdout).trim(),
        "direct-only"
    );
    assert!(
        result.fs_diff.is_none(),
        "world disable flag should force direct execution path"
    );
}

#[tokio::test]
async fn replay_sequence_collects_success_and_failure() {
    let mut ok = make_state("echo ok");
    ok.env.insert("TEST_VAR".into(), "1".into());
    let fail = make_state("false");

    let results: Vec<ExecutionResult> = replay_sequence(vec![ok, fail], 5, false).await.unwrap();
    assert_eq!(results.len(), 2);
    assert_eq!(results[0].exit_code, 0);
    assert_eq!(String::from_utf8_lossy(&results[0].stdout).trim(), "ok");
    assert_ne!(results[1].exit_code, 0);
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
#[test]
fn world_isolation_available_honors_disable_switch() {
    let _lock = ENV_LOCK.lock().unwrap();
    let _guard = EnvGuard::set(&[
        ("SUBSTRATE_WORLD", Some("disabled")),
        ("SUBSTRATE_WORLD_ENABLED", Some("0")),
    ]);

    assert!(
        !world_isolation_available(),
        "env flags should disable world isolation"
    );
}
