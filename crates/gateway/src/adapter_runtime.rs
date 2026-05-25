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
        request: AgentWrapperRunRequest,
    ) -> Result<AgentWrapperRunControl, AgentWrapperError> {
        self.gateway.run_control(&self.agent_kind, request).await
    }
}

fn map_wrapper_error(err: impl std::fmt::Display) -> anyhow::Error {
    anyhow!("substrate-gateway adapter runtime: {err}")
}
