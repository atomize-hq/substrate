#![allow(unused_crate_dependencies)]

mod support;

use std::collections::BTreeSet;
use std::fs;

use agent_drift_analyzer::checkpoint::{
    export_checkpoints, summarize_checkpoint_diagnostics, CheckpointDiagnostics,
};
use agent_drift_analyzer::{
    BundleSession, Checkpoint, CheckpointBoundary, Confidence, DriftClass, DriftScore, EvidenceRef,
    TaskFrame,
};
use agent_session_compactor::{CompactionKind, CompactionRow, RowRef, SourceKind, UserMessageRole};
use camino::Utf8PathBuf;
use support::{load_sample_bundle, read_checkpoints, BundleFixture};
use tempfile::TempDir;
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

#[test]
fn export_bundle_renders_v0_3_diagnostics_slice() {
    let fixture = BundleFixture::sample();
    let result = agent_drift_analyzer::analyze_bundle(&agent_drift_analyzer::AnalyzeRequest {
        input_dir: fixture.input_dir.clone(),
        output_dir: fixture.output_dir.clone(),
    })
    .expect("analyze sample bundle");
    let checkpoints = read_checkpoints(&result.checkpoints_path);
    let summary = fs::read_to_string(&result.summary_path).expect("summary");
    let stats = summarize_checkpoint_diagnostics(&checkpoints);

    assert!(summary.contains(&format!(
        "Flagged checkpoint rate: `{}`",
        format_optional_metric(stats.flagged_checkpoint_rate())
    )));
    assert!(summary.contains(&format!(
        "Drift-class flagged frequency: `{}`",
        format_drift_class_frequencies(&stats)
    )));
    assert!(summary.contains(&format!(
        "Task-frame transition count: `{}`",
        stats.task_frame_transition_count
    )));
    assert!(summary.contains(&format!(
        "Task-frame confidence distribution: `{}`",
        format_confidence_distribution(&stats)
    )));
    assert!(summary.contains(&format!(
        "Working-set churn: `{}`",
        format_optional_metric(stats.working_set_churn())
    )));
    assert!(summary.contains(&format!(
        "Verification density: `{}`",
        format_optional_metric(stats.verification_density())
    )));
    assert!(summary.contains(&format!(
        "Average evidence items per checkpoint: `{}`",
        format_optional_metric(stats.average_evidence_items_per_checkpoint())
    )));
    assert!(summary.contains(&format!(
        "- Flagged checkpoint rate: `{}`",
        format_optional_metric(stats.flagged_checkpoint_rate())
    )));
    assert!(summary.contains(&format!(
        "- Verification density: `{}`",
        format_optional_metric(stats.verification_density())
    )));
}

#[test]
fn export_bundle_weights_top_level_v0_3_metrics_from_raw_counts() {
    let session_alpha = fixture_session(
        "session-alpha",
        vec![
            fixture_row(
                "session-alpha",
                0,
                CompactionKind::UserMessage,
                "/goal A",
                Some(UserMessageRole::Prompt),
            ),
            fixture_row(
                "session-alpha",
                1,
                CompactionKind::ToolCall,
                "{\"command\":\"cargo fmt\"}",
                None,
            ),
            fixture_row(
                "session-alpha",
                2,
                CompactionKind::ToolCall,
                "{\"command\":\"cargo clippy\"}",
                None,
            ),
        ],
    );
    let session_beta = fixture_session(
        "session-beta",
        vec![
            fixture_row(
                "session-beta",
                0,
                CompactionKind::UserMessage,
                "/goal B",
                Some(UserMessageRole::Prompt),
            ),
            fixture_row(
                "session-beta",
                1,
                CompactionKind::ToolCall,
                "{\"command\":\"cargo test\"}",
                None,
            ),
        ],
    );
    let checkpoints = vec![
        fixture_checkpoint(
            &session_alpha,
            1,
            1,
            Confidence::Low,
            CheckpointDiagnostics {
                interval_command_count: 2,
                interval_verification_command_count: 0,
                evidence_item_count: 10,
                ..CheckpointDiagnostics::default()
            },
            &[DriftClass::WrongPlanBranch],
        ),
        fixture_checkpoint(
            &session_alpha,
            2,
            2,
            Confidence::Medium,
            CheckpointDiagnostics {
                task_frame_transitioned: true,
                working_set_changed: true,
                interval_command_count: 2,
                interval_verification_command_count: 0,
                evidence_item_count: 10,
            },
            &[],
        ),
        fixture_checkpoint(
            &session_beta,
            1,
            1,
            Confidence::High,
            CheckpointDiagnostics {
                interval_command_count: 1,
                interval_verification_command_count: 1,
                evidence_item_count: 1,
                ..CheckpointDiagnostics::default()
            },
            &[DriftClass::DeadEndThrash],
        ),
    ];
    let summary = export_summary(vec![session_alpha, session_beta], checkpoints);

    assert!(summary.contains("Flagged checkpoint rate: `0.67`"));
    assert!(summary.contains("Verification density: `0.20`"));
    assert!(summary.contains("Average evidence items per checkpoint: `7.00`"));
}

