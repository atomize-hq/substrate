//! Transport layer selection for host-VM communication.

use anyhow::Result;
use std::path::PathBuf;

/// Canonical guest world-service socket path inside the Lima VM.
pub const CANONICAL_GUEST_SOCKET_PATH: &str = "/run/substrate.sock";
/// URI form used by host-side helpers that forward into the guest socket.
pub const CANONICAL_GUEST_SOCKET_URI: &str = "unix:///run/substrate.sock";
/// Managed host-side socket directory for SSH UDS forwarding.
pub const MANAGED_HOST_SOCKET_DIR: &str = ".substrate/sock";
/// Managed host-side socket filename for SSH UDS forwarding.
pub const MANAGED_HOST_SOCKET_NAME: &str = "agent.sock";
/// Host loopback used for retained compatibility TCP reachability.
pub const COMPATIBILITY_TCP_HOST: &str = "127.0.0.1";
/// Host loopback port used for retained compatibility TCP reachability.
pub const COMPATIBILITY_TCP_PORT: u16 = 17788;

/// Managed host-side socket path for the default Lima-backed Unix socket path.
pub fn managed_host_socket_path() -> PathBuf {
    managed_host_socket_path_from(dirs::home_dir())
}

fn managed_host_socket_path_from(home_dir: Option<PathBuf>) -> PathBuf {
    home_dir
        .unwrap_or_else(|| PathBuf::from("."))
        .join(MANAGED_HOST_SOCKET_DIR)
        .join(MANAGED_HOST_SOCKET_NAME)
}

/// Host loopback endpoint retained for compatibility-only TCP reachability.
pub fn compatibility_tcp_endpoint() -> String {
    format!("{COMPATIBILITY_TCP_HOST}:{COMPATIBILITY_TCP_PORT}")
}

/// Transport options for host ↔ VM communication.
#[derive(Debug, Clone)]
pub enum Transport {
    /// VSock proxying into the canonical guest UDS (fastest on supported VZ guests).
    VSock,
    /// SSH forwarded Unix socket to the canonical guest UDS (most compatible).
    UnixSocket,
    /// TCP loopback compatibility path to the canonical guest UDS.
    TCP,
}

impl Transport {
    /// Auto-select the best available transport.
    pub fn auto_select() -> Result<Self> {
        // Try VSock first (macOS 13+ VZ guest when host tooling is available)
        if Self::vsock_available() {
            return Ok(Self::VSock);
        }

        // Check if SSH is available for UDS forwarding
        if Self::ssh_available() {
            return Ok(Self::UnixSocket);
        }

        // Fall back to the retained compatibility TCP endpoint
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

    /// Get the host-visible connection endpoint for this transport.
    pub fn endpoint(&self) -> String {
        match self {
            Self::VSock | Self::TCP => compatibility_tcp_endpoint(),
            Self::UnixSocket => managed_host_socket_path().display().to_string(),
        }
    }

    /// Get a human-readable description.
    pub fn description(&self) -> &'static str {
        match self {
            Self::VSock => "VSock proxy to guest UDS",
            Self::UnixSocket => "SSH forwarded Unix socket to guest UDS",
            Self::TCP => "TCP loopback compatibility path to guest UDS",
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

        assert_eq!(Transport::VSock.endpoint(), compatibility_tcp_endpoint());
        assert_eq!(Transport::TCP.endpoint(), compatibility_tcp_endpoint());
        assert_eq!(
            Transport::UnixSocket.endpoint(),
            managed_host_socket_path().display().to_string()
        );
    }

    #[test]
    fn test_managed_host_socket_path_layout() {
        let home = PathBuf::from("/tmp/substrate-home");
        assert_eq!(
            managed_host_socket_path_from(Some(home)),
            PathBuf::from("/tmp/substrate-home/.substrate/sock/agent.sock")
        );
    }

    #[test]
    fn test_canonical_transport_constants() {
        assert_eq!(CANONICAL_GUEST_SOCKET_PATH, "/run/substrate.sock");
        assert_eq!(CANONICAL_GUEST_SOCKET_URI, "unix:///run/substrate.sock");
        assert_eq!(COMPATIBILITY_TCP_HOST, "127.0.0.1");
        assert_eq!(COMPATIBILITY_TCP_PORT, 17788);
    }

    #[test]
    fn test_ssh_detection() {
        let available = Transport::ssh_available();
        println!("SSH available: {}", available);
        // Don't assert since SSH may not be installed in CI
    }
}
