use std::{collections::BTreeMap, path::PathBuf, time::Duration};

use tempfile::tempdir;

use super::support::*;

const BACKPRESSURE_ASSERT_TIMEOUT: Duration = Duration::from_millis(200);
const COMPLETION_TIMEOUT: Duration = Duration::from_secs(5);

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

#[tokio::test]
async fn exec_completion_stays_pending_until_buffered_events_are_drained() {
    let temp = tempdir().expect("tempdir");
    let run_start_cwd = temp.path().join("run-start");
    let expected_cwd = run_start_cwd.join("repo");
    std::fs::create_dir_all(&expected_cwd).expect("create repo root");

    let adapter = test_adapter_with_config_and_run_start_cwd(
        CodexBackendConfig {
            binary: Some(fake_codex_binary()),
            ..Default::default()
        },
        Some(run_start_cwd),
    );

    let env = BTreeMap::from([
        (
            "FAKE_CODEX_EXPECT_SANDBOX".to_string(),
            "workspace-write".to_string(),
        ),
        (
            "FAKE_CODEX_EXPECT_APPROVAL".to_string(),
            "never".to_string(),
        ),
        (
            "FAKE_CODEX_EXPECT_CWD".to_string(),
            expected_cwd.display().to_string(),
        ),
        (
            "FAKE_CODEX_SCENARIO".to_string(),
            "model_runtime_rejection_after_buffered_events".to_string(),
        ),
        (
            "FAKE_CODEX_EXPECT_MODEL".to_string(),
            "gpt-5-codex".to_string(),
        ),
        (
            "FAKE_CODEX_MODEL_RUNTIME_REJECTION_SECRET".to_string(),
            "MODEL_RUNTIME_REJECTION_SECRET_DO_NOT_LEAK".to_string(),
        ),
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
    ]);

    let spawned = adapter
        .spawn(crate::backend_harness::NormalizedRequest {
            agent_kind: adapter.kind(),
            prompt: "hello".to_string(),
            model_id: Some("gpt-5-codex".to_string()),
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

    let mut completion = spawned.completion;
    assert!(
        tokio::time::timeout(BACKPRESSURE_ASSERT_TIMEOUT, &mut completion)
            .await
            .is_err(),
        "completion should remain pending while the buffered backend stream is not drained"
    );

    let backend_events: Vec<_> = spawned
        .events
        .map(|result| result.expect("backend event stream is infallible for fake codex"))
        .collect()
        .await;
    assert!(
        backend_events.len() > crate::backend_harness::DEFAULT_EVENT_CHANNEL_CAPACITY,
        "expected enough buffered events to exercise backpressure"
    );

    let completion = tokio::time::timeout(COMPLETION_TIMEOUT, completion)
        .await
        .expect("completion resolves after the event stream is drained")
        .expect("completion is Ok for fake codex");
    let err = adapter
        .map_completion(completion)
        .expect_err("buffered model rejection must map to Backend error");
    match err {
        AgentWrapperError::Backend { message } => {
            assert_eq!(
                message,
                "codex backend error: model rejected by runtime (details redacted)"
            );
        }
        other => panic!("expected Backend error, got: {other:?}"),
    }
}
