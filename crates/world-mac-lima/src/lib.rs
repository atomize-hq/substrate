#![cfg(target_os = "macos")]
//! macOS world backend using Lima VM with Linux isolation inside.
//!
//! This backend provides identical policy enforcement semantics to LinuxLocal
//! by running a Linux VM via Lima and delegating to the world-service inside.

use anyhow::{Context, Result};
use serde::Deserialize;
use std::path::PathBuf;
use substrate_common::FsDiff;
use tokio::runtime::Runtime;
use transport_api_client::AgentClient;
use transport_api_types::{
    ExecuteRequest, ExecuteResponse, MemberDispatchRequestV1,
    MemberRuntimeBackendKindV1 as AgentMemberRuntimeBackendKindV1, PolicySnapshotV3,
    PolicySnapshotWorldFsDimensionV3, PolicySnapshotWorldFsFailClosedV3, PolicySnapshotWorldFsV3,
    PolicySnapshotWorldFsWriteV3, ResolvedMemberRuntimeDescriptorV1, WorldFsDenyEnforcementV3,
    WorldFsMode, WorldNetworkRoutingV1,
};
use world_api::{
    BackendPolicyInputV1, BackendPolicySnapshotV3, BackendPolicySnapshotWorldFsDimensionV3,
    BackendWorldFsDenyEnforcementV3, ExecRequest, ExecResult, SharedWorldBindingSnapshot,
    SharedWorldBindingState, SharedWorldOwnerAction, WorldBackend, WorldHandle, WorldSpec,
};

pub mod forwarding;
mod limactl;
pub mod transport;
pub mod vm;

use crate::transport::{
    managed_host_socket_path, CANONICAL_GUEST_SOCKET_PATH, COMPATIBILITY_TCP_HOST,
    COMPATIBILITY_TCP_PORT,
};
pub use forwarding::{ForwardingHandle, ForwardingKind};
pub use transport::Transport;
pub use vm::LimaVM;

/// macOS backend that delegates to Linux world-service inside Lima VM.
pub struct MacLimaBackend {
    vm_name: String,
    agent_socket: PathBuf,
    transport: Transport,
    runtime: Option<Runtime>,
    forwarding: std::sync::Mutex<Option<ForwardingHandle>>,
    session_cache: std::sync::Mutex<Option<WorldHandle>>,
    shared_owner_cache: std::sync::Mutex<std::collections::HashMap<String, WorldHandle>>,
    shared_owner_mutex: std::sync::Mutex<()>,
    world_policy_state:
        std::sync::Mutex<std::collections::HashMap<String, MacWorldPolicyState>>,
    #[cfg(test)]
    session_setup_override: Option<std::sync::Arc<dyn SessionSetupMock>>,
}

#[derive(Clone)]
struct MacWorldPolicyState {
    fs_mode: WorldFsMode,
    backend_policy: Option<BackendPolicyInputV1>,
}

impl MacLimaBackend {
    pub fn new() -> Result<Self> {
        let vm_name = std::env::var("SUBSTRATE_LIMA_VM_NAME")
            .or_else(|_| std::env::var("LIMA_VM_NAME"))
            .unwrap_or_else(|_| "substrate".to_string());
        let agent_socket = managed_host_socket_path();

        // Auto-select best transport
        let transport = Transport::auto_select()?;

        // Create dedicated runtime
        let runtime = Self::new_runtime()?;

        Ok(Self {
            vm_name,
            agent_socket,
            transport,
            runtime: Some(runtime),
            forwarding: std::sync::Mutex::new(None),
            session_cache: std::sync::Mutex::new(None),
            shared_owner_cache: std::sync::Mutex::new(std::collections::HashMap::new()),
            shared_owner_mutex: std::sync::Mutex::new(()),
            world_policy_state: std::sync::Mutex::new(std::collections::HashMap::new()),
            #[cfg(test)]
            session_setup_override: None,
        })
    }

    // Helper to drive an async future whether or not a Tokio runtime is already active.
    fn block_on_compat<F, T>(&self, fut: F) -> T
    where
        F: std::future::Future<Output = T>,
    {
        // If we're already inside a Tokio runtime (e.g., replay runner), use block_in_place
        // to synchronously wait without nesting a runtime. Otherwise, use our private runtime.
        if tokio::runtime::Handle::try_current().is_ok() {
            tokio::task::block_in_place(|| tokio::runtime::Handle::current().block_on(fut))
        } else {
            self.runtime
                .as_ref()
                .expect("internal runtime missing")
                .block_on(fut)
        }
    }

    fn new_runtime() -> Result<Runtime> {
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .context("Failed to create tokio runtime")
    }

    pub fn new_with_vm_name(vm_name: String) -> Result<Self> {
        let mut backend = Self::new()?;
        backend.vm_name = vm_name;
        Ok(backend)
    }

    #[cfg(test)]
    pub(crate) fn with_session_setup_mock(
        session_setup_override: std::sync::Arc<dyn SessionSetupMock>,
    ) -> Result<Self> {
        let mut backend = Self::new()?;
        backend.session_setup_override = Some(session_setup_override);
        Ok(backend)
    }

    fn ensure_vm_running(&self) -> Result<()> {
        tracing::debug!("Checking if Lima VM '{}' is running", self.vm_name);

        // Check if VM exists and is running (robust JSON check)
        let output = limactl::command()
            .context("Failed to execute limactl")?
            .args(["list", "--json"])
            .output()
            .context("Failed to execute limactl")?;

        if !output.status.success() {
            anyhow::bail!(
                "limactl failed: {}",
                String::from_utf8_lossy(&output.stderr)
            );
        }

        #[derive(Deserialize)]
        struct Instance {
            name: String,
            status: String,
        }

        // limactl list --json returns either a single object or an array
        let running = if let Ok(instance) = serde_json::from_slice::<Instance>(&output.stdout) {
            instance.name == self.vm_name && instance.status == "Running"
        } else if let Ok(instances) = serde_json::from_slice::<Vec<Instance>>(&output.stdout) {
            instances
                .iter()
                .any(|i| i.name == self.vm_name && i.status == "Running")
        } else {
            // If we can't parse, assume not running
            false
        };

        if !running {
            // Start VM (idempotent)
            tracing::info!("Starting Lima VM '{}'...", self.vm_name);
            let status = limactl::command()
                .context("Failed to start Lima VM")?
                .args(["start", &self.vm_name, "--tty=false"])
                .status()
                .context("Failed to start Lima VM")?;

            if !status.success() {
                anyhow::bail!(
                    "Failed to start Lima VM '{}'. Run scripts/mac/lima-doctor.sh for diagnostics",
                    self.vm_name
                );
            }

            // Wait for agent to be ready
            self.wait_for_agent()?;
        } else {
            tracing::debug!("Lima VM '{}' is already running", self.vm_name);
        }

        Ok(())
    }

