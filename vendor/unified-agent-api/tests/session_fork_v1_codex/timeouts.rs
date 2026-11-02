//! Codex backend tests for timeout enforcement during fork flows.
//!
//! Normative source: `docs/specs/codex-app-server-jsonrpc-contract.md` (timeout countdown +
//! `$\/cancelRequest` + pinned safe timeout message).

use std::time::Duration;

use agent_api::{
    backends::codex::{CodexBackend, CodexBackendConfig},
    AgentWrapperBackend, AgentWrapperError, AgentWrapperEventKind, AgentWrapperRunRequest,
};
use serde_json::json;

use crate::support::{drain_to_none, fake_codex_app_server_binary};

const PINNED_CODEX_TIMEOUT_MESSAGE: &str =
    "codex backend error: timeout (details redacted when unsafe)";

#[tokio::test]
async fn fork_id_request_timeout_is_honored_and_emits_terminal_error() {
    let prompt = "hello world";
    let source_thread_id = "thread-123";

    let backend = CodexBackend::new(CodexBackendConfig {
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
    });

    let handle = backend
        .run(AgentWrapperRunRequest {
            prompt: prompt.to_string(),
            timeout: Some(Duration::from_millis(500)),
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

    let agent_api::AgentWrapperRunHandle {
        mut events,
        completion,
    } = handle;

    let (seen, completion_outcome) = tokio::time::timeout(Duration::from_secs(3), async {
        tokio::join!(
            drain_to_none(events.as_mut(), Duration::from_secs(2)),
            completion
        )
    })
    .await
    .expect("timeout should terminate the backend and close the event stream");

    assert!(
        seen.iter().any(|ev| {
            ev.kind == AgentWrapperEventKind::Error
                && ev.message.as_deref() == Some(PINNED_CODEX_TIMEOUT_MESSAGE)
        }),
        "expected a terminal Error event with the pinned timeout message"
    );

    match completion_outcome {
        Err(AgentWrapperError::Backend { ref message })
            if message == PINNED_CODEX_TIMEOUT_MESSAGE => {}
        other => panic!("expected timeout completion error, got {other:?}"),
    }
}

#[tokio::test]
async fn fork_id_default_timeout_is_honored_when_request_timeout_absent() {
    let prompt = "hello world";
    let source_thread_id = "thread-123";

    let backend = CodexBackend::new(CodexBackendConfig {
        binary: Some(fake_codex_app_server_binary()),
        default_timeout: Some(Duration::from_millis(500)),
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

    let agent_api::AgentWrapperRunHandle {
        mut events,
        completion,
    } = handle;

    let (_seen, completion_outcome) = tokio::time::timeout(Duration::from_secs(3), async {
        tokio::join!(
            drain_to_none(events.as_mut(), Duration::from_secs(2)),
            completion
        )
    })
    .await
    .expect("default timeout should terminate the backend and close the event stream");

    match completion_outcome {
        Err(AgentWrapperError::Backend { ref message })
            if message == PINNED_CODEX_TIMEOUT_MESSAGE => {}
        other => panic!("expected timeout completion error, got {other:?}"),
    }
}

#[tokio::test]
async fn fork_id_timeout_duration_zero_disables_timeout() {
    let prompt = "hello world";
    let source_thread_id = "thread-123";

    let backend = CodexBackend::new(CodexBackendConfig {
        binary: Some(fake_codex_app_server_binary()),
        default_timeout: Some(Duration::from_millis(1)),
        env: [
            (
                "FAKE_CODEX_APP_SERVER_SCENARIO".to_string(),
                "fork_id_success".to_string(),
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
    });

    let handle = backend
        .run(AgentWrapperRunRequest {
            prompt: prompt.to_string(),
            timeout: Some(Duration::ZERO),
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

    let agent_api::AgentWrapperRunHandle {
        mut events,
        completion,
    } = handle;

    let (_seen, completion_outcome) = tokio::time::timeout(Duration::from_secs(3), async {
        tokio::join!(
            drain_to_none(events.as_mut(), Duration::from_secs(2)),
            completion
        )
    })
    .await
    .expect("completion should resolve");

    assert!(completion_outcome.is_ok());
}

#[tokio::test]
async fn fork_id_timeout_counts_down_without_polling_completion() {
    let prompt = "hello world";
    let source_thread_id = "thread-123";

    let backend = CodexBackend::new(CodexBackendConfig {
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
    });

    let handle = backend
        .run(AgentWrapperRunRequest {
            prompt: prompt.to_string(),
            timeout: Some(Duration::from_millis(500)),
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

    let agent_api::AgentWrapperRunHandle {
        mut events,
        completion,
    } = handle;

    let _seen = drain_to_none(events.as_mut(), Duration::from_secs(1)).await;

    let completion_outcome = tokio::time::timeout(Duration::from_secs(2), completion)
        .await
        .expect("completion resolves after timeout triggers")
        .unwrap_err();

    match completion_outcome {
        AgentWrapperError::Backend { ref message } if message == PINNED_CODEX_TIMEOUT_MESSAGE => {}
        other => panic!("expected timeout completion error, got {other:?}"),
    }
}
