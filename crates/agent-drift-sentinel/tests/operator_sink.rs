#![allow(unused_crate_dependencies)]

mod support;

use agent_drift_sentinel::{
    build_operator_events, emit_operator_events, LiveCheckpointEvent, LiveRuntime, OperatorEvent,
    RecordingOperatorSink, SchedulerPolicy, TriggerClass, WarningPolicy,
};

#[test]
fn operator_sink_emits_visible_and_silent_checkpoint_events() {
    let checkpoints = support::sample_checkpoints();
    let mut runtime = LiveRuntime::new(SchedulerPolicy::default(), WarningPolicy::default());

    let visible = runtime
        .observe(LiveCheckpointEvent::checkpoint_ready(
            1,
            checkpoints[0].clone(),
            Some("fixture".to_string()),
        ))
        .expect("visible warning checkpoint");
    let visible_events = build_operator_events(&visible);
    assert!(matches!(
        visible_events.as_slice(),
        [OperatorEvent::VisibleWarning(event)]
            if event.presentation.headline.contains("session-alpha:0001")
                && event.diagnostics_summary.task_frame_transitioned
                && !event.diagnostics_summary.working_set_changed
                && event.diagnostics_summary.interval_verification_command_count == 1
                && event.diagnostics_summary.interval_command_count == 1
                && event.diagnostics_summary.verification_density_basis_points == Some(10_000)
                && event.diagnostics_summary.evidence_item_count == 2
    ));

    let silent = runtime
        .observe(LiveCheckpointEvent::checkpoint_ready(
            2,
            checkpoints[1].clone(),
            Some("fixture".to_string()),
        ))
        .expect("silent checkpoint");
    let silent_events = build_operator_events(&silent);
    assert!(matches!(
        silent_events.as_slice(),
        [OperatorEvent::SilentCheckpoint(event)]
            if event.reason.contains("scheduler cooldown deferred replay evaluation")
                && event.trigger == TriggerClass::CheckpointReady
                && !event.diagnostics_summary.task_frame_transitioned
                && event.diagnostics_summary.verification_density_basis_points == Some(10_000)
    ));
}

#[test]
fn operator_sink_emits_heartbeat_and_manual_review_status_events() {
    let checkpoints = support::sample_checkpoints();
    let mut runtime = LiveRuntime::new(SchedulerPolicy::default(), WarningPolicy::default());
    let mut sink = RecordingOperatorSink::default();

    runtime
        .observe(LiveCheckpointEvent::checkpoint_ready(
            1,
            checkpoints[0].clone(),
            Some("fixture".to_string()),
        ))
        .expect("first checkpoint");
    let second = runtime
        .observe(LiveCheckpointEvent::checkpoint_ready(
            2,
            checkpoints[1].clone(),
            Some("fixture".to_string()),
        ))
        .expect("second checkpoint");
    emit_operator_events(&mut sink, &second).expect("emit silent checkpoint event");

    let heartbeat = runtime
        .observe(LiveCheckpointEvent::heartbeat(
            3,
            second.event.cursor.clone(),
            Some("fixture".to_string()),
        ))
        .expect("heartbeat event");
    let heartbeat_events =
        emit_operator_events(&mut sink, &heartbeat).expect("emit heartbeat operator events");
    assert!(matches!(
        heartbeat_events.as_slice(),
        [OperatorEvent::Heartbeat(event)]
            if event.evaluated
                && event.message.contains("heartbeat evaluated")
                && !event.diagnostics_summary.task_frame_transitioned
                && event.diagnostics_summary.interval_command_count == 1
    ));

    let manual_review = runtime
        .observe(LiveCheckpointEvent::manual_review(
            4,
            heartbeat.event.cursor.clone(),
            Some("fixture".to_string()),
        ))
        .expect("manual review event");
    let manual_review_events =
        emit_operator_events(&mut sink, &manual_review).expect("emit manual review status");
    assert!(matches!(
        manual_review_events.as_slice(),
        [OperatorEvent::Status(event)]
            if event.trigger == TriggerClass::ManualReview
                && event.message.contains("without a visible warning")
                && !event.diagnostics_summary.task_frame_transitioned
                && event.diagnostics_summary.evidence_item_count == 2
    ));

    assert_eq!(sink.events().len(), 3);
}
