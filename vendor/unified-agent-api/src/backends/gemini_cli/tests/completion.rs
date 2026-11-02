use std::{
    collections::BTreeMap,
    task::{Context, Poll},
};

use futures_util::{task::noop_waker, StreamExt};
use serde_json::json;

use crate::{AgentWrapperBackend, AgentWrapperEventKind, AgentWrapperRunRequest};

use super::support::{backend_with_env, capture_json, request};

#[tokio::test]
async fn gemini_backend_maps_request_into_headless_argv_and_working_dir() {
    let capture_dir = tempfile::tempdir().expect("create capture dir");
    let capture_path = capture_dir.path().join("capture.json");
    let working_dir = tempfile::tempdir().expect("create working dir");
    let mut env = BTreeMap::new();
    env.insert(
        "FAKE_GEMINI_SCENARIO".to_string(),
        "capture_args".to_string(),
    );
    env.insert(
        "FAKE_GEMINI_CAPTURE".to_string(),
        capture_path.display().to_string(),
    );

    let backend = backend_with_env(env);
    let handle = backend
        .run(AgentWrapperRunRequest {
            prompt: "Reply with OK.".to_string(),
            working_dir: Some(working_dir.path().to_path_buf()),
            extensions: [(
                "agent_api.config.model.v1".to_string(),
                json!("gemini-2.5-flash"),
            )]
            .into_iter()
            .collect(),
            ..Default::default()
        })
        .await
        .expect("run should start");

    let mut events = handle.events;
    while events.next().await.is_some() {}

    let completion = handle.completion.await.expect("completion succeeds");
    assert!(completion.status.success());
    assert_eq!(completion.final_text.as_deref(), Some("OK"));
    assert_eq!(
        completion
            .data
            .as_ref()
            .and_then(|data| data.get("session"))
            .and_then(|session| session.get("id"))
            .and_then(|id| id.as_str()),
        Some("session-test")
    );

    let capture = capture_json(&capture_path);
    let argv = capture["argv"].as_array().expect("argv array");
    let argv = argv
        .iter()
        .map(|value| value.as_str().expect("argv string"))
        .collect::<Vec<_>>();

    assert_eq!(
        argv,
        vec![
            "--prompt",
            "Reply with OK.",
            "--output-format",
            "stream-json",
            "--model",
            "gemini-2.5-flash",
        ]
    );
    let expected_cwd = std::fs::canonicalize(working_dir.path())
        .expect("canonical working dir")
        .display()
        .to_string();
    assert_eq!(capture["cwd"].as_str(), Some(expected_cwd.as_str()));
}

#[tokio::test]
async fn gemini_backend_maps_live_tool_and_text_events() {
    let mut env = BTreeMap::new();
    env.insert(
        "FAKE_GEMINI_SCENARIO".to_string(),
        "tool_roundtrip".to_string(),
    );

    let backend = backend_with_env(env);
    let handle = backend
        .run(request("Run pwd.", None))
        .await
        .expect("run should start");

    let events = handle.events.collect::<Vec<_>>().await;
    assert_eq!(events[0].kind, AgentWrapperEventKind::Status);
    assert_eq!(events[1].kind, AgentWrapperEventKind::ToolCall);
    assert_eq!(events[2].kind, AgentWrapperEventKind::ToolResult);
    assert_eq!(events[3].kind, AgentWrapperEventKind::TextOutput);
    assert_eq!(events[4].kind, AgentWrapperEventKind::Status);

    assert_eq!(events[3].text.as_deref(), Some("done"));
    assert_eq!(
        events[1]
            .data
            .as_ref()
            .and_then(|data| data.get("tool_name"))
            .and_then(|value| value.as_str()),
        Some("run_shell_command")
    );
    assert_eq!(
        events[2]
            .data
            .as_ref()
            .and_then(|data| data.get("status"))
            .and_then(|value| value.as_str()),
        Some("success")
    );

    let completion = handle.completion.await.expect("completion succeeds");
    assert_eq!(completion.final_text.as_deref(), Some("done"));
}

#[tokio::test]
async fn gemini_backend_redacts_parse_errors_in_public_events() {
    let mut env = BTreeMap::new();
    env.insert(
        "FAKE_GEMINI_SCENARIO".to_string(),
        "parse_error_redaction".to_string(),
    );

    let backend = backend_with_env(env);
    let handle = backend
        .run(request("Reply with OK.", None))
        .await
        .expect("run should start");

    let mut saw_error = false;
    let mut events = handle.events;
    while let Some(event) = events.next().await {
        if event.kind == AgentWrapperEventKind::Error {
            let message = event.message.as_deref().unwrap_or("");
            assert!(!message.contains("VERY_SECRET_SHOULD_NOT_APPEAR"));
            saw_error = true;
        }
    }

    assert!(saw_error, "expected a surfaced parse error event");

    let completion = handle.completion.await.expect("completion succeeds");
    assert!(completion.status.success());
}

#[tokio::test]
async fn gemini_backend_completion_waits_for_stream_finality_via_shared_harness() {
    let mut env = BTreeMap::new();
    env.insert(
        "FAKE_GEMINI_SCENARIO".to_string(),
        "three_events_delayed".to_string(),
    );

    let backend = backend_with_env(env);
    let handle = backend
        .run(request("Reply with OK.", None))
        .await
        .expect("run should start");

    let mut events = handle.events;
    let mut completion = handle.completion;

    let first = tokio::select! {
        biased;
        result = completion.as_mut() => panic!("completion resolved before first event: {result:?}"),
        item = events.next() => item.expect("event stream open"),
    };

    assert_eq!(first.kind, AgentWrapperEventKind::Status);

    let waker = noop_waker();
    let mut cx = Context::from_waker(&waker);
    assert!(matches!(completion.as_mut().poll(&mut cx), Poll::Pending));

    while events.next().await.is_some() {}

    let completion = completion.await.expect("completion succeeds");
    assert!(completion.status.success());
    assert_eq!(completion.final_text.as_deref(), Some("OK"));
}
