use serde_json::{json, Value};
use substrate_common::agent_events::{AgentEvent, AgentEventKind, MessageEventKind};

fn minimal_valid_envelope_json() -> Value {
    json!({
        "ts": "2026-04-05T00:00:00Z",
        "kind": "status",
        "agent_id": "demo-agent",
        "orchestration_session_id": "0195f8f1-7a34-7b7f-9c4d-9a7c2f5d6f12",
        "run_id": "0195f8f1-7a35-7b7f-9c4d-9a7c2f5d6f13",
        "data": { "message": "ok" }
    })
}

#[test]
fn envelope_required_fields_are_present_at_top_level() {
    let event: AgentEvent =
        serde_json::from_value(minimal_valid_envelope_json()).expect("deserialize AgentEvent");
    let roundtrip = serde_json::to_value(&event).expect("serialize AgentEvent");

    for key in [
        "ts",
        "kind",
        "agent_id",
        "orchestration_session_id",
        "run_id",
        "data",
    ] {
        assert!(
            roundtrip.get(key).is_some(),
            "expected required envelope field `{key}` at top-level; got: {roundtrip}"
        );
    }
}

#[test]
fn envelope_missing_required_fields_is_rejected() {
    let missing_orchestration = json!({
        "ts": "2026-04-05T00:00:00Z",
        "kind": "status",
        "agent_id": "demo-agent",
        "run_id": "0195f8f1-7a35-7b7f-9c4d-9a7c2f5d6f13",
        "data": { "message": "ok" }
    });

    let result: Result<AgentEvent, _> = serde_json::from_value(missing_orchestration);
    assert!(
        result.is_err(),
        "expected missing required field to error, but deserialization succeeded"
    );
}

#[test]
fn safe_channel_roundtrips() {
    let mut value = minimal_valid_envelope_json();
    value
        .as_object_mut()
        .expect("envelope object")
        .insert("channel".to_string(), json!("agent_status"));

    let event: AgentEvent = serde_json::from_value(value).expect("deserialize AgentEvent");
    let roundtrip = serde_json::to_value(&event).expect("serialize AgentEvent");

    assert_eq!(
        roundtrip.get("channel").and_then(Value::as_str),
        Some("agent_status"),
        "expected safe channel value to be preserved; got: {roundtrip}"
    );
}

#[test]
fn unsafe_channel_is_dropped_and_never_emitted() {
    let unsafe_channel = "Authorization: Bearer secret123\n";
    let mut value = minimal_valid_envelope_json();
    value
        .as_object_mut()
        .expect("envelope object")
        .insert("channel".to_string(), json!(unsafe_channel));

    let event: AgentEvent = serde_json::from_value(value).expect("deserialize AgentEvent");
    let roundtrip = serde_json::to_value(&event).expect("serialize AgentEvent");
    let serialized = roundtrip.to_string();

    assert!(
        roundtrip.get("channel").is_none(),
        "expected unsafe channel to be dropped; got: {roundtrip}"
    );
    assert!(
        !serialized.contains(unsafe_channel),
        "dropped channel value must never be emitted; got: {roundtrip}"
    );
}

#[test]
fn alert_envelope_roundtrips_with_required_code_and_message() {
    let value = json!({
        "ts": "2026-04-05T00:00:00Z",
        "kind": "alert",
        "agent_id": "demo-agent",
        "orchestration_session_id": "0195f8f1-7a34-7b7f-9c4d-9a7c2f5d6f12",
        "run_id": "0195f8f1-7a35-7b7f-9c4d-9a7c2f5d6f13",
        "data": {
            "code": "world_restarted",
            "message": "world restarted due to policy drift"
        }
    });

    let event: AgentEvent = serde_json::from_value(value).expect("deserialize alert AgentEvent");
    let roundtrip = serde_json::to_value(&event).expect("serialize AgentEvent");

    assert_eq!(event.kind, AgentEventKind::Alert);
    assert_eq!(
        roundtrip.pointer("/data/code").and_then(Value::as_str),
        Some("world_restarted")
    );
    assert_eq!(
        roundtrip.pointer("/data/message").and_then(Value::as_str),
        Some("world restarted due to policy drift")
    );
}

#[test]
fn message_constructor_emits_non_alert_message_event_shape() {
    let event = AgentEvent::message(
        "demo-agent",
        "0195f8f1-7a34-7b7f-9c4d-9a7c2f5d6f12",
        "0195f8f1-7a35-7b7f-9c4d-9a7c2f5d6f13",
        MessageEventKind::TaskEnd,
        "command finished",
    );
    let roundtrip = serde_json::to_value(&event).expect("serialize AgentEvent");

    assert_eq!(event.kind, AgentEventKind::TaskEnd);
    assert_eq!(
        roundtrip.pointer("/data/message").and_then(Value::as_str),
        Some("command finished")
    );
    assert!(
        roundtrip.pointer("/data/code").is_none(),
        "message constructor must not synthesize alert-style codes: {roundtrip}"
    );
}

#[test]
fn alert_constructor_emits_required_alert_fields() {
    let event = AgentEvent::alert(
        "demo-agent",
        "0195f8f1-7a34-7b7f-9c4d-9a7c2f5d6f12",
        "0195f8f1-7a35-7b7f-9c4d-9a7c2f5d6f13",
        "world_restart_required",
        "world restart required before continuing",
    );
    let roundtrip = serde_json::to_value(&event).expect("serialize AgentEvent");

    assert_eq!(event.kind, AgentEventKind::Alert);
    assert_eq!(
        roundtrip.pointer("/data/code").and_then(Value::as_str),
        Some("world_restart_required")
    );
    assert_eq!(
        roundtrip.pointer("/data/message").and_then(Value::as_str),
        Some("world restart required before continuing")
    );
}
