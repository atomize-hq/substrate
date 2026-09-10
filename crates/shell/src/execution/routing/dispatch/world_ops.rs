//! World and agent routing helpers.

use super::shim_ops::build_world_env_map_for_cwd;
use crate::execution::agent_events::publish_agent_event;
use crate::execution::agent_events::ShellCommandEventContext;
#[cfg(target_os = "windows")]
use crate::execution::policy_snapshot::world_spec_for_network_policy;
use crate::execution::policy_snapshot::{
    request_world_network_routing, resolve_world_network_policy_for_cwd,
    resolve_world_network_policy_for_snapshot,
};
#[cfg(target_os = "macos")]
use crate::execution::pw;
#[cfg(target_os = "linux")]
use crate::execution::routing::{get_term_size, RawModeGuard};
#[cfg(all(test, any(target_os = "linux", target_os = "windows")))]
use crate::execution::world_env_guard;
#[cfg(target_os = "linux")]
use crate::execution::{policy_snapshot::bootstrap_world_spec, socket_activation};
use base64::engine::general_purpose::STANDARD as BASE64;
use base64::Engine;
use std::env;
use std::io;
use substrate_broker::world_fs_mode;
use substrate_common::agent_events::AgentEvent;
use substrate_common::WorldRootMode;
#[cfg(target_os = "linux")]
use tokio::net::UnixStream;
#[cfg(target_os = "linux")]
use tokio::signal::unix::{signal, SignalKind};
#[cfg(any(target_os = "linux", target_os = "macos"))]
use tokio_tungstenite as tungs;
#[cfg(any(target_os = "linux", target_os = "macos"))]
use transport_api_client::AgentClient;
#[cfg(not(target_os = "windows"))]
use transport_api_types::ExecuteCancelRequestV1;
#[cfg(any(target_os = "linux", all(test, unix)))]
use transport_api_types::PlatformPrincipalV1;
use transport_api_types::{
    ConfigProjectionActivationCarrierV1, E2MemberLaunchActivationCarrierV1, ExecuteRequest,
    ExecuteStreamFrame, MemberDispatchRequest, MemberDispatchRequestV1, MemberDispatchRequestV2,
    MemberRuntimeBackendKindV1, ProcessTelemetry, ResolvedMemberRuntimeDescriptorV1,
    RetainedWorkerLaunchAuthorityProofV1, WorldFsMode,
};
#[cfg(target_os = "linux")]
use world::LinuxLocalBackend;
#[cfg(target_os = "linux")]
use world_api::WorldBackend;

#[cfg_attr(not(target_os = "linux"), allow(dead_code))]
const WORLD_PROJECT_DIR_OVERRIDE_ENV: &str = "SUBSTRATE_WORLD_PROJECT_DIR";
const MACOS_STAGED_WORKSPACE_CURRENT: &str = "/var/lib/substrate/staged-workspace/current";
const SUBSTRATE_PARENT_SPAN_ENV: &str = "SUBSTRATE_PARENT_SPAN_ID";
#[cfg(any(target_os = "linux", all(test, unix)))]
const SUBSTRATE_INTERNAL_CODEX_AUTH_SEED_HOME_ENV: &str = "SUBSTRATE_INTERNAL_CODEX_AUTH_SEED_HOME";
const RESERVED_WORLD_REQUEST_PROFILES: &[&str] = &["world-deps-provision", "world-deps-probe"];
#[cfg(target_os = "linux")]
const AUTHENTICATED_WORLD_SOCKET: &str = "/run/substrate.sock";
#[cfg(target_os = "linux")]
const AUTHENTICATED_WORLD_SERVICE_BINARY: &str = "/usr/local/bin/substrate-world-service";
#[cfg(target_os = "linux")]
const AUTHENTICATED_WORLD_SERVICE_CWD: &str = "/var/lib/substrate";
#[cfg(target_os = "linux")]
const AUTHENTICATED_WORLD_SERVICE_PATH: &str =
    "/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin";
#[cfg(target_os = "linux")]
const AUTHENTICATED_SYSTEMCTL_BINARY: &str = "/usr/bin/systemctl";
#[cfg(target_os = "linux")]
const AUTHENTICATED_WORLD_SOCKET_UNIT: &str = "substrate-world-service.socket";
#[cfg(target_os = "linux")]
const AUTHENTICATED_WORLD_SERVICE_UNIT: &str = "substrate-world-service.service";
#[cfg(target_os = "linux")]
const WORLD_SERVICE_PROBE_IO_TIMEOUT_MS: u64 = 150;
#[cfg(target_os = "linux")]
const WORLD_SERVICE_ACTIVATION_POLL_MS: u64 = 100;
#[cfg(target_os = "linux")]
const WORLD_SERVICE_ACTIVATION_WAIT_MS: u64 = 2_000;
#[cfg(target_os = "linux")]
const WORLD_SERVICE_SYSTEMCTL_SHOW_TIMEOUT_MS: u64 = 2_000;
#[cfg(target_os = "linux")]
const WORLD_SERVICE_SYSTEMCTL_POLL_MS: u64 = 10;
#[cfg(target_os = "linux")]
const WORLD_SERVICE_READINESS_POLL_MS: u64 = 50;
#[cfg(target_os = "linux")]
const WORLD_SERVICE_READINESS_WAIT_MS: u64 = 1_000;
#[cfg(target_os = "linux")]
type WorldServiceCandidateBins = [Option<String>; 4];

#[cfg(target_os = "linux")]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum WorldServiceSpawnEnvironment {
    LegacyCompatibility,
    InstalledLinuxProduct,
}

#[cfg(target_os = "linux")]
#[derive(Debug, PartialEq, Eq)]
struct WorldServiceSpawnPlan {
    candidate_bins: WorldServiceCandidateBins,
    environment: WorldServiceSpawnEnvironment,
}

#[cfg(target_os = "linux")]
#[derive(Debug)]
enum WorldServiceReadinessPosture {
    LegacyCompatibility {
        socket_override_active: bool,
        candidate_bins: fn() -> WorldServiceCandidateBins,
        activation_mode: fn() -> socket_activation::SocketActivationMode,
    },
    InstalledLinuxProduct {
        activation_mode: fn() -> socket_activation::SocketActivationMode,
    },
}

#[cfg(target_os = "linux")]
impl WorldServiceReadinessPosture {
    fn activation_mode(&self) -> socket_activation::SocketActivationMode {
        match self {
            Self::LegacyCompatibility {
                activation_mode, ..
            } => activation_mode(),
            Self::InstalledLinuxProduct { activation_mode } => activation_mode(),
        }
    }

    fn into_spawn_plan(
        self,
        socket_path: &std::path::Path,
    ) -> anyhow::Result<WorldServiceSpawnPlan> {
        match self {
            Self::LegacyCompatibility {
                socket_override_active,
                candidate_bins,
                ..
            } => {
                if socket_override_active {
                    anyhow::bail!(
                        "world backend unavailable (SUBSTRATE_WORLD_SOCKET override): {} did not respond",
                        socket_path.display()
                    );
                }
                Ok(WorldServiceSpawnPlan {
                    candidate_bins: candidate_bins(),
                    environment: WorldServiceSpawnEnvironment::LegacyCompatibility,
                })
            }
            Self::InstalledLinuxProduct { .. } => Ok(WorldServiceSpawnPlan {
                candidate_bins: [
                    Some(AUTHENTICATED_WORLD_SERVICE_BINARY.to_string()),
                    None,
                    None,
                    None,
                ],
                environment: WorldServiceSpawnEnvironment::InstalledLinuxProduct,
            }),
        }
    }
}

#[cfg(target_os = "linux")]
fn resolve_legacy_compatibility_activation_mode() -> socket_activation::SocketActivationMode {
    socket_activation::socket_activation_report().mode
}

#[cfg(target_os = "linux")]
fn classify_installed_product_activation_mode(
    socket_active_state: Option<&str>,
    observation_failed: bool,
    socket_exists: bool,
) -> socket_activation::SocketActivationMode {
    match socket_active_state {
        Some("active" | "listening" | "running" | "activating") => {
            socket_activation::SocketActivationMode::SocketActivation
        }
        Some(_) => socket_activation::SocketActivationMode::Unknown,
        None if observation_failed && socket_exists => {
            socket_activation::SocketActivationMode::Unknown
        }
        None => socket_activation::SocketActivationMode::Manual,
    }
}

#[cfg(target_os = "linux")]
fn terminate_installed_product_observer_child(child: &mut std::process::Child) {
    let _ = child.kill();
    let _ = child.wait();
}

#[cfg(target_os = "linux")]
fn observe_installed_product_unit_active_state(unit: &'static str) -> Result<Option<String>, ()> {
    use std::io::Read;
    use std::time::{Duration, Instant};

    let mut child = std::process::Command::new(AUTHENTICATED_SYSTEMCTL_BINARY)
        .arg("--no-pager")
        .arg("show")
        .arg(unit)
        .arg("--property=ActiveState")
        .arg("--property=UnitFileState")
        .arg("--property=Listen")
        .env_clear()
        .current_dir("/")
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .map_err(|_| ())?;

    let started = Instant::now();
    loop {
        match child.try_wait() {
            Ok(Some(status)) => {
                let mut stdout = Vec::new();
                let mut stderr = Vec::new();
                if let Some(mut output) = child.stdout.take() {
                    let _ = output.read_to_end(&mut stdout);
                }
                if let Some(mut output) = child.stderr.take() {
                    let _ = output.read_to_end(&mut stderr);
                }
                if !status.success() {
                    return if String::from_utf8_lossy(&stderr).contains("could not be found") {
                        Ok(None)
                    } else {
                        Err(())
                    };
                }

                let active_state = String::from_utf8_lossy(&stdout)
                    .lines()
                    .find_map(|line| line.strip_prefix("ActiveState="))
                    .map(str::trim)
                    .filter(|value| !value.is_empty())
                    .unwrap_or("unknown")
                    .to_string();
                return Ok(Some(active_state));
            }
            Ok(None)
                if started.elapsed()
                    <= Duration::from_millis(WORLD_SERVICE_SYSTEMCTL_SHOW_TIMEOUT_MS) =>
            {
                std::thread::sleep(Duration::from_millis(WORLD_SERVICE_SYSTEMCTL_POLL_MS));
            }
            Ok(None) => {
                terminate_installed_product_observer_child(&mut child);
                return Err(());
            }
            Err(_) => {
                terminate_installed_product_observer_child(&mut child);
                return Err(());
            }
        }
    }
}

#[cfg(target_os = "linux")]
fn resolve_installed_product_activation_mode() -> socket_activation::SocketActivationMode {
    let socket_unit = observe_installed_product_unit_active_state(AUTHENTICATED_WORLD_SOCKET_UNIT);
    let service_unit =
        observe_installed_product_unit_active_state(AUTHENTICATED_WORLD_SERVICE_UNIT);
    let observation_failed = socket_unit.is_err() || service_unit.is_err();
    let socket_active_state = socket_unit.ok().flatten();

    classify_installed_product_activation_mode(
        socket_active_state.as_deref(),
        observation_failed,
        std::path::Path::new(AUTHENTICATED_WORLD_SOCKET).exists(),
    )
}

#[cfg(target_os = "linux")]
fn resolve_legacy_compatibility_candidate_bins() -> WorldServiceCandidateBins {
    [
        std::env::var("SUBSTRATE_WORLD_AGENT_BIN").ok(),
        which::which("substrate-world-service")
            .ok()
            .map(|path| path.display().to_string()),
        Some("target/release/world-service".to_string()),
        Some("target/debug/world-service".to_string()),
    ]
}

fn inject_process_trace_env(
    env_map: &mut std::collections::HashMap<String, String>,
    parent_span_id: Option<&str>,
    parent_cmd_id: Option<&str>,
) {
    if let Ok(session_id) = std::env::var("SHIM_SESSION_ID") {
        if !session_id.is_empty() {
            env_map.insert("SHIM_SESSION_ID".to_string(), session_id);
        }
    }
    if let Some(span_id) = parent_span_id {
        env_map.insert(SUBSTRATE_PARENT_SPAN_ENV.to_string(), span_id.to_string());
    }
    if let Some(cmd_id) = parent_cmd_id {
        env_map.insert("SHIM_PARENT_CMD_ID".to_string(), cmd_id.to_string());
    }
}

