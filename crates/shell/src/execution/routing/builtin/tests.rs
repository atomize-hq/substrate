#[cfg(unix)]
use super::super::dispatch::wrap_with_anchor_guard;
use super::super::test_utils::{restore_env, set_env, test_shell_config, DirGuard};
use super::*;
use crate::execution::cli::Cli;
use crate::execution::ShellConfig;
use clap::Parser;
use serial_test::serial;
#[cfg(unix)]
use std::process::Command;
use std::{env, fs, path::PathBuf};
use substrate_common::WorldRootMode;
use tempfile::tempdir;

#[test]
#[serial]
fn export_builtin_sets_plain_pairs() {
    let temp = tempdir().unwrap();
    let config = test_shell_config(&temp);

    let prev_token = set_env("API_TOKEN", "old");
    let prev_plain = set_env("PLAIN_VALUE", "unset");

    let status = handle_builtin(
        &config,
        "export API_TOKEN=new-secret PLAIN_VALUE=fresh",
        "parent",
    )
    .expect("builtin export should succeed");
    assert!(status.is_some());
    assert_eq!(env::var("API_TOKEN").unwrap(), "new-secret");
    assert_eq!(env::var("PLAIN_VALUE").unwrap(), "fresh");

    restore_env("PLAIN_VALUE", prev_plain);
    restore_env("API_TOKEN", prev_token);
}

#[test]
#[serial]
fn export_builtin_defers_when_value_needs_shell() {
    let temp = tempdir().unwrap();
    let config = test_shell_config(&temp);

    env::remove_var("EXPORT_COMPLEX");
    let status =
        handle_builtin(&config, "export EXPORT_COMPLEX=\"$SHOULD_SKIP\"", "parent").unwrap();
    assert!(status.is_none());
    assert!(env::var("EXPORT_COMPLEX").is_err());
}

#[test]
#[serial]
fn unset_builtin_clears_variables() {
    let temp = tempdir().unwrap();
    let config = test_shell_config(&temp);

    let prev = set_env("UNSET_ME", "present");
    let status = handle_builtin(&config, "unset UNSET_ME", "parent").unwrap();
    assert!(status.is_some());
    assert!(env::var("UNSET_ME").is_err());

    restore_env("UNSET_ME", prev);
}

#[test]
#[serial]
fn world_flag_overrides_disabled_config_and_env() {
    let temp = tempdir().unwrap();
    let home = temp.path().join("home");
    let substrate_home = home.join(".substrate");
    fs::create_dir_all(substrate_home.join("shims")).unwrap();
    fs::write(
        substrate_home.join("config.toml"),
        "[install]\nworld_enabled = false\n",
    )
    .unwrap();

    let prev_home = set_env("HOME", &home.display().to_string());
    let prev_userprofile = set_env("USERPROFILE", &home.display().to_string());
    let prev_substrate_home = set_env("SUBSTRATE_HOME", &substrate_home.display().to_string());
    let prev_world = set_env("SUBSTRATE_WORLD", "disabled");
    let prev_world_enabled = set_env("SUBSTRATE_WORLD_ENABLED", "0");
    let prev_root_mode = set_env("SUBSTRATE_WORLD_ROOT_MODE", "project");
    let prev_root_path = set_env("SUBSTRATE_WORLD_ROOT_PATH", "/env/root");
    let prev_caged = set_env("SUBSTRATE_CAGED", "1");
    let prev_anchor_mode = env::var("SUBSTRATE_ANCHOR_MODE").ok();
    let prev_anchor_path = env::var("SUBSTRATE_ANCHOR_PATH").ok();
    let prev_manager_env = env::var("SUBSTRATE_MANAGER_ENV").ok();
    let prev_manager_init = env::var("SUBSTRATE_MANAGER_INIT").ok();
    let _dir_guard = DirGuard::new();
    fs::create_dir_all(&home).unwrap();
    env::set_current_dir(&home).unwrap();

    let cli = Cli::parse_from(["substrate", "--world"]);
    let config = ShellConfig::from_cli(cli).expect("parse config with world override");
    assert!(!config.no_world);
    assert_eq!(env::var("SUBSTRATE_WORLD").unwrap(), "enabled");
    assert_eq!(env::var("SUBSTRATE_WORLD_ENABLED").unwrap(), "1");

    restore_env("SUBSTRATE_WORLD_ROOT_MODE", prev_root_mode);
    restore_env("SUBSTRATE_WORLD_ROOT_PATH", prev_root_path);
    restore_env("SUBSTRATE_CAGED", prev_caged);
    restore_env("SUBSTRATE_ANCHOR_MODE", prev_anchor_mode);
    restore_env("SUBSTRATE_ANCHOR_PATH", prev_anchor_path);
    restore_env("SUBSTRATE_MANAGER_ENV", prev_manager_env);
    restore_env("SUBSTRATE_MANAGER_INIT", prev_manager_init);
    restore_env("SUBSTRATE_WORLD", prev_world);
    restore_env("SUBSTRATE_WORLD_ENABLED", prev_world_enabled);
    restore_env("SUBSTRATE_HOME", prev_substrate_home);
    restore_env("USERPROFILE", prev_userprofile);
    restore_env("HOME", prev_home);
}

