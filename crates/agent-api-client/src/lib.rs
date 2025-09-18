//! Agent API client for forwarding requests to world-agent.
//!
//! This crate provides the client implementation used by host-proxy to forward
//! Agent API requests to world-agent running inside worlds/VMs.

use agent_api_types::{ApiError, ExecuteRequest, ExecuteResponse};
use anyhow::{Context, Result};
use http_body_util::{BodyExt, Full};
use hyper::{body::Bytes, Method, Request, Response, Uri};
use hyper_util::client::legacy::connect::HttpConnector;
use hyper_util::client::legacy::Client;
use hyperlocal::{UnixClientExt, UnixConnector};
use serde::{Deserialize, Serialize};
use std::path::Path;

pub mod retry;
pub mod transport;

pub use transport::Transport;

/// Internal client type for handling different transports.
enum ClientKind {
    Unix(Client<UnixConnector, Full<Bytes>>),
    Tcp(Client<HttpConnector, Full<Bytes>>),
}

/// Client for communicating with world-agent.
pub struct AgentClient {
    transport: Transport,
    client: ClientKind,
}

impl AgentClient {
    /// Create a new client with the given transport.
    pub fn new(transport: Transport) -> Self {
        let client = match &transport {
            Transport::UnixSocket { .. } => ClientKind::Unix(Client::unix()),
            Transport::Tcp { .. } => {
                let http_connector = HttpConnector::new();
                ClientKind::Tcp(
                    Client::builder(hyper_util::rt::TokioExecutor::new()).build(http_connector),
                )
            }
        };
        Self { transport, client }
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

    /// Execute a command via the agent API.
    pub async fn execute(&self, request: ExecuteRequest) -> Result<ExecuteResponse> {
        let response = self
            .post("/v1/execute", &request)
            .await
            .context("Failed to execute command")?;

        self.parse_response(response).await
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
        let uri = self.build_uri(path)?;
        let request = Request::builder()
            .method(Method::GET)
            .uri(uri)
            .body(Full::new(Bytes::new()))
            .context("Failed to build GET request")?;

        match &self.client {
            ClientKind::Unix(client) => client
                .request(request)
                .await
                .context("Failed to send GET request"),
            ClientKind::Tcp(client) => client
                .request(request)
                .await
                .context("Failed to send GET request"),
        }
    }

    /// Make a POST request with JSON body.
    async fn post<T: Serialize>(
        &self,
        path: &str,
        body: &T,
    ) -> Result<Response<hyper::body::Incoming>> {
        let uri = self.build_uri(path)?;
        let json_body = serde_json::to_vec(body).context("Failed to serialize request body")?;

        let request = Request::builder()
            .method(Method::POST)
            .uri(uri)
            .header("content-type", "application/json")
            .body(Full::new(Bytes::from(json_body)))
            .context("Failed to build POST request")?;

        match &self.client {
            ClientKind::Unix(client) => client
                .request(request)
                .await
                .context("Failed to send POST request"),
            ClientKind::Tcp(client) => client
                .request(request)
                .await
                .context("Failed to send POST request"),
        }
    }

    /// Build URI for the given path based on transport.
    fn build_uri(&self, path: &str) -> Result<Uri> {
        match &self.transport {
            Transport::UnixSocket { path: socket_path } => {
                let uri: Uri = hyperlocal::Uri::new(socket_path, path).into();
                Ok(uri)
            }
            Transport::Tcp { host, port } => format!("http://{}:{}{}", host, port, path)
                .parse()
                .context("Failed to build TCP URI"),
        }
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
            // Try to parse as API error first
            if let Ok(api_error) = serde_json::from_slice::<ApiError>(&body_bytes) {
                return Err(anyhow::anyhow!("API error: {}", api_error));
            }

            // Fallback to raw error message
            let error_text = String::from_utf8_lossy(&body_bytes);
            return Err(anyhow::anyhow!("HTTP {} error: {}", status, error_text));
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

    #[test]
    fn test_client_creation() {
        let client = AgentClient::unix_socket("/tmp/test.sock");

        match client.transport {
            Transport::UnixSocket { ref path } => {
                assert_eq!(path, std::path::Path::new("/tmp/test.sock"));
            }
            _ => panic!("Expected Unix socket transport"),
        }
    }

    #[test]
    fn test_tcp_client_creation() {
        let client = AgentClient::tcp("localhost", 8080);

        match client.transport {
            Transport::Tcp { ref host, port } => {
                assert_eq!(host, "localhost");
                assert_eq!(port, 8080);
            }
            _ => panic!("Expected TCP transport"),
        }
    }

    #[test]
    fn test_default_client() {
        let client = AgentClient::default();

        match client.transport {
            Transport::UnixSocket { ref path } => {
                assert!(path
                    .to_string_lossy()
                    .contains(".substrate/sock/agent.sock"));
            }
            _ => panic!("Expected Unix socket transport for default"),
        }
    }

    #[tokio::test]
    async fn test_uri_building() {
        let client = AgentClient::unix_socket("/tmp/test.sock");
        let uri = client.build_uri("/v1/execute").unwrap();

        // hyperlocal URIs have a specific format
        assert!(uri.to_string().contains("/v1/execute"));

        let tcp_client = AgentClient::tcp("localhost", 8080);
        let tcp_uri = tcp_client.build_uri("/v1/execute").unwrap();
        assert_eq!(tcp_uri.to_string(), "http://localhost:8080/v1/execute");
    }
}
