use axum::http::{HeaderMap, StatusCode};
use mockito::{Matcher, Server};
use serde::Deserialize;
use serde_json::{json, Value};
use substrate_gateway::auth::TokenStore;
use substrate_gateway::cli::ModelMapping;
use substrate_gateway::core::GatewayResponse;
use substrate_gateway::providers::streaming::parse_sse_events;
use substrate_gateway::providers::{AuthType, ProviderConfig, ProviderRegistry};
use substrate_gateway::server::openai_conformance_test_support::{
    read_json_fixture, response_text as response_body_text, ConformanceHarness, FixtureNamespace,
    StubProvider,
};

const OPENAI_RESPONSES_TOOL_CHOICE_METADATA_KEY: &str = "openai_responses_tool_choice";
const OPENAI_RESPONSES_REASONING_EFFORT_METADATA_KEY: &str = "openai_responses_reasoning_effort";
const OPENAI_RESPONSES_REASONING_SUMMARY_METADATA_KEY: &str = "openai_responses_reasoning_summary";
const OPENAI_RESPONSES_INCLUDE_METADATA_KEY: &str = "openai_responses_include";
const OPENAI_RESPONSES_TEXT_VERBOSITY_METADATA_KEY: &str = "openai_responses_text_verbosity";

#[derive(Debug, Deserialize)]
struct UsageExpected {
    input_tokens: u32,
    output_tokens: u32,
    total_tokens: u32,
}

#[derive(Debug, Deserialize)]
struct ExpectedShape {
    status: String,
    item_types: Vec<String>,
    texts: Vec<String>,
    call_ids: Vec<String>,
    usage: UsageExpected,
}

#[derive(Debug, Deserialize)]
struct SyncFixture {
    request: Value,
    provider_response: GatewayResponse,
    expected: ExpectedShape,
}

