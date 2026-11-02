#![cfg(feature = "claude_code")]

use std::{
    path::PathBuf,
    pin::Pin,
    time::{Duration, Instant},
};

use agent_api::{
    backends::claude_code::{ClaudeCodeBackend, ClaudeCodeBackendConfig},
    AgentWrapperBackend, AgentWrapperEvent, AgentWrapperEventKind, AgentWrapperRunRequest,
};
use futures_core::Stream;

async fn next_event(
    mut stream: Pin<&mut (dyn Stream<Item = AgentWrapperEvent> + Send)>,
) -> Option<AgentWrapperEvent> {
    std::future::poll_fn(|cx| stream.as_mut().poll_next(cx)).await
}

fn fake_claude_binary() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_fake_claude_stream_json_agent_api"))
}

#[test]
fn claude_backend_advertises_live_events_capability() {
    let backend = ClaudeCodeBackend::new(ClaudeCodeBackendConfig::default());
    let capabilities = backend.capabilities();
    assert!(capabilities.contains("agent_api.events.live"));
}

#[tokio::test]
async fn events_are_observable_before_process_exit() {
    let backend = ClaudeCodeBackend::new(ClaudeCodeBackendConfig {
        binary: Some(fake_claude_binary()),
        claude_home: None,
        default_timeout: None,
        default_working_dir: None,
        env: [(
            "FAKE_CLAUDE_SCENARIO".to_string(),
            "two_events_long_delay".to_string(),
        )]
        .into_iter()
        .collect(),
        allow_mcp_write: false,
        allow_external_sandbox_exec: false,
    });

    let handle = backend
        .run(AgentWrapperRunRequest {
            prompt: "hello".to_string(),
            ..Default::default()
        })
        .await
        .unwrap();

    let mut events = handle.events;
    let mut completion = handle.completion;

    let started_at = Instant::now();
    let first = tokio::select! {
        biased;
        res = &mut completion => panic!("completion resolved before first event: {res:?}"),
        item = tokio::time::timeout(Duration::from_millis(1500), next_event(events.as_mut())) => {
            item.expect("first event arrives within timeout").expect("stream open")
        }
    };
    assert!(
        started_at.elapsed() < Duration::from_millis(1500),
        "first event should be observable before the fake process is allowed to exit"
    );
    assert_eq!(first.kind, AgentWrapperEventKind::Status);

    tokio::time::timeout(Duration::from_secs(3), async {
        while next_event(events.as_mut()).await.is_some() {}
    })
    .await
    .expect("drain stream to None");

    let completion = tokio::time::timeout(Duration::from_secs(2), completion)
        .await
        .expect("completion resolves after stream finality")
        .unwrap();
    assert!(completion.status.success());
}

#[tokio::test]
async fn completion_is_gated_until_events_stream_is_drained_to_none() {
    let backend = ClaudeCodeBackend::new(ClaudeCodeBackendConfig {
        binary: Some(fake_claude_binary()),
        claude_home: None,
        default_timeout: None,
        default_working_dir: None,
        env: [(
            "FAKE_CLAUDE_SCENARIO".to_string(),
            "single_event_then_exit".to_string(),
        )]
        .into_iter()
        .collect(),
        allow_mcp_write: false,
        allow_external_sandbox_exec: false,
    });

    let handle = backend
        .run(AgentWrapperRunRequest {
            prompt: "hello".to_string(),
            ..Default::default()
        })
        .await
        .unwrap();

    let mut events = handle.events;
    let mut completion = handle.completion;

    let first = tokio::time::timeout(Duration::from_secs(1), next_event(events.as_mut()))
        .await
        .expect("first event arrives")
        .expect("stream open");
    assert_eq!(first.kind, AgentWrapperEventKind::Status);

    // DR-0012: even if the backend has finished and closed the channel, `completion` MUST remain
    // pending until the consumer observes stream finality (polls to `None`) or drops the stream.
    tokio::select! {
        biased;
        res = &mut completion => panic!("completion resolved before stream finality: {res:?}"),
        _ = tokio::time::sleep(Duration::from_millis(50)) => {}
    }

    let next = tokio::time::timeout(Duration::from_secs(1), next_event(events.as_mut()))
        .await
        .expect("stream finality observed");
    assert!(
        next.is_none(),
        "expected stream to be closed after a single event"
    );

    let completion = tokio::time::timeout(Duration::from_secs(2), completion)
        .await
        .expect("completion resolves after stream finality")
        .unwrap();
    assert!(completion.status.success());
}

