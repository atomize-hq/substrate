use std::{env, ffi::OsString, path::PathBuf, time::Duration};

use claude_code::ClaudeHomeLayout;

use crate::mcp::AgentWrapperMcpCommandContext;

use super::super::{
    resolve::{resolve_claude_binary_path, resolve_claude_mcp_command_with_env},
    CLAUDE_BINARY_ENV, CLAUDE_HOME_ENV, DISABLE_AUTOUPDATER_ENV, HOME_ENV, PATH_ENV,
    XDG_CACHE_HOME_ENV, XDG_CONFIG_HOME_ENV, XDG_DATA_HOME_ENV,
};
use super::support::{
    assert_backend_spawn_failure, sample_config, sample_config_without_home, sample_context,
    test_env_lock, CurrentDirGuard, EnvGuard,
};

#[cfg(unix)]
use std::fs;

#[cfg(unix)]
use super::support::{temp_test_dir, write_fake_claude};

#[test]
fn resolve_claude_binary_path_prefers_config_over_env() {
    let resolved = resolve_claude_binary_path(
        Some(&PathBuf::from("/tmp/from-config")),
        Some(OsString::from("/tmp/from-env")),
        None,
        None,
        None,
        None,
    )
    .expect("config path should resolve");

    assert_eq!(resolved, PathBuf::from("/tmp/from-config"));
}

#[test]
fn resolve_claude_binary_path_uses_env_when_config_absent() {
    let resolved = resolve_claude_binary_path(
        None,
        Some(OsString::from("/tmp/from-env")),
        None,
        None,
        None,
        None,
    )
    .expect("env path should resolve");

    assert_eq!(resolved, PathBuf::from("/tmp/from-env"));
}

#[test]
fn resolve_claude_binary_path_rejects_blank_env_without_a_resolvable_path() {
    let err = resolve_claude_binary_path(None, Some(OsString::from("")), None, None, None, None)
        .expect_err("blank env should fail resolution");

    assert_backend_spawn_failure(err);
}

#[cfg(unix)]
#[test]
fn resolve_claude_binary_path_uses_effective_path_env_for_unqualified_binary() {
    let temp_dir = temp_test_dir("binary-path");
    let script_path = write_fake_claude(&temp_dir, "#!/usr/bin/env bash\nexit 0\n");

    let resolved = resolve_claude_binary_path(
        None,
        Some(OsString::from("claude")),
        Some(temp_dir.to_string_lossy().as_ref()),
        None,
        None,
        None,
    )
    .expect("effective PATH should resolve claude");

    assert_eq!(
        resolved,
        fs::canonicalize(&script_path).expect("canonicalize fake claude")
    );

    fs::remove_dir_all(temp_dir).expect("temp dir should be removed");
}

#[cfg(unix)]
#[test]
fn resolve_claude_binary_path_prefers_request_path_over_config_and_ambient_path() {
    let _env_lock = test_env_lock();
    let request_dir = temp_test_dir("request-path");
    let config_dir = temp_test_dir("config-path");
    let ambient_dir = temp_test_dir("ambient-path");

    let request_binary = write_fake_claude(&request_dir, "#!/usr/bin/env bash\nexit 0\n");
    let _config_binary = write_fake_claude(&config_dir, "#!/usr/bin/env bash\nexit 0\n");
    let _ambient_binary = write_fake_claude(&ambient_dir, "#!/usr/bin/env bash\nexit 0\n");
    let _ambient_path = EnvGuard::set(PATH_ENV, ambient_dir.as_os_str().to_os_string());

    let resolved = resolve_claude_binary_path(
        None,
        Some(OsString::from("claude")),
        Some(
            env::join_paths([request_dir.as_path(), config_dir.as_path()])
                .expect("join request path")
                .to_string_lossy()
                .as_ref(),
        ),
        env::var_os(PATH_ENV),
        None,
        None,
    )
    .expect("request PATH should resolve claude");

    assert_eq!(
        resolved,
        fs::canonicalize(&request_binary).expect("canonicalize request binary")
    );

    fs::remove_dir_all(request_dir).expect("request dir should be removed");
    fs::remove_dir_all(config_dir).expect("config dir should be removed");
    fs::remove_dir_all(ambient_dir).expect("ambient dir should be removed");
}

