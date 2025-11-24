use std::path::PathBuf;

use anyhow::{anyhow, bail, Result};
use serde::{Deserialize, Serialize};

pub(crate) const DEFAULT_AGENT_TCP_PORT: u16 = 17788;
#[cfg(target_os = "windows")]
pub(crate) const DEFAULT_AGENT_PIPE: &str = r"\\.\pipe\substrate-agent";

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "mode", rename_all = "snake_case")]
pub enum AgentTransportConfig {
    Unix {
        path: PathBuf,
    },
    Tcp {
        host: String,
        port: u16,
    },
    #[cfg(target_os = "windows")]
    NamedPipe {
        path: PathBuf,
    },
}

impl Default for AgentTransportConfig {
    fn default() -> Self {
        #[cfg(target_os = "windows")]
        {
            AgentTransportConfig::NamedPipe {
                path: PathBuf::from(r"\.\pipe\substrate-agent"),
            }
        }

        #[cfg(not(target_os = "windows"))]
        {
            AgentTransportConfig::Unix {
                path: PathBuf::from("/run/substrate.sock"),
            }
        }
    }
}

impl AgentTransportConfig {
    pub fn from_uri(value: &str) -> Result<Self> {
        let trimmed = value.trim();
        if trimmed.is_empty() {
            bail!("transport value is empty");
        }

        if let Some(idx) = trimmed.find("://") {
            let scheme = trimmed[..idx].to_ascii_lowercase();
            let rest = &trimmed[idx + 3..];
            return match scheme.as_str() {
                "unix" | "uds" => Self::parse_unix(rest),
                "tcp" => Self::parse_tcp(rest),
                "named-pipe" | "named_pipe" => Self::parse_named_pipe(rest),
                other => Err(anyhow!("unsupported transport scheme: {}", other)),
            };
        }

        match trimmed.to_ascii_lowercase().as_str() {
            "unix" | "uds" => {
                bail!("unix transport string requires a socket path (e.g. unix:///tmp/agent.sock)")
            }
            "tcp" => Ok(AgentTransportConfig::Tcp {
                host: "127.0.0.1".to_string(),
                port: DEFAULT_AGENT_TCP_PORT,
            }),
            "named-pipe" | "named_pipe" => Self::parse_named_pipe("."),
            other => Err(anyhow!("unsupported transport specifier: {}", other)),
        }
    }

    fn parse_unix(rest: &str) -> Result<Self> {
        let path = rest.trim();
        if path.is_empty() {
            bail!("unix transport requires a socket path");
        }
        Ok(AgentTransportConfig::Unix {
            path: PathBuf::from(path),
        })
    }

    fn parse_tcp(rest: &str) -> Result<Self> {
        let target = rest.trim();
        if target.is_empty() {
            return Ok(AgentTransportConfig::Tcp {
                host: "127.0.0.1".to_string(),
                port: DEFAULT_AGENT_TCP_PORT,
            });
        }

        if target.starts_with('[') {
            let end = target
                .find(']')
                .ok_or_else(|| anyhow!("invalid IPv6 tcp host: {}", target))?;
            let host = &target[1..end];
            let mut port = DEFAULT_AGENT_TCP_PORT;
            if let Some(port_str) = target[end + 1..].strip_prefix(':') {
                if !port_str.is_empty() {
                    port = port_str
                        .parse::<u16>()
                        .map_err(|err| anyhow!("invalid tcp port '{}': {}", port_str, err))?;
                }
            }
            return Ok(AgentTransportConfig::Tcp {
                host: host.to_string(),
                port,
            });
        }

        if let Some(idx) = target.rfind(':') {
            let host_part = &target[..idx];
            let port_part = &target[idx + 1..];
            let host = if host_part.is_empty() {
                "127.0.0.1"
            } else {
                host_part
            };
            let port = if port_part.is_empty() {
                DEFAULT_AGENT_TCP_PORT
            } else {
                port_part
                    .parse::<u16>()
                    .map_err(|err| anyhow!("invalid tcp port '{}': {}", port_part, err))?
            };
            return Ok(AgentTransportConfig::Tcp {
                host: host.to_string(),
                port,
            });
        }

        Ok(AgentTransportConfig::Tcp {
            host: target.to_string(),
            port: DEFAULT_AGENT_TCP_PORT,
        })
    }

