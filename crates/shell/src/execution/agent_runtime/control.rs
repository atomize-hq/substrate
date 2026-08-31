use std::collections::{BTreeMap, BTreeSet};
use std::env;
use std::fs;
use std::fs::File;
#[cfg(unix)]
use std::io::Write;
use std::io::{self, Read};
#[cfg(target_os = "linux")]
use std::os::fd::AsRawFd;
#[cfg(target_os = "linux")]
use std::os::unix::fs::{FileTypeExt, MetadataExt, PermissionsExt};
#[cfg(unix)]
use std::os::unix::net::UnixListener as StdUnixListener;
#[cfg(target_os = "linux")]
use std::os::unix::net::UnixStream as StdUnixStream;
#[cfg(target_os = "linux")]
use std::os::unix::process::CommandExt;
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
use sha2::{Digest, Sha256};
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
use crate::execution::agent_runtime::host_session_authority::schema::{
    HostSessionPostureV1, HostSessionTransitionCallerKindV1, HostSessionTransitionCallerV1,
    TimestampV1,
};
#[cfg(unix)]
use crate::execution::agent_runtime::host_session_authority::start_continuity::StartTurnCompletionKindV1;
#[cfg(unix)]
use crate::execution::agent_runtime::host_session_authority::stop::{
    stop_delivery, AcceptHostSessionStopDeliveryRequestV1, CompleteHostSessionStopRequestV1,
    HostSessionStopCompletionOutcomeV1, HostSessionStopDeliveryOutcomeV1,
    HostSessionStopDeliveryV1,
};
#[cfg(target_os = "linux")]
use crate::execution::agent_runtime::host_session_authority::store_schema::{
    HostSessionPostTurnApplicationV2, HostSessionStartupOwnershipApplicationV1,
    HostSessionTransitionIntentStateV3, HostSessionTransitionIntentV3,
};
#[cfg(unix)]
use crate::execution::agent_runtime::host_session_authority::store_schema::{
    StartTransactionRecordV1, StartTransactionStateV1,
};
use crate::execution::agent_runtime::host_session_authority::trusted_fs::TrustedAuthorityRoot;
use crate::execution::agent_runtime::host_session_authority::{
    HostSessionAuthority, ResolvedCurrentAuthorityV1,
};
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
#[cfg(target_os = "linux")]
const AUTHORITY_SUCCESSOR_LAUNCH_GUARD_FD_ENV: &str =
    "SUBSTRATE_AUTHORITY_SUCCESSOR_LAUNCH_GUARD_FD";
const OWNER_HELPER_READY_TIMEOUT_ERROR_PREFIX: &str =
    "timed out waiting for authoritative owner-helper readiness for orchestration session ";
const OWNER_HELPER_READY_TIMEOUT: Duration = Duration::from_secs(30);
const OWNER_HELPER_READY_POLL_INTERVAL: Duration = Duration::from_millis(100);
const PRIVATE_STOP_UNIX_PATH_MAX: usize = 100;
#[cfg(unix)]
const PRIVATE_HSA_STOP_RESPONSE_TIMEOUT: Duration = Duration::from_secs(10);
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

/// One process-local host execution episode. Every field is observational;
/// durable session authority remains exclusively owned by HostSessionAuthority.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct HostExecutionEpisodeV1 {
    pub(crate) schema_version: u32,
    pub(crate) episode_id: String,
    pub(crate) kind: HostExecutionEpisodeKindV1,
    pub(crate) orchestration_session_id: String,
    pub(crate) observed_authority_revision: u64,
    pub(crate) backend_id: Option<String>,
    pub(crate) process_ref: Option<ProcessRefV1>,
    pub(crate) transport_status: HostExecutionEpisodeTransportStatusV1,
    pub(crate) started_at: TimestampV1,
    pub(crate) last_heartbeat_at: Option<TimestampV1>,
    pub(crate) ended_at: Option<TimestampV1>,
    pub(crate) exit_observation: Option<EpisodeExitObservationV1>,
}

#[allow(clippy::enum_variant_names)]
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub(crate) enum HostExecutionEpisodeKindV1 {
    ReplAttachedEpisode,
    HiddenOwnerHelperStartEpisode,
    HiddenOwnerHelperAttachEpisode,
    HiddenOwnerHelperResumeOneTurnEpisode,
    RuntimeToolboxEpisode,
    SyntheticOrRecoveredEpisode,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub(crate) enum HostExecutionEpisodeTransportStatusV1 {
    Available,
    UnavailableButDurableAuthorityExists,
    UnavailableAndNoAuthoritativeRoute,
    StaleOrOrphaned,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub(crate) enum PrivateTransportAvailabilityV1 {
    Available,
    Missing,
    Refused,
    Failed,
    Stale,
}

#[cfg(target_os = "linux")]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct PrivateEpisodeEndpointIdentityV1 {
    device: u64,
    inode: u64,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ProcessRefV1 {
    pub(crate) pid: u32,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct EpisodeExitObservationV1 {
    pub(crate) observed_at: TimestampV1,
    pub(crate) exit_code: Option<i32>,
    pub(crate) signal: Option<i32>,
}

/// Runtime-only facts that may be associated with an episode. None of these
/// variants is accepted as durable lifecycle or authority evidence.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(tag = "observation", content = "value", deny_unknown_fields)]
pub(crate) enum HostExecutionEpisodeObservationV1 {
    Process(ProcessRefV1),
    Heartbeat {
        observed_at: TimestampV1,
    },
    Readiness {
        ready: bool,
    },
    Prompt {
        accepted: bool,
    },
    Handle {
        present: bool,
    },
    Stream {
        active: bool,
    },
    Endpoint {
        availability: PrivateTransportAvailabilityV1,
    },
    Timeout {
        timed_out: bool,
    },
    Exit(EpisodeExitObservationV1),
}

/// Complete identity needed to bind an observational episode to an exact HSA
/// authority without adding participant or store fields to the canonical V1
/// episode schema.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct HostExecutionEpisodeBindingV1 {
    pub(crate) authority_store_id: String,
    pub(crate) orchestration_session_id: String,
    pub(crate) participant_id: String,
    pub(crate) episode_id: String,
    pub(crate) authority_revision: u64,
}

#[cfg(target_os = "linux")]
#[derive(Clone, Debug)]
pub(crate) struct HostExecutionEpisodeObserverV1 {
    state: Arc<Mutex<HostExecutionEpisodeObserverStateV1>>,
}

#[cfg(target_os = "linux")]
#[derive(Debug)]
struct HostExecutionEpisodeObserverStateV1 {
    episode: HostExecutionEpisodeV1,
    binding: HostExecutionEpisodeBindingV1,
    observed_keys: BTreeSet<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct HostExecutionEpisodeTransportProjectionV1 {
    pub(crate) episode: HostExecutionEpisodeV1,
    pub(crate) binding: HostExecutionEpisodeBindingV1,
    pub(crate) availability: PrivateTransportAvailabilityV1,
    pub(crate) endpoint_path: PathBuf,
    pub(crate) agent_id: String,
    pub(crate) backend_id: String,
    pub(crate) world_id: Option<String>,
    pub(crate) world_generation: Option<u64>,
}

impl HostExecutionEpisodeV1 {
    pub(crate) fn validate_binding(&self, binding: &HostExecutionEpisodeBindingV1) -> Result<()> {
        if self.schema_version != 1
            || self.episode_id.is_empty()
            || self.orchestration_session_id.is_empty()
            || self.observed_authority_revision == 0
            || self
                .process_ref
                .as_ref()
                .is_some_and(|process| process.pid == 0)
            || binding.authority_store_id.is_empty()
            || binding.participant_id.is_empty()
            || self.episode_id != binding.episode_id
            || self.orchestration_session_id != binding.orchestration_session_id
            || self.observed_authority_revision != binding.authority_revision
        {
            anyhow::bail!("host execution episode does not match exact authority binding");
        }
        Ok(())
    }

    /// Records episode-local state only. Callers that can affect an
    /// authority-owned surface must first use `validate_current_authority`.
    pub(crate) fn record_local_observation(
        &mut self,
        binding: &HostExecutionEpisodeBindingV1,
        observation: HostExecutionEpisodeObservationV1,
    ) -> Result<bool> {
        self.validate_binding(binding)?;
        match observation {
            HostExecutionEpisodeObservationV1::Process(process) => {
                if process.pid == 0 {
                    anyhow::bail!("PID zero is not a host execution episode observation");
                }
                match self.process_ref.as_ref() {
                    Some(current) if current == &process => Ok(false),
                    Some(_) => anyhow::bail!(
                        "a competing process cannot replace an existing episode process binding"
                    ),
                    None => {
                        self.process_ref = Some(process);
                        Ok(true)
                    }
                }
            }
            HostExecutionEpisodeObservationV1::Heartbeat { observed_at } => {
                if self
                    .last_heartbeat_at
                    .as_ref()
                    .is_some_and(|current| current.as_str() >= observed_at.as_str())
                {
                    return Ok(false);
                }
                self.last_heartbeat_at = Some(observed_at);
                Ok(true)
            }
            HostExecutionEpisodeObservationV1::Endpoint { availability } => {
                let status = classify_episode_transport(availability, true);
                if self.transport_status == status {
                    return Ok(false);
                }
                self.transport_status = status;
                Ok(true)
            }
            HostExecutionEpisodeObservationV1::Exit(exit) => {
                if let Some(current) = self.exit_observation.as_ref() {
                    if current == &exit {
                        return Ok(false);
                    }
                    anyhow::bail!("conflicting terminal observation for one execution episode");
                }
                self.ended_at = Some(exit.observed_at.clone());
                self.exit_observation = Some(exit);
                Ok(true)
            }
            HostExecutionEpisodeObservationV1::Readiness { .. }
            | HostExecutionEpisodeObservationV1::Prompt { .. }
            | HostExecutionEpisodeObservationV1::Handle { .. }
            | HostExecutionEpisodeObservationV1::Stream { .. }
            | HostExecutionEpisodeObservationV1::Timeout { .. } => Ok(false),
        }
    }

    pub(crate) fn validate_current_authority(
        &self,
        binding: &HostExecutionEpisodeBindingV1,
        current: &ResolvedCurrentAuthorityV1,
    ) -> Result<()> {
        self.validate_binding(binding)?;
        if binding.authority_store_id != current.observation.authority_store_id
            || binding.orchestration_session_id != current.authority.orchestration_session_id
            || binding.participant_id != current.caller.participant_id
            || binding.authority_revision != current.observation.authority_revision
        {
            anyhow::bail!("host execution episode observed a stale or substituted authority");
        }
        Ok(())
    }
}

#[cfg(target_os = "linux")]
impl HostExecutionEpisodeObserverV1 {
    pub(crate) fn new(
        episode: HostExecutionEpisodeV1,
        binding: HostExecutionEpisodeBindingV1,
    ) -> Result<Self> {
        episode.validate_binding(&binding)?;
        Ok(Self {
            state: Arc::new(Mutex::new(HostExecutionEpisodeObserverStateV1 {
                episode,
                binding,
                observed_keys: BTreeSet::new(),
            })),
        })
    }

    /// Records an idempotent process-local observation only after the episode's
    /// exact HSA revision is still current. This method has no authority write.
    pub(crate) fn observe_current(
        &self,
        observation_key: &str,
        observation: HostExecutionEpisodeObservationV1,
    ) -> Result<bool> {
        if observation_key.is_empty() {
            anyhow::bail!("host execution episode observation key must not be empty");
        }
        let mut state = self
            .state
            .lock()
            .map_err(|_| anyhow::anyhow!("host execution episode observer mutex poisoned"))?;
        validate_host_execution_episode_against_current_hsa(&state.episode, &state.binding)?;
        if state.observed_keys.contains(observation_key) {
            return Ok(false);
        }
        let binding = state.binding.clone();
        let changed = state
            .episode
            .record_local_observation(&binding, observation)?;
        state.observed_keys.insert(observation_key.to_string());
        Ok(changed)
    }
}

impl HostExecutionEpisodeKindV1 {
    fn identity_label(self) -> &'static str {
        match self {
            Self::ReplAttachedEpisode => "repl-attached",
            Self::HiddenOwnerHelperStartEpisode => "hidden-owner-helper-start",
            Self::HiddenOwnerHelperAttachEpisode => "hidden-owner-helper-attach",
            Self::HiddenOwnerHelperResumeOneTurnEpisode => "hidden-owner-helper-resume-one-turn",
            Self::RuntimeToolboxEpisode => "runtime-toolbox",
            Self::SyntheticOrRecoveredEpisode => "synthetic-or-recovered",
        }
    }
}

pub(crate) fn classify_episode_transport(
    availability: PrivateTransportAvailabilityV1,
    durable_authority_exists: bool,
) -> HostExecutionEpisodeTransportStatusV1 {
    match availability {
        PrivateTransportAvailabilityV1::Available => {
            HostExecutionEpisodeTransportStatusV1::Available
        }
        PrivateTransportAvailabilityV1::Stale => {
            HostExecutionEpisodeTransportStatusV1::StaleOrOrphaned
        }
        PrivateTransportAvailabilityV1::Missing
        | PrivateTransportAvailabilityV1::Refused
        | PrivateTransportAvailabilityV1::Failed => {
            if durable_authority_exists {
                HostExecutionEpisodeTransportStatusV1::UnavailableButDurableAuthorityExists
            } else {
                HostExecutionEpisodeTransportStatusV1::UnavailableAndNoAuthoritativeRoute
            }
        }
    }
}

pub(crate) fn host_execution_episode_for_authority(
    current: &ResolvedCurrentAuthorityV1,
    kind: HostExecutionEpisodeKindV1,
    episode_seed: &str,
    process_ref: Option<ProcessRefV1>,
    availability: PrivateTransportAvailabilityV1,
) -> Result<(HostExecutionEpisodeV1, HostExecutionEpisodeBindingV1)> {
    let participant_id = current.caller.participant_id.clone();
    if current
        .authority
        .active_authoritative_participant_id
        .as_deref()
        != Some(participant_id.as_str())
    {
        anyhow::bail!("host execution episode requires the exact active HSA participant");
    }
    let episode_id = format!(
        "hee_{}",
        &episode_path_digest(
            "substrate.host-execution-episode.identity.v1",
            &[
                current.observation.authority_store_id.as_bytes(),
                current.authority.orchestration_session_id.as_bytes(),
                participant_id.as_bytes(),
                kind.identity_label().as_bytes(),
                episode_seed.as_bytes(),
            ],
        )[..32]
    );
    let binding = HostExecutionEpisodeBindingV1 {
        authority_store_id: current.observation.authority_store_id.clone(),
        orchestration_session_id: current.authority.orchestration_session_id.clone(),
        participant_id,
        episode_id: episode_id.clone(),
        authority_revision: current.observation.authority_revision,
    };
    let started_at =
        TimestampV1::parse(Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Nanos, true))
            .map_err(|error| anyhow::anyhow!(error.to_string()))?;
    let episode = HostExecutionEpisodeV1 {
        schema_version: 1,
        episode_id,
        kind,
        orchestration_session_id: current.authority.orchestration_session_id.clone(),
        observed_authority_revision: current.observation.authority_revision,
        backend_id: Some(current.caller.descriptor.backend_id.clone()),
        process_ref,
        transport_status: classify_episode_transport(availability, true),
        started_at,
        last_heartbeat_at: None,
        ended_at: None,
        exit_observation: None,
    };
    episode.validate_current_authority(&binding, current)?;
    Ok((episode, binding))
}

#[cfg(target_os = "linux")]
pub(crate) fn host_execution_episode_for_current_authority(
    orchestration_session_id: &str,
    kind: HostExecutionEpisodeKindV1,
    episode_seed: &str,
    process_ref: Option<ProcessRefV1>,
    availability: PrivateTransportAvailabilityV1,
) -> Result<(HostExecutionEpisodeV1, HostExecutionEpisodeBindingV1)> {
    let authority = HostSessionAuthority::from_trusted_root(
        TrustedAuthorityRoot::open(&substrate_paths::substrate_home()?)
            .map_err(|error| anyhow::anyhow!(error.to_string()))?,
    )
    .map_err(|error| anyhow::anyhow!(error.to_string()))?;
    let current = authority
        .resolve_current_exact(orchestration_session_id, None)
        .map_err(|error| anyhow::anyhow!(error.to_string()))?;
    host_execution_episode_for_authority(&current, kind, episode_seed, process_ref, availability)
}

#[cfg(target_os = "linux")]
fn host_execution_episode_for_helper_plan(
    plan: &HiddenOwnerHelperLaunchPlan,
) -> Result<(HostExecutionEpisodeV1, HostExecutionEpisodeBindingV1)> {
    let authority = HostSessionAuthority::from_trusted_root(
        TrustedAuthorityRoot::open(&substrate_paths::substrate_home()?)
            .map_err(|error| anyhow::anyhow!(error.to_string()))?,
    )
    .map_err(|error| anyhow::anyhow!(error.to_string()))?;
    let current = authority
        .resolve_current_exact(plan.orchestration_session_id(), None)
        .map_err(|error| anyhow::anyhow!(error.to_string()))?;
    if current.caller.participant_id != plan.participant_id() {
        anyhow::bail!("owner-helper episode participant is not the exact current HSA participant");
    }
    let kind = match plan.mode {
        OwnerHelperMode::Start => HostExecutionEpisodeKindV1::HiddenOwnerHelperStartEpisode,
        OwnerHelperMode::Attach => HostExecutionEpisodeKindV1::HiddenOwnerHelperAttachEpisode,
        OwnerHelperMode::ResumeOneTurn => {
            HostExecutionEpisodeKindV1::HiddenOwnerHelperResumeOneTurnEpisode
        }
    };
    host_execution_episode_for_authority(
        &current,
        kind,
        &plan.participant.run_id,
        None,
        PrivateTransportAvailabilityV1::Missing,
    )
}

fn episode_path_digest(domain: &str, parts: &[&[u8]]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(domain.as_bytes());
    for part in parts {
        hasher.update((part.len() as u64).to_be_bytes());
        hasher.update(part);
    }
    hasher
        .finalize()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

pub(crate) fn toolbox_transport_path_for_episode(
    substrate_home: &Path,
    uid: u32,
    binding: &HostExecutionEpisodeBindingV1,
) -> PathBuf {
    let home = substrate_home.as_os_str().as_encoded_bytes();
    let uid_bytes = uid.to_be_bytes();
    let store_namespace = episode_path_digest(
        "substrate.host-execution-episode.store-path.v1",
        &[home, &uid_bytes, binding.authority_store_id.as_bytes()],
    );
    let endpoint_identity = episode_path_digest(
        "substrate.host-execution-episode.toolbox-path.v1",
        &[
            home,
            &uid_bytes,
            binding.authority_store_id.as_bytes(),
            binding.orchestration_session_id.as_bytes(),
            binding.participant_id.as_bytes(),
            binding.episode_id.as_bytes(),
        ],
    );
    let suffix = PathBuf::from(format!("u{uid}"))
        .join(&store_namespace[..16])
        .join(format!("{}.sock", &endpoint_identity[..32]));
    let preferred = substrate_home
        .join("run")
        .join("agent-toolbox")
        .join(&suffix);
    if preferred.as_os_str().len() <= PRIVATE_STOP_UNIX_PATH_MAX {
        return preferred;
    }
    PathBuf::from("/tmp")
        .join(format!("substrate-agent-toolbox-u{uid}"))
        .join(&store_namespace[..16])
        .join(format!("{}.sock", &endpoint_identity[..32]))
}

#[cfg(target_os = "linux")]
pub(crate) fn private_transport_availability(
    path: &Path,
    expected_uid: u32,
) -> PrivateTransportAvailabilityV1 {
    let metadata_availability =
        private_transport_endpoint_metadata_availability(path, expected_uid);
    if metadata_availability != PrivateTransportAvailabilityV1::Available {
        return metadata_availability;
    }
    match StdUnixStream::connect(path) {
        Ok(_) => PrivateTransportAvailabilityV1::Available,
        Err(error) if error.kind() == io::ErrorKind::NotFound => {
            PrivateTransportAvailabilityV1::Missing
        }
        Err(error) if error.kind() == io::ErrorKind::ConnectionRefused => {
            PrivateTransportAvailabilityV1::Refused
        }
        Err(_) => PrivateTransportAvailabilityV1::Failed,
    }
}

#[cfg(target_os = "linux")]
fn private_transport_endpoint_metadata_availability(
    path: &Path,
    expected_uid: u32,
) -> PrivateTransportAvailabilityV1 {
    let metadata = match fs::symlink_metadata(path) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == io::ErrorKind::NotFound => {
            return PrivateTransportAvailabilityV1::Missing;
        }
        Err(_) => return PrivateTransportAvailabilityV1::Failed,
    };
    if metadata.file_type().is_symlink()
        || !metadata.file_type().is_socket()
        || metadata.uid() != expected_uid
        || metadata.permissions().mode() & 0o777 != 0o600
    {
        return PrivateTransportAvailabilityV1::Stale;
    }
    PrivateTransportAvailabilityV1::Available
}

#[cfg(target_os = "linux")]
fn private_episode_endpoint_identity(path: &Path) -> Result<PrivateEpisodeEndpointIdentityV1> {
    let metadata = fs::symlink_metadata(path).with_context(|| {
        format!(
            "failed to inspect private episode endpoint {}",
            path.display()
        )
    })?;
    if metadata.file_type().is_symlink() || !metadata.file_type().is_socket() {
        anyhow::bail!(
            "private episode endpoint {} is not an exact Unix socket",
            path.display()
        );
    }
    Ok(PrivateEpisodeEndpointIdentityV1 {
        device: metadata.dev(),
        inode: metadata.ino(),
    })
}

#[cfg(target_os = "linux")]
fn remove_private_episode_endpoint_if_same(
    path: &Path,
    expected: PrivateEpisodeEndpointIdentityV1,
) -> Result<()> {
    let current = match private_episode_endpoint_identity(path) {
        Ok(current) => current,
        Err(error)
            if error
                .downcast_ref::<io::Error>()
                .is_some_and(|error| error.kind() == io::ErrorKind::NotFound) =>
        {
            return Ok(())
        }
        Err(error) => return Err(error),
    };
    if current != expected {
        anyhow::bail!(
            "private episode endpoint {} was replaced by a competing publisher",
            path.display()
        );
    }
    fs::remove_file(path).with_context(|| {
        format!(
            "failed to remove private episode endpoint {}",
            path.display()
        )
    })
}

#[cfg(target_os = "linux")]
fn ensure_private_episode_owned_directory(path: &Path, expected_uid: u32) -> Result<()> {
    match fs::symlink_metadata(path) {
        Ok(metadata) => {
            if metadata.file_type().is_symlink()
                || !metadata.file_type().is_dir()
                || metadata.uid() != expected_uid
                || metadata.permissions().mode() & 0o777 != 0o700
            {
                anyhow::bail!(
                    "private episode directory {} has unsafe type, ownership, or permissions",
                    path.display()
                );
            }
        }
        Err(error) if error.kind() == io::ErrorKind::NotFound => {
            use std::os::unix::fs::DirBuilderExt;
            fs::DirBuilder::new()
                .mode(0o700)
                .create(path)
                .with_context(|| format!("failed to create {}", path.display()))?;
            let metadata = fs::symlink_metadata(path)
                .with_context(|| format!("failed to revalidate {}", path.display()))?;
            if metadata.file_type().is_symlink()
                || !metadata.file_type().is_dir()
                || metadata.uid() != expected_uid
                || metadata.permissions().mode() & 0o777 != 0o700
            {
                anyhow::bail!(
                    "created private episode directory {} changed identity",
                    path.display()
                );
            }
        }
        Err(error) => {
            return Err(error).with_context(|| format!("failed to inspect {}", path.display()));
        }
    }
    Ok(())
}

#[cfg(target_os = "linux")]
fn private_episode_namespace_root(
    path: &Path,
    substrate_home: &Path,
    expected_uid: u32,
) -> Result<PathBuf> {
    if path.starts_with(substrate_home) {
        return Ok(substrate_home.to_path_buf());
    }
    let fallback = PathBuf::from(format!("/tmp/substrate-host-episodes-u{expected_uid}"));
    if path.starts_with(&fallback) {
        return Ok(fallback);
    }
    anyhow::bail!(
        "private episode endpoint {} is outside its exact HSA store namespace",
        path.display()
    )
}

#[cfg(target_os = "linux")]
fn prepare_private_episode_endpoint_for_bind(
    path: &Path,
    expected_uid: u32,
    trusted_namespace_root: &Path,
) -> Result<()> {
    let parent = path.parent().ok_or_else(|| {
        anyhow::anyhow!(
            "private episode transport path '{}' is missing a parent directory",
            path.display()
        )
    })?;
    let relative_parent = parent.strip_prefix(trusted_namespace_root).map_err(|_| {
        anyhow::anyhow!(
            "private episode endpoint {} escapes trusted namespace {}",
            path.display(),
            trusted_namespace_root.display()
        )
    })?;
    if relative_parent.as_os_str().is_empty()
        || relative_parent
            .components()
            .any(|component| !matches!(component, std::path::Component::Normal(_)))
    {
        anyhow::bail!(
            "private episode endpoint {} has an invalid namespace-relative parent",
            path.display()
        );
    }

    ensure_private_episode_owned_directory(trusted_namespace_root, expected_uid)?;
    let mut current = trusted_namespace_root.to_path_buf();
    for component in relative_parent.components() {
        let std::path::Component::Normal(component) = component else {
            unreachable!("private episode parent components were validated")
        };
        current.push(component);
        ensure_private_episode_owned_directory(&current, expected_uid)?;
    }

    let metadata = match fs::symlink_metadata(path) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(()),
        Err(error) => {
            return Err(error).with_context(|| {
                format!(
                    "failed to inspect private episode transport {}",
                    path.display()
                )
            });
        }
    };
    if metadata.file_type().is_symlink()
        || !metadata.file_type().is_socket()
        || metadata.uid() != expected_uid
        || metadata.permissions().mode() & 0o777 != 0o600
    {
        anyhow::bail!(
            "private episode transport {} has unsafe type, ownership, or permissions",
            path.display()
        );
    }
    let stale_identity = PrivateEpisodeEndpointIdentityV1 {
        device: metadata.dev(),
        inode: metadata.ino(),
    };
    match StdUnixStream::connect(path) {
        Ok(_) => anyhow::bail!(
            "private episode transport {} is owned by an active competing publisher",
            path.display()
        ),
        Err(error) if error.kind() == io::ErrorKind::ConnectionRefused => {
            remove_private_episode_endpoint_if_same(path, stale_identity)
        }
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(error).with_context(|| {
            format!(
                "failed to classify existing private episode transport {}",
                path.display()
            )
        }),
    }
}

