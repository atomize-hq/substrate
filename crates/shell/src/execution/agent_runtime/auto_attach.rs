use std::path::Path;

use anyhow::Result;
use substrate_broker::Policy;
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
use super::orchestration_session::OrchestrationSessionPosture;
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
    pub failed_closed_obligation_ids: Vec<String>,
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
        settled: SessionAutoAttachSettleResult,
    },
    Attached {
        obligation_id: String,
        attach_claim_owner: String,
        receipt: HiddenOwnerHelperLaunchReceipt,
        settled: SessionAutoAttachSettleResult,
    },
}

impl SessionAutoAttachExecution {
    pub(crate) fn obligation_id(&self) -> Option<&str> {
        match self {
            Self::NoCandidate { .. } => None,
            Self::AlreadyClaimed { obligation_id }
            | Self::FailedClosed { obligation_id, .. }
            | Self::Attached { obligation_id, .. } => Some(obligation_id.as_str()),
        }
    }

    pub(crate) fn attach_claim_owner(&self) -> Option<&str> {
        match self {
            Self::NoCandidate { .. } | Self::AlreadyClaimed { .. } => None,
            Self::FailedClosed {
                attach_claim_owner, ..
            }
            | Self::Attached {
                attach_claim_owner, ..
            } => Some(attach_claim_owner.as_str()),
        }
    }

    pub(crate) fn completion_reason(&self) -> &str {
        match self {
            Self::NoCandidate { reason } => reason,
            Self::AlreadyClaimed { .. } => {
                "auto_attach_claim_already_owned_by_this_orchestration_session"
            }
            Self::FailedClosed { reason, .. } => reason.as_str(),
            Self::Attached { .. } => ROUTER_AUTO_ATTACH_RESTORED_REASON,
        }
    }

    pub(crate) fn settled(&self) -> Option<&SessionAutoAttachSettleResult> {
        match self {
            Self::Attached { settled, .. } | Self::FailedClosed { settled, .. } => Some(settled),
            Self::NoCandidate { .. } | Self::AlreadyClaimed { .. } => None,
        }
    }
}