    #[cfg(target_os = "windows")]
    fn parse_named_pipe(rest: &str) -> Result<Self> {
        let path = Self::normalize_named_pipe_segment(rest)?;
        Ok(AgentTransportConfig::NamedPipe { path })
    }

    #[cfg(target_os = "windows")]
    fn normalize_named_pipe_segment(segment: &str) -> Result<PathBuf> {
        let part = segment.trim();
        if part.is_empty() || part == "." {
            return Ok(PathBuf::from(DEFAULT_AGENT_PIPE));
        }

        if part.starts_with(r"\\") {
            return Ok(PathBuf::from(part));
        }

        let trimmed = part.trim_start_matches("./").trim_start_matches('/');
        if trimmed.is_empty() {
            return Ok(PathBuf::from(DEFAULT_AGENT_PIPE));
        }

        let replaced = trimmed.replace('/', "\\");
        Ok(PathBuf::from(format!("\\\\.\\pipe\\{}", replaced)))
    }

    #[cfg(not(target_os = "windows"))]
    fn parse_named_pipe(_rest: &str) -> Result<Self> {
        bail!("named pipe transport is only supported on Windows");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[cfg(target_os = "windows")]
    use crate::config::ProxyConfig;

    #[test]
    fn parse_tcp_with_port() {
        let config = AgentTransportConfig::from_uri("tcp://localhost:9000").unwrap();
        match config {
            AgentTransportConfig::Tcp { host, port } => {
                assert_eq!(host, "localhost");
                assert_eq!(port, 9000);
            }
            _ => panic!("expected tcp transport"),
        }
    }

    #[test]
    fn parse_tcp_default_port() {
        let config = AgentTransportConfig::from_uri("tcp://localhost").unwrap();
        match config {
            AgentTransportConfig::Tcp { host, port } => {
                assert_eq!(host, "localhost");
                assert_eq!(port, DEFAULT_AGENT_TCP_PORT);
            }
            _ => panic!("expected tcp transport"),
        }
    }

    #[test]
    fn parse_unix_transport() {
        let config = AgentTransportConfig::from_uri("unix:///tmp/agent.sock").unwrap();
        match config {
            AgentTransportConfig::Unix { path } => {
                assert_eq!(path, PathBuf::from("/tmp/agent.sock"));
            }
            _ => panic!("expected unix transport"),
        }
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn parse_named_pipe_default() {
        let config = AgentTransportConfig::from_uri("named-pipe://.").unwrap();
        match config {
            AgentTransportConfig::NamedPipe { path } => {
                assert_eq!(
                    path.to_string_lossy().trim_start_matches('\\'),
                    DEFAULT_AGENT_PIPE.trim_start_matches('\\')
                );
            }
            _ => panic!("expected named pipe transport"),
        }
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn parse_named_pipe_custom_segment() {
        let config = AgentTransportConfig::from_uri("named-pipe://./custom-agent").unwrap();
        match config {
            AgentTransportConfig::NamedPipe { path } => {
                assert_eq!(path, PathBuf::from(r"\\.\pipe\custom-agent"));
            }
            _ => panic!("expected named pipe transport"),
        }
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn proxy_default_serializes_named_pipe() {
        let config = ProxyConfig::default();
        let json = serde_json::to_value(&config).expect("serialize config");
        assert_eq!(json["agent"]["mode"], "named_pipe");

        let path = json["agent"]["path"]
            .as_str()
            .expect("agent path should be string");
        assert_eq!(
            path.trim_start_matches('\\'),
            DEFAULT_AGENT_PIPE.trim_start_matches('\\')
        );
        assert!(path.contains("substrate-agent"));
    }
}
