use serde_json::json;

use crate::{AgentWrapperBackend, AgentWrapperError};

use super::support::{backend_with_env, request};

#[tokio::test]
async fn opencode_backend_rejects_reserved_backend_namespace_until_keys_are_defined() {
    let backend = backend_with_env(Default::default());
    let mut request = request("Reply with OK.", None);
    request
        .extensions
        .insert("backend.opencode.future_key".to_string(), json!(true));

    let err = backend
        .run(request)
        .await
        .expect_err("unsupported backend namespace key must fail closed before spawn");
    match err {
        AgentWrapperError::UnsupportedCapability {
            agent_kind,
            capability,
        } => {
            assert_eq!(agent_kind, "opencode");
            assert_eq!(capability, "backend.opencode.future_key");
        }
        other => panic!("expected UnsupportedCapability, got {other:?}"),
    }
}
