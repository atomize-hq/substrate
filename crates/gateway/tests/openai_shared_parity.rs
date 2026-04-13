use async_trait::async_trait;
use axum::http::{HeaderMap, HeaderValue, StatusCode};
use serde::Deserialize;
use serde_json::{json, Value};
use substrate_gateway::core::{
    GatewayRequest, GatewayResponse, GatewayStreamResponse, GatewayUsage,
};
use substrate_gateway::models::{ContentBlock, MessageContent};
use substrate_gateway::providers::error::ProviderError;
use substrate_gateway::providers::{GatewayProvider, ProviderRegistry};
use substrate_gateway::server::openai_conformance_test_support::{
    read_json_fixture, response_text, ConformanceHarness, FixtureNamespace, StubProvider,
};

type CapturedRequests = std::sync::Arc<std::sync::Mutex<Vec<GatewayRequest>>>;

#[derive(Debug, Deserialize)]
struct ChatSyncFixture {
    request: Value,
    provider_response: GatewayResponse,
}

#[derive(Debug, Deserialize)]
struct ResponsesSyncFixture {
    request_model: String,
    provider_response: GatewayResponse,
}

#[derive(Debug, Deserialize)]
struct ChatStreamFixture {
    request: Value,
    provider_stream_chunks: Vec<String>,
}

fn chat_sync_fixture() -> ChatSyncFixture {
    read_json_fixture(FixtureNamespace::OpenAiChatCompletions, "sync-text.json")
}

fn responses_sync_fixture() -> ResponsesSyncFixture {
    read_json_fixture(FixtureNamespace::OpenAiResponses, "sync-text.json")
}

fn chat_stream_fixture() -> ChatStreamFixture {
    read_json_fixture(FixtureNamespace::OpenAiChatCompletions, "stream-text.json")
}

fn responses_stream_fixture_request() -> Value {
    read_json_fixture::<Value>(FixtureNamespace::OpenAiResponses, "stream-mixed.json")["request"]
        .clone()
}

fn chat_sync_request() -> Value {
    chat_sync_fixture().request
}

fn responses_sync_request() -> Value {
    let fixture = responses_sync_fixture();
    json!({
        "model": fixture.request_model,
        "input": "hello",
        "stream": false
    })
}

fn responses_public_model() -> String {
    responses_sync_fixture().request_model
}

fn routing_header(provider: &str) -> HeaderMap {
    let mut headers = HeaderMap::new();
    headers.insert(
        "x-provider",
        HeaderValue::from_str(provider).expect("valid provider header"),
    );
    headers
}

fn build_single_provider_harness(provider: StubProvider) -> ConformanceHarness {
    ConformanceHarness::single_provider(provider, "primary-actual", false)
}

fn build_multi_provider_harness(
    primary: StubProvider,
    secondary: StubProvider,
) -> (ConformanceHarness, CapturedRequests, CapturedRequests) {
    let primary_requests = primary.captured_requests();
    let secondary_requests = secondary.captured_requests();

    let mut provider_registry = ProviderRegistry::new();
    provider_registry.insert_provider_for_tests("primary", Box::new(primary));
    provider_registry.insert_provider_for_tests("secondary", Box::new(secondary));

    let harness = ConformanceHarness::with_registry(
        "gateway-default",
        provider_registry,
        vec![
            substrate_gateway::cli::ModelMapping {
                priority: 1,
                provider: "primary".to_string(),
                actual_model: "primary-actual".to_string(),
                inject_continuation_prompt: false,
            },
            substrate_gateway::cli::ModelMapping {
                priority: 2,
                provider: "secondary".to_string(),
                actual_model: "secondary-actual".to_string(),
                inject_continuation_prompt: false,
            },
        ],
    );

    (harness, primary_requests, secondary_requests)
}

fn build_direct_multi_provider_harness(
    primary: StubProvider,
    secondary: StubProvider,
) -> (ConformanceHarness, CapturedRequests, CapturedRequests) {
    let primary_requests = primary.captured_requests();
    let secondary_requests = secondary.captured_requests();

    let mut provider_registry = ProviderRegistry::new();
    provider_registry.insert_provider_for_tests("primary", Box::new(primary));
    provider_registry.insert_provider_for_tests("secondary", Box::new(secondary));

    let harness = ConformanceHarness::with_direct_registry("gateway-direct", provider_registry);

    (harness, primary_requests, secondary_requests)
}

