use std::collections::BTreeMap;

use super::super::normalize_request;
use crate::backend_harness::test_support::ToyAdapter;
use crate::backend_harness::BackendDefaults;
use crate::AgentWrapperRunRequest;

#[test]
fn bh_c03_env_merge_precedence_via_normalize_request() {
    let adapter = ToyAdapter { fail_spawn: false };
    let defaults = BackendDefaults {
        env: BTreeMap::from([
            ("A".to_string(), "1".to_string()),
            ("B".to_string(), "1".to_string()),
        ]),
        default_timeout: None,
    };

    let request = AgentWrapperRunRequest {
        prompt: "hello".to_string(),
        env: BTreeMap::from([("B".to_string(), "2".to_string())]),
        ..Default::default()
    };

    let normalized = normalize_request(&adapter, &defaults, request).expect("normalizes");
    assert_eq!(normalized.env.get("A").map(String::as_str), Some("1"));
    assert_eq!(normalized.env.get("B").map(String::as_str), Some("2"));
}

#[test]
fn bh_c03_env_merge_empty_cases_via_normalize_request() {
    let adapter = ToyAdapter { fail_spawn: false };

    let defaults = BackendDefaults {
        env: BTreeMap::new(),
        default_timeout: None,
    };
    let request = AgentWrapperRunRequest {
        prompt: "hello".to_string(),
        env: BTreeMap::from([("X".to_string(), "x".to_string())]),
        ..Default::default()
    };
    let normalized = normalize_request(&adapter, &defaults, request).expect("normalizes");
    assert_eq!(normalized.env.get("X").map(String::as_str), Some("x"));

    let defaults = BackendDefaults {
        env: BTreeMap::from([("Y".to_string(), "y".to_string())]),
        default_timeout: None,
    };
    let request = AgentWrapperRunRequest {
        prompt: "hello".to_string(),
        env: BTreeMap::new(),
        ..Default::default()
    };
    let normalized = normalize_request(&adapter, &defaults, request).expect("normalizes");
    assert_eq!(normalized.env.get("Y").map(String::as_str), Some("y"));
}
