#![cfg(unix)]
//! Integration tests for Replay module

use std::collections::HashMap;
use std::env;
use std::path::PathBuf;
use std::sync::Mutex;
use substrate_replay::{find_spans_to_replay, replay_batch, replay_span, ReplayConfig, SpanFilter};
use tempfile::NamedTempFile;
use tokio::io::AsyncWriteExt;

static ENV_LOCK: Mutex<()> = Mutex::new(());

struct EnvGuard {
    vars: Vec<(String, Option<String>)>,
}

impl EnvGuard {
    fn set(vars: &[(&str, &str)]) -> Self {
        let mut captured = Vec::new();
        for (key, value) in vars {
            let prev = env::var(key).ok();
            env::set_var(key, value);
            captured.push((key.to_string(), prev));
        }
        Self { vars: captured }
    }
}

impl Drop for EnvGuard {
    fn drop(&mut self) {
        for (key, value) in self.vars.drain(..) {
            if let Some(v) = value {
                env::set_var(&key, v);
            } else {
                env::remove_var(&key);
            }
        }
    }
}

/// Test creating and replaying a simple trace
#[tokio::test]
async fn test_basic_replay_flow() {
    // Create a temporary trace file with test data
    let temp_file = NamedTempFile::new().unwrap();
    let trace_content = r#"
{"ts":"2024-01-01T00:00:00Z","event_type":"command_start","span_id":"test-span-1","session_id":"session-1","component":"shell","cmd":"echo hello","cwd":"/tmp"}
{"ts":"2024-01-01T00:00:01Z","event_type":"command_complete","span_id":"test-span-1","session_id":"session-1","component":"shell","cmd":"echo hello","cwd":"/tmp","exit_code":0,"stdout":"hello\n","replay_context":{"path":"/usr/bin:/bin","env_hash":"abc123","hostname":"test-host","user":"testuser","shell":"/bin/bash","term":"xterm-256color","world_image":null}}
{"ts":"2024-01-01T00:00:02Z","event_type":"command_complete","span_id":"test-span-2","session_id":"session-1","component":"shell","cmd":"false","exit_code":1}
"#;

    let mut file = tokio::fs::File::create(temp_file.path()).await.unwrap();
    file.write_all(trace_content.as_bytes()).await.unwrap();
    file.flush().await.unwrap();
    drop(file);

    // Test replay configuration
    let config = ReplayConfig {
        trace_file: temp_file.path().to_path_buf(),
        strict: false,
        timeout: 10,
        fresh_world: false, // Use direct execution for now
        env_overrides: Default::default(),
        ignore_timing: true,
        max_output_compare: 1024,
    };

    // Test replaying a successful command
    let result = replay_span("test-span-1", &config).await.unwrap();
    assert_eq!(result.exit_code, 0);
    assert!(result.matched);
    assert_eq!(String::from_utf8_lossy(&result.stdout).trim(), "hello");

    // Test replaying a failed command
    let result = replay_span("test-span-2", &config).await.unwrap();
    assert_eq!(result.exit_code, 1);
}