    fn wait_for_agent(&self) -> Result<()> {
        let max_attempts = 30;
        let mut attempts = 0;

        tracing::info!("Waiting for world-service to be ready...");

        while attempts < max_attempts {
            // Try to connect to the agent socket
            if self.test_agent_connection().is_ok() {
                tracing::info!("World-agent is ready");
                return Ok(());
            }

            attempts += 1;
            if attempts % 5 == 0 {
                tracing::debug!(
                    "Still waiting for world-service... ({}/{})",
                    attempts,
                    max_attempts
                );
            }
            std::thread::sleep(std::time::Duration::from_secs(2));
        }

        anyhow::bail!(
            "Agent failed to start within timeout. Run scripts/mac/lima-doctor.sh for diagnostics"
        )
    }

    fn test_agent_connection(&self) -> Result<()> {
        let forwarding_established = self
            .forwarding
            .lock()
            .map_err(|e| anyhow::anyhow!("Lock poisoned: {}", e))?
            .is_some();

        // Simple test: ensure agent socket is present and reachable
        match &self.transport {
            Transport::UnixSocket => {
                if forwarding_established {
                    std::os::unix::net::UnixStream::connect(&self.agent_socket)
                        .context("Failed to connect to agent socket")?;
                } else {
                    self.check_agent_socket_in_vm()?;
                }
            }
            Transport::VSock => {
                self.check_agent_socket_in_vm()?;
            }
            Transport::TCP => {
                if forwarding_established {
                    std::net::TcpStream::connect((COMPATIBILITY_TCP_HOST, COMPATIBILITY_TCP_PORT))
                        .context("Failed to connect to agent TCP port")?;
                } else {
                    self.check_agent_socket_in_vm()?;
                }
            }
        }
        Ok(())
    }

    fn check_agent_socket_in_vm(&self) -> Result<()> {
        let output = limactl::command()
            .context("Failed to check agent socket in VM")?
            .args([
                "shell",
                &self.vm_name,
                "sudo",
                "-n",
                "test",
                "-S",
                CANONICAL_GUEST_SOCKET_PATH,
            ])
            .output()
            .context("Failed to check agent socket in VM")?;

        if !output.status.success() {
            anyhow::bail!("Agent socket not found in VM");
        }

        Ok(())
    }

    fn ensure_forwarding(&self) -> Result<()> {
        let mut forwarding = self
            .forwarding
            .lock()
            .map_err(|e| anyhow::anyhow!("Lock poisoned: {}", e))?;
        if forwarding.is_none() {
            tracing::debug!("Setting up forwarding for VM '{}'", self.vm_name);
            let handle = forwarding::auto_select(&self.vm_name)?;
            tracing::debug!("Forwarding established: {:?}", handle.kind());
            *forwarding = Some(handle);
        }
        Ok(())
    }

    fn ensure_session_setup(&self) -> Result<()> {
        #[cfg(test)]
        if let Some(override_impl) = &self.session_setup_override {
            return override_impl.ensure_setup();
        }

        self.ensure_vm_running()?;
        self.ensure_forwarding()?;
        Ok(())
    }

    async fn verify_agent_ready(&self) -> Result<()> {
        #[cfg(test)]
        if let Some(override_impl) = &self.session_setup_override {
            return override_impl.verify_ready();
        }

        let client = self.build_agent_client()?;
        let caps = client
            .capabilities()
            .await
            .context("Failed to verify agent connectivity")?;

        tracing::info!("Agent connectivity verified: {:?}", caps);
        Ok(())
    }

    pub async fn ensure_persistent_session_ready_async(&self) -> Result<()> {
        self.ensure_session_setup()?;
        self.verify_agent_ready().await
    }

    fn ensure_agent_ready(&self) -> Result<()> {
        self.block_on_compat(self.ensure_persistent_session_ready_async())
    }

    fn shared_world_id(&self, orchestration_session_id: &str, world_generation: u64) -> String {
        format!(
            "vm:{}:{}:{}",
            self.vm_name, orchestration_session_id, world_generation
        )
    }

    fn ensure_shared_owner_session(
        &self,
        owner: &world_api::SharedWorldOwnerSpec,
    ) -> Result<WorldHandle> {
        let _guard = self
            .shared_owner_mutex
            .lock()
            .map_err(|e| anyhow::anyhow!("Lock poisoned: {}", e))?;
        let mut cache = self
            .shared_owner_cache
            .lock()
            .map_err(|e| anyhow::anyhow!("Lock poisoned: {}", e))?;

        match &owner.action {
            SharedWorldOwnerAction::AttachOrCreate => {
                if let Some(handle) = cache.get(&owner.orchestration_session_id) {
                    return Ok(handle.clone());
                }

                let world_id = self.shared_world_id(&owner.orchestration_session_id, 0);
                let handle = WorldHandle {
                    id: world_id.clone(),
                    shared_binding: Some(SharedWorldBindingSnapshot {
                        orchestration_session_id: owner.orchestration_session_id.clone(),
                        world_id,
                        world_generation: 0,
                        binding_state: SharedWorldBindingState::Active,
                    }),
                };
                cache.insert(owner.orchestration_session_id.clone(), handle.clone());
                Ok(handle)
            }
            SharedWorldOwnerAction::ReplaceExpectedGeneration {
                expected_generation,
                reason: _,
            } => {
                let current = cache
                    .get(&owner.orchestration_session_id)
                    .cloned()
                    .ok_or_else(|| {
                        anyhow::anyhow!(
                            "no active shared world found for orchestration session {}",
                            owner.orchestration_session_id
                        )
                    })?;
                let current_generation = current
                    .shared_binding
                    .as_ref()
                    .map(|binding| binding.world_generation)
                    .ok_or_else(|| anyhow::anyhow!("active shared world missing binding proof"))?;
                if current_generation != *expected_generation {
                    anyhow::bail!(
                        "shared world generation conflict for {}: expected {}, found {}",
                        owner.orchestration_session_id,
                        expected_generation,
                        current_generation
                    );
                }

                let next_generation = expected_generation + 1;
                let world_id =
                    self.shared_world_id(&owner.orchestration_session_id, next_generation);
                let handle = WorldHandle {
                    id: world_id.clone(),
                    shared_binding: Some(SharedWorldBindingSnapshot {
                        orchestration_session_id: owner.orchestration_session_id.clone(),
                        world_id,
                        world_generation: next_generation,
                        binding_state: SharedWorldBindingState::Active,
                    }),
                };
                cache.insert(owner.orchestration_session_id.clone(), handle.clone());
                Ok(handle)
            }
        }
    }

