use super::cli::{Cli, HealthCmd, HostAction, HostCmd, WorldAction, WorldCmd};
use crate::builtins as commands;
#[cfg(test)]
use crate::execution::world_env_guard;
use anyhow::Result;
use std::env;
use std::path::PathBuf;
use substrate_broker::world_fs_policy;

#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "windows")]
mod windows;

#[cfg(all(
    not(target_os = "linux"),
    not(target_os = "macos"),
    not(target_os = "windows")
))]
use self::fallback::world_doctor_main;
#[cfg(target_os = "linux")]
use linux::world_doctor_main;
#[cfg(target_os = "macos")]
use macos::world_doctor_main;
#[cfg(target_os = "windows")]
use windows::world_doctor_main;

#[cfg(all(
    not(target_os = "linux"),
    not(target_os = "macos"),
    not(target_os = "windows")
))]
use self::fallback::host_doctor_main;
#[cfg(target_os = "linux")]
use linux::host_doctor_main;
#[cfg(target_os = "macos")]
use macos::host_doctor_main;
#[cfg(target_os = "windows")]
use windows::host_doctor_main;

#[cfg(all(
    not(target_os = "linux"),
    not(target_os = "macos"),
    not(target_os = "windows")
))]
mod fallback {
    use serde_json::json;
    use substrate_broker::world_fs_policy;

    pub(crate) fn world_doctor_main(json_mode: bool, world_enabled: bool) -> i32 {
        let world_fs = world_fs_policy();
        if json_mode {
            let out = json!({
                "schema_version": 1,
                "platform": std::env::consts::OS,
                "world_enabled": world_enabled,
                "ok": false,
                "host": {
                    "platform": std::env::consts::OS,
                    "ok": false,
                    "world_fs_mode": world_fs.mode.as_str(),
                    "world_fs_isolation": world_fs.isolation.as_str(),
                    "world_fs_require_world": world_fs.require_world,
                },
                "world": {"status": "unsupported", "ok": false}
            });
            println!("{}", serde_json::to_string_pretty(&out).unwrap());
        } else {
            println!("== substrate world doctor ==");
            println!("== Host ==");
            println!("WARN  | unsupported platform: {}", std::env::consts::OS);
            println!("== World ==");
            println!("FAIL  | unsupported platform: {}", std::env::consts::OS);
        }
        4
    }

    pub(crate) fn host_doctor_main(json_mode: bool, world_enabled: bool) -> i32 {
        let world_fs = world_fs_policy();
        if json_mode {
            let out = json!({
                "schema_version": 1,
                "platform": std::env::consts::OS,
                "world_enabled": world_enabled,
                "ok": false,
                "host": {
                    "platform": std::env::consts::OS,
                    "ok": false,
                    "world_fs_mode": world_fs.mode.as_str(),
                    "world_fs_isolation": world_fs.isolation.as_str(),
                    "world_fs_require_world": world_fs.require_world,
                },
            });
            println!("{}", serde_json::to_string_pretty(&out).unwrap());
        } else {
            println!("== substrate host doctor ==");
            println!("WARN  | unsupported platform: {}", std::env::consts::OS);
        }
        4
    }
}

pub(crate) fn update_world_env(no_world: bool) {
    #[cfg(test)]
    let _env_lock = world_env_guard();

    if no_world {
        env::set_var("SUBSTRATE_WORLD_ENABLED", "0");
        env::set_var("SUBSTRATE_WORLD", "disabled");
    } else {
        env::set_var("SUBSTRATE_WORLD_ENABLED", "1");
        env::set_var("SUBSTRATE_WORLD", "enabled");
    }
    let world_fs = world_fs_policy();
    env::set_var("SUBSTRATE_WORLD_FS_MODE", world_fs.mode.as_str());
    env::set_var("SUBSTRATE_WORLD_FS_ISOLATION", world_fs.isolation.as_str());
    env::set_var(
        "SUBSTRATE_WORLD_REQUIRE_WORLD",
        if world_fs.require_world { "1" } else { "0" },
    );
}

