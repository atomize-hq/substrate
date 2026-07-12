use std::cmp::Ordering;
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::io;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
#[cfg(any(target_os = "linux", test))]
use sha2::{Digest, Sha256};
use tempfile::NamedTempFile;
use uuid::Uuid;
#[cfg(any(target_os = "linux", test))]
use world_api::SharedWorldBindingState;

use substrate_common::paths as substrate_paths;

use crate::execution::config_model::AgentExecutionScope;

#[cfg(any(target_os = "linux", test))]
use super::dispatch_contract::{
    ApprovalResponseDecisionV1, RetainedWorkerInspectSnapshotV1,
    WorkerContinueApprovalResponsePayloadV1, WorkerContinueClarificationResponsePayloadV1,
};
#[cfg(any(target_os = "linux", test))]
use super::obligation_ledger::ApprovalObligationCloseoutDisposition;
use super::{
    auto_attach::{
        claimed_obligation_id, select_attach_candidate, SessionAutoAttachClaim,
        SessionAutoAttachSettleResult,
    },
    control::PublicSessionPosture,
    host_inbox::{HostInboxMaterializationState, HostInboxRecord},
    mapping::{MEMBER_ROLE, ORCHESTRATOR_ROLE},
    obligation_ledger::{
        OrchestrationObligationAttachState, OrchestrationObligationKind,
        OrchestrationObligationRecord, OrchestrationObligationReviewState,
        OrchestrationObligationSeverity,
    },
    orchestration_session::{
        HostAttachContract, OrchestrationSessionPosture, OrchestrationSessionRecord,
        OrchestrationSessionState, StartupPromptStreamState,
    },
    session::{AgentRuntimeParticipantRecord, AgentRuntimeSessionManifest},
};

#[derive(Clone, Debug)]
pub(crate) struct AgentRuntimeSessionRecord {
    pub session: OrchestrationSessionRecord,
    pub participants: Vec<AgentRuntimeParticipantRecord>,
    #[allow(dead_code)]
    pub warnings: Vec<String>,
    has_authoritative_parent: bool,
    #[allow(dead_code)]
    complete: bool,
}

fn stop_order_probe_should_log_parked_write(
    existing: Option<&OrchestrationSessionRecord>,
    session: &OrchestrationSessionRecord,
) -> bool {
    let Ok(filtered_session_id) = std::env::var("SUBSTRATE_STOP_ORDER_DEBUG_SESSION") else {
        return false;
    };
    if filtered_session_id != session.orchestration_session_id {
        return false;
    }
    session.posture == OrchestrationSessionPosture::ParkedResumable
        && existing
            .is_some_and(|value| value.posture != OrchestrationSessionPosture::ParkedResumable)
}

fn stop_order_probe_log_parked_write(
    existing: Option<&OrchestrationSessionRecord>,
    session: &OrchestrationSessionRecord,
) {
    let log_path = std::env::var("SUBSTRATE_STOP_ORDER_DEBUG_LOG")
        .unwrap_or_else(|_| "/tmp/substrate-stop-order-debug.jsonl".to_string());
    let record = serde_json::json!({
        "ts": chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Nanos, true),
        "pid": std::process::id(),
        "event": "persist_orchestration_session_parked_write",
        "session_id": session.orchestration_session_id,
        "participant_id": serde_json::Value::Null,
        "fields": {
            "previous_posture": existing.map(|value| format!("{:?}", value.posture)),
            "previous_active_session_handle_id": existing.and_then(OrchestrationSessionRecord::active_participant_id),
            "previous_attached_participant_id": existing.and_then(OrchestrationSessionRecord::attached_participant_id),
            "new_posture": format!("{:?}", session.posture),
            "new_active_session_handle_id": session.active_participant_id(),
            "new_attached_participant_id": session.attached_participant_id(),
            "latest_run_id": session.latest_run_id,
            "parked_reason": session.parked_reason,
            "last_parked_at": session
                .last_parked_at
                .map(|value| value.to_rfc3339_opts(chrono::SecondsFormat::Nanos, true)),
        },
    });
    let Ok(mut file) = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(log_path)
    else {
        return;
    };
    let _ = writeln!(file, "{record}");
}

impl AgentRuntimeSessionRecord {
    pub(crate) fn orchestration_session_id(&self) -> &str {
        &self.session.orchestration_session_id
    }

    #[allow(dead_code)]
    pub(crate) fn is_complete(&self) -> bool {
        self.complete
    }

    pub(crate) fn live_participants(&self) -> Vec<AgentRuntimeParticipantRecord> {
        self.participants
            .iter()
            .filter(|participant| {
                participant.is_authoritative_live() && owner_process_is_alive(participant)
            })
            .cloned()
            .collect()
    }

    pub(crate) fn live_orchestrator(&self) -> Option<AgentRuntimeParticipantRecord> {
        let active_participant_id = self.session.active_participant_id()?;
        self.live_participants().into_iter().find(|participant| {
            participant.participant_id() == active_participant_id
                && participant.matches_public_parent_linkage(&self.session)
        })
    }

    pub(crate) fn status_visible_participants(&self) -> Vec<AgentRuntimeParticipantRecord> {
        let mut participants = self.live_participants();
        if let Ok(Some(detached_participant)) = detached_status_visible_participant(self) {
            if participants.iter().all(|participant| {
                participant.handle.participant_id != detached_participant.handle.participant_id
            }) {
                participants.push(detached_participant);
            }
        }
        participants.sort_by(|left, right| {
            left.handle
                .last_transition_at
                .cmp(&right.handle.last_transition_at)
                .then(left.handle.participant_id.cmp(&right.handle.participant_id))
        });
        participants
    }

    #[allow(dead_code)]
    pub(crate) fn live_participant_for_agent(
        &self,
        agent_id: &str,
        scope: AgentExecutionScope,
        role: &str,
    ) -> Option<AgentRuntimeParticipantRecord> {
        self.live_participants().into_iter().find(|participant| {
            participant.handle.agent_id == agent_id
                && participant.handle.execution.scope == scope
                && participant.handle.role == role
        })
    }

    #[allow(dead_code)]
    pub(crate) fn invalidated_world_members(&self) -> Vec<AgentRuntimeParticipantRecord> {
        self.participants
            .iter()
            .filter(|participant| {
                participant.handle.role == MEMBER_ROLE
                    && participant.handle.execution.scope == AgentExecutionScope::World
                    && participant.handle.state
                        == super::session::AgentRuntimeSessionState::Invalidated
            })
            .cloned()
            .collect()
    }

    pub(crate) fn last_updated_at(&self) -> DateTime<Utc> {
        self.participants
            .iter()
            .map(AgentRuntimeParticipantRecord::last_status_at)
            .max()
            .map_or(self.session.last_active_at, |participant_ts| {
                participant_ts.max(self.session.last_active_at)
            })
    }

    fn has_authoritative_parent(&self) -> bool {
        self.has_authoritative_parent
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum HiddenOwnerHelperLaunchReadiness {
    Pending,
    ReadyAttached,
    ReadyDetached(OrchestrationSessionPosture),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum HiddenOwnerHelperLaunchContinuity {
    Pending,
    AttachedLive,
    DetachedReconciled(OrchestrationSessionPosture),
    StaleAttachedTruth,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum StartupPromptReplayState {
    NotTracked,
    PendingAcceptance,
    AcceptedOrTerminal,
}

impl StartupPromptReplayState {
    #[cfg(unix)]
    pub(crate) fn replay_safe(self) -> bool {
        matches!(self, Self::NotTracked | Self::PendingAcceptance)
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum DurableInboxItemKind {
    ApprovalRequired,
    CompletionNotice,
    FollowUpMessage,
    RuntimeAlert,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum DurableInboxItemState {
    Pending,
    Acknowledged,
    Dismissed,
}

impl DurableInboxItemState {
    fn is_pending(self) -> bool {
        matches!(self, Self::Pending)
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub(crate) struct DurableInboxItemRecord {
    pub orchestration_session_id: String,
    pub item_id: String,
    pub kind: DurableInboxItemKind,
    pub state: DurableInboxItemState,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resolved_at: Option<DateTime<Utc>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

impl DurableInboxItemRecord {
    #[allow(dead_code)]
    pub(crate) fn new(
        orchestration_session_id: impl Into<String>,
        item_id: impl Into<String>,
        kind: DurableInboxItemKind,
        message: Option<String>,
    ) -> Self {
        let now = Utc::now();
        Self {
            orchestration_session_id: orchestration_session_id.into(),
            item_id: item_id.into(),
            kind,
            state: DurableInboxItemState::Pending,
            created_at: now,
            updated_at: now,
            resolved_at: None,
            message,
        }
    }

    fn validate(&self) -> Result<()> {
        if self.orchestration_session_id.trim().is_empty() {
            anyhow::bail!("durable inbox item must include orchestration_session_id");
        }
        if self.item_id.trim().is_empty() {
            anyhow::bail!("durable inbox item must include item_id");
        }
        if self.state.is_pending() && self.resolved_at.is_some() {
            anyhow::bail!("pending durable inbox items must not include resolved_at");
        }
        if !self.state.is_pending() && self.resolved_at.is_none() {
            anyhow::bail!("resolved durable inbox items must include resolved_at");
        }

        Ok(())
    }

    fn is_pending(&self) -> bool {
        self.state.is_pending()
    }

    fn transition_state(&mut self, state: DurableInboxItemState) {
        let now = Utc::now();
        self.state = state;
        self.updated_at = now;
        if state.is_pending() {
            self.resolved_at = None;
        } else {
            self.resolved_at = Some(now);
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum PublicControlAction {
    Fork,
    Stop,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum PublicAttachAction {
    Reattach,
    DetachedTurn,
}

impl PublicAttachAction {
    fn continuity_label(self) -> &'static str {
        match self {
            Self::Reattach => "control-only reattach",
            Self::DetachedTurn => "detached turn recovery",
        }
    }
}

#[derive(Clone, Debug)]
pub(crate) struct ResolvedPublicControlTarget {
    pub session: OrchestrationSessionRecord,
    pub active_participant: AgentRuntimeParticipantRecord,
    pub session_posture: PublicSessionPosture,
    pub host_attach_contract: Option<HostAttachContract>,
}

impl ResolvedPublicControlTarget {
    #[allow(dead_code)]
    pub(crate) fn orchestration_session_id(&self) -> &str {
        &self.session.orchestration_session_id
    }
}

#[derive(Clone, Debug)]
pub(crate) struct ResolvedPublicAttachTarget {
    pub session: OrchestrationSessionRecord,
    pub active_participant: AgentRuntimeParticipantRecord,
    #[allow(dead_code)]
    pub session_posture: PublicSessionPosture,
    pub host_attach_contract: Option<HostAttachContract>,
}

impl ResolvedPublicAttachTarget {
    #[allow(dead_code)]
    pub(crate) fn orchestration_session_id(&self) -> &str {
        &self.session.orchestration_session_id
    }
}

#[derive(Clone, Debug)]
pub(crate) struct ResolvedInternalWorldDispatchCaller {
    pub session: OrchestrationSessionRecord,
    pub caller_participant: AgentRuntimeParticipantRecord,
}

impl ResolvedInternalWorldDispatchCaller {
    #[allow(dead_code)]
    pub(crate) fn orchestration_session_id(&self) -> &str {
        &self.session.orchestration_session_id
    }
}

#[cfg(any(target_os = "linux", test))]
const SHARED_WORLD_METADATA_FILE: &str = "session.json";

#[cfg(any(target_os = "linux", test))]
#[cfg(test)]
const SHARED_WORLD_METADATA_ROOT_TEST_ENV: &str = "SUBSTRATE_TEST_SHARED_WORLD_METADATA_ROOT";

#[cfg(any(target_os = "linux", test))]
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "snake_case")]
enum SharedWorldMetadataOwnerMode {
    #[default]
    Generic,
    SharedOrchestration,
}

#[cfg(any(target_os = "linux", test))]
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq)]
struct SharedWorldMetadataRecord {
    world_id: String,
    #[serde(default)]
    owner_mode: SharedWorldMetadataOwnerMode,
    #[serde(default)]
    orchestration_session_id: Option<String>,
    #[serde(default)]
    world_generation: Option<u64>,
    #[serde(default)]
    binding_state: Option<SharedWorldBindingState>,
}

#[cfg(any(target_os = "linux", test))]
fn shared_world_metadata_root() -> PathBuf {
    #[cfg(test)]
    if let Some(root) = std::env::var_os(SHARED_WORLD_METADATA_ROOT_TEST_ENV) {
        return PathBuf::from(root);
    }

    let uid = current_uid();
    default_shared_world_metadata_root(
        uid,
        std::env::var_os("XDG_RUNTIME_DIR")
            .as_deref()
            .map(Path::new),
        Path::new(&format!("/run/user/{uid}")).is_dir(),
    )
}

#[cfg(any(target_os = "linux", test))]
fn default_shared_world_metadata_root(
    uid: u32,
    xdg_runtime_dir: Option<&Path>,
    has_run_user_dir: bool,
) -> PathBuf {
    if let Some(xdg_runtime_dir) = xdg_runtime_dir.filter(|path| !path.as_os_str().is_empty()) {
        return xdg_runtime_dir.join("substrate").join("worlds");
    }

    if has_run_user_dir {
        return PathBuf::from(format!("/run/user/{uid}"))
            .join("substrate")
            .join("worlds");
    }

    PathBuf::from(format!("/tmp/substrate-worlds-{uid}"))
}

#[cfg(all(unix, any(target_os = "linux", test)))]
fn current_uid() -> u32 {
    // SAFETY: geteuid reads process credentials without requiring additional invariants.
    unsafe { libc::geteuid() as u32 }
}

#[cfg(all(not(unix), any(target_os = "linux", test)))]
fn current_uid() -> u32 {
    0
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum RouterAutoAttachSessionReadiness {
    Eligible,
    NoCandidate { reason: &'static str },
}

#[cfg(any(target_os = "linux", test))]
#[allow(dead_code)]
#[derive(Clone, Debug)]
pub(crate) struct ResolvedInternalForkWorldDispatchLineage {
    pub orchestration_session_id: String,
    pub orchestrator_participant_id: String,
    pub source_participant_id: String,
    pub child_participant_id: String,
    pub world_id: String,
    pub world_generation: u64,
}

#[cfg(any(target_os = "linux", test))]
#[derive(Clone, Debug)]
pub(crate) struct ResolvedInternalForkWorldDispatchTarget {
    pub session: OrchestrationSessionRecord,
    pub caller_participant: AgentRuntimeParticipantRecord,
    pub source_participant: AgentRuntimeParticipantRecord,
}

#[cfg(any(target_os = "linux", test))]
impl ResolvedInternalForkWorldDispatchTarget {
    #[allow(dead_code)]
    pub(crate) fn orchestration_session_id(&self) -> &str {
        &self.session.orchestration_session_id
    }

    #[allow(dead_code)]
    pub(crate) fn project_child_lineage(
        &self,
        child_participant: &AgentRuntimeParticipantRecord,
    ) -> Result<ResolvedInternalForkWorldDispatchLineage> {
        if child_participant.handle.orchestration_session_id
            != self.session.orchestration_session_id
        {
            anyhow::bail!(
                "child_not_in_session: orchestration session {} child {} does not belong to the authoritative session",
                self.session.orchestration_session_id,
                child_participant.participant_id()
            );
        }
        if child_participant.handle.backend_id != self.source_participant.handle.backend_id {
            anyhow::bail!(
                "backend_mismatch: orchestration session {} child {} backend is {} not {}",
                self.session.orchestration_session_id,
                child_participant.participant_id(),
                child_participant.handle.backend_id,
                self.source_participant.handle.backend_id
            );
        }
        if child_participant
            .handle
            .orchestrator_participant_id
            .as_deref()
            != Some(self.caller_participant.participant_id())
        {
            anyhow::bail!(
                "stale_linkage: orchestration session {} child {} is not linked to authoritative orchestrator {}",
                self.session.orchestration_session_id,
                child_participant.participant_id(),
                self.caller_participant.participant_id()
            );
        }
        if !child_participant.matches_authoritative_parent_world_binding(&self.session) {
            anyhow::bail!(
                "world_binding_mismatch: orchestration session {} child {} no longer matches the authoritative world binding",
                self.session.orchestration_session_id,
                child_participant.participant_id()
            );
        }
        if child_participant.fork_source_participant_id()
            != Some(self.source_participant.participant_id())
        {
            anyhow::bail!(
                "invalid_fork_lineage: orchestration session {} child {} must point parent_participant_id at fork source {}",
                self.session.orchestration_session_id,
                child_participant.participant_id(),
                self.source_participant.participant_id()
            );
        }

        Ok(ResolvedInternalForkWorldDispatchLineage {
            orchestration_session_id: self.session.orchestration_session_id.clone(),
            orchestrator_participant_id: self.caller_participant.participant_id().to_string(),
            source_participant_id: self.source_participant.participant_id().to_string(),
            child_participant_id: child_participant.participant_id().to_string(),
            world_id: self
                .session
                .world_id
                .clone()
                .expect("authoritative world binding must be present"),
            world_generation: self
                .session
                .world_generation
                .expect("authoritative world binding must be present"),
        })
    }
}

#[cfg(any(target_os = "linux", test))]
#[derive(Clone, Debug)]
pub(crate) struct ResolvedInternalContinueWorldDispatchTarget {
    pub session: OrchestrationSessionRecord,
    pub caller_participant: AgentRuntimeParticipantRecord,
    pub target_participant: AgentRuntimeParticipantRecord,
}

#[cfg(any(target_os = "linux", test))]
impl ResolvedInternalContinueWorldDispatchTarget {
    #[allow(dead_code)]
    pub(crate) fn orchestration_session_id(&self) -> &str {
        &self.session.orchestration_session_id
    }
}

#[allow(dead_code)]
#[cfg(any(target_os = "linux", test))]
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct PreparedInternalApprovalResponseObligationCloseout {
    orchestration_session_id: String,
    approval_obligation_id: String,
    target_participant_id: String,
    target_backend_id: String,
    world_id: String,
    world_generation: u64,
    decision: ApprovalResponseDecisionV1,
}

#[allow(dead_code)]
#[cfg(any(target_os = "linux", test))]
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct PreparedInternalClarificationResponseObligationCloseout {
    orchestration_session_id: String,
    follow_up_obligation_id: String,
    target_participant_id: String,
    target_backend_id: String,
    world_id: String,
    world_generation: u64,
}

#[cfg(any(target_os = "linux", test))]
#[derive(Clone, Debug)]
pub(crate) struct ResolvedInternalInspectWorldDispatchTarget {
    pub session: OrchestrationSessionRecord,
    #[allow(dead_code)]
    pub caller_participant: AgentRuntimeParticipantRecord,
    #[allow(dead_code)]
    pub target_participant: AgentRuntimeParticipantRecord,
}

#[cfg(any(target_os = "linux", test))]
impl ResolvedInternalInspectWorldDispatchTarget {
    #[allow(dead_code)]
    pub(crate) fn orchestration_session_id(&self) -> &str {
        &self.session.orchestration_session_id
    }

    #[allow(dead_code)]
    pub(crate) fn project_snapshot(&self) -> RetainedWorkerInspectSnapshotV1 {
        RetainedWorkerInspectSnapshotV1 {
            participant_state: self.target_participant.handle.state.clone(),
            session_state: self.session.state.clone(),
            session_posture: self.session.posture,
            authoritative_live: self.target_participant.is_authoritative_live()
                && owner_process_is_alive(&self.target_participant),
            attention_required: self.session.posture
                == OrchestrationSessionPosture::AwaitingAttention
                || self.session.pending_inbox_count > 0,
            parent_participant_id: self.target_participant.handle.parent_participant_id.clone(),
            resumed_from_participant_id: self
                .target_participant
                .handle
                .resumed_from_participant_id
                .clone(),
        }
    }
}

#[cfg(any(target_os = "linux", test))]
#[derive(Clone, Debug)]
pub(crate) struct ExactRetainedWorkerStopDispatchTarget {
    orchestration_session_id: String,
    participant_id: String,
    backend_id: String,
    world_id: String,
    world_generation: u64,
    orchestrator_participant_id: String,
}

#[cfg(any(target_os = "linux", test))]
impl ExactRetainedWorkerStopDispatchTarget {
    fn capture(
        orchestration_session_id: &str,
        target_participant: &AgentRuntimeParticipantRecord,
    ) -> Result<Self> {
        let world_id = target_participant
            .handle
            .world_id
            .clone()
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "world_binding_mismatch: orchestration session {} retained worker {} is missing exact stop target world binding",
                    orchestration_session_id,
                    target_participant.participant_id()
                )
            })?;
        let world_generation = target_participant.handle.world_generation.ok_or_else(|| {
            anyhow::anyhow!(
                "world_binding_mismatch: orchestration session {} retained worker {} is missing exact stop target world binding",
                orchestration_session_id,
                target_participant.participant_id()
            )
        })?;
        let orchestrator_participant_id = target_participant
            .handle
            .orchestrator_participant_id
            .clone()
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "stale_linkage: orchestration session {} retained worker {} is missing exact stop target lineage",
                    orchestration_session_id,
                    target_participant.participant_id()
                )
            })?;

        Ok(Self {
            orchestration_session_id: orchestration_session_id.to_string(),
            participant_id: target_participant.participant_id().to_string(),
            backend_id: target_participant.handle.backend_id.clone(),
            world_id,
            world_generation,
            orchestrator_participant_id,
        })
    }

    fn ensure_matches(&self, target_participant: &AgentRuntimeParticipantRecord) -> Result<()> {
        if target_participant.participant_id() != self.participant_id {
            anyhow::bail!(
                "target_not_in_session: orchestration session {} exact stop target changed from {} to {}",
                self.orchestration_session_id,
                self.participant_id,
                target_participant.participant_id()
            );
        }
        if target_participant.handle.backend_id != self.backend_id {
            anyhow::bail!(
                "backend_mismatch: orchestration session {} retained worker {} exact stop target backend changed from {} to {}",
                self.orchestration_session_id,
                self.participant_id,
                self.backend_id,
                target_participant.handle.backend_id
            );
        }

        let refreshed_world_id = target_participant.handle.world_id.as_deref().ok_or_else(|| {
            anyhow::anyhow!(
                "world_binding_mismatch: orchestration session {} retained worker {} is missing exact stop target world binding",
                self.orchestration_session_id,
                self.participant_id
            )
        })?;
        let refreshed_world_generation =
            target_participant.handle.world_generation.ok_or_else(|| {
                anyhow::anyhow!(
                    "world_binding_mismatch: orchestration session {} retained worker {} is missing exact stop target world binding",
                    self.orchestration_session_id,
                    self.participant_id
                )
            })?;
        if refreshed_world_id != self.world_id
            || refreshed_world_generation != self.world_generation
        {
            anyhow::bail!(
                "world_binding_mismatch: orchestration session {} retained worker {} exact stop target world binding changed from {}/{} to {}/{}",
                self.orchestration_session_id,
                self.participant_id,
                self.world_id,
                self.world_generation,
                refreshed_world_id,
                refreshed_world_generation
            );
        }

        let refreshed_orchestrator_participant_id = target_participant
            .handle
            .orchestrator_participant_id
            .as_deref()
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "stale_linkage: orchestration session {} retained worker {} is missing exact stop target lineage",
                    self.orchestration_session_id,
                    self.participant_id
                )
            })?;
        if refreshed_orchestrator_participant_id != self.orchestrator_participant_id {
            anyhow::bail!(
                "stale_linkage: orchestration session {} retained worker {} exact stop target lineage changed from {} to {}",
                self.orchestration_session_id,
                self.participant_id,
                self.orchestrator_participant_id,
                refreshed_orchestrator_participant_id
            );
        }

        Ok(())
    }
}

#[cfg(any(target_os = "linux", test))]
#[derive(Clone, Debug)]
pub(crate) struct ResolvedInternalStopWorldDispatchTarget {
    pub session: OrchestrationSessionRecord,
    #[allow(dead_code)]
    pub caller_participant: AgentRuntimeParticipantRecord,
    #[allow(dead_code)]
    pub target_participant: AgentRuntimeParticipantRecord,
    exact_target: ExactRetainedWorkerStopDispatchTarget,
}

#[cfg(any(target_os = "linux", test))]
impl ResolvedInternalStopWorldDispatchTarget {
    #[allow(dead_code)]
    pub(crate) fn orchestration_session_id(&self) -> &str {
        &self.session.orchestration_session_id
    }

    pub(crate) fn ensure_exact_target_match(&self, refreshed: &Self) -> Result<()> {
        self.exact_target
            .ensure_matches(&refreshed.target_participant)
    }

    pub(crate) fn project_refreshed_exact_target(
        &self,
        record: AgentRuntimeSessionRecord,
    ) -> Result<Self> {
        let session = record.session;
        let mut matching_participants = record
            .participants
            .into_iter()
            .filter(|participant| {
                participant.participant_id() == self.target_participant.participant_id()
            })
            .collect::<Vec<_>>();

        if matching_participants.is_empty() {
            anyhow::bail!(
                "target_not_in_session: orchestration session {} has no exact retained worker {}",
                session.orchestration_session_id,
                self.target_participant.participant_id()
            );
        }
        if matching_participants.len() > 1 {
            anyhow::bail!(
                "ambiguous_target_participant: orchestration session {} has multiple retained worker records for {}",
                session.orchestration_session_id,
                self.target_participant.participant_id()
            );
        }

        let target_participant = matching_participants
            .pop()
            .expect("target participant count checked above");
        if target_participant.handle.orchestration_session_id != session.orchestration_session_id {
            anyhow::bail!(
                "target_not_in_session: orchestration session {} has no exact retained worker {}",
                session.orchestration_session_id,
                self.target_participant.participant_id()
            );
        }
        if target_participant.handle.role != MEMBER_ROLE
            || target_participant.handle.execution.scope != AgentExecutionScope::World
        {
            anyhow::bail!(
                "invalid_target_participant: orchestration session {} participant {} is not a retained world worker",
                session.orchestration_session_id,
                target_participant.participant_id()
            );
        }
        self.exact_target.ensure_matches(&target_participant)?;
        if !target_participant.matches_authoritative_parent_world_binding(&session) {
            anyhow::bail!(
                "world_binding_mismatch: orchestration session {} retained worker {} no longer matches the authoritative world binding",
                session.orchestration_session_id,
                target_participant.participant_id()
            );
        }
        if !target_participant.handle.state.is_live()
            || target_participant.internal.terminal_observed_at.is_some()
        {
            let terminal_state = match target_participant.handle.state {
                super::session::AgentRuntimeSessionState::Stopped => "stopped",
                super::session::AgentRuntimeSessionState::Failed => "failed",
                super::session::AgentRuntimeSessionState::Invalidated => "invalidated",
                super::session::AgentRuntimeSessionState::Allocating
                | super::session::AgentRuntimeSessionState::Ready
                | super::session::AgentRuntimeSessionState::Running
                | super::session::AgentRuntimeSessionState::Restarting
                | super::session::AgentRuntimeSessionState::Stopping => "terminal",
            };
            anyhow::bail!(
                "target_already_terminal: orchestration session {} retained worker {} is already terminal ({})",
                session.orchestration_session_id,
                target_participant.participant_id(),
                terminal_state
            );
        }
        Ok(Self {
            session,
            caller_participant: self.caller_participant.clone(),
            target_participant,
            exact_target: self.exact_target.clone(),
        })
    }
}

#[cfg(any(target_os = "linux", test))]
#[derive(Clone, Debug)]
pub(crate) struct ResolvedInternalCancelWorldDispatchTarget {
    pub session: OrchestrationSessionRecord,
    #[allow(dead_code)]
    pub caller_participant: AgentRuntimeParticipantRecord,
    #[allow(dead_code)]
    pub target_participant: AgentRuntimeParticipantRecord,
    #[allow(dead_code)]
    pub active_run_id: String,
}

#[cfg(any(target_os = "linux", test))]
impl ResolvedInternalCancelWorldDispatchTarget {
    #[allow(dead_code)]
    pub(crate) fn orchestration_session_id(&self) -> &str {
        &self.session.orchestration_session_id
    }
}

#[allow(dead_code)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum PublicTurnTargetKind {
    Host,
    World,
}

#[allow(dead_code)]
#[derive(Clone, Debug)]
pub(crate) struct ResolvedPublicTurnTarget {
    pub session: OrchestrationSessionRecord,
    pub participant: AgentRuntimeParticipantRecord,
    pub target_kind: PublicTurnTargetKind,
    pub session_posture: PublicSessionPosture,
    pub host_attach_contract: Option<HostAttachContract>,
}

impl ResolvedPublicTurnTarget {
    #[allow(dead_code)]
    pub(crate) fn orchestration_session_id(&self) -> &str {
        &self.session.orchestration_session_id
    }
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
enum ParticipantRecordSource {
    Canonical,
    Flat,
    Legacy,
}

#[derive(Clone, Debug)]
struct ResolvedAuthoritativeSessionControl {
    session: OrchestrationSessionRecord,
    participant: AgentRuntimeParticipantRecord,
    session_posture: PublicSessionPosture,
}

#[cfg_attr(not(any(target_os = "linux", test)), allow(dead_code))]
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub(crate) struct ActiveEphemeralWorldTaskRecord {
    pub orchestration_session_id: String,
    pub task_run_id: String,
    pub caller_participant_id: String,
    pub target_backend_id: String,
    pub world_id: String,
    pub world_generation: u64,
}

impl ActiveEphemeralWorldTaskRecord {
    fn validate(&self) -> Result<()> {
        if self.orchestration_session_id.trim().is_empty() {
            anyhow::bail!("active ephemeral task must include orchestration_session_id");
        }
        if self.task_run_id.trim().is_empty() {
            anyhow::bail!("active ephemeral task must include task_run_id");
        }
        if self.caller_participant_id.trim().is_empty() {
            anyhow::bail!("active ephemeral task must include caller_participant_id");
        }
        if self.target_backend_id.trim().is_empty() {
            anyhow::bail!("active ephemeral task must include target_backend_id");
        }
        if self.world_id.trim().is_empty() {
            anyhow::bail!("active ephemeral task must include world_id");
        }
        Ok(())
    }
}

#[cfg_attr(not(any(target_os = "linux", test)), allow(dead_code))]
#[derive(Clone, Debug)]
pub(crate) struct ActiveEphemeralWorldTaskGuard {
    store: AgentRuntimeStateStore,
    orchestration_session_id: String,
    task_run_id: String,
}

impl Drop for ActiveEphemeralWorldTaskGuard {
    fn drop(&mut self) {
        let _ = self
            .store
            .remove_active_ephemeral_world_task(&self.orchestration_session_id, &self.task_run_id);
    }
}

#[derive(Clone, Debug)]
pub(crate) struct AgentRuntimeStateStore {
    substrate_home: PathBuf,
}

#[derive(Clone, Debug)]
struct ResolvedPublicSessionAuthority {
    session: OrchestrationSessionRecord,
    participant: AgentRuntimeParticipantRecord,
    session_posture: PublicSessionPosture,
    host_attach_contract: Option<HostAttachContract>,
}

impl AgentRuntimeStateStore {
    pub(crate) fn new() -> Result<Self> {
        Ok(Self {
            substrate_home: substrate_paths::substrate_home()?,
        })
    }

    fn with_legacy_snapshot_transaction<T>(
        &self,
        operation: impl FnOnce(
            &mut super::host_session_authority::store::LegacyWriterGuard,
        ) -> Result<T>,
    ) -> Result<T> {
        let process = snapshot_write_lock()
            .lock()
            .expect("snapshot write mutex poisoned");
        let mut authority =
            super::host_session_authority::store::legacy_writer_guard(&self.substrate_home)
                .context("legacy authority writer preflight failed")?;
        let outcome = operation(&mut authority);
        let finish = authority
            .finish()
            .context("legacy authority writer final fsync failed");
        drop(process);
        match outcome {
            Ok(value) => finish.map(|()| value),
            Err(error) => {
                let _ = finish;
                Err(error)
            }
        }
    }

    fn transaction_read_json<T: serde::de::DeserializeOwned>(
        transaction: &mut super::host_session_authority::store::LegacyWriterGuard,
        collection: super::host_session_authority::store::LegacyStateStoreCollectionV1,
        descendants: &[&str],
    ) -> Result<Option<T>> {
        transaction
            .read_file(collection, descendants)
            .context("read retained legacy StateStore JSON")?
            .map(|bytes| {
                serde_json::from_slice(&bytes).context("parse retained legacy StateStore JSON")
            })
            .transpose()
    }

    fn transaction_write_json(
        transaction: &mut super::host_session_authority::store::LegacyWriterGuard,
        collection: super::host_session_authority::store::LegacyStateStoreCollectionV1,
        descendants: &[&str],
        value: &impl serde::Serialize,
    ) -> Result<()> {
        let bytes = serde_json::to_vec_pretty(value)
            .context("serialize retained legacy StateStore JSON")?;
        transaction
            .write_file(collection, descendants, &bytes, *Uuid::new_v4().as_bytes())
            .context("publish retained legacy StateStore JSON")
    }

    fn transaction_remove_file(
        transaction: &mut super::host_session_authority::store::LegacyWriterGuard,
        collection: super::host_session_authority::store::LegacyStateStoreCollectionV1,
        descendants: &[&str],
    ) -> Result<bool> {
        transaction
            .remove_file(collection, descendants)
            .context("remove retained legacy StateStore file")
    }

    fn transaction_list_json<T: serde::de::DeserializeOwned>(
        transaction: &mut super::host_session_authority::store::LegacyWriterGuard,
        collection: super::host_session_authority::store::LegacyStateStoreCollectionV1,
        directory: &[&str],
    ) -> Result<Vec<T>> {
        let entries = transaction
            .read_directory(collection, directory)
            .context("enumerate retained legacy StateStore directory")?;
        let mut values = Vec::new();
        for entry in entries {
            if entry.is_directory || !entry.name.ends_with(".json") {
                continue;
            }
            let bytes = entry
                .bytes
                .ok_or_else(|| anyhow::anyhow!("retained StateStore file omitted bytes"))?;
            values.push(
                serde_json::from_slice(&bytes)
                    .context("parse enumerated retained legacy StateStore JSON")?,
            );
        }
        Ok(values)
    }

    fn list_participants_across_sources_transaction(
        &self,
        transaction: &mut super::host_session_authority::store::LegacyWriterGuard,
    ) -> Result<Vec<AgentRuntimeParticipantRecord>> {
        use super::host_session_authority::store::LegacyStateStoreCollectionV1::{
            Handles, Participants, Sessions,
        };
        let mut participants = BTreeMap::new();
        for session_entry in transaction
            .read_directory(Sessions, &[])
            .context("enumerate retained canonical session roots")?
        {
            if !session_entry.is_directory {
                continue;
            }
            for participant in Self::transaction_list_json::<AgentRuntimeParticipantRecord>(
                transaction,
                Sessions,
                &[session_entry.name.as_str(), "participants"],
            )? {
                self.validate_participant_record(&participant)?;
                participants.insert(participant.handle.participant_id.clone(), participant);
            }
        }
        for participant in Self::transaction_list_json::<AgentRuntimeParticipantRecord>(
            transaction,
            Participants,
            &[],
        )? {
            self.validate_participant_record(&participant)?;
            participants
                .entry(participant.handle.participant_id.clone())
                .or_insert(participant);
        }
        for participant in
            Self::transaction_list_json::<AgentRuntimeParticipantRecord>(transaction, Handles, &[])?
        {
            self.validate_participant_record(&participant)?;
            participants
                .entry(participant.handle.participant_id.clone())
                .or_insert(participant);
        }
        let mut participants = participants.into_values().collect::<Vec<_>>();
        participants.sort_by(|left, right| {
            left.handle
                .last_transition_at
                .cmp(&right.handle.last_transition_at)
                .then(left.handle.participant_id.cmp(&right.handle.participant_id))
        });
        Ok(participants)
    }

    fn load_authoritative_session_transaction(
        &self,
        transaction: &mut super::host_session_authority::store::LegacyWriterGuard,
        orchestration_session_id: &str,
    ) -> Result<Option<OrchestrationSessionRecord>> {
        use super::host_session_authority::store::LegacyStateStoreCollectionV1::Sessions;
        let canonical = Self::transaction_read_json::<OrchestrationSessionRecord>(
            transaction,
            Sessions,
            &[orchestration_session_id, "session.json"],
        )?;
        if let Some(session) = canonical {
            self.validate_session_record(&session)?;
            return Ok(Some(session));
        }
        let flat_name = format!("{orchestration_session_id}.json");
        let flat = Self::transaction_read_json::<OrchestrationSessionRecord>(
            transaction,
            Sessions,
            &[flat_name.as_str()],
        )?;
        if let Some(session) = flat.as_ref() {
            self.validate_session_record(session)?;
        }
        Ok(flat)
    }

    fn load_inbox_item_transaction(
        &self,
        transaction: &mut super::host_session_authority::store::LegacyWriterGuard,
        orchestration_session_id: &str,
        item_id: &str,
    ) -> Result<Option<DurableInboxItemRecord>> {
        use super::host_session_authority::store::LegacyStateStoreCollectionV1::Sessions;
        let file_name = format!("{item_id}.json");
        let item = Self::transaction_read_json::<DurableInboxItemRecord>(
            transaction,
            Sessions,
            &[orchestration_session_id, "inbox", file_name.as_str()],
        )?;
        if let Some(item) = item.as_ref() {
            self.validate_inbox_item_record(item)?;
            if item.orchestration_session_id != orchestration_session_id || item.item_id != item_id
            {
                anyhow::bail!("durable inbox artifact identity mismatch");
            }
        }
        Ok(item)
    }

    fn load_obligation_transaction(
        &self,
        transaction: &mut super::host_session_authority::store::LegacyWriterGuard,
        orchestration_session_id: &str,
        obligation_id: &str,
    ) -> Result<Option<OrchestrationObligationRecord>> {
        use super::host_session_authority::store::LegacyStateStoreCollectionV1::Sessions;
        let file_name = format!("{obligation_id}.json");
        let obligation = Self::transaction_read_json::<OrchestrationObligationRecord>(
            transaction,
            Sessions,
            &[orchestration_session_id, "obligations", file_name.as_str()],
        )?;
        if let Some(obligation) = obligation.as_ref() {
            self.validate_obligation_record(obligation)?;
            if obligation.orchestration_session_id != orchestration_session_id
                || obligation.obligation_id != obligation_id
            {
                anyhow::bail!("orchestration obligation artifact identity mismatch");
            }
        }
        Ok(obligation)
    }

    fn list_obligations_transaction(
        &self,
        transaction: &mut super::host_session_authority::store::LegacyWriterGuard,
        orchestration_session_id: &str,
    ) -> Result<Vec<OrchestrationObligationRecord>> {
        use super::host_session_authority::store::LegacyStateStoreCollectionV1::Sessions;
        let mut obligations = Self::transaction_list_json::<OrchestrationObligationRecord>(
            transaction,
            Sessions,
            &[orchestration_session_id, "obligations"],
        )?;
        for obligation in &obligations {
            self.validate_obligation_record(obligation)?;
            if obligation.orchestration_session_id != orchestration_session_id {
                anyhow::bail!("orchestration obligation belongs to another session");
            }
        }
        obligations.sort_by(|left, right| {
            left.created_at
                .cmp(&right.created_at)
                .then(left.obligation_id.cmp(&right.obligation_id))
        });
        Ok(obligations)
    }

    fn load_session_transaction(
        &self,
        transaction: &mut super::host_session_authority::store::LegacyWriterGuard,
        orchestration_session_id: &str,
    ) -> Result<Option<AgentRuntimeSessionRecord>> {
        let session =
            self.load_authoritative_session_transaction(transaction, orchestration_session_id)?;
        let participants = self
            .list_participants_across_sources_transaction(transaction)?
            .into_iter()
            .filter(|participant| {
                participant.handle.orchestration_session_id == orchestration_session_id
            })
            .collect::<Vec<_>>();
        if session.is_none() && participants.is_empty() {
            return Ok(None);
        }
        Ok(Some(self.build_session_record(
            orchestration_session_id,
            session,
            participants,
        )))
    }

    fn write_obligation_transaction(
        transaction: &mut super::host_session_authority::store::LegacyWriterGuard,
        obligation: &OrchestrationObligationRecord,
    ) -> Result<()> {
        use super::host_session_authority::store::LegacyStateStoreCollectionV1::Sessions;
        let file_name = format!("{}.json", obligation.obligation_id);
        Self::transaction_write_json(
            transaction,
            Sessions,
            &[
                obligation.orchestration_session_id.as_str(),
                "obligations",
                file_name.as_str(),
            ],
            obligation,
        )
    }

    #[cfg(any(target_os = "linux", test))]
    fn load_exact_pending_obligation_transaction(
        &self,
        transaction: &mut super::host_session_authority::store::LegacyWriterGuard,
        orchestration_session_id: &str,
        obligation_id: &str,
        expected_kind: OrchestrationObligationKind,
        binding: (&str, &str, &str, u64),
    ) -> Result<OrchestrationObligationRecord> {
        let (target_participant_id, target_backend_id, world_id, world_generation) = binding;
        let approval = expected_kind == OrchestrationObligationKind::ApprovalRequired;
        let obligation = self
            .load_obligation_transaction(transaction, orchestration_session_id, obligation_id)?
            .ok_or_else(|| {
                if approval {
                    anyhow::anyhow!(
                        "approval_obligation_not_found: orchestration session {} has no approval obligation {}",
                        orchestration_session_id,
                        obligation_id
                    )
                } else {
                    anyhow::anyhow!(
                        "follow_up_obligation_not_found: orchestration session {} has no follow-up obligation {}",
                        orchestration_session_id,
                        obligation_id
                    )
                }
            })?;
        if obligation.kind != expected_kind {
            if approval {
                anyhow::bail!(
                    "approval_obligation_kind_mismatch: orchestration session {} obligation {} is not approval_required",
                    orchestration_session_id,
                    obligation_id
                );
            }
            anyhow::bail!(
                "follow_up_obligation_kind_mismatch: orchestration session {} obligation {} is not follow_up_required",
                orchestration_session_id,
                obligation_id
            );
        }
        if !obligation.is_pending() {
            if approval {
                anyhow::bail!(
                    "approval_obligation_already_resolved: orchestration session {} approval obligation {} is already closed",
                    orchestration_session_id,
                    obligation_id
                );
            }
            anyhow::bail!(
                "follow_up_obligation_already_resolved: orchestration session {} follow-up obligation {} is already closed",
                orchestration_session_id,
                obligation_id
            );
        }
        if obligation.source_participant_id.as_deref() != Some(target_participant_id) {
            let prefix = if approval { "approval" } else { "follow_up" };
            anyhow::bail!(
                "{}_obligation_target_mismatch: orchestration session {} {} obligation {} does not bind retained worker {}",
                prefix,
                orchestration_session_id,
                if approval { "approval" } else { "follow-up" },
                obligation_id,
                target_participant_id
            );
        }
        if obligation.target_backend_id.as_deref() != Some(target_backend_id) {
            let prefix = if approval { "approval" } else { "follow_up" };
            anyhow::bail!(
                "{}_obligation_backend_mismatch: orchestration session {} {} obligation {} does not bind backend {}",
                prefix,
                orchestration_session_id,
                if approval { "approval" } else { "follow-up" },
                obligation_id,
                target_backend_id
            );
        }
        if obligation.world_id.as_deref() != Some(world_id)
            || obligation.world_generation != Some(world_generation)
        {
            let prefix = if approval { "approval" } else { "follow_up" };
            anyhow::bail!(
                "{}_obligation_world_binding_mismatch: orchestration session {} {} obligation {} no longer matches authoritative world binding {}/{}",
                prefix,
                orchestration_session_id,
                if approval { "approval" } else { "follow-up" },
                obligation_id,
                world_id,
                world_generation
            );
        }
        Ok(obligation)
    }

    pub(crate) fn participants_dir(&self) -> PathBuf {
        self.substrate_home
            .join("run")
            .join("agent-hub")
            .join("participants")
    }

    pub(crate) fn handles_dir(&self) -> PathBuf {
        self.substrate_home
            .join("run")
            .join("agent-hub")
            .join("handles")
    }

    pub(crate) fn sessions_dir(&self) -> PathBuf {
        self.substrate_home
            .join("run")
            .join("agent-hub")
            .join("sessions")
    }

    #[allow(dead_code)]
    pub(crate) fn host_inbox_dir(&self) -> PathBuf {
        self.substrate_home.join("host_inbox")
    }

    #[cfg(any(target_os = "linux", test))]
    fn invalid_host_inbox_artifacts_dir(&self) -> PathBuf {
        self.host_inbox_dir().join(".invalid_artifacts")
    }

    fn canonical_session_dir(&self, orchestration_session_id: &str) -> PathBuf {
        self.sessions_dir().join(orchestration_session_id)
    }

    fn canonical_session_path(&self, orchestration_session_id: &str) -> PathBuf {
        self.canonical_session_dir(orchestration_session_id)
            .join("session.json")
    }

    fn canonical_participants_dir(&self, orchestration_session_id: &str) -> PathBuf {
        self.canonical_session_dir(orchestration_session_id)
            .join("participants")
    }

    #[cfg(test)]
    fn canonical_participant_path(
        &self,
        orchestration_session_id: &str,
        participant_id: &str,
    ) -> PathBuf {
        self.canonical_participants_dir(orchestration_session_id)
            .join(format!("{participant_id}.json"))
    }

    #[cfg(test)]
    fn canonical_leases_dir(&self, orchestration_session_id: &str) -> PathBuf {
        self.canonical_session_dir(orchestration_session_id)
            .join("leases")
    }

    #[allow(dead_code)]
    pub(crate) fn canonical_inbox_dir(&self, orchestration_session_id: &str) -> PathBuf {
        self.canonical_session_dir(orchestration_session_id)
            .join("inbox")
    }

    #[allow(dead_code)]
    pub(crate) fn canonical_obligations_dir(&self, orchestration_session_id: &str) -> PathBuf {
        self.canonical_session_dir(orchestration_session_id)
            .join("obligations")
    }

    pub(crate) fn canonical_active_ephemeral_tasks_dir(
        &self,
        orchestration_session_id: &str,
    ) -> PathBuf {
        self.canonical_session_dir(orchestration_session_id)
            .join("active-ephemeral-tasks")
    }

    #[allow(dead_code)]
    pub(crate) fn canonical_inbox_item_path(
        &self,
        orchestration_session_id: &str,
        item_id: &str,
    ) -> PathBuf {
        self.canonical_inbox_dir(orchestration_session_id)
            .join(format!("{item_id}.json"))
    }

    #[allow(dead_code)]
    pub(crate) fn canonical_obligation_path(
        &self,
        orchestration_session_id: &str,
        obligation_id: &str,
    ) -> PathBuf {
        self.canonical_obligations_dir(orchestration_session_id)
            .join(format!("{obligation_id}.json"))
    }

    #[allow(dead_code)]
    pub(crate) fn host_inbox_record_path(&self, record_id: &str) -> Result<PathBuf> {
        HostInboxRecord::validate_record_id(record_id)?;
        Ok(self.host_inbox_dir().join(format!("{record_id}.json")))
    }

    #[cfg(any(target_os = "linux", test))]
    fn invalid_host_inbox_artifact_record_path(&self, record_id: &str) -> Result<PathBuf> {
        HostInboxRecord::validate_record_id(record_id)?;
        Ok(self
            .invalid_host_inbox_artifacts_dir()
            .join(format!("{record_id}.json")))
    }

    fn host_inbox_record_id_from_path(path: &Path) -> Result<String> {
        let Some(record_id) = path.file_stem().and_then(|value| value.to_str()) else {
            anyhow::bail!(
                "host inbox artifact {} must have a UTF-8 json path stem",
                path.display()
            );
        };
        HostInboxRecord::validate_record_id(record_id)
            .with_context(|| format!("invalid host inbox artifact path {}", path.display()))?;
        Ok(record_id.to_string())
    }

    pub(crate) fn canonical_active_ephemeral_task_path(
        &self,
        orchestration_session_id: &str,
        task_run_id: &str,
    ) -> PathBuf {
        self.canonical_active_ephemeral_tasks_dir(orchestration_session_id)
            .join(format!("{task_run_id}.json"))
    }

    #[cfg(test)]
    fn canonical_lease_path(
        &self,
        orchestration_session_id: &str,
        participant_id: &str,
    ) -> PathBuf {
        self.canonical_leases_dir(orchestration_session_id)
            .join(format!("{participant_id}.lease"))
    }

    #[cfg(test)]
    fn participant_path(&self, participant_id: &str) -> PathBuf {
        self.participants_dir()
            .join(format!("{participant_id}.json"))
    }

    fn orchestration_session_path(&self, orchestration_session_id: &str) -> PathBuf {
        self.sessions_dir()
            .join(format!("{orchestration_session_id}.json"))
    }

    #[cfg(test)]
    fn lease_path(&self, participant_id: &str) -> PathBuf {
        self.participants_dir()
            .join(format!("{participant_id}.lease"))
    }

    pub(crate) fn persist_participant(
        &self,
        participant: &AgentRuntimeParticipantRecord,
    ) -> Result<()> {
        self.with_legacy_snapshot_transaction(|transaction| {
            self.validate_participant_record(participant)?;
            if let Some(existing) = self
                .list_participants_across_sources_transaction(transaction)?
                .into_iter()
                .find(|existing| {
                    existing.handle.participant_id == participant.handle.participant_id
                })
            {
                if !should_persist_participant_snapshot(&existing, participant) {
                    return Ok(());
                }
            }
            self.write_participant_snapshot(transaction, participant)
        })
    }

    fn write_participant_snapshot(
        &self,
        transaction: &mut super::host_session_authority::store::LegacyWriterGuard,
        participant: &AgentRuntimeParticipantRecord,
    ) -> Result<()> {
        use super::host_session_authority::store::LegacyStateStoreCollectionV1::{
            Participants, Sessions,
        };
        let flat_name = format!("{}.json", participant.handle.participant_id);
        Self::transaction_write_json(
            transaction,
            Participants,
            &[flat_name.as_str()],
            participant,
        )?;
        Self::transaction_write_json(
            transaction,
            Sessions,
            &[
                participant.handle.orchestration_session_id.as_str(),
                "participants",
                flat_name.as_str(),
            ],
            participant,
        )?;
        self.persist_lease(transaction, participant)
    }

    pub(crate) fn load_participant(
        &self,
        participant_id: &str,
    ) -> Result<Option<AgentRuntimeParticipantRecord>> {
        Ok(self
            .list_participants_across_sources()?
            .into_iter()
            .find(|participant| participant.handle.participant_id == participant_id))
    }

    pub(crate) fn list_participants(&self) -> Result<Vec<AgentRuntimeParticipantRecord>> {
        self.list_participants_across_sources()
    }

    pub(crate) fn list_live_participants(&self) -> Result<Vec<AgentRuntimeParticipantRecord>> {
        Ok(self
            .list_participants_across_sources()?
            .into_iter()
            .filter(|participant| {
                participant.is_authoritative_live() && owner_process_is_alive(participant)
            })
            .collect())
    }

    #[allow(dead_code)]
    pub(crate) fn list_invalidated_participants(
        &self,
    ) -> Result<Vec<AgentRuntimeParticipantRecord>> {
        Ok(self
            .read_participant_dir(&self.participants_dir())?
            .into_iter()
            .filter(|participant| {
                participant.handle.state == super::session::AgentRuntimeSessionState::Invalidated
            })
            .collect())
    }

    pub(crate) fn list_participants_across_sources(
        &self,
    ) -> Result<Vec<AgentRuntimeParticipantRecord>> {
        let mut participants = BTreeMap::new();

        for (participant, source) in self.read_canonical_participants()? {
            participants.insert(
                participant.handle.participant_id.clone(),
                (participant, source),
            );
        }

        for participant in self.read_participant_dir(&self.participants_dir())? {
            participants
                .entry(participant.handle.participant_id.clone())
                .or_insert((participant, ParticipantRecordSource::Flat));
        }

        for participant in self.read_participant_dir(&self.handles_dir())? {
            participants
                .entry(participant.handle.participant_id.clone())
                .or_insert((participant, ParticipantRecordSource::Legacy));
        }

        let mut participants = participants
            .into_values()
            .map(|(participant, _)| participant)
            .collect::<Vec<_>>();
        participants.sort_by(|left, right| {
            left.handle
                .last_transition_at
                .cmp(&right.handle.last_transition_at)
                .then(left.handle.participant_id.cmp(&right.handle.participant_id))
        });
        Ok(participants)
    }

    pub(crate) fn list_invalidated_participants_across_sources(
        &self,
    ) -> Result<Vec<AgentRuntimeParticipantRecord>> {
        Ok(self
            .list_participants_across_sources()?
            .into_iter()
            .filter(|participant| {
                participant.handle.state == super::session::AgentRuntimeSessionState::Invalidated
            })
            .collect())
    }

    #[allow(dead_code)]
    pub(crate) fn list_live_participants_for_session(
        &self,
        orchestration_session_id: &str,
    ) -> Result<Vec<AgentRuntimeParticipantRecord>> {
        Ok(self
            .list_live_participants()?
            .into_iter()
            .filter(|participant| {
                participant.handle.orchestration_session_id == orchestration_session_id
            })
            .collect())
    }

    #[cfg_attr(not(any(target_os = "linux", test)), allow(dead_code))]
    pub(crate) fn register_active_ephemeral_world_task(
        &self,
        record: ActiveEphemeralWorldTaskRecord,
    ) -> Result<ActiveEphemeralWorldTaskGuard> {
        self.with_legacy_snapshot_transaction(|transaction| {
            use super::host_session_authority::store::LegacyStateStoreCollectionV1::Sessions;
            record.validate()?;
            let file_name = format!("{}.json", record.task_run_id);
            let descendants = [
                record.orchestration_session_id.as_str(),
                "active-ephemeral-tasks",
                file_name.as_str(),
            ];
            if Self::transaction_read_json::<ActiveEphemeralWorldTaskRecord>(
                transaction,
                Sessions,
                &descendants,
            )?
            .is_some()
            {
                anyhow::bail!(
                    "duplicate_active_task_run_id: active ephemeral task {} is already registered",
                    record.task_run_id
                );
            }
            Self::transaction_write_json(transaction, Sessions, &descendants, &record)?;
            Ok(ActiveEphemeralWorldTaskGuard {
                store: self.clone(),
                orchestration_session_id: record.orchestration_session_id.clone(),
                task_run_id: record.task_run_id.clone(),
            })
        })
    }

    pub(crate) fn load_active_ephemeral_world_task(
        &self,
        orchestration_session_id: &str,
        task_run_id: &str,
    ) -> Result<Option<ActiveEphemeralWorldTaskRecord>> {
        let path = self.canonical_active_ephemeral_task_path(orchestration_session_id, task_run_id);
        let Some(record) = read_json_if_exists::<ActiveEphemeralWorldTaskRecord>(&path)? else {
            return Ok(None);
        };
        record.validate()?;
        if record.orchestration_session_id != orchestration_session_id {
            anyhow::bail!(
                "active_task_registry_mismatch: {} stored orchestration session {} not {}",
                path.display(),
                record.orchestration_session_id,
                orchestration_session_id
            );
        }
        if record.task_run_id != task_run_id {
            anyhow::bail!(
                "active_task_registry_mismatch: {} stored task_run_id {} not {}",
                path.display(),
                record.task_run_id,
                task_run_id
            );
        }

        Ok(Some(record))
    }

    #[allow(dead_code)]
    pub(crate) fn list_active_ephemeral_world_tasks(
        &self,
        orchestration_session_id: &str,
    ) -> Result<Vec<ActiveEphemeralWorldTaskRecord>> {
        let dir = self.canonical_active_ephemeral_tasks_dir(orchestration_session_id);
        if !dir.exists() {
            return Ok(Vec::new());
        }

        let mut records = Vec::new();
        for entry in
            fs::read_dir(&dir).with_context(|| format!("failed to read {}", dir.display()))?
        {
            let entry = entry.with_context(|| format!("failed to iterate {}", dir.display()))?;
            let path = entry.path();
            if path.extension().and_then(|ext| ext.to_str()) != Some("json") {
                continue;
            }
            let record: ActiveEphemeralWorldTaskRecord = serde_json::from_str(
                &fs::read_to_string(&path)
                    .with_context(|| format!("failed to read {}", path.display()))?,
            )
            .with_context(|| format!("failed to parse {}", path.display()))?;
            record.validate()?;
            if record.orchestration_session_id != orchestration_session_id {
                anyhow::bail!(
                    "active_task_registry_mismatch: {} stored orchestration session {} not {}",
                    path.display(),
                    record.orchestration_session_id,
                    orchestration_session_id
                );
            }
            records.push(record);
        }
        records.sort_by(|left, right| left.task_run_id.cmp(&right.task_run_id));
        Ok(records)
    }

    #[cfg_attr(not(any(target_os = "linux", test)), allow(dead_code))]
    fn remove_active_ephemeral_world_task(
        &self,
        orchestration_session_id: &str,
        task_run_id: &str,
    ) -> Result<()> {
        self.with_legacy_snapshot_transaction(|transaction| {
            use super::host_session_authority::store::LegacyStateStoreCollectionV1::Sessions;
            let file_name = format!("{task_run_id}.json");
            Self::transaction_remove_file(
                transaction,
                Sessions,
                &[
                    orchestration_session_id,
                    "active-ephemeral-tasks",
                    file_name.as_str(),
                ],
            )?;
            Ok(())
        })
    }

    pub(crate) fn invalidate_stale_world_members_for_session(
        &self,
        orchestration_session_id: &str,
        active_generation: u64,
    ) -> Result<Vec<String>> {
        self.with_legacy_snapshot_transaction(|transaction| {
            let mut invalidated_participant_ids = Vec::new();
            let mut invalidated_participants = Vec::new();
            for mut participant in self.list_participants_across_sources_transaction(transaction)? {
                if participant.handle.orchestration_session_id != orchestration_session_id
                    || participant.handle.role != MEMBER_ROLE
                    || participant.handle.execution.scope != AgentExecutionScope::World
                    || !participant.is_authoritative_live()
                {
                    continue;
                }

                let Some(world_generation) = participant.handle.world_generation else {
                    continue;
                };
                if world_generation >= active_generation {
                    continue;
                }

                if participant.invalidate_for_world_generation_rollover() {
                    invalidated_participant_ids.push(participant.handle.participant_id.clone());
                    invalidated_participants.push(participant);
                }
            }
            for participant in &invalidated_participants {
                self.validate_participant_record(participant)?;
                self.write_participant_snapshot(transaction, participant)?;
            }
            Ok(invalidated_participant_ids)
        })
    }

    pub(crate) fn load_session(
        &self,
        orchestration_session_id: &str,
    ) -> Result<Option<AgentRuntimeSessionRecord>> {
        let session = self.load_authoritative_session(orchestration_session_id)?;
        let participants = self
            .list_participants_across_sources()?
            .into_iter()
            .filter(|participant| {
                participant.handle.orchestration_session_id == orchestration_session_id
            })
            .collect::<Vec<_>>();
        if session.is_none() && participants.is_empty() {
            return Ok(None);
        }

        Ok(Some(self.build_session_record(
            orchestration_session_id,
            session,
            participants,
        )))
    }

    pub(crate) fn list_sessions(&self) -> Result<Vec<AgentRuntimeSessionRecord>> {
        let mut session_ids = BTreeSet::new();

        for session_id in self.canonical_session_root_ids()? {
            session_ids.insert(session_id);
        }
        for session_id in self.flat_session_ids()? {
            session_ids.insert(session_id);
        }
        for participant in self.list_participants_across_sources()? {
            session_ids.insert(participant.handle.orchestration_session_id.clone());
        }

        let mut sessions = Vec::new();
        for session_id in session_ids {
            if let Some(record) = self.load_session(&session_id)? {
                sessions.push(record);
            }
        }

        sessions.sort_by(|left, right| {
            left.last_updated_at().cmp(&right.last_updated_at()).then(
                left.orchestration_session_id()
                    .cmp(right.orchestration_session_id()),
            )
        });
        Ok(sessions)
    }

    pub(crate) fn list_status_sessions_for_agent(
        &self,
        orchestrator_agent_id: &str,
    ) -> Result<Vec<AgentRuntimeSessionRecord>> {
        let _ = orchestrator_agent_id;
        self.list_sessions()
    }

    #[allow(dead_code)]
    pub(crate) fn list_live_sessions(&self) -> Result<Vec<AgentRuntimeSessionRecord>> {
        Ok(self
            .list_sessions()?
            .into_iter()
            .filter(|record| {
                record.is_complete()
                    && record.session.state == OrchestrationSessionState::Active
                    && owner_pid_is_alive(record.session.shell_owner_pid)
            })
            .collect())
    }

    pub(crate) fn resolve_single_live_session_for_agent(
        &self,
        orchestrator_agent_id: &str,
    ) -> Result<Option<AgentRuntimeSessionRecord>> {
        let active_candidates = self
            .list_sessions()?
            .into_iter()
            .filter(|record| {
                record.has_authoritative_parent()
                    && record.session.orchestrator_agent_id == orchestrator_agent_id
                    && record.session.state == OrchestrationSessionState::Active
                    && owner_pid_is_alive(record.session.shell_owner_pid)
            })
            .collect::<Vec<_>>();
        if active_candidates.len() > 1 {
            anyhow::bail!(
                "multiple active orchestration session candidates found for agent {orchestrator_agent_id}"
            );
        }

        let live_host_orchestrators = self
            .list_live_participants()?
            .into_iter()
            .filter(|participant| {
                participant.handle.agent_id == orchestrator_agent_id
                    && participant.handle.role == ORCHESTRATOR_ROLE
                    && participant.handle.execution.scope == AgentExecutionScope::Host
            })
            .collect::<Vec<_>>();
        if live_host_orchestrators.len() > 1 {
            anyhow::bail!(
                "multiple live orchestrator participant candidates found for agent {orchestrator_agent_id}"
            );
        }

        let Some(record) = active_candidates.into_iter().next() else {
            return if live_host_orchestrators.is_empty() {
                Ok(None)
            } else {
                Err(anyhow::anyhow!(
                    "live host-scoped orchestrator participant exists for agent {orchestrator_agent_id} without an active parent session"
                ))
            };
        };

        let active_participant_id =
            record
                .session
                .active_session_handle_id
                .clone()
                .ok_or_else(|| {
                    anyhow::anyhow!(
                        "active orchestration session {} is missing active_session_handle_id",
                        record.session.orchestration_session_id
                    )
                })?;

        let participant = record
            .participants
            .iter()
            .find(|participant| participant.handle.participant_id == active_participant_id)
            .cloned()
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "active orchestration session {} references missing participant {}",
                    record.session.orchestration_session_id,
                    active_participant_id
                )
            })?;

        if !participant.is_authoritative_live() || !owner_process_is_alive(&participant) {
            anyhow::bail!(
                "active orchestration session {} references inactive participant {}",
                record.session.orchestration_session_id,
                active_participant_id
            );
        }
        if participant.handle.agent_id != orchestrator_agent_id {
            anyhow::bail!(
                "active orchestration session {} belongs to agent {} not {}",
                record.session.orchestration_session_id,
                participant.handle.agent_id,
                orchestrator_agent_id
            );
        }
        if participant.handle.orchestration_session_id != record.session.orchestration_session_id {
            anyhow::bail!(
                "active orchestration session {} does not match participant {} parent {}",
                record.session.orchestration_session_id,
                active_participant_id,
                participant.handle.orchestration_session_id
            );
        }
        if participant.handle.role != ORCHESTRATOR_ROLE
            || participant.handle.execution.scope != AgentExecutionScope::Host
        {
            anyhow::bail!(
                "active orchestration session {} references non-host orchestrator participant {}",
                record.session.orchestration_session_id,
                active_participant_id
            );
        }
        if live_host_orchestrators
            .iter()
            .any(|candidate| candidate.handle.participant_id != participant.handle.participant_id)
        {
            anyhow::bail!(
                "multiple live orchestrator participant candidates found for agent {orchestrator_agent_id}"
            );
        }

        Ok(Some(record))
    }

    pub(crate) fn resolve_public_control_target(
        &self,
        orchestration_session_id: &str,
        action: PublicControlAction,
    ) -> Result<ResolvedPublicControlTarget> {
        let resolved = self.resolve_public_session_authority(orchestration_session_id)?;
        if matches!(action, PublicControlAction::Stop)
            && resolved.session_posture == PublicSessionPosture::Terminal
        {
            anyhow::bail!(
                "owner_unreachable: orchestration session {} no longer has a reachable retained owner",
                orchestration_session_id
            );
        }
        if matches!(action, PublicControlAction::Fork) && resolved.host_attach_contract.is_none() {
            anyhow::bail!(
                "owner_unreachable: orchestration session {} is missing durable host attach contract state",
                orchestration_session_id
            );
        }
        if matches!(action, PublicControlAction::Fork)
            && resolved
                .host_attach_contract
                .as_ref()
                .is_some_and(|contract| !contract.supports_fork())
        {
            anyhow::bail!(
                "owner_unreachable: orchestration session {} durable host attach contract does not allow fork",
                orchestration_session_id
            );
        }
        if matches!(action, PublicControlAction::Stop)
            && resolved
                .host_attach_contract
                .as_ref()
                .is_some_and(|contract| !contract.supports_stop())
        {
            anyhow::bail!(
                "owner_unreachable: orchestration session {} durable host attach contract does not allow stop",
                orchestration_session_id
            );
        }
        if matches!(
            (action, resolved.session_posture),
            (
                PublicControlAction::Stop,
                PublicSessionPosture::DetachedReattachable
            )
        ) && !resolved.participant.is_resume_eligible()
        {
            anyhow::bail!(
                "owner_unreachable: orchestration session {} no longer has a resume-eligible retained owner",
                orchestration_session_id
            );
        }
        if matches!(
            (action, resolved.session_posture),
            (
                PublicControlAction::Stop,
                PublicSessionPosture::DetachedReattachable
            )
        ) && resolved.participant.internal_uaa_session_id().is_none()
        {
            anyhow::bail!(
                "missing_internal_session_id: orchestration session {} active participant {} is missing internal.uaa_session_id",
                orchestration_session_id,
                resolved.participant.handle.participant_id
            );
        }

        Ok(ResolvedPublicControlTarget {
            session: resolved.session,
            active_participant: resolved.participant,
            session_posture: resolved.session_posture,
            host_attach_contract: resolved.host_attach_contract,
        })
    }

    pub(crate) fn resolve_public_attach_target(
        &self,
        orchestration_session_id: &str,
        action: PublicAttachAction,
    ) -> Result<ResolvedPublicAttachTarget> {
        let resolved = self.resolve_public_session_authority(orchestration_session_id)?;

        if resolved.session_posture == PublicSessionPosture::Active {
            anyhow::bail!(
                "session_already_owned: orchestration session {} already has a live retained owner",
                orchestration_session_id
            );
        }
        let host_attach_contract = resolved.host_attach_contract.as_ref().ok_or_else(|| {
            anyhow::anyhow!(
                "owner_unreachable: orchestration session {} is missing durable host attach contract state",
                orchestration_session_id
            )
        })?;
        if !host_attach_contract.supports_public_attach_continuity() {
            anyhow::bail!(
                "owner_unreachable: orchestration session {} durable host attach contract does not allow continuity attach",
                orchestration_session_id
            );
        }
        if host_attach_contract
            .public_attach_continuity_session_id()
            .is_none()
        {
            anyhow::bail!(
                "owner_unreachable: orchestration session {} no longer has continuity required for {}",
                orchestration_session_id,
                action.continuity_label()
            );
        }
        if !resolved.participant.is_public_attach_continuity_source() {
            anyhow::bail!(
                "owner_unreachable: orchestration session {} no longer has a retained owner that can supply continuity for {}",
                orchestration_session_id,
                action.continuity_label()
            );
        }
        if resolved.session_posture != PublicSessionPosture::DetachedReattachable {
            anyhow::bail!(
                "session_not_reattachable: orchestration session {} must be in detached reattachable posture before {} (resolved posture: {:?})",
                orchestration_session_id,
                action.continuity_label(),
                resolved.session_posture
            );
        }

        Ok(ResolvedPublicAttachTarget {
            session: resolved.session,
            active_participant: resolved.participant,
            session_posture: resolved.session_posture,
            host_attach_contract: resolved.host_attach_contract,
        })
    }

    pub(crate) fn resolve_internal_world_dispatch_caller(
        &self,
        orchestration_session_id: &str,
        caller_participant_id: &str,
    ) -> Result<ResolvedInternalWorldDispatchCaller> {
        let Some(record) = self.load_session(orchestration_session_id)? else {
            anyhow::bail!(
                "missing_orchestration_session: internal world dispatch requires authoritative orchestration session {}",
                orchestration_session_id
            );
        };

        let resolved = resolve_authoritative_session_control(&record, orchestration_session_id)?;
        if resolved.participant.participant_id() != caller_participant_id {
            anyhow::bail!(
                "caller_not_authoritative: orchestration session {} authoritative orchestrator participant is {} not {}",
                orchestration_session_id,
                resolved.participant.participant_id(),
                caller_participant_id
            );
        }

        Ok(ResolvedInternalWorldDispatchCaller {
            session: resolved.session,
            caller_participant: resolved.participant,
        })
    }

    fn resolve_public_session_authority(
        &self,
        orchestration_session_id: &str,
    ) -> Result<ResolvedPublicSessionAuthority> {
        let Some(record) = self.load_session(orchestration_session_id)? else {
            return Err(self.public_session_selector_error(orchestration_session_id));
        };
        let resolved = resolve_authoritative_session_control(&record, orchestration_session_id)?;

        if session_requires_linux_first_public_control_posture(&record)
            && !cfg!(target_os = "linux")
        {
            anyhow::bail!(
                "unsupported_platform_or_posture: orchestration session {} requires Linux world-sensitive control posture",
                orchestration_session_id
            );
        }

        Ok(ResolvedPublicSessionAuthority {
            host_attach_contract: resolved.session.host_attach_contract().cloned(),
            session: resolved.session,
            participant: resolved.participant,
            session_posture: resolved.session_posture,
        })
    }

    #[cfg(any(target_os = "linux", test))]
    pub(crate) fn resolve_internal_continue_world_dispatch_target(
        &self,
        orchestration_session_id: &str,
        caller_participant_id: &str,
        target_participant_id: &str,
        target_backend_id: &str,
    ) -> Result<ResolvedInternalContinueWorldDispatchTarget> {
        let Some(record) = self.load_session(orchestration_session_id)? else {
            anyhow::bail!(
                "missing_orchestration_session: internal world dispatch requires authoritative orchestration session {}",
                orchestration_session_id
            );
        };

        let authoritative =
            resolve_authoritative_session_control(&record, orchestration_session_id)?;
        if authoritative.participant.participant_id() != caller_participant_id {
            anyhow::bail!(
                "caller_not_authoritative: orchestration session {} authoritative orchestrator participant is {} not {}",
                orchestration_session_id,
                authoritative.participant.participant_id(),
                caller_participant_id
            );
        }

        let mut matching_participants = record
            .participants
            .iter()
            .filter(|participant| participant.participant_id() == target_participant_id)
            .cloned()
            .collect::<Vec<_>>();

        if matching_participants.is_empty() {
            anyhow::bail!(
                "target_not_in_session: orchestration session {} has no exact retained worker {}",
                orchestration_session_id,
                target_participant_id
            );
        }
        if matching_participants.len() > 1 {
            anyhow::bail!(
                "ambiguous_target_participant: orchestration session {} has multiple retained worker records for {}",
                orchestration_session_id,
                target_participant_id
            );
        }

        let target_participant = matching_participants
            .pop()
            .expect("target participant count checked above");
        if target_participant.handle.backend_id != target_backend_id {
            anyhow::bail!(
                "backend_mismatch: orchestration session {} retained worker {} backend is {} not {}",
                orchestration_session_id,
                target_participant_id,
                target_participant.handle.backend_id,
                target_backend_id
            );
        }
        if target_participant.handle.role != MEMBER_ROLE
            || target_participant.handle.execution.scope != AgentExecutionScope::World
        {
            anyhow::bail!(
                "invalid_target_participant: orchestration session {} participant {} is not a retained world worker",
                orchestration_session_id,
                target_participant_id
            );
        }
        validate_retained_worker_authoritative_lineage(
            &record,
            &authoritative.participant,
            &target_participant,
        )?;
        if !target_participant.matches_authoritative_parent_world_binding(&authoritative.session) {
            anyhow::bail!(
                "world_binding_mismatch: orchestration session {} retained worker {} no longer matches the authoritative world binding",
                orchestration_session_id,
                target_participant_id
            );
        }
        if !exact_continue_retained_worker_is_routable(
            &authoritative.session,
            &authoritative.participant,
            &target_participant,
        ) {
            anyhow::bail!(
                "stale_linkage: orchestration session {} retained worker {} is no longer authoritative-live",
                orchestration_session_id,
                target_participant_id
            );
        }

        Ok(ResolvedInternalContinueWorldDispatchTarget {
            session: authoritative.session,
            caller_participant: authoritative.participant,
            target_participant,
        })
    }

    #[cfg(any(target_os = "linux", test))]
    pub(crate) fn resolve_internal_continue_fork_command_dispatch_target(
        &self,
        orchestration_session_id: &str,
        caller_participant_id: &str,
        target_participant_id: &str,
        target_backend_id: &str,
    ) -> Result<ResolvedInternalContinueWorldDispatchTarget> {
        let resolved = self.resolve_internal_fork_world_dispatch_target(
            orchestration_session_id,
            caller_participant_id,
            target_participant_id,
            target_backend_id,
        )?;
        if !exact_continue_retained_worker_is_routable(
            &resolved.session,
            &resolved.caller_participant,
            &resolved.source_participant,
        ) {
            anyhow::bail!(
                "stale_linkage: orchestration session {} retained worker {} is no longer authoritative-live",
                orchestration_session_id,
                target_participant_id
            );
        }

        Ok(ResolvedInternalContinueWorldDispatchTarget {
            session: resolved.session,
            caller_participant: resolved.caller_participant,
            target_participant: resolved.source_participant,
        })
    }

    #[cfg(any(target_os = "linux", test))]
    #[allow(dead_code)]
    fn prepare_internal_continue_approval_response_obligation_closeout(
        &self,
        resolved_target: &ResolvedInternalContinueWorldDispatchTarget,
        payload: &WorkerContinueApprovalResponsePayloadV1,
    ) -> Result<PreparedInternalApprovalResponseObligationCloseout> {
        let obligation = self.load_exact_pending_approval_obligation_for_continue_target(
            resolved_target.orchestration_session_id(),
            resolved_target.target_participant.participant_id(),
            &resolved_target.target_participant.handle.backend_id,
            resolved_target
                .session
                .world_id
                .as_deref()
                .expect("continue target must keep authoritative world binding"),
            resolved_target
                .session
                .world_generation
                .expect("continue target must keep authoritative world binding"),
            &payload.approval_obligation_id,
        )?;

        Ok(PreparedInternalApprovalResponseObligationCloseout {
            orchestration_session_id: obligation.orchestration_session_id,
            approval_obligation_id: obligation.obligation_id,
            target_participant_id: resolved_target
                .target_participant
                .participant_id()
                .to_string(),
            target_backend_id: resolved_target.target_participant.handle.backend_id.clone(),
            world_id: resolved_target
                .session
                .world_id
                .clone()
                .expect("continue target must keep authoritative world binding"),
            world_generation: resolved_target
                .session
                .world_generation
                .expect("continue target must keep authoritative world binding"),
            decision: payload.decision,
        })
    }

    #[cfg(any(target_os = "linux", test))]
    #[allow(dead_code)]
    fn close_prepared_internal_continue_approval_response_obligation(
        &self,
        closeout: &PreparedInternalApprovalResponseObligationCloseout,
        resolution_note: Option<String>,
    ) -> Result<OrchestrationObligationRecord> {
        self.with_legacy_snapshot_transaction(|transaction| {
            let mut obligation = self.load_exact_pending_obligation_transaction(
                transaction,
                &closeout.orchestration_session_id,
                &closeout.approval_obligation_id,
                OrchestrationObligationKind::ApprovalRequired,
                (
                    &closeout.target_participant_id,
                    &closeout.target_backend_id,
                    &closeout.world_id,
                    closeout.world_generation,
                ),
            )?;
            let disposition = match closeout.decision {
                ApprovalResponseDecisionV1::Approve => {
                    ApprovalObligationCloseoutDisposition::Resolve
                }
                ApprovalResponseDecisionV1::Deny => ApprovalObligationCloseoutDisposition::Dismiss,
            };
            let resolved_at = Utc::now();

            obligation.mark_approval_response_closed(disposition, resolution_note, resolved_at);
            self.persist_obligation_unlocked(transaction, &obligation)?;

            Ok(obligation)
        })
    }

    #[cfg(any(target_os = "linux", test))]
    #[allow(dead_code)]
    fn prepare_internal_continue_clarification_response_obligation_closeout(
        &self,
        resolved_target: &ResolvedInternalContinueWorldDispatchTarget,
        payload: &WorkerContinueClarificationResponsePayloadV1,
    ) -> Result<PreparedInternalClarificationResponseObligationCloseout> {
        let obligation = self.load_exact_pending_follow_up_obligation_for_continue_target(
            resolved_target.orchestration_session_id(),
            resolved_target.target_participant.participant_id(),
            &resolved_target.target_participant.handle.backend_id,
            resolved_target
                .session
                .world_id
                .as_deref()
                .expect("continue target must keep authoritative world binding"),
            resolved_target
                .session
                .world_generation
                .expect("continue target must keep authoritative world binding"),
            &payload.follow_up_obligation_id,
        )?;

        Ok(PreparedInternalClarificationResponseObligationCloseout {
            orchestration_session_id: obligation.orchestration_session_id,
            follow_up_obligation_id: obligation.obligation_id,
            target_participant_id: resolved_target
                .target_participant
                .participant_id()
                .to_string(),
            target_backend_id: resolved_target.target_participant.handle.backend_id.clone(),
            world_id: resolved_target
                .session
                .world_id
                .clone()
                .expect("continue target must keep authoritative world binding"),
            world_generation: resolved_target
                .session
                .world_generation
                .expect("continue target must keep authoritative world binding"),
        })
    }

    #[cfg(any(target_os = "linux", test))]
    #[allow(dead_code)]
    fn close_prepared_internal_continue_clarification_response_obligation(
        &self,
        closeout: &PreparedInternalClarificationResponseObligationCloseout,
        resolution_note: Option<String>,
    ) -> Result<OrchestrationObligationRecord> {
        self.with_legacy_snapshot_transaction(|transaction| {
            let mut obligation = self.load_exact_pending_obligation_transaction(
                transaction,
                &closeout.orchestration_session_id,
                &closeout.follow_up_obligation_id,
                OrchestrationObligationKind::FollowUpRequired,
                (
                    &closeout.target_participant_id,
                    &closeout.target_backend_id,
                    &closeout.world_id,
                    closeout.world_generation,
                ),
            )?;
            let resolved_at = Utc::now();

            obligation.mark_clarification_response_closed(resolution_note, resolved_at);
            self.persist_obligation_unlocked(transaction, &obligation)?;

            Ok(obligation)
        })
    }

    #[cfg(target_os = "linux")]
    pub(crate) fn prepare_internal_continue_approval_response_closeout_for_delivery(
        &self,
        orchestration_session_id: &str,
        caller_participant_id: &str,
        target_participant_id: &str,
        target_backend_id: &str,
        payload: &WorkerContinueApprovalResponsePayloadV1,
    ) -> Result<PreparedInternalApprovalResponseObligationCloseout> {
        let resolved_target = self.resolve_internal_continue_world_dispatch_target(
            orchestration_session_id,
            caller_participant_id,
            target_participant_id,
            target_backend_id,
        )?;
        self.prepare_internal_continue_approval_response_obligation_closeout(
            &resolved_target,
            payload,
        )
    }

    #[cfg(target_os = "linux")]
    pub(crate) fn close_internal_continue_approval_response_after_delivery(
        &self,
        closeout: &PreparedInternalApprovalResponseObligationCloseout,
        resolution_note: Option<String>,
    ) -> Result<OrchestrationObligationRecord> {
        self.close_prepared_internal_continue_approval_response_obligation(
            closeout,
            resolution_note,
        )
    }

    #[cfg(target_os = "linux")]
    #[allow(dead_code)]
    pub(crate) fn prepare_internal_continue_clarification_response_closeout_for_delivery(
        &self,
        orchestration_session_id: &str,
        caller_participant_id: &str,
        target_participant_id: &str,
        target_backend_id: &str,
        payload: &WorkerContinueClarificationResponsePayloadV1,
    ) -> Result<PreparedInternalClarificationResponseObligationCloseout> {
        let resolved_target = self.resolve_internal_continue_world_dispatch_target(
            orchestration_session_id,
            caller_participant_id,
            target_participant_id,
            target_backend_id,
        )?;
        self.prepare_internal_continue_clarification_response_obligation_closeout(
            &resolved_target,
            payload,
        )
    }

    #[cfg(target_os = "linux")]
    #[allow(dead_code)]
    pub(crate) fn close_internal_continue_clarification_response_after_delivery(
        &self,
        closeout: &PreparedInternalClarificationResponseObligationCloseout,
        resolution_note: Option<String>,
    ) -> Result<OrchestrationObligationRecord> {
        self.close_prepared_internal_continue_clarification_response_obligation(
            closeout,
            resolution_note,
        )
    }

    #[cfg(any(target_os = "linux", test))]
    #[allow(dead_code)]
    pub(crate) fn resolve_internal_fork_world_dispatch_target(
        &self,
        orchestration_session_id: &str,
        caller_participant_id: &str,
        target_participant_id: &str,
        target_backend_id: &str,
    ) -> Result<ResolvedInternalForkWorldDispatchTarget> {
        let Some(record) = self.load_session(orchestration_session_id)? else {
            anyhow::bail!(
                "missing_orchestration_session: internal world dispatch requires authoritative orchestration session {}",
                orchestration_session_id
            );
        };

        let authoritative =
            resolve_authoritative_session_control(&record, orchestration_session_id)?;
        if authoritative.participant.participant_id() != caller_participant_id {
            anyhow::bail!(
                "caller_not_authoritative: orchestration session {} authoritative orchestrator participant is {} not {}",
                orchestration_session_id,
                authoritative.participant.participant_id(),
                caller_participant_id
            );
        }

        let mut matching_participants = record
            .participants
            .iter()
            .filter(|participant| participant.participant_id() == target_participant_id)
            .cloned()
            .collect::<Vec<_>>();

        if matching_participants.is_empty() {
            anyhow::bail!(
                "target_not_in_session: orchestration session {} has no exact retained worker {}",
                orchestration_session_id,
                target_participant_id
            );
        }
        if matching_participants.len() > 1 {
            anyhow::bail!(
                "ambiguous_target_participant: orchestration session {} has multiple retained worker records for {}",
                orchestration_session_id,
                target_participant_id
            );
        }

        let source_participant = matching_participants
            .pop()
            .expect("target participant count checked above");
        if source_participant.handle.backend_id != target_backend_id {
            anyhow::bail!(
                "backend_mismatch: orchestration session {} retained worker {} backend is {} not {}",
                orchestration_session_id,
                target_participant_id,
                source_participant.handle.backend_id,
                target_backend_id
            );
        }
        if source_participant.handle.role != MEMBER_ROLE
            || source_participant.handle.execution.scope != AgentExecutionScope::World
        {
            anyhow::bail!(
                "invalid_target_participant: orchestration session {} participant {} is not a retained world worker",
                orchestration_session_id,
                target_participant_id
            );
        }
        validate_retained_worker_authoritative_lineage(
            &record,
            &authoritative.participant,
            &source_participant,
        )?;
        if !source_participant.matches_authoritative_parent_world_binding(&authoritative.session) {
            anyhow::bail!(
                "world_binding_mismatch: orchestration session {} retained worker {} no longer matches the authoritative world binding",
                orchestration_session_id,
                target_participant_id
            );
        }
        if !source_participant.handle.state.is_live()
            || source_participant.internal.terminal_observed_at.is_some()
        {
            anyhow::bail!(
                "target_already_terminal: orchestration session {} retained worker {} is already terminal ({})",
                orchestration_session_id,
                target_participant_id,
                source_participant.reviewable_terminal_state_label()
            );
        }

        Ok(ResolvedInternalForkWorldDispatchTarget {
            session: authoritative.session,
            caller_participant: authoritative.participant,
            source_participant,
        })
    }

    #[cfg(any(target_os = "linux", test))]
    #[allow(dead_code)]
    pub(crate) fn resolve_internal_inspect_world_dispatch_target(
        &self,
        orchestration_session_id: &str,
        caller_participant_id: &str,
        target_participant_id: &str,
        target_backend_id: &str,
    ) -> Result<ResolvedInternalInspectWorldDispatchTarget> {
        let Some(record) = self.load_session(orchestration_session_id)? else {
            anyhow::bail!(
                "missing_orchestration_session: internal world dispatch requires authoritative orchestration session {}",
                orchestration_session_id
            );
        };

        let authoritative =
            resolve_authoritative_session_control(&record, orchestration_session_id)?;
        if authoritative.participant.participant_id() != caller_participant_id {
            anyhow::bail!(
                "caller_not_authoritative: orchestration session {} authoritative orchestrator participant is {} not {}",
                orchestration_session_id,
                authoritative.participant.participant_id(),
                caller_participant_id
            );
        }

        let mut matching_participants = record
            .participants
            .iter()
            .filter(|participant| participant.participant_id() == target_participant_id)
            .cloned()
            .collect::<Vec<_>>();

        if matching_participants.is_empty() {
            anyhow::bail!(
                "target_not_in_session: orchestration session {} has no exact retained worker {}",
                orchestration_session_id,
                target_participant_id
            );
        }
        if matching_participants.len() > 1 {
            anyhow::bail!(
                "ambiguous_target_participant: orchestration session {} has multiple retained worker records for {}",
                orchestration_session_id,
                target_participant_id
            );
        }

        let target_participant = matching_participants
            .pop()
            .expect("target participant count checked above");
        if target_participant.handle.backend_id != target_backend_id {
            anyhow::bail!(
                "backend_mismatch: orchestration session {} retained worker {} backend is {} not {}",
                orchestration_session_id,
                target_participant_id,
                target_participant.handle.backend_id,
                target_backend_id
            );
        }
        if target_participant.handle.role != MEMBER_ROLE
            || target_participant.handle.execution.scope != AgentExecutionScope::World
        {
            anyhow::bail!(
                "invalid_target_participant: orchestration session {} participant {} is not a retained world worker",
                orchestration_session_id,
                target_participant_id
            );
        }
        validate_retained_worker_authoritative_lineage(
            &record,
            &authoritative.participant,
            &target_participant,
        )?;
        if !target_participant.matches_authoritative_parent_world_binding(&authoritative.session) {
            anyhow::bail!(
                "world_binding_mismatch: orchestration session {} retained worker {} no longer matches the authoritative world binding",
                orchestration_session_id,
                target_participant_id
            );
        }

        Ok(ResolvedInternalInspectWorldDispatchTarget {
            session: authoritative.session,
            caller_participant: authoritative.participant,
            target_participant,
        })
    }

    #[cfg(any(target_os = "linux", test))]
    #[allow(dead_code)]
    pub(crate) fn resolve_internal_stop_world_dispatch_target(
        &self,
        orchestration_session_id: &str,
        caller_participant_id: &str,
        target_participant_id: &str,
        target_backend_id: &str,
    ) -> Result<ResolvedInternalStopWorldDispatchTarget> {
        let Some(record) = self.load_session(orchestration_session_id)? else {
            anyhow::bail!(
                "missing_orchestration_session: internal world dispatch requires authoritative orchestration session {}",
                orchestration_session_id
            );
        };

        let authoritative_participant = resolve_authoritative_stop_dispatch_owner_participant(
            &record,
            orchestration_session_id,
        )?;
        if authoritative_participant.participant_id() != caller_participant_id {
            anyhow::bail!(
                "caller_not_authoritative: orchestration session {} authoritative orchestrator participant is {} not {}",
                orchestration_session_id,
                authoritative_participant.participant_id(),
                caller_participant_id
            );
        }

        let mut matching_participants = record
            .participants
            .iter()
            .filter(|participant| participant.participant_id() == target_participant_id)
            .cloned()
            .collect::<Vec<_>>();

        if matching_participants.is_empty() {
            anyhow::bail!(
                "target_not_in_session: orchestration session {} has no exact retained worker {}",
                orchestration_session_id,
                target_participant_id
            );
        }
        if matching_participants.len() > 1 {
            anyhow::bail!(
                "ambiguous_target_participant: orchestration session {} has multiple retained worker records for {}",
                orchestration_session_id,
                target_participant_id
            );
        }

        let target_participant = matching_participants
            .pop()
            .expect("target participant count checked above");
        if target_participant.handle.backend_id != target_backend_id {
            anyhow::bail!(
                "backend_mismatch: orchestration session {} retained worker {} backend is {} not {}",
                orchestration_session_id,
                target_participant_id,
                target_participant.handle.backend_id,
                target_backend_id
            );
        }
        if target_participant.handle.role != MEMBER_ROLE
            || target_participant.handle.execution.scope != AgentExecutionScope::World
        {
            anyhow::bail!(
                "invalid_target_participant: orchestration session {} participant {} is not a retained world worker",
                orchestration_session_id,
                target_participant_id
            );
        }
        validate_retained_worker_authoritative_lineage(
            &record,
            &authoritative_participant,
            &target_participant,
        )?;
        if !target_participant.matches_authoritative_parent_world_binding(&record.session) {
            anyhow::bail!(
                "world_binding_mismatch: orchestration session {} retained worker {} no longer matches the authoritative world binding",
                orchestration_session_id,
                target_participant_id
            );
        }
        if !target_participant.handle.state.is_live()
            || target_participant.internal.terminal_observed_at.is_some()
        {
            let terminal_state = match target_participant.handle.state {
                super::session::AgentRuntimeSessionState::Stopped => "stopped",
                super::session::AgentRuntimeSessionState::Failed => "failed",
                super::session::AgentRuntimeSessionState::Invalidated => "invalidated",
                super::session::AgentRuntimeSessionState::Allocating
                | super::session::AgentRuntimeSessionState::Ready
                | super::session::AgentRuntimeSessionState::Running
                | super::session::AgentRuntimeSessionState::Restarting
                | super::session::AgentRuntimeSessionState::Stopping => "terminal",
            };
            anyhow::bail!(
                "target_already_terminal: orchestration session {} retained worker {} is already terminal ({})",
                orchestration_session_id,
                target_participant_id,
                terminal_state
            );
        }

        let exact_target = ExactRetainedWorkerStopDispatchTarget::capture(
            orchestration_session_id,
            &target_participant,
        )?;

        Ok(ResolvedInternalStopWorldDispatchTarget {
            session: record.session.clone(),
            caller_participant: authoritative_participant,
            target_participant,
            exact_target,
        })
    }

    #[cfg(any(target_os = "linux", test))]
    #[allow(dead_code)]
    pub(crate) fn resolve_internal_cancel_world_dispatch_target(
        &self,
        orchestration_session_id: &str,
        caller_participant_id: &str,
        target_participant_id: &str,
        target_backend_id: &str,
    ) -> Result<ResolvedInternalCancelWorldDispatchTarget> {
        let Some(record) = self.load_session(orchestration_session_id)? else {
            anyhow::bail!(
                "missing_orchestration_session: internal world dispatch requires authoritative orchestration session {}",
                orchestration_session_id
            );
        };

        let authoritative = if record.session.state == OrchestrationSessionState::Active {
            resolve_authoritative_session_control(&record, orchestration_session_id)?
        } else if record.session.has_cancelled_terminal_truth() {
            let participant =
                resolve_authoritative_session_participant(&record, orchestration_session_id)?;
            ResolvedAuthoritativeSessionControl {
                session: record.session.clone(),
                participant: participant.clone(),
                session_posture: classify_public_session_posture(&record, &participant),
            }
        } else {
            resolve_authoritative_session_control(&record, orchestration_session_id)?
        };
        if authoritative.participant.participant_id() != caller_participant_id {
            anyhow::bail!(
                "caller_not_authoritative: orchestration session {} authoritative orchestrator participant is {} not {}",
                orchestration_session_id,
                authoritative.participant.participant_id(),
                caller_participant_id
            );
        }

        let mut matching_participants = record
            .participants
            .iter()
            .filter(|participant| participant.participant_id() == target_participant_id)
            .cloned()
            .collect::<Vec<_>>();

        if matching_participants.is_empty() {
            anyhow::bail!(
                "target_not_in_session: orchestration session {} has no exact retained worker {}",
                orchestration_session_id,
                target_participant_id
            );
        }
        if matching_participants.len() > 1 {
            anyhow::bail!(
                "ambiguous_target_participant: orchestration session {} has multiple retained worker records for {}",
                orchestration_session_id,
                target_participant_id
            );
        }

        let target_participant = matching_participants
            .pop()
            .expect("target participant count checked above");
        if target_participant.handle.backend_id != target_backend_id {
            anyhow::bail!(
                "backend_mismatch: orchestration session {} retained worker {} backend is {} not {}",
                orchestration_session_id,
                target_participant_id,
                target_participant.handle.backend_id,
                target_backend_id
            );
        }
        if target_participant.handle.role != MEMBER_ROLE
            || target_participant.handle.execution.scope != AgentExecutionScope::World
        {
            anyhow::bail!(
                "invalid_target_participant: orchestration session {} participant {} is not a retained world worker",
                orchestration_session_id,
                target_participant_id
            );
        }
        validate_retained_worker_authoritative_lineage(
            &record,
            &authoritative.participant,
            &target_participant,
        )?;
        if !target_participant.matches_authoritative_parent_world_binding(&authoritative.session) {
            anyhow::bail!(
                "world_binding_mismatch: orchestration session {} retained worker {} no longer matches the authoritative world binding",
                orchestration_session_id,
                target_participant_id
            );
        }
        if !target_participant.handle.state.is_live()
            || target_participant.internal.terminal_observed_at.is_some()
        {
            anyhow::bail!(
                "target_already_terminal: orchestration session {} retained worker {} is already terminal ({})",
                orchestration_session_id,
                target_participant_id,
                target_participant.reviewable_terminal_state_label()
            );
        }
        if record.session.state != OrchestrationSessionState::Active {
            anyhow::bail!(
                "missing_active_parent: orchestration session {} is not active",
                orchestration_session_id
            );
        }
        if !target_participant.is_authoritative_live()
            || !owner_process_is_alive(&target_participant)
        {
            anyhow::bail!(
                "stale_linkage: orchestration session {} retained worker {} is no longer authoritative-live",
                orchestration_session_id,
                target_participant_id
            );
        }
        if !target_participant.internal.cancel_supported {
            anyhow::bail!(
                "target_not_cancelable: orchestration session {} retained worker {} does not advertise cancel support",
                orchestration_session_id,
                target_participant_id
            );
        }
        let Some(active_run_id) = target_participant.internal.latest_run_id.clone() else {
            anyhow::bail!(
                "target_not_cancelable: orchestration session {} retained worker {} has no active cancelable work in flight",
                orchestration_session_id,
                target_participant_id
            );
        };
        if target_participant.handle.state != super::session::AgentRuntimeSessionState::Running {
            anyhow::bail!(
                "target_not_cancelable: orchestration session {} retained worker {} has no active cancelable work in flight",
                orchestration_session_id,
                target_participant_id
            );
        }

        Ok(ResolvedInternalCancelWorldDispatchTarget {
            session: authoritative.session,
            caller_participant: authoritative.participant,
            target_participant,
            active_run_id,
        })
    }

    pub(crate) fn count_authoritative_live_retained_workers(
        &self,
        orchestration_session_id: &str,
        authoritative_orchestrator_participant_id: &str,
    ) -> Result<usize> {
        let Some(record) = self.load_session(orchestration_session_id)? else {
            anyhow::bail!(
                "missing_orchestration_session: internal world dispatch requires authoritative orchestration session {}",
                orchestration_session_id
            );
        };

        Ok(record
            .participants
            .iter()
            .filter(|participant| {
                participant.handle.role == MEMBER_ROLE
                    && participant.handle.execution.scope == AgentExecutionScope::World
                    && participant.handle.orchestrator_participant_id.as_deref()
                        == Some(authoritative_orchestrator_participant_id)
                    && participant.matches_authoritative_parent_world_binding(&record.session)
                    && participant.is_authoritative_live()
                    && owner_process_is_alive(participant)
            })
            .count())
    }

    // Public turn routing stays exact:
    // (orchestration_session_id, backend_id) selects one authoritative retained slot,
    // or it fails closed without falling back to fuzzy inventory guesses.
    #[allow(dead_code)]
    pub(crate) fn resolve_public_turn_target(
        &self,
        orchestration_session_id: &str,
        backend_id: &str,
    ) -> Result<ResolvedPublicTurnTarget> {
        if backend_id.trim().is_empty() {
            anyhow::bail!("missing_backend: public turn actions require --backend <backend_id>");
        }
        if let Some(guidance) = retired_public_turn_backend_guidance(backend_id) {
            anyhow::bail!("{guidance}");
        }

        let Some(record) = self.load_session(orchestration_session_id)? else {
            return Err(self.public_turn_session_selector_error(orchestration_session_id));
        };

        if !record.has_authoritative_parent() {
            anyhow::bail!(
                "missing_active_parent: orchestration session {} is missing authoritative parent metadata",
                orchestration_session_id
            );
        }
        if record.session.state != OrchestrationSessionState::Active {
            anyhow::bail!(
                "missing_active_parent: orchestration session {} is not active",
                orchestration_session_id
            );
        }
        if record.session.posture == OrchestrationSessionPosture::BornUnattached {
            anyhow::bail!(
                "unsupported_platform_or_posture: orchestration session {} backend {} is born_unattached and cannot accept follow-up turns before sanctioned host attach",
                orchestration_session_id,
                backend_id
            );
        }

        let authoritative =
            resolve_authoritative_session_control(&record, orchestration_session_id)?;

        let slot_present = public_turn_session_mentions_backend(&record, backend_id);
        let mut candidates =
            public_turn_authoritative_candidates(&record, &authoritative.participant, backend_id);
        if candidates.is_empty() {
            if slot_present {
                anyhow::bail!(
                    "stale_linkage: orchestration session {} backend {} no longer has an authoritative retained turn target",
                    orchestration_session_id,
                    backend_id
                );
            }
            anyhow::bail!(
                "backend_not_in_session: orchestration session {} has no exact backend slot for {}",
                orchestration_session_id,
                backend_id
            );
        }
        if candidates.len() > 1 {
            let participant_ids = candidates
                .iter()
                .map(|candidate| candidate.participant.handle.participant_id.as_str())
                .collect::<Vec<_>>()
                .join(", ");
            anyhow::bail!(
                "ambiguous_backend_slot: orchestration session {} has multiple authoritative retained turn targets for backend {} ({participant_ids})",
                orchestration_session_id,
                backend_id
            );
        }

        let candidate = candidates.pop().expect("candidate count checked above");
        if candidate.kind == PublicTurnTargetKind::World && !cfg!(unix) {
            anyhow::bail!(
                "unsupported_platform_or_posture: orchestration session {} backend {} requires Unix world-sensitive follow-up posture",
                orchestration_session_id,
                backend_id
            );
        }

        let host_attach_contract = authoritative.session.host_attach_contract().cloned();

        Ok(ResolvedPublicTurnTarget {
            session: authoritative.session,
            participant: candidate.participant,
            target_kind: candidate.kind,
            session_posture: authoritative.session_posture,
            host_attach_contract,
        })
    }

    pub(crate) fn classify_hidden_owner_helper_launch_readiness(
        &self,
        orchestration_session_id: &str,
        participant_id: &str,
        require_internal_session_id: bool,
    ) -> Result<HiddenOwnerHelperLaunchReadiness> {
        Ok(
            match self.classify_hidden_owner_helper_launch_continuity(
                orchestration_session_id,
                participant_id,
                require_internal_session_id,
            )? {
                HiddenOwnerHelperLaunchContinuity::AttachedLive => {
                    HiddenOwnerHelperLaunchReadiness::ReadyAttached
                }
                HiddenOwnerHelperLaunchContinuity::DetachedReconciled(posture) => {
                    HiddenOwnerHelperLaunchReadiness::ReadyDetached(posture)
                }
                HiddenOwnerHelperLaunchContinuity::Pending
                | HiddenOwnerHelperLaunchContinuity::StaleAttachedTruth => {
                    HiddenOwnerHelperLaunchReadiness::Pending
                }
            },
        )
    }

    pub(crate) fn classify_hidden_owner_helper_launch_continuity(
        &self,
        orchestration_session_id: &str,
        participant_id: &str,
        require_internal_session_id: bool,
    ) -> Result<HiddenOwnerHelperLaunchContinuity> {
        let Some(record) = self.load_session(orchestration_session_id)? else {
            return Ok(HiddenOwnerHelperLaunchContinuity::Pending);
        };
        if record.session.state != OrchestrationSessionState::Active {
            return Ok(HiddenOwnerHelperLaunchContinuity::Pending);
        }
        if record.session.active_participant_id() != Some(participant_id) {
            return Ok(HiddenOwnerHelperLaunchContinuity::Pending);
        }

        let Some(participant) = record
            .participants
            .iter()
            .find(|participant| participant.participant_id() == participant_id)
        else {
            return Ok(HiddenOwnerHelperLaunchContinuity::Pending);
        };
        if !participant.matches_public_parent_linkage(&record.session) {
            return Ok(HiddenOwnerHelperLaunchContinuity::Pending);
        }
        if require_internal_session_id && participant.internal_uaa_session_id().is_none() {
            return Ok(HiddenOwnerHelperLaunchContinuity::Pending);
        }

        let attached_live = session_attached_to_participant(&record.session, participant)
            && participant.attached_client_present()
            && participant.is_authoritative_live()
            && owner_process_is_alive(participant);
        if attached_live {
            return Ok(HiddenOwnerHelperLaunchContinuity::AttachedLive);
        }

        if let Some(posture) = valid_detached_host_continuity_posture(
            &record.session,
            participant,
            require_internal_session_id,
        ) {
            return Ok(HiddenOwnerHelperLaunchContinuity::DetachedReconciled(
                posture,
            ));
        }

        if recoverable_stale_host_attachment(
            &record,
            &record.session,
            participant,
            require_internal_session_id,
        ) {
            return Ok(HiddenOwnerHelperLaunchContinuity::StaleAttachedTruth);
        }

        Ok(HiddenOwnerHelperLaunchContinuity::Pending)
    }

    #[cfg(unix)]
    pub(crate) fn resumed_public_turn_detach_posture(
        &self,
        orchestration_session_id: &str,
        participant_id: &str,
    ) -> Result<Option<OrchestrationSessionPosture>> {
        let Some(record) = self.load_session(orchestration_session_id)? else {
            return Ok(None);
        };
        if record.session.state != OrchestrationSessionState::Active {
            return Ok(None);
        }
        if record.session.active_participant_id() != Some(participant_id) {
            return Ok(None);
        }

        let Some(participant) = record
            .participants
            .iter()
            .find(|participant| participant.participant_id() == participant_id)
        else {
            return Ok(None);
        };

        let Some(posture) =
            valid_detached_host_continuity_posture(&record.session, participant, true)
        else {
            return Ok(None);
        };

        if record.participants.iter().any(|candidate| {
            candidate.participant_id() != participant_id
                && candidate.matches_public_parent_linkage(&record.session)
                && candidate.is_host_orchestrator()
                && candidate.attached_client_present()
                && candidate.is_authoritative_live()
                && owner_process_is_alive(candidate)
        }) {
            return Ok(None);
        }

        Ok(Some(posture))
    }

    pub(crate) fn startup_prompt_replay_state(
        &self,
        orchestration_session_id: &str,
        participant_id: &str,
    ) -> Result<StartupPromptReplayState> {
        let Some(session) = self.load_orchestration_session(orchestration_session_id)? else {
            return Ok(StartupPromptReplayState::NotTracked);
        };
        let Some(startup_prompt) = session.startup_prompt.as_ref() else {
            return Ok(StartupPromptReplayState::NotTracked);
        };
        if startup_prompt.participant_id != participant_id {
            return Ok(StartupPromptReplayState::AcceptedOrTerminal);
        }
        Ok(match startup_prompt.state {
            StartupPromptStreamState::PendingAcceptance => {
                StartupPromptReplayState::PendingAcceptance
            }
            StartupPromptStreamState::Accepted
            | StartupPromptStreamState::Completed
            | StartupPromptStreamState::Failed => StartupPromptReplayState::AcceptedOrTerminal,
        })
    }

    pub(crate) fn resolve_live_orchestrator_participant(
        &self,
        orchestrator_agent_id: &str,
    ) -> Result<Option<(OrchestrationSessionRecord, AgentRuntimeParticipantRecord)>> {
        Ok(self
            .resolve_single_live_session_for_agent(orchestrator_agent_id)?
            .and_then(|record| {
                record
                    .live_orchestrator()
                    .map(|participant| (record.session, participant))
            }))
    }

    pub(crate) fn validate_participant_record(
        &self,
        participant: &AgentRuntimeParticipantRecord,
    ) -> Result<()> {
        participant.validate()
    }

    pub(crate) fn validate_session_record(
        &self,
        session: &OrchestrationSessionRecord,
    ) -> Result<()> {
        session.validate_persisted_invariants()
    }

    fn validate_inbox_item_record(&self, item: &DurableInboxItemRecord) -> Result<()> {
        item.validate()
    }

    fn validate_obligation_record(&self, obligation: &OrchestrationObligationRecord) -> Result<()> {
        obligation.validate()
    }

    #[allow(dead_code)]
    fn validate_host_inbox_record(&self, record: &HostInboxRecord) -> Result<()> {
        record.validate()
    }

    pub(crate) fn persist_orchestration_session(
        &self,
        session: &OrchestrationSessionRecord,
    ) -> Result<()> {
        self.with_legacy_snapshot_transaction(|transaction| {
            self.validate_session_record(session)?;
            let existing = self.load_authoritative_session_transaction(
                transaction,
                &session.orchestration_session_id,
            )?;
            if let Some(existing_session) = existing.as_ref() {
                if !should_persist_orchestration_session_snapshot(existing_session, session) {
                    return Ok(());
                }
            }
            let should_log_parked_write =
                stop_order_probe_should_log_parked_write(existing.as_ref(), session);
            self.persist_parent_session_snapshot(transaction, session)?;
            if should_log_parked_write {
                stop_order_probe_log_parked_write(existing.as_ref(), session);
            }
            Ok(())
        })
    }

    #[allow(dead_code)]
    pub(crate) fn persist_inbox_item(&self, item: &DurableInboxItemRecord) -> Result<()> {
        self.with_legacy_snapshot_transaction(|transaction| {
            self.persist_inbox_item_unlocked(transaction, item)
        })
    }

    fn persist_inbox_item_unlocked(
        &self,
        transaction: &mut super::host_session_authority::store::LegacyWriterGuard,
        item: &DurableInboxItemRecord,
    ) -> Result<()> {
        use super::host_session_authority::store::LegacyStateStoreCollectionV1::Sessions;
        self.validate_inbox_item_record(item)?;
        let mut session = self
            .load_authoritative_session_transaction(transaction, &item.orchestration_session_id)?
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "missing authoritative parent session {} for durable inbox item {}",
                    item.orchestration_session_id,
                    item.item_id
                )
            })?;
        let existing = self.load_inbox_item_transaction(
            transaction,
            &item.orchestration_session_id,
            &item.item_id,
        )?;
        let next_pending_count = updated_pending_inbox_count(
            session.pending_inbox_count,
            existing
                .as_ref()
                .is_some_and(DurableInboxItemRecord::is_pending),
            item.is_pending(),
        )?;
        apply_pending_inbox_count(&mut session, next_pending_count);

        let file_name = format!("{}.json", item.item_id);
        let descendants = [
            item.orchestration_session_id.as_str(),
            "inbox",
            file_name.as_str(),
        ];
        Self::transaction_write_json(transaction, Sessions, &descendants, item)?;
        if let Err(err) = self.persist_parent_session_snapshot(transaction, &session) {
            match existing.as_ref() {
                Some(previous) => {
                    Self::transaction_write_json(transaction, Sessions, &descendants, previous)?
                }
                None => {
                    Self::transaction_remove_file(transaction, Sessions, &descendants)?;
                }
            }
            return Err(err);
        }
        Ok(())
    }

    #[allow(dead_code)]
    pub(crate) fn persist_obligation(
        &self,
        obligation: &OrchestrationObligationRecord,
    ) -> Result<()> {
        self.with_legacy_snapshot_transaction(|transaction| {
            self.persist_obligation_unlocked(transaction, obligation)
        })
    }

    fn persist_obligation_unlocked(
        &self,
        transaction: &mut super::host_session_authority::store::LegacyWriterGuard,
        obligation: &OrchestrationObligationRecord,
    ) -> Result<()> {
        use super::host_session_authority::store::LegacyStateStoreCollectionV1::Sessions;
        self.validate_obligation_record(obligation)?;
        let mut session = self
            .load_authoritative_session_transaction(
                transaction,
                &obligation.orchestration_session_id,
            )?
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "missing authoritative parent session {} for orchestration obligation {}",
                    obligation.orchestration_session_id,
                    obligation.obligation_id
                )
            })?;
        let existing = self.load_obligation_transaction(
            transaction,
            &obligation.orchestration_session_id,
            &obligation.obligation_id,
        )?;
        let existing_compat_item = self.load_inbox_item_transaction(
            transaction,
            &obligation.orchestration_session_id,
            &obligation.obligation_id,
        )?;
        let mut obligations =
            self.list_obligations_transaction(transaction, &obligation.orchestration_session_id)?;
        obligations.retain(|current| current.obligation_id != obligation.obligation_id);
        obligations.push(obligation.clone());

        let projected_pending_count = projected_pending_inbox_count_from_obligations(&obligations)?;
        apply_pending_inbox_count(&mut session, projected_pending_count);
        let projected_compat_item =
            compatibility_inbox_item_from_obligation(obligation, existing_compat_item.as_ref());
        if let Some(item) = projected_compat_item.as_ref() {
            self.validate_inbox_item_record(item)?;
        }

        let file_name = format!("{}.json", obligation.obligation_id);
        let obligation_descendants = [
            obligation.orchestration_session_id.as_str(),
            "obligations",
            file_name.as_str(),
        ];
        let inbox_descendants = [
            obligation.orchestration_session_id.as_str(),
            "inbox",
            file_name.as_str(),
        ];
        Self::transaction_write_json(transaction, Sessions, &obligation_descendants, obligation)?;
        if let Some(item) = projected_compat_item.as_ref() {
            Self::transaction_write_json(transaction, Sessions, &inbox_descendants, item)?;
        }
        if let Err(err) = self.persist_parent_session_snapshot(transaction, &session) {
            match existing.as_ref() {
                Some(previous) => Self::transaction_write_json(
                    transaction,
                    Sessions,
                    &obligation_descendants,
                    previous,
                )?,
                None => {
                    Self::transaction_remove_file(transaction, Sessions, &obligation_descendants)?;
                }
            }
            if projected_compat_item.is_some() {
                match existing_compat_item.as_ref() {
                    Some(previous) => Self::transaction_write_json(
                        transaction,
                        Sessions,
                        &inbox_descendants,
                        previous,
                    )?,
                    None => {
                        Self::transaction_remove_file(transaction, Sessions, &inbox_descendants)?;
                    }
                }
            }
            return Err(err);
        }

        Ok(())
    }

    #[allow(dead_code)]
    pub(crate) fn persist_host_inbox_record(&self, record: &HostInboxRecord) -> Result<()> {
        let _write_guard = snapshot_write_lock()
            .lock()
            .expect("snapshot write mutex poisoned");
        self.persist_host_inbox_record_unlocked(record)
    }

    fn persist_host_inbox_record_unlocked(&self, record: &HostInboxRecord) -> Result<()> {
        self.validate_host_inbox_record(record)?;
        write_atomic_json(&self.host_inbox_record_path(&record.record_id)?, record)
    }

    fn host_inbox_record_file_name(record_id: &str) -> Result<String> {
        HostInboxRecord::validate_record_id(record_id)?;
        Ok(format!("{record_id}.json"))
    }

    fn load_host_inbox_record_transaction(
        transaction: &mut super::host_session_authority::store::LegacyWriterGuard,
        record_id: &str,
        artifact_label: &str,
    ) -> Result<Option<HostInboxRecord>> {
        use super::host_session_authority::store::LegacyStateStoreCollectionV1::HostInbox;
        let file_name = Self::host_inbox_record_file_name(record_id)?;
        transaction
            .read_file(HostInbox, &[file_name.as_str()])
            .context("read retained host inbox record")?
            .map(|bytes| {
                serde_json::from_slice(&bytes)
                    .with_context(|| format!("failed to parse {artifact_label}"))
            })
            .transpose()
    }

    fn persist_host_inbox_record_transaction(
        &self,
        transaction: &mut super::host_session_authority::store::LegacyWriterGuard,
        record: &HostInboxRecord,
    ) -> Result<()> {
        use super::host_session_authority::store::LegacyStateStoreCollectionV1::HostInbox;
        self.validate_host_inbox_record(record)?;
        let file_name = Self::host_inbox_record_file_name(&record.record_id)?;
        Self::transaction_write_json(transaction, HostInbox, &[file_name.as_str()], record)
    }

    fn mark_host_inbox_record_failed_closed_transaction(
        &self,
        transaction: &mut super::host_session_authority::store::LegacyWriterGuard,
        record: &mut HostInboxRecord,
        reason: impl Into<String>,
        failed_closed_at: chrono::DateTime<Utc>,
    ) -> Result<HostInboxRecord> {
        record.mark_failed_closed(reason, failed_closed_at);
        self.persist_host_inbox_record_transaction(transaction, record)?;
        Ok(record.clone())
    }

    fn synthesize_failed_closed_host_inbox_record_transaction(
        &self,
        transaction: &mut super::host_session_authority::store::LegacyWriterGuard,
        record_id: &str,
        reason: impl Into<String>,
        failed_closed_at: chrono::DateTime<Utc>,
    ) -> Result<HostInboxRecord> {
        let mut record = HostInboxRecord::new(
            String::new(),
            record_id.to_string(),
            OrchestrationObligationKind::RuntimeAlert,
            "malformed host inbox artifact",
            String::new(),
            String::new(),
            String::new(),
        );
        record.severity = OrchestrationObligationSeverity::Error;
        record.created_at = failed_closed_at;
        record.ingress_received_at = failed_closed_at;
        record.mark_failed_closed(reason, failed_closed_at);
        self.persist_host_inbox_record_transaction(transaction, &record)?;
        Ok(record)
    }

    fn load_host_inbox_record_artifact(&self, path: &Path) -> Result<Option<HostInboxRecord>> {
        read_regular_json_if_exists::<HostInboxRecord>(path)
    }

    #[cfg(any(target_os = "linux", test))]
    fn synthesize_failed_closed_invalid_host_inbox_artifact_record(
        &self,
        record_id: &str,
        reason: impl Into<String>,
        failed_closed_at: chrono::DateTime<Utc>,
    ) -> Result<HostInboxRecord> {
        let mut record = HostInboxRecord::new(
            String::new(),
            record_id.to_string(),
            OrchestrationObligationKind::RuntimeAlert,
            "malformed host inbox artifact",
            String::new(),
            String::new(),
            String::new(),
        );
        record.severity = OrchestrationObligationSeverity::Error;
        record.created_at = failed_closed_at;
        record.ingress_received_at = failed_closed_at;
        record.mark_failed_closed(reason, failed_closed_at);
        write_atomic_json(
            &self.invalid_host_inbox_artifact_record_path(&record.record_id)?,
            &record,
        )?;
        Ok(record)
    }

    #[cfg(any(target_os = "linux", test))]
    fn invalid_host_inbox_artifact_record_id(path: &Path) -> String {
        let artifact_name = path
            .file_name()
            .map(|value| value.to_string_lossy())
            .unwrap_or_else(|| path.to_string_lossy());
        let mut hasher = Sha256::new();
        hasher.update(artifact_name.as_bytes());
        let digest = hasher.finalize();
        let mut suffix = String::with_capacity(16);
        for byte in &digest[..8] {
            suffix.push_str(&format!("{byte:02x}"));
        }
        format!("invalid_host_inbox_artifact_{suffix}")
    }

    #[cfg(any(target_os = "linux", test))]
    pub(crate) fn record_invalid_host_inbox_artifact_failure(
        &self,
        path: &Path,
    ) -> Result<HostInboxRecord> {
        let record_id = Self::invalid_host_inbox_artifact_record_id(path);
        let failure_path = self.invalid_host_inbox_artifact_record_path(&record_id)?;
        let reason = match Self::host_inbox_record_id_from_path(path) {
            Ok(_) => format!("invalid_host_inbox_artifact_path: {}", path.display()),
            Err(err) => format!("invalid_host_inbox_artifact_path: {err:#}"),
        };

        let Some(existing) = read_regular_json_if_exists::<HostInboxRecord>(&failure_path)? else {
            return self.synthesize_failed_closed_invalid_host_inbox_artifact_record(
                &record_id,
                reason,
                Utc::now(),
            );
        };
        self.validate_host_inbox_record(&existing)
            .with_context(|| format!("invalid host inbox record in {}", failure_path.display()))?;
        if existing.materialization_state == HostInboxMaterializationState::FailedClosed
            && existing.failed_closed_reason.as_deref() == Some(reason.as_str())
        {
            return Ok(existing);
        }
        self.synthesize_failed_closed_invalid_host_inbox_artifact_record(
            &record_id,
            reason,
            Utc::now(),
        )
    }

    #[cfg(test)]
    pub(crate) fn load_invalid_host_inbox_artifact_failure_record(
        &self,
        source_path: &Path,
    ) -> Result<Option<HostInboxRecord>> {
        let record_id = Self::invalid_host_inbox_artifact_record_id(source_path);
        let path = self.invalid_host_inbox_artifact_record_path(&record_id)?;
        let Some(record) = read_regular_json_if_exists::<HostInboxRecord>(&path)? else {
            return Ok(None);
        };
        self.validate_host_inbox_record(&record)
            .with_context(|| format!("invalid host inbox record in {}", path.display()))?;
        Ok(Some(record))
    }

    fn validate_host_inbox_record_artifact_identity(
        record: &HostInboxRecord,
        expected_record_id: &str,
        artifact_label: &str,
    ) -> Result<()> {
        if record.record_id.is_empty() {
            anyhow::bail!(
                "host inbox artifact {} is missing required record_id",
                artifact_label
            );
        }
        if record.record_id != expected_record_id {
            anyhow::bail!(
                "host inbox artifact {} stored mismatched record_id {}",
                artifact_label,
                record.record_id
            );
        }

        Ok(())
    }

    #[allow(dead_code)]
    pub(crate) fn materialize_host_inbox_record_for_local_host(
        &self,
        record_id: &str,
        local_host_id: &str,
    ) -> Result<HostInboxRecord> {
        let artifact_label = self
            .host_inbox_dir()
            .join(format!("{record_id}.json"))
            .display()
            .to_string();
        self.with_legacy_snapshot_transaction(|transaction| {
        if local_host_id.trim().is_empty() {
            anyhow::bail!("host inbox materialization requires non-empty local_host_id");
        }

        let Some(mut record) = (match Self::load_host_inbox_record_transaction(
            transaction,
            record_id,
            &artifact_label,
        ) {
            Ok(record) => record,
            Err(err)
                if err
                    .chain()
                    .any(|cause| cause.downcast_ref::<serde_json::Error>().is_some()) =>
            {
                return self.synthesize_failed_closed_host_inbox_record_transaction(
                    transaction,
                    record_id,
                    format!("malformed_host_inbox_artifact: {err}"),
                    Utc::now(),
                );
            }
            Err(err) => return Err(err),
        }) else {
            anyhow::bail!("host_inbox_record_not_found: no host inbox record {record_id}");
        };
        if let Err(err) =
            Self::validate_host_inbox_record_artifact_identity(&record, record_id, &artifact_label)
        {
            record.record_id = record_id.to_string();
            return self.mark_host_inbox_record_failed_closed_transaction(
                transaction,
                &mut record,
                err.to_string(),
                Utc::now(),
            );
        }

        match record.materialization_state {
            HostInboxMaterializationState::Materialized => {
                if let Err(err) = self.validate_host_inbox_record(&record) {
                    return self.mark_host_inbox_record_failed_closed_transaction(
                        transaction,
                        &mut record,
                        err.to_string(),
                        Utc::now(),
                    );
                }
                record.ensure_targets_local_host(local_host_id)?;
                let record_id = record.record_id.clone();
                let Some(obligation_id) = record.materialized_obligation_id.as_deref() else {
                    return self.mark_host_inbox_record_failed_closed_transaction(
                        transaction,
                        &mut record,
                        format!(
                            "materialized_host_inbox_record_missing_obligation: host inbox record {} is missing obligation linkage",
                            record_id
                        ),
                        Utc::now(),
                    );
                };
                let obligation_id = obligation_id.to_string();
                let Some(obligation) =
                    self.load_obligation_transaction(
                        transaction,
                        &record.orchestration_session_id,
                        &obligation_id,
                    )?
                else {
                    return self.mark_host_inbox_record_failed_closed_transaction(
                        transaction,
                        &mut record,
                        format!(
                            "materialized_host_inbox_record_missing_obligation: host inbox record {} references missing obligation {}",
                            record_id, obligation_id
                        ),
                        Utc::now(),
                    );
                };
                if !record.matches_materialized_obligation(&obligation) {
                    return self.mark_host_inbox_record_failed_closed_transaction(
                        transaction,
                        &mut record,
                        format!(
                            "materialized_host_inbox_record_mismatch: host inbox record {} no longer matches obligation {}",
                            record_id, obligation_id
                        ),
                        Utc::now(),
                    );
                }
                return Ok(record);
            }
            HostInboxMaterializationState::FailedClosed => {
                if let Err(err) = self.validate_host_inbox_record(&record) {
                    return self.mark_host_inbox_record_failed_closed_transaction(
                        transaction,
                        &mut record,
                        err.to_string(),
                        Utc::now(),
                    );
                }
                return Ok(record);
            }
            HostInboxMaterializationState::Pending => {}
        }

        if let Err(err) = record
            .ensure_pending_materialization_candidate()
            .and_then(|_| record.ensure_targets_local_host(local_host_id))
        {
            let failed_closed_at = Utc::now();
            record.mark_failed_closed(err.to_string(), failed_closed_at);
            self.persist_host_inbox_record_transaction(transaction, &record)?;
            return Ok(record);
        }

        if self
            .load_authoritative_session_transaction(
                transaction,
                &record.orchestration_session_id,
            )?
            .is_none()
        {
            return Ok(record);
        }

        let obligation_id = record.local_obligation_id();
        let materialized_at = Utc::now();
        let obligation = match self.load_obligation_transaction(
            transaction,
            &record.orchestration_session_id,
            &obligation_id,
        )?
        {
            Some(existing) => {
                if !record.matches_materialized_obligation(&existing) {
                    record.mark_failed_closed(
                        format!(
                            "materialized_obligation_conflict: host inbox record {} expected canonical obligation {} to preserve exact materialized truth",
                            record.record_id, obligation_id
                        ),
                        materialized_at,
                    );
                    self.persist_host_inbox_record_transaction(transaction, &record)?;
                    return Ok(record);
                }
                existing
            }
            None => {
                let obligation = record.materialize_as_local_obligation(materialized_at)?;
                self.persist_obligation_unlocked(transaction, &obligation)?;
                obligation
            }
        };

        record.mark_materialized(obligation.obligation_id.clone(), materialized_at);
        self.persist_host_inbox_record_transaction(transaction, &record)?;

        Ok(record)
        })
    }

    #[allow(dead_code)]
    pub(crate) fn acknowledge_inbox_item(
        &self,
        orchestration_session_id: &str,
        item_id: &str,
    ) -> Result<DurableInboxItemRecord> {
        self.resolve_inbox_item(
            orchestration_session_id,
            item_id,
            DurableInboxItemState::Acknowledged,
        )
    }

    #[allow(dead_code)]
    pub(crate) fn dismiss_inbox_item(
        &self,
        orchestration_session_id: &str,
        item_id: &str,
    ) -> Result<DurableInboxItemRecord> {
        self.resolve_inbox_item(
            orchestration_session_id,
            item_id,
            DurableInboxItemState::Dismissed,
        )
    }

    #[allow(dead_code)]
    pub(crate) fn load_inbox_item(
        &self,
        orchestration_session_id: &str,
        item_id: &str,
    ) -> Result<Option<DurableInboxItemRecord>> {
        let path = self.canonical_inbox_item_path(orchestration_session_id, item_id);
        let Some(item) = read_regular_json_if_exists::<DurableInboxItemRecord>(&path)? else {
            return Ok(None);
        };
        self.validate_inbox_item_record(&item)
            .with_context(|| format!("invalid durable inbox item in {}", path.display()))?;
        if item.orchestration_session_id != orchestration_session_id {
            anyhow::bail!(
                "durable inbox item {} belongs to session {} not {}",
                item_id,
                item.orchestration_session_id,
                orchestration_session_id
            );
        }
        if item.item_id != item_id {
            anyhow::bail!(
                "durable inbox artifact {} stored mismatched item_id {}",
                path.display(),
                item.item_id
            );
        }

        Ok(Some(item))
    }

    #[allow(dead_code)]
    pub(crate) fn list_inbox_items(
        &self,
        orchestration_session_id: &str,
    ) -> Result<Vec<DurableInboxItemRecord>> {
        let inbox_dir = self.canonical_inbox_dir(orchestration_session_id);
        let Some(entries) = safe_read_dir(&inbox_dir)? else {
            return Ok(Vec::new());
        };

        let mut items = Vec::new();
        for entry in entries {
            let entry = entry.with_context(|| format!("failed to read {}", inbox_dir.display()))?;
            let path = entry.path();
            if path.extension().and_then(|value| value.to_str()) != Some("json") {
                continue;
            }

            let Some(item) = read_regular_json_if_exists::<DurableInboxItemRecord>(&path)? else {
                continue;
            };
            self.validate_inbox_item_record(&item)
                .with_context(|| format!("invalid durable inbox item in {}", path.display()))?;
            if item.orchestration_session_id != orchestration_session_id {
                anyhow::bail!(
                    "durable inbox item {} belongs to session {} not {}",
                    item.item_id,
                    item.orchestration_session_id,
                    orchestration_session_id
                );
            }
            items.push(item);
        }

        items.sort_by(|left, right| {
            left.created_at
                .cmp(&right.created_at)
                .then(left.item_id.cmp(&right.item_id))
        });
        Ok(items)
    }

    #[allow(dead_code)]
    pub(crate) fn load_host_inbox_record(
        &self,
        record_id: &str,
    ) -> Result<Option<HostInboxRecord>> {
        let path = self.host_inbox_record_path(record_id)?;
        let Some(record) = self.load_host_inbox_record_unvalidated(record_id)? else {
            return Ok(None);
        };
        self.validate_host_inbox_record(&record)
            .with_context(|| format!("invalid host inbox record in {}", path.display()))?;

        Ok(Some(record))
    }

    fn load_host_inbox_record_unvalidated(
        &self,
        record_id: &str,
    ) -> Result<Option<HostInboxRecord>> {
        let path = self.host_inbox_record_path(record_id)?;
        let Some(record) = self.load_host_inbox_record_artifact(&path)? else {
            return Ok(None);
        };
        Self::validate_host_inbox_record_artifact_identity(
            &record,
            record_id,
            &path.display().to_string(),
        )?;

        Ok(Some(record))
    }

    #[allow(dead_code)]
    pub(crate) fn list_host_inbox_records(&self) -> Result<Vec<HostInboxRecord>> {
        let host_inbox_dir = self.host_inbox_dir();
        let Some(entries) = safe_read_dir(&host_inbox_dir)? else {
            return Ok(Vec::new());
        };

        let mut records = Vec::new();
        for entry in entries {
            let entry =
                entry.with_context(|| format!("failed to read {}", host_inbox_dir.display()))?;
            let path = entry.path();
            if path.extension().and_then(|value| value.to_str()) != Some("json") {
                continue;
            }

            let expected_record_id = Self::host_inbox_record_id_from_path(&path)?;
            let Some(record) = self.load_host_inbox_record_artifact(&path)? else {
                continue;
            };
            Self::validate_host_inbox_record_artifact_identity(
                &record,
                &expected_record_id,
                &path.display().to_string(),
            )?;
            self.validate_host_inbox_record(&record)
                .with_context(|| format!("invalid host inbox record in {}", path.display()))?;
            records.push(record);
        }

        records.sort_by(|left, right| {
            left.created_at
                .cmp(&right.created_at)
                .then(left.record_id.cmp(&right.record_id))
        });
        Ok(records)
    }

    #[cfg(any(target_os = "linux", test))]
    pub(crate) fn list_host_inbox_record_ids(&self) -> Result<Vec<String>> {
        let host_inbox_dir = self.host_inbox_dir();
        let Some(entries) = safe_read_dir(&host_inbox_dir)? else {
            return Ok(Vec::new());
        };

        let mut record_ids = Vec::new();
        for entry in entries {
            let entry =
                entry.with_context(|| format!("failed to read {}", host_inbox_dir.display()))?;
            let path = entry.path();
            if path.extension().and_then(|value| value.to_str()) != Some("json") {
                continue;
            }
            let Ok(record_id) = Self::host_inbox_record_id_from_path(&path) else {
                continue;
            };
            record_ids.push(record_id);
        }

        record_ids.sort();
        record_ids.dedup();
        Ok(record_ids)
    }

    #[cfg(any(target_os = "linux", test))]
    pub(crate) fn list_invalid_host_inbox_artifact_paths(&self) -> Result<Vec<PathBuf>> {
        let host_inbox_dir = self.host_inbox_dir();
        let Some(entries) = safe_read_dir(&host_inbox_dir)? else {
            return Ok(Vec::new());
        };

        let mut invalid_paths = Vec::new();
        for entry in entries {
            let entry =
                entry.with_context(|| format!("failed to read {}", host_inbox_dir.display()))?;
            let path = entry.path();
            if path.extension().and_then(|value| value.to_str()) != Some("json") {
                continue;
            }
            if Self::host_inbox_record_id_from_path(&path).is_err() {
                invalid_paths.push(path);
            }
        }

        invalid_paths.sort();
        invalid_paths.dedup();
        Ok(invalid_paths)
    }

    #[allow(dead_code)]
    pub(crate) fn load_obligation(
        &self,
        orchestration_session_id: &str,
        obligation_id: &str,
    ) -> Result<Option<OrchestrationObligationRecord>> {
        let path = self.canonical_obligation_path(orchestration_session_id, obligation_id);
        let Some(obligation) = read_regular_json_if_exists::<OrchestrationObligationRecord>(&path)?
        else {
            return Ok(None);
        };
        self.validate_obligation_record(&obligation)
            .with_context(|| format!("invalid orchestration obligation in {}", path.display()))?;
        if obligation.orchestration_session_id != orchestration_session_id {
            anyhow::bail!(
                "orchestration obligation {} belongs to session {} not {}",
                obligation_id,
                obligation.orchestration_session_id,
                orchestration_session_id
            );
        }
        if obligation.obligation_id != obligation_id {
            anyhow::bail!(
                "orchestration obligation artifact {} stored mismatched obligation_id {}",
                path.display(),
                obligation.obligation_id
            );
        }

        Ok(Some(obligation))
    }

    #[allow(dead_code)]
    pub(crate) fn list_obligations(
        &self,
        orchestration_session_id: &str,
    ) -> Result<Vec<OrchestrationObligationRecord>> {
        let obligations_dir = self.canonical_obligations_dir(orchestration_session_id);
        let Some(entries) = safe_read_dir(&obligations_dir)? else {
            return Ok(Vec::new());
        };

        let mut obligations = Vec::new();
        for entry in entries {
            let entry =
                entry.with_context(|| format!("failed to read {}", obligations_dir.display()))?;
            let path = entry.path();
            if path.extension().and_then(|value| value.to_str()) != Some("json") {
                continue;
            }

            let Some(obligation) =
                read_regular_json_if_exists::<OrchestrationObligationRecord>(&path)?
            else {
                continue;
            };
            self.validate_obligation_record(&obligation)
                .with_context(|| {
                    format!("invalid orchestration obligation in {}", path.display())
                })?;
            if obligation.orchestration_session_id != orchestration_session_id {
                anyhow::bail!(
                    "orchestration obligation {} belongs to session {} not {}",
                    obligation.obligation_id,
                    obligation.orchestration_session_id,
                    orchestration_session_id
                );
            }
            obligations.push(obligation);
        }

        obligations.sort_by(|left, right| {
            left.created_at
                .cmp(&right.created_at)
                .then(left.obligation_id.cmp(&right.obligation_id))
        });
        Ok(obligations)
    }

    #[allow(dead_code)]
    pub(crate) fn claim_session_auto_attach(
        &self,
        orchestration_session_id: &str,
        router_identity: &str,
    ) -> Result<SessionAutoAttachClaim> {
        self.claim_session_auto_attach_matching(
            orchestration_session_id,
            router_identity,
            |_| true,
            "no_eligible_obligations",
        )
    }

    #[allow(dead_code)]
    pub(crate) fn claim_exact_session_auto_attach_obligation(
        &self,
        orchestration_session_id: &str,
        obligation_id: &str,
        router_identity: &str,
    ) -> Result<SessionAutoAttachClaim> {
        self.claim_session_auto_attach_matching(
            orchestration_session_id,
            router_identity,
            |obligation| obligation.obligation_id == obligation_id,
            "requested_obligation_not_claimable",
        )
    }

    fn claim_session_auto_attach_matching<F>(
        &self,
        orchestration_session_id: &str,
        router_identity: &str,
        matcher: F,
        no_candidate_reason: &'static str,
    ) -> Result<SessionAutoAttachClaim>
    where
        F: Fn(&OrchestrationObligationRecord) -> bool,
    {
        if router_identity.trim().is_empty() {
            anyhow::bail!("router-owned auto-attach claims must include a router_identity");
        }
        self.with_legacy_snapshot_transaction(|transaction| {
            let Some(record) =
                self.load_session_transaction(transaction, orchestration_session_id)?
            else {
                return Ok(SessionAutoAttachClaim::NoCandidate {
                    reason: "missing_session",
                });
            };
            let readiness =
                classify_router_auto_attach_session_readiness(&record, orchestration_session_id);
            if let RouterAutoAttachSessionReadiness::NoCandidate { reason } = readiness {
                return Ok(SessionAutoAttachClaim::NoCandidate { reason });
            }

            let obligations =
                self.list_obligations_transaction(transaction, orchestration_session_id)?;
            if let Some(obligation_id) = claimed_obligation_id(&obligations)? {
                return Ok(SessionAutoAttachClaim::AlreadyClaimed {
                    obligation_id: obligation_id.to_string(),
                });
            }
            let matching_obligations = obligations
                .into_iter()
                .filter(|obligation| matcher(obligation))
                .collect::<Vec<_>>();
            let Some(candidate) = select_attach_candidate(&matching_obligations) else {
                return Ok(SessionAutoAttachClaim::NoCandidate {
                    reason: no_candidate_reason,
                });
            };

            let mut claimed = candidate.clone();
            claimed.mark_attach_claimed(router_identity, Utc::now());
            self.validate_obligation_record(&claimed)?;
            Self::write_obligation_transaction(transaction, &claimed)?;

            Ok(SessionAutoAttachClaim::Claimed {
                obligation_id: claimed.obligation_id,
                attach_claim_owner: claimed
                    .attach_claim_owner
                    .expect("mark_attach_claimed must set attach_claim_owner"),
            })
        })
    }

    pub(crate) fn settle_session_auto_attach_after_attach_restored(
        &self,
        orchestration_session_id: &str,
        completion_reason: &str,
    ) -> Result<SessionAutoAttachSettleResult> {
        if completion_reason.trim().is_empty() {
            anyhow::bail!(
                "session attach restoration must include an explanation-ready completion reason"
            );
        }
        self.with_legacy_snapshot_transaction(|transaction| {
            let Some(record) =
                self.load_session_transaction(transaction, orchestration_session_id)?
            else {
                return Ok(SessionAutoAttachSettleResult::default());
            };
            if record.session.posture != OrchestrationSessionPosture::ActiveAttached {
                return Ok(SessionAutoAttachSettleResult::default());
            }

            let obligations =
                self.list_obligations_transaction(transaction, orchestration_session_id)?;
            let _ = claimed_obligation_id(&obligations)?;

            let mut result = SessionAutoAttachSettleResult::default();
            let settled_at = Utc::now();
            for mut obligation in obligations {
                if !obligation.is_pending() {
                    continue;
                }

                let changed = match obligation.attach_state {
                    OrchestrationObligationAttachState::Claimed => {
                        obligation.mark_attach_satisfied(completion_reason, settled_at);
                        result
                            .satisfied_obligation_ids
                            .push(obligation.obligation_id.clone());
                        true
                    }
                    OrchestrationObligationAttachState::Eligible => {
                        obligation.mark_attach_superseded(completion_reason, settled_at);
                        result
                            .superseded_obligation_ids
                            .push(obligation.obligation_id.clone());
                        true
                    }
                    OrchestrationObligationAttachState::NotEligible
                    | OrchestrationObligationAttachState::Satisfied
                    | OrchestrationObligationAttachState::FailedClosed
                    | OrchestrationObligationAttachState::Superseded => false,
                };
                if !changed {
                    continue;
                }

                self.validate_obligation_record(&obligation)?;
                Self::write_obligation_transaction(transaction, &obligation)?;
            }

            Ok(result)
        })
    }

    pub(crate) fn release_session_auto_attach_claim(
        &self,
        orchestration_session_id: &str,
        obligation_id: &str,
        attach_claim_owner: &str,
    ) -> Result<bool> {
        if obligation_id.trim().is_empty() {
            anyhow::bail!("releasing a session auto-attach claim requires obligation_id");
        }
        if attach_claim_owner.trim().is_empty() {
            anyhow::bail!("releasing a session auto-attach claim requires attach_claim_owner");
        }
        self.with_legacy_snapshot_transaction(|transaction| {
            let Some(mut obligation) = self.load_obligation_transaction(
                transaction,
                orchestration_session_id,
                obligation_id,
            )?
            else {
                return Ok(false);
            };
            if !obligation.is_pending()
                || obligation.attach_state != OrchestrationObligationAttachState::Claimed
                || obligation.attach_claim_owner.as_deref() != Some(attach_claim_owner)
            {
                return Ok(false);
            }

            obligation.release_attach_claim(Utc::now());
            self.validate_obligation_record(&obligation)?;
            Self::write_obligation_transaction(transaction, &obligation)?;
            Ok(true)
        })
    }

    pub(crate) fn settle_exact_session_auto_attach_obligation_failed_closed(
        &self,
        orchestration_session_id: &str,
        obligation_id: &str,
        completion_reason: &str,
    ) -> Result<SessionAutoAttachSettleResult> {
        if obligation_id.trim().is_empty() {
            anyhow::bail!(
                "exact session auto-attach fail-closed settlement requires obligation_id"
            );
        }
        if completion_reason.trim().is_empty() {
            anyhow::bail!(
                "exact session auto-attach fail-closed settlement must include an explanation-ready completion reason"
            );
        }
        self.with_legacy_snapshot_transaction(|transaction| {
            let Some(mut obligation) = self.load_obligation_transaction(
                transaction,
                orchestration_session_id,
                obligation_id,
            )?
            else {
                return Ok(SessionAutoAttachSettleResult::default());
            };
            if !obligation.is_pending()
                || !matches!(
                    obligation.attach_state,
                    OrchestrationObligationAttachState::Eligible
                        | OrchestrationObligationAttachState::Claimed
                )
            {
                return Ok(SessionAutoAttachSettleResult::default());
            }

            let settled_at = Utc::now();
            obligation.mark_attach_failed_closed(completion_reason, settled_at);
            self.validate_obligation_record(&obligation)?;
            Self::write_obligation_transaction(transaction, &obligation)?;

            Ok(SessionAutoAttachSettleResult {
                failed_closed_obligation_ids: vec![obligation.obligation_id],
                ..SessionAutoAttachSettleResult::default()
            })
        })
    }

    pub(crate) fn settle_session_auto_attach_failed_closed(
        &self,
        orchestration_session_id: &str,
        completion_reason: &str,
    ) -> Result<SessionAutoAttachSettleResult> {
        if completion_reason.trim().is_empty() {
            anyhow::bail!(
                "session auto-attach fail-closed settlement must include an explanation-ready completion reason"
            );
        }
        self.with_legacy_snapshot_transaction(|transaction| {
            let obligations =
                self.list_obligations_transaction(transaction, orchestration_session_id)?;
            let _ = claimed_obligation_id(&obligations)?;

            let mut result = SessionAutoAttachSettleResult::default();
            let settled_at = Utc::now();
            for mut obligation in obligations {
                if !obligation.is_pending() {
                    continue;
                }

                let changed = match obligation.attach_state {
                    OrchestrationObligationAttachState::Claimed
                    | OrchestrationObligationAttachState::Eligible => {
                        obligation.mark_attach_failed_closed(completion_reason, settled_at);
                        result
                            .failed_closed_obligation_ids
                            .push(obligation.obligation_id.clone());
                        true
                    }
                    OrchestrationObligationAttachState::NotEligible
                    | OrchestrationObligationAttachState::Satisfied
                    | OrchestrationObligationAttachState::FailedClosed
                    | OrchestrationObligationAttachState::Superseded => false,
                };
                if !changed {
                    continue;
                }

                self.validate_obligation_record(&obligation)?;
                Self::write_obligation_transaction(transaction, &obligation)?;
            }

            Ok(result)
        })
    }

    pub(crate) fn set_orchestration_session_world_binding(
        &self,
        session: &mut OrchestrationSessionRecord,
        world_id: impl Into<String>,
        world_generation: u64,
    ) -> Result<()> {
        let world_id = world_id.into();
        let updated = self.with_legacy_snapshot_transaction(|transaction| {
            let mut current = self.current_world_binding_session(transaction, session)?;
            current.set_world_binding(&world_id, world_generation);
            self.validate_session_record(&current)?;
            self.persist_parent_session_snapshot(transaction, &current)?;
            Ok(current)
        })?;
        *session = updated;
        Ok(())
    }

    #[cfg(any(target_os = "linux", test))]
    pub(crate) fn recover_active_shared_world_binding_from_local_metadata(
        &self,
        orchestration_session_id: &str,
    ) -> Result<Option<super::AgentRuntimeParticipantWorldBinding>> {
        self.recover_active_shared_world_binding_from_local_metadata_with_unreadable_fallback(
            orchestration_session_id,
            true,
        )
    }

    #[cfg(any(target_os = "linux", test))]
    pub(crate) fn recover_active_shared_world_binding_from_local_metadata_only(
        &self,
        orchestration_session_id: &str,
    ) -> Result<Option<super::AgentRuntimeParticipantWorldBinding>> {
        self.recover_active_shared_world_binding_from_local_metadata_with_unreadable_fallback(
            orchestration_session_id,
            false,
        )
    }

    #[cfg(any(target_os = "linux", test))]
    fn recover_active_shared_world_binding_from_local_metadata_with_unreadable_fallback(
        &self,
        orchestration_session_id: &str,
        allow_unreadable_live_fallback: bool,
    ) -> Result<Option<super::AgentRuntimeParticipantWorldBinding>> {
        fn is_permission_denied(err: &anyhow::Error) -> bool {
            err.chain().any(|cause| {
                cause
                    .downcast_ref::<io::Error>()
                    .is_some_and(|io_err| io_err.kind() == io::ErrorKind::PermissionDenied)
            })
        }

        if orchestration_session_id.trim().is_empty() {
            return Ok(None);
        }

        fn fallback_or_fail_closed(
            store: &AgentRuntimeStateStore,
            orchestration_session_id: &str,
            blocked_path: &Path,
            err: anyhow::Error,
            allow_unreadable_live_fallback: bool,
        ) -> Result<Option<super::AgentRuntimeParticipantWorldBinding>> {
            if allow_unreadable_live_fallback {
                if let Some(binding) = store
                    .recover_active_shared_world_binding_from_authoritative_live_participants(
                        orchestration_session_id,
                    )?
                {
                    return Ok(Some(binding));
                }
            }

            Err(err).with_context(|| {
                format!(
                    "shared_world_binding_repair_metadata_unreadable: cannot read {} and found no authoritative live world binding for orchestration session {}",
                    blocked_path.display(),
                    orchestration_session_id
                )
            })
        }

        let root = shared_world_metadata_root();
        let Some(entries) = (match safe_read_dir(&root) {
            Ok(entries) => entries,
            Err(err) if is_permission_denied(&err) => {
                return fallback_or_fail_closed(
                    self,
                    orchestration_session_id,
                    &root,
                    err,
                    allow_unreadable_live_fallback,
                );
            }
            Err(err) => return Err(err),
        }) else {
            return Ok(None);
        };

        let mut matches = Vec::new();
        for entry in entries {
            let entry = match entry {
                Ok(entry) => entry,
                Err(err) if err.kind() == io::ErrorKind::PermissionDenied => {
                    return fallback_or_fail_closed(
                        self,
                        orchestration_session_id,
                        &root,
                        err.into(),
                        allow_unreadable_live_fallback,
                    );
                }
                Err(err) => {
                    return Err(err).with_context(|| format!("failed to read {}", root.display()));
                }
            };
            let metadata_path = entry.path().join(SHARED_WORLD_METADATA_FILE);
            if !metadata_path.is_file() {
                continue;
            }

            let Some(metadata) =
                (match read_regular_json_if_exists::<SharedWorldMetadataRecord>(&metadata_path) {
                    Ok(metadata) => metadata,
                    Err(err) if is_permission_denied(&err) => {
                        return fallback_or_fail_closed(
                            self,
                            orchestration_session_id,
                            &metadata_path,
                            err,
                            allow_unreadable_live_fallback,
                        );
                    }
                    Err(_) => continue,
                })
            else {
                continue;
            };
            if metadata.owner_mode != SharedWorldMetadataOwnerMode::SharedOrchestration
                || metadata.orchestration_session_id.as_deref() != Some(orchestration_session_id)
                || metadata.binding_state != Some(SharedWorldBindingState::Active)
            {
                continue;
            }

            let world_generation = match metadata.world_generation {
                Some(world_generation) => world_generation,
                None => continue,
            };
            if metadata.world_id.trim().is_empty() {
                continue;
            }

            matches.push(super::AgentRuntimeParticipantWorldBinding {
                world_id: metadata.world_id,
                world_generation,
            });
        }

        match matches.len() {
            0 => Ok(None),
            1 => Ok(matches.pop()),
            _ => anyhow::bail!(
                "ambiguous_shared_world_binding: multiple active shared-world bindings were found for orchestration session {}",
                orchestration_session_id
            ),
        }
    }

    #[cfg(any(target_os = "linux", test))]
    fn recover_active_shared_world_binding_from_authoritative_live_participants(
        &self,
        orchestration_session_id: &str,
    ) -> Result<Option<super::AgentRuntimeParticipantWorldBinding>> {
        let mut matches = BTreeSet::new();
        for participant in self.list_live_participants_for_session(orchestration_session_id)? {
            if participant.handle.role != MEMBER_ROLE
                || participant.handle.execution.scope != AgentExecutionScope::World
            {
                continue;
            }

            let world_id = participant.handle.world_id.clone().ok_or_else(|| {
                anyhow::anyhow!(
                    "corrupt_shared_world_binding_repair_authority: live retained worker {} is missing world_id",
                    participant.handle.participant_id
                )
            })?;
            let world_generation = participant.handle.world_generation.ok_or_else(|| {
                anyhow::anyhow!(
                    "corrupt_shared_world_binding_repair_authority: live retained worker {} is missing world_generation",
                    participant.handle.participant_id
                )
            })?;
            if world_id.trim().is_empty() {
                anyhow::bail!(
                    "corrupt_shared_world_binding_repair_authority: live retained worker {} has an empty world_id",
                    participant.handle.participant_id
                );
            }

            matches.insert((world_id, world_generation));
        }

        match matches.len() {
            0 => Ok(None),
            1 => {
                let (world_id, world_generation) =
                    matches.into_iter().next().expect("single binding");
                Ok(Some(super::AgentRuntimeParticipantWorldBinding {
                    world_id,
                    world_generation,
                }))
            }
            _ => anyhow::bail!(
                "ambiguous_shared_world_binding: multiple authoritative live world bindings were found for orchestration session {}",
                orchestration_session_id
            ),
        }
    }

    pub(crate) fn clear_orchestration_session_world_binding(
        &self,
        session: &mut OrchestrationSessionRecord,
    ) -> Result<()> {
        let updated = self.with_legacy_snapshot_transaction(|transaction| {
            let mut current = self.current_world_binding_session(transaction, session)?;
            current.clear_world_binding();
            self.validate_session_record(&current)?;
            self.persist_parent_session_snapshot(transaction, &current)?;
            Ok(current)
        })?;
        *session = updated;
        Ok(())
    }

    fn current_world_binding_session(
        &self,
        transaction: &mut super::host_session_authority::store::LegacyWriterGuard,
        supplied: &OrchestrationSessionRecord,
    ) -> Result<OrchestrationSessionRecord> {
        let Some(current) = self.load_authoritative_session_transaction(
            transaction,
            &supplied.orchestration_session_id,
        )?
        else {
            return Ok(supplied.clone());
        };

        let binding_changed = current.world_id != supplied.world_id
            || current.world_generation != supplied.world_generation;
        let mut expected = current.clone();
        expected.world_id.clone_from(&supplied.world_id);
        expected.world_generation = supplied.world_generation;
        if binding_changed {
            expected.last_active_at = supplied.last_active_at;
        }
        if let (Some(expected_contract), Some(supplied_contract)) = (
            expected.host_attach_contract.as_mut(),
            supplied.host_attach_contract.as_ref(),
        ) {
            if expected_contract.continuity_uaa_session_id.is_some()
                && supplied_contract.continuity_uaa_session_id.is_none()
            {
                expected_contract.continuity_uaa_session_id = None;
            }
        }
        if expected != *supplied {
            anyhow::bail!(
                "stale_world_binding_session_snapshot: orchestration session {} changed before world-binding persistence",
                supplied.orchestration_session_id
            );
        }
        Ok(current)
    }

    fn persist_lease(
        &self,
        transaction: &mut super::host_session_authority::store::LegacyWriterGuard,
        participant: &AgentRuntimeParticipantRecord,
    ) -> Result<()> {
        use super::host_session_authority::store::LegacyStateStoreCollectionV1::{
            Participants, Sessions,
        };
        let payload = serde_json::json!({
            "participant_id": participant.handle.participant_id,
            "session_handle_id": participant.handle.session_handle_id,
            "shell_owner_pid": participant.internal.shell_owner_pid,
            "lease_token": participant.internal.lease_token,
            "state": participant.handle.state,
            "ownership_valid": participant.internal.ownership_valid,
            "last_heartbeat_at": participant.internal.last_heartbeat_at,
            "terminal_observed_at": participant.internal.terminal_observed_at,
        });
        let lease_name = format!("{}.lease", participant.handle.participant_id);
        Self::transaction_write_json(transaction, Participants, &[lease_name.as_str()], &payload)?;
        Self::transaction_write_json(
            transaction,
            Sessions,
            &[
                participant.handle.orchestration_session_id.as_str(),
                "leases",
                lease_name.as_str(),
            ],
            &payload,
        )
    }

    fn persist_parent_session_snapshot(
        &self,
        transaction: &mut super::host_session_authority::store::LegacyWriterGuard,
        session: &OrchestrationSessionRecord,
    ) -> Result<()> {
        use super::host_session_authority::store::LegacyStateStoreCollectionV1::Sessions;
        self.validate_session_record(session)?;
        let flat_name = format!("{}.json", session.orchestration_session_id);
        Self::transaction_write_json(transaction, Sessions, &[flat_name.as_str()], session)?;
        Self::transaction_write_json(
            transaction,
            Sessions,
            &[session.orchestration_session_id.as_str(), "session.json"],
            session,
        )
    }

    fn read_participant_dir(&self, dir: &Path) -> Result<Vec<AgentRuntimeParticipantRecord>> {
        let Some(entries) = safe_read_dir(dir)? else {
            return Ok(Vec::new());
        };
        let mut participants = Vec::new();
        for entry in entries {
            let entry = entry.with_context(|| format!("failed to read {}", dir.display()))?;
            let path = entry.path();
            if path.extension().and_then(|value| value.to_str()) != Some("json") {
                continue;
            }
            let Some(participant) =
                read_regular_json_if_exists::<AgentRuntimeParticipantRecord>(&path)?
            else {
                continue;
            };
            self.validate_participant_record(&participant)
                .with_context(|| format!("invalid participant record in {}", path.display()))?;
            participants.push(participant);
        }

        Ok(participants)
    }

    fn read_canonical_participants(
        &self,
    ) -> Result<Vec<(AgentRuntimeParticipantRecord, ParticipantRecordSource)>> {
        let mut participants = Vec::new();

        for orchestration_session_id in self.canonical_session_root_ids()? {
            let participants_dir = self.canonical_participants_dir(&orchestration_session_id);
            let Some(entries) = safe_read_dir(&participants_dir)? else {
                continue;
            };
            for entry in entries {
                let entry = entry
                    .with_context(|| format!("failed to read {}", participants_dir.display()))?;
                let path = entry.path();
                if path.extension().and_then(|value| value.to_str()) != Some("json") {
                    continue;
                }
                let Some(participant) =
                    read_regular_json_if_exists::<AgentRuntimeParticipantRecord>(&path)?
                else {
                    continue;
                };
                self.validate_participant_record(&participant)
                    .with_context(|| format!("invalid participant record in {}", path.display()))?;
                participants.push((participant, ParticipantRecordSource::Canonical));
            }
        }

        Ok(participants)
    }

    #[allow(dead_code)]
    pub(crate) fn load_orchestration_session(
        &self,
        orchestration_session_id: &str,
    ) -> Result<Option<OrchestrationSessionRecord>> {
        self.load_authoritative_session(orchestration_session_id)
    }

    #[allow(dead_code)]
    pub(crate) fn list_orchestration_sessions(&self) -> Result<Vec<OrchestrationSessionRecord>> {
        let mut sessions = Vec::new();
        let mut session_ids = BTreeSet::new();
        for session_id in self.canonical_session_root_ids()? {
            session_ids.insert(session_id);
        }
        for session_id in self.flat_session_ids()? {
            session_ids.insert(session_id);
        }

        for session_id in session_ids {
            if let Some(session) = self.load_authoritative_session(&session_id)? {
                sessions.push(session);
            }
        }

        sessions.sort_by_key(|session| session.last_active_at);
        Ok(sessions)
    }

    #[allow(dead_code)]
    pub(crate) fn list_router_auto_attach_candidate_session_ids(&self) -> Result<Vec<String>> {
        let mut session_ids = Vec::new();
        for record in self.list_sessions()? {
            let orchestration_session_id = record.orchestration_session_id().to_string();
            if !matches!(
                classify_router_auto_attach_session_readiness(&record, &orchestration_session_id),
                RouterAutoAttachSessionReadiness::Eligible
            ) {
                continue;
            }

            let obligations = self.list_obligations(&orchestration_session_id)?;
            let claimed_obligation = match claimed_obligation_id(&obligations) {
                Ok(claimed_obligation) => claimed_obligation,
                Err(_) => continue,
            };
            if claimed_obligation.is_some() || select_attach_candidate(&obligations).is_none() {
                continue;
            }

            session_ids.push(orchestration_session_id);
        }
        session_ids.sort();
        session_ids.dedup();
        Ok(session_ids)
    }

    #[allow(dead_code)]
    pub(crate) fn find_active_orchestration_session_for_pid(
        &self,
        pid: u32,
    ) -> Result<Option<OrchestrationSessionRecord>> {
        let matches = self
            .list_orchestration_sessions()?
            .into_iter()
            .filter(|session| {
                session.shell_owner_pid == pid
                    && session.state == OrchestrationSessionState::Active
                    && owner_pid_is_alive(session.shell_owner_pid)
            })
            .collect::<Vec<_>>();
        match matches.len() {
            0 => Ok(None),
            1 => Ok(matches.into_iter().next()),
            _ => Err(anyhow::anyhow!(
                "multiple active orchestration sessions found for shell pid {pid}"
            )),
        }
    }

    #[allow(dead_code)]
    pub(crate) fn persist_manifest(&self, manifest: &AgentRuntimeSessionManifest) -> Result<()> {
        self.persist_participant(manifest)
    }

    #[allow(dead_code)]
    pub(crate) fn list_manifests(&self) -> Result<Vec<AgentRuntimeSessionManifest>> {
        self.list_participants()
    }

    #[allow(dead_code)]
    pub(crate) fn list_live_manifests(&self) -> Result<Vec<AgentRuntimeSessionManifest>> {
        self.list_live_participants()
    }

    #[allow(dead_code)]
    pub(crate) fn find_live_orchestrator(
        &self,
        agent_id: &str,
    ) -> Result<Option<AgentRuntimeSessionManifest>> {
        Ok(self
            .resolve_live_orchestrator_participant(agent_id)?
            .map(|(_, participant)| participant))
    }

    #[allow(dead_code)]
    pub(crate) fn resolve_live_orchestrator_session(
        &self,
        agent_id: &str,
    ) -> Result<Option<(OrchestrationSessionRecord, AgentRuntimeSessionManifest)>> {
        self.resolve_live_orchestrator_participant(agent_id)
    }

    #[allow(dead_code)]
    fn load_manifest(&self, participant_id: &str) -> Result<Option<AgentRuntimeSessionManifest>> {
        self.load_participant(participant_id)
    }

    fn load_authoritative_session(
        &self,
        orchestration_session_id: &str,
    ) -> Result<Option<OrchestrationSessionRecord>> {
        let canonical_dir = self.canonical_session_dir(orchestration_session_id);
        if safe_metadata(&canonical_dir)?.is_some_and(|metadata| metadata.is_dir()) {
            let canonical_path = self.canonical_session_path(orchestration_session_id);
            if let Some(session) =
                read_regular_json_if_exists::<OrchestrationSessionRecord>(&canonical_path)?
            {
                self.validate_session_record(&session).with_context(|| {
                    format!("invalid session record in {}", canonical_path.display())
                })?;
                return Ok(Some(session));
            }
        }

        let flat_path = self.orchestration_session_path(orchestration_session_id);
        if let Some(session) = read_regular_json_if_exists(&flat_path)? {
            self.validate_session_record(&session)
                .with_context(|| format!("invalid session record in {}", flat_path.display()))?;
            return Ok(Some(session));
        }
        Ok(None)
    }

    fn public_session_selector_error(&self, selector: &str) -> anyhow::Error {
        if selector.trim().is_empty() {
            return anyhow::anyhow!(
                "unknown_session: public control actions require --session <orchestration_session_id>"
            );
        }

        if self
            .list_orchestration_sessions()
            .map(|sessions| {
                sessions
                    .into_iter()
                    .any(|session| session.active_participant_id() == Some(selector))
            })
            .unwrap_or(false)
        {
            return anyhow::anyhow!(
                "unknown_session: selector '{}' matched active_session_handle_id; public control actions accept only orchestration_session_id",
                selector
            );
        }

        if self
            .list_participants_across_sources()
            .map(|participants| {
                participants.into_iter().any(|participant| {
                    participant.participant_id() == selector
                        || participant.handle.session_handle_id == selector
                })
            })
            .unwrap_or(false)
        {
            return anyhow::anyhow!(
                "unknown_session: selector '{}' matched participant_id/session_handle_id; public control actions accept only orchestration_session_id",
                selector
            );
        }

        if self
            .list_participants_across_sources()
            .map(|participants| {
                participants
                    .into_iter()
                    .any(|participant| participant.internal_uaa_session_id() == Some(selector))
            })
            .unwrap_or(false)
        {
            return anyhow::anyhow!(
                "unknown_session: selector '{}' matched internal.uaa_session_id; public control actions accept only orchestration_session_id",
                selector
            );
        }

        anyhow::anyhow!(
            "unknown_session: no orchestration session found for '{}'",
            selector
        )
    }

    #[allow(dead_code)]
    fn public_turn_session_selector_error(&self, selector: &str) -> anyhow::Error {
        if selector.trim().is_empty() {
            return anyhow::anyhow!(
                "unknown_session: public turn actions require --session <orchestration_session_id>"
            );
        }

        if self
            .list_orchestration_sessions()
            .map(|sessions| {
                sessions
                    .into_iter()
                    .any(|session| session.active_participant_id() == Some(selector))
            })
            .unwrap_or(false)
        {
            return anyhow::anyhow!(
                "noncanonical_session_selector: selector '{}' matched active_session_handle_id; public turn actions accept only orchestration_session_id",
                selector
            );
        }

        if self
            .list_participants_across_sources()
            .map(|participants| {
                participants.into_iter().any(|participant| {
                    participant.participant_id() == selector
                        || participant.handle.session_handle_id == selector
                })
            })
            .unwrap_or(false)
        {
            return anyhow::anyhow!(
                "noncanonical_session_selector: selector '{}' matched participant_id/session_handle_id; public turn actions accept only orchestration_session_id",
                selector
            );
        }

        if self
            .list_participants_across_sources()
            .map(|participants| {
                participants
                    .into_iter()
                    .any(|participant| participant.internal_uaa_session_id() == Some(selector))
            })
            .unwrap_or(false)
        {
            return anyhow::anyhow!(
                "noncanonical_session_selector: selector '{}' matched internal.uaa_session_id; public turn actions accept only orchestration_session_id",
                selector
            );
        }

        anyhow::anyhow!(
            "unknown_session: no orchestration session found for '{}'",
            selector
        )
    }

    fn canonical_session_root_ids(&self) -> Result<Vec<String>> {
        let Some(entries) = safe_read_dir(&self.sessions_dir())? else {
            return Ok(Vec::new());
        };

        let mut session_ids = Vec::new();
        for entry in entries {
            let entry = entry
                .with_context(|| format!("failed to read {}", self.sessions_dir().display()))?;
            let path = entry.path();
            let Some(metadata) = safe_metadata(&path)? else {
                continue;
            };
            if metadata.file_type().is_dir() {
                let file_name = entry.file_name();
                let Some(session_id) = file_name.to_str() else {
                    continue;
                };
                session_ids.push(session_id.to_string());
            }
        }
        session_ids.sort();
        Ok(session_ids)
    }

    fn flat_session_ids(&self) -> Result<Vec<String>> {
        let Some(entries) = safe_read_dir(&self.sessions_dir())? else {
            return Ok(Vec::new());
        };

        let mut session_ids = Vec::new();
        for entry in entries {
            let entry = entry
                .with_context(|| format!("failed to read {}", self.sessions_dir().display()))?;
            let path = entry.path();
            let Some(metadata) = safe_metadata(&path)? else {
                continue;
            };
            if !metadata.is_file()
                || path.extension().and_then(|value| value.to_str()) != Some("json")
            {
                continue;
            }
            let Some(stem) = path.file_stem().and_then(|value| value.to_str()) else {
                continue;
            };
            session_ids.push(stem.to_string());
        }
        session_ids.sort();
        Ok(session_ids)
    }

    fn build_session_record(
        &self,
        orchestration_session_id: &str,
        session: Option<OrchestrationSessionRecord>,
        mut participants: Vec<AgentRuntimeParticipantRecord>,
    ) -> AgentRuntimeSessionRecord {
        participants.sort_by(|left, right| {
            left.handle
                .last_transition_at
                .cmp(&right.handle.last_transition_at)
                .then(left.handle.participant_id.cmp(&right.handle.participant_id))
        });

        let has_authoritative_parent = session.is_some();
        let mut warnings = Vec::new();
        if !has_authoritative_parent {
            warnings.push(format!(
                "orchestration session {orchestration_session_id} is missing authoritative parent session metadata"
            ));
        }

        let session = session
            .unwrap_or_else(|| synthesize_session_record(orchestration_session_id, &participants));

        let contract_valid = match validate_runtime_contract(&session, &participants) {
            Ok(()) => true,
            Err(err) => {
                warnings.push(format!(
                    "orchestration session {} violates persisted runtime contract: {err}",
                    session.orchestration_session_id
                ));
                false
            }
        };

        let complete = if !has_authoritative_parent || !contract_valid {
            false
        } else if session.state != OrchestrationSessionState::Active {
            true
        } else {
            match session_authoritative_participant_id(&session) {
                Some(active_participant_id) => match participants
                    .iter()
                    .find(|participant| participant.handle.participant_id == active_participant_id)
                {
                    Some(participant) if participant.matches_public_parent_linkage(&session) => {
                        if session_attached_to_participant(&session, participant) {
                            participant.is_authoritative_live()
                                && owner_process_is_alive(participant)
                        } else {
                            valid_detached_host_continuity_posture(&session, participant, true)
                                .is_some()
                        }
                    }
                    Some(participant) => {
                        warnings.push(format!(
                            "active orchestration session {} references incomplete live orchestrator participant {}",
                            session.orchestration_session_id, participant.handle.participant_id
                        ));
                        false
                    }
                    None => {
                        warnings.push(format!(
                            "active orchestration session {} references missing participant {}",
                            session.orchestration_session_id, active_participant_id
                        ));
                        false
                    }
                },
                None => {
                    if session.posture == OrchestrationSessionPosture::BornUnattached
                        && born_unattached_status_anchor(&AgentRuntimeSessionRecord {
                            session: session.clone(),
                            participants: participants.clone(),
                            warnings: Vec::new(),
                            has_authoritative_parent,
                            complete: false,
                        })
                        .is_some()
                    {
                        true
                    } else {
                        warnings.push(format!(
                            "active orchestration session {} is missing authoritative orchestrator participant linkage",
                            session.orchestration_session_id
                        ));
                        false
                    }
                }
            }
        };

        AgentRuntimeSessionRecord {
            session,
            participants,
            warnings,
            has_authoritative_parent,
            complete,
        }
    }

    fn resolve_inbox_item(
        &self,
        orchestration_session_id: &str,
        item_id: &str,
        state: DurableInboxItemState,
    ) -> Result<DurableInboxItemRecord> {
        if state.is_pending() {
            anyhow::bail!("durable inbox resolution requires a terminal inbox state");
        }
        self.with_legacy_snapshot_transaction(|transaction| {
            let mut item = self
                .load_inbox_item_transaction(transaction, orchestration_session_id, item_id)?
                .ok_or_else(|| {
                    anyhow::anyhow!(
                        "durable inbox item {} not found in session {}",
                        item_id,
                        orchestration_session_id
                    )
                })?;
            item.transition_state(state);
            self.persist_inbox_item_unlocked(transaction, &item)?;
            Ok(item)
        })
    }

    #[cfg(any(target_os = "linux", test))]
    #[allow(dead_code)]
    fn load_exact_pending_approval_obligation_for_continue_target(
        &self,
        orchestration_session_id: &str,
        target_participant_id: &str,
        target_backend_id: &str,
        world_id: &str,
        world_generation: u64,
        approval_obligation_id: &str,
    ) -> Result<OrchestrationObligationRecord> {
        let obligation = self
            .load_obligation(orchestration_session_id, approval_obligation_id)?
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "approval_obligation_not_found: orchestration session {} has no approval obligation {}",
                    orchestration_session_id,
                    approval_obligation_id
                )
            })?;
        if obligation.kind != OrchestrationObligationKind::ApprovalRequired {
            anyhow::bail!(
                "approval_obligation_kind_mismatch: orchestration session {} obligation {} is not approval_required",
                orchestration_session_id,
                approval_obligation_id
            );
        }
        if !obligation.is_pending() {
            anyhow::bail!(
                "approval_obligation_already_resolved: orchestration session {} approval obligation {} is already closed",
                orchestration_session_id,
                approval_obligation_id
            );
        }
        if obligation.source_participant_id.as_deref() != Some(target_participant_id) {
            anyhow::bail!(
                "approval_obligation_target_mismatch: orchestration session {} approval obligation {} does not bind retained worker {}",
                orchestration_session_id,
                approval_obligation_id,
                target_participant_id
            );
        }
        if obligation.target_backend_id.as_deref() != Some(target_backend_id) {
            anyhow::bail!(
                "approval_obligation_backend_mismatch: orchestration session {} approval obligation {} does not bind backend {}",
                orchestration_session_id,
                approval_obligation_id,
                target_backend_id
            );
        }
        if obligation.world_id.as_deref() != Some(world_id)
            || obligation.world_generation != Some(world_generation)
        {
            anyhow::bail!(
                "approval_obligation_world_binding_mismatch: orchestration session {} approval obligation {} no longer matches authoritative world binding {}/{}",
                orchestration_session_id,
                approval_obligation_id,
                world_id,
                world_generation
            );
        }

        Ok(obligation)
    }

    #[cfg(any(target_os = "linux", test))]
    #[allow(dead_code)]
    fn load_exact_pending_follow_up_obligation_for_continue_target(
        &self,
        orchestration_session_id: &str,
        target_participant_id: &str,
        target_backend_id: &str,
        world_id: &str,
        world_generation: u64,
        follow_up_obligation_id: &str,
    ) -> Result<OrchestrationObligationRecord> {
        let obligation = self
            .load_obligation(orchestration_session_id, follow_up_obligation_id)?
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "follow_up_obligation_not_found: orchestration session {} has no follow-up obligation {}",
                    orchestration_session_id,
                    follow_up_obligation_id
                )
            })?;
        if obligation.kind != OrchestrationObligationKind::FollowUpRequired {
            anyhow::bail!(
                "follow_up_obligation_kind_mismatch: orchestration session {} obligation {} is not follow_up_required",
                orchestration_session_id,
                follow_up_obligation_id
            );
        }
        if !obligation.is_pending() {
            anyhow::bail!(
                "follow_up_obligation_already_resolved: orchestration session {} follow-up obligation {} is already closed",
                orchestration_session_id,
                follow_up_obligation_id
            );
        }
        if obligation.source_participant_id.as_deref() != Some(target_participant_id) {
            anyhow::bail!(
                "follow_up_obligation_target_mismatch: orchestration session {} follow-up obligation {} does not bind retained worker {}",
                orchestration_session_id,
                follow_up_obligation_id,
                target_participant_id
            );
        }
        if obligation.target_backend_id.as_deref() != Some(target_backend_id) {
            anyhow::bail!(
                "follow_up_obligation_backend_mismatch: orchestration session {} follow-up obligation {} does not bind backend {}",
                orchestration_session_id,
                follow_up_obligation_id,
                target_backend_id
            );
        }
        if obligation.world_id.as_deref() != Some(world_id)
            || obligation.world_generation != Some(world_generation)
        {
            anyhow::bail!(
                "follow_up_obligation_world_binding_mismatch: orchestration session {} follow-up obligation {} no longer matches authoritative world binding {}/{}",
                orchestration_session_id,
                follow_up_obligation_id,
                world_id,
                world_generation
            );
        }

        Ok(obligation)
    }
}

fn retired_public_turn_backend_guidance(backend_id: &str) -> Option<&'static str> {
    match backend_id {
        "cli:codex" => {
            Some("legacy exact backend 'cli:codex' is retired; use 'cli:codex-host' or 'cli:codex-world'")
        }
        "cli:claude_code" => Some(
            "legacy exact backend 'cli:claude_code' is retired; use 'cli:claude_code-host' or 'cli:claude_code-world'",
        ),
        "cli:codex_world" => {
            Some("legacy exact backend 'cli:codex_world' is retired; use 'cli:codex-world'")
        }
        "cli:claude_code_world" => Some(
            "legacy exact backend 'cli:claude_code_world' is retired; use 'cli:claude_code-world'",
        ),
        _ => None,
    }
}

fn write_atomic_json(path: &Path, value: &impl serde::Serialize) -> Result<()> {
    let parent = path
        .parent()
        .ok_or_else(|| anyhow::anyhow!("{} has no parent directory", path.display()))?;
    fs::create_dir_all(parent).with_context(|| format!("failed to create {}", parent.display()))?;

    let mut tmp = NamedTempFile::new_in(parent)
        .with_context(|| format!("failed to create temp file in {}", parent.display()))?;
    serde_json::to_writer_pretty(tmp.as_file_mut(), value)
        .with_context(|| format!("failed to serialize {}", path.display()))?;
    tmp.as_file_mut()
        .sync_all()
        .with_context(|| format!("failed to flush {}", path.display()))?;
    tmp.persist(path)
        .map_err(|err| err.error)
        .with_context(|| format!("failed to persist {}", path.display()))?;
    Ok(())
}

fn snapshot_write_lock() -> &'static Mutex<()> {
    static SNAPSHOT_WRITE_LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    SNAPSHOT_WRITE_LOCK.get_or_init(|| Mutex::new(()))
}

fn updated_pending_inbox_count(current: u64, was_pending: bool, is_pending: bool) -> Result<u64> {
    match (was_pending, is_pending) {
        (false, false) | (true, true) => Ok(current),
        (false, true) => current
            .checked_add(1)
            .ok_or_else(|| anyhow::anyhow!("pending_inbox_count overflow")),
        (true, false) => current
            .checked_sub(1)
            .ok_or_else(|| anyhow::anyhow!("pending_inbox_count underflow")),
    }
}

fn apply_pending_inbox_count(session: &mut OrchestrationSessionRecord, pending_inbox_count: u64) {
    session.apply_detached_pending_inbox_count(pending_inbox_count);
}

fn projected_pending_inbox_count_from_obligations(
    obligations: &[OrchestrationObligationRecord],
) -> Result<u64> {
    obligations
        .iter()
        .filter(|obligation| obligation.projects_detached_attention())
        .count()
        .try_into()
        .map_err(|_| anyhow::anyhow!("pending_inbox_count overflow"))
}

fn compatibility_inbox_kind_for_obligation(
    kind: OrchestrationObligationKind,
) -> Option<DurableInboxItemKind> {
    match kind {
        OrchestrationObligationKind::ApprovalRequired => {
            Some(DurableInboxItemKind::ApprovalRequired)
        }
        OrchestrationObligationKind::TaskCompleted => Some(DurableInboxItemKind::CompletionNotice),
        OrchestrationObligationKind::FollowUpRequired
        | OrchestrationObligationKind::ForkRequest
        | OrchestrationObligationKind::ForkRecommendation
        | OrchestrationObligationKind::ResultAvailable => {
            Some(DurableInboxItemKind::FollowUpMessage)
        }
        OrchestrationObligationKind::Blocked
        | OrchestrationObligationKind::TaskFailed
        | OrchestrationObligationKind::RuntimeAlert
        | OrchestrationObligationKind::EscalationRecommended => {
            Some(DurableInboxItemKind::RuntimeAlert)
        }
    }
}

fn compatibility_inbox_item_from_obligation(
    obligation: &OrchestrationObligationRecord,
    existing: Option<&DurableInboxItemRecord>,
) -> Option<DurableInboxItemRecord> {
    let kind = compatibility_inbox_kind_for_obligation(obligation.kind)?;
    let message = obligation
        .resolution_note
        .clone()
        .or_else(|| Some(obligation.summary.clone()));
    let mut item = existing.cloned().unwrap_or_else(|| DurableInboxItemRecord {
        orchestration_session_id: obligation.orchestration_session_id.clone(),
        item_id: obligation.obligation_id.clone(),
        kind,
        state: DurableInboxItemState::Pending,
        created_at: obligation.created_at,
        updated_at: obligation.updated_at,
        resolved_at: None,
        message: message.clone(),
    });
    item.kind = kind;
    item.message = message;
    item.updated_at = obligation.updated_at;
    if obligation.projects_detached_attention() {
        item.state = DurableInboxItemState::Pending;
        item.resolved_at = None;
    } else {
        item.state = match obligation.review_state {
            OrchestrationObligationReviewState::Acknowledged => DurableInboxItemState::Acknowledged,
            OrchestrationObligationReviewState::Resolved
                if obligation.kind == OrchestrationObligationKind::ApprovalRequired =>
            {
                DurableInboxItemState::Acknowledged
            }
            OrchestrationObligationReviewState::Unread
            | OrchestrationObligationReviewState::Resolved
            | OrchestrationObligationReviewState::Dismissed => DurableInboxItemState::Dismissed,
        };
        item.resolved_at = obligation.resolved_at.or(Some(obligation.updated_at));
    }
    Some(item)
}

fn participant_snapshot_freshness(participant: &AgentRuntimeParticipantRecord) -> DateTime<Utc> {
    [
        Some(participant.handle.last_transition_at),
        participant.internal.last_event_at,
        participant.internal.last_heartbeat_at,
        participant.internal.ownership_verified_at,
        participant.internal.terminal_observed_at,
    ]
    .into_iter()
    .flatten()
    .max()
    .unwrap_or(participant.handle.last_transition_at)
}

fn participant_state_rank(state: &super::session::AgentRuntimeSessionState) -> u8 {
    match state {
        super::session::AgentRuntimeSessionState::Allocating => 0,
        super::session::AgentRuntimeSessionState::Ready => 1,
        super::session::AgentRuntimeSessionState::Running => 2,
        super::session::AgentRuntimeSessionState::Restarting => 3,
        super::session::AgentRuntimeSessionState::Stopping => 4,
        super::session::AgentRuntimeSessionState::Stopped => 5,
        super::session::AgentRuntimeSessionState::Failed => 6,
        super::session::AgentRuntimeSessionState::Invalidated => 7,
    }
}

fn should_persist_participant_snapshot(
    existing: &AgentRuntimeParticipantRecord,
    incoming: &AgentRuntimeParticipantRecord,
) -> bool {
    if !existing.handle.state.is_live() && incoming.handle.state.is_live() {
        return false;
    }
    if participant_state_rank(&incoming.handle.state)
        < participant_state_rank(&existing.handle.state)
        && incoming.handle.last_transition_at <= existing.handle.last_transition_at
    {
        return false;
    }

    match participant_snapshot_freshness(incoming).cmp(&participant_snapshot_freshness(existing)) {
        Ordering::Greater => true,
        Ordering::Less => false,
        Ordering::Equal => {
            let incoming_terminal = incoming.internal.terminal_observed_at.is_some();
            let existing_terminal = existing.internal.terminal_observed_at.is_some();
            (
                incoming.handle.last_transition_at,
                incoming_terminal,
                participant_state_rank(&incoming.handle.state),
            ) >= (
                existing.handle.last_transition_at,
                existing_terminal,
                participant_state_rank(&existing.handle.state),
            )
        }
    }
}

fn orchestration_session_freshness(session: &OrchestrationSessionRecord) -> DateTime<Utc> {
    session
        .closed_at
        .unwrap_or(session.last_active_at)
        .max(session.last_active_at)
}

fn orchestration_session_state_rank(state: &OrchestrationSessionState) -> u8 {
    match state {
        OrchestrationSessionState::Allocating => 0,
        OrchestrationSessionState::Active => 1,
        OrchestrationSessionState::Stopping => 2,
        OrchestrationSessionState::Stopped => 3,
        OrchestrationSessionState::Failed => 4,
        OrchestrationSessionState::Invalidated => 5,
    }
}

fn should_persist_orchestration_session_snapshot(
    existing: &OrchestrationSessionRecord,
    incoming: &OrchestrationSessionRecord,
) -> bool {
    if matches!(
        existing.state,
        OrchestrationSessionState::Stopped
            | OrchestrationSessionState::Failed
            | OrchestrationSessionState::Invalidated
    ) && incoming.state.is_active()
    {
        return false;
    }
    if orchestration_session_state_rank(&incoming.state)
        < orchestration_session_state_rank(&existing.state)
    {
        return false;
    }

    match orchestration_session_freshness(incoming).cmp(&orchestration_session_freshness(existing))
    {
        Ordering::Greater => true,
        Ordering::Less => false,
        Ordering::Equal => {
            (
                incoming.last_active_at,
                incoming.closed_at.is_some(),
                orchestration_session_state_rank(&incoming.state),
            ) >= (
                existing.last_active_at,
                existing.closed_at.is_some(),
                orchestration_session_state_rank(&existing.state),
            )
        }
    }
}

#[cfg(unix)]
fn owner_process_is_alive(participant: &AgentRuntimeParticipantRecord) -> bool {
    owner_pid_is_alive(participant.internal.shell_owner_pid)
}

#[cfg(unix)]
fn owner_pid_is_alive(pid: u32) -> bool {
    let pid = pid as libc::pid_t;
    if pid <= 0 {
        return false;
    }

    let rc = unsafe { libc::kill(pid, 0) };
    if rc == 0 {
        return true;
    }

    matches!(io::Error::last_os_error().raw_os_error(), Some(libc::EPERM))
}

#[cfg(not(unix))]
fn owner_process_is_alive(participant: &AgentRuntimeParticipantRecord) -> bool {
    owner_pid_is_alive(participant.internal.shell_owner_pid)
}

#[cfg(not(unix))]
fn owner_pid_is_alive(pid: u32) -> bool {
    pid == std::process::id()
}

fn read_json_if_exists<T>(path: &Path) -> Result<Option<T>>
where
    T: serde::de::DeserializeOwned,
{
    match fs::read_to_string(path) {
        Ok(raw) => {
            Ok(Some(serde_json::from_str(&raw).with_context(|| {
                format!("failed to parse {}", path.display())
            })?))
        }
        Err(err) if err.kind() == io::ErrorKind::NotFound => Ok(None),
        Err(err) => Err(err).with_context(|| format!("failed to read {}", path.display())),
    }
}

fn safe_metadata(path: &Path) -> Result<Option<fs::Metadata>> {
    match fs::symlink_metadata(path) {
        Ok(metadata) if metadata.file_type().is_symlink() => Ok(None),
        Ok(metadata) => Ok(Some(metadata)),
        Err(err) if err.kind() == io::ErrorKind::NotFound => Ok(None),
        Err(err) => Err(err).with_context(|| format!("failed to stat {}", path.display())),
    }
}

fn safe_read_dir(path: &Path) -> Result<Option<fs::ReadDir>> {
    let Some(metadata) = safe_metadata(path)? else {
        return Ok(None);
    };
    if !metadata.is_dir() {
        return Ok(None);
    }

    let entries =
        fs::read_dir(path).with_context(|| format!("failed to read {}", path.display()))?;
    Ok(Some(entries))
}

fn read_regular_json_if_exists<T>(path: &Path) -> Result<Option<T>>
where
    T: serde::de::DeserializeOwned,
{
    let Some(metadata) = safe_metadata(path)? else {
        return Ok(None);
    };
    if !metadata.is_file() {
        return Ok(None);
    }

    read_json_if_exists(path)
}

fn synthesize_session_record(
    orchestration_session_id: &str,
    participants: &[AgentRuntimeParticipantRecord],
) -> OrchestrationSessionRecord {
    let template = participants
        .iter()
        .find(|participant| {
            participant.handle.role == ORCHESTRATOR_ROLE
                && participant.handle.execution.scope == AgentExecutionScope::Host
        })
        .or_else(|| participants.first())
        .expect("synthetic session record requires at least one participant");

    let mut session = OrchestrationSessionRecord::new(
        orchestration_session_id.to_string(),
        "<unknown-trace-session>".to_string(),
        "<unknown-workspace-root>".to_string(),
        template,
        None,
    );
    session.opened_at = participants
        .iter()
        .map(|participant| participant.handle.opened_at)
        .min()
        .unwrap_or(session.opened_at);
    session.last_active_at = participants
        .iter()
        .map(AgentRuntimeParticipantRecord::last_status_at)
        .max()
        .unwrap_or(session.last_active_at);

    if let Some(orchestrator) = participants.iter().find(|participant| {
        participant.handle.role == ORCHESTRATOR_ROLE
            && participant.handle.execution.scope == AgentExecutionScope::Host
    }) {
        session.orchestrator_agent_id = orchestrator.handle.agent_id.clone();
        session.orchestrator_backend_id = orchestrator.handle.backend_id.clone();
        session.orchestrator_protocol = orchestrator.handle.protocol.clone();
    }
    session.active_session_handle_id = participants
        .iter()
        .find(|participant| {
            participant.handle.role == ORCHESTRATOR_ROLE
                && participant.handle.execution.scope == AgentExecutionScope::Host
                && participant.is_authoritative_live()
                && owner_process_is_alive(participant)
        })
        .map(|participant| participant.handle.participant_id.clone());
    session.latest_run_id = participants
        .iter()
        .filter_map(|participant| {
            participant
                .internal
                .latest_run_id
                .as_ref()
                .map(|run_id| (participant.last_status_at(), run_id.clone()))
        })
        .max_by(|left, right| left.0.cmp(&right.0))
        .map(|(_, run_id)| run_id);
    session.state = if session.active_session_handle_id.is_some() {
        OrchestrationSessionState::Active
    } else {
        OrchestrationSessionState::Allocating
    };
    session
}

fn session_requires_linux_first_public_control_posture(record: &AgentRuntimeSessionRecord) -> bool {
    record.session.has_world_binding()
        || record.participants.iter().any(|participant| {
            participant.handle.role == MEMBER_ROLE
                && participant.handle.execution.scope == AgentExecutionScope::World
                && participant.handle.state.is_live()
        })
}

#[derive(Clone, Debug)]
#[allow(dead_code)]
struct PublicTurnTargetCandidate {
    participant: AgentRuntimeParticipantRecord,
    kind: PublicTurnTargetKind,
}

#[allow(dead_code)]
fn public_turn_session_mentions_backend(
    record: &AgentRuntimeSessionRecord,
    backend_id: &str,
) -> bool {
    record.session.orchestrator_backend_id == backend_id
        || record
            .participants
            .iter()
            .any(|participant| participant.handle.backend_id == backend_id)
}

#[allow(dead_code)]
fn public_turn_authoritative_candidates(
    record: &AgentRuntimeSessionRecord,
    authoritative_participant: &AgentRuntimeParticipantRecord,
    backend_id: &str,
) -> Vec<PublicTurnTargetCandidate> {
    let mut candidates = Vec::new();
    if authoritative_participant.handle.backend_id == backend_id
        && authoritative_participant.matches_public_parent_linkage(&record.session)
    {
        candidates.push(PublicTurnTargetCandidate {
            participant: authoritative_participant.clone(),
            kind: PublicTurnTargetKind::Host,
        });
    }

    if record.session.world_id.is_none() || record.session.world_generation.is_none() {
        return candidates;
    }

    candidates.extend(
        record
            .participants
            .iter()
            .filter(|participant| {
                participant.handle.backend_id == backend_id
                    && participant.matches_authoritative_parent_world_binding(&record.session)
                    && validate_retained_worker_authoritative_lineage(
                        record,
                        authoritative_participant,
                        participant,
                    )
                    .is_ok()
            })
            .cloned()
            .map(|participant| PublicTurnTargetCandidate {
                participant,
                kind: PublicTurnTargetKind::World,
            }),
    );

    candidates
}

fn resolve_authoritative_session_control(
    record: &AgentRuntimeSessionRecord,
    orchestration_session_id: &str,
) -> Result<ResolvedAuthoritativeSessionControl> {
    if !record.has_authoritative_parent() {
        anyhow::bail!(
            "missing_active_parent: orchestration session {} is missing authoritative parent metadata",
            orchestration_session_id
        );
    }
    if record.session.state != OrchestrationSessionState::Active {
        anyhow::bail!(
            "missing_active_parent: orchestration session {} is not active",
            orchestration_session_id
        );
    }

    let participant = resolve_authoritative_session_participant(record, orchestration_session_id)?;

    Ok(ResolvedAuthoritativeSessionControl {
        session: record.session.clone(),
        participant: participant.clone(),
        session_posture: classify_public_session_posture(record, &participant),
    })
}

pub(crate) fn validate_retained_worker_authoritative_lineage(
    record: &AgentRuntimeSessionRecord,
    authoritative_participant: &AgentRuntimeParticipantRecord,
    target_participant: &AgentRuntimeParticipantRecord,
) -> Result<()> {
    let authoritative_participant_id = authoritative_participant.participant_id();
    let Some(linked_orchestrator_participant_id) = target_participant
        .handle
        .orchestrator_participant_id
        .as_deref()
    else {
        anyhow::bail!(
            "stale_linkage: orchestration session {} retained worker {} is not linked to authoritative orchestrator {}",
            record.orchestration_session_id(),
            target_participant.participant_id(),
            authoritative_participant_id
        );
    };

    let mut current_participant = authoritative_participant.clone();
    let mut visited_participant_ids = BTreeSet::new();

    loop {
        if !current_participant.matches_public_parent_linkage(&record.session) {
            break;
        }
        let current_participant_id = current_participant.participant_id().to_string();
        if !visited_participant_ids.insert(current_participant_id.clone()) {
            break;
        }
        if linked_orchestrator_participant_id == current_participant_id {
            return Ok(());
        }

        let Some(predecessor_participant_id) = current_participant
            .handle
            .resumed_from_participant_id
            .clone()
        else {
            break;
        };

        current_participant = record
            .participants
            .iter()
            .find(|participant| {
                participant.participant_id() == predecessor_participant_id
                    && participant.matches_public_parent_linkage(&record.session)
            })
            .cloned()
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "stale_linkage: orchestration session {} retained worker {} is not linked to authoritative orchestrator {}",
                    record.orchestration_session_id(),
                    target_participant.participant_id(),
                    authoritative_participant_id
                )
            })?;
    }

    anyhow::bail!(
        "stale_linkage: orchestration session {} retained worker {} is not linked to authoritative orchestrator {}",
        record.orchestration_session_id(),
        target_participant.participant_id(),
        authoritative_participant_id
    );
}

pub(crate) fn exact_continue_retained_worker_is_routable(
    _session: &OrchestrationSessionRecord,
    _authoritative_participant: &AgentRuntimeParticipantRecord,
    target_participant: &AgentRuntimeParticipantRecord,
) -> bool {
    target_participant.handle.state.is_live()
        && target_participant.internal.terminal_observed_at.is_none()
}

fn resolve_authoritative_session_participant(
    record: &AgentRuntimeSessionRecord,
    orchestration_session_id: &str,
) -> Result<AgentRuntimeParticipantRecord> {
    if !record.has_authoritative_parent() {
        anyhow::bail!(
            "missing_active_parent: orchestration session {} is missing authoritative parent metadata",
            orchestration_session_id
        );
    }

    let active_participant_id = session_authoritative_participant_id(&record.session).ok_or_else(
        || {
            anyhow::anyhow!(
                "stale_linkage: orchestration session {} is missing authoritative orchestrator participant linkage",
                orchestration_session_id
            )
        },
    )?;
    let participant = record
        .participants
        .iter()
        .find(|participant| participant.participant_id() == active_participant_id)
        .cloned()
        .ok_or_else(|| {
            anyhow::anyhow!(
                "stale_linkage: orchestration session {} references missing participant {}",
                orchestration_session_id,
                active_participant_id
            )
        })?;

    if !participant.handle.state.is_live() {
        anyhow::bail!(
            "stale_linkage: orchestration session {} references inactive participant {}",
            orchestration_session_id,
            active_participant_id
        );
    }
    if !participant.matches_public_parent_linkage(&record.session) {
        anyhow::bail!(
            "stale_linkage: orchestration session {} active participant {} does not match exact orchestrator linkage",
            orchestration_session_id,
            active_participant_id
        );
    }

    Ok(participant)
}

fn resolve_authoritative_stop_dispatch_owner_participant(
    record: &AgentRuntimeSessionRecord,
    orchestration_session_id: &str,
) -> Result<AgentRuntimeParticipantRecord> {
    if !record.has_authoritative_parent() {
        anyhow::bail!(
            "missing_active_parent: orchestration session {} is missing authoritative parent metadata",
            orchestration_session_id
        );
    }
    if record.session.state != OrchestrationSessionState::Active {
        anyhow::bail!(
            "missing_active_parent: orchestration session {} is not active",
            orchestration_session_id
        );
    }

    let active_participant_id = record
        .session
        .sanctioned_stop_owner_participant_id()
        .ok_or_else(|| {
            anyhow::anyhow!(
                "stale_linkage: orchestration session {} is missing authoritative orchestrator participant linkage",
                orchestration_session_id
            )
        })?;
    let participant = record
        .participants
        .iter()
        .find(|participant| participant.participant_id() == active_participant_id)
        .cloned()
        .ok_or_else(|| {
            anyhow::anyhow!(
                "stale_linkage: orchestration session {} references missing participant {}",
                orchestration_session_id,
                active_participant_id
            )
        })?;

    if !participant.handle.state.is_live() {
        anyhow::bail!(
            "stale_linkage: orchestration session {} references inactive participant {}",
            orchestration_session_id,
            active_participant_id
        );
    }
    if !participant.matches_public_parent_linkage(&record.session) {
        anyhow::bail!(
            "stale_linkage: orchestration session {} active participant {} does not match exact orchestrator linkage",
            orchestration_session_id,
            active_participant_id
        );
    }

    Ok(participant)
}

fn detached_status_visible_participant(
    record: &AgentRuntimeSessionRecord,
) -> Result<Option<AgentRuntimeParticipantRecord>> {
    let resolved =
        resolve_authoritative_session_control(record, &record.session.orchestration_session_id)?;
    Ok(
        (resolved.session_posture == PublicSessionPosture::DetachedReattachable)
            .then_some(resolved.participant),
    )
}

#[allow(dead_code)]
fn classify_public_session_posture(
    record: &AgentRuntimeSessionRecord,
    participant: &AgentRuntimeParticipantRecord,
) -> PublicSessionPosture {
    let session = &record.session;
    if session.posture == OrchestrationSessionPosture::Terminal || session.state.is_terminal() {
        return PublicSessionPosture::Terminal;
    }
    if session_attached_to_participant(session, participant)
        && participant.attached_client_present()
        && participant.is_authoritative_live()
        && owner_process_is_alive(participant)
    {
        return PublicSessionPosture::Active;
    }
    if valid_detached_host_continuity_posture(session, participant, true).is_some() {
        return PublicSessionPosture::DetachedReattachable;
    }
    if recoverable_stale_host_attachment(record, session, participant, true) {
        return PublicSessionPosture::DetachedReattachable;
    }
    PublicSessionPosture::Terminal
}

fn classify_router_auto_attach_session_readiness(
    record: &AgentRuntimeSessionRecord,
    orchestration_session_id: &str,
) -> RouterAutoAttachSessionReadiness {
    if record.session.state.is_terminal() {
        return RouterAutoAttachSessionReadiness::NoCandidate {
            reason: "terminal_session",
        };
    }
    if record.session.host_attach_contract().is_none() {
        return RouterAutoAttachSessionReadiness::NoCandidate {
            reason: "missing_attach_contract",
        };
    }

    let resolved = match resolve_authoritative_session_control(record, orchestration_session_id) {
        Ok(resolved) => resolved,
        Err(_) => {
            return RouterAutoAttachSessionReadiness::NoCandidate {
                reason: "stale_linkage",
            };
        }
    };

    match resolved.session_posture {
        PublicSessionPosture::DetachedReattachable => RouterAutoAttachSessionReadiness::Eligible,
        PublicSessionPosture::Active => RouterAutoAttachSessionReadiness::NoCandidate {
            reason: "attached_host_present",
        },
        PublicSessionPosture::Terminal => RouterAutoAttachSessionReadiness::NoCandidate {
            reason: "owner_unreachable",
        },
    }
}

// Detached continuity is valid only when persisted session truth and participant truth agree.
pub(crate) fn valid_detached_host_continuity_posture(
    session: &OrchestrationSessionRecord,
    participant: &AgentRuntimeParticipantRecord,
    require_internal_session_id: bool,
) -> Option<OrchestrationSessionPosture> {
    let contract = session.host_attach_contract()?;
    if session.state.is_terminal() || !participant.handle.state.is_live() {
        return None;
    }
    if session.active_participant_id() != Some(participant.participant_id()) {
        return None;
    }
    if !participant.matches_public_parent_linkage(session) {
        return None;
    }
    if session.attached_participant_id().is_some() || participant.attached_client_present() {
        return None;
    }
    if !participant.is_public_attach_continuity_source() {
        return None;
    }
    if !contract.supports_public_attach_continuity() {
        return None;
    }
    if require_internal_session_id && contract.public_attach_continuity_session_id().is_none() {
        return None;
    }

    match session.posture {
        OrchestrationSessionPosture::ParkedResumable if session.pending_inbox_count == 0 => {
            Some(OrchestrationSessionPosture::ParkedResumable)
        }
        OrchestrationSessionPosture::AwaitingAttention if session.pending_inbox_count > 0 => {
            Some(OrchestrationSessionPosture::AwaitingAttention)
        }
        _ => None,
    }
}

fn recoverable_stale_host_attachment(
    record: &AgentRuntimeSessionRecord,
    session: &OrchestrationSessionRecord,
    participant: &AgentRuntimeParticipantRecord,
    require_internal_session_id: bool,
) -> bool {
    let Some(contract) = session.host_attach_contract() else {
        return false;
    };
    if session.state.is_terminal() || !participant.handle.state.is_live() {
        return false;
    }
    if session.posture != OrchestrationSessionPosture::ActiveAttached {
        return false;
    }
    if session.active_participant_id() != Some(participant.participant_id()) {
        return false;
    }
    if !participant.matches_public_parent_linkage(session) {
        return false;
    }
    if !session_attached_to_participant(session, participant)
        || !participant.attached_client_present()
    {
        return false;
    }
    if owner_process_is_alive(participant) || !participant.is_public_attach_continuity_source() {
        return false;
    }
    if !contract.supports_public_attach_continuity() {
        return false;
    }
    if record.participants.iter().any(|candidate| {
        candidate.participant_id() != participant.participant_id()
            && candidate.matches_public_parent_linkage(session)
            && candidate.is_host_orchestrator()
            && candidate.attached_client_present()
            && candidate.is_authoritative_live()
            && owner_process_is_alive(candidate)
    }) {
        return false;
    }

    !require_internal_session_id || contract.public_attach_continuity_session_id().is_some()
}

fn session_authoritative_participant_id(session: &OrchestrationSessionRecord) -> Option<&str> {
    session
        .active_participant_id()
        .or(session.attached_participant_id())
}

fn session_attached_to_participant(
    session: &OrchestrationSessionRecord,
    participant: &AgentRuntimeParticipantRecord,
) -> bool {
    session.attached_participant_id() == Some(participant.participant_id())
}

fn validate_runtime_contract(
    session: &OrchestrationSessionRecord,
    participants: &[AgentRuntimeParticipantRecord],
) -> Result<()> {
    session.validate_persisted_invariants()?;

    let Some(authoritative_participant_id) = session_authoritative_participant_id(session) else {
        if session.state == OrchestrationSessionState::Active
            && session.posture == OrchestrationSessionPosture::BornUnattached
        {
            if born_unattached_status_anchor(&AgentRuntimeSessionRecord {
                session: session.clone(),
                participants: participants.to_vec(),
                warnings: Vec::new(),
                has_authoritative_parent: true,
                complete: false,
            })
            .is_some()
            {
                return Ok(());
            }
            anyhow::bail!(
                "born_unattached session requires authoritative world member launch proof"
            );
        }
        if session.state == OrchestrationSessionState::Active
            && session.posture == OrchestrationSessionPosture::ActiveAttached
        {
            anyhow::bail!("active_attached session is missing authoritative participant linkage");
        }
        return Ok(());
    };

    let participant = participants
        .iter()
        .find(|participant| participant.participant_id() == authoritative_participant_id)
        .ok_or_else(|| {
            anyhow::anyhow!(
                "authoritative participant {} is missing from the session snapshot",
                authoritative_participant_id
            )
        })?;
    if !participant.matches_public_parent_linkage(session) {
        anyhow::bail!(
            "authoritative participant {} no longer matches the session linkage",
            authoritative_participant_id
        );
    }

    match session.posture {
        OrchestrationSessionPosture::ActiveAttached => {
            if !participant.attached_client_present() {
                anyhow::bail!("active_attached session requires attached host participant truth");
            }
        }
        OrchestrationSessionPosture::BornUnattached => {
            anyhow::bail!("born_unattached sessions must not retain an authoritative participant");
        }
        OrchestrationSessionPosture::ParkedResumable => {
            if !participant.is_resume_eligible() {
                anyhow::bail!("parked_resumable session requires resume-eligible host participant");
            }
        }
        OrchestrationSessionPosture::AwaitingAttention => {
            if !participant.is_resume_eligible() {
                anyhow::bail!(
                    "awaiting_attention session requires resume-eligible host participant"
                );
            }
        }
        OrchestrationSessionPosture::Terminal => {}
    }

    Ok(())
}

pub(crate) fn born_unattached_status_anchor(
    record: &AgentRuntimeSessionRecord,
) -> Option<AgentRuntimeParticipantRecord> {
    let session = &record.session;
    if session.posture != OrchestrationSessionPosture::BornUnattached
        || session.state != OrchestrationSessionState::Active
    {
        return None;
    }

    record
        .participants
        .iter()
        .filter(|participant| participant.matches_authoritative_parent_world_binding(session))
        .max_by(|left, right| left.last_status_at().cmp(&right.last_status_at()))
        .cloned()
}

#[cfg(test)]
mod tests {
    #[cfg(unix)]
    use std::os::unix::fs::PermissionsExt;
    use std::path::PathBuf;
    use std::sync::{Arc, Barrier};

    use super::*;
    use crate::execution::agent_runtime::{
        host_inbox::HostInboxMaterializationState,
        mapping::AgentRuntimeBackendKind,
        session::{AgentRuntimeForkParticipantInit, AgentRuntimeSessionState},
        validator::RuntimeSelectionDescriptor,
        OrchestrationObligationAttachState, OrchestrationObligationKind,
        OrchestrationObligationRecord, OrchestrationObligationReviewState,
        OrchestrationObligationState,
    };
    use serde_json::{json, Value};

    fn descriptor(agent_id: &str, scope: AgentExecutionScope) -> RuntimeSelectionDescriptor {
        RuntimeSelectionDescriptor {
            agent_id: agent_id.to_string(),
            backend_id: format!("cli:{agent_id}"),
            backend_kind: AgentRuntimeBackendKind::Codex,
            protocol: "substrate.agent.session".to_string(),
            execution_scope: scope,
            binary_path: PathBuf::from("/usr/bin/codex"),
        }
    }

    fn set_live(participant: &mut AgentRuntimeParticipantRecord) {
        participant.transition_state(AgentRuntimeSessionState::Ready);
        participant.mark_runtime_ownership_retained();
        participant.set_uaa_session_id("uaa_session");
    }

    fn live_orchestrator(
        agent_id: &str,
        orchestration_session_id: &str,
        participant_id: &str,
    ) -> AgentRuntimeParticipantRecord {
        let mut participant = AgentRuntimeParticipantRecord::new_orchestrator_participant(
            &descriptor(agent_id, AgentExecutionScope::Host),
            orchestration_session_id.to_string(),
            participant_id.to_string(),
            format!("lease_{participant_id}"),
        )
        .expect("orchestrator participant");
        set_live(&mut participant);
        participant
    }

    fn detached_orchestrator(
        agent_id: &str,
        orchestration_session_id: &str,
        participant_id: &str,
    ) -> AgentRuntimeParticipantRecord {
        let mut participant = AgentRuntimeParticipantRecord::new_orchestrator_participant(
            &descriptor(agent_id, AgentExecutionScope::Host),
            orchestration_session_id.to_string(),
            participant_id.to_string(),
            format!("lease_{participant_id}"),
        )
        .expect("orchestrator participant");
        participant.transition_state(AgentRuntimeSessionState::Ready);
        participant.set_uaa_session_id("uaa_session");
        participant.mark_client_detached("owner detached cleanly");
        participant
    }

    fn live_member(
        agent_id: &str,
        orchestration_session_id: &str,
        participant_id: &str,
        orchestrator_participant_id: &str,
    ) -> AgentRuntimeParticipantRecord {
        let mut participant = AgentRuntimeParticipantRecord::new_member_participant(
            &descriptor(agent_id, AgentExecutionScope::World),
            orchestration_session_id.to_string(),
            participant_id.to_string(),
            orchestrator_participant_id.to_string(),
            None,
            Some(super::super::session::AgentRuntimeParticipantWorldBinding {
                world_id: "world-17".to_string(),
                world_generation: 2,
            }),
            format!("lease_{participant_id}"),
        )
        .expect("member participant");
        set_live(&mut participant);
        participant
    }

    fn active_parent(participant: &AgentRuntimeParticipantRecord) -> OrchestrationSessionRecord {
        let mut parent = OrchestrationSessionRecord::new(
            participant.handle.orchestration_session_id.clone(),
            "trace_session".to_string(),
            "/workspace".to_string(),
            participant,
            HostAttachContract::from_manifest_for_test(participant),
        );
        parent.transition_state(OrchestrationSessionState::Active);
        parent.bind_active_session_handle(participant.handle.participant_id.clone());
        parent
    }

    fn parked_parent(participant: &AgentRuntimeParticipantRecord) -> OrchestrationSessionRecord {
        let mut parent = active_parent(participant);
        parent.mark_parked_resumable("owner detached cleanly");
        parent
    }

    fn pending_inbox_item(
        orchestration_session_id: &str,
        item_id: &str,
        kind: DurableInboxItemKind,
    ) -> DurableInboxItemRecord {
        DurableInboxItemRecord::new(
            orchestration_session_id.to_string(),
            item_id.to_string(),
            kind,
            Some(format!("message for {item_id}")),
        )
    }

    fn pending_obligation(
        orchestration_session_id: &str,
        obligation_id: &str,
        kind: OrchestrationObligationKind,
    ) -> OrchestrationObligationRecord {
        let mut obligation = OrchestrationObligationRecord::new(
            orchestration_session_id.to_string(),
            obligation_id.to_string(),
            kind,
            format!("summary for {obligation_id}"),
        );
        obligation.attention_required = true;
        obligation.attach_state = OrchestrationObligationAttachState::Eligible;
        obligation
    }

    fn pending_continue_approval_obligation(
        orchestration_session_id: &str,
        obligation_id: &str,
        source_participant_id: &str,
    ) -> OrchestrationObligationRecord {
        let mut obligation = pending_obligation(
            orchestration_session_id,
            obligation_id,
            OrchestrationObligationKind::ApprovalRequired,
        );
        obligation.source_participant_id = Some(source_participant_id.to_string());
        obligation.target_backend_id = Some("cli:codex_world".to_string());
        obligation.world_id = Some("world-17".to_string());
        obligation.world_generation = Some(2);
        obligation.payload = Some(json!({
            "event_class": "approval_request",
            "request_id": format!("req_{obligation_id}"),
        }));
        obligation
    }

    fn pending_continue_follow_up_obligation(
        orchestration_session_id: &str,
        obligation_id: &str,
        source_participant_id: &str,
    ) -> OrchestrationObligationRecord {
        let mut obligation = pending_obligation(
            orchestration_session_id,
            obligation_id,
            OrchestrationObligationKind::FollowUpRequired,
        );
        obligation.source_participant_id = Some(source_participant_id.to_string());
        obligation.target_backend_id = Some("cli:codex_world".to_string());
        obligation.world_id = Some("world-17".to_string());
        obligation.world_generation = Some(2);
        obligation.payload = Some(json!({
            "event_class": "follow_up_question",
            "request_id": format!("req_{obligation_id}"),
        }));
        obligation
    }

    fn pending_host_inbox_record(
        orchestration_session_id: &str,
        record_id: &str,
        kind: OrchestrationObligationKind,
    ) -> HostInboxRecord {
        let mut record = HostInboxRecord::new(
            orchestration_session_id.to_string(),
            record_id.to_string(),
            kind,
            format!("summary for {record_id}"),
            "host-local",
            "remote_router",
            format!("ingress_{record_id}"),
        );
        record.origin_host_id = Some("host-origin".to_string());
        record
    }

    fn write_legacy_handle_file(
        store: &AgentRuntimeStateStore,
        participant_id: &str,
        agent_id: &str,
        orchestration_session_id: &str,
        ownership_valid: bool,
        extras: Option<Value>,
    ) {
        fs::create_dir_all(store.handles_dir()).expect("create handles dir");
        let mut payload = json!({
            "session_handle_id": participant_id,
            "orchestration_session_id": orchestration_session_id,
            "agent_id": agent_id,
            "backend_id": format!("cli:{agent_id}"),
            "role": "orchestrator",
            "protocol": "substrate.agent.session",
            "execution": { "scope": "host" },
            "state": "ready",
            "opened_at": "2026-04-24T18:30:00Z",
            "last_transition_at": "2026-04-24T18:30:00Z",
            "parent_session_handle_id": null,
            "resumed_from_session_handle_id": null,
            "internal": {
                "resolved_agent_kind": "codex",
                "resolved_binary_path": "/usr/bin/codex",
                "shell_owner_pid": std::process::id(),
                "lease_token": format!("lease_{participant_id}"),
                "uaa_session_id": "uaa_session",
                "cancel_supported": true,
                "control_owner_retained": ownership_valid,
                "event_stream_active": ownership_valid,
                "completion_observer_retained": ownership_valid,
                "ownership_mode": "attached_control",
                "ownership_valid": ownership_valid,
                "last_heartbeat_at": "2026-04-24T18:30:00Z"
            }
        });
        if let Some(extras) = extras {
            merge_json(&mut payload, extras);
        }
        fs::write(
            store.handles_dir().join(format!("{participant_id}.json")),
            serde_json::to_vec_pretty(&payload).expect("serialize legacy handle"),
        )
        .expect("write legacy handle");
    }

    fn write_flat_session_file(
        store: &AgentRuntimeStateStore,
        session: &OrchestrationSessionRecord,
    ) {
        fs::create_dir_all(store.sessions_dir()).expect("create sessions dir");
        fs::write(
            store.orchestration_session_path(&session.orchestration_session_id),
            serde_json::to_vec_pretty(session).expect("serialize session"),
        )
        .expect("write flat session");
    }

    fn write_flat_participant_file(
        store: &AgentRuntimeStateStore,
        participant: &AgentRuntimeParticipantRecord,
    ) {
        fs::create_dir_all(store.participants_dir()).expect("create participants dir");
        fs::write(
            store.participant_path(&participant.handle.participant_id),
            serde_json::to_vec_pretty(participant).expect("serialize flat participant"),
        )
        .expect("write flat participant");
    }

    fn write_canonical_session_file(
        store: &AgentRuntimeStateStore,
        session: &OrchestrationSessionRecord,
    ) {
        fs::create_dir_all(store.canonical_session_dir(&session.orchestration_session_id))
            .expect("create canonical session dir");
        fs::write(
            store.canonical_session_path(&session.orchestration_session_id),
            serde_json::to_vec_pretty(session).expect("serialize canonical session"),
        )
        .expect("write canonical session");
    }

    fn write_canonical_participant_file(
        store: &AgentRuntimeStateStore,
        participant: &AgentRuntimeParticipantRecord,
    ) {
        fs::create_dir_all(
            store.canonical_participants_dir(&participant.handle.orchestration_session_id),
        )
        .expect("create canonical participants dir");
        fs::write(
            store.canonical_participant_path(
                &participant.handle.orchestration_session_id,
                &participant.handle.participant_id,
            ),
            serde_json::to_vec_pretty(participant).expect("serialize canonical participant"),
        )
        .expect("write canonical participant");
    }

    fn merge_json(target: &mut Value, extra: Value) {
        match (target, extra) {
            (Value::Object(target), Value::Object(extra)) => {
                for (key, value) in extra {
                    merge_json(target.entry(key).or_insert(Value::Null), value);
                }
            }
            (target, extra) => *target = extra,
        }
    }

    fn with_store(test: impl FnOnce(&AgentRuntimeStateStore)) {
        let safe_parent = std::env::var_os("XDG_RUNTIME_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|| {
                PathBuf::from(std::env::var_os("HOME").expect("tests require HOME")).join(".cache")
            });
        fs::create_dir_all(&safe_parent).expect("create safe StateStore test parent");
        let temp = tempfile::tempdir_in(safe_parent).expect("safe StateStore tempdir");
        fs::set_permissions(temp.path(), fs::Permissions::from_mode(0o700))
            .expect("secure StateStore test root");
        std::env::set_var("SUBSTRATE_HOME", temp.path());
        std::env::set_var(
            SHARED_WORLD_METADATA_ROOT_TEST_ENV,
            temp.path().join("shared-worlds"),
        );
        let store = AgentRuntimeStateStore::new().expect("state store");
        test(&store);
        std::env::remove_var(SHARED_WORLD_METADATA_ROOT_TEST_ENV);
        std::env::remove_var("SUBSTRATE_HOME");
    }

    #[cfg(any(target_os = "linux", test))]
    fn write_shared_world_metadata_for_test(
        world_id: &str,
        orchestration_session_id: &str,
        world_generation: u64,
        binding_state: SharedWorldBindingState,
    ) -> PathBuf {
        let metadata_dir = shared_world_metadata_root().join(world_id);
        fs::create_dir_all(&metadata_dir).expect("shared world metadata dir");
        let metadata_path = metadata_dir.join(SHARED_WORLD_METADATA_FILE);
        fs::write(
            &metadata_path,
            serde_json::to_vec_pretty(&serde_json::json!({
                "world_id": world_id,
                "project_dir": "/tmp",
                "isolate_network": true,
                "always_isolate": true,
                "allowed_domains": [],
                "cgroup_path": "/tmp",
                "started_at_unix_millis": 0,
                "owner_mode": "shared_orchestration",
                "orchestration_session_id": orchestration_session_id,
                "world_generation": world_generation,
                "binding_state": binding_state,
            }))
            .expect("serialize shared world metadata"),
        )
        .expect("write shared world metadata");
        metadata_dir
    }

    #[cfg(any(target_os = "linux", test))]
    #[test]
    #[serial_test::serial]
    fn active_ephemeral_world_task_registry_round_trips_live_task_identity() {
        with_store(|store| {
            let record = ActiveEphemeralWorldTaskRecord {
                orchestration_session_id: "sess_active".to_string(),
                task_run_id: "task-run-47".to_string(),
                caller_participant_id: "orch_active".to_string(),
                target_backend_id: "cli:codex_world".to_string(),
                world_id: "world-17".to_string(),
                world_generation: 2,
            };
            let path = store.canonical_active_ephemeral_task_path("sess_active", "task-run-47");

            let guard = store
                .register_active_ephemeral_world_task(record.clone())
                .expect("register active ephemeral task");

            assert!(path.exists(), "active task registry path must exist");
            assert_eq!(
                store
                    .load_active_ephemeral_world_task("sess_active", "task-run-47")
                    .expect("load active ephemeral task"),
                Some(record.clone())
            );
            assert_eq!(
                store
                    .list_active_ephemeral_world_tasks("sess_active")
                    .expect("list active ephemeral tasks"),
                vec![record]
            );

            drop(guard);

            assert!(
                !path.exists(),
                "dropping the live-task guard must tear down active-task routability"
            );
            assert_eq!(
                store
                    .load_active_ephemeral_world_task("sess_active", "task-run-47")
                    .expect("load active ephemeral task after teardown"),
                None
            );
        });
    }

    #[cfg(any(target_os = "linux", test))]
    #[test]
    #[serial_test::serial]
    fn active_ephemeral_world_task_registry_allows_duplicate_task_run_id_across_sessions() {
        with_store(|store| {
            let record_a = ActiveEphemeralWorldTaskRecord {
                orchestration_session_id: "sess_active_a".to_string(),
                task_run_id: "task-run-duplicate".to_string(),
                caller_participant_id: "orch_a".to_string(),
                target_backend_id: "cli:codex_world".to_string(),
                world_id: "world-17".to_string(),
                world_generation: 2,
            };
            let record_b = ActiveEphemeralWorldTaskRecord {
                orchestration_session_id: "sess_active_b".to_string(),
                task_run_id: "task-run-duplicate".to_string(),
                caller_participant_id: "orch_b".to_string(),
                target_backend_id: "cli:other_world".to_string(),
                world_id: "world-18".to_string(),
                world_generation: 3,
            };

            let guard_a = store
                .register_active_ephemeral_world_task(record_a.clone())
                .expect("register initial active ephemeral task");
            let guard_b = store
                .register_active_ephemeral_world_task(record_b.clone())
                .expect("duplicate task ids across sessions should stay routable");

            assert_eq!(
                store
                    .load_active_ephemeral_world_task("sess_active_a", "task-run-duplicate")
                    .expect("load active task for first session"),
                Some(record_a)
            );
            assert_eq!(
                store
                    .load_active_ephemeral_world_task("sess_active_b", "task-run-duplicate")
                    .expect("load active task for second session"),
                Some(record_b)
            );

            drop(guard_b);
            drop(guard_a);
        });
    }

    #[cfg(any(target_os = "linux", test))]
    #[test]
    #[serial_test::serial]
    fn active_ephemeral_world_task_registry_rejects_duplicate_task_run_id_within_session() {
        with_store(|store| {
            let _guard = store
                .register_active_ephemeral_world_task(ActiveEphemeralWorldTaskRecord {
                    orchestration_session_id: "sess_active".to_string(),
                    task_run_id: "task-run-duplicate".to_string(),
                    caller_participant_id: "orch_a".to_string(),
                    target_backend_id: "cli:codex_world".to_string(),
                    world_id: "world-17".to_string(),
                    world_generation: 2,
                })
                .expect("register initial active ephemeral task");

            let err = store
                .register_active_ephemeral_world_task(ActiveEphemeralWorldTaskRecord {
                    orchestration_session_id: "sess_active".to_string(),
                    task_run_id: "task-run-duplicate".to_string(),
                    caller_participant_id: "orch_b".to_string(),
                    target_backend_id: "cli:codex_world".to_string(),
                    world_id: "world-18".to_string(),
                    world_generation: 3,
                })
                .expect_err("same-session duplicate active task ids must fail closed");

            assert_eq!(
                err.to_string(),
                "duplicate_active_task_run_id: active ephemeral task task-run-duplicate is already registered"
            );
        });
    }

    #[cfg(any(target_os = "linux", test))]
    #[test]
    #[serial_test::serial]
    fn load_active_ephemeral_world_task_maps_not_found_to_none() {
        with_store(|store| {
            let record = ActiveEphemeralWorldTaskRecord {
                orchestration_session_id: "sess_active".to_string(),
                task_run_id: "task-run-not-found".to_string(),
                caller_participant_id: "orch_active".to_string(),
                target_backend_id: "cli:codex_world".to_string(),
                world_id: "world-17".to_string(),
                world_generation: 2,
            };
            let path =
                store.canonical_active_ephemeral_task_path("sess_active", "task-run-not-found");

            let guard = store
                .register_active_ephemeral_world_task(record)
                .expect("register active ephemeral task");
            drop(guard);

            assert!(
                !path.exists(),
                "dropping the live-task guard must remove the registry file"
            );
            assert_eq!(
                store
                    .load_active_ephemeral_world_task("sess_active", "task-run-not-found")
                    .expect("load active ephemeral task after teardown"),
                None
            );
        });
    }

    #[test]
    #[serial_test::serial]
    fn participants_write_load_roundtrip() {
        with_store(|store| {
            let participant = live_orchestrator("codex", "sess_roundtrip", "ash_roundtrip");
            store
                .persist_participant(&participant)
                .expect("persist participant");

            let loaded = store
                .load_participant("ash_roundtrip")
                .expect("load participant")
                .expect("participant should exist");
            assert_eq!(loaded, participant);
            assert!(store.participant_path("ash_roundtrip").exists());
        });
    }

    #[test]
    #[serial_test::serial]
    fn legacy_session_and_participant_writers_refuse_after_authority_activation() {
        let safe_parent = std::env::var_os("XDG_RUNTIME_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|| {
                PathBuf::from(std::env::var_os("HOME").expect("tests require HOME")).join(".cache")
            });
        fs::create_dir_all(&safe_parent).expect("create safe StateStore test parent");
        let temp = tempfile::tempdir_in(safe_parent).expect("safe StateStore tempdir");
        fs::set_permissions(temp.path(), fs::Permissions::from_mode(0o700))
            .expect("secure StateStore test root");
        let store = AgentRuntimeStateStore {
            substrate_home: temp.path().to_path_buf(),
        };
        {
            let authority_root =
                crate::execution::agent_runtime::host_session_authority::store::bootstrap(
                    &store.substrate_home,
                )
                .expect("activate authority store");
            let root_path = store.substrate_home.join("authority-v1/state-root-v1.json");
            let root_bytes = fs::read(&root_path).expect("read activated root");
            let recognized_temp = store
                .substrate_home
                .join("authority-v1/tmp/root--r2--22222222222222222222222222222222.tmp");
            let recognized_temp_bytes = b"post-activation recognized temp";
            fs::write(&recognized_temp, recognized_temp_bytes)
                .expect("write post-activation recognized temp");
            fs::set_permissions(&recognized_temp, fs::Permissions::from_mode(0o600))
                .expect("secure post-activation recognized temp");
            let participant =
                live_orchestrator("codex", "sess_writer_exclusion", "ash_writer_exclusion");
            let session = active_parent(&participant);
            let active_task = ActiveEphemeralWorldTaskRecord {
                orchestration_session_id: "sess_writer_exclusion".into(),
                task_run_id: "task_writer_exclusion".into(),
                caller_participant_id: "ash_writer_exclusion".into(),
                target_backend_id: "cli:codex_world".into(),
                world_id: "world-writer-exclusion".into(),
                world_generation: 1,
            };
            let inbox_item = DurableInboxItemRecord::new(
                "sess_writer_exclusion",
                "item_writer_exclusion",
                DurableInboxItemKind::ApprovalRequired,
                None,
            );
            let obligation = pending_obligation(
                "sess_writer_exclusion",
                "obligation_writer_exclusion",
                OrchestrationObligationKind::ApprovalRequired,
            );
            let host_inbox = pending_host_inbox_record(
                "sess_writer_exclusion",
                "host_writer_exclusion",
                OrchestrationObligationKind::ApprovalRequired,
            );
            let host_inbox_path = store
                .host_inbox_record_path(&host_inbox.record_id)
                .expect("host inbox path");
            fs::create_dir_all(store.host_inbox_dir()).expect("create host inbox directory");
            write_atomic_json(&host_inbox_path, &host_inbox).expect("seed host inbox record");
            let host_inbox_bytes = fs::read(&host_inbox_path).expect("read host inbox record");

            assert!(store.persist_participant(&participant).is_err());
            assert!(store.persist_orchestration_session(&session).is_err());
            assert!(store
                .register_active_ephemeral_world_task(active_task)
                .is_err());
            assert!(store
                .remove_active_ephemeral_world_task(
                    "sess_writer_exclusion",
                    "task_writer_exclusion",
                )
                .is_err());
            assert!(store.persist_inbox_item(&inbox_item).is_err());
            assert!(store.persist_obligation(&obligation).is_err());
            assert!(store
                .claim_session_auto_attach("sess_writer_exclusion", "router-writer-exclusion")
                .is_err());
            assert!(store
                .settle_session_auto_attach_after_attach_restored(
                    "sess_writer_exclusion",
                    "writer exclusion",
                )
                .is_err());
            assert!(store
                .release_session_auto_attach_claim(
                    "sess_writer_exclusion",
                    "obligation_writer_exclusion",
                    "router-writer-exclusion",
                )
                .is_err());
            assert!(store
                .settle_exact_session_auto_attach_obligation_failed_closed(
                    "sess_writer_exclusion",
                    "obligation_writer_exclusion",
                    "writer exclusion",
                )
                .is_err());
            assert!(store
                .settle_session_auto_attach_failed_closed(
                    "sess_writer_exclusion",
                    "writer exclusion",
                )
                .is_err());
            let mut binding_session = session.clone();
            assert!(store
                .set_orchestration_session_world_binding(
                    &mut binding_session,
                    "world-writer-exclusion",
                    1,
                )
                .is_err());
            assert_eq!(binding_session, session);
            assert!(store
                .clear_orchestration_session_world_binding(&mut binding_session)
                .is_err());
            assert_eq!(binding_session, session);
            assert!(store
                .materialize_host_inbox_record_for_local_host(&host_inbox.record_id, "host-local",)
                .is_err());
            assert_eq!(
                fs::read(&root_path).expect("reread activated root"),
                root_bytes
            );
            assert_eq!(authority_root.root_revision, 1);
            assert!(!store.participants_dir().exists());
            assert!(!store.sessions_dir().exists());
            assert_eq!(
                fs::read(&host_inbox_path).expect("reread host inbox record"),
                host_inbox_bytes
            );
            assert_eq!(
                fs::read(&recognized_temp).expect("reread post-activation recognized temp"),
                recognized_temp_bytes
            );

            let mut stale_member = live_member(
                "codex_world",
                "sess_writer_exclusion",
                "ash_stale_writer_exclusion",
                "ash_writer_exclusion",
            );
            stale_member.handle.world_generation = Some(1);
            fs::create_dir_all(store.participants_dir()).expect("seed participants directory");
            write_atomic_json(
                &store.participant_path(&stale_member.handle.participant_id),
                &stale_member,
            )
            .expect("seed post-root stale member");
            let stale_bytes = fs::read(store.participant_path(&stale_member.handle.participant_id))
                .expect("read seeded stale member");
            assert!(store
                .invalidate_stale_world_members_for_session("sess_writer_exclusion", 2)
                .is_err());
            assert_eq!(
                fs::read(store.participant_path(&stale_member.handle.participant_id))
                    .expect("reread seeded stale member"),
                stale_bytes
            );
        }
    }

    #[test]
    #[serial_test::serial]
    fn preactivation_state_store_writer_uses_shared_cross_process_root_lock() {
        use std::io::{BufRead as _, Read as _};

        const CHILD_TEST: &str = "execution::agent_runtime::state_store::tests::preactivation_state_store_writer_uses_shared_cross_process_root_lock";
        const CHILD_SENTINEL: &str = "A1_LEGACY_WRITER_CHILD_EXECUTED";
        if let Some(root_path) = std::env::var_os("SUBSTRATE_A1_LEGACY_WRITER_CHILD_ROOT") {
            std::env::set_var("SUBSTRATE_HOME", &root_path);
            let store = AgentRuntimeStateStore::new().expect("child state store");
            println!("{CHILD_SENTINEL}");
            std::io::stdout().flush().expect("flush child sentinel");
            let participant = live_orchestrator(
                "codex",
                "sess_cross_process_writer",
                "ash_cross_process_writer",
            );
            store
                .persist_participant(&participant)
                .expect("child persist participant");
            println!("writer-complete");
            return;
        }

        let safe_parent = std::env::var_os("XDG_RUNTIME_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|| {
                PathBuf::from(std::env::var_os("HOME").expect("tests require HOME")).join(".cache")
            });
        fs::create_dir_all(&safe_parent).expect("create safe StateStore test parent");
        let temp = tempfile::tempdir_in(safe_parent).expect("safe StateStore tempdir");
        fs::set_permissions(temp.path(), fs::Permissions::from_mode(0o700))
            .expect("secure StateStore test root");
        let authority_guard =
            crate::execution::agent_runtime::host_session_authority::store::legacy_writer_guard(
                temp.path(),
            )
            .expect("hold shared legacy-writer lock");
        let mut child = std::process::Command::new(std::env::current_exe().unwrap())
            .args(["--exact", CHILD_TEST, "--nocapture"])
            .env("SUBSTRATE_A1_LEGACY_WRITER_CHILD_ROOT", temp.path())
            .stdout(std::process::Stdio::piped())
            .spawn()
            .expect("spawn legacy writer child");
        let mut stdout = std::io::BufReader::new(child.stdout.take().expect("child stdout"));
        let mut prefix = String::new();
        for _ in 0..32 {
            let mut line = String::new();
            assert_ne!(
                stdout.read_line(&mut line).expect("read child output"),
                0,
                "child exited before sentinel; output: {prefix}"
            );
            prefix.push_str(&line);
            if line.contains(CHILD_SENTINEL) {
                break;
            }
        }
        assert!(prefix.contains(CHILD_SENTINEL), "child output: {prefix}");
        std::thread::sleep(std::time::Duration::from_millis(100));
        assert!(child.try_wait().expect("poll blocked child").is_none());
        assert!(!temp
            .path()
            .join("run/agent-hub/participants/ash_cross_process_writer.json")
            .exists());

        drop(authority_guard);
        let mut remainder = String::new();
        stdout
            .read_to_string(&mut remainder)
            .expect("read child completion");
        let status = child.wait().expect("wait for legacy writer child");
        assert!(status.success());
        assert!(remainder.contains("writer-complete"));
        assert!(temp
            .path()
            .join("run/agent-hub/participants/ash_cross_process_writer.json")
            .exists());
    }

    #[cfg(target_os = "linux")]
    #[test]
    #[serial_test::serial]
    fn preactivation_state_store_transaction_rejects_cross_process_root_replacement() {
        use std::io::{BufRead as _, Read as _, Write as _};

        const CHILD_TEST: &str = "execution::agent_runtime::state_store::tests::preactivation_state_store_transaction_rejects_cross_process_root_replacement";
        const CHILD_ROOT_ENV: &str = "SUBSTRATE_A1_REPLACEMENT_CHILD_ROOT";
        const CHILD_SENTINEL: &str = "A1_RETAINED_ROOT_TRANSACTION_ADMITTED";
        const CHILD_COMPLETE: &str = "A1_RETAINED_ROOT_TRANSACTION_REJECTED_REBIND";

        if let Some(root_path) = std::env::var_os(CHILD_ROOT_ENV) {
            let root_path = PathBuf::from(root_path);
            std::env::set_var("SUBSTRATE_HOME", &root_path);
            let expected_root = crate::execution::agent_runtime::host_session_authority::trusted_fs::TrustedAuthorityRoot::open(&root_path)
                .expect("open child expected trusted root")
                .identity()
                .clone();
            let store = AgentRuntimeStateStore::new().expect("child StateStore");
            let attempted =
                live_orchestrator("codex", "sess_rebound_attempt", "ash_rebound_attempt");
            let outcome = store.with_legacy_snapshot_transaction(|transaction| {
                transaction
                    .verify_physical_root(&expected_root)
                    .expect("transaction begins on expected physical root");
                println!("{CHILD_SENTINEL}");
                std::io::stdout().flush().expect("flush admission sentinel");
                let mut release = String::new();
                std::io::stdin()
                    .read_line(&mut release)
                    .expect("wait for replacement handoff");
                assert_eq!(release.trim(), "continue");

                assert!(transaction.verify_physical_root(&expected_root).is_err());
                assert!(AgentRuntimeStateStore::transaction_read_json::<
                    AgentRuntimeParticipantRecord,
                >(
                    transaction,
                    super::super::host_session_authority::store::LegacyStateStoreCollectionV1::Participants,
                    &["replacement-canary.json"],
                )
                .is_err());
                assert!(AgentRuntimeStateStore::transaction_write_json(
                    transaction,
                    super::super::host_session_authority::store::LegacyStateStoreCollectionV1::Participants,
                    &["ash_rebound_attempt.json"],
                    &attempted,
                )
                .is_err());
                assert!(transaction
                    .remove_file(
                        super::super::host_session_authority::store::LegacyStateStoreCollectionV1::Participants,
                        &["replacement-canary.json"],
                    )
                    .is_err());
                Ok(())
            });
            assert!(outcome.is_err(), "rebound transaction fabricated success");
            println!("{CHILD_COMPLETE}");
            return;
        }

        fn tree_snapshot(root: &std::path::Path) -> Vec<(PathBuf, u32, Option<Vec<u8>>)> {
            fn visit(
                root: &std::path::Path,
                directory: &std::path::Path,
                snapshot: &mut Vec<(PathBuf, u32, Option<Vec<u8>>)>,
            ) {
                let mut entries = fs::read_dir(directory)
                    .expect("enumerate proof tree")
                    .map(|entry| entry.expect("read proof tree entry"))
                    .collect::<Vec<_>>();
                entries.sort_by_key(|entry| entry.file_name());
                for entry in entries {
                    let path = entry.path();
                    let relative = path
                        .strip_prefix(root)
                        .expect("proof tree entry remains below root")
                        .to_path_buf();
                    let metadata = fs::symlink_metadata(&path).expect("stat proof tree entry");
                    let mode = metadata.permissions().mode();
                    if metadata.is_dir() {
                        snapshot.push((relative, mode, None));
                        visit(root, &path, snapshot);
                    } else if metadata.is_file() {
                        snapshot.push((
                            relative,
                            mode,
                            Some(fs::read(&path).expect("read proof tree file")),
                        ));
                    } else {
                        panic!("unexpected proof tree entry: {}", path.display());
                    }
                }
            }

            let mut snapshot = Vec::new();
            visit(root, root, &mut snapshot);
            snapshot
        }

        let safe_parent = std::env::var_os("XDG_RUNTIME_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|| {
                PathBuf::from(std::env::var_os("HOME").expect("tests require HOME")).join(".cache")
            });
        fs::create_dir_all(&safe_parent).expect("create safe StateStore test parent");
        let parent = tempfile::tempdir_in(safe_parent).expect("safe StateStore replacement parent");
        fs::set_permissions(parent.path(), fs::Permissions::from_mode(0o700))
            .expect("secure StateStore replacement parent");
        let lexical_root = parent.path().join("bootstrap");
        let replacement_source = parent.path().join("replacement");
        fs::create_dir(&lexical_root).expect("create original bootstrap root");
        fs::create_dir(&replacement_source).expect("create replacement bootstrap root");
        fs::set_permissions(&lexical_root, fs::Permissions::from_mode(0o700))
            .expect("secure original bootstrap root");
        fs::set_permissions(&replacement_source, fs::Permissions::from_mode(0o700))
            .expect("secure replacement bootstrap root");

        for root in [&lexical_root, &replacement_source] {
            crate::execution::agent_runtime::host_session_authority::store::legacy_writer_guard(
                root,
            )
            .expect("establish legacy lock layout")
            .finish()
            .expect("finish legacy lock layout establishment");
        }
        let replacement_store = AgentRuntimeStateStore {
            substrate_home: replacement_source.clone(),
        };
        let replacement_canary =
            live_orchestrator("codex", "sess_replacement_canary", "replacement-canary");
        replacement_store
            .persist_participant(&replacement_canary)
            .expect("seed replacement canary through retained transaction");

        let original_identity = crate::execution::agent_runtime::host_session_authority::trusted_fs::TrustedAuthorityRoot::open(&lexical_root)
            .expect("open original identity")
            .identity()
            .clone();
        let replacement_identity = crate::execution::agent_runtime::host_session_authority::trusted_fs::TrustedAuthorityRoot::open(&replacement_source)
            .expect("open replacement identity")
            .identity()
            .clone();
        assert_ne!(
            original_identity.physical_identity,
            replacement_identity.physical_identity
        );

        let mut child = std::process::Command::new(std::env::current_exe().unwrap())
            .args(["--exact", CHILD_TEST, "--nocapture"])
            .env(CHILD_ROOT_ENV, &lexical_root)
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .spawn()
            .expect("spawn retained-root replacement child");
        let mut child_stdin = child.stdin.take().expect("child stdin");
        let mut child_stdout = std::io::BufReader::new(child.stdout.take().expect("child stdout"));
        let mut prefix = String::new();
        for _ in 0..32 {
            let mut line = String::new();
            assert_ne!(
                child_stdout
                    .read_line(&mut line)
                    .expect("read child admission output"),
                0,
                "child exited before transaction admission; output: {prefix}"
            );
            prefix.push_str(&line);
            if line.contains(CHILD_SENTINEL) {
                break;
            }
        }
        assert!(prefix.contains(CHILD_SENTINEL), "child output: {prefix}");

        let original_before = tree_snapshot(&lexical_root);
        let replacement_before = tree_snapshot(&replacement_source);
        let retained_root = parent.path().join("bootstrap-retained");
        fs::rename(&lexical_root, &retained_root).expect("retain original physical root");
        fs::rename(&replacement_source, &lexical_root).expect("install lexical replacement root");

        let installed_identity = crate::execution::agent_runtime::host_session_authority::trusted_fs::TrustedAuthorityRoot::open(&lexical_root)
            .expect("open installed replacement identity")
            .identity()
            .clone();
        let retained_identity = crate::execution::agent_runtime::host_session_authority::trusted_fs::TrustedAuthorityRoot::open(&retained_root)
            .expect("open retained original identity")
            .identity()
            .clone();
        assert_eq!(
            installed_identity.physical_identity,
            replacement_identity.physical_identity
        );
        assert_eq!(
            retained_identity.physical_identity,
            original_identity.physical_identity
        );

        crate::execution::agent_runtime::host_session_authority::store::legacy_writer_guard(
            &lexical_root,
        )
        .expect("replacement root owns an independent lock")
        .finish()
        .expect("finish independent replacement-root transaction");
        assert_eq!(tree_snapshot(&lexical_root), replacement_before);

        writeln!(child_stdin, "continue").expect("release replacement child");
        drop(child_stdin);
        let mut remainder = String::new();
        child_stdout
            .read_to_string(&mut remainder)
            .expect("read replacement child completion");
        let status = child.wait().expect("wait for replacement child");
        assert!(status.success(), "child output: {prefix}{remainder}");
        assert!(remainder.contains(CHILD_COMPLETE));
        assert!(
            remainder.contains("1 passed"),
            "subprocess filter executed no test: {prefix}{remainder}"
        );

        assert_eq!(tree_snapshot(&retained_root), original_before);
        assert_eq!(tree_snapshot(&lexical_root), replacement_before);
        assert!(!retained_root
            .join("run/agent-hub/participants/ash_rebound_attempt.json")
            .exists());
        assert!(!lexical_root
            .join("run/agent-hub/participants/ash_rebound_attempt.json")
            .exists());

        let restarted_store = AgentRuntimeStateStore {
            substrate_home: lexical_root.clone(),
        };
        let retry = live_orchestrator("codex", "sess_replacement_retry", "ash_replacement_retry");
        restarted_store
            .persist_participant(&retry)
            .expect("fresh replacement-root retry succeeds");
        restarted_store
            .persist_participant(&retry)
            .expect("exact replacement-root retry converges");
        assert_eq!(
            restarted_store
                .load_participant("ash_replacement_retry")
                .expect("load replacement-root retry")
                .expect("replacement-root retry exists"),
            retry
        );
        assert_eq!(
            restarted_store
                .load_participant("replacement-canary")
                .expect("load replacement canary")
                .expect("replacement canary remains"),
            replacement_canary
        );
    }

    #[test]
    fn preactivation_state_store_writer_reconciles_recognized_authority_temp() {
        let safe_parent = std::env::var_os("XDG_RUNTIME_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|| {
                PathBuf::from(std::env::var_os("HOME").expect("tests require HOME")).join(".cache")
            });
        fs::create_dir_all(&safe_parent).expect("create safe StateStore test parent");
        let temp = tempfile::tempdir_in(safe_parent).expect("safe StateStore tempdir");
        fs::set_permissions(temp.path(), fs::Permissions::from_mode(0o700))
            .expect("secure StateStore test root");
        let store = AgentRuntimeStateStore {
            substrate_home: temp.path().to_path_buf(),
        };
        drop(
            crate::execution::agent_runtime::host_session_authority::store::legacy_writer_guard(
                temp.path(),
            )
            .expect("establish preactivation authority lock layout"),
        );
        let recognized_temp = temp
            .path()
            .join("authority-v1/tmp/root--r2--11111111111111111111111111111111.tmp");
        fs::write(&recognized_temp, b"interrupted non-authoritative root temp")
            .expect("write recognized temp");
        fs::set_permissions(&recognized_temp, fs::Permissions::from_mode(0o600))
            .expect("secure recognized temp");

        let participant = live_orchestrator("codex", "sess_reconciled_temp", "ash_reconciled_temp");
        store
            .persist_participant(&participant)
            .expect("preactivation writer should reconcile recognized temp");

        assert!(!recognized_temp.exists());
        assert!(store.participant_path("ash_reconciled_temp").is_file());
    }

    #[test]
    #[serial_test::serial]
    fn dual_read_prefers_participant_file_for_same_identity() {
        with_store(|store| {
            let participant = live_orchestrator("codex", "sess_dual", "ash_dual");
            store
                .persist_participant(&participant)
                .expect("persist participant");
            write_legacy_handle_file(store, "ash_dual", "legacy-agent", "sess_dual", true, None);

            let loaded = store
                .load_participant("ash_dual")
                .expect("load participant")
                .expect("participant should exist");
            assert_eq!(loaded.handle.agent_id, "codex");

            let participants = store.list_participants().expect("list participants");
            assert_eq!(participants.len(), 1);
            assert_eq!(participants[0].handle.agent_id, "codex");
        });
    }

    #[test]
    #[serial_test::serial]
    fn load_session_prefers_canonical_objects_and_keeps_flat_participant_fallbacks() {
        with_store(|store| {
            let canonical_orchestrator =
                live_orchestrator("codex", "sess_precedence", "ash_primary");
            let mut flat_orchestrator =
                live_orchestrator("legacy", "sess_precedence", "ash_primary");
            flat_orchestrator.internal.latest_run_id = Some("run-flat".to_string());
            let flat_member = live_member(
                "claude_code",
                "sess_precedence",
                "ash_member",
                "ash_primary",
            );

            let canonical_parent = active_parent(&canonical_orchestrator);
            let flat_parent = active_parent(&flat_orchestrator);

            store
                .persist_participant(&flat_orchestrator)
                .expect("persist flat orchestrator");
            store
                .persist_participant(&flat_member)
                .expect("persist flat member");
            write_flat_session_file(store, &flat_parent);
            write_canonical_participant_file(store, &canonical_orchestrator);
            write_canonical_session_file(store, &canonical_parent);

            let session = store
                .load_session("sess_precedence")
                .expect("load session")
                .expect("session exists");
            assert!(
                session.warnings.is_empty(),
                "complete record should not warn"
            );
            assert!(
                session.is_complete(),
                "canonical parent plus live participant should be complete"
            );
            assert_eq!(session.session.orchestrator_agent_id, "codex");
            assert_eq!(
                session
                    .participants
                    .iter()
                    .find(|participant| participant.handle.participant_id == "ash_primary")
                    .expect("orchestrator participant")
                    .handle
                    .agent_id,
                "codex"
            );
            assert!(
                session
                    .participants
                    .iter()
                    .any(|participant| participant.handle.participant_id == "ash_member"),
                "canonical parent must not erase flat participant compatibility fallback"
            );
        });
    }

    #[test]
    #[serial_test::serial]
    fn persist_writes_canonical_and_flat_compatibility_layouts() {
        with_store(|store| {
            let participant = live_orchestrator("codex", "sess_dual_write", "ash_dual_write");
            let parent = active_parent(&participant);

            store
                .persist_orchestration_session(&parent)
                .expect("persist parent");
            store
                .persist_participant(&participant)
                .expect("persist participant");

            assert!(
                store.canonical_session_path("sess_dual_write").is_file(),
                "canonical session.json must be written"
            );
            assert!(
                store
                    .orchestration_session_path("sess_dual_write")
                    .is_file(),
                "flat compatibility parent session must remain readable during cutover"
            );
            assert!(
                store
                    .canonical_participant_path("sess_dual_write", "ash_dual_write")
                    .is_file(),
                "canonical participant record must be written"
            );
            assert!(
                store.participant_path("ash_dual_write").is_file(),
                "flat compatibility participant record must remain readable during cutover"
            );
            assert!(
                store
                    .canonical_lease_path("sess_dual_write", "ash_dual_write")
                    .is_file(),
                "canonical lease must be written"
            );
            assert!(
                store.lease_path("ash_dual_write").is_file(),
                "flat compatibility lease must remain readable during cutover"
            );
        });
    }

    #[test]
    #[serial_test::serial]
    fn persist_inbox_item_updates_detached_pending_count_and_writes_canonical_artifact() {
        with_store(|store| {
            let participant =
                detached_orchestrator("codex", "sess_inbox_pending", "ash_inbox_pending");
            let parent = parked_parent(&participant);
            store
                .persist_orchestration_session(&parent)
                .expect("persist parent");
            store
                .persist_participant(&participant)
                .expect("persist participant");

            let item = pending_inbox_item(
                "sess_inbox_pending",
                "item_approval",
                DurableInboxItemKind::ApprovalRequired,
            );
            store.persist_inbox_item(&item).expect("persist inbox item");

            let loaded_session = store
                .load_orchestration_session("sess_inbox_pending")
                .expect("load orchestration session")
                .expect("orchestration session exists");
            assert_eq!(loaded_session.pending_inbox_count, 1);
            assert_eq!(
                loaded_session.posture,
                OrchestrationSessionPosture::AwaitingAttention
            );
            assert!(
                store
                    .canonical_inbox_item_path("sess_inbox_pending", "item_approval")
                    .is_file(),
                "durable inbox artifacts must be stored canonically under sessions/<session>/inbox"
            );

            let loaded_item = store
                .load_inbox_item("sess_inbox_pending", "item_approval")
                .expect("load inbox item")
                .expect("inbox item exists");
            assert_eq!(loaded_item.kind, DurableInboxItemKind::ApprovalRequired);
            assert_eq!(loaded_item.state, DurableInboxItemState::Pending);
            assert!(loaded_item.resolved_at.is_none());
        });
    }

    #[test]
    #[serial_test::serial]
    fn resolving_inbox_item_updates_pending_count_without_deleting_artifact() {
        with_store(|store| {
            let participant =
                detached_orchestrator("codex", "sess_inbox_resolve", "ash_inbox_resolve");
            let parent = parked_parent(&participant);
            store
                .persist_orchestration_session(&parent)
                .expect("persist parent");
            store
                .persist_participant(&participant)
                .expect("persist participant");

            for (item_id, kind) in [
                ("item_completion", DurableInboxItemKind::CompletionNotice),
                ("item_follow_up", DurableInboxItemKind::FollowUpMessage),
                ("item_runtime", DurableInboxItemKind::RuntimeAlert),
            ] {
                store
                    .persist_inbox_item(&pending_inbox_item("sess_inbox_resolve", item_id, kind))
                    .expect("persist inbox item");
            }

            let acknowledged = store
                .acknowledge_inbox_item("sess_inbox_resolve", "item_completion")
                .expect("acknowledge inbox item");
            assert_eq!(acknowledged.state, DurableInboxItemState::Acknowledged);
            assert!(acknowledged.resolved_at.is_some());

            let dismissed = store
                .dismiss_inbox_item("sess_inbox_resolve", "item_follow_up")
                .expect("dismiss inbox item");
            assert_eq!(dismissed.state, DurableInboxItemState::Dismissed);
            assert!(dismissed.resolved_at.is_some());

            let still_pending = store
                .load_inbox_item("sess_inbox_resolve", "item_runtime")
                .expect("load pending inbox item")
                .expect("pending inbox item exists");
            assert_eq!(still_pending.state, DurableInboxItemState::Pending);

            let loaded_session = store
                .load_orchestration_session("sess_inbox_resolve")
                .expect("load orchestration session")
                .expect("orchestration session exists");
            assert_eq!(loaded_session.pending_inbox_count, 1);
            assert_eq!(
                loaded_session.posture,
                OrchestrationSessionPosture::AwaitingAttention
            );

            store
                .dismiss_inbox_item("sess_inbox_resolve", "item_runtime")
                .expect("dismiss final inbox item");
            let settled_session = store
                .load_orchestration_session("sess_inbox_resolve")
                .expect("load settled session")
                .expect("settled session exists");
            assert_eq!(settled_session.pending_inbox_count, 0);
            assert_eq!(
                settled_session.posture,
                OrchestrationSessionPosture::ParkedResumable
            );

            let items = store
                .list_inbox_items("sess_inbox_resolve")
                .expect("list inbox items");
            assert_eq!(items.len(), 3);
            assert!(
                items.iter().all(|item| {
                    store
                        .canonical_inbox_item_path("sess_inbox_resolve", &item.item_id)
                        .is_file()
                }),
                "resolved inbox items must remain durable artifacts instead of being deleted"
            );
        });
    }

    #[test]
    #[serial_test::serial]
    fn born_unattached_session_recovers_original_posture_after_attention_clears() {
        with_store(|store| {
            let participant = live_orchestrator(
                "codex",
                "sess_born_unattached_attention",
                "ash_born_unattached_seed",
            );
            let contract = HostAttachContract::from_manifest_for_test(&participant)
                .expect("host attach contract");
            let session = OrchestrationSessionRecord::new_deferred_host_attach(
                "sess_born_unattached_attention".to_string(),
                "trace_born_unattached_attention".to_string(),
                "/workspace".to_string(),
                contract,
            );
            store
                .persist_orchestration_session(&session)
                .expect("persist deferred host attach session");

            store
                .persist_inbox_item(&pending_inbox_item(
                    "sess_born_unattached_attention",
                    "item_follow_up",
                    DurableInboxItemKind::FollowUpMessage,
                ))
                .expect("persist pending inbox item");

            let pending_session = store
                .load_orchestration_session("sess_born_unattached_attention")
                .expect("load pending born-unattached session")
                .expect("pending session exists");
            assert_eq!(
                pending_session.posture,
                OrchestrationSessionPosture::AwaitingAttention
            );
            assert_eq!(pending_session.pending_inbox_count, 1);

            store
                .dismiss_inbox_item("sess_born_unattached_attention", "item_follow_up")
                .expect("dismiss final born-unattached inbox item");

            let settled_session = store
                .load_orchestration_session("sess_born_unattached_attention")
                .expect("load settled born-unattached session")
                .expect("settled session exists");
            assert_eq!(settled_session.pending_inbox_count, 0);
            assert_eq!(
                settled_session.posture,
                OrchestrationSessionPosture::BornUnattached
            );
        });
    }

    #[test]
    #[serial_test::serial]
    fn persist_obligation_writes_canonical_artifact_and_round_trips() {
        with_store(|store| {
            let participant =
                detached_orchestrator("codex", "sess_obligation_pending", "ash_obligation_pending");
            let parent = parked_parent(&participant);
            store
                .persist_orchestration_session(&parent)
                .expect("persist parent");

            let obligation = pending_obligation(
                "sess_obligation_pending",
                "obl_follow_up",
                OrchestrationObligationKind::FollowUpRequired,
            );
            store
                .persist_obligation(&obligation)
                .expect("persist obligation");

            assert!(
                store
                    .canonical_obligation_path("sess_obligation_pending", "obl_follow_up")
                    .is_file(),
                "orchestration obligations must be stored canonically under sessions/<session>/obligations"
            );

            let loaded = store
                .load_obligation("sess_obligation_pending", "obl_follow_up")
                .expect("load obligation")
                .expect("obligation exists");
            assert_eq!(loaded.kind, OrchestrationObligationKind::FollowUpRequired);
            assert_eq!(loaded.state, OrchestrationObligationState::Pending);
            assert!(loaded.attention_required);

            let obligations = store
                .list_obligations("sess_obligation_pending")
                .expect("list obligations");
            assert_eq!(obligations, vec![loaded]);

            let projected_session = store
                .load_orchestration_session("sess_obligation_pending")
                .expect("load projected session")
                .expect("projected session exists");
            assert_eq!(projected_session.pending_inbox_count, 1);
            assert_eq!(
                projected_session.posture,
                OrchestrationSessionPosture::AwaitingAttention
            );

            let compat_item = store
                .load_inbox_item("sess_obligation_pending", "obl_follow_up")
                .expect("load compatibility inbox item")
                .expect("compatibility item exists");
            assert_eq!(compat_item.kind, DurableInboxItemKind::FollowUpMessage);
            assert_eq!(compat_item.state, DurableInboxItemState::Pending);
        });
    }

    #[test]
    #[serial_test::serial]
    fn persist_obligation_round_trips_ingress_causation_and_host_targeting_fields_when_present() {
        with_store(|store| {
            let participant = detached_orchestrator(
                "codex",
                "sess_obligation_targeted",
                "ash_obligation_targeted",
            );
            let parent = parked_parent(&participant);
            store
                .persist_orchestration_session(&parent)
                .expect("persist parent");

            let mut obligation = pending_obligation(
                "sess_obligation_targeted",
                "obl_targeted",
                OrchestrationObligationKind::FollowUpRequired,
            );
            let ingress_received_at = Utc::now();
            obligation.origin_host_id = Some("host-origin".to_string());
            obligation.target_host_id = Some("host-local".to_string());
            obligation.ingress_source_kind = Some("local_runtime".to_string());
            obligation.ingress_source_id = Some("run-123".to_string());
            obligation.ingress_received_at = Some(ingress_received_at);
            obligation.causation_event_id = Some("event-123".to_string());
            obligation.causation_message_id = Some("message-123".to_string());
            obligation.causation_request_id = Some("request-123".to_string());

            store
                .persist_obligation(&obligation)
                .expect("persist targeted obligation");

            let loaded = store
                .load_obligation("sess_obligation_targeted", "obl_targeted")
                .expect("load targeted obligation")
                .expect("targeted obligation exists");
            assert_eq!(loaded.ingress_source_kind.as_deref(), Some("local_runtime"));
            assert_eq!(loaded.ingress_source_id.as_deref(), Some("run-123"));
            assert_eq!(loaded.ingress_received_at, Some(ingress_received_at));
            assert_eq!(loaded.origin_host_id.as_deref(), Some("host-origin"));
            assert_eq!(loaded.target_host_id.as_deref(), Some("host-local"));
            assert_eq!(loaded.causation_event_id.as_deref(), Some("event-123"));
            assert_eq!(loaded.causation_message_id.as_deref(), Some("message-123"));
            assert_eq!(loaded.causation_request_id.as_deref(), Some("request-123"));
            assert_eq!(loaded, obligation);
        });
    }

    #[test]
    #[serial_test::serial]
    fn load_obligation_defaults_missing_packet_one_fields_for_legacy_artifacts() {
        with_store(|store| {
            let participant = detached_orchestrator(
                "codex",
                "sess_obligation_legacy_targeting",
                "ash_obligation_legacy_targeting",
            );
            let parent = parked_parent(&participant);
            store
                .persist_orchestration_session(&parent)
                .expect("persist parent");

            let obligation = pending_obligation(
                "sess_obligation_legacy_targeting",
                "obl_legacy_targeting",
                OrchestrationObligationKind::Blocked,
            );
            let mut legacy_artifact =
                serde_json::to_value(&obligation).expect("serialize legacy obligation");
            let artifact = legacy_artifact
                .as_object_mut()
                .expect("legacy obligation serializes to an object");
            artifact.remove("ingress_source_kind");
            artifact.remove("ingress_source_id");
            artifact.remove("ingress_received_at");
            artifact.remove("origin_host_id");
            artifact.remove("target_host_id");
            artifact.remove("causation_event_id");
            artifact.remove("causation_message_id");
            artifact.remove("causation_request_id");

            let obligation_path = store.canonical_obligation_path(
                "sess_obligation_legacy_targeting",
                "obl_legacy_targeting",
            );
            write_atomic_json(&obligation_path, &legacy_artifact)
                .expect("write legacy obligation artifact");

            let loaded = store
                .load_obligation("sess_obligation_legacy_targeting", "obl_legacy_targeting")
                .expect("load legacy obligation")
                .expect("legacy obligation exists");
            assert_eq!(loaded.ingress_source_kind, None);
            assert_eq!(loaded.ingress_source_id, None);
            assert_eq!(loaded.ingress_received_at, None);
            assert_eq!(loaded.origin_host_id, None);
            assert_eq!(loaded.target_host_id, None);
            assert_eq!(loaded.causation_event_id, None);
            assert_eq!(loaded.causation_message_id, None);
            assert_eq!(loaded.causation_request_id, None);
            assert_eq!(loaded.kind, OrchestrationObligationKind::Blocked);
            assert_eq!(
                loaded.attach_state,
                OrchestrationObligationAttachState::Eligible
            );
        });
    }

    #[test]
    #[serial_test::serial]
    fn persist_packet_two_worker_request_obligations_preserves_exact_reviewable_fields() {
        with_store(|store| {
            let cases = [
                (
                    "follow_up_question",
                    OrchestrationObligationKind::FollowUpRequired,
                    true,
                    OrchestrationObligationAttachState::Eligible,
                    DurableInboxItemKind::FollowUpMessage,
                    1_u64,
                    OrchestrationSessionPosture::AwaitingAttention,
                ),
                (
                    "approval_request",
                    OrchestrationObligationKind::ApprovalRequired,
                    true,
                    OrchestrationObligationAttachState::Eligible,
                    DurableInboxItemKind::ApprovalRequired,
                    1_u64,
                    OrchestrationSessionPosture::AwaitingAttention,
                ),
                (
                    "fork_request",
                    OrchestrationObligationKind::ForkRequest,
                    true,
                    OrchestrationObligationAttachState::Eligible,
                    DurableInboxItemKind::FollowUpMessage,
                    1_u64,
                    OrchestrationSessionPosture::AwaitingAttention,
                ),
                (
                    "fork_recommendation",
                    OrchestrationObligationKind::ForkRecommendation,
                    false,
                    OrchestrationObligationAttachState::NotEligible,
                    DurableInboxItemKind::FollowUpMessage,
                    0_u64,
                    OrchestrationSessionPosture::ParkedResumable,
                ),
                (
                    "blocked",
                    OrchestrationObligationKind::Blocked,
                    true,
                    OrchestrationObligationAttachState::Eligible,
                    DurableInboxItemKind::RuntimeAlert,
                    1_u64,
                    OrchestrationSessionPosture::AwaitingAttention,
                ),
            ];

            for (
                event_label,
                kind,
                attention_required,
                attach_state,
                expected_compat_kind,
                expected_pending_count,
                expected_posture,
            ) in cases
            {
                let session_id = format!("sess_packet_two_{event_label}");
                let participant_id = format!("ash_packet_two_{event_label}");
                let participant = detached_orchestrator("codex", &session_id, &participant_id);
                let parent = parked_parent(&participant);
                store
                    .persist_orchestration_session(&parent)
                    .expect("persist packet-two parent");

                let mut obligation = OrchestrationObligationRecord::new(
                    session_id.clone(),
                    format!("obl_{event_label}"),
                    kind,
                    format!("summary for {event_label}"),
                );
                obligation.attention_required = attention_required;
                obligation.attach_state = attach_state;
                obligation.source_participant_id = Some(format!("member_{event_label}"));
                obligation.target_backend_id = Some("cli:codex_world".to_string());
                obligation.world_id = Some("world-17".to_string());
                obligation.world_generation = Some(2);
                obligation.payload = Some(json!({
                    "event_class": event_label,
                    "message": format!("payload for {event_label}"),
                    "request_id": format!("req_{event_label}"),
                }));

                store
                    .persist_obligation(&obligation)
                    .expect("persist packet-two obligation");

                let loaded = store
                    .load_obligation(&session_id, &format!("obl_{event_label}"))
                    .expect("load packet-two obligation")
                    .expect("packet-two obligation exists");
                assert_eq!(loaded, obligation);

                let projected_session = store
                    .load_orchestration_session(&session_id)
                    .expect("load packet-two session")
                    .expect("packet-two session exists");
                assert_eq!(
                    projected_session.pending_inbox_count,
                    expected_pending_count
                );
                assert_eq!(projected_session.posture, expected_posture);

                let compat_item = store
                    .load_inbox_item(&session_id, &format!("obl_{event_label}"))
                    .expect("load packet-two compatibility item")
                    .expect("packet-two compatibility item exists");
                assert_eq!(compat_item.kind, expected_compat_kind);
                assert_eq!(
                    compat_item.message.as_deref(),
                    Some(obligation.summary.as_str())
                );
            }
        });
    }

    #[test]
    #[serial_test::serial]
    fn resolving_obligation_updates_detached_projection_and_keeps_compatibility_artifact() {
        with_store(|store| {
            let participant =
                detached_orchestrator("codex", "sess_obligation_resolve", "ash_obligation_resolve");
            let parent = parked_parent(&participant);
            store
                .persist_orchestration_session(&parent)
                .expect("persist parent");

            let mut obligation = pending_obligation(
                "sess_obligation_resolve",
                "obl_attention",
                OrchestrationObligationKind::Blocked,
            );
            store
                .persist_obligation(&obligation)
                .expect("persist pending obligation");

            obligation.state = OrchestrationObligationState::Resolved;
            obligation.review_state = OrchestrationObligationReviewState::Resolved;
            obligation.attention_required = false;
            obligation.resolution_note = Some("host attach recovered".to_string());
            obligation.resolved_at = Some(chrono::Utc::now());
            obligation.updated_at = obligation.resolved_at.expect("resolved_at set");
            store
                .persist_obligation(&obligation)
                .expect("persist resolved obligation");

            let settled_session = store
                .load_orchestration_session("sess_obligation_resolve")
                .expect("load settled session")
                .expect("settled session exists");
            assert_eq!(settled_session.pending_inbox_count, 0);
            assert_eq!(
                settled_session.posture,
                OrchestrationSessionPosture::ParkedResumable
            );

            let compat_item = store
                .load_inbox_item("sess_obligation_resolve", "obl_attention")
                .expect("load compatibility inbox item")
                .expect("compatibility item exists");
            assert_eq!(compat_item.kind, DurableInboxItemKind::RuntimeAlert);
            assert_eq!(compat_item.state, DurableInboxItemState::Dismissed);
            assert_eq!(
                compat_item.message.as_deref(),
                Some("host attach recovered")
            );
            assert!(compat_item.resolved_at.is_some());
        });
    }

    #[test]
    #[serial_test::serial]
    fn persist_obligation_rejects_invalid_records_at_persistence_boundary() {
        with_store(|store| {
            let participant =
                detached_orchestrator("codex", "sess_obligation_invalid", "ash_obligation_invalid");
            let parent = parked_parent(&participant);
            store
                .persist_orchestration_session(&parent)
                .expect("persist parent");

            let mut obligation = pending_obligation(
                "sess_obligation_invalid",
                "obl_invalid",
                OrchestrationObligationKind::Blocked,
            );
            obligation.state = OrchestrationObligationState::Resolved;

            let err = store
                .persist_obligation(&obligation)
                .expect_err("invalid obligation must fail persistence");
            assert!(err
                .to_string()
                .contains("resolved orchestration obligations must include resolved_at"));
        });
    }

    #[test]
    #[serial_test::serial]
    fn persist_obligation_rejects_pending_terminal_review_state_at_persistence_boundary() {
        with_store(|store| {
            let participant = detached_orchestrator(
                "codex",
                "sess_obligation_invalid_review_state",
                "ash_obligation_invalid_review_state",
            );
            let parent = parked_parent(&participant);
            store
                .persist_orchestration_session(&parent)
                .expect("persist parent");

            let mut obligation = pending_obligation(
                "sess_obligation_invalid_review_state",
                "obl_invalid_review_state",
                OrchestrationObligationKind::ApprovalRequired,
            );
            obligation.review_state = OrchestrationObligationReviewState::Dismissed;

            let err = store
                .persist_obligation(&obligation)
                .expect_err("pending terminal review state must fail persistence");
            assert!(err.to_string().contains(
                "pending orchestration obligations cannot advertise terminal review_state"
            ));
        });
    }

    #[test]
    #[serial_test::serial]
    fn claim_session_auto_attach_coalesces_to_one_claim_and_persists_metadata() {
        with_store(|store| {
            let participant =
                detached_orchestrator("codex", "sess_auto_attach_claim", "ash_auto_attach");
            let parent = parked_parent(&participant);
            store
                .persist_orchestration_session(&parent)
                .expect("persist parent");
            store
                .persist_participant(&participant)
                .expect("persist participant");

            let follow_up = pending_obligation(
                "sess_auto_attach_claim",
                "obl_follow_up",
                OrchestrationObligationKind::FollowUpRequired,
            );
            let approval = pending_obligation(
                "sess_auto_attach_claim",
                "obl_approval",
                OrchestrationObligationKind::ApprovalRequired,
            );
            store
                .persist_obligation(&follow_up)
                .expect("persist follow-up obligation");
            store
                .persist_obligation(&approval)
                .expect("persist approval obligation");

            let claim = store
                .claim_session_auto_attach("sess_auto_attach_claim", "router::local")
                .expect("claim auto attach");
            assert_eq!(
                claim,
                SessionAutoAttachClaim::Claimed {
                    obligation_id: "obl_approval".to_string(),
                    attach_claim_owner: "router::local".to_string(),
                }
            );

            let claimed = store
                .load_obligation("sess_auto_attach_claim", "obl_approval")
                .expect("load claimed obligation")
                .expect("claimed obligation exists");
            assert_eq!(
                claimed.attach_state,
                OrchestrationObligationAttachState::Claimed
            );
            assert_eq!(claimed.attach_state.forward_design_state(), "claimed");
            assert_eq!(claimed.attach_attempt_count, 1);
            assert_eq!(claimed.attach_claim_owner.as_deref(), Some("router::local"));
            assert!(claimed.attach_last_attempt_at.is_some());
            assert!(claimed.attach_completion_reason.is_none());

            let sibling = store
                .load_obligation("sess_auto_attach_claim", "obl_follow_up")
                .expect("load sibling obligation")
                .expect("sibling obligation exists");
            assert_eq!(
                sibling.attach_state,
                OrchestrationObligationAttachState::Eligible
            );
            assert_eq!(sibling.attach_state.forward_design_state(), "queued");

            let second_claim = store
                .claim_session_auto_attach("sess_auto_attach_claim", "router::second")
                .expect("coalesced second claim");
            assert_eq!(
                second_claim,
                SessionAutoAttachClaim::AlreadyClaimed {
                    obligation_id: "obl_approval".to_string(),
                }
            );

            let persisted = store
                .load_obligation("sess_auto_attach_claim", "obl_approval")
                .expect("reload claimed obligation")
                .expect("claimed obligation persists");
            assert_eq!(
                persisted.attach_claim_owner.as_deref(),
                Some("router::local")
            );
            assert_eq!(persisted.attach_attempt_count, 1);
        });
    }

    #[test]
    #[serial_test::serial]
    fn claim_exact_session_auto_attach_obligation_claims_requested_policy_eligible_sibling() {
        with_store(|store| {
            let participant =
                detached_orchestrator("codex", "sess_auto_attach_exact_claim", "ash_exact");
            let parent = parked_parent(&participant);
            store
                .persist_orchestration_session(&parent)
                .expect("persist parent");
            store
                .persist_participant(&participant)
                .expect("persist participant");

            let approval = pending_obligation(
                "sess_auto_attach_exact_claim",
                "obl_approval",
                OrchestrationObligationKind::ApprovalRequired,
            );
            let follow_up = pending_obligation(
                "sess_auto_attach_exact_claim",
                "obl_follow_up",
                OrchestrationObligationKind::FollowUpRequired,
            );
            store
                .persist_obligation(&approval)
                .expect("persist approval obligation");
            store
                .persist_obligation(&follow_up)
                .expect("persist follow-up obligation");

            let claim = store
                .claim_exact_session_auto_attach_obligation(
                    "sess_auto_attach_exact_claim",
                    "obl_follow_up",
                    "router::local",
                )
                .expect("claim exact follow-up obligation");
            assert_eq!(
                claim,
                SessionAutoAttachClaim::Claimed {
                    obligation_id: "obl_follow_up".to_string(),
                    attach_claim_owner: "router::local".to_string(),
                }
            );

            let claimed = store
                .load_obligation("sess_auto_attach_exact_claim", "obl_follow_up")
                .expect("reload exact claimed obligation")
                .expect("exact claimed obligation exists");
            assert_eq!(
                claimed.attach_state,
                OrchestrationObligationAttachState::Claimed
            );

            let sibling = store
                .load_obligation("sess_auto_attach_exact_claim", "obl_approval")
                .expect("reload approval sibling")
                .expect("approval sibling exists");
            assert_eq!(
                sibling.attach_state,
                OrchestrationObligationAttachState::Eligible
            );
        });
    }

    #[test]
    #[serial_test::serial]
    fn claim_session_auto_attach_recovers_stale_attached_truth() {
        with_store(|store| {
            let mut participant =
                live_orchestrator("codex", "sess_auto_attach_stale_claim", "ash_stale_claim");
            participant.internal.shell_owner_pid = 999_999_999;
            let parent = active_parent(&participant);
            store
                .persist_orchestration_session(&parent)
                .expect("persist parent");
            store
                .persist_participant(&participant)
                .expect("persist participant");
            store
                .persist_obligation(&pending_obligation(
                    "sess_auto_attach_stale_claim",
                    "obl_stale_claim",
                    OrchestrationObligationKind::ApprovalRequired,
                ))
                .expect("persist stale obligation");

            let claim = store
                .claim_session_auto_attach("sess_auto_attach_stale_claim", "router::local")
                .expect("claim stale-attached auto attach");
            assert_eq!(
                claim,
                SessionAutoAttachClaim::Claimed {
                    obligation_id: "obl_stale_claim".to_string(),
                    attach_claim_owner: "router::local".to_string(),
                }
            );
        });
    }

    #[test]
    #[serial_test::serial]
    fn list_router_auto_attach_candidate_session_ids_filters_to_detached_exact_truth() {
        with_store(|store| {
            let candidate =
                detached_orchestrator("codex", "sess_auto_attach_candidate", "ash_candidate");
            let candidate_parent = parked_parent(&candidate);
            store
                .persist_orchestration_session(&candidate_parent)
                .expect("persist candidate parent");
            store
                .persist_participant(&candidate)
                .expect("persist candidate participant");
            store
                .persist_obligation(&pending_obligation(
                    "sess_auto_attach_candidate",
                    "obl_candidate",
                    OrchestrationObligationKind::ApprovalRequired,
                ))
                .expect("persist candidate obligation");

            let mut stale =
                live_orchestrator("codex", "sess_auto_attach_stale", "ash_stale_candidate");
            stale.internal.shell_owner_pid = 999_999_999;
            let stale_parent = active_parent(&stale);
            store
                .persist_orchestration_session(&stale_parent)
                .expect("persist stale parent");
            store
                .persist_participant(&stale)
                .expect("persist stale participant");
            store
                .persist_obligation(&pending_obligation(
                    "sess_auto_attach_stale",
                    "obl_stale",
                    OrchestrationObligationKind::ApprovalRequired,
                ))
                .expect("persist stale obligation");

            let attached = live_orchestrator("codex", "sess_auto_attach_attached", "ash_attached");
            let attached_parent = active_parent(&attached);
            store
                .persist_orchestration_session(&attached_parent)
                .expect("persist attached parent");
            store
                .persist_participant(&attached)
                .expect("persist attached participant");
            store
                .persist_obligation(&pending_obligation(
                    "sess_auto_attach_attached",
                    "obl_attached",
                    OrchestrationObligationKind::ApprovalRequired,
                ))
                .expect("persist attached obligation");

            let terminal =
                detached_orchestrator("codex", "sess_auto_attach_terminal", "ash_terminal");
            let mut terminal_parent = parked_parent(&terminal);
            terminal_parent.transition_state(OrchestrationSessionState::Stopped);
            store
                .persist_orchestration_session(&terminal_parent)
                .expect("persist terminal parent");
            store
                .persist_obligation(&pending_obligation(
                    "sess_auto_attach_terminal",
                    "obl_terminal",
                    OrchestrationObligationKind::ApprovalRequired,
                ))
                .expect("persist terminal obligation");

            let not_eligible =
                detached_orchestrator("codex", "sess_auto_attach_not_eligible", "ash_ineligible");
            let not_eligible_parent = parked_parent(&not_eligible);
            store
                .persist_orchestration_session(&not_eligible_parent)
                .expect("persist ineligible parent");
            store
                .persist_obligation(&{
                    let mut obligation = pending_obligation(
                        "sess_auto_attach_not_eligible",
                        "obl_not_eligible",
                        OrchestrationObligationKind::ForkRecommendation,
                    );
                    obligation.attach_state = OrchestrationObligationAttachState::NotEligible;
                    obligation
                })
                .expect("persist ineligible obligation");

            let claimed = detached_orchestrator("codex", "sess_auto_attach_claimed", "ash_claimed");
            let claimed_parent = parked_parent(&claimed);
            store
                .persist_orchestration_session(&claimed_parent)
                .expect("persist claimed parent");
            store
                .persist_participant(&claimed)
                .expect("persist claimed participant");
            let mut claimed_obligation = pending_obligation(
                "sess_auto_attach_claimed",
                "obl_claimed",
                OrchestrationObligationKind::ApprovalRequired,
            );
            claimed_obligation.mark_attach_claimed("router::existing", Utc::now());
            store
                .persist_obligation(&claimed_obligation)
                .expect("persist claimed obligation");

            let invalid =
                detached_orchestrator("codex", "sess_auto_attach_invalid_batch", "ash_invalid");
            let invalid_parent = parked_parent(&invalid);
            store
                .persist_orchestration_session(&invalid_parent)
                .expect("persist invalid parent");
            store
                .persist_participant(&invalid)
                .expect("persist invalid participant");
            let mut first_invalid = pending_obligation(
                "sess_auto_attach_invalid_batch",
                "obl_invalid_first",
                OrchestrationObligationKind::Blocked,
            );
            first_invalid.mark_attach_claimed("router::one", Utc::now());
            store
                .persist_obligation(&first_invalid)
                .expect("persist first invalid obligation");
            let mut second_invalid = pending_obligation(
                "sess_auto_attach_invalid_batch",
                "obl_invalid_second",
                OrchestrationObligationKind::ApprovalRequired,
            );
            second_invalid.mark_attach_claimed("router::two", Utc::now());
            store
                .persist_obligation(&second_invalid)
                .expect("persist second invalid obligation");

            assert_eq!(
                store
                    .list_router_auto_attach_candidate_session_ids()
                    .expect("list router auto-attach candidates"),
                vec![
                    "sess_auto_attach_candidate".to_string(),
                    "sess_auto_attach_stale".to_string()
                ]
            );
        });
    }

    #[test]
    #[serial_test::serial]
    fn claim_session_auto_attach_fails_closed_when_multiple_claims_exist() {
        with_store(|store| {
            let participant =
                detached_orchestrator("codex", "sess_auto_attach_invalid", "ash_auto_invalid");
            let parent = parked_parent(&participant);
            store
                .persist_orchestration_session(&parent)
                .expect("persist parent");
            store
                .persist_participant(&participant)
                .expect("persist participant");

            let mut first = pending_obligation(
                "sess_auto_attach_invalid",
                "obl_first",
                OrchestrationObligationKind::Blocked,
            );
            first.mark_attach_claimed("router::one", Utc::now());
            store
                .persist_obligation(&first)
                .expect("persist first claimed obligation");

            let mut second = pending_obligation(
                "sess_auto_attach_invalid",
                "obl_second",
                OrchestrationObligationKind::ApprovalRequired,
            );
            second.mark_attach_claimed("router::two", Utc::now());
            store
                .persist_obligation(&second)
                .expect("persist second claimed obligation");

            let err = store
                .claim_session_auto_attach("sess_auto_attach_invalid", "router::local")
                .expect_err("multiple active claims must fail closed");
            assert!(err
                .to_string()
                .contains("multiple claimed auto-attach obligations"));
        });
    }

    #[test]
    #[serial_test::serial]
    fn settling_attached_session_auto_attach_satisfies_claim_and_supersedes_siblings() {
        with_store(|store| {
            let participant =
                live_orchestrator("codex", "sess_auto_attach_restored", "ash_auto_restored");
            let parent = active_parent(&participant);
            store
                .persist_orchestration_session(&parent)
                .expect("persist parent");
            store
                .persist_participant(&participant)
                .expect("persist participant");

            let mut claimed = pending_obligation(
                "sess_auto_attach_restored",
                "obl_claimed",
                OrchestrationObligationKind::Blocked,
            );
            claimed.mark_attach_claimed("router::local", Utc::now());
            store
                .persist_obligation(&claimed)
                .expect("persist claimed obligation");

            let eligible = pending_obligation(
                "sess_auto_attach_restored",
                "obl_sibling",
                OrchestrationObligationKind::FollowUpRequired,
            );
            store
                .persist_obligation(&eligible)
                .expect("persist eligible sibling obligation");

            let settled = store
                .settle_session_auto_attach_after_attach_restored(
                    "sess_auto_attach_restored",
                    "session_attach_restored_by_test",
                )
                .expect("settle attached session auto attach");
            assert_eq!(
                settled.satisfied_obligation_ids,
                vec!["obl_claimed".to_string()]
            );
            assert_eq!(
                settled.superseded_obligation_ids,
                vec!["obl_sibling".to_string()]
            );

            let claimed = store
                .load_obligation("sess_auto_attach_restored", "obl_claimed")
                .expect("load satisfied obligation")
                .expect("satisfied obligation exists");
            assert_eq!(
                claimed.attach_state,
                OrchestrationObligationAttachState::Satisfied
            );
            assert_eq!(claimed.attach_state.forward_design_state(), "completed");
            assert_eq!(
                claimed.attach_completion_reason.as_deref(),
                Some("session_attach_restored_by_test")
            );
            assert_eq!(claimed.attach_claim_owner.as_deref(), Some("router::local"));
            assert_eq!(
                claimed.review_state,
                OrchestrationObligationReviewState::Unread
            );
            assert_eq!(claimed.state, OrchestrationObligationState::Pending);
            assert!(claimed.resolved_at.is_none());

            let sibling = store
                .load_obligation("sess_auto_attach_restored", "obl_sibling")
                .expect("load superseded sibling")
                .expect("superseded sibling exists");
            assert_eq!(
                sibling.attach_state,
                OrchestrationObligationAttachState::Superseded
            );
            assert_eq!(sibling.attach_state.forward_design_state(), "cancelled");
            assert_eq!(
                sibling.attach_completion_reason.as_deref(),
                Some("session_attach_restored_by_test")
            );
            assert_eq!(
                sibling.review_state,
                OrchestrationObligationReviewState::Unread
            );
            assert_eq!(sibling.state, OrchestrationObligationState::Pending);
            assert!(sibling.resolved_at.is_none());
        });
    }

    #[test]
    #[serial_test::serial]
    fn releasing_session_auto_attach_claim_restores_claimed_obligation_to_eligible() {
        with_store(|store| {
            let participant = live_orchestrator(
                "codex",
                "sess_release_auto_attach_claim",
                "ash_release_claim",
            );
            let parent = active_parent(&participant);
            store
                .persist_orchestration_session(&parent)
                .expect("persist parent");
            store
                .persist_participant(&participant)
                .expect("persist participant");

            let mut claimed = pending_obligation(
                "sess_release_auto_attach_claim",
                "obl_release_claim",
                OrchestrationObligationKind::Blocked,
            );
            claimed.mark_attach_claimed("manual::reattach", Utc::now());
            let original_attempt_count = claimed.attach_attempt_count;
            let original_last_attempt_at = claimed.attach_last_attempt_at;
            store
                .persist_obligation(&claimed)
                .expect("persist claimed obligation");

            assert!(
                store
                    .release_session_auto_attach_claim(
                        "sess_release_auto_attach_claim",
                        "obl_release_claim",
                        "manual::reattach",
                    )
                    .expect("release attach claim"),
                "expected claimed obligation to be released"
            );

            let released = store
                .load_obligation("sess_release_auto_attach_claim", "obl_release_claim")
                .expect("load released obligation")
                .expect("released obligation exists");
            assert_eq!(
                released.attach_state,
                OrchestrationObligationAttachState::Eligible
            );
            assert_eq!(released.attach_claim_owner, None);
            assert_eq!(released.attach_completion_reason, None);
            assert_eq!(released.attach_attempt_count, original_attempt_count);
            assert_eq!(released.attach_last_attempt_at, original_last_attempt_at);
        });
    }

    #[test]
    #[serial_test::serial]
    fn settling_exact_session_auto_attach_obligation_failed_closed_preserves_sibling_review_state()
    {
        with_store(|store| {
            let participant = live_orchestrator(
                "codex",
                "sess_exact_auto_attach_fail_closed",
                "ash_exact_fail_closed",
            );
            let parent = active_parent(&participant);
            store
                .persist_orchestration_session(&parent)
                .expect("persist parent");
            store
                .persist_participant(&participant)
                .expect("persist participant");

            let mut claimed = pending_obligation(
                "sess_exact_auto_attach_fail_closed",
                "obl_claimed_wrong_host",
                OrchestrationObligationKind::ApprovalRequired,
            );
            claimed.review_state = OrchestrationObligationReviewState::Acknowledged;
            claimed.mark_attach_claimed("router::local", Utc::now());
            store
                .persist_obligation(&claimed)
                .expect("persist claimed wrong-host obligation");

            let mut sibling = pending_obligation(
                "sess_exact_auto_attach_fail_closed",
                "obl_sibling_remaining",
                OrchestrationObligationKind::FollowUpRequired,
            );
            sibling.review_state = OrchestrationObligationReviewState::Acknowledged;
            store
                .persist_obligation(&sibling)
                .expect("persist sibling obligation");

            let settled = store
                .settle_exact_session_auto_attach_obligation_failed_closed(
                    "sess_exact_auto_attach_fail_closed",
                    "obl_claimed_wrong_host",
                    "wrong_target_host: obligation obl_claimed_wrong_host targets host host-remote not local host host-local",
                )
                .expect("settle exact wrong-host obligation");
            assert_eq!(
                settled.failed_closed_obligation_ids,
                vec!["obl_claimed_wrong_host".to_string()]
            );

            let claimed = store
                .load_obligation(
                    "sess_exact_auto_attach_fail_closed",
                    "obl_claimed_wrong_host",
                )
                .expect("reload claimed wrong-host obligation")
                .expect("claimed wrong-host obligation exists");
            assert_eq!(
                claimed.attach_state,
                OrchestrationObligationAttachState::FailedClosed
            );
            assert_eq!(
                claimed.attach_completion_reason.as_deref(),
                Some(
                    "wrong_target_host: obligation obl_claimed_wrong_host targets host host-remote not local host host-local"
                )
            );
            assert_eq!(
                claimed.review_state,
                OrchestrationObligationReviewState::Acknowledged
            );
            assert_eq!(claimed.state, OrchestrationObligationState::Pending);
            assert!(claimed.resolved_at.is_none());

            let sibling = store
                .load_obligation(
                    "sess_exact_auto_attach_fail_closed",
                    "obl_sibling_remaining",
                )
                .expect("reload sibling obligation")
                .expect("sibling obligation exists");
            assert_eq!(
                sibling.attach_state,
                OrchestrationObligationAttachState::Eligible
            );
            assert_eq!(sibling.attach_completion_reason, None);
            assert_eq!(
                sibling.review_state,
                OrchestrationObligationReviewState::Acknowledged
            );
            assert_eq!(sibling.state, OrchestrationObligationState::Pending);
            assert!(sibling.resolved_at.is_none());
        });
    }

    #[test]
    #[serial_test::serial]
    fn settling_exact_session_auto_attach_obligation_failed_closed_from_eligible_preserves_no_claim_truth(
    ) {
        with_store(|store| {
            let participant = live_orchestrator(
                "codex",
                "sess_exact_auto_attach_fail_closed_eligible",
                "ash_exact_fail_closed_eligible",
            );
            let parent = active_parent(&participant);
            store
                .persist_orchestration_session(&parent)
                .expect("persist parent");
            store
                .persist_participant(&participant)
                .expect("persist participant");

            let mut eligible = pending_obligation(
                "sess_exact_auto_attach_fail_closed_eligible",
                "obl_wrong_host_eligible",
                OrchestrationObligationKind::ApprovalRequired,
            );
            eligible.review_state = OrchestrationObligationReviewState::Acknowledged;
            store
                .persist_obligation(&eligible)
                .expect("persist eligible wrong-host obligation");

            let settled = store
                .settle_exact_session_auto_attach_obligation_failed_closed(
                    "sess_exact_auto_attach_fail_closed_eligible",
                    "obl_wrong_host_eligible",
                    "wrong_target_host: obligation obl_wrong_host_eligible targets host host-remote not local host host-local",
                )
                .expect("settle eligible exact wrong-host obligation");
            assert_eq!(
                settled.failed_closed_obligation_ids,
                vec!["obl_wrong_host_eligible".to_string()]
            );

            let eligible = store
                .load_obligation(
                    "sess_exact_auto_attach_fail_closed_eligible",
                    "obl_wrong_host_eligible",
                )
                .expect("reload eligible wrong-host obligation")
                .expect("eligible wrong-host obligation exists");
            assert_eq!(
                eligible.attach_state,
                OrchestrationObligationAttachState::FailedClosed
            );
            assert_eq!(eligible.attach_claim_owner, None);
            assert_eq!(eligible.attach_attempt_count, 0);
            assert_eq!(eligible.attach_last_attempt_at, None);
            assert_eq!(
                eligible.attach_completion_reason.as_deref(),
                Some(
                    "wrong_target_host: obligation obl_wrong_host_eligible targets host host-remote not local host host-local"
                )
            );
            assert_eq!(
                eligible.review_state,
                OrchestrationObligationReviewState::Acknowledged
            );
            assert_eq!(eligible.state, OrchestrationObligationState::Pending);
            assert!(eligible.resolved_at.is_none());
        });
    }

    #[test]
    #[serial_test::serial]
    fn list_live_participants_for_session_preserves_same_agent_siblings() {
        with_store(|store| {
            let orchestrator = live_orchestrator("codex", "sess_siblings", "ash_orchestrator");
            let member = live_member("codex", "sess_siblings", "ash_member", "ash_orchestrator");
            store
                .persist_participant(&orchestrator)
                .expect("persist orchestrator");
            store.persist_participant(&member).expect("persist member");

            let participants = store
                .list_live_participants_for_session("sess_siblings")
                .expect("list live participants");
            assert_eq!(participants.len(), 2);
            assert_eq!(
                participants
                    .iter()
                    .map(|participant| participant.handle.participant_id.as_str())
                    .collect::<Vec<_>>(),
                vec!["ash_orchestrator", "ash_member"]
            );
        });
    }

    #[test]
    #[serial_test::serial]
    fn list_live_participants_filters_dead_owner_pid_rows() {
        with_store(|store| {
            let healthy = live_orchestrator("codex", "sess_live", "ash_live");
            let mut dead = live_orchestrator("codex", "sess_dead", "ash_dead");
            dead.internal.shell_owner_pid = u32::MAX;
            store
                .persist_participant(&healthy)
                .expect("persist healthy");
            store.persist_participant(&dead).expect("persist dead");

            let participants = store
                .list_live_participants()
                .expect("list live participants");
            assert_eq!(participants.len(), 1);
            assert_eq!(participants[0].handle.participant_id, "ash_live");
        });
    }

    #[test]
    #[serial_test::serial]
    fn list_live_participants_filters_ownership_invalid_rows() {
        with_store(|store| {
            let healthy = live_orchestrator("codex", "sess_live", "ash_live");
            let mut invalid = live_orchestrator("codex", "sess_invalid", "ash_invalid");
            invalid.internal.ownership_valid = false;
            invalid.internal.control_owner_retained = false;
            invalid.internal.event_stream_active = false;
            invalid.internal.completion_observer_retained = false;
            store
                .persist_participant(&healthy)
                .expect("persist healthy");
            store
                .persist_participant(&invalid)
                .expect("persist invalid");

            let participants = store
                .list_live_participants()
                .expect("list live participants");
            assert_eq!(participants.len(), 1);
            assert_eq!(participants[0].handle.participant_id, "ash_live");
        });
    }

    #[test]
    #[serial_test::serial]
    fn list_sessions_discovers_participant_only_roots_and_excludes_them_from_live_results() {
        with_store(|store| {
            let live_session_orchestrator = live_orchestrator("codex", "sess_live", "ash_live");
            let live_parent = active_parent(&live_session_orchestrator);
            let torn_orchestrator = live_orchestrator("codex", "sess_torn", "ash_torn");

            store
                .persist_participant(&live_session_orchestrator)
                .expect("persist live orchestrator");
            store
                .persist_orchestration_session(&live_parent)
                .expect("persist live parent");
            store
                .persist_participant(&torn_orchestrator)
                .expect("persist torn participant");

            let sessions = store.list_sessions().expect("list sessions");
            assert_eq!(sessions.len(), 2);
            let torn = sessions
                .iter()
                .find(|record| record.orchestration_session_id() == "sess_torn")
                .expect("participant-only torn root discovered");
            assert!(
                !torn.is_complete(),
                "participant-only torn roots must remain incomplete"
            );
            assert!(
                !torn.warnings.is_empty(),
                "participant-only torn roots must surface warnings"
            );

            let live_sessions = store.list_live_sessions().expect("list live sessions");
            assert_eq!(
                live_sessions
                    .iter()
                    .map(|record| record.orchestration_session_id())
                    .collect::<Vec<_>>(),
                vec!["sess_live"]
            );
        });
    }

    #[test]
    #[serial_test::serial]
    fn parent_only_torn_roots_degrade_with_warnings_instead_of_failing_discovery() {
        with_store(|store| {
            let participant = live_orchestrator("codex", "sess_parent_only", "ash_missing");
            let parent = active_parent(&participant);
            store
                .persist_orchestration_session(&parent)
                .expect("persist parent");

            let session = store
                .load_session("sess_parent_only")
                .expect("load parent-only torn root")
                .expect("parent-only torn root exists");
            assert!(
                !session.is_complete(),
                "parent-only torn roots must stay incomplete"
            );
            assert!(
                !session.warnings.is_empty(),
                "parent-only torn roots must surface warnings"
            );
            assert!(
                store
                    .list_live_sessions()
                    .expect("list live sessions")
                    .into_iter()
                    .all(|record| record.orchestration_session_id() != "sess_parent_only"),
                "parent-only torn roots must not be promoted into live discovery"
            );
        });
    }

    #[test]
    #[serial_test::serial]
    fn invalidate_stale_world_members_for_session_is_session_local_and_world_only() {
        with_store(|store| {
            let orchestrator = live_orchestrator("claude_code", "sess_live", "ash_orchestrator");
            let stale_member =
                live_member("codex", "sess_live", "ash_member_old", "ash_orchestrator");
            let mut current_member =
                live_member("codex", "sess_live", "ash_member_new", "ash_orchestrator");
            current_member.handle.world_generation = Some(3);
            let mut host_member = AgentRuntimeParticipantRecord::new_member_participant(
                &descriptor("codex", AgentExecutionScope::Host),
                "sess_live".to_string(),
                "ash_host_member".to_string(),
                "ash_orchestrator".to_string(),
                None,
                None,
                "lease_host".to_string(),
            )
            .expect("host member");
            set_live(&mut host_member);
            let stale_other_session = live_member(
                "codex",
                "sess_other",
                "ash_member_other",
                "ash_orchestrator",
            );

            store
                .persist_participant(&orchestrator)
                .expect("persist orchestrator");
            store
                .persist_participant(&stale_member)
                .expect("persist stale member");
            store
                .persist_participant(&current_member)
                .expect("persist current member");
            store
                .persist_participant(&host_member)
                .expect("persist host member");
            store
                .persist_participant(&stale_other_session)
                .expect("persist other session member");

            let invalidated = store
                .invalidate_stale_world_members_for_session("sess_live", 3)
                .expect("invalidate stale members");

            assert_eq!(invalidated, vec!["ash_member_old"]);
            assert_eq!(
                store
                    .load_participant("ash_member_old")
                    .expect("load stale member")
                    .expect("stale member exists")
                    .handle
                    .state,
                AgentRuntimeSessionState::Invalidated
            );
            assert!(store
                .load_participant("ash_member_new")
                .expect("load current member")
                .expect("current member exists")
                .is_authoritative_live());
            assert!(store
                .load_participant("ash_host_member")
                .expect("load host member")
                .expect("host member exists")
                .is_authoritative_live());
            assert!(store
                .load_participant("ash_member_other")
                .expect("load other session member")
                .expect("other session member exists")
                .is_authoritative_live());
        });
    }

    #[test]
    #[serial_test::serial]
    fn invalidate_stale_world_members_for_session_is_idempotent() {
        with_store(|store| {
            let orchestrator = live_orchestrator("claude_code", "sess_live", "ash_orchestrator");
            let stale_member =
                live_member("codex", "sess_live", "ash_member_old", "ash_orchestrator");

            store
                .persist_participant(&orchestrator)
                .expect("persist orchestrator");
            store
                .persist_participant(&stale_member)
                .expect("persist stale member");

            let first = store
                .invalidate_stale_world_members_for_session("sess_live", 3)
                .expect("first invalidation");
            let second = store
                .invalidate_stale_world_members_for_session("sess_live", 3)
                .expect("second invalidation");

            assert_eq!(first, vec!["ash_member_old"]);
            assert!(second.is_empty(), "second sweep must be a no-op");
        });
    }

    #[test]
    #[serial_test::serial]
    fn invalidate_stale_world_members_for_session_does_not_require_live_owner_pid() {
        with_store(|store| {
            let orchestrator = live_orchestrator("claude_code", "sess_live", "ash_orchestrator");
            let mut stale_member =
                live_member("codex", "sess_live", "ash_member_old", "ash_orchestrator");
            stale_member.internal.shell_owner_pid = 999_999_999;

            store
                .persist_participant(&orchestrator)
                .expect("persist orchestrator");
            store
                .persist_participant(&stale_member)
                .expect("persist stale member");

            let invalidated = store
                .invalidate_stale_world_members_for_session("sess_live", 3)
                .expect("invalidate stale members");

            assert_eq!(invalidated, vec!["ash_member_old"]);
            let stale_member = store
                .load_participant("ash_member_old")
                .expect("load stale member")
                .expect("stale member exists");
            assert_eq!(
                stale_member.handle.state,
                AgentRuntimeSessionState::Invalidated
            );
            assert_eq!(
                stale_member.internal.termination_reason.as_deref(),
                Some("world generation invalidated by replacement binding")
            );
        });
    }

    #[test]
    #[serial_test::serial]
    fn invalidate_stale_world_members_for_session_handles_large_batches() {
        with_store(|store| {
            let orchestrator = live_orchestrator("claude_code", "sess_live", "ash_orchestrator");
            store
                .persist_participant(&orchestrator)
                .expect("persist orchestrator");

            for idx in 0..256 {
                let participant_id = format!("ash_member_stale_{idx:03}");
                let stale_member =
                    live_member("codex", "sess_live", &participant_id, "ash_orchestrator");
                store
                    .persist_participant(&stale_member)
                    .expect("persist stale member");
            }

            let invalidated = store
                .invalidate_stale_world_members_for_session("sess_live", 3)
                .expect("invalidate stale members");

            assert_eq!(invalidated.len(), 256);
            for idx in 0..256 {
                let participant_id = format!("ash_member_stale_{idx:03}");
                let stale_member = store
                    .load_participant(&participant_id)
                    .expect("load stale member")
                    .expect("stale member exists");
                assert_eq!(
                    stale_member.handle.state,
                    AgentRuntimeSessionState::Invalidated,
                    "{participant_id} must be invalidated in the same sweep"
                );
            }
        });
    }

    #[test]
    #[serial_test::serial]
    fn recover_active_shared_world_binding_from_local_metadata_prefers_unique_active_binding() {
        with_store(|store| {
            let world_id = format!("wld_state_store_metadata_{}", uuid::Uuid::now_v7());
            let metadata_dir = write_shared_world_metadata_for_test(
                &world_id,
                "sess_metadata",
                7,
                SharedWorldBindingState::Active,
            );

            let binding = store
                .recover_active_shared_world_binding_from_local_metadata("sess_metadata")
                .expect("recover shared world binding")
                .expect("shared world binding should exist");

            assert_eq!(binding.world_id, world_id);
            assert_eq!(binding.world_generation, 7);

            fs::remove_dir_all(metadata_dir).expect("remove shared world metadata dir");
        });
    }

    #[test]
    #[serial_test::serial]
    fn recover_active_shared_world_binding_from_local_metadata_rejects_ambiguous_active_bindings() {
        with_store(|store| {
            let world_id_a = format!("wld_state_store_metadata_a_{}", uuid::Uuid::now_v7());
            let world_id_b = format!("wld_state_store_metadata_b_{}", uuid::Uuid::now_v7());
            let metadata_dir_a = write_shared_world_metadata_for_test(
                &world_id_a,
                "sess_metadata_ambiguous",
                7,
                SharedWorldBindingState::Active,
            );
            let metadata_dir_b = write_shared_world_metadata_for_test(
                &world_id_b,
                "sess_metadata_ambiguous",
                8,
                SharedWorldBindingState::Active,
            );

            let err = store
                .recover_active_shared_world_binding_from_local_metadata("sess_metadata_ambiguous")
                .expect_err("ambiguous shared world bindings must fail closed");

            assert!(
                err.to_string().contains("ambiguous_shared_world_binding"),
                "unexpected ambiguity error: {err:#}"
            );

            fs::remove_dir_all(metadata_dir_a).expect("remove shared world metadata dir a");
            fs::remove_dir_all(metadata_dir_b).expect("remove shared world metadata dir b");
        });
    }

    #[cfg(unix)]
    fn set_test_dir_mode(path: &Path, mode: u32) {
        fs::set_permissions(path, fs::Permissions::from_mode(mode))
            .expect("set shared world metadata root permissions");
    }

    #[cfg(unix)]
    #[test]
    #[serial_test::serial]
    fn recover_active_shared_world_binding_from_local_metadata_falls_back_to_live_participant_when_root_is_unreadable(
    ) {
        with_store(|store| {
            let metadata_root = shared_world_metadata_root();
            fs::create_dir_all(&metadata_root).expect("create shared world metadata root");

            let orchestrator =
                live_orchestrator("codex", "sess_metadata_permission_denied", "ash_orch");
            let mut member = live_member(
                "codex",
                "sess_metadata_permission_denied",
                "ash_member",
                "ash_orch",
            );
            member.handle.world_id = Some("world-live-fallback".to_string());
            member.handle.world_generation = Some(11);

            store
                .persist_participant(&orchestrator)
                .expect("persist orchestrator");
            store.persist_participant(&member).expect("persist member");

            set_test_dir_mode(&metadata_root, 0o000);
            let recovered = store.recover_active_shared_world_binding_from_local_metadata(
                "sess_metadata_permission_denied",
            );
            set_test_dir_mode(&metadata_root, 0o700);

            let binding = recovered
                .expect("recover shared world binding")
                .expect("shared world binding should fall back to live participant");
            assert_eq!(binding.world_id, "world-live-fallback");
            assert_eq!(binding.world_generation, 11);
        });
    }

    #[cfg(unix)]
    #[test]
    #[serial_test::serial]
    fn recover_active_shared_world_binding_from_local_metadata_fails_closed_when_root_is_unreadable_and_no_live_fallback(
    ) {
        with_store(|store| {
            let metadata_root = shared_world_metadata_root();
            fs::create_dir_all(&metadata_root).expect("create shared world metadata root");

            set_test_dir_mode(&metadata_root, 0o000);
            let recovered = store.recover_active_shared_world_binding_from_local_metadata(
                "sess_metadata_permission_denied_missing_fallback",
            );
            set_test_dir_mode(&metadata_root, 0o700);

            let err = recovered.expect_err("unreadable metadata without fallback must fail closed");
            assert!(
                err.to_string()
                    .contains("shared_world_binding_repair_metadata_unreadable"),
                "unexpected permission-denied error: {err:#}"
            );
        });
    }

    #[cfg(unix)]
    #[test]
    #[serial_test::serial]
    fn recover_active_shared_world_binding_from_local_metadata_rejects_ambiguous_live_fallback_bindings(
    ) {
        with_store(|store| {
            let metadata_root = shared_world_metadata_root();
            fs::create_dir_all(&metadata_root).expect("create shared world metadata root");

            let orchestrator = live_orchestrator(
                "codex",
                "sess_metadata_permission_denied_ambiguous",
                "ash_orch",
            );
            let mut member_a = live_member(
                "codex",
                "sess_metadata_permission_denied_ambiguous",
                "ash_member_a",
                "ash_orch",
            );
            member_a.handle.world_id = Some("world-live-a".to_string());
            member_a.handle.world_generation = Some(11);
            let mut member_b = live_member(
                "codex",
                "sess_metadata_permission_denied_ambiguous",
                "ash_member_b",
                "ash_orch",
            );
            member_b.handle.world_id = Some("world-live-b".to_string());
            member_b.handle.world_generation = Some(12);

            store
                .persist_participant(&orchestrator)
                .expect("persist orchestrator");
            store
                .persist_participant(&member_a)
                .expect("persist member a");
            store
                .persist_participant(&member_b)
                .expect("persist member b");

            set_test_dir_mode(&metadata_root, 0o000);
            let recovered = store.recover_active_shared_world_binding_from_local_metadata(
                "sess_metadata_permission_denied_ambiguous",
            );
            set_test_dir_mode(&metadata_root, 0o700);

            let err = recovered.expect_err("ambiguous live fallback must fail closed");
            assert!(
                err.to_string().contains("ambiguous_shared_world_binding"),
                "unexpected ambiguity error: {err:#}"
            );
        });
    }

    #[test]
    #[serial_test::serial]
    fn list_invalidated_participants_reads_authoritative_tombstones_only() {
        with_store(|store| {
            let mut participant =
                live_member("codex", "sess_live", "ash_member_old", "ash_orchestrator");
            participant.invalidate_for_world_generation_rollover();
            store
                .persist_participant(&participant)
                .expect("persist invalidated participant");
            write_legacy_handle_file(
                store,
                "ash_legacy_only",
                "codex",
                "sess_live",
                false,
                Some(json!({
                    "role": "member",
                    "execution": { "scope": "world" },
                    "state": "invalidated",
                    "world_id": "world-17",
                    "world_generation": 1,
                    "orchestrator_participant_id": "ash_orchestrator",
                    "internal": {
                        "ownership_mode": "member_runtime"
                    }
                })),
            );

            let invalidated = store
                .list_invalidated_participants()
                .expect("list invalidated participants");

            assert_eq!(invalidated.len(), 1);
            assert_eq!(invalidated[0].handle.participant_id, "ash_member_old");
        });
    }

    #[test]
    #[serial_test::serial]
    fn list_invalidated_participants_across_sources_includes_legacy_fallback_tombstones() {
        with_store(|store| {
            let mut participant =
                live_member("codex", "sess_live", "ash_member_old", "ash_orchestrator");
            participant.invalidate_for_world_generation_rollover();
            store
                .persist_participant(&participant)
                .expect("persist invalidated participant");
            write_legacy_handle_file(
                store,
                "ash_legacy_only",
                "codex",
                "sess_live",
                false,
                Some(json!({
                    "role": "member",
                    "execution": { "scope": "world" },
                    "state": "invalidated",
                    "world_id": "world-17",
                    "world_generation": 1,
                    "orchestrator_participant_id": "ash_orchestrator",
                    "internal": {
                        "ownership_mode": "member_runtime"
                    }
                })),
            );

            let invalidated = store
                .list_invalidated_participants_across_sources()
                .expect("list invalidated participants across sources");

            assert_eq!(invalidated.len(), 2);
            let mut participant_ids = invalidated
                .iter()
                .map(|participant| participant.handle.participant_id.as_str())
                .collect::<Vec<_>>();
            participant_ids.sort();
            assert_eq!(participant_ids, vec!["ash_legacy_only", "ash_member_old"]);
        });
    }

    #[test]
    #[serial_test::serial]
    fn resolve_live_orchestrator_participant_fails_closed_on_ambiguity() {
        with_store(|store| {
            let participant_a = live_orchestrator("codex", "sess_a", "ash_a");
            let participant_b = live_orchestrator("codex", "sess_b", "ash_b");
            let parent_a = active_parent(&participant_a);
            let parent_b = active_parent(&participant_b);
            store
                .persist_participant(&participant_a)
                .expect("persist participant a");
            store
                .persist_participant(&participant_b)
                .expect("persist participant b");
            store
                .persist_orchestration_session(&parent_a)
                .expect("persist parent a");
            store
                .persist_orchestration_session(&parent_b)
                .expect("persist parent b");

            let err = store
                .resolve_live_orchestrator_participant("codex")
                .expect_err("ambiguous orchestrators must fail closed");
            assert!(err.to_string().contains(
                "multiple active orchestration session candidates found for agent codex"
            ));
        });
    }

    #[test]
    #[serial_test::serial]
    fn resolve_public_control_target_rejects_active_session_handle_selector() {
        with_store(|store| {
            let participant = live_orchestrator("codex", "sess_public", "ash_selected");
            let parent = active_parent(&participant);
            store
                .persist_orchestration_session(&parent)
                .expect("persist parent");
            store
                .persist_participant(&participant)
                .expect("persist participant");

            let err = store
                .resolve_public_control_target("ash_selected", PublicControlAction::Stop)
                .expect_err("non-canonical active_session_handle_id selectors must be rejected");
            assert!(err.to_string().contains("unknown_session"));
            assert!(err.to_string().contains("active_session_handle_id"));
        });
    }

    #[test]
    #[serial_test::serial]
    fn resolve_public_control_target_rejects_internal_uaa_selector() {
        with_store(|store| {
            let participant = live_orchestrator("codex", "sess_public", "ash_selected");
            let parent = active_parent(&participant);
            store
                .persist_orchestration_session(&parent)
                .expect("persist parent");
            store
                .persist_participant(&participant)
                .expect("persist participant");

            let err = store
                .resolve_public_control_target("uaa_session", PublicControlAction::Fork)
                .expect_err("internal uaa session selectors must be rejected");
            assert!(err.to_string().contains("unknown_session"));
            assert!(err.to_string().contains("internal.uaa_session_id"));
        });
    }

    #[test]
    #[serial_test::serial]
    fn resolve_public_attach_target_reattach_rejects_already_owned_sessions() {
        with_store(|store| {
            let participant = live_orchestrator("codex", "sess_resume_live", "ash_selected");
            let parent = active_parent(&participant);
            store
                .persist_orchestration_session(&parent)
                .expect("persist parent");
            store
                .persist_participant(&participant)
                .expect("persist participant");

            let err = store
                .resolve_public_attach_target("sess_resume_live", PublicAttachAction::Reattach)
                .expect_err("live retained ownership must reject control-only reattach");
            assert!(err.to_string().contains("session_already_owned"));
        });
    }

    #[test]
    #[serial_test::serial]
    fn resolve_public_attach_target_allows_detached_turn_for_parked_session() {
        with_store(|store| {
            let participant = detached_orchestrator("codex", "sess_resume_parked", "ash_detached");
            let mut parent = active_parent(&participant);
            parent.mark_parked_resumable("owner detached cleanly");
            store
                .persist_orchestration_session(&parent)
                .expect("persist parent");
            store
                .persist_participant(&participant)
                .expect("persist participant");

            let target = store
                .resolve_public_attach_target(
                    "sess_resume_parked",
                    PublicAttachAction::DetachedTurn,
                )
                .expect("parked session should remain attachable for detached turns");
            assert_eq!(
                target.active_participant.handle.participant_id,
                "ash_detached"
            );
        });
    }

    #[test]
    #[serial_test::serial]
    fn resolve_public_attach_target_rejects_terminal_session_even_with_continuity_metadata() {
        with_store(|store| {
            let participant = detached_orchestrator("codex", "sess_terminal_attach", "ash_dead");
            let parent = active_parent(&participant);
            store
                .persist_orchestration_session(&parent)
                .expect("persist active parent");
            store
                .persist_participant(&participant)
                .expect("persist detached participant");

            let err = store
                .resolve_public_attach_target("sess_terminal_attach", PublicAttachAction::Reattach)
                .expect_err("terminal public posture must fail closed for public reattach");
            assert!(err.to_string().contains("session_not_reattachable"));
            assert!(err.to_string().contains("Terminal"));
        });
    }

    #[test]
    #[serial_test::serial]
    fn resolve_public_attach_target_requires_continuity_but_fork_does_not() {
        with_store(|store| {
            let participant =
                detached_orchestrator("codex", "sess_missing_internal", "ash_detached");
            let mut participant = participant;
            participant.internal.uaa_session_id = None;
            participant.internal.resume_eligible = false;
            let parent = parked_parent(&participant);
            store
                .persist_orchestration_session(&parent)
                .expect("persist parent");
            store
                .persist_participant(&participant)
                .expect("persist participant");

            let attach_err = store
                .resolve_public_attach_target("sess_missing_internal", PublicAttachAction::Reattach)
                .expect_err("reattach must require continuity");
            assert!(attach_err.to_string().contains("owner_unreachable"));

            let fork_target = store
                .resolve_public_control_target("sess_missing_internal", PublicControlAction::Fork)
                .expect("fork should allow durable successor allocation without continuity");
            assert_eq!(
                fork_target.active_participant.handle.participant_id,
                "ash_detached"
            );
            assert!(
                fork_target.host_attach_contract.is_some(),
                "fork must still require the durable host attach contract"
            );
            assert!(fork_target
                .host_attach_contract
                .as_ref()
                .expect("durable contract")
                .supports_fork());
        });
    }

    #[test]
    #[serial_test::serial]
    fn resolve_public_attach_target_uses_persisted_continuity_truth() {
        with_store(|store| {
            let participant =
                detached_orchestrator("codex", "sess_persisted_resume", "ash_detached");
            let parent = parked_parent(&participant);
            let mut participant = participant;
            participant.internal.uaa_session_id = None;

            store
                .persist_orchestration_session(&parent)
                .expect("persist parent");
            store
                .persist_participant(&participant)
                .expect("persist participant");

            let target = store
                .resolve_public_attach_target("sess_persisted_resume", PublicAttachAction::Reattach)
                .expect("persisted contract continuity should remain authoritative");
            assert_eq!(
                target
                    .host_attach_contract
                    .as_ref()
                    .and_then(|contract| contract.continuity_uaa_session_id.as_deref()),
                Some("uaa_session")
            );
        });
    }

    #[test]
    #[serial_test::serial]
    fn resolve_public_attach_target_respects_persisted_continuity_attach_narrowing() {
        with_store(|store| {
            let participant = detached_orchestrator("codex", "sess_resume_denied", "ash_detached");
            let mut parent = parked_parent(&participant);
            parent
                .host_attach_contract
                .as_mut()
                .expect("durable contract")
                .capabilities
                .session_resume = false;

            store
                .persist_orchestration_session(&parent)
                .expect("persist parent");
            store
                .persist_participant(&participant)
                .expect("persist participant");

            let err = store
                .resolve_public_attach_target("sess_resume_denied", PublicAttachAction::Reattach)
                .expect_err("attach planning must honor persisted capability narrowing");
            assert!(err.to_string().contains("does not allow continuity attach"));
        });
    }

    #[test]
    #[serial_test::serial]
    fn resolve_public_control_target_respects_persisted_fork_narrowing() {
        with_store(|store| {
            let participant = detached_orchestrator("codex", "sess_fork_denied", "ash_detached");
            let mut parent = parked_parent(&participant);
            parent
                .host_attach_contract
                .as_mut()
                .expect("durable contract")
                .capabilities
                .session_fork = false;

            store
                .persist_orchestration_session(&parent)
                .expect("persist parent");
            store
                .persist_participant(&participant)
                .expect("persist participant");

            let err = store
                .resolve_public_control_target("sess_fork_denied", PublicControlAction::Fork)
                .expect_err("fork must honor persisted capability narrowing");
            assert!(err.to_string().contains("does not allow fork"));
        });
    }

    #[test]
    #[serial_test::serial]
    fn resolve_public_control_target_respects_persisted_stop_narrowing() {
        with_store(|store| {
            let participant = detached_orchestrator("codex", "sess_stop_denied", "ash_detached");
            let mut parent = parked_parent(&participant);
            parent
                .host_attach_contract
                .as_mut()
                .expect("durable contract")
                .capabilities
                .session_stop = false;

            store
                .persist_orchestration_session(&parent)
                .expect("persist parent");
            store
                .persist_participant(&participant)
                .expect("persist participant");

            let err = store
                .resolve_public_control_target("sess_stop_denied", PublicControlAction::Stop)
                .expect_err("stop must honor persisted capability narrowing");
            assert!(err.to_string().contains("does not allow stop"));
        });
    }

    #[test]
    #[serial_test::serial]
    fn resolve_public_turn_target_reports_detached_posture_for_parked_host_session() {
        with_store(|store| {
            let participant =
                detached_orchestrator("codex-host", "sess_turn_parked", "ash_detached");
            let mut parent = active_parent(&participant);
            parent.mark_parked_resumable("owner detached cleanly");
            store
                .persist_orchestration_session(&parent)
                .expect("persist parent");
            store
                .persist_participant(&participant)
                .expect("persist participant");

            let target = store
                .resolve_public_turn_target("sess_turn_parked", "cli:codex-host")
                .expect("parked host target");
            assert_eq!(
                target.session_posture,
                PublicSessionPosture::DetachedReattachable
            );
            assert!(target.host_attach_contract.is_some());
        });
    }

    #[test]
    #[serial_test::serial]
    fn resolve_public_turn_target_uses_persisted_continuity_truth() {
        with_store(|store| {
            let participant =
                detached_orchestrator("codex-host", "sess_turn_persisted", "ash_detached");
            let mut parent = active_parent(&participant);
            parent.mark_parked_resumable("owner detached cleanly");
            let mut participant = participant;
            participant.internal.uaa_session_id = None;

            store
                .persist_orchestration_session(&parent)
                .expect("persist parent");
            store
                .persist_participant(&participant)
                .expect("persist participant");

            let target = store
                .resolve_public_turn_target("sess_turn_persisted", "cli:codex-host")
                .expect("persisted contract continuity should keep detached posture");
            assert_eq!(
                target.session_posture,
                PublicSessionPosture::DetachedReattachable
            );
            assert_eq!(
                target
                    .host_attach_contract
                    .as_ref()
                    .and_then(|contract| contract.continuity_uaa_session_id.as_deref()),
                Some("uaa_session")
            );
        });
    }

    #[test]
    #[serial_test::serial]
    fn resolve_public_turn_target_recovers_stale_attached_host_owner_as_detached() {
        with_store(|store| {
            let mut participant = live_orchestrator("codex-host", "sess_turn_stale", "ash_stale");
            participant.internal.shell_owner_pid = 999_999_999;
            let parent = active_parent(&participant);
            store
                .persist_orchestration_session(&parent)
                .expect("persist parent");
            store
                .persist_participant(&participant)
                .expect("persist participant");

            let target = store
                .resolve_public_turn_target("sess_turn_stale", "cli:codex-host")
                .expect("stale attached host target");
            assert_eq!(
                target.session_posture,
                PublicSessionPosture::DetachedReattachable
            );
        });
    }

    #[test]
    #[serial_test::serial]
    fn resolve_public_turn_target_does_not_recover_stale_attached_owner_when_live_successor_exists()
    {
        with_store(|store| {
            let mut stale = live_orchestrator("codex-host", "sess_turn_stale_blocked", "ash_stale");
            stale.internal.shell_owner_pid = 999_999_999;
            let live_successor =
                live_orchestrator("codex-host", "sess_turn_stale_blocked", "ash_successor");
            let parent = active_parent(&stale);
            store
                .persist_orchestration_session(&parent)
                .expect("persist parent");
            store.persist_participant(&stale).expect("persist stale");
            store
                .persist_participant(&live_successor)
                .expect("persist live successor");

            let target = store
                .resolve_public_turn_target("sess_turn_stale_blocked", "cli:codex-host")
                .expect("stale attached host target with live successor should still resolve");
            assert_eq!(target.session_posture, PublicSessionPosture::Terminal);
        });
    }

    #[test]
    #[serial_test::serial]
    fn startup_prompt_replay_state_allows_replay_only_before_acceptance() {
        with_store(|store| {
            let participant = live_orchestrator("codex", "sess_startup_prompt", "ash_start");
            let mut parent = active_parent(&participant);
            parent.initialize_startup_prompt("ash_start");
            store
                .persist_orchestration_session(&parent)
                .expect("persist parent");
            store
                .persist_participant(&participant)
                .expect("persist participant");

            assert_eq!(
                store
                    .startup_prompt_replay_state("sess_startup_prompt", "ash_start")
                    .expect("pending replay state"),
                StartupPromptReplayState::PendingAcceptance
            );

            let mut accepted_parent = parent.clone();
            accepted_parent.mark_startup_prompt_accepted("ash_start");
            store
                .persist_orchestration_session(&accepted_parent)
                .expect("persist accepted parent");
            assert_eq!(
                store
                    .startup_prompt_replay_state("sess_startup_prompt", "ash_start")
                    .expect("accepted replay state"),
                StartupPromptReplayState::AcceptedOrTerminal
            );
        });
    }

    #[test]
    #[serial_test::serial]
    fn hidden_owner_helper_launch_classifier_accepts_detached_attention_needed_session() {
        with_store(|store| {
            let participant =
                detached_orchestrator("codex", "sess_ready_attention", "ash_detached");
            let parent = parked_parent(&participant);
            store
                .persist_orchestration_session(&parent)
                .expect("persist parent");
            store
                .persist_participant(&participant)
                .expect("persist participant");
            store
                .persist_inbox_item(&pending_inbox_item(
                    "sess_ready_attention",
                    "item_attention",
                    DurableInboxItemKind::ApprovalRequired,
                ))
                .expect("persist pending inbox item");

            let continuity = store
                .classify_hidden_owner_helper_launch_continuity(
                    "sess_ready_attention",
                    "ash_detached",
                    true,
                )
                .expect("classify continuity");
            assert_eq!(
                continuity,
                HiddenOwnerHelperLaunchContinuity::DetachedReconciled(
                    OrchestrationSessionPosture::AwaitingAttention,
                )
            );

            let readiness = store
                .classify_hidden_owner_helper_launch_readiness(
                    "sess_ready_attention",
                    "ash_detached",
                    true,
                )
                .expect("classify readiness");
            assert_eq!(
                readiness,
                HiddenOwnerHelperLaunchReadiness::ReadyDetached(
                    OrchestrationSessionPosture::AwaitingAttention,
                )
            );
        });
    }

    #[test]
    #[serial_test::serial]
    fn hidden_owner_helper_launch_classifier_rejects_detached_posture_pending_mismatch() {
        with_store(|store| {
            let participant = detached_orchestrator("codex", "sess_bad_detached", "ash_detached");
            let mut parent = parked_parent(&participant);
            parent.pending_inbox_count = 1;
            write_canonical_session_file(store, &parent);
            write_flat_session_file(store, &parent);
            store
                .persist_participant(&participant)
                .expect("persist participant");

            let err = store
                .classify_hidden_owner_helper_launch_readiness(
                    "sess_bad_detached",
                    "ash_detached",
                    true,
                )
                .expect_err("invalid detached posture must fail closed");
            assert!(err.to_string().contains("invalid session record"));

            let loaded = store
                .load_session("sess_bad_detached")
                .expect_err("invalid detached posture must fail closed during load");
            assert!(loaded.to_string().contains("invalid session record"));
        });
    }

    #[test]
    #[serial_test::serial]
    fn load_session_fails_closed_for_incomplete_host_attach_contract_json() {
        with_store(|store| {
            let participant = live_orchestrator("codex", "sess_legacy_attach", "ash_selected");
            let parent = active_parent(&participant);
            let mut payload = serde_json::to_value(&parent).expect("serialize session");
            let contract = payload
                .get_mut("host_attach_contract")
                .and_then(Value::as_object_mut)
                .expect("host attach contract");
            contract.remove("capabilities");
            contract.remove("attach_launch_knobs");

            fs::create_dir_all(store.sessions_dir()).expect("create sessions dir");
            fs::create_dir_all(store.canonical_session_dir("sess_legacy_attach"))
                .expect("create canonical session dir");
            fs::write(
                store.orchestration_session_path("sess_legacy_attach"),
                serde_json::to_vec_pretty(&payload).expect("serialize legacy flat session"),
            )
            .expect("write legacy flat session");
            fs::write(
                store.canonical_session_path("sess_legacy_attach"),
                serde_json::to_vec_pretty(&payload).expect("serialize legacy canonical session"),
            )
            .expect("write legacy canonical session");
            store
                .persist_participant(&participant)
                .expect("persist participant");

            let err = store
                .load_session("sess_legacy_attach")
                .expect_err("incomplete persisted attach truth must fail closed");
            assert!(err.to_string().contains("failed to parse"));
        });
    }

    #[test]
    #[serial_test::serial]
    fn hidden_owner_helper_launch_continuity_reports_stale_attached_truth_separately() {
        with_store(|store| {
            let mut participant = live_orchestrator("codex", "sess_stale_attached", "ash_stale");
            participant.internal.shell_owner_pid = 999_999_999;
            let parent = active_parent(&participant);
            store
                .persist_orchestration_session(&parent)
                .expect("persist parent");
            store
                .persist_participant(&participant)
                .expect("persist participant");

            let continuity = store
                .classify_hidden_owner_helper_launch_continuity(
                    "sess_stale_attached",
                    "ash_stale",
                    true,
                )
                .expect("classify continuity");
            assert_eq!(
                continuity,
                HiddenOwnerHelperLaunchContinuity::StaleAttachedTruth
            );

            let readiness = store
                .classify_hidden_owner_helper_launch_readiness(
                    "sess_stale_attached",
                    "ash_stale",
                    true,
                )
                .expect("classify readiness");
            assert_eq!(readiness, HiddenOwnerHelperLaunchReadiness::Pending);
        });
    }

    #[test]
    #[serial_test::serial]
    fn resolve_public_turn_target_rejects_active_session_handle_selector() {
        with_store(|store| {
            let participant = live_orchestrator("codex-host", "sess_public_turn", "ash_selected");
            let parent = active_parent(&participant);
            store
                .persist_orchestration_session(&parent)
                .expect("persist parent");
            store
                .persist_participant(&participant)
                .expect("persist participant");

            let err = store
                .resolve_public_turn_target("ash_selected", "cli:codex-host")
                .expect_err("non-canonical active_session_handle_id selectors must be rejected");
            assert!(err.to_string().contains("noncanonical_session_selector"));
            assert!(err.to_string().contains("active_session_handle_id"));
        });
    }

    #[test]
    #[serial_test::serial]
    fn resolve_public_turn_target_rejects_internal_uaa_selector() {
        with_store(|store| {
            let participant = live_orchestrator("codex-host", "sess_public_turn", "ash_selected");
            let parent = active_parent(&participant);
            store
                .persist_orchestration_session(&parent)
                .expect("persist parent");
            store
                .persist_participant(&participant)
                .expect("persist participant");

            let err = store
                .resolve_public_turn_target("uaa_session", "cli:codex-host")
                .expect_err("internal uaa session selectors must be rejected");
            assert!(err.to_string().contains("noncanonical_session_selector"));
            assert!(err.to_string().contains("internal.uaa_session_id"));
        });
    }

    #[test]
    #[serial_test::serial]
    fn resolve_public_turn_target_requires_exact_world_member_linkage() {
        with_store(|store| {
            let orchestrator =
                live_orchestrator("codex-host", "sess_world_turn_stale", "ash_selected");
            let mut member = live_member(
                "codex-world",
                "sess_world_turn_stale",
                "ash_member",
                "ash_stale_owner",
            );
            member.handle.backend_id = "cli:world-member".to_string();
            let mut parent = active_parent(&orchestrator);
            parent.set_world_binding("world-17", 2);

            store
                .persist_orchestration_session(&parent)
                .expect("persist parent");
            store
                .persist_participant(&orchestrator)
                .expect("persist orchestrator");
            store.persist_participant(&member).expect("persist member");

            let err = store
                .resolve_public_turn_target("sess_world_turn_stale", "cli:world-member")
                .expect_err("stale world-member linkage must fail closed");
            assert!(err.to_string().contains("stale_linkage"));
        });
    }

    #[test]
    #[serial_test::serial]
    fn resolve_public_turn_target_accepts_successor_lineage_world_member() {
        with_store(|store| {
            let mut launch_orchestrator =
                live_orchestrator("codex-host", "sess_world_turn_successor", "ash_launch");
            let mut successor =
                live_orchestrator("codex-host", "sess_world_turn_successor", "ash_successor");
            successor.handle.resumed_from_participant_id = Some("ash_launch".to_string());
            successor.handle.resumed_from_session_handle_id = Some("ash_launch".to_string());
            launch_orchestrator.mark_client_detached("successor attached");

            let member = live_member(
                "codex-world",
                "sess_world_turn_successor",
                "ash_member",
                "ash_launch",
            );

            let mut parent = active_parent(&launch_orchestrator);
            parent.set_world_binding("world-17", 2);
            parent.bind_active_session_handle("ash_successor".to_string());

            store
                .persist_orchestration_session(&parent)
                .expect("persist parent");
            store
                .persist_participant(&launch_orchestrator)
                .expect("persist launch orchestrator");
            store
                .persist_participant(&successor)
                .expect("persist successor orchestrator");
            store.persist_participant(&member).expect("persist member");

            let target = store
                .resolve_public_turn_target("sess_world_turn_successor", "cli:codex-world")
                .expect("successor lineage should resolve exact world-member target");

            assert_eq!(target.participant.participant_id(), "ash_member");
            assert_eq!(target.target_kind, PublicTurnTargetKind::World);
        });
    }

    #[test]
    #[serial_test::serial]
    fn resolve_public_turn_target_reports_ambiguity_for_multiple_same_backend_successor_lineage_workers(
    ) {
        with_store(|store| {
            let mut launch_orchestrator =
                live_orchestrator("codex-host", "sess_world_turn_ambiguous", "ash_launch");
            let mut successor =
                live_orchestrator("codex-host", "sess_world_turn_ambiguous", "ash_successor");
            successor.handle.resumed_from_participant_id = Some("ash_launch".to_string());
            successor.handle.resumed_from_session_handle_id = Some("ash_launch".to_string());
            launch_orchestrator.mark_client_detached("successor attached");

            let source = live_member(
                "codex-world",
                "sess_world_turn_ambiguous",
                "ash_source",
                "ash_launch",
            );
            let mut child = live_member(
                "codex-world",
                "sess_world_turn_ambiguous",
                "ash_child",
                "ash_launch",
            );
            child.handle.parent_participant_id = Some("ash_source".to_string());

            let mut parent = active_parent(&launch_orchestrator);
            parent.set_world_binding("world-17", 2);
            parent.bind_active_session_handle("ash_successor".to_string());

            store
                .persist_orchestration_session(&parent)
                .expect("persist parent");
            store
                .persist_participant(&launch_orchestrator)
                .expect("persist launch orchestrator");
            store
                .persist_participant(&successor)
                .expect("persist successor orchestrator");
            store.persist_participant(&source).expect("persist source");
            store.persist_participant(&child).expect("persist child");

            let err = store
                .resolve_public_turn_target("sess_world_turn_ambiguous", "cli:codex-world")
                .expect_err("same-backend retained targets must fail closed as ambiguous");

            let err = err.to_string();
            assert!(err.contains("ambiguous_backend_slot"));
            assert!(err.contains("sess_world_turn_ambiguous"));
            assert!(err.contains("cli:codex-world"));
            assert!(err.contains("ash_source"));
            assert!(err.contains("ash_child"));
        });
    }

    #[test]
    #[serial_test::serial]
    fn resolve_public_turn_target_rejects_retired_codex_exact_selector() {
        with_store(|store| {
            let participant = live_orchestrator("codex-host", "sess_public_turn", "ash_selected");
            let parent = active_parent(&participant);
            store
                .persist_orchestration_session(&parent)
                .expect("persist parent");
            store
                .persist_participant(&participant)
                .expect("persist participant");

            let err = store
                .resolve_public_turn_target("sess_public_turn", "cli:codex")
                .expect_err("retired codex selector must fail closed");
            assert_eq!(
                err.to_string(),
                "legacy exact backend 'cli:codex' is retired; use 'cli:codex-host' or 'cli:codex-world'"
            );
        });
    }

    #[test]
    #[serial_test::serial]
    fn resolve_public_turn_target_rejects_retired_claude_code_exact_selector() {
        with_store(|store| {
            let participant =
                live_orchestrator("claude_code-host", "sess_public_turn", "ash_selected");
            let parent = active_parent(&participant);
            store
                .persist_orchestration_session(&parent)
                .expect("persist parent");
            store
                .persist_participant(&participant)
                .expect("persist participant");

            let err = store
                .resolve_public_turn_target("sess_public_turn", "cli:claude_code")
                .expect_err("retired claude_code selector must fail closed");
            assert_eq!(
                err.to_string(),
                "legacy exact backend 'cli:claude_code' is retired; use 'cli:claude_code-host' or 'cli:claude_code-world'"
            );
        });
    }

    #[test]
    #[serial_test::serial]
    fn resolve_public_turn_target_rejects_retired_codex_world_exact_selector() {
        with_store(|store| {
            let participant = live_orchestrator("codex-host", "sess_public_turn", "ash_selected");
            let parent = active_parent(&participant);
            store
                .persist_orchestration_session(&parent)
                .expect("persist parent");
            store
                .persist_participant(&participant)
                .expect("persist participant");

            let err = store
                .resolve_public_turn_target("sess_public_turn", "cli:codex_world")
                .expect_err("retired codex world selector must fail closed");
            assert_eq!(
                err.to_string(),
                "legacy exact backend 'cli:codex_world' is retired; use 'cli:codex-world'"
            );
        });
    }

    #[test]
    #[serial_test::serial]
    fn resolve_public_turn_target_rejects_retired_claude_code_world_exact_selector() {
        with_store(|store| {
            let participant =
                live_orchestrator("claude_code-host", "sess_public_turn", "ash_selected");
            let parent = active_parent(&participant);
            store
                .persist_orchestration_session(&parent)
                .expect("persist parent");
            store
                .persist_participant(&participant)
                .expect("persist participant");

            let err = store
                .resolve_public_turn_target("sess_public_turn", "cli:claude_code_world")
                .expect_err("retired claude_code world selector must fail closed");
            assert_eq!(
                err.to_string(),
                "legacy exact backend 'cli:claude_code_world' is retired; use 'cli:claude_code-world'"
            );
        });
    }

    #[test]
    #[serial_test::serial]
    fn resolve_public_control_target_stop_allows_detached_resumable_session() {
        with_store(|store| {
            let mut participant =
                detached_orchestrator("codex", "sess_stop_detached", "ash_selected");
            participant.set_uaa_session_id("uaa_session");
            let mut parent = active_parent(&participant);
            parent.mark_parked_resumable("owner detached cleanly");
            store
                .persist_orchestration_session(&parent)
                .expect("persist parent");
            store
                .persist_participant(&participant)
                .expect("persist participant");

            let target = store
                .resolve_public_control_target("sess_stop_detached", PublicControlAction::Stop)
                .expect("stop must remain available for parked resumable sessions");
            assert_eq!(
                target.session_posture,
                PublicSessionPosture::DetachedReattachable
            );
            assert_eq!(
                target.active_participant.handle.participant_id,
                "ash_selected"
            );
        });
    }

    #[test]
    #[serial_test::serial]
    fn resolve_public_control_target_stop_requires_resume_contract_for_detached_session() {
        with_store(|store| {
            let mut participant =
                detached_orchestrator("codex", "sess_stop_missing_internal", "ash_selected");
            participant.internal.uaa_session_id = None;
            participant.internal.resume_eligible = false;
            let mut parent = active_parent(&participant);
            parent.mark_parked_resumable("owner detached cleanly");
            store
                .persist_orchestration_session(&parent)
                .expect("persist parent");
            store
                .persist_participant(&participant)
                .expect("persist participant");

            let err = store
                .resolve_public_control_target(
                    "sess_stop_missing_internal",
                    PublicControlAction::Stop,
                )
                .expect_err("detached stop must fail closed without a resumable retained owner");
            assert!(err.to_string().contains("owner_unreachable"));
        });
    }

    #[test]
    #[serial_test::serial]
    fn resolve_public_control_target_stop_allows_detached_attention_session() {
        with_store(|store| {
            let participant =
                detached_orchestrator("codex", "sess_stop_attention", "ash_attention");
            let parent = parked_parent(&participant);
            store
                .persist_orchestration_session(&parent)
                .expect("persist parent");
            store
                .persist_participant(&participant)
                .expect("persist participant");
            store
                .persist_inbox_item(&pending_inbox_item(
                    "sess_stop_attention",
                    "item_attention",
                    DurableInboxItemKind::ApprovalRequired,
                ))
                .expect("persist pending inbox item");

            let target = store
                .resolve_public_control_target("sess_stop_attention", PublicControlAction::Stop)
                .expect("stop must remain available for attention-needed durable sessions");
            assert_eq!(
                target.session_posture,
                PublicSessionPosture::DetachedReattachable
            );
            assert_eq!(
                target.session.posture,
                OrchestrationSessionPosture::AwaitingAttention
            );
            assert_eq!(target.session.pending_inbox_count, 1);
            assert_eq!(
                target.active_participant.handle.participant_id,
                "ash_attention"
            );
        });
    }

    #[test]
    #[serial_test::serial]
    fn status_visible_participants_include_detached_authoritative_orchestrator() {
        with_store(|store| {
            let mut participant =
                detached_orchestrator("codex", "sess_status_detached", "ash_detached");
            participant.set_uaa_session_id("uaa_session");
            let mut parent = active_parent(&participant);
            parent.mark_parked_resumable("owner detached cleanly");
            store
                .persist_orchestration_session(&parent)
                .expect("persist parent");
            store
                .persist_participant(&participant)
                .expect("persist participant");

            let record = store
                .load_session("sess_status_detached")
                .expect("load session")
                .expect("session record exists");
            let visible = record.status_visible_participants();
            assert_eq!(visible.len(), 1);
            assert_eq!(visible[0].handle.participant_id, "ash_detached");
            assert!(!visible[0].attached_client_present());
        });
    }

    #[test]
    #[serial_test::serial]
    fn status_visible_participants_include_awaiting_attention_orchestrator() {
        with_store(|store| {
            let participant =
                detached_orchestrator("codex", "sess_status_attention", "ash_detached");
            let parent = parked_parent(&participant);
            store
                .persist_orchestration_session(&parent)
                .expect("persist parent");
            store
                .persist_participant(&participant)
                .expect("persist participant");
            store
                .persist_inbox_item(&pending_inbox_item(
                    "sess_status_attention",
                    "item_attention",
                    DurableInboxItemKind::RuntimeAlert,
                ))
                .expect("persist pending inbox item");

            let record = store
                .load_session("sess_status_attention")
                .expect("load session")
                .expect("session record exists");
            assert_eq!(record.session.pending_inbox_count, 1);
            assert_eq!(
                record.session.posture,
                OrchestrationSessionPosture::AwaitingAttention
            );
            let visible = record.status_visible_participants();
            assert_eq!(visible.len(), 1);
            assert_eq!(visible[0].handle.participant_id, "ash_detached");
        });
    }

    #[test]
    #[serial_test::serial]
    fn resolve_public_control_target_requires_exact_parent_linkage() {
        with_store(|store| {
            let participant = live_orchestrator("codex", "sess_stale_linkage", "ash_selected");
            let mut parent = active_parent(&participant);
            parent.orchestrator_agent_id = "claude_code".to_string();
            store
                .persist_orchestration_session(&parent)
                .expect("persist parent");
            store
                .persist_participant(&participant)
                .expect("persist participant");

            let err = store
                .resolve_public_control_target("sess_stale_linkage", PublicControlAction::Stop)
                .expect_err("mismatched active parent linkage must fail closed");
            assert!(err.to_string().contains("stale_linkage"));
        });
    }

    #[test]
    #[serial_test::serial]
    fn resolve_public_control_target_enforces_linux_first_world_posture() {
        with_store(|store| {
            let orchestrator = live_orchestrator("codex", "sess_world_posture", "ash_selected");
            let member = live_member("codex", "sess_world_posture", "ash_member", "ash_selected");
            let mut parent = active_parent(&orchestrator);
            parent.set_world_binding("world-17", 2);
            store
                .persist_orchestration_session(&parent)
                .expect("persist parent");
            store
                .persist_participant(&orchestrator)
                .expect("persist orchestrator");
            store.persist_participant(&member).expect("persist member");

            let result = store
                .resolve_public_control_target("sess_world_posture", PublicControlAction::Stop);

            #[cfg(target_os = "linux")]
            {
                let resolved = result.expect("linux should accept world-sensitive control posture");
                assert_eq!(
                    resolved.session.orchestration_session_id,
                    "sess_world_posture"
                );
            }

            #[cfg(not(target_os = "linux"))]
            {
                let err = result.expect_err(
                    "non-linux platforms must fail closed for world-sensitive control posture",
                );
                assert!(err.to_string().contains("unsupported_platform_or_posture"));
            }
        });
    }

    #[test]
    #[serial_test::serial]
    fn resolve_single_live_session_fails_closed_on_stale_active_handle_reference() {
        with_store(|store| {
            let selected = live_orchestrator("codex", "sess_live", "ash_selected");
            let mut parent = active_parent(&selected);
            parent.bind_active_session_handle("ash_missing");

            store
                .persist_orchestration_session(&parent)
                .expect("persist parent");
            store
                .persist_participant(&selected)
                .expect("persist selected participant");

            let err = store
                .resolve_single_live_session_for_agent("codex")
                .expect_err("stale active handle references must fail closed");
            assert!(err.to_string().contains(
                "active orchestration session sess_live references missing participant ash_missing"
            ));
        });
    }

    #[test]
    #[serial_test::serial]
    fn list_status_sessions_for_agent_includes_participant_visible_torn_roots_without_relaxing_strict_resolution(
    ) {
        with_store(|store| {
            let selected = live_orchestrator("codex", "sess_torn_visible", "ash_selected");
            let mut drifted_parent = active_parent(&selected);
            drifted_parent.orchestrator_agent_id = "claude_code".to_string();

            store
                .persist_orchestration_session(&drifted_parent)
                .expect("persist drifted parent");
            store
                .persist_participant(&selected)
                .expect("persist selected participant");

            let status_sessions = store
                .list_status_sessions_for_agent("codex")
                .expect("list status sessions");
            let torn = status_sessions
                .into_iter()
                .find(|record| record.orchestration_session_id() == "sess_torn_visible")
                .expect("status seam should retain participant-visible torn root");

            assert!(
                !torn.is_complete(),
                "status seam must degrade torn roots instead of authorizing control"
            );
            assert!(
                !torn.warnings.is_empty(),
                "status seam must preserve degraded warnings from build_session_record"
            );

            let err = store
                .resolve_single_live_session_for_agent("codex")
                .expect_err("strict selector must remain fail closed");
            assert!(err.to_string().contains(
                "live host-scoped orchestrator participant exists for agent codex without an active parent session"
            ));
        });
    }

    #[test]
    #[serial_test::serial]
    fn load_session_prefers_canonical_participant_over_conflicting_legacy_handle_fallback() {
        with_store(|store| {
            let canonical_orchestrator =
                live_orchestrator("codex", "sess_canonical_legacy", "ash_primary");
            let canonical_parent = active_parent(&canonical_orchestrator);

            write_legacy_handle_file(
                store,
                "ash_primary",
                "legacy",
                "sess_legacy_conflict",
                true,
                Some(json!({
                    "last_transition_at": "2026-04-24T18:31:00Z",
                    "internal": {
                        "latest_run_id": "run-legacy"
                    }
                })),
            );
            write_canonical_participant_file(store, &canonical_orchestrator);
            write_canonical_session_file(store, &canonical_parent);

            let session = store
                .load_session("sess_canonical_legacy")
                .expect("load session")
                .expect("session exists");
            let selected = session
                .participants
                .iter()
                .find(|participant| participant.handle.participant_id == "ash_primary")
                .expect("selected participant");

            assert_eq!(
                selected.handle.agent_id, "codex",
                "canonical participant must outrank conflicting legacy-handle fallback"
            );
            assert_eq!(
                selected.handle.orchestration_session_id, "sess_canonical_legacy",
                "canonical participant must keep the canonical parent linkage instead of drifting to the legacy handle"
            );
        });
    }

    #[test]
    #[serial_test::serial]
    fn load_session_prefers_flat_participant_over_conflicting_legacy_handle_fallback() {
        with_store(|store| {
            let flat_orchestrator = live_orchestrator("codex", "sess_flat_legacy", "ash_primary");
            let flat_parent = active_parent(&flat_orchestrator);

            write_legacy_handle_file(
                store,
                "ash_primary",
                "legacy",
                "sess_legacy_conflict",
                true,
                Some(json!({
                    "last_transition_at": "2026-04-24T18:31:00Z",
                    "internal": {
                        "latest_run_id": "run-legacy"
                    }
                })),
            );
            write_flat_participant_file(store, &flat_orchestrator);
            write_flat_session_file(store, &flat_parent);

            let session = store
                .load_session("sess_flat_legacy")
                .expect("load session")
                .expect("session exists");
            let selected = session
                .participants
                .iter()
                .find(|participant| participant.handle.participant_id == "ash_primary")
                .expect("selected participant");

            assert_eq!(
                selected.handle.agent_id, "codex",
                "flat compatibility participant must outrank conflicting legacy-handle fallback when the canonical child is absent"
            );
            assert_eq!(
                selected.handle.orchestration_session_id, "sess_flat_legacy",
                "flat compatibility participant must keep the flat parent linkage instead of drifting to the legacy handle"
            );
        });
    }

    #[test]
    #[serial_test::serial]
    fn resolve_single_live_session_fails_closed_on_inactive_selected_participant() {
        with_store(|store| {
            let mut selected = live_orchestrator("codex", "sess_live", "ash_selected");
            selected.internal.ownership_valid = false;
            selected.internal.control_owner_retained = false;
            selected.internal.event_stream_active = false;
            selected.internal.completion_observer_retained = false;
            let parent = active_parent(&selected);

            store
                .persist_orchestration_session(&parent)
                .expect("persist parent");
            store
                .persist_participant(&selected)
                .expect("persist inactive selected participant");

            let err = store
                .resolve_single_live_session_for_agent("codex")
                .expect_err("inactive selected participant must fail closed");
            assert!(err.to_string().contains(
                "active orchestration session sess_live references inactive participant ash_selected"
            ));
        });
    }

    #[test]
    #[serial_test::serial]
    fn persist_participant_rejects_stale_live_snapshot_after_terminal_snapshot() {
        with_store(|store| {
            let live =
                live_orchestrator("codex", "sess_stale_participant", "ash_stale_participant");
            let mut invalidated = live.clone();
            invalidated.transition_state(AgentRuntimeSessionState::Invalidated);
            invalidated.mark_terminal_state("attached control exited");

            store
                .persist_participant(&invalidated)
                .expect("persist invalidated participant");
            store
                .persist_participant(&live)
                .expect("reject stale live participant snapshot");

            let loaded = store
                .load_participant("ash_stale_participant")
                .expect("load participant")
                .expect("participant exists");
            assert_eq!(loaded.handle.state, AgentRuntimeSessionState::Invalidated);
            assert!(loaded.internal.terminal_observed_at.is_some());
        });
    }

    #[test]
    #[serial_test::serial]
    fn persist_orchestration_session_rejects_stale_active_snapshot_after_terminal_snapshot() {
        with_store(|store| {
            let participant = live_orchestrator("codex", "sess_stale_parent", "ash_stale_parent");
            let active = active_parent(&participant);
            let mut invalidated = active.clone();
            invalidated.transition_state(OrchestrationSessionState::Invalidated);
            invalidated.mark_terminal("attached control exited");

            store
                .persist_orchestration_session(&invalidated)
                .expect("persist invalidated parent");
            store
                .persist_orchestration_session(&active)
                .expect("reject stale active parent snapshot");

            let loaded = store
                .load_orchestration_session("sess_stale_parent")
                .expect("load orchestration session")
                .expect("orchestration session exists");
            assert_eq!(loaded.state, OrchestrationSessionState::Invalidated);
            assert!(loaded.closed_at.is_some());
        });
    }

    #[test]
    #[serial_test::serial]
    fn set_world_binding_rejects_stale_parent_after_terminal_snapshot() {
        with_store(|store| {
            let participant =
                live_orchestrator("codex", "sess_stale_set_binding", "ash_stale_set_binding");
            let active = active_parent(&participant);
            store
                .persist_orchestration_session(&active)
                .expect("persist active parent");

            let mut stale = active.clone();
            let stale_before = stale.clone();
            let mut invalidated = active.clone();
            invalidated.transition_state(OrchestrationSessionState::Invalidated);
            invalidated.mark_terminal("attached control exited");
            store
                .persist_orchestration_session(&invalidated)
                .expect("persist invalidated parent");

            let err = store
                .set_orchestration_session_world_binding(&mut stale, "world-stale", 9)
                .expect_err("stale binding set must fail closed");
            assert!(err
                .to_string()
                .contains("stale_world_binding_session_snapshot"));
            assert_eq!(stale, stale_before, "failed set must not mutate the caller");

            let loaded = store
                .load_orchestration_session("sess_stale_set_binding")
                .expect("load parent")
                .expect("parent exists");
            assert_eq!(loaded, invalidated, "terminal parent must remain untouched");
        });
    }

    #[test]
    #[serial_test::serial]
    fn clear_world_binding_rejects_stale_parent_after_terminal_snapshot() {
        with_store(|store| {
            let participant = live_orchestrator(
                "codex",
                "sess_stale_clear_binding",
                "ash_stale_clear_binding",
            );
            let mut active = active_parent(&participant);
            active.set_world_binding("world-current", 4);
            store
                .persist_orchestration_session(&active)
                .expect("persist bound active parent");

            let mut stale = active.clone();
            let stale_before = stale.clone();
            let mut invalidated = active.clone();
            invalidated.transition_state(OrchestrationSessionState::Invalidated);
            invalidated.mark_terminal("attached control exited");
            store
                .persist_orchestration_session(&invalidated)
                .expect("persist invalidated parent");

            let err = store
                .clear_orchestration_session_world_binding(&mut stale)
                .expect_err("stale binding clear must fail closed");
            assert!(err
                .to_string()
                .contains("stale_world_binding_session_snapshot"));
            assert_eq!(
                stale, stale_before,
                "failed clear must not mutate the caller"
            );

            let loaded = store
                .load_orchestration_session("sess_stale_clear_binding")
                .expect("load parent")
                .expect("parent exists");
            assert_eq!(loaded, invalidated, "terminal parent must remain untouched");
        });
    }

    #[test]
    #[serial_test::serial]
    fn world_binding_accepts_durable_continuity_enrichment_without_losing_it() {
        with_store(|store| {
            let participant = live_orchestrator(
                "codex",
                "sess_continuity_enrichment",
                "ash_continuity_enrichment",
            );
            let mut supplied = active_parent(&participant);
            supplied
                .host_attach_contract
                .as_mut()
                .expect("supplied attach contract")
                .continuity_uaa_session_id = None;
            let mut durable = supplied.clone();
            durable
                .host_attach_contract
                .as_mut()
                .expect("durable attach contract")
                .continuity_uaa_session_id = Some("thread-durable".to_string());
            store
                .persist_orchestration_session(&durable)
                .expect("persist continuity-enriched parent");

            store
                .set_orchestration_session_world_binding(&mut supplied, "world-current", 7)
                .expect("durable-only continuity enrichment is not a stale writer");

            assert_eq!(supplied.world_id.as_deref(), Some("world-current"));
            assert_eq!(supplied.world_generation, Some(7));
            assert_eq!(
                supplied
                    .host_attach_contract
                    .as_ref()
                    .and_then(|contract| contract.continuity_uaa_session_id.as_deref()),
                Some("thread-durable")
            );
            assert_eq!(
                store
                    .load_orchestration_session("sess_continuity_enrichment")
                    .expect("load continuity-enriched parent")
                    .expect("continuity-enriched parent exists"),
                supplied
            );
        });
    }

    #[test]
    #[serial_test::serial]
    fn world_binding_repair_accepts_timestamp_from_the_prior_binding_only() {
        with_store(|store| {
            let participant =
                live_orchestrator("codex", "sess_binding_repair", "ash_binding_repair");
            let mut supplied = active_parent(&participant);
            supplied.set_world_binding("world-old", 2);
            store
                .persist_orchestration_session(&supplied)
                .expect("persist old binding");

            let mut externally_repaired = supplied.clone();
            store
                .set_orchestration_session_world_binding(
                    &mut externally_repaired,
                    "world-current",
                    3,
                )
                .expect("persist external binding repair");

            store
                .set_orchestration_session_world_binding(&mut supplied, "world-current", 3)
                .expect("join the binding-only durable repair");

            assert_eq!(supplied.world_id.as_deref(), Some("world-current"));
            assert_eq!(supplied.world_generation, Some(3));
            assert_eq!(
                store
                    .load_orchestration_session("sess_binding_repair")
                    .expect("load binding repair")
                    .expect("binding repair exists"),
                supplied
            );
        });
    }

    #[test]
    #[serial_test::serial]
    fn world_binding_rejects_other_stale_attach_contract_changes() {
        with_store(|store| {
            let participant = live_orchestrator(
                "codex",
                "sess_stale_attach_contract",
                "ash_stale_attach_contract",
            );
            let mut current = active_parent(&participant);
            current
                .host_attach_contract
                .as_mut()
                .expect("current attach contract")
                .continuity_uaa_session_id = Some("thread-current".to_string());
            store
                .persist_orchestration_session(&current)
                .expect("persist current attach contract");

            let mut stale = current.clone();
            let stale_contract = stale
                .host_attach_contract
                .as_mut()
                .expect("stale attach contract");
            stale_contract.continuity_uaa_session_id = None;
            stale_contract.capabilities.session_resume = false;
            let stale_before = stale.clone();

            let err = store
                .set_orchestration_session_world_binding(&mut stale, "world-stale", 8)
                .expect_err("other stale attach-contract changes must fail closed");
            assert!(err
                .to_string()
                .contains("stale_world_binding_session_snapshot"));
            assert_eq!(stale, stale_before, "failed set must not mutate caller");
            assert_eq!(
                store
                    .load_orchestration_session("sess_stale_attach_contract")
                    .expect("load current attach contract")
                    .expect("current attach contract exists"),
                current
            );
        });
    }

    #[test]
    #[serial_test::serial]
    fn persist_orchestration_session_rejects_touched_allocating_regression_after_active() {
        with_store(|store| {
            let participant =
                live_orchestrator("codex", "sess_regressed_parent", "ash_regressed_parent");
            let active = active_parent(&participant);
            let mut allocating = OrchestrationSessionRecord::new(
                "sess_regressed_parent".to_string(),
                "trace_session".to_string(),
                "/workspace".to_string(),
                &participant,
                HostAttachContract::from_manifest_for_test(&participant),
            );

            store
                .persist_orchestration_session(&active)
                .expect("persist active parent");
            allocating.touch_active();
            store
                .persist_orchestration_session(&allocating)
                .expect("reject touched allocating regression");

            let loaded = store
                .load_orchestration_session("sess_regressed_parent")
                .expect("load orchestration session")
                .expect("orchestration session exists");
            assert_eq!(loaded.state, OrchestrationSessionState::Active);
            assert_eq!(
                loaded.active_session_handle_id.as_deref(),
                Some("ash_regressed_parent")
            );
        });
    }

    #[cfg(unix)]
    #[test]
    #[serial_test::serial]
    fn path_hardening_ignores_symlinked_and_non_regular_entries() {
        use std::os::unix::fs::symlink;

        with_store(|store| {
            fs::create_dir_all(store.sessions_dir()).expect("create sessions dir");
            fs::create_dir_all(store.participants_dir()).expect("create participants dir");

            let symlink_participant = live_orchestrator("codex", "sess_symlink", "ash_symlink");
            let real_root = store.substrate_home.join("real-session-root");
            fs::create_dir_all(real_root.join("participants")).expect("create real session root");
            fs::write(
                real_root.join("session.json"),
                serde_json::to_vec_pretty(&active_parent(&symlink_participant))
                    .expect("serialize symlinked session"),
            )
            .expect("write real session json");
            fs::write(
                real_root.join("participants/ash_symlink.json"),
                serde_json::to_vec_pretty(&symlink_participant)
                    .expect("serialize symlinked participant"),
            )
            .expect("write real participant json");
            symlink(&real_root, store.sessions_dir().join("sess_symlink"))
                .expect("symlink canonical root");

            let external_participant_path = store.substrate_home.join("real-participant.json");
            fs::write(
                &external_participant_path,
                serde_json::to_vec_pretty(&symlink_participant)
                    .expect("serialize external participant"),
            )
            .expect("write external participant");
            symlink(
                &external_participant_path,
                store.participants_dir().join("ash_symlink.json"),
            )
            .expect("symlink participant");

            fs::create_dir_all(store.sessions_dir().join("ignored.json"))
                .expect("create non-regular session entry");
            fs::create_dir_all(store.participants_dir().join("ignored.json"))
                .expect("create non-regular participant entry");

            assert!(
                store
                    .load_orchestration_session("sess_symlink")
                    .expect("load symlinked session")
                    .is_none(),
                "symlinked canonical session roots must be ignored"
            );
            assert!(
                store
                    .load_participant("ash_symlink")
                    .expect("load symlinked participant")
                    .is_none(),
                "symlinked participant entries must be ignored"
            );
            assert!(
                store
                    .list_sessions()
                    .expect("list sessions")
                    .into_iter()
                    .all(|record| record.orchestration_session_id() != "sess_symlink"),
                "symlinked canonical roots must not be promoted into discovery"
            );
        });
    }

    #[test]
    #[serial_test::serial]
    fn validation_helper_rejects_malformed_participant_json() {
        with_store(|store| {
            fs::create_dir_all(store.participants_dir()).expect("create participants dir");
            let payload = json!({
                "participant_id": "ash_bad",
                "orchestration_session_id": "sess_bad",
                "agent_id": "codex",
                "backend_id": "cli:codex",
                "role": "member",
                "protocol": "substrate.agent.session",
                "execution": { "scope": "world" },
                "state": "ready",
                "opened_at": "2026-04-24T18:30:00Z",
                "last_transition_at": "2026-04-24T18:30:00Z",
                "world_id": "world-17",
                "world_generation": 1,
                "internal": {
                    "resolved_agent_kind": "codex",
                    "resolved_binary_path": "/usr/bin/codex",
                    "shell_owner_pid": std::process::id(),
                    "lease_token": "lease_bad",
                    "uaa_session_id": "uaa_session",
                    "cancel_supported": true,
                    "control_owner_retained": true,
                    "event_stream_active": true,
                    "completion_observer_retained": true,
                    "ownership_mode": "member_runtime",
                    "ownership_valid": true
                }
            });
            fs::write(
                store.participant_path("ash_bad"),
                serde_json::to_vec_pretty(&payload).expect("serialize payload"),
            )
            .expect("write malformed participant");

            let err = store
                .load_participant("ash_bad")
                .expect_err("malformed participant must fail validation");
            assert!(!err.to_string().is_empty());
        });
    }

    #[test]
    #[serial_test::serial]
    fn resolve_internal_world_dispatch_caller_returns_authoritative_orchestrator() {
        with_store(|store| {
            let orchestrator = live_orchestrator("codex", "sess_dispatch", "orch_dispatch");
            let mut parent = active_parent(&orchestrator);
            parent.set_world_binding("world-17", 2);

            let member = live_member(
                "codex_world",
                "sess_dispatch",
                "member_dispatch",
                "orch_dispatch",
            );

            store
                .persist_orchestration_session(&parent)
                .expect("persist session");
            store
                .persist_participant(&orchestrator)
                .expect("persist orchestrator");
            store.persist_participant(&member).expect("persist member");

            let resolved = store
                .resolve_internal_world_dispatch_caller("sess_dispatch", "orch_dispatch")
                .expect("resolve authoritative caller");

            assert_eq!(resolved.session.orchestration_session_id, "sess_dispatch");
            assert_eq!(
                resolved.caller_participant.participant_id(),
                "orch_dispatch"
            );
            assert_eq!(resolved.caller_participant.handle.role, ORCHESTRATOR_ROLE);
            assert_eq!(
                resolved.caller_participant.handle.execution.scope,
                AgentExecutionScope::Host
            );
        });
    }

    #[test]
    #[serial_test::serial]
    fn resolve_internal_world_dispatch_caller_rejects_non_authoritative_participant() {
        with_store(|store| {
            let orchestrator = live_orchestrator("codex", "sess_dispatch", "orch_dispatch");
            let mut parent = active_parent(&orchestrator);
            parent.set_world_binding("world-17", 2);

            let member = live_member(
                "codex_world",
                "sess_dispatch",
                "member_dispatch",
                "orch_dispatch",
            );

            store
                .persist_orchestration_session(&parent)
                .expect("persist session");
            store
                .persist_participant(&orchestrator)
                .expect("persist orchestrator");
            store.persist_participant(&member).expect("persist member");

            let err = store
                .resolve_internal_world_dispatch_caller("sess_dispatch", "member_dispatch")
                .expect_err("member caller must fail closed");

            assert_eq!(
                err.to_string(),
                "caller_not_authoritative: orchestration session sess_dispatch authoritative orchestrator participant is orch_dispatch not member_dispatch"
            );
        });
    }

    #[test]
    #[serial_test::serial]
    fn resolve_internal_world_dispatch_caller_rejects_cross_session_participant() {
        with_store(|store| {
            let orchestrator_a = live_orchestrator("codex", "sess_dispatch_a", "orch_dispatch_a");
            let mut parent_a = active_parent(&orchestrator_a);
            parent_a.set_world_binding("world-17", 2);

            let orchestrator_b =
                live_orchestrator("claude_code", "sess_dispatch_b", "orch_dispatch_b");
            let mut parent_b = active_parent(&orchestrator_b);
            parent_b.set_world_binding("world-17", 2);

            store
                .persist_orchestration_session(&parent_a)
                .expect("persist session a");
            store
                .persist_participant(&orchestrator_a)
                .expect("persist orchestrator a");
            store
                .persist_orchestration_session(&parent_b)
                .expect("persist session b");
            store
                .persist_participant(&orchestrator_b)
                .expect("persist orchestrator b");

            let err = store
                .resolve_internal_world_dispatch_caller("sess_dispatch_a", "orch_dispatch_b")
                .expect_err("cross-session caller must fail closed");

            assert_eq!(
                err.to_string(),
                "caller_not_authoritative: orchestration session sess_dispatch_a authoritative orchestrator participant is orch_dispatch_a not orch_dispatch_b"
            );
        });
    }

    #[test]
    #[serial_test::serial]
    fn resolve_internal_continue_world_dispatch_target_returns_exact_retained_worker() {
        with_store(|store| {
            let orchestrator = live_orchestrator("codex", "sess_continue", "orch_continue");
            let mut parent = active_parent(&orchestrator);
            parent.set_world_binding("world-17", 2);

            let member = live_member(
                "codex_world",
                "sess_continue",
                "ash_continue",
                "orch_continue",
            );

            store
                .persist_orchestration_session(&parent)
                .expect("persist session");
            store
                .persist_participant(&orchestrator)
                .expect("persist orchestrator");
            store.persist_participant(&member).expect("persist member");

            let resolved = store
                .resolve_internal_continue_world_dispatch_target(
                    "sess_continue",
                    "orch_continue",
                    "ash_continue",
                    "cli:codex_world",
                )
                .expect("resolve exact retained continue target");

            assert_eq!(
                resolved.caller_participant.participant_id(),
                "orch_continue"
            );
            assert_eq!(resolved.target_participant.participant_id(), "ash_continue");
            assert_eq!(
                resolved
                    .target_participant
                    .handle
                    .orchestrator_participant_id
                    .as_deref(),
                Some("orch_continue")
            );
        });
    }

    #[test]
    #[serial_test::serial]
    fn resolve_internal_continue_world_dispatch_target_accepts_successor_authoritative_caller_for_retained_worker(
    ) {
        with_store(|store| {
            let mut launch_orchestrator =
                live_orchestrator("codex", "sess_continue", "orch_continue_launch");
            let mut successor =
                live_orchestrator("codex", "sess_continue", "orch_continue_successor");
            successor.handle.resumed_from_participant_id = Some("orch_continue_launch".to_string());
            successor.handle.resumed_from_session_handle_id =
                Some("orch_continue_launch".to_string());
            launch_orchestrator.mark_client_detached("successor attached");

            let member = live_member(
                "codex_world",
                "sess_continue",
                "ash_continue",
                "orch_continue_launch",
            );

            let mut parent = active_parent(&launch_orchestrator);
            parent.set_world_binding("world-17", 2);
            parent.bind_active_session_handle("orch_continue_successor".to_string());

            store
                .persist_orchestration_session(&parent)
                .expect("persist session");
            store
                .persist_participant(&launch_orchestrator)
                .expect("persist launch orchestrator");
            store
                .persist_participant(&successor)
                .expect("persist successor orchestrator");
            store.persist_participant(&member).expect("persist member");

            let resolved = store
                .resolve_internal_continue_world_dispatch_target(
                    "sess_continue",
                    "orch_continue_successor",
                    "ash_continue",
                    "cli:codex_world",
                )
                .expect("resolve retained continue target through authoritative successor");

            assert_eq!(
                resolved.caller_participant.participant_id(),
                "orch_continue_successor"
            );
            assert_eq!(resolved.target_participant.participant_id(), "ash_continue");
            assert_eq!(
                resolved
                    .target_participant
                    .handle
                    .orchestrator_participant_id
                    .as_deref(),
                Some("orch_continue_launch")
            );
        });
    }

    #[test]
    #[serial_test::serial]
    fn resolve_internal_continue_world_dispatch_target_rejects_backend_mismatch() {
        with_store(|store| {
            let orchestrator = live_orchestrator("codex", "sess_continue", "orch_continue");
            let mut parent = active_parent(&orchestrator);
            parent.set_world_binding("world-17", 2);

            let member = live_member(
                "codex_world",
                "sess_continue",
                "ash_continue",
                "orch_continue",
            );

            store
                .persist_orchestration_session(&parent)
                .expect("persist session");
            store
                .persist_participant(&orchestrator)
                .expect("persist orchestrator");
            store.persist_participant(&member).expect("persist member");

            let err = store
                .resolve_internal_continue_world_dispatch_target(
                    "sess_continue",
                    "orch_continue",
                    "ash_continue",
                    "cli:other_world",
                )
                .expect_err("backend drift must fail closed");

            assert_eq!(
                err.to_string(),
                "backend_mismatch: orchestration session sess_continue retained worker ash_continue backend is cli:codex_world not cli:other_world"
            );
        });
    }

    #[test]
    #[serial_test::serial]
    fn resolve_internal_continue_world_dispatch_target_rejects_world_binding_drift() {
        with_store(|store| {
            let orchestrator = live_orchestrator("codex", "sess_continue", "orch_continue");
            let mut parent = active_parent(&orchestrator);
            parent.set_world_binding("world-17", 2);

            let mut member = live_member(
                "codex_world",
                "sess_continue",
                "ash_continue",
                "orch_continue",
            );
            member.handle.world_generation = Some(3);

            store
                .persist_orchestration_session(&parent)
                .expect("persist session");
            store
                .persist_participant(&orchestrator)
                .expect("persist orchestrator");
            store.persist_participant(&member).expect("persist member");

            let err = store
                .resolve_internal_continue_world_dispatch_target(
                    "sess_continue",
                    "orch_continue",
                    "ash_continue",
                    "cli:codex_world",
                )
                .expect_err("world binding drift must fail closed");

            assert_eq!(
                err.to_string(),
                "world_binding_mismatch: orchestration session sess_continue retained worker ash_continue no longer matches the authoritative world binding"
            );
        });
    }

    #[test]
    #[serial_test::serial]
    fn resolve_internal_continue_world_dispatch_target_accepts_retained_worker_after_owner_pid_exit(
    ) {
        with_store(|store| {
            let orchestrator = live_orchestrator("codex", "sess_continue", "orch_continue");
            let mut parent = active_parent(&orchestrator);
            parent.set_world_binding("world-17", 2);

            let mut member = live_member(
                "codex_world",
                "sess_continue",
                "ash_continue",
                "orch_continue",
            );
            member.internal.shell_owner_pid = 999_999_999;

            store
                .persist_orchestration_session(&parent)
                .expect("persist session");
            store
                .persist_participant(&orchestrator)
                .expect("persist orchestrator");
            store.persist_participant(&member).expect("persist member");

            let resolved = store
                .resolve_internal_continue_world_dispatch_target(
                    "sess_continue",
                    "orch_continue",
                    "ash_continue",
                    "cli:codex_world",
                )
                .expect("exact retained continue should not depend on owner PID liveness");

            assert_eq!(
                resolved.caller_participant.participant_id(),
                "orch_continue"
            );
            assert_eq!(resolved.target_participant.participant_id(), "ash_continue");
        });
    }

    #[test]
    #[serial_test::serial]
    fn resolve_internal_continue_world_dispatch_target_accepts_parked_resumable_retained_worker() {
        with_store(|store| {
            let orchestrator =
                detached_orchestrator("codex", "sess_continue_parked", "orch_continue_parked");
            let mut parent = parked_parent(&orchestrator);
            parent.set_world_binding("world-17", 2);

            let mut member = live_member(
                "codex_world",
                "sess_continue_parked",
                "ash_continue_parked",
                "orch_continue_parked",
            );
            member.release_runtime_ownership();
            member.internal.shell_owner_pid = 999_999_999;

            store
                .persist_orchestration_session(&parent)
                .expect("persist session");
            store
                .persist_participant(&orchestrator)
                .expect("persist orchestrator");
            store.persist_participant(&member).expect("persist member");

            let resolved = store
                .resolve_internal_continue_world_dispatch_target(
                    "sess_continue_parked",
                    "orch_continue_parked",
                    "ash_continue_parked",
                    "cli:codex_world",
                )
                .expect("parked resumable retained worker should remain continue-routable");

            assert_eq!(
                resolved.caller_participant.participant_id(),
                "orch_continue_parked"
            );
            assert_eq!(
                resolved.target_participant.participant_id(),
                "ash_continue_parked"
            );
            assert!(!resolved.target_participant.is_authoritative_live());
        });
    }

    #[test]
    #[serial_test::serial]
    fn prepare_internal_continue_approval_response_obligation_closeout_preserves_pending_state() {
        with_store(|store| {
            let orchestrator = live_orchestrator("codex", "sess_continue", "orch_continue");
            let mut parent = active_parent(&orchestrator);
            parent.set_world_binding("world-17", 2);
            let member = live_member(
                "codex_world",
                "sess_continue",
                "ash_continue",
                "orch_continue",
            );

            store
                .persist_orchestration_session(&parent)
                .expect("persist session");
            store
                .persist_participant(&orchestrator)
                .expect("persist orchestrator");
            store.persist_participant(&member).expect("persist member");

            let obligation = pending_continue_approval_obligation(
                "sess_continue",
                "obl_approval",
                "ash_continue",
            );
            store
                .persist_obligation(&obligation)
                .expect("persist approval obligation");

            let resolved_target = store
                .resolve_internal_continue_world_dispatch_target(
                    "sess_continue",
                    "orch_continue",
                    "ash_continue",
                    "cli:codex_world",
                )
                .expect("resolve continue target");
            let closeout = store
                .prepare_internal_continue_approval_response_obligation_closeout(
                    &resolved_target,
                    &WorkerContinueApprovalResponsePayloadV1 {
                        approval_obligation_id: "obl_approval".to_string(),
                        decision: ApprovalResponseDecisionV1::Approve,
                        thread_id: None,
                    },
                )
                .expect("prepare approval closeout");

            assert_eq!(
                closeout,
                PreparedInternalApprovalResponseObligationCloseout {
                    orchestration_session_id: "sess_continue".to_string(),
                    approval_obligation_id: "obl_approval".to_string(),
                    target_participant_id: "ash_continue".to_string(),
                    target_backend_id: "cli:codex_world".to_string(),
                    world_id: "world-17".to_string(),
                    world_generation: 2,
                    decision: ApprovalResponseDecisionV1::Approve,
                }
            );

            let persisted = store
                .load_obligation("sess_continue", "obl_approval")
                .expect("load persisted obligation")
                .expect("persisted obligation exists");
            assert_eq!(persisted.state, OrchestrationObligationState::Pending);
            assert_eq!(
                persisted.review_state,
                OrchestrationObligationReviewState::Unread
            );
            assert!(persisted.resolved_at.is_none());
            assert!(persisted.attention_required);
        });
    }

    #[test]
    #[serial_test::serial]
    fn prepare_internal_continue_clarification_response_obligation_closeout_preserves_pending_state(
    ) {
        with_store(|store| {
            let orchestrator = live_orchestrator("codex", "sess_continue", "orch_continue");
            let mut parent = active_parent(&orchestrator);
            parent.set_world_binding("world-17", 2);
            let member = live_member(
                "codex_world",
                "sess_continue",
                "ash_continue",
                "orch_continue",
            );

            store
                .persist_orchestration_session(&parent)
                .expect("persist session");
            store
                .persist_participant(&orchestrator)
                .expect("persist orchestrator");
            store.persist_participant(&member).expect("persist member");

            let obligation = pending_continue_follow_up_obligation(
                "sess_continue",
                "obl_follow_up",
                "ash_continue",
            );
            store
                .persist_obligation(&obligation)
                .expect("persist follow-up obligation");

            let resolved_target = store
                .resolve_internal_continue_world_dispatch_target(
                    "sess_continue",
                    "orch_continue",
                    "ash_continue",
                    "cli:codex_world",
                )
                .expect("resolve continue target");
            let closeout = store
                .prepare_internal_continue_clarification_response_obligation_closeout(
                    &resolved_target,
                    &WorkerContinueClarificationResponsePayloadV1 {
                        follow_up_obligation_id: "obl_follow_up".to_string(),
                        clarification_text: "Here is the missing detail.".to_string(),
                        thread_id: None,
                    },
                )
                .expect("prepare clarification closeout");

            assert_eq!(
                closeout,
                PreparedInternalClarificationResponseObligationCloseout {
                    orchestration_session_id: "sess_continue".to_string(),
                    follow_up_obligation_id: "obl_follow_up".to_string(),
                    target_participant_id: "ash_continue".to_string(),
                    target_backend_id: "cli:codex_world".to_string(),
                    world_id: "world-17".to_string(),
                    world_generation: 2,
                }
            );

            let persisted = store
                .load_obligation("sess_continue", "obl_follow_up")
                .expect("load persisted obligation")
                .expect("persisted obligation exists");
            assert_eq!(persisted.state, OrchestrationObligationState::Pending);
            assert_eq!(
                persisted.review_state,
                OrchestrationObligationReviewState::Unread
            );
            assert!(persisted.resolved_at.is_none());
            assert!(persisted.attention_required);
        });
    }

    #[test]
    #[serial_test::serial]
    fn prepare_internal_continue_approval_response_obligation_closeout_fails_closed_for_invalid_bindings(
    ) {
        with_store(|store| {
            let orchestrator = live_orchestrator("codex", "sess_continue", "orch_continue");
            let mut parent = active_parent(&orchestrator);
            parent.set_world_binding("world-17", 2);
            let member = live_member(
                "codex_world",
                "sess_continue",
                "ash_continue",
                "orch_continue",
            );

            store
                .persist_orchestration_session(&parent)
                .expect("persist session");
            store
                .persist_participant(&orchestrator)
                .expect("persist orchestrator");
            store.persist_participant(&member).expect("persist member");

            let resolved_target = store
                .resolve_internal_continue_world_dispatch_target(
                    "sess_continue",
                    "orch_continue",
                    "ash_continue",
                    "cli:codex_world",
                )
                .expect("resolve continue target");

            let missing = store
                .prepare_internal_continue_approval_response_obligation_closeout(
                    &resolved_target,
                    &WorkerContinueApprovalResponsePayloadV1 {
                        approval_obligation_id: "obl_missing".to_string(),
                        decision: ApprovalResponseDecisionV1::Approve,
                        thread_id: None,
                    },
                )
                .expect_err("missing approval obligation must fail closed");
            assert_eq!(
                missing.to_string(),
                "approval_obligation_not_found: orchestration session sess_continue has no approval obligation obl_missing"
            );

            let wrong_kind = pending_obligation(
                "sess_continue",
                "obl_wrong_kind",
                OrchestrationObligationKind::Blocked,
            );
            store
                .persist_obligation(&wrong_kind)
                .expect("persist wrong-kind obligation");
            let wrong_kind_err = store
                .prepare_internal_continue_approval_response_obligation_closeout(
                    &resolved_target,
                    &WorkerContinueApprovalResponsePayloadV1 {
                        approval_obligation_id: "obl_wrong_kind".to_string(),
                        decision: ApprovalResponseDecisionV1::Approve,
                        thread_id: None,
                    },
                )
                .expect_err("wrong-kind obligation must fail closed");
            assert_eq!(
                wrong_kind_err.to_string(),
                "approval_obligation_kind_mismatch: orchestration session sess_continue obligation obl_wrong_kind is not approval_required"
            );

            let mut resolved = pending_continue_approval_obligation(
                "sess_continue",
                "obl_resolved",
                "ash_continue",
            );
            resolved.mark_approval_response_closed(
                ApprovalObligationCloseoutDisposition::Resolve,
                Some("already handled".to_string()),
                Utc::now(),
            );
            store
                .persist_obligation(&resolved)
                .expect("persist resolved obligation");
            let resolved_err = store
                .prepare_internal_continue_approval_response_obligation_closeout(
                    &resolved_target,
                    &WorkerContinueApprovalResponsePayloadV1 {
                        approval_obligation_id: "obl_resolved".to_string(),
                        decision: ApprovalResponseDecisionV1::Approve,
                        thread_id: None,
                    },
                )
                .expect_err("resolved obligation must fail closed");
            assert_eq!(
                resolved_err.to_string(),
                "approval_obligation_already_resolved: orchestration session sess_continue approval obligation obl_resolved is already closed"
            );

            let other_session = live_orchestrator("codex", "sess_other", "orch_other");
            let mut other_parent = active_parent(&other_session);
            other_parent.set_world_binding("world-17", 2);
            store
                .persist_orchestration_session(&other_parent)
                .expect("persist other session");
            store
                .persist_participant(&other_session)
                .expect("persist other orchestrator");
            let cross_session = pending_continue_approval_obligation(
                "sess_other",
                "obl_cross_session",
                "ash_continue",
            );
            store
                .persist_obligation(&cross_session)
                .expect("persist cross-session obligation");
            let cross_session_err = store
                .prepare_internal_continue_approval_response_obligation_closeout(
                    &resolved_target,
                    &WorkerContinueApprovalResponsePayloadV1 {
                        approval_obligation_id: "obl_cross_session".to_string(),
                        decision: ApprovalResponseDecisionV1::Approve,
                        thread_id: None,
                    },
                )
                .expect_err("cross-session obligation must fail closed");
            assert_eq!(
                cross_session_err.to_string(),
                "approval_obligation_not_found: orchestration session sess_continue has no approval obligation obl_cross_session"
            );

            let wrong_target = pending_continue_approval_obligation(
                "sess_continue",
                "obl_wrong_target",
                "ash_other",
            );
            store
                .persist_obligation(&wrong_target)
                .expect("persist wrong-target obligation");
            let wrong_target_err = store
                .prepare_internal_continue_approval_response_obligation_closeout(
                    &resolved_target,
                    &WorkerContinueApprovalResponsePayloadV1 {
                        approval_obligation_id: "obl_wrong_target".to_string(),
                        decision: ApprovalResponseDecisionV1::Approve,
                        thread_id: None,
                    },
                )
                .expect_err("wrong-target obligation must fail closed");
            assert_eq!(
                wrong_target_err.to_string(),
                "approval_obligation_target_mismatch: orchestration session sess_continue approval obligation obl_wrong_target does not bind retained worker ash_continue"
            );
        });
    }

    #[test]
    #[serial_test::serial]
    fn prepare_internal_continue_clarification_response_obligation_closeout_fails_closed_for_invalid_bindings(
    ) {
        with_store(|store| {
            let orchestrator = live_orchestrator("codex", "sess_continue", "orch_continue");
            let mut parent = active_parent(&orchestrator);
            parent.set_world_binding("world-17", 2);
            let member = live_member(
                "codex_world",
                "sess_continue",
                "ash_continue",
                "orch_continue",
            );

            store
                .persist_orchestration_session(&parent)
                .expect("persist session");
            store
                .persist_participant(&orchestrator)
                .expect("persist orchestrator");
            store.persist_participant(&member).expect("persist member");

            let resolved_target = store
                .resolve_internal_continue_world_dispatch_target(
                    "sess_continue",
                    "orch_continue",
                    "ash_continue",
                    "cli:codex_world",
                )
                .expect("resolve continue target");

            let missing = store
                .prepare_internal_continue_clarification_response_obligation_closeout(
                    &resolved_target,
                    &WorkerContinueClarificationResponsePayloadV1 {
                        follow_up_obligation_id: "obl_missing".to_string(),
                        clarification_text: "missing".to_string(),
                        thread_id: None,
                    },
                )
                .expect_err("missing follow-up obligation must fail closed");
            assert_eq!(
                missing.to_string(),
                "follow_up_obligation_not_found: orchestration session sess_continue has no follow-up obligation obl_missing"
            );

            let wrong_kind = pending_obligation(
                "sess_continue",
                "obl_wrong_kind",
                OrchestrationObligationKind::Blocked,
            );
            store
                .persist_obligation(&wrong_kind)
                .expect("persist wrong-kind obligation");
            let wrong_kind_err = store
                .prepare_internal_continue_clarification_response_obligation_closeout(
                    &resolved_target,
                    &WorkerContinueClarificationResponsePayloadV1 {
                        follow_up_obligation_id: "obl_wrong_kind".to_string(),
                        clarification_text: "wrong kind".to_string(),
                        thread_id: None,
                    },
                )
                .expect_err("wrong-kind obligation must fail closed");
            assert_eq!(
                wrong_kind_err.to_string(),
                "follow_up_obligation_kind_mismatch: orchestration session sess_continue obligation obl_wrong_kind is not follow_up_required"
            );

            let mut resolved = pending_continue_follow_up_obligation(
                "sess_continue",
                "obl_resolved",
                "ash_continue",
            );
            resolved.mark_clarification_response_closed(
                Some("already clarified".to_string()),
                Utc::now(),
            );
            store
                .persist_obligation(&resolved)
                .expect("persist resolved obligation");
            let resolved_err = store
                .prepare_internal_continue_clarification_response_obligation_closeout(
                    &resolved_target,
                    &WorkerContinueClarificationResponsePayloadV1 {
                        follow_up_obligation_id: "obl_resolved".to_string(),
                        clarification_text: "resolved".to_string(),
                        thread_id: None,
                    },
                )
                .expect_err("resolved obligation must fail closed");
            assert_eq!(
                resolved_err.to_string(),
                "follow_up_obligation_already_resolved: orchestration session sess_continue follow-up obligation obl_resolved is already closed"
            );

            let other_session = live_orchestrator("codex", "sess_other", "orch_other");
            let mut other_parent = active_parent(&other_session);
            other_parent.set_world_binding("world-17", 2);
            store
                .persist_orchestration_session(&other_parent)
                .expect("persist other session");
            store
                .persist_participant(&other_session)
                .expect("persist other orchestrator");
            let cross_session = pending_continue_follow_up_obligation(
                "sess_other",
                "obl_cross_session",
                "ash_continue",
            );
            store
                .persist_obligation(&cross_session)
                .expect("persist cross-session obligation");
            let cross_session_err = store
                .prepare_internal_continue_clarification_response_obligation_closeout(
                    &resolved_target,
                    &WorkerContinueClarificationResponsePayloadV1 {
                        follow_up_obligation_id: "obl_cross_session".to_string(),
                        clarification_text: "cross session".to_string(),
                        thread_id: None,
                    },
                )
                .expect_err("cross-session obligation must fail closed");
            assert_eq!(
                cross_session_err.to_string(),
                "follow_up_obligation_not_found: orchestration session sess_continue has no follow-up obligation obl_cross_session"
            );

            let wrong_target = pending_continue_follow_up_obligation(
                "sess_continue",
                "obl_wrong_target",
                "ash_other",
            );
            store
                .persist_obligation(&wrong_target)
                .expect("persist wrong-target obligation");
            let wrong_target_err = store
                .prepare_internal_continue_clarification_response_obligation_closeout(
                    &resolved_target,
                    &WorkerContinueClarificationResponsePayloadV1 {
                        follow_up_obligation_id: "obl_wrong_target".to_string(),
                        clarification_text: "wrong target".to_string(),
                        thread_id: None,
                    },
                )
                .expect_err("wrong-target obligation must fail closed");
            assert_eq!(
                wrong_target_err.to_string(),
                "follow_up_obligation_target_mismatch: orchestration session sess_continue follow-up obligation obl_wrong_target does not bind retained worker ash_continue"
            );
        });
    }

    #[test]
    #[serial_test::serial]
    fn close_prepared_internal_continue_approval_response_obligation_durably_closes_after_delivery_proof(
    ) {
        with_store(|store| {
            for (suffix, decision, note, expected_review_state, expected_compat_state) in [
                (
                    "approve",
                    ApprovalResponseDecisionV1::Approve,
                    "approved by host",
                    OrchestrationObligationReviewState::Resolved,
                    DurableInboxItemState::Acknowledged,
                ),
                (
                    "deny",
                    ApprovalResponseDecisionV1::Deny,
                    "denied by host",
                    OrchestrationObligationReviewState::Dismissed,
                    DurableInboxItemState::Dismissed,
                ),
            ] {
                let session_id = format!("sess_continue_{suffix}");
                let orchestrator =
                    live_orchestrator("codex", &session_id, &format!("orch_{suffix}"));
                let mut parent = parked_parent(&orchestrator);
                parent.set_world_binding("world-17", 2);
                let member = live_member(
                    "codex_world",
                    &session_id,
                    &format!("ash_{suffix}"),
                    &format!("orch_{suffix}"),
                );

                store
                    .persist_orchestration_session(&parent)
                    .expect("persist session");
                store
                    .persist_participant(&orchestrator)
                    .expect("persist orchestrator");
                store.persist_participant(&member).expect("persist member");

                let obligation = pending_continue_approval_obligation(
                    &session_id,
                    &format!("obl_{suffix}"),
                    &format!("ash_{suffix}"),
                );
                store
                    .persist_obligation(&obligation)
                    .expect("persist approval obligation");

                let resolved_target = store
                    .resolve_internal_continue_world_dispatch_target(
                        &session_id,
                        &format!("orch_{suffix}"),
                        &format!("ash_{suffix}"),
                        "cli:codex_world",
                    )
                    .expect("resolve continue target");
                let closeout = store
                    .prepare_internal_continue_approval_response_obligation_closeout(
                        &resolved_target,
                        &WorkerContinueApprovalResponsePayloadV1 {
                            approval_obligation_id: format!("obl_{suffix}"),
                            decision,
                            thread_id: None,
                        },
                    )
                    .expect("prepare approval closeout");
                let closeout_started_at = Utc::now();
                let closed = store
                    .close_prepared_internal_continue_approval_response_obligation(
                        &closeout,
                        Some(note.to_string()),
                    )
                    .expect("close prepared approval obligation");
                let closeout_finished_at = Utc::now();

                assert_eq!(closed.state, OrchestrationObligationState::Resolved);
                assert_eq!(closed.review_state, expected_review_state);
                assert!(!closed.attention_required);
                assert_eq!(closed.resolution_note.as_deref(), Some(note));
                assert!(
                    closed.resolved_at.is_some_and(|resolved_at| {
                        resolved_at >= closeout_started_at && resolved_at <= closeout_finished_at
                    }),
                    "resolved_at must be minted by state-store closeout"
                );
                let closed_resolved_at = closed.resolved_at.expect("resolved_at set");
                assert_eq!(closed.updated_at, closed_resolved_at);

                let persisted = store
                    .load_obligation(&session_id, &format!("obl_{suffix}"))
                    .expect("load persisted obligation")
                    .expect("persisted obligation exists");
                assert_eq!(persisted.state, OrchestrationObligationState::Resolved);
                assert_eq!(persisted.review_state, expected_review_state);
                assert!(!persisted.attention_required);
                assert_eq!(persisted.resolution_note.as_deref(), Some(note));
                assert_eq!(persisted.resolved_at, Some(closed_resolved_at));
                assert_eq!(persisted.updated_at, closed.updated_at);

                let compat_item = store
                    .load_inbox_item(&session_id, &format!("obl_{suffix}"))
                    .expect("load compatibility inbox item")
                    .expect("compatibility item exists");
                assert_eq!(compat_item.kind, DurableInboxItemKind::ApprovalRequired);
                assert_eq!(compat_item.state, expected_compat_state);
                assert_eq!(compat_item.message.as_deref(), Some(note));
                assert_eq!(compat_item.resolved_at, Some(closed_resolved_at));

                let settled_session = store
                    .load_orchestration_session(&session_id)
                    .expect("load settled session")
                    .expect("settled session exists");
                assert_eq!(settled_session.pending_inbox_count, 0);
            }
        });
    }

    #[test]
    #[serial_test::serial]
    fn close_prepared_internal_continue_approval_response_obligation_fails_closed_for_stale_delivery_proof(
    ) {
        with_store(|store| {
            let orchestrator = live_orchestrator("codex", "sess_continue_stale", "orch_stale");
            let mut parent = parked_parent(&orchestrator);
            parent.set_world_binding("world-17", 2);
            let member = live_member(
                "codex_world",
                "sess_continue_stale",
                "ash_stale",
                "orch_stale",
            );

            store
                .persist_orchestration_session(&parent)
                .expect("persist session");
            store
                .persist_participant(&orchestrator)
                .expect("persist orchestrator");
            store.persist_participant(&member).expect("persist member");

            let obligation = pending_continue_approval_obligation(
                "sess_continue_stale",
                "obl_stale",
                "ash_stale",
            );
            store
                .persist_obligation(&obligation)
                .expect("persist approval obligation");

            let resolved_target = store
                .resolve_internal_continue_world_dispatch_target(
                    "sess_continue_stale",
                    "orch_stale",
                    "ash_stale",
                    "cli:codex_world",
                )
                .expect("resolve continue target");
            let closeout = store
                .prepare_internal_continue_approval_response_obligation_closeout(
                    &resolved_target,
                    &WorkerContinueApprovalResponsePayloadV1 {
                        approval_obligation_id: "obl_stale".to_string(),
                        decision: ApprovalResponseDecisionV1::Approve,
                        thread_id: None,
                    },
                )
                .expect("prepare approval closeout");

            let mut resolved = store
                .load_obligation("sess_continue_stale", "obl_stale")
                .expect("load persisted obligation")
                .expect("persisted obligation exists");
            resolved.mark_approval_response_closed(
                ApprovalObligationCloseoutDisposition::Resolve,
                Some("closed elsewhere".to_string()),
                Utc::now(),
            );
            store
                .persist_obligation(&resolved)
                .expect("persist resolved obligation");

            let err = store
                .close_prepared_internal_continue_approval_response_obligation(
                    &closeout,
                    Some("approved by host".to_string()),
                )
                .expect_err("stale delivery proof must fail closed");
            assert_eq!(
                err.to_string(),
                "approval_obligation_already_resolved: orchestration session sess_continue_stale approval obligation obl_stale is already closed"
            );
        });
    }

    #[test]
    #[serial_test::serial]
    fn close_prepared_internal_continue_clarification_response_obligation_durably_closes_after_delivery_proof(
    ) {
        with_store(|store| {
            let orchestrator = live_orchestrator("codex", "sess_continue_clarify", "orch_clarify");
            let mut parent = parked_parent(&orchestrator);
            parent.set_world_binding("world-17", 2);
            let member = live_member(
                "codex_world",
                "sess_continue_clarify",
                "ash_clarify",
                "orch_clarify",
            );

            store
                .persist_orchestration_session(&parent)
                .expect("persist session");
            store
                .persist_participant(&orchestrator)
                .expect("persist orchestrator");
            store.persist_participant(&member).expect("persist member");

            let obligation = pending_continue_follow_up_obligation(
                "sess_continue_clarify",
                "obl_follow_up",
                "ash_clarify",
            );
            store
                .persist_obligation(&obligation)
                .expect("persist follow-up obligation");

            let resolved_target = store
                .resolve_internal_continue_world_dispatch_target(
                    "sess_continue_clarify",
                    "orch_clarify",
                    "ash_clarify",
                    "cli:codex_world",
                )
                .expect("resolve continue target");
            let closeout = store
                .prepare_internal_continue_clarification_response_obligation_closeout(
                    &resolved_target,
                    &WorkerContinueClarificationResponsePayloadV1 {
                        follow_up_obligation_id: "obl_follow_up".to_string(),
                        clarification_text: "Here are the details you requested.".to_string(),
                        thread_id: None,
                    },
                )
                .expect("prepare clarification closeout");
            let closeout_started_at = Utc::now();
            let closed = store
                .close_prepared_internal_continue_clarification_response_obligation(
                    &closeout,
                    Some("clarification delivered by host".to_string()),
                )
                .expect("close prepared clarification obligation");
            let closeout_finished_at = Utc::now();

            assert_eq!(closed.state, OrchestrationObligationState::Resolved);
            assert_eq!(
                closed.review_state,
                OrchestrationObligationReviewState::Resolved
            );
            assert!(!closed.attention_required);
            assert_eq!(
                closed.resolution_note.as_deref(),
                Some("clarification delivered by host")
            );
            assert!(
                closed.resolved_at.is_some_and(|resolved_at| {
                    resolved_at >= closeout_started_at && resolved_at <= closeout_finished_at
                }),
                "resolved_at must be minted by state-store closeout"
            );
            let closed_resolved_at = closed.resolved_at.expect("resolved_at set");
            assert_eq!(closed.updated_at, closed_resolved_at);

            let persisted = store
                .load_obligation("sess_continue_clarify", "obl_follow_up")
                .expect("load persisted obligation")
                .expect("persisted obligation exists");
            assert_eq!(persisted.state, OrchestrationObligationState::Resolved);
            assert_eq!(
                persisted.review_state,
                OrchestrationObligationReviewState::Resolved
            );
            assert!(!persisted.attention_required);
            assert_eq!(
                persisted.resolution_note.as_deref(),
                Some("clarification delivered by host")
            );
            assert_eq!(persisted.resolved_at, Some(closed_resolved_at));
            assert_eq!(persisted.updated_at, closed.updated_at);

            let compat_item = store
                .load_inbox_item("sess_continue_clarify", "obl_follow_up")
                .expect("load compatibility inbox item")
                .expect("compatibility item exists");
            assert_eq!(compat_item.kind, DurableInboxItemKind::FollowUpMessage);
            assert_eq!(compat_item.state, DurableInboxItemState::Dismissed);
            assert_eq!(
                compat_item.message.as_deref(),
                Some("clarification delivered by host")
            );
            assert_eq!(compat_item.resolved_at, Some(closed_resolved_at));

            let settled_session = store
                .load_orchestration_session("sess_continue_clarify")
                .expect("load settled session")
                .expect("settled session exists");
            assert_eq!(settled_session.pending_inbox_count, 0);
        });
    }

    #[test]
    #[serial_test::serial]
    fn close_prepared_internal_continue_clarification_response_obligation_fails_closed_for_stale_delivery_proof(
    ) {
        with_store(|store| {
            let orchestrator =
                live_orchestrator("codex", "sess_continue_clarify_stale", "orch_stale");
            let mut parent = parked_parent(&orchestrator);
            parent.set_world_binding("world-17", 2);
            let member = live_member(
                "codex_world",
                "sess_continue_clarify_stale",
                "ash_stale",
                "orch_stale",
            );

            store
                .persist_orchestration_session(&parent)
                .expect("persist session");
            store
                .persist_participant(&orchestrator)
                .expect("persist orchestrator");
            store.persist_participant(&member).expect("persist member");

            let obligation = pending_continue_follow_up_obligation(
                "sess_continue_clarify_stale",
                "obl_stale",
                "ash_stale",
            );
            store
                .persist_obligation(&obligation)
                .expect("persist follow-up obligation");

            let resolved_target = store
                .resolve_internal_continue_world_dispatch_target(
                    "sess_continue_clarify_stale",
                    "orch_stale",
                    "ash_stale",
                    "cli:codex_world",
                )
                .expect("resolve continue target");
            let closeout = store
                .prepare_internal_continue_clarification_response_obligation_closeout(
                    &resolved_target,
                    &WorkerContinueClarificationResponsePayloadV1 {
                        follow_up_obligation_id: "obl_stale".to_string(),
                        clarification_text: "stale".to_string(),
                        thread_id: None,
                    },
                )
                .expect("prepare clarification closeout");

            let mut resolved = store
                .load_obligation("sess_continue_clarify_stale", "obl_stale")
                .expect("load persisted obligation")
                .expect("persisted obligation exists");
            resolved.mark_clarification_response_closed(
                Some("closed elsewhere".to_string()),
                Utc::now(),
            );
            store
                .persist_obligation(&resolved)
                .expect("persist resolved obligation");

            let err = store
                .close_prepared_internal_continue_clarification_response_obligation(
                    &closeout,
                    Some("clarification delivered by host".to_string()),
                )
                .expect_err("stale delivery proof must fail closed");
            assert_eq!(
                err.to_string(),
                "follow_up_obligation_already_resolved: orchestration session sess_continue_clarify_stale follow-up obligation obl_stale is already closed"
            );
        });
    }

    #[test]
    #[serial_test::serial]
    fn resolved_approval_required_obligations_keep_compatibility_ack_vs_dismiss_scoped_to_closeout()
    {
        with_store(|store| {
            for (suffix, disposition, expected_review_state, expected_compat_state, note) in [
                (
                    "approve",
                    ApprovalObligationCloseoutDisposition::Resolve,
                    OrchestrationObligationReviewState::Resolved,
                    DurableInboxItemState::Acknowledged,
                    "approved by host",
                ),
                (
                    "deny",
                    ApprovalObligationCloseoutDisposition::Dismiss,
                    OrchestrationObligationReviewState::Dismissed,
                    DurableInboxItemState::Dismissed,
                    "denied by host",
                ),
            ] {
                let session_id = format!("sess_approval_projection_{suffix}");
                let participant =
                    detached_orchestrator("codex", &session_id, &format!("ash_proj_{suffix}"));
                let parent = parked_parent(&participant);
                store
                    .persist_orchestration_session(&parent)
                    .expect("persist parent");

                let mut obligation = pending_obligation(
                    &session_id,
                    &format!("obl_{suffix}"),
                    OrchestrationObligationKind::ApprovalRequired,
                );
                obligation.mark_approval_response_closed(
                    disposition,
                    Some(note.to_string()),
                    Utc::now(),
                );
                store
                    .persist_obligation(&obligation)
                    .expect("persist resolved approval obligation");

                let compat_item = store
                    .load_inbox_item(&session_id, &format!("obl_{suffix}"))
                    .expect("load compatibility inbox item")
                    .expect("compatibility item exists");
                assert_eq!(compat_item.kind, DurableInboxItemKind::ApprovalRequired);
                assert_eq!(compat_item.state, expected_compat_state);
                assert_eq!(compat_item.message.as_deref(), Some(note));
                assert_eq!(compat_item.resolved_at, obligation.resolved_at);

                let persisted = store
                    .load_obligation(&session_id, &format!("obl_{suffix}"))
                    .expect("load persisted obligation")
                    .expect("persisted obligation exists");
                assert_eq!(persisted.review_state, expected_review_state);
            }
        });
    }

    #[test]
    #[serial_test::serial]
    fn resolve_internal_fork_world_dispatch_target_returns_exact_retained_source() {
        with_store(|store| {
            let orchestrator = live_orchestrator("codex", "sess_fork", "orch_fork");
            let mut parent = active_parent(&orchestrator);
            parent.set_world_binding("world-17", 2);

            let member = live_member("codex_world", "sess_fork", "ash_source", "orch_fork");

            store
                .persist_orchestration_session(&parent)
                .expect("persist session");
            store
                .persist_participant(&orchestrator)
                .expect("persist orchestrator");
            store.persist_participant(&member).expect("persist member");

            let resolved = store
                .resolve_internal_fork_world_dispatch_target(
                    "sess_fork",
                    "orch_fork",
                    "ash_source",
                    "cli:codex_world",
                )
                .expect("resolve exact retained fork source");

            assert_eq!(resolved.caller_participant.participant_id(), "orch_fork");
            assert_eq!(resolved.source_participant.participant_id(), "ash_source");
        });
    }

    #[test]
    #[serial_test::serial]
    fn resolve_internal_fork_world_dispatch_target_accepts_successor_authoritative_caller_after_retained_owner_exits(
    ) {
        with_store(|store| {
            let mut launch_orchestrator =
                live_orchestrator("codex", "sess_fork", "orch_fork_launch");
            let mut successor = live_orchestrator("codex", "sess_fork", "orch_fork_successor");
            successor.handle.resumed_from_participant_id = Some("orch_fork_launch".to_string());
            successor.handle.resumed_from_session_handle_id = Some("orch_fork_launch".to_string());
            launch_orchestrator.mark_client_detached("successor attached");

            let mut member =
                live_member("codex_world", "sess_fork", "ash_source", "orch_fork_launch");
            member.internal.shell_owner_pid = 999_999_999;

            let mut parent = active_parent(&launch_orchestrator);
            parent.set_world_binding("world-17", 2);
            parent.bind_active_session_handle("orch_fork_successor".to_string());

            store
                .persist_orchestration_session(&parent)
                .expect("persist session");
            store
                .persist_participant(&launch_orchestrator)
                .expect("persist launch orchestrator");
            store
                .persist_participant(&successor)
                .expect("persist successor orchestrator");
            store.persist_participant(&member).expect("persist member");

            let resolved = store
                .resolve_internal_fork_world_dispatch_target(
                    "sess_fork",
                    "orch_fork_successor",
                    "ash_source",
                    "cli:codex_world",
                )
                .expect("resolve retained fork source after retained owner exits");

            assert_eq!(
                resolved.caller_participant.participant_id(),
                "orch_fork_successor"
            );
            assert_eq!(resolved.source_participant.participant_id(), "ash_source");
            assert_eq!(
                resolved
                    .source_participant
                    .handle
                    .orchestrator_participant_id
                    .as_deref(),
                Some("orch_fork_launch")
            );
        });
    }

    #[test]
    #[serial_test::serial]
    fn resolve_internal_fork_world_dispatch_target_rejects_unrelated_retained_worker_lineage() {
        with_store(|store| {
            let mut launch_orchestrator =
                live_orchestrator("codex", "sess_fork", "orch_fork_launch");
            let mut successor = live_orchestrator("codex", "sess_fork", "orch_fork_successor");
            successor.handle.resumed_from_participant_id = Some("orch_fork_launch".to_string());
            successor.handle.resumed_from_session_handle_id = Some("orch_fork_launch".to_string());
            launch_orchestrator.mark_client_detached("successor attached");

            let unrelated_orchestrator =
                detached_orchestrator("codex", "sess_fork", "orch_fork_unrelated");
            let member = live_member(
                "codex_world",
                "sess_fork",
                "ash_source_unrelated",
                "orch_fork_unrelated",
            );

            let mut parent = active_parent(&launch_orchestrator);
            parent.set_world_binding("world-17", 2);
            parent.bind_active_session_handle("orch_fork_successor".to_string());

            store
                .persist_orchestration_session(&parent)
                .expect("persist session");
            store
                .persist_participant(&launch_orchestrator)
                .expect("persist launch orchestrator");
            store
                .persist_participant(&successor)
                .expect("persist successor orchestrator");
            store
                .persist_participant(&unrelated_orchestrator)
                .expect("persist unrelated orchestrator");
            store.persist_participant(&member).expect("persist member");

            let err = store
                .resolve_internal_fork_world_dispatch_target(
                    "sess_fork",
                    "orch_fork_successor",
                    "ash_source_unrelated",
                    "cli:codex_world",
                )
                .expect_err("unrelated retained lineage must fail closed");

            assert_eq!(
                err.to_string(),
                "stale_linkage: orchestration session sess_fork retained worker ash_source_unrelated is not linked to authoritative orchestrator orch_fork_successor"
            );
        });
    }

    #[test]
    #[serial_test::serial]
    fn resolve_internal_fork_world_dispatch_target_rejects_unresolved_authoritative_predecessor_lineage(
    ) {
        with_store(|store| {
            let mut successor = live_orchestrator("codex", "sess_fork", "orch_fork_successor");
            successor.handle.resumed_from_participant_id =
                Some("orch_fork_missing_launch".to_string());
            successor.handle.resumed_from_session_handle_id =
                Some("orch_fork_missing_launch".to_string());

            let member = live_member(
                "codex_world",
                "sess_fork",
                "ash_source",
                "orch_fork_missing_launch",
            );

            let mut parent = active_parent(&successor);
            parent.set_world_binding("world-17", 2);
            parent.bind_active_session_handle("orch_fork_successor".to_string());

            store
                .persist_orchestration_session(&parent)
                .expect("persist session");
            store
                .persist_participant(&successor)
                .expect("persist successor orchestrator");
            store.persist_participant(&member).expect("persist member");

            let err = store
                .resolve_internal_fork_world_dispatch_target(
                    "sess_fork",
                    "orch_fork_successor",
                    "ash_source",
                    "cli:codex_world",
                )
                .expect_err("unresolved predecessor lineage must fail closed");

            assert_eq!(
                err.to_string(),
                "stale_linkage: orchestration session sess_fork retained worker ash_source is not linked to authoritative orchestrator orch_fork_successor"
            );
        });
    }

    #[test]
    #[serial_test::serial]
    fn resolve_internal_continue_fork_command_dispatch_target_reuses_exact_retained_source() {
        with_store(|store| {
            let orchestrator = live_orchestrator("codex", "sess_fork", "orch_fork");
            let mut parent = active_parent(&orchestrator);
            parent.set_world_binding("world-17", 2);

            let member = live_member("codex_world", "sess_fork", "ash_source", "orch_fork");

            store
                .persist_orchestration_session(&parent)
                .expect("persist session");
            store
                .persist_participant(&orchestrator)
                .expect("persist orchestrator");
            store.persist_participant(&member).expect("persist member");

            let resolved = store
                .resolve_internal_continue_fork_command_dispatch_target(
                    "sess_fork",
                    "orch_fork",
                    "ash_source",
                    "cli:codex_world",
                )
                .expect("resolve exact retained continue fork-command source");

            assert_eq!(resolved.caller_participant.participant_id(), "orch_fork");
            assert_eq!(resolved.target_participant.participant_id(), "ash_source");
        });
    }

    #[test]
    #[serial_test::serial]
    fn resolve_internal_fork_world_dispatch_target_rejects_non_authoritative_caller() {
        with_store(|store| {
            let orchestrator = live_orchestrator("codex", "sess_fork", "orch_fork");
            let mut parent = active_parent(&orchestrator);
            parent.set_world_binding("world-17", 2);

            let member = live_member("codex_world", "sess_fork", "ash_source", "orch_fork");

            store
                .persist_orchestration_session(&parent)
                .expect("persist session");
            store
                .persist_participant(&orchestrator)
                .expect("persist orchestrator");
            store.persist_participant(&member).expect("persist member");

            let err = store
                .resolve_internal_fork_world_dispatch_target(
                    "sess_fork",
                    "ash_source",
                    "ash_source",
                    "cli:codex_world",
                )
                .expect_err("member caller must fail closed");

            assert_eq!(
                err.to_string(),
                "caller_not_authoritative: orchestration session sess_fork authoritative orchestrator participant is orch_fork not ash_source"
            );
        });
    }

    #[test]
    #[serial_test::serial]
    fn resolve_internal_fork_world_dispatch_target_rejects_cross_session_source() {
        with_store(|store| {
            let orchestrator_a = live_orchestrator("codex", "sess_fork_a", "orch_fork_a");
            let mut parent_a = active_parent(&orchestrator_a);
            parent_a.set_world_binding("world-17", 2);

            let orchestrator_b = live_orchestrator("codex", "sess_fork_b", "orch_fork_b");
            let mut parent_b = active_parent(&orchestrator_b);
            parent_b.set_world_binding("world-17", 2);

            let member_b = live_member("codex_world", "sess_fork_b", "ash_source_b", "orch_fork_b");

            store
                .persist_orchestration_session(&parent_a)
                .expect("persist session a");
            store
                .persist_participant(&orchestrator_a)
                .expect("persist orchestrator a");
            store
                .persist_orchestration_session(&parent_b)
                .expect("persist session b");
            store
                .persist_participant(&orchestrator_b)
                .expect("persist orchestrator b");
            store
                .persist_participant(&member_b)
                .expect("persist member b");

            let err = store
                .resolve_internal_fork_world_dispatch_target(
                    "sess_fork_a",
                    "orch_fork_a",
                    "ash_source_b",
                    "cli:codex_world",
                )
                .expect_err("cross-session source must fail closed");

            assert_eq!(
                err.to_string(),
                "target_not_in_session: orchestration session sess_fork_a has no exact retained worker ash_source_b"
            );
        });
    }

    #[test]
    #[serial_test::serial]
    fn resolve_internal_fork_world_dispatch_target_rejects_world_binding_drift() {
        with_store(|store| {
            let orchestrator = live_orchestrator("codex", "sess_fork", "orch_fork");
            let mut parent = active_parent(&orchestrator);
            parent.set_world_binding("world-17", 2);

            let mut member = live_member("codex_world", "sess_fork", "ash_source", "orch_fork");
            member.handle.world_generation = Some(3);

            store
                .persist_orchestration_session(&parent)
                .expect("persist session");
            store
                .persist_participant(&orchestrator)
                .expect("persist orchestrator");
            store.persist_participant(&member).expect("persist member");

            let err = store
                .resolve_internal_fork_world_dispatch_target(
                    "sess_fork",
                    "orch_fork",
                    "ash_source",
                    "cli:codex_world",
                )
                .expect_err("world binding drift must fail closed");

            assert_eq!(
                err.to_string(),
                "world_binding_mismatch: orchestration session sess_fork retained worker ash_source no longer matches the authoritative world binding"
            );
        });
    }

    #[test]
    #[serial_test::serial]
    fn resolve_internal_fork_world_dispatch_target_allows_non_terminal_retained_source_after_owner_exit(
    ) {
        with_store(|store| {
            let orchestrator = live_orchestrator("codex", "sess_fork", "orch_fork");
            let mut parent = active_parent(&orchestrator);
            parent.set_world_binding("world-17", 2);

            let mut member = live_member("codex_world", "sess_fork", "ash_source", "orch_fork");
            member.internal.shell_owner_pid = 999_999_999;

            store
                .persist_orchestration_session(&parent)
                .expect("persist session");
            store
                .persist_participant(&orchestrator)
                .expect("persist orchestrator");
            store.persist_participant(&member).expect("persist member");

            let resolved = store
                .resolve_internal_fork_world_dispatch_target(
                    "sess_fork",
                    "orch_fork",
                    "ash_source",
                    "cli:codex_world",
                )
                .expect("non-terminal retained source should remain forkable after owner exit");

            assert_eq!(resolved.caller_participant.participant_id(), "orch_fork");
            assert_eq!(resolved.source_participant.participant_id(), "ash_source");
        });
    }

    #[test]
    #[serial_test::serial]
    fn resolve_internal_fork_world_dispatch_target_rejects_terminal_source() {
        with_store(|store| {
            let orchestrator = live_orchestrator("codex", "sess_fork", "orch_fork");
            let mut parent = active_parent(&orchestrator);
            parent.set_world_binding("world-17", 2);

            let mut member = live_member("codex_world", "sess_fork", "ash_source", "orch_fork");
            member.mark_terminal_state("worker invalidated");
            member.transition_state(AgentRuntimeSessionState::Invalidated);

            store
                .persist_orchestration_session(&parent)
                .expect("persist session");
            store
                .persist_participant(&orchestrator)
                .expect("persist orchestrator");
            store.persist_participant(&member).expect("persist member");

            let err = store
                .resolve_internal_fork_world_dispatch_target(
                    "sess_fork",
                    "orch_fork",
                    "ash_source",
                    "cli:codex_world",
                )
                .expect_err("terminal source must fail closed");

            assert_eq!(
                err.to_string(),
                "target_already_terminal: orchestration session sess_fork retained worker ash_source is already terminal (invalidated)"
            );
        });
    }

    #[test]
    #[serial_test::serial]
    fn resolve_internal_continue_fork_command_dispatch_target_rejects_terminal_source() {
        with_store(|store| {
            let orchestrator = live_orchestrator("codex", "sess_fork", "orch_fork");
            let mut parent = active_parent(&orchestrator);
            parent.set_world_binding("world-17", 2);

            let mut member = live_member("codex_world", "sess_fork", "ash_source", "orch_fork");
            member.mark_terminal_state("worker invalidated");
            member.transition_state(AgentRuntimeSessionState::Invalidated);

            store
                .persist_orchestration_session(&parent)
                .expect("persist session");
            store
                .persist_participant(&orchestrator)
                .expect("persist orchestrator");
            store.persist_participant(&member).expect("persist member");

            let err = store
                .resolve_internal_continue_fork_command_dispatch_target(
                    "sess_fork",
                    "orch_fork",
                    "ash_source",
                    "cli:codex_world",
                )
                .expect_err("terminal exact source must fail closed for continue fork command");

            assert_eq!(
                err.to_string(),
                "target_already_terminal: orchestration session sess_fork retained worker ash_source is already terminal (invalidated)"
            );
        });
    }

    #[test]
    #[serial_test::serial]
    fn resolve_internal_continue_fork_command_dispatch_target_accepts_retained_source_after_owner_pid_exit(
    ) {
        with_store(|store| {
            let orchestrator = live_orchestrator("codex", "sess_fork", "orch_fork");
            let mut parent = active_parent(&orchestrator);
            parent.set_world_binding("world-17", 2);

            let mut member = live_member("codex_world", "sess_fork", "ash_source", "orch_fork");
            member.internal.shell_owner_pid = 999_999_999;

            store
                .persist_orchestration_session(&parent)
                .expect("persist session");
            store
                .persist_participant(&orchestrator)
                .expect("persist orchestrator");
            store.persist_participant(&member).expect("persist member");

            let resolved = store
                .resolve_internal_continue_fork_command_dispatch_target(
                    "sess_fork",
                    "orch_fork",
                    "ash_source",
                    "cli:codex_world",
                )
                .expect("continue-fork exact source should not depend on owner PID liveness");

            assert_eq!(resolved.caller_participant.participant_id(), "orch_fork");
            assert_eq!(resolved.target_participant.participant_id(), "ash_source");
        });
    }

    #[test]
    #[serial_test::serial]
    fn resolve_internal_continue_fork_command_dispatch_target_accepts_parked_resumable_source() {
        with_store(|store| {
            let orchestrator = detached_orchestrator("codex", "sess_fork_parked", "orch_fork");
            let mut parent = parked_parent(&orchestrator);
            parent.set_world_binding("world-17", 2);

            let mut member =
                live_member("codex_world", "sess_fork_parked", "ash_source", "orch_fork");
            member.release_runtime_ownership();
            member.internal.shell_owner_pid = 999_999_999;

            store
                .persist_orchestration_session(&parent)
                .expect("persist session");
            store
                .persist_participant(&orchestrator)
                .expect("persist orchestrator");
            store.persist_participant(&member).expect("persist member");

            let resolved = store
                .resolve_internal_continue_fork_command_dispatch_target(
                    "sess_fork_parked",
                    "orch_fork",
                    "ash_source",
                    "cli:codex_world",
                )
                .expect("parked resumable retained source should remain continue-routable");

            assert_eq!(resolved.caller_participant.participant_id(), "orch_fork");
            assert_eq!(resolved.target_participant.participant_id(), "ash_source");
            assert!(!resolved.target_participant.is_authoritative_live());
        });
    }

    #[test]
    #[serial_test::serial]
    fn resolved_internal_fork_world_dispatch_target_projects_explicit_child_lineage() {
        with_store(|store| {
            let orchestrator = live_orchestrator("codex", "sess_fork", "orch_fork");
            let mut parent = active_parent(&orchestrator);
            parent.set_world_binding("world-17", 2);

            let source = live_member("codex_world", "sess_fork", "ash_source", "orch_fork");

            store
                .persist_orchestration_session(&parent)
                .expect("persist session");
            store
                .persist_participant(&orchestrator)
                .expect("persist orchestrator");
            store.persist_participant(&source).expect("persist source");

            let resolved = store
                .resolve_internal_fork_world_dispatch_target(
                    "sess_fork",
                    "orch_fork",
                    "ash_source",
                    "cli:codex_world",
                )
                .expect("resolve exact retained fork source");

            let child = AgentRuntimeParticipantRecord::new_fork_child_participant(
                &descriptor("codex_world", AgentExecutionScope::World),
                AgentRuntimeForkParticipantInit {
                    orchestration_session_id: "sess_fork".to_string(),
                    participant_id: "ash_child".to_string(),
                    orchestrator_participant_id: "orch_fork".to_string(),
                    source_participant_id: "ash_source".to_string(),
                    world: parent
                        .authoritative_world_binding()
                        .expect("authoritative world binding"),
                    lease_token: "lease_child".to_string(),
                },
            )
            .expect("fork child participant");

            let lineage = resolved
                .project_child_lineage(&child)
                .expect("project child lineage");

            assert_eq!(lineage.orchestration_session_id, "sess_fork");
            assert_eq!(lineage.orchestrator_participant_id, "orch_fork");
            assert_eq!(lineage.source_participant_id, "ash_source");
            assert_eq!(lineage.child_participant_id, "ash_child");
            assert_eq!(lineage.world_id, "world-17");
            assert_eq!(lineage.world_generation, 2);
        });
    }

    #[test]
    #[serial_test::serial]
    fn resolved_internal_fork_world_dispatch_target_rejects_plain_spawn_lineage() {
        with_store(|store| {
            let orchestrator = live_orchestrator("codex", "sess_fork", "orch_fork");
            let mut parent = active_parent(&orchestrator);
            parent.set_world_binding("world-17", 2);

            let source = live_member("codex_world", "sess_fork", "ash_source", "orch_fork");

            store
                .persist_orchestration_session(&parent)
                .expect("persist session");
            store
                .persist_participant(&orchestrator)
                .expect("persist orchestrator");
            store.persist_participant(&source).expect("persist source");

            let resolved = store
                .resolve_internal_fork_world_dispatch_target(
                    "sess_fork",
                    "orch_fork",
                    "ash_source",
                    "cli:codex_world",
                )
                .expect("resolve exact retained fork source");

            let child = AgentRuntimeParticipantRecord::new_member_participant(
                &descriptor("codex_world", AgentExecutionScope::World),
                "sess_fork".to_string(),
                "ash_child".to_string(),
                "orch_fork".to_string(),
                Some("ash_source".to_string()),
                Some(
                    parent
                        .authoritative_world_binding()
                        .expect("authoritative world binding"),
                ),
                "lease_child".to_string(),
            )
            .expect("plain spawned child participant");

            let err = resolved
                .project_child_lineage(&child)
                .expect_err("plain spawn lineage must fail closed");

            assert_eq!(
                err.to_string(),
                "invalid_fork_lineage: orchestration session sess_fork child ash_child must point parent_participant_id at fork source ash_source"
            );
        });
    }

    #[test]
    #[serial_test::serial]
    fn resolve_internal_inspect_world_dispatch_target_returns_exact_live_retained_worker() {
        with_store(|store| {
            let orchestrator = live_orchestrator("codex", "sess_inspect", "orch_inspect");
            let mut parent = active_parent(&orchestrator);
            parent.set_world_binding("world-17", 2);

            let member = live_member("codex_world", "sess_inspect", "ash_inspect", "orch_inspect");

            store
                .persist_orchestration_session(&parent)
                .expect("persist session");
            store
                .persist_participant(&orchestrator)
                .expect("persist orchestrator");
            store.persist_participant(&member).expect("persist member");

            let resolved = store
                .resolve_internal_inspect_world_dispatch_target(
                    "sess_inspect",
                    "orch_inspect",
                    "ash_inspect",
                    "cli:codex_world",
                )
                .expect("resolve exact retained inspect target");

            assert_eq!(resolved.caller_participant.participant_id(), "orch_inspect");
            assert_eq!(resolved.target_participant.participant_id(), "ash_inspect");
        });
    }

    #[test]
    #[serial_test::serial]
    fn resolve_internal_inspect_world_dispatch_target_accepts_successor_authoritative_caller_for_retained_worker(
    ) {
        with_store(|store| {
            let mut launch_orchestrator =
                live_orchestrator("codex", "sess_inspect", "orch_inspect_launch");
            let mut successor =
                live_orchestrator("codex", "sess_inspect", "orch_inspect_successor");
            successor.handle.resumed_from_participant_id = Some("orch_inspect_launch".to_string());
            successor.handle.resumed_from_session_handle_id =
                Some("orch_inspect_launch".to_string());
            launch_orchestrator.mark_client_detached("successor attached");

            let member = live_member(
                "codex_world",
                "sess_inspect",
                "ash_inspect",
                "orch_inspect_launch",
            );

            let mut parent = active_parent(&launch_orchestrator);
            parent.set_world_binding("world-17", 2);
            parent.bind_active_session_handle("orch_inspect_successor".to_string());

            store
                .persist_orchestration_session(&parent)
                .expect("persist session");
            store
                .persist_participant(&launch_orchestrator)
                .expect("persist launch orchestrator");
            store
                .persist_participant(&successor)
                .expect("persist successor orchestrator");
            store.persist_participant(&member).expect("persist member");

            let resolved = store
                .resolve_internal_inspect_world_dispatch_target(
                    "sess_inspect",
                    "orch_inspect_successor",
                    "ash_inspect",
                    "cli:codex_world",
                )
                .expect("resolve retained inspect target through authoritative successor");

            assert_eq!(
                resolved.caller_participant.participant_id(),
                "orch_inspect_successor"
            );
            assert_eq!(resolved.target_participant.participant_id(), "ash_inspect");
            assert_eq!(
                resolved
                    .target_participant
                    .handle
                    .orchestrator_participant_id
                    .as_deref(),
                Some("orch_inspect_launch")
            );
        });
    }

    #[test]
    #[serial_test::serial]
    fn resolve_internal_inspect_world_dispatch_target_accepts_non_live_retained_worker() {
        with_store(|store| {
            let orchestrator = live_orchestrator("codex", "sess_inspect", "orch_inspect");
            let mut parent = active_parent(&orchestrator);
            parent.set_world_binding("world-17", 2);

            let mut member =
                live_member("codex_world", "sess_inspect", "ash_inspect", "orch_inspect");
            member.mark_terminal_state("worker exited");
            member.transition_state(AgentRuntimeSessionState::Invalidated);
            member.internal.shell_owner_pid = 999_999_999;

            store
                .persist_orchestration_session(&parent)
                .expect("persist session");
            store
                .persist_participant(&orchestrator)
                .expect("persist orchestrator");
            store.persist_participant(&member).expect("persist member");

            let resolved = store
                .resolve_internal_inspect_world_dispatch_target(
                    "sess_inspect",
                    "orch_inspect",
                    "ash_inspect",
                    "cli:codex_world",
                )
                .expect(
                    "inspect must accept exact retained workers even when not continue-routable",
                );

            assert_eq!(
                resolved.target_participant.handle.state,
                AgentRuntimeSessionState::Invalidated
            );
            assert!(!resolved.target_participant.is_authoritative_live());
        });
    }

    #[test]
    #[serial_test::serial]
    fn resolve_internal_inspect_world_dispatch_target_rejects_non_authoritative_caller() {
        with_store(|store| {
            let orchestrator = live_orchestrator("codex", "sess_inspect", "orch_inspect");
            let mut parent = active_parent(&orchestrator);
            parent.set_world_binding("world-17", 2);

            let member = live_member("codex_world", "sess_inspect", "ash_inspect", "orch_inspect");

            store
                .persist_orchestration_session(&parent)
                .expect("persist session");
            store
                .persist_participant(&orchestrator)
                .expect("persist orchestrator");
            store.persist_participant(&member).expect("persist member");

            let err = store
                .resolve_internal_inspect_world_dispatch_target(
                    "sess_inspect",
                    "ash_inspect",
                    "ash_inspect",
                    "cli:codex_world",
                )
                .expect_err("member caller must fail closed");

            assert_eq!(
                err.to_string(),
                "caller_not_authoritative: orchestration session sess_inspect authoritative orchestrator participant is orch_inspect not ash_inspect"
            );
        });
    }

    #[test]
    #[serial_test::serial]
    fn resolve_internal_inspect_world_dispatch_target_rejects_cross_session_target() {
        with_store(|store| {
            let orchestrator_a = live_orchestrator("codex", "sess_inspect_a", "orch_inspect_a");
            let mut parent_a = active_parent(&orchestrator_a);
            parent_a.set_world_binding("world-17", 2);

            let orchestrator_b = live_orchestrator("codex", "sess_inspect_b", "orch_inspect_b");
            let mut parent_b = active_parent(&orchestrator_b);
            parent_b.set_world_binding("world-17", 2);

            let member_b = live_member(
                "codex_world",
                "sess_inspect_b",
                "ash_inspect_b",
                "orch_inspect_b",
            );

            store
                .persist_orchestration_session(&parent_a)
                .expect("persist session a");
            store
                .persist_participant(&orchestrator_a)
                .expect("persist orchestrator a");
            store
                .persist_orchestration_session(&parent_b)
                .expect("persist session b");
            store
                .persist_participant(&orchestrator_b)
                .expect("persist orchestrator b");
            store
                .persist_participant(&member_b)
                .expect("persist member b");

            let err = store
                .resolve_internal_inspect_world_dispatch_target(
                    "sess_inspect_a",
                    "orch_inspect_a",
                    "ash_inspect_b",
                    "cli:codex_world",
                )
                .expect_err("cross-session target must fail closed");

            assert_eq!(
                err.to_string(),
                "target_not_in_session: orchestration session sess_inspect_a has no exact retained worker ash_inspect_b"
            );
        });
    }

    #[test]
    #[serial_test::serial]
    fn resolve_internal_inspect_world_dispatch_target_rejects_world_binding_drift() {
        with_store(|store| {
            let orchestrator = live_orchestrator("codex", "sess_inspect", "orch_inspect");
            let mut parent = active_parent(&orchestrator);
            parent.set_world_binding("world-17", 2);

            let mut member =
                live_member("codex_world", "sess_inspect", "ash_inspect", "orch_inspect");
            member.handle.world_generation = Some(3);

            store
                .persist_orchestration_session(&parent)
                .expect("persist session");
            store
                .persist_participant(&orchestrator)
                .expect("persist orchestrator");
            store.persist_participant(&member).expect("persist member");

            let err = store
                .resolve_internal_inspect_world_dispatch_target(
                    "sess_inspect",
                    "orch_inspect",
                    "ash_inspect",
                    "cli:codex_world",
                )
                .expect_err("world binding drift must fail closed");

            assert_eq!(
                err.to_string(),
                "world_binding_mismatch: orchestration session sess_inspect retained worker ash_inspect no longer matches the authoritative world binding"
            );
        });
    }

    #[test]
    #[serial_test::serial]
    fn resolved_internal_inspect_world_dispatch_target_projects_live_snapshot() {
        with_store(|store| {
            let orchestrator = live_orchestrator("codex", "sess_inspect", "orch_inspect");
            let mut parent = active_parent(&orchestrator);
            parent.set_world_binding("world-17", 2);

            let member = live_member("codex_world", "sess_inspect", "ash_inspect", "orch_inspect");

            store
                .persist_orchestration_session(&parent)
                .expect("persist session");
            store
                .persist_participant(&orchestrator)
                .expect("persist orchestrator");
            store.persist_participant(&member).expect("persist member");

            let resolved = store
                .resolve_internal_inspect_world_dispatch_target(
                    "sess_inspect",
                    "orch_inspect",
                    "ash_inspect",
                    "cli:codex_world",
                )
                .expect("resolve inspect target");
            let snapshot = resolved.project_snapshot();

            assert_eq!(snapshot.participant_state, AgentRuntimeSessionState::Ready);
            assert_eq!(snapshot.session_state, OrchestrationSessionState::Active);
            assert_eq!(
                snapshot.session_posture,
                OrchestrationSessionPosture::ActiveAttached
            );
            assert!(snapshot.authoritative_live);
            assert!(!snapshot.attention_required);
            assert_eq!(snapshot.parent_participant_id, None);
            assert_eq!(snapshot.resumed_from_participant_id, None);
        });
    }

    #[test]
    #[serial_test::serial]
    fn resolved_internal_inspect_world_dispatch_target_projects_attention_snapshot() {
        with_store(|store| {
            let orchestrator = detached_orchestrator("codex", "sess_inspect", "orch_inspect");
            let mut parent = parked_parent(&orchestrator);
            parent.set_world_binding("world-17", 2);
            parent.set_pending_inbox_count(2);

            let mut member =
                live_member("codex_world", "sess_inspect", "ash_inspect", "orch_inspect");
            member.handle.parent_participant_id = Some("ash_parent".to_string());
            member.handle.resumed_from_participant_id = Some("ash_prev".to_string());

            store
                .persist_orchestration_session(&parent)
                .expect("persist session");
            store
                .persist_participant(&orchestrator)
                .expect("persist orchestrator");
            store.persist_participant(&member).expect("persist member");

            let resolved = store
                .resolve_internal_inspect_world_dispatch_target(
                    "sess_inspect",
                    "orch_inspect",
                    "ash_inspect",
                    "cli:codex_world",
                )
                .expect("resolve inspect target");
            let snapshot = resolved.project_snapshot();

            assert_eq!(snapshot.participant_state, AgentRuntimeSessionState::Ready);
            assert_eq!(snapshot.session_state, OrchestrationSessionState::Active);
            assert_eq!(
                snapshot.session_posture,
                OrchestrationSessionPosture::AwaitingAttention
            );
            assert!(snapshot.authoritative_live);
            assert!(snapshot.attention_required);
            assert_eq!(
                snapshot.parent_participant_id.as_deref(),
                Some("ash_parent")
            );
            assert_eq!(
                snapshot.resumed_from_participant_id.as_deref(),
                Some("ash_prev")
            );

            let stored = store
                .load_session("sess_inspect")
                .expect("load session")
                .expect("session must remain persisted");
            assert_eq!(
                stored.session.posture,
                OrchestrationSessionPosture::AwaitingAttention
            );
            assert_eq!(stored.session.pending_inbox_count, 2);
        });
    }

    #[test]
    #[serial_test::serial]
    fn resolved_internal_inspect_world_dispatch_target_projects_invalidated_snapshot() {
        with_store(|store| {
            let orchestrator = live_orchestrator("codex", "sess_inspect", "orch_inspect");
            let mut parent = active_parent(&orchestrator);
            parent.set_world_binding("world-17", 2);

            let mut member =
                live_member("codex_world", "sess_inspect", "ash_inspect", "orch_inspect");
            member.mark_terminal_state("worker invalidated");
            member.transition_state(AgentRuntimeSessionState::Invalidated);

            store
                .persist_orchestration_session(&parent)
                .expect("persist session");
            store
                .persist_participant(&orchestrator)
                .expect("persist orchestrator");
            store.persist_participant(&member).expect("persist member");

            let resolved = store
                .resolve_internal_inspect_world_dispatch_target(
                    "sess_inspect",
                    "orch_inspect",
                    "ash_inspect",
                    "cli:codex_world",
                )
                .expect("resolve inspect target");
            let snapshot = resolved.project_snapshot();

            assert_eq!(
                snapshot.participant_state,
                AgentRuntimeSessionState::Invalidated
            );
            assert_eq!(snapshot.session_state, OrchestrationSessionState::Active);
            assert_eq!(
                snapshot.session_posture,
                OrchestrationSessionPosture::ActiveAttached
            );
            assert!(!snapshot.authoritative_live);
            assert!(!snapshot.attention_required);
        });
    }

    #[test]
    #[serial_test::serial]
    fn resolved_internal_inspect_world_dispatch_target_projects_terminal_snapshot() {
        with_store(|store| {
            let orchestrator = live_orchestrator("codex", "sess_inspect", "orch_inspect");
            let mut parent = active_parent(&orchestrator);
            parent.set_world_binding("world-17", 2);

            let mut member =
                live_member("codex_world", "sess_inspect", "ash_inspect", "orch_inspect");
            member.mark_terminal_state("worker stopped");
            member.transition_state(AgentRuntimeSessionState::Stopped);

            store
                .persist_orchestration_session(&parent)
                .expect("persist session");
            store
                .persist_participant(&orchestrator)
                .expect("persist orchestrator");
            store.persist_participant(&member).expect("persist member");

            let resolved = store
                .resolve_internal_inspect_world_dispatch_target(
                    "sess_inspect",
                    "orch_inspect",
                    "ash_inspect",
                    "cli:codex_world",
                )
                .expect("resolve inspect target");
            let snapshot = resolved.project_snapshot();

            assert_eq!(
                snapshot.participant_state,
                AgentRuntimeSessionState::Stopped
            );
            assert_eq!(snapshot.session_state, OrchestrationSessionState::Active);
            assert_eq!(
                snapshot.session_posture,
                OrchestrationSessionPosture::ActiveAttached
            );
            assert!(!snapshot.authoritative_live);
            assert!(!snapshot.attention_required);
        });
    }

    #[test]
    #[serial_test::serial]
    fn resolve_internal_stop_world_dispatch_target_returns_exact_live_retained_worker() {
        with_store(|store| {
            let orchestrator = live_orchestrator("codex", "sess_stop", "orch_stop");
            let mut parent = active_parent(&orchestrator);
            parent.set_world_binding("world-17", 2);

            let member = live_member("codex_world", "sess_stop", "ash_stop", "orch_stop");

            store
                .persist_orchestration_session(&parent)
                .expect("persist session");
            store
                .persist_participant(&orchestrator)
                .expect("persist orchestrator");
            store.persist_participant(&member).expect("persist member");

            let resolved = store
                .resolve_internal_stop_world_dispatch_target(
                    "sess_stop",
                    "orch_stop",
                    "ash_stop",
                    "cli:codex_world",
                )
                .expect("resolve exact retained stop target");

            assert_eq!(resolved.caller_participant.participant_id(), "orch_stop");
            assert_eq!(resolved.target_participant.participant_id(), "ash_stop");
        });
    }

    #[test]
    #[serial_test::serial]
    fn resolve_internal_stop_world_dispatch_target_recovery_harness_preserves_exact_target_under_stale_attached_truth(
    ) {
        with_store(|store| {
            let mut orchestrator = live_orchestrator("codex", "sess_stop", "orch_stop");
            orchestrator.internal.shell_owner_pid = 999_999_999;
            let mut parent = active_parent(&orchestrator);
            parent.set_world_binding("world-17", 2);

            let member = live_member("codex_world", "sess_stop", "ash_stop", "orch_stop");

            store
                .persist_orchestration_session(&parent)
                .expect("persist session");
            store
                .persist_participant(&orchestrator)
                .expect("persist stale attached orchestrator");
            store.persist_participant(&member).expect("persist member");

            let continuity = store
                .classify_hidden_owner_helper_launch_continuity("sess_stop", "orch_stop", true)
                .expect("classify stale attached continuity");
            assert_eq!(
                continuity,
                HiddenOwnerHelperLaunchContinuity::StaleAttachedTruth,
                "SPEC-64 recovery harness must pin recoverable stale attached truth independently from stop transport behavior"
            );

            let resolved = store
                .resolve_internal_stop_world_dispatch_target(
                    "sess_stop",
                    "orch_stop",
                    "ash_stop",
                    "cli:codex_world",
                )
                .expect("resolve exact retained stop target under stale attached truth");

            assert_eq!(resolved.orchestration_session_id(), "sess_stop");
            assert_eq!(resolved.caller_participant.participant_id(), "orch_stop");
            assert_eq!(resolved.target_participant.participant_id(), "ash_stop");
            assert_eq!(
                resolved.target_participant.handle.backend_id,
                "cli:codex_world"
            );
            assert_eq!(
                resolved.target_participant.handle.world_id.as_deref(),
                Some("world-17")
            );
            assert_eq!(resolved.target_participant.handle.world_generation, Some(2));
            assert_eq!(
                resolved
                    .target_participant
                    .handle
                    .orchestrator_participant_id
                    .as_deref(),
                Some("orch_stop")
            );
        });
    }

    #[test]
    #[serial_test::serial]
    fn project_refreshed_exact_target_rejects_target_missing_from_session_participant_set() {
        with_store(|store| {
            let orchestrator = live_orchestrator("codex", "sess_stop", "orch_stop");
            let mut parent = active_parent(&orchestrator);
            parent.set_world_binding("world-17", 2);

            let member = live_member("codex_world", "sess_stop", "ash_stop", "orch_stop");

            store
                .persist_orchestration_session(&parent)
                .expect("persist session");
            store
                .persist_participant(&orchestrator)
                .expect("persist orchestrator");
            store.persist_participant(&member).expect("persist member");

            let resolved = store
                .resolve_internal_stop_world_dispatch_target(
                    "sess_stop",
                    "orch_stop",
                    "ash_stop",
                    "cli:codex_world",
                )
                .expect("resolve exact retained stop target");

            let mut refreshed = store
                .load_session("sess_stop")
                .expect("load refreshed session")
                .expect("refreshed session");
            refreshed.session.active_session_handle_id = None;
            refreshed
                .session
                .mark_parked_resumable("owner detached before refreshed projection");
            refreshed
                .participants
                .retain(|participant| participant.participant_id() != "ash_stop");

            let err = resolved
                .project_refreshed_exact_target(refreshed)
                .expect_err("refreshed projection must fail when the exact target is absent from the session participant set");

            assert_eq!(
                err.to_string(),
                "target_not_in_session: orchestration session sess_stop has no exact retained worker ash_stop"
            );
        });
    }

    #[test]
    #[serial_test::serial]
    fn resolve_internal_stop_world_dispatch_target_accepts_successor_authoritative_caller_for_retained_worker(
    ) {
        with_store(|store| {
            let mut launch_orchestrator =
                live_orchestrator("codex", "sess_stop", "orch_stop_launch");
            let mut successor = live_orchestrator("codex", "sess_stop", "orch_stop_successor");
            successor.handle.resumed_from_participant_id = Some("orch_stop_launch".to_string());
            successor.handle.resumed_from_session_handle_id = Some("orch_stop_launch".to_string());
            launch_orchestrator.mark_client_detached("successor attached");

            let member = live_member("codex_world", "sess_stop", "ash_stop", "orch_stop_launch");

            let mut parent = active_parent(&launch_orchestrator);
            parent.set_world_binding("world-17", 2);
            parent.bind_active_session_handle("orch_stop_successor".to_string());

            store
                .persist_orchestration_session(&parent)
                .expect("persist session");
            store
                .persist_participant(&launch_orchestrator)
                .expect("persist launch orchestrator");
            store
                .persist_participant(&successor)
                .expect("persist successor orchestrator");
            store.persist_participant(&member).expect("persist member");

            let resolved = store
                .resolve_internal_stop_world_dispatch_target(
                    "sess_stop",
                    "orch_stop_successor",
                    "ash_stop",
                    "cli:codex_world",
                )
                .expect("resolve retained stop target through authoritative successor");

            assert_eq!(
                resolved.caller_participant.participant_id(),
                "orch_stop_successor"
            );
            assert_eq!(resolved.target_participant.participant_id(), "ash_stop");
            assert_eq!(
                resolved
                    .target_participant
                    .handle
                    .orchestrator_participant_id
                    .as_deref(),
                Some("orch_stop_launch")
            );
        });
    }

    #[test]
    #[serial_test::serial]
    fn resolve_internal_stop_world_dispatch_target_keeps_exact_lineage_distinct_from_owner_rebinding(
    ) {
        with_store(|store| {
            let mut launch_orchestrator =
                live_orchestrator("codex", "sess_stop", "orch_stop_launch");
            let mut parent = active_parent(&launch_orchestrator);
            parent.set_world_binding("world-17", 2);

            let member = live_member("codex_world", "sess_stop", "ash_stop", "orch_stop_launch");

            store
                .persist_orchestration_session(&parent)
                .expect("persist session");
            store
                .persist_participant(&launch_orchestrator)
                .expect("persist launch orchestrator");
            store.persist_participant(&member).expect("persist member");

            let initial = store
                .resolve_internal_stop_world_dispatch_target(
                    "sess_stop",
                    "orch_stop_launch",
                    "ash_stop",
                    "cli:codex_world",
                )
                .expect("resolve initial exact retained stop target");

            let mut successor = live_orchestrator("codex", "sess_stop", "orch_stop_successor");
            successor.handle.resumed_from_participant_id = Some("orch_stop_launch".to_string());
            successor.handle.resumed_from_session_handle_id = Some("orch_stop_launch".to_string());
            launch_orchestrator.mark_client_detached("successor attached");
            parent.bind_active_session_handle("orch_stop_successor".to_string());

            let mut rebound_member = store
                .load_participant("ash_stop")
                .expect("load retained worker before rebinding")
                .expect("retained worker before rebinding");
            rebound_member.handle.orchestrator_participant_id =
                Some("orch_stop_successor".to_string());

            store
                .persist_orchestration_session(&parent)
                .expect("persist rebound session");
            store
                .persist_participant(&launch_orchestrator)
                .expect("persist detached launch orchestrator");
            store
                .persist_participant(&successor)
                .expect("persist successor orchestrator");
            store
                .persist_participant(&rebound_member)
                .expect("persist rebound member lineage");

            let rebound = store
                .resolve_internal_stop_world_dispatch_target(
                    "sess_stop",
                    "orch_stop_successor",
                    "ash_stop",
                    "cli:codex_world",
                )
                .expect("resolve rebound stop target");

            let err = initial
                .ensure_exact_target_match(&rebound)
                .expect_err("owner rebinding must not silently rewrite retained lineage");

            assert_eq!(
                err.to_string(),
                "stale_linkage: orchestration session sess_stop retained worker ash_stop exact stop target lineage changed from orch_stop_launch to orch_stop_successor"
            );
        });
    }

    #[test]
    #[serial_test::serial]
    fn resolve_internal_stop_world_dispatch_target_rejects_stale_attached_host_owner_after_successor_attach(
    ) {
        with_store(|store| {
            let mut launch_orchestrator =
                live_orchestrator("codex", "sess_stop", "orch_stop_launch");
            let mut successor = live_orchestrator("codex", "sess_stop", "orch_stop_successor");
            successor.handle.resumed_from_participant_id = Some("orch_stop_launch".to_string());
            successor.handle.resumed_from_session_handle_id = Some("orch_stop_launch".to_string());
            launch_orchestrator.mark_client_detached("successor attached");

            let member = live_member("codex_world", "sess_stop", "ash_stop", "orch_stop_launch");

            let mut parent = active_parent(&launch_orchestrator);
            parent.set_world_binding("world-17", 2);
            parent.bind_active_session_handle("orch_stop_successor".to_string());

            store
                .persist_orchestration_session(&parent)
                .expect("persist session");
            store
                .persist_participant(&launch_orchestrator)
                .expect("persist launch orchestrator");
            store
                .persist_participant(&successor)
                .expect("persist successor orchestrator");
            store.persist_participant(&member).expect("persist member");

            let err = store
                .resolve_internal_stop_world_dispatch_target(
                    "sess_stop",
                    "orch_stop_launch",
                    "ash_stop",
                    "cli:codex_world",
                )
                .expect_err("stale attached-host owner must fail closed before stop delivery");

            assert_eq!(
                err.to_string(),
                "caller_not_authoritative: orchestration session sess_stop authoritative orchestrator participant is orch_stop_successor not orch_stop_launch"
            );

            let participant_after = store
                .load_participant("ash_stop")
                .expect("load retained worker after stale owner rejection")
                .expect("retained worker after stale owner rejection");
            assert_eq!(
                participant_after.handle.state,
                AgentRuntimeSessionState::Ready
            );
            assert!(
                participant_after.internal.termination_reason.is_none(),
                "pre-delivery stale owner rejection must not persist any stop closeout"
            );
        });
    }

    #[test]
    #[serial_test::serial]
    fn resolve_internal_stop_world_dispatch_target_rejects_attached_snapshot_without_active_owner_truth(
    ) {
        with_store(|store| {
            let orchestrator = live_orchestrator("codex", "sess_stop", "orch_stop");
            let member = live_member("codex_world", "sess_stop", "ash_stop", "orch_stop");

            let mut parent = active_parent(&orchestrator);
            parent.set_world_binding("world-17", 2);
            parent.active_session_handle_id = None;

            store
                .persist_orchestration_session(&parent)
                .expect("persist session");
            store
                .persist_participant(&orchestrator)
                .expect("persist attached orchestrator snapshot");
            store.persist_participant(&member).expect("persist member");

            let err = store
                .resolve_internal_stop_world_dispatch_target(
                    "sess_stop",
                    "orch_stop",
                    "ash_stop",
                    "cli:codex_world",
                )
                .expect_err(
                    "stop dispatch must fail closed when only an attached snapshot remains",
                );

            assert_eq!(
                err.to_string(),
                "stale_linkage: orchestration session sess_stop is missing authoritative orchestrator participant linkage"
            );
        });
    }

    #[test]
    #[serial_test::serial]
    fn resolve_internal_stop_world_dispatch_target_rejects_inactive_session_with_stale_stop_owner_linkage(
    ) {
        with_store(|store| {
            let orchestrator = live_orchestrator("codex", "sess_stop", "orch_stop");
            let member = live_member("codex_world", "sess_stop", "ash_stop", "orch_stop");

            let mut parent = active_parent(&orchestrator);
            parent.set_world_binding("world-17", 2);
            parent.transition_state(OrchestrationSessionState::Stopped);

            assert_eq!(parent.active_participant_id(), Some("orch_stop"));
            assert_eq!(parent.attached_participant_id(), None);

            store
                .persist_orchestration_session(&parent)
                .expect("persist terminal session");
            store
                .persist_participant(&orchestrator)
                .expect("persist stale stop owner");
            store.persist_participant(&member).expect("persist member");

            let err = store
                .resolve_internal_stop_world_dispatch_target(
                    "sess_stop",
                    "orch_stop",
                    "ash_stop",
                    "cli:codex_world",
                )
                .expect_err("terminalized sessions must fail closed before stop-owner resolution");

            assert_eq!(
                err.to_string(),
                "missing_active_parent: orchestration session sess_stop is not active"
            );
        });
    }

    #[test]
    #[serial_test::serial]
    fn resolve_internal_stop_world_dispatch_target_accepts_non_authoritative_live_worker() {
        with_store(|store| {
            let orchestrator = live_orchestrator("codex", "sess_stop", "orch_stop");
            let mut parent = active_parent(&orchestrator);
            parent.set_world_binding("world-17", 2);

            let mut member = live_member("codex_world", "sess_stop", "ash_stop", "orch_stop");
            member.release_runtime_ownership();

            store
                .persist_orchestration_session(&parent)
                .expect("persist session");
            store
                .persist_participant(&orchestrator)
                .expect("persist orchestrator");
            store.persist_participant(&member).expect("persist member");

            let resolved = store
                .resolve_internal_stop_world_dispatch_target(
                    "sess_stop",
                    "orch_stop",
                    "ash_stop",
                    "cli:codex_world",
                )
                .expect("stop must accept exact retained workers even when not continue-routable");

            assert_eq!(resolved.target_participant.participant_id(), "ash_stop");
            assert_eq!(
                resolved.target_participant.handle.state,
                AgentRuntimeSessionState::Ready
            );
            assert!(!resolved.target_participant.is_authoritative_live());
        });
    }

    #[test]
    #[serial_test::serial]
    fn resolve_internal_stop_world_dispatch_target_rejects_non_authoritative_caller() {
        with_store(|store| {
            let orchestrator = live_orchestrator("codex", "sess_stop", "orch_stop");
            let mut parent = active_parent(&orchestrator);
            parent.set_world_binding("world-17", 2);

            let member = live_member("codex_world", "sess_stop", "ash_stop", "orch_stop");

            store
                .persist_orchestration_session(&parent)
                .expect("persist session");
            store
                .persist_participant(&orchestrator)
                .expect("persist orchestrator");
            store.persist_participant(&member).expect("persist member");

            let err = store
                .resolve_internal_stop_world_dispatch_target(
                    "sess_stop",
                    "ash_stop",
                    "ash_stop",
                    "cli:codex_world",
                )
                .expect_err("member caller must fail closed");

            assert_eq!(
                err.to_string(),
                "caller_not_authoritative: orchestration session sess_stop authoritative orchestrator participant is orch_stop not ash_stop"
            );
        });
    }

    #[test]
    #[serial_test::serial]
    fn resolve_internal_stop_world_dispatch_target_rejects_cross_session_target() {
        with_store(|store| {
            let orchestrator_a = live_orchestrator("codex", "sess_stop_a", "orch_stop_a");
            let mut parent_a = active_parent(&orchestrator_a);
            parent_a.set_world_binding("world-17", 2);

            let orchestrator_b = live_orchestrator("codex", "sess_stop_b", "orch_stop_b");
            let mut parent_b = active_parent(&orchestrator_b);
            parent_b.set_world_binding("world-17", 2);

            let member_b = live_member("codex_world", "sess_stop_b", "ash_stop_b", "orch_stop_b");

            store
                .persist_orchestration_session(&parent_a)
                .expect("persist session a");
            store
                .persist_participant(&orchestrator_a)
                .expect("persist orchestrator a");
            store
                .persist_orchestration_session(&parent_b)
                .expect("persist session b");
            store
                .persist_participant(&orchestrator_b)
                .expect("persist orchestrator b");
            store
                .persist_participant(&member_b)
                .expect("persist member b");

            let err = store
                .resolve_internal_stop_world_dispatch_target(
                    "sess_stop_a",
                    "orch_stop_a",
                    "ash_stop_b",
                    "cli:codex_world",
                )
                .expect_err("cross-session target must fail closed");

            assert_eq!(
                err.to_string(),
                "target_not_in_session: orchestration session sess_stop_a has no exact retained worker ash_stop_b"
            );
        });
    }

    #[test]
    #[serial_test::serial]
    fn resolve_internal_stop_world_dispatch_target_rejects_backend_mismatch() {
        with_store(|store| {
            let orchestrator = live_orchestrator("codex", "sess_stop", "orch_stop");
            let mut parent = active_parent(&orchestrator);
            parent.set_world_binding("world-17", 2);

            let member = live_member("codex_world", "sess_stop", "ash_stop", "orch_stop");

            store
                .persist_orchestration_session(&parent)
                .expect("persist session");
            store
                .persist_participant(&orchestrator)
                .expect("persist orchestrator");
            store.persist_participant(&member).expect("persist member");

            let err = store
                .resolve_internal_stop_world_dispatch_target(
                    "sess_stop",
                    "orch_stop",
                    "ash_stop",
                    "cli:other_world",
                )
                .expect_err("backend drift must fail closed");

            assert_eq!(
                err.to_string(),
                "backend_mismatch: orchestration session sess_stop retained worker ash_stop backend is cli:codex_world not cli:other_world"
            );
        });
    }

    #[test]
    #[serial_test::serial]
    fn resolve_internal_stop_world_dispatch_target_rejects_non_worker_target() {
        with_store(|store| {
            let orchestrator = live_orchestrator("codex", "sess_stop", "orch_stop");
            let mut parent = active_parent(&orchestrator);
            parent.set_world_binding("world-17", 2);

            store
                .persist_orchestration_session(&parent)
                .expect("persist session");
            store
                .persist_participant(&orchestrator)
                .expect("persist orchestrator");

            let err = store
                .resolve_internal_stop_world_dispatch_target(
                    "sess_stop",
                    "orch_stop",
                    "orch_stop",
                    "cli:codex",
                )
                .expect_err("host orchestrator target must fail closed");

            assert_eq!(
                err.to_string(),
                "invalid_target_participant: orchestration session sess_stop participant orch_stop is not a retained world worker"
            );
        });
    }

    #[test]
    #[serial_test::serial]
    fn resolve_internal_stop_world_dispatch_target_rejects_stale_orchestrator_linkage() {
        with_store(|store| {
            let orchestrator = live_orchestrator("codex", "sess_stop", "orch_stop");
            let mut parent = active_parent(&orchestrator);
            parent.set_world_binding("world-17", 2);

            let mut member = live_member("codex_world", "sess_stop", "ash_stop", "orch_stop");
            member.handle.orchestrator_participant_id = Some("orch_stale".to_string());

            store
                .persist_orchestration_session(&parent)
                .expect("persist session");
            store
                .persist_participant(&orchestrator)
                .expect("persist orchestrator");
            store.persist_participant(&member).expect("persist member");

            let err = store
                .resolve_internal_stop_world_dispatch_target(
                    "sess_stop",
                    "orch_stop",
                    "ash_stop",
                    "cli:codex_world",
                )
                .expect_err("stale orchestrator linkage must fail closed");

            assert_eq!(
                err.to_string(),
                "stale_linkage: orchestration session sess_stop retained worker ash_stop is not linked to authoritative orchestrator orch_stop"
            );
        });
    }

    #[test]
    #[serial_test::serial]
    fn resolve_internal_stop_world_dispatch_target_rejects_world_binding_drift() {
        with_store(|store| {
            let orchestrator = live_orchestrator("codex", "sess_stop", "orch_stop");
            let mut parent = active_parent(&orchestrator);
            parent.set_world_binding("world-17", 2);

            let mut member = live_member("codex_world", "sess_stop", "ash_stop", "orch_stop");
            member.handle.world_generation = Some(3);

            store
                .persist_orchestration_session(&parent)
                .expect("persist session");
            store
                .persist_participant(&orchestrator)
                .expect("persist orchestrator");
            store.persist_participant(&member).expect("persist member");

            let err = store
                .resolve_internal_stop_world_dispatch_target(
                    "sess_stop",
                    "orch_stop",
                    "ash_stop",
                    "cli:codex_world",
                )
                .expect_err("world binding drift must fail closed");

            assert_eq!(
                err.to_string(),
                "world_binding_mismatch: orchestration session sess_stop retained worker ash_stop no longer matches the authoritative world binding"
            );
        });
    }

    #[test]
    #[serial_test::serial]
    fn resolve_internal_stop_world_dispatch_target_rejects_stopped_worker() {
        with_store(|store| {
            let orchestrator = live_orchestrator("codex", "sess_stop", "orch_stop");
            let mut parent = active_parent(&orchestrator);
            parent.set_world_binding("world-17", 2);

            let mut member = live_member("codex_world", "sess_stop", "ash_stop", "orch_stop");
            member.mark_terminal_state("worker stopped");
            member.transition_state(AgentRuntimeSessionState::Stopped);

            store
                .persist_orchestration_session(&parent)
                .expect("persist session");
            store
                .persist_participant(&orchestrator)
                .expect("persist orchestrator");
            store.persist_participant(&member).expect("persist member");

            let err = store
                .resolve_internal_stop_world_dispatch_target(
                    "sess_stop",
                    "orch_stop",
                    "ash_stop",
                    "cli:codex_world",
                )
                .expect_err("already-stopped retained workers must fail closed");

            assert_eq!(
                err.to_string(),
                "target_already_terminal: orchestration session sess_stop retained worker ash_stop is already terminal (stopped)"
            );
        });
    }

    #[test]
    #[serial_test::serial]
    fn resolve_internal_stop_world_dispatch_target_rejects_other_terminal_worker() {
        with_store(|store| {
            let orchestrator = live_orchestrator("codex", "sess_stop", "orch_stop");
            let mut parent = active_parent(&orchestrator);
            parent.set_world_binding("world-17", 2);

            let mut member = live_member("codex_world", "sess_stop", "ash_stop", "orch_stop");
            member.mark_terminal_state("worker invalidated");
            member.transition_state(AgentRuntimeSessionState::Invalidated);

            store
                .persist_orchestration_session(&parent)
                .expect("persist session");
            store
                .persist_participant(&orchestrator)
                .expect("persist orchestrator");
            store.persist_participant(&member).expect("persist member");

            let err = store
                .resolve_internal_stop_world_dispatch_target(
                    "sess_stop",
                    "orch_stop",
                    "ash_stop",
                    "cli:codex_world",
                )
                .expect_err("other terminal retained workers must fail closed");

            assert_eq!(
                err.to_string(),
                "target_already_terminal: orchestration session sess_stop retained worker ash_stop is already terminal (invalidated)"
            );
        });
    }

    #[test]
    #[serial_test::serial]
    fn resolve_internal_cancel_world_dispatch_target_returns_exact_running_retained_worker() {
        with_store(|store| {
            let orchestrator = live_orchestrator("codex", "sess_cancel", "orch_cancel");
            let mut parent = active_parent(&orchestrator);
            parent.set_world_binding("world-17", 2);

            let mut member = live_member("codex_world", "sess_cancel", "ash_cancel", "orch_cancel");
            member.transition_state(AgentRuntimeSessionState::Running);
            member.internal.latest_run_id = Some("run-cancel".to_string());

            store
                .persist_orchestration_session(&parent)
                .expect("persist session");
            store
                .persist_participant(&orchestrator)
                .expect("persist orchestrator");
            store.persist_participant(&member).expect("persist member");

            let resolved = store
                .resolve_internal_cancel_world_dispatch_target(
                    "sess_cancel",
                    "orch_cancel",
                    "ash_cancel",
                    "cli:codex_world",
                )
                .expect("resolve exact retained cancel target");

            assert_eq!(resolved.caller_participant.participant_id(), "orch_cancel");
            assert_eq!(resolved.target_participant.participant_id(), "ash_cancel");
            assert_eq!(resolved.active_run_id, "run-cancel");
        });
    }

    #[test]
    #[serial_test::serial]
    fn resolve_internal_cancel_world_dispatch_target_accepts_successor_authoritative_caller_for_retained_worker(
    ) {
        with_store(|store| {
            let mut launch_orchestrator =
                live_orchestrator("codex", "sess_cancel", "orch_cancel_launch");
            let mut successor = live_orchestrator("codex", "sess_cancel", "orch_cancel_successor");
            successor.handle.resumed_from_participant_id = Some("orch_cancel_launch".to_string());
            successor.handle.resumed_from_session_handle_id =
                Some("orch_cancel_launch".to_string());
            launch_orchestrator.mark_client_detached("successor attached");

            let mut member = live_member(
                "codex_world",
                "sess_cancel",
                "ash_cancel",
                "orch_cancel_launch",
            );
            member.transition_state(AgentRuntimeSessionState::Running);
            member.internal.latest_run_id = Some("run-cancel".to_string());

            let mut parent = active_parent(&launch_orchestrator);
            parent.set_world_binding("world-17", 2);
            parent.bind_active_session_handle("orch_cancel_successor".to_string());

            store
                .persist_orchestration_session(&parent)
                .expect("persist session");
            store
                .persist_participant(&launch_orchestrator)
                .expect("persist launch orchestrator");
            store
                .persist_participant(&successor)
                .expect("persist successor orchestrator");
            store.persist_participant(&member).expect("persist member");

            let resolved = store
                .resolve_internal_cancel_world_dispatch_target(
                    "sess_cancel",
                    "orch_cancel_successor",
                    "ash_cancel",
                    "cli:codex_world",
                )
                .expect("resolve retained cancel target through authoritative successor");

            assert_eq!(
                resolved.caller_participant.participant_id(),
                "orch_cancel_successor"
            );
            assert_eq!(resolved.target_participant.participant_id(), "ash_cancel");
            assert_eq!(
                resolved
                    .target_participant
                    .handle
                    .orchestrator_participant_id
                    .as_deref(),
                Some("orch_cancel_launch")
            );
            assert_eq!(resolved.active_run_id, "run-cancel");
        });
    }

    #[test]
    #[serial_test::serial]
    fn resolve_internal_cancel_world_dispatch_target_rejects_unresolved_authoritative_predecessor_lineage(
    ) {
        with_store(|store| {
            let mut successor = live_orchestrator("codex", "sess_cancel", "orch_cancel_successor");
            successor.handle.resumed_from_participant_id =
                Some("orch_cancel_missing_launch".to_string());
            successor.handle.resumed_from_session_handle_id =
                Some("orch_cancel_missing_launch".to_string());

            let mut member = live_member(
                "codex_world",
                "sess_cancel",
                "ash_cancel",
                "orch_cancel_missing_launch",
            );
            member.transition_state(AgentRuntimeSessionState::Running);
            member.internal.latest_run_id = Some("run-cancel".to_string());

            let mut parent = active_parent(&successor);
            parent.set_world_binding("world-17", 2);
            parent.bind_active_session_handle("orch_cancel_successor".to_string());

            store
                .persist_orchestration_session(&parent)
                .expect("persist session");
            store
                .persist_participant(&successor)
                .expect("persist successor orchestrator");
            store.persist_participant(&member).expect("persist member");

            let err = store
                .resolve_internal_cancel_world_dispatch_target(
                    "sess_cancel",
                    "orch_cancel_successor",
                    "ash_cancel",
                    "cli:codex_world",
                )
                .expect_err("unresolved predecessor lineage must fail closed");

            assert_eq!(
                err.to_string(),
                "stale_linkage: orchestration session sess_cancel retained worker ash_cancel is not linked to authoritative orchestrator orch_cancel_successor"
            );
        });
    }

    #[test]
    #[serial_test::serial]
    fn resolve_internal_cancel_world_dispatch_target_rejects_non_authoritative_caller() {
        with_store(|store| {
            let orchestrator = live_orchestrator("codex", "sess_cancel", "orch_cancel");
            let mut parent = active_parent(&orchestrator);
            parent.set_world_binding("world-17", 2);

            let mut member = live_member("codex_world", "sess_cancel", "ash_cancel", "orch_cancel");
            member.transition_state(AgentRuntimeSessionState::Running);
            member.internal.latest_run_id = Some("run-cancel".to_string());

            store
                .persist_orchestration_session(&parent)
                .expect("persist session");
            store
                .persist_participant(&orchestrator)
                .expect("persist orchestrator");
            store.persist_participant(&member).expect("persist member");

            let err = store
                .resolve_internal_cancel_world_dispatch_target(
                    "sess_cancel",
                    "ash_cancel",
                    "ash_cancel",
                    "cli:codex_world",
                )
                .expect_err("member caller must fail closed");

            assert_eq!(
                err.to_string(),
                "caller_not_authoritative: orchestration session sess_cancel authoritative orchestrator participant is orch_cancel not ash_cancel"
            );
        });
    }

    #[test]
    #[serial_test::serial]
    fn resolve_internal_cancel_world_dispatch_target_rejects_cross_session_target() {
        with_store(|store| {
            let orchestrator_a = live_orchestrator("codex", "sess_cancel_a", "orch_cancel_a");
            let mut parent_a = active_parent(&orchestrator_a);
            parent_a.set_world_binding("world-17", 2);

            let orchestrator_b = live_orchestrator("codex", "sess_cancel_b", "orch_cancel_b");
            let mut parent_b = active_parent(&orchestrator_b);
            parent_b.set_world_binding("world-17", 2);

            let mut member_b = live_member(
                "codex_world",
                "sess_cancel_b",
                "ash_cancel_b",
                "orch_cancel_b",
            );
            member_b.transition_state(AgentRuntimeSessionState::Running);
            member_b.internal.latest_run_id = Some("run-cancel".to_string());

            store
                .persist_orchestration_session(&parent_a)
                .expect("persist session a");
            store
                .persist_participant(&orchestrator_a)
                .expect("persist orchestrator a");
            store
                .persist_orchestration_session(&parent_b)
                .expect("persist session b");
            store
                .persist_participant(&orchestrator_b)
                .expect("persist orchestrator b");
            store
                .persist_participant(&member_b)
                .expect("persist member b");

            let err = store
                .resolve_internal_cancel_world_dispatch_target(
                    "sess_cancel_a",
                    "orch_cancel_a",
                    "ash_cancel_b",
                    "cli:codex_world",
                )
                .expect_err("cross-session target must fail closed");

            assert_eq!(
                err.to_string(),
                "target_not_in_session: orchestration session sess_cancel_a has no exact retained worker ash_cancel_b"
            );
        });
    }

    #[test]
    #[serial_test::serial]
    fn resolve_internal_cancel_world_dispatch_target_rejects_world_binding_drift() {
        with_store(|store| {
            let orchestrator = live_orchestrator("codex", "sess_cancel", "orch_cancel");
            let mut parent = active_parent(&orchestrator);
            parent.set_world_binding("world-17", 2);

            let mut member = live_member("codex_world", "sess_cancel", "ash_cancel", "orch_cancel");
            member.transition_state(AgentRuntimeSessionState::Running);
            member.internal.latest_run_id = Some("run-cancel".to_string());
            member.handle.world_generation = Some(3);

            store
                .persist_orchestration_session(&parent)
                .expect("persist session");
            store
                .persist_participant(&orchestrator)
                .expect("persist orchestrator");
            store.persist_participant(&member).expect("persist member");

            let err = store
                .resolve_internal_cancel_world_dispatch_target(
                    "sess_cancel",
                    "orch_cancel",
                    "ash_cancel",
                    "cli:codex_world",
                )
                .expect_err("world binding drift must fail closed");

            assert_eq!(
                err.to_string(),
                "world_binding_mismatch: orchestration session sess_cancel retained worker ash_cancel no longer matches the authoritative world binding"
            );
        });
    }

    #[test]
    #[serial_test::serial]
    fn resolve_internal_cancel_world_dispatch_target_rejects_idle_worker() {
        with_store(|store| {
            let orchestrator = live_orchestrator("codex", "sess_cancel", "orch_cancel");
            let mut parent = active_parent(&orchestrator);
            parent.set_world_binding("world-17", 2);

            let mut member = live_member("codex_world", "sess_cancel", "ash_cancel", "orch_cancel");
            member.internal.latest_run_id = Some("run-cancel".to_string());

            store
                .persist_orchestration_session(&parent)
                .expect("persist session");
            store
                .persist_participant(&orchestrator)
                .expect("persist orchestrator");
            store.persist_participant(&member).expect("persist member");

            let err = store
                .resolve_internal_cancel_world_dispatch_target(
                    "sess_cancel",
                    "orch_cancel",
                    "ash_cancel",
                    "cli:codex_world",
                )
                .expect_err("idle retained workers must fail closed");

            assert_eq!(
                err.to_string(),
                "target_not_cancelable: orchestration session sess_cancel retained worker ash_cancel has no active cancelable work in flight"
            );
        });
    }

    #[test]
    #[serial_test::serial]
    fn resolve_internal_cancel_world_dispatch_target_rejects_worker_without_cancel_support() {
        with_store(|store| {
            let orchestrator = live_orchestrator("codex", "sess_cancel", "orch_cancel");
            let mut parent = active_parent(&orchestrator);
            parent.set_world_binding("world-17", 2);

            let mut member = live_member("codex_world", "sess_cancel", "ash_cancel", "orch_cancel");
            member.transition_state(AgentRuntimeSessionState::Running);
            member.internal.latest_run_id = Some("run-cancel".to_string());
            member.internal.cancel_supported = false;

            store
                .persist_orchestration_session(&parent)
                .expect("persist session");
            store
                .persist_participant(&orchestrator)
                .expect("persist orchestrator");
            store.persist_participant(&member).expect("persist member");

            let err = store
                .resolve_internal_cancel_world_dispatch_target(
                    "sess_cancel",
                    "orch_cancel",
                    "ash_cancel",
                    "cli:codex_world",
                )
                .expect_err("workers without cancel support must fail closed");

            assert_eq!(
                err.to_string(),
                "target_not_cancelable: orchestration session sess_cancel retained worker ash_cancel does not advertise cancel support"
            );
        });
    }

    #[test]
    #[serial_test::serial]
    fn resolve_internal_cancel_world_dispatch_target_rejects_worker_without_active_run_id() {
        with_store(|store| {
            let orchestrator = live_orchestrator("codex", "sess_cancel", "orch_cancel");
            let mut parent = active_parent(&orchestrator);
            parent.set_world_binding("world-17", 2);

            let mut member = live_member("codex_world", "sess_cancel", "ash_cancel", "orch_cancel");
            member.transition_state(AgentRuntimeSessionState::Running);

            store
                .persist_orchestration_session(&parent)
                .expect("persist session");
            store
                .persist_participant(&orchestrator)
                .expect("persist orchestrator");
            store.persist_participant(&member).expect("persist member");

            let err = store
                .resolve_internal_cancel_world_dispatch_target(
                    "sess_cancel",
                    "orch_cancel",
                    "ash_cancel",
                    "cli:codex_world",
                )
                .expect_err("running workers without active run ids must fail closed");

            assert_eq!(
                err.to_string(),
                "target_not_cancelable: orchestration session sess_cancel retained worker ash_cancel has no active cancelable work in flight"
            );
        });
    }

    #[test]
    #[serial_test::serial]
    fn resolve_internal_cancel_world_dispatch_target_rejects_non_live_worker() {
        with_store(|store| {
            let orchestrator = live_orchestrator("codex", "sess_cancel", "orch_cancel");
            let mut parent = active_parent(&orchestrator);
            parent.set_world_binding("world-17", 2);

            let mut member = live_member("codex_world", "sess_cancel", "ash_cancel", "orch_cancel");
            member.transition_state(AgentRuntimeSessionState::Invalidated);
            member.internal.latest_run_id = Some("run-cancel".to_string());

            store
                .persist_orchestration_session(&parent)
                .expect("persist session");
            store
                .persist_participant(&orchestrator)
                .expect("persist orchestrator");
            store.persist_participant(&member).expect("persist member");

            let err = store
                .resolve_internal_cancel_world_dispatch_target(
                    "sess_cancel",
                    "orch_cancel",
                    "ash_cancel",
                    "cli:codex_world",
                )
                .expect_err("non-live retained workers must fail closed");

            assert_eq!(
                err.to_string(),
                "target_already_terminal: orchestration session sess_cancel retained worker ash_cancel is already terminal (invalidated)"
            );
        });
    }

    #[test]
    #[serial_test::serial]
    fn resolve_internal_cancel_world_dispatch_target_rejects_already_cancelled_worker() {
        with_store(|store| {
            let orchestrator = live_orchestrator("codex", "sess_cancel", "orch_cancel");
            let mut parent = active_parent(&orchestrator);
            parent.set_world_binding("world-17", 2);

            let mut member = live_member("codex_world", "sess_cancel", "ash_cancel", "orch_cancel");
            member.transition_state(AgentRuntimeSessionState::Running);
            member.internal.latest_run_id = Some("run-cancel".to_string());
            member.mark_cancelled_terminal_state();

            store
                .persist_orchestration_session(&parent)
                .expect("persist session");
            store
                .persist_participant(&orchestrator)
                .expect("persist orchestrator");
            store.persist_participant(&member).expect("persist member");

            let err = store
                .resolve_internal_cancel_world_dispatch_target(
                    "sess_cancel",
                    "orch_cancel",
                    "ash_cancel",
                    "cli:codex_world",
                )
                .expect_err("already-cancelled retained workers must fail closed");

            assert_eq!(
                err.to_string(),
                "target_already_terminal: orchestration session sess_cancel retained worker ash_cancel is already terminal (cancelled)"
            );
        });
    }

    #[test]
    #[serial_test::serial]
    fn resolve_internal_cancel_world_dispatch_target_rejects_repeat_cancel_after_cancelled_session_closeout(
    ) {
        with_store(|store| {
            let orchestrator = live_orchestrator("codex", "sess_cancel", "orch_cancel");
            let mut parent = active_parent(&orchestrator);
            parent.set_world_binding("world-17", 2);
            parent.mark_cancelled_terminal();

            let mut member = live_member("codex_world", "sess_cancel", "ash_cancel", "orch_cancel");
            member.transition_state(AgentRuntimeSessionState::Running);
            member.internal.latest_run_id = Some("run-cancel".to_string());
            member.mark_cancelled_terminal_state();

            store
                .persist_orchestration_session(&parent)
                .expect("persist session");
            store
                .persist_participant(&orchestrator)
                .expect("persist orchestrator");
            store.persist_participant(&member).expect("persist member");

            let err = store
                .resolve_internal_cancel_world_dispatch_target(
                    "sess_cancel",
                    "orch_cancel",
                    "ash_cancel",
                    "cli:codex_world",
                )
                .expect_err("repeat cancel after cancelled closeout must fail on the target");

            assert_eq!(
                err.to_string(),
                "target_already_terminal: orchestration session sess_cancel retained worker ash_cancel is already terminal (cancelled)"
            );
        });
    }

    #[test]
    #[serial_test::serial]
    fn count_authoritative_live_retained_workers_ignores_non_authoritative_entries() {
        with_store(|store| {
            let orchestrator = live_orchestrator("codex", "sess_dispatch", "orch_dispatch");
            let mut parent = active_parent(&orchestrator);
            parent.set_world_binding("world-17", 2);

            let live = live_member("codex_world", "sess_dispatch", "ash_live", "orch_dispatch");
            let mut invalidated = live_member(
                "codex_world",
                "sess_dispatch",
                "ash_invalidated",
                "orch_dispatch",
            );
            invalidated.transition_state(AgentRuntimeSessionState::Invalidated);
            invalidated.internal.shell_owner_pid = std::process::id();

            let mut drifted = live_member(
                "codex_world",
                "sess_dispatch",
                "ash_drifted",
                "orch_dispatch",
            );
            drifted.handle.world_generation = Some(3);

            let mut relinked =
                live_member("codex_world", "sess_dispatch", "ash_relinked", "orch_other");
            relinked.internal.shell_owner_pid = std::process::id();

            let mut stale =
                live_member("codex_world", "sess_dispatch", "ash_stale", "orch_dispatch");
            stale.internal.shell_owner_pid = 999_999_999;

            store
                .persist_orchestration_session(&parent)
                .expect("persist session");
            store
                .persist_participant(&orchestrator)
                .expect("persist orchestrator");
            store
                .persist_participant(&live)
                .expect("persist live member");
            store
                .persist_participant(&invalidated)
                .expect("persist invalidated member");
            store
                .persist_participant(&drifted)
                .expect("persist drifted member");
            store
                .persist_participant(&relinked)
                .expect("persist relinked member");
            store
                .persist_participant(&stale)
                .expect("persist stale member");

            let count = store
                .count_authoritative_live_retained_workers("sess_dispatch", "orch_dispatch")
                .expect("count live retained workers");

            assert_eq!(count, 1);
        });
    }

    #[test]
    #[serial_test::serial]
    fn host_inbox_state_store_persists_records_under_substrate_home_host_inbox() {
        with_store(|store| {
            let record = pending_host_inbox_record(
                "sess_host_inbox",
                "host_record_one",
                OrchestrationObligationKind::ApprovalRequired,
            );

            store
                .persist_host_inbox_record(&record)
                .expect("persist host inbox record");

            assert!(
                store
                    .host_inbox_dir()
                    .ends_with(std::path::Path::new("host_inbox")),
                "host inbox namespace must root at SUBSTRATE_HOME/host_inbox"
            );
            assert!(
                store
                    .host_inbox_record_path("host_record_one")
                    .expect("valid record_id path")
                    .is_file(),
                "host inbox records must live under SUBSTRATE_HOME/host_inbox"
            );
            assert_eq!(
                store
                    .load_host_inbox_record("host_record_one")
                    .expect("load host inbox record"),
                Some(record)
            );
        });
    }

    #[test]
    #[serial_test::serial]
    fn host_inbox_state_store_lists_records_in_creation_order() {
        with_store(|store| {
            let first = pending_host_inbox_record(
                "sess_host_inbox",
                "host_record_alpha",
                OrchestrationObligationKind::FollowUpRequired,
            );
            let mut second = pending_host_inbox_record(
                "sess_host_inbox",
                "host_record_beta",
                OrchestrationObligationKind::RuntimeAlert,
            );
            second.created_at = first.created_at + chrono::Duration::seconds(1);
            second.updated_at = second.created_at;
            second.materialization_state = HostInboxMaterializationState::Materialized;
            second.materialized_at = Some(second.created_at);
            second.materialized_obligation_id = Some("obl-beta".to_string());
            second
                .validate()
                .expect("materialized host inbox record remains valid");

            store
                .persist_host_inbox_record(&second)
                .expect("persist later host inbox record");
            store
                .persist_host_inbox_record(&first)
                .expect("persist earlier host inbox record");

            let ignored_path = store.host_inbox_dir().join("README.txt");
            fs::create_dir_all(store.host_inbox_dir()).expect("create host inbox dir");
            fs::write(&ignored_path, b"ignore").expect("write ignored artifact");

            assert_eq!(
                store
                    .list_host_inbox_records()
                    .expect("list host inbox records"),
                vec![first, second]
            );
        });
    }

    #[test]
    #[serial_test::serial]
    fn host_inbox_state_store_rejects_separator_and_absolute_record_ids() {
        with_store(|store| {
            let err = store
                .host_inbox_record_path("nested/record")
                .expect_err("separator-containing record_id must fail closed");
            assert!(err.to_string().contains("path separators in record_id"));

            let err = store
                .host_inbox_record_path("/tmp/record")
                .expect_err("absolute record_id must fail closed");
            assert!(err.to_string().contains("absolute record_id"));

            let err = store
                .host_inbox_record_path(r"C:\tmp\record")
                .expect_err("windows absolute record_id must fail closed");
            assert!(err.to_string().contains("absolute record_id"));

            let err = store
                .host_inbox_record_path("C:host_record")
                .expect_err("windows drive-relative record_id must fail closed");
            assert!(err.to_string().contains("absolute record_id"));

            let mut record = pending_host_inbox_record(
                "sess_host_inbox",
                "host_record_gamma",
                OrchestrationObligationKind::ApprovalRequired,
            );
            record.record_id = "nested/record".to_string();

            let err = store
                .persist_host_inbox_record(&record)
                .expect_err("persist must reject separator-containing record_id");
            assert!(err.to_string().contains("path separators in record_id"));

            assert_eq!(
                store
                    .list_host_inbox_records()
                    .expect("list host inbox records after rejected persist"),
                Vec::<HostInboxRecord>::new()
            );
        });
    }

    #[test]
    #[serial_test::serial]
    fn host_inbox_state_store_quarantines_invalid_artifact_failures_outside_canonical_record_paths()
    {
        with_store(|store| {
            let malformed_path = store.host_inbox_dir().join("C:.json");
            fs::create_dir_all(store.host_inbox_dir()).expect("create host inbox dir");
            fs::write(&malformed_path, b"{}").expect("write malformed host inbox artifact");

            let canonical_record_id =
                AgentRuntimeStateStore::invalid_host_inbox_artifact_record_id(&malformed_path);
            let canonical_record = pending_host_inbox_record(
                "sess_host_inbox",
                &canonical_record_id,
                OrchestrationObligationKind::ApprovalRequired,
            );
            store
                .persist_host_inbox_record(&canonical_record)
                .expect("persist canonical host inbox record with colliding synthetic id");

            let failed = store
                .record_invalid_host_inbox_artifact_failure(&malformed_path)
                .expect("persist invalid host inbox artifact failure");
            assert_eq!(
                failed.materialization_state,
                HostInboxMaterializationState::FailedClosed
            );

            let persisted_canonical = store
                .load_host_inbox_record(&canonical_record_id)
                .expect("load canonical host inbox record after invalid artifact failure")
                .expect("canonical host inbox record should still exist");
            assert_eq!(persisted_canonical, canonical_record);

            let quarantined = store
                .load_invalid_host_inbox_artifact_failure_record(&malformed_path)
                .expect("load quarantined invalid host inbox artifact failure")
                .expect("quarantined invalid host inbox artifact failure should exist");
            assert_eq!(
                quarantined.materialization_state,
                HostInboxMaterializationState::FailedClosed
            );
            assert!(quarantined
                .failed_closed_reason
                .as_deref()
                .is_some_and(|reason| reason.contains("invalid_host_inbox_artifact_path")));
        });
    }

    #[test]
    #[serial_test::serial]
    fn host_inbox_state_store_materializes_valid_records_into_one_local_obligation() {
        with_store(|store| {
            let orchestrator = live_orchestrator("codex", "sess_host_materialize", "orch_host");
            let parent = active_parent(&orchestrator);
            store
                .persist_orchestration_session(&parent)
                .expect("persist session");

            let mut record = pending_host_inbox_record(
                "sess_host_materialize",
                "host_record_materialize",
                OrchestrationObligationKind::ApprovalRequired,
            );
            record.source_participant_id = Some("worker-host".to_string());
            record.causation_event_id = Some("evt-materialize".to_string());
            record.causation_message_id = Some("msg-materialize".to_string());
            record.causation_request_id = Some("req-materialize".to_string());
            record.target_backend_id = Some("cli:codex_world".to_string());
            record.payload = Some(json!({
                "event_class": "approval_request",
            }));
            store
                .persist_host_inbox_record(&record)
                .expect("persist host inbox record");

            let materialized = store
                .materialize_host_inbox_record_for_local_host(
                    "host_record_materialize",
                    "host-local",
                )
                .expect("materialize host inbox record");

            assert_eq!(
                materialized.materialization_state,
                HostInboxMaterializationState::Materialized
            );
            assert_eq!(
                materialized.materialized_obligation_id.as_deref(),
                Some("host_inbox_host_record_materialize")
            );

            let obligation = store
                .load_obligation(
                    "sess_host_materialize",
                    "host_inbox_host_record_materialize",
                )
                .expect("load materialized obligation")
                .expect("materialized obligation exists");

            assert!(materialized.matches_materialized_obligation(&obligation));
            assert_eq!(
                obligation.source_participant_id.as_deref(),
                Some("worker-host")
            );
            assert_eq!(
                obligation.target_backend_id.as_deref(),
                Some("cli:codex_world")
            );
            assert_eq!(
                obligation.causation_event_id.as_deref(),
                Some("evt-materialize")
            );
            assert_eq!(
                store
                    .list_obligations("sess_host_materialize")
                    .expect("list obligations"),
                vec![obligation]
            );
        });
    }

    #[test]
    #[serial_test::serial]
    fn host_inbox_state_store_materialization_is_idempotent_for_repeat_runs() {
        with_store(|store| {
            let orchestrator = live_orchestrator("codex", "sess_host_idempotent", "orch_host");
            let parent = active_parent(&orchestrator);
            store
                .persist_orchestration_session(&parent)
                .expect("persist session");

            let record = pending_host_inbox_record(
                "sess_host_idempotent",
                "host_record_idempotent",
                OrchestrationObligationKind::FollowUpRequired,
            );
            store
                .persist_host_inbox_record(&record)
                .expect("persist host inbox record");

            let first = store
                .materialize_host_inbox_record_for_local_host(
                    "host_record_idempotent",
                    "host-local",
                )
                .expect("first materialization");
            let second = store
                .materialize_host_inbox_record_for_local_host(
                    "host_record_idempotent",
                    "host-local",
                )
                .expect("second materialization should be idempotent");

            assert_eq!(first, second);
            assert_eq!(
                store
                    .list_obligations("sess_host_idempotent")
                    .expect("list obligations")
                    .len(),
                1
            );
        });
    }

    #[test]
    #[serial_test::serial]
    fn host_inbox_state_store_materialization_is_serialized_for_concurrent_repeat_runs() {
        with_store(|store| {
            let orchestrator = live_orchestrator("codex", "sess_host_concurrent", "orch_host");
            let parent = active_parent(&orchestrator);
            store
                .persist_orchestration_session(&parent)
                .expect("persist session");

            let record = pending_host_inbox_record(
                "sess_host_concurrent",
                "host_record_concurrent",
                OrchestrationObligationKind::ApprovalRequired,
            );
            store
                .persist_host_inbox_record(&record)
                .expect("persist host inbox record");

            let barrier = Arc::new(Barrier::new(2));
            std::thread::scope(|scope| {
                let first_barrier = Arc::clone(&barrier);
                let first = scope.spawn(move || {
                    first_barrier.wait();
                    store
                        .materialize_host_inbox_record_for_local_host(
                            "host_record_concurrent",
                            "host-local",
                        )
                        .expect("first concurrent materialization")
                });
                let second_barrier = Arc::clone(&barrier);
                let second = scope.spawn(move || {
                    second_barrier.wait();
                    store
                        .materialize_host_inbox_record_for_local_host(
                            "host_record_concurrent",
                            "host-local",
                        )
                        .expect("second concurrent materialization")
                });

                let first = first.join().expect("first thread joins");
                let second = second.join().expect("second thread joins");
                assert_eq!(first, second);
            });

            assert_eq!(
                store
                    .list_obligations("sess_host_concurrent")
                    .expect("list obligations")
                    .len(),
                1
            );
            let persisted = store
                .load_host_inbox_record("host_record_concurrent")
                .expect("load persisted host inbox record")
                .expect("persisted record exists");
            assert_eq!(
                persisted.materialization_state,
                HostInboxMaterializationState::Materialized
            );
        });
    }

    #[test]
    #[serial_test::serial]
    fn host_inbox_state_store_fail_closes_wrong_host_records_without_creating_obligations() {
        with_store(|store| {
            let orchestrator = live_orchestrator("codex", "sess_host_wrong", "orch_host");
            let parent = active_parent(&orchestrator);
            store
                .persist_orchestration_session(&parent)
                .expect("persist session");

            let mut record = pending_host_inbox_record(
                "sess_host_wrong",
                "host_record_wrong",
                OrchestrationObligationKind::Blocked,
            );
            record.target_host_id = "host-remote".to_string();
            store
                .persist_host_inbox_record(&record)
                .expect("persist wrong-host inbox record");

            let failed = store
                .materialize_host_inbox_record_for_local_host("host_record_wrong", "host-local")
                .expect("wrong-host materialization should fail closed durably");

            assert_eq!(
                failed.materialization_state,
                HostInboxMaterializationState::FailedClosed
            );
            assert_eq!(
                failed.failed_closed_reason.as_deref(),
                Some(
                    "wrong_target_host: host inbox record host_record_wrong targets host host-remote not local host host-local"
                )
            );
            assert!(store
                .list_obligations("sess_host_wrong")
                .expect("list obligations")
                .is_empty());
        });
    }

    #[test]
    #[serial_test::serial]
    fn host_inbox_state_store_keeps_missing_session_records_pending_for_later_retry() {
        with_store(|store| {
            let record = pending_host_inbox_record(
                "sess_host_missing_session",
                "host_record_missing_session",
                OrchestrationObligationKind::RuntimeAlert,
            );
            store
                .persist_host_inbox_record(&record)
                .expect("persist host inbox record");

            let pending = store
                .materialize_host_inbox_record_for_local_host(
                    "host_record_missing_session",
                    "host-local",
                )
                .expect("missing-session materialization should stay retryable");

            assert_eq!(
                pending.materialization_state,
                HostInboxMaterializationState::Pending
            );
            assert!(pending.failed_closed_reason.is_none());
            assert!(store
                .list_obligations("sess_host_missing_session")
                .expect("list obligations")
                .is_empty());
        });
    }

    #[test]
    #[serial_test::serial]
    fn host_inbox_state_store_materialized_fast_path_rejects_drifted_obligation_truth() {
        with_store(|store| {
            let orchestrator = live_orchestrator("codex", "sess_host_drifted", "orch_host");
            let parent = active_parent(&orchestrator);
            store
                .persist_orchestration_session(&parent)
                .expect("persist session");

            let record = pending_host_inbox_record(
                "sess_host_drifted",
                "host_record_drifted",
                OrchestrationObligationKind::FollowUpRequired,
            );
            store
                .persist_host_inbox_record(&record)
                .expect("persist host inbox record");

            let materialized = store
                .materialize_host_inbox_record_for_local_host("host_record_drifted", "host-local")
                .expect("materialize host inbox record");
            let obligation_id = materialized
                .materialized_obligation_id
                .clone()
                .expect("materialized obligation id");
            let mut drifted = store
                .load_obligation("sess_host_drifted", &obligation_id)
                .expect("load obligation")
                .expect("obligation exists");
            drifted.summary = "drifted summary".to_string();
            store
                .persist_obligation(&drifted)
                .expect("persist drifted obligation");

            let failed = store
                .materialize_host_inbox_record_for_local_host("host_record_drifted", "host-local")
                .expect("drifted obligation truth must fail closed durably");
            assert_eq!(
                failed.materialization_state,
                HostInboxMaterializationState::FailedClosed
            );
            assert_eq!(
                failed.failed_closed_reason.as_deref(),
                Some(
                    "materialized_host_inbox_record_mismatch: host inbox record host_record_drifted no longer matches obligation host_inbox_host_record_drifted"
                )
            );
            let persisted = store
                .load_host_inbox_record("host_record_drifted")
                .expect("reload failed closed drifted artifact")
                .expect("failed closed artifact persists");
            assert_eq!(persisted, failed);
            assert!(
                store
                    .load_obligation("sess_host_drifted", &obligation_id)
                    .expect("load drifted obligation")
                    .is_some(),
                "fail-closed host inbox drift must not silently delete the canonical obligation artifact"
            );
        });
    }

    #[test]
    #[serial_test::serial]
    fn host_inbox_state_store_fail_closes_malformed_pending_artifacts() {
        with_store(|store| {
            let orchestrator = live_orchestrator("codex", "sess_host_malformed", "orch_host");
            let parent = active_parent(&orchestrator);
            store
                .persist_orchestration_session(&parent)
                .expect("persist session");

            let path = store
                .host_inbox_record_path("host_record_malformed")
                .expect("valid record path");
            fs::create_dir_all(store.host_inbox_dir()).expect("create host inbox dir");
            fs::write(
                &path,
                serde_json::to_vec_pretty(&json!({
                    "orchestration_session_id": "sess_host_malformed",
                    "record_id": "host_record_malformed",
                    "kind": "approval_required",
                    "severity": "info",
                    "created_at": "2026-06-08T00:00:00Z",
                    "updated_at": "2026-06-08T00:00:00Z",
                    "summary": "approval requested",
                    "ingress_source_id": "ingress-host-record-malformed",
                    "ingress_received_at": "2026-06-08T00:00:00Z",
                    "materialization_state": "pending"
                }))
                .expect("serialize malformed host inbox artifact"),
            )
            .expect("write malformed host inbox artifact");

            let failed = store
                .materialize_host_inbox_record_for_local_host("host_record_malformed", "host-local")
                .expect("malformed host inbox artifact should fail closed durably");

            assert_eq!(
                failed.materialization_state,
                HostInboxMaterializationState::FailedClosed
            );
            assert_eq!(
                failed.failed_closed_reason.as_deref(),
                Some("host inbox record must not persist an empty ingress_source_kind")
            );
            let persisted = store
                .load_host_inbox_record("host_record_malformed")
                .expect("reload failed closed malformed artifact")
                .expect("failed closed artifact persists");
            assert_eq!(persisted, failed);
            assert!(store
                .list_obligations("sess_host_malformed")
                .expect("list obligations")
                .is_empty());
        });
    }

    #[test]
    #[serial_test::serial]
    fn host_inbox_state_store_missing_record_id_artifacts_fail_closed_without_reconstruction() {
        with_store(|store| {
            let orchestrator = live_orchestrator("codex", "sess_host_missing_id", "orch_host");
            let parent = active_parent(&orchestrator);
            store
                .persist_orchestration_session(&parent)
                .expect("persist session");

            let path = store.host_inbox_dir().join("host_record_missing_id.json");
            fs::create_dir_all(store.host_inbox_dir()).expect("create host inbox dir");
            fs::write(
                &path,
                serde_json::to_vec_pretty(&json!({
                    "orchestration_session_id": "sess_host_missing_id",
                    "kind": "approval_required",
                    "severity": "info",
                    "created_at": "2026-06-08T00:00:00Z",
                    "updated_at": "2026-06-08T00:00:00Z",
                    "summary": "approval requested",
                    "ingress_source_kind": "host_message",
                    "ingress_source_id": "ingress-host-record-missing-id",
                    "ingress_received_at": "2026-06-08T00:00:00Z",
                    "target_host_id": "host-local",
                    "materialization_state": "pending"
                }))
                .expect("serialize host inbox artifact missing record_id"),
            )
            .expect("write host inbox artifact missing record_id");

            let failed = store
                .materialize_host_inbox_record_for_local_host(
                    "host_record_missing_id",
                    "host-local",
                )
                .expect("missing record_id must fail closed durably");
            assert_eq!(
                failed.materialization_state,
                HostInboxMaterializationState::FailedClosed
            );
            assert_eq!(
                failed.record_id, "host_record_missing_id",
                "failed_closed rewrite must adopt the path-stem identity"
            );
            let expected_reason = format!(
                "host inbox artifact {} is missing required record_id",
                path.display()
            );
            assert_eq!(
                failed.failed_closed_reason.as_deref(),
                Some(expected_reason.as_str())
            );
            let persisted = store
                .load_host_inbox_record("host_record_missing_id")
                .expect("reload failed closed missing-id artifact")
                .expect("failed closed artifact persists");
            assert_eq!(persisted, failed);
            assert!(store
                .list_obligations("sess_host_missing_id")
                .expect("list obligations")
                .is_empty());
        });
    }

    #[test]
    #[serial_test::serial]
    fn host_inbox_state_store_read_paths_reject_path_stem_record_id_mismatch() {
        with_store(|store| {
            let path = store
                .host_inbox_record_path("host_record_path_stem")
                .expect("valid record path");
            fs::create_dir_all(store.host_inbox_dir()).expect("create host inbox dir");
            fs::write(
                &path,
                serde_json::to_vec_pretty(&json!({
                    "orchestration_session_id": "sess_host_path_stem",
                    "record_id": "host_record_other",
                    "kind": "approval_required",
                    "severity": "info",
                    "created_at": "2026-06-08T00:00:00Z",
                    "updated_at": "2026-06-08T00:00:00Z",
                    "summary": "approval requested",
                    "ingress_source_kind": "host_message",
                    "ingress_source_id": "ingress-host-record-path-stem",
                    "ingress_received_at": "2026-06-08T00:00:00Z",
                    "target_host_id": "host-local",
                    "materialization_state": "pending"
                }))
                .expect("serialize mismatched host inbox artifact"),
            )
            .expect("write mismatched host inbox artifact");

            let load_err = store
                .load_host_inbox_record("host_record_path_stem")
                .expect_err("load must enforce path-stem versus record_id truth");
            assert!(load_err
                .to_string()
                .contains("stored mismatched record_id host_record_other"));

            let list_err = store
                .list_host_inbox_records()
                .expect_err("list must enforce path-stem versus record_id truth");
            assert!(list_err
                .to_string()
                .contains("stored mismatched record_id host_record_other"));
        });
    }

    #[test]
    #[serial_test::serial]
    fn host_inbox_state_store_fail_closes_mismatched_record_id_artifacts() {
        with_store(|store| {
            let orchestrator = live_orchestrator("codex", "sess_host_record_mismatch", "orch_host");
            let parent = active_parent(&orchestrator);
            store
                .persist_orchestration_session(&parent)
                .expect("persist session");

            let path = store
                .host_inbox_record_path("host_record_path_truth")
                .expect("valid record path");
            fs::create_dir_all(store.host_inbox_dir()).expect("create host inbox dir");
            fs::write(
                &path,
                serde_json::to_vec_pretty(&json!({
                    "orchestration_session_id": "sess_host_record_mismatch",
                    "record_id": "host_record_wrong_truth",
                    "kind": "approval_required",
                    "severity": "info",
                    "created_at": "2026-06-08T00:00:00Z",
                    "updated_at": "2026-06-08T00:00:00Z",
                    "summary": "approval requested",
                    "ingress_source_kind": "host_message",
                    "ingress_source_id": "ingress-host-record-mismatch",
                    "ingress_received_at": "2026-06-08T00:00:00Z",
                    "target_host_id": "host-local",
                    "materialization_state": "pending"
                }))
                .expect("serialize mismatched record_id host inbox artifact"),
            )
            .expect("write mismatched record_id host inbox artifact");

            let failed = store
                .materialize_host_inbox_record_for_local_host(
                    "host_record_path_truth",
                    "host-local",
                )
                .expect("mismatched record_id must fail closed durably");

            assert_eq!(
                failed.materialization_state,
                HostInboxMaterializationState::FailedClosed
            );
            assert_eq!(
                failed.record_id, "host_record_path_truth",
                "failed_closed rewrite must preserve path-stem identity"
            );
            let expected_reason = format!(
                "host inbox artifact {} stored mismatched record_id host_record_wrong_truth",
                path.display()
            );
            assert_eq!(
                failed.failed_closed_reason.as_deref(),
                Some(expected_reason.as_str())
            );
            let persisted = store
                .load_host_inbox_record("host_record_path_truth")
                .expect("reload failed closed mismatched-id artifact")
                .expect("failed closed artifact persists");
            assert_eq!(persisted, failed);
            assert!(store
                .list_obligations("sess_host_record_mismatch")
                .expect("list obligations")
                .is_empty());
        });
    }

    #[test]
    #[serial_test::serial]
    fn host_inbox_state_store_failed_closed_fast_path_revalidates_malformed_artifacts() {
        with_store(|store| {
            let path = store
                .host_inbox_record_path("host_record_failed_closed_malformed")
                .expect("valid record path");
            fs::create_dir_all(store.host_inbox_dir()).expect("create host inbox dir");
            fs::write(
                &path,
                serde_json::to_vec_pretty(&json!({
                    "orchestration_session_id": "sess_host_failed_closed",
                    "record_id": "host_record_failed_closed_malformed",
                    "kind": "approval_required",
                    "severity": "info",
                    "created_at": "2026-06-08T00:00:00Z",
                    "updated_at": "2026-06-08T00:00:00Z",
                    "summary": "approval requested",
                    "ingress_source_kind": "host_message",
                    "ingress_source_id": "ingress-host-record-failed-closed-malformed",
                    "ingress_received_at": "2026-06-08T00:00:00Z",
                    "target_host_id": "host-local",
                    "materialization_state": "failed_closed"
                }))
                .expect("serialize malformed failed_closed host inbox artifact"),
            )
            .expect("write malformed failed_closed host inbox artifact");

            let failed = store
                .materialize_host_inbox_record_for_local_host(
                    "host_record_failed_closed_malformed",
                    "host-local",
                )
                .expect("malformed failed_closed artifact should be normalized through validation");

            assert_eq!(
                failed.materialization_state,
                HostInboxMaterializationState::FailedClosed
            );
            assert_eq!(
                failed.failed_closed_reason.as_deref(),
                Some(
                    "failed_closed host inbox records must include explanation-ready failure truth"
                )
            );
            assert!(failed.failed_closed_at.is_some());
            let persisted = store
                .load_host_inbox_record("host_record_failed_closed_malformed")
                .expect("reload normalized failed_closed artifact")
                .expect("normalized failed_closed artifact persists");
            assert_eq!(persisted, failed);
        });
    }

    #[test]
    #[serial_test::serial]
    fn host_inbox_state_store_fail_closes_unparseable_artifacts() {
        with_store(|store| {
            let path = store
                .host_inbox_record_path("host_record_unparseable")
                .expect("valid record path");
            fs::create_dir_all(store.host_inbox_dir()).expect("create host inbox dir");
            fs::write(&path, b"{\"record_id\":").expect("write unparseable host inbox artifact");

            let failed = store
                .materialize_host_inbox_record_for_local_host(
                    "host_record_unparseable",
                    "host-local",
                )
                .expect("unparseable host inbox artifact should fail closed durably");

            assert_eq!(
                failed.materialization_state,
                HostInboxMaterializationState::FailedClosed
            );
            assert_eq!(failed.record_id, "host_record_unparseable");
            assert_eq!(failed.summary, "malformed host inbox artifact");
            assert_eq!(failed.severity, OrchestrationObligationSeverity::Error);
            assert_eq!(failed.kind, OrchestrationObligationKind::RuntimeAlert);
            let reason = failed
                .failed_closed_reason
                .as_deref()
                .expect("failed_closed reason");
            assert!(
                reason.starts_with("malformed_host_inbox_artifact: failed to parse "),
                "unparseable artifacts should normalize into a stable malformed-artifact reason: {reason}"
            );

            let persisted = store
                .load_host_inbox_record("host_record_unparseable")
                .expect("reload failed closed unparseable artifact")
                .expect("failed closed artifact persists");
            assert_eq!(persisted, failed);
        });
    }
}
