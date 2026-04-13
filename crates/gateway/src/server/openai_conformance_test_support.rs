#![allow(dead_code)]

use crate::cli::{
    AppConfig, ModelConfig, ModelMapping, RouterConfig, ServerConfig, TimeoutConfig, TracingConfig,
};
use crate::core::{GatewayRequest, GatewayResponse, GatewayStreamResponse, GatewayUsage};
use crate::message_tracing::MessageTracer;
use crate::models::{ContentBlock, KnownContentBlock};
use crate::providers::error::ProviderError;
use crate::providers::{GatewayProvider, ProviderRegistry};
use crate::router::Router;
use async_trait::async_trait;
use axum::{
    body::to_bytes,
    extract::State,
    http::HeaderMap,
    response::{IntoResponse, Response},
    routing::post,
    Json, Router as AxumRouter,
};
use bytes::Bytes;
use futures::stream;
use serde::de::DeserializeOwned;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use uuid::Uuid;

use super::ReloadableState;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FixtureNamespace {
    OpenAiChatCompletions,
    OpenAiResponses,
}

impl FixtureNamespace {
    fn as_dir(self) -> &'static str {
        match self {
            FixtureNamespace::OpenAiChatCompletions => "openai_chat_completions",
            FixtureNamespace::OpenAiResponses => "openai_responses",
        }
    }
}

pub fn fixture_path(namespace: FixtureNamespace, name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join(namespace.as_dir())
        .join(name)
}

pub fn read_json_fixture<T: DeserializeOwned>(namespace: FixtureNamespace, name: &str) -> T {
    let raw = fs::read_to_string(fixture_path(namespace, name)).unwrap();
    serde_json::from_str(&raw).unwrap()
}

pub fn split_string_into_chunks(text: &str, split_points: &[usize]) -> Vec<String> {
    let mut out = Vec::new();
    let mut start = 0usize;
    let len = text.len();

    for &point in split_points {
        assert!(point <= len, "split point beyond text length");
        assert!(point >= start, "split points must be nondecreasing");
        out.push(text[start..point].to_string());
        start = point;
    }

    if start < len {
        out.push(text[start..].to_string());
    }

    out
}

pub async fn response_text(response: Response) -> String {
    String::from_utf8(
        to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap()
            .to_vec(),
    )
    .unwrap()
}

pub struct ConformanceHarness {
    state: Arc<crate::server::AppState>,
    captured_requests: Arc<Mutex<Vec<GatewayRequest>>>,
    workspace: PathBuf,
}

impl Drop for ConformanceHarness {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.workspace);
    }
}

impl ConformanceHarness {
    fn build_state(
        router_default_model: impl Into<String>,
        provider_registry: ProviderRegistry,
        models: Vec<ModelConfig>,
    ) -> (Arc<crate::server::AppState>, PathBuf) {
        let router_default_model = router_default_model.into();
        let workspace =
            std::env::temp_dir().join(format!("substrate-gateway-conformance-{}", Uuid::new_v4()));
        fs::create_dir_all(&workspace).unwrap();

        let tracing_path = workspace.join("trace.jsonl");
        let token_store_path = workspace.join("oauth_tokens.json");
        let token_store = crate::auth::TokenStore::new(token_store_path).unwrap();

        let config = AppConfig {
            server: ServerConfig {
                port: 3000,
                host: "127.0.0.1".to_string(),
                api_key: None,
                log_level: "info".to_string(),
                timeouts: TimeoutConfig::default(),
                tracing: TracingConfig {
                    enabled: false,
                    path: tracing_path.display().to_string(),
                    omit_system_prompt: true,
                },
            },
            router: RouterConfig {
                default: router_default_model,
                background: None,
                think: None,
                websearch: None,
                auto_map_regex: None,
                background_regex: None,
                prompt_rules: vec![],
            },
            providers: vec![],
            models,
        };

        let router = Router::new(config.clone());
        let message_tracer = Arc::new(MessageTracer::new(config.server.tracing.clone()));
        let state = Arc::new(crate::server::AppState {
            inner: std::sync::RwLock::new(Arc::new(ReloadableState {
                config,
                router,
                provider_registry: Arc::new(provider_registry),
            })),
            token_store,
            message_tracer,
        });

        (state, workspace)
    }

    pub fn with_registry(
        model_name: impl Into<String>,
        provider_registry: ProviderRegistry,
        model_mappings: Vec<ModelMapping>,
    ) -> Self {
        let model_name = model_name.into();
        let (state, workspace) = Self::build_state(
            model_name.clone(),
            provider_registry,
            vec![ModelConfig {
                name: model_name,
                mappings: model_mappings,
            }],
        );

        Self {
            state,
            captured_requests: Arc::new(Mutex::new(Vec::new())),
            workspace,
        }
    }

    pub fn with_direct_registry(
        router_default_model: impl Into<String>,
        provider_registry: ProviderRegistry,
    ) -> Self {
        let (state, workspace) =
            Self::build_state(router_default_model, provider_registry, Vec::new());

        Self {
            state,
            captured_requests: Arc::new(Mutex::new(Vec::new())),
            workspace,
        }
    }

