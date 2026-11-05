//! Core service implementation for world agent.

#[cfg(target_os = "linux")]
use anyhow::Context;
use anyhow::{anyhow, Result};
use axum::response::Response;
#[cfg(target_os = "linux")]
use axum::{
    body::{boxed, Bytes, StreamBody},
    http::StatusCode,
};
use base64::engine::general_purpose::STANDARD as BASE64;
use base64::Engine;
#[cfg(target_os = "linux")]
use chrono::SecondsFormat;
#[cfg(target_os = "linux")]
use futures_util::StreamExt;
#[cfg(any(target_os = "linux", test))]
use sha2::{Digest, Sha256};
use std::collections::HashMap;
#[cfg(target_os = "linux")]
use std::collections::HashSet;
#[cfg(target_os = "linux")]
use std::convert::Infallible;
#[cfg(target_os = "linux")]
use std::os::unix::fs::{FileTypeExt, PermissionsExt};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU8, Ordering};
#[cfg(target_os = "linux")]
use std::sync::Mutex;
use std::sync::{Arc, RwLock};
#[cfg(target_os = "linux")]
use substrate_common::agent_events::{AgentEvent, AgentEventKind};
use substrate_common::{WorldFsMode, WorldRootMode};
#[cfg(target_os = "linux")]
use tokio::task;
#[cfg(target_os = "linux")]
use tokio_stream::wrappers::UnboundedReceiverStream;
#[cfg(not(target_os = "linux"))]
use transport_api_types::GatewayStatusV1;
#[cfg(any(target_os = "linux", test))]
use transport_api_types::PendingDiffBucketV1;
#[cfg(target_os = "linux")]
use transport_api_types::WorldFsEntryTypeV1;
use transport_api_types::{
    Budget, ExecuteCancelRequestV1, ExecuteCancelResponseV1, ExecuteRequest, ExecuteResponse,
    GatewayLifecycleRequestV1, GatewayLifecycleResponseV1, MemberTurnSubmitRequestV1,
    PendingDiffClearRequestV1, PendingDiffClearResponseV1, PendingDiffReconcileRequestV1,
    PendingDiffReconcileResponseV1, PendingDiffRecordV1, PendingDiffRequestV1, ProcessTelemetry,
    WorldFsReadRequestV1, WorldFsReadResponseV1, WorldNetworkRoutingV1,
};
#[cfg(target_os = "linux")]
use transport_api_types::{
    ExecuteStreamFrame, RuntimeEventIdentityV1, RuntimeFrameIdentityV1, RuntimeTerminalIdentityV1,
    RUNTIME_FRAME_IDENTITY_SCHEMA_VERSION_V1,
};
#[cfg(target_os = "linux")]
use world::stream::{install_stream_sink, StreamKind, StreamSink};
use world_api::{
    MemberDispatchRequestV1 as BackendMemberDispatchRequestV1,
    MemberRuntimeBackendKindV1 as BackendMemberRuntimeBackendKindV1,
    ResolvedMemberRuntimeDescriptorV1 as BackendResolvedMemberRuntimeDescriptorV1,
    SharedWorldBindingSnapshot, SharedWorldBindingState, SharedWorldOwnerSpec, WorldBackend,
    WorldHandle, WorldReuseMode, WorldSpec,
};

#[cfg(target_os = "linux")]
use crate::e3_child_security::{E3PrivilegedChildExclusionV1, HeldNonE3PrivilegedChildLeaseV1};
use crate::enforcement_plan;
#[cfg(target_os = "linux")]
use crate::gateway_runtime::{
    resolve_gateway_backend_binding, unavailable_response as gateway_unavailable_response_impl,
    GatewayControlSettings, GatewayRuntimeFailure, GatewayRuntimeManager,
    GatewayRuntimeStartContext, LinuxWorldPlacementContext,
};
#[cfg(target_os = "linux")]
use crate::member_runtime::{MemberRuntimeLaunchAdmissionV1, MemberRuntimeManager};
#[cfg(target_os = "linux")]
use crate::member_turn_join::MemberTurnJoinRegistry;
use crate::request_routing::resolve_snapshot_routing;
#[cfg(target_os = "linux")]
use crate::runtime_replay::{
    publish_replayable_frame, RuntimeReplayPublisher, RuntimeReplayRegistry,
};

pub(crate) const ANCHOR_MODE_ENV: &str = "SUBSTRATE_ANCHOR_MODE";
pub(crate) const ANCHOR_PATH_ENV: &str = "SUBSTRATE_ANCHOR_PATH";
pub(crate) const WORLD_PROJECT_DIR_OVERRIDE_ENV: &str = "SUBSTRATE_WORLD_PROJECT_DIR";
#[cfg(target_os = "linux")]
pub(crate) const WORLD_FS_MODE_ENV: &str = "SUBSTRATE_WORLD_FS_MODE";
pub(crate) const WORLD_FS_ISOLATION_ENV: &str = "SUBSTRATE_WORLD_FS_ISOLATION";
pub(crate) const WORLD_FS_WRITE_ALLOWLIST_ENV: &str = "SUBSTRATE_WORLD_FS_WRITE_ALLOWLIST";
pub(crate) const WORLD_FS_LANDLOCK_READ_ALLOWLIST_ENV: &str =
    "SUBSTRATE_WORLD_FS_LANDLOCK_READ_ALLOWLIST";
pub(crate) const WORLD_FS_LANDLOCK_WRITE_ALLOWLIST_ENV: &str =
    "SUBSTRATE_WORLD_FS_LANDLOCK_WRITE_ALLOWLIST";
pub(crate) const WORLD_FS_LANDLOCK_DISCOVER_ALLOWLIST_ENV: &str =
    "SUBSTRATE_WORLD_FS_LANDLOCK_DISCOVER_ALLOWLIST";
pub(crate) const LANDLOCK_HELPER_SRC_ENV: &str = "SUBSTRATE_LANDLOCK_HELPER_SRC";

// Only boundary-attached metadata is eligible for this event. In particular, an
// arbitrary backend error (even an io::Error) must remain unclassified.
fn command_execution_failed(error: &anyhow::Error, attempt_id: uuid::Uuid) -> anyhow::Error {
    #[cfg(target_os = "linux")]
    let failure = error.downcast_ref::<world::exec::ShellCommandFailure>();
    #[cfg(target_os = "linux")]
    let (operation, raw_os_error, errno_status) = match failure {
        Some(failure) => {
            let operation = failure.operation();
            let errno = failure.raw_os_error();
            let status = if matches!(operation, "stdout-pipe" | "stderr-pipe") {
                "not-applicable"
            } else if errno.is_some() {
                "present"
            } else {
                "absent"
            };
            (operation, errno, status)
        }
        None => ("unknown", None, "unknown"),
    };
    #[cfg(not(target_os = "linux"))]
    let (operation, raw_os_error, errno_status): (&str, Option<i32>, &str) = {
        let _ = error;
        ("unknown", None, "unknown")
    };

    // A root event also avoids inheriting content-bearing request span fields.
    tracing::error!(
        parent: None,
        service = "world-service",
        process_id = std::process::id(),
        attempt_id = %attempt_id,
        operation,
        raw_os_error,
        errno_status,
        "exec failed"
    );
    anyhow::anyhow!("Command execution failed")
}

const CARGO_BIN_EXE_WORLD_AGENT_ENV: &str = "CARGO_BIN_EXE_world-service";
const CARGO_BIN_EXE_WORLD_AGENT_ALT_ENV: &str = "CARGO_BIN_EXE_world_service";

const NETFILTER_ENABLE_REQUIRED_TEXT: &str =
    "WORLD_NETFILTER_ENABLE must be set to 1/true/yes before requested network isolation can install nftables rules";
const NETFILTER_NFT_FAILURE_TEXT: &str = "nft command failed";
const NETFILTER_RESOLUTION_FAILURE_TEXT: &str = "failed to resolve allowed domain `";
const NETFILTER_NO_ADDRESS_TEXT: &str = "resolved to no addresses";
const NETFILTER_CGROUP_ATTACH_TEXT: &str = ": cgroup attach failed:";
const NETFILTER_CGROUP_HELPER_REFUSAL_TEXT: &str =
    "refused isolated execution before command start";
const NETFILTER_CGROUP_FORCE_DIRECT_TEXT: &str =
    "SUBSTRATE_WORLD_EXEC_FORCE_DIRECT is unsupported when isolate_network=true because cgroup attach is not guaranteed";

fn resolve_landlock_helper_src_from_exe(exe: &std::path::Path) -> Option<std::path::PathBuf> {
    let is_world_service_exe = exe.file_name().is_some_and(|name| {
        name == std::ffi::OsStr::new("world-service")
            || name == std::ffi::OsStr::new("substrate-world-service")
    });
    if is_world_service_exe && exe.is_file() {
        return Some(exe.to_owned());
    }

    // Common case in `cargo test`: current_exe is the test harness under `target/*/deps/...`.
    // Walk up a few parents and look for the workspace-built `world-service` binary.
    let mut dir = exe.parent();
    for _ in 0..8 {
        let Some(d) = dir else { break };
        let candidate = d.join("world-service");
        if candidate.is_file() {
            return Some(candidate);
        }
        dir = d.parent();
    }

    None
}

fn resolve_landlock_helper_src() -> Option<String> {
    fn accept_candidate(candidate: &str) -> Option<std::path::PathBuf> {
        let candidate = candidate.trim();
        if candidate.is_empty() {
            return None;
        }
        let path = std::path::Path::new(candidate);
        if path.is_file() {
            return Some(path.to_owned());
        }
        None
    }

    fn same_file(a: &std::path::Path, b: &std::path::Path) -> bool {
        if a == b {
            return true;
        }
        match (std::fs::canonicalize(a), std::fs::canonicalize(b)) {
            (Ok(a), Ok(b)) => a == b,
            _ => false,
        }
    }

    let exe = std::env::current_exe().ok();

    // Prefer explicitly provided helper src (mainly for tests/tools), but avoid accepting the
    // current process when it is a `cargo test` harness (it will swallow the helper argv and exit 0).
    if let Ok(candidate) = std::env::var(LANDLOCK_HELPER_SRC_ENV) {
        if let Some(path) = accept_candidate(&candidate) {
            let is_test_harness_self = exe.as_ref().is_some_and(|exe| {
                exe.file_name().is_none_or(|n| n != "world-service") && same_file(&path, exe)
            });
            if !is_test_harness_self {
                return Some(path.display().to_string());
            }
        }
    }

    // Best-effort: if a runtime env var exists, honor it.
    for key in [
        CARGO_BIN_EXE_WORLD_AGENT_ENV,
        CARGO_BIN_EXE_WORLD_AGENT_ALT_ENV,
    ] {
        if let Ok(candidate) = std::env::var(key) {
            if let Some(path) = accept_candidate(&candidate) {
                return Some(path.display().to_string());
            }
        }
    }

    #[cfg(test)]
    if let Some(exe) = exe.as_ref() {
        if exe.is_file() {
            return Some(exe.display().to_string());
        }
    }

    let exe = exe?;
    resolve_landlock_helper_src_from_exe(&exe).map(|p| p.display().to_string())
}

/// Main service running inside the world.
#[derive(Clone)]
pub struct WorldService {
    #[cfg(target_os = "linux")]
    host_gateway_turns: Arc<crate::gateway_runtime::HostGatewayTurnOwnerV1>,
    // Keep ordinary configured custody independent of protected E3 setup.
    #[cfg(target_os = "linux")]
    ordinary_selection_authority: Option<Arc<substrate_shell::OrdinarySelectionAuthorityV1>>,
    #[cfg(target_os = "linux")]
    pub(crate) accepted_home_authority:
        Option<Arc<config_projection::ConfiguredAcceptedHomeAuthorityV1>>,
    #[cfg(target_os = "linux")]
    pub(crate) config_projection_service:
        Option<Arc<config_projection::AgentConfigProjectionServiceV1>>,
    #[cfg(target_os = "linux")]
    pub(crate) e3_projection_preparations:
        Option<Arc<crate::e3_config_projection_prepare::E3ConfigProjectionPreparationManagerV1>>,
    backend: Arc<dyn WorldBackend>,
    #[cfg(target_os = "linux")]
    linux_backend: Arc<world::LinuxLocalBackend>,
    #[cfg(target_os = "linux")]
    gateway_runtime: Arc<GatewayRuntimeManager>,
    #[cfg(target_os = "linux")]
    member_runtime: Arc<MemberRuntimeManager>,
    #[cfg(target_os = "linux")]
    runtime_replay: RuntimeReplayRegistry,
    #[cfg(target_os = "linux")]
    member_turn_join: Option<MemberTurnJoinRegistry>,
    #[cfg(target_os = "linux")]
    pending_diff_origin: Arc<RwLock<HashMap<String, PendingDiffOriginTracker>>>,
    #[cfg(target_os = "linux")]
    privileged_child_exclusion: Arc<E3PrivilegedChildExclusionV1>,
    #[allow(dead_code)]
    worlds: Arc<RwLock<HashMap<String, WorldHandle>>>,
    budgets: Arc<RwLock<HashMap<String, AgentBudgetTracker>>>,
    last_policy_resolution_mode: Arc<AtomicU8>,
    last_netfilter_requested: Arc<AtomicU8>,
    last_netfilter_failure_reason: Arc<RwLock<Option<String>>>,
}

/// Owned ordinary effect context. It retains no credential, request-selected
/// home or serialized capability. Permission and exact producer custody are
/// independent and both are revalidated before effects.
#[cfg(target_os = "linux")]
pub(crate) struct OrdinaryWorldEffectContextV1 {
    bootstrap: Arc<substrate_shell::OrdinaryWorldExecuteAdmissionV1>,
    turn: Option<Arc<substrate_shell::OrdinaryWorldTurnAdmissionV1>>,
    live_bootstrap: Option<Arc<substrate_shell::OrdinaryWorldBootstrapEffectAdmissionV1>>,
    start: transport_api_types::OrdinaryStartSelectionCarrierV1,
    enforced_policy_snapshot: transport_api_types::PolicySnapshotV3,
    backend: Arc<world::LinuxLocalBackend>,
    parent_spec: WorldSpec,
    custody_binding_ref: String,
    spec: WorldSpec,
    binding: SharedWorldBindingSnapshot,
    cgroup_path: PathBuf,
    cgroup: Arc<std::fs::File>,
    gateway_runtime: Arc<GatewayRuntimeManager>,
    pub(crate) native_open: Arc<std::sync::atomic::AtomicBool>,
}

/// Lifetime custody only. It cannot validate/issue an effect and has no manager backreference.
#[cfg(target_os = "linux")]
pub(crate) struct OrdinaryGatewayCleanupCustodyV1 {
    _bootstrap: Arc<substrate_shell::OrdinaryWorldExecuteAdmissionV1>,
    _turn: Option<Arc<substrate_shell::OrdinaryWorldTurnAdmissionV1>>,
    _live: Option<Arc<substrate_shell::OrdinaryWorldBootstrapEffectAdmissionV1>>,
    _cgroup: Arc<std::fs::File>,
    _backend: Arc<world::LinuxLocalBackend>,
}

#[cfg(target_os = "linux")]
impl std::fmt::Debug for OrdinaryWorldEffectContextV1 {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("OrdinaryWorldEffectContextV1([redacted])")
    }
}

#[cfg(target_os = "linux")]
impl OrdinaryWorldEffectContextV1 {
    pub(crate) fn adopt_stored_initial_bootstrap_controls(
        &self,
        span_id: &str,
        latch: impl FnMut(&transport_api_types::OrdinaryInitialBootstrapControlRequestV1) -> Result<()>,
    ) -> Result<()> {
        anyhow::ensure!(
            self.turn.is_none(),
            "initial control adoption cannot target accepted work"
        );
        self.bootstrap
            .with_stored_initial_bootstrap_controls(span_id, latch)
    }

    pub(crate) fn initial_bootstrap_closeout_reader(
        &self,
        span_id: &str,
    ) -> Result<substrate_shell::OrdinaryInitialBootstrapCloseoutReaderV1> {
        anyhow::ensure!(
            self.turn.is_none(),
            "initial closeout cannot target accepted work"
        );
        self.bootstrap.initial_bootstrap_closeout_reader(span_id)
    }

    pub(crate) fn with_initial_bootstrap_control(
        &self,
        request: &transport_api_types::OrdinaryInitialBootstrapControlRequestV1,
        peer_uid: u32,
        cancel: impl FnOnce() -> Result<bool>,
    ) -> Result<bool> {
        anyhow::ensure!(
            self.turn.is_none(),
            "initial control cannot target an accepted turn"
        );
        self.bootstrap
            .with_initial_bootstrap_control(request, peer_uid, cancel)
    }

    pub(crate) fn claim_initial_bootstrap_receiver(&self, span_id: &str) -> Result<()> {
        self.revalidate_for_effect()?;
        anyhow::ensure!(
            self.turn.is_none() && self.live_bootstrap.is_none(),
            "receiver claim requires initial owner"
        );
        self.bootstrap.claim_initial_bootstrap_receiver(span_id)?;
        Ok(())
    }

    /// Source history use is authorized by the child's current admission. A
    /// retired source native owner is never reopened or granted another route.
    pub(crate) fn validate_initial_fork_source(
        &self,
        source: &Self,
    ) -> Result<transport_api_types::E2MemberLaunchActivationCarrierV1> {
        self.revalidate_for_effect()?;
        if self.turn.is_some()
            || self.live_bootstrap.is_some()
            || self.start != source.start
            || self.binding != source.binding
            || serde_json::to_vec(&self.spec)? != serde_json::to_vec(&source.spec)?
        {
            anyhow::bail!("ordinary initial Fork changed source workspace/World/selection");
        }
        let activation = self
            .bootstrap
            .validate_initial_fork_source(&source.bootstrap)?;
        self.revalidate_for_effect()?;
        Ok(activation)
    }

    /// Retain existing root/cgroup owners for cleanup without cloning the manager.
    pub(crate) fn hold_gateway_cleanup_custody(&self) -> Arc<OrdinaryGatewayCleanupCustodyV1> {
        Arc::new(OrdinaryGatewayCleanupCustodyV1 {
            _bootstrap: Arc::clone(&self.bootstrap),
            _turn: self.turn.clone(),
            _live: self.live_bootstrap.clone(),
            _cgroup: Arc::clone(&self.cgroup),
            _backend: Arc::clone(&self.backend),
        })
    }

    fn bind_bootstrap(
        bootstrap: Arc<substrate_shell::OrdinaryWorldExecuteAdmissionV1>,
        start: transport_api_types::OrdinaryStartSelectionCarrierV1,
        enforced_policy_snapshot: transport_api_types::PolicySnapshotV3,
        gateway_runtime: Arc<GatewayRuntimeManager>,
        backend: Arc<world::LinuxLocalBackend>,
    ) -> Result<(Arc<Self>, Arc<world::LinuxLocalBackend>, WorldHandle)> {
        bootstrap.revalidate_before_world_effect()?;
        let parent = bootstrap.current_world_parent_policy()?;
        let transport_api_types::OrdinaryGatewayRuntimeIdentityV1::WorldWorker {
            orchestration_session_id,
            world_id,
            world_generation,
            ..
        } = bootstrap.authenticated_runtime()
        else {
            anyhow::bail!("ordinary effect requires authenticated World-worker runtime");
        };
        let owner = SharedWorldOwnerSpec {
            orchestration_session_id: orchestration_session_id.clone(),
            action: world_api::SharedWorldOwnerAction::AttachOrCreate,
        };
        let mut parent_spec = parent.world_spec().clone();
        parent_spec.reuse_mode = WorldReuseMode::SharedOrchestration(owner.clone());
        // The persistent-session producer deliberately has a different physical
        // recipe from the shell parent. Keep the complete parent policy fence,
        // then require positive exact custody for one complete producer recipe.
        let persistent_spec = build_world_spec(
            parent_spec.project_dir.clone(),
            true,
            parent_spec.fs_mode,
            parent_spec.isolate_network,
            parent_spec.allowed_domains.clone(),
            Some(&owner),
        );
        bootstrap.revalidate_before_world_effect()?;
        let (spec, world, cgroup_path, cgroup, custody_binding_ref) = backend
            .with_existing_custody_nonreconciling_v1(
                &orchestration_session_id,
                &world_id,
                world_generation,
                &parent_spec.project_dir,
                |held| {
                    let (spec, world, path, cgroup) =
                        held.world_placement_for_specs_v1(&[parent_spec.clone(), persistent_spec])?;
                    Ok((spec, world, path, cgroup, held.binding_ref().to_string()))
                },
            )?;
        let binding = world
            .shared_binding
            .clone()
            .ok_or_else(|| anyhow!("ordinary backend omitted actual shared binding"))?;
        bootstrap.revalidate_for_effect(&binding)?;
        let context = Arc::new(Self {
            bootstrap,
            turn: None,
            live_bootstrap: None,
            start,
            enforced_policy_snapshot,
            backend: Arc::clone(&backend),
            parent_spec,
            custody_binding_ref,
            spec,
            binding,
            cgroup_path,
            cgroup: Arc::new(cgroup),
            gateway_runtime,
            native_open: Arc::new(std::sync::atomic::AtomicBool::new(true)),
        });
        context.revalidate_for_effect()?;
        Ok((context, backend, world))
    }

    pub(crate) fn bind_live_bootstrap_effect(
        self: &Arc<Self>,
        actual_runtime: &transport_api_types::OrdinaryGatewayRuntimeIdentityV1,
        actual_binding: &SharedWorldBindingSnapshot,
    ) -> Result<Arc<Self>> {
        if self.turn.is_some()
            || self.live_bootstrap.is_some()
            || !self.native_open.load(Ordering::Acquire)
        {
            anyhow::bail!("ordinary live bootstrap can bind only its newly owned native control");
        }
        // This authenticates current permission without backend recovery. The
        // launch observation is an identity join; freshness follows below.
        let live = Arc::new(
            self.bootstrap
                .bind_live_bootstrap_effect(actual_runtime, actual_binding)?,
        );
        let context = Arc::new(Self {
            bootstrap: Arc::clone(&self.bootstrap),
            turn: None,
            live_bootstrap: Some(live),
            start: self.start.clone(),
            enforced_policy_snapshot: self.enforced_policy_snapshot.clone(),
            backend: Arc::clone(&self.backend),
            parent_spec: self.parent_spec.clone(),
            custody_binding_ref: self.custody_binding_ref.clone(),
            spec: self.spec.clone(),
            binding: self.binding.clone(),
            cgroup_path: self.cgroup_path.clone(),
            cgroup: Arc::clone(&self.cgroup),
            gateway_runtime: Arc::clone(&self.gateway_runtime),
            native_open: Arc::clone(&self.native_open),
        });
        context.revalidate_for_effect()?;
        Ok(context)
    }

    pub(crate) fn turn_admission(&self) -> Result<&substrate_shell::OrdinaryWorldTurnAdmissionV1> {
        self.turn
            .as_deref()
            .ok_or_else(|| anyhow!("ordinary registry operation requires genuine turn admission"))
    }

    pub(crate) fn prepare_turn(
        self: &Arc<Self>,
        request: &MemberTurnSubmitRequestV1,
    ) -> Result<Arc<Self>> {
        let admitted = Arc::new(self.bootstrap.prepare_turn(request)?);
        let snapshot = request
            .policy_snapshot_carrier
            .as_ref()
            .ok_or_else(|| anyhow!("ordinary turn omitted authenticated E2 snapshot"))?
            .policy_snapshot()
            .map_err(anyhow::Error::msg)?;
        let context = Arc::new(Self {
            bootstrap: Arc::clone(&self.bootstrap),
            turn: Some(admitted),
            live_bootstrap: None,
            start: self.start.clone(),
            enforced_policy_snapshot: snapshot,
            backend: Arc::clone(&self.backend),
            parent_spec: self.parent_spec.clone(),
            custody_binding_ref: self.custody_binding_ref.clone(),
            spec: self.spec.clone(),
            binding: self.binding.clone(),
            cgroup_path: self.cgroup_path.clone(),
            cgroup: Arc::clone(&self.cgroup),
            gateway_runtime: Arc::clone(&self.gateway_runtime),
            native_open: Arc::new(std::sync::atomic::AtomicBool::new(true)),
        });
        context.revalidate_for_effect()?;
        Ok(context)
    }

    /// Original immutable identity data, never current permission or a capability.
    pub(crate) fn ordinary_start_selection(
        &self,
    ) -> &transport_api_types::OrdinaryStartSelectionCarrierV1 {
        &self.start
    }

    pub(crate) fn requested_selection(&self) -> &transport_api_types::ResolvedAgentSelectionV1 {
        match &self.turn {
            Some(turn) => turn.requested_selection(),
            None => self.live_bootstrap.as_ref().map_or_else(
                || self.bootstrap.requested_selection(),
                |live| live.requested_selection(),
            ),
        }
    }

    pub(crate) fn authenticated_runtime(
        &self,
    ) -> transport_api_types::OrdinaryGatewayRuntimeIdentityV1 {
        match &self.turn {
            Some(turn) => turn.authenticated_runtime(),
            None => self.live_bootstrap.as_ref().map_or_else(
                || self.bootstrap.authenticated_runtime(),
                |live| live.authenticated_runtime(),
            ),
        }
    }

    pub(crate) fn with_ordinary_accepted_cancellation(
        &self,
        request: &transport_api_types::OrdinaryAcceptedWorkCancelRequestV1,
        peer_uid: u32,
        cancel: impl FnOnce() -> Result<bool>,
    ) -> Result<bool> {
        match &self.turn {
            Some(turn) => turn.with_ordinary_accepted_cancellation(request, peer_uid, cancel),
            None => self
                .bootstrap
                .with_ordinary_accepted_cancellation(request, peer_uid, cancel),
        }
    }

    pub(crate) fn retained_stop_control_reader(
        &self,
    ) -> Result<substrate_shell::OrdinaryRetainedStopControlReaderV1> {
        self.bootstrap.ordinary_retained_stop_control_reader()
    }

    /// Existing placement custody for cleanup only. The original bootstrap's
    /// native effect is closed; Stop source permission is checked separately.
    pub(crate) fn revalidate_original_stop_world_placement(
        &self,
        request: &transport_api_types::OrdinaryRetainedWorkerStopRequestV1,
    ) -> Result<()> {
        anyhow::ensure!(
            !self.native_open.load(Ordering::Acquire)
                && self.start.authority_store_id == request.authority_store_id
                && self.binding.orchestration_session_id == request.orchestration_session_id
                && self.binding.world_id == request.world_id
                && self.binding.world_generation == request.world_generation,
            "ordinary Stop original native/World scope remains open or changed"
        );
        self.revalidate_sealed_world_placement()
    }

    /// Keep the actual already-owned control when disposal wins the race with
    /// live binding. This creates no live admission and never reopens effects.
    pub(crate) fn retain_sealed_bootstrap_owner(
        self: &Arc<Self>,
        actual_runtime: &transport_api_types::OrdinaryGatewayRuntimeIdentityV1,
        actual_binding: &SharedWorldBindingSnapshot,
    ) -> Result<Arc<Self>> {
        self.revalidate_sealed_bootstrap_world_placement(actual_runtime, actual_binding)?;
        Ok(Arc::clone(self))
    }

    pub(crate) fn revalidate_sealed_bootstrap_world_placement(
        &self,
        actual_runtime: &transport_api_types::OrdinaryGatewayRuntimeIdentityV1,
        actual_binding: &SharedWorldBindingSnapshot,
    ) -> Result<()> {
        anyhow::ensure!(
            self.turn.is_none()
                && &self.authenticated_runtime() == actual_runtime
                && actual_binding == &self.binding,
            "sealed bootstrap changed actual original runtime or World"
        );
        self.revalidate_sealed_world_placement()
    }

    fn revalidate_sealed_world_placement(&self) -> Result<()> {
        use std::os::unix::fs::MetadataExt;
        anyhow::ensure!(
            !self.native_open.load(Ordering::Acquire),
            "ordinary native effect remains open"
        );
        let (actual, path, current) = self.backend.with_existing_custody_nonreconciling_v1(
            &self.binding.orchestration_session_id,
            &self.binding.world_id,
            self.binding.world_generation,
            &self.spec.project_dir,
            |held| {
                anyhow::ensure!(
                    held.binding_ref() == self.custody_binding_ref,
                    "ordinary Stop original World producer custody changed"
                );
                let (_, world, path, cgroup) =
                    held.world_placement_for_specs_v1(std::slice::from_ref(&self.spec))?;
                Ok((world, path, cgroup))
            },
        )?;
        let binding = actual
            .shared_binding
            .as_ref()
            .ok_or_else(|| anyhow!("ordinary Stop existing World omitted binding"))?;
        anyhow::ensure!(
            binding == &self.binding
                && binding.binding_state == SharedWorldBindingState::Active
                && path == self.cgroup_path,
            "ordinary Stop existing World generation/placement changed"
        );
        let original = self.cgroup.metadata()?;
        let now = current.metadata()?;
        anyhow::ensure!(
            now.is_dir() && (original.dev(), original.ino()) == (now.dev(), now.ino()),
            "ordinary Stop original cgroup physical identity changed"
        );
        Ok(())
    }

    pub(crate) fn revalidate_before_world_effect(&self) -> Result<()> {
        if !self.native_open.load(Ordering::Acquire) {
            anyhow::bail!("ordinary native effect owner has closed");
        }
        let parent = match &self.turn {
            Some(turn) => {
                // Parent construction validates this exact turn before and
                // after reading current policy. Reuse that check here; the
                // effect path still validates again after backend observation.
                turn.current_world_parent_policy()?
            }
            None => match &self.live_bootstrap {
                Some(live) => {
                    live.revalidate_before_world_effect()?;
                    live.current_world_parent_policy()?
                }
                None => {
                    self.bootstrap.revalidate_before_world_effect()?;
                    self.bootstrap.current_world_parent_policy()?
                }
            },
        };
        let mut current = parent.world_spec().clone();
        current.reuse_mode = self.parent_spec.reuse_mode.clone();
        if serde_json::to_vec(&current)? != serde_json::to_vec(&self.parent_spec)? {
            anyhow::bail!("ordinary current parent is incompatible with held World realization");
        }
        Ok(())
    }

    pub(crate) fn revalidate_for_effect(&self) -> Result<()> {
        use std::os::unix::fs::MetadataExt;
        self.revalidate_before_world_effect()?;
        let (actual, path, current) = self.backend.with_existing_custody_nonreconciling_v1(
            &self.binding.orchestration_session_id,
            &self.binding.world_id,
            self.binding.world_generation,
            &self.spec.project_dir,
            |held| {
                if held.binding_ref() != self.custody_binding_ref {
                    anyhow::bail!("ordinary original World producer custody changed");
                }
                let (_, world, path, cgroup) =
                    held.world_placement_for_specs_v1(std::slice::from_ref(&self.spec))?;
                Ok((world, path, cgroup))
            },
        )?;
        let binding = actual
            .shared_binding
            .as_ref()
            .ok_or_else(|| anyhow!("ordinary actual World omitted binding"))?;
        if binding != &self.binding || binding.binding_state != SharedWorldBindingState::Active {
            anyhow::bail!("ordinary actual World generation/binding changed");
        }
        if path != self.cgroup_path {
            anyhow::bail!("ordinary actual cgroup placement changed");
        }
        let held = self.cgroup.metadata()?;
        let now = current.metadata()?;
        if !now.is_dir() || (held.dev(), held.ino()) != (now.dev(), now.ino()) {
            anyhow::bail!("ordinary held cgroup physical identity changed");
        }
        match &self.turn {
            Some(turn) => turn.revalidate_for_effect(binding),
            None => match &self.live_bootstrap {
                Some(live) => live.revalidate_for_effect(binding),
                None => self.bootstrap.revalidate_for_effect(binding),
            },
        }
    }

    pub(crate) fn resolve_current_gateway_handoff(
        &self,
    ) -> Result<transport_api_types::GatewayIntegratedAuthPayloadV1> {
        self.revalidate_for_effect()?;
        match &self.turn {
            Some(turn) => turn.resolve_current_gateway_handoff(),
            None => match &self.live_bootstrap {
                Some(live) => live.resolve_current_gateway_handoff(),
                None => self.bootstrap.resolve_current_gateway_handoff(),
            },
        }
    }

    pub(crate) async fn prepare_ordinary_native_gateway(
        self: &Arc<Self>,
        owned: Option<Arc<crate::gateway_runtime::OrdinaryGatewayRuntimeLeaseV1>>,
    ) -> Result<(
        transport_api_types::GatewayClientWiringV1,
        Arc<crate::gateway_runtime::OrdinaryGatewayRuntimeLeaseV1>,
    )> {
        self.revalidate_for_effect()?;
        let runtime = self.authenticated_runtime();
        let runtime_id = format!(
            "ordinary_{:x}",
            Sha256::digest(serde_json::to_vec(&runtime)?)
        );
        let fresh_preparation = owned.is_none();
        let lease = match owned {
            Some(lease) => {
                if lease.runtime_id != runtime_id {
                    anyhow::bail!("ordinary owned gateway runtime changed");
                }
                lease
            }
            None => self
                .gateway_runtime
                .lease_ordinary_runtime(Arc::clone(self), runtime_id.clone())?,
        };
        let binding = crate::gateway_runtime::ordinary_gateway_adapter(self.requested_selection())?;
        let response = self
            .gateway_runtime
            .restart_ordinary_owned(
                GatewayRuntimeStartContext {
                    world_id: runtime_id,
                    project_dir: self.spec.project_dir.clone(),
                    cgroup_path: self.cgroup_path.clone(),
                    require_cgroup_attach: true,
                    binding,
                    ordinary_effect: Some(Arc::clone(self)),
                    integrated_auth: None,
                },
                Arc::clone(&lease),
                fresh_preparation,
            )
            .await
            .map_err(gateway_runtime_error)?;
        if let Err(error) = self.revalidate_for_effect() {
            self.native_open.store(false, Ordering::Release);
            if let Err(cleanup) = lease.finish_owned().await {
                anyhow::bail!("ordinary gateway post-ready admission failed: {error:#}; terminal cleanup unresolved: {cleanup:#}");
            }
            return Err(error);
        }
        let wiring = response
            .client_wiring
            .filter(|_| response.status == transport_api_types::GatewayStatusV1::Available)
            .ok_or_else(|| anyhow!("ordinary managed gateway is unavailable"))?;
        Ok((wiring, lease))
    }
}

#[cfg(target_os = "linux")]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum PendingDiffOrigin {
    NonPty,
    Pty,
}

#[cfg(target_os = "linux")]
#[derive(Debug, Default)]
struct PendingDiffOriginTracker {
    last_seen_paths: HashSet<String>,
    origin_by_path: HashMap<String, PendingDiffOrigin>,
}

pub struct AgentBudgetTracker {
    #[allow(dead_code)]
    agent_id: String,
    execs_remaining: std::sync::atomic::AtomicU32,
    #[allow(dead_code)]
    runtime_remaining_ms: std::sync::atomic::AtomicU64,
    #[allow(dead_code)]
    egress_remaining: std::sync::atomic::AtomicU64,
}

impl AgentBudgetTracker {
    pub fn new(agent_id: &str, budget: Budget) -> Self {
        Self {
            agent_id: agent_id.to_string(),
            execs_remaining: std::sync::atomic::AtomicU32::new(budget.max_execs.unwrap_or(1000)),
            runtime_remaining_ms: std::sync::atomic::AtomicU64::new(
                budget.max_runtime_ms.unwrap_or(300_000),
            ),
            egress_remaining: std::sync::atomic::AtomicU64::new(
                budget.max_egress_bytes.unwrap_or(100_000_000),
            ),
        }
    }

