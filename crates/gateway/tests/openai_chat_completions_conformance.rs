use axum::http::{HeaderMap, StatusCode};
use serde::de::DeserializeOwned;
use serde::Deserialize;
use serde_json::{json, Value};
use substrate_gateway::core::GatewayResponse;
use substrate_gateway::server::openai_conformance_test_support::{
    read_json_fixture, response_text, ConformanceHarness, FixtureNamespace, StubProvider,
};

#[derive(Debug, Deserialize)]
struct SyncFixture {
    request: Value,
    provider_response: GatewayResponse,
    expected: SyncExpectation,
}

#[derive(Debug, Deserialize)]
struct SyncExpectation {
    object: String,
    finish_reason: String,
    texts: Vec<String>,
    tool_call_ids: Vec<String>,
    usage: UsageExpectation,
}

#[derive(Debug, Deserialize)]
struct UsageExpectation {
    prompt_tokens: u32,
    completion_tokens: u32,
    total_tokens: u32,
}

#[derive(Debug, Deserialize)]
struct StreamFixture {
    request: Value,
    provider_stream_chunks: Vec<String>,
    expected_contains: Vec<String>,
    expected_not_contains: Vec<String>,
    expect_usage_chunk: bool,
}

#[derive(Debug, Deserialize)]
struct NegativeFixture {
    request: Value,
    expected_status: u16,
    expected_class: String,
    expected_message: String,
}

fn load_fixture<T: DeserializeOwned>(name: &str) -> T {
    read_json_fixture(FixtureNamespace::OpenAiChatCompletions, name)
}

fn build_harness(provider: StubProvider) -> ConformanceHarness {
    ConformanceHarness::single_provider(provider, "primary-actual", false)
}

fn request_model(request: &Value) -> &str {
    request["model"].as_str().expect("fixture request model")
}

fn sse_payloads(body: &str) -> Vec<String> {
    body.split("\n\n")
        .filter_map(|block| {
            block
                .strip_prefix("data: ")
                .map(|payload| payload.trim().to_string())
        })
        .collect()
}

fn assert_public_error_envelope(body: &str, fixture: &NegativeFixture) {
    let json: Value = serde_json::from_str(body).expect("gateway error envelope");
    assert_eq!(json["error"]["type"].as_str(), Some("error"));
    assert_eq!(
        json["error"]["class"].as_str(),
        Some(fixture.expected_class.as_str())
    );
    assert_eq!(
        json["error"]["message"].as_str(),
        Some(fixture.expected_message.as_str())
    );
}

async fn assert_sync_fixture(name: &str) {
    let fixture: SyncFixture = load_fixture(name);
    let provider = StubProvider::new(fixture.provider_response.clone(), vec![]);
    let harness = build_harness(provider);

    let response = harness
        .invoke_chat_completions(HeaderMap::new(), fixture.request.clone())
        .await;

    assert_eq!(response.status(), StatusCode::OK, "{name}");

    let body = response_text(response).await;
    let json: Value = serde_json::from_str(&body).expect("chat completion json");

    assert_eq!(
        json["object"].as_str(),
        Some(fixture.expected.object.as_str()),
        "{name}"
    );
    assert_eq!(
        json["model"].as_str(),
        Some(request_model(&fixture.request)),
        "{name}"
    );
    assert_eq!(
        json["choices"].as_array().map(|choices| choices.len()),
        Some(1),
        "{name}"
    );

    let choice = &json["choices"][0];
    assert_eq!(choice["index"].as_u64(), Some(0), "{name}");
    assert_eq!(
        choice["message"]["role"].as_str(),
        Some("assistant"),
        "{name}"
    );
    assert_eq!(
        choice["finish_reason"].as_str(),
        Some(fixture.expected.finish_reason.as_str()),
        "{name}"
    );

    let expected_content = if fixture.expected.texts.is_empty() {
        None
    } else {
        Some(fixture.expected.texts.join("\n"))
    };
    assert_eq!(
        choice["message"]["content"].as_str(),
        expected_content.as_deref(),
        "{name}"
    );

    let tool_call_ids: Vec<String> = choice["message"]["tool_calls"]
        .as_array()
        .map(|tool_calls| {
            tool_calls
                .iter()
                .map(|tool_call| {
                    assert_eq!(tool_call["type"].as_str(), Some("function"), "{name}");
                    assert!(tool_call["function"]["arguments"].is_string(), "{name}");
                    tool_call["id"].as_str().unwrap().to_string()
                })
                .collect()
        })
        .unwrap_or_default();
    assert_eq!(tool_call_ids, fixture.expected.tool_call_ids, "{name}");

    assert_eq!(
        json["usage"]["prompt_tokens"].as_u64(),
        Some(u64::from(fixture.expected.usage.prompt_tokens)),
        "{name}"
    );
    assert_eq!(
        json["usage"]["completion_tokens"].as_u64(),
        Some(u64::from(fixture.expected.usage.completion_tokens)),
        "{name}"
    );
    assert_eq!(
        json["usage"]["total_tokens"].as_u64(),
        Some(u64::from(fixture.expected.usage.total_tokens)),
        "{name}"
    );

    let requests = harness.captured_requests();
    let requests = requests.lock().unwrap();
    assert_eq!(requests.len(), 1, "{name}");
    assert_eq!(requests[0].model, "primary-actual", "{name}");
}