#[derive(Debug, Deserialize)]
struct StreamFixture {
    request: Value,
    provider_stream_chunks: Vec<String>,
    required_events: Vec<String>,
    expected: ExpectedShape,
    forbidden_fragments: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct NegativeFixture {
    request: Value,
    expected_error_contains: String,
    expected_status: u16,
    expected_error_class: String,
    expected_error_message: String,
}

#[derive(Debug, Deserialize)]
struct ToolLoopFixture {
    request: Value,
    expected_tool_use_id: String,
    expected_output: String,
}

#[derive(Debug, Deserialize)]
struct SupportedControlsFixture {
    request: Value,
}

fn collect_output_summary(output: &Value) -> (Vec<String>, Vec<String>, Vec<String>) {
    let mut item_types = Vec::new();
    let mut texts = Vec::new();
    let mut call_ids = Vec::new();

    for item in output.as_array().expect("response.output must be an array") {
        match item.get("type").and_then(Value::as_str) {
            Some("message") => {
                item_types.push("message".to_string());
                let mut text_parts = Vec::new();
                if let Some(parts) = item.get("content").and_then(Value::as_array) {
                    for part in parts {
                        if let Some(text) = part.get("text").and_then(Value::as_str) {
                            text_parts.push(text.to_string());
                        }
                    }
                }
                texts.push(text_parts.join("\n"));
            }
            Some("function_call") => {
                item_types.push("function_call".to_string());
                if let Some(call_id) = item.get("call_id").and_then(Value::as_str) {
                    call_ids.push(call_id.to_string());
                }
            }
            Some(other) => item_types.push(other.to_string()),
            None => item_types.push("".to_string()),
        }
    }

    (item_types, texts, call_ids)
}

fn parse_sse_body(body: &str) -> Vec<(String, Value)> {
    parse_sse_events(body)
        .into_iter()
        .map(|event| {
            let name = event.event.expect("SSE event name should be present");
            let json: Value = serde_json::from_str(&event.data).expect("valid SSE JSON");
            assert_eq!(
                json.get("type").and_then(Value::as_str),
                Some(name.as_str()),
                "SSE data.type must match event name"
            );
            (name, json)
        })
        .collect()
}

fn build_sync_harness(provider_response: &GatewayResponse) -> ConformanceHarness {
    let provider = StubProvider::new(provider_response.clone(), vec![]);
    ConformanceHarness::single_provider(provider, provider_response.model.clone(), false)
}

fn build_stream_harness(stream_chunks: Vec<String>) -> ConformanceHarness {
    let provider = StubProvider::new(
        substrate_gateway::server::openai_conformance_test_support::response_text_response(
            "unused",
            "primary-actual",
        ),
        stream_chunks,
    );
    ConformanceHarness::single_provider(provider, "primary-actual", false)
}

fn build_openai_provider_harness(base_url: &str, actual_model: &str) -> ConformanceHarness {
    let registry = ProviderRegistry::from_configs_with_models(
        &[ProviderConfig {
            name: "openai".to_string(),
            provider_type: "openai".to_string(),
            auth_type: AuthType::ApiKey,
            api_key: Some("openai-secret".to_string()),
            oauth_provider: None,
            project_id: None,
            location: None,
            base_url: Some(format!("{}/v1", base_url)),
            headers: None,
            models: vec![],
            enabled: Some(true),
        }],
        None,
        &[],
    )
    .unwrap();

    ConformanceHarness::with_registry(
        "gateway-default",
        registry,
        vec![ModelMapping {
            priority: 1,
            provider: "openai".to_string(),
            actual_model: actual_model.to_string(),
            inject_continuation_prompt: false,
        }],
    )
}

fn build_codex_oauth_provider_harness(actual_model: &str) -> ConformanceHarness {
    let token_store = TokenStore::new(std::env::temp_dir().join(format!(
        "substrate-gateway-codex-conformance-{}.json",
        uuid::Uuid::new_v4()
    )))
    .unwrap();
    let registry = ProviderRegistry::from_configs_with_models(
        &[ProviderConfig {
            name: "openai-codex".to_string(),
            provider_type: "openai".to_string(),
            auth_type: AuthType::OAuth,
            api_key: None,
            oauth_provider: Some("test-oauth-provider".to_string()),
            project_id: None,
            location: None,
            base_url: Some("https://chatgpt.com/backend-api".to_string()),
            headers: None,
            models: vec![],
            enabled: Some(true),
        }],
        Some(token_store),
        &[],
    )
    .unwrap();

    ConformanceHarness::with_registry(
        "gateway-default",
        registry,
        vec![ModelMapping {
            priority: 1,
            provider: "openai-codex".to_string(),
            actual_model: actual_model.to_string(),
            inject_continuation_prompt: false,
        }],
    )
}

fn responses_api_sync_body(model: &str, text: &str) -> String {
    format!(
        "event: response.completed\ndata: {}\n\n",
        json!({
            "type": "response.completed",
            "response": {
                "id": "resp_live_regression",
                "model": model,
                "output": [
                    {
                        "type": "message",
                        "content": [
                            {
                                "type": "output_text",
                                "text": text
                            }
                        ]
                    }
                ],
                "usage": {
                    "input_tokens": 1,
                    "output_tokens": 1
                }
            }
        })
    )
}

#[tokio::test]
async fn sync_responses_conformance_cases_cover_shape_order_and_ignore_posture() {
    for fixture_name in [
        "codex-sync-text.json",
        "codex-sync-tool-call.json",
        "codex-sync-mixed.json",
    ] {
        let fixture: SyncFixture =
            read_json_fixture(FixtureNamespace::OpenAiResponses, fixture_name);
        let harness = build_sync_harness(&fixture.provider_response);

        let response = harness
            .invoke_responses(HeaderMap::new(), fixture.request.clone())
            .await;

        assert_eq!(response.status(), StatusCode::OK, "{fixture_name}");
        let body = response_body_text(response).await;
        let json: Value = serde_json::from_str(&body).expect("sync response body must be JSON");

        assert_eq!(json["object"], "response", "{fixture_name}");
        assert_eq!(json["status"], fixture.expected.status, "{fixture_name}");
        assert_eq!(json["model"], fixture.request["model"], "{fixture_name}");

        let (item_types, texts, call_ids) = collect_output_summary(&json["output"]);
        assert_eq!(item_types, fixture.expected.item_types, "{fixture_name}");
        assert_eq!(texts, fixture.expected.texts, "{fixture_name}");
        assert_eq!(call_ids, fixture.expected.call_ids, "{fixture_name}");
        assert_eq!(
            json["usage"]["input_tokens"], fixture.expected.usage.input_tokens,
            "{fixture_name}"
        );
        assert_eq!(
            json["usage"]["output_tokens"], fixture.expected.usage.output_tokens,
            "{fixture_name}"
        );
        assert_eq!(
            json["usage"]["total_tokens"], fixture.expected.usage.total_tokens,
            "{fixture_name}"
        );
    }
}

#[tokio::test]
async fn sync_responses_conformance_keeps_reasoning_internal() {
    let provider_response = GatewayResponse {
        id: "resp_reasoning_internal".to_string(),
        r#type: "message".to_string(),
        role: "assistant".to_string(),
        content: vec![
            substrate_gateway::models::ContentBlock::thinking(serde_json::json!({
                "thinking": "secret reasoning"
            })),
            substrate_gateway::models::ContentBlock::text("Visible answer".to_string(), None),
        ],
        model: "primary-actual".to_string(),
        stop_reason: Some("end_turn".to_string()),
        stop_sequence: None,
        usage: substrate_gateway::core::GatewayUsage {
            input_tokens: 3,
            output_tokens: 2,
            cache_creation_input_tokens: None,
            cache_read_input_tokens: None,
        },
    };

    let harness = build_sync_harness(&provider_response);
    let request = json!({
        "model": "gateway-default",
        "input": "hello",
        "stream": false
    });
    let response = harness.invoke_responses(HeaderMap::new(), request).await;

    assert_eq!(response.status(), StatusCode::OK);
    let body = response_body_text(response).await;
    assert!(!body.contains("secret reasoning"));
    assert!(body.contains("Visible answer"));

    let json: Value = serde_json::from_str(&body).expect("sync response body must be JSON");
    let (item_types, texts, call_ids) = collect_output_summary(&json["output"]);
    assert_eq!(item_types, vec!["message"]);
    assert_eq!(texts, vec!["Visible answer"]);
    assert!(call_ids.is_empty());
}

#[tokio::test]
async fn tool_loop_continuation_preserves_call_id_through_the_public_route() {
    let fixture: ToolLoopFixture = read_json_fixture(
        FixtureNamespace::OpenAiResponses,
        "codex-request-tool-loop-function-call-output.json",
    );
    let provider = StubProvider::new(
        substrate_gateway::server::openai_conformance_test_support::response_text_response(
            "tool-loop-ok",
            "primary-actual",
        ),
        vec![],
    );
    let harness = ConformanceHarness::single_provider(provider, "primary-actual", false);
    let captured_requests = harness.captured_requests();

    let response = harness
        .invoke_responses(HeaderMap::new(), fixture.request.clone())
        .await;

    assert_eq!(response.status(), StatusCode::OK);
    let body = response_body_text(response).await;
    let json: Value = serde_json::from_str(&body).expect("tool-loop response body must be JSON");
    assert_eq!(json["object"], "response");
    assert_eq!(json["status"], "completed");

    let requests = captured_requests.lock().unwrap();
    assert_eq!(requests.len(), 1);
    let last_message = requests[0].messages.last().expect("captured request");
    let blocks = match &last_message.content {
        substrate_gateway::models::MessageContent::Blocks(blocks) => blocks,
        other => panic!("expected tool result blocks, got {other:?}"),
    };
    assert!(matches!(
        &blocks[0],
        substrate_gateway::models::ContentBlock::Known(
            substrate_gateway::models::KnownContentBlock::ToolResult { tool_use_id, content, .. }
        ) if tool_use_id == &fixture.expected_tool_use_id
            && matches!(
                content,
                substrate_gateway::models::ToolResultContent::Text(text)
                    if text == &fixture.expected_output
            )
    ));
}

#[tokio::test]
async fn tool_loop_continuation_uses_upstream_responses_api_and_preserves_function_call_output_shape(
) {
    let fixture: ToolLoopFixture = read_json_fixture(
        FixtureNamespace::OpenAiResponses,
        "codex-request-tool-loop-function-call-output.json",
    );
    let mut server = Server::new_async().await;
    let responses_mock = server
        .mock("POST", "/v1/responses")
        .match_header("authorization", "Bearer openai-secret")
        .match_body(Matcher::PartialJson(json!({
            "model": "gpt-4.1-mini",
            "stream": true,
            "input": [
                {
                    "type": "message",
                    "role": "assistant",
                    "content": [
                        {
                            "type": "output_text",
                            "text": "Need tool output."
                        }
                    ]
                },
                {
                    "type": "function_call",
                    "call_id": fixture.expected_tool_use_id,
                    "name": "tool_call",
                    "arguments": "{}"
                },
                {
                    "type": "function_call_output",
                    "call_id": fixture.expected_tool_use_id,
                    "output": fixture.expected_output
                }
            ]
        })))
        .with_status(200)
        .with_body(responses_api_sync_body("gpt-4.1-mini", "tool-loop-ok"))
        .create_async()
        .await;
    let chat_mock = server
        .mock("POST", "/v1/chat/completions")
        .with_status(200)
        .with_body(
            r#"{
            "id":"chatcmpl_wrong_endpoint",
            "object":"chat.completion",
            "model":"gpt-4.1-mini",
            "choices":[{
                "message":{"role":"assistant","content":"wrong endpoint"},
                "finish_reason":"stop"
            }],
            "usage":{"prompt_tokens":1,"completion_tokens":1,"total_tokens":2}
        }"#,
        )
        .create_async()
        .await;

