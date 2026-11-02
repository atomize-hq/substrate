use std::{collections::BTreeMap, time::Duration};

use serde_json::json;

use super::super::{normalize_request, parse_ext_bool, parse_ext_string_enum};
use super::support::NeverCalledAdapter;
use crate::backend_harness::BackendDefaults;
use crate::{AgentWrapperError, AgentWrapperRunRequest};

#[test]
fn universal_invalid_request_empty_prompt_short_circuits_allowlist_and_policy() {
    let adapter = NeverCalledAdapter;
    let defaults = BackendDefaults::default();
    let request = AgentWrapperRunRequest {
        prompt: "   ".to_string(),
        timeout: Some(Duration::from_secs(123)),
        env: BTreeMap::from([("SECRET_ENV".to_string(), "SECRET_VAL".to_string())]),
        extensions: BTreeMap::from([(
            "unknown.key".to_string(),
            serde_json::Value::String("SECRET_SHOULD_NOT_LEAK".to_string()),
        )]),
        ..Default::default()
    };

    let err = match normalize_request(&adapter, &defaults, request) {
        Ok(_) => panic!("empty prompt must be rejected"),
        Err(err) => err,
    };
    match err {
        AgentWrapperError::InvalidRequest { message } => {
            assert_eq!(message, "prompt must not be empty");
        }
        other => panic!("expected InvalidRequest, got: {other:?}"),
    }
}

#[test]
fn parse_ext_bool_rejects_non_boolean() {
    let err = parse_ext_bool(&json!("nope"), "k").expect_err("expected bool parse failure");
    match err {
        AgentWrapperError::InvalidRequest { message } => {
            assert_eq!(message, "k must be a boolean");
            assert!(!message.contains("nope"));
        }
        other => panic!("expected InvalidRequest, got: {other:?}"),
    }
}

#[test]
fn parse_ext_string_enum_rejects_unknown_value_without_leaking_value() {
    let err = parse_ext_string_enum(&json!("nope"), "k", &["a", "b", "c"])
        .expect_err("expected enum parse failure");
    match err {
        AgentWrapperError::InvalidRequest { message } => {
            assert_eq!(message, "k must be one of: a | b | c");
            assert!(!message.contains("nope"));
        }
        other => panic!("expected InvalidRequest, got: {other:?}"),
    }
}
