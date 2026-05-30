#![allow(unused_crate_dependencies)]

mod support;

use agent_drift_sentinel::{
    validate_live_event_sequence, CheckpointCursor, LiveCheckpointEvent, LiveInputError,
};

#[test]
fn live_input_accepts_append_only_checkpoint_and_trigger_sequence() {
    let first = support::checkpoint("session-alpha", 1, 72, true, "review the latest objective");
    let first_cursor = CheckpointCursor::from(&first);
    let second = support::checkpoint(
        "session-alpha",
        2,
        0,
        false,
        "continue with the existing task frame",
    );
    let second_cursor = CheckpointCursor::from(&second);
    let events = vec![
        LiveCheckpointEvent::checkpoint_ready(1, first, Some("fixture/live".to_string())),
        LiveCheckpointEvent::heartbeat(2, first_cursor, Some("fixture/live".to_string())),
        LiveCheckpointEvent::checkpoint_ready(3, second, Some("fixture/live".to_string())),
        LiveCheckpointEvent::manual_review(4, second_cursor, None),
    ];

    validate_live_event_sequence(&events).expect("validate live event sequence");
}

#[test]
fn live_input_rejects_out_of_order_checkpoint_cursor() {
    let events = vec![
        LiveCheckpointEvent::checkpoint_ready(
            1,
            support::checkpoint("session-alpha", 2, 60, true, "repair the plan"),
            None,
        ),
        LiveCheckpointEvent::checkpoint_ready(
            2,
            support::checkpoint("session-alpha", 1, 60, true, "repair the plan"),
            None,
        ),
    ];

    let error = validate_live_event_sequence(&events).expect_err("out-of-order cursor must fail");

    assert!(matches!(
        error,
        LiveInputError::OutOfOrderCheckpointCursor { .. }
    ));
}

#[test]
fn live_input_rejects_trigger_before_first_checkpoint() {
    let event = LiveCheckpointEvent::heartbeat(
        1,
        CheckpointCursor {
            session_id: "session-alpha".to_string(),
            ordinal: 1,
        },
        None,
    );

    let error =
        validate_live_event_sequence(&[event]).expect_err("synthetic event without cursor seed");

    assert!(matches!(
        error,
        LiveInputError::SyntheticEventBeforeCheckpoint { .. }
    ));
}