    pub fn can_execute(&self) -> bool {
        use std::sync::atomic::Ordering;
        self.execs_remaining.load(Ordering::SeqCst) > 0
    }

    pub fn decrement_exec(&self) {
        use std::sync::atomic::Ordering;
        self.execs_remaining.fetch_sub(1, Ordering::SeqCst);
    }
}

impl WorldService {
    pub fn new() -> Result<Self> {
        #[cfg(target_os = "linux")]
        {
            let privileged_child_exclusion = E3PrivilegedChildExclusionV1::new_recovering()?;
            let service =
                Self::new_with_privileged_child_exclusion(Arc::clone(&privileged_child_exclusion))?;
            privileged_child_exclusion.finish_recovery()?;
            Ok(service)
        }

        #[cfg(not(target_os = "linux"))]
        {
            let backend = Self::create_backend()?;

            Ok(Self {
                backend,
                worlds: Arc::new(RwLock::new(HashMap::new())),
                budgets: Arc::new(RwLock::new(HashMap::new())),
                last_policy_resolution_mode: Arc::new(AtomicU8::new(0)),
                last_netfilter_requested: Arc::new(AtomicU8::new(0)),
                last_netfilter_failure_reason: Arc::new(RwLock::new(None)),
            })
        }
    }

    #[cfg(target_os = "linux")]
    pub(crate) fn new_with_privileged_child_exclusion(
        privileged_child_exclusion: Arc<E3PrivilegedChildExclusionV1>,
    ) -> Result<Self> {
        let member_turn_join = match MemberTurnJoinRegistry::open(Path::new("/var/lib/substrate")) {
            Ok(registry) => Some(registry),
            Err(error) => {
                tracing::warn!(
                    error = %error,
                    "durable E2 member-turn join authority is unavailable; activated E2 retained turns will fail closed"
                );
                None
            }
        };
        let service = Self::new_linux(member_turn_join, privileged_child_exclusion)?;
        #[cfg(not(test))]
        service
            .gateway_runtime
            .recover_existing_non_e3_gateways_for_exclusion()?;
        Ok(service)
    }

    #[cfg(target_os = "linux")]
    #[doc(hidden)]
    pub fn new_with_member_turn_state_root_for_test(state_root: &Path) -> Result<Self> {
        let privileged_child_exclusion = E3PrivilegedChildExclusionV1::new_recovering()?;
        let service = Self::new_linux(
            Some(MemberTurnJoinRegistry::open(state_root)?),
            Arc::clone(&privileged_child_exclusion),
        )?;
        if let Err(error) = privileged_child_exclusion.finish_recovery() {
            tracing::warn!(
                error = %error,
                "test-only member-turn service remains recovery-closed around an inherited child"
            );
        }
        Ok(service)
    }

    #[cfg(target_os = "linux")]
    fn new_linux(
        member_turn_join: Option<MemberTurnJoinRegistry>,
        privileged_child_exclusion: Arc<E3PrivilegedChildExclusionV1>,
    ) -> Result<Self> {
        let linux_backend = Arc::new(world::LinuxLocalBackend::new());
        let backend: Arc<dyn WorldBackend> = linux_backend.clone();
        let runtime_replay = RuntimeReplayRegistry::with_durable_e2(member_turn_join.clone());
        // Installed home custody is independent of whether protected projection
        // recovery can open an already-activated HSA. Ordinary first Start must
        // be able to activate HSA after this service has begun accepting peers.
        let accepted_home_authority =
            config_projection::ConfiguredAcceptedHomeAuthorityV1::from_installed_bootstrap_authority()
                .ok()
                .map(Arc::new);
        let ordinary_selection_authority =
            accepted_home_authority.as_ref().and_then(|configured| {
                substrate_shell::OrdinarySelectionAuthorityV1::from_configured_accepted_home(
                    Arc::clone(configured),
                )
                .ok()
                .map(Arc::new)
            });
        let e3_authority = (|| -> Result<_> {
            let configured = accepted_home_authority
                .as_ref()
                .context("configured accepted home is unavailable")?;
            let parent = Arc::new(
                substrate_shell::OpenedConfigProjectionHsaAuthorityV1::from_configured_accepted_home(configured)?,
            );
            let registry = Arc::new(config_projection::ConfigProjectionRegistryV1::open(
                parent.clone(),
            )?);
            let projection_service =
                Arc::new(config_projection::AgentConfigProjectionServiceV1::new(
                    Arc::clone(&registry),
                    Arc::clone(configured),
                )?);
            let preparations = Arc::new(
                crate::e3_config_projection_prepare::E3ConfigProjectionPreparationManagerV1::new(
                    parent,
                    Arc::clone(&projection_service),
                    registry,
                    Arc::clone(&privileged_child_exclusion),
                ),
            );
            Ok((projection_service, preparations))
        })();
        let (config_projection_service, e3_projection_preparations) = match e3_authority {
            Ok((projection_service, preparations)) => {
                (Some(projection_service), Some(preparations))
            }
            Err(_) => {
                tracing::warn!("E3 configured projection authority unavailable; E3 preparation remains unsupported");
                (None, None)
            }
        };

        Ok(Self {
            host_gateway_turns: Arc::new(crate::gateway_runtime::HostGatewayTurnOwnerV1::new()),
            ordinary_selection_authority,
            accepted_home_authority,
            config_projection_service,
            e3_projection_preparations,
            backend,
            linux_backend,
            gateway_runtime: Arc::new(GatewayRuntimeManager::new(Arc::clone(
                &privileged_child_exclusion,
            ))),
            member_runtime: Arc::new(MemberRuntimeManager::with_replay_and_e3_projection_service(
                runtime_replay.clone(),
                member_turn_join.clone(),
                Arc::clone(&privileged_child_exclusion),
            )),
            runtime_replay,
            member_turn_join,
            pending_diff_origin: Arc::new(RwLock::new(HashMap::new())),
            privileged_child_exclusion,
            worlds: Arc::new(RwLock::new(HashMap::new())),
            budgets: Arc::new(RwLock::new(HashMap::new())),
            last_policy_resolution_mode: Arc::new(AtomicU8::new(0)),
            last_netfilter_requested: Arc::new(AtomicU8::new(0)),
            last_netfilter_failure_reason: Arc::new(RwLock::new(None)),
        })
    }

    #[cfg(target_os = "linux")]
    pub(crate) async fn e3_config_projection_prepare(
        &self,
        body: hyper::Body,
    ) -> Result<
        transport_api_types::E3ConfigProjectionPrepareResponseV1,
        config_projection::ConfigProjectionFailureV1,
    > {
        use config_projection::ConfigProjectionFailureV1::UnsupportedConfiguration;
        self.config_projection_service
            .as_ref()
            .ok_or(UnsupportedConfiguration)?;
        self.e3_projection_preparations
            .as_ref()
            .ok_or(UnsupportedConfiguration)?
            .prepare(body)
            .await
    }

    #[cfg(target_os = "linux")]
    pub(crate) fn e3_config_projection_cancel(
        &self,
        request: transport_api_types::E3ConfigProjectionCancelRequestV1,
    ) -> Result<
        transport_api_types::E3ConfigProjectionCancelResponseV1,
        config_projection::ConfigProjectionFailureV1,
    > {
        request
            .validate()
            .map_err(|_| config_projection::ConfigProjectionFailureV1::Malformed)?;
        self.e3_projection_preparations
            .as_ref()
            .ok_or(config_projection::ConfigProjectionFailureV1::UnsupportedConfiguration)?
            .cancel(request)
    }

    #[cfg(target_os = "linux")]
    pub(crate) fn acquire_non_e3_helper_operation(
        &self,
    ) -> Result<HeldNonE3PrivilegedChildLeaseV1> {
        self.privileged_child_exclusion.acquire_non_e3_child()
    }

    #[cfg(target_os = "linux")]
    pub(crate) fn ensure_non_e3_world_session(
        &self,
        spec: &WorldSpec,
    ) -> Result<(WorldHandle, HeldNonE3PrivilegedChildLeaseV1)> {
        let lease = self.acquire_non_e3_helper_operation()?;
        let world = self.backend.ensure_session(spec)?;
        Ok((world, lease))
    }

    /// Ensure a session world (thin wrapper over backend)
    #[cfg(target_os = "linux")]
    pub fn ensure_session_world(&self, spec: &WorldSpec) -> Result<WorldHandle> {
        self.backend.ensure_session(spec)
    }

    /// Ensure the session world exists and return the merged overlay root for PTY sessions.
    #[cfg(target_os = "linux")]
    pub fn ensure_session_overlay_root(&self, spec: &WorldSpec) -> Result<(WorldHandle, PathBuf)> {
        let world = self.linux_backend.ensure_session(spec)?;
        let merged = self.linux_backend.ensure_overlay_root(&world)?;
        Ok((world, merged))
    }

    #[cfg(target_os = "linux")]
    pub fn refresh_session_network_filter(&self, world: &WorldHandle) -> Result<()> {
        self.linux_backend.refresh_network_filter(world)
    }

    #[cfg(target_os = "linux")]
    pub fn session_cgroup_path(&self, world: &WorldHandle) -> Result<PathBuf> {
        self.linux_backend.cgroup_path(world)
    }

    #[cfg(any(target_os = "linux", test))]
    fn normalize_pending_diff_bucket(diff: &substrate_common::FsDiff) -> PendingDiffBucketV1 {
        fn normalize(path: &std::path::Path) -> String {
            path.to_string_lossy().replace('\\', "/")
        }

        let mut writes: Vec<String> = diff.writes.iter().map(|p| normalize(p)).collect();
        let mut mods: Vec<String> = diff.mods.iter().map(|p| normalize(p)).collect();
        let mut deletes: Vec<String> = diff.deletes.iter().map(|p| normalize(p)).collect();
        writes.sort();
        mods.sort();
        deletes.sort();
        writes.dedup();
        mods.dedup();
        deletes.dedup();

        PendingDiffBucketV1 {
            writes,
            mods,
            deletes,
        }
    }

    #[cfg(any(target_os = "linux", test))]
    fn pending_diff_id_for_snapshot(snapshot: &PendingDiffBucketV1) -> String {
        // `diff_id` is used as a conditional clear token (Clear/ack semantics).
        //
        // It must remain stable across harmless reclassification of a path between `writes` and
        // `mods` (e.g., when a sync to the host causes the overlay lower layer to start reporting
        // the path as "existing", flipping upper->lower classification without any new world
        // mutation). To avoid false "diff_id mismatch" failures, compute the id from:
        // - updates = (writes ∪ mods) as a set
        // - deletes as a set
        //
        // The UX buckets are still preserved in the record; only the clear token is stabilized.
        let mut updates: Vec<&str> =
            Vec::with_capacity(snapshot.writes.len() + snapshot.mods.len());
        for item in &snapshot.writes {
            updates.push(item.as_str());
        }
        for item in &snapshot.mods {
            updates.push(item.as_str());
        }
        updates.sort();
        updates.dedup();

        let mut deletes: Vec<&str> = snapshot.deletes.iter().map(|s| s.as_str()).collect();
        deletes.sort();
        deletes.dedup();

        let mut hasher = Sha256::new();
        hasher.update(b"updates\0");
        for item in updates {
            hasher.update(item.as_bytes());
            hasher.update(b"\n");
        }
        hasher.update(b"deletes\0");
        for item in deletes {
            hasher.update(item.as_bytes());
            hasher.update(b"\n");
        }

        let digest = hasher.finalize();
        let mut hex = String::with_capacity(digest.len() * 2);
        for b in digest {
            use std::fmt::Write as _;
            let _ = write!(&mut hex, "{:02x}", b);
        }
        hex
    }

    #[cfg(any(target_os = "linux", test))]
    fn pending_diff_id_for_diff(diff: &substrate_common::FsDiff) -> String {
        let snapshot = Self::normalize_pending_diff_bucket(diff);
        Self::pending_diff_id_for_snapshot(&snapshot)
    }

    #[cfg(target_os = "linux")]
    fn note_pending_diff_origin_for_world(
        &self,
        world_id: &str,
        origin: PendingDiffOrigin,
        snapshot: &PendingDiffBucketV1,
    ) {
        let mut current_paths: HashSet<String> = HashSet::new();
        current_paths.extend(snapshot.writes.iter().cloned());
        current_paths.extend(snapshot.mods.iter().cloned());
        current_paths.extend(snapshot.deletes.iter().cloned());

        let mut guard = self
            .pending_diff_origin
            .write()
            .expect("pending diff origin tracker lock poisoned");

        if current_paths.is_empty() {
            guard.remove(world_id);
            return;
        }

        let tracker = guard.entry(world_id.to_string()).or_default();

        for removed in tracker.last_seen_paths.difference(&current_paths) {
            tracker.origin_by_path.remove(removed);
        }
        for added in current_paths.difference(&tracker.last_seen_paths) {
            tracker.origin_by_path.insert(added.clone(), origin);
        }

        tracker.last_seen_paths = current_paths;
    }

    #[cfg(target_os = "linux")]
    pub(crate) fn note_pty_pending_diff(&self, world_id: &str) {
        let world = WorldHandle {
            id: world_id.to_string(),
            shared_binding: None,
        };
        let (_started_at, diff) = match self.linux_backend.pending_diff(&world) {
            Ok(v) => v,
            Err(err) => {
                tracing::warn!(error = %err, world_id = world_id, "failed to snapshot pending diff after PTY command");
                return;
            }
        };

        let snapshot = Self::normalize_pending_diff_bucket(&diff);
        self.note_pending_diff_origin_for_world(world_id, PendingDiffOrigin::Pty, &snapshot);
    }

    #[cfg(not(target_os = "linux"))]
    fn create_backend() -> Result<Arc<dyn WorldBackend>> {
        anyhow::bail!("World agent only supported on Linux inside VMs")
    }

    /// Execute a command with budget tracking.
    pub async fn execute(&self, req: ExecuteRequest) -> Result<ExecuteResponse> {
        if req
            .member_dispatch
            .as_ref()
            .is_some_and(|dispatch| dispatch.config_projection().is_some())
        {
            return Err(BadRequestError::new(
                "member_dispatch V2 launch is not implemented".to_string(),
            )
            .into());
        }
        if req.member_dispatch.is_some() {
            return Err(BadRequestError::new(
                "member_dispatch requires POST /v1/execute/stream".to_string(),
            )
            .into());
        }

        // Validate agent_id
        if req.agent_id.is_empty() {
            anyhow::bail!("agent_id is required for API calls");
        }

        let always_isolate = should_always_isolate(&req);

        // Apply and track budget
        if let Some(budget) = req.budget.clone() {
            {
                let mut budgets = self.budgets.write().unwrap();
                let tracker = budgets
                    .entry(req.agent_id.clone())
                    .or_insert_with(|| AgentBudgetTracker::new(&req.agent_id, budget));

                // Check budget before execution
                if !tracker.can_execute() {
                    anyhow::bail!("Budget exceeded: max executions reached");
                }

                tracker.decrement_exec();
            }
        }

        let cwd = req
            .cwd
            .clone()
            .map(PathBuf::from)
            .unwrap_or_else(|| std::env::current_dir().unwrap_or_default());
        let env_ref = req.env.as_ref();
        let project_dir = resolve_project_dir(env_ref, Some(&cwd))?;
        let (policy_resolution_mode, policy_inputs) = resolve_policy_inputs(
            &req.policy_snapshot,
            req.world_network.as_ref(),
            &cwd,
            &project_dir,
        )?;

        let PolicyInputs {
            fs_mode,
            isolation_full,
            isolate_network,
            allowed_domains,
            write_allowlist_prefixes,
            landlock_discover_paths,
            landlock_read_paths,
            landlock_write_paths,
            enforcement_plan_b64,
        } = policy_inputs;
        self.record_doctor_request_context(policy_resolution_mode, isolate_network);

        let host_visible = !isolation_full;
        let empty_env: HashMap<String, String> = HashMap::new();
        let guard_env = req.env.as_ref().unwrap_or(&empty_env);
        let guard_deny =
            crate::world_exec_guard::check_command(&req.cmd, &cwd, guard_env, host_visible);

        // Create world spec from request
        let spec = build_world_spec(
            project_dir,
            always_isolate,
            fs_mode,
            isolate_network,
            allowed_domains,
            req.shared_world.as_ref(),
        );

        // Ensure world exists
        #[cfg(target_os = "linux")]
        let (world, _privileged_child_exclusion_lease) =
            match self.ensure_non_e3_world_session(&spec) {
                Ok(selection) => selection,
                Err(e) => {
                    if e.to_string().contains("UnsupportedSecurityPosture") {
                        return Err(e);
                    }
                    self.record_last_netfilter_failure_for_error(isolate_network, &e);
                    tracing::error!(error = %e, error_debug = ?e, "ensure_session failed");
                    return Err(anyhow::anyhow!("Failed to ensure session world"));
                }
            };
        #[cfg(not(target_os = "linux"))]
        let world = match self.backend.ensure_session(&spec) {
            Ok(w) => w,
            Err(e) => {
                self.record_last_netfilter_failure_for_error(isolate_network, &e);
                tracing::error!(error = %e, error_debug = ?e, "ensure_session failed");
                return Err(anyhow::anyhow!("Failed to ensure session world"));
            }
        };
        let shared_world = resolve_shared_world_binding(req.shared_world.as_ref(), &world)?;

        if let Some(deny) = guard_deny {
            let span_id = format!("spn_{}", uuid::Uuid::now_v7());
            let message = crate::world_exec_guard::deny_message(&deny);
            return Ok(ExecuteResponse {
                exit: 5,
                span_id,
                stdout_b64: BASE64.encode(b""),
                stderr_b64: BASE64.encode(message.as_bytes()),
                scopes_used: Vec::new(),
                fs_diff: None,
                shared_world,
                process_telemetry: ProcessTelemetry::default(),
            });
        }

        // Prepare execution request
        let mut env_map = req.env.unwrap_or_default();
        env_map.remove(WORLD_PROJECT_DIR_OVERRIDE_ENV);
        // Snapshot isolation should not rely on callers setting legacy env toggles.
        let isolation = if isolation_full { "full" } else { "workspace" };
        env_map.insert(WORLD_FS_ISOLATION_ENV.to_string(), isolation.to_string());
        if isolation_full && !write_allowlist_prefixes.is_empty() {
            env_map.insert(
                WORLD_FS_WRITE_ALLOWLIST_ENV.to_string(),
                write_allowlist_prefixes.join("\n"),
            );
        }
        #[cfg(target_os = "linux")]
        let landlock_supported = world::landlock::detect_support().supported;
        #[cfg(not(target_os = "linux"))]
        let landlock_supported = false;

        if isolation_full {
            apply_full_isolation_helper_env(
                &mut env_map,
                landlock_supported,
                &landlock_discover_paths,
                &landlock_read_paths,
                &landlock_write_paths,
                enforcement_plan_b64.as_deref(),
            );
        }
        let exec_req = world_api::ExecRequest {
            cmd: wrap_command_for_profile(req.profile.as_deref(), &cwd, &req.cmd),
            cwd,
            env: env_map,
            pty: req.pty,
            span_id: None,
            shared_world: req.shared_world.clone(),
            member_dispatch: req
                .member_dispatch
                .as_ref()
                .map(convert_member_dispatch_request)
                .transpose()?,
        };

        // Execute command
        let attempt_id = uuid::Uuid::now_v7();
        let result = match self.backend.exec(&world, exec_req) {
            Ok(r) => r,
            Err(e) => {
                self.record_last_netfilter_failure_for_error(isolate_network, &e);
                return Err(command_execution_failed(&e, attempt_id));
            }
        };

        self.clear_last_netfilter_failure_on_success(isolate_network);

        #[cfg(target_os = "linux")]
        {
            if let Some(ref diff) = result.fs_diff {
                let snapshot = Self::normalize_pending_diff_bucket(diff);
                self.note_pending_diff_origin_for_world(
                    &world.id,
                    PendingDiffOrigin::NonPty,
                    &snapshot,
                );
            }
        }

        // Generate span ID
        let span_id = format!("spn_{}", uuid::Uuid::now_v7());

        Ok(ExecuteResponse {
            exit: result.exit,
            span_id,
            stdout_b64: BASE64.encode(result.stdout),
            stderr_b64: BASE64.encode(result.stderr),
            scopes_used: result.scopes_used,
            fs_diff: result.fs_diff,
            shared_world,
            process_telemetry: result.process_telemetry,
        })
    }

    /// Retrieve the current session's pending diff record.
    pub async fn pending_diff(&self, req: PendingDiffRequestV1) -> Result<PendingDiffRecordV1> {
        #[cfg(target_os = "linux")]
        {
            if req.agent_id.is_empty() {
                anyhow::bail!("agent_id is required for API calls");
            }

            let _privileged_child_exclusion_lease = self.acquire_non_e3_helper_operation()?;

            let cwd = req
                .cwd
                .clone()
                .map(PathBuf::from)
                .unwrap_or_else(|| std::env::current_dir().unwrap_or_default());
            let env_ref = req.env.as_ref();
            let project_dir = resolve_project_dir(env_ref, Some(&cwd))?;

            let snapshot = req.policy_snapshot.clone();
            let (_, policy_inputs) =
                resolve_policy_inputs(&snapshot, req.world_network.as_ref(), &cwd, &project_dir)?;
            let fs_mode = policy_inputs.fs_mode;
            let isolate_network = policy_inputs.isolate_network;
            let allowed_domains = policy_inputs.allowed_domains.clone();

            let always_isolate = should_always_isolate_for_profile(req.profile.as_deref());

            let spec = build_world_spec(
                project_dir,
                always_isolate,
                fs_mode,
                isolate_network,
                allowed_domains,
                None,
            );

            let world = match self.backend.ensure_session(&spec) {
                Ok(w) => w,
                Err(e) => {
                    tracing::error!(error = %e, error_debug = ?e, "ensure_session failed");
                    return Err(anyhow::anyhow!("Failed to ensure session world"));
                }
            };

            let (started_at, diff) = self
                .linux_backend
                .pending_diff(&world)
                .context("pending_diff failed")?;

            let snapshot = Self::normalize_pending_diff_bucket(&diff);
            let hex = Self::pending_diff_id_for_snapshot(&snapshot);

            let started_at: chrono::DateTime<chrono::Utc> = started_at.into();
            let session_started_at =
                started_at.to_rfc3339_opts(SecondsFormat::Nanos, /* use_z */ true);

            let mut non_pty = PendingDiffBucketV1::default();
            let mut pty = PendingDiffBucketV1::default();

            let origin_map: Option<HashMap<String, PendingDiffOrigin>> = self
                .pending_diff_origin
                .read()
                .ok()
                .and_then(|guard| guard.get(&world.id).map(|t| t.origin_by_path.clone()));

            let classify = |path: &str| -> PendingDiffOrigin {
                origin_map
                    .as_ref()
                    .and_then(|m| m.get(path).copied())
                    .unwrap_or(PendingDiffOrigin::NonPty)
            };

            for item in &snapshot.writes {
                match classify(item) {
                    PendingDiffOrigin::NonPty => non_pty.writes.push(item.clone()),
                    PendingDiffOrigin::Pty => pty.writes.push(item.clone()),
                }
            }
            for item in &snapshot.mods {
                match classify(item) {
                    PendingDiffOrigin::NonPty => non_pty.mods.push(item.clone()),
                    PendingDiffOrigin::Pty => pty.mods.push(item.clone()),
                }
            }
            for item in &snapshot.deletes {
                match classify(item) {
                    PendingDiffOrigin::NonPty => non_pty.deletes.push(item.clone()),
                    PendingDiffOrigin::Pty => pty.deletes.push(item.clone()),
                }
            }

            Ok(PendingDiffRecordV1 {
                schema_version: 1,
                session_started_at,
                diff_id: hex,
                non_pty,
                pty: Some(pty),
            })
        }

        #[cfg(not(target_os = "linux"))]
        {
            let _ = req;
            anyhow::bail!("pending diff discovery is only supported on Linux agents")
        }
    }

    /// Conditionally clear the current session's pending diff snapshot.
    pub async fn pending_diff_clear(
        &self,
        req: PendingDiffClearRequestV1,
    ) -> Result<PendingDiffClearResponseV1> {
        #[cfg(target_os = "linux")]
        {
            if req.agent_id.is_empty() {
                anyhow::bail!("agent_id is required for API calls");
            }

            let _privileged_child_exclusion_lease = self.acquire_non_e3_helper_operation()?;

            let cwd = req
                .cwd
                .clone()
                .map(PathBuf::from)
                .unwrap_or_else(|| std::env::current_dir().unwrap_or_default());
            let env_ref = req.env.as_ref();
            let project_dir = resolve_project_dir(env_ref, Some(&cwd))?;

            let snapshot = req.policy_snapshot.clone();
            let (_, policy_inputs) =
                resolve_policy_inputs(&snapshot, req.world_network.as_ref(), &cwd, &project_dir)?;
            let fs_mode = policy_inputs.fs_mode;
            let isolate_network = policy_inputs.isolate_network;
            let allowed_domains = policy_inputs.allowed_domains.clone();

            let always_isolate = should_always_isolate_for_profile(req.profile.as_deref());

            let spec = build_world_spec(
                project_dir,
                always_isolate,
                fs_mode,
                isolate_network,
                allowed_domains,
                None,
            );

            let world = match self.backend.ensure_session(&spec) {
                Ok(w) => w,
                Err(e) => {
                    tracing::error!(error = %e, error_debug = ?e, "ensure_session failed");
                    return Err(anyhow::anyhow!("Failed to ensure session world"));
                }
            };

            let (_started_at, diff) = self
                .linux_backend
                .pending_diff(&world)
                .context("pending_diff failed")?;
            let current_hex = Self::pending_diff_id_for_diff(&diff);

            if current_hex != req.diff_id {
                return Ok(PendingDiffClearResponseV1 {
                    schema_version: 1,
                    cleared: false,
                });
            }

            self.linux_backend
                .clear_pending_diff(&world)
                .context("clear_pending_diff failed")?;

            if let Ok(mut guard) = self.pending_diff_origin.write() {
                guard.remove(&world.id);
            }

            Ok(PendingDiffClearResponseV1 {
                schema_version: 1,
                cleared: true,
            })
        }

        #[cfg(not(target_os = "linux"))]
        {
            let _ = req;
            anyhow::bail!("pending diff clear is only supported on Linux agents")
        }
    }

    /// Conditionally reconcile (discard) specific pending diff paths by mutating the overlay upper/work layers.
    pub async fn pending_diff_reconcile(
        &self,
        req: PendingDiffReconcileRequestV1,
    ) -> Result<PendingDiffReconcileResponseV1> {
        #[cfg(target_os = "linux")]
        {
            if req.agent_id.is_empty() {
                anyhow::bail!("agent_id is required for API calls");
            }

            let _privileged_child_exclusion_lease = self.acquire_non_e3_helper_operation()?;

            let cwd = req
                .cwd
                .clone()
                .map(PathBuf::from)
                .unwrap_or_else(|| std::env::current_dir().unwrap_or_default());
            let env_ref = req.env.as_ref();
            let project_dir = resolve_project_dir(env_ref, Some(&cwd))?;

            let snapshot = req.policy_snapshot.clone();
            let (_, policy_inputs) =
                resolve_policy_inputs(&snapshot, req.world_network.as_ref(), &cwd, &project_dir)?;
            let fs_mode = policy_inputs.fs_mode;
            let isolate_network = policy_inputs.isolate_network;
            let allowed_domains = policy_inputs.allowed_domains.clone();

            let always_isolate = should_always_isolate_for_profile(req.profile.as_deref());

            let spec = build_world_spec(
                project_dir,
                always_isolate,
                fs_mode,
                isolate_network,
                allowed_domains,
                None,
            );

            let world = match self.backend.ensure_session(&spec) {
                Ok(w) => w,
                Err(e) => {
                    tracing::error!(error = %e, error_debug = ?e, "ensure_session failed");
                    return Err(anyhow::anyhow!("Failed to ensure session world"));
                }
            };

            let (_started_at, diff) = self
                .linux_backend
                .pending_diff(&world)
                .context("pending_diff failed")?;
            let current_hex = Self::pending_diff_id_for_diff(&diff);

            if current_hex != req.diff_id {
                return Ok(PendingDiffReconcileResponseV1 {
                    schema_version: 1,
                    reconciled: false,
                    discarded: 0,
                });
            }

            let mut rel_paths: Vec<PathBuf> = Vec::new();
            let mut normalized_for_tracker: Vec<String> = Vec::new();
            for raw in &req.discard_paths {
                let mut rel = raw.trim().replace('\\', "/");
                while let Some(stripped) = rel.strip_prefix("./") {
                    rel = stripped.to_string();
                }
                if rel.is_empty() {
                    return Err(BadRequestError::new("empty discard path".to_string()).into());
                }
                if rel.starts_with('/') {
                    return Err(BadRequestError::new(format!(
                        "absolute paths are not allowed: {rel}"
                    ))
                    .into());
                }
                if rel.len() >= 2 && rel.as_bytes()[1] == b':' {
                    return Err(BadRequestError::new(format!(
                        "absolute paths are not allowed: {rel}"
                    ))
                    .into());
                }
                if rel.split('/').any(|segment| segment == "..") {
                    return Err(BadRequestError::new(format!(
                        "path segments must not be '..': {rel}"
                    ))
                    .into());
                }
                rel_paths.push(PathBuf::from(&rel));
                normalized_for_tracker.push(rel);
            }

            let removed = self
                .linux_backend
                .discard_pending_paths(&world, &rel_paths)
                .context("discard_pending_paths failed")?;

            if !normalized_for_tracker.is_empty() {
                if let Ok(mut guard) = self.pending_diff_origin.write() {
                    if let Some(tracker) = guard.get_mut(&world.id) {
                        for p in &normalized_for_tracker {
                            tracker.origin_by_path.remove(p);
                            tracker.last_seen_paths.remove(p);
                        }
                    }
                }
            }

            Ok(PendingDiffReconcileResponseV1 {
                schema_version: 1,
                reconciled: true,
                discarded: removed,
            })
        }

        #[cfg(not(target_os = "linux"))]
        {
            let _ = req;
            anyhow::bail!("pending diff reconcile is only supported on Linux agents")
        }
    }

    /// Read metadata and optionally contents from the current session's overlay filesystem.
    pub async fn world_fs_read(&self, req: WorldFsReadRequestV1) -> Result<WorldFsReadResponseV1> {
        #[cfg(target_os = "linux")]
        {
            if req.agent_id.is_empty() {
                anyhow::bail!("agent_id is required for API calls");
            }

            let mut rel = req.path.trim().replace('\\', "/");
            while let Some(stripped) = rel.strip_prefix("./") {
                rel = stripped.to_string();
            }
            if rel.is_empty() {
                return Err(BadRequestError::new("empty path".to_string()).into());
            }
            if rel.starts_with('/') {
                return Err(
                    BadRequestError::new(format!("absolute paths are not allowed: {rel}")).into(),
                );
            }
            if rel.len() >= 2 && rel.as_bytes()[1] == b':' {
                return Err(
                    BadRequestError::new(format!("absolute paths are not allowed: {rel}")).into(),
                );
            }
            if rel.split('/').any(|segment| segment == "..") {
                return Err(
                    BadRequestError::new(format!("path segments must not be '..': {rel}")).into(),
                );
            }

            let _privileged_child_exclusion_lease = self.acquire_non_e3_helper_operation()?;

            let cwd = req
                .cwd
                .clone()
                .map(PathBuf::from)
                .unwrap_or_else(|| std::env::current_dir().unwrap_or_default());
            let env_ref = req.env.as_ref();
            let project_dir = resolve_project_dir(env_ref, Some(&cwd))?;

            let snapshot = req.policy_snapshot.clone();
            let (_, policy_inputs) =
                resolve_policy_inputs(&snapshot, req.world_network.as_ref(), &cwd, &project_dir)?;
            let fs_mode = policy_inputs.fs_mode;
            let isolate_network = policy_inputs.isolate_network;
            let allowed_domains = policy_inputs.allowed_domains.clone();

            let always_isolate = should_always_isolate_for_profile(req.profile.as_deref());

            let spec = build_world_spec(
                project_dir,
                always_isolate,
                fs_mode,
                isolate_network,
                allowed_domains,
                None,
            );

            let world = match self.backend.ensure_session(&spec) {
                Ok(w) => w,
                Err(e) => {
                    tracing::error!(error = %e, error_debug = ?e, "ensure_session failed");
                    return Err(anyhow::anyhow!("Failed to ensure session world"));
                }
            };

            let overlay_root = self
                .linux_backend
                .ensure_overlay_root(&world)
                .context("ensure_overlay_root failed")?;

            let full_path = overlay_root.join(&rel);
            let meta = std::fs::symlink_metadata(&full_path).with_context(|| {
                format!(
                    "failed to read metadata for world fs path {}",
                    full_path.display()
                )
            })?;

            let ft = meta.file_type();
            let entry_type = if ft.is_file() {
                WorldFsEntryTypeV1::RegularFile
            } else if ft.is_dir() {
                WorldFsEntryTypeV1::Directory
            } else if ft.is_symlink() {
                WorldFsEntryTypeV1::Symlink
            } else if ft.is_socket() {
                WorldFsEntryTypeV1::Socket
            } else if ft.is_fifo() {
                WorldFsEntryTypeV1::Fifo
            } else if ft.is_block_device() {
                WorldFsEntryTypeV1::BlockDevice
            } else if ft.is_char_device() {
                WorldFsEntryTypeV1::CharDevice
            } else {
                WorldFsEntryTypeV1::Unknown
            };

            let mode = Some(meta.permissions().mode());
            let size = match entry_type {
                WorldFsEntryTypeV1::RegularFile => Some(meta.len()),
                _ => None,
            };

            let contents_b64 =
                if req.include_contents && entry_type == WorldFsEntryTypeV1::RegularFile {
                    let bytes = std::fs::read(&full_path).with_context(|| {
                        format!(
                            "failed to read bytes for world fs path {}",
                            full_path.display()
                        )
                    })?;
                    Some(BASE64.encode(bytes))
                } else {
                    None
                };

            Ok(WorldFsReadResponseV1 {
                schema_version: 1,
                path: rel,
                entry_type,
                size,
                mode,
                contents_b64,
            })
        }

        #[cfg(not(target_os = "linux"))]
        {
            let _ = req;
            anyhow::bail!("world fs read is only supported on Linux agents")
        }
    }

    /// Return the typed gateway lifecycle/status surface.
    pub async fn gateway_status(
        &self,
        req: GatewayLifecycleRequestV1,
    ) -> Result<GatewayLifecycleResponseV1> {
        #[cfg(target_os = "linux")]
        {
            if req.schema_version != 1 || req.ordinary_selection.is_some() {
                let (effect, owned_gateway) = self.prepare_ordinary_gateway_request(&req)?;
                effect.revalidate_for_effect()?;
                let response = self
                    .gateway_runtime
                    .status(
                        &owned_gateway.runtime_id,
                        &effect.requested_selection().backend_id,
                    )
                    .await
                    .map_err(gateway_runtime_error)?;
                return Self::attach_gateway_request_metadata(response, &req);
            }
            let prepared = self.prepare_gateway_runtime_request(&req)?;
            let Some(binding) = self
                .resolve_gateway_runtime_binding(
                    prepared,
                    GatewayRuntimeBindingMode::ExistingSessionOnly,
                )
                .map_err(gateway_runtime_error)?
            else {
                return Self::attach_gateway_request_metadata(
                    Self::gateway_unavailable_response(),
                    &req,
                );
            };

            let response = self
                .gateway_runtime
                .status(
                    &binding.runtime_id,
                    binding.start_context.binding.backend_id,
                )
                .await
                .map_err(gateway_runtime_error)?;

            Self::attach_gateway_request_metadata(response, &req)
        }

        #[cfg(not(target_os = "linux"))]
        {
            Self::attach_gateway_request_metadata(Self::gateway_unavailable_response(), &req)
        }
    }

