use std::cmp::Ordering;
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use tempfile::NamedTempFile;

use substrate_common::paths as substrate_paths;

use crate::execution::config_model::AgentExecutionScope;

use super::{
    mapping::{MEMBER_ROLE, ORCHESTRATOR_ROLE},
    orchestration_session::{OrchestrationSessionRecord, OrchestrationSessionState},
    session::{AgentRuntimeParticipantRecord, AgentRuntimeSessionManifest},
};

#[derive(Clone, Debug)]
pub(crate) struct AgentRuntimeSessionRecord {
    pub session: OrchestrationSessionRecord,
    pub participants: Vec<AgentRuntimeParticipantRecord>,
    #[allow(dead_code)]
    pub warnings: Vec<String>,
    has_authoritative_parent: bool,
    complete: bool,
}

impl AgentRuntimeSessionRecord {
    pub(crate) fn orchestration_session_id(&self) -> &str {
        &self.session.orchestration_session_id
    }

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
        let active_participant_id = self.session.active_session_handle_id.as_deref()?;
        self.live_participants().into_iter().find(|participant| {
            participant.handle.participant_id == active_participant_id
                && participant.handle.agent_id == self.session.orchestrator_agent_id
                && participant.handle.orchestration_session_id
                    == self.session.orchestration_session_id
                && participant.handle.role == ORCHESTRATOR_ROLE
                && participant.handle.execution.scope == AgentExecutionScope::Host
        })
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

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
enum ParticipantRecordSource {
    Canonical,
    Flat,
    Legacy,
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

        for mut participant in self.list_participants_across_sources()? {
            if participant.handle.orchestration_session_id != orchestration_session_id
                || participant.handle.role != MEMBER_ROLE
                || participant.handle.execution.scope != AgentExecutionScope::World
                || !participant.is_authoritative_live()
                || !owner_process_is_alive(&participant)
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
                self.persist_participant(&participant)?;
            }
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
        Ok(self
            .list_sessions()?
            .into_iter()
            .filter(|record| record.session.orchestrator_agent_id == orchestrator_agent_id)
            .collect())
    }

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

    pub(crate) fn persist_orchestration_session(
        &self,
        session: &OrchestrationSessionRecord,
    ) -> Result<()> {
        let _write_guard = snapshot_write_lock()
            .lock()
            .expect("snapshot write mutex poisoned");
        if let Some(existing) =
            self.load_authoritative_session(&session.orchestration_session_id)?
        {
            if !should_persist_orchestration_session_snapshot(&existing, session) {
                return Ok(());
            }
        }
        self.persist_parent_session_snapshot(session)
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

        sessions.sort_by(|left, right| left.last_active_at.cmp(&right.last_active_at));
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
                return Ok(Some(session));
            }
        }

        read_regular_json_if_exists(&self.orchestration_session_path(orchestration_session_id))
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

        let complete = if !has_authoritative_parent {
            false
        } else if session.state != OrchestrationSessionState::Active {
            true
        } else {
            match session.active_session_handle_id.as_deref() {
                Some(active_participant_id) => match participants
                    .iter()
                    .find(|participant| participant.handle.participant_id == active_participant_id)
                {
                    Some(participant)
                        if participant.is_authoritative_live()
                            && owner_process_is_alive(participant)
                            && participant.handle.agent_id == session.orchestrator_agent_id
                            && participant.handle.orchestration_session_id
                                == session.orchestration_session_id
                            && participant.handle.role == ORCHESTRATOR_ROLE
                            && participant.handle.execution.scope == AgentExecutionScope::Host =>
                    {
                        true
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
                        "active orchestration session {} is missing active_session_handle_id",
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
            protocol: "uaa.agent.session".to_string(),
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
        );
        parent.transition_state(OrchestrationSessionState::Active);
        parent.bind_active_session_handle(participant.handle.participant_id.clone());
        parent
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
            "protocol": "uaa.agent.session",
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
                "protocol": "uaa.agent.session",
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
