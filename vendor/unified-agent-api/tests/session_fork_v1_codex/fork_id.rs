//! Codex backend tests for `agent_api.session.fork.v1` when `selector == "id"`.
//!
//! Normative source: `docs/specs/codex-app-server-jsonrpc-contract.md`.

use std::time::Duration;

use agent_api::{
    backends::codex::{CodexBackend, CodexBackendConfig},
    AgentWrapperBackend, AgentWrapperError, AgentWrapperEvent, AgentWrapperEventKind,
    AgentWrapperRunRequest,
};
use serde_json::json;
use tempfile::tempdir;

use crate::support::{
    add_dirs_payload, any_event_contains, drain_to_none, fake_codex_app_server_binary,
    read_logged_request_methods, request_log_file,
};

#[tokio::test]
async fn fork_id_does_not_list_and_starts_turn_on_forked_thread() {
    let prompt = "hello world";
    let source_thread_id = "thread-123";

    let backend = CodexBackend::new(CodexBackendConfig {
        binary: Some(fake_codex_app_server_binary()),
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
    let handle_events: Vec<&AgentWrapperEvent> = seen
        .iter()
        .filter(|ev| ev.kind == AgentWrapperEventKind::Status)
        .filter(|ev| {
            ev.data
                .as_ref()
                .and_then(|data| data.get("schema"))
                .and_then(serde_json::Value::as_str)
                == Some("agent_api.session.handle.v1")
        })
        .collect();
    assert_eq!(
        handle_events.len(),
        1,
        "expected exactly one Status event with the session handle facet"
    );
    assert_eq!(
        handle_events[0]
            .data
            .as_ref()
            .and_then(|data| data.get("session"))
            .and_then(|session| session.get("id"))
            .and_then(serde_json::Value::as_str),
        Some("forked-1")
    );

    let completion = tokio::time::timeout(Duration::from_secs(2), completion)
        .await
        .expect("completion resolves")
        .expect("completion ok");
    assert!(completion.status.success());
    assert_eq!(
        completion
            .data
            .as_ref()
            .and_then(|data| data.get("schema"))
            .and_then(serde_json::Value::as_str),
        Some("agent_api.session.handle.v1")
    );
    assert_eq!(
        completion
            .data
            .as_ref()
            .and_then(|data| data.get("session"))
            .and_then(|session| session.get("id"))
            .and_then(serde_json::Value::as_str),
        Some("forked-1")
    );
}

#[tokio::test]
async fn fork_id_oversize_forked_thread_id_is_treated_as_unknown_omits_handle_facet_and_warns_once()
{
    let prompt = "hello world";
    let source_thread_id = "thread-123";
    let oversize = "a".repeat(1025);

    let backend = CodexBackend::new(CodexBackendConfig {
        binary: Some(fake_codex_app_server_binary()),
        env: [
            (
                "FAKE_CODEX_APP_SERVER_SCENARIO".to_string(),
                "fork_id_success_oversize_thread_id".to_string(),
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

    let mut events = handle.events;
    let completion = handle.completion;

    let seen = drain_to_none(events.as_mut(), Duration::from_secs(2)).await;
    assert!(
        !seen.iter().any(|ev| {
            ev.data
                .as_ref()
                .and_then(|data| data.get("schema"))
                .and_then(serde_json::Value::as_str)
                == Some("agent_api.session.handle.v1")
        }),
        "expected oversize forked thread id to be treated as unknown (no handle facet)"
    );

    let warnings: Vec<&AgentWrapperEvent> = seen
        .iter()
        .filter(|ev| ev.kind == AgentWrapperEventKind::Status)
        .filter(|ev| {
            ev.message.as_deref().is_some_and(|message| {
                message.contains("session handle id oversize")
                    && message.contains("len_bytes=1025")
                    && !message.contains(&oversize)
            })
        })
        .collect();
    assert_eq!(warnings.len(), 1, "expected exactly one oversize warning");

    assert!(
        !any_event_contains(&seen, &oversize),
        "expected oversize id to not appear in message/text/data"
    );

    let completion = tokio::time::timeout(Duration::from_secs(2), completion)
        .await
        .expect("completion resolves")
        .expect("completion ok");
    assert!(completion.status.success());
    assert_eq!(completion.data, None);
}

#[tokio::test]
async fn fork_id_forked_thread_id_len_1024_emits_handle_facet_and_does_not_warn() {
    let prompt = "hello world";
    let source_thread_id = "thread-123";

    let backend = CodexBackend::new(CodexBackendConfig {
        binary: Some(fake_codex_app_server_binary()),
        env: [
            (
                "FAKE_CODEX_APP_SERVER_SCENARIO".to_string(),
                "fork_id_success_thread_id_len_1024".to_string(),
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

    let mut events = handle.events;
    let completion = handle.completion;

    let seen = drain_to_none(events.as_mut(), Duration::from_secs(2)).await;

    let handle_events: Vec<&AgentWrapperEvent> = seen
        .iter()
        .filter(|ev| ev.kind == AgentWrapperEventKind::Status)
        .filter(|ev| {
            ev.data
                .as_ref()
                .and_then(|data| data.get("schema"))
                .and_then(serde_json::Value::as_str)
                == Some("agent_api.session.handle.v1")
        })
        .collect();
    assert_eq!(handle_events.len(), 1);
    let id = handle_events[0]
        .data
        .as_ref()
        .and_then(|data| data.get("session"))
        .and_then(|session| session.get("id"))
        .and_then(serde_json::Value::as_str)
        .expect("handle facet session.id present");
    assert_eq!(id.len(), 1024);
    assert!(id.as_bytes().iter().all(|b| *b == b'a'));

    assert!(
        !seen.iter().any(|ev| {
            ev.message
                .as_deref()
                .is_some_and(|message| message.contains("session handle id oversize"))
        }),
        "expected no oversize warning for a 1024-byte id"
    );

    let completion = tokio::time::timeout(Duration::from_secs(2), completion)
        .await
        .expect("completion resolves")
        .expect("completion ok");
    assert!(completion.status.success());
    let completion_id = completion
        .data
        .as_ref()
        .and_then(|data| data.get("session"))
        .and_then(|session| session.get("id"))
        .and_then(serde_json::Value::as_str)
        .expect("completion handle facet session.id present");
    assert_eq!(completion_id.len(), 1024);
    assert!(completion_id.as_bytes().iter().all(|b| *b == b'a'));
}

#[tokio::test]
async fn fork_id_not_found_translates_to_session_not_found_and_never_leaks_backend_details() {
    let prompt = "hello world";
    let source_thread_id = "thread-123";
    let secret = "RAW-BACKEND-SECRET-DO-NOT-LEAK";

    let backend = CodexBackend::new(CodexBackendConfig {
        binary: Some(fake_codex_app_server_binary()),
        env: [
            (
                "FAKE_CODEX_APP_SERVER_SCENARIO".to_string(),
                "fork_id_not_found".to_string(),
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
        Some("session not found")
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
        AgentWrapperError::Backend { message } => assert_eq!(message, "session not found"),
        other => panic!("expected Backend error, got: {other:?}"),
    }
}

#[tokio::test]
async fn fork_id_with_accepted_add_dirs_rejects_before_app_server_startup() {
    let temp = tempdir().expect("tempdir");
    let working_dir = temp.path().join("working-dir");
    let extra_dir = working_dir.join("docs");
    std::fs::create_dir_all(&extra_dir).expect("create add-dir target");
    let request_log = request_log_file();

    let err = CodexBackend::new(CodexBackendConfig {
        binary: Some(fake_codex_app_server_binary()),
        env: [
            (
                "FAKE_CODEX_APP_SERVER_SCENARIO".to_string(),
                "fork_id_not_found".to_string(),
            ),
            (
                "FAKE_CODEX_APP_SERVER_REQUEST_LOG".to_string(),
                request_log.path().display().to_string(),
            ),
        ]
        .into_iter()
        .collect(),
        ..Default::default()
    })
    .run(AgentWrapperRunRequest {
        prompt: "hello world".to_string(),
        working_dir: Some(working_dir),
        extensions: [
            (
                "agent_api.session.fork.v1".to_string(),
                json!({"selector":"id","id":"thread-123"}),
            ),
            (
                "agent_api.exec.add_dirs.v1".to_string(),
                add_dirs_payload(&["docs"]),
            ),
        ]
        .into_iter()
        .collect(),
        ..Default::default()
    })
    .await
    .expect_err("accepted add-dirs on fork should reject before startup");

    match err {
        AgentWrapperError::Backend { message } => {
            assert_eq!(message, "add_dirs unsupported for codex fork")
        }
        other => panic!("expected Backend error, got: {other:?}"),
    }

    assert!(
        read_logged_request_methods(&request_log).is_empty(),
        "expected no app-server JSON-RPC traffic on accepted-input rejection path"
    );
}

#[tokio::test]
async fn fork_id_non_directory_add_dirs_beats_fork_rejection() {
    let temp = tempdir().expect("tempdir");
    let working_dir = temp.path().join("working-dir");
    let non_directory = working_dir.join("not-a-dir.txt");
    std::fs::create_dir_all(&working_dir).expect("create working dir");
    std::fs::write(&non_directory, "hello").expect("create file target");
    let request_log = request_log_file();

    let err = CodexBackend::new(CodexBackendConfig {
        binary: Some(fake_codex_app_server_binary()),
        env: [
            (
                "FAKE_CODEX_APP_SERVER_SCENARIO".to_string(),
                "fork_id_not_found".to_string(),
            ),
            (
                "FAKE_CODEX_APP_SERVER_REQUEST_LOG".to_string(),
                request_log.path().display().to_string(),
            ),
        ]
        .into_iter()
        .collect(),
        ..Default::default()
    })
    .run(AgentWrapperRunRequest {
        prompt: "hello world".to_string(),
        working_dir: Some(working_dir),
        extensions: [
            (
                "agent_api.session.fork.v1".to_string(),
                json!({"selector":"id","id":"thread-123"}),
            ),
            (
                "agent_api.exec.add_dirs.v1".to_string(),
                add_dirs_payload(&[non_directory.to_string_lossy().as_ref()]),
            ),
        ]
        .into_iter()
        .collect(),
        ..Default::default()
    })
    .await
    .expect_err("non-directory add-dirs should fail during validation");

    match &err {
        AgentWrapperError::InvalidRequest { message } => {
            assert_eq!(message, "invalid agent_api.exec.add_dirs.v1.dirs[0]")
        }
        other => panic!("expected InvalidRequest, got: {other:?}"),
    }
    assert!(
        !err.to_string()
            .contains(&non_directory.to_string_lossy().to_string()),
        "expected invalid-input path to stay redacted"
    );
    assert!(
        read_logged_request_methods(&request_log).is_empty(),
        "expected no app-server JSON-RPC traffic on invalid-input path"
    );
}
