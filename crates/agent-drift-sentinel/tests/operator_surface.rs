#![allow(unused_crate_dependencies)]

mod support;

use agent_drift_analyzer::{
    Checkpoint, Confidence, DriftClass, DriftState, EvidenceRef, ProgressDimension, ProgressStatus,
    SessionArchetype, SessionArchetypeLabel, SessionProgress, TurnActivityMix, TurnContext,
    TurnExecutionMode,
};
use agent_drift_sentinel::{
    execute,
    operator_surface::{present_checkpoint_with_previous, warning_fingerprint, CheckpointPosture},
    scheduler::ReplayScheduler,
    AdjudicationConfig, SchedulerPolicy, SentinelMode, SentinelRequest, TriggerClass,
    WarningPolicy,
};

#[test]
fn operator_surface_renders_evidence_backed_visible_warning_blocks() {
    let fixture = support::ReplayFixture::sample();
    let result = execute(&SentinelRequest {
        checkpoint_dir: fixture.checkpoint_dir.clone(),
        mode: SentinelMode::Replay,
        cursor: None,
        scheduler_policy: SchedulerPolicy::default(),
        warning_policy: WarningPolicy::default(),
        adjudication: AdjudicationConfig::default(),
    })
    .expect("run replay");

    let rendered = result.report.to_console_text();

    assert!(rendered.contains("Agent Drift Sentinel Replay"));
    assert!(rendered.contains("Visible warnings:"));
    assert!(rendered.contains(
        "Diagnostics: task_frame_transitioned=true, working_set_changed=false, verification=1/1 (100.00%), evidence_items=2"
    ));
    assert!(rendered.contains("Expected next step: align plan to repo truth"));
    assert!(
        rendered.contains("Evidence: session-alpha.jsonl#1:0 flagged score for session-alpha:1")
    );
}

#[test]
fn operator_surface_renders_diagnostics_for_silent_checkpoints() {
    let fixture = support::ReplayFixture::sample();
    let result = execute(&SentinelRequest {
        checkpoint_dir: fixture.checkpoint_dir.clone(),
        mode: SentinelMode::Replay,
        cursor: None,
        scheduler_policy: SchedulerPolicy::default(),
        warning_policy: WarningPolicy::default(),
        adjudication: AdjudicationConfig::default(),
    })
    .expect("run replay");

    let rendered = result.report.to_console_text();

    assert!(rendered.contains("Silent checkpoints:"));
    assert!(rendered.contains(
        "Diagnostics: task_frame_transitioned=false, working_set_changed=false, verification=1/1 (100.00%), evidence_items=2"
    ));
    assert!(rendered.contains("Silent reason: scheduler cooldown deferred replay evaluation"));
}

#[test]
fn operator_surface_replay_flagged_checkpoints_keep_checkpoint_ready_headlines() {
    let fixture = support::ReplayFixture::sample();
    let result = execute(&SentinelRequest {
        checkpoint_dir: fixture.checkpoint_dir.clone(),
        mode: SentinelMode::Replay,
        cursor: None,
        scheduler_policy: SchedulerPolicy::default(),
        warning_policy: WarningPolicy::default(),
        adjudication: AdjudicationConfig::default(),
    })
    .expect("run replay");

    let visible = &result.report.visible_warnings[0];

    assert!(visible.checkpoint.flagged);
    assert!(visible.headline.contains("checkpoint_ready"));
    assert!(!visible
        .headline
        .contains("scheduler_repeated_failure_trigger"));
    assert_eq!(visible.posture, Some(CheckpointPosture::Active));
}

