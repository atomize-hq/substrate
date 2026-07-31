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

fn discover_pending_host_inbox_materialization_candidates(
    store: &AgentRuntimeStateStore,
) -> Result<Vec<(Option<chrono::DateTime<chrono::Utc>>, String)>> {
    let mut pending_records = Vec::new();
    for record_id in store.list_host_inbox_record_ids()? {
        let before = store.load_host_inbox_record(&record_id).unwrap_or_default();
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
    Ok(pending_records)
}

fn build_host_inbox_materialization_retry_execution(
    record_id: &str,
    local_host_id: &str,
    err: &anyhow::Error,
) -> HostInboxMaterializationExecution {
    HostInboxMaterializationExecution {
        record_id: record_id.to_string(),
        orchestration_session_id: None,
        local_host_id: local_host_id.to_string(),
        target_host_id: None,
        obligation_id: None,
        outcome: HostInboxMaterializationOutcome::Pending,
        reason: format!("pending_local_materialization_retry: {err:#}"),
    }
}

fn materialize_discovered_host_inbox_records_for_local_host(
    store: &AgentRuntimeStateStore,
    candidates: Vec<(Option<chrono::DateTime<chrono::Utc>>, String)>,
    local_host_id: &str,
) -> Vec<HostInboxMaterializationExecution> {
    let mut executions = Vec::new();
    for (_, record_id) in candidates {
        match materialize_host_inbox_record_for_local_host(store, &record_id, local_host_id) {
            Ok(execution) => executions.push(execution),
            Err(err) => executions.push(build_host_inbox_materialization_retry_execution(
                &record_id,
                local_host_id,
                &err,
            )),
        }
    }
    executions
}

pub(crate) fn materialize_pending_host_inbox_records_for_local_host(
    store: &AgentRuntimeStateStore,
    local_host_id: &str,
) -> Result<Vec<HostInboxMaterializationExecution>> {
    let candidates = discover_pending_host_inbox_materialization_candidates(store)?;
    let mut executions =
        materialize_discovered_host_inbox_records_for_local_host(store, candidates, local_host_id);
    // Invalid path stems never become router work; we preserve them as
    // durable failed-closed host-inbox outcomes instead.
    for path in store.list_invalid_host_inbox_artifact_paths()? {
        let after = store.record_invalid_host_inbox_artifact_failure(&path)?;
        executions.push(build_host_inbox_materialization_execution(
            None,
            &after,
            local_host_id,
        ));
    }
    Ok(executions)
}

pub(crate) fn materialize_host_inbox_record_for_local_host(
    store: &AgentRuntimeStateStore,
    record_id: &str,
    local_host_id: &str,
) -> Result<HostInboxMaterializationExecution> {
    let before = store.load_host_inbox_record(record_id).unwrap_or_default();
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
    use std::fs;
    use std::path::PathBuf;
    use tempfile::tempdir_in;

    fn with_store(test: impl FnOnce(&AgentRuntimeStateStore)) {
        let authority_env = crate::execution::AuthorityEnvTestGuard::preserve();
        let safe_parent = std::env::var_os("XDG_RUNTIME_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|| {
                PathBuf::from(std::env::var_os("HOME").expect("tests require HOME")).join(".cache")
            });
        fs::create_dir_all(&safe_parent).expect("create host inbox authority parent");
        let temp = tempdir_in(safe_parent).expect("secure host inbox tempdir");
        let substrate_home = temp.path().join("home");
        fs::create_dir_all(&substrate_home).expect("create host inbox substrate home");
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt as _;

            fs::set_permissions(temp.path(), fs::Permissions::from_mode(0o700))
                .expect("secure host inbox tempdir mode");
            fs::set_permissions(&substrate_home, fs::Permissions::from_mode(0o700))
                .expect("secure host inbox substrate home mode");
        }
        authority_env.install_home(&substrate_home);
        let store = AgentRuntimeStateStore::new().expect("state store");
        test(&store);
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
            let discovered = discover_pending_host_inbox_materialization_candidates(store)
                .expect("discover pending host inbox candidates before concurrent rerun");

            let first = materialize_host_inbox_record_for_local_host(
                store,
                "host_record_repeat",
                "host-local",
            )
            .expect("first materialization");
            let second = materialize_discovered_host_inbox_records_for_local_host(
                store,
                discovered,
                "host-local",
            );

            assert_eq!(first.outcome, HostInboxMaterializationOutcome::Materialized);
            assert_eq!(
                second[0].outcome,
                HostInboxMaterializationOutcome::AlreadyMaterialized
            );
            assert_eq!(
                second[0].reason,
                "already_materialized_exact_local_obligation"
            );
            assert_eq!(second[0].local_host_id, "host-local");
            assert_eq!(
                second[0].obligation_id.as_deref(),
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

    #[test]
    #[serial]
    fn host_inbox_materialization_preserves_durable_order_across_retryable_materialization() {
        with_store(|store| {
            let older = pending_record(
                "sess_host_inbox_retry_order",
                "host_record_older",
                "host-local",
            );
            store
                .persist_host_inbox_record(&older)
                .expect("persist older host inbox record before session exists");

            let first_pass =
                materialize_pending_host_inbox_records_for_local_host(store, "host-local")
                    .expect("older record should remain pending without session");
            assert_eq!(first_pass.len(), 1);
            assert_eq!(
                first_pass[0].outcome,
                HostInboxMaterializationOutcome::Pending
            );

            persist_session(store, "sess_host_inbox_retry_order");
            let mut newer = pending_record(
                "sess_host_inbox_retry_order",
                "host_record_newer",
                "host-local",
            );
            newer.created_at = older.created_at + chrono::Duration::seconds(1);
            newer.updated_at = newer.created_at;
            store
                .persist_host_inbox_record(&newer)
                .expect("persist newer host inbox record after session exists");

            let executions =
                materialize_pending_host_inbox_records_for_local_host(store, "host-local")
                    .expect("retryable materialization should preserve durable order");
            assert_eq!(
                executions
                    .iter()
                    .map(|execution| execution.outcome)
                    .collect::<Vec<_>>(),
                vec![
                    HostInboxMaterializationOutcome::Materialized,
                    HostInboxMaterializationOutcome::Materialized
                ]
            );

            let obligations = store
                .list_obligations("sess_host_inbox_retry_order")
                .expect("list obligations after retryable materialization");
            let selected = select_attach_candidate(&obligations)
                .expect("same-kind router candidate should exist after retryable materialization");
            assert_eq!(selected.obligation_id, "host_inbox_host_record_older");
            assert_eq!(obligations[0].created_at, older.created_at);
            assert_eq!(obligations[1].created_at, newer.created_at);
        });
    }

    #[cfg(unix)]
    #[test]
    #[serial]
    fn host_inbox_materialization_continues_other_pending_records_after_unreadable_artifact() {
        use std::os::unix::fs::PermissionsExt;

        with_store(|store| {
            persist_session(store, "sess_host_inbox_continue_after_error");
            let unreadable = pending_record(
                "sess_host_inbox_continue_after_error",
                "host_record_unreadable",
                "host-local",
            );
            let readable = pending_record(
                "sess_host_inbox_continue_after_error",
                "host_record_readable",
                "host-local",
            );
            store
                .persist_host_inbox_record(&unreadable)
                .expect("persist unreadable host inbox record");
            store
                .persist_host_inbox_record(&readable)
                .expect("persist readable host inbox record");

            let unreadable_path = store
                .host_inbox_record_path("host_record_unreadable")
                .expect("unreadable record path");
            let mut perms = fs::metadata(&unreadable_path)
                .expect("unreadable record metadata")
                .permissions();
            let original_mode = perms.mode();
            perms.set_mode(0o000);
            fs::set_permissions(&unreadable_path, perms)
                .expect("restrict unreadable record permissions");

            let executions =
                materialize_pending_host_inbox_records_for_local_host(store, "host-local")
                    .expect("unreadable artifact should not abort other pending records");

            let mut restored = fs::metadata(&unreadable_path)
                .expect("restricted unreadable record metadata")
                .permissions();
            restored.set_mode(original_mode);
            fs::set_permissions(&unreadable_path, restored)
                .expect("restore unreadable record permissions");

            assert_eq!(executions.len(), 2);
            assert!(
                executions.iter().any(|execution| {
                    execution.record_id == "host_record_unreadable"
                        && execution.outcome == HostInboxMaterializationOutcome::Pending
                        && execution
                            .reason
                            .contains("pending_local_materialization_retry")
                }),
                "unreadable artifacts should become explanation-ready retry outcomes"
            );
            assert!(
                executions.iter().any(|execution| {
                    execution.record_id == "host_record_readable"
                        && execution.outcome == HostInboxMaterializationOutcome::Materialized
                }),
                "readable sibling records should still materialize in the same batch"
            );
        });
    }

    #[test]
    #[serial]
    fn host_inbox_materialization_entrypoint_fails_closed_malformed_path_stems_without_blocking_router_candidates(
    ) {
        with_store(|store| {
            persist_session(store, "sess_host_inbox_malformed_path");
            store
                .persist_obligation(
                    &crate::execution::agent_runtime::obligation_ledger::OrchestrationObligationRecord::new(
                        "sess_host_inbox_malformed_path",
                        "obl_follow_up",
                        OrchestrationObligationKind::FollowUpRequired,
                        "follow up needed",
                    ),
                )
                .expect("persist canonical obligation");
            let mut obligation = store
                .load_obligation("sess_host_inbox_malformed_path", "obl_follow_up")
                .expect("load canonical obligation")
                .expect("canonical obligation exists");
            obligation.attention_required = true;
            obligation.attach_state =
                crate::execution::agent_runtime::obligation_ledger::OrchestrationObligationAttachState::Eligible;
            store
                .persist_obligation(&obligation)
                .expect("persist eligible canonical obligation");

            let malformed_path = store.host_inbox_dir().join("C:.json");
            fs::create_dir_all(store.host_inbox_dir()).expect("create host inbox dir");
            fs::write(&malformed_path, b"{}").expect("write malformed path-stem artifact");

            let executions =
                materialize_pending_host_inbox_records_for_local_host(store, "host-local")
                    .expect("malformed path stems must not abort host inbox pre-pass");
            assert_eq!(executions.len(), 1);
            assert_eq!(
                executions[0].outcome,
                HostInboxMaterializationOutcome::FailedClosed
            );
            assert!(
                executions[0]
                    .reason
                    .contains("invalid_host_inbox_artifact_path"),
                "malformed path-stem artifacts should produce an explanation-ready failed-closed outcome"
            );
            assert!(
                executions[0].obligation_id.is_none(),
                "malformed path-stem artifacts must never materialize router work directly"
            );

            let candidates = store
                .list_router_auto_attach_candidate_session_ids()
                .expect("list router candidates after malformed path-stem failure");
            assert_eq!(
                candidates,
                vec!["sess_host_inbox_malformed_path".to_string()]
            );

            let failed_closed_record = store
                .load_invalid_host_inbox_artifact_failure_record(&malformed_path)
                .expect("load persisted malformed artifact failure record")
                .expect("malformed path-stem artifacts should persist a durable failed-closed host inbox record");
            assert_eq!(
                failed_closed_record.materialization_state,
                HostInboxMaterializationState::FailedClosed
            );
            assert!(failed_closed_record
                .failed_closed_reason
                .as_deref()
                .is_some_and(|reason| reason.contains("invalid_host_inbox_artifact_path")));
        });
    }
}
