use super::cli::{Cli, HealthCmd, WorldAction, WorldCmd};
use crate::builtins as commands;
use anyhow::Result;
use std::env;

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

    pub(crate) fn world_doctor_main(json_mode: bool) -> i32 {
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
