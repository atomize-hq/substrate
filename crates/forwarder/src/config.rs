use std::net::SocketAddr;

#[derive(Clone, Debug)]
pub struct ForwarderConfig {
    pub distro: String,
    pub pipe_path: String,
    pub tcp_bridge: Option<SocketAddr>,
}

impl ForwarderConfig {
    pub fn tcp_enabled(&self) -> bool {
        self.tcp_bridge.is_some()
    }
}