#[cfg(unix)]
#[test]
fn resolve_claude_binary_path_prefers_config_path_over_ambient_path() {
    let _env_lock = test_env_lock();
    let config_dir = temp_test_dir("config-precedence");
    let ambient_dir = temp_test_dir("ambient-precedence");

    let config_binary = write_fake_claude(&config_dir, "#!/usr/bin/env bash\nexit 0\n");
    let _ambient_binary = write_fake_claude(&ambient_dir, "#!/usr/bin/env bash\nexit 0\n");
    let _ambient_path = EnvGuard::set(PATH_ENV, ambient_dir.as_os_str().to_os_string());

    let resolved = resolve_claude_binary_path(
        None,
        Some(OsString::from("claude")),
        Some(config_dir.to_string_lossy().as_ref()),
        env::var_os(PATH_ENV),
        None,
        None,
    )
    .expect("config PATH should resolve claude");

    assert_eq!(
        resolved,
        fs::canonicalize(&config_binary).expect("canonicalize config binary")
    );

    fs::remove_dir_all(config_dir).expect("config dir should be removed");
    fs::remove_dir_all(ambient_dir).expect("ambient dir should be removed");
}

#[cfg(unix)]
#[test]
fn resolve_claude_binary_path_uses_ambient_path_when_effective_path_is_absent() {
    let _env_lock = test_env_lock();
    let ambient_dir = temp_test_dir("ambient-only");
    let ambient_binary = write_fake_claude(&ambient_dir, "#!/usr/bin/env bash\nexit 0\n");
    let _ambient_path = EnvGuard::set(PATH_ENV, ambient_dir.as_os_str().to_os_string());
    let _claude_binary = EnvGuard::unset(CLAUDE_BINARY_ENV);

    let resolved = resolve_claude_binary_path(None, None, None, env::var_os(PATH_ENV), None, None)
        .expect("ambient PATH should resolve claude");

    assert_eq!(
        resolved,
        fs::canonicalize(&ambient_binary).expect("canonicalize ambient binary")
    );

    fs::remove_dir_all(ambient_dir).expect("ambient dir should be removed");
}

#[cfg(unix)]
#[test]
fn resolve_claude_binary_path_skips_non_executable_shadow_file_on_path() {
    let shadow_dir = temp_test_dir("shadow-path");
    let executable_dir = temp_test_dir("executable-path");
    let shadow_path = shadow_dir.join("claude");
    let executable_path = write_fake_claude(&executable_dir, "#!/usr/bin/env bash\nexit 0\n");
    fs::write(&shadow_path, "shadow").expect("shadow file should be written");

    let joined_path = env::join_paths([shadow_dir.as_path(), executable_dir.as_path()])
        .expect("join PATH entries");
    let resolved = resolve_claude_binary_path(
        None,
        Some(OsString::from("claude")),
        Some(joined_path.to_string_lossy().as_ref()),
        None,
        None,
        None,
    )
    .expect("effective PATH should skip non-executable shadow");

    assert_eq!(
        resolved,
        fs::canonicalize(&executable_path).expect("canonicalize executable path")
    );

    fs::remove_dir_all(shadow_dir).expect("shadow dir should be removed");
    fs::remove_dir_all(executable_dir).expect("executable dir should be removed");
}

#[cfg(unix)]
#[test]
fn resolve_claude_mcp_command_resolves_relative_request_path_from_request_working_dir() {
    let _env_lock = test_env_lock();
    let request_dir = temp_test_dir("request-working-dir");
    let default_dir = temp_test_dir("default-working-dir");
    let request_binary =
        write_fake_claude(&request_dir.join("bin"), "#!/usr/bin/env bash\nexit 0\n");
    let _default_binary =
        write_fake_claude(&default_dir.join("bin"), "#!/usr/bin/env bash\nexit 0\n");
    let _claude_binary = EnvGuard::unset(CLAUDE_BINARY_ENV);

    let mut config = sample_config_without_home();
    config.binary = None;
    config.default_working_dir = Some(default_dir.clone());

    let mut context = AgentWrapperMcpCommandContext {
        working_dir: Some(request_dir.clone()),
        ..Default::default()
    };
    context.env.insert(PATH_ENV.to_string(), "bin".to_string());

    let resolved =
        resolve_claude_mcp_command_with_env(&config, &context, None, None).expect("resolve");

    assert_eq!(
        resolved.binary_path,
        fs::canonicalize(&request_binary).expect("canonicalize request binary")
    );
    assert_eq!(resolved.working_dir, Some(request_dir.clone()));

    fs::remove_dir_all(request_dir).expect("request dir should be removed");
    fs::remove_dir_all(default_dir).expect("default dir should be removed");
}

