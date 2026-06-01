#![allow(unused_crate_dependencies)]

mod support;

use std::collections::BTreeSet;
use std::fs;

use agent_drift_analyzer::checkpoint::summarize_checkpoint_diagnostics;
use agent_drift_analyzer::{Confidence, DriftClass};
use agent_session_compactor::{CompactionKind, CompactionRow, SourceKind, UserMessageRole};
use camino::Utf8PathBuf;
use support::{load_sample_bundle, read_checkpoints, BundleFixture};
use time::macros::datetime;

#[test]
fn export_bundle_writes_checkpoints_and_summary() {
    let fixture = BundleFixture::sample();
    let result = agent_drift_analyzer::analyze_bundle(&agent_drift_analyzer::AnalyzeRequest {
        input_dir: fixture.input_dir.clone(),
        output_dir: fixture.output_dir.clone(),
    })
    .expect("analyze sample bundle");
    let checkpoints = read_checkpoints(&result.checkpoints_path);
    let summary = fs::read_to_string(&result.summary_path).expect("summary");
    let flagged = checkpoints
        .iter()
        .filter(|checkpoint| checkpoint.flagged)
        .count();
    let longest_flagged_streak = checkpoints
        .iter()
        .fold((0usize, 0usize), |(longest, current), checkpoint| {
            if checkpoint.flagged {
                let next = current + 1;
                (longest.max(next), next)
            } else {
                (longest, 0)
            }
        })
        .0;
    let distinct_task_frames = checkpoints
        .iter()
        .map(|checkpoint| {
            serde_json::to_string(&(
                &checkpoint.task_frame.objective,
                &checkpoint.task_frame.truth_artifacts,
                &checkpoint.task_frame.working_set_paths,
                &checkpoint.task_frame.tools,
                &checkpoint.task_frame.command_families,
                &checkpoint.task_frame.verification_commands,
            ))
            .expect("task frame identity")
        })
        .collect::<BTreeSet<_>>()
        .len();
    let truth_artifacts_referenced = checkpoints
        .iter()
        .flat_map(|checkpoint| checkpoint.task_frame.truth_artifacts.iter().cloned())
        .collect::<BTreeSet<_>>()
        .len();
    let verification_commands_observed = checkpoints
        .iter()
        .flat_map(|checkpoint| checkpoint.task_frame.verification_commands.iter().cloned())
        .collect::<BTreeSet<_>>()
        .len();

    assert_eq!(checkpoints.len(), 2);
    assert!(summary.contains("Agent Drift Analyzer Summary"));
    assert!(summary.contains("session-alpha"));
    assert!(summary.contains("Turns observed: `1`"));
    assert!(summary.contains("User prompts observed: `1`"));
    assert!(summary.contains("Checkpoints emitted: `2`"));
    assert!(summary.contains("Checkpoints per turn: `2.00`"));
    assert!(summary.contains("Checkpoints per user prompt: `2.00`"));
    assert!(summary.contains("Avg rows between checkpoints: `4.00`"));
    assert!(summary.contains("Avg seconds between checkpoints: `unavailable`"));
    assert!(summary.contains(&format!("Flagged checkpoints: `{flagged}`")));
    assert!(summary.contains(&format!(
        "Longest flagged streak: `{longest_flagged_streak}`"
    )));
    assert!(summary.contains("Prompt user messages: `1`"));
    assert!(summary.contains("Steer user messages: `0`"));
    assert!(summary.contains("Unknown user messages: `0`"));
    assert!(summary.contains("- Turns observed: `1`"));
    assert!(summary.contains("- User prompts observed: `1`"));
    assert!(summary.contains("- Checkpoints emitted: `2`"));
    assert!(summary.contains("- Checkpoints per turn: `2.00`"));
    assert!(summary.contains("- Checkpoints per user prompt: `2.00`"));
    assert!(summary.contains("- Avg rows between checkpoints: `4.00`"));
    assert!(summary.contains("- Avg seconds between checkpoints: `unavailable`"));
    assert!(summary.contains(&format!("- Flagged checkpoints: `{flagged}`")));
    assert!(summary.contains(&format!(
        "- Longest flagged streak: `{longest_flagged_streak}`"
    )));
    assert!(summary.contains(&format!("- Distinct task frames: `{distinct_task_frames}`")));
    assert!(summary.contains(&format!(
        "- Truth artifacts referenced: `{truth_artifacts_referenced}`"
    )));
    assert!(summary.contains(&format!(
        "- Verification commands observed: `{verification_commands_observed}`"
    )));
    assert!(summary.contains("- Prompt user messages: `1`"));
}

