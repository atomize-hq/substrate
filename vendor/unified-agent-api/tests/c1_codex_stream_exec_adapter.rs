#![cfg(feature = "codex")]

use std::{
    path::PathBuf,
    pin::Pin,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use agent_api::{
    backends::codex::{CodexBackend, CodexBackendConfig},
    AgentWrapperBackend, AgentWrapperError, AgentWrapperEvent, AgentWrapperEventKind,
    AgentWrapperRunRequest,
};
use futures_core::Stream;
use serde_json::Value;
use tempfile::tempdir;

const ADD_DIRS_RUNTIME_REJECTION_MESSAGE: &str = "add_dirs rejected by runtime";
const ADD_DIR_LEAK_SENTINELS: [&str; 3] = [
    "ADD_DIR_RAW_PATH_SECRET",
    "ADD_DIR_STDOUT_SECRET",
    "ADD_DIR_STDERR_SECRET",
];
const BACKPRESSURE_ASSERT_TIMEOUT: Duration = Duration::from_millis(200);
const POST_DROP_PENDING_TIMEOUT: Duration = Duration::from_millis(50);
const POST_DROP_COMPLETION_TIMEOUT: Duration = Duration::from_secs(5);

async fn drain_to_none(
    mut stream: Pin<&mut (dyn Stream<Item = AgentWrapperEvent> + Send)>,
    timeout: Duration,
) -> Vec<AgentWrapperEvent> {
    let mut out = Vec::new();
    let deadline = tokio::time::sleep(timeout);
    tokio::pin!(deadline);

    loop {
        tokio::select! {
            _ = &mut deadline => break,
            item = std::future::poll_fn(|cx| stream.as_mut().poll_next(cx)) => {
                match item {
                    Some(ev) => out.push(ev),
                    None => break,
                }
            }
        }
    }

    out
}

fn fake_codex_binary() -> PathBuf {
    PathBuf::from(env!(
        "CARGO_BIN_EXE_fake_codex_stream_exec_scenarios_agent_api"
    ))
}

fn unique_missing_dir_path(test_name: &str) -> PathBuf {
    let pid = std::process::id();
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time after unix epoch")
        .as_nanos();
    std::env::temp_dir().join(format!("{test_name}_{pid}_{nanos}_missing"))
}

fn any_event_contains(events: &[AgentWrapperEvent], needle: &str) -> bool {
    events.iter().any(|ev| {
        ev.message
            .as_deref()
            .is_some_and(|message| message.contains(needle))
            || ev.text.as_deref().is_some_and(|text| text.contains(needle))
            || ev
                .data
                .as_ref()
                .and_then(|data| serde_json::to_string(data).ok())
                .is_some_and(|data| data.contains(needle))
    })
}

fn find_first_kind(events: &[AgentWrapperEvent], kind: AgentWrapperEventKind) -> Option<usize> {
    events.iter().position(|ev| ev.kind == kind)
}

fn handle_facet_index(events: &[AgentWrapperEvent]) -> Option<usize> {
    events.iter().position(|event| {
        event.kind == AgentWrapperEventKind::Status
            && event
                .data
                .as_ref()
                .and_then(|data| data.get("schema"))
                .and_then(serde_json::Value::as_str)
                == Some("agent_api.session.handle.v1")
    })
}

fn assert_no_add_dir_sentinel_leaks_in_events(events: &[AgentWrapperEvent]) {
    for sentinel in ADD_DIR_LEAK_SENTINELS {
        assert!(
            !any_event_contains(events, sentinel),
            "expected add-dir runtime rejection sentinel {sentinel} to stay backend-private"
        );
    }
}

fn tool_schema(event: &AgentWrapperEvent) -> Option<&str> {
    event
        .data
        .as_ref()
        .and_then(|data| data.get("schema"))
        .and_then(Value::as_str)
}

fn tool_field<'a>(event: &'a AgentWrapperEvent, field: &str) -> Option<&'a Value> {
    event
        .data
        .as_ref()
        .and_then(|data| data.get("tool"))
        .and_then(|tool| tool.get(field))
}

#[path = "c1_codex_stream_exec_adapter/env_tests.rs"]
mod env_tests;
#[path = "c1_codex_stream_exec_adapter/tool_lifecycle_tests.rs"]
mod tool_lifecycle_tests;

#[tokio::test]
async fn empty_prompt_is_rejected_before_spawning() {
    let backend = CodexBackend::new(CodexBackendConfig::default());
    let err = backend
        .run(AgentWrapperRunRequest {
            prompt: "   ".to_string(),
            ..Default::default()
        })
        .await
        .unwrap_err();
    assert!(matches!(err, AgentWrapperError::InvalidRequest { .. }));
}