#[allow(dead_code)]
#[derive(Clone, Debug)]
pub(crate) struct RouterAutoAttachSessionExecution {
    pub orchestration_session_id: String,
    pub execution: SessionAutoAttachExecution,
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
pub(crate) fn execute_router_auto_attach_for_eligible_sessions(
    store: &AgentRuntimeStateStore,
    router_identity: &str,
    fallback_policy: &Policy,
    world: bool,
    no_world: bool,
) -> Result<Vec<RouterAutoAttachSessionExecution>> {
    let mut executions = Vec::new();
    for orchestration_session_id in store.list_router_auto_attach_candidate_session_ids()? {
        executions.push(execute_router_auto_attach_for_session(
            store,
            &orchestration_session_id,
            router_identity,
            fallback_policy,
            world,
            no_world,
        )?);
    }
    Ok(executions)
}

#[allow(dead_code)]
pub(crate) fn execute_router_auto_attach_for_session(
    store: &AgentRuntimeStateStore,
    orchestration_session_id: &str,
    router_identity: &str,
    fallback_policy: &Policy,
    world: bool,
    no_world: bool,
) -> Result<RouterAutoAttachSessionExecution> {
    let execution = match resolve_router_auto_attach_policy_for_session(
        store,
        orchestration_session_id,
        fallback_policy,
    ) {
        Ok(session_policy) => execute_session_auto_attach(
            store,
            orchestration_session_id,
            router_identity,
            &session_policy,
            world,
            no_world,
        )?,
        Err(err) => fail_closed_auto_attach_policy_resolution(
            store,
            orchestration_session_id,
            router_identity,
            &err.to_string(),
        )?,
    };
    Ok(RouterAutoAttachSessionExecution {
        orchestration_session_id: orchestration_session_id.to_string(),
        execution,
    })
}

#[allow(dead_code)]
pub(crate) fn execute_session_auto_attach(
    store: &AgentRuntimeStateStore,
    orchestration_session_id: &str,
    router_identity: &str,
    policy: &Policy,
    world: bool,
    no_world: bool,
) -> Result<SessionAutoAttachExecution> {
    let claim = match select_policy_eligible_attach_candidate(
        &store.list_obligations(orchestration_session_id)?,
        policy,
    ) {
        Some(candidate) => store.claim_exact_session_auto_attach_obligation(
            orchestration_session_id,
            &candidate.obligation_id,
            router_identity,
        )?,
        None => store.claim_session_auto_attach(orchestration_session_id, router_identity)?,
    };
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
    let obligation = match load_claimed_obligation(store, orchestration_session_id, &obligation_id)
    {
        Ok(obligation) => obligation,
        Err(err) => {
            let reason = err.to_string();
            let settled = mark_attach_failed_closed(
                store,
                orchestration_session_id,
                &obligation_id,
                &reason,
            )?;
            return Ok(SessionAutoAttachExecution::FailedClosed {
                obligation_id,
                attach_claim_owner,
                reason,
                settled,
            });
        }
    };
    if let Err(err) = ensure_router_auto_attach_allowed(policy, &obligation) {
        let reason = err.to_string();
        let settled =
            mark_attach_failed_closed(store, orchestration_session_id, &obligation_id, &reason)?;
        return Ok(SessionAutoAttachExecution::FailedClosed {
            obligation_id,
            attach_claim_owner,
            reason,
            settled,
        });
    }
    if let Err(err) = ensure_router_auto_attach_supported() {
        let reason = err.to_string();
        let settled =
            mark_attach_failed_closed(store, orchestration_session_id, &obligation_id, &reason)?;
        return Ok(SessionAutoAttachExecution::FailedClosed {
            obligation_id,
            attach_claim_owner,
            reason,
            settled,
        });
    }

    let plan = match build_auto_attach_launch_plan(store, orchestration_session_id) {
        Ok(plan) => plan,
        Err(err) => {
            let reason = err.to_string();
            let settled = mark_attach_failed_closed(
                store,
                orchestration_session_id,
                &obligation_id,
                &reason,
            )?;
            return Ok(SessionAutoAttachExecution::FailedClosed {
                obligation_id,
                attach_claim_owner,
                reason,
                settled,
            });
        }
    };

    let receipt = match launch_hidden_owner_helper(&plan, world, no_world) {
        Ok(receipt) => receipt,
        Err(err) => {
            let reason = err.to_string();
            let settled = mark_attach_failed_closed(
                store,
                orchestration_session_id,
                &obligation_id,
                &reason,
            )?;
            return Ok(SessionAutoAttachExecution::FailedClosed {
                obligation_id,
                attach_claim_owner,
                reason,
                settled,
            });
        }
    };

    finalize_session_auto_attach_after_launch(
        store,
        orchestration_session_id,
        obligation_id,
        attach_claim_owner,
        receipt,
    )
}

fn fail_closed_auto_attach_policy_resolution(
    store: &AgentRuntimeStateStore,
    orchestration_session_id: &str,
    router_identity: &str,
    reason: &str,
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
    let settled =
        store.settle_session_auto_attach_failed_closed(orchestration_session_id, reason)?;
    Ok(SessionAutoAttachExecution::FailedClosed {
        obligation_id,
        attach_claim_owner,
        reason: reason.to_string(),
        settled,
    })
}

fn load_claimed_obligation(
    store: &AgentRuntimeStateStore,
    orchestration_session_id: &str,
    obligation_id: &str,
) -> Result<OrchestrationObligationRecord> {
    store
        .load_obligation(orchestration_session_id, obligation_id)?
        .ok_or_else(|| {
            anyhow::anyhow!(
                "missing_claimed_obligation: claimed router auto-attach obligation {obligation_id} disappeared before policy evaluation"
            )
        })
}

fn resolve_router_auto_attach_policy_for_session(
    store: &AgentRuntimeStateStore,
    orchestration_session_id: &str,
    fallback_policy: &Policy,
) -> Result<Policy> {
    let Some(session) = store.load_orchestration_session(orchestration_session_id)? else {
        return Ok(fallback_policy.clone());
    };
    let workspace_root = Path::new(&session.workspace_root);
    let (policy, _) =
        substrate_broker::resolve_effective_policy_with_explain(workspace_root, false).map_err(
            |err| {
                anyhow::anyhow!(
            "failed to resolve router auto-attach policy for orchestration session {} from {}: {}",
            orchestration_session_id,
            workspace_root.display(),
            err
        )
            },
        )?;
    Ok(policy)
}

fn ensure_router_auto_attach_allowed(
    policy: &Policy,
    obligation: &OrchestrationObligationRecord,
) -> Result<()> {
    if !policy.workflow_router_enabled() {
        anyhow::bail!(
            "router_auto_attach_disabled: workflow.router.enabled must be true before router-owned automatic attach may execute"
        );
    }

    let (allowed, policy_key) = router_auto_attach_kind_gate(policy, obligation.kind);
    let Some(policy_key) = policy_key else {
        anyhow::bail!(
            "router_auto_attach_unsupported_kind: obligation kind {:?} is not eligible for router-owned automatic attach",
            obligation.kind
        );
    };
    if !allowed {
        anyhow::bail!(
            "router_auto_attach_policy_denied: obligation {} of kind {:?} requires {}=true before router-owned automatic attach may execute",
            obligation.obligation_id,
            obligation.kind,
            policy_key,
        );
    }

    Ok(())
}

fn select_policy_eligible_attach_candidate<'a>(
    obligations: &'a [OrchestrationObligationRecord],
    policy: &Policy,
) -> Option<&'a OrchestrationObligationRecord> {
    if !policy.workflow_router_enabled() {
        return None;
    }

    let mut candidates = obligations
        .iter()
        .filter(|obligation| {
            obligation.is_auto_attach_eligible()
                && router_auto_attach_kind_gate(policy, obligation.kind).0
        })
        .collect::<Vec<_>>();
    candidates.sort_by(|left, right| {
        candidate_priority(left.kind)
            .cmp(&candidate_priority(right.kind))
            .then(left.created_at.cmp(&right.created_at))
            .then(left.obligation_id.cmp(&right.obligation_id))
    });
    candidates.into_iter().next()
}