fn make_failure_provider() -> FailingProvider {
    FailingProvider {
        api_status: 500,
        message: "backend exploded".to_string(),
    }
}

fn reasoning_sync_response() -> GatewayResponse {
    GatewayResponse {
        id: "resp_reasoning_sync".to_string(),
        r#type: "message".to_string(),
        role: "assistant".to_string(),
        content: vec![
            ContentBlock::thinking(json!({
                "type": "thinking",
                "text": "secret reasoning"
            })),
            ContentBlock::text("Visible answer".to_string(), None),
        ],
        model: "primary-actual".to_string(),
        stop_reason: Some("end_turn".to_string()),
        stop_sequence: None,
        usage: GatewayUsage {
            input_tokens: 4,
            output_tokens: 2,
            cache_creation_input_tokens: None,
            cache_read_input_tokens: None,
        },
    }
}

fn reasoning_stream_chunks() -> Vec<String> {
    vec![
        "event: message_start\ndata: {\"type\":\"message_start\",\"message\":{\"id\":\"msg_reasoning\",\"type\":\"message\",\"role\":\"assistant\",\"content\":[],\"model\":\"primary-actual\",\"stop_reason\":null,\"stop_sequence\":null,\"usage\":{\"input_tokens\":0,\"output_tokens\":0}}}\n\n".to_string(),
        "event: content_block_start\ndata: {\"type\":\"content_block_start\",\"index\":0,\"content_block\":{\"type\":\"thinking\",\"text\":\"secret reasoning\"}}\n\n".to_string(),
        "event: content_block_stop\ndata: {\"type\":\"content_block_stop\",\"index\":0}\n\n".to_string(),
        "event: content_block_delta\ndata: {\"type\":\"content_block_delta\",\"index\":1,\"delta\":{\"type\":\"text_delta\",\"text\":\"Visible stream answer\"}}\n\n".to_string(),
        "event: content_block_stop\ndata: {\"type\":\"content_block_stop\",\"index\":1}\n\n".to_string(),
        "event: message_delta\ndata: {\"type\":\"message_delta\",\"delta\":{\"stop_reason\":\"end_turn\",\"stop_sequence\":null},\"usage\":{\"input_tokens\":4,\"output_tokens\":2}}\n\n".to_string(),
        "event: message_stop\ndata: {\"type\":\"message_stop\"}\n\n".to_string(),
    ]
}

fn assert_error_contract(response: axum::response::Response, expected_status: StatusCode) -> Value {
    assert_eq!(response.status(), expected_status);
    let body = futures::executor::block_on(response_text(response));
    serde_json::from_str(&body).expect("gateway error envelope")
}

fn assert_route_error_shape(json: &Value) {
    assert_eq!(json["error"]["type"], "error");
    assert_eq!(json["error"]["class"], "route");
    assert_eq!(json["error"]["message"], "Route selection failed");
}

fn assert_provider_error_shape(json: &Value, class: &str, message: &str) {
    assert_eq!(json["error"]["type"], "error");
    assert_eq!(json["error"]["class"], class);
    assert_eq!(json["error"]["message"], message);
}

