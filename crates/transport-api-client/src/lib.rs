//! Agent API client for forwarding requests to world-service.
//!
//! This crate provides the client implementation used by host-proxy to forward
//! Agent API requests to world-service running inside worlds/VMs.

use std::path::Path;
use std::sync::Arc;

use anyhow::{anyhow, Context, Result};
use http_body_util::{BodyExt, Full};
use hyper::{body::Bytes, Method, Request, Response, StatusCode};
use serde::{Deserialize, Serialize};
use transport_api_types::{
    ApiError, ExecuteCancelRequestV1, ExecuteCancelResponseV1, ExecuteRequest, ExecuteResponse,
    ExecuteStreamReplayRequestV1, GatewayLifecycleRequestV1, GatewayLifecycleResponseV1,
    MemberTurnSubmitRequestV1, PendingDiffClearRequestV1, PendingDiffClearResponseV1,
    PendingDiffReconcileRequestV1, PendingDiffReconcileResponseV1, PendingDiffRecordV1,
    PendingDiffRequestV1, WorldDoctorReportV1, WorldFsReadRequestV1, WorldFsReadResponseV1,
};

pub mod retry;
pub mod transport;

pub use transport::{build_connector, Connector, Transport, TransportMode};

/// Client for communicating with world-service.
pub struct AgentClient {
    transport: Transport,
    connector: Arc<dyn Connector>,
}

impl AgentClient {
    /// Create a new client with the given transport.
    pub fn new(transport: Transport) -> Result<Self> {
        let connector = build_connector(&transport)
            .with_context(|| format!("Unsupported transport: {}", transport.description()))?;
        Ok(Self {
            transport,
            connector: Arc::from(connector),
        })
    }

    /// Create a client that connects to the default Unix socket.
    pub fn unix_socket<P: AsRef<Path>>(socket_path: P) -> Result<Self> {
        let transport = Transport::UnixSocket {
            path: socket_path.as_ref().to_path_buf(),
        };
        Self::new(transport)
    }

    /// Create a client that connects via TCP.
    pub fn tcp(host: &str, port: u16) -> Result<Self> {
        let transport = Transport::Tcp {
            host: host.to_string(),
            port,
        };
        Self::new(transport)
    }