    fn get_agent_endpoint(&self) -> Result<transport_api_client::Transport> {
        let forwarding = self
            .forwarding
            .lock()
            .map_err(|e| anyhow::anyhow!("Lock poisoned: {}", e))?;
        match forwarding.as_ref() {
            Some(handle) => match handle.kind() {
                ForwardingKind::SshUds { path } => {
                    Ok(transport_api_client::Transport::UnixSocket { path: path.clone() })
                }
                ForwardingKind::SshTcp { port } | ForwardingKind::Vsock { port } => {
                    Ok(transport_api_client::Transport::Tcp {
                        host: "127.0.0.1".to_string(),
                        port: *port,
                    })
                }
            },
            None => anyhow::bail!("Forwarding not established"),
        }
    }

    /// Convert world_api::ExecRequest to transport_api_types::ExecuteRequest.
    fn convert_exec_request(
        &self,
        req: &ExecRequest,
        world_state: &MacWorldPolicyState,
    ) -> Result<ExecuteRequest> {
        let backend_policy = world_state.backend_policy.as_ref().ok_or_else(|| {
            anyhow::anyhow!(
                "macOS backend missing authoritative backend_policy; refusing synthetic fallback"
            )
        })?;
        let policy_snapshot = convert_backend_policy_snapshot(&backend_policy.policy_snapshot)?;
        let world_network = convert_backend_world_network(backend_policy);

        Ok(ExecuteRequest {
            profile: None,
            cmd: req.cmd.clone(),
            cwd: Some(req.cwd.to_string_lossy().to_string()),
            env: Some(req.env.clone()),
            pty: req.pty,
            agent_id: "world-mac-lima".to_string(),
            budget: None,
            policy_snapshot,
            shared_world: req.shared_world.clone(),
            world_network: Some(world_network),
            world_fs_mode: Some(world_state.fs_mode),
            member_dispatch: req.member_dispatch.as_ref().map(convert_member_dispatch),
        })
    }

    /// Convert transport_api_types::ExecuteResponse to world_api::ExecResult.
    fn convert_exec_response(&self, resp: ExecuteResponse) -> ExecResult {
        use base64::Engine;
        let engine = base64::engine::general_purpose::STANDARD;

        ExecResult {
            exit: resp.exit,
            stdout: engine
                .decode(&resp.stdout_b64)
                .unwrap_or_else(|_| resp.stdout_b64.into_bytes()),
            stderr: engine
                .decode(&resp.stderr_b64)
                .unwrap_or_else(|_| resp.stderr_b64.into_bytes()),
            scopes_used: resp.scopes_used,
            fs_diff: resp.fs_diff,
            world_fs_strategy_primary: None,
            world_fs_strategy_final: None,
            world_fs_strategy_fallback_reason: None,
            process_telemetry: resp.process_telemetry,
        }
    }

    fn effective_fs_mode(&self, fs_mode: WorldFsMode) -> WorldFsMode {
        if let Ok(raw) = std::env::var("SUBSTRATE_WORLD_FS_MODE") {
            if let Some(mode) = WorldFsMode::parse(&raw) {
                return mode;
            }
        }
        fs_mode
    }

    fn store_world_policy_state(
        &self,
        world_id: &str,
        fs_mode: WorldFsMode,
        backend_policy: Option<BackendPolicyInputV1>,
    ) -> Result<()> {
        let mut guard = self
            .world_policy_state
            .lock()
            .map_err(|e| anyhow::anyhow!("Lock poisoned: {}", e))?;
        guard.insert(
            world_id.to_string(),
            MacWorldPolicyState {
                fs_mode,
                backend_policy,
            },
        );
        Ok(())
    }

    fn effective_world_policy_state(&self, world: &WorldHandle) -> Result<MacWorldPolicyState> {
        let guard = self
            .world_policy_state
            .lock()
            .map_err(|e| anyhow::anyhow!("Lock poisoned: {}", e))?;
        let mut state = guard.get(&world.id).cloned().ok_or_else(|| {
            anyhow::anyhow!(
                "macOS backend missing policy state for world {}; ensure_session/apply_policy must establish it before exec",
                world.id
            )
        })?;
        state.fs_mode = self.effective_fs_mode(state.fs_mode);
        Ok(state)
    }

    fn cache_world_policy_state(&self, world: &WorldHandle, spec: &WorldSpec) -> Result<()> {
        self.store_world_policy_state(&world.id, spec.fs_mode, spec.backend_policy.clone())
    }

    fn new_generic_world_handle(&self, reuse_session: bool) -> WorldHandle {
        let id = if reuse_session {
            format!("vm:{}", self.vm_name)
        } else {
            format!("vm:{}:{}", self.vm_name, uuid::Uuid::now_v7())
        };
        WorldHandle {
            id,
            shared_binding: None,
        }
    }

    /// Build an AgentClient based on current forwarding.
    fn build_agent_client(&self) -> Result<AgentClient> {
        let transport = self.get_agent_endpoint()?;
        AgentClient::new(transport)
    }
}