#[cfg(unix)]
#[test]
fn resolve_claude_mcp_command_resolves_relative_config_path_from_default_working_dir() {
    let _env_lock = test_env_lock();
    let default_dir = temp_test_dir("config-default-working-dir");
    let default_binary =
        write_fake_claude(&default_dir.join("bin"), "#!/usr/bin/env bash\nexit 0\n");
    let _claude_binary = EnvGuard::unset(CLAUDE_BINARY_ENV);

    let mut config = sample_config_without_home();
    config.binary = None;
    config.default_working_dir = Some(default_dir.clone());
    config.env.insert(PATH_ENV.to_string(), "bin".to_string());

    let resolved = resolve_claude_mcp_command_with_env(
        &config,
        &AgentWrapperMcpCommandContext::default(),
        None,
        None,
    )
    .expect("resolve");

    assert_eq!(
        resolved.binary_path,
        fs::canonicalize(&default_binary).expect("canonicalize default binary")
    );
    assert_eq!(resolved.working_dir, Some(default_dir.clone()));

    fs::remove_dir_all(default_dir).expect("default dir should be removed");
}

#[test]
fn resolve_claude_mcp_command_applies_precedence_and_home_injection() {
    let resolved = resolve_claude_mcp_command_with_env(
        &sample_config(),
        &sample_context(),
        Some(OsString::from("/tmp/from-env")),
        Some(PathBuf::from("/tmp/from-ambient-home")),
    )
    .expect("command should resolve");
    let layout = ClaudeHomeLayout::new("/tmp/claude-home");

    assert_eq!(resolved.binary_path, PathBuf::from("/tmp/fake-claude"));
    assert_eq!(resolved.working_dir, Some(PathBuf::from("request/workdir")));
    assert_eq!(resolved.timeout, Some(Duration::from_secs(5)));
    assert_eq!(
        resolved.env.get("CONFIG_ONLY").map(String::as_str),
        Some("config-only")
    );
    assert_eq!(
        resolved.env.get("OVERRIDE_ME").map(String::as_str),
        Some("request")
    );
    assert_eq!(
        resolved.env.get("REQUEST_ONLY").map(String::as_str),
        Some("request-only")
    );
    assert_eq!(
        resolved
            .env
            .get(DISABLE_AUTOUPDATER_ENV)
            .map(String::as_str),
        Some("1")
    );
    assert_eq!(
        resolved.env.get(CLAUDE_HOME_ENV).map(String::as_str),
        Some("/tmp/claude-home")
    );
    assert_eq!(
        resolved.env.get(HOME_ENV).map(String::as_str),
        Some("/tmp/claude-home")
    );
    assert_eq!(
        resolved.env.get(XDG_CONFIG_HOME_ENV).map(String::as_str),
        Some(layout.xdg_config_home().to_string_lossy().as_ref())
    );
    assert_eq!(
        resolved.env.get(XDG_DATA_HOME_ENV).map(String::as_str),
        Some(layout.xdg_data_home().to_string_lossy().as_ref())
    );
    assert_eq!(
        resolved.env.get(XDG_CACHE_HOME_ENV).map(String::as_str),
        Some(layout.xdg_cache_home().to_string_lossy().as_ref())
    );
    assert_eq!(resolved.materialize_claude_home, Some(layout));
}

#[test]
fn resolve_claude_mcp_command_uses_backend_defaults_when_request_values_absent() {
    let resolved = resolve_claude_mcp_command_with_env(
        &sample_config(),
        &AgentWrapperMcpCommandContext::default(),
        None,
        None,
    )
    .expect("command should resolve");

    assert_eq!(resolved.working_dir, Some(PathBuf::from("default/workdir")));
    assert_eq!(resolved.timeout, Some(Duration::from_secs(30)));
}

