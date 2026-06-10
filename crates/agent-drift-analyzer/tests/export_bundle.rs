#![allow(unused_crate_dependencies)]

mod support;

use std::collections::BTreeSet;
use std::fs;

use agent_drift_analyzer::checkpoint::{
    build_session_checkpoint, export_checkpoints, summarize_checkpoint_diagnostics,
    CheckpointDiagnostics,
};
use agent_drift_analyzer::{
    BundleSession, Checkpoint, CheckpointBoundary, Confidence, DriftClass, DriftScore, DriftState,
    EvidenceRef, ProgressDimension, ProgressStatus, SessionArchetype, SessionArchetypeLabel,
    SessionProgress, TaskFrame,
};
use agent_session_compactor::{CompactionKind, CompactionRow, RowRef, SourceKind, UserMessageRole};
use camino::Utf8PathBuf;
use serde_json::Value;
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
    assert!(summary.contains(
        "- Turn-context overview: `turn-001 (#1); checkpoints in turn 1-2; modes verification_heavy -> autonomous`"
    ));
    assert!(summary.contains(
        "  turn: `turn-001 (#1) rows=9 checkpoints=1 session-prompts=1 mode=verification_heavy activity[dir=2 asst=0 tool=5 read=2 write=0 verify=3 out=2]`"
    ));
    assert!(summary.contains(
        "  turn: `turn-001 (#1) rows=13 checkpoints=2 session-prompts=1 mode=autonomous activity[dir=2 asst=1 tool=8 read=2 write=2 verify=4 out=2]`"
    ));
    assert!(summary.contains(
        "  delegation: `topology=single_agent visibility=none confidence=high markers=none support[none] counter[none]`"
    ));
}

#[test]
fn export_bundle_renders_compact_delegation_inspection_for_delegated_parent_sessions() {
    let session = BundleSession {
        session_id: "session-delegation-summary".to_string(),
        archival_rows: vec![
            fixture_row(
                "session-delegation-summary",
                0,
                CompactionKind::UserMessage,
                "/goal Inspect delegation summary only.",
                Some(UserMessageRole::Prompt),
            ),
            fixture_tool_row(
                "session-delegation-summary",
                1,
                "spawn_agent",
                "{\"agent_type\":\"worker\"}",
            ),
        ],
        compact_rows: vec![fixture_tool_row(
            "session-delegation-summary",
            2,
            "functions.shell_command",
            "{\"command\":\"printf 'child rollout ' && sed -n '1,40p' /Users/spensermcconnell/.codex/sessions/2026/06/08/rollout-2026-06-08T12-00-00-019ea111-1111-7111-8111-111111111111.jsonl\",\"workdir\":\"/repo\"}",
        )],
    };
    let task_frame = TaskFrame {
        objective: "Inspect delegation summary".to_string(),
        confidence: Confidence::Medium,
        truth_artifacts: vec!["docs/spec.md".to_string()],
        working_set_paths: vec!["crates/agent-drift-analyzer/src/checkpoint/mod.rs".to_string()],
        tools: vec!["functions.shell_command".to_string()],
        command_families: vec!["shell".to_string()],
        verification_commands: vec![
            "cargo test -p agent-drift-analyzer checkpoints -- --nocapture".to_string(),
        ],
        supporting_evidence: Vec::new(),
        counter_evidence: Vec::new(),
    };
    let checkpoint = build_session_checkpoint(&session, 1, &task_frame, Vec::new());
    let summary = export_summary(vec![session], vec![checkpoint]);

    assert!(summary.contains(
        "topology=delegating_parent visibility=partial confidence=medium markers=spawn_agent"
    ));
    assert!(summary.contains("delegation marker: spawn_agent"));
    assert!(summary.contains("delegation child rollout surface links child/subagent work"));
    assert!(summary.contains("counter[none]"));
}

