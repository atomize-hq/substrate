use std::{collections::BTreeMap, fs, path::PathBuf, sync::Arc, time::Duration};

use tempfile::tempdir;

use super::support::*;

fn add_dirs_json(dirs: &[PathBuf]) -> JsonValue {
    add_dirs_payload(
        &dirs
            .iter()
            .map(|dir| dir.display().to_string())
            .collect::<Vec<_>>(),
    )
}

async fn run_claude_assertion(
    prompt: &str,
    scenario: &str,
    mut env: BTreeMap<String, String>,
    extensions: BTreeMap<String, JsonValue>,
) {
    let _env_lock = test_env_lock();
    env.insert("FAKE_CLAUDE_SCENARIO".to_string(), scenario.to_string());
    env.insert("FAKE_CLAUDE_EXPECT_PROMPT".to_string(), prompt.to_string());

    let backend = ClaudeCodeBackend::new(ClaudeCodeBackendConfig {
        binary: Some(fake_claude_binary()),
        env,
        ..Default::default()
    });

    let handle = backend
        .run(AgentWrapperRunRequest {
            prompt: prompt.to_string(),
            extensions,
            ..Default::default()
        })
        .await
        .expect("Claude run should start");

    let mut events = handle.events;
    let _seen = drain_to_none(events.as_mut(), Duration::from_secs(2)).await;

    let completion = tokio::time::timeout(Duration::from_secs(2), handle.completion)
        .await
        .expect("completion resolves")
        .expect("run completes successfully");
    assert!(
        completion.status.success(),
        "expected successful fake Claude run, events: {:?}",
        _seen
    );
}

async fn run_claude_assertion_with_adapter(
    prompt: &str,
    scenario: &str,
    mut config: ClaudeCodeBackendConfig,
    run_start_cwd: Option<PathBuf>,
    request_working_dir: Option<PathBuf>,
    extensions: BTreeMap<String, JsonValue>,
) {
    let _env_lock = test_env_lock();
    config
        .env
        .insert("FAKE_CLAUDE_SCENARIO".to_string(), scenario.to_string());
    config
        .env
        .insert("FAKE_CLAUDE_EXPECT_PROMPT".to_string(), prompt.to_string());
    let adapter = Arc::new(new_adapter_with_config_and_run_start_cwd(
        config.clone(),
        run_start_cwd,
    ));
    let defaults = crate::backend_harness::BackendDefaults {
        env: config.env.clone(),
        default_timeout: config.default_timeout,
    };
    let handle = crate::backend_harness::run_harnessed_backend(
        adapter,
        defaults,
        AgentWrapperRunRequest {
            prompt: prompt.to_string(),
            working_dir: request_working_dir,
            extensions,
            ..Default::default()
        },
    )
    .await
    .expect("Claude run should start");

    let mut events = handle.events;
    let _seen = drain_to_none(events.as_mut(), Duration::from_secs(2)).await;

    let completion = tokio::time::timeout(Duration::from_secs(2), handle.completion)
        .await
        .expect("completion resolves")
        .expect("run completes successfully");
    assert!(
        completion.status.success(),
        "expected successful fake Claude run, events: {:?}",
        _seen
    );
}

#[tokio::test]
async fn fresh_run_emits_one_variadic_add_dir_group_before_verbose() {
    let prompt = "hello world";
    let temp = tempdir().expect("tempdir");
    let dir_a = temp.path().join("alpha");
    let dir_b = temp.path().join("beta");
    fs::create_dir_all(&dir_a).expect("alpha dir");
    fs::create_dir_all(&dir_b).expect("beta dir");
    let add_dirs = vec![dir_a, dir_b];

    let env = expected_add_dirs_env(&add_dirs);
    let extensions = [(
        "agent_api.exec.add_dirs.v1".to_string(),
        add_dirs_json(&add_dirs),
    )]
    .into_iter()
    .collect();

    run_claude_assertion(prompt, "fresh_assert", env, extensions).await;
}

