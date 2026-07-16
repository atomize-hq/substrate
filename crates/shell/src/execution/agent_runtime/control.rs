use std::collections::BTreeMap;
use std::env;
use std::fs;
use std::fs::File;
#[cfg(unix)]
use std::io::Write;
use std::io::{self, Read};
#[cfg(unix)]
use std::os::unix::net::UnixListener as StdUnixListener;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitStatus, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use anyhow::{Context, Result};
#[cfg(target_os = "linux")]
use base64::engine::general_purpose::STANDARD as BASE64;
#[cfg(target_os = "linux")]
use base64::Engine;
use chrono::Utc;
use fs2::FileExt;
use futures::StreamExt;
use serde::{Deserialize, Serialize};
use substrate_broker::Policy;
#[cfg(unix)]
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
#[cfg(unix)]
use tokio::net::{UnixListener, UnixStream};
use tokio::sync::{mpsc, oneshot};
#[cfg(target_os = "linux")]
use transport_api_types::{ExecuteStreamFrame, MemberTurnSubmitRequestV1};
use uuid::Uuid;

#[cfg(unix)]
use crate::execution::agent_events::format_event_line;
use crate::execution::agent_runtime::dispatch_contract::WorkerCancelPayloadV1;
use crate::execution::agent_runtime::orchestration_session::{
    HostAttachContract, OrchestrationSessionPosture, StartupPromptStreamState,
};
#[cfg(target_os = "linux")]
use crate::execution::build_agent_client_and_pending_diff_request;
use crate::execution::config_model::{
    AgentExecutionScope, AgentToolboxBindTransport, SubstrateConfig,
};
use crate::execution::prompt_fulfillment::{
    build_runtime_owned_toolbox_env, compose_prompt_with_host_toolbox_contract,
    PromptFulfillmentCancelHandle,
};

use super::{
    backend_allowed, mapping::AgentRuntimeBackendKind, session::AgentRuntimeSessionManifest,
    validator::RuntimeSelectionDescriptor, AgentRuntimeSessionState, AgentRuntimeStateStore,
    OrchestrationSessionRecord, OrchestrationSessionState, ORCHESTRATOR_ROLE, PURE_AGENT_PROTOCOL,
};
use substrate_common::agent_events::{AgentEvent, MessageEventKind};
use substrate_common::paths as substrate_paths;

pub(crate) const AGENT_API_SESSION_RESUME_V1: &str = "agent_api.session.resume.v1";
pub(crate) const AGENT_API_TURN_LIFECYCLE_V1: &str = "agent_api.turn.lifecycle.v1";
pub(crate) const HIDDEN_OWNER_HELPER_SUBCOMMAND: &str = "__owner-helper";
const OWNER_HELPER_READY_TIMEOUT_ERROR_PREFIX: &str =
    "timed out waiting for authoritative owner-helper readiness for orchestration session ";
const OWNER_HELPER_READY_TIMEOUT: Duration = Duration::from_secs(30);
const OWNER_HELPER_READY_POLL_INTERVAL: Duration = Duration::from_millis(100);
const PRIVATE_STOP_UNIX_PATH_MAX: usize = 100;
#[cfg(unix)]
const PRIVATE_PROMPT_READY_TIMEOUT: Duration = Duration::from_secs(10);
#[cfg(unix)]
const PRIVATE_PROMPT_READY_POLL_INTERVAL: Duration = Duration::from_millis(100);
#[cfg(unix)]
const STARTUP_PROMPT_STREAM_ACCEPT_TIMEOUT: Duration = Duration::from_secs(10);
#[cfg(unix)]
const START_DETACH_NORMALIZATION_TIMEOUT: Duration = Duration::from_secs(10);
#[cfg(unix)]
const START_DETACH_NORMALIZATION_POLL_INTERVAL: Duration = Duration::from_millis(50);
#[cfg(unix)]
const START_ATTACHED_GRACE_TIMEOUT: Duration = Duration::from_millis(250);

#[allow(dead_code)]
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum OwnerHelperMode {
    Start,
    Attach,
    ResumeOneTurn,
}

