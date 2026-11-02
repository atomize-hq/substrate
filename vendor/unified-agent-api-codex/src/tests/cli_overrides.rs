use super::*;

#[test]
fn reasoning_config_by_model() {
    assert_eq!(
        reasoning_config_for(Some("gpt-5")).unwrap(),
        DEFAULT_REASONING_CONFIG_GPT5
    );
    assert_eq!(
        reasoning_config_for(Some("gpt-5.1-codex-max")).unwrap(),
        DEFAULT_REASONING_CONFIG_GPT5_1
    );
    assert_eq!(
        reasoning_config_for(Some("gpt-5-codex")).unwrap(),
        DEFAULT_REASONING_CONFIG_GPT5_CODEX
    );
    assert!(reasoning_config_for(None).is_none());
    assert!(reasoning_config_for(Some("gpt-4.1-mini")).is_none());
}

#[test]
fn resolve_cli_overrides_respects_reasoning_defaults() {
    let builder = CliOverrides::default();
    let patch = CliOverridesPatch::default();

    let resolved = resolve_cli_overrides(&builder, &patch, Some("gpt-5"));
    let keys: Vec<_> = resolved
        .config_overrides
        .iter()
        .map(|override_| override_.key.as_str())
        .collect();
    assert!(keys.contains(&"model_reasoning_effort"));
    assert!(keys.contains(&"model_reasoning_summary"));
    assert!(keys.contains(&"model_verbosity"));

    let resolved_without_model = resolve_cli_overrides(&builder, &patch, None);
    assert!(resolved_without_model.config_overrides.is_empty());
}

#[test]
fn explicit_reasoning_overrides_disable_defaults() {
    let mut builder = CliOverrides::default();
    builder
        .config_overrides
        .push(ConfigOverride::new("model_reasoning_effort", "high"));

    let resolved = resolve_cli_overrides(&builder, &CliOverridesPatch::default(), Some("gpt-5"));
    assert_eq!(resolved.config_overrides.len(), 1);
    assert_eq!(resolved.config_overrides[0].value, "high");
}

#[test]
fn request_can_disable_auto_reasoning_defaults() {
    let builder = CliOverrides::default();
    let patch = CliOverridesPatch {
        auto_reasoning_defaults: Some(false),
        ..Default::default()
    };

    let resolved = resolve_cli_overrides(&builder, &patch, Some("gpt-5"));
    assert!(resolved.config_overrides.is_empty());
}

#[test]
fn request_config_overrides_follow_builder_order() {
    let mut builder_overrides = CliOverrides {
        auto_reasoning_defaults: false,
        ..Default::default()
    };
    builder_overrides
        .config_overrides
        .push(ConfigOverride::new("foo", "bar"));

    let mut patch = CliOverridesPatch::default();
    patch
        .config_overrides
        .push(ConfigOverride::new("foo", "baz"));

    let resolved = resolve_cli_overrides(&builder_overrides, &patch, None);
    let values: Vec<_> = resolved
        .config_overrides
        .iter()
        .map(|override_| override_.value.as_str())
        .collect();
    assert_eq!(values, vec!["bar", "baz"]);
}

#[test]
fn request_search_override_can_disable_builder_flag() {
    let builder_overrides = CliOverrides {
        search: FlagState::Enable,
        ..Default::default()
    };

    let patch = CliOverridesPatch {
        search: FlagState::Disable,
        ..Default::default()
    };

    let resolved = resolve_cli_overrides(&builder_overrides, &patch, None);
    let args = cli_override_args(&resolved, true);
    let args: Vec<_> = args
        .iter()
        .map(|arg| arg.to_string_lossy().into_owned())
        .collect();
    assert!(!args.contains(&"--search".to_string()));
}

#[test]
fn request_profile_override_replaces_builder_value() {
    let builder_overrides = CliOverrides {
        profile: Some("builder".to_string()),
        ..Default::default()
    };

    let patch = CliOverridesPatch {
        profile: Some("request".to_string()),
        ..Default::default()
    };

    let resolved = resolve_cli_overrides(&builder_overrides, &patch, None);
    let args: Vec<_> = cli_override_args(&resolved, true)
        .iter()
        .map(|arg| arg.to_string_lossy().into_owned())
        .collect();
    assert!(args.windows(2).any(|window| {
        window.first().map(String::as_str) == Some("--profile")
            && window.get(1).map(String::as_str) == Some("request")
    }));
    assert!(!args.contains(&"builder".to_string()));
}

