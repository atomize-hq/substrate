#![allow(unused_crate_dependencies)]

use agent_drift_sentinel::{
    input::CheckpointCursor, scheduler::ReplayScheduler, DecisionReason, SchedulerPolicy,
    TriggerClass,
};

#[test]
fn scheduler_enforces_checkpoint_cooldown_for_normal_progress() {
    let mut scheduler = ReplayScheduler::new(SchedulerPolicy::default());

    let first = scheduler.observe(
        CheckpointCursor {
            session_id: "session-alpha".to_string(),
            ordinal: 1,
        },
        TriggerClass::CheckpointReady,
        false,
        Some("alpha"),
    );
    let second = scheduler.observe(
        CheckpointCursor {
            session_id: "session-alpha".to_string(),
            ordinal: 2,
        },
        TriggerClass::CheckpointReady,
        false,
        Some("beta"),
    );

    assert!(first.evaluate);
    assert!(!second.evaluate);
    assert_eq!(second.reason, DecisionReason::CooldownDeferred);
}

#[test]
fn scheduler_fast_paths_repeated_failures_even_inside_cooldown() {
    let mut scheduler = ReplayScheduler::new(SchedulerPolicy::default());

    scheduler.observe(
        CheckpointCursor {
            session_id: "session-alpha".to_string(),
            ordinal: 1,
        },
        TriggerClass::CheckpointReady,
        true,
        Some("duplicate"),
    );
    let fast_path = scheduler.observe(
        CheckpointCursor {
            session_id: "session-alpha".to_string(),
            ordinal: 2,
        },
        TriggerClass::RepeatedFailure,
        true,
        Some("duplicate"),
    );

    assert!(fast_path.evaluate);
    assert_eq!(fast_path.reason, DecisionReason::WarningDebounced);
}