#[allow(dead_code)]
impl OwnerHelperMode {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::Start => "start",
            Self::Attach => "attach",
            Self::ResumeOneTurn => "resume_one_turn",
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum PublicPromptAction {
    Start,
    Turn,
}

impl PublicPromptAction {
    #[cfg(unix)]
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::Start => "start",
            Self::Turn => "turn",
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum PublicSessionPosture {
    Active,
    DetachedReattachable,
    Terminal,
}

impl PublicSessionPosture {
    #[cfg(unix)]
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::Active => "active",
            Self::DetachedReattachable => "detached_reattachable",
            Self::Terminal => "terminal",
        }
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct PublicPromptInput {
    pub prompt: Option<String>,
    pub prompt_file: Option<PathBuf>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct LoadedPublicPrompt {
    pub prompt_text: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct PublicPromptCommandRequest {
    pub action: PublicPromptAction,
    pub orchestration_session_id: Option<String>,
    pub backend_id: String,
    pub prompt: LoadedPublicPrompt,
    pub json: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct PersistedWorldBinding {
    pub world_id: String,
    pub world_generation: u64,
}

#[derive(Clone)]
pub(crate) struct PromptSubmitRuntime {
    pub descriptor: RuntimeSelectionDescriptor,
    pub orchestration_session: Arc<Mutex<OrchestrationSessionRecord>>,
    pub manifest: Arc<Mutex<AgentRuntimeSessionManifest>>,
    pub store: AgentRuntimeStateStore,
    pub uaa_session_handle_id: String,
    pub park_after_turn_tx: Option<mpsc::UnboundedSender<()>>,
    pub host_toolbox_surface_authoritative: Arc<AtomicBool>,
}

#[cfg_attr(not(target_os = "linux"), allow(dead_code))]
#[derive(Clone, Debug, PartialEq)]
pub(crate) enum SubmittedPromptStreamEvent {
    Agent(Box<AgentEvent>),
    Stdout(String),
    Stderr(String),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct SubmittedPromptCompletion {
    pub exit_code: i32,
    pub warning: Option<String>,
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
pub(crate) struct HiddenOwnerHelperStartupPromptPlan {
    pub prompt_text: String,
    pub stream_path: PathBuf,
}

#[allow(dead_code)]
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub(crate) struct HiddenOwnerHelperLaunchPlan {
    pub mode: OwnerHelperMode,
    pub descriptor: ResolvedRuntimeDescriptor,
    pub session: HiddenOwnerHelperSessionPlan,
    pub participant: HiddenOwnerHelperParticipantPlan,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub host_attach_contract: Option<HostAttachContract>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub startup_prompt: Option<HiddenOwnerHelperStartupPromptPlan>,
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
        matches!(self.mode, OwnerHelperMode::ResumeOneTurn)
            || matches!(self.mode, OwnerHelperMode::Attach)
                && self.participant.internal_uaa_session_id.is_some()
    }
}

#[allow(dead_code)]
#[derive(Clone, Debug)]
pub(crate) struct HiddenOwnerHelperLaunchReceipt {
    pub helper_pid: u32,
    pub orchestration_session_id: String,
    pub participant_id: String,
    pub backend_id: String,
}

struct HiddenOwnerHelperAttachLaunchGuard {
    _lock_file: File,
}

enum HiddenOwnerHelperAttachJoinResult {
    Joined(HiddenOwnerHelperLaunchReceipt),
    RetryAsLeader,
}

#[allow(dead_code)]
pub(crate) fn launch_hidden_owner_helper(
    plan: &HiddenOwnerHelperLaunchPlan,
    world: bool,
    no_world: bool,
) -> Result<HiddenOwnerHelperLaunchReceipt> {
    let store = AgentRuntimeStateStore::new()?;
    let _attach_launch_guard = if plan.mode == OwnerHelperMode::Attach {
        loop {
            match try_acquire_hidden_owner_helper_attach_launch_guard(&store, plan)? {
                Some(guard) => break Some(guard),
                None => match wait_for_inflight_hidden_owner_helper_attach_launch(&store, plan)? {
                    HiddenOwnerHelperAttachJoinResult::Joined(receipt) => return Ok(receipt),
                    HiddenOwnerHelperAttachJoinResult::RetryAsLeader => continue,
                },
            }
        }
    } else {
        None
    };
    let plan_path = persist_hidden_owner_helper_launch_plan(&store, plan)?;
    let exe = env::current_exe()
        .context("failed to resolve current substrate executable for hidden owner-helper launch")?;
    let mut command = Command::new(exe);
    if world {
        command.arg("--world");
    } else if no_world {
        command.arg("--no-world");
    }
    command
        .args(["agent", HIDDEN_OWNER_HELPER_SUBCOMMAND, "--plan-file"])
        .arg(&plan_path)
        .current_dir(&plan.session.workspace_root)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());

    let mut child = command.spawn().with_context(|| {
        format!(
            "failed to spawn hidden owner-helper for orchestration session {}",
            plan.session.orchestration_session_id
        )
    })?;
    if let Err(err) = wait_for_hidden_owner_helper_readiness(&store, plan) {
        let reconciled = if plan.mode == OwnerHelperMode::Start
            && hidden_owner_helper_readiness_timed_out(&err)
        {
            Some(reconcile_hidden_owner_helper_start_timeout(&store, plan))
        } else {
            None
        };
        match reconciled {
            Some(Ok(HiddenOwnerHelperStartTimeoutReconciliation::Success)) => {}
            Some(Ok(HiddenOwnerHelperStartTimeoutReconciliation::FailureMarkedTerminal)) => {
                let _ = child.kill();
                let _ = child.wait();
                let _ = remove_hidden_owner_helper_launch_plan(&plan_path);
                return Err(anyhow::anyhow!(
                    "{}; persisted terminal startup failure for orchestration session {}",
                    err,
                    plan.orchestration_session_id(),
                ));
            }
            Some(Ok(HiddenOwnerHelperStartTimeoutReconciliation::FailureUnchanged)) | None => {
                let _ = child.kill();
                let _ = child.wait();
                let _ = remove_hidden_owner_helper_launch_plan(&plan_path);
                return Err(err);
            }
            Some(Err(reconcile_err)) => {
                let _ = child.kill();
                let _ = child.wait();
                let _ = remove_hidden_owner_helper_launch_plan(&plan_path);
                return Err(anyhow::anyhow!(
                    "{}; additionally failed to reconcile persisted startup state: {reconcile_err:#}",
                    err
                ));
            }
        }
    }
    #[cfg(unix)]
    if let Err(err) = stabilize_hidden_owner_helper_start_return(&store, plan, &mut child) {
        let _ = child.kill();
        let _ = child.wait();
        let _ = remove_hidden_owner_helper_launch_plan(&plan_path);
        return Err(err);
    }

    Ok(HiddenOwnerHelperLaunchReceipt {
        helper_pid: child.id(),
        orchestration_session_id: plan.session.orchestration_session_id.clone(),
        participant_id: plan.participant.participant_id.clone(),
        backend_id: plan.descriptor.backend_id.clone(),
    })
}

fn try_acquire_hidden_owner_helper_attach_launch_guard(
    store: &AgentRuntimeStateStore,
    plan: &HiddenOwnerHelperLaunchPlan,
) -> Result<Option<HiddenOwnerHelperAttachLaunchGuard>> {
    let path = hidden_owner_helper_attach_lock_path(store, plan.orchestration_session_id());
    let parent = path.parent().ok_or_else(|| {
        anyhow::anyhow!(
            "hidden owner-helper attach lock path '{}' is missing a parent directory",
            path.display()
        )
    })?;
    fs::create_dir_all(parent).with_context(|| format!("failed to create {}", parent.display()))?;
    let file = fs::OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(false)
        .open(&path)
        .with_context(|| {
            format!(
                "failed to open hidden owner-helper attach lock {}",
                path.display()
            )
        })?;
    match file.try_lock_exclusive() {
        Ok(()) => Ok(Some(HiddenOwnerHelperAttachLaunchGuard {
            _lock_file: file,
        })),
        Err(err) if err.kind() == std::io::ErrorKind::WouldBlock => Ok(None),
        Err(err) => Err(anyhow::Error::new(err).context(format!(
            "failed to coordinate hidden owner-helper attach launch for orchestration session {}",
            plan.orchestration_session_id()
        ))),
    }
}

fn wait_for_inflight_hidden_owner_helper_attach_launch(
    store: &AgentRuntimeStateStore,
    plan: &HiddenOwnerHelperLaunchPlan,
) -> Result<HiddenOwnerHelperAttachJoinResult> {
    let started_at = Instant::now();
    loop {
        if joined_hidden_owner_helper_attach_receipt(store, plan.orchestration_session_id()).is_ok()
        {
            return Ok(HiddenOwnerHelperAttachJoinResult::Joined(
                joined_hidden_owner_helper_attach_receipt(store, plan.orchestration_session_id())?,
            ));
        }
        if let Some(guard) = try_acquire_hidden_owner_helper_attach_launch_guard(store, plan)? {
            drop(guard);
            return Ok(HiddenOwnerHelperAttachJoinResult::RetryAsLeader);
        }
        if started_at.elapsed() >= OWNER_HELPER_READY_TIMEOUT {
            anyhow::bail!(
                "{}{}",
                OWNER_HELPER_READY_TIMEOUT_ERROR_PREFIX,
                plan.orchestration_session_id(),
            );
        }
        thread::sleep(OWNER_HELPER_READY_POLL_INTERVAL);
    }
}

fn joined_hidden_owner_helper_attach_receipt(
    store: &AgentRuntimeStateStore,
    orchestration_session_id: &str,
) -> Result<HiddenOwnerHelperLaunchReceipt> {
    let record = store
        .load_session(orchestration_session_id)?
        .ok_or_else(|| anyhow::anyhow!(
            "owner_unreachable: joined hidden owner-helper attach launch lost orchestration session {}",
            orchestration_session_id
        ))?;
    if record.session.posture != OrchestrationSessionPosture::ActiveAttached {
        anyhow::bail!(
            "owner_unreachable: joined hidden owner-helper attach launch for orchestration session {} has not restored active_attached posture yet",
            orchestration_session_id
        );
    }
    let participant = record.live_orchestrator().ok_or_else(|| {
        anyhow::anyhow!(
            "owner_unreachable: joined hidden owner-helper attach launch for orchestration session {} did not restore a live retained owner",
            orchestration_session_id
        )
    })?;
    if !participant.attached_client_present() {
        anyhow::bail!(
            "owner_unreachable: joined hidden owner-helper attach launch for orchestration session {} did not restore an attached host execution client",
            orchestration_session_id
        );
    }
    let helper_pid = participant.internal.shell_owner_pid;
    if helper_pid == 0 {
        anyhow::bail!(
            "owner_unreachable: joined hidden owner-helper attach launch for orchestration session {} is missing authoritative owner pid",
            orchestration_session_id,
        );
    }

    Ok(HiddenOwnerHelperLaunchReceipt {
        helper_pid,
        orchestration_session_id: orchestration_session_id.to_string(),
        participant_id: participant.handle.participant_id,
        backend_id: participant.handle.backend_id,
    })
}

#[cfg(unix)]
fn stabilize_hidden_owner_helper_start_return(
    store: &AgentRuntimeStateStore,
    plan: &HiddenOwnerHelperLaunchPlan,
    child: &mut std::process::Child,
) -> Result<()> {
    if plan.mode != OwnerHelperMode::Start {
        return Ok(());
    }

    let grace_started_at = std::time::Instant::now();
    loop {
        if matches!(
            store.classify_hidden_owner_helper_launch_readiness(
                plan.orchestration_session_id(),
                plan.participant_id(),
                plan.requires_internal_session_id(),
            )?,
            super::state_store::HiddenOwnerHelperLaunchReadiness::ReadyDetached(_)
        ) {
            return Ok(());
        }
        if child
            .try_wait()
            .context("failed to poll hidden owner-helper exit status")?
            .is_some()
        {
            break;
        }
        if grace_started_at.elapsed() >= START_ATTACHED_GRACE_TIMEOUT {
            return Ok(());
        }
        thread::sleep(START_DETACH_NORMALIZATION_POLL_INTERVAL);
    }

    let normalization_started_at = std::time::Instant::now();
    loop {
        if matches!(
            store.classify_hidden_owner_helper_launch_readiness(
                plan.orchestration_session_id(),
                plan.participant_id(),
                plan.requires_internal_session_id(),
            )?,
            super::state_store::HiddenOwnerHelperLaunchReadiness::ReadyDetached(_)
        ) {
            return Ok(());
        }

        if normalization_started_at.elapsed() >= START_DETACH_NORMALIZATION_TIMEOUT {
            match reconcile_hidden_owner_helper_start_timeout(store, plan) {
                Ok(HiddenOwnerHelperStartTimeoutReconciliation::Success) => return Ok(()),
                Ok(
                    HiddenOwnerHelperStartTimeoutReconciliation::FailureMarkedTerminal
                    | HiddenOwnerHelperStartTimeoutReconciliation::FailureUnchanged,
                ) => {}
                Err(reconcile_err) => {
                    anyhow::bail!(
                        "timed out waiting for detached start normalization for orchestration session {} after hidden owner-helper {} exited; additionally failed to reconcile persisted startup state: {reconcile_err:#}",
                        plan.orchestration_session_id(),
                        child.id(),
                    );
                }
            }
            let snapshot_summary = store
                .load_orchestration_session(plan.orchestration_session_id())?
                .map(|session| {
                    format!(
                        "state={:?}, posture={:?}, attached_participant_id={:?}, shell_owner_pid={}",
                        session.state,
                        session.posture,
                        session.attached_participant_id,
                        session.shell_owner_pid,
                    )
                })
                .unwrap_or_else(|| "session_missing".to_string());
            anyhow::bail!(
                "timed out waiting for detached start normalization for orchestration session {} after hidden owner-helper {} exited ({snapshot_summary})",
                plan.orchestration_session_id(),
                child.id(),
            );
        }

        thread::sleep(START_DETACH_NORMALIZATION_POLL_INTERVAL);
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

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum PrivateCancelOutcome {
    Accepted,
    AlreadyTerminal,
    OwnerUnreachable,
    ProtocolError,
}

#[derive(Debug)]
pub(crate) struct PrivateStopRequest {
    pub response_tx: oneshot::Sender<PrivateStopOutcome>,
}

#[derive(Debug)]
pub(crate) struct PrivateCancelRequest {
    pub payload: WorkerCancelPayloadV1,
    pub response_tx: oneshot::Sender<PrivateCancelOutcome>,
}

pub(crate) type PrivateStopRequestReceiver = mpsc::UnboundedReceiver<PrivateStopRequest>;
pub(crate) type PrivateStopRequestSender = mpsc::UnboundedSender<PrivateStopRequest>;
pub(crate) type PrivateCancelRequestReceiver = mpsc::UnboundedReceiver<PrivateCancelRequest>;
pub(crate) type PrivateCancelRequestSender = mpsc::UnboundedSender<PrivateCancelRequest>;

#[derive(Debug)]
pub(crate) struct PrivateStopTransport {
    shutdown_tx: Option<oneshot::Sender<()>>,
    task: Option<tokio::task::JoinHandle<()>>,
    path: PathBuf,
}

#[derive(Debug)]
pub(crate) struct PrivateCancelTransport {
    shutdown_tx: Option<oneshot::Sender<()>>,
    task: Option<tokio::task::JoinHandle<()>>,
    path: PathBuf,
}

#[derive(Debug)]
pub(crate) struct PrivatePromptRequest {
    pub action: PublicPromptAction,
    pub prompt: String,
    pub envelope_tx: mpsc::UnboundedSender<PublicPromptEnvelope>,
}

pub(crate) type PrivatePromptRequestReceiver = mpsc::UnboundedReceiver<PrivatePromptRequest>;
pub(crate) type PrivatePromptRequestSender = mpsc::UnboundedSender<PrivatePromptRequest>;

#[derive(Debug)]
pub(crate) struct PrivatePromptTransport {
    shutdown_tx: Option<oneshot::Sender<()>>,
    task: Option<tokio::task::JoinHandle<()>>,
    path: PathBuf,
}

#[cfg(unix)]
pub(crate) struct StartupPromptTransportListener {
    listener: StdUnixListener,
    path: PathBuf,
}

#[cfg(unix)]
impl StartupPromptTransportListener {
    pub(crate) fn path(&self) -> &Path {
        &self.path
    }

    async fn remove_path(path: &Path) {
        let _ = tokio::fs::remove_file(path).await;
    }
}

impl PrivatePromptTransport {
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

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub(crate) enum PublicPromptEnvelope {
    Accepted {
        version: u8,
        action: PublicPromptAction,
        orchestration_session_id: String,
        backend_id: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        participant_id: Option<String>,
        scope: String,
    },
    Event {
        version: u8,
        event_kind: String,
        data: serde_json::Value,
    },
    Warning {
        version: u8,
        message: String,
    },
    Completed {
        version: u8,
        action: PublicPromptAction,
        orchestration_session_id: String,
        backend_id: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        participant_id: Option<String>,
        turn_outcome: String,
        session_posture: PublicSessionPosture,
        state: String,
        warnings: Vec<String>,
    },
    Failed {
        version: u8,
        terminal: bool,
        stage: String,
        error_code: String,
        message: String,
    },
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

impl PrivateCancelTransport {
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

pub(crate) fn hidden_owner_helper_attach_lock_path(
    store: &AgentRuntimeStateStore,
    orchestration_session_id: &str,
) -> PathBuf {
    let session_fragment = compact_stop_transport_fragment(orchestration_session_id);
    store
        .handles_dir()
        .join("owner-helper")
        .join(format!("{session_fragment}.attach.lock"))
}

#[cfg(unix)]
pub(crate) fn hidden_owner_helper_startup_prompt_stream_path(
    store: &AgentRuntimeStateStore,
    orchestration_session_id: &str,
    participant_id: &str,
) -> PathBuf {
    let session_fragment = compact_stop_transport_fragment(orchestration_session_id);
    let participant_fragment = compact_stop_transport_fragment(participant_id);
    let socket_name = format!("{session_fragment}-{participant_fragment}.startup.sock");
    let preferred = store.handles_dir().join("startup").join(&socket_name);
    if preferred.as_os_str().len() > PRIVATE_STOP_UNIX_PATH_MAX {
        return PathBuf::from("/tmp")
            .join("substrate-agent-hub-startup")
            .join(socket_name);
    }
    preferred
}

pub(crate) fn toolbox_transport_path_for_home(
    substrate_home: &Path,
    orchestration_session_id: &str,
) -> PathBuf {
    let socket_name = format!("{orchestration_session_id}.sock");
    let preferred = substrate_home
        .join("run")
        .join("agent-toolbox")
        .join(&socket_name);
    if preferred.as_os_str().len() > PRIVATE_STOP_UNIX_PATH_MAX {
        return PathBuf::from("/tmp")
            .join("substrate-agent-toolbox")
            .join(socket_name);
    }
    preferred
}

pub(crate) fn toolbox_transport_path(orchestration_session_id: &str) -> Result<PathBuf> {
    Ok(toolbox_transport_path_for_home(
        &substrate_paths::substrate_home()?,
        orchestration_session_id,
    ))
}

pub(crate) fn toolbox_endpoint(orchestration_session_id: &str) -> Result<String> {
    Ok(format!(
        "unix://{}",
        toolbox_transport_path(orchestration_session_id)?.display()
    ))
}

#[cfg(unix)]
pub(crate) fn register_hidden_owner_helper_startup_prompt_listener(
    store: &AgentRuntimeStateStore,
    orchestration_session_id: &str,
    participant_id: &str,
) -> Result<StartupPromptTransportListener> {
    let path = hidden_owner_helper_startup_prompt_stream_path(
        store,
        orchestration_session_id,
        participant_id,
    );
    let parent = path.parent().ok_or_else(|| {
        anyhow::anyhow!(
            "startup prompt transport path '{}' is missing a parent directory",
            path.display()
        )
    })?;
    fs::create_dir_all(parent).with_context(|| format!("failed to create {}", parent.display()))?;
    remove_existing_stop_transport_path(&path)?;
    let listener = StdUnixListener::bind(&path)
        .with_context(|| format!("failed to bind startup prompt transport {}", path.display()))?;
    listener.set_nonblocking(true).with_context(|| {
        format!(
            "failed to configure startup prompt transport {}",
            path.display()
        )
    })?;
    Ok(StartupPromptTransportListener { listener, path })
}

#[cfg(unix)]
pub(crate) async fn consume_hidden_owner_helper_startup_prompt_stream<F>(
    listener: StartupPromptTransportListener,
    mut on_envelope: F,
) -> Result<i32>
where
    F: FnMut(&PublicPromptEnvelope) -> Result<()>,
{
    let StartupPromptTransportListener { listener, path } = listener;
    let tokio_listener = tokio::net::UnixListener::from_std(listener).with_context(|| {
        format!(
            "failed to activate startup prompt transport {}",
            path.display()
        )
    })?;
    let accept =
        tokio::time::timeout(STARTUP_PROMPT_STREAM_ACCEPT_TIMEOUT, tokio_listener.accept())
            .await
            .map_err(|_| {
                anyhow::anyhow!(
                    "stream_bridge_failed: timed out waiting for hidden owner-helper startup prompt stream {}",
                    path.display()
                )
            })?;
    let (stream, _) = accept.with_context(|| {
        format!(
            "stream_bridge_failed: failed to accept hidden owner-helper startup prompt stream {}",
            path.display()
        )
    })?;

    let mut reader = BufReader::new(stream);
    let mut line = String::new();
    let mut saw_accept = false;
    loop {
        line.clear();
        let bytes_read = reader.read_line(&mut line).await?;
        if bytes_read == 0 {
            break;
        }
        let envelope: PublicPromptEnvelope =
            serde_json::from_str(line.trim()).with_context(|| {
                format!(
                    "failed to decode hidden owner-helper startup prompt envelope from {}",
                    path.display()
                )
            })?;
        if matches!(envelope, PublicPromptEnvelope::Accepted { .. }) {
            saw_accept = true;
        }
        on_envelope(&envelope)?;
        match envelope {
            PublicPromptEnvelope::Completed { turn_outcome, .. } => {
                StartupPromptTransportListener::remove_path(&path).await;
                return Ok(completed_exit_code(turn_outcome.as_str()));
            }
            PublicPromptEnvelope::Failed { message, .. } => {
                StartupPromptTransportListener::remove_path(&path).await;
                return Err(anyhow::anyhow!(message));
            }
            _ => {}
        }
    }
    StartupPromptTransportListener::remove_path(&path).await;

    if saw_accept {
        anyhow::bail!("owner_unreachable: startup prompt stream ended after accepting the request");
    }
    anyhow::bail!(
        "owner_unreachable: hidden owner-helper startup prompt stream ended before accepting the request"
    );
}

#[cfg(unix)]
pub(crate) fn run_hidden_owner_helper_startup_prompt_stream_with_action(
    listener: StartupPromptTransportListener,
    json: bool,
    action: PublicPromptAction,
) -> Result<()> {
    run_hidden_owner_helper_startup_prompt_stream_with_projection(
        listener, json, action, None, None,
    )
}

#[cfg(unix)]
pub(crate) fn run_hidden_owner_helper_startup_prompt_stream_with_public_identity(
    listener: StartupPromptTransportListener,
    json: bool,
    action: PublicPromptAction,
    backend_id: &str,
    scope: AgentExecutionScope,
) -> Result<()> {
    run_hidden_owner_helper_startup_prompt_stream_with_projection(
        listener,
        json,
        action,
        Some(backend_id),
        Some(scope),
    )
}

#[cfg(unix)]
fn run_hidden_owner_helper_startup_prompt_stream_with_projection(
    listener: StartupPromptTransportListener,
    json: bool,
    action: PublicPromptAction,
    backend_id_override: Option<&str>,
    scope_override: Option<AgentExecutionScope>,
) -> Result<()> {
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .context("failed to initialize startup prompt transport runtime")?;
    let mut renderer = PublicPromptRenderer::new(json);
    let mut stream_started = false;
    let mut saw_terminal = false;
    let result = rt.block_on(async {
        consume_hidden_owner_helper_startup_prompt_stream(listener, |envelope| {
            stream_started = true;
            if matches!(
                envelope,
                PublicPromptEnvelope::Completed { .. } | PublicPromptEnvelope::Failed { .. }
            ) {
                saw_terminal = true;
            }
            let rewritten = rewrite_startup_prompt_envelope_action(
                envelope,
                action,
                backend_id_override,
                scope_override,
            );
            renderer.render(&rewritten)
        })
        .await
    });

    match result {
        Ok(0) => Ok(()),
        Ok(code) => Err(anyhow::Error::new(PublicPromptRenderedExit {
            exit_code: code,
        })),
        Err(err) if stream_started && action == PublicPromptAction::Turn => {
            if !saw_terminal {
                renderer.render(&failed_prompt_envelope(
                    "bridge",
                    "owner_unreachable",
                    err.to_string(),
                ))?;
            }
            Err(anyhow::Error::new(PublicPromptRenderedExit {
                exit_code: 1,
            }))
        }
        Err(err) => Err(err),
    }
}

#[cfg(unix)]
fn rewrite_startup_prompt_envelope_action(
    envelope: &PublicPromptEnvelope,
    action: PublicPromptAction,
    backend_id_override: Option<&str>,
    scope_override: Option<AgentExecutionScope>,
) -> PublicPromptEnvelope {
    match envelope {
        PublicPromptEnvelope::Accepted {
            version,
            orchestration_session_id,
            backend_id,
            participant_id,
            scope,
            ..
        } => PublicPromptEnvelope::Accepted {
            version: *version,
            action,
            orchestration_session_id: orchestration_session_id.clone(),
            backend_id: backend_id_override
                .unwrap_or(backend_id.as_str())
                .to_string(),
            participant_id: participant_id.clone(),
            scope: scope_override
                .map(scope_label)
                .unwrap_or(scope.as_str())
                .to_string(),
        },
        PublicPromptEnvelope::Completed {
            version,
            orchestration_session_id,
            backend_id,
            participant_id,
            turn_outcome,
            session_posture,
            state,
            warnings,
            ..
        } => PublicPromptEnvelope::Completed {
            version: *version,
            action,
            orchestration_session_id: orchestration_session_id.clone(),
            backend_id: backend_id_override
                .unwrap_or(backend_id.as_str())
                .to_string(),
            participant_id: participant_id.clone(),
            turn_outcome: turn_outcome.clone(),
            session_posture: *session_posture,
            state: state.clone(),
            warnings: warnings.clone(),
        },
        _ => envelope.clone(),
    }
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
        let readiness = store.classify_hidden_owner_helper_launch_readiness(
            plan.orchestration_session_id(),
            plan.participant_id(),
            plan.requires_internal_session_id(),
        )?;
        let startup_prompt_ready = if plan.startup_prompt.is_some() {
            start_launch_startup_prompt_is_accepted_or_terminal(store, plan)?
        } else {
            true
        };
        if startup_prompt_ready
            && (readiness == super::state_store::HiddenOwnerHelperLaunchReadiness::ReadyAttached
                || (matches!(
                    plan.mode,
                    OwnerHelperMode::Start | OwnerHelperMode::ResumeOneTurn
                ) && matches!(
                    readiness,
                    super::state_store::HiddenOwnerHelperLaunchReadiness::ReadyDetached(_)
                )))
        {
            return Ok(());
        }
        if started_at.elapsed() >= OWNER_HELPER_READY_TIMEOUT {
            anyhow::bail!(
                "{}{}",
                OWNER_HELPER_READY_TIMEOUT_ERROR_PREFIX,
                plan.orchestration_session_id(),
            );
        }
        thread::sleep(OWNER_HELPER_READY_POLL_INTERVAL);
    }
}

pub(crate) fn hidden_owner_helper_readiness_timed_out(err: &anyhow::Error) -> bool {
    err.to_string()
        .starts_with(OWNER_HELPER_READY_TIMEOUT_ERROR_PREFIX)
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum HiddenOwnerHelperStartTimeoutReconciliation {
    Success,
    FailureMarkedTerminal,
    FailureUnchanged,
}

pub(crate) fn reconcile_hidden_owner_helper_start_timeout(
    store: &AgentRuntimeStateStore,
    plan: &HiddenOwnerHelperLaunchPlan,
) -> Result<HiddenOwnerHelperStartTimeoutReconciliation> {
    if plan.mode != OwnerHelperMode::Start {
        return Ok(HiddenOwnerHelperStartTimeoutReconciliation::FailureUnchanged);
    }

    if start_launch_startup_prompt_is_terminal(store, plan)? {
        match store.classify_hidden_owner_helper_launch_readiness(
            plan.orchestration_session_id(),
            plan.participant_id(),
            plan.requires_internal_session_id(),
        )? {
            super::state_store::HiddenOwnerHelperLaunchReadiness::ReadyAttached
            | super::state_store::HiddenOwnerHelperLaunchReadiness::ReadyDetached(_) => {
                return Ok(HiddenOwnerHelperStartTimeoutReconciliation::Success);
            }
            super::state_store::HiddenOwnerHelperLaunchReadiness::Pending => {}
        }
    }

    let Some(session) = store.load_orchestration_session(plan.orchestration_session_id())? else {
        return Ok(HiddenOwnerHelperStartTimeoutReconciliation::FailureUnchanged);
    };
    let Some(participant) = store.load_participant(plan.participant_id())? else {
        return Ok(HiddenOwnerHelperStartTimeoutReconciliation::FailureUnchanged);
    };

    if startup_prompt_is_terminal_for_participant(&session, plan.participant_id()) {
        if let Some((next_session, next_participant)) = build_detached_start_reconciliation(
            &session,
            &participant,
            plan.requires_internal_session_id(),
        ) {
            if next_session != session {
                store.persist_orchestration_session(&next_session)?;
            }
            if next_participant != participant {
                store.persist_participant(&next_participant)?;
            }

            match store.classify_hidden_owner_helper_launch_readiness(
                plan.orchestration_session_id(),
                plan.participant_id(),
                plan.requires_internal_session_id(),
            )? {
                super::state_store::HiddenOwnerHelperLaunchReadiness::ReadyAttached
                | super::state_store::HiddenOwnerHelperLaunchReadiness::ReadyDetached(_) => {
                    return Ok(HiddenOwnerHelperStartTimeoutReconciliation::Success);
                }
                super::state_store::HiddenOwnerHelperLaunchReadiness::Pending => {}
            }
        }
    }

    if should_mark_start_timeout_terminal_failure(&session, &participant, plan.participant_id()) {
        let reason = format!(
            "hidden owner-helper readiness timed out during start for orchestration session {}",
            plan.orchestration_session_id(),
        );
        let mut failed_session = session;
        let mut failed_participant = participant;
        persist_start_timeout_terminal_failure(
            store,
            &mut failed_session,
            &mut failed_participant,
            &reason,
        )?;
        return Ok(HiddenOwnerHelperStartTimeoutReconciliation::FailureMarkedTerminal);
    }

    Ok(HiddenOwnerHelperStartTimeoutReconciliation::FailureUnchanged)
}

#[cfg(unix)]
pub(crate) fn reconcile_resumed_public_turn_detach_timeout(
    store: &AgentRuntimeStateStore,
    orchestration_session_id: &str,
    resumed_participant_id: &str,
) -> Result<bool> {
    if store
        .resumed_public_turn_detach_posture(orchestration_session_id, resumed_participant_id)?
        .is_some()
    {
        return Ok(true);
    }

    let Some(session) = store.load_orchestration_session(orchestration_session_id)? else {
        return Ok(false);
    };
    let Some(participant) = store.load_participant(resumed_participant_id)? else {
        return Ok(false);
    };

    let Some((next_session, next_participant)) =
        build_resumed_public_turn_detach_reconciliation(&session, &participant)
    else {
        return Ok(false);
    };

    if next_session != session {
        store.persist_orchestration_session(&next_session)?;
    }
    if next_participant != participant {
        store.persist_participant(&next_participant)?;
    }

    Ok(store
        .resumed_public_turn_detach_posture(orchestration_session_id, resumed_participant_id)?
        .is_some())
}

#[cfg(unix)]
fn build_resumed_public_turn_detach_reconciliation(
    session: &OrchestrationSessionRecord,
    participant: &AgentRuntimeSessionManifest,
) -> Option<(OrchestrationSessionRecord, AgentRuntimeSessionManifest)> {
    build_detached_start_reconciliation(session, participant, true)
}

#[cfg(unix)]
pub(crate) fn reconcile_start_prompt_completion_timeout(
    store: &AgentRuntimeStateStore,
    orchestration_session_id: &str,
    participant_id: &str,
) -> Result<bool> {
    if matches!(
        store.classify_hidden_owner_helper_launch_readiness(
            orchestration_session_id,
            participant_id,
            true,
        )?,
        super::state_store::HiddenOwnerHelperLaunchReadiness::ReadyDetached(_)
    ) {
        return Ok(true);
    }

    let Some(session) = store.load_orchestration_session(orchestration_session_id)? else {
        return Ok(false);
    };
    let Some(participant) = store.load_participant(participant_id)? else {
        return Ok(false);
    };

    if !startup_prompt_is_terminal_for_participant(&session, participant_id) {
        return Ok(false);
    }

    let Some((next_session, next_participant)) =
        build_detached_start_reconciliation(&session, &participant, true)
    else {
        return Ok(false);
    };

    if next_session != session {
        store.persist_orchestration_session(&next_session)?;
    }
    if next_participant != participant {
        store.persist_participant(&next_participant)?;
    }

    Ok(matches!(
        store.classify_hidden_owner_helper_launch_readiness(
            orchestration_session_id,
            participant_id,
            true,
        )?,
        super::state_store::HiddenOwnerHelperLaunchReadiness::ReadyDetached(_)
    ))
}

fn start_launch_startup_prompt_is_accepted_or_terminal(
    store: &AgentRuntimeStateStore,
    plan: &HiddenOwnerHelperLaunchPlan,
) -> Result<bool> {
    if plan.startup_prompt.is_none() {
        return Ok(true);
    }

    Ok(matches!(
        store
            .startup_prompt_replay_state(plan.orchestration_session_id(), plan.participant_id(),)?,
        super::StartupPromptReplayState::AcceptedOrTerminal
    ))
}

fn start_launch_startup_prompt_is_terminal(
    store: &AgentRuntimeStateStore,
    plan: &HiddenOwnerHelperLaunchPlan,
) -> Result<bool> {
    if plan.startup_prompt.is_none() {
        return Ok(true);
    }

    let Some(session) = store.load_orchestration_session(plan.orchestration_session_id())? else {
        return Ok(false);
    };
    Ok(startup_prompt_is_terminal_for_participant(
        &session,
        plan.participant_id(),
    ))
}

fn startup_prompt_is_terminal_for_participant(
    session: &OrchestrationSessionRecord,
    participant_id: &str,
) -> bool {
    matches!(
        session.startup_prompt.as_ref(),
        Some(startup_prompt)
            if startup_prompt.participant_id == participant_id
                && matches!(
                    startup_prompt.state,
                    StartupPromptStreamState::Completed | StartupPromptStreamState::Failed
                )
    )
}

fn build_detached_start_reconciliation(
    session: &OrchestrationSessionRecord,
    participant: &AgentRuntimeSessionManifest,
    require_internal_session_id: bool,
) -> Option<(OrchestrationSessionRecord, AgentRuntimeSessionManifest)> {
    if session.state != OrchestrationSessionState::Active {
        return None;
    }
    if session.active_participant_id() != Some(participant.participant_id()) {
        return None;
    }
    if !participant.matches_public_parent_linkage(session)
        || !participant.is_host_orchestrator()
        || !participant.handle.state.is_live()
    {
        return None;
    }

    let mut detached_session = session.clone();
    let mut detached_participant = participant.clone();
    detached_session.shell_owner_pid = 0;
    detached_participant.release_runtime_ownership();
    detached_participant.mark_client_detached("owner detached cleanly");
    detached_participant.internal.shell_owner_pid = 0;
    detached_participant.touch_heartbeat();
    detached_session.transition_state(OrchestrationSessionState::Active);
    if detached_session.pending_inbox_count > 0 {
        detached_session.mark_awaiting_attention();
    } else {
        detached_session.mark_parked_resumable("owner detached cleanly");
    }

    super::state_store::valid_detached_host_continuity_posture(
        &detached_session,
        &detached_participant,
        require_internal_session_id,
    )?;
    Some((detached_session, detached_participant))
}

fn should_mark_start_timeout_terminal_failure(
    session: &OrchestrationSessionRecord,
    participant: &AgentRuntimeSessionManifest,
    participant_id: &str,
) -> bool {
    if session.state.is_terminal()
        || !participant.handle.state.is_live()
        || participant.participant_id() != participant_id
        || !participant.matches_public_parent_linkage(session)
    {
        return false;
    }

    session.attached_participant_id() == Some(participant_id)
        || participant.attached_client_present()
        || matches!(
            session.startup_prompt.as_ref(),
            Some(startup_prompt) if startup_prompt.participant_id == participant_id
        )
}

fn persist_start_timeout_terminal_failure(
    store: &AgentRuntimeStateStore,
    session: &mut OrchestrationSessionRecord,
    participant: &mut AgentRuntimeSessionManifest,
    reason: &str,
) -> Result<()> {
    session.mark_startup_prompt_failed(participant.participant_id(), reason.to_string());
    if participant.handle.state.is_live() {
        let next_state = if participant.internal_uaa_session_id().is_some()
            || participant.handle.state != AgentRuntimeSessionState::Allocating
        {
            AgentRuntimeSessionState::Invalidated
        } else {
            AgentRuntimeSessionState::Failed
        };
        participant.transition_state(next_state);
    }
    participant.mark_terminal_state(reason.to_string());
    participant.internal.last_error_bucket = Some("bootstrap_run".to_string());
    participant.internal.last_error_message = Some(reason.to_string());
    participant.touch_heartbeat();
    if !session.state.is_terminal() {
        session.transition_state(OrchestrationSessionState::Failed);
    }
    session.mark_terminal(reason.to_string());
    persist_runtime_snapshots(store, session, participant)
}

pub(crate) fn private_stop_request_channel(
) -> (PrivateStopRequestSender, PrivateStopRequestReceiver) {
    mpsc::unbounded_channel()
}

pub(crate) fn private_cancel_request_channel(
) -> (PrivateCancelRequestSender, PrivateCancelRequestReceiver) {
    mpsc::unbounded_channel()
}

pub(crate) fn runtime_is_terminal(manifest: &Arc<Mutex<AgentRuntimeSessionManifest>>) -> bool {
    manifest
        .lock()
        .expect("runtime manifest mutex poisoned")
        .internal
        .terminal_observed_at
        .is_some()
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
    let mut orchestration_snapshot = orchestration_session.clone();
    orchestration_snapshot.sync_host_attach_contract(manifest);
    store.persist_orchestration_session(&orchestration_snapshot)?;
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
        if manifest_guard.handle.state.is_live() {
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

pub(crate) fn apply_runtime_stop_closeout(
    orchestration_session: &mut OrchestrationSessionRecord,
    manifest: &mut AgentRuntimeSessionManifest,
) {
    manifest.transition_state(AgentRuntimeSessionState::Stopped);
    manifest.mark_terminal_state("stopped");
    manifest.touch_heartbeat();
    if runtime_controls_parent_session(&manifest.handle.role) {
        orchestration_session.transition_state(OrchestrationSessionState::Stopped);
        orchestration_session.mark_terminal("stopped");
    } else {
        orchestration_session.touch_active();
    }
}

pub(crate) fn apply_runtime_cancel_closeout(
    orchestration_session: &mut OrchestrationSessionRecord,
    manifest: &mut AgentRuntimeSessionManifest,
) {
    manifest.mark_cancelled_terminal_state();
    manifest.touch_heartbeat();
    orchestration_session.mark_cancelled_terminal();
}

pub(crate) fn persist_runtime_stop_closeout(
    store: &AgentRuntimeStateStore,
    orchestration_session: &mut OrchestrationSessionRecord,
    manifest: &mut AgentRuntimeSessionManifest,
) -> Result<()> {
    apply_runtime_stop_closeout(orchestration_session, manifest);
    persist_runtime_snapshots(store, orchestration_session, manifest)
}

#[allow(dead_code)]
pub(crate) fn persist_runtime_cancel_closeout(
    store: &AgentRuntimeStateStore,
    orchestration_session: &mut OrchestrationSessionRecord,
    manifest: &mut AgentRuntimeSessionManifest,
) -> Result<()> {
    apply_runtime_cancel_closeout(orchestration_session, manifest);
    persist_runtime_snapshots(store, orchestration_session, manifest)
}

#[cfg(unix)]
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

#[cfg(unix)]
pub(crate) fn private_cancel_transport_path(
    store: &AgentRuntimeStateStore,
    orchestration_session_id: &str,
    participant_id: &str,
) -> PathBuf {
    let session_fragment = compact_stop_transport_fragment(orchestration_session_id);
    let participant_fragment = compact_stop_transport_fragment(participant_id);
    let socket_name = format!("{session_fragment}-{participant_fragment}.cancel.sock");
    let preferred = store.handles_dir().join("cancel").join(&socket_name);
    #[cfg(unix)]
    if preferred.as_os_str().len() > PRIVATE_STOP_UNIX_PATH_MAX {
        return PathBuf::from("/tmp")
            .join("substrate-agent-hub-cancel")
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

#[cfg(unix)]
pub(crate) async fn register_private_cancel_transport(
    store: &AgentRuntimeStateStore,
    orchestration_session_id: &str,
    participant_id: &str,
    cancel_tx: PrivateCancelRequestSender,
) -> Result<PrivateCancelTransport> {
    let path = private_cancel_transport_path(store, orchestration_session_id, participant_id);
    let parent = path.parent().ok_or_else(|| {
        anyhow::anyhow!(
            "private cancel transport path '{}' is missing a parent directory",
            path.display()
        )
    })?;
    fs::create_dir_all(parent).with_context(|| format!("failed to create {}", parent.display()))?;
    remove_existing_private_cancel_transport_path(&path)?;
    let listener = UnixListener::bind(&path)
        .with_context(|| format!("failed to bind private cancel transport {}", path.display()))?;
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
                    let cancel_tx = cancel_tx.clone();
                    tokio::spawn(async move {
                        let _ = handle_private_cancel_connection(stream, cancel_tx).await;
                    });
                }
            }
        }
        let _ = tokio::fs::remove_file(&path_for_task).await;
    });
    Ok(PrivateCancelTransport {
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

#[cfg(not(unix))]
pub(crate) async fn register_private_cancel_transport(
    _store: &AgentRuntimeStateStore,
    _orchestration_session_id: &str,
    _participant_id: &str,
    _cancel_tx: PrivateCancelRequestSender,
) -> Result<PrivateCancelTransport> {
    Ok(PrivateCancelTransport {
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
    cancel: PromptFulfillmentCancelHandle,
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

pub(crate) fn spawn_local_private_cancel_owner(
    manifest: Arc<Mutex<AgentRuntimeSessionManifest>>,
    shutdown_requested: Arc<AtomicBool>,
    cancel_requested: Arc<AtomicBool>,
    cancel: PromptFulfillmentCancelHandle,
    mut cancel_rx: PrivateCancelRequestReceiver,
) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        while let Some(request) = cancel_rx.recv().await {
            let outcome = if runtime_is_terminal(&manifest) {
                PrivateCancelOutcome::AlreadyTerminal
            } else {
                shutdown_requested.store(true, Ordering::SeqCst);
                cancel_requested.store(true, Ordering::SeqCst);
                cancel.cancel();
                PrivateCancelOutcome::Accepted
            };
            let _ = request.response_tx.send(outcome);
        }
    })
}

pub(crate) fn private_prompt_request_channel(
) -> (PrivatePromptRequestSender, PrivatePromptRequestReceiver) {
    mpsc::unbounded_channel()
}

#[cfg(unix)]
pub(crate) fn private_prompt_transport_path(
    store: &AgentRuntimeStateStore,
    orchestration_session_id: &str,
    participant_id: &str,
) -> PathBuf {
    let session_fragment = compact_stop_transport_fragment(orchestration_session_id);
    let participant_fragment = compact_stop_transport_fragment(participant_id);
    let socket_name = format!("{session_fragment}-{participant_fragment}.prompt.sock");
    let preferred = store.handles_dir().join("prompt").join(&socket_name);
    #[cfg(unix)]
    if preferred.as_os_str().len() > PRIVATE_STOP_UNIX_PATH_MAX {
        return PathBuf::from("/tmp")
            .join("substrate-agent-hub-prompt")
            .join(socket_name);
    }
    preferred
}

pub(crate) fn prompt_runtime_from_parts(
    descriptor: RuntimeSelectionDescriptor,
    orchestration_session: Arc<Mutex<OrchestrationSessionRecord>>,
    manifest: Arc<Mutex<AgentRuntimeSessionManifest>>,
    store: AgentRuntimeStateStore,
    uaa_session_handle_id: String,
    park_after_turn_tx: Option<mpsc::UnboundedSender<()>>,
    host_toolbox_surface_authoritative: Arc<AtomicBool>,
) -> PromptSubmitRuntime {
    PromptSubmitRuntime {
        descriptor,
        orchestration_session,
        manifest,
        store,
        uaa_session_handle_id,
        park_after_turn_tx,
        host_toolbox_surface_authoritative,
    }
}

pub(crate) fn authoritative_host_toolbox_surface_enabled(
    backend_id: &str,
    execution_scope: AgentExecutionScope,
    role: &str,
    effective_config: &SubstrateConfig,
    base_policy: &Policy,
) -> bool {
    execution_scope == AgentExecutionScope::Host
        && role == ORCHESTRATOR_ROLE
        && effective_config.agents.enabled
        && effective_config.agents.toolbox.enabled
        && matches!(
            effective_config.agents.toolbox.bind.transport,
            AgentToolboxBindTransport::Uds
        )
        && backend_allowed(base_policy, backend_id)
}

pub(crate) fn maybe_compose_prompt_with_authoritative_host_toolbox_contract(
    prompt: &str,
    host_toolbox_surface_authoritative: bool,
) -> String {
    if host_toolbox_surface_authoritative {
        compose_prompt_with_host_toolbox_contract(prompt)
    } else {
        prompt.to_string()
    }
}

pub(crate) fn maybe_build_runtime_owned_toolbox_env(
    orchestration_session_id: &str,
    host_toolbox_surface_authoritative: bool,
) -> Result<BTreeMap<String, String>> {
    if host_toolbox_surface_authoritative {
        Ok(build_runtime_owned_toolbox_env(toolbox_endpoint(
            orchestration_session_id,
        )?))
    } else {
        Ok(BTreeMap::new())
    }
}

pub(crate) async fn submit_host_prompt_turn<F>(
    runtime: &PromptSubmitRuntime,
    run_id: &str,
    prompt: &str,
    mut on_event: F,
) -> Result<SubmittedPromptCompletion>
where
    F: FnMut(SubmittedPromptStreamEvent),
{
    let prompt_fulfillment = super::build_gateway_for_descriptor(&runtime.descriptor)
        .context("build host targeted-turn gateway")?;
    let orchestration_session_id = runtime
        .manifest
        .lock()
        .expect("runtime manifest mutex poisoned")
        .handle
        .orchestration_session_id
        .clone();
    let host_toolbox_surface_authoritative = runtime
        .host_toolbox_surface_authoritative
        .load(Ordering::SeqCst);
    let request_prompt = maybe_compose_prompt_with_authoritative_host_toolbox_contract(
        prompt,
        host_toolbox_surface_authoritative,
    );
    let continuity_session_id = prompt_submit_continuity_session_id(runtime);

    let request = agent_api::AgentWrapperRunRequest {
        prompt: request_prompt,
        working_dir: Some(std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))),
        timeout: None,
        env: maybe_build_runtime_owned_toolbox_env(
            &orchestration_session_id,
            host_toolbox_surface_authoritative,
        )?,
        extensions: std::collections::BTreeMap::from([(
            AGENT_API_SESSION_RESUME_V1.to_string(),
            build_session_resume_extension(&continuity_session_id),
        )]),
    };
    let control = prompt_fulfillment
        .run_control(request)
        .await
        .map_err(|err| anyhow::anyhow!("substrate: error: {}", err))?;
    let agent_api::AgentWrapperRunHandle {
        mut events,
        completion,
    } = control.handle;

    while let Some(wrapper_event) = events.next().await {
        let (orchestration_snapshot, manifest_snapshot, event) = {
            let mut orchestration_guard = runtime
                .orchestration_session
                .lock()
                .expect("orchestration session mutex poisoned");
            let mut manifest_guard = runtime
                .manifest
                .lock()
                .expect("runtime manifest mutex poisoned");
            if let Some(session_id) = extract_session_handle_id(wrapper_event.data.as_ref()) {
                if manifest_guard.internal.uaa_session_id.as_deref() != Some(session_id) {
                    manifest_guard.set_uaa_session_id(session_id.to_string());
                }
            }
            manifest_guard.touch_event(Utc::now());
            orchestration_guard.touch_active();
            let event = translate_prompt_wrapper_event(
                &manifest_guard,
                &orchestration_guard,
                run_id,
                wrapper_event,
            );
            (orchestration_guard.clone(), manifest_guard.clone(), event)
        };
        persist_runtime_snapshots(&runtime.store, &orchestration_snapshot, &manifest_snapshot)?;
        on_event(SubmittedPromptStreamEvent::Agent(Box::new(event)));
    }

    let completion = completion
        .await
        .map_err(|err| anyhow::anyhow!("substrate: error: {}", err))?;
    if let Some(session_id) = extract_session_handle_id(completion.data.as_ref()) {
        let mut manifest_guard = runtime
            .manifest
            .lock()
            .expect("runtime manifest mutex poisoned");
        if manifest_guard.internal.uaa_session_id.as_deref() != Some(session_id) {
            manifest_guard.set_uaa_session_id(session_id.to_string());
        }
    }
    Ok(SubmittedPromptCompletion {
        exit_code: completion.status.code().unwrap_or(-1),
        warning: warning_for_exit_status(&completion.status),
    })
}

fn prompt_submit_continuity_session_id(runtime: &PromptSubmitRuntime) -> String {
    if let Some(session_id) = runtime
        .orchestration_session
        .lock()
        .expect("orchestration session mutex poisoned")
        .host_attach_contract()
        .and_then(HostAttachContract::public_attach_continuity_session_id)
        .map(str::to_owned)
    {
        return session_id;
    }

    if let Some(session_id) = runtime
        .manifest
        .lock()
        .expect("runtime manifest mutex poisoned")
        .internal_uaa_session_id()
        .map(str::to_owned)
    {
        return session_id;
    }

    runtime.uaa_session_handle_id.clone()
}

#[cfg(target_os = "linux")]
pub(crate) async fn submit_world_prompt_turn<F>(
    runtime: &PromptSubmitRuntime,
    run_id: &str,
    prompt: &str,
    mut on_event: F,
) -> Result<SubmittedPromptCompletion>
where
    F: FnMut(SubmittedPromptStreamEvent),
{
    use http_body_util::BodyExt as _;

    let request = {
        let manifest_guard = runtime
            .manifest
            .lock()
            .expect("runtime manifest mutex poisoned");
        MemberTurnSubmitRequestV1 {
            schema_version: 1,
            orchestration_session_id: manifest_guard.handle.orchestration_session_id.clone(),
            participant_id: manifest_guard.handle.participant_id.clone(),
            orchestrator_participant_id: manifest_guard
                .handle
                .orchestrator_participant_id
                .clone()
                .ok_or_else(|| {
                    anyhow::anyhow!(
                        "substrate: error: retained world-scoped member is missing orchestrator_participant_id"
                    )
                })?,
            backend_id: runtime.descriptor.backend_id.clone(),
            run_id: run_id.to_string(),
            world_id: manifest_guard.handle.world_id.clone().ok_or_else(|| {
                anyhow::anyhow!(
                    "substrate: error: retained world-scoped member is missing world_id"
                )
            })?,
            world_generation: manifest_guard.handle.world_generation.ok_or_else(|| {
                anyhow::anyhow!(
                    "substrate: error: retained world-scoped member is missing world_generation"
                )
            })?,
            prompt: prompt.to_string(),
            acceptance_context: None,
        }
    };

    let (client, _pending_diff_request, _agent_id) = build_agent_client_and_pending_diff_request()?;
    let response = client
        .submit_member_turn_stream(request)
        .await
        .map_err(|err| anyhow::anyhow!("substrate: error: {err:#}"))?;

    let mut body = std::pin::pin!(response.into_body());
    let mut buffer = Vec::new();
    let mut observed_exit: Option<i32> = None;
    while let Some(frame) = body.as_mut().frame().await {
        let frame = frame.map_err(|err| anyhow::anyhow!("substrate: error: {err:#}"))?;
        let Some(data) = frame.data_ref() else {
            continue;
        };
        buffer.extend_from_slice(data);

        while let Some(pos) = buffer.iter().position(|&byte| byte == b'\n') {
            let line: Vec<u8> = buffer.drain(..=pos).collect();
            if line.len() <= 1 {
                continue;
            }
            let payload = &line[..line.len() - 1];
            if payload.is_empty() {
                continue;
            }
            let frame = serde_json::from_slice::<ExecuteStreamFrame>(payload)
                .map_err(|err| anyhow::anyhow!("substrate: error: {err:#}"))?;
            match frame {
                ExecuteStreamFrame::Start { .. } => {}
                ExecuteStreamFrame::Event { event, .. } => {
                    on_event(SubmittedPromptStreamEvent::Agent(Box::new(event)));
                }
                ExecuteStreamFrame::Stdout { chunk_b64, .. } => {
                    let decoded = BASE64
                        .decode(chunk_b64.as_bytes())
                        .map_err(|err| anyhow::anyhow!("substrate: error: {err:#}"))?;
                    on_event(SubmittedPromptStreamEvent::Stdout(
                        String::from_utf8_lossy(&decoded).to_string(),
                    ));
                }
                ExecuteStreamFrame::Stderr { chunk_b64, .. } => {
                    let decoded = BASE64
                        .decode(chunk_b64.as_bytes())
                        .map_err(|err| anyhow::anyhow!("substrate: error: {err:#}"))?;
                    on_event(SubmittedPromptStreamEvent::Stderr(
                        String::from_utf8_lossy(&decoded).to_string(),
                    ));
                }
                ExecuteStreamFrame::Exit { exit, .. } => {
                    observed_exit = Some(exit);
                }
                ExecuteStreamFrame::Error { message, .. } => {
                    return Err(anyhow::anyhow!("substrate: error: {message}"));
                }
            }
        }
    }

    let exit_code = observed_exit.ok_or_else(|| {
        anyhow::anyhow!(
            "substrate: error: member turn stream ended without an exact terminal Exit frame"
        )
    })?;
    Ok(SubmittedPromptCompletion {
        exit_code,
        warning: warning_for_exit_code(exit_code),
    })
}

#[cfg(not(target_os = "linux"))]
pub(crate) async fn submit_world_prompt_turn<F>(
    _runtime: &PromptSubmitRuntime,
    _run_id: &str,
    _prompt: &str,
    _on_event: F,
) -> Result<SubmittedPromptCompletion>
where
    F: FnMut(SubmittedPromptStreamEvent),
{
    Err(anyhow::anyhow!(
        "substrate: error: world-targeted follow-up turns are supported on Linux only"
    ))
}

pub(crate) fn spawn_local_private_prompt_owner(
    runtime: PromptSubmitRuntime,
    mut prompt_rx: PrivatePromptRequestReceiver,
) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        while let Some(request) = prompt_rx.recv().await {
            let _ = stream_private_prompt_request(&runtime, request, false).await;
        }
    })
}

#[cfg(target_os = "linux")]
pub(crate) fn spawn_remote_private_prompt_owner(
    runtime: PromptSubmitRuntime,
    mut prompt_rx: PrivatePromptRequestReceiver,
) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        while let Some(request) = prompt_rx.recv().await {
            let _ = stream_private_prompt_request(&runtime, request, true).await;
        }
    })
}

#[cfg(not(target_os = "linux"))]
#[allow(dead_code)]
pub(crate) fn spawn_remote_private_prompt_owner(
    _runtime: PromptSubmitRuntime,
    mut prompt_rx: PrivatePromptRequestReceiver,
) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move { while prompt_rx.recv().await.is_some() {} })
}

#[allow(dead_code)]
pub(crate) fn load_public_prompt_source(input: &PublicPromptInput) -> Result<LoadedPublicPrompt> {
    let raw = match (&input.prompt, &input.prompt_file) {
        (Some(_), Some(_)) => anyhow::bail!(
            "malformed_prompt_source: provide exactly one of --prompt or --prompt-file"
        ),
        (Some(prompt), None) => prompt.clone(),
        (None, Some(path)) if path == Path::new("-") => {
            let mut buffer = String::new();
            io::stdin()
                .read_to_string(&mut buffer)
                .context("malformed_prompt_source: failed to read prompt from stdin")?;
            buffer
        }
        (None, Some(path)) => fs::read_to_string(path)
            .with_context(|| format!("malformed_prompt_source: failed to read {}", path.display()))?,
        (None, None) => anyhow::bail!(
            "missing_prompt_source: provide --prompt or --prompt-file (use --prompt-file - for stdin)"
        ),
    };

    let normalized = raw.trim_end_matches(['\r', '\n']).to_string();
    if normalized.trim().is_empty() {
        anyhow::bail!("empty_prompt: prompt input was empty");
    }
    Ok(LoadedPublicPrompt {
        prompt_text: normalized,
    })
}

#[allow(dead_code)]
pub(crate) fn run_public_prompt_command(
    request: PublicPromptCommandRequest,
    _cli_world: bool,
    _cli_no_world: bool,
) -> Result<()> {
    let store = AgentRuntimeStateStore::new()?;
    let (orchestration_session_id, backend_id) = validate_public_prompt_command_request(&request)?;
    let participant_id = match request.action {
        PublicPromptAction::Start => {
            resolve_public_start_prompt_target(&store, orchestration_session_id, backend_id)?
        }
        PublicPromptAction::Turn => {
            let resolved =
                store.resolve_public_turn_target(orchestration_session_id, backend_id)?;
            if resolved.session_posture != PublicSessionPosture::Active {
                anyhow::bail!(
                    "owner_unreachable: orchestration session {} backend {} is not currently attached to a live retained turn target",
                    orchestration_session_id,
                    backend_id
                );
            }
            resolved.participant.handle.participant_id.clone()
        }
    };

    #[cfg(not(unix))]
    {
        let _ = participant_id;
        anyhow::bail!(
            "unsupported_platform_or_posture: public prompt submission requires a Unix private owner transport"
        );
    }

    #[cfg(unix)]
    {
        let transport_path = private_prompt_transport_path(
            &store,
            orchestration_session_id,
            participant_id.as_str(),
        );
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .context("failed to initialize prompt transport runtime")?;
        let mut renderer = PublicPromptRenderer::new(request.json);
        let mut stream_started = false;
        let mut saw_terminal = false;
        let result = rt.block_on(async {
            wait_for_private_prompt_transport(&transport_path).await?;
            request_private_prompt_stream(
                &transport_path,
                request.action,
                &request.prompt.prompt_text,
                |envelope| {
                    stream_started = true;
                    if matches!(
                        envelope,
                        PublicPromptEnvelope::Completed { .. }
                            | PublicPromptEnvelope::Failed { .. }
                    ) {
                        saw_terminal = true;
                    }
                    renderer.render(envelope)
                },
            )
            .await
        });

        match result {
            Ok(0) => Ok(()),
            Ok(code) => Err(anyhow::Error::new(PublicPromptRenderedExit {
                exit_code: code,
            })),
            Err(err) if stream_started => {
                if !saw_terminal {
                    renderer.render(&failed_prompt_envelope(
                        "bridge",
                        "owner_unreachable",
                        err.to_string(),
                    ))?;
                }
                Err(anyhow::Error::new(PublicPromptRenderedExit {
                    exit_code: 1,
                }))
            }
            Err(err) => Err(err),
        }
    }
}

fn validate_public_prompt_command_request(
    request: &PublicPromptCommandRequest,
) -> Result<(&str, &str)> {
    let session_error = match request.action {
        PublicPromptAction::Start => {
            "runtime_start_failed: public start actions require an orchestration session id"
        }
        PublicPromptAction::Turn => {
            "unknown_session: public turn actions require --session <orchestration_session_id>"
        }
    };
    let backend_error = match request.action {
        PublicPromptAction::Start => {
            "runtime_start_failed: public start actions require an exact backend id"
        }
        PublicPromptAction::Turn => {
            "missing_backend: public turn actions require --backend <backend_id>"
        }
    };

    let orchestration_session_id = request
        .orchestration_session_id
        .as_deref()
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| anyhow::anyhow!(session_error))?;
    let backend_id = (!request.backend_id.trim().is_empty())
        .then_some(request.backend_id.as_str())
        .ok_or_else(|| anyhow::anyhow!(backend_error))?;

