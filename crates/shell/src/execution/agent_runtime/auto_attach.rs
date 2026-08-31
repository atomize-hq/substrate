use std::path::Path;
#[cfg(target_os = "linux")]
use std::path::PathBuf;

#[cfg(target_os = "linux")]
use anyhow::Context;
use anyhow::Result;
use gethostname::gethostname;
#[cfg(target_os = "linux")]
use sha2::{Digest, Sha256};
use substrate_broker::Policy;
use uuid::Uuid;

#[cfg(not(target_os = "linux"))]
use super::control::launch_hidden_owner_helper;
#[cfg(target_os = "linux")]
use super::control::{
    acquire_authority_managed_successor_launch_permit,
    launch_authority_managed_successor_owner_helper,
    wait_for_authority_managed_successor_completion, AuthorityManagedSuccessorLaunchPlanV1,
    ResolvedRuntimeBackendKind, ResolvedRuntimeDescriptor,
};
use super::control::{
    HiddenOwnerHelperLaunchPlan, HiddenOwnerHelperLaunchReceipt, HiddenOwnerHelperParticipantPlan,
    HiddenOwnerHelperSessionPlan, OwnerHelperMode,
};
#[cfg(not(target_os = "linux"))]
use super::dispatch_contract::{
    resolve_persisted_host_attach_contract, AttachLaunchKnobs, AttachModePreference,
    DispatchBaselineKind, DispatchCallerKind, DispatchCapabilityOverrideSet,
    DispatchRequestEnvelope, HostExecutionClientStart,
};
#[cfg(target_os = "linux")]
use super::host_session_authority::schema::{
    AgentExecutionScopeV1, AuthorityObjectKindV1, HostSessionAuthorityPreconditionV1,
    HostSessionPostureV1, HostSessionTransitionCallerKindV1, HostSessionTransitionCallerV1,
    HostSessionTransitionModeV1, RuntimeBackendKindV1, StartContinuationHandleHashInputV2,
    StartContinuationHandleStateV2,
};
#[cfg(target_os = "linux")]
use super::host_session_authority::store::BootstrapClassificationV1;
#[cfg(target_os = "linux")]
use super::host_session_authority::store_schema::{
    AuthorityObjectStorageStateV1, HostSessionStartupOwnershipApplicationV1,
    HostSessionTransitionIntentStateV3, HostSessionTransitionIntentV3, StateRootV3,
};
#[cfg(target_os = "linux")]
use super::host_session_authority::transition::{
    ApplyHostSessionTransitionRequestV1, ClaimHostSessionTransitionRequestV1,
    IssueSuccessorTransitionRequestV1, SuccessorTransitionApplicationOutcomeV1,
    SuccessorTransitionClaimOutcomeV1, SuccessorTransitionIssueOutcomeV1,
};
#[cfg(target_os = "linux")]
use super::host_session_authority::trusted_fs::TrustedAuthorityRoot;
#[cfg(target_os = "linux")]
use super::host_session_authority::{
    AuthorityObservationV1, HostSessionAuthority, ResolvedCurrentAuthorityV1,
};
#[cfg(target_os = "linux")]
use super::obligation_ledger::{
    capture_post_hsa_auto_attach_ledger_snapshot, claim_post_hsa_auto_attach_obligation,
    settle_post_hsa_auto_attach_obligation, PostHsaAutoAttachClaimOutcomeV1,
    PostHsaAutoAttachClaimRequestV1, PostHsaAutoAttachSettlementKindV1,
    PostHsaAutoAttachSettlementOutcomeV1, PostHsaAutoAttachSettlementRequestV1,
    PostHsaAutoAttachSettlementResultV1,
};
use super::obligation_ledger::{
    LocalHostObligationTargetingDisposition, OrchestrationObligationKind,
    OrchestrationObligationRecord,
};
#[cfg(not(target_os = "linux"))]
use super::orchestration_session::OrchestrationSessionPosture;
use super::state_store::AgentRuntimeStateStore;
#[cfg(not(target_os = "linux"))]
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