    #[cfg(target_os = "windows")]
    /// Create a client that connects via a Windows named pipe.
    pub fn named_pipe<P: AsRef<Path>>(pipe_path: P) -> Result<Self> {
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

    /// Replay one accepted execution from the exact durable host cursor and continue live frames.
    pub async fn replay_execute_stream(
        &self,
        request: ExecuteStreamReplayRequestV1,
    ) -> Result<Response<hyper::body::Incoming>> {
        request.validate().map_err(anyhow::Error::msg)?;
        let response = self
            .post("/v1/execute/stream/replay", &request)
            .await
            .context("Failed to initiate streaming execute replay")?;
        if response.status().is_success() {
            return Ok(response);
        }
        let status = response.status();
        let body_bytes = response
            .into_body()
            .collect()
            .await
            .context("Failed to read replay error body")?
            .to_bytes();
        Err(Self::map_http_error(status, &body_bytes))
    }

    /// Send a signal to a live streamed execute request.
    pub async fn cancel_execute(
        &self,
        request: ExecuteCancelRequestV1,
    ) -> Result<ExecuteCancelResponseV1> {
        let response = self
            .post("/v1/execute/cancel", &request)
            .await
            .context("Failed to request streamed execute cancellation")?;

        self.parse_response(response).await
    }

    /// Submit a follow-up turn to a retained world member session and stream incremental output.
    pub async fn submit_member_turn_stream(
        &self,
        request: MemberTurnSubmitRequestV1,
    ) -> Result<Response<hyper::body::Incoming>> {
        let response = self
            .post("/v1/member_turn/stream", &request)
            .await
            .context("Failed to initiate member turn submit stream")?;

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

    /// Retrieve the current session's pending diff record (`POST /v1/pending_diff`).
    pub async fn pending_diff(&self, request: PendingDiffRequestV1) -> Result<PendingDiffRecordV1> {
        let response = self
            .post("/v1/pending_diff", &request)
            .await
            .context("Failed to request pending diff")?;

        self.parse_response(response).await
    }

    /// Conditionally clear the current session's pending diff snapshot (`POST /v1/pending_diff/clear`).
    pub async fn pending_diff_clear(
        &self,
        request: PendingDiffClearRequestV1,
    ) -> Result<PendingDiffClearResponseV1> {
        let response = self
            .post("/v1/pending_diff/clear", &request)
            .await
            .context("Failed to clear pending diff")?;

        self.parse_response(response).await
    }

    /// Reconcile pending diff paths (host-prefer policy) by discarding overlay upper entries (`POST /v1/pending_diff/reconcile`).
    pub async fn pending_diff_reconcile(
        &self,
        request: PendingDiffReconcileRequestV1,
    ) -> Result<PendingDiffReconcileResponseV1> {
        let response = self
            .post("/v1/pending_diff/reconcile", &request)
            .await
            .context("Failed to reconcile pending diff")?;

        self.parse_response(response).await
    }

    /// Read metadata and (optionally) contents from the current session's overlay (`POST /v1/world_fs/read`).
    pub async fn world_fs_read(
        &self,
        request: WorldFsReadRequestV1,
    ) -> Result<WorldFsReadResponseV1> {
        let response = self
            .post("/v1/world_fs/read", &request)
            .await
            .context("Failed to read world fs path")?;

        self.parse_response(response).await
    }

    /// Get the agent-reported world doctor report (`GET /v1/doctor/world`).
    pub async fn doctor_world(&self) -> Result<WorldDoctorReportV1> {
        let response = self
            .get("/v1/doctor/world")
            .await
            .context("Failed to get world doctor report")?;

        self.parse_response(response).await
    }

    /// Get the typed gateway lifecycle/status surface (`POST /v1/gateway/status`).
    pub async fn gateway_status(
        &self,
        request: GatewayLifecycleRequestV1,
    ) -> Result<GatewayLifecycleResponseV1> {
        let response = self
            .post("/v1/gateway/status", &request)
            .await
            .context("Failed to get gateway status")?;

        self.parse_response(response).await
    }

    /// Ensure the gateway lifecycle path is wired (`POST /v1/gateway/sync`).
    pub async fn gateway_sync(
        &self,
        request: GatewayLifecycleRequestV1,
    ) -> Result<GatewayLifecycleResponseV1> {
        let response = self
            .post("/v1/gateway/sync", &request)
            .await
            .context("Failed to sync gateway lifecycle")?;

        self.parse_response(response).await
    }

    /// Restart the gateway lifecycle path (`POST /v1/gateway/restart`).
    pub async fn gateway_restart(
        &self,
        request: GatewayLifecycleRequestV1,
    ) -> Result<GatewayLifecycleResponseV1> {
        let response = self
            .post("/v1/gateway/restart", &request)
            .await
            .context("Failed to restart gateway lifecycle")?;

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

    /// Construct a client using the default Unix socket location when available.
    pub fn try_default() -> Result<Self> {
        let socket_path = dirs::home_dir()
            .unwrap_or_else(|| std::path::PathBuf::from("."))
            .join(".substrate/sock/agent.sock");

        Self::unix_socket(socket_path)
    }

    #[cfg(all(test, unix))]
    fn build_uri_for_test(&self, path: &str) -> Result<hyper::Uri> {
        self.connector.build_uri(path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn e3a_passthrough_request() -> transport_api_types::ExecuteRequest {
        use transport_api_types::{
            AuthorityObjectKindV1, AuthorityObjectRefV1, ConfigProjectionActivationCarrierV1,
            ConfigProjectionRefV1, DispatchPolicyCommitmentRefCarrierV1,
            E2LaunchRequestCommitmentV1, E2MemberLaunchActivationCarrierV1, E2MemberLaunchKindV1,
            InWorldGatewayRefV1, ManagedGatewayActivationIntentRefV1, MemberDispatchRequest,
            MemberDispatchRequestV2, MemberRuntimeBackendKindV1, OpaqueAuthorityCommitmentV1,
            PolicySnapshotV3, PolicySnapshotWorldFsDimensionV3, PolicySnapshotWorldFsFailClosedV3,
            PolicySnapshotWorldFsV3, PolicySnapshotWorldFsWriteV3,
            ResolvedMemberRuntimeDescriptorV1, WorldBindingRefV1,
        };
        let authority_store_id = "cpa_018f0f2e-7b4c-7aa1-8c22-123456789ab0".to_string();
        let series_id = "cps_018f0f2e-7b4c-7aa1-8c22-123456789ab1".to_string();
        let policy_snapshot = PolicySnapshotV3 {
            schema_version: 3,
            net_allowed: Vec::new(),
            world_fs: PolicySnapshotWorldFsV3 {
                host_visible: true,
                fail_closed: PolicySnapshotWorldFsFailClosedV3 { routing: false },
                deny_enforcement: None,
                caged_required: false,
                discover: Some(PolicySnapshotWorldFsDimensionV3 {
                    allow_list: vec![".".to_string()],
                    deny_list: Vec::new(),
                }),
                read: Some(PolicySnapshotWorldFsDimensionV3 {
                    allow_list: vec![".".to_string()],
                    deny_list: Vec::new(),
                }),
                write: PolicySnapshotWorldFsWriteV3 {
                    enabled: true,
                    allow_list: vec![".".to_string()],
                    deny_list: Vec::new(),
                },
            },
        };
        let immutable_worker_cap_ref = DispatchPolicyCommitmentRefCarrierV1 {
            authority_store_id: "authority-store-e3a".to_string(),
            commitment_id: "dpc_018f0f2e-7b4c-7aa1-8c22-123456789ab7".to_string(),
            exact_linkage_hash: "a".repeat(64),
        };
        transport_api_types::ExecuteRequest {
            profile: None,
            cmd: String::new(),
            cwd: Some("/workspace".to_string()),
            env: None,
            pty: false,
            agent_id: "e3a-client-passthrough".to_string(),
            budget: None,
            policy_snapshot,
            shared_world: None,
            world_network: None,
            world_fs_mode: None,
            member_dispatch: Some(MemberDispatchRequest::V2(MemberDispatchRequestV2 {
                schema_version: 2,
                orchestration_session_id: "orch_e3a".to_string(),
                participant_id: "member_e3a".to_string(),
                orchestrator_participant_id: "orchestrator_e3a".to_string(),
                parent_participant_id: Some("source_e3a".to_string()),
                resumed_from_participant_id: None,
                backend_id: "cli:codex".to_string(),
                protocol: "substrate.agent.session".to_string(),
                run_id: "run_e3a".to_string(),
                world_id: "world_e3a".to_string(),
                world_generation: 1,
                initial_prompt: Some("strict V2 passthrough".to_string()),
                resolved_runtime: ResolvedMemberRuntimeDescriptorV1 {
                    backend_kind: MemberRuntimeBackendKindV1::Codex,
                    binary_path: "/bin/true".to_string(),
                },
                retained_worker_launch_authority: None,
                e2_launch_activation: Some(E2MemberLaunchActivationCarrierV1 {
                    schema_version: 1,
                    activation_id: format!("e2a_{}", "a".repeat(32)),
                    launch_kind: E2MemberLaunchKindV1::Fork,
                    reservation_ref: None,
                    commitment_ref: immutable_worker_cap_ref.clone(),
                    immutable_worker_cap_ref,
                    immutable_worker_cap_created_revision: 1,
                    immutable_worker_cap_application_revision: 2,
                    policy_snapshot_bytes_base64: "eyJzY2hlbWFfdmVyc2lvbiI6MywibmV0X2FsbG93ZWQiOltdLCJ3b3JsZF9mcyI6eyJob3N0X3Zpc2libGUiOnRydWUsImZhaWxfY2xvc2VkIjp7InJvdXRpbmciOmZhbHNlfSwiY2FnZWRfcmVxdWlyZWQiOmZhbHNlLCJkaXNjb3ZlciI6eyJhbGxvd19saXN0IjpbIi4iXSwiZGVueV9saXN0IjpbXX0sInJlYWQiOnsiYWxsb3dfbGlzdCI6WyIuIl0sImRlbnlfbGlzdCI6W119LCJ3cml0ZSI6eyJlbmFibGVkIjp0cnVlLCJhbGxvd19saXN0IjpbIi4iXSwiZGVueV9saXN0IjpbXX19fQ==".to_string(),
                    policy_snapshot_byte_length: 274,
                    policy_snapshot_ref: AuthorityObjectRefV1 {
                        ref_id: "ao_0123456789abcdef0123456789abcdef".to_string(),
                        object_kind: AuthorityObjectKindV1::Policy,
                        schema_version: 1,
                        commitment: OpaqueAuthorityCommitmentV1::CanonicalSha256 {
                            digest_hex: "b".repeat(64),
                        },
                    },
                    policy_snapshot_hash:
                        "59e3189cffd6da318ab00888b8cc8fd89d069422f72edf2d4e4a201270d76b2c"
                            .to_string(),
                    policy_snapshot_revision: "policy-revision-e3a".to_string(),
                    reason: Some("strict E3-A client passthrough".to_string()),
                    request_id: "request-e3a".to_string(),
                    idempotency_key: "idempotency-e3a".to_string(),
                    orchestration_session_id: "orch_e3a".to_string(),
                    caller_participant_id: "orchestrator_e3a".to_string(),
                    caller_backend_id: "cli:codex".to_string(),
                    target_backend_id: "cli:codex".to_string(),
                    retained_participant_id: "member_e3a".to_string(),
                    bootstrap_run_id: "run_e3a".to_string(),
                    source_participant_id: Some("source_e3a".to_string()),
                    target_world: WorldBindingRefV1 {
                        world_id: "world_e3a".to_string(),
                        world_generation: 1,
                    },
                    parent_policy_ref: AuthorityObjectRefV1 {
                        ref_id: "ao_fedcba9876543210fedcba9876543210".to_string(),
                        object_kind: AuthorityObjectKindV1::Policy,
                        schema_version: 1,
                        commitment: OpaqueAuthorityCommitmentV1::CanonicalSha256 {
                            digest_hex: "c".repeat(64),
                        },
                    },
                    parent_policy_revision: "parent-policy-revision-e3a".to_string(),
                    request_commitment: E2LaunchRequestCommitmentV1::CanonicalSha256 {
                        domain: "substrate.e3a.test".to_string(),
                        digest_hex: "d".repeat(64),
                    },
                    registry_publication_revision: 2,
                }),
                config_projection: ConfigProjectionActivationCarrierV1 {
                    authority_store_id: authority_store_id.clone(),
                    series_id: series_id.clone(),
                    dormant_projection_ref: ConfigProjectionRefV1 {
                        authority_store_id: authority_store_id.clone(),
                        series_id,
                        record_id: "cpr_018f0f2e-7b4c-7aa1-8c22-123456789ab2".to_string(),
                        revision: 1,
                        record_hash: "1".repeat(64),
                    },
                    activation_intent_ref: ManagedGatewayActivationIntentRefV1 {
                        authority_store_id: authority_store_id.clone(),
                        activation_intent_id: "gai_018f0f2e-7b4c-7aa1-8c22-123456789ab3"
                            .to_string(),
                        intent_hash: "2".repeat(64),
                    },
                    expected_gateway_ref: InWorldGatewayRefV1 {
                        authority_store_id,
                        gateway_instance_id: "cgi_018f0f2e-7b4c-7aa1-8c22-123456789ab4".to_string(),
                        gateway_identity_hash: "3".repeat(64),
                    },
                    fence_id: "cpf_018f0f2e-7b4c-7aa1-8c22-123456789ab5".to_string(),
                    consumer_id: "cpc_018f0f2e-7b4c-7aa1-8c22-123456789ab6".to_string(),
                    consumer_lease_revision: 1,
                    consumer_lease_hash: "4".repeat(64),
                },
            })),
            acceptance_context: None,
        }
    }

    #[tokio::test]
    async fn e3a_execute_stream_preserves_v2_body_without_client_downgrade() {
        use tokio::io::{AsyncReadExt as _, AsyncWriteExt as _};

        let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
            .await
            .expect("bind client passthrough listener");
        let port = listener.local_addr().expect("listener address").port();
        let server = tokio::spawn(async move {
            let (mut stream, _) = listener.accept().await.expect("accept client request");
            let mut bytes = Vec::new();
            let header_end = loop {
                let mut chunk = [0u8; 4096];
                let count = stream.read(&mut chunk).await.expect("read request bytes");
                assert!(count > 0, "client closed before HTTP headers");
                bytes.extend_from_slice(&chunk[..count]);
                if let Some(index) = bytes.windows(4).position(|window| window == b"\r\n\r\n") {
                    break index + 4;
                }
            };
            let headers = std::str::from_utf8(&bytes[..header_end]).expect("UTF-8 headers");
            assert!(headers.starts_with("POST /v1/execute/stream HTTP/1.1\r\n"));
            let content_length = headers
                .lines()
                .find_map(|line| {
                    line.to_ascii_lowercase()
                        .strip_prefix("content-length: ")
                        .and_then(|value| value.parse::<usize>().ok())
                })
                .expect("content length");
            while bytes.len() - header_end < content_length {
                let mut chunk = [0u8; 4096];
                let count = stream.read(&mut chunk).await.expect("read request body");
                assert!(count > 0, "client closed before HTTP body");
                bytes.extend_from_slice(&chunk[..count]);
            }
            let body = serde_json::from_slice::<serde_json::Value>(
                &bytes[header_end..header_end + content_length],
            )
            .expect("decode captured request");
            stream
                .write_all(
                    b"HTTP/1.1 400 Bad Request\r\ncontent-type: application/json\r\ncontent-length: 49\r\nconnection: close\r\n\r\n{\"error\":\"bad_request\",\"message\":\"expected stop\"}",
                )
                .await
                .expect("write response");
            body
        });

        let request = e3a_passthrough_request();
        request.validate().expect("well-formed strict V2 request");
        let expected = serde_json::to_value(&request).expect("shape expected V2 request");
        let client = AgentClient::tcp("127.0.0.1", port).expect("TCP client");
        client
            .execute_stream(request)
            .await
            .expect_err("test server rejects after capturing V2");
        let captured = server.await.expect("passthrough server task");
        assert_eq!(captured, expected);
        assert_eq!(captured["member_dispatch"]["schema_version"], 2);
        assert!(captured["member_dispatch"]["config_projection"].is_object());
    }

    #[cfg(unix)]
    #[test]
    fn test_client_creation() {
        let client = AgentClient::unix_socket("/tmp/test.sock").unwrap();

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
        let client = AgentClient::tcp("localhost", 8080).unwrap();

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
        let client = AgentClient::named_pipe(r"\\.\pipe\substrate-agent").unwrap();

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
        let client = AgentClient::try_default().unwrap();

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
        let client = AgentClient::unix_socket("/tmp/test.sock").unwrap();
        let uri = client.build_uri_for_test("/v1/execute").unwrap();

        // hyperlocal URIs have a specific format
        assert!(uri.to_string().contains("/v1/execute"));

        let tcp_client = AgentClient::tcp("localhost", 8080).unwrap();
        let tcp_uri = tcp_client.build_uri_for_test("/v1/execute").unwrap();
        assert_eq!(tcp_uri.to_string(), "http://localhost:8080/v1/execute");
    }

    #[cfg(unix)]
    #[tokio::test]
    async fn replay_client_rejects_inexact_identity_before_transport() {
        let client = AgentClient::unix_socket("/tmp/replay-validation.sock").unwrap();
        let error = client
            .replay_execute_stream(transport_api_types::ExecuteStreamReplayRequestV1 {
                schema_version: 1,
                acceptance_record_id: "session-only".to_string(),
                stream_id: "rts_exact".to_string(),
                after_frame_sequence: 0,
            })
            .await
            .expect_err("inexact replay identity must fail before transport");
        assert!(error.to_string().contains("acceptance_record_id"));
    }
}