#[test]
fn export_bundle_renders_unavailable_v0_3_metrics_for_missing_samples() {
    let session_empty = fixture_session(
        "session-empty",
        vec![fixture_row(
            "session-empty",
            0,
            CompactionKind::UserMessage,
            "/goal Empty",
            Some(UserMessageRole::Prompt),
        )],
    );
    let session_solo = fixture_session(
        "session-solo",
        vec![
            fixture_row(
                "session-solo",
                0,
                CompactionKind::UserMessage,
                "/goal Solo",
                Some(UserMessageRole::Prompt),
            ),
            fixture_row(
                "session-solo",
                1,
                CompactionKind::AssistantMessage,
                "Thinking",
                None,
            ),
        ],
    );
    let checkpoints = vec![fixture_checkpoint(
        &session_solo,
        1,
        1,
        Confidence::Medium,
        CheckpointDiagnostics::default(),
        &[],
    )];
    let summary = export_summary(vec![session_empty, session_solo], checkpoints);

    assert!(summary.contains("## session-empty"));
    assert!(summary.contains("- Task-frame confidence distribution: `unavailable`"));
    assert!(summary.contains("- Average evidence items per checkpoint: `unavailable`"));
    assert!(summary.contains("## session-solo"));
    assert!(summary.contains("- Working-set churn: `unavailable`"));
    assert!(summary.contains("- Verification density: `unavailable`"));
}

#[test]
fn export_bundle_uses_interval_counters_for_verification_density() {
    let fixture = BundleFixture::sample();
    let result = agent_drift_analyzer::analyze_bundle(&agent_drift_analyzer::AnalyzeRequest {
        input_dir: fixture.input_dir.clone(),
        output_dir: fixture.output_dir.clone(),
    })
    .expect("analyze sample bundle");
    let checkpoints = read_checkpoints(&result.checkpoints_path);
    let summary = fs::read_to_string(&result.summary_path).expect("summary");
    let stats = summarize_checkpoint_diagnostics(&checkpoints);

    assert_eq!(stats.total_interval_command_count, 7);
    assert_eq!(stats.total_interval_verification_command_count, 3);
    assert_optional_metric_eq(stats.verification_density(), 3.0 / 7.0);
    assert!(summary.contains("Verification density: `0.43`"));
    assert!(!summary.contains("Verification density: `0.45`"));
}

fn assert_optional_metric_eq(actual: Option<f64>, expected: f64) {
    let actual = actual.expect("metric should be available");
    assert!(
        (actual - expected).abs() < f64::EPSILON,
        "expected {expected}, got {actual}"
    );
}

fn export_summary(sessions: Vec<BundleSession>, checkpoints: Vec<Checkpoint>) -> String {
    let temp_dir = TempDir::new().expect("temp dir");
    let output_dir = Utf8PathBuf::from_path_buf(temp_dir.path().join("output")).expect("utf8");
    let result = export_checkpoints(&output_dir, &sessions, &checkpoints).expect("export");
    fs::read_to_string(result.summary_path).expect("summary")
}

fn fixture_session(session_id: &str, rows: Vec<CompactionRow>) -> BundleSession {
    BundleSession {
        session_id: session_id.to_string(),
        archival_rows: rows.clone(),
        compact_rows: rows,
    }
}