#[cfg(any(target_os = "macos", target_os = "windows"))]
pub(super) fn normalize_env_for_linux_guest(
    env_map: &mut std::collections::HashMap<String, String>,
) {
    // macOS host PATH often contains directories that are mounted into the guest (e.g. /Users/...),
    // which can lead to confusing behavior where `which node` points at a macOS binary that cannot
    // run inside the Linux VM. Prefer a stable Linux guest PATH.
    const GUEST_BASE_PATH: &str =
        "/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin:/usr/games:/usr/local/games";
    const WORLD_DEPS_BIN: &str = "/var/lib/substrate/world-deps/bin";
    let world_deps_bin = env_map
        .get("SUBSTRATE_WORLD_DEPS_GUEST_BIN_DIR")
        .map(|v| v.trim().to_string())
        .filter(|v| !v.is_empty())
        .unwrap_or_else(|| WORLD_DEPS_BIN.to_string());
    env_map.insert(
        "SUBSTRATE_WORLD_DEPS_GUEST_BIN_DIR".to_string(),
        world_deps_bin.clone(),
    );
    let world_deps_bin_str = world_deps_bin.as_str();
    let current_path = env_map.get("PATH").map(String::as_str).unwrap_or("");
    // If the caller already provided a Linux-ish PATH (common in tests/fixtures and advanced
    // setups), don't clobber it; just ensure the world-deps bin is present.
    if current_path.contains(GUEST_BASE_PATH) {
        let has_world_deps_bin = current_path
            .split(':')
            .any(|segment| segment.trim_end_matches('/') == world_deps_bin_str);
        if !has_world_deps_bin {
            if current_path.trim().is_empty() {
                env_map.insert("PATH".to_string(), world_deps_bin.clone());
            } else {
                env_map.insert(
                    "PATH".to_string(),
                    format!("{world_deps_bin}:{current_path}"),
                );
            }
        }
    } else {
        env_map.insert(
            "PATH".to_string(),
            format!("{world_deps_bin_str}:{GUEST_BASE_PATH}"),
        );
    }

    // Avoid leaking host HOME into the Linux guest. This both reduces accidental use of host
    // toolchains and keeps guest-only state in a predictable location.
    if env_map.get("HOME").is_none_or(|home| {
        home.is_empty()
            || !home.starts_with('/')
            || (cfg!(target_os = "macos") && home.starts_with("/Users/"))
    }) {
        env_map.insert("HOME".to_string(), "/root".to_string());
    }

    // Note: SUBSTRATE_WORLD_DEPS_GUEST_BIN_DIR is set above and may be overridden by tests/fixtures
    // that use a host-exec world-service stub.
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
fn ensure_world_deps_bin_on_path(env_map: &mut std::collections::HashMap<String, String>) {
    const DEFAULT_WORLD_DEPS_BIN: &str = "/var/lib/substrate/world-deps/bin";
    let bin = env_map
        .get("SUBSTRATE_WORLD_DEPS_GUEST_BIN_DIR")
        .map(|v| v.trim().to_string())
        .filter(|v| !v.is_empty())
        .unwrap_or_else(|| DEFAULT_WORLD_DEPS_BIN.to_string());

    env_map.insert(
        "SUBSTRATE_WORLD_DEPS_GUEST_BIN_DIR".to_string(),
        bin.clone(),
    );

    let current = env_map.get("PATH").map(String::as_str).unwrap_or("");
    let bin_norm = bin.trim_end_matches('/');
    let has = current
        .split(':')
        .any(|segment| segment.trim_end_matches('/') == bin_norm);
    if has {
        return;
    }
    if current.trim().is_empty() {
        env_map.insert("PATH".to_string(), bin);
    } else {
        env_map.insert("PATH".to_string(), format!("{bin}:{current}"));
    }
}

#[cfg(any(target_os = "linux", all(test, unix)))]
fn resolve_host_codex_seed_home(
    intended_host_principal: &PlatformPrincipalV1,
) -> anyhow::Result<std::path::PathBuf> {
    crate::execution::install_bootstrap::unix_account_home_for_principal(intended_host_principal)
        .map(|home| home.join(".codex"))
        .map_err(|error| {
            anyhow::anyhow!("failed to resolve intended host principal home: {error:#}")
        })
}

#[cfg(any(target_os = "linux", all(test, unix)))]
fn maybe_inject_codex_auth_seed_home_for_policy(
    env_map: &mut std::collections::HashMap<String, String>,
    backend_kind: MemberRuntimeBackendKindV1,
    backend_id: &str,
    effective_policy: &substrate_broker::Policy,
    intended_host_principal: Option<&PlatformPrincipalV1>,
) -> anyhow::Result<()> {
    env_map.remove(SUBSTRATE_INTERNAL_CODEX_AUTH_SEED_HOME_ENV);
    if backend_kind != MemberRuntimeBackendKindV1::Codex {
        return Ok(());
    }
    if !effective_policy
        .agents_host_credentials_read_allowed_backends
        .iter()
        .any(|candidate| candidate == backend_id)
    {
        return Ok(());
    }
    let intended_host_principal = intended_host_principal.ok_or_else(|| {
        anyhow::anyhow!(
            "typed intended host principal is required for allowlisted Codex credential projection"
        )
    })?;
    let seed_home = resolve_host_codex_seed_home(intended_host_principal)?;
    env_map.insert(
        SUBSTRATE_INTERNAL_CODEX_AUTH_SEED_HOME_ENV.to_string(),
        seed_home.display().to_string(),
    );
    Ok(())
}

#[cfg(target_os = "linux")]
fn maybe_inject_codex_auth_seed_home_for_member_dispatch(
    env_map: &mut std::collections::HashMap<String, String>,
    dispatch: &MemberDispatchTransportRequest,
    cwd_path: &std::path::Path,
    intended_host_principal: Option<&PlatformPrincipalV1>,
) -> anyhow::Result<()> {
    let (effective_policy, _) =
        substrate_broker::resolve_effective_policy_with_explain(cwd_path, false)
            .map_err(|err| crate::execution::config_model::user_error(err.to_string()))?;
    maybe_inject_codex_auth_seed_home_for_policy(
        env_map,
        dispatch.backend_kind,
        &dispatch.backend_id,
        &effective_policy,
        intended_host_principal,
    )?;
    Ok(())
}

/// Collect filesystem diff and network scopes from world backend
#[allow(unused_variables)]
pub(super) fn collect_world_telemetry(
    _span_id: &str,
) -> (Vec<String>, Option<substrate_common::fs_diff::FsDiff>) {
    // Try to get world handle from environment
    let world_id = match env::var("SUBSTRATE_WORLD_ID") {
        Ok(id) => id,
        Err(_) => {
            // No world ID, return empty telemetry
            return (vec![], None);
        }
    };

    // Create world backend and collect telemetry
    #[cfg(target_os = "linux")]
    {
        let backend = LinuxLocalBackend::new();
        let handle = world_api::WorldHandle {
            id: world_id.clone(),
            shared_binding: None,
        };

        // Try to get filesystem diff
        let fs_diff = backend.fs_diff(&handle, _span_id).ok(); // PTY sessions may run in a separate process; missing cache is expected

        // For now, scopes are tracked in the session world's execute method
        // and would need to be retrieved from there
        let scopes_used = vec![];

        (scopes_used, fs_diff)
    }

    #[cfg(not(target_os = "linux"))]
    {
        // World backend only available on Linux for now
        (vec![], None)
    }
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
pub(crate) struct PtyWorldOutcome {
    pub(crate) exit_code: i32,
    pub(crate) fs_strategy: Option<WorldFsStrategyTraceMeta>,
    pub(crate) process_telemetry: ProcessTelemetry,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub(crate) struct ExactDispatchPolicySnapshotMaterialV1 {
    pub(crate) policy_snapshot: transport_api_types::PolicySnapshotV3,
    pub(crate) policy_snapshot_bytes: Vec<u8>,
    pub(crate) policy_snapshot_hash: String,
}

impl ExactDispatchPolicySnapshotMaterialV1 {
    pub(crate) fn from_exact_e1_material(
        policy_snapshot: transport_api_types::PolicySnapshotV3,
        policy_snapshot_bytes: Vec<u8>,
        policy_snapshot_hash: String,
    ) -> anyhow::Result<Self> {
        let material = Self {
            policy_snapshot,
            policy_snapshot_bytes,
            policy_snapshot_hash,
        };
        material.validate()?;
        Ok(material)
    }

    fn validate(&self) -> anyhow::Result<()> {
        use sha2::Digest as _;

        let decoded: transport_api_types::PolicySnapshotV3 =
            serde_json::from_slice(&self.policy_snapshot_bytes).map_err(|error| {
                anyhow::anyhow!("decode exact E1 PolicySnapshotV3 bytes: {error}")
            })?;
        let decoded_bytes = serde_json::to_vec(&decoded)
            .map_err(|error| anyhow::anyhow!("reserialize exact E1 PolicySnapshotV3: {error}"))?;
        let expected_bytes = serde_json::to_vec(&self.policy_snapshot)
            .map_err(|error| anyhow::anyhow!("serialize expected E1 PolicySnapshotV3: {error}"))?;
        let canonical = self
            .policy_snapshot
            .canonicalize()
            .map_err(anyhow::Error::msg)?;
        let canonical_bytes = serde_json::to_vec(&canonical)
            .map_err(|error| anyhow::anyhow!("serialize canonical E1 PolicySnapshotV3: {error}"))?;
        let hash = format!("{:x}", sha2::Sha256::digest(&self.policy_snapshot_bytes));
        if decoded_bytes != self.policy_snapshot_bytes
            || expected_bytes != self.policy_snapshot_bytes
            || canonical_bytes != self.policy_snapshot_bytes
            || self.policy_snapshot_hash.len() != 64
            || !self
                .policy_snapshot_hash
                .bytes()
                .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
            || hash != self.policy_snapshot_hash
        {
            anyhow::bail!(
                "exact E1 policy snapshot bytes, hash, decoded identity, or canonical identity changed"
            );
        }
        Ok(())
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub(crate) struct MemberDispatchTransportRequest {
    pub orchestration_session_id: String,
    pub participant_id: String,
    pub orchestrator_participant_id: String,
    pub parent_participant_id: Option<String>,
    pub resumed_from_participant_id: Option<String>,
    pub backend_id: String,
    pub protocol: String,
    pub run_id: String,
    pub world_id: String,
    pub world_generation: u64,
    pub initial_prompt: Option<String>,
    pub backend_kind: MemberRuntimeBackendKindV1,
    pub binary_path: String,
    pub retained_worker_launch_authority: Option<RetainedWorkerLaunchAuthorityProofV1>,
    pub e2_launch_activation: Option<E2MemberLaunchActivationCarrierV1>,
    pub config_projection: Option<ConfigProjectionActivationCarrierV1>,
    pub exact_policy_snapshot: Option<ExactDispatchPolicySnapshotMaterialV1>,
}

impl MemberDispatchTransportRequest {
    fn resolve_world_network_policy(
        &self,
        cwd_path: &std::path::Path,
    ) -> anyhow::Result<crate::execution::policy_snapshot::ResolvedWorldNetworkPolicy> {
        let Some(material) = self.exact_policy_snapshot.as_ref() else {
            return resolve_world_network_policy_for_cwd(cwd_path);
        };
        material.validate()?;
        let resolved =
            resolve_world_network_policy_for_snapshot(material.policy_snapshot.clone(), cwd_path)?;
        let resolved_bytes = serde_json::to_vec(&resolved.snapshot).map_err(|error| {
            anyhow::anyhow!("serialize resolved dispatch PolicySnapshotV3: {error}")
        })?;
        if resolved_bytes != material.policy_snapshot_bytes {
            anyhow::bail!("exact dispatch policy snapshot changed during world-network resolution");
        }
        Ok(resolved)
    }
}

fn build_execute_request(input: ExecuteRequestInput) -> ExecuteRequest {
    ExecuteRequest {
        profile: input.profile,
        cmd: input.cmd,
        cwd: Some(input.cwd),
        env: Some(input.env_map),
        pty: false,
        agent_id: input.agent_id,
        budget: None,
        policy_snapshot: input.policy_snapshot,
        shared_world: None,
        world_network: Some(input.world_network),
        world_fs_mode: Some(input.world_fs_mode),
        member_dispatch: input.member_dispatch,
        acceptance_context: input.acceptance_context,
    }
}

struct ExecuteRequestInput {
    profile: Option<String>,
    cmd: String,
    cwd: String,
    env_map: std::collections::HashMap<String, String>,
    agent_id: String,
    policy_snapshot: transport_api_types::PolicySnapshotV3,
    world_network: transport_api_types::WorldNetworkRoutingV1,
    world_fs_mode: WorldFsMode,
    member_dispatch: Option<MemberDispatchRequest>,
    acceptance_context: Option<transport_api_types::WorldWorkAcceptanceContextV1>,
}

#[allow(dead_code)]
fn build_member_dispatch_payload(
    request: &MemberDispatchTransportRequest,
) -> MemberDispatchRequest {
    let resolved_runtime = ResolvedMemberRuntimeDescriptorV1 {
        backend_kind: request.backend_kind,
        binary_path: request.binary_path.clone(),
    };
    if let Some(config_projection) = request.config_projection.clone() {
        return MemberDispatchRequest::V2(MemberDispatchRequestV2 {
            schema_version: 2,
            orchestration_session_id: request.orchestration_session_id.clone(),
            participant_id: request.participant_id.clone(),
            orchestrator_participant_id: request.orchestrator_participant_id.clone(),
            parent_participant_id: request.parent_participant_id.clone(),
            resumed_from_participant_id: request.resumed_from_participant_id.clone(),
            backend_id: request.backend_id.clone(),
            protocol: request.protocol.clone(),
            run_id: request.run_id.clone(),
            world_id: request.world_id.clone(),
            world_generation: request.world_generation,
            initial_prompt: request.initial_prompt.clone(),
            resolved_runtime,
            retained_worker_launch_authority: request.retained_worker_launch_authority.clone(),
            e2_launch_activation: request.e2_launch_activation.clone(),
            config_projection,
        });
    }
    MemberDispatchRequest::V1(MemberDispatchRequestV1 {
        schema_version: 1,
        orchestration_session_id: request.orchestration_session_id.clone(),
        participant_id: request.participant_id.clone(),
        orchestrator_participant_id: request.orchestrator_participant_id.clone(),
        parent_participant_id: request.parent_participant_id.clone(),
        resumed_from_participant_id: request.resumed_from_participant_id.clone(),
        backend_id: request.backend_id.clone(),
        protocol: request.protocol.clone(),
        run_id: request.run_id.clone(),
        world_id: request.world_id.clone(),
        world_generation: request.world_generation,
        initial_prompt: request.initial_prompt.clone(),
        resolved_runtime,
        retained_worker_launch_authority: request.retained_worker_launch_authority.clone(),
        e2_launch_activation: request.e2_launch_activation.clone(),
    })
}

#[cfg(target_os = "linux")]
pub(super) fn execute_world_pty_over_ws(
    cmd: &str,
    span_id: &str,
    parent_cmd_id: Option<&str>,
) -> anyhow::Result<PtyWorldOutcome> {
    use base64::engine::general_purpose::STANDARD;
    use base64::Engine;
    use futures::{SinkExt, StreamExt};

    // Ensure agent is ready
    ensure_world_service_ready()?;

    // Connect UDS and do WS handshake
    let rt = tokio::runtime::Runtime::new()?;
    let outcome = rt.block_on(async move {
        let socket_path = std::env::var_os("SUBSTRATE_WORLD_SOCKET")
            .map(std::path::PathBuf::from)
            .unwrap_or_else(|| std::path::PathBuf::from("/run/substrate.sock"));
        let stream = UnixStream::connect(&socket_path)
            .await
            .map_err(|e| anyhow::anyhow!("connect UDS ({}): {}", socket_path.display(), e))?;
        let url = url::Url::parse("ws://localhost/v1/stream").unwrap();
        let (ws, _resp) = tungs::client_async(url, stream)
            .await
            .map_err(|e| anyhow::anyhow!("ws handshake: {}", e))?;
        let (sink, mut stream) = ws.split();
        let sink = std::sync::Arc::new(tokio::sync::Mutex::new(sink));

        if std::env::var("SUBSTRATE_WS_DEBUG").ok().as_deref() == Some("1") {
            eprintln!("using world-service PTY WS");
        }

        // Prepare start frame (strip optional ":pty " prefix used in REPL to force PTY)
        let cmd_sanitized = if let Some(rest) = cmd.strip_prefix(":pty ") {
            rest
        } else {
            cmd
        };
        let cwd = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
        let policy_snapshot = resolve_world_network_policy_for_cwd(&cwd)?.snapshot;
        let (mut env_map, inherit_from_host) = build_world_env_map_for_cwd(&cwd)?;
        if inherit_from_host {
            eprintln!("substrate: warning: world env is forwarding selected host env vars (world.env.inherit_from_host=true)");
        }
        crate::execution::policy_snapshot::inject_world_fs_enforcement_plan_env(
            &policy_snapshot,
            &mut env_map,
        )?;
        ensure_world_deps_bin_on_path(&mut env_map);
        inject_process_trace_env(&mut env_map, Some(span_id), parent_cmd_id);
        #[cfg(unix)]
        let (cols, rows) = get_term_size();
        #[cfg(not(target_os = "linux"))]
        let (cols, rows) = (80u16, 24u16);
        let start = serde_json::json!({
            "type": "start",
            "cmd": cmd_sanitized,
            "cwd": cwd,
            "env": env_map,
            "span_id": span_id,
            "policy_snapshot": policy_snapshot,
            "cols": cols,
            "rows": rows,
        });
        sink.lock()
            .await
            .send(tungs::tungstenite::Message::Text(start.to_string()))
            .await
            .map_err(|e| anyhow::anyhow!("ws send start: {}", e))?;

        // Enter raw mode on the local terminal and ensure restoration
        #[cfg(unix)]
        let _raw_guard = RawModeGuard::for_stdin_if_tty()?;

        // Spawn stdin forwarder (raw bytes)
        let sink_in = sink.clone();
        let stdin_task = tokio::spawn(async move {
            use tokio::io::AsyncReadExt;
            let mut stdin = tokio::io::stdin();
            let mut buf = [0u8; 8192];
            loop {
                match stdin.read(&mut buf).await {
                    Ok(0) => break,
                    Ok(n) => {
                        let b64 = STANDARD.encode(&buf[..n]);
                        let frame = serde_json::json!({"type":"stdin", "data_b64": b64});
                        if sink_in
                            .lock()
                            .await
                            .send(tungs::tungstenite::Message::Text(frame.to_string()))
                            .await
                            .is_err()
                        {
                            break;
                        }
                    }
                    Err(_) => break,
                }
            }
        });

        // Spawn resize watcher (SIGWINCH)
        #[cfg(unix)]
        let resize_task = {
            let sink_resize = sink.clone();
            let mut sig = signal(SignalKind::window_change())
                .map_err(|e| anyhow::anyhow!("sigwinch subscribe: {}", e))?;
            tokio::spawn(async move {
                while sig.recv().await.is_some() {
                    let (c, r) = get_term_size();
                    let frame = serde_json::json!({"type":"resize", "cols": c, "rows": r});
                    if sink_resize
                        .lock()
                        .await
                        .send(tungs::tungstenite::Message::Text(frame.to_string()))
                        .await
                        .is_err()
                    {
                        break;
                    }
                }
            })
        };

        // Spawn Unix signal forwarders (INT, TERM, HUP, QUIT) → WS Signal frames
        #[cfg(unix)]
        let signal_tasks = {
            let mut tasks = Vec::new();

            // SIGINT
            if let Ok(mut sig) = signal(SignalKind::interrupt()) {
                let s = sink.clone();
                tasks.push(tokio::spawn(async move {
                    while sig.recv().await.is_some() {
                        let frame = serde_json::json!({"type":"signal", "sig": "INT"});
                        if s.lock()
                            .await
                            .send(tungs::tungstenite::Message::Text(frame.to_string()))
                            .await
                            .is_err()
                        {
                            break;
                        }
                    }
                }));
            }
            // SIGTERM
            if let Ok(mut sig) = signal(SignalKind::terminate()) {
                let s = sink.clone();
                tasks.push(tokio::spawn(async move {
                    while sig.recv().await.is_some() {
                        let frame = serde_json::json!({"type":"signal", "sig": "TERM"});
                        if s.lock()
                            .await
                            .send(tungs::tungstenite::Message::Text(frame.to_string()))
                            .await
                            .is_err()
                        {
                            break;
                        }
                    }
                }));
            }
            // SIGHUP
            if let Ok(mut sig) = signal(SignalKind::hangup()) {
                let s = sink.clone();
                tasks.push(tokio::spawn(async move {
                    while sig.recv().await.is_some() {
                        let frame = serde_json::json!({"type":"signal", "sig": "HUP"});
                        if s.lock()
                            .await
                            .send(tungs::tungstenite::Message::Text(frame.to_string()))
                            .await
                            .is_err()
                        {
                            break;
                        }
                    }
                }));
            }
            // SIGQUIT
            if let Ok(mut sig) = signal(SignalKind::quit()) {
                let s = sink.clone();
                tasks.push(tokio::spawn(async move {
                    while sig.recv().await.is_some() {
                        let frame = serde_json::json!({"type":"signal", "sig": "QUIT"});
                        if s.lock()
                            .await
                            .send(tungs::tungstenite::Message::Text(frame.to_string()))
                            .await
                            .is_err()
                        {
                            break;
                        }
                    }
                }));
            }

            tasks
        };

        let mut exit_code: i32 = 0;
        let mut fs_strategy: Option<WorldFsStrategyTraceMeta> = None;
        let mut process_telemetry = ProcessTelemetry::default();
        while let Some(msg) = stream.next().await {
            let msg = msg.map_err(|e| anyhow::anyhow!("ws recv: {}", e))?;
            if msg.is_text() {
                if let Ok(v) = serde_json::from_str::<serde_json::Value>(&msg.to_string()) {
                    match v.get("type").and_then(|t| t.as_str()) {
                        Some("stdout") => {
                            if let Some(b64) = v.get("data_b64").and_then(|x| x.as_str()) {
                                if let Ok(bytes) = STANDARD.decode(b64) {
                                    use std::io::Write;
                                    let _ = std::io::stdout().write_all(&bytes);
                                    let _ = std::io::stdout().flush();
                                }
                            }
                        }
                        Some("exit") => {
                            exit_code = v.get("code").and_then(|c| c.as_i64()).unwrap_or(0) as i32;
                            process_telemetry = extract_process_telemetry_from_ws_exit(&v);
                            if let (Some(primary), Some(final_strategy), Some(reason)) = (
                                v.get("world_fs_strategy_primary")
                                    .and_then(serde_json::Value::as_str)
                                    .and_then(substrate_common::WorldFsStrategy::parse),
                                v.get("world_fs_strategy_final")
                                    .and_then(serde_json::Value::as_str)
                                    .and_then(substrate_common::WorldFsStrategy::parse),
                                v.get("world_fs_strategy_fallback_reason")
                                    .and_then(serde_json::Value::as_str)
                                    .and_then(
                                        substrate_common::WorldFsStrategyFallbackReason::parse,
                                    ),
                            ) {
                                fs_strategy = Some(WorldFsStrategyTraceMeta {
                                    primary,
                                    final_strategy,
                                    fallback_reason: reason,
                                });
                            }
                            break;
                        }
                        Some("error") => {
                            if let Some(message) = v.get("message").and_then(|m| m.as_str()) {
                                if message.contains("WORLD_FS_STRATEGY_UNAVAILABLE") {
                                    return Err(anyhow::Error::new(
                                        WorldFsStrategyUnavailableError {
                                            raw_message: message.to_string(),
                                            fallback_reason:
                                                parse_world_fs_strategy_unavailable_reason(message),
                                        },
                                    ));
                                }
                                return Err(anyhow::anyhow!("world-service error: {}", message));
                            }
                            return Err(anyhow::anyhow!("world-service error"));
                        }
                        _ => {}
                    }
                }
            } else if msg.is_close() {
                break;
            }
        }

        // Cleanup background tasks
        stdin_task.abort();
        #[cfg(unix)]
        {
            resize_task.abort();
            for t in signal_tasks {
                t.abort();
            }
        }
        Ok::<PtyWorldOutcome, anyhow::Error>(PtyWorldOutcome {
            exit_code,
            fs_strategy,
            process_telemetry,
        })
    })?;
    Ok(outcome)
}

#[cfg(target_os = "linux")]
pub(super) fn ensure_world_service_ready() -> anyhow::Result<()> {
    ensure_world_service_ready_with(ensure_world_service_ready_for_target)
}

#[cfg(target_os = "linux")]
fn ensure_world_service_ready_with(
    ready: impl FnOnce(&std::path::Path, WorldServiceReadinessPosture) -> anyhow::Result<()>,
) -> anyhow::Result<()> {
    const DEFAULT_SOCKET_PATH: &str = "/run/substrate.sock";

    let socket_path = std::env::var_os("SUBSTRATE_WORLD_SOCKET")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| std::path::PathBuf::from(DEFAULT_SOCKET_PATH));
    let socket_override_active = std::env::var_os("SUBSTRATE_WORLD_SOCKET")
        .map(|path| path != std::ffi::OsStr::new(DEFAULT_SOCKET_PATH))
        .unwrap_or(false);
    ready(
        &socket_path,
        WorldServiceReadinessPosture::LegacyCompatibility {
            socket_override_active,
            candidate_bins: resolve_legacy_compatibility_candidate_bins,
            activation_mode: resolve_legacy_compatibility_activation_mode,
        },
    )
}

#[cfg(target_os = "linux")]
fn ensure_world_service_ready_for_target(
    socket_path: &std::path::Path,
    posture: WorldServiceReadinessPosture,
) -> anyhow::Result<()> {
    ensure_world_service_ready_for_target_with_spawn(socket_path, posture, spawn_world_service)
}

#[cfg(target_os = "linux")]
fn world_service_command(
    binary: &str,
    environment: WorldServiceSpawnEnvironment,
    installed_cwd: &std::path::Path,
) -> std::process::Command {
    let mut command = std::process::Command::new(binary);
    if environment == WorldServiceSpawnEnvironment::InstalledLinuxProduct {
        command
            .env_clear()
            .env("PATH", AUTHENTICATED_WORLD_SERVICE_PATH)
            .env("SUBSTRATE_WORLD_SOCKET", AUTHENTICATED_WORLD_SOCKET)
            .current_dir(installed_cwd);
    }
    command
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null());
    command
}

#[cfg(target_os = "linux")]
fn spawn_world_service(plan: WorldServiceSpawnPlan) -> anyhow::Result<()> {
    let binary = plan
        .candidate_bins
        .into_iter()
        .flatten()
        .find(|path| std::path::Path::new(path).exists())
        .ok_or_else(|| anyhow::anyhow!("world-service binary not found"))?;

    world_service_command(
        &binary,
        plan.environment,
        std::path::Path::new(AUTHENTICATED_WORLD_SERVICE_CWD),
    )
    .spawn()
    .map_err(|error| anyhow::anyhow!("spawn world-service: {}", error))?;
    Ok(())
}

#[cfg(target_os = "linux")]
fn ensure_world_service_ready_for_target_with_spawn(
    socket_path: &std::path::Path,
    posture: WorldServiceReadinessPosture,
    spawn_service: impl FnOnce(WorldServiceSpawnPlan) -> anyhow::Result<()>,
) -> anyhow::Result<()> {
    use std::path::Path;
    use std::thread;
    use std::time::{Duration, Instant};

    // Helper: quick readiness probe via HTTP-over-UDS
    fn probe_caps(sock: &Path) -> bool {
        use std::io::{Read, Write};
        match std::os::unix::net::UnixStream::connect(sock) {
            Ok(mut s) => {
                let timeout = std::time::Duration::from_millis(WORLD_SERVICE_PROBE_IO_TIMEOUT_MS);
                let _ = s.set_read_timeout(Some(timeout));
                let _ = s.set_write_timeout(Some(timeout));
                let req = b"GET /v1/capabilities HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n";
                if s.write_all(req).is_ok() {
                    let mut buf = [0u8; 512];
                    if let Ok(n) = s.read(&mut buf) {
                        return n > 0
                            && std::str::from_utf8(&buf[..n])
                                .unwrap_or("")
                                .contains(" 200 ");
                    }
                }
                false
            }
            Err(_) => false,
        }
    }

    // Fast path: already ready
    if probe_caps(socket_path) {
        return Ok(());
    }

    let activation_mode = posture.activation_mode();

    if matches!(
        activation_mode,
        socket_activation::SocketActivationMode::SocketActivation
    ) {
        let deadline = Instant::now() + Duration::from_millis(WORLD_SERVICE_ACTIVATION_WAIT_MS);
        while Instant::now() < deadline {
            if probe_caps(socket_path) {
                return Ok(());
            }
            thread::sleep(Duration::from_millis(WORLD_SERVICE_ACTIVATION_POLL_MS));
        }
        anyhow::bail!(
            "world-service socket activation detected but {} did not respond. \
             Run 'systemctl status substrate-world-service.socket' for details.",
            socket_path.display()
        );
    }

    // Clean up stale socket if present (no responding server). Only do this when we're
    // confident the socket isn't systemd-managed; if systemd probing fails, keep the
    // path intact to avoid breaking socket activation.
    if matches!(
        activation_mode,
        socket_activation::SocketActivationMode::Manual
    ) && Path::new(socket_path).exists()
    {
        let _ = std::fs::remove_file(socket_path);
    }

    let spawn_plan = posture.into_spawn_plan(socket_path)?;
    spawn_service(spawn_plan)?;

    // Wait up to ~1s for readiness
    let deadline = std::time::Instant::now()
        + std::time::Duration::from_millis(WORLD_SERVICE_READINESS_WAIT_MS);
    while std::time::Instant::now() < deadline {
        if probe_caps(socket_path) {
            return Ok(());
        }
        std::thread::sleep(std::time::Duration::from_millis(
            WORLD_SERVICE_READINESS_POLL_MS,
        ));
    }
    anyhow::bail!("world-service readiness probe failed")
}

#[cfg(target_os = "linux")]
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum LinuxWorldInit {
    Disabled,
    Agent,
    LocalBackend,
    LocalBackendFailed,
}

#[cfg(target_os = "linux")]
pub(crate) fn init_linux_world(world_disabled: bool) -> LinuxWorldInit {
    init_linux_world_with_probe(world_disabled, ensure_world_service_ready)
}

#[cfg(target_os = "linux")]
pub(crate) fn init_linux_world_with_probe<F>(world_disabled: bool, agent_probe: F) -> LinuxWorldInit
where
    F: Fn() -> anyhow::Result<()>,
{
    if world_disabled {
        return LinuxWorldInit::Disabled;
    }

    #[cfg(test)]
    let _env_guard = world_env_guard();

    match agent_probe() {
        Ok(()) => {
            env::set_var("SUBSTRATE_WORLD", "enabled");
            env::remove_var("SUBSTRATE_WORLD_ID");
            LinuxWorldInit::Agent
        }
        Err(_agent_err) => {
            #[cfg(test)]
            if let Ok(mock_id) = env::var("SUBSTRATE_TEST_LOCAL_WORLD_ID") {
                env::set_var("SUBSTRATE_WORLD", "enabled");
                env::set_var("SUBSTRATE_WORLD_ID", mock_id);
                return LinuxWorldInit::LocalBackend;
            }

            let spec = bootstrap_world_spec(
                crate::execution::settings::world_root_from_env().path,
                world_fs_mode(),
            );
            let backend = LinuxLocalBackend::new();
            match backend.ensure_session(&spec) {
                Ok(handle) => {
                    env::set_var("SUBSTRATE_WORLD", "enabled");
                    env::set_var("SUBSTRATE_WORLD_ID", &handle.id);
                    LinuxWorldInit::LocalBackend
                }
                Err(_local_err) => LinuxWorldInit::LocalBackendFailed,
            }
        }
    }
}

#[cfg(target_os = "macos")]
pub(super) fn execute_world_pty_over_ws_macos(
    cmd: &str,
    span_id: &str,
    _parent_cmd_id: Option<&str>,
) -> anyhow::Result<PtyWorldOutcome> {
    use base64::engine::general_purpose::STANDARD;
    use base64::Engine;
    use futures::StreamExt;
    use tungs::tungstenite::Message;

    let ctx = pw::get_context().ok_or_else(|| anyhow::anyhow!("no platform world context"))?;

    // Put the host terminal into raw mode so interactive programs (nano/vim/top)
    // receive keystrokes immediately (not line-buffered until Enter).
    let _terminal_guard = crate::execution::pty::MinimalTerminalGuard::new()?;

    let rt = tokio::runtime::Runtime::new()?;
    let code = rt.block_on(async move {
        async fn handle_ws<S>(
            ws: tungs::WebSocketStream<S>,
            cmd: &str,
            span_id: &str,
        ) -> anyhow::Result<PtyWorldOutcome>
        where
            S: tokio::io::AsyncRead + tokio::io::AsyncWrite + Unpin + Send + 'static,
        {
            use futures::SinkExt;
            use std::sync::Arc;
            use tokio::sync::Mutex;
            let (sink, mut stream) = ws.split();
            let sink = Arc::new(Mutex::new(sink));

            let cmd_sanitized = if let Some(rest) = cmd.strip_prefix(":pty ") {
                rest
            } else {
                cmd
            };
            let cwd = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
            let policy_snapshot = resolve_world_network_policy_for_cwd(&cwd)?.snapshot;
            let (mut env_map, inherit_from_host) = build_world_env_map_for_cwd(&cwd)?;
            if inherit_from_host {
                eprintln!("substrate: warning: world env is forwarding selected host env vars (world.env.inherit_from_host=true)");
            }
            normalize_env_for_linux_guest(&mut env_map);
            apply_macos_staged_workspace_project_dir_override(&mut env_map);
            crate::execution::policy_snapshot::inject_world_fs_enforcement_plan_env(
                &policy_snapshot,
                &mut env_map,
            )?;
            env_map
                .entry("XDG_DATA_HOME".to_string())
                .or_insert_with(|| "/root/.local/share".to_string());

            // Ensure a few common XDG dirs exist to avoid noisy TUI warnings (e.g. nano history).
            // This is best-effort and does not fail the session in read-only modes.
            let cmd_sanitized = format!(
                "mkdir -p \"${{XDG_DATA_HOME:-$HOME/.local/share}}\" >/dev/null 2>&1 || true; {cmd_sanitized}"
            );
            let (cols, rows) = match crate::execution::pty::get_terminal_size() {
                Ok(sz) => (sz.cols, sz.rows),
                Err(_) => (80u16, 24u16),
            };
            let start = serde_json::json!({
                "type": "start",
                "cmd": cmd_sanitized,
                "cwd": cwd,
                "env": env_map,
                "span_id": span_id,
                "policy_snapshot": policy_snapshot,
                "cols": cols,
                "rows": rows,
            });
            sink.lock()
                .await
                .send(Message::Text(start.to_string()))
                .await
                .map_err(|e| anyhow::anyhow!("ws send start: {}", e))?;

            // stdin forwarder
            let mut stdin = tokio::io::stdin();
            let sink_for_stdin = sink.clone();
            let stdin_task = tokio::spawn(async move {
                use tokio::io::AsyncReadExt;
                let mut buf = [0u8; 8192];
                loop {
                    match stdin.read(&mut buf).await {
                        Ok(0) => break,
                        Ok(n) => {
                            let b64 = base64::engine::general_purpose::STANDARD.encode(&buf[..n]);
                            let frame = serde_json::json!({"type":"stdin", "data_b64": b64});
                            if sink_for_stdin
                                .lock()
                                .await
                                .send(Message::Text(frame.to_string()))
                                .await
                                .is_err()
                            {
                                break;
                            }
                        }
                        Err(_) => break,
                    }
                }
            });

            // Terminal resize forwarder (SIGWINCH => WS "resize" frame).
            let sink_for_resize = sink.clone();
            let resize_task = tokio::spawn(async move {
                #[cfg(unix)]
                {
                    use tokio::signal::unix::{signal, SignalKind};
                    if let Ok(mut sigwinch) = signal(SignalKind::window_change()) {
                        while sigwinch.recv().await.is_some() {
                            let (cols, rows) = match crate::execution::pty::get_terminal_size() {
                                Ok(sz) => (sz.cols, sz.rows),
                                Err(_) => continue,
                            };
                            let frame =
                                serde_json::json!({"type":"resize", "cols": cols, "rows": rows});
                            if sink_for_resize
                                .lock()
                                .await
                                .send(Message::Text(frame.to_string()))
                                .await
                                .is_err()
                            {
                                break;
                            }
                        }
                    }
                }
            });

            let mut exit_code: i32 = 0;
            let mut fs_strategy: Option<WorldFsStrategyTraceMeta> = None;
            let mut process_telemetry = ProcessTelemetry::default();
            while let Some(msg) = stream.next().await {
                let msg = msg.map_err(|e| anyhow::anyhow!("ws recv: {}", e))?;
                if msg.is_text() {
                    if let Ok(v) = serde_json::from_str::<serde_json::Value>(&msg.to_string()) {
                        match v.get("type").and_then(|t| t.as_str()) {
                            Some("stdout") => {
                                if let Some(b64) = v.get("data_b64").and_then(|x| x.as_str()) {
                                    if let Ok(bytes) = STANDARD.decode(b64) {
                                        use std::io::Write;
                                        let _ = std::io::stdout().write_all(&bytes);
                                        let _ = std::io::stdout().flush();
                                    }
                                }
                            }
                            Some("exit") => {
                                exit_code =
                                    v.get("code").and_then(|c| c.as_i64()).unwrap_or(0) as i32;
                                process_telemetry = extract_process_telemetry_from_ws_exit(&v);
                                if let (Some(primary), Some(final_strategy), Some(reason)) = (
                                    v.get("world_fs_strategy_primary")
                                        .and_then(serde_json::Value::as_str)
                                        .and_then(substrate_common::WorldFsStrategy::parse),
                                    v.get("world_fs_strategy_final")
                                        .and_then(serde_json::Value::as_str)
                                        .and_then(substrate_common::WorldFsStrategy::parse),
                                    v.get("world_fs_strategy_fallback_reason")
                                        .and_then(serde_json::Value::as_str)
                                        .and_then(
                                            substrate_common::WorldFsStrategyFallbackReason::parse,
                                        ),
                                ) {
                                    fs_strategy = Some(WorldFsStrategyTraceMeta {
                                        primary,
                                        final_strategy,
                                        fallback_reason: reason,
                                    });
                                }
                                break;
                            }
                            Some("error") => {
                                if let Some(message) = v.get("message").and_then(|m| m.as_str()) {
                                    if message.contains("WORLD_FS_STRATEGY_UNAVAILABLE") {
                                        return Err(anyhow::Error::new(
                                            WorldFsStrategyUnavailableError {
                                                raw_message: message.to_string(),
                                                fallback_reason:
                                                    parse_world_fs_strategy_unavailable_reason(
                                                        message,
                                                    ),
                                            },
                                        ));
                                    }
                                    return Err(anyhow::anyhow!(
                                        "world-service error: {}",
                                        message
                                    ));
                                }
                                return Err(anyhow::anyhow!("world-service error"));
                            }
                            _ => {}
                        }
                    }
                } else if msg.is_close() {
                    break;
                }
            }

            stdin_task.abort();
            resize_task.abort();
            Ok::<PtyWorldOutcome, anyhow::Error>(PtyWorldOutcome {
                exit_code,
                fs_strategy,
                process_telemetry,
            })
        }

        let ws = pw::connect_transport_stream_ws(&ctx.transport).await?;
        handle_ws(ws, cmd, span_id).await
    })?;

    Ok(code)
}

pub(crate) struct AgentStreamOutcome {
    pub(crate) exit_code: i32,
    pub(crate) scopes_used: Vec<String>,
    pub(crate) fs_diff: Option<substrate_common::FsDiff>,
    pub(crate) fs_strategy: Option<WorldFsStrategyTraceMeta>,
    pub(crate) process_telemetry: ProcessTelemetry,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct WorldFsStrategyTraceMeta {
    pub(crate) primary: substrate_common::WorldFsStrategy,
    pub(crate) final_strategy: substrate_common::WorldFsStrategy,
    pub(crate) fallback_reason: substrate_common::WorldFsStrategyFallbackReason,
}

#[derive(Debug)]
pub(crate) struct WorldFsStrategyUnavailableError {
    pub(crate) raw_message: String,
    #[cfg_attr(not(target_os = "linux"), allow(dead_code))]
    pub(crate) fallback_reason: Option<substrate_common::WorldFsStrategyFallbackReason>,
}

impl std::fmt::Display for WorldFsStrategyUnavailableError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.raw_message)
    }
}

impl std::error::Error for WorldFsStrategyUnavailableError {}

fn parse_world_fs_strategy_unavailable_reason(
    message: &str,
) -> Option<substrate_common::WorldFsStrategyFallbackReason> {
    if !message.contains("WORLD_FS_STRATEGY_UNAVAILABLE") {
        return None;
    }
    for token in message.split_whitespace() {
        if let Some(value) = token.strip_prefix("fallback_reason=") {
            return substrate_common::WorldFsStrategyFallbackReason::parse(value);
        }
    }
    None
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
fn extract_process_telemetry_from_ws_exit(value: &serde_json::Value) -> ProcessTelemetry {
    let process_events = value
        .get("process_events")
        .cloned()
        .and_then(|raw| serde_json::from_value(raw).ok())
        .unwrap_or_default();
    let process_events_status = value
        .get("process_events_status")
        .and_then(serde_json::Value::as_str)
        .and_then(substrate_common::ProcessEventsStatus::parse)
        .unwrap_or(substrate_common::ProcessEventsStatus::Unavailable);

    ProcessTelemetry {
        process_events,
        process_events_status,
        process_events_reason: value
            .get("process_events_reason")
            .and_then(serde_json::Value::as_str)
            .map(ToOwned::to_owned)
            .or_else(|| {
                (process_events_status == substrate_common::ProcessEventsStatus::Unavailable)
                    .then(|| "backend_disabled".to_string())
            }),
        process_events_dropped: value
            .get("process_events_dropped")
            .and_then(serde_json::Value::as_u64),
        process_events_max: value
            .get("process_events_max")
            .and_then(serde_json::Value::as_u64),
        process_events_backend: value
            .get("process_events_backend")
            .and_then(serde_json::Value::as_str)
            .map(ToOwned::to_owned),
        process_events_error: value
            .get("process_events_error")
            .and_then(serde_json::Value::as_str)
            .map(ToOwned::to_owned),
    }
}

#[cfg_attr(
    target_os = "linux",
    allow(
        dead_code,
        reason = "the F-only Linux consumers use the additive authenticated builder; non-Linux compatibility consumers retain this frozen builder"
    )
)]
pub(crate) fn build_agent_client_and_request(
    cmd: &str,
) -> anyhow::Result<(
    transport_api_client::AgentClient,
    transport_api_types::ExecuteRequest,
    String,
)> {
    build_agent_client_and_request_with_trace_metadata(cmd, None, None)
}