#[test]
fn export_bundle_reports_time_spacing_from_boundary_timestamps() {
    let bundle = load_sample_bundle();
    let archival_rows = bundle
        .archival_rows
        .into_iter()
        .map(|mut row| {
            row.timestamp = Some(
                datetime!(2026-05-29 12:00:00 UTC)
                    + time::Duration::seconds(row.event_index as i64 * 60),
            );
            row
        })
        .collect::<Vec<_>>();
    let compact_rows = bundle
        .compact_rows
        .into_iter()
        .map(|mut row| {
            row.timestamp = Some(
                datetime!(2026-05-29 12:00:00 UTC)
                    + time::Duration::seconds(row.event_index as i64 * 60),
            );
            row
        })
        .collect::<Vec<_>>();
    let fixture = BundleFixture::from_rows(archival_rows, compact_rows, bundle.dedupe_groups);
    let result = agent_drift_analyzer::analyze_bundle(&agent_drift_analyzer::AnalyzeRequest {
        input_dir: fixture.input_dir.clone(),
        output_dir: fixture.output_dir.clone(),
    })
    .expect("analyze sample bundle with timestamps");
    let summary = fs::read_to_string(&result.summary_path).expect("summary");

    assert!(summary.contains("Avg seconds between checkpoints: `240.00`"));
    assert!(summary.contains("- Avg seconds between checkpoints: `240.00`"));
}

#[test]
fn export_bundle_ignores_synthetic_setup_rows_in_user_message_diagnostics() {
    let mut bundle = load_sample_bundle();
    bundle.compact_rows.insert(
        0,
        CompactionRow {
            source_file: Utf8PathBuf::from("/tmp/session-alpha/rollout.jsonl"),
            source_kind: SourceKind::CodexRolloutJsonl,
            session_id: Some("session-alpha".to_string()),
            turn_id: Some("turn-001".to_string()),
            event_index: 0,
            line_number: 0,
            row_ordinal: 0,
            timestamp: None,
            kind: CompactionKind::UserMessage,
            user_message_role: Some(UserMessageRole::Unknown),
            dedupe_identity: None,
            text: "# AGENTS.md instructions".to_string(),
            canonical_text: "# AGENTS.md instructions".to_string(),
            text_hash_hex: "synthetic-hash".to_string(),
        },
    );
    let fixture = BundleFixture::from_rows(
        bundle.archival_rows,
        bundle.compact_rows,
        bundle.dedupe_groups,
    );
    let result = agent_drift_analyzer::analyze_bundle(&agent_drift_analyzer::AnalyzeRequest {
        input_dir: fixture.input_dir.clone(),
        output_dir: fixture.output_dir.clone(),
    })
    .expect("analyze sample bundle with synthetic setup row");
    let summary = fs::read_to_string(&result.summary_path).expect("summary");

    assert!(summary.contains("Prompt user messages: `1`"));
    assert!(summary.contains("Steer user messages: `0`"));
    assert!(summary.contains("Unknown user messages: `0`"));
}