impl Default for MacLimaBackend {
    fn default() -> Self {
        Self::new().expect("Failed to create MacLimaBackend")
    }
}

impl Drop for MacLimaBackend {
    fn drop(&mut self) {
        tracing::debug!("Shutting down MacLimaBackend runtime");
        // Drop the internal runtime outside of any active Tokio context to avoid panics
        if let Some(rt) = self.runtime.take() {
            if tokio::runtime::Handle::try_current().is_ok() {
                let _ = std::thread::spawn(move || drop(rt)).join();
            } else {
                drop(rt);
            }
        }
    }
}

impl WorldBackend for MacLimaBackend {
    fn ensure_session(&self, spec: &WorldSpec) -> Result<WorldHandle> {
        self.ensure_agent_ready()?;

        if let Some(owner) = spec.reuse_mode.shared_owner() {
            let handle = self.ensure_shared_owner_session(owner)?;
            self.cache_world_policy_state(&handle, spec)?;
            return Ok(handle);
        }

        // Cache session if requested
        if spec.reuse_session {
            let cache = self
                .session_cache
                .lock()
                .map_err(|e| anyhow::anyhow!("Lock poisoned: {}", e))?;
            if let Some(ref handle) = *cache {
                tracing::debug!("Reusing cached world session: {}", handle.id);
                self.cache_world_policy_state(handle, spec)?;
                return Ok(handle.clone());
            }
        }

        let handle = self.new_generic_world_handle(spec.reuse_session);

        if spec.reuse_session {
            let mut cache = self
                .session_cache
                .lock()
                .map_err(|e| anyhow::anyhow!("Lock poisoned: {}", e))?;
            *cache = Some(handle.clone());
        }

        self.cache_world_policy_state(&handle, spec)?;
        Ok(handle)
    }

    fn exec(&self, world: &WorldHandle, req: ExecRequest) -> Result<ExecResult> {
        // Build agent client
        let client = self.build_agent_client()?;

        // Convert request
        let world_state = self.effective_world_policy_state(world)?;
        let agent_req = self.convert_exec_request(&req, &world_state)?;

        // Execute via agent
        let resp = self.block_on_compat(async { client.execute(agent_req).await })?;

        // Convert response
        Ok(self.convert_exec_response(resp))
    }

    fn fs_diff(&self, _world: &WorldHandle, span_id: &str) -> Result<FsDiff> {
        // Build agent client
        let client = self.build_agent_client()?;

        // Get trace (which includes fs_diff)
        let trace = self.block_on_compat(async { client.get_trace(span_id).await })?;

        // Extract fs_diff from trace response
        if let Some(fs_diff) = trace.get("fs_diff") {
            // Parse the fs_diff from the JSON value
            let diff: FsDiff = serde_json::from_value(fs_diff.clone())
                .context("Failed to parse fs_diff from trace")?;
            Ok(diff)
        } else {
            // Return empty diff if not available
            Ok(FsDiff::default())
        }
    }

    fn apply_policy(&self, world: &WorldHandle, spec: &WorldSpec) -> Result<()> {
        self.store_world_policy_state(&world.id, spec.fs_mode, spec.backend_policy.clone())?;
        let Some(_backend_policy) = spec.backend_policy.clone() else {
            anyhow::bail!(
                "macOS backend requires authoritative backend_policy when applying policy"
            );
        };
        tracing::debug!(
            world_id = %world.id,
            has_backend_policy = true,
            "Updated macOS backend policy state for backend-mediated execution"
        );
        Ok(())
    }
}

fn convert_backend_world_network(backend_policy: &BackendPolicyInputV1) -> WorldNetworkRoutingV1 {
    WorldNetworkRoutingV1 {
        isolate_network: backend_policy.world_network.isolate_network,
        allowed_domains: backend_policy.world_network.allowed_domains.clone(),
    }
}

fn convert_backend_policy_snapshot(snapshot: &BackendPolicySnapshotV3) -> Result<PolicySnapshotV3> {
    PolicySnapshotV3 {
        schema_version: snapshot.schema_version,
        net_allowed: snapshot.net_allowed.clone(),
        world_fs: PolicySnapshotWorldFsV3 {
            host_visible: snapshot.world_fs.host_visible,
            fail_closed: PolicySnapshotWorldFsFailClosedV3 {
                routing: snapshot.world_fs.fail_closed.routing,
            },
            deny_enforcement: snapshot
                .world_fs
                .deny_enforcement
                .map(convert_backend_deny_enforcement),
            caged_required: snapshot.world_fs.caged_required,
            discover: snapshot
                .world_fs
                .discover
                .as_ref()
                .map(convert_backend_world_fs_dimension),
            read: snapshot
                .world_fs
                .read
                .as_ref()
                .map(convert_backend_world_fs_dimension),
            write: PolicySnapshotWorldFsWriteV3 {
                enabled: snapshot.world_fs.write.enabled,
                allow_list: snapshot.world_fs.write.allow_list.clone(),
                deny_list: snapshot.world_fs.write.deny_list.clone(),
            },
        },
    }
    .canonicalize()
    .map_err(|err| anyhow::anyhow!("invalid backend policy snapshot for macOS backend: {err}"))
}

fn convert_backend_world_fs_dimension(
    dimension: &BackendPolicySnapshotWorldFsDimensionV3,
) -> PolicySnapshotWorldFsDimensionV3 {
    PolicySnapshotWorldFsDimensionV3 {
        allow_list: dimension.allow_list.clone(),
        deny_list: dimension.deny_list.clone(),
    }
}

fn convert_backend_deny_enforcement(
    deny_enforcement: BackendWorldFsDenyEnforcementV3,
) -> WorldFsDenyEnforcementV3 {
    match deny_enforcement {
        BackendWorldFsDenyEnforcementV3::Strict => WorldFsDenyEnforcementV3::Strict,
        BackendWorldFsDenyEnforcementV3::PreferStrict => WorldFsDenyEnforcementV3::PreferStrict,
        BackendWorldFsDenyEnforcementV3::Weak => WorldFsDenyEnforcementV3::Weak,
    }
}

