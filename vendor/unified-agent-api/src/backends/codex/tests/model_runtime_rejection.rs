use std::{collections::BTreeMap, path::PathBuf, time::Duration};

use tempfile::tempdir;

use super::support::*;

const BACKPRESSURE_ASSERT_TIMEOUT: Duration = Duration::from_millis(200);

fn fake_codex_binary() -> PathBuf {
    if let Some(path) = std::env::var_os("CARGO_BIN_EXE_fake_codex_stream_exec_scenarios_agent_api")
    {
        return PathBuf::from(path);
    }

    let current_exe = std::env::current_exe().expect("resolve current test binary path");
    let target_dir = current_exe
        .parent()
        .and_then(|dir| dir.parent())
        .expect("resolve target dir from current test binary");
    let mut binary = target_dir.join("fake_codex_stream_exec_scenarios_agent_api");
    if cfg!(windows) {
        binary.set_extension("exe");
    }
    binary
}

fn base_env() -> BTreeMap<String, String> {
    [
        (
            "FAKE_CODEX_EXPECT_SANDBOX".to_string(),
            "workspace-write".to_string(),
        ),
        (
            "FAKE_CODEX_EXPECT_APPROVAL".to_string(),
            "never".to_string(),
        ),
    ]
    .into_iter()
    .collect()
}

async fn assert_codex_runtime_model_rejection(
    scenario: &str,
    extra_env: impl IntoIterator<Item = (String, String)>,
    await_completion_before_events: bool,
    expect_backpressure_before_drain: bool,
    request_model_id: Option<&str>,
    config_model: Option<&str>,
) {
    let temp = tempdir().expect("tempdir");
    let run_start_cwd = temp.path().join("run-start");
    let expected_cwd = run_start_cwd.join("repo");
    std::fs::create_dir_all(&expected_cwd).expect("create repo root");

    let effective_model = request_model_id.or(config_model).expect("effective model");
    let secret = "MODEL_RUNTIME_REJECTION_SECRET_DO_NOT_LEAK";

    let env = base_env()
        .into_iter()
        .chain([
            (
                "FAKE_CODEX_EXPECT_CWD".to_string(),
                expected_cwd.display().to_string(),
            ),
            ("FAKE_CODEX_SCENARIO".to_string(), scenario.to_string()),
            (
                "FAKE_CODEX_EXPECT_MODEL".to_string(),
                effective_model.to_string(),
            ),
            (
                "FAKE_CODEX_MODEL_RUNTIME_REJECTION_SECRET".to_string(),
                secret.to_string(),
            ),
        ])
        .chain(extra_env)
        .collect::<BTreeMap<_, _>>();

    let adapter = test_adapter_with_config_and_run_start_cwd(
        CodexBackendConfig {
            binary: Some(fake_codex_binary()),
            model: config_model.map(str::to_string),
            ..Default::default()
        },
        Some(run_start_cwd),
    );

    let spawned = adapter
        .spawn(crate::backend_harness::NormalizedRequest {
            agent_kind: adapter.kind(),
            prompt: "hello".to_string(),
            model_id: request_model_id.map(str::to_string),
            working_dir: Some(PathBuf::from("repo")),
            effective_timeout: None,
            env,
            policy: CodexExecPolicy {
                add_dirs: Vec::new(),
                non_interactive: true,
                external_sandbox: false,
                approval_policy: None,
                sandbox_mode: CodexSandboxMode::WorkspaceWrite,
                resume: None,
                fork: None,
            },
        })
        .await
        .expect("spawn succeeds");

    let mut events = Some(spawned.events);
    let mut completion = Some(spawned.completion);

    let completion_message = if await_completion_before_events {
        let mut completion_future = completion.take().expect("completion future available");
        if expect_backpressure_before_drain {
            assert!(
                tokio::time::timeout(BACKPRESSURE_ASSERT_TIMEOUT, &mut completion_future)
                    .await
                    .is_err(),
                "completion should remain pending while buffered events are not drained"
            );
        }

        let backend_events: Vec<_> = events
            .take()
            .expect("events stream available")
            .map(|result| result.expect("backend event stream is infallible for fake codex"))
            .collect()
            .await;
        let mapped_events: Vec<_> = backend_events
            .into_iter()
            .flat_map(|event| adapter.map_event(event))
            .collect();

        let completion = tokio::time::timeout(Duration::from_secs(2), completion_future)
            .await
            .expect("completion resolves")
            .expect("completion is Ok for fake codex");
        let err = adapter
            .map_completion(completion)
            .expect_err("runtime rejection must map to Backend error");
        match err {
            AgentWrapperError::Backend { message } => {
                assert_eq!(
                    message,
                    "codex backend error: model rejected by runtime (details redacted)"
                );
                assert!(!message.contains(secret));
                assert!(!message.contains(effective_model));
                assert_eq!(
                    mapped_events
                        .iter()
                        .filter(|event| event.kind == AgentWrapperEventKind::Error)
                        .filter_map(|event| event.message.as_deref())
                        .collect::<Vec<_>>(),
                    vec!["codex backend error: model rejected by runtime (details redacted)"]
                );
                Some((message, mapped_events))
            }
            other => panic!("expected Backend error, got: {other:?}"),
        }
    } else {
        None
    };

    let (completion_message, mapped_events) =
        if let Some((message, mapped_events)) = completion_message {
            (Some(message), mapped_events)
        } else {
            let backend_events: Vec<_> = events
                .take()
                .expect("events stream available")
                .map(|result| result.expect("backend event stream is infallible for fake codex"))
                .collect()
                .await;
            let mapped_events: Vec<_> = backend_events
                .into_iter()
                .flat_map(|event| adapter.map_event(event))
                .collect();
            (None, mapped_events)
        };

    let error_messages: Vec<_> = mapped_events
        .iter()
        .filter(|event| event.kind == AgentWrapperEventKind::Error)
        .filter_map(|event| event.message.as_deref())
        .collect();

    assert_eq!(error_messages.len(), 1, "events: {mapped_events:?}");
    assert_eq!(
        error_messages[0],
        "codex backend error: model rejected by runtime (details redacted)"
    );
    assert!(!error_messages[0].contains(secret));
    assert!(!error_messages[0].contains(effective_model));

    for event in &mapped_events {
        let Some(message) = event.message.as_deref() else {
            continue;
        };
        assert!(
            !message.contains(secret),
            "leaked secret in event: {event:?}"
        );
        assert!(
            !message.contains(effective_model),
            "leaked model id in event: {event:?}"
        );
    }

    if let Some(completion_message) = completion_message {
        assert_eq!(completion_message, error_messages[0]);
    } else {
        let completion = completion
            .take()
            .expect("completion future available")
            .await
            .expect("completion is Ok for fake codex");
        let err = adapter
            .map_completion(completion)
            .expect_err("runtime rejection must map to Backend error");
        match err {
            AgentWrapperError::Backend { message } => {
                assert_eq!(
                    message,
                    "codex backend error: model rejected by runtime (details redacted)"
                );
                assert!(!message.contains(secret));
                assert!(!message.contains(effective_model));
                assert_eq!(message, error_messages[0]);
            }
            other => panic!("expected Backend error, got: {other:?}"),
        }
    }
}

