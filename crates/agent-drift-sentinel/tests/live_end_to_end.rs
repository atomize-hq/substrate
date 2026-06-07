#![allow(unused_crate_dependencies)]

mod support;

use agent_drift_analyzer::{
    Checkpoint, DriftClass, DriftState, EvidenceRef, TurnActivityMix, TurnContext,
    TurnExecutionMode,
};
use camino::Utf8PathBuf;

use agent_drift_sentinel::{
    emit_operator_events, execute, operator_surface::CheckpointPosture, AdjudicationConfig,
    FixtureLiveCheckpointSource, LiveCheckpointEvent, LiveRuntime, OperatorEvent,
    RecordingOperatorSink, SchedulerPolicy, SentinelMode, SentinelRequest, WarningPolicy,
};

#[test]
fn live_end_to_end_fixture_stream_produces_stable_operator_events() {
    let path = fixture_path("append_only_stream.jsonl");
    let mut source =
        FixtureLiveCheckpointSource::from_path(&path).expect("load append-only live fixture");
    let mut runtime = LiveRuntime::new(SchedulerPolicy::default(), WarningPolicy::default());
    let mut sink = RecordingOperatorSink::default();

    let observations = runtime
        .drain(&mut source)
        .expect("drain fixture-backed live runtime");
    assert_eq!(observations.len(), 4);

    for observation in &observations {
        emit_operator_events(&mut sink, observation).expect("emit operator events");
    }

    let events = sink.into_events();
    assert_eq!(events.len(), 4);

    let visible = match &events[0] {
        OperatorEvent::VisibleWarning(event) => event,
        other => panic!("expected visible warning event, got {other:?}"),
    };
    assert_eq!(visible.cursor.session_id, "session-alpha");
    assert_eq!(visible.cursor.ordinal, 1);
    assert!(visible.presentation.headline.contains("session-alpha:0001"));
    assert!(visible.presentation.render_console_block(None).contains(
        "Diagnostics: task_frame_transitioned=true, working_set_changed=false, verification=1/1 (100.00%), evidence_items=2"
    ));
    assert!(visible
        .presentation
        .render_console_block(None)
        .contains("align plan to repo truth"));

    let heartbeat = match &events[1] {
        OperatorEvent::Heartbeat(event) => event,
        other => panic!("expected heartbeat event, got {other:?}"),
    };
    assert!(!heartbeat.evaluated);
    assert!(heartbeat.message.contains("warning_debounced"));
    assert_eq!(heartbeat.cursor.ordinal, 1);
    assert_eq!(heartbeat.diagnostics_summary, visible.diagnostics_summary);

    let silent = match &events[2] {
        OperatorEvent::SilentCheckpoint(event) => event,
        other => panic!("expected silent checkpoint event, got {other:?}"),
    };
    assert_eq!(silent.checkpoint_id, "session-alpha:0002");
    assert!(silent
        .reason
        .contains("checkpoint recorded without a visible warning"));
    assert_eq!(silent.diagnostics_summary.interval_command_count, 1);
    assert_eq!(
        silent
            .diagnostics_summary
            .interval_verification_command_count,
        1
    );
    assert_eq!(
        silent.diagnostics_summary.verification_density_basis_points,
        Some(10_000)
    );
    assert_eq!(silent.diagnostics_summary.evidence_item_count, 1);

    let status = match &events[3] {
        OperatorEvent::Status(event) => event,
        other => panic!("expected status event, got {other:?}"),
    };
    assert_eq!(status.cursor.ordinal, 2);
    assert!(status.message.contains("manual_review"));
    assert!(status.message.contains("without a visible warning"));
    assert_eq!(status.diagnostics_summary, silent.diagnostics_summary);

    let final_snapshot = &observations[3].snapshot;
    assert_eq!(final_snapshot.processed_events, 4);
    assert_eq!(
        final_snapshot.latest_checkpoint_id.as_deref(),
        Some("session-alpha:0002")
    );
    assert_eq!(final_snapshot.latest_cursor.as_ref(), Some(&status.cursor));
}

