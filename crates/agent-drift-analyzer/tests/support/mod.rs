#![allow(dead_code, unreachable_pub)]

use std::fs;

use agent_drift_analyzer::{analyze_bundle, AnalyzeRequest, Checkpoint, DriftClass, InputBundle};
use agent_session_compactor::{
    BundleManifest, CompactionKind, CompactionRow, DedupeGroup, RowRef, SourceKind,
};
use camino::{Utf8Path, Utf8PathBuf};
use tempfile::TempDir;

pub struct BundleFixture {
    pub _temp_dir: TempDir,
    pub input_dir: Utf8PathBuf,
    pub output_dir: Utf8PathBuf,
}

impl BundleFixture {
    pub fn sample() -> Self {
        Self::from_rows(sample_archival_rows(), sample_compact_rows(), sample_dedupe_groups())
    }

    pub fn from_rows(
        archival_rows: Vec<CompactionRow>,
        compact_rows: Vec<CompactionRow>,
        dedupe_groups: Vec<DedupeGroup>,
    ) -> Self {
        let temp_dir = TempDir::new().expect("temp dir");
        let root = Utf8Path::from_path(temp_dir.path()).expect("utf8 temp dir");
        let input_dir = root.join("input");
        let output_dir = root.join("output");
        fs::create_dir_all(&input_dir).expect("create input dir");

        let manifest = BundleManifest {
            schema_version: "v0.1".to_string(),
            generated_at: time::OffsetDateTime::from_unix_timestamp(1_717_000_000)
                .expect("timestamp"),
            codex_home: Utf8PathBuf::from("/tmp/.codex"),
            output_dir: input_dir.clone(),
            discovered_file_count: 1,
            archival_row_count: archival_rows.len(),
            compact_row_count: compact_rows.len(),
            dedupe_group_count: dedupe_groups.len(),
            session_ids: vec!["session-alpha".to_string()],
            source_files: vec![Utf8PathBuf::from("/tmp/session-alpha/rollout.jsonl")],
        };

        fs::write(
            input_dir.join("manifest.json"),
            serde_json::to_string_pretty(&manifest).expect("manifest json"),
        )
        .expect("write manifest");
        write_jsonl(input_dir.join("rows.archival.jsonl"), &archival_rows);
        write_jsonl(input_dir.join("rows.compact.jsonl"), &compact_rows);
        write_jsonl(input_dir.join("dedupe-audit.jsonl"), &dedupe_groups);

        Self {
            _temp_dir: temp_dir,
            input_dir,
            output_dir,
        }
    }
}

pub fn analyze_sample_bundle() -> agent_drift_analyzer::AnalyzeResult {
    let fixture = BundleFixture::sample();
    analyze_bundle(&AnalyzeRequest {
        input_dir: fixture.input_dir.clone(),
        output_dir: fixture.output_dir.clone(),
    })
    .expect("analyze sample bundle")
}

pub fn load_sample_bundle() -> InputBundle {
    let fixture = BundleFixture::sample();
    agent_drift_analyzer::input::load_bundle(&fixture.input_dir).expect("load bundle")
}

pub fn read_checkpoints(path: &Utf8Path) -> Vec<Checkpoint> {
    fs::read_to_string(path)
        .expect("read checkpoints")
        .lines()
        .map(|line| serde_json::from_str(line).expect("checkpoint json"))
        .collect()
}

pub fn assert_score_flagged(checkpoint: &Checkpoint, class: DriftClass) {
    assert!(
        checkpoint
            .drift_scores
            .iter()
            .any(|score| score.class == class && score.flagged),
        "expected {class:?} to be flagged"
    );
}

fn write_jsonl<T: serde::Serialize>(path: Utf8PathBuf, items: &[T]) {
    let body = items
        .iter()
        .map(|item| serde_json::to_string(item).expect("json line"))
        .collect::<Vec<_>>()
        .join("\n");
    fs::write(path, format!("{body}\n")).expect("write jsonl");
}

