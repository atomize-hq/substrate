use super::support::*;
use serde_json::json;

const MODEL_ID_KEY: &str = crate::EXT_AGENT_API_CONFIG_MODEL_V1;

#[test]
fn codex_policy_fork_model_override_is_rejected_pre_spawn() {
    let adapter = test_adapter();

    let request = AgentWrapperRunRequest {
        prompt: "hello".to_string(),
        extensions: [
            (
                EXT_SESSION_FORK_V1.to_string(),
                json!({
                    "selector": "last",
                }),
            ),
            (MODEL_ID_KEY.to_string(), json!("gpt-5-codex")),
        ]
        .into_iter()
        .collect(),
        ..Default::default()
    };

    let err = adapter
        .validate_and_extract_policy(&request)
        .expect_err("policy extraction should fail");
    match err {
        AgentWrapperError::Backend { message } => {
            assert_eq!(message, "model override unsupported for codex fork");
        }
        other => panic!("expected Backend error, got: {other:?}"),
    }
}

#[test]
fn codex_policy_fork_invalid_model_override_is_invalid_request() {
    let adapter = test_adapter();

    let request = AgentWrapperRunRequest {
        prompt: "hello".to_string(),
        extensions: [
            (
                EXT_SESSION_FORK_V1.to_string(),
                json!({
                    "selector": "last",
                }),
            ),
            (MODEL_ID_KEY.to_string(), json!("   ")),
        ]
        .into_iter()
        .collect(),
        ..Default::default()
    };

    let err = adapter
        .validate_and_extract_policy(&request)
        .expect_err("policy extraction should fail");
    match err {
        AgentWrapperError::InvalidRequest { message } => {
            assert_eq!(message, "invalid agent_api.config.model.v1");
        }
        other => panic!("expected InvalidRequest, got: {other:?}"),
    }
}