#[test]
fn live_end_to_end_replay_and_live_surfaces_share_the_same_diagnostics_summary_for_matching_checkpoints(
) {
    let checkpoints = support::sample_checkpoints();
    let replay_fixture = support::ReplayFixture::from_checkpoints(
        checkpoints[..2].to_vec(),
        support::sample_summary(),
    );
    let replay = execute(&SentinelRequest {
        checkpoint_dir: replay_fixture.checkpoint_dir.clone(),
        mode: SentinelMode::Replay,
        cursor: None,
        scheduler_policy: SchedulerPolicy::default(),
        warning_policy: WarningPolicy::default(),
        adjudication: AdjudicationConfig::default(),
    })
    .expect("run replay");

    let mut runtime = LiveRuntime::new(SchedulerPolicy::default(), WarningPolicy::default());
    let live_visible = runtime
        .observe(LiveCheckpointEvent::checkpoint_ready(
            1,
            checkpoints[0].clone(),
            Some("fixture".to_string()),
        ))
        .expect("live visible checkpoint");
    let live_silent = runtime
        .observe(LiveCheckpointEvent::checkpoint_ready(
            2,
            checkpoints[1].clone(),
            Some("fixture".to_string()),
        ))
        .expect("live silent checkpoint");

    let replay_visible = &replay.report.visible_warnings[0];
    let replay_silent = &replay.report.silent_checkpoints[0];
    let live_visible_event = match build_single_event(&live_visible) {
        OperatorEvent::VisibleWarning(event) => event,
        other => panic!("expected visible warning event, got {other:?}"),
    };
    let live_silent_event = match build_single_event(&live_silent) {
        OperatorEvent::SilentCheckpoint(event) => event,
        other => panic!("expected silent checkpoint event, got {other:?}"),
    };

    assert_eq!(
        replay_visible.diagnostics_summary,
        live_visible_event.diagnostics_summary
    );
    assert_eq!(
        replay_silent.diagnostics_summary,
        live_silent_event.diagnostics_summary
    );
}

#[test]
fn live_end_to_end_replay_and_live_share_checkpoint_ready_headlines_for_matching_checkpoints() {
    let checkpoints = support::sample_checkpoints();
    let replay_fixture = support::ReplayFixture::from_checkpoints(
        checkpoints[..2].to_vec(),
        support::sample_summary(),
    );
    let replay = execute(&SentinelRequest {
        checkpoint_dir: replay_fixture.checkpoint_dir.clone(),
        mode: SentinelMode::Replay,
        cursor: None,
        scheduler_policy: SchedulerPolicy::default(),
        warning_policy: WarningPolicy::default(),
        adjudication: AdjudicationConfig::default(),
    })
    .expect("run replay");

    let mut runtime = LiveRuntime::new(SchedulerPolicy::default(), WarningPolicy::default());
    let live_visible = runtime
        .observe(LiveCheckpointEvent::checkpoint_ready(
            1,
            checkpoints[0].clone(),
            Some("fixture".to_string()),
        ))
        .expect("live visible checkpoint");
    let live_silent = runtime
        .observe(LiveCheckpointEvent::checkpoint_ready(
            2,
            checkpoints[1].clone(),
            Some("fixture".to_string()),
        ))
        .expect("live silent checkpoint");

    let replay_visible = &replay.report.visible_warnings[0];
    let replay_silent = &replay.report.silent_checkpoints[0];

    assert_eq!(replay_visible.headline, live_visible.presentation.headline);
    assert_eq!(replay_silent.headline, live_silent.presentation.headline);
    assert!(replay_visible.headline.contains("checkpoint_ready"));
    assert!(replay_silent.headline.contains("checkpoint_ready"));
    assert!(!replay_visible
        .headline
        .contains("scheduler_repeated_failure_trigger"));
    assert!(!replay_silent
        .headline
        .contains("scheduler_repeated_failure_trigger"));
}

