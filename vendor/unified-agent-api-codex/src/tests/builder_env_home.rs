use super::*;

#[test]
fn builder_defaults_are_sane() {
    let builder = CodexClient::builder();
    assert!(builder.model.is_none());
    assert_eq!(builder.timeout, DEFAULT_TIMEOUT);
    assert_eq!(builder.color_mode, ColorMode::Never);
    assert!(builder.codex_home.is_none());
    assert!(builder.create_home_dirs);
    assert!(builder.working_dir.is_none());
    assert!(builder.images.is_empty());
    assert!(!builder.json_output);
    assert!(!builder.quiet);
    assert!(builder.json_event_log.is_none());
    assert!(builder.cli_overrides.config_overrides.is_empty());
    assert!(!builder.cli_overrides.reasoning.has_overrides());
    assert!(builder.cli_overrides.approval_policy.is_none());
    assert!(builder.cli_overrides.sandbox_mode.is_none());
    assert_eq!(
        builder.cli_overrides.safety_override,
        SafetyOverride::Inherit
    );
    assert!(builder.cli_overrides.cd.is_none());
    assert!(builder.cli_overrides.local_provider.is_none());
    assert_eq!(builder.cli_overrides.search, FlagState::Inherit);
    assert!(builder.cli_overrides.auto_reasoning_defaults);
    assert!(builder.capability_overrides.is_empty());
    assert_eq!(
        builder.capability_cache_policy,
        CapabilityCachePolicy::PreferCache
    );
}

#[test]
fn builder_collects_images() {
    let client = CodexClient::builder()
        .image("foo.png")
        .image("bar.jpg")
        .build();
    assert_eq!(client.images.len(), 2);
    assert_eq!(client.images[0], PathBuf::from("foo.png"));
    assert_eq!(client.images[1], PathBuf::from("bar.jpg"));
}

#[test]
fn builder_sets_json_flag() {
    let client = CodexClient::builder().json(true).build();
    assert!(client.json_output);
}

#[test]
fn builder_sets_json_event_log() {
    let client = CodexClient::builder().json_event_log("events.log").build();
    assert_eq!(client.json_event_log, Some(PathBuf::from("events.log")));
}

#[test]
fn builder_sets_quiet_flag() {
    let client = CodexClient::builder().quiet(true).build();
    assert!(client.quiet);
}

#[test]
fn builder_mirrors_stdout_by_default() {
    let client = CodexClient::builder().build();
    assert!(client.mirror_stdout);
}

#[test]
fn builder_can_disable_stdout_mirroring() {
    let client = CodexClient::builder().mirror_stdout(false).build();
    assert!(!client.mirror_stdout);
}

#[test]
fn builder_uses_env_binary_when_set() {
    let _guard = env_guard();
    let key = CODEX_BINARY_ENV;
    let original = env::var_os(key);
    env::set_var(key, "custom_codex");
    let builder = CodexClient::builder();
    assert_eq!(builder.binary, PathBuf::from("custom_codex"));
    if let Some(value) = original {
        env::set_var(key, value);
    } else {
        env::remove_var(key);
    }
}

#[test]
fn default_binary_falls_back_when_env_missing() {
    let _guard = env_guard();
    let key = CODEX_BINARY_ENV;
    let original = env::var_os(key);
    env::remove_var(key);

    assert_eq!(default_binary_path(), PathBuf::from("codex"));

    if let Some(value) = original {
        env::set_var(key, value);
    } else {
        env::remove_var(key);
    }
}

#[test]
fn default_rust_log_is_error_when_unset() {
    let _guard = env_guard();
    let original = env::var_os("RUST_LOG");
    env::remove_var("RUST_LOG");

    assert_eq!(default_rust_log_value(), Some("error"));

    if let Some(value) = original {
        env::set_var("RUST_LOG", value);
    } else {
        env::remove_var("RUST_LOG");
    }
}

