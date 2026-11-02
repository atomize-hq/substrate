use std::collections::BTreeMap;

use serde_json::Value;

use crate::AgentWrapperError;

pub(crate) const EXT_SESSION_RESUME_V1: &str = "agent_api.session.resume.v1";
pub(crate) const EXT_SESSION_FORK_V1: &str = "agent_api.session.fork.v1";

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum SessionSelectorV1 {
    Last,
    Id { id: String },
}

pub(crate) fn parse_session_resume_v1(
    value: &Value,
) -> Result<SessionSelectorV1, AgentWrapperError> {
    parse_session_selector_object_v1(value, EXT_SESSION_RESUME_V1)
}

pub(crate) fn parse_session_fork_v1(value: &Value) -> Result<SessionSelectorV1, AgentWrapperError> {
    parse_session_selector_object_v1(value, EXT_SESSION_FORK_V1)
}

fn parse_session_selector_object_v1(
    value: &Value,
    ext_key: &str,
) -> Result<SessionSelectorV1, AgentWrapperError> {
    let obj = value
        .as_object()
        .ok_or_else(|| AgentWrapperError::InvalidRequest {
            message: format!("{ext_key} must be an object"),
        })?;

    let unknown_key = obj
        .keys()
        .filter(|k| k.as_str() != "selector" && k.as_str() != "id")
        .min();
    if let Some(key) = unknown_key {
        return Err(AgentWrapperError::InvalidRequest {
            message: format!("{ext_key} has unknown key: {key}"),
        });
    }

    let selector_value = obj
        .get("selector")
        .ok_or_else(|| AgentWrapperError::InvalidRequest {
            message: format!("{ext_key}.selector is required"),
        })?;
    let selector = selector_value
        .as_str()
        .ok_or_else(|| AgentWrapperError::InvalidRequest {
            message: format!("{ext_key}.selector must be a string"),
        })?;

    match selector {
        "last" => {
            if obj.get("id").is_some() {
                return Err(AgentWrapperError::InvalidRequest {
                    message: format!("{ext_key}.id must be absent when selector is \"last\""),
                });
            }

            Ok(SessionSelectorV1::Last)
        }
        "id" => {
            let id_value = obj
                .get("id")
                .ok_or_else(|| AgentWrapperError::InvalidRequest {
                    message: format!("{ext_key}.id is required when selector is \"id\""),
                })?;
            let id = id_value
                .as_str()
                .ok_or_else(|| AgentWrapperError::InvalidRequest {
                    message: format!("{ext_key}.id must be a string"),
                })?;
            if id.trim().is_empty() {
                return Err(AgentWrapperError::InvalidRequest {
                    message: format!("{ext_key}.id must be non-empty"),
                });
            }

            Ok(SessionSelectorV1::Id { id: id.to_string() })
        }
        _ => Err(AgentWrapperError::InvalidRequest {
            message: format!("{ext_key}.selector must be one of: last | id"),
        }),
    }
}