#[tokio::test]
async fn unknown_extension_key_is_rejected_fail_closed() {
    let backend = CodexBackend::new(CodexBackendConfig::default());
    let err = backend
        .run(AgentWrapperRunRequest {
            prompt: "hello".to_string(),
            extensions: [(
                "backend.codex.exec.unknown_key".to_string(),
                serde_json::Value::Bool(true),
            )]
            .into_iter()
            .collect(),
            ..Default::default()
        })
        .await
        .unwrap_err();
    assert!(matches!(
        err,
        AgentWrapperError::UnsupportedCapability { .. }
    ));
}

#[tokio::test]
async fn extension_types_are_validated() {
    let backend = CodexBackend::new(CodexBackendConfig::default());
    let err = backend
        .run(AgentWrapperRunRequest {
            prompt: "hello".to_string(),
            extensions: [(
                "agent_api.exec.non_interactive".to_string(),
                serde_json::Value::String("true".to_string()),
            )]
            .into_iter()
            .collect(),
            ..Default::default()
        })
        .await
        .unwrap_err();
    assert!(matches!(err, AgentWrapperError::InvalidRequest { .. }));
}

#[tokio::test]
async fn non_interactive_true_rejects_contradictory_approval_policy() {
    let backend = CodexBackend::new(CodexBackendConfig::default());
    let err = backend
        .run(AgentWrapperRunRequest {
            prompt: "hello".to_string(),
            extensions: [
                (
                    "agent_api.exec.non_interactive".to_string(),
                    serde_json::Value::Bool(true),
                ),
                (
                    "backend.codex.exec.approval_policy".to_string(),
                    serde_json::Value::String("untrusted".to_string()),
                ),
            ]
            .into_iter()
            .collect(),
            ..Default::default()
        })
        .await
        .unwrap_err();
    assert!(matches!(err, AgentWrapperError::InvalidRequest { .. }));
}

