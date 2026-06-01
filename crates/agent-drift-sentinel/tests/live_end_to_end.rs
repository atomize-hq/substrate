#![allow(unused_crate_dependencies)]

mod support;

use camino::Utf8PathBuf;

use agent_drift_sentinel::{
    emit_operator_events, execute, AdjudicationConfig, FixtureLiveCheckpointSource,
    LiveCheckpointEvent, LiveRuntime, OperatorEvent, RecordingOperatorSink, SchedulerPolicy,
    SentinelMode, SentinelRequest, WarningPolicy,
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
