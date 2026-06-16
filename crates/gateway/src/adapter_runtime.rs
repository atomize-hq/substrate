use std::{path::PathBuf, sync::Arc};

use agent_api::{
    backends::{
        claude_code::{ClaudeCodeBackend, ClaudeCodeBackendConfig},
        codex::{CodexBackend, CodexBackendConfig},
    },
    AgentWrapperError, AgentWrapperGateway, AgentWrapperKind, AgentWrapperRunControl,
    AgentWrapperRunRequest,
};
use anyhow::{anyhow, Result};

const AGENT_API_EXEC_EXTERNAL_SANDBOX_V1: &str = "agent_api.exec.external_sandbox.v1";
const BACKEND_CODEX_EXEC_PREFIX: &str = "backend.codex.exec.";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GatewayAdapterBackendKind {
    Codex,
    ClaudeCode,
}

impl GatewayAdapterBackendKind {
    fn agent_kind(self) -> Result<AgentWrapperKind, AgentWrapperError> {
        let value = match self {
            Self::Codex => "codex",
            Self::ClaudeCode => "claude_code",
        };
        AgentWrapperKind::new(value)
    }
}

pub struct GatewayAdapterRuntime {
    gateway: AgentWrapperGateway,
    agent_kind: AgentWrapperKind,
}

impl GatewayAdapterRuntime {
    pub fn for_backend(
        backend_kind: GatewayAdapterBackendKind,
        binary_path: PathBuf,
    ) -> Result<Self> {
        let mut gateway = AgentWrapperGateway::new();
        let binary = Some(binary_path);

        let agent_kind = match backend_kind {
            GatewayAdapterBackendKind::Codex => {
                gateway
                    .register(Arc::new(CodexBackend::new(CodexBackendConfig {
                        allow_external_sandbox_exec: true,
                        binary,
                        ..Default::default()
                    })))
                    .map_err(map_wrapper_error)?;
                backend_kind.agent_kind().map_err(map_wrapper_error)?
            }
            GatewayAdapterBackendKind::ClaudeCode => {
                gateway
                    .register(Arc::new(ClaudeCodeBackend::new(ClaudeCodeBackendConfig {
                        binary,
                        ..Default::default()
                    })))
                    .map_err(map_wrapper_error)?;
                backend_kind.agent_kind().map_err(map_wrapper_error)?
            }
        };

        Ok(Self {
            gateway,
            agent_kind,
        })
    }

    pub async fn run_control(
        &self,
        mut request: AgentWrapperRunRequest,
    ) -> Result<AgentWrapperRunControl, AgentWrapperError> {
        maybe_enable_codex_external_sandbox(&self.agent_kind, &mut request);
        self.gateway.run_control(&self.agent_kind, request).await
    }
}

fn maybe_enable_codex_external_sandbox(
    agent_kind: &AgentWrapperKind,
    request: &mut AgentWrapperRunRequest,
) {
    if agent_kind.as_str() != "codex" {
        return;
    }

    if request
        .extensions
        .contains_key(AGENT_API_EXEC_EXTERNAL_SANDBOX_V1)
    {
        return;
    }

    if request
        .extensions
        .keys()
        .any(|key| key.starts_with(BACKEND_CODEX_EXEC_PREFIX))
    {
        return;
    }

    request.extensions.insert(
        AGENT_API_EXEC_EXTERNAL_SANDBOX_V1.to_string(),
        serde_json::Value::Bool(true),
    );
}

fn map_wrapper_error(err: impl std::fmt::Display) -> anyhow::Error {
    anyhow!("substrate-gateway adapter runtime: {err}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn codex_requests_default_to_external_sandbox_bypass() {
        let agent_kind = AgentWrapperKind::new("codex").expect("codex agent kind");
        let mut request = AgentWrapperRunRequest::default();

        maybe_enable_codex_external_sandbox(&agent_kind, &mut request);

        assert_eq!(
            request.extensions.get(AGENT_API_EXEC_EXTERNAL_SANDBOX_V1),
            Some(&serde_json::Value::Bool(true))
        );
    }

    #[test]
    fn codex_requests_preserve_explicit_external_sandbox_choice() {
        let agent_kind = AgentWrapperKind::new("codex").expect("codex agent kind");
        let mut request = AgentWrapperRunRequest::default();
        request.extensions.insert(
            AGENT_API_EXEC_EXTERNAL_SANDBOX_V1.to_string(),
            serde_json::Value::Bool(false),
        );

        maybe_enable_codex_external_sandbox(&agent_kind, &mut request);

        assert_eq!(
            request.extensions.get(AGENT_API_EXEC_EXTERNAL_SANDBOX_V1),
            Some(&serde_json::Value::Bool(false))
        );
    }

    #[test]
    fn codex_requests_with_explicit_exec_overrides_do_not_force_bypass() {
        let agent_kind = AgentWrapperKind::new("codex").expect("codex agent kind");
        let mut request = AgentWrapperRunRequest::default();
        request.extensions.insert(
            "backend.codex.exec.sandbox_mode".to_string(),
            serde_json::Value::String("workspace-write".to_string()),
        );

        maybe_enable_codex_external_sandbox(&agent_kind, &mut request);

        assert!(!request
            .extensions
            .contains_key(AGENT_API_EXEC_EXTERNAL_SANDBOX_V1));
    }

    #[test]
    fn non_codex_requests_are_left_unchanged() {
        let agent_kind = AgentWrapperKind::new("claude_code").expect("claude_code agent kind");
        let mut request = AgentWrapperRunRequest::default();

        maybe_enable_codex_external_sandbox(&agent_kind, &mut request);

        assert!(request.extensions.is_empty());
    }
}