#[cfg(unix)]
#[test]
fn resolve_claude_mcp_command_canonicalizes_relative_binary_before_working_dir() {
    let _env_lock = test_env_lock();
    let wrapper_dir = temp_test_dir("relative-wrapper");
    let binary_dir = wrapper_dir.join("bin");
    let working_dir = temp_test_dir("relative-working-dir");
    let binary_path = write_fake_claude(&binary_dir, "#!/usr/bin/env bash\nexit 0\n");
    let cwd_guard = CurrentDirGuard::set(&wrapper_dir);

    let mut config = sample_config_without_home();
    config.binary = Some(PathBuf::from("bin/claude"));
    let context = AgentWrapperMcpCommandContext {
        working_dir: Some(working_dir.clone()),
        ..Default::default()
    };

    let resolved =
        resolve_claude_mcp_command_with_env(&config, &context, None, None).expect("resolve");

    assert_eq!(
        resolved.binary_path,
        fs::canonicalize(&binary_path).expect("canonicalize fake claude")
    );
    assert!(resolved.binary_path.is_absolute());
    assert_eq!(resolved.working_dir, Some(working_dir.clone()));

    drop(cwd_guard);
    fs::remove_dir_all(wrapper_dir).expect("wrapper dir should be removed");
    fs::remove_dir_all(working_dir).expect("working dir should be removed");
}

#[cfg(unix)]
#[test]
fn resolve_claude_mcp_command_prefers_request_path_in_child_env() {
    let _env_lock = test_env_lock();
    let ambient_dir = temp_test_dir("ambient-command-path");
    let _ambient_path = EnvGuard::set(PATH_ENV, ambient_dir.as_os_str().to_os_string());

    let request_path = "/tmp/request-path".to_string();
    let mut config = sample_config_without_home();
    config
        .env
        .insert(PATH_ENV.to_string(), "/tmp/config-path".to_string());
    let mut context = sample_context();
    context
        .env
        .insert(PATH_ENV.to_string(), request_path.clone());

    let resolved = resolve_claude_mcp_command_with_env(
        &config,
        &context,
        Some(OsString::from("/tmp/claude")),
        None,
    )
    .expect("resolve");

    assert_eq!(resolved.env.get(PATH_ENV), Some(&request_path));

    fs::remove_dir_all(ambient_dir).expect("ambient dir should be removed");
}

#[cfg(unix)]
#[test]
fn resolve_claude_mcp_command_uses_config_path_in_child_env_when_request_missing() {
    let _env_lock = test_env_lock();
    let ambient_dir = temp_test_dir("ambient-config-path");
    let _ambient_path = EnvGuard::set(PATH_ENV, ambient_dir.as_os_str().to_os_string());

    let config_path = "/tmp/config-path".to_string();
    let mut config = sample_config_without_home();
    config.env.insert(PATH_ENV.to_string(), config_path.clone());

    let resolved = resolve_claude_mcp_command_with_env(
        &config,
        &AgentWrapperMcpCommandContext::default(),
        Some(OsString::from("/tmp/claude")),
        None,
    )
    .expect("resolve");

    assert_eq!(resolved.env.get(PATH_ENV), Some(&config_path));

    fs::remove_dir_all(ambient_dir).expect("ambient dir should be removed");
}

#[cfg(unix)]
#[test]
fn resolve_claude_mcp_command_injects_ambient_path_into_child_env_when_unset() {
    let _env_lock = test_env_lock();
    let ambient_dir = temp_test_dir("ambient-only-command-path");
    let _ambient_path = EnvGuard::set(PATH_ENV, ambient_dir.as_os_str().to_os_string());

    let resolved = resolve_claude_mcp_command_with_env(
        &sample_config_without_home(),
        &AgentWrapperMcpCommandContext::default(),
        Some(OsString::from("/tmp/claude")),
        None,
    )
    .expect("resolve");

    assert_eq!(
        resolved.env.get(PATH_ENV).map(String::as_str),
        Some(ambient_dir.to_string_lossy().as_ref())
    );

    fs::remove_dir_all(ambient_dir).expect("ambient dir should be removed");
}