#[cfg(target_os = "linux")]
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
    #[cfg(target_os = "linux")]
    let candidate_session_ids =
        list_router_auto_attach_candidate_session_ids(store, router_identity)?;
    #[cfg(not(target_os = "linux"))]
    let candidate_session_ids = store.list_router_auto_attach_candidate_session_ids()?;
    for orchestration_session_id in candidate_session_ids {
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
            fallback_policy,
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
    #[cfg(target_os = "linux")]
    let post_hsa_context =
        resolve_post_hsa_auto_attach_authority_context(store, orchestration_session_id)?;
    #[cfg(target_os = "linux")]
    if post_hsa_context.is_none() {
        if let Some(candidate) =
            preview_router_auto_attach_claim_candidate(store, orchestration_session_id, policy)?
        {
            if let Err(err) = ensure_router_auto_attach_targets_local_host(&candidate) {
                let reason = err.to_string();
                let settled = mark_exact_obligation_failed_closed(
                    store,
                    orchestration_session_id,
                    &candidate.obligation_id,
                    &reason,
                )?;
                return Ok(SessionAutoAttachExecution::FailedClosed {
                    obligation_id: candidate.obligation_id,
                    attach_claim_owner: router_identity.to_string(),
                    reason,
                    settled,
                });
            }
        }
    }
    #[cfg(not(target_os = "linux"))]
    if let Some(candidate) =
        preview_router_auto_attach_claim_candidate(store, orchestration_session_id, policy)?
    {
        if let Err(err) = ensure_router_auto_attach_targets_local_host(&candidate) {
            let reason = err.to_string();
            let settled = mark_exact_obligation_failed_closed(
                store,
                orchestration_session_id,
                &candidate.obligation_id,
                &reason,
            )?;
            return Ok(SessionAutoAttachExecution::FailedClosed {
                obligation_id: candidate.obligation_id,
                attach_claim_owner: router_identity.to_string(),
                reason,
                settled,
            });
        }
    }

    #[cfg(target_os = "linux")]
    let claim = match post_hsa_context.as_ref() {
        Some(context) => claim_post_hsa_session_auto_attach(
            store,
            orchestration_session_id,
            router_identity,
            policy,
            context,
        )?,
        None => match select_policy_eligible_attach_candidate(
            &store.list_obligations(orchestration_session_id)?,
            policy,
        ) {
            Some(candidate) => store.claim_exact_session_auto_attach_obligation(
                orchestration_session_id,
                &candidate.obligation_id,
                router_identity,
            )?,
            None => store.claim_session_auto_attach(orchestration_session_id, router_identity)?,
        },
    };
    #[cfg(not(target_os = "linux"))]
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
            let obligation =
                load_claimed_obligation(store, orchestration_session_id, &obligation_id)?;
            match obligation.attach_claim_owner.as_deref() {
                Some(existing_owner) if existing_owner == router_identity => {
                    (obligation_id, existing_owner.to_string())
                }
                _ => {
                    return Ok(SessionAutoAttachExecution::AlreadyClaimed { obligation_id });
                }
            }
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
    if let Err(err) = ensure_router_auto_attach_targets_local_host(&obligation) {
        let reason = err.to_string();
        let settled = mark_exact_obligation_failed_closed(
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

    let plan = match build_auto_attach_launch_plan(
        store,
        orchestration_session_id,
        &obligation_id,
        &attach_claim_owner,
    ) {
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

    #[cfg(target_os = "linux")]
    let receipt = match (|| -> Result<HiddenOwnerHelperLaunchReceipt> {
        validate_auto_attach_launch_plan(store, &plan, &obligation_id, &attach_claim_owner)?;
        let permit = acquire_authority_managed_successor_launch_permit(&plan)?;
        let launched =
            launch_authority_managed_successor_owner_helper(&plan, permit, world, no_world)?;
        let receipt = launched.launch_receipt.clone();
        wait_for_authority_managed_successor_completion(&plan)?;
        Ok(receipt)
    })() {
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
    #[cfg(not(target_os = "linux"))]
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

    #[cfg(target_os = "linux")]
    {
        finalize_session_auto_attach_after_launch(
            store,
            orchestration_session_id,
            obligation_id,
            attach_claim_owner,
            receipt,
            &plan,
        )
    }
    #[cfg(not(target_os = "linux"))]
    {
        finalize_session_auto_attach_after_launch(
            store,
            orchestration_session_id,
            obligation_id,
            attach_claim_owner,
            receipt,
        )
    }
}

fn preview_router_auto_attach_claim_candidate(
    store: &AgentRuntimeStateStore,
    orchestration_session_id: &str,
    policy: &Policy,
) -> Result<Option<OrchestrationObligationRecord>> {
    if !store
        .list_router_auto_attach_candidate_session_ids()?
        .iter()
        .any(|candidate_session_id| candidate_session_id == orchestration_session_id)
    {
        return Ok(None);
    }

    let obligations = store.list_obligations(orchestration_session_id)?;
    Ok(
        select_policy_eligible_attach_candidate(&obligations, policy)
            .or_else(|| select_attach_candidate(&obligations))
            .cloned(),
    )
}

fn fail_closed_auto_attach_policy_resolution(
    store: &AgentRuntimeStateStore,
    orchestration_session_id: &str,
    router_identity: &str,
    fallback_policy: &Policy,
    reason: &str,
) -> Result<SessionAutoAttachExecution> {
    #[cfg(target_os = "linux")]
    let claim =
        match resolve_post_hsa_auto_attach_authority_context(store, orchestration_session_id)? {
            Some(context) => claim_post_hsa_session_auto_attach(
                store,
                orchestration_session_id,
                router_identity,
                fallback_policy,
                &context,
            )?,
            None => store.claim_session_auto_attach(orchestration_session_id, router_identity)?,
        };
    #[cfg(not(target_os = "linux"))]
    let claim = store.claim_session_auto_attach(orchestration_session_id, router_identity)?;
    let (obligation_id, attach_claim_owner) = match claim {
        SessionAutoAttachClaim::NoCandidate { reason } => {
            return Ok(SessionAutoAttachExecution::NoCandidate { reason });
        }
        SessionAutoAttachClaim::AlreadyClaimed { obligation_id } => {
            let obligation =
                load_claimed_obligation(store, orchestration_session_id, &obligation_id)?;
            match obligation.attach_claim_owner.as_deref() {
                Some(existing_owner) if existing_owner == router_identity => {
                    (obligation_id, existing_owner.to_string())
                }
                _ => {
                    return Ok(SessionAutoAttachExecution::AlreadyClaimed { obligation_id });
                }
            }
        }
        SessionAutoAttachClaim::Claimed {
            obligation_id,
            attach_claim_owner,
        } => (obligation_id, attach_claim_owner),
    };
    let settled =
        mark_attach_failed_closed(store, orchestration_session_id, &obligation_id, reason)?;
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
    let workspace_root = if let Some(session) =
        store.load_orchestration_session(orchestration_session_id)?
    {
        session.workspace_root
    } else {
        #[cfg(target_os = "linux")]
        {
            let Some(context) =
                resolve_post_hsa_auto_attach_authority_context(store, orchestration_session_id)?
            else {
                return Ok(fallback_policy.clone());
            };
            context.workspace_root
        }
        #[cfg(not(target_os = "linux"))]
        {
            return Ok(fallback_policy.clone());
        }
    };
    let workspace_root = Path::new(&workspace_root);
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

fn ensure_router_auto_attach_targets_local_host(
    obligation: &OrchestrationObligationRecord,
) -> Result<()> {
    if obligation.target_host_id.is_none() {
        return Ok(());
    }

    let local_host_id = resolve_router_auto_attach_local_host_id()?;
    ensure_router_auto_attach_targets_exact_local_host(obligation, &local_host_id)
}

fn ensure_router_auto_attach_targets_exact_local_host(
    obligation: &OrchestrationObligationRecord,
    local_host_id: &str,
) -> Result<()> {
    match obligation.classify_local_host_targeting(local_host_id)? {
        LocalHostObligationTargetingDisposition::Untargeted
        | LocalHostObligationTargetingDisposition::TargetedToLocalHost => Ok(()),
        LocalHostObligationTargetingDisposition::WrongHost { target_host_id } => {
            anyhow::bail!(
                "wrong_target_host: obligation {} targets host {} not local host {}",
                obligation.obligation_id,
                target_host_id,
                local_host_id,
            );
        }
    }
}

fn resolve_router_auto_attach_local_host_id() -> Result<String> {
    let local_host_id = gethostname().to_string_lossy().trim().to_string();
    if local_host_id.is_empty() {
        anyhow::bail!(
            "local_host_id_unavailable: router-owned automatic attach requires exact local host identity before evaluating target_host_id"
        );
    }

    Ok(local_host_id)
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

#[cfg(target_os = "linux")]
#[derive(Clone, Debug)]
struct AutoAttachSuccessorIdentityV1 {
    intent_id: String,
    issuer_request_id: String,
    target_participant_id: String,
    run_id: String,
    claim_id: String,
    claimant_attempt_id: String,
}

#[cfg(target_os = "linux")]
fn state_store_authority_home(store: &AgentRuntimeStateStore) -> Result<PathBuf> {
    let sessions_dir = store.sessions_dir();
    let home = sessions_dir
        .parent()
        .and_then(Path::parent)
        .and_then(Path::parent)
        .ok_or_else(|| {
            anyhow::anyhow!(
                "owner_unreachable: auto-attach StateStore has no bound bootstrap-home ancestor"
            )
        })?;
    if sessions_dir != home.join("run").join("agent-hub").join("sessions") {
        anyhow::bail!(
            "owner_unreachable: auto-attach StateStore bootstrap-home binding is malformed"
        );
    }
    Ok(home.to_path_buf())
}

#[cfg(target_os = "linux")]
fn auto_attach_authority(store: &AgentRuntimeStateStore) -> Result<HostSessionAuthority> {
    let home = state_store_authority_home(store)?;
    let trusted = TrustedAuthorityRoot::open(&home)
        .map_err(|error| anyhow::anyhow!(error.to_string()))
        .context("open exact router auto-attach HSA authority store")?;
    HostSessionAuthority::from_trusted_root(trusted)
        .map_err(|error| anyhow::anyhow!(error.to_string()))
}

#[cfg(target_os = "linux")]
struct PostHsaAutoAttachAuthorityContextV1 {
    observation: AuthorityObservationV1,
    active_participant_id: String,
    posture: HostSessionPostureV1,
    workspace_root: String,
}

#[cfg(target_os = "linux")]
fn resolve_post_hsa_auto_attach_authority_context(
    store: &AgentRuntimeStateStore,
    orchestration_session_id: &str,
) -> Result<Option<PostHsaAutoAttachAuthorityContextV1>> {
    let authority = auto_attach_authority(store)?;
    match authority.classify() {
        BootstrapClassificationV1::FreshAbsent
        | BootstrapClassificationV1::UnsupportedLegacyState => return Ok(None),
        BootstrapClassificationV1::InitializationPending => {
            anyhow::bail!("post-HSA auto-attach authority initialization is incomplete")
        }
        BootstrapClassificationV1::CorruptOrUnsupported => {
            authority.read_a12b_root().map_err(|_| {
                anyhow::anyhow!("post-HSA auto-attach authority is corrupt or unsupported")
            })?;
        }
        BootstrapClassificationV1::ValidExisting => {}
    }
    let resolved = authority
        .resolve_current_exact(orchestration_session_id, None)
        .map_err(|error| anyhow::anyhow!(error.to_string()))?;
    let active_participant_id = resolved
        .authority
        .active_authoritative_participant_id
        .clone()
        .ok_or_else(|| {
            anyhow::anyhow!("post-HSA auto-attach authority has no active participant")
        })?;
    Ok(Some(PostHsaAutoAttachAuthorityContextV1 {
        observation: resolved.observation,
        active_participant_id,
        posture: resolved.authority.lifecycle_posture,
        workspace_root: resolved
            .authority
            .workspace_binding
            .workspace_root
            .physical_path,
    }))
}

#[cfg(target_os = "linux")]
fn list_router_auto_attach_candidate_session_ids(
    store: &AgentRuntimeStateStore,
    router_identity: &str,
) -> Result<Vec<String>> {
    let authority = auto_attach_authority(store)?;
    match authority.classify() {
        BootstrapClassificationV1::FreshAbsent
        | BootstrapClassificationV1::UnsupportedLegacyState => {
            return store.list_router_auto_attach_candidate_session_ids();
        }
        BootstrapClassificationV1::InitializationPending => {
            anyhow::bail!("post-HSA auto-attach authority initialization is incomplete")
        }
        BootstrapClassificationV1::CorruptOrUnsupported => {
            authority.read_a12b_root().map_err(|_| {
                anyhow::anyhow!("post-HSA auto-attach authority is corrupt or unsupported")
            })?;
        }
        BootstrapClassificationV1::ValidExisting => {}
    }
    let mut candidates = Vec::new();
    for session_id in store.list_post_hsa_obligation_ledger_session_ids()? {
        let resolved = authority
            .resolve_current_exact(&session_id, None)
            .map_err(|error| anyhow::anyhow!(error.to_string()))?;
        let active_participant_id = resolved
            .authority
            .active_authoritative_participant_id
            .as_deref()
            .ok_or_else(|| {
                anyhow::anyhow!("post-HSA auto-attach candidate has no active participant")
            })?;
        let snapshot = capture_post_hsa_auto_attach_ledger_snapshot(store, &resolved.observation)?;
        if let Some(claimed_obligation_id) = claimed_obligation_id(&snapshot.obligations)? {
            let claimed = snapshot
                .obligations
                .iter()
                .find(|obligation| obligation.obligation_id == claimed_obligation_id)
                .expect("claimed obligation was selected from this snapshot");
            if claimed.attach_claim_owner.as_deref() != Some(router_identity) {
                anyhow::bail!("post-HSA auto-attach obligation is claimed by another owner");
            }
            match resolved.authority.lifecycle_posture {
                HostSessionPostureV1::ParkedResumable | HostSessionPostureV1::AwaitingAttention => {
                    if claimed.authoritative_participant_id.as_deref()
                        != Some(active_participant_id)
                    {
                        anyhow::bail!(
                            "post-HSA auto-attach claim does not belong to the current authority participant"
                        );
                    }
                }
                HostSessionPostureV1::ActiveAttached => {
                    let identity = auto_attach_successor_identity(
                        &resolved.observation.authority_store_id,
                        &session_id,
                        claimed_obligation_id,
                        router_identity,
                    );
                    let root = authority
                        .read_a12b_root()
                        .map_err(|error| anyhow::anyhow!(error.to_string()))?;
                    let intent = find_exact_auto_attach_successor(
                        &root,
                        &identity,
                        &session_id,
                        claimed_obligation_id,
                        router_identity,
                    )?
                    .ok_or_else(|| {
                        anyhow::anyhow!(
                            "active post-HSA auto-attach claim lacks its exact HSA transition"
                        )
                    })?;
                    if claimed.authoritative_participant_id.as_deref()
                        != intent.source_authoritative_participant_id.as_deref()
                        || intent.target_authoritative_participant_id != active_participant_id
                    {
                        anyhow::bail!(
                            "active post-HSA auto-attach claim changed its exact HSA participant binding"
                        );
                    }
                }
                _ => {
                    anyhow::bail!(
                        "post-HSA auto-attach claim has an incompatible authority posture"
                    )
                }
            }
            candidates.push(session_id);
            continue;
        }
        if !matches!(
            resolved.authority.lifecycle_posture,
            HostSessionPostureV1::ParkedResumable | HostSessionPostureV1::AwaitingAttention
        ) {
            continue;
        }
        let Some(candidate) = select_attach_candidate(&snapshot.obligations) else {
            continue;
        };
        if candidate.authoritative_participant_id.as_deref() != Some(active_participant_id) {
            anyhow::bail!(
                "post-HSA auto-attach candidate does not belong to the current authority participant"
            );
        }
        candidates.push(session_id);
    }
    candidates.sort();
    candidates.dedup();
    Ok(candidates)
}

#[cfg(target_os = "linux")]
fn claim_post_hsa_session_auto_attach(
    store: &AgentRuntimeStateStore,
    orchestration_session_id: &str,
    router_identity: &str,
    policy: &Policy,
    context: &PostHsaAutoAttachAuthorityContextV1,
) -> Result<SessionAutoAttachClaim> {
    if context.observation.orchestration_session_id != orchestration_session_id {
        anyhow::bail!("post-HSA auto-attach claim changed orchestration session scope");
    }
    let snapshot = capture_post_hsa_auto_attach_ledger_snapshot(store, &context.observation)?;
    if let Some(existing) = claimed_obligation_id(&snapshot.obligations)? {
        let obligation = snapshot
            .obligations
            .iter()
            .find(|obligation| obligation.obligation_id == existing)
            .expect("claimed obligation was selected from this snapshot");
        if obligation.attach_claim_owner.as_deref() == Some(router_identity) {
            match context.posture {
                HostSessionPostureV1::ParkedResumable | HostSessionPostureV1::AwaitingAttention => {
                    if obligation.authoritative_participant_id.as_deref()
                        != Some(context.active_participant_id.as_str())
                    {
                        anyhow::bail!(
                            "post-HSA auto-attach claim no longer belongs to the current authority participant"
                        );
                    }
                }
                HostSessionPostureV1::ActiveAttached => {
                    let authority = auto_attach_authority(store)?;
                    let root = authority
                        .read_a12b_root()
                        .map_err(|error| anyhow::anyhow!(error.to_string()))?;
                    let identity = auto_attach_successor_identity(
                        &context.observation.authority_store_id,
                        orchestration_session_id,
                        existing,
                        router_identity,
                    );
                    let intent = find_exact_auto_attach_successor(
                        &root,
                        &identity,
                        orchestration_session_id,
                        existing,
                        router_identity,
                    )?
                    .ok_or_else(|| {
                        anyhow::anyhow!(
                            "active post-HSA auto-attach claim has no exact HSA successor"
                        )
                    })?;
                    if intent.source_authoritative_participant_id.as_deref()
                        != obligation.authoritative_participant_id.as_deref()
                        || intent.target_authoritative_participant_id
                            != context.active_participant_id
                        || !matches!(
                            intent.state,
                            HostSessionTransitionIntentStateV3::Applied { .. }
                        )
                    {
                        anyhow::bail!(
                            "active post-HSA auto-attach claim is not bound to its exact source and completion"
                        );
                    }
                }
                _ => {
                    anyhow::bail!(
                        "post-HSA auto-attach claim cannot join from the current authority posture"
                    );
                }
            }
            return Ok(SessionAutoAttachClaim::AlreadyClaimed {
                obligation_id: existing.to_string(),
            });
        }
        anyhow::bail!("post-HSA auto-attach obligation is claimed by another owner");
    }
    if !matches!(
        context.posture,
        HostSessionPostureV1::ParkedResumable | HostSessionPostureV1::AwaitingAttention
    ) {
        return Ok(SessionAutoAttachClaim::NoCandidate {
            reason: "session_not_detached",
        });
    }
    let Some(candidate) = select_policy_eligible_attach_candidate(&snapshot.obligations, policy)
        .or_else(|| select_attach_candidate(&snapshot.obligations))
    else {
        return Ok(SessionAutoAttachClaim::NoCandidate {
            reason: "no_eligible_obligations",
        });
    };
    if candidate.authoritative_participant_id.as_deref()
        != Some(context.active_participant_id.as_str())
    {
        anyhow::bail!(
            "post-HSA auto-attach claim target does not belong to the current authority participant"
        );
    }
    let obligation_id = candidate.obligation_id.clone();
    let outcome = claim_post_hsa_auto_attach_obligation(
        store,
        &PostHsaAutoAttachClaimRequestV1 {
            expected_ledger: snapshot,
            obligation_id,
            claim_owner: router_identity.to_string(),
            claimed_at: chrono::Utc::now(),
        },
    )?;
    match outcome {
        PostHsaAutoAttachClaimOutcomeV1::Claimed(obligation) => {
            Ok(SessionAutoAttachClaim::Claimed {
                obligation_id: obligation.obligation_id,
                attach_claim_owner: obligation
                    .attach_claim_owner
                    .expect("post-HSA claim outcome must include its owner"),
            })
        }
        PostHsaAutoAttachClaimOutcomeV1::Joined(obligation) => {
            Ok(SessionAutoAttachClaim::AlreadyClaimed {
                obligation_id: obligation.obligation_id,
            })
        }
    }
}

#[cfg(target_os = "linux")]
fn session_auto_attach_settle_result_from_post_hsa(
    result: PostHsaAutoAttachSettlementResultV1,
) -> SessionAutoAttachSettleResult {
    SessionAutoAttachSettleResult {
        satisfied_obligation_ids: result.satisfied_obligation_ids,
        superseded_obligation_ids: result.superseded_obligation_ids,
        failed_closed_obligation_ids: result.failed_closed_obligation_ids,
    }
}

#[cfg(target_os = "linux")]
fn settle_post_hsa_auto_attach_failed_closed(
    store: &AgentRuntimeStateStore,
    orchestration_session_id: &str,
    obligation_id: &str,
    claim_owner: &str,
    reason: &str,
) -> Result<Option<SessionAutoAttachSettleResult>> {
    let Some(context) =
        resolve_post_hsa_auto_attach_authority_context(store, orchestration_session_id)?
    else {
        return Ok(None);
    };
    let snapshot = capture_post_hsa_auto_attach_ledger_snapshot(store, &context.observation)?;
    let outcome = settle_post_hsa_auto_attach_obligation(
        store,
        &PostHsaAutoAttachSettlementRequestV1 {
            expected_ledger: snapshot,
            obligation_id: obligation_id.to_string(),
            claim_owner: claim_owner.to_string(),
            completion_reason: reason.to_string(),
            settlement_kind: PostHsaAutoAttachSettlementKindV1::FailedClosed,
            router_auto_attach_intent: None,
            settled_at: chrono::Utc::now(),
        },
    )?;
    let result = match outcome {
        PostHsaAutoAttachSettlementOutcomeV1::Settled(result)
        | PostHsaAutoAttachSettlementOutcomeV1::Joined(result) => result,
    };
    Ok(Some(session_auto_attach_settle_result_from_post_hsa(
        result,
    )))
}

#[cfg(target_os = "linux")]
fn auto_attach_identity_digest(parts: &[&[u8]]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(b"substrate.router-auto-attach.successor.v1");
    for part in parts {
        hasher.update((part.len() as u64).to_be_bytes());
        hasher.update(part);
    }
    format!("{:x}", hasher.finalize())
}

#[cfg(target_os = "linux")]
fn auto_attach_successor_identity(
    authority_store_id: &str,
    orchestration_session_id: &str,
    obligation_id: &str,
    attach_claim_owner: &str,
) -> AutoAttachSuccessorIdentityV1 {
    let digest = auto_attach_identity_digest(&[
        authority_store_id.as_bytes(),
        orchestration_session_id.as_bytes(),
        obligation_id.as_bytes(),
        attach_claim_owner.as_bytes(),
    ]);
    AutoAttachSuccessorIdentityV1 {
        intent_id: format!("intent_router_auto_attach_{digest}"),
        issuer_request_id: format!("request_router_auto_attach_{digest}"),
        target_participant_id: format!("ash_router_auto_attach_{digest}"),
        run_id: format!("run_router_auto_attach_{digest}"),
        claim_id: format!("claim_router_auto_attach_{digest}"),
        claimant_attempt_id: format!("attempt_router_auto_attach_{digest}"),
    }
}

#[cfg(target_os = "linux")]
fn find_exact_auto_attach_successor(
    root: &StateRootV3,
    identity: &AutoAttachSuccessorIdentityV1,
    orchestration_session_id: &str,
    obligation_id: &str,
    attach_claim_owner: &str,
) -> Result<Option<HostSessionTransitionIntentV3>> {
    let mut exact = None;
    for intent in root.successor_transition_intent_map.values() {
        if intent.intent_id == identity.intent_id
            && (intent.orchestration_session_id != orchestration_session_id
                || intent.caller.kind != HostSessionTransitionCallerKindV1::RouterAutoAttach
                || intent.caller.auto_attach_obligation_id.as_deref() != Some(obligation_id)
                || intent.caller.auto_attach_claim_owner.as_deref() != Some(attach_claim_owner))
        {
            anyhow::bail!(
                "router auto-attach successor identity is occupied by a substituted producer or scope"
            );
        }
        if intent.orchestration_session_id != orchestration_session_id
            || intent.caller.auto_attach_obligation_id.as_deref() != Some(obligation_id)
        {
            continue;
        }
        if intent.caller.kind != HostSessionTransitionCallerKindV1::RouterAutoAttach
            || intent.caller.auto_attach_claim_owner.as_deref() != Some(attach_claim_owner)
            || intent.intent_id != identity.intent_id
            || intent.issuer_request_id != identity.issuer_request_id
            || intent.target_authoritative_participant_id != identity.target_participant_id
            || intent.run_id != identity.run_id
        {
            anyhow::bail!(
                "router auto-attach obligation has a conflicting HSA successor transition"
            );
        }
        if exact.replace(intent.clone()).is_some() {
            anyhow::bail!("router auto-attach obligation has ambiguous HSA successors");
        }
    }
    Ok(exact)
}

#[cfg(target_os = "linux")]
fn auto_attach_issue_request_from_intent(
    store: &AgentRuntimeStateStore,
    authority: &HostSessionAuthority,
    root: &StateRootV3,
    intent: &HostSessionTransitionIntentV3,
) -> Result<IssueSuccessorTransitionRequestV1> {
    if intent.mode != HostSessionTransitionModeV1::Attach
        || intent.resume_handle_ref.is_some()
        || intent.transition_input_ref.is_some()
        || intent.post_turn_disposition.is_some()
    {
        anyhow::bail!("router auto-attach successor is not an exact prompt-free Attach");
    }
    let lease_token = read_auto_attach_lease_candidate(store, authority, root, intent)?;
    Ok(IssueSuccessorTransitionRequestV1 {
        intent_id: intent.intent_id.clone(),
        issuer_request_id: intent.issuer_request_id.clone(),
        mode: intent.mode,
        authority_precondition: intent.authority_precondition.clone(),
        orchestration_session_id: intent.orchestration_session_id.clone(),
        shell_trace_session_id: intent.shell_trace_session_id.clone(),
        caller: intent.caller.clone(),
        source_authoritative_participant_id: intent.source_authoritative_participant_id.clone(),
        target_authoritative_participant_id: intent.target_authoritative_participant_id.clone(),
        target_participant_lease_token: lease_token,
        run_id: intent.run_id.clone(),
        resulting_authoritative_lineage: intent.resulting_authoritative_lineage.clone(),
        workspace_binding: intent.workspace_binding.clone(),
        world_binding: intent.world_binding.clone(),
        resume_handle_ref: None,
        transition_input: None,
        post_turn_disposition: None,
    })
}

#[cfg(target_os = "linux")]
fn read_auto_attach_lease_candidate(
    store: &AgentRuntimeStateStore,
    authority: &HostSessionAuthority,
    root: &StateRootV3,
    intent: &HostSessionTransitionIntentV3,
) -> Result<Vec<u8>> {
    let reference = &intent.target_participant_lease_token_ref;
    let index = root
        .object_index
        .get(&reference.ref_id)
        .filter(|index| {
            index.schema_version == 1
                && index.ref_id == reference.ref_id
                && index.object_kind == AuthorityObjectKindV1::LeaseToken
                && index.object_schema_version == reference.schema_version
                && matches!(index.storage_state, AuthorityObjectStorageStateV1::Present)
        })
        .ok_or_else(|| {
            anyhow::anyhow!(
                "router auto-attach lease token lacks an exact present authority index entry"
            )
        })?;
    if reference.object_kind != AuthorityObjectKindV1::LeaseToken || reference.schema_version != 1 {
        anyhow::bail!("router auto-attach lease token reference changed kind or version");
    }

    let trusted = TrustedAuthorityRoot::open(&state_store_authority_home(store)?)
        .map_err(|error| anyhow::anyhow!(error.to_string()))?;
    trusted
        .revalidate()
        .map_err(|error| anyhow::anyhow!(error.to_string()))?;
    let authority_dir = trusted
        .directory()
        .open_controlled_directory("authority-v1")
        .map_err(|_| anyhow::anyhow!("open exact auto-attach authority directory"))?;
    let objects = authority_dir
        .open_controlled_directory("objects")
        .map_err(|_| anyhow::anyhow!("open exact auto-attach object directory"))?;
    let lease_kind = objects
        .open_controlled_directory("lease-token")
        .map_err(|_| anyhow::anyhow!("open exact auto-attach lease-token directory"))?;
    let version = lease_kind
        .open_controlled_directory("v1")
        .map_err(|_| anyhow::anyhow!("open exact auto-attach lease-token version"))?;
    let bytes = version
        .open_file(&format!("{}.obj", reference.ref_id))
        .and_then(|file| file.read_all())
        .map_err(|_| anyhow::anyhow!("read exact auto-attach lease-token object"))?;
    trusted
        .revalidate()
        .map_err(|error| anyhow::anyhow!(error.to_string()))?;
    if bytes.is_empty() || bytes.len() as u64 != index.byte_length {
        anyhow::bail!("router auto-attach lease-token object changed length");
    }
    let verified = authority
        .read_a12b_root()
        .map_err(|error| anyhow::anyhow!(error.to_string()))?;
    if verified.root_revision != root.root_revision
        || verified
            .successor_transition_intent_map
            .get(&intent.intent_id)
            != Some(intent)
    {
        anyhow::bail!("router auto-attach lease-token parent changed during exact recovery");
    }
    Ok(bytes)
}

#[cfg(target_os = "linux")]
struct AutoAttachClaimBinding<'a> {
    orchestration_session_id: &'a str,
    obligation_id: &'a str,
    attach_claim_owner: &'a str,
}

#[cfg(target_os = "linux")]
fn issue_or_exact_join_auto_attach_successor(
    store: &AgentRuntimeStateStore,
    authority: &HostSessionAuthority,
    resolved: &ResolvedCurrentAuthorityV1,
    root: &StateRootV3,
    identity: &AutoAttachSuccessorIdentityV1,
    claim: AutoAttachClaimBinding<'_>,
) -> Result<(HostSessionTransitionIntentV3, Vec<u8>)> {
    let existing = find_exact_auto_attach_successor(
        root,
        identity,
        claim.orchestration_session_id,
        claim.obligation_id,
        claim.attach_claim_owner,
    )?;
    let request = match existing.as_ref() {
        Some(intent) => auto_attach_issue_request_from_intent(store, authority, root, intent)?,
        None => {
            let source_participant_id = resolved
                .authority
                .active_authoritative_participant_id
                .clone()
                .ok_or_else(|| {
                    anyhow::anyhow!(
                        "owner_unreachable: current HSA authority has no active participant"
                    )
                })?;
            let mut lineage = resolved.authority.authoritative_participant_lineage.clone();
            lineage.push(identity.target_participant_id.clone());
            IssueSuccessorTransitionRequestV1 {
                intent_id: identity.intent_id.clone(),
                issuer_request_id: identity.issuer_request_id.clone(),
                mode: HostSessionTransitionModeV1::Attach,
                authority_precondition: HostSessionAuthorityPreconditionV1::ExpectedRevision {
                    authority_revision: resolved.observation.authority_revision,
                    authority_record_commitment: resolved
                        .observation
                        .authority_record_commitment
                        .clone(),
                    active_authoritative_participant_id: source_participant_id.clone(),
                    authoritative_lineage_commitment: resolved
                        .observation
                        .authoritative_lineage_commitment
                        .clone(),
                    lifecycle_posture: resolved.authority.lifecycle_posture,
                },
                orchestration_session_id: claim.orchestration_session_id.to_string(),
                shell_trace_session_id: resolved.authority.shell_trace_session_id.clone(),
                caller: HostSessionTransitionCallerV1 {
                    kind: HostSessionTransitionCallerKindV1::RouterAutoAttach,
                    caller_participant_id: None,
                    auto_attach_obligation_id: Some(claim.obligation_id.to_string()),
                    auto_attach_claim_owner: Some(claim.attach_claim_owner.to_string()),
                },
                source_authoritative_participant_id: Some(source_participant_id),
                target_authoritative_participant_id: identity.target_participant_id.clone(),
                target_participant_lease_token: Uuid::now_v7().to_string().into_bytes(),
                run_id: identity.run_id.clone(),
                resulting_authoritative_lineage: lineage,
                workspace_binding: resolved.authority.workspace_binding.clone(),
                world_binding: resolved.authority.world_binding.clone(),
                resume_handle_ref: None,
                transition_input: None,
                post_turn_disposition: None,
            }
        }
    };
    let lease_token = request.target_participant_lease_token.clone();
    if existing.as_ref().is_some_and(|intent| {
        matches!(
            intent.state,
            HostSessionTransitionIntentStateV3::Applied { .. }
        )
    }) {
        return Ok((
            existing.expect("existing applied successor checked above"),
            lease_token,
        ));
    }
    match authority.issue_successor(resolved, &request) {
        Ok(SuccessorTransitionIssueOutcomeV1::Issued(intent))
        | Ok(SuccessorTransitionIssueOutcomeV1::Joined(intent)) => Ok((intent, lease_token)),
        Err(initial_error) if existing.is_none() => {
            let refreshed = authority
                .read_a12b_root()
                .map_err(|error| anyhow::anyhow!(error.to_string()))?;
            let Some(raced) = find_exact_auto_attach_successor(
                &refreshed,
                identity,
                claim.orchestration_session_id,
                claim.obligation_id,
                claim.attach_claim_owner,
            )?
            else {
                return Err(anyhow::anyhow!(initial_error.to_string()));
            };
            let exact =
                auto_attach_issue_request_from_intent(store, authority, &refreshed, &raced)?;
            let lease_token = exact.target_participant_lease_token.clone();
            match authority
                .issue_successor(resolved, &exact)
                .map_err(|error| anyhow::anyhow!(error.to_string()))?
            {
                SuccessorTransitionIssueOutcomeV1::Joined(intent) => Ok((intent, lease_token)),
                SuccessorTransitionIssueOutcomeV1::Issued(_) => anyhow::bail!(
                    "router auto-attach race recovery issued a replacement HSA intent"
                ),
            }
        }
        Err(error) => Err(anyhow::anyhow!(error.to_string())),
    }
}

#[cfg(target_os = "linux")]
fn claim_and_apply_auto_attach_successor(
    authority: &HostSessionAuthority,
    identity: &AutoAttachSuccessorIdentityV1,
    issued: HostSessionTransitionIntentV3,
) -> Result<HostSessionTransitionIntentV3> {
    maybe_inject_auto_attach_producer_crash("after_issue")?;
    if let HostSessionTransitionIntentStateV3::Applied {
        claim_id,
        claimant_attempt_id,
        ..
    } = &issued.state
    {
        if claim_id != &identity.claim_id || claimant_attempt_id != &identity.claimant_attempt_id {
            anyhow::bail!("router auto-attach applied intent has a substituted claim");
        }
        return Ok(issued);
    }
    let claimed = match &issued.state {
        HostSessionTransitionIntentStateV3::Issued
        | HostSessionTransitionIntentStateV3::Claimed { .. } => {
            match authority
                .claim_successor(&ClaimHostSessionTransitionRequestV1 {
                    intent_id: issued.intent_id.clone(),
                    issuer_request_id: issued.issuer_request_id.clone(),
                    payload_commitment: issued.payload_commitment.clone(),
                    expected_intent_revision: issued.intent_revision,
                    claim_id: identity.claim_id.clone(),
                    claimant_attempt_id: identity.claimant_attempt_id.clone(),
                })
                .map_err(|error| anyhow::anyhow!(error.to_string()))?
            {
                SuccessorTransitionClaimOutcomeV1::Claimed(intent)
                | SuccessorTransitionClaimOutcomeV1::Reclaimed(intent)
                | SuccessorTransitionClaimOutcomeV1::Joined(intent) => intent,
            }
        }
        HostSessionTransitionIntentStateV3::Applied { .. } => unreachable!(),
        HostSessionTransitionIntentStateV3::Rejected { .. }
        | HostSessionTransitionIntentStateV3::Expired { .. } => {
            anyhow::bail!("router auto-attach intent is durably terminal without application")
        }
    };
    maybe_inject_auto_attach_producer_crash("after_claim")?;
    let expected_claim_revision = match &claimed.state {
        HostSessionTransitionIntentStateV3::Claimed { claim_revision, .. } => *claim_revision,
        HostSessionTransitionIntentStateV3::Applied { .. } => {
            claimed.intent_revision.saturating_sub(1)
        }
        _ => anyhow::bail!("router auto-attach claim did not retain exact claim evidence"),
    };
    let applied = match authority
        .apply_successor(&ApplyHostSessionTransitionRequestV1 {
            intent_id: claimed.intent_id.clone(),
            issuer_request_id: claimed.issuer_request_id.clone(),
            payload_commitment: claimed.payload_commitment.clone(),
            expected_intent_revision: claimed.intent_revision,
            claim_id: identity.claim_id.clone(),
            expected_claim_revision,
        })
        .map_err(|error| anyhow::anyhow!(error.to_string()))?
    {
        SuccessorTransitionApplicationOutcomeV1::Applied(intent)
        | SuccessorTransitionApplicationOutcomeV1::Joined(intent) => intent,
    };
    maybe_inject_auto_attach_producer_crash("after_apply")?;
    Ok(applied)
}

#[cfg(target_os = "linux")]
fn exact_settled_auto_attach_start_continuity(
    authority: &HostSessionAuthority,
    root: &StateRootV3,
    resolved: &ResolvedCurrentAuthorityV1,
) -> Result<String> {
    let mut settled = None;
    for reference in &resolved.authority.internal_resume_handle_refs {
        if reference.schema_version != 2 {
            continue;
        }
        let bytes = authority
            .read_authority_object_v2_at(root.root_revision, reference)
            .map_err(|error| anyhow::anyhow!(error.to_string()))?;
        let handle: StartContinuationHandleHashInputV2 =
            super::host_session_authority::canonical_json::from_slice(&bytes)
                .context("decode router auto-attach durable Start continuation")?;
        if handle.authority_store_id != root.authority_store_id
            || handle.orchestration_session_id != resolved.authority.orchestration_session_id
            || handle.backend_id != resolved.caller.descriptor.backend_id
            || handle.protocol != resolved.caller.descriptor.protocol
            || handle.internal_uaa_session_id.trim().is_empty()
        {
            anyhow::bail!(
                "owner_unreachable: durable Start continuation does not authenticate router auto-attach authority"
            );
        }
        if matches!(handle.state, StartContinuationHandleStateV2::Settled { .. })
            && settled.replace(handle.internal_uaa_session_id).is_some()
        {
            anyhow::bail!(
                "owner_unreachable: router auto-attach authority has ambiguous settled Start continuations"
            );
        }
    }
    settled.ok_or_else(|| {
        anyhow::anyhow!(
            "owner_unreachable: router auto-attach authority has no settled durable Start continuation"
        )
    })
}

#[cfg(target_os = "linux")]
fn resolved_runtime_descriptor(resolved: &ResolvedCurrentAuthorityV1) -> ResolvedRuntimeDescriptor {
    let descriptor = &resolved.caller.descriptor;
    ResolvedRuntimeDescriptor {
        agent_id: descriptor.agent_id.clone(),
        backend_id: descriptor.backend_id.clone(),
        backend_kind: match descriptor.backend_kind {
            RuntimeBackendKindV1::Codex => ResolvedRuntimeBackendKind::Codex,
            RuntimeBackendKindV1::ClaudeCode => ResolvedRuntimeBackendKind::ClaudeCode,
        },
        protocol: descriptor.protocol.clone(),
        execution_scope: match descriptor.execution_scope {
            AgentExecutionScopeV1::Host => AgentExecutionScope::Host,
            AgentExecutionScopeV1::World => AgentExecutionScope::World,
        },
        binary_path: descriptor.binary_path.clone(),
    }
}

#[cfg(target_os = "linux")]
#[allow(dead_code)]
pub(crate) fn build_auto_attach_launch_plan(
    store: &AgentRuntimeStateStore,
    orchestration_session_id: &str,
    obligation_id: &str,
    attach_claim_owner: &str,
) -> Result<AuthorityManagedSuccessorLaunchPlanV1> {
    if obligation_id.trim().is_empty() || attach_claim_owner.trim().is_empty() {
        anyhow::bail!(
            "router auto-attach HSA producer requires exact obligation and claim identities"
        );
    }
    let authority = auto_attach_authority(store)?;
    let resolved = authority
        .resolve_current_exact(orchestration_session_id, None)
        .map_err(|error| anyhow::anyhow!(error.to_string()))?;
    let root = authority
        .read_a12b_root()
        .map_err(|error| anyhow::anyhow!(error.to_string()))?;
    let identity = auto_attach_successor_identity(
        &root.authority_store_id,
        orchestration_session_id,
        obligation_id,
        attach_claim_owner,
    );
    let (issued, lease_token) = issue_or_exact_join_auto_attach_successor(
        store,
        &authority,
        &resolved,
        &root,
        &identity,
        AutoAttachClaimBinding {
            orchestration_session_id,
            obligation_id,
            attach_claim_owner,
        },
    )?;
    let applied = claim_and_apply_auto_attach_successor(&authority, &identity, issued)?;
    let resolved = authority
        .resolve_current_exact(orchestration_session_id, None)
        .map_err(|error| anyhow::anyhow!(error.to_string()))?;
    let root = authority
        .read_a12b_root()
        .map_err(|error| anyhow::anyhow!(error.to_string()))?;
    let current = root
        .successor_transition_intent_map
        .get(&applied.intent_id)
        .ok_or_else(|| anyhow::anyhow!("router auto-attach applied intent disappeared"))?;
    if current != &applied {
        anyhow::bail!("router auto-attach applied result changed before plan projection");
    }
    let internal_uaa_session_id =
        exact_settled_auto_attach_start_continuity(&authority, &root, &resolved)?;
    let world_binding = applied.world_binding.as_ref();
    let plan = AuthorityManagedSuccessorLaunchPlanV1 {
        schema_version: 1,
        authority_store_id: root.authority_store_id.clone(),
        helper_plan: HiddenOwnerHelperLaunchPlan {
            mode: OwnerHelperMode::Attach,
            descriptor: resolved_runtime_descriptor(&resolved),
            session: HiddenOwnerHelperSessionPlan {
                orchestration_session_id: applied.orchestration_session_id.clone(),
                shell_trace_session_id: applied.shell_trace_session_id.clone(),
                workspace_root: applied
                    .workspace_binding
                    .workspace_root
                    .physical_path
                    .clone(),
                world_id: world_binding.map(|binding| binding.world_id.clone()),
                world_generation: world_binding.map(|binding| binding.world_generation),
            },
            participant: HiddenOwnerHelperParticipantPlan {
                participant_id: applied.target_authoritative_participant_id.clone(),
                lease_token: String::from_utf8(lease_token)
                    .context("router auto-attach participant lease token is not UTF-8")?,
                run_id: applied.run_id.clone(),
                resumed_from_participant_id: applied.source_authoritative_participant_id.clone(),
                internal_uaa_session_id: Some(internal_uaa_session_id),
            },
            host_attach_contract: None,
            startup_prompt: None,
            source_orchestration_session_id: None,
        },
        applied_transition: applied,
    };
    validate_auto_attach_launch_plan(store, &plan, obligation_id, attach_claim_owner)?;
    Ok(plan)
}

#[cfg(not(target_os = "linux"))]
#[allow(dead_code)]
pub(crate) fn build_auto_attach_launch_plan(
    store: &AgentRuntimeStateStore,
    orchestration_session_id: &str,
    _obligation_id: &str,
    _attach_claim_owner: &str,
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
    let continuity_uaa_session_id = attach_contract
        .public_attach_continuity_session_id()
        .map(ToOwned::to_owned);
    if continuity_uaa_session_id.is_none() {
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
            internal_uaa_session_id: continuity_uaa_session_id,
        },
        host_attach_contract: Some(attach_contract),
        startup_prompt: None,
        source_orchestration_session_id: None,
    })
}

#[cfg(target_os = "linux")]
fn validate_auto_attach_application_identity(
    expected: &HostSessionTransitionIntentV3,
    actual: &HostSessionTransitionIntentV3,
) -> Result<()> {
    if actual.schema_version != expected.schema_version
        || actual.intent_id != expected.intent_id
        || actual.issuer_request_id != expected.issuer_request_id
        || actual.mode != expected.mode
        || actual.authority_precondition != expected.authority_precondition
        || actual.orchestration_session_id != expected.orchestration_session_id
        || actual.shell_trace_session_id != expected.shell_trace_session_id
        || actual.caller != expected.caller
        || actual.source_authoritative_participant_id
            != expected.source_authoritative_participant_id
        || actual.target_authoritative_participant_id
            != expected.target_authoritative_participant_id
        || actual.target_participant_lease_token_ref != expected.target_participant_lease_token_ref
        || actual.run_id != expected.run_id
        || actual.resulting_authoritative_lineage != expected.resulting_authoritative_lineage
        || actual.workspace_binding != expected.workspace_binding
        || actual.world_binding != expected.world_binding
        || actual.descriptor_ref != expected.descriptor_ref
        || actual.host_attach_contract_ref != expected.host_attach_contract_ref
        || actual.resume_handle_ref != expected.resume_handle_ref
        || actual.transition_input_ref != expected.transition_input_ref
        || actual.post_turn_disposition != expected.post_turn_disposition
        || actual.transport_payload_ref != expected.transport_payload_ref
        || actual.payload_commitment != expected.payload_commitment
        || actual.issued_at != expected.issued_at
        || actual.expires_at != expected.expires_at
    {
        anyhow::bail!("router auto-attach launch application identity was substituted");
    }
    let (
        HostSessionTransitionIntentStateV3::Applied {
            claim_id: expected_claim_id,
            claimant_attempt_id: expected_claimant_attempt_id,
            authority_revision_before: expected_authority_revision_before,
            authority_revision_after: expected_authority_revision_after,
            active_authoritative_participant_id: expected_active_participant_id,
            resulting_posture: expected_resulting_posture,
            authority_record_commitment: expected_authority_record_commitment,
            application_result_ref: expected_application_result_ref,
            applied_at: expected_applied_at,
            ..
        },
        HostSessionTransitionIntentStateV3::Applied {
            claim_id: actual_claim_id,
            claimant_attempt_id: actual_claimant_attempt_id,
            authority_revision_before: actual_authority_revision_before,
            authority_revision_after: actual_authority_revision_after,
            active_authoritative_participant_id: actual_active_participant_id,
            resulting_posture: actual_resulting_posture,
            authority_record_commitment: actual_authority_record_commitment,
            application_result_ref: actual_application_result_ref,
            applied_at: actual_applied_at,
            ..
        },
    ) = (&expected.state, &actual.state)
    else {
        anyhow::bail!("router auto-attach launch application is not durably Applied");
    };
    if actual_claim_id != expected_claim_id
        || actual_claimant_attempt_id != expected_claimant_attempt_id
        || actual_authority_revision_before != expected_authority_revision_before
        || actual_authority_revision_after != expected_authority_revision_after
        || actual_active_participant_id != expected_active_participant_id
        || actual_resulting_posture != expected_resulting_posture
        || actual_authority_record_commitment != expected_authority_record_commitment
        || actual_application_result_ref != expected_application_result_ref
        || actual_applied_at != expected_applied_at
    {
        anyhow::bail!("router auto-attach launch application result was substituted");
    }
    Ok(())
}

#[cfg(target_os = "linux")]
fn validate_auto_attach_launch_plan(
    store: &AgentRuntimeStateStore,
    plan: &AuthorityManagedSuccessorLaunchPlanV1,
    obligation_id: &str,
    attach_claim_owner: &str,
) -> Result<()> {
    let authority_home = state_store_authority_home(store)?;
    if plan.schema_version != 1
        || plan.authority_store_id.is_empty()
        || plan
            .applied_transition
            .workspace_binding
            .authority_store_root
            .physical_path
            != authority_home.to_string_lossy()
        || plan.applied_transition.workspace_binding.authority_store_id != plan.authority_store_id
    {
        anyhow::bail!("router auto-attach launch plan authority-store identity was substituted");
    }
    let authority = auto_attach_authority(store)?;
    let root = authority
        .read_a12b_root()
        .map_err(|error| anyhow::anyhow!(error.to_string()))?;
    if root.authority_store_id != plan.authority_store_id {
        anyhow::bail!("router auto-attach launch plan authority store changed");
    }
    let identity = auto_attach_successor_identity(
        &root.authority_store_id,
        &plan.applied_transition.orchestration_session_id,
        obligation_id,
        attach_claim_owner,
    );
    let Some(current) = find_exact_auto_attach_successor(
        &root,
        &identity,
        &plan.applied_transition.orchestration_session_id,
        obligation_id,
        attach_claim_owner,
    )?
    else {
        anyhow::bail!("router auto-attach launch plan HSA intent disappeared");
    };
    validate_auto_attach_application_identity(&plan.applied_transition, &current)?;
    let HostSessionTransitionIntentStateV3::Applied {
        startup_ownership,
        post_turn,
        ..
    } = &current.state
    else {
        anyhow::bail!("router auto-attach launch plan HSA intent is not Applied");
    };
    if current.mode != HostSessionTransitionModeV1::Attach
        || current.caller.kind != HostSessionTransitionCallerKindV1::RouterAutoAttach
        || current.caller.caller_participant_id.is_some()
        || current.caller.auto_attach_obligation_id.as_deref() != Some(obligation_id)
        || current.caller.auto_attach_claim_owner.as_deref() != Some(attach_claim_owner)
        || current.resume_handle_ref.is_some()
        || current.transition_input_ref.is_some()
        || current.post_turn_disposition.is_some()
        || !matches!(
            startup_ownership.as_ref(),
            HostSessionStartupOwnershipApplicationV1::Pending { .. }
                | HostSessionStartupOwnershipApplicationV1::Accepted { .. }
        )
        || !matches!(
            post_turn.as_ref(),
            super::host_session_authority::store_schema::HostSessionPostTurnApplicationV2::NotApplicable
        )
    {
        anyhow::bail!("router auto-attach launch plan changed producer or Attach semantics");
    }
    let resolved = authority
        .resolve_current_exact(&current.orchestration_session_id, None)
        .map_err(|error| anyhow::anyhow!(error.to_string()))?;
    if resolved.observation.authority_store_id != plan.authority_store_id
        || resolved
            .authority
            .active_authoritative_participant_id
            .as_deref()
            != Some(current.target_authoritative_participant_id.as_str())
        || resolved.caller.descriptor_ref != current.descriptor_ref
        || resolved.authority.host_attach_contract_ref.as_ref()
            != Some(&current.host_attach_contract_ref)
    {
        anyhow::bail!(
            "router auto-attach launch plan no longer matches exact current HSA authority"
        );
    }
    let exact_rejoin = auto_attach_issue_request_from_intent(store, &authority, &root, &current)?;
    if exact_rejoin.target_participant_lease_token
        != plan.helper_plan.participant.lease_token.as_bytes()
    {
        anyhow::bail!("router auto-attach launch plan changed its participant lease token");
    }
    if matches!(
        startup_ownership.as_ref(),
        HostSessionStartupOwnershipApplicationV1::Pending { .. }
    ) {
        match authority
            .issue_successor(&resolved, &exact_rejoin)
            .map_err(|error| anyhow::anyhow!(error.to_string()))?
        {
            SuccessorTransitionIssueOutcomeV1::Joined(joined) if joined == current => {}
            _ => {
                anyhow::bail!("router auto-attach launch plan did not exact-rejoin its HSA intent")
            }
        }
    }
    let internal_uaa_session_id =
        exact_settled_auto_attach_start_continuity(&authority, &root, &resolved)?;
    let world_binding = current.world_binding.as_ref();
    let helper = &plan.helper_plan;
    if helper.mode != OwnerHelperMode::Attach
        || helper.descriptor != resolved_runtime_descriptor(&resolved)
        || helper.session.orchestration_session_id != current.orchestration_session_id
        || helper.session.shell_trace_session_id != current.shell_trace_session_id
        || helper.session.workspace_root != current.workspace_binding.workspace_root.physical_path
        || helper.session.world_id.as_deref()
            != world_binding.map(|binding| binding.world_id.as_str())
        || helper.session.world_generation != world_binding.map(|binding| binding.world_generation)
        || helper.participant.participant_id != current.target_authoritative_participant_id
        || helper.participant.run_id != current.run_id
        || helper.participant.resumed_from_participant_id
            != current.source_authoritative_participant_id
        || helper.participant.internal_uaa_session_id.as_deref()
            != Some(internal_uaa_session_id.as_str())
        || helper.host_attach_contract.is_some()
        || helper.startup_prompt.is_some()
        || helper.source_orchestration_session_id.is_some()
    {
        anyhow::bail!(
            "router auto-attach launch projection changed exact HSA identity or introduced input"
        );
    }
    Ok(())
}

#[cfg(all(target_os = "linux", test))]
thread_local! {
    static AUTO_ATTACH_PRODUCER_FAULT: std::cell::Cell<Option<&'static str>> =
        const { std::cell::Cell::new(None) };
}

#[cfg(all(target_os = "linux", test))]
fn inject_auto_attach_producer_fault_for_test(stage: &'static str) {
    AUTO_ATTACH_PRODUCER_FAULT.set(Some(stage));
}

#[cfg(all(target_os = "linux", test))]
fn maybe_inject_auto_attach_producer_crash(stage: &str) -> Result<()> {
    let injected = AUTO_ATTACH_PRODUCER_FAULT.take();
    if injected == Some(stage) {
        anyhow::bail!("injected router auto-attach producer crash {stage}");
    }
    AUTO_ATTACH_PRODUCER_FAULT.set(injected);
    Ok(())
}

#[cfg(all(target_os = "linux", not(test)))]
fn maybe_inject_auto_attach_producer_crash(_stage: &str) -> Result<()> {
    Ok(())
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
) -> Result<SessionAutoAttachSettleResult> {
    #[cfg(target_os = "linux")]
    {
        if resolve_post_hsa_auto_attach_authority_context(store, orchestration_session_id)?
            .is_some()
        {
            let obligation = store
                .load_obligation(orchestration_session_id, obligation_id)?
                .ok_or_else(|| anyhow::anyhow!("exact auto-attach obligation is absent"))?;
            let claim_owner = obligation.attach_claim_owner.as_deref().ok_or_else(|| {
                anyhow::anyhow!("post-HSA auto-attach obligation has no claim owner")
            })?;
            return settle_post_hsa_auto_attach_failed_closed(
                store,
                orchestration_session_id,
                obligation_id,
                claim_owner,
                reason,
            )?
            .ok_or_else(|| anyhow::anyhow!("post-HSA auto-attach authority disappeared"));
        }
    }
    store.settle_session_auto_attach_failed_closed(orchestration_session_id, reason)
}

fn mark_exact_obligation_failed_closed(
    store: &AgentRuntimeStateStore,
    orchestration_session_id: &str,
    obligation_id: &str,
    reason: &str,
) -> Result<SessionAutoAttachSettleResult> {
    #[cfg(target_os = "linux")]
    {
        if resolve_post_hsa_auto_attach_authority_context(store, orchestration_session_id)?
            .is_some()
        {
            let obligation = store
                .load_obligation(orchestration_session_id, obligation_id)?
                .ok_or_else(|| anyhow::anyhow!("exact auto-attach obligation is absent"))?;
            let claim_owner = obligation.attach_claim_owner.as_deref().ok_or_else(|| {
                anyhow::anyhow!("post-HSA auto-attach obligation has no claim owner")
            })?;
            return settle_post_hsa_auto_attach_failed_closed(
                store,
                orchestration_session_id,
                obligation_id,
                claim_owner,
                reason,
            )?
            .ok_or_else(|| anyhow::anyhow!("post-HSA auto-attach authority disappeared"));
        }
    }
    store.settle_exact_session_auto_attach_obligation_failed_closed(
        orchestration_session_id,
        obligation_id,
        reason,
    )
}

#[cfg(target_os = "linux")]
#[derive(Debug)]
struct AuthenticatedAutoAttachCompletionV1 {
    authority_observation: AuthorityObservationV1,
    applied_intent: HostSessionTransitionIntentV3,
}

#[cfg(target_os = "linux")]
fn finalize_session_auto_attach_after_launch(
    store: &AgentRuntimeStateStore,
    orchestration_session_id: &str,
    obligation_id: String,
    attach_claim_owner: String,
    receipt: HiddenOwnerHelperLaunchReceipt,
    plan: &AuthorityManagedSuccessorLaunchPlanV1,
) -> Result<SessionAutoAttachExecution> {
    let completion = match ensure_auto_attach_restored_session(
        store,
        orchestration_session_id,
        &receipt.participant_id,
        plan,
        &obligation_id,
        &attach_claim_owner,
    ) {
        Ok(completion) => completion,
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
    let snapshot =
        capture_post_hsa_auto_attach_ledger_snapshot(store, &completion.authority_observation)?;
    let outcome = settle_post_hsa_auto_attach_obligation(
        store,
        &PostHsaAutoAttachSettlementRequestV1 {
            expected_ledger: snapshot,
            obligation_id: obligation_id.clone(),
            claim_owner: attach_claim_owner.clone(),
            completion_reason: ROUTER_AUTO_ATTACH_RESTORED_REASON.to_string(),
            settlement_kind: PostHsaAutoAttachSettlementKindV1::Satisfied,
            router_auto_attach_intent: Some(completion.applied_intent),
            settled_at: chrono::Utc::now(),
        },
    )?;
    let result = match outcome {
        PostHsaAutoAttachSettlementOutcomeV1::Settled(result)
        | PostHsaAutoAttachSettlementOutcomeV1::Joined(result) => result,
    };
    if result.satisfied_obligation_ids != [obligation_id.as_str()] {
        anyhow::bail!(
            "authenticated router auto-attach completion did not settle its exact obligation"
        );
    }
    let settled = session_auto_attach_settle_result_from_post_hsa(result);

    Ok(SessionAutoAttachExecution::Attached {
        obligation_id,
        attach_claim_owner,
        receipt,
        settled,
    })
}

#[cfg(not(target_os = "linux"))]
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

#[cfg(target_os = "linux")]
fn ensure_auto_attach_restored_session(
    store: &AgentRuntimeStateStore,
    orchestration_session_id: &str,
    participant_id: &str,
    plan: &AuthorityManagedSuccessorLaunchPlanV1,
    obligation_id: &str,
    attach_claim_owner: &str,
) -> Result<AuthenticatedAutoAttachCompletionV1> {
    if plan.applied_transition.orchestration_session_id != orchestration_session_id
        || plan.applied_transition.target_authoritative_participant_id != participant_id
    {
        anyhow::bail!(
            "owner_unreachable: router auto-attach receipt does not match its exact HSA transition"
        );
    }
    validate_auto_attach_launch_plan(store, plan, obligation_id, attach_claim_owner)?;
    let authority = auto_attach_authority(store)?;
    let root = authority
        .read_a12b_root()
        .map_err(|error| anyhow::anyhow!(error.to_string()))?;
    let current = root
        .successor_transition_intent_map
        .get(&plan.applied_transition.intent_id)
        .ok_or_else(|| {
            anyhow::anyhow!("owner_unreachable: router auto-attach intent disappeared")
        })?;
    let HostSessionTransitionIntentStateV3::Applied {
        startup_ownership, ..
    } = &current.state
    else {
        anyhow::bail!("owner_unreachable: router auto-attach intent is no longer Applied");
    };
    if !matches!(
        startup_ownership.as_ref(),
        HostSessionStartupOwnershipApplicationV1::Accepted { .. }
    ) {
        anyhow::bail!(
            "owner_unreachable: router auto-attach lacks exact authenticated HSA completion"
        );
    }
    validate_auto_attach_application_identity(&plan.applied_transition, current)?;
    let resolved = authority
        .resolve_current_exact(orchestration_session_id, None)
        .map_err(|error| anyhow::anyhow!(error.to_string()))?;
    if resolved.authority.lifecycle_posture != HostSessionPostureV1::ActiveAttached
        || resolved
            .authority
            .active_authoritative_participant_id
            .as_deref()
            != Some(participant_id)
    {
        anyhow::bail!(
            "owner_unreachable: router auto-attach exact HSA completion did not restore ownership"
        );
    }
    Ok(AuthenticatedAutoAttachCompletionV1 {
        authority_observation: resolved.observation,
        applied_intent: current.clone(),
    })
}

#[cfg(not(target_os = "linux"))]
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
        OrchestrationObligationReviewState,
    };
    use crate::execution::agent_runtime::orchestration_session::{
        HostAttachContract, OrchestrationSessionPosture,
    };
    #[cfg(not(target_os = "linux"))]
    use crate::execution::agent_runtime::orchestration_session::{
        HostAttachExecutionClientStart, HostAttachLaunchKnobs, HostAttachModePreference,
    };
    use crate::execution::agent_runtime::validator::RuntimeSelectionDescriptor;
    use crate::execution::agent_runtime::{
        AgentRuntimeParticipantRecord, AgentRuntimeSessionState, OrchestrationSessionRecord,
        OrchestrationSessionState, PURE_AGENT_PROTOCOL,
    };

    fn with_store(test: impl FnOnce(&AgentRuntimeStateStore)) {
        use std::os::unix::fs::PermissionsExt;

        let authority_env = crate::execution::AuthorityEnvTestGuard::preserve();
        let safe_parent = std::env::var_os("XDG_RUNTIME_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|| {
                PathBuf::from(std::env::var_os("HOME").expect("tests require HOME")).join(".cache")
            });
        fs::create_dir_all(&safe_parent).expect("safe authority test parent");
        let temp = tempfile::Builder::new()
            .prefix("substrate-auto-attach-")
            .tempdir_in(safe_parent)
            .expect("private authority tempdir");
        fs::set_permissions(temp.path(), fs::Permissions::from_mode(0o700))
            .expect("private authority tempdir mode");
        authority_env.install_home(temp.path());
        let store = AgentRuntimeStateStore::new().expect("state store");
        test(&store);
    }

    #[cfg(target_os = "linux")]
    fn hsa_test_timestamp(
        value: &str,
    ) -> crate::execution::agent_runtime::host_session_authority::schema::TimestampV1 {
        crate::execution::agent_runtime::host_session_authority::schema::TimestampV1::parse(value)
            .expect("valid HSA test timestamp")
    }

    #[cfg(target_os = "linux")]
    fn seed_post_hsa_auto_attach_ledger(
        store: &AgentRuntimeStateStore,
        orchestration_session_id: &str,
        obligations: &[(&str, OrchestrationObligationKind)],
    ) {
        use crate::execution::agent_runtime::obligation_ledger::{
            obligation_payload_commitment, obligation_snapshot_record_commitment,
            ObligationLedgerRevisionCursorV1, ObligationLedgerSessionStateV1,
            SupervisorJournalEventRefV1,
        };
        use crate::execution::agent_runtime::state_store::AcceptedWorldWorkIdentityV1;
        use std::os::unix::fs::PermissionsExt as _;

        let root = auto_attach_authority(store)
            .expect("open ledger fixture authority")
            .bootstrap()
            .expect("bootstrap ledger fixture authority");
        let authority_store_id = root.authority_store_id;
        let participant_id = format!("participant-start-{orchestration_session_id}");
        let acceptance_record_id = format!("acceptance-{orchestration_session_id}");
        let stream_id = format!("stream-{orchestration_session_id}");
        let accepted_work_identity = AcceptedWorldWorkIdentityV1::EphemeralTask {
            task_run_id: format!("task-{orchestration_session_id}"),
        };

        for (index, (obligation_id, kind)) in obligations.iter().enumerate() {
            let sequence = (index + 1) as u64;
            let event_id = format!("event-{orchestration_session_id}-{sequence}");
            let source_journal_event = SupervisorJournalEventRefV1 {
                schema_version: 1,
                journal_entry_id: format!("journal-{orchestration_session_id}-{sequence}"),
                acceptance_record_id: acceptance_record_id.clone(),
                acceptance_record_revision: 1,
                accepted_work_identity: accepted_work_identity.clone(),
                stream_id: stream_id.clone(),
                frame_sequence: sequence,
                event_id: event_id.clone(),
                event_sequence: sequence,
                transport_event_commitment:
                    crate::execution::agent_runtime::host_session_authority::schema::AuthorityObjectCommitmentV1::CanonicalSha256 {
                        digest_hex: format!("{sequence:064x}"),
                    },
            };
            let payload = serde_json::json!({
                "obligation_id": obligation_id,
                "sequence": sequence,
            });
            let mut obligation = OrchestrationObligationRecord::new(
                orchestration_session_id,
                *obligation_id,
                *kind,
                format!("post-HSA auto-attach fixture {obligation_id}"),
            );
            obligation.attention_required = true;
            obligation.attach_state = OrchestrationObligationAttachState::Eligible;
            obligation.authority_store_id = Some(authority_store_id.clone());
            obligation.authoritative_participant_id = Some(participant_id.clone());
            obligation.obligation_revision = Some(1);
            obligation.source_journal_event = Some(source_journal_event.clone());
            obligation.payload_commitment =
                Some(obligation_payload_commitment(&payload).expect("payload commitment"));
            obligation.canonical_record_commitment = Some(
                obligation_snapshot_record_commitment(
                    &authority_store_id,
                    orchestration_session_id,
                    &participant_id,
                    &source_journal_event,
                    obligation_id,
                    1,
                )
                .expect("record commitment"),
            );
            obligation.causation_event_id = Some(event_id);
            obligation.source_participant_id = Some(participant_id.clone());
            obligation.payload = Some(payload);
            obligation
                .validate()
                .expect("valid ledger fixture obligation");
            let path = store.canonical_obligation_ledger_obligation_path(
                orchestration_session_id,
                &acceptance_record_id,
                obligation_id,
            );
            fs::create_dir_all(path.parent().expect("obligation parent"))
                .expect("create obligation fixture parent");
            fs::write(
                &path,
                serde_json::to_vec_pretty(&obligation).expect("serialize obligation fixture"),
            )
            .expect("write obligation fixture");
            fs::set_permissions(&path, fs::Permissions::from_mode(0o600))
                .expect("secure obligation fixture mode");
        }

        let state = ObligationLedgerSessionStateV1 {
            schema_version: 1,
            authority_store_id,
            orchestration_session_id: orchestration_session_id.to_string(),
            authoritative_participant_id: participant_id,
            acceptance_record_id: acceptance_record_id.clone(),
            acceptance_record_revision: 1,
            accepted_work_identity,
            stream_id,
            host_transition_correlation: None,
            authority_revision_observed: 1,
            session_ledger_revision: 1,
            materialized_through_event_sequence: obligations.len() as u64,
            terminal_event_id: None,
            terminal_event_sequence: None,
        };
        state.validate().expect("valid ledger fixture state");
        let state_path = store.canonical_obligation_ledger_state_path(
            orchestration_session_id,
            &acceptance_record_id,
        );
        fs::create_dir_all(state_path.parent().expect("state parent"))
            .expect("create state fixture parent");
        fs::write(
            &state_path,
            serde_json::to_vec_pretty(&state).expect("serialize state fixture"),
        )
        .expect("write state fixture");
        fs::set_permissions(&state_path, fs::Permissions::from_mode(0o600))
            .expect("secure state fixture mode");
        let cursor_path =
            store.canonical_obligation_ledger_revision_cursor_path(orchestration_session_id);
        fs::write(
            &cursor_path,
            serde_json::to_vec_pretty(&ObligationLedgerRevisionCursorV1 {
                schema_version: 1,
                current_session_ledger_revision: 1,
            })
            .expect("serialize cursor fixture"),
        )
        .expect("write cursor fixture");
        fs::set_permissions(cursor_path, fs::Permissions::from_mode(0o600))
            .expect("secure cursor fixture mode");
    }

    #[cfg(target_os = "linux")]
    fn seed_settled_auto_attach_authority(
        store: &AgentRuntimeStateStore,
        orchestration_session_id: &str,
        world_binding: Option<
            crate::execution::agent_runtime::host_session_authority::schema::WorldBindingV1,
        >,
    ) {
        use crate::execution::agent_runtime::host_session_authority::schema::*;
        use crate::execution::agent_runtime::host_session_authority::start_continuity::*;
        use crate::execution::agent_runtime::host_session_authority::store_schema::*;
        use crate::execution::agent_runtime::host_session_authority::transition::*;

        let authority = auto_attach_authority(store).expect("open auto-attach HSA authority");
        let root = authority
            .bootstrap()
            .expect("bootstrap auto-attach authority");
        let workspace_binding = WorkspaceBindingV1 {
            workspace_root: root.bootstrap_home.clone(),
            authority_store_root: root.bootstrap_home.clone(),
            authority_store_id: root.authority_store_id,
        };
        let source_participant_id = format!("participant-start-{orchestration_session_id}");
        let start_intent_id = format!("intent-start-{orchestration_session_id}");
        let start_request_id = format!("request-start-{orchestration_session_id}");
        let start_run_id = format!("run-start-{orchestration_session_id}");
        let start_transaction_id = format!("stx-{orchestration_session_id}");
        let request = IssueHostSessionTransitionRequestV1 {
            intent_id: start_intent_id.clone(),
            issuer_request_id: start_request_id.clone(),
            mode: HostSessionTransitionModeV1::Start,
            authority_precondition: HostSessionAuthorityPreconditionV1::ExpectedAbsent,
            orchestration_session_id: orchestration_session_id.to_string(),
            shell_trace_session_id: format!("trace-{orchestration_session_id}"),
            caller: HostSessionTransitionCallerV1 {
                kind: HostSessionTransitionCallerKindV1::PublicCli,
                caller_participant_id: None,
                auto_attach_obligation_id: None,
                auto_attach_claim_owner: None,
            },
            source_authoritative_participant_id: None,
            target_authoritative_participant_id: source_participant_id.clone(),
            target_participant_lease_token: format!("lease-{orchestration_session_id}")
                .into_bytes(),
            run_id: start_run_id.clone(),
            resulting_authoritative_lineage: vec![source_participant_id.clone()],
            workspace_binding: workspace_binding.clone(),
            world_binding: world_binding.clone(),
            start_contract: StartContractMaterialV1 {
                descriptor: AgentDescriptorV1 {
                    schema_version: 1,
                    agent_id: "codex".into(),
                    backend_id: "cli:codex-host".into(),
                    backend_kind: RuntimeBackendKindV1::Codex,
                    protocol: "substrate.agent.session".into(),
                    execution_scope: AgentExecutionScopeV1::Host,
                    binary_path: "/usr/bin/codex".into(),
                },
                policy: PolicyObjectHashInputV1 {
                    schema_version: 1,
                    policy_revision: "policy-auto-attach".into(),
                    canonical_policy_snapshot_sha256: "a".repeat(64),
                },
                capabilities: HostAttachCapabilitiesV1 {
                    session_resume: true,
                    session_fork: true,
                    session_stop: true,
                    status_snapshot: true,
                    event_stream: true,
                },
                launch_knobs: HostAttachLaunchKnobsV1 {
                    requested_execution_scope: AgentExecutionScopeV1::Host,
                    host_execution_client_start: HostAttachExecutionClientStartV1::StartNow,
                    attach_mode_preference: HostAttachModePreferenceV1::ContinuityPreferred,
                },
            },
            transition_input: None,
        };
        let TransitionIssueOutcomeV1::Issued(issued) = authority
            .issue_start_at(
                &request,
                hsa_test_timestamp("2026-08-29T12:00:00.000000000Z"),
                300,
            )
            .expect("issue seed Start")
        else {
            panic!("seed Start must issue");
        };
        let claim_request = ClaimHostSessionTransitionRequestV1 {
            intent_id: issued.intent_id.clone(),
            issuer_request_id: issued.issuer_request_id.clone(),
            payload_commitment: issued.payload_commitment.clone(),
            expected_intent_revision: issued.intent_revision,
            claim_id: format!("claim-start-{orchestration_session_id}"),
            claimant_attempt_id: format!("attempt-start-{orchestration_session_id}"),
        };
        let TransitionClaimOutcomeV1::Claimed(claimed) = authority
            .claim_start_at(
                &claim_request,
                hsa_test_timestamp("2026-08-29T12:00:01.000000000Z"),
                30,
            )
            .expect("claim seed Start")
        else {
            panic!("seed Start must claim");
        };
        let HostSessionTransitionIntentStateV2::Claimed { claim_revision, .. } = claimed.state
        else {
            panic!("seed Start claim must retain claim evidence");
        };
        let TransitionApplicationOutcomeV1::Applied(applied) = authority
            .apply_start_at(
                &ApplyHostSessionTransitionRequestV1 {
                    intent_id: claimed.intent_id,
                    issuer_request_id: claimed.issuer_request_id,
                    payload_commitment: claimed.payload_commitment,
                    expected_intent_revision: claimed.intent_revision,
                    claim_id: claim_request.claim_id,
                    expected_claim_revision: claim_revision,
                },
                hsa_test_timestamp("2026-08-29T12:00:02.000000000Z"),
            )
            .expect("apply seed Start")
        else {
            panic!("seed Start must apply");
        };
        let HostSessionTransitionIntentStateV2::Applied {
            application_result_ref,
            authority_revision_after,
            ..
        } = &applied.state
        else {
            panic!("seed Start must remain applied");
        };
        let root = authority.read_a12a_root().expect("read applied Start root");
        let registration = EstablishStartContinuationRequestV1 {
            start_transaction_id: start_transaction_id.clone(),
            request_key_sha256: "b".repeat(64),
            authority_store_id: root.authority_store_id.clone(),
            orchestration_session_id: orchestration_session_id.to_string(),
            intent_id: start_intent_id.clone(),
            issuer_request_id: start_request_id.clone(),
            payload_commitment: applied.payload_commitment.clone(),
            application_result_ref: application_result_ref.clone(),
            run_id: start_run_id.clone(),
            expected_authority_revision: *authority_revision_after,
            authoritative_participant_id: source_participant_id.clone(),
            backend_id: request.start_contract.descriptor.backend_id.clone(),
            protocol: request.start_contract.descriptor.protocol.clone(),
            exchange_id: format!("exchange-{orchestration_session_id}"),
            exchange_sequence: 1,
            provider_event_kind: "turn.exchange_opened".into(),
            exchange_evidence_sha256: "c".repeat(64),
            internal_uaa_session_id: format!("uaa-{orchestration_session_id}"),
            observed_at: hsa_test_timestamp("2026-08-29T12:00:04.000000000Z"),
        };
        let transaction = StartTransactionRecordV1 {
            schema_version: 1,
            transaction_id: start_transaction_id,
            request_key_sha256: registration.request_key_sha256.clone(),
            prompt_sha256: "d".repeat(64),
            authority_store_id: registration.authority_store_id.clone(),
            orchestration_session_id: orchestration_session_id.to_string(),
            shell_trace_session_id: request.shell_trace_session_id.clone(),
            authoritative_participant_id: source_participant_id.clone(),
            backend_id: registration.backend_id.clone(),
            protocol: registration.protocol.clone(),
            workspace_root: workspace_binding.workspace_root.physical_path.clone(),
            world_id: world_binding
                .as_ref()
                .map(|binding| binding.world_id.clone()),
            world_generation: world_binding
                .as_ref()
                .map(|binding| binding.world_generation),
            public_backend_id: registration.backend_id.clone(),
            public_scope: "host".into(),
            start_intent_id,
            start_issuer_request_id: start_request_id,
            start_payload_commitment: registration.payload_commitment.clone(),
            start_application_result_ref: registration.application_result_ref.clone(),
            start_run_id,
            start_authority_revision: registration.expected_authority_revision,
            created_at: hsa_test_timestamp("2026-08-29T12:00:02.000000000Z"),
            updated_at: hsa_test_timestamp("2026-08-29T12:00:02.000000000Z"),
            state: StartTransactionStateV1::PromptNotSubmitted,
        };
        assert!(matches!(
            authority
                .begin_start_transaction(&transaction)
                .expect("begin Start transaction"),
            StartTransactionBeginOutcomeV1::Applied(_)
        ));
        authority
            .mark_start_prompt_submission_no_replay_barrier(
                &registration.start_transaction_id,
                &registration.request_key_sha256,
                hsa_test_timestamp("2026-08-29T12:00:03.000000000Z"),
            )
            .expect("commit Start no-replay barrier");
        let StartContinuationOutcomeV1::Applied(registered) = authority
            .establish_start_continuation(&registration)
            .expect("establish seed Start continuation")
        else {
            panic!("seed Start continuation must apply");
        };
        let settlement = SettleStartTurnRequestV1 {
            start_transaction_id: registration.start_transaction_id.clone(),
            request_key_sha256: registration.request_key_sha256.clone(),
            authority_store_id: registration.authority_store_id.clone(),
            orchestration_session_id: orchestration_session_id.to_string(),
            intent_id: registration.intent_id.clone(),
            issuer_request_id: registration.issuer_request_id.clone(),
            payload_commitment: registration.payload_commitment.clone(),
            application_result_ref: registration.application_result_ref.clone(),
            run_id: registration.run_id.clone(),
            authoritative_participant_id: source_participant_id.clone(),
            backend_id: registration.backend_id.clone(),
            protocol: registration.protocol.clone(),
            registered_resume_handle_ref: registered.resume_handle_ref,
            expected_authority_revision: registered.authority_revision_after,
            protocol_actor: HostPostTurnProtocolActorV1::TargetAuthoritativeParticipant {
                participant_id: source_participant_id,
            },
            event_id: format!("event-{orchestration_session_id}"),
            event_sequence: 2,
            provider_event_kind: "turn.completed".into(),
            thread_id: registration.internal_uaa_session_id,
            turn_id: format!("turn-{orchestration_session_id}"),
            completion_evidence_sha256: "e".repeat(64),
            kind: StartTurnCompletionKindV1::ResumableClean,
            obligation_ledger_read: None,
            completed_at: hsa_test_timestamp("2026-08-29T12:00:05.000000000Z"),
        };
        assert!(matches!(
            authority
                .settle_start_turn(&settlement)
                .expect("settle seed Start turn"),
            StartTurnSettlementOutcomeV1::Applied(_)
        ));
        let settled = authority
            .resolve_current_exact(orchestration_session_id, None)
            .expect("resolve settled seed Start authority");
        assert_eq!(
            settled.authority.lifecycle_posture,
            HostSessionPostureV1::ParkedResumable
        );
    }

    #[cfg(target_os = "linux")]
    fn settle_auto_attach_startup_ownership(
        store: &AgentRuntimeStateStore,
        plan: &AuthorityManagedSuccessorLaunchPlanV1,
    ) {
        use crate::execution::agent_runtime::host_session_authority::schema::{
            HostStartupOwnershipProtocolActorV1, HostStartupOwnershipProtocolEventV1,
        };
        use crate::execution::agent_runtime::host_session_authority::transition::*;

        let authority = auto_attach_authority(store).expect("open auto-attach HSA authority");
        let HostSessionTransitionIntentStateV3::Applied { applied_at, .. } =
            &plan.applied_transition.state
        else {
            panic!("router auto-attach test plan must retain its applied lifecycle")
        };
        let request = ResolveStartupOwnershipRequestV1 {
            intent_id: plan.applied_transition.intent_id.clone(),
            issuer_request_id: plan.applied_transition.issuer_request_id.clone(),
            payload_commitment: plan.applied_transition.payload_commitment.clone(),
            protocol_actor: HostStartupOwnershipProtocolActorV1::TargetAuthoritativeParticipant {
                participant_id: plan
                    .applied_transition
                    .target_authoritative_participant_id
                    .clone(),
            },
            protocol_event: HostStartupOwnershipProtocolEventV1::OwnershipAccepted {
                ownership_acknowledgement_id: format!(
                    "auto-attach-accepted-{}",
                    plan.applied_transition.intent_id
                ),
            },
            observed_at: applied_at.clone(),
        };
        assert!(matches!(
            authority
                .resolve_startup_ownership_at(&request, applied_at.clone())
                .expect("settle router auto-attach startup ownership"),
            StartupOwnershipResolutionOutcomeV1::ResolvedSuccessor(_)
        ));
    }

    #[cfg(target_os = "linux")]
    fn auto_attach_world_binding(
    ) -> crate::execution::agent_runtime::host_session_authority::schema::WorldBindingV1 {
        crate::execution::agent_runtime::host_session_authority::schema::WorldBindingV1 {
            world_id: "world-auto-attach".into(),
            world_generation: 17,
        }
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

    #[cfg(not(target_os = "linux"))]
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

    #[test]
    fn ensure_router_auto_attach_targets_exact_local_host_allows_untargeted_obligations() {
        let obligation = eligible_obligation(
            "sess_auto_attach_untargeted",
            "obl_untargeted",
            OrchestrationObligationKind::ApprovalRequired,
        );
        ensure_router_auto_attach_targets_exact_local_host(&obligation, "host-local")
            .expect("untargeted obligations should preserve packet two behavior");
    }

    #[test]
    fn ensure_router_auto_attach_targets_exact_local_host_allows_same_host_targets() {
        let mut obligation = eligible_obligation(
            "sess_auto_attach_local_target",
            "obl_local_target",
            OrchestrationObligationKind::ApprovalRequired,
        );
        obligation.target_host_id = Some("host-local".to_string());

        ensure_router_auto_attach_targets_exact_local_host(&obligation, "host-local")
            .expect("same-host targeted obligations should remain attach-eligible");
    }

    #[test]
    fn ensure_router_auto_attach_targets_exact_local_host_rejects_foreign_targets() {
        let mut obligation = eligible_obligation(
            "sess_auto_attach_foreign_target",
            "obl_foreign_target",
            OrchestrationObligationKind::ApprovalRequired,
        );
        obligation.target_host_id = Some("host-remote".to_string());

        let err = ensure_router_auto_attach_targets_exact_local_host(&obligation, "host-local")
            .expect_err("foreign-targeted obligations must fail closed on the local router");
        assert!(
            err.to_string().contains(
                "wrong_target_host: obligation obl_foreign_target targets host host-remote not local host host-local"
            ),
            "wrong-host errors should stay explanation-ready: {err:#}"
        );
    }

    fn guaranteed_foreign_target_host_id() -> String {
        format!(
            "{}::foreign",
            resolve_router_auto_attach_local_host_id()
                .expect("resolve deterministic local host id")
        )
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
    fn execute_session_auto_attach_fails_closed_before_launch_when_target_host_is_foreign() {
        with_store(|store| {
            let (session, participant) =
                detached_orchestrator("sess_auto_attach_wrong_host", "ash_auto_attach_wrong_host");
            store
                .persist_orchestration_session(&session)
                .expect("persist session");
            store
                .persist_participant(&participant)
                .expect("persist participant");

            let mut wrong_host = eligible_obligation(
                "sess_auto_attach_wrong_host",
                "obl_wrong_host",
                OrchestrationObligationKind::ApprovalRequired,
            );
            wrong_host.target_host_id = Some(guaranteed_foreign_target_host_id());
            let sibling = eligible_obligation(
                "sess_auto_attach_wrong_host",
                "obl_local_sibling",
                OrchestrationObligationKind::FollowUpRequired,
            );
            store
                .persist_obligation(&wrong_host)
                .expect("persist wrong-host obligation");
            store
                .persist_obligation(&sibling)
                .expect("persist eligible sibling obligation");

            let execution = execute_session_auto_attach(
                store,
                "sess_auto_attach_wrong_host",
                "router::local",
                &router_auto_attach_policy(OrchestrationObligationKind::ApprovalRequired),
                false,
                false,
            )
            .expect("wrong-host auto attach should fail closed, not error");
            let SessionAutoAttachExecution::FailedClosed {
                reason, settled, ..
            } = execution
            else {
                panic!("expected wrong-host automatic attach to fail closed");
            };
            assert!(
                reason.contains("wrong_target_host"),
                "wrong-host failure should preserve an explanation-ready reason: {reason}"
            );
            assert_eq!(
                settled.failed_closed_obligation_ids,
                vec!["obl_wrong_host".to_string()]
            );

            let wrong_host = store
                .load_obligation("sess_auto_attach_wrong_host", "obl_wrong_host")
                .expect("reload wrong-host obligation")
                .expect("wrong-host obligation exists after failed-close");
            assert_eq!(
                wrong_host.attach_state,
                OrchestrationObligationAttachState::FailedClosed
            );
            assert_eq!(
                wrong_host.attach_completion_reason.as_deref(),
                Some(reason.as_str())
            );
            assert_eq!(wrong_host.attach_claim_owner, None);
            assert_eq!(wrong_host.attach_attempt_count, 0);
            assert_eq!(wrong_host.attach_last_attempt_at, None);
            assert_eq!(
                wrong_host.review_state,
                OrchestrationObligationReviewState::Unread
            );

            let sibling = store
                .load_obligation("sess_auto_attach_wrong_host", "obl_local_sibling")
                .expect("reload sibling obligation")
                .expect("sibling obligation exists after wrong-host failed-close");
            assert_eq!(
                sibling.attach_state,
                OrchestrationObligationAttachState::Eligible
            );
            assert_eq!(sibling.attach_completion_reason, None);
            assert_eq!(
                sibling.review_state,
                OrchestrationObligationReviewState::Unread
            );
            assert!(
                store
                    .list_router_auto_attach_candidate_session_ids()
                    .expect("list candidates after wrong-host failed-close")
                    .contains(&"sess_auto_attach_wrong_host".to_string()),
                "preserved sibling obligations should keep the session eligible for later router work"
            );
        });
    }

    #[test]
    #[serial_test::serial]
    #[cfg(not(target_os = "linux"))]
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
    #[cfg(target_os = "linux")]
    fn auto_attach_launch_plan_prefers_continuity_when_available() {
        with_store(|store| {
            use crate::execution::agent_runtime::host_session_authority::schema::{
                AuthorityObjectCommitmentV1, HostSessionAuthorityPreconditionV1,
            };
            use crate::execution::agent_runtime::host_session_authority::store_schema::{
                HostSessionPostTurnApplicationV2, HostSessionStartupOwnershipApplicationV1,
                HostSessionTransitionInputHandoffV1, HostSessionTransitionIntentStateV3,
            };

            let world_binding = auto_attach_world_binding();
            seed_settled_auto_attach_authority(
                store,
                "sess_auto_attach_continuity",
                Some(world_binding.clone()),
            );
            let authority = auto_attach_authority(store).expect("open auto-attach authority");
            let before = authority
                .resolve_current_exact("sess_auto_attach_continuity", None)
                .expect("resolve source authority");

            let plan = build_auto_attach_launch_plan(
                store,
                "sess_auto_attach_continuity",
                "obl_auto_attach_continuity",
                "router::auto-attach",
            )
            .expect("build auto attach plan");
            assert_eq!(plan.helper_plan.mode, OwnerHelperMode::Attach);
            assert_eq!(
                plan.applied_transition.mode,
                HostSessionTransitionModeV1::Attach
            );
            assert_eq!(
                plan.applied_transition.caller.kind,
                crate::execution::agent_runtime::host_session_authority::schema::HostSessionTransitionCallerKindV1::RouterAutoAttach
            );
            assert_eq!(
                plan.applied_transition
                    .caller
                    .auto_attach_obligation_id
                    .as_deref(),
                Some("obl_auto_attach_continuity")
            );
            assert_eq!(
                plan.applied_transition
                    .caller
                    .auto_attach_claim_owner
                    .as_deref(),
                Some("router::auto-attach")
            );
            assert!(plan
                .applied_transition
                .caller
                .caller_participant_id
                .is_none());
            assert_eq!(
                plan.applied_transition.source_authoritative_participant_id,
                before.authority.active_authoritative_participant_id
            );
            assert_eq!(
                plan.applied_transition.workspace_binding,
                before.authority.workspace_binding
            );
            assert_eq!(
                plan.applied_transition.world_binding.as_ref(),
                Some(&world_binding)
            );
            assert_eq!(
                plan.applied_transition.descriptor_ref,
                before.caller.descriptor_ref
            );
            assert_eq!(
                plan.applied_transition.host_attach_contract_ref,
                before
                    .authority
                    .host_attach_contract_ref
                    .clone()
                    .expect("source attach contract")
            );
            let HostSessionAuthorityPreconditionV1::ExpectedRevision {
                authority_revision,
                authority_record_commitment,
                active_authoritative_participant_id,
                authoritative_lineage_commitment,
                lifecycle_posture,
            } = &plan.applied_transition.authority_precondition
            else {
                panic!("router auto-attach must bind an exact authority revision");
            };
            assert_eq!(*authority_revision, before.observation.authority_revision);
            assert_eq!(
                authority_record_commitment,
                &before.observation.authority_record_commitment
            );
            assert_eq!(
                active_authoritative_participant_id,
                before
                    .authority
                    .active_authoritative_participant_id
                    .as_deref()
                    .expect("source participant")
            );
            assert_eq!(
                authoritative_lineage_commitment,
                &before.observation.authoritative_lineage_commitment
            );
            assert_eq!(*lifecycle_posture, HostSessionPostureV1::ParkedResumable);
            assert!(matches!(
                plan.applied_transition.payload_commitment,
                AuthorityObjectCommitmentV1::CanonicalSha256 { ref digest_hex }
                    if digest_hex.len() == 64
            ));
            assert!(plan.applied_transition.resume_handle_ref.is_none());
            assert!(plan.applied_transition.transition_input_ref.is_none());
            assert!(plan.applied_transition.post_turn_disposition.is_none());
            assert!(matches!(
                plan.applied_transition.input_handoff,
                HostSessionTransitionInputHandoffV1::NotApplicable
            ));
            let HostSessionTransitionIntentStateV3::Applied {
                claim_id,
                claimant_attempt_id,
                startup_ownership,
                post_turn,
                ..
            } = &plan.applied_transition.state
            else {
                panic!("router auto-attach plan must carry an applied result");
            };
            assert!(claim_id.starts_with("claim_router_auto_attach_"));
            assert!(claimant_attempt_id.starts_with("attempt_router_auto_attach_"));
            assert!(matches!(
                startup_ownership.as_ref(),
                HostSessionStartupOwnershipApplicationV1::Pending { .. }
            ));
            assert!(matches!(
                post_turn.as_ref(),
                HostSessionPostTurnApplicationV2::NotApplicable
            ));
            assert!(plan.helper_plan.startup_prompt.is_none());
            assert!(plan.helper_plan.host_attach_contract.is_none());
            assert!(plan.helper_plan.source_orchestration_session_id.is_none());
            assert_eq!(
                plan.helper_plan.session.world_id.as_deref(),
                Some("world-auto-attach")
            );
            assert_eq!(plan.helper_plan.session.world_generation, Some(17));
            assert_eq!(
                plan.helper_plan
                    .participant
                    .internal_uaa_session_id
                    .as_deref(),
                Some("uaa-sess_auto_attach_continuity")
            );
            assert!(plan.helper_plan.requires_internal_session_id());
            assert_eq!(
                plan.helper_plan
                    .participant
                    .resumed_from_participant_id
                    .as_deref(),
                Some("participant-start-sess_auto_attach_continuity")
            );
        });
    }

    #[test]
    #[serial_test::serial]
    #[cfg(target_os = "linux")]
    fn auto_attach_exact_retry_and_restart_join_one_applied_intent() {
        for stage in ["after_issue", "after_claim", "after_apply"] {
            with_store(|store| {
                let session_id = format!("sess_auto_attach_retry_{stage}");
                let obligation_id = format!("obl_auto_attach_retry_{stage}");
                seed_settled_auto_attach_authority(store, &session_id, None);
                inject_auto_attach_producer_fault_for_test(stage);
                let err = build_auto_attach_launch_plan(
                    store,
                    &session_id,
                    &obligation_id,
                    "router::retry",
                )
                .expect_err("fault must interrupt producer after durable stage");
                assert!(err.to_string().contains(stage));

                let authority = auto_attach_authority(store).expect("open retry authority");
                let interrupted = authority.read_a12b_root().expect("read interrupted root");
                assert_eq!(interrupted.successor_transition_intent_map.len(), 1);
                let interrupted_intent = interrupted
                    .successor_transition_intent_map
                    .values()
                    .next()
                    .expect("interrupted intent");
                assert_eq!(
                    interrupted_intent.caller.kind,
                    HostSessionTransitionCallerKindV1::RouterAutoAttach
                );
                match stage {
                    "after_issue" => assert!(matches!(
                        interrupted_intent.state,
                        HostSessionTransitionIntentStateV3::Issued
                    )),
                    "after_claim" => assert!(matches!(
                        interrupted_intent.state,
                        HostSessionTransitionIntentStateV3::Claimed { .. }
                    )),
                    "after_apply" => assert!(matches!(
                        interrupted_intent.state,
                        HostSessionTransitionIntentStateV3::Applied { .. }
                    )),
                    _ => unreachable!(),
                }

                let first = build_auto_attach_launch_plan(
                    store,
                    &session_id,
                    &obligation_id,
                    "router::retry",
                )
                .expect("retry must converge on durable application");
                let after_first = authority.read_a12b_root().expect("read applied retry root");
                assert_eq!(after_first.successor_transition_intent_map.len(), 1);
                let applied_revision = after_first.root_revision;
                let applied_intent = after_first
                    .successor_transition_intent_map
                    .get(&first.applied_transition.intent_id)
                    .expect("one applied retry intent")
                    .clone();

                let restarted = AgentRuntimeStateStore::new().expect("restart StateStore");
                let second = build_auto_attach_launch_plan(
                    &restarted,
                    &session_id,
                    &obligation_id,
                    "router::retry",
                )
                .expect("restart must exact-join applied result");
                let after_restart = authority.read_a12b_root().expect("read restart root");
                assert_eq!(after_restart.root_revision, applied_revision);
                assert_eq!(after_restart.successor_transition_intent_map.len(), 1);
                assert_eq!(
                    after_restart
                        .successor_transition_intent_map
                        .get(&first.applied_transition.intent_id),
                    Some(&applied_intent)
                );
                assert_eq!(second.helper_plan, first.helper_plan);
                validate_auto_attach_application_identity(
                    &first.applied_transition,
                    &second.applied_transition,
                )
                .expect("restart application identity");
                let current = authority
                    .resolve_current_exact(&session_id, None)
                    .expect("resolve retry authority");
                assert_eq!(
                    current
                        .authority
                        .authoritative_participant_lineage
                        .iter()
                        .filter(|participant_id| {
                            *participant_id == &first.helper_plan.participant.participant_id
                        })
                        .count(),
                    1
                );
            });
        }
    }

    #[test]
    #[serial_test::serial]
    #[cfg(target_os = "linux")]
    fn auto_attach_completion_is_exact_and_completed_retry_launches_no_helper() {
        with_store(|store| {
            seed_post_hsa_auto_attach_ledger(
                store,
                "sess_auto_attach_completion",
                &[(
                    "obl_auto_attach_completion",
                    OrchestrationObligationKind::Blocked,
                )],
            );
            seed_settled_auto_attach_authority(store, "sess_auto_attach_completion", None);
            let context = resolve_post_hsa_auto_attach_authority_context(
                store,
                "sess_auto_attach_completion",
            )
            .expect("resolve completion claim authority")
            .expect("completion authority is post-HSA");
            let claim = claim_post_hsa_session_auto_attach(
                store,
                "sess_auto_attach_completion",
                "router::completion",
                &Policy::default(),
                &context,
            )
            .expect("claim completion obligation");
            assert!(matches!(claim, SessionAutoAttachClaim::Claimed { .. }));
            let plan = build_auto_attach_launch_plan(
                store,
                "sess_auto_attach_completion",
                "obl_auto_attach_completion",
                "router::completion",
            )
            .expect("build completion plan");
            let pending_receipt = HiddenOwnerHelperLaunchReceipt {
                helper_pid: 41,
                orchestration_session_id: plan.helper_plan.session.orchestration_session_id.clone(),
                participant_id: plan.helper_plan.participant.participant_id.clone(),
                backend_id: plan.helper_plan.descriptor.backend_id.clone(),
            };
            let pending = ensure_auto_attach_restored_session(
                store,
                &pending_receipt.orchestration_session_id,
                &pending_receipt.participant_id,
                &plan,
                "obl_auto_attach_completion",
                "router::completion",
            )
            .expect_err("pending startup ownership cannot settle auto-attach");
            assert!(pending
                .to_string()
                .contains("lacks exact authenticated HSA completion"));
            let leader = acquire_authority_managed_successor_launch_permit(&plan)
                .expect("acquire pending launch permit");
            assert!(!leader.joined_completed_launch());
            drop(leader);

            settle_auto_attach_startup_ownership(store, &plan);
            assert_eq!(
                wait_for_authority_managed_successor_completion(&plan)
                    .expect("wait for exact auto-attach completion"),
                HostSessionPostureV1::ActiveAttached
            );
            ensure_auto_attach_restored_session(
                store,
                &pending_receipt.orchestration_session_id,
                &pending_receipt.participant_id,
                &plan,
                "obl_auto_attach_completion",
                "router::completion",
            )
            .expect("accepted startup ownership restores exact session");

            let joined = acquire_authority_managed_successor_launch_permit(&plan)
                .expect("join completed launch");
            assert!(joined.joined_completed_launch());
            let successor_launch =
                launch_authority_managed_successor_owner_helper(&plan, joined, false, true)
                    .expect("completed retry must join without spawning");
            assert_eq!(successor_launch.launch_receipt.helper_pid, 0);
            let execution = finalize_session_auto_attach_after_launch(
                store,
                "sess_auto_attach_completion",
                "obl_auto_attach_completion".into(),
                "router::completion".into(),
                successor_launch.launch_receipt.clone(),
                &plan,
            )
            .expect("finalize only after exact completion");
            let SessionAutoAttachExecution::Attached {
                receipt, settled, ..
            } = execution
            else {
                panic!("exact HSA completion must finalize Attached");
            };
            assert_eq!(receipt.helper_pid, 0);
            assert_eq!(
                settled.satisfied_obligation_ids,
                vec!["obl_auto_attach_completion".to_string()]
            );
            assert!(settled.failed_closed_obligation_ids.is_empty());

            let retry = build_auto_attach_launch_plan(
                store,
                "sess_auto_attach_completion",
                "obl_auto_attach_completion",
                "router::completion",
            )
            .expect("settled retry must join applied result");
            validate_auto_attach_application_identity(
                &plan.applied_transition,
                &retry.applied_transition,
            )
            .expect("settled retry immutable application");
            let root = auto_attach_authority(store)
                .expect("open completion authority")
                .read_a12b_root()
                .expect("read completion root");
            assert_eq!(root.successor_transition_intent_map.len(), 1);
            let current = auto_attach_authority(store)
                .expect("open completion authority")
                .resolve_current_exact("sess_auto_attach_completion", None)
                .expect("resolve completion authority");
            assert_eq!(
                current
                    .authority
                    .authoritative_participant_lineage
                    .iter()
                    .filter(|participant_id| {
                        *participant_id == &plan.helper_plan.participant.participant_id
                    })
                    .count(),
                1
            );
        });
    }

    #[test]
    #[serial_test::serial]
    #[cfg(target_os = "linux")]
    fn post_hsa_auto_attach_claim_and_failure_settlement_are_exact_and_convergent() {
        with_store(|store| {
            let session_id = "sess_post_hsa_claim_settle";
            let target_id = "obl_post_hsa_primary";
            let sibling_id = "obl_post_hsa_sibling";
            let owner = "router::post-hsa";
            seed_post_hsa_auto_attach_ledger(
                store,
                session_id,
                &[
                    (target_id, OrchestrationObligationKind::ApprovalRequired),
                    (sibling_id, OrchestrationObligationKind::FollowUpRequired),
                ],
            );
            seed_settled_auto_attach_authority(store, session_id, None);
            let context = resolve_post_hsa_auto_attach_authority_context(store, session_id)
                .expect("resolve post-HSA claim authority")
                .expect("post-HSA claim authority exists");
            let unclaimed_failure = mark_exact_obligation_failed_closed(
                store,
                session_id,
                target_id,
                "post_hsa_unclaimed_failure",
            )
            .expect_err("post-HSA failure settlement requires the exact claim owner");
            assert!(
                unclaimed_failure
                    .to_string()
                    .contains("has no claim owner"),
                "post-HSA failure must fail before invoking the legacy writer: {unclaimed_failure:#}"
            );
            let initial = capture_post_hsa_auto_attach_ledger_snapshot(store, &context.observation)
                .expect("capture initial post-HSA ledger");
            let request = PostHsaAutoAttachClaimRequestV1 {
                expected_ledger: initial.clone(),
                obligation_id: target_id.to_string(),
                claim_owner: owner.to_string(),
                claimed_at: chrono::Utc::now(),
            };
            let PostHsaAutoAttachClaimOutcomeV1::Claimed(claimed) =
                claim_post_hsa_auto_attach_obligation(store, &request)
                    .expect("claim exact post-HSA obligation")
            else {
                panic!("first post-HSA claim must mutate the exact obligation");
            };
            assert_eq!(claimed.attach_claim_owner.as_deref(), Some(owner));

            let joined_snapshot =
                capture_post_hsa_auto_attach_ledger_snapshot(store, &context.observation)
                    .expect("capture joined post-HSA ledger");
            let PostHsaAutoAttachClaimOutcomeV1::Joined(joined) =
                claim_post_hsa_auto_attach_obligation(
                    store,
                    &PostHsaAutoAttachClaimRequestV1 {
                        expected_ledger: joined_snapshot.clone(),
                        obligation_id: target_id.to_string(),
                        claim_owner: owner.to_string(),
                        claimed_at: chrono::Utc::now(),
                    },
                )
                .expect("exact duplicate claim joins")
            else {
                panic!("duplicate post-HSA claim must join");
            };
            assert_eq!(joined, claimed);

            assert!(claim_post_hsa_auto_attach_obligation(
                store,
                &PostHsaAutoAttachClaimRequestV1 {
                    expected_ledger: joined_snapshot.clone(),
                    obligation_id: target_id.to_string(),
                    claim_owner: "router::conflict".to_string(),
                    claimed_at: chrono::Utc::now(),
                },
            )
            .is_err());
            assert!(claim_post_hsa_auto_attach_obligation(
                store,
                &PostHsaAutoAttachClaimRequestV1 {
                    expected_ledger: joined_snapshot.clone(),
                    obligation_id: "obl_substituted".to_string(),
                    claim_owner: owner.to_string(),
                    claimed_at: chrono::Utc::now(),
                },
            )
            .is_err());
            let mut stale_snapshot = joined_snapshot.clone();
            stale_snapshot.authority_observation.root_revision -= 1;
            assert!(claim_post_hsa_auto_attach_obligation(
                store,
                &PostHsaAutoAttachClaimRequestV1 {
                    expected_ledger: stale_snapshot,
                    obligation_id: target_id.to_string(),
                    claim_owner: owner.to_string(),
                    claimed_at: chrono::Utc::now(),
                },
            )
            .is_err());
            let mut mismatched_observation = joined_snapshot;
            mismatched_observation
                .authority_observation
                .authority_revision += 1;
            assert!(claim_post_hsa_auto_attach_obligation(
                store,
                &PostHsaAutoAttachClaimRequestV1 {
                    expected_ledger: mismatched_observation,
                    obligation_id: target_id.to_string(),
                    claim_owner: owner.to_string(),
                    claimed_at: chrono::Utc::now(),
                },
            )
            .is_err());

            let authority_home = state_store_authority_home(store).expect("authority home");
            let guard_error = match crate::execution::agent_runtime::host_session_authority::store::legacy_writer_guard(
                &authority_home,
            ) {
                Ok(_) => panic!("activated HSA must reject the direct legacy writer"),
                Err(error) => error,
            };
            assert!(guard_error.to_string().contains("legacy"));

            let failed = fail_closed_auto_attach_policy_resolution(
                store,
                session_id,
                owner,
                &Policy::default(),
                "post_hsa_exact_failure",
            )
            .expect("same-owner claimed policy failure settles through the post-HSA seam");
            let SessionAutoAttachExecution::FailedClosed { settled: first, .. } = failed else {
                panic!("same-owner claimed policy failure must settle exact failed-closed truth");
            };
            assert_eq!(
                first.failed_closed_obligation_ids,
                vec![target_id.to_string()]
            );
            assert!(first.satisfied_obligation_ids.is_empty());
            assert!(first.superseded_obligation_ids.is_empty());

            let retry = settle_post_hsa_auto_attach_failed_closed(
                store,
                session_id,
                target_id,
                owner,
                "post_hsa_exact_failure",
            )
            .expect("join exact post-HSA failure")
            .expect("post-HSA failure retry uses the new seam");
            assert_eq!(retry, first);
            assert!(settle_post_hsa_auto_attach_failed_closed(
                store,
                session_id,
                target_id,
                owner,
                "post_hsa_substituted_failure",
            )
            .is_err());

            let sibling = store
                .load_obligation(session_id, sibling_id)
                .expect("load post-HSA sibling")
                .expect("post-HSA sibling exists");
            assert_eq!(
                sibling.attach_state,
                OrchestrationObligationAttachState::Eligible
            );
        });
    }

    #[test]
    #[serial_test::serial]
    #[cfg(target_os = "linux")]
    fn post_hsa_success_settlement_supersedes_siblings_and_exact_retry_joins() {
        with_store(|store| {
            let session_id = "sess_post_hsa_success_settle";
            let target_id = "obl_post_hsa_success_primary";
            let sibling_id = "obl_post_hsa_success_sibling";
            let owner = "router::post-hsa-success";
            seed_post_hsa_auto_attach_ledger(
                store,
                session_id,
                &[
                    (target_id, OrchestrationObligationKind::ApprovalRequired),
                    (sibling_id, OrchestrationObligationKind::FollowUpRequired),
                ],
            );
            seed_settled_auto_attach_authority(store, session_id, None);
            let context = resolve_post_hsa_auto_attach_authority_context(store, session_id)
                .expect("resolve success claim authority")
                .expect("success claim authority exists");
            assert!(matches!(
                claim_post_hsa_session_auto_attach(
                    store,
                    session_id,
                    owner,
                    &Policy::default(),
                    &context,
                )
                .expect("claim success target"),
                SessionAutoAttachClaim::Claimed { .. }
            ));
            let plan = build_auto_attach_launch_plan(store, session_id, target_id, owner)
                .expect("build success settlement plan");
            settle_auto_attach_startup_ownership(store, &plan);
            let receipt = HiddenOwnerHelperLaunchReceipt {
                helper_pid: 0,
                orchestration_session_id: session_id.to_string(),
                participant_id: plan.helper_plan.participant.participant_id.clone(),
                backend_id: plan.helper_plan.descriptor.backend_id.clone(),
            };
            let completion = ensure_auto_attach_restored_session(
                store,
                session_id,
                &receipt.participant_id,
                &plan,
                target_id,
                owner,
            )
            .expect("authenticate exact settlement completion");
            let snapshot = capture_post_hsa_auto_attach_ledger_snapshot(
                store,
                &completion.authority_observation,
            )
            .expect("capture exact settlement ledger");
            let sibling = snapshot
                .obligations
                .iter()
                .find(|obligation| obligation.obligation_id == sibling_id)
                .expect("success settlement sibling exists")
                .clone();
            let mut substituted_sibling = sibling.clone();
            substituted_sibling
                .mark_attach_superseded("substituted_sibling_result", chrono::Utc::now());
            substituted_sibling
                .validate()
                .expect("substituted sibling remains structurally valid");
            let acceptance_id = sibling
                .source_journal_event
                .as_ref()
                .expect("success sibling has source journal")
                .acceptance_record_id
                .clone();
            let sibling_file = format!("{sibling_id}.json");
            let original_bytes =
                serde_json::to_vec_pretty(&sibling).expect("serialize original sibling");
            let substituted_bytes = serde_json::to_vec_pretty(&substituted_sibling)
                .expect("serialize substituted sibling");
            let storage = store
                .bind_post_hsa_obligation_ledger_storage(&completion.authority_observation)
                .expect("bind exact success ledger storage");
            let mut transaction = storage
                .begin_transaction()
                .expect("begin substituted sibling transaction");
            transaction
                .verify_post_hsa_authority_binding(&completion.authority_observation, None)
                .expect("verify substituted sibling authority binding");
            transaction
                .replace_post_hsa_obligation_ledger_file(
                    completion.authority_observation.root_revision,
                    session_id,
                    &["acceptances", &acceptance_id, "obligations", &sibling_file],
                    &original_bytes,
                    &substituted_bytes,
                )
                .expect("simulate substituted sibling state");
            transaction.finish().expect("finish sibling substitution");
            assert!(settle_post_hsa_auto_attach_obligation(
                store,
                &PostHsaAutoAttachSettlementRequestV1 {
                    expected_ledger: snapshot.clone(),
                    obligation_id: target_id.to_string(),
                    claim_owner: owner.to_string(),
                    completion_reason: ROUTER_AUTO_ATTACH_RESTORED_REASON.to_string(),
                    settlement_kind: PostHsaAutoAttachSettlementKindV1::Satisfied,
                    router_auto_attach_intent: Some(completion.applied_intent.clone()),
                    settled_at: chrono::Utc::now(),
                },
            )
            .is_err());
            let mut restore = storage
                .begin_transaction()
                .expect("begin sibling restoration transaction");
            restore
                .replace_post_hsa_obligation_ledger_file(
                    completion.authority_observation.root_revision,
                    session_id,
                    &["acceptances", &acceptance_id, "obligations", &sibling_file],
                    &substituted_bytes,
                    &original_bytes,
                )
                .expect("restore exact sibling state");
            restore.finish().expect("finish sibling restoration");

            let mut stale = snapshot.clone();
            stale.authority_observation.root_revision -= 1;
            assert!(settle_post_hsa_auto_attach_obligation(
                store,
                &PostHsaAutoAttachSettlementRequestV1 {
                    expected_ledger: stale,
                    obligation_id: target_id.to_string(),
                    claim_owner: owner.to_string(),
                    completion_reason: ROUTER_AUTO_ATTACH_RESTORED_REASON.to_string(),
                    settlement_kind: PostHsaAutoAttachSettlementKindV1::Satisfied,
                    router_auto_attach_intent: Some(completion.applied_intent.clone()),
                    settled_at: chrono::Utc::now(),
                },
            )
            .is_err());
            let mut substituted_intent = completion.applied_intent.clone();
            substituted_intent.source_authoritative_participant_id =
                Some("ash_substituted_source".to_string());
            assert!(settle_post_hsa_auto_attach_obligation(
                store,
                &PostHsaAutoAttachSettlementRequestV1 {
                    expected_ledger: snapshot.clone(),
                    obligation_id: target_id.to_string(),
                    claim_owner: owner.to_string(),
                    completion_reason: ROUTER_AUTO_ATTACH_RESTORED_REASON.to_string(),
                    settlement_kind: PostHsaAutoAttachSettlementKindV1::Satisfied,
                    router_auto_attach_intent: Some(substituted_intent),
                    settled_at: chrono::Utc::now(),
                },
            )
            .is_err());
            assert!(settle_post_hsa_auto_attach_obligation(
                store,
                &PostHsaAutoAttachSettlementRequestV1 {
                    expected_ledger: snapshot.clone(),
                    obligation_id: sibling_id.to_string(),
                    claim_owner: owner.to_string(),
                    completion_reason: ROUTER_AUTO_ATTACH_RESTORED_REASON.to_string(),
                    settlement_kind: PostHsaAutoAttachSettlementKindV1::Satisfied,
                    router_auto_attach_intent: Some(completion.applied_intent.clone()),
                    settled_at: chrono::Utc::now(),
                },
            )
            .is_err());
            let target = snapshot
                .obligations
                .iter()
                .find(|obligation| obligation.obligation_id == target_id)
                .expect("success settlement target exists")
                .clone();
            let mut partially_settled_target = target.clone();
            partially_settled_target
                .mark_attach_satisfied(ROUTER_AUTO_ATTACH_RESTORED_REASON, chrono::Utc::now());
            let target_acceptance_id = target
                .source_journal_event
                .as_ref()
                .expect("success target has source journal")
                .acceptance_record_id
                .clone();
            let target_file = format!("{target_id}.json");
            let mut partial = storage
                .begin_transaction()
                .expect("begin target-first partial settlement");
            partial
                .replace_post_hsa_obligation_ledger_file(
                    completion.authority_observation.root_revision,
                    session_id,
                    &[
                        "acceptances",
                        &target_acceptance_id,
                        "obligations",
                        &target_file,
                    ],
                    &serde_json::to_vec_pretty(&target).expect("serialize claimed partial target"),
                    &serde_json::to_vec_pretty(&partially_settled_target)
                        .expect("serialize settled partial target"),
                )
                .expect("publish target-first partial settlement");
            partial
                .finish()
                .expect("finish target-first partial settlement");
            let first = finalize_session_auto_attach_after_launch(
                store,
                session_id,
                target_id.to_string(),
                owner.to_string(),
                receipt.clone(),
                &plan,
            )
            .expect("settle authenticated post-HSA success");
            let SessionAutoAttachExecution::Attached { settled, .. } = first else {
                panic!("authenticated post-HSA completion must attach");
            };
            assert_eq!(
                settled.satisfied_obligation_ids,
                vec![target_id.to_string()]
            );
            assert_eq!(
                settled.superseded_obligation_ids,
                vec![sibling_id.to_string()]
            );

            let retry = finalize_session_auto_attach_after_launch(
                store,
                session_id,
                target_id.to_string(),
                owner.to_string(),
                receipt,
                &plan,
            )
            .expect("join authenticated post-HSA settlement retry");
            let SessionAutoAttachExecution::Attached {
                settled: retry_result,
                ..
            } = retry
            else {
                panic!("exact settlement retry must remain attached");
            };
            assert_eq!(retry_result, settled);

            let target = store
                .load_obligation(session_id, target_id)
                .expect("load satisfied target")
                .expect("satisfied target exists");
            let sibling = store
                .load_obligation(session_id, sibling_id)
                .expect("load superseded sibling")
                .expect("superseded sibling exists");
            assert_eq!(
                target.attach_state,
                OrchestrationObligationAttachState::Satisfied
            );
            assert_eq!(
                sibling.attach_state,
                OrchestrationObligationAttachState::Superseded
            );

            let receipt_path = state_store_authority_home(store)
                .expect("settlement authority home")
                .join("obligation-ledger")
                .join(session_id)
                .join("auto-attach-settlements")
                .join(format!("{target_id}.json"));
            let mut corrupted: serde_json::Value = serde_json::from_slice(
                &fs::read(&receipt_path).expect("read exact settlement receipt"),
            )
            .expect("decode exact settlement receipt");
            corrupted["result"]["satisfied_obligation_ids"] = serde_json::json!([]);
            fs::write(
                &receipt_path,
                serde_json::to_vec_pretty(&corrupted).expect("serialize corrupted receipt"),
            )
            .expect("write corrupted receipt");
            let empty_retry = finalize_session_auto_attach_after_launch(
                store,
                session_id,
                target_id.to_string(),
                owner.to_string(),
                HiddenOwnerHelperLaunchReceipt {
                    helper_pid: 0,
                    orchestration_session_id: session_id.to_string(),
                    participant_id: plan.helper_plan.participant.participant_id.clone(),
                    backend_id: plan.helper_plan.descriptor.backend_id.clone(),
                },
                &plan,
            )
            .expect_err("empty committed settlement result must fail closed");
            assert!(
                empty_retry.to_string().contains("exact target obligation"),
                "empty settlement rejection must be explicit: {empty_retry:#}"
            );
        });
    }

    #[test]
    #[serial_test::serial]
    #[cfg(target_os = "linux")]
    fn execute_session_auto_attach_uses_post_hsa_claim_and_settlement_without_helper_relaunch() {
        with_store(|store| {
            let session_id = "sess_post_hsa_execute";
            let obligation_id = "obl_post_hsa_execute";
            let owner = "router::post-hsa-execute";
            seed_post_hsa_auto_attach_ledger(
                store,
                session_id,
                &[(obligation_id, OrchestrationObligationKind::Blocked)],
            );
            seed_settled_auto_attach_authority(store, session_id, None);

            let claim_context = resolve_post_hsa_auto_attach_authority_context(store, session_id)
                .expect("resolve execute claim authority")
                .expect("execute claim authority exists");
            write_workspace_policy(
                Path::new(&claim_context.workspace_root),
                r#"
workflow:
  router:
    enabled: true
agents:
  world_dispatch:
    obligations:
      blocked_allowed: true
"#,
            );
            assert!(matches!(
                claim_post_hsa_session_auto_attach(
                    store,
                    session_id,
                    owner,
                    &router_auto_attach_policy(OrchestrationObligationKind::Blocked),
                    &claim_context,
                )
                .expect("claim execute obligation"),
                SessionAutoAttachClaim::Claimed { .. }
            ));

            let precompleted =
                build_auto_attach_launch_plan(store, session_id, obligation_id, owner)
                    .expect("prebuild exact post-HSA execute plan");
            settle_auto_attach_startup_ownership(store, &precompleted);
            let before = auto_attach_authority(store)
                .expect("open pre-execute authority")
                .read_a12b_root()
                .expect("read pre-execute authority");
            assert_eq!(before.successor_transition_intent_map.len(), 1);
            assert_eq!(
                list_router_auto_attach_candidate_session_ids(store, owner)
                    .expect("discover exact same-owner claimed recovery"),
                vec![session_id.to_string()],
                "restart discovery must retain the exact claimed RouterAutoAttach operation"
            );

            let execution = execute_router_auto_attach_for_session(
                store,
                session_id,
                owner,
                &Policy::default(),
                false,
                true,
            )
            .expect("execute post-HSA auto-attach through policy-resolving production path")
            .execution;
            let SessionAutoAttachExecution::Attached {
                receipt, settled, ..
            } = execution
            else {
                panic!("post-HSA production path must attach: {execution:?}");
            };
            assert_eq!(receipt.helper_pid, 0);
            assert_eq!(
                settled.satisfied_obligation_ids,
                vec![obligation_id.to_string()]
            );
            let after = auto_attach_authority(store)
                .expect("open post-execute authority")
                .read_a12b_root()
                .expect("read post-execute authority");
            assert_eq!(after.successor_transition_intent_map.len(), 1);
            assert_eq!(
                after
                    .successor_transition_intent_map
                    .values()
                    .next()
                    .expect("exact RouterAutoAttach intent")
                    .caller
                    .auto_attach_obligation_id
                    .as_deref(),
                Some(obligation_id)
            );
        });
    }

    #[test]
    #[serial_test::serial]
    #[cfg(target_os = "linux")]
    fn auto_attach_rejects_manual_substitution_and_mutated_plan_identity() {
        use crate::execution::agent_runtime::host_session_authority::schema::{
            AuthorityObjectCommitmentV1, HostSessionTransitionCallerV1,
        };

        with_store(|store| {
            let world_binding = auto_attach_world_binding();
            seed_settled_auto_attach_authority(
                store,
                "sess_auto_attach_conflict",
                Some(world_binding),
            );
            let plan = build_auto_attach_launch_plan(
                store,
                "sess_auto_attach_conflict",
                "obl_auto_attach_conflict",
                "router::conflict",
            )
            .expect("build conflict plan");
            let authority = auto_attach_authority(store).expect("open conflict authority");
            let before = authority.read_a12b_root().expect("read conflict root");
            let resolved = authority
                .resolve_current_exact("sess_auto_attach_conflict", None)
                .expect("resolve conflict authority");
            let mut manual = auto_attach_issue_request_from_intent(
                store,
                &authority,
                &before,
                &plan.applied_transition,
            )
            .expect("reconstruct exact auto-attach issue request");
            manual.caller = HostSessionTransitionCallerV1 {
                kind: HostSessionTransitionCallerKindV1::PublicCli,
                caller_participant_id: manual.source_authoritative_participant_id.clone(),
                auto_attach_obligation_id: None,
                auto_attach_claim_owner: None,
            };
            authority
                .issue_successor(&resolved, &manual)
                .expect_err("manual Attach cannot substitute for RouterAutoAttach");
            let after_manual = authority.read_a12b_root().expect("read post-manual root");
            assert_eq!(after_manual.root_revision, before.root_revision);
            assert_eq!(after_manual.successor_transition_intent_map.len(), 1);

            let mut stale = plan.clone();
            let HostSessionAuthorityPreconditionV1::ExpectedRevision {
                authority_revision, ..
            } = &mut stale.applied_transition.authority_precondition
            else {
                panic!("auto-attach must carry ExpectedRevision");
            };
            *authority_revision += 1;
            assert!(validate_auto_attach_launch_plan(
                store,
                &stale,
                "obl_auto_attach_conflict",
                "router::conflict"
            )
            .is_err());

            let mut payload = plan.clone();
            payload.applied_transition.payload_commitment =
                AuthorityObjectCommitmentV1::CanonicalSha256 {
                    digest_hex: "0".repeat(64),
                };
            assert!(validate_auto_attach_launch_plan(
                store,
                &payload,
                "obl_auto_attach_conflict",
                "router::conflict"
            )
            .is_err());

            let mut descriptor = plan.clone();
            descriptor.helper_plan.descriptor.backend_id = "cli:substituted".into();
            assert!(validate_auto_attach_launch_plan(
                store,
                &descriptor,
                "obl_auto_attach_conflict",
                "router::conflict"
            )
            .is_err());

            let mut world = plan.clone();
            world.helper_plan.session.world_generation = Some(18);
            assert!(validate_auto_attach_launch_plan(
                store,
                &world,
                "obl_auto_attach_conflict",
                "router::conflict"
            )
            .is_err());

            let mut session = plan.clone();
            session.helper_plan.session.orchestration_session_id = "sess_substituted".into();
            assert!(validate_auto_attach_launch_plan(
                store,
                &session,
                "obl_auto_attach_conflict",
                "router::conflict"
            )
            .is_err());

            let mut lease = plan.clone();
            lease.helper_plan.participant.lease_token = "substituted-lease".into();
            assert!(validate_auto_attach_launch_plan(
                store,
                &lease,
                "obl_auto_attach_conflict",
                "router::conflict"
            )
            .is_err());

            let mut store_identity = plan.clone();
            store_identity.authority_store_id = "as_substituted".into();
            assert!(validate_auto_attach_launch_plan(
                store,
                &store_identity,
                "obl_auto_attach_conflict",
                "router::conflict"
            )
            .is_err());
            assert!(validate_auto_attach_launch_plan(
                store,
                &plan,
                "obl_other_claim",
                "manual::reattach"
            )
            .is_err());
        });
    }

    #[test]
    #[serial_test::serial]
    #[cfg(not(target_os = "linux"))]
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

            let err = build_auto_attach_launch_plan(
                store,
                "sess_auto_attach_fresh",
                "obl_auto_attach_fresh",
                "router::auto-attach",
            )
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
    #[cfg(not(target_os = "linux"))]
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

            let err = build_auto_attach_launch_plan(
                store,
                "sess_auto_attach_missing_continuity",
                "obl_auto_attach_missing_continuity",
                "router::auto-attach",
            )
            .expect_err("missing required continuity must fail closed");
            assert!(
                err.to_string()
                    .contains("persisted host attach contract no longer has continuity required"),
                "error should explain missing persisted continuity truth: {err:#}"
            );
        });
    }

    #[test]
    #[serial_test::serial]
    #[cfg(not(target_os = "linux"))]
    fn auto_attach_launch_plan_fails_closed_when_continuity_selector_is_blank() {
        with_store(|store| {
            let (mut session, participant) = detached_orchestrator(
                "sess_auto_attach_blank_continuity",
                "ash_auto_attach_blank_source",
            );
            let mut contract = session
                .host_attach_contract()
                .cloned()
                .expect("attach contract");
            contract.continuity_uaa_session_id = Some(" \t  \n ".to_string());
            session.host_attach_contract = Some(contract);

            store
                .persist_orchestration_session(&session)
                .expect("persist session");
            store
                .persist_participant(&participant)
                .expect("persist participant");

            let err = build_auto_attach_launch_plan(
                store,
                "sess_auto_attach_blank_continuity",
                "obl_auto_attach_blank_continuity",
                "router::auto-attach",
            )
            .expect_err("blank continuity selector must fail closed");
            assert!(
                err.to_string()
                    .contains("persisted host attach contract no longer has continuity required"),
                "error should treat blank continuity selector the same as missing continuity: {err:#}"
            );
        });
    }
}
