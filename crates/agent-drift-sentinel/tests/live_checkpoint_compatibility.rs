#![allow(unused_crate_dependencies)]

mod support;

use agent_drift_sentinel::{
    operator_surface::present_checkpoint, scheduler::ReplayScheduler,
    verify_live_checkpoint_compatibility, DecisionReason, LiveInputError, SchedulerPolicy,
    TriggerClass, WarningDisposition, WarningPolicy,
};

fn current_schema_checkpoint(
    session_id: &str,
    ordinal: usize,
    raw_score: u8,
    flagged: bool,
    expected_next_step: &str,
) -> agent_drift_analyzer::Checkpoint {
    let mut checkpoint =
        support::checkpoint(session_id, ordinal, raw_score, flagged, expected_next_step);
    checkpoint.schema_version = "v0.2".to_string();
    checkpoint
}

#[test]
fn live_checkpoint_compatibility_proves_existing_analyzer_checkpoint_is_sufficient() {
    let checkpoint = current_schema_checkpoint(
        "session-alpha",
        1,
        88,
        true,
        "re-read the implementation plan",
    );
    let compatibility =
        verify_live_checkpoint_compatibility(&checkpoint).expect("checkpoint is live-compatible");
    let mut scheduler = ReplayScheduler::new(SchedulerPolicy::default());
    let decision = scheduler.observe(
        compatibility.cursor.clone(),
        TriggerClass::CheckpointReady,
        compatibility.flagged,
        Some(&compatibility.warning_fingerprint),
    );
    let presentation = present_checkpoint(
        &checkpoint,
        TriggerClass::CheckpointReady,
        &decision,
        &WarningPolicy::default(),
    );

    assert!(decision.evaluate);
    assert_eq!(decision.reason, DecisionReason::InitialCheckpoint);
    assert_eq!(compatibility.max_flagged_score, Some(88));
    assert!(matches!(
        presentation.disposition,
        WarningDisposition::Visible
    ));
    assert!(presentation.headline.contains("session-alpha:0001"));
    assert_eq!(
        presentation.expected_next_step,
        "re-read the implementation plan"
    );
}

#[test]
fn live_checkpoint_compatibility_surfaces_analyzer_contract_gaps_explicitly() {
    let mut checkpoint = current_schema_checkpoint(
        "session-alpha",
        1,
        88,
        true,
        "re-read the implementation plan",
    );
    checkpoint.task_frame.objective.clear();

    let error = verify_live_checkpoint_compatibility(&checkpoint)
        .expect_err("missing objective must be reported as an analyzer contract gap");

    assert!(matches!(
        error,
        LiveInputError::CompatibilityGap {
            field: "task_frame.objective",
            ..
        }
    ));
}
