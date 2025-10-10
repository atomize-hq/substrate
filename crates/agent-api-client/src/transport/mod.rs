//! Transport layer abstraction for agent communication.

use std::path::PathBuf;

use anyhow::{Context, Result};
use async_trait::async_trait;
use http_body_util::Full;
use hyper::body::Incoming;
use hyper::{body::Bytes, Request, Response, Uri};
use hyper_util::client::legacy::connect::HttpConnector;
use hyper_util::client::legacy::Client;
use hyper_util::rt::TokioExecutor;
#[cfg(target_os = "windows")]
use hyper_util::rt::TokioIo;
#[cfg(unix)]
use hyperlocal::{UnixClientExt, UnixConnector};
#[cfg(target_os = "windows")]
use tokio::net::windows::named_pipe::ClientOptions;

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

/// Telemetry-friendly identifier for the active transport.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransportMode {
    Unix,
    Tcp,
    #[cfg(target_os = "windows")]
    NamedPipe,
}

/// Common interface implemented by each transport connector.
#[async_trait]
pub trait Connector: Send + Sync {
    /// Transport type for telemetry logging.
    fn mode(&self) -> TransportMode;

    /// Human-readable endpoint (path, pipe, or host:port) for diagnostics.
    fn endpoint(&self) -> Option<String>;

    /// Build the URI for the given request path.
    fn build_uri(&self, path: &str) -> Result<Uri>;

    /// Allow the connector to adjust the request prior to dispatch (headers, etc.).
    fn prepare_request(&self, _request: &mut Request<Full<Bytes>>) {}

    /// Execute the HTTP request using the underlying transport.
    async fn execute(&self, request: Request<Full<Bytes>>) -> Result<Response<Incoming>>;
}

#[cfg(unix)]
struct UnixConnectorImpl {
    client: Client<UnixConnector, Full<Bytes>>,
    path: PathBuf,
}

#[cfg(unix)]
#[async_trait]
impl Connector for UnixConnectorImpl {
    fn mode(&self) -> TransportMode {
        TransportMode::Unix
    }

    fn endpoint(&self) -> Option<String> {
        Some(self.path.display().to_string())
    }

    fn build_uri(&self, path: &str) -> Result<Uri> {
        let uri: Uri = hyperlocal::Uri::new(&self.path, path).into();
        Ok(uri)
    }

    async fn execute(&self, request: Request<Full<Bytes>>) -> Result<Response<Incoming>> {
        self.client
            .request(request)
            .await
            .context("Failed to send request over Unix socket")
    }
}

struct TcpConnectorImpl {
    client: Client<HttpConnector, Full<Bytes>>,
    host: String,
    port: u16,
}

#[async_trait]
impl Connector for TcpConnectorImpl {
    fn mode(&self) -> TransportMode {
        TransportMode::Tcp
    }

    fn endpoint(&self) -> Option<String> {
        Some(format!("{}:{}", self.host, self.port))
    }

    fn build_uri(&self, path: &str) -> Result<Uri> {
        let uri = format!("http://{}:{}{}", self.host, self.port, path)
            .parse()
            .context("Failed to build TCP URI")?;
        Ok(uri)
    }

    fn prepare_request(&self, request: &mut Request<Full<Bytes>>) {
        let host_header = format!("{}:{}", self.host, self.port);
        request
            .headers_mut()
            .entry(hyper::header::HOST)
            .or_insert_with(|| hyper::header::HeaderValue::from_str(&host_header).unwrap());
    }

    async fn execute(&self, request: Request<Full<Bytes>>) -> Result<Response<Incoming>> {
        self.client
            .request(request)
            .await
            .context("Failed to send request over TCP")
    }
}

#[cfg(target_os = "windows")]
struct NamedPipeConnectorImpl {
    path: PathBuf,
}

#[cfg(target_os = "windows")]
#[async_trait]
impl Connector for NamedPipeConnectorImpl {
    fn mode(&self) -> TransportMode {
        TransportMode::NamedPipe
    }

    fn endpoint(&self) -> Option<String> {
        Some(self.path.display().to_string())
    }

    fn build_uri(&self, path: &str) -> Result<Uri> {
        let uri = format!("http://localhost{}", path)
            .parse()
            .context("Failed to build named pipe URI")?;
        Ok(uri)
    }

    fn prepare_request(&self, request: &mut Request<Full<Bytes>>) {
        request
            .headers_mut()
            .entry(hyper::header::HOST)
            .or_insert_with(|| hyper::header::HeaderValue::from_static("localhost"));
        // Use explicit close semantics over named pipes to avoid lingering connections
        request
            .headers_mut()
            .entry(hyper::header::CONNECTION)
            .or_insert_with(|| hyper::header::HeaderValue::from_static("close"));
    }

    async fn execute(&self, request: Request<Full<Bytes>>) -> Result<Response<Incoming>> {
        let pipe = ClientOptions::new()
            .open(&self.path)
            .with_context(|| format!("Failed to open named pipe {}", self.path.display()))?;
        let io = TokioIo::new(pipe);
        let (mut sender, connection) = hyper::client::conn::http1::Builder::new()
            .handshake(io)
            .await
            .context("Failed to perform HTTP handshake over named pipe")?;

        tokio::spawn(async move {
            if let Err(err) = connection.await {
                tracing::debug!(error = %err, "Named pipe connection closed with error");
            }
        });

        sender
            .send_request(request)
            .await
            .context("Failed to send request over named pipe")
    }
}

/// Build a connector implementation for the requested transport.
pub fn build_connector(transport: &Transport) -> Result<Box<dyn Connector>> {
    match transport {
        #[cfg(unix)]
        Transport::UnixSocket { path } => {
            let client = Client::unix();
            Ok(Box::new(UnixConnectorImpl {
                client,
                path: path.clone(),
            }))
        }
        #[cfg(not(unix))]
        Transport::UnixSocket { .. } => Err(anyhow::anyhow!(
            "Unix socket transport is not available on this platform"
        )),
        Transport::Tcp { host, port } => {
            let http_connector = HttpConnector::new();
            let client = Client::builder(TokioExecutor::new()).build(http_connector);
            Ok(Box::new(TcpConnectorImpl {
                client,
                host: host.clone(),
                port: *port,
            }))
        }
        #[cfg(target_os = "windows")]
        Transport::NamedPipe { path } => {
            Ok(Box::new(NamedPipeConnectorImpl { path: path.clone() }))
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
            let expected = format!(
                "Named pipe: {}",
                PathBuf::from(r"\\.\pipe\substrate-agent").display()
            );
            assert_eq!(pipe_transport.description(), expected);
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
