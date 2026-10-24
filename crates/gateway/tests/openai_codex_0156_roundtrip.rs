use mockito::{Matcher, Server};
use serde_json::{json, Value};
use substrate_gateway::providers::{AuthType, ProviderConfig, ProviderRegistry};
use substrate_gateway::server::openai_conformance_test_support::ConformanceHarness;

fn event(kind: &str, value: Value) -> String {
    format!("event: {kind}\ndata: {value}\n\n")
}

fn harness(upstream: &Server) -> ConformanceHarness {
    harness_url(&upstream.url())
}
fn harness_url(url: &str) -> ConformanceHarness {
    let registry = ProviderRegistry::from_configs_with_models(
        &[ProviderConfig {
            name: "local".into(),
            provider_type: "openai".into(),
            auth_type: AuthType::ApiKey,
            api_key: Some("fixture-secret".into()),
            oauth_provider: None,
            project_id: None,
            location: None,
            base_url: Some(url.into()),
            headers: None,
            models: vec!["codex-fixture".into()],
            enabled: Some(true),
        }],
        None,
        &[],
    )
    .unwrap();
    ConformanceHarness::with_direct_registry("codex-fixture", registry)
}

#[tokio::test]
async fn initial_flat_strict_instructions_cache_request_regression() {
    let mut upstream = Server::new_async().await;
    let output = json!({"type":"function_call","id":"fc_one","call_id":"call_one","name":"run","arguments":" {\"x\": 1} "});
    let stream = event(
        "response.output_item.done",
        json!({"type":"response.output_item.done","output_index":0,"item":output}),
    ) + &event(
        "response.completed",
        json!({"type":"response.completed","response":{"id":"resp_one","status":"completed","output":[],"usage":{"input_tokens":3,"output_tokens":2}}}),
    );
    let mock = upstream.mock("POST", "/responses")
        .match_body(Matcher::PartialJson(json!({"instructions":"caller instructions","tools":[{"type":"function","name":"run","parameters":{"type":"object"},"strict":false}],"prompt_cache_key":"cache-fixture","client_metadata":{"fixture":"one"}})))
        .with_status(200).with_header("content-type","text/event-stream").with_body(stream).create_async().await;
    let gateway = harness(&upstream);
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let app = gateway.router();
    let task = tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
    let response = reqwest::Client::new().post(format!("http://{address}/v1/responses")).json(&json!({
        "model":"codex-fixture","instructions":"caller instructions","input":"hello",
        "tools":[{"type":"function","name":"run","parameters":{"type":"object"},"strict":false}],
        "prompt_cache_key":"cache-fixture","client_metadata":{"fixture":"one"}
    })).send().await.unwrap();
    let status = response.status();
    let body: Value = response.json().await.unwrap();
    task.abort();
    assert_eq!(status.as_u16(), 200, "{body}");
    assert_eq!(body["output"][0]["arguments"], " {\"x\": 1} ");
    assert_eq!(body["output"][0]["call_id"], "call_one");
    mock.assert_async().await;
}

struct RunningGateway {
    _harness: ConformanceHarness,
    url: String,
    task: tokio::task::JoinHandle<()>,
}
impl Drop for RunningGateway {
    fn drop(&mut self) {
        self.task.abort();
    }
}
impl RunningGateway {
    async fn start(upstream: &Server) -> Self {
        Self::from_harness(harness(upstream)).await
    }
    async fn from_harness(gateway: ConformanceHarness) -> Self {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let url = format!("http://{}", listener.local_addr().unwrap());
        let app = gateway.router();
        let task = tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
        Self {
            _harness: gateway,
            url,
            task,
        }
    }
    async fn post(&self, body: Value) -> reqwest::Response {
        reqwest::Client::new()
            .post(format!("{}/v1/responses", self.url))
            .json(&body)
            .send()
            .await
            .unwrap()
    }
}
fn completed_stream(items: &[Value], end_turn: Option<Value>) -> String {
    let mut body = event(
        "response.created",
        json!({"type":"response.created","response":{"id":"upstream-id","status":"in_progress"}}),
    );
    for (i, item) in items.iter().enumerate() {
        body += &event(
            "response.output_item.done",
            json!({"type":"response.output_item.done","output_index":i,"item":item}),
        );
    }
    let mut response = json!({"id":"upstream-id","status":"completed","output":items,"usage":{"input_tokens":7,"output_tokens":3,"input_tokens_details":{"cached_tokens":4}}});
    if let Some(value) = end_turn {
        response["end_turn"] = value;
    }
    body + &event(
        "response.completed",
        json!({"type":"response.completed","response":response}),
    )
}
async fn sync_json(response: reqwest::Response) -> Value {
    let status = response.status();
    let body = response.text().await.unwrap();
    assert_eq!(status.as_u16(), 200, "{body}");
    serde_json::from_str(&body).unwrap()
}

#[tokio::test]
async fn client_discovery_loaded_tools_change_next_request() {
    let mut upstream = Server::new_async().await;
    let call = json!({"type":"tool_search_call","id":"search-item","call_id":"discover","status":"completed","execution":"client","arguments":{"query":"find shell"}});
    let loaded = json!({"type":"function","name":"shell","parameters":{"type":"object"},"strict":false,"defer_loading":true});
    let result = json!({"type":"tool_search_output","id":"loaded-item","call_id":"discover","status":"completed","execution":"client","tools":[loaded.clone()]});
    let invocation = json!({"type":"function_call","id":"invoke-item","call_id":"execute","name":"shell","arguments":" {\"cmd\":\"echo ✅\"} "});
    let first=upstream.mock("POST","/responses").match_body(Matcher::PartialJson(json!({"input":[{"type":"message","role":"user","content":[{"type":"input_text","text":"discover"}]}]}))).with_status(200).with_body(completed_stream(std::slice::from_ref(&call),None)).create_async().await;
    let next = upstream
        .mock("POST", "/responses")
        .match_body(Matcher::PartialJson(
            json!({"input":[call.clone(),result.clone()],"tools":[loaded.clone()]}),
        ))
        .with_status(200)
        .with_body(completed_stream(
            std::slice::from_ref(&invocation),
            Some(json!(false)),
        ))
        .create_async()
        .await;
    let gateway = RunningGateway::start(&upstream).await;
    let one=sync_json(gateway.post(json!({"model":"codex-fixture","input":"discover","tools":[{"type":"tool_search","execution":"client","description":"discover"}]})).await).await;
    assert_eq!(one["output"], json!([call.clone()]));
    let two=sync_json(gateway.post(json!({"model":"codex-fixture","input":[one["output"][0].clone(),result],"tools":[loaded]})).await).await;
    assert_eq!(two["output"], json!([invocation]));
    assert_eq!(two["end_turn"], false);
    first.assert_async().await;
    next.assert_async().await;
}

#[tokio::test]
async fn lite_and_ordinary_instructions_are_not_replaced() {
    let mut upstream = Server::new_async().await;
    let gateway = RunningGateway::start(&upstream).await;
    for instructions in [None, Some(""), Some("native base ✅")] {
        let mut request = json!({"model":"codex-fixture","input":[{"type":"additional_tools","id":"at_stable","role":"developer","tools":[{"type":"custom","name":"exec","format":{"type":"text"}}]},{"type":"message","id":"msg_stable","role":"developer","content":[{"type":"input_text","text":"base separate"}]},{"type":"message","role":"user","content":[{"type":"input_text","text":"go"}]}],"parallel_tool_calls":false,"store":false,"reasoning":{"context":"all_turns"},"prompt_cache_key":"native-cache","client_metadata":{"session_id":"fixture","turn_id":"turn-one"}});
        if let Some(text) = instructions {
            request["instructions"] = text.into();
        }
        let expected_input = request["input"].clone();
        let instruction_expectation = instructions.map(Value::from);
        let mock = upstream
            .mock("POST", "/responses")
            .match_request(move |r| {
                let value: Value = serde_json::from_slice(r.body().unwrap()).unwrap();
                value["input"] == expected_input
                    && value.get("instructions") == instruction_expectation.as_ref()
                    && value.get("tools").is_none()
                    && value["store"] == false
                    && value["parallel_tool_calls"] == false
                    && value["reasoning"] == json!({"context":"all_turns"})
                    && value["prompt_cache_key"] == "native-cache"
                    && value["client_metadata"]["turn_id"] == "turn-one"
            })
            .with_status(200)
            .with_body(completed_stream(&[], None))
            .create_async()
            .await;
        sync_json(gateway.post(request).await).await;
        mock.assert_async().await;
    }
    for bad in [Value::Null, json!({"wrong":"type"})] {
        let response = gateway
            .post(json!({"model":"codex-fixture","input":"go","instructions":bad}))
            .await;
        assert_eq!(response.status().as_u16(), 400);
    }
}

