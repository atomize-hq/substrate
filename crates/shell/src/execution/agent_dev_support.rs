use anyhow::Result;

use crate::execution::agent_runtime::state_store::{
    AgentRuntimeStateStore, DurableInboxItemKind, DurableInboxItemRecord,
};

/// Persists one pending runtime-alert inbox item through the authoritative state-store path.
///
/// This is reserved for test and validation support. It must not be surfaced through the public
/// `substrate agent ...` grammar.
pub fn persist_runtime_alert_for_dev_support(
    orchestration_session_id: &str,
    item_id: &str,
    message: Option<String>,
) -> Result<()> {
    let store = AgentRuntimeStateStore::new()?;
    let item = DurableInboxItemRecord::new(
        orchestration_session_id,
        item_id,
        DurableInboxItemKind::RuntimeAlert,
        message,
    );
    store.persist_inbox_item(&item)
}
