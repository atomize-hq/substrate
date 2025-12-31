use super::config::{load_install_config, save_install_config, InstallConfig};
use crate::execution::env_sh_path;
#[cfg(target_os = "linux")]
use crate::execution::socket_activation;
use crate::WorldEnableArgs;
use anyhow::{bail, Result};
use std::env;
use std::path::PathBuf;
use std::time::Duration;
use substrate_common::paths as substrate_paths;

use helper_script::run_helper_script;
use log_ops::{append_log_line, initialize_log_file, print_dry_run_plan};
use manager_env::update_manager_env_exports;
use paths::{locate_helper_script, next_log_path, resolve_version_dir, resolve_world_socket_path};
use verify::verify_world_health;

mod helper_script;
mod log_ops;
mod manager_env;
mod paths;
mod verify;

pub fn run_enable(args: &WorldEnableArgs) -> Result<()> {
    if cfg!(target_os = "windows") {
        bail!("substrate world enable is not yet supported on Windows");
    }

    if let Some(home) = &args.home {
        env::set_var("SUBSTRATE_HOME", home);
    }

    let substrate_home = substrate_paths::substrate_home()?;
    let config_path = substrate_paths::config_file()?;
    let mut corrupt_config = false;
    let mut config = match load_install_config(&config_path) {
        Ok(cfg) => cfg,
        Err(err) => {
            corrupt_config = true;
            println!(
                "substrate: warn: config at {} is invalid ({err}); it will be replaced after provisioning.",
                config_path.display()
            );
            let mut cfg = InstallConfig::default();
            cfg.set_existed(false);
            cfg.set_world_enabled(false);
            cfg
        }
    };

    if !config.exists() && !corrupt_config {
        println!(
            "substrate: info: no config at {}; continuing and creating it after provisioning",
            config_path.display()
        );
    }

    let helper_override = env::var("SUBSTRATE_WORLD_ENABLE_SCRIPT")
        .ok()
        .map(PathBuf::from);
    let version_dir = if helper_override.is_some() {
        None
    } else {
        Some(resolve_version_dir(&substrate_home)?)
    };
    let script_path =
        locate_helper_script(&substrate_home, version_dir.as_deref(), helper_override)?;
    let log_path = next_log_path(&substrate_home)?;

    if args.dry_run {
        print_dry_run_plan(&script_path, args, &substrate_home, &log_path)?;
        println!(
            "Dry run only – no changes were made. Run 'substrate world doctor --json' after provisioning to verify connectivity."
        );
        return Ok(());
    }

    initialize_log_file(&log_path)?;
    append_log_line(&log_path, &format!("helper: {}", script_path.display()))?;
    let socket_override = resolve_world_socket_path();
    let wait_seconds = if socket_override.is_some() {
        args.timeout.min(5)
    } else {
        args.timeout
    };
    run_helper_script(
        &script_path,
        args,
        &substrate_home,
        &log_path,
        socket_override.as_deref(),
    )?;

    verify_world_health(
        &log_path,
        Duration::from_secs(wait_seconds),
        args.verbose,
        socket_override.as_deref(),
    )?;

    #[cfg(target_os = "linux")]
    {
        let activation_report = socket_activation::refresh_socket_activation_report();
        if activation_report.is_socket_activated() {
            append_log_line(
                &log_path,
                &format!(
                    "socket activation detected: {} (active_state={}, unit_file_state={})",
                    activation_report
                        .socket_unit
                        .as_ref()
                        .map(|u| u.name)
                        .unwrap_or("substrate-world-agent.socket"),
                    activation_report
                        .socket_unit
                        .as_ref()
                        .map(|u| u.active_state.as_str())
                        .unwrap_or("unknown"),
                    activation_report
                        .socket_unit
                        .as_ref()
                        .map(|u| u.unit_file_state.as_str())
                        .unwrap_or("unknown")
                ),
            )?;
            println!(
                "Socket activation: systemd is listening on {} ({}).",
                activation_report.socket_path,
                activation_report
                    .socket_unit
                    .as_ref()
                    .map(|u| u.active_state.as_str())
                    .unwrap_or("unknown")
            );
        }
    }

    config.set_world_enabled(true);
    save_install_config(&config_path, &config)?;
    update_manager_env_exports(&env_sh_path()?, true)?;

    println!(
        "World provisioning complete. Config updated at {}.",
        config_path.display()
    );
    println!(
        "Provisioning log: {}\nUpdated {}.\nNext: run 'substrate world doctor --json' or start a new shell to use the world backend.",
        log_path.display(),
        env_sh_path()?.display()
    );

    Ok(())
}
