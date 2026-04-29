use std::collections::BTreeMap;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use tempfile::NamedTempFile;

use substrate_common::paths as substrate_paths;

use crate::execution::config_model::AgentExecutionScope;

use super::{
    mapping::ORCHESTRATOR_ROLE,
    orchestration_session::{OrchestrationSessionRecord, OrchestrationSessionState},
    session::{AgentRuntimeParticipantRecord, AgentRuntimeSessionManifest},
};

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

    fn legacy_handle_path(&self, participant_id: &str) -> PathBuf {
        self.handles_dir().join(format!("{participant_id}.json"))
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
        self.validate_participant_record(participant)?;
        self.ensure_participants_dir()?;
        write_atomic_json(
            &self.participant_path(&participant.handle.participant_id),
            participant,
        )?;
        self.persist_lease(participant)
    }

    pub(crate) fn load_participant(
        &self,
        participant_id: &str,
    ) -> Result<Option<AgentRuntimeParticipantRecord>> {
        let participant_path = self.participant_path(participant_id);
        if let Some(participant) =
            read_json_if_exists::<AgentRuntimeParticipantRecord>(&participant_path)?
        {
            self.validate_participant_record(&participant)?;
            return Ok(Some(participant));
        }

        let legacy_handle_path = self.legacy_handle_path(participant_id);
        let Some(participant) =
            read_json_if_exists::<AgentRuntimeParticipantRecord>(&legacy_handle_path)?
        else {
            return Ok(None);
        };
        self.validate_participant_record(&participant)?;
        Ok(Some(participant))
    }

    pub(crate) fn list_participants(&self) -> Result<Vec<AgentRuntimeParticipantRecord>> {
        let mut participants = BTreeMap::new();

        for participant in self.read_participant_dir(&self.participants_dir())? {
            participants.insert(participant.handle.participant_id.clone(), participant);
        }

        for participant in self.read_participant_dir(&self.handles_dir())? {
            participants
                .entry(participant.handle.participant_id.clone())
                .or_insert(participant);
        }

        let mut participants = participants.into_values().collect::<Vec<_>>();
        participants.sort_by(|left, right| {
            left.handle
                .last_transition_at
                .cmp(&right.handle.last_transition_at)
        });
        Ok(participants)
    }

    pub(crate) fn list_live_participants(&self) -> Result<Vec<AgentRuntimeParticipantRecord>> {
        Ok(self
            .list_participants()?
            .into_iter()
            .filter(|participant| {
                participant.is_authoritative_live() && owner_process_is_alive(participant)
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

    pub(crate) fn resolve_live_orchestrator_participant(
        &self,
        orchestrator_agent_id: &str,
    ) -> Result<Option<(OrchestrationSessionRecord, AgentRuntimeParticipantRecord)>> {
        let active_parents = self
            .list_orchestration_sessions()?
            .into_iter()
            .filter(|session| {
                session.orchestrator_agent_id == orchestrator_agent_id
                    && session.state == OrchestrationSessionState::Active
                    && owner_pid_is_alive(session.shell_owner_pid)
            })
            .collect::<Vec<_>>();
        if active_parents.len() > 1 {
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

        let Some(parent) = active_parents.into_iter().next() else {
            return if live_host_orchestrators.is_empty() {
                Ok(None)
            } else {
                Err(anyhow::anyhow!(
                    "live host-scoped orchestrator participant exists for agent {orchestrator_agent_id} without an active parent session"
                ))
            };
        };

        let active_participant_id = parent.active_session_handle_id.clone().ok_or_else(|| {
            anyhow::anyhow!(
                "active orchestration session {} is missing active_session_handle_id",
                parent.orchestration_session_id
            )
        })?;
        let participant = self
            .load_participant(&active_participant_id)?
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "active orchestration session {} references missing participant {}",
                    parent.orchestration_session_id,
                    active_participant_id
                )
            })?;
        self.validate_participant_record(&participant)?;
        if !participant.is_authoritative_live() || !owner_process_is_alive(&participant) {
            anyhow::bail!(
                "active orchestration session {} references inactive participant {}",
                parent.orchestration_session_id,
                active_participant_id
            );
        }
        if participant.handle.agent_id != orchestrator_agent_id {
            anyhow::bail!(
                "active orchestration session {} belongs to agent {} not {}",
                parent.orchestration_session_id,
                participant.handle.agent_id,
                orchestrator_agent_id
            );
        }
        if participant.handle.orchestration_session_id != parent.orchestration_session_id {
            anyhow::bail!(
                "active orchestration session {} does not match participant {} parent {}",
                parent.orchestration_session_id,
                active_participant_id,
                participant.handle.orchestration_session_id
            );
        }
        if participant.handle.role != ORCHESTRATOR_ROLE
            || participant.handle.execution.scope != AgentExecutionScope::Host
        {
            anyhow::bail!(
                "active orchestration session {} references non-host orchestrator participant {}",
                parent.orchestration_session_id,
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

        Ok(Some((parent, participant)))
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
        self.ensure_sessions_dir()?;
        write_atomic_json(
            &self.orchestration_session_path(&session.orchestration_session_id),
            session,
        )
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
        )
    }

    fn read_participant_dir(&self, dir: &Path) -> Result<Vec<AgentRuntimeParticipantRecord>> {
        let entries = match fs::read_dir(dir) {
            Ok(entries) => entries,
            Err(err) if err.kind() == io::ErrorKind::NotFound => return Ok(Vec::new()),
            Err(err) => {
                return Err(err).with_context(|| format!("failed to read {}", dir.display()));
            }
        };

        let mut participants = Vec::new();
        for entry in entries {
            let entry = entry.with_context(|| format!("failed to read {}", dir.display()))?;
            let path = entry.path();
            if path.extension().and_then(|value| value.to_str()) != Some("json") {
                continue;
            }
            let participant: AgentRuntimeParticipantRecord = read_json_file(&path)
                .with_context(|| format!("failed to load {}", path.display()))?;
            self.validate_participant_record(&participant)
                .with_context(|| format!("invalid participant record in {}", path.display()))?;
            participants.push(participant);
        }

        Ok(participants)
    }

    #[allow(dead_code)]
    pub(crate) fn load_orchestration_session(
        &self,
        orchestration_session_id: &str,
    ) -> Result<Option<OrchestrationSessionRecord>> {
        let path = self.orchestration_session_path(orchestration_session_id);
        read_json_if_exists(&path)
    }

    #[allow(dead_code)]
    pub(crate) fn list_orchestration_sessions(&self) -> Result<Vec<OrchestrationSessionRecord>> {
        let dir = self.sessions_dir();
        let entries = match fs::read_dir(&dir) {
            Ok(entries) => entries,
            Err(err) if err.kind() == io::ErrorKind::NotFound => return Ok(Vec::new()),
            Err(err) => {
                return Err(err).with_context(|| format!("failed to read {}", dir.display()));
            }
        };

        let mut sessions = Vec::new();
        for entry in entries {
            let entry = entry.with_context(|| format!("failed to read {}", dir.display()))?;
            let path = entry.path();
            if path.extension().and_then(|value| value.to_str()) != Some("json") {
                continue;
            }
            let session: OrchestrationSessionRecord = read_json_file(&path)?;
            sessions.push(session);
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

fn read_json_file<T>(path: &Path) -> Result<T>
where
    T: serde::de::DeserializeOwned,
{
    read_json_if_exists(path)?.ok_or_else(|| anyhow::anyhow!("missing {}", path.display()))
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
