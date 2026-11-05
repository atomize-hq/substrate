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

// Each Host operation owns its own HTTP client and opens its own Unix stream.
// The generic transport connector remains pooled for all existing APIs.
#[cfg(target_os = "linux")]
#[derive(Clone)]
struct HostGatewayTurnConnectorV1 {
    socket: std::path::PathBuf,
}

#[cfg(target_os = "linux")]
impl tower_service::Service<hyper::Uri> for HostGatewayTurnConnectorV1 {
    type Response = hyper_util::rt::TokioIo<tokio::net::UnixStream>;
    type Error = std::io::Error;
    type Future = std::pin::Pin<
        Box<dyn std::future::Future<Output = std::io::Result<Self::Response>> + Send>,
    >;

    fn poll_ready(
        &mut self,
        _context: &mut std::task::Context<'_>,
    ) -> std::task::Poll<std::io::Result<()>> {
        std::task::Poll::Ready(Ok(()))
    }

    fn call(&mut self, _uri: hyper::Uri) -> Self::Future {
        let socket = self.socket.clone();
        Box::pin(async move {
            tokio::net::UnixStream::connect(socket)
                .await
                .map(hyper_util::rt::TokioIo::new)
        })
    }
}

async fn host_gateway_turn_once(connector: &dyn Connector, bytes: Vec<u8>) -> Result<Bytes> {
    #[cfg(target_os = "linux")]
    {
        // AgentClient constructors bind transport metadata and this private
        // connector to the same socket. Only tests can inject a fixture pair.
        if connector.mode() != TransportMode::Unix {
            return Err(anyhow!("host_gateway_turn_unsupported_configuration"));
        }
        let socket = connector
            .endpoint()
            .ok_or_else(|| anyhow!("host_gateway_turn_unsupported_configuration"))?;
        let mut builder =
            hyper_util::client::legacy::Client::builder(hyper_util::rt::TokioExecutor::new());
        builder
            .pool_max_idle_per_host(0)
            .retry_canceled_requests(false);
        let client = builder.build(HostGatewayTurnConnectorV1 {
            socket: socket.into(),
        });
        let http_request = Request::builder()
            .method(Method::POST)
            .uri("http://localhost/v1/gateway/host-turn")
            .header("content-type", "application/json")
            .body(Full::new(Bytes::from(bytes)))
            .map_err(|_| anyhow!("host_gateway_turn_invalid_request"))?;
        let response = client
            .request(http_request)
            .await
            .map_err(|_| anyhow!("host_gateway_turn_transport_failed"))?;
        if response.status() == StatusCode::NOT_FOUND {
            return Err(anyhow!("host_gateway_turn_unsupported_configuration"));
        }
        if !response.status().is_success() {
            return Err(anyhow!("host_gateway_turn_rejected"));
        }
        // Keep this dedicated client alive until the bounded reply is drained.
        http_body_util::Limited::new(response.into_body(), 64 * 1024)
            .collect()
            .await
            .map(|body| body.to_bytes())
            .map_err(|_| anyhow!("host_gateway_turn_invalid_response"))
    }
    #[cfg(not(target_os = "linux"))]
    {
        let _ = (connector, bytes);
        Err(anyhow!("host_gateway_turn_unsupported_configuration"))
    }
}

impl AgentClient {
    /// Observe one exact existing destination over the authenticated native UDS.
    /// No execution or later-use authority is returned.
    ///
    /// ```no_run
    /// # async fn observe(client:&transport_api_client::AgentClient, request:&transport_api_types::WorkspaceCurrentRequestV1)->anyhow::Result<()> {
    /// let observation=client.workspace_preflight_v1(request).await?;
    /// # let _=observation; Ok(()) }
    /// ```
    #[cfg(target_os = "linux")]
    pub async fn workspace_preflight_v1(
        &self,
        request: &transport_api_types::WorkspaceCurrentRequestV1,
    ) -> Result<transport_api_types::WorkspacePreflightResponseV1> {
        request.validate().map_err(anyhow::Error::msg)?;
        anyhow::ensure!(
            matches!(
                request.operation,
                transport_api_types::WorkspaceCurrentOperationV1::CurrentActorRead { .. }
            ),
            "Denied"
        );
        let Transport::UnixSocket { path } = &self.transport else {
            anyhow::bail!("Unsupported");
        };
        let bytes = transport::workspace_owner_on_authenticated_uds_v1(
            path,
            serde_json::to_vec(request)?,
            "/v1/workspace/preflight",
        )
        .await?;
        let response: transport_api_types::WorkspacePreflightResponseV1 =
            serde_json::from_slice(&bytes)?;
        if let transport_api_types::WorkspacePreflightResponseV1::EligibleObservation(value) =
            &response
        {
            let transport_api_types::WorkspaceCurrentOperationV1::CurrentActorRead {
                lane_ref,
                association_incarnation,
                association_revision,
                expected_actor,
                ..
            } = &request.operation
            else {
                anyhow::bail!("WrongBinding");
            };
            anyhow::ensure!(
                value.workspace_ref == request.resource.workspace_ref
                    && value.workspace_incarnation == request.resource.workspace_incarnation
                    && &value.lane_ref == lane_ref
                    && &value.actor_id == expected_actor
                    && value.world_id == request.resource.world_id
                    && value.world_generation == request.resource.world_generation
                    && value.backend_id == request.resource.backend_id
                    && &value.revisions.association_incarnation == association_incarnation
                    && &value.revisions.association_revision == association_revision
                    && value.revisions.authority_store_id == request.observation.authority_store_id
                    && value.revisions.root_revision == request.observation.root_revision
                    && value.revisions.authority_revision == request.observation.authority_revision,
                "WrongBinding"
            );
        }
        Ok(response)
    }

    /// Publish only the exact source-owner association selected by an authorized request.
    ///
    /// ```no_run
    /// # async fn publish(client:&transport_api_client::AgentClient, update:&transport_api_types::WorkspaceAssociationUpdateV1)->anyhow::Result<()> {
    /// let result=client.workspace_association_update_v1(update).await?;
    /// # let _=result; Ok(()) }
    /// ```
    #[cfg(target_os = "linux")]
    pub async fn workspace_association_update_v1(
        &self,
        request: &transport_api_types::WorkspaceAssociationUpdateV1,
    ) -> Result<transport_api_types::WorkspaceAssociationUpdateResponseV1> {
        request.current.validate().map_err(anyhow::Error::msg)?;
        anyhow::ensure!(
            matches!(
                request.current.operation,
                transport_api_types::WorkspaceCurrentOperationV1::PublishAssociation
            ),
            "Denied"
        );
        let Transport::UnixSocket { path } = &self.transport else {
            anyhow::bail!("Unsupported");
        };
        let bytes = transport::workspace_owner_on_authenticated_uds_v1(
            path,
            serde_json::to_vec(request)?,
            "/v1/workspace/association",
        )
        .await?;
        Ok(serde_json::from_slice(&bytes)?)
    }
    /// Prepare one E3 gateway attempt on the authenticated inherited Linux socket.
    pub async fn e3_config_projection_prepare(
        &self,
        request: transport_api_types::E3ConfigProjectionPrepareRequestV1,
    ) -> Result<transport_api_types::E3ConfigProjectionPrepareResponseV1> {
        if !matches!(&self.transport, Transport::UnixSocket { path } if path == Path::new("/run/substrate.sock"))
        {
            return Err(anyhow!("UnsupportedConfiguration"));
        }
        request
            .validate()
            .map_err(|_| anyhow!("Malformed E3 preparation"))?;
        let bytes =
            serde_json::to_vec(&request).map_err(|_| anyhow!("Malformed E3 preparation"))?;
        if bytes.len() > 64 * 1024 {
            return Err(anyhow!("Malformed E3 preparation"));
        }
        let uri = self
            .connector
            .build_uri("/v1/e3/config-projection/prepare")
            .map_err(|_| anyhow!("UnsupportedConfiguration"))?;
        let mut http_request = Request::builder()
            .method(Method::POST)
            .uri(uri)
            .header("content-type", "application/json")
            .body(Full::new(Bytes::from(bytes)))
            .map_err(|_| anyhow!("Malformed E3 preparation"))?;
        self.connector.prepare_request(&mut http_request);
        let response = self
            .connector
            .execute(http_request)
            .await
            .map_err(|_| anyhow!("E3 preparation transport failed"))?;
        if response.status() == StatusCode::NOT_FOUND {
            return Err(anyhow!("UnsupportedConfiguration"));
        }
        if !response.status().is_success() {
            return Err(anyhow!("E3 preparation rejected"));
        }
        let bytes = http_body_util::Limited::new(response.into_body(), 64 * 1024)
            .collect()
            .await
            .map_err(|_| anyhow!("Malformed E3 preparation response"))?
            .to_bytes();
        let response: transport_api_types::E3ConfigProjectionPrepareResponseV1 =
            serde_json::from_slice(&bytes)
                .map_err(|_| anyhow!("Malformed E3 preparation response"))?;
        if response.preparation_id != request.preparation_id
            || response.preparation_idempotency_key != request.preparation_idempotency_key
        {
            return Err(anyhow!("WrongBinding E3 preparation response"));
        }
        Ok(response)
    }

