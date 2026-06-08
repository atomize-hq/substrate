use anyhow::Result;

use crate::execution::agent_runtime::host_inbox::{HostInboxMaterializationState, HostInboxRecord};
use crate::execution::agent_runtime::AgentRuntimeStateStore;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum HostInboxMaterializationOutcome {
    Materialized,
    AlreadyMaterialized,
    Pending,
    FailedClosed,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct HostInboxMaterializationExecution {
    pub record_id: String,
    pub orchestration_session_id: Option<String>,
    pub local_host_id: String,
    pub target_host_id: Option<String>,
    pub obligation_id: Option<String>,
    pub outcome: HostInboxMaterializationOutcome,
    pub reason: String,
}

pub(crate) fn materialize_pending_host_inbox_records_for_local_host(
    store: &AgentRuntimeStateStore,
    local_host_id: &str,
) -> Result<Vec<HostInboxMaterializationExecution>> {
    let mut pending_records = Vec::new();
    for record_id in store.list_host_inbox_record_ids()? {
        let before = match store.load_host_inbox_record(&record_id) {
            Ok(record) => record,
            Err(_) => None,
        };
        match before {
            Some(record)
                if record.materialization_state == HostInboxMaterializationState::Pending =>
            {
                pending_records.push((Some(record.created_at), record.record_id));
            }
            Some(_) => continue,
            None => pending_records.push((None, record_id)),
        }
    }
    pending_records.sort_by(|left, right| left.0.cmp(&right.0).then(left.1.cmp(&right.1)));

    let mut executions = Vec::new();
    for (_, record_id) in pending_records {
        executions.push(materialize_host_inbox_record_for_local_host(
            store,
            &record_id,
            local_host_id,
        )?);
    }
    Ok(executions)
}

pub(crate) fn materialize_host_inbox_record_for_local_host(
    store: &AgentRuntimeStateStore,
    record_id: &str,
    local_host_id: &str,
) -> Result<HostInboxMaterializationExecution> {
    let before = match store.load_host_inbox_record(record_id) {
        Ok(record) => record,
        Err(_) => None,
    };
    let after = store.materialize_host_inbox_record_for_local_host(record_id, local_host_id)?;
    Ok(build_host_inbox_materialization_execution(
        before.as_ref(),
        &after,
        local_host_id,
    ))
}

fn build_host_inbox_materialization_execution(
    before: Option<&HostInboxRecord>,
    after: &HostInboxRecord,
    local_host_id: &str,
) -> HostInboxMaterializationExecution {
    let outcome = match (
        before.map(|record| record.materialization_state),
        after.materialization_state,
    ) {
        (
            Some(HostInboxMaterializationState::Materialized),
            HostInboxMaterializationState::Materialized,
        ) => HostInboxMaterializationOutcome::AlreadyMaterialized,
        (_, HostInboxMaterializationState::Materialized) => {
            HostInboxMaterializationOutcome::Materialized
        }
        (_, HostInboxMaterializationState::FailedClosed) => {
            HostInboxMaterializationOutcome::FailedClosed
        }
        _ => HostInboxMaterializationOutcome::Pending,
    };
    let reason = match outcome {
        HostInboxMaterializationOutcome::Materialized => {
            "materialized_exact_local_obligation".to_string()
        }
        HostInboxMaterializationOutcome::AlreadyMaterialized => {
            "already_materialized_exact_local_obligation".to_string()
        }
        HostInboxMaterializationOutcome::Pending => {
            "pending_local_materialization_retry".to_string()
        }
        HostInboxMaterializationOutcome::FailedClosed => after
            .failed_closed_reason
            .clone()
            .unwrap_or_else(|| "host inbox record failed closed without reason".to_string()),
    };

    HostInboxMaterializationExecution {
        record_id: after.record_id.clone(),
        orchestration_session_id: (!after.orchestration_session_id.is_empty())
            .then(|| after.orchestration_session_id.clone()),
        local_host_id: local_host_id.to_string(),
        target_host_id: (!after.target_host_id.is_empty()).then(|| after.target_host_id.clone()),
        obligation_id: after.materialized_obligation_id.clone(),
        outcome,
        reason,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::execution::agent_runtime::auto_attach::select_attach_candidate;
    use crate::execution::agent_runtime::mapping::AgentRuntimeBackendKind;
    use crate::execution::agent_runtime::obligation_ledger::OrchestrationObligationKind;
    use crate::execution::agent_runtime::orchestration_session::{
        HostAttachContract, OrchestrationSessionRecord, OrchestrationSessionState,
    };
    use crate::execution::agent_runtime::validator::RuntimeSelectionDescriptor;
    use crate::execution::agent_runtime::{
        AgentRuntimeParticipantRecord, AgentRuntimeSessionState, PURE_AGENT_PROTOCOL,
    };
    use crate::execution::config_model::AgentExecutionScope;
    use serial_test::serial;
    use std::path::PathBuf;
    use tempfile::tempdir;

    fn with_store(test: impl FnOnce(&AgentRuntimeStateStore)) {
        let temp = tempdir().expect("tempdir");
        std::env::set_var("SUBSTRATE_HOME", temp.path());
        let store = AgentRuntimeStateStore::new().expect("state store");
        test(&store);
        std::env::remove_var("SUBSTRATE_HOME");
    }

    fn persist_session(store: &AgentRuntimeStateStore, orchestration_session_id: &str) {
        let descriptor = RuntimeSelectionDescriptor {
            agent_id: "codex".to_string(),
            backend_id: "cli:codex".to_string(),
            backend_kind: AgentRuntimeBackendKind::Codex,
            protocol: PURE_AGENT_PROTOCOL.to_string(),
            execution_scope: AgentExecutionScope::Host,
            binary_path: PathBuf::from("/bin/sh"),
        };
        let participant = detached_orchestrator_participant(&descriptor, orchestration_session_id);
        let mut session = OrchestrationSessionRecord::new(
            orchestration_session_id.to_string(),
            format!("trace_{orchestration_session_id}"),
            "/workspace".to_string(),
            &participant,
            HostAttachContract::from_manifest_for_test(&participant),
        );
        session.transition_state(OrchestrationSessionState::Active);
        session.bind_active_session_handle(participant.handle.participant_id.clone());
        session.mark_parked_resumable("owner detached cleanly");
        store
            .persist_participant(&participant)
            .expect("persist detached orchestrator");
        store
            .persist_orchestration_session(&session)
            .expect("persist authoritative session");
    }

    fn detached_orchestrator_participant(
        descriptor: &RuntimeSelectionDescriptor,
        orchestration_session_id: &str,
    ) -> AgentRuntimeParticipantRecord {
        let participant_id = format!("orch_{orchestration_session_id}");
        let mut participant = AgentRuntimeParticipantRecord::new_orchestrator_participant(
            descriptor,
            orchestration_session_id.to_string(),
            participant_id.clone(),
            format!("lease_{participant_id}"),
        )
        .expect("orchestrator participant");
        participant.transition_state(AgentRuntimeSessionState::Ready);
        participant.set_uaa_session_id(format!("uaa-{orchestration_session_id}"));
        participant.mark_client_detached("owner detached cleanly");
        participant.touch_heartbeat();
        participant
    }

    fn pending_record(
        orchestration_session_id: &str,
        record_id: &str,
        target_host_id: &str,
    ) -> HostInboxRecord {
        HostInboxRecord::new(
            orchestration_session_id.to_string(),
            record_id.to_string(),
            OrchestrationObligationKind::ApprovalRequired,
            "approval needed",
            target_host_id.to_string(),
            "router".to_string(),
            format!("ingress_{record_id}"),
        )
    }

    #[test]
    #[serial]
    fn host_inbox_materialization_entrypoint_discovers_pending_local_records_before_router() {
        with_store(|store| {
            persist_session(store, "sess_host_inbox_materialize");
            store
                .persist_host_inbox_record(&pending_record(
                    "sess_host_inbox_materialize",
                    "host_record_local",
                    "host-local",
                ))
                .expect("persist pending host inbox record");

            let executions =
                materialize_pending_host_inbox_records_for_local_host(store, "host-local")
                    .expect("materialize pending local host inbox records");

            assert_eq!(executions.len(), 1);
            assert_eq!(
                executions[0].outcome,
                HostInboxMaterializationOutcome::Materialized
            );
            assert_eq!(
                executions[0].orchestration_session_id.as_deref(),
                Some("sess_host_inbox_materialize")
            );
            assert_eq!(
                executions[0].obligation_id.as_deref(),
                Some("host_inbox_host_record_local")
            );

            let candidates = store
                .list_router_auto_attach_candidate_session_ids()
                .expect("list router auto-attach candidates");
            assert_eq!(candidates, vec!["sess_host_inbox_materialize".to_string()]);
        });
    }

    #[test]
    #[serial]
    fn host_inbox_materialization_entrypoint_emits_idempotent_and_fail_closed_outcomes() {
        with_store(|store| {
            persist_session(store, "sess_host_inbox_repeat");
            store
                .persist_host_inbox_record(&pending_record(
                    "sess_host_inbox_repeat",
                    "host_record_repeat",
                    "host-local",
                ))
                .expect("persist repeatable host inbox record");

            let first = materialize_host_inbox_record_for_local_host(
                store,
                "host_record_repeat",
                "host-local",
            )
            .expect("first materialization");
            let second = materialize_host_inbox_record_for_local_host(
                store,
                "host_record_repeat",
                "host-local",
            )
            .expect("idempotent rerun");

            assert_eq!(first.outcome, HostInboxMaterializationOutcome::Materialized);
            assert_eq!(
                second.outcome,
                HostInboxMaterializationOutcome::AlreadyMaterialized
            );
            assert_eq!(second.reason, "already_materialized_exact_local_obligation");
            assert_eq!(second.local_host_id, "host-local");
            assert_eq!(
                second.obligation_id.as_deref(),
                Some("host_inbox_host_record_repeat")
            );

            store
                .persist_host_inbox_record(&pending_record(
                    "sess_host_inbox_repeat",
                    "host_record_wrong_host",
                    "host-foreign",
                ))
                .expect("persist wrong-host host inbox record");
            let failed = materialize_host_inbox_record_for_local_host(
                store,
                "host_record_wrong_host",
                "host-local",
            )
            .expect("wrong-host materialization should fail closed durably");

            assert_eq!(
                failed.outcome,
                HostInboxMaterializationOutcome::FailedClosed
            );
            assert!(failed.reason.contains("wrong_target_host"));
            assert_eq!(failed.target_host_id.as_deref(), Some("host-foreign"));
            assert!(failed.obligation_id.is_none());
        });
    }

    #[test]
    #[serial]
    fn host_inbox_materialization_entrypoint_keeps_router_consuming_obligations_only() {
        with_store(|store| {
            persist_session(store, "sess_host_inbox_router");
            store
                .persist_host_inbox_record(&pending_record(
                    "sess_host_inbox_router",
                    "host_record_router",
                    "host-local",
                ))
                .expect("persist pending host inbox record");

            let before = store
                .list_router_auto_attach_candidate_session_ids()
                .expect("list router candidates before materialization");
            assert!(
                before.is_empty(),
                "router candidate discovery must ignore pending host inbox records"
            );

            materialize_pending_host_inbox_records_for_local_host(store, "host-local")
                .expect("materialize pending local host records");

            let after = store
                .list_router_auto_attach_candidate_session_ids()
                .expect("list router candidates after materialization");
            assert_eq!(after, vec!["sess_host_inbox_router".to_string()]);

            let obligations = store
                .list_obligations("sess_host_inbox_router")
                .expect("list materialized obligations");
            assert_eq!(obligations.len(), 1);
            assert_eq!(
                obligations[0].obligation_id,
                "host_inbox_host_record_router"
            );
        });
    }

    #[test]
    #[serial]
    fn host_inbox_materialization_entrypoint_preserves_durable_order_for_same_kind_router_work() {
        with_store(|store| {
            persist_session(store, "sess_host_inbox_order");

            let durable_first =
                pending_record("sess_host_inbox_order", "host_record_z_first", "host-local");
            let mut durable_second = pending_record(
                "sess_host_inbox_order",
                "host_record_a_second",
                "host-local",
            );
            durable_second.created_at = durable_first.created_at + chrono::Duration::seconds(1);
            durable_second.updated_at = durable_second.created_at;

            store
                .persist_host_inbox_record(&durable_second)
                .expect("persist later durable host inbox record");
            store
                .persist_host_inbox_record(&durable_first)
                .expect("persist earlier durable host inbox record");

            let executions =
                materialize_pending_host_inbox_records_for_local_host(store, "host-local")
                    .expect("materialize pending host inbox records in durable order");

            assert_eq!(
                executions
                    .iter()
                    .map(|execution| execution.record_id.as_str())
                    .collect::<Vec<_>>(),
                vec!["host_record_z_first", "host_record_a_second"]
            );

            let obligations = store
                .list_obligations("sess_host_inbox_order")
                .expect("list materialized obligations");
            let selected = select_attach_candidate(&obligations)
                .expect("same-kind router candidate should exist after materialization");
            assert_eq!(selected.obligation_id, "host_inbox_host_record_z_first");
        });
    }
}