#[cfg(unix)]
#[test]
fn resolve_claude_mcp_command_prefers_request_path_over_config_and_ambient_path() {
    let _env_lock = test_env_lock();
    let request_dir = temp_test_dir("request-command-path");
    let config_dir = temp_test_dir("config-command-path");
    let ambient_dir = temp_test_dir("ambient-command-path");
    let request_binary = write_fake_claude(&request_dir, "#!/usr/bin/env bash\nexit 0\n");
    let _config_binary = write_fake_claude(&config_dir, "#!/usr/bin/env bash\nexit 0\n");
    let _ambient_binary = write_fake_claude(&ambient_dir, "#!/usr/bin/env bash\nexit 0\n");
    let _ambient_path = EnvGuard::set(PATH_ENV, ambient_dir.as_os_str().to_os_string());
    let _claude_binary = EnvGuard::unset(CLAUDE_BINARY_ENV);

    let mut config = sample_config_without_home();
    config.binary = None;
    config.env.insert(
        PATH_ENV.to_string(),
        config_dir.to_string_lossy().into_owned(),
    );

    let mut context = AgentWrapperMcpCommandContext::default();
    context.env.insert(
        PATH_ENV.to_string(),
        request_dir.to_string_lossy().into_owned(),
    );

    let resolved =
        resolve_claude_mcp_command_with_env(&config, &context, None, None).expect("resolve");

    assert_eq!(
        resolved.binary_path,
        fs::canonicalize(&request_binary).expect("canonicalize request binary")
    );

    fs::remove_dir_all(request_dir).expect("request dir should be removed");
    fs::remove_dir_all(config_dir).expect("config dir should be removed");
    fs::remove_dir_all(ambient_dir).expect("ambient dir should be removed");
}

#[cfg(unix)]
#[test]
fn resolve_claude_mcp_command_prefers_config_path_over_ambient_path() {
    let _env_lock = test_env_lock();
    let config_dir = temp_test_dir("config-command-only-path");
    let ambient_dir = temp_test_dir("ambient-command-only-path");
    let config_binary = write_fake_claude(&config_dir, "#!/usr/bin/env bash\nexit 0\n");
    let _ambient_binary = write_fake_claude(&ambient_dir, "#!/usr/bin/env bash\nexit 0\n");
    let _ambient_path = EnvGuard::set(PATH_ENV, ambient_dir.as_os_str().to_os_string());
    let _claude_binary = EnvGuard::unset(CLAUDE_BINARY_ENV);

    let mut config = sample_config_without_home();
    config.binary = None;
    config.env.insert(
        PATH_ENV.to_string(),
        config_dir.to_string_lossy().into_owned(),
    );

    let resolved = resolve_claude_mcp_command_with_env(
        &config,
        &AgentWrapperMcpCommandContext::default(),
        None,
        None,
    )
    .expect("command should resolve");

    assert_eq!(
        resolved.binary_path,
        fs::canonicalize(&config_binary).expect("canonicalize config binary")
    );

    fs::remove_dir_all(config_dir).expect("config dir should be removed");
    fs::remove_dir_all(ambient_dir).expect("ambient dir should be removed");
}

#[test]
fn disable_autoupdater_default_does_not_override_explicit_values() {
    let mut config = sample_config();
    config
        .env
        .insert(DISABLE_AUTOUPDATER_ENV.to_string(), "0".to_string());
    let resolved = resolve_claude_mcp_command_with_env(
        &config,
        &AgentWrapperMcpCommandContext::default(),
        None,
        None,
    )
    .expect("command should resolve");
    assert_eq!(
        resolved
            .env
            .get(DISABLE_AUTOUPDATER_ENV)
            .map(String::as_str),
        Some("0")
    );

    let mut context = AgentWrapperMcpCommandContext::default();
    context
        .env
        .insert(DISABLE_AUTOUPDATER_ENV.to_string(), "2".to_string());
    let resolved =
        resolve_claude_mcp_command_with_env(&config, &context, None, None).expect("resolve");
    assert_eq!(
        resolved
            .env
            .get(DISABLE_AUTOUPDATER_ENV)
            .map(String::as_str),
        Some("2")
    );
}

