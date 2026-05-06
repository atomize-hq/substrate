use std::fs;
#[cfg(unix)]
use std::io;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

#[cfg(target_os = "linux")]
use agent_api_client::AgentClient;
#[cfg(target_os = "linux")]
use agent_api_types::ExecuteCancelRequestV1;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
#[cfg(unix)]
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
#[cfg(unix)]
use tokio::net::{UnixListener, UnixStream};
use tokio::sync::{mpsc, oneshot};

use crate::execution::config_model::AgentExecutionScope;

use super::{
    mapping::AgentRuntimeBackendKind, session::AgentRuntimeSessionManifest,
    validator::RuntimeSelectionDescriptor, AgentRuntimeSessionState, AgentRuntimeStateStore,
    OrchestrationSessionRecord, OrchestrationSessionState, ORCHESTRATOR_ROLE,
};

pub(crate) const AGENT_API_SESSION_RESUME_V1: &str = "agent_api.session.resume.v1";
pub(crate) const HIDDEN_OWNER_HELPER_SUBCOMMAND: &str = "__owner-helper";
const OWNER_HELPER_READY_TIMEOUT: Duration = Duration::from_secs(10);
const OWNER_HELPER_READY_POLL_INTERVAL: Duration = Duration::from_millis(100);
#[cfg(unix)]
const PRIVATE_STOP_UNIX_PATH_MAX: usize = 100;

#[allow(dead_code)]
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum OwnerHelperMode {
    Start,
    Resume,
    Fork,
}

#[allow(dead_code)]
impl OwnerHelperMode {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::Start => "start",
            Self::Resume => "resume",
            Self::Fork => "fork",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct PersistedWorldBinding {
    pub world_id: String,
    pub world_generation: u64,
}

#[allow(dead_code)]
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum ResolvedRuntimeBackendKind {
    Codex,
    ClaudeCode,
}

impl From<AgentRuntimeBackendKind> for ResolvedRuntimeBackendKind {
    fn from(value: AgentRuntimeBackendKind) -> Self {
        match value {
            AgentRuntimeBackendKind::Codex => Self::Codex,
            AgentRuntimeBackendKind::ClaudeCode => Self::ClaudeCode,
        }
    }
}

impl TryFrom<ResolvedRuntimeBackendKind> for AgentRuntimeBackendKind {
    type Error = anyhow::Error;

    fn try_from(value: ResolvedRuntimeBackendKind) -> Result<Self> {
        Ok(match value {
            ResolvedRuntimeBackendKind::Codex => AgentRuntimeBackendKind::Codex,
            ResolvedRuntimeBackendKind::ClaudeCode => AgentRuntimeBackendKind::ClaudeCode,
        })
    }
}

#[allow(dead_code)]
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub(crate) struct ResolvedRuntimeDescriptor {
    pub agent_id: String,
    pub backend_id: String,
    pub backend_kind: ResolvedRuntimeBackendKind,
    pub protocol: String,
    pub execution_scope: AgentExecutionScope,
    pub binary_path: String,
}

impl From<&RuntimeSelectionDescriptor> for ResolvedRuntimeDescriptor {
    fn from(value: &RuntimeSelectionDescriptor) -> Self {
        Self {
            agent_id: value.agent_id.clone(),
            backend_id: value.backend_id.clone(),
            backend_kind: value.backend_kind.into(),
            protocol: value.protocol.clone(),
            execution_scope: value.execution_scope,
            binary_path: value.binary_path.display().to_string(),
        }
    }
}

impl TryFrom<&ResolvedRuntimeDescriptor> for RuntimeSelectionDescriptor {
    type Error = anyhow::Error;

