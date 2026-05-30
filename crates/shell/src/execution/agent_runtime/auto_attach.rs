use anyhow::Result;
use chrono::Utc;
use uuid::Uuid;

use super::control::{
    launch_hidden_owner_helper, HiddenOwnerHelperLaunchPlan, HiddenOwnerHelperLaunchReceipt,
    HiddenOwnerHelperParticipantPlan, HiddenOwnerHelperSessionPlan, OwnerHelperMode,
};
use super::dispatch_contract::{
    resolve_persisted_host_attach_contract, AttachLaunchKnobs, AttachModePreference,
    DispatchBaselineKind, DispatchCallerKind, DispatchCapabilityOverrideSet,
    DispatchRequestEnvelope, HostExecutionClientStart,
};
use super::obligation_ledger::{OrchestrationObligationKind, OrchestrationObligationRecord};
use super::state_store::AgentRuntimeStateStore;
use super::validator::{materialize_runtime_descriptor, RuntimeSelectionDescriptor};
use crate::execution::config_model::AgentExecutionScope;

pub(crate) const MANUAL_REATTACH_ATTACH_RESTORED_REASON: &str =
    "session_attach_restored_by_manual_reattach";
pub(crate) const ROUTER_AUTO_ATTACH_RESTORED_REASON: &str =
    "session_attach_restored_by_router_auto_attach";
const ROUTER_AUTO_ATTACH_UNSUPPORTED_REASON: &str =
    "unsupported_platform_or_posture: router-owned automatic attach is supported on Linux only in this slice";

#[allow(dead_code)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum SessionAutoAttachClaim {
    NoCandidate {
        reason: &'static str,
    },
    AlreadyClaimed {
        obligation_id: String,
    },
    Claimed {
        obligation_id: String,
        attach_claim_owner: String,
    },
}

#[allow(dead_code)]
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct SessionAutoAttachSettleResult {
    pub satisfied_obligation_ids: Vec<String>,
    pub superseded_obligation_ids: Vec<String>,
}

#[allow(dead_code)]
#[derive(Clone, Debug)]
pub(crate) enum SessionAutoAttachExecution {
    NoCandidate {
        reason: &'static str,
    },
    AlreadyClaimed {
        obligation_id: String,
    },
    FailedClosed {
        obligation_id: String,
        attach_claim_owner: String,
        reason: String,
    },
    Attached {
        obligation_id: String,
        attach_claim_owner: String,
        receipt: HiddenOwnerHelperLaunchReceipt,
    },
}

#[allow(dead_code)]
pub(crate) fn claimed_obligation_id(
    obligations: &[OrchestrationObligationRecord],
) -> Result<Option<&str>> {
    let mut claimed = obligations
        .iter()
        .filter(|obligation| obligation.is_auto_attach_claimed());
    let Some(first) = claimed.next() else {
        return Ok(None);
    };
    if let Some(second) = claimed.next() {
        anyhow::bail!(
            "session {} has multiple claimed auto-attach obligations ({} and {})",
            first.orchestration_session_id,
            first.obligation_id,
            second.obligation_id
        );
    }

    Ok(Some(first.obligation_id.as_str()))
}

#[allow(dead_code)]
pub(crate) fn select_attach_candidate(
    obligations: &[OrchestrationObligationRecord],
) -> Option<&OrchestrationObligationRecord> {
    let mut candidates = obligations
        .iter()
        .filter(|obligation| obligation.is_auto_attach_eligible())
        .collect::<Vec<_>>();
    candidates.sort_by(|left, right| {
        candidate_priority(left.kind)
            .cmp(&candidate_priority(right.kind))
            .then(left.created_at.cmp(&right.created_at))
            .then(left.obligation_id.cmp(&right.obligation_id))
    });
    candidates.into_iter().next()
}