    /// Return the typed gateway lifecycle sync surface.
    pub async fn gateway_sync(
        &self,
        req: GatewayLifecycleRequestV1,
    ) -> Result<GatewayLifecycleResponseV1> {
        #[cfg(target_os = "linux")]
        {
            if req.schema_version != 1 || req.ordinary_selection.is_some() {
                let (effect, owned_gateway) = self.prepare_ordinary_gateway_request(&req)?;
                let (wiring, _owned_gateway) = effect
                    .prepare_ordinary_native_gateway(Some(owned_gateway))
                    .await?;
                return Self::attach_gateway_request_metadata(
                    GatewayLifecycleResponseV1 {
                        status: transport_api_types::GatewayStatusV1::Available,
                        client_wiring: Some(wiring),
                        identity_tuple: None,
                        placement_posture: None,
                    },
                    &req,
                );
            }
            let prepared = self.prepare_gateway_runtime_request(&req)?;
            let Some(binding) = self
                .resolve_gateway_runtime_binding(prepared, GatewayRuntimeBindingMode::EnsureSession)
                .map_err(gateway_runtime_error)?
            else {
                return Self::attach_gateway_request_metadata(
                    Self::gateway_unavailable_response(),
                    &req,
                );
            };

            let response = self
                .gateway_runtime
                .sync(binding.start_context)
                .await
                .map_err(gateway_runtime_error)?;

            Self::attach_gateway_request_metadata(response, &req)
        }

        #[cfg(not(target_os = "linux"))]
        {
            Self::attach_gateway_request_metadata(Self::gateway_unavailable_response(), &req)
        }
    }

    /// Return the typed gateway lifecycle restart surface.
    pub async fn gateway_restart(
        &self,
        req: GatewayLifecycleRequestV1,
    ) -> Result<GatewayLifecycleResponseV1> {
        #[cfg(target_os = "linux")]
        {
            if req.schema_version != 1 || req.ordinary_selection.is_some() {
                let (effect, owned_gateway) = self.prepare_ordinary_gateway_request(&req)?;
                let (wiring, _owned_gateway) = effect
                    .prepare_ordinary_native_gateway(Some(owned_gateway))
                    .await?;
                return Self::attach_gateway_request_metadata(
                    GatewayLifecycleResponseV1 {
                        status: transport_api_types::GatewayStatusV1::Available,
                        client_wiring: Some(wiring),
                        identity_tuple: None,
                        placement_posture: None,
                    },
                    &req,
                );
            }
            let prepared = self.prepare_gateway_runtime_request(&req)?;
            let Some(binding) = self
                .resolve_gateway_runtime_binding(
                    prepared,
                    GatewayRuntimeBindingMode::ExistingSessionOnly,
                )
                .map_err(gateway_runtime_error)?
            else {
                return Self::attach_gateway_request_metadata(
                    Self::gateway_unavailable_response(),
                    &req,
                );
            };

            let response = self
                .gateway_runtime
                .restart(binding.start_context)
                .await
                .map_err(gateway_runtime_error)?;

            Self::attach_gateway_request_metadata(response, &req)
        }

        #[cfg(not(target_os = "linux"))]
        {
            Self::attach_gateway_request_metadata(Self::gateway_unavailable_response(), &req)
        }
    }

    /// Execute a command and stream incremental output frames via NDJSON.
    #[cfg(target_os = "linux")]
    pub async fn execute_stream(&self, req: ExecuteRequest) -> Result<Response> {
        if req.schema_version != 1 || req.ordinary_start_selection.is_some() {
            return self.execute_ordinary_member(req).await;
        }
        if req
            .member_dispatch
            .as_ref()
            .is_some_and(|dispatch| dispatch.config_projection().is_some())
        {
            return Err(BadRequestError::new(
                "member_dispatch V2 launch is not implemented".to_string(),
            )
            .into());
        }
        if req.agent_id.is_empty() {
            anyhow::bail!("agent_id is required for API calls");
        }

        if req.pty {
            anyhow::bail!("PTY streaming is handled via /v1/stream");
        }

        let privileged_child_exclusion_lease = self.acquire_non_e3_helper_operation()?;

        if let Some(budget) = req.budget.clone() {
            let mut budgets = self.budgets.write().unwrap();
            let tracker = budgets
                .entry(req.agent_id.clone())
                .or_insert_with(|| AgentBudgetTracker::new(&req.agent_id, budget));
            if !tracker.can_execute() {
                anyhow::bail!("Budget exceeded: max executions reached");
            }
            tracker.decrement_exec();
        }

        let cwd = req
            .cwd
            .clone()
            .map(PathBuf::from)
            .unwrap_or_else(|| std::env::current_dir().unwrap_or_default());
        let env_ref = req.env.as_ref();
        let project_dir = resolve_project_dir(env_ref, Some(&cwd))?;
        let (policy_resolution_mode, policy_inputs) = resolve_policy_inputs(
            &req.policy_snapshot,
            req.world_network.as_ref(),
            &cwd,
            &project_dir,
        )?;

        let PolicyInputs {
            fs_mode,
            isolation_full,
            isolate_network,
            allowed_domains,
            write_allowlist_prefixes,
            landlock_discover_paths,
            landlock_read_paths,
            landlock_write_paths,
            enforcement_plan_b64,
        } = policy_inputs;
        self.record_doctor_request_context(policy_resolution_mode, isolate_network);
        let always_isolate = should_always_isolate(&req);

        let shared_owner_spec = requested_shared_world_owner_spec(&req);

        let spec = build_world_spec(
            project_dir.clone(),
            always_isolate,
            fs_mode,
            isolate_network,
            allowed_domains,
            shared_owner_spec.as_ref(),
        );

        let exact_bound_world_adoption = req
            .member_dispatch
            .as_ref()
            .map(|dispatch| {
                exact_bound_world_ownership_adoption(dispatch, &req.policy_snapshot, &spec)
            })
            .transpose()?
            .flatten();

        let durable_world = select_execute_stream_world(
            exact_bound_world_adoption.as_ref(),
            |adoption| {
                self.linux_backend
                    .adopt_exact_bound_world_ownership(adoption)
            },
            || self.backend.ensure_session(&spec),
        );
        let world = match durable_world {
            Ok(selection) => selection.into_world(),
            Err(e) => {
                self.record_last_netfilter_failure_for_error(isolate_network, &e);
                if exact_bound_world_adoption.is_some() {
                    tracing::error!(error = %e, "exact bound-world adoption failed");
                    anyhow::bail!("Failed to adopt exact session world");
                }
                tracing::error!(error = %e, error_debug = ?e, "ensure_session failed");
                anyhow::bail!("Failed to ensure session world");
            }
        };
        let shared_world = resolve_shared_world_binding(shared_owner_spec.as_ref(), &world)?;

        if let Some(dispatch) = req.member_dispatch.clone() {
            let acceptance_context = req.acceptance_context.clone();
            let placement = self.resolve_authoritative_member_placement_context(
                &dispatch,
                world,
                shared_world.as_ref(),
                &project_dir,
                &cwd,
                isolation_full,
            )?;
            let env_map = build_member_runtime_launch_env(
                req.env.clone().unwrap_or_default(),
                MemberRuntimeLaunchEnvContext {
                    placement_root: &placement.placement_root,
                    isolation_full,
                    write_allowlist_prefixes: &write_allowlist_prefixes,
                    landlock_discover_paths: &landlock_discover_paths,
                    landlock_read_paths: &landlock_read_paths,
                    landlock_write_paths: &landlock_write_paths,
                    enforcement_plan_b64: enforcement_plan_b64.as_deref(),
                },
            );
            let span_id = format!("spn_{}", uuid::Uuid::now_v7());
            let launch_placement = placement.launch_placement();
            let dispatch = dispatch.as_v1().cloned().ok_or_else(|| {
                BadRequestError::new("member_dispatch V2 launch is not implemented".to_string())
            })?;
            return self
                .member_runtime
                .launch(
                    req.agent_id.clone(),
                    env_map,
                    span_id,
                    MemberRuntimeLaunchAdmissionV1 {
                        dispatch,
                        acceptance_context,
                        policy_snapshot: req.policy_snapshot.clone(),
                        ordinary_effect: None,
                    },
                    placement.binding,
                    launch_placement,
                )
                .await;
        }

        let host_visible = !isolation_full;
        let empty_env: HashMap<String, String> = HashMap::new();
        let guard_env = req.env.as_ref().unwrap_or(&empty_env);
        let guard_deny =
            crate::world_exec_guard::check_command(&req.cmd, &cwd, guard_env, host_visible);

        if let Some(deny) = guard_deny {
            let span_id = format!("spn_{}", uuid::Uuid::now_v7());
            let message = crate::world_exec_guard::deny_message(&deny);
            let mut producer = RuntimeEventStreamProducer::new();
            let start = producer.start(span_id.clone())?;
            let replay_publisher = self
                .runtime_replay
                .begin_for_acceptance(req.acceptance_context.as_ref(), &start)?;
            let stderr = producer.stderr(BASE64.encode(message.as_bytes()))?;
            let exit = producer.exit(5, span_id, Vec::new(), None, ProcessTelemetry::default())?;
            if let Some(publisher) = replay_publisher.as_ref() {
                for frame in [&stderr, &exit] {
                    if let Err(error) = publisher.publish(frame) {
                        tracing::error!(error = %error, "failed to retain replayable guard-deny frame");
                    }
                }
            }
            let frames = vec![start, stderr, exit];

            let stream = futures_util::stream::iter(frames.into_iter().map(|frame| {
                let payload = frame
                    .canonical_ndjson_bytes()
                    .expect("serialize identified frame");
                Ok::<Bytes, Infallible>(Bytes::from(payload))
            }));

            let body = boxed(StreamBody::new(stream));
            let response = Response::builder()
                .status(StatusCode::OK)
                .header("content-type", "application/x-ndjson")
                .body(body)
                .map_err(|e| anyhow!("failed to build stream response: {e}"))?;
            return Ok(response);
        }

        let mut exec_req = world_api::ExecRequest {
            cmd: req.cmd.clone(),
            cwd: cwd.clone(),
            env: {
                let mut env_map = req.env.clone().unwrap_or_default();
                env_map.remove(WORLD_PROJECT_DIR_OVERRIDE_ENV);
                let isolation = if isolation_full { "full" } else { "workspace" };
                env_map.insert(WORLD_FS_ISOLATION_ENV.to_string(), isolation.to_string());
                if isolation_full && !write_allowlist_prefixes.is_empty() {
                    env_map.insert(
                        WORLD_FS_WRITE_ALLOWLIST_ENV.to_string(),
                        write_allowlist_prefixes.join("\n"),
                    );
                }
                #[cfg(target_os = "linux")]
                let landlock_supported = world::landlock::detect_support().supported;
                #[cfg(not(target_os = "linux"))]
                let landlock_supported = false;

                if isolation_full {
                    apply_full_isolation_helper_env(
                        &mut env_map,
                        landlock_supported,
                        &landlock_discover_paths,
                        &landlock_read_paths,
                        &landlock_write_paths,
                        enforcement_plan_b64.as_deref(),
                    );
                }
                env_map
            },
            pty: false,
            span_id: None,
            shared_world: shared_owner_spec.clone(),
            member_dispatch: req
                .member_dispatch
                .as_ref()
                .map(convert_member_dispatch_request)
                .transpose()?,
        };

        let span_id = format!("spn_{}", uuid::Uuid::now_v7());
        exec_req.span_id = Some(span_id.clone());
        world::exec::note_pending_exec(&span_id);

        let (tx, rx) = tokio::sync::mpsc::unbounded_channel::<ExecuteStreamFrame>();
        let producer = Arc::new(Mutex::new(RuntimeEventStreamProducer::new()));
        let start = producer
            .lock()
            .expect("runtime event stream producer lock poisoned")
            .start(span_id.clone())?;
        let replay_publisher = self
            .runtime_replay
            .begin_for_acceptance(req.acceptance_context.as_ref(), &start)?;
        let _ = tx.send(start);

        let backend = self.backend.clone();
        #[cfg(target_os = "linux")]
        let service = self.clone();
        let agent_id = req.agent_id.clone();
        let shared_world_for_events = shared_world.clone();
        let span_id_for_cleanup = span_id.clone();
        let producer_for_exec = producer.clone();
        let replay_publisher_for_exec = replay_publisher.clone();
        task::spawn_blocking(move || {
            let _privileged_child_exclusion_lease = privileged_child_exclusion_lease;
            let sink = Arc::new(StreamingSink::new(
                tx.clone(),
                producer_for_exec.clone(),
                replay_publisher_for_exec.clone(),
            ));
            let guard = install_stream_sink(sink);
            let result = backend.exec(&world, exec_req);
            drop(guard);

            match result {
                Ok(exec_result) => {
                    service.clear_last_netfilter_failure_on_success(isolate_network);
                    #[cfg(target_os = "linux")]
                    if let Some(ref diff) = exec_result.fs_diff {
                        let snapshot = WorldService::normalize_pending_diff_bucket(diff);
                        service.note_pending_diff_origin_for_world(
                            &world.id,
                            PendingDiffOrigin::NonPty,
                            &snapshot,
                        );
                    }

                    if let (Some(primary), Some(final_strategy), Some(reason)) = (
                        exec_result.world_fs_strategy_primary,
                        exec_result.world_fs_strategy_final,
                        exec_result.world_fs_strategy_fallback_reason,
                    ) {
                        let event = AgentEvent {
                            ts: chrono::Utc::now(),
                            agent_id: agent_id.clone(),
                            kind: AgentEventKind::Status,
                            orchestration_session_id: shared_world_for_events
                                .as_ref()
                                .map(|binding| binding.orchestration_session_id.clone())
                                .unwrap_or_else(|| span_id.clone()),
                            run_id: span_id.clone(),
                            parent_run_id: None,
                            participant_id: None,
                            parent_participant_id: None,
                            resumed_from_participant_id: None,
                            backend_id: None,
                            thread_id: None,
                            role: None,
                            world_id: Some(
                                shared_world_for_events
                                    .as_ref()
                                    .map(|binding| binding.world_id.clone())
                                    .unwrap_or_else(|| world.id.clone()),
                            ),
                            world_generation: shared_world_for_events
                                .as_ref()
                                .map(|binding| binding.world_generation),
                            cmd_id: None,
                            span_id: Some(span_id.clone()),
                            event_identity: None,
                            worker_event: None,
                            channel: None,
                            identity_tuple: None,
                            placement_posture: None,
                            project: None,
                            data: serde_json::json!({
                                "world_fs_strategy_primary": primary.as_str(),
                                "world_fs_strategy_final": final_strategy.as_str(),
                                "world_fs_strategy_fallback_reason": reason.as_str(),
                            }),
                        };
                        let frame = producer_for_exec
                            .lock()
                            .expect("runtime event stream producer lock poisoned")
                            .event(event);
                        match frame {
                            Ok(frame) => {
                                if let Err(err) = publish_replayable_frame(
                                    replay_publisher_for_exec.as_ref(),
                                    &tx,
                                    frame,
                                ) {
                                    tracing::error!(error = %err, "failed to publish replayable runtime Event");
                                }
                            }
                            Err(err) => {
                                tracing::error!(error = %err, "failed to identify runtime Event");
                            }
                        }
                    }
                    let frame = producer_for_exec
                        .lock()
                        .expect("runtime event stream producer lock poisoned")
                        .exit(
                            exec_result.exit,
                            span_id,
                            exec_result.scopes_used,
                            exec_result.fs_diff,
                            exec_result.process_telemetry,
                        );
                    match frame {
                        Ok(frame) => {
                            if let Err(err) = publish_replayable_frame(
                                replay_publisher_for_exec.as_ref(),
                                &tx,
                                frame,
                            ) {
                                tracing::error!(error = %err, "failed to publish replayable runtime Exit");
                            }
                        }
                        Err(err) => {
                            tracing::error!(error = %err, "failed to identify runtime Exit");
                        }
                    }
                }
                Err(e) => {
                    service.record_last_netfilter_failure_for_error(isolate_network, &e);
                    tracing::error!(error = %e, agent = agent_id, "exec failed");
                    let frame = producer_for_exec
                        .lock()
                        .expect("runtime event stream producer lock poisoned")
                        .transport_error(e.to_string());
                    if let Ok(frame) = frame {
                        if let Err(err) =
                            publish_replayable_frame(replay_publisher_for_exec.as_ref(), &tx, frame)
                        {
                            tracing::error!(error = %err, "failed to publish replayable runtime Error");
                        }
                    }
                }
            }
            world::exec::clear_registered_exec(&span_id_for_cleanup);
        });

        let stream = UnboundedReceiverStream::new(rx).map(|frame| {
            let payload = frame
                .canonical_ndjson_bytes()
                .expect("serialize identified frame");
            Ok::<Bytes, Infallible>(Bytes::from(payload))
        });

        let body = boxed(StreamBody::new(stream));
        let response = Response::builder()
            .status(StatusCode::OK)
            .header("content-type", "application/x-ndjson")
            .body(body)
            .context("Failed to build streaming response")?;

        Ok(response)
    }

    #[cfg(not(target_os = "linux"))]
    pub async fn execute_stream(&self, _req: ExecuteRequest) -> Result<Response> {
        anyhow::bail!("World agent streaming is only supported on Linux");
    }

    #[cfg(target_os = "linux")]
    pub async fn submit_member_turn_stream(
        &self,
        req: MemberTurnSubmitRequestV1,
    ) -> Result<Response> {
        req.validate().map_err(BadRequestError::new)?;
        self.member_runtime.submit_turn(req).await
    }

    #[cfg(target_os = "linux")]
    pub(crate) fn control_ordinary_initial_bootstrap(
        &self,
        request: &transport_api_types::OrdinaryInitialBootstrapControlRequestV1,
        peer_uid: u32,
    ) -> Result<transport_api_types::OrdinaryInitialBootstrapControlDeliveryV1> {
        self.member_runtime
            .control_ordinary_initial_bootstrap(request, peer_uid)
    }

    #[cfg(target_os = "linux")]
    pub(crate) fn cancel_ordinary_accepted_work(
        &self,
        request: &transport_api_types::OrdinaryAcceptedWorkCancelRequestV1,
        peer_uid: u32,
    ) -> Result<bool> {
        self.member_runtime
            .cancel_ordinary_accepted_work(request, peer_uid)
    }

    #[cfg(target_os = "linux")]
    pub(crate) fn retire_ordinary_retained_worker(
        &self,
        request: &transport_api_types::OrdinaryRetainedWorkerStopRequestV1,
        peer_uid: u32,
    ) -> Result<transport_api_types::OrdinaryRetainedWorkerStopResponseV1> {
        self.member_runtime
            .retire_ordinary_member(request, peer_uid)
    }

    /// Dedicated ordinary intake. Neither endpoint choice nor a DTO confers
    /// permission; the manager must authenticate its actual held ordinary member.
    pub(crate) async fn submit_ordinary_member_turn_stream(
        &self,
        req: transport_api_types::OrdinaryMemberTurnSubmitRequestV1,
    ) -> Result<Response> {
        #[cfg(target_os = "linux")]
        {
            self.member_runtime.submit_ordinary_turn(req).await
        }
        #[cfg(not(target_os = "linux"))]
        {
            let _ = req;
            anyhow::bail!("ordinary World member turn submission is only supported on Linux")
        }
    }

    #[cfg(target_os = "linux")]
    pub async fn replay_execute_stream(
        &self,
        req: transport_api_types::ExecuteStreamReplayRequestV1,
    ) -> Result<Response> {
        if let Some(member_turn_join) = self.member_turn_join.as_ref() {
            if let Some(stream) = member_turn_join
                .subscribe_for_replay(
                    &req.acceptance_record_id,
                    &req.stream_id,
                    req.after_frame_sequence,
                )
                .map_err(anyhow::Error::new)?
            {
                let stream = stream.map(|frame| {
                    let payload = frame
                        .canonical_ndjson_bytes()
                        .expect("serialize replayed durable E2 frame");
                    Ok::<Bytes, Infallible>(Bytes::from(payload))
                });
                return Response::builder()
                    .status(StatusCode::OK)
                    .header("content-type", "application/x-ndjson")
                    .body(boxed(StreamBody::new(stream)))
                    .context("failed to build durable E2 runtime replay response");
            }
        }
        let stream = self.runtime_replay.subscribe(&req)?.map(|frame| {
            let payload = frame
                .canonical_ndjson_bytes()
                .expect("serialize replayed identified frame");
            Ok::<Bytes, Infallible>(Bytes::from(payload))
        });
        Response::builder()
            .status(StatusCode::OK)
            .header("content-type", "application/x-ndjson")
            .body(boxed(StreamBody::new(stream)))
            .context("failed to build runtime replay response")
    }

    #[cfg(not(target_os = "linux"))]
    pub async fn replay_execute_stream(
        &self,
        _req: transport_api_types::ExecuteStreamReplayRequestV1,
    ) -> Result<Response> {
        anyhow::bail!("World stream replay is only supported on Linux");
    }

    #[cfg(not(target_os = "linux"))]
    pub async fn submit_member_turn_stream(
        &self,
        _req: MemberTurnSubmitRequestV1,
    ) -> Result<Response> {
        anyhow::bail!("World member turn submission is only supported on Linux");
    }

    #[cfg(target_os = "linux")]
    pub async fn execute_cancel(
        &self,
        req: ExecuteCancelRequestV1,
    ) -> Result<ExecuteCancelResponseV1> {
        if req.span_id.trim().is_empty() {
            return Err(BadRequestError::new("span_id is required".to_string()).into());
        }

        let delivered = self
            .member_runtime
            .cancel_for_legacy_http(&req.span_id, &req.sig)
            .with_context(|| format!("failed to cancel member runtime span {}", req.span_id))?;
        if delivered {
            return Ok(ExecuteCancelResponseV1 {
                schema_version: 1,
                delivered: true,
            });
        }

        let mut delivered = false;
        for _ in 0..80 {
            delivered =
                world::exec::signal_registered_exec(&req.span_id, &req.sig).with_context(|| {
                    format!("failed to cancel streamed execute span {}", req.span_id)
                })?;
            if delivered {
                break;
            }
            tokio::time::sleep(std::time::Duration::from_millis(25)).await;
        }

        Ok(ExecuteCancelResponseV1 {
            schema_version: 1,
            delivered,
        })
    }

    #[cfg(not(target_os = "linux"))]
    pub async fn execute_cancel(
        &self,
        _req: ExecuteCancelRequestV1,
    ) -> Result<ExecuteCancelResponseV1> {
        anyhow::bail!("World agent execute cancellation is only supported on Linux");
    }

    /// Get trace information for a span.
    pub async fn get_trace(&self, span_id: &str) -> Result<serde_json::Value> {
        // TODO: Implement trace retrieval
        Ok(serde_json::json!({
            "span_id": span_id,
            "status": "not_implemented"
        }))
    }

    /// Request additional scopes.
    pub async fn request_scopes(&self, _scopes: Vec<String>) -> Result<serde_json::Value> {
        // TODO: Implement scope request handling
        Ok(serde_json::json!({
            "status": "not_implemented"
        }))
    }

    /// Immutable carrier joins do not grant permission. The executing HSA reader
    /// may reconcile temporary material; its separate reads are not atomic.
    #[cfg(target_os = "linux")]
    fn authenticate_ordinary_start_selection(
        &self,
        carrier: &transport_api_types::OrdinaryStartSelectionCarrierV1,
    ) -> Result<substrate_shell::AuthenticatedOrdinarySelectionV1> {
        carrier.validate().map_err(BadRequestError::new)?;
        let authority = self
            .ordinary_selection_authority
            .as_ref()
            .ok_or_else(|| anyhow!("ordinary installed selection authority is unavailable"))?;
        let selected = authority.authenticate(&carrier.original_session_id)?;
        if carrier.authority_store_id != selected.authority_store_id()
            || carrier.original_session_id != selected.original_session_id()
            || carrier.original_start_intent_id != selected.original_start_intent_id()
            || carrier.original_start_issuer_request_id
                != selected.original_start_issuer_request_id()?
            || carrier.context_sha256 != selected.context_sha256()?
            || carrier.native_selection_sha256 != selected.native_selection().selection_digest
            || carrier.requested_selection_sha256 != selected.requested_selection().selection_digest
        {
            anyhow::bail!("ordinary Start carrier conflicts with authenticated origin");
        }
        selected.revalidate()?;
        Ok(selected)
    }

    #[cfg(target_os = "linux")]
    async fn execute_ordinary_member(&self, req: ExecuteRequest) -> Result<Response> {
        req.validate().map_err(BadRequestError::new)?;
        if req.agent_id.is_empty() || req.pty {
            anyhow::bail!("ordinary member requires agent_id and non-PTY streaming");
        }
        let start = req
            .ordinary_start_selection
            .as_ref()
            .ok_or_else(|| anyhow!("ordinary member omitted immutable Start selection"))?;
        let selected = Arc::new(self.authenticate_ordinary_start_selection(start)?);
        let admitted = Arc::new(selected.prepare_world_execute(&req)?);
        // This authenticates the member/run/current policy/account independently
        // of the still-unimplemented current Host-native-run verifier.
        admitted.revalidate_before_world_effect()?;
        let (effect, actual_backend, world) = OrdinaryWorldEffectContextV1::bind_bootstrap(
            admitted,
            start.clone(),
            req.policy_snapshot.clone(),
            Arc::clone(&self.gateway_runtime),
            Arc::clone(&self.linux_backend),
        )?;
        let dispatch = req
            .member_dispatch
            .as_ref()
            .and_then(transport_api_types::MemberDispatchRequest::as_v1)
            .ok_or_else(|| anyhow!("ordinary member omitted MemberDispatch V1"))?;
        let project_dir = effect.spec.project_dir.clone();
        let (_, inputs) = resolve_policy_inputs(
            &req.policy_snapshot,
            req.world_network.as_ref(),
            &project_dir,
            &project_dir,
        )?;
        effect.revalidate_for_effect()?;
        let placement_root = if inputs.isolation_full {
            actual_backend.ensure_overlay_root(&world)?
        } else {
            project_dir.clone()
        };
        let effective_cwd =
            resolve_member_runtime_effective_cwd(&project_dir, &placement_root, &project_dir)?;
        let env = build_member_runtime_launch_env(
            req.env.clone().unwrap_or_default(),
            MemberRuntimeLaunchEnvContext {
                placement_root: &placement_root,
                isolation_full: inputs.isolation_full,
                write_allowlist_prefixes: &inputs.write_allowlist_prefixes,
                landlock_discover_paths: &inputs.landlock_discover_paths,
                landlock_read_paths: &inputs.landlock_read_paths,
                landlock_write_paths: &inputs.landlock_write_paths,
                enforcement_plan_b64: inputs.enforcement_plan_b64.as_deref(),
            },
        );
        effect.revalidate_for_effect()?;
        self.member_runtime
            .launch(
                req.agent_id,
                env,
                format!("spn_{}", uuid::Uuid::now_v7()),
                MemberRuntimeLaunchAdmissionV1 {
                    dispatch: dispatch.clone(),
                    acceptance_context: req.acceptance_context,
                    policy_snapshot: req.policy_snapshot,
                    ordinary_effect: Some(Arc::clone(&effect)),
                },
                effect.binding.clone(),
                LinuxWorldPlacementContext {
                    working_dir: effective_cwd,
                    cgroup_path: effect.cgroup_path.clone(),
                    require_cgroup_attach: true,
                },
            )
            .await
    }

    #[cfg(target_os = "linux")]
    fn prepare_ordinary_gateway_request(
        &self,
        req: &GatewayLifecycleRequestV1,
    ) -> Result<(
        Arc<OrdinaryWorldEffectContextV1>,
        Arc<crate::gateway_runtime::OrdinaryGatewayRuntimeLeaseV1>,
    )> {
        req.validate_identity_contract()
            .map_err(anyhow::Error::msg)?;
        let context = req
            .ordinary_selection
            .as_ref()
            .ok_or_else(|| anyhow!("selected gateway omitted ordinary context"))?;
        let selected = self.authenticate_ordinary_start_selection(&context.start)?;
        if matches!(
            &context.runtime,
            transport_api_types::OrdinaryGatewayRuntimeIdentityV1::HostActor { .. }
        ) {
            // No original run, metadata sidecar or request claim substitutes for
            // a genuine current Applied-Start/successor Host-run observation.
            anyhow::bail!(
                "gateway_invalid_integration: current Host run verifier is not implemented"
            );
        }
        let (effect, owned_gateway) = self
            .member_runtime
            .ordinary_gateway_effect(&context.runtime)?;
        if effect.start != context.start
            || effect.requested_selection() != selected.requested_selection()
            || req
                .cwd
                .as_deref()
                .is_some_and(|cwd| Path::new(cwd) != effect.spec.project_dir.as_path())
            || serde_json::to_vec(
                &req.policy_snapshot
                    .canonicalize()
                    .map_err(anyhow::Error::msg)?,
            )? != serde_json::to_vec(
                &effect
                    .enforced_policy_snapshot
                    .canonicalize()
                    .map_err(anyhow::Error::msg)?,
            )?
        {
            anyhow::bail!("selected gateway request conflicts with its actual ordinary runtime");
        }
        effect.revalidate_for_effect()?;
        // Request credentials are never used as trusted current custody.
        if let Some(supplied) = req.integrated_auth.as_ref() {
            let current = effect.resolve_current_gateway_handoff()?;
            if supplied.backend_id != current.backend_id
                || supplied.cli_codex.is_some() != current.cli_codex.is_some()
                || supplied.api_env.is_some() != current.api_env.is_some()
            {
                anyhow::bail!("selected gateway auth facets conflict with pinned profile");
            }
            if let (Some(supplied), Some(current)) = (&supplied.cli_codex, &current.cli_codex) {
                let resolve = |auth: &transport_api_types::GatewayCliCodexIntegratedAuthV1| {
                    let mut fields = HashMap::from([(
                        "SUBSTRATE_LLM_BACKEND_AUTH_CLI_CODEX_ACCESS_TOKEN".to_string(),
                        auth.access_token.clone(),
                    )]);
                    if let Some(id) = &auth.account_id {
                        fields.insert(
                            "SUBSTRATE_LLM_BACKEND_AUTH_CLI_CODEX_ACCOUNT_ID".into(),
                            id.clone(),
                        );
                    }
                    substrate_gateway::auth::codex_auth_context::CodexIntegratedAuthHandoff::from_fields(&fields)
                        .and_then(|handoff| handoff.resolve())
                };
                if resolve(supplied)?.account_id != resolve(current)?.account_id {
                    anyhow::bail!(
                        "selected gateway auth account differs from current trusted custody"
                    );
                }
            }
            if let Some(api) = &supplied.api_env {
                if api.env.len() != 1
                    || api
                        .env
                        .get("ANTHROPIC_API_KEY")
                        .is_none_or(|value| value.trim().is_empty())
                {
                    anyhow::bail!("selected Claude gateway auth facet is unsupported");
                }
            }
        }
        effect.revalidate_for_effect()?;
        Ok((effect, owned_gateway))
    }

    #[cfg(target_os = "linux")]
    fn prepare_gateway_runtime_request(
        &self,
        req: &GatewayLifecycleRequestV1,
    ) -> Result<PreparedGatewayRuntimeRequest> {
        req.validate_identity_contract()
            .map_err(|err| anyhow!("gateway_invalid_integration: {err}"))?;
        if req.schema_version != 1 || req.ordinary_selection.is_some() {
            anyhow::bail!("gateway_invalid_integration: selected gateway requires its active ordinary runtime context");
        }
        let cwd = req
            .cwd
            .as_ref()
            .map(PathBuf::from)
            .unwrap_or_else(|| std::env::current_dir().unwrap_or_default());
        let env_ref = req.env.as_ref();
        let project_dir = resolve_project_dir(env_ref, Some(&cwd))?;
        let (policy_resolution_mode, policy_inputs) = resolve_policy_inputs(
            &req.policy_snapshot,
            req.world_network.as_ref(),
            &cwd,
            &project_dir,
        )?;
        self.record_doctor_request_context(policy_resolution_mode, policy_inputs.isolate_network);

        let world_spec = build_world_spec(
            project_dir.clone(),
            should_always_isolate_for_profile(req.profile.as_deref()),
            policy_inputs.fs_mode,
            policy_inputs.isolate_network,
            policy_inputs.allowed_domains,
            None,
        );
        let control =
            GatewayControlSettings::from_request_env(env_ref).map_err(gateway_runtime_error)?;
        let selected_backend = control.default_backend.clone();
        let integrated_auth = match req.integrated_auth.as_ref() {
            Some(payload) => {
                payload
                    .validate_for_selected_backend(&selected_backend)
                    .map_err(|message| {
                        gateway_runtime_error(GatewayRuntimeFailure::invalid_integration(message))
                    })?;
                Some(payload.clone())
            }
            None => None,
        };

        Ok(PreparedGatewayRuntimeRequest {
            project_dir,
            world_spec,
            selected_backend,
            integrated_auth,
        })
    }

    #[cfg(target_os = "linux")]
    fn resolve_authoritative_member_placement_context(
        &self,
        dispatch: &transport_api_types::MemberDispatchRequest,
        world: WorldHandle,
        authoritative_binding: Option<&SharedWorldBindingSnapshot>,
        project_dir: &Path,
        cwd: &Path,
        isolation_full: bool,
    ) -> Result<AuthoritativeMemberPlacementContext> {
        let binding = validate_member_dispatch_binding(dispatch, &world, authoritative_binding)?;
        let cgroup_path = self
            .session_cgroup_path(&world)
            .with_context(|| format!("failed to resolve session cgroup for world {}", world.id))?;
        let placement_root = if isolation_full {
            self.linux_backend
                .ensure_overlay_root(&world)
                .with_context(|| format!("failed to resolve overlay root for world {}", world.id))?
        } else {
            project_dir.to_path_buf()
        };
        let effective_cwd =
            resolve_member_runtime_effective_cwd(project_dir, &placement_root, cwd)?;

        Ok(AuthoritativeMemberPlacementContext {
            world,
            binding,
            placement_root,
            effective_cwd,
            cgroup_path,
        })
    }

    #[cfg(target_os = "linux")]
    fn resolve_gateway_runtime_binding(
        &self,
        prepared: PreparedGatewayRuntimeRequest,
        mode: GatewayRuntimeBindingMode,
    ) -> std::result::Result<Option<ResolvedGatewayRuntimeBinding>, GatewayRuntimeFailure> {
        let Some(binding) = resolve_gateway_backend_binding(&prepared.selected_backend) else {
            return Ok(None);
        };

        if !prepared.world_spec.isolate_network {
            return Ok(Some(ResolvedGatewayRuntimeBinding::non_isolated(
                prepared, binding,
            )));
        }

        let maybe_world = match mode {
            GatewayRuntimeBindingMode::EnsureSession => Some(
                self.backend
                    .ensure_session(&prepared.world_spec)
                    .map_err(|err| {
                        self.record_last_netfilter_failure_for_error(
                            prepared.world_spec.isolate_network,
                            &err,
                        );
                        GatewayRuntimeFailure::transient(format!(
                            "failed to ensure session world: {}",
                            err
                        ))
                    })?,
            ),
            GatewayRuntimeBindingMode::ExistingSessionOnly => self
                .linux_backend
                .find_compatible_session(&prepared.world_spec)
                .map_err(|err| GatewayRuntimeFailure::transient(err.to_string()))?,
        };

        let Some(world) = maybe_world else {
            return Ok(None);
        };
        let cgroup_path = self
            .session_cgroup_path(&world)
            .map_err(|err| GatewayRuntimeFailure::transient(err.to_string()))?;
        let runtime_id = isolated_gateway_runtime_id(&world.id, binding.backend_id);

        Ok(Some(ResolvedGatewayRuntimeBinding {
            runtime_id: runtime_id.clone(),
            start_context: GatewayRuntimeStartContext {
                world_id: runtime_id,
                project_dir: prepared.project_dir,
                cgroup_path,
                require_cgroup_attach: true,
                binding,
                ordinary_effect: None,
                integrated_auth: prepared.integrated_auth,
            },
        }))
    }