async fn assert_stream_fixture(name: &str) {
    let fixture: StreamFixture = load_fixture(name);
    let provider = StubProvider::new(
        substrate_gateway::server::openai_conformance_test_support::response_text_response(
            "unused",
            "primary-actual",
        ),
        fixture.provider_stream_chunks.clone(),
    );
    let harness = build_harness(provider);

    let response = harness
        .invoke_chat_completions(HeaderMap::new(), fixture.request.clone())
        .await;

    assert_eq!(response.status(), StatusCode::OK, "{name}");
    assert_eq!(
        response
            .headers()
            .get("content-type")
            .and_then(|v| v.to_str().ok()),
        Some("text/event-stream"),
        "{name}"
    );

    let body = response_text(response).await;
    for fragment in &fixture.expected_contains {
        assert!(body.contains(fragment), "{name}: missing {fragment}");
    }
    for fragment in &fixture.expected_not_contains {
        assert!(!body.contains(fragment), "{name}: unexpected {fragment}");
    }
    assert!(!body.contains("event: "), "{name}");
    assert!(body.contains("data: [DONE]"), "{name}");

    let payloads = sse_payloads(&body);
    assert!(payloads.iter().any(|payload| payload == "[DONE]"), "{name}");

    let json_payloads: Vec<Value> = payloads
        .iter()
        .filter(|payload| payload.as_str() != "[DONE]")
        .map(|payload| serde_json::from_str(payload).expect("chunk json"))
        .collect();
    assert!(
        json_payloads
            .iter()
            .all(|payload| payload["object"].as_str() == Some("chat.completion.chunk")),
        "{name}"
    );
    assert!(
        json_payloads
            .iter()
            .all(|payload| payload["model"].as_str() == Some(request_model(&fixture.request))),
        "{name}"
    );

    if fixture.expect_usage_chunk {
        assert!(
            json_payloads.iter().any(|payload| {
                payload
                    .get("choices")
                    .and_then(|choices| choices.as_array())
                    .is_some_and(|choices| choices.is_empty())
                    && payload.get("usage").is_some()
            }),
            "{name}"
        );
    }

    let requests = harness.captured_requests();
    let requests = requests.lock().unwrap();
    assert_eq!(requests.len(), 1, "{name}");
    assert_eq!(requests[0].model, "primary-actual", "{name}");
    assert_eq!(requests[0].stream, Some(true), "{name}");
}

#[tokio::test]
async fn sync_fixtures_cover_text_and_mixed_output() {
    for name in ["sync-text.json", "sync-mixed.json"] {
        assert_sync_fixture(name).await;
    }
}