#[test]
fn request_oss_override_can_disable_builder_flag() {
    let builder_overrides = CliOverrides {
        oss: FlagState::Enable,
        ..Default::default()
    };

    let resolved = resolve_cli_overrides(&builder_overrides, &CliOverridesPatch::default(), None);
    let args: Vec<_> = cli_override_args(&resolved, true)
        .iter()
        .map(|arg| arg.to_string_lossy().into_owned())
        .collect();
    assert!(args.contains(&"--oss".to_string()));

    let patch = CliOverridesPatch {
        oss: FlagState::Disable,
        ..Default::default()
    };
    let resolved = resolve_cli_overrides(&builder_overrides, &patch, None);
    let args: Vec<_> = cli_override_args(&resolved, true)
        .iter()
        .map(|arg| arg.to_string_lossy().into_owned())
        .collect();
    assert!(!args.contains(&"--oss".to_string()));
}

#[test]
fn feature_toggles_merge_builder_and_request() {
    let mut builder_overrides = CliOverrides::default();
    builder_overrides
        .feature_toggles
        .enable
        .push("builder-enable".to_string());
    builder_overrides
        .feature_toggles
        .disable
        .push("builder-disable".to_string());

    let mut patch = CliOverridesPatch::default();
    patch
        .feature_toggles
        .enable
        .push("request-enable".to_string());
    patch
        .feature_toggles
        .disable
        .push("request-disable".to_string());

    let resolved = resolve_cli_overrides(&builder_overrides, &patch, None);
    let args: Vec<_> = cli_override_args(&resolved, true)
        .iter()
        .map(|arg| arg.to_string_lossy().into_owned())
        .collect();

    assert!(args.windows(2).any(|window| {
        window.first().map(String::as_str) == Some("--enable")
            && window.get(1).map(String::as_str) == Some("builder-enable")
    }));
    assert!(args.windows(2).any(|window| {
        window.first().map(String::as_str) == Some("--enable")
            && window.get(1).map(String::as_str) == Some("request-enable")
    }));
    assert!(args.windows(2).any(|window| {
        window.first().map(String::as_str) == Some("--disable")
            && window.get(1).map(String::as_str) == Some("builder-disable")
    }));
    assert!(args.windows(2).any(|window| {
        window.first().map(String::as_str) == Some("--disable")
            && window.get(1).map(String::as_str) == Some("request-disable")
    }));
}

#[test]
fn cli_override_args_apply_safety_precedence() {
    let mut resolved = ResolvedCliOverrides {
        config_overrides: Vec::new(),
        feature_toggles: FeatureToggles::default(),
        approval_policy: None,
        sandbox_mode: None,
        safety_override: SafetyOverride::FullAuto,
        profile: None,
        cd: None,
        remote: None,
        remote_auth_token_env: None,
        local_provider: None,
        oss: false,
        search: FlagState::Enable,
    };
    let args = cli_override_args(&resolved, true);
    let args: Vec<_> = args
        .iter()
        .map(|value| value.to_string_lossy().into_owned())
        .collect();
    assert!(args.contains(&"--full-auto".to_string()));
    assert!(args.contains(&"--search".to_string()));
    assert!(!args.contains(&"--ask-for-approval".to_string()));

    resolved.approval_policy = Some(ApprovalPolicy::OnRequest);
    let args_with_policy = cli_override_args(&resolved, true);
    let args_with_policy: Vec<_> = args_with_policy
        .iter()
        .map(|value| value.to_string_lossy().into_owned())
        .collect();
    assert!(!args_with_policy.contains(&"--full-auto".to_string()));
    assert!(args_with_policy.contains(&"--ask-for-approval".to_string()));

    let resolved = ResolvedCliOverrides {
        config_overrides: vec![ConfigOverride::new("foo", "bar")],
        feature_toggles: FeatureToggles::default(),
        approval_policy: Some(ApprovalPolicy::OnRequest),
        sandbox_mode: Some(SandboxMode::WorkspaceWrite),
        safety_override: SafetyOverride::DangerouslyBypass,
        profile: Some("team".to_string()),
        cd: Some(PathBuf::from("/tmp/worktree")),
        remote: Some("staging".to_string()),
        remote_auth_token_env: Some("CODEX_REMOTE_TOKEN".to_string()),
        local_provider: Some(LocalProvider::Ollama),
        oss: false,
        search: FlagState::Enable,
    };
    let args = cli_override_args(&resolved, true);
    let args: Vec<_> = args
        .iter()
        .map(|value| value.to_string_lossy().into_owned())
        .collect();
    assert!(args.contains(&"--config".to_string()));
    assert!(args.contains(&"foo=bar".to_string()));
    assert!(args.contains(&"--dangerously-bypass-approvals-and-sandbox".to_string()));
    assert!(args.contains(&"--profile".to_string()));
    assert!(args.contains(&"team".to_string()));
    assert!(args.contains(&"--cd".to_string()));
    assert!(args.contains(&"/tmp/worktree".to_string()));
    assert!(args.contains(&"--remote".to_string()));
    assert!(args.contains(&"staging".to_string()));
    assert!(args.contains(&"--remote-auth-token-env".to_string()));
    assert!(args.contains(&"CODEX_REMOTE_TOKEN".to_string()));
    assert!(args.contains(&"--local-provider".to_string()));
    assert!(args.contains(&"ollama".to_string()));
    assert!(args.contains(&"--search".to_string()));
    assert!(!args.contains(&"--ask-for-approval".to_string()));
    assert!(!args.contains(&"--sandbox".to_string()));

    let args_without_search = cli_override_args(&resolved, false);
    let args_without_search: Vec<_> = args_without_search
        .iter()
        .map(|value| value.to_string_lossy().into_owned())
        .collect();
    assert!(!args_without_search.contains(&"--search".to_string()));
}