    fn try_from(value: &ResolvedRuntimeDescriptor) -> Result<Self> {
        Ok(Self {
            agent_id: value.agent_id.clone(),
            backend_id: value.backend_id.clone(),
            backend_kind: value.backend_kind.try_into()?,
            protocol: value.protocol.clone(),
            execution_scope: value.execution_scope,
            binary_path: PathBuf::from(&value.binary_path),
        })
    }
}

#[allow(dead_code)]
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub(crate) struct HiddenOwnerHelperSessionPlan {
    pub orchestration_session_id: String,
    pub shell_trace_session_id: String,
    pub workspace_root: String,
    pub world_id: Option<String>,
    pub world_generation: Option<u64>,
}

#[allow(dead_code)]
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub(crate) struct HiddenOwnerHelperParticipantPlan {
    pub participant_id: String,
    pub lease_token: String,
    pub run_id: String,
    pub resumed_from_participant_id: Option<String>,
    pub internal_uaa_session_id: Option<String>,
}

#[allow(dead_code)]
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub(crate) struct HiddenOwnerHelperLaunchPlan {
    pub mode: OwnerHelperMode,
    pub descriptor: ResolvedRuntimeDescriptor,
    pub session: HiddenOwnerHelperSessionPlan,
    pub participant: HiddenOwnerHelperParticipantPlan,
    pub source_orchestration_session_id: Option<String>,
}

#[allow(dead_code)]
impl HiddenOwnerHelperLaunchPlan {
    pub(crate) fn orchestration_session_id(&self) -> &str {
        &self.session.orchestration_session_id
    }

    pub(crate) fn participant_id(&self) -> &str {
        &self.participant.participant_id
    }