#[cfg(target_os = "linux")]
pub(crate) fn build_authenticated_world_deps_client_and_request(
    context: &crate::builtins::world_deps::AuthenticatedWorldDepsContextV1,
    cmd: &str,
    cwd_override: Option<&std::path::Path>,
    profile: &str,
) -> anyhow::Result<(
    transport_api_client::AgentClient,
    transport_api_types::ExecuteRequest,
    String,
)> {
    build_authenticated_world_deps_client_and_request_impl(
        context,
        cmd,
        cwd_override,
        profile,
        ensure_world_service_ready_for_target,
    )
}

#[cfg(target_os = "linux")]
fn build_authenticated_world_deps_client_and_request_impl(
    context: &crate::builtins::world_deps::AuthenticatedWorldDepsContextV1,
    cmd: &str,
    cwd_override: Option<&std::path::Path>,
    profile: &str,
    ensure_ready: impl FnOnce(&std::path::Path, WorldServiceReadinessPosture) -> anyhow::Result<()>,
) -> anyhow::Result<(
    transport_api_client::AgentClient,
    transport_api_types::ExecuteRequest,
    String,
)> {
    context.revalidate_authority()?;
    if !RESERVED_WORLD_REQUEST_PROFILES.contains(&profile) {
        return Err(anyhow::anyhow!(
            "invalid authenticated world-deps request profile"
        ));
    }
    let cwd_path = cwd_override.unwrap_or_else(|| context.launch_cwd());
    if !cwd_path.is_absolute() {
        return Err(anyhow::anyhow!(
            "authenticated world-deps request cwd must be absolute"
        ));
    }
    let cwd = cwd_path.to_str().ok_or_else(|| {
        anyhow::anyhow!("authenticated world-deps request cwd must be valid UTF-8")
    })?;

    ensure_ready(
        std::path::Path::new(AUTHENTICATED_WORLD_SOCKET),
        WorldServiceReadinessPosture::InstalledLinuxProduct {
            activation_mode: resolve_installed_product_activation_mode,
        },
    )?;

    let env_map = std::collections::HashMap::from([
        (
            "SUBSTRATE_WORLD_DEPS_GUEST_BIN_DIR".to_string(),
            "/var/lib/substrate/world-deps/bin".to_string(),
        ),
        (
            "PATH".to_string(),
            "/var/lib/substrate/world-deps/bin:/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin"
                .to_string(),
        ),
        ("HOME".to_string(), "/root".to_string()),
        ("XDG_CONFIG_HOME".to_string(), "/root/.config".to_string()),
        (
            "XDG_DATA_HOME".to_string(),
            "/root/.local/share".to_string(),
        ),
        (
            "XDG_CACHE_HOME".to_string(),
            "/root/.cache".to_string(),
        ),
        ("TERM".to_string(), "xterm-256color".to_string()),
    ]);
    let network_policy = context.runtime_network_policy();
    let request = build_execute_request(ExecuteRequestInput {
        profile: Some(profile.to_string()),
        cmd: cmd.to_string(),
        cwd: cwd.to_string(),
        env_map,
        agent_id: "human".to_string(),
        policy_snapshot: network_policy.snapshot.clone(),
        world_network: request_world_network_routing(network_policy),
        world_fs_mode: context.effective_policy().world_fs_policy().mode,
        member_dispatch: None,
        acceptance_context: None,
    });
    request.validate().map_err(|error| anyhow::anyhow!(error))?;
    let client = AgentClient::unix_socket(AUTHENTICATED_WORLD_SOCKET)?;
    Ok((client, request, "human".to_string()))
}

pub(crate) fn build_agent_client_and_request_with_trace_metadata(
    cmd: &str,
    parent_span_id: Option<&str>,
    parent_cmd_id: Option<&str>,
) -> anyhow::Result<(
    transport_api_client::AgentClient,
    transport_api_types::ExecuteRequest,
    String,
)> {
    build_agent_client_and_request_impl(cmd, parent_span_id, parent_cmd_id)
}

#[allow(dead_code)]
pub(crate) fn build_agent_client_and_member_dispatch_request(
    request: &MemberDispatchTransportRequest,
) -> anyhow::Result<(
    transport_api_client::AgentClient,
    transport_api_types::ExecuteRequest,
    String,
)> {
    let cwd_path = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
    build_agent_client_and_member_dispatch_request_impl(
        request,
        &cwd_path,
        #[cfg(target_os = "linux")]
        None,
    )
}

#[allow(dead_code)]
pub(crate) fn build_agent_client_and_member_dispatch_request_for_cwd(
    request: &MemberDispatchTransportRequest,
    cwd_path: &std::path::Path,
    acceptance_context: Option<transport_api_types::WorldWorkAcceptanceContextV1>,
    #[cfg(target_os = "linux")] intended_host_principal: Option<&PlatformPrincipalV1>,
) -> anyhow::Result<(
    transport_api_client::AgentClient,
    transport_api_types::ExecuteRequest,
    String,
)> {
    let (client, mut execute_request, agent_id) =
        build_agent_client_and_member_dispatch_request_impl(
            request,
            cwd_path,
            #[cfg(target_os = "linux")]
            intended_host_principal,
        )?;
    execute_request.acceptance_context = acceptance_context;
    execute_request
        .validate()
        .map_err(|error| anyhow::anyhow!(error))?;
    Ok((client, execute_request, agent_id))
}

pub(crate) fn build_agent_client_and_pending_diff_request() -> anyhow::Result<(
    transport_api_client::AgentClient,
    transport_api_types::PendingDiffRequestV1,
    String,
)> {
    build_agent_client_and_pending_diff_request_impl()
}

fn current_world_fs_mode() -> WorldFsMode {
    std::env::var("SUBSTRATE_WORLD_FS_MODE")
        .ok()
        .and_then(|value| WorldFsMode::parse(&value))
        .unwrap_or_else(world_fs_mode)
}

fn current_world_request_profile() -> Option<String> {
    std::env::var("SUBSTRATE_WORLD_REQUEST_PROFILE")
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .filter(|value| !RESERVED_WORLD_REQUEST_PROFILES.contains(&value.as_str()))
}

#[cfg(target_os = "windows")]
fn mark_windows_world_dispatch_active_without_session() {
    std::env::set_var("SUBSTRATE_WORLD", "enabled");
    std::env::remove_var("SUBSTRATE_WORLD_ID");
}

#[cfg(target_os = "windows")]
fn validate_execute_response_shared_world(
    requested: Option<&transport_api_types::SharedWorldOwnerSpec>,
    response: &transport_api_types::ExecuteResponse,
) -> anyhow::Result<()> {
    crate::execution::repl_persistent_session::validate_shared_world_echo(
        requested,
        response.shared_world.as_ref(),
        "execute_response.shared_world",
        None,
    )
    .map(|_| ())
    .map_err(|message| anyhow::anyhow!("protocol error: {message}"))
}

#[cfg(target_os = "linux")]
fn build_agent_client_and_request_impl(
    cmd: &str,
    parent_span_id: Option<&str>,
    parent_cmd_id: Option<&str>,
) -> anyhow::Result<(
    transport_api_client::AgentClient,
    transport_api_types::ExecuteRequest,
    String,
)> {
    ensure_world_service_ready()?;

    let socket_path = std::env::var_os("SUBSTRATE_WORLD_SOCKET")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| std::path::PathBuf::from("/run/substrate.sock"));

    let client = AgentClient::unix_socket(&socket_path)?;
    let cwd_path = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
    let cwd = cwd_path.display().to_string();
    let agent_id = std::env::var("SUBSTRATE_AGENT_ID").unwrap_or_else(|_| "human".to_string());
    let network_policy = resolve_world_network_policy_for_cwd(&cwd_path)?;
    let world_network = request_world_network_routing(&network_policy);
    let policy_snapshot = network_policy.snapshot;
    let (mut env_map, inherit_from_host) = build_world_env_map_for_cwd(&cwd_path)?;
    if inherit_from_host {
        eprintln!("substrate: warning: world env is forwarding selected host env vars (world.env.inherit_from_host=true)");
    }
    crate::execution::policy_snapshot::inject_world_fs_enforcement_plan_env(
        &policy_snapshot,
        &mut env_map,
    )?;
    ensure_world_deps_bin_on_path(&mut env_map);
    preserve_world_project_dir_override(&mut env_map, &cwd_path);
    inject_process_trace_env(&mut env_map, parent_span_id, parent_cmd_id);

    let request = build_execute_request(ExecuteRequestInput {
        profile: current_world_request_profile(),
        cmd: cmd.to_string(),
        cwd,
        env_map,
        agent_id: agent_id.clone(),
        policy_snapshot,
        world_network,
        world_fs_mode: current_world_fs_mode(),
        member_dispatch: None,
        acceptance_context: None,
    });

    Ok((client, request, agent_id))
}

#[allow(dead_code)]
#[cfg(target_os = "linux")]
fn build_agent_client_and_member_dispatch_request_impl(
    dispatch: &MemberDispatchTransportRequest,
    cwd_path: &std::path::Path,
    intended_host_principal: Option<&PlatformPrincipalV1>,
) -> anyhow::Result<(
    transport_api_client::AgentClient,
    transport_api_types::ExecuteRequest,
    String,
)> {
    let socket_path = std::env::var_os("SUBSTRATE_WORLD_SOCKET")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| std::path::PathBuf::from("/run/substrate.sock"));

    let client = AgentClient::unix_socket(&socket_path)?;
    let cwd_path = cwd_path.to_path_buf();
    let cwd = cwd_path.display().to_string();
    let agent_id = std::env::var("SUBSTRATE_AGENT_ID").unwrap_or_else(|_| "human".to_string());
    let network_policy = dispatch.resolve_world_network_policy(&cwd_path)?;
    let world_network = request_world_network_routing(&network_policy);
    let policy_snapshot = network_policy.snapshot;
    let (mut env_map, inherit_from_host) = build_world_env_map_for_cwd(&cwd_path)?;
    if inherit_from_host {
        eprintln!("substrate: warning: world env is forwarding selected host env vars (world.env.inherit_from_host=true)");
    }
    crate::execution::policy_snapshot::inject_world_fs_enforcement_plan_env(
        &policy_snapshot,
        &mut env_map,
    )?;
    ensure_world_deps_bin_on_path(&mut env_map);
    maybe_inject_codex_auth_seed_home_for_member_dispatch(
        &mut env_map,
        dispatch,
        &cwd_path,
        intended_host_principal,
    )?;
    preserve_world_project_dir_override(&mut env_map, &cwd_path);
    let request = build_execute_request(ExecuteRequestInput {
        profile: current_world_request_profile(),
        cmd: String::new(),
        cwd,
        env_map,
        agent_id: agent_id.clone(),
        policy_snapshot,
        world_network,
        world_fs_mode: current_world_fs_mode(),
        member_dispatch: Some(build_member_dispatch_payload(dispatch)),
        acceptance_context: None,
    });

    Ok((client, request, agent_id))
}

#[cfg_attr(not(target_os = "linux"), allow(dead_code))]
pub(super) fn preserve_world_project_dir_override(
    env_map: &mut std::collections::HashMap<String, String>,
    cwd_path: &std::path::Path,
) {
    if env_map
        .get(WORLD_PROJECT_DIR_OVERRIDE_ENV)
        .map(|value| value.trim())
        .filter(|value| {
            *value == MACOS_STAGED_WORKSPACE_CURRENT
                || value.starts_with(&format!("{MACOS_STAGED_WORKSPACE_CURRENT}/"))
        })
        .is_some()
    {
        return;
    }

    let anchor_mode = env_map
        .get("SUBSTRATE_ANCHOR_MODE")
        .and_then(|value| WorldRootMode::parse(value));
    let anchor_path = env_map
        .get("SUBSTRATE_ANCHOR_PATH")
        .map(|value| value.trim())
        .filter(|value| !value.is_empty())
        .map(std::path::PathBuf::from);

    let fallback_root = crate::execution::settings::world_root_from_env().anchor_root(cwd_path);
    let project_dir = match anchor_mode {
        Some(WorldRootMode::Project) => anchor_path.unwrap_or_else(|| cwd_path.to_path_buf()),
        Some(WorldRootMode::FollowCwd) => cwd_path.to_path_buf(),
        Some(WorldRootMode::Custom) => anchor_path.unwrap_or(fallback_root),
        None => fallback_root,
    };
    env_map.insert(
        WORLD_PROJECT_DIR_OVERRIDE_ENV.to_string(),
        project_dir.display().to_string(),
    );
}

#[cfg(target_os = "macos")]
pub(super) fn apply_macos_staged_workspace_project_dir_override(
    env_map: &mut std::collections::HashMap<String, String>,
) {
    if env_map
        .get(WORLD_PROJECT_DIR_OVERRIDE_ENV)
        .map(|value| value.trim())
        .is_some_and(|value| !value.is_empty())
    {
        return;
    }

    env_map.insert(
        WORLD_PROJECT_DIR_OVERRIDE_ENV.to_string(),
        MACOS_STAGED_WORKSPACE_CURRENT.to_string(),
    );
}

#[cfg(target_os = "linux")]
fn build_agent_client_and_pending_diff_request_impl() -> anyhow::Result<(
    transport_api_client::AgentClient,
    transport_api_types::PendingDiffRequestV1,
    String,
)> {
    ensure_world_service_ready()?;

    let socket_path = std::env::var_os("SUBSTRATE_WORLD_SOCKET")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| std::path::PathBuf::from("/run/substrate.sock"));

    let client = AgentClient::unix_socket(&socket_path)?;
    let cwd_path = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
    let cwd = cwd_path.display().to_string();
    let agent_id = std::env::var("SUBSTRATE_AGENT_ID").unwrap_or_else(|_| "human".to_string());
    let network_policy = resolve_world_network_policy_for_cwd(&cwd_path)?;
    let world_network = request_world_network_routing(&network_policy);
    let policy_snapshot = network_policy.snapshot;
    let (mut env_map, _inherit_from_host) = build_world_env_map_for_cwd(&cwd_path)?;
    crate::execution::policy_snapshot::inject_world_fs_enforcement_plan_env(
        &policy_snapshot,
        &mut env_map,
    )?;
    ensure_world_deps_bin_on_path(&mut env_map);

    let request = transport_api_types::PendingDiffRequestV1 {
        profile: current_world_request_profile(),
        cwd: Some(cwd),
        env: Some(env_map),
        agent_id: agent_id.clone(),
        policy_snapshot,
        world_network: Some(world_network),
    };

    Ok((client, request, agent_id))
}

#[cfg(target_os = "macos")]
fn build_agent_client_and_request_impl(
    cmd: &str,
    parent_span_id: Option<&str>,
    parent_cmd_id: Option<&str>,
) -> anyhow::Result<(
    transport_api_client::AgentClient,
    transport_api_types::ExecuteRequest,
    String,
)> {
    // Allow explicit socket overrides (used by tests/fixtures and advanced setups).
    // When set, we bypass Lima detection/startup and connect directly.
    if let Some(socket_path) = std::env::var_os("SUBSTRATE_WORLD_SOCKET") {
        let socket_path = std::path::PathBuf::from(socket_path);
        let client = AgentClient::unix_socket(&socket_path)?;
        let cwd_path = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
        let cwd = cwd_path.display().to_string();
        let (mut env_map, inherit_from_host) = build_world_env_map_for_cwd(&cwd_path)?;
        if inherit_from_host {
            eprintln!("substrate: warning: world env is forwarding selected host env vars (world.env.inherit_from_host=true)");
        }
        normalize_env_for_linux_guest(&mut env_map);
        apply_macos_staged_workspace_project_dir_override(&mut env_map);
        ensure_world_deps_bin_on_path(&mut env_map);
        let agent_id = std::env::var("SUBSTRATE_AGENT_ID").unwrap_or_else(|_| "human".to_string());
        let network_policy = resolve_world_network_policy_for_cwd(&cwd_path)?;
        let world_network = request_world_network_routing(&network_policy);
        let policy_snapshot = network_policy.snapshot;
        crate::execution::policy_snapshot::inject_world_fs_enforcement_plan_env(
            &policy_snapshot,
            &mut env_map,
        )?;
        inject_process_trace_env(&mut env_map, parent_span_id, parent_cmd_id);

        let request = build_execute_request(ExecuteRequestInput {
            profile: current_world_request_profile(),
            cmd: cmd.to_string(),
            cwd,
            env_map,
            agent_id: agent_id.clone(),
            policy_snapshot,
            world_network,
            world_fs_mode: current_world_fs_mode(),
            member_dispatch: None,
            acceptance_context: None,
        });

        return Ok((client, request, agent_id));
    }

    let ctx = match pw::get_context() {
        Some(ctx) => ctx,
        None => {
            // Subcommands like `substrate health` may execute without going through the full shell
            // initialization path, so the platform world context might not be populated yet.
            let detected =
                pw::detect().map_err(|e| anyhow::anyhow!("platform world detect failed: {e:#}"))?;
            pw::store_context_globally(detected);
            pw::get_context().ok_or_else(|| anyhow::anyhow!("no platform world context"))?
        }
    };
    (ctx.ensure_ready.as_ref())()?;

    let client = match &ctx.transport {
        pw::WorldTransport::Unix(path) => AgentClient::unix_socket(path),
        pw::WorldTransport::Tcp { host, port } => AgentClient::tcp(host, *port),
        pw::WorldTransport::Vsock { port } => AgentClient::tcp("127.0.0.1", *port),
    }?;

    let cwd_path = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
    let cwd = cwd_path.display().to_string();
    let (mut env_map, inherit_from_host) = build_world_env_map_for_cwd(&cwd_path)?;
    if inherit_from_host {
        eprintln!("substrate: warning: world env is forwarding selected host env vars (world.env.inherit_from_host=true)");
    }
    normalize_env_for_linux_guest(&mut env_map);
    apply_macos_staged_workspace_project_dir_override(&mut env_map);
    ensure_world_deps_bin_on_path(&mut env_map);
    let agent_id = std::env::var("SUBSTRATE_AGENT_ID").unwrap_or_else(|_| "human".to_string());
    let network_policy = resolve_world_network_policy_for_cwd(&cwd_path)?;
    let world_network = request_world_network_routing(&network_policy);
    let policy_snapshot = network_policy.snapshot;
    crate::execution::policy_snapshot::inject_world_fs_enforcement_plan_env(
        &policy_snapshot,
        &mut env_map,
    )?;
    inject_process_trace_env(&mut env_map, parent_span_id, parent_cmd_id);

    let request = build_execute_request(ExecuteRequestInput {
        profile: current_world_request_profile(),
        cmd: cmd.to_string(),
        cwd,
        env_map,
        agent_id: agent_id.clone(),
        policy_snapshot,
        world_network,
        world_fs_mode: current_world_fs_mode(),
        member_dispatch: None,
        acceptance_context: None,
    });

    Ok((client, request, agent_id))
}