#[tokio::test]
async fn default_opaque_present_and_absent_include_complete_done() {
    let mut upstream = Server::new_async().await;
    let gateway = RunningGateway::start(&upstream).await;
    for include in [
        None,
        Some(json!([])),
        Some(json!(["reasoning.encrypted_content"])),
    ] {
        for effort in [None, Some("none"), Some("high")] {
            for stream in [false, true] {
                let done = json!({"type":"reasoning","id":"rs_fixture","summary":[{"type":"summary_text","text":"UPSTREAM_SUMMARY"}],"encrypted_content":"DONE_OPAQUE_✅"});
                let added = event(
                    "response.output_item.added",
                    json!({"type":"response.output_item.added","output_index":0,"item":{"type":"reasoning","id":"rs_fixture","encrypted_content":"ADDED_NOT_AUTHORITATIVE"}}),
                );
                let mut private = done.clone();
                private["content"] =
                    json!([{"type":"reasoning_text","text":"PRIVATE_REASONING_SENTINEL"}]);
                let response_body = added + &completed_stream(&[private], None);
                let mock = upstream
                    .mock("POST", "/responses")
                    .with_status(200)
                    .with_body(response_body)
                    .expect(1)
                    .create_async()
                    .await;
                let mut request = json!({"model":"codex-fixture","input":"reason","stream":stream});
                if let Some(value) = include.clone() {
                    request["include"] = value;
                }
                if let Some(value) = effort {
                    request["reasoning"] = json!({"effort":value});
                }
                let response = gateway.post(request).await;
                let output = if stream {
                    assert_eq!(response.status().as_u16(), 200);
                    let wire = response.text().await.unwrap();
                    assert!(!wire.contains("PRIVATE_REASONING_SENTINEL"));
                    assert!(!wire.contains("ADDED_NOT_AUTHORITATIVE"));
                    let events = substrate_gateway::providers::streaming::parse_sse_events(&wire);
                    let created: Value = serde_json::from_str(&events[0].data).unwrap();
                    let terminal: Value =
                        serde_json::from_str(&events.last().unwrap().data).unwrap();
                    assert_eq!(terminal["type"], "response.completed");
                    assert_eq!(terminal["response"]["id"], created["response"]["id"]);
                    terminal["response"]["output"].clone()
                } else {
                    sync_json(response).await["output"].clone()
                };
                assert_eq!(output, json!([done.clone()]));
                let item: substrate_gateway::models::response_transport::ResponseItem =
                    serde_json::from_value(done.clone()).unwrap();
                let message = item.into_message();
                let serialized = serde_json::to_value(message.clone()).unwrap();
                let restored: substrate_gateway::models::Message =
                    serde_json::from_value(serialized).unwrap();
                let substrate_gateway::models::MessageContent::Blocks(blocks) = restored.content
                else {
                    panic!("blocks")
                };
                assert_eq!(
                    serde_json::to_value(
                        substrate_gateway::models::response_transport::ResponseItem::from_block(
                            &blocks[0]
                        )
                        .unwrap()
                    )
                    .unwrap(),
                    done
                );
                assert!(!format!("{message:?}").contains("DONE_OPAQUE"));
                mock.assert_async().await;
                let replay = upstream
                    .mock("POST", "/responses")
                    .match_body(Matcher::PartialJson(json!({"input":output})))
                    .with_status(200)
                    .with_body(completed_stream(&[], None))
                    .expect(1)
                    .create_async()
                    .await;
                sync_json(
                    gateway
                        .post(json!({"model":"codex-fixture","input":output}))
                        .await,
                )
                .await;
                replay.assert_async().await;
            }
        }
    }
}

#[tokio::test]
async fn terminal_tristate_and_literal_public_wire() {
    let mut upstream = Server::new_async().await;
    let gateway = RunningGateway::start(&upstream).await;
    for value in [
        None,
        Some(Value::Null),
        Some(json!(false)),
        Some(json!(true)),
    ] {
        for stream in [false, true] {
            let mock = upstream
                .mock("POST", "/responses")
                .with_status(200)
                .with_body(completed_stream(&[], value.clone()))
                .expect(1)
                .create_async()
                .await;
            let response = gateway
                .post(json!({"model":"codex-fixture","input":"terminal","stream":stream}))
                .await;
            let result = if stream {
                let wire = response.text().await.unwrap();
                let events = substrate_gateway::providers::streaming::parse_sse_events(&wire);
                serde_json::from_str::<Value>(&events.last().unwrap().data).unwrap()["response"]
                    .clone()
            } else {
                sync_json(response).await
            };
            assert_eq!(result["status"], "completed");
            match value.as_ref() {
                Some(Value::Bool(b)) => assert_eq!(result.get("end_turn"), Some(&json!(b))),
                _ => assert!(result.get("end_turn").is_none()),
            }
            assert_eq!(result["usage"]["input_tokens"], 7);
            assert_eq!(result["usage"]["input_tokens_details"]["cached_tokens"], 4);
            mock.assert_async().await;
        }
    }
    let mock = upstream
        .mock("POST", "/responses")
        .with_status(200)
        .with_body(completed_stream(&[], Some(json!("wrong"))))
        .create_async()
        .await;
    assert_eq!(
        gateway
            .post(json!({"model":"codex-fixture","input":"terminal"}))
            .await
            .status()
            .as_u16(),
        502
    );
    mock.assert_async().await;
}

#[tokio::test]
async fn closed_media_compaction_and_result_edges() {
    let mut upstream = Server::new_async().await;
    let gateway = RunningGateway::start(&upstream).await;
    let history = json!([
        {"type":"compaction","id":"compact","encrypted_content":"compact opaque"},
        {"type":"context_compaction","id":"context"},
        {"type":"context_compaction","encrypted_content":""},
        {"type":"custom_tool_call","id":"custom","call_id":"call_custom","name":"apply_patch","namespace":"coding","input":"*** Begin Patch\n+✅\n*** End Patch","status":"completed"},
        {"type":"custom_tool_call_output","id":"result","call_id":"call_custom","name":"apply_patch","output":[{"type":"input_text","text":"ok"},{"type":"input_image","image_url":"data:image/png;base64,YQ==","detail":"original"},{"type":"encrypted_content","encrypted_content":"result opaque"}]}
    ]);
    let expected = history.clone();
    let mock = upstream
        .mock("POST", "/responses")
        .match_body(Matcher::PartialJson(json!({"input":expected})))
        .with_status(200)
        .with_body(completed_stream(&[], None))
        .create_async()
        .await;
    sync_json(
        gateway
            .post(json!({"model":"codex-fixture","input":history}))
            .await,
    )
    .await;
    mock.assert_async().await;
    for item in [
        json!({"type":"message","role":"user","content":[{"type":"input_image","file_id":"file-private"}]}),
        json!({"type":"message","role":"user","content":[{"type":"input_audio","data":"audio"}]}),
        json!({"type":"context_compaction","encrypted_content":null}),
        json!({"type":"compaction_summary","encrypted_content":"opaque"}),
        json!({"type":"configuration_update","reasoning":{"effort":"high"}}),
        json!({"type":"reasoning","summary":[],"content":[{"type":"reasoning_text","text":"private"}]}),
        json!({"type":"function_call_output","call_id":"orphan","output":"no anchor"}),
        json!({"type":"custom_tool_call_output","call_id":"orphan","output":"no anchor"}),
        json!({"type":"tool_search_output","call_id":"orphan","execution":"client","status":"completed","tools":[]}),
    ] {
        let response = gateway
            .post(json!({"model":"codex-fixture","input":[item]}))
            .await;
        assert_eq!(response.status().as_u16(), 400);
    }
}

#[tokio::test]
async fn opaque_context_never_falls_back_across_destination() {
    let mut upstream = Server::new_async().await;
    let mut redirect = Server::new_async().await;
    let second = redirect
        .mock("POST", "/second")
        .expect(0)
        .with_status(200)
        .create_async()
        .await;
    let first = upstream
        .mock("POST", "/responses")
        .with_status(307)
        .with_header("location", &format!("{}/second", redirect.url()))
        .create_async()
        .await;
    let gateway = RunningGateway::start(&upstream).await;
    let response=gateway.post(json!({"model":"codex-fixture","input":[{"type":"reasoning","summary":[],"encrypted_content":"same-caller fixture"}]})).await;
    assert_eq!(response.status().as_u16(), 502);
    first.assert_async().await;
    second.assert_async().await;
}

