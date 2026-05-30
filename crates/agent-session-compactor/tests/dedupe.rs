use anyhow as _;
use blake3 as _;

use agent_session_compactor::canonicalize::canonicalize_row_text;
use agent_session_compactor::dedupe::dedupe_rows_exact;
use agent_session_compactor::normalize::{CompactionKind, CompactionRow, SourceKind};
use camino::Utf8PathBuf;
use clap as _;
use codex as _;
use serde as _;
use serde_json as _;
use tempfile as _;
use thiserror as _;
use time::macros::datetime;
use time as _;
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
    assert_eq!(result.dedupe_groups[0].representative.source_file, first.source_file);
    assert_eq!(result.dedupe_groups[0].duplicates.len(), 1);
    assert_eq!(
        result.dedupe_groups[0].duplicates[0].source_file,
        Utf8PathBuf::from("/tmp/rollout-b.jsonl")
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
        timestamp: Some(datetime!(2026-05-29 12:00:00 UTC)),
        kind,
        text: text.to_string(),
        canonical_text,
        text_hash_hex,
    }
}