    pub(crate) fn requires_internal_session_id(&self) -> bool {
        matches!(self.mode, OwnerHelperMode::Resume | OwnerHelperMode::Fork)
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum PrivateStopOutcome {
    Accepted,
    AlreadyTerminal,
    OwnerUnreachable,
    ProtocolError,
}

#[derive(Debug)]
pub(crate) struct PrivateStopRequest {
    pub response_tx: oneshot::Sender<PrivateStopOutcome>,
}

pub(crate) type PrivateStopRequestReceiver = mpsc::UnboundedReceiver<PrivateStopRequest>;
pub(crate) type PrivateStopRequestSender = mpsc::UnboundedSender<PrivateStopRequest>;

#[derive(Debug)]
pub(crate) struct PrivateStopTransport {
    shutdown_tx: Option<oneshot::Sender<()>>,
    task: Option<tokio::task::JoinHandle<()>>,
    path: PathBuf,
}

impl PrivateStopTransport {
    #[allow(dead_code)]
    pub(crate) fn path(&self) -> &Path {
        &self.path
    }

    pub(crate) async fn close(&mut self) {
        if let Some(shutdown_tx) = self.shutdown_tx.take() {
            let _ = shutdown_tx.send(());
        }
        if let Some(task) = self.task.take() {
            let _ = task.await;
        }
        #[cfg(unix)]
        {
            let _ = tokio::fs::remove_file(&self.path).await;
        }
    }
}

pub(crate) fn build_session_resume_extension(session_id: &str) -> serde_json::Value {
    serde_json::json!({
        "selector": "id",
        "id": session_id,
    })
}

#[allow(dead_code)]
pub(crate) fn hidden_owner_helper_plan_path(
    store: &AgentRuntimeStateStore,
    orchestration_session_id: &str,
    participant_id: &str,
) -> PathBuf {
    let session_fragment = compact_stop_transport_fragment(orchestration_session_id);
    let participant_fragment = compact_stop_transport_fragment(participant_id);
    store
        .handles_dir()
        .join("owner-helper")
        .join(format!("{session_fragment}-{participant_fragment}.json"))
}

#[allow(dead_code)]
pub(crate) fn persist_hidden_owner_helper_launch_plan(
    store: &AgentRuntimeStateStore,
    plan: &HiddenOwnerHelperLaunchPlan,
) -> Result<PathBuf> {
    let path = hidden_owner_helper_plan_path(
        store,
        plan.orchestration_session_id(),
        plan.participant_id(),
    );
    let parent = path.parent().ok_or_else(|| {
        anyhow::anyhow!(
            "hidden owner-helper launch plan path '{}' is missing a parent directory",
            path.display()
        )
    })?;
    fs::create_dir_all(parent).with_context(|| format!("failed to create {}", parent.display()))?;
    fs::write(&path, serde_json::to_vec_pretty(plan)?).with_context(|| {
        format!(
            "failed to write hidden owner-helper launch plan {}",
            path.display()
        )
    })?;
    Ok(path)
}

#[allow(dead_code)]
pub(crate) fn load_hidden_owner_helper_launch_plan(
    path: &Path,
) -> Result<HiddenOwnerHelperLaunchPlan> {
    let bytes = fs::read(path).with_context(|| {
        format!(
            "failed to read hidden owner-helper launch plan {}",
            path.display()
        )
    })?;
    serde_json::from_slice(&bytes).with_context(|| {
        format!(
            "failed to decode hidden owner-helper launch plan {}",
            path.display()
        )
    })
}

#[allow(dead_code)]
pub(crate) fn remove_hidden_owner_helper_launch_plan(path: &Path) -> Result<()> {
    match fs::remove_file(path) {
        Ok(()) => Ok(()),
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(err) => Err(err).with_context(|| {
            format!(
                "failed to remove hidden owner-helper launch plan {}",
                path.display()
            )
        }),
    }
}

#[allow(dead_code)]
pub(crate) fn wait_for_hidden_owner_helper_readiness(
    store: &AgentRuntimeStateStore,
    plan: &HiddenOwnerHelperLaunchPlan,
) -> Result<()> {
    let started_at = std::time::Instant::now();
    loop {
        if store.hidden_owner_helper_launch_ready(
            plan.orchestration_session_id(),
            plan.participant_id(),
            plan.requires_internal_session_id(),
        )? {
            return Ok(());
        }
        if started_at.elapsed() >= OWNER_HELPER_READY_TIMEOUT {
            anyhow::bail!(
                "timed out waiting for authoritative owner-helper readiness for orchestration session {}",
                plan.orchestration_session_id()
            );
        }
        thread::sleep(OWNER_HELPER_READY_POLL_INTERVAL);
    }
}

pub(crate) fn private_stop_request_channel(
) -> (PrivateStopRequestSender, PrivateStopRequestReceiver) {
    mpsc::unbounded_channel()
}

pub(crate) fn runtime_is_terminal(manifest: &Arc<Mutex<AgentRuntimeSessionManifest>>) -> bool {
    let state = manifest
        .lock()
        .expect("runtime manifest mutex poisoned")
        .handle
        .state
        .clone();
    !state.is_live()
}

pub(crate) fn runtime_stop_transport_ids(
    manifest: &Arc<Mutex<AgentRuntimeSessionManifest>>,
) -> (String, String) {
    let manifest = manifest.lock().expect("runtime manifest mutex poisoned");
    (
        manifest.handle.orchestration_session_id.clone(),
        manifest.handle.participant_id.clone(),
    )
}

pub(crate) fn runtime_controls_parent_session(role: &str) -> bool {
    role == ORCHESTRATOR_ROLE
}

pub(crate) fn mark_orchestration_session_failed(
    store: &AgentRuntimeStateStore,
    orchestration_session: &Arc<Mutex<OrchestrationSessionRecord>>,
    message: impl Into<String>,
) {
    let message = message.into();
    let snapshot = {
        let mut guard = orchestration_session
            .lock()
            .expect("orchestration session mutex poisoned");
        guard.transition_state(OrchestrationSessionState::Failed);
        guard.mark_terminal(message);
        guard.clone()
    };
    let _ = store.persist_orchestration_session(&snapshot);
}

pub(crate) fn persist_runtime_snapshots(
    store: &AgentRuntimeStateStore,
    orchestration_session: &OrchestrationSessionRecord,
    manifest: &AgentRuntimeSessionManifest,
) -> Result<()> {
    store.persist_orchestration_session(orchestration_session)?;
    store.persist_participant(manifest)
}

pub(crate) fn mark_runtime_startup_failed(
    store: &AgentRuntimeStateStore,
    orchestration_session: &Arc<Mutex<OrchestrationSessionRecord>>,
    manifest: &Arc<Mutex<AgentRuntimeSessionManifest>>,
    message: &str,
) {
    let (orchestration_snapshot, manifest_snapshot) = {
        let mut manifest_guard = manifest.lock().expect("runtime manifest mutex poisoned");
        let controls_parent_session = runtime_controls_parent_session(&manifest_guard.handle.role);
        if manifest_guard.handle.state == AgentRuntimeSessionState::Allocating {
            manifest_guard.transition_state(AgentRuntimeSessionState::Failed);
        }
        if !manifest_guard.has_valid_ownership() {
            manifest_guard.mark_terminal_state(message.to_string());
        }
        manifest_guard.internal.last_error_bucket = Some("bootstrap_run".to_string());
        manifest_guard.internal.last_error_message = Some(message.to_string());
        let orchestration_snapshot = {
            let mut orchestration_guard = orchestration_session
                .lock()
                .expect("orchestration session mutex poisoned");
            if controls_parent_session {
                orchestration_guard.transition_state(OrchestrationSessionState::Failed);
                orchestration_guard.mark_terminal(message.to_string());
            } else {
                orchestration_guard.touch_active();
            }
            orchestration_guard.clone()
        };
        (orchestration_snapshot, manifest_guard.clone())
    };
    let _ = persist_runtime_snapshots(store, &orchestration_snapshot, &manifest_snapshot);
}

pub(crate) fn persist_world_binding_authority(
    store: &AgentRuntimeStateStore,
    orchestration_session: &Arc<Mutex<OrchestrationSessionRecord>>,
    world_binding: Option<&PersistedWorldBinding>,
) -> Result<OrchestrationSessionRecord> {
    let snapshot = {
        let mut guard = orchestration_session
            .lock()
            .expect("orchestration session mutex poisoned");
        match world_binding {
            Some(binding) => {
                store.set_orchestration_session_world_binding(
                    &mut guard,
                    binding.world_id.clone(),
                    binding.world_generation,
                )?;
            }
            None => {
                store.clear_orchestration_session_world_binding(&mut guard)?;
            }
        };
        guard.clone()
    };
    Ok(snapshot)
}

pub(crate) fn invalidate_stale_world_members_after_binding(
    store: &AgentRuntimeStateStore,
    orchestration_session_id: &str,
    active_generation: u64,
) -> Result<Vec<String>> {
    store.invalidate_stale_world_members_for_session(orchestration_session_id, active_generation)
}

pub(crate) fn note_runtime_stop_requested(
    store: &AgentRuntimeStateStore,
    orchestration_session: &Arc<Mutex<OrchestrationSessionRecord>>,
    manifest: &Arc<Mutex<AgentRuntimeSessionManifest>>,
) -> Result<()> {
    let (orchestration_snapshot, manifest_snapshot) = {
        let mut orchestration_guard = orchestration_session
            .lock()
            .expect("orchestration session mutex poisoned");
        let mut manifest_guard = manifest.lock().expect("runtime manifest mutex poisoned");
        let controls_parent_session = runtime_controls_parent_session(&manifest_guard.handle.role);
        if manifest_guard.handle.state.is_live() {
            manifest_guard.transition_state(AgentRuntimeSessionState::Stopping);
            manifest_guard.touch_heartbeat();
        }
        if controls_parent_session && orchestration_guard.state == OrchestrationSessionState::Active
        {
            orchestration_guard.transition_state(OrchestrationSessionState::Stopping);
        } else if orchestration_guard.state == OrchestrationSessionState::Active {
            orchestration_guard.touch_active();
        }
        (orchestration_guard.clone(), manifest_guard.clone())
    };
    persist_runtime_snapshots(store, &orchestration_snapshot, &manifest_snapshot)
}

pub(crate) fn private_stop_transport_path(
    store: &AgentRuntimeStateStore,
    orchestration_session_id: &str,
    participant_id: &str,
) -> PathBuf {
    let session_fragment = compact_stop_transport_fragment(orchestration_session_id);
    let participant_fragment = compact_stop_transport_fragment(participant_id);
    let socket_name = format!("{session_fragment}-{participant_fragment}.sock");
    let preferred = store.handles_dir().join("stop").join(&socket_name);
    #[cfg(unix)]
    if preferred.as_os_str().len() > PRIVATE_STOP_UNIX_PATH_MAX {
        return PathBuf::from("/tmp")
            .join("substrate-agent-hub-stop")
            .join(socket_name);
    }
    preferred
}

fn compact_stop_transport_fragment(id: &str) -> String {
    let normalized = id
        .chars()
        .filter(|ch| ch.is_ascii_alphanumeric())
        .collect::<String>();
    if normalized.len() <= 12 {
        return normalized;
    }

    format!(
        "{}{}",
        &normalized[..6],
        &normalized[normalized.len() - 6..]
    )
}

#[cfg(unix)]
pub(crate) async fn register_private_stop_transport(
    store: &AgentRuntimeStateStore,
    orchestration_session_id: &str,
    participant_id: &str,
    stop_tx: PrivateStopRequestSender,
) -> Result<PrivateStopTransport> {
    let path = private_stop_transport_path(store, orchestration_session_id, participant_id);
    let parent = path.parent().ok_or_else(|| {
        anyhow::anyhow!(
            "private stop transport path '{}' is missing a parent directory",
            path.display()
        )
    })?;
    fs::create_dir_all(parent).with_context(|| format!("failed to create {}", parent.display()))?;
    remove_existing_stop_transport_path(&path)?;
    let listener = UnixListener::bind(&path)
        .with_context(|| format!("failed to bind private stop transport {}", path.display()))?;
    let (shutdown_tx, mut shutdown_rx) = oneshot::channel();
    let path_for_task = path.clone();
    let task = tokio::spawn(async move {
        loop {
            tokio::select! {
                _ = &mut shutdown_rx => break,
                accept = listener.accept() => {
                    let Ok((stream, _)) = accept else {
                        break;
                    };
                    let stop_tx = stop_tx.clone();
                    tokio::spawn(async move {
                        let _ = handle_private_stop_connection(stream, stop_tx).await;
                    });
                }
            }
        }
        let _ = tokio::fs::remove_file(&path_for_task).await;
    });
    Ok(PrivateStopTransport {
        shutdown_tx: Some(shutdown_tx),
        task: Some(task),
        path,
    })
}

#[cfg(not(unix))]
pub(crate) async fn register_private_stop_transport(
    _store: &AgentRuntimeStateStore,
    _orchestration_session_id: &str,
    _participant_id: &str,
    _stop_tx: PrivateStopRequestSender,
) -> Result<PrivateStopTransport> {
    Ok(PrivateStopTransport {
        shutdown_tx: None,
        task: None,
        path: PathBuf::new(),
    })
}

pub(crate) fn spawn_local_private_stop_owner(
    store: AgentRuntimeStateStore,
    orchestration_session: Arc<Mutex<OrchestrationSessionRecord>>,
    manifest: Arc<Mutex<AgentRuntimeSessionManifest>>,
    shutdown_requested: Arc<AtomicBool>,
    cancel: agent_api::AgentWrapperCancelHandle,
    mut stop_rx: PrivateStopRequestReceiver,
) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        while let Some(request) = stop_rx.recv().await {
            let outcome = if runtime_is_terminal(&manifest) {
                PrivateStopOutcome::AlreadyTerminal
            } else {
                shutdown_requested.store(true, Ordering::SeqCst);
                let _ = note_runtime_stop_requested(&store, &orchestration_session, &manifest);
                cancel.cancel();
                PrivateStopOutcome::Accepted
            };
            let _ = request.response_tx.send(outcome);
        }
    })
}