#[tokio::test]
async fn model_echo_is_shared_across_endpoints() {
    let chat_fixture = chat_sync_fixture();
    let chat_provider = StubProvider::new(chat_fixture.provider_response.clone(), vec![]);
    let chat_harness = build_single_provider_harness(chat_provider);

    let chat_response = chat_harness
        .invoke_chat_completions(HeaderMap::new(), chat_fixture.request.clone())
        .await;
    assert_eq!(chat_response.status(), StatusCode::OK);
    let chat_body = response_text(chat_response).await;
    let chat_json: Value = serde_json::from_str(&chat_body).expect("chat completion json");
    assert_eq!(chat_json["model"], chat_fixture.request["model"]);
    assert!(
        chat_body.contains("Boundary text from fixture."),
        "chat response should surface the public answer"
    );
    {
        let chat_requests = chat_harness.captured_requests();
        let chat_requests = chat_requests.lock().unwrap();
        assert_eq!(chat_requests.len(), 1);
        assert_eq!(chat_requests[0].model, "primary-actual");
    }

    let responses_fixture = responses_sync_fixture();
    let responses_provider = StubProvider::new(responses_fixture.provider_response.clone(), vec![]);
    let responses_harness =
        ConformanceHarness::single_provider(responses_provider, "primary-actual", false);

    let responses_response = responses_harness
        .invoke_responses(HeaderMap::new(), responses_sync_request())
        .await;
    assert_eq!(responses_response.status(), StatusCode::OK);
    let responses_body = response_text(responses_response).await;
    let responses_json: Value = serde_json::from_str(&responses_body).expect("responses json");
    assert_eq!(responses_json["model"], responses_public_model());
    assert!(responses_body.contains("Hello from text"));
    {
        let responses_requests = responses_harness.captured_requests();
        let responses_requests = responses_requests.lock().unwrap();
        assert_eq!(responses_requests.len(), 1);
        assert_eq!(responses_requests[0].model, "primary-actual");
    }
}

#[tokio::test]
async fn chat_direct_fallback_streaming_preserves_sse_contract() {
    let fixture = chat_stream_fixture();
    let provider = StubProvider::new(
        substrate_gateway::server::openai_conformance_test_support::response_text_response(
            "unused",
            "gateway-direct",
        ),
        fixture.provider_stream_chunks,
    );
    let captured_requests = provider.captured_requests();
    let harness = ConformanceHarness::with_direct_registry("gateway-direct", {
        let mut registry = ProviderRegistry::new();
        registry.insert_provider_for_tests("direct", Box::new(provider));
        registry
    });

    let mut request = fixture.request;
    request["model"] = json!("gateway-direct");

    let response = harness
        .invoke_chat_completions(HeaderMap::new(), request)
        .await;

    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        response
            .headers()
            .get("content-type")
            .and_then(|value| value.to_str().ok()),
        Some("text/event-stream")
    );

    let body = response_text(response).await;
    assert!(body.contains("\"object\":\"chat.completion.chunk\""));
    assert!(body.contains("data: [DONE]"));

    let requests = captured_requests.lock().unwrap();
    assert_eq!(requests.len(), 1);
    assert_eq!(requests[0].model, "gateway-direct");
    assert_eq!(requests[0].stream, Some(true));
}

#[tokio::test]
async fn chat_direct_fallback_honors_x_provider_and_rejects_missing_provider() {
    let chat_fixture = chat_sync_fixture();
    let (harness, primary_requests, secondary_requests) = build_direct_multi_provider_harness(
        StubProvider::new(chat_fixture.provider_response.clone(), vec![]),
        StubProvider::new(
            GatewayResponse {
                model: "gateway-direct".to_string(),
                ..chat_fixture.provider_response.clone()
            },
            vec![],
        ),
    );

    let mut request = chat_sync_request();
    request["model"] = json!("gateway-direct");

    let forced_response = harness
        .invoke_chat_completions(routing_header("secondary"), request.clone())
        .await;
    assert_eq!(forced_response.status(), StatusCode::OK);
    let forced_body = response_text(forced_response).await;
    let forced_json: Value = serde_json::from_str(&forced_body).expect("chat completion json");
    assert_eq!(forced_json["model"], json!("gateway-direct"));
    assert_eq!(primary_requests.lock().unwrap().len(), 0);
    assert_eq!(secondary_requests.lock().unwrap().len(), 1);
    assert_eq!(
        secondary_requests.lock().unwrap()[0].model,
        "gateway-direct"
    );

    let missing_response = harness
        .invoke_chat_completions(routing_header("missing"), request)
        .await;
    let missing_json = assert_error_contract(missing_response, StatusCode::BAD_REQUEST);
    assert_route_error_shape(&missing_json);
}

