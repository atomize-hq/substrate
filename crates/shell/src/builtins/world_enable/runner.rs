use super::config::{load_install_config, save_install_config, InstallConfig};
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
use paths::{
    locate_helper_script, next_log_path, resolve_manager_env_path, resolve_prefix,
    resolve_version_dir, resolve_world_socket_path,
};
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

    let prefix = resolve_prefix(args.prefix.as_deref())?;
    let config_path = substrate_paths::config_file()?;
    let manager_env_path = resolve_manager_env_path()?;
    let mut corrupt_config = false;
    let mut config = match load_install_config(&config_path) {
        Ok(cfg) => cfg,
        Err(err) => {
            corrupt_config = true;
            println!(
                "substrate: warn: install metadata at {} is invalid ({err}); it will be replaced after provisioning.",
                config_path.display()
            );
            let mut cfg = InstallConfig::default();
            cfg.set_existed(false);
            cfg.set_world_enabled(false);
            cfg
        }
    };

    if config.world_enabled && config.exists() && !args.force && !args.dry_run {
        println!(
            "World backend already enabled (metadata at {}). Use --force to rerun provisioning.",
            config_path.display()
        );
        return Ok(());
    }

    if !config.exists() && !corrupt_config {
        println!(
            "substrate: info: no install metadata at {}; continuing and creating it after provisioning",
            config_path.display()
        );
    }

    let helper_override = env::var("SUBSTRATE_WORLD_ENABLE_SCRIPT")
        .ok()
        .map(PathBuf::from);
    let version_dir = if helper_override.is_some() {
        None
    } else {
        Some(resolve_version_dir(&prefix)?)
    };
    let script_path = locate_helper_script(&prefix, version_dir.as_deref(), helper_override)?;
    let log_path = next_log_path(&prefix)?;

    if args.dry_run {
        print_dry_run_plan(&script_path, args, &prefix, &log_path)?;
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
        &prefix,
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
    update_manager_env_exports(&manager_env_path, true)?;

    println!(
        "World provisioning complete. Metadata updated at {}.",
        config_path.display()
    );
    println!(
        "Provisioning log: {}\nManager env updated at {} with SUBSTRATE_WORLD exports.\nNext: run 'substrate world doctor --json' or start a new shell to use the world backend.",
        log_path.display(),
        manager_env_path.display()
    );

    Ok(())
}
