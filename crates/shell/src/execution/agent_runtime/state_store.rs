use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use tempfile::NamedTempFile;

use substrate_common::paths as substrate_paths;

use super::session::AgentRuntimeSessionManifest;

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

    fn ensure_dirs(&self) -> Result<()> {
        fs::create_dir_all(self.handles_dir())
            .with_context(|| format!("failed to create {}", self.handles_dir().display()))
    }

    fn manifest_path(&self, session_handle_id: &str) -> PathBuf {
        self.handles_dir().join(format!("{session_handle_id}.json"))
    }

    fn lease_path(&self, session_handle_id: &str) -> PathBuf {
        self.handles_dir()
            .join(format!("{session_handle_id}.lease"))
    }

    pub(crate) fn persist_manifest(&self, manifest: &AgentRuntimeSessionManifest) -> Result<()> {
        self.ensure_dirs()?;
        write_atomic_json(
            &self.manifest_path(&manifest.handle.session_handle_id),
            manifest,
        )?;
        self.persist_lease(manifest)
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
    let pid = manifest.internal.shell_owner_pid as libc::pid_t;
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
    manifest.internal.shell_owner_pid == std::process::id()
}
