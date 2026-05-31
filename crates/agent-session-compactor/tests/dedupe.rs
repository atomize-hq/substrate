use std::fs;

use anyhow as _;
use blake3 as _;

use agent_session_compactor::canonicalize::canonicalize_row_text;
use agent_session_compactor::dedupe::dedupe_rows_exact;
use agent_session_compactor::ingest::ingest_rollout_file;
use agent_session_compactor::normalize::{
    CompactionKind, CompactionRow, SourceKind, UserMessageRole,
};
use camino::Utf8PathBuf;
use clap as _;
use codex as _;
use serde as _;
use serde_json as _;
use tempfile::TempDir;
use thiserror as _;
use time as _;
use time::macros::datetime;
use walkdir as _;

#[test]
fn dedupe_keeps_first_exact_match_per_kind_and_emits_audit_group() {
    let first = row(
        "/tmp/rollout-a.jsonl",
        1,
        0,
        CompactionKind::AssistantMessage,
        "Hello\r\nWorld  ",
    );
    let duplicate = row(
        "/tmp/rollout-b.jsonl",
        2,
        1,
        CompactionKind::AssistantMessage,
        "\u{1b}[32mHello\u{1b}[0m\nWorld",
    );
    let different_kind = row(
        "/tmp/rollout-c.jsonl",
        3,
        2,
        CompactionKind::UserMessage,
        "Hello\nWorld",
    );
    let distinct = row(
        "/tmp/rollout-d.jsonl",
        4,
        3,
        CompactionKind::ToolOutput,
        "different",
    );

    let result = dedupe_rows_exact(&[first.clone(), duplicate, different_kind.clone(), distinct]);

    assert_eq!(result.archival_rows.len(), 4);
    assert_eq!(result.compact_rows.len(), 3);
    assert_eq!(result.compact_rows[0].source_file, first.source_file);
    assert_eq!(result.compact_rows[1].kind, different_kind.kind);
    assert_eq!(result.dedupe_groups.len(), 1);
    assert_eq!(
        result.dedupe_groups[0].representative.source_file,
        first.source_file
    );
    assert_eq!(result.dedupe_groups[0].duplicates.len(), 1);
    assert_eq!(
        result.dedupe_groups[0].duplicates[0].source_file,
        Utf8PathBuf::from("/tmp/rollout-b.jsonl")
    );
}

#[test]
fn dedupe_preserves_distinct_tool_events_with_matching_visible_text() {
    let temp_dir = TempDir::new().expect("temp dir");
    let rollout_path = temp_dir.path().join("rollout-dedupe.jsonl");
    fs::write(
        &rollout_path,
        concat!(
            "{\"type\":\"session_meta\",\"payload\":{\"id\":\"session-123\"}}\n",
            "{\"timestamp\":\"2026-05-29T12:00:00Z\",\"type\":\"response_item\",\"payload\":{\"type\":\"function_call\",\"name\":\"shell\",\"arguments\":\"{\\\"cmd\\\":\\\"pwd\\\"}\",\"call_id\":\"call-1\"}}\n",
            "{\"timestamp\":\"2026-05-29T12:00:01Z\",\"type\":\"response_item\",\"payload\":{\"type\":\"function_call\",\"name\":\"shell\",\"arguments\":\"{\\\"cmd\\\":\\\"pwd\\\"}\",\"call_id\":\"call-2\"}}\n",
            "{\"timestamp\":\"2026-05-29T12:00:02Z\",\"type\":\"response_item\",\"payload\":{\"type\":\"function_call_output\",\"output\":\"/tmp/worktree\",\"call_id\":\"call-1\"}}\n",
            "{\"timestamp\":\"2026-05-29T12:00:03Z\",\"type\":\"response_item\",\"payload\":{\"type\":\"function_call_output\",\"output\":\"/tmp/worktree\",\"call_id\":\"call-2\"}}\n"
        ),
    )
    .expect("write rollout fixture");

    let ingested = ingest_rollout_file(
        camino::Utf8Path::from_path(&rollout_path).expect("rollout path should be valid UTF-8"),
    )
    .expect("ingest rollout");
    let rows = agent_session_compactor::normalize::normalize_rollout_file(&ingested);
    let result = dedupe_rows_exact(&rows);

    assert_eq!(rows.len(), 4);
    assert_eq!(result.archival_rows.len(), 4);
    assert_eq!(result.compact_rows.len(), 4);
    assert!(result.dedupe_groups.is_empty());
}

#[test]
fn dedupe_keeps_user_rows_with_matching_text_when_roles_differ() {
    let mut prompt = row(
        "/tmp/rollout-a.jsonl",
        1,
        0,
        CompactionKind::UserMessage,
        "/goal Ship the packet",
    );
    prompt.user_message_role = Some(UserMessageRole::Prompt);

    let mut steer = row(
        "/tmp/rollout-b.jsonl",
        2,
        1,
        CompactionKind::UserMessage,
        "/goal Ship the packet",
    );
    steer.user_message_role = Some(UserMessageRole::Steer);

    let result = dedupe_rows_exact(&[prompt.clone(), steer.clone()]);

    assert_eq!(result.archival_rows.len(), 2);
    assert_eq!(result.compact_rows.len(), 2);
    assert!(result.dedupe_groups.is_empty());
    assert_eq!(
        result.compact_rows[0].user_message_role,
        prompt.user_message_role
    );
    assert_eq!(
        result.compact_rows[1].user_message_role,
        steer.user_message_role
    );
}

fn row(
    path: &str,
    line_number: usize,
    event_index: usize,
    kind: CompactionKind,
    text: &str,
) -> CompactionRow {
    let (canonical_text, text_hash_hex) = canonicalize_row_text(text);
    CompactionRow {
        source_file: Utf8PathBuf::from(path),
        source_kind: SourceKind::CodexRolloutJsonl,
        session_id: Some("session-123".to_string()),
        turn_id: Some("turn-abc".to_string()),
        event_index,
        line_number,
        row_ordinal: 0,
        timestamp: Some(datetime!(2026-05-29 12:00:00 UTC)),
        kind,
        user_message_role: None,
        dedupe_identity: None,
        text: text.to_string(),
        canonical_text,
        text_hash_hex,
    }
}
