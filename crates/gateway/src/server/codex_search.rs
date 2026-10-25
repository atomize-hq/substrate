//! Managed-only standalone search. The E3 router owns admission before extraction.
use crate::models::search::{
    bounded_json, preflight_json, SearchError, SearchRequest, REQUEST_LIMIT, RESPONSE_LIMIT,
};
use crate::providers::GatewayProvider;
use axum::{
    body::Body,
    extract::{Extension, Request},
    http::{header, StatusCode},
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

pub(super) struct SearchState {
    selected: Option<(Arc<Box<dyn GatewayProvider>>, String)>,
    permits: Arc<Semaphore>,
    deadline: Duration,
}
impl SearchState {
    pub(super) fn new(snapshot: &super::ReloadableState) -> Self {
        // Select the configured default's first mapping once, never route by
        // prompt, caller headers, provider fallback or request model substitution.
        let selected = snapshot
            .config
            .models
            .iter()
            .find(|model| model.name == snapshot.config.router.default)
            .and_then(|model| model.mappings.first())
            .and_then(|mapping| {
                let config = snapshot
                    .config
                    .providers
                    .iter()
                    .find(|p| p.name == mapping.provider)?;
                if config.provider_type != "openai"
                    || config.auth_type != crate::providers::AuthType::OAuth
                    || !config.is_enabled()
                {
                    return None;
                }
                snapshot
                    .provider_registry
                    .get_provider(&mapping.provider)
                    .map(|provider| (provider, mapping.actual_model.clone()))
            });
        Self {
            selected,
            permits: Arc::new(Semaphore::new(4)),
            deadline: Duration::from_secs(60),
        }
    }
}

pub(super) async fn handle_codex_search(
    Extension(state): Extension<Arc<SearchState>>,
    request: Request,
) -> Response {
    // The route's member middleware has already run, including the kernel
    // readback. This route has no standalone registration or body extractor.
    let permit = match Arc::clone(&state.permits).try_acquire_owned() {
        Ok(permit) => permit,
        Err(_) => return SearchError::Busy.into_response(),
    };
    let deadline = tokio::time::Instant::now() + state.deadline;
    let operation = async {
        let (provider, model) = state.selected.as_ref().ok_or(SearchError::Unsupported)?;
        // Incoming routing/auth/destination overrides are never authority here.
        for name in [
            "origin",
            "x-provider",
            "x-base-url",
            "x-account-id",
            "authorization",
            "api-key",
            "chatgpt-account-id",
            "x-openai-actor-authorization",
        ] {
            if request.headers().contains_key(name) {
                return Err(SearchError::Unsupported);
            }
        }
        let content_type = request
            .headers()
            .get(header::CONTENT_TYPE)
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.split(';').next());
        if content_type.map(str::trim) != Some("application/json") {
            return Err(SearchError::InvalidRequest);
        }
        if request
            .headers()
            .get(header::CONTENT_LENGTH)
            .is_some_and(|v| {
                v.to_str()
                    .ok()
                    .and_then(|v| v.parse::<usize>().ok())
                    .is_none_or(|n| n > REQUEST_LIMIT)
            })
        {
            return Err(SearchError::Limit);
        }
        let mut body = request.into_body().into_data_stream();
        let mut bytes = Vec::new();
        while let Some(chunk) = body.next().await {
            let chunk = chunk.map_err(|_| SearchError::InvalidRequest)?;
            if bytes
                .len()
                .checked_add(chunk.len())
                .is_none_or(|n| n > REQUEST_LIMIT)
            {
                return Err(SearchError::Limit);
            }
            bytes
                .try_reserve_exact(chunk.len())
                .map_err(|_| SearchError::Limit)?;
            bytes.extend_from_slice(&chunk);
        }
        preflight_json(&bytes, REQUEST_LIMIT)?;
        let parsed: SearchRequest =
            serde_json::from_slice(&bytes).map_err(|_| SearchError::InvalidRequest)?;
        drop(bytes);
        if parsed.model != *model {
            return Err(SearchError::Unsupported);
        }
        let response = provider.send_search(parsed).await?;
        let bytes = bounded_json(&response, RESPONSE_LIMIT)?;
        drop(response);
        Ok::<_, SearchError>(Bytes::from(bytes))
    };
    match tokio::time::timeout_at(deadline, operation).await {
        // Bounded serde work is synchronous. A poll that crosses the deadline
        // must not turn an overdue operation into a public success.
        Ok(Ok(_)) if tokio::time::Instant::now() >= deadline => {
            SearchError::Deadline.into_response()
        }
        Ok(Ok(bytes)) => (
            [(header::CONTENT_TYPE, "application/json")],
            Body::from_stream(PermittedBody {
                bytes: Some(bytes),
                _permit: permit,
            }),
        )
            .into_response(),
        Ok(Err(error)) => error.into_response(),
        Err(_) => SearchError::Deadline.into_response(),
    }
}