fn fixture_row(
    session_id: &str,
    event_index: usize,
    kind: CompactionKind,
    text: &str,
    user_message_role: Option<UserMessageRole>,
) -> CompactionRow {
    CompactionRow {
        source_file: Utf8PathBuf::from(format!("/tmp/{session_id}/rollout.jsonl")),
        source_kind: SourceKind::CodexRolloutJsonl,
        session_id: Some(session_id.to_string()),
        turn_id: Some("turn-001".to_string()),
        event_index,
        line_number: event_index + 1,
        row_ordinal: 0,
        timestamp: None,
        kind,
        user_message_role,
        dedupe_identity: None,
        text: text.to_string(),
        canonical_text: text.to_string(),
        text_hash_hex: format!("{session_id}-{event_index}"),
    }
}

fn fixture_checkpoint(
    session: &BundleSession,
    ordinal: usize,
    boundary_row_index: usize,
    confidence: Confidence,
    diagnostics: CheckpointDiagnostics,
    flagged_classes: &[DriftClass],
) -> Checkpoint {
    let boundary_row = &session.compact_rows[boundary_row_index];
    let working_set_paths = (ordinal > 1)
        .then_some(vec![format!(
            "src/{}/updated-{ordinal}.rs",
            session.session_id
        )])
        .unwrap_or_else(|| vec![format!("src/{}/base.rs", session.session_id)]);

    Checkpoint {
        schema_version: "v0.2".to_string(),
        session_id: session.session_id.clone(),
        checkpoint_id: format!("{}:{ordinal:04}", session.session_id),
        ordinal,
        boundary: CheckpointBoundary {
            start: RowRef::from_row(boundary_row),
            end: RowRef::from_row(boundary_row),
        },
        diagnostics,
        task_frame: TaskFrame {
            objective: format!("Objective {}", session.session_id),
            confidence,
            truth_artifacts: vec![format!("docs/{}.md", session.session_id)],
            working_set_paths,
            tools: vec!["functions.shell_command".to_string()],
            command_families: vec!["cargo".to_string()],
            verification_commands: vec!["cargo test".to_string()],
            supporting_evidence: vec![EvidenceRef {
                row: RowRef::from_row(boundary_row),
                reason: format!("supporting {}", session.session_id),
            }],
            counter_evidence: Vec::new(),
        },
        drift_scores: [
            DriftClass::WrongPlanBranch,
            DriftClass::IgnoringRepoTruth,
            DriftClass::DeadEndThrash,
        ]
        .into_iter()
        .map(|class| DriftScore {
            class,
            raw_score: if flagged_classes.contains(&class) {
                80
            } else {
                0
            },
            confidence,
            flagged: flagged_classes.contains(&class),
            evidence: Vec::new(),
        })
        .collect(),
        expected_next_step: "continue on the current task frame".to_string(),
        flagged: !flagged_classes.is_empty(),
    }
}

fn format_optional_metric(metric: Option<f64>) -> String {
    metric
        .map(|value| format!("{value:.2}"))
        .unwrap_or_else(|| "unavailable".to_string())
}

fn format_drift_class_frequencies(
    stats: &agent_drift_analyzer::checkpoint::CheckpointDiagnosticStats,
) -> String {
    [
        DriftClass::WrongPlanBranch,
        DriftClass::IgnoringRepoTruth,
        DriftClass::DeadEndThrash,
    ]
    .into_iter()
    .map(|class| {
        let label = match class {
            DriftClass::WrongPlanBranch => "wrong_plan_branch",
            DriftClass::IgnoringRepoTruth => "ignoring_repo_truth",
            DriftClass::DeadEndThrash => "dead_end_thrash",
        };
        format!(
            "{label}={}",
            format_optional_metric(stats.drift_class_flagged_rate(class))
        )
    })
    .collect::<Vec<_>>()
    .join(", ")
}

fn format_confidence_distribution(
    stats: &agent_drift_analyzer::checkpoint::CheckpointDiagnosticStats,
) -> String {
    (stats.checkpoint_count > 0)
        .then(|| {
            format!(
                "low={}, medium={}, high={}",
                stats.confidence_distribution.low,
                stats.confidence_distribution.medium,
                stats.confidence_distribution.high
            )
        })
        .unwrap_or_else(|| "unavailable".to_string())
}