fn candidate_priority(kind: OrchestrationObligationKind) -> u8 {
    match kind {
        OrchestrationObligationKind::ApprovalRequired => 0,
        OrchestrationObligationKind::Blocked => 1,
        OrchestrationObligationKind::ForkRequest => 2,
        OrchestrationObligationKind::FollowUpRequired => 3,
        _ => u8::MAX,
    }
}

#[allow(dead_code)]
pub(crate) fn execute_session_auto_attach(
    store: &AgentRuntimeStateStore,
    orchestration_session_id: &str,
    router_identity: &str,
    world: bool,
    no_world: bool,
) -> Result<SessionAutoAttachExecution> {
    let claim = store.claim_session_auto_attach(orchestration_session_id, router_identity)?;
    let (obligation_id, attach_claim_owner) = match claim {
        SessionAutoAttachClaim::NoCandidate { reason } => {
            return Ok(SessionAutoAttachExecution::NoCandidate { reason });
        }
        SessionAutoAttachClaim::AlreadyClaimed { obligation_id } => {
            return Ok(SessionAutoAttachExecution::AlreadyClaimed { obligation_id });
        }
        SessionAutoAttachClaim::Claimed {
            obligation_id,
            attach_claim_owner,
        } => (obligation_id, attach_claim_owner),
    };
    if let Err(err) = ensure_router_auto_attach_supported() {
        let reason = err.to_string();
        mark_attach_failed_closed(store, orchestration_session_id, &obligation_id, &reason)?;
        return Ok(SessionAutoAttachExecution::FailedClosed {
            obligation_id,
            attach_claim_owner,
            reason,
        });
    }

    let plan = match build_auto_attach_launch_plan(store, orchestration_session_id) {
        Ok(plan) => plan,
        Err(err) => {
            let reason = err.to_string();
            mark_attach_failed_closed(store, orchestration_session_id, &obligation_id, &reason)?;
            return Ok(SessionAutoAttachExecution::FailedClosed {
                obligation_id,
                attach_claim_owner,
                reason,
            });
        }
    };

    let receipt = match launch_hidden_owner_helper(&plan, world, no_world) {
        Ok(receipt) => receipt,
        Err(err) => {
            let reason = err.to_string();
            mark_attach_failed_closed(store, orchestration_session_id, &obligation_id, &reason)?;
            return Ok(SessionAutoAttachExecution::FailedClosed {
                obligation_id,
                attach_claim_owner,
                reason,
            });
        }
    };

    ensure_auto_attach_restored_session(store, orchestration_session_id, &receipt.participant_id)
        .inspect_err(|err| {
        let reason = err.to_string();
        let _ = mark_attach_failed_closed(store, orchestration_session_id, &obligation_id, &reason);
    })?;
    store.settle_session_auto_attach_after_attach_restored(
        orchestration_session_id,
        ROUTER_AUTO_ATTACH_RESTORED_REASON,
    )?;

    Ok(SessionAutoAttachExecution::Attached {
        obligation_id,
        attach_claim_owner,
        receipt,
    })
}

