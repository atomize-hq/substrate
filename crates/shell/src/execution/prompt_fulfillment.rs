use agent_api::{AgentWrapperRunControl, AgentWrapperRunRequest};
use substrate_gateway::adapter_runtime::{GatewayAdapterBackendKind, GatewayAdapterRuntime};

use crate::execution::agent_runtime::{
    mapping::AgentRuntimeBackendKind, validator::RuntimeSelectionDescriptor,
};

pub(crate) struct PromptFulfillmentBridge {
    runtime: GatewayAdapterRuntime,
}

impl PromptFulfillmentBridge {
    pub(crate) fn for_descriptor(descriptor: &RuntimeSelectionDescriptor) -> anyhow::Result<Self> {
        let backend_kind = match descriptor.backend_kind {
            AgentRuntimeBackendKind::Codex => GatewayAdapterBackendKind::Codex,
            AgentRuntimeBackendKind::ClaudeCode => GatewayAdapterBackendKind::ClaudeCode,
        };
        let runtime =
            GatewayAdapterRuntime::for_backend(backend_kind, descriptor.binary_path.clone())?;

        Ok(Self { runtime })
    }

    pub(crate) async fn run_control(
        &self,
        request: AgentWrapperRunRequest,
    ) -> Result<AgentWrapperRunControl, agent_api::AgentWrapperError> {
        self.runtime.run_control(request).await
    }
}