#[tokio::test]
async fn fresh_run_accepts_absolute_add_dirs_without_effective_working_dir() {
    let prompt = "hello world";
    let temp = tempdir().expect("tempdir");
    let absolute_dir = temp.path().join("shared-context");
    fs::create_dir_all(&absolute_dir).expect("absolute dir");

    let env = expected_add_dirs_env(std::slice::from_ref(&absolute_dir));
    let config = ClaudeCodeBackendConfig {
        binary: Some(fake_claude_binary()),
        env,
        ..Default::default()
    };
    let extensions = [(
        "agent_api.exec.add_dirs.v1".to_string(),
        add_dirs_json(std::slice::from_ref(&absolute_dir)),
    )]
    .into_iter()
    .collect();

    run_claude_assertion_with_adapter(prompt, "fresh_assert", config, None, None, extensions).await;
}

#[tokio::test]
async fn fresh_run_omits_add_dir_when_extension_key_is_absent() {
    run_claude_assertion(
        "hello world",
        "fresh_assert",
        expect_no_add_dir_env(),
        BTreeMap::new(),
    )
    .await;
}

#[tokio::test]
async fn resume_last_keeps_add_dir_group_before_continue() {
    let prompt = "hello world";
    let temp = tempdir().expect("tempdir");
    let dir_a = temp.path().join("alpha");
    let dir_b = temp.path().join("beta");
    fs::create_dir_all(&dir_a).expect("alpha dir");
    fs::create_dir_all(&dir_b).expect("beta dir");
    let add_dirs = vec![dir_a, dir_b];

    let env = expected_add_dirs_env(&add_dirs);
    let extensions = [
        (
            "agent_api.exec.add_dirs.v1".to_string(),
            add_dirs_json(&add_dirs),
        ),
        (
            "agent_api.session.resume.v1".to_string(),
            serde_json::json!({"selector": "last"}),
        ),
    ]
    .into_iter()
    .collect();

    run_claude_assertion(prompt, "resume_last_assert", env, extensions).await;
}

#[tokio::test]
async fn resume_id_keeps_add_dir_group_before_resume_flag() {
    let prompt = "hello world";
    let resume_id = "sess-123";
    let temp = tempdir().expect("tempdir");
    let dir_a = temp.path().join("alpha");
    let dir_b = temp.path().join("beta");
    fs::create_dir_all(&dir_a).expect("alpha dir");
    fs::create_dir_all(&dir_b).expect("beta dir");
    let add_dirs = vec![dir_a, dir_b];

    let mut env = expected_add_dirs_env(&add_dirs);
    env.insert(
        "FAKE_CLAUDE_EXPECT_RESUME_ID".to_string(),
        resume_id.to_string(),
    );
    let extensions = [
        (
            "agent_api.exec.add_dirs.v1".to_string(),
            add_dirs_json(&add_dirs),
        ),
        (
            "agent_api.session.resume.v1".to_string(),
            serde_json::json!({"selector": "id", "id": resume_id}),
        ),
    ]
    .into_iter()
    .collect();

    run_claude_assertion(prompt, "resume_id_assert", env, extensions).await;
}

#[tokio::test]
async fn fork_last_keeps_add_dir_group_before_continue_and_fork_session() {
    let prompt = "hello world";
    let temp = tempdir().expect("tempdir");
    let dir_a = temp.path().join("alpha");
    let dir_b = temp.path().join("beta");
    fs::create_dir_all(&dir_a).expect("alpha dir");
    fs::create_dir_all(&dir_b).expect("beta dir");
    let add_dirs = vec![dir_a, dir_b];

    let env = expected_add_dirs_env(&add_dirs);
    let extensions = [
        (
            "agent_api.exec.add_dirs.v1".to_string(),
            add_dirs_json(&add_dirs),
        ),
        (
            "agent_api.session.fork.v1".to_string(),
            serde_json::json!({"selector": "last"}),
        ),
    ]
    .into_iter()
    .collect();

    run_claude_assertion(prompt, "fork_last_assert", env, extensions).await;
}

#[tokio::test]
async fn fork_id_keeps_add_dir_group_before_fork_session_and_resume() {
    let prompt = "hello world";
    let fork_id = "sess-123";
    let temp = tempdir().expect("tempdir");
    let dir_a = temp.path().join("alpha");
    let dir_b = temp.path().join("beta");
    fs::create_dir_all(&dir_a).expect("alpha dir");
    fs::create_dir_all(&dir_b).expect("beta dir");
    let add_dirs = vec![dir_a, dir_b];

    let mut env = expected_add_dirs_env(&add_dirs);
    env.insert(
        "FAKE_CLAUDE_EXPECT_RESUME_ID".to_string(),
        fork_id.to_string(),
    );
    let extensions = [
        (
            "agent_api.exec.add_dirs.v1".to_string(),
            add_dirs_json(&add_dirs),
        ),
        (
            "agent_api.session.fork.v1".to_string(),
            serde_json::json!({"selector": "id", "id": fork_id}),
        ),
    ]
    .into_iter()
    .collect();

    run_claude_assertion(prompt, "fork_id_assert", env, extensions).await;
}

