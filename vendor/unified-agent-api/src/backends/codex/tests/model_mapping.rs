use std::{collections::BTreeMap, path::PathBuf, time::Duration};

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

async fn spawn_and_drain(
    model_id: Option<String>,
    config_model: Option<String>,
    resume: Option<SessionSelectorV1>,
    add_dirs: Vec<PathBuf>,
    env: BTreeMap<String, String>,
    run_start_cwd: PathBuf,
    prompt: &str,
) {
    let config = CodexBackendConfig {
        binary: Some(fake_codex_binary()),
        model: config_model,
        ..Default::default()
    };

    let adapter = test_adapter_with_config_and_run_start_cwd(config, Some(run_start_cwd));

    let spawned = adapter
        .spawn(crate::backend_harness::NormalizedRequest {
            agent_kind: adapter.kind(),
            prompt: prompt.to_string(),
            model_id,
            working_dir: Some(PathBuf::from("repo")),
            effective_timeout: None,
            env,
            policy: CodexExecPolicy {
                add_dirs,
                non_interactive: true,
                external_sandbox: false,
                approval_policy: None,
                sandbox_mode: CodexSandboxMode::WorkspaceWrite,
                resume,
                fork: None,
            },
        })
        .await
        .expect("spawn succeeds");

    let backend_events: Vec<_> = spawned
        .events
        .map(|result| result.expect("backend event stream is infallible for fake codex"))
        .collect()
        .await;

    let completion = tokio::time::timeout(Duration::from_secs(2), spawned.completion)
        .await
        .expect("completion resolves")
        .expect("run completes successfully");
    assert!(
        completion.status.success(),
        "events: {backend_events:?}, completion: {completion:?}"
    );
}

#[tokio::test]
async fn codex_exec_model_id_beats_backend_config_model_and_orders_before_add_dir() {
    let temp = tempdir().expect("tempdir");
    let run_start_cwd = temp.path().join("run-start");
    let expected_cwd = run_start_cwd.join("repo");
    let expected_add_dir = expected_cwd.join("docs");
    std::fs::create_dir_all(&expected_add_dir).expect("create add-dir target");

    let env = base_env()
        .into_iter()
        .chain(add_dir_expectations(std::slice::from_ref(
            &expected_add_dir,
        )))
        .chain([
            (
                "FAKE_CODEX_EXPECT_CWD".to_string(),
                expected_cwd.display().to_string(),
            ),
            (
                "FAKE_CODEX_EXPECT_MODEL".to_string(),
                "request-model".to_string(),
            ),
        ])
        .collect();

    spawn_and_drain(
        Some("request-model".to_string()),
        Some("config-model".to_string()),
        None,
        vec![expected_add_dir],
        env,
        run_start_cwd,
        "hello",
    )
    .await;
}

#[tokio::test]
async fn codex_exec_without_model_id_emits_no_model_flag_when_config_model_absent() {
    let temp = tempdir().expect("tempdir");
    let run_start_cwd = temp.path().join("run-start");
    let expected_cwd = run_start_cwd.join("repo");
    let expected_add_dir = expected_cwd.join("docs");
    std::fs::create_dir_all(&expected_add_dir).expect("create add-dir target");

    let env = base_env()
        .into_iter()
        .chain(add_dir_expectations(std::slice::from_ref(
            &expected_add_dir,
        )))
        .chain([
            (
                "FAKE_CODEX_EXPECT_CWD".to_string(),
                expected_cwd.display().to_string(),
            ),
            (
                "FAKE_CODEX_EXPECT_MODEL".to_string(),
                "<absent>".to_string(),
            ),
        ])
        .collect();

    spawn_and_drain(
        None,
        None,
        None,
        vec![expected_add_dir],
        env,
        run_start_cwd,
        "hello",
    )
    .await;
}

#[tokio::test]
async fn codex_exec_without_model_id_uses_backend_config_model() {
    let temp = tempdir().expect("tempdir");
    let run_start_cwd = temp.path().join("run-start");
    let expected_cwd = run_start_cwd.join("repo");
    let expected_add_dir = expected_cwd.join("docs");
    std::fs::create_dir_all(&expected_add_dir).expect("create add-dir target");

    let env = base_env()
        .into_iter()
        .chain(add_dir_expectations(std::slice::from_ref(
            &expected_add_dir,
        )))
        .chain([
            (
                "FAKE_CODEX_EXPECT_CWD".to_string(),
                expected_cwd.display().to_string(),
            ),
            (
                "FAKE_CODEX_EXPECT_MODEL".to_string(),
                "config-model".to_string(),
            ),
        ])
        .collect();

    spawn_and_drain(
        None,
        Some("config-model".to_string()),
        None,
        vec![expected_add_dir],
        env,
        run_start_cwd,
        "hello",
    )
    .await;
}

#[tokio::test]
async fn codex_resume_emits_model_id_and_orders_before_add_dir() {
    let temp = tempdir().expect("tempdir");
    let run_start_cwd = temp.path().join("run-start");
    let expected_cwd = run_start_cwd.join("repo");
    let expected_add_dir = expected_cwd.join("docs");
    std::fs::create_dir_all(&expected_add_dir).expect("create add-dir target");

    let env = base_env()
        .into_iter()
        .chain(add_dir_expectations(std::slice::from_ref(
            &expected_add_dir,
        )))
        .chain([
            (
                "FAKE_CODEX_EXPECT_CWD".to_string(),
                expected_cwd.display().to_string(),
            ),
            (
                "FAKE_CODEX_SCENARIO".to_string(),
                "resume_last_assert".to_string(),
            ),
            (
                "FAKE_CODEX_EXPECT_PROMPT".to_string(),
                "resume me".to_string(),
            ),
            (
                "FAKE_CODEX_EXPECT_MODEL".to_string(),
                "request-model".to_string(),
            ),
        ])
        .collect();

    spawn_and_drain(
        Some("request-model".to_string()),
        None,
        Some(SessionSelectorV1::Last),
        vec![expected_add_dir],
        env,
        run_start_cwd,
        "resume me",
    )
    .await;
}