#[test]
fn default_rust_log_respects_existing_env() {
    let _guard = env_guard();
    let original = env::var_os("RUST_LOG");
    env::set_var("RUST_LOG", "info");

    assert_eq!(default_rust_log_value(), None);

    if let Some(value) = original {
        env::set_var("RUST_LOG", value);
    } else {
        env::remove_var("RUST_LOG");
    }
}

#[test]
fn command_env_sets_expected_overrides() {
    let _guard = env_guard();
    let rust_log_original = env::var_os(RUST_LOG_ENV);
    env::remove_var(RUST_LOG_ENV);

    let temp = tempfile::tempdir().unwrap();
    let home = temp.path().join("codex_home");
    let env_prep =
        CommandEnvironment::new(PathBuf::from("/custom/codex"), Some(home.clone()), true);
    let overrides = env_prep.environment_overrides().unwrap();
    let map: HashMap<OsString, OsString> = overrides.into_iter().collect();

    assert_eq!(
        map.get(&OsString::from(CODEX_BINARY_ENV)),
        Some(&OsString::from("/custom/codex"))
    );
    assert_eq!(
        map.get(&OsString::from(CODEX_HOME_ENV)),
        Some(&home.as_os_str().to_os_string())
    );
    assert_eq!(
        map.get(&OsString::from(RUST_LOG_ENV)),
        Some(&OsString::from(DEFAULT_RUST_LOG))
    );

    assert!(home.is_dir());
    assert!(home.join("conversations").is_dir());
    assert!(home.join("logs").is_dir());

    match rust_log_original {
        Some(value) => env::set_var(RUST_LOG_ENV, value),
        None => env::remove_var(RUST_LOG_ENV),
    }
}

#[test]
fn command_env_applies_home_and_binary_per_command() {
    let _guard = env_guard();
    let binary_key = CODEX_BINARY_ENV;
    let home_key = CODEX_HOME_ENV;
    let rust_log_key = RUST_LOG_ENV;
    let original_binary = env::var_os(binary_key);
    let original_home = env::var_os(home_key);
    let original_rust_log = env::var_os(rust_log_key);

    env::set_var(binary_key, "/tmp/ignored_codex");
    env::set_var(home_key, "/tmp/ambient_home");
    env::remove_var(rust_log_key);

    let temp = tempfile::tempdir().unwrap();
    let home = temp.path().join("scoped_home");
    let env_prep = CommandEnvironment::new(
        PathBuf::from("/app/bundled/codex"),
        Some(home.clone()),
        true,
    );

    let mut command = Command::new("echo");
    env_prep.apply(&mut command).unwrap();

    let envs: HashMap<OsString, Option<OsString>> = command
        .as_std()
        .get_envs()
        .map(|(key, value)| (key.to_os_string(), value.map(|v| v.to_os_string())))
        .collect();

    assert_eq!(
        envs.get(&OsString::from(binary_key)),
        Some(&Some(OsString::from("/app/bundled/codex")))
    );
    assert_eq!(
        envs.get(&OsString::from(home_key)),
        Some(&Some(home.as_os_str().to_os_string()))
    );
    assert_eq!(
        envs.get(&OsString::from(rust_log_key)),
        Some(&Some(OsString::from(DEFAULT_RUST_LOG)))
    );
    assert_eq!(
        env::var_os(home_key),
        Some(OsString::from("/tmp/ambient_home"))
    );
    assert!(home.is_dir());
    assert!(home.join("conversations").is_dir());
    assert!(home.join("logs").is_dir());

    match original_binary {
        Some(value) => env::set_var(binary_key, value),
        None => env::remove_var(binary_key),
    }
    match original_home {
        Some(value) => env::set_var(home_key, value),
        None => env::remove_var(home_key),
    }
    match original_rust_log {
        Some(value) => env::set_var(rust_log_key, value),
        None => env::remove_var(rust_log_key),
    }
}