#[test]
fn operator_surface_renders_unavailable_density_for_zero_command_checkpoints() {
    let mut checkpoints = support::sample_checkpoints();
    checkpoints[0].diagnostics.interval_command_count = 0;
    checkpoints[0]
        .diagnostics
        .interval_verification_command_count = 0;
    let fixture = support::ReplayFixture::from_checkpoints(checkpoints, support::sample_summary());
    let result = execute(&SentinelRequest {
        checkpoint_dir: fixture.checkpoint_dir.clone(),
        mode: SentinelMode::Replay,
        cursor: None,
        scheduler_policy: SchedulerPolicy::default(),
        warning_policy: WarningPolicy::default(),
        adjudication: AdjudicationConfig::default(),
    })
    .expect("run replay");

    let rendered = result.report.to_console_text();

    assert!(rendered.contains(
        "Diagnostics: task_frame_transitioned=true, working_set_changed=false, verification=0/0 (unavailable), evidence_items=2"
    ));
}

#[test]
fn operator_surface_renders_compact_turn_context_for_v0_4_checkpoints() {
    let mut visible = checkpoint_with_drift(
        "session-turn-context",
        1,
        DriftClass::TruthGroundingGap,
        82,
        true,
        "align plan to repo truth",
        &["flagged score for session-turn-context:1"],
    );
    visible.schema_version = "v0.4".to_string();
    visible.turn_context = Some(sample_turn_context(1));

    let mut silent = checkpoint_with_drift(
        "session-turn-context",
        2,
        DriftClass::TruthGroundingGap,
        20,
        false,
        "continue on the current task frame",
        &["historical truth-grounding gap: flagged score for session-turn-context:1"],
    );
    silent.schema_version = "v0.4".to_string();
    silent.turn_context = Some(sample_turn_context(1));

    let fixture =
        support::ReplayFixture::from_checkpoints(vec![visible, silent], support::sample_summary());
    let result = execute(&SentinelRequest {
        checkpoint_dir: fixture.checkpoint_dir.clone(),
        mode: SentinelMode::Replay,
        cursor: None,
        scheduler_policy: SchedulerPolicy::default(),
        warning_policy: WarningPolicy::default(),
        adjudication: AdjudicationConfig::default(),
    })
    .expect("run replay");

    let rendered = result.report.to_console_text();

    assert!(rendered.contains(
        "- Turn context: turn-1 (#1) rows=4 elapsed=12s checkpoints=2 session-prompts=1 mode=autonomous activity[dir=1 asst=1 tool=1 read=1 write=1 verify=0 out=1]"
    ));
}

#[test]
fn operator_surface_renders_compact_archetype_inspection_for_v0_5_checkpoints() {
    let mut visible = checkpoint_with_schema_state(
        "v0.5",
        "session-archetype",
        1,
        DriftClass::TruthGroundingGap,
        DriftState::Active,
        82,
        true,
        "align plan to repo truth",
        &["flagged score for session-archetype:1"],
    );
    visible.turn_context = Some(sample_turn_context(1));
    visible.session_archetype = Some(sample_session_archetype(
        &visible,
        SessionArchetypeLabel::AutonomousImplementation,
    ));

    let mut silent = checkpoint_with_schema_state(
        "v0.5",
        "session-archetype",
        2,
        DriftClass::TruthGroundingGap,
        DriftState::Recovered,
        20,
        false,
        "continue on the current task frame",
        &["explicit analyzer recovery evidence"],
    );
    silent.turn_context = Some(sample_turn_context(2));
    silent.session_archetype = Some(sample_session_archetype(
        &silent,
        SessionArchetypeLabel::VerificationCloseout,
    ));

    let fixture =
        support::ReplayFixture::from_checkpoints(vec![visible, silent], support::sample_summary());
    let result = execute(&SentinelRequest {
        checkpoint_dir: fixture.checkpoint_dir.clone(),
        mode: SentinelMode::Replay,
        cursor: None,
        scheduler_policy: SchedulerPolicy::default(),
        warning_policy: WarningPolicy::default(),
        adjudication: AdjudicationConfig::default(),
    })
    .expect("run replay");

    let rendered = result.report.to_console_text();

    assert!(rendered.contains("- Turn context: turn-1 (#1)"));
    assert!(rendered.contains("- Archetype: label=autonomous_implementation confidence=medium"));
    assert!(rendered.contains("- Archetype: label=verification_closeout confidence=medium"));
    assert!(rendered.contains("source edit command strengthened implementation-like evidence"));
    assert!(rendered.contains("+1 more"));
    assert!(rendered.contains("counter[inspection-style command widened the visible search space]"));
    assert!(rendered.contains("- Diagnostics: task_frame_transitioned=true"));
}

