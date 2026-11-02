//! Codex backend tests for explicit cancellation during fork flows.
//!
//! Normative source: `docs/specs/codex-app-server-jsonrpc-contract.md` (JSON-RPC cancellation
//! wiring) + the Unified Agent API run protocol spec (event/completion precedence).

use std::time::Duration;

use agent_api::{
    backends::codex::{CodexBackend, CodexBackendConfig},
    AgentWrapperError, AgentWrapperGateway, AgentWrapperKind, AgentWrapperRunRequest,
};
use serde_json::json;

use crate::support::{drain_to_none, fake_codex_app_server_binary};

#[tokio::test]
async fn explicit_cancel_sends_cancel_request_and_completion_is_cancelled() {
    let prompt = "hello world";
    let source_thread_id = "thread-123";

    let backend = std::sync::Arc::new(CodexBackend::new(CodexBackendConfig {
        binary: Some(fake_codex_app_server_binary()),
        env: [
            (
                "FAKE_CODEX_APP_SERVER_SCENARIO".to_string(),
                "block_until_cancel".to_string(),
            ),
            (
                "FAKE_CODEX_APP_SERVER_EXPECT_PROMPT".to_string(),
                prompt.to_string(),
            ),
            (
                "FAKE_CODEX_APP_SERVER_EXPECT_SOURCE_THREAD_ID".to_string(),
                source_thread_id.to_string(),
            ),
        ]
        .into_iter()
        .collect(),
        ..Default::default()
    }));

    let mut gateway = AgentWrapperGateway::new();
    gateway.register(backend).expect("register codex backend");
    let kind = AgentWrapperKind::new("codex").expect("valid kind");

    let run = gateway
        .run_control(
            &kind,
            AgentWrapperRunRequest {
                prompt: prompt.to_string(),
                extensions: [(
                    "agent_api.session.fork.v1".to_string(),
                    json!({"selector":"id","id": source_thread_id}),
                )]
                .into_iter()
                .collect(),
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
        Duration::from_secs(1),
        std::future::poll_fn(|cx| events.as_mut().poll_next(cx)),
    )
    .await
    .expect("first event should arrive");
    let Some(_first) = first else {
        panic!("events stream ended before first event");
    };

    cancel.cancel();
    cancel.cancel();

    let (rest, completion_outcome) = tokio::time::timeout(Duration::from_secs(3), async {
        tokio::join!(
            drain_to_none(events.as_mut(), Duration::from_secs(2)),
            completion
        )
    })
    .await
    .expect("cancellation should terminate the backend and close the event stream");

    let _ = rest;

    match completion_outcome {
        Err(AgentWrapperError::Backend { ref message }) if message == "cancelled" => {}
        other => panic!("expected cancelled completion error, got {other:?}"),
    }
}
