use std::time::Duration;

use super::support::*;

#[tokio::test]
async fn external_sandbox_allow_flag_supported_includes_allow_flag_and_emits_warning_before_handle_and_user_events(
) {
    let prompt = "hello";
    let log_path = unique_temp_log_path("claude_external_sandbox_allow_supported");

    let backend = ClaudeCodeBackend::new(ClaudeCodeBackendConfig {
        binary: Some(fake_claude_binary()),
        env: [
            (
                "FAKE_CLAUDE_SCENARIO".to_string(),
                "final_text_and_tools".to_string(),
            ),
            (
                "FAKE_CLAUDE_HELP_SUPPORTS_ALLOW_FLAG".to_string(),
                "1".to_string(),
            ),
            (
                "FAKE_CLAUDE_EXPECT_DANGEROUS_SKIP_PERMISSIONS".to_string(),
                "1".to_string(),
            ),
            (
                "FAKE_CLAUDE_EXPECT_ALLOW_DANGEROUSLY_SKIP_PERMISSIONS".to_string(),
                "1".to_string(),
            ),
            (
                "FAKE_CLAUDE_INVOCATION_LOG".to_string(),
                log_path.to_string_lossy().to_string(),
            ),
        ]
        .into_iter()
        .collect(),
        allow_external_sandbox_exec: true,
        ..Default::default()
    });

    let handle = backend
        .run(AgentWrapperRunRequest {
            prompt: prompt.to_string(),
            extensions: [(
                "agent_api.exec.external_sandbox.v1".to_string(),
                json!(true),
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

    let warnings = warning_indices(&seen);
    assert_eq!(warnings.len(), 1);
    let warning_idx = warnings[0];

    let handle_idx = session_handle_facet_index(&seen)
        .expect("expected a Status event with the session handle facet");
    assert!(warning_idx < handle_idx);

    let first_user_visible = first_user_visible_index(&seen)
        .expect("expected at least one user-visible event (TextOutput/ToolCall/ToolResult)");
    assert!(warning_idx < first_user_visible);

    let completion = tokio::time::timeout(Duration::from_secs(2), completion)
        .await
        .expect("completion resolves")
        .expect("completion ok");
    assert!(completion.status.success());

    let invocations = read_invocations(&log_path);
    assert_eq!(count(&invocations, "help"), 1);
    assert_eq!(count(&invocations, "print"), 1);
    let help_pos = first_index(&invocations, "help").unwrap();
    let print_pos = first_index(&invocations, "print").unwrap();
    assert!(help_pos < print_pos);
}

#[tokio::test]
async fn external_sandbox_allow_flag_unsupported_excludes_allow_flag_and_emits_warning_before_handle_and_user_events(
) {
    let prompt = "hello";
    let log_path = unique_temp_log_path("claude_external_sandbox_allow_unsupported");

    let backend = ClaudeCodeBackend::new(ClaudeCodeBackendConfig {
        binary: Some(fake_claude_binary()),
        env: [
            (
                "FAKE_CLAUDE_SCENARIO".to_string(),
                "final_text_and_tools".to_string(),
            ),
            (
                "FAKE_CLAUDE_HELP_SUPPORTS_ALLOW_FLAG".to_string(),
                "0".to_string(),
            ),
            (
                "FAKE_CLAUDE_EXPECT_DANGEROUS_SKIP_PERMISSIONS".to_string(),
                "1".to_string(),
            ),
            (
                "FAKE_CLAUDE_EXPECT_ALLOW_DANGEROUSLY_SKIP_PERMISSIONS".to_string(),
                "0".to_string(),
            ),
            (
                "FAKE_CLAUDE_INVOCATION_LOG".to_string(),
                log_path.to_string_lossy().to_string(),
            ),
        ]
        .into_iter()
        .collect(),
        allow_external_sandbox_exec: true,
        ..Default::default()
    });

    let handle = backend
        .run(AgentWrapperRunRequest {
            prompt: prompt.to_string(),
            extensions: [(
                "agent_api.exec.external_sandbox.v1".to_string(),
                json!(true),
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

    let warnings = warning_indices(&seen);
    assert_eq!(warnings.len(), 1);
    let warning_idx = warnings[0];

    let handle_idx = session_handle_facet_index(&seen)
        .expect("expected a Status event with the session handle facet");
    assert!(warning_idx < handle_idx);

    let first_user_visible = first_user_visible_index(&seen)
        .expect("expected at least one user-visible event (TextOutput/ToolCall/ToolResult)");
    assert!(warning_idx < first_user_visible);

    let completion = tokio::time::timeout(Duration::from_secs(2), completion)
        .await
        .expect("completion resolves")
        .expect("completion ok");
    assert!(completion.status.success());

    let invocations = read_invocations(&log_path);
    assert_eq!(count(&invocations, "help"), 1);
    assert_eq!(count(&invocations, "print"), 1);
    let help_pos = first_index(&invocations, "help").unwrap();
    let print_pos = first_index(&invocations, "print").unwrap();
    assert!(help_pos < print_pos);
}

#[tokio::test]
async fn external_sandbox_help_preflight_failure_returns_backend_error_before_print_and_redacts_output(
) {
    let secret = format!("S3C_SECRET_{}", std::process::id());
    let log_path = unique_temp_log_path("claude_external_sandbox_help_failure");

    let backend = ClaudeCodeBackend::new(ClaudeCodeBackendConfig {
        binary: Some(fake_claude_binary()),
        env: [
            ("FAKE_CLAUDE_HELP_DELAY_MS".to_string(), "300".to_string()),
            ("FAKE_CLAUDE_HELP_FAIL".to_string(), "1".to_string()),
            ("FAKE_CLAUDE_HELP_FAIL_SECRET".to_string(), secret.clone()),
            (
                "FAKE_CLAUDE_PRINT_SHOULD_NOT_RUN".to_string(),
                "1".to_string(),
            ),
            (
                "FAKE_CLAUDE_INVOCATION_LOG".to_string(),
                log_path.to_string_lossy().to_string(),
            ),
        ]
        .into_iter()
        .collect(),
        allow_external_sandbox_exec: true,
        ..Default::default()
    });

    let handle = tokio::time::timeout(
        Duration::from_millis(100),
        backend.run(AgentWrapperRunRequest {
            prompt: "hello".to_string(),
            extensions: [(
                "agent_api.exec.external_sandbox.v1".to_string(),
                json!(true),
            )]
            .into_iter()
            .collect(),
            ..Default::default()
        }),
    )
    .await
    .expect("run should return before help preflight completes")
    .expect("run yields a handle; preflight failure is surfaced via events/completion");

    let mut events = handle.events;
    let completion = handle.completion;

    let seen = drain_to_none(events.as_mut(), Duration::from_secs(2)).await;
    let warnings = warning_indices(&seen);
    assert_eq!(warnings.len(), 1);
    let warning_idx = warnings[0];
    let error_idx = first_error_index(&seen).expect("expected at least one Error event");
    assert!(warning_idx < error_idx);

    let error_messages: Vec<&str> = seen
        .iter()
        .filter(|ev| ev.kind == AgentWrapperEventKind::Error)
        .filter_map(|ev| ev.message.as_deref())
        .collect();
    assert!(!error_messages.is_empty());
    for message in error_messages {
        assert!(!message.contains(&secret));
    }

    let err = tokio::time::timeout(Duration::from_secs(2), completion)
        .await
        .expect("completion resolves")
        .expect_err("expected completion error on preflight failure");
    match err {
        AgentWrapperError::Backend { message } => {
            assert!(!message.contains(&secret));
        }
        other => panic!("expected AgentWrapperError::Backend, got: {other:?}"),
    }

    let invocations = read_invocations(&log_path);
    assert_eq!(count(&invocations, "help"), 1);
    assert_eq!(count(&invocations, "print"), 0);
}

#[tokio::test]
async fn external_sandbox_print_spawn_failure_after_cached_preflight_emits_warning_before_error() {
    let prompt = "hello";
    let log_path = unique_temp_log_path("claude_external_sandbox_print_spawn_failure");

    let backend = ClaudeCodeBackend::new(ClaudeCodeBackendConfig {
        binary: Some(fake_claude_binary()),
        env: [
            (
                "FAKE_CLAUDE_SCENARIO".to_string(),
                "final_text_and_tools".to_string(),
            ),
            (
                "FAKE_CLAUDE_HELP_SUPPORTS_ALLOW_FLAG".to_string(),
                "1".to_string(),
            ),
            (
                "FAKE_CLAUDE_EXPECT_DANGEROUS_SKIP_PERMISSIONS".to_string(),
                "1".to_string(),
            ),
            (
                "FAKE_CLAUDE_EXPECT_ALLOW_DANGEROUSLY_SKIP_PERMISSIONS".to_string(),
                "1".to_string(),
            ),
            (
                "FAKE_CLAUDE_INVOCATION_LOG".to_string(),
                log_path.to_string_lossy().to_string(),
            ),
        ]
        .into_iter()
        .collect(),
        allow_external_sandbox_exec: true,
        ..Default::default()
    });

    let warm_handle = backend
        .run(AgentWrapperRunRequest {
            prompt: prompt.to_string(),
            extensions: [(
                "agent_api.exec.external_sandbox.v1".to_string(),
                json!(true),
            )]
            .into_iter()
            .collect(),
            ..Default::default()
        })
        .await
        .expect("warm-up run");
    let mut warm_events = warm_handle.events;
    let warm_completion = warm_handle.completion;
    let _ = drain_to_none(warm_events.as_mut(), Duration::from_secs(2)).await;
    let warm_completion = tokio::time::timeout(Duration::from_secs(2), warm_completion)
        .await
        .expect("warm-up completion resolves")
        .expect("warm-up completion ok");
    assert!(warm_completion.status.success());

    let missing_dir = unique_missing_dir_path("claude_external_sandbox_print_spawn_failure");
    assert!(!missing_dir.exists());

    let handle = backend
        .run(AgentWrapperRunRequest {
            prompt: prompt.to_string(),
            working_dir: Some(missing_dir),
            extensions: [(
                "agent_api.exec.external_sandbox.v1".to_string(),
                json!(true),
            )]
            .into_iter()
            .collect(),
            ..Default::default()
        })
        .await
        .expect("run yields a handle; spawn failure is surfaced via events/completion");

    let mut events = handle.events;
    let completion = handle.completion;

    let seen = drain_to_none(events.as_mut(), Duration::from_secs(2)).await;
    let warnings = warning_indices(&seen);
    assert_eq!(warnings.len(), 1);
    let warning_idx = warnings[0];
    let error_idx = first_error_index(&seen).expect("expected at least one Error event");
    assert!(warning_idx < error_idx);

    let err = tokio::time::timeout(Duration::from_secs(2), completion)
        .await
        .expect("completion resolves")
        .expect_err("expected completion error on spawn failure");
    match err {
        AgentWrapperError::Backend { .. } => {}
        other => panic!("expected AgentWrapperError::Backend, got: {other:?}"),
    }

    let invocations = read_invocations(&log_path);
    assert_eq!(count(&invocations, "help"), 1);
    assert_eq!(count(&invocations, "print"), 1);
}

#[tokio::test]
async fn external_sandbox_help_preflight_is_cached_per_backend_instance() {
    let prompt = "hello";
    let log_path = unique_temp_log_path("claude_external_sandbox_help_cached");

    let backend = ClaudeCodeBackend::new(ClaudeCodeBackendConfig {
        binary: Some(fake_claude_binary()),
        env: [
            (
                "FAKE_CLAUDE_SCENARIO".to_string(),
                "final_text_and_tools".to_string(),
            ),
            (
                "FAKE_CLAUDE_HELP_SUPPORTS_ALLOW_FLAG".to_string(),
                "1".to_string(),
            ),
            (
                "FAKE_CLAUDE_EXPECT_DANGEROUS_SKIP_PERMISSIONS".to_string(),
                "1".to_string(),
            ),
            (
                "FAKE_CLAUDE_EXPECT_ALLOW_DANGEROUSLY_SKIP_PERMISSIONS".to_string(),
                "1".to_string(),
            ),
            (
                "FAKE_CLAUDE_INVOCATION_LOG".to_string(),
                log_path.to_string_lossy().to_string(),
            ),
        ]
        .into_iter()
        .collect(),
        allow_external_sandbox_exec: true,
        ..Default::default()
    });

    for run_idx in 0..2 {
        let handle = backend
            .run(AgentWrapperRunRequest {
                prompt: prompt.to_string(),
                extensions: [(
                    "agent_api.exec.external_sandbox.v1".to_string(),
                    json!(true),
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
        let warnings = warning_indices(&seen);
        assert_eq!(
            warnings.len(),
            1,
            "run {run_idx}: expected exactly one pinned external sandbox warning Status event"
        );

        let completion = tokio::time::timeout(Duration::from_secs(2), completion)
            .await
            .expect("completion resolves")
            .expect("completion ok");
        assert!(completion.status.success());
    }

    let invocations = read_invocations(&log_path);
    assert_eq!(count(&invocations, "help"), 1);
    assert_eq!(count(&invocations, "print"), 2);
}