#[test]
fn operator_surface_renders_compact_progress_inspection_for_v0_7_checkpoints() {
    let mut visible = checkpoint_with_schema_state(
        "v0.7",
        "session-progress",
        1,
        DriftClass::TruthGroundingGap,
        DriftState::Active,
        82,
        true,
        "align plan to repo truth",
        &["flagged score for session-progress:1"],
    );
    visible.turn_context = Some(sample_turn_context(1));
    visible.session_archetype = Some(sample_session_archetype(
        &visible,
        SessionArchetypeLabel::AutonomousImplementation,
    ));
    visible.session_progress = Some(sample_session_progress(
        &visible,
        ProgressStatus::Advancing,
        ProgressDimension::ImplementationVerificationWall,
    ));

    let mut silent = checkpoint_with_schema_state(
        "v0.7",
        "session-progress",
        2,
        DriftClass::TruthGroundingGap,
        DriftState::Recovered,
        20,
        false,
        "continue on the current task frame",
        &["explicit analyzer recovery evidence"],
    );
    silent.turn_context = Some(sample_turn_context(2));
    silent.session_archetype = Some(sample_session_archetype(
        &silent,
        SessionArchetypeLabel::VerificationCloseout,
    ));
    silent.session_progress = Some(sample_session_progress(
        &silent,
        ProgressStatus::Mixed,
        ProgressDimension::VerificationCloseoutNarrowing,
    ));

    let fixture =
        support::ReplayFixture::from_checkpoints(vec![visible, silent], support::sample_summary());
    let result = execute(&SentinelRequest {
        checkpoint_dir: fixture.checkpoint_dir.clone(),
        mode: SentinelMode::Replay,
        cursor: None,
        scheduler_policy: SchedulerPolicy::default(),
        warning_policy: WarningPolicy::default(),
        adjudication: AdjudicationConfig::default(),
    })
    .expect("run replay");

    let visible_render = result.report.visible_warnings[0].render_console_block(None);
    let silent_render = result.report.silent_checkpoints[0].render_console_block(None);

    assert!(visible_render.contains(
        "- Progress: status=advancing dimension=implementation_verification_wall confidence=medium"
    ));
    assert!(silent_render.contains(
        "- Progress: status=mixed dimension=verification_closeout_narrowing confidence=medium"
    ));
    assert!(silent_render.contains("- Posture: recovered"));
    assert!(silent_render
        .contains("Evidence: session-progress.jsonl#2:0 explicit analyzer recovery evidence"));
    assert_progress_line_after_archetype(&visible_render);
    assert_progress_line_after_archetype(&silent_render);
}