#[allow(dead_code)]
#[cfg(target_os = "macos")]
fn build_agent_client_and_member_dispatch_request_impl(
    dispatch: &MemberDispatchTransportRequest,
    cwd_path: &std::path::Path,
) -> anyhow::Result<(
    transport_api_client::AgentClient,
    transport_api_types::ExecuteRequest,
    String,
)> {
    if let Some(socket_path) = std::env::var_os("SUBSTRATE_WORLD_SOCKET") {
        let socket_path = std::path::PathBuf::from(socket_path);
        let client = AgentClient::unix_socket(&socket_path)?;
        let cwd_path = cwd_path.to_path_buf();
        let cwd = cwd_path.display().to_string();
        let (mut env_map, inherit_from_host) = build_world_env_map_for_cwd(&cwd_path)?;
        if inherit_from_host {
            eprintln!("substrate: warning: world env is forwarding selected host env vars (world.env.inherit_from_host=true)");
        }
        normalize_env_for_linux_guest(&mut env_map);
        apply_macos_staged_workspace_project_dir_override(&mut env_map);
        ensure_world_deps_bin_on_path(&mut env_map);
        let agent_id = std::env::var("SUBSTRATE_AGENT_ID").unwrap_or_else(|_| "human".to_string());
        let network_policy = dispatch.resolve_world_network_policy(&cwd_path)?;
        let world_network = request_world_network_routing(&network_policy);
        let policy_snapshot = network_policy.snapshot;
        crate::execution::policy_snapshot::inject_world_fs_enforcement_plan_env(
            &policy_snapshot,
            &mut env_map,
        )?;
        let request = build_execute_request(ExecuteRequestInput {
            profile: current_world_request_profile(),
            cmd: String::new(),
            cwd,
            env_map,
            agent_id: agent_id.clone(),
            policy_snapshot,
            world_network,
            world_fs_mode: current_world_fs_mode(),
            member_dispatch: Some(build_member_dispatch_payload(dispatch)),
            acceptance_context: None,
        });

        return Ok((client, request, agent_id));
    }

    let ctx = match pw::get_context() {
        Some(ctx) => ctx,
        None => {
            let detected =
                pw::detect().map_err(|e| anyhow::anyhow!("platform world detect failed: {e:#}"))?;
            pw::store_context_globally(detected);
            pw::get_context().ok_or_else(|| anyhow::anyhow!("no platform world context"))?
        }
    };
    (ctx.ensure_ready.as_ref())()?;

    let client = match &ctx.transport {
        pw::WorldTransport::Unix(path) => AgentClient::unix_socket(path),
        pw::WorldTransport::Tcp { host, port } => AgentClient::tcp(host, *port),
        pw::WorldTransport::Vsock { port } => AgentClient::tcp("127.0.0.1", *port),
    }?;

    let cwd_path = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
    let cwd = cwd_path.display().to_string();
    let (mut env_map, inherit_from_host) = build_world_env_map_for_cwd(&cwd_path)?;
    if inherit_from_host {
        eprintln!("substrate: warning: world env is forwarding selected host env vars (world.env.inherit_from_host=true)");
    }
    normalize_env_for_linux_guest(&mut env_map);
    apply_macos_staged_workspace_project_dir_override(&mut env_map);
    ensure_world_deps_bin_on_path(&mut env_map);
    let agent_id = std::env::var("SUBSTRATE_AGENT_ID").unwrap_or_else(|_| "human".to_string());
    let network_policy = dispatch.resolve_world_network_policy(&cwd_path)?;
    let world_network = request_world_network_routing(&network_policy);
    let policy_snapshot = network_policy.snapshot;
    crate::execution::policy_snapshot::inject_world_fs_enforcement_plan_env(
        &policy_snapshot,
        &mut env_map,
    )?;
    let request = build_execute_request(ExecuteRequestInput {
        profile: current_world_request_profile(),
        cmd: String::new(),
        cwd,
        env_map,
        agent_id: agent_id.clone(),
        policy_snapshot,
        world_network,
        world_fs_mode: current_world_fs_mode(),
        member_dispatch: Some(build_member_dispatch_payload(dispatch)),
        acceptance_context: None,
    });

    Ok((client, request, agent_id))
}

#[cfg(target_os = "macos")]
fn build_agent_client_and_pending_diff_request_impl() -> anyhow::Result<(
    transport_api_client::AgentClient,
    transport_api_types::PendingDiffRequestV1,
    String,
)> {
    // Allow explicit socket overrides (used by tests/fixtures and advanced setups).
    // When set, we bypass Lima detection/startup and connect directly.
    if let Some(socket_path) = std::env::var_os("SUBSTRATE_WORLD_SOCKET") {
        let socket_path = std::path::PathBuf::from(socket_path);
        let client = AgentClient::unix_socket(&socket_path)?;
        let cwd_path = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
        let cwd = cwd_path.display().to_string();
        let (mut env_map, _inherit_from_host) = build_world_env_map_for_cwd(&cwd_path)?;
        normalize_env_for_linux_guest(&mut env_map);
        ensure_world_deps_bin_on_path(&mut env_map);
        let agent_id = std::env::var("SUBSTRATE_AGENT_ID").unwrap_or_else(|_| "human".to_string());
        let network_policy = resolve_world_network_policy_for_cwd(&cwd_path)?;
        let world_network = request_world_network_routing(&network_policy);
        let policy_snapshot = network_policy.snapshot;
        crate::execution::policy_snapshot::inject_world_fs_enforcement_plan_env(
            &policy_snapshot,
            &mut env_map,
        )?;

        let request = transport_api_types::PendingDiffRequestV1 {
            profile: current_world_request_profile(),
            cwd: Some(cwd),
            env: Some(env_map),
            agent_id: agent_id.clone(),
            policy_snapshot,
            world_network: Some(world_network),
        };

        return Ok((client, request, agent_id));
    }

    let ctx = match pw::get_context() {
        Some(ctx) => ctx,
        None => {
            let detected =
                pw::detect().map_err(|e| anyhow::anyhow!("platform world detect failed: {e:#}"))?;
            pw::store_context_globally(detected);
            pw::get_context().ok_or_else(|| anyhow::anyhow!("no platform world context"))?
        }
    };
    (ctx.ensure_ready.as_ref())()?;

    let client = match &ctx.transport {
        pw::WorldTransport::Unix(path) => AgentClient::unix_socket(path),
        pw::WorldTransport::Tcp { host, port } => AgentClient::tcp(host, *port),
        pw::WorldTransport::Vsock { port } => AgentClient::tcp("127.0.0.1", *port),
    }?;

    let cwd_path = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
    let cwd = cwd_path.display().to_string();
    let (mut env_map, _inherit_from_host) = build_world_env_map_for_cwd(&cwd_path)?;
    normalize_env_for_linux_guest(&mut env_map);
    ensure_world_deps_bin_on_path(&mut env_map);
    let agent_id = std::env::var("SUBSTRATE_AGENT_ID").unwrap_or_else(|_| "human".to_string());
    let network_policy = resolve_world_network_policy_for_cwd(&cwd_path)?;
    let world_network = request_world_network_routing(&network_policy);
    let policy_snapshot = network_policy.snapshot;
    crate::execution::policy_snapshot::inject_world_fs_enforcement_plan_env(
        &policy_snapshot,
        &mut env_map,
    )?;

    let request = transport_api_types::PendingDiffRequestV1 {
        profile: current_world_request_profile(),
        cwd: Some(cwd),
        env: Some(env_map),
        agent_id: agent_id.clone(),
        policy_snapshot,
        world_network: Some(world_network),
    };

    Ok((client, request, agent_id))
}

#[cfg(target_os = "windows")]
fn build_agent_client_and_request_impl(
    cmd: &str,
    parent_span_id: Option<&str>,
    parent_cmd_id: Option<&str>,
) -> anyhow::Result<(
    transport_api_client::AgentClient,
    transport_api_types::ExecuteRequest,
    String,
)> {
    use crate::execution::platform_world::windows;
    #[cfg(test)]
    let _env_guard = world_env_guard();

    let client = windows::build_agent_client()?;
    let cwd = windows::current_dir_wsl()?;
    let host_cwd = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
    let network_policy = resolve_world_network_policy_for_cwd(&host_cwd)?;
    mark_windows_world_dispatch_active_without_session();

    let profile = current_world_request_profile();
    let (mut env_map, inherit_from_host) = build_world_env_map_for_cwd(&host_cwd)?;
    if inherit_from_host {
        eprintln!("substrate: warning: world env is forwarding selected host env vars (world.env.inherit_from_host=true)");
    }
    normalize_env_for_linux_guest(&mut env_map);
    if profile.as_deref() == Some("world-deps-provision") {
        env_map.retain(|k, _| {
            k == "PATH"
                || k == "HOME"
                || k.starts_with("SUBSTRATE_")
                || k.starts_with("WORLD_")
                || k.starts_with("SHIM_")
        });
    }
    let agent_id = std::env::var("SUBSTRATE_AGENT_ID").unwrap_or_else(|_| "human".to_string());
    let world_network = request_world_network_routing(&network_policy);
    let policy_snapshot = network_policy.snapshot;
    crate::execution::policy_snapshot::inject_world_fs_enforcement_plan_env(
        &policy_snapshot,
        &mut env_map,
    )?;
    inject_process_trace_env(&mut env_map, parent_span_id, parent_cmd_id);

    let request = build_execute_request(ExecuteRequestInput {
        profile,
        cmd: cmd.to_string(),
        cwd,
        env_map,
        agent_id: agent_id.clone(),
        policy_snapshot,
        world_network,
        world_fs_mode: current_world_fs_mode(),
        member_dispatch: None,
        acceptance_context: None,
    });

    Ok((client, request, agent_id))
}

#[allow(dead_code)]
#[cfg(target_os = "windows")]
fn build_agent_client_and_member_dispatch_request_impl(
    dispatch: &MemberDispatchTransportRequest,
    cwd_path: &std::path::Path,
) -> anyhow::Result<(
    transport_api_client::AgentClient,
    transport_api_types::ExecuteRequest,
    String,
)> {
    use crate::execution::platform_world::windows;
    #[cfg(test)]
    let _env_guard = world_env_guard();

    let client = windows::build_agent_client()?;
    let cwd = windows::current_dir_wsl()?;
    let host_cwd = cwd_path.to_path_buf();
    let network_policy = dispatch.resolve_world_network_policy(&host_cwd)?;
    mark_windows_world_dispatch_active_without_session();

    let profile = current_world_request_profile();
    let (mut env_map, inherit_from_host) = build_world_env_map_for_cwd(&host_cwd)?;
    if inherit_from_host {
        eprintln!("substrate: warning: world env is forwarding selected host env vars (world.env.inherit_from_host=true)");
    }
    normalize_env_for_linux_guest(&mut env_map);
    if profile.as_deref() == Some("world-deps-provision") {
        env_map.retain(|k, _| {
            k == "PATH"
                || k == "HOME"
                || k.starts_with("SUBSTRATE_")
                || k.starts_with("WORLD_")
                || k.starts_with("SHIM_")
        });
    }
    let agent_id = std::env::var("SUBSTRATE_AGENT_ID").unwrap_or_else(|_| "human".to_string());
    let world_network = request_world_network_routing(&network_policy);
    let policy_snapshot = network_policy.snapshot;
    crate::execution::policy_snapshot::inject_world_fs_enforcement_plan_env(
        &policy_snapshot,
        &mut env_map,
    )?;
    let request = build_execute_request(ExecuteRequestInput {
        profile,
        cmd: String::new(),
        cwd,
        env_map,
        agent_id: agent_id.clone(),
        policy_snapshot,
        world_network,
        world_fs_mode: current_world_fs_mode(),
        member_dispatch: Some(build_member_dispatch_payload(dispatch)),
        acceptance_context: None,
    });

    Ok((client, request, agent_id))
}

#[cfg(target_os = "windows")]
fn build_agent_client_and_pending_diff_request_impl() -> anyhow::Result<(
    transport_api_client::AgentClient,
    transport_api_types::PendingDiffRequestV1,
    String,
)> {
    use crate::execution::platform_world::windows;
    #[cfg(test)]
    let _env_guard = world_env_guard();

    let client = windows::build_agent_client()?;
    mark_windows_world_dispatch_active_without_session();
    let cwd = windows::current_dir_wsl()?;
    let host_cwd = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
    let profile = current_world_request_profile();
    let (mut env_map, _inherit_from_host) = build_world_env_map_for_cwd(&host_cwd)?;
    normalize_env_for_linux_guest(&mut env_map);
    let agent_id = std::env::var("SUBSTRATE_AGENT_ID").unwrap_or_else(|_| "human".to_string());
    let network_policy = resolve_world_network_policy_for_cwd(&host_cwd)?;
    let world_network = request_world_network_routing(&network_policy);
    let policy_snapshot = network_policy.snapshot;
    crate::execution::policy_snapshot::inject_world_fs_enforcement_plan_env(
        &policy_snapshot,
        &mut env_map,
    )?;

    let request = transport_api_types::PendingDiffRequestV1 {
        profile,
        cwd: Some(cwd),
        env: Some(env_map),
        agent_id: agent_id.clone(),
        policy_snapshot,
        world_network: Some(world_network),
    };

    Ok((client, request, agent_id))
}

pub(crate) fn stream_non_pty_via_agent(
    command: &str,
    parent_span_id: Option<&str>,
    parent_cmd_id: Option<&str>,
    command_event_context: Option<ShellCommandEventContext>,
) -> anyhow::Result<AgentStreamOutcome> {
    let (client, request, agent_id) =
        build_agent_client_and_request_with_trace_metadata(command, parent_span_id, parent_cmd_id)?;

    let host_visible = request.policy_snapshot.world_fs.host_visible;
    let empty_env: std::collections::HashMap<String, String> = std::collections::HashMap::new();
    let env_map = request.env.as_ref().unwrap_or(&empty_env);
    let cwd = request
        .cwd
        .as_deref()
        .map(std::path::Path::new)
        .unwrap_or_else(|| std::path::Path::new("."));
    if let Some(deny) =
        substrate_common::world_exec_guard::check_command(&request.cmd, cwd, env_map, host_visible)
    {
        let message = substrate_common::world_exec_guard::deny_message(&deny);
        emit_stream_chunk_with_context(
            &agent_id,
            command_event_context.as_ref(),
            None,
            message.as_bytes(),
            true,
        );
        return Ok(AgentStreamOutcome {
            exit_code: 5,
            scopes_used: Vec::new(),
            fs_diff: None,
            fs_strategy: None,
            process_telemetry: ProcessTelemetry::default(),
        });
    }

    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(async move {
        #[cfg(target_os = "windows")]
        {
            use anyhow::Context as _;

            fn parse_timeout_ms(var: &str) -> Option<std::time::Duration> {
                std::env::var(var)
                    .ok()
                    .and_then(|v| v.trim().parse::<u64>().ok())
                    .map(std::time::Duration::from_millis)
            }

            let timeout = parse_timeout_ms("SUBSTRATE_WSL_AGENT_EXEC_TIMEOUT_MS")
                .unwrap_or_else(|| std::time::Duration::from_secs(120));
            let requested_shared_world = request.shared_world.clone();

            let response = tokio::time::timeout(timeout, async {
                client
                    .execute(request)
                    .await
                    .context("world-service /v1/execute request failed")
            })
            .await
            .with_context(|| {
                format!(
                    "Timed out after {}s waiting for world-service /v1/execute (transport: {}).\nHint: ensure the Windows named-pipe forwarder is running (\\\\.\\pipe\\substrate-agent) and the WSL agent is healthy (try `pwsh -File scripts/windows/wsl-warm.ps1 -DistroName substrate-wsl`).",
                    timeout.as_secs(),
                    client.transport().description()
                )
            })??;
            validate_execute_response_shared_world(requested_shared_world.as_ref(), &response)?;
            let stdout = BASE64
                .decode(response.stdout_b64.as_bytes())
                .unwrap_or_else(|_| response.stdout_b64.clone().into_bytes());
            let stderr = BASE64
                .decode(response.stderr_b64.as_bytes())
                .unwrap_or_else(|_| response.stderr_b64.clone().into_bytes());
            emit_stream_chunk_with_context(
                &agent_id,
                command_event_context.as_ref(),
                None,
                &stdout,
                false,
            );
            emit_stream_chunk_with_context(
                &agent_id,
                command_event_context.as_ref(),
                None,
                &stderr,
                true,
            );

            Ok(AgentStreamOutcome {
                exit_code: response.exit,
                scopes_used: response.scopes_used,
                fs_diff: response.fs_diff,
                fs_strategy: None,
                process_telemetry: ProcessTelemetry::not_supported_platform(),
            })
        }

        #[cfg(not(target_os = "windows"))]
        {
            use transport_api_types::ApiError;
            use http_body_util::BodyExt;

            let response = client.execute_stream(request).await?;
            if !response.status().is_success() {
                let status = response.status();
                let body_bytes = response
                    .into_body()
                    .collect()
                    .await
                    .map_err(|e| anyhow::anyhow!("stream read failed: {}", e))?
                    .to_bytes();
                if let Ok(api_error) = serde_json::from_slice::<ApiError>(&body_bytes) {
                    anyhow::bail!("API error: {}", api_error);
                }
                let text = String::from_utf8_lossy(&body_bytes);
                anyhow::bail!("HTTP {} error: {}", status, text);
            }

            let (sigint_tx, mut sigint_rx) = tokio::sync::mpsc::unbounded_channel::<()>();
            let sigint_task = tokio::spawn(async move {
                loop {
                    if tokio::signal::ctrl_c().await.is_err() {
                        break;
                    }
                    if sigint_tx.send(()).is_err() {
                        break;
                    }
                }
            });

            let result = process_agent_stream(
                response.into_body(),
                agent_id,
                command_event_context,
                &mut sigint_rx,
                |span_id, sig| async {
                    client
                        .cancel_execute(ExecuteCancelRequestV1 { span_id, sig })
                        .await
                        .map(|_| ())
                },
            )
            .await;
            sigint_task.abort();
            result
        }
    })
}

#[cfg(not(target_os = "windows"))]
async fn process_agent_stream<Fut>(
    body: hyper::body::Incoming,
    agent_label: String,
    command_event_context: Option<ShellCommandEventContext>,
    sigint_rx: &mut tokio::sync::mpsc::UnboundedReceiver<()>,
    cancel: impl FnMut(String, String) -> Fut,
) -> anyhow::Result<AgentStreamOutcome>
where
    Fut: std::future::Future<Output = anyhow::Result<()>>,
{
    process_agent_stream_body(
        body,
        agent_label,
        command_event_context.as_ref(),
        sigint_rx,
        cancel,
    )
    .await
}

#[cfg(not(target_os = "windows"))]
async fn process_agent_stream_body<B, Fut>(
    body: B,
    agent_label: String,
    command_event_context: Option<&ShellCommandEventContext>,
    sigint_rx: &mut tokio::sync::mpsc::UnboundedReceiver<()>,
    mut cancel: impl FnMut(String, String) -> Fut,
) -> anyhow::Result<AgentStreamOutcome>
where
    B: hyper::body::Body<Data = hyper::body::Bytes>,
    B::Error: std::fmt::Display,
    Fut: std::future::Future<Output = anyhow::Result<()>>,
{
    use http_body_util::BodyExt;
    let mut body = std::pin::pin!(body);

    let mut buffer = Vec::new();
    let mut exit_code = None;
    let mut scopes_used = Vec::new();
    let mut fs_diff = None;
    let mut fs_strategy = None;
    let mut process_telemetry = ProcessTelemetry::default();
    let mut active_span_id: Option<String> = None;

    loop {
        while sigint_rx.try_recv().is_ok() {
            if let Some(span_id) = active_span_id.as_deref() {
                let cancel_span_id: String = span_id.to_owned();
                if let Err(err) = cancel(cancel_span_id, "INT".to_string()).await {
                    eprintln!("substrate: warn: failed to interrupt world command: {err:#}");
                }
            }
        }

        let frame = match tokio::time::timeout(
            std::time::Duration::from_millis(100),
            body.as_mut().frame(),
        )
        .await
        {
            Ok(frame) => frame,
            Err(_) => continue,
        };

        let Some(frame) = frame else {
            break;
        };
        let frame = frame.map_err(|e| anyhow::anyhow!("stream frame error: {}", e))?;
        if let Some(data) = frame.data_ref() {
            buffer.extend_from_slice(data);
            consume_agent_stream_buffer_with_context(
                &agent_label,
                command_event_context,
                &mut buffer,
                &mut active_span_id,
                &mut exit_code,
                &mut scopes_used,
                &mut fs_diff,
                &mut fs_strategy,
                &mut process_telemetry,
            )?;
            if exit_code.is_some() {
                break;
            }
        }
    }

    if exit_code.is_none() && !buffer.is_empty() {
        consume_agent_stream_buffer_with_context(
            &agent_label,
            command_event_context,
            &mut buffer,
            &mut active_span_id,
            &mut exit_code,
            &mut scopes_used,
            &mut fs_diff,
            &mut fs_strategy,
            &mut process_telemetry,
        )?;
    }

    let exit_code =
        exit_code.ok_or_else(|| anyhow::anyhow!("agent stream completed without exit frame"))?;

    Ok(AgentStreamOutcome {
        exit_code,
        scopes_used,
        fs_diff,
        fs_strategy,
        process_telemetry,
    })
}