#[test]
fn live_end_to_end_replay_ordinary_flagged_checkpoints_do_not_take_repeated_failure_fast_path() {
    let scheduler_policy = SchedulerPolicy {
        checkpoint_cooldown: 3,
        repeated_failure_threshold: 2,
        ..SchedulerPolicy::default()
    };
    let checkpoints = vec![
        checkpoint_with_state(
            "session-policy-split",
            1,
            DriftClass::TruthGroundingGap,
            DriftState::Active,
            82,
            true,
            "align plan to repo truth",
            &["flagged score for session-policy-split:1"],
        ),
        checkpoint_with_state(
            "session-policy-split",
            2,
            DriftClass::TruthGroundingGap,
            DriftState::Active,
            86,
            true,
            "confirm fresh repo evidence before continuing",
            &["flagged score for session-policy-split:2"],
        ),
    ];
    let replay_fixture = support::ReplayFixture::from_checkpoints(
        checkpoints.clone(),
        support::sample_summary(),
    );
    let replay = execute(&SentinelRequest {
        checkpoint_dir: replay_fixture.checkpoint_dir.clone(),
        mode: SentinelMode::Replay,
        cursor: None,
        scheduler_policy,
        warning_policy: WarningPolicy::default(),
        adjudication: AdjudicationConfig::default(),
    })
    .expect("run replay");

    assert_eq!(replay.report.visible_warnings.len(), 1);
    assert_eq!(replay.report.silent_checkpoints.len(), 1);

    let replay_silent = replay
        .report
        .silent_checkpoints
        .iter()
        .find(|checkpoint| checkpoint.checkpoint.checkpoint_id == "session-policy-split:0002")
        .expect("replay second checkpoint");

    let mut runtime = LiveRuntime::new(scheduler_policy, WarningPolicy::default());
    runtime
        .observe(LiveCheckpointEvent::checkpoint_ready(
            1,
            checkpoints[0].clone(),
            Some("fixture".to_string()),
        ))
        .expect("live first checkpoint");
    let live_second = runtime
        .observe(LiveCheckpointEvent::checkpoint_ready(
            2,
            checkpoints[1].clone(),
            Some("fixture".to_string()),
        ))
        .expect("live second checkpoint");
    let live_second_event = match build_single_event(&live_second) {
        OperatorEvent::SilentCheckpoint(event) => event,
        other => panic!("expected silent checkpoint event, got {other:?}"),
    };

    assert_eq!(replay_silent.headline, live_second.presentation.headline);
    assert_eq!(replay_silent.posture, live_second.presentation.posture);
    assert!(replay_silent.headline.contains("checkpoint_ready"));
    assert!(!replay_silent
        .headline
        .contains("scheduler_repeated_failure_trigger"));
    assert!(live_second_event
        .reason
        .contains("scheduler cooldown deferred replay evaluation"));

    let live_fast_path = runtime
        .observe(LiveCheckpointEvent::repeated_failure(
            3,
            live_second.event.cursor.clone(),
            Some("fixture".to_string()),
        ))
        .expect("live repeated-failure event");
    let live_fast_path_event = match build_single_event(&live_fast_path) {
        OperatorEvent::Status(event) => event,
        other => panic!("expected status event, got {other:?}"),
    };

    assert!(live_fast_path.decision.evaluate);
    assert!(live_fast_path_event
        .message
        .contains("scheduler_repeated_failure_trigger"));
    assert_ne!(live_fast_path.presentation.headline, replay_silent.headline);
}

#[test]
fn live_end_to_end_replay_and_live_surfaces_share_turn_context_rendering_for_v0_4_checkpoints() {
    let mut checkpoints = support::sample_checkpoints();
    for (index, checkpoint) in checkpoints.iter_mut().take(2).enumerate() {
        checkpoint.schema_version = "v0.4".to_string();
        checkpoint.turn_context = Some(sample_turn_context(index + 1));
    }
    let replay_fixture = support::ReplayFixture::from_checkpoints(
        checkpoints[..2].to_vec(),
        support::sample_summary(),
    );
    let replay = execute(&SentinelRequest {
        checkpoint_dir: replay_fixture.checkpoint_dir.clone(),
        mode: SentinelMode::Replay,
        cursor: None,
        scheduler_policy: SchedulerPolicy::default(),
        warning_policy: WarningPolicy::default(),
        adjudication: AdjudicationConfig::default(),
    })
    .expect("run replay");

    let mut runtime = LiveRuntime::new(SchedulerPolicy::default(), WarningPolicy::default());
    let live_visible = runtime
        .observe(LiveCheckpointEvent::checkpoint_ready(
            1,
            checkpoints[0].clone(),
            Some("fixture".to_string()),
        ))
        .expect("live visible checkpoint");
    let live_silent = runtime
        .observe(LiveCheckpointEvent::checkpoint_ready(
            2,
            checkpoints[1].clone(),
            Some("fixture".to_string()),
        ))
        .expect("live silent checkpoint");

    let replay_visible = &replay.report.visible_warnings[0];
    let replay_silent = &replay.report.silent_checkpoints[0];
    let replay_visible_render = replay_visible.render_console_block(None);
    let live_visible_render = live_visible.presentation.render_console_block(None);
    let replay_silent_render = replay_silent.render_console_block(None);
    let live_silent_render = live_silent.presentation.render_console_block(None);

    assert_eq!(
        extract_turn_context_line(&replay_visible_render),
        extract_turn_context_line(&live_visible_render)
    );
    assert_eq!(
        extract_turn_context_line(&replay_silent_render),
        extract_turn_context_line(&live_silent_render)
    );
    assert!(replay_visible_render.contains(
        "- Turn context: turn-1 (#1) rows=5 elapsed=21s checkpoints=2 session-prompts=1 mode=autonomous activity[dir=1 asst=1 tool=1 read=1 write=1 verify=1 out=1]"
    ));
    assert!(replay_silent_render.contains(
        "- Turn context: turn-2 (#2) rows=5 elapsed=21s checkpoints=2 session-prompts=2 mode=autonomous activity[dir=1 asst=1 tool=1 read=1 write=1 verify=1 out=1]"
    ));
}

