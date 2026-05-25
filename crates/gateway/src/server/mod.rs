mod oauth_handlers;
mod openai_compat;
pub mod openai_conformance_test_support;
mod openai_responses;

use crate::auth::codex_auth_context::{
    install_integrated_codex_auth_handoff, CodexIntegratedAuthHandoff,
};
use crate::auth::TokenStore;
use crate::cli::AppConfig;
use crate::core::GatewayRequest;
use crate::launch::{GatewayLaunchContract, GatewayMode, TokenStoreStrategy};
use crate::message_tracing::MessageTracer;
use crate::models::{AnthropicMessagesRequest, RouteType};
use crate::providers::error::ProviderError;
use crate::providers::{AuthType, ProviderRegistry};
use crate::router::Router;
use crate::structured_events::{
    normalized_events_from_provider_response, AnthropicSseNormalizedEventExtractor,
    NormalizedEventSseStream,
};
use axum::{
    body::Body,
    extract::State,
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router as AxumRouter,
};
use chrono::Local;
use futures::stream::TryStreamExt;
use std::env;
#[cfg(unix)]
use std::io::Read;
use std::pin::Pin;
use std::sync::Arc;
use substrate_common::{
    GatewayAuthBundleV1, GATEWAY_AUTH_BUNDLE_BACKEND_API_OPENAI,
    GATEWAY_AUTH_BUNDLE_BACKEND_CLI_CLAUDE_CODE, GATEWAY_AUTH_BUNDLE_BACKEND_CLI_CODEX,
    SUBSTRATE_LLM_AUTH_BUNDLE_FD, SUBSTRATE_LLM_BACKEND_AUTH_API_ANTHROPIC_API_KEY,
    SUBSTRATE_LLM_BACKEND_AUTH_API_OPENAI_API_KEY,
};
use tokio::net::TcpListener;
use tracing::{debug, error, info};

const OPENAI_API_KEY_ENV: &str = "OPENAI_API_KEY";
const ANTHROPIC_API_KEY_ENV: &str = "ANTHROPIC_API_KEY";

#[derive(Debug, Clone)]
pub enum IntegratedGatewayAuthContext {
    CliCodex(CodexIntegratedAuthHandoff),
    ApiKey {
        backend_id: &'static str,
        env_var: &'static str,
        bundle_field: &'static str,
        api_key: String,
    },
}

impl IntegratedGatewayAuthContext {
    pub fn from_launch_mode(mode: GatewayMode) -> anyhow::Result<Option<Self>> {
        match mode {
            GatewayMode::HostOnly => Ok(None),
            GatewayMode::InWorld => Self::from_auth_bundle_env().map(Some),
        }
    }

    fn from_auth_bundle_env() -> anyhow::Result<Self> {
        let bundle = read_gateway_auth_bundle_from_env()?;
        match bundle.backend_id.as_str() {
            GATEWAY_AUTH_BUNDLE_BACKEND_CLI_CODEX => Ok(Self::CliCodex(
                CodexIntegratedAuthHandoff::from_fields(&bundle.fields)?,
            )),
            GATEWAY_AUTH_BUNDLE_BACKEND_CLI_CLAUDE_CODE => Ok(Self::ApiKey {
                backend_id: GATEWAY_AUTH_BUNDLE_BACKEND_CLI_CLAUDE_CODE,
                env_var: ANTHROPIC_API_KEY_ENV,
                bundle_field: SUBSTRATE_LLM_BACKEND_AUTH_API_ANTHROPIC_API_KEY,
                api_key: read_required_bundle_field(
                    &bundle,
                    GATEWAY_AUTH_BUNDLE_BACKEND_CLI_CLAUDE_CODE,
                    SUBSTRATE_LLM_BACKEND_AUTH_API_ANTHROPIC_API_KEY,
                )?,
            }),
            GATEWAY_AUTH_BUNDLE_BACKEND_API_OPENAI => Ok(Self::ApiKey {
                backend_id: GATEWAY_AUTH_BUNDLE_BACKEND_API_OPENAI,
                env_var: OPENAI_API_KEY_ENV,
                bundle_field: SUBSTRATE_LLM_BACKEND_AUTH_API_OPENAI_API_KEY,
                api_key: read_required_bundle_field(
                    &bundle,
                    GATEWAY_AUTH_BUNDLE_BACKEND_API_OPENAI,
                    SUBSTRATE_LLM_BACKEND_AUTH_API_OPENAI_API_KEY,
                )?,
            }),
            other => Err(anyhow::anyhow!(
                "Integrated gateway startup received invalid gateway auth bundle: unsupported gateway auth bundle backend_id '{}'",
                other
            )),
        }
    }

    fn apply_to_config(&self, config: &mut AppConfig) -> anyhow::Result<()> {
        install_integrated_codex_auth_handoff(None)?;

        match self {
            Self::CliCodex(handoff) => {
                install_integrated_codex_auth_handoff(Some(handoff.clone()))?;
                Ok(())
            }
            Self::ApiKey {
                backend_id,
                env_var,
                bundle_field,
                api_key,
            } => overlay_api_key_auth(config, backend_id, env_var, bundle_field, api_key),
        }
    }
}

fn read_required_bundle_field(
    bundle: &GatewayAuthBundleV1,
    backend_id: &str,
    field_name: &str,
) -> anyhow::Result<String> {
    bundle
        .fields
        .get(field_name)
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .ok_or_else(|| {
            anyhow::anyhow!(
                "Integrated gateway startup received invalid gateway auth bundle: gateway auth bundle for '{}' is missing required field '{}'",
                backend_id,
                field_name
            )
        })
}

/// Reloadable components - rebuilt on config reload
pub struct ReloadableState {
    pub config: AppConfig,
    pub router: Router,
    pub provider_registry: Arc<ProviderRegistry>,
}

/// Application state shared across handlers
pub struct AppState {
    /// Reloadable state behind a single lock for atomic updates
    inner: std::sync::RwLock<Arc<ReloadableState>>,

    /// Persistent state - NOT reloaded
    pub token_store: TokenStore,
    pub message_tracer: Arc<MessageTracer>,
}

impl AppState {
    /// Get a snapshot of current reloadable state
    pub fn snapshot(&self) -> Arc<ReloadableState> {
        self.inner.read().unwrap().clone()
    }
}

fn build_app(state: Arc<AppState>) -> AxumRouter {
    AxumRouter::new()
        .route("/v1/messages", post(handle_messages))
        .route("/v1/structured-events", post(handle_structured_events))
        .route("/v1/messages/count_tokens", post(handle_count_tokens))
        .route("/v1/chat/completions", post(handle_openai_chat_completions))
        .route(
            "/v1/responses",
            post(openai_responses::handle_openai_responses),
        )
        .route("/health", get(health_check))
        .route(
            "/api/oauth/authorize",
            post(oauth_handlers::oauth_authorize),
        )
        .route("/api/oauth/exchange", post(oauth_handlers::oauth_exchange))
        .route("/api/oauth/callback", get(oauth_handlers::oauth_callback))
        .route("/auth/callback", get(oauth_handlers::oauth_callback))
        .route("/api/oauth/tokens", get(oauth_handlers::oauth_list_tokens))
        .route(
            "/api/oauth/tokens/delete",
            post(oauth_handlers::oauth_delete_token),
        )
        .route(
            "/api/oauth/tokens/refresh",
            post(oauth_handlers::oauth_refresh_token),
        )
        .with_state(state)
}

fn prepare_startup_config(
    config: &mut AppConfig,
    mode: GatewayMode,
    integrated_auth: Option<&IntegratedGatewayAuthContext>,
) -> anyhow::Result<()> {
    install_integrated_codex_auth_handoff(None)?;

    match mode {
        GatewayMode::HostOnly => {
            if integrated_auth.is_some() {
                anyhow::bail!(
                    "Host-only gateway startup does not accept an integrated auth bundle context"
                );
            }
            config.resolve_env_vars()?;
        }
        GatewayMode::InWorld => {
            let integrated_auth = integrated_auth.ok_or_else(|| {
                anyhow::anyhow!(
                    "Integrated gateway startup requires a startup-owned auth bundle context before provider initialization"
                )
            })?;
            integrated_auth.apply_to_config(config)?;
            config.resolve_env_vars_for_integrated_mode()?;
        }
    }

    Ok(())
}

fn overlay_api_key_auth(
    config: &mut AppConfig,
    backend_id: &str,
    env_var: &str,
    bundle_field: &str,
    api_key: &str,
) -> anyhow::Result<()> {
    let mut applied = 0usize;

    for provider in &mut config.providers {
        if !provider.is_enabled() || provider.auth_type != AuthType::ApiKey {
            continue;
        }

        match provider.api_key.as_deref() {
            Some(value)
                if value == format!("${env_var}") || value == format!("${bundle_field}") =>
            {
                provider.api_key = Some(api_key.to_string());
                applied += 1;
            }
            _ => {}
        }
    }

    if applied == 0 {
        anyhow::bail!(
            "Integrated gateway startup could not apply {} auth bundle: no provider api_key matched ${} or ${}",
            backend_id,
            env_var,
            bundle_field
        );
    }

    Ok(())
}

fn take_auth_bundle_fd_env() -> anyhow::Result<String> {
    match env::var(SUBSTRATE_LLM_AUTH_BUNDLE_FD) {
        Ok(value) => {
            env::remove_var(SUBSTRATE_LLM_AUTH_BUNDLE_FD);
            Ok(value)
        }
        Err(env::VarError::NotPresent) => Err(anyhow::anyhow!(
            "Integrated gateway startup is missing pointer env {}",
            SUBSTRATE_LLM_AUTH_BUNDLE_FD
        )),
        Err(err) => Err(anyhow::anyhow!(
            "Integrated gateway startup could not read pointer env {}: {}",
            SUBSTRATE_LLM_AUTH_BUNDLE_FD,
            err
        )),
    }
}

#[cfg(unix)]
fn read_gateway_auth_bundle_from_env() -> anyhow::Result<GatewayAuthBundleV1> {
    use std::fs::File;
    use std::os::fd::{FromRawFd, OwnedFd, RawFd};

    let raw_fd = take_auth_bundle_fd_env()?;
    let fd = raw_fd.trim().parse::<RawFd>().map_err(|err| {
        anyhow::anyhow!(
            "Integrated gateway startup could not parse {} as a file descriptor: {}",
            SUBSTRATE_LLM_AUTH_BUNDLE_FD,
            err
        )
    })?;

    let mut body = String::new();
    {
        let owned_fd = unsafe { OwnedFd::from_raw_fd(fd) };
        let mut file = File::from(owned_fd);
        file.read_to_string(&mut body).map_err(|err| {
            anyhow::anyhow!(
                "Integrated gateway startup could not read auth bundle from {}: {}",
                SUBSTRATE_LLM_AUTH_BUNDLE_FD,
                err
            )
        })?;
    }

    let bundle: GatewayAuthBundleV1 = serde_json::from_str(&body).map_err(|err| {
        anyhow::anyhow!(
            "Integrated gateway startup received malformed gateway auth bundle JSON: {}",
            err
        )
    })?;

    bundle.validate().map_err(|err| {
        anyhow::anyhow!(
            "Integrated gateway startup received invalid gateway auth bundle: {}",
            err
        )
    })?;

    Ok(bundle)
}

#[cfg(not(unix))]
fn read_gateway_auth_bundle_from_env() -> anyhow::Result<GatewayAuthBundleV1> {
    let _ = take_auth_bundle_fd_env();
    anyhow::bail!(
        "Integrated gateway startup via {} is unsupported on this platform",
        SUBSTRATE_LLM_AUTH_BUNDLE_FD
    )
}