#[test]
fn export_bundle_renders_compact_delegation_inspection_for_ambiguous_opaque_sessions() {
    let session = BundleSession {
        session_id: "session-delegation-ambiguous".to_string(),
        archival_rows: vec![
            fixture_row(
                "session-delegation-ambiguous",
                0,
                CompactionKind::UserMessage,
                "/goal Inspect delegation summary only.",
                Some(UserMessageRole::Prompt),
            ),
            fixture_tool_row(
                "session-delegation-ambiguous",
                1,
                "multi_agent_v1",
                "{\"mode\":\"delegated\"}",
            ),
            fixture_row(
                "session-delegation-ambiguous",
                2,
                CompactionKind::DeveloperMessage,
                "Child session id 019ea222-2222-7222-8222-222222222222 remains in separate rollout /Users/spensermcconnell/.codex/sessions/2026/06/08/rollout-2026-06-08T12-30-00-019ea222-2222-7222-8222-222222222222.jsonl",
                None,
            ),
        ],
        compact_rows: vec![
            fixture_row(
                "session-delegation-ambiguous",
                0,
                CompactionKind::UserMessage,
                "/goal Inspect delegation summary only.",
                Some(UserMessageRole::Prompt),
            ),
            fixture_tool_row(
                "session-delegation-ambiguous",
                1,
                "multi_agent_v1",
                "{\"mode\":\"delegated\"}",
            ),
            fixture_row(
                "session-delegation-ambiguous",
                2,
                CompactionKind::DeveloperMessage,
                "Child session id 019ea222-2222-7222-8222-222222222222 remains in separate rollout /Users/spensermcconnell/.codex/sessions/2026/06/08/rollout-2026-06-08T12-30-00-019ea222-2222-7222-8222-222222222222.jsonl",
                None,
            ),
        ],
    };
    let task_frame = TaskFrame {
        objective: "Inspect delegation summary".to_string(),
        confidence: Confidence::Medium,
        truth_artifacts: vec!["docs/spec.md".to_string()],
        working_set_paths: vec!["crates/agent-drift-analyzer/src/checkpoint/mod.rs".to_string()],
        tools: vec!["functions.shell_command".to_string()],
        command_families: vec!["shell".to_string()],
        verification_commands: vec![
            "cargo test -p agent-drift-analyzer checkpoints -- --nocapture".to_string(),
        ],
        supporting_evidence: Vec::new(),
        counter_evidence: Vec::new(),
    };
    let checkpoint = build_session_checkpoint(&session, 2, &task_frame, Vec::new());
    let summary = export_summary(vec![session], vec![checkpoint]);

    assert!(summary.contains(
        "topology=mixed_or_ambiguous visibility=opaque confidence=low markers=multi_agent_v1"
    ));
    assert!(summary.contains("delegation directive surface references separate child rollout"));
}

#[test]
fn export_bundle_renders_compact_session_archetype_inspection() {
    let session = fixture_session(
        "session-archetype-summary",
        vec![
            fixture_row(
                "session-archetype-summary",
                0,
                CompactionKind::UserMessage,
                "/goal Lock the Packet R4-3 summary surface.",
                Some(UserMessageRole::Prompt),
            ),
            fixture_tool_row(
                "session-archetype-summary",
                1,
                "functions.apply_patch",
                "{\"command\":\"apply_patch <<'PATCH'\\n*** Begin Patch\\n*** Update File: crates/agent-drift-analyzer/src/checkpoint/export.rs\\n*** End Patch\\nPATCH\",\"workdir\":\"/repo\"}",
            ),
        ],
    );
    let checkpoint = fixture_checkpoint_with_archetype(
        &session,
        1,
        1,
        Confidence::Medium,
        SessionArchetype {
            label: SessionArchetypeLabel::AutonomousImplementation,
            confidence: Confidence::Medium,
            supporting_evidence: vec![
                EvidenceRef {
                    row: RowRef::from_row(&session.compact_rows[1]),
                    reason: "stable working set plus source edits supported concentrated implementation"
                        .to_string(),
                },
                EvidenceRef {
                    row: RowRef::from_row(&session.compact_rows[1]),
                    reason: "source edit command strengthened implementation-like evidence"
                        .to_string(),
                },
                EvidenceRef {
                    row: RowRef::from_row(&session.compact_rows[1]),
                    reason: "local verification against source scope also supported implementation follow-through"
                        .to_string(),
                },
            ],
            counter_evidence: vec![EvidenceRef {
                row: RowRef::from_row(&session.compact_rows[0]),
                reason: "inspection-style command widened the visible search space".to_string(),
            }],
        },
    );
    let summary = export_summary(vec![session], vec![checkpoint]);

    assert!(summary.contains("  archetype: `label=autonomous_implementation confidence=medium"));
    assert!(summary
        .contains("support[stable working set plus source edits supported concentrated i..."));
    assert!(summary.contains("source edit command strengthened implementation-like evidence"));
    assert!(summary.contains("+1 more]"));
    assert!(summary.contains("counter[inspection-style command widened the visible search space]`"));
}

