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
fn parent_run_id_roundtrips_when_present() {
    let mut value = minimal_valid_envelope_json();
    value.as_object_mut().expect("envelope object").insert(
        "parent_run_id".to_string(),
        json!("0195f8f1-7a35-7b7f-9c4d-9a7c2f5d6f11"),
    );

    let event: AgentEvent = serde_json::from_value(value).expect("deserialize AgentEvent");
    let roundtrip = serde_json::to_value(&event).expect("serialize AgentEvent");

    assert_eq!(
        roundtrip.get("parent_run_id").and_then(Value::as_str),
        Some("0195f8f1-7a35-7b7f-9c4d-9a7c2f5d6f11"),
        "expected parent_run_id to survive roundtrip; got: {roundtrip}"
    );
}

#[test]
fn parent_run_id_omits_by_field_absence_when_unset() {
    let event: AgentEvent =
        serde_json::from_value(minimal_valid_envelope_json()).expect("deserialize AgentEvent");
    let roundtrip = serde_json::to_value(&event).expect("serialize AgentEvent");

    assert!(
        roundtrip.get("parent_run_id").is_none(),
        "expected parent_run_id to omit when unset; got: {roundtrip}"
    );
    assert!(
        !roundtrip.to_string().contains("\"parent_run_id\":null"),
        "parent_run_id must not serialize as null when unset: {roundtrip}"
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

#[test]
fn tuple_publication_uses_canonical_object_names_and_required_fields() {
    let value = json!({
        "ts": "2026-04-05T00:00:00Z",
        "kind": "status",
        "agent_id": "demo-agent",
        "orchestration_session_id": "0195f8f1-7a34-7b7f-9c4d-9a7c2f5d6f12",
        "run_id": "0195f8f1-7a35-7b7f-9c4d-9a7c2f5d6f13",
        "backend_id": "cli:codex",
        "data": { "message": "ok" },
        "identity_tuple": {
            "client": "codex",
            "router": "substrate_gateway",
            "provider": "openai",
            "auth_authority": "codex_subscription",
            "protocol": "openai.responses"
        },
        "placement_posture": {
            "execution": "in_world"
        }
    });

    let event: AgentEvent = serde_json::from_value(value).expect("deserialize AgentEvent");
    let roundtrip = serde_json::to_value(&event).expect("serialize AgentEvent");

    for key in ["client", "router", "protocol"] {
        assert!(
            roundtrip
                .pointer(&format!("/identity_tuple/{key}"))
                .is_some(),
            "expected required tuple field `{key}` under `/identity_tuple`; got: {roundtrip}"
        );
    }

    assert_eq!(
        roundtrip
            .pointer("/identity_tuple/client")
            .and_then(Value::as_str),
        Some("codex")
    );
    assert_eq!(
        roundtrip
            .pointer("/identity_tuple/router")
            .and_then(Value::as_str),
        Some("substrate_gateway")
    );
    assert_eq!(
        roundtrip
            .pointer("/identity_tuple/provider")
            .and_then(Value::as_str),
        Some("openai")
    );
    assert_eq!(
        roundtrip
            .pointer("/identity_tuple/auth_authority")
            .and_then(Value::as_str),
        Some("codex_subscription")
    );
    assert_eq!(
        roundtrip
            .pointer("/identity_tuple/protocol")
            .and_then(Value::as_str),
        Some("openai.responses")
    );
    assert_eq!(
        roundtrip
            .pointer("/placement_posture/execution")
            .and_then(Value::as_str),
        Some("in_world")
    );
    assert_eq!(
        roundtrip.get("backend_id").and_then(Value::as_str),
        Some("cli:codex"),
        "backend_id should remain a separate selector, not substitute for tuple fields: {roundtrip}"
    );

    for legacy_key in ["client", "router", "provider", "auth_authority", "protocol"] {
        assert!(
            roundtrip.get(legacy_key).is_none(),
            "tuple metadata should publish under canonical objects, not as legacy flat `{legacy_key}` fields: {roundtrip}"
        );
    }
}

#[test]
fn tuple_optional_fields_omit_by_field_absence_only() {
    let value = json!({
        "ts": "2026-04-05T00:00:00Z",
        "kind": "status",
        "agent_id": "demo-agent",
        "orchestration_session_id": "0195f8f1-7a34-7b7f-9c4d-9a7c2f5d6f12",
        "run_id": "0195f8f1-7a35-7b7f-9c4d-9a7c2f5d6f13",
        "data": { "message": "ok" },
        "identity_tuple": {
            "client": "codex",
            "router": "substrate_gateway",
            "protocol": "openai.responses"
        },
        "placement_posture": {
            "execution": "in_world"
        }
    });

    let event: AgentEvent = serde_json::from_value(value).expect("deserialize AgentEvent");
    let roundtrip = serde_json::to_value(&event).expect("serialize AgentEvent");

    assert!(
        roundtrip.get("identity_tuple").is_some(),
        "expected canonical identity tuple object to be preserved: {roundtrip}"
    );
    assert!(
        roundtrip.pointer("/identity_tuple/provider").is_none(),
        "provider must omit by field absence only when unresolved: {roundtrip}"
    );
    assert!(
        roundtrip
            .pointer("/identity_tuple/auth_authority")
            .is_none(),
        "auth_authority must omit by field absence only when unresolved: {roundtrip}"
    );
    assert!(
        !roundtrip.to_string().contains("unknown"),
        "optional tuple omissions must not be backfilled with placeholder text: {roundtrip}"
    );
    assert!(
        !roundtrip.to_string().contains("\"provider\":null"),
        "optional tuple omissions must not serialize as null: {roundtrip}"
    );
    assert!(
        !roundtrip.to_string().contains("\"auth_authority\":null"),
        "optional tuple omissions must not serialize as null: {roundtrip}"
    );
}

#[test]
fn tuple_ids_reject_backend_grammar_uppercase_and_placeholder_tokens() {
    let invalid = json!({
        "ts": "2026-04-05T00:00:00Z",
        "kind": "status",
        "agent_id": "demo-agent",
        "orchestration_session_id": "0195f8f1-7a34-7b7f-9c4d-9a7c2f5d6f12",
        "run_id": "0195f8f1-7a35-7b7f-9c4d-9a7c2f5d6f13",
        "data": { "message": "ok" },
        "identity_tuple": {
            "client": "Codex",
            "router": "api:openai",
            "provider": "unknown",
            "auth_authority": "n/a",
            "protocol": "openai_responses"
        },
        "placement_posture": {
            "execution": "host_only"
        }
    });

    let result: Result<AgentEvent, _> = serde_json::from_value(invalid);
    assert!(
        result.is_err(),
        "expected tuple token validation to reject uppercase ids, backend-id grammar, placeholder omissions, and non-dotted protocols"
    );
}

#[test]
fn direct_provider_path_requires_host_only_without_bridge_transport() {
    let invalid = json!({
        "ts": "2026-04-05T00:00:00Z",
        "kind": "status",
        "agent_id": "demo-agent",
        "orchestration_session_id": "0195f8f1-7a34-7b7f-9c4d-9a7c2f5d6f12",
        "run_id": "0195f8f1-7a35-7b7f-9c4d-9a7c2f5d6f13",
        "data": { "message": "ok" },
        "identity_tuple": {
            "client": "codex",
            "router": "direct_provider_path",
            "protocol": "openai.responses"
        },
        "placement_posture": {
            "execution": "in_world",
            "host_to_world_bridge": true
        }
    });

    let result: Result<AgentEvent, _> = serde_json::from_value(invalid);
    assert!(
        result.is_err(),
        "expected direct_provider_path to be rejected unless placement_posture.execution=host_only and bridge transport is absent"
    );
}

#[test]
fn trace_tuple_metadata_preserves_existing_join_keys() {
    let value = json!({
        "ts": "2026-04-05T00:00:00Z",
        "kind": "status",
        "agent_id": "demo-agent",
        "orchestration_session_id": "0195f8f1-7a34-7b7f-9c4d-9a7c2f5d6f12",
        "run_id": "0195f8f1-7a35-7b7f-9c4d-9a7c2f5d6f13",
        "parent_run_id": "0195f8f1-7a35-7b7f-9c4d-9a7c2f5d6f10",
        "backend_id": "cli:codex",
        "world_id": "wld_test",
        "cmd_id": "cmd_test",
        "span_id": "spn_test",
        "data": { "message": "ok" },
        "identity_tuple": {
            "client": "codex",
            "router": "substrate_gateway",
            "provider": "openai",
            "auth_authority": "codex_subscription",
            "protocol": "openai.responses"
        },
        "placement_posture": {
            "execution": "in_world"
        }
    });

    let event: AgentEvent = serde_json::from_value(value).expect("deserialize AgentEvent");
    let roundtrip = serde_json::to_value(&event).expect("serialize AgentEvent");

    assert_eq!(
        roundtrip.get("backend_id").and_then(Value::as_str),
        Some("cli:codex")
    );
    assert_eq!(
        roundtrip.get("world_id").and_then(Value::as_str),
        Some("wld_test")
    );
    assert_eq!(
        roundtrip.get("cmd_id").and_then(Value::as_str),
        Some("cmd_test")
    );
    assert_eq!(
        roundtrip.get("span_id").and_then(Value::as_str),
        Some("spn_test")
    );
    assert_eq!(
        roundtrip.get("parent_run_id").and_then(Value::as_str),
        Some("0195f8f1-7a35-7b7f-9c4d-9a7c2f5d6f10")
    );
    assert_eq!(
        roundtrip
            .pointer("/identity_tuple/router")
            .and_then(Value::as_str),
        Some("substrate_gateway"),
        "tuple metadata should augment existing join keys instead of replacing them: {roundtrip}"
    );
    assert_eq!(
        roundtrip
            .pointer("/placement_posture/execution")
            .and_then(Value::as_str),
        Some("in_world")
    );
}

#[test]
fn tuple_publication_rejects_secret_like_values_and_credential_paths() {
    let invalid = json!({
        "ts": "2026-04-05T00:00:00Z",
        "kind": "status",
        "agent_id": "demo-agent",
        "orchestration_session_id": "0195f8f1-7a34-7b7f-9c4d-9a7c2f5d6f12",
        "run_id": "0195f8f1-7a35-7b7f-9c4d-9a7c2f5d6f13",
        "data": { "message": "ok" },
        "identity_tuple": {
            "client": "codex",
            "router": "substrate_gateway",
            "provider": "https://api.openai.com/v1",
            "auth_authority": "~/.codex/auth.json",
            "protocol": "openai.responses"
        },
        "placement_posture": {
            "execution": "in_world"
        }
    });

    let result: Result<AgentEvent, _> = serde_json::from_value(invalid);
    assert!(
        result.is_err(),
        "expected tuple publication to reject endpoint URLs and raw credential paths instead of serializing secret-adjacent values"
    );
}
