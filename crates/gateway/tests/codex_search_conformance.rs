use serde_json::json;
use substrate_gateway::models::search::{SearchCommands, SearchRequest, SearchResponse};

#[test]
fn pinned_commands_accept_empty_and_preserve_full_literal() {
    let wire = json!({
        "search_query":[{"q":"query α","recency":7,"domains":["b.example","a.example"]}],
        "image_query":[{"q":"image","domains":[]}],
        "open":[{"ref_id":"turn0search0","lineno":0}],
        "click":[{"ref_id":"turn0view0","id":17}],
        "find":[{"ref_id":"turn0view0","pattern":"needle"}],
        "screenshot":[{"ref_id":"turn0view0","pageno":1}],
        "finance":[{"ticker":"BTC","type":"crypto","market":""}],
        "weather":[{"location":"Country, Area, City","start":"2026-10-24","duration":3}],
        "sports":[{"tool":"sports","fn":"schedule","league":"nba","team":"GSW","opponent":"LAL","date_from":"2026-10-24","date_to":"2026-10-25","num_games":2,"locale":"en-US"}],
        "time":[{"utc_offset":"+03:00"}],"response_length":"long"
    });
    let commands: SearchCommands = serde_json::from_value(wire.clone()).unwrap();
    assert_eq!(serde_json::to_value(commands).unwrap(), wire);
    assert_eq!(
        serde_json::to_value(SearchCommands::default()).unwrap(),
        json!({})
    );
}

#[test]
fn request_rejects_null_unknown_and_wrong_types_without_diagnostics() {
    for bad in [
        json!({"commands":null}),
        json!({"commands":{"unknown":"sentinel"}}),
        json!({"model":1}),
        json!({"unknown":"sentinel"}),
    ] {
        let mut request = json!({"id":"native-id","model":"inference-slug"});
        request
            .as_object_mut()
            .unwrap()
            .extend(bad.as_object().unwrap().clone());
        assert!(serde_json::from_value::<SearchRequest>(request).is_err());
    }
    let request: SearchRequest = serde_json::from_value(
        json!({"id":"private-sentinel","model":"slug","input":"prompt-sentinel"}),
    )
    .unwrap();
    assert!(!format!("{request:?}").contains("sentinel"));
}

#[test]
fn response_preserves_named_opaque_values_and_omission() {
    for wire in [
        json!({"output":"unicode 🦀"}),
        json!({"output":"ok","results":null,"encrypted_output":null}),
        json!({"output":"ok","results":[{"future_result":{"payload":[1,false,null,"sentinel"]}}],"encrypted_output":"encrypted-sentinel"}),
    ] {
        let response: SearchResponse = serde_json::from_value(wire.clone()).unwrap();
        assert!(!format!("{response:?}").contains("sentinel"));
        assert_eq!(serde_json::to_value(response).unwrap(), wire);
    }
    for wire in [
        json!({"output":null}),
        json!({"output":"ok","results":{}}),
        json!({"output":"ok","unknown":"sentinel"}),
    ] {
        assert!(serde_json::from_value::<SearchResponse>(wire).is_err());
    }
}

