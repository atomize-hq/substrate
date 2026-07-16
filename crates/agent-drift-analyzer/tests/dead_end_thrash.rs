#![allow(unused_crate_dependencies)]

mod support;

use std::fs;

use agent_drift_analyzer::{
    AnalyzeRequest, Checkpoint, ChildWorkVisibility, Confidence, DelegationTopology, DriftClass,
    DriftScore, DriftState, ProgressDimension, ProgressSignalCode, ProgressStatus,
};
use agent_session_compactor::{
    BundleManifest, CompactionKind, CompactionRow, DedupeGroup, DelegationEvidenceRef,
    DelegationLink, DelegationLinkState, RowRef, SourceKind, UserMessageRole,
};
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

    assert_eq!(first.raw_score, 30);
    assert!(first.flagged);
    assert_eq!(first.state, DriftState::Active);
    assert!(first.evidence.len() >= 3);
    assert!(first
        .evidence
        .iter()
        .any(|item| item.reason == "repeated failure evidence"));
    assert!(first.evidence.iter().any(|item| item
        .reason
        .starts_with("historical repeated verification evidence:")));

    assert!(second.raw_score >= 60);
    assert!(second.flagged);
    assert_eq!(second.state, DriftState::Active);
    assert!(second
        .evidence
        .iter()
        .any(|item| item.reason == "repeated failure evidence"));
    assert!(second.evidence.iter().any(|item| item
        .reason
        .starts_with("stall without frontier movement evidence:")));
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
fn dead_end_thrash_is_cleared_when_checkpoint_has_no_history_and_no_thrash() {
    let rows = vec![
        row(
            0,
            CompactionKind::UserMessage,
            "/goal Update crates/agent-drift-analyzer/src/lib.rs using docs/specs/agent-drift-analyzer-v0.4-spec.md and verify with `cargo test -p agent-drift-analyzer dead_end_thrash -- --nocapture`.",
        ),
        row(
            1,
            CompactionKind::SystemMessage,
            "Stay inside crates/agent-drift-analyzer/src/lib.rs and the v0.4 spec.",
        ),
        tool_row(2, "sed -n '1,120p' docs/specs/agent-drift-analyzer-v0.4-spec.md"),
        tool_row(
            3,
            "apply_patch <<'PATCH'\n*** Begin Patch\n*** Update File: crates/agent-drift-analyzer/src/lib.rs\n*** End Patch\nPATCH",
        ),
        tool_row(4, "cargo test -p agent-drift-analyzer dead_end_thrash -- --nocapture"),
    ];
    let mut archival_rows = rows.clone();
    let mut duplicate = archival_rows[2].clone();
    duplicate.row_ordinal = 1;
    duplicate.line_number += 100;
    archival_rows.push(duplicate.clone());
    let dedupe_groups = vec![DedupeGroup {
        kind: CompactionKind::ToolCall,
        canonical_text_hash_hex: "no-thrash-dedup".to_string(),
        representative: RowRef::from_row(&rows[2]),
        duplicates: vec![RowRef::from_row(&duplicate)],
    }];
    let fixture = BundleFixture::from_rows(archival_rows, rows, dedupe_groups);
    let result = agent_drift_analyzer::analyze_bundle(&AnalyzeRequest {
        input_dir: fixture.input_dir.clone(),
        output_dir: fixture.output_dir.clone(),
    })
    .expect("analyze no-history dead-end bundle");
    let checkpoints = read_checkpoints(&result.checkpoints_path);
    let cleared = checkpoints[0]
        .drift_scores
        .iter()
        .find(|score| score.class == agent_drift_analyzer::DriftClass::DeadEndThrash)
        .expect("cleared dead end thrash score");

    assert_eq!(checkpoints.len(), 1);
    assert!(!cleared.flagged);
    assert_eq!(cleared.raw_score, 0);
    assert_eq!(cleared.state, DriftState::Cleared);
    assert!(cleared.evidence.is_empty());
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
    assert_eq!(archived_repeat.raw_score, 30);
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

#[test]
fn dead_end_thrash_treats_explicit_error_rows_as_repeated_failure_evidence() {
    let rows = vec![
        row(
            0,
            CompactionKind::UserMessage,
            "/goal Verify explicit error rows stay failure evidence.",
        ),
        tool_row(
            1,
            "cargo test -p agent-drift-analyzer dead_end_thrash -- --nocapture",
        ),
        tool_row(
            2,
            "cargo test -p agent-drift-analyzer dead_end_thrash -- --nocapture",
        ),
        tool_row(
            3,
            "cargo test -p agent-drift-analyzer dead_end_thrash -- --nocapture",
        ),
        row(4, CompactionKind::Error, "world failed"),
        row(5, CompactionKind::Error, "world failed"),
    ];
    let mut archival_rows = rows.clone();
    archival_rows[4].text_hash_hex = "hash-explicit-error".to_string();
    archival_rows[5].text_hash_hex = "hash-explicit-error".to_string();
    let compact_rows = archival_rows.clone();
    let dedupe_groups = vec![DedupeGroup {
        kind: CompactionKind::ToolCall,
        canonical_text_hash_hex: "dup-hash-explicit-error".to_string(),
        representative: RowRef::from_row(&archival_rows[1]),
        duplicates: vec![RowRef::from_row(&archival_rows[2])],
    }];
    let fixture = BundleFixture::from_rows(archival_rows, compact_rows, dedupe_groups);

    let result = agent_drift_analyzer::analyze_bundle(&AnalyzeRequest {
        input_dir: fixture.input_dir.clone(),
        output_dir: fixture.output_dir.clone(),
    })
    .expect("analyze explicit error bundle");
    let checkpoints = read_checkpoints(&result.checkpoints_path);
    let thrash = checkpoints[0]
        .drift_scores
        .iter()
        .find(|score| score.class == agent_drift_analyzer::DriftClass::DeadEndThrash)
        .expect("dead end thrash score");

    assert!(thrash.flagged);
    assert_eq!(thrash.raw_score, 30);
    assert!(thrash
        .evidence
        .iter()
        .any(|item| item.reason == "repeated failure evidence"));
}

#[test]
fn dead_end_thrash_stall_score_is_not_driven_by_repeated_failure_loop_count() {
    let base_rows = vec![
        row(
            0,
            CompactionKind::UserMessage,
            "/goal Verify a decisive stalled failure scores once.",
        ),
        row(1, CompactionKind::Error, "world failed"),
        row(2, CompactionKind::Error, "world failed"),
    ];
    let mut base_archival_rows = base_rows.clone();
    base_archival_rows[1].text_hash_hex = "hash-stall-base-world-failed".to_string();
    base_archival_rows[2].text_hash_hex = "hash-stall-base-world-failed".to_string();
    let base_fixture =
        BundleFixture::from_rows(base_archival_rows.clone(), base_archival_rows, Vec::new());
    let base_result = agent_drift_analyzer::analyze_bundle(&AnalyzeRequest {
        input_dir: base_fixture.input_dir.clone(),
        output_dir: base_fixture.output_dir.clone(),
    })
    .expect("analyze base decisive-stall bundle");
    let base_checkpoints = read_checkpoints(&base_result.checkpoints_path);
    let base_thrash = base_checkpoints[0]
        .drift_scores
        .iter()
        .find(|score| score.class == agent_drift_analyzer::DriftClass::DeadEndThrash)
        .expect("base dead end thrash score");

    let expanded_rows = vec![
        row(
            0,
            CompactionKind::UserMessage,
            "/goal Verify a decisive stalled failure does not scale with extra repeated loops.",
        ),
        row(1, CompactionKind::Error, "world failed"),
        row(2, CompactionKind::Error, "world failed"),
        row(3, CompactionKind::Error, "compile failed"),
        row(4, CompactionKind::Error, "compile failed"),
    ];
    let mut expanded_archival_rows = expanded_rows.clone();
    expanded_archival_rows[1].text_hash_hex = "hash-stall-expanded-world-failed".to_string();
    expanded_archival_rows[2].text_hash_hex = "hash-stall-expanded-world-failed".to_string();
    expanded_archival_rows[3].text_hash_hex = "hash-stall-expanded-compile-failed".to_string();
    expanded_archival_rows[4].text_hash_hex = "hash-stall-expanded-compile-failed".to_string();
    let expanded_fixture = BundleFixture::from_rows(
        expanded_archival_rows.clone(),
        expanded_archival_rows,
        Vec::new(),
    );
    let expanded_result = agent_drift_analyzer::analyze_bundle(&AnalyzeRequest {
        input_dir: expanded_fixture.input_dir.clone(),
        output_dir: expanded_fixture.output_dir.clone(),
    })
    .expect("analyze expanded decisive-stall bundle");
    let expanded_checkpoints = read_checkpoints(&expanded_result.checkpoints_path);
    let expanded_thrash = expanded_checkpoints[0]
        .drift_scores
        .iter()
        .find(|score| score.class == agent_drift_analyzer::DriftClass::DeadEndThrash)
        .expect("expanded dead end thrash score");

    assert!(base_thrash.flagged);
    assert!(expanded_thrash.flagged);
    assert_eq!(base_thrash.raw_score, 30);
    assert_eq!(expanded_thrash.raw_score, 30);
    assert_eq!(base_thrash.raw_score, expanded_thrash.raw_score);
}

#[test]
fn dead_end_thrash_treats_non_zero_exit_code_tool_output_as_failure_evidence() {
    let rows = vec![
        row(
            0,
            CompactionKind::UserMessage,
            "/goal Verify explicit non-zero exit codes stay failure evidence.",
        ),
        tool_row(
            1,
            "cargo test -p agent-drift-analyzer dead_end_thrash -- --nocapture",
        ),
        tool_row(
            2,
            "cargo test -p agent-drift-analyzer dead_end_thrash -- --nocapture",
        ),
        tool_row(
            3,
            "cargo test -p agent-drift-analyzer dead_end_thrash -- --nocapture",
        ),
        row(4, CompactionKind::ToolOutput, "Exit code: 101"),
        row(5, CompactionKind::ToolOutput, "Exit code: 101"),
    ];
    let mut archival_rows = rows.clone();
    archival_rows[4].text_hash_hex = "hash-exit-code-101".to_string();
    archival_rows[5].text_hash_hex = "hash-exit-code-101".to_string();
    let compact_rows = archival_rows.clone();
    let dedupe_groups = vec![DedupeGroup {
        kind: CompactionKind::ToolCall,
        canonical_text_hash_hex: "dup-hash-non-zero-exit".to_string(),
        representative: RowRef::from_row(&archival_rows[1]),
        duplicates: vec![RowRef::from_row(&archival_rows[2])],
    }];
    let fixture = BundleFixture::from_rows(archival_rows, compact_rows, dedupe_groups);

    let result = agent_drift_analyzer::analyze_bundle(&AnalyzeRequest {
        input_dir: fixture.input_dir.clone(),
        output_dir: fixture.output_dir.clone(),
    })
    .expect("analyze non-zero exit code bundle");
    let checkpoints = read_checkpoints(&result.checkpoints_path);
    let thrash = checkpoints[0]
        .drift_scores
        .iter()
        .find(|score| score.class == agent_drift_analyzer::DriftClass::DeadEndThrash)
        .expect("dead end thrash score");

    assert!(thrash.flagged);
    assert_eq!(thrash.raw_score, 30);
    assert!(thrash
        .evidence
        .iter()
        .any(|item| item.reason == "repeated failure evidence"));
}

#[test]
fn dead_end_thrash_ignores_repeated_neutral_tool_output_evidence() {
    let rows = vec![
        row(
            0,
            CompactionKind::UserMessage,
            "/goal Verify neutral tool output stays out of repeated failure loops.",
        ),
        tool_row(
            1,
            "cargo test -p agent-drift-analyzer dead_end_thrash -- --nocapture",
        ),
        tool_row(
            2,
            "cargo test -p agent-drift-analyzer dead_end_thrash -- --nocapture",
        ),
        tool_row(
            3,
            "cargo test -p agent-drift-analyzer dead_end_thrash -- --nocapture",
        ),
        row(4, CompactionKind::ToolOutput, "Exit code: 0"),
        row(5, CompactionKind::ToolOutput, "Exit code: 0"),
        row(6, CompactionKind::ToolOutput, "Plan updated"),
        row(7, CompactionKind::ToolOutput, "Plan updated"),
        row(
            8,
            CompactionKind::ToolOutput,
            "function_call_output: wrote analyzer patch",
        ),
        row(
            9,
            CompactionKind::ToolOutput,
            "function_call_output: wrote analyzer patch",
        ),
    ];
    let mut archival_rows = rows.clone();
    archival_rows[4].text_hash_hex = "hash-exit-zero".to_string();
    archival_rows[5].text_hash_hex = "hash-exit-zero".to_string();
    archival_rows[6].text_hash_hex = "hash-plan-updated".to_string();
    archival_rows[7].text_hash_hex = "hash-plan-updated".to_string();
    archival_rows[8].text_hash_hex = "hash-function-output".to_string();
    archival_rows[9].text_hash_hex = "hash-function-output".to_string();
    let compact_rows = archival_rows.clone();
    let dedupe_groups = vec![DedupeGroup {
        kind: CompactionKind::ToolCall,
        canonical_text_hash_hex: "dup-hash-neutral-output".to_string(),
        representative: RowRef::from_row(&archival_rows[1]),
        duplicates: vec![RowRef::from_row(&archival_rows[2])],
    }];
    let fixture = BundleFixture::from_rows(archival_rows, compact_rows, dedupe_groups);

    let result = agent_drift_analyzer::analyze_bundle(&AnalyzeRequest {
        input_dir: fixture.input_dir.clone(),
        output_dir: fixture.output_dir.clone(),
    })
    .expect("analyze neutral tool output bundle");
    let checkpoints = read_checkpoints(&result.checkpoints_path);
    let thrash = checkpoints[0]
        .drift_scores
        .iter()
        .find(|score| score.class == agent_drift_analyzer::DriftClass::DeadEndThrash)
        .expect("dead end thrash score");

    assert!(!thrash.flagged);
    assert_eq!(thrash.raw_score, 20);
    assert_eq!(thrash.state, DriftState::HistoricalOnly);
    assert!(thrash
        .evidence
        .iter()
        .all(|item| item.reason != "repeated failure evidence"));
    assert!(thrash.evidence.iter().any(|item| item
        .reason
        .starts_with("historical repeated verification evidence:")));
}

#[test]
fn dead_end_thrash_keeps_repeated_successful_verification_as_historical_context() {
    let rows = vec![
        row(
            0,
            CompactionKind::UserMessage,
            "/goal Finish the analyzer work and verify with cargo fmt, cargo clippy, and cargo test.",
        ),
        tool_row(1, "cargo fmt --all -- --check"),
        tool_row(2, "cargo fmt --all -- --check"),
        tool_row(3, "cargo fmt --all -- --check"),
        tool_row(4, "cargo clippy --workspace --all-targets -- -D warnings"),
        tool_row(5, "cargo clippy --workspace --all-targets -- -D warnings"),
        tool_row(6, "cargo clippy --workspace --all-targets -- -D warnings"),
        tool_row(7, "cargo test -p agent-drift-analyzer -- --nocapture"),
        tool_row(8, "cargo test -p agent-drift-analyzer -- --nocapture"),
        tool_row(9, "cargo test -p agent-drift-analyzer -- --nocapture"),
        row(10, CompactionKind::AssistantMessage, "Verification completed successfully."),
    ];
    let archival_rows = rows.clone();
    let compact_rows = rows
        .into_iter()
        .enumerate()
        .filter(|(index, _)| !matches!(index, 2 | 5 | 8))
        .map(|(_, row)| row)
        .collect();
    let dedupe_groups = vec![DedupeGroup {
        kind: CompactionKind::ToolCall,
        canonical_text_hash_hex: "dup-hash-successful-verification".to_string(),
        representative: RowRef::from_row(&archival_rows[1]),
        duplicates: vec![RowRef::from_row(&archival_rows[2])],
    }];
    let fixture = BundleFixture::from_rows(archival_rows, compact_rows, dedupe_groups);

    let result = agent_drift_analyzer::analyze_bundle(&AnalyzeRequest {
        input_dir: fixture.input_dir.clone(),
        output_dir: fixture.output_dir.clone(),
    })
    .expect("analyze repeated successful verification bundle");
    let checkpoints = read_checkpoints(&result.checkpoints_path);
    let thrash = checkpoints[0]
        .drift_scores
        .iter()
        .find(|score| score.class == agent_drift_analyzer::DriftClass::DeadEndThrash)
        .expect("dead end thrash score");

    assert!(!thrash.flagged);
    assert_eq!(thrash.raw_score, 20);
    assert_eq!(thrash.state, DriftState::HistoricalOnly);
    assert!(thrash.evidence.iter().any(|item| {
        item.reason
            == "historical repeated verification evidence: repeated verification command: cargo fmt --all -- --check"
    }));
    assert!(thrash.evidence.iter().any(|item| {
        item.reason
            == "historical repeated verification evidence: repeated verification command: cargo clippy --workspace --all-targets -- -D warnings"
    }));
    assert!(thrash.evidence.iter().any(|item| {
        item.reason
            == "historical repeated verification evidence: repeated verification command: cargo test -p agent-drift-analyzer -- --nocapture"
    }));
}

#[test]
fn dead_end_thrash_keeps_historical_verification_loops_out_of_active_failure_score() {
    let rows = vec![
        row(
            0,
            CompactionKind::UserMessage,
            "/goal Finish verification before investigating the remaining failure.",
        ),
        tool_row(1, "cargo test -p agent-drift-analyzer -- --nocapture"),
        tool_row(2, "cargo test -p agent-drift-analyzer -- --nocapture"),
        tool_row(3, "cargo test -p agent-drift-analyzer -- --nocapture"),
        row(
            4,
            CompactionKind::AssistantMessage,
            "Verification is done; now inspect the fresh failure.",
        ),
        row(
            5,
            CompactionKind::UserMessage,
            "/goal Diagnose the new repeated failure without reopening verification.",
        ),
        row(6, CompactionKind::Error, "world failed"),
        row(7, CompactionKind::Error, "world failed"),
    ];
    let mut archival_rows = rows.clone();
    archival_rows[6].text_hash_hex = "hash-mixed-current-failure".to_string();
    archival_rows[7].text_hash_hex = "hash-mixed-current-failure".to_string();
    let compact_rows = rows
        .into_iter()
        .enumerate()
        .filter(|(index, _)| *index != 2)
        .map(|(_, row)| row)
        .collect();
    let dedupe_groups = vec![DedupeGroup {
        kind: CompactionKind::ToolCall,
        canonical_text_hash_hex: "dup-hash-mixed-historical-verification".to_string(),
        representative: RowRef::from_row(&archival_rows[1]),
        duplicates: vec![RowRef::from_row(&archival_rows[2])],
    }];
    let fixture = BundleFixture::from_rows(archival_rows, compact_rows, dedupe_groups);

    let result = agent_drift_analyzer::analyze_bundle(&AnalyzeRequest {
        input_dir: fixture.input_dir.clone(),
        output_dir: fixture.output_dir.clone(),
    })
    .expect("analyze mixed historical-verification/current-failure bundle");
    let checkpoints = read_checkpoints(&result.checkpoints_path);
    let thrash = checkpoints[1]
        .drift_scores
        .iter()
        .find(|score| score.class == agent_drift_analyzer::DriftClass::DeadEndThrash)
        .expect("dead end thrash score");

    assert_eq!(checkpoints.len(), 2);
    assert!(thrash.flagged);
    assert!(thrash.raw_score >= 30);
    assert_eq!(thrash.state, DriftState::Active);
    assert!(thrash
        .evidence
        .iter()
        .any(|item| item.reason == "repeated failure evidence"));
    assert!(thrash.evidence.iter().any(|item| {
        item.reason
            == "historical repeated verification evidence: repeated verification command: cargo test -p agent-drift-analyzer -- --nocapture"
    }));
    assert!(!thrash
        .evidence
        .iter()
        .any(|item| item.reason.starts_with("repeated verification command:")));
}

#[test]
fn dead_end_thrash_flags_regressing_frontier_with_repeated_failure_activity() {
    let mut rows = vec![
        row(
            0,
            CompactionKind::UserMessage,
            "/goal Troubleshoot the failing checkpoint verifier without changing scope.",
        ),
        tool_row(
            1,
            "cargo test --color never -p agent-drift-analyzer checkpoints::captures_progress -- --nocapture",
        ),
        row(
            2,
            CompactionKind::ToolOutput,
            "Exit code: 101\nerror[E0425]: cannot find value `progress` in this scope\ncould not compile `agent-drift-analyzer` (lib test) due to 1 previous error",
        ),
        row(3, CompactionKind::Error, "world failed"),
        row(
            4,
            CompactionKind::UserMessage,
            "/goal Re-run the same troubleshooting verifier after a focused fix edit.",
        ),
        tool_row(
            5,
            "apply_patch <<'PATCH'\n*** Begin Patch\n*** Update File: crates/agent-drift-analyzer/src/checkpoint/progress.rs\n*** End Patch\nPATCH",
        ),
        tool_row(
            6,
            "cargo test --color always -p agent-drift-analyzer checkpoints::captures_progress -- --nocapture",
        ),
        row(
            7,
            CompactionKind::ToolOutput,
            "Exit code: 101\nrunning 1 test\ntest checkpoints::captures_progress ... FAILED\n\nfailures:\n    checkpoints::captures_progress\n\ntest result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 26 filtered out\nAssertionError: expected advancing",
        ),
        row(8, CompactionKind::Error, "world failed"),
        row(
            9,
            CompactionKind::UserMessage,
            "/goal Re-run the same troubleshooting verifier after the regression.",
        ),
        tool_row(
            10,
            "apply_patch <<'PATCH'\n*** Begin Patch\n*** Update File: crates/agent-drift-analyzer/src/checkpoint/progress.rs\n*** End Patch\nPATCH",
        ),
        tool_row(
            11,
            "cargo test --color auto -p agent-drift-analyzer checkpoints::captures_progress -- --nocapture",
        ),
        row(
            12,
            CompactionKind::ToolOutput,
            "Exit code: 101\nerror[E0425]: cannot find value `progress` in this scope\ncould not compile `agent-drift-analyzer` (lib test) due to 1 previous error",
        ),
        row(13, CompactionKind::Error, "world failed"),
    ];
    rows[3].text_hash_hex = "hash-regressing-frontier-failure".to_string();
    rows[8].text_hash_hex = "hash-regressing-frontier-failure".to_string();
    rows[13].text_hash_hex = "hash-regressing-frontier-failure".to_string();
    let fixture = BundleFixture::from_rows(rows.clone(), rows, Vec::new());

    let result = agent_drift_analyzer::analyze_bundle(&AnalyzeRequest {
        input_dir: fixture.input_dir.clone(),
        output_dir: fixture.output_dir.clone(),
    })
    .expect("analyze regressing-frontier dead-end-thrash bundle");
    let checkpoints = read_checkpoints(&result.checkpoints_path);
    let checkpoint = checkpoints.last().expect("regressing checkpoint");
    let progress = checkpoint
        .session_progress
        .as_ref()
        .expect("regressing session progress");
    let thrash = checkpoint
        .drift_scores
        .iter()
        .find(|score| score.class == agent_drift_analyzer::DriftClass::DeadEndThrash)
        .expect("dead end thrash score");

    assert_eq!(checkpoints.len(), 3);
    assert_eq!(
        progress.dimension,
        ProgressDimension::TroubleshootingFrontier
    );
    assert_eq!(progress.status, ProgressStatus::Regressing);
    assert!(progress
        .signals
        .iter()
        .any(|signal| signal.code == ProgressSignalCode::PreviouslyCleanScopeBroken));
    assert!(progress.signals.iter().all(|signal| {
        !matches!(
            signal.code,
            ProgressSignalCode::FailureFrontierAdvanced
                | ProgressSignalCode::FailureCountReduced
                | ProgressSignalCode::VerificationClean
        )
    }));
    assert_eq!(thrash.raw_score, 30);
    assert_eq!(thrash.confidence, Confidence::Medium);
    assert_eq!(thrash.state, DriftState::Active);
    assert!(thrash.flagged);
    assert!(thrash.evidence.iter().any(|item| {
        item.reason
            .starts_with("stall without frontier movement evidence:")
            && item.reason.contains("fell back")
    }));
    assert!(thrash
        .evidence
        .iter()
        .any(|item| item.reason == "repeated failure evidence" && item.row.event_index == 13));
    assert!(thrash
        .evidence
        .iter()
        .all(|item| !item.reason.contains("repeated verification")));
}

#[test]
fn dead_end_thrash_keeps_opaque_parent_orchestration_clear_without_child_activity() {
    let mut spawn = row(
        1,
        CompactionKind::ToolCall,
        "{\"goal\":\"implement the delegated acceptance control\"}",
    );
    spawn.dedupe_identity = Some(
        "{\"call_id\":\"call-spawn\",\"name\":\"spawn_agent\",\"type\":\"function_call\"}"
            .to_string(),
    );
    let mut wait = row(
        3,
        CompactionKind::ToolCall,
        "{\"session_id\":\"019ea333-3333-7333-8333-333333333333\"}",
    );
    wait.dedupe_identity = Some(
        "{\"call_id\":\"call-wait\",\"name\":\"wait_agent\",\"type\":\"function_call\"}"
            .to_string(),
    );
    let rows = vec![
        row(
            0,
            CompactionKind::UserMessage,
            "/goal Coordinate delegated work without overclaiming child progress.",
        ),
        spawn,
        row(
            2,
            CompactionKind::SystemMessage,
            "Child session id 019ea333-3333-7333-8333-333333333333 remains in a separate rollout file.",
        ),
        wait,
    ];
    let fixture = BundleFixture::from_rows(rows.clone(), rows, Vec::new());

    let result = agent_drift_analyzer::analyze_bundle(&AnalyzeRequest {
        input_dir: fixture.input_dir.clone(),
        output_dir: fixture.output_dir.clone(),
    })
    .expect("analyze opaque-parent dead-end-thrash bundle");
    let checkpoints = read_checkpoints(&result.checkpoints_path);
    let checkpoint = checkpoints.last().expect("opaque-parent checkpoint");
    let progress = checkpoint
        .session_progress
        .as_ref()
        .expect("opaque-parent session progress");
    let thrash = checkpoint
        .drift_scores
        .iter()
        .find(|score| score.class == agent_drift_analyzer::DriftClass::DeadEndThrash)
        .expect("dead end thrash score");

    assert_eq!(
        progress.dimension,
        ProgressDimension::ParentVisibleOrchestration
    );
    assert_eq!(
        (
            thrash.raw_score,
            thrash.confidence,
            thrash.state,
            thrash.flagged,
            thrash.evidence.len(),
        ),
        (0, Confidence::Low, DriftState::Cleared, false, 0),
    );
}

#[test]
fn dead_end_thrash_scores_stay_on_linked_parent_and_child_trajectories() {
    const PARENT: &str = "session-linked-parent";
    const CHILD: &str = "session-linked-child";

    let parent_rows = vec![
        delegated_row(
            PARENT,
            0,
            CompactionKind::UserMessage,
            "/goal Coordinate the bounded child task without claiming its implementation work.",
            None,
        ),
        delegated_row(
            PARENT,
            1,
            CompactionKind::ToolCall,
            r#"{"task_name":"r7_4_1_child","message":"Implement and verify only the child target."}"#,
            Some("spawn_agent"),
        ),
        delegated_row(
            PARENT,
            2,
            CompactionKind::ToolCall,
            r#"{"session_id":"session-linked-child"}"#,
            Some("wait_agent"),
        ),
        delegated_row(
            PARENT,
            3,
            CompactionKind::ToolCall,
            r#"{"session_id":"session-linked-child"}"#,
            Some("wait_agent"),
        ),
    ];
    let mut child_rows = vec![
        delegated_row(
            CHILD,
            0,
            CompactionKind::UserMessage,
            "/goal Troubleshoot only child_target without widening scope.",
            None,
        ),
        delegated_tool_row(
            CHILD,
            1,
            "cargo test -p child-target child_target -- --exact",
        ),
        delegated_row(CHILD, 2, CompactionKind::Error, "child target failed", None),
        delegated_row(
            CHILD,
            3,
            CompactionKind::UserMessage,
            "/goal Re-run the same child_target verifier before widening scope.",
            None,
        ),
        delegated_tool_row(
            CHILD,
            4,
            "cargo test -p child-target child_target -- --exact",
        ),
        delegated_row(CHILD, 5, CompactionKind::Error, "child target failed", None),
    ];
    child_rows[2].text_hash_hex = "hash-linked-child-failure".to_string();
    child_rows[5].text_hash_hex = "hash-linked-child-failure".to_string();

    let rows = parent_rows
        .into_iter()
        .chain(child_rows)
        .collect::<Vec<_>>();
    let fixture = BundleFixture::from_rows(rows.clone(), rows, Vec::new());
    let manifest_path = fixture.input_dir.join("manifest.json");
    let mut manifest: BundleManifest = serde_json::from_str(
        &fs::read_to_string(&manifest_path).expect("read linked scorer manifest"),
    )
    .expect("parse linked scorer manifest");
    manifest.delegation_links = vec![DelegationLink {
        parent_session_id: PARENT.to_string(),
        child_session_id: CHILD.to_string(),
        child_origin_parent_session_id: Some(PARENT.to_string()),
        depth: Some(1),
        state: DelegationLinkState::Verified,
        parent_evidence: vec![delegation_evidence(PARENT, 1)],
        child_evidence: vec![delegation_evidence(CHILD, 0)],
    }];
    fs::write(
        &manifest_path,
        serde_json::to_string_pretty(&manifest).expect("serialize linked scorer manifest"),
    )
    .expect("write linked scorer manifest");

    let result = agent_drift_analyzer::analyze_bundle(&AnalyzeRequest {
        input_dir: fixture.input_dir.clone(),
        output_dir: fixture.output_dir.clone(),
    })
    .expect("analyze linked scorer bundle");
    let parent = final_checkpoint_for_session(&result, PARENT);
    let child = final_checkpoint_for_session(&result, CHILD);
    let parent_thrash = drift_score(parent, DriftClass::DeadEndThrash);
    let child_thrash = drift_score(child, DriftClass::DeadEndThrash);

    assert_eq!(result.sessions.len(), 2);
    assert_eq!(
        (
            parent.delegation.topology,
            parent.delegation.child_work_visibility,
            parent.delegation.child_session_ids.as_slice(),
        ),
        (
            DelegationTopology::DelegatingParent,
            ChildWorkVisibility::Linked,
            [CHILD.to_string()].as_slice(),
        ),
    );
    assert_eq!(
        (
            child.delegation.topology,
            child.delegation.child_work_visibility,
            child.delegation.parent_session_id.as_deref(),
        ),
        (
            DelegationTopology::DelegatedChild,
            ChildWorkVisibility::Linked,
            Some(PARENT),
        ),
    );
    assert_eq!(
        (
            parent_thrash.raw_score,
            parent_thrash.state,
            parent_thrash.flagged,
            parent_thrash.evidence.len(),
        ),
        (0, DriftState::Cleared, false, 0),
        "repeated parent waits must not become dead-end thrash",
    );
    assert!(child_thrash.flagged, "child-local thrash must stay visible");
    assert_eq!(child_thrash.state, DriftState::Active);
    assert!(!child_thrash.evidence.is_empty());
    assert_checkpoint_scores_stay_session_local(parent, PARENT);
    assert_checkpoint_scores_stay_session_local(child, CHILD);
}

#[test]
fn dead_end_thrash_retains_medium_confidence_for_partial_parent_visible_activity() {
    let mut spawn = row(
        1,
        CompactionKind::ToolCall,
        "{\"goal\":\"fix packet R5-4\"}",
    );
    spawn.dedupe_identity = Some(
        "{\"call_id\":\"call-spawn\",\"name\":\"spawn_agent\",\"type\":\"function_call\"}"
            .to_string(),
    );
    let rows = vec![
        row(
            0,
            CompactionKind::UserMessage,
            "/goal Coordinate delegated findings without claiming child execution progress.",
        ),
        spawn,
        tool_row(
            2,
            "printf 'child rollout ' && sed -n '1,40p' /Users/spensermcconnell/.codex/sessions/2026/06/08/rollout-2026-06-08T12-00-00-019ea111-1111-7111-8111-111111111111.jsonl",
        ),
        tool_row(
            3,
            "cargo test -p agent-drift-analyzer checkpoints::captures_progress -- --nocapture",
        ),
        row(
            4,
            CompactionKind::ToolOutput,
            "Exit code: 101\nrunning 1 test\ntest checkpoints::captures_progress ... FAILED\n\nfailures:\n    checkpoints::captures_progress\n\ntest result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 26 filtered out\nAssertionError: expected advancing",
        ),
    ];
    let fixture = BundleFixture::from_rows(rows.clone(), rows, Vec::new());

    let result = agent_drift_analyzer::analyze_bundle(&AnalyzeRequest {
        input_dir: fixture.input_dir.clone(),
        output_dir: fixture.output_dir.clone(),
    })
    .expect("analyze partial parent-visible dead-end-thrash bundle");
    let checkpoints = read_checkpoints(&result.checkpoints_path);
    let checkpoint = checkpoints
        .last()
        .expect("partial parent-visible checkpoint");
    let progress = checkpoint
        .session_progress
        .as_ref()
        .expect("partial parent-visible session progress");
    let thrash = checkpoint
        .drift_scores
        .iter()
        .find(|score| score.class == agent_drift_analyzer::DriftClass::DeadEndThrash)
        .expect("dead end thrash score");

    assert_eq!(
        progress.dimension,
        ProgressDimension::ParentVisibleOrchestration
    );
    assert_eq!(progress.status, ProgressStatus::Mixed);
    assert_eq!(progress.confidence, Confidence::Medium);
    assert!(progress
        .signals
        .iter()
        .any(|signal| signal.code == ProgressSignalCode::DelegationVisibilityLimited));
    assert_eq!(
        (
            thrash.raw_score,
            thrash.confidence,
            thrash.state,
            thrash.flagged,
            thrash.evidence.len(),
        ),
        (0, Confidence::Medium, DriftState::Cleared, false, 0),
    );
}

#[test]
fn dead_end_thrash_scores_equal_progress_equally_across_turn_shapes() {
    let mut rows = vec![
        row(
            0,
            CompactionKind::UserMessage,
            "/goal Troubleshoot checkpoints::captures_progress without widening scope.",
        ),
        row(
            1,
            CompactionKind::AssistantMessage,
            "I will keep the first attempt on the focused troubleshooting target.",
        ),
        row(
            2,
            CompactionKind::AssistantMessage,
            "I will preserve the verifier output as the baseline witness.",
        ),
        tool_row(
            3,
            "apply_patch <<'PATCH'\n*** Begin Patch\n*** Update File: crates/agent-drift-analyzer/src/checkpoint/progress.rs\n*** End Patch\nPATCH",
        ),
        tool_row(
            4,
            "cargo test --color never -p agent-drift-analyzer checkpoints::captures_progress -- --nocapture",
        ),
        row(
            5,
            CompactionKind::ToolOutput,
            "Exit code: 101\nerror[E0425]: cannot find value `progress` in this scope\ncould not compile `agent-drift-analyzer` (lib test) due to 1 previous error",
        ),
        row(6, CompactionKind::Error, "world failed"),
        row(
            7,
            CompactionKind::UserMessage,
            "/goal Re-run the same troubleshooting verifier after the focused fix.",
        ),
        row(
            8,
            CompactionKind::AssistantMessage,
            "I will keep the rerun on the same troubleshooting target.",
        ),
        row(
            9,
            CompactionKind::AssistantMessage,
            "I will preserve the existing failure-only history.",
        ),
        tool_row(
            10,
            "printf 'focused edit retained' > /tmp/focused-edit-receipt",
        ),
        tool_row(
            11,
            "cargo test --color always -p agent-drift-analyzer checkpoints::captures_progress -- --nocapture",
        ),
        row(
            12,
            CompactionKind::ToolOutput,
            "Exit code: 101\nrunning 1 test\ntest checkpoints::captures_progress ... FAILED\n\nfailures:\n    checkpoints::captures_progress\n\ntest result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 26 filtered out\nAssertionError: expected advancing",
        ),
        row(13, CompactionKind::Error, "world failed"),
    ];
    rows[6].text_hash_hex = "hash-turn-shape-failure".to_string();
    rows[13].text_hash_hex = "hash-turn-shape-failure".to_string();

    let long_autonomous_rows = rows.clone();
    let mut many_short_conversational_rows = rows;
    for row in &mut many_short_conversational_rows[7..] {
        row.turn_id = Some("turn-002".to_string());
    }

    let analyze = |rows: Vec<CompactionRow>, shape: &str| {
        let fixture = BundleFixture::from_rows(rows.clone(), rows, Vec::new());
        let result = agent_drift_analyzer::analyze_bundle(&AnalyzeRequest {
            input_dir: fixture.input_dir.clone(),
            output_dir: fixture.output_dir.clone(),
        })
        .unwrap_or_else(|error| panic!("analyze {shape} dead-end-thrash bundle: {error}"));
        read_checkpoints(&result.checkpoints_path)
    };

    let long_checkpoints = analyze(long_autonomous_rows, "long-autonomous");
    let short_checkpoints = analyze(many_short_conversational_rows, "many-short-conversational");

    assert_eq!(long_checkpoints.len(), 2);
    assert_eq!(short_checkpoints.len(), 2);
    assert!(short_checkpoints.iter().all(|checkpoint| {
        checkpoint.turn_context.as_ref().is_some_and(|context| {
            context.execution_mode == agent_drift_analyzer::TurnExecutionMode::Conversational
        })
    }));
    for checkpoints in [&long_checkpoints, &short_checkpoints] {
        assert!(checkpoints[..checkpoints.len() - 1]
            .iter()
            .all(|checkpoint| {
                checkpoint
                    .drift_scores
                    .iter()
                    .find(|score| score.class == agent_drift_analyzer::DriftClass::DeadEndThrash)
                    .is_none_or(|score| score.state != DriftState::Active)
            }));
    }

    let long_checkpoint = long_checkpoints.last().expect("long-autonomous checkpoint");
    let short_checkpoint = short_checkpoints
        .last()
        .expect("many-short-conversational checkpoint");
    assert_eq!(
        long_checkpoint
            .turn_context
            .as_ref()
            .expect("long-autonomous turn context")
            .execution_mode,
        agent_drift_analyzer::TurnExecutionMode::Autonomous,
    );
    let long_progress = long_checkpoint
        .session_progress
        .as_ref()
        .expect("long-autonomous session progress");
    let short_progress = short_checkpoint
        .session_progress
        .as_ref()
        .expect("many-short-conversational session progress");
    assert_eq!(long_progress, short_progress);
    assert_eq!(
        long_progress.dimension,
        ProgressDimension::TroubleshootingFrontier
    );
    assert_eq!(long_progress.status, ProgressStatus::Advancing);
    assert!(long_progress
        .signals
        .iter()
        .any(|signal| signal.code == ProgressSignalCode::FailureFrontierAdvanced));

    let long_thrash = long_checkpoint
        .drift_scores
        .iter()
        .find(|score| score.class == agent_drift_analyzer::DriftClass::DeadEndThrash)
        .expect("long-autonomous dead end thrash score");
    let short_thrash = short_checkpoint
        .drift_scores
        .iter()
        .find(|score| score.class == agent_drift_analyzer::DriftClass::DeadEndThrash)
        .expect("many-short-conversational dead end thrash score");
    assert_eq!(long_thrash, short_thrash);
    assert_eq!(
        (
            long_thrash.raw_score,
            long_thrash.confidence,
            long_thrash.state,
            long_thrash.flagged,
        ),
        (20, Confidence::Medium, DriftState::HistoricalOnly, false),
    );
    assert!(long_thrash
        .evidence
        .iter()
        .any(|item| item.reason.starts_with("churn with progress evidence:")));
    assert!(long_thrash
        .evidence
        .iter()
        .any(|item| item.reason.contains("historical repeated failure evidence")));
    assert!(long_thrash
        .evidence
        .iter()
        .all(|item| !item.reason.contains("repeated verification")));
}

#[test]
fn dead_end_thrash_suppresses_repeated_activity_when_the_frontier_advances() {
    let mut rows = vec![
        row(
            0,
            CompactionKind::UserMessage,
            "/goal Troubleshoot checkpoints::captures_progress without widening scope.",
        ),
        tool_row(
            1,
            "cargo test -p agent-drift-analyzer checkpoints::captures_progress -- --nocapture",
        ),
        row(
            2,
            CompactionKind::ToolOutput,
            "Exit code: 101\nrunning 2 tests\ntest checkpoints::captures_progress ... FAILED\n\nfailures:\n    checkpoints::captures_progress\n\nthread 'checkpoints::captures_progress' panicked at crates/agent-drift-analyzer/src/checkpoint/progress.rs:12:34:\nAssertionError: expected advancing\n\ntest result: FAILED. 0 passed; 2 failed; 0 ignored; 0 measured; 26 filtered out",
        ),
        row(3, CompactionKind::Error, "world failed"),
        row(
            4,
            CompactionKind::UserMessage,
            "/goal Re-run the same troubleshooting verifier after a focused fix edit.",
        ),
        tool_row(
            5,
            "apply_patch <<'PATCH'\n*** Begin Patch\n*** Update File: crates/agent-drift-analyzer/src/checkpoint/progress.rs\n*** End Patch\nPATCH",
        ),
        tool_row(
            6,
            "cargo test -p agent-drift-analyzer checkpoints::captures_progress -- --nocapture",
        ),
        row(
            7,
            CompactionKind::ToolOutput,
            "Exit code: 101\nrunning 1 test\ntest checkpoints::captures_progress ... FAILED\n\nfailures:\n    checkpoints::captures_progress\n\nthread 'checkpoints::captures_progress' panicked at crates/agent-drift-analyzer/src/checkpoint/progress.rs:12:34:\nAssertionError: expected advancing\n\ntest result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 26 filtered out",
        ),
        row(8, CompactionKind::Error, "world failed"),
    ];
    rows[3].text_hash_hex = "hash-frontier-advancing-failure".to_string();
    rows[8].text_hash_hex = "hash-frontier-advancing-failure".to_string();
    let fixture = BundleFixture::from_rows(rows.clone(), rows, Vec::new());

    let result = agent_drift_analyzer::analyze_bundle(&AnalyzeRequest {
        input_dir: fixture.input_dir.clone(),
        output_dir: fixture.output_dir.clone(),
    })
    .expect("analyze frontier-advancing thrash bundle");
    let checkpoints = read_checkpoints(&result.checkpoints_path);
    let thrash = checkpoints[1]
        .drift_scores
        .iter()
        .find(|score| score.class == agent_drift_analyzer::DriftClass::DeadEndThrash)
        .expect("dead end thrash score");

    assert_eq!(checkpoints.len(), 2);
    assert!(!thrash.flagged);
    assert_eq!(thrash.raw_score, 20);
    assert_eq!(thrash.state, DriftState::HistoricalOnly);
    assert!(thrash
        .evidence
        .iter()
        .any(|item| item.reason.starts_with("churn with progress evidence:")));
}

#[test]
fn dead_end_thrash_names_stalls_when_repeated_activity_has_no_frontier_movement() {
    let mut rows = vec![
        row(
            0,
            CompactionKind::UserMessage,
            "/goal Troubleshoot checkpoints::captures_progress without widening scope.",
        ),
        tool_row(
            1,
            "cargo test -p agent-drift-analyzer checkpoints::captures_progress -- --nocapture",
        ),
        row(
            2,
            CompactionKind::ToolOutput,
            "Exit code: 101\nrunning 1 test\ntest checkpoints::captures_progress ... FAILED\n\nfailures:\n    checkpoints::captures_progress\n\nAssertionError: expected advancing",
        ),
        row(3, CompactionKind::Error, "world failed"),
        row(
            4,
            CompactionKind::UserMessage,
            "/goal Re-run the same troubleshooting verifier before widening scope.",
        ),
        tool_row(
            5,
            "cargo test -p agent-drift-analyzer checkpoints::captures_progress -- --nocapture",
        ),
        row(
            6,
            CompactionKind::ToolOutput,
            "Exit code: 101\nrunning 1 test\ntest checkpoints::captures_progress ... FAILED\n\nfailures:\n    checkpoints::captures_progress\n\nAssertionError: expected advancing",
        ),
        row(7, CompactionKind::Error, "world failed"),
    ];
    rows[3].text_hash_hex = "hash-frontier-stalled-failure".to_string();
    rows[7].text_hash_hex = "hash-frontier-stalled-failure".to_string();
    let fixture = BundleFixture::from_rows(rows.clone(), rows, Vec::new());

    let result = agent_drift_analyzer::analyze_bundle(&AnalyzeRequest {
        input_dir: fixture.input_dir.clone(),
        output_dir: fixture.output_dir.clone(),
    })
    .expect("analyze frontier-stalled thrash bundle");
    let checkpoints = read_checkpoints(&result.checkpoints_path);
    let thrash = checkpoints[1]
        .drift_scores
        .iter()
        .find(|score| score.class == agent_drift_analyzer::DriftClass::DeadEndThrash)
        .expect("dead end thrash score");

    assert_eq!(checkpoints.len(), 2);
    assert!(thrash.flagged);
    assert_eq!(thrash.raw_score, 30);
    assert_eq!(thrash.state, DriftState::Active);
    assert!(thrash.evidence.iter().any(|item| item
        .reason
        .starts_with("stall without frontier movement evidence:")));
}

#[test]
fn dead_end_thrash_clears_replay_shaped_memsrc_verifier_tail() {
    let rows = vec![
        row(
            0,
            CompactionKind::UserMessage,
            "/goal Finish the memsrc analyzer work and rerun the verification wall before closeout.",
        ),
        tool_row(1, "sed -n '1,220p' crates/memsrc/src/compare/mod.rs"),
        tool_row(2, "sed -n '1,220p' crates/memsrc/tests/extraction_golden.rs"),
        tool_row(
            3,
            "tmpdir=$(mktemp -d); cargo run -p memsrc -- compare-yaml --package memory.source.json --baseline baseline-memory --out \"$tmpdir/root-compare.json\" >/dev/null && sed -n '1,220p' \"$tmpdir/root-compare.json\"",
        ),
        tool_row(
            4,
            "tmpdir=$(mktemp -d); cargo run -p memsrc -- compare-yaml --package eval-corpus/ts-memory/memory.source.json --baseline baseline-memory --out \"$tmpdir/eval-compare.json\" >/dev/null && sed -n '1,220p' \"$tmpdir/eval-compare.json\"",
        ),
        tool_row(
            5,
            "cargo run -p memsrc -- check --package crates/memsrc/fixtures/adversarial/memory.source.json",
        ),
        row(6, CompactionKind::AssistantMessage, "Need one more verification pass."),
        tool_row(7, "cargo check -p memsrc"),
        tool_row(8, "cargo check -p memsrc"),
        tool_row(9, "cargo check -p memsrc"),
        tool_row(10, "cargo clippy -p memsrc -- -D warnings"),
        tool_row(11, "cargo clippy -p memsrc -- -D warnings"),
        tool_row(12, "cargo clippy -p memsrc -- -D warnings"),
        tool_row(
            13,
            "tmpdir=$(mktemp -d); cargo run -p memsrc -- compare-yaml --package memory.source.json --baseline baseline-memory --out \"$tmpdir/root-compare.json\" >/dev/null && sed -n '1,220p' \"$tmpdir/root-compare.json\"",
        ),
        tool_row(
            14,
            "tmpdir=$(mktemp -d); cargo run -p memsrc -- compare-yaml --package eval-corpus/ts-memory/memory.source.json --baseline baseline-memory --out \"$tmpdir/eval-compare.json\" >/dev/null && sed -n '1,220p' \"$tmpdir/eval-compare.json\"",
        ),
        tool_row(
            15,
            "cargo run -p memsrc -- check --package crates/memsrc/fixtures/adversarial/memory.source.json",
        ),
        row(16, CompactionKind::AssistantMessage, "Verification completed successfully."),
    ];
    let archival_rows = rows.clone();
    let compact_rows = rows
        .into_iter()
        .enumerate()
        .filter(|(index, _)| !matches!(index, 8 | 11))
        .map(|(_, row)| row)
        .collect();
    let dedupe_groups = vec![
        DedupeGroup {
            kind: CompactionKind::ToolCall,
            canonical_text_hash_hex: "dup-hash-replay-shaped-check".to_string(),
            representative: RowRef::from_row(&archival_rows[7]),
            duplicates: vec![RowRef::from_row(&archival_rows[8])],
        },
        DedupeGroup {
            kind: CompactionKind::ToolCall,
            canonical_text_hash_hex: "dup-hash-replay-shaped-clippy".to_string(),
            representative: RowRef::from_row(&archival_rows[10]),
            duplicates: vec![RowRef::from_row(&archival_rows[11])],
        },
    ];
    let fixture = BundleFixture::from_rows(archival_rows, compact_rows, dedupe_groups);

    let result = agent_drift_analyzer::analyze_bundle(&AnalyzeRequest {
        input_dir: fixture.input_dir.clone(),
        output_dir: fixture.output_dir.clone(),
    })
    .expect("analyze replay-shaped memsrc verifier bundle");
    let checkpoints = read_checkpoints(&result.checkpoints_path);
    let thrash = checkpoints
        .last()
        .expect("final checkpoint")
        .drift_scores
        .iter()
        .find(|score| score.class == agent_drift_analyzer::DriftClass::DeadEndThrash)
        .expect("dead end thrash score");

    assert!(!thrash.flagged);
    assert_eq!(thrash.raw_score, 20);
    assert_eq!(thrash.state, DriftState::HistoricalOnly);
    assert!(thrash.evidence.iter().any(|item| {
        item.reason
            == "historical repeated verification evidence: repeated verification command: cargo check -p memsrc"
    }));
    assert!(thrash.evidence.iter().any(|item| {
        item.reason
            == "historical repeated verification evidence: repeated verification command: cargo clippy -p memsrc -- -D warnings"
    }));
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
    let payload = format!("{{\"command\":{command:?},\"workdir\":\"/repo\"}}");
    row(event_index, CompactionKind::ToolCall, &payload)
}

fn delegated_row(
    session_id: &str,
    event_index: usize,
    kind: CompactionKind,
    text: &str,
    tool_name: Option<&str>,
) -> CompactionRow {
    let mut item = row(event_index, kind, text);
    item.source_file = Utf8PathBuf::from(format!("/tmp/{session_id}/rollout.jsonl"));
    item.session_id = Some(session_id.to_string());
    item.dedupe_identity = tool_name.map(|name| {
        format!(
            "{{\"call_id\":\"call-{session_id}-{event_index}\",\"name\":\"{name}\",\"type\":\"function_call\"}}"
        )
    });
    item.text_hash_hex = format!("hash-{session_id}-{event_index}");
    item
}

fn delegated_tool_row(session_id: &str, event_index: usize, command: &str) -> CompactionRow {
    delegated_row(
        session_id,
        event_index,
        CompactionKind::ToolCall,
        &format!("{{\"command\":{command:?},\"workdir\":\"/repo\"}}"),
        Some("functions.shell_command"),
    )
}

fn delegation_evidence(session_id: &str, event_index: usize) -> DelegationEvidenceRef {
    DelegationEvidenceRef {
        source_file: Utf8PathBuf::from(format!("/tmp/{session_id}/rollout.jsonl")),
        line_number: event_index + 1,
        event_index,
    }
}

fn final_checkpoint_for_session<'a>(
    result: &'a agent_drift_analyzer::AnalyzeResult,
    session_id: &str,
) -> &'a Checkpoint {
    result
        .sessions
        .iter()
        .find(|session| session.session_id == session_id)
        .and_then(|session| session.checkpoints.last())
        .unwrap_or_else(|| panic!("final checkpoint for {session_id}"))
}

fn drift_score(checkpoint: &Checkpoint, class: DriftClass) -> &DriftScore {
    checkpoint
        .drift_scores
        .iter()
        .find(|score| score.class == class)
        .unwrap_or_else(|| panic!("{class:?} score"))
}

fn assert_checkpoint_scores_stay_session_local(checkpoint: &Checkpoint, session_id: &str) {
    assert!(
        checkpoint
            .boundary
            .start
            .source_file
            .as_str()
            .contains(session_id),
        "checkpoint boundary must belong to {session_id}"
    );
    assert!(
        checkpoint.drift_scores.iter().all(|score| {
            score
                .evidence
                .iter()
                .all(|item| item.row.source_file.as_str().contains(session_id))
        }),
        "every drift score and evidence row must stay on {session_id}"
    );
}
