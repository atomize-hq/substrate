use anyhow as _;
use blake3 as _;
use std::fs;
use std::sync::Mutex;

use agent_session_compactor::canonicalize::canonicalize_row_text;
use agent_session_compactor::dedupe::{dedupe_rows_exact, DedupeGroup};
use agent_session_compactor::export::{export_bundle, ExportBundleRequest};
use agent_session_compactor::normalize::{
    CompactionKind, CompactionRow, SourceKind, UserMessageRole,
};
use camino::{Utf8Path, Utf8PathBuf};
use clap as _;
use codex as _;
use serde as _;
use serde_json as _;
use tempfile::TempDir;
use thiserror as _;
use time as _;
use time::macros::datetime;
use walkdir as _;

const EXPORT_FAIL_AT_ENV: &str = "AGENT_SESSION_COMPACTOR_EXPORT_FAIL_AT";
static EXPORT_ENV_MUTEX: Mutex<()> = Mutex::new(());

#[test]
fn export_bundle_writes_manifest_rows_audit_and_summary() {
    with_export_failure(None, || {
        let temp_dir = TempDir::new().expect("temp dir");
        let output_dir = Utf8Path::from_path(temp_dir.path())
            .expect("utf8 temp path")
            .join("bundle");
        let fixture = ExportFixture::sample();
        let manifest = export_bundle(&fixture.request(&output_dir)).expect("export bundle");

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

        let compact_rows =
            fs::read_to_string(output_dir.join("rows.compact.jsonl")).expect("compact");
        assert_eq!(compact_rows.lines().count(), 2);
        assert!(compact_rows
            .lines()
            .all(|line| !line.contains("\"canonical_text\"")));
        assert!(compact_rows
            .lines()
            .any(|line| line.contains("\"user_message_role\":\"prompt\"")));

        let archival_rows =
            fs::read_to_string(output_dir.join("rows.archival.jsonl")).expect("archival");
        assert!(archival_rows
            .lines()
            .all(|line| !line.contains("\"canonical_text\"")));

        let audit_rows = fs::read_to_string(output_dir.join("dedupe-audit.jsonl")).expect("audit");
        assert_eq!(audit_rows.lines().count(), 1);

        let summary = fs::read_to_string(output_dir.join("summary.md")).expect("summary");
        assert!(summary.contains("Archival rows: `3`"));
        assert!(summary.contains("Dedupe groups: `1`"));

        let exported_files = bundle_entries(&output_dir);
        assert_eq!(
            exported_files,
            vec![
                "dedupe-audit.jsonl".to_string(),
                "manifest.json".to_string(),
                "rows.archival.jsonl".to_string(),
                "rows.compact.jsonl".to_string(),
                "summary.md".to_string(),
            ]
        );
        assert!(staging_entries(
            Utf8Path::from_path(temp_dir.path()).expect("utf8 temp path"),
            "bundle"
        )
        .is_empty());
    });
}

#[test]
fn export_bundle_failure_before_manifest_leaves_only_hidden_staging_files() {
    let temp_dir = TempDir::new().expect("temp dir");
    let output_dir = Utf8Path::from_path(temp_dir.path())
        .expect("utf8 temp path")
        .join("bundle");
    let fixture = ExportFixture::sample();

    let error = with_export_failure(Some("before_manifest"), || {
        export_bundle(&fixture.request(&output_dir)).expect_err("failure before manifest")
    });

    assert!(matches!(
        error,
        agent_session_compactor::ExportError::InjectedFailure { .. }
    ));
    assert!(!output_dir.exists());

    let staging_dirs = staging_entries(
        Utf8Path::from_path(temp_dir.path()).expect("utf8 temp path"),
        "bundle",
    );
    assert_eq!(staging_dirs.len(), 1);

    let staging_dir = Utf8Path::from_path(&staging_dirs[0]).expect("utf8 staging path");
    assert!(staging_dir.join("rows.archival.jsonl").exists());
    assert!(staging_dir.join("rows.compact.jsonl").exists());
    assert!(staging_dir.join("dedupe-audit.jsonl").exists());
    assert!(staging_dir.join("summary.md").exists());
    assert!(!staging_dir.join("manifest.json").exists());
}

