use super::*;
use crate::context::TraceContext;
use crate::span::SpanBuilder;
use crate::util::{get_policy_git_hash_at, hash_env_vars};
use chrono::Utc;
use serde_json::Value;
use std::path::Path;
use std::process::Command;
use std::sync::{Arc, Barrier};
use tempfile::TempDir;

fn new_trace_context() -> TraceContext {
    TraceContext::new()
}

static ENV_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

fn initialize_test_git_repository(path: &Path) -> String {
    std::fs::create_dir_all(path).unwrap();
    let run = |args: &[&str]| {
        let output = Command::new("git")
            .arg("-C")
            .arg(path)
            .args(args)
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "git {:?} failed: {}",
            args,
            String::from_utf8_lossy(&output.stderr)
        );
        output
    };
    run(&["init", "--quiet"]);
    run(&["config", "user.name", "Substrate Trace Test"]);
    run(&["config", "user.email", "trace-test@substrate.invalid"]);
    std::fs::write(
        path.join("policy.yaml"),
        format!("mode: observe\nrepository: {}\n", path.display()),
    )
    .unwrap();
    run(&["add", "policy.yaml"]);
    run(&["commit", "--quiet", "-m", "test policy"]);
    String::from_utf8(run(&["rev-parse", "HEAD"]).stdout)
        .unwrap()
        .trim()
        .to_string()
}

#[test]
fn test_span_creation() {
    let span_id = new_span(None);
    assert!(span_id.starts_with("spn_"));
}

#[test]
fn test_span_builder() {
    let ctx = new_trace_context();
    let span = ctx
        .create_span_builder()
        .with_command("echo test")
        .with_cwd("/tmp")
        .with_world_id("wld_123")
        .start();

    assert!(span.is_ok());
    let active = span.unwrap();
    assert!(active.span_id.starts_with("spn_"));
}

#[test]
fn test_trace_initialization() {
    let tmp_dir = TempDir::new().unwrap();
    let trace_path = tmp_dir.path().join("trace.jsonl");

    let ctx = new_trace_context();
    let result = ctx.init_trace(Some(trace_path.clone()));
    assert!(result.is_ok());
    assert!(trace_path.exists());
}

#[test]
fn legacy_ambient_compatibility_remains_the_default_posture() {
    let _guard = ENV_LOCK
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let previous = std::env::var_os("SHIM_TRACE_LOG");
    let tmp = TempDir::new().unwrap();
    let ambient_trace = tmp.path().join("legacy-ambient").join("trace.jsonl");
    std::env::set_var("SHIM_TRACE_LOG", &ambient_trace);

    let context = TraceContext::default();
    context.init_trace(None).unwrap();
    assert!(ambient_trace.is_file());

    match previous {
        Some(value) => std::env::set_var("SHIM_TRACE_LOG", value),
        None => std::env::remove_var("SHIM_TRACE_LOG"),
    }
}

#[test]
fn explicit_product_trace_uses_bound_prefix_and_rejects_conflicts() {
    let _guard = ENV_LOCK
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let previous = std::env::var_os("SHIM_TRACE_LOG");
    let tmp = TempDir::new().unwrap();
    let prefix_a = tmp.path().join("selected-a");
    let prefix_b = tmp.path().join("ambient-b");
    std::env::set_var("SHIM_TRACE_LOG", prefix_b.join("trace.jsonl"));

    let context = TraceContext::explicit_product(&prefix_a).unwrap();
    context.init_trace(None).unwrap();
    context
        .append_to_trace(&serde_json::json!({"event_type": "r2_1_explicit_product"}))
        .unwrap();

    assert!(prefix_a.join("trace.jsonl").is_file());
    assert!(!prefix_b.exists());
    context.init_trace(None).unwrap();
    context
        .init_trace(Some(prefix_a.join("trace.jsonl")))
        .unwrap();
    let error = context
        .init_trace(Some(prefix_b.join("trace.jsonl")))
        .unwrap_err();
    assert!(error
        .to_string()
        .contains("conflicts with bound product trace"));
    assert!(!prefix_b.exists());

    match previous {
        Some(value) => std::env::set_var("SHIM_TRACE_LOG", value),
        None => std::env::remove_var("SHIM_TRACE_LOG"),
    }
}

