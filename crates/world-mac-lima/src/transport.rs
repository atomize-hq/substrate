//! Transport layer selection for host-VM communication.

use anyhow::{anyhow, Result};
use std::path::PathBuf;
use transport_api_types::{
    InstallBootstrapContextCarrierV1, PlatformBootstrapMappingV1, PlatformInstanceIdentityV1,
    PlatformTransportIdentityV1,
};

/// Canonical guest world-service socket path inside the Lima VM.
pub const CANONICAL_GUEST_SOCKET_PATH: &str = "/run/substrate.sock";
/// URI form used by host-side helpers that forward into the guest socket.
pub const CANONICAL_GUEST_SOCKET_URI: &str = "unix:///run/substrate.sock";
/// Managed host-side socket directory under SUBSTRATE_HOME for SSH UDS forwarding.
pub const MANAGED_HOST_SOCKET_DIR: &str = "sock";
/// Managed host-side socket filename for SSH UDS forwarding.
pub const MANAGED_HOST_SOCKET_NAME: &str = "agent.sock";
/// Host loopback used for retained compatibility TCP reachability.
pub const COMPATIBILITY_TCP_HOST: &str = "127.0.0.1";
/// Host loopback port used for retained compatibility TCP reachability.
pub const COMPATIBILITY_TCP_PORT: u16 = 17788;

/// Managed host-side socket path for the default Lima-backed Unix socket path.
pub fn managed_host_socket_path() -> PathBuf {
    managed_host_socket_path_from(
        substrate_common::paths::substrate_home().unwrap_or_else(|_| {
            dirs::home_dir()
                .unwrap_or_else(|| PathBuf::from("."))
                .join(substrate_common::paths::SUBSTRATE_DIR_NAME)
        }),
    )
}

fn managed_host_socket_path_from(substrate_home: PathBuf) -> PathBuf {
    substrate_home
        .join(MANAGED_HOST_SOCKET_DIR)
        .join(MANAGED_HOST_SOCKET_NAME)
}

/// Managed host-side socket path derived from a validated Lima mapping.
pub fn managed_host_socket_path_for_mapping(
    host_carrier: &InstallBootstrapContextCarrierV1,
    mapping: &PlatformBootstrapMappingV1,
) -> Result<PathBuf> {
    host_carrier.validate().map_err(anyhow::Error::from)?;
    mapping
        .validate(host_carrier)
        .map_err(anyhow::Error::from)?;

    let PlatformInstanceIdentityV1::Lima { .. } = &mapping.platform_instance else {
        return Err(anyhow!("platform mapping is not Lima"));
    };
    let PlatformTransportIdentityV1::Lima {
        host_socket,
        guest_socket,
    } = &mapping.realized_transport
    else {
        return Err(anyhow!("platform transport is not Lima"));
    };

    if guest_socket != CANONICAL_GUEST_SOCKET_PATH {
        return Err(anyhow!(
            "platform mapping guest socket does not match the canonical Lima socket"
        ));
    }

    let expected_host_socket =
        managed_host_socket_path_from(PathBuf::from(&host_carrier.context.selected_host_prefix));
    let expected_host_socket = expected_host_socket
        .to_str()
        .ok_or_else(|| anyhow!("managed host socket path is not valid UTF-8"))?;
    if host_socket != expected_host_socket {
        return Err(anyhow!(
            "platform mapping host socket does not match the selected host prefix"
        ));
    }

    Ok(PathBuf::from(host_socket))
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
    use transport_api_types::{
        InstallBootstrapContextCarrierV1, InstallBootstrapContextV1, PlatformBootstrapMappingV1,
        PlatformInstanceIdentityV1, PlatformTransportIdentityV1,
    };

    fn test_host_carrier() -> InstallBootstrapContextCarrierV1 {
        InstallBootstrapContextCarrierV1::from_context(
            InstallBootstrapContextV1::new_unix("/opt/substrate", "alice", 1000).unwrap(),
        )
        .unwrap()
    }

    fn test_lima_mapping() -> PlatformBootstrapMappingV1 {
        PlatformBootstrapMappingV1::new_lima(
            &test_host_carrier(),
            "substrate",
            "0123456789abcdef0123456789abcdef",
            "/Users/alice/.lima",
            "/home/substrate/.substrate",
            "substrate",
            1000,
            "/opt/substrate/sock/agent.sock",
            CANONICAL_GUEST_SOCKET_PATH,
        )
        .unwrap()
    }

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
        let substrate_home = PathBuf::from("/tmp/substrate-home");
        assert_eq!(
            managed_host_socket_path_from(substrate_home),
            PathBuf::from("/tmp/substrate-home/sock/agent.sock")
        );
    }

    #[test]
    fn test_managed_host_socket_path_for_mapping_uses_validated_identity() {
        let host = test_host_carrier();
        let mapping = test_lima_mapping();

        assert_eq!(
            managed_host_socket_path_for_mapping(&host, &mapping).unwrap(),
            PathBuf::from("/opt/substrate/sock/agent.sock")
        );
    }

    #[test]
    fn test_managed_host_socket_path_for_mapping_rejects_mismatches() {
        let host = test_host_carrier();

        let mut wrong_transport_kind = test_lima_mapping();
        wrong_transport_kind.realized_transport = PlatformTransportIdentityV1::Wsl {
            pipe_path: r"\\.\pipe\substrate-agent".to_string(),
            guest_socket: CANONICAL_GUEST_SOCKET_PATH.to_string(),
        };
        assert!(managed_host_socket_path_for_mapping(&host, &wrong_transport_kind).is_err());

        let mut wrong_platform_kind = test_lima_mapping();
        wrong_platform_kind.platform_instance = PlatformInstanceIdentityV1::Wsl {
            distro_name: "Substrate-WSL".to_string(),
            guest_machine_id: "0123456789abcdef0123456789abcdef".to_string(),
        };
        assert!(managed_host_socket_path_for_mapping(&host, &wrong_platform_kind).is_err());

        let mut wrong_host_socket = test_lima_mapping();
        wrong_host_socket.realized_transport = PlatformTransportIdentityV1::Lima {
            host_socket: "/opt/other/sock/agent.sock".to_string(),
            guest_socket: CANONICAL_GUEST_SOCKET_PATH.to_string(),
        };
        assert!(managed_host_socket_path_for_mapping(&host, &wrong_host_socket).is_err());

        let mut wrong_guest_socket = test_lima_mapping();
        wrong_guest_socket.realized_transport = PlatformTransportIdentityV1::Lima {
            host_socket: "/opt/substrate/sock/agent.sock".to_string(),
            guest_socket: "/tmp/agent.sock".to_string(),
        };
        assert!(managed_host_socket_path_for_mapping(&host, &wrong_guest_socket).is_err());

        let mut wrong_commitment = test_lima_mapping();
        wrong_commitment.host_context_commitment = "0".repeat(64);
        assert!(managed_host_socket_path_for_mapping(&host, &wrong_commitment).is_err());
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
