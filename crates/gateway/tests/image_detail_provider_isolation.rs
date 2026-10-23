//! GW-A06: local HTTP proves existing non-OpenAI serializers retain their bodies.
//! These tests do not establish acceptance by a live external provider.
use futures::TryStreamExt;
use mockito::{Matcher, Server};
use serde_json::{json, Value};
use std::collections::HashMap;
use substrate_gateway::core::GatewayRequest;
use substrate_gateway::models::{
    ContentBlock, CountTokensRequest, ImageDetail, KnownContentBlock, KnownToolResultBlock,
    MessageContent, SystemPrompt, ToolResultBlock, ToolResultContent,
};
use substrate_gateway::providers::anthropic_compatible::AnthropicCompatibleProvider;
use substrate_gateway::providers::gemini::{GeminiAuthConfig, GeminiProvider};
use substrate_gateway::providers::GatewayProvider;

fn request_with_detail(detail: Option<ImageDetail>, stream: bool) -> GatewayRequest {
    let mut request: GatewayRequest = serde_json::from_value(json!({
        "model":"provider-model", "max_output_tokens":32, "stream":stream,
        "messages":[{"role":"user", "content":[
            {"type":"text", "text":"Describe", "cache_control":{"type":"ephemeral"}},
            {"type":"image", "source":{"type":"url", "url":"https://example.com/image.png"}},
            {"type":"image", "source":{"type":"base64", "media_type":"image/png", "data":"aW1hZ2U="}},
            {"type":"tool_result", "tool_use_id":"call_image", "content":[
                {"type":"text", "text":"result"},
                {"type":"image", "source":{"type":"base64", "media_type":"image/jpeg", "data":"anBlZw=="}}
            ], "cache_control":{"type":"ephemeral"}}
        ]}],
        "system":[{"type":"text", "text":"policy"},
            {"type":"image", "source":{"type":"url", "url":"https://example.com/system.png"}}]
    })).unwrap();
    // Assign only through typed state: canonical serde deliberately ignores detail.
    let MessageContent::Blocks(blocks) = &mut request.messages[0].content else {
        panic!("blocks")
    };
    for block in blocks {
        match block {
            ContentBlock::Known(KnownContentBlock::Image { source }) => source.detail = detail,
            ContentBlock::Known(KnownContentBlock::ToolResult {
                content: ToolResultContent::Blocks(results),
                ..
            }) => {
                for result in results {
                    if let ToolResultBlock::Known(KnownToolResultBlock::Image { source }) = result {
                        source.detail = detail;
                    }
                }
            }
            _ => {}
        }
    }
    if let Some(SystemPrompt::Blocks(blocks)) = &mut request.system {
        for block in blocks {
            if let ContentBlock::Known(KnownContentBlock::Image { source }) = block {
                source.detail = detail;
            }
        }
    }
    request
}

// Independent literals: neither expected body is built with a production serializer.
fn anthropic_messages() -> Value {
    json!([{"role":"user", "content":[
        {"type":"text", "text":"Describe", "cache_control":{"type":"ephemeral"}},
        {"type":"image", "source":{"type":"url", "url":"https://example.com/image.png"}},
        {"type":"image", "source":{"type":"base64", "media_type":"image/png", "data":"aW1hZ2U="}},
        {"type":"tool_result", "tool_use_id":"call_image", "content":[
            {"type":"text", "text":"result"},
            {"type":"image", "source":{"type":"base64", "media_type":"image/jpeg", "data":"anBlZw=="}}
        ], "cache_control":{"type":"ephemeral"}}
    ]}])
}

fn anthropic_system() -> Value {
    json!([{"type":"text", "text":"policy"},
        {"type":"image", "source":{"type":"url", "url":"https://example.com/system.png"}}])
}

