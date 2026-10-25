//! Managed-only paired image routes. Existing E3 admission runs before extraction.
use crate::models::images::{
    bounded_json, check_deadline, checked_add, preflight_json, validate_label, ImageContext,
    ImageEditRequest, ImageError, ImageGenerationRequest, REQUEST_ID_HEADER, REQUEST_LIMIT,
};
use crate::providers::GatewayProvider;
use axum::{
    body::Body,
    extract::{Extension, Request},
    http::{header, HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use bytes::Bytes;
use futures::{Stream, StreamExt};
use std::{
    pin::Pin,
    sync::Arc,
    task::{Context, Poll},
    time::Duration,
};
use tokio::sync::{OwnedSemaphorePermit, Semaphore};

pub(super) struct ImageState {
    selected: Option<Arc<Box<dyn GatewayProvider>>>,
    permits: Arc<Semaphore>,
    deadline: Duration,
}
impl ImageState {
    pub(super) fn new(snapshot: &super::ReloadableState) -> Self {
        // Choose the held account's provider using the configured default once.
        // The inference actual_model and catalogue do not constrain image models.
        let selected = snapshot
            .config
            .models
            .iter()
            .find(|m| m.name == snapshot.config.router.default)
            .and_then(|m| m.mappings.first())
            .and_then(|m| {
                let p = snapshot
                    .config
                    .providers
                    .iter()
                    .find(|p| p.name == m.provider)?;
                if p.provider_type != "openai"
                    || p.auth_type != crate::providers::AuthType::OAuth
                    || !p.is_enabled()
                {
                    return None;
                }
                snapshot.provider_registry.get_provider(&m.provider)
            });
        Self {
            selected,
            permits: Arc::new(Semaphore::new(1)),
            deadline: Duration::from_secs(180),
        }
    }
}

pub(super) async fn generate(
    Extension(state): Extension<Arc<ImageState>>,
    request: Request,
) -> Response {
    handle(state, request, false).await
}
pub(super) async fn edit(
    Extension(state): Extension<Arc<ImageState>>,
    request: Request,
) -> Response {
    handle(state, request, true).await
}
fn label(headers: &HeaderMap, name: &str) -> Result<Option<String>, ImageError> {
    let mut values = headers.get_all(name).iter();
    let Some(value) = values.next() else {
        return Ok(None);
    };
    if values.next().is_some() {
        return Err(ImageError::InvalidRequest);
    }
    let s = value.to_str().map_err(|_| ImageError::InvalidRequest)?;
    validate_label(s)?;
    Ok(Some(s.to_owned()))
}
async fn handle(state: Arc<ImageState>, request: Request, edit: bool) -> Response {
    let permit = match Arc::clone(&state.permits).try_acquire_owned() {
        Ok(p) => p,
        Err(_) => return ImageError::Busy.into_response(),
    };
    let deadline = tokio::time::Instant::now() + state.deadline;
    let operation = async {
        let provider = state.selected.as_ref().ok_or(ImageError::Unsupported)?;
        if request.uri().query().is_some() {
            return Err(ImageError::InvalidRequest);
        }
        for name in [
            "origin",
            "x-provider",
            "x-base-url",
            "x-account-id",
            "authorization",
            "api-key",
            "x-api-key",
            "chatgpt-account-id",
            "x-openai-actor-authorization",
        ] {
            if request.headers().contains_key(name) {
                return Err(ImageError::Unsupported);
            }
        }
        let mut context = ImageContext::new(deadline);
        context.turn_id = label(request.headers(), "x-codex-image-turn-id")?;
        context.originator = label(request.headers(), "originator")?;
        context.validate()?;
        let mut content_types = request.headers().get_all(header::CONTENT_TYPE).iter();
        let ct = content_types
            .next()
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.split(';').next())
            .map(str::trim);
        if content_types.next().is_some() || ct != Some("application/json") {
            return Err(ImageError::InvalidRequest);
        }
        let mut lengths = request.headers().get_all(header::CONTENT_LENGTH).iter();
        if lengths.next().is_some_and(|v| {
            v.to_str()
                .ok()
                .and_then(|s| s.parse::<usize>().ok())
                .is_none_or(|n| n > REQUEST_LIMIT)
        }) || lengths.next().is_some()
        {
            return Err(ImageError::Limit);
        }
        let mut stream = request.into_body().into_data_stream();
        let mut bytes = Vec::new();
        while let Some(chunk) = stream.next().await {
            check_deadline(deadline)?;
            let chunk = chunk.map_err(|_| ImageError::InvalidRequest)?;
            checked_add(bytes.len(), chunk.len(), REQUEST_LIMIT)?;
            bytes
                .try_reserve_exact(chunk.len())
                .map_err(|_| ImageError::Limit)?;
            bytes.extend_from_slice(&chunk);
        }
        preflight_json(&bytes, false)?;
        check_deadline(deadline)?;
        let result = if edit {
            let parsed: ImageEditRequest =
                serde_json::from_slice(&bytes).map_err(|_| ImageError::InvalidRequest)?;
            drop(bytes);
            provider.edit_image(parsed, context).await?
        } else {
            let parsed: ImageGenerationRequest =
                serde_json::from_slice(&bytes).map_err(|_| ImageError::InvalidRequest)?;
            drop(bytes);
            provider.generate_image(parsed, context).await?
        };
        check_deadline(deadline)?;
        let bytes = bounded_json(&result.response, true)?;
        drop(result.response);
        check_deadline(deadline)?;
        Ok::<_, ImageError>((Bytes::from(bytes), result.request_id))
    };
    match tokio::time::timeout_at(deadline, operation).await {
        Ok(Ok(_)) if tokio::time::Instant::now() >= deadline => {
            ImageError::Deadline.into_response()
        }
        Ok(Ok((bytes, request_id))) => {
            let mut response = (
                [(header::CONTENT_TYPE, "application/json")],
                Body::from_stream(PermittedBody {
                    bytes: Some(bytes),
                    _permit: permit,
                }),
            )
                .into_response();
            if let Some(id) = request_id {
                match id.parse() {
                    Ok(value) => {
                        response.headers_mut().insert(REQUEST_ID_HEADER, value);
                    }
                    Err(_) => return ImageError::InvalidResponse.into_response(),
                }
            }
            response
        }
        Ok(Err(e)) => e.into_response(),
        Err(_) => ImageError::Deadline.into_response(),
    }
}
// The public body owns capacity even after yielding its data, until consume/drop.
struct PermittedBody {
    bytes: Option<Bytes>,
    _permit: OwnedSemaphorePermit,
}
impl Stream for PermittedBody {
    type Item = Result<Bytes, std::io::Error>;
    fn poll_next(mut self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        Poll::Ready(self.bytes.take().map(Ok))
    }
}
impl IntoResponse for ImageError {
    fn into_response(self) -> Response {
        let (status, code) = match self {
            Self::Unsupported => (StatusCode::FORBIDDEN, "image_not_admitted"),
            Self::Authentication => (StatusCode::SERVICE_UNAVAILABLE, "image_auth_unavailable"),
            Self::InvalidRequest => (StatusCode::BAD_REQUEST, "invalid_image_request"),
            Self::Limit => (StatusCode::PAYLOAD_TOO_LARGE, "image_limit_exceeded"),
            Self::Busy => (StatusCode::TOO_MANY_REQUESTS, "image_busy"),
            Self::Deadline => (StatusCode::GATEWAY_TIMEOUT, "image_deadline_exceeded"),
            Self::Transport => (StatusCode::BAD_GATEWAY, "image_outcome_unknown"),
            Self::UpstreamDenied => (StatusCode::FORBIDDEN, "image_upstream_denied"),
            Self::RateLimited => (StatusCode::TOO_MANY_REQUESTS, "image_upstream_rate_limited"),
            Self::Upstream => (StatusCode::BAD_GATEWAY, "image_upstream_failed"),
            Self::InvalidResponse => (StatusCode::BAD_GATEWAY, "invalid_image_response"),
        };
        (
            status,
            Json(serde_json::json!({"error":{"code":code,"message":self.to_string()}})),
        )
            .into_response()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::auth::codex_auth_context::{CodexIntegratedAuthHandoff, InstalledHandoffGuard};
    use crate::models::images::{
        FILE_LIMIT, INPUT_LIMIT, LABEL_LIMIT, OUTPUT_LIMIT, RESPONSE_LIMIT,
    };
    use crate::providers::{OpenAIProvider, OpenAIProviderConfig, ProviderRegistry};
    use axum::{body::to_bytes, http::Request as HttpRequest};
    use base64::{engine::general_purpose::STANDARD, Engine as _};
    use serde_json::{json, Value};
    use std::{
        collections::VecDeque,
        sync::{
            atomic::{AtomicUsize, Ordering},
            Mutex,
        },
    };
    use tokio::{
        io::{AsyncReadExt, AsyncWriteExt},
        net::TcpListener,
    };
    use tower::Service;

    // These tests use the actual kernel-readback middleware without changing the
    // parent test process's dumpability or core limits.
    fn isolated(name: &str, test: impl std::future::Future<Output = ()>) {
        if std::env::var_os("CODEX_IMAGES_TEST_CHILD").is_none() {
            let output = std::process::Command::new(std::env::current_exe().unwrap())
                .args(["--exact", &format!("server::codex_images::tests::{name}")])
                .env("CODEX_IMAGES_TEST_CHILD", "1")
                .output()
                .unwrap();
            assert!(
                output.status.success(),
                "{}\n{}",
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
            );
            assert!(String::from_utf8_lossy(&output.stdout).contains("1 passed"));
            return;
        }
        let limits = nix::libc::rlimit {
            rlim_cur: 0,
            rlim_max: 0,
        };
        assert_eq!(
            unsafe { nix::libc::setrlimit(nix::libc::RLIMIT_CORE, &limits) },
            0
        );
        assert_eq!(
            unsafe { nix::libc::prctl(nix::libc::PR_SET_DUMPABLE, 0, 0, 0, 0) },
            0
        );
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap()
            .block_on(test);
    }

    enum Plan {
        Reply {
            status: u16,
            content_type: &'static str,
            chunks: Vec<Vec<u8>>,
            declared: Option<usize>,
            location: Option<String>,
        },
        Hold,
        Chunked(Vec<Vec<u8>>),
        Raw(Vec<u8>),
    }
    fn reply(value: &Value) -> Plan {
        Plan::Reply {
            status: 200,
            content_type: "application/json",
            chunks: vec![serde_json::to_vec(value).unwrap()],
            declared: None,
            location: None,
        }
    }
    struct Upstream {
        url: url::Url,
        plans: Arc<Mutex<VecDeque<Plan>>>,
        seen: Arc<Mutex<Vec<(String, Value)>>>,
        closed: Arc<AtomicUsize>,
        task: tokio::task::JoinHandle<()>,
    }
    impl Drop for Upstream {
        fn drop(&mut self) {
            self.task.abort();
        }
    }
    impl Upstream {
        async fn new() -> Self {
            let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
            let url = format!("http://{}/fixture-only", listener.local_addr().unwrap())
                .parse()
                .unwrap();
            let plans = Arc::new(Mutex::new(VecDeque::new()));
            let seen = Arc::new(Mutex::new(Vec::new()));
            let closed = Arc::new(AtomicUsize::new(0));
            let (p, s, c) = (plans.clone(), seen.clone(), closed.clone());
            let task = tokio::spawn(async move {
                loop {
                    let (mut socket, _) = listener.accept().await.unwrap();
                    let (p, s, c) = (p.clone(), s.clone(), c.clone());
                    tokio::spawn(async move {
                        let mut bytes = Vec::new();
                        let split = loop {
                            let mut chunk = [0u8; 4096];
                            let n = socket.read(&mut chunk).await.unwrap();
                            assert_ne!(n, 0);
                            bytes.extend_from_slice(&chunk[..n]);
                            if let Some(i) = bytes.windows(4).position(|b| b == b"\r\n\r\n") {
                                break i + 4;
                            }
                            assert!(bytes.len() < 32768);
                        };
                        let headers = String::from_utf8(bytes[..split].to_vec()).unwrap();
                        let length: usize = headers
                            .lines()
                            .find_map(|line| {
                                line.to_ascii_lowercase()
                                    .strip_prefix("content-length:")
                                    .map(|v| v.trim().parse().unwrap())
                            })
                            .unwrap();
                        while bytes.len() - split < length {
                            let mut chunk = [0u8; 4096];
                            let n = socket.read(&mut chunk).await.unwrap();
                            assert_ne!(n, 0);
                            bytes.extend_from_slice(&chunk[..n]);
                        }
                        let value = serde_json::from_slice(&bytes[split..split + length]).unwrap();
                        s.lock().unwrap().push((headers, value));
                        let plan = p
                            .lock()
                            .unwrap()
                            .pop_front()
                            .expect("unexpected upstream I/O");
                        match plan {
                            Plan::Raw(raw) => {
                                let _ = socket.write_all(&raw).await;
                            }
                            Plan::Chunked(chunks) => {
                                if socket.write_all(b"HTTP/1.1 200 Fixture\r\nContent-Type: application/json\r\nTransfer-Encoding: chunked\r\nConnection: close\r\n\r\n").await.is_err() { return; }
                                for chunk in chunks {
                                    if socket
                                        .write_all(format!("{:x}\r\n", chunk.len()).as_bytes())
                                        .await
                                        .is_err()
                                        || socket.write_all(&chunk).await.is_err()
                                        || socket.write_all(b"\r\n").await.is_err()
                                    {
                                        return;
                                    }
                                }
                                let _ = socket.write_all(b"0\r\n\r\n").await;
                            }

                            Plan::Hold => {
                                let mut byte = [0];
                                // Success is actual socket EOF/reset, not a timer or task abort.
                                let read = socket.read(&mut byte).await;
                                assert!(matches!(read, Ok(0) | Err(_)));
                                c.fetch_add(1, Ordering::SeqCst);
                            }
                            Plan::Reply {
                                status,
                                content_type,
                                chunks,
                                declared,
                                location,
                            } => {
                                let length =
                                    declared.unwrap_or_else(|| chunks.iter().map(Vec::len).sum());
                                let location = location
                                    .map(|s| format!("Location: {s}\r\n"))
                                    .unwrap_or_default();
                                let header = format!("HTTP/1.1 {status} Fixture\r\nContent-Type: {content_type}\r\nContent-Length: {length}\r\n{location}Connection: close\r\n\r\n");
                                if socket.write_all(header.as_bytes()).await.is_err() {
                                    return;
                                }
                                for chunk in chunks {
                                    if socket.write_all(&chunk).await.is_err() {
                                        break;
                                    }
                                    if socket.flush().await.is_err() {
                                        break;
                                    }
                                    tokio::time::sleep(Duration::from_millis(2)).await;
                                }
                            }
                        }
                    });
                }
            });
            Self {
                url,
                plans,
                seen,
                closed,
                task,
            }
        }
        fn push(&self, plan: Plan) {
            self.plans.lock().unwrap().push_back(plan);
        }
        fn count(&self) -> usize {
            self.seen.lock().unwrap().len()
        }
    }
    async fn until(mut condition: impl FnMut() -> bool) {
        tokio::time::timeout(Duration::from_secs(2), async {
            while !condition() {
                tokio::time::sleep(Duration::from_millis(2)).await;
            }
        })
        .await
        .expect("observable socket/permit transition timed out");
    }
    fn held_auth() -> InstalledHandoffGuard {
        InstalledHandoffGuard::set(Some(CodexIntegratedAuthHandoff::new(
            Some("synthetic-account".into()),
            secrecy::SecretString::new("synthetic-held-token".into()),
        )))
    }
    fn state(upstream: &Upstream) -> Arc<super::super::AppState> {
        let config: crate::cli::AppConfig = toml::from_str(
            r#"
[server]
host = "127.0.0.1"
port = 3000
log_level = "info"
[router]
default = "member-alias"
[[providers]]
name = "held-codex"
provider_type = "openai"
auth_type = "oauth"
oauth_provider = "held-codex"
models = ["actual-inference-slug"]
[[models]]
name = "member-alias"
[[models.mappings]]
priority = 1
provider = "held-codex"
actual_model = "actual-inference-slug"
"#,
        )
        .unwrap();
        let mut provider = OpenAIProvider::with_headers(OpenAIProviderConfig {
            name: "held-codex".into(),
            api_key: "must-never-be-used".into(),
            base_url: "http://unselected.invalid".into(),
            models: vec!["actual-inference-slug".into()],
            custom_headers: Vec::new(),
            oauth_provider: Some("held-codex".into()),
            token_store: None,
            codex_auth_source: Some(crate::auth::CodexAuthSource::Integrated),
        });
        provider.set_images_loopback_for_test(upstream.url.clone());
        let mut registry = ProviderRegistry::new();
        registry.insert_provider_for_tests("held-codex", Box::new(provider));
        Arc::new(super::super::AppState {
            inner: std::sync::RwLock::new(Arc::new(super::super::ReloadableState {
                router: crate::router::Router::new(config.clone()),
                config: config.clone(),
                provider_registry: Arc::new(registry),
            })),
            token_store: crate::auth::TokenStore::disabled(),
            message_tracer: Arc::new(crate::message_tracing::MessageTracer::new(
                config.server.tracing,
            )),
        })
    }
    fn input() -> Arc<config_projection::ManagedGatewayLaunchInputV1> {
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        Arc::new(crate::launch::e3_readiness_tests::input(&listener))
    }
    fn managed(state: Arc<super::super::AppState>) -> axum::Router {
        super::super::build_e3_in_world_app(
            state,
            input(),
            "00".repeat(32),
            Arc::new(std::sync::atomic::AtomicBool::new(false)),
        )
    }
    fn request(body: Body) -> HttpRequest<Body> {
        HttpRequest::builder()
            .method("POST")
            .uri("/v1/images/generations")
            .header("Content-Type", "application/json")
            .header("X-Substrate-Orchestration-Session", "e3-test-session")
            .header("X-Substrate-Participant", "e3-test-member")
            .header("X-Substrate-Projection", "44".repeat(32))
            .body(body)
            .unwrap()
    }
    fn minimal() -> Value {
        json!({"prompt":"draw red","model":"gpt-image-2","background":"auto","quality":"auto","size":"auto"})
    }
    fn json_request(value: &Value) -> HttpRequest<Body> {
        request(Body::from(serde_json::to_vec(value).unwrap()))
    }
    async fn response_json(response: Response) -> Value {
        let bytes = to_bytes(response.into_body(), RESPONSE_LIMIT)
            .await
            .unwrap();
        serde_json::from_slice(&bytes).unwrap()
    }

    const PNG: &str = "iVBORw0KGgoAAAANSUhEUgAAAAIAAAACCAYAAABytg0kAAAAFElEQVQI12P8z8Dwn4GBgYGJAQoAHxcCAmep4oIAAAAASUVORK5CYII=";
    fn output() -> Value {
        json!({"created":1720000000u64,"data":[{"b64_json":PNG,"generation_id":"gen-native"}],"background":"transparent","quality":"high","size":"2x2"})
    }
    fn edit_request(refs: Vec<Value>) -> HttpRequest<Body> {
        let mut r = json_request(
            &json!({"model":"gpt-image-2","prompt":"edit red","background":"auto","quality":"auto","size":"auto","images":refs}),
        );
        *r.uri_mut() = "/v1/images/edits".parse().unwrap();
        r
    }
    fn inline(mime: &str, bytes: &[u8]) -> Value {
        json!({"image_url":format!("data:{mime};base64,{}",STANDARD.encode(bytes))})
    }

    #[test]
    fn managed_images_literal_generate_edit_and_retained_loop() {
        isolated(
            "managed_images_literal_generate_edit_and_retained_loop",
            async {
                let _auth = held_auth();
                let upstream = Upstream::new().await;
                let app_state = state(&upstream);
                let app = managed(app_state.clone());
                let raw = format!(
                    r#"{{"created":1720000000,"data":[{{"b64_json":"{PNG}","generation_id":"gen-native"}},{{"b64_json":"{PNG}","generation_id":null}}],"background":null,"quality":"high","size":"2x2","usage":{{"private":"unconsumed-sentinel"}}}}"#
                );
                upstream.push(Plan::Raw(format!("HTTP/1.1 200 Fixture\r\nContent-Type: application/json\r\nContent-Length: {}\r\nX-Codex-Imagegen-Request-Id: req-native\r\nX-Secret: unconsumed-sentinel\r\nConnection: close\r\n\r\n{raw}", raw.len()).into_bytes()));
                let literal = r#"{"model":"gpt-image-2","prompt":"draw α","background":"auto","quality":"auto","size":"auto"}"#;
                let mut req = request(Body::from(literal));
                req.headers_mut()
                    .insert("x-codex-image-turn-id", "native-turn".parse().unwrap());
                req.headers_mut()
                    .insert("originator", "codex_cli_rs".parse().unwrap());
                req.headers_mut()
                    .insert("x-unused", "unconsumed-sentinel".parse().unwrap());
                let response = app.clone().call(req).await.unwrap();
                assert_eq!(response.status(), StatusCode::OK);
                assert_eq!(response.headers()[REQUEST_ID_HEADER], "req-native");
                assert!(!response.headers().contains_key("x-secret"));
                let value = response_json(response).await;
                assert_eq!(
                    value,
                    json!({"created":1720000000u64,"data":[{"b64_json":PNG,"generation_id":"gen-native"},{"b64_json":PNG,"generation_id":null}],"background":null,"quality":"high","size":"2x2"})
                );
                let compressed = STANDARD
                    .decode(value["data"][0]["b64_json"].as_str().unwrap())
                    .unwrap();
                assert_eq!(
                    compressed,
                    include_bytes!("../../tests/fixtures/codex_images/red.png")
                );
                let pixels = image::load_from_memory(&compressed).unwrap().to_rgba8();
                assert_eq!(pixels.dimensions(), (2, 2));
                assert_eq!(
                    pixels.as_raw(),
                    &[255, 0, 0, 255, 255, 0, 0, 255, 255, 0, 0, 255, 255, 0, 0, 255]
                );
                let refs = vec![
                    inline(
                        "image/jpeg",
                        include_bytes!("../../tests/fixtures/codex_images/red.jpg"),
                    ),
                    inline(
                        "image/webp",
                        include_bytes!("../../tests/fixtures/codex_images/red.webp"),
                    ),
                    inline("image/png", &compressed),
                    json!({"image_url":"https://never-fetched.invalid/reference?a=1"}),
                ];
                upstream.push(reply(&output()));
                let response = app.clone().call(edit_request(refs.clone())).await.unwrap();
                assert_eq!(response.status(), StatusCode::OK);
                let edit = response_json(response).await;
                // Retained native image references reuse inline original PNG, never an ID lookup.
                upstream.push(reply(&output()));
                let response = app.clone().call(edit_request(vec![json!({"image_url":format!("data:image/png;base64,{}",edit["data"][0]["b64_json"].as_str().unwrap())})])).await.unwrap();
                assert_eq!(response.status(), StatusCode::OK);
                drop(response);
                upstream.push(reply(&json!({"created":0,"data":[{"b64_json":PNG}]})));
                let response = app
                    .clone()
                    .call(request(Body::from(
                        r#"{"model":"gpt-image-2","prompt":"omitted defaults"}"#,
                    )))
                    .await
                    .unwrap();
                assert_eq!(response.status(), StatusCode::OK);
                assert_eq!(
                    response_json(response).await,
                    json!({"created":0,"data":[{"b64_json":PNG}]})
                );
                {
                    let seen = upstream.seen.lock().unwrap();
                    assert_eq!(seen.len(), 4);
                    assert_eq!(
                        seen[0].1,
                        json!({"model":"gpt-image-2","prompt":"draw α","background":"auto","quality":"auto","size":"auto"})
                    );
                    assert_eq!(
                        seen[1].1,
                        json!({"model":"gpt-image-2","prompt":"edit red","background":"auto","quality":"auto","size":"auto","images":refs})
                    );
                    assert_eq!(
                        seen[2].1["images"][0],
                        json!({"image_url":format!("data:image/png;base64,{PNG}")})
                    );
                    assert_eq!(
                        seen[3].1,
                        json!({"model":"gpt-image-2","prompt":"omitted defaults"})
                    );
                    for (i, (headers, _)) in seen.iter().enumerate() {
                        let lower = headers.to_ascii_lowercase();
                        assert!(lower.starts_with(if i == 1 || i == 2 {
                            "post /backend-api/codex/images/edits http/1.1"
                        } else {
                            "post /backend-api/codex/images/generations http/1.1"
                        }));
                        assert!(lower.contains("authorization: bearer synthetic-held-token\r\n"));
                        assert!(lower.contains("chatgpt-account-id: synthetic-account\r\n"));
                        for s in [
                            "x-substrate",
                            "unconsumed-sentinel",
                            "must-never-be-used",
                            "generation_id",
                            "x-provider",
                        ] {
                            assert!(!lower.contains(s));
                        }
                    }
                    assert!(seen[0]
                        .0
                        .to_ascii_lowercase()
                        .contains("x-codex-image-turn-id: native-turn\r\n"));
                    assert!(seen[0]
                        .0
                        .to_ascii_lowercase()
                        .contains("originator: codex_cli_rs\r\n"));
                }
                for path in ["/v1/images/generations", "/v1/images/edits"] {
                    let mut req = json_request(&minimal());
                    *req.uri_mut() = path.parse().unwrap();
                    assert_eq!(
                        super::super::build_app(app_state.clone())
                            .call(req)
                            .await
                            .unwrap()
                            .status(),
                        StatusCode::NOT_FOUND
                    );
                }
            },
        );
    }

    #[test]
    fn managed_images_admission_and_zero_io() {
        isolated("managed_images_admission_and_zero_io", async {
            let _auth = held_auth();
            let upstream = Upstream::new().await;
            let app_state = state(&upstream);
            let app = managed(app_state.clone());
            for path in ["/v1/images/generations", "/v1/images/edits"] {
                for header in [
                    "X-Substrate-Orchestration-Session",
                    "X-Substrate-Participant",
                    "X-Substrate-Projection",
                ] {
                    for mode in 0..3 {
                        let body = Body::from_stream(futures::stream::poll_fn(
                            |_| -> Poll<Option<Result<Bytes, std::io::Error>>> {
                                panic!("denied body polled")
                            },
                        ));
                        let mut req = request(body);
                        *req.uri_mut() = path.parse().unwrap();
                        match mode {
                            0 => {
                                req.headers_mut().remove(header);
                            }
                            1 => {
                                req.headers_mut().insert(header, "wrong".parse().unwrap());
                            }
                            _ => {
                                req.headers_mut()
                                    .append(header, "duplicate".parse().unwrap());
                            }
                        }
                        assert_eq!(
                            app.clone().call(req).await.unwrap().status(),
                            StatusCode::FORBIDDEN
                        );
                    }
                }
                for method in ["GET", "HEAD", "PUT", "PATCH", "DELETE", "OPTIONS", "TRACE"] {
                    let mut req = json_request(&minimal());
                    *req.uri_mut() = path.parse().unwrap();
                    *req.method_mut() = method.parse().unwrap();
                    assert_eq!(
                        app.clone().call(req).await.unwrap().status(),
                        StatusCode::METHOD_NOT_ALLOWED
                    );
                }
                for header in [
                    "origin",
                    "x-provider",
                    "x-base-url",
                    "x-account-id",
                    "authorization",
                    "api-key",
                    "x-api-key",
                    "chatgpt-account-id",
                    "x-openai-actor-authorization",
                ] {
                    let mut req = json_request(&minimal());
                    *req.uri_mut() = path.parse().unwrap();
                    req.headers_mut().insert(
                        axum::http::HeaderName::from_bytes(header.as_bytes()).unwrap(),
                        "sentinel".parse().unwrap(),
                    );
                    assert_eq!(
                        app.clone().call(req).await.unwrap().status(),
                        StatusCode::FORBIDDEN
                    );
                }
            }
            for path in [
                "/v1/images/generations/",
                "/v1/images/edits/extra",
                "/images/generations",
                "/images/edits",
                "/v1/images",
                "/v1/images/variations",
            ] {
                let mut req = json_request(&minimal());
                *req.uri_mut() = path.parse().unwrap();
                assert_eq!(
                    app.clone().call(req).await.unwrap().status(),
                    StatusCode::NOT_FOUND
                );
            }
            for path in [
                "/v1/images/generations?base_url=https://sentinel.invalid",
                "/v1/images/edits?provider=sentinel",
            ] {
                let mut req = json_request(&minimal());
                *req.uri_mut() = path.parse().unwrap();
                assert_eq!(
                    app.clone().call(req).await.unwrap().status(),
                    StatusCode::BAD_REQUEST
                );
            }
            for header in ["originator", "x-codex-image-turn-id"] {
                let mut req = json_request(&minimal());
                req.headers_mut().append(header, "one".parse().unwrap());
                req.headers_mut().append(header, "two".parse().unwrap());
                assert_eq!(
                    app.clone().call(req).await.unwrap().status(),
                    StatusCode::BAD_REQUEST
                );
                let mut req = json_request(&minimal());
                req.headers_mut()
                    .insert(header, "x".repeat(LABEL_LIMIT + 1).parse().unwrap());
                assert_eq!(
                    app.clone().call(req).await.unwrap().status(),
                    StatusCode::PAYLOAD_TOO_LARGE
                );
            }
            for provider_type in ["anthropic", "gemini", "azure-openai"] {
                let snapshot = app_state.snapshot();
                let mut changed = super::super::ReloadableState {
                    config: snapshot.config.clone(),
                    router: snapshot.router.clone(),
                    provider_registry: snapshot.provider_registry.clone(),
                };
                changed.config.providers[0].provider_type = provider_type.into();
                let selected = Arc::new(ImageState::new(&changed));
                let body = Body::from_stream(futures::stream::poll_fn(
                    |_| -> Poll<Option<Result<Bytes, std::io::Error>>> {
                        panic!("unselected body polled")
                    },
                ));
                assert_eq!(
                    generate(Extension(selected), request(body)).await.status(),
                    StatusCode::FORBIDDEN
                );
            }
            let _noauth = InstalledHandoffGuard::set(None);
            assert_eq!(
                app.clone()
                    .call(json_request(&minimal()))
                    .await
                    .unwrap()
                    .status(),
                StatusCode::SERVICE_UNAVAILABLE
            );
            assert_eq!(
                app.clone()
                    .call(edit_request(vec![
                        json!({"image_url":"https://never-fetched.invalid/a.png"})
                    ]))
                    .await
                    .unwrap()
                    .status(),
                StatusCode::SERVICE_UNAVAILABLE
            );
            assert_eq!(upstream.count(), 0);
        });
    }

    #[test]
    fn managed_images_request_validation_and_limits() {
        isolated("managed_images_request_validation_and_limits", async {
            let _auth = held_auth();
            let upstream = Upstream::new().await;
            let app = managed(state(&upstream));
            for (key, val) in [
                ("model", json!("actual-inference-slug")),
                ("model", json!("member-alias")),
                ("model", json!("gpt-image-1")),
                ("background", json!("transparent")),
                ("quality", json!("high")),
                ("size", json!("1024x1024")),
                ("n", json!(1)),
            ] {
                let mut wire = minimal();
                wire[key] = val;
                assert_eq!(
                    app.clone()
                        .call(json_request(&wire))
                        .await
                        .unwrap()
                        .status(),
                    StatusCode::FORBIDDEN
                );
            }
            for key in ["background", "quality", "size", "n", "model", "prompt"] {
                let mut wire = minimal();
                wire[key] = Value::Null;
                assert_eq!(
                    app.clone()
                        .call(json_request(&wire))
                        .await
                        .unwrap()
                        .status(),
                    StatusCode::BAD_REQUEST
                );
            }
            for raw in [b"{".to_vec(), b"null".to_vec(), vec![0xff], br#"{"prompt":"sentinel","model":"gpt-image-2","base_url":"https://sentinel.invalid"}"#.to_vec(),br#"{"prompt":"a","prompt":"b","model":"gpt-image-2"}"#.to_vec()] {
                let response=app.clone().call(request(Body::from(raw))).await.unwrap();
                assert_eq!(response.status(),StatusCode::BAD_REQUEST);
                assert!(!response_json(response).await.to_string().contains("sentinel"));
            }
            let mut wire = minimal();
            wire["prompt"] = json!("x".repeat(crate::models::images::PROMPT_LIMIT + 1));
            assert_eq!(
                app.clone()
                    .call(json_request(&wire))
                    .await
                    .unwrap()
                    .status(),
                StatusCode::PAYLOAD_TOO_LARGE
            );
            for refs in [
                vec![],
                vec![json!({"image_url":"https://example.com/i"}); 6],
                vec![json!({"file_id":"file"})],
                vec![json!({"image_url":null})],
                vec![json!({"image_url":"file:///tmp/sentinel"})],
                vec![json!({"image_url":"/tmp/sentinel"})],
                vec![json!({"image_url":"https://user:pass@example.com/i"})],
                vec![json!({"image_url":"data:image/gif;base64,R0lG"})],
                vec![json!({"image_url":"data:image/png;base64,!!!!"})],
                vec![json!({"image_url":"data:image/png;base64,AA=A"})],
                vec![inline("image/png", b"\x89PNG\r\n\x1a\n")],
                vec![inline(
                    "image/png",
                    include_bytes!("../../tests/fixtures/codex_images/red.jpg"),
                )],
                vec![inline(
                    "image/jpeg",
                    &include_bytes!("../../tests/fixtures/codex_images/red.jpg")[..120],
                )],
                vec![inline(
                    "image/webp",
                    &include_bytes!("../../tests/fixtures/codex_images/red.webp")[..20],
                )],
            ] {
                let response = app.clone().call(edit_request(refs)).await.unwrap();
                assert_eq!(response.status(), StatusCode::BAD_REQUEST);
            }
            for mime in ["image/png", "image/jpeg", "image/webp"] {
                let bytes = match mime {
                    "image/png" => {
                        include_bytes!("../../tests/fixtures/codex_images/red.png").as_slice()
                    }
                    "image/jpeg" => {
                        include_bytes!("../../tests/fixtures/codex_images/red.jpg").as_slice()
                    }
                    _ => include_bytes!("../../tests/fixtures/codex_images/red.webp").as_slice(),
                };
                let url = format!("data:{mime};base64,{}", STANDARD.encode(bytes));
                upstream.push(reply(&output()));
                let r = app
                    .clone()
                    .call(edit_request(vec![json!({"image_url":url.clone()})]))
                    .await
                    .unwrap();
                assert_eq!(r.status(), StatusCode::OK);
                drop(r);
                assert_eq!(
                    upstream.seen.lock().unwrap().last().unwrap().1["images"][0]["image_url"],
                    url
                );
            }
            let before = upstream.count();
            let mut req = json_request(&minimal());
            req.headers_mut().insert(
                header::CONTENT_LENGTH,
                (REQUEST_LIMIT + 1).to_string().parse().unwrap(),
            );
            assert_eq!(
                app.clone().call(req).await.unwrap().status(),
                StatusCode::PAYLOAD_TOO_LARGE
            );
            let req = request(Body::from(vec![b' '; REQUEST_LIMIT + 1]));
            assert_eq!(
                app.clone().call(req).await.unwrap().status(),
                StatusCode::PAYLOAD_TOO_LARGE
            );
            let deep = format!(
                r#"{{"prompt":"p","model":"gpt-image-2","unknown":{}0{}}}"#,
                "[".repeat(33),
                "]".repeat(33)
            );
            assert_eq!(
                app.clone()
                    .call(request(Body::from(deep)))
                    .await
                    .unwrap()
                    .status(),
                StatusCode::BAD_REQUEST
            );
            let dense = format!(
                r#"{{"prompt":"p","model":"gpt-image-2","unknown":[{}0]}}"#,
                "0,".repeat(900000)
            );
            assert_eq!(
                app.clone()
                    .call(request(Body::from(dense)))
                    .await
                    .unwrap()
                    .status(),
                StatusCode::PAYLOAD_TOO_LARGE
            );
            let r=app.clone().call(edit_request(vec![json!({"image_url":format!("https://example.com/{}","x".repeat(crate::models::images::URL_LIMIT))})])).await.unwrap();
            assert_eq!(r.status(), StatusCode::PAYLOAD_TOO_LARGE);
            assert_eq!(upstream.count(), before);
        });
    }

    #[test]
    fn managed_images_response_failures_no_retry_redirect_or_raw_leak() {
        isolated(
            "managed_images_response_failures_no_retry_redirect_or_raw_leak",
            async {
                let _auth = held_auth();
                let upstream = Upstream::new().await;
                let app = managed(state(&upstream));
                for (status, expected) in [
                    (401, StatusCode::FORBIDDEN),
                    (403, StatusCode::FORBIDDEN),
                    (429, StatusCode::TOO_MANY_REQUESTS),
                    (500, StatusCode::BAD_GATEWAY),
                    (503, StatusCode::BAD_GATEWAY),
                ] {
                    upstream.push(Plan::Reply {
                        status,
                        content_type: "application/json",
                        chunks: vec![b"raw-error-sentinel".to_vec()],
                        declared: None,
                        location: None,
                    });
                    let r = app.clone().call(json_request(&minimal())).await.unwrap();
                    assert_eq!(r.status(), expected);
                    assert!(!response_json(r).await.to_string().contains("sentinel"));
                }
                for wire in [
                    json!({"created":0,"data":[]}),
                    json!({"created":null,"data":[{"b64_json":PNG}]}),
                    json!({"created":0,"data":[{"url":"https://sentinel.invalid/i"}]}),
                    json!({"created":0,"data":[{"b64_json":"!!!!"}]}),
                    json!({"created":0,"data":[{"b64_json":STANDARD.encode(include_bytes!("../../tests/fixtures/codex_images/red.jpg"))}]}),
                    json!({"created":0,"data":[{"b64_json":STANDARD.encode(&include_bytes!("../../tests/fixtures/codex_images/red.png")[..45])}]}),
                    json!({"created":0,"data":[{"b64_json":PNG},{"b64_json":"invalid"}]}),
                    json!({"created":0,"data":[{"b64_json":PNG}],"quality":"future"}),
                    json!({"created":0,"data":[{"b64_json":PNG,"generation_id":1}]}),
                ] {
                    upstream.push(reply(&wire));
                    let r = app.clone().call(json_request(&minimal())).await.unwrap();
                    assert_eq!(r.status(), StatusCode::BAD_GATEWAY);
                    assert!(!response_json(r).await.to_string().contains("sentinel"));
                }
                for (ct, raw, declared, expected) in [
                    (
                        "text/plain",
                        b"sentinel".to_vec(),
                        None,
                        StatusCode::BAD_GATEWAY,
                    ),
                    (
                        "application/json",
                        vec![0xff],
                        None,
                        StatusCode::BAD_GATEWAY,
                    ),
                    (
                        "application/json",
                        b"{".to_vec(),
                        Some(100),
                        StatusCode::BAD_GATEWAY,
                    ),
                    (
                        "application/json",
                        Vec::new(),
                        Some(RESPONSE_LIMIT + 1),
                        StatusCode::PAYLOAD_TOO_LARGE,
                    ),
                ] {
                    upstream.push(Plan::Reply {
                        status: 200,
                        content_type: ct,
                        chunks: vec![raw],
                        declared,
                        location: None,
                    });
                    assert_eq!(
                        app.clone()
                            .call(json_request(&minimal()))
                            .await
                            .unwrap()
                            .status(),
                        expected
                    );
                }
                upstream.push(Plan::Chunked(vec![vec![b' '; RESPONSE_LIMIT], vec![b' ']]));
                assert_eq!(
                    app.clone()
                        .call(json_request(&minimal()))
                        .await
                        .unwrap()
                        .status(),
                    StatusCode::PAYLOAD_TOO_LARGE
                );
                let raw = format!(
                    r#"{{"created":0,"data":[{{"b64_json":"{PNG}"}}],"unknown":[{}0]}}"#,
                    "0,".repeat(450000)
                );
                upstream.push(Plan::Reply {
                    status: 200,
                    content_type: "application/json",
                    chunks: vec![raw.into_bytes()],
                    declared: None,
                    location: None,
                });
                assert_eq!(
                    app.clone()
                        .call(json_request(&minimal()))
                        .await
                        .unwrap()
                        .status(),
                    StatusCode::PAYLOAD_TOO_LARGE
                );
                let destination = TcpListener::bind("127.0.0.1:0").await.unwrap();
                upstream.push(Plan::Reply {
                    status: 302,
                    content_type: "application/json",
                    chunks: vec![],
                    declared: None,
                    location: Some(format!(
                        "http://{}/steal",
                        destination.local_addr().unwrap()
                    )),
                });
                let before = upstream.count();
                assert_eq!(
                    app.clone()
                        .call(json_request(&minimal()))
                        .await
                        .unwrap()
                        .status(),
                    StatusCode::BAD_GATEWAY
                );
                assert_eq!(upstream.count(), before + 1);
                assert!(
                    tokio::time::timeout(Duration::from_millis(40), destination.accept())
                        .await
                        .is_err()
                );
                for header in [
                    format!(
                        "X-Codex-Imagegen-Request-Id: {}\r\n",
                        "x".repeat(LABEL_LIMIT + 1)
                    ),
                    "X-Codex-Imagegen-Request-Id: one\r\nX-Codex-Imagegen-Request-Id: two\r\n"
                        .into(),
                ] {
                    let raw = serde_json::to_string(&output()).unwrap();
                    upstream.push(Plan::Raw(format!("HTTP/1.1 200 Fixture\r\nContent-Type: application/json\r\nContent-Length: {}\r\n{header}Connection: close\r\n\r\n{raw}",raw.len()).into_bytes()));
                    let r = app.clone().call(json_request(&minimal())).await.unwrap();
                    assert!(matches!(
                        r.status(),
                        StatusCode::BAD_GATEWAY | StatusCode::PAYLOAD_TOO_LARGE
                    ));
                }
            },
        );
    }

    #[test]
    fn managed_images_shared_body_permit_deadline_and_socket_disconnect() {
        isolated(
            "managed_images_shared_body_permit_deadline_and_socket_disconnect",
            async {
                let _auth = held_auth();
                let upstream = Upstream::new().await;
                let app_state = state(&upstream);
                let selected = Arc::new(ImageState::new(&app_state.snapshot()));
                upstream.push(reply(&output()));
                let body = generate(Extension(selected.clone()), json_request(&minimal())).await;
                assert_eq!(body.status(), StatusCode::OK);
                assert_eq!(selected.permits.available_permits(), 0);
                assert_eq!(
                    edit(
                        Extension(selected.clone()),
                        edit_request(vec![json!({"image_url":"https://never-fetched.invalid/i"})])
                    )
                    .await
                    .status(),
                    StatusCode::TOO_MANY_REQUESTS
                );
                assert_eq!(upstream.count(), 1);
                let mut stream = body.into_body().into_data_stream();
                assert!(stream.next().await.unwrap().is_ok());
                assert_eq!(selected.permits.available_permits(), 0);
                drop(stream);
                assert_eq!(selected.permits.available_permits(), 1);
                upstream.push(reply(&output()));
                let response = edit(
                    Extension(selected.clone()),
                    edit_request(vec![json!({"image_url":"https://never-fetched.invalid/i"})]),
                )
                .await;
                assert_eq!(response.status(), StatusCode::OK);
                response_json(response).await;
                assert_eq!(selected.permits.available_permits(), 1);
                let mut timed = ImageState::new(&app_state.snapshot());
                timed.deadline = Duration::from_millis(40);
                let timed = Arc::new(timed);
                upstream.push(Plan::Hold);
                let response = generate(Extension(timed.clone()), json_request(&minimal())).await;
                assert_eq!(response.status(), StatusCode::GATEWAY_TIMEOUT);
                until(|| upstream.closed.load(Ordering::SeqCst) == 1).await;
                assert_eq!(timed.permits.available_permits(), 1);
                assert!(response_json(response).await["error"]["message"]
                    .as_str()
                    .unwrap()
                    .contains("remote outcome is unknown"));
                let gateway_listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
                let address = gateway_listener.local_addr().unwrap();
                let test_app = axum::Router::new()
                    .route("/v1/images/generations", axum::routing::post(generate))
                    .route("/v1/images/edits", axum::routing::post(edit))
                    .layer(Extension(selected.clone()))
                    .route_layer(axum::middleware::from_fn_with_state(
                        input(),
                        super::super::e3_validate_member_identity,
                    ));
                let serving = tokio::spawn(async move {
                    axum::serve(gateway_listener, test_app).await.unwrap();
                });
                for (i, path) in ["generations", "edits"].into_iter().enumerate() {
                    upstream.push(Plan::Hold);
                    let mut socket = tokio::net::TcpStream::connect(address).await.unwrap();
                    let body = if path == "edits" {
                        r#"{"model":"gpt-image-2","prompt":"edit","images":[{"image_url":"https://never-fetched.invalid/i"}]}"#.to_owned()
                    } else {
                        serde_json::to_string(&minimal()).unwrap()
                    };
                    let wire=format!("POST /v1/images/{path} HTTP/1.1\r\nHost: {address}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nX-Substrate-Orchestration-Session: e3-test-session\r\nX-Substrate-Participant: e3-test-member\r\nX-Substrate-Projection: {}\r\n\r\n{body}",body.len(),"44".repeat(32));
                    socket.write_all(wire.as_bytes()).await.unwrap();
                    until(|| upstream.count() == 4 + i).await;
                    assert_eq!(selected.permits.available_permits(), 0);
                    drop(socket);
                    until(|| {
                        upstream.closed.load(Ordering::SeqCst) == 2 + i
                            && selected.permits.available_permits() == 1
                    })
                    .await;
                }
                serving.abort();
                assert!(serving.await.unwrap_err().is_cancelled());
                let body =
                    Body::from_stream(futures::stream::pending::<Result<Bytes, std::io::Error>>());
                let task = tokio::spawn(generate(Extension(selected.clone()), request(body)));
                until(|| selected.permits.available_permits() == 0).await;
                task.abort();
                assert!(task.await.unwrap_err().is_cancelled());
                assert_eq!(selected.permits.available_permits(), 1);
                upstream.push(reply(&output()));
                let response =
                    generate(Extension(selected.clone()), json_request(&minimal())).await;
                assert_eq!(response.status(), StatusCode::OK);
                drop(response);
                assert_eq!(selected.permits.available_permits(), 1);
                assert_eq!(upstream.count(), 6);
            },
        );
    }
    #[test]
    fn managed_images_exact_wire_file_and_aggregate_boundaries() {
        isolated(
            "managed_images_exact_wire_file_and_aggregate_boundaries",
            async {
                let _auth = held_auth();
                let upstream = Upstream::new().await;
                let app_state = state(&upstream);
                let app = managed(app_state.clone());
                // Valid original PNG with inert trailing file bytes, no transcoding.
                let mut file = include_bytes!("../../tests/fixtures/codex_images/red.png").to_vec();
                file.resize(FILE_LIMIT, 0);
                let encoded = STANDARD.encode(&file);
                drop(file);
                let reference = json!({"image_url":format!("data:image/png;base64,{encoded}")});
                let refs = vec![
                    reference.clone(),
                    reference.clone(),
                    inline(
                        "image/png",
                        include_bytes!("../../tests/fixtures/codex_images/red.png"),
                    ),
                ];
                assert_eq!(
                    app.clone().call(edit_request(refs)).await.unwrap().status(),
                    StatusCode::PAYLOAD_TOO_LARGE
                );
                assert_eq!(upstream.count(), 0);
                assert_eq!(2 * FILE_LIMIT, INPUT_LIMIT);
                upstream.push(reply(&output()));
                let response = app
                    .clone()
                    .call(edit_request(vec![reference.clone(), reference]))
                    .await
                    .unwrap();
                assert_eq!(response.status(), StatusCode::OK);
                drop(response);
                upstream.seen.lock().unwrap().clear();
                // Exact compressed output limit preserves every original base64 byte.
                upstream.push(reply(&json!({"created":0,"data":[{"b64_json":encoded}]})));
                let response = app.clone().call(json_request(&minimal())).await.unwrap();
                assert_eq!(response.status(), StatusCode::OK);
                let value = response_json(response).await;
                assert_eq!(value["data"][0]["b64_json"], encoded);
                drop(value);
                drop(encoded);
                // Two individually admissible PNG files exceed aggregate output by
                // one real small result; validate all entries, never return partial.
                let mut file = include_bytes!("../../tests/fixtures/codex_images/red.png").to_vec();
                file.resize(OUTPUT_LIMIT / 2, 0);
                let encoded = STANDARD.encode(file);
                upstream.push(reply(&json!({"created":0,"data":[{"b64_json":encoded},{"b64_json":encoded},{"b64_json":PNG}]})));
                assert_eq!(
                    app.clone()
                        .call(json_request(&minimal()))
                        .await
                        .unwrap()
                        .status(),
                    StatusCode::PAYLOAD_TOO_LARGE
                );
                drop(encoded);
                upstream.push(reply(&output()));
                let mut raw = serde_json::to_vec(&minimal()).unwrap();
                raw.resize(REQUEST_LIMIT, b' ');
                let response = app.clone().call(request(Body::from(raw))).await.unwrap();
                assert_eq!(response.status(), StatusCode::OK);
                drop(response);
                let mut raw = serde_json::to_vec(&output()).unwrap();
                raw.resize(RESPONSE_LIMIT, b' ');
                upstream.push(Plan::Reply {
                    status: 200,
                    content_type: "application/json",
                    chunks: vec![raw],
                    declared: None,
                    location: None,
                });
                let response = app.clone().call(json_request(&minimal())).await.unwrap();
                assert_eq!(response.status(), StatusCode::OK);
                assert_eq!(response_json(response).await, output());
                // Native source filters an empty optional request ID; neither it nor
                // absent IDs create a new identifier/header.
                for id in ["X-Codex-Imagegen-Request-Id: \r\n", ""] {
                    let raw = serde_json::to_string(&output()).unwrap();
                    upstream.push(Plan::Raw(format!("HTTP/1.1 200 Fixture\r\nContent-Type: application/json\r\nContent-Length: {}\r\n{id}Connection: close\r\n\r\n{raw}",raw.len()).into_bytes()));
                    let response = app.clone().call(json_request(&minimal())).await.unwrap();
                    assert_eq!(response.status(), StatusCode::OK);
                    assert!(!response.headers().contains_key(REQUEST_ID_HEADER));
                    drop(response);
                }
            },
        );
    }
    #[test]
    fn managed_images_decode_deadline_releases_capacity_without_late_success() {
        isolated(
            "managed_images_decode_deadline_releases_capacity_without_late_success",
            async {
                let _auth = held_auth();
                let upstream = Upstream::new().await;
                let app_state = state(&upstream);
                let mut selected = ImageState::new(&app_state.snapshot());
                selected.deadline = Duration::from_millis(20);
                let selected = Arc::new(selected);
                crate::models::images::set_decode_observer(Some(|| {
                    std::thread::sleep(Duration::from_millis(35))
                }));
                let response = edit(
                    Extension(selected.clone()),
                    edit_request(vec![inline(
                        "image/png",
                        include_bytes!("../../tests/fixtures/codex_images/red.png"),
                    )]),
                )
                .await;
                crate::models::images::set_decode_observer(None);
                assert_eq!(response.status(), StatusCode::GATEWAY_TIMEOUT);
                assert_eq!(upstream.count(), 0);
                assert_eq!(selected.permits.available_permits(), 1);
                upstream.push(reply(&output()));
                crate::models::images::set_decode_observer(Some(|| {
                    std::thread::sleep(Duration::from_millis(35))
                }));
                let response =
                    generate(Extension(selected.clone()), json_request(&minimal())).await;
                crate::models::images::set_decode_observer(None);
                assert_eq!(response.status(), StatusCode::GATEWAY_TIMEOUT);
                assert_eq!(upstream.count(), 1);
                assert_eq!(selected.permits.available_permits(), 1);
                // The consumed socket is not retried and capacity really is reusable.
                upstream.push(reply(&output()));
                let response =
                    generate(Extension(selected.clone()), json_request(&minimal())).await;
                assert_eq!(response.status(), StatusCode::OK);
                drop(response);
                assert_eq!(upstream.count(), 2);
                assert_eq!(selected.permits.available_permits(), 1);
            },
        );
    }
    #[test]
    fn img_r1_protected_edit_rejects_before_decoder_and_upstream_io() {
        isolated(
            "img_r1_protected_edit_rejects_before_decoder_and_upstream_io",
            async {
                let _auth = held_auth();
                let upstream = Upstream::new().await;
                let app = managed(state(&upstream));
                for bytes in crate::models::images::webp_guard_test_data::invalid_cases() {
                    crate::models::images::set_decoder_entry_observer(Some(|| {
                        panic!("IMG-R1 protected route reached allocating decoder")
                    }));
                    let response = app
                        .clone()
                        .call(edit_request(vec![inline("image/webp", &bytes)]))
                        .await
                        .unwrap();
                    crate::models::images::set_decoder_entry_observer(None);
                    assert!(matches!(
                        response.status(),
                        StatusCode::BAD_REQUEST | StatusCode::PAYLOAD_TOO_LARGE
                    ));
                    let error = response_json(response).await.to_string();
                    assert!(!error.contains("RIFF"));
                    assert!(!error.contains("VP8"));
                    assert_eq!(upstream.count(), 0);
                }
                for bytes in crate::models::images::webp_guard_test_data::valid_cases() {
                    upstream.push(reply(&output()));
                    let response = app
                        .clone()
                        .call(edit_request(vec![inline("image/webp", bytes)]))
                        .await
                        .unwrap();
                    assert_eq!(response.status(), StatusCode::OK);
                    drop(response);
                    assert_eq!(
                        upstream.seen.lock().unwrap().last().unwrap().1["images"][0],
                        inline("image/webp", bytes)
                    );
                }
                assert_eq!(upstream.count(), 6);
            },
        );
    }
}