    /// Cancel the exact prepared carrier without retrying on another transport.
    pub async fn e3_config_projection_cancel(
        &self,
        request: transport_api_types::E3ConfigProjectionCancelRequestV1,
    ) -> Result<transport_api_types::E3ConfigProjectionCancelResponseV1> {
        if !matches!(&self.transport, Transport::UnixSocket { path } if path == Path::new("/run/substrate.sock"))
        {
            return Err(anyhow!("UnsupportedConfiguration"));
        }
        request
            .validate()
            .map_err(|_| anyhow!("Malformed E3 cancellation"))?;
        let response = self
            .post("/v1/e3/config-projection/cancel", &request)
            .await
            .map_err(|_| anyhow!("E3 cancellation transport failed"))?;
        if response.status() == StatusCode::NOT_FOUND {
            return Err(anyhow!("UnsupportedConfiguration"));
        }
        if !response.status().is_success() {
            return Err(anyhow!("E3 cancellation rejected"));
        }
        let bytes = http_body_util::Limited::new(response.into_body(), 64 * 1024)
            .collect()
            .await
            .map_err(|_| anyhow!("Malformed E3 cancellation response"))?
            .to_bytes();
        let response: transport_api_types::E3ConfigProjectionCancelResponseV1 =
            serde_json::from_slice(&bytes)
                .map_err(|_| anyhow!("Malformed E3 cancellation response"))?;
        if response.preparation_id != request.preparation_id
            || response.cancelled_dormant_projection_ref
                != request.config_projection.dormant_projection_ref
        {
            return Err(anyhow!("WrongBinding E3 cancellation response"));
        }
        Ok(response)
    }

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
    /// Ordinary initial member input uses its separately authenticated route.
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