#[tokio::test]
async fn responses_direct_fallback_streaming_preserves_sse_contract() {
    let provider = StubProvider::new(
        substrate_gateway::server::openai_conformance_test_support::response_text_response(
            "unused",
            "gateway-direct",
        ),
        read_json_fixture::<Value>(FixtureNamespace::OpenAiResponses, "stream-text.json")
            ["provider_stream_chunks"]
            .as_array()
            .expect("stream fixture chunks")
            .iter()
            .map(|value| value.as_str().expect("stream chunk").to_string())
            .collect(),
    );
    let captured_requests = provider.captured_requests();
    let harness = ConformanceHarness::with_direct_registry("gateway-direct", {
        let mut registry = ProviderRegistry::new();
        registry.insert_provider_for_tests("direct", Box::new(provider));
        registry
    });

    let mut request = json!({
        "model": "gateway-direct",
        "input": "hello",
        "stream": true
    });
    request["model"] = json!("gateway-direct");

    let response = harness.invoke_responses(HeaderMap::new(), request).await;

    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        response
            .headers()
            .get("content-type")
            .and_then(|value| value.to_str().ok()),
        Some("text/event-stream")
    );

    let body = response_text(response).await;
    assert!(body.contains("event: response.created"));
    assert!(body.contains("event: response.completed"));

    let requests = captured_requests.lock().unwrap();
    assert_eq!(requests.len(), 1);
    assert_eq!(requests[0].model, "gateway-direct");
    assert_eq!(requests[0].stream, Some(true));
}

#[tokio::test]
async fn responses_direct_fallback_honors_x_provider_and_rejects_missing_provider() {
    let responses_fixture = responses_sync_fixture();
    let (harness, primary_requests, secondary_requests) = build_direct_multi_provider_harness(
        StubProvider::new(responses_fixture.provider_response.clone(), vec![]),
        StubProvider::new(
            GatewayResponse {
                model: "gateway-direct".to_string(),
                ..responses_fixture.provider_response.clone()
            },
            vec![],
        ),
    );

    let request = json!({
        "model": "gateway-direct",
        "input": "hello",
        "stream": false
    });

    let forced_response = harness
        .invoke_responses(routing_header("secondary"), request.clone())
        .await;
    assert_eq!(forced_response.status(), StatusCode::OK);
    let forced_body = response_text(forced_response).await;
    let forced_json: Value = serde_json::from_str(&forced_body).expect("responses json");
    assert_eq!(forced_json["model"], json!("gateway-direct"));
    assert_eq!(primary_requests.lock().unwrap().len(), 0);
    assert_eq!(secondary_requests.lock().unwrap().len(), 1);
    assert_eq!(
        secondary_requests.lock().unwrap()[0].model,
        "gateway-direct"
    );

    let missing_response = harness
        .invoke_responses(routing_header("missing"), request)
        .await;
    let missing_json = assert_error_contract(missing_response, StatusCode::BAD_REQUEST);
    assert_route_error_shape(&missing_json);
}

