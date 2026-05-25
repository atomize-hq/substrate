use std::cmp::Ordering;
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tempfile::NamedTempFile;

use substrate_common::paths as substrate_paths;

use crate::execution::config_model::AgentExecutionScope;

use super::{
    control::PublicSessionPosture,
    mapping::{MEMBER_ROLE, ORCHESTRATOR_ROLE},
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
    Resume,
    Fork,
    Stop,
}

impl PublicControlAction {
    fn requires_attach_contract(self) -> bool {
        matches!(self, Self::Resume | Self::Fork)
    }

    fn requires_continuity_contract(self) -> bool {
        matches!(self, Self::Resume)
    }

    fn rejects_live_owner(self) -> bool {
        matches!(self, Self::Resume)
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

#[derive(Clone, Debug)]
pub(crate) struct AgentRuntimeStateStore {
    substrate_home: PathBuf,
}

impl AgentRuntimeStateStore {
    pub(crate) fn new() -> Result<Self> {
        Ok(Self {
            substrate_home: substrate_paths::substrate_home()?,
        })
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

    fn canonical_participant_path(
        &self,
        orchestration_session_id: &str,
        participant_id: &str,
    ) -> PathBuf {
        self.canonical_participants_dir(orchestration_session_id)
            .join(format!("{participant_id}.json"))
    }

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
    pub(crate) fn canonical_inbox_item_path(
        &self,
        orchestration_session_id: &str,
        item_id: &str,
    ) -> PathBuf {
        self.canonical_inbox_dir(orchestration_session_id)
            .join(format!("{item_id}.json"))
    }

    fn canonical_lease_path(
        &self,
        orchestration_session_id: &str,
        participant_id: &str,
    ) -> PathBuf {
        self.canonical_leases_dir(orchestration_session_id)
            .join(format!("{participant_id}.lease"))
    }

    fn ensure_participants_dir(&self) -> Result<()> {
        fs::create_dir_all(self.participants_dir())
            .with_context(|| format!("failed to create {}", self.participants_dir().display()))
    }

    fn ensure_sessions_dir(&self) -> Result<()> {
        fs::create_dir_all(self.sessions_dir())
            .with_context(|| format!("failed to create {}", self.sessions_dir().display()))
    }

    fn participant_path(&self, participant_id: &str) -> PathBuf {
        self.participants_dir()
            .join(format!("{participant_id}.json"))
    }

    fn orchestration_session_path(&self, orchestration_session_id: &str) -> PathBuf {
        self.sessions_dir()
            .join(format!("{orchestration_session_id}.json"))
    }

    fn lease_path(&self, participant_id: &str) -> PathBuf {
        self.participants_dir()
            .join(format!("{participant_id}.lease"))
    }

    pub(crate) fn persist_participant(
        &self,
        participant: &AgentRuntimeParticipantRecord,
    ) -> Result<()> {
        let _write_guard = snapshot_write_lock()
            .lock()
            .expect("snapshot write mutex poisoned");
        self.validate_participant_record(participant)?;
        if let Some(existing) = self.load_participant(&participant.handle.participant_id)? {
            if !should_persist_participant_snapshot(&existing, participant) {
                return Ok(());
            }
        }
        self.write_participant_snapshot(participant)
    }

    fn write_participant_snapshot(
        &self,
        participant: &AgentRuntimeParticipantRecord,
    ) -> Result<()> {
        self.ensure_participants_dir()?;
        write_atomic_json(
            &self.participant_path(&participant.handle.participant_id),
            participant,
        )?;
        write_atomic_json(
            &self.canonical_participant_path(
                &participant.handle.orchestration_session_id,
                &participant.handle.participant_id,
            ),
            participant,
        )?;
        self.persist_lease(participant)
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

    pub(crate) fn invalidate_stale_world_members_for_session(
        &self,
        orchestration_session_id: &str,
        active_generation: u64,
    ) -> Result<Vec<String>> {
        let mut invalidated_participant_ids = Vec::new();
        let mut invalidated_participants = Vec::new();

        for mut participant in self.list_participants_across_sources()? {
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
        if invalidated_participants.is_empty() {
            return Ok(invalidated_participant_ids);
        }

        let _write_guard = snapshot_write_lock()
            .lock()
            .expect("snapshot write mutex poisoned");
        for participant in &invalidated_participants {
            self.validate_participant_record(participant)?;
            self.write_participant_snapshot(participant)?;
        }

        Ok(invalidated_participant_ids)
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

        if action.rejects_live_owner() && resolved.session_posture == PublicSessionPosture::Active {
            anyhow::bail!(
                "session_already_owned: orchestration session {} already has a live retained owner",
                orchestration_session_id
            );
        }
        if matches!(action, PublicControlAction::Stop)
            && resolved.session_posture == PublicSessionPosture::Terminal
        {
            anyhow::bail!(
                "owner_unreachable: orchestration session {} no longer has a reachable retained owner",
                orchestration_session_id
            );
        }
        let host_attach_contract = resolved.session.host_attach_contract().cloned();
        if action.requires_attach_contract() && host_attach_contract.is_none() {
            anyhow::bail!(
                "owner_unreachable: orchestration session {} is missing durable host attach contract state",
                orchestration_session_id
            );
        }
        if matches!(action, PublicControlAction::Resume)
            && host_attach_contract
                .as_ref()
                .is_some_and(|contract| !contract.supports_resume())
        {
            anyhow::bail!(
                "owner_unreachable: orchestration session {} durable host attach contract does not allow resume",
                orchestration_session_id
            );
        }
        if matches!(action, PublicControlAction::Fork)
            && host_attach_contract
                .as_ref()
                .is_some_and(|contract| !contract.supports_fork())
        {
            anyhow::bail!(
                "owner_unreachable: orchestration session {} durable host attach contract does not allow fork",
                orchestration_session_id
            );
        }
        if matches!(action, PublicControlAction::Stop)
            && host_attach_contract
                .as_ref()
                .is_some_and(|contract| !contract.supports_stop())
        {
            anyhow::bail!(
                "owner_unreachable: orchestration session {} durable host attach contract does not allow stop",
                orchestration_session_id
            );
        }
        if action.requires_continuity_contract()
            && host_attach_contract
                .as_ref()
                .is_none_or(|contract| !contract.has_continuity_selector())
        {
            anyhow::bail!(
                "owner_unreachable: orchestration session {} no longer has continuity required for control-only reattach",
                orchestration_session_id
            );
        }
        if (matches!(action, PublicControlAction::Resume)
            || matches!(
                (action, resolved.session_posture),
                (
                    PublicControlAction::Stop,
                    PublicSessionPosture::DetachedReattachable
                )
            ))
            && !resolved.participant.is_resume_eligible()
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
            host_attach_contract,
        })
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

        let authoritative =
            resolve_authoritative_session_control(&record, orchestration_session_id)?;

        let slot_present = public_turn_session_mentions_backend(&record, backend_id);
        let mut candidates = public_turn_authoritative_candidates(&record, backend_id);
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
        if candidate.kind == PublicTurnTargetKind::World && !cfg!(target_os = "linux") {
            anyhow::bail!(
                "unsupported_platform_or_posture: orchestration session {} backend {} requires Linux world-sensitive follow-up posture",
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

    pub(crate) fn persist_orchestration_session(
        &self,
        session: &OrchestrationSessionRecord,
    ) -> Result<()> {
        let _write_guard = snapshot_write_lock()
            .lock()
            .expect("snapshot write mutex poisoned");
        self.validate_session_record(session)?;
        if let Some(existing) =
            self.load_authoritative_session(&session.orchestration_session_id)?
        {
            if !should_persist_orchestration_session_snapshot(&existing, session) {
                return Ok(());
            }
        }
        self.persist_parent_session_snapshot(session)
    }

    #[allow(dead_code)]
    pub(crate) fn persist_inbox_item(&self, item: &DurableInboxItemRecord) -> Result<()> {
        let _write_guard = snapshot_write_lock()
            .lock()
            .expect("snapshot write mutex poisoned");
        self.validate_inbox_item_record(item)?;

        let mut session = self
            .load_authoritative_session(&item.orchestration_session_id)?
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "missing authoritative parent session {} for durable inbox item {}",
                    item.orchestration_session_id,
                    item.item_id
                )
            })?;
        let existing = self.load_inbox_item(&item.orchestration_session_id, &item.item_id)?;
        let next_pending_count = updated_pending_inbox_count(
            session.pending_inbox_count,
            existing
                .as_ref()
                .is_some_and(DurableInboxItemRecord::is_pending),
            item.is_pending(),
        )?;
        apply_pending_inbox_count(&mut session, next_pending_count);

        let item_path =
            self.canonical_inbox_item_path(&item.orchestration_session_id, &item.item_id);
        write_atomic_json(&item_path, item)?;
        if let Err(err) = self.persist_parent_session_snapshot(&session) {
            rollback_inbox_item_write(&item_path, existing.as_ref())?;
            return Err(err);
        }

        Ok(())
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

    pub(crate) fn set_orchestration_session_world_binding(
        &self,
        session: &mut OrchestrationSessionRecord,
        world_id: impl Into<String>,
        world_generation: u64,
    ) -> Result<()> {
        let _write_guard = snapshot_write_lock()
            .lock()
            .expect("snapshot write mutex poisoned");
        session.set_world_binding(world_id, world_generation);
        self.persist_parent_session_snapshot(session)
    }

    pub(crate) fn clear_orchestration_session_world_binding(
        &self,
        session: &mut OrchestrationSessionRecord,
    ) -> Result<()> {
        let _write_guard = snapshot_write_lock()
            .lock()
            .expect("snapshot write mutex poisoned");
        session.clear_world_binding();
        self.persist_parent_session_snapshot(session)
    }

    fn persist_lease(&self, participant: &AgentRuntimeParticipantRecord) -> Result<()> {
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
        write_atomic_json(
            &self.lease_path(&participant.handle.participant_id),
            &payload,
        )?;
        write_atomic_json(
            &self.canonical_lease_path(
                &participant.handle.orchestration_session_id,
                &participant.handle.participant_id,
            ),
            &payload,
        )
    }

    fn persist_parent_session_snapshot(&self, session: &OrchestrationSessionRecord) -> Result<()> {
        self.validate_session_record(session)?;
        self.ensure_sessions_dir()?;
        write_atomic_json(
            &self.orchestration_session_path(&session.orchestration_session_id),
            session,
        )?;
        write_atomic_json(
            &self.canonical_session_path(&session.orchestration_session_id),
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
                    warnings.push(format!(
                        "active orchestration session {} is missing authoritative orchestrator participant linkage",
                        session.orchestration_session_id
                    ));
                    false
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

        let mut item = self
            .load_inbox_item(orchestration_session_id, item_id)?
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "durable inbox item {} not found in session {}",
                    item_id,
                    orchestration_session_id
                )
            })?;
        item.transition_state(state);
        self.persist_inbox_item(&item)?;
        Ok(item)
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
    let now = Utc::now();
    session.pending_inbox_count = pending_inbox_count;
    session.last_active_at = now;
    if pending_inbox_count > 0 {
        session.last_attention_at = Some(now);
    }

    if session.state.is_terminal() {
        return;
    }

    if session.attached_participant_id().is_some() {
        return;
    }

    let desired_posture = if pending_inbox_count > 0 {
        OrchestrationSessionPosture::AwaitingAttention
    } else {
        OrchestrationSessionPosture::ParkedResumable
    };
    if session.posture != desired_posture {
        session.posture = desired_posture;
        session.posture_changed_at = now;
    }
    if desired_posture == OrchestrationSessionPosture::ParkedResumable {
        session.last_parked_at = Some(now);
    }
}

fn rollback_inbox_item_write(
    item_path: &Path,
    previous: Option<&DurableInboxItemRecord>,
) -> Result<()> {
    match previous {
        Some(previous) => write_atomic_json(item_path, previous),
        None => match fs::remove_file(item_path) {
            Ok(()) => Ok(()),
            Err(err) if err.kind() == io::ErrorKind::NotFound => Ok(()),
            Err(err) => {
                Err(err).with_context(|| format!("failed to roll back {}", item_path.display()))
            }
        },
    }
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
    backend_id: &str,
) -> Vec<PublicTurnTargetCandidate> {
    let mut candidates = Vec::new();
    let active_participant_id = session_authoritative_participant_id(&record.session);

    if let Some(active_participant_id) = active_participant_id {
        if let Some(participant) = record
            .participants
            .iter()
            .find(|participant| participant.participant_id() == active_participant_id)
            .filter(|participant| {
                participant.handle.backend_id == backend_id
                    && participant.matches_public_parent_linkage(&record.session)
            })
        {
            candidates.push(PublicTurnTargetCandidate {
                participant: participant.clone(),
                kind: PublicTurnTargetKind::Host,
            });
        }
    }

    let Some(world_id) = record.session.world_id.as_deref() else {
        return candidates;
    };
    let Some(world_generation) = record.session.world_generation else {
        return candidates;
    };

    candidates.extend(
        record
            .participants
            .iter()
            .filter(|participant| {
                participant.handle.backend_id == backend_id
                    && participant.handle.orchestration_session_id
                        == record.session.orchestration_session_id
                    && participant.handle.role == MEMBER_ROLE
                    && participant.handle.execution.scope == AgentExecutionScope::World
                    && participant.handle.orchestrator_participant_id.as_deref()
                        == active_participant_id
                    && participant.handle.world_id.as_deref() == Some(world_id)
                    && participant.handle.world_generation == Some(world_generation)
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

    Ok(ResolvedAuthoritativeSessionControl {
        session: record.session.clone(),
        participant: participant.clone(),
        session_posture: classify_public_session_posture(record, &participant),
    })
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
    if !participant.is_resume_eligible() {
        return None;
    }
    if !contract.supports_resume() || !contract.supports_continuity_attach() {
        return None;
    }
    if require_internal_session_id && !contract.has_continuity_selector() {
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
    if owner_process_is_alive(participant) || !participant.is_resume_eligible() {
        return false;
    }
    if !contract.supports_resume() || !contract.supports_continuity_attach() {
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

    !require_internal_session_id || contract.has_continuity_selector()
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

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use serde_json::{json, Value};
    use tempfile::TempDir;

    use super::*;
    use crate::execution::agent_runtime::{
        mapping::AgentRuntimeBackendKind, session::AgentRuntimeSessionState,
        validator::RuntimeSelectionDescriptor,
    };

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
        let temp = TempDir::new().expect("tempdir");
        std::env::set_var("SUBSTRATE_HOME", temp.path());
        let store = AgentRuntimeStateStore::new().expect("state store");
        test(&store);
        std::env::remove_var("SUBSTRATE_HOME");
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
    fn resolve_public_control_target_resume_rejects_already_owned_sessions() {
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
                .resolve_public_control_target("sess_resume_live", PublicControlAction::Resume)
                .expect_err("live retained ownership must reject public resume");
            assert!(err.to_string().contains("session_already_owned"));
        });
    }

    #[test]
    #[serial_test::serial]
    fn resolve_public_control_target_allows_resume_for_parked_session() {
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
                .resolve_public_control_target("sess_resume_parked", PublicControlAction::Resume)
                .expect("parked session should remain resumable");
            assert_eq!(
                target.active_participant.handle.participant_id,
                "ash_detached"
            );
        });
    }

    #[test]
    #[serial_test::serial]
    fn resolve_public_control_target_requires_continuity_for_resume_but_not_fork() {
        with_store(|store| {
            let participant =
                detached_orchestrator("codex", "sess_missing_internal", "ash_detached");
            let mut participant = participant;
            participant.internal.uaa_session_id = None;
            participant.internal.resume_eligible = false;
            let parent = active_parent(&participant);
            store
                .persist_orchestration_session(&parent)
                .expect("persist parent");
            store
                .persist_participant(&participant)
                .expect("persist participant");

            let resume_err = store
                .resolve_public_control_target("sess_missing_internal", PublicControlAction::Resume)
                .expect_err("resume must require continuity");
            assert!(resume_err.to_string().contains("owner_unreachable"));

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
    fn resolve_public_control_target_uses_persisted_continuity_truth() {
        with_store(|store| {
            let participant =
                detached_orchestrator("codex", "sess_persisted_resume", "ash_detached");
            let parent = active_parent(&participant);
            let mut participant = participant;
            participant.internal.uaa_session_id = None;

            store
                .persist_orchestration_session(&parent)
                .expect("persist parent");
            store
                .persist_participant(&participant)
                .expect("persist participant");

            let target = store
                .resolve_public_control_target("sess_persisted_resume", PublicControlAction::Resume)
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
    fn resolve_public_control_target_respects_persisted_resume_narrowing() {
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
                .resolve_public_control_target("sess_resume_denied", PublicControlAction::Resume)
                .expect_err("resume must honor persisted capability narrowing");
            assert!(err.to_string().contains("does not allow resume"));
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
            let participant = detached_orchestrator("codex", "sess_turn_parked", "ash_detached");
            let mut parent = active_parent(&participant);
            parent.mark_parked_resumable("owner detached cleanly");
            store
                .persist_orchestration_session(&parent)
                .expect("persist parent");
            store
                .persist_participant(&participant)
                .expect("persist participant");

            let target = store
                .resolve_public_turn_target("sess_turn_parked", "cli:codex")
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
            let participant = detached_orchestrator("codex", "sess_turn_persisted", "ash_detached");
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
                .resolve_public_turn_target("sess_turn_persisted", "cli:codex")
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
            let mut participant = live_orchestrator("codex", "sess_turn_stale", "ash_stale");
            participant.internal.shell_owner_pid = 999_999_999;
            let parent = active_parent(&participant);
            store
                .persist_orchestration_session(&parent)
                .expect("persist parent");
            store
                .persist_participant(&participant)
                .expect("persist participant");

            let target = store
                .resolve_public_turn_target("sess_turn_stale", "cli:codex")
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
            let mut stale = live_orchestrator("codex", "sess_turn_stale_blocked", "ash_stale");
            stale.internal.shell_owner_pid = 999_999_999;
            let live_successor =
                live_orchestrator("codex", "sess_turn_stale_blocked", "ash_successor");
            let parent = active_parent(&stale);
            store
                .persist_orchestration_session(&parent)
                .expect("persist parent");
            store.persist_participant(&stale).expect("persist stale");
            store
                .persist_participant(&live_successor)
                .expect("persist live successor");

            let target = store
                .resolve_public_turn_target("sess_turn_stale_blocked", "cli:codex")
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
            let participant = live_orchestrator("codex", "sess_public_turn", "ash_selected");
            let parent = active_parent(&participant);
            store
                .persist_orchestration_session(&parent)
                .expect("persist parent");
            store
                .persist_participant(&participant)
                .expect("persist participant");

            let err = store
                .resolve_public_turn_target("ash_selected", "cli:codex")
                .expect_err("non-canonical active_session_handle_id selectors must be rejected");
            assert!(err.to_string().contains("noncanonical_session_selector"));
            assert!(err.to_string().contains("active_session_handle_id"));
        });
    }

    #[test]
    #[serial_test::serial]
    fn resolve_public_turn_target_rejects_internal_uaa_selector() {
        with_store(|store| {
            let participant = live_orchestrator("codex", "sess_public_turn", "ash_selected");
            let parent = active_parent(&participant);
            store
                .persist_orchestration_session(&parent)
                .expect("persist parent");
            store
                .persist_participant(&participant)
                .expect("persist participant");

            let err = store
                .resolve_public_turn_target("uaa_session", "cli:codex")
                .expect_err("internal uaa session selectors must be rejected");
            assert!(err.to_string().contains("noncanonical_session_selector"));
            assert!(err.to_string().contains("internal.uaa_session_id"));
        });
    }

    #[test]
    #[serial_test::serial]
    fn resolve_public_turn_target_requires_exact_world_member_linkage() {
        with_store(|store| {
            let orchestrator = live_orchestrator("codex", "sess_world_turn_stale", "ash_selected");
            let mut member = live_member(
                "codex",
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
}