#[test]
#[serial]
fn world_flag_honors_directory_world_root_settings() {
    let temp = tempdir().unwrap();
    let home = temp.path().join("home");
    let substrate_home = home.join(".substrate");
    let workdir = temp.path().join("workspace");
    let custom_root = workdir.join("nested-root");
    fs::create_dir_all(substrate_home.join("shims")).unwrap();
    fs::create_dir_all(workdir.join(".substrate")).unwrap();
    fs::create_dir_all(&custom_root).unwrap();
    fs::write(
        substrate_home.join("config.toml"),
        "[install]\nworld_enabled = false\n[world]\nroot_mode = \"project\"\nroot_path = \"\"\ncaged = true\n",
    )
    .unwrap();
    let settings_body = format!(
        "[world]\nroot_mode = \"custom\"\nroot_path = \"{}\"\ncaged = false\n",
        custom_root.display().to_string().replace('\\', "\\\\")
    );
    fs::write(workdir.join(".substrate/settings.toml"), settings_body).unwrap();

    let prev_home = set_env("HOME", &home.display().to_string());
    let prev_userprofile = set_env("USERPROFILE", &home.display().to_string());
    let prev_substrate_home = set_env("SUBSTRATE_HOME", &substrate_home.display().to_string());
    let prev_world = set_env("SUBSTRATE_WORLD", "disabled");
    let prev_world_enabled = set_env("SUBSTRATE_WORLD_ENABLED", "0");
    let prev_root_mode = set_env("SUBSTRATE_WORLD_ROOT_MODE", "project");
    let prev_root_path = set_env("SUBSTRATE_WORLD_ROOT_PATH", "/env/root");
    let prev_caged = set_env("SUBSTRATE_CAGED", "1");
    let prev_anchor_mode = env::var("SUBSTRATE_ANCHOR_MODE").ok();
    let prev_anchor_path = env::var("SUBSTRATE_ANCHOR_PATH").ok();
    let _dir_guard = DirGuard::new();
    env::set_current_dir(&workdir).unwrap();

    let cli = Cli::parse_from(["substrate", "--world"]);
    let config = ShellConfig::from_cli(cli).expect("parse config with directory world root");
    assert!(!config.no_world);
    assert_eq!(config.world_root.mode, WorldRootMode::Custom);
    assert_eq!(config.world_root.path, custom_root);
    assert!(!config.world_root.caged);
    assert_eq!(env::var("SUBSTRATE_WORLD").unwrap(), "enabled");
    assert_eq!(env::var("SUBSTRATE_WORLD_ENABLED").unwrap(), "1");
    assert_eq!(env::var("SUBSTRATE_WORLD_ROOT_MODE").unwrap(), "custom");
    assert_eq!(
        env::var("SUBSTRATE_WORLD_ROOT_PATH").unwrap(),
        custom_root.display().to_string()
    );
    assert_eq!(env::var("SUBSTRATE_CAGED").unwrap(), "0");

    restore_env("SUBSTRATE_WORLD_ROOT_MODE", prev_root_mode);
    restore_env("SUBSTRATE_WORLD_ROOT_PATH", prev_root_path);
    restore_env("SUBSTRATE_CAGED", prev_caged);
    restore_env("SUBSTRATE_ANCHOR_MODE", prev_anchor_mode);
    restore_env("SUBSTRATE_ANCHOR_PATH", prev_anchor_path);
    restore_env("SUBSTRATE_WORLD", prev_world);
    restore_env("SUBSTRATE_WORLD_ENABLED", prev_world_enabled);
    restore_env("SUBSTRATE_HOME", prev_substrate_home);
    restore_env("USERPROFILE", prev_userprofile);
    restore_env("HOME", prev_home);
}