#[tokio::test]
async fn exec_applies_cli_overrides_and_request_patch() {
    let _guard = env_guard_async().await;
    clear_capability_cache();

    let temp = tempfile::tempdir().unwrap();
    let log_path = temp.path().join("exec.log");
    let builder_cd = temp.path().join("builder-cd");
    let request_cd = temp.path().join("request-cd");
    let script = format!(
        r#"#!/bin/bash
echo "$@" >> "{log}"
if printf '%s\n' "$@" | grep -qx "exec"; then
  echo "ok"
fi
"#,
        log = log_path.display()
    );
    let binary = write_fake_codex(temp.path(), &script);
    let client = CodexClient::builder()
        .binary(&binary)
        .timeout(Duration::from_secs(5))
        .mirror_stdout(false)
        .quiet(true)
        .auto_reasoning_defaults(false)
        .config_override("foo", "bar")
        .reasoning_summary(ReasoningSummary::Concise)
        .approval_policy(ApprovalPolicy::OnRequest)
        .sandbox_mode(SandboxMode::WorkspaceWrite)
        .cd(&builder_cd)
        .local_provider(LocalProvider::Custom)
        .oss(true)
        .enable_feature("builder-on")
        .disable_feature("builder-off")
        .search(true)
        .build();

    let mut request = ExecRequest::new("list flags")
        .config_override("extra", "value")
        .oss(false)
        .enable_feature("request-on")
        .disable_feature("request-off")
        .search(false);
    request.overrides.cd = Some(request_cd.clone());
    request.overrides.safety_override = Some(SafetyOverride::DangerouslyBypass);

    let response = client.send_prompt_with(request).await.unwrap();
    assert_eq!(response.trim(), "ok");

    let logged = std_fs::read_to_string(&log_path).unwrap();
    assert!(logged.contains("--config"));
    assert!(logged.contains("foo=bar"));
    assert!(logged.contains("extra=value"));
    assert!(logged.contains("model_reasoning_summary=concise"));
    assert!(logged.contains("--dangerously-bypass-approvals-and-sandbox"));
    assert!(logged.contains(&request_cd.display().to_string()));
    assert!(!logged.contains(&builder_cd.display().to_string()));
    assert!(logged.contains("--local-provider"));
    assert!(logged.contains("custom"));
    assert!(logged.contains("--enable"));
    assert!(logged.contains("builder-on"));
    assert!(logged.contains("request-on"));
    assert!(logged.contains("--disable"));
    assert!(logged.contains("builder-off"));
    assert!(logged.contains("request-off"));
    assert!(!logged.contains("--oss"));
    assert!(!logged.contains("--ask-for-approval"));
    assert!(!logged.contains("--sandbox"));
    assert!(!logged.contains("--search"));
}