    Ok((orchestration_session_id, backend_id))
}

#[cfg(unix)]
#[allow(dead_code)]
pub(crate) async fn request_private_stop(path: &Path) -> Result<PrivateStopOutcome> {
    let mut stream = match UnixStream::connect(path).await {
        Ok(stream) => stream,
        Err(err) => {
            let kind = err.kind();
            return Err(anyhow::Error::new(err)
                .context(format_private_stop_transport_connect_error(path, kind)));
        }
    };
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
pub(crate) fn private_stop_transport_error_kind(err: &anyhow::Error) -> Option<io::ErrorKind> {
    err.chain()
        .find_map(|cause| cause.downcast_ref::<io::Error>().map(std::io::Error::kind))
}

#[cfg(unix)]
fn format_private_stop_transport_connect_error(path: &Path, kind: io::ErrorKind) -> String {
    let prefix = match kind {
        io::ErrorKind::NotFound => "missing_transport: ",
        io::ErrorKind::ConnectionRefused => "refused_transport: ",
        _ => "",
    };
    format!(
        "{prefix}failed to connect to private stop transport {}",
        path.display()
    )
}

#[cfg(unix)]
#[allow(dead_code)]
pub(crate) async fn request_private_cancel(
    path: &Path,
    payload: &WorkerCancelPayloadV1,
) -> Result<PrivateCancelOutcome> {
    let mut stream = UnixStream::connect(path).await.with_context(|| {
        format!(
            "failed to connect to private cancel transport {}",
            path.display()
        )
    })?;
    let request = PrivateCancelRequestV1 {
        version: 1,
        action: "cancel".to_string(),
        reason: payload.reason.clone(),
        graceful: payload.graceful,
    };
    stream
        .write_all(serde_json::to_string(&request)?.as_bytes())
        .await?;
    stream.write_all(b"\n").await?;
    stream.flush().await?;

    let mut reader = BufReader::new(stream);
    let mut line = String::new();
    reader.read_line(&mut line).await?;
    let response: PrivateCancelResponse = serde_json::from_str(line.trim()).with_context(|| {
        format!(
            "failed to decode private cancel transport response from {}",
            path.display()
        )
    })?;
    Ok(response.outcome)
}

#[cfg(unix)]
pub(crate) async fn register_private_prompt_transport(
    store: &AgentRuntimeStateStore,
    orchestration_session_id: &str,
    participant_id: &str,
    prompt_tx: PrivatePromptRequestSender,
) -> Result<PrivatePromptTransport> {
    let path = private_prompt_transport_path(store, orchestration_session_id, participant_id);
    let parent = path.parent().ok_or_else(|| {
        anyhow::anyhow!(
            "private prompt transport path '{}' is missing a parent directory",
            path.display()
        )
    })?;
    fs::create_dir_all(parent).with_context(|| format!("failed to create {}", parent.display()))?;
    remove_existing_stop_transport_path(&path)?;
    let listener = UnixListener::bind(&path)
        .with_context(|| format!("failed to bind private prompt transport {}", path.display()))?;
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
                    let prompt_tx = prompt_tx.clone();
                    tokio::spawn(async move {
                        let _ = handle_private_prompt_connection(stream, prompt_tx).await;
                    });
                }
            }
        }
        let _ = tokio::fs::remove_file(&path_for_task).await;
    });
    Ok(PrivatePromptTransport {
        shutdown_tx: Some(shutdown_tx),
        task: Some(task),
        path,
    })
}

