use std::{collections::BTreeMap, path::PathBuf, time::Duration};

use futures_util::StreamExt;
use serde_json::json;

use crate::{backends::opencode::OpencodeBackendConfig, AgentWrapperBackend, AgentWrapperError};

use super::support::{backend_with_config, backend_with_env, backend_with_timeout, request};

#[tokio::test]
async fn opencode_backend_validation_uses_fake_binary_timeout_path_without_live_provider_state() {
    let mut env = BTreeMap::new();
    env.insert(
        "FAKE_OPENCODE_SCENARIO".to_string(),
        "slow_until_killed".to_string(),
    );

    let backend = backend_with_timeout(env, Duration::from_secs(1));
    let handle = backend
        .run(request("Reply with OK.", None))
        .await
        .expect("run should start");

    let mut events = handle.events;
    assert!(
        tokio::time::timeout(Duration::from_secs(2), events.next())
            .await
            .expect("first event should arrive before the harness timeout path completes")
            .is_some(),
        "fake-binary validation path should still surface an initial event"
    );
    while events.next().await.is_some() {}

    let err = handle
        .completion
        .await
        .expect_err("timeout validation path should surface a backend error");
    match err {
        AgentWrapperError::Backend { message } => {
            assert_eq!(message, "opencode backend error: timeout");
        }
        other => panic!("expected Backend timeout error, got {other:?}"),
    }
}

#[tokio::test]
async fn opencode_backend_missing_binary_surfaces_safe_spawn_error() {
    let backend = backend_with_config(OpencodeBackendConfig {
        binary: Some(PathBuf::from("/definitely/not/a/real/opencode-binary")),
        default_timeout: None,
        env: BTreeMap::new(),
    });

    let handle = backend
        .run(request("Reply with OK.", None))
        .await
        .expect("run handle should still be returned before startup failure surfaces");

    let mut events = handle.events;
    let first = events.next().await.expect("error event should surface");
    assert_eq!(first.kind, crate::AgentWrapperEventKind::Error);
    assert_eq!(
        first.message.as_deref(),
        Some("opencode backend error: binary not found")
    );
    assert!(
        events.next().await.is_none(),
        "startup failure should close the stream"
    );

    let err = handle
        .completion
        .await
        .expect_err("startup failure should resolve completion as backend error");
    match err {
        AgentWrapperError::Backend { message } => {
            assert_eq!(message, "opencode backend error: binary not found");
        }
        other => panic!("expected Backend spawn error, got {other:?}"),
    }
}

#[tokio::test]
async fn opencode_backend_rejects_mutually_exclusive_resume_and_fork_before_spawn() {
    let backend = backend_with_config(OpencodeBackendConfig {
        binary: Some(PathBuf::from("/definitely/not/a/real/opencode-binary")),
        default_timeout: None,
        env: BTreeMap::new(),
    });
    let mut run_request = request("Reply with OK.", None);
    run_request.extensions.insert(
        "agent_api.session.resume.v1".to_string(),
        json!({"selector": "last"}),
    );
    run_request.extensions.insert(
        "agent_api.session.fork.v1".to_string(),
        json!({"selector": "id", "id": "session-123"}),
    );

    let err = backend
        .run(run_request)
        .await
        .expect_err("mutually exclusive selectors must fail before spawn");
    match err {
        AgentWrapperError::InvalidRequest { message } => assert_eq!(
            message,
            "agent_api.session.resume.v1 and agent_api.session.fork.v1 are mutually exclusive"
        ),
        other => panic!("expected InvalidRequest, got {other:?}"),
    }
}

#[tokio::test]
async fn opencode_backend_keeps_add_dirs_fail_closed() {
    let backend = backend_with_env(Default::default());
    let mut run_request = request("Reply with OK.", None);
    run_request.extensions.insert(
        "agent_api.exec.add_dirs.v1".to_string(),
        json!({"dirs": ["."]}),
    );

    let err = backend
        .run(run_request)
        .await
        .expect_err("add_dirs must remain unsupported for opencode");
    match err {
        AgentWrapperError::UnsupportedCapability {
            agent_kind,
            capability,
        } => {
            assert_eq!(agent_kind, "opencode");
            assert_eq!(capability, "agent_api.exec.add_dirs.v1");
        }
        other => panic!("expected UnsupportedCapability, got {other:?}"),
    }
}