#[test]
fn pinned_settings_variants_and_input_message_population() {
    for access in [
        json!(false),
        json!(true),
        json!("cached"),
        json!("indexed"),
        json!("live"),
    ] {
        for size in ["low", "medium", "high"] {
            let wire = json!({"id":"native-body-id","model":"actual-slug","reasoning":{"effort":"custom-native-effort","summary":"detailed","context":"all_turns"},"input":[{"type":"message","role":"assistant","phase":"commentary","content":[{"type":"output_text","text":"context"}]}],"settings":{"user_location":{"type":"approximate","country":"US","region":"IN","city":"Indianapolis","timezone":"America/Indiana/Indianapolis"},"search_context_size":size,"filters":{"allowed_domains":["b.test","a.test"],"blocked_domains":[]},"image_settings":{"max_results":3,"caption":false},"allowed_callers":["direct","shell","code_interpreter"],"external_web_access":access},"max_output_tokens":18446744073709551615u64});
            let request: SearchRequest = serde_json::from_value(wire.clone()).unwrap();
            assert_eq!(serde_json::to_value(request).unwrap(), wire);
        }
    }
    for content in [
        json!({"type":"input_text","text":"user"}),
        json!({"type":"output_text","text":"assistant"}),
        json!({"type":"input_image","image_url":"data:image/png;base64,AAAA","detail":"original"}),
        json!({"type":"input_image","file_id":"file-native"}),
        json!({"type":"input_audio","audio_url":"data:audio/wav;base64,AAAA"}),
    ] {
        let wire = json!({"id":"id","model":"slug","input":[{"type":"message","role":"assistant","content":[content]}]});
        let request: SearchRequest = serde_json::from_value(wire.clone()).unwrap();
        assert_eq!(serde_json::to_value(request).unwrap(), wire);
    }
}

#[test]
fn unknown_and_null_nested_fields_never_disappear() {
    for content in [
        json!({"type":"input_image","image_url":"data:fixture","file_id":"file-fixture"}),
        json!({"type":"input_image","image_url":"data:fixture","unknown":1}),
        json!({"type":"input_text","text":"user","unknown":1}),
    ] {
        let wire = json!({"id":"id","model":"slug","input":[{"type":"message","role":"assistant","content":[content]}]});
        assert!(serde_json::from_value::<SearchRequest>(wire).is_err());
    }
    for wire in [
        json!({"settings":{"user_location":null}}),
        json!({"settings":{"external_web_access":"future"}}),
        json!({"commands":{"open":[{"ref_id":"r","lineno":null}]}}),
        json!({"reasoning":{"effort":""}}),
        json!({"reasoning":{"summary":"future"}}),
        json!({"input":[{"type":"function_call","name":"x","arguments":"{}","call_id":"c"}]}),
    ] {
        let mut request = json!({"id":"id","model":"slug"});
        request
            .as_object_mut()
            .unwrap()
            .extend(wire.as_object().unwrap().clone());
        assert!(serde_json::from_value::<SearchRequest>(request).is_err());
    }
}

#[tokio::test]
async fn other_provider_search_defaults_to_deny() {
    use substrate_gateway::models::search::SearchError;
    use substrate_gateway::providers::{AnthropicCompatibleProvider, GatewayProvider};
    let provider = AnthropicCompatibleProvider::new(
        "other".into(),
        "synthetic".into(),
        "http://127.0.0.1:1".into(),
        vec![],
        None,
        None,
    );
    let request = serde_json::from_value(json!({"id":"id","model":"slug"})).unwrap();
    assert_eq!(
        provider.send_search(request).await.unwrap_err(),
        SearchError::Unsupported
    );
}

#[test]
fn all_pinned_closed_enum_variants_are_preserved() {
    for asset in ["equity", "fund", "crypto", "index"] {
        let wire = json!({"finance":[{"ticker":"X","type":asset}]});
        let decoded: SearchCommands = serde_json::from_value(wire.clone()).unwrap();
        assert_eq!(serde_json::to_value(decoded).unwrap(), wire);
    }
    for league in [
        "nba", "wnba", "nfl", "nhl", "mlb", "epl", "ncaamb", "ncaawb", "ipl",
    ] {
        for function in ["schedule", "standings"] {
            let wire = json!({"sports":[{"fn":function,"league":league}]});
            let decoded: SearchCommands = serde_json::from_value(wire.clone()).unwrap();
            assert_eq!(serde_json::to_value(decoded).unwrap(), wire);
        }
    }
    for summary in ["auto", "concise", "detailed", "none"] {
        for context in ["auto", "current_turn", "all_turns"] {
            let wire = json!({"id":"id","model":"slug","reasoning":{"effort":"ultra","summary":summary,"context":context}});
            let decoded: SearchRequest = serde_json::from_value(wire.clone()).unwrap();
            assert_eq!(serde_json::to_value(decoded).unwrap(), wire);
        }
    }
}