#[allow(dead_code)]
pub(crate) fn build_auto_attach_launch_plan(
    store: &AgentRuntimeStateStore,
    orchestration_session_id: &str,
) -> Result<HiddenOwnerHelperLaunchPlan> {
    let record = store
        .load_session(orchestration_session_id)?
        .ok_or_else(|| anyhow::anyhow!("missing_session: orchestration session {orchestration_session_id} disappeared before automatic attach could launch"))?;
    if record.session.state.is_terminal() {
        anyhow::bail!(
            "terminal_session: orchestration session {orchestration_session_id} is terminal and cannot auto-attach"
        );
    }
    if record.live_orchestrator().is_some() {
        anyhow::bail!(
            "session_already_owned: orchestration session {orchestration_session_id} already has a live retained owner"
        );
    }

    let active_participant_id = record
        .session
        .active_participant_id()
        .ok_or_else(|| anyhow::anyhow!(
            "stale_linkage: orchestration session {orchestration_session_id} is missing authoritative orchestrator participant linkage"
        ))?;
    let participant = record
        .participants
        .iter()
        .find(|participant| participant.participant_id() == active_participant_id)
        .cloned()
        .ok_or_else(|| anyhow::anyhow!(
            "stale_linkage: orchestration session {orchestration_session_id} references missing participant {active_participant_id}"
        ))?;
    if !participant.handle.state.is_live() {
        anyhow::bail!(
            "stale_linkage: orchestration session {orchestration_session_id} references inactive participant {active_participant_id}"
        );
    }
    if !participant.matches_public_parent_linkage(&record.session) {
        anyhow::bail!(
            "stale_linkage: orchestration session {orchestration_session_id} active participant {active_participant_id} does not match exact orchestrator linkage"
        );
    }

    let attach_contract = record.session.host_attach_contract().cloned().ok_or_else(|| {
        anyhow::anyhow!(
            "owner_unreachable: orchestration session {} is missing durable host attach contract state",
            orchestration_session_id
        )
    })?;
    let attach_mode_preference = match attach_contract.attach_launch_knobs.attach_mode_preference {
        crate::execution::agent_runtime::orchestration_session::HostAttachModePreference::ContinuityRequired => {
            AttachModePreference::ContinuityRequired
        }
        crate::execution::agent_runtime::orchestration_session::HostAttachModePreference::ContinuityPreferred
        | crate::execution::agent_runtime::orchestration_session::HostAttachModePreference::FreshAllowed => {
            AttachModePreference::ContinuityPreferred
        }
    };
    if attach_contract.continuity_uaa_session_id.is_none() {
        match attach_contract.attach_launch_knobs.attach_mode_preference {
            crate::execution::agent_runtime::orchestration_session::HostAttachModePreference::ContinuityRequired => {
                anyhow::bail!(
                    "owner_unreachable: persisted host attach contract no longer has continuity required for this attach launch"
                );
            }
            crate::execution::agent_runtime::orchestration_session::HostAttachModePreference::ContinuityPreferred
            | crate::execution::agent_runtime::orchestration_session::HostAttachModePreference::FreshAllowed => {
                anyhow::bail!(
                    "unsupported_attach_mode: orchestration session {orchestration_session_id} would require fresh control-only attach, and that mode is not sanctioned in this slice"
                );
            }
        }
    }
    let envelope = DispatchRequestEnvelope {
        caller_kind: DispatchCallerKind::HumanReattach,
        baseline_kind: DispatchBaselineKind::PersistedHostAttach,
        backend_id: Some(attach_contract.backend_id.clone()),
        orchestration_session_id: Some(orchestration_session_id.to_string()),
        requested_execution_scope_override: None,
        capability_overrides: DispatchCapabilityOverrideSet::default(),
        attach_launch_knobs: AttachLaunchKnobs {
            requested_execution_scope: AgentExecutionScope::Host,
            host_execution_client_start: HostExecutionClientStart::StartNow,
            attach_mode_preference,
        },
        has_prompt_payload: false,
    };
    let resolved = resolve_persisted_host_attach_contract(&envelope, &attach_contract)
        .map_err(|err| anyhow::anyhow!("owner_unreachable: {err}"))?;
    let descriptor: RuntimeSelectionDescriptor = materialize_runtime_descriptor(&resolved)
        .map_err(|err| anyhow::anyhow!("owner_unreachable: {}", err.reason))?;

    Ok(HiddenOwnerHelperLaunchPlan {
        mode: OwnerHelperMode::Attach,
        descriptor: (&descriptor).into(),
        session: HiddenOwnerHelperSessionPlan {
            orchestration_session_id: record.session.orchestration_session_id.clone(),
            shell_trace_session_id: record.session.shell_trace_session_id.clone(),
            workspace_root: record.session.workspace_root.clone(),
            world_id: record.session.world_id.clone(),
            world_generation: record.session.world_generation,
        },
        participant: HiddenOwnerHelperParticipantPlan {
            participant_id: format!("ash_{}", Uuid::now_v7()),
            lease_token: Uuid::now_v7().to_string(),
            run_id: Uuid::now_v7().to_string(),
            resumed_from_participant_id: Some(participant.handle.participant_id.clone()),
            internal_uaa_session_id: attach_contract.continuity_uaa_session_id.clone(),
        },
        host_attach_contract: Some(attach_contract),
        startup_prompt: None,
        source_orchestration_session_id: None,
    })
}

