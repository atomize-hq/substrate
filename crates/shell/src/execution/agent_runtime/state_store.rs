use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use tempfile::NamedTempFile;

use substrate_common::paths as substrate_paths;

use super::{
    orchestration_session::{OrchestrationSessionRecord, OrchestrationSessionState},
    session::AgentRuntimeSessionManifest,
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

    fn ensure_handles_dir(&self) -> Result<()> {
        fs::create_dir_all(self.handles_dir())
            .with_context(|| format!("failed to create {}", self.handles_dir().display()))
    }

    fn ensure_sessions_dir(&self) -> Result<()> {
        fs::create_dir_all(self.sessions_dir())
            .with_context(|| format!("failed to create {}", self.sessions_dir().display()))
    }

    fn manifest_path(&self, session_handle_id: &str) -> PathBuf {
        self.handles_dir().join(format!("{session_handle_id}.json"))
    }

    fn orchestration_session_path(&self, orchestration_session_id: &str) -> PathBuf {
        self.sessions_dir()
            .join(format!("{orchestration_session_id}.json"))
    }

    fn lease_path(&self, session_handle_id: &str) -> PathBuf {
        self.handles_dir()
            .join(format!("{session_handle_id}.lease"))
    }

    pub(crate) fn persist_manifest(&self, manifest: &AgentRuntimeSessionManifest) -> Result<()> {
        self.ensure_handles_dir()?;
        write_atomic_json(
            &self.manifest_path(&manifest.handle.session_handle_id),
            manifest,
        )?;
        self.persist_lease(manifest)
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

    fn persist_lease(&self, manifest: &AgentRuntimeSessionManifest) -> Result<()> {
        let payload = serde_json::json!({
            "session_handle_id": manifest.handle.session_handle_id,
            "shell_owner_pid": manifest.internal.shell_owner_pid,
            "lease_token": manifest.internal.lease_token,
            "state": manifest.handle.state,
            "ownership_valid": manifest.internal.ownership_valid,
            "last_heartbeat_at": manifest.internal.last_heartbeat_at,
            "terminal_observed_at": manifest.internal.terminal_observed_at,
        });
        write_atomic_json(
            &self.lease_path(&manifest.handle.session_handle_id),
            &payload,
        )
    }

    pub(crate) fn list_manifests(&self) -> Result<Vec<AgentRuntimeSessionManifest>> {
        let dir = self.handles_dir();
        let entries = match fs::read_dir(&dir) {
            Ok(entries) => entries,
            Err(err) if err.kind() == io::ErrorKind::NotFound => return Ok(Vec::new()),
            Err(err) => {
                return Err(err).with_context(|| format!("failed to read {}", dir.display()));
            }
        };

        let mut manifests = Vec::new();
        for entry in entries {
            let entry = entry.with_context(|| format!("failed to read {}", dir.display()))?;
            let path = entry.path();
            if path.extension().and_then(|value| value.to_str()) != Some("json") {
                continue;
            }
            let raw = fs::read_to_string(&path)
                .with_context(|| format!("failed to read {}", path.display()))?;
            let manifest: AgentRuntimeSessionManifest = serde_json::from_str(&raw)
                .with_context(|| format!("failed to parse {}", path.display()))?;
            manifests.push(manifest);
        }

        manifests.sort_by(|left, right| {
            left.handle
                .last_transition_at
                .cmp(&right.handle.last_transition_at)
        });
        Ok(manifests)
    }

    pub(crate) fn list_live_manifests(&self) -> Result<Vec<AgentRuntimeSessionManifest>> {
        Ok(self
            .list_manifests()?
            .into_iter()
            // A persisted live manifest is only authoritative while the REPL still owns the
            // retained control boundary and the original shell owner process is still alive.
            .filter(|manifest| manifest.is_authoritative_live() && owner_process_is_alive(manifest))
            .collect())
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
    pub(crate) fn find_live_orchestrator(
        &self,
        agent_id: &str,
    ) -> Result<Option<AgentRuntimeSessionManifest>> {
        let mut latest: Option<AgentRuntimeSessionManifest> = None;
        for manifest in self.list_live_manifests()? {
            if manifest.handle.agent_id != agent_id {
                continue;
            }
            let replace = match &latest {
                Some(existing) => {
                    manifest.handle.last_transition_at >= existing.handle.last_transition_at
                }
                None => true,
            };
            if replace {
                latest = Some(manifest);
            }
        }
        Ok(latest)
    }

    #[allow(dead_code)]
    pub(crate) fn resolve_live_orchestrator_session(
        &self,
        agent_id: &str,
    ) -> Result<Option<(OrchestrationSessionRecord, AgentRuntimeSessionManifest)>> {
        let live_children = self
            .list_live_manifests()?
            .into_iter()
            .filter(|manifest| manifest.handle.agent_id == agent_id)
            .collect::<Vec<_>>();
        let active_parents = self
            .list_orchestration_sessions()?
            .into_iter()
            .filter(|session| {
                session.orchestrator_agent_id == agent_id
                    && session.state == OrchestrationSessionState::Active
                    && owner_pid_is_alive(session.shell_owner_pid)
            })
            .collect::<Vec<_>>();

        if active_parents.len() > 1 {
            return Err(anyhow::anyhow!(
                "multiple active orchestration session candidates found for agent {agent_id}"
            ));
        }

        for manifest in &live_children {
            let Some(parent) =
                self.load_orchestration_session(&manifest.handle.orchestration_session_id)?
            else {
                return Err(anyhow::anyhow!(
                    "missing orchestration session record for live handle {}",
                    manifest.handle.session_handle_id
                ));
            };
            if parent.state != OrchestrationSessionState::Active {
                return Err(anyhow::anyhow!(
                    "inactive orchestration session {} for live handle {}",
                    parent.orchestration_session_id,
                    manifest.handle.session_handle_id
                ));
            }
            let Some(active_session_handle_id) = parent.active_session_handle_id.as_deref() else {
                return Err(anyhow::anyhow!(
                    "active orchestration session {} is missing active_session_handle_id",
                    parent.orchestration_session_id
                ));
            };
            if active_session_handle_id != manifest.handle.session_handle_id {
                return Err(anyhow::anyhow!(
                    "active orchestration session {} points to {}, not live handle {}",
                    parent.orchestration_session_id,
                    active_session_handle_id,
                    manifest.handle.session_handle_id
                ));
            }
        }

        let Some(parent) = active_parents.into_iter().next() else {
            return if live_children.is_empty() {
                Ok(None)
            } else {
                Err(anyhow::anyhow!(
                    "live orchestrator child exists for agent {agent_id} without an active parent session"
                ))
            };
        };

        let active_session_handle_id =
            parent.active_session_handle_id.clone().ok_or_else(|| {
                anyhow::anyhow!(
                    "active orchestration session {} is missing active_session_handle_id",
                    parent.orchestration_session_id
                )
            })?;
        let child = self
            .load_manifest(&active_session_handle_id)?
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "active orchestration session {} references missing handle {}",
                    parent.orchestration_session_id,
                    active_session_handle_id
                )
            })?;
        if !child.is_authoritative_live() || !owner_process_is_alive(&child) {
            return Err(anyhow::anyhow!(
                "active orchestration session {} references inactive handle {}",
                parent.orchestration_session_id,
                active_session_handle_id
            ));
        }
        if child.handle.agent_id != agent_id {
            return Err(anyhow::anyhow!(
                "active orchestration session {} belongs to agent {} not {}",
                parent.orchestration_session_id,
                child.handle.agent_id,
                agent_id
            ));
        }
        if child.handle.orchestration_session_id != parent.orchestration_session_id {
            return Err(anyhow::anyhow!(
                "active orchestration session {} does not match handle {} parent {}",
                parent.orchestration_session_id,
                active_session_handle_id,
                child.handle.orchestration_session_id
            ));
        }
        if live_children.iter().any(|manifest| {
            manifest.handle.session_handle_id != child.handle.session_handle_id
                && manifest.handle.orchestration_session_id != parent.orchestration_session_id
        }) {
            return Err(anyhow::anyhow!(
                "multiple live orchestration session candidates found for agent {agent_id}"
            ));
        }

        Ok(Some((parent, child)))
    }

    #[allow(dead_code)]
    fn load_manifest(
        &self,
        session_handle_id: &str,
    ) -> Result<Option<AgentRuntimeSessionManifest>> {
        let path = self.manifest_path(session_handle_id);
        read_json_if_exists(&path)
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
fn owner_process_is_alive(manifest: &AgentRuntimeSessionManifest) -> bool {
    owner_pid_is_alive(manifest.internal.shell_owner_pid)
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
fn owner_process_is_alive(manifest: &AgentRuntimeSessionManifest) -> bool {
    owner_pid_is_alive(manifest.internal.shell_owner_pid)
}

#[cfg(not(unix))]
fn owner_pid_is_alive(pid: u32) -> bool {
    pid == std::process::id()
}

#[allow(dead_code)]
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

#[allow(dead_code)]
fn read_json_file<T>(path: &Path) -> Result<T>
where
    T: serde::de::DeserializeOwned,
{
    read_json_if_exists(path)?.ok_or_else(|| anyhow::anyhow!("missing {}", path.display()))
}