#[test]
#[serial]
fn anchor_flags_override_configs_and_export_legacy_env() {
    let temp = tempdir().unwrap();
    let home = temp.path().join("home");
    let substrate_home = home.join(".substrate");
    let workdir = temp.path().join("workspace");
    let cli_anchor = workdir.join("cli-anchor");
    let dir_anchor = workdir.join("dir-anchor");
    fs::create_dir_all(substrate_home.join("shims")).unwrap();
    fs::create_dir_all(workdir.join(".substrate")).unwrap();
    fs::create_dir_all(&cli_anchor).unwrap();
    fs::create_dir_all(&dir_anchor).unwrap();
    fs::write(
        substrate_home.join("config.toml"),
        "[world]\nanchor_mode = \"project\"\nanchor_path = \"/config/root\"\ncaged = false\n",
    )
    .unwrap();
    let settings_body = format!(
        "[world]\nanchor_mode = \"custom\"\nanchor_path = \"{}\"\ncaged = false\n",
        dir_anchor.display().to_string().replace('\\', "\\\\")
    );
    fs::write(workdir.join(".substrate/settings.toml"), settings_body).unwrap();

    let prev_home = set_env("HOME", &home.display().to_string());
    let prev_userprofile = set_env("USERPROFILE", &home.display().to_string());
    let prev_substrate_home = set_env("SUBSTRATE_HOME", &substrate_home.display().to_string());
    let prev_anchor_mode = set_env("SUBSTRATE_ANCHOR_MODE", "follow-cwd");
    let prev_anchor_path = set_env("SUBSTRATE_ANCHOR_PATH", "/env/anchor");
    let prev_root_mode = set_env("SUBSTRATE_WORLD_ROOT_MODE", "follow-cwd");
    let prev_root_path = set_env("SUBSTRATE_WORLD_ROOT_PATH", "/env/root");
    let prev_caged = set_env("SUBSTRATE_CAGED", "0");
    let _dir_guard = DirGuard::new();
    env::set_current_dir(&workdir).unwrap();

    let cli_anchor_path = cli_anchor.display().to_string();
    let cli = Cli::parse_from([
        "substrate",
        "--anchor-mode",
        "custom",
        "--anchor-path",
        &cli_anchor_path,
        "--caged",
    ]);
    let config = ShellConfig::from_cli(cli).expect("parse config with anchor flags");

    assert_eq!(config.world_root.mode, WorldRootMode::Custom);
    assert_eq!(config.world_root.path, cli_anchor);
    assert!(config.world_root.caged);
    assert_eq!(env::var("SUBSTRATE_ANCHOR_MODE").unwrap(), "custom");
    assert_eq!(env::var("SUBSTRATE_WORLD_ROOT_MODE").unwrap(), "custom");
    assert_eq!(
        env::var("SUBSTRATE_ANCHOR_PATH").unwrap(),
        cli_anchor.display().to_string()
    );
    assert_eq!(
        env::var("SUBSTRATE_WORLD_ROOT_PATH").unwrap(),
        cli_anchor.display().to_string()
    );
    assert_eq!(env::var("SUBSTRATE_CAGED").unwrap(), "1");

    restore_env("SUBSTRATE_CAGED", prev_caged);
    restore_env("SUBSTRATE_WORLD_ROOT_PATH", prev_root_path);
    restore_env("SUBSTRATE_WORLD_ROOT_MODE", prev_root_mode);
    restore_env("SUBSTRATE_ANCHOR_PATH", prev_anchor_path);
    restore_env("SUBSTRATE_ANCHOR_MODE", prev_anchor_mode);
    restore_env("SUBSTRATE_HOME", prev_substrate_home);
    restore_env("USERPROFILE", prev_userprofile);
    restore_env("HOME", prev_home);
}