#[test]
fn explicit_product_rejects_relative_or_parent_traversing_prefixes() {
    assert!(TraceContext::explicit_product(Path::new("/")).is_err());
    assert!(TraceContext::explicit_product(Path::new("relative-prefix")).is_err());
    assert!(TraceContext::explicit_product(Path::new("/tmp/selected/../other")).is_err());
    assert!(TraceContext::explicit_product(Path::new("/tmp/./selected")).is_err());
    assert!(TraceContext::explicit_product(Path::new("/tmp//selected")).is_err());
}

#[test]
fn explicit_product_policy_git_reads_only_bound_prefix() {
    let _guard = ENV_LOCK
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let previous_home = std::env::var_os("HOME");
    let tmp = TempDir::new().unwrap();
    let prefix_a = tmp.path().join("selected-a");
    let prefix_b = tmp.path().join("ambient-b");
    let commit_a = initialize_test_git_repository(&prefix_a);
    let commit_b = initialize_test_git_repository(&prefix_b);
    assert_ne!(commit_a, commit_b);
    std::env::set_var("HOME", &prefix_b);

    let context = TraceContext::explicit_product(&prefix_a).unwrap();
    let replay = context
        .build_replay_context(None, ExecutionOrigin::Host)
        .unwrap();
    assert_eq!(replay.policy_commit.as_deref(), Some(commit_a.as_str()));
    assert_ne!(replay.policy_commit.as_deref(), Some(commit_b.as_str()));

    let missing = tmp.path().join("selected-without-git");
    std::fs::create_dir_all(&missing).unwrap();
    assert_eq!(get_policy_git_hash_at(&missing).unwrap(), None);
    let missing_context = TraceContext::explicit_product(&missing).unwrap();
    assert_eq!(
        missing_context
            .build_replay_context(None, ExecutionOrigin::Host)
            .unwrap()
            .policy_commit,
        None
    );

    match previous_home {
        Some(value) => std::env::set_var("HOME", value),
        None => std::env::remove_var("HOME"),
    }
}

#[cfg(unix)]
#[test]
fn explicit_policy_git_rejects_symlink_to_ambient_repository() {
    use std::os::unix::fs::symlink;

    let tmp = TempDir::new().unwrap();
    let prefix_a = tmp.path().join("selected-a");
    let prefix_b = tmp.path().join("ambient-b");
    std::fs::create_dir_all(&prefix_a).unwrap();
    initialize_test_git_repository(&prefix_b);
    symlink(prefix_b.join(".git"), prefix_a.join(".git")).unwrap();

    assert_eq!(get_policy_git_hash_at(&prefix_a).unwrap(), None);

    let prefix_with_nested_escape = tmp.path().join("selected-with-nested-escape");
    let git_dir = prefix_with_nested_escape.join(".git");
    std::fs::create_dir_all(&git_dir).unwrap();
    std::fs::write(git_dir.join("HEAD"), "ref: refs/heads/master\n").unwrap();
    symlink(prefix_b.join(".git/refs"), git_dir.join("refs")).unwrap();
    assert_eq!(
        get_policy_git_hash_at(&prefix_with_nested_escape).unwrap(),
        None
    );
}

#[test]
fn test_policy_decision_serialization() {
    let decision = PolicyDecision {
        action: "allow".to_string(),
        reason: None,
        restrictions: Some(vec!["no_network".to_string()]),
    };

    let json = serde_json::to_string(&decision).unwrap();
    assert!(json.contains("\"action\":\"allow\""));
    assert!(json.contains("\"restrictions\":[\"no_network\"]"));
}

#[test]
fn test_fs_diff() {
    let mut diff = FsDiff {
        writes: vec!["file1.txt".into()],
        mods: vec!["file2.txt".into()],
        deletes: vec![],
        truncated: false,
        tree_hash: None,
        summary: None,
        display_path: None,
    };
    diff.display_path = Some(std::collections::HashMap::from([(
        "file1.txt".to_string(),
        "C:\\path\\file1.txt".to_string(),
    )]));

    let json = serde_json::to_string(&diff).unwrap();
    assert!(json.contains("\"writes\":[\"file1.txt\"]"));
    assert!(json.contains("\"mods\":[\"file2.txt\"]"));
    assert!(json.contains("display_path"));
}

