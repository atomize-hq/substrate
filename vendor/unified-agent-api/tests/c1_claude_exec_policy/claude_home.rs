use std::{ffi::OsString, time::Duration};

use super::support::*;

#[tokio::test]
async fn claude_home_redirects_wrapper_managed_user_home_env_and_materializes_layout() {
    let home_root = unique_missing_dir_path("claude_home_redirect_root");
    let snapshot_path = unique_temp_log_path("claude_home_env_snapshot");

    let backend = ClaudeCodeBackend::new(ClaudeCodeBackendConfig {
        binary: Some(fake_claude_binary()),
        claude_home: Some(home_root.clone()),
        env: [
            (
                "FAKE_CLAUDE_SCENARIO".to_string(),
                "claude_home_env_snapshot".to_string(),
            ),
            (
                "FAKE_CLAUDE_ENV_SNAPSHOT_PATH".to_string(),
                snapshot_path.to_string_lossy().to_string(),
            ),
        ]
        .into_iter()
        .collect(),
        ..Default::default()
    });

    let handle = backend
        .run(AgentWrapperRunRequest {
            prompt: "hello".to_string(),
            ..Default::default()
        })
        .await
        .expect("run");

    let mut events = handle.events;
    let completion = handle.completion;

    let seen = drain_to_none(events.as_mut(), Duration::from_secs(2)).await;
    assert!(
        seen.iter()
            .any(|ev| ev.kind == AgentWrapperEventKind::Status),
        "expected at least one Status event"
    );

    let completion = tokio::time::timeout(Duration::from_secs(2), completion)
        .await
        .expect("completion resolves")
        .expect("completion ok");
    assert!(completion.status.success());

    let envs = read_env_snapshot(&snapshot_path);
    let layout = ClaudeHomeLayout::new(&home_root);

    assert_eq!(
        envs.get("CLAUDE_HOME").map(String::as_str),
        Some(layout.root().to_str().expect("utf-8 claude home root"))
    );
    assert_eq!(
        envs.get("HOME").map(String::as_str),
        Some(layout.root().to_str().expect("utf-8 home root"))
    );
    assert_eq!(
        envs.get("XDG_CONFIG_HOME").map(String::as_str),
        Some(
            layout
                .xdg_config_home()
                .to_str()
                .expect("utf-8 xdg config home")
        )
    );
    assert_eq!(
        envs.get("XDG_DATA_HOME").map(String::as_str),
        Some(
            layout
                .xdg_data_home()
                .to_str()
                .expect("utf-8 xdg data home")
        )
    );
    assert_eq!(
        envs.get("XDG_CACHE_HOME").map(String::as_str),
        Some(
            layout
                .xdg_cache_home()
                .to_str()
                .expect("utf-8 xdg cache home")
        )
    );

    assert!(
        layout.root().is_dir(),
        "expected isolated home root to exist"
    );
    assert!(
        layout.xdg_config_home().is_dir(),
        "expected XDG config home to exist"
    );
    assert!(
        layout.xdg_data_home().is_dir(),
        "expected XDG data home to exist"
    );
    assert!(
        layout.xdg_cache_home().is_dir(),
        "expected XDG cache home to exist"
    );
}

#[tokio::test]
async fn claude_home_request_env_overrides_win_and_parent_env_is_unchanged() {
    let key = "C1_CLAUDE_PARENT_ENV_SENTINEL";
    let previous = std::env::var_os(key);
    std::env::set_var(key, "original");
    let _guard = EnvGuard { key, previous };

    let home_root = unique_missing_dir_path("claude_home_request_override_root");
    let snapshot_path = unique_temp_log_path("claude_home_request_override_snapshot");
    let override_home = unique_missing_dir_path("claude_home_request_override_home");
    let override_xdg_config = unique_missing_dir_path("claude_home_request_override_xdg_config");

    let backend = ClaudeCodeBackend::new(ClaudeCodeBackendConfig {
        binary: Some(fake_claude_binary()),
        claude_home: Some(home_root.clone()),
        env: [
            (
                "FAKE_CLAUDE_SCENARIO".to_string(),
                "claude_home_env_snapshot".to_string(),
            ),
            (
                "FAKE_CLAUDE_ENV_SNAPSHOT_PATH".to_string(),
                snapshot_path.to_string_lossy().to_string(),
            ),
        ]
        .into_iter()
        .collect(),
        ..Default::default()
    });

    let handle = backend
        .run(AgentWrapperRunRequest {
            prompt: "hello".to_string(),
            env: [
                (
                    "HOME".to_string(),
                    override_home.to_string_lossy().to_string(),
                ),
                (
                    "XDG_CONFIG_HOME".to_string(),
                    override_xdg_config.to_string_lossy().to_string(),
                ),
            ]
            .into_iter()
            .collect(),
            ..Default::default()
        })
        .await
        .expect("run");

    let mut events = handle.events;
    let completion = handle.completion;

    let seen = drain_to_none(events.as_mut(), Duration::from_secs(2)).await;
    assert!(
        seen.iter()
            .any(|ev| ev.kind == AgentWrapperEventKind::Status),
        "expected at least one Status event"
    );

    let completion = tokio::time::timeout(Duration::from_secs(2), completion)
        .await
        .expect("completion resolves")
        .expect("completion ok");
    assert!(completion.status.success());

    let envs = read_env_snapshot(&snapshot_path);
    let layout = ClaudeHomeLayout::new(&home_root);

    assert_eq!(
        envs.get("CLAUDE_HOME").map(String::as_str),
        Some(layout.root().to_str().expect("utf-8 claude home root"))
    );
    assert_eq!(
        envs.get("HOME").map(String::as_str),
        Some(override_home.to_str().expect("utf-8 override home"))
    );
    assert_eq!(
        envs.get("XDG_CONFIG_HOME").map(String::as_str),
        Some(
            override_xdg_config
                .to_str()
                .expect("utf-8 override xdg config")
        )
    );
    assert_eq!(
        envs.get("XDG_DATA_HOME").map(String::as_str),
        Some(
            layout
                .xdg_data_home()
                .to_str()
                .expect("utf-8 xdg data home")
        )
    );
    assert_eq!(
        envs.get("XDG_CACHE_HOME").map(String::as_str),
        Some(
            layout
                .xdg_cache_home()
                .to_str()
                .expect("utf-8 xdg cache home")
        )
    );

    assert!(
        layout.root().is_dir(),
        "expected isolated home root to exist"
    );
    assert!(
        layout.xdg_config_home().is_dir(),
        "expected XDG config home to exist"
    );
    assert!(
        layout.xdg_data_home().is_dir(),
        "expected XDG data home to exist"
    );
    assert!(
        layout.xdg_cache_home().is_dir(),
        "expected XDG cache home to exist"
    );
    assert_eq!(
        std::env::var_os(key),
        Some(OsString::from("original")),
        "expected backend to not mutate parent process environment"
    );
}
