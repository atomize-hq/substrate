use super::cli::{Cli, HealthCmd, WorldAction, WorldCmd};
use crate::builtins as commands;
use anyhow::Result;
use std::env;
use substrate_broker::world_fs_mode;

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
mod fallback {
    use serde_json::json;
    use substrate_broker::world_fs_mode;

    pub(crate) fn world_doctor_main(json_mode: bool) -> i32 {
        let fs_mode = world_fs_mode();
        if json_mode {
            let out = json!({
                "platform": std::env::consts::OS,
                "overlay_present": serde_json::Value::Null,
                "fuse": {"dev": serde_json::Value::Null, "bin": serde_json::Value::Null},
                "cgroup_v2": serde_json::Value::Null,
                "nft_present": serde_json::Value::Null,
                "dmesg_restrict": serde_json::Value::Null,
                "overlay_root": serde_json::Value::Null,
                "copydiff_root": serde_json::Value::Null,
                "world_fs_mode": fs_mode.as_str(),
                "ok": true
            });
            println!("{}", serde_json::to_string_pretty(&out).unwrap());
        } else {
            eprintln!("substrate world doctor currently supports Linux and macOS");
            println!("overlay: N/A");
            println!("fuse-overlayfs: N/A");
            println!("cgroup v2: N/A");
            println!("nft: N/A");
            println!("dmesg_restrict: N/A");
            println!("world_fs_mode: {}", fs_mode.as_str());
        }
        0
    }
}

pub(crate) fn update_world_env(no_world: bool) {
    if no_world {
        env::set_var("SUBSTRATE_WORLD_ENABLED", "0");
        env::set_var("SUBSTRATE_WORLD", "disabled");
    } else {
        env::set_var("SUBSTRATE_WORLD_ENABLED", "1");
        env::set_var("SUBSTRATE_WORLD", "enabled");
    }
    let fs_mode = world_fs_mode();
    env::set_var("SUBSTRATE_WORLD_FS_MODE", fs_mode.as_str());
}

pub(crate) fn handle_world_command(cmd: &WorldCmd, cli: &Cli) -> Result<()> {
    match &cmd.action {
        WorldAction::Doctor { json } => {
            let code = world_doctor_main(*json);
            std::process::exit(code);
        }
        WorldAction::Enable(opts) => {
            commands::world_enable::run_enable(opts)?;
        }
        WorldAction::Deps(opts) => {
            commands::world_deps::run(opts, cli.no_world, cli.world)?;
        }
        WorldAction::Cleanup(opts) => {
            commands::world_cleanup::run(opts)?;
        }
    }
    Ok(())
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
