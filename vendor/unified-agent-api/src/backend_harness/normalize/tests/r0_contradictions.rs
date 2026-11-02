use serde_json::{json, Value};

use super::super::normalize_request;
use super::support::PolicyFnAdapter;
use crate::backend_harness::BackendDefaults;
use crate::{AgentWrapperError, AgentWrapperRunRequest};

#[test]
fn bh_r0_external_sandbox_key_is_rejected_before_policy_validation_via_normalize_request() {
    const SUPPORTED: [&str; 1] = ["agent_api.exec.non_interactive"];
    let adapter = PolicyFnAdapter::panic_on_policy(&SUPPORTED);
    let defaults = BackendDefaults::default();
    let secret = "SECRET_SHOULD_NOT_LEAK";

    let mut request = AgentWrapperRunRequest {
        prompt: "hello".to_string(),
        ..Default::default()
    };
    request.extensions.insert(
        "agent_api.exec.external_sandbox.v1".to_string(),
        Value::String(secret.to_string()),
    );
    request.extensions.insert(
        "agent_api.exec.non_interactive".to_string(),
        Value::Bool(false),
    );

    let err = match normalize_request(&adapter, &defaults, request) {
        Ok(_) => panic!("unsupported key must fail closed"),
        Err(err) => err,
    };
    match &err {
        AgentWrapperError::UnsupportedCapability {
            agent_kind,
            capability,
        } => {
            assert_eq!(agent_kind, "toy");
            assert_eq!(capability, "agent_api.exec.external_sandbox.v1");
        }
        other => panic!("expected UnsupportedCapability, got: {other:?}"),
    }
    assert!(!err.to_string().contains(secret));
}

#[test]
fn bh_r0_unsupported_capability_beats_contradiction_rules_for_resume_fork_via_normalize_request() {
    const SUPPORTED: [&str; 1] = ["agent_api.session.resume.v1"];
    let adapter = PolicyFnAdapter::panic_on_policy(&SUPPORTED);
    let defaults = BackendDefaults::default();
    let secret = "SECRET_SHOULD_NOT_LEAK";

    let mut request = AgentWrapperRunRequest {
        prompt: "hello".to_string(),
        ..Default::default()
    };
    request.extensions.insert(
        "agent_api.session.fork.v1".to_string(),
        Value::String(secret.to_string()),
    );
    request.extensions.insert(
        "agent_api.session.resume.v1".to_string(),
        json!({"selector": "last"}),
    );

    let err = match normalize_request(&adapter, &defaults, request) {
        Ok(_) => panic!("unsupported fork key must fail closed before contradiction rules"),
        Err(err) => err,
    };
    match &err {
        AgentWrapperError::UnsupportedCapability { capability, .. } => {
            assert_eq!(capability, "agent_api.session.fork.v1");
        }
        other => panic!("expected UnsupportedCapability, got: {other:?}"),
    }
    assert!(!err.to_string().contains(secret));
}

#[test]
fn bh_r0_unsupported_capability_beats_contradiction_rules_for_fork_resume_via_normalize_request() {
    const SUPPORTED: [&str; 1] = ["agent_api.session.fork.v1"];
    let adapter = PolicyFnAdapter::panic_on_policy(&SUPPORTED);
    let defaults = BackendDefaults::default();
    let secret = "SECRET_SHOULD_NOT_LEAK";

    let mut request = AgentWrapperRunRequest {
        prompt: "hello".to_string(),
        ..Default::default()
    };
    request.extensions.insert(
        "agent_api.session.fork.v1".to_string(),
        json!({"selector": "last"}),
    );
    request.extensions.insert(
        "agent_api.session.resume.v1".to_string(),
        Value::String(secret.to_string()),
    );

    let err = match normalize_request(&adapter, &defaults, request) {
        Ok(_) => panic!("unsupported resume key must fail closed before contradiction rules"),
        Err(err) => err,
    };
    match &err {
        AgentWrapperError::UnsupportedCapability { capability, .. } => {
            assert_eq!(capability, "agent_api.session.resume.v1");
        }
        other => panic!("expected UnsupportedCapability, got: {other:?}"),
    }
    assert!(!err.to_string().contains(secret));
}

#[test]
fn bh_r0_resume_fork_contradiction_applies_only_after_all_keys_are_supported_via_normalize_request()
{
    const SUPPORTED: [&str; 2] = ["agent_api.session.fork.v1", "agent_api.session.resume.v1"];

    fn validate_policy(request: &AgentWrapperRunRequest) -> Result<(), AgentWrapperError> {
        if request
            .extensions
            .contains_key("agent_api.session.resume.v1")
            && request.extensions.contains_key("agent_api.session.fork.v1")
        {
            return Err(AgentWrapperError::InvalidRequest {
                message: "agent_api.session.resume.v1 and agent_api.session.fork.v1 are mutually exclusive"
                    .to_string(),
            });
        }
        Ok(())
    }

    fn validate_policy_toy(
        request: &AgentWrapperRunRequest,
    ) -> Result<crate::backend_harness::test_support::ToyPolicy, AgentWrapperError> {
        validate_policy(request)?;
        Ok(crate::backend_harness::test_support::ToyPolicy)
    }

    let adapter = PolicyFnAdapter::new(&SUPPORTED, validate_policy_toy);
    let defaults = BackendDefaults::default();

    let mut request = AgentWrapperRunRequest {
        prompt: "hello".to_string(),
        ..Default::default()
    };
    request.extensions.insert(
        "agent_api.session.fork.v1".to_string(),
        json!({"selector": "last"}),
    );
    request.extensions.insert(
        "agent_api.session.resume.v1".to_string(),
        json!({"selector": "last"}),
    );

    let err = match normalize_request(&adapter, &defaults, request) {
        Ok(_) => panic!("expected mutual-exclusion InvalidRequest"),
        Err(err) => err,
    };
    match err {
        AgentWrapperError::InvalidRequest { message } => {
            assert_eq!(
                message,
                "agent_api.session.resume.v1 and agent_api.session.fork.v1 are mutually exclusive"
            );
        }
        other => panic!("expected InvalidRequest, got: {other:?}"),
    }
}
