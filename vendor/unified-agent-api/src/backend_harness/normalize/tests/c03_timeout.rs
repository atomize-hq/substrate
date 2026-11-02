use std::{collections::BTreeMap, time::Duration};

use super::super::normalize_request;
use crate::backend_harness::test_support::ToyAdapter;
use crate::backend_harness::BackendDefaults;
use crate::AgentWrapperRunRequest;

#[test]
fn bh_c03_timeout_derivation_matrix_via_normalize_request() {
    let adapter = ToyAdapter { fail_spawn: false };

    struct Case {
        request: Option<Duration>,
        default: Option<Duration>,
        expected: Option<Duration>,
    }

    let cases = [
        Case {
            request: Some(Duration::from_secs(5)),
            default: Some(Duration::from_secs(7)),
            expected: Some(Duration::from_secs(5)),
        },
        Case {
            request: Some(Duration::from_secs(5)),
            default: None,
            expected: Some(Duration::from_secs(5)),
        },
        Case {
            request: None,
            default: Some(Duration::from_secs(7)),
            expected: Some(Duration::from_secs(7)),
        },
        Case {
            request: None,
            default: None,
            expected: None,
        },
    ];

    for case in cases {
        let defaults = BackendDefaults {
            env: BTreeMap::new(),
            default_timeout: case.default,
        };
        let request = AgentWrapperRunRequest {
            prompt: "hello".to_string(),
            timeout: case.request,
            ..Default::default()
        };
        let normalized = normalize_request(&adapter, &defaults, request).expect("normalizes");
        assert_eq!(normalized.effective_timeout, case.expected);
    }
}

#[test]
fn bh_c03_timeout_duration_zero_is_preserved_via_normalize_request() {
    let adapter = ToyAdapter { fail_spawn: false };
    let defaults = BackendDefaults {
        env: BTreeMap::new(),
        default_timeout: Some(Duration::from_secs(7)),
    };
    let request = AgentWrapperRunRequest {
        prompt: "hello".to_string(),
        timeout: Some(Duration::ZERO),
        ..Default::default()
    };
    let normalized = normalize_request(&adapter, &defaults, request).expect("normalizes");
    assert_eq!(normalized.effective_timeout, Some(Duration::ZERO));
}