#[test]
fn test_graph_edge() {
    let mut metadata = std::collections::HashMap::new();
    metadata.insert("latency_ms".to_string(), serde_json::json!(42));

    let edge = GraphEdge {
        edge_type: EdgeType::DataFlow,
        from_span: "spn_123".to_string(),
        to_span: "spn_456".to_string(),
        metadata,
    };

    let json = serde_json::to_string(&edge).unwrap();
    assert!(json.contains("\"edge_type\":\"data_flow\""));
    assert!(json.contains("\"latency_ms\":42"));
}

#[test]
fn test_env_hash() {
    // Test that hash_env_vars returns a consistent hash for the same environment
    let hash1 = hash_env_vars().unwrap();
    let hash2 = hash_env_vars().unwrap();

    // The hash should be consistent within the same test run
    assert_eq!(
        hash1, hash2,
        "Hash should be deterministic for the same environment"
    );

    // The hash should not be empty
    assert!(!hash1.is_empty(), "Hash should not be empty");

    // The hash should be a valid hex string of correct length (SHA256 = 64 hex chars)
    assert_eq!(hash1.len(), 64, "SHA256 hash should be 64 hex characters");
    assert!(
        hash1.chars().all(|c| c.is_ascii_hexdigit()),
        "Hash should be valid hex"
    );
}

#[test]
fn test_rotation_on_write() {
    let tmp = TempDir::new().unwrap();
    let log_path = tmp.path().join("trace.jsonl");

    // Pre-fill with >1MB file to trigger rotation
    let large = vec![b'x'; 2 * 1024 * 1024];
    std::fs::write(&log_path, &large).unwrap();

    // Configure small rotation threshold and retention
    std::env::set_var("TRACE_LOG_MAX_MB", "1");
    std::env::set_var("TRACE_LOG_KEEP", "2");

    let ctx = new_trace_context();
    ctx.init_trace(Some(log_path.clone())).unwrap();

    let entry = serde_json::json!({"event_type":"test","ts":Utc::now().to_rfc3339()});
    ctx.append_to_trace(&entry).unwrap();

    // Original should be rotated to .1 and new file should be small
    let rotated = log_path.with_extension("jsonl.1");
    assert!(rotated.exists());
    assert!(log_path.exists());

    let orig_size = std::fs::metadata(&rotated).unwrap().len();
    assert!(orig_size >= 2 * 1024 * 1024);

    let new_size = std::fs::metadata(&log_path).unwrap().len();
    assert!(new_size < 16 * 1024);
}

#[test]
fn test_rotation_retention_policy() {
    let tmp = TempDir::new().unwrap();
    let log_path = tmp.path().join("trace.jsonl");

    // Seed rotated files .1 and .2
    std::fs::write(log_path.with_extension("jsonl.1"), b"a").unwrap();
    std::fs::write(log_path.with_extension("jsonl.2"), b"b").unwrap();
    // Current file large enough to trigger rotation
    let large = vec![b'x'; 2 * 1024 * 1024];
    std::fs::write(&log_path, &large).unwrap();

    std::env::set_var("TRACE_LOG_MAX_MB", "1");
    std::env::set_var("TRACE_LOG_KEEP", "2");

    let ctx = new_trace_context();
    ctx.init_trace(Some(log_path.clone())).unwrap();

    // After rotation, .2 should become .2 (shifted from .1), and .1 should be the old current
    assert!(log_path.with_extension("jsonl.1").exists());
    assert!(log_path.with_extension("jsonl.2").exists());
    // .3 should NOT exist because keep=2
    assert!(!log_path.with_extension("jsonl.3").exists());
}

