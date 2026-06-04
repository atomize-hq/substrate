#![allow(unused_crate_dependencies)]

mod support;

use agent_drift_analyzer::{AnalyzeRequest, DriftState};
use agent_session_compactor::{CompactionKind, CompactionRow, SourceKind};
use camino::Utf8PathBuf;
use support::{
    analyze_clean_recovery_bundle, analyze_sample_bundle, load_sample_bundle, read_checkpoints,
    BundleFixture,
};

#[test]
fn dead_end_thrash_stays_active_when_verification_interval_remains_out_of_scope() {
    let result = analyze_sample_bundle();
    let checkpoints = &result.sessions[0].checkpoints;
    let first = checkpoints[0]
        .drift_scores
        .iter()
        .find(|score| score.class == agent_drift_analyzer::DriftClass::DeadEndThrash)
        .expect("first dead end thrash score");
    let second = checkpoints[1]
        .drift_scores
        .iter()
        .find(|score| score.class == agent_drift_analyzer::DriftClass::DeadEndThrash)
        .expect("second dead end thrash score");

    assert!(first.raw_score >= 60);
    assert!(first.flagged);
    assert_eq!(first.state, DriftState::Active);
    assert!(first.evidence.len() >= 3);
    assert!(first
        .evidence
        .iter()
        .any(|item| item.reason == "repeated failure evidence"));

    assert!(second.raw_score >= 60);
    assert!(second.flagged);
    assert_eq!(second.state, DriftState::Active);
    assert!(second
        .evidence
        .iter()
        .any(|item| item.reason == "repeated failure evidence"));
    assert!(second
        .evidence
        .iter()
        .any(|item| item.reason.starts_with("repeated verification command:")));
}

#[test]
fn dead_end_thrash_clears_after_one_clean_in_scope_verification_interval() {
    let result = analyze_clean_recovery_bundle();
    let checkpoints = &result.sessions[0].checkpoints;
    let recovered = checkpoints[1]
        .drift_scores
        .iter()
        .find(|score| score.class == agent_drift_analyzer::DriftClass::DeadEndThrash)
        .expect("recovered dead end thrash score");

    assert_eq!(recovered.raw_score, 20);
    assert!(!recovered.flagged);
    assert_eq!(recovered.state, DriftState::Recovered);
    assert!(recovered.evidence.iter().any(|item| item
        .reason
        .starts_with("historical repeated failure evidence:")));
    assert!(recovered.evidence.iter().any(|item| item
        .reason
        .starts_with("historical repeated verification evidence:")));
}

#[test]
fn dead_end_thrash_does_not_clear_when_latest_repeat_exists_only_in_archival_rows() {
    let mut bundle = load_sample_bundle();
    bundle.archival_rows.retain(|row| row.event_index != 12);
    bundle.compact_rows.retain(|row| row.event_index != 12);

    let mut late_failure = bundle
        .archival_rows
        .iter()
        .find(|row| row.text == "error: failed to compile analyzer")
        .expect("sample failure row")
        .clone();
    late_failure.event_index = 10;
    late_failure.line_number = 11;
    late_failure.row_ordinal = 1;

    let insert_at = bundle
        .archival_rows
        .iter()
        .position(|row| row.event_index == 11 && row.row_ordinal == 0)
        .expect("cargo test row");
    bundle.archival_rows.insert(insert_at, late_failure);

    let fixture = BundleFixture::from_rows(
        bundle.archival_rows,
        bundle.compact_rows,
        bundle.dedupe_groups,
    );
    let result = agent_drift_analyzer::analyze_bundle(&AnalyzeRequest {
        input_dir: fixture.input_dir.clone(),
        output_dir: fixture.output_dir.clone(),
    })
    .expect("analyze archival-only repeat bundle");
    let archived_repeat = result.sessions[0].checkpoints[1]
        .drift_scores
        .iter()
        .find(|score| score.class == agent_drift_analyzer::DriftClass::DeadEndThrash)
        .expect("archival-only dead end thrash score");

    assert!(archived_repeat.flagged);
    assert!(archived_repeat.raw_score >= 60);
    assert_eq!(archived_repeat.state, DriftState::Active);
    assert!(archived_repeat.evidence.iter().any(|item| {
        item.reason == "repeated failure evidence"
            && item.row.event_index == 10
            && item.row.row_ordinal == 1
    }));
}

