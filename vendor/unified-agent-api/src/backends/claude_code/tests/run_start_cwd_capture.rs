use std::{path::PathBuf, time::Duration};

use tempfile::tempdir;

use super::support::*;

const EXT_ADD_DIRS_V1: &str = "agent_api.exec.add_dirs.v1";

fn capture_test_request() -> AgentWrapperRunRequest {
    AgentWrapperRunRequest {
        prompt: "hello".to_string(),
        working_dir: Some(PathBuf::from("repo")),
        extensions: [(EXT_ADD_DIRS_V1.to_string(), add_dirs_payload(&["docs"]))]
            .into_iter()
            .collect(),
        ..Default::default()
    }
}

fn capture_test_config(
    expected_cwd: &std::path::Path,
    expected_add_dir: &std::path::Path,
) -> ClaudeCodeBackendConfig {
    ClaudeCodeBackendConfig {
        binary: Some(fake_claude_binary()),
        env: expected_add_dirs_env(&[expected_add_dir.to_path_buf()])
            .into_iter()
            .chain([
                (
                    "FAKE_CLAUDE_SCENARIO".to_string(),
                    "fresh_assert".to_string(),
                ),
                ("FAKE_CLAUDE_EXPECT_PROMPT".to_string(), "hello".to_string()),
                (
                    "FAKE_CLAUDE_EXPECT_CWD".to_string(),
                    expected_cwd.display().to_string(),
                ),
            ])
            .collect(),
        ..Default::default()
    }
}

#[tokio::test]
async fn run_captures_run_start_cwd_before_future_is_awaited() {
    let _env_lock = test_env_lock();
    let temp = tempdir().expect("tempdir");
    let run_start_root = temp.path().join("run-start");
    let later_root = temp.path().join("later");
    std::fs::create_dir_all(&run_start_root).expect("create run-start root");
    let canonical_run_start = {
        let _cwd_guard = CurrentDirGuard::set(&run_start_root);
        std::env::current_dir().expect("read invocation cwd")
    };
    let expected_cwd = canonical_run_start.join("repo");
    let expected_add_dir = expected_cwd.join("docs");
    std::fs::create_dir_all(&expected_add_dir).expect("create run-start add-dir target");
    std::fs::create_dir_all(later_root.join("repo").join("docs"))
        .expect("create later add-dir target");

    let backend = ClaudeCodeBackend::new(capture_test_config(&expected_cwd, &expected_add_dir));
    let future = {
        let _cwd_guard = CurrentDirGuard::set(&run_start_root);
        backend.run(capture_test_request())
    };

    let handle = {
        let _cwd_guard = CurrentDirGuard::set(&later_root);
        future.await.expect("run should start")
    };

    let mut events = handle.events;
    let seen = drain_to_none(events.as_mut(), Duration::from_secs(2)).await;

    let completion = tokio::time::timeout(Duration::from_secs(2), handle.completion)
        .await
        .expect("completion resolves")
        .expect("run completes successfully");
    assert!(
        completion.status.success(),
        "expected successful fake Claude run, events: {seen:?}",
    );
}

#[tokio::test]
async fn run_control_captures_run_start_cwd_before_future_is_awaited() {
    let _env_lock = test_env_lock();
    let temp = tempdir().expect("tempdir");
    let run_start_root = temp.path().join("run-start");
    let later_root = temp.path().join("later");
    std::fs::create_dir_all(&run_start_root).expect("create run-start root");
    let canonical_run_start = {
        let _cwd_guard = CurrentDirGuard::set(&run_start_root);
        std::env::current_dir().expect("read invocation cwd")
    };
    let expected_cwd = canonical_run_start.join("repo");
    let expected_add_dir = expected_cwd.join("docs");
    std::fs::create_dir_all(&expected_add_dir).expect("create run-start add-dir target");
    std::fs::create_dir_all(later_root.join("repo").join("docs"))
        .expect("create later add-dir target");

    let backend = ClaudeCodeBackend::new(capture_test_config(&expected_cwd, &expected_add_dir));
    let future = {
        let _cwd_guard = CurrentDirGuard::set(&run_start_root);
        backend.run_control(capture_test_request())
    };

    let control = {
        let _cwd_guard = CurrentDirGuard::set(&later_root);
        future.await.expect("run control should start")
    };

    let mut events = control.handle.events;
    let seen = drain_to_none(events.as_mut(), Duration::from_secs(2)).await;

    let completion = tokio::time::timeout(Duration::from_secs(2), control.handle.completion)
        .await
        .expect("completion resolves")
        .expect("run completes successfully");
    assert!(
        completion.status.success(),
        "expected successful fake Claude run, events: {seen:?}",
    );
}