#[cfg(target_os = "linux")]
pub(crate) fn spawn_remote_private_stop_owner(
    store: AgentRuntimeStateStore,
    orchestration_session: Arc<Mutex<OrchestrationSessionRecord>>,
    manifest: Arc<Mutex<AgentRuntimeSessionManifest>>,
    shutdown_requested: Arc<AtomicBool>,
    client: Arc<AgentClient>,
    span_id: String,
    mut stop_rx: PrivateStopRequestReceiver,
) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        while let Some(request) = stop_rx.recv().await {
            let outcome = if runtime_is_terminal(&manifest) {
                PrivateStopOutcome::AlreadyTerminal
            } else {
                shutdown_requested.store(true, Ordering::SeqCst);
                match client
                    .cancel_execute(ExecuteCancelRequestV1 {
                        span_id: span_id.clone(),
                        sig: "INT".to_string(),
                    })
                    .await
                {
                    Ok(_) => {
                        let _ =
                            note_runtime_stop_requested(&store, &orchestration_session, &manifest);
                        PrivateStopOutcome::Accepted
                    }
                    Err(_) => {
                        shutdown_requested.store(false, Ordering::SeqCst);
                        PrivateStopOutcome::ProtocolError
                    }
                }
            };
            let _ = request.response_tx.send(outcome);
        }
    })
}