pub(crate) fn validate_resume_fork_mutual_exclusion(
    extensions: &BTreeMap<String, Value>,
) -> Result<(), AgentWrapperError> {
    if extensions.contains_key(EXT_SESSION_RESUME_V1)
        && extensions.contains_key(EXT_SESSION_FORK_V1)
    {
        return Err(AgentWrapperError::InvalidRequest {
            message: format!(
                "{EXT_SESSION_RESUME_V1} and {EXT_SESSION_FORK_V1} are mutually exclusive"
            ),
        });
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn resume_v1_valid_cases_parse() {
        struct Case {
            value: Value,
            expected: SessionSelectorV1,
        }

        let cases = [
            Case {
                value: json!({"selector": "last"}),
                expected: SessionSelectorV1::Last,
            },
            Case {
                value: json!({"selector": "id", "id": "abc"}),
                expected: SessionSelectorV1::Id {
                    id: "abc".to_string(),
                },
            },
            Case {
                value: json!({"selector": "id", "id": "  abc  "}),
                expected: SessionSelectorV1::Id {
                    id: "  abc  ".to_string(),
                },
            },
        ];

        for (idx, case) in cases.iter().enumerate() {
            let parsed = parse_session_resume_v1(&case.value)
                .unwrap_or_else(|err| panic!("case {idx}: expected Ok, got {err:?}"));
            assert_eq!(parsed, case.expected, "case {idx}");
        }
    }

    #[test]
    fn resume_v1_invalid_cases_rejected_with_pinned_messages() {
        struct Case {
            value: Value,
            expected_message: &'static str,
        }

        let cases = [
            Case {
                value: json!({}),
                expected_message: "agent_api.session.resume.v1.selector is required",
            },
            Case {
                value: json!({"selector": 1}),
                expected_message: "agent_api.session.resume.v1.selector must be a string",
            },
            Case {
                value: json!({"selector": "nope"}),
                expected_message: "agent_api.session.resume.v1.selector must be one of: last | id",
            },
            Case {
                value: json!({"selector": "id"}),
                expected_message:
                    "agent_api.session.resume.v1.id is required when selector is \"id\"",
            },
            Case {
                value: json!({"selector": "id", "id": true}),
                expected_message: "agent_api.session.resume.v1.id must be a string",
            },
            Case {
                value: json!({"selector": "id", "id": ""}),
                expected_message: "agent_api.session.resume.v1.id must be non-empty",
            },
            Case {
                value: json!({"selector": "id", "id": "   "}),
                expected_message: "agent_api.session.resume.v1.id must be non-empty",
            },
            Case {
                value: json!({"selector": "last", "id": "abc"}),
                expected_message:
                    "agent_api.session.resume.v1.id must be absent when selector is \"last\"",
            },
        ];

        for (idx, case) in cases.iter().enumerate() {
            let err = parse_session_resume_v1(&case.value)
                .expect_err("expected InvalidRequest schema validation error");
            match err {
                AgentWrapperError::InvalidRequest { message } => {
                    assert_eq!(message, case.expected_message, "case {idx}");
                }
                other => panic!("case {idx}: expected InvalidRequest, got: {other:?}"),
            }
        }
    }

    #[test]
    fn resume_v1_non_object_and_unknown_key_do_not_leak_values_in_error_messages() {
        let secret = "SECRET_SHOULD_NOT_LEAK";

        let err = parse_session_resume_v1(&json!(secret)).expect_err("expected non-object failure");
        match err {
            AgentWrapperError::InvalidRequest { message } => {
                assert_eq!(message, "agent_api.session.resume.v1 must be an object");
                assert!(!message.contains(secret));
            }
            other => panic!("expected InvalidRequest, got: {other:?}"),
        }

        let err = parse_session_resume_v1(&json!({"selector": "last", "extra": secret}))
            .expect_err("expected closed-schema failure");
        match err {
            AgentWrapperError::InvalidRequest { message } => {
                assert_eq!(
                    message,
                    "agent_api.session.resume.v1 has unknown key: extra"
                );
                assert!(!message.contains(secret));
            }
            other => panic!("expected InvalidRequest, got: {other:?}"),
        }
    }

    #[test]
    fn fork_v1_valid_cases_parse() {
        struct Case {
            value: Value,
            expected: SessionSelectorV1,
        }

        let cases = [
            Case {
                value: json!({"selector": "last"}),
                expected: SessionSelectorV1::Last,
            },
            Case {
                value: json!({"selector": "id", "id": "abc"}),
                expected: SessionSelectorV1::Id {
                    id: "abc".to_string(),
                },
            },
            Case {
                value: json!({"selector": "id", "id": "  abc  "}),
                expected: SessionSelectorV1::Id {
                    id: "  abc  ".to_string(),
                },
            },
        ];

        for (idx, case) in cases.iter().enumerate() {
            let parsed = parse_session_fork_v1(&case.value)
                .unwrap_or_else(|err| panic!("case {idx}: expected Ok, got {err:?}"));
            assert_eq!(parsed, case.expected, "case {idx}");
        }
    }

    #[test]
    fn fork_v1_invalid_cases_rejected_with_pinned_messages() {
        struct Case {
            value: Value,
            expected_message: &'static str,
        }

        let cases = [
            Case {
                value: json!({}),
                expected_message: "agent_api.session.fork.v1.selector is required",
            },
            Case {
                value: json!({"selector": 1}),
                expected_message: "agent_api.session.fork.v1.selector must be a string",
            },
            Case {
                value: json!({"selector": "nope"}),
                expected_message: "agent_api.session.fork.v1.selector must be one of: last | id",
            },
            Case {
                value: json!({"selector": "id"}),
                expected_message:
                    "agent_api.session.fork.v1.id is required when selector is \"id\"",
            },
            Case {
                value: json!({"selector": "id", "id": true}),
                expected_message: "agent_api.session.fork.v1.id must be a string",
            },
            Case {
                value: json!({"selector": "id", "id": ""}),
                expected_message: "agent_api.session.fork.v1.id must be non-empty",
            },
            Case {
                value: json!({"selector": "id", "id": "   "}),
                expected_message: "agent_api.session.fork.v1.id must be non-empty",
            },
            Case {
                value: json!({"selector": "last", "id": "abc"}),
                expected_message:
                    "agent_api.session.fork.v1.id must be absent when selector is \"last\"",
            },
        ];

        for (idx, case) in cases.iter().enumerate() {
            let err = parse_session_fork_v1(&case.value)
                .expect_err("expected InvalidRequest schema validation error");
            match err {
                AgentWrapperError::InvalidRequest { message } => {
                    assert_eq!(message, case.expected_message, "case {idx}");
                }
                other => panic!("case {idx}: expected InvalidRequest, got: {other:?}"),
            }
        }
    }

    #[test]
    fn fork_v1_non_object_and_unknown_key_do_not_leak_values_in_error_messages() {
        let secret = "SECRET_SHOULD_NOT_LEAK";

        let err = parse_session_fork_v1(&json!(secret)).expect_err("expected non-object failure");
        match err {
            AgentWrapperError::InvalidRequest { message } => {
                assert_eq!(message, "agent_api.session.fork.v1 must be an object");
                assert!(!message.contains(secret));
            }
            other => panic!("expected InvalidRequest, got: {other:?}"),
        }

        let err = parse_session_fork_v1(&json!({"selector": "last", "extra": secret}))
            .expect_err("expected closed-schema failure");
        match err {
            AgentWrapperError::InvalidRequest { message } => {
                assert_eq!(message, "agent_api.session.fork.v1 has unknown key: extra");
                assert!(!message.contains(secret));
            }
            other => panic!("expected InvalidRequest, got: {other:?}"),
        }
    }
}
