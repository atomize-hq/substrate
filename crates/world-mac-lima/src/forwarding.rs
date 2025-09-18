//! Forwarding management for host-VM communication.

use anyhow::{Context, Result};
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use tracing::{debug, info, warn};

/// Forwarding transport kind.
#[derive(Debug, Clone)]
pub enum ForwardingKind {
    /// VSock proxy forwarding
    Vsock { port: u16 },
    /// SSH Unix Domain Socket forwarding
    SshUds { path: PathBuf },
    /// SSH TCP forwarding
    SshTcp { port: u16 },
}

/// Handle for an active forwarding process.
pub struct ForwardingHandle {
    kind: ForwardingKind,
    child: Option<Child>,
}

impl ForwardingHandle {
    /// Get the forwarding kind.
    pub fn kind(&self) -> &ForwardingKind {
        &self.kind
    }
}

impl Drop for ForwardingHandle {
    fn drop(&mut self) {
        debug!("Dropping ForwardingHandle for {:?}", self.kind);

        // Terminate child process if running
        if let Some(mut child) = self.child.take() {
            match child.kill() {
                Ok(_) => {
                    debug!("Killed forwarding process");
                    let _ = child.wait();
                }
                Err(e) => {
                    warn!("Failed to kill forwarding process: {}", e);
                }
            }
        }

        // Clean up sockets if needed
        if let ForwardingKind::SshUds { ref path } = self.kind {
            if path.exists() {
                if let Err(e) = std::fs::remove_file(path) {
                    warn!("Failed to remove socket file: {}", e);
                }
            }
        }
    }
}

/// Auto-select and establish forwarding.
pub fn auto_select(vm_name: &str) -> Result<ForwardingHandle> {
    eprintln!("DEBUG: Auto-selecting forwarding transport for VM '{}'", vm_name);
    info!("Auto-selecting forwarding transport for VM '{}'", vm_name);

    // Try VSock first
    if vsock_supported() {
        eprintln!("DEBUG: VSock proxy is available, attempting VSock forwarding");
        info!("VSock proxy is available, attempting VSock forwarding");
        match create_vsock_forwarding(vm_name) {
            Ok(handle) => {
                eprintln!("DEBUG: Successfully established VSock forwarding");
                info!("Successfully established VSock forwarding");
                return Ok(handle);
            }
            Err(e) => {
                eprintln!("DEBUG: VSock forwarding failed: {}", e);
                warn!("VSock forwarding failed, falling back: {}", e);
            }
        }
    } else {
        eprintln!("DEBUG: VSock proxy not available");
        info!("VSock proxy not available");
    }

    // Try SSH UDS
    if ssh_available() {
        eprintln!("DEBUG: SSH is available, attempting SSH Unix socket forwarding");
        info!("SSH is available, attempting SSH Unix socket forwarding");
        match create_ssh_uds_forwarding(vm_name) {
            Ok(handle) => {
                eprintln!("DEBUG: Successfully established SSH Unix socket forwarding");
                info!("Successfully established SSH Unix socket forwarding");
                return Ok(handle);
            }
            Err(e) => {
                eprintln!("DEBUG: SSH UDS forwarding failed: {}", e);
                warn!("SSH UDS forwarding failed, falling back: {}", e);
            }
        }

        // SSH TCP fallback requires a TCP <-> UDS bridge inside the guest (e.g., socat).
        // We do not provision that in the base image, and the agent listens on UDS only.
        // To avoid confusing half-open forwards, we intentionally skip TCP fallback here.
        warn!("Skipping SSH TCP fallback: agent uses UDS; enable vsock-proxy or fix SSH UDS");
    } else {
        eprintln!("DEBUG: SSH is not available");
        warn!("SSH is not available");
    }

    anyhow::bail!("No forwarding transport available. Run scripts/mac/lima-doctor.sh")
}

fn vsock_supported() -> bool {
    // Check if vsock-proxy is available
    which::which("vsock-proxy").is_ok()
}

fn ssh_available() -> bool {
    which::which("ssh").is_ok()
}

fn create_vsock_forwarding(vm_name: &str) -> Result<ForwardingHandle> {
    // Find available port
    let port = 17788u16;

    // Start vsock-proxy
    let child = Command::new("vsock-proxy")
        .args(&[
            "--vm", vm_name,
            &port.to_string(),
            "unix:///run/substrate.sock"
        ])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .context("Failed to start vsock-proxy")?;

    // Wait for proxy to be ready
    std::thread::sleep(std::time::Duration::from_millis(500));

    // Health check
    let url = format!("http://127.0.0.1:{}/v1/capabilities", port);
    let response = ureq::get(&url)
        .timeout(std::time::Duration::from_secs(2))
        .call();

    if response.is_err() {
        anyhow::bail!("VSock proxy health check failed");
    }

    Ok(ForwardingHandle {
        kind: ForwardingKind::Vsock { port },
        child: Some(child),
    })
}