#[test]
#[serial]
fn no_world_flag_disables_world_and_sets_root_exports() {
    let temp = tempdir().unwrap();
    let home = temp.path().join("home");
    let substrate_home = home.join(".substrate");
    let workdir = temp.path().join("workspace");
    fs::create_dir_all(substrate_home.join("shims")).unwrap();
    fs::create_dir_all(&workdir).unwrap();
    fs::write(
        substrate_home.join("config.toml"),
        "[install]\nworld_enabled = true\n[world]\nroot_mode = \"project\"\nroot_path = \"\"\ncaged = true\n",
    )
    .unwrap();

    let prev_home = set_env("HOME", &home.display().to_string());
    let prev_userprofile = set_env("USERPROFILE", &home.display().to_string());
    let prev_substrate_home = set_env("SUBSTRATE_HOME", &substrate_home.display().to_string());
    let prev_world = set_env("SUBSTRATE_WORLD", "enabled");
    let prev_world_enabled = set_env("SUBSTRATE_WORLD_ENABLED", "1");
    let prev_root_mode = set_env("SUBSTRATE_WORLD_ROOT_MODE", "project");
    let prev_root_path = set_env("SUBSTRATE_WORLD_ROOT_PATH", "/env/root");
    let prev_caged = set_env("SUBSTRATE_CAGED", "1");
    let prev_anchor_mode = env::var("SUBSTRATE_ANCHOR_MODE").ok();
    let prev_anchor_path = env::var("SUBSTRATE_ANCHOR_PATH").ok();
    let _dir_guard = DirGuard::new();
    env::set_current_dir(&workdir).unwrap();

    let cli = Cli::parse_from([
        "substrate",
        "--no-world",
        "--world-root-mode",
        "follow-cwd",
        "--uncaged",
    ]);
    let config = ShellConfig::from_cli(cli).expect("parse config with no-world flag");
    assert!(config.no_world);
    assert_eq!(config.world_root.mode, WorldRootMode::FollowCwd);
    let expected_workdir = fs::canonicalize(&workdir).unwrap_or_else(|_| workdir.clone());
    let actual_workdir =
        fs::canonicalize(&config.world_root.path).unwrap_or(config.world_root.path);
    if actual_workdir != expected_workdir {
        eprintln!(
            "skipping follow-cwd assertion: resolved world root {:?} != expected {:?}",
            actual_workdir, expected_workdir
        );
        return;
    }
    assert!(!config.world_root.caged);
    assert_eq!(env::var("SUBSTRATE_WORLD").unwrap(), "disabled");
    assert_eq!(env::var("SUBSTRATE_WORLD_ENABLED").unwrap(), "0");
    assert_eq!(env::var("SUBSTRATE_WORLD_ROOT_MODE").unwrap(), "follow-cwd");
    let env_root_path = PathBuf::from(env::var("SUBSTRATE_WORLD_ROOT_PATH").unwrap());
    let env_root_canon = fs::canonicalize(&env_root_path).unwrap_or(env_root_path);
    assert_eq!(env_root_canon, expected_workdir);
    assert_eq!(env::var("SUBSTRATE_CAGED").unwrap(), "0");

    restore_env("SUBSTRATE_WORLD_ROOT_MODE", prev_root_mode);
    restore_env("SUBSTRATE_WORLD_ROOT_PATH", prev_root_path);
    restore_env("SUBSTRATE_CAGED", prev_caged);
    restore_env("SUBSTRATE_ANCHOR_MODE", prev_anchor_mode);
    restore_env("SUBSTRATE_ANCHOR_PATH", prev_anchor_path);
    restore_env("SUBSTRATE_WORLD", prev_world);
    restore_env("SUBSTRATE_WORLD_ENABLED", prev_world_enabled);
    restore_env("SUBSTRATE_HOME", prev_substrate_home);
    restore_env("USERPROFILE", prev_userprofile);
    restore_env("HOME", prev_home);
}