#[tokio::test]
async fn ordered_instruction_roles_and_text_parts_normalize_identically_across_endpoints() {
    let provider_response =
        substrate_gateway::server::openai_conformance_test_support::response_text_response(
            "ok",
            "primary-actual",
        );

    let chat_harness =
        build_single_provider_harness(StubProvider::new(provider_response.clone(), vec![]));
    let responses_harness =
        build_single_provider_harness(StubProvider::new(provider_response, vec![]));

    let chat_request = json!({
        "model": "gateway-default",
        "messages": [
            {
                "role": "system",
                "content": [
                    {"type": "text", "text": "sys"},
                    {"type": "text", "text": "sys-2"}
                ]
            },
            {
                "role": "developer",
                "content": [
                    {"type": "text", "text": "dev"},
                    {"type": "text", "text": "dev-2"}
                ]
            },
            {
                "role": "user",
                "content": [
                    {"type": "text", "text": "hello"},
                    {"type": "text", "text": "hello-2"}
                ]
            }
        ],
        "stream": false
    });
    let responses_request = json!({
        "model": "gateway-default",
        "input": [
            {
                "type": "message",
                "role": "system",
                "content": [
                    {"type": "input_text", "text": "sys"},
                    {"type": "input_text", "text": "sys-2"}
                ]
            },
            {
                "type": "message",
                "role": "developer",
                "content": [
                    {"type": "input_text", "text": "dev"},
                    {"type": "input_text", "text": "dev-2"}
                ]
            },
            {
                "type": "message",
                "role": "user",
                "content": [
                    {"type": "input_text", "text": "hello"},
                    {"type": "input_text", "text": "hello-2"}
                ]
            }
        ],
        "stream": false
    });

    let chat_response = chat_harness
        .invoke_chat_completions(HeaderMap::new(), chat_request)
        .await;
    let responses_response = responses_harness
        .invoke_responses(HeaderMap::new(), responses_request)
        .await;

    assert_eq!(chat_response.status(), StatusCode::OK);
    assert_eq!(responses_response.status(), StatusCode::OK);

    let chat_requests = chat_harness.captured_requests();
    let responses_requests = responses_harness.captured_requests();
    let chat_request = chat_requests.lock().unwrap()[0].clone();
    let responses_request = responses_requests.lock().unwrap()[0].clone();

    assert!(chat_request.system.is_none());
    assert!(responses_request.system.is_none());
    assert_eq!(chat_request.messages.len(), 3);
    assert_eq!(responses_request.messages.len(), 3);

    for idx in 0..3 {
        assert_eq!(
            chat_request.messages[idx].role,
            responses_request.messages[idx].role
        );
        match (
            &chat_request.messages[idx].content,
            &responses_request.messages[idx].content,
        ) {
            (MessageContent::Blocks(chat_blocks), MessageContent::Blocks(response_blocks)) => {
                assert_eq!(chat_blocks.len(), response_blocks.len());
                assert_eq!(chat_blocks[0].as_text(), response_blocks[0].as_text());
                assert_eq!(chat_blocks[1].as_text(), response_blocks[1].as_text());
            }
            (MessageContent::Text(left), MessageContent::Text(right)) => {
                assert_eq!(left, right);
            }
            other => panic!("expected matching normalized content, got {other:?}"),
        }
    }
}

#[tokio::test]
async fn x_provider_forcing_is_shared_and_misses_reject_consistently() {
    let chat_fixture = chat_sync_fixture();
    let (chat_harness, chat_primary, chat_secondary) = build_multi_provider_harness(
        StubProvider::new(chat_fixture.provider_response.clone(), vec![]),
        StubProvider::new(
            GatewayResponse {
                model: "secondary-actual".to_string(),
                ..chat_fixture.provider_response.clone()
            },
            vec![],
        ),
    );

    let chat_response = chat_harness
        .invoke_chat_completions(routing_header("secondary"), chat_sync_request())
        .await;
    assert_eq!(chat_response.status(), StatusCode::OK);
    let chat_body = response_text(chat_response).await;
    let chat_json: Value = serde_json::from_str(&chat_body).expect("chat completion json");
    assert_eq!(chat_json["model"], chat_sync_request()["model"]);
    assert_eq!(chat_primary.lock().unwrap().len(), 0);
    assert_eq!(chat_secondary.lock().unwrap().len(), 1);
    assert_eq!(chat_secondary.lock().unwrap()[0].model, "secondary-actual");

    let responses_fixture = responses_sync_fixture();
    let (responses_harness, responses_primary, responses_secondary) = build_multi_provider_harness(
        StubProvider::new(responses_fixture.provider_response.clone(), vec![]),
        StubProvider::new(
            GatewayResponse {
                model: "secondary-actual".to_string(),
                ..responses_fixture.provider_response.clone()
            },
            vec![],
        ),
    );

    let responses_response = responses_harness
        .invoke_responses(routing_header("secondary"), responses_sync_request())
        .await;
    assert_eq!(responses_response.status(), StatusCode::OK);
    let responses_body = response_text(responses_response).await;
    let responses_json: Value = serde_json::from_str(&responses_body).expect("responses json");
    assert_eq!(responses_json["model"], responses_public_model());
    assert_eq!(responses_primary.lock().unwrap().len(), 0);
    assert_eq!(responses_secondary.lock().unwrap().len(), 1);
    assert_eq!(
        responses_secondary.lock().unwrap()[0].model,
        "secondary-actual"
    );

    let chat_miss = chat_harness
        .invoke_chat_completions(routing_header("missing"), chat_sync_request())
        .await;
    let chat_miss_json = assert_error_contract(chat_miss, StatusCode::BAD_REQUEST);
    assert_route_error_shape(&chat_miss_json);

    let responses_miss = responses_harness
        .invoke_responses(routing_header("missing"), responses_sync_request())
        .await;
    let responses_miss_json = assert_error_contract(responses_miss, StatusCode::BAD_REQUEST);
    assert_route_error_shape(&responses_miss_json);

    assert_eq!(
        chat_miss_json["error"]["class"],
        responses_miss_json["error"]["class"]
    );
    assert_eq!(
        chat_miss_json["error"]["message"],
        responses_miss_json["error"]["message"]
    );
}