fn sample_archival_rows() -> Vec<CompactionRow> {
    vec![
        row(
            0,
            1,
            0,
            CompactionKind::UserMessage,
            "/goal Implement Packet 6 using docs/specs/agent-drift-analyzer-v0.1-plan.md and docs/specs/agent-drift-analyzer-v0.1-spec.md. Verify with `cargo build -p agent-drift-analyzer` and `cargo test -p agent-drift-analyzer -- --nocapture`.",
            None,
        ),
        row(
            1,
            2,
            0,
            CompactionKind::SystemMessage,
            "Use crates/agent-drift-analyzer/src/lib.rs and crates/agent-drift-analyzer/src/input.rs as the library-first surface.",
            None,
        ),
        tool_row(
            2,
            3,
            "{\"command\":\"sed -n '1,200p' README.md\",\"workdir\":\"/repo\"}",
            Some("{\"call_id\":\"call-1\",\"name\":\"functions.shell_command\",\"type\":\"function_call\"}"),
        ),
        tool_row(
            3,
            4,
            "{\"command\":\"cargo build -p agent-drift-analyzer\",\"workdir\":\"/repo\"}",
            Some("{\"call_id\":\"call-2\",\"name\":\"functions.shell_command\",\"type\":\"function_call\"}"),
        ),
        tool_row(
            4,
            5,
            "{\"command\":\"cargo build -p agent-drift-analyzer\",\"workdir\":\"/repo\"}",
            Some("{\"call_id\":\"call-3\",\"name\":\"functions.shell_command\",\"type\":\"function_call\"}"),
        ),
        tool_row(
            5,
            6,
            "{\"command\":\"cargo build -p agent-drift-analyzer\",\"workdir\":\"/repo\"}",
            Some("{\"call_id\":\"call-4\",\"name\":\"functions.shell_command\",\"type\":\"function_call\"}"),
        ),
        row(
            6,
            7,
            0,
            CompactionKind::ToolOutput,
            "error: failed to compile analyzer",
            None,
        ),
        row(
            7,
            8,
            0,
            CompactionKind::ToolOutput,
            "error: failed to compile analyzer",
            None,
        ),
        tool_row(
            8,
            9,
            "{\"command\":\"apply_patch <<'PATCH'\\n*** Begin Patch\\n*** Update File: crates/agent-drift-analyzer/src/lib.rs\\n*** End Patch\\nPATCH\",\"workdir\":\"/repo\"}",
            Some("{\"call_id\":\"call-5\",\"name\":\"functions.shell_command\",\"type\":\"function_call\"}"),
        ),
        tool_row(
            9,
            10,
            "{\"command\":\"sed -n '1,120p' crates/agent-drift-analyzer/src/lib.rs\",\"workdir\":\"/repo\"}",
            Some("{\"call_id\":\"call-6\",\"name\":\"functions.shell_command\",\"type\":\"function_call\"}"),
        ),
    ]
}

fn sample_compact_rows() -> Vec<CompactionRow> {
    sample_archival_rows()
        .into_iter()
        .enumerate()
        .filter(|(index, _)| !matches!(index, 4 | 7))
        .map(|(_, row)| row)
        .collect()
}

fn sample_dedupe_groups() -> Vec<DedupeGroup> {
    vec![DedupeGroup {
        kind: CompactionKind::ToolCall,
        canonical_text_hash_hex: "dup-hash".to_string(),
        representative: RowRef {
            source_file: Utf8PathBuf::from("/tmp/session-alpha/rollout.jsonl"),
            line_number: 4,
            event_index: 3,
            row_ordinal: 0,
        },
        duplicates: vec![RowRef {
            source_file: Utf8PathBuf::from("/tmp/session-alpha/rollout.jsonl"),
            line_number: 5,
            event_index: 4,
            row_ordinal: 0,
        }],
    }]
}

fn row(
    event_index: usize,
    line_number: usize,
    row_ordinal: usize,
    kind: CompactionKind,
    text: &str,
    dedupe_identity: Option<&str>,
) -> CompactionRow {
    CompactionRow {
        source_file: Utf8PathBuf::from("/tmp/session-alpha/rollout.jsonl"),
        source_kind: SourceKind::CodexRolloutJsonl,
        session_id: Some("session-alpha".to_string()),
        turn_id: Some("turn-001".to_string()),
        event_index,
        line_number,
        row_ordinal,
        timestamp: None,
        kind,
        dedupe_identity: dedupe_identity.map(ToOwned::to_owned),
        text: text.to_string(),
        canonical_text: text.to_string(),
        text_hash_hex: format!("hash-{}", text.split_whitespace().collect::<String>()),
    }
}

fn tool_row(
    event_index: usize,
    line_number: usize,
    text: &str,
    dedupe_identity: Option<&str>,
) -> CompactionRow {
    row(
        event_index,
        line_number,
        0,
        CompactionKind::ToolCall,
        text,
        dedupe_identity,
    )
}
