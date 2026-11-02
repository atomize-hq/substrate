//! Codex backend tests for non-interactive fail-fast behavior during fork flows.
//!
//! Normative source: `docs/specs/codex-app-server-jsonrpc-contract.md` (approval request detection
//! + `$\/cancelRequest` + pinned `"approval required"` failure translation).

use std::time::Duration;

use agent_api::{
    backends::codex::{CodexBackend, CodexBackendConfig},
    AgentWrapperBackend, AgentWrapperError, AgentWrapperEventKind, AgentWrapperRunRequest,
};
use serde_json::json;

use crate::support::{any_event_contains, drain_to_none, fake_codex_app_server_binary};

#[tokio::test]
async fn approval_required_fails_fast_sends_cancel_request_and_surfaces_approval_required() {
    let prompt = "hello world";
    let source_thread_id = "thread-123";
    let secret = "RAW-BACKEND-SECRET-DO-NOT-LEAK";

    let backend = CodexBackend::new(CodexBackendConfig {
        binary: Some(fake_codex_app_server_binary()),
        env: [
            (
                "FAKE_CODEX_APP_SERVER_SCENARIO".to_string(),
                "approval_required_during_turn_start".to_string(),
            ),
            (
                "FAKE_CODEX_APP_SERVER_EXPECT_PROMPT".to_string(),
                prompt.to_string(),
            ),
            (
                "FAKE_CODEX_APP_SERVER_EXPECT_SOURCE_THREAD_ID".to_string(),
                source_thread_id.to_string(),
            ),
            (
                "FAKE_CODEX_APP_SERVER_SECRET_SENTINEL".to_string(),
                secret.to_string(),
            ),
        ]
        .into_iter()
        .collect(),
        ..Default::default()
    });

    let handle = backend
        .run(AgentWrapperRunRequest {
            prompt: prompt.to_string(),
            extensions: [(
                "agent_api.session.fork.v1".to_string(),
                json!({"selector":"id","id": source_thread_id}),
            )]
            .into_iter()
            .collect(),
            ..Default::default()
        })
        .await
        .expect("run");

    let mut events = handle.events;
    let completion = handle.completion;

    let seen = drain_to_none(events.as_mut(), Duration::from_secs(2)).await;
    let error_events: Vec<_> = seen
        .iter()
        .filter(|ev| ev.kind == AgentWrapperEventKind::Error)
        .collect();
    assert_eq!(error_events.len(), 1, "expected exactly one Error event");
    assert_eq!(
        seen.last().map(|ev| ev.kind.clone()),
        Some(AgentWrapperEventKind::Error),
        "expected Error event to be terminal"
    );
    assert_eq!(
        seen.last().and_then(|ev| ev.message.as_deref()),
        Some("approval required")
    );
    assert!(
        !any_event_contains(&seen, secret),
        "expected backend secrets to never leak into message/text/data"
    );

    let err = tokio::time::timeout(Duration::from_secs(2), completion)
        .await
        .expect("completion resolves")
        .unwrap_err();
    match err {
        AgentWrapperError::Backend { message } => assert_eq!(message, "approval required"),
        other => panic!("expected Backend error, got: {other:?}"),
    }
}