#[tokio::test]
async fn provider_failure_maps_to_shared_502_envelope() {
    let failing_chat = ConformanceHarness::with_registry(
        "gateway-default",
        {
            let mut registry = ProviderRegistry::new();
            registry.insert_provider_for_tests("test-provider", Box::new(make_failure_provider()));
            registry
        },
        vec![substrate_gateway::cli::ModelMapping {
            priority: 1,
            provider: "test-provider".to_string(),
            actual_model: "primary-actual".to_string(),
            inject_continuation_prompt: false,
        }],
    );

    let chat_response = failing_chat
        .invoke_chat_completions(HeaderMap::new(), chat_sync_request())
        .await;
    let chat_json = assert_error_contract(chat_response, StatusCode::BAD_GATEWAY);
    assert_provider_error_shape(
        &chat_json,
        "transport_drift",
        "Transport behavior drifted from the operator contract",
    );

    let responses_harness = ConformanceHarness::with_registry(
        "gateway-default",
        {
            let mut registry = ProviderRegistry::new();
            registry.insert_provider_for_tests("test-provider", Box::new(make_failure_provider()));
            registry
        },
        vec![substrate_gateway::cli::ModelMapping {
            priority: 1,
            provider: "test-provider".to_string(),
            actual_model: "primary-actual".to_string(),
            inject_continuation_prompt: false,
        }],
    );

    let responses_response = responses_harness
        .invoke_responses(HeaderMap::new(), responses_sync_request())
        .await;
    let responses_json = assert_error_contract(responses_response, StatusCode::BAD_GATEWAY);
    assert_provider_error_shape(
        &responses_json,
        "transport_drift",
        "Transport behavior drifted from the operator contract",
    );

    assert_eq!(
        chat_json["error"]["class"],
        responses_json["error"]["class"]
    );
    assert_eq!(
        chat_json["error"]["message"],
        responses_json["error"]["message"]
    );
}

#[tokio::test]
async fn reasoning_is_suppressed_in_sync_outputs_for_both_endpoints() {
    let sync_response = reasoning_sync_response();

    let chat_harness =
        build_single_provider_harness(StubProvider::new(sync_response.clone(), vec![]));
    let chat_response = chat_harness
        .invoke_chat_completions(HeaderMap::new(), chat_sync_request())
        .await;
    assert_eq!(chat_response.status(), StatusCode::OK);
    let chat_body = response_text(chat_response).await;
    assert!(chat_body.contains("Visible answer"));
    assert!(!chat_body.contains("secret reasoning"));
    assert!(!chat_body.contains("\"type\":\"thinking\""));

    let responses_harness = ConformanceHarness::single_provider(
        StubProvider::new(sync_response, vec![]),
        "primary-actual",
        false,
    );
    let responses_response = responses_harness
        .invoke_responses(HeaderMap::new(), responses_sync_request())
        .await;
    assert_eq!(responses_response.status(), StatusCode::OK);
    let responses_body = response_text(responses_response).await;
    assert!(responses_body.contains("Visible answer"));
    assert!(!responses_body.contains("secret reasoning"));
    assert!(!responses_body.contains("\"type\":\"thinking\""));
}

