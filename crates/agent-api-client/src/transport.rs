//! Transport layer abstraction for agent communication.

use std::path::PathBuf;

/// Transport options for communicating with world-agent.
#[derive(Debug, Clone)]
pub enum Transport {
    /// Unix domain socket connection.
    UnixSocket { path: PathBuf },
    /// TCP connection.
    Tcp { host: String, port: u16 },
    /// Windows named pipe connection.
    #[cfg(target_os = "windows")]
    NamedPipe { path: PathBuf },
}

impl Transport {
    /// Get a human-readable description of this transport.
    pub fn description(&self) -> String {
        match self {
            Self::UnixSocket { path } => {
                format!("Unix socket: {}", path.display())
            }
            Self::Tcp { host, port } => {
                format!("TCP: {}:{}", host, port)
            }
            #[cfg(target_os = "windows")]
            Self::NamedPipe { path } => {
                format!("Named pipe: {}", path.display())
            }
        }
    }

    /// Check if this transport supports keepalive.
    pub fn supports_keepalive(&self) -> bool {
        match self {
            Self::UnixSocket { .. } => false,
            Self::Tcp { .. } => true,
            #[cfg(target_os = "windows")]
            Self::NamedPipe { .. } => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transport_descriptions() {
        let unix_transport = Transport::UnixSocket {
            path: PathBuf::from("/tmp/test.sock"),
        };
        assert_eq!(unix_transport.description(), "Unix socket: /tmp/test.sock");

        let tcp_transport = Transport::Tcp {
            host: "localhost".to_string(),
            port: 8080,
        };
        assert_eq!(tcp_transport.description(), "TCP: localhost:8080");

        #[cfg(target_os = "windows")]
        {
            let pipe_transport = Transport::NamedPipe {
                path: PathBuf::from(r"\\.\pipe\substrate-agent"),
            };
            assert_eq!(
                pipe_transport.description(),
                "Named pipe: \\.\\pipe\\substrate-agent"
            );
        }
    }

    #[test]
    fn test_keepalive_support() {
        let unix_transport = Transport::UnixSocket {
            path: PathBuf::from("/tmp/test.sock"),
        };
        assert!(!unix_transport.supports_keepalive());

        let tcp_transport = Transport::Tcp {
            host: "localhost".to_string(),
            port: 8080,
        };
        assert!(tcp_transport.supports_keepalive());

        #[cfg(target_os = "windows")]
        {
            let pipe_transport = Transport::NamedPipe {
                path: PathBuf::from(r"\\.\pipe\substrate-agent"),
            };
            assert!(!pipe_transport.supports_keepalive());
        }
    }
}
