//! Transport layer selection for host-VM communication.

use anyhow::Result;

/// Transport options for host ↔ VM communication.
#[derive(Debug, Clone)]
pub enum Transport {
    /// VSock (fastest, macOS 13+ with Virtualization.framework).
    VSock,
    /// SSH forwarded Unix socket (most compatible).
    UnixSocket,
    /// TCP loopback (fallback).
    TCP,
}

impl Transport {
    /// Auto-select the best available transport.
    pub fn auto_select() -> Result<Self> {
        // Try VSock first (macOS 13+)
        if Self::vsock_available() {
            return Ok(Self::VSock);
        }

        // Check if SSH is available for UDS forwarding
        if Self::ssh_available() {
            return Ok(Self::UnixSocket);
        }

        // Fall back to TCP
        Ok(Self::TCP)
    }

    fn vsock_available() -> bool {
        // Check for vsock-proxy command
        which::which("vsock-proxy").is_ok()
    }

    fn ssh_available() -> bool {
        // Check if SSH command is available
        which::which("ssh").is_ok()
    }

    /// Get the connection endpoint for this transport.
    pub fn endpoint(&self) -> String {
        match self {
            Self::VSock => "vsock://2:1024".to_string(), // Guest CID 2, port 1024
            Self::UnixSocket => {
                let home = dirs::home_dir().unwrap_or_else(|| std::path::PathBuf::from("."));
                home.join(".substrate/sock/agent.sock")
                    .display()
                    .to_string()
            }
            Self::TCP => "127.0.0.1:7788".to_string(),
        }
    }

    /// Get a human-readable description.
    pub fn description(&self) -> &'static str {
        match self {
            Self::VSock => "VSock (fastest)",
            Self::UnixSocket => "SSH forwarded Unix socket",
            Self::TCP => "TCP loopback (fallback)",
        }
    }
}

impl Default for Transport {
    fn default() -> Self {
        Self::auto_select().unwrap_or(Self::TCP)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transport_auto_select() {
        let transport = Transport::auto_select();
        assert!(transport.is_ok());
    }

    #[test]
    fn test_transport_endpoints() {
        let transports = [Transport::VSock, Transport::UnixSocket, Transport::TCP];

        for transport in &transports {
            let endpoint = transport.endpoint();
            assert!(!endpoint.is_empty());
            println!("{}: {}", transport.description(), endpoint);
        }
    }

    #[test]
    fn test_ssh_detection() {
        let available = Transport::ssh_available();
        println!("SSH available: {}", available);
        // Don't assert since SSH may not be installed in CI
    }
}
