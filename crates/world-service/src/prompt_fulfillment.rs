use std::path::PathBuf;

use agent_api::{AgentWrapperError, AgentWrapperRunHandle, AgentWrapperRunRequest};
use anyhow::Result;
use substrate_gateway::adapter_runtime::{GatewayAdapterBackendKind, GatewayAdapterRuntime};
use transport_api_types::MemberRuntimeBackendKindV1;

mod native_terminal;
pub(crate) use native_terminal::retain_native_completion;
pub(crate) use native_terminal::{NativeSettlementOutcome, NativeTerminalObservation};

pub(crate) struct PromptFulfillmentRunControl {
    pub(crate) handle: AgentWrapperRunHandle,
    pub(crate) cancel: agent_api::AgentWrapperCancelHandle,
    pub(crate) terminal: NativeTerminalObservation,
}

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
    ) -> Result<PromptFulfillmentRunControl, AgentWrapperError> {
        let control = self.runtime.run_control(request).await?;
        let mut handle = control.handle;
        let terminate = control.cancel.clone();
        let (completion, terminal) = match control.cancel.process_settlement() {
            Some(processes) => native_terminal::retain_native_completion_with_process_observation(
                handle.completion,
                control.cancel.settlement(),
                Some(processes),
                (),
                move || terminate.cancel(),
                None,
                control.cancel.disposal_requested(),
            ),
            None => retain_native_completion(
                handle.completion,
                control.cancel.settlement(),
                (),
                move || terminate.cancel(),
                None,
                control.cancel.disposal_requested(),
            ),
        };
        handle.completion = completion;
        Ok(PromptFulfillmentRunControl {
            handle,
            cancel: control.cancel,
            terminal,
        })
    }
}

/// Dispose consumer events before awaiting DR-0012 completion. This observation
/// is not successful cleanup when the actual SDK acknowledgment is absent/failed.
pub(crate) async fn cancel_and_join(
    handle: AgentWrapperRunHandle,
    cancel: agent_api::AgentWrapperCancelHandle,
    terminal: NativeTerminalObservation,
) -> (
    Result<agent_api::AgentWrapperCompletion, AgentWrapperError>,
    std::sync::Arc<native_terminal::NativeTerminalOutcome>,
) {
    terminal.request_route_disposal();
    cancel.cancel();
    drop(handle.events);
    let raw = handle.completion.await;
    (raw, terminal.wait().await)
}

pub(crate) async fn join_failed_control(
    error: anyhow::Error,
    handle: AgentWrapperRunHandle,
    cancel: agent_api::AgentWrapperCancelHandle,
    terminal: NativeTerminalObservation,
) -> anyhow::Error {
    let (raw, observed) = cancel_and_join(handle, cancel, terminal).await;
    let error = match raw {
        Err(AgentWrapperError::Backend { message }) if message == "cancelled" => error,
        Err(backend) => error.context(format!("enclosing backend/cleanup outcome: {backend}")),
        Ok(_) => error,
    };
    match observed.require_checked() {
        Ok(()) => error,
        Err(cleanup) => error.context(format!(
            "submission_outcome_unknown: native acknowledgment unavailable: {cleanup}"
        )),
    }
}