    #[cfg(target_os = "linux")]
    pub fn gateway_runtime_pid_for_test(
        &self,
        req: &GatewayLifecycleRequestV1,
    ) -> Result<Option<u32>> {
        let prepared = self.prepare_gateway_runtime_request(req)?;
        let Some(binding) = self
            .resolve_gateway_runtime_binding(
                prepared,
                GatewayRuntimeBindingMode::ExistingSessionOnly,
            )
            .map_err(gateway_runtime_error)?
        else {
            return Ok(None);
        };
        Ok(self.gateway_runtime.pid_for_world(&binding.runtime_id))
    }

    #[cfg(target_os = "linux")]
    pub fn forget_gateway_runtime_for_test(&self, req: &GatewayLifecycleRequestV1) -> Result<()> {
        let prepared = self.prepare_gateway_runtime_request(req)?;
        let Some(binding) = self
            .resolve_gateway_runtime_binding(
                prepared,
                GatewayRuntimeBindingMode::ExistingSessionOnly,
            )
            .map_err(gateway_runtime_error)?
        else {
            return Ok(());
        };
        self.gateway_runtime
            .forget_runtime_for_test(&binding.runtime_id);
        Ok(())
    }

    fn gateway_unavailable_response() -> GatewayLifecycleResponseV1 {
        #[cfg(target_os = "linux")]
        {
            gateway_unavailable_response_impl()
        }
        #[cfg(not(target_os = "linux"))]
        {
            GatewayLifecycleResponseV1 {
                status: GatewayStatusV1::Unavailable,
                client_wiring: None,
                identity_tuple: None,
                placement_posture: None,
            }
        }
    }

    fn attach_gateway_request_metadata(
        mut response: GatewayLifecycleResponseV1,
        req: &GatewayLifecycleRequestV1,
    ) -> Result<GatewayLifecycleResponseV1> {
        if response.identity_tuple.is_none() {
            response.identity_tuple = req.identity_tuple.clone();
        }
        if response.placement_posture.is_none() {
            response.placement_posture = req.placement_posture.clone();
        }
        response
            .validate_identity_contract()
            .map_err(|err| anyhow!("gateway_invalid_integration: {err}"))?;
        Ok(response)
    }
}

#[cfg(target_os = "linux")]
#[derive(Debug)]
struct PreparedGatewayRuntimeRequest {
    project_dir: PathBuf,
    world_spec: WorldSpec,
    selected_backend: String,
    integrated_auth: Option<transport_api_types::GatewayIntegratedAuthPayloadV1>,
}

#[cfg(target_os = "linux")]
#[derive(Debug, Clone)]
struct AuthoritativeMemberPlacementContext {
    #[allow(dead_code)]
    world: WorldHandle,
    binding: SharedWorldBindingSnapshot,
    placement_root: PathBuf,
    effective_cwd: PathBuf,
    cgroup_path: PathBuf,
}

#[cfg(target_os = "linux")]
impl AuthoritativeMemberPlacementContext {
    fn launch_placement(&self) -> LinuxWorldPlacementContext {
        LinuxWorldPlacementContext {
            working_dir: self.effective_cwd.clone(),
            cgroup_path: self.cgroup_path.clone(),
            require_cgroup_attach: true,
        }
    }
}

#[cfg(target_os = "linux")]
#[derive(Clone, Copy)]
enum GatewayRuntimeBindingMode {
    EnsureSession,
    ExistingSessionOnly,
}

#[cfg(target_os = "linux")]
struct ResolvedGatewayRuntimeBinding {
    runtime_id: String,
    start_context: GatewayRuntimeStartContext,
}

#[cfg(target_os = "linux")]
impl ResolvedGatewayRuntimeBinding {
    fn non_isolated(
        prepared: PreparedGatewayRuntimeRequest,
        binding: &'static crate::gateway_runtime::GatewayBackendBinding,
    ) -> Self {
        let runtime_id = non_isolated_gateway_runtime_id(
            &prepared.project_dir,
            &prepared.world_spec,
            binding.backend_id,
        );
        Self {
            runtime_id: runtime_id.clone(),
            start_context: GatewayRuntimeStartContext {
                world_id: runtime_id,
                project_dir: prepared.project_dir,
                cgroup_path: PathBuf::from("/nonexistent"),
                require_cgroup_attach: false,
                binding,
                ordinary_effect: None,
                integrated_auth: prepared.integrated_auth,
            },
        }
    }
}

#[cfg(target_os = "linux")]
fn non_isolated_gateway_runtime_id(
    project_dir: &Path,
    world_spec: &WorldSpec,
    backend_id: &str,
) -> String {
    let canonical_project_dir = std::fs::canonicalize(project_dir)
        .unwrap_or_else(|_| project_dir.to_path_buf())
        .to_string_lossy()
        .replace('\\', "/");
    let mut allowed_domains = world_spec.allowed_domains.clone();
    allowed_domains.sort();
    allowed_domains.dedup();

    let mut hasher = Sha256::new();
    hasher.update(b"project_dir\0");
    hasher.update(canonical_project_dir.as_bytes());
    hasher.update(b"\0isolate_network\0");
    hasher.update(if world_spec.isolate_network {
        b"1"
    } else {
        b"0"
    });
    hasher.update(b"\0always_isolate\0");
    hasher.update(if world_spec.always_isolate {
        b"1"
    } else {
        b"0"
    });
    hasher.update(b"\0allowed_domains\0");
    for domain in allowed_domains {
        hasher.update(domain.as_bytes());
        hasher.update(b"\n");
    }
    hasher.update(b"\0backend_id\0");
    hasher.update(backend_id.as_bytes());

    let digest = hasher.finalize();
    let mut encoded = String::with_capacity(16);
    for byte in digest.iter().take(8) {
        encoded.push_str(&format!("{byte:02x}"));
    }
    format!("gwrt_{encoded}")
}

#[cfg(target_os = "linux")]
fn isolated_gateway_runtime_id(world_id: &str, backend_id: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(b"world_id\0");
    hasher.update(world_id.as_bytes());
    hasher.update(b"\0backend_id\0");
    hasher.update(backend_id.as_bytes());

    let digest = hasher.finalize();
    let mut encoded = String::with_capacity(16);
    for byte in digest.iter().take(8) {
        encoded.push_str(&format!("{byte:02x}"));
    }
    format!("gwrt_{encoded}")
}

#[cfg(target_os = "linux")]
fn gateway_runtime_error(err: GatewayRuntimeFailure) -> anyhow::Error {
    anyhow!(err.to_string())
}

#[cfg(target_os = "linux")]
struct MemberRuntimeLaunchEnvContext<'a> {
    placement_root: &'a Path,
    isolation_full: bool,
    write_allowlist_prefixes: &'a [String],
    landlock_discover_paths: &'a [String],
    landlock_read_paths: &'a [String],
    landlock_write_paths: &'a [String],
    enforcement_plan_b64: Option<&'a str>,
}

#[cfg(target_os = "linux")]
fn build_member_runtime_launch_env(
    mut env_map: HashMap<String, String>,
    ctx: MemberRuntimeLaunchEnvContext<'_>,
) -> HashMap<String, String> {
    env_map.insert(
        WORLD_PROJECT_DIR_OVERRIDE_ENV.to_string(),
        ctx.placement_root.display().to_string(),
    );
    env_map.insert(
        WORLD_FS_ISOLATION_ENV.to_string(),
        if ctx.isolation_full {
            "full".to_string()
        } else {
            "workspace".to_string()
        },
    );
    if ctx.isolation_full && !ctx.write_allowlist_prefixes.is_empty() {
        env_map.insert(
            WORLD_FS_WRITE_ALLOWLIST_ENV.to_string(),
            ctx.write_allowlist_prefixes.join("\n"),
        );
    }

    let landlock_supported = world::landlock::detect_support().supported;
    if ctx.isolation_full {
        apply_full_isolation_helper_env(
            &mut env_map,
            landlock_supported,
            ctx.landlock_discover_paths,
            ctx.landlock_read_paths,
            ctx.landlock_write_paths,
            ctx.enforcement_plan_b64,
        );
    }

    env_map
}

#[cfg(target_os = "linux")]
pub(crate) fn build_member_runtime_turn_env(
    mut env_map: HashMap<String, String>,
    policy_snapshot: &transport_api_types::PolicySnapshotV3,
    placement_root: &Path,
    process_working_dir: &Path,
) -> Result<HashMap<String, String>> {
    const TURN_POLICY_ENV_KEYS: &[&str] = &[
        WORLD_FS_MODE_ENV,
        WORLD_FS_ISOLATION_ENV,
        WORLD_FS_WRITE_ALLOWLIST_ENV,
        WORLD_FS_LANDLOCK_DISCOVER_ALLOWLIST_ENV,
        WORLD_FS_LANDLOCK_READ_ALLOWLIST_ENV,
        WORLD_FS_LANDLOCK_WRITE_ALLOWLIST_ENV,
        enforcement_plan::WORLD_FS_ENFORCEMENT_PLAN_B64_ENV,
        LANDLOCK_HELPER_SRC_ENV,
        "SUBSTRATE_LANDLOCK_HELPER_PATH",
        "SUBSTRATE_MOUNT_MERGED_DIR",
        "SUBSTRATE_MOUNT_PROJECT_DIR",
        "SUBSTRATE_MOUNT_CWD",
        "SUBSTRATE_MOUNT_FS_MODE",
        "SUBSTRATE_INNER_CMD",
        "SUBSTRATE_INNER_LOGIN_SHELL",
    ];

    for key in TURN_POLICY_ENV_KEYS {
        env_map.remove(*key);
    }

    let (_, policy_inputs) =
        resolve_policy_inputs(policy_snapshot, None, process_working_dir, placement_root)?;
    let PolicyInputs {
        fs_mode,
        isolation_full,
        isolate_network: _,
        allowed_domains: _,
        mut write_allowlist_prefixes,
        landlock_discover_paths,
        landlock_read_paths,
        mut landlock_write_paths,
        enforcement_plan_b64,
    } = policy_inputs;

    if matches!(fs_mode, WorldFsMode::ReadOnly) {
        // PolicySnapshotV3 retains a canonical write allowlist even when writes are disabled.
        // It is dormant authority in that state and must not become an active per-turn rule.
        write_allowlist_prefixes.clear();
        landlock_write_paths.clear();
    }

    if isolation_full
        && landlock_discover_paths.is_empty()
        && landlock_read_paths.is_empty()
        && landlock_write_paths.is_empty()
    {
        anyhow::bail!(
            "authenticated retained-turn full-isolation policy has no enforceable Landlock paths"
        );
    }
    let landlock_support = world::landlock::detect_support();
    if isolation_full && !landlock_support.supported {
        anyhow::bail!(
            "authenticated retained-turn full-isolation policy requires Landlock support: {}",
            landlock_support
                .reason
                .as_deref()
                .unwrap_or("support probe returned no reason")
        );
    }

    env_map = build_member_runtime_launch_env(
        env_map,
        MemberRuntimeLaunchEnvContext {
            placement_root,
            isolation_full,
            write_allowlist_prefixes: &write_allowlist_prefixes,
            landlock_discover_paths: &landlock_discover_paths,
            landlock_read_paths: &landlock_read_paths,
            landlock_write_paths: &landlock_write_paths,
            enforcement_plan_b64: enforcement_plan_b64.as_deref(),
        },
    );
    let helper_src = resolve_landlock_helper_src().ok_or_else(|| {
        anyhow!(
            "authenticated retained-turn policy could not resolve the source-built Landlock helper"
        )
    })?;
    env_map.insert(LANDLOCK_HELPER_SRC_ENV.to_string(), helper_src);
    env_map.insert(WORLD_FS_MODE_ENV.to_string(), fs_mode.as_str().to_string());
    env_map.insert(
        "SUBSTRATE_MOUNT_PROJECT_DIR".to_string(),
        placement_root.display().to_string(),
    );
    env_map.insert(
        "SUBSTRATE_MOUNT_CWD".to_string(),
        process_working_dir.display().to_string(),
    );
    env_map.insert(
        "SUBSTRATE_MOUNT_FS_MODE".to_string(),
        fs_mode.as_str().to_string(),
    );
    env_map.insert("SUBSTRATE_INNER_LOGIN_SHELL".to_string(), "0".to_string());

    Ok(env_map)
}

#[cfg(target_os = "linux")]
fn resolve_member_runtime_effective_cwd(
    project_dir: &Path,
    placement_root: &Path,
    requested_cwd: &Path,
) -> Result<PathBuf> {
    let requested_cwd =
        std::fs::canonicalize(requested_cwd).unwrap_or_else(|_| requested_cwd.to_path_buf());
    let project_dir =
        std::fs::canonicalize(project_dir).unwrap_or_else(|_| project_dir.to_path_buf());

    if requested_cwd == project_dir {
        return Ok(placement_root.to_path_buf());
    }

    let relative = requested_cwd.strip_prefix(&project_dir).map_err(|_| {
        anyhow!(
            "member runtime cwd {} is outside the authoritative project root {}",
            requested_cwd.display(),
            project_dir.display()
        )
    })?;
    let effective_cwd = placement_root.join(relative);
    if !effective_cwd.is_dir() {
        anyhow::bail!(
            "member runtime cwd {} is missing from authoritative placement root {}",
            requested_cwd.display(),
            placement_root.display()
        );
    }

    Ok(effective_cwd)
}

#[cfg(test)]
mod pending_diff_id_tests {
    use super::WorldService;
    use std::path::PathBuf;
    use substrate_common::FsDiff;
    use transport_api_types::PendingDiffBucketV1;

    #[test]
    fn diff_id_is_stable_across_writes_vs_mods_reclassification() {
        let s1 = PendingDiffBucketV1 {
            writes: vec!["flipcheck.md".to_string()],
            mods: vec![],
            deletes: vec![],
        };
        let s2 = PendingDiffBucketV1 {
            writes: vec![],
            mods: vec!["flipcheck.md".to_string()],
            deletes: vec![],
        };
        assert_eq!(
            WorldService::pending_diff_id_for_snapshot(&s1),
            WorldService::pending_diff_id_for_snapshot(&s2),
            "diff_id must not change when a path moves between writes and mods"
        );
    }

    #[test]
    fn diff_id_changes_when_updates_or_deletes_change() {
        let base = PendingDiffBucketV1 {
            writes: vec!["a.txt".to_string()],
            mods: vec![],
            deletes: vec![],
        };
        let with_more_updates = PendingDiffBucketV1 {
            writes: vec!["a.txt".to_string(), "b.txt".to_string()],
            mods: vec![],
            deletes: vec![],
        };
        let with_delete = PendingDiffBucketV1 {
            writes: vec!["a.txt".to_string()],
            mods: vec![],
            deletes: vec!["gone.txt".to_string()],
        };
        assert_ne!(
            WorldService::pending_diff_id_for_snapshot(&base),
            WorldService::pending_diff_id_for_snapshot(&with_more_updates),
            "diff_id must change when the update set changes"
        );
        assert_ne!(
            WorldService::pending_diff_id_for_snapshot(&base),
            WorldService::pending_diff_id_for_snapshot(&with_delete),
            "diff_id must change when the delete set changes"
        );
    }

    #[test]
    fn diff_id_for_diff_is_stable_across_fs_diff_writes_mods_flip() {
        let d1 = FsDiff {
            writes: vec![PathBuf::from("flipcheck.md")],
            ..Default::default()
        };
        let d2 = FsDiff {
            mods: vec![PathBuf::from("flipcheck.md")],
            ..Default::default()
        };
        assert_eq!(
            WorldService::pending_diff_id_for_diff(&d1),
            WorldService::pending_diff_id_for_diff(&d2),
            "diff_id must not change when FsDiff reclassifies a path between writes and mods"
        );
    }
}

#[cfg(all(target_os = "linux", test))]
mod gateway_runtime_binding_tests {
    use super::*;
    use serde_json::json;
    use tempfile::TempDir;
    use transport_api_types::{
        GatewayApiEnvIntegratedAuthV1, GatewayCliCodexIntegratedAuthV1,
        GatewayIntegratedAuthPayloadV1, PolicySnapshotV3, PolicySnapshotWorldFsFailClosedV3,
        PolicySnapshotWorldFsV3, PolicySnapshotWorldFsWriteV3, WorldNetworkRoutingV1,
    };

    fn minimal_policy_snapshot() -> PolicySnapshotV3 {
        PolicySnapshotV3 {
            schema_version: 3,
            net_allowed: Vec::new(),
            world_fs: PolicySnapshotWorldFsV3 {
                host_visible: true,
                fail_closed: PolicySnapshotWorldFsFailClosedV3 { routing: false },
                deny_enforcement: None,
                caged_required: false,
                discover: None,
                read: None,
                write: PolicySnapshotWorldFsWriteV3::default(),
            },
        }
    }

    fn gateway_request(cwd: &Path) -> GatewayLifecycleRequestV1 {
        GatewayLifecycleRequestV1 {
            schema_version: 1,
            ordinary_selection: None,
            profile: None,
            cwd: Some(cwd.display().to_string()),
            env: None,
            agent_id: "gateway-binding-test".to_string(),
            policy_snapshot: minimal_policy_snapshot(),
            world_network: Some(WorldNetworkRoutingV1 {
                isolate_network: false,
                allowed_domains: Vec::new(),
            }),
            integrated_auth: Some(GatewayIntegratedAuthPayloadV1 {
                backend_id: "cli:codex".to_string(),
                cli_codex: Some(GatewayCliCodexIntegratedAuthV1 {
                    account_id: Some("acct_test".to_string()),
                    access_token: "header.payload.signature".to_string(),
                }),
                api_env: None,
            }),
            identity_tuple: None,
            placement_posture: None,
        }
    }

    fn gateway_request_with_backend(cwd: &Path, backend_id: &str) -> GatewayLifecycleRequestV1 {
        let mut request = gateway_request(cwd);
        let mut env = HashMap::new();
        env.insert(
            "SUBSTRATE_LLM_DEFAULT_BACKEND".to_string(),
            backend_id.to_string(),
        );
        request.env = Some(env);
        request
    }

    #[test]
    fn request_preparation_rejects_integrated_auth_for_different_backend() {
        let temp_dir = TempDir::new().unwrap();
        let service = WorldService::new().expect("service");
        let mut request = gateway_request_with_backend(temp_dir.path(), "cli:codex");
        let mut env = HashMap::new();
        env.insert("OPENAI_API_KEY".to_string(), "sk-test".to_string());
        request.integrated_auth = Some(GatewayIntegratedAuthPayloadV1 {
            backend_id: "api:openai".to_string(),
            cli_codex: None,
            api_env: Some(GatewayApiEnvIntegratedAuthV1 { env }),
        });

        let err = service
            .prepare_gateway_runtime_request(&request)
            .expect_err("mismatched auth payload should fail");
        assert!(
            err.to_string().contains("does not match selected backend"),
            "unexpected error: {err:#}"
        );
    }

    #[test]
    fn non_isolated_runtime_id_is_stable_for_equivalent_specs() {
        let project_dir = PathBuf::from("/tmp/project");
        let mut world_spec = build_world_spec(
            project_dir.clone(),
            false,
            WorldFsMode::Writable,
            false,
            vec!["b.example.com".to_string(), "a.example.com".to_string()],
            None,
        );
        let first = non_isolated_gateway_runtime_id(&project_dir, &world_spec, "cli:codex");
        world_spec.allowed_domains.reverse();
        let second = non_isolated_gateway_runtime_id(&project_dir, &world_spec, "cli:codex");
        assert_eq!(first, second);
    }

    #[test]
    fn non_isolated_runtime_id_changes_with_binding_inputs() {
        let project_dir = PathBuf::from("/tmp/project");
        let base = build_world_spec(
            project_dir.clone(),
            false,
            WorldFsMode::Writable,
            false,
            vec!["example.com".to_string()],
            None,
        );
        let changed_project =
            non_isolated_gateway_runtime_id(Path::new("/tmp/other"), &base, "cli:codex");
        let changed_always_isolate = non_isolated_gateway_runtime_id(
            &project_dir,
            &build_world_spec(
                project_dir.clone(),
                true,
                WorldFsMode::Writable,
                false,
                vec!["example.com".to_string()],
                None,
            ),
            "cli:codex",
        );
        let changed_domains = non_isolated_gateway_runtime_id(
            &project_dir,
            &build_world_spec(
                project_dir.clone(),
                false,
                WorldFsMode::Writable,
                false,
                vec!["other.example.com".to_string()],
                None,
            ),
            "cli:codex",
        );
        let changed_backend = non_isolated_gateway_runtime_id(&project_dir, &base, "api:openai");
        let base_id = non_isolated_gateway_runtime_id(&project_dir, &base, "cli:codex");
        assert_ne!(base_id, changed_project);
        assert_ne!(base_id, changed_always_isolate);
        assert_ne!(base_id, changed_domains);
        assert_ne!(base_id, changed_backend);
    }

    #[test]
    fn request_preparation_accepts_valid_cli_codex() {
        let temp_dir = TempDir::new().unwrap();
        let service = WorldService::new().expect("service");

        service
            .prepare_gateway_runtime_request(&gateway_request(temp_dir.path()))
            .expect("valid cli:codex payload should prepare");
    }

    #[test]
    fn request_preparation_accepts_realized_cli_codex_backend() {
        let temp_dir = TempDir::new().unwrap();
        let service = WorldService::new().expect("service");
        let mut request = gateway_request(temp_dir.path());
        request.env = Some(HashMap::from([(
            "SUBSTRATE_LLM_DEFAULT_BACKEND".to_string(),
            "cli:codex-host".to_string(),
        )]));
        request.integrated_auth = Some(GatewayIntegratedAuthPayloadV1 {
            backend_id: "cli:codex-host".to_string(),
            cli_codex: Some(GatewayCliCodexIntegratedAuthV1 {
                account_id: Some("acct_test".to_string()),
                access_token: "header.payload.signature".to_string(),
            }),
            api_env: None,
        });

        service
            .prepare_gateway_runtime_request(&request)
            .expect("valid cli:codex-host payload should prepare");
    }

    #[test]
    fn non_isolated_binding_uses_synthetic_runtime_key() {
        let temp_dir = TempDir::new().unwrap();
        let service = WorldService::new().expect("service");
        let prepared = service
            .prepare_gateway_runtime_request(&gateway_request(temp_dir.path()))
            .expect("prepared request");
        let binding = service
            .resolve_gateway_runtime_binding(
                prepared,
                GatewayRuntimeBindingMode::ExistingSessionOnly,
            )
            .expect("binding resolution")
            .expect("non-isolated binding");

        assert!(binding.runtime_id.starts_with("gwrt_"));
        assert_eq!(binding.start_context.world_id, binding.runtime_id);
        assert!(!binding.start_context.require_cgroup_attach);
        assert_eq!(
            binding.start_context.cgroup_path,
            PathBuf::from("/nonexistent")
        );
    }

    #[test]
    fn unbound_api_backend_returns_none_without_invalid_integration() {
        let temp_dir = TempDir::new().unwrap();
        let service = WorldService::new().expect("service");
        let mut request = gateway_request_with_backend(temp_dir.path(), "api:anthropic");
        let mut env = HashMap::new();
        env.insert("OPENAI_API_KEY".to_string(), "sk-test".to_string());
        request.integrated_auth = Some(GatewayIntegratedAuthPayloadV1 {
            backend_id: "api:anthropic".to_string(),
            cli_codex: None,
            api_env: Some(GatewayApiEnvIntegratedAuthV1 { env }),
        });
        let prepared = service
            .prepare_gateway_runtime_request(&request)
            .expect("prepared request");
        let binding = service
            .resolve_gateway_runtime_binding(
                prepared,
                GatewayRuntimeBindingMode::ExistingSessionOnly,
            )
            .expect("binding resolution");

        assert!(
            binding.is_none(),
            "unbound backends should not map to a runtime"
        );
    }

    #[test]
    fn isolated_binding_does_not_synthesize_runtime_without_session() {
        let temp_dir = TempDir::new().unwrap();
        let service = WorldService::new().expect("service");
        let mut request = gateway_request(temp_dir.path());
        request.world_network = Some(WorldNetworkRoutingV1 {
            isolate_network: true,
            allowed_domains: Vec::new(),
        });
        let prepared = service
            .prepare_gateway_runtime_request(&request)
            .expect("prepared request");
        let binding = service
            .resolve_gateway_runtime_binding(
                prepared,
                GatewayRuntimeBindingMode::ExistingSessionOnly,
            )
            .expect("binding resolution");

        assert!(binding.is_none());
    }

    #[test]
    fn request_preparation_rejects_unknown_lifecycle_request_fields() {
        let err = serde_json::from_value::<GatewayLifecycleRequestV1>(json!({
            "profile": null,
            "cwd": "/tmp",
            "env": null,
            "agent_id": "gateway-binding-test",
            "policy_snapshot": {
                "schema_version": 3,
                "net_allowed": [],
                "world_fs": {
                    "host_visible": true,
                    "fail_closed": { "routing": false },
                    "caged_required": false,
                    "write": { "enabled": true, "allow_list": ["."], "deny_list": [] }
                }
            },
            "world_network": {
                "isolate_network": false,
                "allowed_domains": []
            },
            "integrated_auth": {
                "backend_id": "cli:codex",
                "cli_codex": {
                    "access_token": "header.payload.signature"
                }
            },
            "unexpected": true
        }))
        .expect_err("unknown request field should fail");

        assert!(err.to_string().contains("unknown field"));
    }

    #[test]
    fn request_preparation_rejects_multi_facet_payloads() {
        let temp_dir = TempDir::new().unwrap();
        let service = WorldService::new().expect("service");
        let mut request = gateway_request(temp_dir.path());
        let mut env = HashMap::new();
        env.insert("OPENAI_API_KEY".to_string(), "sk-test".to_string());
        request.integrated_auth = Some(GatewayIntegratedAuthPayloadV1 {
            backend_id: "cli:codex".to_string(),
            cli_codex: Some(GatewayCliCodexIntegratedAuthV1 {
                account_id: Some("acct_test".to_string()),
                access_token: "header.payload.signature".to_string(),
            }),
            api_env: Some(GatewayApiEnvIntegratedAuthV1 { env }),
        });

        let err = service
            .prepare_gateway_runtime_request(&request)
            .expect_err("multi-facet payload should fail");
        assert!(err.to_string().contains("exactly one auth facet"));
    }

    #[test]
    fn request_preparation_rejects_blank_required_values() {
        let temp_dir = TempDir::new().unwrap();
        let service = WorldService::new().expect("service");
        let mut request = gateway_request(temp_dir.path());
        request.integrated_auth = Some(GatewayIntegratedAuthPayloadV1 {
            backend_id: "cli:codex".to_string(),
            cli_codex: Some(GatewayCliCodexIntegratedAuthV1 {
                account_id: Some("acct_test".to_string()),
                access_token: "   ".to_string(),
            }),
            api_env: None,
        });

        let err = service
            .prepare_gateway_runtime_request(&request)
            .expect_err("blank required value should fail");
        assert!(err.to_string().contains("empty cli_codex.access_token"));
    }
}

#[cfg(target_os = "linux")]
pub(crate) struct RuntimeEventStreamProducer {
    stream_id: String,
    next_frame_sequence: u64,
    next_event_sequence: u64,
    started: bool,
    closed: bool,
}

#[cfg(target_os = "linux")]
impl RuntimeEventStreamProducer {
    pub(crate) fn new() -> Self {
        Self {
            stream_id: format!("rts_{}", uuid::Uuid::now_v7()),
            next_frame_sequence: 1,
            next_event_sequence: 1,
            started: false,
            closed: false,
        }
    }

    pub(crate) fn with_stream_id(stream_id: String) -> Result<Self> {
        if !stream_id.starts_with("rts_") || stream_id.len() <= 4 || stream_id.trim() != stream_id {
            anyhow::bail!("reserved runtime stream identity is invalid");
        }
        Ok(Self {
            stream_id,
            next_frame_sequence: 1,
            next_event_sequence: 1,
            started: false,
            closed: false,
        })
    }

    pub(crate) fn start(&mut self, span_id: String) -> Result<ExecuteStreamFrame> {
        if self.started {
            anyhow::bail!("runtime event stream Start was already emitted");
        }
        let frame_identity = self.next_frame_identity(false)?;
        self.started = true;
        Ok(ExecuteStreamFrame::Start {
            frame_identity,
            span_id,
        })
    }

    pub(crate) fn stdout(&mut self, chunk_b64: String) -> Result<ExecuteStreamFrame> {
        Ok(ExecuteStreamFrame::Stdout {
            frame_identity: self.next_frame_identity(true)?,
            chunk_b64,
        })
    }

    pub(crate) fn stderr(&mut self, chunk_b64: String) -> Result<ExecuteStreamFrame> {
        Ok(ExecuteStreamFrame::Stderr {
            frame_identity: self.next_frame_identity(true)?,
            chunk_b64,
        })
    }

    pub(crate) fn event(&mut self, mut event: AgentEvent) -> Result<ExecuteStreamFrame> {
        let (frame_identity, event_identity) = self.next_semantic_identities()?;
        event.event_identity = Some(event_identity);
        Ok(ExecuteStreamFrame::Event {
            frame_identity,
            event,
        })
    }

    pub(crate) fn exit(
        &mut self,
        exit: i32,
        span_id: String,
        scopes_used: Vec<String>,
        fs_diff: Option<substrate_common::FsDiff>,
        process_telemetry: ProcessTelemetry,
    ) -> Result<ExecuteStreamFrame> {
        let (frame_identity, event_identity) = self.next_semantic_identities()?;
        let terminal_identity = RuntimeTerminalIdentityV1::from(&event_identity);
        self.closed = true;
        Ok(ExecuteStreamFrame::Exit {
            frame_identity,
            event_identity,
            terminal_identity,
            exit,
            span_id,
            scopes_used,
            fs_diff,
            process_telemetry,
        })
    }

    pub(crate) fn transport_error(&mut self, message: String) -> Result<ExecuteStreamFrame> {
        let frame_identity = self.next_frame_identity(true)?;
        self.closed = true;
        Ok(ExecuteStreamFrame::Error {
            frame_identity,
            message,
        })
    }

    fn next_frame_identity(&mut self, require_started: bool) -> Result<RuntimeFrameIdentityV1> {
        self.ensure_open(require_started)?;
        let frame_sequence = self.next_frame_sequence;
        let next_frame_sequence = frame_sequence
            .checked_add(1)
            .ok_or_else(|| anyhow!("runtime frame sequence exhausted"))?;
        self.next_frame_sequence = next_frame_sequence;
        Ok(RuntimeFrameIdentityV1 {
            schema_version: RUNTIME_FRAME_IDENTITY_SCHEMA_VERSION_V1,
            stream_id: self.stream_id.clone(),
            frame_sequence,
        })
    }

    fn next_semantic_identities(
        &mut self,
    ) -> Result<(RuntimeFrameIdentityV1, RuntimeEventIdentityV1)> {
        self.ensure_open(true)?;
        let frame_sequence = self.next_frame_sequence;
        let event_sequence = self.next_event_sequence;
        let next_frame_sequence = frame_sequence
            .checked_add(1)
            .ok_or_else(|| anyhow!("runtime frame sequence exhausted"))?;
        let next_event_sequence = event_sequence
            .checked_add(1)
            .ok_or_else(|| anyhow!("runtime event sequence exhausted"))?;
        self.next_frame_sequence = next_frame_sequence;
        self.next_event_sequence = next_event_sequence;
        Ok((
            RuntimeFrameIdentityV1 {
                schema_version: RUNTIME_FRAME_IDENTITY_SCHEMA_VERSION_V1,
                stream_id: self.stream_id.clone(),
                frame_sequence,
            },
            RuntimeEventIdentityV1 {
                event_id: format!("evt_{}", uuid::Uuid::now_v7()),
                event_sequence,
            },
        ))
    }

    fn ensure_open(&self, require_started: bool) -> Result<()> {
        if self.closed {
            anyhow::bail!("runtime event stream is closed");
        }
        if require_started && !self.started {
            anyhow::bail!("runtime event stream Start has not been emitted");
        }
        Ok(())
    }
}

#[cfg(target_os = "linux")]
struct StreamingSink {
    tx: tokio::sync::mpsc::UnboundedSender<ExecuteStreamFrame>,
    producer: Arc<Mutex<RuntimeEventStreamProducer>>,
    replay_publisher: Option<RuntimeReplayPublisher>,
}

#[cfg(target_os = "linux")]
impl StreamingSink {
    fn new(
        tx: tokio::sync::mpsc::UnboundedSender<ExecuteStreamFrame>,
        producer: Arc<Mutex<RuntimeEventStreamProducer>>,
        replay_publisher: Option<RuntimeReplayPublisher>,
    ) -> Self {
        Self {
            tx,
            producer,
            replay_publisher,
        }
    }
}

#[cfg(target_os = "linux")]
impl StreamSink for StreamingSink {
    fn write(&self, kind: StreamKind, chunk: &[u8]) {
        if chunk.is_empty() {
            return;
        }
        let encoded = BASE64.encode(chunk);
        // Assignment and enqueue are one producer operation. Releasing this lock between them
        // would let concurrent stdout/stderr readers enqueue later identities first.
        let mut producer = self
            .producer
            .lock()
            .expect("runtime event stream producer lock poisoned");
        let frame = match kind {
            StreamKind::Stdout => producer.stdout(encoded),
            StreamKind::Stderr => producer.stderr(encoded),
        };
        match frame {
            Ok(frame) => {
                if let Err(err) =
                    publish_replayable_frame(self.replay_publisher.as_ref(), &self.tx, frame)
                {
                    tracing::error!(error = %err, "failed to publish replayable runtime output frame");
                }
            }
            Err(err) => {
                tracing::error!(error = %err, "failed to identify runtime output frame");
            }
        }
    }
}

fn should_always_isolate(req: &ExecuteRequest) -> bool {
    should_always_isolate_for_profile(req.profile.as_deref())
}

fn should_always_isolate_for_profile(profile: Option<&str>) -> bool {
    // `world deps provision` is explicitly intended to mutate guest system packages (apt/dpkg),
    // which is incompatible with the default cage's write restrictions. The internal
    // `world-deps-probe` profile shares that relaxed posture for read-only dpkg-query probes so
    // fail-early validation still works on runners where unprivileged user namespaces are blocked.
    !matches!(profile, Some("world-deps-provision" | "world-deps-probe"))
}

fn wrap_command_for_profile(profile: Option<&str>, cwd: &Path, cmd: &str) -> String {
    match profile {
        Some("world-deps-provision") => build_systemd_provision_wrapper(cwd, cmd),
        _ => cmd.to_string(),
    }
}

