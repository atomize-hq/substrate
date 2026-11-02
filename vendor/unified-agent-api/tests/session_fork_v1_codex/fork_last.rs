//! Codex backend tests for `agent_api.session.fork.v1` when `selector == "last"`.
//!
//! Normative source: `docs/specs/codex-app-server-jsonrpc-contract.md` (`thread/list` paging +
//! deterministic selection).

use std::{path::PathBuf, time::Duration};

use agent_api::{
    backends::codex::{CodexBackend, CodexBackendConfig},
    AgentWrapperBackend, AgentWrapperError, AgentWrapperEventKind, AgentWrapperRunRequest,
};
use serde_json::json;
use tempfile::tempdir;

use crate::support::{
    add_dirs_payload, drain_to_none, fake_codex_app_server_binary, read_logged_request_methods,
    request_log_file,
};

fn make_temp_working_dir() -> PathBuf {
    let mut path = std::env::temp_dir();
    let unique = format!(
        "agent_api-test-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos()
    );
    path.push(unique);
    std::fs::create_dir_all(&path).expect("create temp working dir");
    path
}

#[tokio::test]
async fn fork_last_pages_thread_list_and_selects_max_tuple() {
    let prompt = "hello world";
    let working_dir = make_temp_working_dir();

    let backend = CodexBackend::new(CodexBackendConfig {
        binary: Some(fake_codex_app_server_binary()),
        env: [
            (
                "FAKE_CODEX_APP_SERVER_SCENARIO".to_string(),
                "fork_last_success_paged".to_string(),
            ),
            (
                "FAKE_CODEX_APP_SERVER_EXPECT_PROMPT".to_string(),
                prompt.to_string(),
            ),
            (
                "FAKE_CODEX_APP_SERVER_EXPECT_CWD".to_string(),
                working_dir.to_string_lossy().to_string(),
            ),
        ]
        .into_iter()
        .collect(),
        ..Default::default()
    });

    let handle = backend
        .run(AgentWrapperRunRequest {
            prompt: prompt.to_string(),
            working_dir: Some(working_dir),
            extensions: [(
                "agent_api.session.fork.v1".to_string(),
                json!({"selector":"last"}),
            )]
            .into_iter()
            .collect(),
            ..Default::default()
        })
        .await
        .expect("run");

    let mut events = handle.events;
    let completion = handle.completion;

    let _seen = drain_to_none(events.as_mut(), Duration::from_secs(2)).await;

    let completion = tokio::time::timeout(Duration::from_secs(2), completion)
        .await
        .expect("completion resolves")
        .expect("completion ok");
    assert!(completion.status.success());
}

#[tokio::test]
async fn fork_last_empty_list_translates_to_no_session_found_and_emits_terminal_error_event() {
    let prompt = "hello world";
    let working_dir = make_temp_working_dir();

    let backend = CodexBackend::new(CodexBackendConfig {
        binary: Some(fake_codex_app_server_binary()),
        env: [
            (
                "FAKE_CODEX_APP_SERVER_SCENARIO".to_string(),
                "fork_last_empty".to_string(),
            ),
            (
                "FAKE_CODEX_APP_SERVER_EXPECT_PROMPT".to_string(),
                prompt.to_string(),
            ),
            (
                "FAKE_CODEX_APP_SERVER_EXPECT_CWD".to_string(),
                working_dir.to_string_lossy().to_string(),
            ),
        ]
        .into_iter()
        .collect(),
        ..Default::default()
    });

    let handle = backend
        .run(AgentWrapperRunRequest {
            prompt: prompt.to_string(),
            working_dir: Some(working_dir),
            extensions: [(
                "agent_api.session.fork.v1".to_string(),
                json!({"selector":"last"}),
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
        Some("no session found")
    );

    let err = tokio::time::timeout(Duration::from_secs(2), completion)
        .await
        .expect("completion resolves")
        .unwrap_err();
    match err {
        AgentWrapperError::Backend { message } => assert_eq!(message, "no session found"),
        other => panic!("expected Backend error, got: {other:?}"),
    }
}

#[tokio::test]
async fn fork_last_with_accepted_add_dirs_rejects_before_app_server_startup() {
    let temp = tempdir().expect("tempdir");
    let working_dir = temp.path().join("working-dir");
    let extra_dir = working_dir.join("docs");
    std::fs::create_dir_all(&extra_dir).expect("create add-dir target");
    let request_log = request_log_file();

    let backend = CodexBackend::new(CodexBackendConfig {
        binary: Some(fake_codex_app_server_binary()),
        env: [
            (
                "FAKE_CODEX_APP_SERVER_SCENARIO".to_string(),
                "fork_last_empty".to_string(),
            ),
            (
                "FAKE_CODEX_APP_SERVER_REQUEST_LOG".to_string(),
                request_log.path().display().to_string(),
            ),
        ]
        .into_iter()
        .collect(),
        ..Default::default()
    });

    let err = backend
        .run(AgentWrapperRunRequest {
            prompt: "hello world".to_string(),
            working_dir: Some(working_dir),
            extensions: [
                (
                    "agent_api.session.fork.v1".to_string(),
                    json!({"selector":"last"}),
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
async fn fork_last_invalid_add_dirs_beats_fork_rejection() {
    let temp = tempdir().expect("tempdir");
    let working_dir = temp.path().join("working-dir");
    std::fs::create_dir_all(&working_dir).expect("create working dir");
    let request_log = request_log_file();

    let err = CodexBackend::new(CodexBackendConfig {
        binary: Some(fake_codex_app_server_binary()),
        env: [
            (
                "FAKE_CODEX_APP_SERVER_SCENARIO".to_string(),
                "fork_last_empty".to_string(),
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
                json!({"selector":"last"}),
            ),
            (
                "agent_api.exec.add_dirs.v1".to_string(),
                json!({"dirs":"not-an-array"}),
            ),
        ]
        .into_iter()
        .collect(),
        ..Default::default()
    })
    .await
    .expect_err("invalid add-dirs should fail during validation");

    match err {
        AgentWrapperError::InvalidRequest { message } => {
            assert_eq!(message, "invalid agent_api.exec.add_dirs.v1.dirs")
        }
        other => panic!("expected InvalidRequest, got: {other:?}"),
    }

    assert!(
        read_logged_request_methods(&request_log).is_empty(),
        "expected no app-server JSON-RPC traffic on invalid-input path"
    );
}