#[tokio::test]
async fn reasoning_is_suppressed_in_stream_outputs_for_both_endpoints() {
    let chat_harness = build_single_provider_harness(StubProvider::new(
        GatewayResponse {
            id: "resp_stream_chat".to_string(),
            r#type: "message".to_string(),
            role: "assistant".to_string(),
            content: vec![ContentBlock::text(
                "Visible stream answer".to_string(),
                None,
            )],
            model: "primary-actual".to_string(),
            stop_reason: Some("end_turn".to_string()),
            stop_sequence: None,
            usage: GatewayUsage {
                input_tokens: 4,
                output_tokens: 2,
                cache_creation_input_tokens: None,
                cache_read_input_tokens: None,
            },
        },
        reasoning_stream_chunks(),
    ));
    let chat_request = {
        let mut request = chat_sync_request();
        request["stream"] = json!(true);
        request
    };
    let chat_response = chat_harness
        .invoke_chat_completions(HeaderMap::new(), chat_request)
        .await;
    assert_eq!(chat_response.status(), StatusCode::OK);
    assert_eq!(
        chat_response
            .headers()
            .get("content-type")
            .and_then(|value| value.to_str().ok()),
        Some("text/event-stream")
    );
    let chat_body = response_text(chat_response).await;
    assert!(chat_body.contains("Visible stream answer"));
    assert!(chat_body.contains("\"model\":\"gateway-default\""));
    assert!(!chat_body.contains("secret reasoning"));
    assert!(!chat_body.contains("\"type\":\"thinking\""));

    let responses_harness = ConformanceHarness::single_provider(
        StubProvider::new(
            GatewayResponse {
                id: "resp_stream_responses".to_string(),
                r#type: "message".to_string(),
                role: "assistant".to_string(),
                content: vec![ContentBlock::text(
                    "Visible stream answer".to_string(),
                    None,
                )],
                model: "primary-actual".to_string(),
                stop_reason: Some("end_turn".to_string()),
                stop_sequence: None,
                usage: GatewayUsage {
                    input_tokens: 4,
                    output_tokens: 2,
                    cache_creation_input_tokens: None,
                    cache_read_input_tokens: None,
                },
            },
            reasoning_stream_chunks(),
        ),
        "primary-actual",
        false,
    );
    let mut request = responses_stream_fixture_request();
    request["stream"] = json!(true);
    let responses_response = responses_harness
        .invoke_responses(HeaderMap::new(), request)
        .await;
    assert_eq!(responses_response.status(), StatusCode::OK);
    assert_eq!(
        responses_response
            .headers()
            .get("content-type")
            .and_then(|value| value.to_str().ok()),
        Some("text/event-stream")
    );
    let responses_body = response_text(responses_response).await;
    assert!(responses_body.contains("Visible stream answer"));
    assert!(responses_body.contains("\"model\":\"gateway-default\""));
    assert!(!responses_body.contains("secret reasoning"));
    assert!(!responses_body.contains("\"type\":\"thinking\""));
}

#[derive(Debug, Clone)]
struct FailingProvider {
    api_status: u16,
    message: String,
}

#[async_trait]
impl GatewayProvider for FailingProvider {
    async fn send_message(
        &self,
        _request: GatewayRequest,
    ) -> Result<GatewayResponse, ProviderError> {
        Err(ProviderError::ApiError {
            status: self.api_status,
            message: self.message.clone(),
        })
    }

    async fn send_message_stream(
        &self,
        _request: GatewayRequest,
    ) -> Result<GatewayStreamResponse, ProviderError> {
        Err(ProviderError::ApiError {
            status: self.api_status,
            message: self.message.clone(),
        })
    }

    async fn count_tokens(
        &self,
        _request: substrate_gateway::models::CountTokensRequest,
    ) -> Result<substrate_gateway::models::CountTokensResponse, ProviderError> {
        Ok(substrate_gateway::models::CountTokensResponse { input_tokens: 0 })
    }

    fn supports_model(&self, _model: &str) -> bool {
        true
    }
}
