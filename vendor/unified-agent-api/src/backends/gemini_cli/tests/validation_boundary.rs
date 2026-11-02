use std::{collections::BTreeMap, path::PathBuf, time::Duration};

use futures_util::StreamExt;
use serde_json::json;

use crate::{backends::gemini_cli::GeminiCliBackendConfig, AgentWrapperBackend, AgentWrapperError};

use super::support::{backend_with_config, backend_with_env, backend_with_timeout, request};

#[tokio::test]
async fn gemini_backend_validation_uses_fake_binary_timeout_path_without_live_provider_state() {
    let mut env = BTreeMap::new();
    env.insert(
        "FAKE_GEMINI_SCENARIO".to_string(),
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
            assert_eq!(message, "gemini backend error: timeout");
        }
        other => panic!("expected Backend timeout error, got {other:?}"),
    }
}

#[tokio::test]
async fn gemini_backend_missing_binary_surfaces_safe_spawn_error() {
    let backend = backend_with_config(GeminiCliBackendConfig {
        binary: Some(PathBuf::from("/definitely/not/a/real/gemini-binary")),
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
        Some("gemini backend error: binary not found")
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
            assert_eq!(message, "gemini backend error: binary not found");
        }
        other => panic!("expected Backend spawn error, got {other:?}"),
    }
}

#[tokio::test]
async fn gemini_backend_invalid_input_exit_code_is_translated_conservatively() {
    let mut env = BTreeMap::new();
    env.insert(
        "FAKE_GEMINI_SCENARIO".to_string(),
        "invalid_input".to_string(),
    );

    let backend = backend_with_env(env);
    let handle = backend
        .run(request("Reply with OK.", None))
        .await
        .expect("run should start");

    let mut events = handle.events;
    while events.next().await.is_some() {}

    let err = handle
        .completion
        .await
        .expect_err("invalid input should surface a backend error");
    match err {
        AgentWrapperError::Backend { message } => {
            assert_eq!(message, "gemini backend error: invalid input");
        }
        other => panic!("expected Backend invalid input error, got {other:?}"),
    }
}

#[tokio::test]
async fn gemini_backend_keeps_unknown_backend_namespace_fail_closed() {
    let backend = backend_with_env(Default::default());
    let mut run_request = request("Reply with OK.", None);
    run_request
        .extensions
        .insert("backend.gemini_cli.future_key".to_string(), json!(true));

    let err = backend
        .run(run_request)
        .await
        .expect_err("unsupported backend namespace key must fail closed before spawn");
    match err {
        AgentWrapperError::UnsupportedCapability {
            agent_kind,
            capability,
        } => {
            assert_eq!(agent_kind, "gemini_cli");
            assert_eq!(capability, "backend.gemini_cli.future_key");
        }
        other => panic!("expected UnsupportedCapability, got {other:?}"),
    }
}