#[test]
fn operator_surface_classifies_active_recovered_and_historical_only_posture() {
    let fixture = support::ReplayFixture::from_checkpoints(
        vec![
            checkpoint_with_drift(
                "session-posture",
                1,
                DriftClass::TruthGroundingGap,
                82,
                true,
                "align plan to repo truth",
                &["flagged score for session-posture:1"],
            ),
            checkpoint_with_drift(
                "session-posture",
                2,
                DriftClass::TruthGroundingGap,
                20,
                false,
                "continue on the current task frame",
                &["historical truth-grounding gap: flagged score for session-posture:1"],
            ),
            checkpoint_with_drift(
                "session-posture",
                3,
                DriftClass::TruthGroundingGap,
                20,
                false,
                "continue on the current task frame",
                &["historical truth-grounding gap: flagged score for session-posture:1"],
            ),
        ],
        support::sample_summary(),
    );
    let result = execute(&SentinelRequest {
        checkpoint_dir: fixture.checkpoint_dir.clone(),
        mode: SentinelMode::Replay,
        cursor: None,
        scheduler_policy: SchedulerPolicy::default(),
        warning_policy: WarningPolicy::default(),
        adjudication: AdjudicationConfig::default(),
    })
    .expect("run replay");

    assert_eq!(
        result.report.visible_warnings[0].posture,
        Some(CheckpointPosture::Active)
    );
    let recovered = result
        .report
        .silent_checkpoints
        .iter()
        .find(|checkpoint| checkpoint.checkpoint.checkpoint_id == "session-posture:0002")
        .expect("recovered checkpoint");
    assert_eq!(recovered.posture, Some(CheckpointPosture::Recovered));
    assert!(recovered
        .evidence_lines
        .iter()
        .any(|line| line
            .contains("historical truth-grounding gap: flagged score for session-posture:1")));
    let historical_only = result
        .report
        .silent_checkpoints
        .iter()
        .find(|checkpoint| checkpoint.checkpoint.checkpoint_id == "session-posture:0003")
        .expect("historical-only checkpoint");
    assert_eq!(
        historical_only.posture,
        Some(CheckpointPosture::HistoricalOnly)
    );
    assert!(historical_only
        .evidence_lines
        .iter()
        .any(|line| line
            .contains("historical truth-grounding gap: flagged score for session-posture:1")));

    let rendered = result.report.to_console_text();
    assert!(rendered.contains("- Posture: active"));
    assert!(rendered.contains("- Posture: recovered"));
    assert!(rendered.contains("- Posture: historical-only"));
    assert!(
        rendered.contains("historical truth-grounding gap: flagged score for session-posture:1")
    );
}

#[test]
fn operator_surface_preserves_recovered_posture_when_replay_resumes_after_cursor() {
    let fixture = support::ReplayFixture::from_checkpoints(
        vec![
            checkpoint_with_drift(
                "session-posture",
                1,
                DriftClass::TruthGroundingGap,
                82,
                true,
                "align plan to repo truth",
                &["flagged score for session-posture:1"],
            ),
            checkpoint_with_drift(
                "session-posture",
                2,
                DriftClass::TruthGroundingGap,
                20,
                false,
                "continue on the current task frame",
                &["historical truth-grounding gap: flagged score for session-posture:1"],
            ),
            checkpoint_with_drift(
                "session-posture",
                3,
                DriftClass::TruthGroundingGap,
                20,
                false,
                "continue on the current task frame",
                &["historical truth-grounding gap: flagged score for session-posture:1"],
            ),
        ],
        support::sample_summary(),
    );
    let result = execute(&SentinelRequest {
        checkpoint_dir: fixture.checkpoint_dir.clone(),
        mode: SentinelMode::Replay,
        cursor: Some(agent_drift_sentinel::CheckpointCursor {
            session_id: "session-posture".to_string(),
            ordinal: 1,
        }),
        scheduler_policy: SchedulerPolicy::default(),
        warning_policy: WarningPolicy::default(),
        adjudication: AdjudicationConfig::default(),
    })
    .expect("run replay");

    assert_eq!(result.report.visible_warnings.len(), 0);
    let recovered = result
        .report
        .silent_checkpoints
        .iter()
        .find(|checkpoint| checkpoint.checkpoint.checkpoint_id == "session-posture:0002")
        .expect("recovered checkpoint after cursor");
    assert_eq!(recovered.posture, Some(CheckpointPosture::Recovered));

    let historical_only = result
        .report
        .silent_checkpoints
        .iter()
        .find(|checkpoint| checkpoint.checkpoint.checkpoint_id == "session-posture:0003")
        .expect("historical-only checkpoint after cursor");
    assert_eq!(
        historical_only.posture,
        Some(CheckpointPosture::HistoricalOnly)
    );
}