#[cfg(unix)]
#[cfg(unix)]
#[tokio::test]
async fn apply_and_diff_capture_outputs_and_status() {
    let dir = tempfile::tempdir().unwrap();
    let script_path = dir.path().join("codex");
    std::fs::write(
        &script_path,
        r#"#!/usr/bin/env bash
set -e
cmd="$1"
if [[ "$cmd" == "apply" ]]; then
  echo "applied"
  echo "apply-stderr" >&2
  exit 0
elif [[ "$cmd" == "cloud" && "${2:-}" == "diff" ]]; then
  echo "diff-body"
  echo "diff-stderr" >&2
  exit 3
else
  echo "unknown $cmd" >&2
  exit 99
fi
"#,
    )
    .unwrap();
    let mut perms = std::fs::metadata(&script_path).unwrap().permissions();
    perms.set_mode(0o755);
    std::fs::set_permissions(&script_path, perms).unwrap();

    let client = CodexClient::builder()
        .binary(&script_path)
        .mirror_stdout(false)
        .quiet(true)
        .build();

    let apply = client.apply().await.unwrap();
    assert!(apply.status.success());
    assert_eq!(apply.stdout.trim(), "applied");
    assert_eq!(apply.stderr.trim(), "apply-stderr");

    let diff = client.diff().await.unwrap();
    assert!(!diff.status.success());
    assert_eq!(diff.status.code(), Some(3));
    assert_eq!(diff.stdout.trim(), "diff-body");
    assert_eq!(diff.stderr.trim(), "diff-stderr");
}

#[cfg(unix)]
#[tokio::test]
async fn apply_respects_rust_log_default() {
    let _guard = env_guard_async().await;
    let original = env::var_os("RUST_LOG");
    env::remove_var("RUST_LOG");

    let dir = tempfile::tempdir().unwrap();
    let script_path = dir.path().join("codex-rust-log");
    std::fs::write(
        &script_path,
        r#"#!/usr/bin/env bash
echo "${RUST_LOG:-missing}"
exit 0
"#,
    )
    .unwrap();
    let mut perms = std::fs::metadata(&script_path).unwrap().permissions();
    perms.set_mode(0o755);
    std::fs::set_permissions(&script_path, perms).unwrap();

    let client = CodexClient::builder()
        .binary(&script_path)
        .mirror_stdout(false)
        .quiet(true)
        .build();

    let apply = client.apply().await.unwrap();
    assert_eq!(apply.stdout.trim(), "error");

    if let Some(value) = original {
        env::set_var("RUST_LOG", value);
    } else {
        env::remove_var("RUST_LOG");
    }
}

#[test]
fn command_env_respects_existing_rust_log() {
    let _guard = env_guard();
    let rust_log_original = env::var_os(RUST_LOG_ENV);
    env::set_var(RUST_LOG_ENV, "trace");

    let env_prep = CommandEnvironment::new(PathBuf::from("codex"), None, true);
    let overrides = env_prep.environment_overrides().unwrap();
    let map: HashMap<OsString, OsString> = overrides.into_iter().collect();

    assert_eq!(
        map.get(&OsString::from(CODEX_BINARY_ENV)),
        Some(&OsString::from("codex"))
    );
    assert!(!map.contains_key(&OsString::from(RUST_LOG_ENV)));

    match rust_log_original {
        Some(value) => env::set_var(RUST_LOG_ENV, value),
        None => env::remove_var(RUST_LOG_ENV),
    }
}

#[test]
fn command_env_can_skip_home_creation() {
    let _guard = env_guard();
    let rust_log_original = env::var_os(RUST_LOG_ENV);
    env::remove_var(RUST_LOG_ENV);

    let temp = tempfile::tempdir().unwrap();
    let home = temp.path().join("codex_home");
    let env_prep = CommandEnvironment::new(PathBuf::from("codex"), Some(home.clone()), false);
    let overrides = env_prep.environment_overrides().unwrap();
    let map: HashMap<OsString, OsString> = overrides.into_iter().collect();

    assert!(!home.exists());
    assert!(!home.join("conversations").exists());
    assert!(!home.join("logs").exists());
    assert_eq!(
        map.get(&OsString::from(CODEX_HOME_ENV)),
        Some(&home.as_os_str().to_os_string())
    );

    match rust_log_original {
        Some(value) => env::set_var(RUST_LOG_ENV, value),
        None => env::remove_var(RUST_LOG_ENV),
    }
}

