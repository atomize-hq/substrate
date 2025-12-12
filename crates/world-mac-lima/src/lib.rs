#![cfg(target_os = "macos")]
//! macOS world backend using Lima VM with Linux isolation inside.
//!
//! This backend provides identical policy enforcement semantics to LinuxLocal
//! by running a Linux VM via Lima and delegating to the world-agent inside.

use agent_api_client::AgentClient;
use agent_api_types::{ExecuteRequest, ExecuteResponse, WorldFsMode};
use anyhow::{Context, Result};
use serde::Deserialize;
use std::path::PathBuf;
use std::process::Command;
use substrate_common::FsDiff;
use tokio::runtime::Runtime;
use world_api::{ExecRequest, ExecResult, WorldBackend, WorldHandle, WorldSpec};

pub mod forwarding;
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
        let vm_name = "substrate".to_string();
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
        let output = Command::new("limactl")
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
            let status = Command::new("limactl")
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
        let output = Command::new("limactl")
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
            eprintln!("DEBUG: Setting up forwarding for VM '{}'", self.vm_name);
            tracing::info!("Setting up forwarding for VM '{}'", self.vm_name);
            let handle = forwarding::auto_select(&self.vm_name)?;
            eprintln!("DEBUG: Forwarding established: {:?}", handle.kind());
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
        ExecuteRequest {
            profile: None,
            cmd: req.cmd.clone(),
            cwd: Some(req.cwd.to_string_lossy().to_string()),
            env: Some(req.env.clone()),
            pty: req.pty,
            agent_id: "world-mac-lima".to_string(),
            budget: None,
            world_fs_mode: Some(fs_mode),
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

        let handle = WorldHandle { id: world_id };

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_backend_creation() {
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
        let prev = std::env::var("SUBSTRATE_WORLD_FS_MODE").ok();
        std::env::set_var("SUBSTRATE_WORLD_FS_MODE", "read_only");

        if let Ok(backend) = MacLimaBackend::new() {
            let req = ExecRequest {
                cmd: "echo hi".to_string(),
                cwd: PathBuf::from("/tmp"),
                env: std::collections::HashMap::new(),
                pty: false,
                span_id: None,
            };
            let fs_mode = backend.effective_fs_mode().expect("fs_mode");
            let agent_req = backend.convert_exec_request(&req, fs_mode);
            assert_eq!(
                agent_req.world_fs_mode,
                Some(WorldFsMode::ReadOnly),
                "mac backend should pass through env-derived fs mode"
            );
        }

        match prev {
            Some(value) => std::env::set_var("SUBSTRATE_WORLD_FS_MODE", value),
            None => std::env::remove_var("SUBSTRATE_WORLD_FS_MODE"),
        }
    }
}