#[test]
fn export_bundle_renders_progress_distribution_and_compact_progress_lines() {
    let fixture = BundleFixture::sample();
    let result = agent_drift_analyzer::analyze_bundle(&agent_drift_analyzer::AnalyzeRequest {
        input_dir: fixture.input_dir.clone(),
        output_dir: fixture.output_dir.clone(),
    })
    .expect("analyze sample bundle");
    let summary = fs::read_to_string(&result.summary_path).expect("summary");
    let checkpoints = read_checkpoints(&result.checkpoints_path);

    let expected_status_distribution = format_progress_status_distribution(&checkpoints);
    let expected_dimension_distribution = format_progress_dimension_distribution(&checkpoints);

    assert!(summary.contains(&format!(
        "Progress status distribution: `{expected_status_distribution}`"
    )));
    assert!(summary.contains(&format!(
        "Progress dimension distribution: `{expected_dimension_distribution}`"
    )));
    assert!(summary.contains(&format!(
        "- Progress status distribution: `{expected_status_distribution}`"
    )));
    assert!(summary.contains(&format!(
        "- Progress dimension distribution: `{expected_dimension_distribution}`"
    )));

    for checkpoint in &checkpoints {
        assert!(summary.contains(&format!(
            "  progress: `{}`",
            format_checkpoint_progress(checkpoint.session_progress.as_ref())
        )));
    }
}

#[test]
fn export_bundle_renders_unavailable_progress_for_legacy_checkpoint_surfaces() {
    let session = fixture_session(
        "session-legacy-progress",
        vec![fixture_row(
            "session-legacy-progress",
            0,
            CompactionKind::UserMessage,
            "/goal Preserve legacy export rendering.",
            Some(UserMessageRole::Prompt),
        )],
    );
    let checkpoint = fixture_checkpoint_with_archetype(
        &session,
        1,
        0,
        Confidence::Low,
        SessionArchetype {
            label: SessionArchetypeLabel::Planning,
            confidence: Confidence::Low,
            supporting_evidence: Vec::new(),
            counter_evidence: Vec::new(),
        },
    );
    let summary = export_summary(vec![session], vec![checkpoint]);

    assert!(summary
        .contains("  archetype: `label=planning confidence=low support[none] counter[none]`"));
    assert!(summary.contains("  progress: `unavailable`"));
}