#[test]
fn request_env_overrides_injected_home_keys() {
    let layout = ClaudeHomeLayout::new("/tmp/claude-home");
    let mut context = AgentWrapperMcpCommandContext::default();
    context
        .env
        .insert(HOME_ENV.to_string(), "/tmp/request-home".to_string());
    context.env.insert(
        XDG_CONFIG_HOME_ENV.to_string(),
        "/tmp/request-xdg-config".to_string(),
    );

    let resolved = resolve_claude_mcp_command_with_env(&sample_config(), &context, None, None)
        .expect("resolve");

    assert_eq!(
        resolved.env.get(HOME_ENV).map(String::as_str),
        Some("/tmp/request-home")
    );
    assert_eq!(
        resolved.env.get(XDG_CONFIG_HOME_ENV).map(String::as_str),
        Some("/tmp/request-xdg-config")
    );
    assert_eq!(
        resolved.env.get(CLAUDE_HOME_ENV).map(String::as_str),
        Some("/tmp/claude-home")
    );
    assert_eq!(resolved.materialize_claude_home, Some(layout));
}

#[test]
fn request_env_override_of_claude_home_disables_materialization() {
    let mut context = AgentWrapperMcpCommandContext::default();
    context.env.insert(
        CLAUDE_HOME_ENV.to_string(),
        "/tmp/request-claude-home".to_string(),
    );

    let resolved = resolve_claude_mcp_command_with_env(&sample_config(), &context, None, None)
        .expect("resolve");

    assert_eq!(
        resolved.env.get(CLAUDE_HOME_ENV).map(String::as_str),
        Some("/tmp/request-claude-home")
    );
    assert_eq!(resolved.materialize_claude_home, None);
}

#[test]
fn request_env_with_same_injected_home_values_keeps_materialization() {
    let layout = ClaudeHomeLayout::new("/tmp/claude-home");
    let mut context = AgentWrapperMcpCommandContext::default();
    context
        .env
        .insert(CLAUDE_HOME_ENV.to_string(), "/tmp/claude-home".to_string());
    context
        .env
        .insert(HOME_ENV.to_string(), "/tmp/claude-home".to_string());
    context.env.insert(
        XDG_CONFIG_HOME_ENV.to_string(),
        layout.xdg_config_home().to_string_lossy().into_owned(),
    );
    context.env.insert(
        XDG_DATA_HOME_ENV.to_string(),
        layout.xdg_data_home().to_string_lossy().into_owned(),
    );
    context.env.insert(
        XDG_CACHE_HOME_ENV.to_string(),
        layout.xdg_cache_home().to_string_lossy().into_owned(),
    );

    let resolved = resolve_claude_mcp_command_with_env(&sample_config(), &context, None, None)
        .expect("resolve");

    assert_eq!(resolved.materialize_claude_home, Some(layout));
}

#[test]
fn ambient_claude_home_is_used_when_config_home_is_absent() {
    let ambient_home = PathBuf::from("/tmp/ambient-claude-home");
    let layout = ClaudeHomeLayout::new(ambient_home.clone());
    let resolved = resolve_claude_mcp_command_with_env(
        &sample_config_without_home(),
        &AgentWrapperMcpCommandContext::default(),
        None,
        Some(ambient_home.clone()),
    )
    .expect("command should resolve");

    assert_eq!(
        resolved.env.get(CLAUDE_HOME_ENV).map(String::as_str),
        Some(ambient_home.to_string_lossy().as_ref())
    );
    assert_eq!(
        resolved.env.get(HOME_ENV).map(String::as_str),
        Some(ambient_home.to_string_lossy().as_ref())
    );
    assert_eq!(
        resolved.env.get(XDG_CONFIG_HOME_ENV).map(String::as_str),
        Some(layout.xdg_config_home().to_string_lossy().as_ref())
    );
    assert_eq!(resolved.materialize_claude_home, Some(layout));
}

#[test]
fn blank_ambient_claude_home_is_ignored_when_config_home_is_absent() {
    let resolved = resolve_claude_mcp_command_with_env(
        &sample_config_without_home(),
        &AgentWrapperMcpCommandContext::default(),
        None,
        Some(PathBuf::new()),
    )
    .expect("command should resolve");

    assert_eq!(resolved.env.get(CLAUDE_HOME_ENV), None);
    assert_eq!(resolved.env.get(HOME_ENV), None);
    assert_eq!(resolved.materialize_claude_home, None);
}

