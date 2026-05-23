use std::path::PathBuf;

use agent_api::{AgentWrapperError, AgentWrapperRunControl, AgentWrapperRunRequest};
use anyhow::Result;
use substrate_gateway::adapter_runtime::{GatewayAdapterBackendKind, GatewayAdapterRuntime};
use transport_api_types::MemberRuntimeBackendKindV1;

pub(crate) struct PromptFulfillmentBridge {
    runtime: GatewayAdapterRuntime,
}

impl PromptFulfillmentBridge {
    pub(crate) fn for_member_backend(
        backend_kind: &MemberRuntimeBackendKindV1,
        binary_path: PathBuf,
    ) -> Result<Self> {
        let backend_kind = match backend_kind {
            MemberRuntimeBackendKindV1::Codex => GatewayAdapterBackendKind::Codex,
            MemberRuntimeBackendKindV1::ClaudeCode => GatewayAdapterBackendKind::ClaudeCode,
        };
        let runtime = GatewayAdapterRuntime::for_backend(backend_kind, binary_path)?;
        Ok(Self { runtime })
    }

    pub(crate) async fn run_control(
        &self,
        request: AgentWrapperRunRequest,
    ) -> Result<AgentWrapperRunControl, AgentWrapperError> {
        self.runtime.run_control(request).await
    }
}