const RECENT_REQUESTS_WINDOW: usize = 20;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FailureClass {
    Auth,
    Url,
    Deployment,
    Route,
    TransportDrift,
}

impl FailureClass {
    fn as_str(self) -> &'static str {
        match self {
            FailureClass::Auth => "auth",
            FailureClass::Url => "url",
            FailureClass::Deployment => "deployment",
            FailureClass::Route => "route",
            FailureClass::TransportDrift => "transport_drift",
        }
    }
}

fn classify_provider_error(error: &ProviderError) -> FailureClass {
    match error {
        ProviderError::AuthError(_) => FailureClass::Auth,
        ProviderError::ModelNotSupported(_) => FailureClass::Deployment,
        ProviderError::ConfigError(message) => {
            let lowered = message.to_ascii_lowercase();
            if lowered.contains("url") || lowered.contains("base_url") {
                FailureClass::Url
            } else if lowered.contains("codex route")
                || lowered.contains("authoritative provenance")
                || lowered.contains("prior function_call")
            {
                FailureClass::Route
            } else {
                FailureClass::TransportDrift
            }
        }
        ProviderError::HttpError(http_error) => {
            if http_error.is_timeout() || http_error.is_connect() || http_error.is_builder() {
                FailureClass::Url
            } else {
                FailureClass::TransportDrift
            }
        }
        ProviderError::SerializationError(_) => FailureClass::TransportDrift,
        ProviderError::ApiError { status, message } => {
            let lowered = message.to_ascii_lowercase();

            if *status == 401
                || *status == 403
                || lowered.contains("unauthorized")
                || lowered.contains("authentication")
                || lowered.contains("api key")
                || lowered.contains("bearer")
            {
                FailureClass::Auth
            } else if lowered.contains("deployment")
                || lowered.contains("deployment not found")
                || (lowered.contains("model") && lowered.contains("not found"))
                || lowered.contains("model not supported")
            {
                FailureClass::Deployment
            } else if *status == 404 {
                FailureClass::Url
            } else {
                FailureClass::TransportDrift
            }
        }
    }
}

fn public_error_message(class: FailureClass) -> &'static str {
    match class {
        FailureClass::Auth => "Authentication failed",
        FailureClass::Url => "Request target failed",
        FailureClass::Deployment => "Deployment mapping failed",
        FailureClass::Route => "Route selection failed",
        FailureClass::TransportDrift => "Transport behavior drifted from the operator contract",
    }
}

fn failure_class_rank(class: FailureClass) -> u8 {
    match class {
        FailureClass::TransportDrift => 0,
        FailureClass::Deployment => 1,
        FailureClass::Url => 2,
        FailureClass::Auth => 3,
        FailureClass::Route => 4,
    }
}

fn prefer_failure_class(current: Option<FailureClass>, next: FailureClass) -> Option<FailureClass> {
    match current {
        None => Some(next),
        Some(existing) if failure_class_rank(next) > failure_class_rank(existing) => Some(next),
        Some(existing) => Some(existing),
    }
}

/// Write routing information to file for statusline script
fn write_routing_info(model: &str, provider: &str, route_type: &RouteType) {
    if let Some(home) = dirs::home_dir() {
        let file_path = home.join(".substrate-gateway/last_routing.json");

        // Read existing recent requests history
        let mut recent: Vec<String> = Vec::new();
        if let Ok(existing_content) = std::fs::read_to_string(&file_path) {
            if let Ok(existing) = serde_json::from_str::<serde_json::Value>(&existing_content) {
                if let Some(items) = existing.get("recent").and_then(|t| t.as_array()) {
                    for item in items {
                        if let Some(entry) = item.as_str() {
                            recent.push(entry.to_string());
                        }
                    }
                }
            }
        }

        // Add current model/provider to recent
        let current_entry = format!("{}@{}", model, provider);
        recent.insert(0, current_entry);
        recent.truncate(RECENT_REQUESTS_WINDOW);

        // Create routing info
        let routing_info = serde_json::json!({
            "model": model,
            "provider": provider,
            "route_type": route_type.to_string(),
            "timestamp": Local::now().format("%H:%M:%S").to_string(),
            "recent": recent
        });

        if let Ok(json) = serde_json::to_string(&routing_info) {
            if let Err(e) = std::fs::write(file_path, json) {
                tracing::debug!("Failed to write routing info: {}", e);
            }
        } else {
            tracing::debug!("Failed to serialize routing info");
        }
    }
}

/// Start the HTTP server
pub async fn start_server(
    mut config: AppConfig,
    launch: GatewayLaunchContract,
    integrated_auth: Option<IntegratedGatewayAuthContext>,
) -> anyhow::Result<()> {
    let GatewayLaunchContract {
        mode, token_store, ..
    } = launch;

    prepare_startup_config(&mut config, mode, integrated_auth.as_ref())?;
    let router = Router::new(config.clone());

    // Initialize OAuth token store FIRST (needed by provider registry)
    let token_store = match token_store {
        TokenStoreStrategy::Persistent(path) => TokenStore::new(path),
        TokenStoreStrategy::Disabled => Ok(TokenStore::disabled()),
    }
    .map_err(|e| anyhow::anyhow!("Failed to initialize token store: {}", e))?;

    let existing_tokens = token_store.list_providers();
    if !existing_tokens.is_empty() {
        info!(
            "🔐 Loaded {} OAuth tokens from storage",
            existing_tokens.len()
        );
    }

    // Initialize provider registry from config (with token store and model mappings)
    let provider_registry = Arc::new(
        ProviderRegistry::from_configs_with_models_and_mode(
            &config.providers,
            Some(token_store.clone()),
            &config.models,
            mode,
        )
        .map_err(|e| anyhow::anyhow!("Failed to initialize provider registry: {}", e))?,
    );

    info!(
        "📦 Loaded {} providers with {} models",
        provider_registry.list_providers().len(),
        provider_registry.list_models().len()
    );

    // Initialize message tracer
    let message_tracer = Arc::new(MessageTracer::new(config.server.tracing.clone()));

    // Build reloadable state
    let reloadable = Arc::new(ReloadableState {
        config: config.clone(),
        router,
        provider_registry,
    });

    let state = Arc::new(AppState {
        inner: std::sync::RwLock::new(reloadable),
        token_store,
        message_tracer,
    });

    // Clone state before moving it
    let oauth_state = state.clone();
    let app = build_app(state);

    // Bind to main address
    let addr = format!("{}:{}", config.server.host, config.server.port);
    let listener = TcpListener::bind(&addr).await?;

    info!("🚀 Server listening on {}", addr);

    // Start OAuth callback server on port 1455 (required for OpenAI Codex)
    // This is necessary because OpenAI's OAuth app only allows localhost:1455/auth/callback
    tokio::spawn(async move {
        let oauth_callback_app = AxumRouter::new()
            .route("/auth/callback", get(oauth_handlers::oauth_callback))
            .with_state(oauth_state);

        let oauth_addr = "127.0.0.1:1455";
        match TcpListener::bind(oauth_addr).await {
            Ok(oauth_listener) => {
                info!("🔐 OAuth callback server listening on {}", oauth_addr);
                if let Err(e) = axum::serve(oauth_listener, oauth_callback_app).await {
                    error!("OAuth callback server error: {}", e);
                }
            }
            Err(e) => {
                // Don't fail if port 1455 is already in use - just warn
                error!(
                    "⚠️  Failed to bind OAuth callback server on {}: {}",
                    oauth_addr, e
                );
                error!("⚠️  OpenAI Codex OAuth will not work. Port 1455 must be available.");
            }
        }
    });

    // Start main server
    axum::serve(listener, app).await?;

    Ok(())
}

struct StructuredEventTracingStream {
    inner: Pin<Box<dyn futures::stream::Stream<Item = Result<bytes::Bytes, ProviderError>> + Send>>,
    extractor: AnthropicSseNormalizedEventExtractor,
    tracer: Arc<MessageTracer>,
    trace_id: String,
}

impl StructuredEventTracingStream {
    fn new(
        inner: Pin<
            Box<dyn futures::stream::Stream<Item = Result<bytes::Bytes, ProviderError>> + Send>,
        >,
        tracer: Arc<MessageTracer>,
        trace_id: String,
    ) -> Self {
        Self {
            inner,
            extractor: AnthropicSseNormalizedEventExtractor::new(),
            tracer,
            trace_id,
        }
    }
}

impl futures::stream::Stream for StructuredEventTracingStream {
    type Item = Result<bytes::Bytes, ProviderError>;

    fn poll_next(
        mut self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        match self.inner.as_mut().poll_next(cx) {
            std::task::Poll::Ready(Some(Ok(bytes))) => {
                if !self.trace_id.is_empty() {
                    let events = self.extractor.push_bytes(&bytes);
                    for event in events {
                        self.tracer.trace_event(&self.trace_id, &event);
                    }
                }
                std::task::Poll::Ready(Some(Ok(bytes)))
            }
            std::task::Poll::Ready(None) => {
                if !self.trace_id.is_empty() {
                    let events = self.extractor.finalize();
                    for event in events {
                        self.tracer.trace_event(&self.trace_id, &event);
                    }
                }
                std::task::Poll::Ready(None)
            }
            other => other,
        }
    }
}

/// Health check endpoint
async fn health_check() -> impl IntoResponse {
    Json(serde_json::json!({
        "status": "ok",
        "service": "substrate-gateway"
    }))
}