#[cfg(unix)]
#[allow(dead_code)]
pub(crate) async fn request_private_stop(path: &Path) -> Result<PrivateStopOutcome> {
    let mut stream = UnixStream::connect(path).await.with_context(|| {
        format!(
            "failed to connect to private stop transport {}",
            path.display()
        )
    })?;
    let request = serde_json::json!({
        "version": 1,
        "action": "stop",
    });
    stream
        .write_all(serde_json::to_string(&request)?.as_bytes())
        .await?;
    stream.write_all(b"\n").await?;
    stream.flush().await?;

    let mut reader = BufReader::new(stream);
    let mut line = String::new();
    reader.read_line(&mut line).await?;
    let response: PrivateStopResponse = serde_json::from_str(line.trim()).with_context(|| {
        format!(
            "failed to decode private stop transport response from {}",
            path.display()
        )
    })?;
    Ok(response.outcome)
}

#[cfg(unix)]
fn remove_existing_stop_transport_path(path: &Path) -> Result<()> {
    match fs::symlink_metadata(path) {
        Ok(_) => fs::remove_file(path).with_context(|| {
            format!(
                "failed to remove stale private stop transport {}",
                path.display()
            )
        }),
        Err(err) if err.kind() == io::ErrorKind::NotFound => Ok(()),
        Err(err) => Err(err).with_context(|| {
            format!(
                "failed to inspect existing private stop transport {}",
                path.display()
            )
        }),
    }
}