#[tokio::test]
async fn gwa06_anthropic_sync_stream_and_count_keep_canonical_image_bodies() {
    for detail in [
        Some(ImageDetail::High),
        Some(ImageDetail::Low),
        Some(ImageDetail::Auto),
        None,
    ] {
        for stream in [false, true] {
            let mut server = Server::new_async().await;
            let provider = AnthropicCompatibleProvider::new(
                "anthropic".into(),
                "anthropic-test-key".into(),
                server.url(),
                vec![],
                None,
                None,
            );
            let upstream = if stream {
                "event: message_stop\ndata: {\"type\":\"message_stop\"}\n\n".into()
            } else {
                json!({"id":"message_image", "type":"message", "role":"assistant", "model":"provider-model",
                    "content":[{"type":"text", "text":"ok"}], "stop_reason":"end_turn",
                    "stop_sequence":null, "usage":{"input_tokens":2,"output_tokens":1}}).to_string()
            };
            let mock = server
                .mock("POST", "/v1/messages")
                .match_header("x-api-key", "anthropic-test-key")
                .match_header("anthropic-version", "2023-06-01")
                .match_body(Matcher::Json(
                    json!({"model":"provider-model", "max_output_tokens":32,
                    "stream":stream, "messages":anthropic_messages(), "system":anthropic_system()}),
                ))
                .with_status(200)
                .with_body(&upstream)
                .create_async()
                .await;
            let request = request_with_detail(detail, stream);
            if stream {
                let chunks: Vec<_> = provider
                    .send_message_stream(request)
                    .await
                    .unwrap()
                    .stream
                    .try_collect()
                    .await
                    .unwrap();
                let body: Vec<_> = chunks
                    .into_iter()
                    .flat_map(|chunk| chunk.to_vec())
                    .collect();
                assert_eq!(body, upstream.as_bytes());
            } else {
                let response = provider.send_message(request).await.unwrap();
                assert_eq!(response.content[0].as_text(), Some("ok"));
            }
            mock.assert_async().await;
        }
        let mut server = Server::new_async().await;
        let provider = AnthropicCompatibleProvider::new(
            "anthropic".into(),
            "anthropic-test-key".into(),
            server.url(),
            vec![],
            None,
            None,
        );
        let mock = server
            .mock("POST", "/v1/messages/count_tokens")
            .match_header("x-api-key", "anthropic-test-key")
            .match_body(Matcher::Json(
                json!({"model":"provider-model", "messages":anthropic_messages(),
                "system":anthropic_system()}),
            ))
            .with_status(200)
            .with_body(r#"{"input_tokens":42}"#)
            .create_async()
            .await;
        let request = request_with_detail(detail, false);
        let result = provider
            .count_tokens(CountTokensRequest {
                model: request.model,
                messages: request.messages,
                system: request.system,
                tools: None,
            })
            .await
            .unwrap();
        assert_eq!(result.input_tokens, 42);
        mock.assert_async().await;
    }
}

#[tokio::test]
async fn gwa06_gemini_sync_and_stream_keep_inline_image_projection() {
    for detail in [
        Some(ImageDetail::High),
        Some(ImageDetail::Low),
        Some(ImageDetail::Auto),
        None,
    ] {
        for stream in [false, true] {
            let mut server = Server::new_async().await;
            let provider = GeminiProvider::new(
                "gemini".into(),
                Some(server.url()),
                vec![],
                HashMap::new(),
                GeminiAuthConfig {
                    api_key: Some("gemini-test-key".into()),
                    oauth_provider_id: None,
                    token_store: None,
                    project_id: None,
                    location: None,
                },
            );
            let path = if stream {
                "/models/provider-model:streamGenerateContent"
            } else {
                "/models/provider-model:generateContent"
            };
            let query = if stream {
                "key=gemini-test-key&alt=sse"
            } else {
                "key=gemini-test-key"
            };
            let payload = json!({"candidates":[{"content":{"role":"model", "parts":[{"text":"ok"}]},
                "finishReason":"STOP"}], "usageMetadata":{"promptTokenCount":2,"candidatesTokenCount":1}});
            let upstream = if stream {
                format!("data: {payload}\n\n")
            } else {
                payload.to_string()
            };
            let mock = server
                .mock("POST", path)
                .match_query(Matcher::Exact(query.into()))
                .match_body(Matcher::Json(json!({
                    "contents":[{"role":"user", "parts":[{"text":"Describe"},
                        {"inline_data":{"mimeType":"image/png", "data":"aW1hZ2U="}}]}],
                    "systemInstruction":{"parts":[{"text":"policy"}]},
                    "generationConfig":{"topK":40,"maxOutputTokens":32}
                })))
                .with_status(200)
                .with_body(upstream)
                .create_async()
                .await;
            let request = request_with_detail(detail, stream);
            if stream {
                let chunks: Vec<_> = provider
                    .send_message_stream(request)
                    .await
                    .unwrap()
                    .stream
                    .try_collect()
                    .await
                    .unwrap();
                let body: Vec<_> = chunks
                    .into_iter()
                    .flat_map(|chunk| chunk.to_vec())
                    .collect();
                assert!(String::from_utf8(body).unwrap().contains("ok"));
            } else {
                assert_eq!(
                    provider.send_message(request).await.unwrap().content[0].as_text(),
                    Some("ok")
                );
            }
            mock.assert_async().await;
        }
    }
}