fn create_ssh_uds_forwarding(vm_name: &str) -> Result<ForwardingHandle> {
    // Create socket directory
    let socket_dir = dirs::home_dir()
        .ok_or_else(|| anyhow::anyhow!("No home directory"))?
        .join(".substrate/sock");

    debug!("Creating socket directory: {}", socket_dir.display());
    std::fs::create_dir_all(&socket_dir)
        .context("Failed to create socket directory")?;

    // Set permissions to 0700
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = std::fs::metadata(&socket_dir)?.permissions();
        perms.set_mode(0o700);
        std::fs::set_permissions(&socket_dir, perms)?;
    }

    let socket_path = socket_dir.join("agent.sock");

    // Remove old socket if exists
    if socket_path.exists() {
        debug!("Removing old socket file");
        std::fs::remove_file(&socket_path)?;
    }

    let ssh_config = lima_ssh_config_path(vm_name)?;
    debug!("Using SSH config: {}", ssh_config.display());

    let ssh_config_str = ssh_config.to_string_lossy();
    let socket_forward = format!("{}:/run/substrate.sock", socket_path.display());
    let vm_host = format!("lima-{}", vm_name);

    let ssh_args = vec![
        "-F", &ssh_config_str,
        "-o", "ControlMaster=no",
        "-o", "ControlPath=none",
        "-o", "ExitOnForwardFailure=yes",
        "-o", "StreamLocalBindUnlink=yes",
        "-L", &socket_forward,
        &vm_host,
        "-N"
    ];

    debug!("Running SSH command: ssh {:?}", ssh_args);

    // Start SSH forwarding
    let child = Command::new("ssh")
        .args(&ssh_args)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .context("Failed to start SSH UDS forwarding")?;

    debug!("SSH process spawned with PID: {:?}", child.id());

    // Wait for socket to appear (up to ~10s)
    for i in 0..20 {
        if socket_path.exists() {
            info!("SSH UDS forwarding established at {}", socket_path.display());
            return Ok(ForwardingHandle {
                kind: ForwardingKind::SshUds { path: socket_path },
                child: Some(child),
            });
        }
        debug!("Waiting for socket to appear... attempt {}/20", i + 1);
        std::thread::sleep(std::time::Duration::from_millis(500));
    }

    anyhow::bail!("SSH UDS forwarding failed to establish - socket never appeared at {}", socket_path.display())
}

fn create_ssh_tcp_forwarding(vm_name: &str) -> Result<ForwardingHandle> {
    let port = 17788u16;

    let ssh_config = lima_ssh_config_path(vm_name)?;
    let ssh_config_str = ssh_config.to_string_lossy();
    let port_forward = format!("127.0.0.1:{}:/run/substrate.sock", port);
    let vm_host = format!("lima-{}", vm_name);

    // Start SSH TCP forwarding (note: this requires a TCP<->UDS bridge in the guest to be usable)
    let child = Command::new("ssh")
        .args(&[
            "-F", &ssh_config_str,
            "-o", "ControlMaster=no",
            "-o", "ControlPath=none",
            "-o", "ExitOnForwardFailure=yes",
            "-L", &port_forward,
            &vm_host,
            "-N"
        ])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .context("Failed to start SSH TCP forwarding")?;

    // Wait for port to be available
    std::thread::sleep(std::time::Duration::from_millis(1000));

    // Try to connect
    if let Err(e) = std::net::TcpStream::connect(format!("127.0.0.1:{}", port)) {
        anyhow::bail!("SSH TCP forwarding failed to establish: {}", e);
    }

    Ok(ForwardingHandle {
        kind: ForwardingKind::SshTcp { port },
        child: Some(child),
    })
}

fn lima_ssh_config_path(vm_name: &str) -> Result<PathBuf> {
    let path = dirs::home_dir()
        .ok_or_else(|| anyhow::anyhow!("No home directory"))?
        .join(format!(".lima/{}/ssh.config", vm_name));

    if !path.exists() {
        anyhow::bail!("Lima SSH config not found at: {}", path.display());
    }

    Ok(path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vsock_detection() {
        let supported = vsock_supported();
        println!("VSock supported: {}", supported);
    }

    #[test]
    fn test_ssh_detection() {
        let available = ssh_available();
        println!("SSH available: {}", available);
        assert!(available, "SSH should be available on dev machines");
    }

    #[test]
    fn test_forwarding_kind_debug() {
        let kinds = vec![
            ForwardingKind::Vsock { port: 17788 },
            ForwardingKind::SshUds { path: PathBuf::from("/tmp/test.sock") },
            ForwardingKind::SshTcp { port: 17788 },
        ];

        for kind in kinds {
            println!("Forwarding kind: {:?}", kind);
        }
    }
}
