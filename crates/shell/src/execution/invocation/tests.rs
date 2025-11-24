use super::*;
use crate::Cli;
use clap::Parser;
use serial_test::serial;
use std::fs;
use std::path::PathBuf;
use tempfile::tempdir;

fn set_env(key: &str, value: &str) -> Option<String> {
    let previous = std::env::var(key).ok();
    std::env::set_var(key, value);
    previous
}

fn restore_env(key: &str, previous: Option<String>) {
    if let Some(value) = previous {
        std::env::set_var(key, value);
    } else {
        std::env::remove_var(key);
    }
}

#[test]
#[serial]
fn wrap_mode_uses_cli_shell_and_shimmed_path() {
    let temp = tempdir().unwrap();
    let home = temp.path().join("home");
    let substrate_home = home.join(".substrate");
    fs::create_dir_all(substrate_home.join("shims")).unwrap();
    fs::write(
        substrate_home.join("config.toml"),
        "[install]\nworld_enabled = true\n",
    )
    .unwrap();

    let home_str = home.display().to_string();
    let substrate_home_str = substrate_home.display().to_string();
    let path_value = if cfg!(windows) {
        "C:\\bin;D:\\bin"
    } else {
        "/bin:/usr/bin"
    };
    let prev_home = set_env("HOME", &home_str);
    let prev_userprofile = set_env("USERPROFILE", &home_str);
    let prev_substrate_home = set_env("SUBSTRATE_HOME", &substrate_home_str);
    let prev_path = set_env("PATH", path_value);
    let prev_shim_original_path = std::env::var("SHIM_ORIGINAL_PATH").ok();
    let prev_world = std::env::var("SUBSTRATE_WORLD").ok();
    let prev_world_enabled = std::env::var("SUBSTRATE_WORLD_ENABLED").ok();
    let prev_no_shims = std::env::var("SUBSTRATE_NO_SHIMS").ok();
    std::env::remove_var("SHIM_ORIGINAL_PATH");
    std::env::remove_var("SUBSTRATE_WORLD");
    std::env::remove_var("SUBSTRATE_WORLD_ENABLED");
    std::env::remove_var("SUBSTRATE_NO_SHIMS");

    let cli = Cli::parse_from(["substrate", "-c", "echo hi", "--shell", "/bin/zsh"]);
    let config = ShellConfig::from_cli(cli).expect("build shell config from CLI");

    match &config.mode {
        ShellMode::Wrap(cmd) => assert_eq!(cmd, "echo hi"),
        other => panic!("expected wrap mode, got {other:?}"),
    }
    assert_eq!(config.shell_path, "/bin/zsh");
    assert!(!config.no_world);
    assert!(!config.skip_shims);

    let sep = if cfg!(windows) { ';' } else { ':' };
    let expected = format!(
        "{}{sep}{}",
        PathBuf::from(&config.shim_dir).display(),
        std::env::var("PATH").unwrap()
    );
    let expected = substrate_common::dedupe_path(&expected);
    assert_eq!(config.shimmed_path.as_deref(), Some(expected.as_str()));

    restore_env("SUBSTRATE_WORLD_ENABLED", prev_world_enabled);
    restore_env("SUBSTRATE_WORLD", prev_world);
    restore_env("SUBSTRATE_NO_SHIMS", prev_no_shims);
    restore_env("SHIM_ORIGINAL_PATH", prev_shim_original_path);
    restore_env("PATH", prev_path);
    restore_env("SUBSTRATE_HOME", prev_substrate_home);
    restore_env("USERPROFILE", prev_userprofile);
    restore_env("HOME", prev_home);
}

#[test]
#[serial]
fn skip_shims_and_no_world_disable_shimmed_path() {
    let temp = tempdir().unwrap();
    let home = temp.path().join("home");
    let substrate_home = home.join(".substrate");
    fs::create_dir_all(substrate_home.join("shims")).unwrap();
    fs::write(
        substrate_home.join("config.toml"),
        "[install]\nworld_enabled = true\n",
    )
    .unwrap();

    let home_str = home.display().to_string();
    let substrate_home_str = substrate_home.display().to_string();
    let path_value = if cfg!(windows) {
        "C:\\bin;D:\\bin"
    } else {
        "/bin:/usr/bin"
    };
    let prev_home = set_env("HOME", &home_str);
    let prev_userprofile = set_env("USERPROFILE", &home_str);
    let prev_substrate_home = set_env("SUBSTRATE_HOME", &substrate_home_str);
    let prev_path = set_env("PATH", path_value);
    let prev_no_shims = set_env("SUBSTRATE_NO_SHIMS", "1");
    let prev_world = std::env::var("SUBSTRATE_WORLD").ok();
    let prev_world_enabled = std::env::var("SUBSTRATE_WORLD_ENABLED").ok();
    let prev_shim_original_path = std::env::var("SHIM_ORIGINAL_PATH").ok();
    std::env::remove_var("SHIM_ORIGINAL_PATH");

    let cli = Cli::parse_from(["substrate", "--no-world", "-c", "echo hi"]);
    let config = ShellConfig::from_cli(cli).expect("config honors skip flags");

    assert!(config.no_world);
    assert!(config.skip_shims);
    assert!(config.shimmed_path.is_none());
    assert_eq!(std::env::var("SUBSTRATE_WORLD").unwrap(), "disabled");
    assert_eq!(std::env::var("SUBSTRATE_WORLD_ENABLED").unwrap(), "0");

    restore_env("SHIM_ORIGINAL_PATH", prev_shim_original_path);
    restore_env("SUBSTRATE_NO_SHIMS", prev_no_shims);
    restore_env("SUBSTRATE_WORLD_ENABLED", prev_world_enabled);
    restore_env("SUBSTRATE_WORLD", prev_world);
    restore_env("PATH", prev_path);
    restore_env("SUBSTRATE_HOME", prev_substrate_home);
    restore_env("USERPROFILE", prev_userprofile);
    restore_env("HOME", prev_home);
}