#[tokio::test]
async fn resume_applies_search_and_selector_overrides() {
    let _guard = env_guard_async().await;
    clear_capability_cache();

    let temp = tempfile::tempdir().unwrap();
    let log_path = temp.path().join("resume.log");
    let builder_cd = temp.path().join("builder-cd");
    let request_cd = temp.path().join("request-cd");
    let script = format!(
        r#"#!/bin/bash
echo "$@" >> "{log}"
if printf '%s\n' "$@" | grep -qx "exec"; then
  echo '{{"type":"thread.started","thread_id":"thread-1"}}'
  echo '{{"type":"turn.started","thread_id":"thread-1","turn_id":"turn-1"}}'
  echo '{{"type":"turn.completed","thread_id":"thread-1","turn_id":"turn-1"}}'
fi
"#,
        log = log_path.display()
    );
    let binary = write_fake_codex(temp.path(), &script);
    let client = CodexClient::builder()
        .binary(&binary)
        .timeout(Duration::from_secs(5))
        .mirror_stdout(false)
        .quiet(true)
        .config_override("resume_hint", "enabled")
        .approval_policy(ApprovalPolicy::OnRequest)
        .sandbox_mode(SandboxMode::WorkspaceWrite)
        .local_provider(LocalProvider::Ollama)
        .cd(&builder_cd)
        .search(true)
        .build();

    let request_last = ResumeRequest::last().prompt("continue");
    let stream = client.stream_resume(request_last).await.unwrap();
    let events: Vec<_> = stream.events.collect().await;
    assert_eq!(events.len(), 3);
    stream.completion.await.unwrap();

    let mut request_all = ResumeRequest::all().prompt("summarize");
    request_all.overrides.search = FlagState::Disable;
    request_all.overrides.safety_override = Some(SafetyOverride::DangerouslyBypass);
    request_all.overrides.cd = Some(request_cd.clone());
    let stream_all = client.stream_resume(request_all).await.unwrap();
    let _ = stream_all.events.collect::<Vec<_>>().await;
    stream_all.completion.await.unwrap();

    let logged: Vec<_> = std_fs::read_to_string(&log_path)
        .unwrap()
        .lines()
        .map(str::to_string)
        .collect();
    assert!(logged.len() >= 2);

    assert!(logged[0].contains("--last"));
    assert!(logged[0].contains("--search"));
    assert!(logged[0].contains("resume_hint=enabled"));
    assert!(logged[0].contains("--ask-for-approval"));
    assert!(logged[0].contains("--sandbox"));
    assert!(logged[0].contains(&builder_cd.display().to_string()));
    assert!(logged[0].contains("ollama"));

    assert!(logged[1].contains("--all"));
    assert!(logged[1].contains("--dangerously-bypass-approvals-and-sandbox"));
    assert!(logged[1].contains(&request_cd.display().to_string()));
    assert!(!logged[1].contains(&builder_cd.display().to_string()));
    assert!(!logged[1].contains("--ask-for-approval"));
    assert!(!logged[1].contains("--sandbox"));
    assert!(!logged[1].contains("--search"));
}

#[tokio::test]
async fn apply_respects_cli_overrides_without_search() {
    let _guard = env_guard_async().await;
    clear_capability_cache();

    let temp = tempfile::tempdir().unwrap();
    let log_path = temp.path().join("apply.log");
    let script = format!(
        r#"#!/bin/bash
echo "$@" >> "{log}"
if printf '%s\n' "$@" | grep -qx "apply"; then
  echo "applied"
fi
"#,
        log = log_path.display()
    );
    let binary = write_fake_codex(temp.path(), &script);
    let client = CodexClient::builder()
        .binary(&binary)
        .timeout(Duration::from_secs(5))
        .mirror_stdout(false)
        .quiet(true)
        .cd(temp.path().join("apply-cd"))
        .config_override("feature.toggle", "true")
        .search(true)
        .build();

    let artifacts = client.apply().await.unwrap();
    assert_eq!(artifacts.stdout.trim(), "applied");

    let logged = std_fs::read_to_string(&log_path).unwrap();
    assert!(logged.contains("--config"));
    assert!(logged.contains("feature.toggle=true"));
    assert!(logged.contains("apply-cd"));
    assert!(!logged.contains("--search"));
}

#[test]
fn color_mode_strings_are_stable() {
    assert_eq!(ColorMode::Auto.as_str(), "auto");
    assert_eq!(ColorMode::Always.as_str(), "always");
    assert_eq!(ColorMode::Never.as_str(), "never");
}