    let harness = build_openai_provider_harness(&server.url(), "gpt-4.1-mini");
    let response = harness
        .invoke_responses(HeaderMap::new(), fixture.request.clone())
        .await;

    assert!(
        responses_mock.matched_async().await,
        "gateway should forward public responses continuations to /v1/responses with function_call_output input items"
    );
    assert!(
        !chat_mock.matched_async().await,
        "gateway must not lower public responses continuations into /v1/chat/completions tool messages"
    );
    let status = response.status();
    let body = response_body_text(response).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let json: Value =
        serde_json::from_str(&body).expect("tool-loop regression response body must be JSON");
    assert_eq!(json["object"], "response");
    assert_eq!(json["status"], "completed");

    let (item_types, texts, call_ids) = collect_output_summary(&json["output"]);
    assert_eq!(item_types, vec!["message".to_string()]);
    assert_eq!(texts, vec!["tool-loop-ok".to_string()]);
    assert!(call_ids.is_empty());

    responses_mock.assert_async().await;
}

#[tokio::test]
async fn parallel_tool_calls_is_preserved_on_the_public_responses_route() {
    let provider = StubProvider::new(
        substrate_gateway::server::openai_conformance_test_support::response_text_response(
            "parallel-tool-calls-ok",
            "primary-actual",
        ),
        vec![],
    );
    let harness = ConformanceHarness::single_provider(provider, "primary-actual", false);

    let request = serde_json::json!({
        "model": "gateway-default",
        "input": "hello",
        "parallel_tool_calls": false,
        "stream": false
    });

    let response = harness.invoke_responses(HeaderMap::new(), request).await;

    assert_eq!(response.status(), StatusCode::OK);

    let captured_requests = harness.captured_requests();
    let captured_requests = captured_requests.lock().unwrap();
    assert_eq!(captured_requests.len(), 1);
    assert_eq!(
        captured_requests[0]
            .metadata
            .as_ref()
            .and_then(|metadata| metadata.get("parallel_tool_calls")),
        Some(&serde_json::json!(false))
    );
}