#[cfg(unix)]
async fn handle_private_stop_connection(
    stream: UnixStream,
    stop_tx: PrivateStopRequestSender,
) -> Result<()> {
    let mut reader = BufReader::new(stream);
    let mut line = String::new();
    let bytes_read = reader.read_line(&mut line).await?;
    let outcome = if bytes_read == 0 || parse_private_stop_request(line.trim()).is_err() {
        PrivateStopOutcome::ProtocolError
    } else {
        let (response_tx, response_rx) = oneshot::channel();
        if stop_tx.send(PrivateStopRequest { response_tx }).is_err() {
            PrivateStopOutcome::OwnerUnreachable
        } else {
            match tokio::time::timeout(Duration::from_secs(5), response_rx).await {
                Ok(Ok(outcome)) => outcome,
                Ok(Err(_)) | Err(_) => PrivateStopOutcome::OwnerUnreachable,
            }
        }
    };

    let response = PrivateStopResponse {
        version: 1,
        outcome,
    };
    let mut stream = reader.into_inner();
    stream
        .write_all(serde_json::to_string(&response)?.as_bytes())
        .await?;
    stream.write_all(b"\n").await?;
    stream.flush().await?;
    Ok(())
}

#[cfg(unix)]
fn parse_private_stop_request(payload: &str) -> Result<PrivateStopRequestV1> {
    let request: PrivateStopRequestV1 =
        serde_json::from_str(payload).context("failed to decode private stop request")?;
    if request.version != 1 || request.action != "stop" {
        anyhow::bail!("unsupported private stop request");
    }
    Ok(request)
}

#[cfg(unix)]
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct PrivateStopRequestV1 {
    version: u8,
    action: String,
}

#[cfg(unix)]
#[derive(Debug, Deserialize, Serialize)]
struct PrivateStopResponse {
    version: u8,
    outcome: PrivateStopOutcome,
}

#[cfg(test)]
mod tests {
    use super::{OwnerHelperMode, PrivateStopOutcome};

    #[test]
    fn owner_helper_modes_remain_internal_and_exact() {
        assert_eq!(OwnerHelperMode::Start.as_str(), "start");
        assert_eq!(OwnerHelperMode::Resume.as_str(), "resume");
        assert_eq!(OwnerHelperMode::Fork.as_str(), "fork");
    }

    #[test]
    fn private_stop_outcomes_are_exact() {
        let outcomes = [
            PrivateStopOutcome::Accepted,
            PrivateStopOutcome::AlreadyTerminal,
            PrivateStopOutcome::OwnerUnreachable,
            PrivateStopOutcome::ProtocolError,
        ];
        assert_eq!(outcomes.len(), 4);
    }
}
