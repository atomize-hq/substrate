use super::config::{load_install_config, save_install_config, InstallConfig};
use crate::execution::env_sh_path;
#[cfg(target_os = "linux")]
use crate::execution::socket_activation;
use crate::WorldEnableArgs;
use anyhow::{bail, Result};
use std::env;
use std::path::{Path, PathBuf};
use std::time::Duration;
use substrate_common::paths as substrate_paths;

use helper_script::run_helper_script;
use log_ops::{append_log_line, initialize_log_file, print_dry_run_plan};
use manager_env::update_manager_env_exports;
use paths::{
    locate_helper_script, next_log_path, resolve_version_dir, resolve_world_socket_path,
    select_accepted_staged_world_agent,
};
use provision_deps::{
    ensure_supported_backend_or_exit, exit_probe_result_not_supported, print_pacman_requirements,
    print_probe_result, print_verbose_pacman_requirements, print_verbose_requirements,
    probe_world_manager, provision_apt_requirements, provision_pacman_requirements, WorldManager,
};
use verify::verify_world_health;

mod helper_script;
mod log_ops;
mod manager_env;
mod paths;
mod provision_deps;
mod verify;

fn resolve_helper_version_dir(
    substrate_home: &std::path::Path,
    helper_override: &Option<PathBuf>,
) -> Result<Option<PathBuf>> {
    if helper_override.is_some() {
        return Ok(None);
    }

    let prefix_helper = substrate_home.join("scripts/substrate/world-enable.sh");
    match resolve_version_dir(substrate_home) {
        Ok(version_dir) => Ok(Some(version_dir)),
        Err(_) if prefix_helper.exists() => Ok(None),
        Err(err) => Err(err),
    }
}

fn enforce_standard_version_dir_preflight(
    version_dir: Option<&Path>,
    helper_override: &Option<PathBuf>,
) {
    if helper_override.is_some() {
        return;
    }

    let Some(version_dir) = version_dir else {
        return;
    };

    if select_accepted_staged_world_agent(version_dir).is_none() {
        eprintln!(
            "substrate: accepted staged world-agent artifact missing under {}",
            version_dir.display()
        );
        std::process::exit(3);
    }
}

