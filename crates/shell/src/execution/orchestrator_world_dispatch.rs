use anyhow::Result;

use crate::execution::agent_runtime::{
    AgentRuntimeParticipantRecord, AgentRuntimeStateStore, OrchestrationSessionRecord,
    ValidatedWorldDispatchRequestV1, WorldDispatchRequestV1,
};

#[allow(dead_code)]
#[derive(Clone, Debug)]
pub(crate) struct PreparedOrchestratorWorldDispatch {
    pub request: ValidatedWorldDispatchRequestV1,
    pub session: OrchestrationSessionRecord,
    pub caller_participant: AgentRuntimeParticipantRecord,
}

#[allow(dead_code)]
pub(crate) fn prepare_orchestrator_world_dispatch(
    store: &AgentRuntimeStateStore,
    request: WorldDispatchRequestV1,
) -> Result<PreparedOrchestratorWorldDispatch> {
    let request = request.validate()?;
    let authority = store.resolve_internal_world_dispatch_caller(
        &request.orchestration_session_id,
        &request.caller_participant_id,
    )?;

    Ok(PreparedOrchestratorWorldDispatch {
        request,
        session: authority.session,
        caller_participant: authority.caller_participant,
    })
}