fn convert_member_dispatch(
    dispatch: &world_api::MemberDispatchRequestV1,
) -> MemberDispatchRequestV1 {
    MemberDispatchRequestV1 {
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
        resolved_runtime: ResolvedMemberRuntimeDescriptorV1 {
            backend_kind: convert_member_runtime_backend_kind(
                dispatch.resolved_runtime.backend_kind.clone(),
            ),
            binary_path: dispatch.resolved_runtime.binary_path.clone(),
        },
    }
}

fn convert_member_runtime_backend_kind(
    backend_kind: world_api::MemberRuntimeBackendKindV1,
) -> AgentMemberRuntimeBackendKindV1 {
    match backend_kind {
        world_api::MemberRuntimeBackendKindV1::Codex => AgentMemberRuntimeBackendKindV1::Codex,
        world_api::MemberRuntimeBackendKindV1::ClaudeCode => {
            AgentMemberRuntimeBackendKindV1::ClaudeCode
        }
    }
}

#[cfg(test)]
pub(crate) trait SessionSetupMock: Send + Sync {
    fn ensure_setup(&self) -> Result<()>;
    fn verify_ready(&self) -> Result<()>;
}

#[cfg(test)]
mod test_util {
    use std::sync::{LazyLock, Mutex, MutexGuard};

    static ENV_LOCK: LazyLock<Mutex<()>> = LazyLock::new(|| Mutex::new(()));

    pub(crate) fn lock_env() -> MutexGuard<'static, ()> {
        ENV_LOCK.lock().expect("ENV_LOCK poisoned")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;

    fn sample_backend_policy(
        net_allowed: &[&str],
        allowed_domains: &[&str],
        write_enabled: bool,
        isolate_network: bool,
    ) -> BackendPolicyInputV1 {
        BackendPolicyInputV1 {
            schema_version: 1,
            policy_snapshot: BackendPolicySnapshotV3 {
                schema_version: 3,
                net_allowed: net_allowed
                    .iter()
                    .map(|entry| (*entry).to_string())
                    .collect(),
                world_fs: world_api::BackendPolicySnapshotWorldFsV3 {
                    host_visible: false,
                    fail_closed: world_api::BackendPolicySnapshotWorldFsFailClosedV3 {
                        routing: true,
                    },
                    deny_enforcement: Some(BackendWorldFsDenyEnforcementV3::Strict),
                    caged_required: true,
                    discover: Some(world_api::BackendPolicySnapshotWorldFsDimensionV3 {
                        allow_list: vec![".".to_string()],
                        deny_list: vec!["tmp".to_string()],
                    }),
                    read: Some(world_api::BackendPolicySnapshotWorldFsDimensionV3 {
                        allow_list: vec![".".to_string()],
                        deny_list: vec!["private".to_string()],
                    }),
                    write: world_api::BackendPolicySnapshotWorldFsWriteV3 {
                        enabled: write_enabled,
                        allow_list: vec!["out".to_string()],
                        deny_list: vec!["out/blocked".to_string()],
                    },
                },
            },
            world_network: world_api::BackendWorldNetworkRoutingV1 {
                isolate_network,
                allowed_domains: allowed_domains
                    .iter()
                    .map(|entry| (*entry).to_string())
                    .collect(),
            },
        }
    }

    struct AlwaysReadySessionSetup {
        setup_calls: AtomicUsize,
        verify_calls: AtomicUsize,
    }

    impl AlwaysReadySessionSetup {
        fn new() -> Self {
            Self {
                setup_calls: AtomicUsize::new(0),
                verify_calls: AtomicUsize::new(0),
            }
        }
    }

    impl SessionSetupMock for AlwaysReadySessionSetup {
        fn ensure_setup(&self) -> Result<()> {
            self.setup_calls.fetch_add(1, Ordering::SeqCst);
            Ok(())
        }

        fn verify_ready(&self) -> Result<()> {
            self.verify_calls.fetch_add(1, Ordering::SeqCst);
            Ok(())
        }
    }

    #[test]
    fn test_backend_creation() {
        let _env_guard = crate::test_util::lock_env();
        let prev_substrate_lima_vm_name = std::env::var_os("SUBSTRATE_LIMA_VM_NAME");
        let prev_lima_vm_name = std::env::var_os("LIMA_VM_NAME");
        std::env::remove_var("SUBSTRATE_LIMA_VM_NAME");
        std::env::remove_var("LIMA_VM_NAME");

        // This should work even if Lima is not installed
        match MacLimaBackend::new() {
            Ok(backend) => {
                assert_eq!(backend.vm_name, "substrate");
                assert!(backend
                    .agent_socket
                    .to_string_lossy()
                    .contains(".substrate/sock"));
            }
            Err(e) => {
                println!("Expected failure when Lima not available: {}", e);
            }
        }

        match prev_substrate_lima_vm_name {
            Some(value) => std::env::set_var("SUBSTRATE_LIMA_VM_NAME", value),
            None => std::env::remove_var("SUBSTRATE_LIMA_VM_NAME"),
        }
        match prev_lima_vm_name {
            Some(value) => std::env::set_var("LIMA_VM_NAME", value),
            None => std::env::remove_var("LIMA_VM_NAME"),
        }
    }

    #[test]
    fn test_backend_vm_name_override_prefers_substrate_env() {
        let _env_guard = crate::test_util::lock_env();
        let prev_substrate_lima_vm_name = std::env::var_os("SUBSTRATE_LIMA_VM_NAME");
        let prev_lima_vm_name = std::env::var_os("LIMA_VM_NAME");

        std::env::set_var("LIMA_VM_NAME", "substrate-arch");
        std::env::set_var("SUBSTRATE_LIMA_VM_NAME", "substrate-arch-override");

        let backend = MacLimaBackend::new().expect("backend");
        assert_eq!(backend.vm_name, "substrate-arch-override");

        match prev_substrate_lima_vm_name {
            Some(value) => std::env::set_var("SUBSTRATE_LIMA_VM_NAME", value),
            None => std::env::remove_var("SUBSTRATE_LIMA_VM_NAME"),
        }
        match prev_lima_vm_name {
            Some(value) => std::env::set_var("LIMA_VM_NAME", value),
            None => std::env::remove_var("LIMA_VM_NAME"),
        }
    }

