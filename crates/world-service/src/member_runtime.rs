use agent_api::{
    AgentWrapperCancelHandle, AgentWrapperCompletion, AgentWrapperError, AgentWrapperEvent,
    AgentWrapperEventKind, AgentWrapperRunRequest,
};
use anyhow::{anyhow, Context, Result};
use axum::{
    body::{boxed, Bytes, StreamBody},
    http::StatusCode,
    response::Response,
};
use futures_util::StreamExt;
use serde::{Deserialize, Serialize};
use serde_json::json;
use sha2::{Digest as _, Sha256};
use std::{
    collections::{BTreeMap, BTreeSet, HashMap},
    convert::Infallible,
    fs,
    io::Write,
    os::unix::fs::{MetadataExt, OpenOptionsExt, PermissionsExt},
    path::Path,
    sync::{Arc, Mutex, RwLock},
};
use substrate_common::agent_events::{
    AgentEvent, AgentEventKind, MessageEventKind, NormalizedWorldWorkerEventFacetV1,
    WorldWorkerEventClassV1, WorldWorkerEventV1,
};
use tokio_stream::wrappers::UnboundedReceiverStream;
use transport_api_types::{
    E2MemberLaunchActivationCarrierV1, E2MemberLaunchKindV1, ExecuteStreamFrame,
    MemberDispatchRequestV1, MemberRuntimeBackendKindV1, MemberTurnSubmitRequestV1,
    ProcessTelemetry,
};
use world_api::SharedWorldBindingSnapshot;

use crate::e3_child_security::{E3PrivilegedChildExclusionV1, HeldNonE3PrivilegedChildLeaseV1};
use crate::gateway_runtime::{prepare_linux_world_entry_launcher, LinuxWorldPlacementContext};
use crate::member_turn_join::{
    MemberTurnJoinError, MemberTurnJoinLeader, MemberTurnJoinRegistry, MemberTurnJoinStateV1,
    MemberTurnJoinSubscription,
};
use crate::prompt_fulfillment::{
    join_failed_control, NativeSettlementOutcome, NativeTerminalObservation,
    PromptFulfillmentBridge, PromptFulfillmentRunControl,
};
use crate::runtime_replay::{
    publish_replayable_frame, RuntimeReplayPublisher, RuntimeReplayRegistry,
};
use crate::service::RuntimeEventStreamProducer;

const MEMBER_ROLE: &str = "member";
const SESSION_HANDLE_SCHEMA_V1: &str = "agent_api.session.handle.v1";
const CANCELLED_MESSAGE: &str = "cancelled";
const ADD_DIRS_EXTENSION_V1: &str = "agent_api.exec.add_dirs.v1";
const SESSION_RESUME_EXTENSION_V1: &str = "agent_api.session.resume.v1";
const SESSION_FORK_EXTENSION_V1: &str = "agent_api.session.fork.v1";
const SUBSTRATE_INTERNAL_CODEX_AUTH_SEED_HOME_ENV: &str = "SUBSTRATE_INTERNAL_CODEX_AUTH_SEED_HOME";
const SUBSTRATE_E2_TURN_ACTUAL_BINARY_ENV: &str = "SUBSTRATE_E2_TURN_ACTUAL_BINARY";
const SUBSTRATE_E2_TURN_WORLD_ENTRY_ENV: &str = "SUBSTRATE_E2_TURN_WORLD_ENTRY";
const SUBMITTED_TURN_POLICY_LAUNCHER_BYTES: &[u8] = br##"#!/bin/bash
set -euo pipefail
: "${SUBSTRATE_E2_TURN_ACTUAL_BINARY:?}"
: "${SUBSTRATE_E2_TURN_WORLD_ENTRY:?}"
: "${SUBSTRATE_LANDLOCK_HELPER_SRC:?}"
exec 3< <(
    printf 'unset SUBSTRATE_INNER_CMD SUBSTRATE_E2_TURN_ACTUAL_BINARY SUBSTRATE_E2_TURN_WORLD_ENTRY\nexec %q' "${SUBSTRATE_E2_TURN_ACTUAL_BINARY}"
    printf ' %q' "$@"
    printf '\n'
)
export SUBSTRATE_INNER_CMD='exec /bin/bash /proc/self/fd/3'
export SUBSTRATE_WORLD_ENTRY_BINARY="${SUBSTRATE_LANDLOCK_HELPER_SRC}"
exec "${SUBSTRATE_E2_TURN_WORLD_ENTRY}" "__substrate_world_landlock_exec"
"##;

const ORDINARY_HOME_POLICY_LAUNCHER_BYTES: &[u8] = br##"#!/bin/bash
set -euo pipefail
: "${SUBSTRATE_ORDINARY_HOME_HANDOFF:?}"
: "${SUBSTRATE_E2_TURN_ACTUAL_BINARY:?}"
: "${SUBSTRATE_E2_TURN_WORLD_ENTRY:?}"
: "${SUBSTRATE_LANDLOCK_HELPER_SRC:?}"
export SUBSTRATE_INNER_CMD=': ordinary issued argv is executed directly'
export SUBSTRATE_WORLD_ENTRY_BINARY="${SUBSTRATE_LANDLOCK_HELPER_SRC}"
exec "${SUBSTRATE_E2_TURN_WORLD_ENTRY}" "__substrate_ordinary_world_home_exec" "$@"
"##;

#[cfg(test)]
#[path = "member_runtime_ordinary_fixture_tests.rs"]
mod ordinary_fixture_tests;
#[cfg(test)]
use ordinary_fixture_tests::CheckedOrdinaryNativeFixtureV1;

#[derive(Clone)]
pub(crate) struct MemberRuntimeManager {
    active_members: Arc<RwLock<ActiveMemberRegistry>>,
    active_turns_by_span_id: Arc<RwLock<HashMap<String, Arc<ActiveSubmittedTurn>>>>,
    runtime_replay: RuntimeReplayRegistry,
    member_turn_join: Option<MemberTurnJoinRegistry>,
    privileged_child_exclusion: Arc<E3PrivilegedChildExclusionV1>,
    #[cfg(test)]
    ordinary_native_fixture: Option<Arc<CheckedOrdinaryNativeFixtureV1>>,
}

pub(crate) struct MemberRuntimeLaunchAdmissionV1 {
    pub(crate) dispatch: MemberDispatchRequestV1,
    pub(crate) acceptance_context: Option<transport_api_types::WorldWorkAcceptanceContextV1>,
    pub(crate) policy_snapshot: transport_api_types::PolicySnapshotV3,
    pub(crate) ordinary_effect: Option<Arc<crate::service::OrdinaryWorldEffectContextV1>>,
}

#[derive(Default)]
struct ActiveMemberRegistry {
    ordinary_pending_participants: BTreeMap<String, Arc<PendingOrdinaryBootstrapV1>>,
    by_participant_id: HashMap<String, Arc<ActiveMemberRuntime>>,
    by_retained_key: HashMap<RetainedMemberKey, RetainedMemberSlot>,
    #[cfg(target_os = "linux")]
    ordinary_retirements: BTreeMap<(String, String, String), CachedOrdinaryRetirementV1>,
    #[cfg(target_os = "linux")]
    ordinary_initial_retirements:
        BTreeMap<(String, String, String, String), CachedOrdinaryInitialRetirementV1>,
}

#[cfg(target_os = "linux")]
#[derive(Clone)]
struct CachedOrdinaryInitialRetirementV1 {
    reader: Arc<substrate_shell::OrdinaryInitialBootstrapCloseoutReaderV1>,
    report: transport_api_types::OrdinaryInitialBootstrapRetirementV1,
}

#[cfg(target_os = "linux")]
struct PreparedOrdinaryInitialRetirementV1 {
    frames: Vec<ExecuteStreamFrame>,
    receipt: CachedOrdinaryInitialRetirementV1,
}

#[cfg(target_os = "linux")]
#[derive(Clone)]
struct CachedOrdinaryRetirementV1 {
    reader: Arc<substrate_shell::OrdinaryRetainedStopControlReaderV1>,
    acknowledgment: transport_api_types::OrdinaryRetainedWorkerStopAcknowledgmentV1,
}

struct ActiveMemberRuntime {
    agent_id: String,
    participant_id: String,
    orchestration_session_id: String,
    orchestrator_participant_id: String,
    parent_participant_id: Option<String>,
    resumed_from_participant_id: Option<String>,
    backend_id: String,
    backend_kind: MemberRuntimeBackendKindV1,
    binary_path: std::path::PathBuf,
    launcher_dir: std::path::PathBuf,
    workspace_dir: std::path::PathBuf,
    process_working_dir: std::path::PathBuf,
    env: BTreeMap<String, String>,
    ordinary_effect: Option<Arc<crate::service::OrdinaryWorldEffectContextV1>>,
    ordinary_native_home: Mutex<Option<Arc<PreparedOrdinaryNativeHomeV1>>>,
    ordinary_native_history: Option<Arc<OrdinaryNativeWorkerHistoryV1>>,
    binding: SharedWorldBindingSnapshot,
    protocol: serde_json::Value,
    bootstrap: Mutex<Option<ActiveBootstrapRuntime>>,
    active_turn_span_id: Mutex<Option<String>>,
    uaa_session_id: Mutex<Option<String>>,
    e2_launch_activation: Option<E2MemberLaunchActivationCarrierV1>,
    authenticated_worker_cap_identity: Mutex<Option<RetainedTurnWorkerCapIdentity>>,
    #[allow(dead_code)]
    privileged_child_exclusion_lease: Arc<HeldNonE3PrivilegedChildLeaseV1>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct RetainedTurnWorkerCapIdentity {
    commitment_ref: transport_api_types::DispatchPolicyCommitmentRefCarrierV1,
    created_revision: u64,
    application_revision: u64,
}

struct ActiveBootstrapRuntime {
    span_id: String,
    terminal: Option<NativeTerminalObservation>,
    cancel: Option<AgentWrapperCancelHandle>,
    last_signal: Option<String>,
    initial_control: Option<Arc<OrdinaryInitialBootstrapCancellationV1>>,
}

/// One initial launch's existing SDK cancellation handle. A pending request is
/// only latched; it is never a native terminal or logical-retirement ACK.
#[derive(Default)]
struct OrdinaryInitialBootstrapCancellationV1 {
    state: Mutex<InitialBootstrapCancellationStateV1>,
}
#[derive(Default)]
struct InitialBootstrapCancellationStateV1 {
    signal: Option<String>,
    native: Option<(AgentWrapperCancelHandle, NativeTerminalObservation)>,
}
impl OrdinaryInitialBootstrapCancellationV1 {
    fn request_initial_control(&self, signal: &str) -> Result<bool> {
        anyhow::ensure!(
            matches!(signal, "INT" | "TERM"),
            "invalid initial control signal"
        );
        let mut state = self
            .state
            .lock()
            .map_err(|_| anyhow!("initial control latch poisoned"))?;
        state.signal = Some(signal.into());
        if let Some((cancel, terminal)) = &state.native {
            terminal.request_route_disposal();
            cancel.cancel();
            Ok(true)
        } else {
            Ok(false)
        }
    }
    fn bind_native_control(
        &self,
        cancel: AgentWrapperCancelHandle,
        terminal: NativeTerminalObservation,
    ) -> Result<()> {
        let mut state = self
            .state
            .lock()
            .map_err(|_| anyhow!("initial control latch poisoned"))?;
        anyhow::ensure!(
            state.native.is_none(),
            "initial native control already installed"
        );
        if state.signal.is_some() {
            terminal.request_route_disposal();
            cancel.cancel();
        }
        state.native = Some((cancel, terminal));
        Ok(())
    }
    fn initial_signal(&self) -> Result<Option<String>> {
        Ok(self
            .state
            .lock()
            .map_err(|_| anyhow!("initial control latch poisoned"))?
            .signal
            .clone())
    }
    fn initial_control_requested(&self) -> Result<bool> {
        Ok(self.initial_signal()?.is_some())
    }
}

struct PendingOrdinaryBootstrapV1 {
    effect: Arc<crate::service::OrdinaryWorldEffectContextV1>,
    span_id: String,
    control: Arc<OrdinaryInitialBootstrapCancellationV1>,
}

struct ActiveSubmittedTurn {
    terminal: NativeTerminalObservation,
    ordinary_effect: Option<Arc<crate::service::OrdinaryWorldEffectContextV1>>,
    ordinary_native_home: Option<Arc<PreparedOrdinaryNativeHomeV1>>,
    participant_id: String,
    cancel: AgentWrapperCancelHandle,
    last_signal: Mutex<Option<String>>,
}

struct OrdinaryMemberLaunchReservationV1 {
    registry: Arc<RwLock<ActiveMemberRegistry>>,
    participant_id: String,
    pending: Arc<PendingOrdinaryBootstrapV1>,
}

impl Drop for OrdinaryMemberLaunchReservationV1 {
    fn drop(&mut self) {
        if let Ok(mut registry) = self.registry.write() {
            if registry
                .ordinary_pending_participants
                .get(&self.participant_id)
                .is_some_and(|actual| Arc::ptr_eq(actual, &self.pending))
            {
                registry
                    .ordinary_pending_participants
                    .remove(&self.participant_id);
            }
        }
    }
}

/// Physical history is retained by the original admitted worker, independently
/// from any one native process or gateway. These identity fields grant no permit.
struct OrdinaryNativeWorkerHistoryV1 {
    path: std::path::PathBuf,
    parent_path: std::path::PathBuf,
    parent: fs::File,
    directory: fs::File,
    start: transport_api_types::OrdinaryStartSelectionCarrierV1,
    runtime: transport_api_types::OrdinaryGatewayRuntimeIdentityV1,
    selection_digest: String,
    config: Arc<Mutex<Option<(String, String)>>>,
    in_use: Arc<OrdinaryNativeHistoryStateV1>,
    // An explicit child identity over the SAME held native store. Retaining
    // the genuine source owner/ID is essential; no copied home reconstructs it.
    fork_source: Option<Arc<PinnedOrdinaryForkSourceV1>>,
    handoff_terminal: Arc<OrdinaryHomeHandoffTerminalV1>,
}

enum OrdinaryNativeHistoryRequestV1 {
    InitialStart,
    Resume(Arc<OrdinaryNativeWorkerHistoryV1>),
    Fork(Arc<PinnedOrdinaryForkSourceV1>),
}

struct PinnedOrdinaryForkSourceV1 {
    active: Arc<ActiveMemberRuntime>,
    history: Arc<OrdinaryNativeWorkerHistoryV1>,
    session_id: String,
    effect: Arc<crate::service::OrdinaryWorldEffectContextV1>,
    ownership: OrdinaryForkSourceOwnershipV1,
}

impl PinnedOrdinaryForkSourceV1 {
    fn revalidate(&self) -> Result<()> {
        self.history.in_use.require_usable()?;
        if self.ownership.is_transferred()? {
            // Genuine native thread_fork identity has been surfaced by this
            // child's owned UAA control. Source Stop cannot revoke child data.
            // This is retained data custody only. Each new child effect separately
            // authenticates its own current turn, never this retired bootstrap.
            return self.history.revalidate();
        }
        self.ownership.require_initial_source(&self.active)?;
        let original = self
            .active
            .ordinary_effect
            .as_ref()
            .ok_or_else(|| anyhow!("ordinary Fork source has no original admission"))?;
        let activation = self.effect.validate_initial_fork_source(original)?;
        let cap = self
            .active
            .authenticated_worker_cap_identity
            .lock()
            .map_err(|_| anyhow!("ordinary Fork source cap lock poisoned"))?;
        if self.active.e2_launch_activation.as_ref() != Some(&activation)
            || cap.as_ref()
                != Some(&RetainedTurnWorkerCapIdentity {
                    commitment_ref: activation.immutable_worker_cap_ref,
                    created_revision: activation.immutable_worker_cap_created_revision,
                    application_revision: activation.immutable_worker_cap_application_revision,
                })
            || self.active.uaa_session_id().as_deref() != Some(self.session_id.as_str())
        {
            anyhow::bail!("ordinary Fork held source cap/session changed");
        }
        self.history.require_original_worker(
            original.ordinary_start_selection(),
            &original.authenticated_runtime(),
            original.requested_selection(),
        )
    }
}

/// Registry/lifetime join only. This never authenticates a cap or grants a
/// native effect. Transfer is called only from this child's owned UAA facet.
struct OrdinaryForkSourceOwnershipV1 {
    registry: std::sync::Weak<RwLock<ActiveMemberRegistry>>,
    child_session: Mutex<Option<String>>,
}
impl OrdinaryForkSourceOwnershipV1 {
    fn is_transferred(&self) -> Result<bool> {
        Ok(self
            .child_session
            .lock()
            .map_err(|_| anyhow!("ordinary Fork ownership lock poisoned"))?
            .is_some())
    }

    fn require_initial_source(&self, source: &Arc<ActiveMemberRuntime>) -> Result<()> {
        if self.is_transferred()? {
            anyhow::bail!("consumed ordinary Fork source cannot issue another initial launch");
        }
        let registry = self
            .registry
            .upgrade()
            .ok_or_else(|| anyhow!("ordinary Fork source registry retired"))?;
        let registry = registry
            .read()
            .map_err(|_| anyhow!("ordinary Fork source registry lock poisoned"))?;
        if !registry
            .by_participant_id
            .get(&source.participant_id)
            .is_some_and(|current| Arc::ptr_eq(current, source))
            || source.has_active_bootstrap()
            || source
                .active_turn_span_id
                .lock()
                .map_err(|_| anyhow!("ordinary Fork source turn lock poisoned"))?
                .is_some()
        {
            anyhow::bail!("ordinary Fork source is no longer the exact retained idle owner");
        }
        Ok(())
    }

    fn observe_child_session(&self, source: &str, child: &str) -> Result<()> {
        if child.trim().is_empty() || child == source {
            anyhow::bail!("ordinary Fork did not surface a distinct child identity");
        }
        let mut observed = self
            .child_session
            .lock()
            .map_err(|_| anyhow!("ordinary Fork ownership lock poisoned"))?;
        if observed.as_ref().is_some_and(|previous| previous != child) {
            anyhow::bail!("ordinary Fork changed its surfaced child identity");
        }
        *observed = Some(child.to_owned());
        Ok(())
    }
}

/// Every physical-store writer shares this state. An unconfirmed wait/stdio
/// outcome permanently closes reuse before custody can be dropped; a later
/// lease Drop cannot turn indeterminate ownership into permission.
#[derive(Default)]
struct OrdinaryNativeHistoryStateV1(std::sync::atomic::AtomicU8);
impl OrdinaryNativeHistoryStateV1 {
    fn require_usable(&self) -> Result<()> {
        if self.0.load(std::sync::atomic::Ordering::Acquire) == 2 {
            anyhow::bail!("ordinary native history has unconfirmed native custody");
        }
        Ok(())
    }

    fn poison(&self) {
        self.0.store(2, std::sync::atomic::Ordering::Release);
    }
}
struct OrdinaryNativeHistoryUseV1 {
    state: Arc<OrdinaryNativeHistoryStateV1>,
    closed: OrdinaryNativeHistoryUseClosedV1,
}
/// Read-only release data for one exact use. It holds no store-use lease and
/// cannot establish that another worker's use is absent or that reuse is safe.
#[derive(Clone)]
struct OrdinaryNativeHistoryUseClosedV1(Arc<std::sync::atomic::AtomicBool>);
impl OrdinaryNativeHistoryUseClosedV1 {
    fn is_closed(&self) -> bool {
        self.0.load(std::sync::atomic::Ordering::Acquire)
    }
}
impl OrdinaryNativeHistoryUseV1 {
    fn acquire(in_use: Arc<OrdinaryNativeHistoryStateV1>) -> Result<Self> {
        in_use
            .0
            .compare_exchange(
                0,
                1,
                std::sync::atomic::Ordering::AcqRel,
                std::sync::atomic::Ordering::Acquire,
            )
            .map_err(|_| anyhow!("ordinary native history busy or native custody unconfirmed"))?;
        Ok(Self {
            state: in_use,
            closed: OrdinaryNativeHistoryUseClosedV1(Arc::new(std::sync::atomic::AtomicBool::new(
                false,
            ))),
        })
    }

    fn closed_observation(&self) -> OrdinaryNativeHistoryUseClosedV1 {
        self.closed.clone()
    }
}
impl Drop for OrdinaryNativeHistoryUseV1 {
    fn drop(&mut self) {
        let _ = self.state.0.compare_exchange(
            1,
            0,
            std::sync::atomic::Ordering::AcqRel,
            std::sync::atomic::Ordering::Acquire,
        );
        self.closed
            .0
            .store(true, std::sync::atomic::Ordering::Release);
    }
}

impl OrdinaryNativeWorkerHistoryV1 {
    fn fork_child(source: Arc<PinnedOrdinaryForkSourceV1>) -> Result<Self> {
        source.ownership.require_initial_source(&source.active)?;
        source.revalidate()?;
        let original = &source.history;
        let child = Self {
            path: original.path.clone(),
            parent_path: original.parent_path.clone(),
            parent: original.parent.try_clone()?,
            directory: original.directory.try_clone()?,
            start: source.effect.ordinary_start_selection().clone(),
            runtime: source.effect.authenticated_runtime(),
            selection_digest: source.effect.requested_selection().selection_digest.clone(),
            config: Arc::clone(&original.config),
            in_use: Arc::clone(&original.in_use),
            handoff_terminal: Arc::clone(&original.handoff_terminal),
            fork_source: Some(source),
        };
        child.revalidate()?;
        Ok(child)
    }

    fn create(
        launcher_dir: &Path,
        start: &transport_api_types::OrdinaryStartSelectionCarrierV1,
        runtime: &transport_api_types::OrdinaryGatewayRuntimeIdentityV1,
        selected: &transport_api_types::ResolvedAgentSelectionV1,
    ) -> Result<Self> {
        use std::os::fd::{AsRawFd, FromRawFd};
        start.validate().map_err(anyhow::Error::msg)?;
        selected.validate().map_err(anyhow::Error::msg)?;
        runtime.validate().map_err(anyhow::Error::msg)?;
        if start.requested_selection_sha256 != selected.selection_digest
            || runtime.backend_id() != selected.backend_id
        {
            anyhow::bail!("ordinary history identity data conflicts with original selection");
        }
        if !matches!(
            runtime,
            transport_api_types::OrdinaryGatewayRuntimeIdentityV1::WorldWorker { .. }
        ) {
            anyhow::bail!("ordinary worker history requires World-worker identity");
        }
        let parent = fs::OpenOptions::new()
            .read(true)
            .custom_flags(libc::O_NOFOLLOW | libc::O_DIRECTORY | libc::O_CLOEXEC)
            .open(launcher_dir)?;
        let metadata = parent.metadata()?;
        if !metadata.is_dir()
            || metadata.uid() != unsafe { libc::geteuid() }
            || metadata.permissions().mode() & 0o022 != 0
        {
            anyhow::bail!("ordinary native launcher directory changed");
        }
        let name = format!("ordinary-native-{}", uuid::Uuid::now_v7());
        let child = std::ffi::CString::new(name.as_str())?;
        if unsafe { libc::mkdirat(parent.as_raw_fd(), child.as_ptr(), 0o700) } != 0 {
            return Err(std::io::Error::last_os_error()).context("create ordinary worker history");
        }
        let raw = unsafe {
            libc::openat(
                parent.as_raw_fd(),
                child.as_ptr(),
                libc::O_RDONLY | libc::O_DIRECTORY | libc::O_NOFOLLOW | libc::O_CLOEXEC,
            )
        };
        if raw < 0 {
            return Err(std::io::Error::last_os_error()).context("hold ordinary worker history");
        }
        let history = Self {
            path: launcher_dir.join(name),
            parent_path: launcher_dir.to_path_buf(),
            parent,
            directory: unsafe { fs::File::from_raw_fd(raw) },
            start: start.clone(),
            runtime: runtime.clone(),
            selection_digest: selected.selection_digest.clone(),
            config: Arc::new(Mutex::new(None)),
            in_use: Arc::new(OrdinaryNativeHistoryStateV1::default()),
            fork_source: None,
            handoff_terminal: Arc::new(OrdinaryHomeHandoffTerminalV1::default()),
        };
        history.revalidate()?;
        Ok(history)
    }

    fn require_original_worker(
        &self,
        start: &transport_api_types::OrdinaryStartSelectionCarrierV1,
        runtime: &transport_api_types::OrdinaryGatewayRuntimeIdentityV1,
        selected: &transport_api_types::ResolvedAgentSelectionV1,
    ) -> Result<()> {
        // The current run remains authenticated independently by the effect.
        // Only the run ID varies between turns of this exact original worker.
        let mut current = runtime.clone();
        match (&self.runtime, &mut current) {
            (
                transport_api_types::OrdinaryGatewayRuntimeIdentityV1::WorldWorker {
                    run_id: original,
                    ..
                },
                transport_api_types::OrdinaryGatewayRuntimeIdentityV1::WorldWorker {
                    run_id, ..
                },
            ) => {
                *run_id = original.clone();
            }
            _ => anyhow::bail!("ordinary history role differs from original World worker"),
        }
        if self.start != *start
            || self.runtime != current
            || self.selection_digest != selected.selection_digest
        {
            anyhow::bail!("ordinary native history differs from original admitted worker");
        }
        self.revalidate()
    }

    fn require_admitted_turn(
        &self,
        effect: &crate::service::OrdinaryWorldEffectContextV1,
    ) -> Result<()> {
        effect.turn_admission()?;
        effect.revalidate_for_effect()?;
        let mut current = effect.authenticated_runtime();
        match (&self.runtime, &mut current) {
            (
                transport_api_types::OrdinaryGatewayRuntimeIdentityV1::WorldWorker {
                    caller_participant_id: original,
                    ..
                },
                transport_api_types::OrdinaryGatewayRuntimeIdentityV1::WorldWorker {
                    caller_participant_id,
                    ..
                },
            ) => *caller_participant_id = original.clone(),
            _ => anyhow::bail!("ordinary turn history requires the exact World-worker role"),
        }
        // Normalize only this comparison after real current B1/E2 admission.
        // The original history and actual current effect identities stay intact.
        self.require_original_worker(
            effect.ordinary_start_selection(),
            &current,
            effect.requested_selection(),
        )?;
        effect.revalidate_for_effect()
    }

    fn revalidate(&self) -> Result<()> {
        self.in_use.require_usable()?;
        if let Some(source) = &self.fork_source {
            source.history.revalidate()?;
        }
        let parent = fs::symlink_metadata(&self.parent_path)?;
        let held_parent = self.parent.metadata()?;
        let actual = fs::symlink_metadata(&self.path)?;
        let held = self.directory.metadata()?;
        if !parent.is_dir()
            || !actual.is_dir()
            || (parent.dev(), parent.ino()) != (held_parent.dev(), held_parent.ino())
            || (actual.dev(), actual.ino()) != (held.dev(), held.ino())
            || parent.uid() != unsafe { libc::geteuid() }
            || actual.uid() != unsafe { libc::geteuid() }
            || parent.permissions().mode() & 0o022 != 0
            || actual.permissions().mode() & 0o777 != 0o700
        {
            anyhow::bail!("ordinary native history physical binding changed");
        }
        let config = self
            .config
            .lock()
            .map_err(|_| anyhow!("ordinary history config lock poisoned"))?;
        if let Some((name, digest)) = config.as_ref() {
            self.validate_config(name, digest)?;
        }
        Ok(())
    }

    fn validate_config(&self, name: &str, digest: &str) -> Result<()> {
        use std::{
            io::Read,
            os::fd::{AsRawFd, FromRawFd},
        };
        let name = std::ffi::CString::new(name)?;
        let raw = unsafe {
            libc::openat(
                self.directory.as_raw_fd(),
                name.as_ptr(),
                libc::O_RDONLY | libc::O_NOFOLLOW | libc::O_CLOEXEC,
            )
        };
        if raw < 0 {
            return Err(std::io::Error::last_os_error()).context("reopen ordinary native config");
        }
        let file = unsafe { fs::File::from_raw_fd(raw) };
        let meta = file.metadata()?;
        if !meta.is_file()
            || meta.uid() != unsafe { libc::geteuid() }
            || meta.permissions().mode() & 0o777 != 0o600
            || meta.len() > 65536
        {
            anyhow::bail!("ordinary native config shape changed");
        }
        let mut bytes = Vec::new();
        file.take(65537).read_to_end(&mut bytes)?;
        if format!("{:x}", Sha256::digest(&bytes)) != digest {
            anyhow::bail!("ordinary native config differs from pinned rendering");
        }
        Ok(())
    }

    fn write_config(&self, name: &str, content: &str) -> Result<String> {
        use std::os::fd::{AsRawFd, FromRawFd};
        self.revalidate()?;
        let mut config = self
            .config
            .lock()
            .map_err(|_| anyhow!("ordinary history config lock poisoned"))?;
        if config
            .as_ref()
            .is_some_and(|(previous, _)| previous != name)
        {
            anyhow::bail!("ordinary native history changed harness configuration");
        }
        let temporary =
            std::ffi::CString::new(format!(".ordinary-config-{}", uuid::Uuid::now_v7()))?;
        let name_c = std::ffi::CString::new(name)?;
        let raw = unsafe {
            libc::openat(
                self.directory.as_raw_fd(),
                temporary.as_ptr(),
                libc::O_WRONLY | libc::O_CREAT | libc::O_EXCL | libc::O_NOFOLLOW | libc::O_CLOEXEC,
                0o600,
            )
        };
        if raw < 0 {
            return Err(std::io::Error::last_os_error()).context("create ordinary native config");
        }
        let mut file = unsafe { fs::File::from_raw_fd(raw) };
        file.write_all(content.as_bytes())?;
        file.sync_all()?;
        if unsafe {
            libc::renameat(
                self.directory.as_raw_fd(),
                temporary.as_ptr(),
                self.directory.as_raw_fd(),
                name_c.as_ptr(),
            )
        } != 0
        {
            return Err(std::io::Error::last_os_error()).context("replace ordinary native config");
        }
        self.directory.sync_all()?;
        let digest = format!("{:x}", Sha256::digest(content.as_bytes()));
        *config = Some((name.into(), digest.clone()));
        drop(config);
        self.revalidate()?;
        Ok(digest)
    }

    fn descriptor_path(&self) -> Result<std::path::PathBuf> {
        use std::os::fd::AsRawFd;
        self.revalidate()?;
        let path = std::path::PathBuf::from(format!(
            "/proc/{}/fd/{}",
            std::process::id(),
            self.directory.as_raw_fd()
        ));
        let observed = fs::metadata(&path)?;
        let held = self.directory.metadata()?;
        if !observed.is_dir() || (observed.dev(), observed.ino()) != (held.dev(), held.ino()) {
            anyhow::bail!("ordinary native descriptor path is unavailable");
        }
        Ok(path)
    }
}

/// Each process owns fresh effect/gateway custody, while history survives turns.
struct PreparedOrdinaryNativeHomeV1 {
    _history_use: OrdinaryNativeHistoryUseV1,
    history: Arc<OrdinaryNativeWorkerHistoryV1>,
    config_sha256: String,
    gateway: Arc<crate::gateway_runtime::OrdinaryGatewayRuntimeLeaseV1>,
    effect: Arc<crate::service::OrdinaryWorldEffectContextV1>,
    #[cfg(test)]
    ordinary_native_fixture: Option<Arc<CheckedOrdinaryNativeFixtureV1>>,
}

impl PreparedOrdinaryNativeHomeV1 {
    // The checked fixture adds an eighth argument only in test builds.
    #[cfg_attr(test, allow(clippy::too_many_arguments))]
    fn create(
        history: Arc<OrdinaryNativeWorkerHistoryV1>,
        history_use: OrdinaryNativeHistoryUseV1,
        workspace_dir: &Path,
        selected: &transport_api_types::ResolvedAgentSelectionV1,
        wiring: &transport_api_types::GatewayClientWiringV1,
        gateway: Arc<crate::gateway_runtime::OrdinaryGatewayRuntimeLeaseV1>,
        effect: Arc<crate::service::OrdinaryWorldEffectContextV1>,
        #[cfg(test)] ordinary_native_fixture: Option<Arc<CheckedOrdinaryNativeFixtureV1>>,
    ) -> Result<Self> {
        let (name, content) = ordinary_native_config(workspace_dir, selected, wiring)?;
        let config_sha256 = history.write_config(name, &content)?;
        let home = Self {
            _history_use: history_use,
            history,
            config_sha256,
            gateway,
            effect,
            #[cfg(test)]
            ordinary_native_fixture,
        };
        home.revalidate()?;
        Ok(home)
    }

    fn revalidate(&self) -> Result<()> {
        if let Some(source) = &self.history.fork_source {
            source.revalidate()?;
        }
        #[cfg(test)]
        if let Some(fixture) = self.ordinary_native_fixture.as_ref() {
            fixture.revalidate_for_effect(&self.effect)?;
        }
        self.history.revalidate()?;
        let config = self
            .history
            .config
            .lock()
            .map_err(|_| anyhow!("ordinary history config lock poisoned"))?;
        if config
            .as_ref()
            .is_none_or(|(_, digest)| *digest != self.config_sha256)
        {
            anyhow::bail!("ordinary process config no longer matches its owned rendering");
        }
        Ok(())
    }
}

impl Drop for PreparedOrdinaryNativeHomeV1 {
    fn drop(&mut self) {
        // Permission revocation never prevents dropping owned route custody.
        // History remains retained data and is never deleted by process cleanup.
        self.effect
            .native_open
            .store(false, std::sync::atomic::Ordering::Release);
    }
}

fn ordinary_native_config(
    workspace_dir: &Path,
    selected: &transport_api_types::ResolvedAgentSelectionV1,
    wiring: &transport_api_types::GatewayClientWiringV1,
) -> Result<(&'static str, String)> {
    selected.validate().map_err(anyhow::Error::msg)?;
    let openai = ordinary_native_managed_origin(&wiring.openai_base_url)?;
    ordinary_native_managed_origin(&wiring.anthropic_base_url)?;
    let (name, content) = match selected.selection.harness {
        transport_api_types::AgentHarnessFamilyV1::Codex => {
            let quote = |value: &str| toml::Value::String(value.to_string()).to_string();
            ("config.toml", format!(
            "model = {model}\nmodel_provider = \"substrate-managed-gateway\"\ncheck_for_update_on_startup = false\ncli_auth_credentials_store = \"ephemeral\"\n[model_providers.substrate-managed-gateway]\nname = \"Substrate managed gateway\"\nbase_url = {url}\nwire_api = \"responses\"\nrequires_openai_auth = false\nsupports_websockets = false\n[projects.{workspace}]\ntrust_level = \"untrusted\"\n",
            model = quote(&selected.routed_model_id), url = quote(&format!("{openai}/v1")),
            workspace = quote(&workspace_dir.to_string_lossy()),
        ))
        }
        transport_api_types::AgentHarnessFamilyV1::ClaudeCode => {
            ("settings.json", "{}\n".to_string())
        }
    };
    Ok((name, content))
}

fn ordinary_native_managed_origin(endpoint: &str) -> Result<String> {
    let port = endpoint
        .strip_prefix("http://127.0.0.1:")
        .and_then(|port| port.parse::<u16>().ok())
        .filter(|port| *port != 0)
        .ok_or_else(|| anyhow!("ordinary native requires canonical managed local endpoint"))?;
    let canonical = format!("http://127.0.0.1:{port}");
    if endpoint != canonical {
        anyhow::bail!("ordinary native endpoint is not a canonical local origin");
    }
    Ok(canonical)
}

fn mask_ordinary_native_inherited_env(runtime_env: &mut BTreeMap<String, String>) {
    let recognized = |key: &str| {
        [
            "OPENAI_",
            "ANTHROPIC_",
            "CODEX_",
            "CLAUDE_",
            "CHATGPT_",
            "SUBSTRATE_LLM_",
            "SUBSTRATE_INTERNAL_CODEX_",
            "SUBSTRATE_ORDINARY_HOME_",
        ]
        .iter()
        .any(|prefix| key.starts_with(prefix))
            || [
                "HOME",
                "XDG_CONFIG_HOME",
                "HTTP_PROXY",
                "HTTPS_PROXY",
                "ALL_PROXY",
                "NO_PROXY",
                "http_proxy",
                "https_proxy",
                "all_proxy",
                "no_proxy",
            ]
            .contains(&key)
    };
    let keys: BTreeSet<String> = runtime_env
        .keys()
        .cloned()
        .chain(std::env::vars_os().filter_map(|(key, _)| key.into_string().ok()))
        .filter(|key| recognized(key))
        .collect();
    for key in keys {
        runtime_env.insert(key, String::new());
    }
}

fn configure_ordinary_native_env(
    runtime_env: &mut BTreeMap<String, String>,
    selected: &transport_api_types::ResolvedAgentSelectionV1,
    native_path: &Path,
    wiring: &transport_api_types::GatewayClientWiringV1,
) -> Result<()> {
    selected.validate().map_err(anyhow::Error::msg)?;
    ordinary_native_managed_origin(&wiring.openai_base_url)?;
    ordinary_native_managed_origin(&wiring.anthropic_base_url)?;
    mask_ordinary_native_inherited_env(runtime_env);
    let native_path = native_path.display().to_string();
    runtime_env.insert("HOME".into(), native_path.clone());
    runtime_env.insert("XDG_CONFIG_HOME".into(), native_path.clone());
    runtime_env.insert("NO_PROXY".into(), "127.0.0.1,::1,localhost".into());
    match selected.selection.harness {
        transport_api_types::AgentHarnessFamilyV1::Codex => {
            runtime_env.insert("CODEX_HOME".into(), native_path);
        }
        transport_api_types::AgentHarnessFamilyV1::ClaudeCode => {
            runtime_env.insert("CLAUDE_CONFIG_DIR".into(), native_path);
            runtime_env.insert(
                "ANTHROPIC_BASE_URL".into(),
                ordinary_native_managed_origin(&wiring.anthropic_base_url)?,
            );
            runtime_env.insert(
                "ANTHROPIC_AUTH_TOKEN".into(),
                "substrate-managed-gateway".into(),
            );
            runtime_env.insert("ANTHROPIC_MODEL".into(), selected.routed_model_id.clone());
        }
    }
    Ok(())
}

async fn prepare_ordinary_native_runtime_env(
    runtime_env: &mut BTreeMap<String, String>,
    effect: &Arc<crate::service::OrdinaryWorldEffectContextV1>,
    workspace_dir: &Path,
    launcher_dir: &Path,
    request: OrdinaryNativeHistoryRequestV1,
    #[cfg(test)] ordinary_native_fixture: Option<Arc<CheckedOrdinaryNativeFixtureV1>>,
) -> Result<Arc<PreparedOrdinaryNativeHomeV1>> {
    effect.revalidate_for_effect()?;
    let history = match request {
        OrdinaryNativeHistoryRequestV1::InitialStart => {
            Arc::new(OrdinaryNativeWorkerHistoryV1::create(
                launcher_dir,
                effect.ordinary_start_selection(),
                &effect.authenticated_runtime(),
                effect.requested_selection(),
            )?)
        }
        OrdinaryNativeHistoryRequestV1::Resume(history) => {
            // Reject missing/substituted history before acquiring a gateway.
            history.require_admitted_turn(effect)?;
            history
        }
        OrdinaryNativeHistoryRequestV1::Fork(source) => {
            Arc::new(OrdinaryNativeWorkerHistoryV1::fork_child(source)?)
        }
    };
    let history_use = OrdinaryNativeHistoryUseV1::acquire(Arc::clone(&history.in_use))?;
    let (wiring, gateway) = effect.prepare_ordinary_native_gateway(None).await?;
    let selected = effect.requested_selection();
    let home = Arc::new(PreparedOrdinaryNativeHomeV1::create(
        history,
        history_use,
        workspace_dir,
        selected,
        &wiring,
        gateway,
        Arc::clone(effect),
        #[cfg(test)]
        ordinary_native_fixture,
    )?);
    configure_ordinary_native_env(
        runtime_env,
        selected,
        &home.history.descriptor_path()?,
        &wiring,
    )?;
    runtime_env.remove(crate::internal_exec::ordinary_home_handoff::HANDOFF_ENV);
    effect.revalidate_for_effect()?;
    home.revalidate()?;
    Ok(home)
}

/// Bounded to one latest run. The independent supervisor writes only after the
/// actual listener JoinHandle resolves, including when its startup awaiter dies.
#[derive(Default)]
struct OrdinaryHomeHandoffTerminalV1 {
    latest: Mutex<Option<(String, Option<&'static str>)>>,
    changed: tokio::sync::Notify,
}

struct OrdinaryHomeHandoffOwnerV1 {
    shutdown: tokio::sync::oneshot::Sender<()>,
    completed: tokio::sync::oneshot::Receiver<&'static str>,
}

#[cfg(test)]
fn observe_ordinary_home_stage_for_test(stage: &str, details: serde_json::Value) {
    let Some(root) = std::env::var_os("SUBSTRATE_TEST_ORDINARY_EVIDENCE_ROOT") else {
        return;
    };
    if matches!(
        stage,
        "listener_scheduled"
            | "listener_first_poll"
            | "peer_accepted"
            | "before_current_effect_revalidation"
            | "after_current_effect_revalidation"
            | "after_history_revalidation"
            | "reply_written"
    ) {
        tracing::debug!(target: "substrate::ordinary_home_io", phase = stage,
            peer_pid = details.get("peer_pid").and_then(serde_json::Value::as_i64),
            "ordinary home timing");
    }
    // Only the explicitly invoked synthetic component fixture enables this.
    // Never record the locator/nonce, request environment, or credential inputs.
    let record = json!({"component_only":true,"stage":stage,"details":details});
    let path = Path::new(&root).join("ordinary-home-stages.jsonl");
    let observed = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .and_then(|mut file| writeln!(file, "{record}"));
    if let Err(error) = observed {
        eprintln!("could not retain ordinary component stage: {error}");
    }
}

fn supervise_ordinary_home_handoff(
    listener_task: tokio::task::JoinHandle<Result<bool>>,
    stop_tx: tokio::sync::oneshot::Sender<()>,
    terminal: Arc<OrdinaryHomeHandoffTerminalV1>,
    run_id: String,
) -> OrdinaryHomeHandoffOwnerV1 {
    let (shutdown, shutdown_rx) = tokio::sync::oneshot::channel();
    let (mut completed_tx, completed) = tokio::sync::oneshot::channel();
    // This owner is independent of run_control's awaiter and owns the actual
    // JoinHandle. Dropping that awaiter closes completed, requests stop, then
    // still joins the entire accepted callback/listener before recording terminal.
    tokio::spawn(async move {
        let mut listener_task = listener_task;
        let joined = tokio::select! {
            result = &mut listener_task => result,
            _ = shutdown_rx => { let _ = stop_tx.send(()); listener_task.await },
            _ = completed_tx.closed() => { let _ = stop_tx.send(()); listener_task.await },
        };
        #[cfg(test)]
        observe_ordinary_home_stage_for_test(
            "listener_join",
            match &joined {
                Ok(Ok(used)) => json!({"used":used}),
                Ok(Err(error)) => json!({"listener_error":format!("{error:#}")}),
                Err(error) => json!({"join_error":error.to_string()}),
            },
        );
        let result = match joined {
            Ok(Ok(true)) => "joined_used",
            Ok(Ok(false)) => "joined_stopped",
            Ok(Err(_)) => "joined_refused",
            Err(_) => "joined_failed",
        };
        if let Ok(mut latest) = terminal.latest.lock() {
            *latest = Some((run_id, Some(result)));
        }
        terminal.changed.notify_waiters();
        let _ = completed_tx.send(result);
    });
    OrdinaryHomeHandoffOwnerV1 {
        shutdown,
        completed,
    }
}

fn prepare_ordinary_start_home_launcher(
    actual_binary: &Path,
    launcher_dir: &Path,
    snapshot: &transport_api_types::PolicySnapshotV3,
    placement: &LinuxWorldPlacementContext,
    process_cwd: &Path,
    runtime_env: &mut BTreeMap<String, String>,
) -> Result<PreparedSubmittedTurnPolicyLauncher> {
    let baseline = runtime_env
        .iter()
        .map(|(k, v)| (k.clone(), v.clone()))
        .collect();
    let mut current = crate::service::build_member_runtime_turn_env(
        baseline,
        snapshot,
        &placement.working_dir,
        process_cwd,
    )?;
    current.insert(
        SUBSTRATE_E2_TURN_ACTUAL_BINARY_ENV.into(),
        actual_binary.display().to_string(),
    );
    current.insert(
        SUBSTRATE_E2_TURN_WORLD_ENTRY_ENV.into(),
        launcher_dir.join("world-entry.sh").display().to_string(),
    );
    let launcher = write_submitted_turn_policy_launcher(launcher_dir, true)?;
    *runtime_env = current.into_iter().collect();
    Ok(launcher)
}

/// Exact pinned app-server argv, callable only after genuine Fork admission.
fn classify_ordinary_fork_codex_args(args: &[String]) -> Result<Option<usize>> {
    let borrowed: Vec<_> = args.iter().map(String::as_str).collect();
    for (index, probe) in [
        vec!["--version"],
        vec!["features", "list", "--json"],
        vec!["features", "list"],
        vec!["--help"],
    ]
    .iter()
    .enumerate()
    {
        if &borrowed == probe {
            return Ok(Some(index));
        }
    }
    if borrowed != ["app-server"] {
        anyhow::bail!("ordinary Fork argv differs from exact pinned app-server invocation");
    }
    Ok(None)
}

/// Closed pinned-Codex shapes only. Some(index) denotes one of four probes;
/// None denotes the single actual admitted Start/Resume invocation.
fn classify_ordinary_codex_args(
    args: &[String],
    resume: Option<&str>,
    sdk_temp: &Path,
    selected_model: &str,
    selected_workspace: &Path,
) -> Result<Option<usize>> {
    let borrowed: Vec<_> = args.iter().map(String::as_str).collect();
    for (index, probe) in [
        vec!["--version"],
        vec!["features", "list", "--json"],
        vec!["features", "list"],
        vec!["--help"],
    ]
    .iter()
    .enumerate()
    {
        if &borrowed == probe {
            return Ok(Some(index));
        }
    }
    // Complete pinned UAA0.3.7/Codex0.3.7 argv, not a contains/ends_with parser.
    // Start reads stdin without a '-' argv; Resume appends resume,id,'-'.
    let output = args
        .get(11)
        .context("ordinary SDK output argument missing")?;
    let mut expected = vec![
        "--dangerously-bypass-approvals-and-sandbox".to_owned(),
        "exec".into(),
        "--color".into(),
        "never".into(),
        "--skip-git-repo-check".into(),
        "--json".into(),
        "--model".into(),
        selected_model.into(),
        "--add-dir".into(),
        selected_workspace
            .to_str()
            .context("ordinary selected workspace is not UTF-8")?
            .into(),
        "--output-last-message".into(),
        output.clone(),
    ];
    if let Some(id) = resume {
        expected.extend(["resume".into(), id.into(), "-".into()]);
    }
    if args != expected {
        anyhow::bail!(
            "ordinary native argv conflicts with exact selected model/workspace/session/options"
        );
    }
    let output = Path::new(output);
    let name = output
        .file_name()
        .and_then(|v| v.to_str())
        .context("ordinary SDK output filename")?;
    let suffix = format!("_{}.txt", std::process::id());
    let timestamp = name
        .strip_prefix("codex_last_message_")
        .and_then(|v| v.strip_suffix(&suffix));
    if output.parent() != Some(sdk_temp)
        || timestamp.is_none_or(|v| v.is_empty() || !v.bytes().all(|b| b.is_ascii_digit()))
        || fs::symlink_metadata(output).is_ok()
    {
        anyhow::bail!("ordinary SDK output is not its exact new parent-generated scratch file");
    }
    Ok(None)
}

/// The current ordinary Claude adapter emits one closed Start/explicit Resume
/// shape. Selection and prompt come from the independently admitted request;
/// no ambient session, native Fork, extra config, or generic fallback is admitted.
fn classify_ordinary_claude_args(
    args: &[String],
    resume: Option<&str>,
    selected_model: &str,
    selected_prompt: &str,
) -> Result<()> {
    let mut expected: Vec<String> = [
        "--print",
        "--output-format",
        "stream-json",
        "--model",
        selected_model,
        "--permission-mode",
        "bypassPermissions",
        "--include-partial-messages",
    ]
    .into_iter()
    .map(str::to_owned)
    .collect();
    if let Some(id) = resume {
        expected.extend(["--resume".into(), id.into()]);
    }
    expected.extend(["--verbose".into(), selected_prompt.into()]);
    if args != expected {
        anyhow::bail!(
            "ordinary Claude argv conflicts with exact selected model/prompt/session/options"
        );
    }
    Ok(())
}

fn observe_ordinary_home_io_failure(
    operation: &'static str,
    peer_pid: libc::pid_t,
    started: std::time::Instant,
    error: &std::io::Error,
) {
    tracing::debug!(target: "substrate::ordinary_home_io", operation, peer_pid,
        elapsed_us = started.elapsed().as_micros() as u64,
        error_kind = ?error.kind(), errno = error.raw_os_error(),
        "ordinary child handoff I/O failed");
}

async fn issue_ordinary_home_handoff(
    home: &Arc<PreparedOrdinaryNativeHomeV1>,
    request: &mut AgentWrapperRunRequest,
) -> Result<OrdinaryHomeHandoffOwnerV1> {
    let runtime_env = &mut request.env;
    use crate::internal_exec::ordinary_home_handoff::{
        HomeReply, Locator, HANDOFF_ENV, HOME_REPLY_TIMEOUT,
    };
    use std::io::Read as _;
    use std::os::fd::AsRawFd;
    use std::os::linux::net::SocketAddrExt;
    use std::os::unix::net::{SocketAddr, UnixListener};
    use tokio::io::{AsyncReadExt, AsyncWriteExt};

    if runtime_env
        .get("SUBSTRATE_WORLD_FS_ISOLATION")
        .is_some_and(|v| v.eq_ignore_ascii_case("full"))
        || runtime_env
            .get("SUBSTRATE_MOUNT_FS_MODE")
            .is_some_and(|v| v.eq_ignore_ascii_case("read_only"))
    {
        anyhow::bail!("ordinary child-home custody unsupported for this filesystem posture");
    }
    // A dropped prior startup awaiter does not let a later turn resurrect its IPC.
    loop {
        let notified = home.history.handoff_terminal.changed.notified();
        tokio::pin!(notified);
        notified.as_mut().enable();
        let pending = home
            .history
            .handoff_terminal
            .latest
            .lock()
            .map_err(|_| anyhow!("ordinary handoff terminal state poisoned"))?
            .as_ref()
            .is_some_and(|(_, result)| result.is_none());
        if !pending {
            break;
        }
        tokio::time::timeout(std::time::Duration::from_secs(5), notified)
            .await
            .context("previous ordinary home handoff did not join")?;
    }
    if runtime_env.contains_key(crate::enforcement_plan::WORLD_FS_ENFORCEMENT_PLAN_B64_ENV) {
        anyhow::bail!("ordinary child-home custody does not admit deny-plan posture");
    }
    let sdk_temp = std::env::temp_dir();
    if !home.history.path.starts_with("/tmp")
        || fs::canonicalize(&home.history.path)? != home.history.path
        || !sdk_temp.starts_with("/tmp")
        || fs::canonicalize(&sdk_temp)? != sdk_temp
    {
        anyhow::bail!(
            "ordinary managed home and SDK output require existing canonical /tmp scratch coverage"
        );
    }
    home.effect.revalidate_for_effect()?;
    home.revalidate()?;
    let mut entropy = [0u8; 32];
    fs::File::open("/dev/urandom")?.read_exact(&mut entropy)?;
    let nonce: String = entropy.iter().map(|byte| format!("{byte:02x}")).collect();
    let locator = Locator {
        socket: format!("substrate-ordinary-home-{}", &nonce[..32]),
        nonce: nonce.clone(),
    };
    let address = SocketAddr::from_abstract_name(locator.socket.as_bytes())?;
    let listener = UnixListener::bind_addr(&address)?;
    listener.set_nonblocking(true)?;
    let listener = tokio::net::UnixListener::from_std(listener)?;
    let selected_harness = home.effect.requested_selection().selection.harness;
    let selected_prompt = request.prompt.clone();
    let binary = runtime_env
        .get(SUBSTRATE_E2_TURN_ACTUAL_BINARY_ENV)
        .cloned()
        .context("ordinary actual binary missing")?;
    let helper = fs::OpenOptions::new()
        .read(true)
        .custom_flags(libc::O_NOFOLLOW | libc::O_CLOEXEC)
        .open(
            runtime_env
                .get(crate::service::LANDLOCK_HELPER_SRC_ENV)
                .context("ordinary source helper missing")?,
        )?;
    let helper_identity = helper.metadata()?;
    if !helper_identity.is_file() || helper_identity.permissions().mode() & 0o022 != 0 {
        anyhow::bail!("ordinary helper executable custody changed");
    }
    let resume = request
        .extensions
        .get(SESSION_RESUME_EXTENSION_V1)
        .map(|v| {
            if v.get("selector").and_then(|v| v.as_str()) != Some("id") {
                anyhow::bail!("ordinary Resume selector changed");
            }
            v.get("id")
                .and_then(|v| v.as_str())
                .map(str::to_owned)
                .context("ordinary Resume session missing")
        })
        .transpose()?;
    // Fork lineage remains pinned for the child's lifetime. A genuine admitted
    // turn resumes that child; only its bootstrap uses the initial Fork path.
    let retained_turn = home.effect.turn_admission().is_ok();
    if retained_turn && resume.is_none() {
        anyhow::bail!("ordinary retained turn requires Resume");
    }
    let fork = if retained_turn {
        None
    } else {
        home.history.fork_source.as_ref()
    };
    let initial_fork = fork.is_some();
    if initial_fork && resume.is_some() {
        anyhow::bail!("ordinary initial Fork cannot include Resume");
    }
    if let Some(source) = fork {
        source.revalidate()?;
        if selected_harness != transport_api_types::AgentHarnessFamilyV1::Codex
            || request.extensions.len() != 1
            || request.extensions.get(SESSION_FORK_EXTENSION_V1)
                != Some(&json!({"selector":"id","id":source.session_id}))
        {
            anyhow::bail!("ordinary Fork selector/options differ from exact held source");
        }
    } else if request.extensions.contains_key(SESSION_FORK_EXTENSION_V1) {
        anyhow::bail!("ordinary native Fork omitted authenticated source ownership");
    }
    let selected_model = home.effect.requested_selection().routed_model_id.clone();
    if fork.is_none()
        && request
            .extensions
            .get("agent_api.config.model.v1")
            .and_then(|v| v.as_str())
            != Some(selected_model.as_str())
    {
        anyhow::bail!("ordinary request model differs from held selection");
    }
    let selected_workspace = request
        .working_dir
        .clone()
        .context("ordinary request workspace missing")?;
    let declared_dirs = request
        .extensions
        .get(ADD_DIRS_EXTENSION_V1)
        .and_then(|v| v.get("dirs"))
        .and_then(|v| v.as_array());
    match selected_harness {
        transport_api_types::AgentHarnessFamilyV1::Codex => {
            if fork.is_none()
                && declared_dirs.is_none_or(|dirs| {
                    dirs.len() != 1 || dirs[0].as_str() != selected_workspace.to_str()
                })
            {
                anyhow::bail!("ordinary declared add-dir differs from held process workspace");
            }
        }
        transport_api_types::AgentHarnessFamilyV1::ClaudeCode => {
            // The current adapter uses the exact working directory and no add-dir
            // extension. Do not reinterpret an injected empty/extra extension.
            if request.extensions.contains_key(ADD_DIRS_EXTENSION_V1)
                || request.extensions.len() != 1 + usize::from(resume.is_some())
                || request.extensions.keys().any(|key| {
                    key != "agent_api.config.model.v1" && key != SESSION_RESUME_EXTENSION_V1
                })
            {
                anyhow::bail!("ordinary Claude workspace/options differ from issued request");
            }
        }
    }
    let held_workspace = fs::OpenOptions::new()
        .read(true)
        .custom_flags(libc::O_DIRECTORY | libc::O_NOFOLLOW | libc::O_CLOEXEC)
        .open(&selected_workspace)?;
    let history = Arc::clone(&home.history);
    let effect = Arc::clone(&home.effect);
    let config_sha256 = home.config_sha256.clone();
    #[cfg(test)]
    let fixture = home.ordinary_native_fixture.clone();
    let run_id = match effect.authenticated_runtime() {
        transport_api_types::OrdinaryGatewayRuntimeIdentityV1::WorldWorker { run_id, .. } => run_id,
        _ => anyhow::bail!("ordinary child-home handoff requires World-worker admission"),
    };
    *history
        .handoff_terminal
        .latest
        .lock()
        .map_err(|_| anyhow!("ordinary handoff terminal state poisoned"))? =
        Some((run_id.clone(), None));
    let (stop_tx, stop_rx) = tokio::sync::oneshot::channel();
    #[cfg(test)]
    observe_ordinary_home_stage_for_test("listener_scheduled", json!({}));
    let listener_task = tokio::spawn(async move {
        #[cfg(test)]
        observe_ordinary_home_stage_for_test("listener_first_poll", json!({}));
        let serving = async move {
            let _held_helper = helper;
            let held_workspace_identity = held_workspace.metadata()?;
            let _held_workspace = held_workspace;
            // UAA probes once, then lower Codex streaming probes once: both
            // explicitly bypass the process cache when env overrides are present.
            // Each of these four closed shapes is admitted at most twice.
            let mut probes = [0u8; 4];
            loop {
                let (mut stream, _) = listener.accept().await?;
                let callback_started = std::time::Instant::now();
                let callback = async {
                    // The SDK starts this process directly; every submitted launcher step
                    // execs in-place. Kernel peer identity is checked in both directions.
                    let peer = crate::internal_exec::ordinary_home_handoff::peer_credentials(
                        stream.as_raw_fd(),
                    )?;
                    if peer.uid != unsafe { libc::geteuid() } {
                        anyhow::bail!("ordinary home child UID differs from issuer");
                    }
                    crate::internal_exec::ordinary_home_handoff::require_direct_child(
                        peer.pid,
                        std::process::id(),
                    )?;
                    #[cfg(test)]
                    observe_ordinary_home_stage_for_test(
                        "peer_accepted",
                        json!({"peer_pid":peer.pid,"elapsed_us":callback_started.elapsed().as_micros()}),
                    );
                    let mut presented = Vec::new();
                    (&mut stream)
                        .take(65)
                        .read_to_end(&mut presented)
                        .await
                        .inspect_err(|error| {
                            observe_ordinary_home_io_failure(
                                "nonce_read",
                                peer.pid,
                                callback_started,
                                error,
                            );
                        })?;
                    if presented != nonce.as_bytes() {
                        anyhow::bail!("ordinary home nonce differs from current issued handoff");
                    }
                    let mapped =
                        fs::metadata(format!("/proc/{}/exe", peer.pid)).inspect_err(|error| {
                            observe_ordinary_home_io_failure(
                                "proc_exe_metadata",
                                peer.pid,
                                callback_started,
                                error,
                            );
                        })?;
                    if (mapped.dev(), mapped.ino())
                        != (helper_identity.dev(), helper_identity.ino())
                    {
                        anyhow::bail!("ordinary handoff peer is not the held source helper");
                    }
                    let cmdline =
                        fs::read(format!("/proc/{}/cmdline", peer.pid)).inspect_err(|error| {
                            observe_ordinary_home_io_failure(
                                "proc_cmdline_read",
                                peer.pid,
                                callback_started,
                                error,
                            );
                        })?;
                    if cmdline.len() > 4096 {
                        anyhow::bail!("ordinary helper argv exceeds handoff shape");
                    }
                    let argv: Vec<String> = cmdline
                        .split(|b| *b == 0)
                        .filter(|b| !b.is_empty())
                        .map(|b| String::from_utf8(b.to_vec()).map_err(anyhow::Error::from))
                        .collect::<Result<_>>()?;
                    if argv.get(1).map(String::as_str)
                        != Some(crate::internal_exec::ORDINARY_HOME_EXEC_ARG)
                    {
                        anyhow::bail!("ordinary child entered a different helper mode");
                    }
                    let args = argv.into_iter().skip(2).collect::<Vec<_>>();
                    let child_cwd =
                        fs::metadata(format!("/proc/{}/cwd", peer.pid)).inspect_err(|error| {
                            observe_ordinary_home_io_failure(
                                "proc_cwd_metadata",
                                peer.pid,
                                callback_started,
                                error,
                            );
                        })?;
                    if (child_cwd.dev(), child_cwd.ino())
                        != (held_workspace_identity.dev(), held_workspace_identity.ino())
                    {
                        anyhow::bail!(
                            "ordinary actual child working directory differs from held workspace"
                        );
                    }
                    let probe = match selected_harness {
                        transport_api_types::AgentHarnessFamilyV1::Codex => {
                            if initial_fork {
                                // The source ownership above is revalidated again at effect.
                                classify_ordinary_fork_codex_args(&args)?
                            } else {
                                classify_ordinary_codex_args(
                                    &args,
                                    resume.as_deref(),
                                    &sdk_temp,
                                    &selected_model,
                                    &selected_workspace,
                                )?
                            }
                        }
                        transport_api_types::AgentHarnessFamilyV1::ClaudeCode => {
                            classify_ordinary_claude_args(
                                &args,
                                resume.as_deref(),
                                &selected_model,
                                &selected_prompt,
                            )?;
                            None
                        }
                    };
                    if let Some(index) = probe {
                        probes[index] += 1;
                        if probes[index] > 2 {
                            anyhow::bail!(
                                "ordinary SDK probe repeated outside pinned two-pass budget"
                            );
                        }
                    }

                    #[cfg(test)]
                    observe_ordinary_home_stage_for_test(
                        "peer_proc_and_argv_validated",
                        json!({"peer_pid":peer.pid,"probe_index":probe,
                            "elapsed_us":callback_started.elapsed().as_micros()}),
                    );
                    // A bearer route/nonce is not a permit. The live sealed current-run
                    // owner independently revalidates source/account/policy immediately
                    // before issuing this exact still-held directory/config descriptor.
                    #[cfg(test)]
                    observe_ordinary_home_stage_for_test(
                        "before_current_effect_revalidation",
                        json!({"peer_pid":peer.pid,"elapsed_us":callback_started.elapsed().as_micros()}),
                    );
                    #[cfg(not(test))]
                    effect.revalidate_for_effect()?;
                    #[cfg(test)]
                    match fixture.as_ref() {
                        // This existing checked fixture method performs its
                        // physical+worker checks and a full current effect join.
                        Some(fixture) => fixture.revalidate_for_effect(&effect)?,
                        None => effect.revalidate_for_effect()?,
                    }
                    #[cfg(test)]
                    observe_ordinary_home_stage_for_test(
                        "after_current_effect_revalidation",
                        json!({"peer_pid":peer.pid,"elapsed_us":callback_started.elapsed().as_micros()}),
                    );
                    #[cfg(test)]
                    observe_ordinary_home_stage_for_test(
                        "after_fixture_revalidation",
                        json!({"peer_pid":peer.pid,"elapsed_us":callback_started.elapsed().as_micros()}),
                    );
                    if let Some(source) = &history.fork_source {
                        source.revalidate()?;
                    }
                    history.revalidate()?;
                    #[cfg(test)]
                    observe_ordinary_home_stage_for_test(
                        "after_history_revalidation",
                        json!({"peer_pid":peer.pid,"elapsed_us":callback_started.elapsed().as_micros()}),
                    );
                    let (config_name, pinned_digest) = history
                        .config
                        .lock()
                        .map_err(|_| anyhow!("ordinary history config lock poisoned"))?
                        .clone()
                        .context("ordinary issued config missing")?;
                    if pinned_digest != config_sha256 {
                        anyhow::bail!("ordinary issued current-run config changed");
                    }
                    let metadata = history.directory.metadata()?;
                    let reply = HomeReply {
                        nonce: nonce.clone(),
                        service_pid: std::process::id(),
                        child_pid: peer.pid as u32,
                        source_fd: history.directory.as_raw_fd(),
                        device: metadata.dev(),
                        inode: metadata.ino(),
                        uid: metadata.uid(),
                        config_name,
                        config_sha256: config_sha256.clone(),
                        sdk_temp: sdk_temp.display().to_string(),
                        binary: binary.clone(),
                        args,
                    };
                    let bytes = serde_json::to_vec(&reply)?;
                    if bytes.len() > 4096 {
                        anyhow::bail!("ordinary home reply exceeds fixed shape");
                    }
                    #[cfg(test)]
                    {
                        // Only this authenticated direct child is inspected. State is
                        // metadata, never a substitute for its SDK-owned Child.wait.
                        let peer_state = fs::read_to_string(format!("/proc/{}/stat", peer.pid))
                            .map(|stat| {
                                stat.rsplit_once(')')
                                    .and_then(|(_, tail)| tail.split_whitespace().next())
                                    .map(str::to_owned)
                            });
                        observe_ordinary_home_stage_for_test(
                            "before_reply_write",
                            json!({"peer_pid":peer.pid,"elapsed_us":callback_started.elapsed().as_micros(),
                                "peer_state":peer_state.as_ref().ok(),
                                "peer_state_error":peer_state.err().map(|error|error.to_string())}),
                        );
                    }
                    stream.write_all(&bytes).await?;
                    #[cfg(test)]
                    observe_ordinary_home_stage_for_test(
                        "reply_written",
                        json!({"peer_pid":peer.pid,"elapsed_us":callback_started.elapsed().as_micros()}),
                    );
                    stream.shutdown().await?;
                    Ok::<_, anyhow::Error>(probe.is_none())
                };
                let used = tokio::time::timeout(HOME_REPLY_TIMEOUT, callback)
                    .await
                    .context("ordinary home callback deadline")??;
                if used {
                    return Ok::<_, anyhow::Error>(true);
                }
            }
        };
        tokio::select! {
            _ = stop_rx => Ok(false),
            // Two real SDK passes of at most four closed probes, then one
            // native Start/Resume. Each callback retains the same finite limit;
            // at most nine successful replies can be issued for this one-use run.
            result = tokio::time::timeout(HOME_REPLY_TIMEOUT.saturating_mul(9), serving) => result.context("ordinary home handoff deadline")?,
        }
    });
    let owner = supervise_ordinary_home_handoff(
        listener_task,
        stop_tx,
        Arc::clone(&home.history.handoff_terminal),
        run_id,
    );
    runtime_env.insert(HANDOFF_ENV.into(), serde_json::to_string(&locator)?);
    Ok(owner)
}
async fn run_ordinary_native_control<C: Send + 'static>(
    prompt_fulfillment: Arc<PromptFulfillmentBridge>,
    mut request: AgentWrapperRunRequest,
    home: Option<Arc<PreparedOrdinaryNativeHomeV1>>,
    custody: C,
) -> std::result::Result<PromptFulfillmentRunControl, AgentWrapperError> {
    // Preserve the existing nonordinary/protected request path and its future.
    let Some(home) = home else {
        return prompt_fulfillment.run_control(request).await;
    };
    let route = OrdinaryNativeRouteDisposalV1::from_home(&home);
    let deadline = request
        .timeout
        .filter(|value| !value.is_zero())
        .map(|value| tokio::time::Instant::now() + value);
    let startup_home = Arc::clone(&home);
    let startup = async move {
        let home = Some(&startup_home);
        // Both ordinary Start and Resume now use the same admitted policy helper.
        #[cfg(test)]
        observe_ordinary_home_stage_for_test(
            "issue_begin",
            json!({"isolation":request.env.get("SUBSTRATE_WORLD_FS_ISOLATION"),
            "fs_mode":request.env.get("SUBSTRATE_MOUNT_FS_MODE"),
            "plan_present":request.env.contains_key(crate::enforcement_plan::WORLD_FS_ENFORCEMENT_PLAN_B64_ENV),
            "plan_empty":request.env.get(crate::enforcement_plan::WORLD_FS_ENFORCEMENT_PLAN_B64_ENV).map(|v|v.is_empty())}),
        );
        let handoff = match home {
            Some(home) => Some(
                issue_ordinary_home_handoff(home, &mut request)
                    .await
                    .map_err(|error| {
                        #[cfg(test)]
                        observe_ordinary_home_stage_for_test(
                            "issue_refused",
                            json!({"error":format!("{error:#}")}),
                        );
                        #[cfg(not(test))]
                        let _ = error;
                        AgentWrapperError::Backend {
                            message: "ordinary current-run home handoff refused".into(),
                        }
                    })?,
            ),
            None => None,
        };
        #[cfg(test)]
        observe_ordinary_home_stage_for_test("sdk_startup_begin", json!({}));
        let control = prompt_fulfillment.run_control(request).await;
        #[cfg(test)]
        observe_ordinary_home_stage_for_test(
            "sdk_startup_returned",
            match &control {
                Ok(_) => json!({"returned_control":true}),
                Err(error) => json!({"returned_control":false,"error":error.to_string()}),
            },
        );
        match control {
            Ok(mut control) => {
                let completion = control.handle.completion;
                #[cfg(test)]
                let observed_home = home.cloned();
                control.handle.completion = Box::pin(async move {
                    let outcome = completion.await;
                    let handoff_result = if let Some(owner) = handoff {
                        let _ = owner.shutdown.send(());
                        Some(owner.completed.await.unwrap_or("join_result_lost"))
                    } else {
                        None
                    };
                    #[cfg(test)]
                    if let (Some(home), Some(root)) = (
                        observed_home,
                        std::env::var_os("SUBSTRATE_TEST_ORDINARY_ROOT"),
                    ) {
                        use std::io::Write as _;
                        let (success, final_text) = match &outcome {
                            Ok(result) => (result.status.success(), result.final_text.clone()),
                            Err(_) => (false, None),
                        };
                        let latest = home.history.handoff_terminal.latest.lock().unwrap().clone();
                        let mut log = fs::OpenOptions::new()
                            .create(true)
                            .append(true)
                            .open(Path::new(&root).join("sdk-completions.jsonl"))
                            .unwrap();
                        writeln!(
                            log,
                            "{}",
                            json!({"success":success,"final_text":final_text,
                        "handoff":handoff_result,"latest_join":latest})
                        )
                        .unwrap();
                    }
                    if handoff_result.is_some_and(|v| v != "joined_used") {
                        return Err(AgentWrapperError::Backend {
                            message: format!(
                                "ordinary home handoff terminal join failed; backend outcome: {}",
                                outcome
                                    .as_ref()
                                    .err()
                                    .map(ToString::to_string)
                                    .unwrap_or_else(|| "backend completion received".into())
                            ),
                        });
                    }
                    outcome
                });
                Ok(control)
            }
            Err(error) => {
                if let Some(owner) = handoff {
                    let _ = owner.shutdown.send(());
                    // Actual join is awaited on returned startup failure. Dropped
                    // awaiters are independently handled by the supervisor above.
                    let joined = owner.completed.await;
                    #[cfg(test)]
                    observe_ordinary_home_stage_for_test(
                        "sdk_startup_refusal_cleanup",
                        json!({"handoff_join":joined.ok()}),
                    );
                    #[cfg(not(test))]
                    let _ = joined;
                }
                Err(error)
            }
        }
    };
    retain_ordinary_native_startup(startup, route, (home, custody), deadline).await
}

/// Disposal-only references, derived from the already held admitted home. This
/// type has no effect API and does not mint admission or acknowledge native wait.
#[derive(Clone)]
struct OrdinaryNativeRouteDisposalV1 {
    history_state: Arc<OrdinaryNativeHistoryStateV1>,
    native_open: Arc<std::sync::atomic::AtomicBool>,
    gateway: Arc<crate::gateway_runtime::OrdinaryGatewayRuntimeLeaseV1>,
    #[cfg(test)]
    runtime: transport_api_types::OrdinaryGatewayRuntimeIdentityV1,
}
impl OrdinaryNativeRouteDisposalV1 {
    fn from_home(home: &PreparedOrdinaryNativeHomeV1) -> Self {
        Self {
            history_state: Arc::clone(&home.history.in_use),
            native_open: Arc::clone(&home.effect.native_open),
            gateway: Arc::clone(&home.gateway),
            #[cfg(test)]
            runtime: home.effect.authenticated_runtime(),
        }
    }

    async fn dispose(&self) -> Result<(), AgentWrapperError> {
        self.native_open
            .store(false, std::sync::atomic::Ordering::Release);
        let _terminal =
            self.gateway
                .finish_owned()
                .await
                .map_err(|error| AgentWrapperError::Backend {
                    message: format!("ordinary owned gateway terminal disposal failed: {error:#}"),
                })?;
        #[cfg(test)]
        if let Some(root) = std::env::var_os("SUBSTRATE_TEST_ORDINARY_ROOT") {
            use std::os::unix::process::ExitStatusExt as _;
            let (pid, status) = _terminal;
            let record = fs::OpenOptions::new().create(true).append(true)
                .open(Path::new(&root).join("gateway-terminals.jsonl"))
                .and_then(|mut log| writeln!(log, "{}", json!({
                    "component_only":true,"pid":pid,"code":status.code(),
                    "signal":status.signal(),"runtime_id":self.gateway.runtime_id,
                    "native_open":self.native_open.load(std::sync::atomic::Ordering::Acquire),
                    "runtime":self.runtime,"owned_child_terminal":true,
                    "manifest_and_entry_disposed":true,
                })));
            if let Err(error) = record {
                return Err(AgentWrapperError::Backend {
                    message: format!(
                        "could not retain ordinary gateway terminal observation: {error}"
                    ),
                });
            }
        }
        Ok(())
    }
}

async fn ordinary_route_deadline(deadline: Option<tokio::time::Instant>) {
    match deadline {
        Some(deadline) => tokio::time::sleep_until(deadline).await,
        None => std::future::pending().await,
    }
}

fn ordinary_route_result<T>(
    outcome: Result<T, AgentWrapperError>,
    disposal: Result<(), AgentWrapperError>,
) -> Result<T, AgentWrapperError> {
    match disposal {
        Ok(()) => outcome,
        Err(error) => Err(AgentWrapperError::Backend {
            message: format!(
                "{error}; original backend outcome: {}",
                outcome
                    .as_ref()
                    .err()
                    .map(ToString::to_string)
                    .unwrap_or_else(|| "backend completion received".into())
            ),
        }),
    }
}

/// Own startup, home and exclusion before the first startup await. Dropping the
/// delivery future revokes the route concurrently; it never drops native startup
/// or its custody before the SDK returns its control/error. Successful delivery
/// transfers custody into the retained completion owner before exposing control.
fn retain_ordinary_native_startup<C, F>(
    startup: F,
    route: OrdinaryNativeRouteDisposalV1,
    custody: C,
    deadline: Option<tokio::time::Instant>,
) -> impl std::future::Future<Output = Result<PromptFulfillmentRunControl, AgentWrapperError>>
where
    C: Send + 'static,
    F: std::future::Future<Output = Result<PromptFulfillmentRunControl, AgentWrapperError>>
        + Send
        + 'static,
{
    let (mut sender, receiver) = tokio::sync::oneshot::channel();
    tokio::spawn(async move {
        let mut startup = Box::pin(startup);
        let (outcome, early_disposal) = tokio::select! {
            outcome = &mut startup => (outcome, None),
            _ = sender.closed() => {
                let (outcome, disposal) = tokio::join!(startup, route.dispose());
                (outcome, Some(disposal))
            },
            // The SDK owns its timeout outcome. This timer only revokes routes;
            // invoking cancel here could replace a normative timeout with cancelled.
            _ = ordinary_route_deadline(deadline) => {
                let (outcome, disposal) = tokio::join!(startup, route.dispose());
                (outcome, Some(disposal))
            },
        };
        let outcome = match outcome {
            Ok(mut control) => {
                // Even a route-disposal failure cannot drop a returned native
                // control before its completion/custody owner has been installed.
                if let Some(disposal) = early_disposal {
                    let completion = control.handle.completion;
                    control.handle.completion =
                        Box::pin(async move { ordinary_route_result(completion.await, disposal) });
                }
                let terminate = control.cancel.clone();
                control.handle.completion = retain_ordinary_native_completion(
                    control.handle.completion,
                    route,
                    control.terminal.clone(),
                    custody,
                    move || terminate.cancel(),
                    deadline,
                );
                Ok(control)
            }
            Err(error) => {
                let disposal = match early_disposal {
                    Some(disposal) => disposal,
                    None => route.dispose().await,
                };
                ordinary_route_result(Err(error), disposal)
            }
        };
        // Failed delivery drops the control whose independent completion owner
        // requests cancellation and continues to retain the actual native custody.
        let _ = sender.send(outcome);
    });
    async move {
        receiver.await.map_err(|_| AgentWrapperError::Backend {
            message: "submission_outcome_unknown: ordinary startup owner abandoned".into(),
        })?
    }
}

/// Normal returned-control completion includes exact owned gateway disposal.
/// The existing independent supervisor retains this future if its caller drops.
fn retain_ordinary_native_completion<C: Send + 'static>(
    mut completion: agent_api::DynAgentWrapperCompletion,
    route: OrdinaryNativeRouteDisposalV1,
    native_terminal: NativeTerminalObservation,
    custody: C,
    terminate: impl FnOnce() + Send + 'static,
    deadline: Option<tokio::time::Instant>,
) -> agent_api::DynAgentWrapperCompletion {
    let abandonment = native_terminal.clone();
    let completion = Box::pin(async move {
        let ready = tokio::select! {
            outcome = &mut completion => Some(outcome),
            _ = native_terminal.route_disposal_requested() => None,
            _ = ordinary_route_deadline(deadline) => None,
        };
        let (outcome, disposal) = match ready {
            Some(outcome) => (outcome, route.dispose().await),
            None => tokio::join!(completion, route.dispose()),
        };
        let observed = native_terminal.wait().await;
        #[cfg(test)]
        if let Some(root) = std::env::var_os("SUBSTRATE_TEST_ORDINARY_ROOT") {
            let record = fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(Path::new(&root).join("native-history-settlements.jsonl"))
                .and_then(|mut log| {
                    writeln!(
                        log,
                        "{}",
                        json!({
                            "component_only":true,"runtime":route.runtime,
                            "checked":observed.require_checked().is_ok(),
                        })
                    )
                });
            if let Err(error) = record {
                eprintln!("could not preserve checked native history settlement: {error}");
            }
        }
        if observed.require_checked().is_err() {
            // The publisher may retire the member later. No retained/captured
            // source may acquire this shared store in the intervening window.
            route.history_state.poison();
            tracing::warn!("owned gateway disposed with native acknowledgment unconfirmed");
        }
        // Raw SDK completion is preserved. The enclosing publisher checks the
        // independent native outcome and cannot treat route disposal as native ACK.
        ordinary_route_result(outcome, disposal)
    });
    retain_ordinary_world_completion(completion, custody, move || {
        abandonment.request_route_disposal();
        terminate();
    })
}

/// Keep actual ordinary resources through SDK terminal observation even when a
/// later fallible registration/persistence step drops the public completion.
fn retain_ordinary_world_completion<C: Send + 'static>(
    mut completion: agent_api::DynAgentWrapperCompletion,
    custody: C,
    terminate: impl FnOnce() + Send + 'static,
) -> agent_api::DynAgentWrapperCompletion {
    let (mut sender, receiver) = tokio::sync::oneshot::channel();
    tokio::spawn(async move {
        let result = tokio::select! {
            result = &mut completion => result,
            _ = sender.closed() => { terminate(); completion.await },
        };
        drop(custody);
        let _ = sender.send(result);
    });
    Box::pin(async move {
        receiver.await.map_err(|_| AgentWrapperError::Backend {
            message: "ordinary World supervisor ended before SDK terminal observation".into(),
        })?
    })
}

fn cleanup_prepared_ordinary_member_launcher(
    launcher_dir: &mut Option<std::path::PathBuf>,
    ordinary: bool,
) {
    if ordinary {
        // Even failed/retired ordinary histories remain private data. Neither
        // this pathname nor an orphan directory can reconstruct admission.
        launcher_dir.take();
    } else {
        cleanup_prepared_launcher_dir(launcher_dir);
    }
}

struct PreparedSubmittedTurnPolicyLauncher {
    path: std::path::PathBuf,
}

impl Drop for PreparedSubmittedTurnPolicyLauncher {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.path);
    }
}

#[derive(Clone)]
struct MemberStreamContext {
    orchestration_session_id: String,
    run_id: String,
    acceptance_context: Option<transport_api_types::WorldWorkAcceptanceContextV1>,
    participant_id: String,
    orchestrator_participant_id: String,
    parent_participant_id: Option<String>,
    resumed_from_participant_id: Option<String>,
    backend_id: String,
    protocol: serde_json::Value,
    authenticated_thread_id: Option<String>,
}

struct PreparedMemberRuntimeEvent {
    event: AgentEvent,
    worker_event_facet: Option<NormalizedWorldWorkerEventFacetV1>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum MemberStreamMode {
    Bootstrap,
    SubmittedTurn,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
struct RetainedMemberKey {
    orchestration_session_id: String,
    world_generation: u64,
    backend_id: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct RetainedMemberSlot {
    primary_participant_id: String,
    fork_child_participant_id: Option<String>,
}

impl MemberRuntimeManager {
    pub(crate) fn new() -> Self {
        let privileged_child_exclusion = E3PrivilegedChildExclusionV1::new_recovering()
            .expect("bind standalone member-runtime child exclusion");
        privileged_child_exclusion
            .finish_recovery()
            .expect("finish standalone member-runtime child-exclusion recovery");
        Self {
            active_members: Arc::default(),
            active_turns_by_span_id: Arc::default(),
            runtime_replay: RuntimeReplayRegistry::default(),
            member_turn_join: None,
            privileged_child_exclusion,
            #[cfg(test)]
            ordinary_native_fixture: None,
        }
    }

    #[allow(dead_code)]
    pub(crate) fn with_replay_registry(
        runtime_replay: RuntimeReplayRegistry,
        member_turn_join: Option<MemberTurnJoinRegistry>,
    ) -> Self {
        Self {
            runtime_replay,
            member_turn_join,
            ..Self::new()
        }
    }

    pub(crate) fn with_replay_and_e3_projection_service(
        runtime_replay: RuntimeReplayRegistry,
        member_turn_join: Option<MemberTurnJoinRegistry>,
        privileged_child_exclusion: Arc<E3PrivilegedChildExclusionV1>,
    ) -> Self {
        Self {
            active_members: Arc::default(),
            active_turns_by_span_id: Arc::default(),
            runtime_replay,
            member_turn_join,
            privileged_child_exclusion,
            #[cfg(test)]
            ordinary_native_fixture: None,
        }
    }

    /// Unit-test composition only; never called by production service construction.
    #[cfg(test)]
    pub(crate) fn with_checked_ordinary_native_fixture_for_test(
        mut self,
        path: &Path,
        digest: &str,
    ) -> Result<Self> {
        {
            let members = self
                .active_members
                .read()
                .map_err(|_| anyhow!("members lock poisoned"))?;
            if self.ordinary_native_fixture.is_some()
                || !members.ordinary_pending_participants.is_empty()
                || !members.by_participant_id.is_empty()
                || !members.by_retained_key.is_empty()
                || !self
                    .active_turns_by_span_id
                    .read()
                    .map_err(|_| anyhow!("turns lock poisoned"))?
                    .is_empty()
            {
                anyhow::bail!("component executable must be composed before runtime admission");
            }
        }
        self.ordinary_native_fixture = Some(Arc::new(CheckedOrdinaryNativeFixtureV1::open(
            path, digest,
        )?));
        Ok(self)
    }

    pub(crate) async fn launch(
        &self,
        agent_id: String,
        env: HashMap<String, String>,
        span_id: String,
        admission: MemberRuntimeLaunchAdmissionV1,
        binding: SharedWorldBindingSnapshot,
        placement: LinuxWorldPlacementContext,
    ) -> Result<Response> {
        let ordinary_launch = admission.ordinary_effect.is_some();
        let MemberRuntimeLaunchAdmissionV1 {
            dispatch,
            acceptance_context,
            policy_snapshot,
            ordinary_effect,
        } = admission;
        let ordinary_launch_reservation = ordinary_effect
            .as_ref()
            .map(|effect| self.reserve_ordinary_launch(effect, &span_id))
            .transpose()?;
        if let Some(effect) = ordinary_effect.as_ref() {
            effect.revalidate_for_effect()?;
        }
        if let Some(context) = acceptance_context.as_ref() {
            context
                .validate()
                .map_err(crate::service::BadRequestError::new)?;
            if context.request_id != dispatch.run_id || context.message_id.is_some() {
                return Err(crate::service::BadRequestError::new(
                    "task acceptance context does not match member dispatch".to_string(),
                )
                .into());
            }
        }
        validate_retained_worker_launch_authority_proof(&dispatch, &binding)?;
        let pinned_e2_activation = validate_e2_member_launch_activation(
            &dispatch,
            &binding,
            &policy_snapshot,
            acceptance_context.as_ref(),
        )?;
        let requires_exact_registered_readiness =
            dispatch.retained_worker_launch_authority.is_some() || pinned_e2_activation.is_some();
        #[cfg(test)]
        let ordinary_native_fixture = match (&self.ordinary_native_fixture, &ordinary_effect) {
            (Some(fixture), Some(effect))
                if dispatch
                    .e2_launch_activation
                    .as_ref()
                    .is_some_and(|activation| {
                        activation.launch_kind == E2MemberLaunchKindV1::Fork
                    }) =>
            {
                let source_id = dispatch
                    .e2_launch_activation
                    .as_ref()
                    .unwrap()
                    .source_participant_id
                    .as_ref()
                    .ok_or_else(|| anyhow!("component Fork omitted exact source"))?;
                let source = self
                    .active_members
                    .read()
                    .map_err(|_| anyhow!("component source registry lock poisoned"))?
                    .by_participant_id
                    .get(source_id)
                    .cloned()
                    .ok_or_else(|| anyhow!("component Fork source is not held"))?;
                let source_effect = source
                    .ordinary_effect
                    .as_ref()
                    .ok_or_else(|| anyhow!("component source has no original admission"))?;
                Some(fixture.for_authenticated_fork(effect, source_effect)?)
            }
            _ => self.ordinary_native_fixture.clone(),
        };
        #[cfg(not(test))]
        let actual_binary = validate_member_runtime_binary(&dispatch)?;
        #[cfg(test)]
        let actual_binary = match (&ordinary_native_fixture, &ordinary_effect) {
            (Some(fixture), Some(effect)) => {
                fixture.bind_dispatch(effect, &dispatch, &binding, &placement.working_dir)?
            }
            (Some(_), None) => {
                anyhow::bail!("component executable cannot replace nonordinary admission")
            }
            (None, _) => validate_member_runtime_binary(&dispatch)?,
        };
        let ordinary_fork_source = match ordinary_effect.as_ref() {
            Some(effect)
                if dispatch
                    .e2_launch_activation
                    .as_ref()
                    .is_some_and(|activation| {
                        activation.launch_kind == E2MemberLaunchKindV1::Fork
                    }) =>
            {
                Some(self.find_ordinary_fork_source(
                    &dispatch,
                    effect,
                    &placement.working_dir,
                    &binding,
                    &actual_binary,
                )?)
            }
            _ => None,
        };
        // Durable reception precedes launcher/home/SDK preparation. A restarted
        // service cannot turn an exact already-claimed Execute into new input.
        if let Some(effect) = ordinary_effect.as_ref() {
            effect.claim_initial_bootstrap_receiver(&span_id)?;
            let reservation = ordinary_launch_reservation
                .as_ref()
                .ok_or_else(|| anyhow!("ordinary receiver actual reservation absent"))?;
            effect.adopt_stored_initial_bootstrap_controls(&span_id, |request| {
                reservation
                    .pending
                    .control
                    .request_initial_control(&request.signal)?;
                Ok(())
            })?;
        }
        let PreparedMemberRuntimeLauncher {
            mut launcher_path,
            launcher_dir,
            env: launcher_env,
        } = prepare_member_runtime_launcher(&actual_binary, &placement)?;
        let runtime_env = env
            .into_iter()
            .chain(launcher_env.into_iter())
            .collect::<BTreeMap<_, _>>();
        let mut runtime_env = runtime_env;
        let process_working_dir = member_runtime_process_working_dir(
            dispatch.resolved_runtime.backend_kind,
            &placement.working_dir,
            &launcher_dir,
        );
        let mut prepared_launcher_dir = Some(launcher_dir.clone());
        let ordinary_native_home = match ordinary_effect.as_ref() {
            Some(effect) => match prepare_ordinary_native_runtime_env(
                &mut runtime_env,
                effect,
                &placement.working_dir,
                &launcher_dir,
                match ordinary_fork_source.as_ref() {
                    Some(source) => OrdinaryNativeHistoryRequestV1::Fork(Arc::clone(source)),
                    None => OrdinaryNativeHistoryRequestV1::InitialStart,
                },
                #[cfg(test)]
                ordinary_native_fixture.clone(),
            )
            .await
            {
                Ok(home) => Some(home),
                Err(error) => {
                    cleanup_prepared_ordinary_member_launcher(
                        &mut prepared_launcher_dir,
                        ordinary_launch,
                    );
                    return Err(error);
                }
            },
            None => {
                if let Err(err) = prepare_runtime_env_for_member_backend(
                    &mut runtime_env,
                    dispatch.resolved_runtime.backend_kind,
                    &placement.working_dir,
                    &launcher_dir,
                ) {
                    cleanup_prepared_ordinary_member_launcher(
                        &mut prepared_launcher_dir,
                        ordinary_launch,
                    );
                    return Err(err);
                }
                None
            }
        };

        let mut ordinary_start_policy_launcher = None;
        if ordinary_native_home.is_some() {
            match prepare_ordinary_start_home_launcher(
                &actual_binary,
                &launcher_dir,
                &policy_snapshot,
                &placement,
                &process_working_dir,
                &mut runtime_env,
            ) {
                Ok(launcher) => {
                    launcher_path = launcher.path.clone();
                    ordinary_start_policy_launcher = Some(launcher);
                }
                Err(error) => {
                    cleanup_prepared_ordinary_member_launcher(&mut prepared_launcher_dir, true);
                    return Err(error);
                }
            }
        }
        let prompt_fulfillment = match PromptFulfillmentBridge::for_member_backend(
            &dispatch.resolved_runtime.backend_kind,
            launcher_path,
        ) {
            Ok(prompt_fulfillment) => prompt_fulfillment,
            Err(err) => {
                cleanup_prepared_ordinary_member_launcher(
                    &mut prepared_launcher_dir,
                    ordinary_launch,
                );
                return Err(err);
            }
        };
        let initial_prompt = match dispatch.initial_prompt.clone() {
            Some(initial_prompt) => initial_prompt,
            None => {
                cleanup_prepared_ordinary_member_launcher(
                    &mut prepared_launcher_dir,
                    ordinary_launch,
                );
                return Err(crate::service::BadRequestError::new(
                    "member_dispatch.initial_prompt is required for launch-time first turn"
                        .to_string(),
                )
                .into());
            }
        };
        let mut launch_extensions = member_runtime_workspace_access_extensions(
            dispatch.resolved_runtime.backend_kind,
            &placement.working_dir,
        );
        if let Some(source) = ordinary_fork_source.as_ref() {
            // Pinned UAA Fork rejects model/add-dir overrides. Selection comes
            // from the authenticated source and the same held rendered home.
            launch_extensions.clear();
            launch_extensions.insert(
                SESSION_FORK_EXTENSION_V1.into(),
                json!({"selector":"id","id":source.session_id}),
            );
        } else if let Some(effect) = ordinary_effect.as_ref() {
            launch_extensions.insert(
                "agent_api.config.model.v1".into(),
                json!(effect.requested_selection().routed_model_id),
            );
        }
        let privileged_child_exclusion_lease = Arc::new(
            self.privileged_child_exclusion
                .acquire_non_e3_child()
                .context("UnsupportedSecurityPosture: member child admission is closed")?,
        );
        if let Some(effect) = ordinary_effect.as_ref() {
            if let Err(error) = effect.revalidate_for_effect().and_then(|_| {
                ordinary_native_home
                    .as_ref()
                    .ok_or_else(|| anyhow!("ordinary native home missing"))?
                    .revalidate()
            }) {
                cleanup_prepared_ordinary_member_launcher(
                    &mut prepared_launcher_dir,
                    ordinary_launch,
                );
                return Err(error);
            }
        }
        let PromptFulfillmentRunControl {
            handle,
            cancel,
            terminal,
        } = match run_ordinary_native_control(
            Arc::new(prompt_fulfillment),
            AgentWrapperRunRequest {
                prompt: initial_prompt,
                working_dir: Some(process_working_dir.clone()),
                timeout: None,
                env: runtime_env.clone(),
                extensions: launch_extensions,
            },
            ordinary_native_home.clone(),
            ordinary_native_home.as_ref().map(|_| {
                (
                    Arc::clone(&privileged_child_exclusion_lease),
                    ordinary_start_policy_launcher.take(),
                )
            }),
        )
        .await
        {
            Ok(control) => control,
            Err(err) => {
                cleanup_prepared_ordinary_member_launcher(
                    &mut prepared_launcher_dir,
                    ordinary_launch,
                );
                return Err(map_wrapper_error(err));
            }
        };

        if let Some(reservation) = &ordinary_launch_reservation {
            if let Err(error) = reservation
                .pending
                .control
                .bind_native_control(cancel.clone(), terminal.clone())
            {
                let error = join_failed_control(error, handle, cancel, terminal).await;
                cleanup_prepared_ordinary_member_launcher(
                    &mut prepared_launcher_dir,
                    ordinary_launch,
                );
                return Err(error);
            }
            if reservation.pending.control.initial_control_requested()? {
                reservation
                    .pending
                    .effect
                    .native_open
                    .store(false, std::sync::atomic::Ordering::Release);
            }
        }
        let cleanup_cancel = cancel.clone();

        let ordinary_effect = match ordinary_effect {
            Some(effect) => {
                // These are the actual validated arguments used by the owned
                // native control above, joined to the observed backend binding.
                let actual_runtime =
                    transport_api_types::OrdinaryGatewayRuntimeIdentityV1::WorldWorker {
                        orchestration_session_id: dispatch.orchestration_session_id.clone(),
                        caller_participant_id: dispatch.orchestrator_participant_id.clone(),
                        participant_id: dispatch.participant_id.clone(),
                        backend_id: dispatch.backend_id.clone(),
                        run_id: dispatch.run_id.clone(),
                        world_id: binding.world_id.clone(),
                        world_generation: binding.world_generation,
                    };
                let owner = effect
                    .bind_live_bootstrap_effect(&actual_runtime, &binding)
                    .or_else(|error| {
                        if effect
                            .native_open
                            .load(std::sync::atomic::Ordering::Acquire)
                        {
                            Err(error)
                        } else {
                            effect.retain_sealed_bootstrap_owner(&actual_runtime, &binding)
                        }
                    });
                match owner {
                    Ok(live) => Some(live),
                    Err(error) => {
                        let error = join_failed_control(
                            error,
                            handle,
                            cleanup_cancel.clone(),
                            terminal.clone(),
                        )
                        .await;
                        drop(ordinary_native_home);
                        cleanup_prepared_ordinary_member_launcher(
                            &mut prepared_launcher_dir,
                            ordinary_launch,
                        );
                        return Err(error);
                    }
                }
            }
            None => None,
        };

        let active = Arc::new(ActiveMemberRuntime {
            agent_id,
            participant_id: dispatch.participant_id.clone(),
            orchestration_session_id: dispatch.orchestration_session_id.clone(),
            orchestrator_participant_id: dispatch.orchestrator_participant_id.clone(),
            parent_participant_id: dispatch.parent_participant_id.clone(),
            resumed_from_participant_id: dispatch.resumed_from_participant_id.clone(),
            backend_id: dispatch.backend_id.clone(),
            backend_kind: dispatch.resolved_runtime.backend_kind,
            binary_path: actual_binary,
            launcher_dir: prepared_launcher_dir
                .take()
                .expect("prepared launcher dir must exist while bootstrap is active"),
            workspace_dir: placement.working_dir.clone(),
            process_working_dir,
            env: runtime_env,
            ordinary_effect,
            ordinary_native_history: ordinary_native_home
                .as_ref()
                .map(|home| Arc::clone(&home.history)),
            ordinary_native_home: Mutex::new(ordinary_native_home),
            binding: binding.clone(),
            protocol: json!(dispatch.protocol),
            bootstrap: Mutex::new(Some(ActiveBootstrapRuntime {
                span_id: span_id.clone(),
                terminal: Some(terminal.clone()),
                cancel: Some(cancel),
                last_signal: None,
                initial_control: ordinary_launch_reservation
                    .as_ref()
                    .map(|reservation| Arc::clone(&reservation.pending.control)),
            })),
            active_turn_span_id: Mutex::new(None),
            uaa_session_id: Mutex::new(None),
            e2_launch_activation: dispatch.e2_launch_activation.clone(),
            authenticated_worker_cap_identity: Mutex::new(pinned_e2_activation),
            privileged_child_exclusion_lease,
        });
        if let Err(err) = self.register_member(active.clone()) {
            let err =
                join_failed_control(err, handle, cleanup_cancel.clone(), terminal.clone()).await;
            active.close_bootstrap();
            active.cleanup_launcher_dir();
            return Err(err);
        }

        let context = MemberStreamContext {
            orchestration_session_id: dispatch.orchestration_session_id.clone(),
            run_id: dispatch.run_id.clone(),
            acceptance_context,
            participant_id: dispatch.participant_id.clone(),
            orchestrator_participant_id: dispatch.orchestrator_participant_id.clone(),
            parent_participant_id: dispatch.parent_participant_id.clone(),
            resumed_from_participant_id: dispatch.resumed_from_participant_id.clone(),
            backend_id: dispatch.backend_id.clone(),
            protocol: json!(dispatch.protocol),
            authenticated_thread_id: None,
        };
        let (mut producer, start) = match start_member_runtime_stream(
            &context,
            MemberStreamMode::Bootstrap,
            span_id.clone(),
        ) {
            Ok(stream) => stream,
            Err(error) => {
                let error =
                    join_failed_control(error, handle, cleanup_cancel.clone(), terminal.clone())
                        .await;
                self.unregister_member(&active.participant_id);
                return Err(error);
            }
        };
        let replay_publisher = match self
            .runtime_replay
            .begin_for_acceptance(context.acceptance_context.as_ref(), &start)
        {
            Ok(publisher) => publisher,
            Err(error) => {
                let error =
                    join_failed_control(error, handle, cleanup_cancel.clone(), terminal.clone())
                        .await;
                self.unregister_member(&active.participant_id);
                return Err(error);
            }
        };
        let (tx, rx) = tokio::sync::mpsc::unbounded_channel::<ExecuteStreamFrame>();
        let _ = tx.send(start);

        // The active bootstrap now owns the same latch. Release the actual
        // pending reservation before a fast completion can publish retirement.
        drop(ordinary_launch_reservation);
        let manager = self.clone();
        let participant_id = dispatch.participant_id.clone();
        tokio::spawn(async move {
            let mut events = handle.events;
            let completion = handle.completion;
            let mut emitted_registered = false;
            let mut pending_authority_bootstrap_events = Vec::new();

            while let Some(wrapper_event) = tokio::select! {
                _ = tx.closed() => { active.cancel_bootstrap(); None },
                event = events.next() => event,
            } {
                if let Some(session_id) =
                    surfaced_uaa_session_id_from_data(wrapper_event.data.as_ref())
                {
                    manager.remember_uaa_session_id(&participant_id, session_id);
                }
                if requires_exact_registered_readiness
                    && !emitted_registered
                    && registered_event_from_data(
                        &context,
                        &binding,
                        &span_id,
                        wrapper_event.data.as_ref(),
                        active.agent_id.as_str(),
                    )
                    .is_none()
                {
                    const MAX_PRE_REGISTERED_BOOTSTRAP_EVENTS: usize = 64;
                    if pending_authority_bootstrap_events.len()
                        == MAX_PRE_REGISTERED_BOOTSTRAP_EVENTS
                    {
                        tracing::error!(
                            "authority-managed member bootstrap exceeded pre-registration event bound"
                        );
                        active.cancel_bootstrap();
                        break;
                    }
                    pending_authority_bootstrap_events.push(wrapper_event);
                    continue;
                }
                match frame_from_wrapper_event(
                    &context,
                    &binding,
                    &span_id,
                    wrapper_event,
                    &mut emitted_registered,
                    MemberStreamMode::Bootstrap,
                    active.agent_id.as_str(),
                    &mut producer,
                ) {
                    Ok(Some(frame)) => {
                        publish_member_replay_frame(
                            replay_publisher.as_ref(),
                            &tx,
                            frame,
                            "member bootstrap Event",
                        );
                    }
                    Ok(None) => {}
                    Err(err) => {
                        tracing::error!(error = %err, "failed to identify member bootstrap Event");
                        break;
                    }
                }
                if requires_exact_registered_readiness && emitted_registered {
                    for pending_event in pending_authority_bootstrap_events.drain(..) {
                        match frame_from_wrapper_event(
                            &context,
                            &binding,
                            &span_id,
                            pending_event,
                            &mut emitted_registered,
                            MemberStreamMode::Bootstrap,
                            active.agent_id.as_str(),
                            &mut producer,
                        ) {
                            Ok(Some(frame)) => {
                                publish_member_replay_frame(
                                    replay_publisher.as_ref(),
                                    &tx,
                                    frame,
                                    "buffered member bootstrap Event",
                                );
                            }
                            Ok(None) => {}
                            Err(err) => {
                                tracing::error!(
                                    error = %err,
                                    "failed to identify buffered member bootstrap Event"
                                );
                                break;
                            }
                        }
                    }
                }
            }

            drop(events);
            terminal.request_route_disposal();
            let raw_completion = completion.await;
            let native_terminal = terminal.wait().await;
            let completion = native_terminal.enclosing_result(raw_completion);
            if let Ok(ref completion) = completion {
                if let Some(session_id) =
                    surfaced_uaa_session_id_from_data(completion.data.as_ref())
                {
                    manager.remember_uaa_session_id(&participant_id, session_id);
                }
            }
            if requires_exact_registered_readiness && !emitted_registered {
                if let Ok(ref completion) = completion {
                    if let Some(event) = registered_event_from_data(
                        &context,
                        &binding,
                        &span_id,
                        completion.data.as_ref(),
                        active.agent_id.as_str(),
                    ) {
                        emitted_registered = true;
                        match producer.event(event) {
                            Ok(frame) => {
                                publish_member_replay_frame(
                                    replay_publisher.as_ref(),
                                    &tx,
                                    frame,
                                    "member bootstrap completion registration",
                                );
                            }
                            Err(err) => tracing::error!(
                                error = %err,
                                "failed to identify member bootstrap completion registration"
                            ),
                        }
                        for pending_event in pending_authority_bootstrap_events.drain(..) {
                            match frame_from_wrapper_event(
                                &context,
                                &binding,
                                &span_id,
                                pending_event,
                                &mut emitted_registered,
                                MemberStreamMode::Bootstrap,
                                active.agent_id.as_str(),
                                &mut producer,
                            ) {
                                Ok(Some(frame)) => {
                                    publish_member_replay_frame(
                                        replay_publisher.as_ref(),
                                        &tx,
                                        frame,
                                        "buffered member bootstrap completion Event",
                                    );
                                }
                                Ok(None) => {}
                                Err(err) => {
                                    tracing::error!(
                                        error = %err,
                                        "failed to identify buffered member bootstrap completion Event"
                                    );
                                    break;
                                }
                            }
                        }
                    }
                }
            }
            let mut preserve_retained_member = completion.as_ref().ok().is_some_and(|completion| {
                should_preserve_retained_member_after_clean_bootstrap(
                    active.as_ref(),
                    &completion.status,
                )
            });
            let mut initial_retirement = None;
            let frames = if preserve_retained_member && active.ordinary_effect.is_some() {
                match completion {
                    Ok(completion) => {
                        prepare_retained_native_bootstrap_frames(
                            active.as_ref(),
                            &context,
                            &binding,
                            &span_id,
                            completion,
                            &terminal,
                            emitted_registered,
                            &mut producer,
                        )
                        .await
                    }
                    Err(error) => Err(anyhow!(error)),
                }
            } else if active.ordinary_effect.is_some() && context.acceptance_context.is_none() {
                let logical_exit = match &completion {
                    Ok(completion) => exit_code_from_status(&completion.status),
                    Err(AgentWrapperError::Backend { message }) if message == CANCELLED_MESSAGE => {
                        cancel_exit_code(active.bootstrap_last_signal().as_deref())
                    }
                    Err(_) => 1,
                };
                match manager
                    .prepare_failed_ordinary_initial_bootstrap_frames(
                        &active,
                        &context,
                        &binding,
                        &span_id,
                        logical_exit,
                        &mut producer,
                    )
                    .await
                {
                    Ok(prepared) => {
                        initial_retirement = Some(prepared.receipt);
                        Ok(prepared.frames)
                    }
                    Err(error) => Err(error),
                }
            } else {
                frames_from_completion(
                    &context,
                    &binding,
                    &span_id,
                    completion,
                    active.bootstrap_last_signal(),
                    &mut emitted_registered,
                    MemberStreamMode::Bootstrap,
                    active.agent_id.as_str(),
                    &mut producer,
                )
            };
            if frames.is_err() {
                preserve_retained_member = false;
            }
            manager.finish_bootstrap(&participant_id, preserve_retained_member);
            // In particular, the failed initial report may become observable
            // only after this task releases its actual member/exclusion owner.
            drop(active);
            // Publish the cache only after releasing actual member custody;
            // concurrent control requests must not observe premature retirement.
            if let Some(receipt) = initial_retirement {
                let observed = receipt.report.observation();
                let key = (
                    observed.source_start.authority_store_id.clone(),
                    observed.activation.orchestration_session_id.clone(),
                    observed.activation.retained_participant_id.clone(),
                    observed.service_span_id.clone(),
                );
                match manager.active_members.write() {
                    Ok(mut registry) => { registry.ordinary_initial_retirements.insert(key, receipt); },
                    Err(_) => tracing::error!("initial retirement receipt cache unavailable; original stream report remains authoritative data"),
                }
            }
            match frames {
                Ok(frames) => {
                    for frame in frames {
                        publish_member_replay_frame(
                            replay_publisher.as_ref(),
                            &tx,
                            frame,
                            "member bootstrap completion",
                        );
                    }
                }
                Err(err) => {
                    tracing::error!(error = %err, "failed to identify member bootstrap completion");
                    // Missing/failed native acknowledgment is not logical-worker
                    // retirement. The client records interrupted ownership.
                    if let Ok(frame) = producer
                        .transport_error(format!("native bootstrap completion unconfirmed: {err}"))
                    {
                        publish_member_replay_frame(
                            replay_publisher.as_ref(),
                            &tx,
                            frame,
                            "unconfirmed native bootstrap",
                        );
                    }
                }
            }
        });

        stream_response(rx)
    }

    pub(crate) async fn submit_ordinary_turn(
        &self,
        req: transport_api_types::OrdinaryMemberTurnSubmitRequestV1,
    ) -> Result<Response> {
        let req = req.into_request();
        let active = self.find_submit_target(&req)?;
        let bootstrap_effect = active.ordinary_effect.as_ref().ok_or_else(|| {
            crate::service::BadRequestError::new(
                "ordinary retained intake requires an actual held ordinary member".to_string(),
            )
        })?;
        // This joins the actual received B1 request hash and independently
        // authenticated E2 cap under configured current authority. The route
        // and zero-capable decoder cannot select or manufacture admission.
        let effect = bootstrap_effect.prepare_turn(&req)?;
        let caller = authenticated_ordinary_turn_caller(active.as_ref(), &effect)?;
        validate_submit_turn_request(
            &req,
            RetainedMemberIdentity {
                orchestrator_participant_id: &caller,
                ..RetainedMemberIdentity::from_active(active.as_ref())
            },
        )?;
        self.validate_submit_target_slot(&req)?;
        effect.revalidate_for_effect()?;
        active
            .ordinary_native_history
            .as_ref()
            .ok_or_else(|| anyhow!("ordinary Resume omitted original worker history"))?;
        let registry = self
            .member_turn_join
            .as_ref()
            .ok_or_else(|| anyhow::Error::new(MemberTurnJoinError::UnsupportedLegacyState))?;
        if let Some(reservation) = registry
            .join_existing_ordinary(&req, effect.turn_admission()?)
            .map_err(anyhow::Error::new)?
        {
            if let Some(response) = self.replay_joined_e2_turn(registry, reservation)? {
                return Ok(response);
            }
        }
        let snapshot = resolve_submitted_turn_policy(&req, active.as_ref())?;
        let session = active.uaa_session_id().ok_or_else(|| {
            crate::service::BadRequestError::new(
                "ordinary retained member has no surfaced native session".to_string(),
            )
        })?;
        self.submit_e2_turn(req, active, snapshot, session, Some(effect))
            .await
    }

    pub(crate) async fn submit_turn(&self, req: MemberTurnSubmitRequestV1) -> Result<Response> {
        req.validate()
            .map_err(crate::service::BadRequestError::new)?;
        if req.policy_snapshot_carrier.is_some() && req.acceptance_context.is_some() {
            if let Some(response) = self.replay_existing_e2_turn(&req)? {
                return Ok(response);
            }
        }
        let active = self.find_submit_target(&req)?;
        let ordinary_effect = active
            .ordinary_effect
            .as_ref()
            .map(|effect| effect.prepare_turn(&req))
            .transpose()?;
        if let Some(effect) = ordinary_effect.as_ref() {
            effect.revalidate_for_effect()?;
            active
                .ordinary_native_history
                .as_ref()
                .ok_or_else(|| anyhow!("ordinary Resume omitted original worker history"))?;
        }
        validate_submit_turn_request(&req, RetainedMemberIdentity::from_active(active.as_ref()))?;
        let turn_policy_snapshot = resolve_submitted_turn_policy(&req, active.as_ref())?;
        self.validate_submit_target_slot(&req)?;
        let uaa_session_id = active.uaa_session_id().ok_or_else(|| {
            crate::service::BadRequestError::new(format!(
                "member_turn_submit.participant_id {} has no surfaced uaa_session_id",
                req.participant_id
            ))
        })?;

        if req.policy_snapshot_carrier.is_some() && req.acceptance_context.is_some() {
            return self
                .submit_e2_turn(
                    req,
                    active,
                    turn_policy_snapshot,
                    uaa_session_id,
                    ordinary_effect,
                )
                .await;
        }

        let span_id = format!("spn_{}", uuid::Uuid::now_v7());
        self.reserve_turn_slot(&active, &span_id)?;

        let mut turn_policy_launcher = match turn_policy_snapshot.as_ref() {
            Some(_) => match prepare_submitted_turn_policy_launcher(active.as_ref()) {
                Ok(launcher) => Some(launcher),
                Err(error) => {
                    self.clear_reserved_turn_slot(&active, &span_id);
                    return Err(error);
                }
            },
            None => None,
        };
        let prompt_fulfillment = PromptFulfillmentBridge::for_member_backend(
            &active.backend_kind,
            turn_policy_launcher.as_ref().map_or_else(
                || active.launcher_dir.join("world-entry.sh"),
                |launcher| launcher.path.clone(),
            ),
        );
        let prompt_fulfillment = match prompt_fulfillment {
            Ok(prompt_fulfillment) => prompt_fulfillment,
            Err(error) => {
                self.clear_reserved_turn_slot(&active, &span_id);
                return Err(error);
            }
        };
        let mut run_request = match build_submitted_turn_run_request(
            active.as_ref(),
            req.prompt.clone(),
            &uaa_session_id,
            turn_policy_snapshot.as_ref(),
        ) {
            Ok(request) => request,
            Err(error) => {
                self.clear_reserved_turn_slot(&active, &span_id);
                return Err(error);
            }
        };
        let ordinary_native_home = match ordinary_effect.as_ref() {
            Some(effect) => {
                match prepare_ordinary_native_runtime_env(
                    &mut run_request.env,
                    effect,
                    &active.workspace_dir,
                    &active.launcher_dir,
                    OrdinaryNativeHistoryRequestV1::Resume(Arc::clone(
                        active.ordinary_native_history.as_ref().ok_or_else(|| {
                            anyhow!("ordinary Resume omitted original worker history")
                        })?,
                    )),
                    #[cfg(test)]
                    self.ordinary_native_fixture.clone(),
                )
                .await
                {
                    Ok(home) => {
                        run_request.extensions.insert(
                            "agent_api.config.model.v1".into(),
                            json!(effect.requested_selection().routed_model_id),
                        );
                        Some(home)
                    }
                    Err(error) => {
                        self.clear_reserved_turn_slot(&active, &span_id);
                        return Err(error);
                    }
                }
            }
            None => None,
        };
        if let Some(effect) = ordinary_effect.as_ref() {
            if let Err(error) = effect.revalidate_for_effect().and_then(|_| {
                ordinary_native_home
                    .as_ref()
                    .ok_or_else(|| anyhow!("ordinary native home missing"))?
                    .revalidate()
            }) {
                self.clear_reserved_turn_slot(&active, &span_id);
                return Err(error);
            }
        }
        let PromptFulfillmentRunControl {
            handle,
            cancel,
            terminal,
        } = match run_ordinary_native_control(
            Arc::new(prompt_fulfillment),
            run_request,
            ordinary_native_home.clone(),
            ordinary_native_home.as_ref().map(|_| {
                (
                    Arc::clone(&active.privileged_child_exclusion_lease),
                    turn_policy_launcher.take(),
                )
            }),
        )
        .await
        {
            Ok(control) => control,
            Err(err) => {
                self.clear_reserved_turn_slot(&active, &span_id);
                return Err(map_wrapper_error(err));
            }
        };

        let cleanup_cancel = cancel.clone();

        let context = active.submit_context(req.run_id.clone(), req.acceptance_context.clone());
        let (mut producer, start) = match start_member_runtime_stream(
            &context,
            MemberStreamMode::SubmittedTurn,
            span_id.clone(),
        ) {
            Ok(stream) => stream,
            Err(error) => {
                let error =
                    join_failed_control(error, handle, cleanup_cancel.clone(), terminal.clone())
                        .await;
                if terminal.wait().await.require_checked().is_err() {
                    self.unregister_member(&active.participant_id);
                }
                self.clear_reserved_turn_slot(&active, &span_id);
                return Err(error);
            }
        };
        let replay_publisher = match self
            .runtime_replay
            .begin_for_acceptance(context.acceptance_context.as_ref(), &start)
        {
            Ok(publisher) => publisher,
            Err(error) => {
                let error =
                    join_failed_control(error, handle, cleanup_cancel.clone(), terminal.clone())
                        .await;
                if terminal.wait().await.require_checked().is_err() {
                    self.unregister_member(&active.participant_id);
                }
                self.clear_reserved_turn_slot(&active, &span_id);
                return Err(error);
            }
        };
        let turn = Arc::new(ActiveSubmittedTurn {
            terminal: terminal.clone(),
            ordinary_effect,
            ordinary_native_home,
            participant_id: active.participant_id.clone(),
            cancel,
            last_signal: Mutex::new(None),
        });
        self.register_turn(span_id.clone(), turn.clone());
        let (tx, rx) = tokio::sync::mpsc::unbounded_channel::<ExecuteStreamFrame>();
        let _ = tx.send(start);

        let manager = self.clone();
        let binding = active.binding.clone();
        tokio::spawn(async move {
            let _turn_policy_launcher = turn_policy_launcher;
            let mut events = handle.events;
            let completion = handle.completion;
            let mut emitted_registered = false;

            while let Some(wrapper_event) = tokio::select! {
                _ = tx.closed() => { turn.terminal.request_route_disposal(); turn.cancel.cancel(); None },
                event = events.next() => event,
            } {
                if let Some(session_id) =
                    surfaced_uaa_session_id_from_data(wrapper_event.data.as_ref())
                {
                    manager.remember_uaa_session_id(&turn.participant_id, session_id);
                }
                match frame_from_wrapper_event(
                    &context,
                    &binding,
                    &span_id,
                    wrapper_event,
                    &mut emitted_registered,
                    MemberStreamMode::SubmittedTurn,
                    active.agent_id.as_str(),
                    &mut producer,
                ) {
                    Ok(Some(frame)) => {
                        publish_member_replay_frame(
                            replay_publisher.as_ref(),
                            &tx,
                            frame,
                            "submitted member Event",
                        );
                    }
                    Ok(None) => {}
                    Err(err) => {
                        tracing::error!(error = %err, "failed to identify submitted member Event");
                        turn.terminal.request_route_disposal();
                        turn.cancel.cancel();
                        break;
                    }
                }
            }

            drop(events);
            terminal.request_route_disposal();
            let raw_completion = completion.await;
            let native_terminal = terminal.wait().await;
            let completion = native_terminal.enclosing_result(raw_completion);
            if let Ok(ref completion) = completion {
                if let Some(session_id) =
                    surfaced_uaa_session_id_from_data(completion.data.as_ref())
                {
                    manager.remember_uaa_session_id(&turn.participant_id, session_id);
                }
            }
            let frames = frames_from_completion(
                &context,
                &binding,
                &span_id,
                completion,
                turn.last_signal(),
                &mut emitted_registered,
                MemberStreamMode::SubmittedTurn,
                active.agent_id.as_str(),
                &mut producer,
            );
            match frames {
                Ok(frames) => {
                    for frame in frames {
                        publish_member_replay_frame(
                            replay_publisher.as_ref(),
                            &tx,
                            frame,
                            "submitted member completion",
                        );
                    }
                }
                Err(err) => {
                    tracing::error!(error = %err, "failed to identify submitted member completion");
                }
            }

            if matches!(
                &native_terminal.settlement,
                NativeSettlementOutcome::Checked(Err(_))
            ) {
                manager.unregister_member(&active.participant_id);
            }
            manager.unregister_turn(&span_id);
        });

        stream_response(rx)
    }

    fn replay_existing_e2_turn(
        &self,
        request: &MemberTurnSubmitRequestV1,
    ) -> Result<Option<Response>> {
        let registry = self
            .member_turn_join
            .as_ref()
            .ok_or_else(|| anyhow::Error::new(MemberTurnJoinError::UnsupportedLegacyState))?;
        let Some(reservation) = registry
            .join_existing(request)
            .map_err(anyhow::Error::new)?
        else {
            return Ok(None);
        };
        self.replay_joined_e2_turn(registry, reservation)
    }

    fn replay_joined_e2_turn(
        &self,
        registry: &MemberTurnJoinRegistry,
        reservation: crate::member_turn_join::MemberTurnJoinReservation,
    ) -> Result<Option<Response>> {
        let state = registry
            .recover_orphaned(&reservation)
            .map_err(anyhow::Error::new)?;
        match state {
            MemberTurnJoinStateV1::Reserved => Ok(None),
            MemberTurnJoinStateV1::FailedBeforeLaunch {
                error_code,
                http_status,
            } => Err(anyhow::Error::new(
                MemberTurnJoinError::FailedBeforeLaunch {
                    error_code,
                    http_status,
                },
            )),
            MemberTurnJoinStateV1::LaunchingPreCall { .. }
            | MemberTurnJoinStateV1::LaunchingCallEntered { .. }
            | MemberTurnJoinStateV1::Started
            | MemberTurnJoinStateV1::Completed { .. }
            | MemberTurnJoinStateV1::LaunchIndeterminate { .. } => {
                let subscription = registry
                    .subscribe(&reservation, 0)
                    .map_err(anyhow::Error::new)?;
                durable_stream_response(subscription).map(Some)
            }
        }
    }

    async fn submit_e2_turn(
        &self,
        req: MemberTurnSubmitRequestV1,
        active: Arc<ActiveMemberRuntime>,
        turn_policy_snapshot: Option<transport_api_types::PolicySnapshotV3>,
        uaa_session_id: String,
        ordinary_effect: Option<Arc<crate::service::OrdinaryWorldEffectContextV1>>,
    ) -> Result<Response> {
        let registry = self
            .member_turn_join
            .as_ref()
            .ok_or_else(|| anyhow::Error::new(MemberTurnJoinError::UnsupportedLegacyState))?;
        let reservation = match ordinary_effect.as_ref() {
            Some(effect) => {
                effect.revalidate_for_effect()?;
                registry.reserve_ordinary(&req, effect.turn_admission()?)
            }
            None => registry.reserve(&req),
        }
        .map_err(anyhow::Error::new)?;
        if let MemberTurnJoinStateV1::FailedBeforeLaunch {
            error_code,
            http_status,
        } = reservation.state()
        {
            return Err(anyhow::Error::new(
                MemberTurnJoinError::FailedBeforeLaunch {
                    error_code: error_code.clone(),
                    http_status: *http_status,
                },
            ));
        }
        let subscription = registry
            .subscribe(&reservation, 0)
            .map_err(anyhow::Error::new)?;
        if matches!(
            reservation.state(),
            MemberTurnJoinStateV1::Completed { .. }
                | MemberTurnJoinStateV1::LaunchIndeterminate { .. }
        ) {
            return durable_stream_response(subscription);
        }

        self.reserve_turn_slot(&active, reservation.span_id())?;
        let Some(mut leader) = registry
            .try_become_leader(&reservation)
            .map_err(anyhow::Error::new)?
        else {
            registry
                .await_launch_outcome(&reservation)
                .await
                .map_err(anyhow::Error::new)?;
            return durable_stream_response(subscription);
        };

        let turn_policy_launcher = match turn_policy_snapshot.as_ref() {
            Some(_) => match prepare_submitted_turn_policy_launcher(active.as_ref()) {
                Ok(launcher) => Some(launcher),
                Err(error) => {
                    let failure = persist_prelaunch_failure(
                        &mut leader,
                        "policy_launcher_preparation_failed_v1",
                        500,
                    );
                    self.clear_reserved_turn_slot(&active, reservation.span_id());
                    tracing::warn!(error = %error, "failed to prepare E2 turn policy launcher");
                    return Err(failure);
                }
            },
            None => None,
        };
        let prompt_fulfillment = match PromptFulfillmentBridge::for_member_backend(
            &active.backend_kind,
            turn_policy_launcher.as_ref().map_or_else(
                || active.binary_path.clone(),
                |launcher| launcher.path.clone(),
            ),
        ) {
            Ok(prompt_fulfillment) => prompt_fulfillment,
            Err(error) => {
                let failure = persist_prelaunch_failure(
                    &mut leader,
                    "provider_bridge_preparation_failed_v1",
                    500,
                );
                self.clear_reserved_turn_slot(&active, reservation.span_id());
                tracing::warn!(error = %error, "failed to prepare E2 provider bridge");
                return Err(failure);
            }
        };
        let mut run_request = match build_submitted_turn_run_request(
            active.as_ref(),
            req.prompt.clone(),
            &uaa_session_id,
            turn_policy_snapshot.as_ref(),
        ) {
            Ok(request) => request,
            Err(error) => {
                let failure = persist_prelaunch_failure(
                    &mut leader,
                    "turn_request_preparation_failed_v1",
                    400,
                );
                self.clear_reserved_turn_slot(&active, reservation.span_id());
                tracing::warn!(error = %error, "failed to prepare E2 submitted-turn request");
                return Err(failure);
            }
        };
        let ordinary_native_home = match ordinary_effect.as_ref() {
            Some(effect) => {
                match prepare_ordinary_native_runtime_env(
                    &mut run_request.env,
                    effect,
                    &active.workspace_dir,
                    &active.launcher_dir,
                    OrdinaryNativeHistoryRequestV1::Resume(Arc::clone(
                        active.ordinary_native_history.as_ref().ok_or_else(|| {
                            anyhow!("ordinary Resume omitted original worker history")
                        })?,
                    )),
                    #[cfg(test)]
                    self.ordinary_native_fixture.clone(),
                )
                .await
                {
                    Ok(home) => {
                        run_request.extensions.insert(
                            "agent_api.config.model.v1".into(),
                            json!(effect.requested_selection().routed_model_id),
                        );
                        Some(home)
                    }
                    Err(_error) => {
                        let failure = persist_prelaunch_failure(
                            &mut leader,
                            "ordinary_native_preparation_refused_v1",
                            400,
                        );
                        self.clear_reserved_turn_slot(&active, reservation.span_id());
                        return Err(failure);
                    }
                }
            }
            None => None,
        };
        if let Some(effect) = ordinary_effect.as_ref() {
            if effect.revalidate_for_effect().is_err()
                || ordinary_native_home
                    .as_ref()
                    .is_none_or(|home| home.revalidate().is_err())
            {
                let failure = persist_prelaunch_failure(
                    &mut leader,
                    "ordinary_native_effect_refused_v1",
                    400,
                );
                self.clear_reserved_turn_slot(&active, reservation.span_id());
                return Err(failure);
            }
        }
        let mut context = active.submit_context(req.run_id.clone(), req.acceptance_context.clone());
        if let Some(effect) = ordinary_effect.as_ref() {
            context.orchestrator_participant_id =
                authenticated_ordinary_turn_caller(active.as_ref(), effect)?;
        }
        context.validate_acceptance_for_mode(MemberStreamMode::SubmittedTurn)?;
        leader.mark_call_entered().map_err(anyhow::Error::new)?;

        let manager = self.clone();
        let span_id = reservation.span_id().to_string();
        let stream_id = reservation.stream_id().to_string();
        let binding = active.binding.clone();
        tokio::spawn(async move {
            let mut _turn_policy_launcher = turn_policy_launcher;
            if let Some(effect) = ordinary_effect.as_ref() {
                if effect.revalidate_for_effect().is_err()
                    || ordinary_native_home
                        .as_ref()
                        .is_none_or(|home| home.revalidate().is_err())
                {
                    let _ = leader
                        .mark_launch_indeterminate("ordinary_effect_changed_after_call_entered_v1");
                    manager.clear_reserved_turn_slot(&active, &span_id);
                    return;
                }
            }
            let PromptFulfillmentRunControl {
                handle,
                cancel,
                terminal,
            } = match run_ordinary_native_control(
                Arc::new(prompt_fulfillment),
                run_request,
                ordinary_native_home.clone(),
                ordinary_native_home.as_ref().map(|_| {
                    (
                        Arc::clone(&active.privileged_child_exclusion_lease),
                        _turn_policy_launcher.take(),
                    )
                }),
            )
            .await
            {
                Ok(control) => control,
                Err(_error) => {
                    let _ = leader
                        .mark_launch_indeterminate("run_control_failed_after_call_entered_v1");
                    manager.clear_reserved_turn_slot(&active, &span_id);
                    tracing::error!(
                        error_code = "run_control_failed_after_call_entered_v1",
                        "E2 member-turn launch became indeterminate"
                    );
                    return;
                }
            };
            let cleanup_cancel = cancel.clone();

            let mut producer = match RuntimeEventStreamProducer::with_stream_id(stream_id) {
                Ok(producer) => producer,
                Err(error) => {
                    let error = join_failed_control(
                        error,
                        handle,
                        cleanup_cancel.clone(),
                        terminal.clone(),
                    )
                    .await;
                    if terminal.wait().await.require_checked().is_err() {
                        manager.unregister_member(&active.participant_id);
                    }
                    let _ = leader.mark_launch_indeterminate("reserved_stream_identity_invalid_v1");
                    manager.clear_reserved_turn_slot(&active, &span_id);
                    tracing::error!(error = %error, "invalid reserved E2 member-turn stream identity");
                    return;
                }
            };
            let start = match producer.start(span_id.clone()) {
                Ok(start) => start,
                Err(error) => {
                    let error = join_failed_control(
                        error,
                        handle,
                        cleanup_cancel.clone(),
                        terminal.clone(),
                    )
                    .await;
                    if terminal.wait().await.require_checked().is_err() {
                        manager.unregister_member(&active.participant_id);
                    }
                    let _ = leader.mark_launch_indeterminate("start_construction_failed_v1");
                    manager.clear_reserved_turn_slot(&active, &span_id);
                    tracing::error!(error = %error, "failed to construct real E2 member-turn Start");
                    return;
                }
            };
            if let Err(error) = leader.publish_frame(&start) {
                let error = join_failed_control(
                    error.into(),
                    handle,
                    cleanup_cancel.clone(),
                    terminal.clone(),
                )
                .await;
                if terminal.wait().await.require_checked().is_err() {
                    manager.unregister_member(&active.participant_id);
                }
                let _ = leader.mark_launch_indeterminate("start_persistence_failed_v1");
                manager.clear_reserved_turn_slot(&active, &span_id);
                tracing::error!(error = %error, "failed to persist real E2 member-turn Start");
                return;
            }

            let turn = Arc::new(ActiveSubmittedTurn {
                terminal: terminal.clone(),
                ordinary_effect,
                ordinary_native_home,
                participant_id: active.participant_id.clone(),
                cancel,
                last_signal: Mutex::new(None),
            });
            manager.register_turn(span_id.clone(), turn.clone());
            let mut events = handle.events;
            let completion = handle.completion;
            let mut emitted_registered = false;
            let mut persistence_failed = false;

            while let Some(wrapper_event) = events.next().await {
                if let Some(session_id) =
                    surfaced_uaa_session_id_from_data(wrapper_event.data.as_ref())
                {
                    manager.remember_uaa_session_id(&turn.participant_id, session_id);
                }
                match frame_from_wrapper_event(
                    &context,
                    &binding,
                    &span_id,
                    wrapper_event,
                    &mut emitted_registered,
                    MemberStreamMode::SubmittedTurn,
                    active.agent_id.as_str(),
                    &mut producer,
                ) {
                    Ok(Some(frame)) => {
                        if let Err(error) = leader.publish_frame(&frame) {
                            tracing::error!(error = %error, "failed to persist E2 member-turn Event");
                            turn.terminal.request_route_disposal();
                            turn.cancel.cancel();
                            persistence_failed = true;
                            break;
                        }
                    }
                    Ok(None) => {}
                    Err(error) => {
                        tracing::error!(error = %error, "failed to identify E2 member-turn Event");
                        turn.terminal.request_route_disposal();
                        turn.cancel.cancel();
                        persistence_failed = true;
                        break;
                    }
                }
            }

            drop(events);
            terminal.request_route_disposal();
            let raw_completion = completion.await;
            let native_terminal = terminal.wait().await;
            let completion = native_terminal.enclosing_result(raw_completion);
            if let Ok(ref completion) = completion {
                if let Some(session_id) =
                    surfaced_uaa_session_id_from_data(completion.data.as_ref())
                {
                    manager.remember_uaa_session_id(&turn.participant_id, session_id);
                }
            }
            if matches!(
                &native_terminal.settlement,
                NativeSettlementOutcome::Checked(Err(_))
            ) {
                let _ = leader.mark_launch_indeterminate("native_settlement_unconfirmed_v1");
                manager.unregister_member(&active.participant_id);
                manager.unregister_turn(&span_id);
                return;
            }
            if !persistence_failed {
                match frames_from_completion(
                    &context,
                    &binding,
                    &span_id,
                    completion,
                    turn.last_signal(),
                    &mut emitted_registered,
                    MemberStreamMode::SubmittedTurn,
                    active.agent_id.as_str(),
                    &mut producer,
                ) {
                    Ok(frames) => {
                        for frame in frames {
                            if let Err(error) = leader.publish_frame(&frame) {
                                tracing::error!(error = %error, "failed to persist E2 member-turn completion");
                                persistence_failed = true;
                                break;
                            }
                        }
                    }
                    Err(error) => {
                        tracing::error!(error = %error, "failed to identify E2 member-turn completion");
                        persistence_failed = true;
                    }
                }
            }
            if persistence_failed {
                let _ = leader.mark_launch_indeterminate("frame_persistence_failed_v1");
            }
            manager.unregister_turn(&span_id);
        });

        registry
            .await_launch_outcome(&reservation)
            .await
            .map_err(anyhow::Error::new)?;
        durable_stream_response(subscription)
    }

    #[cfg(target_os = "linux")]
    pub(crate) fn control_ordinary_initial_bootstrap(
        &self,
        request: &transport_api_types::OrdinaryInitialBootstrapControlRequestV1,
        peer_uid: u32,
    ) -> Result<transport_api_types::OrdinaryInitialBootstrapControlDeliveryV1> {
        use transport_api_types::OrdinaryInitialBootstrapControlDeliveryV1 as Delivery;
        request.validate().map_err(anyhow::Error::msg)?;
        let registry = self
            .active_members
            .read()
            .map_err(|_| anyhow!("initial member registry poisoned"))?;
        let target = &request.activation.retained_participant_id;
        if let Some(pending) = registry.ordinary_pending_participants.get(target) {
            anyhow::ensure!(
                pending.span_id == request.service_span_id,
                "initial control actual pending span differs"
            );
            let mut delivered = false;
            pending
                .effect
                .with_initial_bootstrap_control(request, peer_uid, || {
                    delivered = pending.control.request_initial_control(&request.signal)?;
                    if delivered {
                        pending
                            .effect
                            .native_open
                            .store(false, std::sync::atomic::Ordering::Release);
                    }
                    Ok(true)
                })?;
            return Ok(if delivered {
                Delivery::Delivered
            } else {
                Delivery::Latched
            });
        }
        let Some(active) = registry.by_participant_id.get(target) else {
            let key = (
                request.source_start.authority_store_id.clone(),
                request.activation.orchestration_session_id.clone(),
                target.clone(),
                request.service_span_id.clone(),
            );
            let Some(cached) = registry.ordinary_initial_retirements.get(&key) else {
                return Ok(Delivery::UnknownOwner);
            };
            cached.reader.with_control(request, peer_uid, || {
                anyhow::ensure!(
                    cached.report.matches_control(request),
                    "initial retirement cached receiver differs"
                );
                Ok(true)
            })?;
            return Ok(Delivery::Retired {
                report: Box::new(cached.report.clone()),
            });
        };
        let effect = active
            .ordinary_effect
            .as_ref()
            .ok_or_else(|| anyhow!("initial control has no actual ordinary owner"))?;
        let mut delivery = Delivery::UnknownOwner;
        effect.with_initial_bootstrap_control(request, peer_uid, || {
            let held = active
                .bootstrap
                .lock()
                .map_err(|_| anyhow!("initial bootstrap state poisoned"))?;
            let Some(bootstrap) = held.as_ref() else {
                return Ok(false);
            };
            anyhow::ensure!(
                bootstrap.span_id == request.service_span_id,
                "initial control actual bootstrap span differs"
            );
            let control = bootstrap
                .initial_control
                .as_ref()
                .ok_or_else(|| anyhow!("initial control actual latch absent"))?;
            anyhow::ensure!(
                bootstrap.cancel.is_some() && bootstrap.terminal.is_some(),
                "initial control actual SDK owner absent"
            );
            effect
                .native_open
                .store(false, std::sync::atomic::Ordering::Release);
            delivery = if control.request_initial_control(&request.signal)? {
                Delivery::Delivered
            } else {
                Delivery::Latched
            };
            Ok(true)
        })?;
        Ok(delivery)
    }

    #[cfg(target_os = "linux")]
    pub(crate) fn cancel_ordinary_accepted_work(
        &self,
        request: &transport_api_types::OrdinaryAcceptedWorkCancelRequestV1,
        peer_uid: u32,
    ) -> Result<bool> {
        request.validate().map_err(anyhow::Error::msg)?;
        // The span is only a locator. Keep the actual owner registration read
        // locked through source admission and the synchronous SDK cancellation.
        let turns = self
            .active_turns_by_span_id
            .read()
            .map_err(|_| anyhow!("ordinary cancellation turn registry poisoned"))?;
        if let Some(turn) = turns.get(&request.runtime_submission_id) {
            let effect = turn
                .ordinary_effect
                .as_ref()
                .ok_or_else(|| anyhow!("ordinary cancellation owner has no original admission"))?;
            if let transport_api_types::OrdinaryAcceptedWorkCancelControlV1::StopWorker { stop } =
                &request.control
            {
                let members = self
                    .active_members
                    .read()
                    .map_err(|_| anyhow!("ordinary cancellation member registry poisoned"))?;
                let member = members
                    .by_participant_id
                    .get(&turn.participant_id)
                    .ok_or_else(|| {
                        anyhow!("ordinary Stop cancellation actual member unavailable")
                    })?;
                anyhow::ensure!(
                    member.uaa_session_id().as_deref() == Some(stop.native_session_id.as_str()),
                    "ordinary Stop cancellation changed actual native member"
                );
            }
            return effect.with_ordinary_accepted_cancellation(request, peer_uid, || {
                *turn
                    .last_signal
                    .lock()
                    .map_err(|_| anyhow!("ordinary cancellation signal owner poisoned"))? =
                    Some(request.signal.clone());
                turn.terminal.request_route_disposal();
                turn.cancel.cancel();
                Ok(true)
            });
        }
        drop(turns);
        // Ephemeral execution owns its SDK handle in the original bootstrap,
        // rather than the retained submitted-turn map. Hold its actual member
        // registration, but acquire the bootstrap mutex only inside admission.
        let members = self
            .active_members
            .read()
            .map_err(|_| anyhow!("ordinary cancellation member registry poisoned"))?;
        for member in members.by_participant_id.values() {
            let matches_span = member
                .bootstrap
                .lock()
                .map_err(|_| anyhow!("ordinary cancellation bootstrap owner poisoned"))?
                .as_ref()
                .is_some_and(|bootstrap| bootstrap.span_id == request.runtime_submission_id);
            if !matches_span {
                continue;
            }
            let effect = member.ordinary_effect.as_ref().ok_or_else(|| {
                anyhow!("ordinary cancellation bootstrap has no original admission")
            })?;
            return effect.with_ordinary_accepted_cancellation(request, peer_uid, || {
                let mut held = member
                    .bootstrap
                    .lock()
                    .map_err(|_| anyhow!("ordinary cancellation bootstrap owner poisoned"))?;
                let bootstrap = held
                    .as_mut()
                    .filter(|bootstrap| bootstrap.span_id == request.runtime_submission_id)
                    .ok_or_else(|| {
                        anyhow!("ordinary cancellation original bootstrap owner closed")
                    })?;
                let terminal = bootstrap
                    .terminal
                    .as_ref()
                    .ok_or_else(|| anyhow!("ordinary cancellation actual terminal owner absent"))?;
                let cancel = bootstrap
                    .cancel
                    .as_ref()
                    .ok_or_else(|| anyhow!("ordinary cancellation actual SDK handle absent"))?;
                bootstrap.last_signal = Some(request.signal.clone());
                terminal.request_route_disposal();
                cancel.cancel();
                Ok(true)
            });
        }
        anyhow::bail!("ordinary cancellation actual native owner unavailable")
    }

    // Native fixture cleanup remains an internal owner action. Public generic
    // HTTP cancellation cannot turn possession of an ordinary span into control.
    #[cfg(test)]
    pub(crate) fn cancel(&self, span_id: &str, sig: &str) -> Result<bool> {
        self.cancel_with_control(span_id, sig, true)
    }

    pub(crate) fn cancel_for_legacy_http(&self, span_id: &str, sig: &str) -> Result<bool> {
        self.cancel_with_control(span_id, sig, false)
    }

    fn cancel_with_control(
        &self,
        span_id: &str,
        sig: &str,
        internal_cleanup: bool,
    ) -> Result<bool> {
        validate_cancel_signal(sig)?;
        let normalized_signal = sig.trim().to_ascii_uppercase();

        let submitted_turn = self
            .active_turns_by_span_id
            .read()
            .expect("member runtime registry lock poisoned")
            .get(span_id)
            .cloned();
        if let Some(submitted_turn) = submitted_turn {
            anyhow::ensure!(
                internal_cleanup || submitted_turn.ordinary_effect.is_none(),
                "ordinary accepted cancellation requires independent source and delivery admission"
            );
            if let Ok(mut guard) = submitted_turn.last_signal.lock() {
                *guard = Some(normalized_signal.clone());
            }
            submitted_turn.terminal.request_route_disposal();
            submitted_turn.cancel.cancel();
            return Ok(true);
        }

        let active_members = self
            .active_members
            .read()
            .expect("member runtime registry lock poisoned")
            .by_participant_id
            .values()
            .cloned()
            .collect::<Vec<_>>();
        for active in active_members {
            if !internal_cleanup && active.ordinary_effect.is_some() {
                let bootstrap = active
                    .bootstrap
                    .lock()
                    .map_err(|_| anyhow!("ordinary cancellation bootstrap owner poisoned"))?;
                anyhow::ensure!(bootstrap.as_ref().is_none_or(|bootstrap| bootstrap.span_id != span_id),
                    "ordinary initial cancellation requires independent source and actual owner admission");
            }
            if active.cancel_bootstrap_if_span_matches(span_id, normalized_signal.as_str()) {
                return Ok(true);
            }
        }
        Ok(false)
    }

    fn find_ordinary_fork_source(
        &self,
        dispatch: &MemberDispatchRequestV1,
        effect: &Arc<crate::service::OrdinaryWorldEffectContextV1>,
        workspace: &Path,
        binding: &SharedWorldBindingSnapshot,
        actual_binary: &Path,
    ) -> Result<Arc<PinnedOrdinaryForkSourceV1>> {
        let activation = dispatch
            .e2_launch_activation
            .as_ref()
            .filter(|activation| activation.launch_kind == E2MemberLaunchKindV1::Fork)
            .ok_or_else(|| anyhow!("ordinary Fork omitted launch activation"))?;
        let source_id = activation
            .source_participant_id
            .as_ref()
            .ok_or_else(|| anyhow!("ordinary Fork omitted source participant"))?;
        let source = self
            .active_members
            .read()
            .map_err(|_| anyhow!("ordinary Fork source registry lock poisoned"))?
            .by_participant_id
            .get(source_id)
            .cloned()
            .ok_or_else(|| anyhow!("ordinary Fork exact source runtime is not held"))?;
        if source.backend_kind != MemberRuntimeBackendKindV1::Codex
            || dispatch.resolved_runtime.backend_kind != MemberRuntimeBackendKindV1::Codex
            || source.orchestration_session_id != dispatch.orchestration_session_id
            || source.backend_id != dispatch.backend_id
            || source.protocol != json!(dispatch.protocol)
            || source.binary_path != actual_binary
            || source.workspace_dir != workspace
            || source.binding != *binding
            || source.has_active_bootstrap()
            || source
                .active_turn_span_id
                .lock()
                .map_err(|_| anyhow!("ordinary Fork source turn lock poisoned"))?
                .is_some()
        {
            anyhow::bail!(
                "ordinary Fork source differs from exact idle Codex runtime/workspace/World"
            );
        }
        let pinned = Arc::new(PinnedOrdinaryForkSourceV1 {
            history: Arc::clone(
                source
                    .ordinary_native_history
                    .as_ref()
                    .ok_or_else(|| anyhow!("ordinary Fork source has no held native history"))?,
            ),
            session_id: source.uaa_session_id().ok_or_else(|| {
                anyhow!("ordinary Fork source has no surfaced UAA session identity")
            })?,
            active: source,
            effect: Arc::clone(effect),
            ownership: OrdinaryForkSourceOwnershipV1 {
                registry: Arc::downgrade(&self.active_members),
                child_session: Mutex::new(None),
            },
        });
        pinned.revalidate()?;
        Ok(pinned)
    }

    fn reserve_ordinary_launch(
        &self,
        effect: &Arc<crate::service::OrdinaryWorldEffectContextV1>,
        span_id: &str,
    ) -> Result<OrdinaryMemberLaunchReservationV1> {
        effect.revalidate_for_effect()?;
        let transport_api_types::OrdinaryGatewayRuntimeIdentityV1::WorldWorker {
            participant_id,
            ..
        } = effect.authenticated_runtime()
        else {
            anyhow::bail!("ordinary launch requires authenticated member");
        };
        let mut registry = self
            .active_members
            .write()
            .map_err(|_| anyhow!("ordinary member registry lock poisoned"))?;
        if registry.by_participant_id.contains_key(&participant_id)
            || registry
                .ordinary_pending_participants
                .contains_key(&participant_id)
        {
            anyhow::bail!("ordinary member already active or reserved");
        }
        let pending = Arc::new(PendingOrdinaryBootstrapV1 {
            effect: Arc::clone(effect),
            span_id: span_id.into(),
            control: Arc::new(OrdinaryInitialBootstrapCancellationV1::default()),
        });
        registry
            .ordinary_pending_participants
            .insert(participant_id.clone(), Arc::clone(&pending));
        Ok(OrdinaryMemberLaunchReservationV1 {
            registry: Arc::clone(&self.active_members),
            participant_id,
            pending,
        })
    }

    pub(crate) fn ordinary_gateway_effect(
        &self,
        runtime: &transport_api_types::OrdinaryGatewayRuntimeIdentityV1,
    ) -> Result<(
        Arc<crate::service::OrdinaryWorldEffectContextV1>,
        Arc<crate::gateway_runtime::OrdinaryGatewayRuntimeLeaseV1>,
    )> {
        // A copied ActiveMember binding or prior bootstrap run is never enough.
        // Only a currently registered native bootstrap/turn keeps its own route.
        let turns = self
            .active_turns_by_span_id
            .read()
            .map_err(|_| anyhow!("ordinary turn registry lock poisoned"))?;
        for turn in turns.values() {
            if let (Some(effect), Some(home)) = (&turn.ordinary_effect, &turn.ordinary_native_home)
            {
                if &effect.authenticated_runtime() == runtime {
                    effect.revalidate_for_effect()?;
                    home.revalidate()?;
                    return Ok((Arc::clone(effect), Arc::clone(&home.gateway)));
                }
            }
        }
        drop(turns);
        let members = self
            .active_members
            .read()
            .map_err(|_| anyhow!("ordinary member registry lock poisoned"))?;
        for member in members.by_participant_id.values() {
            if member
                .bootstrap
                .lock()
                .map_err(|_| anyhow!("ordinary bootstrap lock poisoned"))?
                .is_none()
            {
                continue;
            }
            if let Some(effect) = &member.ordinary_effect {
                if &effect.authenticated_runtime() == runtime {
                    let home = member
                        .ordinary_native_home
                        .lock()
                        .map_err(|_| anyhow!("ordinary native home lock poisoned"))?;
                    let home = home
                        .as_ref()
                        .ok_or_else(|| anyhow!("ordinary native home is unavailable"))?;
                    effect.revalidate_for_effect()?;
                    home.revalidate()?;
                    return Ok((Arc::clone(effect), Arc::clone(&home.gateway)));
                }
            }
        }
        anyhow::bail!("selected gateway has no exact active native runtime")
    }

    fn register_member(&self, active: Arc<ActiveMemberRuntime>) -> Result<()> {
        let retained_key = RetainedMemberKey::from_active(active.as_ref());
        let mut guard = self
            .active_members
            .write()
            .expect("member runtime registry lock poisoned");
        if guard.by_participant_id.contains_key(&active.participant_id) {
            return Err(crate::service::BadRequestError::new(format!(
                "member_dispatch.participant_id {} is already retained",
                active.participant_id
            ))
            .into());
        }
        match guard.by_retained_key.get(&retained_key).cloned() {
            Some(slot) => {
                let slot = prune_stale_fork_child_slot(&mut guard, &retained_key, slot);
                let updated_slot = slot.with_registered_fork_child(
                    &retained_key,
                    &guard.by_participant_id,
                    active.as_ref(),
                )?;
                guard
                    .by_retained_key
                    .insert(retained_key.clone(), updated_slot);
            }
            None => {
                guard.by_retained_key.insert(
                    retained_key.clone(),
                    RetainedMemberSlot::new(active.participant_id.clone()),
                );
            }
        }
        guard
            .by_participant_id
            .insert(active.participant_id.clone(), active);
        Ok(())
    }

    /// Explicit ordinary parked-member retirement. Source permission and the
    /// accepted-work cut come from the original runtime owner; this manager
    /// separately checks its actual registered native participant and cleanup.
    #[cfg(target_os = "linux")]
    pub(crate) fn retire_ordinary_member(
        &self,
        request: &transport_api_types::OrdinaryRetainedWorkerStopRequestV1,
        peer_uid: u32,
    ) -> Result<transport_api_types::OrdinaryRetainedWorkerStopResponseV1> {
        use transport_api_types::{
            OrdinaryRetainedWorkerStopAcknowledgmentV1 as Ack,
            OrdinaryRetainedWorkerStopBusyReasonV1 as Busy,
            OrdinaryRetainedWorkerStopResponseV1 as Response,
        };
        request.validate().map_err(anyhow::Error::msg)?;
        let key = (
            request.authority_store_id.clone(),
            request.orchestration_session_id.clone(),
            request.target_participant_id.clone(),
        );
        // Match the existing ordinary gateway reader's registry -> Root order.
        // In particular, never wait for this write lock while holding Root.
        let mut registry = self
            .active_members
            .write()
            .map_err(|_| anyhow!("member registry poisoned"))?;
        let active = registry
            .by_participant_id
            .get(&request.target_participant_id)
            .cloned();
        let cached = registry.ordinary_retirements.get(&key).cloned();
        let reader = match (&active, &cached) {
            (_, Some(cached)) => Arc::clone(&cached.reader),
            (Some(active), None) => Arc::new(
                active
                    .ordinary_effect
                    .as_ref()
                    .ok_or_else(|| anyhow!("ordinary Stop requires an actual ordinary member"))?
                    .retained_stop_control_reader()?,
            ),
            (None, None) => return Ok(Response::UnknownOwner {}),
        };
        reader.with_locked_retirement(request, peer_uid, |admission| {
            if let Some(cached) = registry.ordinary_retirements.get(&key) {
                cached
                    .acknowledgment
                    .validate(request)
                    .map_err(anyhow::Error::msg)?;
                if admission
                    .existing_acknowledgment()
                    .is_some_and(|ack| ack != &cached.acknowledgment)
                {
                    anyhow::bail!(
                        "ordinary Stop historical acknowledgment differs from actual member owner"
                    );
                }
                admission.revalidate_before_retirement()?;
                return Ok(Response::Retired {
                    acknowledgment: Box::new(cached.acknowledgment.clone()),
                });
            }
            // A historical Closed record is never a new retirement grant.
            if admission.existing_acknowledgment().is_some() {
                return Ok(Response::UnknownOwner {});
            }
            let Some(active) = active.as_ref().filter(|active| {
                registry
                    .by_participant_id
                    .get(&request.target_participant_id)
                    .is_some_and(|actual| Arc::ptr_eq(actual, active))
            }) else {
                return Ok(Response::UnknownOwner {});
            };
            let effect = active
                .ordinary_effect
                .as_ref()
                .ok_or_else(|| anyhow!("ordinary Stop actual member admission absent"))?;
            let activation = active
                .e2_launch_activation
                .as_ref()
                .ok_or_else(|| anyhow!("ordinary Stop actual E2 activation absent"))?;
            let cap = active
                .authenticated_worker_cap_identity
                .lock()
                .map_err(|_| anyhow!("ordinary Stop cap state poisoned"))?;
            anyhow::ensure!(
                active.orchestration_session_id == request.orchestration_session_id
                    && active.backend_id == request.target_backend_id
                    && active.participant_id == request.target_participant_id
                    && active.binding.orchestration_session_id == request.orchestration_session_id
                    && active.binding.world_id == request.world_id
                    && active.binding.world_generation == request.world_generation
                    && activation.bootstrap_run_id == request.bootstrap_run_id
                    && activation.immutable_worker_cap_ref == request.worker_cap_ref
                    && active.uaa_session_id().as_deref()
                        == Some(request.native_session_id.as_str())
                    && cap.as_ref()
                        == Some(&RetainedTurnWorkerCapIdentity {
                            commitment_ref: request.worker_cap_ref.clone(),
                            created_revision: activation.immutable_worker_cap_created_revision,
                            application_revision: activation
                                .immutable_worker_cap_application_revision
                        }),
                "ordinary Stop actual native member/cap differs from admitted target"
            );
            let bootstrap = active
                .bootstrap
                .lock()
                .map_err(|_| anyhow!("ordinary Stop bootstrap state poisoned"))?;
            if bootstrap.is_some() {
                return Ok(Response::NotQuiescent {
                    reason: Busy::Bootstrap,
                });
            }
            let turn = active
                .active_turn_span_id
                .lock()
                .map_err(|_| anyhow!("ordinary Stop turn state poisoned"))?;
            if turn.is_some() {
                return Ok(Response::NotQuiescent {
                    reason: Busy::AcceptedTurn,
                });
            }
            let home = active
                .ordinary_native_home
                .lock()
                .map_err(|_| anyhow!("ordinary Stop native home state poisoned"))?;
            if home.is_some() {
                return Ok(Response::NotQuiescent {
                    reason: Busy::NativeHistoryUse,
                });
            }
            let history = active
                .ordinary_native_history
                .as_ref()
                .ok_or_else(|| anyhow!("ordinary Stop native history absent"))?;
            history.require_original_worker(
                effect.ordinary_start_selection(),
                &effect.authenticated_runtime(),
                effect.requested_selection(),
            )?;
            history.in_use.require_usable()?;
            let history_use = match OrdinaryNativeHistoryUseV1::acquire(Arc::clone(&history.in_use))
            {
                Ok(lease) => lease,
                Err(_) => {
                    return Ok(Response::NotQuiescent {
                        reason: Busy::NativeHistoryUse,
                    })
                }
            };
            let handoff = history
                .handoff_terminal
                .latest
                .lock()
                .map_err(|_| anyhow!("ordinary Stop listener terminal state poisoned"))?;
            if !handoff
                .as_ref()
                .is_some_and(|(_, outcome)| outcome.is_some())
            {
                return Ok(Response::NotQuiescent {
                    reason: Busy::NativeHistoryUse,
                });
            }
            let retained_key = RetainedMemberKey::from_active(active);
            let slot = registry
                .by_retained_key
                .get(&retained_key)
                .filter(|slot| slot.contains(&active.participant_id))
                .ok_or_else(|| anyhow!("ordinary Stop actual retained slot owner differs"))?;
            let next_slot = slot.clone().without_participant(&active.participant_id);
            effect.revalidate_original_stop_world_placement(request)?;
            let acknowledgment = Ack {
                schema_version: 1,
                request: request.clone(),
                retirement_id: format!("ormr_{}", uuid::Uuid::now_v7()),
            };
            acknowledgment
                .validate(request)
                .map_err(anyhow::Error::msg)?;
            admission.revalidate_before_retirement()?;
            // This write lock linearizes removal against both turn reservation
            // and an initial Fork's exact source-membership check. Children and
            // the original physical history remain independent retained data.
            registry.by_participant_id.remove(&active.participant_id);
            match next_slot {
                Some(slot) => {
                    registry.by_retained_key.insert(retained_key, slot);
                }
                None => {
                    registry.by_retained_key.remove(&retained_key);
                }
            }
            registry.ordinary_retirements.insert(
                key.clone(),
                CachedOrdinaryRetirementV1 {
                    reader: Arc::clone(&reader),
                    acknowledgment: acknowledgment.clone(),
                },
            );
            drop(handoff);
            drop(history_use);
            drop(home);
            drop(turn);
            drop(bootstrap);
            drop(cap);
            Ok(Response::Retired {
                acknowledgment: Box::new(acknowledgment),
            })
        })
        // The local active Arc drops before the returned response is observable;
        // the retry cache holds only identity/source custody and checked data.
    }

    fn unregister_member(&self, participant_id: &str) {
        if let Ok(mut guard) = self.active_members.write() {
            if let Some(active) = guard.by_participant_id.remove(participant_id) {
                let retained_key = RetainedMemberKey::from_active(active.as_ref());
                if let Some(slot) = guard.by_retained_key.get(&retained_key).cloned() {
                    match slot.without_participant(participant_id) {
                        Some(updated_slot) => {
                            guard.by_retained_key.insert(retained_key, updated_slot);
                        }
                        None => {
                            guard.by_retained_key.remove(&retained_key);
                        }
                    }
                }
                active.close_bootstrap();
                active.cleanup_launcher_dir();
            }
        }
    }

    /// Only the actual registered initial owner can produce this report. SDK
    /// waits, home callback/listener join and gateway disposal remain separate
    /// predicates; a raw Exit or cancellation request supplies none of them.
    async fn prepare_failed_ordinary_initial_bootstrap_frames(
        &self,
        active: &Arc<ActiveMemberRuntime>,
        context: &MemberStreamContext,
        binding: &SharedWorldBindingSnapshot,
        span_id: &str,
        logical_exit: i32,
        producer: &mut RuntimeEventStreamProducer,
    ) -> Result<PreparedOrdinaryInitialRetirementV1> {
        use std::os::unix::process::ExitStatusExt;
        use transport_api_types::{
            NativeProcessTerminationV1 as Status, OrdinaryGatewayDisposalV1 as Gateway,
            OrdinaryInitialBootstrapRetirementV1 as Report,
        };
        let effect = active
            .ordinary_effect
            .as_ref()
            .ok_or_else(|| anyhow!("initial retirement requires actual ordinary ownership"))?;
        let activation = active
            .e2_launch_activation
            .as_ref()
            .ok_or_else(|| anyhow!("initial retirement actual E2 absent"))?;
        anyhow::ensure!(
            context.acceptance_context.is_none()
                && context.orchestration_session_id == active.orchestration_session_id
                && context.participant_id == active.participant_id
                && context.backend_id == active.backend_id
                && context.orchestrator_participant_id == active.orchestrator_participant_id
                && context.run_id == activation.bootstrap_run_id
                && binding.orchestration_session_id == active.binding.orchestration_session_id
                && binding.world_id == active.binding.world_id
                && binding.world_generation == active.binding.world_generation,
            "initial retirement actual member context differs"
        );
        let terminal = {
            let bootstrap = active
                .bootstrap
                .lock()
                .map_err(|_| anyhow!("initial retirement bootstrap poisoned"))?;
            let bootstrap = bootstrap
                .as_ref()
                .ok_or_else(|| anyhow!("initial retirement bootstrap owner absent"))?;
            anyhow::ensure!(
                bootstrap.span_id == span_id
                    && bootstrap.initial_control.is_some()
                    && bootstrap.cancel.is_some(),
                "initial retirement actual receiver/SDK owner differs"
            );
            bootstrap
                .terminal
                .clone()
                .ok_or_else(|| anyhow!("initial retirement SDK terminal owner absent"))?
        };
        let observed = terminal.wait().await;
        let native = observed
            .initial_bootstrap_process_closeout()
            .map_err(|error| anyhow!(error))?;
        let home = active
            .ordinary_native_home
            .lock()
            .map_err(|_| anyhow!("initial retirement home poisoned"))?
            .as_ref()
            .cloned()
            .ok_or_else(|| anyhow!("initial retirement actual home absent"))?;
        let history = Arc::clone(&home.history);
        let history_use_closed = home._history_use.closed_observation();
        history.require_original_worker(
            effect.ordinary_start_selection(),
            &effect.authenticated_runtime(),
            effect.requested_selection(),
        )?;
        {
            let joined = history
                .handoff_terminal
                .latest
                .lock()
                .map_err(|_| anyhow!("initial retirement handoff poisoned"))?;
            anyhow::ensure!(
                joined
                    .as_ref()
                    .is_some_and(|(run, outcome)| run == &activation.bootstrap_run_id
                        && matches!(outcome, Some("joined_used" | "joined_stopped"))),
                "initial retirement actual handoff join unconfirmed"
            );
        }
        let (pid, status) = home.gateway.finish_owned().await?;
        let gateway_runtime_id = home.gateway.runtime_id.clone();
        let termination = match (status.code(), status.signal()) {
            (Some(exit_code), _) => Status::Exited { exit_code },
            (_, Some(signal)) => Status::Signaled { signal },
            _ => anyhow::bail!("initial retirement gateway terminal status unavailable"),
        };
        anyhow::ensure!(
            !effect
                .native_open
                .load(std::sync::atomic::Ordering::Acquire),
            "initial retirement native admission remains open"
        );
        effect.revalidate_sealed_bootstrap_world_placement(
            &effect.authenticated_runtime(),
            binding,
        )?;
        let reader = effect.initial_bootstrap_closeout_reader(span_id)?;
        {
            let mut registry = self
                .active_members
                .write()
                .map_err(|_| anyhow!("initial retirement registry poisoned"))?;
            anyhow::ensure!(
                registry
                    .by_participant_id
                    .get(&active.participant_id)
                    .is_some_and(|actual| Arc::ptr_eq(actual, active))
                    && !registry
                        .ordinary_pending_participants
                        .contains_key(&active.participant_id),
                "initial retirement actual registry owner changed"
            );
            let bootstrap = active
                .bootstrap
                .lock()
                .map_err(|_| anyhow!("initial retirement bootstrap poisoned"))?;
            anyhow::ensure!(
                bootstrap
                    .as_ref()
                    .is_some_and(|bootstrap| bootstrap.span_id == span_id),
                "initial retirement actual bootstrap changed"
            );
            anyhow::ensure!(
                active
                    .active_turn_span_id
                    .lock()
                    .map_err(|_| anyhow!("initial retirement turn state poisoned"))?
                    .is_none(),
                "initial retirement has accepted turn ownership"
            );
            anyhow::ensure!(
                active
                    .ordinary_native_home
                    .lock()
                    .map_err(|_| anyhow!("initial retirement home poisoned"))?
                    .as_ref()
                    .is_some_and(|actual| Arc::ptr_eq(actual, &home)),
                "initial retirement actual home changed"
            );
            let key = RetainedMemberKey::from_active(active);
            let slot = registry
                .by_retained_key
                .get(&key)
                .filter(|slot| slot.contains(&active.participant_id))
                .ok_or_else(|| anyhow!("initial retirement retained slot differs"))?;
            let next = slot.clone().without_participant(&active.participant_id);
            registry.by_participant_id.remove(&active.participant_id);
            match next {
                Some(next) => {
                    registry.by_retained_key.insert(key, next);
                }
                None => {
                    registry.by_retained_key.remove(&key);
                }
            }
        }
        active.close_bootstrap();
        drop(home);
        // Check this exact home's actual release. A parent or descendant may
        // independently acquire the same store afterward; that is not this use.
        anyhow::ensure!(
            history_use_closed.is_closed(),
            "initial retirement actual history use remains owned"
        );
        let mut event = AgentEvent::message(
            active.agent_id.as_str(),
            context.orchestration_session_id.clone(),
            context.run_id.clone(),
            MessageEventKind::Status,
            "initial bootstrap retired",
        );
        stamp_event_identity(
            &mut event,
            active.agent_id.as_str(),
            context,
            binding,
            span_id,
            None,
        );
        let mut announcement = producer.event(event)?;
        let ExecuteStreamFrame::Event {
            frame_identity,
            event,
        } = &mut announcement
        else {
            unreachable!()
        };
        let report = reader.build_report(
            format!("obrt_{}", uuid::Uuid::now_v7()),
            native,
            Gateway::ProcessWaited {
                runtime_id: gateway_runtime_id,
                pid,
                termination,
            },
            frame_identity.clone(),
            event
                .event_identity
                .clone()
                .ok_or_else(|| anyhow!("initial retirement event identity absent"))?,
        )?;
        event.data = json!({(Report::EVENT_DATA_KEY): report.clone()});
        Report::from_service_event(frame_identity, event).map_err(anyhow::Error::msg)?;
        let exit = producer.exit(
            logical_exit,
            span_id.into(),
            Vec::new(),
            None,
            ProcessTelemetry::default(),
        )?;
        Ok(PreparedOrdinaryInitialRetirementV1 {
            frames: vec![announcement, exit],
            receipt: CachedOrdinaryInitialRetirementV1 {
                reader: Arc::new(reader),
                report,
            },
        })
    }

    fn finish_bootstrap(&self, participant_id: &str, preserve_retained_member: bool) {
        let active = self
            .active_members
            .read()
            .ok()
            .and_then(|guard| guard.by_participant_id.get(participant_id).cloned());
        let Some(active) = active else {
            return;
        };

        if preserve_retained_member {
            active.close_bootstrap();
        } else {
            self.unregister_member(participant_id);
        }
    }

    fn register_turn(&self, span_id: String, turn: Arc<ActiveSubmittedTurn>) {
        self.active_turns_by_span_id
            .write()
            .expect("member runtime registry lock poisoned")
            .insert(span_id, turn);
    }

    fn unregister_turn(&self, span_id: &str) {
        if let Ok(mut guard) = self.active_turns_by_span_id.write() {
            if let Some(turn) = guard.remove(span_id) {
                if let Some(active) =
                    self.active_members.read().ok().and_then(|active| {
                        active.by_participant_id.get(&turn.participant_id).cloned()
                    })
                {
                    self.clear_reserved_turn_slot(&active, span_id);
                }
            }
        }
    }

    fn reserve_turn_slot(&self, active: &Arc<ActiveMemberRuntime>, span_id: &str) -> Result<()> {
        // Retirement owns the write side. Keep this read lock through slot
        // reservation so a stale cloned member cannot launch after removal.
        let registry = self
            .active_members
            .read()
            .map_err(|_| anyhow!("member registry poisoned"))?;
        if !registry
            .by_participant_id
            .get(&active.participant_id)
            .is_some_and(|actual| Arc::ptr_eq(actual, active))
        {
            return Err(crate::service::BadRequestError::new(
                "retained member is no longer registered",
            )
            .into());
        }
        let mut guard = active
            .active_turn_span_id
            .lock()
            .map_err(|_| anyhow!("member runtime turn slot lock poisoned"))?;
        if let Some(existing) = guard.as_ref() {
            if existing == span_id {
                return Ok(());
            }
            return Err(crate::service::BadRequestError::new(format!(
                "member_turn_submit.participant_id {} already has an active submitted turn ({existing})",
                active.participant_id
            ))
            .into());
        }
        *guard = Some(span_id.to_string());
        Ok(())
    }

    fn clear_reserved_turn_slot(&self, active: &Arc<ActiveMemberRuntime>, span_id: &str) {
        if let Ok(mut guard) = active.active_turn_span_id.lock() {
            if guard.as_deref() == Some(span_id) {
                *guard = None;
            }
        }
    }

    fn remember_uaa_session_id(&self, participant_id: &str, session_id: String) {
        if let Some(active) = self
            .active_members
            .read()
            .ok()
            .and_then(|guard| guard.by_participant_id.get(participant_id).cloned())
        {
            active.remember_uaa_session_id(session_id);
        }
    }

    fn find_submit_target(
        &self,
        req: &MemberTurnSubmitRequestV1,
    ) -> Result<Arc<ActiveMemberRuntime>> {
        let retained_key = RetainedMemberKey::from_submit(req);
        let guard = self
            .active_members
            .read()
            .expect("member runtime registry lock poisoned");
        if let Some(active) = guard.by_participant_id.get(&req.participant_id).cloned() {
            return Ok(active);
        }

        if let Some(existing_slot) = guard.by_retained_key.get(&retained_key) {
            let existing_participant_ids = existing_slot.participant_ids();
            return Err(retained_slot_owner_mismatch_error(
                &retained_key,
                &existing_participant_ids,
                &req.participant_id,
            )
            .into());
        }

        Err(crate::service::BadRequestError::new(format!(
            "member_turn_submit.participant_id {} is not retained",
            req.participant_id
        ))
        .into())
    }

    fn validate_submit_target_slot(&self, req: &MemberTurnSubmitRequestV1) -> Result<()> {
        let retained_key = RetainedMemberKey::from_submit(req);
        let guard = self
            .active_members
            .read()
            .expect("member runtime registry lock poisoned");
        match guard.by_retained_key.get(&retained_key) {
            Some(slot) if slot.contains(&req.participant_id) => Ok(()),
            Some(slot) => {
                let participant_ids = slot.participant_ids();
                Err(retained_slot_owner_mismatch_error(
                    &retained_key,
                    &participant_ids,
                    &req.participant_id,
                )
                .into())
            }
            None => Err(missing_retained_slot_error(&retained_key).into()),
        }
    }
}

fn validate_retained_worker_launch_authority_proof(
    dispatch: &MemberDispatchRequestV1,
    binding: &SharedWorldBindingSnapshot,
) -> Result<()> {
    let requires_fresh_spawn_proof = dispatch
        .e2_launch_activation
        .as_ref()
        .is_some_and(|activation| activation.launch_kind == E2MemberLaunchKindV1::FreshSpawn);
    let Some(proof) = dispatch.retained_worker_launch_authority.as_ref() else {
        if requires_fresh_spawn_proof {
            return Err(crate::service::BadRequestError::new(
                "E2 Fresh Spawn activation requires retained_worker_launch_authority".to_string(),
            )
            .into());
        }
        return Ok(());
    };
    proof.validate().map_err(|error| {
        crate::service::BadRequestError::new(format!(
            "invalid retained_worker_launch_authority: {error}"
        ))
    })?;
    if proof.orchestration_session_id != dispatch.orchestration_session_id
        || proof.orchestration_session_id != binding.orchestration_session_id
        || proof.retained_participant_id != dispatch.participant_id
        || proof.caller_participant_id != dispatch.orchestrator_participant_id
        || proof.backend_id != dispatch.backend_id
        || proof.protocol != dispatch.protocol
        || proof.bootstrap_run_id != dispatch.run_id
        || proof.world_binding.world_id != dispatch.world_id
        || proof.world_binding.world_id != binding.world_id
        || proof.world_binding.world_generation != dispatch.world_generation
        || proof.world_binding.world_generation != binding.world_generation
        || dispatch.parent_participant_id.is_some()
        || dispatch.resumed_from_participant_id.is_some()
    {
        return Err(crate::service::BadRequestError::new(
            "retained_worker_launch_authority conflicts with member dispatch or exact world binding"
                .to_string(),
        )
        .into());
    }
    Ok(())
}

fn validate_e2_member_launch_activation(
    dispatch: &MemberDispatchRequestV1,
    binding: &SharedWorldBindingSnapshot,
    enforced_policy_snapshot: &transport_api_types::PolicySnapshotV3,
    acceptance_context: Option<&transport_api_types::WorldWorkAcceptanceContextV1>,
) -> Result<Option<RetainedTurnWorkerCapIdentity>> {
    let Some(activation) = dispatch.e2_launch_activation.as_ref() else {
        if dispatch.retained_worker_launch_authority.is_some() {
            return Err(crate::service::BadRequestError::new(
                "unsupported_legacy_state: authority-managed retained member omitted E2 launch activation"
                    .to_string(),
            )
            .into());
        }
        return Ok(None);
    };
    activation.validate().map_err(|error| {
        crate::service::BadRequestError::new(format!("invalid e2_launch_activation: {error}"))
    })?;
    let activation_snapshot = activation.policy_snapshot().map_err(|error| {
        crate::service::BadRequestError::new(format!("invalid e2_launch_activation: {error}"))
    })?;
    let activation_snapshot_bytes = serde_json::to_vec(&activation_snapshot).map_err(|error| {
        crate::service::BadRequestError::new(format!(
            "failed to serialize E2 launch activation snapshot: {error}"
        ))
    })?;
    let enforced_snapshot_bytes =
        serde_json::to_vec(enforced_policy_snapshot).map_err(|error| {
            crate::service::BadRequestError::new(format!(
                "failed to serialize enforced member launch policy snapshot: {error}"
            ))
        })?;
    let enforced_snapshot_hash = format!("{:x}", Sha256::digest(&enforced_snapshot_bytes));
    if activation_snapshot_bytes != enforced_snapshot_bytes
        || activation.policy_snapshot_hash != enforced_snapshot_hash
    {
        return Err(crate::service::BadRequestError::new(
            "E2 launch activation snapshot conflicts with enforced policy snapshot".to_string(),
        )
        .into());
    }
    let exact_activation_id = format!(
        "e2a_{}",
        activation
            .commitment_ref
            .exact_linkage_hash
            .get(..32)
            .ok_or_else(|| {
                crate::service::BadRequestError::new(
                    "invalid E2 launch commitment linkage hash".to_string(),
                )
            })?
    );
    let expected_lineage = match activation.launch_kind {
        E2MemberLaunchKindV1::FreshSpawn => {
            dispatch.parent_participant_id.is_none()
                && dispatch.resumed_from_participant_id.is_none()
                && dispatch.retained_worker_launch_authority.is_some()
        }
        E2MemberLaunchKindV1::Fork => {
            dispatch.parent_participant_id.as_deref() == activation.source_participant_id.as_deref()
                && dispatch.resumed_from_participant_id.is_none()
                && dispatch.retained_worker_launch_authority.is_none()
        }
    };
    if !expected_lineage
        || activation.activation_id != exact_activation_id
        || activation.commitment_ref != activation.immutable_worker_cap_ref
        || activation.orchestration_session_id != dispatch.orchestration_session_id
        || activation.caller_participant_id != dispatch.orchestrator_participant_id
        || activation.target_backend_id != dispatch.backend_id
        || activation.retained_participant_id != dispatch.participant_id
        || activation.bootstrap_run_id != dispatch.run_id
        || activation.target_world.world_id != dispatch.world_id
        || activation.target_world.world_id != binding.world_id
        || activation.target_world.world_generation != dispatch.world_generation
        || activation.target_world.world_generation != binding.world_generation
        || acceptance_context
            .is_some_and(|context| activation.caller_backend_id != context.caller_backend_id)
    {
        return Err(crate::service::BadRequestError::new(
            "E2 launch activation conflicts with member dispatch, lineage, cap, or exact world binding"
                .to_string(),
        )
        .into());
    }
    Ok(Some(RetainedTurnWorkerCapIdentity {
        commitment_ref: activation.immutable_worker_cap_ref.clone(),
        created_revision: activation.immutable_worker_cap_created_revision,
        application_revision: activation.immutable_worker_cap_application_revision,
    }))
}

fn validate_member_runtime_binary(
    dispatch: &MemberDispatchRequestV1,
) -> Result<std::path::PathBuf> {
    let path = Path::new(&dispatch.resolved_runtime.binary_path);
    if !path.is_file() {
        return Err(anyhow!(
            "member_dispatch.resolved_runtime.binary_path does not exist or is not a file: {}",
            dispatch.resolved_runtime.binary_path
        ));
    }
    Ok(path.to_path_buf())
}

struct PreparedMemberRuntimeLauncher {
    launcher_path: std::path::PathBuf,
    launcher_dir: std::path::PathBuf,
    env: Vec<(String, String)>,
}

#[derive(Debug, Default, Deserialize)]
struct CodexSeedHomeConfig {
    model: Option<String>,
    model_provider: Option<String>,
    provider: Option<String>,
    base_url: Option<String>,
    openai_base_url: Option<String>,
    #[serde(default)]
    model_providers: BTreeMap<String, CodexSeedProviderRoutingConfig>,
    #[serde(default)]
    providers: BTreeMap<String, CodexSeedProviderRoutingConfig>,
}

#[derive(Clone, Debug, Default, Deserialize)]
struct CodexSeedProviderRoutingConfig {
    base_url: Option<String>,
    #[serde(default, flatten)]
    unsupported_fields: BTreeMap<String, toml::Value>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
struct CodexBaseUrlConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    base_url: Option<String>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
struct CodexProjectTrustConfig {
    trust_level: String,
}

#[derive(Debug, Serialize)]
struct CodexStartupSubset {
    model: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    model_provider: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    provider: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    base_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    openai_base_url: Option<String>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    model_providers: BTreeMap<String, CodexBaseUrlConfig>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    providers: BTreeMap<String, CodexBaseUrlConfig>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    projects: BTreeMap<String, CodexProjectTrustConfig>,
}

fn prepare_runtime_env_for_member_backend(
    runtime_env: &mut BTreeMap<String, String>,
    backend_kind: MemberRuntimeBackendKindV1,
    workspace_dir: &Path,
    launcher_dir: &Path,
) -> Result<()> {
    match backend_kind {
        MemberRuntimeBackendKindV1::Codex => {
            prepare_codex_runtime_env(runtime_env, workspace_dir, launcher_dir)?;
        }
        MemberRuntimeBackendKindV1::ClaudeCode => {}
    }
    Ok(())
}

fn prepare_codex_runtime_env(
    runtime_env: &mut BTreeMap<String, String>,
    workspace_dir: &Path,
    launcher_dir: &Path,
) -> Result<()> {
    let authoritative_project_dir =
        codex_authoritative_project_dir(runtime_env, workspace_dir).to_path_buf();
    let seed_home_raw = runtime_env
        .remove(SUBSTRATE_INTERNAL_CODEX_AUTH_SEED_HOME_ENV)
        .ok_or_else(|| {
            anyhow!(
                "direct cli:codex-world compatibility bridge requires the policy-gated bootstrap seed source; without that bounded bridge input this Packet 2 seam would otherwise launch against the real workspace with ambient repo-local .codex still active"
            )
        })?;

    let seed_home = Path::new(seed_home_raw.trim());
    let codex_home = launcher_dir.join("codex-home");
    let layout = codex::CodexHomeLayout::new(codex_home.clone());
    layout.materialize(true)?;
    layout.seed_auth_from(
        seed_home,
        codex::AuthSeedOptions {
            require_auth: true,
            require_credentials: false,
            ..codex::AuthSeedOptions::default()
        },
    )?;
    write_codex_startup_subset(seed_home, &authoritative_project_dir, &layout)?;
    runtime_env.insert("CODEX_HOME".to_string(), codex_home.display().to_string());
    Ok(())
}

fn codex_authoritative_project_dir<'a>(
    runtime_env: &'a BTreeMap<String, String>,
    workspace_dir: &'a Path,
) -> &'a Path {
    runtime_env
        .get(crate::service::WORLD_PROJECT_DIR_OVERRIDE_ENV)
        .map(String::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(Path::new)
        .unwrap_or(workspace_dir)
}

fn write_codex_startup_subset(
    seed_home: &Path,
    workspace_dir: &Path,
    layout: &codex::CodexHomeLayout,
) -> Result<()> {
    let startup_subset = read_codex_startup_subset(seed_home, workspace_dir)?;
    let rendered = toml::to_string(&startup_subset)
        .context("render bounded Codex startup subset for isolated CODEX_HOME")?;
    fs::write(layout.config_path(), rendered).with_context(|| {
        format!(
            "write bounded Codex startup subset into isolated CODEX_HOME at {}",
            layout.config_path().display()
        )
    })?;
    Ok(())
}

fn read_codex_startup_subset(seed_home: &Path, workspace_dir: &Path) -> Result<CodexStartupSubset> {
    let config_path = seed_home.join("config.toml");
    let raw = fs::read_to_string(&config_path).with_context(|| {
        format!(
            "direct cli:codex-world compatibility bridge could not derive bounded startup config from {}; config.toml is missing or unreadable, so isolated CODEX_HOME would otherwise fall back to Codex defaults",
            config_path.display()
        )
    })?;
    let parsed: CodexSeedHomeConfig = toml::from_str(&raw).with_context(|| {
        format!(
            "direct cli:codex-world compatibility bridge could not parse bounded startup config from {}; config.toml must contain a top-level model for truthful isolated startup",
            config_path.display()
        )
    })?;

    let model = normalize_non_empty_value(parsed.model).ok_or_else(|| {
        anyhow!(
            "direct cli:codex-world compatibility bridge could not derive a bounded startup model from {}; isolated CODEX_HOME would otherwise fall back to an unsupported default model",
            config_path.display()
        )
    })?;

    let model_provider = normalize_non_empty_value(parsed.model_provider);
    let provider = normalize_non_empty_value(parsed.provider);
    let base_url = normalize_non_empty_value(parsed.base_url);
    let openai_base_url = normalize_non_empty_value(parsed.openai_base_url);

    let mut model_providers = BTreeMap::new();
    if let Some(provider_name) = model_provider.as_deref() {
        let selected_base_url = require_selected_provider_base_url(
            &config_path,
            "model_providers",
            provider_name,
            parsed.model_providers.get(provider_name),
        )?;
        model_providers.insert(
            provider_name.to_string(),
            CodexBaseUrlConfig {
                name: Some(provider_name.to_string()),
                base_url: Some(selected_base_url),
            },
        );
    }

    let mut providers = BTreeMap::new();
    if let Some(provider_name) = provider.as_deref() {
        let selected_base_url = require_selected_provider_base_url(
            &config_path,
            "providers",
            provider_name,
            parsed.providers.get(provider_name),
        )?;
        providers.insert(
            provider_name.to_string(),
            CodexBaseUrlConfig {
                name: None,
                base_url: Some(selected_base_url),
            },
        );
    }

    let projects = BTreeMap::from([(
        workspace_dir.display().to_string(),
        CodexProjectTrustConfig {
            trust_level: "untrusted".to_string(),
        },
    )]);

    Ok(CodexStartupSubset {
        model,
        model_provider,
        provider,
        base_url,
        openai_base_url,
        model_providers,
        providers,
        projects,
    })
}

fn ensure_supported_provider_routing_shape(
    config_path: &Path,
    routing_family: &str,
    provider_name: &str,
    selected: &CodexSeedProviderRoutingConfig,
) -> Result<()> {
    if selected.unsupported_fields.is_empty() {
        return Ok(());
    }

    let unsupported_fields = selected
        .unsupported_fields
        .keys()
        .map(String::as_str)
        .collect::<Vec<_>>()
        .join(", ");

    Err(anyhow!(
        "direct cli:codex-world compatibility bridge found unsupported provider routing fields under [{routing_family}.{provider_name}] in {}; Packet 2 only supports base_url for the selected provider block, so isolated CODEX_HOME would otherwise receive incomplete routing config ({unsupported_fields})",
        config_path.display()
    ))
}

fn require_selected_provider_base_url(
    config_path: &Path,
    routing_family: &str,
    provider_name: &str,
    selected: Option<&CodexSeedProviderRoutingConfig>,
) -> Result<String> {
    let selected = selected.ok_or_else(|| {
        anyhow!(
            "direct cli:codex-world compatibility bridge could not derive truthful startup routing from {}; {routing_family}.{provider_name} is selected but its coupled provider block is missing, so isolated CODEX_HOME would otherwise receive an incomplete provider selection",
            config_path.display()
        )
    })?;
    ensure_supported_provider_routing_shape(config_path, routing_family, provider_name, selected)?;
    normalize_non_empty_value(selected.base_url.clone()).ok_or_else(|| {
        anyhow!(
            "direct cli:codex-world compatibility bridge could not derive truthful startup routing from {}; [{routing_family}.{provider_name}] must provide a supported non-empty base_url for the selected provider, so isolated CODEX_HOME would otherwise receive an incomplete provider selection",
            config_path.display()
        )
    })
}

fn normalize_non_empty_value(value: Option<String>) -> Option<String> {
    value.and_then(|value| {
        let trimmed = value.trim();
        (!trimmed.is_empty()).then(|| trimmed.to_string())
    })
}

fn member_runtime_process_working_dir(
    backend_kind: MemberRuntimeBackendKindV1,
    workspace_dir: &Path,
    _launcher_dir: &Path,
) -> std::path::PathBuf {
    match backend_kind {
        // Keep Codex cwd aligned with the real workspace so repo-root and relative-path
        // semantics stay truthful, but mark that workspace untrusted in the isolated
        // user-level config so repo-local `.codex` layers cannot piggyback on this bridge.
        MemberRuntimeBackendKindV1::Codex => workspace_dir.to_path_buf(),
        MemberRuntimeBackendKindV1::ClaudeCode => workspace_dir.to_path_buf(),
    }
}

fn member_runtime_workspace_access_extensions(
    backend_kind: MemberRuntimeBackendKindV1,
    workspace_dir: &Path,
) -> BTreeMap<String, serde_json::Value> {
    match backend_kind {
        MemberRuntimeBackendKindV1::Codex => BTreeMap::from([(
            ADD_DIRS_EXTENSION_V1.to_string(),
            json!({
                "dirs": [workspace_dir.display().to_string()],
            }),
        )]),
        MemberRuntimeBackendKindV1::ClaudeCode => BTreeMap::new(),
    }
}

fn prepare_member_runtime_launcher(
    actual_binary_path: &Path,
    placement: &LinuxWorldPlacementContext,
) -> Result<PreparedMemberRuntimeLauncher> {
    let launcher_dir = std::env::temp_dir().join(format!(
        "substrate-member-runtime-entry-{}",
        uuid::Uuid::now_v7()
    ));
    let launcher = prepare_linux_world_entry_launcher(&launcher_dir, actual_binary_path, placement)
        .map_err(|err| anyhow!(err.to_string()))?;

    Ok(PreparedMemberRuntimeLauncher {
        launcher_path: launcher.launcher_path,
        launcher_dir,
        env: launcher.env,
    })
}

fn cleanup_prepared_launcher_dir(launcher_dir: &mut Option<std::path::PathBuf>) {
    if let Some(launcher_dir) = launcher_dir.take() {
        let _ = fs::remove_dir_all(launcher_dir);
    }
}

fn resolve_submitted_turn_policy(
    request: &MemberTurnSubmitRequestV1,
    active: &ActiveMemberRuntime,
) -> Result<Option<transport_api_types::PolicySnapshotV3>> {
    if let Some(carrier) = request.policy_snapshot_carrier.as_ref() {
        let activation = active.e2_launch_activation.as_ref().ok_or_else(|| {
            crate::service::BadRequestError::new(
                "unsupported_legacy_state: retained turn cannot establish E2 launch authority"
                    .to_string(),
            )
        })?;
        let candidate = RetainedTurnWorkerCapIdentity {
            commitment_ref: carrier.immutable_worker_cap_ref.clone(),
            created_revision: carrier.immutable_worker_cap_created_revision,
            application_revision: carrier.immutable_worker_cap_application_revision,
        };
        let authenticated = active
            .authenticated_worker_cap_identity
            .lock()
            .expect("retained worker-cap identity lock poisoned");
        match authenticated.as_ref() {
            Some(existing) if existing != &candidate => {
                return Err(crate::service::BadRequestError::new(
                    "unsupported_legacy_state: retained-turn immutable worker-cap identity changed"
                        .to_string(),
                )
                .into());
            }
            None => {
                return Err(crate::service::BadRequestError::new(
                    "unsupported_legacy_state: E2 member omitted its launch-pinned worker cap"
                        .to_string(),
                )
                .into())
            }
            Some(_) => {}
        }
        if activation.orchestration_session_id != request.orchestration_session_id
            || activation.retained_participant_id != request.participant_id
            || activation.caller_backend_id != carrier.caller_backend_id
            || activation.target_backend_id != request.backend_id
            || activation.target_world.world_id != request.world_id
            || activation.target_world.world_generation != request.world_generation
        {
            return Err(crate::service::BadRequestError::new(
                "retained-turn carrier conflicts with launch-pinned E2 activation".to_string(),
            )
            .into());
        }
        return carrier
            .policy_snapshot()
            .map(Some)
            .map_err(crate::service::BadRequestError::new)
            .map_err(Into::into);
    }
    if active.e2_launch_activation.is_some() {
        return Err(crate::service::BadRequestError::new(
            "unsupported_legacy_state: E2-activated retained turn requires policy_snapshot_carrier"
                .to_string(),
        )
        .into());
    }
    Ok(None)
}

fn prepare_submitted_turn_policy_launcher(
    active: &ActiveMemberRuntime,
) -> Result<PreparedSubmittedTurnPolicyLauncher> {
    write_submitted_turn_policy_launcher(&active.launcher_dir, active.ordinary_effect.is_some())
}

fn write_submitted_turn_policy_launcher(
    launcher_dir: &Path,
    ordinary: bool,
) -> Result<PreparedSubmittedTurnPolicyLauncher> {
    let bytes = if ordinary {
        ORDINARY_HOME_POLICY_LAUNCHER_BYTES
    } else {
        SUBMITTED_TURN_POLICY_LAUNCHER_BYTES
    };
    let launcher_dir_metadata = fs::symlink_metadata(launcher_dir).with_context(|| {
        format!(
            "inspect retained-turn launcher directory {}",
            launcher_dir.display()
        )
    })?;
    if !launcher_dir_metadata.file_type().is_dir()
        || launcher_dir_metadata.uid() != unsafe { libc::geteuid() }
        || launcher_dir_metadata.permissions().mode() & 0o022 != 0
    {
        anyhow::bail!(
            "retained-turn launcher directory is not an owned non-writable-by-others directory"
        );
    }

    let world_entry_path = launcher_dir.join("world-entry.sh");
    let world_entry_metadata = fs::symlink_metadata(&world_entry_path).with_context(|| {
        format!(
            "inspect retained world entry {}",
            world_entry_path.display()
        )
    })?;
    if !world_entry_metadata.file_type().is_file()
        || world_entry_metadata.uid() != unsafe { libc::geteuid() }
        || world_entry_metadata.permissions().mode() & 0o111 == 0
    {
        anyhow::bail!("retained world entry is not an owned executable regular file");
    }

    let path = launcher_dir.join(format!(
        "e2-retained-turn-entry-{}.sh",
        uuid::Uuid::now_v7()
    ));
    let mut file = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o700)
        .custom_flags(libc::O_NOFOLLOW | libc::O_CLOEXEC)
        .open(&path)
        .with_context(|| format!("create retained-turn policy launcher {}", path.display()))?;
    file.write_all(bytes)
        .with_context(|| format!("write retained-turn policy launcher {}", path.display()))?;
    file.sync_all()
        .with_context(|| format!("fsync retained-turn policy launcher {}", path.display()))?;
    drop(file);

    let metadata = fs::symlink_metadata(&path)
        .with_context(|| format!("read back retained-turn policy launcher {}", path.display()))?;
    if !metadata.file_type().is_file()
        || metadata.uid() != unsafe { libc::geteuid() }
        || metadata.permissions().mode() & 0o777 != 0o700
        || metadata.len() != bytes.len() as u64
        || fs::read(&path)? != bytes
    {
        let _ = fs::remove_file(&path);
        anyhow::bail!("retained-turn policy launcher exact readback changed");
    }
    fs::File::open(launcher_dir)
        .and_then(|directory| directory.sync_all())
        .with_context(|| {
            format!(
                "fsync retained-turn launcher directory {}",
                launcher_dir.display()
            )
        })?;

    Ok(PreparedSubmittedTurnPolicyLauncher { path })
}

fn build_submitted_turn_run_request(
    active: &ActiveMemberRuntime,
    prompt: String,
    uaa_session_id: &str,
    policy_snapshot: Option<&transport_api_types::PolicySnapshotV3>,
) -> Result<AgentWrapperRunRequest> {
    let mut extensions =
        member_runtime_workspace_access_extensions(active.backend_kind, &active.workspace_dir);
    extensions.insert(
        SESSION_RESUME_EXTENSION_V1.to_string(),
        json!({
            "selector": "id",
            "id": uaa_session_id,
        }),
    );

    let env = match policy_snapshot {
        Some(snapshot) => {
            let placement_root = active
                .env
                .get(crate::service::WORLD_PROJECT_DIR_OVERRIDE_ENV)
                .map(std::path::PathBuf::from)
                .ok_or_else(|| {
                    anyhow!(
                        "authenticated retained-turn policy requires the launch-authenticated placement root"
                    )
                })?;
            if !placement_root.is_absolute() || placement_root != active.workspace_dir {
                anyhow::bail!(
                    "authenticated retained-turn placement root changed from retained runtime"
                );
            }
            let mut baseline_env = active
                .env
                .iter()
                .map(|(key, value)| (key.clone(), value.clone()))
                .collect::<HashMap<_, _>>();
            baseline_env.remove(SUBSTRATE_E2_TURN_ACTUAL_BINARY_ENV);
            baseline_env.remove(SUBSTRATE_E2_TURN_WORLD_ENTRY_ENV);
            let mut turn_env = crate::service::build_member_runtime_turn_env(
                baseline_env,
                snapshot,
                &placement_root,
                &active.process_working_dir,
            )?;
            let retained_binary = active.binary_path.to_str().ok_or_else(|| {
                anyhow!(
                    "authenticated retained-turn binary path is not valid UTF-8 for strict Landlock transport"
                )
            })?;
            if !active.binary_path.is_absolute() || retained_binary.contains('\n') {
                anyhow::bail!(
                    "authenticated retained-turn binary path is not an absolute single Landlock rule target"
                );
            }
            let retained_binary_metadata =
                fs::symlink_metadata(&active.binary_path).with_context(|| {
                    format!(
                        "revalidate authenticated retained-turn binary {}",
                        active.binary_path.display()
                    )
                })?;
            if !retained_binary_metadata.file_type().is_file() {
                anyhow::bail!(
                    "authenticated retained-turn binary is no longer a regular non-symlink file"
                );
            }
            let read_paths = turn_env
                .entry(crate::service::WORLD_FS_LANDLOCK_READ_ALLOWLIST_ENV.to_string())
                .or_default();
            if !read_paths.lines().any(|path| path == retained_binary) {
                if !read_paths.is_empty() {
                    read_paths.push('\n');
                }
                read_paths.push_str(retained_binary);
            }
            turn_env.insert(
                SUBSTRATE_E2_TURN_ACTUAL_BINARY_ENV.to_string(),
                active.binary_path.display().to_string(),
            );
            turn_env.insert(
                SUBSTRATE_E2_TURN_WORLD_ENTRY_ENV.to_string(),
                active
                    .launcher_dir
                    .join("world-entry.sh")
                    .display()
                    .to_string(),
            );
            turn_env.into_iter().collect::<BTreeMap<_, _>>()
        }
        None => active.env.clone(),
    };

    Ok(AgentWrapperRunRequest {
        prompt,
        working_dir: Some(active.process_working_dir.clone()),
        timeout: None,
        env,
        extensions,
    })
}

#[allow(clippy::too_many_arguments)]
fn frame_from_wrapper_event(
    context: &MemberStreamContext,
    binding: &SharedWorldBindingSnapshot,
    span_id: &str,
    wrapper_event: AgentWrapperEvent,
    emitted_registered: &mut bool,
    mode: MemberStreamMode,
    agent_id: &str,
    producer: &mut RuntimeEventStreamProducer,
) -> Result<Option<ExecuteStreamFrame>> {
    if should_suppress_untyped_retained_transport_advisory(context, mode, &wrapper_event) {
        return Ok(None);
    }
    let prepared = agent_event_from_wrapper_event(
        context,
        binding,
        span_id,
        wrapper_event,
        emitted_registered,
        mode,
        agent_id,
    )?;
    let mut frame = producer.event(prepared.event)?;
    if let Some(facet) = prepared.worker_event_facet {
        let ExecuteStreamFrame::Event {
            frame_identity,
            event,
        } = &mut frame
        else {
            unreachable!("producer.event must return an Event frame");
        };
        let event_identity = event.event_identity.clone().ok_or_else(|| {
            anyhow!("member runtime typed worker event omitted event_identity after stamping")
        })?;
        event.worker_event = Some(world_worker_event_from_facet(
            context,
            binding,
            frame_identity,
            &event_identity,
            event.ts,
            facet,
        )?);
        event
            .validate_identity_contract()
            .map_err(anyhow::Error::msg)?;
    }
    Ok(Some(frame))
}

fn should_suppress_untyped_retained_transport_advisory(
    context: &MemberStreamContext,
    mode: MemberStreamMode,
    wrapper_event: &AgentWrapperEvent,
) -> bool {
    mode == MemberStreamMode::SubmittedTurn
        && context.acceptance_context.is_some()
        && (wrapper_event.kind == AgentWrapperEventKind::Status
            || (wrapper_event.kind == AgentWrapperEventKind::TextOutput
                && context.authenticated_thread_id.is_none()))
        && typed_retained_thread_id_from_data(wrapper_event.data.as_ref()).is_none()
        && normalized_world_worker_event_class_label(wrapper_event.data.as_ref()).is_none()
        && normalized_world_worker_stream_event_type(wrapper_event.data.as_ref()).is_none()
        && !normalized_world_worker_runtime_tools_facet(wrapper_event.data.as_ref())
}

/// Emit a typed status report followed by its exact original Exit frame.
/// The report never changes old Exit semantics in a stream that lacks it.
#[allow(clippy::too_many_arguments)]
async fn prepare_retained_native_bootstrap_frames(
    active: &ActiveMemberRuntime,
    context: &MemberStreamContext,
    binding: &SharedWorldBindingSnapshot,
    span_id: &str,
    completion: AgentWrapperCompletion,
    terminal: &NativeTerminalObservation,
    emitted_registered: bool,
    producer: &mut RuntimeEventStreamProducer,
) -> Result<Vec<ExecuteStreamFrame>> {
    use std::os::unix::process::ExitStatusExt;
    use transport_api_types::{
        NativeProcessTerminationV1, NativeSettlementAcknowledgmentV1, OrdinaryGatewayDisposalV1,
        RetainedBootstrapCompletionV1, RetainedBootstrapObservationV1,
    };
    let observed_terminal = terminal.wait().await;
    observed_terminal
        .require_checked()
        .map_err(|error| anyhow!(error))?;
    if !emitted_registered
        || !completion.status.success()
        || !observed_terminal
            .raw
            .as_ref()
            .is_ok_and(|raw| raw.status == completion.status)
        || completion.data.as_ref().is_some_and(|data| {
            data.get(RetainedBootstrapCompletionV1::EVENT_DATA_KEY)
                .is_some()
                || data
                    .get(transport_api_types::RetainedForkBootstrapCompletionV1::EVENT_DATA_KEY)
                    .is_some()
                || data
                    .get(transport_api_types::OrdinaryInitialBootstrapRetirementV1::EVENT_DATA_KEY)
                    .is_some()
        })
    {
        anyhow::bail!("retained bootstrap lacks exact checked native Registered/completion");
    }
    let effect = active
        .ordinary_effect
        .as_ref()
        .ok_or_else(|| anyhow!("ordinary bootstrap owner missing"))?;
    let activation = active
        .e2_launch_activation
        .clone()
        .ok_or_else(|| anyhow!("ordinary bootstrap activation missing"))?;
    let native_session_id = active
        .uaa_session_id()
        .ok_or_else(|| anyhow!("ordinary bootstrap native identity missing"))?;
    let history = active
        .ordinary_native_history
        .as_ref()
        .ok_or_else(|| anyhow!("ordinary native history owner missing"))?;
    history.in_use.require_usable()?;
    let fork_provenance = match (activation.launch_kind, history.fork_source.as_ref()) {
        (E2MemberLaunchKindV1::FreshSpawn, None) => None,
        (E2MemberLaunchKindV1::Fork, Some(source)) => {
            // Held data ancestry survives source Stop. This does not grant the
            // historical source caller another send or account permission.
            source.history.revalidate()?;
            let original = source
                .active
                .ordinary_effect
                .as_ref()
                .ok_or_else(|| anyhow!("ordinary Fork source admission missing"))?;
            source.history.require_original_worker(
                original.ordinary_start_selection(),
                &original.authenticated_runtime(),
                original.requested_selection(),
            )?;
            if original.ordinary_start_selection() != effect.ordinary_start_selection()
                || source.active.uaa_session_id().as_deref() != Some(source.session_id.as_str())
            {
                anyhow::bail!("ordinary Fork source native ancestry changed");
            }
            let transferred_child_native_session_id = source
                .ownership
                .child_session
                .lock()
                .map_err(|_| anyhow!("ordinary Fork ownership lock poisoned"))?
                .clone()
                .ok_or_else(|| anyhow!("ordinary Fork child transfer unobserved"))?;
            Some(transport_api_types::OrdinaryForkBootstrapProvenanceV1 {
                source_runtime: original.authenticated_runtime(),
                source_activation: source
                    .active
                    .e2_launch_activation
                    .clone()
                    .ok_or_else(|| anyhow!("ordinary Fork source activation missing"))?,
                source_native_session_id: source.session_id.clone(),
                transferred_child_native_session_id,
            })
        }
        _ => anyhow::bail!("ordinary bootstrap launch kind/history ancestry conflicts"),
    };
    // Fork's raw status is protocol success, not the MCP process wait. Only
    // actual copied SDK child waits may populate its physical observation.
    let native_processes = if fork_provenance.is_some() {
        match observed_terminal.initial_bootstrap_process_closeout()
            .map_err(|error| anyhow!(error))? {
            transport_api_types::OrdinaryInitialBootstrapProcessCloseoutV1::ProcessesWaited { processes } => processes,
            transport_api_types::OrdinaryInitialBootstrapProcessCloseoutV1::ClosedBeforeAnySpawn {} =>
                anyhow::bail!("Fork bootstrap closed before any actual app-server spawn"),
        }
    } else {
        Vec::new()
    };
    let native_termination = if fork_provenance.is_some() {
        native_processes
            .first()
            .ok_or_else(|| anyhow!("Fork child wait snapshot is empty"))?
            .termination
            .clone()
    } else {
        NativeProcessTerminationV1::Exited {
            exit_code: completion
                .status
                .code()
                .ok_or_else(|| anyhow!("native bootstrap was signaled"))?,
        }
    };
    // Clone owned cleanup custody under the mutex; no synchronous guard spans
    // the asynchronous wait. This re-observes the exact already closed lease.
    let gateway = active
        .ordinary_native_home
        .lock()
        .map_err(|_| anyhow!("ordinary native home poisoned"))?
        .as_ref()
        .map(|home| Arc::clone(&home.gateway))
        .ok_or_else(|| anyhow!("ordinary native cleanup owner missing"))?;
    let (pid, status) = gateway.finish_owned().await?;
    let gateway_termination = match (status.code(), status.signal()) {
        (Some(exit_code), _) => NativeProcessTerminationV1::Exited { exit_code },
        (_, Some(signal)) => NativeProcessTerminationV1::Signaled { signal },
        _ => anyhow::bail!("ordinary gateway wait omitted a terminal status"),
    };
    let mut event = AgentEvent::message(
        active.agent_id.as_str(),
        context.orchestration_session_id.clone(),
        context.run_id.clone(),
        MessageEventKind::Status,
        "retained bootstrap completed",
    );
    stamp_event_identity(
        &mut event,
        active.agent_id.as_str(),
        context,
        binding,
        span_id,
        None,
    );
    let mut announcement = producer.event(event)?;
    let exit = producer.exit(
        exit_code_from_status(&completion.status),
        span_id.into(),
        Vec::new(),
        None,
        ProcessTelemetry::default(),
    )?;
    let ExecuteStreamFrame::Exit {
        frame_identity,
        event_identity,
        terminal_identity,
        ..
    } = &exit
    else {
        unreachable!()
    };
    let observation = RetainedBootstrapObservationV1 {
        source_start: effect.ordinary_start_selection().clone(),
        runtime: effect.authenticated_runtime(),
        activation,
        native_session_id,
        span_id: span_id.into(),
        frame_identity: frame_identity.clone(),
        event_identity: event_identity.clone(),
        terminal_identity: terminal_identity.clone(),
        native_settlement: NativeSettlementAcknowledgmentV1::SdkChecked,
        native_termination,
        gateway_disposal: OrdinaryGatewayDisposalV1::ProcessWaited {
            runtime_id: gateway.runtime_id.clone(),
            pid,
            termination: gateway_termination,
        },
    };
    let ExecuteStreamFrame::Event {
        frame_identity,
        event,
    } = &mut announcement
    else {
        unreachable!()
    };
    if let Some(provenance) = fork_provenance {
        let report = transport_api_types::RetainedForkBootstrapCompletionV1::from_observation(
            observation,
            provenance,
            native_processes,
        )
        .map_err(anyhow::Error::msg)?;
        event.data = json!({(transport_api_types::RetainedForkBootstrapCompletionV1::EVENT_DATA_KEY): report});
        transport_api_types::RetainedForkBootstrapCompletionV1::from_service_event(
            frame_identity,
            event,
        )
        .map_err(anyhow::Error::msg)?;
    } else {
        let report = RetainedBootstrapCompletionV1::from_observation(observation)
            .map_err(anyhow::Error::msg)?;
        event.data = json!({(RetainedBootstrapCompletionV1::EVENT_DATA_KEY): report});
        RetainedBootstrapCompletionV1::from_service_event(frame_identity, event)
            .map_err(anyhow::Error::msg)?;
    }
    Ok(vec![announcement, exit])
}

#[allow(clippy::too_many_arguments)]
fn frames_from_completion(
    context: &MemberStreamContext,
    binding: &SharedWorldBindingSnapshot,
    span_id: &str,
    completion: std::result::Result<AgentWrapperCompletion, AgentWrapperError>,
    cancel_signal: Option<String>,
    emitted_registered: &mut bool,
    mode: MemberStreamMode,
    agent_id: &str,
    producer: &mut RuntimeEventStreamProducer,
) -> Result<Vec<ExecuteStreamFrame>> {
    let frames = match completion {
        Ok(completion) => {
            let mut frames = Vec::new();
            if mode == MemberStreamMode::Bootstrap && !*emitted_registered {
                if let Some(event) = registered_event_from_data(
                    context,
                    binding,
                    span_id,
                    completion.data.as_ref(),
                    agent_id,
                ) {
                    *emitted_registered = true;
                    frames.push(producer.event(event)?);
                }
            }

            frames.push(producer.exit(
                exit_code_from_status(&completion.status),
                span_id.to_string(),
                Vec::new(),
                None,
                ProcessTelemetry::default(),
            )?);
            frames
        }
        Err(AgentWrapperError::Backend { message }) if message == CANCELLED_MESSAGE => {
            vec![producer.exit(
                cancel_exit_code(cancel_signal.as_deref()),
                span_id.to_string(),
                Vec::new(),
                None,
                ProcessTelemetry::default(),
            )?]
        }
        Err(err) => vec![producer.transport_error(format!("member runtime failed: {err}"))?],
    };
    Ok(frames)
}

fn agent_event_from_wrapper_event(
    context: &MemberStreamContext,
    binding: &SharedWorldBindingSnapshot,
    span_id: &str,
    wrapper_event: AgentWrapperEvent,
    emitted_registered: &mut bool,
    mode: MemberStreamMode,
    agent_id: &str,
) -> Result<PreparedMemberRuntimeEvent> {
    if wrapper_event.data.as_ref().is_some_and(|data| {
        data.get(transport_api_types::RetainedBootstrapCompletionV1::EVENT_DATA_KEY)
            .is_some()
            || data
                .get(transport_api_types::RetainedForkBootstrapCompletionV1::EVENT_DATA_KEY)
                .is_some()
            || data
                .get(transport_api_types::OrdinaryInitialBootstrapRetirementV1::EVENT_DATA_KEY)
                .is_some()
    }) {
        anyhow::bail!("native backend supplied a reserved service completion marker");
    }
    let worker_event_facet = normalized_world_worker_event_facet_v1(context, mode, &wrapper_event)?;
    let surfaced_thread_id = worker_event_facet
        .as_ref()
        .map(|facet| facet.thread_id.clone())
        .or_else(|| surfaced_uaa_thread_id_from_data(wrapper_event.data.as_ref()));
    let compatibility_uaa_event =
        normalized_world_worker_payload_uaa_event(wrapper_event.data.as_ref());
    if mode == MemberStreamMode::Bootstrap {
        if let Some(event) = registered_event_from_data(
            context,
            binding,
            span_id,
            wrapper_event.data.as_ref(),
            agent_id,
        ) {
            *emitted_registered = true;
            return Ok(PreparedMemberRuntimeEvent {
                event,
                worker_event_facet: None,
            });
        }
    }

    let compatibility_message =
        wrapper_event_compatibility_message(&wrapper_event, worker_event_facet.is_some());

    let mut event = match wrapper_event.kind {
        AgentWrapperEventKind::Status => AgentEvent::message(
            agent_id,
            context.orchestration_session_id.clone(),
            context.run_id.clone(),
            MessageEventKind::Status,
            compatibility_message.clone(),
        ),
        AgentWrapperEventKind::TextOutput => AgentEvent::message(
            agent_id,
            context.orchestration_session_id.clone(),
            context.run_id.clone(),
            MessageEventKind::TaskProgress,
            compatibility_message.clone(),
        ),
        AgentWrapperEventKind::ToolCall | AgentWrapperEventKind::ToolResult => AgentEvent::message(
            agent_id,
            context.orchestration_session_id.clone(),
            context.run_id.clone(),
            MessageEventKind::TaskProgress,
            compatibility_message.clone(),
        ),
        AgentWrapperEventKind::Error => AgentEvent::alert(
            agent_id,
            context.orchestration_session_id.clone(),
            context.run_id.clone(),
            "agent_wrapper_error",
            compatibility_message.clone(),
        ),
        AgentWrapperEventKind::Unknown => AgentEvent::message(
            agent_id,
            context.orchestration_session_id.clone(),
            context.run_id.clone(),
            MessageEventKind::TaskProgress,
            compatibility_message,
        ),
    };

    stamp_event_identity(
        &mut event,
        agent_id,
        context,
        binding,
        span_id,
        wrapper_event.channel,
    );
    event.thread_id = surfaced_thread_id;

    if wrapper_event.data.is_some() {
        if let Some(obj) = event.data.as_object_mut() {
            if let Some(uaa_event) = compatibility_uaa_event {
                obj.insert("uaa_event".to_string(), uaa_event);
            }
            obj.insert("protocol".to_string(), context.protocol.clone());
        }
    }

    Ok(PreparedMemberRuntimeEvent {
        event,
        worker_event_facet,
    })
}

fn registered_event_from_data(
    context: &MemberStreamContext,
    binding: &SharedWorldBindingSnapshot,
    span_id: &str,
    data: Option<&serde_json::Value>,
    agent_id: &str,
) -> Option<AgentEvent> {
    let data = data?;
    if data.get("schema").and_then(serde_json::Value::as_str) != Some(SESSION_HANDLE_SCHEMA_V1) {
        return None;
    }

    let mut event = AgentEvent {
        ts: chrono::Utc::now(),
        kind: AgentEventKind::Registered,
        data: data.clone(),
        agent_id: agent_id.to_string(),
        orchestration_session_id: context.orchestration_session_id.clone(),
        run_id: context.run_id.clone(),
        parent_run_id: None,
        participant_id: Some(context.participant_id.clone()),
        parent_participant_id: context.parent_participant_id.clone(),
        resumed_from_participant_id: context.resumed_from_participant_id.clone(),
        backend_id: Some(context.backend_id.clone()),
        thread_id: surfaced_uaa_session_id_from_data(Some(data)),
        role: Some(MEMBER_ROLE.to_string()),
        world_id: Some(binding.world_id.clone()),
        world_generation: Some(binding.world_generation),
        cmd_id: None,
        span_id: Some(span_id.to_string()),
        event_identity: None,
        worker_event: None,
        channel: None,
        identity_tuple: None,
        placement_posture: None,
        project: None,
    };
    event.set_pure_agent_telemetry_identity(agent_id.to_string());
    Some(event)
}

fn stamp_event_identity(
    event: &mut AgentEvent,
    agent_id: &str,
    context: &MemberStreamContext,
    binding: &SharedWorldBindingSnapshot,
    span_id: &str,
    channel: Option<String>,
) {
    event.role = Some(MEMBER_ROLE.to_string());
    event.backend_id = Some(context.backend_id.clone());
    event.participant_id = Some(context.participant_id.clone());
    event.parent_participant_id = context.parent_participant_id.clone();
    event.resumed_from_participant_id = context.resumed_from_participant_id.clone();
    event.world_id = Some(binding.world_id.clone());
    event.world_generation = Some(binding.world_generation);
    event.span_id = Some(span_id.to_string());
    event.set_channel(channel);
    event.set_pure_agent_telemetry_identity(agent_id.to_string());
}

fn world_worker_event_from_facet(
    context: &MemberStreamContext,
    binding: &SharedWorldBindingSnapshot,
    frame_identity: &transport_api_types::RuntimeFrameIdentityV1,
    event_identity: &transport_api_types::RuntimeEventIdentityV1,
    emitted_at: chrono::DateTime<chrono::Utc>,
    facet: NormalizedWorldWorkerEventFacetV1,
) -> Result<WorldWorkerEventV1> {
    let acceptance_context = context
        .acceptance_context
        .as_ref()
        .ok_or_else(|| anyhow!("typed retained worker event requires acceptance_context"))?;
    let worker_event = WorldWorkerEventV1 {
        schema_version: 1,
        acceptance_record_id: acceptance_context.proposed_acceptance_record_id.clone(),
        stream_id: frame_identity.stream_id.clone(),
        frame_sequence: frame_identity.frame_sequence,
        event_id: event_identity.event_id.clone(),
        event_sequence: event_identity.event_sequence,
        request_id: acceptance_context.request_id.clone(),
        active_run_id: context.run_id.clone(),
        host_transition_correlation: acceptance_context.host_transition_correlation.clone(),
        causation_message_id: facet.causation_message_id,
        causation_request_id: facet.causation_request_id,
        orchestration_session_id: context.orchestration_session_id.clone(),
        source_participant_id: context.participant_id.clone(),
        target_participant_id: context.orchestrator_participant_id.clone(),
        source_backend_id: context.backend_id.clone(),
        target_backend_id: acceptance_context.caller_backend_id.clone(),
        world_id: binding.world_id.clone(),
        world_generation: binding.world_generation,
        thread_id: facet.thread_id,
        event_class: facet.event_class,
        attention_required: facet.attention_required,
        payload: facet.payload,
        emitted_at,
    };
    worker_event.validate().map_err(anyhow::Error::msg)?;
    Ok(worker_event)
}

fn normalized_world_worker_event_facet_v1(
    context: &MemberStreamContext,
    mode: MemberStreamMode,
    wrapper_event: &AgentWrapperEvent,
) -> Result<Option<NormalizedWorldWorkerEventFacetV1>> {
    if mode != MemberStreamMode::SubmittedTurn {
        return Ok(None);
    }
    let Some(acceptance_context) = context.acceptance_context.as_ref() else {
        return Ok(None);
    };
    let causation_message_id = acceptance_context.message_id.clone().ok_or_else(|| {
        anyhow!("typed retained worker event requires acceptance_context.message_id")
    })?;
    let thread_id = typed_retained_thread_id_from_data(wrapper_event.data.as_ref())
        .or_else(|| context.authenticated_thread_id.clone())
        .ok_or_else(|| anyhow!("typed retained worker event omitted thread_id"))?;
    let event_class = normalized_world_worker_event_class_v1(wrapper_event)?;
    let attention_required = event_class.attention_required_by_default()
        || normalized_world_worker_attention_required(wrapper_event.data.as_ref()).unwrap_or(false);
    let facet = NormalizedWorldWorkerEventFacetV1 {
        schema_version: 1,
        thread_id,
        event_class,
        attention_required,
        causation_message_id,
        causation_request_id: acceptance_context.request_id.clone(),
        payload: normalized_world_worker_event_payload(wrapper_event)?,
    };
    facet.validate().map_err(anyhow::Error::msg)?;
    Ok(Some(facet))
}

fn normalized_world_worker_event_class_v1(
    wrapper_event: &AgentWrapperEvent,
) -> Result<WorldWorkerEventClassV1> {
    let data = wrapper_event.data.as_ref();

    if let Some(label) = normalized_world_worker_event_class_label(data) {
        return match label {
            "reply" => Ok(WorldWorkerEventClassV1::Reply),
            "progress_update" => Ok(WorldWorkerEventClassV1::ProgressUpdate),
            "control_ack" => Ok(WorldWorkerEventClassV1::ControlAck),
            "follow_up_question" => Ok(WorldWorkerEventClassV1::FollowUpQuestion),
            "approval_request" => Ok(WorldWorkerEventClassV1::ApprovalRequest),
            "blocked" => Ok(WorldWorkerEventClassV1::Blocked),
            "attention_required" => Ok(WorldWorkerEventClassV1::AttentionRequired),
            "fork_request" => Ok(WorldWorkerEventClassV1::ForkRequest),
            "fork_recommendation" => Ok(WorldWorkerEventClassV1::ForkRecommendation),
            "result" => Ok(WorldWorkerEventClassV1::Result),
            "failure" => Ok(WorldWorkerEventClassV1::Failure),
            "approval_response" | "fork_command" | "progress_ack" | "control_directive" => {
                anyhow::bail!(
                    "unsupported_worker_event_class: retained member emitted deferred worker event class {}",
                    label
                )
            }
            other => anyhow::bail!(
                "unsupported_worker_event_class: retained member emitted worker event class {}",
                other
            ),
        };
    }

    if let Some(event_class) = normalized_world_worker_event_class_from_stream_shape(data) {
        return Ok(event_class);
    }
    if let Some(event_class) = normalized_world_worker_event_class_from_runtime_shape(data) {
        return Ok(event_class);
    }
    if wrapper_event.kind == AgentWrapperEventKind::TextOutput
        && wrapper_event.channel.as_deref() == Some("assistant")
        && wrapper_event
            .text
            .as_deref()
            .is_some_and(|text| !text.is_empty())
    {
        return Ok(WorldWorkerEventClassV1::Reply);
    }
    if wrapper_event.kind == AgentWrapperEventKind::Error
        && wrapper_event.channel.as_deref() == Some("error")
        && wrapper_event
            .message
            .as_deref()
            .is_some_and(|message| !message.is_empty())
    {
        return Ok(WorldWorkerEventClassV1::Failure);
    }

    anyhow::bail!(
        "unsupported_worker_event_shape: retained member emitted no supported typed worker event shape"
    )
}

fn normalized_world_worker_event_class_from_stream_shape(
    data: Option<&serde_json::Value>,
) -> Option<WorldWorkerEventClassV1> {
    let event_type = normalized_world_worker_stream_event_type(data)?;
    match event_type {
        "thread.started" | "thread.resumed" | "turn.started" => {
            Some(WorldWorkerEventClassV1::ProgressUpdate)
        }
        "turn.completed" => Some(WorldWorkerEventClassV1::Result),
        "turn.failed" | "item.failed" | "error" => Some(WorldWorkerEventClassV1::Failure),
        "item.started" | "item.created" | "item.delta" | "item.updated" | "item.completed" => {
            match normalized_world_worker_stream_item_type(data) {
                Some("agent_message")
                    if event_type == "item.completed"
                        || normalized_world_worker_stream_item_status(data)
                            == Some("completed") =>
                {
                    Some(WorldWorkerEventClassV1::Reply)
                }
                Some("error") => Some(WorldWorkerEventClassV1::Failure),
                Some(_) | None => Some(WorldWorkerEventClassV1::ProgressUpdate),
            }
        }
        _ => None,
    }
}

fn normalized_world_worker_event_class_from_runtime_shape(
    data: Option<&serde_json::Value>,
) -> Option<WorldWorkerEventClassV1> {
    if normalized_world_worker_runtime_tools_facet(data) {
        if normalized_world_worker_runtime_tools_facet_failed(data) {
            return Some(WorldWorkerEventClassV1::Failure);
        }
        return Some(WorldWorkerEventClassV1::ProgressUpdate);
    }
    None
}

fn normalized_world_worker_runtime_tools_facet(data: Option<&serde_json::Value>) -> bool {
    normalized_world_worker_string_field(data, &["/schema", "/raw_event/schema"])
        == Some("agent_api.tools.structured.v1")
}

fn normalized_world_worker_runtime_tools_facet_failed(data: Option<&serde_json::Value>) -> bool {
    normalized_world_worker_string_field(data, &["/tool/status", "/raw_event/tool/status"])
        == Some("failed")
        || normalized_world_worker_string_field(data, &["/tool/phase", "/raw_event/tool/phase"])
            == Some("fail")
}

fn normalized_world_worker_stream_event_type(data: Option<&serde_json::Value>) -> Option<&str> {
    normalized_world_worker_string_field(data, &["/type", "/raw_event/type"])
}

fn normalized_world_worker_stream_item_type(data: Option<&serde_json::Value>) -> Option<&str> {
    normalized_world_worker_string_field(
        data,
        &[
            "/item_type",
            "/item/item_type",
            "/item/type",
            "/raw_event/item_type",
            "/raw_event/item/item_type",
            "/raw_event/item/type",
        ],
    )
}

fn normalized_world_worker_stream_item_status(data: Option<&serde_json::Value>) -> Option<&str> {
    normalized_world_worker_string_field(
        data,
        &[
            "/status",
            "/item/status",
            "/raw_event/status",
            "/raw_event/item/status",
        ],
    )
}

fn normalized_world_worker_event_class_label(data: Option<&serde_json::Value>) -> Option<&str> {
    normalized_world_worker_string_field(data, &["/event_class", "/raw_event/event_class"])
}

fn normalized_world_worker_attention_required(data: Option<&serde_json::Value>) -> Option<bool> {
    normalized_world_worker_bool_field(
        data,
        &["/attention_required", "/raw_event/attention_required"],
    )
}

fn normalized_world_worker_event_payload(
    wrapper_event: &AgentWrapperEvent,
) -> Result<serde_json::Value> {
    let data = wrapper_event.data.as_ref();
    for (pointer, label) in [
        ("/payload", "payload"),
        ("/raw_event/payload", "raw_event.payload"),
        ("/content", "content"),
        ("/raw_event/content", "raw_event.content"),
        ("/tool", "tool"),
        ("/raw_event/tool", "raw_event.tool"),
    ] {
        if let Some(value) = data.and_then(|data| data.pointer(pointer)) {
            if !value.is_object() {
                anyhow::bail!(
                    "unsupported_worker_event_shape: retained member emitted malformed {} object",
                    label
                );
            }
        }
    }

    let mut payload = normalized_world_worker_payload_object(data);
    if !payload.contains_key("message") {
        if let Some(message) = normalized_world_worker_payload_message(data) {
            payload.insert("message".to_string(), serde_json::Value::String(message));
        } else if wrapper_event.kind == AgentWrapperEventKind::TextOutput
            && wrapper_event.channel.as_deref() == Some("assistant")
        {
            if let Some(text) = wrapper_event
                .text
                .as_deref()
                .filter(|text| !text.is_empty())
            {
                payload.insert(
                    "message".to_string(),
                    serde_json::Value::String(text.to_string()),
                );
            }
        } else if wrapper_event.kind == AgentWrapperEventKind::Error
            && wrapper_event.channel.as_deref() == Some("error")
        {
            if let Some(message) = wrapper_event
                .message
                .as_deref()
                .filter(|message| !message.is_empty())
            {
                payload.insert(
                    "message".to_string(),
                    serde_json::Value::String(message.to_string()),
                );
            }
        }
    }
    if !payload.contains_key("event_id") {
        if let Some(event_id) =
            normalized_world_worker_string_field(data, &["/event_id", "/raw_event/event_id"])
        {
            payload.insert(
                "event_id".to_string(),
                serde_json::Value::String(event_id.to_string()),
            );
        }
    }
    if !payload.contains_key("message_id") {
        if let Some(message_id) =
            normalized_world_worker_string_field(data, &["/message_id", "/raw_event/message_id"])
        {
            payload.insert(
                "message_id".to_string(),
                serde_json::Value::String(message_id.to_string()),
            );
        }
    }
    let payload_has_supported_semantics = !payload.is_empty();
    if let Some(uaa_event) = normalized_world_worker_payload_uaa_event(data) {
        payload.entry("uaa_event".to_string()).or_insert(uaa_event);
    }
    if normalized_world_worker_event_class_label(data).is_some() && !payload_has_supported_semantics
    {
        anyhow::bail!(
            "unsupported_worker_event_shape: retained member emitted no supported typed payload"
        );
    }
    if payload.is_empty() {
        anyhow::bail!(
            "unsupported_worker_event_shape: retained member emitted no supported typed payload"
        );
    }
    Ok(serde_json::Value::Object(payload))
}

fn normalized_world_worker_payload_message(data: Option<&serde_json::Value>) -> Option<String> {
    normalized_world_worker_string_field(
        data,
        &[
            "/payload/message",
            "/raw_event/payload/message",
            "/message",
            "/raw_event/message",
            "/content/text",
            "/raw_event/content/text",
        ],
    )
    .map(str::to_string)
}

fn wrapper_event_default_message(kind: &AgentWrapperEventKind) -> &'static str {
    match kind {
        AgentWrapperEventKind::Status => "member runtime status",
        AgentWrapperEventKind::TextOutput => "member runtime output",
        AgentWrapperEventKind::ToolCall | AgentWrapperEventKind::ToolResult => {
            "member runtime tool activity"
        }
        AgentWrapperEventKind::Error => "member runtime error",
        AgentWrapperEventKind::Unknown => "member runtime emitted an unknown event",
    }
}

fn wrapper_event_compatibility_message(
    wrapper_event: &AgentWrapperEvent,
    typed_retained_event: bool,
) -> String {
    if typed_retained_event {
        return normalized_world_worker_payload_message(wrapper_event.data.as_ref())
            .unwrap_or_else(|| wrapper_event_default_message(&wrapper_event.kind).to_string());
    }

    match wrapper_event.kind {
        AgentWrapperEventKind::Status => wrapper_event
            .message
            .clone()
            .unwrap_or_else(|| wrapper_event_default_message(&wrapper_event.kind).to_string()),
        AgentWrapperEventKind::TextOutput => wrapper_event
            .text
            .clone()
            .unwrap_or_else(|| wrapper_event_default_message(&wrapper_event.kind).to_string()),
        AgentWrapperEventKind::ToolCall
        | AgentWrapperEventKind::ToolResult
        | AgentWrapperEventKind::Error => wrapper_event
            .message
            .clone()
            .unwrap_or_else(|| wrapper_event_default_message(&wrapper_event.kind).to_string()),
        AgentWrapperEventKind::Unknown => {
            wrapper_event_default_message(&wrapper_event.kind).to_string()
        }
    }
}

fn normalized_world_worker_payload_object(
    data: Option<&serde_json::Value>,
) -> serde_json::Map<String, serde_json::Value> {
    for pointer in ["/payload", "/raw_event/payload"] {
        if let Some(serde_json::Value::Object(object)) =
            data.and_then(|value| value.pointer(pointer))
        {
            return normalized_world_worker_sanitized_object(object);
        }
    }
    serde_json::Map::new()
}

fn normalized_world_worker_payload_uaa_event(
    data: Option<&serde_json::Value>,
) -> Option<serde_json::Value> {
    let serde_json::Value::Object(object) = data? else {
        return None;
    };
    let uaa_event = normalized_world_worker_sanitized_object(object);
    (!uaa_event.is_empty()).then_some(serde_json::Value::Object(uaa_event))
}

fn normalized_world_worker_sanitized_object(
    object: &serde_json::Map<String, serde_json::Value>,
) -> serde_json::Map<String, serde_json::Value> {
    object
        .iter()
        .filter(|(key, _)| !normalized_world_worker_secret_key(key))
        .map(|(key, value)| (key.clone(), normalized_world_worker_sanitized_value(value)))
        .collect()
}

fn normalized_world_worker_sanitized_value(value: &serde_json::Value) -> serde_json::Value {
    match value {
        serde_json::Value::Object(object) => {
            serde_json::Value::Object(normalized_world_worker_sanitized_object(object))
        }
        serde_json::Value::Array(items) => serde_json::Value::Array(
            items
                .iter()
                .map(normalized_world_worker_sanitized_value)
                .collect(),
        ),
        _ => value.clone(),
    }
}

fn normalized_world_worker_secret_key(key: &str) -> bool {
    key.eq_ignore_ascii_case("prompt") || key.eq_ignore_ascii_case("command")
}

fn normalized_world_worker_string_field<'a>(
    data: Option<&'a serde_json::Value>,
    pointers: &[&str],
) -> Option<&'a str> {
    let data = data?;
    pointers.iter().find_map(|pointer| {
        data.pointer(pointer)
            .and_then(serde_json::Value::as_str)
            .map(str::trim)
            .filter(|value| !value.is_empty())
    })
}

fn normalized_world_worker_bool_field(
    data: Option<&serde_json::Value>,
    pointers: &[&str],
) -> Option<bool> {
    let data = data?;
    pointers
        .iter()
        .find_map(|pointer| data.pointer(pointer).and_then(serde_json::Value::as_bool))
}

fn surfaced_uaa_session_id_from_data(data: Option<&serde_json::Value>) -> Option<String> {
    let data = data?;
    for pointer in ["/internal/uaa_session_id", "/session/id"] {
        if let Some(session_id) = data.pointer(pointer).and_then(serde_json::Value::as_str) {
            let trimmed = session_id.trim();
            if !trimmed.is_empty() {
                return Some(trimmed.to_string());
            }
        }
    }
    None
}

fn surfaced_uaa_thread_id_from_data(data: Option<&serde_json::Value>) -> Option<String> {
    let data = data?;
    for pointer in [
        "/thread_id",
        "/session/id",
        "/raw_event/thread_id",
        "/raw_event/session/id",
    ] {
        if let Some(thread_id) = data.pointer(pointer).and_then(serde_json::Value::as_str) {
            let trimmed = thread_id.trim();
            if !trimmed.is_empty() {
                return Some(trimmed.to_string());
            }
        }
    }
    None
}

fn typed_retained_thread_id_from_data(data: Option<&serde_json::Value>) -> Option<String> {
    let data = data?;
    for pointer in ["/thread_id", "/raw_event/thread_id"] {
        if let Some(thread_id) = data.pointer(pointer).and_then(serde_json::Value::as_str) {
            let trimmed = thread_id.trim();
            if !trimmed.is_empty() {
                return Some(trimmed.to_string());
            }
        }
    }
    None
}

/// Return an actual current caller only after this turn's admitted worker,
/// World and retained native slot all join. Request labels alone grant nothing.
fn authenticated_ordinary_turn_caller(
    active: &ActiveMemberRuntime,
    effect: &crate::service::OrdinaryWorldEffectContextV1,
) -> Result<String> {
    effect.turn_admission()?;
    // These held identity comparisons perform no effect. Authenticate current
    // permission after they join, before returning the caller to the consumer.
    let transport_api_types::OrdinaryGatewayRuntimeIdentityV1::WorldWorker {
        orchestration_session_id,
        caller_participant_id,
        participant_id,
        backend_id,
        world_id,
        world_generation,
        ..
    } = effect.authenticated_runtime()
    else {
        anyhow::bail!("ordinary retained turn changed runtime role");
    };
    if orchestration_session_id != active.orchestration_session_id
        || participant_id != active.participant_id
        || backend_id != active.backend_id
        || world_id != active.binding.world_id
        || world_generation != active.binding.world_generation
    {
        anyhow::bail!("ordinary current turn differs from held native worker/World");
    }
    effect.revalidate_for_effect()?;
    Ok(caller_participant_id)
}

#[derive(Clone, Copy)]
struct RetainedMemberIdentity<'a> {
    orchestration_session_id: &'a str,
    orchestrator_participant_id: &'a str,
    backend_id: &'a str,
    world_id: &'a str,
    world_generation: u64,
}

impl<'a> RetainedMemberIdentity<'a> {
    fn from_active(active: &'a ActiveMemberRuntime) -> Self {
        Self {
            orchestration_session_id: &active.orchestration_session_id,
            orchestrator_participant_id: &active.orchestrator_participant_id,
            backend_id: &active.backend_id,
            world_id: &active.binding.world_id,
            world_generation: active.binding.world_generation,
        }
    }
}

fn validate_submit_turn_request(
    req: &MemberTurnSubmitRequestV1,
    retained: RetainedMemberIdentity<'_>,
) -> Result<()> {
    if retained.orchestration_session_id != req.orchestration_session_id {
        return Err(crate::service::BadRequestError::new(format!(
            "member_turn_submit.orchestration_session_id mismatch (expected {}, got {})",
            retained.orchestration_session_id, req.orchestration_session_id
        ))
        .into());
    }
    if retained.orchestrator_participant_id != req.orchestrator_participant_id {
        return Err(crate::service::BadRequestError::new(format!(
            "member_turn_submit.orchestrator_participant_id mismatch (expected {}, got {})",
            retained.orchestrator_participant_id, req.orchestrator_participant_id
        ))
        .into());
    }
    if retained.backend_id != req.backend_id {
        return Err(crate::service::BadRequestError::new(format!(
            "member_turn_submit.backend_id mismatch (expected {}, got {})",
            retained.backend_id, req.backend_id
        ))
        .into());
    }
    if retained.world_id != req.world_id {
        return Err(crate::service::BadRequestError::new(format!(
            "member_turn_submit.world_id mismatch (expected {}, got {})",
            retained.world_id, req.world_id
        ))
        .into());
    }
    if retained.world_generation != req.world_generation {
        return Err(crate::service::BadRequestError::new(format!(
            "member_turn_submit.world_generation mismatch (expected {}, got {})",
            retained.world_generation, req.world_generation
        ))
        .into());
    }
    Ok(())
}

fn publish_member_replay_frame(
    publisher: Option<&RuntimeReplayPublisher>,
    tx: &tokio::sync::mpsc::UnboundedSender<ExecuteStreamFrame>,
    frame: ExecuteStreamFrame,
    frame_kind: &'static str,
) {
    if let Err(error) = publish_replayable_frame(publisher, tx, frame) {
        tracing::error!(error = %error, frame_kind, "failed to retain replayable member runtime frame");
    }
}

fn stream_response(
    rx: tokio::sync::mpsc::UnboundedReceiver<ExecuteStreamFrame>,
) -> Result<Response> {
    let stream = UnboundedReceiverStream::new(rx).map(|frame| {
        let payload = frame
            .canonical_ndjson_bytes()
            .expect("serialize identified member runtime frame");
        Ok::<Bytes, Infallible>(Bytes::from(payload))
    });

    Response::builder()
        .status(StatusCode::OK)
        .header("content-type", "application/x-ndjson")
        .body(boxed(StreamBody::new(stream)))
        .map_err(|err| anyhow!("failed to build member runtime stream response: {err}"))
}

fn durable_stream_response(stream: MemberTurnJoinSubscription) -> Result<Response> {
    let stream = stream.map(|frame| {
        let payload = frame
            .canonical_ndjson_bytes()
            .expect("serialize durable E2 member-turn frame");
        Ok::<Bytes, Infallible>(Bytes::from(payload))
    });

    Response::builder()
        .status(StatusCode::OK)
        .header("content-type", "application/x-ndjson")
        .body(boxed(StreamBody::new(stream)))
        .map_err(|error| anyhow!("failed to build durable E2 member-turn stream: {error}"))
}

fn validate_cancel_signal(sig: &str) -> Result<()> {
    match sig.trim().to_ascii_uppercase().as_str() {
        "INT" | "SIGINT" | "TERM" | "SIGTERM" | "HUP" | "SIGHUP" | "QUIT" | "SIGQUIT" => Ok(()),
        _ => Err(anyhow!("unsupported execute cancellation signal: {sig}")),
    }
}

fn cancel_exit_code(sig: Option<&str>) -> i32 {
    match sig.unwrap_or("INT").trim().to_ascii_uppercase().as_str() {
        "HUP" | "SIGHUP" => 129,
        "QUIT" | "SIGQUIT" => 131,
        "TERM" | "SIGTERM" => 143,
        _ => 130,
    }
}

fn exit_code_from_status(status: &std::process::ExitStatus) -> i32 {
    #[cfg(unix)]
    {
        use std::os::unix::process::ExitStatusExt;

        if let Some(code) = status.code() {
            return code;
        }
        if let Some(sig) = status.signal() {
            return 128 + sig;
        }
    }

    status.code().unwrap_or(1)
}

fn map_wrapper_error(err: AgentWrapperError) -> anyhow::Error {
    match err {
        AgentWrapperError::UnknownBackend { agent_kind } => {
            anyhow!("unsupported member runtime backend: {agent_kind}")
        }
        AgentWrapperError::UnknownRuntimeFamily { runtime_family } => {
            anyhow!("unsupported member runtime family: {runtime_family}")
        }
        AgentWrapperError::UnsupportedTargetTriple {
            runtime_family,
            target_triple,
        } => anyhow!(
            "member runtime target triple '{target_triple}' is unsupported for runtime family '{runtime_family}'"
        ),
        AgentWrapperError::MissingValidatedRuntime {
            runtime_family,
            target_triple,
        } => anyhow!(
            "member runtime target triple '{target_triple}' has no validated runtime for runtime family '{runtime_family}'"
        ),
        AgentWrapperError::UnsupportedCapability {
            agent_kind,
            capability,
        } => anyhow!("member runtime backend {agent_kind} does not support {capability}"),
        AgentWrapperError::InvalidAgentKind { message }
        | AgentWrapperError::InvalidRequest { message }
        | AgentWrapperError::Backend { message } => anyhow!(message),
    }
}

fn persist_prelaunch_failure(
    leader: &mut MemberTurnJoinLeader,
    error_code: &str,
    http_status: u16,
) -> anyhow::Error {
    match leader.fail_before_launch(error_code, http_status) {
        Ok(()) => anyhow::Error::new(MemberTurnJoinError::FailedBeforeLaunch {
            error_code: error_code.to_string(),
            http_status,
        }),
        Err(error) => anyhow::Error::new(error),
    }
}

impl ActiveMemberRuntime {
    fn remember_uaa_session_id(&self, session_id: String) {
        if let Ok(mut guard) = self.uaa_session_id.lock() {
            if let Some(source) = self
                .ordinary_native_history
                .as_ref()
                .and_then(|history| history.fork_source.as_ref())
            {
                if let Err(error) = source
                    .ownership
                    .observe_child_session(&source.session_id, &session_id)
                {
                    tracing::warn!(error = %error, "ordinary Fork child identity refused");
                    drop(guard);
                    self.cancel_bootstrap();
                    return;
                }
            }
            *guard = Some(session_id);
        }
    }

    fn uaa_session_id(&self) -> Option<String> {
        self.uaa_session_id
            .lock()
            .ok()
            .and_then(|guard| guard.clone())
    }

    fn bootstrap_last_signal(&self) -> Option<String> {
        self.bootstrap.lock().ok().and_then(|guard| {
            guard.as_ref().and_then(|bootstrap| {
                bootstrap
                    .initial_control
                    .as_ref()
                    .and_then(|control| control.initial_signal().ok().flatten())
                    .or_else(|| bootstrap.last_signal.clone())
            })
        })
    }

    fn initial_control_requested(&self) -> Result<bool> {
        let bootstrap = self
            .bootstrap
            .lock()
            .map_err(|_| anyhow!("initial bootstrap state poisoned"))?;
        match bootstrap
            .as_ref()
            .and_then(|bootstrap| bootstrap.initial_control.as_ref())
        {
            Some(control) => control.initial_control_requested(),
            None => Ok(false),
        }
    }

    fn has_active_bootstrap(&self) -> bool {
        self.bootstrap
            .lock()
            .ok()
            .and_then(|guard| guard.as_ref().map(|_| ()))
            .is_some()
    }

    fn is_bootstrapping_or_resumable(&self) -> bool {
        self.has_active_bootstrap() || self.uaa_session_id().is_some()
    }

    fn cancel_bootstrap(&self) {
        if let Ok(guard) = self.bootstrap.lock() {
            if let Some(bootstrap) = guard.as_ref() {
                if let Some(terminal) = bootstrap.terminal.as_ref() {
                    terminal.request_route_disposal();
                }
                if let Some(cancel) = bootstrap.cancel.as_ref() {
                    cancel.cancel();
                }
            }
        }
    }

    fn cancel_bootstrap_if_span_matches(&self, span_id: &str, signal: &str) -> bool {
        let Ok(mut guard) = self.bootstrap.lock() else {
            return false;
        };
        let Some(bootstrap) = guard.as_mut() else {
            return false;
        };
        if bootstrap.span_id != span_id {
            return false;
        }

        bootstrap.last_signal = Some(signal.to_string());
        if let Some(terminal) = bootstrap.terminal.as_ref() {
            terminal.request_route_disposal();
        }
        if let Some(cancel) = bootstrap.cancel.as_ref() {
            cancel.cancel();
        }
        true
    }

    fn close_bootstrap(&self) {
        if let Ok(mut guard) = self.bootstrap.lock() {
            guard.take();
        }
        if let Ok(mut home) = self.ordinary_native_home.lock() {
            home.take();
        }
    }

    fn cleanup_launcher_dir(&self) {
        if self.ordinary_native_history.is_none() {
            let _ = fs::remove_dir_all(&self.launcher_dir);
        }
    }

    fn submit_context(
        &self,
        run_id: String,
        acceptance_context: Option<transport_api_types::WorldWorkAcceptanceContextV1>,
    ) -> MemberStreamContext {
        MemberStreamContext {
            orchestration_session_id: self.orchestration_session_id.clone(),
            run_id,
            acceptance_context,
            participant_id: self.participant_id.clone(),
            orchestrator_participant_id: self.orchestrator_participant_id.clone(),
            parent_participant_id: self.parent_participant_id.clone(),
            resumed_from_participant_id: self.resumed_from_participant_id.clone(),
            backend_id: self.backend_id.clone(),
            protocol: self.protocol.clone(),
            authenticated_thread_id: self.uaa_session_id(),
        }
    }
}

impl MemberStreamContext {
    fn validate_acceptance_for_mode(&self, mode: MemberStreamMode) -> Result<()> {
        let Some(context) = self.acceptance_context.as_ref() else {
            return Ok(());
        };
        context
            .validate()
            .map_err(crate::service::BadRequestError::new)?;
        let message_shape_matches = match mode {
            MemberStreamMode::Bootstrap => context.message_id.is_none(),
            MemberStreamMode::SubmittedTurn => context.message_id.is_some(),
        };
        if context.request_id != self.run_id || !message_shape_matches {
            return Err(crate::service::BadRequestError::new(
                "member stream acceptance context does not match runtime request".to_string(),
            )
            .into());
        }
        Ok(())
    }
}

fn start_member_runtime_stream(
    context: &MemberStreamContext,
    mode: MemberStreamMode,
    span_id: String,
) -> Result<(RuntimeEventStreamProducer, ExecuteStreamFrame)> {
    context.validate_acceptance_for_mode(mode)?;
    let mut producer = RuntimeEventStreamProducer::new();
    let start = producer.start(span_id)?;
    Ok((producer, start))
}

impl ActiveSubmittedTurn {
    fn last_signal(&self) -> Option<String> {
        self.last_signal.lock().ok().and_then(|guard| guard.clone())
    }
}

impl RetainedMemberKey {
    fn from_active(active: &ActiveMemberRuntime) -> Self {
        Self {
            orchestration_session_id: active.orchestration_session_id.clone(),
            world_generation: active.binding.world_generation,
            backend_id: active.backend_id.clone(),
        }
    }

    fn from_submit(req: &MemberTurnSubmitRequestV1) -> Self {
        Self {
            orchestration_session_id: req.orchestration_session_id.clone(),
            world_generation: req.world_generation,
            backend_id: req.backend_id.clone(),
        }
    }
}

impl RetainedMemberSlot {
    fn new(primary_participant_id: String) -> Self {
        Self {
            primary_participant_id,
            fork_child_participant_id: None,
        }
    }

    fn contains(&self, participant_id: &str) -> bool {
        self.primary_participant_id == participant_id
            || self.fork_child_participant_id.as_deref() == Some(participant_id)
    }

    fn participant_ids(&self) -> BTreeSet<String> {
        let mut participant_ids = BTreeSet::from([self.primary_participant_id.clone()]);
        if let Some(fork_child_participant_id) = self.fork_child_participant_id.as_ref() {
            participant_ids.insert(fork_child_participant_id.clone());
        }
        participant_ids
    }

    fn stale_fork_child_participant_id(
        &self,
        active_members: &HashMap<String, Arc<ActiveMemberRuntime>>,
    ) -> Option<String> {
        let fork_child_participant_id = self.fork_child_participant_id.as_ref()?;
        match active_members.get(fork_child_participant_id) {
            Some(active) if active.is_bootstrapping_or_resumable() => None,
            Some(_) | None => Some(fork_child_participant_id.clone()),
        }
    }

    fn with_registered_fork_child(
        self,
        retained_key: &RetainedMemberKey,
        active_members: &HashMap<String, Arc<ActiveMemberRuntime>>,
        candidate: &ActiveMemberRuntime,
    ) -> Result<Self> {
        if self.fork_child_participant_id.is_some() {
            return Err(
                retained_slot_registration_conflict_error(retained_key, &self, candidate).into(),
            );
        }

        let Some(parent_participant_id) = candidate.parent_participant_id.as_deref() else {
            return Err(
                retained_slot_registration_conflict_error(retained_key, &self, candidate).into(),
            );
        };
        if parent_participant_id != self.primary_participant_id {
            return Err(
                retained_slot_registration_conflict_error(retained_key, &self, candidate).into(),
            );
        }
        if candidate.resumed_from_participant_id.is_some() {
            return Err(
                retained_slot_registration_conflict_error(retained_key, &self, candidate).into(),
            );
        }

        let Some(parent) = active_members.get(parent_participant_id) else {
            return Err(
                retained_slot_registration_conflict_error(retained_key, &self, candidate).into(),
            );
        };
        // A genuine ordinary Fork may be issued by a later controller while
        // its source retains the original controller identity. Its private
        // native-history pin must join this exact registry parent and this
        // child's authenticated effect; metadata alone never grants this edge.
        // Do not revalidate the source here: that would reacquire this registry
        // while register_member holds its write lock. Native handoff performs
        // the independent current-effect revalidation before the actual spawn.
        let orchestrator_matches =
            parent.orchestrator_participant_id == candidate.orchestrator_participant_id;
        let pinned_ordinary_fork = !orchestrator_matches
            && parent.ordinary_effect.is_some()
            && candidate
                .ordinary_effect
                .as_ref()
                .zip(candidate.ordinary_native_history.as_ref())
                .is_some_and(|(effect, history)| {
                    history.fork_source.as_ref().is_some_and(|source| {
                        Arc::ptr_eq(&source.active, parent)
                            && parent
                                .ordinary_native_history
                                .as_ref()
                                .is_some_and(|history| Arc::ptr_eq(history, &source.history))
                            && source.effect.authenticated_runtime()
                                == effect.authenticated_runtime()
                    })
                });
        if parent.orchestration_session_id != candidate.orchestration_session_id
            || (!orchestrator_matches && !pinned_ordinary_fork)
            || parent.backend_id != candidate.backend_id
            || parent.binding.world_id != candidate.binding.world_id
            || parent.binding.world_generation != candidate.binding.world_generation
        {
            return Err(
                retained_slot_registration_conflict_error(retained_key, &self, candidate).into(),
            );
        }

        Ok(Self {
            primary_participant_id: self.primary_participant_id,
            fork_child_participant_id: Some(candidate.participant_id.clone()),
        })
    }

    fn without_participant(self, participant_id: &str) -> Option<Self> {
        if self.fork_child_participant_id.as_deref() == Some(participant_id) {
            return Some(Self {
                primary_participant_id: self.primary_participant_id,
                fork_child_participant_id: None,
            });
        }
        if self.primary_participant_id == participant_id {
            return self.fork_child_participant_id.map(RetainedMemberSlot::new);
        }
        Some(self)
    }
}

fn prune_stale_fork_child_slot(
    registry: &mut ActiveMemberRegistry,
    retained_key: &RetainedMemberKey,
    slot: RetainedMemberSlot,
) -> RetainedMemberSlot {
    let Some(stale_child_participant_id) =
        slot.stale_fork_child_participant_id(&registry.by_participant_id)
    else {
        return slot;
    };

    if let Some(stale_child) = registry
        .by_participant_id
        .remove(&stale_child_participant_id)
    {
        stale_child.cancel_bootstrap();
        stale_child.close_bootstrap();
        stale_child.cleanup_launcher_dir();
    }

    let pruned_slot = RetainedMemberSlot {
        primary_participant_id: slot.primary_participant_id,
        fork_child_participant_id: None,
    };
    registry
        .by_retained_key
        .insert(retained_key.clone(), pruned_slot.clone());
    pruned_slot
}

fn retained_slot_owner_mismatch_error(
    retained_key: &RetainedMemberKey,
    expected_participant_ids: &BTreeSet<String>,
    actual_participant_id: &str,
) -> crate::service::BadRequestError {
    let expected = if expected_participant_ids.len() == 1 {
        format!(
            "expected {}, got {}",
            expected_participant_ids
                .iter()
                .next()
                .expect("non-empty participant set"),
            actual_participant_id
        )
    } else {
        format!(
            "active participant_ids [{}], got {}",
            expected_participant_ids
                .iter()
                .map(String::as_str)
                .collect::<Vec<_>>()
                .join(", "),
            actual_participant_id
        )
    };
    crate::service::BadRequestError::new(format!(
        "member_turn_submit.participant_id mismatch for retained member orchestration_session_id {} world_generation {} backend_id {} ({expected})",
        retained_key.orchestration_session_id,
        retained_key.world_generation,
        retained_key.backend_id,
    ))
}

fn missing_retained_slot_error(
    retained_key: &RetainedMemberKey,
) -> crate::service::BadRequestError {
    crate::service::BadRequestError::new(format!(
        "member_turn_submit retained member is not active for orchestration_session_id {} world_generation {} backend_id {}",
        retained_key.orchestration_session_id,
        retained_key.world_generation,
        retained_key.backend_id,
    ))
}

fn retained_slot_registration_conflict_error(
    retained_key: &RetainedMemberKey,
    slot: &RetainedMemberSlot,
    candidate: &ActiveMemberRuntime,
) -> crate::service::BadRequestError {
    let active_participant_ids = slot
        .participant_ids()
        .into_iter()
        .collect::<Vec<_>>()
        .join(", ");
    crate::service::BadRequestError::new(format!(
        "member_dispatch retained member slot conflict for orchestration_session_id {} world_generation {} backend_id {} (active participant_ids [{}], requested participant_id {}, parent_participant_id {:?}, resumed_from_participant_id {:?}); only a direct fork child of the current retained participant may co-register",
        retained_key.orchestration_session_id,
        retained_key.world_generation,
        retained_key.backend_id,
        active_participant_ids,
        candidate.participant_id,
        candidate.parent_participant_id,
        candidate.resumed_from_participant_id,
    ))
}

fn should_preserve_retained_member_after_clean_bootstrap(
    active: &ActiveMemberRuntime,
    completion_status: &std::process::ExitStatus,
) -> bool {
    // Ephemeral tasks still close immediately, even if they surface a session.
    if active.participant_id.starts_with("awm_")
        || active.uaa_session_id().is_none()
        || exit_code_from_status(completion_status) != 0
        || active.initial_control_requested().unwrap_or(true)
    {
        return false;
    }
    let Some(effect) = active.ordinary_effect.as_ref() else {
        // Preserve the existing protected/legacy retained-worker behavior.
        return active.participant_id.starts_with("ash_");
    };
    let Some(activation) = active.e2_launch_activation.as_ref() else {
        return false;
    };
    let transport_api_types::OrdinaryGatewayRuntimeIdentityV1::WorldWorker {
        orchestration_session_id,
        caller_participant_id,
        participant_id,
        backend_id,
        run_id,
        world_id,
        world_generation,
    } = effect.authenticated_runtime()
    else {
        return false;
    };
    let Ok(worker_cap) = active.authenticated_worker_cap_identity.lock() else {
        return false;
    };
    // Retention parks the exact already admitted worker; it grants no new turn.
    // A subsequent Resume independently authenticates its current source/policy.
    // Do not re-open the completed native effect owner or trust an rwp_* prefix.
    matches!(
        activation.launch_kind,
        E2MemberLaunchKindV1::FreshSpawn | E2MemberLaunchKindV1::Fork
    ) && orchestration_session_id == active.orchestration_session_id
        && orchestration_session_id == active.binding.orchestration_session_id
        && orchestration_session_id == activation.orchestration_session_id
        && caller_participant_id == active.orchestrator_participant_id
        && caller_participant_id == activation.caller_participant_id
        && participant_id == active.participant_id
        && participant_id == activation.retained_participant_id
        && backend_id == active.backend_id
        && backend_id == activation.target_backend_id
        && run_id == activation.bootstrap_run_id
        && world_id == active.binding.world_id
        && world_id == activation.target_world.world_id
        && world_generation == active.binding.world_generation
        && world_generation == activation.target_world.world_generation
        && worker_cap.as_ref().is_some_and(|cap| {
            cap.commitment_ref == activation.immutable_worker_cap_ref
                && cap.created_revision == activation.immutable_worker_cap_created_revision
                && cap.application_revision == activation.immutable_worker_cap_application_revision
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[tokio::test(flavor = "current_thread")]
    async fn test_initial_control_latches_before_sdk_handle_and_delivers_once() {
        // Actual public SDK control, canceled before its startup driver is
        // polled. This proves no service source grant, World or retirement.
        let latch = OrdinaryInitialBootstrapCancellationV1::default();
        assert!(!latch.initial_control_requested().unwrap());
        assert!(
            !latch.request_initial_control("INT").unwrap(),
            "accepted pending request has no SDK handle yet"
        );
        assert_eq!(latch.initial_signal().unwrap().as_deref(), Some("INT"));
        let bridge = PromptFulfillmentBridge::for_member_backend(
            &MemberRuntimeBackendKindV1::Codex,
            "/bin/false".into(),
        )
        .unwrap();
        let control = bridge
            .run_control(AgentWrapperRunRequest {
                prompt: "SDK admission closeout fixture".into(),
                ..Default::default()
            })
            .await
            .unwrap();
        let unit = control.cancel.settlement().unwrap();
        let processes = control.cancel.process_settlement().unwrap();
        latch
            .bind_native_control(control.cancel.clone(), control.terminal.clone())
            .unwrap();
        assert!(latch.request_initial_control("TERM").unwrap());
        assert_eq!(latch.initial_signal().unwrap().as_deref(), Some("TERM"));
        assert!(
            latch
                .bind_native_control(control.cancel.clone(), control.terminal.clone())
                .is_err(),
            "one reservation cannot substitute another native control"
        );
        assert!(latch.request_initial_control("KILL").is_err());
        assert_eq!(latch.initial_signal().unwrap().as_deref(), Some("TERM"));
        tokio::time::timeout(std::time::Duration::from_secs(3), unit)
            .await
            .unwrap()
            .unwrap();
        let observed = processes.await;
        assert!(observed.acknowledged);
        assert_eq!(observed.closed_before_any_spawn, Some(true));
        assert_eq!(observed.observations, Some(Vec::new()));
        let mut events = control.handle.events;
        while events.next().await.is_some() {}
        assert!(
            matches!(control.handle.completion.await, Err(AgentWrapperError::Backend { message }) if message == "cancelled")
        );
        let terminal = control.terminal.wait().await;
        terminal.require_checked().unwrap();
        assert!(matches!(
            terminal.initial_bootstrap_process_closeout().unwrap(),
            transport_api_types::OrdinaryInitialBootstrapProcessCloseoutV1::ClosedBeforeAnySpawn {}
        ));
    }

    include!("member_runtime_native0156_tests.rs");

    // These exercise the consumed UAA adapters and generated launcher, using only
    // a test-owned regular file. They prove attachment-step composition, not
    // kernel cgroup membership or native/provider behavior.
    struct ResumeLauncherHarness {
        temp: tempfile::TempDir,
        manager: MemberRuntimeManager,
        request: MemberTurnSubmitRequestV1,
        placement: LinuxWorldPlacementContext,
    }

    async fn resume_launcher_frames(response: Response) -> Vec<serde_json::Value> {
        let bytes = tokio::time::timeout(
            std::time::Duration::from_secs(15),
            hyper::body::to_bytes(response.into_body()),
        )
        .await
        .expect("member stream must terminate")
        .expect("read member stream");
        String::from_utf8(bytes.to_vec())
            .expect("UTF-8 stream")
            .lines()
            .filter(|line| !line.is_empty())
            .map(|line| serde_json::from_str(line).expect("stream frame"))
            .collect()
    }

    async fn launch_resume_launcher_fixture(
        backend: MemberRuntimeBackendKindV1,
    ) -> ResumeLauncherHarness {
        use std::os::unix::fs::PermissionsExt;

        let temp = tempfile::tempdir().expect("fixture directory");
        let cgroup_path = temp.path().join("test-owned-cgroup-surrogate");
        fs::create_dir(&cgroup_path).expect("surrogate directory");
        fs::write(cgroup_path.join("cgroup.procs"), "").expect("surrogate target");
        let placement = LinuxWorldPlacementContext {
            working_dir: temp.path().to_path_buf(),
            cgroup_path,
            require_cgroup_attach: true,
        };
        let binary = temp.path().join("fixture-runtime.sh");
        let backend_name = match backend {
            MemberRuntimeBackendKindV1::Codex => "codex",
            MemberRuntimeBackendKindV1::ClaudeCode => "claude",
        };
        let script = r##"#!/bin/sh
set -eu
case "${1-}" in
  --version|--help|features)
    printf '%s %s\n' "$$" "$*" >> "$FIXTURE_DIR/probes"
    case "$1" in
      --version) printf 'codex 1.2.3\n' ;;
      --help) printf 'Usage: --add-dir --allow-dangerously-skip-permissions\n' ;;
      features) printf '{"features":["add_dir"]}\n' ;;
    esac
    exit 0 ;;
esac
count=0
if [ -f "$FIXTURE_DIR/count" ]; then count=$(cat "$FIXTURE_DIR/count"); fi
count=$((count + 1))
printf '%s' "$count" > "$FIXTURE_DIR/count"
printf '%s\n' "$@" > "$FIXTURE_DIR/args-$count"
attached=$(cat "$SUBSTRATE_WORLD_ENTRY_CGROUP_PROCS_PATH" 2>/dev/null || true)
printf '%s:%s\n' "$$" "$attached" >> "$FIXTURE_DIR/work"
pwd > "$FIXTURE_DIR/cwd-$count"
if [ "$FIXTURE_BACKEND" = codex ]; then
  cat > "$FIXTURE_DIR/stdin-$count"
  if [ "$count" -eq 1 ]; then event=thread.started; else event=thread.resumed; fi
  printf '{"type":"%s","thread_id":"thread-launcher-fixture"}\n' "$event"
  printf '{"type":"turn.started","thread_id":"thread-launcher-fixture","turn_id":"turn-%s"}\n' "$count"
  printf '{"type":"turn.completed","thread_id":"thread-launcher-fixture","turn_id":"turn-%s"}\n' "$count"
else
  printf '{"type":"system","subtype":"init","session_id":"thread-launcher-fixture"}\n'
  printf '{"type":"result","subtype":"success","is_error":false,"result":"fixture success","session_id":"thread-launcher-fixture","duration_ms":1,"duration_api_ms":1,"num_turns":1,"total_cost_usd":0}\n'
fi
"##;
        fs::write(&binary, script).expect("fixture executable");
        fs::set_permissions(&binary, fs::Permissions::from_mode(0o755))
            .expect("fixture permissions");
        let seed = temp.path().join("seed");
        fs::create_dir(&seed).expect("seed directory");
        fs::write(
            seed.join("auth.json"),
            r#"{"OPENAI_API_KEY":"fixture-only"}"#,
        )
        .expect("fixture auth seed");
        fs::write(seed.join("config.toml"), "model = \"gpt-5.4\"\n")
            .expect("fixture startup config");
        let mut dispatch = sample_authority_managed_dispatch();
        dispatch.participant_id = "ash_member".to_string();
        dispatch.retained_worker_launch_authority = None;
        dispatch.e2_launch_activation = None;
        dispatch.resolved_runtime.backend_kind = backend;
        dispatch.resolved_runtime.binary_path = binary.display().to_string();
        dispatch.backend_id = match backend {
            MemberRuntimeBackendKindV1::Codex => "cli:codex",
            MemberRuntimeBackendKindV1::ClaudeCode => "cli:claude_code",
        }
        .to_string();
        let mut request = sample_submit_turn_request();
        request.backend_id = dispatch.backend_id.clone();
        let env = HashMap::from([
            ("FIXTURE_DIR".to_string(), temp.path().display().to_string()),
            ("FIXTURE_BACKEND".to_string(), backend_name.to_string()),
            (
                SUBSTRATE_INTERNAL_CODEX_AUTH_SEED_HOME_ENV.to_string(),
                seed.display().to_string(),
            ),
        ]);
        let manager = MemberRuntimeManager::new();
        let response = manager
            .launch(
                "fixture-agent".to_string(),
                env,
                "spn_fixture_bootstrap".to_string(),
                MemberRuntimeLaunchAdmissionV1 {
                    ordinary_effect: None,
                    dispatch,
                    acceptance_context: None,
                    policy_snapshot: sample_e2_submit_turn_request()
                        .policy_snapshot_carrier
                        .expect("snapshot carrier")
                        .policy_snapshot()
                        .expect("snapshot"),
                },
                sample_world_binding(),
                placement.clone(),
            )
            .await
            .expect("real manager bootstrap");
        let frames = resume_launcher_frames(response).await;
        assert!(
            frames
                .iter()
                .any(|frame| frame["type"] == "exit" && frame["exit"] == 0),
            "{frames:?}"
        );
        let active = manager
            .find_submit_target(&request)
            .expect("retained member");
        assert_eq!(
            active.uaa_session_id().as_deref(),
            Some("thread-launcher-fixture")
        );
        assert!(active.launcher_dir.join("world-entry.sh").is_file());
        assert_eq!(fs::read_to_string(temp.path().join("count")).unwrap(), "1");
        ResumeLauncherHarness {
            temp,
            manager,
            request,
            placement,
        }
    }

    async fn assert_sequential_resume_launcher_attachment(backend: MemberRuntimeBackendKindV1) {
        let mut harness = launch_resume_launcher_fixture(backend).await;
        for index in 2..=3 {
            fs::write(harness.placement.cgroup_procs_path(), "").expect("reset surrogate");
            harness.request.run_id = format!("run_fixture_{index}");
            harness.request.prompt = format!("fixture prompt {index}");
            let frames = resume_launcher_frames(
                harness
                    .manager
                    .submit_turn(harness.request.clone())
                    .await
                    .expect("resume"),
            )
            .await;
            assert!(
                frames
                    .iter()
                    .any(|frame| frame["type"] == "exit" && frame["exit"] == 0),
                "{frames:?}"
            );
            let args =
                fs::read_to_string(harness.temp.path().join(format!("args-{index}"))).unwrap();
            let args = args.lines().collect::<Vec<_>>();
            let resume_flag = if backend == MemberRuntimeBackendKindV1::Codex {
                "resume"
            } else {
                "--resume"
            };
            assert!(
                args.windows(2)
                    .any(|pair| pair == [resume_flag, "thread-launcher-fixture"]),
                "{args:?}"
            );
            assert!(!args
                .iter()
                .any(|arg| matches!(*arg, "--fork-session" | "--last")));
            let prompt = if backend == MemberRuntimeBackendKindV1::Codex {
                fs::read_to_string(harness.temp.path().join(format!("stdin-{index}"))).unwrap()
            } else {
                args.last().unwrap().to_string()
            };
            assert!(prompt.contains(&harness.request.prompt), "{prompt:?}");
            assert_eq!(
                fs::read_to_string(harness.temp.path().join(format!("cwd-{index}")))
                    .unwrap()
                    .trim(),
                harness.placement.working_dir.to_str().unwrap()
            );
            assert_eq!(
                harness
                    .manager
                    .find_submit_target(&harness.request)
                    .unwrap()
                    .uaa_session_id()
                    .as_deref(),
                Some("thread-launcher-fixture")
            );
        }
        let work = fs::read_to_string(harness.temp.path().join("work")).unwrap();
        let rows = work
            .lines()
            .map(|line| line.split_once(':').unwrap())
            .collect::<Vec<_>>();
        assert_eq!(rows.len(), 3, "bootstrap and exactly two resumes: {work}");
        let mut pids = std::collections::HashSet::new();
        for (pid, attached) in rows {
            assert!(
                pids.insert(pid),
                "actual work PIDs must be distinct: {work}"
            );
            assert_eq!(
                pid, attached,
                "actual work must follow launcher attachment, independently of probes: {work}"
            );
        }
        eprintln!("{backend:?} actual-work pid:observed-surrogate-pid (composition only): {work}");
        if let Ok(probes) = fs::read_to_string(harness.temp.path().join("probes")) {
            eprintln!("{backend:?} separate capability probes: {probes}");
            for probe in probes.lines() {
                assert!(!pids.contains(probe.split_whitespace().next().unwrap()));
            }
        }
        harness
            .manager
            .unregister_member(&harness.request.participant_id);
    }

    async fn assert_missing_resume_launcher_target_rejects(backend: MemberRuntimeBackendKindV1) {
        let mut harness = launch_resume_launcher_fixture(backend).await;
        let before = fs::read_to_string(harness.temp.path().join("work")).unwrap();
        // This is exclusively our temporary regular file, never a kernel control file.
        fs::remove_file(harness.placement.cgroup_procs_path()).expect("remove surrogate");
        harness.request.run_id = "run_fixture_missing_target".to_string();
        match harness.manager.submit_turn(harness.request.clone()).await {
            Ok(response) => {
                let frames = resume_launcher_frames(response).await;
                assert!(
                    !frames
                        .iter()
                        .any(|frame| frame["type"] == "exit" && frame["exit"] == 0),
                    "missing required target must reject: {frames:?}"
                );
            }
            Err(error) => eprintln!("{backend:?} rejected before stream: {error:#}"),
        }
        assert_eq!(
            fs::read_to_string(harness.temp.path().join("work")).unwrap(),
            before,
            "missing attachment target must reject before actual work"
        );
        assert!(
            !harness.placement.cgroup_procs_path().exists(),
            "launcher must not recreate target"
        );
        assert_eq!(
            harness
                .manager
                .find_submit_target(&harness.request)
                .unwrap()
                .uaa_session_id()
                .as_deref(),
            Some("thread-launcher-fixture")
        );
        fs::write(harness.placement.cgroup_procs_path(), "").expect("restore surrogate");
        harness.request.run_id = "run_fixture_recovery".to_string();
        let frames = resume_launcher_frames(
            harness
                .manager
                .submit_turn(harness.request.clone())
                .await
                .expect("recovery resume"),
        )
        .await;
        assert!(
            frames
                .iter()
                .any(|frame| frame["type"] == "exit" && frame["exit"] == 0),
            "{frames:?}"
        );
        assert_eq!(
            fs::read_to_string(harness.temp.path().join("count")).unwrap(),
            "2",
            "one bootstrap, no rejected work, exactly one recovery"
        );
        let args = fs::read_to_string(harness.temp.path().join("args-2")).unwrap();
        assert!(args.contains("thread-launcher-fixture"), "{args}");
        harness
            .manager
            .unregister_member(&harness.request.participant_id);
    }

    #[tokio::test]
    async fn codex_resume_launcher_attaches_bootstrap_and_two_same_session_turns() {
        assert_sequential_resume_launcher_attachment(MemberRuntimeBackendKindV1::Codex).await;
    }

    #[tokio::test]
    async fn claude_resume_launcher_attaches_bootstrap_and_two_same_session_turns() {
        assert_sequential_resume_launcher_attachment(MemberRuntimeBackendKindV1::ClaudeCode).await;
    }

    #[tokio::test]
    async fn codex_resume_launcher_missing_target_rejects_before_work() {
        assert_missing_resume_launcher_target_rejects(MemberRuntimeBackendKindV1::Codex).await;
    }

    #[tokio::test]
    async fn claude_resume_launcher_missing_target_rejects_before_work() {
        assert_missing_resume_launcher_target_rejects(MemberRuntimeBackendKindV1::ClaudeCode).await;
    }

    fn sample_e2_launch_activation(
        participant_id: &str,
        bootstrap_run_id: &str,
    ) -> E2MemberLaunchActivationCarrierV1 {
        use base64::Engine as _;
        use sha2::Digest as _;

        let snapshot = sample_e2_submit_turn_request()
            .policy_snapshot_carrier
            .expect("E2 turn carrier")
            .policy_snapshot()
            .expect("E2 launch snapshot");
        let bytes = serde_json::to_vec(&snapshot).expect("exact E1 launch snapshot bytes");
        let commitment_ref = transport_api_types::DispatchPolicyCommitmentRefCarrierV1 {
            authority_store_id: "authority-store-e2-world-service".to_string(),
            commitment_id: "dpc_018f0f3a-9b2c-7def-8abc-0123456789ad".to_string(),
            exact_linkage_hash: "a".repeat(64),
        };
        let carrier = E2MemberLaunchActivationCarrierV1 {
            schema_version: 1,
            activation_id: format!("e2a_{}", "a".repeat(32)),
            launch_kind: E2MemberLaunchKindV1::FreshSpawn,
            reservation_ref: Some(
                transport_api_types::E2DispatchPolicyReservationRefCarrierV1 {
                    authority_store_id: commitment_ref.authority_store_id.clone(),
                    reservation_id: "dpr_world_service_fixture".to_string(),
                    reservation_hash: "c".repeat(64),
                },
            ),
            commitment_ref: commitment_ref.clone(),
            immutable_worker_cap_ref: commitment_ref,
            immutable_worker_cap_created_revision: 7,
            immutable_worker_cap_application_revision: 9,
            policy_snapshot_bytes_base64: base64::engine::general_purpose::STANDARD.encode(&bytes),
            policy_snapshot_byte_length: bytes.len() as u64,
            policy_snapshot_ref: transport_api_types::AuthorityObjectRefV1 {
                ref_id: "ao_0123456789abcdef0123456789abcdef".to_string(),
                object_kind: transport_api_types::AuthorityObjectKindV1::Policy,
                schema_version: 1,
                commitment: transport_api_types::OpaqueAuthorityCommitmentV1::CanonicalSha256 {
                    digest_hex: "b".repeat(64),
                },
            },
            policy_snapshot_hash: format!("{:x}", sha2::Sha256::digest(&bytes)),
            policy_snapshot_revision: "policy-revision-e2-world-service".to_string(),
            reason: Some("authenticated retained launch cap".to_string()),
            request_id: "req_fixture".to_string(),
            idempotency_key: "idem_fixture".to_string(),
            orchestration_session_id: "orch_123".to_string(),
            caller_participant_id: "ash_orchestrator".to_string(),
            caller_backend_id: "cli:codex".to_string(),
            target_backend_id: "cli:codex".to_string(),
            retained_participant_id: participant_id.to_string(),
            bootstrap_run_id: bootstrap_run_id.to_string(),
            source_participant_id: None,
            target_world: transport_api_types::WorldBindingRefV1 {
                world_id: "world_123".to_string(),
                world_generation: 7,
            },
            parent_policy_ref: transport_api_types::AuthorityObjectRefV1 {
                ref_id: "ao_fedcba9876543210fedcba9876543210".to_string(),
                object_kind: transport_api_types::AuthorityObjectKindV1::Policy,
                schema_version: 1,
                commitment: transport_api_types::OpaqueAuthorityCommitmentV1::CanonicalSha256 {
                    digest_hex: "d".repeat(64),
                },
            },
            parent_policy_revision: "parent-policy-revision".to_string(),
            request_commitment: transport_api_types::E2LaunchRequestCommitmentV1::HmacSha256 {
                key_id: "dpk_world_service_fixture".to_string(),
                domain: "substrate.dispatch_policy_commitment.spawn_request.v1".to_string(),
                digest_hex: "e".repeat(64),
            },
            registry_publication_revision: 9,
        };
        carrier.validate().expect("valid E2 launch activation");
        carrier
    }

    fn sample_authority_managed_dispatch() -> MemberDispatchRequestV1 {
        use transport_api_types::{
            ResolvedMemberRuntimeDescriptorV1, RetainedWorkerAdmissionCommitmentCarrierV1,
            RetainedWorkerAuthorityObjectCommitmentV1, RetainedWorkerLaunchAuthorityProofV1,
            RetainedWorkerLaunchWorldBindingV1,
        };
        let commitment = |value: char| RetainedWorkerAuthorityObjectCommitmentV1::CanonicalSha256 {
            digest_hex: value.to_string().repeat(64),
        };
        let e2_launch_activation = sample_e2_launch_activation("rwp_member", "rwr_bootstrap");
        MemberDispatchRequestV1 {
            schema_version: 1,
            orchestration_session_id: "orch_123".to_string(),
            participant_id: "rwp_member".to_string(),
            orchestrator_participant_id: "ash_orchestrator".to_string(),
            parent_participant_id: None,
            resumed_from_participant_id: None,
            backend_id: "cli:codex".to_string(),
            protocol: "substrate.agent.session".to_string(),
            run_id: "rwr_bootstrap".to_string(),
            world_id: "world_123".to_string(),
            world_generation: 7,
            initial_prompt: Some("start".to_string()),
            resolved_runtime: ResolvedMemberRuntimeDescriptorV1 {
                backend_kind: MemberRuntimeBackendKindV1::Codex,
                binary_path: "/bin/true".to_string(),
            },
            retained_worker_launch_authority: Some(RetainedWorkerLaunchAuthorityProofV1 {
                schema_version: 1,
                authority_store_id: "has_fixture".to_string(),
                issuer_request_id: "req_fixture".to_string(),
                canonical_spawn_fingerprint: RetainedWorkerAdmissionCommitmentCarrierV1 {
                    schema_version: 1,
                    algorithm: "hmac-sha-256".to_string(),
                    key_id: "adk_fixture".to_string(),
                    digest_hex: "a".repeat(64),
                },
                registration_id: "rwr_registration".to_string(),
                registration_commitment: commitment('b'),
                authority_revision_after: 2,
                authority_record_commitment_after: commitment('c'),
                orchestration_session_id: "orch_123".to_string(),
                caller_participant_id: "ash_orchestrator".to_string(),
                retained_participant_id: "rwp_member".to_string(),
                bootstrap_run_id: "rwr_bootstrap".to_string(),
                transport_claim_id: "rtc_claim".to_string(),
                backend_id: "cli:codex".to_string(),
                protocol: "substrate.agent.session".to_string(),
                world_binding: RetainedWorkerLaunchWorldBindingV1 {
                    world_id: "world_123".to_string(),
                    world_generation: 7,
                },
                current_policy_ref_id: "ao_policy".to_string(),
                current_policy_revision: "policy-1".to_string(),
                retained_worker_ref_id: "ao_worker".to_string(),
                retained_worker_commitment: commitment('d'),
            }),
            e2_launch_activation: Some(e2_launch_activation),
        }
    }

    #[test]
    fn retained_launch_authority_requires_exact_dispatch_and_world_binding() {
        let binding = sample_world_binding();
        let request = sample_authority_managed_dispatch();
        validate_retained_worker_launch_authority_proof(&request, &binding).expect("exact proof");

        let mut missing = request.clone();
        missing.retained_worker_launch_authority = None;
        let error = validate_retained_worker_launch_authority_proof(&missing, &binding)
            .expect_err("managed identity must require proof");
        assert!(error
            .to_string()
            .contains("requires retained_worker_launch_authority"));

        let mut legacy = request.clone();
        legacy.participant_id = "ash_pre_activation_member".to_string();
        legacy.run_id = "run_pre_activation_bootstrap".to_string();
        legacy.retained_worker_launch_authority = None;
        legacy.e2_launch_activation = None;
        validate_retained_worker_launch_authority_proof(&legacy, &binding)
            .expect("explicit pre-activation compatibility identity may omit proof");

        for changed in [
            "session",
            "participant",
            "caller",
            "backend",
            "protocol",
            "run",
            "world",
            "generation",
        ] {
            let mut conflict = request.clone();
            match changed {
                "session" => conflict.orchestration_session_id.push_str("-changed"),
                "participant" => conflict.participant_id.push_str("-changed"),
                "caller" => conflict.orchestrator_participant_id.push_str("-changed"),
                "backend" => conflict.backend_id.push_str("-changed"),
                "protocol" => conflict.protocol.push_str("-changed"),
                "run" => conflict.run_id.push_str("-changed"),
                "world" => conflict.world_id.push_str("-changed"),
                "generation" => conflict.world_generation += 1,
                _ => unreachable!(),
            }
            assert!(
                validate_retained_worker_launch_authority_proof(&conflict, &binding).is_err(),
                "{changed} mismatch must fail closed"
            );
        }
    }

    #[tokio::test]
    async fn authority_managed_launch_rejects_missing_malformed_or_mismatched_proof_before_binary()
    {
        let temp = tempfile::tempdir().expect("member launch placement");
        let placement = LinuxWorldPlacementContext {
            working_dir: temp.path().to_path_buf(),
            cgroup_path: temp.path().join("unused-cgroup"),
            require_cgroup_attach: false,
        };
        let binding = sample_world_binding();

        let mut missing = sample_authority_managed_dispatch();
        missing.resolved_runtime.binary_path =
            temp.path().join("missing-runtime").display().to_string();
        missing.retained_worker_launch_authority = None;
        let missing_error = match MemberRuntimeManager::new()
            .launch(
                "ambient-shell".to_string(),
                HashMap::new(),
                "spn_missing_proof".to_string(),
                MemberRuntimeLaunchAdmissionV1 {
                    ordinary_effect: None,
                    policy_snapshot: missing
                        .e2_launch_activation
                        .as_ref()
                        .expect("fixture activation")
                        .policy_snapshot()
                        .expect("fixture launch snapshot"),
                    dispatch: missing,
                    acceptance_context: None,
                },
                binding.clone(),
                placement.clone(),
            )
            .await
        {
            Ok(_) => panic!("missing authority-managed proof must fail before launch"),
            Err(error) => error,
        };
        assert!(
            missing_error
                .to_string()
                .contains("requires retained_worker_launch_authority"),
            "missing proof must win over missing binary: {missing_error:#}"
        );

        let mut malformed = sample_authority_managed_dispatch();
        malformed.resolved_runtime.binary_path =
            temp.path().join("missing-runtime").display().to_string();
        malformed
            .retained_worker_launch_authority
            .as_mut()
            .expect("fixture proof")
            .schema_version = 2;
        let malformed_error = match MemberRuntimeManager::new()
            .launch(
                "ambient-shell".to_string(),
                HashMap::new(),
                "spn_malformed_proof".to_string(),
                MemberRuntimeLaunchAdmissionV1 {
                    ordinary_effect: None,
                    policy_snapshot: malformed
                        .e2_launch_activation
                        .as_ref()
                        .expect("fixture activation")
                        .policy_snapshot()
                        .expect("fixture launch snapshot"),
                    dispatch: malformed,
                    acceptance_context: None,
                },
                binding.clone(),
                placement.clone(),
            )
            .await
        {
            Ok(_) => panic!("malformed authority-managed proof must fail before launch"),
            Err(error) => error,
        };
        assert!(
            malformed_error
                .to_string()
                .contains("unsupported retained_worker_launch_authority.schema_version"),
            "malformed proof must win over missing binary: {malformed_error:#}"
        );

        let mut mismatched = sample_authority_managed_dispatch();
        mismatched.resolved_runtime.binary_path =
            temp.path().join("missing-runtime").display().to_string();
        mismatched
            .retained_worker_launch_authority
            .as_mut()
            .expect("fixture proof")
            .backend_id = "cli:other".to_string();
        let mismatched_error = match MemberRuntimeManager::new()
            .launch(
                "ambient-shell".to_string(),
                HashMap::new(),
                "spn_mismatched_proof".to_string(),
                MemberRuntimeLaunchAdmissionV1 {
                    ordinary_effect: None,
                    policy_snapshot: mismatched
                        .e2_launch_activation
                        .as_ref()
                        .expect("fixture activation")
                        .policy_snapshot()
                        .expect("fixture launch snapshot"),
                    dispatch: mismatched,
                    acceptance_context: None,
                },
                binding,
                placement,
            )
            .await
        {
            Ok(_) => panic!("mismatched authority-managed proof must fail before launch"),
            Err(error) => error,
        };
        assert!(
            mismatched_error
                .to_string()
                .contains("conflicts with member dispatch or exact world binding"),
            "mismatched proof must win over missing binary: {mismatched_error:#}"
        );
    }

    #[tokio::test]
    async fn e2_launch_activation_must_exact_join_the_enforced_outer_snapshot_before_binary() {
        let temp = tempfile::tempdir().expect("member launch placement");
        let placement = LinuxWorldPlacementContext {
            working_dir: temp.path().to_path_buf(),
            cgroup_path: temp.path().join("unused-cgroup"),
            require_cgroup_attach: false,
        };
        let binding = sample_world_binding();
        let mut dispatch = sample_authority_managed_dispatch();
        dispatch.resolved_runtime.binary_path =
            temp.path().join("missing-runtime").display().to_string();
        let mut substituted_snapshot = dispatch
            .e2_launch_activation
            .as_ref()
            .expect("fixture activation")
            .policy_snapshot()
            .expect("fixture launch snapshot");
        substituted_snapshot.world_fs.fail_closed.routing =
            !substituted_snapshot.world_fs.fail_closed.routing;

        let error = match MemberRuntimeManager::new()
            .launch(
                "ambient-shell".to_string(),
                HashMap::new(),
                "spn_substituted_launch_snapshot".to_string(),
                MemberRuntimeLaunchAdmissionV1 {
                    ordinary_effect: None,
                    dispatch,
                    acceptance_context: None,
                    policy_snapshot: substituted_snapshot,
                },
                binding,
                placement,
            )
            .await
        {
            Ok(_) => panic!("substituted outer snapshot must fail before launch"),
            Err(error) => error,
        };
        assert!(
            error
                .to_string()
                .contains("E2 launch activation snapshot conflicts with enforced policy snapshot"),
            "snapshot mismatch must win over missing binary: {error:#}"
        );
    }

    #[test]
    fn e2_launch_activation_pins_caller_backend_from_acceptance_context() {
        let dispatch = sample_authority_managed_dispatch();
        let snapshot = dispatch
            .e2_launch_activation
            .as_ref()
            .expect("fixture activation")
            .policy_snapshot()
            .expect("fixture launch snapshot");
        let acceptance = transport_api_types::WorldWorkAcceptanceContextV1 {
            schema_version: 1,
            proposed_acceptance_record_id: "wwa_018f0f3a-9b2c-7def-8abc-0123456789ab".to_string(),
            request_id: dispatch.run_id.clone(),
            message_id: None,
            caller_backend_id: "cli:claude-code".to_string(),
            host_transition_correlation: None,
        };
        acceptance.validate().expect("valid acceptance context");

        let error = validate_e2_member_launch_activation(
            &dispatch,
            &sample_world_binding(),
            &snapshot,
            Some(&acceptance),
        )
        .expect_err("launch caller backend substitution must fail closed");
        assert!(
            error.to_string().contains("E2 launch activation conflicts"),
            "unexpected error: {error:#}"
        );
    }

    fn sample_submit_turn_request() -> MemberTurnSubmitRequestV1 {
        MemberTurnSubmitRequestV1 {
            schema_version: 1,
            orchestration_session_id: "orch_123".to_string(),
            participant_id: "ash_member".to_string(),
            orchestrator_participant_id: "ash_orchestrator".to_string(),
            backend_id: "cli:codex".to_string(),
            run_id: "run_turn".to_string(),
            world_id: "world_123".to_string(),
            world_generation: 7,
            prompt: "continue".to_string(),
            policy_snapshot_carrier: None,
            acceptance_context: None,
        }
    }

    fn sample_e2_submit_turn_request() -> MemberTurnSubmitRequestV1 {
        use base64::Engine as _;
        use sha2::Digest as _;

        let mut request = sample_submit_turn_request();
        let snapshot = transport_api_types::PolicySnapshotV3 {
            schema_version: 3,
            net_allowed: Vec::new(),
            world_fs: transport_api_types::PolicySnapshotWorldFsV3 {
                host_visible: false,
                fail_closed: transport_api_types::PolicySnapshotWorldFsFailClosedV3 {
                    routing: true,
                },
                deny_enforcement: Some(transport_api_types::WorldFsDenyEnforcementV3::Strict),
                caged_required: true,
                discover: Some(transport_api_types::PolicySnapshotWorldFsDimensionV3 {
                    allow_list: vec!["workspace/exact.txt".to_string()],
                    deny_list: Vec::new(),
                }),
                read: Some(transport_api_types::PolicySnapshotWorldFsDimensionV3 {
                    allow_list: vec!["workspace/exact.txt".to_string()],
                    deny_list: Vec::new(),
                }),
                write: transport_api_types::PolicySnapshotWorldFsWriteV3 {
                    enabled: false,
                    allow_list: vec!["workspace/exact.txt".to_string()],
                    deny_list: Vec::new(),
                },
            },
        }
        .canonicalize()
        .expect("canonical E2 WorldService fixture snapshot");
        let bytes = serde_json::to_vec(&snapshot).expect("exact E1 snapshot bytes");
        let carrier = transport_api_types::DispatchPolicySnapshotCarrierV1 {
            schema_version: 1,
            immutable_worker_cap_ref: transport_api_types::DispatchPolicyCommitmentRefCarrierV1 {
                authority_store_id: "authority-store-e2-world-service".to_string(),
                commitment_id: "dpc_018f0f3a-9b2c-7def-8abc-0123456789ad".to_string(),
                exact_linkage_hash: "a".repeat(64),
            },
            immutable_worker_cap_created_revision: 7,
            immutable_worker_cap_application_revision: 9,
            subject: transport_api_types::RetainedTurnPolicyCommitmentSubjectV1 {
                retained_participant_id: request.participant_id.clone(),
                active_run_id: request.run_id.clone(),
                message_id: None,
            },
            orchestration_session_id: request.orchestration_session_id.clone(),
            caller_participant_id: request.orchestrator_participant_id.clone(),
            caller_backend_id: "cli:codex".to_string(),
            target_backend_id: request.backend_id.clone(),
            target_world: transport_api_types::WorldBindingRefV1 {
                world_id: request.world_id.clone(),
                world_generation: request.world_generation,
            },
            policy_snapshot_bytes_base64: base64::engine::general_purpose::STANDARD.encode(&bytes),
            policy_snapshot_byte_length: bytes.len() as u64,
            policy_snapshot_ref: transport_api_types::AuthorityObjectRefV1 {
                ref_id: "ao_0123456789abcdef0123456789abcdef".to_string(),
                object_kind: transport_api_types::AuthorityObjectKindV1::Policy,
                schema_version: 1,
                commitment: transport_api_types::OpaqueAuthorityCommitmentV1::CanonicalSha256 {
                    digest_hex: "b".repeat(64),
                },
            },
            policy_snapshot_hash: format!("{:x}", sha2::Sha256::digest(&bytes)),
            policy_snapshot_revision: "policy-revision-e2-world-service".to_string(),
            reason: Some("authenticated retained-turn narrowing".to_string()),
        };
        carrier.validate().expect("valid E2 WorldService carrier");
        request.policy_snapshot_carrier = Some(carrier);
        request
    }

    fn sample_member_stream_context(message_id: Option<&str>) -> MemberStreamContext {
        MemberStreamContext {
            orchestration_session_id: "orch_123".to_string(),
            run_id: "run_turn".to_string(),
            acceptance_context: Some(transport_api_types::WorldWorkAcceptanceContextV1 {
                schema_version: 1,
                proposed_acceptance_record_id: "wwa_018f0f2e-7b4c-7aa1-8c22-123456789abc"
                    .to_string(),
                request_id: "run_turn".to_string(),
                message_id: message_id.map(str::to_string),
                caller_backend_id: "cli:codex".to_string(),
                host_transition_correlation: None,
            }),
            participant_id: "ash_member".to_string(),
            orchestrator_participant_id: "ash_orchestrator".to_string(),
            parent_participant_id: None,
            resumed_from_participant_id: None,
            backend_id: "cli:codex".to_string(),
            protocol: json!("substrate.agent.session"),
            authenticated_thread_id: None,
        }
    }

    #[test]
    fn task_acceptance_context_is_retained_before_first_start_acknowledgement() {
        let context = sample_member_stream_context(None);
        let retained = context.acceptance_context.clone();
        let (_, start) = start_member_runtime_stream(
            &context,
            MemberStreamMode::Bootstrap,
            "spn_task_b1".to_string(),
        )
        .expect("start task stream after retaining exact context");
        assert_eq!(context.acceptance_context, retained);
        let ExecuteStreamFrame::Start {
            frame_identity,
            span_id,
        } = start
        else {
            panic!("first task acknowledgement must be Start");
        };
        assert_eq!(span_id, "spn_task_b1");
        assert_eq!(frame_identity.frame_sequence, 1);
    }

    #[test]
    fn retained_acceptance_context_requires_message_and_exact_run_before_start() {
        let context =
            sample_member_stream_context(Some("wwm_018f0f2e-7b4c-7aa1-8c22-123456789abd"));
        let retained = context.acceptance_context.clone();
        let (_, start) = start_member_runtime_stream(
            &context,
            MemberStreamMode::SubmittedTurn,
            "spn_turn_b1".to_string(),
        )
        .expect("start retained stream after retaining exact context");
        assert_eq!(context.acceptance_context, retained);
        assert!(matches!(
            start,
            ExecuteStreamFrame::Start {
                frame_identity,
                span_id,
            } if frame_identity.frame_sequence == 1 && span_id == "spn_turn_b1"
        ));

        let mut wrong_run = context.clone();
        wrong_run.run_id = "substituted-run".to_string();
        assert!(start_member_runtime_stream(
            &wrong_run,
            MemberStreamMode::SubmittedTurn,
            "spn_must_not_start".to_string(),
        )
        .is_err());
        assert!(start_member_runtime_stream(
            &sample_member_stream_context(None),
            MemberStreamMode::SubmittedTurn,
            "spn_must_not_start".to_string(),
        )
        .is_err());
    }

    fn sample_retained_identity() -> RetainedMemberIdentity<'static> {
        RetainedMemberIdentity {
            orchestration_session_id: "orch_123",
            orchestrator_participant_id: "ash_orchestrator",
            backend_id: "cli:codex",
            world_id: "world_123",
            world_generation: 7,
        }
    }

    #[test]
    fn test_retired_member_clone_cannot_reserve_another_turn() {
        // Registry/slot seam only: no native/service retirement is fabricated.
        let manager = MemberRuntimeManager::new();
        let root = tempfile::tempdir().unwrap();
        let original = sample_active_member_runtime(&root, "slot-owner", "bootstrap-original");
        original.close_bootstrap();
        manager.register_member(Arc::clone(&original)).unwrap();
        manager
            .reserve_turn_slot(&original, "before-removal")
            .unwrap();
        manager.clear_reserved_turn_slot(&original, "before-removal");
        manager.unregister_member("slot-owner");
        assert!(manager.reserve_turn_slot(&original, "stale-clone").is_err());
        assert!(original.active_turn_span_id.lock().unwrap().is_none());
        let replacement_root = tempfile::tempdir().unwrap();
        let replacement =
            sample_active_member_runtime(&replacement_root, "slot-owner", "bootstrap-replacement");
        replacement.close_bootstrap();
        manager.register_member(Arc::clone(&replacement)).unwrap();
        assert!(
            manager.reserve_turn_slot(&original, "stale-clone").is_err(),
            "same text ID cannot substitute for actual registered member"
        );
        manager
            .reserve_turn_slot(&replacement, "new-owner")
            .unwrap();
        manager
            .reserve_turn_slot(&replacement, "new-owner")
            .unwrap();
        assert!(manager
            .reserve_turn_slot(&replacement, "different-turn")
            .is_err());
        manager.clear_reserved_turn_slot(&replacement, "new-owner");
        manager.unregister_member("slot-owner");
    }

    fn sample_active_member_runtime(
        temp_dir: &tempfile::TempDir,
        participant_id: &str,
        bootstrap_span_id: &str,
    ) -> Arc<ActiveMemberRuntime> {
        sample_active_member_runtime_with_lineage(
            temp_dir,
            participant_id,
            bootstrap_span_id,
            None,
            None,
        )
    }

    fn sample_active_member_runtime_with_lineage(
        temp_dir: &tempfile::TempDir,
        participant_id: &str,
        bootstrap_span_id: &str,
        parent_participant_id: Option<&str>,
        resumed_from_participant_id: Option<&str>,
    ) -> Arc<ActiveMemberRuntime> {
        let workspace_dir = temp_dir.path().join("workspace");
        let process_working_dir = temp_dir.path().join("process");
        let binary_path = temp_dir.path().join("member-runtime");
        let launcher_dir = temp_dir.path().join("launcher");
        let codex_home = launcher_dir.join("codex-home");
        fs::create_dir_all(&workspace_dir).expect("create workspace dir");
        fs::create_dir_all(&process_working_dir).expect("create process dir");
        fs::create_dir_all(&launcher_dir).expect("create launcher dir");
        fs::create_dir_all(&codex_home).expect("create codex home");
        fs::write(&binary_path, "#!/bin/sh\nexit 0\n").expect("write binary");
        let exclusion =
            E3PrivilegedChildExclusionV1::new_recovering().expect("bind fixture child exclusion");
        exclusion
            .finish_recovery()
            .expect("finish fixture child-exclusion recovery");
        let privileged_child_exclusion_lease = Arc::new(
            exclusion
                .acquire_non_e3_child()
                .expect("acquire fixture child lease"),
        );

        Arc::new(ActiveMemberRuntime {
            agent_id: "codex_world".to_string(),
            participant_id: participant_id.to_string(),
            orchestration_session_id: "orch_123".to_string(),
            orchestrator_participant_id: "ash_orchestrator".to_string(),
            parent_participant_id: parent_participant_id.map(str::to_string),
            resumed_from_participant_id: resumed_from_participant_id.map(str::to_string),
            backend_id: "cli:codex".to_string(),
            backend_kind: MemberRuntimeBackendKindV1::Codex,
            binary_path,
            launcher_dir: launcher_dir.clone(),
            workspace_dir,
            process_working_dir,
            env: BTreeMap::from([("CODEX_HOME".to_string(), codex_home.display().to_string())]),
            ordinary_effect: None,
            ordinary_native_home: Mutex::new(None),
            ordinary_native_history: None,
            binding: sample_world_binding(),
            protocol: json!("substrate.agent.session"),
            bootstrap: Mutex::new(Some(ActiveBootstrapRuntime {
                span_id: bootstrap_span_id.to_string(),
                terminal: None,
                cancel: None,
                last_signal: None,
                initial_control: None,
            })),
            active_turn_span_id: Mutex::new(None),
            uaa_session_id: Mutex::new(None),
            e2_launch_activation: None,
            authenticated_worker_cap_identity: Mutex::new(None),
            privileged_child_exclusion_lease,
        })
    }

    fn sample_e2_active_member_runtime(
        temp_dir: &tempfile::TempDir,
        participant_id: &str,
        bootstrap_span_id: &str,
    ) -> Arc<ActiveMemberRuntime> {
        let mut active = sample_active_member_runtime(temp_dir, participant_id, bootstrap_span_id);
        let activation = sample_e2_launch_activation(participant_id, "rwr_bootstrap");
        let cap_identity = RetainedTurnWorkerCapIdentity {
            commitment_ref: activation.immutable_worker_cap_ref.clone(),
            created_revision: activation.immutable_worker_cap_created_revision,
            application_revision: activation.immutable_worker_cap_application_revision,
        };
        let active_mut = Arc::get_mut(&mut active).expect("unique active E2 fixture");
        active_mut.e2_launch_activation = Some(activation);
        active_mut.authenticated_worker_cap_identity = Mutex::new(Some(cap_identity));
        active
    }

    #[test]
    fn prepare_codex_runtime_env_seeds_isolated_home_and_removes_internal_seed_env() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let seed_home = temp_dir.path().join("seed-home");
        let workspace_dir = temp_dir.path().join("workspace");
        fs::create_dir_all(&seed_home).expect("create seed home");
        fs::create_dir_all(&workspace_dir).expect("create workspace dir");
        fs::write(
            seed_home.join("auth.json"),
            r#"{"account_id":"acct_test","access_token":"token_test"}"#,
        )
        .expect("write auth");
        fs::write(seed_home.join("config.toml"), "model = \"gpt-5.4\"\n")
            .expect("write startup config");
        fs::write(seed_home.join(".credentials.json"), "{}").expect("write credentials");
        let launcher_dir = temp_dir.path().join("launcher");
        fs::create_dir_all(&launcher_dir).expect("create launcher dir");
        let expected_codex_home = launcher_dir.join("codex-home").display().to_string();

        let mut runtime_env = BTreeMap::from([(
            SUBSTRATE_INTERNAL_CODEX_AUTH_SEED_HOME_ENV.to_string(),
            seed_home.display().to_string(),
        )]);

        prepare_codex_runtime_env(&mut runtime_env, &workspace_dir, &launcher_dir)
            .expect("seed auth");

        let codex_home = launcher_dir.join("codex-home");
        assert_eq!(
            runtime_env.get("CODEX_HOME").map(String::as_str),
            Some(expected_codex_home.as_str())
        );
        assert!(
            !runtime_env.contains_key(SUBSTRATE_INTERNAL_CODEX_AUTH_SEED_HOME_ENV),
            "internal seed env must be removed before spawning the member runtime"
        );
        assert_eq!(
            fs::read_to_string(codex_home.join("auth.json")).expect("seeded auth"),
            r#"{"account_id":"acct_test","access_token":"token_test"}"#
        );
        assert_eq!(
            fs::read_to_string(codex_home.join("config.toml")).expect("rendered config"),
            format!(
                "model = \"gpt-5.4\"\n\n[projects.\"{}\"]\ntrust_level = \"untrusted\"\n",
                workspace_dir.display()
            )
        );
        assert!(
            codex_home.join(".credentials.json").is_file(),
            "optional credentials file should be copied when present"
        );
    }

    #[test]
    fn codex_startup_selected_model_provider_gets_derived_display_name() {
        let seed = tempfile::tempdir().unwrap();
        fs::write(
            seed.path().join("config.toml"),
            r#"
model = "gpt-6-astra"
model_provider = " selected-provider "
provider = "legacy-provider"
[model_providers.selected-provider]
base_url = "http://127.0.0.1:4123/v1"
[model_providers.unselected]
base_url = "https://unselected.invalid"
name = "must not copy"
[providers.legacy-provider]
base_url = "https://legacy.invalid/v1"
"#,
        )
        .unwrap();
        let subset =
            read_codex_startup_subset(seed.path(), Path::new("/bounded-workspace")).unwrap();
        let rendered = toml::to_string(&subset).unwrap();
        let actual: toml::Value = toml::from_str(&rendered).unwrap();
        let selected = actual["model_providers"].as_table().unwrap();
        assert_eq!(selected.len(), 1);
        assert_eq!(
            selected["selected-provider"]["name"].as_str(),
            Some("selected-provider")
        );
        assert_eq!(
            selected["selected-provider"]["base_url"].as_str(),
            Some("http://127.0.0.1:4123/v1")
        );
        assert_eq!(selected["selected-provider"].as_table().unwrap().len(), 2);
        assert_eq!(
            actual["providers"]["legacy-provider"]
                .as_table()
                .unwrap()
                .len(),
            1
        );
        assert_eq!(
            actual["providers"]["legacy-provider"]["base_url"].as_str(),
            Some("https://legacy.invalid/v1")
        );
        assert!(!rendered.contains("unselected"));
    }

    #[test]
    fn codex_startup_selected_provider_seed_allowlist_stays_base_url_only() {
        for family in ["model_providers", "providers"] {
            for field in ["name", "arbitrary", "api_key", "env_key", "http_headers"] {
                let seed = tempfile::tempdir().unwrap();
                let selector = if family == "model_providers" {
                    "model_provider"
                } else {
                    "provider"
                };
                fs::write(seed.path().join("config.toml"), format!(
                    "model = \"gpt-6-astra\"\n{selector} = \"selected\"\n[{family}.selected]\nbase_url = \"http://127.0.0.1:4123/v1\"\n{field} = \"must-reject\"\n"
                )).unwrap();
                let error = read_codex_startup_subset(seed.path(), Path::new("/bounded-workspace"))
                    .unwrap_err();
                assert!(
                    error
                        .to_string()
                        .contains("unsupported provider routing fields"),
                    "{error:#}"
                );
                assert!(error.to_string().contains(field), "{error:#}");
            }
        }
    }

    #[test]
    fn prepare_codex_runtime_env_materializes_bounded_user_startup_subset_only() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let seed_home = temp_dir.path().join("seed-home");
        fs::create_dir_all(&seed_home).expect("create seed home");
        fs::write(
            seed_home.join("auth.json"),
            r#"{"account_id":"acct_test","access_token":"token_test"}"#,
        )
        .expect("write auth");
        fs::write(
            seed_home.join("config.toml"),
            r#"
model = "gpt-5.4"
model_provider = "compat-openai"
openai_base_url = "https://api.openai-proxy.example.invalid/v1"

[model_providers.compat-openai]
base_url = "https://gateway.example.invalid/v1"

[mcp_servers.hidden]
command = "should-not-copy"

[hooks.pre_exec]
command = "should-not-copy"

[agents.helper]
prompt = "should-not-copy"

[requirements]
mode = "should-not-copy"
"#,
        )
        .expect("write user config");
        fs::write(
            seed_home.join("engineering.config.toml"),
            "model = \"gpt-5.5\"\n",
        )
        .expect("write profile overlay");
        let workspace_dir = temp_dir.path().join("workspace");
        let project_codex_dir = workspace_dir.join(".codex");
        fs::create_dir_all(&project_codex_dir).expect("create project codex dir");
        fs::write(
            project_codex_dir.join("config.toml"),
            "model = \"gpt-5.6\"\n",
        )
        .expect("write project config");
        let launcher_dir = workspace_dir.join("launcher");
        fs::create_dir_all(&launcher_dir).expect("create launcher dir");

        let mut runtime_env = BTreeMap::from([(
            SUBSTRATE_INTERNAL_CODEX_AUTH_SEED_HOME_ENV.to_string(),
            seed_home.display().to_string(),
        )]);

        prepare_codex_runtime_env(&mut runtime_env, &workspace_dir, &launcher_dir)
            .expect("seed auth and bounded startup subset");

        let codex_home = launcher_dir.join("codex-home");
        let rendered = fs::read_to_string(codex_home.join("config.toml"))
            .expect("rendered bounded startup config");
        assert_eq!(
            rendered,
            format!(
                concat!(
                    "model = \"gpt-5.4\"\n",
                    "model_provider = \"compat-openai\"\n",
                    "openai_base_url = \"https://api.openai-proxy.example.invalid/v1\"\n\n",
                    "[model_providers.compat-openai]\n",
                    "name = \"compat-openai\"\n",
                    "base_url = \"https://gateway.example.invalid/v1\"\n\n",
                    "[projects.\"{}\"]\n",
                    "trust_level = \"untrusted\"\n"
                ),
                workspace_dir.display()
            )
        );
        assert!(
            !rendered.contains("mcp_servers"),
            "bounded startup subset must not project MCP configuration"
        );
        assert!(
            !rendered.contains("hooks"),
            "bounded startup subset must not project hook configuration"
        );
        assert!(
            !rendered.contains("agents"),
            "bounded startup subset must not project custom agent configuration"
        );
        assert!(
            !rendered.contains("requirements"),
            "bounded startup subset must not project managed requirements state"
        );
        let parsed: toml::Value = rendered
            .parse()
            .expect("parse rendered bounded startup config");
        assert_eq!(
            parsed
                .get("projects")
                .and_then(toml::Value::as_table)
                .and_then(|projects| projects.get(&workspace_dir.display().to_string()))
                .and_then(toml::Value::as_table)
                .and_then(|project| project.get("trust_level"))
                .and_then(toml::Value::as_str),
            Some("untrusted"),
            "isolated CODEX_HOME must mark the real workspace untrusted so repo-local .codex cannot apply"
        );
        assert!(
            !codex_home.join("engineering.config.toml").exists(),
            "Packet 2 must not replay profile overlays into isolated CODEX_HOME"
        );
        assert!(
            !codex_home.join(".codex").join("config.toml").exists(),
            "Packet 2 must not replay repo project config into isolated CODEX_HOME"
        );
        assert!(
            !runtime_env.contains_key(SUBSTRATE_INTERNAL_CODEX_AUTH_SEED_HOME_ENV),
            "internal seed env must be removed before spawning the member runtime"
        );
    }

    #[test]
    fn prepare_codex_runtime_env_marks_authoritative_project_root_untrusted_when_override_is_present(
    ) {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let seed_home = temp_dir.path().join("seed-home");
        let project_root = temp_dir.path().join("workspace");
        let nested_cwd = project_root.join("nested").join("pkg");
        let project_codex_dir = project_root.join(".codex");
        let launcher_dir = temp_dir.path().join("launcher");
        fs::create_dir_all(&seed_home).expect("create seed home");
        fs::create_dir_all(&nested_cwd).expect("create nested cwd");
        fs::create_dir_all(&project_codex_dir).expect("create project codex dir");
        fs::create_dir_all(&launcher_dir).expect("create launcher dir");
        fs::write(
            seed_home.join("auth.json"),
            r#"{"account_id":"acct_test","access_token":"token_test"}"#,
        )
        .expect("write auth");
        fs::write(seed_home.join("config.toml"), "model = \"gpt-5.4\"\n")
            .expect("write startup config");
        fs::write(
            project_codex_dir.join("config.toml"),
            "model = \"gpt-5.6\"\n",
        )
        .expect("write project config");

        let mut runtime_env = BTreeMap::from([
            (
                SUBSTRATE_INTERNAL_CODEX_AUTH_SEED_HOME_ENV.to_string(),
                seed_home.display().to_string(),
            ),
            (
                crate::service::WORLD_PROJECT_DIR_OVERRIDE_ENV.to_string(),
                project_root.display().to_string(),
            ),
        ]);

        prepare_codex_runtime_env(&mut runtime_env, &nested_cwd, &launcher_dir)
            .expect("seed auth and bounded startup subset");

        let rendered = fs::read_to_string(launcher_dir.join("codex-home").join("config.toml"))
            .expect("rendered bounded startup config");
        let parsed: toml::Value = rendered.parse().expect("parse rendered config");
        let projects = parsed
            .get("projects")
            .and_then(toml::Value::as_table)
            .expect("rendered projects table");
        assert_eq!(
            projects.len(),
            1,
            "only the authoritative project root should be demoted in isolated CODEX_HOME"
        );
        assert_eq!(
            projects
                .get(&project_root.display().to_string())
                .and_then(toml::Value::as_table)
                .and_then(|project| project.get("trust_level"))
                .and_then(toml::Value::as_str),
            Some("untrusted"),
            "repo-root trust demotion must use SUBSTRATE_WORLD_PROJECT_DIR when launch cwd is nested"
        );
        assert!(
            !projects.contains_key(&nested_cwd.display().to_string()),
            "effective nested cwd must not become the trust-demotion key"
        );
        assert!(
            !launcher_dir
                .join("codex-home")
                .join(".codex")
                .join("config.toml")
                .exists(),
            "Packet 2 must still not replay repo project config into isolated CODEX_HOME"
        );
    }

    #[test]
    fn prepare_codex_runtime_env_fails_closed_when_seed_home_bridge_is_missing() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let workspace_dir = temp_dir.path().join("workspace");
        let launcher_dir = temp_dir.path().join("launcher");
        fs::create_dir_all(&workspace_dir).expect("create workspace dir");
        fs::create_dir_all(&launcher_dir).expect("create launcher dir");

        let mut runtime_env = BTreeMap::new();

        let err = prepare_codex_runtime_env(&mut runtime_env, &workspace_dir, &launcher_dir)
            .expect_err("missing seed-home bridge must fail closed");
        let message = err.to_string();
        assert!(
            message.contains("policy-gated bootstrap seed source"),
            "unexpected error: {err:#}"
        );
        assert!(
            message.contains("Packet 2 seam"),
            "unexpected error: {err:#}"
        );
        assert!(
            message.contains("ambient repo-local .codex still active"),
            "unexpected error: {err:#}"
        );
        assert!(
            !message.contains(SUBSTRATE_INTERNAL_CODEX_AUTH_SEED_HOME_ENV),
            "unexpected error leaked internal bridge env var name: {err:#}"
        );
        assert!(
            !runtime_env.contains_key("CODEX_HOME"),
            "fail-closed bridge errors must not materialize isolated CODEX_HOME"
        );
    }

    #[test]
    fn prepare_codex_runtime_env_requires_auth_json_when_seed_home_is_declared() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let seed_home = temp_dir.path().join("seed-home");
        let workspace_dir = temp_dir.path().join("workspace");
        fs::create_dir_all(&seed_home).expect("create seed home");
        fs::create_dir_all(&workspace_dir).expect("create workspace dir");
        let launcher_dir = temp_dir.path().join("launcher");
        fs::create_dir_all(&launcher_dir).expect("create launcher dir");

        let mut runtime_env = BTreeMap::from([(
            SUBSTRATE_INTERNAL_CODEX_AUTH_SEED_HOME_ENV.to_string(),
            seed_home.display().to_string(),
        )]);

        let err = prepare_codex_runtime_env(&mut runtime_env, &workspace_dir, &launcher_dir)
            .expect_err("missing auth.json must fail closed");
        assert!(
            err.to_string().contains("auth.json"),
            "unexpected error: {err:#}"
        );
    }

    #[test]
    fn prepare_codex_runtime_env_fails_closed_when_startup_model_cannot_be_derived() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let seed_home = temp_dir.path().join("seed-home");
        let workspace_dir = temp_dir.path().join("workspace");
        fs::create_dir_all(&seed_home).expect("create seed home");
        fs::create_dir_all(&workspace_dir).expect("create workspace dir");
        fs::write(
            seed_home.join("auth.json"),
            r#"{"account_id":"acct_test","access_token":"token_test"}"#,
        )
        .expect("write auth");
        fs::write(
            seed_home.join("config.toml"),
            r#"
[model_providers.compat-openai]
base_url = "https://gateway.example.invalid/v1"
"#,
        )
        .expect("write incomplete config");
        let launcher_dir = temp_dir.path().join("launcher");
        fs::create_dir_all(&launcher_dir).expect("create launcher dir");

        let mut runtime_env = BTreeMap::from([(
            SUBSTRATE_INTERNAL_CODEX_AUTH_SEED_HOME_ENV.to_string(),
            seed_home.display().to_string(),
        )]);

        let err = prepare_codex_runtime_env(&mut runtime_env, &workspace_dir, &launcher_dir)
            .expect_err("missing startup model must fail closed");
        let message = err.to_string();
        assert!(
            message.contains("could not derive a bounded startup model"),
            "unexpected error: {err:#}"
        );
        assert!(
            message.contains("unsupported default model"),
            "unexpected error: {err:#}"
        );
        assert!(
            !runtime_env.contains_key(SUBSTRATE_INTERNAL_CODEX_AUTH_SEED_HOME_ENV),
            "internal seed env must still be removed on fail-closed startup subset errors"
        );
    }

    #[test]
    fn prepare_codex_runtime_env_fails_closed_when_selected_provider_uses_unsupported_routing_shape(
    ) {
        for (selection, provider_table_header) in [
            (
                "model_provider = \"compat-openai\"",
                "[model_providers.compat-openai]",
            ),
            ("provider = \"compat-openai\"", "[providers.compat-openai]"),
        ] {
            let temp_dir = tempfile::tempdir().expect("temp dir");
            let seed_home = temp_dir.path().join("seed-home");
            let workspace_dir = temp_dir.path().join("workspace");
            fs::create_dir_all(&seed_home).expect("create seed home");
            fs::create_dir_all(&workspace_dir).expect("create workspace dir");
            fs::write(
                seed_home.join("auth.json"),
                r#"{"account_id":"acct_test","access_token":"token_test"}"#,
            )
            .expect("write auth");
            fs::write(
                seed_home.join("config.toml"),
                format!(
                    "model = \"gpt-5.4\"\n{selection}\n\n{provider_table_header}\nbase_url = \"https://gateway.example.invalid/v1\"\nwire_api = \"responses\"\n",
                ),
            )
            .expect("write unsupported provider routing config");
            let launcher_dir = temp_dir.path().join("launcher");
            fs::create_dir_all(&launcher_dir).expect("create launcher dir");

            let mut runtime_env = BTreeMap::from([(
                SUBSTRATE_INTERNAL_CODEX_AUTH_SEED_HOME_ENV.to_string(),
                seed_home.display().to_string(),
            )]);

            let err = prepare_codex_runtime_env(&mut runtime_env, &workspace_dir, &launcher_dir)
                .expect_err("unsupported selected provider routing shape must fail closed");
            let message = err.to_string();
            assert!(
                message.contains("unsupported provider routing fields"),
                "unexpected error: {err:#}"
            );
            assert!(message.contains("wire_api"), "unexpected error: {err:#}");
            assert!(
                !runtime_env.contains_key(SUBSTRATE_INTERNAL_CODEX_AUTH_SEED_HOME_ENV),
                "internal seed env must still be removed on fail-closed provider routing errors"
            );
        }
    }

    #[test]
    fn prepare_codex_runtime_env_fails_closed_when_selected_provider_block_is_missing_or_has_no_supported_base_url(
    ) {
        for (selection, provider_table_header, provider_body, expected_message_fragment) in [
            (
                "model_provider = \"compat-openai\"",
                "[model_providers.other-provider]",
                "base_url = \"https://gateway.example.invalid/v1\"\n",
                "coupled provider block is missing",
            ),
            (
                "provider = \"compat-openai\"",
                "[providers.compat-openai]",
                "",
                "must provide a supported non-empty base_url",
            ),
        ] {
            let temp_dir = tempfile::tempdir().expect("temp dir");
            let seed_home = temp_dir.path().join("seed-home");
            let workspace_dir = temp_dir.path().join("workspace");
            fs::create_dir_all(&seed_home).expect("create seed home");
            fs::create_dir_all(&workspace_dir).expect("create workspace dir");
            fs::write(
                seed_home.join("auth.json"),
                r#"{"account_id":"acct_test","access_token":"token_test"}"#,
            )
            .expect("write auth");
            fs::write(
                seed_home.join("config.toml"),
                format!(
                    "model = \"gpt-5.4\"\n{selection}\n\n{provider_table_header}\n{provider_body}",
                ),
            )
            .expect("write incomplete provider routing config");
            let launcher_dir = temp_dir.path().join("launcher");
            fs::create_dir_all(&launcher_dir).expect("create launcher dir");

            let mut runtime_env = BTreeMap::from([(
                SUBSTRATE_INTERNAL_CODEX_AUTH_SEED_HOME_ENV.to_string(),
                seed_home.display().to_string(),
            )]);

            let err = prepare_codex_runtime_env(&mut runtime_env, &workspace_dir, &launcher_dir)
                .expect_err("selected provider routing truth must fail closed");
            let message = err.to_string();
            assert!(
                message.contains(expected_message_fragment),
                "unexpected error: {err:#}"
            );
            assert!(
                !runtime_env.contains_key(SUBSTRATE_INTERNAL_CODEX_AUTH_SEED_HOME_ENV),
                "internal seed env must still be removed on fail-closed provider routing errors"
            );
        }
    }

    #[test]
    fn prepare_codex_runtime_env_cleanup_prepared_launcher_dir_removes_directory_once() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let launcher_dir = temp_dir.path().join("launcher");
        fs::create_dir_all(&launcher_dir).expect("create launcher dir");
        fs::write(launcher_dir.join("auth.json"), "{}").expect("write launcher artifact");

        let mut cleanup_slot = Some(launcher_dir.clone());
        cleanup_prepared_launcher_dir(&mut cleanup_slot);
        cleanup_prepared_launcher_dir(&mut cleanup_slot);

        assert!(
            cleanup_slot.is_none(),
            "cleanup must clear the launcher slot"
        );
        assert!(
            !launcher_dir.exists(),
            "cleanup must remove prepared launcher artifacts on fail-closed exits"
        );
    }

    #[test]
    fn codex_member_runtime_launch_shape_keeps_workspace_cwd_and_explicit_workspace_add_dirs() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let workspace_dir = temp_dir.path().join("workspace");
        let launcher_dir = temp_dir.path().join("launcher");
        fs::create_dir_all(&workspace_dir).expect("create workspace dir");
        fs::create_dir_all(&launcher_dir).expect("create launcher dir");

        let process_working_dir = member_runtime_process_working_dir(
            MemberRuntimeBackendKindV1::Codex,
            &workspace_dir,
            &launcher_dir,
        );
        let extensions = member_runtime_workspace_access_extensions(
            MemberRuntimeBackendKindV1::Codex,
            &workspace_dir,
        );

        assert_eq!(
            process_working_dir, workspace_dir,
            "Codex retained sessions must preserve the real workspace cwd for truthful relative-path semantics"
        );
        assert_eq!(
            extensions.get(ADD_DIRS_EXTENSION_V1),
            Some(&json!({
                "dirs": [workspace_dir.display().to_string()],
            })),
            "Codex retained sessions must still declare workspace access explicitly via add_dirs"
        );
    }

    #[test]
    fn claude_member_runtime_launch_shape_keeps_workspace_cwd_without_add_dirs() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let workspace_dir = temp_dir.path().join("workspace");
        let launcher_dir = temp_dir.path().join("launcher");
        fs::create_dir_all(&workspace_dir).expect("create workspace dir");
        fs::create_dir_all(&launcher_dir).expect("create launcher dir");

        let process_working_dir = member_runtime_process_working_dir(
            MemberRuntimeBackendKindV1::ClaudeCode,
            &workspace_dir,
            &launcher_dir,
        );
        let extensions = member_runtime_workspace_access_extensions(
            MemberRuntimeBackendKindV1::ClaudeCode,
            &workspace_dir,
        );

        assert_eq!(
            process_working_dir, workspace_dir,
            "non-Codex backends should keep their existing workspace cwd behavior"
        );
        assert!(
            extensions.is_empty(),
            "non-Codex backends should not inherit Codex add_dirs behavior"
        );
    }

    #[test]
    fn surfaced_uaa_session_id_prefers_internal_then_session_id() {
        let payload = json!({
            "internal": {
                "uaa_session_id": "uaa_internal"
            },
            "session": {
                "id": "uaa_session"
            }
        });
        assert_eq!(
            surfaced_uaa_session_id_from_data(Some(&payload)).as_deref(),
            Some("uaa_internal")
        );

        let payload = json!({
            "session": {
                "id": "uaa_session"
            }
        });
        assert_eq!(
            surfaced_uaa_session_id_from_data(Some(&payload)).as_deref(),
            Some("uaa_session")
        );
    }

    #[test]
    fn surfaced_uaa_thread_id_prefers_explicit_thread_then_session_id() {
        let payload = json!({
            "thread_id": "thread-explicit",
            "session": {
                "id": "thread-session"
            }
        });
        assert_eq!(
            surfaced_uaa_thread_id_from_data(Some(&payload)).as_deref(),
            Some("thread-explicit")
        );

        let payload = json!({
            "session": {
                "id": "thread-session"
            },
            "raw_event": {
                "thread_id": "thread-raw-event"
            }
        });
        assert_eq!(
            surfaced_uaa_thread_id_from_data(Some(&payload)).as_deref(),
            Some("thread-session")
        );

        let payload = json!({
            "session": {
                "id": "thread-session"
            }
        });
        assert_eq!(
            surfaced_uaa_thread_id_from_data(Some(&payload)).as_deref(),
            Some("thread-session")
        );
    }

    #[test]
    fn validate_submit_turn_request_accepts_matching_retained_identity() {
        validate_submit_turn_request(&sample_submit_turn_request(), sample_retained_identity())
            .expect("matching retained member identity should validate");
    }

    #[test]
    fn e2_submit_turn_consumes_exact_carrier_before_runtime_submission() {
        let request = sample_e2_submit_turn_request();
        let authenticated = request
            .policy_snapshot_carrier
            .as_ref()
            .expect("E2 request carries authenticated policy")
            .clone();
        request
            .validate()
            .expect("WorldService request boundary consumes exact carrier");
        validate_submit_turn_request(&request, sample_retained_identity())
            .expect("carrier-bound request matches retained runtime");
        request
            .validate_against_authenticated_policy_carrier(&authenticated)
            .expect("request preserves exact authenticated carrier");
        assert_eq!(
            request
                .policy_snapshot_carrier
                .as_ref()
                .expect("carried policy")
                .policy_snapshot_bytes()
                .expect("exact E1 bytes"),
            serde_json::to_vec(
                &authenticated
                    .policy_snapshot()
                    .expect("decode carried PolicySnapshotV3")
            )
            .expect("reserialize exact E1 snapshot")
        );

        let encoded = serde_json::to_value(&request).expect("serialize WorldService request");
        let decoded: MemberTurnSubmitRequestV1 =
            serde_json::from_value(encoded).expect("strict WorldService request decode");
        assert_eq!(decoded.policy_snapshot_carrier, Some(authenticated.clone()));

        let mut omitted = request;
        omitted.policy_snapshot_carrier = None;
        assert!(omitted
            .validate_against_authenticated_policy_carrier(&authenticated)
            .is_err());
    }

    #[test]
    fn e2_submitted_turn_run_request_replaces_launch_policy_with_carried_snapshot() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let mut active = sample_e2_active_member_runtime(&temp_dir, "ash_member", "spn_bootstrap");
        let active_mut = Arc::get_mut(&mut active).expect("unique active fixture");
        active_mut.process_working_dir = active_mut.workspace_dir.clone();
        active_mut.env.extend([
            (
                crate::service::WORLD_PROJECT_DIR_OVERRIDE_ENV.to_string(),
                active_mut.workspace_dir.display().to_string(),
            ),
            (
                crate::service::WORLD_FS_ISOLATION_ENV.to_string(),
                "stale-launch-policy".to_string(),
            ),
            (
                crate::service::WORLD_FS_LANDLOCK_READ_ALLOWLIST_ENV.to_string(),
                "/stale/launch/read".to_string(),
            ),
            (
                crate::enforcement_plan::WORLD_FS_ENFORCEMENT_PLAN_B64_ENV.to_string(),
                "stale-launch-plan".to_string(),
            ),
        ]);

        let request = sample_e2_submit_turn_request();
        let snapshot = resolve_submitted_turn_policy(&request, active.as_ref())
            .expect("authenticated retained-turn carrier")
            .expect("E2 snapshot");
        let run_request = build_submitted_turn_run_request(
            active.as_ref(),
            request.prompt.clone(),
            "uaa_session",
            Some(&snapshot),
        )
        .expect("carrier-derived submitted-turn request");

        let expected_plan = crate::enforcement_plan::maybe_encode_from_snapshot(&snapshot)
            .expect("encode exact E1 snapshot");
        assert_eq!(
            run_request
                .env
                .get(crate::enforcement_plan::WORLD_FS_ENFORCEMENT_PLAN_B64_ENV),
            expected_plan.as_ref(),
            "the actual submitted child must receive the carrier-derived plan"
        );
        assert_ne!(
            run_request
                .env
                .get(crate::service::WORLD_FS_LANDLOCK_READ_ALLOWLIST_ENV)
                .map(String::as_str),
            Some("/stale/launch/read"),
            "launch-time policy must not survive into an E2 turn"
        );
        assert_eq!(
            run_request
                .env
                .get(crate::service::WORLD_FS_ISOLATION_ENV)
                .map(String::as_str),
            Some("full")
        );
        assert!(
            !run_request
                .env
                .contains_key(crate::service::WORLD_FS_LANDLOCK_WRITE_ALLOWLIST_ENV),
            "write.enabled=false must not activate its dormant canonical write allowlist"
        );
        let retained_binary = active.binary_path.display().to_string();
        let retained_read_paths = run_request
            .env
            .get(crate::service::WORLD_FS_LANDLOCK_READ_ALLOWLIST_ENV)
            .expect("retained-turn read allowlist");
        assert!(
            retained_read_paths
                .lines()
                .any(|path| path == retained_binary),
            "the exact launch-authenticated retained binary must remain readable so Landlock can execute it"
        );
        assert!(
            !retained_read_paths
                .lines()
                .any(|path| path == active.workspace_dir.display().to_string()),
            "the retained binary prerequisite must not grant its workspace parent"
        );
    }

    #[test]
    fn activated_e2_retained_turn_rejects_missing_carrier_before_submission() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let active = sample_e2_active_member_runtime(&temp_dir, "rwp_member", "spn_bootstrap");
        let mut request = sample_submit_turn_request();
        request.participant_id = "rwp_member".to_string();

        let error = resolve_submitted_turn_policy(&request, active.as_ref())
            .expect_err("authority-managed retained turn must carry E2 policy");
        assert!(
            error
                .to_string()
                .contains("requires policy_snapshot_carrier"),
            "unexpected error: {error:#}"
        );
    }

    #[test]
    fn retained_turn_rejects_worker_cap_substitution_after_authentication() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let active = sample_e2_active_member_runtime(&temp_dir, "ash_member", "spn_bootstrap");
        let request = sample_e2_submit_turn_request();
        resolve_submitted_turn_policy(&request, active.as_ref())
            .expect("verify launch-pinned worker cap")
            .expect("E2 policy snapshot");

        let mut substituted = request;
        substituted
            .policy_snapshot_carrier
            .as_mut()
            .expect("carrier")
            .immutable_worker_cap_ref
            .commitment_id = "dpc_018f0f3a-9b2c-7def-8abc-0123456789ae".to_string();
        substituted
            .validate()
            .expect("structurally valid substitution");
        let error = resolve_submitted_turn_policy(&substituted, active.as_ref())
            .expect_err("immutable cap substitution must fail before submission");
        assert!(
            error.to_string().contains("worker-cap identity changed"),
            "unexpected error: {error:#}"
        );
    }

    #[test]
    fn retained_turn_rejects_caller_backend_substitution_from_launch_activation() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let active = sample_e2_active_member_runtime(&temp_dir, "ash_member", "spn_bootstrap");
        let mut substituted = sample_e2_submit_turn_request();
        substituted
            .policy_snapshot_carrier
            .as_mut()
            .expect("carrier")
            .caller_backend_id = "cli:claude-code".to_string();
        substituted
            .policy_snapshot_carrier
            .as_mut()
            .expect("carrier")
            .subject
            .message_id = Some("wwm_018f0f3a-9b2c-7def-8abc-0123456789ac".to_string());
        substituted.acceptance_context = Some(transport_api_types::WorldWorkAcceptanceContextV1 {
            schema_version: 1,
            proposed_acceptance_record_id: "wwa_018f0f3a-9b2c-7def-8abc-0123456789ab".to_string(),
            request_id: substituted.run_id.clone(),
            message_id: Some("wwm_018f0f3a-9b2c-7def-8abc-0123456789ac".to_string()),
            caller_backend_id: "cli:claude-code".to_string(),
            host_transition_correlation: None,
        });
        substituted
            .validate()
            .expect("request and turn carrier agree on the substituted caller backend");

        let error = resolve_submitted_turn_policy(&substituted, active.as_ref())
            .expect_err("turn caller backend must match launch-pinned activation");
        assert!(
            error.to_string().contains("launch-pinned E2 activation"),
            "unexpected error: {error:#}"
        );
    }

    #[test]
    fn consecutive_e2_turns_derive_independent_non_accumulating_policy_envs() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let mut active = sample_active_member_runtime(&temp_dir, "ash_member", "spn_bootstrap");
        let active_mut = Arc::get_mut(&mut active).expect("unique active fixture");
        active_mut.process_working_dir = active_mut.workspace_dir.clone();
        active_mut.env.insert(
            crate::service::WORLD_PROJECT_DIR_OVERRIDE_ENV.to_string(),
            active_mut.workspace_dir.display().to_string(),
        );

        let snapshot_a = sample_e2_submit_turn_request()
            .policy_snapshot_carrier
            .expect("carrier A")
            .policy_snapshot()
            .expect("snapshot A");
        let mut snapshot_b = snapshot_a.clone();
        snapshot_b
            .world_fs
            .discover
            .as_mut()
            .expect("discover dimension")
            .allow_list = vec!["workspace/turn-b.txt".to_string()];
        snapshot_b
            .world_fs
            .read
            .as_mut()
            .expect("read dimension")
            .allow_list = vec!["workspace/turn-b.txt".to_string()];
        snapshot_b = snapshot_b.canonicalize().expect("canonical snapshot B");

        let request_a = build_submitted_turn_run_request(
            active.as_ref(),
            "turn A".to_string(),
            "uaa_session",
            Some(&snapshot_a),
        )
        .expect("turn A request");
        let request_b = build_submitted_turn_run_request(
            active.as_ref(),
            "turn B".to_string(),
            "uaa_session",
            Some(&snapshot_b),
        )
        .expect("turn B request");

        let reads_a = request_a
            .env
            .get(crate::service::WORLD_FS_LANDLOCK_READ_ALLOWLIST_ENV)
            .expect("turn A read allowlist");
        let reads_b = request_b
            .env
            .get(crate::service::WORLD_FS_LANDLOCK_READ_ALLOWLIST_ENV)
            .expect("turn B read allowlist");
        assert!(reads_a.contains("workspace/exact.txt"));
        assert!(!reads_a.contains("turn-b.txt"));
        assert!(reads_b.contains("workspace/turn-b.txt"));
        assert!(!reads_b.contains("exact.txt"));
    }

    type SubmitTurnDriftCase = (
        &'static str,
        fn(&mut MemberTurnSubmitRequestV1),
        &'static str,
    );

    #[test]
    fn validate_submit_turn_request_rejects_retained_identity_drift() {
        let cases: [SubmitTurnDriftCase; 5] = [
            (
                "orchestration_session_id",
                |req| req.orchestration_session_id = "orch_999".to_string(),
                "member_turn_submit.orchestration_session_id mismatch",
            ),
            (
                "orchestrator_participant_id",
                |req| req.orchestrator_participant_id = "ash_orchestrator_other".to_string(),
                "member_turn_submit.orchestrator_participant_id mismatch",
            ),
            (
                "backend_id",
                |req| req.backend_id = "cli:anthropic".to_string(),
                "member_turn_submit.backend_id mismatch",
            ),
            (
                "world_id",
                |req| req.world_id = "world_999".to_string(),
                "member_turn_submit.world_id mismatch",
            ),
            (
                "world_generation",
                |req| req.world_generation = 9,
                "member_turn_submit.world_generation mismatch",
            ),
        ];

        for (field, mutate, expected) in cases {
            let mut req = sample_submit_turn_request();
            mutate(&mut req);

            let err = match validate_submit_turn_request(&req, sample_retained_identity()) {
                Ok(()) => panic!("expected {field} drift to be rejected"),
                Err(err) => err,
            };
            assert!(
                err.to_string().contains(expected),
                "expected {field} drift to mention {expected}, got: {err}"
            );
        }
    }

    #[test]
    fn retained_member_key_uses_session_generation_and_backend_id() {
        let req = sample_submit_turn_request();

        assert_eq!(
            RetainedMemberKey::from_submit(&req),
            RetainedMemberKey {
                orchestration_session_id: "orch_123".to_string(),
                world_generation: 7,
                backend_id: "cli:codex".to_string(),
            }
        );
    }

    fn sample_stream_context() -> MemberStreamContext {
        MemberStreamContext {
            orchestration_session_id: "orch_123".to_string(),
            run_id: "run_bootstrap".to_string(),
            acceptance_context: None,
            participant_id: "ash_member".to_string(),
            orchestrator_participant_id: "ash_orchestrator".to_string(),
            parent_participant_id: None,
            resumed_from_participant_id: None,
            backend_id: "cli:codex".to_string(),
            protocol: json!("substrate.agent.session"),
            authenticated_thread_id: None,
        }
    }

    fn sample_world_binding() -> SharedWorldBindingSnapshot {
        SharedWorldBindingSnapshot {
            orchestration_session_id: "orch_123".to_string(),
            world_id: "world_123".to_string(),
            world_generation: 7,
            binding_state: world_api::SharedWorldBindingState::Active,
        }
    }

    #[test]
    fn test_native_backend_cannot_supply_service_completion_marker() {
        for marker in [
            transport_api_types::RetainedBootstrapCompletionV1::EVENT_DATA_KEY,
            transport_api_types::RetainedForkBootstrapCompletionV1::EVENT_DATA_KEY,
            transport_api_types::OrdinaryInitialBootstrapRetirementV1::EVENT_DATA_KEY,
        ] {
            for mode in [MemberStreamMode::Bootstrap, MemberStreamMode::SubmittedTurn] {
                let result = agent_event_from_wrapper_event(
                    &sample_stream_context(),
                    &sample_world_binding(),
                    "bootstrap-span",
                    AgentWrapperEvent {
                        agent_kind: agent_api::AgentWrapperKind::new("codex").unwrap(),
                        kind: AgentWrapperEventKind::Status,
                        channel: None,
                        text: None,
                        message: None,
                        data: Some(json!({(marker): {"schema_version":1}})),
                    },
                    &mut false,
                    mode,
                    "codex_world",
                );
                assert!(result
                    .err()
                    .unwrap()
                    .to_string()
                    .contains("reserved service completion marker"));
            }
        }
    }

    #[test]
    fn bootstrap_completion_without_session_handle_emits_only_exit() {
        let mut emitted_registered = false;
        let mut producer = RuntimeEventStreamProducer::new();
        producer.start("spn_bootstrap".to_string()).expect("Start");
        let frames = frames_from_completion(
            &sample_stream_context(),
            &sample_world_binding(),
            "spn_bootstrap",
            Ok(AgentWrapperCompletion {
                status: std::process::Command::new("sh")
                    .arg("-c")
                    .arg("exit 0")
                    .status()
                    .expect("completion status"),
                final_text: None,
                data: None,
            }),
            None,
            &mut emitted_registered,
            MemberStreamMode::Bootstrap,
            "codex_world",
            &mut producer,
        )
        .expect("completion frames");

        assert_eq!(
            frames.len(),
            1,
            "bootstrap without continuity metadata must stay terminal"
        );
        assert!(matches!(
            frames.first(),
            Some(ExecuteStreamFrame::Exit { exit: 0, .. })
        ));
        assert!(
            !emitted_registered,
            "non-retained bootstrap should not invent a registered event"
        );
    }

    #[test]
    fn registered_event_surfaces_authoritative_bootstrap_identity() {
        let event = registered_event_from_data(
            &sample_stream_context(),
            &sample_world_binding(),
            "spn_bootstrap",
            Some(&json!({
                "schema": SESSION_HANDLE_SCHEMA_V1,
                "session": {
                    "id": "uaa_session"
                }
            })),
            "codex_world",
        )
        .expect("registered event");

        assert_eq!(event.kind, AgentEventKind::Registered);
        assert_eq!(event.participant_id.as_deref(), Some("ash_member"));
        assert_eq!(event.parent_participant_id, None);
        assert_eq!(event.resumed_from_participant_id, None);
        assert_eq!(event.backend_id.as_deref(), Some("cli:codex"));
        assert_eq!(event.thread_id.as_deref(), Some("uaa_session"));
        assert_eq!(event.world_id.as_deref(), Some("world_123"));
        assert_eq!(event.world_generation, Some(7));
        assert_eq!(event.span_id.as_deref(), Some("spn_bootstrap"));
    }

    #[test]
    fn submitted_turn_event_surfaces_thread_id_from_uaa_payload() {
        let wrapper_event = AgentWrapperEvent {
            agent_kind: agent_api::AgentWrapperKind::new("codex").expect("agent kind"),
            kind: AgentWrapperEventKind::Status,
            channel: Some("status".to_string()),
            text: None,
            message: Some("submitted turn".to_string()),
            data: Some(json!({
                "thread_id": "thread-submitted",
                "turn_id": "turn-2"
            })),
        };

        let prepared = agent_event_from_wrapper_event(
            &sample_stream_context(),
            &sample_world_binding(),
            "spn_submitted",
            wrapper_event,
            &mut false,
            MemberStreamMode::SubmittedTurn,
            "codex_world",
        )
        .expect("submitted turn event");
        let event = prepared.event;

        assert_eq!(event.thread_id.as_deref(), Some("thread-submitted"));
        assert_eq!(event.participant_id.as_deref(), Some("ash_member"));
        assert_eq!(event.backend_id.as_deref(), Some("cli:codex"));
        assert_eq!(event.world_id.as_deref(), Some("world_123"));
        assert_eq!(event.world_generation, Some(7));

        let missing_explicit_thread = AgentWrapperEvent {
            agent_kind: agent_api::AgentWrapperKind::new("codex").expect("agent kind"),
            kind: AgentWrapperEventKind::Status,
            channel: Some("status".to_string()),
            text: None,
            message: Some("submitted turn".to_string()),
            data: Some(json!({
                "event_class": "progress_update",
                "session": {
                    "id": "thread-from-session"
                },
                "payload": {
                    "message": "submitted turn"
                }
            })),
        };
        let err = match agent_event_from_wrapper_event(
            &sample_member_stream_context(Some("wwm_018f0f3a-9b2c-7def-8abc-0123456789ac")),
            &sample_world_binding(),
            "spn_submitted",
            missing_explicit_thread,
            &mut false,
            MemberStreamMode::SubmittedTurn,
            "codex_world",
        ) {
            Ok(_) => panic!("submitted turn without explicit thread_id must fail closed"),
            Err(err) => err,
        };
        assert!(
            err.to_string()
                .contains("typed retained worker event omitted thread_id"),
            "unexpected error: {err}"
        );

        let mut authenticated_context =
            sample_member_stream_context(Some("wwm_018f0f3a-9b2c-7def-8abc-0123456789ac"));
        authenticated_context.authenticated_thread_id = Some("thread-pinned".to_string());
        let prepared = agent_event_from_wrapper_event(
            &authenticated_context,
            &sample_world_binding(),
            "spn_submitted",
            AgentWrapperEvent {
                agent_kind: agent_api::AgentWrapperKind::new("codex").expect("agent kind"),
                kind: AgentWrapperEventKind::TextOutput,
                channel: Some("assistant".to_string()),
                text: Some("authenticated assistant reply".to_string()),
                message: None,
                data: None,
            },
            &mut false,
            MemberStreamMode::SubmittedTurn,
            "codex_world",
        )
        .expect("authenticated retained session supplies the thread identity");
        let facet = prepared
            .worker_event_facet
            .expect("submitted reply must carry a typed worker event facet");
        assert_eq!(facet.thread_id, "thread-pinned");
        assert_eq!(facet.event_class, WorldWorkerEventClassV1::Reply);

        let prepared = agent_event_from_wrapper_event(
            &authenticated_context,
            &sample_world_binding(),
            "spn_submitted",
            AgentWrapperEvent {
                agent_kind: agent_api::AgentWrapperKind::new("codex").expect("agent kind"),
                kind: AgentWrapperEventKind::Error,
                channel: Some("error".to_string()),
                text: None,
                message: Some("submitted child failed before agent output".to_string()),
                data: None,
            },
            &mut false,
            MemberStreamMode::SubmittedTurn,
            "codex_world",
        )
        .expect("authenticated retained execution error is typed as a failure");
        let facet = prepared
            .worker_event_facet
            .expect("submitted failure must carry a typed worker event facet");
        assert_eq!(facet.thread_id, "thread-pinned");
        assert_eq!(facet.event_class, WorldWorkerEventClassV1::Failure);
        assert_eq!(
            facet.payload.get("message"),
            Some(&json!("submitted child failed before agent output"))
        );
    }

    #[test]
    fn submitted_turn_payload_is_sanitized_to_compatibility_subset() {
        let wrapper_event = AgentWrapperEvent {
            agent_kind: agent_api::AgentWrapperKind::new("codex").expect("agent kind"),
            kind: AgentWrapperEventKind::Status,
            channel: Some("status".to_string()),
            text: None,
            message: Some("raw wrapper secret".to_string()),
            data: Some(json!({
                "type": "item.started",
                "thread_id": "thread-submitted",
                "turn_id": "turn-2",
                "item_id": "cmd-1",
                "status": "in_progress",
                "item_type": "command_execution",
                "event_id": "evt-provider-1",
                "message_id": "msg-provider-1",
                "message": "tool progress",
                "content": {
                    "command": "cargo test",
                    "prompt": "secret prompt"
                },
                "tool": {
                    "kind": "command_execution",
                    "status": "running",
                    "phase": "execute",
                    "prompt": "also secret"
                },
                "prompt": "root secret"
            })),
        };

        let prepared = agent_event_from_wrapper_event(
            &sample_member_stream_context(Some("wwm_018f0f3a-9b2c-7def-8abc-0123456789ac")),
            &sample_world_binding(),
            "spn_submitted",
            wrapper_event,
            &mut false,
            MemberStreamMode::SubmittedTurn,
            "codex_world",
        )
        .expect("submitted turn event");
        let worker_event = prepared
            .worker_event_facet
            .as_ref()
            .expect("submitted turn event must carry typed worker_event facet");

        assert_eq!(
            worker_event.event_class,
            WorldWorkerEventClassV1::ProgressUpdate
        );
        assert_eq!(
            worker_event.payload.get("message"),
            Some(&json!("tool progress"))
        );
        assert_eq!(
            prepared.event.data.get("message"),
            Some(&json!("tool progress"))
        );
        assert!(worker_event
            .payload
            .pointer("/uaa_event/content/command")
            .is_none());
        assert_eq!(
            worker_event.payload.pointer("/uaa_event/tool/kind"),
            Some(&json!("command_execution"))
        );
        assert_eq!(
            worker_event.payload.pointer("/uaa_event/event_id"),
            Some(&json!("evt-provider-1"))
        );
        assert_eq!(
            prepared.event.data.pointer("/uaa_event/event_id"),
            Some(&json!("evt-provider-1"))
        );
        assert_eq!(
            prepared.event.data.pointer("/uaa_event/message_id"),
            Some(&json!("msg-provider-1"))
        );
        assert!(worker_event.payload.get("prompt").is_none());
        assert!(worker_event
            .payload
            .pointer("/uaa_event/content/prompt")
            .is_none());
        assert!(worker_event
            .payload
            .pointer("/uaa_event/tool/prompt")
            .is_none());
        assert!(prepared.event.data.pointer("/uaa_event/prompt").is_none());
        assert!(prepared
            .event
            .data
            .pointer("/uaa_event/content/prompt")
            .is_none());
        assert!(prepared
            .event
            .data
            .pointer("/uaa_event/tool/prompt")
            .is_none());

        let missing_supported_payload = AgentWrapperEvent {
            agent_kind: agent_api::AgentWrapperKind::new("codex").expect("agent kind"),
            kind: AgentWrapperEventKind::Status,
            channel: Some("status".to_string()),
            text: None,
            message: Some("tool progress".to_string()),
            data: Some(json!({
                "event_class": "progress_update",
                "thread_id": "thread-submitted"
            })),
        };
        let err = match agent_event_from_wrapper_event(
            &sample_member_stream_context(Some("wwm_018f0f3a-9b2c-7def-8abc-0123456789ac")),
            &sample_world_binding(),
            "spn_submitted",
            missing_supported_payload,
            &mut false,
            MemberStreamMode::SubmittedTurn,
            "codex_world",
        ) {
            Ok(_) => panic!("submitted turn without supported payload must fail closed"),
            Err(err) => err,
        };
        assert!(
            err.to_string()
                .contains("unsupported_worker_event_shape: retained member emitted no supported typed payload"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn submitted_turn_control_ack_remains_typed_without_nested_event_class() {
        let wrapper_event = AgentWrapperEvent {
            agent_kind: agent_api::AgentWrapperKind::new("codex").expect("agent kind"),
            kind: AgentWrapperEventKind::Status,
            channel: Some("status".to_string()),
            text: None,
            message: Some("raw wrapper directive".to_string()),
            data: Some(json!({
                "event_class": "control_ack",
                "thread_id": "thread-submitted",
                "payload": {
                    "message": "directive applied",
                    "detail": "directive detail"
                }
            })),
        };

        let prepared = agent_event_from_wrapper_event(
            &sample_member_stream_context(Some("wwm_018f0f3a-9b2c-7def-8abc-0123456789ac")),
            &sample_world_binding(),
            "spn_submitted",
            wrapper_event,
            &mut false,
            MemberStreamMode::SubmittedTurn,
            "codex_world",
        )
        .expect("submitted turn control_ack event");
        let worker_event = prepared
            .worker_event_facet
            .as_ref()
            .expect("submitted turn control_ack must carry typed worker_event facet");

        assert_eq!(
            worker_event.event_class,
            WorldWorkerEventClassV1::ControlAck
        );
        assert_eq!(
            worker_event.payload.get("message"),
            Some(&json!("directive applied"))
        );
        assert_eq!(
            prepared.event.data.get("message"),
            Some(&json!("directive applied"))
        );
        assert_eq!(
            worker_event.payload.get("detail"),
            Some(&json!("directive detail"))
        );
        assert!(worker_event.payload.get("event_class").is_none());
        assert_eq!(
            prepared.event.data.pointer("/uaa_event/event_class"),
            Some(&json!("control_ack"))
        );

        let codex_assistant_reply = AgentWrapperEvent {
            agent_kind: agent_api::AgentWrapperKind::new("codex").expect("agent kind"),
            kind: AgentWrapperEventKind::TextOutput,
            channel: Some("assistant".to_string()),
            text: Some("assistant reply".to_string()),
            message: None,
            data: Some(json!({
                "thread_id": "thread-submitted"
            })),
        };
        let prepared = agent_event_from_wrapper_event(
            &sample_member_stream_context(Some("wwm_018f0f3a-9b2c-7def-8abc-0123456789ac")),
            &sample_world_binding(),
            "spn_submitted",
            codex_assistant_reply,
            &mut false,
            MemberStreamMode::SubmittedTurn,
            "codex_world",
        )
        .expect("Codex assistant TextOutput is a reply when its thread is authenticated");
        assert_eq!(
            prepared
                .worker_event_facet
                .expect("typed worker event facet")
                .event_class,
            WorldWorkerEventClassV1::Reply
        );
    }

    #[test]
    fn bootstrap_completion_with_session_handle_emits_registered_then_exit() {
        let mut emitted_registered = false;
        let mut producer = RuntimeEventStreamProducer::new();
        let start = producer.start("spn_bootstrap".to_string()).expect("Start");
        let frames = frames_from_completion(
            &sample_stream_context(),
            &sample_world_binding(),
            "spn_bootstrap",
            Ok(AgentWrapperCompletion {
                status: std::process::Command::new("sh")
                    .arg("-c")
                    .arg("exit 0")
                    .status()
                    .expect("completion status"),
                final_text: None,
                data: Some(json!({
                    "schema": SESSION_HANDLE_SCHEMA_V1,
                    "session": {
                        "id": "uaa_session"
                    }
                })),
            }),
            None,
            &mut emitted_registered,
            MemberStreamMode::Bootstrap,
            "codex_world",
            &mut producer,
        )
        .expect("completion frames");

        assert_eq!(
            frames.len(),
            2,
            "bootstrap with continuity metadata must surface registration before exit"
        );
        assert!(matches!(
            frames.first(),
            Some(ExecuteStreamFrame::Event { event, .. }) if event.kind == AgentEventKind::Registered
        ));
        assert!(matches!(
            frames.get(1),
            Some(ExecuteStreamFrame::Exit { exit: 0, .. })
        ));
        assert!(
            emitted_registered,
            "registered event tracking must be updated"
        );

        let start_identity = match start {
            ExecuteStreamFrame::Start { frame_identity, .. } => frame_identity,
            other => panic!("unexpected frame: {other:?}"),
        };
        let (event_frame_identity, event_identity) = match &frames[0] {
            ExecuteStreamFrame::Event {
                frame_identity,
                event,
            } => (
                frame_identity,
                event.event_identity.as_ref().expect("event identity"),
            ),
            other => panic!("unexpected frame: {other:?}"),
        };
        let (exit_frame_identity, exit_event_identity, terminal_identity) = match &frames[1] {
            ExecuteStreamFrame::Exit {
                frame_identity,
                event_identity,
                terminal_identity,
                ..
            } => (frame_identity, event_identity, terminal_identity),
            other => panic!("unexpected frame: {other:?}"),
        };
        assert_eq!(start_identity.frame_sequence, 1);
        assert_eq!(event_frame_identity.frame_sequence, 2);
        assert_eq!(exit_frame_identity.frame_sequence, 3);
        assert_eq!(event_identity.event_sequence, 1);
        assert_eq!(exit_event_identity.event_sequence, 2);
        assert_eq!(event_frame_identity.stream_id, start_identity.stream_id);
        assert_eq!(exit_frame_identity.stream_id, start_identity.stream_id);
        assert!(terminal_identity.matches_event(exit_event_identity));
        assert!(producer
            .transport_error("late transport error".to_string())
            .is_err());
    }

    #[test]
    fn retained_slot_owner_mismatch_error_mentions_expected_participant() {
        let err = retained_slot_owner_mismatch_error(
            &RetainedMemberKey {
                orchestration_session_id: "orch_123".to_string(),
                world_generation: 7,
                backend_id: "cli:codex".to_string(),
            },
            &BTreeSet::from(["ash_member_existing".to_string()]),
            "ash_member_other",
        );

        assert!(
            err.to_string()
                .contains("member_turn_submit.participant_id mismatch"),
            "unexpected error: {err}"
        );
        assert!(
            err.to_string()
                .contains("expected ash_member_existing, got ash_member_other"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn find_submit_target_rejects_participant_id_drift_for_retained_slot() {
        let manager = MemberRuntimeManager::new();
        let retained_key = RetainedMemberKey {
            orchestration_session_id: "orch_123".to_string(),
            world_generation: 7,
            backend_id: "cli:codex".to_string(),
        };
        manager
            .active_members
            .write()
            .expect("member runtime registry lock poisoned")
            .by_retained_key
            .insert(
                retained_key,
                RetainedMemberSlot::new("ash_member_existing".to_string()),
            );

        let mut req = sample_submit_turn_request();
        req.participant_id = "ash_member_other".to_string();

        let err = match manager.find_submit_target(&req) {
            Ok(_) => panic!("participant drift should be rejected"),
            Err(err) => err,
        };
        assert!(
            err.to_string()
                .contains("member_turn_submit.participant_id mismatch"),
            "unexpected error: {err}"
        );
        assert!(
            err.to_string()
                .contains("expected ash_member_existing, got ash_member_other"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn validate_submit_target_slot_accepts_retained_fork_child_in_shared_slot() {
        let manager = MemberRuntimeManager::new();
        manager
            .active_members
            .write()
            .expect("member runtime registry lock poisoned")
            .by_retained_key
            .insert(
                RetainedMemberKey {
                    orchestration_session_id: "orch_123".to_string(),
                    world_generation: 7,
                    backend_id: "cli:codex".to_string(),
                },
                RetainedMemberSlot {
                    primary_participant_id: "ash_member_source".to_string(),
                    fork_child_participant_id: Some("ash_member_child".to_string()),
                },
            );

        let mut req = sample_submit_turn_request();
        req.participant_id = "ash_member_child".to_string();

        manager
            .validate_submit_target_slot(&req)
            .expect("source plus direct fork child should accept the exact participant");
    }

    #[test]
    fn finish_bootstrap_preserves_retained_slot_when_session_handle_exists() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let active = sample_active_member_runtime(&temp_dir, "ash_member", "spn_bootstrap");
        active.remember_uaa_session_id("uaa_session".to_string());
        let manager = MemberRuntimeManager::new();
        manager
            .register_member(active.clone())
            .expect("register retained member");

        manager.finish_bootstrap(&active.participant_id, true);

        let guard = manager
            .active_members
            .read()
            .expect("member runtime registry lock poisoned");
        assert!(
            guard.by_participant_id.contains_key(&active.participant_id),
            "retained member should survive clean bootstrap exit once resumable identity exists"
        );
        assert_eq!(
            guard
                .by_retained_key
                .get(&RetainedMemberKey::from_active(active.as_ref())),
            Some(&RetainedMemberSlot::new(active.participant_id.clone())),
            "retained slot ownership should stay exact after bootstrap cleanup"
        );
        drop(guard);
        assert!(
            active.bootstrap.lock().expect("bootstrap lock").is_none(),
            "preserved retained member should infer parked truth by clearing the active bootstrap slot"
        );
        assert!(
            temp_dir.path().join("launcher").exists(),
            "parked retained workers must keep launcher artifacts for later submit_turn resume"
        );
        assert!(
            temp_dir.path().join("launcher").join("codex-home").exists(),
            "parked retained workers must keep isolated CODEX_HOME for later resume"
        );
    }

    #[tokio::test]
    async fn test_initial_retirement_refuses_unowned_scope_without_removing_member() {
        let temp = tempfile::tempdir().unwrap();
        let active = sample_active_member_runtime(&temp, "ash_member", "spn_bootstrap");
        let manager = MemberRuntimeManager::new();
        manager.register_member(Arc::clone(&active)).unwrap();
        let context = sample_stream_context();
        let (mut producer, _) = start_member_runtime_stream(
            &context,
            MemberStreamMode::Bootstrap,
            "spn_bootstrap".into(),
        )
        .unwrap();
        assert!(manager
            .prepare_failed_ordinary_initial_bootstrap_frames(
                &active,
                &context,
                &active.binding,
                "spn_bootstrap",
                130,
                &mut producer
            )
            .await
            .is_err());
        let registry = manager.active_members.read().unwrap();
        assert!(registry
            .by_participant_id
            .get(&active.participant_id)
            .is_some_and(|actual| Arc::ptr_eq(actual, &active)));
        assert!(active.bootstrap.lock().unwrap().is_some());
        assert!(active.launcher_dir.exists());
    }

    #[test]
    fn test_history_release_observation_is_owned_and_does_not_grant_store_reuse() {
        let shared = Arc::new(OrdinaryNativeHistoryStateV1::default());
        let child_use = OrdinaryNativeHistoryUseV1::acquire(Arc::clone(&shared)).unwrap();
        let child_closed = child_use.closed_observation();
        assert!(!child_closed.is_closed());
        assert!(OrdinaryNativeHistoryUseV1::acquire(Arc::clone(&shared)).is_err());
        drop(child_use);
        let parent_use = OrdinaryNativeHistoryUseV1::acquire(Arc::clone(&shared)).unwrap();
        assert!(
            child_closed.is_closed(),
            "another worker's use cannot erase this owner's actual release"
        );
        let parent_closed = parent_use.closed_observation();
        assert!(!parent_closed.is_closed());
        shared.poison();
        drop(parent_use);
        assert!(parent_closed.is_closed());
        assert!(
            OrdinaryNativeHistoryUseV1::acquire(shared).is_err(),
            "release data cannot turn poisoned custody into reuse permission"
        );
    }

    #[test]
    fn finish_bootstrap_unregisters_member_without_resumable_identity() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let active = sample_active_member_runtime(&temp_dir, "ash_member", "spn_bootstrap");
        let manager = MemberRuntimeManager::new();
        manager
            .register_member(active.clone())
            .expect("register retained member");

        manager.finish_bootstrap(&active.participant_id, false);

        let guard = manager
            .active_members
            .read()
            .expect("member runtime registry lock poisoned");
        assert!(
            !guard.by_participant_id.contains_key(&active.participant_id),
            "bootstrap without resumable identity must still close out retained continuity"
        );
        assert!(
            !guard
                .by_retained_key
                .contains_key(&RetainedMemberKey::from_active(active.as_ref())),
            "retained slot must be removed when continuity was never surfaced"
        );
        drop(guard);
        assert!(
            active.bootstrap.lock().expect("bootstrap lock").is_none(),
            "terminal bootstrap cleanup should also clear the active bootstrap slot"
        );
        assert!(
            !temp_dir.path().join("launcher").exists(),
            "terminal bootstrap cleanup should still remove launcher artifacts"
        );
    }

    #[test]
    fn test_retained_bootstrap_preservation_requires_origin_session_and_success() {
        use std::os::unix::process::ExitStatusExt;

        let success = std::process::ExitStatus::from_raw(0);
        let failure = std::process::ExitStatus::from_raw(1 << 8);
        let signalled = std::process::ExitStatus::from_raw(libc::SIGTERM);
        let legacy_dir = tempfile::tempdir().expect("legacy fixture");
        let legacy = sample_active_member_runtime(&legacy_dir, "ash_member", "spn_bootstrap");
        assert!(!should_preserve_retained_member_after_clean_bootstrap(
            legacy.as_ref(),
            &success,
        ));
        legacy.remember_uaa_session_id("legacy-session".to_string());
        assert!(should_preserve_retained_member_after_clean_bootstrap(
            legacy.as_ref(),
            &success,
        ));
        for status in [&failure, &signalled] {
            assert!(!should_preserve_retained_member_after_clean_bootstrap(
                legacy.as_ref(),
                status,
            ));
        }

        // A plausible retained prefix and structural E2 carrier are not authority.
        // These data-only fixtures deliberately have no held ordinary effect.
        for participant in ["rwp_member", "awm_task"] {
            let temp = tempfile::tempdir().expect("unheld fixture");
            let active = sample_e2_active_member_runtime(&temp, participant, "spn_bootstrap");
            active.remember_uaa_session_id("unheld-session".to_string());
            assert!(!should_preserve_retained_member_after_clean_bootstrap(
                active.as_ref(),
                &success,
            ));
        }
    }

    #[test]
    fn finish_bootstrap_unregisters_ephemeral_member_even_with_resumable_identity() {
        use std::os::unix::process::ExitStatusExt;

        let temp_dir = tempfile::tempdir().expect("temp dir");
        let active = sample_active_member_runtime(&temp_dir, "awm_task", "spn_bootstrap");
        active.remember_uaa_session_id("uaa_session".to_string());
        let manager = MemberRuntimeManager::new();
        manager
            .register_member(active.clone())
            .expect("register ephemeral member");

        manager.finish_bootstrap(
            &active.participant_id,
            should_preserve_retained_member_after_clean_bootstrap(
                active.as_ref(),
                &std::process::ExitStatus::from_raw(0),
            ),
        );

        let guard = manager
            .active_members
            .read()
            .expect("member runtime registry lock poisoned");
        assert!(
            !guard.by_participant_id.contains_key(&active.participant_id),
            "ephemeral run_world_task members must not remain retained after bootstrap exit"
        );
        assert!(
            !guard
                .by_retained_key
                .contains_key(&RetainedMemberKey::from_active(active.as_ref())),
            "ephemeral run_world_task members must not occupy retained slot identity"
        );
        drop(guard);
        assert!(
            !temp_dir.path().join("launcher").exists(),
            "ephemeral bootstrap cleanup should still remove launcher artifacts"
        );
    }

    #[test]
    fn register_member_allows_direct_fork_child_in_same_slot() {
        let source_dir = tempfile::tempdir().expect("source temp dir");
        let child_dir = tempfile::tempdir().expect("child temp dir");
        let source = sample_active_member_runtime(&source_dir, "ash_member_source", "spn_source");
        let child = sample_active_member_runtime_with_lineage(
            &child_dir,
            "ash_member_child",
            "spn_child",
            Some("ash_member_source"),
            None,
        );
        let manager = MemberRuntimeManager::new();

        manager
            .register_member(source.clone())
            .expect("register source retained member");
        manager
            .register_member(child.clone())
            .expect("register child retained member in shared slot");

        let guard = manager
            .active_members
            .read()
            .expect("member runtime registry lock poisoned");
        assert!(
            guard.by_participant_id.contains_key(&source.participant_id),
            "source retained member should stay registered"
        );
        assert!(
            guard.by_participant_id.contains_key(&child.participant_id),
            "child retained member should stay registered"
        );
        assert_eq!(
            guard
                .by_retained_key
                .get(&RetainedMemberKey::from_active(source.as_ref())),
            Some(&RetainedMemberSlot {
                primary_participant_id: source.participant_id.clone(),
                fork_child_participant_id: Some(child.participant_id.clone()),
            }),
            "shared retained slot should only track the source plus its direct fork child"
        );
    }

    #[test]
    fn test_register_member_rejects_successor_caller_without_genuine_fork_source() {
        // These are deliberately unissued registry fixtures. Parent-like
        // metadata and a claimed Fork activation cannot substitute for the
        // ordinary path's genuine pinned source across controller successors.
        for claimed_kind in [
            None,
            Some(E2MemberLaunchKindV1::FreshSpawn),
            Some(E2MemberLaunchKindV1::Fork),
        ] {
            let source_dir = tempfile::tempdir().expect("source temp dir");
            let child_dir = tempfile::tempdir().expect("child temp dir");
            let source = sample_active_member_runtime(&source_dir, "rwp_source", "spn_source");
            let mut child = sample_active_member_runtime_with_lineage(
                &child_dir,
                "ash_child",
                "spn_child",
                Some("rwp_source"),
                None,
            );
            let child_mut = Arc::get_mut(&mut child).expect("unique child fixture");
            child_mut.orchestrator_participant_id = "ash_successor_controller".into();
            if let Some(kind) = claimed_kind {
                let mut activation = sample_e2_launch_activation("ash_child", "rwr_child");
                activation.launch_kind = kind;
                activation.source_participant_id = Some("rwp_source".into());
                child_mut.e2_launch_activation = Some(activation);
            }
            let manager = MemberRuntimeManager::new();
            manager.register_member(source).expect("register source");
            let error = manager.register_member(child).expect_err(
                "a successor caller needs the actual ordinary authenticated source pin",
            );
            assert!(error.to_string().contains("retained member slot conflict"));
            let guard = manager.active_members.read().expect("registry");
            assert_eq!(guard.by_participant_id.len(), 1);
            assert!(!guard.by_participant_id.contains_key("ash_child"));
        }
    }

    #[test]
    fn register_member_rejects_unrelated_duplicate_in_same_slot() {
        let source_dir = tempfile::tempdir().expect("source temp dir");
        let duplicate_dir = tempfile::tempdir().expect("duplicate temp dir");
        let source = sample_active_member_runtime(&source_dir, "ash_member_source", "spn_source");
        let duplicate =
            sample_active_member_runtime(&duplicate_dir, "ash_member_duplicate", "spn_duplicate");
        let manager = MemberRuntimeManager::new();

        manager
            .register_member(source)
            .expect("register source retained member");
        let err = match manager.register_member(duplicate) {
            Ok(_) => panic!("unrelated same-slot retained registration should fail closed"),
            Err(err) => err,
        };

        assert!(
            err.to_string()
                .contains("member_dispatch retained member slot conflict"),
            "unexpected error: {err}"
        );
        assert!(
            err.to_string().contains(
                "only a direct fork child of the current retained participant may co-register"
            ),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn register_member_replaces_stale_fork_child_in_same_slot() {
        let source_dir = tempfile::tempdir().expect("source temp dir");
        let stale_child_dir = tempfile::tempdir().expect("stale child temp dir");
        let replacement_child_dir = tempfile::tempdir().expect("replacement child temp dir");
        let source = sample_active_member_runtime(&source_dir, "ash_member_source", "spn_source");
        let stale_child = sample_active_member_runtime_with_lineage(
            &stale_child_dir,
            "ash_member_child_stale",
            "spn_child_stale",
            Some("ash_member_source"),
            None,
        );
        let replacement_child = sample_active_member_runtime_with_lineage(
            &replacement_child_dir,
            "ash_member_child_replacement",
            "spn_child_replacement",
            Some("ash_member_source"),
            None,
        );
        let manager = MemberRuntimeManager::new();

        manager
            .register_member(source.clone())
            .expect("register source retained member");
        manager
            .register_member(stale_child.clone())
            .expect("register stale child retained member");

        stale_child.close_bootstrap();

        manager
            .register_member(replacement_child.clone())
            .expect("stale child should not block replacement registration");

        let guard = manager
            .active_members
            .read()
            .expect("member runtime registry lock poisoned");
        assert!(
            !guard
                .by_participant_id
                .contains_key(&stale_child.participant_id),
            "stale child should be evicted from active members"
        );
        assert!(
            guard
                .by_participant_id
                .contains_key(&replacement_child.participant_id),
            "replacement child should become the retained fork child"
        );
        assert_eq!(
            guard
                .by_retained_key
                .get(&RetainedMemberKey::from_active(source.as_ref())),
            Some(&RetainedMemberSlot {
                primary_participant_id: source.participant_id.clone(),
                fork_child_participant_id: Some(replacement_child.participant_id.clone()),
            }),
            "shared retained slot should track the replacement child after stale eviction"
        );
        assert!(
            !stale_child.launcher_dir.exists(),
            "stale child eviction should clean launcher artifacts"
        );
    }

    #[test]
    fn register_member_replaces_missing_stale_fork_child_in_same_slot() {
        let source_dir = tempfile::tempdir().expect("source temp dir");
        let missing_child_dir = tempfile::tempdir().expect("missing child temp dir");
        let replacement_child_dir = tempfile::tempdir().expect("replacement child temp dir");
        let source = sample_active_member_runtime(&source_dir, "ash_member_source", "spn_source");
        let missing_child = sample_active_member_runtime_with_lineage(
            &missing_child_dir,
            "ash_member_child_missing",
            "spn_child_missing",
            Some("ash_member_source"),
            None,
        );
        let replacement_child = sample_active_member_runtime_with_lineage(
            &replacement_child_dir,
            "ash_member_child_replacement",
            "spn_child_replacement",
            Some("ash_member_source"),
            None,
        );
        let manager = MemberRuntimeManager::new();
        let retained_key = RetainedMemberKey::from_active(source.as_ref());

        manager
            .register_member(source.clone())
            .expect("register source retained member");
        manager
            .register_member(missing_child.clone())
            .expect("register missing child retained member");

        {
            let mut guard = manager
                .active_members
                .write()
                .expect("member runtime registry lock poisoned");
            guard
                .by_participant_id
                .remove(&missing_child.participant_id)
                .expect("missing child should start in active members");
            assert_eq!(
                guard.by_retained_key.get(&retained_key),
                Some(&RetainedMemberSlot {
                    primary_participant_id: source.participant_id.clone(),
                    fork_child_participant_id: Some(missing_child.participant_id.clone()),
                }),
                "test setup should leave a stale fork-child slot behind"
            );
        }

        manager
            .register_member(replacement_child.clone())
            .expect("missing stale child should not block replacement registration");

        let guard = manager
            .active_members
            .read()
            .expect("member runtime registry lock poisoned");
        assert!(
            !guard
                .by_participant_id
                .contains_key(&missing_child.participant_id),
            "missing child should stay absent from active members"
        );
        assert!(
            guard
                .by_participant_id
                .contains_key(&replacement_child.participant_id),
            "replacement child should become the retained fork child"
        );
        assert_eq!(
            guard.by_retained_key.get(&retained_key),
            Some(&RetainedMemberSlot {
                primary_participant_id: source.participant_id.clone(),
                fork_child_participant_id: Some(replacement_child.participant_id.clone()),
            }),
            "shared retained slot should repair itself when the stale child entry is missing"
        );
    }

    #[test]
    fn register_member_rejects_second_live_fork_child_in_same_slot() {
        let source_dir = tempfile::tempdir().expect("source temp dir");
        let first_child_dir = tempfile::tempdir().expect("first child temp dir");
        let second_child_dir = tempfile::tempdir().expect("second child temp dir");
        let source = sample_active_member_runtime(&source_dir, "ash_member_source", "spn_source");
        let first_child = sample_active_member_runtime_with_lineage(
            &first_child_dir,
            "ash_member_child_first",
            "spn_child_first",
            Some("ash_member_source"),
            None,
        );
        let second_child = sample_active_member_runtime_with_lineage(
            &second_child_dir,
            "ash_member_child_second",
            "spn_child_second",
            Some("ash_member_source"),
            None,
        );
        let manager = MemberRuntimeManager::new();

        manager
            .register_member(source)
            .expect("register source retained member");
        manager
            .register_member(first_child)
            .expect("register first child retained member");
        let err = match manager.register_member(second_child) {
            Ok(_) => panic!("a live fork child should still block a second child"),
            Err(err) => err,
        };

        assert!(
            err.to_string()
                .contains("member_dispatch retained member slot conflict"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn unregister_member_preserves_direct_fork_child_slot_until_last_participant_leaves() {
        let source_dir = tempfile::tempdir().expect("source temp dir");
        let child_dir = tempfile::tempdir().expect("child temp dir");
        let source = sample_active_member_runtime(&source_dir, "ash_member_source", "spn_source");
        let child = sample_active_member_runtime_with_lineage(
            &child_dir,
            "ash_member_child",
            "spn_child",
            Some("ash_member_source"),
            None,
        );
        let manager = MemberRuntimeManager::new();

        manager
            .register_member(source.clone())
            .expect("register source retained member");
        manager
            .register_member(child.clone())
            .expect("register child retained member");

        manager.unregister_member(&source.participant_id);

        let retained_key = RetainedMemberKey::from_active(child.as_ref());
        {
            let guard = manager
                .active_members
                .read()
                .expect("member runtime registry lock poisoned");
            assert!(
                !guard.by_participant_id.contains_key(&source.participant_id),
                "source retained member should be removed"
            );
            assert!(
                guard.by_participant_id.contains_key(&child.participant_id),
                "child retained member should remain registered"
            );
            assert_eq!(
                guard.by_retained_key.get(&retained_key),
                Some(&RetainedMemberSlot::new(child.participant_id.clone())),
                "shared retained slot should promote the direct fork child once the source leaves"
            );
        }

        manager.unregister_member(&child.participant_id);

        let guard = manager
            .active_members
            .read()
            .expect("member runtime registry lock poisoned");
        assert!(
            !guard.by_retained_key.contains_key(&retained_key),
            "shared retained slot should clear once the last participant leaves"
        );
    }

    // Component ownership proof. The outer gate deliberately delays the service
    // acknowledgment; it is not an injected failure in the SDK's OS wait/stdio.
    // SDK hidden-startup/completion-before-ACK branches have separate SDK tests.
    #[test]
    fn test_service_route_disposal_while_native_ack_pending() {
        const CHILD: &str = "SUBSTRATE_SERVICE_ROUTE_ACK_CHILD";
        if std::env::var_os(CHILD).is_none() {
            let root = tempfile::tempdir().unwrap();
            fs::set_permissions(root.path(), fs::Permissions::from_mode(0o700)).unwrap();
            let output = std::process::Command::new(std::env::current_exe().unwrap())
                .args([
                    "--exact",
                    "member_runtime::tests::test_service_route_disposal_while_native_ack_pending",
                    "--nocapture",
                    "--test-threads=1",
                ])
                .env(CHILD, "1")
                .env("SUBSTRATE_GATEWAY_RUNTIME_ROOT", root.path())
                .output()
                .unwrap();
            print!("{}", String::from_utf8_lossy(&output.stdout));
            eprint!("{}", String::from_utf8_lossy(&output.stderr));
            assert!(output.status.success());
            root.close().unwrap();
            return;
        }
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap()
            .block_on(service_route_ack_cases());
    }

    struct RouteCustodyObservation {
        history: std::sync::Weak<OrdinaryNativeWorkerHistoryV1>,
        lease: std::sync::Weak<HeldNonE3PrivilegedChildLeaseV1>,
        history_file: std::path::PathBuf,
        policy: std::path::PathBuf,
    }
    impl RouteCustodyObservation {
        fn assert_held(&self, exclusion: &Arc<E3PrivilegedChildExclusionV1>) {
            assert!(self.history.upgrade().is_some());
            assert!(self.lease.upgrade().is_some());
            assert!(self.history_file.is_file());
            assert!(self.policy.is_file());
            assert!(exclusion
                .acquire_e3_exclusive("route-component", 7)
                .is_err());
        }
        async fn assert_released(&self, exclusion: &Arc<E3PrivilegedChildExclusionV1>) {
            tokio::time::timeout(std::time::Duration::from_secs(3), async {
                while self.history.upgrade().is_some() || self.lease.upgrade().is_some() {
                    tokio::task::yield_now().await;
                }
            })
            .await
            .unwrap();
            assert!(!self.policy.exists());
            assert!(
                self.history_file.is_file(),
                "retired history is not deleted by route cleanup"
            );
            let mut e3 = exclusion
                .acquire_e3_exclusive("route-component", 7)
                .unwrap();
            e3.release_after_cleanup_v1().unwrap();
        }
    }
    struct RouteTestCustody {
        _history: Arc<OrdinaryNativeWorkerHistoryV1>,
        _lease: Arc<HeldNonE3PrivilegedChildLeaseV1>,
        _policy: PreparedSubmittedTurnPolicyLauncher,
    }
    fn route_test_custody(
        temp: &Path,
        exclusion: &Arc<E3PrivilegedChildExclusionV1>,
    ) -> (
        RouteTestCustody,
        RouteCustodyObservation,
        transport_api_types::OrdinaryGatewayRuntimeIdentityV1,
    ) {
        let launcher = temp.join("launcher");
        fs::create_dir(&launcher).unwrap();
        fs::set_permissions(&launcher, fs::Permissions::from_mode(0o700)).unwrap();
        let (selected, start, runtime) =
            ordinary_worker_component_fixture(transport_api_types::AgentHarnessFamilyV1::Codex);
        let history = Arc::new(
            OrdinaryNativeWorkerHistoryV1::create(&launcher, &start, &runtime, &selected).unwrap(),
        );
        let history_file = history.path.join("history");
        fs::write(&history_file, "retained actual history").unwrap();
        let policy = launcher.join("turn-policy.sh");
        fs::write(&policy, "component policy launcher").unwrap();
        let lease = Arc::new(exclusion.acquire_non_e3_child().unwrap());
        let observed = RouteCustodyObservation {
            history: Arc::downgrade(&history),
            lease: Arc::downgrade(&lease),
            history_file,
            policy: policy.clone(),
        };
        (
            RouteTestCustody {
                _history: history,
                _lease: lease,
                _policy: PreparedSubmittedTurnPolicyLauncher { path: policy },
            },
            observed,
            runtime,
        )
    }

    async fn service_route_ack_cases() {
        use crate::gateway_runtime::member_runtime_route_disposal_tests::RouteFixture;
        use std::{
            sync::atomic::{AtomicBool, Ordering},
            time::Duration,
        };
        for mode in [
            "cancel",
            "bootstrap-cancel",
            "drop",
            "deadline",
            "raw-error",
            "normal",
            "failed-control",
            "startup-delivery-drop",
        ] {
            for fail_ack in [false, true] {
                let temp = tempfile::tempdir().unwrap();
                let exclusion = E3PrivilegedChildExclusionV1::new_recovering().unwrap();
                exclusion.finish_recovery().unwrap();
                let gateway = RouteFixture::new(&exclusion);
                let (custody, observed, runtime) = route_test_custody(temp.path(), &exclusion);
                let open = Arc::new(AtomicBool::new(true));
                let route = OrdinaryNativeRouteDisposalV1 {
                    history_state: Arc::clone(&custody._history.in_use),
                    native_open: Arc::clone(&open),
                    gateway: Arc::clone(&gateway.lease),
                    runtime,
                };
                let binary = temp.path().join("native.sh");
                fs::write(
                    &binary,
                    r##"#!/bin/sh
set -eu
case "${1-}" in
 --version) printf 'codex 1.2.3\n'; exit 0 ;;
 --help) printf 'Usage: --add-dir --allow-dangerously-skip-permissions\n'; exit 0 ;;
 features) printf '{"features":["add_dir"]}\n'; exit 0 ;;
esac
cat >/dev/null
printf '%s\n' "$$" >> "$FIXTURE_DIR/native-started"
if [ "$FIXTURE_MODE" = wait ]; then exec /bin/sleep 30; fi
printf '{"type":"thread.started","thread_id":"route-component"}\n'
printf '{"type":"turn.completed","thread_id":"route-component"}\n'
"##,
                )
                .unwrap();
                fs::set_permissions(&binary, fs::Permissions::from_mode(0o700)).unwrap();
                let bridge = PromptFulfillmentBridge::for_member_backend(
                    &MemberRuntimeBackendKindV1::Codex,
                    binary,
                )
                .unwrap();
                let control = bridge
                    .run_control(AgentWrapperRunRequest {
                        prompt: "component route custody".into(),
                        working_dir: Some(temp.path().to_path_buf()),
                        timeout: (mode == "deadline").then_some(Duration::from_secs(2)),
                        env: BTreeMap::from([
                            ("FIXTURE_DIR".into(), temp.path().display().to_string()),
                            (
                                "FIXTURE_MODE".into(),
                                if matches!(mode, "normal" | "raw-error") {
                                    "finish"
                                } else {
                                    "wait"
                                }
                                .into(),
                            ),
                        ]),
                        extensions: BTreeMap::new(),
                    })
                    .await
                    .unwrap();
                let cancel = control.cancel.clone();
                let native_terminal = control.terminal.clone();
                let actual_waited = Arc::new(AtomicBool::new(false));
                let waited = Arc::clone(&actual_waited);
                let (release_ack, ack) = tokio::sync::oneshot::channel();
                let completion = control.handle.completion;
                let completion: agent_api::DynAgentWrapperCompletion = Box::pin(async move {
                    let raw = completion.await;
                    if mode == "raw-error" {
                        raw?;
                        Err(AgentWrapperError::Backend {
                            message: "original injected service failure".into(),
                        })
                    } else {
                        raw
                    }
                });
                let terminate = cancel.clone();
                let (completion, terminal) = crate::prompt_fulfillment::retain_native_completion(
                    completion,
                    Some(Box::pin(async move {
                        native_terminal.wait().await.require_checked()?;
                        waited.store(true, Ordering::Release);
                        ack.await.unwrap();
                        if fail_ack {
                            Err(AgentWrapperError::Backend {
                                message: "deliberately failed service ACK".into(),
                            })
                        } else {
                            Ok(())
                        }
                    })),
                    (),
                    move || terminate.cancel(),
                    None,
                    cancel.disposal_requested(),
                );
                let mut events = control.handle.events;
                let event_owner =
                    tokio::spawn(async move { while events.next().await.is_some() {} });
                let native_pid = tokio::time::timeout(Duration::from_secs(5), async {
                    loop {
                        match fs::read_to_string(temp.path().join("native-started")) {
                            Ok(contents) if contents.is_empty() || !contents.ends_with('\n') => {}
                            Ok(contents) => {
                                assert_eq!(
                                    contents.lines().count(),
                                    1,
                                    "duplicate native invocation"
                                );
                                let pid = contents
                                    .strip_suffix('\n')
                                    .unwrap()
                                    .parse::<i32>()
                                    .expect("native fixture must publish a complete valid PID");
                                assert!(pid > 0, "native fixture PID must be positive");
                                break pid;
                            }
                            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
                            Err(error) => panic!("read native fixture PID: {error}"),
                        }
                        tokio::task::yield_now().await;
                    }
                })
                .await
                .unwrap();
                let manager = MemberRuntimeManager::with_replay_and_e3_projection_service(
                    RuntimeReplayRegistry::default(),
                    None,
                    Arc::clone(&exclusion),
                );
                manager.register_turn(
                    "route-span".into(),
                    Arc::new(ActiveSubmittedTurn {
                        terminal: terminal.clone(),
                        ordinary_effect: None,
                        ordinary_native_home: None,
                        participant_id: "structural-route-worker".into(),
                        cancel: cancel.clone(),
                        last_signal: Mutex::new(None),
                    }),
                );
                assert!(!manager.cancel("different-span", "TERM").unwrap());
                let terminate = cancel.clone();
                let mut completion = if mode == "startup-delivery-drop" {
                    let (release_control, held_control) = tokio::sync::oneshot::channel();
                    let retained_terminal = terminal.clone();
                    let retained_cancel = cancel.clone();
                    let delivery = retain_ordinary_native_startup(
                        async move {
                            held_control.await.unwrap();
                            Ok(PromptFulfillmentRunControl {
                                handle: agent_api::AgentWrapperRunHandle {
                                    events: Box::pin(futures_util::stream::empty()),
                                    completion,
                                },
                                cancel: retained_cancel,
                                terminal: retained_terminal,
                            })
                        },
                        route,
                        custody,
                        None,
                    );
                    drop(delivery);
                    gateway.assert_disposed().await;
                    assert!(!open.load(Ordering::Acquire));
                    observed.assert_held(&exclusion);
                    release_control.send(()).unwrap();
                    None
                } else {
                    Some(retain_ordinary_native_completion(
                        completion,
                        route,
                        terminal.clone(),
                        custody,
                        move || terminate.cancel(),
                        (mode == "deadline")
                            .then(|| tokio::time::Instant::now() + Duration::from_millis(20)),
                    ))
                };
                let bootstrap_temp = tempfile::tempdir().unwrap();
                let bootstrap = sample_active_member_runtime(
                    &bootstrap_temp,
                    "bootstrap-worker",
                    "bootstrap-span",
                );
                if mode == "bootstrap-cancel" {
                    *bootstrap.bootstrap.lock().unwrap() = Some(ActiveBootstrapRuntime {
                        span_id: "bootstrap-span".into(),
                        terminal: Some(terminal.clone()),
                        cancel: Some(cancel.clone()),
                        last_signal: None,
                        initial_control: None,
                    });
                    manager.register_member(Arc::clone(&bootstrap)).unwrap();
                    assert!(manager.cancel("bootstrap-span", "TERM").unwrap());
                } else if mode == "cancel" {
                    assert!(manager.cancel("route-span", "TERM").unwrap());
                } else if mode == "drop" {
                    drop(completion.take());
                } else if mode == "failed-control" {
                    let completion = completion.take().unwrap();
                    let (raw_sender, raw_receiver) = tokio::sync::oneshot::channel();
                    let terminal_for_failure = terminal.clone();
                    let cancel_for_failure = cancel.clone();
                    tokio::spawn(async move {
                        let error = join_failed_control(
                            anyhow!("original registration failure"),
                            agent_api::AgentWrapperRunHandle {
                                events: Box::pin(futures_util::stream::empty()),
                                completion,
                            },
                            cancel_for_failure,
                            terminal_for_failure,
                        )
                        .await;
                        let _ = raw_sender.send(error);
                    });
                    // Observe route/custody before releasing the same held ACK.
                    gateway.assert_disposed().await;
                    assert!(!open.load(Ordering::Acquire));
                    observed.assert_held(&exclusion);
                    let mut raw_receiver = raw_receiver;
                    assert!(
                        tokio::time::timeout(Duration::from_millis(10), &mut raw_receiver)
                            .await
                            .is_err()
                    );
                    release_ack.send(()).unwrap();
                    let error = raw_receiver.await.unwrap();
                    let message = format!("{error:#}");
                    assert!(message.contains("original registration failure"));
                    assert_eq!(
                        message.contains("native acknowledgment unavailable"),
                        fail_ack
                    );
                    observed.assert_released(&exclusion).await;
                    event_owner.await.unwrap();
                    manager.unregister_turn("route-span");
                    continue;
                }
                gateway.assert_disposed().await;
                assert!(
                    !open.load(Ordering::Acquire),
                    "exact per-turn admission is closed before ACK"
                );
                observed.assert_held(&exclusion);
                let mut checked = Box::pin(terminal.wait());
                assert!(
                    tokio::time::timeout(Duration::from_millis(10), &mut checked)
                        .await
                        .is_err()
                );
                if let Some(completion) = &mut completion {
                    assert!(tokio::time::timeout(Duration::from_millis(10), completion)
                        .await
                        .is_err());
                }
                // The actual SDK ACK remains separate from our deliberately held
                // service ACK. Wait for genuine checked SDK ownership before releasing it.
                tokio::time::timeout(Duration::from_secs(5), async {
                    while !actual_waited.load(Ordering::Acquire) {
                        tokio::task::yield_now().await;
                    }
                })
                .await
                .unwrap();
                observed.assert_held(&exclusion);
                release_ack.send(()).unwrap();
                let checked = checked.await;
                assert_eq!(checked.require_checked().is_err(), fail_ack);
                if let Some(completion) = completion {
                    let raw = completion.await;
                    if matches!(mode, "cancel" | "bootstrap-cancel") {
                        assert!(
                            matches!(&raw, Err(AgentWrapperError::Backend { message }) if message == "cancelled")
                        );
                    } else if mode == "raw-error" {
                        assert!(
                            matches!(&raw, Err(AgentWrapperError::Backend { message }) if message == "original injected service failure")
                        );
                    } else if mode == "normal" {
                        assert!(raw.as_ref().unwrap().status.success());
                    } else {
                        assert!(raw.is_err());
                    }
                    let enclosing = checked.enclosing_result(raw);
                    if fail_ack {
                        assert!(
                            matches!(enclosing, Err(AgentWrapperError::Backend { message }) if message.contains("submission_outcome_unknown") && message.contains("deliberately failed service ACK"))
                        );
                    }
                }
                observed.assert_released(&exclusion).await;
                event_owner.await.unwrap();
                assert_ne!(unsafe { libc::kill(native_pid, 0) }, 0);
                assert_eq!(
                    fs::read_to_string(temp.path().join("native-started"))
                        .unwrap()
                        .lines()
                        .count(),
                    1,
                    "accepted native input must not be delivered again"
                );
                manager.unregister_turn("route-span");
                if mode == "bootstrap-cancel" {
                    manager.unregister_member("bootstrap-worker");
                }
                eprintln!("service route case={mode} ack_failed={fail_ack}: real gateway disposed before synthetic service ACK; actual SDK wait separately checked");
            }
        }
        service_route_startup_cases().await;
    }

    async fn service_route_startup_cases() {
        use crate::gateway_runtime::member_runtime_route_disposal_tests::RouteFixture;
        use std::{
            sync::atomic::{AtomicBool, Ordering},
            time::Duration,
        };
        for mode in ["startup-error", "startup-drop", "startup-deadline"] {
            let temp = tempfile::tempdir().unwrap();
            let exclusion = E3PrivilegedChildExclusionV1::new_recovering().unwrap();
            exclusion.finish_recovery().unwrap();
            let gateway = RouteFixture::new(&exclusion);
            let (custody, observed, runtime) = route_test_custody(temp.path(), &exclusion);
            let open = Arc::new(AtomicBool::new(true));
            let route = OrdinaryNativeRouteDisposalV1 {
                history_state: Arc::clone(&custody._history.in_use),
                native_open: Arc::clone(&open),
                gateway: Arc::clone(&gateway.lease),
                runtime,
            };
            let (release, held) = tokio::sync::oneshot::channel();
            let waited = Arc::new(AtomicBool::new(false));
            let actual_waited = Arc::clone(&waited);
            let mut child = tokio::process::Command::new("/bin/sleep")
                .arg("0.01")
                .kill_on_drop(true)
                .spawn()
                .unwrap();
            let startup = async move {
                held.await.unwrap();
                assert!(child.wait().await.unwrap().success());
                actual_waited.store(true, Ordering::Release);
                Err(AgentWrapperError::Backend {
                    message: "original startup error".into(),
                })
            };
            let delivery = retain_ordinary_native_startup(
                startup,
                route,
                custody,
                (mode == "startup-deadline")
                    .then(|| tokio::time::Instant::now() + Duration::from_millis(10)),
            );
            let mut delivery = Box::pin(delivery);
            if mode == "startup-drop" {
                drop(delivery);
                gateway.assert_disposed().await;
                assert!(!open.load(Ordering::Acquire));
                observed.assert_held(&exclusion);
                assert!(!waited.load(Ordering::Acquire));
                release.send(()).unwrap();
            } else if mode == "startup-deadline" {
                gateway.assert_disposed().await;
                observed.assert_held(&exclusion);
                assert!(!open.load(Ordering::Acquire));
                assert!(
                    tokio::time::timeout(Duration::from_millis(10), &mut delivery)
                        .await
                        .is_err()
                );
                release.send(()).unwrap();
                assert!(
                    matches!(delivery.await, Err(AgentWrapperError::Backend { message }) if message == "original startup error")
                );
            } else {
                release.send(()).unwrap();
                assert!(
                    matches!(delivery.await, Err(AgentWrapperError::Backend { message }) if message == "original startup error")
                );
                gateway.assert_disposed().await;
            }
            observed.assert_released(&exclusion).await;
            assert!(waited.load(Ordering::Acquire));
        }
    }

    // Checked lifetime/registry seam only. No synthetic cap is admitted and no
    // claim is made about a vendor-native child's history format.
    #[test]
    fn test_initial_fork_source_registry_transfer_and_checked_stop_preserve_store() {
        let temp = tempfile::tempdir().unwrap();
        let manager = MemberRuntimeManager::new();
        let mut source = sample_active_member_runtime(&temp, "source-owner", "source-span");
        let (selected, start, runtime) =
            ordinary_worker_component_fixture(transport_api_types::AgentHarnessFamilyV1::Codex);
        let history = Arc::new(
            OrdinaryNativeWorkerHistoryV1::create(
                &source.launcher_dir,
                &start,
                &runtime,
                &selected,
            )
            .unwrap(),
        );
        Arc::get_mut(&mut source).unwrap().ordinary_native_history = Some(Arc::clone(&history));
        source.close_bootstrap();
        manager.register_member(Arc::clone(&source)).unwrap();
        let ownership = OrdinaryForkSourceOwnershipV1 {
            registry: Arc::downgrade(&manager.active_members),
            child_session: Mutex::new(None),
        };
        ownership.require_initial_source(&source).unwrap();
        *source.active_turn_span_id.lock().unwrap() = Some("concurrent-resume".into());
        assert!(ownership.require_initial_source(&source).is_err());
        *source.active_turn_span_id.lock().unwrap() = None;
        ownership.require_initial_source(&source).unwrap();
        // Physical store serialization independently refuses a concurrent writer.
        let child_store_lease =
            OrdinaryNativeHistoryUseV1::acquire(Arc::clone(&history.in_use)).unwrap();
        assert!(OrdinaryNativeHistoryUseV1::acquire(Arc::clone(&history.in_use)).is_err());
        let unconsumed = OrdinaryForkSourceOwnershipV1 {
            registry: Arc::downgrade(&manager.active_members),
            child_session: Mutex::new(None),
        };
        let child_temp = tempfile::tempdir().unwrap();
        let child = sample_active_member_runtime_with_lineage(
            &child_temp,
            "child-owner",
            "child-span",
            Some("source-owner"),
            None,
        );
        manager.register_member(Arc::clone(&child)).unwrap();
        ownership
            .observe_child_session("source-session", "child-session")
            .unwrap();
        assert!(ownership
            .observe_child_session("source-session", "other-child")
            .is_err());
        assert!(
            ownership.require_initial_source(&source).is_err(),
            "a consumed pin cannot admit a new Fork even while source remains registered"
        );
        manager.unregister_member("source-owner");
        assert!(
            unconsumed.require_initial_source(&source).is_err(),
            "a captured unconsumed pin must observe source retirement"
        );
        assert!(ownership.is_transferred().unwrap());
        assert!(manager
            .active_members
            .read()
            .unwrap()
            .by_participant_id
            .get("child-owner")
            .is_some_and(|actual| Arc::ptr_eq(actual, &child)));
        history.revalidate().unwrap();
        assert!(
            history.path.is_dir(),
            "checked source retirement retains physical history"
        );
        assert!(OrdinaryNativeHistoryUseV1::acquire(Arc::clone(&history.in_use)).is_err());
        drop(child_store_lease);
        drop(OrdinaryNativeHistoryUseV1::acquire(Arc::clone(&history.in_use)).unwrap());
        // A different retained object with the same textual ID is no substitute.
        let replacement_temp = tempfile::tempdir().unwrap();
        let replacement =
            sample_active_member_runtime(&replacement_temp, "source-owner", "replacement-span");
        replacement.close_bootstrap();
        manager
            .active_members
            .write()
            .unwrap()
            .by_participant_id
            .insert("source-owner".into(), replacement);
        assert!(unconsumed.require_initial_source(&source).is_err());
    }

    #[test]
    fn test_initial_fork_shared_history_unconfirmed_ack_refuses_reuse() {
        const CHILD: &str = "SUBSTRATE_FORK_HISTORY_ACK_CHILD";
        if std::env::var_os(CHILD).is_none() {
            let root = tempfile::tempdir().unwrap();
            fs::set_permissions(root.path(), fs::Permissions::from_mode(0o700)).unwrap();
            let output = std::process::Command::new(std::env::current_exe().unwrap())
                .args(["--exact", "member_runtime::tests::test_initial_fork_shared_history_unconfirmed_ack_refuses_reuse",
                    "--nocapture", "--test-threads=1"])
                .env(CHILD, "1").env("SUBSTRATE_GATEWAY_RUNTIME_ROOT", root.path()).output().unwrap();
            print!("{}", String::from_utf8_lossy(&output.stdout));
            eprint!("{}", String::from_utf8_lossy(&output.stderr));
            assert!(output.status.success());
            return;
        }
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap()
            .block_on(initial_fork_history_ack_cases());
    }

    async fn initial_fork_history_ack_cases() {
        use crate::gateway_runtime::member_runtime_route_disposal_tests::RouteFixture;
        for mode in ["checked", "native-wait-error", "unsupported"] {
            let temp = tempfile::tempdir().unwrap();
            let exclusion = E3PrivilegedChildExclusionV1::new_recovering().unwrap();
            exclusion.finish_recovery().unwrap();
            let gateway = RouteFixture::new(&exclusion);
            let (custody, _, runtime) = route_test_custody(temp.path(), &exclusion);
            let state = Arc::clone(&custody._history.in_use);
            let lease = OrdinaryNativeHistoryUseV1::acquire(Arc::clone(&state)).unwrap();
            let route = OrdinaryNativeRouteDisposalV1 {
                history_state: Arc::clone(&state),
                native_open: Arc::new(std::sync::atomic::AtomicBool::new(true)),
                gateway: Arc::clone(&gateway.lease),
                runtime,
            };
            let (release, ack) = tokio::sync::oneshot::channel();
            let settlement: Option<agent_api::DynAgentWrapperSettlement> = if mode == "unsupported"
            {
                None
            } else {
                Some(Box::pin(async move {
                    ack.await.unwrap();
                    if mode == "native-wait-error" {
                        Err(AgentWrapperError::Backend {
                            message: "native wait unconfirmed".into(),
                        })
                    } else {
                        Ok(())
                    }
                }))
            };
            let (completion, terminal) = crate::prompt_fulfillment::retain_native_completion(
                Box::pin(async {
                    Err(AgentWrapperError::Backend {
                        message: "original native error".into(),
                    })
                }),
                settlement,
                (),
                || {},
                None,
                None,
            );
            let completion = retain_ordinary_native_completion(
                completion,
                route,
                terminal.clone(),
                (custody, lease),
                || {},
                None,
            );
            tokio::pin!(completion);
            gateway.assert_disposed().await;
            if mode != "unsupported" {
                assert!(tokio::time::timeout(
                    std::time::Duration::from_millis(20),
                    &mut completion
                )
                .await
                .is_err());
                assert!(OrdinaryNativeHistoryUseV1::acquire(Arc::clone(&state)).is_err());
                release.send(()).unwrap();
            }
            let raw = completion.await;
            assert!(
                matches!(raw, Err(AgentWrapperError::Backend { message }) if message == "original native error")
            );
            let outcome = terminal.wait().await;
            assert_eq!(outcome.require_checked().is_ok(), mode == "checked");
            // No publisher/retirement has run: the failed/unsupported ACK itself
            // must poison the shared store before the enclosing custody drops.
            if mode == "checked" {
                drop(OrdinaryNativeHistoryUseV1::acquire(Arc::clone(&state)).unwrap());
            } else {
                assert!(state.require_usable().is_err());
                assert!(OrdinaryNativeHistoryUseV1::acquire(Arc::clone(&state)).is_err());
            }
        }
    }
}

// These fixtures construct identity data only. They do not create an ordinary
// configured-home seal, HSA permit, World effect or installed service authority.
#[cfg(test)]
fn ordinary_worker_component_fixture(
    harness: transport_api_types::AgentHarnessFamilyV1,
) -> (
    transport_api_types::ResolvedAgentSelectionV1,
    transport_api_types::OrdinaryStartSelectionCarrierV1,
    transport_api_types::OrdinaryGatewayRuntimeIdentityV1,
) {
    use transport_api_types::*;
    let source = AgentSelectionSourceV1 {
        scope: AgentSelectionSourceScopeV1::Global,
        root_identity: "component-root".into(),
        relative_path: "config.yaml".into(),
        revision: "component-revision".into(),
        sha256: "a".repeat(64),
    };
    let codex = harness == AgentHarnessFamilyV1::Codex;
    let provider = if codex { "openai" } else { "anthropic" };
    let upstream = if codex {
        "component-model"
    } else {
        "claude-sonnet-4-5-20250929"
    };
    let selected = ResolvedAgentSelectionV1 {
        version: 1,
        logical_agent_id: "component".into(),
        realized_agent_id: "component-world".into(),
        backend_id: "cli:component-world".into(),
        placement: AgentSelectionPlacementV1::World,
        harness_contract_revision: if codex {
            "codex-cli.v1"
        } else {
            "claude-code-cli.v1"
        }
        .into(),
        selection: AgentSelectionTupleV1 {
            harness,
            provider: provider.into(),
            connection_profile: "global:component".into(),
            model: upstream.into(),
        },
        connection_profile: AgentConnectionProfileV1 {
            id: "global:component".into(),
            revision: "component-profile".into(),
            provider: provider.into(),
            billing: if codex {
                AgentConnectionBillingV1::Subscription
            } else {
                AgentConnectionBillingV1::Api
            },
            endpoint_identity: "component-provider".into(),
            upstream_base_url: if codex {
                "https://chatgpt.com/backend-api"
            } else {
                "https://api.anthropic.com"
            }
            .into(),
            auth: if codex {
                AgentConnectionAuthV1::CodexAccount {
                    reference: "component-account".into(),
                }
            } else {
                AgentConnectionAuthV1::ApiKeyEnvironment {
                    name: "ANTHROPIC_API_KEY".into(),
                }
            },
            source: source.clone(),
        },
        routed_model_id: if codex { upstream } else { "claude-sonnet-4.5" }.into(),
        upstream_model_id: upstream.into(),
        route_revision: if codex {
            "codex-subscription.v1"
        } else {
            "claude-code-existing.v1"
        }
        .into(),
        selection_origin: AgentSelectionOriginV1::ConfiguredDefault,
        inventory_source: AgentSelectionSourceV1 {
            relative_path: "agents/component.yaml".into(),
            ..source.clone()
        },
        config_sources: vec![source.clone()],
        preference_source: Some(source),
        selection_digest: String::new(),
    }
    .seal()
    .expect("valid component selection data");
    let start = OrdinaryStartSelectionCarrierV1 {
        schema_version: 1,
        authority_store_id: "component-authority".into(),
        original_start_intent_id: "component-start".into(),
        original_start_issuer_request_id: format!("req_{}", "b".repeat(64)),
        original_session_id: "component-session".into(),
        context_sha256: "c".repeat(64),
        native_selection_sha256: "d".repeat(64),
        requested_selection_sha256: selected.selection_digest.clone(),
    };
    let runtime = OrdinaryGatewayRuntimeIdentityV1::WorldWorker {
        orchestration_session_id: "component-session".into(),
        caller_participant_id: "component-orchestrator".into(),
        participant_id: "component-worker".into(),
        backend_id: selected.backend_id.clone(),
        run_id: "component-bootstrap".into(),
        world_id: "component-world".into(),
        world_generation: 7,
    };
    (selected, start, runtime)
}

#[cfg(test)]
#[test]
fn test_ordinary_worker_history_reuses_exact_start_and_refuses_substitution() {
    use transport_api_types::{AgentHarnessFamilyV1, OrdinaryGatewayRuntimeIdentityV1};
    let temp = tempfile::tempdir().unwrap();
    let launcher = temp.path().join("launcher");
    fs::create_dir(&launcher).unwrap();
    fs::set_permissions(&launcher, fs::Permissions::from_mode(0o700)).unwrap();
    let (selected, start, runtime) = ordinary_worker_component_fixture(AgentHarnessFamilyV1::Codex);
    let original_start = serde_json::to_vec(&start).unwrap();
    let history =
        OrdinaryNativeWorkerHistoryV1::create(&launcher, &start, &runtime, &selected).unwrap();
    let original = history.directory.metadata().unwrap();
    let wiring = transport_api_types::GatewayClientWiringV1 {
        openai_base_url: "http://127.0.0.1:32123".into(),
        anthropic_base_url: "http://127.0.0.1:32123".into(),
    };
    let (name, config) = ordinary_native_config(temp.path(), &selected, &wiring).unwrap();
    history.write_config(name, &config).unwrap();
    let mut pids = BTreeSet::new();
    for index in 0..3 {
        let mut current = runtime.clone();
        if let OrdinaryGatewayRuntimeIdentityV1::WorldWorker { run_id, .. } = &mut current {
            *run_id = format!("component-turn-{index}");
        }
        history
            .require_original_worker(&start, &current, &selected)
            .unwrap();
        let output = std::process::Command::new("/bin/sh").arg("-c")
            .arg("set -eu; test -f \"$HOME/config.toml\"; printf '%s\\n' \"$COMPONENT_TURN\" >> \"$HOME/history\"; printf '%s' \"$$\"")
            .env("HOME", history.descriptor_path().unwrap()).env("COMPONENT_TURN", index.to_string()).output().unwrap();
        assert!(output.status.success());
        pids.insert(String::from_utf8(output.stdout).unwrap());
        let now = fs::metadata(&history.path).unwrap();
        assert_eq!((original.dev(), original.ino()), (now.dev(), now.ino()));
    }
    assert_eq!(pids.len(), 3);
    assert_eq!(
        fs::read_to_string(history.path.join("history")).unwrap(),
        "0\n1\n2\n"
    );
    let mut wrong_worker = runtime.clone();
    if let OrdinaryGatewayRuntimeIdentityV1::WorldWorker { participant_id, .. } = &mut wrong_worker
    {
        *participant_id = "different-worker".into();
    }
    assert!(history
        .require_original_worker(&start, &wrong_worker, &selected)
        .is_err());
    let host = OrdinaryGatewayRuntimeIdentityV1::HostActor {
        orchestration_session_id: "component-session".into(),
        participant_id: "component-worker".into(),
        backend_id: selected.backend_id.clone(),
        run_id: "component-turn".into(),
    };
    assert!(history
        .require_original_worker(&start, &host, &selected)
        .is_err());
    let mut wrong_start = start.clone();
    wrong_start.original_start_intent_id = "different-start".into();
    assert!(history
        .require_original_worker(&wrong_start, &runtime, &selected)
        .is_err());
    let mut wrong_selection = selected.clone();
    wrong_selection.selection_digest = "e".repeat(64);
    assert!(history
        .require_original_worker(&start, &runtime, &wrong_selection)
        .is_err());
    let moved = launcher.join("held-original-history");
    fs::rename(&history.path, &moved).unwrap();
    fs::create_dir(&history.path).unwrap();
    fs::set_permissions(&history.path, fs::Permissions::from_mode(0o700)).unwrap();
    assert!(history.revalidate().is_err());
    assert!(history.descriptor_path().is_err());
    assert_eq!(
        fs::read_to_string(moved.join("history")).unwrap(),
        "0\n1\n2\n"
    );
    assert_eq!(serde_json::to_vec(&history.start).unwrap(), original_start);
    fs::remove_dir(&history.path).unwrap();
    fs::rename(&moved, &history.path).unwrap();
    history.revalidate().unwrap();
    let moved_parent = temp.path().join("held-original-launcher");
    fs::rename(&launcher, &moved_parent).unwrap();
    fs::create_dir(&launcher).unwrap();
    assert!(history.revalidate().is_err());
    fs::remove_dir(&launcher).unwrap();
    fs::rename(&moved_parent, &launcher).unwrap();
    history.revalidate().unwrap();
    let path = history.path.clone();
    drop(history);
    assert!(path.join("history").is_file());
    let mut cleanup = Some(launcher.clone());
    cleanup_prepared_ordinary_member_launcher(&mut cleanup, true);
    assert!(cleanup.is_none());
    assert!(path.join("history").is_file());
}

#[cfg(test)]
#[test]
fn test_ordinary_managed_origins_are_strict() {
    assert_eq!(
        ordinary_native_managed_origin("http://127.0.0.1:32123").unwrap(),
        "http://127.0.0.1:32123"
    );
    for bad in [
        "http://127.0.0.1:0",
        "http://127.0.0.1:01",
        "http://127.0.0.1:+1",
        "http://127.0.0.1:65536",
        "http://127.0.0.1:1/",
        "http://127.0.0.1:1/v1",
        "http://127.0.0.1:1?x=1",
        "http://user@127.0.0.1:1",
        "https://127.0.0.1:1",
        "http://localhost:1",
        "http://[::1]:1",
    ] {
        assert!(ordinary_native_managed_origin(bad).is_err(), "{bad}");
    }
}

#[cfg(test)]
#[tokio::test(flavor = "current_thread")]
async fn test_ordinary_native_inherited_env_in_actual_codex_and_claude_children() {
    const CHILD: &str = "SUBSTRATE_ORDINARY_ENV_COMPONENT_CHILD";
    if std::env::var_os(CHILD).is_none() {
        // Synthetic tokens exist only in this isolated libtest child environment.
        let result = std::process::Command::new(std::env::current_exe().unwrap())
            .arg("test_ordinary_native_inherited_env_in_actual_codex_and_claude_children")
            .arg("--nocapture")
            .arg("--test-threads=1")
            .env(CHILD, "1")
            .env("OPENAI_API_KEY", "component-inherited-provider-secret")
            .env("ANTHROPIC_API_KEY", "component-inherited-provider-secret")
            .env(
                "SUBSTRATE_LLM_BACKEND_AUTH_CLI_CODEX_ACCESS_TOKEN",
                "component-inherited-provider-secret",
            )
            .output()
            .unwrap();
        assert!(
            result.status.success(),
            "{}\n{}",
            String::from_utf8_lossy(&result.stdout),
            String::from_utf8_lossy(&result.stderr)
        );
        return;
    }
    for harness in [
        transport_api_types::AgentHarnessFamilyV1::Codex,
        transport_api_types::AgentHarnessFamilyV1::ClaudeCode,
    ] {
        let temp = tempfile::tempdir().unwrap();
        let launcher = temp.path().join("launcher");
        fs::create_dir(&launcher).unwrap();
        fs::set_permissions(&launcher, fs::Permissions::from_mode(0o700)).unwrap();
        let (selected, start, runtime) = ordinary_worker_component_fixture(harness);
        let history =
            OrdinaryNativeWorkerHistoryV1::create(&launcher, &start, &runtime, &selected).unwrap();
        let wiring = transport_api_types::GatewayClientWiringV1 {
            openai_base_url: "http://127.0.0.1:32123".into(),
            anthropic_base_url: "http://127.0.0.1:32123".into(),
        };
        let (name, content) = ordinary_native_config(temp.path(), &selected, &wiring).unwrap();
        history.write_config(name, &content).unwrap();
        let binary = temp.path().join("fixture-native.sh");
        fs::write(&binary, r##"#!/bin/sh
set -eu
for value in "${OPENAI_API_KEY-}" "${ANTHROPIC_API_KEY-}" "${SUBSTRATE_LLM_BACKEND_AUTH_CLI_CODEX_ACCESS_TOKEN-}"; do
  if [ -n "$value" ]; then exit 88; fi
done
case "${1-}" in
  --version) printf '%s\n' version >> "$FIXTURE_DIR/probes"; printf 'codex 1.2.3\n'; exit 0 ;;
  --help) printf '%s\n' help >> "$FIXTURE_DIR/probes"; printf 'Usage: --add-dir --allow-dangerously-skip-permissions\n'; exit 0 ;;
  features) printf '%s\n' features >> "$FIXTURE_DIR/probes"; printf '{"features":["add_dir"]}\n'; exit 0 ;;
esac
printf '%s\n' "$@" > "$FIXTURE_DIR/args"
printf '%s\n%s\n%s\n' "$HOME" "${ANTHROPIC_BASE_URL-}" "${ANTHROPIC_MODEL-}" > "$FIXTURE_DIR/env"
if [ "$FIXTURE_BACKEND" = codex ]; then
  cat "$CODEX_HOME/config.toml" > "$FIXTURE_DIR/config"
  cat > "$FIXTURE_DIR/stdin"
  printf '{"type":"thread.started","thread_id":"thread-component"}\n'
  printf '{"type":"turn.completed","thread_id":"thread-component"}\n'
else
  cat "$CLAUDE_CONFIG_DIR/settings.json" > "$FIXTURE_DIR/config"
  test "$ANTHROPIC_AUTH_TOKEN" = substrate-managed-gateway
  printf '{"type":"system","subtype":"init","session_id":"thread-component"}\n'
  printf '{"type":"result","subtype":"success","is_error":false,"result":"component","session_id":"thread-component","duration_ms":1,"duration_api_ms":1,"num_turns":1,"total_cost_usd":0}\n'
fi
"##).unwrap();
        fs::set_permissions(&binary, fs::Permissions::from_mode(0o755)).unwrap();
        let mut env = BTreeMap::from([
            ("FIXTURE_DIR".into(), temp.path().display().to_string()),
            (
                "FIXTURE_BACKEND".into(),
                if harness == transport_api_types::AgentHarnessFamilyV1::Codex {
                    "codex"
                } else {
                    "claude"
                }
                .into(),
            ),
        ]);
        configure_ordinary_native_env(
            &mut env,
            &selected,
            &history.descriptor_path().unwrap(),
            &wiring,
        )
        .unwrap();
        assert_eq!(env.get("OPENAI_API_KEY").map(String::as_str), Some(""));
        assert_eq!(env.get("ANTHROPIC_API_KEY").map(String::as_str), Some(""));
        let backend = if harness == transport_api_types::AgentHarnessFamilyV1::Codex {
            MemberRuntimeBackendKindV1::Codex
        } else {
            MemberRuntimeBackendKindV1::ClaudeCode
        };
        let bridge = PromptFulfillmentBridge::for_member_backend(&backend, binary).unwrap();
        let mut extensions = member_runtime_workspace_access_extensions(backend, temp.path());
        extensions.insert(
            "agent_api.config.model.v1".into(),
            json!(selected.routed_model_id),
        );
        let control = bridge
            .run_control(AgentWrapperRunRequest {
                prompt: "component".into(),
                working_dir: Some(temp.path().to_path_buf()),
                timeout: None,
                env,
                extensions,
            })
            .await
            .unwrap();
        let mut events = control.handle.events;
        while events.next().await.is_some() {}
        let result = control.handle.completion.await.unwrap();
        assert!(result.status.success());
        let observed = fs::read_to_string(temp.path().join("env")).unwrap();
        assert!(observed.starts_with(&history.descriptor_path().unwrap().display().to_string()));
        let args = fs::read_to_string(temp.path().join("args")).unwrap();
        assert!(args.lines().any(|value| value == selected.routed_model_id));
        if harness == transport_api_types::AgentHarnessFamilyV1::Codex {
            assert!(args.lines().any(|value| value == "--add-dir"));
            assert!(!fs::read_to_string(temp.path().join("probes"))
                .unwrap()
                .is_empty());
            let config: toml::Value = fs::read_to_string(temp.path().join("config"))
                .unwrap()
                .parse()
                .unwrap();
            assert_eq!(
                config["model"].as_str(),
                Some(selected.routed_model_id.as_str())
            );
            assert_eq!(
                config["model_providers"]["substrate-managed-gateway"]["base_url"].as_str(),
                Some("http://127.0.0.1:32123/v1")
            );
        } else {
            assert!(observed.contains("\nhttp://127.0.0.1:32123\nclaude-sonnet-4.5\n"));
            assert_eq!(
                fs::read_to_string(temp.path().join("config")).unwrap(),
                "{}\n"
            );
        }
    }
}

#[cfg(test)]
#[tokio::test(flavor = "current_thread")]
async fn test_ordinary_world_completion_retains_started_child_custody() {
    use std::time::Duration;
    for mode in [
        "normal",
        "drop-after-returned-control-error",
        "explicit-cancel",
    ] {
        let temp = tempfile::tempdir().unwrap();
        let launcher = temp.path().join("launcher");
        fs::create_dir(&launcher).unwrap();
        fs::set_permissions(&launcher, fs::Permissions::from_mode(0o700)).unwrap();
        let (selected, start, runtime) =
            ordinary_worker_component_fixture(transport_api_types::AgentHarnessFamilyV1::Codex);
        let history = Arc::new(
            OrdinaryNativeWorkerHistoryV1::create(&launcher, &start, &runtime, &selected).unwrap(),
        );
        fs::write(history.path.join("history"), "retained").unwrap();
        let history_path = history.path.clone();
        let weak = Arc::downgrade(&history);
        let policy_path = launcher.join("turn-policy.sh");
        fs::write(&policy_path, "component policy launcher").unwrap();
        let policy = PreparedSubmittedTurnPolicyLauncher {
            path: policy_path.clone(),
        };
        let exclusion = E3PrivilegedChildExclusionV1::new_recovering().unwrap();
        exclusion.finish_recovery().unwrap();
        let lease = Arc::new(exclusion.acquire_non_e3_child().unwrap());
        let mut child = tokio::process::Command::new("/bin/sleep")
            .arg(if mode == "normal" { "0.08" } else { "30" })
            .kill_on_drop(true)
            .spawn()
            .unwrap();
        let pid = child.id().unwrap();
        assert_eq!(unsafe { libc::kill(pid as i32, 0) }, 0);
        let completion: agent_api::DynAgentWrapperCompletion = Box::pin(async move {
            let status = child
                .wait()
                .await
                .map_err(|error| AgentWrapperError::Backend {
                    message: error.to_string(),
                })?;
            Ok(AgentWrapperCompletion {
                status,
                final_text: None,
                data: None,
            })
        });
        let completion = retain_ordinary_world_completion(
            completion,
            (history, lease, Some(policy)),
            move || unsafe {
                libc::kill(pid as i32, libc::SIGTERM);
            },
        );
        assert!(weak.upgrade().is_some());
        assert!(policy_path.is_file());
        assert!(exclusion
            .acquire_e3_exclusive("component-world", 7)
            .is_err());
        let mut cleanup = Some(launcher.clone());
        cleanup_prepared_ordinary_member_launcher(&mut cleanup, true);
        assert!(history_path.is_dir());
        if mode == "drop-after-returned-control-error" {
            drop(completion);
            tokio::time::timeout(Duration::from_secs(3), async {
                while weak.upgrade().is_some() {
                    tokio::time::sleep(Duration::from_millis(5)).await;
                }
            })
            .await
            .unwrap();
        } else {
            if mode == "explicit-cancel" {
                unsafe {
                    libc::kill(pid as i32, libc::SIGTERM);
                }
            }
            let result = tokio::time::timeout(Duration::from_secs(3), completion)
                .await
                .unwrap()
                .unwrap();
            assert_eq!(result.status.success(), mode == "normal");
        }
        assert!(weak.upgrade().is_none());
        assert!(!policy_path.exists());
        assert_ne!(unsafe { libc::kill(pid as i32, 0) }, 0);
        assert!(history_path.join("history").is_file());
        let mut e3 = exclusion
            .acquire_e3_exclusive("component-world", 7)
            .unwrap();
        e3.release_after_cleanup_v1().unwrap();
    }
}

#[cfg(test)]
#[tokio::test]
async fn test_ordinary_home_handoff_supervisor_joins_after_owner_drop() {
    use std::sync::atomic::{AtomicBool, Ordering};
    let terminal = Arc::new(OrdinaryHomeHandoffTerminalV1::default());
    *terminal.latest.lock().unwrap() = Some(("dropped-startup".into(), None));
    let joined = Arc::new(AtomicBool::new(false));
    let observed = Arc::clone(&joined);
    let (stop_tx, stop_rx) = tokio::sync::oneshot::channel();
    let task = tokio::spawn(async move {
        let _ = stop_rx.await;
        observed.store(true, Ordering::SeqCst);
        Ok(false)
    });
    let owner = supervise_ordinary_home_handoff(
        task,
        stop_tx,
        Arc::clone(&terminal),
        "dropped-startup".into(),
    );
    drop(owner);
    tokio::time::timeout(std::time::Duration::from_secs(3), async {
        loop {
            let notified = terminal.changed.notified();
            tokio::pin!(notified);
            notified.as_mut().enable();
            if terminal
                .latest
                .lock()
                .unwrap()
                .as_ref()
                .is_some_and(|(_, v)| v.is_some())
            {
                break;
            }
            notified.await;
        }
    })
    .await
    .unwrap();
    assert!(
        joined.load(Ordering::SeqCst),
        "terminal record requires actual listener completion"
    );
    assert_eq!(
        *terminal.latest.lock().unwrap(),
        Some(("dropped-startup".into(), Some("joined_stopped")))
    );
}

#[cfg(test)]
#[test]
fn test_ordinary_home_handoff_codex_argv_is_closed() {
    let temp = tempfile::Builder::new()
        .prefix("ordinary-sdk-argv-")
        .tempdir_in("/tmp")
        .unwrap();
    let workspace = Path::new("/fixture/exact-workspace");
    let model = "held-model";
    for (index, args) in [
        vec!["--version"],
        vec!["features", "list", "--json"],
        vec!["features", "list"],
        vec!["--help"],
    ]
    .into_iter()
    .enumerate()
    {
        let args = args.into_iter().map(str::to_owned).collect::<Vec<_>>();
        assert_eq!(
            classify_ordinary_codex_args(&args, None, temp.path(), model, workspace).unwrap(),
            Some(index)
        );
        let mut injected = args;
        injected.push("exec".into());
        assert!(
            classify_ordinary_codex_args(&injected, None, temp.path(), model, workspace).is_err()
        );
    }
    let path = temp
        .path()
        .join(format!("codex_last_message_123_{}.txt", std::process::id()));
    let start: Vec<String> = vec![
        "--dangerously-bypass-approvals-and-sandbox",
        "exec",
        "--color",
        "never",
        "--skip-git-repo-check",
        "--json",
        "--model",
        model,
        "--add-dir",
        workspace.to_str().unwrap(),
        "--output-last-message",
        path.to_str().unwrap(),
    ]
    .into_iter()
    .map(str::to_owned)
    .collect();
    assert_eq!(
        classify_ordinary_codex_args(&start, None, temp.path(), model, workspace).unwrap(),
        None
    );
    let mut resume = start.clone();
    resume.extend(["resume".into(), "held-thread".into(), "-".into()]);
    assert_eq!(
        classify_ordinary_codex_args(&resume, Some("held-thread"), temp.path(), model, workspace)
            .unwrap(),
        None
    );
    assert!(classify_ordinary_codex_args(
        &resume,
        Some("other-thread"),
        temp.path(),
        model,
        workspace
    )
    .is_err());
    let mut cases = Vec::new();
    let mut duplicate_exec = resume.clone();
    duplicate_exec.insert(2, "exec".into());
    cases.push(duplicate_exec);
    let mut earlier_resume = resume.clone();
    earlier_resume.splice(2..2, ["resume".into(), "other-thread".into()]);
    cases.push(earlier_resume);
    let mut output_alias = resume.clone();
    output_alias.insert(2, "--output-last-message=/tmp/arbitrary".into());
    cases.push(output_alias);
    let mut wrong_model = resume.clone();
    wrong_model[7] = "substituted-model".into();
    cases.push(wrong_model);
    let mut wrong_workspace = resume.clone();
    wrong_workspace[9] = "/other/workspace".into();
    cases.push(wrong_workspace);
    for extra in [
        vec!["--profile", "ambient"],
        vec!["--cd", "/other"],
        vec!["--config", "model=other"],
        vec!["--model=other"],
        vec!["--last"],
        vec!["--fork-session"],
    ] {
        let mut args = resume.clone();
        args.splice(2..2, extra.into_iter().map(str::to_owned));
        cases.push(args);
    }
    for args in cases {
        assert!(
            classify_ordinary_codex_args(&args, Some("held-thread"), temp.path(), model, workspace)
                .is_err(),
            "injected argv passed: {args:?}"
        );
    }
    let mut wrong_output = start.clone();
    wrong_output[11] = "/tmp/arbitrary".into();
    assert!(
        classify_ordinary_codex_args(&wrong_output, None, temp.path(), model, workspace).is_err()
    );
    fs::write(path, b"preexisting output").unwrap();
    assert!(classify_ordinary_codex_args(&start, None, temp.path(), model, workspace).is_err());
}

#[cfg(test)]
#[test]
fn test_ordinary_home_handoff_claude_argv_is_closed() {
    let model = "claude-sonnet-4.5";
    let prompt = "exact ordinary Claude prompt";
    let start = [
        "--print",
        "--output-format",
        "stream-json",
        "--model",
        model,
        "--permission-mode",
        "bypassPermissions",
        "--include-partial-messages",
        "--verbose",
        prompt,
    ]
    .map(str::to_owned)
    .to_vec();
    classify_ordinary_claude_args(&start, None, model, prompt).unwrap();
    let mut resume = start.clone();
    resume.splice(8..8, ["--resume".into(), "held-session".into()]);
    classify_ordinary_claude_args(&resume, Some("held-session"), model, prompt).unwrap();
    assert!(classify_ordinary_claude_args(&resume, None, model, prompt).is_err());
    assert!(classify_ordinary_claude_args(&resume, Some("other-session"), model, prompt).is_err());
    assert!(classify_ordinary_claude_args(&start, None, "other-model", prompt).is_err());
    assert!(classify_ordinary_claude_args(&start, None, model, "other-prompt").is_err());
    for injection in [
        vec!["--continue"],
        vec!["--fork-session"],
        vec!["--resume", "other-session"],
        vec!["--model", "other-model"],
        vec!["--add-dir", "/other"],
        vec!["--settings", "/other"],
        vec!["--system-prompt", "other"],
        vec!["--dangerously-skip-permissions"],
        vec!["--help"],
    ] {
        let mut bad = start.clone();
        bad.splice(8..8, injection.into_iter().map(str::to_owned));
        assert!(classify_ordinary_claude_args(&bad, None, model, prompt).is_err());
    }
    for omitted in 0..start.len() {
        let mut bad = start.clone();
        bad.remove(omitted);
        assert!(classify_ordinary_claude_args(&bad, None, model, prompt).is_err());
    }
    for probe in [["--help"], ["--version"]] {
        let probe = probe.map(str::to_owned).to_vec();
        assert!(classify_ordinary_claude_args(&probe, None, model, prompt).is_err());
    }
}

#[cfg(test)]
mod initial_fork_join_tests {
    use super::*;

    #[test]
    fn test_fork_app_server_shape_does_not_expand_exec_admission() {
        assert_eq!(
            classify_ordinary_fork_codex_args(&["app-server".into()]).unwrap(),
            None
        );
        for args in [
            vec!["app-server", "--analytics-default-enabled"],
            vec!["app-server", "--model", "other"],
            vec!["--config", "model=other", "app-server"],
            vec!["exec"],
            vec!["mcp-server"],
            vec!["app-server", "--add-dir", "/other"],
        ] {
            let args = args.into_iter().map(str::to_owned).collect::<Vec<_>>();
            assert!(classify_ordinary_fork_codex_args(&args).is_err());
        }
        let temp = tempfile::tempdir().unwrap();
        assert!(
            classify_ordinary_codex_args(
                &["app-server".into()],
                None,
                temp.path(),
                "model",
                temp.path(),
            )
            .is_err(),
            "ordinary Start/Resume must retain their closed exec classifier"
        );
    }

    #[tokio::test]
    async fn test_history_use_survives_abandoned_completion_until_native_ack() {
        use std::sync::atomic::Ordering;
        let in_use = Arc::new(OrdinaryNativeHistoryStateV1::default());
        let custody = OrdinaryNativeHistoryUseV1::acquire(Arc::clone(&in_use)).unwrap();
        let (ack_tx, ack_rx) = tokio::sync::oneshot::channel();
        let completion = retain_ordinary_world_completion(
            Box::pin(async move {
                let _ = ack_rx.await;
                Err(AgentWrapperError::Backend {
                    message: "native acknowledgment error".into(),
                })
            }),
            custody,
            || {},
        );
        drop(completion);
        tokio::task::yield_now().await;
        assert_eq!(in_use.0.load(Ordering::Acquire), 1);
        assert!(OrdinaryNativeHistoryUseV1::acquire(Arc::clone(&in_use)).is_err());
        ack_tx.send(()).unwrap();
        tokio::time::timeout(std::time::Duration::from_secs(3), async {
            while in_use.0.load(Ordering::Acquire) == 1 {
                tokio::task::yield_now().await;
            }
        })
        .await
        .unwrap();
        drop(OrdinaryNativeHistoryUseV1::acquire(in_use).unwrap());
    }
}