#[tokio::test]
async fn supported_route_matrix_controls_are_preserved_in_gateway_request_shape() {
    let fixture: SupportedControlsFixture = read_json_fixture(
        FixtureNamespace::OpenAiResponses,
        "codex-supported-controls.json",
    );
    let provider = StubProvider::new(
        substrate_gateway::server::openai_conformance_test_support::response_text_response(
            "matrix-ok",
            "primary-actual",
        ),
        vec![],
    );
    let harness = ConformanceHarness::single_provider(provider, "primary-actual", false);

    let response = harness
        .invoke_responses(HeaderMap::new(), fixture.request.clone())
        .await;

    assert_eq!(response.status(), StatusCode::OK);

    let captured_requests = harness.captured_requests();
    let captured_requests = captured_requests.lock().unwrap();
    assert_eq!(captured_requests.len(), 1);

    let captured = &captured_requests[0];
    assert_eq!(captured.stream, Some(false));
    assert_eq!(
        captured
            .metadata
            .as_ref()
            .and_then(|metadata| metadata.get("parallel_tool_calls")),
        Some(&serde_json::json!(false))
    );
    assert_eq!(
        captured
            .metadata
            .as_ref()
            .and_then(|metadata| metadata.get(OPENAI_RESPONSES_REASONING_EFFORT_METADATA_KEY)),
        Some(&serde_json::json!("low"))
    );
    assert_eq!(
        captured
            .metadata
            .as_ref()
            .and_then(|metadata| metadata.get(OPENAI_RESPONSES_REASONING_SUMMARY_METADATA_KEY)),
        Some(&serde_json::json!("concise"))
    );
    assert_eq!(
        captured
            .metadata
            .as_ref()
            .and_then(|metadata| metadata.get(OPENAI_RESPONSES_INCLUDE_METADATA_KEY)),
        Some(&serde_json::json!(["reasoning.encrypted_content"]))
    );
    assert_eq!(
        captured
            .metadata
            .as_ref()
            .and_then(|metadata| metadata.get(OPENAI_RESPONSES_TEXT_VERBOSITY_METADATA_KEY)),
        Some(&serde_json::json!("low"))
    );
    assert_eq!(
        captured
            .metadata
            .as_ref()
            .and_then(|metadata| metadata.get(OPENAI_RESPONSES_TOOL_CHOICE_METADATA_KEY)),
        Some(&serde_json::json!({
            "type": "function",
            "function": { "name": "lookup" }
        }))
    );
    assert_eq!(captured.tools.as_ref().map(|tools| tools.len()), Some(1));
    assert_eq!(
        captured
            .tools
            .as_ref()
            .and_then(|tools| tools[0].name.as_deref()),
        Some("lookup")
    );

    assert_eq!(captured.messages.len(), 1);
    let blocks = match &captured.messages[0].content {
        substrate_gateway::models::MessageContent::Blocks(blocks) => blocks,
        other => panic!("expected multimodal blocks, got {other:?}"),
    };
    assert!(matches!(
        &blocks[0],
        substrate_gateway::models::ContentBlock::Known(
            substrate_gateway::models::KnownContentBlock::Text { text, .. }
        ) if text == "Describe this image"
    ));
    assert!(matches!(
        &blocks[1],
        substrate_gateway::models::ContentBlock::Known(
            substrate_gateway::models::KnownContentBlock::Image { source }
        ) if source.url.as_deref() == Some("https://example.com/image.png")
    ));
}