    #[test]
    fn test_backend_creation_uses_substrate_home_without_home() {
        let _env_guard = crate::test_util::lock_env();
        let prev_home = std::env::var_os("HOME");
        let prev_substrate_home = std::env::var_os("SUBSTRATE_HOME");
        let prev_substrate_lima_vm_name = std::env::var_os("SUBSTRATE_LIMA_VM_NAME");
        let prev_lima_vm_name = std::env::var_os("LIMA_VM_NAME");

        std::env::remove_var("HOME");
        std::env::set_var("SUBSTRATE_HOME", "/tmp/substrate-home-only");
        std::env::remove_var("SUBSTRATE_LIMA_VM_NAME");
        std::env::remove_var("LIMA_VM_NAME");

        let backend = MacLimaBackend::new().expect("backend should honor SUBSTRATE_HOME");
        assert_eq!(
            backend.agent_socket,
            PathBuf::from("/tmp/substrate-home-only/sock/agent.sock")
        );

        match prev_home {
            Some(value) => std::env::set_var("HOME", value),
            None => std::env::remove_var("HOME"),
        }
        match prev_substrate_home {
            Some(value) => std::env::set_var("SUBSTRATE_HOME", value),
            None => std::env::remove_var("SUBSTRATE_HOME"),
        }
        match prev_substrate_lima_vm_name {
            Some(value) => std::env::set_var("SUBSTRATE_LIMA_VM_NAME", value),
            None => std::env::remove_var("SUBSTRATE_LIMA_VM_NAME"),
        }
        match prev_lima_vm_name {
            Some(value) => std::env::set_var("LIMA_VM_NAME", value),
            None => std::env::remove_var("LIMA_VM_NAME"),
        }
    }

    #[test]
    fn test_transport_selection() {
        let transport = Transport::auto_select().unwrap();
        // Should pick a reasonable default
        matches!(
            transport,
            Transport::UnixSocket | Transport::TCP | Transport::VSock
        );
    }

    #[test]
    fn convert_exec_request_propagates_env_fs_mode() {
        let _env_guard = crate::test_util::lock_env();
        let prev = std::env::var("SUBSTRATE_WORLD_FS_MODE").ok();
        std::env::set_var("SUBSTRATE_WORLD_FS_MODE", "read_only");

        if let Ok(backend) = MacLimaBackend::new() {
            let world = WorldHandle {
                id: "vm:substrate".to_string(),
                shared_binding: None,
            };
            let spec = WorldSpec {
                fs_mode: WorldFsMode::ReadOnly,
                backend_policy: Some(sample_backend_policy(
                    &["https://api.example.com"],
                    &["api.example.com"],
                    false,
                    true,
                )),
                ..WorldSpec::default()
            };
            backend.apply_policy(&world, &spec).expect("apply policy");

            let req = ExecRequest {
                cmd: "echo hi".to_string(),
                cwd: PathBuf::from("/tmp"),
                env: std::collections::HashMap::new(),
                pty: false,
                span_id: None,
                shared_world: Some(world_api::SharedWorldOwnerSpec {
                    orchestration_session_id: "orch_123".to_string(),
                    action: world_api::SharedWorldOwnerAction::AttachOrCreate,
                }),
                member_dispatch: Some(world_api::MemberDispatchRequestV1 {
                    schema_version: 1,
                    orchestration_session_id: "orch_123".to_string(),
                    participant_id: "participant_123".to_string(),
                    orchestrator_participant_id: "participant_root".to_string(),
                    parent_participant_id: None,
                    resumed_from_participant_id: None,
                    backend_id: "backend_123".to_string(),
                    protocol: "stdio".to_string(),
                    run_id: "run_123".to_string(),
                    world_id: "wld_123".to_string(),
                    world_generation: 0,
                    initial_prompt: Some("prompt".to_string()),
                    resolved_runtime: world_api::ResolvedMemberRuntimeDescriptorV1 {
                        backend_kind: world_api::MemberRuntimeBackendKindV1::Codex,
                        binary_path: "/usr/bin/env".to_string(),
                    },
                }),
            };
            let world_state = backend
                .effective_world_policy_state(&world)
                .expect("world policy state");
            let agent_req = backend
                .convert_exec_request(&req, &world_state)
                .expect("convert exec request");
            assert_eq!(
                agent_req.world_fs_mode,
                Some(WorldFsMode::ReadOnly),
                "mac backend should pass through env-derived fs mode"
            );
            assert_eq!(agent_req.shared_world, req.shared_world);
            assert_eq!(
                agent_req.policy_snapshot.net_allowed,
                vec!["https://api.example.com".to_string()]
            );
            assert!(!agent_req.policy_snapshot.world_fs.host_visible);
            assert!(agent_req.policy_snapshot.world_fs.fail_closed.routing);
            assert!(!agent_req.policy_snapshot.world_fs.write.enabled);
            assert_eq!(
                agent_req.policy_snapshot.world_fs.write.allow_list,
                vec!["out".to_string()]
            );
            assert_eq!(
                agent_req.policy_snapshot.world_fs.write.deny_list,
                vec!["out/blocked".to_string()]
            );
            assert_eq!(
                agent_req.world_network,
                Some(WorldNetworkRoutingV1 {
                    isolate_network: true,
                    allowed_domains: vec!["api.example.com".to_string()],
                })
            );
            assert_eq!(
                agent_req.member_dispatch.as_ref().map(|dispatch| (
                    dispatch.orchestration_session_id.as_str(),
                    dispatch.participant_id.as_str(),
                    dispatch.orchestrator_participant_id.as_str(),
                    dispatch.backend_id.as_str(),
                    dispatch.world_id.as_str(),
                    dispatch.world_generation,
                )),
                Some((
                    "orch_123",
                    "participant_123",
                    "participant_root",
                    "backend_123",
                    "wld_123",
                    0,
                ))
            );
        }

        match prev {
            Some(value) => std::env::set_var("SUBSTRATE_WORLD_FS_MODE", value),
            None => std::env::remove_var("SUBSTRATE_WORLD_FS_MODE"),
        }
    }