#[tokio::test]
async fn codex_runtime_model_rejection_is_safely_redacted_and_parity_is_preserved() {
    assert_codex_runtime_model_rejection(
        "model_runtime_rejection_after_thread_started",
        std::iter::empty(),
        false,
        false,
        Some("gpt-5-codex"),
        None,
    )
    .await;
}

#[tokio::test]
async fn codex_runtime_model_rejection_remains_fatal_even_on_zero_exit() {
    assert_codex_runtime_model_rejection(
        "model_runtime_rejection_after_thread_started",
        [(
            "FAKE_CODEX_RUNTIME_REJECTION_EXIT_CODE".to_string(),
            "0".to_string(),
        )],
        true,
        false,
        Some("gpt-5-codex"),
        None,
    )
    .await;
}

#[tokio::test]
async fn codex_runtime_model_rejection_waits_for_buffered_terminal_error_before_completion() {
    assert_codex_runtime_model_rejection(
        "model_runtime_rejection_after_buffered_events",
        [
            (
                "FAKE_CODEX_BUFFERED_EVENT_COUNT".to_string(),
                "1024".to_string(),
            ),
            (
                "FAKE_CODEX_BUFFERED_EVENT_PADDING_BYTES".to_string(),
                "1024".to_string(),
            ),
            (
                "FAKE_CODEX_RUNTIME_REJECTION_EXIT_CODE".to_string(),
                "0".to_string(),
            ),
        ],
        true,
        true,
        Some("gpt-5-codex"),
        None,
    )
    .await;
}

#[tokio::test]
async fn codex_config_model_runtime_rejection_remains_fatal_even_on_zero_exit() {
    assert_codex_runtime_model_rejection(
        "model_runtime_rejection_after_thread_started",
        [(
            "FAKE_CODEX_RUNTIME_REJECTION_EXIT_CODE".to_string(),
            "0".to_string(),
        )],
        true,
        false,
        None,
        Some("gpt-5-codex"),
    )
    .await;
}

#[tokio::test]
async fn codex_config_model_runtime_rejection_waits_for_buffered_terminal_error_before_completion()
{
    assert_codex_runtime_model_rejection(
        "model_runtime_rejection_after_buffered_events",
        [
            (
                "FAKE_CODEX_BUFFERED_EVENT_COUNT".to_string(),
                "1024".to_string(),
            ),
            (
                "FAKE_CODEX_BUFFERED_EVENT_PADDING_BYTES".to_string(),
                "1024".to_string(),
            ),
            (
                "FAKE_CODEX_RUNTIME_REJECTION_EXIT_CODE".to_string(),
                "0".to_string(),
            ),
        ],
        true,
        true,
        None,
        Some("gpt-5-codex"),
    )
    .await;
}
