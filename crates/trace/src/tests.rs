use super::*;
use crate::context::{set_global_trace_context, TraceContext};
use crate::span::SpanBuilder;
use crate::util::hash_env_vars;
use chrono::Utc;
use serde_json::Value;
use std::sync::{Arc, Barrier};
use tempfile::TempDir;

fn ensure_trace_context() -> TraceContext {
    let ctx = TraceContext::default();
    let _ = set_global_trace_context(ctx.clone());
    ctx
}

#[test]
fn test_span_creation() {
    let span_id = new_span(None);
    assert!(span_id.starts_with("spn_"));
}

#[test]
fn test_span_builder() {
    ensure_trace_context();

    let span = create_span_builder()
        .unwrap()
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

    ensure_trace_context();
    let result = init_trace(Some(trace_path.clone()));
    assert!(result.is_ok());
    assert!(trace_path.exists());
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

    let ctx = ensure_trace_context();
    ctx.init_trace(Some(log_path.clone())).unwrap();

    let entry = serde_json::json!({"event_type":"test","ts":Utc::now().to_rfc3339()});
    append_to_trace(&entry).unwrap();

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

    let ctx = ensure_trace_context();
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