#[test]
fn live_end_to_end_replay_and_live_surfaces_share_posture_for_transition_sequences() {
    let checkpoints = vec![
        checkpoint_with_state(
            "session-posture",
            1,
            DriftClass::TruthGroundingGap,
            DriftState::Active,
            82,
            true,
            "align plan to repo truth",
            &["flagged score for session-posture:1"],
        ),
        checkpoint_with_state(
            "session-posture",
            2,
            DriftClass::TruthGroundingGap,
            DriftState::Recovered,
            20,
            false,
            "continue on the current task frame",
            &["explicit analyzer recovery evidence"],
        ),
        checkpoint_with_state(
            "session-posture",
            3,
            DriftClass::TruthGroundingGap,
            DriftState::HistoricalOnly,
            20,
            false,
            "continue on the current task frame",
            &["explicit analyzer historical evidence"],
        ),
    ];
    let replay_fixture =
        support::ReplayFixture::from_checkpoints(checkpoints.clone(), support::sample_summary());
    let replay = execute(&SentinelRequest {
        checkpoint_dir: replay_fixture.checkpoint_dir.clone(),
        mode: SentinelMode::Replay,
        cursor: None,
        scheduler_policy: SchedulerPolicy::default(),
        warning_policy: WarningPolicy::default(),
        adjudication: AdjudicationConfig::default(),
    })
    .expect("run replay");

    let mut runtime = LiveRuntime::new(SchedulerPolicy::default(), WarningPolicy::default());
    let live_active = runtime
        .observe(LiveCheckpointEvent::checkpoint_ready(
            1,
            checkpoints[0].clone(),
            Some("fixture".to_string()),
        ))
        .expect("live active checkpoint");
    let live_recovered = runtime
        .observe(LiveCheckpointEvent::checkpoint_ready(
            2,
            checkpoints[1].clone(),
            Some("fixture".to_string()),
        ))
        .expect("live recovered checkpoint");
    let live_historical_only = runtime
        .observe(LiveCheckpointEvent::checkpoint_ready(
            3,
            checkpoints[2].clone(),
            Some("fixture".to_string()),
        ))
        .expect("live historical-only checkpoint");

    let replay_active = &replay.report.visible_warnings[0];
    let replay_recovered = replay
        .report
        .silent_checkpoints
        .iter()
        .find(|checkpoint| checkpoint.checkpoint.checkpoint_id == "session-posture:0002")
        .expect("recovered replay checkpoint");
    let replay_historical_only = replay
        .report
        .silent_checkpoints
        .iter()
        .find(|checkpoint| checkpoint.checkpoint.checkpoint_id == "session-posture:0003")
        .expect("historical-only replay checkpoint");

    let live_active_event = match build_single_event(&live_active) {
        OperatorEvent::VisibleWarning(event) => event,
        other => panic!("expected visible warning event, got {other:?}"),
    };
    let live_recovered_event = match build_single_event(&live_recovered) {
        OperatorEvent::SilentCheckpoint(event) => event,
        other => panic!("expected silent checkpoint event, got {other:?}"),
    };
    let live_historical_only_event = match build_single_event(&live_historical_only) {
        OperatorEvent::SilentCheckpoint(event) => event,
        other => panic!("expected silent checkpoint event, got {other:?}"),
    };

    assert_eq!(replay_active.posture, Some(CheckpointPosture::Active));
    assert_eq!(replay_recovered.posture, Some(CheckpointPosture::Recovered));
    assert_eq!(
        replay_historical_only.posture,
        Some(CheckpointPosture::HistoricalOnly)
    );
    assert_eq!(live_active_event.posture, replay_active.posture);
    assert_eq!(live_recovered_event.posture, replay_recovered.posture);
    assert_eq!(
        live_historical_only_event.posture,
        replay_historical_only.posture
    );
    assert!(live_recovered
        .presentation
        .render_console_block(None)
        .contains("- Posture: recovered"));
    assert!(live_historical_only
        .presentation
        .render_console_block(None)
        .contains("- Posture: historical-only"));
}