#[tokio::test]
async fn explicit_tool_choice_is_preserved_in_metadata_and_flat_tools() {
    let provider = StubProvider::new(
        substrate_gateway::server::openai_conformance_test_support::response_text_response(
            "tool-choice-ok",
            "primary-actual",
        ),
        vec![],
    );
    let harness = ConformanceHarness::single_provider(provider, "primary-actual", false);

    let request = serde_json::json!({
        "model": "gateway-default",
        "input": "hello",
        "stream": false,
        "tools": [
            { "type": "function", "function": { "name": "alpha", "parameters": {"type":"object"} } },
            { "type": "function", "function": { "name": "beta", "parameters": {"type":"object"} } }
        ],
        "tool_choice": { "type": "function", "function": { "name": "beta" } }
    });

    let response = harness.invoke_responses(HeaderMap::new(), request).await;
    assert_eq!(response.status(), StatusCode::OK);

    let captured_requests = harness.captured_requests();
    let captured_requests = captured_requests.lock().unwrap();
    assert_eq!(captured_requests.len(), 1);
    let captured = &captured_requests[0];

    assert_eq!(
        captured
            .metadata
            .as_ref()
            .and_then(|metadata| metadata.get(OPENAI_RESPONSES_TOOL_CHOICE_METADATA_KEY)),
        Some(&serde_json::json!({
            "type": "function",
            "function": { "name": "beta" }
        }))
    );
    assert_eq!(captured.tools.as_ref().map(|tools| tools.len()), Some(1));
    assert_eq!(
        captured
            .tools
            .as_ref()
            .and_then(|tools| tools[0].name.as_deref()),
        Some("beta")
    );
}

