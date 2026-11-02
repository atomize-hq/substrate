#[path = "../examples/support/fixtures.rs"]
mod fixtures;

use serde_json::Value;

#[test]
fn streaming_fixture_covers_event_shapes() {
    let events: Vec<&'static str> = fixtures::streaming_events().collect();
    assert!(
        events.iter().any(|line| line.contains("thread.started")),
        "streaming fixture should include thread.start"
    );
    assert!(
        events.iter().any(|line| line.contains("turn.failed")),
        "streaming fixture should include a failure case"
    );

    for line in events {
        let value: Value = serde_json::from_str(line).expect("valid streaming fixture JSON");
        let kind = value
            .get("type")
            .and_then(Value::as_str)
            .expect("fixture event has type");

        if kind.starts_with("item.") {
            assert!(
                value.get("item").is_some(),
                "item.* events include an item body"
            );
            assert!(
                value.get("thread_id").is_some(),
                "item events carry thread_id"
            );
            assert!(value.get("turn_id").is_some(), "item events carry turn_id");
        }
    }
}

#[test]
fn resume_fixture_includes_thread_and_turn_ids() {
    let resume_events: Vec<&'static str> = fixtures::resume_events().collect();
    assert!(
        !resume_events.is_empty(),
        "resume fixture should not be empty"
    );

    let first: Value =
        serde_json::from_str(resume_events[0]).expect("first resume event parses as JSON");
    assert_eq!(
        first.get("type").and_then(Value::as_str),
        Some("thread.resumed"),
        "resume fixture should start with thread.resumed"
    );

    for line in resume_events {
        let value: Value = serde_json::from_str(line).expect("valid resume fixture JSON");
        assert!(
            value.get("thread_id").is_some(),
            "resume events carry thread_id"
        );
        assert!(value.get("type").is_some(), "resume events carry type");
    }
}

#[test]
fn apply_fixture_parses_and_carries_exit_code() {
    let result: Value =
        serde_json::from_str(fixtures::apply_result()).expect("apply result fixture parses");
    assert_eq!(
        result.get("type").and_then(Value::as_str),
        Some("apply.result"),
        "apply result fixture has type"
    );
    assert!(result.get("exit_code").is_some(), "exit_code is present");
    assert!(result.get("stdout").is_some(), "stdout is present");
    assert!(result.get("stderr").is_some(), "stderr is present");
}
