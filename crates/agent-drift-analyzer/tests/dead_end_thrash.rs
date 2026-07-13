#![allow(unused_crate_dependencies)]

mod support;

use agent_drift_analyzer::{
    AnalyzeRequest, Confidence, DriftState, ProgressDimension, ProgressSignalCode, ProgressStatus,
};
use agent_session_compactor::{
    CompactionKind, CompactionRow, DedupeGroup, RowRef, SourceKind, UserMessageRole,
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
    let mut row = row(event_index, CompactionKind::ToolCall, &payload);
    row.dedupe_identity = Some(
        "{\"call_id\":\"call-1\",\"name\":\"functions.shell_command\",\"type\":\"function_call\"}"
            .to_string(),
    );
    row
}
