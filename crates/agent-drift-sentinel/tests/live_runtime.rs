#![allow(unused_crate_dependencies)]

mod support;

use agent_drift_sentinel::{
    LiveCheckpointEvent, LiveRuntime, LiveRuntimeError, SchedulerPolicy, TriggerClass,
    WarningDisposition, WarningPolicy,
};

fn current_schema_sample_checkpoints() -> Vec<agent_drift_analyzer::Checkpoint> {
    support::sample_checkpoints()
        .into_iter()
        .map(|mut checkpoint| {
            checkpoint.schema_version = "v0.2".to_string();
            checkpoint
        })
        .collect()
}

#[test]
fn live_runtime_reuses_scheduler_state_across_incremental_events() {
    let checkpoints = current_schema_sample_checkpoints();
    let mut runtime = LiveRuntime::new(SchedulerPolicy::default(), WarningPolicy::default());

    let first = runtime
        .observe(LiveCheckpointEvent::checkpoint_ready(
            1,
            checkpoints[0].clone(),
            Some("fixture".to_string()),
        ))
        .expect("first checkpoint ready");
    assert!(first.decision.evaluate);
    assert_eq!(first.presentation.trigger, TriggerClass::CheckpointReady);
    assert_eq!(first.snapshot.processed_events, 1);
    assert_eq!(
        first.snapshot.latest_checkpoint_id.as_deref(),
        Some("session-alpha:0001")
    );

    let second = runtime
        .observe(LiveCheckpointEvent::checkpoint_ready(
            2,
            checkpoints[1].clone(),
            Some("fixture".to_string()),
        ))
        .expect("second checkpoint ready");
    assert!(!second.decision.evaluate);
    assert_eq!(
        second.snapshot.latest_checkpoint_id.as_deref(),
        Some("session-alpha:0002")
    );
    assert!(matches!(
        &second.presentation.disposition,
        WarningDisposition::Silent { reason }
            if reason.contains("scheduler cooldown deferred replay evaluation")
    ));

    let third = runtime
        .observe(LiveCheckpointEvent::manual_review(
            3,
            second.event.cursor.clone(),
            Some("fixture".to_string()),
        ))
        .expect("manual review event");
    assert!(third.decision.evaluate);
    assert_eq!(third.presentation.trigger, TriggerClass::ManualReview);
    assert_eq!(third.snapshot.processed_events, 3);
    assert_eq!(
        third.snapshot.last_trigger,
        Some(TriggerClass::ManualReview)
    );
    assert!(matches!(
        &third.presentation.disposition,
        WarningDisposition::Silent { reason }
            if reason.contains("below visible score threshold")
    ));
}

#[test]
fn live_runtime_rejects_synthetic_cursor_mismatches() {
    let checkpoints = current_schema_sample_checkpoints();
    let mut runtime = LiveRuntime::new(SchedulerPolicy::default(), WarningPolicy::default());
    runtime
        .observe(LiveCheckpointEvent::checkpoint_ready(
            1,
            checkpoints[0].clone(),
            None,
        ))
        .expect("checkpoint ready");

    let error = runtime
        .observe(LiveCheckpointEvent::heartbeat(
            2,
            agent_drift_sentinel::CheckpointCursor {
                session_id: "session-alpha".to_string(),
                ordinal: 99,
            },
            None,
        ))
        .expect_err("heartbeat cursor mismatch");

    assert!(matches!(
        error,
        LiveRuntimeError::CursorMismatch {
            trigger,
            expected_ordinal: 1,
            actual_ordinal: 99,
            ..
        } if trigger == "heartbeat"
    ));
}
