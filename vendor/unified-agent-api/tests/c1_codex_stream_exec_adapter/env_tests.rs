use std::collections::BTreeMap;

use super::*;

#[tokio::test]
async fn request_env_overrides_config_env_and_parent_env_is_unchanged() {
    struct EnvGuard {
        key: &'static str,
        previous: Option<String>,
    }

    impl Drop for EnvGuard {
        fn drop(&mut self) {
            if let Some(value) = self.previous.as_ref() {
                std::env::set_var(self.key, value);
            } else {
                std::env::remove_var(self.key);
            }
        }
    }

    let key = "C1_PARENT_ENV_SENTINEL";
    let previous = std::env::var(key).ok();
    std::env::set_var(key, "original");
    let _guard = EnvGuard { key, previous };

    let backend = CodexBackend::new(CodexBackendConfig {
        binary: Some(fake_codex_binary()),
        env: [
            ("FAKE_CODEX_SCENARIO".to_string(), "env_assert".to_string()),
            ("C1_TEST_KEY".to_string(), "config".to_string()),
            ("C1_ONLY_CONFIG".to_string(), "config-only".to_string()),
            (
                "FAKE_CODEX_ASSERT_ENV_C1_TEST_KEY".to_string(),
                "request".to_string(),
            ),
            (
                "FAKE_CODEX_ASSERT_ENV_C1_ONLY_CONFIG".to_string(),
                "config-only".to_string(),
            ),
        ]
        .into_iter()
        .collect(),
        ..Default::default()
    });

    let handle = backend
        .run(AgentWrapperRunRequest {
            prompt: "hello".to_string(),
            env: [("C1_TEST_KEY".to_string(), "request".to_string())]
                .into_iter()
                .collect::<BTreeMap<_, _>>(),
            ..Default::default()
        })
        .await
        .unwrap();

    let mut events = handle.events;
    let completion = handle.completion;
    let _ = drain_to_none(events.as_mut(), Duration::from_secs(2)).await;

    let completion = tokio::time::timeout(Duration::from_secs(2), completion)
        .await
        .expect("completion resolves")
        .unwrap();
    assert!(completion.status.success());

    assert_eq!(
        std::env::var(key).ok().as_deref(),
        Some("original"),
        "expected backend to not mutate parent process environment"
    );
}

#[tokio::test]
async fn request_env_override_wins_over_codex_home_injection_and_parent_codex_home_is_unchanged() {
    let original_codex_home = std::env::var_os("CODEX_HOME");

    let injected_home = unique_missing_dir_path("codex_home_injected_root");
    let override_home = unique_missing_dir_path("codex_home_override_root");

    let backend = CodexBackend::new(CodexBackendConfig {
        binary: Some(fake_codex_binary()),
        codex_home: Some(injected_home),
        env: [
            ("FAKE_CODEX_SCENARIO".to_string(), "env_assert".to_string()),
            ("C1_ISOLATED_KEY".to_string(), "config".to_string()),
            (
                "C1_ISOLATED_CONFIG_ONLY".to_string(),
                "config-only".to_string(),
            ),
            (
                "FAKE_CODEX_ASSERT_ENV_CODEX_HOME".to_string(),
                override_home.to_string_lossy().to_string(),
            ),
            (
                "FAKE_CODEX_ASSERT_ENV_C1_ISOLATED_KEY".to_string(),
                "request".to_string(),
            ),
            (
                "FAKE_CODEX_ASSERT_ENV_C1_ISOLATED_CONFIG_ONLY".to_string(),
                "config-only".to_string(),
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
                    "CODEX_HOME".to_string(),
                    override_home.to_string_lossy().to_string(),
                ),
                ("C1_ISOLATED_KEY".to_string(), "request".to_string()),
            ]
            .into_iter()
            .collect::<BTreeMap<_, _>>(),
            ..Default::default()
        })
        .await
        .unwrap();

    let mut events = handle.events;
    let completion = handle.completion;
    let _ = drain_to_none(events.as_mut(), Duration::from_secs(2)).await;

    let completion = tokio::time::timeout(Duration::from_secs(2), completion)
        .await
        .expect("completion resolves")
        .unwrap();
    assert!(completion.status.success());

    assert_eq!(
        std::env::var_os("CODEX_HOME"),
        original_codex_home,
        "expected backend to not mutate parent CODEX_HOME"
    );
}