#[test]
fn configured_claude_home_beats_ambient_claude_home() {
    let layout = ClaudeHomeLayout::new("/tmp/claude-home");
    let resolved = resolve_claude_mcp_command_with_env(
        &sample_config(),
        &AgentWrapperMcpCommandContext::default(),
        None,
        Some(PathBuf::from("/tmp/ambient-claude-home")),
    )
    .expect("command should resolve");

    assert_eq!(
        resolved.env.get(CLAUDE_HOME_ENV).map(String::as_str),
        Some("/tmp/claude-home")
    );
    assert_eq!(resolved.materialize_claude_home, Some(layout));
}

#[test]
fn request_env_overrides_other_home_keys_without_touching_claude_home_keeps_materialization() {
    let layout = ClaudeHomeLayout::new("/tmp/claude-home");
    let mut context = AgentWrapperMcpCommandContext::default();
    context
        .env
        .insert(HOME_ENV.to_string(), "/tmp/request-home".to_string());
    context.env.insert(
        XDG_DATA_HOME_ENV.to_string(),
        "/tmp/request-xdg-data".to_string(),
    );
    context.env.insert(
        XDG_CACHE_HOME_ENV.to_string(),
        "/tmp/request-xdg-cache".to_string(),
    );

    let resolved = resolve_claude_mcp_command_with_env(&sample_config(), &context, None, None)
        .expect("resolve");

    assert_eq!(
        resolved.env.get(CLAUDE_HOME_ENV).map(String::as_str),
        Some("/tmp/claude-home")
    );
    assert_eq!(
        resolved.env.get(HOME_ENV).map(String::as_str),
        Some("/tmp/request-home")
    );
    assert_eq!(
        resolved.env.get(XDG_DATA_HOME_ENV).map(String::as_str),
        Some("/tmp/request-xdg-data")
    );
    assert_eq!(
        resolved.env.get(XDG_CACHE_HOME_ENV).map(String::as_str),
        Some("/tmp/request-xdg-cache")
    );
    assert_eq!(resolved.materialize_claude_home, Some(layout));
}

#[test]
fn config_env_override_of_home_keeps_materialization() {
    let layout = ClaudeHomeLayout::new("/tmp/claude-home");
    let mut config = sample_config();
    config
        .env
        .insert(HOME_ENV.to_string(), "/tmp/config-home".to_string());

    let resolved = resolve_claude_mcp_command_with_env(
        &config,
        &AgentWrapperMcpCommandContext::default(),
        None,
        None,
    )
    .expect("command should resolve");

    assert_eq!(
        resolved.env.get(CLAUDE_HOME_ENV).map(String::as_str),
        Some("/tmp/claude-home")
    );
    assert_eq!(
        resolved.env.get(HOME_ENV).map(String::as_str),
        Some("/tmp/config-home")
    );
    assert_eq!(resolved.materialize_claude_home, Some(layout));
}

#[test]
fn request_env_override_of_ambient_claude_home_disables_materialization() {
    let ambient_home = PathBuf::from("/tmp/ambient-claude-home");
    let mut context = AgentWrapperMcpCommandContext::default();
    context.env.insert(
        CLAUDE_HOME_ENV.to_string(),
        "/tmp/request-claude-home".to_string(),
    );
    context.env.insert(
        XDG_CONFIG_HOME_ENV.to_string(),
        "/tmp/request-xdg-config".to_string(),
    );

    let resolved = resolve_claude_mcp_command_with_env(
        &sample_config_without_home(),
        &context,
        None,
        Some(ambient_home.clone()),
    )
    .expect("command should resolve");

    assert_eq!(
        resolved.env.get(CLAUDE_HOME_ENV).map(String::as_str),
        Some("/tmp/request-claude-home")
    );
    assert_eq!(
        resolved.env.get(XDG_CONFIG_HOME_ENV).map(String::as_str),
        Some("/tmp/request-xdg-config")
    );
    assert_eq!(resolved.materialize_claude_home, None);
}

#[test]
fn no_claude_home_is_materialized_without_config_or_ambient_home() {
    let resolved = resolve_claude_mcp_command_with_env(
        &sample_config_without_home(),
        &AgentWrapperMcpCommandContext::default(),
        None,
        None,
    )
    .expect("command should resolve");

    assert_eq!(resolved.env.get(CLAUDE_HOME_ENV), None);
    assert_eq!(resolved.env.get(HOME_ENV), None);
    assert_eq!(resolved.materialize_claude_home, None);
}