#[test]
fn export_bundle_distinguishes_many_short_conversational_turns_in_summary() {
    let mut bundle = load_sample_bundle();
    for row in bundle
        .archival_rows
        .iter_mut()
        .chain(bundle.compact_rows.iter_mut())
        .filter(|row| row.event_index >= 9)
    {
        row.turn_id = Some("turn-002".to_string());
        match row.event_index {
            10 => {
                row.kind = CompactionKind::AssistantMessage;
                row.text = "I am summarizing the next patch step.".to_string();
                row.dedupe_identity = None;
            }
            11 => {
                row.kind = CompactionKind::DeveloperMessage;
                row.text = "Stay inside Packet R3-5 only.".to_string();
                row.dedupe_identity = None;
            }
            12 => {
                row.kind = CompactionKind::AssistantMessage;
                row.text = "Waiting for the next instruction.".to_string();
                row.dedupe_identity = None;
            }
            _ => {}
        }
    }

    let fixture = BundleFixture::from_rows(
        bundle.archival_rows,
        bundle.compact_rows,
        bundle.dedupe_groups,
    );
    let result = agent_drift_analyzer::analyze_bundle(&agent_drift_analyzer::AnalyzeRequest {
        input_dir: fixture.input_dir.clone(),
        output_dir: fixture.output_dir.clone(),
    })
    .expect("analyze conversational turn bundle");
    let summary = fs::read_to_string(&result.summary_path).expect("summary");

    assert!(summary.contains("Turns observed: `2`"));
    assert!(summary.contains("- Checkpoints per turn: `1.00`"));
    assert!(summary.contains(
        "- Turn-context overview: `turn-001 (#1) -> turn-002 (#2); checkpoints in turn 1; modes verification_heavy -> conversational`"
    ));
    assert!(summary.contains(
        "  turn: `turn-001 (#1) rows=9 checkpoints=1 session-prompts=1 mode=verification_heavy activity[dir=2 asst=0 tool=5 read=2 write=0 verify=3 out=2]`"
    ));
    assert!(summary.contains(
        "  turn: `turn-002 (#2) rows=4 checkpoints=1 session-prompts=1 mode=conversational activity[dir=1 asst=3 tool=0 read=0 write=0 verify=0 out=0]`"
    ));
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
        .all(|checkpoint| checkpoint.schema_version == "v0.6"));
    assert!(checkpoints
        .iter()
        .all(|checkpoint| checkpoint.session_progress.is_some()));

    let first = &checkpoints[0];
    let second = &checkpoints[1];

    assert!(!first.diagnostics.task_frame_transitioned);
    assert!(!first.diagnostics.working_set_changed);
    assert_eq!(first.diagnostics.interval_command_count, 4);
    assert_eq!(first.diagnostics.interval_verification_command_count, 2);
    assert!(first.diagnostics.evidence_item_count > 0);
    assert!(first
        .drift_scores
        .iter()
        .all(|score| matches!(score.state, DriftState::Active | DriftState::Cleared)));

    assert!(second.diagnostics.task_frame_transitioned);
    assert!(second.diagnostics.working_set_changed);
    assert_eq!(second.diagnostics.interval_command_count, 3);
    assert_eq!(second.diagnostics.interval_verification_command_count, 1);
    assert!(second.diagnostics.evidence_item_count > 0);
    assert!(second
        .drift_scores
        .iter()
        .all(|score| matches!(score.state, DriftState::Active | DriftState::Recovered)));
}

#[test]
fn export_bundle_writes_raw_state_fields_for_every_drift_score() {
    let fixture = BundleFixture::sample();
    let result = agent_drift_analyzer::analyze_bundle(&agent_drift_analyzer::AnalyzeRequest {
        input_dir: fixture.input_dir.clone(),
        output_dir: fixture.output_dir.clone(),
    })
    .expect("analyze sample bundle");
    let raw_checkpoints = fs::read_to_string(&result.checkpoints_path).expect("checkpoints jsonl");
    let mut saw_cleared_state = false;

    for line in raw_checkpoints.lines() {
        let checkpoint: Value = serde_json::from_str(line).expect("checkpoint json");
        let drift_scores = checkpoint["drift_scores"]
            .as_array()
            .expect("checkpoint drift_scores array");
        assert!(
            !drift_scores.is_empty(),
            "checkpoint must include drift scores"
        );

        for score in drift_scores {
            let state = score
                .get("state")
                .and_then(Value::as_str)
                .expect("serialized drift score state");
            assert!(matches!(
                state,
                "active" | "recovered" | "historical_only" | "cleared"
            ));
            saw_cleared_state |= state == "cleared";
        }
    }

    assert!(
        saw_cleared_state,
        "expected at least one serialized cleared state"
    );
}