#[tokio::test]
async fn real_adapter_errors_eof_cancel_and_replay_are_truthful() {
    let mut upstream = Server::new_async().await;
    let gateway = RunningGateway::start(&upstream).await;
    for stream in [false, true] {
        for wire in [
            event(
                "response.failed",
                json!({"type":"response.failed","error":{"message":"UPSTREAM_ERROR_SENTINEL"}}),
            ),
            event("response.incomplete", json!({"type":"response.incomplete"})),
            event(
                "response.output_item.added",
                json!({"type":"response.output_item.added","output_index":0,"item":{"type":"reasoning","encrypted_content":"PROVISIONAL_PRIVATE"}}),
            ),
            "data: {malformed_PRIVATE_SENTINEL}\n\n".into(),
        ] {
            let mock = upstream
                .mock("POST", "/responses")
                .with_status(200)
                .with_body(wire)
                .create_async()
                .await;
            let response = gateway
                .post(json!({"model":"codex-fixture","input":"error","stream":stream}))
                .await;
            if stream {
                let body = response.text().await;
                if let Ok(body) = body {
                    assert!(!body.contains("response.completed"));
                    assert!(!body.contains("PRIVATE_SENTINEL"));
                }
            } else {
                assert_eq!(response.status().as_u16(), 502);
                let body = response.text().await.unwrap();
                assert!(!body.contains("SENTINEL"));
            }
            mock.assert_async().await;
        }
    }
}

#[tokio::test]
async fn flat_strict_custom_namespace_complete_loop() {
    let mut upstream = Server::new_async().await;
    let gateway = RunningGateway::start(&upstream).await;
    let definitions = json!([
        {"type":"function","name":"ordinary","parameters":{"type":"object"},"strict":false},
        {"type":"custom","name":"raw","format":{"type":"grammar","syntax":"lark","definition":"start: /.+/"}},
        {"type":"namespace","name":"one","description":"first","tools":[{"type":"function","name":"same","strict":false}]},
        {"type":"namespace","name":"two","description":"second","tools":[{"type":"custom","name":"same","format":{"type":"text"}}]}
    ]);
    let calls = vec![
        json!({"type":"function_call","id":"fn_item","call_id":"fn","name":"same","namespace":"one","arguments":" {\"word\": \"✅\"} "}),
        json!({"type":"custom_tool_call","id":"custom_item","call_id":"custom","name":"same","namespace":"two","status":"completed","input":"raw ✅\n not JSON"}),
    ];
    let first=upstream.mock("POST","/responses").match_body(Matcher::PartialJson(json!({"input":[{"type":"message","role":"user","content":[{"type":"input_text","text":"first turn"}]}],"tools":definitions.clone()}))).with_status(200).with_body(completed_stream(&calls,Some(json!(false)))).create_async().await;
    let one = sync_json(
        gateway
            .post(json!({"model":"codex-fixture","input":"first turn","tools":definitions.clone()}))
            .await,
    )
    .await;
    assert_eq!(one["output"], json!(calls));
    let mut history = one["output"].as_array().unwrap().clone();
    history.push(json!({"type":"function_call_output","id":"fn_result","call_id":"fn","name":"same","namespace":"one","output":"literal result"}));
    history.push(json!({"type":"custom_tool_call_output","id":"custom_result","call_id":"custom","name":"same","output":[{"type":"input_text","text":"structured"}]}));
    // Reload the serialized core history: bytes/identities survive without gateway persistence.
    let items: Vec<substrate_gateway::models::response_transport::ResponseItem> =
        serde_json::from_value(json!(history)).unwrap();
    let messages: Vec<_> = items.into_iter().map(|i| i.into_message()).collect();
    let reloaded: Vec<substrate_gateway::models::Message> =
        serde_json::from_slice(&serde_json::to_vec(&messages.clone()).unwrap()).unwrap();
    let reconstructed: Vec<_> = reloaded
        .iter()
        .flat_map(|m| match &m.content {
            substrate_gateway::models::MessageContent::Blocks(v) => v
                .iter()
                .filter_map(substrate_gateway::models::response_transport::ResponseItem::from_block)
                .collect::<Vec<_>>(),
            _ => vec![],
        })
        .collect();
    let reconstructed = serde_json::to_value(reconstructed).unwrap();
    assert_eq!(reconstructed, json!(history));
    let second = upstream
        .mock("POST", "/responses")
        .match_body(Matcher::PartialJson(
            json!({"input":history.clone(),"tools":definitions.clone()}),
        ))
        .with_status(200)
        .with_body(completed_stream(&[], Some(json!(true))))
        .create_async()
        .await;
    sync_json(
        gateway
            .post(json!({"model":"codex-fixture","input":reconstructed,"tools":definitions}))
            .await,
    )
    .await;
    first.assert_async().await;
    second.assert_async().await;
    // Same identity with changed kind/name/namespace cannot acquire fabricated provenance.
    for invalid in [
        json!([{ "type":"function_call","call_id":"id","name":"same","namespace":"one","arguments":"{}"},{"type":"custom_tool_call_output","call_id":"id","output":"wrong kind"}]),
        json!([{ "type":"function_call","call_id":"id","name":"same","namespace":"one","arguments":"{}"},{"type":"function_call_output","call_id":"id","namespace":"two","output":"wrong namespace"}]),
        json!([{ "type":"function_call","call_id":"id","name":"same","arguments":"{}"},{"type":"function_call","call_id":"id","name":"changed","arguments":"{}"}]),
    ] {
        assert_eq!(
            gateway
                .post(json!({"model":"codex-fixture","input":invalid}))
                .await
                .status()
                .as_u16(),
            400
        );
    }
}

#[tokio::test]
async fn request_frame_state_wire_and_copy_budgets_fail_without_loss() {
    use substrate_gateway::models::response_transport::{PAYLOAD_LIMIT as B, WIRE_LIMIT};
    let mut upstream = Server::new_async().await;
    let gateway = RunningGateway::start(&upstream).await;
    let empty = completed_stream(&[], None);
    let accepted = upstream
        .mock("POST", "/responses")
        .with_status(200)
        .with_body(empty.clone())
        .expect(2)
        .create_async()
        .await;
    for size in [B - 1, B, B + 1] {
        let base =
            json!({"model":"codex-fixture","input":"entity","unused_padding":""}).to_string();
        let mut value: Value = serde_json::from_str(&base).unwrap();
        value["unused_padding"] = "p".repeat(size - base.len()).into();
        let raw = value.to_string();
        assert_eq!(raw.len(), size);
        let response = reqwest::Client::new()
            .post(format!("{}/v1/responses", gateway.url))
            .body(raw)
            .header("content-type", "application/json")
            .send()
            .await
            .unwrap();
        assert_eq!(response.status().as_u16(), if size > B { 400 } else { 200 });
    }
    accepted.assert_async().await;
    for size in [B - 1, B, B + 1] {
        let terminal = event(
            "response.completed",
            json!({"type":"response.completed","response":{"status":"completed","output":[]}}),
        );
        let wire = format!(":{}\n{}", "p".repeat(size - terminal.len() - 2), terminal);
        assert_eq!(wire.len(), size);
        let mock = upstream
            .mock("POST", "/responses")
            .with_status(200)
            .with_body(wire)
            .create_async()
            .await;
        assert_eq!(
            gateway
                .post(json!({"model":"codex-fixture","input":"frame"}))
                .await
                .status()
                .as_u16(),
            if size > B { 502 } else { 200 }
        );
        mock.assert_async().await;
    }
    for total in [WIRE_LIMIT - 1, WIRE_LIMIT, WIRE_LIMIT + 1] {
        let terminal = event(
            "response.completed",
            json!({"type":"response.completed","response":{"status":"completed","output":[]}}),
        );
        let mut remaining = total - terminal.len();
        let mut wire = String::new();
        while remaining > 0 {
            let size = remaining.min(B);
            if size < 3 {
                wire.push_str(&" ".repeat(size));
                break;
            }
            wire.push(':');
            wire.push_str(&"p".repeat(size - 3));
            wire.push_str("\n\n");
            remaining -= size;
        }
        wire += &terminal;
        assert_eq!(wire.len(), total);
        let mock = upstream
            .mock("POST", "/responses")
            .with_status(200)
            .with_body(wire)
            .create_async()
            .await;
        assert_eq!(
            gateway
                .post(json!({"model":"codex-fixture","input":"wire"}))
                .await
                .status()
                .as_u16(),
            if total > WIRE_LIMIT { 502 } else { 200 }
        );
        mock.assert_async().await;
    }
    for n in [B / 2 - 300, B / 2] {
        let items = vec![
            json!({"type":"reasoning","id":"one","summary":[],"encrypted_content":"a".repeat(n)}),
            json!({"type":"reasoning","id":"two","summary":[],"encrypted_content":"b".repeat(n)}),
        ];
        let mock = upstream
            .mock("POST", "/responses")
            .with_status(200)
            .with_body(completed_stream(&items, None))
            .create_async()
            .await;
        assert_eq!(
            gateway
                .post(json!({"model":"codex-fixture","input":"state"}))
                .await
                .status()
                .as_u16(),
            if n == B / 2 { 502 } else { 200 }
        );
        mock.assert_async().await;
    }
}