#[test]
fn native_metadata_preserves_exact_named_population_and_opaque_leaves() {
    for arguments in [
        json!(null),
        json!(false),
        json!(17),
        json!("argument-sentinel"),
        json!([]),
        json!(["array", 3, null]),
        json!({"Authorization":"credential-sentinel","command":"do-not-execute"}),
        json!({"_codex_executed_tool_call_truncated":{"original_bytes":10000,"max_bytes":8192,"omitted_calls":2,"original_name_bytes":20}}),
    ] {
        for result in [
            json!(null),
            json!(false),
            json!(3.25),
            json!("omitted_due_to_size_limit"),
            json!(["opaque-sentinel", null]),
            json!({"x-provider":"inert","nested":{"command":"inert-sentinel"}}),
        ] {
            let wire = json!({"id":"native-id","model":"slug","input":[{"type":"message","role":"assistant","phase":"final_answer","content":[{"type":"output_text","text":"text"}],"internal_chat_message_metadata_passthrough":{"turn_id":"native-turn","create_time":1772222222.125,"content_item_kinds":["text","future-kind",""],"cell_id":"native-cell","executed_tool_calls":[{"name":"native-tool","arguments":arguments,"tool_result_sources":[{"type":"resource","id":"source-id"},{"type":"parse_failed","id":""}],"tool_result_metadata":result}],"tool_calls_complete":false}}]});
            let decoded: SearchRequest = serde_json::from_value(wire.clone()).unwrap();
            assert!(!format!("{decoded:?}").contains("sentinel"));
            assert_eq!(serde_json::to_value(decoded).unwrap(), wire);
        }
    }
    for metadata in [
        json!({}),
        json!({"executed_tool_calls":[]}),
        json!({"executed_tool_calls":[{"name":"tool","arguments":null}]}),
        json!({"turn_id":"","cell_id":"","create_time":0,"content_item_kinds":[],"tool_calls_complete":true}),
    ] {
        let wire = json!({"id":"id","model":"slug","input":[{"type":"message","role":"user","content":[],"internal_chat_message_metadata_passthrough":metadata}]});
        let decoded: SearchRequest = serde_json::from_value(wire.clone()).unwrap();
        assert_eq!(serde_json::to_value(decoded).unwrap(), wire);
    }
}

#[test]
fn native_metadata_unknown_misplaced_and_non_native_null_fields_reject() {
    for metadata in [
        json!(null),
        json!(false),
        json!({"unknown":"sentinel"}),
        json!({"turn_id":null}),
        json!({"create_time":"1"}),
        json!({"cell_id":null}),
        json!({"content_item_kinds":[1]}),
        json!({"executed_tool_calls":null}),
        json!({"executed_tool_calls":[{"name":"x"}]}),
        json!({"executed_tool_calls":[{"name":{},"arguments":{}}]}),
        json!({"executed_tool_calls":[{"name":"x","arguments":{},"tool_result_sources":null}]}),
        json!({"executed_tool_calls":[{"name":"x","arguments":{},"tool_result_sources":[{"type":"x","id":"x","unknown":true}]}]}),
        json!({"tool_calls_complete":null}),
    ] {
        let wire = json!({"id":"id","model":"slug","input":[{"type":"message","role":"assistant","content":[],"internal_chat_message_metadata_passthrough":metadata}]});
        assert!(serde_json::from_value::<SearchRequest>(wire).is_err());
    }
    for key in [
        "internal_chat_message_metadata_passthrough",
        "arguments",
        "tool_result_metadata",
        "executed_tool_calls",
    ] {
        let mut wire = json!({"id":"id","model":"slug"});
        wire[key] = json!({"untrusted":"sentinel"});
        assert!(serde_json::from_value::<SearchRequest>(wire).is_err());
    }
}
