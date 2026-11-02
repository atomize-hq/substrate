use std::{env, ffi::OsString, path::PathBuf, time::Duration};

use crate::mcp::AgentWrapperMcpCommandContext;

#[cfg(unix)]
use std::fs;

use super::super::{
    resolve::{resolve_codex_binary_path, resolve_codex_mcp_command},
    CODEX_BINARY_ENV, CODEX_HOME_ENV, PATH_ENV,
};
use super::support::{
    assert_backend_spawn_failure, sample_config, sample_context, test_env_lock, CurrentDirGuard,
    EnvGuard,
};

#[test]
fn resolve_codex_mcp_command_applies_precedence_and_materializes_injected_home() {
    let resolved = resolve_codex_mcp_command(&sample_config(), &sample_context()).expect("resolve");

    assert_eq!(resolved.binary_path, PathBuf::from("/tmp/fake-codex"));
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
            .get(super::super::CODEX_BINARY_ENV)
            .map(String::as_str),
        Some("/tmp/fake-codex")
    );
    assert_eq!(
        resolved.env.get(CODEX_HOME_ENV).map(String::as_str),
        Some("/tmp/codex-home")
    );
    assert_eq!(
        resolved.materialize_codex_home,
        Some(PathBuf::from("/tmp/codex-home"))
    );
}

#[test]
fn resolve_codex_mcp_command_skips_materialize_when_request_overrides_codex_home() {
    let mut context = sample_context();
    context
        .env
        .insert(CODEX_HOME_ENV.to_string(), "/tmp/request-home".to_string());

    let resolved = resolve_codex_mcp_command(&sample_config(), &context).expect("resolve");

    assert_eq!(
        resolved.env.get(CODEX_HOME_ENV).map(String::as_str),
        Some("/tmp/request-home")
    );
    assert_eq!(resolved.materialize_codex_home, None);
}

#[test]
fn resolve_codex_mcp_command_uses_backend_defaults_when_request_values_absent() {
    let resolved =
        resolve_codex_mcp_command(&sample_config(), &AgentWrapperMcpCommandContext::default())
            .expect("resolve");

    assert_eq!(resolved.working_dir, Some(PathBuf::from("default/workdir")));
    assert_eq!(resolved.timeout, Some(Duration::from_secs(30)));
}

#[cfg(unix)]
#[test]
fn resolve_codex_mcp_command_canonicalizes_relative_binary_before_working_dir() {
    let _env_lock = test_env_lock();
    let wrapper_dir = super::support::temp_test_dir("relative-wrapper");
    let binary_dir = wrapper_dir.join("bin");
    let working_dir = super::support::temp_test_dir("relative-working-dir");
    let binary_path =
        super::support::write_fake_codex(&binary_dir, "#!/usr/bin/env bash\nexit 0\n");
    let cwd_guard = CurrentDirGuard::set(&wrapper_dir);

    let mut config = sample_config();
    config.binary = Some(PathBuf::from("bin/codex"));
    config.default_working_dir = Some(working_dir.clone());

    let resolved = resolve_codex_mcp_command(&config, &AgentWrapperMcpCommandContext::default())
        .expect("resolve");

    assert_eq!(
        resolved.binary_path,
        std::fs::canonicalize(&binary_path).expect("canonicalize fake codex")
    );
    assert!(resolved.binary_path.is_absolute());
    assert_eq!(resolved.working_dir, Some(working_dir.clone()));

    drop(cwd_guard);
    std::fs::remove_dir_all(wrapper_dir).expect("wrapper dir should be removed");
    std::fs::remove_dir_all(working_dir).expect("working dir should be removed");
}

#[cfg(unix)]
#[test]
fn resolve_codex_mcp_command_prefers_request_path_in_child_env() {
    let _env_lock = test_env_lock();
    let ambient_dir = super::support::temp_test_dir("ambient-command-path");
    let _ambient_path = EnvGuard::set(PATH_ENV, ambient_dir.as_os_str().to_os_string());

    let request_path = "/tmp/request-path".to_string();
    let mut config = sample_config();
    config
        .env
        .insert(PATH_ENV.to_string(), "/tmp/config-path".to_string());
    let mut context = sample_context();
    context
        .env
        .insert(PATH_ENV.to_string(), request_path.clone());

    let resolved = resolve_codex_mcp_command(&config, &context).expect("resolve");

    assert_eq!(resolved.env.get(PATH_ENV), Some(&request_path));

    std::fs::remove_dir_all(ambient_dir).expect("ambient dir should be removed");
}

