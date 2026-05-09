#![cfg(target_os = "macos")]
//! macOS world backend using Lima VM with Linux isolation inside.
//!
//! This backend provides identical policy enforcement semantics to LinuxLocal
//! by running a Linux VM via Lima and delegating to the world-agent inside.

use agent_api_client::AgentClient;
use agent_api_types::{
    ExecuteRequest, ExecuteResponse, MemberDispatchRequestV1,
    MemberRuntimeBackendKindV1 as AgentMemberRuntimeBackendKindV1, PolicySnapshotV3,
    PolicySnapshotWorldFsDimensionV3, PolicySnapshotWorldFsFailClosedV3, PolicySnapshotWorldFsV3,
    PolicySnapshotWorldFsWriteV3, ResolvedMemberRuntimeDescriptorV1, WorldFsMode,
};
use anyhow::{Context, Result};
use serde::Deserialize;
use std::path::PathBuf;
use substrate_common::FsDiff;
use tokio::runtime::Runtime;
use world_api::{
    ExecRequest, ExecResult, SharedWorldBindingSnapshot, SharedWorldBindingState,
    SharedWorldOwnerAction, WorldBackend, WorldHandle, WorldSpec,
};

pub mod forwarding;
mod limactl;
pub mod transport;
pub mod vm;

pub use forwarding::{ForwardingHandle, ForwardingKind};
pub use transport::Transport;
pub use vm::LimaVM;

/// macOS backend that delegates to Linux world-agent inside Lima VM.
pub struct MacLimaBackend {
    vm_name: String,
    agent_socket: PathBuf,
    transport: Transport,
    runtime: Option<Runtime>,
    forwarding: std::sync::Mutex<Option<ForwardingHandle>>,
    session_cache: std::sync::Mutex<Option<WorldHandle>>,
    fs_mode: std::sync::Mutex<WorldFsMode>,
}