    /// Initial ordinary input over the official Root-owned service connection.
    /// The carrying socket's ownership, peer PID/start identity and path are
    /// checked by the existing connector. There is no pooling, retry or fallback.
    /// The dedicated client remains alive until the returned body is dropped.
    /// ```no_run
    /// # async fn start(request: transport_api_types::ExecuteRequest) -> anyhow::Result<()> {
    /// let client = transport_api_client::AgentClient::unix_socket("/run/substrate.sock")?;
    /// let stream = client.execute_ordinary_member_stream(request).await?;
    /// # let _ = stream; Ok(()) }
    /// ```
    pub async fn execute_ordinary_member_stream(
        &self,
        request: ExecuteRequest,
    ) -> Result<Response<http_body_util::combinators::UnsyncBoxBody<Bytes, hyper::Error>>> {
        if !cfg!(target_os = "linux")
            || !matches!(&self.transport,
            Transport::UnixSocket { path } if path == Path::new("/run/substrate.sock"))
        {
            return Err(anyhow!(
                "ordinary_initial_execute_unsupported_configuration"
            ));
        }
        request
            .validate()
            .map_err(|_| anyhow!("ordinary_initial_execute_invalid_request"))?;
        if request.schema_version != 2
            || request.ordinary_start_selection.is_none()
            || request.member_dispatch.is_none()
        {
            return Err(anyhow!("ordinary_initial_execute_invalid_request"));
        }
        #[cfg(target_os = "linux")]
        {
            let bytes = serde_json::to_vec(&request)
                .map_err(|_| anyhow!("ordinary_initial_execute_invalid_request"))?;
            let mut builder =
                hyper_util::client::legacy::Client::builder(hyper_util::rt::TokioExecutor::new());
            builder
                .pool_max_idle_per_host(0)
                .retry_canceled_requests(false);
            let client = builder.build(HostGatewayInstanceConnectorV3);
            let request = Request::builder()
                .method(Method::POST)
                .uri("http://localhost/v1/ordinary/member/execute/stream")
                .header("content-type", "application/json")
                .body(Full::new(Bytes::from(bytes)))
                .map_err(|_| anyhow!("ordinary_initial_execute_invalid_request"))?;
            let response = client
                .request(request)
                .await
                .map_err(|_| anyhow!("ordinary_initial_execute_transport_unknown"))?;
            if !response.status().is_success() {
                return Err(anyhow!("ordinary_initial_execute_refused_or_unknown"));
            }
            Ok(response.map(|body| {
                body.map_frame(move |frame| {
                    let _keep_client = &client;
                    frame
                })
                .boxed_unsync()
            }))
        }
        #[cfg(not(target_os = "linux"))]
        Err(anyhow!(
            "ordinary_initial_execute_unsupported_configuration"
        ))
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

    /// Submit an ordinary retained turn. The receiver independently authenticates the exact DTO.
    ///
    /// ```no_run
    /// # async fn submit(client: &transport_api_client::AgentClient,
    /// # request: transport_api_types::OrdinaryMemberTurnSubmitRequestV1) -> anyhow::Result<()> {
    /// let _response = client.submit_ordinary_member_turn_stream(request).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn submit_ordinary_member_turn_stream(
        &self,
        request: transport_api_types::OrdinaryMemberTurnSubmitRequestV1,
    ) -> Result<Response<hyper::body::Incoming>> {
        let response = self
            .post("/v1/ordinary/member_turn/stream", &request)
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

    /// Send one closed Host turn operation on the installed Linux UDS only.
    /// No retry or alternate transport is attempted. Transport/decode errors and
    /// Unknown never acknowledge cleanup; retain the same acquisition key/secret.
    /// Server availability and independently held authority remain prerequisites.
    ///
    /// ```no_run
    /// use transport_api_client::AgentClient;
    /// use transport_api_types::{HostGatewayTurnRequestV1, HostGatewayTurnResponseV1};
    /// # async fn example(request: HostGatewayTurnRequestV1) -> anyhow::Result<HostGatewayTurnResponseV1> {
    /// let client = AgentClient::unix_socket("/run/substrate.sock")?;
    /// client.gateway_host_turn(request).await
    /// # }
    /// ```
    pub async fn gateway_host_turn(
        &self,
        request: transport_api_types::HostGatewayTurnRequestV1,
    ) -> Result<transport_api_types::HostGatewayTurnResponseV1> {
        if !cfg!(target_os = "linux")
            || !matches!(&self.transport, Transport::UnixSocket { path } if path == Path::new("/run/substrate.sock"))
        {
            return Err(anyhow!("host_gateway_turn_unsupported_configuration"));
        }
        request
            .validate()
            .map_err(|_| anyhow!("host_gateway_turn_invalid_request"))?;
        let bytes = serde_json::to_vec(&request)
            .map_err(|_| anyhow!("host_gateway_turn_invalid_request"))?;
        if bytes.len() > 64 * 1024 {
            return Err(anyhow!("host_gateway_turn_invalid_request"));
        }
        let bytes = host_gateway_turn_once(self.connector.as_ref(), bytes).await?;
        let response: transport_api_types::HostGatewayTurnResponseV1 =
            serde_json::from_slice(&bytes)
                .map_err(|_| anyhow!("host_gateway_turn_invalid_response"))?;
        response
            .validate_for_request(&request)
            .map_err(|_| anyhow!("host_gateway_turn_wrong_binding"))?;
        Ok(response)
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

#[cfg(target_os = "linux")]
impl AgentClient {
    /// Perform exactly one native-owner operation on its authenticated request connection.
    /// This never uses generic retries, TCP, readiness or execute fallback.
    ///
    /// ```no_run
    /// # async fn read(client: &transport_api_client::AgentClient, request: &transport_api_types::WorkspaceCurrentRequestV1) -> anyhow::Result<()> {
    /// let response = client.workspace_current_v1(request).await?;
    /// # let _ = response;
    /// # Ok(()) }
    /// ```
    pub async fn workspace_current_v1(
        &self,
        request: &transport_api_types::WorkspaceCurrentRequestV1,
    ) -> Result<transport_api_types::WorkspaceCurrentResponseV1> {
        request.validate().map_err(anyhow::Error::msg)?;
        let Transport::UnixSocket { path } = &self.transport else {
            return Err(anyhow!("Unsupported"));
        };
        let bytes = serde_json::to_vec(request)?;
        if bytes.len() > 32768 {
            return Err(anyhow!("Malformed"));
        }
        let response = transport::workspace_current_on_authenticated_uds_v1(path, bytes).await?;
        let response: transport_api_types::WorkspaceCurrentResponseV1 =
            serde_json::from_slice(&response)?;
        if let transport_api_types::WorkspaceCurrentResponseV1::CurrentActor(current) = &response {
            let transport_api_types::WorkspaceCurrentActorV1 {
                workspace_ref,
                workspace_incarnation,
                orchestration_session_id,
                world_id,
                world_generation,
                backend_id,
                authority_store_id,
                root_revision,
                authority_revision,
                authority_record_sha256,
                authoritative_lineage_sha256,
                lane_ref,
                association_incarnation,
                association_revision,
                backend_custody_ref,
                actor_id,
            } = current.as_ref();
            let transport_api_types::WorkspaceCurrentOperationV1::CurrentActorRead {
                lane_ref: lane,
                association_incarnation: incarnation,
                association_revision: revision,
                backend_custody_ref: custody,
                expected_actor,
            } = &request.operation
            else {
                return Err(anyhow!("WrongBinding"));
            };
            let r = &request.resource;
            let o = &request.observation;
            if workspace_ref != &r.workspace_ref
                || workspace_incarnation != &r.workspace_incarnation
                || orchestration_session_id != &r.orchestration_session_id
                || world_id != &r.world_id
                || world_generation != &r.world_generation
                || backend_id != &r.backend_id
                || authority_store_id != &o.authority_store_id
                || root_revision != &o.root_revision
                || authority_revision != &o.authority_revision
                || authority_record_sha256 != &o.authority_record_sha256
                || authoritative_lineage_sha256 != &o.authoritative_lineage_sha256
                || lane_ref != lane
                || association_incarnation != incarnation
                || association_revision != revision
                || backend_custody_ref != custody
                || actor_id != expected_actor
            {
                return Err(anyhow!("WrongBinding"));
            }
        }
        Ok(response)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(target_os = "linux")]
    #[tokio::test]
    async fn test_ordinary_initial_stream_client_refuses_other_transports_before_connect() {
        let root = tempfile::tempdir().unwrap();
        let path = root.path().join("unqualified.sock");
        let listener = std::os::unix::net::UnixListener::bind(&path).unwrap();
        listener.set_nonblocking(true).unwrap();
        for client in [
            AgentClient::unix_socket(&path).unwrap(),
            AgentClient::tcp("127.0.0.1", 9).unwrap(),
        ] {
            assert_eq!(
                client
                    .execute_ordinary_member_stream(e3a_passthrough_request())
                    .await
                    .err()
                    .unwrap()
                    .to_string(),
                "ordinary_initial_execute_unsupported_configuration"
            );
        }
        assert_eq!(
            listener.accept().err().unwrap().kind(),
            std::io::ErrorKind::WouldBlock
        );
    }

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
            schema_version: 1,
            ordinary_start_selection: None,
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

    #[tokio::test]
    async fn e3e_cancel_404_is_unsupported_without_retry_or_body_disclosure() {
        use tokio::io::{AsyncReadExt as _, AsyncWriteExt as _};
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();
        let server = tokio::spawn(async move {
            let (mut stream, _) = listener.accept().await.unwrap();
            let mut request = Vec::new();
            loop {
                let mut chunk = [0u8; 4096];
                let count = stream.read(&mut chunk).await.unwrap();
                assert_ne!(count, 0);
                request.extend_from_slice(&chunk[..count]);
                if request.windows(4).any(|window| window == b"\r\n\r\n") {
                    break;
                }
            }
            assert!(request.starts_with(b"POST /v1/e3/config-projection/cancel HTTP/1.1\r\n"));
            stream.write_all(b"HTTP/1.1 404 Not Found\r\ncontent-length: 1000000000\r\nconnection: close\r\n\r\nsynthetic-response-secret").await.unwrap();
            // Keep the misleading response body incomplete: 404 must not wait for or expose it.
            assert!(
                tokio::time::timeout(std::time::Duration::from_millis(200), listener.accept())
                    .await
                    .is_err()
            );
        });
        let transport = Transport::Tcp {
            host: "127.0.0.1".into(),
            port,
        };
        // Exercise HTTP behavior with a local test connector while preserving the production
        // metadata check. No privileged service socket is replaced for this unit test.
        let client = AgentClient {
            transport: Transport::UnixSocket {
                path: "/run/substrate.sock".into(),
            },
            connector: Arc::from(build_connector(&transport).unwrap()),
        };
        let dispatch = e3a_passthrough_request().member_dispatch.unwrap();
        let carrier = dispatch.config_projection().unwrap().clone();
        let request = transport_api_types::E3ConfigProjectionCancelRequestV1 {
            schema_version: 1,
            preparation_id: "e3p_018f0f2e-7b4c-7aa1-8c22-123456789ab0".into(),
            preparation_idempotency_key: format!("e3pik_{}", "a".repeat(64)),
            config_projection: carrier,
        };
        let error = tokio::time::timeout(
            std::time::Duration::from_secs(2),
            client.e3_config_projection_cancel(request),
        )
        .await
        .unwrap()
        .unwrap_err();
        assert_eq!(error.to_string(), "UnsupportedConfiguration");
        server.await.unwrap();
    }

    fn e3e_prepare_request() -> transport_api_types::E3ConfigProjectionPrepareRequestV1 {
        let dispatch = e3a_passthrough_request().member_dispatch.unwrap();
        let mut value = serde_json::to_value(dispatch).unwrap();
        let request = value.as_object_mut().unwrap();
        request.remove("initial_prompt");
        request.remove("config_projection");
        request.insert("schema_version".into(), serde_json::json!(1));
        request.insert("backend_id".into(), serde_json::json!("cli:codex-world"));
        request.get_mut("e2_launch_activation").unwrap()["target_backend_id"] =
            serde_json::json!("cli:codex-world");
        request.insert(
            "preparation_id".into(),
            serde_json::json!("e3p_01900000-0000-7000-8000-000000000001"),
        );
        request.insert("authoring_input_ref".into(), serde_json::from_str(r#"{"authority_store_id":"cpa_018f0f2e-7b4c-7aa1-8c22-123456789ab0","effective_config_source_hash":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa","agent_inventory_source_hash":"bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb","runtime_artifact_manifest_id":"ram_01900000-0000-7000-8000-000000000001","runtime_artifact_manifest_revision":1,"runtime_artifact_manifest_hash":"cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc","input_ref_hash":"ebb25473d73a319cbf2842b7970aa85b0783e1b160d071cce677a920cc457227"}"#).unwrap());
        request.insert(
            "resumed_from_participant_id".into(),
            serde_json::Value::Null,
        );
        request.insert(
            "retained_worker_launch_authority".into(),
            serde_json::Value::Null,
        );
        // Fixed canonical nonsecret fixture digest; strict decoding validates this binding.
        request.insert(
            "preparation_idempotency_key".into(),
            serde_json::json!(
                "e3pik_e5b6672d2eb535be6ffbe78e1f6a6979d2333700009931e5cc4632cf5f581f6d"
            ),
        );
        request.insert("integrated_auth".into(), serde_json::json!({
            "backend_id":"cli:codex-world", "cli_codex":{"access_token":"synthetic-e3-prepare-token", "account_id":null}
        }));
        serde_json::from_value(value).unwrap()
    }

    #[tokio::test]
    async fn e3e_prepare_404_is_unsupported_without_retry_or_response_body_read() {
        use tokio::io::{AsyncReadExt, AsyncWriteExt};
        let request = e3e_prepare_request();
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();
        let server = tokio::spawn(async move {
            let (mut stream, _) = listener.accept().await.unwrap();
            let mut bytes = Vec::new();
            let mut chunk = [0; 4096];
            loop {
                let n = stream.read(&mut chunk).await.unwrap();
                assert_ne!(n, 0);
                bytes.extend_from_slice(&chunk[..n]);
                let Some(header_end) = bytes.windows(4).position(|part| part == b"\r\n\r\n") else {
                    continue;
                };
                let header = std::str::from_utf8(&bytes[..header_end]).unwrap();
                assert!(header.starts_with("POST /v1/e3/config-projection/prepare HTTP/1.1\r\n"));
                let length: usize = header
                    .lines()
                    .find_map(|line| {
                        line.to_ascii_lowercase()
                            .strip_prefix("content-length:")
                            .map(|n| n.trim().parse().unwrap())
                    })
                    .unwrap();
                if bytes.len() >= header_end + 4 + length {
                    break;
                }
            }
            stream.write_all(b"HTTP/1.1 404 Not Found\r\ncontent-length: 1000000000\r\nconnection: close\r\n\r\nsynthetic-response-secret").await.unwrap();
            // Deliberately incomplete response: the client must return before reading it or retrying.
            assert!(
                tokio::time::timeout(std::time::Duration::from_millis(200), listener.accept())
                    .await
                    .is_err()
            );
        });
        let connector_transport = Transport::Tcp {
            host: "127.0.0.1".into(),
            port,
        };
        let client = AgentClient {
            transport: Transport::UnixSocket {
                path: "/run/substrate.sock".into(),
            },
            connector: Arc::from(build_connector(&connector_transport).unwrap()),
        };
        let error = tokio::time::timeout(
            std::time::Duration::from_secs(2),
            client.e3_config_projection_prepare(request),
        )
        .await
        .unwrap()
        .unwrap_err();
        assert_eq!(error.to_string(), "UnsupportedConfiguration");
        server.await.unwrap();
    }

    #[tokio::test]
    async fn e3e_cancel_rejects_tcp_and_alternate_uds_before_connecting() {
        let dispatch = e3a_passthrough_request().member_dispatch.unwrap();
        let request = transport_api_types::E3ConfigProjectionCancelRequestV1 {
            schema_version: 1,
            preparation_id: "e3p_018f0f2e-7b4c-7aa1-8c22-123456789ab0".into(),
            preparation_idempotency_key: format!("e3pik_{}", "a".repeat(64)),
            config_projection: dispatch.config_projection().unwrap().clone(),
        };
        for client in [
            AgentClient::tcp("127.0.0.1", 1).unwrap(),
            AgentClient::unix_socket("/not-an-e3-socket").unwrap(),
        ] {
            assert_eq!(
                client
                    .e3_config_projection_cancel(request.clone())
                    .await
                    .unwrap_err()
                    .to_string(),
                "UnsupportedConfiguration"
            );
        }
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

    /// Synthetic wire capture only: the server does not authenticate or execute.
    #[cfg(unix)]
    #[tokio::test]
    async fn ordinary_selected_execute_stream_preserves_strict_request_body() {
        use tokio::io::{AsyncReadExt as _, AsyncWriteExt as _};

        let directory = tempfile::tempdir().expect("capture socket directory");
        let socket = directory.path().join("capture.sock");
        let listener = tokio::net::UnixListener::bind(&socket).expect("bind capture socket");
        let server = tokio::spawn(async move {
            let (mut stream, _) = listener.accept().await.expect("accept selected request");
            let mut bytes = Vec::new();
            let header_end = loop {
                let mut chunk = [0u8; 4096];
                let count = stream.read(&mut chunk).await.expect("read request headers");
                assert!(count > 0, "client closed before headers");
                bytes.extend_from_slice(&chunk[..count]);
                assert!(
                    bytes.len() <= 65536,
                    "capture request exceeds fixture bound"
                );
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
            assert!(
                header_end + content_length <= 65536,
                "capture body exceeds fixture bound"
            );
            while bytes.len() - header_end < content_length {
                let mut chunk = [0u8; 4096];
                let count = stream.read(&mut chunk).await.expect("read selected body");
                assert!(count > 0, "client closed before body");
                bytes.extend_from_slice(&chunk[..count]);
            }
            let body = bytes[header_end..header_end + content_length].to_vec();
            let decoded: transport_api_types::ExecuteRequest =
                serde_json::from_slice(&body).expect("strict server-side selected request decode");
            decoded
                .validate()
                .expect("validate captured selected request");
            assert_eq!(decoded.schema_version, 2);
            assert!(decoded.ordinary_start_selection.is_some());
            assert!(decoded.member_dispatch.as_ref().unwrap().as_v1().is_some());
            stream
                .write_all(b"HTTP/1.1 200 OK\r\ncontent-length: 0\r\nconnection: close\r\n\r\n")
                .await
                .expect("write capture-only response");
            body
        });

        let request: transport_api_types::ExecuteRequest =
            serde_json::from_value(serde_json::json!({
                "schema_version": 2,
                "ordinary_start_selection": {
                    "schema_version": 1,
                    "authority_store_id": "store_original",
                    "original_start_intent_id": "intent_original",
                    "original_start_issuer_request_id": format!("req_{}", "a".repeat(64)),
                    "original_session_id": "session_original",
                    "context_sha256": "b".repeat(64),
                    "native_selection_sha256": "c".repeat(64),
                    "requested_selection_sha256": "d".repeat(64)
                },
                "profile": null,
                "cmd": "",
                "cwd": "/project",
                "env": null,
                "pty": false,
                "agent_id": "host_actor",
                "budget": null,
                "policy_snapshot": {
                    "schema_version": 3,
                    "net_allowed": [],
                    "world_fs": {
                        "host_visible": true,
                        "fail_closed": { "routing": false },
                        "caged_required": false,
                        "write": { "enabled": true, "allow_list": ["."], "deny_list": [] }
                    }
                },
                "member_dispatch": {
                    "schema_version": 1,
                    "orchestration_session_id": "session_original",
                    "participant_id": "world_worker",
                    "orchestrator_participant_id": "host_actor",
                    "backend_id": "cli:worker-world",
                    "protocol": "substrate.agent.session",
                    "run_id": "run_original",
                    "world_id": "world_original",
                    "world_generation": 7,
                    "initial_prompt": "hello",
                    "resolved_runtime": { "backend_kind": "codex", "binary_path": "/usr/bin/codex" }
                }
            }))
            .expect("synthetic selected request fixture");
        let expected = serde_json::to_vec(&request).expect("expected complete request bytes");
        let client = AgentClient::unix_socket(&socket).expect("real Unix-socket AgentClient");
        tokio::time::timeout(
            std::time::Duration::from_secs(5),
            client.execute_stream(request),
        )
        .await
        .expect("client capture timeout")
        .expect("capture-only response");
        let captured = tokio::time::timeout(std::time::Duration::from_secs(5), server)
            .await
            .expect("server capture timeout")
            .expect("capture server task");
        assert_eq!(
            captured, expected,
            "AgentClient must preserve the complete selected request bytes"
        );
    }
}

#[cfg(all(test, target_os = "linux"))]
mod host_gateway_turn_client_tests {
    use super::*;
    use tokio::io::{AsyncReadExt as _, AsyncWriteExt as _};
    use transport_api_types::{
        HostGatewayTurnOperationV1, HostGatewayTurnRequestV1, HostGatewayTurnResultV1,
    };

    fn release() -> HostGatewayTurnRequestV1 {
        let token = "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA";
        HostGatewayTurnRequestV1 {
            schema_version: 1,
            operation: HostGatewayTurnOperationV1::Release {
                acquire_id: HostGatewayTurnRequestV1::acquire_id_from_cleanup_token(token).unwrap(),
                lease_handle: None,
                cleanup_token: token.into(),
            },
        }
    }

    // Only the test connector uses a fixture socket. This does not establish
    // inherited-listener, service authentication, authority or cleanup proof.
    async fn exchange(
        status: &str,
        body: &str,
        request: HostGatewayTurnRequestV1,
    ) -> Result<transport_api_types::HostGatewayTurnResponseV1> {
        let directory = tempfile::tempdir().unwrap();
        let socket = directory.path().join("host-turn.sock");
        let listener = tokio::net::UnixListener::bind(&socket).unwrap();
        let status = status.to_owned();
        let body = body.to_owned();
        let expected = serde_json::to_vec(&request).unwrap();
        let server = tokio::spawn(async move {
            let (mut stream, _) = listener.accept().await.unwrap();
            let mut bytes = Vec::new();
            let (header_end, content_length) = loop {
                let mut chunk = [0u8; 4096];
                let count = stream.read(&mut chunk).await.unwrap();
                assert!(count > 0);
                bytes.extend_from_slice(&chunk[..count]);
                assert!(bytes.len() <= 65536);
                if let Some(end) = bytes.windows(4).position(|w| w == b"\r\n\r\n") {
                    let end = end + 4;
                    let headers = std::str::from_utf8(&bytes[..end]).unwrap();
                    assert!(headers.starts_with("POST /v1/gateway/host-turn HTTP/1.1\r\n"));
                    let length = headers
                        .lines()
                        .find_map(|line| {
                            line.to_ascii_lowercase()
                                .strip_prefix("content-length: ")
                                .and_then(|value| value.parse::<usize>().ok())
                        })
                        .unwrap();
                    break (end, length);
                }
            };
            while bytes.len() < header_end + content_length {
                let mut chunk = [0u8; 4096];
                let count = stream.read(&mut chunk).await.unwrap();
                assert!(count > 0);
                bytes.extend_from_slice(&chunk[..count]);
                assert!(bytes.len() <= 65536);
            }
            assert_eq!(&bytes[header_end..header_end + content_length], expected);
            let decoded: HostGatewayTurnRequestV1 = serde_json::from_slice(&expected).unwrap();
            assert!(matches!(
                decoded.operation,
                HostGatewayTurnOperationV1::Release {
                    lease_handle: None,
                    ..
                }
            ));
            stream.write_all(format!("HTTP/1.1 {status}\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{body}", body.len()).as_bytes()).await.unwrap();
        });
        let fixture_transport = Transport::UnixSocket { path: socket };
        let client = AgentClient {
            transport: Transport::UnixSocket {
                path: "/run/substrate.sock".into(),
            },
            connector: Arc::from(build_connector(&fixture_transport).unwrap()),
        };
        let result = tokio::time::timeout(
            std::time::Duration::from_secs(5),
            client.gateway_host_turn(request),
        )
        .await
        .expect("bounded client exchange");
        tokio::time::timeout(std::time::Duration::from_secs(5), server)
            .await
            .unwrap()
            .unwrap();
        result
    }

    #[tokio::test]
    async fn test_host_gateway_turn_client_release_unknown_and_safe_errors() {
        let id = HostGatewayTurnRequestV1::acquire_id_from_cleanup_token(
            "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA",
        )
        .unwrap();
        for kind in ["released", "unknown"] {
            let body = serde_json::json!({"schema_version": 1, "result": {"kind": kind, "acquire_id": id}}).to_string();
            let response = exchange("200 OK", &body, release()).await.unwrap();
            assert!(
                matches!(response.result, HostGatewayTurnResultV1::Released { .. })
                    == (kind == "released")
            );
            assert!(
                matches!(response.result, HostGatewayTurnResultV1::Unknown { .. })
                    == (kind == "unknown")
            );
        }
        let error = exchange("403 Forbidden", "reflected-secret-and-ticket", release())
            .await
            .err()
            .unwrap();
        assert_eq!(format!("{error:#}"), "host_gateway_turn_rejected");
        let error = exchange("404 Not Found", "reflected-secret-and-ticket", release())
            .await
            .err()
            .unwrap();
        assert_eq!(
            format!("{error:#}"),
            "host_gateway_turn_unsupported_configuration"
        );
    }

    #[tokio::test]
    async fn test_host_gateway_turn_client_strict_response_ingress() {
        let id = HostGatewayTurnRequestV1::acquire_id_from_cleanup_token(
            "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA",
        )
        .unwrap();
        let valid =
            format!(r#"{{"schema_version":1,"result":{{"kind":"released","acquire_id":"{id}"}}}}"#);
        let duplicate = valid.replace(
            "\"schema_version\":1",
            "\"schema_version\":1,\"schema_version\":1",
        );
        let wrong_id = valid.replace(&id, &format!("hgt_{}", "f".repeat(64)));
        let wrong_kind = valid.replace("released", "revalidated");
        for body in [
            duplicate,
            wrong_id,
            wrong_kind,
            valid.replace("\"schema_version\":1", "\"schema_version\":2"),
            valid.replace(
                "\"kind\":\"released\"",
                "\"kind\":\"released\",\"principal\":0",
            ),
            "x".repeat(65537),
        ] {
            let error = exchange("200 OK", &body, release()).await.err().unwrap();
            assert!(matches!(
                error.to_string().as_str(),
                "host_gateway_turn_invalid_response" | "host_gateway_turn_wrong_binding"
            ));
        }
    }

    #[tokio::test]
    async fn test_host_gateway_turn_client_rejects_other_transports_before_send() {
        let directory = tempfile::tempdir().unwrap();
        let socket = directory.path().join("unused.sock");
        let listener = tokio::net::UnixListener::bind(&socket).unwrap();
        for client in [
            AgentClient::unix_socket(&socket).unwrap(),
            AgentClient::tcp("127.0.0.1", 9).unwrap(),
        ] {
            let error = client.gateway_host_turn(release()).await.err().unwrap();
            assert_eq!(
                error.to_string(),
                "host_gateway_turn_unsupported_configuration"
            );
        }
        assert!(
            tokio::time::timeout(std::time::Duration::from_millis(20), listener.accept())
                .await
                .is_err()
        );
    }

    #[tokio::test]
    async fn test_host_gateway_turn_client_uses_fresh_keep_alive_connections() {
        use std::collections::HashSet;
        use std::sync::atomic::{AtomicUsize, Ordering};

        let directory = tempfile::tempdir().unwrap();
        let socket = directory.path().join("keep-alive.sock");
        let listener = tokio::net::UnixListener::bind(&socket).unwrap();
        let (observed, mut observations) = tokio::sync::mpsc::unbounded_channel();
        let (shutdown, mut stop) = tokio::sync::oneshot::channel::<()>();
        let accepted = Arc::new(AtomicUsize::new(0));
        let server = tokio::spawn(async move {
            let mut connections = tokio::task::JoinSet::new();
            loop {
                tokio::select! {
                    _ = &mut stop => break,
                    incoming = listener.accept() => {
                        let (mut stream, _) = incoming.unwrap();
                        let connection = accepted.fetch_add(1, Ordering::SeqCst);
                        let observed = observed.clone();
                        connections.spawn(async move {
                            let mut pending = Vec::new();
                            loop {
                                let (header_end, length) = loop {
                                    if let Some(end) = pending.windows(4).position(|w| w == b"\r\n\r\n") {
                                        let end = end + 4;
                                        let headers = std::str::from_utf8(&pending[..end]).unwrap();
                                        assert!(headers.starts_with("POST /v1/gateway/host-turn HTTP/1.1\r\n"));
                                        let length = headers.lines().find_map(|line| line.to_ascii_lowercase().strip_prefix("content-length: ").and_then(|value| value.parse::<usize>().ok())).unwrap();
                                        break (end, length);
                                    }
                                    let mut chunk = [0u8; 4096];
                                    let count = stream.read(&mut chunk).await.unwrap();
                                    if count == 0 { return; }
                                    pending.extend_from_slice(&chunk[..count]);
                                    assert!(pending.len() <= 65536);
                                };
                                while pending.len() < header_end + length {
                                    let mut chunk = [0u8; 4096];
                                    let count = stream.read(&mut chunk).await.unwrap();
                                    assert!(count > 0);
                                    pending.extend_from_slice(&chunk[..count]);
                                    assert!(pending.len() <= 65536);
                                }
                                let body: serde_json::Value = serde_json::from_slice(&pending[header_end..header_end + length]).unwrap();
                                let decoded: HostGatewayTurnRequestV1 = serde_json::from_slice(&pending[header_end..header_end + length]).unwrap();
                                decoded.validate().unwrap();
                                pending.drain(..header_end + length);
                                let reply = serde_json::json!({"schema_version": 1, "result": {
                                    "kind": "unknown", "acquire_id": body["operation"]["acquire_id"]
                                }}).to_string();
                                observed.send((connection, body["operation"]["kind"].as_str().unwrap().to_owned())).unwrap();
                                // Deliberately keep the connection reusable. The old
                                // connector must not be rescued by a close response.
                                stream.write_all(format!("HTTP/1.1 200 OK\r\ncontent-length: {}\r\nconnection: keep-alive\r\n\r\n{reply}", reply.len()).as_bytes()).await.unwrap();
                            }
                        });
                    }
                }
            }
            connections.abort_all();
            while connections.join_next().await.is_some() {}
        });
        let fixture_transport = Transport::UnixSocket { path: socket };
        let client = AgentClient {
            transport: Transport::UnixSocket {
                path: "/run/substrate.sock".into(),
            },
            connector: Arc::from(build_connector(&fixture_transport).unwrap()),
        };
        let release = release();
        let id = match &release.operation {
            HostGatewayTurnOperationV1::Release { acquire_id, .. } => acquire_id.clone(),
            _ => unreachable!(),
        };
        let revalidate = HostGatewayTurnRequestV1 {
            schema_version: 1,
            operation: HostGatewayTurnOperationV1::Revalidate {
                acquire_id: id,
                lease_handle: format!("hgl_{}_{}", "1".repeat(32), "2".repeat(32)),
            },
        };
        let mut seen = Vec::new();
        for request in [revalidate.clone(), release, revalidate] {
            let response = tokio::time::timeout(
                std::time::Duration::from_secs(5),
                client.gateway_host_turn(request),
            )
            .await
            .unwrap()
            .unwrap();
            assert!(matches!(
                response.result,
                HostGatewayTurnResultV1::Unknown { .. }
            ));
            seen.push(
                tokio::time::timeout(std::time::Duration::from_secs(5), observations.recv())
                    .await
                    .unwrap()
                    .unwrap(),
            );
            tokio::task::yield_now().await;
        }
        let _ = shutdown.send(());
        tokio::time::timeout(std::time::Duration::from_secs(5), server)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(
            seen.iter()
                .map(|(_, kind)| kind.as_str())
                .collect::<Vec<_>>(),
            ["revalidate", "release", "revalidate"]
        );
        assert_eq!(
            seen.iter()
                .map(|(connection, _)| *connection)
                .collect::<HashSet<_>>()
                .len(),
            3,
            "every Host operation needs a new Unix accept/peer authentication boundary"
        );
    }
}

#[cfg(target_os = "linux")]
#[derive(Clone)]
struct HostGatewayInstanceConnectorV3;
#[cfg(target_os = "linux")]
impl tower_service::Service<hyper::Uri> for HostGatewayInstanceConnectorV3 {
    type Response = hyper_util::rt::TokioIo<tokio::net::UnixStream>;
    type Error = std::io::Error;
    type Future = std::pin::Pin<
        Box<dyn std::future::Future<Output = std::io::Result<Self::Response>> + Send>,
    >;
    fn poll_ready(
        &mut self,
        _: &mut std::task::Context<'_>,
    ) -> std::task::Poll<std::io::Result<()>> {
        std::task::Poll::Ready(Ok(()))
    }
    fn call(&mut self, _: hyper::Uri) -> Self::Future {
        Box::pin(async {
            connect_host_gateway_instance_v3()
                .await
                .map(hyper_util::rt::TokioIo::new)
        })
    }
}

#[cfg(target_os = "linux")]
fn host_gateway_substrate_gid_v3() -> std::io::Result<u32> {
    let refuse = || {
        std::io::Error::new(
            std::io::ErrorKind::PermissionDenied,
            "Host service group unavailable",
        )
    };
    let mut size = 1024usize;
    loop {
        let mut bytes = vec![0u8; size];
        let mut group = std::mem::MaybeUninit::<libc::group>::uninit();
        let mut result = std::ptr::null_mut();
        let status = unsafe {
            libc::getgrnam_r(
                c"substrate".as_ptr(),
                group.as_mut_ptr(),
                bytes.as_mut_ptr().cast(),
                bytes.len(),
                &mut result,
            )
        };
        if status == libc::ERANGE && size < 65_536 {
            size = (size * 2).min(65_536);
            continue;
        }
        if status != 0 || result.is_null() || result != group.as_mut_ptr() {
            return Err(refuse());
        }
        let gid = unsafe { group.assume_init() }.gr_gid;
        if gid == 0 {
            return Err(refuse());
        }
        return Ok(gid);
    }
}

#[cfg(target_os = "linux")]
async fn connect_host_gateway_instance_v3() -> std::io::Result<tokio::net::UnixStream> {
    use std::os::unix::fs::{FileTypeExt, MetadataExt};
    let refuse = || {
        std::io::Error::new(
            std::io::ErrorKind::PermissionDenied,
            "Host service endpoint unauthenticated",
        )
    };
    let gid = host_gateway_substrate_gid_v3()?;
    let directory = std::fs::symlink_metadata("/run")?;
    if !directory.is_dir() || directory.uid() != 0 || directory.mode() & 0o022 != 0 {
        return Err(refuse());
    }
    let before = std::fs::symlink_metadata("/run/substrate.sock")?;
    if !before.file_type().is_socket()
        || before.uid() != 0
        || before.gid() != gid
        || before.mode() & 0o7777 != 0o660
    {
        return Err(refuse());
    }
    let stream = tokio::net::UnixStream::connect("/run/substrate.sock").await?;
    if stream.peer_cred()?.uid() != 0 {
        return Err(refuse());
    }
    let after = std::fs::symlink_metadata("/run/substrate.sock")?;
    let parent = std::fs::symlink_metadata("/run")?;
    if after.dev() != before.dev()
        || after.ino() != before.ino()
        || after.uid() != before.uid()
        || after.gid() != before.gid()
        || after.mode() != before.mode()
        || !after.file_type().is_socket()
        || parent.dev() != directory.dev()
        || parent.ino() != directory.ino()
        || parent.uid() != directory.uid()
        || parent.mode() != directory.mode()
    {
        return Err(refuse());
    }
    Ok(stream)
}

impl AgentClient {
    /// Use the strict instance-bound Host API over a fresh, authenticated root
    /// service connection. Observation grants no execution admission. Keep the
    /// original epoch/key after any uncertainty; never reobserve to redeliver.
    ///
    /// ```no_run
    /// # async fn example() -> anyhow::Result<()> {
    /// let client = transport_api_client::AgentClient::unix_socket("/run/substrate.sock")?;
    /// let request = transport_api_types::HostGatewayInstanceRequestV3 {
    ///     schema_version: 3,
    ///     operation: transport_api_types::HostGatewayInstanceOperationV3::Observe {},
    /// };
    /// let _ = client.gateway_host_instance(request).await?;
    /// # Ok(()) }
    /// ```
    pub async fn gateway_host_instance(
        &self,
        request: transport_api_types::HostGatewayInstanceRequestV3,
    ) -> Result<transport_api_types::HostGatewayInstanceResponseV3> {
        if !cfg!(target_os = "linux")
            || !matches!(&self.transport, Transport::UnixSocket { path } if path == Path::new("/run/substrate.sock"))
        {
            return Err(anyhow!("host_gateway_instance_unsupported_configuration"));
        }
        request
            .validate()
            .map_err(|_| anyhow!("host_gateway_instance_invalid_request"))?;
        #[cfg(target_os = "linux")]
        {
            let bytes = serde_json::to_vec(&request)
                .map_err(|_| anyhow!("host_gateway_instance_invalid_request"))?;
            if bytes.len() > 64 * 1024 {
                return Err(anyhow!("host_gateway_instance_invalid_request"));
            }
            let mut builder =
                hyper_util::client::legacy::Client::builder(hyper_util::rt::TokioExecutor::new());
            builder
                .pool_max_idle_per_host(0)
                .retry_canceled_requests(false);
            let client = builder.build(HostGatewayInstanceConnectorV3);
            let http_request = Request::builder()
                .method(Method::POST)
                .uri("http://localhost/v1/gateway/host-turn")
                .header("content-type", "application/json")
                .body(Full::new(Bytes::from(bytes)))
                .map_err(|_| anyhow!("host_gateway_instance_invalid_request"))?;
            let response = client
                .request(http_request)
                .await
                .map_err(|_| anyhow!("host_gateway_instance_transport_failed"))?;
            if response.status() == StatusCode::NOT_FOUND {
                return Err(anyhow!("host_gateway_instance_unsupported_configuration"));
            }
            if !response.status().is_success() {
                return Err(anyhow!("host_gateway_instance_refused_or_unknown"));
            }
            let bytes = http_body_util::Limited::new(response.into_body(), 64 * 1024)
                .collect()
                .await
                .map_err(|_| anyhow!("host_gateway_instance_invalid_response"))?
                .to_bytes();
            let response: transport_api_types::HostGatewayInstanceResponseV3 =
                serde_json::from_slice(&bytes)
                    .map_err(|_| anyhow!("host_gateway_instance_invalid_response"))?;
            response
                .validate_for_request(&request)
                .map_err(|_| anyhow!("host_gateway_instance_wrong_binding"))?;
            Ok(response)
        }
        #[cfg(not(target_os = "linux"))]
        Err(anyhow!("host_gateway_instance_unsupported_configuration"))
    }
}

#[cfg(test)]
mod host_gateway_instance_v3_client_tests {
    use super::*;
    fn observe() -> transport_api_types::HostGatewayInstanceRequestV3 {
        transport_api_types::HostGatewayInstanceRequestV3 {
            schema_version: 3,
            operation: transport_api_types::HostGatewayInstanceOperationV3::Observe {},
        }
    }
    #[cfg(unix)]
    #[tokio::test]
    async fn test_instance_client_refuses_other_transports_before_observation() {
        for client in [
            AgentClient::unix_socket("/tmp/unqualified-host.sock").unwrap(),
            AgentClient::tcp("127.0.0.1", 1).unwrap(),
        ] {
            let error = client.gateway_host_instance(observe()).await.err().unwrap();
            assert_eq!(
                error.to_string(),
                "host_gateway_instance_unsupported_configuration"
            );
        }
    }
    #[cfg(target_os = "linux")]
    #[tokio::test]
    async fn test_instance_client_refuses_bad_epoch_before_connecting() {
        let token = "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA";
        let request = transport_api_types::HostGatewayInstanceRequestV3 {
            schema_version: 3,
            operation: transport_api_types::HostGatewayInstanceOperationV3::Bound {
                service_instance_id: "caller-selected-invalid-epoch".into(),
                route_verifier: None,
                operation: transport_api_types::HostGatewayTurnOperationV1::Release {
                    acquire_id: transport_api_types::HostGatewayTurnRequestV1::acquire_id_from_cleanup_token(token).unwrap(),
                    lease_handle: None, cleanup_token: token.into(),
                },
            },
        };
        let client = AgentClient::unix_socket("/run/substrate.sock").unwrap();
        let error = client.gateway_host_instance(request).await.err().unwrap();
        assert_eq!(error.to_string(), "host_gateway_instance_invalid_request");
    }
}

/// Response obtained over the actual authenticated installed root-service UDS.
/// It is neither Clone nor deserializable. A decoded DTO alone cannot become
/// this response provenance; only the connected service method constructs it.
/// The runtime still authenticates its original intent and accepted-work cut.
/// ```compile_fail
/// use transport_api_client::OrdinaryRetainedStopServiceResponseV1;
/// let _ = OrdinaryRetainedStopServiceResponseV1 {
///     response: transport_api_types::OrdinaryRetainedWorkerStopResponseV1::UnknownOwner {},
/// };
/// ```
pub struct OrdinaryRetainedStopServiceResponseV1 {
    response: transport_api_types::OrdinaryRetainedWorkerStopResponseV1,
}

impl OrdinaryRetainedStopServiceResponseV1 {
    /// Inspect response data. Only Retired contains an exact acknowledgment;
    /// busy/unknown state never establishes worker retirement or cap release.
    /// ```no_run
    /// # fn inspect(reply: &transport_api_client::OrdinaryRetainedStopServiceResponseV1) {
    /// let data = reply.response();
    /// # let _ = data;
    /// # }
    /// ```
    pub fn response(&self) -> &transport_api_types::OrdinaryRetainedWorkerStopResponseV1 {
        &self.response
    }
}

impl AgentClient {
    /// Deliver an admitted initial control request to its exact installed Root
    /// owner. A latched/delivered response is not settlement or retirement.
    /// Only the authenticated official UDS is supported, without pooling,
    /// request retry, TCP or pathname fallback.
    /// ```no_run
    /// # async fn control(request: &transport_api_types::OrdinaryInitialBootstrapControlRequestV1) -> anyhow::Result<()> {
    /// let client = transport_api_client::AgentClient::unix_socket("/run/substrate.sock")?;
    /// let delivery = client.control_ordinary_initial_bootstrap(request).await?;
    /// # let _ = delivery; Ok(()) }
    /// ```
    pub async fn control_ordinary_initial_bootstrap(
        &self,
        request: &transport_api_types::OrdinaryInitialBootstrapControlRequestV1,
    ) -> Result<transport_api_types::OrdinaryInitialBootstrapControlResponseV1> {
        if !cfg!(target_os = "linux")
            || !matches!(&self.transport,
            Transport::UnixSocket { path } if path == Path::new("/run/substrate.sock"))
        {
            return Err(anyhow!(
                "ordinary_initial_control_unsupported_configuration"
            ));
        }
        request
            .validate()
            .map_err(|_| anyhow!("ordinary_initial_control_invalid_request"))?;
        #[cfg(target_os = "linux")]
        {
            let bytes = serde_json::to_vec(request)
                .map_err(|_| anyhow!("ordinary_initial_control_invalid_request"))?;
            let mut builder =
                hyper_util::client::legacy::Client::builder(hyper_util::rt::TokioExecutor::new());
            builder
                .pool_max_idle_per_host(0)
                .retry_canceled_requests(false);
            let client = builder.build(HostGatewayInstanceConnectorV3);
            let outgoing = Request::builder()
                .method(Method::POST)
                .uri("http://localhost/v1/ordinary/member/bootstrap/control")
                .header("content-type", "application/json")
                .body(Full::new(Bytes::from(bytes)))
                .map_err(|_| anyhow!("ordinary_initial_control_invalid_request"))?;
            let response = client
                .request(outgoing)
                .await
                .map_err(|_| anyhow!("ordinary_initial_control_transport_unknown"))?;
            if !response.status().is_success() {
                return Err(anyhow!("ordinary_initial_control_refused_or_unknown"));
            }
            let bytes = http_body_util::Limited::new(response.into_body(), 32768)
                .collect()
                .await
                .map_err(|_| anyhow!("ordinary_initial_control_invalid_response"))?
                .to_bytes();
            let response: transport_api_types::OrdinaryInitialBootstrapControlResponseV1 =
                serde_json::from_slice(&bytes)
                    .map_err(|_| anyhow!("ordinary_initial_control_invalid_response"))?;
            if let transport_api_types::OrdinaryInitialBootstrapControlDeliveryV1::Retired {
                report,
            } = &response.delivery
            {
                anyhow::ensure!(
                    report.matches_control(request),
                    "ordinary_initial_control_invalid_response"
                );
            }
            Ok(response)
        }
        #[cfg(not(target_os = "linux"))]
        Err(anyhow!(
            "ordinary_initial_control_unsupported_configuration"
        ))
    }

    /// Submit one admitted ordinary Stop to the installed root-owned UDS. The
    /// same connection authenticates SO_PEERCRED and socket identity before any
    /// HTTP bytes are sent. No pooled connection, retry, TCP or path fallback.
    /// ```no_run
    /// # async fn stop(request: &transport_api_types::OrdinaryRetainedWorkerStopRequestV1) -> anyhow::Result<()> {
    /// let client = transport_api_client::AgentClient::unix_socket("/run/substrate.sock")?;
    /// let reply = client.retire_ordinary_retained_worker(request).await?;
    /// # let _ = reply; Ok(())
    /// # }
    /// ```
    pub async fn retire_ordinary_retained_worker(
        &self,
        request: &transport_api_types::OrdinaryRetainedWorkerStopRequestV1,
    ) -> Result<OrdinaryRetainedStopServiceResponseV1> {
        if !cfg!(target_os = "linux")
            || !matches!(&self.transport, Transport::UnixSocket { path } if path == Path::new("/run/substrate.sock"))
        {
            return Err(anyhow!("ordinary_retained_stop_unsupported_configuration"));
        }
        request
            .validate()
            .map_err(|_| anyhow!("ordinary_retained_stop_invalid_request"))?;
        #[cfg(target_os = "linux")]
        {
            let bytes = serde_json::to_vec(request)
                .map_err(|_| anyhow!("ordinary_retained_stop_invalid_request"))?;
            if bytes.len() > 32768 {
                return Err(anyhow!("ordinary_retained_stop_invalid_request"));
            }
            // Reuse the established installed root-service connector. Its
            // checks bind /run, exact root:substrate 0660 socket and peer UID on
            // the stream carrying this request; no new authentication scheme.
            let mut builder =
                hyper_util::client::legacy::Client::builder(hyper_util::rt::TokioExecutor::new());
            builder
                .pool_max_idle_per_host(0)
                .retry_canceled_requests(false);
            let client = builder.build(HostGatewayInstanceConnectorV3);
            let http_request = Request::builder()
                .method(Method::POST)
                .uri("http://localhost/v1/ordinary/member/stop")
                .header("content-type", "application/json")
                .body(Full::new(Bytes::from(bytes)))
                .map_err(|_| anyhow!("ordinary_retained_stop_invalid_request"))?;
            let response = client
                .request(http_request)
                .await
                .map_err(|_| anyhow!("ordinary_retained_stop_transport_unknown"))?;
            if !response.status().is_success() {
                return Err(anyhow!("ordinary_retained_stop_refused_or_unknown"));
            }
            let bytes = http_body_util::Limited::new(response.into_body(), 32768)
                .collect()
                .await
                .map_err(|_| anyhow!("ordinary_retained_stop_invalid_response"))?
                .to_bytes();
            let response: transport_api_types::OrdinaryRetainedWorkerStopResponseV1 =
                serde_json::from_slice(&bytes)
                    .map_err(|_| anyhow!("ordinary_retained_stop_invalid_response"))?;
            if let transport_api_types::OrdinaryRetainedWorkerStopResponseV1::Retired {
                acknowledgment,
            } = &response
            {
                acknowledgment
                    .validate(request)
                    .map_err(|_| anyhow!("ordinary_retained_stop_wrong_binding"))?;
            }
            Ok(OrdinaryRetainedStopServiceResponseV1 { response })
        }
        #[cfg(not(target_os = "linux"))]
        Err(anyhow!("ordinary_retained_stop_unsupported_configuration"))
    }
}

#[cfg(all(test, target_os = "linux"))]
mod ordinary_retained_stop_client_tests {
    use super::*;
    fn ordinary_stop_request_data_for_client_test(
    ) -> transport_api_types::OrdinaryRetainedWorkerStopRequestV1 {
        transport_api_types::OrdinaryRetainedWorkerStopRequestV1 {
            schema_version: 1,
            stop_request_id: "request-codec-only".into(),
            authority_store_id: "authority-codec-only".into(),
            orchestration_session_id: "session-codec-only".into(),
            caller_participant_id: "caller-codec-only".into(),
            target_participant_id: "target-codec-only".into(),
            target_backend_id: "cli:codex-world".into(),
            world_id: "world-codec-only".into(),
            world_generation: 0,
            registration_id: "registration-codec-only".into(),
            bootstrap_run_id: "bootstrap-codec-only".into(),
            native_session_id: "native-codec-only".into(),
            worker_cap_ref: transport_api_types::DispatchPolicyCommitmentRefCarrierV1 {
                authority_store_id: "authority-codec-only".into(),
                commitment_id: "dpc_019c1e90-7cb4-7aa0-8aaa-aaaaaaaaaaaa".into(),
                exact_linkage_hash: "a".repeat(64),
            },
        }
    }
    #[tokio::test]
    async fn test_ordinary_stop_client_refuses_other_transports_before_connect() {
        let root = tempfile::tempdir().unwrap();
        let socket = root.path().join("unqualified.sock");
        let listener = std::os::unix::net::UnixListener::bind(&socket).unwrap();
        listener.set_nonblocking(true).unwrap();
        let request = ordinary_stop_request_data_for_client_test();
        request.validate().unwrap();
        for client in [
            AgentClient::unix_socket(&socket).unwrap(),
            AgentClient::tcp("127.0.0.1", 1).unwrap(),
        ] {
            assert_eq!(
                client
                    .retire_ordinary_retained_worker(&request)
                    .await
                    .err()
                    .unwrap()
                    .to_string(),
                "ordinary_retained_stop_unsupported_configuration"
            );
        }
        assert_eq!(
            listener.accept().unwrap_err().kind(),
            std::io::ErrorKind::WouldBlock
        );
    }
    #[tokio::test]
    async fn test_ordinary_stop_client_refuses_invalid_data_before_connect() {
        let mut request = ordinary_stop_request_data_for_client_test();
        request.native_session_id.clear();
        let client = AgentClient::unix_socket("/run/substrate.sock").unwrap();
        assert_eq!(
            client
                .retire_ordinary_retained_worker(&request)
                .await
                .err()
                .unwrap()
                .to_string(),
            "ordinary_retained_stop_invalid_request"
        );
    }
}

impl AgentClient {
    /// Deliver one source-authorized ordinary accepted cancellation over the
    /// authenticated installed root UDS. Connection failure/response loss is
    /// ambiguous to the existing Supervisor lease; this call never retries.
    /// ```no_run
    /// # async fn cancel(request: &transport_api_types::OrdinaryAcceptedWorkCancelRequestV1) -> anyhow::Result<()> {
    /// let client = transport_api_client::AgentClient::unix_socket("/run/substrate.sock")?;
    /// let reply = client.cancel_ordinary_accepted_work(request).await?;
    /// # let _ = reply; Ok(())
    /// # }
    /// ```
    pub async fn cancel_ordinary_accepted_work(
        &self,
        request: &transport_api_types::OrdinaryAcceptedWorkCancelRequestV1,
    ) -> Result<transport_api_types::OrdinaryAcceptedWorkCancelResponseV1> {
        if !cfg!(target_os = "linux")
            || !matches!(&self.transport, Transport::UnixSocket { path } if path == Path::new("/run/substrate.sock"))
        {
            return Err(anyhow!(
                "ordinary_accepted_cancel_unsupported_configuration"
            ));
        }
        request
            .validate()
            .map_err(|_| anyhow!("ordinary_accepted_cancel_invalid_request"))?;
        #[cfg(target_os = "linux")]
        {
            let bytes = serde_json::to_vec(request)
                .map_err(|_| anyhow!("ordinary_accepted_cancel_invalid_request"))?;
            if bytes.len() > 32768 {
                return Err(anyhow!("ordinary_accepted_cancel_invalid_request"));
            }
            let mut builder =
                hyper_util::client::legacy::Client::builder(hyper_util::rt::TokioExecutor::new());
            builder
                .pool_max_idle_per_host(0)
                .retry_canceled_requests(false);
            let client = builder.build(HostGatewayInstanceConnectorV3);
            let http_request = Request::builder()
                .method(Method::POST)
                .uri("http://localhost/v1/ordinary/work/cancel")
                .header("content-type", "application/json")
                .body(Full::new(Bytes::from(bytes)))
                .map_err(|_| anyhow!("ordinary_accepted_cancel_invalid_request"))?;
            let response = client
                .request(http_request)
                .await
                .map_err(|_| anyhow!("ordinary_accepted_cancel_transport_unknown"))?;
            if !response.status().is_success() {
                return Err(anyhow!("ordinary_accepted_cancel_refused_or_unknown"));
            }
            let bytes = http_body_util::Limited::new(response.into_body(), 32768)
                .collect()
                .await
                .map_err(|_| anyhow!("ordinary_accepted_cancel_invalid_response"))?
                .to_bytes();
            serde_json::from_slice(&bytes)
                .map_err(|_| anyhow!("ordinary_accepted_cancel_invalid_response"))
        }
        #[cfg(not(target_os = "linux"))]
        Err(anyhow!(
            "ordinary_accepted_cancel_unsupported_configuration"
        ))
    }
}

#[cfg(all(test, target_os = "linux"))]
mod ordinary_accepted_cancel_client_tests {
    use super::*;
    fn request_data() -> transport_api_types::OrdinaryAcceptedWorkCancelRequestV1 {
        transport_api_types::OrdinaryAcceptedWorkCancelRequestV1 {
            schema_version: 1,
            authority_store_id: "authority-codec-only".into(),
            orchestration_session_id: "session-codec-only".into(),
            acceptance_record_id: "acceptance-codec-only".into(),
            cancel_request_id: "cancel-codec-only".into(),
            delivery_claim_id: "delivery-codec-only".into(),
            runtime_submission_id: "span-codec-only".into(),
            target_backend_id: "cli:codex-world".into(),
            world_id: "world-codec-only".into(),
            world_generation: 0,
            signal: "INT".into(),
            control: transport_api_types::OrdinaryAcceptedWorkCancelControlV1::CancelWork {},
        }
    }
    #[tokio::test]
    async fn test_ordinary_accepted_cancel_client_refuses_unqualified_transport() {
        let fixture = tempfile::tempdir().unwrap();
        let path = fixture.path().join("unqualified.sock");
        let listener = std::os::unix::net::UnixListener::bind(&path).unwrap();
        listener.set_nonblocking(true).unwrap();
        let request = request_data();
        request.validate().unwrap();
        for client in [
            AgentClient::unix_socket(&path).unwrap(),
            AgentClient::tcp("127.0.0.1", 9).unwrap(),
        ] {
            assert_eq!(
                client
                    .cancel_ordinary_accepted_work(&request)
                    .await
                    .err()
                    .unwrap()
                    .to_string(),
                "ordinary_accepted_cancel_unsupported_configuration"
            );
        }
        assert_eq!(
            listener.accept().unwrap_err().kind(),
            std::io::ErrorKind::WouldBlock
        );
    }
    #[tokio::test]
    async fn test_ordinary_accepted_cancel_client_rejects_invalid_official_request() {
        let mut request = request_data();
        request.signal = "KILL".into();
        let client = AgentClient::unix_socket("/run/substrate.sock").unwrap();
        assert_eq!(
            client
                .cancel_ordinary_accepted_work(&request)
                .await
                .err()
                .unwrap()
                .to_string(),
            "ordinary_accepted_cancel_invalid_request"
        );
    }
}
