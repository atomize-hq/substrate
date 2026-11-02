use std::{collections::BTreeMap, path::PathBuf, time::Duration};

use tempfile::tempdir;

use super::support::*;

const EXT_ADD_DIRS_V1: &str = "agent_api.exec.add_dirs.v1";

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

fn add_dir_expectations(dirs: &[PathBuf]) -> BTreeMap<String, String> {
    let mut env = BTreeMap::from([(
        "FAKE_CODEX_EXPECT_ADD_DIR_COUNT".to_string(),
        dirs.len().to_string(),
    )]);
    for (index, dir) in dirs.iter().enumerate() {
        env.insert(
            format!("FAKE_CODEX_EXPECT_ADD_DIR_{index}"),
            dir.display().to_string(),
        );
    }
    env
}

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
) -> CodexBackendConfig {
    CodexBackendConfig {
        binary: Some(fake_codex_binary()),
        env: base_env()
            .into_iter()
            .chain(add_dir_expectations(&[expected_add_dir.to_path_buf()]))
            .chain([(
                "FAKE_CODEX_EXPECT_CWD".to_string(),
                expected_cwd.display().to_string(),
            )])
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

    let backend = CodexBackend::new(capture_test_config(&expected_cwd, &expected_add_dir));
    let future = {
        let _cwd_guard = CurrentDirGuard::set(&run_start_root);
        backend.run(capture_test_request())
    };

    let handle = {
        let _cwd_guard = CurrentDirGuard::set(&later_root);
        future.await.expect("run should start")
    };

    let mut events = handle.events;
    let mut seen = Vec::new();
    while let Some(event) = events.next().await {
        seen.push(event);
    }

    let completion = tokio::time::timeout(Duration::from_secs(2), handle.completion)
        .await
        .expect("completion resolves")
        .expect("run completes successfully");
    assert!(completion.status.success(), "events: {seen:?}");
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

    let backend = CodexBackend::new(capture_test_config(&expected_cwd, &expected_add_dir));
    let future = {
        let _cwd_guard = CurrentDirGuard::set(&run_start_root);
        backend.run_control(capture_test_request())
    };

    let control = {
        let _cwd_guard = CurrentDirGuard::set(&later_root);
        future.await.expect("run control should start")
    };

    let mut events = control.handle.events;
    let mut seen = Vec::new();
    while let Some(event) = events.next().await {
        seen.push(event);
    }

    let completion = tokio::time::timeout(Duration::from_secs(2), control.handle.completion)
        .await
        .expect("completion resolves")
        .expect("run completes successfully");
    assert!(completion.status.success(), "events: {seen:?}");
}