#[test]
fn live_end_to_end_replay_and_live_keep_posture_session_local_at_session_boundaries() {
    let checkpoints = vec![
        checkpoint_with_state(
            "session-a",
            1,
            DriftClass::TruthGroundingGap,
            DriftState::Active,
            82,
            true,
            "align plan to repo truth",
            &["flagged score for session-a:1"],
        ),
        checkpoint_with_state(
            "session-b",
            1,
            DriftClass::TruthGroundingGap,
            DriftState::HistoricalOnly,
            20,
            false,
            "continue on the current task frame",
            &["explicit analyzer historical evidence for session-b"],
        ),
    ];
    let replay_fixture =
        support::ReplayFixture::from_checkpoints(checkpoints.clone(), support::sample_summary());
    let replay = execute(&SentinelRequest {
        checkpoint_dir: replay_fixture.checkpoint_dir.clone(),
        mode: SentinelMode::Replay,
        cursor: None,
        scheduler_policy: SchedulerPolicy::default(),
        warning_policy: WarningPolicy::default(),
        adjudication: AdjudicationConfig::default(),
    })
    .expect("run replay");

    let mut runtime = LiveRuntime::new(SchedulerPolicy::default(), WarningPolicy::default());
    runtime
        .observe(LiveCheckpointEvent::checkpoint_ready(
            1,
            checkpoints[0].clone(),
            Some("fixture".to_string()),
        ))
        .expect("live active checkpoint");
    let live_historical_only = runtime
        .observe(LiveCheckpointEvent::checkpoint_ready(
            2,
            checkpoints[1].clone(),
            Some("fixture".to_string()),
        ))
        .expect("live historical-only checkpoint");

    let replay_historical_only = replay
        .report
        .silent_checkpoints
        .iter()
        .find(|checkpoint| checkpoint.checkpoint.checkpoint_id == "session-b:0001")
        .expect("historical-only replay checkpoint");
    let live_historical_only_event = match build_single_event(&live_historical_only) {
        OperatorEvent::SilentCheckpoint(event) => event,
        other => panic!("expected silent checkpoint event, got {other:?}"),
    };

    assert_eq!(
        replay_historical_only.posture,
        Some(CheckpointPosture::HistoricalOnly)
    );
    assert_eq!(
        live_historical_only.presentation.posture,
        replay_historical_only.posture
    );
    assert_eq!(
        live_historical_only_event.posture,
        replay_historical_only.posture
    );
}