#[test]
fn dead_end_thrash_downgrades_to_historical_only_after_the_recovery_transition() {
    let fixture = BundleFixture::clean_recovery();
    let mut bundle =
        agent_drift_analyzer::input::load_bundle(&fixture.input_dir).expect("load clean bundle");
    bundle.archival_rows.push(CompactionRow {
        source_file: Utf8PathBuf::from("/tmp/session-alpha/rollout.jsonl"),
        source_kind: SourceKind::CodexRolloutJsonl,
        session_id: Some("session-alpha".to_string()),
        turn_id: Some("turn-001".to_string()),
        event_index: 13,
        line_number: 14,
        row_ordinal: 0,
        timestamp: None,
        kind: CompactionKind::AssistantMessage,
        user_message_role: None,
        dedupe_identity: None,
        text: "The earlier thrash is now only historical context.".to_string(),
        canonical_text: "The earlier thrash is now only historical context.".to_string(),
        text_hash_hex: "hash-13".to_string(),
    });
    bundle.archival_rows.push(CompactionRow {
        source_file: Utf8PathBuf::from("/tmp/session-alpha/rollout.jsonl"),
        source_kind: SourceKind::CodexRolloutJsonl,
        session_id: Some("session-alpha".to_string()),
        turn_id: Some("turn-001".to_string()),
        event_index: 14,
        line_number: 15,
        row_ordinal: 0,
        timestamp: None,
        kind: CompactionKind::ToolCall,
        user_message_role: None,
        dedupe_identity: Some(
            "{\"call_id\":\"call-2\",\"name\":\"functions.shell_command\",\"type\":\"function_call\"}"
                .to_string(),
        ),
        text: "{\"command\":\"cargo test -p agent-drift-analyzer -- --nocapture\",\"workdir\":\"/repo\"}"
            .to_string(),
        canonical_text:
            "{\"command\":\"cargo test -p agent-drift-analyzer -- --nocapture\",\"workdir\":\"/repo\"}"
                .to_string(),
        text_hash_hex: "hash-14".to_string(),
    });
    bundle.compact_rows.push(CompactionRow {
        source_file: Utf8PathBuf::from("/tmp/session-alpha/rollout.jsonl"),
        source_kind: SourceKind::CodexRolloutJsonl,
        session_id: Some("session-alpha".to_string()),
        turn_id: Some("turn-001".to_string()),
        event_index: 13,
        line_number: 14,
        row_ordinal: 0,
        timestamp: None,
        kind: CompactionKind::AssistantMessage,
        user_message_role: None,
        dedupe_identity: None,
        text: "The earlier thrash is now only historical context.".to_string(),
        canonical_text: "The earlier thrash is now only historical context.".to_string(),
        text_hash_hex: "hash-13".to_string(),
    });
    bundle.compact_rows.push(CompactionRow {
        source_file: Utf8PathBuf::from("/tmp/session-alpha/rollout.jsonl"),
        source_kind: SourceKind::CodexRolloutJsonl,
        session_id: Some("session-alpha".to_string()),
        turn_id: Some("turn-001".to_string()),
        event_index: 14,
        line_number: 15,
        row_ordinal: 0,
        timestamp: None,
        kind: CompactionKind::ToolCall,
        user_message_role: None,
        dedupe_identity: Some(
            "{\"call_id\":\"call-2\",\"name\":\"functions.shell_command\",\"type\":\"function_call\"}"
                .to_string(),
        ),
        text: "{\"command\":\"cargo test -p agent-drift-analyzer -- --nocapture\",\"workdir\":\"/repo\"}"
            .to_string(),
        canonical_text:
            "{\"command\":\"cargo test -p agent-drift-analyzer -- --nocapture\",\"workdir\":\"/repo\"}"
                .to_string(),
        text_hash_hex: "hash-14".to_string(),
    });

    let fixture = BundleFixture::from_rows(
        bundle.archival_rows,
        bundle.compact_rows,
        bundle.dedupe_groups,
    );
    let result = agent_drift_analyzer::analyze_bundle(&AnalyzeRequest {
        input_dir: fixture.input_dir.clone(),
        output_dir: fixture.output_dir.clone(),
    })
    .expect("analyze dead-end historical-only bundle");
    let checkpoints = read_checkpoints(&result.checkpoints_path);
    let historical_only = checkpoints
        .last()
        .expect("historical checkpoint")
        .drift_scores
        .iter()
        .find(|score| score.class == agent_drift_analyzer::DriftClass::DeadEndThrash)
        .expect("historical dead end thrash score");

    assert!(checkpoints.len() >= 3);
    assert_eq!(historical_only.state, DriftState::HistoricalOnly);
    assert!(!historical_only.flagged);
    assert!(historical_only.evidence.iter().any(|item| item
        .reason
        .starts_with("historical repeated failure evidence:")));
}