/// Handle /v1/chat/completions requests (OpenAI-compatible endpoint)
///
/// Note: This endpoint has limited functionality. The primary use case for this proxy
/// is Claude Code (Anthropic client) connecting via /v1/messages.
async fn handle_openai_chat_completions(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(request_json): Json<serde_json::Value>,
) -> Result<Response, AppError> {
    let openai_request: openai_compat::OpenAIRequest = serde_json::from_value(request_json)
        .map_err(|e| AppError::Routing(format!("Invalid request format: {}", e)))?;
    let model = openai_request.model.clone();
    let include_usage = openai_request
        .stream_options
        .as_ref()
        .and_then(|o| o.include_usage)
        .unwrap_or(false);
    let start_time = std::time::Instant::now();

    // Get snapshot of reloadable state
    let inner = state.snapshot();

    // 1. Transform OpenAI request to gateway core format
    let mut gateway_request = openai_compat::transform_openai_to_gateway_request(openai_request)
        .map_err(AppError::Routing)?;

    // 2. Route the request (may modify system prompt to remove CCM-SUBAGENT-MODEL tag)
    let decision = inner
        .router
        .route(&mut gateway_request)
        .map_err(|e| AppError::Routing(e.to_string()))?;
    let forced_provider = headers
        .get("x-provider")
        .and_then(|v| v.to_str().ok())
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string());

    // 3. Try model mappings with fallback (1:N mapping)
    if let Some(model_config) = inner
        .config
        .models
        .iter()
        .find(|m| m.name.eq_ignore_ascii_case(&decision.model_name))
    {
        if let Some(ref provider_name) = forced_provider {
            info!(
                "🎯 Using forced provider from X-Provider header: {}",
                provider_name
            );
        }

        // Sort mappings by priority (or filter by forced provider)
        let mut sorted_mappings = model_config.mappings.clone();

        if let Some(ref provider_name) = forced_provider {
            // Filter to only the specified provider
            sorted_mappings.retain(|m| m.provider == *provider_name);
            if sorted_mappings.is_empty() {
                return Err(AppError::Routing(format!(
                    "Provider '{}' not found in mappings for model '{}'",
                    provider_name, decision.model_name
                )));
            }
        } else {
            // Use priority ordering
            sorted_mappings.sort_by_key(|m| m.priority);
        }

        let mut last_failure_class: Option<FailureClass> = None;

        // Try each mapping in priority order (or just the forced one)
        for (idx, mapping) in sorted_mappings.iter().enumerate() {
            // Try to get provider from registry
            if let Some(provider) = inner.provider_registry.get_provider(&mapping.provider) {
                // Build retry indicator (only show if not first attempt)
                let retry_info = if idx > 0 {
                    format!(" [{}/{}]", idx + 1, sorted_mappings.len())
                } else {
                    String::new()
                };

                // Build route type display (include matched prompt snippet if available)
                let route_type_display = match &decision.matched_prompt {
                    Some(matched) => {
                        // Trim prompt to max 30 chars
                        let trimmed = if matched.len() > 30 {
                            format!("{}...", &matched[..27])
                        } else {
                            matched.clone()
                        };
                        format!("{}:{}", decision.route_type, trimmed)
                    }
                    None => decision.route_type.to_string(),
                };

                info!(
                    "[{:<15}:sync] {:<25} → {}/{}{}",
                    route_type_display, model, mapping.provider, mapping.actual_model, retry_info
                );

                // Update model to actual model name
                gateway_request.model = mapping.actual_model.clone();

                // Inject continuation prompt if configured (skip for background tasks)
                if mapping.inject_continuation_prompt
                    && decision.route_type != RouteType::Background
                {
                    if let Some(last_msg) = gateway_request.messages.last_mut() {
                        if should_inject_continuation(last_msg) {
                            info!(
                                "💉 Injecting continuation prompt for model: {}",
                                mapping.actual_model
                            );
                            inject_continuation_text(last_msg);
                        }
                    }
                }

                // Write routing info immediately on first attempt
                if idx == 0 {
                    write_routing_info(
                        &mapping.actual_model,
                        &mapping.provider,
                        &decision.route_type,
                    );
                }

                if gateway_request.stream == Some(true) {
                    match provider.send_message_stream(gateway_request.clone()).await {
                        Ok(stream_response) => {
                            let openai_stream =
                                openai_compat::OpenAIChatCompletionsChunkStream::new(
                                    stream_response.stream,
                                    model.clone(),
                                    include_usage,
                                );

                            let body_stream = openai_stream.map_err(|e| {
                                error!("Stream error: {}", e);
                                std::io::Error::other(e.to_string())
                            });
                            let body = Body::from_stream(body_stream);

                            let response = Response::builder()
                                .status(200)
                                .header("Content-Type", "text/event-stream")
                                .header("Cache-Control", "no-cache")
                                .header("Connection", "keep-alive")
                                .body(body)
                                .unwrap();

                            return Ok(response);
                        }
                        Err(e) => {
                            last_failure_class = prefer_failure_class(
                                last_failure_class,
                                classify_provider_error(&e),
                            );
                            info!(
                                "⚠️ Provider {} streaming failed: {}, trying next fallback",
                                mapping.provider, e
                            );
                            continue;
                        }
                    }
                } else {
                    match provider.send_message(gateway_request.clone()).await {
                        Ok(anthropic_response) => {
                            // Calculate and log metrics
                            let latency_ms = start_time.elapsed().as_millis() as u64;
                            let tok_s = (anthropic_response.usage.output_tokens as f32 * 1000.0)
                                / latency_ms as f32;
                            info!(
                                "📊 {}@{} {}ms {:.0}t/s {}tok",
                                mapping.actual_model,
                                mapping.provider,
                                latency_ms,
                                tok_s,
                                anthropic_response.usage.output_tokens
                            );

                            // Write routing info on fallback success (idx==0 already wrote above)
                            if idx > 0 {
                                write_routing_info(
                                    &mapping.actual_model,
                                    &mapping.provider,
                                    &decision.route_type,
                                );
                            }

                            // Transform Anthropic response to OpenAI format
                            let openai_response =
                                openai_compat::transform_gateway_response_to_openai(
                                    anthropic_response,
                                    model.clone(),
                                );

                            return Ok(Json(openai_response).into_response());
                        }
                        Err(e) => {
                            last_failure_class = prefer_failure_class(
                                last_failure_class,
                                classify_provider_error(&e),
                            );
                            info!(
                                "⚠️ Provider {} failed: {}, trying next fallback",
                                mapping.provider, e
                            );
                            continue;
                        }
                    }
                }
            } else {
                last_failure_class =
                    prefer_failure_class(last_failure_class, FailureClass::Deployment);
                info!(
                    "⚠️ Provider {} not found in registry, trying next fallback",
                    mapping.provider
                );
                continue;
            }
        }

        error!(
            "❌ All provider mappings failed for model: {}",
            decision.model_name
        );
        Err(AppError::provider_class(
            last_failure_class.unwrap_or(FailureClass::Deployment),
            format!(
                "All {} provider mappings failed for the routed model",
                sorted_mappings.len()
            ),
        ))
    } else {
        // No model mapping found, try direct provider registry lookup (backward compatibility)
        let direct_provider = if let Some(ref provider_name) = forced_provider {
            let provider = inner
                .provider_registry
                .get_provider(provider_name)
                .ok_or_else(|| {
                    AppError::Routing(format!(
                        "Provider '{}' not found for model '{}'",
                        provider_name, decision.model_name
                    ))
                })?;

            if !provider.supports_model(&decision.model_name) {
                return Err(AppError::Routing(format!(
                    "Provider '{}' does not support model '{}'",
                    provider_name, decision.model_name
                )));
            }

            Some((provider_name.as_str(), provider))
        } else {
            inner
                .provider_registry
                .get_provider_for_model(&decision.model_name)
                .ok()
                .map(|provider| ("direct", provider))
        };

        if let Some((provider_name, provider)) = direct_provider {
            info!(
                "📦 Using provider from registry (direct lookup): {}/{}",
                provider_name, decision.model_name
            );

            // Update model to routed model
            gateway_request.model = decision.model_name.clone();

            if gateway_request.stream == Some(true) {
                let stream_response = provider
                    .send_message_stream(gateway_request)
                    .await
                    .map_err(|e| {
                        AppError::provider_class(
                            classify_provider_error(&e),
                            "Provider streaming request failed",
                        )
                    })?;

                let openai_stream = openai_compat::OpenAIChatCompletionsChunkStream::new(
                    stream_response.stream,
                    model,
                    include_usage,
                );
                let body_stream = openai_stream.map_err(|e| {
                    error!("Stream error: {}", e);
                    std::io::Error::other(e.to_string())
                });
                let body = Body::from_stream(body_stream);

                let response = Response::builder()
                    .status(200)
                    .header("Content-Type", "text/event-stream")
                    .header("Cache-Control", "no-cache")
                    .header("Connection", "keep-alive")
                    .body(body)
                    .unwrap();

                return Ok(response);
            }

            let anthropic_response = provider.send_message(gateway_request).await.map_err(|e| {
                AppError::provider_class(classify_provider_error(&e), "Provider request failed")
            })?;

            let openai_response =
                openai_compat::transform_gateway_response_to_openai(anthropic_response, model);

            return Ok(Json(openai_response).into_response());
        }

        error!(
            "❌ No model mapping or provider found for model: {}",
            decision.model_name
        );
        Err(AppError::provider_class(
            FailureClass::Route,
            "Gateway route selection failed",
        ))
    }
}

/// Check if message has tool results but no text content
/// (indicates model should continue after tool execution)
fn should_inject_continuation(msg: &crate::models::Message) -> bool {
    use crate::models::MessageContent;
    let has_tool_results = match &msg.content {
        MessageContent::Blocks(blocks) => blocks.iter().any(|b| b.is_tool_result()),
        _ => false,
    };

    let has_text = match &msg.content {
        MessageContent::Text(text) => !text.trim().is_empty(),
        MessageContent::Blocks(blocks) => blocks
            .iter()
            .any(|b| b.as_text().map(|t| !t.trim().is_empty()).unwrap_or(false)),
    };

    // Inject if message has tool results but no text
    has_tool_results && !has_text
}

