#![cfg(feature = "codex")]

#[allow(dead_code, unused_imports)]
#[path = "c2_codex_session_resume_v1/support.rs"]
mod support;

use agent_api::{AgentWrapperBackend, AgentWrapperError, AgentWrapperEventKind};

use support::{
    add_dirs_extension, add_dirs_fixture, assert_no_add_dir_sentinel_leaks_in_events, base_env,
    build_probe_only_backend, drain_to_none, run_request, AddDirProbeMode,
    ADD_DIRS_RUNTIME_REJECTION_MESSAGE, EXTERNAL_SANDBOX_WARNING, STREAM_TIMEOUT,
};

#[cfg(unix)]
async fn assert_preflight_rejection_case(external_sandbox: bool) {
    let fixture = add_dirs_fixture();
    let fixture_backend = build_probe_only_backend(
        AddDirProbeMode::Unsupported,
        base_env(),
        None,
        external_sandbox,
    );

    let mut extensions = vec![add_dirs_extension(&fixture.dirs)];
    if external_sandbox {
        extensions.push((
            "agent_api.exec.external_sandbox.v1".to_string(),
            serde_json::json!(true),
        ));
    }

    let handle = tokio::time::timeout(
        std::time::Duration::from_millis(100),
        fixture_backend
            .backend
            .run(run_request("hello world", extensions)),
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
    assert!(
        !seen.iter().any(|event| {
            event.kind == AgentWrapperEventKind::Status
                && event.message.as_deref() == Some(EXTERNAL_SANDBOX_WARNING)
        }),
        "add-dir probe rejection should not emit the external sandbox startup warning"
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

#[cfg(unix)]
#[tokio::test]
async fn exec_add_dirs_preflight_rejection_surfaces_via_handle() {
    assert_preflight_rejection_case(false).await;
}

#[cfg(unix)]
#[tokio::test]
async fn exec_add_dirs_preflight_rejection_beats_external_sandbox_startup_stream() {
    assert_preflight_rejection_case(true).await;
}

#[cfg(unix)]
#[tokio::test]
async fn exec_add_dirs_slow_supported_probe_is_not_limited_by_run_timeout() {
    let fixture = add_dirs_fixture();
    let fixture_backend =
        build_probe_only_backend(AddDirProbeMode::SlowSupported, base_env(), None, false);
    let mut request = run_request("hello world", [add_dirs_extension(&fixture.dirs)]);
    request.timeout = Some(std::time::Duration::from_millis(50));

    let handle = tokio::time::timeout(
        std::time::Duration::from_secs(1),
        fixture_backend.backend.run(request),
    )
    .await
    .expect("slow probe should still let backend.run return a handle")
    .expect("slow supported probe should not preflight-reject add-dirs");

    let mut events = handle.events;
    let seen = drain_to_none(events.as_mut(), STREAM_TIMEOUT).await;
    assert!(
        !seen
            .iter()
            .any(|event| { event.message.as_deref() == Some(ADD_DIRS_RUNTIME_REJECTION_MESSAGE) }),
        "slow supported probes must not be misclassified as add-dir runtime rejection"
    );

    let completion = tokio::time::timeout(STREAM_TIMEOUT, handle.completion)
        .await
        .expect("completion resolves")
        .expect("spawned exec path should still produce a completion");
    assert!(
        !completion.status.success(),
        "fake exec path exits non-zero so the regression proves spawn happened"
    );

    let exec_log = std::fs::read_to_string(&fixture_backend.exec_log)
        .expect("spawned exec path should be recorded");
    assert!(
        exec_log.contains("exec"),
        "expected fake codex exec path to run after successful probe"
    );
}

#[cfg(unix)]
#[tokio::test]
async fn exec_add_dirs_env_sensitive_probe_uses_request_env() {
    let fixture = add_dirs_fixture();
    let fixture_backend = build_probe_only_backend(
        AddDirProbeMode::EnvSensitiveSupported,
        base_env(),
        None,
        false,
    );
    let mut request = run_request("hello world", [add_dirs_extension(&fixture.dirs)]);
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
        .expect("spawned exec path should still produce a completion");
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
    for dir in &fixture.dirs {
        assert!(
            exec_log.contains(&dir.display().to_string()),
            "expected logged exec argv to include add-dir {}",
            dir.display()
        );
    }
    assert!(
        exec_log.contains("probe_env=1"),
        "expected request env overrides to reach the exec invocation"
    );
}