#[test]
fn export_bundle_keeps_dead_end_thrash_active_when_recovery_interval_stays_out_of_scope() {
    let fixture = BundleFixture::sample();
    let result = agent_drift_analyzer::analyze_bundle(&agent_drift_analyzer::AnalyzeRequest {
        input_dir: fixture.input_dir.clone(),
        output_dir: fixture.output_dir.clone(),
    })
    .expect("analyze sample bundle");
    let checkpoints = read_checkpoints(&result.checkpoints_path);
    let recovered = checkpoints[1]
        .drift_scores
        .iter()
        .find(|score| score.class == DriftClass::DeadEndThrash)
        .expect("recovered dead end thrash score");

    assert!(recovered.flagged);
    assert!(recovered.raw_score >= 60);
    assert!(recovered
        .evidence
        .iter()
        .any(|item| item.reason == "repeated failure evidence"));
}

#[test]
fn export_bundle_preserves_historical_dead_end_thrash_evidence_after_clean_recovery() {
    let fixture = BundleFixture::clean_recovery();
    let result = agent_drift_analyzer::analyze_bundle(&agent_drift_analyzer::AnalyzeRequest {
        input_dir: fixture.input_dir.clone(),
        output_dir: fixture.output_dir.clone(),
    })
    .expect("analyze sample bundle");
    let checkpoints = read_checkpoints(&result.checkpoints_path);
    let recovered = checkpoints[1]
        .drift_scores
        .iter()
        .find(|score| score.class == DriftClass::DeadEndThrash)
        .expect("recovered dead end thrash score");

    assert!(!recovered.flagged);
    assert_eq!(recovered.raw_score, 20);
    assert!(recovered.evidence.iter().any(|item| item
        .reason
        .starts_with("historical repeated failure evidence:")));
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
        DriftClass::TruthGroundingGap,
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

#[test]
fn export_bundle_dedupes_duplicate_evidence_items_in_checkpoint_diagnostics() {
    let session = fixture_session(
        "session-evidence",
        vec![
            fixture_row(
                "session-evidence",
                0,
                CompactionKind::UserMessage,
                "/goal Evidence",
                Some(UserMessageRole::Prompt),
            ),
            fixture_row(
                "session-evidence",
                1,
                CompactionKind::ToolCall,
                "{\"command\":\"cargo test\"}",
                None,
            ),
        ],
    );
    let first_row = &session.compact_rows[0];
    let second_row = &session.compact_rows[1];
    let duplicated = EvidenceRef {
        row: RowRef::from_row(first_row),
        reason: "same reason".to_string(),
    };
    let unique = EvidenceRef {
        row: RowRef::from_row(second_row),
        reason: "unique reason".to_string(),
    };
    let task_frame = TaskFrame {
        objective: "Validate evidence dedupe".to_string(),
        confidence: Confidence::Medium,
        truth_artifacts: vec!["docs/spec.md".to_string()],
        working_set_paths: vec!["src/lib.rs".to_string()],
        tools: vec!["functions.shell_command".to_string()],
        command_families: vec!["cargo".to_string()],
        verification_commands: vec!["cargo test".to_string()],
        supporting_evidence: vec![duplicated.clone(), unique.clone()],
        counter_evidence: vec![duplicated.clone()],
    };
    let checkpoint = build_session_checkpoint(
        &session,
        1,
        &task_frame,
        vec![DriftScore {
            class: DriftClass::WrongPlanBranch,
            state: DriftState::Active,
            raw_score: 80,
            confidence: Confidence::Medium,
            flagged: true,
            evidence: vec![duplicated],
        }],
    );
    let stats = summarize_checkpoint_diagnostics(std::slice::from_ref(&checkpoint));
    let summary = export_summary(vec![session], vec![checkpoint.clone()]);

    assert_eq!(checkpoint.diagnostics.evidence_item_count, 2);
    assert_eq!(stats.total_evidence_item_count, 2);
    assert_optional_metric_eq(stats.average_evidence_items_per_checkpoint(), 2.0);
    assert!(summary.contains("Average evidence items per checkpoint: `2.00`"));
}

#[test]
fn build_session_checkpoint_preserves_requested_ordinal_when_out_of_range() {
    let session = fixture_session(
        "session-out-of-range",
        vec![
            fixture_row(
                "session-out-of-range",
                0,
                CompactionKind::UserMessage,
                "/goal Out of range",
                Some(UserMessageRole::Prompt),
            ),
            fixture_row(
                "session-out-of-range",
                1,
                CompactionKind::ToolCall,
                "{\"command\":\"cargo test\"}",
                None,
            ),
        ],
    );
    let task_frame = TaskFrame {
        objective: "Preserve helper behavior".to_string(),
        confidence: Confidence::Medium,
        truth_artifacts: vec!["docs/spec.md".to_string()],
        working_set_paths: vec!["src/lib.rs".to_string()],
        tools: vec!["functions.shell_command".to_string()],
        command_families: vec!["cargo".to_string()],
        verification_commands: vec!["cargo test".to_string()],
        supporting_evidence: Vec::new(),
        counter_evidence: Vec::new(),
    };

    let checkpoint = build_session_checkpoint(&session, 99, &task_frame, Vec::new());

    assert_eq!(checkpoint.ordinal, 99);
    assert_eq!(checkpoint.checkpoint_id, "session-out-of-range:0099");
    assert_eq!(checkpoint.boundary.end.event_index, 1);
    assert_eq!(checkpoint.diagnostics.interval_command_count, 1);
    assert_eq!(
        checkpoint.diagnostics.interval_verification_command_count,
        1
    );
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

fn fixture_tool_row(
    session_id: &str,
    event_index: usize,
    tool_name: &str,
    text: &str,
) -> CompactionRow {
    let mut row = fixture_row(
        session_id,
        event_index,
        CompactionKind::ToolCall,
        text,
        None,
    );
    row.dedupe_identity = Some(format!(
        "{{\"call_id\":\"call-{event_index}\",\"name\":\"{tool_name}\",\"type\":\"function_call\"}}"
    ));
    row
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
        schema_version: "v0.4".to_string(),
        session_id: session.session_id.clone(),
        checkpoint_id: format!("{}:{ordinal:04}", session.session_id),
        ordinal,
        boundary: CheckpointBoundary {
            start: RowRef::from_row(boundary_row),
            end: RowRef::from_row(boundary_row),
        },
        turn_context: None,
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
        session_archetype: None,
        session_progress: None,
        drift_scores: [
            DriftClass::WrongPlanBranch,
            DriftClass::TruthGroundingGap,
            DriftClass::DeadEndThrash,
        ]
        .into_iter()
        .map(|class| DriftScore {
            class,
            state: if flagged_classes.contains(&class) {
                DriftState::Active
            } else {
                DriftState::Cleared
            },
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

fn fixture_checkpoint_with_archetype(
    session: &BundleSession,
    ordinal: usize,
    boundary_row_index: usize,
    confidence: Confidence,
    session_archetype: SessionArchetype,
) -> Checkpoint {
    let mut checkpoint = fixture_checkpoint(
        session,
        ordinal,
        boundary_row_index,
        confidence,
        CheckpointDiagnostics::default(),
        &[],
    );
    checkpoint.schema_version = "v0.5".to_string();
    checkpoint.session_archetype = Some(session_archetype);
    checkpoint
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
        DriftClass::TruthGroundingGap,
        DriftClass::DeadEndThrash,
    ]
    .into_iter()
    .map(|class| {
        let label = match class {
            DriftClass::WrongPlanBranch => "wrong_plan_branch",
            DriftClass::TruthGroundingGap => "truth_grounding_gap",
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

fn format_progress_status_distribution(checkpoints: &[Checkpoint]) -> String {
    let observed = checkpoints
        .iter()
        .filter_map(|checkpoint| checkpoint.session_progress.as_ref())
        .collect::<Vec<_>>();
    if observed.is_empty() {
        return "unavailable".to_string();
    }

    [
        ProgressStatus::Advancing,
        ProgressStatus::Mixed,
        ProgressStatus::Stalled,
        ProgressStatus::Regressing,
        ProgressStatus::InsufficientEvidence,
    ]
    .into_iter()
    .map(|status| {
        format!(
            "{}={}",
            format_progress_status(status),
            observed
                .iter()
                .filter(|progress| progress.status == status)
                .count()
        )
    })
    .collect::<Vec<_>>()
    .join(", ")
}

fn format_progress_dimension_distribution(checkpoints: &[Checkpoint]) -> String {
    let observed = checkpoints
        .iter()
        .filter_map(|checkpoint| checkpoint.session_progress.as_ref())
        .collect::<Vec<_>>();
    if observed.is_empty() {
        return "unavailable".to_string();
    }

    [
        ProgressDimension::TroubleshootingFrontier,
        ProgressDimension::PlanningConvergence,
        ProgressDimension::ImplementationVerificationWall,
        ProgressDimension::VerificationCloseoutNarrowing,
        ProgressDimension::ParentVisibleOrchestration,
    ]
    .into_iter()
    .map(|dimension| {
        format!(
            "{}={}",
            format_progress_dimension(dimension),
            observed
                .iter()
                .filter(|progress| progress.dimension == dimension)
                .count()
        )
    })
    .collect::<Vec<_>>()
    .join(", ")
}

fn format_checkpoint_progress(progress: Option<&SessionProgress>) -> String {
    let Some(progress) = progress else {
        return "unavailable".to_string();
    };

    format!(
        "status={} dimension={} confidence={} support[{}] counter[{}]",
        format_progress_status(progress.status),
        format_progress_dimension(progress.dimension),
        match progress.confidence {
            Confidence::Low => "low",
            Confidence::Medium => "medium",
            Confidence::High => "high",
        },
        format_progress_support(progress),
        format_reasons(&progress.counter_evidence)
    )
}

fn format_progress_status(status: ProgressStatus) -> &'static str {
    match status {
        ProgressStatus::Advancing => "advancing",
        ProgressStatus::Mixed => "mixed",
        ProgressStatus::Stalled => "stalled",
        ProgressStatus::Regressing => "regressing",
        ProgressStatus::InsufficientEvidence => "insufficient_evidence",
    }
}

fn format_progress_dimension(dimension: ProgressDimension) -> &'static str {
    match dimension {
        ProgressDimension::TroubleshootingFrontier => "troubleshooting_frontier",
        ProgressDimension::PlanningConvergence => "planning_convergence",
        ProgressDimension::ImplementationVerificationWall => "implementation_verification_wall",
        ProgressDimension::VerificationCloseoutNarrowing => "verification_closeout_narrowing",
        ProgressDimension::ParentVisibleOrchestration => "parent_visible_orchestration",
    }
}

fn format_progress_support(progress: &SessionProgress) -> String {
    let mut summaries = progress
        .signals
        .iter()
        .map(|signal| truncate_for_summary(&signal.summary, 64))
        .fold(Vec::<String>::new(), |mut acc, summary| {
            if acc.iter().all(|existing| existing != &summary) {
                acc.push(summary);
            }
            acc
        });

    if summaries.is_empty() {
        return format_reasons(&progress.supporting_evidence);
    }

    let remaining = summaries.len().saturating_sub(2);
    summaries.truncate(2);
    if remaining > 0 {
        summaries.push(format!("+{remaining} more"));
    }

    summaries.join("; ")
}

fn format_reasons(evidence: &[EvidenceRef]) -> String {
    if evidence.is_empty() {
        return "none".to_string();
    }

    let mut reasons = Vec::<String>::new();
    for item in evidence {
        if reasons.iter().any(|reason| reason == &item.reason) {
            continue;
        }
        reasons.push(truncate_for_summary(&item.reason, 64));
    }

    let remaining = reasons.len().saturating_sub(2);
    reasons.truncate(2);
    if remaining > 0 {
        reasons.push(format!("+{remaining} more"));
    }

    reasons.join("; ")
}

fn truncate_for_summary(text: &str, limit: usize) -> String {
    if text.chars().count() <= limit {
        return text.to_string();
    }

    let truncated = text
        .chars()
        .take(limit.saturating_sub(3))
        .collect::<String>();
    format!("{truncated}...")
}