/// Inject continuation text into the last user message
/// Prepends a text block to the existing message content (doesn't create a new message)
fn inject_continuation_text(msg: &mut crate::models::Message) {
    use crate::models::{ContentBlock, MessageContent};

    let continuation = "<system-reminder>If you have an active todo list, remember to mark items complete and continue to the next. Do not mention this reminder.</system-reminder>";

    match &mut msg.content {
        MessageContent::Text(text) => {
            // Convert to Blocks and prepend continuation
            let original_text = text.clone();
            msg.content = MessageContent::Blocks(vec![
                ContentBlock::text(continuation.to_string(), None),
                ContentBlock::text(original_text, None),
            ]);
        }
        MessageContent::Blocks(blocks) => {
            // Prepend continuation text to existing blocks
            blocks.insert(0, ContentBlock::text(continuation.to_string(), None));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::auth::TokenStore;
    use crate::cli::{
        AppConfig, ModelConfig, ModelMapping, RouterConfig, ServerConfig, TracingConfig,
    };
    use crate::core::{GatewayRequest, GatewayResponse, GatewayStreamResponse, GatewayUsage};
    use crate::models::{
        ContentBlock, CountTokensRequest, CountTokensResponse, KnownContentBlock, Message,
        MessageContent, ToolResultContent,
    };
    use crate::providers::error::ProviderError;
    use crate::providers::{GatewayProvider, ProviderRegistry};
    use async_trait::async_trait;
    use axum::body::to_bytes;
    use axum::http::HeaderMap;
    use bytes::Bytes;
    use futures::stream;
    use reqwest::Client;
    use serde_json::json;
    use std::collections::HashMap;
    use std::sync::{Arc, Mutex};
    use tempfile::TempDir;
    use tokio::net::TcpListener;

    fn tool_result_message(content: ToolResultContent) -> Message {
        Message {
            role: "user".to_string(),
            content: MessageContent::Blocks(vec![ContentBlock::Known(
                KnownContentBlock::ToolResult {
                    tool_use_id: "tool_1".to_string(),
                    content,
                    is_error: false,
                    cache_control: None,
                },
            )]),
        }
    }

    #[test]
    fn should_inject_continuation_for_tool_result_only_message() {
        let msg = tool_result_message(ToolResultContent::Text("done".to_string()));
        assert!(should_inject_continuation(&msg));
    }

    #[test]
    fn should_not_inject_continuation_when_tool_result_turn_already_has_text() {
        let msg = Message {
            role: "user".to_string(),
            content: MessageContent::Blocks(vec![
                ContentBlock::Known(KnownContentBlock::ToolResult {
                    tool_use_id: "tool_1".to_string(),
                    content: ToolResultContent::Text("done".to_string()),
                    is_error: false,
                    cache_control: None,
                }),
                ContentBlock::text("keep going".to_string(), None),
            ]),
        };

        assert!(!should_inject_continuation(&msg));
    }

    #[test]
    fn inject_continuation_text_prepends_the_internal_reminder() {
        let mut msg = Message {
            role: "user".to_string(),
            content: MessageContent::Text("continue".to_string()),
        };

        inject_continuation_text(&mut msg);

        match msg.content {
            MessageContent::Blocks(blocks) => {
                assert_eq!(blocks.len(), 2);
                assert_eq!(
                    blocks[0].as_text(),
                    Some("<system-reminder>If you have an active todo list, remember to mark items complete and continue to the next. Do not mention this reminder.</system-reminder>")
                );
                assert_eq!(blocks[1].as_text(), Some("continue"));
            }
            _ => panic!("continuation injection should convert text into blocks"),
        }
    }

    #[test]
    fn inject_continuation_text_prepends_to_existing_blocks() {
        let mut msg = tool_result_message(ToolResultContent::Text("done".to_string()));

        inject_continuation_text(&mut msg);

        match msg.content {
            MessageContent::Blocks(blocks) => {
                assert_eq!(blocks.len(), 2);
                assert_eq!(
                    blocks[0].as_text(),
                    Some("<system-reminder>If you have an active todo list, remember to mark items complete and continue to the next. Do not mention this reminder.</system-reminder>")
                );
                assert_eq!(blocks[1].as_text(), None);
            }
            _ => panic!("continuation injection should keep blocks"),
        }
    }

    struct StubProvider {
        response: GatewayResponse,
        stream_chunks: Vec<String>,
        captured_requests: Arc<Mutex<Vec<GatewayRequest>>>,
    }

    impl StubProvider {
        fn new(response: GatewayResponse, stream_chunks: Vec<String>) -> Self {
            Self {
                response,
                stream_chunks,
                captured_requests: Arc::new(Mutex::new(Vec::new())),
            }
        }

        fn captured_requests(&self) -> Arc<Mutex<Vec<GatewayRequest>>> {
            self.captured_requests.clone()
        }
    }

    #[async_trait]
    impl GatewayProvider for StubProvider {
        async fn send_message(
            &self,
            request: GatewayRequest,
        ) -> Result<GatewayResponse, ProviderError> {
            self.captured_requests.lock().unwrap().push(request);
            Ok(self.response.clone())
        }

        async fn send_message_stream(
            &self,
            request: GatewayRequest,
        ) -> Result<GatewayStreamResponse, ProviderError> {
            self.captured_requests.lock().unwrap().push(request);
            let items = self
                .stream_chunks
                .clone()
                .into_iter()
                .map(|chunk| Ok(Bytes::from(chunk)));

            Ok(GatewayStreamResponse {
                stream: Box::pin(stream::iter(items)),
                headers: HashMap::new(),
            })
        }

        async fn count_tokens(
            &self,
            _request: CountTokensRequest,
        ) -> Result<CountTokensResponse, ProviderError> {
            Ok(CountTokensResponse { input_tokens: 42 })
        }

        fn supports_model(&self, _model: &str) -> bool {
            true
        }
    }

    struct FallbackStubProvider {
        failures: Arc<Mutex<Vec<ProviderError>>>,
        captured_requests: Arc<Mutex<Vec<GatewayRequest>>>,
    }

    impl FallbackStubProvider {
        fn new(failures: Vec<ProviderError>) -> Self {
            Self {
                failures: Arc::new(Mutex::new(failures)),
                captured_requests: Arc::new(Mutex::new(Vec::new())),
            }
        }

        fn captured_requests(&self) -> Arc<Mutex<Vec<GatewayRequest>>> {
            self.captured_requests.clone()
        }
    }

    #[async_trait]
    impl GatewayProvider for FallbackStubProvider {
        async fn send_message(
            &self,
            request: GatewayRequest,
        ) -> Result<GatewayResponse, ProviderError> {
            self.captured_requests.lock().unwrap().push(request);
            let mut failures = self.failures.lock().unwrap();
            if failures.is_empty() {
                return Err(ProviderError::AuthError("exhausted failures".to_string()));
            }

            Err(failures.remove(0))
        }

        async fn send_message_stream(
            &self,
            request: GatewayRequest,
        ) -> Result<GatewayStreamResponse, ProviderError> {
            self.captured_requests.lock().unwrap().push(request);
            Err(ProviderError::AuthError(format!(
                "streaming not supported for {}",
                self.captured_requests.lock().unwrap().len()
            )))
        }

        async fn count_tokens(
            &self,
            _request: CountTokensRequest,
        ) -> Result<CountTokensResponse, ProviderError> {
            Err(ProviderError::AuthError(
                "count_tokens not supported".to_string(),
            ))
        }

        fn supports_model(&self, _model: &str) -> bool {
            true
        }
    }

    #[test]
    fn failure_classification_covers_operator_taxonomy() {
        assert_eq!(
            classify_provider_error(&ProviderError::AuthError("bad key".to_string())),
            FailureClass::Auth
        );
        assert_eq!(
            classify_provider_error(&ProviderError::ConfigError(
                "base_url missing or malformed".to_string()
            )),
            FailureClass::Url
        );
        assert_eq!(
            classify_provider_error(&ProviderError::ModelNotSupported(
                "kimi-k2-thinking-deployment".to_string()
            )),
            FailureClass::Deployment
        );
        assert_eq!(
            classify_provider_error(&ProviderError::ApiError {
                status: 400,
                message: "api-version mismatch".to_string(),
            }),
            FailureClass::TransportDrift
        );
        assert_eq!(
            prefer_failure_class(Some(FailureClass::Deployment), FailureClass::Auth),
            Some(FailureClass::Auth)
        );
        assert_eq!(
            prefer_failure_class(Some(FailureClass::Auth), FailureClass::Deployment),
            Some(FailureClass::Auth)
        );
        assert_eq!(
            prefer_failure_class(Some(FailureClass::TransportDrift), FailureClass::Deployment),
            Some(FailureClass::Deployment)
        );
    }

    #[tokio::test]
    async fn app_error_response_is_redacted_and_classified() {
        let cases = [
            (FailureClass::Auth, "auth", "Authentication failed"),
            (FailureClass::Url, "url", "Request target failed"),
            (
                FailureClass::Deployment,
                "deployment",
                "Deployment mapping failed",
            ),
            (
                FailureClass::TransportDrift,
                "transport_drift",
                "Transport behavior drifted from the operator contract",
            ),
        ];

        for (class, expected_class, expected_message) in cases {
            let response = AppError::provider_class(
                class,
                "provider secret should never appear in the response",
            )
            .into_response();

            assert_eq!(response.status(), StatusCode::BAD_GATEWAY);

            let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
            let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

            assert_eq!(json["error"]["class"], expected_class);
            assert_eq!(json["error"]["type"], "error");
            assert_eq!(json["error"]["message"], expected_message);
            assert!(!body
                .windows("provider secret".len())
                .any(|w| w == b"provider secret"));
        }
    }

    #[tokio::test]
    async fn app_error_route_responses_use_route_class() {
        let response = AppError::Routing("missing routed model".to_string()).into_response();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

        assert_eq!(json["error"]["class"], "route");
        assert_eq!(json["error"]["message"], "Route selection failed");
    }

    #[tokio::test]
    async fn handle_messages_preserves_best_fallback_failure_class() {
        let temp_dir = TempDir::new().unwrap();
        let provider = FallbackStubProvider::new(vec![ProviderError::AuthError(
            "invalid credential".to_string(),
        )]);
        let captured_requests = provider.captured_requests();
        let mut registry = ProviderRegistry::new();
        registry.insert_provider_for_tests("test-provider", Box::new(provider));

        let config = AppConfig {
            server: ServerConfig {
                port: 13456,
                host: "127.0.0.1".to_string(),
                api_key: None,
                log_level: "info".to_string(),
                timeouts: Default::default(),
                tracing: TracingConfig {
                    enabled: false,
                    path: temp_dir.path().join("trace.jsonl").display().to_string(),
                    omit_system_prompt: true,
                },
            },
            router: RouterConfig {
                default: "gateway-default".to_string(),
                background: None,
                think: Some("gateway-think".to_string()),
                websearch: None,
                auto_map_regex: Some("^claude-".to_string()),
                background_regex: None,
                prompt_rules: vec![],
            },
            providers: vec![],
            models: vec![ModelConfig {
                name: "gateway-default".to_string(),
                mappings: vec![
                    ModelMapping {
                        priority: 1,
                        provider: "missing-provider".to_string(),
                        actual_model: "kimi-k2-actual".to_string(),
                        inject_continuation_prompt: false,
                    },
                    ModelMapping {
                        priority: 2,
                        provider: "test-provider".to_string(),
                        actual_model: "kimi-k2-actual".to_string(),
                        inject_continuation_prompt: false,
                    },
                ],
            }],
        };

        let reloadable = Arc::new(ReloadableState {
            router: Router::new(config.clone()),
            provider_registry: Arc::new(registry),
            config: config.clone(),
        });

        let token_store = TokenStore::new(temp_dir.path().join("oauth_tokens.json")).unwrap();
        let state = Arc::new(AppState {
            inner: std::sync::RwLock::new(reloadable),
            token_store,
            message_tracer: Arc::new(MessageTracer::new(config.server.tracing.clone())),
        });

        let request = json!({
            "model": "claude-sonnet-4-5",
            "messages": [
                {
                    "role": "user",
                    "content": "Smoke the fallback class handling."
                }
            ],
            "max_tokens": 128
        });

        let response = match handle_messages(State(state), HeaderMap::new(), Json(request)).await {
            Ok(response) => response,
            Err(err) => err.into_response(),
        };

        assert_eq!(response.status(), StatusCode::BAD_GATEWAY);

        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

        assert_eq!(json["error"]["class"], "auth");
        assert_eq!(json["error"]["message"], "Authentication failed");
        assert!(!body
            .windows("invalid credential".len())
            .any(|w| w == b"invalid credential"));

        let requests = captured_requests.lock().unwrap();
        assert_eq!(requests.len(), 1);
        assert_eq!(requests[0].model, "kimi-k2-actual");
    }

    fn create_test_state(
        temp_dir: &TempDir,
        provider: StubProvider,
        inject_continuation_prompt: bool,
    ) -> (Arc<AppState>, Arc<Mutex<Vec<GatewayRequest>>>) {
        let captured_requests = provider.captured_requests();
        let mut registry = ProviderRegistry::new();
        registry.insert_provider_for_tests("test-provider", Box::new(provider));

        let config = AppConfig {
            server: ServerConfig {
                port: 13456,
                host: "127.0.0.1".to_string(),
                api_key: None,
                log_level: "info".to_string(),
                timeouts: Default::default(),
                tracing: TracingConfig {
                    enabled: false,
                    path: temp_dir.path().join("trace.jsonl").display().to_string(),
                    omit_system_prompt: true,
                },
            },
            router: RouterConfig {
                default: "gateway-default".to_string(),
                background: None,
                think: Some("gateway-think".to_string()),
                websearch: None,
                auto_map_regex: Some("^claude-".to_string()),
                background_regex: None,
                prompt_rules: vec![],
            },
            providers: vec![],
            models: vec![
                ModelConfig {
                    name: "gateway-default".to_string(),
                    mappings: vec![ModelMapping {
                        priority: 1,
                        provider: "test-provider".to_string(),
                        actual_model: "kimi-k2-actual".to_string(),
                        inject_continuation_prompt,
                    }],
                },
                ModelConfig {
                    name: "gateway-think".to_string(),
                    mappings: vec![ModelMapping {
                        priority: 1,
                        provider: "test-provider".to_string(),
                        actual_model: "kimi-k2-thinking-actual".to_string(),
                        inject_continuation_prompt: false,
                    }],
                },
            ],
        };

        let reloadable = Arc::new(ReloadableState {
            router: Router::new(config.clone()),
            provider_registry: Arc::new(registry),
            config: config.clone(),
        });

        let token_store = TokenStore::new(temp_dir.path().join("oauth_tokens.json")).unwrap();
        let state = Arc::new(AppState {
            inner: std::sync::RwLock::new(reloadable),
            token_store,
            message_tracer: Arc::new(MessageTracer::new(config.server.tracing.clone())),
        });

        (state, captured_requests)
    }

    #[tokio::test]
    async fn admin_ui_routes_are_not_registered() {
        let temp_dir = TempDir::new().unwrap();
        let provider = StubProvider::new(
            final_only_response("unused", "kimi-k2-actual", "end_turn"),
            vec![],
        );
        let (state, _) = create_test_state(&temp_dir, provider, false);
        let app = build_app(state);
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let server = tokio::spawn(async move {
            axum::serve(listener, app).await.unwrap();
        });
        let client = Client::new();

        for (method, uri) in [
            ("GET", "/"),
            ("GET", "/api/config/json"),
            ("POST", "/api/config/json"),
            ("POST", "/api/reload"),
        ] {
            let url = format!("http://{}{}", addr, uri);
            let response = match method {
                "GET" => client.get(&url).send().await.unwrap(),
                "POST" => client.post(&url).send().await.unwrap(),
                _ => unreachable!("unexpected method"),
            };
            assert_eq!(
                response.status(),
                StatusCode::NOT_FOUND,
                "expected {method} {uri} to be absent",
            );
        }

        server.abort();
    }

    #[tokio::test]
    async fn health_and_oauth_routes_remain_registered() {
        let temp_dir = TempDir::new().unwrap();
        let provider = StubProvider::new(
            final_only_response("unused", "kimi-k2-actual", "end_turn"),
            vec![],
        );
        let (state, _) = create_test_state(&temp_dir, provider, false);
        let app = build_app(state);
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let server = tokio::spawn(async move {
            axum::serve(listener, app).await.unwrap();
        });
        let client = Client::new();

        let health_response = client
            .get(format!("http://{}/health", addr))
            .send()
            .await
            .unwrap();
        assert_eq!(health_response.status(), StatusCode::OK);

        let tokens_response = client
            .get(format!("http://{}/api/oauth/tokens", addr))
            .send()
            .await
            .unwrap();
        assert_eq!(tokens_response.status(), StatusCode::OK);

        let callback_response = client
            .get(format!("http://{}/api/oauth/callback?code=test-code", addr))
            .send()
            .await
            .unwrap();
        assert_eq!(callback_response.status(), StatusCode::OK);

        server.abort();
    }

    fn final_only_response(text: &str, model: &str, stop_reason: &str) -> GatewayResponse {
        GatewayResponse {
            id: "msg_test".to_string(),
            r#type: "message".to_string(),
            role: "assistant".to_string(),
            content: vec![ContentBlock::text(text.to_string(), None)],
            model: model.to_string(),
            stop_reason: Some(stop_reason.to_string()),
            stop_sequence: None,
            usage: GatewayUsage {
                input_tokens: 10,
                output_tokens: 5,
                cache_creation_input_tokens: None,
                cache_read_input_tokens: None,
            },
        }
    }

    fn action_and_tool_response(action: &str) -> GatewayResponse {
        GatewayResponse {
            id: "msg_test".to_string(),
            r#type: "message".to_string(),
            role: "assistant".to_string(),
            content: vec![
                ContentBlock::text(action.to_string(), None),
                ContentBlock::tool_use(
                    "tool_1".to_string(),
                    "Read".to_string(),
                    serde_json::json!({ "file_path": "/tmp" }),
                ),
            ],
            model: "kimi-k2-actual".to_string(),
            stop_reason: Some("tool_use".to_string()),
            stop_sequence: None,
            usage: GatewayUsage {
                input_tokens: 10,
                output_tokens: 5,
                cache_creation_input_tokens: None,
                cache_read_input_tokens: None,
            },
        }
    }

    #[tokio::test]
    async fn handle_messages_returns_final_only_completion_without_internal_leakage() {
        let temp_dir = TempDir::new().unwrap();
        let provider = StubProvider::new(
            final_only_response("All done.", "kimi-k2-actual", "end_turn"),
            vec![],
        );
        let (state, captured_requests) = create_test_state(&temp_dir, provider, false);

        let request = json!({
            "model": "claude-sonnet-4-5",
            "messages": [
                {
                    "role": "user",
                    "content": "Give me the final answer only."
                }
            ],
            "max_tokens": 256
        });

        let response = handle_messages(State(state), HeaderMap::new(), Json(request))
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["stop_reason"], "end_turn");
        assert_eq!(json["content"][0]["type"], "text");
        assert_eq!(json["content"][0]["text"], "All done.");
        assert!(!body
            .windows("<system-reminder>".len())
            .any(|w| w == b"<system-reminder>"));

        let requests = captured_requests.lock().unwrap();
        assert_eq!(requests.len(), 1);
        assert_eq!(requests[0].model, "kimi-k2-actual");
        assert_eq!(requests[0].messages.len(), 1);
        match &requests[0].messages[0].content {
            MessageContent::Text(text) => assert_eq!(text, "Give me the final answer only."),
            other => panic!("expected text request, got {:?}", other),
        }
    }

    #[tokio::test]
    async fn handle_messages_injects_continuation_for_tool_result_follow_up() {
        let temp_dir = TempDir::new().unwrap();
        let provider = StubProvider::new(
            final_only_response("Continuing.", "kimi-k2-actual", "end_turn"),
            vec![],
        );
        let (state, captured_requests) = create_test_state(&temp_dir, provider, true);

        let request = json!({
            "model": "claude-sonnet-4-5",
            "messages": [
                {
                    "role": "user",
                    "content": "Plan a repository audit."
                },
                {
                    "role": "assistant",
                    "content": [
                        {
                            "type": "tool_use",
                            "id": "tool_1",
                            "name": "Read",
                            "input": { "file_path": "/tmp/demo" }
                        }
                    ]
                },
                {
                    "role": "user",
                    "content": [
                        {
                            "type": "tool_result",
                            "tool_use_id": "tool_1",
                            "content": "done"
                        }
                    ]
                }
            ],
            "max_tokens": 256
        });

        let response = handle_messages(State(state), HeaderMap::new(), Json(request))
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let requests = captured_requests.lock().unwrap();
        assert_eq!(requests.len(), 1);

        let last_message = requests[0]
            .messages
            .last()
            .expect("expected tool-result follow-up");
        match &last_message.content {
            MessageContent::Blocks(blocks) => {
                assert_eq!(
                    blocks[0].as_text(),
                    Some("<system-reminder>If you have an active todo list, remember to mark items complete and continue to the next. Do not mention this reminder.</system-reminder>")
                );
                assert!(blocks[1].is_tool_result());
            }
            other => panic!(
                "expected blocks after continuation injection, got {:?}",
                other
            ),
        }
    }

    #[tokio::test]
    async fn claude_code_live_smoke_exercises_normal_think_and_tool_continuation_branches() {
        let temp_dir = TempDir::new().unwrap();
        let provider = StubProvider::new(
            final_only_response("Smoke ok.", "unused-provider-model", "end_turn"),
            vec![],
        );
        let (state, captured_requests) = create_test_state(&temp_dir, provider, true);

        let normal_request = json!({
            "model": "claude-sonnet-4-5",
            "messages": [
                {
                    "role": "user",
                    "content": "Summarize the current repository state."
                }
            ],
            "max_tokens": 128
        });

        let response =
            handle_messages(State(state.clone()), HeaderMap::new(), Json(normal_request))
                .await
                .unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let think_request = json!({
            "model": "claude-sonnet-4-5",
            "messages": [
                {
                    "role": "user",
                    "content": "Plan the next implementation steps."
                }
            ],
            "thinking": {
                "type": "enabled",
                "budget_tokens": 10000
            },
            "max_tokens": 128
        });

        let response = handle_messages(State(state.clone()), HeaderMap::new(), Json(think_request))
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let continuation_request = json!({
            "model": "claude-sonnet-4-5",
            "messages": [
                {
                    "role": "user",
                    "content": "Plan a repository audit."
                },
                {
                    "role": "assistant",
                    "content": [
                        {
                            "type": "tool_use",
                            "id": "tool_1",
                            "name": "Read",
                            "input": { "file_path": "/tmp/demo" }
                        }
                    ]
                },
                {
                    "role": "user",
                    "content": [
                        {
                            "type": "tool_result",
                            "tool_use_id": "tool_1",
                            "content": "done"
                        }
                    ]
                }
            ],
            "thinking": {
                "type": "enabled",
                "budget_tokens": 10000
            },
            "max_tokens": 128
        });

        let response = handle_messages(State(state), HeaderMap::new(), Json(continuation_request))
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let requests = captured_requests.lock().unwrap();
        assert_eq!(requests.len(), 3);
        assert_eq!(requests[0].model, "kimi-k2-actual");
        assert_eq!(requests[1].model, "kimi-k2-thinking-actual");
        assert_eq!(requests[2].model, "kimi-k2-actual");

        assert!(requests[0].reasoning.is_none());
        assert!(requests[1].reasoning.is_some());
        assert!(requests[2].reasoning.is_some());

        let last_message = requests[2]
            .messages
            .last()
            .expect("expected continuation follow-up");
        match &last_message.content {
            MessageContent::Blocks(blocks) => {
                assert_eq!(
                    blocks[0].as_text(),
                    Some("<system-reminder>If you have an active todo list, remember to mark items complete and continue to the next. Do not mention this reminder.</system-reminder>")
                );
                assert!(blocks[1].is_tool_result());
            }
            other => panic!(
                "expected continuation request blocks after prompt injection, got {:?}",
                other
            ),
        }
    }

    #[tokio::test]
    async fn handle_messages_streams_tool_use_sse_from_provider_path() {
        let temp_dir = TempDir::new().unwrap();
        let provider = StubProvider::new(
            final_only_response("unused", "kimi-k2-actual", "end_turn"),
            vec![
                "event: message_start\ndata: {\"type\":\"message_start\",\"message\":{\"id\":\"msg_stream\",\"type\":\"message\",\"role\":\"assistant\",\"content\":[],\"model\":\"kimi-k2-actual\",\"stop_reason\":null,\"stop_sequence\":null,\"usage\":{\"input_tokens\":0,\"output_tokens\":0}}}\n\n".to_string(),
                "event: content_block_start\ndata: {\"type\":\"content_block_start\",\"index\":0,\"content_block\":{\"type\":\"tool_use\",\"id\":\"tool_1\",\"name\":\"Read\",\"input\":{}}}\n\n".to_string(),
                "event: message_delta\ndata: {\"type\":\"message_delta\",\"delta\":{\"stop_reason\":\"tool_use\",\"stop_sequence\":null},\"usage\":{\"input_tokens\":10,\"output_tokens\":3}}\n\n".to_string(),
                "event: message_stop\ndata: {\"type\":\"message_stop\"}\n\n".to_string(),
            ],
        );
        let (state, captured_requests) = create_test_state(&temp_dir, provider, false);

        let request = json!({
            "model": "claude-sonnet-4-5",
            "messages": [
                {
                    "role": "user",
                    "content": "Inspect the repository and call tools if needed."
                }
            ],
            "max_tokens": 256,
            "stream": true
        });

        let response = handle_messages(State(state), HeaderMap::new(), Json(request))
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(
            response.headers().get("content-type").unwrap(),
            "text/event-stream"
        );

        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let text = String::from_utf8(body.to_vec()).unwrap();
        assert!(text.contains("event: content_block_start"));
        assert!(text.contains("\"type\":\"tool_use\""));
        assert!(text.contains("\"stop_reason\":\"tool_use\""));
        assert!(!text.contains("<|tool_calls_section_begin|>"));

        let requests = captured_requests.lock().unwrap();
        assert_eq!(requests.len(), 1);
        assert_eq!(requests[0].model, "kimi-k2-actual");
        assert_eq!(requests[0].stream, Some(true));
    }

    #[tokio::test]
    async fn handle_openai_chat_completions_streams_openai_chunks_and_done() {
        let temp_dir = TempDir::new().unwrap();
        let provider = StubProvider::new(
            final_only_response("unused", "kimi-k2-actual", "end_turn"),
            vec![
                "event: message_start\ndata: {\"type\":\"message_start\",\"message\":{\"id\":\"msg_stream\",\"type\":\"message\",\"role\":\"assistant\",\"content\":[],\"model\":\"kimi-k2-actual\",\"stop_reason\":null,\"stop_sequence\":null,\"usage\":{\"input_tokens\":0,\"output_tokens\":0}}}\n\n".to_string(),
                "event: content_block_start\ndata: {\"type\":\"content_block_start\",\"index\":0,\"content_block\":{\"type\":\"tool_use\",\"id\":\"tool_1\",\"name\":\"Read\",\"input\":{}}}\n\n".to_string(),
                "event: content_block_delta\ndata: {\"type\":\"content_block_delta\",\"index\":0,\"delta\":{\"type\":\"input_json_delta\",\"partial_json\":\"{\\\"path\\\":\\\"README.md\\\"}\"}}\n\n".to_string(),
                "event: content_block_delta\ndata: {\"type\":\"content_block_delta\",\"index\":1,\"delta\":{\"type\":\"text_delta\",\"text\":\"Hello\"}}\n\n".to_string(),
                "event: message_delta\ndata: {\"type\":\"message_delta\",\"delta\":{\"stop_reason\":\"tool_use\",\"stop_sequence\":null},\"usage\":{\"input_tokens\":10,\"output_tokens\":3}}\n\n".to_string(),
                "event: message_stop\ndata: {\"type\":\"message_stop\"}\n\n".to_string(),
            ],
        );
        let (state, captured_requests) = create_test_state(&temp_dir, provider, false);

        let request = json!({
            "model": "claude-sonnet-4-5",
            "messages": [
                { "role": "user", "content": "Inspect the repository." }
            ],
            "stream": true,
            "stream_options": { "include_usage": true },
            "max_tokens": 64
        });

        let response =
            handle_openai_chat_completions(State(state), HeaderMap::new(), Json(request))
                .await
                .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(
            response.headers().get("content-type").unwrap(),
            "text/event-stream"
        );

        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let text = String::from_utf8(body.to_vec()).unwrap();

        assert!(text.contains("\"object\":\"chat.completion.chunk\""));
        assert!(text.contains("\"model\":\"claude-sonnet-4-5\""));
        assert!(text.contains("\"tool_calls\""));
        assert!(text.contains("\"choices\":[]"));
        assert!(text.contains("data: [DONE]"));
        assert!(!text.contains("event: "));
        assert!(!text.contains("content_block_start"));

        let requests = captured_requests.lock().unwrap();
        assert_eq!(requests.len(), 1);
        assert_eq!(requests[0].stream, Some(true));
    }

    #[tokio::test]
    async fn handle_structured_events_returns_events_json() {
        let temp_dir = TempDir::new().unwrap();
        let provider = StubProvider::new(action_and_tool_response("Let me check."), vec![]);
        let (state, _captured_requests) = create_test_state(&temp_dir, provider, false);

        let request = json!({
            "model": "claude-sonnet-4-5",
            "messages": [
                { "role": "user", "content": "Do something." }
            ],
            "max_tokens": 256
        });

        let response = handle_structured_events(State(state), HeaderMap::new(), Json(request))
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert!(json.get("events").is_some());
        assert_eq!(json["events"][0]["event_kind"], "action");
        assert_eq!(json["events"][0]["source_origin"], "assistant_progress");
        assert_eq!(json["events"][0]["summary_text"], "Let me check.");
        assert_eq!(json["events"][1]["event_kind"], "tool_intent");
        assert_eq!(json["events"][1]["source_origin"], "tool_request");
        assert_eq!(json["events"][1]["tool_id"], "tool_1");
        assert_eq!(json["events"][1]["tool_name"], "Read");
        assert_eq!(json["events"][1]["tool_arguments"]["file_path"], "/tmp");
    }

    #[tokio::test]
    async fn handle_structured_events_streams_events_sse() {
        let temp_dir = TempDir::new().unwrap();
        let provider = StubProvider::new(
            final_only_response("unused", "kimi-k2-actual", "end_turn"),
            vec![
                "event: content_block_delta\ndata: {\"type\":\"content_block_delta\",\"index\":0,\"delta\":{\"type\":\"text_delta\",\"text\":\"Let me check.\"}}\n\n".to_string(),
                "event: content_block_start\ndata: {\"type\":\"content_block_start\",\"index\":1,\"content_block\":{\"type\":\"tool_use\",\"id\":\"tool_1\",\"name\":\"Read\",\"input\":{}}}\n\n".to_string(),
                "event: content_block_delta\ndata: {\"type\":\"content_block_delta\",\"index\":1,\"delta\":{\"type\":\"input_json_delta\",\"partial_json\":\"{\\\"file_path\\\":\\\"/tmp\\\"}\"}}\n\n".to_string(),
                "event: content_block_stop\ndata: {\"type\":\"content_block_stop\",\"index\":1}\n\n".to_string(),
                "event: message_delta\ndata: {\"type\":\"message_delta\",\"delta\":{\"stop_reason\":\"tool_use\",\"stop_sequence\":null}}\n\n".to_string(),
                "event: message_stop\ndata: {\"type\":\"message_stop\"}\n\n".to_string(),
            ],
        );
        let (state, _captured_requests) = create_test_state(&temp_dir, provider, false);

        let request = json!({
            "model": "claude-sonnet-4-5",
            "messages": [
                { "role": "user", "content": "Do something." }
            ],
            "max_tokens": 256,
            "stream": true
        });

        let response = handle_structured_events(State(state), HeaderMap::new(), Json(request))
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(
            response.headers().get("content-type").unwrap(),
            "text/event-stream"
        );

        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let text = String::from_utf8(body.to_vec()).unwrap();
        assert!(text.contains("event: action"));
        assert!(text.contains("event: tool_intent"));
        assert!(text.contains("\"tool_id\":\"tool_1\""));
        assert!(text.contains("\"tool_name\":\"Read\""));
        assert!(!text.contains("<|tool_calls_section_begin|>"));
        assert!(!text.contains("\"type\":\"tool_use\""));
    }

    fn create_test_state_with_tracing(
        temp_dir: &TempDir,
        provider: StubProvider,
        inject_continuation_prompt: bool,
        tracing_enabled: bool,
    ) -> Arc<AppState> {
        let mut registry = ProviderRegistry::new();
        registry.insert_provider_for_tests("test-provider", Box::new(provider));

        let config = AppConfig {
            server: ServerConfig {
                port: 13456,
                host: "127.0.0.1".to_string(),
                api_key: None,
                log_level: "info".to_string(),
                timeouts: Default::default(),
                tracing: TracingConfig {
                    enabled: tracing_enabled,
                    path: temp_dir.path().join("trace.jsonl").display().to_string(),
                    omit_system_prompt: true,
                },
            },
            router: RouterConfig {
                default: "gateway-default".to_string(),
                background: None,
                think: Some("gateway-think".to_string()),
                websearch: None,
                auto_map_regex: Some("^claude-".to_string()),
                background_regex: None,
                prompt_rules: vec![],
            },
            providers: vec![],
            models: vec![
                ModelConfig {
                    name: "gateway-default".to_string(),
                    mappings: vec![ModelMapping {
                        priority: 1,
                        provider: "test-provider".to_string(),
                        actual_model: "kimi-k2-actual".to_string(),
                        inject_continuation_prompt,
                    }],
                },
                ModelConfig {
                    name: "gateway-think".to_string(),
                    mappings: vec![ModelMapping {
                        priority: 1,
                        provider: "test-provider".to_string(),
                        actual_model: "kimi-k2-thinking-actual".to_string(),
                        inject_continuation_prompt: false,
                    }],
                },
            ],
        };

        let reloadable = Arc::new(ReloadableState {
            router: Router::new(config.clone()),
            provider_registry: Arc::new(registry),
            config: config.clone(),
        });

        let token_store = TokenStore::new(temp_dir.path().join("oauth_tokens.json")).unwrap();
        Arc::new(AppState {
            inner: std::sync::RwLock::new(reloadable),
            token_store,
            message_tracer: Arc::new(MessageTracer::new(config.server.tracing.clone())),
        })
    }

    #[tokio::test]
    async fn tracing_writes_evt_entries_with_monotonic_seq() {
        let temp_dir = TempDir::new().unwrap();
        let provider = StubProvider::new(action_and_tool_response("Let me check."), vec![]);
        let state = create_test_state_with_tracing(&temp_dir, provider, false, true);

        let request = json!({
            "model": "claude-sonnet-4-5",
            "messages": [
                { "role": "user", "content": "Do something." }
            ],
            "max_tokens": 256
        });

        let response = handle_messages(State(state), HeaderMap::new(), Json(request))
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let trace_path = temp_dir.path().join("trace.jsonl");
        let raw = std::fs::read_to_string(trace_path).expect("read trace.jsonl");
        let mut evt: Vec<serde_json::Value> = raw
            .lines()
            .filter_map(|line| serde_json::from_str::<serde_json::Value>(line).ok())
            .filter(|v| v.get("dir").and_then(|d| d.as_str()) == Some("evt"))
            .collect();

        assert!(!evt.is_empty(), "expected at least one evt entry");

        let id = evt[0]
            .get("id")
            .and_then(|v| v.as_str())
            .unwrap()
            .to_string();
        evt.retain(|v| v.get("id").and_then(|x| x.as_str()) == Some(id.as_str()));

        let mut seqs: Vec<u64> = evt
            .iter()
            .filter_map(|v| v.get("seq").and_then(|s| s.as_u64()))
            .collect();
        seqs.sort_unstable();

        assert_eq!(seqs[0], 0);
        for (idx, seq) in seqs.iter().enumerate() {
            assert_eq!(*seq as usize, idx);
        }
    }
}

/// Handle /v1/structured-events requests (both streaming and non-streaming)
async fn handle_structured_events(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(request_json): Json<serde_json::Value>,
) -> Result<Response, AppError> {
    let model = request_json
        .get("model")
        .and_then(|m| m.as_str())
        .unwrap_or("unknown");
    let start_time = std::time::Instant::now();

    // Get snapshot of reloadable state
    let inner = state.snapshot();

    // Generate trace ID for correlating request/response
    let trace_id = state.message_tracer.new_trace_id();

    // 1. Parse ingress request and convert to gateway core format for routing
    let ingress_request: AnthropicMessagesRequest = serde_json::from_value(request_json.clone())
        .map_err(|e| AppError::Parse(format!("Invalid request format: {}", e)))?;
    let original_public_model = ingress_request.model.clone();
    let mut request_for_routing: GatewayRequest =
        ingress_request.try_into().map_err(AppError::Parse)?;

    // 2. Route the request (may modify system prompt to remove CCM-SUBAGENT-MODEL tag)
    let decision = inner
        .router
        .route(&mut request_for_routing)
        .map_err(|e| AppError::Routing(e.to_string()))?;

    // 3. Try model mappings with fallback (1:N mapping)
    if let Some(model_config) = inner
        .config
        .models
        .iter()
        .find(|m| m.name.eq_ignore_ascii_case(&decision.model_name))
    {
        // Check for X-Provider header to override priority
        let forced_provider = headers
            .get("x-provider")
            .and_then(|v| v.to_str().ok())
            .filter(|s| !s.is_empty()) // Ignore empty strings
            .map(|s| s.to_string());

        // Sort mappings by priority (or filter by forced provider)
        let mut sorted_mappings = model_config.mappings.clone();

        if let Some(ref provider_name) = forced_provider {
            // Filter to only the specified provider
            sorted_mappings.retain(|m| m.provider == *provider_name);
            if sorted_mappings.is_empty() {
                return Err(AppError::Routing(format!(
                    "Provider '{}' not found in mappings for model '{}'",
                    provider_name, decision.model_name
                )));
            }
        } else {
            // Use priority ordering
            sorted_mappings.sort_by_key(|m| m.priority);
        }

        let mut last_failure_class: Option<FailureClass> = None;

        for mapping in sorted_mappings.iter() {
            if let Some(provider) = inner.provider_registry.get_provider(&mapping.provider) {
                let mut gateway_request = request_for_routing.clone();

                // Save original model name for consistency (public-facing model)
                let original_model = original_public_model.clone();

                // Update model to actual model name
                gateway_request.model = mapping.actual_model.clone();

                // Inject continuation prompt if configured (skip for background tasks)
                if mapping.inject_continuation_prompt
                    && decision.route_type != RouteType::Background
                {
                    if let Some(last_msg) = gateway_request.messages.last_mut() {
                        if should_inject_continuation(last_msg) {
                            inject_continuation_text(last_msg);
                        }
                    }
                }

                let is_streaming = gateway_request.stream == Some(true);

                // Trace the request
                state.message_tracer.trace_request(
                    &trace_id,
                    &gateway_request,
                    &mapping.provider,
                    &decision.route_type,
                    is_streaming,
                );

                if is_streaming {
                    match provider.send_message_stream(gateway_request).await {
                        Ok(stream_response) => {
                            let traced_provider_stream: Pin<
                                Box<
                                    dyn futures::stream::Stream<
                                            Item = Result<bytes::Bytes, ProviderError>,
                                        > + Send,
                                >,
                            > = if trace_id.is_empty() {
                                stream_response.stream
                            } else {
                                Box::pin(StructuredEventTracingStream::new(
                                    stream_response.stream,
                                    state.message_tracer.clone(),
                                    trace_id.clone(),
                                ))
                            };

                            let normalized_stream =
                                NormalizedEventSseStream::new(traced_provider_stream);

                            let body_stream = normalized_stream.map_err(|e| {
                                error!("Stream error: {}", e);
                                std::io::Error::other(e.to_string())
                            });

                            let body = Body::from_stream(body_stream);
                            let mut response_builder = Response::builder()
                                .status(200)
                                .header("Content-Type", "text/event-stream")
                                .header("Cache-Control", "no-cache")
                                .header("Connection", "keep-alive");

                            for (name, value) in stream_response.headers {
                                response_builder = response_builder.header(name, value);
                            }

                            return Ok(response_builder.body(body).unwrap());
                        }
                        Err(e) => {
                            last_failure_class = prefer_failure_class(
                                last_failure_class,
                                classify_provider_error(&e),
                            );
                            state.message_tracer.trace_error(&trace_id, &e.to_string());
                            continue;
                        }
                    }
                } else {
                    match provider.send_message(gateway_request).await {
                        Ok(mut response) => {
                            response.model = original_model;

                            let latency_ms = start_time.elapsed().as_millis() as u64;
                            state
                                .message_tracer
                                .trace_response(&trace_id, &response, latency_ms);

                            let events = normalized_events_from_provider_response(&response);
                            if !trace_id.is_empty() {
                                for event in &events {
                                    state.message_tracer.trace_event(&trace_id, event);
                                }
                            }

                            return Ok(
                                Json(serde_json::json!({ "events": events })).into_response()
                            );
                        }
                        Err(e) => {
                            last_failure_class = prefer_failure_class(
                                last_failure_class,
                                classify_provider_error(&e),
                            );
                            state.message_tracer.trace_error(&trace_id, &e.to_string());
                            continue;
                        }
                    }
                }
            } else {
                last_failure_class =
                    prefer_failure_class(last_failure_class, FailureClass::Deployment);
                continue;
            }
        }

        Err(AppError::provider_class(
            last_failure_class.unwrap_or(FailureClass::Deployment),
            format!(
                "All {} provider mappings failed for the routed model",
                sorted_mappings.len()
            ),
        ))
    } else {
        // No model mapping found, try direct provider registry lookup (backward compatibility)
        if let Ok(provider) = inner
            .provider_registry
            .get_provider_for_model(&decision.model_name)
        {
            let mut gateway_request = request_for_routing.clone();

            let original_model = original_public_model.clone();
            gateway_request.model = decision.model_name.clone();

            let is_streaming = gateway_request.stream == Some(true);
            state.message_tracer.trace_request(
                &trace_id,
                &gateway_request,
                &decision.model_name,
                &decision.route_type,
                is_streaming,
            );

            if is_streaming {
                let stream_response = provider
                    .send_message_stream(gateway_request)
                    .await
                    .map_err(|e| {
                        AppError::provider_class(
                            classify_provider_error(&e),
                            "Provider streaming request failed",
                        )
                    })?;

                let traced_provider_stream: Pin<
                    Box<
                        dyn futures::stream::Stream<Item = Result<bytes::Bytes, ProviderError>>
                            + Send,
                    >,
                > = if trace_id.is_empty() {
                    stream_response.stream
                } else {
                    Box::pin(StructuredEventTracingStream::new(
                        stream_response.stream,
                        state.message_tracer.clone(),
                        trace_id.clone(),
                    ))
                };

                let normalized_stream = NormalizedEventSseStream::new(traced_provider_stream);

                let body_stream = normalized_stream.map_err(|e| {
                    error!("Stream error: {}", e);
                    std::io::Error::other(e.to_string())
                });

                let body = Body::from_stream(body_stream);
                let mut response_builder = Response::builder()
                    .status(200)
                    .header("Content-Type", "text/event-stream")
                    .header("Cache-Control", "no-cache")
                    .header("Connection", "keep-alive");

                for (name, value) in stream_response.headers {
                    response_builder = response_builder.header(name, value);
                }

                return Ok(response_builder.body(body).unwrap());
            }

            let mut provider_response =
                provider.send_message(gateway_request).await.map_err(|e| {
                    AppError::provider_class(classify_provider_error(&e), "Provider request failed")
                })?;

            provider_response.model = original_model;
            let latency_ms = start_time.elapsed().as_millis() as u64;
            state
                .message_tracer
                .trace_response(&trace_id, &provider_response, latency_ms);

            let events = normalized_events_from_provider_response(&provider_response);
            if !trace_id.is_empty() {
                for event in &events {
                    state.message_tracer.trace_event(&trace_id, event);
                }
            }

            return Ok(Json(serde_json::json!({ "events": events })).into_response());
        }

        error!("❌ No model mapping or provider found for model: {}", model);
        Err(AppError::provider_class(
            FailureClass::Route,
            "Gateway route selection failed",
        ))
    }
}

/// Handle /v1/messages requests (both streaming and non-streaming)
async fn handle_messages(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(request_json): Json<serde_json::Value>,
) -> Result<Response, AppError> {
    let model = request_json
        .get("model")
        .and_then(|m| m.as_str())
        .unwrap_or("unknown");
    let start_time = std::time::Instant::now();

    // Get snapshot of reloadable state
    let inner = state.snapshot();

    // Generate trace ID for correlating request/response
    let trace_id = state.message_tracer.new_trace_id();

    // DEBUG: Log request body for debugging
    if let Ok(json_str) = serde_json::to_string_pretty(&request_json) {
        tracing::debug!("📥 Incoming request body:\n{}", json_str);
    }

    // 1. Parse ingress request and convert to gateway core format for routing
    let ingress_request: AnthropicMessagesRequest = serde_json::from_value(request_json.clone())
        .map_err(|e| {
            // Log the full request on parse failure for debugging
            if let Ok(pretty) = serde_json::to_string_pretty(&request_json) {
                tracing::error!(
                    "❌ Failed to parse request: {}\n📋 Request body:\n{}",
                    e,
                    pretty
                );
            } else {
                tracing::error!("❌ Failed to parse request: {}", e);
            }
            AppError::Parse(format!("Invalid request format: {}", e))
        })?;
    let original_public_model = ingress_request.model.clone();
    let mut request_for_routing: GatewayRequest =
        ingress_request.try_into().map_err(AppError::Parse)?;

    // 2. Route the request (may modify system prompt to remove CCM-SUBAGENT-MODEL tag)
    let decision = inner
        .router
        .route(&mut request_for_routing)
        .map_err(|e| AppError::Routing(e.to_string()))?;

    // 3. Try model mappings with fallback (1:N mapping)
    if let Some(model_config) = inner
        .config
        .models
        .iter()
        .find(|m| m.name.eq_ignore_ascii_case(&decision.model_name))
    {
        // Check for X-Provider header to override priority
        let forced_provider = headers
            .get("x-provider")
            .and_then(|v| v.to_str().ok())
            .filter(|s| !s.is_empty()) // Ignore empty strings
            .map(|s| s.to_string());

        if let Some(ref provider_name) = forced_provider {
            info!(
                "🎯 Using forced provider from X-Provider header: {}",
                provider_name
            );
        }

        // Sort mappings by priority (or filter by forced provider)
        let mut sorted_mappings = model_config.mappings.clone();

        if let Some(ref provider_name) = forced_provider {
            // Filter to only the specified provider
            sorted_mappings.retain(|m| m.provider == *provider_name);
            if sorted_mappings.is_empty() {
                return Err(AppError::Routing(format!(
                    "Provider '{}' not found in mappings for model '{}'",
                    provider_name, decision.model_name
                )));
            }
        } else {
            // Use priority ordering
            sorted_mappings.sort_by_key(|m| m.priority);
        }

        let mut last_failure_class: Option<FailureClass> = None;

        // Try each mapping in priority order (or just the forced one)
        for (idx, mapping) in sorted_mappings.iter().enumerate() {
            // Try to get provider from registry
            if let Some(provider) = inner.provider_registry.get_provider(&mapping.provider) {
                // Trust the model mapping configuration - no need to validate

                let mut gateway_request = request_for_routing.clone();

                // Save original model name for response (public-facing model)
                let original_model = original_public_model.clone();

                // Update model to actual model name
                gateway_request.model = mapping.actual_model.clone();

                // Inject continuation prompt if configured (skip for background tasks)
                if mapping.inject_continuation_prompt
                    && decision.route_type != RouteType::Background
                {
                    if let Some(last_msg) = gateway_request.messages.last_mut() {
                        if should_inject_continuation(last_msg) {
                            info!(
                                "💉 Injecting continuation prompt for model: {}",
                                mapping.actual_model
                            );
                            inject_continuation_text(last_msg);
                        }
                    }
                }

                // Check if streaming is requested
                let is_streaming = gateway_request.stream == Some(true);

                // Build retry indicator (only show if not first attempt)
                let retry_info = if idx > 0 {
                    format!(" [{}/{}]", idx + 1, sorted_mappings.len())
                } else {
                    String::new()
                };

                let stream_mode = if is_streaming { "stream" } else { "sync" };

                // Build route type display (include matched prompt snippet if available)
                let route_type_display = match &decision.matched_prompt {
                    Some(matched) => {
                        // Trim prompt to max 30 chars
                        let trimmed = if matched.len() > 30 {
                            format!("{}...", &matched[..27])
                        } else {
                            matched.clone()
                        };
                        format!("{}:{}", decision.route_type, trimmed)
                    }
                    None => decision.route_type.to_string(),
                };

                info!(
                    "[{:<15}:{}] {:<25} → {}/{}{}",
                    route_type_display,
                    stream_mode,
                    model,
                    mapping.provider,
                    mapping.actual_model,
                    retry_info
                );

                // Trace the request
                state.message_tracer.trace_request(
                    &trace_id,
                    &gateway_request,
                    &mapping.provider,
                    &decision.route_type,
                    is_streaming,
                );

                // Write routing info immediately on first attempt
                if idx == 0 {
                    write_routing_info(
                        &mapping.actual_model,
                        &mapping.provider,
                        &decision.route_type,
                    );
                }

                if is_streaming {
                    // Streaming request
                    match provider.send_message_stream(gateway_request).await {
                        Ok(stream_response) => {
                            // Write routing info on fallback success (idx==0 already wrote above)
                            if idx > 0 {
                                write_routing_info(
                                    &mapping.actual_model,
                                    &mapping.provider,
                                    &decision.route_type,
                                );
                            }

                            // Convert provider stream to HTTP response
                            // The provider already returns properly formatted SSE bytes (event: + data: lines)
                            // We pass them through as-is (Claude Code expects Anthropic SSE),
                            // but optionally trace normalized structured events (C-06) from those bytes.
                            let traced_stream: Pin<
                                Box<
                                    dyn futures::stream::Stream<
                                            Item = Result<bytes::Bytes, ProviderError>,
                                        > + Send,
                                >,
                            > = if trace_id.is_empty() {
                                stream_response.stream
                            } else {
                                Box::pin(StructuredEventTracingStream::new(
                                    stream_response.stream,
                                    state.message_tracer.clone(),
                                    trace_id.clone(),
                                ))
                            };

                            let body_stream = traced_stream.map_err(|e| {
                                error!("Stream error: {}", e);
                                std::io::Error::other(e.to_string())
                            });

                            let body = Body::from_stream(body_stream);
                            let mut response_builder = Response::builder()
                                .status(200)
                                .header("Content-Type", "text/event-stream")
                                .header("Cache-Control", "no-cache")
                                .header("Connection", "keep-alive");

                            // Forward Anthropic rate limit headers
                            for (name, value) in stream_response.headers {
                                response_builder = response_builder.header(name, value);
                            }

                            let response = response_builder.body(body).unwrap();

                            return Ok(response);
                        }
                        Err(e) => {
                            state.message_tracer.trace_error(&trace_id, &e.to_string());
                            info!(
                                "⚠️ Provider {} streaming failed: {}, trying next fallback",
                                mapping.provider, e
                            );
                            continue;
                        }
                    }
                } else {
                    // Non-streaming request (original behavior)
                    match provider.send_message(gateway_request).await {
                        Ok(mut response) => {
                            // Restore original model name in response
                            response.model = original_model;
                            info!(
                                "✅ Request succeeded with provider: {}, response model: {}",
                                mapping.provider, response.model
                            );

                            // Calculate and log metrics
                            let latency_ms = start_time.elapsed().as_millis() as u64;
                            let tok_s =
                                (response.usage.output_tokens as f32 * 1000.0) / latency_ms as f32;
                            info!(
                                "📊 {}@{} {}ms {:.0}t/s {}tok",
                                mapping.actual_model,
                                mapping.provider,
                                latency_ms,
                                tok_s,
                                response.usage.output_tokens
                            );

                            // Trace the response
                            state
                                .message_tracer
                                .trace_response(&trace_id, &response, latency_ms);

                            // Trace normalized structured events (C-06)
                            if !trace_id.is_empty() {
                                let events = normalized_events_from_provider_response(&response);
                                for event in events {
                                    state.message_tracer.trace_event(&trace_id, &event);
                                }
                            }

                            // Write routing info on fallback success (idx==0 already wrote above)
                            if idx > 0 {
                                write_routing_info(
                                    &mapping.actual_model,
                                    &mapping.provider,
                                    &decision.route_type,
                                );
                            }

                            return Ok(Json(response).into_response());
                        }
                        Err(e) => {
                            last_failure_class = prefer_failure_class(
                                last_failure_class,
                                classify_provider_error(&e),
                            );
                            state.message_tracer.trace_error(&trace_id, &e.to_string());
                            info!(
                                "⚠️ Provider {} failed: {}, trying next fallback",
                                mapping.provider, e
                            );
                            continue;
                        }
                    }
                }
            } else {
                last_failure_class =
                    prefer_failure_class(last_failure_class, FailureClass::Deployment);
                info!(
                    "⚠️ Provider {} not found in registry, trying next fallback",
                    mapping.provider
                );
                continue;
            }
        }

        error!(
            "❌ All provider mappings failed for model: {}",
            decision.model_name
        );
        Err(AppError::provider_class(
            last_failure_class.unwrap_or(FailureClass::Deployment),
            format!(
                "All {} provider mappings failed for the routed model",
                sorted_mappings.len()
            ),
        ))
    } else {
        // No model mapping found, try direct provider registry lookup (backward compatibility)
        if let Ok(provider) = inner
            .provider_registry
            .get_provider_for_model(&decision.model_name)
        {
            info!(
                "📦 Using provider from registry (direct lookup): {}",
                decision.model_name
            );

            let mut gateway_request = request_for_routing.clone();

            // Save original model name for response (public-facing model)
            let original_model = original_public_model.clone();

            // Update model to routed model
            gateway_request.model = decision.model_name.clone();

            // Call provider
            let mut provider_response =
                provider.send_message(gateway_request).await.map_err(|e| {
                    AppError::provider_class(
                        classify_provider_error(&e),
                        "Azure provider request failed",
                    )
                })?;

            // Restore original model name in response
            provider_response.model = original_model;

            // Return provider response
            return Ok(Json(provider_response).into_response());
        }

        error!(
            "❌ No model mapping or provider found for model: {}",
            decision.model_name
        );
        Err(AppError::provider_class(
            FailureClass::Route,
            "Gateway route selection failed",
        ))
    }
}

/// Handle /v1/messages/count_tokens requests
async fn handle_count_tokens(
    State(state): State<Arc<AppState>>,
    Json(request_json): Json<serde_json::Value>,
) -> Result<Response, AppError> {
    let model = request_json
        .get("model")
        .and_then(|m| m.as_str())
        .unwrap_or("unknown");
    debug!("Received count_tokens request for model: {}", model);

    // Get snapshot of reloadable state
    let inner = state.snapshot();

    // 1. Parse as CountTokensRequest first
    use crate::models::CountTokensRequest;
    let count_request: CountTokensRequest = serde_json::from_value(request_json.clone())
        .map_err(|e| AppError::Parse(format!("Invalid count_tokens request format: {}", e)))?;

    // 2. Create a minimal GatewayRequest for routing
    let mut routing_request = GatewayRequest {
        model: count_request.model.clone(),
        messages: count_request.messages.clone(),
        max_output_tokens: 1024, // Dummy value for routing
        reasoning: None,
        temperature: None,
        top_p: None,
        top_k: None,
        stop_sequences: None,
        stream: None,
        metadata: None,
        system: count_request.system.clone(),
        tools: count_request.tools.clone(),
    };
    let decision = inner
        .router
        .route(&mut routing_request)
        .map_err(|e| AppError::Routing(e.to_string()))?;

    debug!(
        "🧮 Routed count_tokens: {} → {} ({})",
        model, decision.model_name, decision.route_type
    );

    // 3. Try model mappings with fallback (1:N mapping)
    if let Some(model_config) = inner
        .config
        .models
        .iter()
        .find(|m| m.name.eq_ignore_ascii_case(&decision.model_name))
    {
        debug!(
            "📋 Found {} provider mappings for token counting: {}",
            model_config.mappings.len(),
            decision.model_name
        );

        // Sort mappings by priority
        let mut sorted_mappings = model_config.mappings.clone();
        sorted_mappings.sort_by_key(|m| m.priority);

        // Try each mapping in priority order
        for (idx, mapping) in sorted_mappings.iter().enumerate() {
            debug!(
                "🔄 Trying token count mapping {}/{}: provider={}, actual_model={}",
                idx + 1,
                sorted_mappings.len(),
                mapping.provider,
                mapping.actual_model
            );

            // Try to get provider from registry
            if let Some(provider) = inner.provider_registry.get_provider(&mapping.provider) {
                // Trust the model mapping configuration - no need to validate

                // Update model to actual model name
                let mut count_request_for_provider = count_request.clone();
                count_request_for_provider.model = mapping.actual_model.clone();

                // Call provider's count_tokens
                match provider.count_tokens(count_request_for_provider).await {
                    Ok(response) => {
                        debug!(
                            "✅ Token count succeeded with provider: {}",
                            mapping.provider
                        );
                        return Ok(Json(response).into_response());
                    }
                    Err(e) => {
                        debug!(
                            "⚠️ Provider {} failed: {}, trying next fallback",
                            mapping.provider, e
                        );
                        continue;
                    }
                }
            } else {
                debug!(
                    "⚠️ Provider {} not found in registry, trying next fallback",
                    mapping.provider
                );
                continue;
            }
        }

        error!(
            "❌ All provider mappings failed for token counting: {}",
            decision.model_name
        );
        Err(AppError::provider_class(
            FailureClass::Deployment,
            format!(
                "All {} provider mappings failed for token counting",
                sorted_mappings.len()
            ),
        ))
    } else {
        // No model mapping found, try direct provider registry lookup (backward compatibility)
        if let Ok(provider) = inner
            .provider_registry
            .get_provider_for_model(&decision.model_name)
        {
            debug!(
                "📦 Using provider from registry (direct lookup) for token counting: {}",
                decision.model_name
            );

            // Update model to routed model
            let mut count_request_for_provider = count_request.clone();
            count_request_for_provider.model = decision.model_name.clone();

            // Call provider's count_tokens
            let response = provider
                .count_tokens(count_request_for_provider)
                .await
                .map_err(|e| {
                    AppError::provider_class(
                        classify_provider_error(&e),
                        "Azure provider request failed",
                    )
                })?;

            debug!("✅ Token count completed via provider");
            return Ok(Json(response).into_response());
        }

        error!(
            "❌ No model mapping or provider found for token counting: {}",
            decision.model_name
        );
        Err(AppError::provider_class(
            FailureClass::Route,
            "Gateway route selection failed",
        ))
    }
}

/// Application error types
#[derive(Debug)]
pub enum AppError {
    Routing(String),
    Parse(String),
    Provider {
        class: FailureClass,
        message: String,
    },
}

impl AppError {
    fn provider_class(class: FailureClass, message: impl Into<String>) -> Self {
        Self::Provider {
            class,
            message: message.into(),
        }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, class, message) = match self {
            AppError::Routing(_) => (
                StatusCode::BAD_REQUEST,
                FailureClass::Route,
                public_error_message(FailureClass::Route),
            ),
            AppError::Parse(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                FailureClass::TransportDrift,
                public_error_message(FailureClass::TransportDrift),
            ),
            AppError::Provider { class, .. } => {
                (StatusCode::BAD_GATEWAY, class, public_error_message(class))
            }
        };

        let body = Json(serde_json::json!({
            "error": {
                "type": "error",
                "class": class.as_str(),
                "message": message
            }
        }));

        (status, body).into_response()
    }
}

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AppError::Routing(msg) => write!(f, "Routing error: {}", msg),
            AppError::Parse(msg) => write!(f, "Parse error: {}", msg),
            AppError::Provider { message, .. } => write!(f, "Provider error: {}", message),
        }
    }
}

impl std::error::Error for AppError {}