pub(crate) fn handle_world_command(cmd: &WorldCmd, cli: &Cli) -> Result<()> {
    match &cmd.action {
        WorldAction::Doctor { json } => {
            let launch_cwd = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
            let cli_world_enabled = if cli.world {
                Some(true)
            } else if cli.no_world {
                Some(false)
            } else {
                None
            };
            let effective = match crate::execution::config_model::resolve_effective_config(
                &launch_cwd,
                &crate::execution::config_model::CliConfigOverrides {
                    world_enabled: cli_world_enabled,
                    ..Default::default()
                },
            ) {
                Ok(cfg) => cfg,
                Err(err) => {
                    eprintln!("substrate world doctor: {:#}", err);
                    std::process::exit(2);
                }
            };
            env::set_var("SUBSTRATE_POLICY_MODE", effective.policy.mode.as_str());
            let _ = substrate_broker::set_global_broker(substrate_broker::BrokerHandle::new());
            let _ = substrate_broker::detect_profile(&launch_cwd);
            let code = world_doctor_main(*json, effective.world.enabled);
            std::process::exit(code);
        }
        WorldAction::Enable(opts) => {
            commands::world_enable::run_enable(opts)?;
        }
        WorldAction::Deps(opts) => {
            let code = commands::world_deps::run(opts, cli.no_world, cli.world);
            std::process::exit(code);
        }
        WorldAction::Cleanup(opts) => {
            commands::world_cleanup::run(opts)?;
        }
        WorldAction::Verify(opts) => {
            let code = commands::world_verify::run(opts)?;
            std::process::exit(code);
        }
    }
    Ok(())
}

pub(crate) fn handle_host_command(cmd: &HostCmd, cli: &Cli) -> Result<()> {
    match &cmd.action {
        HostAction::Doctor { json } => {
            let launch_cwd = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
            let cli_world_enabled = if cli.world {
                Some(true)
            } else if cli.no_world {
                Some(false)
            } else {
                None
            };
            let effective = match crate::execution::config_model::resolve_effective_config(
                &launch_cwd,
                &crate::execution::config_model::CliConfigOverrides {
                    world_enabled: cli_world_enabled,
                    ..Default::default()
                },
            ) {
                Ok(cfg) => cfg,
                Err(err) => {
                    eprintln!("substrate host doctor: {:#}", err);
                    std::process::exit(2);
                }
            };
            env::set_var("SUBSTRATE_POLICY_MODE", effective.policy.mode.as_str());
            let _ = substrate_broker::set_global_broker(substrate_broker::BrokerHandle::new());
            let _ = substrate_broker::detect_profile(&launch_cwd);
            let code = host_doctor_main(*json, effective.world.enabled);
            std::process::exit(code);
        }
    }
}

pub(crate) fn handle_health_command(cmd: &HealthCmd, cli: &Cli) -> Result<()> {
    commands::health::run(cmd.json, cli.no_world, cli.world)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn world_enable_exports_policy_fs_mode() {
        let _ = substrate_broker::set_global_broker(substrate_broker::BrokerHandle::new());

        let temp = tempdir().expect("tempdir");
        let policy_path = temp.path().join("policy.yaml");
        let policy = substrate_broker::Policy {
            world_fs_mode: substrate_common::WorldFsMode::ReadOnly,
            world_fs_require_world: true,
            ..Default::default()
        };
        fs::write(&policy_path, serde_yaml::to_string(&policy).unwrap()).expect("write policy");

        substrate_broker::reload_policy(&policy_path).expect("load policy");

        let prev_world = env::var("SUBSTRATE_WORLD").ok();
        let prev_enabled = env::var("SUBSTRATE_WORLD_ENABLED").ok();
        let prev_fs_mode = env::var("SUBSTRATE_WORLD_FS_MODE").ok();

        update_world_env(false);

        assert_eq!(
            env::var("SUBSTRATE_WORLD_FS_MODE").as_deref(),
            Ok("read_only"),
            "policy-driven fs mode should be exported for downstream consumers"
        );

        match prev_world {
            Some(value) => env::set_var("SUBSTRATE_WORLD", value),
            None => env::remove_var("SUBSTRATE_WORLD"),
        }
        match prev_enabled {
            Some(value) => env::set_var("SUBSTRATE_WORLD_ENABLED", value),
            None => env::remove_var("SUBSTRATE_WORLD_ENABLED"),
        }
        match prev_fs_mode {
            Some(value) => env::set_var("SUBSTRATE_WORLD_FS_MODE", value),
            None => env::remove_var("SUBSTRATE_WORLD_FS_MODE"),
        }
    }
}