#[cfg(unix)]
#[test]
fn resolve_codex_mcp_command_uses_config_path_in_child_env_when_request_missing() {
    let _env_lock = test_env_lock();
    let ambient_dir = super::support::temp_test_dir("ambient-config-path");
    let _ambient_path = EnvGuard::set(PATH_ENV, ambient_dir.as_os_str().to_os_string());

    let config_path = "/tmp/config-path".to_string();
    let mut config = sample_config();
    config.env.insert(PATH_ENV.to_string(), config_path.clone());

    let resolved = resolve_codex_mcp_command(&config, &AgentWrapperMcpCommandContext::default())
        .expect("resolve");

    assert_eq!(resolved.env.get(PATH_ENV), Some(&config_path));

    std::fs::remove_dir_all(ambient_dir).expect("ambient dir should be removed");
}

#[cfg(unix)]
#[test]
fn resolve_codex_mcp_command_injects_ambient_path_into_child_env_when_unset() {
    let _env_lock = test_env_lock();
    let ambient_dir = super::support::temp_test_dir("ambient-only-command-path");
    let _ambient_path = EnvGuard::set(PATH_ENV, ambient_dir.as_os_str().to_os_string());

    let resolved =
        resolve_codex_mcp_command(&sample_config(), &AgentWrapperMcpCommandContext::default())
            .expect("resolve");

    assert_eq!(
        resolved.env.get(PATH_ENV).map(String::as_str),
        Some(ambient_dir.to_string_lossy().as_ref())
    );

    std::fs::remove_dir_all(ambient_dir).expect("ambient dir should be removed");
}

#[cfg(unix)]
#[test]
fn resolve_codex_binary_path_uses_effective_path_env_for_unqualified_binary() {
    let temp_dir = super::support::temp_test_dir("binary-path");
    let script_path = super::support::write_fake_codex(&temp_dir, "#!/usr/bin/env bash\nexit 0\n");

    let resolved = resolve_codex_binary_path(
        None,
        Some(OsString::from("codex")),
        Some(temp_dir.to_string_lossy().as_ref()),
        None,
        None,
        None,
    )
    .expect("effective PATH should resolve codex");

    assert_eq!(
        resolved,
        std::fs::canonicalize(&script_path).expect("canonicalize fake codex")
    );

    std::fs::remove_dir_all(temp_dir).expect("temp dir should be removed");
}

#[cfg(unix)]
#[test]
fn resolve_codex_binary_path_prefers_request_path_over_ambient_path() {
    let _env_lock = test_env_lock();
    let request_dir = super::support::temp_test_dir("request-path");
    let ambient_dir = super::support::temp_test_dir("ambient-path");
    let request_binary =
        super::support::write_fake_codex(&request_dir, "#!/usr/bin/env bash\nexit 0\n");
    let _ambient_binary =
        super::support::write_fake_codex(&ambient_dir, "#!/usr/bin/env bash\nexit 0\n");
    let _ambient_path = EnvGuard::set(PATH_ENV, ambient_dir.as_os_str().to_os_string());

    let resolved = resolve_codex_binary_path(
        None,
        Some(OsString::from("codex")),
        Some(request_dir.to_string_lossy().as_ref()),
        env::var_os(PATH_ENV),
        None,
        None,
    )
    .expect("request PATH should resolve codex");

    assert_eq!(
        resolved,
        std::fs::canonicalize(&request_binary).expect("canonicalize request binary")
    );

    std::fs::remove_dir_all(request_dir).expect("request dir should be removed");
    std::fs::remove_dir_all(ambient_dir).expect("ambient dir should be removed");
}