#[tokio::test]
async fn opaque_private_summary_and_error_sentinels_have_safe_sinks() {
    use substrate_gateway::cli::TracingConfig;
    use substrate_gateway::core::{GatewayRequest, GatewayResponse, GatewayUsage};
    use substrate_gateway::message_tracing::MessageTracer;
    use substrate_gateway::models::response_transport::{Opaque, ResponseItem};
    use substrate_gateway::models::RouteType;
    use substrate_gateway::structured_events::NormalizedEvent;
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("trace.jsonl");
    let tracer = MessageTracer::new(TracingConfig {
        enabled: true,
        path: path.display().to_string(),
        omit_system_prompt: false,
    });
    let item = ResponseItem::Reasoning {
        id: Some("rs_sink".into()),
        summary: vec![serde_json::from_value(
            json!({"type":"summary_text","text":"SUMMARY_TRACE_SENTINEL"}),
        )
        .unwrap()],
        encrypted_content: Some(Opaque("OPAQUE_TRACE_SENTINEL".into())),
    };
    let message = item.into_message();
    let request: GatewayRequest = serde_json::from_value(
        json!({"model":"fixture","messages":[message.clone()],"max_output_tokens":1}),
    )
    .unwrap();
    let response = GatewayResponse {
        id: "fixture".into(),
        r#type: "message".into(),
        role: "assistant".into(),
        content: match message.content {
            substrate_gateway::models::MessageContent::Blocks(b) => b,
            _ => unreachable!(),
        },
        model: "fixture".into(),
        stop_reason: Some("end_turn".into()),
        end_turn: Some(false),
        stop_sequence: None,
        usage: GatewayUsage {
            input_tokens: 1,
            output_tokens: 1,
            cache_creation_input_tokens: None,
            cache_read_input_tokens: Some(1),
        },
    };
    tracer.trace_request("trace", &request, "fixture", &RouteType::Default, false);
    tracer.trace_response("trace", &response, 1);
    tracer.trace_error("trace", "ERROR_TRACE_SENTINEL");
    tracer.trace_event(
        "trace",
        &NormalizedEvent::tool_intent(
            "call".into(),
            "tool".into(),
            Some(json!({"arg":"ARG_TRACE_SENTINEL"})),
        ),
    );
    let trace = std::fs::read_to_string(path).unwrap();
    assert_eq!(trace.lines().count(), 4);
    for sentinel in [
        "OPAQUE_TRACE_SENTINEL",
        "SUMMARY_TRACE_SENTINEL",
        "ERROR_TRACE_SENTINEL",
        "ARG_TRACE_SENTINEL",
    ] {
        assert!(!trace.contains(sentinel));
        assert!(!format!("{request:?}{response:?}").contains(sentinel));
    }
    assert!(serde_json::to_string(&response)
        .unwrap()
        .contains("OPAQUE_TRACE_SENTINEL"));
    assert!(
        substrate_gateway::structured_events::normalized_events_from_provider_response(&response)
            .iter()
            .all(|e| e.final_text.is_none())
    );
    let mut upstream = Server::new_async().await;
    let gateway = RunningGateway::start(&upstream).await;
    // Extractor failures happen before the handler and cannot interpolate a payload.
    let malformed = reqwest::Client::new()
        .post(format!("{}/v1/responses", gateway.url))
        .body("{\"instructions\":\"MALFORMED_TRACE_SENTINEL")
        .send()
        .await
        .unwrap();
    assert_eq!(malformed.status().as_u16(), 400);
    assert!(!malformed
        .text()
        .await
        .unwrap()
        .contains("MALFORMED_TRACE_SENTINEL"));
    let error = upstream
        .mock("POST", "/responses")
        .with_status(502)
        .with_body("UPSTREAM_BODY_SENTINEL")
        .create_async()
        .await;
    let result = gateway
        .post(json!({"model":"codex-fixture","input":"error"}))
        .await;
    assert_eq!(result.status().as_u16(), 502);
    assert!(!result
        .text()
        .await
        .unwrap()
        .contains("UPSTREAM_BODY_SENTINEL"));
    error.assert_async().await;
}

fn configured_harness(
    primary: &Server,
    secondary: &Server,
    primary_kind: &str,
) -> ConformanceHarness {
    use substrate_gateway::cli::ModelMapping;
    let configs = [
        ("first", primary, primary_kind, "account-one", "actual-one"),
        ("second", secondary, "openai", "account-two", "actual-two"),
    ]
    .into_iter()
    .map(|(name, server, kind, key, model)| ProviderConfig {
        name: name.into(),
        provider_type: kind.into(),
        auth_type: AuthType::ApiKey,
        api_key: Some(key.into()),
        oauth_provider: None,
        project_id: None,
        location: None,
        base_url: Some(server.url()),
        headers: None,
        models: vec![model.into()],
        enabled: Some(true),
    })
    .collect::<Vec<_>>();
    let registry = ProviderRegistry::from_configs_with_models(&configs, None, &[]).unwrap();
    ConformanceHarness::with_registry(
        "visible",
        registry,
        vec![
            ModelMapping {
                priority: 1,
                provider: "first".into(),
                actual_model: "actual-one".into(),
            },
            ModelMapping {
                priority: 2,
                provider: "second".into(),
                actual_model: "actual-two".into(),
            },
        ],
    )
}

#[tokio::test]
async fn configured_backend_account_and_model_remain_pinned_on_failure() {
    for kind in ["openai", "anthropic"] {
        let mut first = Server::new_async().await;
        let mut second = Server::new_async().await;
        let first_call=first.mock("POST","/responses").match_header("authorization","Bearer account-one").match_body(Matcher::PartialJson(json!({"model":"actual-one","input":[{"type":"reasoning","summary":[],"encrypted_content":"pinned opaque"}]}))).with_status(503).expect(if kind=="openai"{1}else{0}).create_async().await;
        let second_call = second
            .mock("POST", "/responses")
            .expect(0)
            .create_async()
            .await;
        let messages = first
            .mock("POST", "/v1/messages")
            .expect(0)
            .create_async()
            .await;
        let gateway = RunningGateway::from_harness(configured_harness(&first, &second, kind)).await;
        let response=gateway.post(json!({"model":"visible","input":[{"type":"reasoning","summary":[],"encrypted_content":"pinned opaque"}]})).await;
        assert_eq!(
            response.status().as_u16(),
            if kind == "openai" { 502 } else { 400 }
        );
        first_call.assert_async().await;
        second_call.assert_async().await;
        messages.assert_async().await;
    }
}

struct HeldUpstream {
    url: String,
    task: tokio::task::JoinHandle<()>,
    closed: Option<tokio::sync::oneshot::Receiver<()>>,
}
impl Drop for HeldUpstream {
    fn drop(&mut self) {
        self.task.abort();
    }
}
impl HeldUpstream {
    async fn start(frame: String) -> Self {
        use tokio::io::{AsyncReadExt, AsyncWriteExt};
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let url = format!("http://{}", listener.local_addr().unwrap());
        let (send, closed) = tokio::sync::oneshot::channel();
        let task = tokio::spawn(async move {
            let (mut socket, _) = listener.accept().await.unwrap();
            let mut request = Vec::new();
            let mut buffer = [0u8; 4096];
            loop {
                let n = socket.read(&mut buffer).await.unwrap();
                assert!(n > 0);
                request.extend_from_slice(&buffer[..n]);
                if let Some(end) = request.windows(4).position(|w| w == b"\r\n\r\n") {
                    let header = String::from_utf8_lossy(&request[..end]);
                    let length = header
                        .lines()
                        .find_map(|line| {
                            line.to_ascii_lowercase()
                                .strip_prefix("content-length:")
                                .map(|n| n.trim().parse::<usize>().unwrap())
                        })
                        .unwrap();
                    if request.len() >= end + 4 + length {
                        break;
                    }
                }
            }
            socket.write_all(b"HTTP/1.1 200 OK\r\nContent-Type: text/event-stream\r\nTransfer-Encoding: chunked\r\nConnection: close\r\n\r\n").await.unwrap();
            socket
                .write_all(format!("{:x}\r\n{frame}\r\n", frame.len()).as_bytes())
                .await
                .unwrap();
            let n = socket.read(&mut buffer).await.unwrap_or(0);
            assert_eq!(n, 0, "gateway retained upstream after terminal/cancel");
            let _ = send.send(());
        });
        Self {
            url,
            task,
            closed: Some(closed),
        }
    }
    async fn assert_closed(&mut self) {
        tokio::time::timeout(
            std::time::Duration::from_secs(3),
            self.closed.take().unwrap(),
        )
        .await
        .expect("upstream connection was not released")
        .unwrap();
    }
}