fn build_systemd_provision_wrapper(cwd: &Path, cmd: &str) -> String {
    let working_directory = cwd.display().to_string();
    [
        "systemd-run".to_string(),
        "--quiet".to_string(),
        "--wait".to_string(),
        "--pipe".to_string(),
        "--collect".to_string(),
        "--service-type=exec".to_string(),
        format!("--working-directory={}", sh_quote(&working_directory)),
        format!(
            "--description={}",
            sh_quote("Substrate world-deps provisioning")
        ),
        "/bin/sh".to_string(),
        "-lc".to_string(),
        sh_quote(cmd),
    ]
    .join(" ")
}

fn sh_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\"'\"'"))
}

#[derive(Debug, thiserror::Error)]
#[error("{message}")]
pub(crate) struct BadRequestError {
    message: String,
}

impl BadRequestError {
    pub(crate) fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }

    pub(crate) fn message(&self) -> &str {
        &self.message
    }
}

#[derive(Clone, Debug)]
struct PolicyInputs {
    fs_mode: WorldFsMode,
    isolation_full: bool,
    isolate_network: bool,
    allowed_domains: Vec<String>,
    write_allowlist_prefixes: Vec<String>,
    landlock_discover_paths: Vec<String>,
    landlock_read_paths: Vec<String>,
    landlock_write_paths: Vec<String>,
    enforcement_plan_b64: Option<String>,
}

fn resolve_policy_inputs(
    policy_snapshot: &transport_api_types::PolicySnapshotV3,
    world_network: Option<&WorldNetworkRoutingV1>,
    _cwd: &Path,
    project_dir: &Path,
) -> Result<(transport_api_types::PolicyResolutionModeV1, PolicyInputs)> {
    use transport_api_types::PolicyResolutionModeV1;

    let resolved =
        resolve_snapshot_routing(policy_snapshot, world_network).map_err(BadRequestError::new)?;
    let snapshot = resolved.snapshot;

    let isolation_full = resolved.isolation_full;
    let fs_mode = resolved.fs_mode;

    let enforcement_plan_b64 = enforcement_plan::maybe_encode_from_snapshot(&snapshot)
        .map_err(|err| BadRequestError::new(err.to_string()))?;

    let (
        write_allowlist_prefixes,
        landlock_discover_paths,
        landlock_read_paths,
        landlock_write_paths,
    ) = if isolation_full {
        let read_allowlist = snapshot
            .world_fs
            .read
            .as_ref()
            .map(|d| d.allow_list.as_slice())
            .unwrap_or(&[]);
        let discover_allowlist = snapshot
            .world_fs
            .discover
            .as_ref()
            .map(|d| d.allow_list.as_slice())
            .unwrap_or(read_allowlist);
        let write_allowlist = snapshot.world_fs.write.allow_list.as_slice();

        (
            resolve_project_write_allowlist_prefixes(project_dir, write_allowlist),
            resolve_landlock_allowlist_paths(project_dir, discover_allowlist),
            resolve_landlock_allowlist_paths(project_dir, read_allowlist),
            resolve_landlock_allowlist_paths(project_dir, write_allowlist),
        )
    } else {
        (Vec::new(), Vec::new(), Vec::new(), Vec::new())
    };

    Ok((
        PolicyResolutionModeV1::SnapshotV3,
        PolicyInputs {
            fs_mode,
            isolation_full,
            isolate_network: resolved.world_network.isolate_network,
            allowed_domains: resolved.world_network.allowed_domains,
            write_allowlist_prefixes,
            landlock_discover_paths,
            landlock_read_paths,
            landlock_write_paths,
            enforcement_plan_b64,
        },
    ))
}

pub(crate) fn build_world_spec(
    project_dir: PathBuf,
    always_isolate: bool,
    fs_mode: WorldFsMode,
    isolate_network: bool,
    allowed_domains: Vec<String>,
    shared_world: Option<&SharedWorldOwnerSpec>,
) -> WorldSpec {
    WorldSpec {
        reuse_session: true,
        reuse_mode: shared_world
            .cloned()
            .map(WorldReuseMode::SharedOrchestration)
            .unwrap_or(WorldReuseMode::GenericCompatible),
        isolate_network,
        limits: world_api::ResourceLimits::default(),
        enable_preload: false,
        allowed_domains,
        project_dir,
        always_isolate,
        fs_mode,
        backend_policy: None,
    }
}

#[cfg(target_os = "linux")]
fn requested_shared_world_owner_spec(req: &ExecuteRequest) -> Option<SharedWorldOwnerSpec> {
    req.shared_world.clone().or_else(|| {
        req.member_dispatch
            .as_ref()
            .map(|dispatch| SharedWorldOwnerSpec {
                orchestration_session_id: dispatch.common().orchestration_session_id.to_string(),
                action: world_api::SharedWorldOwnerAction::AttachOrCreate,
            })
    })
}

#[cfg(target_os = "linux")]
fn canonical_policy_snapshot_sha256(
    policy_snapshot: &transport_api_types::PolicySnapshotV3,
) -> Result<String> {
    let canonical = policy_snapshot
        .canonicalize()
        .map_err(|error| BadRequestError::new(format!("invalid policy snapshot: {error}")))?;
    let bytes = serde_json::to_vec(&canonical)
        .map_err(|_| BadRequestError::new("failed to canonicalize policy snapshot".to_string()))?;
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    Ok(format!("{:x}", hasher.finalize()))
}

#[cfg(target_os = "linux")]
fn exact_bound_world_ownership_adoption(
    dispatch: &transport_api_types::MemberDispatchRequest,
    policy_snapshot: &transport_api_types::PolicySnapshotV3,
    target_spec: &WorldSpec,
) -> Result<Option<world::ExactBoundWorldOwnershipAdoptionV1>> {
    let dispatch = dispatch.common();
    let managed_identity =
        dispatch.participant_id.starts_with("rwp_") || dispatch.run_id.starts_with("rwr_");
    let Some(proof) = dispatch.retained_worker_launch_authority.as_ref() else {
        if managed_identity {
            return Err(BadRequestError::new(
                "authority-managed retained member dispatch requires retained_worker_launch_authority"
                    .to_string(),
            )
            .into());
        }
        return Ok(None);
    };

    proof.validate().map_err(|error| {
        BadRequestError::new(format!("invalid retained_worker_launch_authority: {error}"))
    })?;
    if proof.orchestration_session_id != dispatch.orchestration_session_id
        || proof.retained_participant_id != dispatch.participant_id
        || proof.caller_participant_id != dispatch.orchestrator_participant_id
        || proof.backend_id != dispatch.backend_id
        || proof.protocol != dispatch.protocol
        || proof.bootstrap_run_id != dispatch.run_id
        || proof.world_binding.world_id != dispatch.world_id
        || proof.world_binding.world_generation != dispatch.world_generation
        || dispatch.parent_participant_id.is_some()
        || dispatch.resumed_from_participant_id.is_some()
    {
        return Err(BadRequestError::new(
            "retained_worker_launch_authority conflicts with member dispatch or exact world binding"
                .to_string(),
        )
        .into());
    }

    let owner = target_spec.reuse_mode.shared_owner().ok_or_else(|| {
        BadRequestError::new(
            "authority-managed retained member dispatch requires exact shared owner target"
                .to_string(),
        )
    })?;
    if owner.orchestration_session_id != dispatch.orchestration_session_id
        || !matches!(
            owner.action,
            world_api::SharedWorldOwnerAction::AttachOrCreate
        )
    {
        return Err(BadRequestError::new(
            "authority-managed retained member dispatch conflicts with shared owner target"
                .to_string(),
        )
        .into());
    }

    let policy_snapshot_hash = canonical_policy_snapshot_sha256(policy_snapshot)?;
    if proof.current_policy_revision != policy_snapshot_hash {
        return Err(BadRequestError::new(
            "retained_worker_launch_authority conflicts with exact policy snapshot".to_string(),
        )
        .into());
    }

    Ok(Some(world::ExactBoundWorldOwnershipAdoptionV1 {
        orchestration_session_id: dispatch.orchestration_session_id.to_string(),
        world_id: dispatch.world_id.to_string(),
        world_generation: dispatch.world_generation,
        participant_id: dispatch.participant_id.to_string(),
        policy_ref_id: proof.current_policy_ref_id.clone(),
        policy_revision: proof.current_policy_revision.clone(),
        policy_snapshot_hash,
        target_spec: target_spec.clone(),
        adoption_correlation_id: proof.transport_claim_id.clone(),
    }))
}

#[cfg(target_os = "linux")]
struct DurableExecuteStreamWorldSelection {
    world: WorldHandle,
}

#[cfg(target_os = "linux")]
impl DurableExecuteStreamWorldSelection {
    fn into_world(self) -> WorldHandle {
        self.world
    }
}

#[cfg(target_os = "linux")]
fn select_execute_stream_world<Adopt, Ensure>(
    adoption: Option<&world::ExactBoundWorldOwnershipAdoptionV1>,
    adopt_exact: Adopt,
    ensure_compatibility: Ensure,
) -> Result<DurableExecuteStreamWorldSelection>
where
    Adopt: FnOnce(&world::ExactBoundWorldOwnershipAdoptionV1) -> Result<WorldHandle>,
    Ensure: FnOnce() -> Result<WorldHandle>,
{
    let world = match adoption {
        Some(adoption) => adopt_exact(adoption)?,
        None => ensure_compatibility()?,
    };
    Ok(DurableExecuteStreamWorldSelection { world })
}

fn convert_member_dispatch_request(
    dispatch: &transport_api_types::MemberDispatchRequest,
) -> Result<BackendMemberDispatchRequestV1> {
    let dispatch = dispatch.as_v1().ok_or_else(|| {
        BadRequestError::new("member_dispatch V2 launch is not implemented".to_string())
    })?;
    Ok(BackendMemberDispatchRequestV1 {
        schema_version: dispatch.schema_version,
        orchestration_session_id: dispatch.orchestration_session_id.clone(),
        participant_id: dispatch.participant_id.clone(),
        orchestrator_participant_id: dispatch.orchestrator_participant_id.clone(),
        parent_participant_id: dispatch.parent_participant_id.clone(),
        resumed_from_participant_id: dispatch.resumed_from_participant_id.clone(),
        backend_id: dispatch.backend_id.clone(),
        protocol: dispatch.protocol.clone(),
        run_id: dispatch.run_id.clone(),
        world_id: dispatch.world_id.clone(),
        world_generation: dispatch.world_generation,
        initial_prompt: dispatch.initial_prompt.clone(),
        resolved_runtime: BackendResolvedMemberRuntimeDescriptorV1 {
            backend_kind: convert_member_runtime_backend_kind(
                dispatch.resolved_runtime.backend_kind,
            ),
            binary_path: dispatch.resolved_runtime.binary_path.clone(),
        },
    })
}

fn convert_member_runtime_backend_kind(
    backend_kind: transport_api_types::MemberRuntimeBackendKindV1,
) -> BackendMemberRuntimeBackendKindV1 {
    match backend_kind {
        transport_api_types::MemberRuntimeBackendKindV1::Codex => {
            BackendMemberRuntimeBackendKindV1::Codex
        }
        transport_api_types::MemberRuntimeBackendKindV1::ClaudeCode => {
            BackendMemberRuntimeBackendKindV1::ClaudeCode
        }
    }
}

#[cfg(target_os = "linux")]
fn validate_member_dispatch_binding(
    dispatch: &transport_api_types::MemberDispatchRequest,
    world: &WorldHandle,
    authoritative_binding: Option<&SharedWorldBindingSnapshot>,
) -> Result<SharedWorldBindingSnapshot> {
    let dispatch = dispatch.common();
    let binding = authoritative_binding
        .cloned()
        .or_else(|| world.shared_binding.clone())
        .ok_or_else(|| {
        BadRequestError::new(format!(
            "member_dispatch requires authoritative shared world binding for orchestration session {}",
            dispatch.orchestration_session_id
        ))
    })?;

    if binding.orchestration_session_id != dispatch.orchestration_session_id {
        return Err(BadRequestError::new(format!(
            "member_dispatch.orchestration_session_id mismatch (expected {}, got {})",
            binding.orchestration_session_id, dispatch.orchestration_session_id
        ))
        .into());
    }
    if binding.world_id != dispatch.world_id {
        return Err(BadRequestError::new(format!(
            "member_dispatch.world_id mismatch (expected {}, got {})",
            binding.world_id, dispatch.world_id
        ))
        .into());
    }
    if binding.world_generation != dispatch.world_generation {
        return Err(BadRequestError::new(format!(
            "member_dispatch.world_generation mismatch (expected {}, got {})",
            binding.world_generation, dispatch.world_generation
        ))
        .into());
    }
    if binding.binding_state != SharedWorldBindingState::Active {
        return Err(BadRequestError::new(format!(
            "member_dispatch requires active shared world binding, got {:?}",
            binding.binding_state
        ))
        .into());
    }
    if world.id != dispatch.world_id {
        return Err(BadRequestError::new(format!(
            "member_dispatch.world_id mismatch (expected {}, got {})",
            world.id, dispatch.world_id
        ))
        .into());
    }

    Ok(binding)
}

pub(crate) fn resolve_shared_world_binding(
    requested: Option<&SharedWorldOwnerSpec>,
    world: &WorldHandle,
) -> Result<Option<SharedWorldBindingSnapshot>> {
    let Some(owner_spec) = requested else {
        return Ok(world.shared_binding.clone());
    };

    let binding = world.shared_binding.clone().ok_or_else(|| {
        anyhow!(
            "missing authoritative shared world proof for orchestration session {}",
            owner_spec.orchestration_session_id
        )
    })?;

    if binding.orchestration_session_id.trim().is_empty() {
        anyhow::bail!("malformed shared world proof: orchestration_session_id is empty");
    }
    if binding.orchestration_session_id != owner_spec.orchestration_session_id {
        anyhow::bail!(
            "malformed shared world proof: orchestration_session_id mismatch (expected {}, got {})",
            owner_spec.orchestration_session_id,
            binding.orchestration_session_id
        );
    }
    if binding.world_id.trim().is_empty() {
        anyhow::bail!("malformed shared world proof: world_id is empty");
    }
    if binding.world_id != world.id {
        anyhow::bail!(
            "malformed shared world proof: world_id mismatch (expected {}, got {})",
            world.id,
            binding.world_id
        );
    }
    if binding.binding_state != SharedWorldBindingState::Active {
        anyhow::bail!(
            "malformed shared world proof: binding_state must be active, got {:?}",
            binding.binding_state
        );
    }

    Ok(Some(binding))
}

pub(crate) fn apply_full_isolation_helper_env(
    env_map: &mut HashMap<String, String>,
    landlock_supported: bool,
    landlock_discover_paths: &[String],
    landlock_read_paths: &[String],
    landlock_write_paths: &[String],
    enforcement_plan_b64: Option<&str>,
) {
    if let Some(plan) = enforcement_plan_b64 {
        env_map.insert(
            enforcement_plan::WORLD_FS_ENFORCEMENT_PLAN_B64_ENV.to_string(),
            plan.to_string(),
        );
    }

    let landlock_env_needed = landlock_supported
        && (!landlock_discover_paths.is_empty()
            || !landlock_read_paths.is_empty()
            || !landlock_write_paths.is_empty());
    if landlock_env_needed {
        if !landlock_discover_paths.is_empty() {
            env_map.insert(
                WORLD_FS_LANDLOCK_DISCOVER_ALLOWLIST_ENV.to_string(),
                landlock_discover_paths.join("\n"),
            );
        }
        if !landlock_read_paths.is_empty() {
            env_map.insert(
                WORLD_FS_LANDLOCK_READ_ALLOWLIST_ENV.to_string(),
                landlock_read_paths.join("\n"),
            );
        }
        if !landlock_write_paths.is_empty() {
            env_map.insert(
                WORLD_FS_LANDLOCK_WRITE_ALLOWLIST_ENV.to_string(),
                landlock_write_paths.join("\n"),
            );
        }
    }

    let helper_needed = landlock_env_needed || enforcement_plan_b64.is_some();
    if helper_needed {
        if let Some(helper_src) = resolve_landlock_helper_src() {
            env_map
                .entry(LANDLOCK_HELPER_SRC_ENV.to_string())
                .or_insert(helper_src);
        }
    }
}

impl WorldService {
    pub(crate) fn policy_snapshot_v1_supported(&self) -> bool {
        false
    }

    pub(crate) fn last_netfilter_failure_reason(&self) -> Option<String> {
        self.last_netfilter_failure_reason
            .read()
            .expect("last netfilter failure reason lock poisoned")
            .clone()
    }

    pub(crate) fn last_netfilter_requested(&self) -> bool {
        self.last_netfilter_requested.load(Ordering::Relaxed) == 1
    }

    pub(crate) fn last_policy_resolution_mode(
        &self,
    ) -> Option<transport_api_types::PolicyResolutionModeV1> {
        match self.last_policy_resolution_mode.load(Ordering::Relaxed) {
            1 => Some(transport_api_types::PolicyResolutionModeV1::SnapshotV3),
            2 => Some(transport_api_types::PolicyResolutionModeV1::LegacyLocal),
            _ => None,
        }
    }

    pub(crate) fn set_last_policy_resolution_mode(
        &self,
        mode: transport_api_types::PolicyResolutionModeV1,
    ) {
        let encoded = match mode {
            transport_api_types::PolicyResolutionModeV1::SnapshotV3 => 1,
            transport_api_types::PolicyResolutionModeV1::LegacyLocal => 2,
        };
        self.last_policy_resolution_mode
            .store(encoded, Ordering::Relaxed);
    }

    pub(crate) fn set_last_netfilter_requested(&self, requested: bool) {
        self.last_netfilter_requested
            .store(u8::from(requested), Ordering::Relaxed);
    }

    pub(crate) fn set_last_netfilter_failure_reason(&self, reason: Option<String>) {
        *self
            .last_netfilter_failure_reason
            .write()
            .expect("last netfilter failure reason lock poisoned") = reason;
    }

    pub(crate) fn record_doctor_request_context(
        &self,
        mode: transport_api_types::PolicyResolutionModeV1,
        requested: bool,
    ) {
        self.set_last_policy_resolution_mode(mode);
        self.set_last_netfilter_requested(requested);
    }

    pub(crate) fn clear_last_netfilter_failure_on_success(&self, requested: bool) {
        if requested {
            self.set_last_netfilter_failure_reason(None);
        }
    }

    pub(crate) fn record_last_netfilter_failure_for_error(
        &self,
        requested: bool,
        error: &anyhow::Error,
    ) {
        if !requested {
            return;
        }
        if let Some(reason) = classify_last_netfilter_failure_reason(error) {
            self.set_last_netfilter_failure_reason(Some(reason));
        }
    }
}

fn classify_last_netfilter_failure_reason(error: &anyhow::Error) -> Option<String> {
    let message = format!("{error:#}");
    let is_known_failure = message.contains(NETFILTER_ENABLE_REQUIRED_TEXT)
        || message.contains(NETFILTER_NFT_FAILURE_TEXT)
        || message.contains(NETFILTER_RESOLUTION_FAILURE_TEXT)
        || message.contains(NETFILTER_NO_ADDRESS_TEXT)
        || message.contains(NETFILTER_CGROUP_ATTACH_TEXT)
        || message.contains(NETFILTER_CGROUP_HELPER_REFUSAL_TEXT)
        || message.contains(NETFILTER_CGROUP_FORCE_DIRECT_TEXT);

    if is_known_failure {
        Some(message)
    } else {
        None
    }
}

pub(crate) fn resolve_project_dir(
    env: Option<&HashMap<String, String>>,
    cwd: Option<&Path>,
) -> Result<PathBuf> {
    if let Some(path) = env
        .and_then(|map| map.get(WORLD_PROJECT_DIR_OVERRIDE_ENV))
        .map(|value| value.trim())
        .filter(|value| !value.is_empty())
    {
        return Ok(PathBuf::from(path));
    }

    let cwd_path = cwd
        .map(|path| path.to_path_buf())
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_default());

    let mode = env
        .and_then(|map| map.get(ANCHOR_MODE_ENV))
        .and_then(|value| WorldRootMode::parse(value))
        .unwrap_or(WorldRootMode::Project);

    let root_path = env
        .and_then(|map| map.get(ANCHOR_PATH_ENV))
        .map(|value| value.trim())
        .filter(|value| !value.is_empty())
        .map(PathBuf::from);

    let base_dir = match mode {
        WorldRootMode::Project => root_path.unwrap_or_else(|| cwd_path.clone()),
        WorldRootMode::FollowCwd => cwd_path.clone(),
        WorldRootMode::Custom => root_path
            .ok_or_else(|| anyhow!("anchor mode 'custom' requires SUBSTRATE_ANCHOR_PATH"))?,
    };

    Ok(base_dir)
}

#[cfg(target_os = "linux")]
pub(crate) fn is_full_isolation(env: Option<&HashMap<String, String>>) -> bool {
    if let Some(env) = env {
        if let Some(raw) = env.get(WORLD_FS_ISOLATION_ENV) {
            return raw.trim().eq_ignore_ascii_case("full");
        }
    }
    std::env::var(WORLD_FS_ISOLATION_ENV)
        .ok()
        .is_some_and(|raw| raw.trim().eq_ignore_ascii_case("full"))
}

pub(crate) fn resolve_project_write_allowlist_prefixes(
    project_dir: &Path,
    write_allowlist: &[String],
) -> Vec<String> {
    let project_str = project_dir.to_string_lossy();
    let mut prefixes: Vec<String> = Vec::new();

    for raw_pattern in write_allowlist {
        let pattern = raw_pattern.trim();
        if pattern.is_empty() {
            continue;
        }

        // Apply only allowlist entries that target paths under the project root. Reduce globs to
        // the directory prefix up to the first wildcard/meta character.
        let rel = if pattern.starts_with('/') {
            if pattern == project_str {
                "*"
            } else if let Some(stripped) = pattern.strip_prefix(&format!("{}/", project_str)) {
                stripped
            } else {
                continue;
            }
        } else {
            pattern
        };

        let rel = rel.trim_start_matches("./");

        if matches!(rel, "*" | "**" | "/*" | "/**") {
            prefixes.push(".".to_string());
            continue;
        }

        let mut prefix = rel;
        if let Some(idx) = rel.find(['*', '?', '[']) {
            prefix = &rel[..idx];
        }

        let prefix = prefix.trim_matches('/');
        if prefix.is_empty() {
            prefixes.push(".".to_string());
            continue;
        }

        if prefix.split('/').any(|c| c == "..") {
            continue;
        }

        prefixes.push(prefix.to_string());
    }

    prefixes.sort();
    prefixes.dedup();
    prefixes
}

pub(crate) fn resolve_landlock_allowlist_paths(
    project_dir: &Path,
    patterns: &[String],
) -> Vec<String> {
    let project_dir_str = project_dir.to_string_lossy();
    let mut out: Vec<String> = Vec::new();

    for raw_pattern in patterns {
        let pattern = raw_pattern.trim();
        if pattern.is_empty() {
            continue;
        }

        let pattern = pattern.trim_start_matches("./");

        // Landlock allowlists are enforced relative to the project root (mirrors the full-isolation
        // mount semantics). Absolute patterns are only honored when they refer to the project dir.
        let rel = if pattern.starts_with('/') {
            if pattern == project_dir_str {
                "*"
            } else if let Some(stripped) = pattern.strip_prefix(&format!("{}/", project_dir_str)) {
                stripped
            } else {
                continue;
            }
        } else {
            pattern
        };

        let rel = rel.trim_start_matches("./");

        if matches!(rel, "*" | "**" | "/*" | "/**") {
            out.push("/project".to_string());
            out.push(project_dir_str.to_string());
            continue;
        }

        let mut prefix = rel;
        if let Some(idx) = rel.find(['*', '?', '[']) {
            prefix = &rel[..idx];
        }

        let prefix = prefix.trim_matches('/');
        if prefix.is_empty() || prefix == "." {
            out.push("/project".to_string());
            out.push(project_dir_str.to_string());
            continue;
        }

        if prefix.split('/').any(|c| c == "..") {
            continue;
        }

        out.push(format!("/project/{prefix}"));
        out.push(format!("{}/{}", project_dir_str, prefix));
    }

    out.sort();
    out.dedup();
    out
}

#[cfg(all(test, target_os = "linux"))]
mod safe_execution_diagnostic_tests {
    use super::*;
    use std::io::Write;

    #[derive(Clone)]
    struct LogBuffer(Arc<Mutex<Vec<u8>>>);