#[tokio::test]
async fn streaming_responses_conformance_cases_cover_event_order_and_completed_payload() {
    for fixture_name in [
        "codex-stream-text.json",
        "codex-stream-tool-call.json",
        "codex-stream-mixed.json",
        "codex-stream-with-usage.json",
    ] {
        let fixture: StreamFixture =
            read_json_fixture(FixtureNamespace::OpenAiResponses, fixture_name);
        let harness = build_stream_harness(fixture.provider_stream_chunks.clone());

        let response = harness
            .invoke_responses(HeaderMap::new(), fixture.request.clone())
            .await;

        assert_eq!(response.status(), StatusCode::OK, "{fixture_name}");
        assert_eq!(
            response.headers().get("content-type").unwrap(),
            "text/event-stream",
            "{fixture_name}"
        );

        let body = response_body_text(response).await;
        for fragment in &fixture.forbidden_fragments {
            assert!(
                !body.contains(fragment),
                "{fixture_name}: forbidden fragment leaked: {fragment}"
            );
        }

        let events = parse_sse_body(&body);
        let event_names: Vec<String> = events.iter().map(|(name, _)| name.clone()).collect();
        assert_eq!(event_names, fixture.required_events, "{fixture_name}");

        let completed = events
            .iter()
            .rev()
            .find(|(name, _)| name == "response.completed")
            .expect("stream must terminate with response.completed")
            .1
            .get("response")
            .cloned()
            .expect("completed event must carry a response");

        assert_eq!(completed["object"], "response", "{fixture_name}");
        assert_eq!(
            completed["status"], fixture.expected.status,
            "{fixture_name}"
        );
        assert_eq!(
            completed["model"], fixture.request["model"],
            "{fixture_name}"
        );

        let (item_types, texts, call_ids) = collect_output_summary(&completed["output"]);
        assert_eq!(item_types, fixture.expected.item_types, "{fixture_name}");
        assert_eq!(texts, fixture.expected.texts, "{fixture_name}");
        assert_eq!(call_ids, fixture.expected.call_ids, "{fixture_name}");
        assert_eq!(
            completed["usage"]["input_tokens"], fixture.expected.usage.input_tokens,
            "{fixture_name}"
        );
        assert_eq!(
            completed["usage"]["output_tokens"], fixture.expected.usage.output_tokens,
            "{fixture_name}"
        );
        assert_eq!(
            completed["usage"]["total_tokens"], fixture.expected.usage.total_tokens,
            "{fixture_name}"
        );
    }
}

#[tokio::test]
async fn generic_negative_responses_requests_return_redacted_gateway_envelopes() {
    for fixture_name in [
        "codex-negative-built-in-tool.json",
        "codex-negative-non-function-tool.json",
        "codex-negative-unsupported-text-format.json",
        "codex-negative-invalid-call-id.json",
    ] {
        let fixture: NegativeFixture =
            read_json_fixture(FixtureNamespace::OpenAiResponses, fixture_name);
        let provider = StubProvider::new(
            substrate_gateway::server::openai_conformance_test_support::response_text_response(
                "unused",
                "primary-actual",
            ),
            vec![],
        );
        let harness = ConformanceHarness::single_provider(provider, "primary-actual", false);

        let response = harness
            .invoke_responses(HeaderMap::new(), fixture.request.clone())
            .await;

        assert_eq!(
            response.status(),
            StatusCode::from_u16(fixture.expected_status).unwrap(),
            "{fixture_name}"
        );

        let body = response_body_text(response).await;
        assert!(
            !body.contains(&fixture.expected_error_contains),
            "{fixture_name}: public error body should stay redacted"
        );
        let json: Value = serde_json::from_str(&body).expect("negative response body must be JSON");

        assert_eq!(json["error"]["type"], "error", "{fixture_name}");
        assert_eq!(
            json["error"]["class"], fixture.expected_error_class,
            "{fixture_name}"
        );
        assert_eq!(
            json["error"]["message"], fixture.expected_error_message,
            "{fixture_name}"
        );
    }
}

#[tokio::test]
async fn codex_route_negative_requests_return_redacted_gateway_envelopes() {
    for fixture_name in [
        "codex-negative-max-output-tokens.json",
        "codex-negative-metadata.json",
        "codex-negative-truncation.json",
        "codex-negative-previous-response-id.json",
        "codex-negative-temperature.json",
        "codex-negative-top-p.json",
        "codex-negative-user.json",
        "codex-negative-service-tier.json",
        "codex-negative-stream-options.json",
        "codex-negative-required-tool-choice.json",
        "codex-negative-invalid-reasoning-summary.json",
        "codex-negative-encrypted-include-without-reasoning.json",
    ] {
        let fixture: NegativeFixture =
            read_json_fixture(FixtureNamespace::OpenAiResponses, fixture_name);
        let harness = build_codex_oauth_provider_harness("codex-mini-latest");

        let response = harness
            .invoke_responses(HeaderMap::new(), fixture.request.clone())
            .await;

        assert_eq!(
            response.status(),
            StatusCode::from_u16(fixture.expected_status).unwrap(),
            "{fixture_name}"
        );

        let body = response_body_text(response).await;
        assert!(
            !body.contains(&fixture.expected_error_contains),
            "{fixture_name}: public error body should stay redacted"
        );
        let json: Value = serde_json::from_str(&body).expect("negative response body must be JSON");

        assert_eq!(json["error"]["type"], "error", "{fixture_name}");
        assert_eq!(
            json["error"]["class"], fixture.expected_error_class,
            "{fixture_name}"
        );
        assert_eq!(
            json["error"]["message"], fixture.expected_error_message,
            "{fixture_name}"
        );
    }
}