#[test]
fn operator_surface_prefers_explicit_v0_3_and_v0_4_state_without_previous_checkpoint() {
    for schema_version in ["v0.3", "v0.4"] {
        let recovered = checkpoint_with_schema_state(
            schema_version,
            "session-explicit-state",
            2,
            DriftClass::TruthGroundingGap,
            DriftState::Recovered,
            20,
            false,
            "continue on the current task frame",
            &["explicit analyzer recovery evidence"],
        );
        let historical_only = checkpoint_with_schema_state(
            schema_version,
            "session-explicit-state",
            3,
            DriftClass::TruthGroundingGap,
            DriftState::HistoricalOnly,
            20,
            false,
            "continue on the current task frame",
            &["explicit analyzer historical evidence"],
        );
        let mut scheduler = ReplayScheduler::new(SchedulerPolicy::default());

        let recovered_decision = scheduler.observe(
            agent_drift_sentinel::CheckpointCursor::from(&recovered),
            TriggerClass::CheckpointReady,
            recovered.flagged,
            Some(&warning_fingerprint(&recovered)),
        );
        let recovered_presentation = present_checkpoint_with_previous(
            &recovered,
            None,
            TriggerClass::CheckpointReady,
            &recovered_decision,
            &WarningPolicy::default(),
        );

        let historical_decision = scheduler.observe(
            agent_drift_sentinel::CheckpointCursor::from(&historical_only),
            TriggerClass::CheckpointReady,
            historical_only.flagged,
            Some(&warning_fingerprint(&historical_only)),
        );
        let historical_presentation = present_checkpoint_with_previous(
            &historical_only,
            None,
            TriggerClass::CheckpointReady,
            &historical_decision,
            &WarningPolicy::default(),
        );

        assert_eq!(
            recovered_presentation.posture,
            Some(CheckpointPosture::Recovered),
            "{schema_version} recovered posture should stay explicit-state backed"
        );
        assert_eq!(
            historical_presentation.posture,
            Some(CheckpointPosture::HistoricalOnly),
            "{schema_version} historical-only posture should stay explicit-state backed"
        );
        assert!(recovered_presentation
            .evidence_lines
            .iter()
            .any(|line| line.contains("explicit analyzer recovery evidence")));
        assert!(historical_presentation
            .evidence_lines
            .iter()
            .any(|line| line.contains("explicit analyzer historical evidence")));
    }
}

#[test]
fn operator_surface_detects_dead_end_historical_verification_evidence() {
    let fixture = support::ReplayFixture::from_checkpoints(
        vec![checkpoint_with_drift(
            "session-history",
            1,
            DriftClass::DeadEndThrash,
            20,
            false,
            "re-run verification deliberately",
            &["historical repeated verification evidence: repeated verification command: cargo test -p agent-drift-sentinel -- --nocapture"],
        )],
        support::sample_summary(),
    );
    let result = execute(&SentinelRequest {
        checkpoint_dir: fixture.checkpoint_dir.clone(),
        mode: SentinelMode::Replay,
        cursor: None,
        scheduler_policy: SchedulerPolicy::default(),
        warning_policy: WarningPolicy::default(),
        adjudication: AdjudicationConfig::default(),
    })
    .expect("run replay");

    assert_eq!(result.report.visible_warnings.len(), 0);
    assert_eq!(result.report.silent_checkpoints.len(), 1);
    assert_eq!(
        result.report.silent_checkpoints[0].posture,
        Some(CheckpointPosture::HistoricalOnly)
    );
    assert!(result.report.silent_checkpoints[0]
        .evidence_lines
        .iter()
        .any(|line| line.contains("historical repeated verification evidence:")));
}

#[test]
fn operator_surface_public_previous_aware_presenter_can_render_recovered_posture() {
    let previous = checkpoint_with_drift(
        "session-public",
        1,
        DriftClass::TruthGroundingGap,
        82,
        true,
        "align plan to repo truth",
        &["flagged score for session-public:1"],
    );
    let current = checkpoint_with_drift(
        "session-public",
        2,
        DriftClass::TruthGroundingGap,
        20,
        false,
        "continue on the current task frame",
        &["historical truth-grounding gap: flagged score for session-public:1"],
    );
    let mut scheduler = ReplayScheduler::new(SchedulerPolicy::default());
    let decision = scheduler.observe(
        agent_drift_sentinel::CheckpointCursor::from(&current),
        TriggerClass::CheckpointReady,
        current.flagged,
        Some(&warning_fingerprint(&current)),
    );

    let presentation = present_checkpoint_with_previous(
        &current,
        Some(&previous),
        TriggerClass::CheckpointReady,
        &decision,
        &WarningPolicy::default(),
    );

    assert_eq!(presentation.posture, Some(CheckpointPosture::Recovered));
    assert!(presentation
        .evidence_lines
        .iter()
        .any(|line| line
            .contains("historical truth-grounding gap: flagged score for session-public:1")));
}