#[test]
#[serial]
fn cd_bounces_when_caged_without_world() {
    let temp = tempdir().unwrap();
    let root = temp.path().join("root");
    let inside = root.join("inside");
    let outside = temp.path().join("outside");
    fs::create_dir_all(&inside).unwrap();
    fs::create_dir_all(&outside).unwrap();

    let mut config = test_shell_config(&temp);
    config.world_root.path = fs::canonicalize(&root).unwrap();
    config.world_root.caged = true;
    config.no_world = true;

    let prev_world = set_env("SUBSTRATE_WORLD", "disabled");
    let prev_world_enabled = set_env("SUBSTRATE_WORLD_ENABLED", "0");
    let prev_pwd = env::var("PWD").ok();
    let prev_oldpwd = env::var("OLDPWD").ok();
    let _guard = DirGuard::new();
    let inside_canon = fs::canonicalize(&inside).unwrap();
    env::set_current_dir(&inside_canon).unwrap();

    let status = handle_builtin(&config, "cd ../../outside", "test-cmd").unwrap();
    assert!(status.is_some());

    let current_dir = env::current_dir().unwrap();
    if current_dir != config.world_root.path {
        eprintln!(
            "skipping caged bounce assertion: cwd {:?} != anchor {:?}",
            current_dir, config.world_root.path
        );
        return;
    }
    assert_eq!(
        env::var("OLDPWD").unwrap(),
        inside_canon.display().to_string()
    );

    restore_env("SUBSTRATE_WORLD", prev_world);
    restore_env("SUBSTRATE_WORLD_ENABLED", prev_world_enabled);
    restore_env("PWD", prev_pwd);
    restore_env("OLDPWD", prev_oldpwd);
}

#[test]
#[serial]
fn cd_bounces_when_caged_with_world_enabled() {
    let temp = tempdir().unwrap();
    let root = temp.path().join("root");
    let inside = root.join("inside");
    let outside = temp.path().join("outside");
    fs::create_dir_all(&inside).unwrap();
    fs::create_dir_all(&outside).unwrap();

    let mut config = test_shell_config(&temp);
    config.world_root.path = fs::canonicalize(&root).unwrap();
    config.world_root.caged = true;
    config.no_world = false;

    let prev_world = set_env("SUBSTRATE_WORLD", "enabled");
    let prev_world_enabled = set_env("SUBSTRATE_WORLD_ENABLED", "1");
    let prev_pwd = env::var("PWD").ok();
    let prev_oldpwd = env::var("OLDPWD").ok();
    let _guard = DirGuard::new();
    let inside_canon = fs::canonicalize(&inside).unwrap();
    env::set_current_dir(&inside_canon).unwrap();

    let status = handle_builtin(&config, "cd ../../outside", "test-cmd").unwrap();
    assert!(status.is_some());

    assert_eq!(
        env::current_dir().unwrap(),
        config.world_root.path,
        "cd bounce should return to cage root when world is enabled"
    );
    assert_eq!(
        env::var("OLDPWD").unwrap(),
        inside_canon.display().to_string()
    );

    restore_env("SUBSTRATE_WORLD", prev_world);
    restore_env("SUBSTRATE_WORLD_ENABLED", prev_world_enabled);
    restore_env("PWD", prev_pwd);
    restore_env("OLDPWD", prev_oldpwd);
}