    pub fn single_provider(
        provider: StubProvider,
        actual_model: impl Into<String>,
        inject_continuation_prompt: bool,
    ) -> Self {
        let captured_requests = provider.captured_requests();
        let mut provider_registry = ProviderRegistry::new();
        provider_registry.insert_provider_for_tests("test-provider", Box::new(provider));

        let (state, workspace) = Self::build_state(
            "gateway-default",
            provider_registry,
            vec![ModelConfig {
                name: "gateway-default".to_string(),
                mappings: vec![ModelMapping {
                    priority: 1,
                    provider: "test-provider".to_string(),
                    actual_model: actual_model.into(),
                    inject_continuation_prompt,
                }],
            }],
        );

        Self {
            state,
            captured_requests,
            workspace,
        }
    }

    pub fn state(&self) -> Arc<crate::server::AppState> {
        self.state.clone()
    }

    pub fn captured_requests(&self) -> Arc<Mutex<Vec<GatewayRequest>>> {
        self.captured_requests.clone()
    }

    pub fn router(&self) -> AxumRouter {
        AxumRouter::new()
            .route(
                "/v1/chat/completions",
                post(super::handle_openai_chat_completions),
            )
            .route(
                "/v1/responses",
                post(super::openai_responses::handle_openai_responses),
            )
            .with_state(self.state.clone())
    }

    pub async fn invoke_chat_completions(
        &self,
        headers: HeaderMap,
        request_json: serde_json::Value,
    ) -> Response {
        match super::handle_openai_chat_completions(
            State(self.state.clone()),
            headers,
            Json(request_json),
        )
        .await
        {
            Ok(response) => response,
            Err(err) => err.into_response(),
        }
    }

    pub async fn invoke_responses(
        &self,
        headers: HeaderMap,
        request_json: serde_json::Value,
    ) -> Response {
        match super::openai_responses::handle_openai_responses(
            State(self.state.clone()),
            headers,
            Json(request_json),
        )
        .await
        {
            Ok(response) => response,
            Err(err) => err.into_response(),
        }
    }
}

pub fn build_provider_stream(chunks: Vec<String>) -> GatewayStreamResponse {
    let items = chunks.into_iter().map(|chunk| Ok(Bytes::from(chunk)));
    GatewayStreamResponse {
        stream: Box::pin(stream::iter(items)),
        headers: HashMap::new(),
    }
}

#[derive(Debug, Clone)]
pub struct StubProvider {
    response: GatewayResponse,
    stream_chunks: Vec<String>,
    captured_requests: Arc<Mutex<Vec<GatewayRequest>>>,
}

impl StubProvider {
    pub fn new(response: GatewayResponse, stream_chunks: Vec<String>) -> Self {
        Self {
            response,
            stream_chunks,
            captured_requests: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn captured_requests(&self) -> Arc<Mutex<Vec<GatewayRequest>>> {
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
        Ok(build_provider_stream(self.stream_chunks.clone()))
    }

    async fn count_tokens(
        &self,
        _request: crate::models::CountTokensRequest,
    ) -> Result<crate::models::CountTokensResponse, ProviderError> {
        Ok(crate::models::CountTokensResponse { input_tokens: 0 })
    }

    fn supports_model(&self, _model: &str) -> bool {
        true
    }
}

pub fn response_text_response(text: &str, model: &str) -> GatewayResponse {
    GatewayResponse {
        id: "resp_1".to_string(),
        r#type: "message".to_string(),
        role: "assistant".to_string(),
        content: vec![ContentBlock::text(text.to_string(), None)],
        model: model.to_string(),
        stop_reason: Some("end_turn".to_string()),
        stop_sequence: None,
        usage: GatewayUsage {
            input_tokens: 11,
            output_tokens: 7,
            cache_creation_input_tokens: None,
            cache_read_input_tokens: None,
        },
    }
}

pub fn response_with_text_and_tool(model: &str) -> GatewayResponse {
    GatewayResponse {
        id: "resp_2".to_string(),
        r#type: "message".to_string(),
        role: "assistant".to_string(),
        content: vec![
            ContentBlock::text("before".to_string(), None),
            ContentBlock::Known(KnownContentBlock::Thinking {
                raw: serde_json::json!({"type":"thinking","text":"hidden"}),
            }),
            ContentBlock::Known(KnownContentBlock::ToolUse {
                id: "call_1".to_string(),
                name: "lookup".to_string(),
                input: serde_json::json!({"query":"x"}),
            }),
            ContentBlock::text("after".to_string(), None),
        ],
        model: model.to_string(),
        stop_reason: Some("tool_use".to_string()),
        stop_sequence: None,
        usage: GatewayUsage {
            input_tokens: 9,
            output_tokens: 3,
            cache_creation_input_tokens: None,
            cache_read_input_tokens: None,
        },
    }
}

pub fn response_with_tool_use(action: &str, model: &str) -> GatewayResponse {
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
        model: model.to_string(),
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

pub fn anthropic_sse(event: &str, payload: serde_json::Value) -> Bytes {
    Bytes::from(format!(
        "event: {event}\ndata: {}\n\n",
        serde_json::to_string(&payload).unwrap()
    ))
}
