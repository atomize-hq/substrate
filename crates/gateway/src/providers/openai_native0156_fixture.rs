//! Private test process hosting the actual Responses handler and OAuth adapter.
use super::*;
use crate::auth::codex_auth_context::{
    integrated_codex_auth_test_lock, CodexIntegratedAuthHandoff, InstalledHandoffGuard,
};
use crate::cli::ModelMapping;
use crate::providers::ProviderRegistry;
use crate::server::openai_conformance_test_support::ConformanceHarness;
use serde_json::{json, Value};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

struct Responder(Child);
impl Drop for Responder {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

async fn wait_json(path: &Path) -> Value {
    let deadline = Instant::now() + Duration::from_secs(15);
    loop {
        if let Ok(bytes) = fs::read(path) {
            if let Ok(value) = serde_json::from_slice(&bytes) {
                return value;
            }
        }
        assert!(
            Instant::now() < deadline,
            "missing readiness: {}",
            path.display()
        );
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
}

async fn serve(root: &Path, preflight: bool) {
    fs::create_dir_all(root).unwrap();
    let log = fs::File::create(root.join("responder.log")).unwrap();
    let _responder = Responder(
        Command::new("/usr/bin/python3")
            .arg(
                Path::new(env!("CARGO_MANIFEST_DIR"))
                    .join("tests/fixtures/codex0156_native_responder.py"),
            )
            .arg("--root")
            .arg(root)
            .env_clear()
            .env("PATH", "/usr/bin:/bin")
            .stdin(Stdio::null())
            .stdout(log.try_clone().unwrap())
            .stderr(log)
            .spawn()
            .expect("Python ssl responder"),
    );
    let ready = wait_json(&root.join("tls-ready.json")).await;
    let port = ready["port"].as_u64().unwrap() as u16;
    assert_ne!(port, 0);
    let certificate =
        reqwest::Certificate::from_pem(&fs::read(root.join("certificate.pem")).unwrap()).unwrap();
    let client = Client::builder()
        .no_proxy()
        .resolve("chatgpt.com", ([127, 0, 0, 1], port).into())
        .add_root_certificate(certificate)
        .redirect(reqwest::redirect::Policy::none())
        .timeout(Duration::from_secs(40))
        .build()
        .unwrap();
    let _lock = integrated_codex_auth_test_lock().lock().await;
    let _auth = InstalledHandoffGuard::set(Some(CodexIntegratedAuthHandoff::new(
        Some("native0156-synthetic-account".into()),
        secrecy::SecretString::new("native0156-synthetic-token".into()),
    )));
    let mut provider = OpenAIProvider::with_headers(OpenAIProviderConfig {
        name: "native0156-fixture".into(),
        api_key: String::new(),
        base_url: "https://unused.invalid".into(),
        models: vec!["gpt-6-astra".into()],
        custom_headers: vec![],
        oauth_provider: Some("native0156-fixture".into()),
        token_store: None,
        codex_auth_source: Some(CodexAuthSource::Integrated),
    });
    assert_eq!(
        provider.effective_base_url(),
        "https://chatgpt.com/backend-api"
    );
    provider.responses_client = client;
    let mut registry = ProviderRegistry::new();
    registry.insert_provider_for_tests("native0156-fixture", Box::new(provider));
    let harness = ConformanceHarness::with_registry(
        "gpt-6-astra",
        registry,
        vec![ModelMapping {
            priority: 1,
            provider: "native0156-fixture".into(),
            actual_model: "openai-codex/gpt-6-astra".into(),
        }],
    );
    // A diagnostic can retain the exact observed loopback URL/config. This
    // changes only the private fixture listener, never the OAuth upstream URL.
    let bind_port = fs::read_to_string(root.join("gateway-bind-port"))
        .map(|value| value.trim().parse::<u16>().expect("fixture port"))
        .unwrap_or(0);
    let listener = tokio::net::TcpListener::bind((std::net::Ipv4Addr::LOCALHOST, bind_port))
        .await
        .unwrap();
    let address = listener.local_addr().unwrap();
    let server = tokio::spawn(async move {
        axum::serve(listener, harness.router()).await.unwrap();
    });
    fs::write(
        root.join("gateway-ready.json"),
        serde_json::to_vec(&json!({
            "url": format!("http://{address}/v1"), "pid": std::process::id(), "tls_port": port,
        }))
        .unwrap(),
    )
    .unwrap();
    if preflight {
        fs::write(root.join("scenario.json"), r#"{"mode":"preflight"}"#).unwrap();
        let response = Client::builder()
            .no_proxy()
            .build()
            .unwrap()
            .post(format!("http://{address}/v1/responses"))
            .json(&json!({"model":"gpt-6-astra", "input":"TLS preflight", "stream":true}))
            .send()
            .await
            .unwrap();
        let status = response.status();
        let body = response.text().await.unwrap();
        fs::write(root.join("preflight-response.sse"), &body).unwrap();
        assert!(status.is_success(), "{status}: {body}");
        assert!(body.contains("native0156 TLS verified"), "{body}");
        assert!(body.contains("response.completed"), "{body}");
        let request = wait_json(&root.join("request-000.json")).await;
        assert_eq!(request["synthetic_auth"], true);
        assert_eq!(request["synthetic_account"], true);
        fs::write(
            root.join("tls-preflight-passed.json"),
            serde_json::to_vec_pretty(&json!({
                "fixed_host": "chatgpt.com", "nonzero_resolver_port": port,
                "normal_hostname_verification": true, "redirects": false, "proxy": false,
                "actual_oauth_adapter": true, "actual_responses_router": true,
            }))
            .unwrap(),
        )
        .unwrap();
    } else {
        let deadline = Instant::now() + Duration::from_secs(240);
        while !root.join("stop").exists() {
            assert!(Instant::now() < deadline, "fixture deadline");
            tokio::time::sleep(Duration::from_millis(25)).await;
        }
    }
    server.abort();
    let _ = server.await;
}

#[tokio::test]
#[ignore = "requires Python ssl, openssl and loopback; explicit native qualification"]
async fn codex0156_local_https_adapter_preflight() {
    let root = PathBuf::from(
        std::env::var_os("SUBSTRATE_NATIVE_CODEX_0156_ATTEMPT_DIR")
            .expect("attempt directory required"),
    );
    serve(&root, true).await;
}

#[tokio::test]
#[ignore = "private fixture worker, launched by the manager unit test"]
async fn codex0156_actual_gateway_fixture_worker() {
    let root = PathBuf::from(
        std::env::var_os("SUBSTRATE_NATIVE_CODEX_0156_ATTEMPT_DIR")
            .expect("attempt directory required"),
    );
    serve(&root, false).await;
}

/// One explicitly authorized live request; no fake upstream, DNS or TLS client.
#[tokio::test]
#[ignore = "explicit live-account authorization required; no automatic suite execution"]
async fn codex0156_live_text_gateway_worker() {
    use crate::auth::CodexAuthState;
    let root = PathBuf::from(std::env::var_os("SUBSTRATE_LIVE_TEXT_ROOT").unwrap());
    let _lock = integrated_codex_auth_test_lock().lock().await;
    let handoff = match CodexIntegratedAuthHandoff::from_env().expect("auth env shape") {
        Some(handoff) => handoff,
        None => {
            let state = CodexAuthState::load_default().expect("existing Codex account unavailable");
            CodexIntegratedAuthHandoff::new(state.account_id, state.access_token)
        }
    };
    let _auth = InstalledHandoffGuard::set(Some(handoff));
    CodexAuthSource::Integrated
        .resolve()
        .expect("existing selected account resolver");
    let provider = OpenAIProvider::with_headers(OpenAIProviderConfig {
        name: "live-text-only".into(),
        api_key: String::new(),
        base_url: "https://unused.invalid".into(),
        models: vec!["gpt-6-astra".into()],
        custom_headers: vec![],
        oauth_provider: Some("live-text-only".into()),
        token_store: None,
        codex_auth_source: Some(CodexAuthSource::Integrated),
    });
    assert_eq!(
        provider.effective_base_url(),
        "https://chatgpt.com/backend-api"
    );
    let mut registry = ProviderRegistry::new();
    let diagnostics = std::sync::Arc::new(live_diagnostics::Diagnostics::new(root.clone()));
    registry.insert_provider_for_tests(
        "live-text-only",
        Box::new(live_diagnostics::ObservedProvider {
            inner: provider,
            diagnostics: diagnostics.clone(),
        }),
    );
    let harness = ConformanceHarness::with_registry(
        "gpt-6-astra",
        registry,
        vec![ModelMapping {
            priority: 1,
            provider: "live-text-only".into(),
            actual_model: "gpt-6-astra".into(),
        }],
    );
    let listener = tokio::net::TcpListener::bind((std::net::Ipv4Addr::LOCALHOST, 0))
        .await
        .unwrap();
    let address = listener.local_addr().unwrap();
    let router = harness.router().layer(axum::middleware::from_fn_with_state(
        diagnostics.clone(),
        live_diagnostics::limit_local_posts,
    ));
    let server = tokio::spawn(async move {
        axum::serve(listener, router).await.unwrap();
    });
    fs::write(
        root.join("gateway-ready.json"),
        serde_json::to_vec_pretty(&json!({
            "url":format!("http://{address}/v1"), "pid":std::process::id(),
            "upstream":"https://chatgpt.com/backend-api/codex/responses",
            "model":"gpt-6-astra", "real_account_resolver":true,
            "custom_dns":false,"custom_tls":false,"scripted_upstream":false,
        }))
        .unwrap(),
    )
    .unwrap();
    let deadline = Instant::now() + Duration::from_secs(150);
    while !root.join("stop").exists() {
        assert!(Instant::now() < deadline, "live worker owner deadline");
        tokio::time::sleep(Duration::from_millis(25)).await;
    }
    fs::write(
        root.join("gateway-count.json"),
        serde_json::to_vec_pretty(&json!({
            "local_responses_posts": diagnostics.local_count(),
            "admitted_local_limit": live_diagnostics::LOCAL_POST_LIMIT,
            "diagnostic_write_failed": diagnostics.write_failed(),
            "upstream_http_attempt_count_observed":false,
        }))
        .unwrap(),
    )
    .unwrap();
    server.abort();
    let _ = server.await;
}

// Closed diagnostic payloads: no request/error strings, headers, bodies or URLs.
mod live_diagnostics {
    use super::*;
    use crate::core::{GatewayRequest, GatewayResponse, GatewayStreamResponse};
    use crate::models::{images::*, search::*};
    use crate::models::{CountTokensRequest, CountTokensResponse};
    use crate::providers::{error::ProviderError, GatewayProvider};
    use futures::StreamExt;
    use serde::Serialize;
    use std::sync::{
        atomic::{AtomicBool, AtomicUsize, Ordering},
        Arc, Mutex,
    };

    pub(super) const LOCAL_POST_LIMIT: usize = 1;
    tokio::task_local! { pub(super) static LOCAL_REQUEST: usize; }

    #[derive(Serialize, Debug, PartialEq, Eq)]
    #[serde(rename_all = "snake_case")]
    enum Category {
        Http,
        Serialization,
        ModelUnsupported,
        Api,
        Configuration,
        Authentication,
    }
    #[derive(Serialize, Debug, PartialEq, Eq)]
    #[serde(rename_all = "snake_case")]
    enum Check {
        UpstreamRejected,
        Temperature,
        TopP,
        StopSequences,
        MaxOutputTokens,
        Metadata,
        Truncation,
        PreviousResponse,
        User,
        StreamOptions,
        ServiceTier,
        ReasoningEffort,
        ReasoningSummary,
        Include,
        Verbosity,
        ResponsesHistory,
        ResponsesRequired,
        ResponsesState,
        Provenance,
        UnsupportedOutputItem,
        MalformedSemanticEvent,
    }
    #[derive(Serialize, Debug, PartialEq, Eq)]
    #[serde(rename_all = "snake_case")]
    enum JsonCategory {
        Io,
        Syntax,
        Data,
        Eof,
    }
    #[derive(Serialize, Debug, PartialEq, Eq)]
    #[serde(rename_all = "snake_case")]
    enum StatusOrigin {
        HttpError,
        UpstreamRejected,
        ProviderDeclared,
    }
    #[derive(Serialize, Debug, PartialEq, Eq)]
    struct SafeError {
        category: Category,
        status: Option<u16>,
        // ApiError may contain a locally synthesized contract status, not HTTP.
        status_origin: Option<StatusOrigin>,
        http_flags: Option<[bool; 6]>, // timeout, connect, builder, decode, redirect, body
        json: Option<(JsonCategory, usize, usize)>,
        check: Option<Check>,
    }
    fn safe_error(error: &ProviderError) -> SafeError {
        let mut out = SafeError {
            category: Category::Api,
            status: None,
            status_origin: None,
            http_flags: None,
            json: None,
            check: None,
        };
        match error {
            ProviderError::HttpError(e) => {
                out.category = Category::Http;
                out.status = e.status().map(|s| s.as_u16());
                out.status_origin = out.status.map(|_| StatusOrigin::HttpError);
                out.http_flags = Some([
                    e.is_timeout(),
                    e.is_connect(),
                    e.is_builder(),
                    e.is_decode(),
                    e.is_redirect(),
                    e.is_body(),
                ]);
            }
            ProviderError::SerializationError(e) => {
                out.category = Category::Serialization;
                out.json = Some((
                    match e.classify() {
                        serde_json::error::Category::Io => JsonCategory::Io,
                        serde_json::error::Category::Syntax => JsonCategory::Syntax,
                        serde_json::error::Category::Data => JsonCategory::Data,
                        serde_json::error::Category::Eof => JsonCategory::Eof,
                    },
                    e.line(),
                    e.column(),
                ));
            }
            ProviderError::ModelNotSupported(_) => out.category = Category::ModelUnsupported,
            ProviderError::AuthError(_) => out.category = Category::Authentication,
            ProviderError::ConfigError(message) => {
                out.category = Category::Configuration;
                out.check=match message.as_str() {
                    "temperature is not supported on the Codex route"=>Some(Check::Temperature),
                    "top_p is not supported on the Codex route"=>Some(Check::TopP),
                    "stop_sequences is not supported on the Codex route"=>Some(Check::StopSequences),
                    "max_output_tokens is not supported on the Codex route"=>Some(Check::MaxOutputTokens),
                    "metadata is not supported on the Codex route"=>Some(Check::Metadata),
                    "truncation is not supported on the Codex route"=>Some(Check::Truncation),
                    "previous_response_id is not supported on the Codex route"=>Some(Check::PreviousResponse),
                    "user is not supported on the Codex route"=>Some(Check::User),
                    "stream_options is not supported on the Codex route"=>Some(Check::StreamOptions),
                    "service_tier is not supported on the Codex route"=>Some(Check::ServiceTier),
                    "Unsupported reasoning.effort on the Codex route"=>Some(Check::ReasoningEffort),
                    "Unsupported reasoning.summary on the Codex route"=>Some(Check::ReasoningSummary),
                    "Unsupported include value on the Codex route"=>Some(Check::Include),
                    "Unsupported text.verbosity on the Codex route"=>Some(Check::Verbosity),
                    "Unsupported Responses history"=>Some(Check::ResponsesHistory),
                    "Request requires Responses transport"=>Some(Check::ResponsesRequired),
                    "Responses state unsupported"=>Some(Check::ResponsesState),
                    "Responses continuation requires authoritative provenance for each function_call_output"=>Some(Check::Provenance),
                    _=>None,
                };
            }
            ProviderError::ApiError { status, message } => {
                out.status = Some(*status);
                out.status_origin = Some(StatusOrigin::ProviderDeclared);
                out.check = match message.as_str() {
                    "Upstream request rejected" | "Upstream Responses request rejected" => {
                        Some(Check::UpstreamRejected)
                    }
                    "Unsupported Responses output item" => Some(Check::UnsupportedOutputItem),
                    "Malformed Codex semantic SSE event" => Some(Check::MalformedSemanticEvent),
                    _ => None,
                };
                if out.check == Some(Check::UpstreamRejected) {
                    out.status_origin = Some(StatusOrigin::UpstreamRejected);
                }
            }
        }
        out
    }
    #[derive(Serialize, Clone, Copy)]
    #[serde(rename_all = "snake_case")]
    enum Stage {
        Started,
        Returned,
        StreamError,
    }
    #[derive(Serialize)]
    struct Observation {
        local_request: usize,
        provider_call: usize,
        stage: Stage,
        stream_obtained: bool,
        error: Option<SafeError>,
    }
    pub(super) struct Diagnostics {
        root: PathBuf,
        local: AtomicUsize,
        calls: AtomicUsize,
        writer: Mutex<()>,
        write_failed: AtomicBool,
    }
    impl Diagnostics {
        pub(super) fn new(root: PathBuf) -> Self {
            Self {
                root,
                local: AtomicUsize::new(0),
                calls: AtomicUsize::new(0),
                writer: Mutex::new(()),
                write_failed: AtomicBool::new(false),
            }
        }
        pub(super) fn write_failed(&self) -> bool {
            self.write_failed.load(Ordering::SeqCst)
        }
        pub(super) fn local_count(&self) -> usize {
            self.local.load(Ordering::SeqCst)
        }
        pub(super) fn local_request(&self) -> (usize, bool) {
            let sequence = self.local.fetch_add(1, Ordering::SeqCst) + 1;
            let _lock = self.writer.lock().unwrap();
            let written = (|| -> std::io::Result<()> {
                fs::write(
                    self.root.join("diagnostic-local-count.tmp"),
                    serde_json::to_vec(&json!({
                        "local_posts":self.local_count(),"admitted_limit":LOCAL_POST_LIMIT,
                        "rejected_local_posts":self.local_count().saturating_sub(LOCAL_POST_LIMIT),
                    }))?,
                )?;
                fs::rename(
                    self.root.join("diagnostic-local-count.tmp"),
                    self.root.join("diagnostic-local-count.json"),
                )
            })();
            if written.is_err() {
                self.write_failed.store(true, Ordering::SeqCst);
            }
            (sequence, sequence <= LOCAL_POST_LIMIT)
        }
        fn record(
            &self,
            local: usize,
            call: usize,
            stage: Stage,
            stream_obtained: bool,
            error: Option<&ProviderError>,
        ) {
            use std::io::Write;
            let record = Observation {
                local_request: local,
                provider_call: call,
                stage,
                stream_obtained,
                error: error.map(safe_error),
            };
            let _lock = self.writer.lock().unwrap();
            let written = (|| -> std::io::Result<()> {
                let mut file = fs::OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(self.root.join("provider-diagnostics.jsonl"))?;
                serde_json::to_writer(&mut file, &record)?;
                file.write_all(b"\n")?;
                file.flush()
            })();
            if written.is_err() {
                self.write_failed.store(true, Ordering::SeqCst);
            }
        }
        fn begin(&self) -> (usize, usize) {
            let local = LOCAL_REQUEST.try_with(|id| *id).unwrap_or(0);
            let call = self.calls.fetch_add(1, Ordering::SeqCst) + 1;
            self.record(local, call, Stage::Started, false, None);
            (local, call)
        }
    }
    pub(super) async fn limit_local_posts(
        axum::extract::State(diagnostics): axum::extract::State<Arc<Diagnostics>>,
        request: axum::extract::Request,
        next: axum::middleware::Next,
    ) -> axum::response::Response {
        if request.method() == axum::http::Method::POST && request.uri().path() == "/v1/responses" {
            let (sequence, admitted) = diagnostics.local_request();
            if !admitted {
                return axum::response::Response::builder()
                    .status(503)
                    .body(axum::body::Body::from(
                        "private diagnostic request budget exhausted",
                    ))
                    .unwrap();
            }
            LOCAL_REQUEST.scope(sequence, next.run(request)).await
        } else {
            next.run(request).await
        }
    }
    pub(super) struct ObservedProvider<P> {
        pub(super) inner: P,
        pub(super) diagnostics: Arc<Diagnostics>,
    }
    #[async_trait::async_trait]
    impl<P: GatewayProvider> GatewayProvider for ObservedProvider<P> {
        fn supports_request(&self, request: &GatewayRequest) -> bool {
            self.inner.supports_request(request)
        }
        fn supports_model(&self, model: &str) -> bool {
            self.inner.supports_model(model)
        }
        async fn count_tokens(
            &self,
            request: CountTokensRequest,
        ) -> Result<CountTokensResponse, ProviderError> {
            self.inner.count_tokens(request).await
        }
        async fn send_search(&self, request: SearchRequest) -> Result<SearchResponse, SearchError> {
            self.inner.send_search(request).await
        }
        async fn generate_image(
            &self,
            request: ImageGenerationRequest,
            context: ImageContext,
        ) -> Result<ImageResult, ImageError> {
            self.inner.generate_image(request, context).await
        }
        async fn edit_image(
            &self,
            request: ImageEditRequest,
            context: ImageContext,
        ) -> Result<ImageResult, ImageError> {
            self.inner.edit_image(request, context).await
        }
        async fn send_message(
            &self,
            request: GatewayRequest,
        ) -> Result<GatewayResponse, ProviderError> {
            let (local, call) = self.diagnostics.begin();
            let result = self.inner.send_message(request).await;
            self.diagnostics
                .record(local, call, Stage::Returned, false, result.as_ref().err());
            result
        }
        async fn send_message_stream(
            &self,
            request: GatewayRequest,
        ) -> Result<GatewayStreamResponse, ProviderError> {
            let (local, call) = self.diagnostics.begin();
            let result = self.inner.send_message_stream(request).await;
            self.diagnostics.record(
                local,
                call,
                Stage::Returned,
                result.is_ok(),
                result.as_ref().err(),
            );
            result.map(|response| {
                let diagnostics = self.diagnostics.clone();
                GatewayStreamResponse {
                    headers: response.headers,
                    stream: Box::pin(response.stream.map(move |chunk| {
                        if let Err(error) = &chunk {
                            diagnostics.record(local, call, Stage::StreamError, true, Some(error));
                        }
                        chunk
                    })),
                }
            })
        }
    }
    #[cfg(test)]
    mod tests {
        use super::*;
        use bytes::Bytes;
        const SENTINEL: &str = "PRIVATE_TOKEN_ACCOUNT_HEADER_URL_PROMPT_BODY_REASONING_SENTINEL";
        fn request() -> GatewayRequest {
            serde_json::from_value(json!({"model":SENTINEL,"messages":[],"max_output_tokens":1,"stream":true,"instructions":SENTINEL})).unwrap()
        }
        fn diagnostics(root: &Path) -> Arc<Diagnostics> {
            Arc::new(Diagnostics::new(root.to_path_buf()))
        }
        fn read(root: &Path) -> Vec<Value> {
            fs::read_to_string(root.join("provider-diagnostics.jsonl"))
                .unwrap()
                .lines()
                .map(|line| serde_json::from_str(line).unwrap())
                .collect()
        }
        #[test]
        fn test_safe_categories_redact_all_untrusted_strings() {
            let builder = reqwest::Client::new()
                .get(format!("http://[invalid/{SENTINEL}"))
                .build()
                .unwrap_err();
            let errors = [
                ProviderError::AuthError(SENTINEL.into()),
                ProviderError::ModelNotSupported(SENTINEL.into()),
                ProviderError::ConfigError(SENTINEL.into()),
                ProviderError::ApiError {
                    status: 429,
                    message: SENTINEL.into(),
                },
                ProviderError::SerializationError(
                    serde_json::from_str::<u32>(&format!("\"{SENTINEL}\"")).unwrap_err(),
                ),
                ProviderError::HttpError(builder),
            ];
            let values: Vec<_> = errors
                .iter()
                .map(|e| serde_json::to_value(safe_error(e)).unwrap())
                .collect();
            assert_eq!(
                values
                    .iter()
                    .map(|v| v["category"].as_str().unwrap())
                    .collect::<Vec<_>>(),
                [
                    "authentication",
                    "model_unsupported",
                    "configuration",
                    "api",
                    "serialization",
                    "http"
                ]
            );
            assert_eq!(values[3]["status"], 429);
            assert_eq!(values[4]["json"][0], "data");
            assert_eq!(values[4]["json"][1], 1);
            assert_eq!(values[5]["http_flags"][2], true);
            assert!(!serde_json::to_string(&values).unwrap().contains(SENTINEL));
            assert!(values.iter().all(|v| v["check"].is_null()));
            let known = safe_error(&ProviderError::ConfigError(
                "Unsupported Responses history".into(),
            ));
            assert_eq!(known.check, Some(Check::ResponsesHistory));
            let injected = safe_error(&ProviderError::ConfigError(format!(
                "Unsupported Responses history {SENTINEL}"
            )));
            assert_eq!(injected.check, None);
            assert_eq!(
                safe_error(&ProviderError::ApiError {
                    status: 503,
                    message: "Upstream request rejected".into()
                })
                .check,
                Some(Check::UpstreamRejected)
            );
        }
        #[tokio::test]
        async fn test_http_and_json_categories_without_network() {
            let status = reqwest::Response::from(
                axum::http::Response::builder()
                    .status(503)
                    .body(SENTINEL)
                    .unwrap(),
            )
            .error_for_status()
            .unwrap_err();
            let safe = safe_error(&ProviderError::HttpError(status));
            assert_eq!(safe.status, Some(503));
            assert_eq!(safe.status_origin, Some(StatusOrigin::HttpError));
            let decode = reqwest::Response::from(axum::http::Response::new(SENTINEL))
                .json::<u32>()
                .await
                .unwrap_err();
            let safe = safe_error(&ProviderError::HttpError(decode));
            assert!(safe.http_flags.unwrap()[3]);
            for (raw, category) in [("{", JsonCategory::Eof), ("@", JsonCategory::Syntax)] {
                let e = serde_json::from_str::<Value>(raw).unwrap_err();
                assert_eq!(
                    safe_error(&ProviderError::SerializationError(e))
                        .json
                        .unwrap()
                        .0,
                    category
                );
            }
            struct FailedRead;
            impl std::io::Read for FailedRead {
                fn read(&mut self, _: &mut [u8]) -> std::io::Result<usize> {
                    Err(std::io::Error::other(SENTINEL))
                }
            }
            let e = serde_json::from_reader::<_, Value>(FailedRead).unwrap_err();
            assert_eq!(e.classify(), serde_json::error::Category::Io);
            assert!(
                !serde_json::to_string(&safe_error(&ProviderError::SerializationError(e)))
                    .unwrap()
                    .contains(SENTINEL)
            );
            assert_eq!(
                safe_error(&codex_transport_drift("unknown static check")).status_origin,
                Some(StatusOrigin::ProviderDeclared)
            );
        }
        #[tokio::test]
        async fn test_actual_openai_presend_error_remains_unchanged() {
            let _lock = integrated_codex_auth_test_lock().lock().await;
            let _no_credentials = InstalledHandoffGuard::set(None);
            let provider = || {
                OpenAIProvider::with_headers(OpenAIProviderConfig {
                    name: "offline-observer".into(),
                    api_key: String::new(),
                    base_url: "https://unused.invalid".into(),
                    models: vec![SENTINEL.into()],
                    custom_headers: vec![],
                    oauth_provider: Some("offline-observer".into()),
                    token_store: None,
                    codex_auth_source: Some(CodexAuthSource::Integrated),
                })
            };
            let root = tempfile::tempdir().unwrap();
            let observed = ObservedProvider {
                inner: provider(),
                diagnostics: diagnostics(root.path()),
            };
            let mut invalid = request();
            invalid.temperature = Some(0.2);
            let direct = provider()
                .send_message_stream(invalid.clone())
                .await
                .err()
                .expect("pre-send rejection");
            let wrapped = LOCAL_REQUEST
                .scope(1, observed.send_message_stream(invalid))
                .await
                .err()
                .expect("same pre-send rejection");
            match (direct, wrapped) {
                (ProviderError::ConfigError(a), ProviderError::ConfigError(b)) => assert_eq!(a, b),
                _ => panic!("actual provider rejection changed"),
            }
            let rows = read(root.path());
            assert_eq!(rows.len(), 2);
            assert_eq!(rows[1]["error"]["category"], "configuration");
            assert_eq!(rows[1]["error"]["check"], "temperature");
            assert_eq!(rows[1]["stream_obtained"], false);
        }
        struct Spy {
            seen: Arc<Mutex<Vec<(&'static str, Value)>>>,
            polls: Arc<AtomicUsize>,
            drops: Arc<AtomicUsize>,
        }
        struct RawStream {
            index: usize,
            polls: Arc<AtomicUsize>,
            drops: Arc<AtomicUsize>,
        }
        impl futures::Stream for RawStream {
            type Item = Result<Bytes, ProviderError>;
            fn poll_next(
                mut self: std::pin::Pin<&mut Self>,
                _cx: &mut std::task::Context<'_>,
            ) -> std::task::Poll<Option<Self::Item>> {
                self.polls.fetch_add(1, Ordering::SeqCst);
                let next = match self.index {
                    0 => Some(Ok(Bytes::from_static(SENTINEL.as_bytes()))),
                    1 => Some(Err(ProviderError::ApiError {
                        status: 418,
                        message: SENTINEL.into(),
                    })),
                    _ => None,
                };
                self.index += 1;
                std::task::Poll::Ready(next)
            }
        }
        impl Drop for RawStream {
            fn drop(&mut self) {
                self.drops.fetch_add(1, Ordering::SeqCst);
            }
        }
        impl Spy {
            fn record(&self, method: &'static str, value: Value) {
                self.seen.lock().unwrap().push((method, value));
            }
        }
        #[async_trait::async_trait]
        impl GatewayProvider for Spy {
            fn supports_model(&self, model: &str) -> bool {
                self.record("model", json!(model));
                model == SENTINEL
            }
            fn supports_request(&self, r: &GatewayRequest) -> bool {
                self.record("supports", serde_json::to_value(r).unwrap());
                false
            }
            async fn send_message(
                &self,
                r: GatewayRequest,
            ) -> Result<GatewayResponse, ProviderError> {
                self.record("message", serde_json::to_value(r).unwrap());
                Err(ProviderError::AuthError(SENTINEL.into()))
            }
            async fn send_message_stream(
                &self,
                r: GatewayRequest,
            ) -> Result<GatewayStreamResponse, ProviderError> {
                self.record("stream", serde_json::to_value(r).unwrap());
                Ok(GatewayStreamResponse {
                    headers: std::collections::HashMap::from([(
                        "private-header".into(),
                        SENTINEL.into(),
                    )]),
                    stream: Box::pin(RawStream {
                        index: 0,
                        polls: self.polls.clone(),
                        drops: self.drops.clone(),
                    }),
                })
            }
            async fn count_tokens(
                &self,
                r: CountTokensRequest,
            ) -> Result<CountTokensResponse, ProviderError> {
                self.record("tokens", serde_json::to_value(r).unwrap());
                Ok(CountTokensResponse { input_tokens: 123 })
            }
            async fn send_search(&self, r: SearchRequest) -> Result<SearchResponse, SearchError> {
                self.record("search", serde_json::to_value(r).unwrap());
                Ok(SearchResponse {
                    output: SENTINEL.into(),
                    results: None,
                    encrypted_output: None,
                })
            }
            async fn generate_image(
                &self,
                r: ImageGenerationRequest,
                c: ImageContext,
            ) -> Result<ImageResult, ImageError> {
                self.record("generate",json!({"request":r,"originator":c.originator,"turn_id":c.turn_id,"live_deadline":c.deadline>tokio::time::Instant::now()}));
                Err(ImageError::Busy)
            }
            async fn edit_image(
                &self,
                r: ImageEditRequest,
                c: ImageContext,
            ) -> Result<ImageResult, ImageError> {
                self.record("edit",json!({"request":r,"originator":c.originator,"turn_id":c.turn_id,"live_deadline":c.deadline>tokio::time::Instant::now()}));
                Err(ImageError::Limit)
            }
        }
        #[tokio::test]
        async fn test_all_methods_and_stream_are_forwarded_unchanged() {
            let root = tempfile::tempdir().unwrap();
            let seen = Arc::new(Mutex::new(Vec::new()));
            let polls = Arc::new(AtomicUsize::new(0));
            let drops = Arc::new(AtomicUsize::new(0));
            let observed = ObservedProvider {
                inner: Spy {
                    seen: seen.clone(),
                    polls: polls.clone(),
                    drops: drops.clone(),
                },
                diagnostics: diagnostics(root.path()),
            };
            assert!(observed.supports_model(SENTINEL));
            assert!(!observed.supports_request(&request()));
            match observed.send_message(request()).await {
                Err(ProviderError::AuthError(text)) => assert_eq!(text, SENTINEL),
                _ => panic!("result changed"),
            }
            let mut stream = LOCAL_REQUEST
                .scope(17, observed.send_message_stream(request()))
                .await
                .unwrap();
            assert_eq!(polls.load(Ordering::SeqCst), 0);
            assert_eq!(stream.headers["private-header"], SENTINEL);
            assert_eq!(
                stream.stream.next().await.unwrap().unwrap(),
                Bytes::from_static(SENTINEL.as_bytes())
            );
            match stream.stream.next().await.unwrap() {
                Err(ProviderError::ApiError { status, message }) => {
                    assert_eq!(status, 418);
                    assert_eq!(message, SENTINEL);
                }
                _ => panic!("stream error changed"),
            }
            assert!(stream.stream.next().await.is_none());
            assert_eq!(polls.load(Ordering::SeqCst), 3);
            drop(stream);
            assert_eq!(drops.load(Ordering::SeqCst), 1);
            let unpolled = observed.send_message_stream(request()).await.unwrap();
            drop(unpolled);
            assert_eq!(polls.load(Ordering::SeqCst), 3);
            assert_eq!(drops.load(Ordering::SeqCst), 2);
            assert_eq!(
                observed
                    .count_tokens(
                        serde_json::from_value(json!({"model":SENTINEL,"messages":[]})).unwrap()
                    )
                    .await
                    .unwrap()
                    .input_tokens,
                123
            );
            assert_eq!(
                observed
                    .send_search(
                        serde_json::from_value(json!({"id":SENTINEL,"model":SENTINEL})).unwrap()
                    )
                    .await
                    .unwrap()
                    .output,
                SENTINEL
            );
            let context = || {
                let mut c =
                    ImageContext::new(tokio::time::Instant::now() + Duration::from_secs(30));
                c.originator = Some(SENTINEL.into());
                c.turn_id = Some(SENTINEL.into());
                c
            };
            assert!(matches!(
                observed
                    .generate_image(
                        serde_json::from_value(json!({"model":SENTINEL,"prompt":SENTINEL}))
                            .unwrap(),
                        context()
                    )
                    .await,
                Err(ImageError::Busy)
            ));
            assert!(matches!(
                observed
                    .edit_image(
                        serde_json::from_value(
                            json!({"model":SENTINEL,"prompt":SENTINEL,"images":[]})
                        )
                        .unwrap(),
                        context()
                    )
                    .await,
                Err(ImageError::Limit)
            ));
            let calls = seen.lock().unwrap();
            assert_eq!(
                calls.iter().map(|(m, _)| *m).collect::<Vec<_>>(),
                [
                    "model", "supports", "message", "stream", "stream", "tokens", "search",
                    "generate", "edit"
                ]
            );
            for method in ["supports", "message", "stream"] {
                assert_eq!(
                    calls.iter().find(|(m, _)| *m == method).unwrap().1,
                    serde_json::to_value(request()).unwrap()
                );
            }
            for method in ["generate", "edit"] {
                let v = &calls.iter().find(|(m, _)| *m == method).unwrap().1;
                assert_eq!(v["originator"], SENTINEL);
                assert_eq!(v["turn_id"], SENTINEL);
                assert_eq!(v["live_deadline"], true);
                assert_eq!(v["request"]["prompt"], SENTINEL);
            }
            let records = read(root.path());
            assert!(records.iter().any(|v| v["local_request"] == 17
                && v["stream_obtained"] == true
                && v["stage"] == "stream_error"
                && v["error"]["status"] == 418));
            assert!(
                !fs::read_to_string(root.path().join("provider-diagnostics.jsonl"))
                    .unwrap()
                    .contains(SENTINEL)
            );
        }
        #[tokio::test]
        async fn test_sink_failure_does_not_replace_provider_result() {
            let root = tempfile::tempdir().unwrap();
            let observed = ObservedProvider {
                inner: Spy {
                    seen: Arc::new(Mutex::new(Vec::new())),
                    polls: Arc::new(AtomicUsize::new(0)),
                    drops: Arc::new(AtomicUsize::new(0)),
                },
                diagnostics: diagnostics(&root.path().join("absent")),
            };
            assert!(matches!(
                observed.send_message(request()).await,
                Err(ProviderError::AuthError(_))
            ));
            assert!(observed.diagnostics.write_failed());
        }
        #[tokio::test]
        async fn test_actual_middleware_caps_forwarded_local_requests_and_correlates() {
            use tower::Service;
            let root = tempfile::tempdir().unwrap();
            let diagnostics = diagnostics(root.path());
            let seen = Arc::new(AtomicUsize::new(0));
            let count = seen.clone();
            let router = axum::Router::new()
                .route(
                    "/v1/responses",
                    axum::routing::post(move || {
                        let count = count.clone();
                        async move {
                            assert_eq!(LOCAL_REQUEST.with(|id| *id), 1);
                            count.fetch_add(1, Ordering::SeqCst);
                            "unchanged"
                        }
                    }),
                )
                .layer(axum::middleware::from_fn_with_state(
                    diagnostics.clone(),
                    limit_local_posts,
                ));
            let responses = futures::future::join_all((0..20).map(|_| {
                router.clone().call(
                    axum::http::Request::builder()
                        .method("POST")
                        .uri("/v1/responses")
                        .body(axum::body::Body::empty())
                        .unwrap(),
                )
            }))
            .await;
            let statuses: Vec<_> = responses
                .into_iter()
                .map(|r| r.unwrap().status().as_u16())
                .collect();
            assert_eq!(statuses.iter().filter(|s| **s == 200).count(), 1);
            assert_eq!(statuses.iter().filter(|s| **s == 503).count(), 19);
            assert_eq!(seen.load(Ordering::SeqCst), 1);
            let saved: Value = serde_json::from_slice(
                &fs::read(root.path().join("diagnostic-local-count.json")).unwrap(),
            )
            .unwrap();
            assert_eq!(
                saved,
                json!({"local_posts":20,"admitted_limit":1,"rejected_local_posts":19})
            );
        }
    }
}
