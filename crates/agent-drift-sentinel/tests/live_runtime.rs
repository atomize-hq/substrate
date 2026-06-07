#![allow(unused_crate_dependencies)]

mod support;

use agent_drift_analyzer::{DriftClass, DriftState, EvidenceRef};
use agent_drift_sentinel::{
    operator_surface::CheckpointPosture, LiveCheckpointEvent, LiveRuntime, LiveRuntimeError,
    SchedulerPolicy, TriggerClass, WarningDisposition, WarningPolicy,
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

#[test]
fn live_runtime_preserves_posture_on_repeated_failure_fast_paths() {
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
    let mut runtime = LiveRuntime::new(SchedulerPolicy::default(), WarningPolicy::default());

    runtime
        .observe(LiveCheckpointEvent::checkpoint_ready(
            1,
            checkpoints[0].clone(),
            Some("fixture".to_string()),
        ))
        .expect("active checkpoint");
    let recovered_checkpoint = runtime
        .observe(LiveCheckpointEvent::checkpoint_ready(
            2,
            checkpoints[1].clone(),
            Some("fixture".to_string()),
        ))
        .expect("recovered checkpoint");
    let recovered_fast_path = runtime
        .observe(LiveCheckpointEvent::repeated_failure(
            3,
            recovered_checkpoint.event.cursor.clone(),
            Some("fixture".to_string()),
        ))
        .expect("recovered repeated-failure fast path");
    let historical_checkpoint = runtime
        .observe(LiveCheckpointEvent::checkpoint_ready(
            4,
            checkpoints[2].clone(),
            Some("fixture".to_string()),
        ))
        .expect("historical-only checkpoint");
    let historical_fast_path = runtime
        .observe(LiveCheckpointEvent::repeated_failure(
            5,
            historical_checkpoint.event.cursor.clone(),
            Some("fixture".to_string()),
        ))
        .expect("historical-only repeated-failure fast path");

    assert_eq!(
        recovered_fast_path.presentation.trigger,
        TriggerClass::RepeatedFailure
    );
    assert_eq!(
        recovered_fast_path.presentation.posture,
        Some(CheckpointPosture::Recovered)
    );
    assert!(recovered_fast_path
        .presentation
        .headline
        .contains("scheduler_repeated_failure_trigger"));
    assert_eq!(
        historical_fast_path.presentation.trigger,
        TriggerClass::RepeatedFailure
    );
    assert_eq!(
        historical_fast_path.presentation.posture,
        Some(CheckpointPosture::HistoricalOnly)
    );
    assert!(historical_fast_path
        .presentation
        .headline
        .contains("scheduler_repeated_failure_trigger"));
    assert_eq!(
        historical_fast_path.snapshot.last_trigger,
        Some(TriggerClass::RepeatedFailure)
    );
}

#[test]
fn live_runtime_keeps_checkpoint_ready_and_repeated_failure_headlines_distinct() {
    let checkpoints = vec![
        checkpoint_with_state(
            "session-trigger-split",
            1,
            DriftClass::TruthGroundingGap,
            DriftState::Active,
            82,
            true,
            "align plan to repo truth",
            &["flagged score for session-trigger-split:1"],
        ),
        checkpoint_with_state(
            "session-trigger-split",
            2,
            DriftClass::TruthGroundingGap,
            DriftState::Recovered,
            20,
            false,
            "continue on the current task frame",
            &["explicit analyzer recovery evidence"],
        ),
    ];
    let mut runtime = LiveRuntime::new(SchedulerPolicy::default(), WarningPolicy::default());

    runtime
        .observe(LiveCheckpointEvent::checkpoint_ready(
            1,
            checkpoints[0].clone(),
            Some("fixture".to_string()),
        ))
        .expect("active checkpoint");
    let checkpoint_ready = runtime
        .observe(LiveCheckpointEvent::checkpoint_ready(
            2,
            checkpoints[1].clone(),
            Some("fixture".to_string()),
        ))
        .expect("recovered checkpoint");
    let repeated_failure = runtime
        .observe(LiveCheckpointEvent::repeated_failure(
            3,
            checkpoint_ready.event.cursor.clone(),
            Some("fixture".to_string()),
        ))
        .expect("repeated-failure fast path");

    assert!(checkpoint_ready
        .presentation
        .headline
        .contains("checkpoint_ready"));
    assert!(!checkpoint_ready
        .presentation
        .headline
        .contains("scheduler_repeated_failure_trigger"));
    assert!(repeated_failure
        .presentation
        .headline
        .contains("scheduler_repeated_failure_trigger"));
    assert_ne!(
        checkpoint_ready.presentation.headline,
        repeated_failure.presentation.headline
    );
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
) -> agent_drift_analyzer::Checkpoint {
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
    checkpoint
}
