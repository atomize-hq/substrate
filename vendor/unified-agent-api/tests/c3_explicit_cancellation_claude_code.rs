#![cfg(feature = "claude_code")]

use std::{path::PathBuf, pin::Pin, sync::Arc, time::Duration};

use agent_api::{
    backends::claude_code::{ClaudeCodeBackend, ClaudeCodeBackendConfig},
    AgentWrapperError, AgentWrapperEvent, AgentWrapperGateway, AgentWrapperKind,
    AgentWrapperRunRequest,
};
use futures_core::Stream;

const FIRST_EVENT_TIMEOUT: Duration = Duration::from_secs(1);
const CANCEL_TERMINATION_TIMEOUT: Duration = Duration::from_secs(10);

fn fake_claude_binary() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_fake_claude_stream_json_agent_api"))
}

async fn drain_to_none(
    mut stream: Pin<&mut (dyn Stream<Item = AgentWrapperEvent> + Send)>,
) -> Vec<AgentWrapperEvent> {
    let mut out = Vec::new();
    while let Some(ev) = std::future::poll_fn(|cx| stream.as_mut().poll_next(cx)).await {
        out.push(ev);
    }
    out
}

fn claude_gateway_block_until_killed() -> (AgentWrapperGateway, AgentWrapperKind) {
    let backend = Arc::new(ClaudeCodeBackend::new(ClaudeCodeBackendConfig {
        binary: Some(fake_claude_binary()),
        claude_home: None,
        default_timeout: None,
        default_working_dir: None,
        env: [(
            "FAKE_CLAUDE_SCENARIO".to_string(),
            "block_until_killed".to_string(),
        )]
        .into_iter()
        .collect(),
        allow_mcp_write: false,
        allow_external_sandbox_exec: false,
    }));

    let mut gateway = AgentWrapperGateway::new();
    gateway.register(backend).expect("register claude backend");

    let kind = AgentWrapperKind::new("claude_code").expect("valid agent kind");
    assert!(
        gateway
            .backend(&kind)
            .expect("claude backend registered")
            .capabilities()
            .contains("agent_api.control.cancel.v1"),
        "expected claude backend to advertise explicit cancellation"
    );

    (gateway, kind)
}

#[tokio::test]
async fn explicit_cancel_terminates_blocking_backend_and_completion_is_cancelled() {
    let (gateway, kind) = claude_gateway_block_until_killed();

    let run = gateway
        .run_control(
            &kind,
            AgentWrapperRunRequest {
                prompt: "hello".to_string(),
                ..Default::default()
            },
        )
        .await
        .expect("run_control");

    let agent_api::AgentWrapperRunControl { handle, cancel } = run;
    let agent_api::AgentWrapperRunHandle {
        mut events,
        completion,
    } = handle;

    let first = tokio::time::timeout(
        FIRST_EVENT_TIMEOUT,
        std::future::poll_fn(|cx| events.as_mut().poll_next(cx)),
    )
    .await
    .expect("first event should arrive");
    let Some(_first) = first else {
        panic!("events stream ended before the first event");
    };

    cancel.cancel();
    cancel.cancel();

    let (_events, completion_outcome) = tokio::time::timeout(CANCEL_TERMINATION_TIMEOUT, async {
        tokio::join!(drain_to_none(events.as_mut()), completion)
    })
    .await
    .expect("cancellation should terminate the backend and close the event stream");

    match completion_outcome {
        Err(AgentWrapperError::Backend { ref message }) if message == "cancelled" => {}
        other => panic!("expected cancelled completion error, got {other:?}"),
    }
}

#[tokio::test]
async fn dropping_events_does_not_prevent_explicit_cancel() {
    let (gateway, kind) = claude_gateway_block_until_killed();

    let run = gateway
        .run_control(
            &kind,
            AgentWrapperRunRequest {
                prompt: "hello".to_string(),
                ..Default::default()
            },
        )
        .await
        .expect("run_control");

    let agent_api::AgentWrapperRunControl { handle, cancel } = run;
    let agent_api::AgentWrapperRunHandle { events, completion } = handle;
    drop(events);
    tokio::task::yield_now().await;

    cancel.cancel();
    cancel.cancel();

    let completion_outcome = tokio::time::timeout(CANCEL_TERMINATION_TIMEOUT, completion)
        .await
        .expect("completion resolves after cancel");
    match completion_outcome {
        Err(AgentWrapperError::Backend { ref message }) if message == "cancelled" => {}
        other => panic!("expected cancelled completion error, got {other:?}"),
    }
}