#[allow(dead_code)]
pub(crate) fn consume_agent_stream_buffer(
    agent_label: &str,
    buffer: &mut Vec<u8>,
    exit_code: &mut Option<i32>,
    scopes_used: &mut Vec<String>,
    fs_diff: &mut Option<substrate_common::FsDiff>,
) -> anyhow::Result<()> {
    let mut ignored = None;
    let mut process_telemetry = ProcessTelemetry::default();
    consume_agent_stream_buffer_with_meta(
        agent_label,
        buffer,
        &mut None,
        exit_code,
        scopes_used,
        fs_diff,
        &mut ignored,
        &mut process_telemetry,
    )
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn consume_agent_stream_buffer_with_meta(
    agent_label: &str,
    buffer: &mut Vec<u8>,
    active_span_id: &mut Option<String>,
    exit_code: &mut Option<i32>,
    scopes_used: &mut Vec<String>,
    fs_diff: &mut Option<substrate_common::FsDiff>,
    fs_strategy: &mut Option<WorldFsStrategyTraceMeta>,
    process_telemetry: &mut ProcessTelemetry,
) -> anyhow::Result<()> {
    consume_agent_stream_buffer_with_context(
        agent_label,
        None,
        buffer,
        active_span_id,
        exit_code,
        scopes_used,
        fs_diff,
        fs_strategy,
        process_telemetry,
    )
}

#[allow(clippy::too_many_arguments)]
fn consume_agent_stream_buffer_with_context(
    agent_label: &str,
    command_event_context: Option<&ShellCommandEventContext>,
    buffer: &mut Vec<u8>,
    active_span_id: &mut Option<String>,
    exit_code: &mut Option<i32>,
    scopes_used: &mut Vec<String>,
    fs_diff: &mut Option<substrate_common::FsDiff>,
    fs_strategy: &mut Option<WorldFsStrategyTraceMeta>,
    process_telemetry: &mut ProcessTelemetry,
) -> anyhow::Result<()> {
    use anyhow::Context as _;

    while let Some(pos) = buffer.iter().position(|&b| b == b'\n') {
        let line: Vec<u8> = buffer.drain(..=pos).collect();
        if line.len() <= 1 {
            continue;
        }
        let payload = &line[..line.len() - 1];
        if payload.is_empty() {
            continue;
        }

        let frame: ExecuteStreamFrame = serde_json::from_slice(payload).with_context(|| {
            format!(
                "invalid agent stream frame: {}",
                String::from_utf8_lossy(payload)
            )
        })?;

        match frame {
            ExecuteStreamFrame::Start { span_id, .. } => {
                *active_span_id = Some(span_id);
            }
            ExecuteStreamFrame::Stdout { chunk_b64, .. } => {
                let bytes = BASE64
                    .decode(chunk_b64.as_bytes())
                    .map_err(|e| anyhow::anyhow!("invalid stdout chunk: {}", e))?;
                emit_stream_chunk_with_context(
                    agent_label,
                    command_event_context,
                    active_span_id.as_deref(),
                    &bytes,
                    false,
                );
            }
            ExecuteStreamFrame::Stderr { chunk_b64, .. } => {
                let bytes = BASE64
                    .decode(chunk_b64.as_bytes())
                    .map_err(|e| anyhow::anyhow!("invalid stderr chunk: {}", e))?;
                emit_stream_chunk_with_context(
                    agent_label,
                    command_event_context,
                    active_span_id.as_deref(),
                    &bytes,
                    true,
                );
            }
            ExecuteStreamFrame::Event { event, .. } => {
                if let (Some(primary), Some(final_strategy), Some(reason)) = (
                    event
                        .data
                        .get("world_fs_strategy_primary")
                        .and_then(serde_json::Value::as_str)
                        .and_then(substrate_common::WorldFsStrategy::parse),
                    event
                        .data
                        .get("world_fs_strategy_final")
                        .and_then(serde_json::Value::as_str)
                        .and_then(substrate_common::WorldFsStrategy::parse),
                    event
                        .data
                        .get("world_fs_strategy_fallback_reason")
                        .and_then(serde_json::Value::as_str)
                        .and_then(substrate_common::WorldFsStrategyFallbackReason::parse),
                ) {
                    *fs_strategy = Some(WorldFsStrategyTraceMeta {
                        primary,
                        final_strategy,
                        fallback_reason: reason,
                    });
                }
                let _ = publish_agent_event(event);
            }
            ExecuteStreamFrame::Exit {
                exit,
                scopes_used: scopes,
                fs_diff: diff,
                process_telemetry: exit_process_telemetry,
                ..
            } => {
                *exit_code = Some(exit);
                *scopes_used = scopes;
                *fs_diff = diff;
                *process_telemetry = exit_process_telemetry;
            }
            ExecuteStreamFrame::Error { message, .. } => {
                if message.contains("WORLD_FS_STRATEGY_UNAVAILABLE") {
                    return Err(anyhow::Error::new(WorldFsStrategyUnavailableError {
                        raw_message: message.clone(),
                        fallback_reason: parse_world_fs_strategy_unavailable_reason(&message),
                    }));
                }
                eprintln!("world-service error: {}", message);
                anyhow::bail!(message);
            }
        }
    }

    Ok(())
}

fn emit_stream_chunk_with_context(
    agent_label: &str,
    command_event_context: Option<&ShellCommandEventContext>,
    span_id: Option<&str>,
    data: &[u8],
    is_stderr: bool,
) {
    let (orchestration_session_id, run_id, effective_span_id) = match command_event_context {
        Some(context) => match context.run_id.as_deref() {
            Some(run_id) => (
                Some(context.emission.orchestration_session_id.as_str()),
                run_id,
                span_id.or(context.span_id.as_deref()),
            ),
            None => (None, "", None),
        },
        None => (None, "", None),
    };

    emit_stream_chunk(
        agent_label,
        orchestration_session_id,
        run_id,
        effective_span_id,
        data,
        is_stderr,
    );
}

pub(super) fn emit_stream_chunk(
    agent_label: &str,
    orchestration_session_id: Option<&str>,
    run_id: &str,
    span_id: Option<&str>,
    data: &[u8],
    is_stderr: bool,
) {
    use std::io::Write;

    if is_stderr {
        let mut stderr = io::stderr();
        let _ = stderr.write_all(data);
        let _ = stderr.flush();
    } else {
        let mut stdout = io::stdout();
        let _ = stdout.write_all(data);
        let _ = stdout.flush();
    }

    let Some(orchestration_session_id) = orchestration_session_id else {
        return;
    };
    let text = String::from_utf8_lossy(data);
    let mut event = AgentEvent::stream_chunk(
        agent_label,
        orchestration_session_id.to_string(),
        run_id.to_string(),
        is_stderr,
        text.to_string(),
    );
    event.span_id = span_id.map(|s| s.to_string());
    let _ = publish_agent_event(event);
}

#[cfg(all(test, any(target_os = "linux", target_os = "macos")))]
mod tests {
    #[cfg(target_os = "linux")]
    use super::{
        build_authenticated_world_deps_client_and_request_impl,
        classify_installed_product_activation_mode, ensure_world_service_ready_for_target,
        ensure_world_service_ready_for_target_with_spawn, ensure_world_service_ready_with,
        resolve_legacy_compatibility_candidate_bins, terminate_installed_product_observer_child,
        world_service_command, WorldServiceCandidateBins, WorldServiceReadinessPosture,
        WorldServiceSpawnEnvironment, AUTHENTICATED_SYSTEMCTL_BINARY,
        AUTHENTICATED_WORLD_SERVICE_BINARY, AUTHENTICATED_WORLD_SERVICE_CWD,
        AUTHENTICATED_WORLD_SERVICE_PATH, AUTHENTICATED_WORLD_SERVICE_UNIT,
        AUTHENTICATED_WORLD_SOCKET, AUTHENTICATED_WORLD_SOCKET_UNIT,
        WORLD_SERVICE_ACTIVATION_POLL_MS, WORLD_SERVICE_ACTIVATION_WAIT_MS,
        WORLD_SERVICE_PROBE_IO_TIMEOUT_MS, WORLD_SERVICE_READINESS_POLL_MS,
        WORLD_SERVICE_READINESS_WAIT_MS, WORLD_SERVICE_SYSTEMCTL_POLL_MS,
        WORLD_SERVICE_SYSTEMCTL_SHOW_TIMEOUT_MS,
    };
    use super::{
        build_execute_request, build_member_dispatch_payload, current_world_request_profile,
        emit_stream_chunk, ensure_world_deps_bin_on_path, extract_process_telemetry_from_ws_exit,
        maybe_inject_codex_auth_seed_home_for_policy, preserve_world_project_dir_override,
        process_agent_stream_body, ExecuteRequestInput, MemberDispatchTransportRequest, BASE64,
        SUBSTRATE_INTERNAL_CODEX_AUTH_SEED_HOME_ENV, WORLD_PROJECT_DIR_OVERRIDE_ENV,
    };
    use crate::execution::agent_events::{
        acquire_event_test_guard, clear_agent_event_sender, init_event_channel,
        ShellCommandEventContext, ShellEventEmissionContext,
    };
    use crate::execution::ExactDispatchPolicySnapshotMaterialV1;
    use base64::Engine;
    use futures::stream;
    use http_body_util::StreamBody;
    use serde_json::json;
    use std::convert::Infallible;
    use std::sync::{Arc, Mutex};
    use substrate_common::agent_events::AgentEventKind;
    use transport_api_types::{
        ExecuteStreamFrame, MemberRuntimeBackendKindV1, PlatformPrincipalV1, PolicySnapshotV3,
        PolicySnapshotWorldFsFailClosedV3, PolicySnapshotWorldFsV3, PolicySnapshotWorldFsWriteV3,
        ResolvedMemberRuntimeDescriptorV1, WorldFsMode, WorldNetworkRoutingV1,
    };

    fn e3a_projection_carrier() -> transport_api_types::ConfigProjectionActivationCarrierV1 {
        use transport_api_types::{
            ConfigProjectionActivationCarrierV1, ConfigProjectionRefV1, InWorldGatewayRefV1,
            ManagedGatewayActivationIntentRefV1,
        };
        let authority_store_id = "cpa_018f0f2e-7b4c-7aa1-8c22-123456789ab0".to_string();
        let series_id = "cps_018f0f2e-7b4c-7aa1-8c22-123456789ab1".to_string();
        ConfigProjectionActivationCarrierV1 {
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
        }
    }

    fn e3a_retained_launch_authority(
        participant_id: &str,
        run_id: &str,
    ) -> transport_api_types::RetainedWorkerLaunchAuthorityProofV1 {
        use transport_api_types::{
            RetainedWorkerAdmissionCommitmentCarrierV1, RetainedWorkerAuthorityObjectCommitmentV1,
            RetainedWorkerLaunchAuthorityProofV1, RetainedWorkerLaunchWorldBindingV1,
        };

        RetainedWorkerLaunchAuthorityProofV1 {
            schema_version: 1,
            authority_store_id: "has_e3a_fixture".to_string(),
            issuer_request_id: "request-e3a".to_string(),
            canonical_spawn_fingerprint: RetainedWorkerAdmissionCommitmentCarrierV1 {
                schema_version: 1,
                algorithm: "hmac-sha-256".to_string(),
                key_id: "adk_e3a_fixture".to_string(),
                digest_hex: "a".repeat(64),
            },
            registration_id: "rwr_e3a_fixture".to_string(),
            registration_commitment: RetainedWorkerAuthorityObjectCommitmentV1::CanonicalSha256 {
                digest_hex: "b".repeat(64),
            },
            authority_revision_after: 2,
            authority_record_commitment_after:
                RetainedWorkerAuthorityObjectCommitmentV1::CanonicalSha256 {
                    digest_hex: "c".repeat(64),
                },
            orchestration_session_id: "orch_e3a".to_string(),
            caller_participant_id: "orchestrator-e3a".to_string(),
            retained_participant_id: participant_id.to_string(),
            bootstrap_run_id: run_id.to_string(),
            transport_claim_id: "rtc_e3a_fixture".to_string(),
            backend_id: "cli:codex".to_string(),
            protocol: "substrate.agent.session".to_string(),
            world_binding: RetainedWorkerLaunchWorldBindingV1 {
                world_id: "world-e3a".to_string(),
                world_generation: 9,
            },
            current_policy_ref_id: "ao_policy_e3a_fixture".to_string(),
            current_policy_revision: "policy-e3a".to_string(),
            retained_worker_ref_id: "ao_worker_e3a_fixture".to_string(),
            retained_worker_commitment:
                RetainedWorkerAuthorityObjectCommitmentV1::CanonicalSha256 {
                    digest_hex: "d".repeat(64),
                },
        }
    }

    fn e3a_launch_activation(
        path: &str,
        participant_id: &str,
        run_id: &str,
        parent_participant_id: Option<&str>,
    ) -> transport_api_types::E2MemberLaunchActivationCarrierV1 {
        use sha2::Digest as _;
        use transport_api_types::{
            AuthorityObjectKindV1, DispatchPolicyCommitmentRefCarrierV1,
            E2DispatchPolicyReservationRefCarrierV1, E2LaunchRequestCommitmentV1,
            E2MemberLaunchActivationCarrierV1, E2MemberLaunchKindV1, OpaqueAuthorityCommitmentV1,
            PolicyRefV1, WorldBindingRefV1,
        };

        let launch_kind = if parent_participant_id.is_some() {
            E2MemberLaunchKindV1::Fork
        } else {
            E2MemberLaunchKindV1::FreshSpawn
        };
        let snapshot = PolicySnapshotV3 {
            schema_version: 3,
            net_allowed: vec!["api.example.com".to_string()],
            world_fs: PolicySnapshotWorldFsV3 {
                host_visible: false,
                fail_closed: PolicySnapshotWorldFsFailClosedV3 { routing: true },
                deny_enforcement: None,
                caged_required: true,
                discover: Some(transport_api_types::PolicySnapshotWorldFsDimensionV3 {
                    allow_list: vec!["exact".to_string()],
                    deny_list: Vec::new(),
                }),
                read: Some(transport_api_types::PolicySnapshotWorldFsDimensionV3 {
                    allow_list: vec!["exact".to_string()],
                    deny_list: Vec::new(),
                }),
                write: PolicySnapshotWorldFsWriteV3 {
                    enabled: false,
                    allow_list: vec!["exact".to_string()],
                    deny_list: Vec::new(),
                },
            },
        };
        let snapshot_bytes = serde_json::to_vec(&snapshot).expect("serialize E3-A policy fixture");
        let linkage_hash = "6".repeat(64);
        let commitment_ref = DispatchPolicyCommitmentRefCarrierV1 {
            authority_store_id: "authority-store-e3a".to_string(),
            commitment_id: "dpc_018f0f2e-7b4c-7aa1-8c22-123456789ab7".to_string(),
            exact_linkage_hash: linkage_hash.clone(),
        };
        let policy_ref = |suffix: char, digest: char| PolicyRefV1 {
            ref_id: format!("ao_{}", suffix.to_string().repeat(32)),
            object_kind: AuthorityObjectKindV1::Policy,
            schema_version: 1,
            commitment: OpaqueAuthorityCommitmentV1::CanonicalSha256 {
                digest_hex: digest.to_string().repeat(64),
            },
        };
        let (reservation_ref, request_commitment) = match launch_kind {
            E2MemberLaunchKindV1::FreshSpawn => (
                Some(E2DispatchPolicyReservationRefCarrierV1 {
                    authority_store_id: commitment_ref.authority_store_id.clone(),
                    reservation_id: format!("reservation-{path}"),
                    reservation_hash: "5".repeat(64),
                }),
                E2LaunchRequestCommitmentV1::HmacSha256 {
                    key_id: "request-key-e3a".to_string(),
                    domain: "substrate.e3a.test".to_string(),
                    digest_hex: "a".repeat(64),
                },
            ),
            E2MemberLaunchKindV1::Fork => (
                None,
                E2LaunchRequestCommitmentV1::CanonicalSha256 {
                    domain: "substrate.e3a.test".to_string(),
                    digest_hex: "a".repeat(64),
                },
            ),
        };
        let activation = E2MemberLaunchActivationCarrierV1 {
            schema_version: 1,
            activation_id: format!("e2a_{}", &linkage_hash[..32]),
            launch_kind,
            reservation_ref,
            commitment_ref: commitment_ref.clone(),
            immutable_worker_cap_ref: commitment_ref,
            immutable_worker_cap_created_revision: 1,
            immutable_worker_cap_application_revision: 2,
            policy_snapshot_bytes_base64: BASE64.encode(&snapshot_bytes),
            policy_snapshot_byte_length: snapshot_bytes.len() as u64,
            policy_snapshot_ref: policy_ref('8', '8'),
            policy_snapshot_hash: format!("{:x}", sha2::Sha256::digest(&snapshot_bytes)),
            policy_snapshot_revision: "policy-revision-e3a".to_string(),
            reason: Some("strict E3-A transport".to_string()),
            request_id: format!("request-{path}"),
            idempotency_key: format!("idempotency-{path}"),
            orchestration_session_id: "orch_e3a".to_string(),
            caller_participant_id: "orchestrator-e3a".to_string(),
            caller_backend_id: "cli:codex".to_string(),
            target_backend_id: "cli:codex".to_string(),
            retained_participant_id: participant_id.to_string(),
            bootstrap_run_id: run_id.to_string(),
            source_participant_id: parent_participant_id.map(str::to_string),
            target_world: WorldBindingRefV1 {
                world_id: "world-e3a".to_string(),
                world_generation: 9,
            },
            parent_policy_ref: policy_ref('9', '9'),
            parent_policy_revision: "parent-policy-revision-e3a".to_string(),
            request_commitment,
            registry_publication_revision: 2,
        };
        activation.validate().expect("valid E3-A activation");
        activation
    }

    fn e3a_transport_for_path(path: &str) -> MemberDispatchTransportRequest {
        let parent_participant_id = match path {
            "fork" => Some("source-fork"),
            "continue-fork" => Some("source-continue-fork"),
            "spawn" | "toolbox" => None,
            other => panic!("unsupported E3-A fixture path {other}"),
        };
        let participant_id = format!("member-{path}");
        let run_id = format!("run-{path}");
        let retained_worker_launch_authority = parent_participant_id
            .is_none()
            .then(|| e3a_retained_launch_authority(&participant_id, &run_id));
        MemberDispatchTransportRequest {
            orchestration_session_id: "orch_e3a".to_string(),
            participant_id: participant_id.clone(),
            orchestrator_participant_id: "orchestrator-e3a".to_string(),
            parent_participant_id: parent_participant_id.map(str::to_string),
            resumed_from_participant_id: None,
            backend_id: "cli:codex".to_string(),
            protocol: "substrate.agent.session".to_string(),
            run_id: run_id.clone(),
            world_id: "world-e3a".to_string(),
            world_generation: 9,
            initial_prompt: Some(path.to_string()),
            backend_kind: MemberRuntimeBackendKindV1::Codex,
            binary_path: "/usr/bin/codex".to_string(),
            retained_worker_launch_authority,
            e2_launch_activation: Some(e3a_launch_activation(
                path,
                &participant_id,
                &run_id,
                parent_participant_id,
            )),
            config_projection: Some(e3a_projection_carrier()),
            exact_policy_snapshot: None,
        }
    }

    #[cfg(target_os = "linux")]
    async fn read_e3a_http_request(stream: &mut tokio::net::UnixStream) -> (String, Vec<u8>) {
        use tokio::io::AsyncReadExt as _;

        let mut bytes = Vec::new();
        let mut header_end = None;
        let mut content_length = None;
        loop {
            let mut chunk = [0_u8; 1024];
            let count = stream.read(&mut chunk).await.expect("read E3-A request");
            assert!(count > 0, "E3-A client closed before a complete request");
            bytes.extend_from_slice(&chunk[..count]);
            if header_end.is_none() {
                if let Some(position) = bytes.windows(4).position(|part| part == b"\r\n\r\n") {
                    header_end = Some(position + 4);
                    let header = String::from_utf8_lossy(&bytes[..position + 4]);
                    content_length = header.lines().find_map(|line| {
                        let (name, value) = line.split_once(':')?;
                        name.eq_ignore_ascii_case("content-length")
                            .then(|| value.trim().parse::<usize>().ok())
                            .flatten()
                    });
                }
            }
            if let (Some(end), Some(length)) = (header_end, content_length) {
                if bytes.len() >= end + length {
                    return (
                        String::from_utf8_lossy(&bytes[..end]).into_owned(),
                        bytes[end..end + length].to_vec(),
                    );
                }
            }
        }
    }

    #[cfg(target_os = "linux")]
    async fn write_e3a_http_bad_request(stream: &mut tokio::net::UnixStream, message: &str) {
        use tokio::io::AsyncWriteExt as _;

        let body = serde_json::to_vec(&serde_json::json!({ "error": message }))
            .expect("serialize E3-A bad request");
        let header = format!(
            "HTTP/1.1 400 Bad Request\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
            body.len()
        );
        stream
            .write_all(header.as_bytes())
            .await
            .expect("write E3-A response header");
        stream
            .write_all(&body)
            .await
            .expect("write E3-A response body");
        stream.shutdown().await.expect("close E3-A response");
    }

    #[cfg(target_os = "linux")]
    struct PoisonedEnvGuard {
        previous: Vec<(&'static str, Option<std::ffi::OsString>)>,
    }

    #[cfg(target_os = "linux")]
    impl PoisonedEnvGuard {
        fn apply(entries: &[(&'static str, &str)]) -> Self {
            let previous = entries
                .iter()
                .map(|(key, _)| (*key, std::env::var_os(key)))
                .collect();
            for (key, value) in entries {
                std::env::set_var(key, value);
            }
            Self { previous }
        }
    }

    #[cfg(target_os = "linux")]
    impl Drop for PoisonedEnvGuard {
        fn drop(&mut self) {
            for (key, value) in self.previous.drain(..).rev() {
                match value {
                    Some(value) => std::env::set_var(key, value),
                    None => std::env::remove_var(key),
                }
            }
        }
    }

    #[cfg(target_os = "linux")]
    fn authenticated_world_deps_context_fixture() -> (
        tempfile::TempDir,
        std::path::PathBuf,
        crate::builtins::world_deps::AuthenticatedWorldDepsContextV1,
    ) {
        use std::fs;
        use std::os::unix::fs::PermissionsExt;

        let (principal, account_home) =
            crate::execution::install_bootstrap::current_unix_principal_and_home()
                .expect("resolve current Unix principal");
        let PlatformPrincipalV1::Unix { account, uid } = principal else {
            panic!("current Linux principal must use the Unix variant");
        };
        let temp = tempfile::Builder::new()
            .prefix("substrate-authenticated-f-builder-")
            .tempdir_in(account_home)
            .expect("create secure authenticated builder fixture");
        let selected_prefix = temp.path().join("selected-a");
        fs::create_dir(&selected_prefix).expect("create selected prefix");
        fs::set_permissions(&selected_prefix, fs::Permissions::from_mode(0o700))
            .expect("secure selected prefix");
        fs::write(
            selected_prefix.join("config.yaml"),
            "world:\n  enabled: true\n  deps:\n    builtins: disabled\n    inventory_mode: merged\n    enabled: []\n",
        )
        .expect("write selected config");
        fs::write(
            selected_prefix.join("policy.yaml"),
            "id: selected-policy\nnet_allowed: [selected.example]\nworld_fs:\n  host_visible: false\n  write:\n    enabled: false\n  fail_closed:\n    routing: true\n",
        )
        .expect("write selected policy");

        let launch_cwd = temp.path().join("workspace");
        fs::create_dir_all(launch_cwd.join(".substrate")).expect("create workspace metadata");
        fs::write(launch_cwd.join(".substrate/workspace.yaml"), "")
            .expect("write workspace marker");
        let install_context = transport_api_types::InstallBootstrapContextV1::new_unix(
            selected_prefix.to_str().expect("UTF-8 selected prefix"),
            &account,
            uid,
        )
        .expect("construct install context");
        let carrier =
            transport_api_types::InstallBootstrapContextCarrierV1::from_context(install_context)
                .expect("commit install context");
        let context = crate::builtins::world_deps::bind_authenticated_world_deps_context_v1(
            &carrier,
            &launch_cwd,
            &crate::execution::config_model::CliConfigOverrides::default(),
        )
        .expect("bind authenticated world-deps context");
        (temp, selected_prefix, context)
    }

    #[cfg(target_os = "linux")]
    fn manual_activation_mode() -> crate::execution::socket_activation::SocketActivationMode {
        crate::execution::socket_activation::SocketActivationMode::Manual
    }

    #[cfg(target_os = "linux")]
    fn unknown_activation_mode() -> crate::execution::socket_activation::SocketActivationMode {
        crate::execution::socket_activation::SocketActivationMode::Unknown
    }

    #[cfg(target_os = "linux")]
    fn socket_activation_mode() -> crate::execution::socket_activation::SocketActivationMode {
        crate::execution::socket_activation::SocketActivationMode::SocketActivation
    }

    #[cfg(target_os = "linux")]
    fn panic_activation_mode() -> crate::execution::socket_activation::SocketActivationMode {
        panic!("activation observation must not run after a successful capability probe")
    }

    #[cfg(target_os = "linux")]
    fn empty_candidate_bins() -> WorldServiceCandidateBins {
        std::array::from_fn(|_| None)
    }

    #[cfg(target_os = "linux")]
    fn panic_candidate_bins() -> WorldServiceCandidateBins {
        panic!("candidate resolution must not run")
    }

    #[cfg(target_os = "linux")]
    fn no_legacy_binary_candidates(
        activation_mode: fn() -> crate::execution::socket_activation::SocketActivationMode,
    ) -> WorldServiceReadinessPosture {
        WorldServiceReadinessPosture::LegacyCompatibility {
            socket_override_active: false,
            candidate_bins: empty_candidate_bins,
            activation_mode,
        }
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn compatibility_readiness_preserves_ambient_socket_and_lazy_binary_order() {
        use std::os::unix::fs::PermissionsExt;

        let _authority_env = crate::execution::AuthorityEnvTestGuard::preserve();
        let temp = tempfile::tempdir().expect("compatibility readiness fixture");
        let path_binary = temp.path().join("substrate-world-service");
        std::fs::write(&path_binary, "#!/bin/sh\nexit 0\n").expect("write PATH candidate");
        std::fs::set_permissions(&path_binary, std::fs::Permissions::from_mode(0o755))
            .expect("make PATH candidate executable");
        let socket = temp.path().join("compatibility.sock");
        let _poison = PoisonedEnvGuard::apply(&[
            ("SUBSTRATE_WORLD_SOCKET", socket.to_string_lossy().as_ref()),
            (
                "SUBSTRATE_WORLD_AGENT_BIN",
                "/explicit/compatibility/world-service",
            ),
            ("PATH", temp.path().to_string_lossy().as_ref()),
        ]);

        ensure_world_service_ready_with(|resolved_socket, posture| {
            assert_eq!(resolved_socket, socket);
            let WorldServiceReadinessPosture::LegacyCompatibility {
                socket_override_active,
                candidate_bins: resolve_candidate_bins,
                ..
            } = posture
            else {
                panic!("compatibility wrapper selected installed-product posture");
            };
            assert!(socket_override_active);
            assert_eq!(
                resolve_candidate_bins(),
                [
                    Some("/explicit/compatibility/world-service".to_string()),
                    Some(path_binary.display().to_string()),
                    Some("target/release/world-service".to_string()),
                    Some("target/debug/world-service".to_string()),
                ]
            );
            Ok(())
        })
        .expect("delegate resolved compatibility inputs");
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn readiness_contract_retains_published_timeout_and_polling_values() {
        assert_eq!(WORLD_SERVICE_PROBE_IO_TIMEOUT_MS, 150);
        assert_eq!(WORLD_SERVICE_ACTIVATION_POLL_MS, 100);
        assert_eq!(WORLD_SERVICE_ACTIVATION_WAIT_MS, 2_000);
        assert_eq!(WORLD_SERVICE_SYSTEMCTL_POLL_MS, 10);
        assert_eq!(WORLD_SERVICE_SYSTEMCTL_SHOW_TIMEOUT_MS, 2_000);
        assert_eq!(WORLD_SERVICE_READINESS_POLL_MS, 50);
        assert_eq!(WORLD_SERVICE_READINESS_WAIT_MS, 1_000);
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn explicit_readiness_fast_path_probes_only_the_supplied_socket() {
        use std::io::{Read, Write};
        use std::os::unix::net::UnixListener;

        let temp = tempfile::tempdir().expect("explicit readiness fixture");
        let socket = temp.path().join("ready.sock");
        let listener = UnixListener::bind(&socket).expect("bind explicit readiness socket");
        let server = std::thread::spawn(move || {
            let (mut stream, _) = listener.accept().expect("accept readiness probe");
            let mut request = [0_u8; 256];
            let count = stream.read(&mut request).expect("read readiness probe");
            assert!(String::from_utf8_lossy(&request[..count]).contains("GET /v1/capabilities"));
            stream
                .write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 0\r\n\r\n")
                .expect("write readiness response");
        });

        let _authority_env = crate::execution::AuthorityEnvTestGuard::preserve();
        let _poison = PoisonedEnvGuard::apply(&[
            ("SUBSTRATE_WORLD_SOCKET", "/poisoned/ambient/socket"),
            ("SUBSTRATE_WORLD_AGENT_BIN", "/poisoned/ambient/binary"),
            ("SUBSTRATE_SOCKET_ACTIVATION_OVERRIDE", "manual"),
            ("SUBSTRATE_SYSTEMCTL_TIMEOUT_MS", "1"),
        ]);

        ensure_world_service_ready_for_target(
            &socket,
            WorldServiceReadinessPosture::InstalledLinuxProduct {
                activation_mode: panic_activation_mode,
            },
        )
        .expect("explicit socket is ready");
        server.join().expect("join readiness server");
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn explicit_readiness_removes_stale_socket_only_in_manual_mode() {
        let temp = tempfile::tempdir().expect("stale readiness fixture");
        let manual_socket = temp.path().join("manual.sock");
        std::fs::write(&manual_socket, "stale").expect("write manual stale socket sentinel");
        let manual_error = ensure_world_service_ready_for_target(
            &manual_socket,
            no_legacy_binary_candidates(manual_activation_mode),
        )
        .expect_err("missing compatibility binary must fail");
        assert_eq!(manual_error.to_string(), "world-service binary not found");
        assert!(!manual_socket.exists());

        let unknown_socket = temp.path().join("unknown.sock");
        std::fs::write(&unknown_socket, "stale").expect("write unknown stale socket sentinel");
        let unknown_error = ensure_world_service_ready_for_target(
            &unknown_socket,
            no_legacy_binary_candidates(unknown_activation_mode),
        )
        .expect_err("missing compatibility binary must fail");
        assert_eq!(unknown_error.to_string(), "world-service binary not found");
        assert!(unknown_socket.exists());
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn compatibility_readiness_rejects_socket_override_with_preserved_error() {
        let temp = tempfile::tempdir().expect("override readiness fixture");
        let socket = temp.path().join("override.sock");
        let error = ensure_world_service_ready_for_target(
            &socket,
            WorldServiceReadinessPosture::LegacyCompatibility {
                socket_override_active: true,
                candidate_bins: panic_candidate_bins,
                activation_mode: manual_activation_mode,
            },
        )
        .expect_err("unresponsive compatibility override must fail");

        assert_eq!(
            error.to_string(),
            format!(
                "world backend unavailable (SUBSTRATE_WORLD_SOCKET override): {} did not respond",
                socket.display()
            )
        );
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn compatibility_readiness_spawns_selected_binary_and_preserves_poll_error() {
        use std::os::unix::fs::PermissionsExt;

        let _authority_env = crate::execution::AuthorityEnvTestGuard::preserve();
        let temp = tempfile::tempdir().expect("spawn readiness fixture");
        let socket = temp.path().join("never-ready.sock");
        let sentinel = temp.path().join("spawned");
        let binary = temp.path().join("world-service-sentinel");
        std::fs::write(
            &binary,
            format!(
                "#!/bin/sh\n[ \"${{SUBSTRATE_F3_LEGACY_INHERIT:-}}\" = inherited ] || exit 97\nprintf spawned > '{}'\n",
                sentinel.display()
            ),
        )
        .expect("write spawn sentinel binary");
        std::fs::set_permissions(&binary, std::fs::Permissions::from_mode(0o755))
            .expect("make spawn sentinel executable");
        let _candidate_env = PoisonedEnvGuard::apply(&[
            (
                "SUBSTRATE_WORLD_AGENT_BIN",
                binary.to_string_lossy().as_ref(),
            ),
            ("PATH", temp.path().to_string_lossy().as_ref()),
            ("SUBSTRATE_F3_LEGACY_INHERIT", "inherited"),
        ]);

        let error = ensure_world_service_ready_for_target(
            &socket,
            WorldServiceReadinessPosture::LegacyCompatibility {
                socket_override_active: false,
                candidate_bins: resolve_legacy_compatibility_candidate_bins,
                activation_mode: manual_activation_mode,
            },
        )
        .expect_err("sentinel binary never opens the readiness socket");

        assert_eq!(error.to_string(), "world-service readiness probe failed");
        assert_eq!(
            std::fs::read_to_string(&sentinel).expect("spawn sentinel output"),
            "spawned"
        );
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn compatibility_readiness_preserves_activation_wait_and_error() {
        let temp = tempfile::tempdir().expect("activation readiness fixture");
        let socket = temp.path().join("activation.sock");
        let started = std::time::Instant::now();

        let error = ensure_world_service_ready_for_target(
            &socket,
            WorldServiceReadinessPosture::LegacyCompatibility {
                socket_override_active: false,
                candidate_bins: panic_candidate_bins,
                activation_mode: socket_activation_mode,
            },
        )
        .expect_err("unresponsive socket activation must fail before spawn fallback");

        assert_eq!(
            error.to_string(),
            format!(
                "world-service socket activation detected but {} did not respond. Run 'systemctl status substrate-world-service.socket' for details.",
                socket.display()
            )
        );
        assert!(started.elapsed() >= std::time::Duration::from_millis(1_900));
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn installed_product_readiness_preserves_activation_wait_and_error() {
        let temp = tempfile::tempdir().expect("installed activation readiness fixture");
        let socket = temp.path().join("activation.sock");
        let started = std::time::Instant::now();

        let error = ensure_world_service_ready_for_target_with_spawn(
            &socket,
            WorldServiceReadinessPosture::InstalledLinuxProduct {
                activation_mode: socket_activation_mode,
            },
            |_| panic!("socket-activated installed product must not use spawn fallback"),
        )
        .expect_err("unresponsive socket activation must fail before spawn fallback");

        assert_eq!(
            error.to_string(),
            format!(
                "world-service socket activation detected but {} did not respond. Run 'systemctl status substrate-world-service.socket' for details.",
                socket.display()
            )
        );
        assert!(started.elapsed() >= std::time::Duration::from_millis(1_900));
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn installed_product_activation_classification_matches_fixed_unit_semantics() {
        use crate::execution::socket_activation::SocketActivationMode;

        for active_state in ["active", "listening", "running", "activating"] {
            assert_eq!(
                classify_installed_product_activation_mode(Some(active_state), false, false),
                SocketActivationMode::SocketActivation
            );
        }
        assert_eq!(
            classify_installed_product_activation_mode(Some("inactive"), false, false),
            SocketActivationMode::Unknown
        );
        assert_eq!(
            classify_installed_product_activation_mode(None, true, true),
            SocketActivationMode::Unknown
        );
        assert_eq!(
            classify_installed_product_activation_mode(None, false, true),
            SocketActivationMode::Manual
        );
        assert_eq!(
            classify_installed_product_activation_mode(None, true, false),
            SocketActivationMode::Manual
        );
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn installed_product_activation_observer_is_fixed_and_nonambient() {
        let source = include_str!("world_ops.rs");
        let start = source
            .find("fn observe_installed_product_unit_active_state")
            .expect("installed product observer definition");
        let end = source[start..]
            .find("fn resolve_legacy_compatibility_candidate_bins")
            .map(|offset| start + offset)
            .expect("observer definition boundary");
        let observer = &source[start..end];

        for required in [
            "Command::new(AUTHENTICATED_SYSTEMCTL_BINARY)",
            ".env_clear()",
            ".current_dir(\"/\")",
            "AUTHENTICATED_WORLD_SOCKET_UNIT",
            "AUTHENTICATED_WORLD_SERVICE_UNIT",
            "AUTHENTICATED_WORLD_SOCKET).exists()",
        ] {
            assert!(
                observer.contains(required),
                "missing fixed observer term: {required}"
            );
        }
        for forbidden in [
            "std::env",
            "env::var",
            "socket_activation_report",
            "which::which",
            "current_dir()",
            "SUBSTRATE_SOCKET_ACTIVATION_OVERRIDE",
            "SUBSTRATE_SYSTEMCTL_TIMEOUT_MS",
            "SUBSTRATE_WORLD_SOCKET\"",
        ] {
            assert!(
                !observer.contains(forbidden),
                "installed product observer contains ambient term: {forbidden}"
            );
        }
        assert_eq!(AUTHENTICATED_SYSTEMCTL_BINARY, "/usr/bin/systemctl");
        assert_eq!(
            AUTHENTICATED_WORLD_SOCKET_UNIT,
            "substrate-world-service.socket"
        );
        assert_eq!(
            AUTHENTICATED_WORLD_SERVICE_UNIT,
            "substrate-world-service.service"
        );
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn installed_product_observer_cleanup_terminates_and_reaps_child() {
        let mut child = std::process::Command::new("/usr/bin/sleep")
            .arg("30")
            .env_clear()
            .spawn()
            .expect("spawn fixed cleanup fixture");

        terminate_installed_product_observer_child(&mut child);

        assert!(
            child
                .try_wait()
                .expect("poll reaped fixed cleanup fixture")
                .is_some(),
            "cleanup must terminate and reap the observer child"
        );
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn installed_product_readiness_ignores_ambient_activation_and_binary_selectors() {
        let _authority_env = crate::execution::AuthorityEnvTestGuard::preserve();
        let temp = tempfile::tempdir().expect("installed-product readiness fixture");
        let socket = temp.path().join("missing-installed.sock");
        let poison_binary = temp.path().join("poison-binary");
        let poison_sentinel = temp.path().join("poison-spawned");
        std::fs::write(
            &poison_binary,
            format!(
                "#!/bin/sh\nprintf poisoned > '{}'\n",
                poison_sentinel.display()
            ),
        )
        .expect("write poison binary");
        let _poison = PoisonedEnvGuard::apply(&[
            ("SUBSTRATE_WORLD_SOCKET", "/poisoned/ambient/socket"),
            (
                "SUBSTRATE_WORLD_AGENT_BIN",
                poison_binary.to_string_lossy().as_ref(),
            ),
            ("SUBSTRATE_SOCKET_ACTIVATION_OVERRIDE", "socket_activation"),
            ("SUBSTRATE_SYSTEMCTL_TIMEOUT_MS", "1"),
            ("PATH", temp.path().to_string_lossy().as_ref()),
        ]);
        let spawn_calls = std::cell::Cell::new(0_u8);
        std::fs::write(&socket, "stale").expect("write installed stale socket sentinel");

        let error = ensure_world_service_ready_for_target_with_spawn(
            &socket,
            WorldServiceReadinessPosture::InstalledLinuxProduct {
                activation_mode: manual_activation_mode,
            },
            |spawn_plan| {
                spawn_calls.set(spawn_calls.get() + 1);
                assert_eq!(
                    spawn_plan.candidate_bins,
                    [
                        Some(AUTHENTICATED_WORLD_SERVICE_BINARY.to_string()),
                        None,
                        None,
                        None,
                    ]
                );
                assert_eq!(
                    spawn_plan.environment,
                    WorldServiceSpawnEnvironment::InstalledLinuxProduct
                );
                Ok(())
            },
        )
        .expect_err("injected spawn does not open the readiness socket");

        assert_eq!(error.to_string(), "world-service readiness probe failed");
        assert_eq!(spawn_calls.get(), 1);
        assert!(!socket.exists());
        assert!(!poison_sentinel.exists());
        assert_eq!(
            WorldServiceReadinessPosture::InstalledLinuxProduct {
                activation_mode: manual_activation_mode,
            }
            .into_spawn_plan(&socket)
            .expect("installed candidate selection")
            .candidate_bins,
            [
                Some(AUTHENTICATED_WORLD_SERVICE_BINARY.to_string()),
                None,
                None,
                None,
            ]
        );
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn installed_product_spawn_clears_parent_environment_and_fixes_target() {
        use std::os::unix::fs::PermissionsExt;

        let _authority_env = crate::execution::AuthorityEnvTestGuard::preserve();
        let temp = tempfile::tempdir().expect("installed spawn environment fixture");
        let binary = temp.path().join("inspect-installed-environment");
        let cwd_output = temp.path().join("cwd");
        let clean_output = temp.path().join("clean");
        std::fs::write(
            &binary,
            format!(
                "#!/bin/sh\n\
                 [ \"${{SUBSTRATE_WORLD_SOCKET:-}}\" = '{AUTHENTICATED_WORLD_SOCKET}' ] || exit 91\n\
                 [ \"${{PATH:-}}\" = '{AUTHENTICATED_WORLD_SERVICE_PATH}' ] || exit 92\n\
                 [ -z \"${{SUBSTRATE_WORLD_AGENT_BIN+x}}\" ] || exit 93\n\
                 [ -z \"${{WORLD_ARBITRARY_F_POISON+x}}\" ] || exit 94\n\
                 [ -z \"${{OPENAI_API_KEY+x}}\" ] || exit 95\n\
                 pwd > '{}'\n\
                 printf clean > '{}'\n",
                cwd_output.display(),
                clean_output.display()
            ),
        )
        .expect("write installed spawn inspector");
        std::fs::set_permissions(&binary, std::fs::Permissions::from_mode(0o755))
            .expect("make installed spawn inspector executable");
        let _poison = PoisonedEnvGuard::apply(&[
            ("SUBSTRATE_WORLD_SOCKET", "/poisoned/ambient/socket"),
            ("SUBSTRATE_WORLD_AGENT_BIN", "/poisoned/ambient/binary"),
            ("WORLD_ARBITRARY_F_POISON", "poisoned"),
            ("OPENAI_API_KEY", "poisoned-secret"),
            ("PATH", "/poisoned/ambient/path"),
        ]);

        let status = world_service_command(
            binary.to_str().expect("UTF-8 test binary"),
            WorldServiceSpawnEnvironment::InstalledLinuxProduct,
            temp.path(),
        )
        .status()
        .expect("run installed spawn inspector");

        assert!(status.success());
        assert_eq!(
            std::fs::read_to_string(cwd_output)
                .expect("read installed cwd")
                .trim(),
            temp.path().to_str().expect("UTF-8 test cwd")
        );
        assert_eq!(
            std::fs::read_to_string(clean_output).expect("read clean sentinel"),
            "clean"
        );
        assert_eq!(AUTHENTICATED_WORLD_SERVICE_CWD, "/var/lib/substrate");
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn authenticated_builder_uses_fixed_readiness_target_and_exact_generated_environment() {
        let _authority_env = crate::execution::AuthorityEnvTestGuard::preserve();
        let (_temp, _selected_prefix, context) = authenticated_world_deps_context_fixture();
        let marker = "poison-marker-a1-f3-environment";
        let _poison = PoisonedEnvGuard::apply(&[
            ("SUBSTRATE_HOME", marker),
            ("SUBSTRATE_ROOT", marker),
            ("SUBSTRATE_WORLD_SOCKET", marker),
            ("SUBSTRATE_WORLD_AGENT_BIN", marker),
            ("SUBSTRATE_ARBITRARY_F_POISON", marker),
            ("WORLD_ARBITRARY_F_POISON", marker),
            ("CODEX_HOME", marker),
            ("HOME", marker),
            ("XDG_CONFIG_HOME", marker),
            ("XDG_DATA_HOME", marker),
            ("XDG_CACHE_HOME", marker),
            ("LC_ALL", marker),
            ("OPENAI_API_KEY", marker),
            ("AUTHORIZATION", marker),
            ("PRIVATE_KEY", marker),
            ("SUBSTRATE_INSTALL_BOOTSTRAP_CONTEXT_V1", marker),
            ("SHIM_CALL_STACK", marker),
            ("SUBSTRATE_POLICY_MODE", marker),
            ("SUBSTRATE_WORLD_REQUEST_PROFILE", marker),
            ("SUBSTRATE_WORLD_DEPS_GUEST_BIN_DIR", marker),
            ("SUBSTRATE_AGENT_ID", marker),
        ]);
        let mut readiness_calls = 0;

        let (_, request, agent_id) = build_authenticated_world_deps_client_and_request_impl(
            &context,
            "printf authenticated",
            None,
            "world-deps-probe",
            |socket, posture| {
                readiness_calls += 1;
                assert_eq!(socket, std::path::Path::new(AUTHENTICATED_WORLD_SOCKET));
                assert!(matches!(
                    posture,
                    WorldServiceReadinessPosture::InstalledLinuxProduct { .. }
                ));
                assert_eq!(
                    AUTHENTICATED_WORLD_SERVICE_BINARY,
                    "/usr/local/bin/substrate-world-service"
                );
                Ok(())
            },
        )
        .expect("build authenticated request");

        assert_eq!(readiness_calls, 1);
        assert_eq!(agent_id, "human");
        assert_eq!(request.agent_id, "human");
        assert_eq!(request.profile.as_deref(), Some("world-deps-probe"));
        assert_eq!(request.cwd.as_deref(), context.launch_cwd().to_str());
        assert_eq!(
            serde_json::to_value(&request.policy_snapshot).expect("serialize request policy"),
            serde_json::to_value(&context.runtime_network_policy().snapshot)
                .expect("serialize context policy")
        );
        assert_eq!(
            request.world_network,
            Some(
                crate::execution::policy_snapshot::request_world_network_routing(
                    context.runtime_network_policy()
                )
            )
        );
        assert_eq!(request.world_fs_mode, Some(context.world_fs_policy().mode));
        let expected = std::collections::HashMap::from([
            (
                "SUBSTRATE_WORLD_DEPS_GUEST_BIN_DIR".to_string(),
                "/var/lib/substrate/world-deps/bin".to_string(),
            ),
            (
                "PATH".to_string(),
                "/var/lib/substrate/world-deps/bin:/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin".to_string(),
            ),
            ("HOME".to_string(), "/root".to_string()),
            ("XDG_CONFIG_HOME".to_string(), "/root/.config".to_string()),
            (
                "XDG_DATA_HOME".to_string(),
                "/root/.local/share".to_string(),
            ),
            (
                "XDG_CACHE_HOME".to_string(),
                "/root/.cache".to_string(),
            ),
            ("TERM".to_string(), "xterm-256color".to_string()),
        ]);
        assert_eq!(request.env.as_ref(), Some(&expected));
        assert_eq!(
            request.env.as_ref().map(std::collections::HashMap::len),
            Some(7)
        );
        let serialized = serde_json::to_string(&request).expect("serialize authenticated request");
        assert!(!serialized.contains(marker));
        assert!(!format!("{request:?}").contains(marker));
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn authenticated_builder_rejects_invalid_authority_before_readiness_or_construction() {
        use std::os::unix::fs::PermissionsExt;

        let _authority_env = crate::execution::AuthorityEnvTestGuard::preserve();
        let (temp, selected_prefix, context) = authenticated_world_deps_context_fixture();
        let retired = temp.path().join("retired-selected-a");
        std::fs::rename(&selected_prefix, &retired).expect("replace selected authority root");
        std::fs::create_dir(&selected_prefix).expect("create replacement authority root");
        std::fs::set_permissions(&selected_prefix, std::fs::Permissions::from_mode(0o700))
            .expect("secure replacement authority root");
        let readiness_called = std::cell::Cell::new(false);

        let error = build_authenticated_world_deps_client_and_request_impl(
            &context,
            "true",
            None,
            "world-deps-probe",
            |_, _| {
                readiness_called.set(true);
                Ok(())
            },
        )
        .err()
        .expect("replaced authority root must fail closed");

        assert!(!readiness_called.get());
        assert!(format!("{error:#}").contains("authenticated dependency root changed"));
        assert!(!format!("{error:#}").contains("poison-marker"));
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn authenticated_builder_rejects_profile_and_relative_cwd_before_readiness() {
        use std::os::unix::ffi::OsStringExt;

        let _authority_env = crate::execution::AuthorityEnvTestGuard::preserve();
        let (_temp, _selected_prefix, context) = authenticated_world_deps_context_fixture();
        let calls = std::cell::Cell::new(0_u8);

        let profile_error = build_authenticated_world_deps_client_and_request_impl(
            &context,
            "true",
            None,
            "ambient-profile",
            |_, _| {
                calls.set(calls.get() + 1);
                Ok(())
            },
        )
        .err()
        .expect("non-reserved authenticated profile must fail");
        assert!(profile_error
            .to_string()
            .contains("invalid authenticated world-deps request profile"));
        let cwd_error = build_authenticated_world_deps_client_and_request_impl(
            &context,
            "true",
            Some(std::path::Path::new("relative")),
            "world-deps-probe",
            |_, _| {
                calls.set(calls.get() + 1);
                Ok(())
            },
        )
        .err()
        .expect("relative authenticated cwd must fail");
        assert!(cwd_error
            .to_string()
            .contains("authenticated world-deps request cwd must be absolute"));
        let non_utf8_cwd = std::path::PathBuf::from(std::ffi::OsString::from_vec(vec![
            b'/', b't', b'm', b'p', b'/', 0xff,
        ]));
        let utf8_error = build_authenticated_world_deps_client_and_request_impl(
            &context,
            "true",
            Some(&non_utf8_cwd),
            "world-deps-probe",
            |_, _| {
                calls.set(calls.get() + 1);
                Ok(())
            },
        )
        .err()
        .expect("non-UTF-8 authenticated cwd must fail");
        assert!(utf8_error
            .to_string()
            .contains("authenticated world-deps request cwd must be valid UTF-8"));
        assert_eq!(calls.get(), 0);
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn authenticated_builder_has_exactly_two_authorized_production_consumers() {
        fn collect_rust_sources(dir: &std::path::Path, out: &mut Vec<std::path::PathBuf>) {
            for entry in std::fs::read_dir(dir).expect("read shell source directory") {
                let path = entry.expect("read shell source entry").path();
                if path.is_dir() {
                    collect_rust_sources(&path, out);
                } else if path.extension().and_then(std::ffi::OsStr::to_str) == Some("rs") {
                    out.push(path);
                }
            }
        }

        let manifest_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
        let source_root = manifest_dir.join("src");
        let definition_file = source_root.join("execution/routing/dispatch/world_ops.rs");
        let mut sources = Vec::new();
        collect_rust_sources(&source_root, &mut sources);
        let mut consumers = Vec::new();
        for source in sources {
            if source == definition_file {
                continue;
            }
            let body = std::fs::read_to_string(&source).expect("read Rust source");
            let mut current_function = None::<String>;
            for (line_index, line) in body.lines().enumerate() {
                if let Some(fn_offset) = line.find("fn ") {
                    let name = &line[fn_offset + 3..];
                    if let Some(paren_offset) = name.find('(') {
                        current_function = Some(name[..paren_offset].trim().to_string());
                    }
                }
                if line.contains("build_authenticated_world_deps_client_and_request(") {
                    consumers.push((
                        source
                            .strip_prefix(manifest_dir)
                            .expect("source under manifest")
                            .to_string_lossy()
                            .to_string(),
                        current_function
                            .clone()
                            .expect("authenticated builder call must be inside a function"),
                        line_index + 1,
                    ));
                }
            }
        }
        consumers.sort();

        assert_eq!(
            consumers
                .iter()
                .map(|(path, function, _)| (path.as_str(), function.as_str()))
                .collect::<Vec<_>>(),
            vec![
                (
                    "src/builtins/world_deps/surfaces.rs",
                    "run_world_command_for_deps_at",
                ),
                (
                    "src/builtins/world_enable/runner/provision_deps.rs",
                    "execute_with_profile",
                ),
            ]
        );
    }

    fn with_env_var<T>(key: &str, value: &str, f: impl FnOnce() -> T) -> T {
        let _authority_env = crate::execution::AuthorityEnvTestGuard::preserve();
        std::env::set_var(key, value);
        f()
    }

    fn encode_stream_frame(frame: ExecuteStreamFrame) -> hyper::body::Bytes {
        let mut payload = serde_json::to_vec(&frame).expect("serialize frame");
        payload.push(b'\n');
        hyper::body::Bytes::from(payload)
    }

    fn test_frame_identity(frame_sequence: u64) -> transport_api_types::RuntimeFrameIdentityV1 {
        transport_api_types::RuntimeFrameIdentityV1 {
            schema_version: transport_api_types::RUNTIME_FRAME_IDENTITY_SCHEMA_VERSION_V1,
            stream_id: "rts_world_ops_fixture".to_string(),
            frame_sequence,
        }
    }

    fn test_event_identity(event_sequence: u64) -> transport_api_types::RuntimeEventIdentityV1 {
        transport_api_types::RuntimeEventIdentityV1 {
            event_id: format!("evt_world_ops_fixture_{event_sequence}"),
            event_sequence,
        }
    }

    #[test]
    fn ensure_world_deps_bin_sets_default_and_prepends_path() {
        let mut env_map = std::collections::HashMap::<String, String>::new();
        env_map.insert("PATH".to_string(), "/usr/bin:/bin".to_string());

        ensure_world_deps_bin_on_path(&mut env_map);

        assert_eq!(
            env_map
                .get("SUBSTRATE_WORLD_DEPS_GUEST_BIN_DIR")
                .map(String::as_str),
            Some("/var/lib/substrate/world-deps/bin")
        );
        assert_eq!(
            env_map.get("PATH").map(String::as_str),
            Some("/var/lib/substrate/world-deps/bin:/usr/bin:/bin")
        );
    }

    #[test]
    fn ensure_world_deps_bin_respects_override_and_avoids_duplicates() {
        let mut env_map = std::collections::HashMap::<String, String>::new();
        env_map.insert(
            "SUBSTRATE_WORLD_DEPS_GUEST_BIN_DIR".to_string(),
            "/tmp/custom-bin/".to_string(),
        );
        env_map.insert(
            "PATH".to_string(),
            "/tmp/custom-bin:/usr/local/bin:/usr/bin".to_string(),
        );

        ensure_world_deps_bin_on_path(&mut env_map);

        assert_eq!(
            env_map
                .get("SUBSTRATE_WORLD_DEPS_GUEST_BIN_DIR")
                .map(String::as_str),
            Some("/tmp/custom-bin/")
        );
        assert_eq!(
            env_map.get("PATH").map(String::as_str),
            Some("/tmp/custom-bin:/usr/local/bin:/usr/bin")
        );
    }

    #[cfg(unix)]
    #[test]
    fn codex_member_dispatch_injects_internal_seed_home_when_backend_is_allowlisted() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let ambient_home = temp_dir.path().display().to_string();
        let (principal, account_home) =
            crate::execution::install_bootstrap::current_unix_principal_and_home()
                .expect("resolve current principal");
        let expected_seed_home = account_home.join(".codex").display().to_string();

        with_env_var("HOME", &ambient_home, || {
            let mut env_map = std::collections::HashMap::<String, String>::from([(
                SUBSTRATE_INTERNAL_CODEX_AUTH_SEED_HOME_ENV.to_string(),
                "/poisoned/ambient/.codex".to_string(),
            )]);
            let policy = substrate_broker::Policy {
                agents_host_credentials_read_allowed_backends: vec!["cli:codex-world".to_string()],
                ..substrate_broker::Policy::default()
            };

            maybe_inject_codex_auth_seed_home_for_policy(
                &mut env_map,
                MemberRuntimeBackendKindV1::Codex,
                "cli:codex-world",
                &policy,
                Some(&principal),
            )
            .expect("inject account-bound Codex seed home");

            assert_eq!(
                env_map
                    .get(SUBSTRATE_INTERNAL_CODEX_AUTH_SEED_HOME_ENV)
                    .map(String::as_str),
                Some(expected_seed_home.as_str())
            );
        });
    }

    #[cfg(unix)]
    #[test]
    fn codex_member_dispatch_skips_internal_seed_home_when_backend_is_not_allowlisted() {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let home = temp_dir.path().display().to_string();
        with_env_var("HOME", &home, || {
            let mut env_map = std::collections::HashMap::<String, String>::from([(
                SUBSTRATE_INTERNAL_CODEX_AUTH_SEED_HOME_ENV.to_string(),
                "/poisoned/ambient/.codex".to_string(),
            )]);
            let policy = substrate_broker::Policy::default();

            maybe_inject_codex_auth_seed_home_for_policy(
                &mut env_map,
                MemberRuntimeBackendKindV1::Codex,
                "cli:codex-world",
                &policy,
                None,
            )
            .expect("non-allowlisted backend does not require a principal");

            assert!(
                !env_map.contains_key(SUBSTRATE_INTERNAL_CODEX_AUTH_SEED_HOME_ENV),
                "unexpected seed home injection without backend allowlist"
            );
        });
    }

    #[cfg(unix)]
    #[test]
    fn codex_member_dispatch_injects_internal_seed_home_when_backend_is_allowlisted_only_for_exact_backend(
    ) {
        let temp_dir = tempfile::tempdir().expect("temp dir");
        let home = temp_dir.path().display().to_string();
        with_env_var("HOME", &home, || {
            let mut env_map = std::collections::HashMap::<String, String>::from([(
                SUBSTRATE_INTERNAL_CODEX_AUTH_SEED_HOME_ENV.to_string(),
                "/poisoned/ambient/.codex".to_string(),
            )]);
            let policy = substrate_broker::Policy {
                agents_host_credentials_read_allowed_backends: vec!["cli:codex-host".to_string()],
                ..substrate_broker::Policy::default()
            };

            maybe_inject_codex_auth_seed_home_for_policy(
                &mut env_map,
                MemberRuntimeBackendKindV1::Codex,
                "cli:codex-world",
                &policy,
                None,
            )
            .expect("different allowlisted backend does not require a principal");

            assert!(
                !env_map.contains_key(SUBSTRATE_INTERNAL_CODEX_AUTH_SEED_HOME_ENV),
                "seed home injection must stay pinned to the exact allowlisted backend"
            );
        });
    }

    #[cfg(unix)]
    #[test]
    fn codex_member_dispatch_rejects_allowlisted_seed_without_typed_principal() {
        let mut env_map = std::collections::HashMap::<String, String>::from([(
            SUBSTRATE_INTERNAL_CODEX_AUTH_SEED_HOME_ENV.to_string(),
            "/poisoned/ambient/.codex".to_string(),
        )]);
        let policy = substrate_broker::Policy {
            agents_host_credentials_read_allowed_backends: vec!["cli:codex-world".to_string()],
            ..substrate_broker::Policy::default()
        };

        let error = maybe_inject_codex_auth_seed_home_for_policy(
            &mut env_map,
            MemberRuntimeBackendKindV1::Codex,
            "cli:codex-world",
            &policy,
            None,
        )
        .expect_err("allowlisted Codex seed requires typed intended principal");

        assert!(
            error.to_string().contains("intended host principal"),
            "unexpected missing-principal error: {error:#}"
        );
        assert!(!env_map.contains_key(SUBSTRATE_INTERNAL_CODEX_AUTH_SEED_HOME_ENV));
    }

    #[cfg(unix)]
    #[test]
    fn codex_member_dispatch_rejects_principal_that_fails_account_uid_round_trip() {
        let (principal, _) = crate::execution::install_bootstrap::current_unix_principal_and_home()
            .expect("resolve current principal");
        let PlatformPrincipalV1::Unix { account, uid } = principal else {
            panic!("current Unix principal must use the Unix variant");
        };
        let mismatched = PlatformPrincipalV1::Unix {
            account,
            uid: uid ^ 1,
        };
        let mut env_map = std::collections::HashMap::<String, String>::from([(
            SUBSTRATE_INTERNAL_CODEX_AUTH_SEED_HOME_ENV.to_string(),
            "/poisoned/ambient/.codex".to_string(),
        )]);
        let policy = substrate_broker::Policy {
            agents_host_credentials_read_allowed_backends: vec!["cli:codex-world".to_string()],
            ..substrate_broker::Policy::default()
        };

        let error = maybe_inject_codex_auth_seed_home_for_policy(
            &mut env_map,
            MemberRuntimeBackendKindV1::Codex,
            "cli:codex-world",
            &policy,
            Some(&mismatched),
        )
        .expect_err("mismatched account and UID must fail closed");

        assert!(
            error.to_string().contains("round-trip")
                || error.to_string().contains("does not exist")
                || error.to_string().contains("could not be resolved"),
            "unexpected mismatched-principal error: {error:#}"
        );
        assert!(!env_map.contains_key(SUBSTRATE_INTERNAL_CODEX_AUTH_SEED_HOME_ENV));
    }

    #[cfg(unix)]
    #[test]
    fn non_codex_member_dispatch_clears_poisoned_internal_seed_home() {
        let mut env_map = std::collections::HashMap::<String, String>::from([(
            SUBSTRATE_INTERNAL_CODEX_AUTH_SEED_HOME_ENV.to_string(),
            "/poisoned/ambient/.codex".to_string(),
        )]);
        let policy = substrate_broker::Policy {
            agents_host_credentials_read_allowed_backends: vec!["cli:codex-world".to_string()],
            ..substrate_broker::Policy::default()
        };

        maybe_inject_codex_auth_seed_home_for_policy(
            &mut env_map,
            MemberRuntimeBackendKindV1::ClaudeCode,
            "cli:claude-world",
            &policy,
            None,
        )
        .expect("non-Codex backend does not require a principal");

        assert!(
            !env_map.contains_key(SUBSTRATE_INTERNAL_CODEX_AUTH_SEED_HOME_ENV),
            "reserved seed home must not survive a non-Codex backend"
        );
    }

    #[test]
    fn preserve_world_project_dir_override_records_logical_root() {
        let _authority_env = crate::execution::AuthorityEnvTestGuard::preserve();
        let _env_guard = crate::execution::world_env_guard();
        let prev_mode = std::env::var("SUBSTRATE_ANCHOR_MODE").ok();
        let prev_path = std::env::var("SUBSTRATE_ANCHOR_PATH").ok();
        let prev_caged = std::env::var("SUBSTRATE_CAGED").ok();

        std::env::set_var("SUBSTRATE_ANCHOR_MODE", "custom");
        std::env::set_var("SUBSTRATE_ANCHOR_PATH", "/tmp/substrate-world-root");
        std::env::set_var("SUBSTRATE_CAGED", "1");

        let mut env_map = std::collections::HashMap::<String, String>::new();
        env_map.insert("SUBSTRATE_ANCHOR_MODE".to_string(), "custom".to_string());
        env_map.insert(
            "SUBSTRATE_ANCHOR_PATH".to_string(),
            "/tmp/substrate-world-root".to_string(),
        );
        preserve_world_project_dir_override(&mut env_map, std::path::Path::new("/tmp/ignored"));

        assert_eq!(
            env_map
                .get(WORLD_PROJECT_DIR_OVERRIDE_ENV)
                .map(String::as_str),
            Some("/tmp/substrate-world-root")
        );

        match prev_mode {
            Some(value) => std::env::set_var("SUBSTRATE_ANCHOR_MODE", value),
            None => std::env::remove_var("SUBSTRATE_ANCHOR_MODE"),
        }
        match prev_path {
            Some(value) => std::env::set_var("SUBSTRATE_ANCHOR_PATH", value),
            None => std::env::remove_var("SUBSTRATE_ANCHOR_PATH"),
        }
        match prev_caged {
            Some(value) => std::env::set_var("SUBSTRATE_CAGED", value),
            None => std::env::remove_var("SUBSTRATE_CAGED"),
        }
    }

    #[test]
    fn preserve_world_project_dir_override_uses_dispatch_cwd_for_follow_cwd() {
        let _authority_env = crate::execution::AuthorityEnvTestGuard::preserve();
        let _env_guard = crate::execution::world_env_guard();
        let prev_mode = std::env::var("SUBSTRATE_ANCHOR_MODE").ok();
        let prev_path = std::env::var("SUBSTRATE_ANCHOR_PATH").ok();

        std::env::set_var("SUBSTRATE_ANCHOR_MODE", "custom");
        std::env::set_var("SUBSTRATE_ANCHOR_PATH", "/tmp/process-anchor");

        let mut env_map = std::collections::HashMap::<String, String>::new();
        env_map.insert(
            "SUBSTRATE_ANCHOR_MODE".to_string(),
            "follow-cwd".to_string(),
        );
        env_map.insert(
            "SUBSTRATE_ANCHOR_PATH".to_string(),
            "/tmp/should-not-win".to_string(),
        );

        preserve_world_project_dir_override(
            &mut env_map,
            std::path::Path::new("/tmp/member-dispatch-cwd"),
        );

        assert_eq!(
            env_map
                .get(WORLD_PROJECT_DIR_OVERRIDE_ENV)
                .map(String::as_str),
            Some("/tmp/member-dispatch-cwd")
        );

        match prev_mode {
            Some(value) => std::env::set_var("SUBSTRATE_ANCHOR_MODE", value),
            None => std::env::remove_var("SUBSTRATE_ANCHOR_MODE"),
        }
        match prev_path {
            Some(value) => std::env::set_var("SUBSTRATE_ANCHOR_PATH", value),
            None => std::env::remove_var("SUBSTRATE_ANCHOR_PATH"),
        }
    }

    #[test]
    fn preserve_world_project_dir_override_keeps_explicit_env_override() {
        let mut env_map = std::collections::HashMap::<String, String>::new();
        env_map.insert(
            WORLD_PROJECT_DIR_OVERRIDE_ENV.to_string(),
            "/var/lib/substrate/staged-workspace/current".to_string(),
        );
        env_map.insert(
            "SUBSTRATE_ANCHOR_MODE".to_string(),
            "follow-cwd".to_string(),
        );

        preserve_world_project_dir_override(&mut env_map, std::path::Path::new("/"));

        assert_eq!(
            env_map
                .get(WORLD_PROJECT_DIR_OVERRIDE_ENV)
                .map(String::as_str),
            Some("/var/lib/substrate/staged-workspace/current")
        );
    }

    #[test]
    fn current_world_request_profile_accepts_non_reserved_values() {
        let _authority_env = crate::execution::AuthorityEnvTestGuard::preserve();
        with_env_var(
            "SUBSTRATE_WORLD_REQUEST_PROFILE",
            "wdap-smoke-profile",
            || {
                assert_eq!(
                    current_world_request_profile().as_deref(),
                    Some("wdap-smoke-profile")
                );
            },
        );
    }

    #[test]
    fn member_dispatch_payload_preserves_frozen_lineage_and_world_fields() {
        let payload = build_member_dispatch_payload(&MemberDispatchTransportRequest {
            orchestration_session_id: "orch_123".to_string(),
            participant_id: "ash_member_123".to_string(),
            orchestrator_participant_id: "ash_orch_123".to_string(),
            parent_participant_id: Some("ash_parent_123".to_string()),
            resumed_from_participant_id: Some("ash_prev_123".to_string()),
            backend_id: "cli:codex".to_string(),
            protocol: "substrate.agent.session".to_string(),
            run_id: "run_123".to_string(),
            world_id: "world_123".to_string(),
            world_generation: 9,
            initial_prompt: Some("first turn".to_string()),
            backend_kind: MemberRuntimeBackendKindV1::Codex,
            binary_path: "/usr/bin/codex".to_string(),
            retained_worker_launch_authority: None,
            e2_launch_activation: None,
            config_projection: None,
            exact_policy_snapshot: None,
        });
        let payload = payload
            .as_v1()
            .expect("legacy transport must preserve the V1 member-dispatch variant");

        assert_eq!(payload.schema_version, 1);
        assert_eq!(payload.orchestration_session_id, "orch_123");
        assert_eq!(payload.participant_id, "ash_member_123");
        assert_eq!(payload.orchestrator_participant_id, "ash_orch_123");
        assert_eq!(
            payload.parent_participant_id.as_deref(),
            Some("ash_parent_123")
        );
        assert_eq!(
            payload.resumed_from_participant_id.as_deref(),
            Some("ash_prev_123")
        );
        assert_eq!(payload.backend_id, "cli:codex");
        assert_eq!(payload.protocol, "substrate.agent.session");
        assert_eq!(payload.run_id, "run_123");
        assert_eq!(payload.world_id, "world_123");
        assert_eq!(payload.world_generation, 9);
        assert_eq!(payload.initial_prompt.as_deref(), Some("first turn"));
        assert_eq!(payload.retained_worker_launch_authority, None);
        assert_eq!(
            payload.resolved_runtime,
            ResolvedMemberRuntimeDescriptorV1 {
                backend_kind: MemberRuntimeBackendKindV1::Codex,
                binary_path: "/usr/bin/codex".to_string(),
            }
        );
    }

    #[test]
    fn e3a_member_dispatch_payload_preserves_injected_projection_for_all_admitted_paths() {
        let projection = e3a_projection_carrier();
        projection
            .validate()
            .expect("strict E3-A projection carrier");
        for path in ["spawn", "fork", "toolbox", "continue-fork"] {
            let payload = build_member_dispatch_payload(&e3a_transport_for_path(path));
            payload
                .validate()
                .unwrap_or_else(|error| panic!("valid {path} V2 payload: {error}"));
            let encoded = serde_json::to_vec(&payload).expect("encode strict path payload");
            let decoded: transport_api_types::MemberDispatchRequest =
                serde_json::from_slice(&encoded).expect("decode strict path payload");
            assert_eq!(decoded, payload, "{path} must survive the wire codec");
            let transport_api_types::MemberDispatchRequest::V2(payload) = payload else {
                panic!("{path} projection must select strict V2 without fallback");
            };
            assert_eq!(payload.schema_version, 2);
            assert_eq!(payload.config_projection, projection);
            assert_eq!(payload.initial_prompt.as_deref(), Some(path));
        }
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn e3a_injected_toolbox_transport_reaches_world_service_through_agent_client() {
        const REJECTION: &str = "member_dispatch V2 launch is not implemented";

        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("build E3-A runtime");
        runtime.block_on(async {
            let temp = tempfile::Builder::new()
                .prefix("e3a-toolbox-wire-")
                .tempdir()
                .expect("create E3-A tempdir");
            let socket_path = temp.path().join("world.sock");
            let listener =
                tokio::net::UnixListener::bind(&socket_path).expect("bind E3-A world socket");

            let transport = e3a_transport_for_path("toolbox");
            let activation = transport
                .e2_launch_activation
                .as_ref()
                .expect("toolbox V2 activation");
            let policy_snapshot = activation
                .policy_snapshot()
                .expect("decode toolbox activation snapshot");
            let projection = transport
                .config_projection
                .clone()
                .expect("injected toolbox projection");
            let request = build_execute_request(ExecuteRequestInput {
                profile: None,
                cmd: String::new(),
                cwd: temp.path().display().to_string(),
                env_map: std::collections::HashMap::new(),
                agent_id: "e3a-toolbox".to_string(),
                policy_snapshot,
                world_network: WorldNetworkRoutingV1 {
                    isolate_network: true,
                    allowed_domains: vec!["api.example.com".to_string()],
                },
                world_fs_mode: WorldFsMode::ReadOnly,
                member_dispatch: Some(build_member_dispatch_payload(&transport)),
                acceptance_context: None,
            });
            request.validate().expect("valid injected toolbox request");

            let expected_body = serde_json::to_vec(&request).expect("encode toolbox request");
            let state_root = temp.path().join("member-turn-state");
            std::fs::create_dir_all(&state_root).expect("create E3-A member-turn state root");
            let service =
                world_service::WorldService::new_with_member_turn_state_root_for_test(&state_root)
                    .expect("construct E3-A world service");
            let server = tokio::spawn(async move {
                let (mut stream, _) = listener.accept().await.expect("accept E3-A client");
                let (header, body) = read_e3a_http_request(&mut stream).await;
                assert!(header.starts_with("POST /v1/execute/stream "));
                assert_eq!(body, expected_body, "client transport must not rewrite V2");
                let received: transport_api_types::ExecuteRequest =
                    serde_json::from_slice(&body).expect("decode toolbox request at world-service");
                assert_eq!(
                    received
                        .member_dispatch
                        .as_ref()
                        .unwrap()
                        .config_projection(),
                    Some(&projection)
                );
                let error = service
                    .execute_stream(received)
                    .await
                    .expect_err("world-service must reject E3-A before V2 launch");
                assert_eq!(error.to_string(), REJECTION);
                write_e3a_http_bad_request(&mut stream, REJECTION).await;
            });

            let client = transport_api_client::AgentClient::unix_socket(&socket_path)
                .expect("construct E3-A client");
            let error = client
                .execute_stream(request)
                .await
                .expect_err("client must observe world-service V2 rejection");
            assert!(error.to_string().contains(REJECTION));
            server.await.expect("join E3-A world-service server");
        });
    }

    #[test]
    fn member_dispatch_policy_carrier_preserves_exact_snapshot_bytes() {
        let cwd = tempfile::tempdir().expect("create policy carrier cwd");
        let snapshot = PolicySnapshotV3 {
            schema_version: 3,
            net_allowed: vec!["api.example.com".to_string()],
            world_fs: PolicySnapshotWorldFsV3 {
                host_visible: false,
                fail_closed: PolicySnapshotWorldFsFailClosedV3 { routing: true },
                deny_enforcement: None,
                caged_required: true,
                discover: Some(transport_api_types::PolicySnapshotWorldFsDimensionV3 {
                    allow_list: vec!["exact".to_string()],
                    deny_list: Vec::new(),
                }),
                read: Some(transport_api_types::PolicySnapshotWorldFsDimensionV3 {
                    allow_list: vec!["exact".to_string()],
                    deny_list: Vec::new(),
                }),
                write: PolicySnapshotWorldFsWriteV3 {
                    enabled: false,
                    allow_list: vec!["exact".to_string()],
                    deny_list: Vec::new(),
                },
            },
        };
        let exact_bytes = serde_json::to_vec(&snapshot).expect("serialize exact snapshot");
        let exact_hash = {
            use sha2::Digest as _;
            format!("{:x}", sha2::Sha256::digest(&exact_bytes))
        };
        let material = ExactDispatchPolicySnapshotMaterialV1::from_exact_e1_material(
            snapshot.clone(),
            exact_bytes,
            exact_hash,
        )
        .expect("construct exact snapshot material");
        let request = MemberDispatchTransportRequest {
            orchestration_session_id: "orch_123".to_string(),
            participant_id: "ash_member_123".to_string(),
            orchestrator_participant_id: "ash_orch_123".to_string(),
            parent_participant_id: None,
            resumed_from_participant_id: None,
            backend_id: "cli:codex".to_string(),
            protocol: "substrate.agent.session".to_string(),
            run_id: "run_123".to_string(),
            world_id: "world_123".to_string(),
            world_generation: 9,
            initial_prompt: None,
            backend_kind: MemberRuntimeBackendKindV1::Codex,
            binary_path: "/usr/bin/codex".to_string(),
            retained_worker_launch_authority: None,
            e2_launch_activation: None,
            config_projection: None,
            exact_policy_snapshot: Some(material.clone()),
        };

        let resolved = request
            .resolve_world_network_policy(cwd.path())
            .expect("resolve carried policy");

        assert_eq!(
            serde_json::to_vec(&resolved.snapshot).expect("serialize resolved snapshot"),
            serde_json::to_vec(&snapshot).expect("serialize exact carrier snapshot")
        );

        let mut substituted_hash = material.clone();
        substituted_hash.policy_snapshot_hash = "0".repeat(64);
        assert!(substituted_hash.validate().is_err());

        let mut substituted_bytes = material;
        substituted_bytes.policy_snapshot_bytes.push(b' ');
        assert!(substituted_bytes.validate().is_err());
    }

    #[test]
    fn build_execute_request_supports_typed_member_dispatch_shape() {
        let request = build_execute_request(ExecuteRequestInput {
            profile: Some("world-member-dispatch".to_string()),
            cmd: String::new(),
            cwd: "/tmp/worktree".to_string(),
            env_map: std::collections::HashMap::new(),
            agent_id: "tester".to_string(),
            policy_snapshot: PolicySnapshotV3 {
                schema_version: 3,
                net_allowed: Vec::new(),
                world_fs: PolicySnapshotWorldFsV3 {
                    host_visible: true,
                    fail_closed: PolicySnapshotWorldFsFailClosedV3 { routing: false },
                    deny_enforcement: None,
                    caged_required: false,
                    discover: None,
                    read: None,
                    write: PolicySnapshotWorldFsWriteV3 {
                        enabled: true,
                        allow_list: vec![".".to_string()],
                        deny_list: Vec::new(),
                    },
                },
            },
            world_network: WorldNetworkRoutingV1 {
                isolate_network: false,
                allowed_domains: Vec::new(),
            },
            world_fs_mode: WorldFsMode::Writable,
            acceptance_context: None,
            member_dispatch: Some(build_member_dispatch_payload(
                &MemberDispatchTransportRequest {
                    orchestration_session_id: "orch_123".to_string(),
                    participant_id: "ash_member_123".to_string(),
                    orchestrator_participant_id: "ash_orch_123".to_string(),
                    parent_participant_id: None,
                    resumed_from_participant_id: None,
                    backend_id: "cli:codex".to_string(),
                    protocol: "substrate.agent.session".to_string(),
                    run_id: "run_123".to_string(),
                    world_id: "world_123".to_string(),
                    world_generation: 9,
                    initial_prompt: None,
                    backend_kind: MemberRuntimeBackendKindV1::Codex,
                    binary_path: "/usr/bin/codex".to_string(),
                    retained_worker_launch_authority: None,
                    e2_launch_activation: None,
                    config_projection: None,
                    exact_policy_snapshot: None,
                },
            )),
        });

        assert!(request.cmd.is_empty());
        assert!(!request.pty);
        assert_eq!(request.agent_id, "tester");
        assert_eq!(
            request
                .member_dispatch
                .as_ref()
                .map(|dispatch| dispatch.common().run_id),
            Some("run_123")
        );
        assert_eq!(
            request.member_dispatch.as_ref().map(|dispatch| dispatch
                .common()
                .resolved_runtime
                .binary_path
                .as_str()),
            Some("/usr/bin/codex")
        );
        request
            .validate()
            .expect("typed member dispatch request validates");
    }

    #[test]
    fn current_world_request_profile_rejects_reserved_world_deps_profiles() {
        let _authority_env = crate::execution::AuthorityEnvTestGuard::preserve();
        for reserved in ["world-deps-provision", "world-deps-probe"] {
            with_env_var("SUBSTRATE_WORLD_REQUEST_PROFILE", reserved, || {
                assert_eq!(
                    current_world_request_profile(),
                    None,
                    "reserved internal profile should not be forwarded from env: {reserved}"
                );
            });
        }
    }

    #[test]
    fn windows_live_dispatch_builders_avoid_backend_bootstrap_before_client_selection() {
        let source = include_str!("world_ops.rs");
        let windows_sections = [
            (
                "#[cfg(target_os = \"windows\")]\nfn build_agent_client_and_request_impl(",
                "#[allow(dead_code)]\n#[cfg(target_os = \"windows\")]\nfn build_agent_client_and_member_dispatch_request_impl(",
            ),
            (
                "#[allow(dead_code)]\n#[cfg(target_os = \"windows\")]\nfn build_agent_client_and_member_dispatch_request_impl(",
                "#[cfg(target_os = \"windows\")]\nfn build_agent_client_and_pending_diff_request_impl()",
            ),
            (
                "#[cfg(target_os = \"windows\")]\nfn build_agent_client_and_pending_diff_request_impl()",
                "pub(crate) fn stream_non_pty_via_agent(",
            ),
        ];

        for (start_marker, end_marker) in windows_sections {
            let start = source.find(start_marker).expect("windows dispatch section");
            let end = source[start..]
                .find(end_marker)
                .map(|offset| start + offset)
                .expect("windows dispatch section end");
            let section = &source[start..end];

            assert!(
                section.contains("windows::build_agent_client()?"),
                "missing typed Windows client selection in section starting with {start_marker}"
            );
            assert!(
                section.contains("mark_windows_world_dispatch_active_without_session();"),
                "missing session-free Windows world marker in section starting with {start_marker}"
            );
            let client_pos = section
                .find("windows::build_agent_client()?")
                .expect("typed Windows client selection position");
            let marker_pos = section
                .find("mark_windows_world_dispatch_active_without_session();")
                .expect("session-free Windows world marker position");
            assert!(
                client_pos < marker_pos,
                "Windows live dispatch must not mark the world active before typed client selection in {start_marker}"
            );
            assert!(
                !section.contains("windows::get_backend()?"),
                "Windows live dispatch unexpectedly reintroduced backend bootstrap in {start_marker}"
            );
            assert!(
                !section.contains("ensure_session("),
                "Windows live dispatch unexpectedly reintroduced session realization in {start_marker}"
            );
        }
    }

    #[test]
    fn windows_world_doctor_source_persists_context_before_client_selection() {
        let source = include_str!("../../platform/windows.rs");
        let start = source
            .find("pub(crate) fn world_doctor_main(")
            .expect("world_doctor_main");
        let end = source[start..]
            .find("let ok =")
            .map(|offset| start + offset)
            .expect("world_doctor_main summary boundary");
        let section = &source[start..end];

        let detect_pos = section
            .find("crate::execution::pw::detect().and_then(|detected| {")
            .expect("detect path");
        let store_pos = section
            .find("crate::execution::pw::store_context_globally(detected);")
            .expect("context store");
        let client_pos = section
            .find("crate::execution::pw::windows::build_agent_client()?")
            .expect("typed client build");

        assert!(
            detect_pos < store_pos && store_pos < client_pos,
            "Windows world doctor must persist the detected context before building the typed client so host diagnostics and world doctor share one mapping"
        );
    }

    #[test]
    fn macos_world_doctor_human_output_formats_optional_guest_socket_safely() {
        let source = include_str!("../../platform/macos.rs");
        assert!(
            source.contains(
                "assessment\n                        .transport_guest_socket\n                        .as_deref()\n                        .unwrap_or(\"unavailable\")"
            ),
            "human macOS doctor output must render the optional guest socket without requiring Display on Option<String>"
        );
    }

    #[test]
    fn macos_platform_detect_source_resolves_limactl_with_repo_fallbacks() {
        let source = include_str!("../../platform_world/mod.rs");
        let start = source
            .find("let result = (|| -> Result<(String, String, u32, String)> {")
            .expect("macOS guest observation start");
        let end = source[start..]
            .find("let stdout = String::from_utf8(output.stdout)")
            .map(|offset| start + offset)
            .expect("macOS guest observation parse boundary");
        let section = &source[start..end];

        assert!(
            section.contains("SUBSTRATE_TEST_LIMACTL_PATH"),
            "macOS guest observation must honor the existing limactl test-override path"
        );
        assert!(
            section.contains("/opt/homebrew/bin/limactl")
                && section.contains("/usr/local/bin/limactl")
                && section.contains("/opt/homebrew/sbin/limactl")
                && section.contains("/usr/local/sbin/limactl"),
            "macOS guest observation must preserve the established Homebrew fallback search for limactl"
        );
        assert!(
            section.contains("Command::new(&limactl_path)"),
            "macOS guest observation must launch the resolved limactl path"
        );
        assert!(
            !section.contains("Command::new(\"limactl\")"),
            "macOS guest observation must not bypass the resolved limactl path with a raw PATH-only launch"
        );
    }

    #[test]
    fn macos_world_doctor_source_preserves_undeclared_vm_state_and_mapping_provenance() {
        let source = include_str!("../../platform/macos.rs");
        assert!(
            source.contains("\"undeclared\".to_string()"),
            "macOS doctor must preserve the undeclared VM state instead of fabricating an unavailable VM identity"
        );
        assert!(
            !source.contains("\"unavailable\".to_string()"),
            "macOS doctor must not fabricate an unavailable VM name placeholder"
        );
        assert!(
            source.contains("info(\"declared VM: not declared\");"),
            "macOS doctor human output must report the undeclared VM state explicitly"
        );
        assert!(
            source.contains(
                "Lima VM not declared (set SUBSTRATE_LIMA_VM_NAME when no verified mapping is available)"
            ),
            "macOS doctor human output must explain the undeclared VM boundary"
        );
        assert!(
            source.contains(
                "info(\"Lima control root: resolved from authenticated platform mapping.\");"
            ),
            "macOS world doctor must report mapping-derived control-root provenance consistently"
        );
        assert!(
            !source.contains("info(\"Lima control root: resolved from account database.\");"),
            "macOS world doctor must not claim account-database control-root provenance after mapping projection"
        );
    }

    #[test]
    fn process_agent_stream_requests_cancel_after_start_frame() {
        let rt = tokio::runtime::Runtime::new().expect("runtime");
        rt.block_on(async {
            let frames = vec![
                encode_stream_frame(ExecuteStreamFrame::Start {
                    frame_identity: test_frame_identity(1),
                    span_id: "spn_interrupt".to_string(),
                }),
                encode_stream_frame(ExecuteStreamFrame::Exit {
                    frame_identity: test_frame_identity(2),
                    event_identity: test_event_identity(1),
                    terminal_identity: transport_api_types::RuntimeTerminalIdentityV1::from(
                        &test_event_identity(1),
                    ),
                    exit: 130,
                    span_id: "spn_interrupt".to_string(),
                    scopes_used: Vec::new(),
                    fs_diff: None,
                    process_telemetry: transport_api_types::ProcessTelemetry::default(),
                }),
            ];

            let (awaiting_exit_tx, awaiting_exit_rx) = tokio::sync::oneshot::channel();
            let (release_exit_tx, release_exit_rx) = tokio::sync::oneshot::channel();
            let stream = stream::unfold(
                (
                    0usize,
                    frames,
                    Some(awaiting_exit_tx),
                    Some(release_exit_rx),
                ),
                |(idx, frames, mut awaiting_exit_tx, mut release_exit_rx)| async move {
                    match idx {
                        0 => Some((
                            Ok::<_, Infallible>(hyper::body::Frame::data(frames[0].clone())),
                            (1, frames, awaiting_exit_tx, release_exit_rx),
                        )),
                        1 => {
                            awaiting_exit_tx
                                .take()
                                .expect("single stream exit-wait notification")
                                .send(())
                                .expect("SIGINT task must await Start processing");
                            release_exit_rx
                                .take()
                                .expect("single stream Exit release receiver")
                                .await
                                .expect("cancel callback must release terminal frame");
                            Some((
                                Ok::<_, Infallible>(hyper::body::Frame::data(frames[1].clone())),
                                (2, frames, awaiting_exit_tx, release_exit_rx),
                            ))
                        }
                        _ => None,
                    }
                },
            );
            let body = StreamBody::new(stream);

            let (sigint_tx, mut sigint_rx) = tokio::sync::mpsc::unbounded_channel();
            let cancels = Arc::new(Mutex::new(Vec::<(String, String)>::new()));
            let cancels_for_cancel = Arc::clone(&cancels);

            let sigint_task = tokio::spawn(async move {
                awaiting_exit_rx
                    .await
                    .expect("stream must request its post-Start frame");
                sigint_tx
                    .send(())
                    .expect("stream SIGINT receiver must remain live");
            });
            let release_exit_tx = Arc::new(Mutex::new(Some(release_exit_tx)));
            let release_exit_for_cancel = Arc::clone(&release_exit_tx);

            let outcome = process_agent_stream_body(
                body,
                "agent".to_string(),
                None,
                &mut sigint_rx,
                move |span_id, sig| {
                    let cancels = Arc::clone(&cancels_for_cancel);
                    let release_exit = Arc::clone(&release_exit_for_cancel);
                    async move {
                        cancels.lock().expect("cancel lock").push((span_id, sig));
                        release_exit
                            .lock()
                            .expect("terminal release lock")
                            .take()
                            .expect("release terminal exactly once")
                            .send(())
                            .expect("stream must still await terminal release");
                        Ok(())
                    }
                },
            )
            .await
            .expect("process stream");
            sigint_task
                .await
                .expect("SIGINT publication task must terminate");

            assert_eq!(outcome.exit_code, 130);
            assert_eq!(
                cancels.lock().expect("cancel lock").as_slice(),
                &[("spn_interrupt".to_string(), "INT".to_string())]
            );
        });
    }

    #[test]
    #[serial_test::serial]
    fn emit_stream_chunk_some_emits_orchestration_scoped_agent_event() {
        if crate::execution::run_in_bounded_test_subprocess(
            concat!(
                module_path!(),
                "::",
                stringify!(emit_stream_chunk_some_emits_orchestration_scoped_agent_event)
            ),
            "event_registry",
        ) {
            return;
        }
        let _guard = acquire_event_test_guard();
        let mut rx = init_event_channel();

        emit_stream_chunk(
            "agent",
            Some("orch-live"),
            "run-1",
            Some("spn-1"),
            b"hello stdout",
            false,
        );

        let event = rx.try_recv().expect("stream chunk event");
        assert_eq!(event.kind, AgentEventKind::PtyData);
        assert_eq!(event.orchestration_session_id, "orch-live");
        assert_eq!(event.run_id, "run-1");
        assert_eq!(event.span_id.as_deref(), Some("spn-1"));
        clear_agent_event_sender();
    }

    #[test]
    #[serial_test::serial]
    fn emit_stream_chunk_none_emits_no_orchestration_scoped_agent_event() {
        if crate::execution::run_in_bounded_test_subprocess(
            concat!(
                module_path!(),
                "::",
                stringify!(emit_stream_chunk_none_emits_no_orchestration_scoped_agent_event)
            ),
            "event_registry",
        ) {
            return;
        }
        let _guard = acquire_event_test_guard();
        let mut rx = init_event_channel();

        emit_stream_chunk("agent", None, "run-1", Some("spn-1"), b"hello stderr", true);

        assert!(
            rx.try_recv().is_err(),
            "stream chunk without orchestration context must not emit an agent event"
        );
        clear_agent_event_sender();
    }

    #[test]
    #[serial_test::serial]
    fn process_agent_stream_body_uses_launch_owned_run_id_for_stream_rows() {
        if crate::execution::run_in_bounded_test_subprocess(
            concat!(
                module_path!(),
                "::",
                stringify!(process_agent_stream_body_uses_launch_owned_run_id_for_stream_rows)
            ),
            "event_registry",
        ) {
            return;
        }
        let _guard = acquire_event_test_guard();
        let rt = tokio::runtime::Runtime::new().expect("runtime");
        rt.block_on(async {
            let mut rx = init_event_channel();
            let frames = vec![
                encode_stream_frame(ExecuteStreamFrame::Start {
                    frame_identity: test_frame_identity(1),
                    span_id: "spn-world".to_string(),
                }),
                encode_stream_frame(ExecuteStreamFrame::Stdout {
                    frame_identity: test_frame_identity(2),
                    chunk_b64: BASE64.encode(b"hello world"),
                }),
                encode_stream_frame(ExecuteStreamFrame::Exit {
                    frame_identity: test_frame_identity(3),
                    event_identity: test_event_identity(1),
                    terminal_identity: transport_api_types::RuntimeTerminalIdentityV1::from(
                        &test_event_identity(1),
                    ),
                    exit: 0,
                    span_id: "spn-world".to_string(),
                    scopes_used: Vec::new(),
                    fs_diff: None,
                    process_telemetry: transport_api_types::ProcessTelemetry::default(),
                }),
            ];
            let body = StreamBody::new(stream::iter(
                frames
                    .into_iter()
                    .map(|frame| Ok::<_, Infallible>(hyper::body::Frame::data(frame))),
            ));
            let mut sigint_rx = tokio::sync::mpsc::unbounded_channel().1;
            let context = ShellCommandEventContext::new(
                ShellEventEmissionContext {
                    orchestration_session_id: "orch-live".to_string(),
                    agent_id: "shell".to_string(),
                    role: Some("orchestrator".to_string()),
                    backend_id: Some("shell:repl".to_string()),
                    participant_id: Some("participant-1".to_string()),
                    parent_participant_id: None,
                    resumed_from_participant_id: None,
                    world_id: Some("world-1".to_string()),
                    world_generation: Some(4),
                },
                "cmd-123",
                Some("cmd-123".to_string()),
                Some("spn-shell".to_string()),
            );

            let outcome = process_agent_stream_body(
                body,
                "agent".to_string(),
                Some(&context),
                &mut sigint_rx,
                |_span_id, _sig| async { Ok(()) },
            )
            .await
            .expect("process stream");

            assert_eq!(outcome.exit_code, 0);
            let event = rx.try_recv().expect("stream chunk event");
            assert_eq!(event.kind, AgentEventKind::PtyData);
            assert_eq!(event.orchestration_session_id, "orch-live");
            assert_eq!(event.run_id, "cmd-123");
            assert_eq!(event.span_id.as_deref(), Some("spn-world"));
        });
        clear_agent_event_sender();
    }

    #[test]
    fn extract_process_telemetry_from_ws_exit_preserves_ptrace_not_permitted_reason() {
        let exit = json!({
            "type": "exit",
            "exit": 0,
            "span_id": "spn_ptrace_denied",
            "scopes_used": [],
            "process_events": [],
            "process_events_status": "unavailable",
            "process_events_reason": "ptrace_not_permitted"
        });

        let process_telemetry = extract_process_telemetry_from_ws_exit(&exit);

        assert_eq!(
            process_telemetry.process_events_status,
            substrate_common::ProcessEventsStatus::Unavailable
        );
        assert_eq!(
            process_telemetry.process_events_reason.as_deref(),
            Some("ptrace_not_permitted")
        );
        assert!(process_telemetry.process_events.is_empty());
        assert!(process_telemetry.process_events_dropped.is_none());
    }

    #[test]
    fn extract_process_telemetry_from_ws_exit_preserves_linux_process_event_fields() {
        let exit = json!({
            "type": "exit",
            "exit": 0,
            "span_id": "spn_parent",
            "scopes_used": [],
            "process_events": [
                {
                    "ts": "2026-04-01T00:00:00Z",
                    "ts_unix_ns": 1_743_465_600_000_000_000u64,
                    "event_type": "world_process_start",
                    "session_id": "ses_linux",
                    "world_id": "wld_linux",
                    "pid": 42,
                    "ppid": 1,
                    "cwd": "/project",
                    "parent_span": "spn_parent",
                    "parent_cmd_id": "cmd_parent",
                    "argv_omitted": true
                },
                {
                    "ts": "2026-04-01T00:00:01Z",
                    "ts_unix_ns": 1_743_465_601_000_000_000u64,
                    "event_type": "world_process_exit",
                    "session_id": "ses_linux",
                    "world_id": "wld_linux",
                    "pid": 42,
                    "ppid": 1,
                    "cwd": "/project",
                    "parent_span": "spn_parent",
                    "parent_cmd_id": "cmd_parent",
                    "argv_omitted": true,
                    "exit_code": 0,
                    "duration_ms": 11
                }
            ],
            "process_events_status": "truncated",
            "process_events_reason": "capture_overflow",
            "process_events_dropped": 7,
            "process_events_max": 10000,
            "process_events_backend": "ptrace"
        });

        let process_telemetry = extract_process_telemetry_from_ws_exit(&exit);

        assert_eq!(
            process_telemetry.process_events_status,
            substrate_common::ProcessEventsStatus::Truncated
        );
        assert_eq!(
            process_telemetry.process_events_reason.as_deref(),
            Some("capture_overflow")
        );
        assert_eq!(process_telemetry.process_events_dropped, Some(7));
        assert_eq!(process_telemetry.process_events_max, Some(10_000));
        assert_eq!(
            process_telemetry.process_events_backend.as_deref(),
            Some("ptrace")
        );
        assert_eq!(process_telemetry.process_events.len(), 2);
        assert_eq!(process_telemetry.process_events[0].argv_omitted, Some(true));
        assert_eq!(
            process_telemetry.process_events[0].parent_span,
            "spn_parent"
        );
        assert_eq!(
            process_telemetry.process_events[0].parent_cmd_id.as_deref(),
            Some("cmd_parent")
        );
        assert_eq!(process_telemetry.process_events[1].exit_code, Some(0));
        assert_eq!(process_telemetry.process_events[1].duration_ms, Some(11));
    }
}
