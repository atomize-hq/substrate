#![cfg(unix)]

use serde_json::{json, Value};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::OnceLock;
use tokio::sync::Mutex;

use substrate_replay::replay::{
    execute_direct, execute_in_world, parse_command, record_replay_strategy, replay_sequence,
    world_isolation_available, ExecutionResult, ExecutionState,
};
use substrate_trace::{set_global_trace_context, TraceContext};

static ENV_LOCK: Mutex<()> = Mutex::const_new(());

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
        recorded_origin: substrate_trace::ExecutionOrigin::Host,
        recorded_origin_source: None,
        recorded_transport: None,
        target_origin: substrate_trace::ExecutionOrigin::Host,
        origin_reason: None,
        origin_reason_code: None,
        world_disable_source: None,
    }
}

fn replay_strategy_trace_path() -> &'static PathBuf {
    static TRACE_PATH: OnceLock<PathBuf> = OnceLock::new();
    TRACE_PATH.get_or_init(|| {
        let tempdir = Box::leak(Box::new(tempfile::tempdir().unwrap()));
        let trace_path = tempdir.path().join("trace.jsonl");
        let ctx = TraceContext::default();
        ctx.init_trace(Some(trace_path.clone())).unwrap();
        set_global_trace_context(ctx).unwrap();
        trace_path
    })
}

fn latest_replay_strategy(trace_path: &PathBuf) -> Option<Value> {
    fs::read_to_string(trace_path)
        .ok()?
        .lines()
        .filter_map(|line| serde_json::from_str::<Value>(line).ok())
        .filter(|value| {
            value.get("event_type").and_then(|value| value.as_str()) == Some("replay_strategy")
        })
        .last()
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
async fn execute_direct_defaults_stdin_to_null_when_missing() {
    let state = make_state("cat");

    let result = execute_direct(&state, 1).await.unwrap();
    assert_eq!(result.exit_code, 0);
    assert!(result.stdout.is_empty());
}

#[tokio::test]
async fn execute_in_world_respects_disable_env() {
    let _lock = ENV_LOCK.lock().await;
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

#[test]
fn record_replay_strategy_emits_world_disable_source_for_effective_disable() {
    let trace_path = replay_strategy_trace_path();
    fs::write(trace_path, "").unwrap();

    let mut state = make_state("echo effective-disable");
    state.target_origin = substrate_trace::ExecutionOrigin::Host;
    state.recorded_origin = substrate_trace::ExecutionOrigin::Host;
    state.origin_reason =
        Some("world isolation disabled by effective config (source unknown)".to_string());
    state.origin_reason_code = Some("world_disabled_unknown".to_string());
    state.world_disable_source = Some(json!({
        "key": "world.enabled",
        "layer": "unknown",
        "value_display": false
    }));

    record_replay_strategy(
        &state,
        "host",
        None,
        None,
        json!({"origin_summary": "host"}),
    );

    let Some(strategy) = latest_replay_strategy(trace_path) else {
        panic!("expected replay_strategy entry");
    };
    assert_eq!(
        strategy
            .get("origin_reason")
            .and_then(|value| value.as_str()),
        Some("world isolation disabled by effective config (source unknown)")
    );
    assert_eq!(
        strategy
            .get("origin_reason_code")
            .and_then(|value| value.as_str()),
        Some("world_disabled_unknown")
    );
    assert_eq!(
        strategy.get("world_disable_source"),
        Some(&json!({
            "key": "world.enabled",
            "layer": "unknown",
            "value_display": false
        }))
    );
}

#[test]
fn record_replay_strategy_omits_world_disable_source_for_replay_local_opt_out() {
    let trace_path = replay_strategy_trace_path();
    fs::write(trace_path, "").unwrap();

    let mut state = make_state("echo replay-local");
    state.target_origin = substrate_trace::ExecutionOrigin::Host;
    state.recorded_origin = substrate_trace::ExecutionOrigin::Host;
    state.origin_reason = Some("SUBSTRATE_REPLAY_USE_WORLD=disabled".to_string());
    state.origin_reason_code = Some("env_disabled".to_string());
    state.world_disable_source = Some(json!({
        "key": "world.enabled",
        "layer": "unknown",
        "value_display": false
    }));

    record_replay_strategy(
        &state,
        "host",
        None,
        Some("SUBSTRATE_REPLAY_USE_WORLD=disabled"),
        json!({"origin_summary": "host"}),
    );

    let Some(strategy) = latest_replay_strategy(trace_path) else {
        panic!("expected replay_strategy entry");
    };
    assert_eq!(
        strategy
            .get("origin_reason_code")
            .and_then(|value| value.as_str()),
        Some("env_disabled")
    );
    assert!(
        strategy.get("world_disable_source").is_none(),
        "replay-local opt-out must not emit world_disable_source: {strategy:?}"
    );
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
#[test]
fn world_isolation_available_honors_disable_switch() {
    let _lock = ENV_LOCK.blocking_lock();
    let _guard = EnvGuard::set(&[
        ("SUBSTRATE_WORLD", Some("disabled")),
        ("SUBSTRATE_WORLD_ENABLED", Some("0")),
    ]);

    assert!(
        !world_isolation_available(),
        "env flags should disable world isolation"
    );
}
