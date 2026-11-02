use std::{collections::BTreeMap, path::PathBuf, sync::Arc, time::Duration};

#[cfg(windows)]
use std::path::{Component, Path};
use tempfile::tempdir;

use super::support::*;

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

async fn assert_exec_add_dirs_case(
    config: CodexBackendConfig,
    run_start_cwd: PathBuf,
    request_working_dir: Option<PathBuf>,
    add_dirs: &[&str],
) {
    let adapter = Arc::new(test_adapter_with_config_and_run_start_cwd(
        config.clone(),
        Some(run_start_cwd),
    ));
    let defaults = BackendDefaults {
        env: config.env.clone(),
        default_timeout: config.default_timeout,
    };
    let handle = crate::backend_harness::run_harnessed_backend(
        adapter,
        defaults,
        AgentWrapperRunRequest {
            prompt: "hello".to_string(),
            working_dir: request_working_dir,
            extensions: [(EXT_ADD_DIRS_V1.to_string(), add_dirs_payload(add_dirs))]
                .into_iter()
                .collect(),
            ..Default::default()
        },
    )
    .await
    .expect("run should start");

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
async fn codex_exec_resolves_relative_request_working_dir_before_add_dirs_and_spawn() {
    let temp = tempdir().expect("tempdir");
    let run_start_cwd = temp.path().join("run-start");
    let expected_cwd = run_start_cwd.join("repo");
    let expected_add_dir = expected_cwd.join("docs");
    std::fs::create_dir_all(&expected_add_dir).expect("create add-dir target");

    let config = CodexBackendConfig {
        binary: Some(fake_codex_binary()),
        env: base_env()
            .into_iter()
            .chain(add_dir_expectations(std::slice::from_ref(
                &expected_add_dir,
            )))
            .chain([(
                "FAKE_CODEX_EXPECT_CWD".to_string(),
                expected_cwd.display().to_string(),
            )])
            .collect(),
        ..Default::default()
    };

    assert_exec_add_dirs_case(
        config,
        run_start_cwd,
        Some(PathBuf::from("repo")),
        &["docs"],
    )
    .await;
}

#[tokio::test]
async fn codex_exec_resolves_relative_default_working_dir_before_add_dirs_and_spawn() {
    let temp = tempdir().expect("tempdir");
    let run_start_cwd = temp.path().join("run-start");
    let expected_cwd = run_start_cwd.join("repo");
    let expected_add_dir = expected_cwd.join("docs");
    std::fs::create_dir_all(&expected_add_dir).expect("create add-dir target");

    let config = CodexBackendConfig {
        binary: Some(fake_codex_binary()),
        default_working_dir: Some(PathBuf::from("repo")),
        env: base_env()
            .into_iter()
            .chain(add_dir_expectations(std::slice::from_ref(
                &expected_add_dir,
            )))
            .chain([(
                "FAKE_CODEX_EXPECT_CWD".to_string(),
                expected_cwd.display().to_string(),
            )])
            .collect(),
        ..Default::default()
    };

    assert_exec_add_dirs_case(config, run_start_cwd, None, &["docs"]).await;
}

#[cfg(windows)]
#[tokio::test]
async fn codex_exec_resolves_drive_relative_request_working_dir_before_add_dirs_and_spawn() {
    let temp = tempdir().expect("tempdir");
    let run_start_cwd = temp.path().join("run-start");
    let expected_cwd = run_start_cwd.join("repo");
    let expected_add_dir = expected_cwd.join("docs");
    std::fs::create_dir_all(&expected_add_dir).expect("create add-dir target");

    let config = CodexBackendConfig {
        binary: Some(fake_codex_binary()),
        env: base_env()
            .into_iter()
            .chain(add_dir_expectations(std::slice::from_ref(
                &expected_add_dir,
            )))
            .chain([(
                "FAKE_CODEX_EXPECT_CWD".to_string(),
                expected_cwd.display().to_string(),
            )])
            .collect(),
        ..Default::default()
    };

    assert_exec_add_dirs_case(
        config,
        run_start_cwd.clone(),
        Some(windows_drive_relative("repo", &run_start_cwd)),
        &["docs"],
    )
    .await;
}

#[cfg(windows)]
#[tokio::test]
async fn codex_exec_deduplicates_case_insensitive_add_dirs_before_spawn() {
    let temp = tempdir().expect("tempdir");
    let run_start_cwd = temp.path().join("run-start");
    let expected_cwd = run_start_cwd.join("repo");
    let expected_add_dir = expected_cwd.join("docs");
    std::fs::create_dir_all(&expected_add_dir).expect("create add-dir target");

    let config = CodexBackendConfig {
        binary: Some(fake_codex_binary()),
        env: base_env()
            .into_iter()
            .chain(add_dir_expectations(std::slice::from_ref(
                &expected_add_dir,
            )))
            .chain([(
                "FAKE_CODEX_EXPECT_CWD".to_string(),
                expected_cwd.display().to_string(),
            )])
            .collect(),
        ..Default::default()
    };

    assert_exec_add_dirs_case(
        config,
        run_start_cwd,
        Some(PathBuf::from("repo")),
        &["docs", "DOCS"],
    )
    .await;
}

#[cfg(windows)]
fn windows_drive_relative(relative: &str, absolute_path: &Path) -> PathBuf {
    let prefix = absolute_path
        .components()
        .find_map(|component| match component {
            Component::Prefix(value) => Some(value.as_os_str().to_string_lossy().into_owned()),
            _ => None,
        })
        .expect("absolute windows path should include a prefix");
    PathBuf::from(format!("{prefix}{relative}"))
}