/// Test filtering spans from trace
#[tokio::test]
async fn test_span_filtering() {
    let temp_file = NamedTempFile::new().unwrap();
    let trace_content = r#"
{"ts":"2024-01-01T00:00:00Z","event_type":"command_complete","span_id":"echo-1","session_id":"s1","component":"shell","cmd":"echo test","exit_code":0}
{"ts":"2024-01-01T00:00:01Z","event_type":"command_complete","span_id":"ls-1","session_id":"s1","component":"shell","cmd":"ls /tmp","exit_code":0}
{"ts":"2024-01-01T00:00:02Z","event_type":"command_complete","span_id":"echo-2","session_id":"s1","component":"shim","cmd":"echo another","exit_code":0}
{"ts":"2024-01-01T00:00:03Z","event_type":"command_complete","span_id":"cat-1","session_id":"s1","component":"shell","cmd":"cat file.txt","exit_code":1}
"#;

    let mut file = tokio::fs::File::create(temp_file.path()).await.unwrap();
    file.write_all(trace_content.as_bytes()).await.unwrap();
    file.flush().await.unwrap();
    drop(file);

    // Filter by command pattern
    let filter = SpanFilter {
        command_patterns: vec!["echo".to_string()],
        ..Default::default()
    };

    let spans = find_spans_to_replay(temp_file.path(), filter)
        .await
        .unwrap();
    assert_eq!(spans.len(), 2);
    assert!(spans.contains(&"echo-1".to_string()));
    assert!(spans.contains(&"echo-2".to_string()));

    // Filter by component
    let filter = SpanFilter {
        component: Some("shim".to_string()),
        ..Default::default()
    };

    let spans = find_spans_to_replay(temp_file.path(), filter)
        .await
        .unwrap();
    assert_eq!(spans.len(), 1);
    assert_eq!(spans[0], "echo-2");

    // Filter by exit code
    let filter = SpanFilter {
        exit_codes: Some(vec![1]),
        ..Default::default()
    };

    let spans = find_spans_to_replay(temp_file.path(), filter)
        .await
        .unwrap();
    assert_eq!(spans.len(), 1);
    assert_eq!(spans[0], "cat-1");
}

/// Test batch replay and regression reporting
#[tokio::test]
async fn test_batch_replay() {
    let temp_file = NamedTempFile::new().unwrap();
    let trace_content = r#"
{"ts":"2024-01-01T00:00:00Z","event_type":"command_complete","span_id":"cmd-1","session_id":"s1","component":"shell","cmd":"echo test1","exit_code":0,"stdout":"test1\n"}
{"ts":"2024-01-01T00:00:01Z","event_type":"command_complete","span_id":"cmd-2","session_id":"s1","component":"shell","cmd":"echo test2","exit_code":0,"stdout":"test2\n"}
{"ts":"2024-01-01T00:00:02Z","event_type":"command_complete","span_id":"cmd-3","session_id":"s1","component":"shell","cmd":"true","exit_code":0}
"#;

    let mut file = tokio::fs::File::create(temp_file.path()).await.unwrap();
    file.write_all(trace_content.as_bytes()).await.unwrap();
    file.flush().await.unwrap();
    drop(file);

    let config = ReplayConfig {
        trace_file: temp_file.path().to_path_buf(),
        strict: false,
        timeout: 10,
        fresh_world: false,
        env_overrides: Default::default(),
        ignore_timing: true,
        max_output_compare: 1024,
    };

    let span_ids = vec![
        "cmd-1".to_string(),
        "cmd-2".to_string(),
        "cmd-3".to_string(),
    ];

    let report = replay_batch(&span_ids, &config).await.unwrap();

    // All commands should match
    assert_eq!(report.total_spans, 3);
    assert_eq!(report.matched, 3);
    assert_eq!(report.diverged, 0);
    assert_eq!(report.pass_rate, 100.0);
    assert!(report
        .recommendations
        .iter()
        .any(|r| r.contains("successfully")));
}

/// Test environment reconstruction
#[tokio::test]
async fn test_env_reconstruction() {
    use chrono::Utc;
    use std::collections::HashMap;
    use substrate_replay::state::{reconstruct_state, ReplayContext, TraceSpan};

    let span = TraceSpan {
        ts: Utc::now(),
        event_type: "command_complete".to_string(),
        span_id: "test-span".to_string(),
        session_id: "test-session".to_string(),
        component: "shell".to_string(),
        cmd: "env | grep TEST".to_string(),
        cwd: Some(PathBuf::from("/workspace")),
        exit_code: Some(0),
        duration_ms: Some(50),
        policy_decision: None,
        fs_diff: None,
        scopes_used: None,
        replay_context: Some(ReplayContext {
            path: Some("/custom/bin:/usr/bin".to_string()),
            env_hash: "hash123".to_string(),
            hostname: Some("dev-machine".to_string()),
            user: Some("developer".to_string()),
            shell: Some("/bin/zsh".to_string()),
            term: Some("xterm-256color".to_string()),
            world_image: None,
        }),
        stdout: None,
        stderr: None,
        env_hash: None,
    };

    let mut overrides = HashMap::new();
    overrides.insert("TEST_VAR".to_string(), "test_value".to_string());

    let exec_state = reconstruct_state(&span, &overrides).unwrap();

    assert_eq!(exec_state.command, "env");
    assert_eq!(exec_state.cwd, PathBuf::from("/workspace"));
    assert_eq!(
        exec_state.env.get("PATH"),
        Some(&"/custom/bin:/usr/bin".to_string())
    );
    assert_eq!(exec_state.env.get("USER"), Some(&"developer".to_string()));
    assert_eq!(
        exec_state.env.get("TEST_VAR"),
        Some(&"test_value".to_string())
    );
    assert_eq!(
        exec_state.env.get("SUBSTRATE_REPLAY"),
        Some(&"1".to_string())
    );
}