#[tokio::test]
async fn actual_handler_cancellation_and_completed_terminal_release_upstream() {
    use futures::StreamExt;
    for terminal in [false, true] {
        let frame = if terminal {
            completed_stream(&[], None)
        } else {
            event(
                "response.output_text.delta",
                json!({"type":"response.output_text.delta","output_index":0,"content_index":0,"delta":"visible partial"}),
            )
        };
        let mut upstream = HeldUpstream::start(frame).await;
        let gateway = RunningGateway::from_harness(harness_url(&upstream.url)).await;
        let response = gateway
            .post(json!({"model":"codex-fixture","input":"cleanup","stream":!terminal}))
            .await;
        assert_eq!(response.status().as_u16(), 200);
        if terminal {
            sync_json(response).await;
        } else {
            let mut body = response.bytes_stream();
            let first = tokio::time::timeout(std::time::Duration::from_secs(3), body.next())
                .await
                .unwrap()
                .unwrap()
                .unwrap();
            assert!(!String::from_utf8_lossy(&first).contains("response.completed"));
            drop(body);
        }
        upstream.assert_closed().await;
    }
}

#[tokio::test]
async fn hosted_search_summary_and_cache_usage_are_exact() {
    let mut upstream = Server::new_async().await;
    let gateway = RunningGateway::start(&upstream).await;
    let tool = json!({"type":"web_search","external_web_access":true,"indexed_web_access":false,"filters":{"allowed_domains":["example.com"]},"user_location":{"type":"approximate","country":"US"},"search_context_size":"low","search_content_types":["text"]});
    let activity = json!({"type":"web_search_call","id":"hosted","status":"completed","action":{"type":"search","query":"one","queries":["two"]}});
    for effort in [None, Some("none")] {
        let mut reasoning = json!({"summary":"concise"});
        if let Some(effort) = effort {
            reasoning["effort"] = effort.into();
        }
        let expected = reasoning.clone();
        let expected_tool = tool.clone();
        let mock = upstream
            .mock("POST", "/responses")
            .match_request(move |r| {
                let wire: Value = serde_json::from_slice(r.body().unwrap()).unwrap();
                wire["tools"] == json!([expected_tool]) && wire["reasoning"] == expected
            })
            .with_status(200)
            .with_body(completed_stream(std::slice::from_ref(&activity), None))
            .create_async()
            .await;
        let response=sync_json(gateway.post(json!({"model":"codex-fixture","input":"search","tools":[tool.clone()],"reasoning":reasoning})).await).await;
        assert_eq!(response["output"], json!([activity.clone()]));
        assert_eq!(
            response["usage"]["input_tokens_details"]["cached_tokens"],
            4
        );
        mock.assert_async().await;
    }
}

#[tokio::test]
async fn copy_admission_has_exact_boundaries_and_checked_overflow() {
    use substrate_gateway::models::response_transport::{
        bounded_add, NORMALIZED_LIMIT, PAYLOAD_LIMIT as B, QUEUE_LIMIT, WORKING_LIMIT,
    };
    for limit in [B, 2 * B, NORMALIZED_LIMIT, QUEUE_LIMIT, WORKING_LIMIT] {
        assert_eq!(bounded_add(limit - 1, 0, limit).unwrap(), limit - 1);
        assert_eq!(bounded_add(limit - 1, 1, limit).unwrap(), limit);
        assert!(bounded_add(limit, 1, limit).is_err());
        assert!(bounded_add(usize::MAX, 1, limit).is_err());
    }
    // Simultaneous public-parse phase reservations; the trace parse phase is smaller.
    assert_eq!(
        8 * B + 2 * B + 8 * B + 2 * B + 4 * B + 4 * B + 4 * B,
        WORKING_LIMIT
    );
}