pub fn run_enable(args: &WorldEnableArgs) -> Result<()> {
    if args.provision_deps {
        return run_enable_with_provision_deps(args);
    }

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
    let version_dir = resolve_helper_version_dir(&substrate_home, &helper_override)?;
    enforce_standard_version_dir_preflight(version_dir.as_deref(), &helper_override);
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

fn run_enable_with_provision_deps(args: &WorldEnableArgs) -> Result<()> {
    ensure_supported_backend_or_exit();

    let cwd = env::current_dir().unwrap_or_else(|_| ".".into());
    let requirements =
        crate::builtins::world_deps::resolve_effective_enabled_provisioning_requirements_v1(&cwd)?;
    if !requirements.apt.is_empty() && !requirements.pacman.is_empty() {
        eprintln!(
            "substrate: `substrate world enable --provision-deps` found both apt and pacman packages in the enabled deps set; mixed-manager provisioning is unsupported and exits 4 without mutating the world."
        );
        std::process::exit(4);
    }

    let required_manager = if !requirements.apt.is_empty() {
        Some(WorldManager::Apt)
    } else if !requirements.pacman.is_empty() {
        Some(WorldManager::Pacman)
    } else {
        None
    };

    if let Some(home) = &args.home {
        env::set_var("SUBSTRATE_HOME", home);
    }

    let substrate_home = substrate_paths::substrate_home()?;
    let helper_override = env::var("SUBSTRATE_WORLD_ENABLE_SCRIPT")
        .ok()
        .map(PathBuf::from);
    let version_dir = resolve_helper_version_dir(&substrate_home, &helper_override)?;
    enforce_standard_version_dir_preflight(version_dir.as_deref(), &helper_override);

    if args.dry_run {
        let probe = match probe_world_manager() {
            Ok(probe) => probe,
            Err(err) => provision_deps::exit_backend_unavailable(&err),
        };

        print_probe_result(probe);

        if let Some(required_manager) = required_manager {
            if probe.manager != required_manager {
                exit_probe_result_not_supported(probe, required_manager);
            }

            match required_manager {
                WorldManager::Apt => {
                    if args.verbose {
                        print_verbose_requirements(&requirements.apt);
                    } else {
                        for requirement in &requirements.apt {
                            println!("{}", provision_deps::render_requirement(requirement));
                        }
                    }
                }
                WorldManager::Pacman => {
                    if args.verbose {
                        print_verbose_pacman_requirements(&requirements.pacman);
                    } else {
                        print_pacman_requirements(&requirements.pacman);
                    }
                }
                WorldManager::Unsupported => unreachable!("unsupported required manager"),
            }
        }

        return Ok(());
    }
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

    let script_path =
        locate_helper_script(&substrate_home, version_dir.as_deref(), helper_override)?;
    let log_path = next_log_path(&substrate_home)?;

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

    if let Err(err) = verify_world_health(
        &log_path,
        Duration::from_secs(wait_seconds),
        args.verbose,
        socket_override.as_deref(),
    ) {
        provision_deps::exit_backend_unavailable(&format!("{err:#}"));
    }

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

    if let Some(required_manager) = required_manager {
        let probe = match probe_world_manager() {
            Ok(probe) => probe,
            Err(err) => provision_deps::exit_backend_unavailable(&err),
        };

        print_probe_result(probe);

        if probe.manager != required_manager {
            exit_probe_result_not_supported(probe, required_manager);
        }

        if args.verbose {
            match required_manager {
                WorldManager::Apt => print_verbose_requirements(&requirements.apt),
                WorldManager::Pacman => print_verbose_pacman_requirements(&requirements.pacman),
                WorldManager::Unsupported => unreachable!("unsupported required manager"),
            }
        }
    }

    config.set_world_enabled(true);
    save_install_config(&config_path, &config)?;
    update_manager_env_exports(&env_sh_path()?, true)?;

    match required_manager {
        Some(WorldManager::Apt) => provision_apt_requirements(&requirements.apt),
        Some(WorldManager::Pacman) => provision_pacman_requirements(&requirements.pacman),
        None => {}
        Some(WorldManager::Unsupported) => unreachable!("unsupported required manager"),
    }
    run_sync_after_provisioning();

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

fn run_sync_after_provisioning() {
    let previous_skip_apt = env::var_os("SUBSTRATE_WORLD_DEPS_SKIP_APT");
    let previous_skip_pacman = env::var_os("SUBSTRATE_WORLD_DEPS_SKIP_PACMAN");
    env::set_var("SUBSTRATE_WORLD_DEPS_SKIP_APT", "1");
    env::set_var("SUBSTRATE_WORLD_DEPS_SKIP_PACMAN", "1");

    let sync_cmd = crate::WorldDepsCmd {
        action: crate::WorldDepsAction::Current(crate::execution::WorldDepsCurrentCmd {
            action: crate::execution::WorldDepsCurrentAction::Sync(
                crate::execution::WorldDepsCurrentSyncArgs {
                    dry_run: false,
                    verbose: false,
                    all: false,
                },
            ),
        }),
    };
    let exit_code = crate::builtins::world_deps::run(&sync_cmd, false, false);

    match previous_skip_apt {
        Some(value) => env::set_var("SUBSTRATE_WORLD_DEPS_SKIP_APT", value),
        None => env::remove_var("SUBSTRATE_WORLD_DEPS_SKIP_APT"),
    }
    match previous_skip_pacman {
        Some(value) => env::set_var("SUBSTRATE_WORLD_DEPS_SKIP_PACMAN", value),
        None => env::remove_var("SUBSTRATE_WORLD_DEPS_SKIP_PACMAN"),
    }

    if exit_code != 0 {
        std::process::exit(exit_code);
    }
}