#[cfg(target_os = "linux")]
pub(crate) fn discover_toolbox_episode_transport(
    orchestration_session_id: &str,
) -> Result<HostExecutionEpisodeTransportProjectionV1> {
    let substrate_home = substrate_paths::substrate_home()?;
    let authority = HostSessionAuthority::from_trusted_root(
        TrustedAuthorityRoot::open(&substrate_home)
            .map_err(|error| anyhow::anyhow!(error.to_string()))?,
    )
    .map_err(|error| anyhow::anyhow!(error.to_string()))?;
    let current = authority
        .resolve_current_exact(orchestration_session_id, None)
        .map_err(|error| anyhow::anyhow!(error.to_string()))?;
    let (mut episode, binding) = host_execution_episode_for_authority(
        &current,
        HostExecutionEpisodeKindV1::RuntimeToolboxEpisode,
        current.caller.participant_id.as_str(),
        None,
        PrivateTransportAvailabilityV1::Failed,
    )?;
    let uid = unsafe { libc::geteuid() };
    let endpoint_path = toolbox_transport_path_for_episode(&substrate_home, uid, &binding);
    let availability = private_transport_availability(&endpoint_path, uid);
    episode.transport_status = classify_episode_transport(availability, true);
    episode.validate_current_authority(&binding, &current)?;
    let (world_id, world_generation) = current
        .authority
        .world_binding
        .as_ref()
        .map(|binding| {
            (
                Some(binding.world_id.clone()),
                Some(binding.world_generation),
            )
        })
        .unwrap_or((None, None));
    Ok(HostExecutionEpisodeTransportProjectionV1 {
        episode,
        binding,
        availability,
        endpoint_path,
        agent_id: current.caller.descriptor.agent_id,
        backend_id: current.caller.descriptor.backend_id,
        world_id,
        world_generation,
    })
}

#[cfg(target_os = "linux")]
pub(crate) fn validate_host_execution_episode_against_current_hsa(
    episode: &HostExecutionEpisodeV1,
    binding: &HostExecutionEpisodeBindingV1,
) -> Result<()> {
    let authority = HostSessionAuthority::from_trusted_root(
        TrustedAuthorityRoot::open(&substrate_paths::substrate_home()?)
            .map_err(|error| anyhow::anyhow!(error.to_string()))?,
    )
    .map_err(|error| anyhow::anyhow!(error.to_string()))?;
    let current = authority
        .resolve_current_exact(&binding.orchestration_session_id, None)
        .map_err(|error| anyhow::anyhow!(error.to_string()))?;
    episode.validate_current_authority(binding, &current)
}

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