fn router_auto_attach_kind_gate(
    policy: &Policy,
    kind: OrchestrationObligationKind,
) -> (bool, Option<&'static str>) {
    match kind {
        OrchestrationObligationKind::ApprovalRequired => (
            policy.world_dispatch_approval_requests_allowed(),
            Some("agents.world_dispatch.obligations.approval_allowed"),
        ),
        OrchestrationObligationKind::FollowUpRequired => (
            policy.world_dispatch_follow_up_allowed(),
            Some("agents.world_dispatch.obligations.follow_up_allowed"),
        ),
        OrchestrationObligationKind::Blocked => (
            policy.world_dispatch_blocked_allowed(),
            Some("agents.world_dispatch.obligations.blocked_allowed"),
        ),
        OrchestrationObligationKind::ForkRequest => (
            policy.world_dispatch_fork_requests_allowed(),
            Some("agents.world_dispatch.fork.requests_allowed"),
        ),
        _ => (false, None),
    }
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
    _obligation_id: &str,
    reason: &str,
) -> Result<SessionAutoAttachSettleResult> {
    store.settle_session_auto_attach_failed_closed(orchestration_session_id, reason)
}

fn finalize_session_auto_attach_after_launch(
    store: &AgentRuntimeStateStore,
    orchestration_session_id: &str,
    obligation_id: String,
    attach_claim_owner: String,
    receipt: HiddenOwnerHelperLaunchReceipt,
) -> Result<SessionAutoAttachExecution> {
    if let Err(err) = ensure_auto_attach_restored_session(
        store,
        orchestration_session_id,
        &receipt.participant_id,
    ) {
        let reason = err.to_string();
        let settled =
            mark_attach_failed_closed(store, orchestration_session_id, &obligation_id, &reason)?;
        return Ok(SessionAutoAttachExecution::FailedClosed {
            obligation_id,
            attach_claim_owner,
            reason,
            settled,
        });
    }
    let settled = store.settle_session_auto_attach_after_attach_restored(
        orchestration_session_id,
        ROUTER_AUTO_ATTACH_RESTORED_REASON,
    )?;

    Ok(SessionAutoAttachExecution::Attached {
        obligation_id,
        attach_claim_owner,
        receipt,
        settled,
    })
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
    if record.session.posture != OrchestrationSessionPosture::ActiveAttached {
        anyhow::bail!(
            "owner_unreachable: orchestration session {orchestration_session_id} did not restore active_attached posture after automatic attach"
        );
    }
    if record.session.attached_participant_id() != Some(participant_id) {
        anyhow::bail!(
            "owner_unreachable: orchestration session {orchestration_session_id} restored attached participant {:?} instead of expected {} after automatic attach",
            record.session.attached_participant_id(),
            participant_id
        );
    }
    let Some(live_owner) = record.live_orchestrator() else {
        anyhow::bail!(
            "owner_unreachable: orchestration session {orchestration_session_id} did not restore a live retained owner after automatic attach"
        );
    };
    if !live_owner.attached_client_present() {
        anyhow::bail!(
            "owner_unreachable: orchestration session {orchestration_session_id} did not restore an attached host execution client after automatic attach"
        );
    }
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
    use std::{fs, path::Path, path::PathBuf};
    use substrate_broker::Policy;

    use crate::execution::agent_runtime::mapping::AgentRuntimeBackendKind;
    use crate::execution::agent_runtime::obligation_ledger::{
        OrchestrationObligationAttachState, OrchestrationObligationRecord,
    };
    use crate::execution::agent_runtime::orchestration_session::{
        HostAttachContract, HostAttachExecutionClientStart, HostAttachLaunchKnobs,
        HostAttachModePreference, OrchestrationSessionPosture,
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

    fn detached_live_orchestrator(
        session_id: &str,
        participant_id: &str,
    ) -> (OrchestrationSessionRecord, AgentRuntimeParticipantRecord) {
        let (session, mut participant) = detached_orchestrator(session_id, participant_id);
        participant.mark_runtime_ownership_retained();
        participant.mark_client_detached("owner detached cleanly");
        (session, participant)
    }

    fn eligible_obligation(
        orchestration_session_id: &str,
        obligation_id: &str,
        kind: OrchestrationObligationKind,
    ) -> OrchestrationObligationRecord {
        let mut obligation = OrchestrationObligationRecord::new(
            orchestration_session_id,
            obligation_id,
            kind,
            format!("summary for {obligation_id}"),
        );
        obligation.attention_required = true;
        obligation.attach_state = OrchestrationObligationAttachState::Eligible;
        obligation
    }

    fn router_auto_attach_policy(kind: OrchestrationObligationKind) -> Policy {
        let mut policy = Policy {
            workflow_router_enabled: true,
            ..Policy::default()
        };
        match kind {
            OrchestrationObligationKind::ApprovalRequired => {
                policy.agents_world_dispatch_obligations_approval_allowed = true;
            }
            OrchestrationObligationKind::FollowUpRequired => {
                policy.agents_world_dispatch_obligations_follow_up_allowed = true;
            }
            OrchestrationObligationKind::Blocked => {
                policy.agents_world_dispatch_obligations_blocked_allowed = true;
            }
            OrchestrationObligationKind::ForkRequest => {
                policy.agents_world_dispatch_fork_requests_allowed = true;
            }
            other => panic!("unexpected auto-attach kind in test helper: {other:?}"),
        }
        policy
    }

    fn write_workspace_policy(workspace_root: &Path, yaml: &str) {
        let policy_dir = workspace_root.join(".substrate");
        fs::create_dir_all(&policy_dir).expect("create workspace policy dir");
        fs::write(policy_dir.join("workspace.yaml"), "schema_version: 1\n")
            .expect("write workspace marker");
        fs::write(policy_dir.join("policy.yaml"), yaml).expect("write workspace policy");
    }

    #[test]
    fn select_attach_candidate_preserves_packet_two_attach_eligibility_defaults() {
        let approval = eligible_obligation(
            "sess_auto_attach_packet_two",
            "obl_approval",
            OrchestrationObligationKind::ApprovalRequired,
        );
        let blocked = eligible_obligation(
            "sess_auto_attach_packet_two",
            "obl_blocked",
            OrchestrationObligationKind::Blocked,
        );
        let fork_request = eligible_obligation(
            "sess_auto_attach_packet_two",
            "obl_fork_request",
            OrchestrationObligationKind::ForkRequest,
        );
        let follow_up = eligible_obligation(
            "sess_auto_attach_packet_two",
            "obl_follow_up",
            OrchestrationObligationKind::FollowUpRequired,
        );
        let fork_recommendation = eligible_obligation(
            "sess_auto_attach_packet_two",
            "obl_fork_recommendation",
            OrchestrationObligationKind::ForkRecommendation,
        );

        let obligations = [
            follow_up.clone(),
            blocked.clone(),
            fork_request.clone(),
            fork_recommendation.clone(),
            approval.clone(),
        ];
        let candidate = select_attach_candidate(&obligations)
            .expect("packet-two obligations should keep attach eligibility");
        assert_eq!(candidate.obligation_id, approval.obligation_id);

        let recommendation_only = select_attach_candidate(&[fork_recommendation]).is_none();
        assert!(
            recommendation_only,
            "fork recommendations must remain non-attach-eligible by default"
        );

        let blocked_candidates = [follow_up.clone(), fork_request.clone(), blocked.clone()];
        let blocked_candidate = select_attach_candidate(&blocked_candidates)
            .expect("blocked should outrank fork_request and follow_up");
        assert_eq!(blocked_candidate.obligation_id, blocked.obligation_id);

        let fork_request_candidates = [follow_up.clone(), fork_request.clone()];
        let fork_request_candidate = select_attach_candidate(&fork_request_candidates)
            .expect("fork_request should outrank follow_up");
        assert_eq!(
            fork_request_candidate.obligation_id,
            fork_request.obligation_id
        );

        let follow_up_candidates = [follow_up];
        let follow_up_candidate = select_attach_candidate(&follow_up_candidates)
            .expect("follow_up should remain eligible");
        assert_eq!(follow_up_candidate.obligation_id, "obl_follow_up");
    }

    #[test]
    fn router_auto_attach_policy_is_deny_by_default_and_per_kind_gated() {
        let router_disabled = Policy::default();
        let approval = eligible_obligation(
            "sess_auto_attach_policy_disabled",
            "obl_approval",
            OrchestrationObligationKind::ApprovalRequired,
        );
        let disabled_err = ensure_router_auto_attach_allowed(&router_disabled, &approval)
            .expect_err("disabled router policy must fail closed");
        assert!(
            disabled_err
                .to_string()
                .contains("workflow.router.enabled must be true"),
            "router-disabled error must point at workflow.router.enabled: {disabled_err:#}"
        );

        let cases = [
            (
                OrchestrationObligationKind::ApprovalRequired,
                "agents.world_dispatch.obligations.approval_allowed",
            ),
            (
                OrchestrationObligationKind::FollowUpRequired,
                "agents.world_dispatch.obligations.follow_up_allowed",
            ),
            (
                OrchestrationObligationKind::Blocked,
                "agents.world_dispatch.obligations.blocked_allowed",
            ),
            (
                OrchestrationObligationKind::ForkRequest,
                "agents.world_dispatch.fork.requests_allowed",
            ),
        ];

        for (kind, policy_key) in cases {
            let obligation_id = format!("obl_{kind:?}").to_lowercase();
            let obligation =
                eligible_obligation("sess_auto_attach_policy_gate", &obligation_id, kind);
            let router_enabled_but_denied = Policy {
                workflow_router_enabled: true,
                ..Policy::default()
            };
            let denied_err =
                ensure_router_auto_attach_allowed(&router_enabled_but_denied, &obligation)
                    .expect_err("per-kind router policy should stay deny-by-default");
            assert!(
                denied_err.to_string().contains(policy_key),
                "denied error for {kind:?} must reference {policy_key}: {denied_err:#}"
            );

            ensure_router_auto_attach_allowed(&router_auto_attach_policy(kind), &obligation)
                .expect("per-kind router gate should allow matching auto-attach kinds");
        }
    }

    #[test]
    fn select_policy_eligible_attach_candidate_skips_denied_higher_priority_sibling() {
        let approval = eligible_obligation(
            "sess_auto_attach_policy_selection",
            "obl_approval",
            OrchestrationObligationKind::ApprovalRequired,
        );
        let follow_up = eligible_obligation(
            "sess_auto_attach_policy_selection",
            "obl_follow_up",
            OrchestrationObligationKind::FollowUpRequired,
        );
        let policy = router_auto_attach_policy(OrchestrationObligationKind::FollowUpRequired);

        let obligations = [approval, follow_up];
        let candidate = select_policy_eligible_attach_candidate(&obligations, &policy)
            .expect("policy-eligible follow-up should still route auto-attach");
        assert_eq!(candidate.obligation_id, "obl_follow_up");
    }

    #[test]
    #[serial_test::serial]
    fn execute_session_auto_attach_fails_closed_before_launch_when_router_policy_is_disabled() {
        with_store(|store| {
            let (session, participant) = detached_orchestrator(
                "sess_auto_attach_policy_disabled",
                "ash_auto_attach_policy_disabled",
            );
            store
                .persist_orchestration_session(&session)
                .expect("persist session");
            store
                .persist_participant(&participant)
                .expect("persist participant");

            let approval = eligible_obligation(
                "sess_auto_attach_policy_disabled",
                "obl_approval",
                OrchestrationObligationKind::ApprovalRequired,
            );
            let follow_up = eligible_obligation(
                "sess_auto_attach_policy_disabled",
                "obl_follow_up",
                OrchestrationObligationKind::FollowUpRequired,
            );
            store
                .persist_obligation(&approval)
                .expect("persist approval obligation");
            store
                .persist_obligation(&follow_up)
                .expect("persist follow-up obligation");

            let execution = execute_session_auto_attach(
                store,
                "sess_auto_attach_policy_disabled",
                "router::local",
                &Policy::default(),
                false,
                false,
            )
            .expect("policy-denied auto attach should fail closed, not error");
            let SessionAutoAttachExecution::FailedClosed { reason, .. } = execution else {
                panic!("expected policy-denied automatic attach to fail closed");
            };
            assert!(
                reason.contains("workflow.router.enabled must be true"),
                "policy denial should point at workflow.router.enabled: {reason}"
            );

            let obligation = store
                .load_obligation("sess_auto_attach_policy_disabled", "obl_approval")
                .expect("reload failed-closed obligation")
                .expect("obligation exists after failed-close");
            assert_eq!(
                obligation.attach_state,
                OrchestrationObligationAttachState::FailedClosed
            );
            assert_eq!(
                obligation.attach_completion_reason.as_deref(),
                Some(reason.as_str())
            );

            let sibling = store
                .load_obligation("sess_auto_attach_policy_disabled", "obl_follow_up")
                .expect("reload sibling obligation")
                .expect("sibling obligation exists after failed-close");
            assert_eq!(
                sibling.attach_state,
                OrchestrationObligationAttachState::FailedClosed
            );
            assert_eq!(
                sibling.attach_completion_reason.as_deref(),
                Some(reason.as_str())
            );
            assert!(
                !store
                    .list_router_auto_attach_candidate_session_ids()
                    .expect("list candidates after session fail-close")
                    .contains(&"sess_auto_attach_policy_disabled".to_string()),
                "session-wide fail-close should prevent silent router retries after policy denial"
            );
        });
    }

    #[test]
    #[serial_test::serial]
    fn finalize_session_auto_attach_after_launch_returns_failed_closed_when_restore_check_fails() {
        with_store(|store| {
            let (mut session, participant) = detached_live_orchestrator(
                "sess_auto_attach_restore_failed",
                "ash_auto_attach_restore_failed",
            );
            store
                .persist_orchestration_session(&session)
                .expect("persist detached session");
            store
                .persist_participant(&participant)
                .expect("persist detached participant");

            store
                .persist_obligation(&{
                    let mut obligation = eligible_obligation(
                        "sess_auto_attach_restore_failed",
                        "obl_restore_failed",
                        OrchestrationObligationKind::Blocked,
                    );
                    obligation.mark_attach_claimed("router::local", chrono::Utc::now());
                    obligation
                })
                .expect("persist eligible obligation");

            session.bind_active_session_handle("ash_auto_attach_restore_failed");
            store
                .persist_orchestration_session(&session)
                .expect("persist attached session without client restore");

            let execution = finalize_session_auto_attach_after_launch(
                store,
                "sess_auto_attach_restore_failed",
                "obl_restore_failed".to_string(),
                "router::local".to_string(),
                HiddenOwnerHelperLaunchReceipt {
                    helper_pid: std::process::id(),
                    orchestration_session_id: "sess_auto_attach_restore_failed".to_string(),
                    participant_id: "ash_auto_attach_restore_failed".to_string(),
                    backend_id: "cli:codex".to_string(),
                },
            )
            .expect("restore verification failure should fail closed, not error");
            let SessionAutoAttachExecution::FailedClosed {
                reason, settled, ..
            } = execution
            else {
                panic!("restore verification failure should return failed-closed execution");
            };
            assert!(
                reason.contains("did not restore an attached host execution client"),
                "restore verification failure should explain the authoritative attach mismatch: {reason}"
            );
            assert_eq!(
                settled.failed_closed_obligation_ids,
                vec!["obl_restore_failed".to_string()]
            );
            assert!(
                !store
                    .list_router_auto_attach_candidate_session_ids()
                    .expect("list candidates after restore failure")
                    .contains(&"sess_auto_attach_restore_failed".to_string()),
                "failed-closed restore verification should keep the router from retrying the same session silently"
            );
        });
    }

    #[test]
    #[serial_test::serial]
    fn execute_router_auto_attach_for_eligible_sessions_discovers_detached_candidates_only() {
        with_store(|store| {
            let (detached_session, detached_participant) =
                detached_orchestrator("sess_auto_attach_batch", "ash_auto_attach_batch");
            store
                .persist_orchestration_session(&detached_session)
                .expect("persist detached session");
            store
                .persist_participant(&detached_participant)
                .expect("persist detached participant");
            store
                .persist_obligation(&eligible_obligation(
                    "sess_auto_attach_batch",
                    "obl_batch",
                    OrchestrationObligationKind::ApprovalRequired,
                ))
                .expect("persist detached obligation");

            let (mut attached_session, attached_participant) =
                detached_orchestrator("sess_auto_attach_attached_skip", "ash_attached_skip");
            attached_session.posture = OrchestrationSessionPosture::ActiveAttached;
            attached_session.attached_participant_id =
                Some(attached_participant.handle.participant_id.clone());
            store
                .persist_orchestration_session(&attached_session)
                .expect("persist attached session");
            store
                .persist_obligation(&eligible_obligation(
                    "sess_auto_attach_attached_skip",
                    "obl_attached_skip",
                    OrchestrationObligationKind::ApprovalRequired,
                ))
                .expect("persist attached obligation");

            let executions = execute_router_auto_attach_for_eligible_sessions(
                store,
                "router::packet_two",
                &Policy::default(),
                false,
                false,
            )
            .expect("execute detached router auto-attach batch");

            assert_eq!(executions.len(), 1);
            assert_eq!(
                executions[0].orchestration_session_id,
                "sess_auto_attach_batch"
            );
            let SessionAutoAttachExecution::FailedClosed {
                obligation_id,
                attach_claim_owner,
                reason,
                ..
            } = &executions[0].execution
            else {
                panic!("detached candidate should fail closed under disabled router policy");
            };
            assert_eq!(obligation_id, "obl_batch");
            assert_eq!(attach_claim_owner, "router::packet_two");
            assert!(
                reason.contains("workflow.router.enabled must be true"),
                "policy-disabled batch execution should record the router gate failure: {reason}"
            );

            let detached = store
                .load_obligation("sess_auto_attach_batch", "obl_batch")
                .expect("load detached obligation")
                .expect("detached obligation exists");
            assert_eq!(
                detached.attach_state,
                OrchestrationObligationAttachState::FailedClosed
            );

            let attached = store
                .load_obligation("sess_auto_attach_attached_skip", "obl_attached_skip")
                .expect("load attached obligation")
                .expect("attached obligation exists");
            assert_eq!(
                attached.attach_state,
                OrchestrationObligationAttachState::Eligible
            );
        });
    }

    #[test]
    #[serial_test::serial]
    fn execute_router_auto_attach_for_eligible_sessions_resolves_policy_per_candidate_workspace() {
        with_store(|store| {
            let workspace_denied = tempfile::tempdir().expect("denied workspace");
            let workspace_allowed = tempfile::tempdir().expect("allowed workspace");
            write_workspace_policy(workspace_denied.path(), "{}\n");
            write_workspace_policy(
                workspace_allowed.path(),
                r#"
workflow:
  router:
    enabled: true
agents:
  world_dispatch:
    obligations:
      approval_allowed: true
"#,
            );

            let (mut denied_session, denied_participant) =
                detached_orchestrator("sess_auto_attach_denied_workspace", "ash_denied");
            denied_session.workspace_root = workspace_denied.path().display().to_string();
            store
                .persist_orchestration_session(&denied_session)
                .expect("persist denied session");
            store
                .persist_participant(&denied_participant)
                .expect("persist denied participant");

            let (mut allowed_session, allowed_participant) =
                detached_orchestrator("sess_auto_attach_allowed_workspace", "ash_allowed");
            allowed_session.workspace_root = workspace_allowed.path().display().to_string();
            store
                .persist_orchestration_session(&allowed_session)
                .expect("persist allowed session");
            store
                .persist_participant(&allowed_participant)
                .expect("persist allowed participant");

            let denied = resolve_router_auto_attach_policy_for_session(
                store,
                "sess_auto_attach_denied_workspace",
                &Policy::default(),
            )
            .expect("resolve denied workspace policy");
            assert!(
                !denied.workflow_router_enabled(),
                "denied workspace should preserve its own disabled router policy"
            );

            let allowed = resolve_router_auto_attach_policy_for_session(
                store,
                "sess_auto_attach_allowed_workspace",
                &Policy::default(),
            )
            .expect("resolve allowed workspace policy");
            assert!(
                allowed.workflow_router_enabled(),
                "allowed workspace should resolve its own router-enabled policy"
            );
            assert!(
                allowed.world_dispatch_approval_requests_allowed(),
                "allowed workspace should preserve its own approval gate"
            );
        });
    }

    #[test]
    #[serial_test::serial]
    fn execute_router_auto_attach_for_eligible_sessions_fail_closes_bad_workspace_per_session() {
        with_store(|store| {
            let workspace_invalid = tempfile::tempdir().expect("invalid workspace");
            let workspace_denied = tempfile::tempdir().expect("denied workspace");
            write_workspace_policy(workspace_invalid.path(), "workflow: [\n");
            write_workspace_policy(workspace_denied.path(), "{}\n");

            let (mut invalid_session, invalid_participant) =
                detached_orchestrator("sess_auto_attach_invalid_workspace", "ash_invalid");
            invalid_session.workspace_root = workspace_invalid.path().display().to_string();
            store
                .persist_orchestration_session(&invalid_session)
                .expect("persist invalid session");
            store
                .persist_participant(&invalid_participant)
                .expect("persist invalid participant");
            store
                .persist_obligation(&eligible_obligation(
                    "sess_auto_attach_invalid_workspace",
                    "obl_invalid_workspace",
                    OrchestrationObligationKind::ApprovalRequired,
                ))
                .expect("persist invalid workspace obligation");
            store
                .persist_obligation(&eligible_obligation(
                    "sess_auto_attach_invalid_workspace",
                    "obl_invalid_workspace_sibling",
                    OrchestrationObligationKind::FollowUpRequired,
                ))
                .expect("persist invalid workspace sibling obligation");

            let (mut denied_session, denied_participant) =
                detached_orchestrator("sess_auto_attach_denied_workspace", "ash_denied");
            denied_session.workspace_root = workspace_denied.path().display().to_string();
            store
                .persist_orchestration_session(&denied_session)
                .expect("persist denied session");
            store
                .persist_participant(&denied_participant)
                .expect("persist denied participant");
            store
                .persist_obligation(&eligible_obligation(
                    "sess_auto_attach_denied_workspace",
                    "obl_denied_workspace",
                    OrchestrationObligationKind::ApprovalRequired,
                ))
                .expect("persist denied workspace obligation");

            let executions = execute_router_auto_attach_for_eligible_sessions(
                store,
                "router::packet_two",
                &Policy::default(),
                false,
                false,
            )
            .expect("policy-resolution failure should fail closed per session");

            assert_eq!(executions.len(), 2);

            let invalid_execution = executions
                .iter()
                .find(|execution| {
                    execution.orchestration_session_id == "sess_auto_attach_invalid_workspace"
                })
                .expect("invalid workspace execution");
            let SessionAutoAttachExecution::FailedClosed {
                obligation_id,
                attach_claim_owner,
                reason,
                ..
            } = &invalid_execution.execution
            else {
                panic!("invalid workspace should fail closed without aborting batch");
            };
            assert_eq!(obligation_id, "obl_invalid_workspace");
            assert_eq!(attach_claim_owner, "router::packet_two");
            assert!(
                reason.contains("failed to resolve router auto-attach policy"),
                "policy-resolution failure should be recorded on the affected session: {reason}"
            );

            let denied_execution = executions
                .iter()
                .find(|execution| {
                    execution.orchestration_session_id == "sess_auto_attach_denied_workspace"
                })
                .expect("denied workspace execution");
            let SessionAutoAttachExecution::FailedClosed {
                obligation_id,
                attach_claim_owner,
                reason,
                ..
            } = &denied_execution.execution
            else {
                panic!("healthy sibling session should still execute in the same batch");
            };
            assert_eq!(obligation_id, "obl_denied_workspace");
            assert_eq!(attach_claim_owner, "router::packet_two");
            assert!(
                reason.contains("workflow.router.enabled must be true"),
                "sibling session should still reach ordinary policy gating: {reason}"
            );

            for (session_id, obligation_id) in [
                (
                    "sess_auto_attach_invalid_workspace",
                    "obl_invalid_workspace",
                ),
                ("sess_auto_attach_denied_workspace", "obl_denied_workspace"),
            ] {
                let obligation = store
                    .load_obligation(session_id, obligation_id)
                    .expect("reload obligation after fail-close")
                    .expect("obligation exists after fail-close");
                assert_eq!(
                    obligation.attach_state,
                    OrchestrationObligationAttachState::FailedClosed
                );
            }

            let invalid_sibling = store
                .load_obligation(
                    "sess_auto_attach_invalid_workspace",
                    "obl_invalid_workspace_sibling",
                )
                .expect("reload invalid workspace sibling after fail-close")
                .expect("invalid workspace sibling exists after fail-close");
            assert_eq!(
                invalid_sibling.attach_state,
                OrchestrationObligationAttachState::FailedClosed
            );
            assert!(
                invalid_sibling
                    .attach_completion_reason
                    .as_deref()
                    .expect("fail-closed sibling reason")
                    .contains("failed to resolve router auto-attach policy"),
                "session-level policy-resolution failure should dead-letter sibling obligations too"
            );
            assert!(
                !store
                    .list_router_auto_attach_candidate_session_ids()
                    .expect("list candidates after session fail-close")
                    .contains(&"sess_auto_attach_invalid_workspace".to_string()),
                "session-level fail-close should prevent silent router retries for siblings"
            );
        });
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