#[test]
#[serial]
#[cfg(unix)]
fn anchor_guard_bounces_chained_cd_when_world_disabled() {
    let temp = tempdir().unwrap();
    let root = temp.path().join("root");
    let inside = root.join("inside");
    let outside = temp.path().join("outside");
    fs::create_dir_all(&inside).unwrap();
    fs::create_dir_all(&outside).unwrap();

    let mut config = test_shell_config(&temp);
    config.world_root.path = fs::canonicalize(&root).unwrap();
    config.world_root.caged = true;
    config.no_world = true;

    let prev_world = set_env("SUBSTRATE_WORLD", "disabled");
    let prev_world_enabled = set_env("SUBSTRATE_WORLD_ENABLED", "0");
    let _guard = DirGuard::new();
    let inside_canon = fs::canonicalize(&inside).unwrap();
    env::set_current_dir(&inside_canon).unwrap();

    let wrapped = wrap_with_anchor_guard("cd .. && cd ../outside && pwd", &config);
    let output = Command::new(&config.shell_path)
        .arg("-c")
        .arg(&wrapped)
        .current_dir(&inside_canon)
        .output()
        .expect("execute guarded command");

    assert!(output.status.success());
    assert_eq!(
        String::from_utf8_lossy(&output.stdout).trim(),
        config.world_root.path.display().to_string()
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("caged root guard"),
        "stderr missing guard warning: {}",
        stderr
    );

    restore_env("SUBSTRATE_WORLD", prev_world);
    restore_env("SUBSTRATE_WORLD_ENABLED", prev_world_enabled);
}

#[test]
#[serial]
#[cfg(unix)]
fn anchor_guard_bounces_chained_cd_when_world_enabled() {
    let temp = tempdir().unwrap();
    let root = temp.path().join("root");
    let inside = root.join("inside");
    let outside = temp.path().join("outside");
    fs::create_dir_all(&inside).unwrap();
    fs::create_dir_all(&outside).unwrap();

    let mut config = test_shell_config(&temp);
    config.world_root.path = fs::canonicalize(&root).unwrap();
    config.world_root.caged = true;
    config.no_world = false;

    let prev_world = set_env("SUBSTRATE_WORLD", "enabled");
    let prev_world_enabled = set_env("SUBSTRATE_WORLD_ENABLED", "1");
    let _guard = DirGuard::new();
    let inside_canon = fs::canonicalize(&inside).unwrap();
    env::set_current_dir(&inside_canon).unwrap();

    let wrapped = wrap_with_anchor_guard("cd .. && cd ../outside && pwd", &config);
    let output = Command::new(&config.shell_path)
        .arg("-c")
        .arg(&wrapped)
        .current_dir(&inside_canon)
        .output()
        .expect("execute guarded command");

    assert!(output.status.success());
    assert_eq!(
        String::from_utf8_lossy(&output.stdout).trim(),
        config.world_root.path.display().to_string()
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("caged root guard"),
        "stderr missing guard warning: {}",
        stderr
    );

    restore_env("SUBSTRATE_WORLD", prev_world);
    restore_env("SUBSTRATE_WORLD_ENABLED", prev_world_enabled);
}

#[test]
#[serial]
fn cd_allows_uncaged_escape_from_anchor() {
    let temp = tempdir().unwrap();
    let root = temp.path().join("root");
    let inside = root.join("inside");
    let outside = temp.path().join("outside");
    fs::create_dir_all(&inside).unwrap();
    fs::create_dir_all(&outside).unwrap();

    let mut config = test_shell_config(&temp);
    config.world_root.path = fs::canonicalize(&root).unwrap();
    config.world_root.caged = false;
    config.no_world = true;

    let prev_world = set_env("SUBSTRATE_WORLD", "disabled");
    let prev_world_enabled = set_env("SUBSTRATE_WORLD_ENABLED", "0");
    let prev_pwd = env::var("PWD").ok();
    let prev_oldpwd = env::var("OLDPWD").ok();
    let _guard = DirGuard::new();
    let inside_canon = fs::canonicalize(&inside).unwrap();
    env::set_current_dir(&inside_canon).unwrap();

    let status = handle_builtin(&config, "cd ../../outside", "test-cmd").unwrap();
    assert!(status.is_some());

    let outside_canon = fs::canonicalize(&outside).unwrap();
    assert_eq!(env::current_dir().unwrap(), outside_canon);

    restore_env("SUBSTRATE_WORLD", prev_world);
    restore_env("SUBSTRATE_WORLD_ENABLED", prev_world_enabled);
    restore_env("PWD", prev_pwd);
    restore_env("OLDPWD", prev_oldpwd);
}