#[tokio::test]
async fn parse_errors_do_not_leak_raw_lines_and_stream_continues() {
    let backend = CodexBackend::new(CodexBackendConfig {
        binary: Some(fake_codex_binary()),
        env: [(
            "FAKE_CODEX_SCENARIO".to_string(),
            "parse_error_midstream".to_string(),
        )]
        .into_iter()
        .collect(),
        ..Default::default()
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

    let seen = drain_to_none(events.as_mut(), Duration::from_secs(2)).await;
    assert!(
        find_first_kind(&seen, AgentWrapperEventKind::Error).is_some(),
        "expected an Error event for the parse failure"
    );
    assert!(
        !any_event_contains(&seen, "RAW-LINE-SECRET-PARSE"),
        "expected redaction to avoid raw JSONL line content"
    );

    let first_error = find_first_kind(&seen, AgentWrapperEventKind::Error).unwrap();
    assert!(
        seen.iter()
            .skip(first_error + 1)
            .any(|ev| ev.kind == AgentWrapperEventKind::Status),
        "expected the stream to continue after a per-line error"
    );

    let completion = tokio::time::timeout(Duration::from_secs(2), completion)
        .await
        .expect("completion resolves")
        .unwrap();
    assert!(completion.status.success());
}

#[tokio::test]
async fn normalize_errors_do_not_leak_raw_lines_and_stream_continues() {
    let backend = CodexBackend::new(CodexBackendConfig {
        binary: Some(fake_codex_binary()),
        env: [(
            "FAKE_CODEX_SCENARIO".to_string(),
            "normalize_error_midstream".to_string(),
        )]
        .into_iter()
        .collect(),
        ..Default::default()
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

    let seen = drain_to_none(events.as_mut(), Duration::from_secs(2)).await;
    assert!(
        find_first_kind(&seen, AgentWrapperEventKind::Error).is_some(),
        "expected an Error event for the normalize failure"
    );
    assert!(
        !any_event_contains(&seen, "RAW-LINE-SECRET-NORM"),
        "expected redaction to avoid raw JSONL line content"
    );

    let first_error = find_first_kind(&seen, AgentWrapperEventKind::Error).unwrap();
    assert!(
        seen.iter()
            .skip(first_error + 1)
            .any(|ev| ev.kind == AgentWrapperEventKind::Status),
        "expected the stream to continue after a per-line error"
    );

    let completion = tokio::time::timeout(Duration::from_secs(2), completion)
        .await
        .expect("completion resolves")
        .unwrap();
    assert!(completion.status.success());
}

#[tokio::test]
async fn nonzero_exit_is_redacted_and_completion_is_ok_with_nonzero_status() {
    let backend = CodexBackend::new(CodexBackendConfig {
        binary: Some(fake_codex_binary()),
        env: [(
            "FAKE_CODEX_SCENARIO".to_string(),
            "nonzero_exit".to_string(),
        )]
        .into_iter()
        .collect(),
        ..Default::default()
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

    let seen = drain_to_none(events.as_mut(), Duration::from_secs(2)).await;
    assert!(
        find_first_kind(&seen, AgentWrapperEventKind::Error).is_some(),
        "expected an Error event for the non-zero exit"
    );
    assert!(
        !any_event_contains(&seen, "RAW-STDERR-SECRET"),
        "expected stderr redaction on non-zero exit"
    );

    let completion = tokio::time::timeout(Duration::from_secs(2), completion)
        .await
        .expect("completion resolves")
        .unwrap();
    assert!(!completion.status.success());
    assert_eq!(completion.final_text, None);
}

#[tokio::test]
async fn short_model_ids_do_not_reclassify_transport_failures_as_runtime_rejection() {
    for requested_model in ["a", "1"] {
        let backend = CodexBackend::new(CodexBackendConfig {
            binary: Some(fake_codex_binary()),
            env: [
                (
                    "FAKE_CODEX_SCENARIO".to_string(),
                    "model_substring_transport_error_after_thread_started".to_string(),
                ),
                (
                    "FAKE_CODEX_EXPECT_SANDBOX".to_string(),
                    "workspace-write".to_string(),
                ),
                (
                    "FAKE_CODEX_EXPECT_APPROVAL".to_string(),
                    "never".to_string(),
                ),
                (
                    "FAKE_CODEX_EXPECT_MODEL".to_string(),
                    requested_model.to_string(),
                ),
            ]
            .into_iter()
            .collect(),
            ..Default::default()
        });

        let handle = backend
            .run(AgentWrapperRunRequest {
                prompt: "hello".to_string(),
                extensions: [(
                    "agent_api.config.model.v1".to_string(),
                    serde_json::Value::String(requested_model.to_string()),
                )]
                .into_iter()
                .collect(),
                ..Default::default()
            })
            .await
            .expect("spawn succeeds");

        let mut events = handle.events;
        let completion = handle.completion;

        let seen = drain_to_none(events.as_mut(), Duration::from_secs(2)).await;
        assert!(
            any_event_contains(
                &seen,
                &format!("transport failure while routing request for model {requested_model}")
            ),
            "expected original transport error event for model {requested_model}; events: {seen:?}"
        );
        assert!(
            !any_event_contains(&seen, "model rejected by runtime"),
            "unexpected runtime rejection remap for model {requested_model}; events: {seen:?}"
        );
        assert_eq!(
            seen.last().map(|event| event.kind.clone()),
            Some(AgentWrapperEventKind::Error),
            "expected terminal error event for model {requested_model}; events: {seen:?}"
        );
        assert!(
            seen.last()
                .and_then(|event| event.message.as_deref())
                .is_some_and(|message| message.starts_with("codex exited non-zero:")),
            "expected ordinary non-zero-exit terminal error for model {requested_model}; events: {seen:?}"
        );

        let completion = tokio::time::timeout(Duration::from_secs(2), completion)
            .await
            .expect("completion resolves")
            .expect("non-zero exit still resolves completion");
        assert!(
            !completion.status.success(),
            "expected non-zero completion for model {requested_model}"
        );
        assert_eq!(completion.final_text, None);
    }
}

#[tokio::test]
async fn dropping_events_unblocks_buffered_model_runtime_rejection_completion() {
    let requested_model = Some("gpt-5-codex");
    let secret = "MODEL_RUNTIME_REJECTION_SECRET_DO_NOT_LEAK";
    let effective_model = "gpt-5-codex";

    let backend = CodexBackend::new(CodexBackendConfig {
        binary: Some(fake_codex_binary()),
        env: [
            (
                "FAKE_CODEX_SCENARIO".to_string(),
                "model_runtime_rejection_after_buffered_events".to_string(),
            ),
            (
                "FAKE_CODEX_EXPECT_SANDBOX".to_string(),
                "workspace-write".to_string(),
            ),
            (
                "FAKE_CODEX_EXPECT_APPROVAL".to_string(),
                "never".to_string(),
            ),
            (
                "FAKE_CODEX_EXPECT_MODEL".to_string(),
                effective_model.to_string(),
            ),
            (
                "FAKE_CODEX_MODEL_RUNTIME_REJECTION_SECRET".to_string(),
                secret.to_string(),
            ),
            (
                "FAKE_CODEX_BUFFERED_EVENT_COUNT".to_string(),
                "4096".to_string(),
            ),
            (
                "FAKE_CODEX_BUFFERED_EVENT_PADDING_BYTES".to_string(),
                "2048".to_string(),
            ),
            (
                "FAKE_CODEX_BUFFERED_EVENT_SLEEP_EVERY".to_string(),
                "8".to_string(),
            ),
            (
                "FAKE_CODEX_BUFFERED_EVENT_SLEEP_MILLIS".to_string(),
                "1".to_string(),
            ),
            (
                "FAKE_CODEX_RUNTIME_REJECTION_EXIT_CODE".to_string(),
                "0".to_string(),
            ),
        ]
        .into_iter()
        .collect(),
        ..Default::default()
    });

    let handle = backend
        .run(AgentWrapperRunRequest {
            prompt: "hello".to_string(),
            extensions: requested_model
                .map(|model| {
                    [(
                        "agent_api.config.model.v1".to_string(),
                        Value::String(model.to_string()),
                    )]
                    .into_iter()
                    .collect()
                })
                .unwrap_or_default(),
            ..Default::default()
        })
        .await
        .expect("spawn succeeds");

    let mut events = handle.events;
    let mut completion = handle.completion;

    let first = tokio::time::timeout(
        Duration::from_secs(1),
        std::future::poll_fn(|cx| events.as_mut().poll_next(cx)),
    )
    .await
    .expect("first event arrives");
    assert!(
        first.is_some(),
        "expected at least one live event before drop"
    );

    assert!(
        tokio::time::timeout(BACKPRESSURE_ASSERT_TIMEOUT, &mut completion)
            .await
            .is_err(),
        "completion should remain pending while events are still attached"
    );

    drop(events);

    assert!(
        tokio::time::timeout(POST_DROP_PENDING_TIMEOUT, &mut completion)
            .await
            .is_err(),
        "completion should remain pending until buffered suppressed model errors are classified"
    );

    let err = tokio::time::timeout(POST_DROP_COMPLETION_TIMEOUT, completion)
        .await
        .expect("completion resolves after dropping events")
        .unwrap_err();
    match err {
        AgentWrapperError::Backend { message } => {
            assert_eq!(
                message,
                "codex backend error: model rejected by runtime (details redacted)"
            );
            assert!(!message.contains(secret));
            assert!(!message.contains(effective_model));
        }
        other => panic!("expected Backend error, got: {other:?}"),
    }
}

#[tokio::test]
async fn dropping_events_unblocks_buffered_config_model_runtime_rejection_completion() {
    let requested_model: Option<&str> = None;
    let secret = "MODEL_RUNTIME_REJECTION_SECRET_DO_NOT_LEAK";
    let effective_model = "gpt-5-codex-from-config";

    let backend = CodexBackend::new(CodexBackendConfig {
        binary: Some(fake_codex_binary()),
        model: Some(effective_model.to_string()),
        env: [
            (
                "FAKE_CODEX_SCENARIO".to_string(),
                "model_runtime_rejection_after_buffered_events".to_string(),
            ),
            (
                "FAKE_CODEX_EXPECT_SANDBOX".to_string(),
                "workspace-write".to_string(),
            ),
            (
                "FAKE_CODEX_EXPECT_APPROVAL".to_string(),
                "never".to_string(),
            ),
            (
                "FAKE_CODEX_EXPECT_MODEL".to_string(),
                effective_model.to_string(),
            ),
            (
                "FAKE_CODEX_MODEL_RUNTIME_REJECTION_SECRET".to_string(),
                secret.to_string(),
            ),
            (
                "FAKE_CODEX_BUFFERED_EVENT_COUNT".to_string(),
                "4096".to_string(),
            ),
            (
                "FAKE_CODEX_BUFFERED_EVENT_PADDING_BYTES".to_string(),
                "2048".to_string(),
            ),
            (
                "FAKE_CODEX_BUFFERED_EVENT_SLEEP_EVERY".to_string(),
                "8".to_string(),
            ),
            (
                "FAKE_CODEX_BUFFERED_EVENT_SLEEP_MILLIS".to_string(),
                "1".to_string(),
            ),
            (
                "FAKE_CODEX_RUNTIME_REJECTION_EXIT_CODE".to_string(),
                "0".to_string(),
            ),
        ]
        .into_iter()
        .collect(),
        ..Default::default()
    });

    let handle = backend
        .run(AgentWrapperRunRequest {
            prompt: "hello".to_string(),
            extensions: requested_model
                .map(|model| {
                    [(
                        "agent_api.config.model.v1".to_string(),
                        Value::String(model.to_string()),
                    )]
                    .into_iter()
                    .collect()
                })
                .unwrap_or_default(),
            ..Default::default()
        })
        .await
        .expect("spawn succeeds");

    let mut events = handle.events;
    let mut completion = handle.completion;

    let first = tokio::time::timeout(
        Duration::from_secs(1),
        std::future::poll_fn(|cx| events.as_mut().poll_next(cx)),
    )
    .await
    .expect("first event arrives");
    assert!(
        first.is_some(),
        "expected at least one live event before drop"
    );

    assert!(
        tokio::time::timeout(BACKPRESSURE_ASSERT_TIMEOUT, &mut completion)
            .await
            .is_err(),
        "completion should remain pending while events are still attached"
    );

    drop(events);

    assert!(
        tokio::time::timeout(POST_DROP_PENDING_TIMEOUT, &mut completion)
            .await
            .is_err(),
        "completion should remain pending until buffered suppressed model errors are classified"
    );

    let err = tokio::time::timeout(POST_DROP_COMPLETION_TIMEOUT, completion)
        .await
        .expect("completion resolves after dropping events")
        .unwrap_err();
    match err {
        AgentWrapperError::Backend { message } => {
            assert_eq!(
                message,
                "codex backend error: model rejected by runtime (details redacted)"
            );
            assert!(!message.contains(secret));
            assert!(!message.contains(effective_model));
        }
        other => panic!("expected Backend error, got: {other:?}"),
    }
}

#[tokio::test]
async fn exec_add_dirs_runtime_rejection_emits_single_terminal_error_and_no_leaks() {
    let temp = tempdir().expect("tempdir");
    let dir_a = temp.path().join("alpha");
    let dir_b = temp.path().join("beta");
    std::fs::create_dir_all(&dir_a).expect("alpha dir");
    std::fs::create_dir_all(&dir_b).expect("beta dir");

    let backend = CodexBackend::new(CodexBackendConfig {
        binary: Some(fake_codex_binary()),
        env: [(
            "FAKE_CODEX_SCENARIO".to_string(),
            "add_dirs_runtime_rejection_exec".to_string(),
        )]
        .into_iter()
        .collect(),
        ..Default::default()
    });

    let handle = backend
        .run(AgentWrapperRunRequest {
            prompt: "hello".to_string(),
            extensions: [(
                "agent_api.exec.add_dirs.v1".to_string(),
                serde_json::json!({
                    "dirs": [
                        dir_a.display().to_string(),
                        dir_b.display().to_string(),
                    ]
                }),
            )]
            .into_iter()
            .collect(),
            ..Default::default()
        })
        .await
        .expect("expected handle before runtime rejection");

    let mut events = handle.events;
    let completion = handle.completion;

    let seen = drain_to_none(events.as_mut(), Duration::from_secs(2)).await;
    let handle_idx = handle_facet_index(&seen).expect("expected session handle facet");
    let error_indices: Vec<_> = seen
        .iter()
        .enumerate()
        .filter_map(|(idx, event)| (event.kind == AgentWrapperEventKind::Error).then_some(idx))
        .collect();
    assert_eq!(error_indices.len(), 1, "expected exactly one Error event");
    let error_idx = error_indices[0];
    assert!(
        handle_idx < error_idx,
        "expected thread.started-derived handle facet before backend error"
    );
    assert_eq!(
        seen.last().map(|event| event.kind.clone()),
        Some(AgentWrapperEventKind::Error),
        "expected Error event to be terminal"
    );
    assert_eq!(
        seen[error_idx].message.as_deref(),
        Some(ADD_DIRS_RUNTIME_REJECTION_MESSAGE)
    );
    assert_no_add_dir_sentinel_leaks_in_events(&seen);

    let err = tokio::time::timeout(Duration::from_secs(2), completion)
        .await
        .expect("completion resolves")
        .expect_err("expected runtime rejection completion error");
    match err {
        AgentWrapperError::Backend { message } => {
            assert_eq!(message, ADD_DIRS_RUNTIME_REJECTION_MESSAGE);
            for sentinel in ADD_DIR_LEAK_SENTINELS {
                assert!(
                    !message.contains(sentinel),
                    "expected add-dir runtime rejection sentinel {sentinel} to stay out of completion error"
                );
            }
        }
        other => panic!("expected Backend error, got: {other:?}"),
    }
}