pub(crate) fn verify_public_start_continuity_settled(
    orchestration_session_id: &str,
) -> Result<HostSessionPostureV1> {
    let substrate_home = substrate_paths::substrate_home()?;
    let authority = HostSessionAuthority::from_trusted_root(
        TrustedAuthorityRoot::open(&substrate_home)
            .map_err(|error| anyhow::anyhow!(error.to_string()))?,
    )
    .map_err(|error| anyhow::anyhow!(error.to_string()))?;
    let resolved = authority
        .resolve_current_exact(orchestration_session_id, None)
        .map_err(|error| anyhow::anyhow!(error.to_string()))?;
    let start_handles = resolved
        .authority
        .internal_resume_handle_refs
        .iter()
        .filter(|reference| reference.schema_version == 2)
        .count();
    if resolved.authority.authority_revision < 3
        || start_handles != 2
        || !matches!(
            resolved.authority.lifecycle_posture,
            HostSessionPostureV1::ParkedResumable
                | HostSessionPostureV1::AwaitingAttention
                | HostSessionPostureV1::Terminal
        )
    {
        anyhow::bail!(
            "public Start has not durably registered and settled its inaugural continuation"
        );
    }
    Ok(resolved.authority.lifecycle_posture)
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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub request_key_sha256: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub prompt_sha256: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub public_backend_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub public_scope: Option<AgentExecutionScope>,
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

#[cfg(target_os = "linux")]
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct AuthorityManagedSuccessorLaunchPlanV1 {
    pub schema_version: u32,
    pub authority_store_id: String,
    pub helper_plan: HiddenOwnerHelperLaunchPlan,
    pub applied_transition: HostSessionTransitionIntentV3,
}

#[cfg(target_os = "linux")]
impl AuthorityManagedSuccessorLaunchPlanV1 {
    fn validate_transport_shape(&self) -> Result<()> {
        if self.schema_version != 1
            || self.authority_store_id.is_empty()
            || self.applied_transition.workspace_binding.authority_store_id
                != self.authority_store_id
            || !matches!(
                self.helper_plan.mode,
                OwnerHelperMode::Attach | OwnerHelperMode::ResumeOneTurn
            )
            || self.applied_transition.mode
                != match self.helper_plan.mode {
                    OwnerHelperMode::Attach => crate::execution::agent_runtime::host_session_authority::schema::HostSessionTransitionModeV1::Attach,
                    OwnerHelperMode::ResumeOneTurn => crate::execution::agent_runtime::host_session_authority::schema::HostSessionTransitionModeV1::ResumeOneTurn,
                    OwnerHelperMode::Start => unreachable!(),
                }
            || self.applied_transition.orchestration_session_id
                != self.helper_plan.session.orchestration_session_id
            || self.applied_transition.shell_trace_session_id
                != self.helper_plan.session.shell_trace_session_id
            || self.applied_transition.target_authoritative_participant_id
                != self.helper_plan.participant.participant_id
            || self.applied_transition.run_id != self.helper_plan.participant.run_id
            || !matches!(
                self.applied_transition.state,
                HostSessionTransitionIntentStateV3::Applied { .. }
            )
        {
            anyhow::bail!("authority-managed successor launch envelope is inconsistent");
        }
        Ok(())
    }
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

#[cfg(target_os = "linux")]
pub(crate) struct AuthorityManagedSuccessorLaunchReceiptV1 {
    pub launch_receipt: HiddenOwnerHelperLaunchReceipt,
    _launch_guard: Option<AuthorityManagedSuccessorLaunchGuardV1>,
}

#[cfg(target_os = "linux")]
struct AuthorityManagedSuccessorLaunchGuardV1 {
    lock_file: File,
}

#[cfg(target_os = "linux")]
pub(crate) struct AuthorityManagedSuccessorLaunchPermitV1 {
    authority_store_id: String,
    intent_id: String,
    launch_guard: Option<AuthorityManagedSuccessorLaunchGuardV1>,
}

#[cfg(target_os = "linux")]
impl AuthorityManagedSuccessorLaunchPermitV1 {
    pub(crate) fn joined_completed_launch(&self) -> bool {
        self.launch_guard.is_none()
    }
}

#[cfg(target_os = "linux")]
enum AuthorityManagedSuccessorLaunchJoinV1 {
    Joined,
    RetryAsLeader(AuthorityManagedSuccessorLaunchGuardV1),
}

#[cfg(all(target_os = "linux", test))]
thread_local! {
    static AUTHORITY_SUCCESSOR_COMPLETION_RACE_HOOK: std::cell::RefCell<Option<Box<dyn FnOnce()>>> =
        std::cell::RefCell::new(None);
}

#[cfg(all(target_os = "linux", test))]
pub(crate) fn inject_authority_successor_completion_race_for_test(hook: impl FnOnce() + 'static) {
    AUTHORITY_SUCCESSOR_COMPLETION_RACE_HOOK.with(|slot| {
        *slot.borrow_mut() = Some(Box::new(hook));
    });
}

#[cfg(all(target_os = "linux", test))]
fn maybe_inject_authority_successor_completion_race_for_test() {
    AUTHORITY_SUCCESSOR_COMPLETION_RACE_HOOK.with(|slot| {
        if let Some(hook) = slot.borrow_mut().take() {
            hook();
        }
    });
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
    #[cfg(target_os = "linux")]
    let (mut launch_episode, launch_binding) = host_execution_episode_for_helper_plan(plan)?;
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
    #[cfg(target_os = "linux")]
    {
        launch_episode.record_local_observation(
            &launch_binding,
            HostExecutionEpisodeObservationV1::Process(ProcessRefV1 { pid: child.id() }),
        )?;
    }
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

#[cfg(target_os = "linux")]
fn append_checked_install_bootstrap_context_arg(command: &mut Command) -> Result<()> {
    let carrier =
        crate::execution::install_bootstrap::checked_install_bootstrap_context_from_projections()
            .context(
            "failed to obtain authenticated install bootstrap context for owner-helper launch",
        )?;
    let encoded = carrier.encode().context(
        "failed to encode authenticated install bootstrap context for owner-helper launch",
    )?;
    command.arg("--install-bootstrap-context-v1").arg(encoded);
    Ok(())
}

/// Starts the real public Start exchange without consulting the legacy
/// compatibility projection for readiness. The startup backchannel and HSA
/// settlement are the caller's completion authority.
#[cfg(unix)]
pub(crate) fn launch_hidden_owner_helper_for_durable_start(
    plan: &HiddenOwnerHelperLaunchPlan,
    world: bool,
    no_world: bool,
) -> Result<HiddenOwnerHelperLaunchReceipt> {
    if plan.mode != OwnerHelperMode::Start || plan.startup_prompt.is_none() {
        anyhow::bail!(
            "durable Start launcher requires a Start plan with the real prompt backchannel"
        );
    }

    #[cfg(target_os = "linux")]
    let (mut launch_episode, launch_binding) = host_execution_episode_for_helper_plan(plan)?;

    let plan_path = persist_durable_start_launch_plan(plan)?;
    let exe = env::current_exe()
        .context("failed to resolve current substrate executable for hidden owner-helper launch")?;
    let mut command = Command::new(exe);
    #[cfg(target_os = "linux")]
    append_checked_install_bootstrap_context_arg(&mut command)?;
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

    let child = command.spawn().with_context(|| {
        format!(
            "failed to spawn hidden owner-helper for orchestration session {}",
            plan.session.orchestration_session_id
        )
    })?;
    #[cfg(target_os = "linux")]
    {
        launch_episode.record_local_observation(
            &launch_binding,
            HostExecutionEpisodeObservationV1::Process(ProcessRefV1 { pid: child.id() }),
        )?;
    }

    Ok(HiddenOwnerHelperLaunchReceipt {
        helper_pid: child.id(),
        orchestration_session_id: plan.session.orchestration_session_id.clone(),
        participant_id: plan.participant.participant_id.clone(),
        backend_id: plan.descriptor.backend_id.clone(),
    })
}

/// Launches an already-applied HSA Attach or ResumeOneTurn without consulting
/// legacy readiness or compatibility state. Durable HSA reconciliation is the
/// caller's only completion authority.
#[cfg(target_os = "linux")]
pub(crate) fn launch_authority_managed_successor_owner_helper(
    plan: &AuthorityManagedSuccessorLaunchPlanV1,
    permit: AuthorityManagedSuccessorLaunchPermitV1,
    world: bool,
    no_world: bool,
) -> Result<AuthorityManagedSuccessorLaunchReceiptV1> {
    plan.validate_transport_shape()?;
    let (mut launch_episode, launch_binding) =
        host_execution_episode_for_helper_plan(&plan.helper_plan)?;
    if permit.authority_store_id != plan.authority_store_id
        || permit.intent_id != plan.applied_transition.intent_id
    {
        anyhow::bail!("authority-managed successor launch permit identity was substituted");
    }
    let Some(launch_guard) = permit.launch_guard else {
        if authority_managed_successor_completion(plan)?.is_none() {
            anyhow::bail!("authority-managed successor joined launch lost its HSA completion");
        }
        return Ok(AuthorityManagedSuccessorLaunchReceiptV1 {
            launch_receipt: authority_managed_successor_joined_receipt(plan),
            _launch_guard: None,
        });
    };
    let plan_path = persist_authority_managed_successor_launch_plan(plan)?;
    let exe = env::current_exe().context(
        "failed to resolve current substrate executable for authority-managed owner-helper launch",
    )?;
    let mut command = Command::new(exe);
    append_checked_install_bootstrap_context_arg(&mut command)?;
    if world {
        command.arg("--world");
    } else if no_world {
        command.arg("--no-world");
    }
    command
        .args(["agent", HIDDEN_OWNER_HELPER_SUBCOMMAND, "--plan-file"])
        .arg(&plan_path)
        .current_dir(&plan.helper_plan.session.workspace_root)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    let launch_guard_fd = launch_guard.lock_file.as_raw_fd();
    command.env(
        AUTHORITY_SUCCESSOR_LAUNCH_GUARD_FD_ENV,
        launch_guard_fd.to_string(),
    );
    // Clear close-on-exec only in the forked child. Toggling the parent descriptor
    // would let an unrelated concurrent spawn inherit and prolong this guard.
    unsafe {
        command.pre_exec(move || {
            let flags = libc::fcntl(launch_guard_fd, libc::F_GETFD);
            if flags < 0 {
                return Err(io::Error::last_os_error());
            }
            if libc::fcntl(launch_guard_fd, libc::F_SETFD, flags & !libc::FD_CLOEXEC) < 0 {
                return Err(io::Error::last_os_error());
            }
            Ok(())
        });
    }
    let child = command.spawn().with_context(|| {
        format!(
            "failed to spawn authority-managed owner-helper for orchestration session {}",
            plan.helper_plan.session.orchestration_session_id
        )
    })?;
    launch_episode.record_local_observation(
        &launch_binding,
        HostExecutionEpisodeObservationV1::Process(ProcessRefV1 { pid: child.id() }),
    )?;

    Ok(AuthorityManagedSuccessorLaunchReceiptV1 {
        launch_receipt: HiddenOwnerHelperLaunchReceipt {
            helper_pid: child.id(),
            orchestration_session_id: plan.helper_plan.session.orchestration_session_id.clone(),
            participant_id: plan.helper_plan.participant.participant_id.clone(),
            backend_id: plan.helper_plan.descriptor.backend_id.clone(),
        },
        _launch_guard: Some(launch_guard),
    })
}

#[cfg(target_os = "linux")]
pub(crate) fn acquire_authority_managed_successor_launch_permit(
    plan: &AuthorityManagedSuccessorLaunchPlanV1,
) -> Result<AuthorityManagedSuccessorLaunchPermitV1> {
    plan.validate_transport_shape()?;
    if authority_managed_successor_completion(plan)?.is_some() {
        return Ok(AuthorityManagedSuccessorLaunchPermitV1 {
            authority_store_id: plan.authority_store_id.clone(),
            intent_id: plan.applied_transition.intent_id.clone(),
            launch_guard: None,
        });
    }
    let launch_guard = match try_acquire_authority_managed_successor_launch_guard(plan)? {
        Some(guard) => {
            #[cfg(test)]
            maybe_inject_authority_successor_completion_race_for_test();
            if authority_managed_successor_completion(plan)?.is_some() {
                drop(guard);
                None
            } else {
                Some(guard)
            }
        }
        None => match wait_for_inflight_authority_managed_successor_launch(plan)? {
            AuthorityManagedSuccessorLaunchJoinV1::Joined => None,
            AuthorityManagedSuccessorLaunchJoinV1::RetryAsLeader(guard) => Some(guard),
        },
    };
    Ok(AuthorityManagedSuccessorLaunchPermitV1 {
        authority_store_id: plan.authority_store_id.clone(),
        intent_id: plan.applied_transition.intent_id.clone(),
        launch_guard,
    })
}

#[cfg(target_os = "linux")]
fn try_acquire_authority_managed_successor_launch_guard(
    plan: &AuthorityManagedSuccessorLaunchPlanV1,
) -> Result<Option<AuthorityManagedSuccessorLaunchGuardV1>> {
    let intent_id = &plan.applied_transition.intent_id;
    let path = authority_managed_successor_launch_guard_path(plan)?;
    let parent = path.parent().ok_or_else(|| {
        anyhow::anyhow!(
            "authority-managed successor launch guard '{}' is missing a parent directory",
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
                "failed to open authority-managed successor launch guard {}",
                path.display()
            )
        })?;
    match file.try_lock_exclusive() {
        Ok(()) => Ok(Some(AuthorityManagedSuccessorLaunchGuardV1 {
            lock_file: file,
        })),
        Err(err) if err.kind() == io::ErrorKind::WouldBlock => Ok(None),
        Err(err) => Err(anyhow::Error::new(err).context(format!(
            "failed to coordinate authority-managed successor launch for intent {intent_id}"
        ))),
    }
}

#[cfg(target_os = "linux")]
fn wait_for_inflight_authority_managed_successor_launch(
    plan: &AuthorityManagedSuccessorLaunchPlanV1,
) -> Result<AuthorityManagedSuccessorLaunchJoinV1> {
    let started_at = Instant::now();
    loop {
        if authority_managed_successor_completion(plan)?.is_some() {
            return Ok(AuthorityManagedSuccessorLaunchJoinV1::Joined);
        }
        if let Some(guard) = try_acquire_authority_managed_successor_launch_guard(plan)? {
            if authority_managed_successor_completion(plan)?.is_some() {
                drop(guard);
                return Ok(AuthorityManagedSuccessorLaunchJoinV1::Joined);
            }
            return Ok(AuthorityManagedSuccessorLaunchJoinV1::RetryAsLeader(guard));
        }
        if started_at.elapsed() >= OWNER_HELPER_READY_TIMEOUT {
            anyhow::bail!(
                "authority-managed successor launch leader did not settle exact HSA evidence"
            );
        }
        thread::sleep(OWNER_HELPER_READY_POLL_INTERVAL);
    }
}

#[cfg(target_os = "linux")]
fn authority_managed_successor_joined_receipt(
    plan: &AuthorityManagedSuccessorLaunchPlanV1,
) -> HiddenOwnerHelperLaunchReceipt {
    HiddenOwnerHelperLaunchReceipt {
        helper_pid: 0,
        orchestration_session_id: plan.helper_plan.session.orchestration_session_id.clone(),
        participant_id: plan.helper_plan.participant.participant_id.clone(),
        backend_id: plan.helper_plan.descriptor.backend_id.clone(),
    }
}

#[cfg(target_os = "linux")]
fn persist_authority_managed_successor_launch_plan(
    plan: &AuthorityManagedSuccessorLaunchPlanV1,
) -> Result<PathBuf> {
    plan.validate_transport_shape()?;
    let path = durable_start_control_root()?
        .join("successor-plans")
        .join(format!("{}.json", plan.applied_transition.intent_id));
    let parent = path.parent().ok_or_else(|| {
        anyhow::anyhow!(
            "authority-managed successor plan path '{}' is missing a parent directory",
            path.display()
        )
    })?;
    fs::create_dir_all(parent).with_context(|| format!("failed to create {}", parent.display()))?;
    fs::write(&path, serde_json::to_vec_pretty(plan)?).with_context(|| {
        format!(
            "failed to write authority-managed successor plan {}",
            path.display()
        )
    })?;
    Ok(path)
}

#[cfg(target_os = "linux")]
pub(crate) fn load_authority_managed_successor_launch_plan(
    path: &Path,
) -> Result<AuthorityManagedSuccessorLaunchPlanV1> {
    let bytes = fs::read(path).with_context(|| {
        format!(
            "failed to read authority-managed successor launch plan {}",
            path.display()
        )
    })?;
    let plan: AuthorityManagedSuccessorLaunchPlanV1 =
        serde_json::from_slice(&bytes).with_context(|| {
            format!(
                "failed to decode authority-managed successor launch plan {}",
                path.display()
            )
        })?;
    plan.validate_transport_shape()?;
    validate_inherited_authority_successor_launch_guard(&plan)?;
    Ok(plan)
}

#[cfg(target_os = "linux")]
fn authority_managed_successor_launch_guard_path(
    plan: &AuthorityManagedSuccessorLaunchPlanV1,
) -> Result<PathBuf> {
    let intent_id = &plan.applied_transition.intent_id;
    if intent_id.is_empty()
        || !intent_id
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_'))
    {
        anyhow::bail!("authority-managed successor intent cannot identify a launch guard");
    }
    Ok(Path::new(
        &plan
            .applied_transition
            .workspace_binding
            .authority_store_root
            .physical_path,
    )
    .join("runtime-control")
    .join("durable-start")
    .join("successor-launch-guards")
    .join(format!("{intent_id}.lock")))
}

#[cfg(target_os = "linux")]
fn set_authority_successor_launch_guard_close_on_exec(fd: i32, enabled: bool) -> Result<()> {
    let flags = unsafe { libc::fcntl(fd, libc::F_GETFD) };
    if flags < 0 {
        return Err(io::Error::last_os_error())
            .context("failed to read authority successor launch-guard descriptor flags");
    }
    let next = if enabled {
        flags | libc::FD_CLOEXEC
    } else {
        flags & !libc::FD_CLOEXEC
    };
    if unsafe { libc::fcntl(fd, libc::F_SETFD, next) } < 0 {
        return Err(io::Error::last_os_error())
            .context("failed to set authority successor launch-guard descriptor flags");
    }
    Ok(())
}

#[cfg(target_os = "linux")]
fn validate_inherited_authority_successor_launch_guard(
    plan: &AuthorityManagedSuccessorLaunchPlanV1,
) -> Result<()> {
    let fd = env::var(AUTHORITY_SUCCESSOR_LAUNCH_GUARD_FD_ENV)
        .context("authority-managed successor helper is missing its inherited launch guard")?
        .parse::<i32>()
        .context("authority-managed successor helper launch guard is malformed")?;
    if fd < 0 {
        anyhow::bail!("authority-managed successor helper launch guard is invalid");
    }
    let expected_path = authority_managed_successor_launch_guard_path(plan)?;
    let expected = fs::metadata(&expected_path).with_context(|| {
        format!(
            "failed to inspect authority successor launch guard {}",
            expected_path.display()
        )
    })?;
    let inherited = fs::metadata(format!("/proc/self/fd/{fd}"))
        .context("failed to inspect inherited authority successor launch guard")?;
    if !expected.is_file() || expected.dev() != inherited.dev() || expected.ino() != inherited.ino()
    {
        anyhow::bail!("authority-managed successor helper launch guard identity was substituted");
    }
    set_authority_successor_launch_guard_close_on_exec(fd, true)
}

#[cfg(target_os = "linux")]
pub(crate) fn wait_for_authority_managed_successor_completion(
    plan: &AuthorityManagedSuccessorLaunchPlanV1,
) -> Result<HostSessionPostureV1> {
    plan.validate_transport_shape()?;
    let started_at = Instant::now();
    loop {
        if let Some(posture) = authority_managed_successor_completion(plan)? {
            return Ok(posture);
        }
        if started_at.elapsed() >= OWNER_HELPER_READY_TIMEOUT {
            anyhow::bail!(
                "authority-managed successor remains durably pending exact actor-event reconciliation"
            );
        }
        thread::sleep(OWNER_HELPER_READY_POLL_INTERVAL);
    }
}

#[cfg(target_os = "linux")]
fn authority_managed_successor_completion(
    plan: &AuthorityManagedSuccessorLaunchPlanV1,
) -> Result<Option<HostSessionPostureV1>> {
    plan.validate_transport_shape()?;
    let substrate_home = Path::new(
        &plan
            .applied_transition
            .workspace_binding
            .authority_store_root
            .physical_path,
    );
    let authority = HostSessionAuthority::from_trusted_root(
        TrustedAuthorityRoot::open(substrate_home)
            .map_err(|error| anyhow::anyhow!(error.to_string()))?,
    )
    .map_err(|error| anyhow::anyhow!(error.to_string()))?;
    let root = authority
        .read_a12b_root()
        .map_err(|error| anyhow::anyhow!(error.to_string()))?;
    if root.authority_store_id != plan.authority_store_id {
        anyhow::bail!("authority-managed successor store identity changed");
    }
    let current = root
        .successor_transition_intent_map
        .get(&plan.applied_transition.intent_id)
        .ok_or_else(|| anyhow::anyhow!("authority-managed successor intent disappeared"))?;
    verify_authority_managed_successor_identity(&plan.applied_transition, current)?;
    let HostSessionTransitionIntentStateV3::Applied {
        startup_ownership,
        post_turn,
        ..
    } = &current.state
    else {
        anyhow::bail!("authority-managed successor is no longer durably Applied");
    };
    let resolved = authority
        .resolve_current_exact(&current.orchestration_session_id, None)
        .map_err(|error| anyhow::anyhow!(error.to_string()))?;
    if resolved.observation.authority_store_id != plan.authority_store_id
        || resolved
            .authority
            .active_authoritative_participant_id
            .as_deref()
            != Some(current.target_authoritative_participant_id.as_str())
    {
        anyhow::bail!("authority-managed successor current authority identity changed");
    }
    Ok(match current.mode {
            crate::execution::agent_runtime::host_session_authority::schema::HostSessionTransitionModeV1::Attach => {
                match startup_ownership.as_ref() {
                    HostSessionStartupOwnershipApplicationV1::Accepted { .. } => {
                        Some(resolved.authority.lifecycle_posture)
                    }
                    HostSessionStartupOwnershipApplicationV1::TerminalReconciled { .. } => {
                        anyhow::bail!("authority-managed Attach reconciled to terminal failure")
                    }
                    HostSessionStartupOwnershipApplicationV1::Pending { .. } => None,
                    HostSessionStartupOwnershipApplicationV1::NotApplicable => {
                        anyhow::bail!("authority-managed Attach lost startup ownership state")
                    }
                }
            }
            crate::execution::agent_runtime::host_session_authority::schema::HostSessionTransitionModeV1::ResumeOneTurn => {
                match post_turn.as_ref() {
                    HostSessionPostTurnApplicationV2::Applied {
                        resulting_posture, ..
                    } if *resulting_posture == resolved.authority.lifecycle_posture => {
                        Some(*resulting_posture)
                    }
                    HostSessionPostTurnApplicationV2::Applied { .. } => {
                        anyhow::bail!("authority-managed ResumeOneTurn posture changed after reconciliation")
                    }
                    HostSessionPostTurnApplicationV2::Pending { .. }
                    | HostSessionPostTurnApplicationV2::AwaitingObligationCut { .. } => None,
                    HostSessionPostTurnApplicationV2::NotApplicable => {
                        anyhow::bail!("authority-managed ResumeOneTurn lost post-turn state")
                    }
                }
            }
            crate::execution::agent_runtime::host_session_authority::schema::HostSessionTransitionModeV1::Start => {
                anyhow::bail!("authority-managed successor cannot use Start")
            }
    })
}

#[cfg(target_os = "linux")]
fn verify_authority_managed_successor_identity(
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
        anyhow::bail!("authority-managed successor launch identity was substituted");
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
        anyhow::bail!("authority-managed successor launch application identity changed");
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
        anyhow::bail!("authority-managed successor launch application identity was substituted");
    }
    Ok(())
}

#[cfg(unix)]
fn persist_durable_start_launch_plan(plan: &HiddenOwnerHelperLaunchPlan) -> Result<PathBuf> {
    let path = durable_start_launch_plan_path(plan)?;
    let parent = path.parent().ok_or_else(|| {
        anyhow::anyhow!(
            "durable Start launch plan path '{}' is missing a parent directory",
            path.display()
        )
    })?;
    fs::create_dir_all(parent).with_context(|| format!("failed to create {}", parent.display()))?;
    fs::write(&path, serde_json::to_vec_pretty(plan)?).with_context(|| {
        format!(
            "failed to write durable Start launch plan {}",
            path.display()
        )
    })?;
    Ok(path)
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
    pub(crate) payload: PrivateStopRequestPayloadV1,
    pub response_tx: oneshot::Sender<PrivateStopOutcome>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum PrivateStopRequestPayloadV1 {
    Legacy,
    AuthorityManaged(Box<HostSessionStopDeliveryV1>),
}

#[cfg(unix)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct AuthorityManagedStopCloseoutV1 {
    delivery: HostSessionStopDeliveryV1,
    acceptance_id: String,
}

#[cfg(unix)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum AuthorityManagedStopAcceptanceV1 {
    Accepted(Box<AuthorityManagedStopCloseoutV1>),
    AlreadyTerminal,
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
    #[cfg(target_os = "linux")]
    endpoint_identity: Option<PrivateEpisodeEndpointIdentityV1>,
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
    #[cfg(target_os = "linux")]
    endpoint_identity: Option<PrivateEpisodeEndpointIdentityV1>,
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
        #[cfg(target_os = "linux")]
        {
            if let Some(identity) = self.endpoint_identity.take() {
                let _ = remove_private_episode_endpoint_if_same(&self.path, identity);
            } else {
                let _ = tokio::fs::remove_file(&self.path).await;
            }
        }
        #[cfg(all(unix, not(target_os = "linux")))]
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
        #[cfg(target_os = "linux")]
        {
            if let Some(identity) = self.endpoint_identity.take() {
                let _ = remove_private_episode_endpoint_if_same(&self.path, identity);
            } else {
                let _ = tokio::fs::remove_file(&self.path).await;
            }
        }
        #[cfg(all(unix, not(target_os = "linux")))]
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

#[cfg(unix)]
fn durable_start_control_root() -> Result<PathBuf> {
    Ok(substrate_paths::substrate_home()?
        .join("runtime-control")
        .join("durable-start"))
}

#[cfg(unix)]
fn durable_start_startup_prompt_stream_path(
    orchestration_session_id: &str,
    participant_id: &str,
) -> Result<PathBuf> {
    let session_fragment = compact_stop_transport_fragment(orchestration_session_id);
    let participant_fragment = compact_stop_transport_fragment(participant_id);
    let socket_name = format!("{session_fragment}-{participant_fragment}.startup.sock");
    let preferred = durable_start_control_root()?
        .join("startup")
        .join(&socket_name);
    if preferred.as_os_str().len() > PRIVATE_STOP_UNIX_PATH_MAX {
        return Ok(PathBuf::from("/tmp")
            .join("substrate-durable-start")
            .join("startup")
            .join(socket_name));
    }
    Ok(preferred)
}

#[cfg(unix)]
fn durable_start_launch_plan_path(plan: &HiddenOwnerHelperLaunchPlan) -> Result<PathBuf> {
    let session_fragment = compact_stop_transport_fragment(plan.orchestration_session_id());
    let participant_fragment = compact_stop_transport_fragment(plan.participant_id());
    Ok(durable_start_control_root()?
        .join("owner-helper")
        .join(format!("{session_fragment}-{participant_fragment}.json")))
}

#[cfg(unix)]
fn durable_start_private_transport_path(
    kind: &str,
    suffix: &str,
    orchestration_session_id: &str,
    participant_id: &str,
) -> Result<PathBuf> {
    #[cfg(target_os = "linux")]
    {
        let substrate_home = substrate_paths::substrate_home()?;
        let (_, binding) = durable_private_transport_episode(
            orchestration_session_id,
            participant_id,
            PrivateTransportAvailabilityV1::Missing,
        )?;
        let uid = unsafe { libc::geteuid() };
        Ok(private_episode_transport_path_for_binding(
            &substrate_home,
            uid,
            &binding,
            kind,
            suffix,
        ))
    }

    #[cfg(not(target_os = "linux"))]
    {
        let session_fragment = compact_stop_transport_fragment(orchestration_session_id);
        let participant_fragment = compact_stop_transport_fragment(participant_id);
        let socket_name = format!("{session_fragment}-{participant_fragment}.{suffix}.sock");
        let preferred = durable_start_control_root()?.join(kind).join(&socket_name);
        if preferred.as_os_str().len() > PRIVATE_STOP_UNIX_PATH_MAX {
            return Ok(PathBuf::from("/tmp")
                .join("substrate-durable-start")
                .join(kind)
                .join(socket_name));
        }
        Ok(preferred)
    }
}

#[cfg(target_os = "linux")]
fn durable_private_transport_episode(
    orchestration_session_id: &str,
    participant_id: &str,
    availability: PrivateTransportAvailabilityV1,
) -> Result<(HostExecutionEpisodeV1, HostExecutionEpisodeBindingV1)> {
    let authority = HostSessionAuthority::from_trusted_root(
        TrustedAuthorityRoot::open(&substrate_paths::substrate_home()?)
            .map_err(|error| anyhow::anyhow!(error.to_string()))?,
    )
    .map_err(|error| anyhow::anyhow!(error.to_string()))?;
    let current = authority
        .resolve_current_exact(orchestration_session_id, None)
        .map_err(|error| anyhow::anyhow!(error.to_string()))?;
    if current.caller.participant_id != participant_id {
        anyhow::bail!("durable private transport participant is not exact current HSA authority");
    }
    host_execution_episode_for_authority(
        &current,
        HostExecutionEpisodeKindV1::SyntheticOrRecoveredEpisode,
        &format!("durable-private-owner:{participant_id}"),
        Some(ProcessRefV1 {
            pid: std::process::id(),
        }),
        availability,
    )
}

#[cfg(target_os = "linux")]
fn private_episode_transport_path_for_binding(
    substrate_home: &Path,
    uid: u32,
    binding: &HostExecutionEpisodeBindingV1,
    kind: &str,
    suffix: &str,
) -> PathBuf {
    let home = substrate_home.as_os_str().as_encoded_bytes();
    let uid_bytes = uid.to_be_bytes();
    let revision_bytes = binding.authority_revision.to_be_bytes();
    let store_namespace = episode_path_digest(
        "substrate.host-execution-episode.private-store-path.v1",
        &[home, &uid_bytes, binding.authority_store_id.as_bytes()],
    );
    let endpoint_identity = episode_path_digest(
        "substrate.host-execution-episode.private-endpoint-path.v1",
        &[
            home,
            &uid_bytes,
            binding.authority_store_id.as_bytes(),
            binding.orchestration_session_id.as_bytes(),
            binding.participant_id.as_bytes(),
            binding.episode_id.as_bytes(),
            &revision_bytes,
            kind.as_bytes(),
            suffix.as_bytes(),
        ],
    );
    let socket_name = format!("{}.{}.sock", &endpoint_identity[..32], suffix);
    let suffix_path = PathBuf::from(format!("u{uid}"))
        .join(&store_namespace[..16])
        .join(kind)
        .join(socket_name);
    let preferred = substrate_home
        .join("run")
        .join("host-episodes")
        .join(&suffix_path);
    if preferred.as_os_str().len() <= PRIVATE_STOP_UNIX_PATH_MAX {
        return preferred;
    }
    PathBuf::from("/tmp")
        .join(format!("substrate-host-episodes-u{uid}"))
        .join(&store_namespace[..16])
        .join(kind)
        .join(format!("{}.{}.sock", &endpoint_identity[..32], suffix))
}

#[cfg(unix)]
pub(crate) fn durable_start_stop_transport_path(
    orchestration_session_id: &str,
    participant_id: &str,
) -> Result<PathBuf> {
    durable_start_private_transport_path("stop", "stop", orchestration_session_id, participant_id)
}

#[cfg(unix)]
pub(crate) fn durable_start_toolbox_transport_path(
    orchestration_session_id: &str,
) -> Result<PathBuf> {
    #[cfg(target_os = "linux")]
    {
        toolbox_transport_path(orchestration_session_id)
    }

    #[cfg(not(target_os = "linux"))]
    {
        let session_fragment = compact_stop_transport_fragment(orchestration_session_id);
        let socket_name = format!("{session_fragment}.sock");
        let preferred = durable_start_control_root()?
            .join("toolbox")
            .join(&socket_name);
        if preferred.as_os_str().len() > PRIVATE_STOP_UNIX_PATH_MAX {
            return Ok(PathBuf::from("/tmp")
                .join("substrate-durable-start")
                .join("toolbox")
                .join(socket_name));
        }
        Ok(preferred)
    }
}

pub(crate) fn toolbox_transport_path_for_home(
    substrate_home: &Path,
    orchestration_session_id: &str,
) -> PathBuf {
    #[cfg(target_os = "linux")]
    {
        #[cfg(unix)]
        let uid = unsafe { libc::geteuid() };
        #[cfg(not(unix))]
        let uid = 0;
        let authority_store_id = episode_path_digest(
            "substrate.host-execution-episode.legacy-store-path.v1",
            &[substrate_home.as_os_str().as_encoded_bytes()],
        );
        toolbox_transport_path_for_episode(
            substrate_home,
            uid,
            &HostExecutionEpisodeBindingV1 {
                authority_store_id,
                orchestration_session_id: orchestration_session_id.to_string(),
                participant_id: "legacy-participant-projection".to_string(),
                episode_id: "legacy-toolbox-projection".to_string(),
                authority_revision: 1,
            },
        )
    }

    #[cfg(not(target_os = "linux"))]
    {
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
}

pub(crate) fn toolbox_transport_path(orchestration_session_id: &str) -> Result<PathBuf> {
    #[cfg(target_os = "linux")]
    {
        let substrate_home = substrate_paths::substrate_home()?;
        let authority = HostSessionAuthority::from_trusted_root(
            TrustedAuthorityRoot::open(&substrate_home)
                .map_err(|error| anyhow::anyhow!(error.to_string()))?,
        )
        .map_err(|error| anyhow::anyhow!(error.to_string()))?;
        let current = authority
            .resolve_current_exact(orchestration_session_id, None)
            .map_err(|error| anyhow::anyhow!(error.to_string()))?;
        let (_, binding) = host_execution_episode_for_authority(
            &current,
            HostExecutionEpisodeKindV1::RuntimeToolboxEpisode,
            current.caller.participant_id.as_str(),
            None,
            PrivateTransportAvailabilityV1::Available,
        )?;
        let uid = unsafe { libc::geteuid() };
        Ok(toolbox_transport_path_for_episode(
            &substrate_home,
            uid,
            &binding,
        ))
    }

    #[cfg(not(target_os = "linux"))]
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

#[cfg(target_os = "linux")]
const _: fn(&AgentRuntimeStateStore, &str, &str) -> Result<StartupPromptTransportListener> =
    register_hidden_owner_helper_startup_prompt_listener;

#[cfg(unix)]
pub(crate) fn register_durable_start_startup_prompt_listener(
    orchestration_session_id: &str,
    participant_id: &str,
) -> Result<StartupPromptTransportListener> {
    let path = durable_start_startup_prompt_stream_path(orchestration_session_id, participant_id)?;
    let parent = path.parent().ok_or_else(|| {
        anyhow::anyhow!(
            "durable Start prompt transport path '{}' is missing a parent directory",
            path.display()
        )
    })?;
    fs::create_dir_all(parent).with_context(|| format!("failed to create {}", parent.display()))?;
    remove_existing_stop_transport_path(&path)?;
    let listener = StdUnixListener::bind(&path).with_context(|| {
        format!(
            "failed to bind durable Start prompt transport {}",
            path.display()
        )
    })?;
    listener.set_nonblocking(true).with_context(|| {
        format!(
            "failed to configure durable Start prompt transport {}",
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
            let terminal = matches!(
                envelope,
                PublicPromptEnvelope::Completed { .. } | PublicPromptEnvelope::Failed { .. }
            );
            let rewritten = rewrite_startup_prompt_envelope_action(
                envelope,
                action,
                backend_id_override,
                scope_override,
            );
            if terminal && action == PublicPromptAction::Start {
                renderer.render_start_terminal_strict(&rewritten)?;
            } else {
                renderer.render(&rewritten)?;
            }
            if terminal {
                saw_terminal = true;
            }
            Ok(())
        })
        .await
    });

    match result {
        Ok(0) => Ok(()),
        Ok(code) => Err(anyhow::Error::new(PublicPromptRenderedExit {
            exit_code: code,
        })),
        Err(_err) if saw_terminal => Err(anyhow::Error::new(PublicPromptRenderedExit {
            exit_code: 1,
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
pub(crate) fn render_committed_public_start_transaction(
    transaction: &StartTransactionRecordV1,
    json: bool,
) -> Result<i32> {
    let substrate_home = substrate_paths::substrate_home()?;
    let authority = HostSessionAuthority::from_trusted_root(
        TrustedAuthorityRoot::open(&substrate_home)
            .map_err(|error| anyhow::anyhow!(error.to_string()))?,
    )
    .map_err(|error| anyhow::anyhow!(error.to_string()))?;
    let committed = authority
        .committed_start_public_result(transaction)
        .map_err(|error| anyhow::anyhow!(error.to_string()))?;
    let resulting_posture = committed.resulting_posture;
    let session_posture = match resulting_posture {
        HostSessionPostureV1::ParkedResumable | HostSessionPostureV1::AwaitingAttention => {
            PublicSessionPosture::DetachedReattachable
        }
        HostSessionPostureV1::Terminal | HostSessionPostureV1::Invalid => {
            PublicSessionPosture::Terminal
        }
        HostSessionPostureV1::ActiveAttached
        | HostSessionPostureV1::DetachedReconciled
        | HostSessionPostureV1::StaleRecoverable => {
            anyhow::bail!("committed public Start settlement has an invalid response posture")
        }
    };
    let (envelope, exit_code) = match committed.completion_kind {
        StartTurnCompletionKindV1::ResumableClean | StartTurnCompletionKindV1::TerminalClean => (
            PublicPromptEnvelope::Completed {
                version: 1,
                action: PublicPromptAction::Start,
                orchestration_session_id: transaction.orchestration_session_id.clone(),
                backend_id: transaction.public_backend_id.clone(),
                participant_id: Some(transaction.authoritative_participant_id.clone()),
                turn_outcome: "success".to_string(),
                session_posture,
                state: match resulting_posture {
                    HostSessionPostureV1::ParkedResumable => "parked_resumable",
                    HostSessionPostureV1::AwaitingAttention => "awaiting_attention",
                    HostSessionPostureV1::Terminal => "terminal",
                    _ => unreachable!("settled public Start posture was checked above"),
                }
                .to_string(),
                warnings: Vec::new(),
            },
            0,
        ),
        StartTurnCompletionKindV1::TerminalFailure { reason } => (
            failed_prompt_envelope("runtime", "owner_unreachable", reason),
            1,
        ),
    };
    PublicPromptRenderer::new(json).render_start_terminal_strict(&envelope)?;
    Ok(exit_code)
}

#[cfg(unix)]
pub(crate) fn render_indeterminate_public_start_transaction(
    transaction: &StartTransactionRecordV1,
    json: bool,
) -> Result<i32> {
    let substrate_home = substrate_paths::substrate_home()?;
    let authority = HostSessionAuthority::from_trusted_root(
        TrustedAuthorityRoot::open(&substrate_home)
            .map_err(|error| anyhow::anyhow!(error.to_string()))?,
    )
    .map_err(|error| anyhow::anyhow!(error.to_string()))?;
    let current = authority
        .retryable_start_transaction(&transaction.request_key_sha256)
        .map_err(|error| anyhow::anyhow!(error.to_string()))?
        .ok_or_else(|| anyhow::anyhow!("durable public Start transaction disappeared"))?;
    if current != *transaction
        || !matches!(
            current.state,
            StartTransactionStateV1::PromptSubmissionIndeterminate { .. }
        )
    {
        anyhow::bail!("public Start indeterminate result does not exact-join durable authority");
    }
    let envelope = failed_prompt_envelope(
        "runtime",
        "submission_outcome_unknown",
        format!(
            "durable Start transaction {} crossed the no-replay barrier without committed authenticated provider acceptance; the real prompt was not replayed",
            current.transaction_id
        ),
    );
    PublicPromptRenderer::new(json).render_start_terminal_strict(&envelope)?;
    Ok(1)
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
    use std::os::unix::fs::DirBuilderExt;

    let path = private_stop_transport_path(store, orchestration_session_id, participant_id);
    let parent = path.parent().ok_or_else(|| {
        anyhow::anyhow!(
            "private stop transport path '{}' is missing a parent directory",
            path.display()
        )
    })?;
    let mut directory_builder = fs::DirBuilder::new();
    directory_builder.recursive(true).mode(0o700);
    directory_builder
        .create(parent)
        .with_context(|| format!("failed to create private directory {}", parent.display()))?;
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
        #[cfg(target_os = "linux")]
        endpoint_identity: None,
    })
}

#[cfg(unix)]
pub(crate) async fn register_durable_start_private_stop_transport(
    orchestration_session_id: &str,
    participant_id: &str,
    stop_tx: PrivateStopRequestSender,
) -> Result<PrivateStopTransport> {
    #[cfg(target_os = "linux")]
    let (episode, binding) = durable_private_transport_episode(
        orchestration_session_id,
        participant_id,
        PrivateTransportAvailabilityV1::Missing,
    )?;
    #[cfg(target_os = "linux")]
    let expected_uid = unsafe { libc::geteuid() };
    #[cfg(target_os = "linux")]
    let path = private_episode_transport_path_for_binding(
        &substrate_paths::substrate_home()?,
        expected_uid,
        &binding,
        "stop",
        "stop",
    );
    #[cfg(not(target_os = "linux"))]
    let path = durable_start_private_transport_path(
        "stop",
        "stop",
        orchestration_session_id,
        participant_id,
    )?;
    #[cfg(target_os = "linux")]
    let trusted_namespace_root =
        private_episode_namespace_root(&path, &substrate_paths::substrate_home()?, expected_uid)?;
    #[cfg(target_os = "linux")]
    prepare_private_episode_endpoint_for_bind(&path, expected_uid, &trusted_namespace_root)?;
    #[cfg(not(target_os = "linux"))]
    {
        let parent = path.parent().ok_or_else(|| {
            anyhow::anyhow!(
                "durable Start stop transport path '{}' is missing a parent directory",
                path.display()
            )
        })?;
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create {}", parent.display()))?;
        remove_existing_stop_transport_path(&path)?;
    }
    let listener = UnixListener::bind(&path).with_context(|| {
        format!(
            "failed to bind durable Start stop transport {}",
            path.display()
        )
    })?;
    #[cfg(target_os = "linux")]
    fs::set_permissions(&path, fs::Permissions::from_mode(0o600)).with_context(|| {
        format!(
            "failed to secure durable Start stop transport {}",
            path.display()
        )
    })?;
    #[cfg(target_os = "linux")]
    let endpoint_identity = private_episode_endpoint_identity(&path)?;
    #[cfg(target_os = "linux")]
    if private_transport_endpoint_metadata_availability(&path, expected_uid)
        != PrivateTransportAvailabilityV1::Available
    {
        let _ = remove_private_episode_endpoint_if_same(&path, endpoint_identity);
        anyhow::bail!(
            "durable Start stop transport {} failed exact owner/mode availability validation",
            path.display()
        );
    }
    #[cfg(target_os = "linux")]
    {
        let observer = HostExecutionEpisodeObserverV1::new(episode, binding)?;
        if let Err(error) = observer.observe_current(
            "durable-stop-endpoint-available",
            HostExecutionEpisodeObservationV1::Endpoint {
                availability: PrivateTransportAvailabilityV1::Available,
            },
        ) {
            let _ = remove_private_episode_endpoint_if_same(&path, endpoint_identity);
            return Err(error).context("durable Stop endpoint publication became stale");
        }
    }
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
        #[cfg(target_os = "linux")]
        let _ = remove_private_episode_endpoint_if_same(&path_for_task, endpoint_identity);
        #[cfg(not(target_os = "linux"))]
        let _ = tokio::fs::remove_file(&path_for_task).await;
    });
    Ok(PrivateStopTransport {
        shutdown_tx: Some(shutdown_tx),
        task: Some(task),
        path,
        #[cfg(target_os = "linux")]
        endpoint_identity: Some(endpoint_identity),
    })
}

#[cfg(unix)]
pub(crate) async fn register_private_cancel_transport(
    store: &AgentRuntimeStateStore,
    orchestration_session_id: &str,
    participant_id: &str,
    cancel_tx: PrivateCancelRequestSender,
) -> Result<PrivateCancelTransport> {
    use std::os::unix::fs::DirBuilderExt;

    let path = private_cancel_transport_path(store, orchestration_session_id, participant_id);
    let parent = path.parent().ok_or_else(|| {
        anyhow::anyhow!(
            "private cancel transport path '{}' is missing a parent directory",
            path.display()
        )
    })?;
    let mut directory_builder = fs::DirBuilder::new();
    directory_builder.recursive(true).mode(0o700);
    directory_builder
        .create(parent)
        .with_context(|| format!("failed to create private directory {}", parent.display()))?;
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
            let outcome = if !matches!(&request.payload, PrivateStopRequestPayloadV1::Legacy) {
                PrivateStopOutcome::ProtocolError
            } else if runtime_is_terminal(&manifest) {
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

#[cfg(target_os = "linux")]
fn prompt_submission_episode_for_runtime(
    runtime: &PromptSubmitRuntime,
) -> Result<Option<HostExecutionEpisodeObserverV1>> {
    let substrate_home = substrate_paths::substrate_home()?;
    if !substrate_home
        .join("authority-v1/state-root-v1.json")
        .exists()
    {
        return Ok(None);
    }
    let authority = HostSessionAuthority::from_trusted_root(
        TrustedAuthorityRoot::open(&substrate_home)
            .map_err(|error| anyhow::anyhow!(error.to_string()))?,
    )
    .map_err(|error| anyhow::anyhow!(error.to_string()))?;
    let root = authority
        .read_a12b_root()
        .map_err(|error| anyhow::anyhow!(error.to_string()))?;
    let manifest = runtime
        .manifest
        .lock()
        .map_err(|_| anyhow::anyhow!("runtime manifest mutex poisoned"))?
        .clone();
    let orchestration_session_id = manifest.handle.orchestration_session_id.as_str();
    if !root
        .session_namespace_map
        .contains_key(orchestration_session_id)
    {
        return Ok(None);
    }
    let current = authority
        .resolve_current_exact(orchestration_session_id, None)
        .map_err(|error| anyhow::anyhow!(error.to_string()))?;
    if current.caller.participant_id != manifest.handle.participant_id
        || current.caller.descriptor.backend_id != runtime.descriptor.backend_id
        || current.authority.lifecycle_posture != HostSessionPostureV1::ActiveAttached
    {
        anyhow::bail!(
            "stale_transport: prompt submission runtime does not match exact active HSA authority"
        );
    }
    let participant_id = current.caller.participant_id.clone();
    let (mut episode, binding) = host_execution_episode_for_authority(
        &current,
        HostExecutionEpisodeKindV1::SyntheticOrRecoveredEpisode,
        &format!("durable-private-owner:{participant_id}"),
        Some(ProcessRefV1 {
            pid: std::process::id(),
        }),
        PrivateTransportAvailabilityV1::Missing,
    )?;
    let expected_uid = unsafe { libc::geteuid() };
    let endpoint_path = private_episode_transport_path_for_binding(
        &substrate_home,
        expected_uid,
        &binding,
        "prompt",
        "prompt",
    );
    episode.record_local_observation(
        &binding,
        HostExecutionEpisodeObservationV1::Endpoint {
            availability: private_transport_endpoint_metadata_availability(
                &endpoint_path,
                expected_uid,
            ),
        },
    )?;
    episode.validate_current_authority(&binding, &current)?;
    HostExecutionEpisodeObserverV1::new(episode, binding).map(Some)
}

#[cfg(target_os = "linux")]
fn observe_prompt_submission_episode(
    observer: Option<&HostExecutionEpisodeObserverV1>,
    observation_key: &str,
    observation: HostExecutionEpisodeObservationV1,
) -> Result<()> {
    if let Some(observer) = observer {
        observer.observe_current(observation_key, observation)?;
    }
    Ok(())
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
    #[cfg(target_os = "linux")]
    let episode_observer = prompt_submission_episode_for_runtime(runtime)?;
    #[cfg(target_os = "linux")]
    observe_prompt_submission_episode(
        episode_observer.as_ref(),
        &format!("prompt:{run_id}:ready"),
        HostExecutionEpisodeObservationV1::Readiness { ready: true },
    )?;
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
    #[cfg(target_os = "linux")]
    {
        observe_prompt_submission_episode(
            episode_observer.as_ref(),
            &format!("prompt:{run_id}:accepted"),
            HostExecutionEpisodeObservationV1::Prompt { accepted: true },
        )?;
        observe_prompt_submission_episode(
            episode_observer.as_ref(),
            &format!("prompt:{run_id}:stream-open"),
            HostExecutionEpisodeObservationV1::Stream { active: true },
        )?;
    }
    let agent_api::AgentWrapperRunHandle {
        mut events,
        completion,
    } = control.handle;

    let mut event_index = 0_u64;
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
        #[cfg(target_os = "linux")]
        if episode_observer.is_some() {
            event_index += 1;
            let observed_at =
                TimestampV1::parse(Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Nanos, true))
                    .map_err(|error| anyhow::anyhow!(error.to_string()))?;
            observe_prompt_submission_episode(
                episode_observer.as_ref(),
                &format!("prompt:{run_id}:event:{event_index}"),
                HostExecutionEpisodeObservationV1::Heartbeat { observed_at },
            )?;
        } else {
            persist_runtime_snapshots(&runtime.store, &orchestration_snapshot, &manifest_snapshot)?;
        }
        #[cfg(not(target_os = "linux"))]
        persist_runtime_snapshots(&runtime.store, &orchestration_snapshot, &manifest_snapshot)?;
        on_event(SubmittedPromptStreamEvent::Agent(Box::new(event)));
    }

    let completion = completion
        .await
        .map_err(|err| anyhow::anyhow!("substrate: error: {}", err))?;
    #[cfg(target_os = "linux")]
    observe_prompt_submission_episode(
        episode_observer.as_ref(),
        &format!("prompt:{run_id}:stream-closed"),
        HostExecutionEpisodeObservationV1::Stream { active: false },
    )?;
    if let Some(session_id) = extract_session_handle_id(completion.data.as_ref()) {
        let mut manifest_guard = runtime
            .manifest
            .lock()
            .expect("runtime manifest mutex poisoned");
        if manifest_guard.internal.uaa_session_id.as_deref() != Some(session_id) {
            manifest_guard.set_uaa_session_id(session_id.to_string());
        }
        #[cfg(target_os = "linux")]
        observe_prompt_submission_episode(
            episode_observer.as_ref(),
            &format!("prompt:{run_id}:handle"),
            HostExecutionEpisodeObservationV1::Handle { present: true },
        )?;
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
    let (orchestration_session_id, backend_id) = validate_public_prompt_command_request(&request)?;

    #[cfg(target_os = "linux")]
    if request.action == PublicPromptAction::Turn {
        if let Some(transport) =
            resolve_current_hsa_private_prompt_transport(orchestration_session_id, backend_id)?
        {
            return run_public_prompt_transport(&request, &transport.path, Some(&transport));
        }
    }

    let store = AgentRuntimeStateStore::new()?;
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
        #[cfg(target_os = "linux")]
        return run_public_prompt_transport(&request, &transport_path, None);
        #[cfg(not(target_os = "linux"))]
        return run_public_prompt_transport(&request, &transport_path);
    }
}

#[cfg(target_os = "linux")]
#[derive(Clone, Debug)]
struct ExactPrivatePromptTransportV1 {
    episode: HostExecutionEpisodeV1,
    binding: HostExecutionEpisodeBindingV1,
    path: PathBuf,
}

#[cfg(target_os = "linux")]
fn resolve_current_hsa_private_prompt_transport(
    orchestration_session_id: &str,
    backend_id: &str,
) -> Result<Option<ExactPrivatePromptTransportV1>> {
    let substrate_home = substrate_paths::substrate_home()?;
    if !substrate_home
        .join("authority-v1/state-root-v1.json")
        .exists()
    {
        return Ok(None);
    }
    let authority = HostSessionAuthority::from_trusted_root(
        TrustedAuthorityRoot::open(&substrate_home)
            .map_err(|error| anyhow::anyhow!(error.to_string()))?,
    )
    .map_err(|error| anyhow::anyhow!(error.to_string()))?;
    let root = authority
        .read_a12b_root()
        .map_err(|error| anyhow::anyhow!(error.to_string()))?;
    if !root
        .session_namespace_map
        .contains_key(orchestration_session_id)
    {
        return Ok(None);
    }
    let current = authority
        .resolve_current_exact(orchestration_session_id, None)
        .map_err(|error| anyhow::anyhow!(error.to_string()))?;
    if current.caller.descriptor.backend_id != backend_id {
        anyhow::bail!(
            "backend_not_in_session: orchestration session {} has no exact backend slot for {}",
            orchestration_session_id,
            backend_id
        );
    }
    if current.authority.lifecycle_posture != HostSessionPostureV1::ActiveAttached {
        anyhow::bail!(
            "owner_unreachable: orchestration session {} backend {} is not currently attached to a live retained turn target",
            orchestration_session_id,
            backend_id
        );
    }
    let participant_id = current.caller.participant_id.clone();
    let (mut episode, binding) = host_execution_episode_for_authority(
        &current,
        HostExecutionEpisodeKindV1::SyntheticOrRecoveredEpisode,
        &format!("durable-private-owner:{participant_id}"),
        None,
        PrivateTransportAvailabilityV1::Missing,
    )?;
    let expected_uid = unsafe { libc::geteuid() };
    let path = private_episode_transport_path_for_binding(
        &substrate_home,
        expected_uid,
        &binding,
        "prompt",
        "prompt",
    );
    let availability = private_transport_endpoint_metadata_availability(&path, expected_uid);
    episode.record_local_observation(
        &binding,
        HostExecutionEpisodeObservationV1::Endpoint { availability },
    )?;
    episode.validate_current_authority(&binding, &current)?;
    Ok(Some(ExactPrivatePromptTransportV1 {
        episode,
        binding,
        path,
    }))
}

#[cfg(unix)]
fn run_public_prompt_transport(
    request: &PublicPromptCommandRequest,
    transport_path: &Path,
    #[cfg(target_os = "linux")] exact_transport: Option<&ExactPrivatePromptTransportV1>,
) -> Result<()> {
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .context("failed to initialize prompt transport runtime")?;
    let mut renderer = PublicPromptRenderer::new(request.json);
    let mut stream_started = false;
    let mut saw_terminal = false;
    let result = rt.block_on(async {
        #[cfg(target_os = "linux")]
        if let Some(exact_transport) = exact_transport {
            wait_for_current_hsa_private_prompt_transport(exact_transport).await?;
            validate_host_execution_episode_against_current_hsa(
                &exact_transport.episode,
                &exact_transport.binding,
            )?;
        } else {
            wait_for_private_prompt_transport(transport_path).await?;
        }
        #[cfg(not(target_os = "linux"))]
        wait_for_private_prompt_transport(transport_path).await?;

        let stream_result = request_private_prompt_stream(
            transport_path,
            request.action,
            &request.prompt.prompt_text,
            |envelope| {
                stream_started = true;
                if matches!(
                    envelope,
                    PublicPromptEnvelope::Completed { .. } | PublicPromptEnvelope::Failed { .. }
                ) {
                    saw_terminal = true;
                }
                renderer.render(envelope)
            },
        )
        .await;

        #[cfg(target_os = "linux")]
        if exact_transport.is_some() {
            return stream_result.map_err(|error| {
                let classification = match private_stop_transport_error_kind(&error) {
                    Some(io::ErrorKind::NotFound) => "missing_transport",
                    Some(io::ErrorKind::ConnectionRefused) => "refused_transport",
                    _ => "failed_transport",
                };
                error.context(format!(
                    "{classification}: exact HSA private prompt transport {} became unavailable",
                    transport_path.display()
                ))
            });
        }
        stream_result
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
pub(crate) async fn request_private_authority_managed_stop(
    path: &Path,
    delivery: &HostSessionStopDeliveryV1,
) -> Result<PrivateStopOutcome> {
    #[cfg(target_os = "linux")]
    {
        let expected_path = durable_start_stop_transport_path(
            &delivery.orchestration_session_id,
            &delivery.authoritative_participant_id,
        )?;
        if expected_path != path {
            anyhow::bail!(
                "stale_transport: private HSA Stop endpoint does not match exact current episode binding"
            );
        }
        let availability =
            private_transport_endpoint_metadata_availability(path, unsafe { libc::geteuid() });
        match availability {
            PrivateTransportAvailabilityV1::Available => {}
            PrivateTransportAvailabilityV1::Missing | PrivateTransportAvailabilityV1::Refused => {
                return recover_authority_managed_stop_episode(delivery, availability)
                    .with_context(|| {
                        format!(
                            "recovery_failed: exact HSA Stop recovery after {} transport",
                            match availability {
                                PrivateTransportAvailabilityV1::Missing => "missing",
                                PrivateTransportAvailabilityV1::Refused => "refused",
                                _ => unreachable!("availability arm is exact"),
                            }
                        )
                    });
            }
            PrivateTransportAvailabilityV1::Stale => {
                anyhow::bail!(
                    "stale_transport: private HSA Stop endpoint failed exact owner, mode, or socket validation"
                );
            }
            PrivateTransportAvailabilityV1::Failed => {
                anyhow::bail!(
                    "failed_transport: private HSA Stop endpoint could not be classified safely"
                );
            }
        }
    }
    let result = request_private_authority_managed_stop_with_response_timeout(
        path,
        delivery,
        PRIVATE_HSA_STOP_RESPONSE_TIMEOUT,
    )
    .await;
    #[cfg(target_os = "linux")]
    if let Err(error) = &result {
        let availability = match private_stop_transport_error_kind(error) {
            Some(io::ErrorKind::NotFound) => Some(PrivateTransportAvailabilityV1::Missing),
            Some(io::ErrorKind::ConnectionRefused) => Some(PrivateTransportAvailabilityV1::Refused),
            _ => None,
        };
        if let Some(availability) = availability {
            return recover_authority_managed_stop_episode(delivery, availability).with_context(
                || {
                    format!(
                        "recovery_failed: exact HSA Stop recovery after {} transport",
                        match availability {
                            PrivateTransportAvailabilityV1::Missing => "missing",
                            PrivateTransportAvailabilityV1::Refused => "refused",
                            _ => "unavailable",
                        }
                    )
                },
            );
        }
    }
    result
}

#[cfg(target_os = "linux")]
fn recover_authority_managed_stop_episode(
    delivery: &HostSessionStopDeliveryV1,
    availability: PrivateTransportAvailabilityV1,
) -> Result<PrivateStopOutcome> {
    let authority = authority_for_stop_delivery(delivery)?;
    let current = authority
        .resolve_current_exact(&delivery.orchestration_session_id, None)
        .map_err(|error| anyhow::anyhow!(error.to_string()))?;
    if current.caller.participant_id != delivery.authoritative_participant_id {
        anyhow::bail!("recovered Stop episode participant is not current HSA authority");
    }
    let (episode, binding) = host_execution_episode_for_authority(
        &current,
        HostExecutionEpisodeKindV1::SyntheticOrRecoveredEpisode,
        &delivery.intent_id,
        Some(ProcessRefV1 {
            pid: std::process::id(),
        }),
        availability,
    )?;
    let observer = HostExecutionEpisodeObserverV1::new(episode, binding)?;
    observer.observe_current(
        "private-stop-unavailable",
        HostExecutionEpisodeObservationV1::Endpoint { availability },
    )?;

    let closeout = match accept_authority_managed_stop(delivery)? {
        AuthorityManagedStopAcceptanceV1::AlreadyTerminal => {
            return Ok(PrivateStopOutcome::AlreadyTerminal);
        }
        AuthorityManagedStopAcceptanceV1::Accepted(closeout) => closeout,
    };
    let accepted = authority
        .resolve_current_exact(&delivery.orchestration_session_id, None)
        .map_err(|error| anyhow::anyhow!(error.to_string()))?;
    let (accepted_episode, accepted_binding) = host_execution_episode_for_authority(
        &accepted,
        HostExecutionEpisodeKindV1::SyntheticOrRecoveredEpisode,
        &delivery.intent_id,
        Some(ProcessRefV1 {
            pid: std::process::id(),
        }),
        availability,
    )?;
    let accepted_observer =
        HostExecutionEpisodeObserverV1::new(accepted_episode, accepted_binding)?;
    accepted_observer.observe_current(
        "private-stop-closeout",
        HostExecutionEpisodeObservationV1::Exit(EpisodeExitObservationV1 {
            observed_at: hsa_stop_timestamp()?,
            exit_code: Some(0),
            signal: None,
        }),
    )?;
    complete_authority_managed_stop(&closeout)?;
    Ok(PrivateStopOutcome::Accepted)
}

#[cfg(unix)]
async fn request_private_authority_managed_stop_with_response_timeout(
    path: &Path,
    delivery: &HostSessionStopDeliveryV1,
    response_timeout: Duration,
) -> Result<PrivateStopOutcome> {
    let mut stream = match UnixStream::connect(path).await {
        Ok(stream) => stream,
        Err(err) => {
            let kind = err.kind();
            return Err(anyhow::Error::new(err)
                .context(format_private_stop_transport_connect_error(path, kind)));
        }
    };
    let request = PrivateStopRequestV2 {
        version: 2,
        action: "stop".into(),
        hsa_stop: delivery.clone(),
    };
    stream
        .write_all(serde_json::to_string(&request)?.as_bytes())
        .await?;
    stream.write_all(b"\n").await?;
    stream.flush().await?;

    let mut reader = BufReader::new(stream);
    let mut line = String::new();
    let bytes_read = tokio::time::timeout(response_timeout, reader.read_line(&mut line))
        .await
        .with_context(|| {
            format!(
                "timeout_transport: timed out waiting for exact private HSA Stop response from {}",
                path.display()
            )
        })??;
    if bytes_read == 0 {
        anyhow::bail!("private HSA Stop transport closed before its exact response");
    }
    let response: PrivateStopResponse = serde_json::from_str(line.trim()).with_context(|| {
        format!(
            "failed to decode private HSA Stop transport response from {}",
            path.display()
        )
    })?;
    Ok(response.outcome)
}

#[cfg(unix)]
pub(crate) fn accept_authority_managed_stop(
    delivery: &HostSessionStopDeliveryV1,
) -> Result<AuthorityManagedStopAcceptanceV1> {
    let authority = authority_for_stop_delivery(delivery)?;
    let intent = authority
        .stop_transaction_for_session(&delivery.orchestration_session_id)
        .map_err(|error| anyhow::anyhow!(error.to_string()))?
        .ok_or_else(|| anyhow::anyhow!("exact HSA Stop intent was not found"))?;
    if stop_delivery(&intent) != *delivery {
        anyhow::bail!("private HSA Stop delivery is substituted or mismatched");
    }
    let (acceptance_id, accepted_at) = match &intent.state {
        super::host_session_authority::store_schema::HostSessionStopIntentStateV1::Issued => (
            format!("{}:delivery", intent.intent_id),
            hsa_stop_timestamp()?,
        ),
        super::host_session_authority::store_schema::HostSessionStopIntentStateV1::DeliveryAccepted {
            acceptance_id,
            accepted_by_participant_id,
            accepted_at,
        } if accepted_by_participant_id == &delivery.authoritative_participant_id => {
            (acceptance_id.clone(), accepted_at.clone())
        }
        super::host_session_authority::store_schema::HostSessionStopIntentStateV1::Completed { .. } => {
            return Ok(AuthorityManagedStopAcceptanceV1::AlreadyTerminal);
        }
        _ => anyhow::bail!("private HSA Stop delivery conflicts with committed authority"),
    };
    let outcome = authority
        .accept_stop_delivery(&AcceptHostSessionStopDeliveryRequestV1 {
            intent_id: delivery.intent_id.clone(),
            request_id: delivery.request_id.clone(),
            payload_commitment: delivery.payload_commitment.clone(),
            orchestration_session_id: delivery.orchestration_session_id.clone(),
            authoritative_participant_id: delivery.authoritative_participant_id.clone(),
            authority_revision: delivery.authority_revision,
            authority_record_commitment: delivery.authority_record_commitment.clone(),
            acceptance_id: acceptance_id.clone(),
            accepted_at,
        })
        .map_err(|error| anyhow::anyhow!(error.to_string()))?;
    if !matches!(
        outcome,
        HostSessionStopDeliveryOutcomeV1::Accepted(_) | HostSessionStopDeliveryOutcomeV1::Joined(_)
    ) {
        anyhow::bail!("private HSA Stop delivery did not commit exact acceptance");
    }
    Ok(AuthorityManagedStopAcceptanceV1::Accepted(Box::new(
        AuthorityManagedStopCloseoutV1 {
            delivery: delivery.clone(),
            acceptance_id,
        },
    )))
}

#[cfg(unix)]
pub(crate) fn complete_authority_managed_stop(
    closeout: &AuthorityManagedStopCloseoutV1,
) -> Result<()> {
    let authority = authority_for_stop_delivery(&closeout.delivery)?;
    let intent = authority
        .stop_transaction_for_session(&closeout.delivery.orchestration_session_id)
        .map_err(|error| anyhow::anyhow!(error.to_string()))?
        .ok_or_else(|| anyhow::anyhow!("exact HSA Stop intent was not found"))?;
    if stop_delivery(&intent) != closeout.delivery {
        anyhow::bail!("private HSA Stop closeout is substituted or mismatched");
    }
    let (result_id, completed_at) = match &intent.state {
        super::host_session_authority::store_schema::HostSessionStopIntentStateV1::DeliveryAccepted {
            acceptance_id,
            ..
        } if acceptance_id == &closeout.acceptance_id => (
            format!("{}:result", intent.intent_id),
            hsa_stop_timestamp()?,
        ),
        super::host_session_authority::store_schema::HostSessionStopIntentStateV1::Completed {
            delivery_acceptance_id,
            result_id,
            completed_at,
            ..
        } if delivery_acceptance_id.as_ref() == Some(&closeout.acceptance_id) => {
            (result_id.clone(), completed_at.clone())
        }
        _ => anyhow::bail!("private HSA Stop closeout has no exact accepted delivery"),
    };
    let outcome = authority
        .complete_stop(&CompleteHostSessionStopRequestV1 {
            intent_id: intent.intent_id.clone(),
            request_id: intent.request_id.clone(),
            payload_commitment: intent.payload_commitment.clone(),
            orchestration_session_id: intent.orchestration_session_id.clone(),
            authoritative_participant_id: intent.authoritative_participant_id.clone(),
            caller: HostSessionTransitionCallerV1 {
                kind: HostSessionTransitionCallerKindV1::Repl,
                caller_participant_id: Some(intent.authoritative_participant_id.clone()),
                auto_attach_obligation_id: None,
                auto_attach_claim_owner: None,
            },
            delivery_acceptance_id: Some(closeout.acceptance_id.clone()),
            result_id,
            completed_at,
        })
        .map_err(|error| anyhow::anyhow!(error.to_string()))?;
    if !matches!(
        outcome,
        HostSessionStopCompletionOutcomeV1::Completed(_)
            | HostSessionStopCompletionOutcomeV1::Joined(_)
    ) {
        anyhow::bail!("private HSA Stop closeout did not commit exact terminal truth");
    }
    Ok(())
}

#[cfg(unix)]
fn authority_for_stop_delivery(
    delivery: &HostSessionStopDeliveryV1,
) -> Result<HostSessionAuthority> {
    let authority = HostSessionAuthority::from_trusted_root(
        TrustedAuthorityRoot::open(Path::new(&delivery.bootstrap_home.physical_path))
            .map_err(|error| anyhow::anyhow!(error.to_string()))?,
    )
    .map_err(|error| anyhow::anyhow!(error.to_string()))?;
    let root = authority
        .read_a12b_root()
        .map_err(|error| anyhow::anyhow!(error.to_string()))?;
    if root.authority_store_id != delivery.authority_store_id
        || root.bootstrap_home != delivery.bootstrap_home
        || !root
            .session_namespace_map
            .contains_key(&delivery.orchestration_session_id)
    {
        anyhow::bail!("private HSA Stop authority store or session identity is mismatched");
    }
    Ok(authority)
}

#[cfg(unix)]
fn hsa_stop_timestamp() -> Result<TimestampV1> {
    TimestampV1::parse(Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Nanos, true))
        .map_err(|error| anyhow::anyhow!(error.to_string()))
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
        #[cfg(target_os = "linux")]
        endpoint_identity: None,
    })
}

#[cfg(unix)]
pub(crate) async fn register_durable_start_private_prompt_transport(
    orchestration_session_id: &str,
    participant_id: &str,
    prompt_tx: PrivatePromptRequestSender,
) -> Result<PrivatePromptTransport> {
    #[cfg(target_os = "linux")]
    let (episode, binding) = durable_private_transport_episode(
        orchestration_session_id,
        participant_id,
        PrivateTransportAvailabilityV1::Missing,
    )?;
    #[cfg(target_os = "linux")]
    let expected_uid = unsafe { libc::geteuid() };
    #[cfg(target_os = "linux")]
    let path = private_episode_transport_path_for_binding(
        &substrate_paths::substrate_home()?,
        expected_uid,
        &binding,
        "prompt",
        "prompt",
    );
    #[cfg(not(target_os = "linux"))]
    let path = durable_start_private_transport_path(
        "prompt",
        "prompt",
        orchestration_session_id,
        participant_id,
    )?;
    #[cfg(target_os = "linux")]
    let trusted_namespace_root =
        private_episode_namespace_root(&path, &substrate_paths::substrate_home()?, expected_uid)?;
    #[cfg(target_os = "linux")]
    prepare_private_episode_endpoint_for_bind(&path, expected_uid, &trusted_namespace_root)?;
    #[cfg(not(target_os = "linux"))]
    {
        let parent = path.parent().ok_or_else(|| {
            anyhow::anyhow!(
                "durable Start prompt transport path '{}' is missing a parent directory",
                path.display()
            )
        })?;
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create {}", parent.display()))?;
        remove_existing_stop_transport_path(&path)?;
    }
    let listener = UnixListener::bind(&path).with_context(|| {
        format!(
            "failed to bind durable Start prompt transport {}",
            path.display()
        )
    })?;
    #[cfg(target_os = "linux")]
    fs::set_permissions(&path, fs::Permissions::from_mode(0o600)).with_context(|| {
        format!(
            "failed to secure durable Start prompt transport {}",
            path.display()
        )
    })?;
    #[cfg(target_os = "linux")]
    let endpoint_identity = private_episode_endpoint_identity(&path)?;
    #[cfg(target_os = "linux")]
    if private_transport_endpoint_metadata_availability(&path, expected_uid)
        != PrivateTransportAvailabilityV1::Available
    {
        let _ = remove_private_episode_endpoint_if_same(&path, endpoint_identity);
        anyhow::bail!(
            "durable Start prompt transport {} failed exact owner/mode availability validation",
            path.display()
        );
    }
    #[cfg(target_os = "linux")]
    {
        let observer = HostExecutionEpisodeObserverV1::new(episode, binding)?;
        if let Err(error) = observer.observe_current(
            "durable-prompt-endpoint-available",
            HostExecutionEpisodeObservationV1::Endpoint {
                availability: PrivateTransportAvailabilityV1::Available,
            },
        ) {
            let _ = remove_private_episode_endpoint_if_same(&path, endpoint_identity);
            return Err(error).context("durable prompt endpoint publication became stale");
        }
    }
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
        #[cfg(target_os = "linux")]
        let _ = remove_private_episode_endpoint_if_same(&path_for_task, endpoint_identity);
        #[cfg(not(target_os = "linux"))]
        let _ = tokio::fs::remove_file(&path_for_task).await;
    });
    Ok(PrivatePromptTransport {
        shutdown_tx: Some(shutdown_tx),
        task: Some(task),
        path,
        #[cfg(target_os = "linux")]
        endpoint_identity: Some(endpoint_identity),
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
    let outcome = if bytes_read == 0 {
        PrivateStopOutcome::ProtocolError
    } else if let Ok(payload) = parse_private_stop_request(line.trim()) {
        let (response_tx, response_rx) = oneshot::channel();
        if stop_tx
            .send(PrivateStopRequest {
                payload,
                response_tx,
            })
            .is_err()
        {
            PrivateStopOutcome::OwnerUnreachable
        } else {
            match tokio::time::timeout(Duration::from_secs(5), response_rx).await {
                Ok(Ok(outcome)) => outcome,
                Ok(Err(_)) | Err(_) => PrivateStopOutcome::OwnerUnreachable,
            }
        }
    } else {
        PrivateStopOutcome::ProtocolError
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
fn parse_private_stop_request(payload: &str) -> Result<PrivateStopRequestPayloadV1> {
    let syntax: serde_json::Value =
        serde_json::from_str(payload).context("failed to decode private stop request")?;
    let version = syntax
        .as_object()
        .and_then(|object| object.get("version"))
        .and_then(serde_json::Value::as_u64)
        .ok_or_else(|| anyhow::anyhow!("private stop request has no exact version"))?;
    match version {
        1 => {
            let request: PrivateStopRequestV1 = serde_json::from_value(syntax)
                .context("failed to decode legacy private stop request")?;
            if request.version != 1 || request.action != "stop" {
                anyhow::bail!("unsupported private stop request");
            }
            Ok(PrivateStopRequestPayloadV1::Legacy)
        }
        2 => {
            let request: PrivateStopRequestV2 = serde_json::from_value(syntax)
                .context("failed to decode HSA private stop request")?;
            if request.version != 2 || request.action != "stop" {
                anyhow::bail!("unsupported private stop request");
            }
            Ok(PrivateStopRequestPayloadV1::AuthorityManaged(Box::new(
                request.hsa_stop,
            )))
        }
        _ => anyhow::bail!("unsupported private stop request"),
    }
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
struct PrivateStopRequestV2 {
    version: u8,
    action: String,
    hsa_stop: HostSessionStopDeliveryV1,
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

pub(crate) fn public_prompt_rendered_exit(exit_code: i32) -> anyhow::Error {
    anyhow::Error::new(PublicPromptRenderedExit { exit_code })
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
#[cfg(target_os = "linux")]
async fn wait_for_current_hsa_private_prompt_transport(
    transport: &ExactPrivatePromptTransportV1,
) -> Result<()> {
    let started_at = std::time::Instant::now();
    loop {
        validate_host_execution_episode_against_current_hsa(&transport.episode, &transport.binding)
            .context("stale_transport: private prompt episode authority changed")?;
        let availability =
            private_transport_endpoint_metadata_availability(&transport.path, unsafe {
                libc::geteuid()
            });
        match availability {
            PrivateTransportAvailabilityV1::Available => {
                validate_host_execution_episode_against_current_hsa(
                    &transport.episode,
                    &transport.binding,
                )
                .context("stale_transport: private prompt episode authority changed")?;
                return Ok(());
            }
            PrivateTransportAvailabilityV1::Missing | PrivateTransportAvailabilityV1::Refused => {
                if started_at.elapsed() >= PRIVATE_PROMPT_READY_TIMEOUT {
                    anyhow::bail!(
                        "{}_transport: timed out waiting for exact HSA private prompt transport {}",
                        match availability {
                            PrivateTransportAvailabilityV1::Missing => "missing",
                            PrivateTransportAvailabilityV1::Refused => "refused",
                            _ => unreachable!("availability arm is exact"),
                        },
                        transport.path.display()
                    );
                }
            }
            PrivateTransportAvailabilityV1::Stale => {
                anyhow::bail!(
                    "stale_transport: exact HSA private prompt transport {} failed owner, mode, or socket validation",
                    transport.path.display()
                );
            }
            PrivateTransportAvailabilityV1::Failed => {
                anyhow::bail!(
                    "failed_transport: exact HSA private prompt transport {} could not be classified safely",
                    transport.path.display()
                );
            }
        }
        tokio::time::sleep(PRIVATE_PROMPT_READY_POLL_INTERVAL).await;
    }
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
trait PromptRenderOutput {
    fn with_stdout<T, F>(&mut self, render: F) -> T
    where
        F: FnOnce(&mut dyn Write) -> T;

    fn with_stderr<T, F>(&mut self, render: F) -> T
    where
        F: FnOnce(&mut dyn Write) -> T;
}

#[cfg(unix)]
struct StandardPromptRenderOutput;

#[cfg(unix)]
impl PromptRenderOutput for StandardPromptRenderOutput {
    fn with_stdout<T, F>(&mut self, render: F) -> T
    where
        F: FnOnce(&mut dyn Write) -> T,
    {
        let stdout = io::stdout();
        let mut lock = stdout.lock();
        render(&mut lock)
    }

    fn with_stderr<T, F>(&mut self, render: F) -> T
    where
        F: FnOnce(&mut dyn Write) -> T,
    {
        let stderr = io::stderr();
        let mut lock = stderr.lock();
        render(&mut lock)
    }
}

#[cfg(all(unix, test))]
#[derive(Default)]
struct PromptRenderBuffer {
    stdout: Vec<u8>,
    stderr: Vec<u8>,
    stdout_flushes: usize,
    stderr_flushes: usize,
    stdout_write_error: bool,
    stderr_write_error: bool,
    stdout_flush_error: bool,
    stderr_flush_error: bool,
}

#[cfg(all(unix, test))]
struct PromptRenderBufferWriter<'a> {
    bytes: &'a mut Vec<u8>,
    flushes: &'a mut usize,
    fail_writes: bool,
    fail_flush: bool,
}

#[cfg(all(unix, test))]
impl Write for PromptRenderBufferWriter<'_> {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        if self.fail_writes {
            return Err(io::Error::other("in-memory prompt writer rejected write"));
        }
        self.bytes.extend_from_slice(bytes);
        Ok(bytes.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        *self.flushes += 1;
        if self.fail_flush {
            return Err(io::Error::other("in-memory prompt writer rejected flush"));
        }
        Ok(())
    }
}

#[cfg(all(unix, test))]
impl PromptRenderOutput for PromptRenderBuffer {
    fn with_stdout<T, F>(&mut self, render: F) -> T
    where
        F: FnOnce(&mut dyn Write) -> T,
    {
        let mut writer = PromptRenderBufferWriter {
            bytes: &mut self.stdout,
            flushes: &mut self.stdout_flushes,
            fail_writes: self.stdout_write_error,
            fail_flush: self.stdout_flush_error,
        };
        render(&mut writer)
    }

    fn with_stderr<T, F>(&mut self, render: F) -> T
    where
        F: FnOnce(&mut dyn Write) -> T,
    {
        let mut writer = PromptRenderBufferWriter {
            bytes: &mut self.stderr,
            flushes: &mut self.stderr_flushes,
            fail_writes: self.stderr_write_error,
            fail_flush: self.stderr_flush_error,
        };
        render(&mut writer)
    }
}

#[cfg(unix)]
impl PublicPromptRenderer {
    fn new(json: bool) -> Self {
        Self { json }
    }

    fn render(&mut self, envelope: &PublicPromptEnvelope) -> Result<()> {
        self.render_with_output(envelope, &mut StandardPromptRenderOutput)
    }

    fn render_start_terminal_strict(&mut self, envelope: &PublicPromptEnvelope) -> Result<()> {
        self.render_start_terminal_strict_with_output(envelope, &mut StandardPromptRenderOutput)
    }

    fn render_start_terminal_strict_with_output<O: PromptRenderOutput>(
        &mut self,
        envelope: &PublicPromptEnvelope,
        output: &mut O,
    ) -> Result<()> {
        if self.json {
            return output.with_stdout(|stdout| {
                writeln!(stdout, "{}", serde_json::to_string(envelope)?)
                    .context("failed to render committed Start JSON response")?;
                stdout
                    .flush()
                    .context("failed to flush committed Start JSON response")
            });
        }
        match envelope {
            PublicPromptEnvelope::Completed {
                action,
                orchestration_session_id,
                backend_id,
                participant_id,
                turn_outcome,
                session_posture,
                ..
            } => output.with_stdout(|stdout| {
                writeln!(
                    stdout,
                    "action={} orchestration_session_id={} backend_id={} participant_id={} turn_outcome={} session_posture={}",
                    action.as_str(),
                    orchestration_session_id,
                    backend_id,
                    participant_id.as_deref().unwrap_or("-"),
                    turn_outcome,
                    session_posture.as_str()
                )
                .context("failed to render committed Start response")?;
                stdout
                    .flush()
                    .context("failed to flush committed Start response")
            }),
            PublicPromptEnvelope::Failed { message, .. } => output.with_stderr(|stderr| {
                writeln!(stderr, "{message}")
                    .context("failed to render committed Start failure")?;
                stderr
                    .flush()
                    .context("failed to flush committed Start failure")
            }),
            _ => anyhow::bail!("strict Start renderer requires a terminal envelope"),
        }
    }

    fn render_with_output<O: PromptRenderOutput>(
        &mut self,
        envelope: &PublicPromptEnvelope,
        output: &mut O,
    ) -> Result<()> {
        if self.json {
            return output.with_stdout(|stdout| {
                writeln!(stdout, "{}", serde_json::to_string(envelope)?)
                    .context("failed to render prompt envelope")?;
                let _ = stdout.flush();
                Ok(())
            });
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
                output.with_stdout(|stdout| {
                    let _ = writeln!(
                        stdout,
                        "action={} orchestration_session_id={} backend_id={} participant_id={} turn_outcome={} session_posture={}",
                        action.as_str(),
                        orchestration_session_id,
                        backend_id,
                        participant_id.as_deref().unwrap_or("-"),
                        turn_outcome,
                        session_posture.as_str()
                    );
                    let _ = stdout.flush();
                });
            }
            PublicPromptEnvelope::Warning { message, .. }
            | PublicPromptEnvelope::Failed { message, .. } => {
                output.with_stderr(|stderr| {
                    let _ = writeln!(stderr, "{message}");
                    let _ = stderr.flush();
                });
            }
            PublicPromptEnvelope::Event {
                event_kind, data, ..
            } => {
                if event_kind == "stderr" {
                    output.with_stderr(|stderr| {
                        if let Ok(event) = serde_json::from_value::<AgentEvent>(data.clone()) {
                            let _ = writeln!(stderr, "{}", format_event_line(&event));
                        } else {
                            let fallback = prompt_event_text(data);
                            let fallback = if fallback.is_empty() {
                                structured_prompt_event_fallback_text(data).unwrap_or(fallback)
                            } else {
                                fallback
                            };
                            let _ = stderr.write_all(fallback.as_bytes());
                        }
                        let _ = stderr.flush();
                    });
                } else if let Ok(event) = serde_json::from_value::<AgentEvent>(data.clone()) {
                    output.with_stdout(|stdout| {
                        let _ = writeln!(stdout, "{}", format_event_line(&event));
                        let _ = stdout.flush();
                    });
                } else {
                    let fallback = prompt_event_text(data);
                    let fallback = if fallback.is_empty() {
                        structured_prompt_event_fallback_text(data).unwrap_or(fallback)
                    } else {
                        fallback
                    };
                    output.with_stdout(|stdout| {
                        let _ = stdout.write_all(fallback.as_bytes());
                        let _ = stdout.flush();
                    });
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
    #[cfg(unix)]
    use super::prompt_event_text;
    use super::{
        apply_runtime_cancel_closeout, apply_runtime_stop_closeout,
        prompt_completion_session_state, reconcile_hidden_owner_helper_start_timeout,
        runtime_is_terminal, validate_public_prompt_command_request, HiddenOwnerHelperLaunchPlan,
        HiddenOwnerHelperParticipantPlan, HiddenOwnerHelperSessionPlan,
        HiddenOwnerHelperStartTimeoutReconciliation, HiddenOwnerHelperStartupPromptPlan,
        LoadedPublicPrompt, OwnerHelperMode, PrivateCancelOutcome, PrivateStopOutcome,
        PromptSubmitRuntime, PublicPromptAction, PublicPromptCommandRequest, PublicSessionPosture,
        ResolvedRuntimeBackendKind, ResolvedRuntimeDescriptor, PURE_AGENT_PROTOCOL,
    };
    #[cfg(unix)]
    use super::{
        failed_prompt_envelope, handle_private_prompt_connection, parse_private_stop_request,
        private_prompt_request_channel, structured_prompt_event_fallback_text,
        PrivateStopRequestPayloadV1, PromptRenderBuffer, PublicPromptEnvelope,
        PublicPromptRenderer,
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
    use std::path::PathBuf;
    use std::sync::atomic::AtomicBool;
    use std::sync::{Arc, Mutex};
    #[cfg(unix)]
    use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
    #[cfg(unix)]
    use tokio::net::UnixStream;

    #[cfg(target_os = "linux")]
    #[test]
    fn authority_successor_launch_guard_survives_parent_drop_across_exec() {
        use fs2::FileExt as _;
        use std::os::fd::AsRawFd as _;
        use std::os::unix::process::CommandExt as _;
        use std::process::{Command, Stdio};

        let temp = tempfile::tempdir().expect("launch guard tempdir");
        let path = temp.path().join("intent.lock");
        let leader = std::fs::OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(false)
            .open(&path)
            .expect("open leader launch guard");
        leader.lock_exclusive().expect("lock leader launch guard");
        let fd = leader.as_raw_fd();
        let mut command = Command::new("/bin/sh");
        command
            .args(["-c", "sleep 30"])
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null());
        unsafe {
            command.pre_exec(move || {
                let flags = libc::fcntl(fd, libc::F_GETFD);
                if flags < 0 {
                    return Err(std::io::Error::last_os_error());
                }
                if libc::fcntl(fd, libc::F_SETFD, flags & !libc::FD_CLOEXEC) < 0 {
                    return Err(std::io::Error::last_os_error());
                }
                Ok(())
            });
        }
        let mut child = command
            .spawn()
            .expect("spawn child holding inherited launch guard");
        drop(leader);

        let challenger = std::fs::OpenOptions::new()
            .write(true)
            .open(&path)
            .expect("open challenger launch guard");
        let error = challenger
            .try_lock_exclusive()
            .expect_err("child must retain the launch guard after parent drop");
        assert_eq!(error.kind(), std::io::ErrorKind::WouldBlock);

        child.kill().expect("kill launch-guard child");
        child.wait().expect("join launch-guard child");
        challenger
            .try_lock_exclusive()
            .expect("launch guard must release after child exit");
    }

    #[cfg(target_os = "linux")]
    fn owner_helper_install_context_for_test(
        selected_prefix: &std::path::Path,
    ) -> transport_api_types::InstallBootstrapContextCarrierV1 {
        let (principal, _) = crate::execution::install_bootstrap::current_unix_principal_and_home()
            .expect("current Unix principal");
        let transport_api_types::PlatformPrincipalV1::Unix { account, uid } = principal else {
            panic!("expected Unix principal");
        };
        let context = transport_api_types::InstallBootstrapContextV1::new_unix(
            selected_prefix
                .to_str()
                .expect("UTF-8 selected install prefix"),
            &account,
            uid,
        )
        .expect("install bootstrap context");
        transport_api_types::InstallBootstrapContextCarrierV1::from_context(context)
            .expect("install bootstrap carrier")
    }

    #[cfg(target_os = "linux")]
    #[test]
    #[serial_test::serial]
    fn authority_owner_helper_child_args_carry_exact_checked_install_context() {
        let _authority_env = crate::execution::AuthorityEnvTestGuard::preserve();
        let temp = tempfile::tempdir().expect("install context tempdir");
        let carrier = owner_helper_install_context_for_test(&temp.path().join("selected-prefix"));
        let encoded = crate::execution::install_bootstrap::install_bootstrap_projections(&carrier)
            .expect("install bootstrap projections");
        let mut command = std::process::Command::new("/bin/true");

        super::append_checked_install_bootstrap_context_arg(&mut command)
            .expect("checked install bootstrap child argument");

        let args = command
            .get_args()
            .map(|arg| arg.to_string_lossy().into_owned())
            .collect::<Vec<_>>();
        assert_eq!(
            args,
            vec![
                "--install-bootstrap-context-v1".to_string(),
                encoded.clone(),
            ]
        );
        let child_carrier = transport_api_types::InstallBootstrapContextCarrierV1::decode(&args[1])
            .expect("child carrier authenticates");
        assert_eq!(child_carrier, carrier);
        assert_eq!(
            child_carrier.context.selected_host_prefix,
            temp.path().join("selected-prefix").display().to_string()
        );
        assert_eq!(
            child_carrier.context.intended_host_principal,
            carrier.context.intended_host_principal
        );
    }

    #[cfg(target_os = "linux")]
    #[test]
    #[serial_test::serial]
    fn authority_owner_helper_child_args_reject_conflicting_projection_before_spawn() {
        let _authority_env = crate::execution::AuthorityEnvTestGuard::preserve();
        let temp = tempfile::tempdir().expect("install context tempdir");
        let carrier = owner_helper_install_context_for_test(&temp.path().join("selected-prefix"));
        crate::execution::install_bootstrap::install_bootstrap_projections(&carrier)
            .expect("install bootstrap projections");
        std::env::set_var("SUBSTRATE_INSTALL_PRIMARY_UID", "4294967295");
        let mut command = std::process::Command::new("/bin/true");

        let error = super::append_checked_install_bootstrap_context_arg(&mut command)
            .expect_err("conflicting projection must fail closed");

        assert!(format!("{error:#}")
            .contains("install bootstrap environment projection is missing or conflicting"));
        assert_eq!(command.get_args().count(), 0);
    }

    #[cfg(target_os = "linux")]
    #[test]
    #[serial_test::serial]
    fn authority_owner_helper_child_args_reject_malformed_carrier_before_spawn() {
        let _authority_env = crate::execution::AuthorityEnvTestGuard::preserve();
        std::env::set_var("SUBSTRATE_INSTALL_BOOTSTRAP_CONTEXT_V1", "not%canonical");
        let mut command = std::process::Command::new("/bin/true");

        let error = super::append_checked_install_bootstrap_context_arg(&mut command)
            .expect_err("malformed carrier must fail closed");

        assert!(format!("{error:#}").contains("checked install bootstrap projection is invalid"));
        assert_eq!(command.get_args().count(), 0);
    }

    #[cfg(target_os = "linux")]
    #[test]
    #[serial_test::serial]
    fn authority_owner_helper_child_args_do_not_fabricate_installed_authority() {
        let _authority_env = crate::execution::AuthorityEnvTestGuard::preserve();
        std::env::remove_var("SUBSTRATE_INSTALL_BOOTSTRAP_CONTEXT_V1");
        let mut command = std::process::Command::new("/bin/true");

        let error = super::append_checked_install_bootstrap_context_arg(&mut command)
            .expect_err("missing carrier must fail closed");

        assert!(format!("{error:#}").contains("checked install bootstrap projection is missing"));
        assert_eq!(command.get_args().count(), 0);
    }

    fn with_store(test: impl FnOnce(&AgentRuntimeStateStore)) {
        let authority_env = crate::execution::AuthorityEnvTestGuard::preserve();
        let safe_parent = std::env::var_os("XDG_RUNTIME_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|| {
                PathBuf::from(std::env::var_os("HOME").expect("tests require HOME")).join(".cache")
            });
        std::fs::create_dir_all(&safe_parent).expect("create control test authority parent");
        let temp = tempfile::tempdir_in(safe_parent).expect("secure control test tempdir");
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt as _;

            std::fs::set_permissions(temp.path(), std::fs::Permissions::from_mode(0o700))
                .expect("secure control test tempdir mode");
        }
        authority_env.install_home(temp.path());
        let store = AgentRuntimeStateStore::new().expect("state store");
        test(&store);
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
                request_key_sha256: None,
                prompt_sha256: None,
                public_backend_id: None,
                public_scope: None,
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
                request_key_sha256: None,
                prompt_sha256: None,
                public_backend_id: None,
                public_scope: None,
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

    #[cfg(unix)]
    #[test]
    fn authority_managed_private_stop_wire_is_closed_and_versioned() {
        let exact = serde_json::json!({
            "version": 2,
            "action": "stop",
            "hsa_stop": {
                "schema_version": 1,
                "authority_store_id": "store-1",
                "bootstrap_home": {
                    "physical_path": "/private/home",
                    "physical_identity": {
                        "kind": "Linux",
                        "value": { "device_id": 7, "inode": 11 }
                    }
                },
                "orchestration_session_id": "session-1",
                "intent_id": "stop-intent-1",
                "request_id": "stop-request-1",
                "payload_commitment": {
                    "kind": "CanonicalSha256",
                    "value": {
                        "digest_hex": "dddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd"
                    }
                },
                "authority_revision": 3,
                "authority_record_commitment": {
                    "kind": "CanonicalSha256",
                    "value": {
                        "digest_hex": "eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee"
                    }
                },
                "authoritative_participant_id": "participant-1"
            }
        });
        assert!(matches!(
            parse_private_stop_request(&exact.to_string()).unwrap(),
            PrivateStopRequestPayloadV1::AuthorityManaged(delivery)
                if delivery.intent_id == "stop-intent-1"
                    && delivery.authoritative_participant_id == "participant-1"
        ));

        let mut missing_delivery = exact.clone();
        missing_delivery.as_object_mut().unwrap().remove("hsa_stop");
        assert!(parse_private_stop_request(&missing_delivery.to_string()).is_err());

        let mut extra_top_level = exact.clone();
        extra_top_level
            .as_object_mut()
            .unwrap()
            .insert("unexpected".into(), serde_json::Value::Bool(true));
        assert!(parse_private_stop_request(&extra_top_level.to_string()).is_err());

        let mut extra_delivery_field = exact.clone();
        extra_delivery_field["hsa_stop"]["unexpected"] = serde_json::Value::Bool(true);
        assert!(parse_private_stop_request(&extra_delivery_field.to_string()).is_err());

        let mut wrong_action = exact;
        wrong_action["action"] = serde_json::Value::String("cancel".into());
        assert!(parse_private_stop_request(&wrong_action.to_string()).is_err());
    }

    #[cfg(unix)]
    #[tokio::test(flavor = "current_thread")]
    async fn authority_managed_private_stop_silent_peer_is_bounded() {
        let temp = tempfile::tempdir().expect("private HSA Stop tempdir");
        let path = temp.path().join("silent-owner.sock");
        let listener = tokio::net::UnixListener::bind(&path).expect("bind silent owner");
        let server = tokio::spawn(async move {
            let (_stream, _) = listener.accept().await.expect("accept Stop delivery");
            tokio::time::sleep(std::time::Duration::from_secs(1)).await;
        });
        let delivery = authority_managed_stop_delivery_fixture();

        let result = tokio::time::timeout(
            std::time::Duration::from_millis(500),
            super::request_private_authority_managed_stop_with_response_timeout(
                &path,
                &delivery,
                std::time::Duration::from_millis(50),
            ),
        )
        .await;
        server.abort();

        let error = result
            .expect("private HSA Stop response wait must be internally bounded")
            .expect_err("silent owner must fail closed");
        assert!(
            error
                .to_string()
                .contains("timed out waiting for exact private HSA Stop response"),
            "unexpected error: {error:#}"
        );
    }

    #[cfg(unix)]
    #[tokio::test(flavor = "current_thread")]
    async fn authority_managed_private_stop_transport_failures_remain_distinct() {
        let temp = tempfile::tempdir().expect("private HSA Stop tempdir");
        let delivery = authority_managed_stop_delivery_fixture();

        let missing = temp.path().join("missing-owner.sock");
        let missing_error = super::request_private_authority_managed_stop_with_response_timeout(
            &missing,
            &delivery,
            std::time::Duration::from_millis(50),
        )
        .await
        .expect_err("missing owner transport must fail closed");
        assert!(
            missing_error.to_string().contains("missing_transport:"),
            "unexpected missing transport error: {missing_error:#}"
        );

        let refused = temp.path().join("refused-owner.sock");
        let stale_listener =
            std::os::unix::net::UnixListener::bind(&refused).expect("bind refused socket");
        drop(stale_listener);
        let refused_error = super::request_private_authority_managed_stop_with_response_timeout(
            &refused,
            &delivery,
            std::time::Duration::from_millis(50),
        )
        .await
        .expect_err("refused owner transport must fail closed");
        assert!(
            refused_error.to_string().contains("refused_transport:"),
            "unexpected refused transport error: {refused_error:#}"
        );

        let protocol = temp.path().join("protocol-owner.sock");
        let listener = tokio::net::UnixListener::bind(&protocol).expect("bind protocol owner");
        let server = tokio::spawn(async move {
            let (mut stream, _) = listener.accept().await.expect("accept Stop delivery");
            stream
                .write_all(b"not-json\n")
                .await
                .expect("write invalid Stop response");
        });
        let protocol_error = super::request_private_authority_managed_stop_with_response_timeout(
            &protocol,
            &delivery,
            std::time::Duration::from_millis(50),
        )
        .await
        .expect_err("malformed owner response must fail closed");
        server.await.expect("protocol owner task");
        assert!(
            protocol_error
                .to_string()
                .contains("failed to decode private HSA Stop transport response"),
            "unexpected protocol error: {protocol_error:#}"
        );
    }

    #[cfg(unix)]
    fn authority_managed_stop_delivery_fixture(
    ) -> crate::execution::agent_runtime::host_session_authority::stop::HostSessionStopDeliveryV1
    {
        serde_json::from_value(serde_json::json!({
            "schema_version": 1,
            "authority_store_id": "store-1",
            "bootstrap_home": {
                "physical_path": "/private/home",
                "physical_identity": {
                    "kind": "Linux",
                    "value": { "device_id": 7, "inode": 11 }
                }
            },
            "orchestration_session_id": "session-1",
            "intent_id": "stop-intent-1",
            "request_id": "stop-request-1",
            "payload_commitment": {
                "kind": "CanonicalSha256",
                "value": {
                    "digest_hex": "dddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd"
                }
            },
            "authority_revision": 3,
            "authority_record_commitment": {
                "kind": "CanonicalSha256",
                "value": {
                    "digest_hex": "eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee"
                }
            },
            "authoritative_participant_id": "participant-1"
        }))
        .expect("valid Stop delivery")
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

    #[cfg(unix)]
    #[test]
    fn prompt_event_text_keeps_top_level_text_passthrough() {
        let text = prompt_event_text(&serde_json::json!({
            "text": "stdout chunk\n",
        }));

        assert_eq!(text, "stdout chunk\n");
    }

    #[cfg(unix)]
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

    #[cfg(unix)]
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
    #[test]
    fn strict_start_terminal_renderer_propagates_delivery_write_and_flush_failures() {
        let completed = PublicPromptEnvelope::Completed {
            version: 1,
            action: PublicPromptAction::Start,
            orchestration_session_id: "sess-start-delivery".to_string(),
            backend_id: "cli:codex".to_string(),
            participant_id: Some("participant-start-delivery".to_string()),
            turn_outcome: "success".to_string(),
            session_posture: PublicSessionPosture::DetachedReattachable,
            state: "parked_resumable".to_string(),
            warnings: Vec::new(),
        };
        let mut renderer = PublicPromptRenderer::new(false);
        let mut failed_stdout = PromptRenderBuffer {
            stdout_write_error: true,
            ..PromptRenderBuffer::default()
        };
        assert!(renderer
            .render_start_terminal_strict_with_output(&completed, &mut failed_stdout)
            .is_err());

        let failed = failed_prompt_envelope(
            "runtime",
            "owner_unreachable",
            "exact inaugural Start failure",
        );
        let mut failed_stderr = PromptRenderBuffer {
            stderr_write_error: true,
            ..PromptRenderBuffer::default()
        };
        assert!(renderer
            .render_start_terminal_strict_with_output(&failed, &mut failed_stderr)
            .is_err());

        let mut failed_stdout_flush = PromptRenderBuffer {
            stdout_flush_error: true,
            ..PromptRenderBuffer::default()
        };
        assert!(renderer
            .render_start_terminal_strict_with_output(&completed, &mut failed_stdout_flush)
            .is_err());

        let mut json_renderer = PublicPromptRenderer::new(true);
        let mut failed_json_flush = PromptRenderBuffer {
            stdout_flush_error: true,
            ..PromptRenderBuffer::default()
        };
        assert!(json_renderer
            .render_start_terminal_strict_with_output(&completed, &mut failed_json_flush)
            .is_err());
    }

    #[cfg(unix)]
    #[test]
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

        let mut output = PromptRenderBuffer::default();
        let mut renderer = PublicPromptRenderer::new(false);
        renderer
            .render_with_output(&envelope, &mut output)
            .expect("render structured fallback");

        assert_eq!(
            output.stdout,
            b"[codex] task_progress: fields=alpha, beta, gamma (+1 more)\n"
        );
        assert_eq!(output.stderr, b"");
        assert_eq!(output.stdout_flushes, 1);
        assert_eq!(output.stderr_flushes, 0);
        assert!(
            !output.stdout.contains(&b'{')
                && !output
                    .stdout
                    .windows(b"queued".len())
                    .any(|bytes| bytes == b"queued"),
            "fallback must stay bounded and avoid dumping raw nested payloads: {:?}",
            String::from_utf8_lossy(&output.stdout)
        );

        let completed = PublicPromptEnvelope::Completed {
            version: 1,
            action: PublicPromptAction::Turn,
            orchestration_session_id: "sess-render".to_string(),
            backend_id: "cli:codex".to_string(),
            participant_id: Some("ash-render".to_string()),
            turn_outcome: "success".to_string(),
            session_posture: PublicSessionPosture::DetachedReattachable,
            state: "ready".to_string(),
            warnings: Vec::new(),
        };
        let mut completed_output = PromptRenderBuffer::default();
        renderer
            .render_with_output(&completed, &mut completed_output)
            .expect("render completed envelope");
        assert_eq!(
            completed_output.stdout,
            b"action=turn orchestration_session_id=sess-render backend_id=cli:codex participant_id=ash-render turn_outcome=success session_posture=detached_reattachable\n"
        );
        assert_eq!(completed_output.stderr, b"");
        assert_eq!(completed_output.stdout_flushes, 1);

        let mut json_output = PromptRenderBuffer::default();
        let mut json_renderer = PublicPromptRenderer::new(true);
        json_renderer
            .render_with_output(&completed, &mut json_output)
            .expect("render JSON envelope");
        let mut expected_json = serde_json::to_vec(&completed).expect("serialize JSON envelope");
        expected_json.push(b'\n');
        assert_eq!(json_output.stdout, expected_json);
        assert_eq!(json_output.stderr, b"");
        assert_eq!(json_output.stdout_flushes, 1);

        let mut ignored_write_error = PromptRenderBuffer {
            stdout_write_error: true,
            ..PromptRenderBuffer::default()
        };
        renderer
            .render_with_output(&envelope, &mut ignored_write_error)
            .expect("non-JSON renderer preserves ignored write errors");
        assert_eq!(ignored_write_error.stdout, b"");
        assert_eq!(ignored_write_error.stdout_flushes, 1);

        let mut propagated_write_error = PromptRenderBuffer {
            stdout_write_error: true,
            ..PromptRenderBuffer::default()
        };
        let json_error = json_renderer
            .render_with_output(&completed, &mut propagated_write_error)
            .expect_err("JSON renderer preserves propagated write errors");
        assert!(json_error
            .to_string()
            .contains("failed to render prompt envelope"));
        assert_eq!(propagated_write_error.stdout_flushes, 0);
    }

    #[cfg(unix)]
    #[test]
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

        let mut output = PromptRenderBuffer::default();
        let mut renderer = PublicPromptRenderer::new(false);
        renderer
            .render_with_output(&envelope, &mut output)
            .expect("render structured stderr fallback");

        assert_eq!(
            output.stderr,
            b"[codex] task_progress: fields=alpha, beta, gamma (+1 more)\n"
        );
        assert_eq!(output.stdout, b"");
        assert_eq!(output.stderr_flushes, 1);
        assert_eq!(output.stdout_flushes, 0);
        assert!(
            !output.stderr.contains(&b'{')
                && !output
                    .stderr
                    .windows(b"queued".len())
                    .any(|bytes| bytes == b"queued"),
            "fallback must stay bounded and avoid dumping raw nested payloads: {:?}",
            String::from_utf8_lossy(&output.stderr)
        );

        let mut warning_output = PromptRenderBuffer::default();
        renderer
            .render_with_output(
                &PublicPromptEnvelope::Warning {
                    version: 1,
                    message: "warning text".to_string(),
                },
                &mut warning_output,
            )
            .expect("render warning envelope");
        assert_eq!(warning_output.stdout, b"");
        assert_eq!(warning_output.stderr, b"warning text\n");
        assert_eq!(warning_output.stderr_flushes, 1);

        let mut failed_output = PromptRenderBuffer::default();
        renderer
            .render_with_output(
                &PublicPromptEnvelope::Failed {
                    version: 1,
                    terminal: true,
                    stage: "render".to_string(),
                    error_code: "render_failed".to_string(),
                    message: "failure text".to_string(),
                },
                &mut failed_output,
            )
            .expect("render failed envelope");
        assert_eq!(failed_output.stdout, b"");
        assert_eq!(failed_output.stderr, b"failure text\n");
        assert_eq!(failed_output.stderr_flushes, 1);

        let mut ignored_stderr_error = PromptRenderBuffer {
            stderr_write_error: true,
            ..PromptRenderBuffer::default()
        };
        renderer
            .render_with_output(&envelope, &mut ignored_stderr_error)
            .expect("non-JSON stderr renderer preserves ignored write errors");
        assert_eq!(ignored_stderr_error.stderr, b"");
        assert_eq!(ignored_stderr_error.stderr_flushes, 1);
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

            let (waiter_contended_tx, waiter_contended_rx) = std::sync::mpsc::channel();
            let joined = std::thread::scope(|scope| {
                let waiter = scope.spawn(|| {
                    assert!(
                        super::try_acquire_hidden_owner_helper_attach_launch_guard(store, &plan)
                            .expect("probe inflight attach launch guard")
                            .is_none(),
                        "waiter must observe the retained inflight attach launch before joining"
                    );
                    waiter_contended_tx
                        .send(())
                        .expect("publish inflight attach contention");
                    super::wait_for_inflight_hidden_owner_helper_attach_launch(store, &plan)
                        .expect("detached live owner should not satisfy attach join")
                });
                waiter_contended_rx
                    .recv()
                    .expect("waiter reached inflight attach acquisition");
                drop(guard);
                waiter.join().expect("attach join waiter should not panic")
            });

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

    #[test]
    fn host_execution_episode_v1_canonical_tables_are_complete() {
        use super::{
            HostExecutionEpisodeKindV1, HostExecutionEpisodeTransportStatusV1,
            PrivateTransportAvailabilityV1,
        };

        let kinds = [
            HostExecutionEpisodeKindV1::ReplAttachedEpisode,
            HostExecutionEpisodeKindV1::HiddenOwnerHelperStartEpisode,
            HostExecutionEpisodeKindV1::HiddenOwnerHelperAttachEpisode,
            HostExecutionEpisodeKindV1::HiddenOwnerHelperResumeOneTurnEpisode,
            HostExecutionEpisodeKindV1::RuntimeToolboxEpisode,
            HostExecutionEpisodeKindV1::SyntheticOrRecoveredEpisode,
        ];
        assert_eq!(
            kinds.map(|kind| serde_json::to_string(&kind).unwrap()),
            [
                "\"ReplAttachedEpisode\"",
                "\"HiddenOwnerHelperStartEpisode\"",
                "\"HiddenOwnerHelperAttachEpisode\"",
                "\"HiddenOwnerHelperResumeOneTurnEpisode\"",
                "\"RuntimeToolboxEpisode\"",
                "\"SyntheticOrRecoveredEpisode\"",
            ]
        );

        let cases = [
            (
                PrivateTransportAvailabilityV1::Available,
                true,
                HostExecutionEpisodeTransportStatusV1::Available,
            ),
            (
                PrivateTransportAvailabilityV1::Missing,
                true,
                HostExecutionEpisodeTransportStatusV1::UnavailableButDurableAuthorityExists,
            ),
            (
                PrivateTransportAvailabilityV1::Refused,
                true,
                HostExecutionEpisodeTransportStatusV1::UnavailableButDurableAuthorityExists,
            ),
            (
                PrivateTransportAvailabilityV1::Failed,
                true,
                HostExecutionEpisodeTransportStatusV1::UnavailableButDurableAuthorityExists,
            ),
            (
                PrivateTransportAvailabilityV1::Missing,
                false,
                HostExecutionEpisodeTransportStatusV1::UnavailableAndNoAuthoritativeRoute,
            ),
            (
                PrivateTransportAvailabilityV1::Stale,
                true,
                HostExecutionEpisodeTransportStatusV1::StaleOrOrphaned,
            ),
        ];
        for (availability, durable_authority_exists, expected) in cases {
            assert_eq!(
                super::classify_episode_transport(availability, durable_authority_exists),
                expected
            );
        }

        let statuses = [
            HostExecutionEpisodeTransportStatusV1::Available,
            HostExecutionEpisodeTransportStatusV1::UnavailableButDurableAuthorityExists,
            HostExecutionEpisodeTransportStatusV1::UnavailableAndNoAuthoritativeRoute,
            HostExecutionEpisodeTransportStatusV1::StaleOrOrphaned,
        ];
        assert_eq!(
            statuses.map(|status| serde_json::to_string(&status).unwrap()),
            [
                "\"Available\"",
                "\"UnavailableButDurableAuthorityExists\"",
                "\"UnavailableAndNoAuthoritativeRoute\"",
                "\"StaleOrOrphaned\"",
            ]
        );
    }

    #[test]
    fn host_execution_episode_v1_requires_exact_binding() {
        use super::{
            EpisodeExitObservationV1, HostExecutionEpisodeBindingV1, HostExecutionEpisodeKindV1,
            HostExecutionEpisodeTransportStatusV1, HostExecutionEpisodeV1, ProcessRefV1,
        };
        use crate::execution::agent_runtime::host_session_authority::schema::TimestampV1;

        let started_at = TimestampV1::parse("2026-08-31T12:00:00.000000000Z").unwrap();
        let episode = HostExecutionEpisodeV1 {
            schema_version: 1,
            episode_id: "hee_exact".to_string(),
            kind: HostExecutionEpisodeKindV1::RuntimeToolboxEpisode,
            orchestration_session_id: "session_exact".to_string(),
            observed_authority_revision: 7,
            backend_id: Some("cli:codex".to_string()),
            process_ref: Some(ProcessRefV1 { pid: 42 }),
            transport_status: HostExecutionEpisodeTransportStatusV1::Available,
            started_at: started_at.clone(),
            last_heartbeat_at: Some(started_at.clone()),
            ended_at: None,
            exit_observation: None::<EpisodeExitObservationV1>,
        };
        let exact = HostExecutionEpisodeBindingV1 {
            authority_store_id: "store_exact".to_string(),
            orchestration_session_id: "session_exact".to_string(),
            participant_id: "participant_exact".to_string(),
            episode_id: "hee_exact".to_string(),
            authority_revision: 7,
        };
        episode.validate_binding(&exact).unwrap();

        for stale in [
            HostExecutionEpisodeBindingV1 {
                authority_revision: 8,
                ..exact.clone()
            },
            HostExecutionEpisodeBindingV1 {
                episode_id: "hee_competing".to_string(),
                ..exact.clone()
            },
            HostExecutionEpisodeBindingV1 {
                orchestration_session_id: "session_substituted".to_string(),
                ..exact.clone()
            },
        ] {
            assert!(episode.validate_binding(&stale).is_err());
        }
    }

    #[test]
    fn host_execution_episode_observations_are_local_revision_bound_and_idempotent() {
        use super::{
            EpisodeExitObservationV1, HostExecutionEpisodeBindingV1, HostExecutionEpisodeKindV1,
            HostExecutionEpisodeObservationV1, HostExecutionEpisodeTransportStatusV1,
            HostExecutionEpisodeV1, PrivateTransportAvailabilityV1, ProcessRefV1,
        };
        use crate::execution::agent_runtime::host_session_authority::schema::TimestampV1;

        let ts = |value| TimestampV1::parse(value).unwrap();
        let mut episode = HostExecutionEpisodeV1 {
            schema_version: 1,
            episode_id: "hee_observations".to_string(),
            kind: HostExecutionEpisodeKindV1::ReplAttachedEpisode,
            orchestration_session_id: "session_observations".to_string(),
            observed_authority_revision: 9,
            backend_id: Some("cli:codex-host".to_string()),
            process_ref: None,
            transport_status: HostExecutionEpisodeTransportStatusV1::Available,
            started_at: ts("2026-08-31T12:00:00.000000000Z"),
            last_heartbeat_at: None,
            ended_at: None,
            exit_observation: None,
        };
        let binding = HostExecutionEpisodeBindingV1 {
            authority_store_id: "store_observations".to_string(),
            orchestration_session_id: episode.orchestration_session_id.clone(),
            participant_id: "participant_observations".to_string(),
            episode_id: episode.episode_id.clone(),
            authority_revision: episode.observed_authority_revision,
        };

        let observation_table = [
            HostExecutionEpisodeObservationV1::Process(ProcessRefV1 { pid: 42 }),
            HostExecutionEpisodeObservationV1::Heartbeat {
                observed_at: ts("2026-08-31T12:00:01.000000000Z"),
            },
            HostExecutionEpisodeObservationV1::Readiness { ready: true },
            HostExecutionEpisodeObservationV1::Prompt { accepted: true },
            HostExecutionEpisodeObservationV1::Handle { present: true },
            HostExecutionEpisodeObservationV1::Stream { active: true },
            HostExecutionEpisodeObservationV1::Endpoint {
                availability: PrivateTransportAvailabilityV1::Missing,
            },
            HostExecutionEpisodeObservationV1::Timeout { timed_out: true },
            HostExecutionEpisodeObservationV1::Exit(EpisodeExitObservationV1 {
                observed_at: ts("2026-08-31T12:00:02.000000000Z"),
                exit_code: Some(0),
                signal: None,
            }),
        ];
        assert_eq!(
            observation_table.map(|observation| {
                serde_json::to_value(observation).unwrap()["observation"]
                    .as_str()
                    .unwrap()
                    .to_string()
            }),
            [
                "Process",
                "Heartbeat",
                "Readiness",
                "Prompt",
                "Handle",
                "Stream",
                "Endpoint",
                "Timeout",
                "Exit",
            ]
        );

        let process = HostExecutionEpisodeObservationV1::Process(ProcessRefV1 { pid: 42 });
        assert!(episode
            .record_local_observation(&binding, process.clone())
            .unwrap());
        assert!(!episode.record_local_observation(&binding, process).unwrap());
        assert!(episode
            .record_local_observation(
                &binding,
                HostExecutionEpisodeObservationV1::Process(ProcessRefV1 { pid: 43 }),
            )
            .is_err());
        assert!(episode
            .record_local_observation(
                &binding,
                HostExecutionEpisodeObservationV1::Process(ProcessRefV1 { pid: 0 }),
            )
            .is_err());

        let heartbeat = HostExecutionEpisodeObservationV1::Heartbeat {
            observed_at: ts("2026-08-31T12:00:01.000000000Z"),
        };
        assert!(episode
            .record_local_observation(&binding, heartbeat.clone())
            .unwrap());
        assert!(!episode
            .record_local_observation(&binding, heartbeat)
            .unwrap());
        for observation in [
            HostExecutionEpisodeObservationV1::Readiness { ready: true },
            HostExecutionEpisodeObservationV1::Prompt { accepted: true },
            HostExecutionEpisodeObservationV1::Handle { present: true },
            HostExecutionEpisodeObservationV1::Stream { active: true },
            HostExecutionEpisodeObservationV1::Timeout { timed_out: true },
        ] {
            assert!(!episode
                .record_local_observation(&binding, observation)
                .unwrap());
        }
        let endpoint = HostExecutionEpisodeObservationV1::Endpoint {
            availability: PrivateTransportAvailabilityV1::Missing,
        };
        assert!(episode
            .record_local_observation(&binding, endpoint.clone())
            .unwrap());
        assert!(!episode
            .record_local_observation(&binding, endpoint)
            .unwrap());

        let exit = HostExecutionEpisodeObservationV1::Exit(EpisodeExitObservationV1 {
            observed_at: ts("2026-08-31T12:00:02.000000000Z"),
            exit_code: Some(0),
            signal: None,
        });
        assert!(episode
            .record_local_observation(&binding, exit.clone())
            .unwrap());
        assert!(!episode.record_local_observation(&binding, exit).unwrap());
        assert!(episode
            .record_local_observation(
                &binding,
                HostExecutionEpisodeObservationV1::Exit(EpisodeExitObservationV1 {
                    observed_at: ts("2026-08-31T12:00:03.000000000Z"),
                    exit_code: Some(1),
                    signal: None,
                }),
            )
            .is_err());

        let before_stale = episode.clone();
        let stale_binding = HostExecutionEpisodeBindingV1 {
            authority_revision: binding.authority_revision + 1,
            ..binding.clone()
        };
        assert!(episode
            .record_local_observation(
                &stale_binding,
                HostExecutionEpisodeObservationV1::Endpoint {
                    availability: PrivateTransportAvailabilityV1::Available,
                },
            )
            .is_err());
        assert_eq!(episode, before_stale);
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn private_transport_availability_classifies_live_missing_refused_failed_and_stale() {
        use super::PrivateTransportAvailabilityV1;
        use std::os::unix::fs::{symlink, PermissionsExt};
        use std::os::unix::net::UnixListener;

        let temp = tempfile::tempdir().expect("transport availability tempdir");
        let uid = unsafe { libc::geteuid() };
        let path = temp.path().join("episode.sock");
        assert_eq!(
            super::private_transport_availability(&path, uid),
            PrivateTransportAvailabilityV1::Missing
        );

        let listener = UnixListener::bind(&path).expect("bind live episode socket");
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600))
            .expect("secure live episode socket");
        assert_eq!(
            super::private_transport_availability(&path, uid),
            PrivateTransportAvailabilityV1::Available
        );
        drop(listener);
        assert_eq!(
            super::private_transport_availability(&path, uid),
            PrivateTransportAvailabilityV1::Refused
        );

        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o660))
            .expect("make stale endpoint permissive");
        assert_eq!(
            super::private_transport_availability(&path, uid),
            PrivateTransportAvailabilityV1::Stale
        );
        std::fs::remove_file(&path).expect("remove stale endpoint");
        let target = temp.path().join("target");
        std::fs::write(&target, b"target").expect("write symlink target");
        symlink(&target, &path).expect("create stale endpoint symlink");
        assert_eq!(
            super::private_transport_availability(&path, uid),
            PrivateTransportAvailabilityV1::Stale
        );

        let too_long = temp.path().join("x".repeat(300));
        assert_eq!(
            super::private_transport_availability(&too_long, uid),
            PrivateTransportAvailabilityV1::Failed
        );
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn durable_private_episode_paths_are_injective_revision_bound_and_length_safe() {
        use super::HostExecutionEpisodeBindingV1;
        use std::path::Path;

        let binding = HostExecutionEpisodeBindingV1 {
            authority_store_id: "store/alpha".to_string(),
            orchestration_session_id: "session/a".to_string(),
            participant_id: "participant:a".to_string(),
            episode_id: "episode:a".to_string(),
            authority_revision: 17,
        };
        let exact = super::private_episode_transport_path_for_binding(
            Path::new("/short"),
            1000,
            &binding,
            "stop",
            "stop",
        );
        assert_eq!(
            exact,
            super::private_episode_transport_path_for_binding(
                Path::new("/short"),
                1000,
                &binding,
                "stop",
                "stop",
            )
        );

        for distinct in [
            HostExecutionEpisodeBindingV1 {
                authority_store_id: "store_alpha".to_string(),
                ..binding.clone()
            },
            HostExecutionEpisodeBindingV1 {
                orchestration_session_id: "session_a".to_string(),
                ..binding.clone()
            },
            HostExecutionEpisodeBindingV1 {
                participant_id: "participant_a".to_string(),
                ..binding.clone()
            },
            HostExecutionEpisodeBindingV1 {
                episode_id: "episode_a".to_string(),
                ..binding.clone()
            },
            HostExecutionEpisodeBindingV1 {
                authority_revision: 18,
                ..binding.clone()
            },
        ] {
            assert_ne!(
                exact,
                super::private_episode_transport_path_for_binding(
                    Path::new("/short"),
                    1000,
                    &distinct,
                    "stop",
                    "stop",
                )
            );
        }
        assert_ne!(
            exact,
            super::private_episode_transport_path_for_binding(
                Path::new("/short"),
                1001,
                &binding,
                "stop",
                "stop",
            )
        );
        assert_ne!(
            exact,
            super::private_episode_transport_path_for_binding(
                Path::new("/other"),
                1000,
                &binding,
                "stop",
                "stop",
            )
        );
        assert_ne!(
            exact,
            super::private_episode_transport_path_for_binding(
                Path::new("/short"),
                1000,
                &binding,
                "prompt",
                "prompt",
            )
        );

        let long_home = PathBuf::from(format!("/{}", "long-home/".repeat(32)));
        let fallback = super::private_episode_transport_path_for_binding(
            &long_home, 1000, &binding, "stop", "stop",
        );
        assert!(fallback.as_os_str().len() <= super::PRIVATE_STOP_UNIX_PATH_MAX);
        assert!(fallback.starts_with("/tmp/substrate-host-episodes-u1000"));
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn durable_private_episode_publication_rejects_collisions_and_unsafe_paths() {
        use std::os::unix::fs::{symlink, PermissionsExt};
        use std::os::unix::net::UnixListener;

        let temp = tempfile::tempdir().expect("private episode namespace tempdir");
        let root = temp.path().join("namespace");
        let path = root.join("store/stop/episode.stop.sock");
        let uid = unsafe { libc::geteuid() };

        super::prepare_private_episode_endpoint_for_bind(&path, uid, &root)
            .expect("prepare exact private episode namespace");
        let listener = UnixListener::bind(&path).expect("bind active private episode publisher");
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600))
            .expect("secure active private episode publisher");
        let active_error = super::prepare_private_episode_endpoint_for_bind(&path, uid, &root)
            .expect_err("an active competing publisher must not be replaced");
        assert!(active_error
            .to_string()
            .contains("active competing publisher"));

        let stale_identity = super::private_episode_endpoint_identity(&path)
            .expect("capture stale endpoint identity");
        drop(listener);
        super::prepare_private_episode_endpoint_for_bind(&path, uid, &root)
            .expect("an exact refused endpoint may be reclaimed");
        assert!(!path.exists());

        let replacement = UnixListener::bind(&path).expect("bind replacement endpoint");
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600))
            .expect("secure replacement endpoint");
        let cas_error = super::remove_private_episode_endpoint_if_same(&path, stale_identity)
            .expect_err("stale cleanup identity must not remove a replacement publisher");
        assert!(cas_error.to_string().contains("competing publisher"));
        assert!(path.exists());
        drop(replacement);
        std::fs::remove_file(&path).expect("remove replacement endpoint");

        let target = root.join("symlink-target");
        std::fs::write(&target, b"target").expect("write symlink target");
        symlink(&target, &path).expect("create endpoint symlink");
        assert!(super::prepare_private_episode_endpoint_for_bind(&path, uid, &root).is_err());
        std::fs::remove_file(&path).expect("remove endpoint symlink");

        let permissive = UnixListener::bind(&path).expect("bind permissive endpoint");
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o660))
            .expect("make endpoint permissions unsafe");
        drop(permissive);
        assert!(super::prepare_private_episode_endpoint_for_bind(&path, uid, &root).is_err());
        std::fs::remove_file(&path).expect("remove permissive endpoint");

        assert!(
            super::prepare_private_episode_endpoint_for_bind(&path, uid.saturating_add(1), &root)
                .is_err(),
            "wrong ownership identity must fail closed"
        );
        let store_dir = root.join("store");
        std::fs::set_permissions(&store_dir, std::fs::Permissions::from_mode(0o755))
            .expect("make namespace component permissions unsafe");
        assert!(super::prepare_private_episode_endpoint_for_bind(&path, uid, &root).is_err());
    }

    #[test]
    fn toolbox_episode_paths_are_uid_store_and_exact_binding_namespaced() {
        use super::HostExecutionEpisodeBindingV1;
        use std::path::Path;

        let binding = HostExecutionEpisodeBindingV1 {
            authority_store_id: "store/alpha".to_string(),
            orchestration_session_id: "session/a".to_string(),
            participant_id: "participant:a".to_string(),
            episode_id: "episode:a".to_string(),
            authority_revision: 17,
        };
        let exact = super::toolbox_transport_path_for_episode(Path::new("/short"), 1000, &binding);
        let same = super::toolbox_transport_path_for_episode(Path::new("/short"), 1000, &binding);
        assert_eq!(exact, same);

        for distinct in [
            HostExecutionEpisodeBindingV1 {
                authority_store_id: "store_alpha".to_string(),
                ..binding.clone()
            },
            HostExecutionEpisodeBindingV1 {
                orchestration_session_id: "session_a".to_string(),
                ..binding.clone()
            },
            HostExecutionEpisodeBindingV1 {
                participant_id: "participant_a".to_string(),
                ..binding.clone()
            },
            HostExecutionEpisodeBindingV1 {
                episode_id: "episode_a".to_string(),
                ..binding.clone()
            },
        ] {
            assert_ne!(
                exact,
                super::toolbox_transport_path_for_episode(Path::new("/short"), 1000, &distinct)
            );
        }
        assert_ne!(
            exact,
            super::toolbox_transport_path_for_episode(Path::new("/short"), 1001, &binding)
        );
        assert_ne!(
            exact,
            super::toolbox_transport_path_for_episode(Path::new("/other"), 1000, &binding)
        );
        assert_eq!(
            exact,
            super::toolbox_transport_path_for_episode(
                Path::new("/short"),
                1000,
                &HostExecutionEpisodeBindingV1 {
                    authority_revision: 18,
                    ..binding.clone()
                },
            ),
            "one publisher keeps a stable logical endpoint while its exact revision binding advances"
        );

        let long_home = PathBuf::from(format!("/{}", "long-home/".repeat(32)));
        let fallback = super::toolbox_transport_path_for_episode(&long_home, 1000, &binding);
        assert!(fallback.as_os_str().len() <= super::PRIVATE_STOP_UNIX_PATH_MAX);
        assert!(fallback.starts_with("/tmp/substrate-agent-toolbox-u1000"));
    }
}
