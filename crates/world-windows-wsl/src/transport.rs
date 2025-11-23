use anyhow::{Context, Result};
use std::net::SocketAddr;

pub const DEFAULT_DISTRO: &str = "substrate-wsl";
pub const DEFAULT_AGENT_PIPE: &str = r"\\.\pipe\substrate-agent";
pub const DEFAULT_TCP_ADDR: &str = "127.0.0.1";
pub const DEFAULT_TCP_PORT: u16 = 17788;

pub fn detect_tcp_forwarder() -> Result<Option<(String, u16)>> {
    if let Ok(addr) = std::env::var("SUBSTRATE_FORWARDER_TCP_ADDR") {
        let socket: SocketAddr = addr
            .parse()
            .context("invalid SUBSTRATE_FORWARDER_TCP_ADDR")?;
        return Ok(Some((socket.ip().to_string(), socket.port())));
    }

    let tcp_enabled = std::env::var("SUBSTRATE_FORWARDER_TCP")
        .map(|v| matches!(v.trim().to_ascii_lowercase().as_str(), "1" | "true" | "yes"))
        .unwrap_or(false);

    if !tcp_enabled {
        return Ok(None);
    }

    let host = std::env::var("SUBSTRATE_FORWARDER_TCP_HOST")
        .unwrap_or_else(|_| DEFAULT_TCP_ADDR.to_string());
    let port = std::env::var("SUBSTRATE_FORWARDER_TCP_PORT")
        .ok()
        .and_then(|p| p.parse::<u16>().ok())
        .unwrap_or(DEFAULT_TCP_PORT);
    Ok(Some((host, port)))
}
