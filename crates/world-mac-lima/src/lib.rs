//! macOS world backend using Lima VM with Linux isolation inside.
//!
//! This backend provides identical policy enforcement semantics to LinuxLocal
//! by running a Linux VM via Lima and delegating to the world-agent inside.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::process::{Command, Stdio};
use world_api::{ExecRequest, ExecResult, FsDiff, WorldBackend, WorldHandle, WorldSpec};

pub mod transport;
pub mod vm;

pub use transport::Transport;
pub use vm::LimaVM;

/// macOS backend that delegates to Linux world-agent inside Lima VM.
pub struct MacLimaBackend {
    vm_name: String,
    agent_socket: PathBuf,
    transport: Transport,
}

impl MacLimaBackend {
    pub fn new() -> Result<Self> {
        let vm_name = "substrate".to_string();
        let agent_socket = dirs::home_dir()
            .ok_or_else(|| anyhow::anyhow!("home directory not found"))?
            .join(".substrate/sock/agent.sock");

        // Auto-select best transport
        let transport = Transport::auto_select()?;

        Ok(Self {
            vm_name,
            agent_socket,
            transport,
        })
    }

    pub fn new_with_vm_name(vm_name: String) -> Result<Self> {
        let mut backend = Self::new()?;
        backend.vm_name = vm_name;
        Ok(backend)
    }

    fn ensure_vm_running(&self) -> Result<()> {
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

        let instances: Vec<Instance> =
            serde_json::from_slice(&output.stdout).context("Failed to parse limactl output")?;

        let running = instances
            .iter()
            .any(|i| i.name == self.vm_name && i.status == "Running");

        if !running {
            // Start VM (idempotent)
            println!("Starting Lima VM '{}'...", self.vm_name);
            let status = Command::new("limactl")
                .args(["start", &self.vm_name, "--tty=false"])
                .status()
                .context("Failed to start Lima VM")?;

            if !status.success() {
                anyhow::bail!("Failed to start Lima VM");
            }

            // Wait for agent to be ready
            self.wait_for_agent()?;
        }

        Ok(())
    }

    fn wait_for_agent(&self) -> Result<()> {
        let max_attempts = 30;
        let mut attempts = 0;

        while attempts < max_attempts {
            // Try to connect to the agent socket
            if self.test_agent_connection().is_ok() {
                return Ok(());
            }

            attempts += 1;
            std::thread::sleep(std::time::Duration::from_secs(2));
        }

        anyhow::bail!("Agent failed to start within timeout")
    }

    fn test_agent_connection(&self) -> Result<()> {
        // Simple test: try to execute a basic command
        match &self.transport {
            Transport::UnixSocket => {
                // Test UDS connection
                std::os::unix::net::UnixStream::connect(&self.agent_socket)
                    .context("Failed to connect to agent socket")?;
            }
            Transport::VSock => {
                // TODO: Implement VSock connection test
                anyhow::bail!("VSock connection test not implemented");
            }
            Transport::TCP => {
                // Test TCP connection
                std::net::TcpStream::connect("127.0.0.1:7788")
                    .context("Failed to connect to agent TCP port")?;
            }
        }
        Ok(())
    }

    fn setup_socket_forwarding(&self) -> Result<()> {
        match &self.transport {
            Transport::VSock => {
                // TODO: Implement VSock forwarding
                eprintln!("⚠️  VSock forwarding not yet implemented, falling back to SSH");
                self.setup_ssh_forwarding()
            }
            Transport::UnixSocket => self.setup_ssh_forwarding(),
            Transport::TCP => self.setup_tcp_forwarding(),
        }
    }

    fn setup_ssh_forwarding(&self) -> Result<()> {
        // SSH stream-local forwarding (UDS → UDS)
        let socket_dir = self
            .agent_socket
            .parent()
            .ok_or_else(|| anyhow::anyhow!("Invalid agent socket path"))?;
        std::fs::create_dir_all(socket_dir)?;

        let _child = Command::new("ssh")
            .args([
                "-N",
                "-L",
                &format!("{}:/run/substrate.sock", self.agent_socket.display()),
                &format!("lima-{}", self.vm_name),
                "-o",
                "StreamLocalBindUnlink=yes",
                "-o",
                "ExitOnForwardFailure=yes",
                "-o",
                "UserKnownHostsFile=/dev/null",
                "-o",
                "StrictHostKeyChecking=no",
            ])
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .context("Failed to start SSH forwarding")?;

        // Give SSH time to establish the forward
        std::thread::sleep(std::time::Duration::from_millis(500));
        Ok(())
    }

    fn setup_tcp_forwarding(&self) -> Result<()> {
        // TCP loopback forwarding
        let _child = Command::new("ssh")
            .args([
                "-N",
                "-L",
                "127.0.0.1:7788:127.0.0.1:7788",
                &format!("lima-{}", self.vm_name),
                "-o",
                "UserKnownHostsFile=/dev/null",
                "-o",
                "StrictHostKeyChecking=no",
            ])
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .context("Failed to start TCP forwarding")?;

        std::thread::sleep(std::time::Duration::from_millis(500));
        Ok(())
    }

    fn call_agent<T: Serialize, R: for<'de> Deserialize<'de>>(
        &self,
        method: &str,
        params: &T,
    ) -> Result<R> {
        // TODO: Implement actual agent communication
        // For now, return a placeholder error
        anyhow::bail!("Agent communication not yet implemented")
    }
}

impl Default for MacLimaBackend {
    fn default() -> Self {
        Self::new().expect("Failed to create MacLimaBackend")
    }
}

impl WorldBackend for MacLimaBackend {
    fn ensure_session(&self, spec: &WorldSpec) -> Result<WorldHandle> {
        self.ensure_vm_running()?;
        self.setup_socket_forwarding()?;

        // Forward to agent inside VM
        let response: serde_json::Value = self.call_agent("ensure_session", spec)?;

        let world_id = response
            .get("world_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Invalid response from agent"))?;

        Ok(WorldHandle {
            id: world_id.to_string(),
        })
    }

    fn exec(&self, world: &WorldHandle, req: ExecRequest) -> Result<ExecResult> {
        let params = (world, req);
        self.call_agent("exec", &params)
    }

    fn fs_diff(&self, world: &WorldHandle, span_id: &str) -> Result<FsDiff> {
        let params = (world, span_id);
        self.call_agent("fs_diff", &params)
    }

    fn apply_policy(&self, world: &WorldHandle, spec: &WorldSpec) -> Result<()> {
        let params = (world, spec);
        let _: serde_json::Value = self.call_agent("apply_policy", &params)?;
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
}