#[test]
fn operator_surface_labels_scheduler_trigger_separately_from_analyzer_posture() {
    let recovered = checkpoint_with_state(
        "session-trigger-label",
        2,
        DriftClass::TruthGroundingGap,
        DriftState::Recovered,
        20,
        false,
        "continue on the current task frame",
        &["explicit analyzer recovery evidence"],
    );
    let historical_only = checkpoint_with_state(
        "session-trigger-label",
        3,
        DriftClass::TruthGroundingGap,
        DriftState::HistoricalOnly,
        20,
        false,
        "continue on the current task frame",
        &["explicit analyzer historical evidence"],
    );
    let mut scheduler = ReplayScheduler::new(SchedulerPolicy::default());

    let recovered_decision = scheduler.observe(
        agent_drift_sentinel::CheckpointCursor::from(&recovered),
        TriggerClass::RepeatedFailure,
        recovered.flagged,
        Some(&warning_fingerprint(&recovered)),
    );
    let recovered_presentation = present_checkpoint_with_previous(
        &recovered,
        None,
        TriggerClass::RepeatedFailure,
        &recovered_decision,
        &WarningPolicy::default(),
    );

    let historical_decision = scheduler.observe(
        agent_drift_sentinel::CheckpointCursor::from(&historical_only),
        TriggerClass::RepeatedFailure,
        historical_only.flagged,
        Some(&warning_fingerprint(&historical_only)),
    );
    let historical_presentation = present_checkpoint_with_previous(
        &historical_only,
        None,
        TriggerClass::RepeatedFailure,
        &historical_decision,
        &WarningPolicy::default(),
    );

    assert!(recovered_presentation
        .headline
        .contains("scheduler_repeated_failure_trigger"));
    assert_eq!(
        recovered_presentation.posture,
        Some(CheckpointPosture::Recovered)
    );
    assert!(historical_presentation
        .headline
        .contains("scheduler_repeated_failure_trigger"));
    assert_eq!(
        historical_presentation.posture,
        Some(CheckpointPosture::HistoricalOnly)
    );
}

#[allow(clippy::too_many_arguments)]
fn checkpoint_with_drift(
    session_id: &str,
    ordinal: usize,
    class: DriftClass,
    raw_score: u8,
    flagged: bool,
    expected_next_step: &str,
    evidence_reasons: &[&str],
) -> Checkpoint {
    let mut checkpoint =
        support::checkpoint(session_id, ordinal, raw_score, flagged, expected_next_step);
    checkpoint.flagged = flagged;
    checkpoint.drift_scores[0].class = class;
    checkpoint.drift_scores[0].raw_score = raw_score;
    checkpoint.drift_scores[0].flagged = flagged;
    checkpoint.drift_scores[0].evidence = evidence_reasons
        .iter()
        .map(|reason| EvidenceRef {
            row: checkpoint.boundary.start.clone(),
            reason: (*reason).to_string(),
        })
        .collect();
    checkpoint.diagnostics.evidence_item_count = checkpoint.drift_scores[0].evidence.len();
    checkpoint
}

#[allow(clippy::too_many_arguments)]
fn checkpoint_with_state(
    session_id: &str,
    ordinal: usize,
    class: DriftClass,
    state: DriftState,
    raw_score: u8,
    flagged: bool,
    expected_next_step: &str,
    evidence_reasons: &[&str],
) -> Checkpoint {
    checkpoint_with_schema_state(
        "v0.4",
        session_id,
        ordinal,
        class,
        state,
        raw_score,
        flagged,
        expected_next_step,
        evidence_reasons,
    )
}