#[tokio::test]
async fn fresh_run_resolves_relative_request_working_dir_before_add_dirs_and_spawn() {
    let prompt = "hello world";
    let temp = tempdir().expect("tempdir");
    let run_start_cwd = temp.path().join("run-start");
    let expected_cwd = run_start_cwd.join("repo");
    let expected_add_dir = expected_cwd.join("docs");
    fs::create_dir_all(&expected_add_dir).expect("create add-dir target");

    let env = expected_add_dirs_env(std::slice::from_ref(&expected_add_dir))
        .into_iter()
        .chain([(
            "FAKE_CLAUDE_EXPECT_CWD".to_string(),
            expected_cwd.display().to_string(),
        )])
        .collect();
    let config = ClaudeCodeBackendConfig {
        binary: Some(fake_claude_binary()),
        env,
        ..Default::default()
    };
    let extensions = [(
        "agent_api.exec.add_dirs.v1".to_string(),
        add_dirs_payload(&["docs"]),
    )]
    .into_iter()
    .collect();

    run_claude_assertion_with_adapter(
        prompt,
        "fresh_assert",
        config,
        Some(run_start_cwd),
        Some(PathBuf::from("repo")),
        extensions,
    )
    .await;
}

#[tokio::test]
async fn fresh_run_resolves_relative_default_working_dir_before_add_dirs_and_spawn() {
    let prompt = "hello world";
    let temp = tempdir().expect("tempdir");
    let run_start_cwd = temp.path().join("run-start");
    let expected_cwd = run_start_cwd.join("repo");
    let expected_add_dir = expected_cwd.join("docs");
    fs::create_dir_all(&expected_add_dir).expect("create add-dir target");

    let env = expected_add_dirs_env(std::slice::from_ref(&expected_add_dir))
        .into_iter()
        .chain([(
            "FAKE_CLAUDE_EXPECT_CWD".to_string(),
            expected_cwd.display().to_string(),
        )])
        .collect();
    let config = ClaudeCodeBackendConfig {
        binary: Some(fake_claude_binary()),
        default_working_dir: Some(PathBuf::from("repo")),
        env,
        ..Default::default()
    };
    let extensions = [(
        "agent_api.exec.add_dirs.v1".to_string(),
        add_dirs_payload(&["docs"]),
    )]
    .into_iter()
    .collect();

    run_claude_assertion_with_adapter(
        prompt,
        "fresh_assert",
        config,
        Some(run_start_cwd),
        None,
        extensions,
    )
    .await;
}

#[cfg(windows)]
#[tokio::test]
async fn fresh_run_deduplicates_case_insensitive_add_dirs_before_spawn() {
    let prompt = "hello world";
    let temp = tempdir().expect("tempdir");
    let run_start_cwd = temp.path().join("run-start");
    let expected_cwd = run_start_cwd.join("repo");
    let expected_add_dir = expected_cwd.join("docs");
    fs::create_dir_all(&expected_add_dir).expect("create add-dir target");

    let env = expected_add_dirs_env(std::slice::from_ref(&expected_add_dir))
        .into_iter()
        .chain([(
            "FAKE_CLAUDE_EXPECT_CWD".to_string(),
            expected_cwd.display().to_string(),
        )])
        .collect();
    let config = ClaudeCodeBackendConfig {
        binary: Some(fake_claude_binary()),
        env,
        ..Default::default()
    };
    let extensions = [(
        "agent_api.exec.add_dirs.v1".to_string(),
        add_dirs_payload(&["docs", "DOCS"]),
    )]
    .into_iter()
    .collect();

    run_claude_assertion_with_adapter(
        prompt,
        "fresh_assert",
        config,
        Some(run_start_cwd),
        Some(PathBuf::from("repo")),
        extensions,
    )
    .await;
}