#[test]
fn trace_contexts_do_not_share_policy_or_outputs() {
    let tmp = TempDir::new().unwrap();
    let log_a = tmp.path().join("trace_a.jsonl");
    let log_b = tmp.path().join("trace_b.jsonl");

    let ctx_a = TraceContext::new();
    let ctx_b = TraceContext::new();
    ctx_a.init_trace(Some(log_a.clone())).unwrap();
    ctx_b.init_trace(Some(log_b.clone())).unwrap();
    ctx_a.set_policy_id("policy-a");
    ctx_b.set_policy_id("policy-b");

    let barrier = Arc::new(Barrier::new(2));

    let span_a = {
        let ctx = ctx_a.clone();
        let barrier = barrier.clone();
        std::thread::spawn(move || {
            barrier.wait();
            let span = SpanBuilder::new(ctx)
                .with_command("echo alpha")
                .start()
                .expect("start span for context A");
            let span_id = span.get_span_id().to_string();
            span.finish(0, vec![], None)
                .expect("finish span for context A");
            span_id
        })
    };

    let span_b = {
        let ctx = ctx_b.clone();
        let barrier = barrier.clone();
        std::thread::spawn(move || {
            barrier.wait();
            let span = SpanBuilder::new(ctx)
                .with_command("echo beta")
                .start()
                .expect("start span for context B");
            let span_id = span.get_span_id().to_string();
            span.finish(0, vec![], None)
                .expect("finish span for context B");
            span_id
        })
    };

    let span_id_a = span_a.join().expect("thread a panicked");
    let span_id_b = span_b.join().expect("thread b panicked");

    let records_a: Vec<Value> = std::fs::read_to_string(&log_a)
        .unwrap()
        .lines()
        .map(|line| serde_json::from_str::<Value>(line).unwrap())
        .collect();
    let records_b: Vec<Value> = std::fs::read_to_string(&log_b)
        .unwrap()
        .lines()
        .map(|line| serde_json::from_str::<Value>(line).unwrap())
        .collect();

    assert!(!records_a.is_empty(), "context A should write spans");
    assert!(!records_b.is_empty(), "context B should write spans");

    assert!(records_a
        .iter()
        .all(|value| value["policy_id"] == "policy-a"));
    assert!(records_b
        .iter()
        .all(|value| value["policy_id"] == "policy-b"));

    assert!(records_a.iter().any(|value| value["span_id"] == span_id_a));
    assert!(records_b.iter().any(|value| value["span_id"] == span_id_b));

    assert!(records_a.iter().all(|value| value["span_id"] != span_id_b));
    assert!(records_b.iter().all(|value| value["span_id"] != span_id_a));

    assert_eq!(ctx_a.policy_id(), "policy-a");
    assert_eq!(ctx_b.policy_id(), "policy-b");
}

#[test]
fn command_complete_includes_policy_resolution_mode_default() {
    let tmp_dir = TempDir::new().unwrap();
    let trace_path = tmp_dir.path().join("trace.jsonl");

    let ctx = new_trace_context();
    ctx.init_trace(Some(trace_path.clone())).unwrap();

    let span = ctx
        .create_span_builder()
        .with_command("echo policy-meta")
        .with_cwd("/tmp")
        .start()
        .unwrap();
    let span_id = span.get_span_id().to_string();
    span.finish(0, vec![], None).unwrap();

    let records: Vec<Value> = std::fs::read_to_string(&trace_path)
        .unwrap()
        .lines()
        .map(|line| serde_json::from_str::<Value>(line).unwrap())
        .collect();

    let completion = records
        .iter()
        .find(|value| {
            value.get("span_id").and_then(|v| v.as_str()) == Some(span_id.as_str())
                && value.get("event_type").and_then(|v| v.as_str()) == Some("command_complete")
        })
        .expect("expected a command_complete record");

    assert_eq!(
        completion
            .get("policy_resolution_mode")
            .and_then(|v| v.as_str()),
        Some("legacy_local")
    );
    assert!(
        completion.get("policy_snapshot_schema").is_none(),
        "policy_snapshot_schema should be omitted when not using snapshots"
    );
    assert!(
        completion.get("policy_snapshot_hash").is_none(),
        "policy_snapshot_hash should be omitted when not using snapshots"
    );
}

