use anyhow::Result;

use crate::execution::agent_runtime::{
    state_store::AgentRuntimeStateStore, OrchestrationObligationAttachState,
    OrchestrationObligationKind, OrchestrationObligationRecord,
};

/// Persists one pending runtime-alert obligation through the authoritative state-store path.
///
/// This is reserved for test and validation support. It must not be surfaced through the public
/// `substrate agent ...` grammar.
pub fn persist_runtime_alert_for_dev_support(
    orchestration_session_id: &str,
    item_id: &str,
    message: Option<String>,
) -> Result<()> {
    let store = AgentRuntimeStateStore::new()?;
    let mut obligation = OrchestrationObligationRecord::new(
        orchestration_session_id,
        item_id,
        OrchestrationObligationKind::RuntimeAlert,
        message.unwrap_or_else(|| format!("runtime alert for {item_id}")),
    );
    obligation.attention_required = true;
    obligation.attach_state = OrchestrationObligationAttachState::Eligible;
    store.persist_obligation(&obligation)
}
