#[cfg(feature = "codex")]
mod codex {
    use std::path::PathBuf;

    use agent_api::{
        backends::codex::{CodexBackend, CodexBackendConfig},
        AgentWrapperBackend, AgentWrapperError, AgentWrapperRunRequest,
    };
    use serde_json::Value;

    fn test_backend() -> CodexBackend {
        CodexBackend::new(CodexBackendConfig {
            binary: Some(PathBuf::from("definitely-not-a-real-codex-binary")),
            allow_external_sandbox_exec: true,
            ..Default::default()
        })
    }

    #[tokio::test]
    async fn external_sandbox_type_mismatch_is_rejected_pre_spawn() {
        let backend = test_backend();

        let err = backend
            .run(AgentWrapperRunRequest {
                prompt: "hello".to_string(),
                extensions: [(
                    "agent_api.exec.external_sandbox.v1".to_string(),
                    Value::String("not-bool".to_string()),
                )]
                .into_iter()
                .collect(),
                ..Default::default()
            })
            .await
            .unwrap_err();

        let message = match err {
            AgentWrapperError::InvalidRequest { message } => message,
            other => panic!("expected InvalidRequest, got: {other:?}"),
        };
        assert!(
            message.contains("agent_api.exec.external_sandbox.v1"),
            "expected error message to mention the key (got: {message:?})"
        );
        assert!(
            message.to_ascii_lowercase().contains("boolean"),
            "expected error message to mention boolean (got: {message:?})"
        );
    }

    #[tokio::test]
    async fn external_sandbox_true_rejects_non_interactive_false_pre_spawn() {
        let backend = test_backend();

        let err = backend
            .run(AgentWrapperRunRequest {
                prompt: "hello".to_string(),
                extensions: [
                    (
                        "agent_api.exec.external_sandbox.v1".to_string(),
                        Value::Bool(true),
                    ),
                    (
                        "agent_api.exec.non_interactive".to_string(),
                        Value::Bool(false),
                    ),
                ]
                .into_iter()
                .collect(),
                ..Default::default()
            })
            .await
            .unwrap_err();

        assert!(matches!(err, AgentWrapperError::InvalidRequest { .. }));
    }

    #[tokio::test]
    async fn external_sandbox_true_rejects_backend_exec_policy_keys_pre_spawn() {
        let backend = test_backend();

        let err = backend
            .run(AgentWrapperRunRequest {
                prompt: "hello".to_string(),
                extensions: [
                    (
                        "agent_api.exec.external_sandbox.v1".to_string(),
                        Value::Bool(true),
                    ),
                    (
                        "backend.codex.exec.sandbox_mode".to_string(),
                        Value::String("danger-full-access".to_string()),
                    ),
                ]
                .into_iter()
                .collect(),
                ..Default::default()
            })
            .await
            .unwrap_err();

        assert!(matches!(err, AgentWrapperError::InvalidRequest { .. }));
    }
}

#[cfg(feature = "claude_code")]
mod claude_code {
    use std::path::PathBuf;

    use agent_api::{
        backends::claude_code::{ClaudeCodeBackend, ClaudeCodeBackendConfig},
        AgentWrapperBackend, AgentWrapperError, AgentWrapperRunRequest,
    };
    use serde_json::Value;

    fn test_backend() -> ClaudeCodeBackend {
        ClaudeCodeBackend::new(ClaudeCodeBackendConfig {
            binary: Some(PathBuf::from("definitely-not-a-real-claude-binary")),
            allow_external_sandbox_exec: true,
            ..Default::default()
        })
    }

    #[tokio::test]
    async fn external_sandbox_type_mismatch_is_rejected_pre_spawn() {
        let backend = test_backend();

        let err = backend
            .run(AgentWrapperRunRequest {
                prompt: "hello".to_string(),
                extensions: [(
                    "agent_api.exec.external_sandbox.v1".to_string(),
                    Value::String("not-bool".to_string()),
                )]
                .into_iter()
                .collect(),
                ..Default::default()
            })
            .await
            .unwrap_err();

        let message = match err {
            AgentWrapperError::InvalidRequest { message } => message,
            other => panic!("expected InvalidRequest, got: {other:?}"),
        };
        assert!(
            message.contains("agent_api.exec.external_sandbox.v1"),
            "expected error message to mention the key (got: {message:?})"
        );
        assert!(
            message.to_ascii_lowercase().contains("boolean"),
            "expected error message to mention boolean (got: {message:?})"
        );
    }

    #[tokio::test]
    async fn external_sandbox_true_rejects_non_interactive_false_pre_spawn() {
        let backend = test_backend();

        let err = backend
            .run(AgentWrapperRunRequest {
                prompt: "hello".to_string(),
                extensions: [
                    (
                        "agent_api.exec.external_sandbox.v1".to_string(),
                        Value::Bool(true),
                    ),
                    (
                        "agent_api.exec.non_interactive".to_string(),
                        Value::Bool(false),
                    ),
                ]
                .into_iter()
                .collect(),
                ..Default::default()
            })
            .await
            .unwrap_err();

        assert!(matches!(err, AgentWrapperError::InvalidRequest { .. }));
    }
}