#[test]
fn export_bundle_failure_before_publish_keeps_complete_bundle_out_of_final_path() {
    let temp_dir = TempDir::new().expect("temp dir");
    let output_dir = Utf8Path::from_path(temp_dir.path())
        .expect("utf8 temp path")
        .join("bundle");
    let fixture = ExportFixture::sample();

    let error = with_export_failure(Some("before_publish"), || {
        export_bundle(&fixture.request(&output_dir)).expect_err("failure before publish")
    });

    assert!(matches!(
        error,
        agent_session_compactor::ExportError::InjectedFailure { .. }
    ));
    assert!(!output_dir.exists());

    let staging_dirs = staging_entries(
        Utf8Path::from_path(temp_dir.path()).expect("utf8 temp path"),
        "bundle",
    );
    assert_eq!(staging_dirs.len(), 1);
    let staging_dir = Utf8Path::from_path(&staging_dirs[0]).expect("utf8 staging path");
    assert_eq!(
        bundle_entries(staging_dir),
        vec![
            "dedupe-audit.jsonl".to_string(),
            "manifest.json".to_string(),
            "rows.archival.jsonl".to_string(),
            "rows.compact.jsonl".to_string(),
            "summary.md".to_string(),
        ]
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
        user_message_role: matches!(kind, CompactionKind::UserMessage)
            .then_some(UserMessageRole::Prompt),
        dedupe_identity: None,
        text: text.to_string(),
        canonical_text,
        text_hash_hex,
    }
}

struct ExportFixture {
    source_files: Vec<Utf8PathBuf>,
    archival_rows: Vec<CompactionRow>,
    compact_rows: Vec<CompactionRow>,
    dedupe_groups: Vec<DedupeGroup>,
}

impl ExportFixture {
    fn sample() -> Self {
        let archival_rows = vec![
            row(
                "/tmp/rollout-a.jsonl",
                1,
                0,
                CompactionKind::UserMessage,
                "user",
            ),
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
        Self {
            source_files: dedupe_result
                .archival_rows
                .iter()
                .map(|row| row.source_file.clone())
                .collect(),
            archival_rows: dedupe_result.archival_rows,
            compact_rows: dedupe_result.compact_rows,
            dedupe_groups: dedupe_result.dedupe_groups,
        }
    }

    fn request<'a>(&'a self, output_dir: &'a Utf8Path) -> ExportBundleRequest<'a> {
        ExportBundleRequest {
            codex_home: Utf8Path::new("/tmp/.codex"),
            output_dir,
            generated_at: datetime!(2026-05-29 12:00:00 UTC),
            session_ids: vec!["session-123".to_string()],
            source_files: self.source_files.clone(),
            archival_rows: &self.archival_rows,
            compact_rows: &self.compact_rows,
            dedupe_groups: &self.dedupe_groups,
        }
    }
}

fn bundle_entries(dir: &Utf8Path) -> Vec<String> {
    let mut entries = fs::read_dir(dir)
        .expect("read dir")
        .map(|entry| entry.expect("dir entry").file_name())
        .map(|entry| entry.into_string().expect("utf8 entry"))
        .collect::<Vec<_>>();
    entries.sort();
    entries
}

fn staging_entries(parent_dir: &Utf8Path, bundle_name: &str) -> Vec<std::path::PathBuf> {
    let mut entries = fs::read_dir(parent_dir)
        .expect("read parent dir")
        .map(|entry| entry.expect("dir entry").path())
        .filter(|path| {
            path.file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| name.starts_with(&format!(".{bundle_name}.staging-")))
        })
        .collect::<Vec<_>>();
    entries.sort();
    entries
}

fn with_export_failure<T>(point: Option<&str>, f: impl FnOnce() -> T) -> T {
    let _guard = EXPORT_ENV_MUTEX.lock().expect("env mutex");
    let previous = std::env::var(EXPORT_FAIL_AT_ENV).ok();
    match point {
        Some(point) => std::env::set_var(EXPORT_FAIL_AT_ENV, point),
        None => std::env::remove_var(EXPORT_FAIL_AT_ENV),
    }
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(f));
    match previous {
        Some(previous) => std::env::set_var(EXPORT_FAIL_AT_ENV, previous),
        None => std::env::remove_var(EXPORT_FAIL_AT_ENV),
    }
    match result {
        Ok(result) => result,
        Err(payload) => std::panic::resume_unwind(payload),
    }
}