    #[test]
    fn apply_policy_updates_backend_policy_state() {
        let _env_guard = crate::test_util::lock_env();

        if let Ok(backend) = MacLimaBackend::new() {
            let world = WorldHandle {
                id: "vm:substrate".to_string(),
                shared_binding: None,
            };
            let req = ExecRequest {
                cmd: "echo hi".to_string(),
                cwd: PathBuf::from("/tmp"),
                env: std::collections::HashMap::new(),
                pty: false,
                span_id: None,
                shared_world: None,
                member_dispatch: None,
            };

            let first = WorldSpec {
                backend_policy: Some(sample_backend_policy(
                    &["https://first.example.com"],
                    &["first.example.com"],
                    true,
                    true,
                )),
                ..WorldSpec::default()
            };
            backend
                .apply_policy(&world, &first)
                .expect("apply first policy");
            let first_state = backend
                .effective_world_policy_state(&world)
                .expect("first world state");
            let first_agent_req = backend
                .convert_exec_request(&req, &first_state)
                .expect("convert first request");
            assert_eq!(
                first_agent_req.policy_snapshot.net_allowed,
                vec!["https://first.example.com".to_string()]
            );
            assert_eq!(
                first_agent_req.world_network,
                Some(WorldNetworkRoutingV1 {
                    isolate_network: true,
                    allowed_domains: vec!["first.example.com".to_string()],
                })
            );

            let second = WorldSpec {
                backend_policy: Some(sample_backend_policy(
                    &["https://second.example.com"],
                    &["second.example.com"],
                    false,
                    false,
                )),
                ..WorldSpec::default()
            };
            backend
                .apply_policy(&world, &second)
                .expect("apply second policy");
            let second_state = backend
                .effective_world_policy_state(&world)
                .expect("second world state");
            let second_agent_req = backend
                .convert_exec_request(&req, &second_state)
                .expect("convert second request");
            assert_eq!(
                second_agent_req.policy_snapshot.net_allowed,
                vec!["https://second.example.com".to_string()]
            );
            assert_eq!(
                second_agent_req.world_network,
                Some(WorldNetworkRoutingV1 {
                    isolate_network: false,
                    allowed_domains: vec!["second.example.com".to_string()],
                })
            );
            assert!(!second_agent_req.policy_snapshot.world_fs.write.enabled);
        }
    }

    #[test]
    fn apply_policy_fail_closes_without_backend_policy() {
        let _env_guard = crate::test_util::lock_env();

        if let Ok(backend) = MacLimaBackend::new() {
            let world = WorldHandle {
                id: "vm:substrate".to_string(),
                shared_binding: None,
            };
            let req = ExecRequest {
                cmd: "echo hi".to_string(),
                cwd: PathBuf::from("/tmp"),
                env: std::collections::HashMap::new(),
                pty: false,
                span_id: None,
                shared_world: None,
                member_dispatch: None,
            };

            let err = backend
                .apply_policy(&world, &WorldSpec::default())
                .expect_err("missing backend policy must fail-close");
            assert!(
                err.to_string()
                    .contains("requires authoritative backend_policy"),
                "unexpected error: {err}"
            );

            let world_state = backend
                .effective_world_policy_state(&world)
                .expect("world state should be recorded fail-closed");
            let err = backend
                .convert_exec_request(&req, &world_state)
                .expect_err("missing policy state must remain fail-closed");
            assert!(
                err.to_string()
                    .contains("missing authoritative backend_policy"),
                "unexpected conversion error: {err}"
            );
        }
    }

    #[test]
    fn world_policy_state_is_scoped_per_world_handle() {
        let _env_guard = crate::test_util::lock_env();

        if let Ok(backend) = MacLimaBackend::new() {
            let first_world = WorldHandle {
                id: "vm:substrate:first".to_string(),
                shared_binding: None,
            };
            let second_world = WorldHandle {
                id: "vm:substrate:second".to_string(),
                shared_binding: None,
            };
            let req = ExecRequest {
                cmd: "echo hi".to_string(),
                cwd: PathBuf::from("/tmp"),
                env: std::collections::HashMap::new(),
                pty: false,
                span_id: None,
                shared_world: None,
                member_dispatch: None,
            };

            backend
                .apply_policy(
                    &first_world,
                    &WorldSpec {
                        fs_mode: WorldFsMode::ReadOnly,
                        backend_policy: Some(sample_backend_policy(
                            &["https://first.example.com"],
                            &["first.example.com"],
                            false,
                            true,
                        )),
                        ..WorldSpec::default()
                    },
                )
                .expect("apply first policy");
            backend
                .apply_policy(
                    &second_world,
                    &WorldSpec {
                        fs_mode: WorldFsMode::Writable,
                        backend_policy: Some(sample_backend_policy(
                            &["https://second.example.com"],
                            &["second.example.com"],
                            true,
                            false,
                        )),
                        ..WorldSpec::default()
                    },
                )
                .expect("apply second policy");

            let first_agent_req = backend
                .convert_exec_request(
                    &req,
                    &backend
                        .effective_world_policy_state(&first_world)
                        .expect("first world state"),
                )
                .expect("convert first request");
            let second_agent_req = backend
                .convert_exec_request(
                    &req,
                    &backend
                        .effective_world_policy_state(&second_world)
                        .expect("second world state"),
                )
                .expect("convert second request");

            assert_eq!(
                first_agent_req.policy_snapshot.net_allowed,
                vec!["https://first.example.com".to_string()]
            );
            assert_eq!(
                second_agent_req.policy_snapshot.net_allowed,
                vec!["https://second.example.com".to_string()]
            );
            assert_eq!(first_agent_req.world_fs_mode, Some(WorldFsMode::ReadOnly));
            assert_eq!(second_agent_req.world_fs_mode, Some(WorldFsMode::Writable));
            assert_eq!(
                first_agent_req.world_network,
                Some(WorldNetworkRoutingV1 {
                    isolate_network: true,
                    allowed_domains: vec!["first.example.com".to_string()],
                })
            );
            assert_eq!(
                second_agent_req.world_network,
                Some(WorldNetworkRoutingV1 {
                    isolate_network: false,
                    allowed_domains: vec!["second.example.com".to_string()],
                })
            );
        }
    }

