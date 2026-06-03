#![allow(unused_crate_dependencies)]

mod support;

use agent_drift_analyzer::{AnalyzeRequest, DriftClass};
use agent_session_compactor::{
    CompactionKind, CompactionRow, DedupeGroup, RowRef, SourceKind, UserMessageRole,
};
use camino::Utf8PathBuf;
use support::{analyze_sample_bundle, read_checkpoints, BundleFixture};

#[test]
fn truth_grounding_gap_flags_verification_without_truth_reads() {
    let result = analyze_sample_bundle();
    let score = result.sessions[0]
        .checkpoints
        .iter()
        .flat_map(|checkpoint| checkpoint.drift_scores.iter())
        .find(|score| score.class == DriftClass::TruthGroundingGap && score.flagged)
        .expect("truth grounding gap score");

    assert!(score.raw_score >= 60);
    assert!(score.flagged);
}

#[test]
fn truth_grounding_gap_preserves_history_without_keeping_the_latest_interval_active() {
    let rows = vec![
        row(
            0,
            CompactionKind::UserMessage,
            "/goal Update crates/agent-drift-analyzer/src/lib.rs using docs/specs/agent-drift-analyzer-v0.4-spec.md and verify with `cargo test -p agent-drift-analyzer -- --nocapture`.",
        ),
        row(
            1,
            CompactionKind::SystemMessage,
            "Read docs/specs/agent-drift-analyzer-v0.4-spec.md before acting.",
        ),
        tool_row(2, "cargo test -p agent-drift-analyzer -- --nocapture"),
        row(
            3,
            CompactionKind::AssistantMessage,
            "I need to re-ground on the spec before editing again.",
        ),
        tool_row(4, "sed -n '1,120p' docs/specs/agent-drift-analyzer-v0.4-spec.md"),
        tool_row(
            5,
            "apply_patch <<'PATCH'\n*** Begin Patch\n*** Update File: crates/agent-drift-analyzer/src/lib.rs\n*** End Patch\nPATCH",
        ),
    ];
    let mut archival_rows = rows.clone();
    let mut duplicate = archival_rows[2].clone();
    duplicate.row_ordinal = 1;
    duplicate.line_number += 100;
    archival_rows.push(duplicate.clone());
    let dedupe_groups = vec![DedupeGroup {
        kind: CompactionKind::ToolCall,
        canonical_text_hash_hex: "truth-grounding-gap-dup".to_string(),
        representative: RowRef::from_row(&rows[2]),
        duplicates: vec![RowRef::from_row(&duplicate)],
    }];
    let fixture = BundleFixture::from_rows(archival_rows, rows, dedupe_groups);
    let result = agent_drift_analyzer::analyze_bundle(&AnalyzeRequest {
        input_dir: fixture.input_dir.clone(),
        output_dir: fixture.output_dir.clone(),
    })
    .expect("analyze custom bundle");
    let checkpoints = read_checkpoints(&result.checkpoints_path);

    assert_eq!(checkpoints.len(), 2);
    let first = checkpoints[0]
        .drift_scores
        .iter()
        .find(|score| score.class == DriftClass::TruthGroundingGap)
        .expect("first truth grounding gap score");
    let second = checkpoints[1]
        .drift_scores
        .iter()
        .find(|score| score.class == DriftClass::TruthGroundingGap)
        .expect("second truth grounding gap score");

    assert!(first.flagged);
    assert!(!second.flagged);
    assert!(second
        .evidence
        .iter()
        .any(|evidence| evidence
            .reason
            .starts_with("historical truth-grounding gap:")));
}

#[test]
fn truth_grounding_gap_does_not_turn_clean_grounding_into_historical_gap_evidence() {
    let rows = vec![
        row(
            0,
            CompactionKind::UserMessage,
            "/goal Update crates/agent-drift-analyzer/src/lib.rs using docs/specs/agent-drift-analyzer-v0.4-spec.md and verify with `cargo test -p agent-drift-analyzer -- --nocapture`.",
        ),
        row(
            1,
            CompactionKind::SystemMessage,
            "Read docs/specs/agent-drift-analyzer-v0.4-spec.md before acting.",
        ),
        tool_row(2, "sed -n '1,120p' docs/specs/agent-drift-analyzer-v0.4-spec.md"),
        row(
            3,
            CompactionKind::AssistantMessage,
            "The spec is grounded; I am moving to the next checkpoint.",
        ),
        tool_row(4, "sed -n '1,120p' docs/specs/agent-drift-analyzer-v0.4-spec.md"),
        tool_row(
            5,
            "apply_patch <<'PATCH'\n*** Begin Patch\n*** Update File: crates/agent-drift-analyzer/src/lib.rs\n*** End Patch\nPATCH",
        ),
    ];
    let mut archival_rows = rows.clone();
    let mut duplicate = archival_rows[2].clone();
    duplicate.row_ordinal = 1;
    duplicate.line_number += 100;
    archival_rows.push(duplicate.clone());
    let dedupe_groups = vec![DedupeGroup {
        kind: CompactionKind::ToolCall,
        canonical_text_hash_hex: "truth-grounding-gap-clean-dup".to_string(),
        representative: RowRef::from_row(&rows[2]),
        duplicates: vec![RowRef::from_row(&duplicate)],
    }];
    let fixture = BundleFixture::from_rows(archival_rows, rows, dedupe_groups);
    let result = agent_drift_analyzer::analyze_bundle(&AnalyzeRequest {
        input_dir: fixture.input_dir.clone(),
        output_dir: fixture.output_dir.clone(),
    })
    .expect("analyze custom bundle");
    let checkpoints = read_checkpoints(&result.checkpoints_path);

    assert_eq!(checkpoints.len(), 2);
    let first = checkpoints[0]
        .drift_scores
        .iter()
        .find(|score| score.class == DriftClass::TruthGroundingGap)
        .expect("first truth grounding gap score");
    let second = checkpoints[1]
        .drift_scores
        .iter()
        .find(|score| score.class == DriftClass::TruthGroundingGap)
        .expect("second truth grounding gap score");

    assert!(!first.flagged);
    assert!(!second.flagged);
    assert!(first
        .evidence
        .iter()
        .any(|evidence| !evidence
            .reason
            .starts_with("historical truth-grounding gap:")));
    assert!(!second
        .evidence
        .iter()
        .any(|evidence| evidence
            .reason
            .starts_with("historical truth-grounding gap:")));
}

fn row(event_index: usize, kind: CompactionKind, text: &str) -> CompactionRow {
    CompactionRow {
        source_file: Utf8PathBuf::from("/tmp/session-alpha/rollout.jsonl"),
        source_kind: SourceKind::CodexRolloutJsonl,
        session_id: Some("session-alpha".to_string()),
        turn_id: Some("turn-001".to_string()),
        event_index,
        line_number: event_index + 1,
        row_ordinal: 0,
        timestamp: None,
        kind,
        user_message_role: matches!(kind, CompactionKind::UserMessage)
            .then_some(UserMessageRole::Prompt),
        dedupe_identity: None,
        text: text.to_string(),
        canonical_text: text.to_string(),
        text_hash_hex: format!("hash-{event_index}"),
    }
}

fn tool_row(event_index: usize, command: &str) -> CompactionRow {
    let payload = format!(
        "{{\"command\":{command:?},\"workdir\":\"/repo\"}}",
    );
    let mut row = row(event_index, CompactionKind::ToolCall, &payload);
    row.dedupe_identity = Some(
        "{\"call_id\":\"call-1\",\"name\":\"functions.shell_command\",\"type\":\"function_call\"}"
            .to_string(),
    );
    row
}
