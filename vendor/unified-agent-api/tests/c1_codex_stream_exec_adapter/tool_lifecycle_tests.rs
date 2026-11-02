use super::*;

#[tokio::test]
async fn tool_lifecycle_ok() {
    let backend = CodexBackend::new(CodexBackendConfig {
        binary: Some(fake_codex_binary()),
        env: [(
            "FAKE_CODEX_SCENARIO".to_string(),
            "tool_lifecycle_ok".to_string(),
        )]
        .into_iter()
        .collect(),
        ..Default::default()
    });

    let handle = backend
        .run(AgentWrapperRunRequest {
            prompt: "hello".to_string(),
            ..Default::default()
        })
        .await
        .unwrap();

    let mut events = handle.events;
    let completion = handle.completion;

    let seen = drain_to_none(events.as_mut(), Duration::from_secs(2)).await;

    let first_tool_call =
        find_first_kind(&seen, AgentWrapperEventKind::ToolCall).expect("expected a ToolCall event");
    let first_tool_result = find_first_kind(&seen, AgentWrapperEventKind::ToolResult)
        .expect("expected a ToolResult event");
    assert!(
        first_tool_call < first_tool_result,
        "expected ToolCall to occur before ToolResult"
    );

    for ev in seen.iter() {
        if matches!(
            ev.kind,
            AgentWrapperEventKind::ToolCall | AgentWrapperEventKind::ToolResult
        ) {
            assert_eq!(
                tool_schema(ev),
                Some("agent_api.tools.structured.v1"),
                "expected tools facet schema on every ToolCall/ToolResult"
            );
        }
    }

    assert!(
        !any_event_contains(&seen, "STDOUT-SENTINEL-DO-NOT-LEAK"),
        "expected tool output sentinel to not appear in text/message/data"
    );
    assert!(
        !any_event_contains(&seen, "STDERR-SENTINEL-DO-NOT-LEAK"),
        "expected tool output sentinel to not appear in text/message/data"
    );

    let completion = tokio::time::timeout(Duration::from_secs(2), completion)
        .await
        .expect("completion resolves")
        .unwrap();
    assert!(completion.status.success());
}

#[tokio::test]
async fn tool_lifecycle_fail_unknown_type() {
    let backend = CodexBackend::new(CodexBackendConfig {
        binary: Some(fake_codex_binary()),
        env: [(
            "FAKE_CODEX_SCENARIO".to_string(),
            "tool_lifecycle_fail_unknown_type".to_string(),
        )]
        .into_iter()
        .collect(),
        ..Default::default()
    });

    let handle = backend
        .run(AgentWrapperRunRequest {
            prompt: "hello".to_string(),
            ..Default::default()
        })
        .await
        .unwrap();

    let mut events = handle.events;
    let completion = handle.completion;

    let seen = drain_to_none(events.as_mut(), Duration::from_secs(2)).await;
    assert!(
        find_first_kind(&seen, AgentWrapperEventKind::Error).is_some(),
        "expected an Error event when item.failed has no deterministically-attributable item_type"
    );
    assert!(
        !seen.iter().any(|ev| {
            ev.kind == AgentWrapperEventKind::ToolResult
                && tool_field(ev, "phase").and_then(Value::as_str) == Some("fail")
        }),
        "expected no failure ToolResult when item_type is absent"
    );

    let completion = tokio::time::timeout(Duration::from_secs(2), completion)
        .await
        .expect("completion resolves")
        .unwrap();
    assert!(completion.status.success());
}

#[tokio::test]
async fn tool_lifecycle_fail_known_type() {
    let backend = CodexBackend::new(CodexBackendConfig {
        binary: Some(fake_codex_binary()),
        env: [(
            "FAKE_CODEX_SCENARIO".to_string(),
            "tool_lifecycle_fail_known_type".to_string(),
        )]
        .into_iter()
        .collect(),
        ..Default::default()
    });

    let handle = backend
        .run(AgentWrapperRunRequest {
            prompt: "hello".to_string(),
            ..Default::default()
        })
        .await
        .unwrap();

    let mut events = handle.events;
    let completion = handle.completion;

    let seen = drain_to_none(events.as_mut(), Duration::from_secs(2)).await;
    assert!(
        seen.iter().any(|ev| {
            ev.kind == AgentWrapperEventKind::ToolResult
                && tool_field(ev, "phase").and_then(Value::as_str) == Some("fail")
                && tool_field(ev, "status").and_then(Value::as_str) == Some("failed")
                && tool_field(ev, "kind").and_then(Value::as_str) == Some("command_execution")
        }),
        "expected failure ToolResult when item.failed has deterministically-attributable item_type"
    );

    let completion = tokio::time::timeout(Duration::from_secs(2), completion)
        .await
        .expect("completion resolves")
        .unwrap();
    assert!(completion.status.success());
}