#[test]
fn live_end_to_end_keeps_scheduler_trigger_labels_distinct_from_analyzer_posture() {
    let checkpoints = vec![
        checkpoint_with_state(
            "session-trigger-label",
            1,
            DriftClass::TruthGroundingGap,
            DriftState::Active,
            82,
            true,
            "align plan to repo truth",
            &["flagged score for session-trigger-label:1"],
        ),
        checkpoint_with_state(
            "session-trigger-label",
            2,
            DriftClass::TruthGroundingGap,
            DriftState::Recovered,
            20,
            false,
            "continue on the current task frame",
            &["explicit analyzer recovery evidence"],
        ),
        checkpoint_with_state(
            "session-trigger-label",
            3,
            DriftClass::TruthGroundingGap,
            DriftState::HistoricalOnly,
            20,
            false,
            "continue on the current task frame",
            &["explicit analyzer historical evidence"],
        ),
    ];
    let replay_fixture = support::ReplayFixture::from_checkpoints(
        vec![checkpoints[0].clone()],
        support::sample_summary(),
    );
    let replay = execute(&SentinelRequest {
        checkpoint_dir: replay_fixture.checkpoint_dir.clone(),
        mode: SentinelMode::Replay,
        cursor: None,
        scheduler_policy: SchedulerPolicy::default(),
        warning_policy: WarningPolicy::default(),
        adjudication: AdjudicationConfig::default(),
    })
    .expect("run replay");

    let mut runtime = LiveRuntime::new(SchedulerPolicy::default(), WarningPolicy::default());
    runtime
        .observe(LiveCheckpointEvent::checkpoint_ready(
            1,
            checkpoints[0].clone(),
            Some("fixture".to_string()),
        ))
        .expect("live active checkpoint");
    let live_recovered_checkpoint = runtime
        .observe(LiveCheckpointEvent::checkpoint_ready(
            2,
            checkpoints[1].clone(),
            Some("fixture".to_string()),
        ))
        .expect("live recovered checkpoint");
    let live_recovered_fast_path = runtime
        .observe(LiveCheckpointEvent::repeated_failure(
            3,
            live_recovered_checkpoint.event.cursor.clone(),
            Some("fixture".to_string()),
        ))
        .expect("live recovered repeated-failure trigger");
    let live_historical_checkpoint = runtime
        .observe(LiveCheckpointEvent::checkpoint_ready(
            4,
            checkpoints[2].clone(),
            Some("fixture".to_string()),
        ))
        .expect("live historical-only checkpoint");
    let live_historical_fast_path = runtime
        .observe(LiveCheckpointEvent::repeated_failure(
            5,
            live_historical_checkpoint.event.cursor.clone(),
            Some("fixture".to_string()),
        ))
        .expect("live historical-only repeated-failure trigger");

    let replay_visible = &replay.report.visible_warnings[0];
    let live_recovered_status = match build_single_event(&live_recovered_fast_path) {
        OperatorEvent::Status(event) => event,
        other => panic!("expected status event, got {other:?}"),
    };
    let live_historical_status = match build_single_event(&live_historical_fast_path) {
        OperatorEvent::Status(event) => event,
        other => panic!("expected status event, got {other:?}"),
    };

    assert!(replay_visible.headline.contains("checkpoint_ready"));
    assert!(!replay_visible
        .headline
        .contains("scheduler_repeated_failure_trigger"));
    assert_eq!(
        live_recovered_fast_path.presentation.posture,
        Some(CheckpointPosture::Recovered)
    );
    assert!(live_recovered_status
        .message
        .contains("scheduler_repeated_failure_trigger"));
    assert_eq!(
        live_historical_fast_path.presentation.posture,
        Some(CheckpointPosture::HistoricalOnly)
    );
    assert!(live_historical_status
        .message
        .contains("scheduler_repeated_failure_trigger"));
}

fn fixture_path(name: &str) -> Utf8PathBuf {
    Utf8PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("live")
        .join(name)
}

fn build_single_event(observation: &agent_drift_sentinel::LiveObservation) -> OperatorEvent {
    let events = emit_operator_events(&mut RecordingOperatorSink::default(), observation)
        .expect("emit operator events");
    assert_eq!(events.len(), 1);
    events.into_iter().next().expect("single event")
}

fn extract_turn_context_line(rendered: &str) -> Option<&str> {
    rendered
        .lines()
        .find(|line| line.starts_with("- Turn context: "))
}

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
    let mut checkpoint =
        support::checkpoint(session_id, ordinal, raw_score, flagged, expected_next_step);
    checkpoint.schema_version = "v0.3".to_string();
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
        rows_since_turn_start: 5,
        seconds_since_turn_start: Some(21),
        checkpoints_in_turn: 2,
        prompts_observed_in_session: turn_ordinal,
        execution_mode: TurnExecutionMode::Autonomous,
        activity_mix: TurnActivityMix {
            directive_row_count: 1,
            assistant_message_count: 1,
            tool_call_count: 1,
            read_like_command_count: 1,
            write_like_command_count: 1,
            verification_like_command_count: 1,
            tool_output_count: 1,
        },
    }
}