#[test]
fn codex_home_layout_exposes_paths() {
    let root = PathBuf::from("/tmp/codex_layout_root");
    let layout = CodexHomeLayout::new(&root);

    assert_eq!(layout.root(), root.as_path());
    assert_eq!(layout.config_path(), root.join("config.toml"));
    assert_eq!(layout.auth_path(), root.join("auth.json"));
    assert_eq!(layout.credentials_path(), root.join(".credentials.json"));
    assert_eq!(layout.history_path(), root.join("history.jsonl"));
    assert_eq!(layout.conversations_dir(), root.join("conversations"));
    assert_eq!(layout.logs_dir(), root.join("logs"));
}

#[test]
fn codex_home_layout_respects_materialization_flag() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().join("codex_home_layout");
    let layout = CodexHomeLayout::new(&root);

    layout.materialize(false).unwrap();
    assert!(!root.exists());

    layout.materialize(true).unwrap();
    assert!(root.is_dir());
    assert!(layout.conversations_dir().is_dir());
    assert!(layout.logs_dir().is_dir());
}

#[test]
fn seed_auth_copies_files_and_creates_targets() {
    let temp = tempfile::tempdir().unwrap();
    let seed = temp.path().join("seed_home");
    std::fs::create_dir_all(&seed).unwrap();
    std::fs::write(seed.join("auth.json"), "auth").unwrap();
    std::fs::write(seed.join(".credentials.json"), "creds").unwrap();

    let target_root = temp.path().join("target_home");
    let layout = CodexHomeLayout::new(&target_root);
    let outcome = layout
        .seed_auth_from(&seed, AuthSeedOptions::default())
        .unwrap();

    assert!(outcome.copied_auth);
    assert!(outcome.copied_credentials);
    assert_eq!(std::fs::read_to_string(layout.auth_path()).unwrap(), "auth");
    assert_eq!(
        std::fs::read_to_string(layout.credentials_path()).unwrap(),
        "creds"
    );
}

#[test]
fn seed_auth_skips_optional_files() {
    let temp = tempfile::tempdir().unwrap();
    let seed = temp.path().join("seed_home");
    std::fs::create_dir_all(&seed).unwrap();
    std::fs::write(seed.join("auth.json"), "auth").unwrap();

    let target_root = temp.path().join("target_home");
    let layout = CodexHomeLayout::new(&target_root);
    let outcome = layout
        .seed_auth_from(&seed, AuthSeedOptions::default())
        .unwrap();

    assert!(outcome.copied_auth);
    assert!(!outcome.copied_credentials);
    assert_eq!(std::fs::read_to_string(layout.auth_path()).unwrap(), "auth");
    assert!(!layout.credentials_path().exists());
}

#[test]
fn seed_auth_errors_when_required_missing() {
    let temp = tempfile::tempdir().unwrap();
    let seed = temp.path().join("seed_home");
    std::fs::create_dir_all(&seed).unwrap();

    let target_root = temp.path().join("target_home");
    let layout = CodexHomeLayout::new(&target_root);
    let err = layout
        .seed_auth_from(
            &seed,
            AuthSeedOptions {
                require_auth: true,
                require_credentials: true,
                ..Default::default()
            },
        )
        .unwrap_err();

    match err {
        AuthSeedError::SeedFileMissing { path } => {
            assert!(path.ends_with("auth.json"), "{path:?}")
        }
        other => panic!("unexpected error: {other:?}"),
    }
}

#[test]
fn codex_client_returns_configured_home_layout() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().join("app_codex_home");
    let client = CodexClient::builder().codex_home(&root).build();

    let layout = client.codex_home_layout().expect("layout missing");
    assert_eq!(layout.root(), root.as_path());
    assert!(!root.exists());

    let client_without_home = CodexClient::builder().build();
    assert!(client_without_home.codex_home_layout().is_none());
}