#[tokio::test]
async fn dropping_events_stream_unblocks_completion() {
    let backend = ClaudeCodeBackend::new(ClaudeCodeBackendConfig {
        binary: Some(fake_claude_binary()),
        claude_home: None,
        default_timeout: None,
        default_working_dir: None,
        env: [(
            "FAKE_CLAUDE_SCENARIO".to_string(),
            "single_event_then_exit".to_string(),
        )]
        .into_iter()
        .collect(),
        allow_mcp_write: false,
        allow_external_sandbox_exec: false,
    });

    let handle = backend
        .run(AgentWrapperRunRequest {
            prompt: "hello".to_string(),
            ..Default::default()
        })
        .await
        .unwrap();

    let agent_api::AgentWrapperRunHandle { events, completion } = handle;
    drop(events);

    let completion = tokio::time::timeout(Duration::from_secs(2), completion)
        .await
        .expect("completion resolves after dropping events")
        .unwrap();
    assert!(completion.status.success());
}

#[tokio::test]
async fn final_text_is_populated_even_if_events_stream_is_dropped() {
    let backend = ClaudeCodeBackend::new(ClaudeCodeBackendConfig {
        binary: Some(fake_claude_binary()),
        claude_home: None,
        default_timeout: None,
        default_working_dir: None,
        env: [(
            "FAKE_CLAUDE_SCENARIO".to_string(),
            "final_text_and_tools".to_string(),
        )]
        .into_iter()
        .collect(),
        allow_mcp_write: false,
        allow_external_sandbox_exec: false,
    });

    let handle = backend
        .run(AgentWrapperRunRequest {
            prompt: "hello".to_string(),
            ..Default::default()
        })
        .await
        .unwrap();

    let agent_api::AgentWrapperRunHandle { events, completion } = handle;
    drop(events);

    let completion = tokio::time::timeout(Duration::from_secs(2), completion)
        .await
        .expect("completion resolves after dropping events")
        .unwrap();
    assert!(completion.status.success());
    assert_eq!(completion.final_text.as_deref(), Some("hello"));
}

#[tokio::test]
async fn tools_facet_and_final_text_are_populated() {
    let backend = ClaudeCodeBackend::new(ClaudeCodeBackendConfig {
        binary: Some(fake_claude_binary()),
        claude_home: None,
        default_timeout: None,
        default_working_dir: None,
        env: [(
            "FAKE_CLAUDE_SCENARIO".to_string(),
            "final_text_and_tools".to_string(),
        )]
        .into_iter()
        .collect(),
        allow_mcp_write: false,
        allow_external_sandbox_exec: false,
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

    let mut saw_tool_call = false;
    let mut saw_tool_result = false;
    let mut saw_delta = false;

    while let Some(event) = next_event(events.as_mut()).await {
        match event.kind {
            AgentWrapperEventKind::ToolCall => {
                assert_eq!(event.channel.as_deref(), Some("tool"));
                assert_eq!(event.text, None);
                assert_eq!(event.message, None);
                assert!(event.data.is_some());

                let data = event.data.as_ref().expect("tool facet present");
                assert_eq!(
                    data.get("schema").and_then(|v| v.as_str()),
                    Some("agent_api.tools.structured.v1")
                );
                assert!(data
                    .pointer("/tool/backend_item_id")
                    .is_some_and(|v| v.is_null()));
                assert!(data.pointer("/tool/thread_id").is_some_and(|v| v.is_null()));
                assert!(data.pointer("/tool/turn_id").is_some_and(|v| v.is_null()));

                let phase = data.pointer("/tool/phase").and_then(|v| v.as_str());
                if phase == Some("delta") {
                    saw_delta = true;
                }

                saw_tool_call = true;
            }
            AgentWrapperEventKind::ToolResult => {
                assert_eq!(event.channel.as_deref(), Some("tool"));
                assert_eq!(event.text, None);
                assert_eq!(event.message, None);
                assert!(event.data.is_some());

                let data = event.data.as_ref().expect("tool facet present");
                assert_eq!(
                    data.get("schema").and_then(|v| v.as_str()),
                    Some("agent_api.tools.structured.v1")
                );
                assert!(data
                    .pointer("/tool/backend_item_id")
                    .is_some_and(|v| v.is_null()));
                assert!(data.pointer("/tool/thread_id").is_some_and(|v| v.is_null()));
                assert!(data.pointer("/tool/turn_id").is_some_and(|v| v.is_null()));

                saw_tool_result = true;
            }
            _ => {}
        }
    }

    assert!(saw_tool_call);
    assert!(saw_tool_result);
    assert!(saw_delta);

    let completion = tokio::time::timeout(Duration::from_secs(2), completion)
        .await
        .expect("completion resolves after stream finality")
        .unwrap();
    assert_eq!(completion.final_text.as_deref(), Some("hello"));
}