#[test]
fn reconstruct_state_preserves_caged_anchor_env() {
    use chrono::Utc;
    use substrate_replay::state::{reconstruct_state, TraceSpan};

    let _env_lock = ENV_LOCK.lock().unwrap();
    let _guard = EnvGuard::set(&[
        ("SUBSTRATE_ANCHOR_MODE", "custom"),
        ("SUBSTRATE_WORLD_ROOT_MODE", "custom"),
        ("SUBSTRATE_ANCHOR_PATH", "/opt/caged-root"),
        ("SUBSTRATE_WORLD_ROOT_PATH", "/opt/caged-root"),
        ("SUBSTRATE_CAGED", "1"),
    ]);

    let span = TraceSpan {
        ts: Utc::now(),
        event_type: "command_complete".to_string(),
        span_id: "span-caged".to_string(),
        session_id: "session-caged".to_string(),
        component: "shell".to_string(),
        cmd: "pwd".to_string(),
        cwd: Some(PathBuf::from("/opt/caged-root/workdir")),
        exit_code: Some(0),
        duration_ms: Some(5),
        policy_decision: None,
        fs_diff: None,
        scopes_used: None,
        replay_context: None,
        stdout: None,
        stderr: None,
        env_hash: None,
    };

    let exec_state = reconstruct_state(&span, &HashMap::new()).unwrap();
    assert_eq!(
        exec_state.env.get("SUBSTRATE_ANCHOR_PATH"),
        Some(&"/opt/caged-root".to_string())
    );
    assert_eq!(
        exec_state.env.get("SUBSTRATE_CAGED"),
        Some(&"1".to_string())
    );
    assert_eq!(exec_state.cwd, PathBuf::from("/opt/caged-root/workdir"));
}

#[test]
fn reconstruct_state_preserves_uncaged_anchor_env() {
    use chrono::Utc;
    use substrate_replay::state::{reconstruct_state, TraceSpan};

    let _env_lock = ENV_LOCK.lock().unwrap();
    let _guard = EnvGuard::set(&[
        ("SUBSTRATE_ANCHOR_MODE", "project"),
        ("SUBSTRATE_WORLD_ROOT_MODE", "project"),
        ("SUBSTRATE_ANCHOR_PATH", "/srv/project"),
        ("SUBSTRATE_WORLD_ROOT_PATH", "/srv/project"),
        ("SUBSTRATE_CAGED", "0"),
    ]);

    let span = TraceSpan {
        ts: Utc::now(),
        event_type: "command_complete".to_string(),
        span_id: "span-uncaged".to_string(),
        session_id: "session-uncaged".to_string(),
        component: "shell".to_string(),
        cmd: "pwd".to_string(),
        cwd: Some(PathBuf::from("/srv/project/app")),
        exit_code: Some(0),
        duration_ms: Some(5),
        policy_decision: None,
        fs_diff: None,
        scopes_used: None,
        replay_context: None,
        stdout: None,
        stderr: None,
        env_hash: None,
    };

    let exec_state = reconstruct_state(&span, &HashMap::new()).unwrap();
    assert_eq!(
        exec_state.env.get("SUBSTRATE_ANCHOR_MODE"),
        Some(&"project".to_string())
    );
    assert_eq!(
        exec_state.env.get("SUBSTRATE_CAGED"),
        Some(&"0".to_string())
    );
    assert_eq!(exec_state.cwd, PathBuf::from("/srv/project/app"));
}