#[cfg(not(unix))]
pub(crate) async fn register_private_prompt_transport(
    _store: &AgentRuntimeStateStore,
    _orchestration_session_id: &str,
    _participant_id: &str,
    _prompt_tx: PrivatePromptRequestSender,
) -> Result<PrivatePromptTransport> {
    Ok(PrivatePromptTransport {
        shutdown_tx: None,
        task: None,
        path: PathBuf::new(),
    })
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
fn remove_existing_private_cancel_transport_path(path: &Path) -> Result<()> {
    match fs::symlink_metadata(path) {
        Ok(_) => fs::remove_file(path).with_context(|| {
            format!(
                "failed to remove stale private cancel transport {}",
                path.display()
            )
        }),
        Err(err) if err.kind() == io::ErrorKind::NotFound => Ok(()),
        Err(err) => Err(err).with_context(|| {
            format!(
                "failed to inspect existing private cancel transport {}",
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
async fn handle_private_cancel_connection(
    stream: UnixStream,
    cancel_tx: PrivateCancelRequestSender,
) -> Result<()> {
    let mut reader = BufReader::new(stream);
    let mut line = String::new();
    let outcome = match reader.read_line(&mut line).await? {
        0 => PrivateCancelOutcome::ProtocolError,
        _ => match parse_private_cancel_request(line.trim()) {
            Ok(request) => {
                let (response_tx, response_rx) = oneshot::channel();
                let payload = WorkerCancelPayloadV1 {
                    reason: request.reason,
                    graceful: request.graceful,
                };
                if cancel_tx
                    .send(PrivateCancelRequest {
                        payload,
                        response_tx,
                    })
                    .is_err()
                {
                    PrivateCancelOutcome::OwnerUnreachable
                } else {
                    match tokio::time::timeout(Duration::from_secs(5), response_rx).await {
                        Ok(Ok(outcome)) => outcome,
                        Ok(Err(_)) | Err(_) => PrivateCancelOutcome::OwnerUnreachable,
                    }
                }
            }
            Err(_) => PrivateCancelOutcome::ProtocolError,
        },
    };

    let response = PrivateCancelResponse {
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
fn parse_private_cancel_request(payload: &str) -> Result<PrivateCancelRequestV1> {
    let request: PrivateCancelRequestV1 =
        serde_json::from_str(payload).context("failed to decode private cancel request")?;
    if request.version != 1 || request.action != "cancel" {
        anyhow::bail!("unsupported private cancel request");
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
#[serde(deny_unknown_fields)]
struct PrivateCancelRequestV1 {
    version: u8,
    action: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    reason: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    graceful: Option<bool>,
}

#[cfg(unix)]
#[derive(Debug, Deserialize, Serialize)]
struct PrivateStopResponse {
    version: u8,
    outcome: PrivateStopOutcome,
}

#[cfg(unix)]
#[derive(Debug, Deserialize, Serialize)]
struct PrivateCancelResponse {
    version: u8,
    outcome: PrivateCancelOutcome,
}

#[cfg(unix)]
#[derive(Debug, Deserialize, Serialize)]
struct PrivatePromptRequestV1 {
    version: u8,
    action: PublicPromptAction,
    prompt: String,
}

#[cfg(unix)]
async fn request_private_prompt_stream<F>(
    path: &Path,
    action: PublicPromptAction,
    prompt: &str,
    mut on_envelope: F,
) -> Result<i32>
where
    F: FnMut(&PublicPromptEnvelope) -> Result<()>,
{
    let mut stream = UnixStream::connect(path).await.with_context(|| {
        format!(
            "failed to connect to private prompt transport {}",
            path.display()
        )
    })?;
    let request = PrivatePromptRequestV1 {
        version: 1,
        action,
        prompt: prompt.to_string(),
    };
    stream
        .write_all(serde_json::to_string(&request)?.as_bytes())
        .await?;
    stream.write_all(b"\n").await?;
    stream.flush().await?;

    let mut reader = BufReader::new(stream);
    let mut line = String::new();
    let mut saw_accept = false;
    loop {
        line.clear();
        let bytes_read = reader.read_line(&mut line).await?;
        if bytes_read == 0 {
            break;
        }
        let envelope: PublicPromptEnvelope =
            serde_json::from_str(line.trim()).with_context(|| {
                format!(
                    "failed to decode private prompt transport response from {}",
                    path.display()
                )
            })?;
        if matches!(envelope, PublicPromptEnvelope::Accepted { .. }) {
            saw_accept = true;
        }
        on_envelope(&envelope)?;
        match envelope {
            PublicPromptEnvelope::Completed { turn_outcome, .. } => {
                return Ok(completed_exit_code(turn_outcome.as_str()));
            }
            PublicPromptEnvelope::Failed { message, .. } => return Err(anyhow::anyhow!(message)),
            _ => {}
        }
    }

    if saw_accept {
        anyhow::bail!("owner_unreachable: prompt stream ended before terminal envelope");
    }
    anyhow::bail!("owner_unreachable: prompt owner closed the stream before accepting the request");
}

#[cfg(unix)]
async fn handle_private_prompt_connection(
    stream: UnixStream,
    prompt_tx: PrivatePromptRequestSender,
) -> Result<()> {
    let mut reader = BufReader::new(stream);
    let mut line = String::new();
    let bytes_read = reader.read_line(&mut line).await?;
    let request = if bytes_read == 0 {
        None
    } else {
        Some(parse_private_prompt_request(line.trim())?)
    };

    let mut stream = reader.into_inner();
    let Some(request) = request else {
        let failed = failed_prompt_envelope(
            "bridge",
            "malformed_prompt_source",
            "empty private prompt request",
        );
        stream
            .write_all(serde_json::to_string(&failed)?.as_bytes())
            .await?;
        stream.write_all(b"\n").await?;
        stream.flush().await?;
        return Ok(());
    };

    let (envelope_tx, mut envelope_rx) = mpsc::unbounded_channel();
    if prompt_tx
        .send(PrivatePromptRequest {
            action: request.action,
            prompt: request.prompt,
            envelope_tx,
        })
        .is_err()
    {
        let failed = failed_prompt_envelope(
            "bridge",
            "owner_unreachable",
            "private prompt owner is no longer available",
        );
        stream
            .write_all(serde_json::to_string(&failed)?.as_bytes())
            .await?;
        stream.write_all(b"\n").await?;
        stream.flush().await?;
        return Ok(());
    }

    let mut saw_accept = false;
    let mut saw_terminal = false;
    while let Some(envelope) = envelope_rx.recv().await {
        if matches!(envelope, PublicPromptEnvelope::Accepted { .. }) {
            saw_accept = true;
        }
        stream
            .write_all(serde_json::to_string(&envelope)?.as_bytes())
            .await?;
        stream.write_all(b"\n").await?;
        stream.flush().await?;
        if matches!(
            envelope,
            PublicPromptEnvelope::Completed { .. } | PublicPromptEnvelope::Failed { .. }
        ) {
            saw_terminal = true;
            break;
        }
    }
    if saw_accept && !saw_terminal {
        let failed = failed_prompt_envelope(
            "bridge",
            "owner_unreachable",
            "private prompt owner closed after accepting the request",
        );
        stream
            .write_all(serde_json::to_string(&failed)?.as_bytes())
            .await?;
        stream.write_all(b"\n").await?;
        stream.flush().await?;
    }
    Ok(())
}

#[cfg(unix)]
fn parse_private_prompt_request(payload: &str) -> Result<PrivatePromptRequestV1> {
    let request: PrivatePromptRequestV1 =
        serde_json::from_str(payload).context("failed to decode private prompt request")?;
    if request.version != 1 {
        anyhow::bail!("unsupported private prompt request");
    }
    if request.prompt.trim().is_empty() {
        anyhow::bail!("private prompt request requires a non-empty prompt");
    }
    Ok(request)
}

async fn stream_private_prompt_request(
    runtime: &PromptSubmitRuntime,
    request: PrivatePromptRequest,
    world_scoped: bool,
) -> Result<()> {
    let manifest_snapshot = runtime
        .manifest
        .lock()
        .expect("runtime manifest mutex poisoned")
        .clone();
    if !manifest_snapshot.handle.state.is_live() {
        let _ = request.envelope_tx.send(failed_prompt_envelope(
            "runtime",
            "owner_unreachable",
            "runtime is no longer authoritative-live",
        ));
        return Ok(());
    }
    let run_id = Uuid::now_v7().to_string();
    let accepted = PublicPromptEnvelope::Accepted {
        version: 1,
        action: request.action,
        orchestration_session_id: manifest_snapshot.handle.orchestration_session_id.clone(),
        backend_id: runtime.descriptor.backend_id.clone(),
        participant_id: Some(manifest_snapshot.handle.participant_id.clone()),
        scope: scope_label(runtime.descriptor.execution_scope).to_string(),
    };
    if request.envelope_tx.send(accepted).is_err() {
        return Ok(());
    }

    let submit_result = if world_scoped {
        submit_world_prompt_turn(runtime, &run_id, &request.prompt, |event| {
            let _ = request
                .envelope_tx
                .send(event_to_public_prompt_envelope(event));
        })
        .await
    } else {
        submit_host_prompt_turn(runtime, &run_id, &request.prompt, |event| {
            let _ = request
                .envelope_tx
                .send(event_to_public_prompt_envelope(event));
        })
        .await
    };

    match submit_result {
        Ok(completion) => {
            let mut warnings = Vec::new();
            if let Some(message) = completion.warning {
                warnings.push(message.clone());
                let _ = request.envelope_tx.send(PublicPromptEnvelope::Warning {
                    version: 1,
                    message,
                });
            }
            let (session_posture, state) = prompt_completion_session_state(runtime);
            let _ = request.envelope_tx.send(PublicPromptEnvelope::Completed {
                version: 1,
                action: request.action,
                orchestration_session_id: manifest_snapshot.handle.orchestration_session_id,
                backend_id: runtime.descriptor.backend_id.clone(),
                participant_id: Some(manifest_snapshot.handle.participant_id),
                turn_outcome: turn_outcome_label(completion.exit_code).to_string(),
                session_posture,
                state,
                warnings,
            });
            request_helper_park_after_turn(runtime, request.action);
        }
        Err(err) => {
            let _ = request.envelope_tx.send(failed_prompt_envelope(
                "runtime",
                "owner_unreachable",
                err.to_string(),
            ));
            request_helper_park_after_turn(runtime, request.action);
        }
    }
    Ok(())
}

fn request_helper_park_after_turn(runtime: &PromptSubmitRuntime, action: PublicPromptAction) {
    if action != PublicPromptAction::Turn {
        return;
    }
    if let Some(park_after_turn_tx) = runtime.park_after_turn_tx.as_ref() {
        let _ = park_after_turn_tx.send(());
    }
}

fn event_to_public_prompt_envelope(event: SubmittedPromptStreamEvent) -> PublicPromptEnvelope {
    let (event_kind, data) = match event {
        SubmittedPromptStreamEvent::Agent(event) => (
            "message".to_string(),
            serde_json::to_value(event).unwrap_or_default(),
        ),
        SubmittedPromptStreamEvent::Stdout(text) => {
            ("message".to_string(), serde_json::json!({ "text": text }))
        }
        SubmittedPromptStreamEvent::Stderr(text) => {
            ("stderr".to_string(), serde_json::json!({ "text": text }))
        }
    };
    PublicPromptEnvelope::Event {
        version: 1,
        event_kind,
        data,
    }
}

fn scope_label(scope: AgentExecutionScope) -> &'static str {
    match scope {
        AgentExecutionScope::Host => "host",
        AgentExecutionScope::World => "world",
    }
}

#[cfg(target_os = "linux")]
fn warning_for_exit_code(exit_code: i32) -> Option<String> {
    (exit_code != 0).then(|| format!("Command failed with status: {exit_code}"))
}

fn turn_outcome_label(exit_code: i32) -> &'static str {
    match exit_code {
        0 => "success",
        130 => "cancelled",
        _ => "nonzero_exit",
    }
}

#[allow(dead_code)]
pub(crate) fn world_task_terminal_state_from_exit_code(
    exit_code: i32,
) -> super::dispatch_contract::WorldTaskTerminalStateV1 {
    match exit_code {
        0 => super::dispatch_contract::WorldTaskTerminalStateV1::Completed,
        130 => super::dispatch_contract::WorldTaskTerminalStateV1::Cancelled,
        _ => super::dispatch_contract::WorldTaskTerminalStateV1::Failed,
    }
}

#[cfg(unix)]
fn completed_exit_code(turn_outcome: &str) -> i32 {
    match turn_outcome {
        "success" => 0,
        "cancelled" => 130,
        _ => 1,
    }
}

fn prompt_completion_session_state(
    runtime: &PromptSubmitRuntime,
) -> (PublicSessionPosture, String) {
    let session = runtime
        .orchestration_session
        .lock()
        .expect("orchestration session mutex poisoned")
        .clone();
    let manifest = runtime
        .manifest
        .lock()
        .expect("runtime manifest mutex poisoned")
        .clone();

    let posture =
        if !manifest.handle.state.is_live() || session.state != OrchestrationSessionState::Active {
            PublicSessionPosture::Terminal
        } else {
            PublicSessionPosture::Active
        };
    (
        posture,
        runtime_state_label(&manifest, &session).to_string(),
    )
}

fn runtime_state_label(
    manifest: &AgentRuntimeSessionManifest,
    session: &OrchestrationSessionRecord,
) -> &'static str {
    if manifest.has_cancelled_terminal_truth() {
        return manifest.reviewable_terminal_state_label();
    }
    if session.has_cancelled_terminal_truth() {
        return session.reviewable_terminal_state_label();
    }

    match manifest.handle.state {
        AgentRuntimeSessionState::Allocating
        | AgentRuntimeSessionState::Ready
        | AgentRuntimeSessionState::Running
        | AgentRuntimeSessionState::Restarting
        | AgentRuntimeSessionState::Stopping => "active",
        AgentRuntimeSessionState::Stopped => "stopped",
        AgentRuntimeSessionState::Invalidated => "invalidated",
        AgentRuntimeSessionState::Failed => "failed",
    }
}

fn failed_prompt_envelope(
    stage: impl Into<String>,
    error_code: impl Into<String>,
    message: impl Into<String>,
) -> PublicPromptEnvelope {
    PublicPromptEnvelope::Failed {
        version: 1,
        terminal: true,
        stage: stage.into(),
        error_code: error_code.into(),
        message: message.into(),
    }
}

#[derive(Debug)]
pub(crate) struct PublicPromptRenderedExit {
    exit_code: i32,
}

impl std::fmt::Display for PublicPromptRenderedExit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "public prompt command already rendered terminal output")
    }
}

impl std::error::Error for PublicPromptRenderedExit {}

pub(crate) fn public_prompt_rendered_exit_code(err: &anyhow::Error) -> Option<i32> {
    err.downcast_ref::<PublicPromptRenderedExit>()
        .map(|exit| exit.exit_code)
}

fn resolve_public_start_prompt_target(
    store: &AgentRuntimeStateStore,
    orchestration_session_id: &str,
    backend_id: &str,
) -> Result<String> {
    let Some(record) = store.load_orchestration_session(orchestration_session_id)? else {
        anyhow::bail!(
            "unknown_session: no orchestration session found for '{}'",
            orchestration_session_id
        );
    };
    if record.state != OrchestrationSessionState::Active {
        anyhow::bail!(
            "runtime_start_failed: orchestration session {} is not active",
            orchestration_session_id
        );
    }
    let active_participant_id = record.active_participant_id().ok_or_else(|| {
        anyhow::anyhow!(
            "runtime_start_failed: orchestration session {} is missing an active participant",
            orchestration_session_id
        )
    })?;
    let active_participant = store
        .load_participant(active_participant_id)?
        .ok_or_else(|| {
            anyhow::anyhow!(
                "runtime_start_failed: orchestration session {} lost its active participant {}",
                orchestration_session_id,
                active_participant_id
            )
        })?;
    if active_participant.handle.backend_id != backend_id {
        anyhow::bail!(
            "runtime_start_failed: orchestration session {} active backend {} did not match requested backend {}",
            orchestration_session_id,
            active_participant.handle.backend_id,
            backend_id
        );
    }
    Ok(active_participant.handle.participant_id.clone())
}

#[cfg(unix)]
async fn wait_for_private_prompt_transport(path: &Path) -> Result<()> {
    let started_at = std::time::Instant::now();
    loop {
        match tokio::fs::metadata(path).await {
            Ok(_) => return Ok(()),
            Err(err) if err.kind() == io::ErrorKind::NotFound => {
                if started_at.elapsed() >= PRIVATE_PROMPT_READY_TIMEOUT {
                    anyhow::bail!(
                        "stream_bridge_failed: timed out waiting for private prompt transport {}",
                        path.display()
                    );
                }
                tokio::time::sleep(PRIVATE_PROMPT_READY_POLL_INTERVAL).await;
            }
            Err(err) => {
                return Err(err).with_context(|| {
                    format!(
                        "stream_bridge_failed: failed to inspect private prompt transport {}",
                        path.display()
                    )
                });
            }
        }
    }
}

fn warning_for_exit_status(status: &ExitStatus) -> Option<String> {
    if status.success() {
        return None;
    }

    #[cfg(unix)]
    {
        use std::os::unix::process::ExitStatusExt;
        if let Some(sig) = status.signal() {
            return Some(format!("Command terminated by signal {sig}"));
        }
    }

    Some(format!(
        "Command failed with status: {}",
        status.code().unwrap_or(-1)
    ))
}

fn extract_session_handle_id(data: Option<&serde_json::Value>) -> Option<&str> {
    let value = data?;
    if value.get("schema").and_then(serde_json::Value::as_str)
        == Some(super::SESSION_HANDLE_SCHEMA_V1)
    {
        return value
            .get("session")
            .and_then(serde_json::Value::as_object)
            .and_then(|session| session.get("id"))
            .and_then(serde_json::Value::as_str)
            .filter(|id| !id.trim().is_empty());
    }

    value
        .get("type")
        .and_then(serde_json::Value::as_str)
        .filter(|event_type| matches!(*event_type, "thread.started" | "turn.started"))?;
    value
        .get("thread_id")
        .and_then(serde_json::Value::as_str)
        .filter(|id| !id.trim().is_empty())
}

fn translate_prompt_wrapper_event(
    manifest: &AgentRuntimeSessionManifest,
    orchestration_session: &OrchestrationSessionRecord,
    run_id: &str,
    wrapper_event: agent_api::AgentWrapperEvent,
) -> AgentEvent {
    let mut event = match wrapper_event.kind {
        agent_api::AgentWrapperEventKind::Status => AgentEvent::message(
            manifest.handle.agent_id.clone(),
            manifest.handle.orchestration_session_id.clone(),
            run_id.to_string(),
            MessageEventKind::Status,
            wrapper_event
                .message
                .clone()
                .unwrap_or_else(|| "agent runtime status".to_string()),
        ),
        agent_api::AgentWrapperEventKind::TextOutput => AgentEvent::message(
            manifest.handle.agent_id.clone(),
            manifest.handle.orchestration_session_id.clone(),
            run_id.to_string(),
            MessageEventKind::TaskProgress,
            wrapper_event
                .text
                .clone()
                .unwrap_or_else(|| "agent runtime output".to_string()),
        ),
        agent_api::AgentWrapperEventKind::ToolCall
        | agent_api::AgentWrapperEventKind::ToolResult => AgentEvent::message(
            manifest.handle.agent_id.clone(),
            manifest.handle.orchestration_session_id.clone(),
            run_id.to_string(),
            MessageEventKind::TaskProgress,
            wrapper_event
                .message
                .clone()
                .unwrap_or_else(|| "agent runtime tool activity".to_string()),
        ),
        agent_api::AgentWrapperEventKind::Error => AgentEvent::alert(
            manifest.handle.agent_id.clone(),
            manifest.handle.orchestration_session_id.clone(),
            run_id.to_string(),
            "agent_wrapper_error",
            wrapper_event
                .message
                .clone()
                .unwrap_or_else(|| "agent runtime error".to_string()),
        ),
        agent_api::AgentWrapperEventKind::Unknown => AgentEvent::message(
            manifest.handle.agent_id.clone(),
            manifest.handle.orchestration_session_id.clone(),
            run_id.to_string(),
            MessageEventKind::TaskProgress,
            "agent runtime emitted an unknown event".to_string(),
        ),
    };

    event.role = Some(manifest.handle.role.clone());
    event.backend_id = Some(manifest.handle.backend_id.clone());
    event.set_pure_agent_telemetry_identity(manifest.handle.agent_id.clone());
    event.set_channel(wrapper_event.channel.clone());
    event.world_id = orchestration_session.world_id.clone();
    event.world_generation = orchestration_session.world_generation;
    event.participant_id = Some(manifest.handle.participant_id.clone());
    event.parent_participant_id = manifest.handle.parent_participant_id.clone();
    event.resumed_from_participant_id = manifest.handle.resumed_from_participant_id.clone();

    if let Some(data) = wrapper_event.data {
        if let Some(obj) = event.data.as_object_mut() {
            obj.insert("uaa_event".to_string(), data);
            obj.insert(
                "protocol".to_string(),
                serde_json::json!(PURE_AGENT_PROTOCOL),
            );
        }
    }

    event
}

#[cfg(unix)]
struct PublicPromptRenderer {
    json: bool,
}

#[cfg(unix)]
impl PublicPromptRenderer {
    fn new(json: bool) -> Self {
        Self { json }
    }

    fn render(&mut self, envelope: &PublicPromptEnvelope) -> Result<()> {
        if self.json {
            let stdout = io::stdout();
            let mut lock = stdout.lock();
            writeln!(lock, "{}", serde_json::to_string(envelope)?)
                .context("failed to render prompt envelope")?;
            let _ = lock.flush();
            return Ok(());
        }

        match envelope {
            PublicPromptEnvelope::Accepted { .. } => {}
            PublicPromptEnvelope::Completed {
                action,
                orchestration_session_id,
                backend_id,
                participant_id,
                turn_outcome,
                session_posture,
                ..
            } => {
                let stdout = io::stdout();
                let mut lock = stdout.lock();
                let _ = writeln!(
                    lock,
                    "action={} orchestration_session_id={} backend_id={} participant_id={} turn_outcome={} session_posture={}",
                    action.as_str(),
                    orchestration_session_id,
                    backend_id,
                    participant_id.as_deref().unwrap_or("-"),
                    turn_outcome,
                    session_posture.as_str()
                );
                let _ = lock.flush();
            }
            PublicPromptEnvelope::Warning { message, .. }
            | PublicPromptEnvelope::Failed { message, .. } => {
                let stderr = io::stderr();
                let mut lock = stderr.lock();
                let _ = writeln!(lock, "{message}");
                let _ = lock.flush();
            }
            PublicPromptEnvelope::Event {
                event_kind, data, ..
            } => {
                if event_kind == "stderr" {
                    let stderr = io::stderr();
                    let mut lock = stderr.lock();
                    if let Ok(event) = serde_json::from_value::<AgentEvent>(data.clone()) {
                        let _ = writeln!(lock, "{}", format_event_line(&event));
                    } else {
                        let fallback = prompt_event_text(data);
                        let fallback = if fallback.is_empty() {
                            structured_prompt_event_fallback_text(data).unwrap_or(fallback)
                        } else {
                            fallback
                        };
                        let _ = lock.write_all(fallback.as_bytes());
                    }
                    let _ = lock.flush();
                } else if let Ok(event) = serde_json::from_value::<AgentEvent>(data.clone()) {
                    let stdout = io::stdout();
                    let mut lock = stdout.lock();
                    let _ = writeln!(lock, "{}", format_event_line(&event));
                    let _ = lock.flush();
                } else {
                    let fallback = prompt_event_text(data);
                    let fallback = if fallback.is_empty() {
                        structured_prompt_event_fallback_text(data).unwrap_or(fallback)
                    } else {
                        fallback
                    };
                    let stdout = io::stdout();
                    let mut lock = stdout.lock();
                    let _ = lock.write_all(fallback.as_bytes());
                    let _ = lock.flush();
                }
            }
        }
        Ok(())
    }
}

#[cfg(unix)]
fn prompt_event_text(data: &serde_json::Value) -> String {
    fn direct_prompt_event_text(data: &serde_json::Value) -> Option<String> {
        if let Some(text) = data.get("text").and_then(serde_json::Value::as_str) {
            return Some(text.to_string());
        }
        if let Some(message) = data.get("message").and_then(serde_json::Value::as_str) {
            return Some(message.to_string());
        }
        data.get("chunk")
            .and_then(serde_json::Value::as_str)
            .map(|chunk| {
                let stream = data
                    .get("stream")
                    .and_then(serde_json::Value::as_str)
                    .unwrap_or("stdout");
                format!("{stream}: {chunk}")
            })
    }

    fn escape_nested_prompt_event_text(text: &str) -> String {
        text.replace('\r', "\\r").replace('\n', "\\n")
    }

    if let Some(text) = direct_prompt_event_text(data) {
        return text;
    }

    let Some(text) = data.get("data").and_then(direct_prompt_event_text) else {
        return String::new();
    };
    let agent = data
        .get("agent_id")
        .and_then(serde_json::Value::as_str)
        .filter(|agent| !agent.trim().is_empty())
        .unwrap_or("agent");
    format!("[{agent}] {}\n", escape_nested_prompt_event_text(&text))
}

#[cfg(unix)]
fn structured_prompt_event_fallback_text(data: &serde_json::Value) -> Option<String> {
    const STRUCTURED_PROMPT_EVENT_FALLBACK_KEY_LIMIT: usize = 3;
    const STRUCTURED_PROMPT_EVENT_FALLBACK_TEXT_LIMIT: usize = 120;

    fn direct_structured_prompt_event_text(data: &serde_json::Value) -> Option<String> {
        if let Some(message) = data.get("message").and_then(serde_json::Value::as_str) {
            return Some(message.to_string());
        }
        data.get("chunk")
            .and_then(serde_json::Value::as_str)
            .map(|chunk| {
                let stream = data
                    .get("stream")
                    .and_then(serde_json::Value::as_str)
                    .unwrap_or("stdout");
                format!("{stream}: {chunk}")
            })
    }

    fn escape_structured_prompt_event_text(text: &str) -> String {
        text.replace('\r', "\\r").replace('\n', "\\n")
    }

    fn compact_structured_prompt_scalar_text(text: &str) -> Option<String> {
        let escaped = escape_structured_prompt_event_text(text);
        let trimmed = escaped.trim();
        if trimmed.is_empty() {
            return None;
        }

        let bounded = trimmed
            .chars()
            .take(STRUCTURED_PROMPT_EVENT_FALLBACK_TEXT_LIMIT + 1)
            .collect::<String>();
        let bounded_len = bounded.chars().count();
        if bounded_len > STRUCTURED_PROMPT_EVENT_FALLBACK_TEXT_LIMIT {
            let prefix = bounded
                .chars()
                .take(STRUCTURED_PROMPT_EVENT_FALLBACK_TEXT_LIMIT)
                .collect::<String>();
            return Some(format!("{prefix}..."));
        }

        Some(trimmed.to_string())
    }

    fn summarize_structured_prompt_payload(payload: &serde_json::Value) -> Option<String> {
        match payload {
            serde_json::Value::Null => None,
            serde_json::Value::Bool(value) => Some(value.to_string()),
            serde_json::Value::Number(value) => Some(value.to_string()),
            serde_json::Value::String(value) => compact_structured_prompt_scalar_text(value),
            serde_json::Value::Array(values) => Some(format!("items={}", values.len())),
            serde_json::Value::Object(fields) => {
                if fields.is_empty() {
                    return Some("fields=none".to_string());
                }

                let keys = fields
                    .keys()
                    .take(STRUCTURED_PROMPT_EVENT_FALLBACK_KEY_LIMIT)
                    .map(String::as_str)
                    .collect::<Vec<_>>();
                let extra = fields.len().saturating_sub(keys.len());
                if extra > 0 {
                    Some(format!("fields={} (+{extra} more)", keys.join(", ")))
                } else {
                    Some(format!("fields={}", keys.join(", ")))
                }
            }
        }
    }

    let payload = data.get("data").unwrap_or(data);
    let kind = data
        .get("kind")
        .and_then(serde_json::Value::as_str)
        .filter(|kind| !kind.trim().is_empty());
    let rendered = if let Some(text) = direct_structured_prompt_event_text(payload) {
        escape_structured_prompt_event_text(&text)
    } else if payload.is_null() {
        kind.unwrap_or("event").to_string()
    } else if let Some(summary) = summarize_structured_prompt_payload(payload) {
        match kind {
            Some(kind) => format!("{kind}: {summary}"),
            None => summary,
        }
    } else {
        kind.unwrap_or("structured event").to_string()
    };

    if rendered.trim().is_empty() {
        return None;
    }

    let agent = data
        .get("agent_id")
        .and_then(serde_json::Value::as_str)
        .filter(|agent| !agent.trim().is_empty());
    Some(match agent {
        Some(agent) => format!("[{agent}] {rendered}\n"),
        None => format!("{rendered}\n"),
    })
}

#[cfg(test)]
mod tests {
    use super::{
        apply_runtime_cancel_closeout, apply_runtime_stop_closeout,
        prompt_completion_session_state, prompt_event_text,
        reconcile_hidden_owner_helper_start_timeout, runtime_is_terminal,
        validate_public_prompt_command_request, HiddenOwnerHelperLaunchPlan,
        HiddenOwnerHelperParticipantPlan, HiddenOwnerHelperSessionPlan,
        HiddenOwnerHelperStartTimeoutReconciliation, HiddenOwnerHelperStartupPromptPlan,
        LoadedPublicPrompt, OwnerHelperMode, PrivateCancelOutcome, PrivateStopOutcome,
        PromptSubmitRuntime, PublicPromptAction, PublicPromptCommandRequest, PublicSessionPosture,
        ResolvedRuntimeBackendKind, ResolvedRuntimeDescriptor, PURE_AGENT_PROTOCOL,
    };
    #[cfg(unix)]
    use super::{
        handle_private_prompt_connection, private_prompt_request_channel,
        structured_prompt_event_fallback_text, PublicPromptEnvelope, PublicPromptRenderer,
    };
    use crate::execution::agent_runtime::orchestration_session::HostAttachContract;
    use crate::execution::agent_runtime::{
        mapping::AgentRuntimeBackendKind,
        orchestration_session::{
            OrchestrationSessionPosture, OrchestrationSessionRecord, OrchestrationSessionState,
            StartupPromptStreamState,
        },
        session::{
            AgentRuntimeParticipantRecord, AgentRuntimeParticipantWorldBinding,
            AgentRuntimeReplacementParticipantInit, AgentRuntimeSessionState,
        },
        validator::RuntimeSelectionDescriptor,
        AgentRuntimeStateStore, OrchestrationObligationAttachState, OrchestrationObligationKind,
        OrchestrationObligationRecord, ORCHESTRATOR_ROLE,
    };
    use crate::execution::config_model::AgentExecutionScope;
    #[cfg(unix)]
    use std::fs::File;
    #[cfg(unix)]
    use std::io::Read;
    #[cfg(unix)]
    use std::io::Write;
    #[cfg(unix)]
    use std::os::fd::FromRawFd;
    use std::path::PathBuf;
    use std::sync::atomic::AtomicBool;
    use std::sync::{Arc, Mutex};
    use tempfile::TempDir;
    #[cfg(unix)]
    use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
    #[cfg(unix)]
    use tokio::net::UnixStream;

    fn with_store(test: impl FnOnce(&AgentRuntimeStateStore)) {
        let temp = TempDir::new().expect("tempdir");
        std::env::set_var("SUBSTRATE_HOME", temp.path());
        let store = AgentRuntimeStateStore::new().expect("state store");
        test(&store);
        std::env::remove_var("SUBSTRATE_HOME");
    }

    fn test_plan(
        orchestration_session_id: &str,
        participant_id: &str,
    ) -> HiddenOwnerHelperLaunchPlan {
        HiddenOwnerHelperLaunchPlan {
            mode: OwnerHelperMode::Start,
            descriptor: ResolvedRuntimeDescriptor {
                agent_id: "codex".to_string(),
                backend_id: "cli:codex".to_string(),
                backend_kind: ResolvedRuntimeBackendKind::Codex,
                protocol: PURE_AGENT_PROTOCOL.to_string(),
                execution_scope: AgentExecutionScope::Host,
                binary_path: "/usr/bin/codex".to_string(),
            },
            session: HiddenOwnerHelperSessionPlan {
                orchestration_session_id: orchestration_session_id.to_string(),
                shell_trace_session_id: "trace_session".to_string(),
                workspace_root: "/workspace".to_string(),
                world_id: None,
                world_generation: None,
            },
            participant: HiddenOwnerHelperParticipantPlan {
                participant_id: participant_id.to_string(),
                lease_token: format!("lease_{participant_id}"),
                run_id: "run_start".to_string(),
                resumed_from_participant_id: None,
                internal_uaa_session_id: None,
            },
            host_attach_contract: None,
            startup_prompt: Some(HiddenOwnerHelperStartupPromptPlan {
                prompt_text: "hello".to_string(),
                stream_path: PathBuf::from("/tmp/startup.sock"),
            }),
            source_orchestration_session_id: None,
        }
    }

    fn resumed_turn_test_plan(
        orchestration_session_id: &str,
        participant_id: &str,
        resumed_from_participant_id: &str,
        internal_uaa_session_id: &str,
    ) -> HiddenOwnerHelperLaunchPlan {
        HiddenOwnerHelperLaunchPlan {
            mode: OwnerHelperMode::ResumeOneTurn,
            descriptor: ResolvedRuntimeDescriptor {
                agent_id: "codex".to_string(),
                backend_id: "cli:codex".to_string(),
                backend_kind: ResolvedRuntimeBackendKind::Codex,
                protocol: PURE_AGENT_PROTOCOL.to_string(),
                execution_scope: AgentExecutionScope::Host,
                binary_path: "/usr/bin/codex".to_string(),
            },
            session: HiddenOwnerHelperSessionPlan {
                orchestration_session_id: orchestration_session_id.to_string(),
                shell_trace_session_id: "trace_session".to_string(),
                workspace_root: "/workspace".to_string(),
                world_id: None,
                world_generation: None,
            },
            participant: HiddenOwnerHelperParticipantPlan {
                participant_id: participant_id.to_string(),
                lease_token: format!("lease_{participant_id}"),
                run_id: "run_resume_one_turn".to_string(),
                resumed_from_participant_id: Some(resumed_from_participant_id.to_string()),
                internal_uaa_session_id: Some(internal_uaa_session_id.to_string()),
            },
            host_attach_contract: None,
            startup_prompt: Some(HiddenOwnerHelperStartupPromptPlan {
                prompt_text: "hello".to_string(),
                stream_path: PathBuf::from("/tmp/startup.sock"),
            }),
            source_orchestration_session_id: None,
        }
    }

    fn attach_test_plan(
        orchestration_session_id: &str,
        participant_id: &str,
        internal_uaa_session_id: &str,
    ) -> HiddenOwnerHelperLaunchPlan {
        HiddenOwnerHelperLaunchPlan {
            mode: OwnerHelperMode::Attach,
            descriptor: ResolvedRuntimeDescriptor {
                agent_id: "codex".to_string(),
                backend_id: "cli:codex".to_string(),
                backend_kind: ResolvedRuntimeBackendKind::Codex,
                protocol: PURE_AGENT_PROTOCOL.to_string(),
                execution_scope: AgentExecutionScope::Host,
                binary_path: "/usr/bin/codex".to_string(),
            },
            session: HiddenOwnerHelperSessionPlan {
                orchestration_session_id: orchestration_session_id.to_string(),
                shell_trace_session_id: "trace_session".to_string(),
                workspace_root: "/workspace".to_string(),
                world_id: None,
                world_generation: None,
            },
            participant: HiddenOwnerHelperParticipantPlan {
                participant_id: participant_id.to_string(),
                lease_token: format!("lease_{participant_id}"),
                run_id: "run_attach".to_string(),
                resumed_from_participant_id: Some(participant_id.to_string()),
                internal_uaa_session_id: Some(internal_uaa_session_id.to_string()),
            },
            host_attach_contract: None,
            startup_prompt: None,
            source_orchestration_session_id: None,
        }
    }

    fn prompt_submit_runtime_for_test(
        store: &AgentRuntimeStateStore,
        orchestration_session: OrchestrationSessionRecord,
        manifest: AgentRuntimeParticipantRecord,
    ) -> PromptSubmitRuntime {
        PromptSubmitRuntime {
            descriptor: RuntimeSelectionDescriptor {
                agent_id: "codex".to_string(),
                backend_id: "cli:codex".to_string(),
                backend_kind: AgentRuntimeBackendKind::Codex,
                protocol: PURE_AGENT_PROTOCOL.to_string(),
                execution_scope: AgentExecutionScope::Host,
                binary_path: PathBuf::from("/usr/bin/codex"),
            },
            orchestration_session: Arc::new(Mutex::new(orchestration_session)),
            manifest: Arc::new(Mutex::new(manifest)),
            store: store.clone(),
            uaa_session_handle_id: "uaa_session".to_string(),
            park_after_turn_tx: None,
            host_toolbox_surface_authoritative: Arc::new(AtomicBool::new(false)),
        }
    }

    #[test]
    fn prompt_submit_continuity_prefers_persisted_session_contract() {
        with_store(|store| {
            let descriptor = RuntimeSelectionDescriptor {
                agent_id: "codex".to_string(),
                backend_id: "cli:codex".to_string(),
                backend_kind: AgentRuntimeBackendKind::Codex,
                protocol: PURE_AGENT_PROTOCOL.to_string(),
                execution_scope: AgentExecutionScope::Host,
                binary_path: PathBuf::from("/usr/bin/codex"),
            };
            let mut manifest = AgentRuntimeParticipantRecord::new_orchestrator_participant(
                &descriptor,
                "sess_prompt_continuity_contract".to_string(),
                "ash_prompt_continuity_contract".to_string(),
                "lease_prompt_continuity_contract".to_string(),
            )
            .expect("orchestrator participant");
            manifest.transition_state(AgentRuntimeSessionState::Ready);
            manifest.set_uaa_session_id("uaa_manifest_fresh".to_string());

            let mut host_attach_contract =
                HostAttachContract::from_manifest_for_test(&manifest).expect("attach contract");
            host_attach_contract.continuity_uaa_session_id = Some("uaa_contract_fresh".to_string());

            let orchestration_session = OrchestrationSessionRecord::new(
                "sess_prompt_continuity_contract".to_string(),
                "trace_session".to_string(),
                "/workspace".to_string(),
                &manifest,
                Some(host_attach_contract),
            );
            let mut runtime =
                prompt_submit_runtime_for_test(store, orchestration_session, manifest);
            runtime.uaa_session_handle_id = "uaa_runtime_stale".to_string();

            assert_eq!(
                super::prompt_submit_continuity_session_id(&runtime),
                "uaa_contract_fresh"
            );
        });
    }

    #[test]
    fn prompt_submit_continuity_falls_back_to_manifest_when_session_contract_missing() {
        with_store(|store| {
            let descriptor = RuntimeSelectionDescriptor {
                agent_id: "codex".to_string(),
                backend_id: "cli:codex".to_string(),
                backend_kind: AgentRuntimeBackendKind::Codex,
                protocol: PURE_AGENT_PROTOCOL.to_string(),
                execution_scope: AgentExecutionScope::Host,
                binary_path: PathBuf::from("/usr/bin/codex"),
            };
            let mut manifest = AgentRuntimeParticipantRecord::new_orchestrator_participant(
                &descriptor,
                "sess_prompt_continuity_manifest".to_string(),
                "ash_prompt_continuity_manifest".to_string(),
                "lease_prompt_continuity_manifest".to_string(),
            )
            .expect("orchestrator participant");
            manifest.transition_state(AgentRuntimeSessionState::Ready);
            manifest.set_uaa_session_id("uaa_manifest_fresh".to_string());

            let orchestration_session = OrchestrationSessionRecord::new(
                "sess_prompt_continuity_manifest".to_string(),
                "trace_session".to_string(),
                "/workspace".to_string(),
                &manifest,
                None,
            );
            let mut runtime =
                prompt_submit_runtime_for_test(store, orchestration_session, manifest);
            runtime.uaa_session_handle_id = "uaa_runtime_stale".to_string();

            assert_eq!(
                super::prompt_submit_continuity_session_id(&runtime),
                "uaa_manifest_fresh"
            );
        });
    }

    #[test]
    fn owner_helper_modes_remain_internal_and_exact() {
        assert_eq!(OwnerHelperMode::Start.as_str(), "start");
        assert_eq!(OwnerHelperMode::Attach.as_str(), "attach");
        assert_eq!(OwnerHelperMode::ResumeOneTurn.as_str(), "resume_one_turn");
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

    #[test]
    fn private_cancel_outcomes_are_exact() {
        let outcomes = [
            PrivateCancelOutcome::Accepted,
            PrivateCancelOutcome::AlreadyTerminal,
            PrivateCancelOutcome::OwnerUnreachable,
            PrivateCancelOutcome::ProtocolError,
        ];
        assert_eq!(outcomes.len(), 4);
    }

    #[test]
    fn runtime_terminal_predicate_requires_explicit_terminal_observation() {
        let descriptor = RuntimeSelectionDescriptor {
            agent_id: "codex-world".to_string(),
            backend_id: "cli:codex-world".to_string(),
            backend_kind: AgentRuntimeBackendKind::Codex,
            protocol: PURE_AGENT_PROTOCOL.to_string(),
            execution_scope: AgentExecutionScope::Host,
            binary_path: PathBuf::from("/usr/bin/codex"),
        };
        let mut manifest = AgentRuntimeParticipantRecord::new_orchestrator_participant(
            &descriptor,
            "sess_transport_interruption".to_string(),
            "ash_transport_interruption".to_string(),
            "lease_transport_interruption".to_string(),
        )
        .expect("runtime participant");
        manifest.transition_state(AgentRuntimeSessionState::Invalidated);
        let manifest = Arc::new(Mutex::new(manifest));

        assert!(
            !runtime_is_terminal(&manifest),
            "diagnostic invalidation without exact terminal observation must not yield AlreadyTerminal"
        );

        manifest
            .lock()
            .expect("runtime manifest mutex poisoned")
            .mark_terminal_state("exact terminal Exit observed");
        assert!(runtime_is_terminal(&manifest));
    }

    #[test]
    fn prompt_event_text_keeps_top_level_text_passthrough() {
        let text = prompt_event_text(&serde_json::json!({
            "text": "stdout chunk\n",
        }));

        assert_eq!(text, "stdout chunk\n");
    }

    #[test]
    fn prompt_event_text_renders_nested_structured_agent_messages() {
        let text = prompt_event_text(&serde_json::json!({
            "agent_id": "codex",
            "data": {
                "message": "startup prompt success"
            }
        }));

        assert_eq!(text, "[codex] startup prompt success\n");
    }

    #[test]
    fn prompt_event_text_escapes_nested_structured_agent_messages() {
        let text = prompt_event_text(&serde_json::json!({
            "agent_id": "codex",
            "data": {
                "message": "startup\nprompt\rsuccess"
            }
        }));

        assert_eq!(text, "[codex] startup\\nprompt\\rsuccess\n");
    }

    #[cfg(unix)]
    #[test]
    fn structured_prompt_event_fallback_renders_nested_payload_without_message() {
        let text = structured_prompt_event_fallback_text(&serde_json::json!({
            "agent_id": "codex",
            "kind": "task_progress",
            "data": {
                "protocol": "substrate.agent.session",
                "uaa_event": {
                    "status": "queued"
                }
            }
        }))
        .expect("structured fallback text");

        assert_eq!(text, "[codex] task_progress: fields=protocol, uaa_event\n");
    }

    #[cfg(unix)]
    #[test]
    fn structured_prompt_event_fallback_uses_kind_when_payload_is_null() {
        let text = structured_prompt_event_fallback_text(&serde_json::json!({
            "agent_id": "codex",
            "kind": "status",
            "data": null
        }))
        .expect("structured fallback text");

        assert_eq!(text, "[codex] status\n");
    }

    #[cfg(unix)]
    fn capture_stdout_once(test: impl FnOnce()) -> String {
        assert_eq!(
            std::io::stdout().flush().map(|_| 0).unwrap_or(-1),
            0,
            "flush stdout before capture"
        );

        let mut pipe_fds = [0; 2];
        assert_eq!(
            unsafe { libc::pipe(pipe_fds.as_mut_ptr()) },
            0,
            "create stdout capture pipe"
        );
        let read_fd = pipe_fds[0];
        let write_fd = pipe_fds[1];
        let saved_stdout = unsafe { libc::dup(libc::STDOUT_FILENO) };
        assert!(saved_stdout >= 0, "dup stdout");
        assert_eq!(
            unsafe { libc::dup2(write_fd, libc::STDOUT_FILENO) },
            libc::STDOUT_FILENO,
            "redirect stdout to capture pipe"
        );

        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(test));

        assert_eq!(
            std::io::stdout().flush().map(|_| 0).unwrap_or(-1),
            0,
            "flush stdout after capture"
        );
        assert_eq!(
            unsafe { libc::dup2(saved_stdout, libc::STDOUT_FILENO) },
            libc::STDOUT_FILENO,
            "restore stdout after capture"
        );
        unsafe {
            libc::close(saved_stdout);
            libc::close(write_fd);
        }

        let mut output = String::new();
        unsafe { File::from_raw_fd(read_fd) }
            .read_to_string(&mut output)
            .expect("read captured stdout");

        if let Err(payload) = result {
            std::panic::resume_unwind(payload);
        }

        output
    }

    #[cfg(unix)]
    fn capture_stderr_once(test: impl FnOnce()) -> String {
        assert_eq!(
            std::io::stderr().flush().map(|_| 0).unwrap_or(-1),
            0,
            "flush stderr before capture"
        );

        let mut pipe_fds = [0; 2];
        assert_eq!(
            unsafe { libc::pipe(pipe_fds.as_mut_ptr()) },
            0,
            "create stderr capture pipe"
        );
        let read_fd = pipe_fds[0];
        let write_fd = pipe_fds[1];
        let saved_stderr = unsafe { libc::dup(libc::STDERR_FILENO) };
        assert!(saved_stderr >= 0, "dup stderr");
        assert_eq!(
            unsafe { libc::dup2(write_fd, libc::STDERR_FILENO) },
            libc::STDERR_FILENO,
            "redirect stderr to capture pipe"
        );

        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(test));

        assert_eq!(
            std::io::stderr().flush().map(|_| 0).unwrap_or(-1),
            0,
            "flush stderr after capture"
        );
        assert_eq!(
            unsafe { libc::dup2(saved_stderr, libc::STDERR_FILENO) },
            libc::STDERR_FILENO,
            "restore stderr after capture"
        );
        unsafe {
            libc::close(saved_stderr);
            libc::close(write_fd);
        }

        let mut output = String::new();
        unsafe { File::from_raw_fd(read_fd) }
            .read_to_string(&mut output)
            .expect("read captured stderr");

        if let Err(payload) = result {
            std::panic::resume_unwind(payload);
        }

        output
    }

    #[cfg(unix)]
    #[test]
    #[serial_test::serial]
    fn public_prompt_renderer_renders_bounded_structured_fallback_when_decode_fails() {
        let envelope = PublicPromptEnvelope::Event {
            version: 1,
            event_kind: "stdout".to_string(),
            data: serde_json::json!({
                "agent_id": "codex",
                "kind": "task_progress",
                "data": {
                    "alpha": "one",
                    "beta": "two",
                    "gamma": "three",
                    "delta": {
                        "status": "queued"
                    }
                }
            }),
        };

        let output = capture_stdout_once(|| {
            let mut renderer = PublicPromptRenderer::new(false);
            renderer
                .render(&envelope)
                .expect("render structured fallback");
        });
        let rendered_line = output
            .lines()
            .find(|line| line.starts_with("[codex] task_progress:"))
            .map(|line| format!("{line}\n"))
            .expect("captured stdout should contain bounded structured fallback line");

        assert_eq!(
            rendered_line,
            "[codex] task_progress: fields=alpha, beta, gamma (+1 more)\n"
        );
        assert!(
            !rendered_line.contains('{') && !rendered_line.contains("queued"),
            "fallback must stay bounded and avoid dumping raw nested payloads: {rendered_line}"
        );
    }

    #[cfg(unix)]
    #[test]
    #[serial_test::serial]
    fn public_prompt_renderer_renders_bounded_structured_stderr_fallback_when_decode_fails() {
        let envelope = PublicPromptEnvelope::Event {
            version: 1,
            event_kind: "stderr".to_string(),
            data: serde_json::json!({
                "agent_id": "codex",
                "kind": "task_progress",
                "data": {
                    "alpha": "one",
                    "beta": "two",
                    "gamma": "three",
                    "delta": {
                        "status": "queued"
                    }
                }
            }),
        };

        let output = capture_stderr_once(|| {
            let mut renderer = PublicPromptRenderer::new(false);
            renderer
                .render(&envelope)
                .expect("render structured stderr fallback");
        });
        let rendered_line = output
            .lines()
            .find(|line| line.starts_with("[codex] task_progress:"))
            .map(|line| format!("{line}\n"))
            .expect("captured stderr should contain bounded structured fallback line");

        assert_eq!(
            rendered_line,
            "[codex] task_progress: fields=alpha, beta, gamma (+1 more)\n"
        );
        assert!(
            !rendered_line.contains('{') && !rendered_line.contains("queued"),
            "fallback must stay bounded and avoid dumping raw nested payloads: {rendered_line}"
        );
    }

    #[test]
    fn stop_closeout_helper_converges_on_stopped_terminal_snapshots() {
        let descriptor = RuntimeSelectionDescriptor {
            agent_id: "codex".to_string(),
            backend_id: "cli:codex".to_string(),
            backend_kind: AgentRuntimeBackendKind::Codex,
            protocol: PURE_AGENT_PROTOCOL.to_string(),
            execution_scope: AgentExecutionScope::Host,
            binary_path: PathBuf::from("/usr/bin/codex"),
        };
        let mut manifest = AgentRuntimeParticipantRecord::new_orchestrator_participant(
            &descriptor,
            "sess_stop_helper".to_string(),
            "ash_stop_helper".to_string(),
            "lease_stop_helper".to_string(),
        )
        .expect("orchestrator participant");
        manifest.transition_state(AgentRuntimeSessionState::Ready);
        manifest.set_uaa_session_id("uaa_session");
        manifest.mark_runtime_ownership_retained();
        let mut orchestration = OrchestrationSessionRecord::new(
            "sess_stop_helper".to_string(),
            "trace_session".to_string(),
            "/workspace".to_string(),
            &manifest,
            HostAttachContract::from_manifest_for_test(&manifest),
        );
        orchestration.transition_state(OrchestrationSessionState::Active);
        orchestration.bind_active_session_handle(manifest.handle.participant_id.clone());

        apply_runtime_stop_closeout(&mut orchestration, &mut manifest);

        assert_eq!(manifest.handle.state, AgentRuntimeSessionState::Stopped);
        assert_eq!(
            manifest.internal.termination_reason.as_deref(),
            Some("stopped")
        );
        assert!(!manifest.internal.resume_eligible);
        assert!(!manifest.internal.attached_client_present);
        assert_eq!(orchestration.state, OrchestrationSessionState::Stopped);
        assert!(orchestration.closed_at.is_some());
        assert_eq!(
            orchestration.posture,
            crate::execution::agent_runtime::orchestration_session::OrchestrationSessionPosture::Terminal
        );
    }

    #[test]
    fn cancel_closeout_helper_converges_on_explicit_cancelled_terminal_truth() {
        let descriptor = RuntimeSelectionDescriptor {
            agent_id: "codex".to_string(),
            backend_id: "cli:codex_world".to_string(),
            backend_kind: AgentRuntimeBackendKind::Codex,
            protocol: PURE_AGENT_PROTOCOL.to_string(),
            execution_scope: AgentExecutionScope::World,
            binary_path: PathBuf::from("/usr/bin/codex"),
        };
        let mut manifest = AgentRuntimeParticipantRecord::new_member_participant(
            &descriptor,
            "sess_cancel_helper".to_string(),
            "ash_cancel_helper".to_string(),
            "orch_cancel_helper".to_string(),
            None,
            Some(AgentRuntimeParticipantWorldBinding {
                world_id: "world-17".to_string(),
                world_generation: 2,
            }),
            "lease_cancel_helper".to_string(),
        )
        .expect("member participant");
        manifest.transition_state(AgentRuntimeSessionState::Running);
        manifest.internal.latest_run_id = Some("run-cancel-helper".to_string());
        manifest.mark_runtime_ownership_retained();
        let mut orchestration = OrchestrationSessionRecord::new(
            "sess_cancel_helper".to_string(),
            "trace_session".to_string(),
            "/workspace".to_string(),
            &manifest,
            None,
        );
        orchestration.transition_state(OrchestrationSessionState::Active);

        apply_runtime_cancel_closeout(&mut orchestration, &mut manifest);

        assert_eq!(manifest.handle.state, AgentRuntimeSessionState::Invalidated);
        assert!(manifest.has_cancelled_terminal_truth());
        assert_eq!(
            manifest.internal.termination_reason.as_deref(),
            Some("cancelled")
        );
        assert_eq!(manifest.internal.latest_run_id, None);
        assert_eq!(orchestration.state, OrchestrationSessionState::Invalidated);
        assert!(orchestration.has_cancelled_terminal_truth());
        assert_eq!(
            orchestration.invalidation_reason.as_deref(),
            Some("cancelled")
        );
    }

    #[test]
    fn public_turn_prompt_requests_require_exact_session_and_backend_contract() {
        let prompt = LoadedPublicPrompt {
            prompt_text: "hello".to_string(),
        };
        let missing_session = PublicPromptCommandRequest {
            action: PublicPromptAction::Turn,
            orchestration_session_id: None,
            backend_id: "cli:codex".to_string(),
            prompt: prompt.clone(),
            json: false,
        };
        let missing_backend = PublicPromptCommandRequest {
            action: PublicPromptAction::Turn,
            orchestration_session_id: Some("sess_public".to_string()),
            backend_id: "   ".to_string(),
            prompt,
            json: false,
        };

        let session_err = validate_public_prompt_command_request(&missing_session)
            .expect_err("turn requests must require an orchestration session id");
        assert!(session_err
            .to_string()
            .contains("public turn actions require --session <orchestration_session_id>"));

        let backend_err = validate_public_prompt_command_request(&missing_backend)
            .expect_err("turn requests must require an exact backend id");
        assert!(backend_err
            .to_string()
            .contains("missing_backend: public turn actions require --backend <backend_id>"));
    }

    #[test]
    #[serial_test::serial]
    fn prompt_completion_session_state_surfaces_cancelled_participant_truth() {
        with_store(|store| {
            let descriptor = RuntimeSelectionDescriptor {
                agent_id: "codex".to_string(),
                backend_id: "cli:codex".to_string(),
                backend_kind: AgentRuntimeBackendKind::Codex,
                protocol: PURE_AGENT_PROTOCOL.to_string(),
                execution_scope: AgentExecutionScope::Host,
                binary_path: PathBuf::from("/usr/bin/codex"),
            };
            let mut manifest = AgentRuntimeParticipantRecord::new_orchestrator_participant(
                &descriptor,
                "sess_prompt_cancelled_participant".to_string(),
                "ash_prompt_cancelled_participant".to_string(),
                "lease_prompt_cancelled_participant".to_string(),
            )
            .expect("orchestrator participant");
            manifest.transition_state(AgentRuntimeSessionState::Running);
            manifest.mark_cancelled_terminal_state();

            let orchestration_session = OrchestrationSessionRecord::new(
                "sess_prompt_cancelled_participant".to_string(),
                "trace_session".to_string(),
                "/workspace".to_string(),
                &manifest,
                HostAttachContract::from_manifest_for_test(&manifest),
            );
            let runtime =
                prompt_submit_runtime_for_test(store, orchestration_session, manifest.clone());

            let (posture, state) = prompt_completion_session_state(&runtime);

            assert_eq!(posture, PublicSessionPosture::Terminal);
            assert_eq!(state, "cancelled");
        });
    }

    #[test]
    #[serial_test::serial]
    fn prompt_completion_session_state_surfaces_cancelled_session_truth() {
        with_store(|store| {
            let descriptor = RuntimeSelectionDescriptor {
                agent_id: "codex".to_string(),
                backend_id: "cli:codex".to_string(),
                backend_kind: AgentRuntimeBackendKind::Codex,
                protocol: PURE_AGENT_PROTOCOL.to_string(),
                execution_scope: AgentExecutionScope::Host,
                binary_path: PathBuf::from("/usr/bin/codex"),
            };
            let mut manifest = AgentRuntimeParticipantRecord::new_orchestrator_participant(
                &descriptor,
                "sess_prompt_cancelled_session".to_string(),
                "ash_prompt_cancelled_session".to_string(),
                "lease_prompt_cancelled_session".to_string(),
            )
            .expect("orchestrator participant");
            manifest.transition_state(AgentRuntimeSessionState::Running);

            let mut orchestration_session = OrchestrationSessionRecord::new(
                "sess_prompt_cancelled_session".to_string(),
                "trace_session".to_string(),
                "/workspace".to_string(),
                &manifest,
                HostAttachContract::from_manifest_for_test(&manifest),
            );
            orchestration_session.transition_state(OrchestrationSessionState::Active);
            orchestration_session.mark_cancelled_terminal();
            let runtime = prompt_submit_runtime_for_test(store, orchestration_session, manifest);

            let (posture, state) = prompt_completion_session_state(&runtime);

            assert_eq!(posture, PublicSessionPosture::Terminal);
            assert_eq!(state, "cancelled");
        });
    }

    #[test]
    #[serial_test::serial]
    fn start_timeout_reconciliation_parks_stale_attached_truth_into_detached_success() {
        with_store(|store| {
            let descriptor = RuntimeSelectionDescriptor {
                agent_id: "codex".to_string(),
                backend_id: "cli:codex".to_string(),
                backend_kind: AgentRuntimeBackendKind::Codex,
                protocol: PURE_AGENT_PROTOCOL.to_string(),
                execution_scope: AgentExecutionScope::Host,
                binary_path: PathBuf::from("/usr/bin/codex"),
            };
            let mut participant = AgentRuntimeParticipantRecord::new_orchestrator_participant(
                &descriptor,
                "sess_start_timeout_success".to_string(),
                "ash_start_timeout_success".to_string(),
                "lease_start_timeout_success".to_string(),
            )
            .expect("orchestrator participant");
            participant.transition_state(AgentRuntimeSessionState::Ready);
            participant.set_uaa_session_id("uaa_session");

            let mut orchestration = OrchestrationSessionRecord::new(
                "sess_start_timeout_success".to_string(),
                "trace_session".to_string(),
                "/workspace".to_string(),
                &participant,
                HostAttachContract::from_manifest_for_test(&participant),
            );
            orchestration.transition_state(OrchestrationSessionState::Active);
            orchestration.bind_active_session_handle(participant.handle.participant_id.clone());
            orchestration.initialize_startup_prompt(participant.handle.participant_id.clone());
            orchestration.mark_startup_prompt_completed(
                participant.handle.participant_id.as_str(),
                "success",
            );
            participant.mark_client_detached("owner detached cleanly");

            store
                .persist_orchestration_session(&orchestration)
                .expect("persist orchestration");
            store
                .persist_participant(&participant)
                .expect("persist participant");

            let reconciliation = reconcile_hidden_owner_helper_start_timeout(
                store,
                &test_plan("sess_start_timeout_success", "ash_start_timeout_success"),
            )
            .expect("reconcile timeout");
            assert_eq!(
                reconciliation,
                HiddenOwnerHelperStartTimeoutReconciliation::Success
            );

            let persisted = store
                .load_orchestration_session("sess_start_timeout_success")
                .expect("load orchestration")
                .expect("orchestration exists");
            assert_eq!(persisted.state, OrchestrationSessionState::Active);
            assert_eq!(
                persisted.posture,
                OrchestrationSessionPosture::ParkedResumable
            );
            assert_eq!(persisted.attached_participant_id(), None);

            let readiness = store
                .classify_hidden_owner_helper_launch_readiness(
                    "sess_start_timeout_success",
                    "ash_start_timeout_success",
                    false,
                )
                .expect("classify readiness");
            assert_eq!(
                readiness,
                crate::execution::agent_runtime::state_store::HiddenOwnerHelperLaunchReadiness::ReadyDetached(
                    OrchestrationSessionPosture::ParkedResumable
                )
            );
        });
    }

    #[test]
    #[serial_test::serial]
    fn start_timeout_reconciliation_projects_awaiting_attention_from_persisted_obligation() {
        with_store(|store| {
            let descriptor = RuntimeSelectionDescriptor {
                agent_id: "codex".to_string(),
                backend_id: "cli:codex".to_string(),
                backend_kind: AgentRuntimeBackendKind::Codex,
                protocol: PURE_AGENT_PROTOCOL.to_string(),
                execution_scope: AgentExecutionScope::Host,
                binary_path: PathBuf::from("/usr/bin/codex"),
            };
            let mut participant = AgentRuntimeParticipantRecord::new_orchestrator_participant(
                &descriptor,
                "sess_start_timeout_attention".to_string(),
                "ash_start_timeout_attention".to_string(),
                "lease_start_timeout_attention".to_string(),
            )
            .expect("orchestrator participant");
            participant.transition_state(AgentRuntimeSessionState::Ready);
            participant.set_uaa_session_id("uaa_session");

            let mut orchestration = OrchestrationSessionRecord::new(
                "sess_start_timeout_attention".to_string(),
                "trace_session".to_string(),
                "/workspace".to_string(),
                &participant,
                HostAttachContract::from_manifest_for_test(&participant),
            );
            orchestration.transition_state(OrchestrationSessionState::Active);
            orchestration.bind_active_session_handle(participant.handle.participant_id.clone());
            orchestration.initialize_startup_prompt(participant.handle.participant_id.clone());
            orchestration.mark_startup_prompt_completed(
                participant.handle.participant_id.as_str(),
                "success",
            );
            participant.mark_client_detached("owner detached cleanly");

            store
                .persist_orchestration_session(&orchestration)
                .expect("persist orchestration");
            store
                .persist_participant(&participant)
                .expect("persist participant");

            let mut obligation = OrchestrationObligationRecord::new(
                "sess_start_timeout_attention",
                "obl_attention",
                OrchestrationObligationKind::FollowUpRequired,
                "attention needed before host resumes",
            );
            obligation.attention_required = true;
            obligation.attach_state = OrchestrationObligationAttachState::Eligible;
            store
                .persist_obligation(&obligation)
                .expect("persist obligation");

            let reconciliation = reconcile_hidden_owner_helper_start_timeout(
                store,
                &test_plan(
                    "sess_start_timeout_attention",
                    "ash_start_timeout_attention",
                ),
            )
            .expect("reconcile timeout");
            assert_eq!(
                reconciliation,
                HiddenOwnerHelperStartTimeoutReconciliation::Success
            );

            let persisted = store
                .load_orchestration_session("sess_start_timeout_attention")
                .expect("load orchestration")
                .expect("orchestration exists");
            assert_eq!(persisted.state, OrchestrationSessionState::Active);
            assert_eq!(
                persisted.posture,
                OrchestrationSessionPosture::AwaitingAttention
            );
            assert_eq!(persisted.pending_inbox_count, 1);

            let readiness = store
                .classify_hidden_owner_helper_launch_readiness(
                    "sess_start_timeout_attention",
                    "ash_start_timeout_attention",
                    false,
                )
                .expect("classify readiness");
            assert_eq!(
                readiness,
                crate::execution::agent_runtime::state_store::HiddenOwnerHelperLaunchReadiness::ReadyDetached(
                    OrchestrationSessionPosture::AwaitingAttention
                )
            );
        });
    }

    #[test]
    #[serial_test::serial]
    fn helper_readiness_does_not_wait_for_startup_prompt_terminalization_once_attached_live() {
        with_store(|store| {
            let descriptor = RuntimeSelectionDescriptor {
                agent_id: "codex".to_string(),
                backend_id: "cli:codex".to_string(),
                backend_kind: AgentRuntimeBackendKind::Codex,
                protocol: PURE_AGENT_PROTOCOL.to_string(),
                execution_scope: AgentExecutionScope::Host,
                binary_path: PathBuf::from("/usr/bin/codex"),
            };
            let mut participant = AgentRuntimeParticipantRecord::new_orchestrator_participant(
                &descriptor,
                "sess_start_ready_attached".to_string(),
                "ash_start_ready_attached".to_string(),
                "lease_start_ready_attached".to_string(),
            )
            .expect("orchestrator participant");
            participant.transition_state(AgentRuntimeSessionState::Ready);
            participant.set_uaa_session_id("uaa_session");
            participant.mark_runtime_ownership_retained();

            let mut orchestration = OrchestrationSessionRecord::new(
                "sess_start_ready_attached".to_string(),
                "trace_session".to_string(),
                "/workspace".to_string(),
                &participant,
                HostAttachContract::from_manifest_for_test(&participant),
            );
            orchestration.transition_state(OrchestrationSessionState::Active);
            orchestration.bind_active_session_handle(participant.handle.participant_id.clone());
            orchestration.initialize_startup_prompt(participant.handle.participant_id.clone());
            orchestration.mark_startup_prompt_accepted(participant.handle.participant_id.as_str());

            store
                .persist_orchestration_session(&orchestration)
                .expect("persist orchestration");
            store
                .persist_participant(&participant)
                .expect("persist participant");

            super::wait_for_hidden_owner_helper_readiness(
                store,
                &test_plan("sess_start_ready_attached", "ash_start_ready_attached"),
            )
            .expect("attached live readiness should not wait for startup prompt completion");
        });
    }

    #[test]
    #[serial_test::serial]
    fn helper_readiness_accepts_resume_one_turn_after_fast_detach_once_prompt_is_terminal() {
        with_store(|store| {
            let descriptor = RuntimeSelectionDescriptor {
                agent_id: "codex".to_string(),
                backend_id: "cli:codex".to_string(),
                backend_kind: AgentRuntimeBackendKind::Codex,
                protocol: PURE_AGENT_PROTOCOL.to_string(),
                execution_scope: AgentExecutionScope::Host,
                binary_path: PathBuf::from("/usr/bin/codex"),
            };
            let mut participant = AgentRuntimeParticipantRecord::new_replacement_participant(
                &descriptor,
                AgentRuntimeReplacementParticipantInit {
                    orchestration_session_id: "sess_resume_turn_ready".to_string(),
                    participant_id: "ash_resume_turn_ready".to_string(),
                    role: ORCHESTRATOR_ROLE.to_string(),
                    orchestrator_participant_id: None,
                    parent_participant_id: None,
                    resumed_from_participant_id: "ash_source".to_string(),
                    world: None,
                    lease_token: "lease_resume_turn_ready".to_string(),
                },
            )
            .expect("replacement participant");
            participant.transition_state(AgentRuntimeSessionState::Ready);
            participant.set_uaa_session_id("thread-test");
            participant.mark_client_detached("owner detached cleanly");

            let mut orchestration = OrchestrationSessionRecord::new(
                "sess_resume_turn_ready".to_string(),
                "trace_session".to_string(),
                "/workspace".to_string(),
                &participant,
                HostAttachContract::from_manifest_for_test(&participant),
            );
            orchestration.transition_state(OrchestrationSessionState::Active);
            orchestration.bind_active_session_handle(participant.handle.participant_id.clone());
            orchestration.mark_parked_resumable("owner detached cleanly");
            orchestration.initialize_startup_prompt(participant.handle.participant_id.clone());
            orchestration.mark_startup_prompt_completed(
                participant.handle.participant_id.as_str(),
                "success",
            );

            store
                .persist_orchestration_session(&orchestration)
                .expect("persist orchestration");
            store
                .persist_participant(&participant)
                .expect("persist participant");

            super::wait_for_hidden_owner_helper_readiness(
                store,
                &resumed_turn_test_plan(
                    "sess_resume_turn_ready",
                    "ash_resume_turn_ready",
                    "ash_source",
                    "thread-test",
                ),
            )
            .expect("resume_one_turn readiness should accept a terminalized fast-detach handoff");
        });
    }

    #[test]
    #[serial_test::serial]
    fn inflight_attach_join_returns_authoritative_receipt_when_ready_state_already_exists() {
        with_store(|store| {
            let descriptor = RuntimeSelectionDescriptor {
                agent_id: "codex".to_string(),
                backend_id: "cli:codex".to_string(),
                backend_kind: AgentRuntimeBackendKind::Codex,
                protocol: PURE_AGENT_PROTOCOL.to_string(),
                execution_scope: AgentExecutionScope::Host,
                binary_path: PathBuf::from("/usr/bin/codex"),
            };
            let mut participant = AgentRuntimeParticipantRecord::new_orchestrator_participant(
                &descriptor,
                "sess_attach_join".to_string(),
                "ash_attach_join".to_string(),
                "lease_attach_join".to_string(),
            )
            .expect("orchestrator participant");
            participant.transition_state(AgentRuntimeSessionState::Ready);
            participant.set_uaa_session_id("uaa_attach_join");
            participant.mark_runtime_ownership_retained();

            let mut orchestration = OrchestrationSessionRecord::new(
                "sess_attach_join".to_string(),
                "trace_session".to_string(),
                "/workspace".to_string(),
                &participant,
                HostAttachContract::from_manifest_for_test(&participant),
            );
            orchestration.transition_state(OrchestrationSessionState::Active);
            orchestration.bind_active_session_handle(participant.handle.participant_id.clone());

            store
                .persist_orchestration_session(&orchestration)
                .expect("persist attached orchestration");
            store
                .persist_participant(&participant)
                .expect("persist attached participant");

            let plan = attach_test_plan("sess_attach_join", "ash_attach_join", "uaa_attach_join");
            let _guard = super::try_acquire_hidden_owner_helper_attach_launch_guard(store, &plan)
                .expect("acquire attach launch guard")
                .expect("attach launch guard must be available");

            let joined = super::wait_for_inflight_hidden_owner_helper_attach_launch(store, &plan)
                .expect("join existing ready attach launch");

            let super::HiddenOwnerHelperAttachJoinResult::Joined(receipt) = joined else {
                panic!("ready attach join must not fall back to a new leader launch");
            };
            assert_eq!(receipt.orchestration_session_id, "sess_attach_join");
            assert_eq!(receipt.participant_id, "ash_attach_join");
            assert_eq!(receipt.backend_id, "cli:codex");
            assert_eq!(receipt.helper_pid, std::process::id());
        });
    }

    #[test]
    #[serial_test::serial]
    fn inflight_attach_join_does_not_accept_detached_live_owner_as_attached_success() {
        with_store(|store| {
            let descriptor = RuntimeSelectionDescriptor {
                agent_id: "codex".to_string(),
                backend_id: "cli:codex".to_string(),
                backend_kind: AgentRuntimeBackendKind::Codex,
                protocol: PURE_AGENT_PROTOCOL.to_string(),
                execution_scope: AgentExecutionScope::Host,
                binary_path: PathBuf::from("/usr/bin/codex"),
            };
            let mut participant = AgentRuntimeParticipantRecord::new_orchestrator_participant(
                &descriptor,
                "sess_attach_join_detached".to_string(),
                "ash_attach_join_detached".to_string(),
                "lease_attach_join_detached".to_string(),
            )
            .expect("orchestrator participant");
            participant.transition_state(AgentRuntimeSessionState::Ready);
            participant.set_uaa_session_id("uaa_attach_join_detached");
            participant.mark_runtime_ownership_retained();
            participant.mark_client_detached("owner detached cleanly");

            let mut orchestration = OrchestrationSessionRecord::new(
                "sess_attach_join_detached".to_string(),
                "trace_session".to_string(),
                "/workspace".to_string(),
                &participant,
                HostAttachContract::from_manifest_for_test(&participant),
            );
            orchestration.transition_state(OrchestrationSessionState::Active);
            orchestration.bind_active_session_handle(participant.handle.participant_id.clone());
            orchestration.mark_parked_resumable("owner detached cleanly");

            store
                .persist_orchestration_session(&orchestration)
                .expect("persist detached orchestration");
            store
                .persist_participant(&participant)
                .expect("persist detached participant");

            let plan = attach_test_plan(
                "sess_attach_join_detached",
                "ash_attach_join_detached",
                "uaa_attach_join_detached",
            );
            let guard = super::try_acquire_hidden_owner_helper_attach_launch_guard(store, &plan)
                .expect("acquire attach launch guard")
                .expect("attach launch guard must be available");

            let release_thread = std::thread::spawn(move || {
                std::thread::sleep(std::time::Duration::from_millis(100));
                drop(guard);
            });

            let joined = super::wait_for_inflight_hidden_owner_helper_attach_launch(store, &plan)
                .expect("detached live owner should not satisfy attach join");
            release_thread
                .join()
                .expect("release thread should finish cleanly");

            let super::HiddenOwnerHelperAttachJoinResult::RetryAsLeader = joined else {
                panic!("detached parked owner must not satisfy attach join success");
            };
        });
    }

    #[test]
    #[serial_test::serial]
    fn start_timeout_reconciliation_marks_terminal_failure_when_startup_prompt_is_not_terminal() {
        with_store(|store| {
            let descriptor = RuntimeSelectionDescriptor {
                agent_id: "codex".to_string(),
                backend_id: "cli:codex".to_string(),
                backend_kind: AgentRuntimeBackendKind::Codex,
                protocol: PURE_AGENT_PROTOCOL.to_string(),
                execution_scope: AgentExecutionScope::Host,
                binary_path: PathBuf::from("/usr/bin/codex"),
            };
            let mut participant = AgentRuntimeParticipantRecord::new_orchestrator_participant(
                &descriptor,
                "sess_start_timeout_failure".to_string(),
                "ash_start_timeout_failure".to_string(),
                "lease_start_timeout_failure".to_string(),
            )
            .expect("orchestrator participant");
            participant.transition_state(AgentRuntimeSessionState::Ready);
            participant.set_uaa_session_id("uaa_session");

            let mut orchestration = OrchestrationSessionRecord::new(
                "sess_start_timeout_failure".to_string(),
                "trace_session".to_string(),
                "/workspace".to_string(),
                &participant,
                HostAttachContract::from_manifest_for_test(&participant),
            );
            orchestration.transition_state(OrchestrationSessionState::Active);
            orchestration.bind_active_session_handle(participant.handle.participant_id.clone());
            orchestration.initialize_startup_prompt(participant.handle.participant_id.clone());
            orchestration.mark_startup_prompt_accepted(participant.handle.participant_id.as_str());
            participant.mark_client_detached("owner detached cleanly");

            store
                .persist_orchestration_session(&orchestration)
                .expect("persist orchestration");
            store
                .persist_participant(&participant)
                .expect("persist participant");

            let reconciliation = reconcile_hidden_owner_helper_start_timeout(
                store,
                &test_plan("sess_start_timeout_failure", "ash_start_timeout_failure"),
            )
            .expect("reconcile timeout");
            assert_eq!(
                reconciliation,
                HiddenOwnerHelperStartTimeoutReconciliation::FailureMarkedTerminal
            );

            let persisted = store
                .load_orchestration_session("sess_start_timeout_failure")
                .expect("load orchestration")
                .expect("orchestration exists");
            assert_eq!(persisted.state, OrchestrationSessionState::Failed);
            assert_eq!(persisted.posture, OrchestrationSessionPosture::Terminal);
            assert_eq!(persisted.attached_participant_id(), None);
            assert_eq!(
                persisted.startup_prompt_state(),
                Some(StartupPromptStreamState::Failed)
            );

            let persisted_participant = store
                .load_participant("ash_start_timeout_failure")
                .expect("load participant")
                .expect("participant exists");
            assert_eq!(
                persisted_participant.handle.state,
                AgentRuntimeSessionState::Invalidated
            );
            assert!(persisted_participant
                .internal
                .terminal_observed_at
                .is_some());
            assert_eq!(
                persisted_participant.internal.last_error_bucket.as_deref(),
                Some("bootstrap_run")
            );
        });
    }

    #[cfg(unix)]
    #[tokio::test(flavor = "current_thread")]
    async fn private_prompt_bridge_emits_terminal_failed_after_accepted_owner_drop() {
        let (client, server) = UnixStream::pair().expect("unix stream pair");
        let (prompt_tx, mut prompt_rx) = private_prompt_request_channel();
        let server_task = tokio::spawn(async move {
            handle_private_prompt_connection(server, prompt_tx)
                .await
                .expect("private prompt bridge should complete");
        });
        let owner_task = tokio::spawn(async move {
            let request = prompt_rx.recv().await.expect("prompt request");
            request
                .envelope_tx
                .send(PublicPromptEnvelope::Accepted {
                    version: 1,
                    action: PublicPromptAction::Turn,
                    orchestration_session_id: "orch-parked".to_string(),
                    backend_id: "cli:codex".to_string(),
                    participant_id: Some("ash_resumed".to_string()),
                    scope: "host".to_string(),
                })
                .expect("accepted envelope should send");
        });

        let mut client = client;
        client
            .write_all(br#"{"version":1,"action":"turn","prompt":"resume"}"#)
            .await
            .expect("write request");
        client.write_all(b"\n").await.expect("newline");
        client.flush().await.expect("flush request");

        let mut reader = BufReader::new(client);
        let mut line = String::new();
        let mut envelopes = Vec::new();
        loop {
            line.clear();
            let bytes_read = reader.read_line(&mut line).await.expect("read envelope");
            if bytes_read == 0 {
                break;
            }
            envelopes.push(
                serde_json::from_str::<PublicPromptEnvelope>(line.trim())
                    .expect("decode prompt envelope"),
            );
        }

        owner_task.await.expect("owner task");
        server_task.await.expect("server task");

        assert!(matches!(
            envelopes.first(),
            Some(PublicPromptEnvelope::Accepted { .. })
        ));
        assert!(matches!(
            envelopes.get(1),
            Some(PublicPromptEnvelope::Failed {
                error_code,
                message,
                ..
            }) if error_code == "owner_unreachable"
                && message.contains("closed after accepting")
        ));
    }

    #[test]
    fn world_task_terminal_state_tracks_public_prompt_exit_semantics() {
        assert_eq!(
            super::world_task_terminal_state_from_exit_code(0),
            super::super::dispatch_contract::WorldTaskTerminalStateV1::Completed
        );
        assert_eq!(
            super::world_task_terminal_state_from_exit_code(130),
            super::super::dispatch_contract::WorldTaskTerminalStateV1::Cancelled
        );
        assert_eq!(
            super::world_task_terminal_state_from_exit_code(17),
            super::super::dispatch_contract::WorldTaskTerminalStateV1::Failed
        );
    }
}