#[test]
fn export_bundle_serializes_v0_2_checkpoint_diagnostics() {
    let fixture = BundleFixture::sample();
    let result = agent_drift_analyzer::analyze_bundle(&agent_drift_analyzer::AnalyzeRequest {
        input_dir: fixture.input_dir.clone(),
        output_dir: fixture.output_dir.clone(),
    })
    .expect("analyze sample bundle");
    let checkpoints = read_checkpoints(&result.checkpoints_path);

    assert_eq!(checkpoints.len(), 2);
    assert!(checkpoints
        .iter()
        .all(|checkpoint| checkpoint.schema_version == "v0.2"));

    let first = &checkpoints[0];
    let second = &checkpoints[1];

    assert!(!first.diagnostics.task_frame_transitioned);
    assert!(!first.diagnostics.working_set_changed);
    assert_eq!(first.diagnostics.interval_command_count, 4);
    assert_eq!(first.diagnostics.interval_verification_command_count, 2);
    assert!(first.diagnostics.evidence_item_count > 0);

    assert!(second.diagnostics.task_frame_transitioned);
    assert!(second.diagnostics.working_set_changed);
    assert_eq!(second.diagnostics.interval_command_count, 3);
    assert_eq!(second.diagnostics.interval_verification_command_count, 1);
    assert!(second.diagnostics.evidence_item_count > 0);
}

#[test]
fn export_bundle_summarizes_checkpoint_local_diagnostics() {
    let fixture = BundleFixture::sample();
    let result = agent_drift_analyzer::analyze_bundle(&agent_drift_analyzer::AnalyzeRequest {
        input_dir: fixture.input_dir.clone(),
        output_dir: fixture.output_dir.clone(),
    })
    .expect("analyze sample bundle");
    let checkpoints = read_checkpoints(&result.checkpoints_path);
    let stats = summarize_checkpoint_diagnostics(&checkpoints);

    assert_eq!(stats.checkpoint_count, checkpoints.len());
    assert_eq!(
        stats.flagged_checkpoint_count,
        checkpoints
            .iter()
            .filter(|checkpoint| checkpoint.flagged)
            .count()
    );
    assert_eq!(
        stats.task_frame_transition_count,
        checkpoints
            .iter()
            .filter(|checkpoint| checkpoint.diagnostics.task_frame_transitioned)
            .count()
    );
    assert_eq!(stats.adjacent_checkpoint_pair_count, 1);
    assert_eq!(
        stats.working_set_change_count,
        checkpoints
            .iter()
            .filter(|checkpoint| checkpoint.diagnostics.working_set_changed)
            .count()
    );
    assert_eq!(
        stats.total_evidence_item_count,
        checkpoints
            .iter()
            .map(|checkpoint| checkpoint.diagnostics.evidence_item_count)
            .sum::<usize>()
    );
    assert_eq!(
        stats.confidence_distribution.low,
        checkpoints
            .iter()
            .filter(|checkpoint| checkpoint.task_frame.confidence == Confidence::Low)
            .count()
    );
    assert_eq!(
        stats.confidence_distribution.medium,
        checkpoints
            .iter()
            .filter(|checkpoint| checkpoint.task_frame.confidence == Confidence::Medium)
            .count()
    );
    assert_eq!(
        stats.confidence_distribution.high,
        checkpoints
            .iter()
            .filter(|checkpoint| checkpoint.task_frame.confidence == Confidence::High)
            .count()
    );
    for class in [
        DriftClass::WrongPlanBranch,
        DriftClass::IgnoringRepoTruth,
        DriftClass::DeadEndThrash,
    ] {
        let expected = checkpoints
            .iter()
            .filter(|checkpoint| {
                checkpoint
                    .drift_scores
                    .iter()
                    .any(|score| score.class == class && score.flagged)
            })
            .count();
        assert_eq!(
            stats.drift_class_flagged_counts.get(&class).copied(),
            Some(expected)
        );
        assert_optional_metric_eq(
            stats.drift_class_flagged_rate(class),
            expected as f64 / checkpoints.len() as f64,
        );
    }
    assert_optional_metric_eq(
        stats.flagged_checkpoint_rate(),
        stats.flagged_checkpoint_count as f64 / checkpoints.len() as f64,
    );
    assert_optional_metric_eq(stats.working_set_churn(), 1.0);
    assert_optional_metric_eq(
        stats.average_evidence_items_per_checkpoint(),
        stats.total_evidence_item_count as f64 / checkpoints.len() as f64,
    );
}

fn assert_optional_metric_eq(actual: Option<f64>, expected: f64) {
    let actual = actual.expect("metric should be available");
    assert!(
        (actual - expected).abs() < f64::EPSILON,
        "expected {expected}, got {actual}"
    );
}
