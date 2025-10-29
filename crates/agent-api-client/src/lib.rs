//! Agent API client for forwarding requests to world-agent.
//!
//! This crate provides the client implementation used by host-proxy to forward
//! Agent API requests to world-agent running inside worlds/VMs.

use std::path::Path;
use std::sync::Arc;

use agent_api_types::{ApiError, ExecuteRequest, ExecuteResponse};
use anyhow::{anyhow, Context, Result};
use http_body_util::{BodyExt, Full};
use hyper::{body::Bytes, Method, Request, Response, StatusCode};
use serde::{Deserialize, Serialize};

pub mod retry;
pub mod transport;

pub use transport::{build_connector, Connector, Transport, TransportMode};

/// Client for communicating with world-agent.
pub struct AgentClient {
    transport: Transport,
    connector: Arc<dyn Connector>,
}

impl AgentClient {
    /// Create a new client with the given transport.
    pub fn new(transport: Transport) -> Self {
        let connector = build_connector(&transport)
            .unwrap_or_else(|err| panic!("Unsupported transport: {err}"));
        Self {
            transport,
            connector: Arc::from(connector),
        }
    }

    /// Create a client that connects to the default Unix socket.
    pub fn unix_socket<P: AsRef<Path>>(socket_path: P) -> Self {
        let transport = Transport::UnixSocket {
            path: socket_path.as_ref().to_path_buf(),
        };
        Self::new(transport)
    }

    /// Create a client that connects via TCP.
    pub fn tcp(host: &str, port: u16) -> Self {
        let transport = Transport::Tcp {
            host: host.to_string(),
            port,
        };
        Self::new(transport)
    }

    #[cfg(target_os = "windows")]
    /// Create a client that connects via a Windows named pipe.
    pub fn named_pipe<P: AsRef<Path>>(pipe_path: P) -> Self {
        let transport = Transport::NamedPipe {
            path: pipe_path.as_ref().to_path_buf(),
        };
        Self::new(transport)
    }

    /// Return the transport metadata used by this client.
    pub fn transport_mode(&self) -> TransportMode {
        match &self.transport {
            Transport::UnixSocket { .. } => TransportMode::Unix,
            Transport::Tcp { .. } => TransportMode::Tcp,
            #[cfg(target_os = "windows")]
            Transport::NamedPipe { .. } => TransportMode::NamedPipe,
        }
    }

    /// Optional human-readable endpoint for telemetry.
    pub fn transport_endpoint(&self) -> Option<String> {
        self.connector.endpoint()
    }

    /// Access the underlying transport configuration.
    pub fn transport(&self) -> &Transport {
        &self.transport
    }

    fn map_http_error(status: StatusCode, body_bytes: &[u8]) -> anyhow::Error {
        if let Ok(api_error) = serde_json::from_slice::<ApiError>(body_bytes) {
            anyhow!("API error: {}", api_error)
        } else {
            let error_text = String::from_utf8_lossy(body_bytes);
            anyhow!("HTTP {} error: {}", status, error_text)
        }
    }

    /// Execute a command via the agent API.
    pub async fn execute(&self, request: ExecuteRequest) -> Result<ExecuteResponse> {
        let response = self
            .post("/v1/execute", &request)
            .await
            .context("Failed to execute command")?;

        self.parse_response(response).await
    }

    /// Execute a command and stream incremental output frames.
    pub async fn execute_stream(
        &self,
        request: ExecuteRequest,
    ) -> Result<Response<hyper::body::Incoming>> {
        let response = self
            .post("/v1/execute/stream", &request)
            .await
            .context("Failed to initiate streaming execute")?;

        if response.status().is_success() {
            return Ok(response);
        }

        let status = response.status();
        let body_bytes = response
            .into_body()
            .collect()
            .await
            .context("Failed to read error body")?
            .to_bytes();

        Err(Self::map_http_error(status, &body_bytes))
    }

    /// Get agent capabilities.
    pub async fn capabilities(&self) -> Result<serde_json::Value> {
        let response = self
            .get("/v1/capabilities")
            .await
            .context("Failed to get capabilities")?;

        self.parse_response(response).await
    }

    /// Get trace information for a span.
    pub async fn get_trace(&self, span_id: &str) -> Result<serde_json::Value> {
        let path = format!("/v1/trace/{}", span_id);
        let response = self.get(&path).await.context("Failed to get trace")?;

        self.parse_response(response).await
    }

    /// Request additional scopes.
    pub async fn request_scopes(&self, scopes: Vec<String>) -> Result<serde_json::Value> {
        let response = self
            .post("/v1/request_scopes", &scopes)
            .await
            .context("Failed to request scopes")?;

        self.parse_response(response).await
    }