    #[test]
    fn ensure_session_without_reuse_gets_distinct_world_scoped_policy_state() {
        let _env_guard = crate::test_util::lock_env();
        let session_setup = Arc::new(AlwaysReadySessionSetup::new());
        let backend =
            MacLimaBackend::with_session_setup_mock(session_setup.clone()).expect("backend");

        let first_spec = WorldSpec {
            reuse_session: false,
            fs_mode: WorldFsMode::ReadOnly,
            backend_policy: Some(sample_backend_policy(
                &["https://first.example.com"],
                &["first.example.com"],
                false,
                true,
            )),
            ..WorldSpec::default()
        };
        let second_spec = WorldSpec {
            reuse_session: false,
            fs_mode: WorldFsMode::Writable,
            backend_policy: Some(sample_backend_policy(
                &["https://second.example.com"],
                &["second.example.com"],
                true,
                false,
            )),
            ..WorldSpec::default()
        };

        let first_world = backend.ensure_session(&first_spec).expect("first world");
        let second_world = backend.ensure_session(&second_spec).expect("second world");
        assert_ne!(first_world.id, second_world.id);

        let req = ExecRequest {
            cmd: "echo hi".to_string(),
            cwd: PathBuf::from("/tmp"),
            env: std::collections::HashMap::new(),
            pty: false,
            span_id: None,
            shared_world: None,
            member_dispatch: None,
        };

        let first_agent_req = backend
            .convert_exec_request(
                &req,
                &backend
                    .effective_world_policy_state(&first_world)
                    .expect("first world state"),
            )
            .expect("convert first request");
        let second_agent_req = backend
            .convert_exec_request(
                &req,
                &backend
                    .effective_world_policy_state(&second_world)
                    .expect("second world state"),
            )
            .expect("convert second request");

        assert_eq!(first_agent_req.world_fs_mode, Some(WorldFsMode::ReadOnly));
        assert_eq!(second_agent_req.world_fs_mode, Some(WorldFsMode::Writable));
        assert_eq!(
            first_agent_req.policy_snapshot.net_allowed,
            vec!["https://first.example.com".to_string()]
        );
        assert_eq!(
            second_agent_req.policy_snapshot.net_allowed,
            vec!["https://second.example.com".to_string()]
        );
    }

    #[test]
    fn ensure_session_owner_mode_sets_authoritative_shared_binding() {
        let _env_guard = crate::test_util::lock_env();
        let session_setup = Arc::new(AlwaysReadySessionSetup::new());
        let backend =
            MacLimaBackend::with_session_setup_mock(session_setup.clone()).expect("backend");

        let attach = WorldSpec {
            reuse_mode: world_api::WorldReuseMode::SharedOrchestration(
                world_api::SharedWorldOwnerSpec {
                    orchestration_session_id: "orch_123".to_string(),
                    action: world_api::SharedWorldOwnerAction::AttachOrCreate,
                },
            ),
            ..WorldSpec::default()
        };
        let attached = backend
            .ensure_session(&attach)
            .expect("attach/create session");
        let attached_binding = attached.shared_binding.expect("attach/create binding");
        assert_eq!(attached_binding.orchestration_session_id, "orch_123");
        assert_eq!(attached_binding.world_id, attached.id);
        assert_eq!(attached_binding.world_generation, 0);
        assert_eq!(
            attached_binding.binding_state,
            SharedWorldBindingState::Active
        );

        let replace = WorldSpec {
            reuse_mode: world_api::WorldReuseMode::SharedOrchestration(
                world_api::SharedWorldOwnerSpec {
                    orchestration_session_id: "orch_123".to_string(),
                    action: world_api::SharedWorldOwnerAction::ReplaceExpectedGeneration {
                        expected_generation: attached_binding.world_generation,
                        reason: "restart".to_string(),
                    },
                },
            ),
            ..WorldSpec::default()
        };

        let replaced = backend
            .ensure_session(&replace)
            .expect("replacement session");
        let binding = replaced.shared_binding.expect("replacement shared binding");
        assert_eq!(binding.orchestration_session_id, "orch_123");
        assert_eq!(binding.world_id, replaced.id);
        assert_eq!(binding.world_generation, 1);
        assert_eq!(binding.binding_state, SharedWorldBindingState::Active);
        assert_ne!(binding.world_id, attached.id);
        assert_eq!(session_setup.setup_calls.load(Ordering::SeqCst), 2);
        assert_eq!(session_setup.verify_calls.load(Ordering::SeqCst), 2);
    }

    #[test]
    fn ensure_persistent_session_ready_async_runs_setup_and_verify() {
        let _env_guard = crate::test_util::lock_env();
        let session_setup = Arc::new(AlwaysReadySessionSetup::new());
        let backend =
            MacLimaBackend::with_session_setup_mock(session_setup.clone()).expect("backend");
        let runtime = MacLimaBackend::new_runtime().expect("runtime");

        runtime
            .block_on(backend.ensure_persistent_session_ready_async())
            .expect("async readiness");

        assert_eq!(session_setup.setup_calls.load(Ordering::SeqCst), 1);
        assert_eq!(session_setup.verify_calls.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn ensure_session_owner_mode_replace_requires_existing_generation() {
        let _env_guard = crate::test_util::lock_env();
        let backend =
            MacLimaBackend::with_session_setup_mock(Arc::new(AlwaysReadySessionSetup::new()))
                .expect("backend");

        let replace = WorldSpec {
            reuse_mode: world_api::WorldReuseMode::SharedOrchestration(
                world_api::SharedWorldOwnerSpec {
                    orchestration_session_id: "orch_123".to_string(),
                    action: world_api::SharedWorldOwnerAction::ReplaceExpectedGeneration {
                        expected_generation: 4,
                        reason: "restart".to_string(),
                    },
                },
            ),
            ..WorldSpec::default()
        };

        let err = backend
            .ensure_session(&replace)
            .expect_err("replace should fail closed");
        assert!(
            err.to_string()
                .contains("no active shared world found for orchestration session orch_123"),
            "unexpected error: {err:#}"
        );
    }
}