impl MacLimaBackend {
    pub fn new() -> Result<Self> {
        let vm_name = std::env::var("SUBSTRATE_LIMA_VM_NAME")
            .or_else(|_| std::env::var("LIMA_VM_NAME"))
            .unwrap_or_else(|_| "substrate".to_string());
        let agent_socket = dirs::home_dir()
            .ok_or_else(|| anyhow::anyhow!("home directory not found"))?
            .join(".substrate/sock/agent.sock");

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
            fs_mode: std::sync::Mutex::new(WorldFsMode::Writable),
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

        tracing::info!("Waiting for world-agent to be ready...");

        while attempts < max_attempts {
            // Try to connect to the agent socket
            if self.test_agent_connection().is_ok() {
                tracing::info!("World-agent is ready");
                return Ok(());
            }

            attempts += 1;
            if attempts % 5 == 0 {
                tracing::debug!(
                    "Still waiting for world-agent... ({}/{})",
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
                    std::net::TcpStream::connect("127.0.0.1:7788")
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
                "/run/substrate.sock",
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

    fn get_agent_endpoint(&self) -> Result<agent_api_client::Transport> {
        let forwarding = self
            .forwarding
            .lock()
            .map_err(|e| anyhow::anyhow!("Lock poisoned: {}", e))?;
        match forwarding.as_ref() {
            Some(handle) => match handle.kind() {
                ForwardingKind::SshUds { path } => {
                    Ok(agent_api_client::Transport::UnixSocket { path: path.clone() })
                }
                ForwardingKind::SshTcp { port } | ForwardingKind::Vsock { port } => {
                    Ok(agent_api_client::Transport::Tcp {
                        host: "127.0.0.1".to_string(),
                        port: *port,
                    })
                }
            },
            None => anyhow::bail!("Forwarding not established"),
        }
    }

    /// Convert world_api::ExecRequest to agent_api_types::ExecuteRequest.
    fn convert_exec_request(&self, req: &ExecRequest, fs_mode: WorldFsMode) -> ExecuteRequest {
        let write_enabled = matches!(fs_mode, WorldFsMode::Writable);
        let policy_snapshot = PolicySnapshotV3 {
            schema_version: 3,
            net_allowed: Vec::new(),
            world_fs: PolicySnapshotWorldFsV3 {
                host_visible: true,
                fail_closed: PolicySnapshotWorldFsFailClosedV3 { routing: false },
                deny_enforcement: None,
                caged_required: false,
                discover: Some(PolicySnapshotWorldFsDimensionV3 {
                    allow_list: vec![".".to_string()],
                    deny_list: Vec::new(),
                }),
                read: Some(PolicySnapshotWorldFsDimensionV3 {
                    allow_list: vec![".".to_string()],
                    deny_list: Vec::new(),
                }),
                write: PolicySnapshotWorldFsWriteV3 {
                    enabled: write_enabled,
                    allow_list: vec![".".to_string()],
                    deny_list: Vec::new(),
                },
            },
        };

        ExecuteRequest {
            profile: None,
            cmd: req.cmd.clone(),
            cwd: Some(req.cwd.to_string_lossy().to_string()),
            env: Some(req.env.clone()),
            pty: req.pty,
            agent_id: "world-mac-lima".to_string(),
            budget: None,
            policy_snapshot,
            shared_world: req.shared_world.clone(),
            world_network: None,
            world_fs_mode: Some(fs_mode),
            member_dispatch: req.member_dispatch.as_ref().map(convert_member_dispatch),
        }
    }

    /// Convert agent_api_types::ExecuteResponse to world_api::ExecResult.
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

    fn store_fs_mode(&self, fs_mode: WorldFsMode) -> Result<()> {
        let mut guard = self
            .fs_mode
            .lock()
            .map_err(|e| anyhow::anyhow!("Lock poisoned: {}", e))?;
        *guard = fs_mode;
        Ok(())
    }

    fn effective_fs_mode(&self) -> Result<WorldFsMode> {
        if let Ok(raw) = std::env::var("SUBSTRATE_WORLD_FS_MODE") {
            if let Some(mode) = WorldFsMode::parse(&raw) {
                return Ok(mode);
            }
        }
        let guard = self
            .fs_mode
            .lock()
            .map_err(|e| anyhow::anyhow!("Lock poisoned: {}", e))?;
        Ok(*guard)
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
        self.ensure_vm_running()?;
        self.ensure_forwarding()?;

        self.store_fs_mode(spec.fs_mode)?;

        // Cache session if requested
        if spec.reuse_session {
            let cache = self
                .session_cache
                .lock()
                .map_err(|e| anyhow::anyhow!("Lock poisoned: {}", e))?;
            if let Some(ref handle) = *cache {
                tracing::debug!("Reusing cached world session: {}", handle.id);
                return Ok(handle.clone());
            }
        }

        // Verify connectivity via agent client
        let client = self.build_agent_client()?;
        let caps = self
            .block_on_compat(async { client.capabilities().await })
            .context("Failed to verify agent connectivity")?;

        tracing::info!("Agent connectivity verified: {:?}", caps);

        // Generate world ID
        let world_id = format!("vm:{}", self.vm_name);

        let handle = WorldHandle {
            id: world_id.clone(),
            shared_binding: spec.reuse_mode.shared_owner().map(|owner| {
                SharedWorldBindingSnapshot {
                    orchestration_session_id: owner.orchestration_session_id.clone(),
                    world_id: world_id.clone(),
                    world_generation: match &owner.action {
                        SharedWorldOwnerAction::AttachOrCreate => 0,
                        SharedWorldOwnerAction::ReplaceExpectedGeneration {
                            expected_generation,
                            ..
                        } => expected_generation + 1,
                    },
                    binding_state: SharedWorldBindingState::Active,
                }
            }),
        };

        if spec.reuse_session {
            let mut cache = self
                .session_cache
                .lock()
                .map_err(|e| anyhow::anyhow!("Lock poisoned: {}", e))?;
            *cache = Some(handle.clone());
        }

        Ok(handle)
    }

    fn exec(&self, _world: &WorldHandle, req: ExecRequest) -> Result<ExecResult> {
        // Build agent client
        let client = self.build_agent_client()?;

        // Convert request
        let fs_mode = self.effective_fs_mode()?;
        let agent_req = self.convert_exec_request(&req, fs_mode);

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

    fn apply_policy(&self, _world: &WorldHandle, spec: &WorldSpec) -> Result<()> {
        self.store_fs_mode(spec.fs_mode)?;
        // TODO: Implement policy application when agent endpoint is available
        // For now, this is a no-op as mentioned in the plan
        tracing::debug!("Policy application not yet implemented for macOS backend");
        Ok(())
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
            let fs_mode = backend.effective_fs_mode().expect("fs_mode");
            let agent_req = backend.convert_exec_request(&req, fs_mode);
            assert_eq!(
                agent_req.world_fs_mode,
                Some(WorldFsMode::ReadOnly),
                "mac backend should pass through env-derived fs mode"
            );
            assert_eq!(agent_req.shared_world, req.shared_world);
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
    fn ensure_session_owner_mode_sets_authoritative_shared_binding() {
        let _env_guard = crate::test_util::lock_env();
        if let Ok(backend) = MacLimaBackend::new() {
            let spec = WorldSpec {
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

            let handle = backend.ensure_session(&spec).expect("session");
            let binding = handle.shared_binding.expect("shared binding");
            assert_eq!(binding.orchestration_session_id, "orch_123");
            assert_eq!(binding.world_id, handle.id);
            assert_eq!(binding.world_generation, 5);
            assert_eq!(binding.binding_state, SharedWorldBindingState::Active);
        }
    }
}