#[cfg(unix)]
#[test]
fn resolve_codex_binary_path_uses_ambient_path_when_effective_path_is_absent() {
    let _env_lock = test_env_lock();
    let ambient_dir = super::support::temp_test_dir("ambient-only");
    let ambient_binary =
        super::support::write_fake_codex(&ambient_dir, "#!/usr/bin/env bash\nexit 0\n");
    let _ambient_path = EnvGuard::set(PATH_ENV, ambient_dir.as_os_str().to_os_string());
    let _codex_binary = EnvGuard::unset(CODEX_BINARY_ENV);

    let resolved = resolve_codex_binary_path(None, None, None, env::var_os(PATH_ENV), None, None)
        .expect("ambient PATH should resolve codex");

    assert_eq!(
        resolved,
        std::fs::canonicalize(&ambient_binary).expect("canonicalize ambient binary")
    );

    std::fs::remove_dir_all(ambient_dir).expect("ambient dir should be removed");
}

#[cfg(unix)]
#[test]
fn resolve_codex_binary_path_skips_non_executable_shadow_file_on_path() {
    let shadow_dir = super::support::temp_test_dir("shadow-path");
    let executable_dir = super::support::temp_test_dir("executable-path");
    let shadow_path = shadow_dir.join("codex");
    let executable_path =
        super::support::write_fake_codex(&executable_dir, "#!/usr/bin/env bash\nexit 0\n");
    fs::write(&shadow_path, "shadow").expect("shadow file should be written");

    let joined_path = env::join_paths([shadow_dir.as_path(), executable_dir.as_path()])
        .expect("join PATH entries");
    let resolved = resolve_codex_binary_path(
        None,
        Some(OsString::from("codex")),
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

#[test]
fn resolve_codex_binary_path_rejects_unresolved_default_binary() {
    let err = resolve_codex_binary_path(None, None, None, None, None, None)
        .expect_err("default bare codex should fail when PATH cannot resolve it");

    assert_backend_spawn_failure(err);
}

#[cfg(unix)]
#[test]
fn resolve_codex_mcp_command_resolves_relative_request_path_from_request_working_dir() {
    let _env_lock = test_env_lock();
    let request_dir = super::support::temp_test_dir("request-working-dir");
    let default_dir = super::support::temp_test_dir("default-working-dir");
    let request_binary =
        super::support::write_fake_codex(&request_dir.join("bin"), "#!/usr/bin/env bash\nexit 0\n");
    let _default_binary =
        super::support::write_fake_codex(&default_dir.join("bin"), "#!/usr/bin/env bash\nexit 0\n");
    let _codex_binary = EnvGuard::unset(CODEX_BINARY_ENV);

    let mut config = sample_config();
    config.binary = None;
    config.default_working_dir = Some(default_dir.clone());

    let mut context = AgentWrapperMcpCommandContext {
        working_dir: Some(request_dir.clone()),
        ..Default::default()
    };
    context.env.insert(PATH_ENV.to_string(), "bin".to_string());

    let resolved = resolve_codex_mcp_command(&config, &context).expect("resolve");

    assert_eq!(
        resolved.binary_path,
        std::fs::canonicalize(&request_binary).expect("canonicalize request binary")
    );
    assert_eq!(resolved.working_dir, Some(request_dir.clone()));

    std::fs::remove_dir_all(request_dir).expect("request dir should be removed");
    std::fs::remove_dir_all(default_dir).expect("default dir should be removed");
}

#[cfg(unix)]
#[test]
fn resolve_codex_mcp_command_resolves_relative_config_path_from_default_working_dir() {
    let _env_lock = test_env_lock();
    let default_dir = super::support::temp_test_dir("config-default-working-dir");
    let default_binary =
        super::support::write_fake_codex(&default_dir.join("bin"), "#!/usr/bin/env bash\nexit 0\n");
    let _codex_binary = EnvGuard::unset(CODEX_BINARY_ENV);

    let mut config = sample_config();
    config.binary = None;
    config.default_working_dir = Some(default_dir.clone());
    config.env.insert(PATH_ENV.to_string(), "bin".to_string());

    let resolved = resolve_codex_mcp_command(&config, &AgentWrapperMcpCommandContext::default())
        .expect("resolve");

    assert_eq!(
        resolved.binary_path,
        std::fs::canonicalize(&default_binary).expect("canonicalize default binary")
    );
    assert_eq!(resolved.working_dir, Some(default_dir.clone()));

    std::fs::remove_dir_all(default_dir).expect("default dir should be removed");
}