fn ensure_router_auto_attach_supported() -> Result<()> {
    if cfg!(target_os = "linux") {
        return Ok(());
    }

    anyhow::bail!(ROUTER_AUTO_ATTACH_UNSUPPORTED_REASON);
}

fn mark_attach_failed_closed(
    store: &AgentRuntimeStateStore,
    orchestration_session_id: &str,
    obligation_id: &str,
    reason: &str,
) -> Result<()> {
    let Some(mut obligation) = store.load_obligation(orchestration_session_id, obligation_id)?
    else {
        return Ok(());
    };
    if !obligation.is_pending() {
        return Ok(());
    }

    obligation.mark_attach_failed_closed(reason, Utc::now());
    store.persist_obligation(&obligation)
}

fn ensure_auto_attach_restored_session(
    store: &AgentRuntimeStateStore,
    orchestration_session_id: &str,
    participant_id: &str,
) -> Result<()> {
    let record = store
        .load_session(orchestration_session_id)?
        .ok_or_else(|| anyhow::anyhow!(
            "owner_unreachable: orchestration session {orchestration_session_id} disappeared before automatic attach could be verified"
        ))?;
    let Some(live_owner) = record.live_orchestrator() else {
        anyhow::bail!(
            "owner_unreachable: orchestration session {orchestration_session_id} did not restore a live retained owner after automatic attach"
        );
    };
    if live_owner.participant_id() != participant_id {
        anyhow::bail!(
            "owner_unreachable: orchestration session {orchestration_session_id} restored participant {} instead of expected {} after automatic attach",
            live_owner.participant_id(),
            participant_id
        );
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    use crate::execution::agent_runtime::mapping::AgentRuntimeBackendKind;
    use crate::execution::agent_runtime::orchestration_session::{
        HostAttachContract, HostAttachExecutionClientStart, HostAttachLaunchKnobs,
        HostAttachModePreference,
    };
    use crate::execution::agent_runtime::validator::RuntimeSelectionDescriptor;
    use crate::execution::agent_runtime::{
        AgentRuntimeParticipantRecord, AgentRuntimeSessionState, OrchestrationSessionRecord,
        OrchestrationSessionState, PURE_AGENT_PROTOCOL,
    };

    fn with_store(test: impl FnOnce(&AgentRuntimeStateStore)) {
        let temp = tempfile::tempdir().expect("tempdir");
        std::env::set_var("SUBSTRATE_HOME", temp.path());
        let store = AgentRuntimeStateStore::new().expect("state store");
        test(&store);
        std::env::remove_var("SUBSTRATE_HOME");
    }

    fn detached_orchestrator(
        session_id: &str,
        participant_id: &str,
    ) -> (OrchestrationSessionRecord, AgentRuntimeParticipantRecord) {
        let descriptor = RuntimeSelectionDescriptor {
            agent_id: "codex".to_string(),
            backend_id: "cli:codex".to_string(),
            backend_kind: AgentRuntimeBackendKind::Codex,
            protocol: PURE_AGENT_PROTOCOL.to_string(),
            execution_scope: AgentExecutionScope::Host,
            binary_path: PathBuf::from("/bin/sh"),
        };
        let mut participant = AgentRuntimeParticipantRecord::new_orchestrator_participant(
            &descriptor,
            session_id.to_string(),
            participant_id.to_string(),
            format!("lease_{participant_id}"),
        )
        .expect("orchestrator participant");
        participant.transition_state(AgentRuntimeSessionState::Ready);
        participant.set_uaa_session_id(format!("uaa-{session_id}"));
        participant.mark_client_detached("owner detached cleanly");
        participant.touch_heartbeat();

        let mut orchestration = OrchestrationSessionRecord::new(
            session_id.to_string(),
            format!("trace_{session_id}"),
            "/workspace".to_string(),
            &participant,
            HostAttachContract::from_manifest_for_test(&participant),
        );
        orchestration.transition_state(OrchestrationSessionState::Active);
        orchestration.bind_active_session_handle(participant.handle.participant_id.clone());
        orchestration.mark_parked_resumable("owner detached cleanly");
        (orchestration, participant)
    }

    #[test]
    #[serial_test::serial]
    fn auto_attach_launch_plan_prefers_continuity_when_available() {
        with_store(|store| {
            let (session, participant) =
                detached_orchestrator("sess_auto_attach_continuity", "ash_auto_attach_source");
            store
                .persist_orchestration_session(&session)
                .expect("persist session");
            store
                .persist_participant(&participant)
                .expect("persist participant");

            let plan = build_auto_attach_launch_plan(store, "sess_auto_attach_continuity")
                .expect("build auto attach plan");
            assert_eq!(plan.mode, OwnerHelperMode::Attach);
            assert_eq!(
                plan.participant.internal_uaa_session_id.as_deref(),
                Some("uaa-sess_auto_attach_continuity")
            );
            assert!(plan.requires_internal_session_id());
            assert_eq!(
                plan.participant.resumed_from_participant_id.as_deref(),
                Some("ash_auto_attach_source")
            );
        });
    }

    #[test]
    #[serial_test::serial]
    fn auto_attach_launch_plan_fails_closed_when_continuity_is_unavailable_and_fresh_would_be_required(
    ) {
        with_store(|store| {
            let (mut session, participant) =
                detached_orchestrator("sess_auto_attach_fresh", "ash_auto_attach_fresh_source");
            let mut contract = session
                .host_attach_contract()
                .cloned()
                .expect("attach contract");
            contract.continuity_uaa_session_id = None;
            contract.attach_launch_knobs = HostAttachLaunchKnobs {
                requested_execution_scope: AgentExecutionScope::Host,
                host_execution_client_start: HostAttachExecutionClientStart::StartNow,
                attach_mode_preference: HostAttachModePreference::FreshAllowed,
            };
            session.host_attach_contract = Some(contract);

            store
                .persist_orchestration_session(&session)
                .expect("persist session");
            store
                .persist_participant(&participant)
                .expect("persist participant");

            let err = build_auto_attach_launch_plan(store, "sess_auto_attach_fresh")
                .expect_err("fresh-needed automatic attach must fail closed");
            assert!(
                err.to_string().contains("unsupported_attach_mode"),
                "error should classify the fresh-needed branch as unsupported: {err:#}"
            );
            assert!(
                err.to_string()
                    .contains("would require fresh control-only attach"),
                "error should explain that continuity is missing and fresh attach is not allowed: {err:#}"
            );
        });
    }

    #[test]
    #[serial_test::serial]
    fn auto_attach_launch_plan_fails_closed_when_persisted_truth_requires_missing_continuity() {
        with_store(|store| {
            let (mut session, participant) = detached_orchestrator(
                "sess_auto_attach_missing_continuity",
                "ash_auto_attach_missing_source",
            );
            let mut contract = session
                .host_attach_contract()
                .cloned()
                .expect("attach contract");
            contract.continuity_uaa_session_id = None;
            session.host_attach_contract = Some(contract);

            store
                .persist_orchestration_session(&session)
                .expect("persist session");
            store
                .persist_participant(&participant)
                .expect("persist participant");

            let err = build_auto_attach_launch_plan(store, "sess_auto_attach_missing_continuity")
                .expect_err("missing required continuity must fail closed");
            assert!(
                err.to_string()
                    .contains("persisted host attach contract no longer has continuity required"),
                "error should explain missing persisted continuity truth: {err:#}"
            );
        });
    }
}
