use std::sync::Arc;

use agent_api::AgentWrapperGateway;

use super::mapping::AgentRuntimeBackendKind;
use super::validator::RuntimeSelectionDescriptor;

pub(crate) fn build_gateway_for_descriptor(
    descriptor: &RuntimeSelectionDescriptor,
) -> anyhow::Result<AgentWrapperGateway> {
    let mut gateway = AgentWrapperGateway::new();

    match descriptor.backend_kind {
        AgentRuntimeBackendKind::Codex => {
            let backend = agent_api::backends::codex::CodexBackend::new(
                agent_api::backends::codex::CodexBackendConfig {
                    binary: Some(descriptor.binary_path.clone()),
                    ..Default::default()
                },
            );
            gateway.register(Arc::new(backend))?;
        }
        AgentRuntimeBackendKind::ClaudeCode => {
            let backend = agent_api::backends::claude_code::ClaudeCodeBackend::new(
                agent_api::backends::claude_code::ClaudeCodeBackendConfig {
                    binary: Some(descriptor.binary_path.clone()),
                    ..Default::default()
                },
            );
            gateway.register(Arc::new(backend))?;
        }
    }

    Ok(gateway)
}
