use agent_api::{AgentWrapperBackend, AgentWrapperError, AgentWrapperEventKind};
use serde_json::json;

use super::support::{
    add_dir_expectations, add_dirs_extension, add_dirs_fixture,
    assert_no_add_dir_sentinel_leaks_in_events, base_env, build_backend, drain_to_none,
    handle_facet_index, run_request, ADD_DIRS_RUNTIME_REJECTION_MESSAGE, ADD_DIR_LEAK_SENTINELS,
    STREAM_TIMEOUT,
};

#[cfg(unix)]
use super::support::{build_probe_only_backend, AddDirProbeMode};

const BACKPRESSURE_ASSERT_TIMEOUT: std::time::Duration = std::time::Duration::from_millis(200);
const POST_DROP_PENDING_TIMEOUT: std::time::Duration = std::time::Duration::from_millis(50);

async fn assert_runtime_rejection_case(
    scenario: &str,
    resume_extension: (String, serde_json::Value),
    prompt: &str,
    extra_env: impl IntoIterator<Item = (String, String)>,
) {
    let fixture = add_dirs_fixture();
    let mut env = base_env();
    env.insert("FAKE_CODEX_SCENARIO".to_string(), scenario.to_string());
    env.insert("FAKE_CODEX_EXPECT_PROMPT".to_string(), prompt.to_string());
    env.extend(add_dir_expectations(&fixture.dirs));
    env.extend(extra_env);

    let backend = build_backend(env, None, false);
    let handle = backend
        .run(run_request(
            prompt,
            [add_dirs_extension(&fixture.dirs), resume_extension],
        ))
        .await
        .unwrap();
    let mut events = handle.events;
    let seen = drain_to_none(events.as_mut(), STREAM_TIMEOUT).await;
    assert_runtime_rejection_events(&seen);

    let err = tokio::time::timeout(STREAM_TIMEOUT, handle.completion)
        .await
        .expect("completion resolves")
        .unwrap_err();
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

async fn assert_dropped_events_unblock_completion(
    scenario: &str,
    prompt: &str,
    extensions: impl IntoIterator<Item = (String, serde_json::Value)>,
    extra_env: impl IntoIterator<Item = (String, String)>,
    expected_message: &'static str,
    assert_pending_after_drop: bool,
) {
    let mut env = base_env();
    env.insert("FAKE_CODEX_SCENARIO".to_string(), scenario.to_string());
    env.insert("FAKE_CODEX_EXPECT_PROMPT".to_string(), prompt.to_string());
    env.extend([
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
    ]);
    env.extend(extra_env);

    let backend = build_backend(env, None, false);
    let handle = backend.run(run_request(prompt, extensions)).await.unwrap();
    let mut events = handle.events;
    let mut completion = handle.completion;

    let first = tokio::time::timeout(
        STREAM_TIMEOUT,
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

    if assert_pending_after_drop {
        assert!(
            tokio::time::timeout(POST_DROP_PENDING_TIMEOUT, &mut completion)
                .await
                .is_err(),
            "completion should remain pending until buffered suppressed errors are classified"
        );
    }

    let err = tokio::time::timeout(STREAM_TIMEOUT, completion)
        .await
        .expect("completion resolves after dropping events")
        .unwrap_err();
    match err {
        AgentWrapperError::Backend { message } => assert_eq!(message, expected_message),
        other => panic!("expected Backend error, got: {other:?}"),
    }
}

fn assert_runtime_rejection_events(seen: &[agent_api::AgentWrapperEvent]) {
    let handle_idx = handle_facet_index(seen).expect("expected session handle facet");
    let error_indices: Vec<_> = seen
        .iter()
        .enumerate()
        .filter_map(|(idx, event)| (event.kind == AgentWrapperEventKind::Error).then_some(idx))
        .collect();
    assert_eq!(error_indices.len(), 1, "expected exactly one Error event");
    let error_idx = error_indices[0];
    assert!(
        handle_idx < error_idx,
        "expected thread.resumed-derived handle facet before backend error"
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
    assert_no_add_dir_sentinel_leaks_in_events(seen);
}

#[cfg(unix)]
async fn assert_preflight_rejection_case(
    mode: AddDirProbeMode,
    resume_extension: (String, serde_json::Value),
    extra_env: impl IntoIterator<Item = (String, String)>,
) {
    let fixture = add_dirs_fixture();
    let mut env = base_env();
    env.extend(extra_env);

    let fixture_backend = build_probe_only_backend(mode, env, None, false);
    let handle = tokio::time::timeout(
        std::time::Duration::from_millis(100),
        fixture_backend.backend.run(run_request(
            "hello world",
            [add_dirs_extension(&fixture.dirs), resume_extension],
        )),
    )
    .await
    .expect("preflight rejection should not block backend.run")
    .expect("preflight rejection should surface through the returned handle");

    let mut events = handle.events;
    let seen = drain_to_none(events.as_mut(), STREAM_TIMEOUT).await;
    let error_indices: Vec<_> = seen
        .iter()
        .enumerate()
        .filter_map(|(idx, event)| (event.kind == AgentWrapperEventKind::Error).then_some(idx))
        .collect();
    assert_eq!(error_indices.len(), 1, "expected exactly one Error event");
    let error_idx = error_indices[0];
    assert_eq!(
        seen[error_idx].message.as_deref(),
        Some(ADD_DIRS_RUNTIME_REJECTION_MESSAGE)
    );
    assert_eq!(
        seen.last().map(|event| event.kind.clone()),
        Some(AgentWrapperEventKind::Error),
        "expected Error event to be terminal"
    );
    assert_no_add_dir_sentinel_leaks_in_events(&seen);

    let err = tokio::time::timeout(STREAM_TIMEOUT, handle.completion)
        .await
        .expect("completion resolves")
        .unwrap_err();
    match err {
        AgentWrapperError::Backend { message } => {
            assert_eq!(message, ADD_DIRS_RUNTIME_REJECTION_MESSAGE);
        }
        other => panic!("expected Backend error, got: {other:?}"),
    }
    assert!(
        !fixture_backend.exec_log.exists(),
        "preflight rejection should not invoke codex exec"
    );
}

#[tokio::test]
async fn resume_last_add_dirs_runtime_rejection_emits_handle_before_backend_error() {
    assert_runtime_rejection_case(
        "add_dirs_runtime_rejection_resume_last",
        (
            "agent_api.session.resume.v1".to_string(),
            json!({"selector": "last"}),
        ),
        "hello world",
        std::iter::empty(),
    )
    .await;
}

#[tokio::test]
async fn resume_id_add_dirs_runtime_rejection_emits_handle_before_backend_error() {
    let resume_id = "thread-123";
    assert_runtime_rejection_case(
        "add_dirs_runtime_rejection_resume_id",
        (
            "agent_api.session.resume.v1".to_string(),
            json!({"selector": "id", "id": resume_id}),
        ),
        "hello world",
        [(
            "FAKE_CODEX_EXPECT_RESUME_ID".to_string(),
            resume_id.to_string(),
        )],
    )
    .await;
}

#[tokio::test]
async fn resume_last_selection_failure_completion_unblocks_after_dropping_events() {
    assert_dropped_events_unblock_completion(
        "resume_last_not_found_buffered_transport_errors",
        "hello world",
        [(
            "agent_api.session.resume.v1".to_string(),
            json!({"selector": "last"}),
        )],
        std::iter::empty(),
        "no session found",
        false,
    )
    .await;
}

#[tokio::test]
async fn resume_last_add_dirs_runtime_rejection_completion_unblocks_after_dropping_events() {
    let fixture = add_dirs_fixture();

    assert_dropped_events_unblock_completion(
        "add_dirs_runtime_rejection_resume_last_buffered_tail",
        "hello world",
        [
            add_dirs_extension(&fixture.dirs),
            (
                "agent_api.session.resume.v1".to_string(),
                json!({"selector": "last"}),
            ),
        ],
        add_dir_expectations(&fixture.dirs),
        ADD_DIRS_RUNTIME_REJECTION_MESSAGE,
        true,
    )
    .await;
}

#[cfg(unix)]
#[tokio::test]
async fn resume_last_add_dirs_preflight_rejects_when_probe_reports_unknown() {
    assert_preflight_rejection_case(
        AddDirProbeMode::Unknown,
        (
            "agent_api.session.resume.v1".to_string(),
            json!({"selector": "last"}),
        ),
        std::iter::empty(),
    )
    .await;
}

#[cfg(unix)]
#[tokio::test]
async fn resume_id_add_dirs_preflight_rejects_when_probe_reports_unknown() {
    let resume_id = "thread-123";
    assert_preflight_rejection_case(
        AddDirProbeMode::Unsupported,
        (
            "agent_api.session.resume.v1".to_string(),
            json!({"selector": "id", "id": resume_id}),
        ),
        [(
            "FAKE_CODEX_EXPECT_RESUME_ID".to_string(),
            resume_id.to_string(),
        )],
    )
    .await;
}

#[cfg(unix)]
#[tokio::test]
async fn resume_id_add_dirs_env_sensitive_probe_uses_request_env() {
    let fixture = add_dirs_fixture();
    let resume_id = "thread-123";
    let fixture_backend = build_probe_only_backend(
        AddDirProbeMode::EnvSensitiveSupported,
        base_env(),
        None,
        false,
    );
    let mut request = run_request(
        "hello world",
        [
            add_dirs_extension(&fixture.dirs),
            (
                "agent_api.session.resume.v1".to_string(),
                json!({"selector": "id", "id": resume_id}),
            ),
        ],
    );
    request.env.insert(
        "FAKE_CODEX_ENABLE_ADD_DIR_PROBE".to_string(),
        "1".to_string(),
    );

    let handle = fixture_backend
        .backend
        .run(request)
        .await
        .expect("env-sensitive probe should honor request env and return a handle");

    let mut events = handle.events;
    let seen = drain_to_none(events.as_mut(), STREAM_TIMEOUT).await;
    assert!(
        seen.iter()
            .all(|event| event.message.as_deref() != Some(ADD_DIRS_RUNTIME_REJECTION_MESSAGE)),
        "request-scoped env should prevent add-dir preflight rejection"
    );

    let completion = tokio::time::timeout(STREAM_TIMEOUT, handle.completion)
        .await
        .expect("completion resolves")
        .expect("spawned resume path should still produce a completion");
    assert!(
        !completion.status.success(),
        "fake exec path exits non-zero so the regression proves spawn happened"
    );

    let exec_log = std::fs::read_to_string(&fixture_backend.exec_log)
        .expect("spawned exec path should be recorded");
    assert!(
        exec_log.contains("--add-dir"),
        "expected add-dir flags to be emitted after env-aware probe support"
    );
    assert!(
        exec_log.contains("resume"),
        "expected the logged argv to include the resume flow"
    );
    assert!(
        exec_log.contains(resume_id),
        "expected the logged argv to include the requested resume id"
    );
    assert!(
        exec_log.contains("probe_env=1"),
        "expected request env overrides to reach the resume invocation"
    );
}