#[derive(Clone)]
struct TraceWriter(std::sync::Arc<std::sync::Mutex<Vec<u8>>>);
impl std::io::Write for TraceWriter {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        self.0.lock().unwrap().extend_from_slice(bytes);
        Ok(bytes.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}
fn captured_logs() -> std::sync::Arc<std::sync::Mutex<Vec<u8>>> {
    static LOGS: std::sync::OnceLock<std::sync::Arc<std::sync::Mutex<Vec<u8>>>> =
        std::sync::OnceLock::new();
    LOGS.get_or_init(|| {
        let bytes = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
        let writer = TraceWriter(bytes.clone());
        let subscriber = tracing_subscriber::fmt()
            .with_ansi(false)
            .without_time()
            .with_max_level(tracing::Level::TRACE)
            .with_writer(move || writer.clone())
            .finish();
        tracing::subscriber::set_global_default(subscriber).unwrap();
        bytes
    })
    .clone()
}
fn mapped_harness(upstream: &Server, kind: &str) -> ConformanceHarness {
    use substrate_gateway::cli::ModelMapping;
    let registry = ProviderRegistry::from_configs_with_models(
        &[ProviderConfig {
            name: "local".into(),
            provider_type: kind.into(),
            auth_type: AuthType::ApiKey,
            api_key: Some("fixture-secret".into()),
            oauth_provider: None,
            project_id: None,
            location: None,
            base_url: Some(upstream.url()),
            headers: None,
            models: vec!["codex-fixture".into()],
            enabled: Some(true),
        }],
        None,
        &[],
    )
    .unwrap();
    ConformanceHarness::with_registry(
        "visible",
        registry,
        vec![ModelMapping {
            priority: 1,
            provider: "local".into(),
            actual_model: "codex-fixture".into(),
        }],
    )
}

#[tokio::test]
async fn default_opaque_and_internal_terminal_stay_off_messages_chat_and_logs() {
    let logs = captured_logs();
    let mut upstream = Server::new_async().await;
    let reasoning = json!({"type":"reasoning","id":"private","summary":[{"type":"summary_text","text":"LEGACY_SUMMARY_SENTINEL"}],"encrypted_content":"LEGACY_OPAQUE_SENTINEL","content":[{"type":"reasoning_text","text":"LEGACY_PLAINTEXT_SENTINEL"}]});
    let visible = json!({"type":"message","id":"visible","role":"assistant","content":[{"type":"output_text","text":"public text"}]});
    let mock = upstream
        .mock("POST", "/responses")
        .with_status(200)
        .with_body(completed_stream(&[reasoning, visible], Some(json!(false))))
        .expect(4)
        .create_async()
        .await;
    let gateway = RunningGateway::from_harness(mapped_harness(&upstream, "openai")).await;
    for endpoint in ["messages", "chat/completions"] {
        for stream in [false, true] {
            let response=reqwest::Client::new().post(format!("{}/v1/{endpoint}",gateway.url)).json(&json!({"model":"visible","max_tokens":100,"messages":[{"role":"user","content":"hello"}],"stream":stream,"metadata":{"openai_public_responses":true,"responses_caller":true},"responses_caller":true})).send().await.unwrap();
            let status = response.status();
            let body = response.text().await.unwrap();
            assert_eq!(status.as_u16(), 200, "{body}");
            assert!(body.contains("public text"));
            for secret in [
                "LEGACY_",
                "gateway.response_",
                "end_turn\":false",
                "encrypted_content",
            ] {
                assert!(!body.contains(secret), "{endpoint}: {body}");
            }
        }
    }
    mock.assert_async().await;
    for block in [
        json!({"type":"gateway_response_item","item":{"type":"reasoning","summary":[],"encrypted_content":"MESSAGES_SMUGGLE_SENTINEL"}}),
        json!({"type":"tool_use","id":"x","name":"f","input":{},"transport":{"kind":"function","encrypted_content":"MESSAGES_SMUGGLE_SENTINEL"}}),
        json!({"type":"tool_use","id":"x","name":"f","input":{},"transport":"MESSAGES_SMUGGLE_SENTINEL"}),
        json!({"type":"tool_result","tool_use_id":"x","content":[{"type":"encrypted_content","encrypted_content":"MESSAGES_SMUGGLE_SENTINEL"}]}),
    ] {
        let response=reqwest::Client::new().post(format!("{}/v1/messages",gateway.url)).json(&json!({"model":"visible","max_tokens":100,"messages":[{"role":"user","content":[block]}]})).send().await.unwrap();
        assert_eq!(response.status().as_u16(), 400);
        assert!(!response
            .text()
            .await
            .unwrap()
            .contains("MESSAGES_SMUGGLE_SENTINEL"));
    }
    let log = String::from_utf8(logs.lock().unwrap().clone()).unwrap();
    for secret in [
        "LEGACY_OPAQUE_SENTINEL",
        "LEGACY_PLAINTEXT_SENTINEL",
        "LEGACY_SUMMARY_SENTINEL",
        "MESSAGES_SMUGGLE_SENTINEL",
    ] {
        assert!(
            !log.contains(secret),
            "private transport reached tracing output"
        );
    }
}

#[tokio::test]
async fn anthropic_wire_retains_owned_tool_types_cache_and_signed_thinking() {
    let mut upstream = Server::new_async().await;
    let gateway = RunningGateway::from_harness(mapped_harness(&upstream, "anthropic")).await;
    let tools = json!([{"name":"plain","description":"original","input_schema":{"type":"object"}},{"type":"web_search_20250305","name":"web_search"},{"type":"custom","name":"typed","input_schema":{"type":"object"}}]);
    let messages = json!([{"role":"assistant","content":[{"type":"thinking","thinking":"signed text","signature":"actual-provider-signature"},{"type":"text","text":"action","cache_control":{"type":"ephemeral"}}]},{"role":"user","content":[{"type":"tool_result","tool_use_id":"old","content":[],"is_error":true,"cache_control":{"type":"ephemeral"}}]}]);
    let expected_tools = tools.clone();
    let expected_messages = messages.clone();
    let mock=upstream.mock("POST","/v1/messages").match_request(move|r|{let wire:Value=serde_json::from_slice(r.body().unwrap()).unwrap();wire["tools"]==expected_tools && wire["messages"]==expected_messages && wire.get("instructions").is_none()}).with_status(200).with_body(json!({"id":"anthropic","type":"message","role":"assistant","content":[{"type":"text","text":"ok"}],"model":"codex-fixture","stop_reason":"end_turn","stop_sequence":null,"usage":{"input_tokens":1,"output_tokens":2,"cache_read_input_tokens":1}}).to_string()).create_async().await;
    let response = reqwest::Client::new()
        .post(format!("{}/v1/messages", gateway.url))
        .json(&json!({"model":"visible","max_tokens":100,"tools":tools,"messages":messages}))
        .send()
        .await
        .unwrap();
    let body = sync_json(response).await;
    assert_eq!(body["usage"]["cache_read_input_tokens"], 1);
    mock.assert_async().await;
}

#[tokio::test]
async fn inert_discovery_and_native_result_core_reload_keep_exact_shapes() {
    use substrate_gateway::models::response_transport::ResponseItem;
    use substrate_gateway::models::{Message, MessageContent};
    let mut upstream = Server::new_async().await;
    let gateway = RunningGateway::start(&upstream).await;
    let inert = json!({"type":"tool_search_call","id":"inert","execution":"client","arguments":{"query":"inert"},"status":"completed"});
    let mock = upstream
        .mock("POST", "/responses")
        .with_status(200)
        .with_body(completed_stream(
            std::slice::from_ref(&inert),
            Some(json!(true)),
        ))
        .create_async()
        .await;
    let response = sync_json(
        gateway
            .post(json!({"model":"codex-fixture","input":"inert"}))
            .await,
    )
    .await;
    assert_eq!(response["output"], json!([inert]));
    mock.assert_async().await;
    for output in [
        json!([]),
        json!([{"type":"input_text","text":"text"}]),
        json!([{"type":"encrypted_content","encrypted_content":"empty-result-opaque"}]),
    ] {
        let item = json!({"type":"function_call_output","call_id":"call","output":output});
        let normalized = serde_json::from_value::<ResponseItem>(item.clone())
            .unwrap()
            .into_message();
        let restored: Message =
            serde_json::from_value(serde_json::to_value(normalized).unwrap()).unwrap();
        let MessageContent::Blocks(blocks) = restored.content else {
            panic!("blocks")
        };
        assert_eq!(
            serde_json::to_value(ResponseItem::from_block(&blocks[0]).unwrap()).unwrap(),
            item
        );
    }
    for history in [
        json!([{ "type":"tool_search_output","call_id":"","execution":"client","status":"completed","tools":[]}]),
        json!([{ "type":"reasoning","summary":[],"encrypted_content":"opaque"},{"type":"function_call_output","call_id":"orphan","output":"missing"}]),
    ] {
        assert_eq!(
            gateway
                .post(
                    json!({"model":"codex-fixture","input":history,"previous_response_id":"stored"})
                )
                .await
                .status()
                .as_u16(),
            400
        );
    }
}

#[tokio::test]
async fn closed_output_media_citations_compaction_and_usage_fail_safely() {
    let mut upstream = Server::new_async().await;
    let gateway = RunningGateway::start(&upstream).await;
    for item in [
        json!({"type":"message","role":"assistant","content":[{"type":"output_text","text":"citation","annotations":[{"type":"url_citation","url":"https://example.com"}]}]}),
        json!({"type":"message","role":"assistant","content":[{"type":"output_audio","data":"private"}]}),
        json!({"type":"compaction","encrypted_content":"history-only"}),
        json!({"type":"configuration_update","private":"unsupported"}),
    ] {
        let mock = upstream
            .mock("POST", "/responses")
            .with_status(200)
            .with_body(completed_stream(&[item], None))
            .create_async()
            .await;
        let response = gateway
            .post(json!({"model":"codex-fixture","input":"closed output"}))
            .await;
        assert_eq!(response.status().as_u16(), 502);
        mock.assert_async().await;
    }
    for bad in [json!(4294967296u64), json!(-1), json!("wrong")] {
        for stream in [false, true] {
            let frame = event(
                "response.completed",
                json!({"type":"response.completed","response":{"status":"completed","output":[],"usage":{"input_tokens":bad,"output_tokens":1}}}),
            );
            let mock = upstream
                .mock("POST", "/responses")
                .with_status(200)
                .with_body(frame)
                .create_async()
                .await;
            assert_eq!(
                gateway
                    .post(json!({"model":"codex-fixture","input":"invalid usage","stream":stream}))
                    .await
                    .status()
                    .as_u16(),
                502
            );
            mock.assert_async().await;
        }
    }
}

#[tokio::test]
async fn responses_provider_terminal_drops_network_before_consumer_polls() {
    let mut upstream = HeldUpstream::start(completed_stream(&[], None)).await;
    let registry = ProviderRegistry::from_configs_with_models(
        &[ProviderConfig {
            name: "held".into(),
            provider_type: "openai".into(),
            auth_type: AuthType::ApiKey,
            api_key: Some("fixture".into()),
            oauth_provider: None,
            project_id: None,
            location: None,
            base_url: Some(upstream.url.clone()),
            headers: None,
            models: vec!["codex-fixture".into()],
            enabled: Some(true),
        }],
        None,
        &[],
    )
    .unwrap();
    let provider = registry.get_provider_for_model("codex-fixture").unwrap();
    let request=serde_json::from_value(json!({"model":"codex-fixture","max_output_tokens":1,"messages":[{"role":"user","content":"terminal"}]})).unwrap();
    let stream = provider.send_message_stream(request).await.unwrap();
    upstream.assert_closed().await;
    drop(stream);
}

#[tokio::test]
async fn image_detail_survives_core_history_reload_and_actual_responses_wire() {
    use substrate_gateway::core::GatewayRequest;
    use substrate_gateway::models::response_transport::ResponseItem;
    let mut upstream = Server::new_async().await;
    let gateway = RunningGateway::start(&upstream).await;
    let registry = ProviderRegistry::from_configs_with_models(
        &[ProviderConfig {
            name: "image-history".into(),
            provider_type: "openai".into(),
            auth_type: AuthType::ApiKey,
            api_key: Some("fixture".into()),
            oauth_provider: None,
            project_id: None,
            location: None,
            base_url: Some(upstream.url()),
            headers: None,
            models: vec!["codex-fixture".into()],
            enabled: Some(true),
        }],
        None,
        &[],
    )
    .unwrap();
    for detail in [
        None,
        Some("auto"),
        Some("low"),
        Some("high"),
        Some("original"),
    ] {
        for url in [
            "https://example.com/received.png",
            "data:image/jpeg;base64,anBlZw==",
        ] {
            let mut image = json!({"type":"input_image","image_url":url});
            if let Some(detail) = detail {
                image["detail"] = detail.into();
            }
            let item = json!({"type":"message","role":"user","content":[image]});
            let mock = upstream
                .mock("POST", "/responses")
                .match_body(Matcher::PartialJson(
                    json!({"input":[item.clone()],"prompt_cache_key":"native image history"}),
                ))
                .with_status(200)
                .with_body(completed_stream(&[], None))
                .expect(2)
                .create_async()
                .await;
            sync_json(gateway.post(json!({"model":"codex-fixture","input":[item.clone()],"prompt_cache_key":"native image history"})).await).await;
            let message = serde_json::from_value::<ResponseItem>(item)
                .unwrap()
                .into_message();
            let core:GatewayRequest=serde_json::from_value(json!({"model":"codex-fixture","max_output_tokens":1,"messages":[message],"prompt_cache_key":"native image history"})).unwrap();
            let restored: GatewayRequest =
                serde_json::from_slice(&serde_json::to_vec(&core).unwrap()).unwrap();
            assert_eq!(
                serde_json::to_value(&core).unwrap(),
                serde_json::to_value(&restored).unwrap()
            );
            registry
                .get_provider_for_model("codex-fixture")
                .unwrap()
                .send_message(restored)
                .await
                .unwrap();
            mock.assert_async().await;
        }
    }
}

#[tokio::test]
async fn completed_summary_vectors_and_typed_cache_controls_are_exact() {
    let mut upstream = Server::new_async().await;
    let gateway = RunningGateway::start(&upstream).await;
    for item in [
        json!({"type":"reasoning","summary":[]}),
        json!({"type":"reasoning","summary":[{"type":"summary_text","text":"first"},{"type":"summary_text","text":"second"}],"encrypted_content":""}),
    ] {
        let mock = upstream
            .mock("POST", "/responses")
            .match_body(Matcher::PartialJson(
                json!({"prompt_cache_key":"","client_metadata":{}}),
            ))
            .with_status(200)
            .with_body(completed_stream(std::slice::from_ref(&item), None))
            .create_async()
            .await;
        let body=sync_json(gateway.post(json!({"model":"codex-fixture","input":"summary","prompt_cache_key":"","client_metadata":{}})).await).await;
        assert_eq!(body["output"], json!([item]));
        mock.assert_async().await;
    }
    for (name, bad) in [
        ("prompt_cache_key", Value::Null),
        ("prompt_cache_key", json!(7)),
        ("client_metadata", Value::Null),
        ("client_metadata", json!({"field":false})),
        ("access_programs", Value::Null),
        ("context_management", json!({})),
    ] {
        let mut request = json!({"model":"codex-fixture","input":"invalid control"});
        request[name] = bad;
        assert_eq!(gateway.post(request).await.status().as_u16(), 400);
    }
}

#[tokio::test]
async fn selected_model_copy_is_admitted_before_outbound_construction() {
    use substrate_gateway::cli::ModelMapping;
    use substrate_gateway::models::response_transport::PAYLOAD_LIMIT;
    let mut upstream = Server::new_async().await;
    let unsent = upstream
        .mock("POST", "/responses")
        .expect(0)
        .create_async()
        .await;
    let model = "m".repeat(PAYLOAD_LIMIT + 1);
    let registry = ProviderRegistry::from_configs_with_models(
        &[ProviderConfig {
            name: "selected".into(),
            provider_type: "openai".into(),
            auth_type: AuthType::ApiKey,
            api_key: Some("fixture".into()),
            oauth_provider: None,
            project_id: None,
            location: None,
            base_url: Some(upstream.url()),
            headers: None,
            models: vec![model.clone()],
            enabled: Some(true),
        }],
        None,
        &[],
    )
    .unwrap();
    let gateway = RunningGateway::from_harness(ConformanceHarness::with_registry(
        "visible",
        registry,
        vec![ModelMapping {
            priority: 1,
            provider: "selected".into(),
            actual_model: model,
        }],
    ))
    .await;
    let response=gateway.post(json!({"model":"visible","input":[{"type":"reasoning","summary":[],"encrypted_content":"selected model fixture"}]})).await;
    assert_eq!(response.status().as_u16(), 400);
    unsent.assert_async().await;
}

fn gwb_obs_registry(upstream: &Server, kind: &str, model: &str) -> ProviderRegistry {
    ProviderRegistry::from_configs_with_models(
        &[ProviderConfig {
            name: "local".into(),
            provider_type: kind.into(),
            auth_type: AuthType::ApiKey,
            api_key: Some("fixture-secret".into()),
            oauth_provider: None,
            project_id: None,
            location: None,
            base_url: Some(upstream.url()),
            headers: None,
            models: vec![model.into()],
            enabled: Some(true),
        }],
        None,
        &[],
    )
    .unwrap()
}

#[tokio::test]
async fn gwb_obs01_advanced_api_key_sampling_controls_reach_responses() {
    let mut upstream = Server::new_async().await;
    let gateway = RunningGateway::start(&upstream).await;
    for streaming in [false, true] {
        let mock = upstream
            .mock("POST", "/responses")
            .match_body(Matcher::PartialJson(
                if streaming {
                // Existing Chat and Responses streaming DTOs pass through Value,
                // whose Number preserves the core f32 value after widening.
                json!({"instructions":"caller","temperature":0.20000000298023224,"top_p":0.800000011920929})
            } else {
                json!({"instructions":"caller","temperature":0.2,"top_p":0.8})
            },
            ))
            .with_status(200)
            .with_body(completed_stream(&[], None))
            .create_async()
            .await;
        let response = gateway.post(json!({"model":"codex-fixture","input":"hello","instructions":"caller","temperature":0.2,"top_p":0.8,"stream":streaming})).await;
        mock.assert_async().await;
        assert_eq!(response.status().as_u16(), 200);
        let text = response.text().await.unwrap();
        if streaming {
            assert!(text.contains("response.completed"));
        }
        mock.assert_async().await;
    }
}

#[tokio::test]
async fn gwb_obs01_advanced_responses_stop_is_rejected_without_io() {
    let mut upstream = Server::new_async().await;
    let unsent = upstream
        .mock("POST", "/responses")
        .expect(0)
        .with_status(200)
        .with_body(completed_stream(&[], None))
        .create_async()
        .await;
    let gateway = RunningGateway::start(&upstream).await;
    for streaming in [false, true] {
        let response=gateway.post(json!({"model":"codex-fixture","input":"hello","instructions":"caller","stop":["END"],"stream":streaming})).await;
        assert_eq!(response.status().as_u16(), 502);
        let body: Value = response.json().await.unwrap();
        assert_eq!(
            body,
            json!({"error":{"type":"error","class":"transport_drift","message":"Transport behavior drifted from the operator contract"}})
        );
    }
    unsent.assert_async().await;
}

#[tokio::test]
async fn gwb_obs02_completed_function_status_preserves_reloaded_call_loop() {
    use substrate_gateway::models::response_transport::ResponseItem;
    use substrate_gateway::models::{Message, MessageContent};
    let mut upstream = Server::new_async().await;
    let gateway = RunningGateway::start(&upstream).await;
    for streaming in [false, true] {
        let item = json!({"type":"function_call","id":"function-item","call_id":"function-call","name":"same","namespace":"tools","arguments":" {\"x\": 1} ","status":"completed"});
        let mut normalized = item.clone();
        normalized.as_object_mut().unwrap().remove("status");
        let first = upstream
            .mock("POST", "/responses")
            .with_status(200)
            .with_body(completed_stream(&[item], Some(json!(false))))
            .create_async()
            .await;
        let response = gateway
            .post(json!({"model":"codex-fixture","input":"call","stream":streaming}))
            .await;
        let text = response.text().await.unwrap();
        let output = if streaming {
            assert!(text.contains("response.completed"), "{text}");
            text.split("\n\n")
                .find_map(|frame| {
                    let data = frame.lines().find_map(|line| line.strip_prefix("data: "))?;
                    let v: Value = serde_json::from_str(data).ok()?;
                    (v["type"] == "response.completed").then(|| v["response"]["output"].clone())
                })
                .unwrap()
        } else {
            serde_json::from_str::<Value>(&text).unwrap()["output"].clone()
        };
        assert_eq!(output, json!([normalized.clone()]));
        first.assert_async().await;
        let typed: ResponseItem = serde_json::from_value(normalized.clone()).unwrap();
        let message = typed.into_message();
        let reloaded: Message =
            serde_json::from_slice(&serde_json::to_vec(&message).unwrap()).unwrap();
        let MessageContent::Blocks(blocks) = reloaded.content else {
            panic!("typed call history missing")
        };
        let restored = ResponseItem::from_block(&blocks[0]).unwrap();
        assert_eq!(serde_json::to_value(&restored).unwrap(), normalized);
        let history = json!([restored,{"type":"function_call_output","call_id":"function-call","name":"same","namespace":"tools","output":"result"}]);
        let next = upstream
            .mock("POST", "/responses")
            .match_body(Matcher::PartialJson(json!({"input":history.clone()})))
            .with_status(200)
            .with_body(completed_stream(&[], Some(json!(true))))
            .create_async()
            .await;
        sync_json(
            gateway
                .post(json!({"model":"codex-fixture","input":history}))
                .await,
        )
        .await;
        next.assert_async().await;
    }
}

#[tokio::test]
async fn gwb_obs02_uncompleted_or_malformed_function_status_never_completes() {
    let mut upstream = Server::new_async().await;
    let gateway = RunningGateway::start(&upstream).await;
    for status in [
        Value::Null,
        json!(7),
        json!("unknown"),
        json!("in_progress"),
        json!("incomplete"),
    ] {
        for streaming in [false, true] {
            let item = json!({"type":"function_call","call_id":"call","name":"tool","arguments":"{}","status":status});
            let mock = upstream
                .mock("POST", "/responses")
                .with_status(200)
                .with_body(completed_stream(&[item], None))
                .create_async()
                .await;
            let response = gateway
                .post(json!({"model":"codex-fixture","input":"bad lifecycle","stream":streaming}))
                .await;
            let code = response.status();
            let text = response.text().await.unwrap();
            assert!(!code.is_success() || !text.contains("response.completed"));
            assert!(!text.contains("\"status\":\"completed\""));
            mock.assert_async().await;
        }
    }
}

#[tokio::test]
async fn gwb_obs03_actual_api_key_parser_and_error_sinks_exclude_sentinels() {
    let logs = captured_logs();
    let mut upstream = Server::new_async().await;
    let directory = tempfile::tempdir().unwrap();
    let trace_path = directory.path().join("trace.jsonl");
    let tracer = substrate_gateway::message_tracing::MessageTracer::new(
        substrate_gateway::cli::TracingConfig {
            enabled: true,
            path: trace_path.display().to_string(),
            omit_system_prompt: false,
        },
    );
    let cases = [
        (
            "openai",
            "/chat/completions",
            json!({"id":"chat","object":"chat.completion","created":1,"model":"chat-fixture","choices":[{"index":0,"message":{"role":"assistant","content":"ok"},"finish_reason":"stop"}],"usage":{"prompt_tokens":"OBS03_OPENAI_PRIVATE_SENTINEL","completion_tokens":0,"total_tokens":0}}),
            "OBS03_OPENAI_PRIVATE_SENTINEL",
        ),
        (
            "anthropic",
            "/v1/messages",
            json!({"id":"message","type":"message","role":"assistant","model":"chat-fixture","content":[],"usage":{"input_tokens":"OBS03_ANTHROPIC_PRIVATE_SENTINEL","output_tokens":0}}),
            "OBS03_ANTHROPIC_PRIVATE_SENTINEL",
        ),
        (
            "gemini",
            "/models/chat-fixture:generateContent",
            json!({"candidates":[],"usageMetadata":{"promptTokenCount":"OBS03_GEMINI_PRIVATE_SENTINEL"}}),
            "OBS03_GEMINI_PRIVATE_SENTINEL",
        ),
    ];
    let mut sentinels = Vec::new();
    for (kind, path, body, sentinel) in cases {
        let registry = gwb_obs_registry(&upstream, kind, "chat-fixture");
        let provider = registry.get_provider("local").unwrap();
        let malformed = upstream
            .mock("POST", path)
            .match_query(Matcher::Any)
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(body.to_string())
            .expect(2)
            .create_async()
            .await;
        let request=serde_json::from_value(json!({"model":"chat-fixture","max_output_tokens":1,"messages":[{"role":"user","content":"OBS03_REQUEST_PRIVATE_SENTINEL"}]})).unwrap();
        let error = provider.send_message(request).await.unwrap_err();
        assert!(!format!("{error:?} {error}").contains(sentinel));
        tracer.trace_error("actual-provider-error", &error.to_string());
        let gateway = RunningGateway::from_harness(ConformanceHarness::with_direct_registry(
            "chat-fixture",
            registry,
        ))
        .await;
        let response = gateway
            .post(json!({"model":"chat-fixture","input":"OBS03_REQUEST_PRIVATE_SENTINEL"}))
            .await;
        assert!(!response.status().is_success());
        assert!(!response.text().await.unwrap().contains(sentinel));
        malformed.assert_async().await;
        let rejected = upstream
            .mock("POST", path)
            .match_query(Matcher::Any)
            .with_status(404)
            .with_body(sentinel)
            .create_async()
            .await;
        let response = gateway
            .post(json!({"model":"chat-fixture","input":"error"}))
            .await;
        assert!(!response.status().is_success());
        assert!(!response.text().await.unwrap().contains(sentinel));
        rejected.assert_async().await;
        sentinels.push(sentinel);
    }
    let text = String::from_utf8(logs.lock().unwrap().clone()).unwrap();
    let trace = std::fs::read_to_string(trace_path).unwrap();
    assert_eq!(trace.lines().count(), 3);
    for sentinel in sentinels
        .into_iter()
        .chain(["OBS03_REQUEST_PRIVATE_SENTINEL"])
    {
        assert!(
            !text.contains(sentinel),
            "raw payload reached diagnostic sink: {sentinel}"
        );
        assert!(!trace.contains(sentinel));
    }
}

#[tokio::test]
async fn gwb_obs04_unused_oversized_mapping_is_borrowed_during_selection() {
    use substrate_gateway::cli::ModelMapping;
    use substrate_gateway::models::response_transport::WORKING_LIMIT;
    let mut upstream = Server::new_async().await;
    let registry = gwb_obs_registry(&upstream, "openai", "codex-fixture");
    let gateway = RunningGateway::from_harness(ConformanceHarness::with_registry(
        "visible",
        registry,
        vec![
            ModelMapping {
                priority: 2,
                provider: "unused".into(),
                actual_model: "m".repeat(WORKING_LIMIT + 1),
            },
            ModelMapping {
                priority: 1,
                provider: "local".into(),
                actual_model: "codex-fixture".into(),
            },
        ],
    ))
    .await;
    for streaming in [false, true] {
        let mock = upstream
            .mock("POST", "/responses")
            .match_body(Matcher::PartialJson(
                json!({"model":"codex-fixture","instructions":"pinned"}),
            ))
            .with_status(200)
            .with_body(completed_stream(&[], None))
            .create_async()
            .await;
        let response=gateway.post(json!({"model":"visible","input":"hello","instructions":"pinned","stream":streaming})).await;
        assert_eq!(response.status().as_u16(), 200);
        let text = response.text().await.unwrap();
        if streaming {
            assert!(text.contains("response.completed"));
        }
        mock.assert_async().await;
    }
}

#[tokio::test]
async fn gwb_obs04_selected_destination_fields_are_bounded_before_io() {
    use substrate_gateway::cli::ModelMapping;
    use substrate_gateway::models::response_transport::PAYLOAD_LIMIT;
    let mut upstream = Server::new_async().await;
    let unsent = upstream
        .mock("POST", "/responses")
        .expect(0)
        .create_async()
        .await;
    for oversized_provider in [false, true] {
        let registry = gwb_obs_registry(&upstream, "openai", "codex-fixture");
        let mapping = ModelMapping {
            priority: 1,
            provider: if oversized_provider {
                "p".repeat(PAYLOAD_LIMIT + 1)
            } else {
                "local".into()
            },
            actual_model: if oversized_provider {
                "codex-fixture".into()
            } else {
                "m".repeat(PAYLOAD_LIMIT + 1)
            },
        };
        let gateway = RunningGateway::from_harness(ConformanceHarness::with_registry(
            "visible",
            registry,
            vec![mapping],
        ))
        .await;
        for streaming in [false, true] {
            let response=gateway.post(json!({"model":"visible","input":"hello","instructions":"pin","stream":streaming})).await;
            assert_eq!(response.status().as_u16(), 400);
            let text = response.text().await.unwrap();
            assert!(
                text.len() < 1024,
                "selected configuration echoed into error"
            );
        }
    }
    unsent.assert_async().await;
}
