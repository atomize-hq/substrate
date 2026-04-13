use serde::Deserialize;
use serde_json::json;
use substrate_gateway::core::GatewayResponse;
use substrate_gateway::server::openai_conformance_test_support::{
    read_json_fixture, response_text as response_body_text, split_string_into_chunks,
    ConformanceHarness, FixtureNamespace, StubProvider,
};

#[derive(Debug, Deserialize)]
struct SmokeFixture {
    request: serde_json::Value,
    provider_response: GatewayResponse,
    provider_stream_chunks: Vec<String>,
}

#[tokio::test]
async fn openai_chat_completions_harness_runs_sync_and_stream_offline() {
    let fixture: SmokeFixture = read_json_fixture(
        FixtureNamespace::OpenAiChatCompletions,
        "harness-smoke.json",
    );
    let provider = StubProvider::new(
        fixture.provider_response.clone(),
        fixture.provider_stream_chunks.clone(),
    );
    let harness = ConformanceHarness::single_provider(provider, "primary-actual", false);
    let captured_requests = harness.captured_requests();

    let sync_response = harness
        .invoke_chat_completions(axum::http::HeaderMap::new(), fixture.request.clone())
        .await;

    assert_eq!(sync_response.status(), axum::http::StatusCode::OK);
    let sync_body = response_body_text(sync_response).await;
    assert!(sync_body.contains("\"object\":\"chat.completion\""));
    assert!(sync_body.contains("\"model\":\"gateway-default\""));
    assert!(sync_body.contains("Boundary text from fixture."));

    let mut stream_request = fixture.request.clone();
    stream_request["stream"] = json!(true);

    let stream_response = harness
        .invoke_chat_completions(axum::http::HeaderMap::new(), stream_request)
        .await;

    assert_eq!(stream_response.status(), axum::http::StatusCode::OK);
    assert_eq!(
        stream_response.headers().get("content-type").unwrap(),
        "text/event-stream"
    );

    let stream_body = response_body_text(stream_response).await;
    assert!(stream_body.contains("\"object\":\"chat.completion.chunk\""));
    assert!(stream_body.contains("\"content\":\"Boundary\""));
    assert!(stream_body.contains("data: [DONE]"));
    assert!(!stream_body.contains("event: "));
    assert!(!stream_body.contains("content_block_delta"));

    let requests = captured_requests.lock().unwrap();
    assert_eq!(requests.len(), 2);
    assert_eq!(requests[0].stream, Some(false));
    assert_eq!(requests[1].stream, Some(true));

    let split_chunks = split_string_into_chunks("abcdef", &[2, 4]);
    assert_eq!(split_chunks, vec!["ab", "cd", "ef"]);
}