#[allow(clippy::too_many_arguments)]
fn checkpoint_with_schema_state(
    schema_version: &str,
    session_id: &str,
    ordinal: usize,
    class: DriftClass,
    state: DriftState,
    raw_score: u8,
    flagged: bool,
    expected_next_step: &str,
    evidence_reasons: &[&str],
) -> Checkpoint {
    let mut checkpoint =
        support::checkpoint(session_id, ordinal, raw_score, flagged, expected_next_step);
    checkpoint.schema_version = schema_version.to_string();
    if schema_version == "v0.4" {
        checkpoint.turn_context = Some(sample_turn_context(ordinal));
    }
    checkpoint.flagged = flagged;
    checkpoint.drift_scores[0].class = class;
    checkpoint.drift_scores[0].state = state;
    checkpoint.drift_scores[0].raw_score = raw_score;
    checkpoint.drift_scores[0].flagged = flagged;
    checkpoint.drift_scores[0].evidence = evidence_reasons
        .iter()
        .map(|reason| EvidenceRef {
            row: checkpoint.boundary.start.clone(),
            reason: (*reason).to_string(),
        })
        .collect();
    checkpoint.diagnostics.evidence_item_count = checkpoint.drift_scores[0].evidence.len();
    checkpoint
}

fn sample_turn_context(turn_ordinal: usize) -> TurnContext {
    TurnContext {
        turn_id: Some(format!("turn-{turn_ordinal}")),
        turn_ordinal,
        rows_since_turn_start: 4,
        seconds_since_turn_start: Some(12),
        checkpoints_in_turn: 2,
        prompts_observed_in_session: turn_ordinal,
        execution_mode: TurnExecutionMode::Autonomous,
        activity_mix: TurnActivityMix {
            directive_row_count: 1,
            assistant_message_count: 1,
            tool_call_count: 1,
            read_like_command_count: 1,
            write_like_command_count: 1,
            verification_like_command_count: 0,
            tool_output_count: 1,
        },
    }
}

fn sample_session_archetype(
    checkpoint: &Checkpoint,
    label: SessionArchetypeLabel,
) -> SessionArchetype {
    SessionArchetype {
        label,
        confidence: Confidence::Medium,
        supporting_evidence: vec![
            EvidenceRef {
                row: checkpoint.boundary.start.clone(),
                reason:
                    "stable working set plus source edits supported concentrated implementation"
                        .to_string(),
            },
            EvidenceRef {
                row: checkpoint.boundary.start.clone(),
                reason: "source edit command strengthened implementation-like evidence"
                    .to_string(),
            },
            EvidenceRef {
                row: checkpoint.boundary.start.clone(),
                reason: "local verification against source scope also supported implementation follow-through"
                    .to_string(),
            },
        ],
        counter_evidence: vec![EvidenceRef {
            row: checkpoint.boundary.start.clone(),
            reason: "inspection-style command widened the visible search space".to_string(),
        }],
    }
}

fn sample_session_progress(
    checkpoint: &Checkpoint,
    status: ProgressStatus,
    dimension: ProgressDimension,
) -> SessionProgress {
    SessionProgress {
        status,
        dimension,
        confidence: Confidence::Medium,
        signals: Vec::new(),
        supporting_evidence: vec![EvidenceRef {
            row: checkpoint.boundary.start.clone(),
            reason: "progress evidence stayed focused on the active verification target"
                .to_string(),
        }],
        counter_evidence: vec![EvidenceRef {
            row: checkpoint.boundary.start.clone(),
            reason: "one branch of the session remained open while verification narrowed"
                .to_string(),
        }],
    }
}

fn assert_progress_line_after_archetype(rendered: &str) {
    let lines = rendered.lines().collect::<Vec<_>>();
    let archetype_index = lines
        .iter()
        .position(|line| line.starts_with("- Archetype: "))
        .expect("archetype line");
    let progress_index = lines
        .iter()
        .position(|line| line.starts_with("- Progress: "))
        .expect("progress line");

    assert_eq!(progress_index, archetype_index + 1);
}
