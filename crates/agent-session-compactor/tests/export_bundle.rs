use anyhow as _;
use blake3 as _;
use std::fs;

use agent_session_compactor::canonicalize::canonicalize_row_text;
use agent_session_compactor::dedupe::dedupe_rows_exact;
use agent_session_compactor::export::{export_bundle, ExportBundleRequest};
use agent_session_compactor::normalize::{CompactionKind, CompactionRow, SourceKind};
use camino::{Utf8Path, Utf8PathBuf};
use clap as _;
use codex as _;
use serde as _;
use serde_json as _;
use tempfile::TempDir;
use thiserror as _;
use time::macros::datetime;
use time as _;
use walkdir as _;

#[test]
fn export_bundle_writes_manifest_rows_audit_and_summary() {
    let temp_dir = TempDir::new().expect("temp dir");
    let output_dir = Utf8Path::from_path(temp_dir.path()).expect("utf8 temp path");
    let archival_rows = vec![
        row("/tmp/rollout-a.jsonl", 1, 0, CompactionKind::UserMessage, "user"),
        row(
            "/tmp/rollout-b.jsonl",
            2,
            1,
            CompactionKind::AssistantMessage,
            "assistant",
        ),
        row(
            "/tmp/rollout-c.jsonl",
            3,
            2,
            CompactionKind::AssistantMessage,
            "\u{1b}[31massistant\u{1b}[0m",
        ),
    ];
    let dedupe_result = dedupe_rows_exact(&archival_rows);
    let manifest = export_bundle(&ExportBundleRequest {
        codex_home: Utf8Path::new("/tmp/.codex"),
        output_dir,
        generated_at: datetime!(2026-05-29 12:00:00 UTC),
        session_ids: vec!["session-123".to_string()],
        source_files: dedupe_result
            .archival_rows
            .iter()
            .map(|row| row.source_file.clone())
            .collect(),
        archival_rows: &dedupe_result.archival_rows,
        compact_rows: &dedupe_result.compact_rows,
        dedupe_groups: &dedupe_result.dedupe_groups,
    })
    .expect("export bundle");

    assert_eq!(manifest.archival_row_count, 3);
    assert_eq!(manifest.compact_row_count, 2);
    assert_eq!(manifest.dedupe_group_count, 1);
    assert!(output_dir.join("manifest.json").exists());
    assert!(output_dir.join("rows.archival.jsonl").exists());
    assert!(output_dir.join("rows.compact.jsonl").exists());
    assert!(output_dir.join("dedupe-audit.jsonl").exists());
    assert!(output_dir.join("summary.md").exists());

    let manifest_json = fs::read_to_string(output_dir.join("manifest.json")).expect("manifest");
    assert!(manifest_json.contains("\"schema_version\": \"v0.1\""));

    let compact_rows = fs::read_to_string(output_dir.join("rows.compact.jsonl")).expect("compact");
    assert_eq!(compact_rows.lines().count(), 2);

    let audit_rows = fs::read_to_string(output_dir.join("dedupe-audit.jsonl")).expect("audit");
    assert_eq!(audit_rows.lines().count(), 1);

    let summary = fs::read_to_string(output_dir.join("summary.md")).expect("summary");
    assert!(summary.contains("Archival rows: `3`"));
    assert!(summary.contains("Dedupe groups: `1`"));
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
