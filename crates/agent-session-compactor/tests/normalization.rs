use anyhow as _;
use blake3 as _;
use std::fs;

use agent_session_compactor::ingest::ingest_rollout_file;
use agent_session_compactor::normalize::{normalize_rollout_file, CompactionKind};
use camino::Utf8Path;
use clap as _;
use codex as _;
use serde as _;
use serde_json as _;
use tempfile::TempDir;
use thiserror as _;
use time as _;
use walkdir as _;

#[test]
fn normalization_maps_rollout_events_into_provenance_preserving_rows() {
    let temp_dir = TempDir::new().expect("temp dir");
    let rollout_path = temp_dir.path().join("rollout-normalization.jsonl");
    fs::write(
        &rollout_path,
        concat!(
            "{\"timestamp\":\"2026-05-29T12:00:00Z\",\"type\":\"session_meta\",\"payload\":{\"id\":\"session-123\",\"base_instructions\":{\"text\":\"Base instructions\"}}}\n",
            "{\"timestamp\":\"2026-05-29T12:00:01Z\",\"type\":\"turn_context\",\"payload\":{\"turn_id\":\"turn-abc\",\"user_instructions\":\"Repo-local rules\"}}\n",
            "{\"timestamp\":\"2026-05-29T12:00:02Z\",\"type\":\"event_msg\",\"payload\":{\"type\":\"task_started\",\"turn_id\":\"turn-abc\"}}\n",
            "{\"timestamp\":\"2026-05-29T12:00:03Z\",\"type\":\"event_msg\",\"payload\":{\"type\":\"user_message\",\"message\":\"Ship the packet\"}}\n",
            "{\"timestamp\":\"2026-05-29T12:00:04Z\",\"type\":\"response_item\",\"payload\":{\"type\":\"reasoning\",\"summary\":[{\"type\":\"summary_text\",\"text\":\"Check parser seams first\"}]}}\n",
            "{\"timestamp\":\"2026-05-29T12:00:05Z\",\"type\":\"response_item\",\"payload\":{\"type\":\"function_call\",\"name\":\"exec_command\",\"arguments\":\"{\\\"cmd\\\":\\\"pwd\\\"}\",\"call_id\":\"call-1\"}}\n",
            "{\"timestamp\":\"2026-05-29T12:00:06Z\",\"type\":\"response_item\",\"payload\":{\"type\":\"function_call_output\",\"call_id\":\"call-1\",\"output\":\"/tmp/worktree\"}}\n",
            "{\"timestamp\":\"2026-05-29T12:00:07Z\",\"type\":\"event_msg\",\"payload\":{\"type\":\"token_count\",\"info\":{\"total\":1}}}\n",
            "{\"timestamp\":\"2026-05-29T12:00:08Z\",\"type\":\"response_item\",\"payload\":{\"type\":\"message\",\"role\":\"assistant\",\"content\":[{\"type\":\"output_text\",\"text\":\"Packet complete\"}]}}\n",
            "{\"type\":\"response_item\",\"payload\":\n"
        ),
    )
    .expect("write normalization fixture");

    let ingested = ingest_rollout_file(
        Utf8Path::from_path(&rollout_path).expect("rollout path should be valid UTF-8"),
    )
    .expect("ingest rollout");
    let rows = normalize_rollout_file(&ingested);

    let kinds: Vec<_> = rows.iter().map(|row| row.kind).collect();
    assert_eq!(
        kinds,
        vec![
            CompactionKind::SystemMessage,
            CompactionKind::SystemMessage,
            CompactionKind::Status,
            CompactionKind::UserMessage,
            CompactionKind::Reasoning,
            CompactionKind::ToolCall,
            CompactionKind::ToolOutput,
            CompactionKind::AssistantMessage,
            CompactionKind::Error,
        ]
    );
    assert_eq!(rows[0].turn_id, None);
    assert_eq!(rows[1].turn_id.as_deref(), Some("turn-abc"));
    assert_eq!(rows[3].turn_id.as_deref(), Some("turn-abc"));
    assert_eq!(rows[3].text, "Ship the packet");
    assert_eq!(rows[4].text, "Check parser seams first");
    assert_eq!(rows[5].text, "{\"cmd\":\"pwd\"}");
    assert_eq!(rows[6].text, "/tmp/worktree");
    assert_eq!(rows[7].text, "Packet complete");
    assert_eq!(rows[8].kind, CompactionKind::Error);
    assert!(rows[8].text.contains("failed to parse codex rollout JSONL"));
    assert_eq!(rows[7].line_number, 9);
    assert_eq!(rows[8].event_index, 9);
    assert!(!rows.iter().any(|row| row.text.contains("token_count")));
    assert!(rows[3].canonical_text.contains("Ship the packet"));
    assert_eq!(rows[3].text_hash_hex.len(), 64);
}