#[tokio::test]
async fn sync_tool_loop_fixture_preserves_tool_call_id() {
    let fixture: SyncFixture = load_fixture("sync-tool-call.json");
    let provider = StubProvider::new(fixture.provider_response.clone(), vec![]);
    let harness = build_harness(provider);

    let response = harness
        .invoke_chat_completions(HeaderMap::new(), fixture.request.clone())
        .await;

    assert_eq!(response.status(), StatusCode::OK);
    let body = response_text(response).await;
    let json: Value = serde_json::from_str(&body).expect("chat completion json");

    assert_eq!(
        json["choices"][0]["finish_reason"].as_str(),
        Some("tool_calls")
    );
    assert_eq!(
        json["choices"][0]["message"]["tool_calls"][0]["id"].as_str(),
        Some("call_fixture_1")
    );

    let requests = harness.captured_requests();
    let requests = requests.lock().unwrap();
    assert_eq!(requests.len(), 1);
    assert_eq!(requests[0].messages.len(), 2);
    assert_eq!(requests[0].messages[1].role, "user");
}

#[tokio::test]
async fn explicit_function_tool_choice_filters_tools_to_the_selected_function() {
    let fixture: SyncFixture = load_fixture("sync-tool-choice-function-selection.json");
    let provider = StubProvider::new(fixture.provider_response.clone(), vec![]);
    let harness = build_harness(provider);

    let response = harness
        .invoke_chat_completions(HeaderMap::new(), fixture.request.clone())
        .await;

    assert_eq!(response.status(), StatusCode::OK);
    let body = response_text(response).await;
    let json: Value = serde_json::from_str(&body).expect("chat completion json");
    assert_eq!(json["object"].as_str(), Some("chat.completion"));
    assert_eq!(json["model"].as_str(), Some("gateway-default"));
    assert_eq!(json["choices"][0]["finish_reason"].as_str(), Some("stop"));
    assert_eq!(
        json["choices"][0]["message"]["content"].as_str(),
        Some("Boundary text from fixture.")
    );

    let requests = harness.captured_requests();
    let requests = requests.lock().unwrap();
    assert_eq!(requests.len(), 1);
    let tools = requests[0].tools.as_ref().expect("filtered tools");
    assert_eq!(tools.len(), 1);
    assert_eq!(tools[0].name.as_deref(), Some("beta"));
}

#[tokio::test]
async fn unknown_top_level_fields_are_ignored_at_the_route_boundary() {
    let fixture: SyncFixture = load_fixture("sync-text.json");
    let provider = StubProvider::new(fixture.provider_response.clone(), vec![]);
    let harness = build_harness(provider);

    let mut request = fixture.request.clone();
    request["ignored_future_field"] = json!({"nested": true});

    let response = harness
        .invoke_chat_completions(HeaderMap::new(), request)
        .await;
    assert_eq!(response.status(), StatusCode::OK);

    let body = response_text(response).await;
    let json: Value = serde_json::from_str(&body).expect("chat completion json");
    assert_eq!(json["model"].as_str(), Some("gateway-default"));

    let requests = harness.captured_requests();
    let requests = requests.lock().unwrap();
    assert_eq!(requests.len(), 1);
}

#[tokio::test]
async fn stream_fixtures_cover_text_tool_and_usage_cases() {
    for name in [
        "stream-text.json",
        "stream-tool-call.json",
        "stream-mixed.json",
        "stream-include-usage.json",
    ] {
        assert_stream_fixture(name).await;
    }
}

#[tokio::test]
async fn negative_fixtures_return_redacted_gateway_error_envelope() {
    for name in [
        "negative-unsupported-field.json",
        "negative-non-function-tool.json",
        "negative-tool-call-id-mismatch.json",
        "negative-system-developer-image.json",
    ] {
        let fixture: NegativeFixture = load_fixture(name);
        let provider = StubProvider::new(
            substrate_gateway::server::openai_conformance_test_support::response_text_response(
                "unused",
                "primary-actual",
            ),
            vec![],
        );
        let harness = build_harness(provider);

        let response = harness
            .invoke_chat_completions(HeaderMap::new(), fixture.request.clone())
            .await;

        assert_eq!(
            response.status().as_u16(),
            fixture.expected_status,
            "{name}"
        );
        let body = response_text(response).await;
        assert_public_error_envelope(&body, &fixture);

        let requests = harness.captured_requests();
        let requests = requests.lock().unwrap();
        assert!(requests.is_empty(), "{name}");
    }
}