    /// Make a GET request.
    async fn get(&self, path: &str) -> Result<Response<hyper::body::Incoming>> {
        let uri = self
            .connector
            .build_uri(path)
            .context("Failed to build GET URI")?;
        let mut request = Request::builder()
            .method(Method::GET)
            .uri(uri)
            .body(Full::new(Bytes::new()))
            .context("Failed to build GET request")?;

        self.connector.prepare_request(&mut request);

        self.connector
            .execute(request)
            .await
            .context("Failed to send GET request")
    }

    /// Make a POST request with JSON body.
    async fn post<T: Serialize>(
        &self,
        path: &str,
        body: &T,
    ) -> Result<Response<hyper::body::Incoming>> {
        let uri = self
            .connector
            .build_uri(path)
            .context("Failed to build POST URI")?;
        let json_body = serde_json::to_vec(body).context("Failed to serialize request body")?;

        let mut request = Request::builder()
            .method(Method::POST)
            .uri(uri)
            .header("content-type", "application/json")
            .body(Full::new(Bytes::from(json_body)))
            .context("Failed to build POST request")?;

        self.connector.prepare_request(&mut request);

        self.connector
            .execute(request)
            .await
            .context("Failed to send POST request")
    }

    /// Parse response body as JSON.
    async fn parse_response<T: for<'de> Deserialize<'de>>(
        &self,
        response: Response<hyper::body::Incoming>,
    ) -> Result<T> {
        let status = response.status();
        let body_bytes = response
            .into_body()
            .collect()
            .await
            .context("Failed to read response body")?
            .to_bytes();

        if !status.is_success() {
            return Err(Self::map_http_error(status, &body_bytes));
        }

        serde_json::from_slice(&body_bytes).context("Failed to parse JSON response")
    }

    /// Check if the agent is reachable.
    pub async fn health_check(&self) -> Result<bool> {
        match self.capabilities().await {
            Ok(_) => Ok(true),
            Err(_) => Ok(false),
        }
    }

    #[cfg(all(test, unix))]
    fn build_uri_for_test(&self, path: &str) -> Result<hyper::Uri> {
        self.connector.build_uri(path)
    }
}

impl Default for AgentClient {
    fn default() -> Self {
        // Default to Unix socket at standard location
        let socket_path = dirs::home_dir()
            .unwrap_or_else(|| std::path::PathBuf::from("."))
            .join(".substrate/sock/agent.sock");

        Self::unix_socket(socket_path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(unix)]
    #[test]
    fn test_client_creation() {
        let client = AgentClient::unix_socket("/tmp/test.sock");

        match client.transport() {
            Transport::UnixSocket { ref path } => {
                assert_eq!(path, std::path::Path::new("/tmp/test.sock"));
                assert_eq!(client.transport_mode(), TransportMode::Unix);
            }
            _ => panic!("Expected Unix socket transport"),
        }
    }

    #[test]
    fn test_tcp_client_creation() {
        let client = AgentClient::tcp("localhost", 8080);

        match client.transport() {
            Transport::Tcp { ref host, port } => {
                assert_eq!(host, "localhost");
                assert_eq!(*port, 8080);
                assert_eq!(client.transport_mode(), TransportMode::Tcp);
            }
            _ => panic!("Expected TCP transport"),
        }
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn test_named_pipe_client_creation() {
        let client = AgentClient::named_pipe(r"\\.\pipe\substrate-agent");

        match client.transport() {
            Transport::NamedPipe { ref path } => {
                assert_eq!(path, std::path::Path::new(r"\\.\pipe\substrate-agent"));
                assert_eq!(client.transport_mode(), TransportMode::NamedPipe);
            }
            _ => panic!("Expected NamedPipe transport"),
        }
    }

    #[cfg(unix)]
    #[test]
    fn test_default_client() {
        let client = AgentClient::default();

        match client.transport() {
            Transport::UnixSocket { ref path } => {
                assert!(path
                    .to_string_lossy()
                    .contains(".substrate/sock/agent.sock"));
            }
            _ => panic!("Expected Unix socket transport for default"),
        }
    }

    #[cfg(unix)]
    #[tokio::test]
    async fn test_uri_building() {
        let client = AgentClient::unix_socket("/tmp/test.sock");
        let uri = client.build_uri_for_test("/v1/execute").unwrap();

        // hyperlocal URIs have a specific format
        assert!(uri.to_string().contains("/v1/execute"));

        let tcp_client = AgentClient::tcp("localhost", 8080);
        let tcp_uri = tcp_client.build_uri_for_test("/v1/execute").unwrap();
        assert_eq!(tcp_uri.to_string(), "http://localhost:8080/v1/execute");
    }
}