#[test]
fn command_complete_includes_policy_snapshot_meta_when_set() {
    let tmp_dir = TempDir::new().unwrap();
    let trace_path = tmp_dir.path().join("trace.jsonl");

    let ctx = new_trace_context();
    ctx.init_trace(Some(trace_path.clone())).unwrap();

    let mut span = ctx
        .create_span_builder()
        .with_command("echo policy-snapshot")
        .with_cwd("/tmp")
        .start()
        .unwrap();
    let span_id = span.get_span_id().to_string();

    span.set_policy_snapshot_meta(1, "deadbeef".to_string());
    span.finish(0, vec![], None).unwrap();

    let records: Vec<Value> = std::fs::read_to_string(&trace_path)
        .unwrap()
        .lines()
        .map(|line| serde_json::from_str::<Value>(line).unwrap())
        .collect();

    let completion = records
        .iter()
        .find(|value| {
            value.get("span_id").and_then(|v| v.as_str()) == Some(span_id.as_str())
                && value.get("event_type").and_then(|v| v.as_str()) == Some("command_complete")
        })
        .expect("expected a command_complete record");

    assert_eq!(
        completion
            .get("policy_resolution_mode")
            .and_then(|v| v.as_str()),
        Some("snapshot_v1")
    );
    assert_eq!(
        completion
            .get("policy_snapshot_schema")
            .and_then(|v| v.as_u64()),
        Some(1)
    );
    assert_eq!(
        completion
            .get("policy_snapshot_hash")
            .and_then(|v| v.as_str()),
        Some("deadbeef")
    );
}

#[test]
fn command_complete_parent_span_is_not_self_when_env_mutated() {
    let _guard = ENV_LOCK.lock().unwrap();
    let previous = std::env::var_os("SHIM_PARENT_SPAN");
    std::env::remove_var("SHIM_PARENT_SPAN");

    let tmp_dir = TempDir::new().unwrap();
    let trace_path = tmp_dir.path().join("trace.jsonl");
    let ctx = new_trace_context();
    ctx.init_trace(Some(trace_path.clone())).unwrap();

    let span = ctx
        .create_span_builder()
        .with_command("echo parent-span")
        .with_cwd("/tmp")
        .start()
        .unwrap();
    let span_id = span.get_span_id().to_string();

    std::env::set_var("SHIM_PARENT_SPAN", &span_id);
    span.finish(0, vec![], None).unwrap();

    let records: Vec<Value> = std::fs::read_to_string(&trace_path)
        .unwrap()
        .lines()
        .map(|line| serde_json::from_str::<Value>(line).unwrap())
        .collect();

    let completion = records
        .iter()
        .find(|value| {
            value.get("span_id").and_then(|v| v.as_str()) == Some(span_id.as_str())
                && value.get("event_type").and_then(|v| v.as_str()) == Some("command_complete")
        })
        .expect("expected a command_complete record");

    assert!(
        completion.get("parent_span").is_none() || completion.get("parent_span").unwrap().is_null(),
        "expected command_complete parent_span to be omitted/null, not self"
    );

    match previous {
        Some(value) => std::env::set_var("SHIM_PARENT_SPAN", value),
        None => std::env::remove_var("SHIM_PARENT_SPAN"),
    }
}

#[test]
fn command_complete_parent_span_is_captured_at_start() {
    let _guard = ENV_LOCK.lock().unwrap();
    let previous = std::env::var_os("SHIM_PARENT_SPAN");
    std::env::set_var("SHIM_PARENT_SPAN", "spn_parent_test");

    let tmp_dir = TempDir::new().unwrap();
    let trace_path = tmp_dir.path().join("trace.jsonl");
    let ctx = new_trace_context();
    ctx.init_trace(Some(trace_path.clone())).unwrap();

    let span = ctx
        .create_span_builder()
        .with_command("echo capture-parent")
        .with_cwd("/tmp")
        .start()
        .unwrap();
    let span_id = span.get_span_id().to_string();

    std::env::set_var("SHIM_PARENT_SPAN", &span_id);
    span.finish(0, vec![], None).unwrap();

    let records: Vec<Value> = std::fs::read_to_string(&trace_path)
        .unwrap()
        .lines()
        .map(|line| serde_json::from_str::<Value>(line).unwrap())
        .collect();

    let completion = records
        .iter()
        .find(|value| {
            value.get("span_id").and_then(|v| v.as_str()) == Some(span_id.as_str())
                && value.get("event_type").and_then(|v| v.as_str()) == Some("command_complete")
        })
        .expect("expected a command_complete record");

    assert_eq!(
        completion.get("parent_span").and_then(|v| v.as_str()),
        Some("spn_parent_test")
    );

    match previous {
        Some(value) => std::env::set_var("SHIM_PARENT_SPAN", value),
        None => std::env::remove_var("SHIM_PARENT_SPAN"),
    }
}
