//! World environment helpers for routing.
#![cfg(any(target_os = "macos", target_os = "windows"))]

use super::super::pw;
use substrate_trace::TransportMeta;

pub(crate) fn world_transport_to_meta(transport: &pw::WorldTransport) -> TransportMeta {
    match transport {
        pw::WorldTransport::Unix(path) => TransportMeta {
            mode: "unix".to_string(),
            endpoint: Some(path.display().to_string()),
        },
        pw::WorldTransport::Tcp { host, port } => TransportMeta {
            mode: "tcp".to_string(),
            endpoint: Some(format!("{}:{}", host, port)),
        },
        pw::WorldTransport::Vsock { port } => TransportMeta {
            mode: "vsock".to_string(),
            endpoint: Some(format!("{}", port)),
        },
        #[cfg(target_os = "windows")]
        pw::WorldTransport::NamedPipe(path) => TransportMeta {
            mode: "named_pipe".to_string(),
            endpoint: Some(path.display().to_string()),
        },
    }
}
