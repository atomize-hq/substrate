use serde_json::Value;

use super::super::normalize_request;
use super::support::PolicyFnAdapter;
use crate::backend_harness::test_support::ToyAdapter;
use crate::backend_harness::BackendDefaults;
use crate::{AgentWrapperError, AgentWrapperRunRequest};

#[test]
fn bh_c02_unknown_extension_key_is_rejected_via_normalize_request() {
    const SUPPORTED: [&str; 1] = ["known.key"];
    let adapter = PolicyFnAdapter::panic_on_policy(&SUPPORTED);
    let defaults = BackendDefaults::default();

    let mut request = AgentWrapperRunRequest {
        prompt: "hello".to_string(),
        ..Default::default()
    };
    request.extensions.insert(
        "unknown.key".to_string(),
        Value::String("SECRET_SHOULD_NOT_LEAK".to_string()),
    );

    let err = match normalize_request(&adapter, &defaults, request) {
        Ok(_) => panic!("unknown key must fail closed"),
        Err(err) => err,
    };
    match &err {
        AgentWrapperError::UnsupportedCapability {
            agent_kind,
            capability,
        } => {
            assert_eq!(agent_kind, "toy");
            assert_eq!(capability, "unknown.key");
        }
        other => panic!("expected UnsupportedCapability, got: {other:?}"),
    }
    assert!(!err.to_string().contains("SECRET_SHOULD_NOT_LEAK"));
}

#[test]
fn bh_c02_multiple_unknown_extension_keys_report_lexicographically_smallest_via_normalize_request()
{
    let adapter = ToyAdapter { fail_spawn: false };
    let defaults = BackendDefaults::default();
    let mut request = AgentWrapperRunRequest {
        prompt: "hello".to_string(),
        ..Default::default()
    };
    request
        .extensions
        .insert("zzz.unknown".to_string(), Value::Bool(true));
    request
        .extensions
        .insert("aaa.unknown".to_string(), Value::Bool(true));

    let err = match normalize_request(&adapter, &defaults, request) {
        Ok(_) => panic!("unknown key must fail closed"),
        Err(err) => err,
    };
    match err {
        AgentWrapperError::UnsupportedCapability { capability, .. } => {
            assert_eq!(capability, "aaa.unknown");
        }
        other => panic!("expected UnsupportedCapability, got: {other:?}"),
    }
}

#[test]
fn bh_c02_all_keys_allowed_passes_via_normalize_request() {
    let adapter = ToyAdapter { fail_spawn: false };
    let defaults = BackendDefaults::default();
    let mut request = AgentWrapperRunRequest {
        prompt: "hello".to_string(),
        ..Default::default()
    };
    request.extensions.insert(
        "agent_api.exec.non_interactive".to_string(),
        Value::Bool(true),
    );
    request
        .extensions
        .insert("backend.toy.example".to_string(), Value::Bool(true));

    let normalized = normalize_request(&adapter, &defaults, request).expect("all keys allowed");
    assert_eq!(normalized.agent_kind.as_str(), "toy");
    assert_eq!(normalized.prompt, "hello");
    assert_eq!(normalized.model_id, None);
}