    impl Write for LogBuffer {
        fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
            self.0.lock().unwrap().extend_from_slice(bytes);
            Ok(bytes.len())
        }

        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }

    fn capture_failure(error: &anyhow::Error, attempt_id: uuid::Uuid) -> (String, String) {
        let buffer = LogBuffer(Arc::new(Mutex::new(Vec::new())));
        let writer = buffer.clone();
        // Same fmt subscriber/event formatter as run_world_service; only output,
        // deterministic timestamp/ANSI settings and the filter differ for capture.
        let subscriber = tracing_subscriber::fmt()
            .with_max_level(tracing::Level::TRACE)
            .without_time()
            .with_ansi(false)
            .with_writer(move || writer.clone())
            .finish();
        let public = tracing::subscriber::with_default(subscriber, || {
            let span = tracing::info_span!("request", request = "span-content-sentinel");
            let _entered = span.enter();
            command_execution_failed(error, attempt_id)
        });
        let output = String::from_utf8(buffer.0.lock().unwrap().clone()).unwrap();
        (output, format!("{public:#}"))
    }

    fn assert_safe_event(
        output: &str,
        attempt_id: uuid::Uuid,
        operation: &str,
        errno: Option<i32>,
        errno_status: &str,
    ) {
        assert_eq!(output.lines().count(), 1, "{output}");
        // Assert the complete formatted payload, so any added raw field fails.
        let mut expected =
            format!(
            "exec failed service=\"world-service\" process_id={} attempt_id={} operation=\"{}\"",
            std::process::id(), attempt_id, operation,
        );
        if let Some(errno) = errno {
            expected.push_str(&format!(" raw_os_error={errno}"));
        }
        expected.push_str(&format!(" errno_status=\"{errno_status}\"\n"));
        assert_eq!(
            output.split_once("exec failed").unwrap().1,
            &expected["exec failed".len()..]
        );
        assert!(output.contains("ERROR"));
        assert!(!output.contains("span-content-sentinel"));
    }

    #[test]
    fn test_safe_diagnostic_actual_event_suppresses_wrapped_spawn_content() {
        let fixture = tempfile::tempdir().unwrap();
        let cwd = fixture.path().join("cwd-content-sentinel-missing");
        let env = HashMap::from([
            (
                "ENV_CONTENT_SENTINEL_KEY".into(),
                "env-content-sentinel-value".into(),
            ),
            ("SHIM_SESSION_ID".into(), "session-content-sentinel".into()),
            ("ACCOUNT_FIXTURE".into(), "account-content-sentinel".into()),
        ]);
        let error = world::exec::execute_shell_command(
            "printf command-content-sentinel",
            &cwd,
            &env,
            false,
            None,
        )
        .unwrap_err()
        .context("session-context-content-sentinel")
        .context("request-context-content-sentinel");
        let failure = error
            .downcast_ref::<world::exec::ShellCommandFailure>()
            .unwrap();
        assert_eq!(failure.raw_os_error(), Some(libc::ENOENT));
        let internal = format!("{error:#}");
        assert!(internal.contains("printf command-content-sentinel"));
        assert!(internal.contains("session-context-content-sentinel"));
        assert!(internal.contains("request-context-content-sentinel"));

        let mut attempts = HashSet::new();
        for _ in 0..3 {
            let attempt_id = uuid::Uuid::now_v7();
            assert!(attempts.insert(attempt_id));
            let (output, public) = capture_failure(&error, attempt_id);
            print!("{output}");
            println!("public: {public}");
            assert_safe_event(&output, attempt_id, "spawn", Some(libc::ENOENT), "present");
            assert_eq!(public, "Command execution failed");
            for forbidden in [
                "command-content-sentinel",
                "cwd-content-sentinel-missing",
                "ENV_CONTENT_SENTINEL_KEY",
                "env-content-sentinel-value",
                "SHIM_SESSION_ID",
                "session-content-sentinel",
                "ACCOUNT_FIXTURE",
                "account-content-sentinel",
                "session-context-content-sentinel",
                "request-context-content-sentinel",
                "Failed to spawn command",
                "No such file or directory",
                " error=",
                " error_debug=",
                fixture.path().to_str().unwrap(),
            ] {
                assert!(!output.contains(forbidden), "unexpected field: {forbidden}");
                assert!(!public.contains(forbidden));
            }
        }
    }

    #[test]
    fn test_safe_diagnostic_unknown_errors_never_infer_operation_or_errno() {
        for error in [
            anyhow::anyhow!("Failed to spawn command: command-content-sentinel (os error 2)"),
            anyhow::Error::new(std::io::Error::from_raw_os_error(libc::ENOENT))
                .context("Failed to wait for child process completion"),
            anyhow::Error::new(std::io::Error::other("raw-content-sentinel"))
                .context("Failed to capture stdout pipe"),
        ] {
            let attempt_id = uuid::Uuid::now_v7();
            let (output, public) = capture_failure(&error, attempt_id);
            print!("{output}");
            println!("public: {public}");
            assert_safe_event(&output, attempt_id, "unknown", None, "unknown");
            assert_eq!(public, "Command execution failed");
            for forbidden in [
                "raw-content-sentinel",
                "command-content-sentinel",
                "Failed to",
                "No such file",
                "raw_os_error=",
            ] {
                assert!(!output.contains(forbidden));
                assert!(!public.contains(forbidden));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use world_api::{SharedWorldOwnerAction, WorldHandle};

    /// Untrusted structural World/Spawn/Fork carriers traverse the real client,
    /// strict HTTP handler and independent service. None is authenticated birth.
    #[cfg(target_os = "linux")]
    #[tokio::test(flavor = "current_thread")]
    async fn test_ordinary_selected_agent_client_refuses_unheld_authority_before_effects() {
        use axum::{
            body::Body,
            http::Request,
            middleware::{self, Next},
            routing::post,
            Router,
        };
        use base64::Engine as _;
        use serde_json::json;
        use std::sync::{atomic::AtomicUsize, Mutex};
        struct SelectedTransportEffectTrapV1 {
            calls: AtomicUsize,
        }

        impl WorldBackend for SelectedTransportEffectTrapV1 {
            fn ensure_session(&self, _spec: &WorldSpec) -> Result<WorldHandle> {
                self.calls.fetch_add(1, Ordering::SeqCst);
                anyhow::bail!("selected transport test reached ensure_session")
            }

            fn exec(
                &self,
                _world: &WorldHandle,
                _request: world_api::ExecRequest,
            ) -> Result<world_api::ExecResult> {
                self.calls.fetch_add(1, Ordering::SeqCst);
                anyhow::bail!("selected transport test reached exec")
            }

            fn fs_diff(&self, _world: &WorldHandle, _span_id: &str) -> Result<world_api::FsDiff> {
                self.calls.fetch_add(1, Ordering::SeqCst);
                anyhow::bail!("selected transport test reached fs_diff")
            }

            fn apply_policy(&self, _world: &WorldHandle, _spec: &WorldSpec) -> Result<()> {
                self.calls.fetch_add(1, Ordering::SeqCst);
                anyhow::bail!("selected transport test reached apply_policy")
            }
        }

        let exclusion =
            E3PrivilegedChildExclusionV1::new_recovering().expect("hold test service namespace");
        exclusion.finish_recovery().expect("finish test recovery");
        let backend = Arc::new(SelectedTransportEffectTrapV1 {
            calls: AtomicUsize::new(0),
        });
        let replay = RuntimeReplayRegistry::with_durable_e2(None);
        let service = WorldService {
            host_gateway_turns: Arc::new(crate::gateway_runtime::HostGatewayTurnOwnerV1::new()),
            ordinary_selection_authority: None,
            accepted_home_authority: None,
            config_projection_service: None,
            e3_projection_preparations: None,
            backend: backend.clone(),
            linux_backend: Arc::new(world::LinuxLocalBackend::new()),
            gateway_runtime: Arc::new(GatewayRuntimeManager::new(Arc::clone(&exclusion))),
            member_runtime: Arc::new(MemberRuntimeManager::with_replay_and_e3_projection_service(
                replay.clone(),
                None,
                Arc::clone(&exclusion),
            )),
            runtime_replay: replay,
            member_turn_join: None,
            pending_diff_origin: Arc::new(RwLock::new(HashMap::new())),
            privileged_child_exclusion: exclusion,
            worlds: Arc::new(RwLock::new(HashMap::new())),
            budgets: Arc::new(RwLock::new(HashMap::new())),
            last_policy_resolution_mode: Arc::new(AtomicU8::new(0)),
            last_netfilter_requested: Arc::new(AtomicU8::new(0)),
            last_netfilter_failure_reason: Arc::new(RwLock::new(None)),
        };

        let received = Arc::new(Mutex::new(Vec::<Vec<u8>>::new()));
        let capture = Arc::clone(&received);
        let app = Router::new()
            .route("/v1/execute/stream", post(crate::handlers::execute_stream))
            .layer(middleware::from_fn(
                move |request: Request<Body>, next: Next<Body>| {
                    let capture = Arc::clone(&capture);
                    async move {
                        assert_eq!(request.uri().path(), "/v1/execute/stream");
                        let (parts, body) = request.into_parts();
                        let bytes = hyper::body::to_bytes(body).await.unwrap();
                        capture.lock().unwrap().push(bytes.to_vec());
                        next.run(Request::from_parts(parts, Body::from(bytes)))
                            .await
                    }
                },
            ))
            .with_state(service.clone());
        let listener = std::net::TcpListener::bind(("127.0.0.1", 0)).unwrap();
        listener.set_nonblocking(true).unwrap();
        let port = listener.local_addr().unwrap().port();
        let (stop, stopped) = tokio::sync::oneshot::channel::<()>();
        let server = tokio::spawn(
            axum::Server::from_tcp(listener)
                .unwrap()
                .serve(app.into_make_service())
                .with_graceful_shutdown(async {
                    let _ = stopped.await;
                }),
        );
        let client = transport_api_client::AgentClient::tcp("127.0.0.1", port).unwrap();
        let snapshot: transport_api_types::PolicySnapshotV3 = serde_json::from_value(json!({
            "schema_version":3,"net_allowed":[],"world_fs":{
                "host_visible":true,"fail_closed":{"routing":false},"caged_required":false,
                "write":{"enabled":true,"allow_list":[],"deny_list":[]}
            }
        }))
        .unwrap();
        let snapshot = snapshot.canonicalize().unwrap();
        let snapshot_bytes = serde_json::to_vec(&snapshot).unwrap();
        let policy_ref = json!({"ref_id":"ao_0123456789abcdef0123456789abcdef",
            "object_kind":"policy","schema_version":1,
            "commitment":{"kind":"CanonicalSha256","value":{"digest_hex":"b".repeat(64)}}});
        let commitment_ref = json!({"authority_store_id":"unheld_store",
            "commitment_id":"dpc_018f0f3a-9b2c-7def-8abc-0123456789ad","exact_linkage_hash":"a".repeat(64)});
        let start = json!({"schema_version":1,"authority_store_id":"unheld_store",
            "original_start_intent_id":"unheld_start",
            "original_start_issuer_request_id":format!("req_{}","a".repeat(64)),
            "original_session_id":"unheld_session","context_sha256":"b".repeat(64),
            "native_selection_sha256":"c".repeat(64),"requested_selection_sha256":"d".repeat(64)});
        for kind in ["world", "fresh_spawn", "fork"] {
            let mut member = json!({"schema_version":1,"orchestration_session_id":"unheld_session",
                "participant_id":"ash_unheld_member","orchestrator_participant_id":"ash_unheld_caller",
                "backend_id":"cli:codex-world","protocol":"substrate.agent.session",
                "run_id":"unheld_run","world_id":"unheld_world","world_generation":1,
                "initial_prompt":"synthetic boundary request",
                "resolved_runtime":{"backend_kind":"codex","binary_path":"/synthetic/never-spawn"}});
            if kind != "world" {
                member["e2_launch_activation"] = json!({
                    "schema_version":1,"activation_id":format!("e2a_{}","a".repeat(32)),
                    "launch_kind":kind,"commitment_ref":commitment_ref,
                    "immutable_worker_cap_ref":commitment_ref,"immutable_worker_cap_created_revision":1,
                    "immutable_worker_cap_application_revision":2,"registry_publication_revision":2,
                    "policy_snapshot_bytes_base64":base64::engine::general_purpose::STANDARD.encode(&snapshot_bytes),
                    "policy_snapshot_byte_length":snapshot_bytes.len(),"policy_snapshot_ref":policy_ref,
                    "policy_snapshot_hash":format!("{:x}",Sha256::digest(&snapshot_bytes)),
                    "policy_snapshot_revision":"synthetic_revision","request_id":"unheld_request",
                    "idempotency_key":"unheld_idempotency","orchestration_session_id":"unheld_session",
                    "caller_participant_id":"ash_unheld_caller","caller_backend_id":"cli:codex-host",
                    "target_backend_id":"cli:codex-world","retained_participant_id":"ash_unheld_member",
                    "bootstrap_run_id":"unheld_run","target_world":{"world_id":"unheld_world","world_generation":1},
                    "parent_policy_ref":policy_ref,"parent_policy_revision":"synthetic_revision"
                });
                if kind == "fresh_spawn" {
                    member["e2_launch_activation"]["reservation_ref"] = json!({
                        "authority_store_id":"unheld_store","reservation_id":"unheld_reservation",
                        "reservation_hash":"e".repeat(64)});
                    member["e2_launch_activation"]["request_commitment"] = json!({"hmac_sha256":{
                        "key_id":"unheld_key","domain":"synthetic_spawn","digest_hex":"f".repeat(64)}});
                } else {
                    member["parent_participant_id"] = json!("ash_unheld_source");
                    member["e2_launch_activation"]["source_participant_id"] =
                        json!("ash_unheld_source");
                    member["e2_launch_activation"]["request_commitment"] = json!({"canonical_sha256":{
                        "domain":"synthetic_fork","digest_hex":"f".repeat(64)}});
                }
            }
            let request: ExecuteRequest = serde_json::from_value(json!({
                "schema_version":2,"ordinary_start_selection":start,"cmd":"","pty":false,
                "cwd":"/synthetic/unheld-boundary","agent_id":"synthetic_actor",
                "policy_snapshot":snapshot,"member_dispatch":member
            }))
            .expect("structurally valid untrusted selected request");
            request.validate().unwrap();
            let expected_bytes = serde_json::to_vec(&request).unwrap();
            let result = tokio::time::timeout(
                std::time::Duration::from_secs(5),
                client.execute_stream(request),
            )
            .await
            .unwrap();
            let error = match result {
                Ok(_) => panic!("unheld authority admitted {kind}"),
                Err(error) => error,
            };
            assert!(
                error
                    .to_string()
                    .contains("ordinary initial input requires authenticated ordinary route"),
                "{kind}: {error}"
            );
            assert_eq!(received.lock().unwrap().last().unwrap(), &expected_bytes);
            assert_eq!(backend.calls.load(Ordering::SeqCst), 0, "{kind}");
            assert!(service.worlds.read().unwrap().is_empty(), "{kind}");
            assert!(service.budgets.read().unwrap().is_empty(), "{kind}");
            assert_eq!(
                service.last_policy_resolution_mode.load(Ordering::SeqCst),
                0,
                "{kind}"
            );
            assert_eq!(
                service.last_netfilter_requested.load(Ordering::SeqCst),
                0,
                "{kind}"
            );
            assert!(
                service
                    .last_netfilter_failure_reason
                    .read()
                    .unwrap()
                    .is_none(),
                "{kind}"
            );
            eprintln!("real AgentClient/strict handler/service refusal before effects: {kind}");
        }
        assert_eq!(received.lock().unwrap().len(), 3);
        stop.send(()).unwrap();
        drop(client);
        tokio::time::timeout(std::time::Duration::from_secs(5), server)
            .await
            .unwrap()
            .unwrap()
            .unwrap();
        let mut exclusive = service
            .privileged_child_exclusion
            .acquire_e3_exclusive("transport-boundary", 1)
            .unwrap();
        exclusive.release_after_cleanup_v1().unwrap();
    }

    #[cfg(target_os = "linux")]
    #[tokio::test(flavor = "current_thread")]
    async fn ordinary_selected_http_intakes_reject_raw_duplicates_before_effects() {
        use axum::{body::Body, http::Request, routing::post, Router};
        use std::sync::atomic::AtomicUsize;
        use tower::ServiceExt;

        struct RawIntakeEffectTrapV1 {
            calls: AtomicUsize,
        }

        impl WorldBackend for RawIntakeEffectTrapV1 {
            fn ensure_session(&self, _spec: &WorldSpec) -> Result<WorldHandle> {
                self.calls.fetch_add(1, Ordering::SeqCst);
                anyhow::bail!("raw intake test reached ensure_session")
            }

            fn exec(
                &self,
                _world: &WorldHandle,
                _request: world_api::ExecRequest,
            ) -> Result<world_api::ExecResult> {
                self.calls.fetch_add(1, Ordering::SeqCst);
                anyhow::bail!("raw intake test reached exec")
            }

            fn fs_diff(&self, _world: &WorldHandle, _span_id: &str) -> Result<world_api::FsDiff> {
                self.calls.fetch_add(1, Ordering::SeqCst);
                anyhow::bail!("raw intake test reached fs_diff")
            }

            fn apply_policy(&self, _world: &WorldHandle, _spec: &WorldSpec) -> Result<()> {
                self.calls.fetch_add(1, Ordering::SeqCst);
                anyhow::bail!("raw intake test reached apply_policy")
            }
        }

        let exclusion =
            E3PrivilegedChildExclusionV1::new_recovering().expect("hold test service namespace");
        exclusion.finish_recovery().expect("finish test recovery");
        let backend = Arc::new(RawIntakeEffectTrapV1 {
            calls: AtomicUsize::new(0),
        });
        let replay = RuntimeReplayRegistry::with_durable_e2(None);
        let service = WorldService {
            host_gateway_turns: Arc::new(crate::gateway_runtime::HostGatewayTurnOwnerV1::new()),
            ordinary_selection_authority: None,
            accepted_home_authority: None,
            config_projection_service: None,
            e3_projection_preparations: None,
            backend: backend.clone(),
            linux_backend: Arc::new(world::LinuxLocalBackend::new()),
            gateway_runtime: Arc::new(GatewayRuntimeManager::new(Arc::clone(&exclusion))),
            member_runtime: Arc::new(MemberRuntimeManager::with_replay_and_e3_projection_service(
                replay.clone(),
                None,
                Arc::clone(&exclusion),
            )),
            runtime_replay: replay,
            member_turn_join: None,
            pending_diff_origin: Arc::new(RwLock::new(HashMap::new())),
            privileged_child_exclusion: exclusion,
            worlds: Arc::new(RwLock::new(HashMap::new())),
            budgets: Arc::new(RwLock::new(HashMap::new())),
            last_policy_resolution_mode: Arc::new(AtomicU8::new(0)),
            last_netfilter_requested: Arc::new(AtomicU8::new(0)),
            last_netfilter_failure_reason: Arc::new(RwLock::new(None)),
        };
        let app = Router::new()
            .route("/v1/execute", post(crate::handlers::execute))
            .route("/v1/execute/stream", post(crate::handlers::execute_stream))
            .route("/v1/gateway/status", post(crate::handlers::gateway_status))
            .route("/v1/gateway/sync", post(crate::handlers::gateway_sync))
            .route(
                "/v1/gateway/restart",
                post(crate::handlers::gateway_restart),
            )
            .with_state(service.clone());
        let start = serde_json::json!({
            "schema_version": 1,
            "authority_store_id": "http_store",
            "original_start_intent_id": "http_start",
            "original_start_issuer_request_id": format!("req_{}", "a".repeat(64)),
            "original_session_id": "http_session",
            "context_sha256": "b".repeat(64),
            "native_selection_sha256": "c".repeat(64),
            "requested_selection_sha256": "d".repeat(64)
        });
        let snapshot = serde_json::json!({
            "schema_version": 3,
            "net_allowed": [],
            "world_fs": {
                "host_visible": true,
                "fail_closed": {"routing": false},
                "caged_required": false,
                "write": {"enabled": true, "allow_list": [], "deny_list": []}
            }
        });
        // These are untrusted structural carriers, with no configured authority.
        // The sentinels keep an old duplicate-erasing intake from starting effects.
        let execute = serde_json::json!({
            "schema_version": 2,
            "cmd": "",
            "cwd": "/synthetic/raw-intake",
            "pty": false,
            "agent_id": "",
            "budget": {"max_execs": 1},
            "policy_snapshot": snapshot,
            "ordinary_start_selection": start,
            "member_dispatch": {
                "schema_version": 1,
                "orchestration_session_id": "http_session",
                "participant_id": "http_member",
                "orchestrator_participant_id": "http_caller",
                "backend_id": "cli:codex-world",
                "protocol": "substrate.agent.session",
                "run_id": "http_run",
                "world_id": "http_world",
                "world_generation": 1,
                "initial_prompt": "synthetic",
                "resolved_runtime": {
                    "backend_kind": "codex",
                    "binary_path": "/synthetic/missing"
                }
            }
        });
        let gateway = serde_json::json!({
            "schema_version": 2,
            "cwd": "/synthetic/raw-intake",
            "env": {"SUBSTRATE_LLM_GATEWAY_ENABLED": "false"},
            "agent_id": "http_agent",
            "policy_snapshot": snapshot,
            "ordinary_selection": {
                "schema_version": 1,
                "start": start,
                "runtime": {
                    "selection_role": "host_actor",
                    "orchestration_session_id": "http_session",
                    "participant_id": "http_caller",
                    "backend_id": "cli:codex-host",
                    "run_id": "http_run"
                }
            }
        });
        let start_raw = serde_json::to_string(&start).unwrap();
        let execute_raw = serde_json::to_string(&execute).unwrap();
        let gateway_raw = serde_json::to_string(&gateway).unwrap();
        serde_json::from_str::<ExecuteRequest>(&execute_raw)
            .expect("valid untrusted execute shape");
        serde_json::from_str::<GatewayLifecycleRequestV1>(&gateway_raw)
            .expect("valid untrusted gateway shape");

        for (path, raw) in [
            ("/v1/execute", &execute_raw),
            ("/v1/execute/stream", &execute_raw),
            ("/v1/gateway/status", &gateway_raw),
            ("/v1/gateway/sync", &gateway_raw),
            ("/v1/gateway/restart", &gateway_raw),
        ] {
            let digest_field = format!("\"native_selection_sha256\":\"{}\"", "c".repeat(64));
            let duplicated_digest =
                start_raw.replacen(&digest_field, &format!("{digest_field},{digest_field}"), 1);
            let duplicated_start_version = start_raw.replacen(
                "\"schema_version\":1",
                "\"schema_version\":1,\"schema_version\":1",
                1,
            );
            for (case, body, expected) in [
                (
                    "outer duplicate version",
                    raw.replacen(
                        "\"schema_version\":2",
                        "\"schema_version\":2,\"schema_version\":2",
                        1,
                    ),
                    "duplicate field",
                ),
                (
                    "nested duplicate selection digest",
                    raw.replacen(&start_raw, &duplicated_digest, 1),
                    "duplicate field",
                ),
                (
                    "nested duplicate Start version",
                    raw.replacen(&start_raw, &duplicated_start_version, 1),
                    "duplicate field",
                ),
                (
                    "nested duplicate runtime run",
                    raw.replacen(
                        "\"run_id\":\"http_run\"",
                        "\"run_id\":\"http_run\",\"run_id\":\"http_run\"",
                        1,
                    ),
                    if matches!(path, "/v1/execute" | "/v1/execute/stream") {
                        "duplicate member_dispatch field `run_id`"
                    } else {
                        "duplicate field `run_id`"
                    },
                ),
                (
                    "unknown outer version",
                    raw.replacen("\"schema_version\":2", "\"schema_version\":3", 1),
                    "unsupported",
                ),
            ] {
                assert_ne!(&body, raw, "{path}: {case} must change the raw body");
                let response = app
                    .clone()
                    .oneshot(
                        Request::builder()
                            .method("POST")
                            .uri(path)
                            .header("content-type", "application/json")
                            .body(Body::from(body))
                            .unwrap(),
                    )
                    .await
                    .expect("in-process handler response");
                assert_eq!(response.status(), StatusCode::BAD_REQUEST, "{path}: {case}");
                let bytes = hyper::body::to_bytes(response.into_body()).await.unwrap();
                let error: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
                let object = error.as_object().expect("error object");
                assert_eq!(object.len(), 1, "{path}: {case}: {error}");
                let message = object["error"].as_str().expect("error message");
                assert!(
                    message.starts_with("Invalid JSON: ") && message.contains(expected),
                    "{path}: {case}: {message}"
                );
                assert_eq!(backend.calls.load(Ordering::SeqCst), 0, "{path}: {case}");
                assert!(service.budgets.read().unwrap().is_empty(), "{path}: {case}");
                assert!(service.worlds.read().unwrap().is_empty(), "{path}: {case}");
                assert_eq!(
                    service.last_policy_resolution_mode.load(Ordering::SeqCst),
                    0,
                    "{path}: {case}"
                );
                assert_eq!(
                    service.last_netfilter_requested.load(Ordering::SeqCst),
                    0,
                    "{path}: {case}"
                );
                assert!(service
                    .last_netfilter_failure_reason
                    .read()
                    .unwrap()
                    .is_none());
            }
        }
    }

    #[cfg(target_os = "linux")]
    fn exact_bound_world_policy_snapshot() -> transport_api_types::PolicySnapshotV3 {
        transport_api_types::PolicySnapshotV3 {
            schema_version: 3,
            net_allowed: Vec::new(),
            world_fs: transport_api_types::PolicySnapshotWorldFsV3 {
                host_visible: true,
                fail_closed: transport_api_types::PolicySnapshotWorldFsFailClosedV3 {
                    routing: false,
                },
                deny_enforcement: None,
                caged_required: false,
                discover: None,
                read: None,
                write: transport_api_types::PolicySnapshotWorldFsWriteV3::default(),
            },
        }
    }

    #[cfg(target_os = "linux")]
    fn e3a_v2_execute_request() -> transport_api_types::ExecuteRequest {
        use transport_api_types::{
            AuthorityObjectKindV1, AuthorityObjectRefV1, ConfigProjectionActivationCarrierV1,
            ConfigProjectionRefV1, DispatchPolicyCommitmentRefCarrierV1,
            E2LaunchRequestCommitmentV1, E2MemberLaunchActivationCarrierV1, E2MemberLaunchKindV1,
            InWorldGatewayRefV1, ManagedGatewayActivationIntentRefV1, MemberDispatchRequest,
            MemberDispatchRequestV2, MemberRuntimeBackendKindV1, OpaqueAuthorityCommitmentV1,
            ResolvedMemberRuntimeDescriptorV1, WorldBindingRefV1,
        };

        let policy_snapshot = exact_bound_world_policy_snapshot()
            .canonicalize()
            .expect("canonical E3-A policy");
        let policy_bytes = serde_json::to_vec(&policy_snapshot).expect("serialize E3-A policy");
        let policy_hash = format!("{:x}", Sha256::digest(&policy_bytes));
        let cap = DispatchPolicyCommitmentRefCarrierV1 {
            authority_store_id: "authority-store-e3a".to_string(),
            commitment_id: "dpc_018f0f2e-7b4c-7aa1-8c22-123456789ab7".to_string(),
            exact_linkage_hash: "a".repeat(64),
        };
        let authority_store_id = "cpa_018f0f2e-7b4c-7aa1-8c22-123456789ab0".to_string();
        let series_id = "cps_018f0f2e-7b4c-7aa1-8c22-123456789ab1".to_string();
        let dispatch = MemberDispatchRequestV2 {
            schema_version: 2,
            orchestration_session_id: "orch_e3a".to_string(),
            participant_id: "ash_member_e3a".to_string(),
            orchestrator_participant_id: "ash_orchestrator_e3a".to_string(),
            parent_participant_id: Some("ash_source_e3a".to_string()),
            resumed_from_participant_id: None,
            backend_id: "cli:codex".to_string(),
            protocol: "substrate.agent.session".to_string(),
            run_id: "run_e3a".to_string(),
            world_id: "world_e3a".to_string(),
            world_generation: 7,
            initial_prompt: Some("must not launch".to_string()),
            resolved_runtime: ResolvedMemberRuntimeDescriptorV1 {
                backend_kind: MemberRuntimeBackendKindV1::Codex,
                binary_path: "/bin/true".to_string(),
            },
            retained_worker_launch_authority: None,
            e2_launch_activation: Some(E2MemberLaunchActivationCarrierV1 {
                schema_version: 1,
                activation_id: format!("e2a_{}", "a".repeat(32)),
                launch_kind: E2MemberLaunchKindV1::Fork,
                reservation_ref: None,
                commitment_ref: cap.clone(),
                immutable_worker_cap_ref: cap,
                immutable_worker_cap_created_revision: 1,
                immutable_worker_cap_application_revision: 2,
                policy_snapshot_bytes_base64: BASE64.encode(&policy_bytes),
                policy_snapshot_byte_length: policy_bytes.len() as u64,
                policy_snapshot_ref: AuthorityObjectRefV1 {
                    ref_id: "ao_0123456789abcdef0123456789abcdef".to_string(),
                    object_kind: AuthorityObjectKindV1::Policy,
                    schema_version: 1,
                    commitment: OpaqueAuthorityCommitmentV1::CanonicalSha256 {
                        digest_hex: "b".repeat(64),
                    },
                },
                policy_snapshot_hash: policy_hash,
                policy_snapshot_revision: "policy-revision-e3a".to_string(),
                reason: Some("strict E3-A reachability".to_string()),
                request_id: "request-e3a".to_string(),
                idempotency_key: "idempotency-e3a".to_string(),
                orchestration_session_id: "orch_e3a".to_string(),
                caller_participant_id: "ash_orchestrator_e3a".to_string(),
                caller_backend_id: "cli:codex".to_string(),
                target_backend_id: "cli:codex".to_string(),
                retained_participant_id: "ash_member_e3a".to_string(),
                bootstrap_run_id: "run_e3a".to_string(),
                source_participant_id: Some("ash_source_e3a".to_string()),
                target_world: WorldBindingRefV1 {
                    world_id: "world_e3a".to_string(),
                    world_generation: 7,
                },
                parent_policy_ref: AuthorityObjectRefV1 {
                    ref_id: "ao_fedcba9876543210fedcba9876543210".to_string(),
                    object_kind: AuthorityObjectKindV1::Policy,
                    schema_version: 1,
                    commitment: OpaqueAuthorityCommitmentV1::CanonicalSha256 {
                        digest_hex: "c".repeat(64),
                    },
                },
                parent_policy_revision: "parent-policy-revision-e3a".to_string(),
                request_commitment: E2LaunchRequestCommitmentV1::CanonicalSha256 {
                    domain: "substrate.e3a.test".to_string(),
                    digest_hex: "d".repeat(64),
                },
                registry_publication_revision: 2,
            }),
            config_projection: ConfigProjectionActivationCarrierV1 {
                authority_store_id: authority_store_id.clone(),
                series_id: series_id.clone(),
                dormant_projection_ref: ConfigProjectionRefV1 {
                    authority_store_id: authority_store_id.clone(),
                    series_id,
                    record_id: "cpr_018f0f2e-7b4c-7aa1-8c22-123456789ab2".to_string(),
                    revision: 1,
                    record_hash: "1".repeat(64),
                },
                activation_intent_ref: ManagedGatewayActivationIntentRefV1 {
                    authority_store_id: authority_store_id.clone(),
                    activation_intent_id: "gai_018f0f2e-7b4c-7aa1-8c22-123456789ab3".to_string(),
                    intent_hash: "2".repeat(64),
                },
                expected_gateway_ref: InWorldGatewayRefV1 {
                    authority_store_id,
                    gateway_instance_id: "cgi_018f0f2e-7b4c-7aa1-8c22-123456789ab4".to_string(),
                    gateway_identity_hash: "3".repeat(64),
                },
                fence_id: "cpf_018f0f2e-7b4c-7aa1-8c22-123456789ab5".to_string(),
                consumer_id: "cpc_018f0f2e-7b4c-7aa1-8c22-123456789ab6".to_string(),
                consumer_lease_revision: 1,
                consumer_lease_hash: "4".repeat(64),
            },
        };
        dispatch.validate().expect("well-formed injected E3-A V2");

        transport_api_types::ExecuteRequest {
            schema_version: 1,
            ordinary_start_selection: None,
            profile: None,
            cmd: String::new(),
            cwd: Some("/tmp/e3a-must-not-resolve".to_string()),
            env: None,
            pty: false,
            agent_id: "e3a-no-effects".to_string(),
            budget: Some(transport_api_types::Budget {
                max_execs: Some(1),
                max_runtime_ms: None,
                max_egress_bytes: None,
            }),
            policy_snapshot,
            shared_world: None,
            world_network: None,
            world_fs_mode: None,
            member_dispatch: Some(MemberDispatchRequest::V2(dispatch)),
            acceptance_context: None,
        }
    }

    #[cfg(target_os = "linux")]
    #[tokio::test]
    async fn e3a_v2_execute_paths_fail_closed_before_world_budget_or_launch_effects() {
        let exclusion = E3PrivilegedChildExclusionV1::new_recovering()
            .expect("bind E3-A fixture child exclusion");
        exclusion
            .finish_recovery()
            .expect("finish E3-A fixture child-exclusion recovery");
        let service = WorldService::new_linux(None, exclusion).expect("E3-A test service");
        let request = e3a_v2_execute_request();
        request
            .validate()
            .expect("well-formed E3-A execute request");

        for result in [
            service.execute(request.clone()).await.map(|_| ()),
            service.execute_stream(request).await.map(|_| ()),
        ] {
            let error = result.expect_err("E3-A V2 launch must stop before implementation");
            assert_eq!(
                error
                    .downcast_ref::<BadRequestError>()
                    .map(|error| error.message.as_str()),
                Some("member_dispatch V2 launch is not implemented")
            );
        }

        assert!(service.budgets.read().unwrap().is_empty());
        assert!(service.worlds.read().unwrap().is_empty());
        assert_eq!(
            service.last_policy_resolution_mode.load(Ordering::SeqCst),
            0
        );
        assert_eq!(service.last_netfilter_requested.load(Ordering::SeqCst), 0);
        assert!(service
            .last_netfilter_failure_reason
            .read()
            .unwrap()
            .is_none());
    }

    #[cfg(target_os = "linux")]
    fn exact_bound_world_managed_dispatch(
        policy_snapshot: &transport_api_types::PolicySnapshotV3,
    ) -> transport_api_types::MemberDispatchRequest {
        use transport_api_types::{
            MemberRuntimeBackendKindV1, ResolvedMemberRuntimeDescriptorV1,
            RetainedWorkerAdmissionCommitmentCarrierV1, RetainedWorkerAuthorityObjectCommitmentV1,
            RetainedWorkerLaunchAuthorityProofV1, RetainedWorkerLaunchWorldBindingV1,
        };
        let commitment = |value: char| RetainedWorkerAuthorityObjectCommitmentV1::CanonicalSha256 {
            digest_hex: value.to_string().repeat(64),
        };
        let policy_hash = canonical_policy_snapshot_sha256(policy_snapshot).unwrap();
        transport_api_types::MemberDispatchRequest::V1(
            transport_api_types::MemberDispatchRequestV1 {
                schema_version: 1,
                orchestration_session_id: "orch_exact".into(),
                participant_id: "rwp_exact".into(),
                orchestrator_participant_id: "ash_orchestrator".into(),
                parent_participant_id: None,
                resumed_from_participant_id: None,
                backend_id: "cli:codex".into(),
                protocol: "substrate.agent.session".into(),
                run_id: "rwr_exact".into(),
                world_id: "wld_hsa_bound".into(),
                world_generation: 7,
                initial_prompt: Some("PROMPT_MUST_NOT_REACH_ADOPTION".into()),
                resolved_runtime: ResolvedMemberRuntimeDescriptorV1 {
                    backend_kind: MemberRuntimeBackendKindV1::Codex,
                    binary_path: "/bin/true".into(),
                },
                retained_worker_launch_authority: Some(RetainedWorkerLaunchAuthorityProofV1 {
                    schema_version: 1,
                    authority_store_id: "has_exact".into(),
                    issuer_request_id: "req_exact".into(),
                    canonical_spawn_fingerprint: RetainedWorkerAdmissionCommitmentCarrierV1 {
                        schema_version: 1,
                        algorithm: "hmac-sha-256".into(),
                        key_id: "adk_exact".into(),
                        digest_hex: "a".repeat(64),
                    },
                    registration_id: "rwr_registration".into(),
                    registration_commitment: commitment('b'),
                    authority_revision_after: 2,
                    authority_record_commitment_after: commitment('c'),
                    orchestration_session_id: "orch_exact".into(),
                    caller_participant_id: "ash_orchestrator".into(),
                    retained_participant_id: "rwp_exact".into(),
                    bootstrap_run_id: "rwr_exact".into(),
                    transport_claim_id: "rtc_exact".into(),
                    backend_id: "cli:codex".into(),
                    protocol: "substrate.agent.session".into(),
                    world_binding: RetainedWorkerLaunchWorldBindingV1 {
                        world_id: "wld_hsa_bound".into(),
                        world_generation: 7,
                    },
                    current_policy_ref_id: "ao_policy_exact".into(),
                    current_policy_revision: policy_hash,
                    retained_worker_ref_id: "ao_worker_exact".into(),
                    retained_worker_commitment: commitment('d'),
                }),
                e2_launch_activation: None,
            },
        )
    }

    #[test]
    #[cfg(target_os = "linux")]
    fn authority_managed_dispatch_projects_exact_bound_world_adoption_without_prompt_bytes() {
        let snapshot = exact_bound_world_policy_snapshot();
        let dispatch = exact_bound_world_managed_dispatch(&snapshot);
        let project_dir = PathBuf::from("/tmp/project-exact");
        let spec = build_world_spec(
            project_dir.clone(),
            false,
            WorldFsMode::Writable,
            false,
            Vec::new(),
            Some(&SharedWorldOwnerSpec {
                orchestration_session_id: "orch_exact".into(),
                action: SharedWorldOwnerAction::AttachOrCreate,
            }),
        );

        let adoption = exact_bound_world_ownership_adoption(&dispatch, &snapshot, &spec)
            .expect("valid exact proof")
            .expect("authority-managed adoption");

        assert_eq!(adoption.orchestration_session_id, "orch_exact");
        assert_eq!(adoption.world_id, "wld_hsa_bound");
        assert_eq!(adoption.world_generation, 7);
        assert_eq!(adoption.participant_id, "rwp_exact");
        assert_eq!(adoption.policy_ref_id, "ao_policy_exact");
        assert_eq!(adoption.policy_revision, adoption.policy_snapshot_hash);
        assert_eq!(adoption.target_spec.project_dir, project_dir);
        assert_eq!(adoption.adoption_correlation_id, "rtc_exact");
        assert!(!format!("{adoption:?}").contains("PROMPT_MUST_NOT_REACH_ADOPTION"));
    }

    #[test]
    #[cfg(target_os = "linux")]
    fn authority_managed_dispatch_requires_exact_proof_and_policy_while_legacy_none_is_unchanged() {
        let snapshot = exact_bound_world_policy_snapshot();
        let dispatch = exact_bound_world_managed_dispatch(&snapshot);
        let spec = build_world_spec(
            PathBuf::from("/tmp/project-exact"),
            false,
            WorldFsMode::Writable,
            false,
            Vec::new(),
            Some(&SharedWorldOwnerSpec {
                orchestration_session_id: "orch_exact".into(),
                action: SharedWorldOwnerAction::AttachOrCreate,
            }),
        );

        let mut missing = dispatch.clone();
        missing
            .as_v1_mut()
            .unwrap()
            .retained_worker_launch_authority = None;
        assert!(exact_bound_world_ownership_adoption(&missing, &snapshot, &spec).is_err());

        let mut policy_conflict = dispatch.clone();
        policy_conflict
            .as_v1_mut()
            .unwrap()
            .retained_worker_launch_authority
            .as_mut()
            .unwrap()
            .current_policy_revision = "f".repeat(64);
        assert!(exact_bound_world_ownership_adoption(&policy_conflict, &snapshot, &spec).is_err());

        let mut world_conflict = dispatch.clone();
        world_conflict.as_v1_mut().unwrap().world_id = "wld_other".into();
        assert!(exact_bound_world_ownership_adoption(&world_conflict, &snapshot, &spec).is_err());

        let mut legacy = dispatch;
        let legacy_v1 = legacy.as_v1_mut().unwrap();
        legacy_v1.participant_id = "ash_legacy_member".into();
        legacy_v1.run_id = "run_legacy".into();
        legacy_v1.retained_worker_launch_authority = None;
        assert!(
            exact_bound_world_ownership_adoption(&legacy, &snapshot, &spec)
                .expect("compatibility None")
                .is_none()
        );
    }

    #[test]
    #[cfg(target_os = "linux")]
    fn production_world_selection_orders_durable_adoption_before_launch_and_never_falls_back() {
        use std::cell::RefCell;

        let snapshot = exact_bound_world_policy_snapshot();
        let dispatch = exact_bound_world_managed_dispatch(&snapshot);
        let spec = build_world_spec(
            PathBuf::from("/tmp/project-exact"),
            false,
            WorldFsMode::Writable,
            false,
            Vec::new(),
            Some(&SharedWorldOwnerSpec {
                orchestration_session_id: "orch_exact".into(),
                action: SharedWorldOwnerAction::AttachOrCreate,
            }),
        );
        let adoption = exact_bound_world_ownership_adoption(&dispatch, &snapshot, &spec)
            .unwrap()
            .unwrap();
        let order = RefCell::new(Vec::new());
        let selected = select_execute_stream_world(
            Some(&adoption),
            |_| {
                order.borrow_mut().push("durable_adoption");
                Ok(WorldHandle {
                    id: "wld_hsa_bound".into(),
                    shared_binding: Some(SharedWorldBindingSnapshot {
                        orchestration_session_id: "orch_exact".into(),
                        world_id: "wld_hsa_bound".into(),
                        world_generation: 7,
                        binding_state: SharedWorldBindingState::Active,
                    }),
                })
            },
            || -> Result<WorldHandle> {
                panic!("managed selection must never call compatibility ensure_session")
            },
        )
        .expect("durably selected exact world");
        order.borrow_mut().push("member_launch");
        assert_eq!(selected.into_world().id, "wld_hsa_bound");
        assert_eq!(*order.borrow(), ["durable_adoption", "member_launch"]);

        order.borrow_mut().clear();
        let failed = select_execute_stream_world(
            Some(&adoption),
            |_| {
                order.borrow_mut().push("adoption_failed");
                anyhow::bail!("durability unavailable")
            },
            || -> Result<WorldHandle> {
                panic!("managed failure must not fall back to ensure_session")
            },
        );
        assert!(failed.is_err());
        assert_eq!(*order.borrow(), ["adoption_failed"]);
    }

    #[test]
    #[cfg(target_os = "linux")]
    fn production_world_selection_keeps_compatibility_none_on_original_ensure_path() {
        use std::cell::RefCell;

        let order = RefCell::new(Vec::new());
        let selected = select_execute_stream_world(
            None,
            |_| -> Result<WorldHandle> {
                panic!("compatibility None must not enter exact adoption")
            },
            || {
                order.borrow_mut().push("compatibility_ensure");
                Ok(WorldHandle {
                    id: "wld_legacy".into(),
                    shared_binding: None,
                })
            },
        )
        .expect("compatibility ensure_session");
        order.borrow_mut().push("member_launch");
        assert_eq!(selected.into_world().id, "wld_legacy");
        assert_eq!(*order.borrow(), ["compatibility_ensure", "member_launch"]);
    }

    #[cfg(target_os = "linux")]
    fn frame_identity(frame: &ExecuteStreamFrame) -> &transport_api_types::RuntimeFrameIdentityV1 {
        match frame {
            ExecuteStreamFrame::Start { frame_identity, .. }
            | ExecuteStreamFrame::Stdout { frame_identity, .. }
            | ExecuteStreamFrame::Stderr { frame_identity, .. }
            | ExecuteStreamFrame::Event { frame_identity, .. }
            | ExecuteStreamFrame::Exit { frame_identity, .. }
            | ExecuteStreamFrame::Error { frame_identity, .. } => frame_identity,
        }
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn runtime_event_stream_producer_assigns_gap_free_frame_and_event_identity() {
        let mut producer = RuntimeEventStreamProducer::new();
        let frames = vec![
            producer.start("spn_test".to_string()).expect("Start"),
            producer.stdout(BASE64.encode(b"out")).expect("stdout"),
            producer.stderr(BASE64.encode(b"err")).expect("stderr"),
            producer
                .event(AgentEvent::message(
                    "agent",
                    "session",
                    "run",
                    substrate_common::agent_events::MessageEventKind::Status,
                    "working",
                ))
                .expect("Event"),
            producer
                .exit(
                    0,
                    "spn_test".to_string(),
                    Vec::new(),
                    None,
                    ProcessTelemetry::default(),
                )
                .expect("Exit"),
        ];

        let stream_id = frame_identity(&frames[0]).stream_id.clone();
        assert!(!stream_id.is_empty());
        assert_eq!(
            frames
                .iter()
                .map(|frame| frame_identity(frame).frame_sequence)
                .collect::<Vec<_>>(),
            vec![1, 2, 3, 4, 5]
        );
        assert!(frames
            .iter()
            .all(|frame| frame_identity(frame).stream_id == stream_id));

        let event_identity = match &frames[3] {
            ExecuteStreamFrame::Event { event, .. } => {
                event.event_identity.as_ref().expect("semantic identity")
            }
            other => panic!("unexpected frame: {other:?}"),
        };
        assert_eq!(event_identity.event_sequence, 1);
        let (terminal_event, terminal_identity) = match &frames[4] {
            ExecuteStreamFrame::Exit {
                event_identity,
                terminal_identity,
                ..
            } => (event_identity, terminal_identity),
            other => panic!("unexpected frame: {other:?}"),
        };
        assert_eq!(terminal_event.event_sequence, 2);
        assert!(terminal_identity.matches_event(terminal_event));
        assert!(producer.stdout(BASE64.encode(b"late")).is_err());

        let original = frames[4].canonical_ndjson_bytes().expect("canonical Exit");
        assert_eq!(
            frames[4]
                .clone()
                .canonical_ndjson_bytes()
                .expect("replayed Exit"),
            original
        );
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn runtime_event_stream_producer_uses_distinct_stream_ids_and_nonterminal_errors() {
        let mut first = RuntimeEventStreamProducer::new();
        let mut second = RuntimeEventStreamProducer::new();
        let first_start = first.start("spn_first".to_string()).expect("first Start");
        let second_start = second
            .start("spn_second".to_string())
            .expect("second Start");
        assert_ne!(
            frame_identity(&first_start).stream_id,
            frame_identity(&second_start).stream_id
        );

        let error = first
            .transport_error("transport lost".to_string())
            .expect("Error");
        assert_eq!(frame_identity(&error).frame_sequence, 2);
        assert!(error.terminal_identity().is_none());
        assert!(first
            .exit(
                1,
                "spn_first".to_string(),
                Vec::new(),
                None,
                ProcessTelemetry::default(),
            )
            .is_err());
    }

    #[cfg(target_os = "linux")]
    #[tokio::test]
    async fn replay_service_returns_canonical_frames_after_exact_cursor() {
        let service = WorldService::new().expect("world service");
        let mut producer = RuntimeEventStreamProducer::new();
        let start = producer
            .start("spn_replay_service".to_string())
            .expect("Start");
        let stream_id = frame_identity(&start).stream_id.clone();
        let publisher = service
            .runtime_replay
            .begin("wwa_replay_service", &start)
            .expect("begin replay service stream");
        let stdout = producer.stdout(BASE64.encode(b"once")).expect("Stdout");
        publisher.publish(&stdout).expect("publish Stdout");
        let exit = producer
            .exit(
                0,
                "spn_replay_service".to_string(),
                Vec::new(),
                None,
                ProcessTelemetry::default(),
            )
            .expect("Exit");
        publisher.publish(&exit).expect("publish Exit");

        let response = service
            .replay_execute_stream(transport_api_types::ExecuteStreamReplayRequestV1 {
                schema_version: 1,
                acceptance_record_id: "wwa_replay_service".to_string(),
                stream_id,
                after_frame_sequence: 1,
            })
            .await
            .expect("replay response");
        let body = hyper::body::to_bytes(response.into_body())
            .await
            .expect("replay response body");
        let expected = [
            stdout.canonical_ndjson_bytes().expect("canonical Stdout"),
            exit.canonical_ndjson_bytes().expect("canonical Exit"),
        ]
        .concat();
        assert_eq!(body.as_ref(), expected.as_slice());
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn streaming_sink_enqueues_concurrent_output_in_assigned_sequence_order() {
        for _ in 0..256 {
            let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
            let producer = Arc::new(Mutex::new(RuntimeEventStreamProducer::new()));
            let start = producer
                .lock()
                .expect("producer lock")
                .start("spn_concurrent".to_string())
                .expect("Start");
            let stream_id = frame_identity(&start).stream_id.clone();
            let replay_registry = RuntimeReplayRegistry::default();
            let replay_publisher = replay_registry
                .begin("wwa_concurrent_sink", &start)
                .expect("begin concurrent sink replay");
            tx.send(start).expect("enqueue Start");
            let sink = Arc::new(StreamingSink::new(tx, producer, Some(replay_publisher)));
            let barrier = Arc::new(std::sync::Barrier::new(3));

            std::thread::scope(|scope| {
                for (kind, chunk) in [
                    (StreamKind::Stdout, b"stdout".as_slice()),
                    (StreamKind::Stderr, b"stderr".as_slice()),
                ] {
                    let sink = sink.clone();
                    let barrier = barrier.clone();
                    scope.spawn(move || {
                        barrier.wait();
                        sink.write(kind, chunk);
                    });
                }
                barrier.wait();
            });

            let frames = std::iter::from_fn(|| rx.try_recv().ok()).collect::<Vec<_>>();
            assert_eq!(frames.len(), 3);
            assert_eq!(
                frames
                    .iter()
                    .map(|frame| frame_identity(frame).frame_sequence)
                    .collect::<Vec<_>>(),
                vec![1, 2, 3]
            );
            let mut replay = replay_registry
                .subscribe(&transport_api_types::ExecuteStreamReplayRequestV1 {
                    schema_version: 1,
                    acceptance_record_id: "wwa_concurrent_sink".to_string(),
                    stream_id,
                    after_frame_sequence: 0,
                })
                .expect("subscribe to concurrent sink replay");
            let replay_sequences = std::iter::from_fn(|| replay.try_recv().ok())
                .map(|frame| frame_identity(&frame).frame_sequence)
                .collect::<Vec<_>>();
            assert_eq!(replay_sequences, vec![1, 2, 3]);
        }
    }

    #[test]
    fn landlock_helper_src_accepts_substrate_world_service_name() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let candidate = tmp.path().join("substrate-world-service");
        std::fs::write(&candidate, b"").expect("write");

        let resolved = resolve_landlock_helper_src_from_exe(&candidate);
        assert_eq!(resolved.as_deref(), Some(candidate.as_path()));
    }

    #[test]
    fn test_budget_tracker() {
        let budget = Budget {
            max_execs: Some(5),
            max_runtime_ms: Some(10000),
            max_egress_bytes: Some(1000000),
        };

        let tracker = AgentBudgetTracker::new("test-agent", budget);

        assert!(tracker.can_execute());

        // Use up all executions
        for _ in 0..5 {
            tracker.decrement_exec();
        }

        assert!(!tracker.can_execute());
    }

    #[test]
    fn test_execute_response_serde_fs_diff_roundtrip() {
        let resp = transport_api_types::ExecuteResponse {
            exit: 0,
            span_id: "spn_test".to_string(),
            stdout_b64: BASE64.encode(b"ok"),
            stderr_b64: BASE64.encode(b""),
            scopes_used: vec!["tcp:example.com:443".to_string()],
            fs_diff: Some(substrate_common::FsDiff {
                writes: vec![std::path::PathBuf::from("/tmp/a.txt")],
                mods: vec![],
                deletes: vec![],
                truncated: false,
                tree_hash: None,
                display_path: None,
                summary: None,
            }),
            shared_world: None,
            process_telemetry: ProcessTelemetry {
                process_events: vec![substrate_common::ProcessEvent {
                    event_type: substrate_common::ProcessEventType::WorldProcessStart,
                    ts: "2026-04-01T00:00:00Z".to_string(),
                    ts_unix_ns: 1_743_465_600_000_000_000,
                    session_id: "ses_test".to_string(),
                    world_id: "wld_test".to_string(),
                    pid: 42,
                    ppid: 1,
                    cwd: "/tmp".to_string(),
                    parent_span: "spn_test".to_string(),
                    parent_cmd_id: Some("cmd_test".to_string()),
                    argv: None,
                    argv_omitted: Some(true),
                    exe: None,
                    exit_code: None,
                    signal: None,
                    duration_ms: None,
                    env: None,
                }],
                process_events_status: substrate_common::ProcessEventsStatus::Truncated,
                process_events_reason: Some("capture_overflow".to_string()),
                process_events_dropped: Some(2),
                process_events_max: None,
                process_events_backend: None,
                process_events_error: None,
            },
        };

        let json = serde_json::to_string(&resp).expect("serialize ExecuteResponse");
        let back: transport_api_types::ExecuteResponse =
            serde_json::from_str(&json).expect("deserialize ExecuteResponse");

        assert_eq!(back.exit, 0);
        assert_eq!(back.span_id, "spn_test");
        assert_eq!(back.scopes_used, vec!["tcp:example.com:443".to_string()]);
        assert_eq!(
            back.process_telemetry.process_events_status,
            substrate_common::ProcessEventsStatus::Truncated
        );
        assert_eq!(
            back.process_telemetry.process_events_reason.as_deref(),
            Some("capture_overflow")
        );
        assert_eq!(back.process_telemetry.process_events_dropped, Some(2));
        assert_eq!(back.process_telemetry.process_events.len(), 1);
        let fd = back.fs_diff.expect("fs_diff present");
        assert_eq!(fd.writes.len(), 1);
        assert_eq!(fd.writes[0], std::path::PathBuf::from("/tmp/a.txt"));
        assert!(fd.mods.is_empty());
        assert!(fd.deletes.is_empty());
    }

    #[test]
    fn build_world_spec_uses_shared_orchestration_reuse_mode() {
        let owner = SharedWorldOwnerSpec {
            orchestration_session_id: "orch_test".to_string(),
            action: SharedWorldOwnerAction::AttachOrCreate,
        };

        let spec = build_world_spec(
            PathBuf::from("/tmp/project"),
            true,
            WorldFsMode::Writable,
            false,
            vec!["example.com".to_string()],
            Some(&owner),
        );

        assert_eq!(spec.reuse_mode, WorldReuseMode::SharedOrchestration(owner));
    }

    #[test]
    fn resolve_shared_world_binding_rejects_mismatched_owner_proof() {
        let requested = SharedWorldOwnerSpec {
            orchestration_session_id: "orch_expected".to_string(),
            action: SharedWorldOwnerAction::AttachOrCreate,
        };
        let world = WorldHandle {
            id: "wld_test".to_string(),
            shared_binding: Some(SharedWorldBindingSnapshot {
                orchestration_session_id: "orch_other".to_string(),
                world_id: "wld_test".to_string(),
                world_generation: 0,
                binding_state: SharedWorldBindingState::Active,
            }),
        };

        let err = resolve_shared_world_binding(Some(&requested), &world).expect_err("mismatch");
        assert!(
            err.to_string()
                .contains("orchestration_session_id mismatch"),
            "unexpected error: {err:#}"
        );
    }

    #[test]
    fn process_telemetry_redaction_helpers_redact_common_secret_shapes() {
        let argv = vec![
            "curl".to_string(),
            "-H".to_string(),
            "Authorization: Bearer abc123".to_string(),
            "--token".to_string(),
            "secret".to_string(),
            "--password=supersecret".to_string(),
        ];

        let redacted = substrate_common::redact_process_argv(&argv);
        assert_eq!(redacted[2], "Authorization: Bearer ***");
        assert_eq!(redacted[4], "***");
        assert_eq!(redacted[5], "--password=***");

        assert!(substrate_common::process_env_key_allowlisted("HTTP_PROXY"));
        assert!(!substrate_common::process_env_key_allowlisted(
            "AWS_SECRET_ACCESS_KEY"
        ));
        assert_eq!(
            substrate_common::redact_process_env_value(
                "HTTP_PROXY",
                "http://user:pass@example.com"
            ),
            "http://***@example.com"
        );
    }

    #[test]
    fn world_deps_provision_profile_wraps_command_in_transient_systemd_unit() {
        let cmd = wrap_command_for_profile(
            Some("world-deps-provision"),
            Path::new("/tmp/substrate world"),
            "echo 'hello'",
        );

        assert!(cmd.starts_with("systemd-run "));
        assert!(cmd.contains("--wait"));
        assert!(cmd.contains("--pipe"));
        assert!(cmd.contains("--collect"));
        assert!(cmd.contains("--working-directory='/tmp/substrate world'"));
        assert!(cmd.contains("/bin/sh -lc 'echo '\"'\"'hello'\"'\"''"));
    }

    #[test]
    fn non_provision_profiles_do_not_wrap_command() {
        let cmd = wrap_command_for_profile(
            Some("world-deps-probe"),
            Path::new("/tmp/substrate world"),
            "dpkg-query -W",
        );
        assert_eq!(cmd, "dpkg-query -W");

        let cmd = wrap_command_for_profile(None, Path::new("/tmp"), "echo ok");
        assert_eq!(cmd, "echo ok");
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn test_resolve_helper_src_prefers_sibling_world_service_over_test_harness() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let deps = tmp.path().join("target").join("debug").join("deps");
        std::fs::create_dir_all(&deps).expect("create deps");

        let harness = deps.join("wfgad3_wildcard_deny_symlink_handling-abcdef");
        std::fs::write(&harness, b"").expect("write harness");

        let world_service = deps.parent().unwrap().join("world-service");
        std::fs::write(&world_service, b"").expect("write world-service");

        let resolved =
            resolve_landlock_helper_src_from_exe(&harness).expect("resolve from harness exe");
        assert_eq!(resolved, world_service);
    }

    #[cfg(target_os = "linux")]
    #[tokio::test]
    async fn test_service_creation() {
        let service = WorldService::new().unwrap();
        assert_eq!(service.worlds.read().unwrap().len(), 0);
        assert!(!service.last_netfilter_requested());
        assert!(service.last_netfilter_failure_reason().is_none());
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn record_doctor_request_context_updates_requested_state_for_non_pty_requests() {
        let service = WorldService::new().expect("service");

        service.record_doctor_request_context(
            transport_api_types::PolicyResolutionModeV1::SnapshotV3,
            true,
        );

        assert_eq!(
            service.last_policy_resolution_mode(),
            Some(transport_api_types::PolicyResolutionModeV1::SnapshotV3)
        );
        assert!(service.last_netfilter_requested());

        service.record_doctor_request_context(
            transport_api_types::PolicyResolutionModeV1::LegacyLocal,
            false,
        );

        assert_eq!(
            service.last_policy_resolution_mode(),
            Some(transport_api_types::PolicyResolutionModeV1::LegacyLocal)
        );
        assert!(!service.last_netfilter_requested());
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn record_last_netfilter_failure_for_error_tracks_only_published_failure_classes() {
        let service = WorldService::new().expect("service");
        let known_errors = [
            anyhow::anyhow!(NETFILTER_ENABLE_REQUIRED_TEXT),
            anyhow::anyhow!("nft command failed: permission denied"),
            anyhow::anyhow!("failed to resolve allowed domain `example.com`: dns lookup failed"),
            anyhow::anyhow!("allowed domain `example.com` resolved to no addresses"),
            anyhow::anyhow!(
                "project bind mount helper refused isolated execution before command start: substrate: error: project_bind_mount: cgroup attach failed: /sys/fs/cgroup/substrate/world/cgroup.procs"
            ),
        ];

        for err in &known_errors {
            service.set_last_netfilter_failure_reason(None);
            service.record_last_netfilter_failure_for_error(true, err);
            assert_eq!(
                service.last_netfilter_failure_reason(),
                Some(format!("{err:#}"))
            );
        }
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn record_last_netfilter_failure_for_error_ignores_unrelated_or_unrequested_errors() {
        let service = WorldService::new().expect("service");
        service.record_last_netfilter_failure_for_error(
            true,
            &anyhow::anyhow!("plain command failed"),
        );
        assert!(service.last_netfilter_failure_reason().is_none());

        service.record_last_netfilter_failure_for_error(
            false,
            &anyhow::anyhow!(NETFILTER_ENABLE_REQUIRED_TEXT),
        );
        assert!(service.last_netfilter_failure_reason().is_none());
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn clear_last_netfilter_failure_on_success_only_resets_isolated_runs() {
        let service = WorldService::new().expect("service");
        service.set_last_netfilter_failure_reason(Some("nft command failed: denied".to_string()));

        service.clear_last_netfilter_failure_on_success(false);
        assert_eq!(
            service.last_netfilter_failure_reason(),
            Some("nft command failed: denied".to_string())
        );

        service.clear_last_netfilter_failure_on_success(true);
        assert!(service.last_netfilter_failure_reason().is_none());
    }

    #[test]
    fn resolve_project_dir_prefers_internal_override() {
        let mut env = HashMap::new();
        env.insert(
            WORLD_PROJECT_DIR_OVERRIDE_ENV.to_string(),
            "/tmp/substrate-project-root".to_string(),
        );
        env.insert(ANCHOR_MODE_ENV.to_string(), "follow-cwd".to_string());
        env.insert(
            ANCHOR_PATH_ENV.to_string(),
            "/tmp/ignored-anchor".to_string(),
        );

        let project_dir =
            resolve_project_dir(Some(&env), Some(Path::new("/tmp/overridden-cwd"))).unwrap();

        assert_eq!(project_dir, PathBuf::from("/tmp/substrate-project-root"));
    }

    #[cfg(not(target_os = "linux"))]
    #[test]
    fn test_service_creation_fails_on_non_linux() {
        let result = WorldService::new();
        assert!(result.is_err());
    }
}

#[cfg(all(test, target_os = "linux"))]
#[path = "ordinary_world_integration_tests.rs"]
mod ordinary_world_integration_tests;

#[cfg(target_os = "linux")]
impl WorldService {
    // Called only after the inherited-listener/peer extension has authenticated
    // the configured nonroot principal. The detached operation, rather than the
    // HTTP waiter, retains table ownership through startup and checked disposal.
    pub(crate) async fn host_gateway_turn(
        &self,
        principal_uid: u32,
        request: transport_api_types::HostGatewayInstanceRequestV3,
    ) -> Result<transport_api_types::HostGatewayInstanceResponseV3> {
        request
            .validate()
            .map_err(|_| anyhow!("host_gateway_turn_invalid_request"))?;
        let owner = Arc::clone(&self.host_gateway_turns);
        let authority = self.ordinary_selection_authority.clone();
        let exclusion = Arc::clone(&self.privileged_child_exclusion);
        tokio::spawn(async move {
            owner
                .execute(principal_uid, request, authority, exclusion)
                .await
        })
        .await
        .map_err(|_| anyhow!("host_gateway_turn_outcome_unknown"))?
    }
}

#[cfg(target_os = "linux")]
impl WorldService {
    pub(crate) fn workspace_current_v1(
        &self,
        peer: &substrate_shell::WorkspaceNativePeerV1,
        request: &transport_api_types::WorkspaceCurrentRequestV1,
    ) -> transport_api_types::WorkspaceCurrentResponseV1 {
        // This is the actual service-owned backend. No substitute is constructed.
        substrate_shell::workspace_current_for_peer_v1(peer, request, &self.linux_backend)
    }
}

#[cfg(all(test, target_os = "linux"))]
mod workspace_current_joined_private_tests {
    use super::*;
    use std::os::unix::fs::{MetadataExt, PermissionsExt};
    use std::path::{Path, PathBuf};
    use transport_api_types::{
        WorkspaceCurrentOperationV1 as O, WorkspaceCurrentRequestV1 as Q,
        WorkspaceCurrentResponseV1 as R,
    };

    fn root() -> PathBuf {
        let root = PathBuf::from(
            std::env::var_os("TASK22_PRIVATE_ROOT").expect("exact released fixture root"),
        );
        let product = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .ancestors()
            .nth(2)
            .expect("compiled source checkout");
        let private_parent = product.join(".codex/task20-workspace-proof/joined");
        assert_eq!(
            root.parent(),
            Some(private_parent.as_path()),
            "exact owned private source-proof case"
        );
        assert_eq!(unsafe { libc::geteuid() }, 0);
        assert_eq!(
            std::env::var("TASK22_PRIVATE_NAMESPACE").unwrap(),
            "joined-v1"
        );
        root
    }
    // Fixture processes must never inherit Root's real HOME, provider/shim settings,
    // or one identity's home/user environment after setpriv selects the other identity.
    fn configure_private_environment(command: &mut std::process::Command, root: &Path, uid: u32) {
        let (home, account, runtime, temporary) = match uid {
            0 => (
                PathBuf::from("/run/task22-root/home"),
                "root",
                PathBuf::from("/run/task22-service-runtime"),
                PathBuf::from("/run/task22-root/tmp"),
            ),
            1000 => (
                PathBuf::from("/run/task22-owner/home"),
                "task22-owner",
                PathBuf::from("/run/task22-owner/unit-runtime"),
                PathBuf::from("/run/task22-owner/tmp"),
            ),
            _ => panic!("fixture supports exactly Root and its native owner"),
        };
        command
            .env_clear()
            .env("PATH", "/usr/bin:/bin")
            .env("HOME", &home)
            .env("USER", account)
            .env("LOGNAME", account)
            .env("XDG_CONFIG_HOME", home.join(".config"))
            .env("XDG_CACHE_HOME", home.join(".cache"))
            .env("XDG_DATA_HOME", home.join(".local/share"))
            .env("XDG_RUNTIME_DIR", runtime)
            .env("TMPDIR", temporary)
            .env("LANG", "C")
            .env("LC_ALL", "C")
            .env("PYTHONDONTWRITEBYTECODE", "1")
            .env("PYTHONNOUSERSITE", "1")
            .env(
                "TASK22_PRODUCT",
                std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                    .ancestors()
                    .nth(2)
                    .expect("compiled source checkout"),
            )
            .env("TASK22_PRIVATE_ROOT", root)
            .env("TASK22_PRIVATE_NAMESPACE", "joined-v1");
        for key in [
            "TASK22_PRIVATE_CASE",
            "TASK22_SHELL_TEST_BINARY",
            "TASK22_SERVICE_TEST_BINARY",
            "TASK22_SUBSTRATE_BINARY",
        ] {
            command.env(
                key,
                std::env::var_os(key).expect("released task-local artifact/input"),
            );
        }
    }
    fn shell_driver(
        root: &Path,
        mode: &str,
        world: &world_api::WorldHandle,
        uid: u32,
    ) -> std::process::Output {
        let mut command = std::process::Command::new("/usr/bin/setpriv");
        configure_private_environment(&mut command, root, uid);
        command
            .arg(format!("--reuid={uid}"))
            .arg(format!("--regid={uid}"))
            .arg("--clear-groups")
            .arg(
                std::env::var_os("TASK22_SHELL_TEST_BINARY")
                    .expect("built private shell unit binary"),
            );
        command.args(["execution::agent_runtime::host_session_authority::facade::workspace_current_private_driver_v1::test_workspace_current_private_producer_driver", "--ignored", "--exact", "--nocapture"])
            .current_dir(root).env("TASK22_DRIVER_MODE", mode).env("TASK22_WORLD_ID", &world.id)
            .env("TASK22_WORLD_GENERATION", world.shared_binding.as_ref().unwrap().world_generation.to_string());
        command
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped());
        let child = command.spawn().unwrap();
        if mode == "transport-held" {
            // Actual Child handle is the witness; do not infer a PID from set differences.
            let witness = root.join("control/transport-caller.pid");
            assert!(!witness.exists());
            std::fs::write(&witness, child.id().to_string()).unwrap();
        }
        let output = child.wait_with_output().unwrap();
        assert!(
            output.status.success(),
            "driver {mode}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        output
    }
    fn native(root: &Path, operation: &str, request: &Q) -> (bool, R) {
        let file = root.join("control/native-request.json");
        std::fs::write(&file, serde_json::to_vec(request).unwrap()).unwrap();
        // Harness input only; the UID-1000-owned control directory remains 0700.
        std::fs::set_permissions(&file, std::fs::Permissions::from_mode(0o644)).unwrap();
        let mut command = std::process::Command::new("/usr/bin/setpriv");
        configure_private_environment(&mut command, root, 1000);
        let output = command
            .args(["--reuid=1000", "--regid=1000", "--clear-groups"])
            .arg(std::env::var_os("TASK22_SUBSTRATE_BINARY").expect("built actual native binary"))
            .args(["--install-prefix"])
            .arg(PathBuf::from("/run/task22-owner/home"))
            .args(["workspace", operation, "--request"])
            .arg(&file)
            .current_dir(PathBuf::from("/run/task22-owner/workspace"))
            .env(
                "SUBSTRATE_WORLD_SOCKET",
                PathBuf::from("/run/task22-current.sock"),
            )
            .output()
            .unwrap();
        let value = serde_json::from_slice(&output.stdout).unwrap_or_else(|error| {
            panic!(
                "native response {error}; stdout={} stderr={}",
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
            )
        });
        (output.status.success(), value)
    }
    fn direct(root: &Path, world: &world_api::WorldHandle, request: &Q, uid: u32) -> R {
        direct_mode(root, world, request, uid, "transport")
    }
    fn direct_mode(
        root: &Path,
        world: &world_api::WorldHandle,
        request: &Q,
        uid: u32,
        mode: &str,
    ) -> R {
        std::fs::write(
            root.join("control/transport-request.json"),
            serde_json::to_vec(request).unwrap(),
        )
        .unwrap();
        std::fs::set_permissions(
            root.join("control/transport-request.json"),
            std::fs::Permissions::from_mode(0o644),
        )
        .unwrap();
        let output = shell_driver(root, mode, world, uid);
        let stdout = String::from_utf8(output.stdout).unwrap();
        let line = stdout
            .lines()
            .find_map(|line| line.strip_prefix("TASK22_RESPONSE="))
            .expect("typed direct response");
        serde_json::from_str(line).unwrap()
    }
    #[derive(PartialEq, Eq)]
    struct State {
        metadata: (u64, u64, u32, u32, u32, u64, u64, i64, i64, i64, i64),
        bytes: Vec<u8>,
    }
    impl std::fmt::Debug for State {
        fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            formatter
                .debug_struct("State")
                .field("metadata", &self.metadata)
                .field("content_byte_count", &self.bytes.len())
                .finish()
        }
    }
    #[derive(Debug, PartialEq, Eq)]
    struct ObservedState {
        files: std::collections::BTreeMap<PathBuf, State>,
        namespaces: Vec<PathBuf>,
        mountinfo: String,
        cgroup_membership: String,
        children: std::collections::BTreeSet<u32>,
    }
    fn snapshot(root: &Path) -> ObservedState {
        fn visit(root: &Path, path: &Path, out: &mut std::collections::BTreeMap<PathBuf, State>) {
            let m = std::fs::symlink_metadata(path).unwrap();
            // This exact directory contains only harness inputs/negative preimages. It is
            // never an owner authority, backend root, runtime, socket or cgroup directory.
            if path == root.join("control") {
                return;
            }
            // Socket entries are measured by identity/mode/ownership like other entries.
            out.insert(
                path.strip_prefix(root).unwrap().to_path_buf(),
                State {
                    metadata: (
                        m.dev(),
                        m.ino(),
                        m.mode(),
                        m.uid(),
                        m.gid(),
                        m.nlink(),
                        m.len(),
                        m.mtime(),
                        m.mtime_nsec(),
                        m.ctime(),
                        m.ctime_nsec(),
                    ),
                    bytes: if m.is_file() {
                        std::fs::read(path).unwrap()
                    } else {
                        Vec::new()
                    },
                },
            );
            if m.is_dir() {
                for child in std::fs::read_dir(path).unwrap() {
                    visit(root, &child.unwrap().path(), out);
                }
            }
        }
        let mut files = std::collections::BTreeMap::new();
        visit(root, root, &mut files);
        let mut children = std::collections::BTreeSet::new();
        for task in std::fs::read_dir("/proc/self/task").unwrap() {
            if let Ok(bytes) = std::fs::read_to_string(task.unwrap().path().join("children")) {
                for pid in bytes.split_whitespace() {
                    children.insert(pid.parse().unwrap());
                }
            }
        }
        ObservedState {
            files,
            namespaces: ["mnt", "net", "pid", "user", "cgroup"]
                .into_iter()
                .map(|name| std::fs::read_link(format!("/proc/self/ns/{name}")).unwrap())
                .collect(),
            mountinfo: std::fs::read_to_string("/proc/self/mountinfo").unwrap(),
            cgroup_membership: std::fs::read_to_string("/proc/self/cgroup").unwrap(),
            children,
        }
    }
    fn read_request(root: &Path, publication: &R) -> Q {
        let mut request: Q =
            serde_json::from_slice(&std::fs::read(root.join("control/request.json")).unwrap())
                .unwrap();
        let R::AssociationPublished {
            lane_ref,
            association_incarnation,
            association_revision,
            backend_custody_ref,
            ..
        } = publication
        else {
            panic!("genuine publisher result");
        };
        request.operation = O::CurrentActorRead {
            lane_ref: lane_ref.clone(),
            association_incarnation: association_incarnation.clone(),
            association_revision: *association_revision,
            backend_custody_ref: backend_custody_ref.clone(),
            expected_actor: std::fs::read_to_string(root.join("control/expected-actor.txt"))
                .unwrap(),
        };
        request
    }

    fn selected_read_request_v1(
        root: &Path,
        publication: &transport_api_types::WorkspaceAssociationUpdateResponseV1,
    ) -> Q {
        let transport_api_types::WorkspaceAssociationUpdateResponseV1::AssociationPublished {
            lane_ref,
            association_incarnation,
            association_revision,
            backend_custody_ref,
            ..
        } = publication
        else {
            panic!("actual selected association publisher");
        };
        let mut request: Q =
            serde_json::from_slice(&std::fs::read(root.join("control/request.json")).unwrap())
                .unwrap();
        request.operation = O::CurrentActorRead {
            lane_ref: lane_ref.clone(),
            association_incarnation: association_incarnation.clone(),
            association_revision: *association_revision,
            backend_custody_ref: backend_custody_ref.clone(),
            expected_actor: std::fs::read_to_string(root.join("control/expected-actor.txt"))
                .unwrap(),
        };
        request
    }

    async fn preflight_gate_v1(
        axum::extract::State(root): axum::extract::State<PathBuf>,
        request: hyper::Request<hyper::Body>,
        next: axum::middleware::Next<hyper::Body>,
    ) -> axum::response::Response {
        if request.uri().path() == "/v1/workspace/preflight"
            && root.join("control/preflight-gate").exists()
        {
            std::fs::write(
                root.join("control/preflight-gate-arrived"),
                b"native C1 completed; service before C1",
            )
            .unwrap();
            let deadline = std::time::Instant::now() + std::time::Duration::from_secs(20);
            while !root.join("control/preflight-gate-release").exists()
                && std::time::Instant::now() < deadline
            {
                tokio::time::sleep(std::time::Duration::from_millis(10)).await;
            }
            assert!(
                root.join("control/preflight-gate-release").exists(),
                "bounded source proof gate"
            );
        }
        next.run(request).await
    }

    fn native_adapter_v1(
        root: &Path,
        operation: &str,
        body: Option<&serde_json::Value>,
        path: Option<&Path>,
    ) -> serde_json::Value {
        let request_file = root.join("control/preflight-setup-request.json");
        if let Some(body) = body {
            std::fs::write(&request_file, serde_json::to_vec(body).unwrap()).unwrap();
            std::fs::set_permissions(&request_file, std::fs::Permissions::from_mode(0o600))
                .unwrap();
            use std::os::unix::ffi::OsStrExt;
            let name = std::ffi::CString::new(request_file.as_os_str().as_bytes()).unwrap();
            assert_eq!(unsafe { libc::chown(name.as_ptr(), 1000, 1000) }, 0);
        }
        let mut command = std::process::Command::new("/usr/bin/setpriv");
        configure_private_environment(&mut command, root, 1000);
        command
            .args(["--reuid=1000", "--regid=1000", "--clear-groups"])
            .arg(std::env::var_os("TASK22_SUBSTRATE_BINARY").unwrap())
            .args([
                "--install-prefix",
                "/run/task22-owner/home",
                "workspace",
                operation,
            ])
            .env("SUBSTRATE_WORLD_SOCKET", "/run/task22-current.sock")
            .current_dir("/run/task22-owner/workspace");
        if body.is_some() {
            command.arg("--request").arg(&request_file);
        } else if let Some(path) = path {
            command.arg(path);
        }
        let output = command.output().unwrap();
        serde_json::from_slice(&output.stdout).unwrap_or_else(|e| {
            panic!(
                "native {operation}: {e}: stdout={} stderr={}",
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
            )
        })
    }

    fn run_preflight_profile_v1(
        service: &WorldService,
        root: &Path,
        world: &world_api::WorldHandle,
        spec: &world_api::WorldSpec,
    ) {
        use sha2::{Digest, Sha256};
        use world_api::WorldBackend;
        shell_driver(root, "revision-settings", world, 1000);
        let mut sibling_spec = spec.clone();
        sibling_spec.reuse_mode =
            world_api::WorldReuseMode::SharedOrchestration(world_api::SharedWorldOwnerSpec {
                orchestration_session_id: "session-sibling".into(),
                action: world_api::SharedWorldOwnerAction::AttachOrCreate,
            });
        let sibling_world = service.linux_backend.ensure_session(&sibling_spec).unwrap();
        shell_driver(root, "setup-sibling", &sibling_world, 1000);
        let mut sibling: Q =
            serde_json::from_slice(&std::fs::read(root.join("control/request.json")).unwrap())
                .unwrap();
        let sibling_actor =
            std::fs::read_to_string(root.join("control/expected-actor.txt")).unwrap();
        shell_driver(root, "observe", world, 1000);
        let publish: Q =
            serde_json::from_slice(&std::fs::read(root.join("control/request.json")).unwrap())
                .unwrap();
        let publication = native_adapter_v1(
            root,
            "publish-world-association",
            Some(&serde_json::json!({"current":publish,"active":true})),
            None,
        );
        assert_eq!(
            publication["state"], "AssociationPublished",
            "{publication}"
        );
        let publication: transport_api_types::WorkspaceAssociationUpdateResponseV1 =
            serde_json::from_value(publication).unwrap();
        let current = selected_read_request_v1(root, &publication);
        let sibling_publication = native_adapter_v1(
            root,
            "publish-world-association",
            Some(&serde_json::json!({"current":sibling,"active":true})),
            None,
        );
        assert_eq!(
            sibling_publication["state"], "AssociationPublished",
            "{sibling_publication}"
        );
        let transport_api_types::WorkspaceAssociationUpdateResponseV1::AssociationPublished {
            lane_ref,
            association_incarnation,
            association_revision,
            backend_custody_ref,
            ..
        } = serde_json::from_value(sibling_publication).unwrap()
        else {
            panic!("genuine sibling publisher");
        };
        sibling.operation = O::CurrentActorRead {
            lane_ref,
            association_incarnation,
            association_revision,
            backend_custody_ref,
            expected_actor: sibling_actor,
        };
        let initial = PathBuf::from("/run/task22-owner/initial-integration");
        std::fs::create_dir(&initial).unwrap();
        std::fs::set_permissions(&initial, std::fs::Permissions::from_mode(0o700)).unwrap();
        let name = std::ffi::CString::new(initial.to_str().unwrap()).unwrap();
        assert_eq!(unsafe { libc::chown(name.as_ptr(), 1000, 1000) }, 0);
        let offered = native_adapter_v1(root, "open", None, Some(&initial));
        assert_eq!(offered["state"], "InitializationOffered", "{offered}");
        let native = PathBuf::from(std::env::var_os("TASK22_SUBSTRATE_BINARY").unwrap());
        let writer = PathBuf::from(std::env::var_os("TASK22_SHELL_TEST_BINARY").unwrap());
        if std::env::var("TASK22_PRIVATE_CASE").as_deref() == Ok("preflight-custody") {
            let mut selected = current.clone();
            let metadata = PathBuf::from("/run/task22-service-runtime/substrate/worlds")
                .join(&world.id)
                .join("session.json");
            let original = std::fs::read(&metadata).unwrap();
            for defect in ["pending", "corrupt", "same-byte-restore"] {
                let before = snapshot(root);
                let reply =
                    native_adapter_v1(root, "preflight", Some(&serde_json::json!(selected)), None);
                assert_eq!(reply["state"], "EligibleObservation", "{reply}");
                assert_eq!(snapshot(root), before, "positive custody preflight PF-ZERO");
                let pending = metadata.with_file_name(".session.json.preflight-pending.tmp");
                match defect {
                    "pending" => std::fs::write(&pending, b"incomplete").unwrap(),
                    "corrupt" => std::fs::write(&metadata, b"{invalid").unwrap(),
                    _ => {
                        std::fs::write(&metadata, b"changed").unwrap();
                        std::fs::write(&metadata, &original).unwrap();
                    }
                }
                let before = snapshot(root);
                let reply =
                    native_adapter_v1(root, "preflight", Some(&serde_json::json!(selected)), None);
                assert_ne!(reply["state"], "EligibleObservation", "custody {defect}");
                assert_eq!(
                    snapshot(root),
                    before,
                    "failed custody preflight PF-ZERO {defect}"
                );
                if defect == "pending" {
                    std::fs::remove_file(pending).unwrap();
                } else if defect == "corrupt" {
                    std::fs::write(&metadata, &original).unwrap();
                }
                let publication = native_adapter_v1(
                    root,
                    "publish-world-association",
                    Some(&serde_json::json!({"current":publish,"active":true})),
                    None,
                );
                assert_eq!(
                    publication["state"], "AssociationPublished",
                    "{publication}"
                );
                selected =
                    selected_read_request_v1(root, &serde_json::from_value(publication).unwrap());
                println!("PF-REV/PF-ZERO native custody {defect} refusal");
            }
            let before = snapshot(root);
            let reply =
                native_adapter_v1(root, "preflight", Some(&serde_json::json!(selected)), None);
            assert_eq!(reply["state"], "EligibleObservation", "{reply}");
            assert_eq!(snapshot(root), before);
            let replacement = world_api::WorldSpec {
                reuse_mode: world_api::WorldReuseMode::SharedOrchestration(
                    world_api::SharedWorldOwnerSpec {
                        orchestration_session_id: "session-start-1".into(),
                        action: world_api::SharedWorldOwnerAction::ReplaceExpectedGeneration {
                            expected_generation: 0,
                            reason: "explicit private preflight owner invalidation".into(),
                        },
                    },
                ),
                ..spec.clone()
            };
            let produced = service.linux_backend.ensure_session(&replacement).unwrap();
            assert_eq!(
                produced.shared_binding.as_ref().unwrap().world_generation,
                1
            );
            let before = snapshot(root);
            let reply =
                native_adapter_v1(root, "preflight", Some(&serde_json::json!(selected)), None);
            assert_ne!(
                reply["state"], "EligibleObservation",
                "genuine backend owner World generation invalidation"
            );
            assert_eq!(
                snapshot(root),
                before,
                "World invalidated failed read zero effects"
            );
            println!("PF-REV/PF-LATER/PF-ZERO actual backend owner replacement refused stale no-send observation");
            return;
        }
        let profile = serde_json::json!({"schema_version":1,"qualification":"private-source","native_binary":native,
            "native_sha256":format!("{:x}",Sha256::digest(std::fs::read(&native).unwrap())),"writer_binary":writer,
            "writer_sha256":format!("{:x}",Sha256::digest(std::fs::read(&writer).unwrap())),"native_home":"/run/task22-owner/home","native_runtime":"/run/task22-owner/unit-runtime","native_tmp":"/run/task22-owner/tmp","initialization_effect_path":root.join("run/task22-owner/initial-integration"),
            "socket":"/run/task22-current.sock","selected_location":"/run/task22-owner/workspace","effect_root":root,"control":root.join("control"),
            "native_context":current.native_context,"current":current,"sibling":sibling,"initialization":offered["offer"],
            "continuation":match std::env::var("TASK22_PRIVATE_CASE").as_deref(){Ok("preflight-resume")=>"resume",Ok("preflight-owner")=>"owner",Ok("preflight-legacy")=>"legacy",_=>"attach"}});
        let profile_path = root.join("control/genuine-no-send-profile.json");
        std::fs::write(&profile_path, serde_json::to_vec(&profile).unwrap()).unwrap();
        std::fs::set_permissions(&profile_path, std::fs::Permissions::from_mode(0o600)).unwrap();
        let mut command = std::process::Command::new(
            std::env::var_os("TASK22_INTEGRATION_TEST_BINARY")
                .expect("exact Cargo integration executable"),
        );
        configure_private_environment(&mut command, root, 0);
        command
            .args([
                "test_real_workspace_open_init_and_existing_world_no_send",
                "--ignored",
                "--exact",
                "--nocapture",
                "--test-threads=1",
            ])
            .env("SUBSTRATE_PREFLIGHT_PROOF_PROFILE", &profile_path);
        let output = command.output().unwrap();
        println!("{}", String::from_utf8_lossy(&output.stdout));
        assert!(
            output.status.success(),
            "genuine full native integration: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        println!("PF_PROFILE_ACTUAL_NATIVE_SOURCE_COMPLETE=private-source; installed qualification remains R17");
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    #[ignore = "requires the explicitly released private two-identity mount namespace"]
    async fn test_workspace_current_joined_private_native_service_publisher_and_lineage() {
        use world_api::WorldBackend;
        let root = root();
        let case = std::env::var("TASK22_PRIVATE_CASE").expect("explicit fixture population");
        let service = WorldService::new().unwrap();
        assert!(
            service.accepted_home_authority.is_none()
                && service.config_projection_service.is_none()
                && service.e3_projection_preparations.is_none(),
            "the new read must succeed later without E3 initialization/preparation readiness"
        );
        let spec = world_api::WorldSpec {
            project_dir: PathBuf::from("/run/task22-owner/workspace"),
            isolate_network: false,
            reuse_mode: world_api::WorldReuseMode::SharedOrchestration(
                world_api::SharedWorldOwnerSpec {
                    orchestration_session_id: "session-start-1".into(),
                    action: world_api::SharedWorldOwnerAction::AttachOrCreate,
                },
            ),
            ..world_api::WorldSpec::default()
        };
        // The fixture prepares only the empty directory selected by the existing
        // SessionWorld::default_shared_root_dir_for(XDG_RUNTIME_DIR) rule. Shared
        // ensure_session takes this existing root's ownership lock before allocation.
        let backend_root =
            PathBuf::from(std::env::var_os("XDG_RUNTIME_DIR").unwrap()).join("substrate/worlds");
        assert_eq!(
            backend_root,
            PathBuf::from("/run/task22-service-runtime/substrate/worlds")
        );
        let metadata_root = std::fs::metadata(&backend_root).unwrap();
        assert_eq!(metadata_root.uid(), 0);
        assert_eq!(metadata_root.mode() & 0o777, 0o700);
        assert!(
            std::fs::read_dir(&backend_root).unwrap().next().is_none(),
            "no handwritten custody/cache/World metadata"
        );
        // The real existing service backend invokes its real producer, before read measurement.
        let world = service.linux_backend.ensure_session(&spec).unwrap();
        assert_eq!(world.shared_binding.as_ref().unwrap().world_generation, 0);
        shell_driver(&root, "setup", &world, 1000);
        // Invoke only the existing genuine installed-home publisher function, in the private
        // namespace. No installer/provision/service action and no test record constructor.
        let mut producer_command = std::process::Command::new("/usr/bin/bash");
        configure_private_environment(&mut producer_command, &root, 0);
        let producer = producer_command
            .args([
                "--noprofile",
                "--norc",
                "-c",
                r#"
            set -euo pipefail
            source "$TASK22_PRODUCT/scripts/linux/world-lifecycle.sh"
            sudo_cmd() { "$@"; }
            INSTALL_BOOTSTRAP_CONTEXT_V1=$(cat "$TASK22_PRIVATE_ROOT/control/carrier.txt")
            INSTALL_BOOTSTRAP_COMMITMENT="$TASK22_CARRIER_COMMITMENT"
            INSTALL_BOOTSTRAP_ACCOUNT=task22-owner
            INSTALL_BOOTSTRAP_UID=1000
            INSTALL_BOOTSTRAP_PRIMARY_GID=1000
            INSTALL_PREFIX=/run/task22-owner/home
            unset FAKE_ROOT
            publish_installed_home_bootstrap_v1
        "#,
            ])
            .env("TASK22_CARRIER_COMMITMENT", {
                let request: Q = serde_json::from_slice(
                    &std::fs::read(root.join("control/request.json")).unwrap(),
                )
                .unwrap();
                request.native_context.host_context_commitment
            })
            .output()
            .unwrap();
        assert!(
            producer.status.success(),
            "genuine bootstrap publisher: {}",
            String::from_utf8_lossy(&producer.stderr)
        );
        if case.starts_with("preflight") {
            let mut proof =
                std::process::Command::new(std::env::var_os("TASK22_SHELL_TEST_BINARY").unwrap());
            configure_private_environment(&mut proof, &root, 0);
            let proof = proof.args([
                "execution::workspace_open::installed_authority_cut_private_tests::workspace_open_installed_authority_watch_genuine_publisher_and_restoration",
                "--ignored", "--exact", "--nocapture",
            ]).output().unwrap();
            println!("{}", String::from_utf8_lossy(&proof.stdout));
            assert!(
                proof.status.success(),
                "installed registry proof: {}",
                String::from_utf8_lossy(&proof.stderr)
            );
        }
        {
            use std::sync::atomic::{AtomicBool, Ordering};
            use tower::ServiceExt;
            let polled = Arc::new(AtomicBool::new(false));
            let observed = polled.clone();
            let body = futures_util::stream::once(async move {
                observed.store(true, Ordering::SeqCst);
                Ok::<_, std::io::Error>(hyper::body::Bytes::from_static(b"private-body"))
            });
            let response = crate::build_router(service.clone())
                .oneshot(
                    hyper::Request::post("/v1/workspace/current")
                        .body(hyper::Body::wrap_stream(body))
                        .unwrap(),
                )
                .await
                .unwrap();
            assert!(
                !polled.load(Ordering::SeqCst),
                "unauthenticated/TCP route read the body"
            );
            let value: R =
                serde_json::from_slice(&hyper::body::to_bytes(response.into_body()).await.unwrap())
                    .unwrap();
            assert!(matches!(
                value,
                R::Refused {
                    reason: transport_api_types::WorkspaceCurrentRefusalV1::Denied
                }
            ));
        }
        let shutdown = tokio_util::sync::CancellationToken::new();
        let proof_router = crate::build_router(service.clone()).layer(
            axum::middleware::from_fn_with_state(root.clone(), preflight_gate_v1),
        );
        let server = tokio::spawn(crate::run_uds_server(
            proof_router,
            PathBuf::from("/run/task22-current.sock"),
            None,
            None,
            shutdown.clone(),
        ));
        for _ in 0..100 {
            if PathBuf::from("/run/task22-current.sock").exists() {
                break;
            }
            tokio::time::sleep(std::time::Duration::from_millis(10)).await;
        }
        assert!(
            PathBuf::from("/run/task22-current.sock").exists(),
            "bounded fixture listener setup"
        );
        let publish: Q =
            serde_json::from_slice(&std::fs::read(root.join("control/request.json")).unwrap())
                .unwrap();
        std::fs::write(root.join("control/transport-request.json"), b"{}").unwrap();
        std::fs::set_permissions(
            root.join("control/transport-request.json"),
            std::fs::Permissions::from_mode(0o644),
        )
        .unwrap();
        if case == "publisher-failure" {
            use std::io::Write;
            let mut fill = std::fs::File::create(PathBuf::from(
                "/run/task22-owner/workspace/.substrate/.task22-capacity-fill",
            ))
            .unwrap();
            loop {
                if fill.write_all(&[0u8; 4096]).is_err() {
                    break;
                }
            }
            drop(fill);
            let before = snapshot(&root);
            let (success, response) = native(&root, "publish-association", &publish);
            assert!(!success);
            assert!(
                matches!(
                    response,
                    R::AssociationPublicationFailed {
                        effects: transport_api_types::WorkspaceAssociationEffectsV1 {
                            staging_created: Some(true),
                            published_visible: Some(false),
                            directory_sync_confirmed: Some(false)
                        },
                        ..
                    }
                ),
                "{response:?}"
            );
            assert_ne!(
                snapshot(&root),
                before,
                "failed publisher staging must remain visible"
            );
            let pending = PathBuf::from(
                "/run/task22-owner/workspace/.substrate/.workspace-association.v1.pending",
            );
            assert!(pending.exists());
            assert!(!PathBuf::from(
                "/run/task22-owner/workspace/.substrate/workspace-association.v1.json"
            )
            .exists());
            let mut read = publish.clone();
            read.operation = O::CurrentActorRead {
                lane_ref: "unpublished-negative-selector".into(),
                association_incarnation: "unpublished-negative-selector".into(),
                association_revision: 1,
                backend_custody_ref: "a".repeat(64),
                expected_actor: "participant-start-1".into(),
            };
            let before = snapshot(&root);
            assert!(!native(&root, "current-actor", &read).0);
            assert!(matches!(
                direct(&root, &world, &read, 1000),
                R::Refused { .. }
            ));
            assert!(!native(&root, "publish-association", &publish).0);
            assert_eq!(
                snapshot(&root),
                before,
                "pending state was cleaned/repaired"
            );
            shutdown.cancel();
            server.await.unwrap().unwrap();
            return;
        }
        let before_publish = snapshot(&root);
        let (success, publication) = native(&root, "publish-association", &publish);
        assert!(success, "publisher {publication:?}");
        assert_ne!(
            snapshot(&root),
            before_publish,
            "publication writes must be measured separately"
        );
        let assoc = std::fs::metadata(PathBuf::from(
            "/run/task22-owner/workspace/.substrate/workspace-association.v1.json",
        ))
        .unwrap();
        assert_eq!(assoc.uid(), 1000);
        assert_eq!(assoc.gid(), 0);
        assert_eq!(assoc.mode() & 0o7777, 0o600);
        let identity_bytes = std::fs::read(PathBuf::from(
            "/run/task22-owner/workspace/.substrate/workspace-state.v1.json",
        ))
        .unwrap();
        let mut request = read_request(&root, &publication);
        if case == "preflight"
            || case == "preflight-resume"
            || case == "preflight-owner"
            || case == "preflight-legacy"
            || case == "preflight-custody"
        {
            run_preflight_profile_v1(&service, &root, &world, &spec);
            shutdown.cancel();
            server.await.unwrap().unwrap();
            return;
        }
        let modes: &[&str] = match case.as_str() {
            "attach" => &[
                "initial",
                "registered",
                "retained",
                "settled",
                "attach",
                "detach",
                "attach",
                "retained",
            ],
            "resume" => &[
                "initial",
                "registered",
                "retained",
                "settled",
                "resume",
                "retained",
            ],
            // Genuine registered Start promotes to V3, preserves an active lineage
            // leaf and has no trailing retained participant: supported Stop input.
            "stop" => &["initial", "registered"],
            "custody-pending" | "custody-corrupt" | "custody-substituted" | "custody-revision" => {
                &["initial"]
            }
            _ => panic!("unsupported fixture population"),
        };
        for &mode in modes {
            if mode != "initial" {
                shell_driver(&root, mode, &world, 1000);
                request = read_request(&root, &publication);
            }
            if mode == "detach" {
                // Startup reconciliation preserves an active authoritative member;
                // current-actor disclosure does not assert runtime liveness.
                let before = snapshot(&root);
                let parity = shell_driver(&root, "parity-detached", &world, 1000);
                let output = String::from_utf8(parity.stdout).unwrap();
                let witness = output
                    .lines()
                    .find(|line| line.starts_with("TASK22_DETACHED_PARITY="))
                    .expect("actual detached default/new-reader parity witness");
                println!("{witness}");
                assert_eq!(snapshot(&root), before, "detached reader parity effects");
            }
            let before = snapshot(&root);
            let (success, response) = native(&root, "current-actor", &request);
            assert!(
                success && matches!(response, R::CurrentActor(..)),
                "mode {mode}: {response:?}"
            );
            assert_eq!(snapshot(&root), before, "mode {mode}: read effects");
            assert_eq!(
                std::fs::read(PathBuf::from(
                    "/run/task22-owner/workspace/.substrate/workspace-state.v1.json"
                ))
                .unwrap(),
                identity_bytes
            );
        }
        // Exact selectors are expectations, never fallback selectors or authority.
        for mutation in 0..16 {
            let mut wrong = request.clone();
            match mutation {
                0 => wrong.resource.world_generation += 1,
                1 => wrong.resource.world_id.push('x'),
                2 => wrong.resource.orchestration_session_id.push('x'),
                3 => wrong.resource.backend_id.push('x'),
                4 => wrong.resource.workspace_ref.push('x'),
                5 => wrong.resource.workspace_incarnation.push('x'),
                6 => wrong.observation.authority_store_id.push('x'),
                7 => wrong.observation.home_inode += 1,
                8 => wrong.observation.root_revision += 1,
                9 => {
                    if let O::CurrentActorRead { expected_actor, .. } = &mut wrong.operation {
                        expected_actor.push('x');
                    }
                }
                10 => {
                    if let O::CurrentActorRead {
                        association_revision,
                        ..
                    } = &mut wrong.operation
                    {
                        *association_revision += 1;
                    }
                }
                11 => {
                    if let O::CurrentActorRead {
                        backend_custody_ref,
                        ..
                    } = &mut wrong.operation
                    {
                        *backend_custody_ref = "f".repeat(64);
                    }
                }
                12 => wrong.observation.home_path.push('x'),
                13 => wrong.resource.workspace_path.push('x'),
                14 => {
                    if let O::CurrentActorRead { lane_ref, .. } = &mut wrong.operation {
                        lane_ref.push('x');
                    }
                }
                _ => {
                    if let O::CurrentActorRead {
                        association_incarnation,
                        ..
                    } = &mut wrong.operation
                    {
                        association_incarnation.push('x');
                    }
                }
            }
            let before = snapshot(&root);
            assert!(
                !native(&root, "current-actor", &wrong).0,
                "wrong selector {mutation}"
            );
            assert!(
                matches!(direct(&root, &world, &wrong, 1000), R::Refused { .. }),
                "independent service wrong selector {mutation}"
            );
            assert_eq!(
                snapshot(&root),
                before,
                "wrong selector {mutation}: read effects"
            );
        }

        {
            // These are negative target fixtures, not claimed positive authority records.
            let foreign = PathBuf::from("/run/task22-foreign-owner-workspace");
            std::fs::create_dir_all(foreign.join(".substrate")).unwrap();
            std::fs::write(foreign.join(".substrate/workspace.yaml"), b"{}\n").unwrap();
            std::fs::write(
                foreign.join(".substrate/workspace-state.v1.json"),
                b"negative wrong-owner record",
            )
            .unwrap();
            let missing_owner = PathBuf::from("/run/task22-missing-owner-workspace");
            std::fs::create_dir_all(missing_owner.join(".substrate")).unwrap();
            std::fs::write(missing_owner.join(".substrate/workspace.yaml"), b"{}\n").unwrap();
            for path in [&missing_owner, &missing_owner.join(".substrate")] {
                let name = std::ffi::CString::new(path.to_str().unwrap()).unwrap();
                assert_eq!(unsafe { libc::chown(name.as_ptr(), 1000, 1000) }, 0);
                std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o700)).unwrap();
            }
            let mut responses = Vec::new();
            for path in [
                foreign,
                missing_owner,
                PathBuf::from("/run/task22-nonexistent-workspace"),
            ] {
                let mut wrong = request.clone();
                wrong.resource.workspace_path = path.to_str().unwrap().into();
                let before = snapshot(&root);
                responses.push(direct(&root, &world, &wrong, 1000));
                assert_eq!(
                    snapshot(&root),
                    before,
                    "unestablished target denial effects"
                );
            }
            assert!(
                responses.iter().all(|reply| reply
                    == &R::Refused {
                        reason: transport_api_types::WorkspaceCurrentRefusalV1::Denied
                    }),
                "global allow disclosed owner/target existence: {responses:?}"
            );
        }
        let policy = PathBuf::from("/run/task22-owner/home/policy.yaml");
        let saved_policy = std::fs::read(&policy).unwrap();
        // Current permission, independent mutation permission and existing Workspace precedence.
        let workspace_policy = PathBuf::from("/run/task22-owner/workspace/.substrate/policy.yaml");
        // This new policy source must be genuinely created by its native owner before
        // either boundary reads it; Root-only0600 input would only test unreadability.
        let mut policy_writer = std::process::Command::new("/usr/bin/setpriv");
        configure_private_environment(&mut policy_writer, &root, 1000);
        let created_policy = policy_writer
            .args(["--reuid=1000", "--regid=1000", "--clear-groups", "/usr/bin/python3", "-I", "-B", "-c",
                "import pathlib,sys\nwith pathlib.Path(sys.argv[1]).open('x', encoding='utf-8') as output:\n    output.write(sys.argv[2])"])
            .arg(&workspace_policy)
            .arg("workspace_authority: {current_actor_read: deny, association_publish: deny}\n")
            .output()
            .unwrap();
        assert!(
            created_policy.status.success(),
            "private native policy producer: {}",
            String::from_utf8_lossy(&created_policy.stderr)
        );
        let policy_metadata = std::fs::metadata(&workspace_policy).unwrap();
        assert_eq!(policy_metadata.uid(), 1000);
        assert_eq!(policy_metadata.gid(), 1000);
        assert_eq!(policy_metadata.mode() & 0o777, 0o600);
        let before = snapshot(&root);
        assert!(!native(&root, "current-actor", &request).0);
        assert!(matches!(
            direct(&root, &world, &request, 1000),
            R::Refused { .. }
        ));
        assert!(!native(&root, "publish-association", &publish).0);
        assert_eq!(snapshot(&root), before, "Workspace deny precedence effects");
        std::fs::write(
            &policy,
            "workspace_authority: {current_actor_read: deny, association_publish: deny}\n",
        )
        .unwrap();
        std::fs::write(
            &workspace_policy,
            "workspace_authority: {current_actor_read: allow}\n",
        )
        .unwrap();
        assert!(
            native(&root, "current-actor", &request).0,
            "Workspace read allow replaces global deny"
        );
        assert!(
            !native(&root, "publish-association", &publish).0,
            "read allow is not mutation permission"
        );
        std::fs::remove_file(&workspace_policy).unwrap();
        assert!(
            !native(&root, "current-actor", &request).0,
            "removing effective Workspace allow revokes"
        );
        std::fs::write(&policy, &saved_policy).unwrap();
        for bytes in [
            b"{}\n".as_slice(),
            b"workspace_authority: {current_actor_read: deny, association_publish: deny}\n",
        ] {
            std::fs::write(&policy, bytes).unwrap();
            let before = snapshot(&root);
            let native_denied = native(&root, "current-actor", &request).1;
            assert!(matches!(
                native_denied,
                R::Refused {
                    reason: transport_api_types::WorkspaceCurrentRefusalV1::Denied
                }
            ));
            assert!(
                !native(&root, "publish-association", &publish).0,
                "mutation absence/deny"
            );
            assert!(matches!(
                direct(&root, &world, &publish, 1000),
                R::Refused {
                    reason: transport_api_types::WorkspaceCurrentRefusalV1::Denied
                }
            ));
            let service_denied = direct(&root, &world, &request, 1000);
            assert_eq!(
                service_denied, native_denied,
                "independent service policy decision"
            );
            assert_eq!(snapshot(&root), before, "permission denial read effects");
        }
        std::fs::write(&policy, &saved_policy).unwrap();
        assert!(
            native(&root, "current-actor", &request).0,
            "freshly restored private policy only"
        );
        let before = snapshot(&root);
        assert!(matches!(
            direct(&root, &world, &request, 0),
            R::Refused {
                reason: transport_api_types::WorkspaceCurrentRefusalV1::Denied
            }
        ));
        assert_eq!(snapshot(&root), before, "wrong peer read effects");

        {
            let backend = service.linux_backend.clone();
            let resource = request.resource.clone();
            let (entered_tx, entered_rx) = std::sync::mpsc::channel();
            let (release_tx, release_rx) = std::sync::mpsc::channel();
            let holder = std::thread::spawn(move || {
                backend
                    .with_existing_custody_nonreconciling_v1(
                        &resource.orchestration_session_id,
                        &resource.world_id,
                        resource.world_generation,
                        Path::new(&resource.workspace_path),
                        |_| {
                            entered_tx.send(()).unwrap();
                            release_rx.recv().unwrap();
                            Ok(())
                        },
                    )
                    .unwrap()
            });
            entered_rx
                .recv_timeout(std::time::Duration::from_secs(2))
                .unwrap();
            let fd = unsafe { libc::inotify_init1(libc::IN_CLOEXEC | libc::IN_NONBLOCK) };
            assert!(fd >= 0);
            let name = std::ffi::CString::new(policy.to_str().unwrap()).unwrap();
            assert!(
                unsafe {
                    libc::inotify_add_watch(
                        fd,
                        name.as_ptr(),
                        libc::IN_OPEN | libc::IN_CLOSE_NOWRITE,
                    )
                } >= 0
            );
            let direct_root = root.clone();
            let direct_world = world.clone();
            let direct_request = request.clone();
            let call = tokio::task::spawn_blocking(move || {
                direct_mode(
                    &direct_root,
                    &direct_world,
                    &direct_request,
                    1000,
                    "transport-held",
                )
            });
            let witness = root.join("control/transport-caller.pid");
            for _ in 0..200 {
                if witness.exists() {
                    break;
                }
                tokio::time::sleep(std::time::Duration::from_millis(10)).await;
            }
            assert!(witness.exists(), "actual Child handle PID witness required");
            let mut opened = 0;
            for _ in 0..200 {
                let mut buffer = [0u8; 4096];
                let count = unsafe { libc::read(fd, buffer.as_mut_ptr().cast(), buffer.len()) };
                if count > 0 {
                    let mut cursor = 0;
                    while cursor + std::mem::size_of::<libc::inotify_event>() <= count as usize {
                        let event = unsafe {
                            std::ptr::read_unaligned(
                                buffer.as_ptr().add(cursor).cast::<libc::inotify_event>(),
                            )
                        };
                        if event.mask & libc::IN_OPEN != 0 {
                            opened += 1;
                        }
                        cursor += std::mem::size_of::<libc::inotify_event>() + event.len as usize;
                    }
                }
                if opened >= 3 {
                    break;
                }
                tokio::time::sleep(std::time::Duration::from_millis(10)).await;
            }
            assert!(opened >= 3, "service did not reach held policy capture");
            std::fs::write(
                &policy,
                "workspace_authority: {current_actor_read: deny, association_publish: allow}\n",
            )
            .unwrap();
            let caller_pid: u32 =
                std::fs::read_to_string(root.join("control/transport-caller.pid"))
                    .unwrap()
                    .parse()
                    .unwrap();
            let before = snapshot(&root);
            release_tx.send(()).unwrap();
            holder.join().unwrap();
            let completed = root.join("control/transport-completed");
            for _ in 0..200 {
                if completed.exists() {
                    break;
                }
                tokio::time::sleep(std::time::Duration::from_millis(10)).await;
            }
            let returned = completed.exists();
            let after = snapshot(&root);
            // Teardown follows the after-snapshot, even when an assertion will fail.
            std::fs::write(root.join("control/transport-release"), b"release").unwrap();
            let response = call.await.unwrap();
            assert!(
                returned,
                "actual client method did not return before measurement"
            );
            assert!(
                before.children.contains(&caller_pid) && after.children.contains(&caller_pid),
                "witnessed actual harness caller must remain alive in both snapshots"
            );
            assert!(
                matches!(response, R::Refused { .. }),
                "policy changed within read interval yielded actor"
            );
            assert_eq!(after, before, "interval refusal effects");
            assert_eq!(unsafe { libc::close(fd) }, 0);
            std::fs::write(&policy, &saved_policy).unwrap();
        }
        let association_path =
            PathBuf::from("/run/task22-owner/workspace/.substrate/workspace-association.v1.json");
        let association_bytes = std::fs::read(&association_path).unwrap();
        let pending_path = PathBuf::from(
            "/run/task22-owner/workspace/.substrate/.workspace-association.v1.pending",
        );
        std::fs::write(&pending_path, b"partial-negative-fixture").unwrap();
        let before = snapshot(&root);
        assert!(!native(&root, "current-actor", &request).0);
        assert!(matches!(
            direct(&root, &world, &request, 1000),
            R::Refused { .. }
        ));
        assert_eq!(
            snapshot(&root),
            before,
            "pending association repaired during read"
        );
        std::fs::remove_file(&pending_path).unwrap();
        std::fs::write(&association_path, b"{corrupt-negative-fixture").unwrap();
        let before = snapshot(&root);
        assert!(!native(&root, "current-actor", &request).0);
        assert!(matches!(
            direct(&root, &world, &request, 1000),
            R::Refused { .. }
        ));
        assert_eq!(
            snapshot(&root),
            before,
            "corrupt association repaired during read"
        );
        std::fs::write(&association_path, &association_bytes).unwrap();
        let held_association = association_path.with_extension("retained-negative-fixture");
        std::fs::rename(&association_path, &held_association).unwrap();
        let original_association = std::fs::symlink_metadata(&held_association).unwrap();
        {
            use std::io::Write;
            use std::os::fd::AsRawFd;
            use std::os::unix::fs::OpenOptionsExt;
            // Only this newly created negative replacement changes ownership.
            let mut replacement = std::fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .mode(0o600)
                .open(&association_path)
                .unwrap();
            assert_eq!(unsafe { libc::fchown(replacement.as_raw_fd(), 1000, 0) }, 0);
            replacement.write_all(&association_bytes).unwrap();
        }
        let substituted_association = std::fs::symlink_metadata(&association_path).unwrap();
        assert!(original_association.is_file() && substituted_association.is_file());
        assert_eq!(
            (
                original_association.uid(),
                original_association.gid(),
                original_association.mode() & 0o7777
            ),
            (1000, 0, 0o600)
        );
        assert_eq!(substituted_association.dev(), original_association.dev());
        assert_ne!(substituted_association.ino(), original_association.ino());
        assert_eq!(
            (
                substituted_association.uid(),
                substituted_association.gid(),
                substituted_association.mode() & 0o7777
            ),
            (1000, 0, 0o600)
        );
        assert_eq!(
            (
                original_association.nlink(),
                substituted_association.nlink()
            ),
            (1, 1)
        );
        assert_eq!(substituted_association.len(), original_association.len());
        // Every serialized identity/binding selector is still byte-for-byte genuine.
        assert_eq!(std::fs::read(&association_path).unwrap(), association_bytes);
        let unchanged_record: serde_json::Value =
            serde_json::from_slice(&association_bytes).unwrap();
        assert_eq!(
            unchanged_record["record_identity"]["device_id"].as_u64(),
            Some(original_association.dev())
        );
        assert_eq!(
            unchanged_record["record_identity"]["inode"].as_u64(),
            Some(original_association.ino())
        );
        assert_ne!(
            unchanged_record["record_identity"]["inode"].as_u64(),
            Some(substituted_association.ino())
        );
        let before = snapshot(&root);
        assert!(!native(&root, "current-actor", &request).0);
        assert!(matches!(
            direct(&root, &world, &request, 1000),
            R::Refused { .. }
        ));
        assert_eq!(
            snapshot(&root),
            before,
            "substituted association adopted/repaired"
        );
        std::fs::remove_file(&association_path).unwrap();
        std::fs::rename(&held_association, &association_path).unwrap();

        if case.starts_with("custody-") {
            let metadata = PathBuf::from(std::env::var_os("XDG_RUNTIME_DIR").unwrap())
                .join("substrate/worlds")
                .join(&world.id)
                .join("session.json");
            match case.as_str() {
                "custody-pending" => {
                    std::fs::write(
                        metadata.with_file_name(".session.json.pending-negative.tmp"),
                        b"partial",
                    )
                    .unwrap();
                }
                "custody-corrupt" => {
                    std::fs::write(&metadata, b"{corrupt-custody").unwrap();
                }
                "custody-substituted" => {
                    let bytes = std::fs::read(&metadata).unwrap();
                    let original = std::fs::symlink_metadata(&metadata).unwrap();
                    // Same private /run mount, OUTSIDE the target World directory:
                    // an extra entry there would mask the inode-binding check.
                    let preserved = metadata
                        .parent()
                        .unwrap()
                        .parent()
                        .unwrap()
                        .join(format!(".task22-custody-negative-preimage-{}", world.id));
                    assert!(!preserved.exists());
                    std::fs::rename(&metadata, &preserved).unwrap();
                    std::fs::write(&metadata, &bytes).unwrap();
                    let substituted = std::fs::symlink_metadata(&metadata).unwrap();
                    assert_eq!(substituted.dev(), original.dev());
                    assert_ne!(substituted.ino(), original.ino());
                    assert_eq!(
                        (
                            substituted.uid(),
                            substituted.gid(),
                            substituted.mode() & 0o7777
                        ),
                        (original.uid(), original.gid(), original.mode() & 0o7777)
                    );
                    assert_eq!((original.nlink(), substituted.nlink()), (1, 1));
                    assert_eq!(std::fs::read(&metadata).unwrap(), bytes);
                    let target_entries: Vec<_> = std::fs::read_dir(metadata.parent().unwrap())
                        .unwrap()
                        .map(|entry| entry.unwrap().file_name())
                        .collect();
                    assert_eq!(
                        target_entries,
                        vec![std::ffi::OsString::from("session.json")]
                    );
                }
                "custody-revision" => {
                    let next = world_api::WorldSpec {
                        reuse_mode: world_api::WorldReuseMode::SharedOrchestration(
                            world_api::SharedWorldOwnerSpec {
                                orchestration_session_id: "session-start-1".into(),
                                action:
                                    world_api::SharedWorldOwnerAction::ReplaceExpectedGeneration {
                                        expected_generation: 0,
                                        reason: "explicit private producer invalidation".into(),
                                    },
                            },
                        ),
                        ..spec.clone()
                    };
                    let produced = service.linux_backend.ensure_session(&next).unwrap();
                    assert_eq!(
                        produced.shared_binding.as_ref().unwrap().world_generation,
                        1
                    );
                }
                _ => unreachable!(),
            }
            let before = snapshot(&root);
            assert!(!native(&root, "current-actor", &request).0);
            assert!(matches!(
                direct(&root, &world, &request, 1000),
                R::Refused { .. }
            ));
            assert_eq!(
                snapshot(&root),
                before,
                "stale/pending/corrupt/substituted custody repaired or executed"
            );
            shutdown.cancel();
            server.await.unwrap().unwrap();
            return;
        }
        if case == "attach" || case == "resume" {
            // This genuine state has a trailing retained participant, so the existing
            // Stop producer refuses it. Keep that support ceiling and every read.
            let before = snapshot(&root);
            let refusal = shell_driver(&root, "stop-retained-refusal", &world, 1000);
            let refusal_output = String::from_utf8(refusal.stdout).unwrap();
            let ceiling = refusal_output
                .lines()
                .find(|line| line.starts_with("TASK22_STOP_SUPPORT_CEILING="))
                .expect("exact genuine unsupported Stop refusal witness");
            println!("{ceiling}");
            assert_eq!(snapshot(&root), before, "unsupported Stop issuance effects");
            shutdown.cancel();
            server.await.unwrap().unwrap();
            return;
        }
        assert_eq!(
            case, "stop",
            "supported active-leaf Stop is a separate fixture"
        );
        shell_driver(&root, "stop", &world, 1000);
        let stopped = read_request(&root, &publication);
        let before = snapshot(&root);
        assert!(
            !native(&root, "current-actor", &stopped).0,
            "Stop must never revive a prior actor"
        );
        assert!(
            matches!(direct(&root, &world, &stopped, 1000), R::Refused { .. }),
            "direct service Stop must never revive a prior actor"
        );
        assert_eq!(snapshot(&root), before, "Stop refusal effects");
        shutdown.cancel();
        server.await.unwrap().unwrap();
    }
}

#[cfg(target_os = "linux")]
impl WorldService {
    pub(crate) fn workspace_preflight_v1(
        &self,
        peer: &substrate_shell::WorkspaceNativePeerV1,
        request: &transport_api_types::WorkspaceCurrentRequestV1,
    ) -> transport_api_types::WorkspacePreflightResponseV1 {
        substrate_shell::workspace_preflight_for_peer_v1(peer, request, &self.linux_backend)
    }
    pub(crate) fn workspace_association_update_v1(
        &self,
        peer: &substrate_shell::WorkspaceNativePeerV1,
        request: &transport_api_types::WorkspaceAssociationUpdateV1,
    ) -> transport_api_types::WorkspaceAssociationUpdateResponseV1 {
        substrate_shell::workspace_association_update_for_peer_v1(
            peer,
            request,
            &self.linux_backend,
        )
    }
}