// The permit remains owned after yielding the single data chunk, until the
// response body itself is released. Cancellation before this point drops the
// operation future and its request/response/network buffers without a drainer.
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

impl IntoResponse for SearchError {
    fn into_response(self) -> Response {
        let (status, code) = match self {
            Self::Unsupported => (StatusCode::FORBIDDEN, "search_not_admitted"),
            Self::Authentication => (StatusCode::SERVICE_UNAVAILABLE, "search_auth_unavailable"),
            Self::InvalidRequest => (StatusCode::BAD_REQUEST, "invalid_search_request"),
            Self::Limit => (StatusCode::PAYLOAD_TOO_LARGE, "search_limit_exceeded"),
            Self::Busy => (StatusCode::TOO_MANY_REQUESTS, "search_busy"),
            Self::Deadline => (StatusCode::GATEWAY_TIMEOUT, "search_deadline_exceeded"),
            Self::Transport => (StatusCode::BAD_GATEWAY, "search_outcome_unknown"),
            Self::UpstreamDenied => (StatusCode::FORBIDDEN, "search_upstream_denied"),
            Self::RateLimited => (
                StatusCode::TOO_MANY_REQUESTS,
                "search_upstream_rate_limited",
            ),
            Self::Upstream => (StatusCode::BAD_GATEWAY, "search_upstream_failed"),
            Self::InvalidResponse => (StatusCode::BAD_GATEWAY, "invalid_search_response"),
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
    use crate::providers::{OpenAIProvider, OpenAIProviderConfig, ProviderRegistry};
    use axum::{body::to_bytes, http::Request as HttpRequest};
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
        if std::env::var_os("CODEX_SEARCH_TEST_CHILD").is_none() {
            let output = std::process::Command::new(std::env::current_exe().unwrap())
                .args(["--exact", &format!("server::codex_search::tests::{name}")])
                .env("CODEX_SEARCH_TEST_CHILD", "1")
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
            let url = format!(
                "http://{}/codex/alpha/search",
                listener.local_addr().unwrap()
            )
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
        provider.set_search_loopback_for_test(upstream.url.clone());
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
            .uri("/v1/alpha/search")
            .header("Content-Type", "application/json")
            .header("X-Substrate-Orchestration-Session", "e3-test-session")
            .header("X-Substrate-Participant", "e3-test-member")
            .header("X-Substrate-Projection", "44".repeat(32))
            .body(body)
            .unwrap()
    }
    fn minimal() -> Value {
        json!({"id":"native-id","model":"actual-inference-slug","commands":{}})
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

    #[test]
    fn managed_search_literal_protocol_and_admission() {
        isolated("managed_search_literal_protocol_and_admission", async {
            let _auth = held_auth();
            let upstream = Upstream::new().await;
            let app_state = state(&upstream);
            let app = managed(app_state.clone());
            // Independent native request literal, not a DTO serialization oracle.
            let mut full = json!({"id":"native-session-body-data","model":"actual-inference-slug","input":[{"type":"message","role":"user","content":[{"type":"input_text","text":"previous user α"}]},{"type":"message","role":"assistant","phase":"commentary","content":[{"type":"output_text","text":"previous answer"}]},{"type":"message","role":"user","content":[{"type":"input_text","text":"current user"}]}],"commands":{"search_query":[{"q":"first","recency":1,"domains":["b.test","a.test"]}],"image_query":[{"q":"second"}],"open":[{"ref_id":"https://example.test/page","lineno":0}],"click":[{"ref_id":"prior","id":1}],"find":[{"ref_id":"prior","pattern":"word"}],"screenshot":[{"ref_id":"prior","pageno":0}],"finance":[{"ticker":"X","type":"fund","market":"USA"}],"weather":[{"location":"City","start":"2026-10-24","duration":2}],"sports":[{"tool":"sports","fn":"standings","league":"nfl","team":"X","opponent":"Y","date_from":"2026-10-24","date_to":"2026-10-25","num_games":1,"locale":"en"}],"time":[{"utc_offset":"+05:30"}],"response_length":"long"},"settings":{"user_location":{"type":"approximate","country":"US","region":"IN","city":"Indianapolis","timezone":"America/Indiana/Indianapolis"},"search_context_size":"high","filters":{"allowed_domains":["b.test","a.test"],"blocked_domains":["blocked.test"]},"image_settings":{"max_results":4,"caption":true},"allowed_callers":["direct","shell","code_interpreter"],"external_web_access":"indexed"},"max_output_tokens":1234});
            full["input"][1]["internal_chat_message_metadata_passthrough"] = json!({
                "turn_id":"not-member-authority", "create_time":1772222222.125,
                "content_item_kinds":["text","future-kind"], "cell_id":"not-native-session-authority",
                "executed_tool_calls":[
                    {"name":"not-executed", "arguments":{"authorization":"credential-looking-sentinel","x-provider":"unselected","command":"do-not-execute","url":"file:///must-not-read"}, "tool_result_sources":[{"type":"parse_failed","id":""}], "tool_result_metadata":{"header":"inert-sentinel","opaque":[1,false,null]}},
                    {"name":"native-truncated", "arguments":{"_codex_executed_tool_call_truncated":{"original_bytes":10000,"max_bytes":8192,"omitted_calls":1,"original_name_bytes":8}}, "tool_result_metadata":"omitted_due_to_size_limit"},
                    {"name":"native-null", "arguments":null, "tool_result_metadata":null}
                ], "tool_calls_complete":true
            });
            let result = json!({"output":"fixture 🦀 output","results":[{"type":"future_dto","ref_id":"turn0search0","opaque":{"prompt":"untrusted-sentinel","array":[null,3,true]}}],"encrypted_output":"encrypted-sentinel"});
            let bytes = serde_json::to_vec(&result).unwrap();
            let split = bytes.iter().position(|b| *b == 0xf0).unwrap() + 1;
            upstream.push(Plan::Reply {
                status: 200,
                content_type: "application/json; charset=utf-8",
                chunks: vec![
                    bytes[..split].to_vec(),
                    bytes[split..split + 1].to_vec(),
                    bytes[split + 1..].to_vec(),
                ],
                declared: None,
                location: None,
            });
            let response = app.clone().call(json_request(&full)).await.unwrap();
            assert_eq!(response.status(), StatusCode::OK);
            assert_eq!(response_json(response).await, result);
            let next = json!({"id":"native-session-body-data","model":"actual-inference-slug","commands":{"open":[{"ref_id":"turn0search0"}],"click":[{"ref_id":"turn0search0","id":17}],"find":[{"ref_id":"turn0search0","pattern":"followup"}],"screenshot":[{"ref_id":"turn0search0","pageno":1}]}});
            upstream.push(reply(&json!({"output":"referenced result"})));
            let response = app.clone().call(json_request(&next)).await.unwrap();
            assert_eq!(response.status(), StatusCode::OK);
            assert_eq!(
                response_json(response).await,
                json!({"output":"referenced result"})
            );
            {
                let seen = upstream.seen.lock().unwrap();
                assert_eq!(seen[0].1, full);
                assert_eq!(seen[1].1, next);
                for (headers, _) in seen.iter() {
                    let lower = headers.to_ascii_lowercase();
                    assert!(lower.starts_with("post /codex/alpha/search http/1.1\r\n"));
                    assert!(lower.contains("authorization: bearer synthetic-held-token\r\n"));
                    assert!(lower.contains("chatgpt-account-id: synthetic-account\r\n"));
                    assert!(!lower.contains("must-never-be-used"));
                    assert!(!lower.contains("x-substrate"));
                }
            }
            for header in [
                "origin",
                "x-provider",
                "x-base-url",
                "x-account-id",
                "authorization",
                "api-key",
                "chatgpt-account-id",
                "x-openai-actor-authorization",
            ] {
                let mut request = json_request(&minimal());
                request.headers_mut().insert(
                    axum::http::HeaderName::from_bytes(header.as_bytes()).unwrap(),
                    "untrusted-sentinel".parse().unwrap(),
                );
                assert_eq!(
                    app.clone().call(request).await.unwrap().status(),
                    StatusCode::FORBIDDEN
                );
            }
            for header in [
                "X-Substrate-Orchestration-Session",
                "X-Substrate-Participant",
                "X-Substrate-Projection",
            ] {
                for mode in 0..3 {
                    // A stream that panics if polled proves admission precedes extraction.
                    let body = Body::from_stream(futures::stream::poll_fn(
                        |_| -> Poll<Option<Result<Bytes, std::io::Error>>> {
                            panic!("denied request body was polled")
                        },
                    ));
                    let mut req = request(body);
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
                *req.method_mut() = method.parse().unwrap();
                assert_eq!(
                    app.clone().call(req).await.unwrap().status(),
                    StatusCode::METHOD_NOT_ALLOWED
                );
            }
            for path in [
                "/v1/alpha/search/",
                "/v1/alpha/search/extra",
                "/alpha/search",
                "/v1/messages",
                "/api/oauth/tokens",
            ] {
                let mut req = json_request(&minimal());
                *req.uri_mut() = path.parse().unwrap();
                assert_eq!(
                    app.clone().call(req).await.unwrap().status(),
                    StatusCode::NOT_FOUND
                );
            }
            for model in ["member-alias", "unselected-slug"] {
                let mut wire = minimal();
                wire["model"] = json!(model);
                assert_eq!(
                    app.clone()
                        .call(json_request(&wire))
                        .await
                        .unwrap()
                        .status(),
                    StatusCode::FORBIDDEN
                );
            }
            // Fixed selected-provider admission also precedes body extraction.
            for provider_type in ["anthropic", "gemini", "azure-openai"] {
                let mut snapshot = super::super::ReloadableState {
                    config: app_state.snapshot().config.clone(),
                    router: app_state.snapshot().router.clone(),
                    provider_registry: app_state.snapshot().provider_registry.clone(),
                };
                snapshot.config.providers[0].provider_type = provider_type.into();
                let selected = Arc::new(SearchState::new(&snapshot));
                let body = Body::from_stream(futures::stream::poll_fn(
                    |_| -> Poll<Option<Result<Bytes, std::io::Error>>> {
                        panic!("unadmitted provider polled body")
                    },
                ));
                assert_eq!(
                    handle_codex_search(Extension(selected), request(body))
                        .await
                        .status(),
                    StatusCode::FORBIDDEN
                );
            }
            let _no_auth = InstalledHandoffGuard::set(None);
            assert_eq!(
                app.clone()
                    .call(json_request(&minimal()))
                    .await
                    .unwrap()
                    .status(),
                StatusCode::SERVICE_UNAVAILABLE
            );
            assert_eq!(upstream.count(), 2);
            // The general standalone router remains closed to this operation.
            assert_eq!(
                super::super::build_app(app_state)
                    .call(json_request(&minimal()))
                    .await
                    .unwrap()
                    .status(),
                StatusCode::NOT_FOUND
            );
        });
    }

    #[test]
    fn managed_search_limits_and_safe_failures() {
        isolated("managed_search_limits_and_safe_failures", async {
            let _auth = held_auth();
            let upstream = Upstream::new().await;
            let app_state = state(&upstream);
            let selected = Arc::new(SearchState::new(&app_state.snapshot()));
            let app = managed(app_state);
            for raw in [
                b"{".to_vec(),
                b"null".to_vec(),
                vec![0xff],
                br#"{"id":"id","model":"actual-inference-slug","unknown":"raw-error-sentinel"}"#
                    .to_vec(),
                br#"{"id":"id","model":"actual-inference-slug","commands":null}"#.to_vec(),
                vec![b'x'; REQUEST_LIMIT + 1],
            ] {
                let response = app.clone().call(request(Body::from(raw))).await.unwrap();
                assert!(matches!(
                    response.status(),
                    StatusCode::BAD_REQUEST | StatusCode::PAYLOAD_TOO_LARGE
                ));
                assert!(!response_json(response)
                    .await
                    .to_string()
                    .contains("sentinel"));
            }
            let nested = format!("{}0{}", "[".repeat(200), "]".repeat(200));
            let deep = format!(
                r#"{{"id":"id","model":"actual-inference-slug","input":[{{"type":"message","role":"assistant","content":[],"internal_chat_message_metadata_passthrough":{{"executed_tool_calls":[{{"name":"tool","arguments":{nested}}}]}}}}]}}"#
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
                r#"{{"id":"id","model":"actual-inference-slug","input":[{{"type":"message","role":"assistant","content":[],"internal_chat_message_metadata_passthrough":{{"executed_tool_calls":[{{"name":"tool","arguments":[{}]}}]}}}}]}}"#,
                "null,".repeat(70000) + "null"
            );
            assert_eq!(
                app.clone()
                    .call(request(Body::from(dense)))
                    .await
                    .unwrap()
                    .status(),
                StatusCode::PAYLOAD_TOO_LARGE
            );
            assert_eq!(upstream.count(), 0);
            let prefix = r#"{"id":"native-id","model":"actual-inference-slug","input":""#;
            let raw = format!(
                "{prefix}{}\"}}",
                "x".repeat(REQUEST_LIMIT - prefix.len() - 2)
            );
            assert_eq!(raw.len(), REQUEST_LIMIT);
            upstream.push(reply(&json!({"output":"request boundary"})));
            let response = app.clone().call(request(Body::from(raw))).await.unwrap();
            assert_eq!(response.status(), StatusCode::OK);
            drop(response);
            // Exact response wire boundary succeeds; +1 cumulative byte fails.
            let raw = format!("{{\"output\":\"{}\"}}", "x".repeat(RESPONSE_LIMIT - 13));
            assert_eq!(raw.len(), RESPONSE_LIMIT);
            upstream.push(Plan::Reply {
                status: 200,
                content_type: "application/json",
                chunks: vec![raw.into_bytes()],
                declared: None,
                location: None,
            });
            let response =
                handle_codex_search(Extension(selected.clone()), json_request(&minimal())).await;
            assert_eq!(response.status(), StatusCode::OK);
            assert_eq!(
                to_bytes(response.into_body(), RESPONSE_LIMIT)
                    .await
                    .unwrap()
                    .len(),
                RESPONSE_LIMIT
            );
            upstream.push(Plan::Reply {
                status: 200,
                content_type: "application/json",
                chunks: Vec::new(),
                declared: Some(RESPONSE_LIMIT + 1),
                location: None,
            });
            assert_eq!(
                handle_codex_search(Extension(selected.clone()), json_request(&minimal()))
                    .await
                    .status(),
                StatusCode::PAYLOAD_TOO_LARGE
            );
            // Cumulative chunked bytes are checked independently of Content-Length.
            upstream.push(Plan::Chunked(vec![vec![b' '; RESPONSE_LIMIT], vec![b' ']]));
            assert_eq!(
                handle_codex_search(Extension(selected.clone()), json_request(&minimal()))
                    .await
                    .status(),
                StatusCode::PAYLOAD_TOO_LARGE
            );
            // A dense opaque tree can exhaust the logical budget below the wire ceiling.
            let dense = format!(
                "{{\"output\":\"ok\",\"results\":[{}]}}",
                "0,".repeat(140000) + "0"
            );
            assert!(dense.len() < RESPONSE_LIMIT);
            upstream.push(Plan::Reply {
                status: 200,
                content_type: "application/json",
                chunks: vec![dense.into_bytes()],
                declared: None,
                location: None,
            });
            assert_eq!(
                handle_codex_search(Extension(selected.clone()), json_request(&minimal()))
                    .await
                    .status(),
                StatusCode::PAYLOAD_TOO_LARGE
            );
            for (status, expected) in [
                (401, StatusCode::FORBIDDEN),
                (403, StatusCode::FORBIDDEN),
                (429, StatusCode::TOO_MANY_REQUESTS),
                (500, StatusCode::BAD_GATEWAY),
            ] {
                upstream.push(Plan::Reply {
                    status,
                    content_type: "application/json",
                    chunks: vec![b"raw-upstream-error-sentinel".to_vec()],
                    declared: None,
                    location: None,
                });
                let response =
                    handle_codex_search(Extension(selected.clone()), json_request(&minimal()))
                        .await;
                assert_eq!(response.status(), expected);
                assert!(!response_json(response)
                    .await
                    .to_string()
                    .contains("sentinel"));
                assert_eq!(selected.permits.available_permits(), 4);
            }
            for (content_type, raw, declared) in [
                ("text/plain", b"sentinel".to_vec(), None),
                ("application/json", vec![0xff], None),
                ("application/json", b"{\"output\":null}".to_vec(), None),
                (
                    "application/json",
                    b"{\"output\":\"sentinel\",\"extra\":1}".to_vec(),
                    None,
                ),
                ("application/json", b"{".to_vec(), None),
                ("application/json", b"{".to_vec(), Some(100)),
            ] {
                upstream.push(Plan::Reply {
                    status: 200,
                    content_type,
                    chunks: vec![raw],
                    declared,
                    location: None,
                });
                let response =
                    handle_codex_search(Extension(selected.clone()), json_request(&minimal()))
                        .await;
                assert_eq!(response.status(), StatusCode::BAD_GATEWAY);
                assert!(!response_json(response)
                    .await
                    .to_string()
                    .contains("sentinel"));
                assert_eq!(selected.permits.available_permits(), 4);
            }
            let destination = TcpListener::bind("127.0.0.1:0").await.unwrap();
            upstream.push(Plan::Reply {
                status: 302,
                content_type: "application/json",
                chunks: Vec::new(),
                declared: None,
                location: Some(format!(
                    "http://{}/credential-theft",
                    destination.local_addr().unwrap()
                )),
            });
            assert_eq!(
                handle_codex_search(Extension(selected.clone()), json_request(&minimal()))
                    .await
                    .status(),
                StatusCode::BAD_GATEWAY
            );
            assert!(
                tokio::time::timeout(Duration::from_millis(40), destination.accept())
                    .await
                    .is_err()
            );
            assert_eq!(selected.permits.available_permits(), 4);
        });
    }

    #[test]
    fn managed_search_body_permits_deadline_and_disconnect() {
        isolated(
            "managed_search_body_permits_deadline_and_disconnect",
            async {
                let _auth = held_auth();
                let upstream = Upstream::new().await;
                let app_state = state(&upstream);
                let selected = Arc::new(SearchState::new(&app_state.snapshot()));
                let mut bodies = Vec::new();
                for _ in 0..4 {
                    upstream.push(reply(&json!({"output":"held response"})));
                    let response =
                        handle_codex_search(Extension(selected.clone()), json_request(&minimal()))
                            .await;
                    assert_eq!(response.status(), StatusCode::OK);
                    bodies.push(response);
                }
                assert_eq!(selected.permits.available_permits(), 0);
                assert_eq!(
                    handle_codex_search(Extension(selected.clone()), json_request(&minimal()))
                        .await
                        .status(),
                    StatusCode::TOO_MANY_REQUESTS
                );
                assert_eq!(upstream.count(), 4);
                // Even after yielding data, the body owns its permit until release.
                let mut stream = bodies.pop().unwrap().into_body().into_data_stream();
                assert!(stream.next().await.unwrap().is_ok());
                assert_eq!(selected.permits.available_permits(), 0);
                drop(stream);
                assert_eq!(selected.permits.available_permits(), 1);
                drop(bodies);
                assert_eq!(selected.permits.available_permits(), 4);
                let mut timed = SearchState::new(&app_state.snapshot());
                timed.deadline = Duration::from_millis(40);
                let timed = Arc::new(timed);
                upstream.push(Plan::Hold);
                let response =
                    handle_codex_search(Extension(timed.clone()), json_request(&minimal())).await;
                assert_eq!(response.status(), StatusCode::GATEWAY_TIMEOUT);
                until(|| upstream.closed.load(Ordering::SeqCst) == 1).await;
                assert_eq!(timed.permits.available_permits(), 4);
                assert!(response_json(response).await["error"]["message"]
                    .as_str()
                    .unwrap()
                    .contains("remote outcome is unknown"));
                // Actual gateway TCP disconnect, while provider is awaiting a real
                // upstream socket: prove both EOF and release without aborting handler.
                let gateway_listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
                let address = gateway_listener.local_addr().unwrap();
                let test_app = axum::Router::new()
                    .route("/v1/alpha/search", axum::routing::post(handle_codex_search))
                    .layer(Extension(selected.clone()))
                    .route_layer(axum::middleware::from_fn_with_state(
                        input(),
                        super::super::e3_validate_member_identity,
                    ));
                let serving = tokio::spawn(async move {
                    axum::serve(gateway_listener, test_app).await.unwrap();
                });
                upstream.push(Plan::Hold);
                let mut socket = tokio::net::TcpStream::connect(address).await.unwrap();
                let body = serde_json::to_string(&minimal()).unwrap();
                let headers=format!("POST /v1/alpha/search HTTP/1.1\r\nHost: {address}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nX-Substrate-Orchestration-Session: e3-test-session\r\nX-Substrate-Participant: e3-test-member\r\nX-Substrate-Projection: {}\r\n\r\n{body}",body.len(),"44".repeat(32));
                socket.write_all(headers.as_bytes()).await.unwrap();
                until(|| upstream.count() == 6).await;
                assert_eq!(selected.permits.available_permits(), 3);
                drop(socket);
                until(|| {
                    upstream.closed.load(Ordering::SeqCst) == 2
                        && selected.permits.available_permits() == 4
                })
                .await;
                serving.abort();
                assert!(serving.await.unwrap_err().is_cancelled());
                // Cancellation while receiving the request body also releases capacity.
                let body =
                    Body::from_stream(futures::stream::pending::<Result<Bytes, std::io::Error>>());
                let task = tokio::spawn(handle_codex_search(
                    Extension(selected.clone()),
                    request(body),
                ));
                until(|| selected.permits.available_permits() == 3).await;
                task.abort();
                assert!(task.await.unwrap_err().is_cancelled());
                assert_eq!(selected.permits.available_permits(), 4);
            },
        );
    }
}
