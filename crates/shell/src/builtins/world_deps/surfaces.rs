use super::errors::{
    WorldDepsBackendRequiredError, WorldDepsBackendUnavailableError, WorldDepsSafetyViolationError,
    WorldDepsUnmetPrerequisiteError,
};
use super::inventory::{
    builtin_inventory_v1, find_workspace_inventory_chain, load_inventory_dir_v1,
    merge_inventory_layer_v1, summarize_inventory_v1, AptSpecV1, HostPlatform, InstallMethodV1,
    InventoryItemDefV1, InventoryListItemSummaryV1, InventoryViewV1, WrapperDefV1, WrapperKindV1,
};
#[cfg(unix)]
use super::AuthenticatedWorldDepsContextV1;
#[cfg(not(target_os = "linux"))]
use crate::execution::build_agent_client_and_request;
#[cfg(target_os = "linux")]
use crate::execution::build_authenticated_world_deps_client_and_request;
use crate::execution::config_model;
use crate::execution::{
    WorldDepsCurrentAction, WorldDepsCurrentCmd, WorldDepsCurrentInstallArgs,
    WorldDepsCurrentListArgs, WorldDepsCurrentListViewArg, WorldDepsCurrentShowArgs,
    WorldDepsCurrentSyncArgs, WorldDepsGlobalAction, WorldDepsGlobalCmd, WorldDepsScopedListArgs,
    WorldDepsScopedListViewArg, WorldDepsScopedMutateArgs, WorldDepsScopedResetArgs,
    WorldDepsWorkspaceAction, WorldDepsWorkspaceCmd,
};
use crate::{WorldDepsAction, WorldDepsCmd};
use anyhow::{anyhow, Context, Result};
use base64::engine::general_purpose::STANDARD as BASE64;
use base64::Engine;
use serde::Serialize;
use std::collections::{HashMap, HashSet};
use std::env;
use std::error::Error as StdError;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
#[cfg(not(unix))]
use substrate_common::paths as substrate_paths;
use tempfile::NamedTempFile;
use tokio::runtime::Runtime;

#[derive(Debug, Serialize)]
struct ListOutputV1 {
    schema_version: u32,
    scope: String,
    view: String,
    items: Vec<InventoryListItemSummaryV1>,
}

#[derive(Debug, Serialize)]
struct ShowOutputV1 {
    schema_version: u32,
    scope: String,
    name: String,
    kind: String,
    item: InventoryItemDefV1,
}

pub fn run(
    cmd: &WorldDepsCmd,
    cli_no_world: bool,
    _cli_force_world: bool,
    #[cfg(unix)] context: &AuthenticatedWorldDepsContextV1,
) -> i32 {
    #[cfg(not(target_os = "linux"))]
    let result = {
        #[cfg(unix)]
        let _ = context;
        Err(anyhow!(WorldDepsBackendUnavailableError::new(
            "authenticated world-deps execution is available on Linux only",
        )))
    };
    #[cfg(target_os = "linux")]
    let result = (|| -> Result<()> {
        context.revalidate_authority()?;
        match &cmd.action {
            WorldDepsAction::Current(current) => {
                if cli_no_world {
                    match &current.action {
                        WorldDepsCurrentAction::List(args)
                            if args.view == WorldDepsCurrentListViewArg::Applied =>
                        {
                            return Err(anyhow!(WorldDepsBackendRequiredError::new(
                                "world backend required for `substrate world deps current list applied` (world disabled via --no-world)"
                            )));
                        }
                        WorldDepsCurrentAction::Show(args) if args.explain => {
                            return Err(anyhow!(WorldDepsBackendRequiredError::new(
                                "world backend required for `substrate world deps current show --explain` (world disabled via --no-world)"
                            )));
                        }
                        WorldDepsCurrentAction::Install(_) | WorldDepsCurrentAction::Sync(_) => {
                            return Err(anyhow!(WorldDepsBackendRequiredError::new(
                                "world backend required for `substrate world deps current install|sync` (world disabled via --no-world)"
                            )));
                        }
                        _ => {}
                    }
                }
                run_current(current, context)
            }
            WorldDepsAction::Global(global) => run_global(global, context),
            WorldDepsAction::Workspace(workspace) => run_workspace(workspace, context),
        }
    })();

    match result {
        Ok(()) => 0,
        Err(err) => {
            let code = world_deps_exit_code(&err);
            if code == 5 && looks_like_world_deps_hardening_violation(&err) {
                eprintln!(
                    "substrate: world deps blocked by hardening/cage: required writes to `/var/lib/substrate/world-deps` are not permitted.\nHint: ensure `/var/lib/substrate/world-deps` is bind-mounted read-write inside the world and retry."
                );
                eprintln!("Underlying error: {:#}", err);
            } else if code == 5 {
                eprintln!("{:#}", err);
            } else if code == 3 {
                if cfg!(target_os = "macos") {
                    eprintln!("Remediation:\n  - Run: scripts/mac/lima-warm.sh");
                } else if cfg!(windows) {
                    eprintln!("Remediation:\n  - Run: scripts/windows/wsl-warm.ps1");
                }
                if let Some(reason) = world_backend_unavailable_reason(&err) {
                    let header = if cfg!(target_os = "macos") {
                        "substrate: world backend unavailable for world deps on macOS; run `substrate world doctor --json` to inspect backend status, then retry."
                    } else {
                        "substrate: world backend unavailable for world deps; run `substrate world doctor --json` to inspect backend status, then retry."
                    };
                    eprintln!("{header}\nUnderlying error: {reason}");
                } else {
                    eprintln!("{:#}", err);
                }
            } else {
                eprintln!("{:#}", err);
            }
            code
        }
    }
}

fn world_backend_unavailable_reason(err: &anyhow::Error) -> Option<String> {
    err.chain()
        .find_map(|cause| cause.downcast_ref::<WorldDepsBackendUnavailableError>())
        .map(|e| e.reason().to_string())
}

fn world_deps_exit_code(err: &anyhow::Error) -> i32 {
    if config_model::is_user_error(err) {
        return 2;
    }
    if err.is::<WorldDepsUnmetPrerequisiteError>() {
        return 4;
    }
    if err.is::<WorldDepsBackendRequiredError>() {
        return 3;
    }
    if err.is::<WorldDepsSafetyViolationError>() {
        return 5;
    }
    // Hardening conflicts should win even when surfaced through backend-unavailable wrappers.
    if looks_like_world_deps_hardening_violation(err) {
        return 5;
    }
    if err
        .chain()
        .any(|cause| cause.is::<WorldDepsBackendUnavailableError>())
    {
        return 3;
    }
    1
}

fn looks_like_world_deps_hardening_violation(err: &anyhow::Error) -> bool {
    let mut current: Option<&(dyn StdError + 'static)> = Some(err.as_ref());
    let hardening_paths = [
        "/var/lib/substrate/world-deps",
        "/var/lib/apt",
        "/var/cache/apt",
        "/var/lib/dpkg",
        "/etc/apt",
    ];
    while let Some(e) = current {
        let msg = e.to_string();
        if hardening_paths.iter().any(|path| msg.contains(path))
            && (msg.contains("Permission denied")
                || msg.contains("permission denied")
                || msg.contains("Read-only file system")
                || msg.contains("read-only file system"))
        {
            return true;
        }
        current = e.source();
    }
    false
}

pub(crate) fn run_current(
    cmd: &WorldDepsCurrentCmd,
    #[cfg(unix)] context: &AuthenticatedWorldDepsContextV1,
) -> Result<()> {
    match &cmd.action {
        WorldDepsCurrentAction::List(args) => run_current_list(
            args,
            #[cfg(unix)]
            context,
        ),
        WorldDepsCurrentAction::Show(args) => run_current_show(
            args,
            #[cfg(unix)]
            context,
        ),
        WorldDepsCurrentAction::Install(args) => run_current_install(
            args,
            #[cfg(unix)]
            context,
        ),
        WorldDepsCurrentAction::Sync(args) => run_current_sync(
            args,
            #[cfg(unix)]
            context,
        ),
    }
}

pub(crate) fn run_global(
    cmd: &WorldDepsGlobalCmd,
    #[cfg(unix)] context: &AuthenticatedWorldDepsContextV1,
) -> Result<()> {
    match &cmd.action {
        WorldDepsGlobalAction::List(args) => run_global_list(
            args,
            #[cfg(unix)]
            context,
        ),
        WorldDepsGlobalAction::Add(args) => run_global_add(
            args,
            #[cfg(unix)]
            context,
        ),
        WorldDepsGlobalAction::Remove(args) => run_global_remove(
            args,
            #[cfg(unix)]
            context,
        ),
        WorldDepsGlobalAction::Reset(args) => run_global_reset(
            args,
            #[cfg(unix)]
            context,
        ),
    }
}

pub(crate) fn run_workspace(
    cmd: &WorldDepsWorkspaceCmd,
    #[cfg(unix)] context: &AuthenticatedWorldDepsContextV1,
) -> Result<()> {
    match &cmd.action {
        WorldDepsWorkspaceAction::List(args) => run_workspace_list(
            args,
            #[cfg(unix)]
            context,
        ),
        WorldDepsWorkspaceAction::Add(args) => run_workspace_add(
            args,
            #[cfg(unix)]
            context,
        ),
        WorldDepsWorkspaceAction::Remove(args) => run_workspace_remove(
            args,
            #[cfg(unix)]
            context,
        ),
        WorldDepsWorkspaceAction::Reset(args) => run_workspace_reset(
            args,
            #[cfg(unix)]
            context,
        ),
    }
}

fn run_current_list(
    args: &WorldDepsCurrentListArgs,
    #[cfg(unix)] context: &AuthenticatedWorldDepsContextV1,
) -> Result<()> {
    if args.all && args.view != WorldDepsCurrentListViewArg::Applied {
        return Err(config_model::user_error(
            "--all is only valid for `substrate world deps current list applied`",
        ));
    }
    #[cfg(unix)]
    let (cwd, cfg) = (context.launch_cwd(), context.effective_config());
    #[cfg(not(unix))]
    let cwd = env::current_dir().unwrap_or_else(|_| ".".into());
    #[cfg(not(unix))]
    let cfg = config_model::resolve_effective_config(&cwd, &Default::default())
        .context("failed to resolve effective config")?;
    let global_deps_dir =
        if cfg.world.deps.inventory_mode == config_model::WorldDepsInventoryMode::Merged {
            #[cfg(unix)]
            {
                Some(context.global_deps_dir().to_path_buf())
            }
            #[cfg(not(unix))]
            {
                Some(substrate_paths::substrate_home()?.join("deps"))
            }
        } else {
            None
        };

    match args.view {
        WorldDepsCurrentListViewArg::Available => {
            let view = resolve_current_inventory_view(cwd, cfg, global_deps_dir.as_deref())?;
            if view.is_empty() {
                eprintln!("substrate: note: no deps inventory items visible for this directory; add definitions under $SUBSTRATE_HOME/deps/ or <workspace_root>/.substrate/deps/");
            }

            let items = summarize_inventory_v1(&view);
            if args.json {
                let out = ListOutputV1 {
                    schema_version: 1,
                    scope: "current".to_string(),
                    view: "available".to_string(),
                    items,
                };
                println!("{}", serde_json::to_string_pretty(&out)?);
            } else {
                print_inventory_table(&items);
            }
            Ok(())
        }
        WorldDepsCurrentListViewArg::Enabled => {
            run_current_list_enabled(cwd, cfg, global_deps_dir.as_deref(), args.json)
        }
        WorldDepsCurrentListViewArg::Applied => run_current_list_applied(
            cwd,
            cfg,
            global_deps_dir.as_deref(),
            args.all,
            args.json,
            #[cfg(target_os = "linux")]
            context,
        ),
    }
}

fn run_current_show(
    args: &WorldDepsCurrentShowArgs,
    #[cfg(unix)] context: &AuthenticatedWorldDepsContextV1,
) -> Result<()> {
    #[cfg(unix)]
    let (cwd, cfg) = (context.launch_cwd(), context.effective_config());
    #[cfg(not(unix))]
    let cwd = env::current_dir().unwrap_or_else(|_| ".".into());
    #[cfg(not(unix))]
    let cfg = config_model::resolve_effective_config(&cwd, &Default::default())
        .context("failed to resolve effective config")?;
    let global_deps_dir =
        if cfg.world.deps.inventory_mode == config_model::WorldDepsInventoryMode::Merged {
            #[cfg(unix)]
            {
                Some(context.global_deps_dir().to_path_buf())
            }
            #[cfg(not(unix))]
            {
                Some(substrate_paths::substrate_home()?.join("deps"))
            }
        } else {
            None
        };
    let view = resolve_current_inventory_view(cwd, cfg, global_deps_dir.as_deref())?;
    let item = view.get(&args.item_name).ok_or_else(|| {
        config_model::user_error(format!("unknown deps item '{}'", args.item_name))
    })?;

    if args.explain {
        let explain = build_current_show_explain_v1(
            cwd,
            cfg,
            &view,
            &args.item_name,
            &item,
            #[cfg(unix)]
            context,
        )?;
        if args.json {
            eprintln!("{}", serde_json::to_string(&explain)?);
        } else {
            eprintln!(
                "substrate: note: enabled={} (via: global={}, workspace={})",
                explain.enabled,
                explain.enabled_via_global_patch,
                explain.enabled_via_workspace_patch
            );
            eprintln!("substrate: note: world={}", explain.world);
            for wrapper in &explain.wrappers {
                let mut extra: Vec<String> = Vec::new();
                if let Some(bash_source) = &wrapper.bash_source {
                    extra.push(format!("bash_source={bash_source}"));
                }
                if let Some(function) = &wrapper.function {
                    extra.push(format!("function={function}"));
                }
                if let Some(exec) = &wrapper.exec {
                    extra.push(format!("exec={exec}"));
                }
                if !wrapper.env_keys.is_empty() {
                    extra.push(format!("env_keys=[{}]", wrapper.env_keys.join(",")));
                }
                let extra = if extra.is_empty() {
                    "".to_string()
                } else {
                    format!(" {}", extra.join(" "))
                };
                eprintln!(
                    "substrate: note: wrapper '{}' kind={}{} invocation={}",
                    wrapper.name, wrapper.kind, extra, wrapper.invocation
                );
            }
            if let Some(why) = &explain.why {
                eprintln!("substrate: note: {why}");
            }
            if let Some(remediation) = &explain.remediation {
                eprintln!("substrate: note: remediation: {remediation}");
            }
            if let Some(instructions) = &explain.manual_instructions {
                eprintln!("substrate: note: manual_instructions:");
                for line in instructions.lines() {
                    eprintln!("  {line}");
                }
            }
            if let Some(cmd) = &explain.next_command {
                if cmd.contains('\'') {
                    eprintln!("substrate: hint: run {cmd}");
                } else {
                    eprintln!("substrate: hint: run '{cmd}'");
                }
            }
        }
    }

    if args.json {
        let (kind, name) = match &item {
            InventoryItemDefV1::Package(pkg) => ("package", pkg.name.as_str()),
            InventoryItemDefV1::Bundle(bundle) => ("bundle", bundle.name.as_str()),
        };
        let out = ShowOutputV1 {
            schema_version: 1,
            scope: "current".to_string(),
            name: name.to_string(),
            kind: kind.to_string(),
            item,
        };
        println!("{}", serde_json::to_string_pretty(&out)?);
    } else {
        println!("{}", serde_yaml::to_string(&item)?);
    }
    Ok(())
}

#[derive(Debug)]
struct InstallPlanV1 {
    apt: Vec<AptSpecV1>,
    pacman_packages: Vec<String>,
    apt_packages: Vec<String>,
    script_packages: Vec<String>,
    manual_packages: Vec<ManualPackagePlanV1>,
}

#[derive(Debug)]
struct ManualPackagePlanV1 {
    name: String,
    manual_instructions: String,
}

fn run_current_install(
    args: &WorldDepsCurrentInstallArgs,
    #[cfg(unix)] context: &AuthenticatedWorldDepsContextV1,
) -> Result<()> {
    #[cfg(unix)]
    let (cwd, cfg) = (context.launch_cwd(), context.effective_config());
    #[cfg(not(unix))]
    let cwd = env::current_dir().unwrap_or_else(|_| ".".into());
    #[cfg(not(unix))]
    let cfg = config_model::resolve_effective_config(&cwd, &Default::default())
        .context("failed to resolve effective config")?;
    let global_deps_dir =
        if cfg.world.deps.inventory_mode == config_model::WorldDepsInventoryMode::Merged {
            #[cfg(unix)]
            {
                Some(context.global_deps_dir().to_path_buf())
            }
            #[cfg(not(unix))]
            {
                Some(substrate_paths::substrate_home()?.join("deps"))
            }
        } else {
            None
        };
    let view = resolve_current_inventory_view(cwd, cfg, global_deps_dir.as_deref())?;

    let plan = compute_install_plan_v1(&view, &args.item_names)?;
    if args.verbose {
        let mut expanded = expand_items_to_packages_v1(&view, &args.item_names)?;
        expanded.sort();
        eprintln!("substrate: note: expanded packages: {}", expanded.join(","));
    }

    preflight_runtime_system_requirements_v1(
        &plan.apt,
        &plan.pacman_packages,
        args.dry_run,
        args.verbose,
        #[cfg(target_os = "linux")]
        context,
    )?;

    if args.dry_run {
        print_install_plan_v1(&plan);
        return Ok(());
    }

    apply_install_plan_v1(
        &view,
        &plan,
        ApplyInstallMode::InstallOnly,
        #[cfg(target_os = "linux")]
        context,
    )?;
    println!(
        "World deps applied: image(apt)={}, prefix(script)={}",
        plan.apt.len(),
        csv(&plan.script_packages),
    );
    println!("substrate: note: this updates the world only (enabled list not modified)");
    println!("substrate: hint: run 'substrate world deps current list applied' to verify");
    Ok(())
}

fn run_current_sync(
    args: &WorldDepsCurrentSyncArgs,
    #[cfg(unix)] context: &AuthenticatedWorldDepsContextV1,
) -> Result<()> {
    #[cfg(unix)]
    let (cwd, cfg) = (context.launch_cwd(), context.effective_config());
    #[cfg(not(unix))]
    let cwd = env::current_dir().unwrap_or_else(|_| ".".into());
    #[cfg(not(unix))]
    let cfg = config_model::resolve_effective_config(&cwd, &Default::default())
        .context("failed to resolve effective config")?;
    let global_deps_dir =
        if cfg.world.deps.inventory_mode == config_model::WorldDepsInventoryMode::Merged {
            #[cfg(unix)]
            {
                Some(context.global_deps_dir().to_path_buf())
            }
            #[cfg(not(unix))]
            {
                Some(substrate_paths::substrate_home()?.join("deps"))
            }
        } else {
            None
        };
    let view = resolve_current_inventory_view(cwd, cfg, global_deps_dir.as_deref())?;

    let item_names: Vec<String> = if args.all {
        let mut out: Vec<String> = Vec::new();
        out.extend(view.packages.keys().cloned());
        out.extend(view.bundles.keys().cloned());
        out.sort();
        out
    } else {
        cfg.world.deps.enabled.clone()
    };

    let plan = compute_install_plan_v1(&view, &item_names)?;
    if args.verbose {
        let mut expanded = expand_items_to_packages_v1(&view, &item_names)?;
        expanded.sort();
        eprintln!("substrate: note: expanded packages: {}", expanded.join(","));
    }

    preflight_runtime_system_requirements_v1(
        &plan.apt,
        &plan.pacman_packages,
        args.dry_run,
        args.verbose,
        #[cfg(target_os = "linux")]
        context,
    )?;

    if args.dry_run {
        print_install_plan_v1(&plan);
        return Ok(());
    }

    apply_install_plan_v1(
        &view,
        &plan,
        ApplyInstallMode::SyncEnabled,
        #[cfg(target_os = "linux")]
        context,
    )?;
    println!("World deps synced");
    println!("substrate: note: applied effective enabled deps list for this directory (sources: workspace, global, defaults as applicable)");
    println!("substrate: hint: run 'substrate world deps current list applied' to verify");
    Ok(())
}

#[derive(Copy, Clone, Debug)]
enum ApplyInstallMode {
    InstallOnly,
    SyncEnabled,
}

fn compute_install_plan_v1(view: &InventoryViewV1, item_names: &[String]) -> Result<InstallPlanV1> {
    let package_names = expand_items_to_packages_v1(view, item_names)?;

    let mut apt_packages: Vec<String> = Vec::new();
    let mut pacman_packages: Vec<String> = Vec::new();
    let mut script_packages: Vec<String> = Vec::new();
    let mut manual_packages: Vec<ManualPackagePlanV1> = Vec::new();

    for pkg_name in &package_names {
        let pkg = view.packages.get(pkg_name).ok_or_else(|| {
            config_model::user_error(format!(
                "invalid deps inventory: referenced package '{pkg_name}' is not visible for this platform"
            ))
        })?;

        match pkg.install.method {
            InstallMethodV1::Apt => {
                apt_packages.push(pkg.name.clone());
            }
            InstallMethodV1::Pacman => {
                pacman_packages.extend(
                    pkg.install
                        .pacman
                        .iter()
                        .map(|value| value.trim().to_string())
                        .filter(|value| !value.is_empty()),
                );
            }
            InstallMethodV1::Script => {
                script_packages.push(pkg.name.clone());
            }
            InstallMethodV1::Manual => {
                manual_packages.push(ManualPackagePlanV1 {
                    name: pkg.name.clone(),
                    manual_instructions: pkg
                        .install
                        .manual_instructions
                        .clone()
                        .unwrap_or_default(),
                });
            }
        }
    }

    let apt = normalize_apt_requirements_v1(view, &package_names)?;
    let apt_packages = dedupe_ordered(&apt_packages);
    pacman_packages.sort();
    pacman_packages.dedup();
    let script_packages = dedupe_ordered(&script_packages);

    manual_packages.sort_by(|a, b| a.name.cmp(&b.name));
    manual_packages.dedup_by(|a, b| a.name == b.name);

    Ok(InstallPlanV1 {
        apt,
        pacman_packages,
        apt_packages,
        script_packages,
        manual_packages,
    })
}

pub(crate) fn resolve_enabled_apt_requirements_v1(
    view: &InventoryViewV1,
    item_names: &[String],
) -> Result<Vec<AptSpecV1>> {
    let package_names = expand_items_to_packages_v1(view, item_names)?;
    normalize_apt_requirements_v1(view, &package_names)
}

pub(crate) fn resolve_enabled_pacman_packages_v1(
    view: &InventoryViewV1,
    item_names: &[String],
) -> Result<Vec<String>> {
    let package_names = expand_items_to_packages_v1(view, item_names)?;
    let mut packages: Vec<String> = Vec::new();

    for pkg_name in package_names {
        let pkg = view.packages.get(&pkg_name).ok_or_else(|| {
            config_model::user_error(format!(
                "invalid deps inventory: referenced package '{pkg_name}' is not visible for this platform"
            ))
        })?;
        if pkg.install.method != InstallMethodV1::Pacman {
            continue;
        }
        packages.extend(
            pkg.install
                .pacman
                .iter()
                .map(|value| value.trim().to_string())
                .filter(|value| !value.is_empty()),
        );
    }

    packages.sort();
    packages.dedup();
    Ok(packages)
}

fn normalize_apt_requirements_v1(
    view: &InventoryViewV1,
    package_names: &[String],
) -> Result<Vec<AptSpecV1>> {
    let mut grouped: HashMap<String, Vec<String>> = HashMap::new();

    for pkg_name in package_names {
        let pkg = view.packages.get(pkg_name).ok_or_else(|| {
            config_model::user_error(format!(
                "invalid deps inventory: referenced package '{pkg_name}' is not visible for this platform"
            ))
        })?;
        if pkg.install.method != InstallMethodV1::Apt {
            continue;
        }
        for spec in &pkg.install.apt {
            grouped.entry(spec.name.clone()).or_default().extend(
                spec.version
                    .iter()
                    .map(|value| value.trim().to_string())
                    .filter(|value| !value.is_empty()),
            );
        }
    }

    let mut names: Vec<String> = grouped.keys().cloned().collect();
    names.sort();

    let mut conflicts: Vec<(String, Vec<String>)> = Vec::new();
    let mut normalized: Vec<AptSpecV1> = Vec::with_capacity(names.len());

    for name in names {
        let mut versions = grouped.remove(&name).unwrap_or_default();
        versions.sort();
        versions.dedup();
        match versions.len() {
            0 => normalized.push(AptSpecV1 {
                name,
                version: None,
            }),
            1 => normalized.push(AptSpecV1 {
                name,
                version: versions.into_iter().next(),
            }),
            _ => conflicts.push((name, versions)),
        }
    }

    if conflicts.is_empty() {
        return Ok(normalized);
    }

    let mut message = String::from("substrate: conflicting APT requirements:\n");
    for (name, versions) in conflicts {
        message.push_str(&format!(
            "  - {name}: {}\n",
            versions
                .into_iter()
                .map(|version| format!("{name}={version}"))
                .collect::<Vec<_>>()
                .join(", ")
        ));
    }
    Err(config_model::user_error(message.trim_end().to_string()))
}

#[derive(Debug, Clone)]
struct AptRequirementProbeStatusV1 {
    requirement: AptSpecV1,
    satisfied: bool,
    installed_version: Option<String>,
}

fn preflight_runtime_system_requirements_v1(
    apt_requirements: &[AptSpecV1],
    pacman_requirements: &[String],
    dry_run: bool,
    verbose: bool,
    #[cfg(target_os = "linux")] context: &AuthenticatedWorldDepsContextV1,
) -> Result<()> {
    if apt_requirements.is_empty() && pacman_requirements.is_empty() {
        return Ok(());
    }

    if dry_run {
        print_normalized_apt_requirements_v1(apt_requirements);
        print_normalized_pacman_requirements_v1(pacman_requirements);
    }

    let apt_statuses = if apt_requirements.is_empty() || runtime_apt_preflight_disabled_v1() {
        None
    } else {
        Some(probe_world_apt_requirements_v1(
            apt_requirements,
            #[cfg(target_os = "linux")]
            context,
        )?)
    };
    let pacman_statuses =
        if pacman_requirements.is_empty() || runtime_pacman_preflight_disabled_v1() {
            None
        } else {
            Some(probe_world_pacman_requirements_v1(
                pacman_requirements,
                #[cfg(target_os = "linux")]
                context,
            )?)
        };

    let apt_missing = apt_statuses
        .as_ref()
        .map(|statuses| statuses.iter().any(|status| !status.satisfied))
        .unwrap_or(false);
    let pacman_missing = pacman_statuses
        .as_ref()
        .map(|statuses| statuses.iter().any(|status| !status.satisfied))
        .unwrap_or(false);

    if !apt_missing && !pacman_missing {
        return Ok(());
    }

    Err(anyhow!(WorldDepsUnmetPrerequisiteError::new(
        build_runtime_system_remediation_v1(
            apt_requirements,
            apt_statuses.as_deref().unwrap_or(&[]),
            pacman_requirements,
            pacman_statuses.as_deref().unwrap_or(&[]),
            verbose,
        )
    )))
}

fn runtime_apt_preflight_disabled_v1() -> bool {
    matches!(
        env::var("SUBSTRATE_WORLD_DEPS_SKIP_APT")
            .ok()
            .as_deref()
            .map(str::trim)
            .map(str::to_ascii_lowercase)
            .as_deref(),
        Some("1" | "true" | "yes")
    )
}

fn runtime_pacman_preflight_disabled_v1() -> bool {
    matches!(
        env::var("SUBSTRATE_WORLD_DEPS_SKIP_PACMAN")
            .ok()
            .as_deref()
            .map(str::trim)
            .map(str::to_ascii_lowercase)
            .as_deref(),
        Some("1" | "true" | "yes")
    )
}

fn print_normalized_apt_requirements_v1(requirements: &[AptSpecV1]) {
    for requirement in requirements {
        println!("{}", render_apt_requirement_v1(requirement));
    }
}

fn print_normalized_pacman_requirements_v1(requirements: &[String]) {
    for requirement in requirements {
        println!("{requirement}");
    }
}

fn render_apt_requirement_v1(requirement: &AptSpecV1) -> String {
    match &requirement.version {
        Some(version) if !version.trim().is_empty() => format!("{}={version}", requirement.name),
        _ => requirement.name.clone(),
    }
}

fn probe_world_apt_requirements_v1(
    requirements: &[AptSpecV1],
    #[cfg(target_os = "linux")] context: &AuthenticatedWorldDepsContextV1,
) -> Result<Vec<AptRequirementProbeStatusV1>> {
    let out = run_world_command_output_for_deps_with_profile(
        &build_world_apt_probe_command_v1(requirements),
        Some("/tmp"),
        Some("world-deps-probe"),
        #[cfg(target_os = "linux")]
        context,
    )
    .map_err(classify_world_backend_error)?;

    if out.exit != 0 {
        let snippet = output_snippet_for_error(&out);
        return Err(anyhow!(WorldDepsBackendUnavailableError::new(format!(
            "world apt probe failed (exit={}): {}",
            out.exit,
            if snippet.trim().is_empty() {
                "unknown error".to_string()
            } else {
                snippet
            }
        ))));
    }

    let mut observed: HashMap<String, (bool, Option<String>)> = HashMap::new();
    for line in out.stdout.lines() {
        let Some(rest) = line.strip_prefix("__SUBSTRATE_WDAP1__ ") else {
            continue;
        };
        let mut parts = rest.splitn(3, ' ');
        let Some(name) = parts.next() else {
            continue;
        };
        let satisfied = matches!(parts.next(), Some("1"));
        let installed_version = parts
            .next()
            .map(str::trim)
            .filter(|value| !value.is_empty() && *value != "-")
            .map(ToOwned::to_owned);
        observed.insert(name.to_string(), (satisfied, installed_version));
    }

    Ok(requirements
        .iter()
        .cloned()
        .map(|requirement| {
            let (satisfied, installed_version) = observed
                .get(&requirement.name)
                .cloned()
                .unwrap_or((false, None));
            AptRequirementProbeStatusV1 {
                requirement,
                satisfied,
                installed_version,
            }
        })
        .collect())
}

fn build_world_apt_probe_command_v1(requirements: &[AptSpecV1]) -> String {
    let mut script = String::new();
    script.push_str("set +e\n");
    script.push_str("probe_pkg() {\n");
    script.push_str("  pkg_name=\"$1\"\n");
    script.push_str("  expected_version=\"$2\"\n");
    script.push_str("  if ! command -v dpkg-query >/dev/null 2>&1; then\n");
    script.push_str("    printf '__SUBSTRATE_WDAP1__ %s 0 -\\n' \"$pkg_name\"\n");
    script.push_str("    return 0\n");
    script.push_str("  fi\n");
    script.push_str("  output=\"$(dpkg-query -W -f='${Status} ${Version}\\n' \"$pkg_name\" 2>/dev/null || true)\"\n");
    script.push_str("  case \"$output\" in\n");
    script.push_str("    install\\ ok\\ installed\\ *)\n");
    script.push_str("      installed_version=\"${output#install ok installed }\"\n");
    script.push_str("      if [ -n \"$expected_version\" ] && [ \"$installed_version\" != \"$expected_version\" ]; then\n");
    script.push_str(
        "        printf '__SUBSTRATE_WDAP1__ %s 0 %s\\n' \"$pkg_name\" \"$installed_version\"\n",
    );
    script.push_str("      else\n");
    script.push_str(
        "        printf '__SUBSTRATE_WDAP1__ %s 1 %s\\n' \"$pkg_name\" \"$installed_version\"\n",
    );
    script.push_str("      fi\n");
    script.push_str("      ;;\n");
    script.push_str("    *)\n");
    script.push_str("      printf '__SUBSTRATE_WDAP1__ %s 0 -\\n' \"$pkg_name\"\n");
    script.push_str("      ;;\n");
    script.push_str("  esac\n");
    script.push_str("}\n");

    for requirement in requirements {
        script.push_str("probe_pkg ");
        script.push_str(&sh_quote(&requirement.name));
        script.push(' ');
        script.push_str(&sh_quote(requirement.version.as_deref().unwrap_or("")));
        script.push('\n');
    }

    script.push_str("exit 0\n");
    script
}

#[derive(Debug, Clone)]
struct PacmanRequirementProbeStatusV1 {
    requirement: String,
    satisfied: bool,
}

fn probe_world_pacman_requirements_v1(
    requirements: &[String],
    #[cfg(target_os = "linux")] context: &AuthenticatedWorldDepsContextV1,
) -> Result<Vec<PacmanRequirementProbeStatusV1>> {
    let out = run_world_command_output_for_deps_with_profile(
        &build_world_pacman_probe_command_v1(requirements),
        Some("/tmp"),
        Some("world-deps-probe"),
        #[cfg(target_os = "linux")]
        context,
    )
    .map_err(classify_world_backend_error)?;

    if out.exit != 0 {
        let snippet = output_snippet_for_error(&out);
        return Err(anyhow!(WorldDepsBackendUnavailableError::new(format!(
            "world pacman probe failed (exit={}): {}",
            out.exit,
            if snippet.trim().is_empty() {
                "unknown error".to_string()
            } else {
                snippet
            }
        ))));
    }

    let mut observed: HashMap<String, bool> = HashMap::new();
    for line in out.stdout.lines() {
        let Some(rest) = line.strip_prefix("__SUBSTRATE_WDAP1__ ") else {
            continue;
        };
        let mut parts = rest.splitn(2, ' ');
        let Some(name) = parts.next() else {
            continue;
        };
        let satisfied = matches!(parts.next(), Some("1"));
        observed.insert(name.to_string(), satisfied);
    }

    Ok(requirements
        .iter()
        .cloned()
        .map(|requirement| {
            let satisfied = observed.get(&requirement).copied().unwrap_or(false);
            PacmanRequirementProbeStatusV1 {
                requirement,
                satisfied,
            }
        })
        .collect())
}

fn build_world_pacman_probe_command_v1(requirements: &[String]) -> String {
    let mut script = String::new();
    script.push_str("set +e\n");
    script.push_str("probe_pkg() {\n");
    script.push_str("  pkg_name=\"$1\"\n");
    script.push_str("  if ! command -v pacman >/dev/null 2>&1; then\n");
    script.push_str("    printf '__SUBSTRATE_WDAP1__ %s 0\\n' \"$pkg_name\"\n");
    script.push_str("    return 0\n");
    script.push_str("  fi\n");
    script.push_str("  if pacman -Q \"$pkg_name\" >/dev/null 2>&1; then\n");
    script.push_str("    printf '__SUBSTRATE_WDAP1__ %s 1\\n' \"$pkg_name\"\n");
    script.push_str("  else\n");
    script.push_str("    printf '__SUBSTRATE_WDAP1__ %s 0\\n' \"$pkg_name\"\n");
    script.push_str("  fi\n");
    script.push_str("}\n");

    for requirement in requirements {
        script.push_str("probe_pkg ");
        script.push_str(&sh_quote(requirement));
        script.push('\n');
    }

    script.push_str("exit 0\n");
    script
}

fn build_runtime_system_remediation_v1(
    apt_requirements: &[AptSpecV1],
    apt_statuses: &[AptRequirementProbeStatusV1],
    pacman_requirements: &[String],
    pacman_statuses: &[PacmanRequirementProbeStatusV1],
    verbose: bool,
) -> String {
    const PROVISION_COMMAND: &str = "substrate world enable --provision-deps";

    let mut lines: Vec<String> = vec![
        "substrate: system-package world deps are not satisfied in this world.".to_string(),
        "substrate: provision system packages during world enable, then retry.".to_string(),
        PROVISION_COMMAND.to_string(),
    ];

    if cfg!(windows) {
        lines.push(
            "substrate world enable --provision-deps is unsupported on Windows. Substrate will not mutate the Windows host OS."
                .to_string(),
        );
    } else {
        lines.push("Substrate will not mutate the host OS.".to_string());
    }

    let unsatisfied_apt = apt_statuses
        .iter()
        .filter(|status| !status.satisfied)
        .map(|status| match &status.installed_version {
            Some(version) => format!(
                "{} (installed {})",
                render_apt_requirement_v1(&status.requirement),
                version
            ),
            None => render_apt_requirement_v1(&status.requirement),
        })
        .collect::<Vec<_>>();
    if !unsatisfied_apt.is_empty() {
        lines.push(format!(
            "substrate: unsatisfied APT requirements: {}",
            unsatisfied_apt.join(", ")
        ));
    }

    let unsatisfied_pacman = pacman_statuses
        .iter()
        .filter(|status| !status.satisfied)
        .map(|status| status.requirement.clone())
        .collect::<Vec<_>>();
    if !unsatisfied_pacman.is_empty() {
        lines.push(format!(
            "substrate: unsatisfied pacman requirements: {}",
            unsatisfied_pacman.join(", ")
        ));
    }

    if verbose {
        if !apt_requirements.is_empty() {
            lines.push("substrate: normalized APT requirements:".to_string());
            lines.extend(apt_requirements.iter().map(render_apt_requirement_v1));
        }
        if !pacman_requirements.is_empty() {
            lines.push("substrate: normalized pacman requirements:".to_string());
            lines.extend(pacman_requirements.iter().cloned());
        }
    }

    lines.join("\n")
}

fn apply_install_plan_v1(
    view: &InventoryViewV1,
    plan: &InstallPlanV1,
    mode: ApplyInstallMode,
    #[cfg(target_os = "linux")] context: &AuthenticatedWorldDepsContextV1,
) -> Result<()> {
    if !plan.manual_packages.is_empty() {
        eprintln!("MANUAL (blocked):");
        for pkg in &plan.manual_packages {
            eprintln!("  - {}", pkg.name);
            let instructions = pkg.manual_instructions.trim();
            if instructions.is_empty() {
                continue;
            }
            for line in instructions.lines() {
                eprintln!("      {line}");
            }
        }
        return Err(anyhow!(WorldDepsUnmetPrerequisiteError::new(
            "manual install required"
        )));
    }

    let mut packages: Vec<String> = Vec::new();
    packages.extend(plan.apt_packages.iter().cloned());
    packages.extend(plan.script_packages.iter().cloned());
    ensure_no_entrypoint_collisions_v1(view, &packages)?;

    ensure_world_backend_available(
        #[cfg(target_os = "linux")]
        context,
    )?;

    if matches!(mode, ApplyInstallMode::SyncEnabled) {
        let keep_names = collect_world_deps_bin_keep_names_v1(view, &packages)?;
        reconcile_world_deps_bin_v1(
            &keep_names,
            #[cfg(target_os = "linux")]
            context,
        )
        .context("failed to reconcile world-deps wrappers")?;
    }

    apply_apt_entrypoint_wrappers_v1(
        view,
        &plan.apt_packages,
        #[cfg(target_os = "linux")]
        context,
    )?;

    for pkg_name in &plan.script_packages {
        let pkg = view.packages.get(pkg_name).ok_or_else(|| {
            config_model::user_error(format!(
                "invalid deps inventory: referenced package '{pkg_name}' is not visible for this platform"
            ))
        })?;
        apply_script_package_v1(
            pkg,
            #[cfg(target_os = "linux")]
            context,
        )
        .with_context(|| format!("failed to apply script package '{pkg_name}'"))?;
    }

    let statuses = query_world_package_entrypoint_presence(
        view,
        &plan.script_packages,
        #[cfg(target_os = "linux")]
        context,
    )?;
    let mut missing = plan
        .script_packages
        .iter()
        .filter(|name| !statuses.get(*name).copied().unwrap_or(false))
        .cloned()
        .collect::<Vec<_>>();
    if !missing.is_empty() {
        missing.sort();
        missing.dedup();
        return Err(anyhow!(WorldDepsUnmetPrerequisiteError::new(format!(
            "after install, package(s) still missing in world: {}; ensure script installs create runnable entrypoints under $SUBSTRATE_WORLD_DEPS_GUEST_BIN_DIR and retry",
            missing.join(",")
        ))));
    }

    Ok(())
}

fn collect_world_deps_bin_keep_names_v1(
    view: &InventoryViewV1,
    package_names: &[String],
) -> Result<Vec<String>> {
    let mut keep: Vec<String> = Vec::new();

    for pkg_name in package_names {
        let pkg = view.packages.get(pkg_name).ok_or_else(|| {
            config_model::user_error(format!(
                "invalid deps inventory: referenced package '{pkg_name}' is not visible for this platform"
            ))
        })?;

        if pkg.runnable && !pkg.entrypoints.is_empty() {
            for entrypoint in &pkg.entrypoints {
                validate_world_deps_entrypoint_filename(entrypoint)?;
                keep.push(entrypoint.clone());
            }
        }

        for wrapper in &pkg.wrappers {
            validate_world_deps_bin_filename(&wrapper.name)?;
            keep.push(wrapper.name.clone());
        }
    }

    keep.sort();
    keep.dedup();
    Ok(keep)
}

fn reconcile_world_deps_bin_v1(
    keep_names: &[String],
    #[cfg(target_os = "linux")] context: &AuthenticatedWorldDepsContextV1,
) -> Result<()> {
    let cmd = build_world_deps_bin_reconcile_command_v1(keep_names);
    run_world_command_checked_for_deps(
        &cmd,
        Some("/tmp"),
        #[cfg(target_os = "linux")]
        context,
    )?;
    Ok(())
}

fn build_world_deps_bin_reconcile_command_v1(keep_names: &[String]) -> String {
    let mut cmd = String::new();
    cmd.push_str("set -eu\n");
    cmd.push_str("world_deps_bin=\"${SUBSTRATE_WORLD_DEPS_GUEST_BIN_DIR:-/var/lib/substrate/world-deps/bin}\"\n");
    cmd.push_str("world_deps_bin=\"${world_deps_bin%/}\"\n");
    cmd.push_str("mkdir -p \"$world_deps_bin\"\n");
    cmd.push_str("should_keep() {\n");
    if keep_names.is_empty() {
        cmd.push_str("  return 1\n");
    } else {
        cmd.push_str("  name=\"$1\"\n");
        cmd.push_str("  for keep in");
        for keep in keep_names {
            cmd.push(' ');
            cmd.push_str(&sh_quote(keep));
        }
        cmd.push_str("; do\n");
        cmd.push_str("    [ \"$name\" = \"$keep\" ] && return 0\n");
        cmd.push_str("  done\n");
        cmd.push_str("  return 1\n");
    }
    cmd.push_str("}\n");
    cmd.push_str("for path in \"$world_deps_bin\"/*; do\n");
    cmd.push_str("  [ -e \"$path\" ] || continue\n");
    cmd.push_str("  name=\"${path##*/}\"\n");
    cmd.push_str("  case \"$name\" in\n");
    cmd.push_str("    .*) continue ;;\n");
    cmd.push_str("  esac\n");
    cmd.push_str("  if should_keep \"$name\"; then\n");
    cmd.push_str("    continue\n");
    cmd.push_str("  fi\n");
    cmd.push_str("  if [ -L \"$path\" ]; then\n");
    cmd.push_str("    rm -f \"$path\"\n");
    cmd.push_str("    continue\n");
    cmd.push_str("  fi\n");
    cmd.push_str("  if [ -d \"$path\" ]; then\n");
    cmd.push_str(
        "    echo \"substrate: world deps wrapper collision: $path is a directory\" >&2\n",
    );
    cmd.push_str("    exit 5\n");
    cmd.push_str("  fi\n");
    cmd.push_str("  rm -f \"$path\"\n");
    cmd.push_str("done\n");
    cmd.push_str("exit 0\n");
    cmd
}

fn ensure_no_entrypoint_collisions_v1(
    view: &InventoryViewV1,
    package_names: &[String],
) -> Result<()> {
    let mut owners: HashMap<String, HashSet<String>> = HashMap::new();

    for name in package_names {
        let pkg = view.packages.get(name).ok_or_else(|| {
            config_model::user_error(format!(
                "invalid deps inventory: referenced package '{name}' is not visible for this platform"
            ))
        })?;
        if !pkg.runnable || pkg.entrypoints.is_empty() {
            continue;
        }

        let mut seen: HashSet<&str> = HashSet::new();
        for entrypoint in &pkg.entrypoints {
            if !seen.insert(entrypoint.as_str()) {
                continue;
            }
            owners
                .entry(entrypoint.clone())
                .or_default()
                .insert(pkg.name.clone());
        }
    }

    let mut collisions: Vec<(String, Vec<String>)> = Vec::new();
    for (entrypoint, pkgs) in owners {
        if pkgs.len() <= 1 {
            continue;
        }
        let mut pkgs: Vec<String> = pkgs.into_iter().collect();
        pkgs.sort();
        collisions.push((entrypoint, pkgs));
    }

    if collisions.is_empty() {
        return Ok(());
    }
    collisions.sort_by(|a, b| a.0.cmp(&b.0));

    let mut lines: Vec<String> = Vec::new();
    for (entrypoint, pkgs) in collisions {
        lines.push(format!(
            "entrypoint '{entrypoint}' is claimed by multiple packages: {}",
            pkgs.join(",")
        ));
    }

    let message = if lines.len() == 1 {
        format!("world deps wrapper collision: {}", lines[0])
    } else {
        format!(
            "world deps wrapper collisions:\n  - {}",
            lines.join("\n  - ")
        )
    };
    Err(anyhow!(WorldDepsSafetyViolationError::new(message)))
}

fn apply_apt_entrypoint_wrappers_v1(
    view: &InventoryViewV1,
    apt_packages: &[String],
    #[cfg(target_os = "linux")] context: &AuthenticatedWorldDepsContextV1,
) -> Result<()> {
    if apt_packages.is_empty() {
        return Ok(());
    }

    let mut entrypoints: Vec<String> = Vec::new();
    for pkg_name in apt_packages {
        let pkg = view.packages.get(pkg_name).ok_or_else(|| {
            config_model::user_error(format!(
                "invalid deps inventory: referenced package '{pkg_name}' is not visible for this platform"
            ))
        })?;
        if pkg.install.method != InstallMethodV1::Apt {
            continue;
        }
        if !pkg.runnable || pkg.entrypoints.is_empty() {
            continue;
        }
        for entrypoint in &pkg.entrypoints {
            validate_world_deps_entrypoint_filename(entrypoint)?;
            entrypoints.push(entrypoint.clone());
        }
    }

    entrypoints.sort();
    entrypoints.dedup();
    if entrypoints.is_empty() {
        return Ok(());
    }

    let cmd = build_world_apt_entrypoint_wrapper_command_v1(&entrypoints);
    run_world_command_checked_for_deps(
        &cmd,
        Some("/tmp"),
        #[cfg(target_os = "linux")]
        context,
    )
    .context("failed to create apt entrypoint wrappers")?;
    Ok(())
}

fn build_world_apt_entrypoint_wrapper_command_v1(entrypoints: &[String]) -> String {
    let mut cmd = String::new();
    cmd.push_str("set -eu\n");
    cmd.push_str("world_deps_bin=\"${SUBSTRATE_WORLD_DEPS_GUEST_BIN_DIR:-/var/lib/substrate/world-deps/bin}\"\n");
    cmd.push_str("world_deps_bin=\"${world_deps_bin%/}\"\n");
    cmd.push_str("mkdir -p \"$world_deps_bin\"\n");

    for (idx, entrypoint) in entrypoints.iter().enumerate() {
        let delimiter = format!("__SUBSTRATE_WDH1_APT_WRAPPER_{idx}__");

        cmd.push_str("wrapper=\"$world_deps_bin/");
        cmd.push_str(entrypoint);
        cmd.push_str("\"\n");
        cmd.push_str("if [ -L \"$wrapper\" ]; then\n");
        cmd.push_str(
            "  echo \"substrate: world deps wrapper collision: $wrapper is a symlink\" >&2\n",
        );
        cmd.push_str("  exit 5\n");
        cmd.push_str("fi\n");
        cmd.push_str("if [ -d \"$wrapper\" ]; then\n");
        cmd.push_str(
            "  echo \"substrate: world deps wrapper collision: $wrapper is a directory\" >&2\n",
        );
        cmd.push_str("  exit 5\n");
        cmd.push_str("fi\n");
        // NOTE: Do not escape quotes as `\"` here. In POSIX sh, `\"` is a literal quote character,
        // which breaks mktemp by making the template start with `"` (nonexistent directory).
        cmd.push_str("tmp=\"$(mktemp \"$world_deps_bin/.substrate-wdh1-wrapper.XXXXXX\")\"\n");
        cmd.push_str("cat > \"$tmp\" <<'");
        cmd.push_str(&delimiter);
        cmd.push_str("'\n");
        cmd.push_str("#!/bin/sh\n");
        cmd.push_str("set -eu\n");
        // Resolve the entrypoint without recursing into the wrapper itself. Many apt packages
        // install binaries outside `/usr/bin` (e.g. `/usr/games`), so do not assume a fixed path.
        cmd.push_str("PATH='/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin:/usr/games:/usr/local/games'\n");
        cmd.push_str("resolved=\"$(command -v ");
        cmd.push_str(entrypoint);
        cmd.push_str(" 2>/dev/null || true)\"\n");
        cmd.push_str("if [ -z \"$resolved\" ]; then\n");
        cmd.push_str("  echo \"substrate: world deps apt entrypoint not found: ");
        cmd.push_str(entrypoint);
        cmd.push_str("\" >&2\n");
        cmd.push_str("  exit 127\n");
        cmd.push_str("fi\n");
        cmd.push_str("exec \"$resolved\" \"$@\"\n");
        cmd.push_str(&delimiter);
        cmd.push('\n');
        cmd.push_str("chmod 0755 \"$tmp\"\n");
        cmd.push_str("mv -f \"$tmp\" \"$wrapper\"\n");
    }

    cmd
}

#[cfg(test)]
mod tests {
    use super::*;
    #[cfg(unix)]
    use crate::execution::install_bootstrap::current_unix_principal_and_home;
    #[cfg(unix)]
    use serial_test::serial;
    #[cfg(unix)]
    use std::fs;
    #[cfg(unix)]
    use std::os::unix::fs::PermissionsExt;
    #[cfg(unix)]
    use std::process::Command;
    #[cfg(unix)]
    use tempfile::{Builder, TempDir};
    #[cfg(unix)]
    use transport_api_types::{InstallBootstrapContextCarrierV1, InstallBootstrapContextV1};

    #[test]
    fn test_world_script_install_command_carries_one_payload() {
        // Rendering only: do not execute this real installer or inspect a live world.
        let target = config_projection::CODEX_RELEASE_TARGET_V1;
        let spec = resolve_codex_runtime_install_spec_for_target_v1(target).unwrap();
        let script = render_codex_runtime_install_script_v1(
            super::super::inventory::codex_runtime_install_script_template_v1(),
            &spec,
        );
        // The built-in codex-runtime package has no wrappers.
        let command = build_world_script_install_command_v1(&script, &[]);
        // world/src/exec.rs transports the command as SUBSTRATE_INNER_CMD.
        let environment_string = format!("SUBSTRATE_INNER_CMD={command}");
        eprintln!(
            "installer target={target} body_bytes={} command_bytes={} environment_string_bytes={} environment_string_with_nul_bytes={}",
            script.len(), command.len(), environment_string.len(), environment_string.len() + 1,
        );
        assert_eq!(command.matches(&script).count(), 1, "{target}");

        // A length discriminator also catches duplication when the payload changes.
        let short = build_world_script_install_command_v1("# size probe\n", &[]);
        let padding = "# payload size discrimination: ' \" $() ` ; & |\n".repeat(2048);
        let long = build_world_script_install_command_v1(&format!("# size probe\n{padding}"), &[]);
        assert_eq!(long.len() - short.len(), padding.len());
    }

    #[cfg(unix)]
    #[test]
    fn test_world_script_install_command_executes_both_interpreters() {
        for bash_available in [true, false] {
            for exit_code in [0, 37] {
                for ending in ["", "\n", "\n\n"] {
                    let fixture = TempDir::new().unwrap();
                    let tools = fixture.path().join("tools");
                    let bin = fixture.path().join("guest bin ' $ ;");
                    fs::create_dir(&tools).unwrap();
                    fs::create_dir(fixture.path().join("home")).unwrap();
                    for tool in ["cat", "mkdir"] {
                        std::os::unix::fs::symlink(format!("/bin/{tool}"), tools.join(tool))
                            .unwrap();
                    }
                    // Preserve the actual interpreter options while disabling host
                    // profile loading. The payload checks Bash's real login state.
                    let (interpreter, launcher) = if bash_available {
                        ("bash", "exec /bin/bash --noprofile --norc \"$@\"\n")
                    } else {
                        ("sh", "exec /bin/sh \"$@\"\n")
                    };
                    let launcher_path = tools.join(interpreter);
                    fs::write(
                        &launcher_path,
                        format!(
                            "#!/bin/sh\n[ \"$#\" -eq 2 ] || exit 91\nprintf '%s\\n' \"$1\" > \"$FIXTURE_ROOT/options\"\nprintf '%s' \"$2\" > \"$FIXTURE_ROOT/payload\"\n{launcher}"
                        ),
                    )
                    .unwrap();
                    fs::set_permissions(&launcher_path, fs::Permissions::from_mode(0o755)).unwrap();
                    let chmod = tools.join("chmod");
                    fs::write(
                        &chmod,
                        r#"#!/bin/sh
[ -e "$FIXTURE_ROOT/installer-complete" ] || exit 92
if [ "$2" = "$SUBSTRATE_WORLD_DEPS_GUEST_BIN_DIR/first" ]; then
  [ ! -e "$SUBSTRATE_WORLD_DEPS_GUEST_BIN_DIR/second" ] || exit 93
fi
printf '%s\n' "$2" >> "$FIXTURE_ROOT/wrapper-order"
exec /bin/chmod "$@"
"#,
                    )
                    .unwrap();
                    fs::set_permissions(&chmod, fs::Permissions::from_mode(0o755)).unwrap();
                    let script = format!(
                        r#"if [ "$FIXTURE_EXPECT_BASH" = 1 ]; then
  [ -n "${{BASH_VERSION:-}}" ] || exit 80
  shopt -q login_shell || exit 81
  printf bash > "$FIXTURE_ROOT/interpreter"
else
  if [ -n "${{BASH_VERSION:-}}" ]; then
    if shopt -q login_shell; then exit 86; fi
  fi
  printf sh > "$FIXTURE_ROOT/interpreter"
fi
[ ! -e "$SUBSTRATE_WORLD_DEPS_GUEST_BIN_DIR/first" ] || exit 82
[ ! -e "$SUBSTRATE_WORLD_DEPS_GUEST_BIN_DIR/second" ] || exit 83
SCRIPT_LEVEL=child
printf '%s\n' "$SCRIPT_LEVEL" "$(printf '%s' substitution)" > "$FIXTURE_ROOT/expanded"
cat <<'LITERAL_PAYLOAD' > "$FIXTURE_ROOT/literal"
literal ' " $SCRIPT_LEVEL $(exit 84) `exit 85` ; & | < > * ? [ ] \
second line
LITERAL_PAYLOAD
printf '%s\n' "$SUBSTRATE_WORLD_DEPS_GUEST_BIN_DIR" > "$FIXTURE_ROOT/bin-path"
printf done > "$FIXTURE_ROOT/installer-complete"
exit {exit_code}{ending}"#
                    );
                    let wrappers = [
                        WrapperFileV1 {
                            name: "first".into(),
                            body: "#!/bin/sh\nprintf '%s' 'literal $() ` ; \\\"'".into(),
                        },
                        WrapperFileV1 {
                            name: "second".into(),
                            body: "#!/bin/sh\nprintf '%s' \"$HOME\"\n\n".into(),
                        },
                    ];
                    let command = build_world_script_install_command_v1(&script, &wrappers);
                    let output = Command::new("/bin/sh")
                        .args(["-c", &command])
                        .env_clear()
                        .env("PATH", &tools)
                        .env("HOME", fixture.path().join("home"))
                        .env("FIXTURE_ROOT", fixture.path())
                        .env(
                            "FIXTURE_EXPECT_BASH",
                            if bash_available { "1" } else { "0" },
                        )
                        .env("SCRIPT_LEVEL", "outer")
                        .env("SUBSTRATE_WORLD_DEPS_GUEST_BIN_DIR", &bin)
                        .current_dir(fixture.path())
                        .output()
                        .unwrap();
                    assert_eq!(
                        output.status.code(),
                        Some(exit_code),
                        "bash={bash_available} ending={ending:?}: {}",
                        String::from_utf8_lossy(&output.stderr)
                    );
                    assert_eq!(
                        fs::read_to_string(fixture.path().join("options")).unwrap(),
                        if bash_available { "-lc\n" } else { "-c\n" }
                    );
                    assert_eq!(
                        fs::read_to_string(fixture.path().join("payload")).unwrap(),
                        script.trim_end_matches('\n')
                    );
                    assert_eq!(
                        fs::read_to_string(fixture.path().join("interpreter")).unwrap(),
                        interpreter
                    );
                    assert_eq!(
                        fs::read(fixture.path().join("expanded")).unwrap(),
                        b"child\nsubstitution\n"
                    );
                    assert_eq!(
                        fs::read_to_string(fixture.path().join("literal")).unwrap(),
                        "literal ' \" $SCRIPT_LEVEL $(exit 84) `exit 85` ; & | < > * ? [ ] \\\nsecond line\n"
                    );
                    assert_eq!(
                        fs::read_to_string(fixture.path().join("bin-path")).unwrap(),
                        format!("{}\n", bin.display())
                    );
                    if exit_code == 0 {
                        for wrapper in &wrappers {
                            let path = bin.join(&wrapper.name);
                            let mut expected = wrapper.body.clone();
                            if !expected.ends_with('\n') {
                                expected.push('\n');
                            }
                            assert_eq!(fs::read(&path).unwrap(), expected.as_bytes());
                            assert_eq!(
                                fs::metadata(path).unwrap().permissions().mode() & 0o777,
                                0o755
                            );
                        }
                        assert_eq!(
                            fs::read_to_string(fixture.path().join("wrapper-order")).unwrap(),
                            format!("{}/first\n{}/second\n", bin.display(), bin.display())
                        );
                    } else {
                        assert!(!bin.join("first").exists());
                        assert!(!bin.join("second").exists());
                        assert!(!fixture.path().join("wrapper-order").exists());
                    }
                }
            }
        }
    }

    #[cfg(unix)]
    #[test]
    #[serial]
    fn authenticated_scope_mutations_select_a_and_leave_conflicting_b_unchanged() {
        let _authority_env = crate::execution::AuthorityEnvTestGuard::preserve();
        let (principal, account_home) =
            current_unix_principal_and_home().expect("current Unix principal");
        let transport_api_types::PlatformPrincipalV1::Unix { account, uid } = principal else {
            panic!("expected Unix principal");
        };
        let temp = Builder::new()
            .prefix("substrate-world-deps-scopes-")
            .tempdir_in(account_home)
            .expect("secure scope fixture");
        let selected_a = temp.path().join("selected-a");
        let ambient_b = temp.path().join("ambient-b");
        for prefix in [&selected_a, &ambient_b] {
            fs::create_dir(prefix).expect("create authority prefix");
            fs::set_permissions(prefix, fs::Permissions::from_mode(0o700))
                .expect("secure authority prefix");
            fs::write(
                prefix.join("config.yaml"),
                "world:\n  deps:\n    builtins: disabled\n    inventory_mode: merged\n    enabled: []\n",
            )
            .expect("write config");
            fs::set_permissions(
                prefix.join("config.yaml"),
                fs::Permissions::from_mode(0o600),
            )
            .expect("secure authenticated fixture config");
        }
        let package = |name: &str| {
            format!(
                "version: 1\nname: {name}\nrunnable: false\ninstall:\n  method: manual\n  manual_instructions: test only\n"
            )
        };
        for (prefix, name) in [(&selected_a, "selected-only"), (&ambient_b, "ambient-only")] {
            let path = prefix.join("deps/packages").join(format!("{name}.yaml"));
            fs::create_dir_all(path.parent().expect("package parent")).expect("create inventory");
            fs::write(path, package(name)).expect("write package");
        }
        let workspace = temp.path().join("workspace");
        fs::create_dir_all(workspace.join(".substrate/deps/packages"))
            .expect("create workspace inventory");
        fs::write(
            workspace.join(".substrate/workspace.yaml"),
            "world:\n  deps:\n    enabled: []\n",
        )
        .expect("write workspace config");
        fs::write(
            workspace.join(".substrate/deps/packages/workspace-only.yaml"),
            package("workspace-only"),
        )
        .expect("write workspace package");

        let install_context = InstallBootstrapContextV1::new_unix(
            selected_a.to_str().expect("UTF-8 selected prefix"),
            &account,
            uid,
        )
        .expect("valid install context");
        let carrier = InstallBootstrapContextCarrierV1::from_context(install_context)
            .expect("committed install context");
        std::env::set_var("SUBSTRATE_HOME", &ambient_b);
        std::env::set_var("SUBSTRATE_ROOT", &ambient_b);
        let context = crate::builtins::world_deps::bind_authenticated_world_deps_context_v1(
            &carrier,
            &workspace,
            &config_model::CliConfigOverrides::default(),
        )
        .expect("bind authenticated scope context");
        let ambient_before = fs::read(ambient_b.join("config.yaml")).expect("read ambient config");

        run_global_add(
            &WorldDepsScopedMutateArgs {
                item_names: vec!["selected-only".to_string()],
                json: false,
            },
            &context,
        )
        .expect("mutate selected global config");
        run_workspace_add(
            &WorldDepsScopedMutateArgs {
                item_names: vec!["workspace-only".to_string()],
                json: false,
            },
            &context,
        )
        .expect("mutate explicit workspace config");

        let selected_after =
            fs::read_to_string(selected_a.join("config.yaml")).expect("read selected config");
        let workspace_after = fs::read_to_string(workspace.join(".substrate/workspace.yaml"))
            .expect("read workspace config");
        assert!(selected_after.contains("selected-only"));
        assert!(workspace_after.contains("workspace-only"));
        assert_eq!(
            fs::read(ambient_b.join("config.yaml")).expect("read ambient config after"),
            ambient_before
        );
    }

    #[test]
    fn wdh1_mktemp_template_is_shell_quoted_not_literal_quotes() {
        let cmd = build_world_apt_entrypoint_wrapper_command_v1(&["sl".to_string()]);
        assert!(
            cmd.contains("mktemp \"$world_deps_bin/.substrate-wdh1-wrapper.XXXXXX\""),
            "expected mktemp template to be shell-quoted: {cmd}"
        );
        assert!(
            !cmd.contains("mktemp \\\"$world_deps_bin/.substrate-wdh1-wrapper.XXXXXX\\\""),
            "expected mktemp template to not contain literal quotes: {cmd}"
        );
    }

    #[test]
    fn wdh1_apt_wrapper_resolves_entrypoint_without_assuming_usr_bin() {
        let cmd = build_world_apt_entrypoint_wrapper_command_v1(&["sl".to_string()]);
        assert!(
            cmd.contains("PATH='/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin:/usr/games:/usr/local/games'"),
            "expected wrapper to resolve entrypoint with a sanitized PATH including /usr/games: {cmd}"
        );
        assert!(
            cmd.contains("resolved=\"$(command -v sl 2>/dev/null || true)\""),
            "expected wrapper to use command -v for entrypoint resolution: {cmd}"
        );
        assert!(
            !cmd.contains("exec /usr/bin/sl"),
            "expected wrapper to not hardcode /usr/bin for apt entrypoints: {cmd}"
        );
    }

    #[test]
    fn world_deps_codex_runtime_selects_trusted_0156_independently_of_uaa_metadata() {
        let spec = resolve_codex_runtime_install_spec_for_target_v1("x86_64-unknown-linux-musl")
            .expect("resolve validated codex runtime");

        let advertised =
            agent_api::resolve_runtime_support("codex", "x86_64-unknown-linux-musl").unwrap();
        assert_eq!(
            advertised.version, "0.125.0",
            "consumed UAA metadata is deliberately unchanged"
        );
        assert_eq!(
            spec.release_catalog.selected.get(&spec.target_triple),
            Some(&spec.version)
        );
        assert_eq!(
            spec.archive_name,
            spec.release_input.manifest.artifacts[0].archive.name
        );
        assert_eq!(
            spec.archive_sha256,
            spec.release_input.manifest.artifacts[0].archive.sha256
        );
        assert_eq!(
            spec.extracted_executable_sha256,
            spec.release_input.manifest.artifacts[0].extracted.sha256
        );
        //156 remains an immutable measured subject even when a later selection changes.
        let historical = spec
            .release_catalog
            .releases
            .iter()
            .find(|r| r.version == "0.156.1")
            .expect("retained156 manifest");
        assert_eq!(
            historical.artifacts[0].archive.name,
            "codex-x86_64-unknown-linux-musl.tar.gz"
        );
        assert_eq!(historical.artifacts[0].archive.url,"https://github.com/openai/codex/releases/download/rust-v0.156.1/codex-x86_64-unknown-linux-musl.tar.gz");
        assert_eq!(
            historical.artifacts[0].archive.sha256,
            "aff46539a83aff86e3c62c592bce2c50d95391f9df289afaf03a50c01d14533d"
        );
        assert_eq!(
            historical.artifacts[0].extracted.sha256,
            "0b2e9301d6100dddda3b9d5c80ebaeaa3a2f1962388f2f36f6b96a9f08b1f33f"
        );
    }

    #[test]
    fn world_deps_codex_runtime_support_fails_closed_for_validated_but_unmapped_target() {
        let err = resolve_codex_runtime_install_spec_for_target_v1("aarch64-unknown-linux-musl")
            .expect_err("validated-but-unmapped target should fail closed");

        assert!(
            format!("{err:#}").contains("guest target 'aarch64-unknown-linux-musl'"),
            "expected validated guest target diagnostics, got: {err:#}"
        );
        assert!(
            format!("{err:#}").contains("no pinned official Codex release mapping"),
            "expected unmapped release-mapping diagnostics, got: {err:#}"
        );
    }

    #[test]
    fn world_deps_codex_runtime_guest_arch_mapping_normalizes_arm64() {
        assert_eq!(
            map_codex_runtime_target_triple_for_guest_arch_v1("arm64")
                .expect("normalize arm64 guest arch"),
            "aarch64-unknown-linux-musl"
        );
    }

    #[test]
    fn world_deps_codex_runtime_guest_arch_mapping_fails_closed_for_unsupported_arch() {
        let err = map_codex_runtime_target_triple_for_guest_arch_v1("riscv64")
            .expect_err("unsupported guest arch should fail closed");

        assert!(
            format!("{err:#}").contains("unsupported for guest arch 'riscv64'"),
            "expected guest arch diagnostics, got: {err:#}"
        );
        assert!(
            format!("{err:#}").contains("guest target triple"),
            "expected guest target derivation diagnostics, got: {err:#}"
        );
    }

    #[test]
    fn world_deps_codex_runtime_script_records_self_contained_posture() {
        let spec =
            resolve_codex_runtime_install_spec_for_target_v1("x86_64-unknown-linux-musl").unwrap();
        let script = render_codex_runtime_install_script_v1(
            crate::builtins::world_deps::inventory::codex_runtime_install_script_template_v1(),
            &spec,
        );
        assert!(script.contains("no Node/npm runtime bundle"));
        assert!(script.contains("CodexOfficialRelease"));
        for message in [
            "substrate: downloading Codex runtime",
            "substrate: staging Codex runtime",
            "substrate: installing Codex runtime",
            "with wget",
            "with python3",
            "with curl",
        ] {
            assert!(
                script.contains(message),
                "missing progress breadcrumb: {message}"
            );
        }
        assert!(!script.contains("source_stream = \"Codex0125OfficialArchive\""));
        assert!(script.contains("codex-code-mode-host"));
        assert!(script.contains("entries[0].isreg()"));
        assert!(script.contains("libc.renameat2"));
        assert!(!script.contains("__SUBSTRATE_CODEX_"));
        assert!(script.contains("--max-time 900"));
        assert!(script.contains("run_download_with_timeout 900"));
        assert!(script.find("flock -x 9").unwrap() < script.find("libc.renameat2").unwrap());
    }

    #[cfg(unix)]
    #[test]
    #[ignore = "designated non-executing official pair proof; requires SUBSTRATE_CODEX_0156_ARCHIVE_DIR"]
    fn world_deps_codex_runtime_install_script_is_idempotent_and_links_guest_entrypoint() {
        use sha2::{Digest, Sha256};
        use std::os::unix::fs::MetadataExt;
        let spec =
            resolve_codex_runtime_install_spec_for_target_v1("x86_64-unknown-linux-musl").unwrap();
        let archives = PathBuf::from(
            std::env::var("SUBSTRATE_CODEX_0156_ARCHIVE_DIR")
                .expect("designated official-pair proof requires SUBSTRATE_CODEX_0156_ARCHIVE_DIR"),
        );
        let fixture = TempDir::new().unwrap();
        let root = fixture.path().join("world-deps");
        let downloads = root.join("codex-runtime/downloads");
        fs::create_dir_all(&downloads).unwrap();
        for pin in [
            &spec.release_bundle.main,
            &spec.release_bundle.code_mode_host,
        ] {
            let source = archives.join(&pin.archive_name);
            assert_eq!(
                format!("{:x}", Sha256::digest(fs::read(&source).unwrap())),
                pin.archive_sha256
            );
            fs::copy(
                source,
                downloads.join(format!("{}-{}", spec.version, pin.archive_name)),
            )
            .unwrap();
        }
        // Every selected acquisition command is present and fails closed. The
        // Python fallback is unreachable because wget is present.
        let stubs = fixture.path().join("deny-acquisition");
        fs::create_dir(&stubs).unwrap();
        let acquisition_log = fixture.path().join("acquisition-attempts");
        for name in ["wget", "curl"] {
            let path = stubs.join(name);
            fs::write(
                &path,
                format!(
                    "#!/bin/sh\nprintf '%s\\n' acquisition-denied >> '{}'\nexit 99\n",
                    acquisition_log.display()
                ),
            )
            .unwrap();
            fs::set_permissions(path, fs::Permissions::from_mode(0o755)).unwrap();
        }
        let system = fixture.path().join("etc-codex");
        fs::create_dir(&system).unwrap();
        fs::set_permissions(&system, fs::Permissions::from_mode(0o755)).unwrap();
        let mut script = render_codex_runtime_install_script_v1(
            super::super::inventory::codex_runtime_install_script_template_v1(),
            &spec,
        )
        .replace("/var/lib/substrate/world-deps", root.to_str().unwrap())
        .replace("/etc/codex", system.to_str().unwrap())
        // Only filesystem UID/GID observations and the group lookup are
        // substituted for this disposable nonprivileged fixture. Pins,
        // schema, provenance, hashes, and all guards are unchanged.
        .replace("grp.getgrnam(\"substrate\").gr_gid", "os.getgid()")
        .replace("target_stat.st_uid", "0")
        .replace("host_stat.st_uid", "0")
        .replace("system_target.st_uid", "0")
        .replace("system_target.st_gid", "0");
        let common = fixture.path().join("common-support");
        fs::create_dir(&common).unwrap();
        for (index, original) in [
            "/etc/hosts",
            "/etc/nsswitch.conf",
            "/etc/passwd",
            "/etc/group",
            "/etc/resolv.conf",
            "/etc/ssl/certs/ca-certificates.crt",
        ]
        .into_iter()
        .enumerate()
        {
            let file = common.join(format!("support-{index}"));
            fs::write(&file, format!("fixture support {original}\n")).unwrap();
            fs::set_permissions(&file, fs::Permissions::from_mode(0o644)).unwrap();
            script = script.replace(original, file.to_str().unwrap());
        }
        let path = fixture.path().join("installer.sh");
        let fixture_path = format!("{}:{}", stubs.display(), std::env::var("PATH").unwrap());
        let run = |body: &str| {
            fs::write(&path, body).unwrap();
            Command::new("bash")
                .args(["-c", "umask 077; exec bash \"$1\"", "fixture-installer"])
                .arg(&path)
                .env("PATH", &fixture_path)
                .output()
                .unwrap()
        };
        let bin = root.join("codex-runtime/bin");
        fs::create_dir(&bin).unwrap();
        fs::write(bin.join("codex"), b"old-main").unwrap();
        fs::write(bin.join("codex-code-mode-host"), b"old-host").unwrap();
        let failure = run(&script.replace(
            "# Physical directory publication",
            "exit 77\n# Physical directory publication",
        ));
        assert!(!failure.status.success());
        assert_eq!(fs::read(bin.join("codex")).unwrap(), b"old-main");
        assert_eq!(
            fs::read(bin.join("codex-code-mode-host")).unwrap(),
            b"old-host"
        );
        let failure = run(&script.replace("PYPAIR\nln -sf", "PYPAIR\nexit 78\nln -sf"));
        assert!(!failure.status.success());
        for (name, pin) in [
            ("codex", &spec.release_bundle.main),
            ("codex-code-mode-host", &spec.release_bundle.code_mode_host),
        ] {
            assert_eq!(
                format!("{:x}", Sha256::digest(fs::read(bin.join(name)).unwrap())),
                pin.extracted_executable_sha256
            );
        }
        let inodes = [
            fs::metadata(bin.join("codex")).unwrap().ino(),
            fs::metadata(bin.join("codex-code-mode-host"))
                .unwrap()
                .ino(),
        ];
        for _ in 0..2 {
            let result = run(&script);
            assert!(
                result.status.success(),
                "{}",
                String::from_utf8_lossy(&result.stderr)
            );
            let progress = String::from_utf8_lossy(&result.stderr);
            assert!(progress.contains(&format!(
                "substrate: staging Codex runtime {}",
                spec.version
            )));
            assert!(progress.contains(&format!(
                "substrate: installing Codex runtime {}",
                spec.version
            )));
            assert_eq!(
                inodes,
                [
                    fs::metadata(bin.join("codex")).unwrap().ino(),
                    fs::metadata(bin.join("codex-code-mode-host"))
                        .unwrap()
                        .ino()
                ]
            );
        }
        // Already-correct bytes under a non-searchable bin must not keep the
        // fast path broken. Correcting traversal modes preserves both files.
        fs::set_permissions(&bin, fs::Permissions::from_mode(0o700)).unwrap();
        let repaired = run(&script);
        assert!(
            repaired.status.success(),
            "{}",
            String::from_utf8_lossy(&repaired.stderr)
        );
        assert_eq!(
            inodes,
            [
                fs::metadata(bin.join("codex")).unwrap().ino(),
                fs::metadata(bin.join("codex-code-mode-host"))
                    .unwrap()
                    .ino()
            ]
        );
        // A distinct owner observation rejects before chmod or publication.
        let stat_stub = stubs.join("stat");
        fs::write(&stat_stub, format!("#!/bin/sh\nif [ \"$1\" = -c ] && [ \"$2\" = %u ] && [ \"$3\" = '{}' ]; then echo {}; exit 0; fi\nexec /usr/bin/stat \"$@\"\n", bin.display(), unsafe { libc::geteuid() } + 1)).unwrap();
        fs::set_permissions(&stat_stub, fs::Permissions::from_mode(0o755)).unwrap();
        assert!(!run(&script).status.success());
        fs::remove_file(stat_stub).unwrap();
        assert_eq!(
            inodes,
            [
                fs::metadata(bin.join("codex")).unwrap().ino(),
                fs::metadata(bin.join("codex-code-mode-host"))
                    .unwrap()
                    .ino()
            ]
        );
        for directory in [&root, &root.join("codex-runtime"), &bin] {
            assert_eq!(
                fs::metadata(directory).unwrap().permissions().mode() & 0o7777,
                0o755
            );
        }
        for name in ["codex", "codex-code-mode-host"] {
            assert_eq!(
                fs::metadata(bin.join(name)).unwrap().permissions().mode() & 0o7777,
                0o755
            );
        }
        assert!(
            !acquisition_log.exists(),
            "cache fixture attempted acquisition"
        );
        let head: serde_json::Value = serde_json::from_slice(
            &fs::read(root.join("runtime-artifacts-v1/heads/codex-official-release.json")).unwrap(),
        )
        .unwrap();
        assert_eq!(head["source_stream"], "CodexOfficialRelease");
        assert_eq!(head["head_revision"], 1);
        let reference = &head["head_ref"];
        let record: serde_json::Value = serde_json::from_slice(
            &fs::read(root.join(format!(
                "runtime-artifacts-v1/records/codex-official-release/{:020}-{}.json",
                reference["revision"].as_u64().unwrap(),
                reference["source_record_id"].as_str().unwrap()
            )))
            .unwrap(),
        )
        .unwrap();
        assert_eq!(record["entries"].as_array().unwrap().len(), 2);
        assert_eq!(
            record["build_input"]["CodexOfficialRelease"],
            serde_json::to_value(&spec.release_input).unwrap()
        );
        assert_eq!(
            fs::read_link(root.join("bin/codex")).unwrap(),
            bin.join("codex")
        );
        // Deliberate wrong-cache failure cannot acquire or replace either inode.
        fs::write(
            downloads.join(format!(
                "{}-{}",
                spec.version, spec.release_bundle.code_mode_host.archive_name
            )),
            b"wrong cache",
        )
        .unwrap();
        assert!(!run(&script).status.success());
        assert!(acquisition_log.exists());
        assert_eq!(
            inodes,
            [
                fs::metadata(bin.join("codex")).unwrap().ino(),
                fs::metadata(bin.join("codex-code-mode-host"))
                    .unwrap()
                    .ino()
            ]
        );
    }
    #[test]
    fn world_deps_codex_release_manifest_history_rollback_and_pending_isolation() {
        use sha2::{Digest, Sha256};
        use std::os::unix::fs::MetadataExt;
        let fixture = TempDir::new().unwrap();
        let filesystem = fixture.path().join("filesystem");
        let root = filesystem.join("var/lib/substrate/world-deps");
        let downloads = root.join("codex-runtime/downloads");
        fs::create_dir_all(&downloads).unwrap();
        for logical in [
            "/etc/hosts",
            "/etc/nsswitch.conf",
            "/etc/passwd",
            "/etc/group",
            "/etc/resolv.conf",
            "/etc/ssl/certs/ca-certificates.crt",
        ] {
            let p = filesystem.join(logical.trim_start_matches('/'));
            fs::create_dir_all(p.parent().unwrap()).unwrap();
            fs::write(&p, logical).unwrap();
            fs::set_permissions(p, fs::Permissions::from_mode(0o644)).unwrap();
        }
        let system = filesystem.join("etc/codex");
        fs::create_dir(&system).unwrap();
        fs::set_permissions(system, fs::Permissions::from_mode(0o755)).unwrap();
        let stubs = fixture.path().join("deny-acquisition");
        fs::create_dir(&stubs).unwrap();
        for name in ["wget", "curl"] {
            let p = stubs.join(name);
            fs::write(&p, "#!/bin/sh\nexit 99\n").unwrap();
            fs::set_permissions(p, fs::Permissions::from_mode(0o755)).unwrap();
        }
        let mut elf = vec![0u8; 120];
        elf[..16].copy_from_slice(b"\x7fELF\x02\x01\x01\0\0\0\0\0\0\0\0\0");
        elf[16..18].copy_from_slice(&2u16.to_le_bytes());
        elf[18..20].copy_from_slice(&62u16.to_le_bytes());
        elf[20..24].copy_from_slice(&1u32.to_le_bytes());
        elf[24..32].copy_from_slice(&0x400000u64.to_le_bytes());
        elf[32..40].copy_from_slice(&64u64.to_le_bytes());
        elf[52..54].copy_from_slice(&64u16.to_le_bytes());
        elf[54..56].copy_from_slice(&56u16.to_le_bytes());
        elf[56..58].copy_from_slice(&1u16.to_le_bytes());
        elf[64..68].copy_from_slice(&1u32.to_le_bytes());
        elf[68..72].copy_from_slice(&5u32.to_le_bytes());
        elf[80..88].copy_from_slice(&0x400000u64.to_le_bytes());
        elf[96..104].copy_from_slice(&120u64.to_le_bytes());
        elf[104..112].copy_from_slice(&120u64.to_le_bytes());
        elf[112..120].copy_from_slice(&0x1000u64.to_le_bytes());
        let base = resolve_codex_runtime_install_spec_for_target_v1(
            config_projection::CODEX_RELEASE_TARGET_V1,
        )
        .unwrap();
        let historical_release = base
            .release_catalog
            .releases
            .iter()
            .find(|r| r.version == "0.156.1")
            .expect("retained historical156 evidence")
            .clone();
        let mut catalog = base.release_catalog.clone();
        catalog.releases.clear();
        let mut variants = Vec::new();
        for (index, version) in ["0.200.0-fixture", "0.200.1-fixture"]
            .into_iter()
            .enumerate()
        {
            let mut record = base.release_input.manifest.clone();
            record.version = version.into();
            record.source.tag = format!("rust-v{version}");
            for (slot, artifact) in record.artifacts.iter_mut().enumerate() {
                let mut bytes = elf.clone();
                bytes.extend_from_slice(&[index as u8, slot as u8]);
                artifact.extracted.byte_length = bytes.len() as u64;
                artifact.extracted.sha256 = format!("{:x}", Sha256::digest(&bytes));
                artifact.extracted.elf = config_projection::E3ElfExecutionModelV1::StaticExec;
                artifact.archive.url = format!(
                    "https://github.com/openai/codex/releases/download/{}/{}",
                    record.source.tag, artifact.archive.name
                );
                let member = fixture.path().join(format!("member-{index}-{slot}"));
                fs::write(&member, &bytes).unwrap();
                let archive = downloads.join(format!("{version}-{}", artifact.archive.name));
                let output=Command::new("python3").args(["-c","import io,sys,tarfile; data=open(sys.argv[1],'rb').read(); t=tarfile.open(sys.argv[2],'w:gz'); i=tarfile.TarInfo(sys.argv[3]); i.size=len(data); i.mode=0o755; t.addfile(i,io.BytesIO(data)); t.close()"]).arg(member).arg(&archive).arg(&artifact.archive.member).output().unwrap();
                assert!(output.status.success());
                artifact.archive.byte_length = fs::metadata(&archive).unwrap().len();
                artifact.archive.sha256 =
                    format!("{:x}", Sha256::digest(fs::read(archive).unwrap()));
            }
            record.validate_structure().unwrap();
            catalog.releases.push(record.clone());
            variants.push(record);
        }
        let make_spec = |record: &config_projection::CodexReleaseRecordV1| {
            let mut spec = base.clone();
            spec.version = record.version.clone();
            spec.release_input = config_projection::CodexReleaseInputV1 {
                binding: record.binding().unwrap(),
                manifest: record.clone(),
            };
            spec.release_catalog = catalog.clone();
            spec.release_catalog
                .selected
                .insert(record.target.clone(), record.version.clone());
            let pin = |a: &config_projection::CodexReleaseArtifactV1| {
                config_projection::Codex0156OfficialArchiveV1 {
                    archive_name: a.archive.name.clone(),
                    archive_url: a.archive.url.clone(),
                    archive_sha256: a.archive.sha256.clone(),
                    archive_byte_length: a.archive.byte_length,
                    archive_entry_path: a.archive.member.clone(),
                    extracted_executable_sha256: a.extracted.sha256.clone(),
                    extracted_byte_length: a.extracted.byte_length,
                }
            };
            spec.release_bundle = CodexInstallerReleaseV1 {
                version: record.version.clone(),
                target_triple: record.target.clone(),
                main: pin(&record.artifacts[0]),
                code_mode_host: pin(&record.artifacts[1]),
            };
            spec.archive_name = spec.release_bundle.main.archive_name.clone();
            spec.archive_url = spec.release_bundle.main.archive_url.clone();
            spec.archive_sha256 = spec.release_bundle.main.archive_sha256.clone();
            spec.archive_entry_path = spec.release_bundle.main.archive_entry_path.clone();
            spec.extracted_executable_sha256 =
                spec.release_bundle.main.extracted_executable_sha256.clone();
            spec
        };
        let first = make_spec(&variants[0]);
        let second = make_spec(&variants[1]);
        assert_ne!(first.release_input.binding, second.release_input.binding);
        assert_ne!(
            first.release_bundle.main.extracted_executable_sha256,
            second.release_bundle.main.extracted_executable_sha256
        );
        let path = fixture.path().join("installer.sh");
        let run = |body: &str| {
            fs::write(&path, body).unwrap();
            Command::new("bash")
                .arg(&path)
                .env(
                    "PATH",
                    format!("{}:{}", stubs.display(), std::env::var("PATH").unwrap()),
                )
                .env("SUBSTRATE_ARTIFACT_FIXTURE_ROOT", &filesystem)
                .output()
                .unwrap()
        };
        let a = codex_component_installer_script_v1(&first, &filesystem);
        let b = codex_component_installer_script_v1(&second, &filesystem);
        let result = run(&a);
        assert!(
            result.status.success(),
            "{}",
            String::from_utf8_lossy(&result.stderr)
        );
        let source = root.join("runtime-artifacts-v1");
        let head_path = source.join("heads/codex-official-release.json");
        let old_head = fs::read(&head_path).unwrap();
        let bin = root.join("codex-runtime/bin");
        let old_inode = fs::metadata(bin.join("codex")).unwrap().ino();
        let before = run(&b.replace(
            "# Physical directory publication",
            "exit 77\n# Physical directory publication",
        ));
        assert!(!before.status.success());
        assert_eq!(old_head, fs::read(&head_path).unwrap());
        assert_eq!(old_inode, fs::metadata(bin.join("codex")).unwrap().ino());
        let after = run(&b.replace("PYPAIR\nln -sf", "PYPAIR\nexit 78\nln -sf"));
        assert!(!after.status.success());
        assert_eq!(old_head, fs::read(&head_path).unwrap());
        assert_ne!(old_inode, fs::metadata(bin.join("codex")).unwrap().ino());
        let mut stripped = second.clone();
        stripped
            .release_catalog
            .releases
            .retain(|r| r.version == second.version);
        let missing_history = run(&codex_component_installer_script_v1(&stripped, &filesystem));
        assert!(
            !missing_history.status.success(),
            "removing deselected historical manifest must reject"
        );
        assert_eq!(old_head, fs::read(&head_path).unwrap());
        let result = run(&b);
        assert!(
            result.status.success(),
            "own-manifest A->B history failed: {}",
            String::from_utf8_lossy(&result.stderr)
        );
        let head_b = fs::read(&head_path).unwrap();
        let value: serde_json::Value = serde_json::from_slice(&head_b).unwrap();
        assert_eq!(value["head_revision"], 2);
        let inode_b = fs::metadata(bin.join("codex")).unwrap().ino();
        let retry = run(&b);
        assert!(retry.status.success());
        assert_eq!(head_b, fs::read(&head_path).unwrap());
        assert_eq!(inode_b, fs::metadata(bin.join("codex")).unwrap().ino());
        let result = run(&a);
        assert!(
            result.status.success(),
            "forward rollback failed: {}",
            String::from_utf8_lossy(&result.stderr)
        );
        let head_a = fs::read(&head_path).unwrap();
        let value: serde_json::Value = serde_json::from_slice(&head_a).unwrap();
        assert_eq!(value["head_revision"], 3);
        assert_ne!(old_head, head_a);
        assert_ne!(inode_b, fs::metadata(bin.join("codex")).unwrap().ino());
        assert_eq!(
            fs::read_dir(source.join("records/codex-official-release"))
                .unwrap()
                .count(),
            3
        );
        // Unknown/forged foreign heads cannot be reinterpreted as the current stream.
        let temp = source
            .join("heads")
            .join(format!(".e3-artifact-tmp.{}.head", uuid::Uuid::now_v7()));
        let mut foreign = value.clone();
        foreign["source_stream"] = serde_json::json!("UnknownStream");
        foreign.as_object_mut().unwrap().remove("head_hash");
        foreign["head_hash"]=serde_json::json!(config_projection::ConfigProjectionCodecV1::domain_sha256("",&serde_json::json!({"domain":"substrate.e3.installer-artifact-source-head.v1","head":foreign})).unwrap());
        fs::write(
            &temp,
            config_projection::ConfigProjectionCodecV1::encode_canonical_json(&foreign).unwrap(),
        )
        .unwrap();
        fs::set_permissions(&temp, fs::Permissions::from_mode(0o640)).unwrap();
        assert!(!run(&a).status.success());
        assert!(temp.exists());
        assert_eq!(head_a, fs::read(&head_path).unwrap());
        fs::remove_file(&temp).unwrap();
        // Current pending-head recovery retains exact head/inodes on idempotent replay.
        fs::write(&temp, &head_a).unwrap();
        fs::set_permissions(&temp, fs::Permissions::from_mode(0o640)).unwrap();
        let result = run(&a);
        assert!(
            result.status.success(),
            "{}",
            String::from_utf8_lossy(&result.stderr)
        );
        assert!(!temp.exists());
        assert_eq!(head_a, fs::read(&head_path).unwrap());
        // A source manifest binding cannot authorize the other generated pair.
        let mut mixed = first.clone();
        mixed.release_bundle.code_mode_host = second.release_bundle.code_mode_host.clone();
        assert!(validate_codex_runtime_install_spec_v1(&mixed).is_err());
        assert_eq!(head_a, fs::read(&head_path).unwrap());
        // A frozen historical156 pending head is authenticated in its OWN stream.
        // This logical historical fixture deliberately does not claim current files:
        // upgrades have already replaced their original descriptor observations.
        let rehash = |mut value: serde_json::Value, domain: &str, member: &str, field: &str| {
            value.as_object_mut().unwrap().remove(field);
            let hash = config_projection::ConfigProjectionCodecV1::domain_sha256(
                "",
                &serde_json::json!({"domain":domain,member:value}),
            )
            .unwrap();
            value[field] = serde_json::json!(hash);
            value
        };
        let reference = &value["head_ref"];
        let current_record = source.join(format!(
            "records/codex-official-release/{:020}-{}.json",
            reference["revision"].as_u64().unwrap(),
            reference["source_record_id"].as_str().unwrap()
        ));
        let mut legacy: serde_json::Value =
            serde_json::from_slice(&fs::read(current_record).unwrap()).unwrap();
        let pins = config_projection::Codex0156ReleaseBundleV1::pinned();
        legacy["source_stream"] = serde_json::json!("Codex0156OfficialArchives");
        legacy["source_record_id"] = serde_json::json!(format!("iar_{}", uuid::Uuid::now_v7()));
        legacy["revision"] = serde_json::json!(1);
        legacy["predecessor_ref"] = serde_json::Value::Null;
        legacy["build_input"] = serde_json::json!({"Codex0156OfficialArchives":pins});
        for (index, pin) in [&pins.main, &pins.code_mode_host].into_iter().enumerate() {
            legacy["entries"][index]["sha256"] = serde_json::json!(pin.extracted_executable_sha256);
            legacy["entries"][index]["byte_length"] = serde_json::json!(pin.extracted_byte_length);
            legacy["entries"][index]["runtime_support"]["elf_execution_model"] =
                serde_json::to_value(&historical_release.artifacts[index].extracted.elf).unwrap();
        }
        let host_file = serde_json::json!({"absolute_path":config_projection::CODEX_HOST_PATH_V1,"device_id":legacy["entries"][1]["device_id"],"inode":legacy["entries"][1]["inode"],"mode":0o755,"byte_length":pins.code_mode_host.extracted_byte_length,"sha256":pins.code_mode_host.extracted_executable_sha256});
        legacy["entries"][0]["runtime_support"]["support_policy_version"] = serde_json::json!(2);
        legacy["entries"][0]["runtime_support"]["code_mode_host"] = serde_json::json!({"file":host_file,"owner_uid":0,"elf_execution_model":historical_release.artifacts[1].extracted.elf,"provenance":pins.code_mode_host.provenance()});
        for index in [0, 1] {
            legacy["entries"][index]["runtime_support"] = rehash(
                legacy["entries"][index]["runtime_support"].clone(),
                "substrate.e3.runtime-support-manifest.v1",
                "manifest",
                "manifest_hash",
            );
            legacy["entries"][index] = rehash(
                legacy["entries"][index].clone(),
                "substrate.e3.installer-artifact-source-entry.v1",
                "entry",
                "entry_hash",
            );
        }
        legacy = rehash(
            legacy,
            "substrate.e3.installer-artifact-source-record.v1",
            "record",
            "record_hash",
        );
        let legacy_dir = source.join("records/codex-0.156.1-x86_64-unknown-linux-musl");
        fs::create_dir(&legacy_dir).unwrap();
        fs::set_permissions(&legacy_dir, fs::Permissions::from_mode(0o750)).unwrap();
        let legacy_record_path = legacy_dir.join(format!(
            "{:020}-{}.json",
            1,
            legacy["source_record_id"].as_str().unwrap()
        ));
        let legacy_record_bytes =
            config_projection::ConfigProjectionCodecV1::encode_canonical_json(&legacy).unwrap();
        fs::write(&legacy_record_path, &legacy_record_bytes).unwrap();
        fs::set_permissions(&legacy_record_path, fs::Permissions::from_mode(0o640)).unwrap();
        let legacy_head = rehash(
            serde_json::json!({"schema_version":1,"source_store_id":legacy["source_store_id"],"source_stream":"Codex0156OfficialArchives","head_ref":{"source_store_id":legacy["source_store_id"],"source_record_id":legacy["source_record_id"],"revision":1,"record_hash":legacy["record_hash"]},"head_revision":1,"predecessor_head_hash":null,"updated_at":legacy["created_at"]}),
            "substrate.e3.installer-artifact-source-head.v1",
            "head",
            "head_hash",
        );
        let legacy_head_bytes =
            config_projection::ConfigProjectionCodecV1::encode_canonical_json(&legacy_head)
                .unwrap();
        fs::write(&temp, &legacy_head_bytes).unwrap();
        fs::set_permissions(&temp, fs::Permissions::from_mode(0o640)).unwrap();
        let result = run(&a);
        assert!(
            result.status.success(),
            "valid frozen foreign head rejected: {}",
            String::from_utf8_lossy(&result.stderr)
        );
        assert_eq!(fs::read(&temp).unwrap(), legacy_head_bytes);
        assert!(!source
            .join("heads/codex-0.156.1-x86_64-unknown-linux-musl.json")
            .exists());
        assert_eq!(head_a, fs::read(&head_path).unwrap());
        // OWNER-PRE2: both staged files are valid before either immutable/head commit.
        let legacy_record_temporary =
            legacy_dir.join(format!(".e3-artifact-tmp.{}.record", uuid::Uuid::now_v7()));
        fs::rename(&legacy_record_path, &legacy_record_temporary).unwrap();
        let result = run(&a);
        assert!(
            result.status.success(),
            "OWNER-PRE2 complete foreign pending record/head pair rejected: {}",
            String::from_utf8_lossy(&result.stderr)
        );
        assert_eq!(
            fs::read(&legacy_record_temporary).unwrap(),
            legacy_record_bytes
        );
        assert_eq!(fs::read(&temp).unwrap(), legacy_head_bytes);
        assert!(!legacy_record_path.exists());
        assert!(!source
            .join("heads/codex-0.156.1-x86_64-unknown-linux-musl.json")
            .exists());
        assert_eq!(head_a, fs::read(&head_path).unwrap());
        // Incomplete, ambiguous, wrong-mode and symlinked staged records are not
        // authenticated merely because their shared pending head names a known stream.
        let held_pending = fixture.path().join("held-foreign-pending-record");
        fs::rename(&legacy_record_temporary, &held_pending).unwrap();
        assert!(
            !run(&a).status.success(),
            "foreign head without final or staged record accepted"
        );
        fs::rename(&held_pending, &legacy_record_temporary).unwrap();
        let duplicate =
            legacy_dir.join(format!(".e3-artifact-tmp.{}.record", uuid::Uuid::now_v7()));
        fs::write(&duplicate, &legacy_record_bytes).unwrap();
        fs::set_permissions(&duplicate, fs::Permissions::from_mode(0o640)).unwrap();
        assert!(
            !run(&a).status.success(),
            "ambiguous foreign record temporaries accepted"
        );
        fs::remove_file(&duplicate).unwrap();
        fs::set_permissions(&legacy_record_temporary, fs::Permissions::from_mode(0o600)).unwrap();
        assert!(
            !run(&a).status.success(),
            "wrong-mode foreign record temporary accepted"
        );
        fs::set_permissions(&legacy_record_temporary, fs::Permissions::from_mode(0o640)).unwrap();
        fs::rename(&legacy_record_temporary, &held_pending).unwrap();
        std::os::unix::fs::symlink(&held_pending, &legacy_record_temporary).unwrap();
        assert!(
            !run(&a).status.success(),
            "symlinked foreign record temporary accepted"
        );
        fs::remove_file(&legacy_record_temporary).unwrap();
        fs::rename(&held_pending, &legacy_record_temporary).unwrap();
        assert_eq!(head_a, fs::read(&head_path).unwrap());
        assert_eq!(legacy_head_bytes, fs::read(&temp).unwrap());
        fs::rename(&legacy_record_temporary, &legacy_record_path).unwrap();
        for mutation in 0..7 {
            let mut bad = legacy_head.clone();
            match mutation {
                0 => bad["head_ref"]["source_record_id"] = serde_json::json!("../../unsafe"),
                1 => {
                    bad["head_ref"]["source_store_id"] =
                        serde_json::json!(format!("ias_{}", uuid::Uuid::now_v7()))
                }
                2 => bad["head_ref"]["revision"] = serde_json::json!(2),
                3 => bad["head_ref"]["record_hash"] = serde_json::json!("00".repeat(32)),
                4 => bad["unexpected"] = serde_json::json!(true),
                5 => bad["updated_at"] = serde_json::json!("invalid"),
                _ => bad["source_stream"] = serde_json::json!("Codex0125OfficialArchive"),
            };
            bad = rehash(
                bad,
                "substrate.e3.installer-artifact-source-head.v1",
                "head",
                "head_hash",
            );
            fs::write(
                &temp,
                config_projection::ConfigProjectionCodecV1::encode_canonical_json(&bad).unwrap(),
            )
            .unwrap();
            assert!(
                !run(&a).status.success(),
                "foreign mutation {mutation} accepted"
            );
            assert_eq!(head_a, fs::read(&head_path).unwrap());
            assert!(temp.exists());
        }
        fs::write(&temp, &legacy_head_bytes).unwrap();
        let mut bad_record = legacy.clone();
        bad_record["build_input"]["Codex0156OfficialArchives"]["main"]["archive_sha256"] =
            serde_json::json!("00".repeat(32));
        bad_record = rehash(
            bad_record,
            "substrate.e3.installer-artifact-source-record.v1",
            "record",
            "record_hash",
        );
        let mut bad_head = legacy_head.clone();
        bad_head["head_ref"]["record_hash"] = bad_record["record_hash"].clone();
        bad_head = rehash(
            bad_head,
            "substrate.e3.installer-artifact-source-head.v1",
            "head",
            "head_hash",
        );
        fs::write(
            &legacy_record_path,
            config_projection::ConfigProjectionCodecV1::encode_canonical_json(&bad_record).unwrap(),
        )
        .unwrap();
        fs::write(
            &temp,
            config_projection::ConfigProjectionCodecV1::encode_canonical_json(&bad_head).unwrap(),
        )
        .unwrap();
        assert!(
            !run(&a).status.success(),
            "self-hashed altered historical pins accepted"
        );
        fs::write(&legacy_record_path, &legacy_record_bytes).unwrap();
        fs::write(&temp, &legacy_head_bytes).unwrap();
        // Recover a B transaction staged under B, while the next invocation selects A.
        let interrupted=run(&b.replace("commit_staged_publication(record_temporary, record_path, record_body, True)","raise RuntimeError('fixture interruption after both immutable/head staging')\ncommit_staged_publication(record_temporary, record_path, record_body, True)"));
        assert!(!interrupted.status.success());
        assert_eq!(head_a, fs::read(&head_path).unwrap());
        let result = run(&a);
        assert!(
            result.status.success(),
            "own-manifest pending B recovery while selecting A failed: {}",
            String::from_utf8_lossy(&result.stderr)
        );
        let recovered: serde_json::Value =
            serde_json::from_slice(&fs::read(&head_path).unwrap()).unwrap();
        assert_eq!(recovered["head_revision"], 5);
        assert_eq!(
            fs::read_dir(source.join("records/codex-official-release"))
                .unwrap()
                .count(),
            5
        );
        assert_eq!(fs::read(&temp).unwrap(), legacy_head_bytes);
        let before_retry = fs::read(&head_path).unwrap();
        let inode = fs::metadata(bin.join("codex")).unwrap().ino();
        assert!(run(&a).status.success());
        assert_eq!(before_retry, fs::read(&head_path).unwrap());
        assert_eq!(inode, fs::metadata(bin.join("codex")).unwrap().ino());
    }
    fn codex_component_installer_script_v1(
        spec: &CodexRuntimeInstallSpecV1,
        filesystem: &Path,
    ) -> String {
        let root = filesystem.join("var/lib/substrate/world-deps");
        let prelude = r#"import builtins,os,stat
fixture_root=os.environ["SUBSTRATE_ARTIFACT_FIXTURE_ROOT"]
def observed_path(path):
    path=os.fspath(path)
    if path.startswith(("/etc/","/var/lib/substrate/","/usr/local/lib/substrate/")): return os.path.join(fixture_root,path.lstrip("/"))
    return path
original_stat,original_open,original_listdir,original_listxattr=os.stat,builtins.open,os.listdir,os.listxattr
def observed_stat(path,*args,**kwargs):
    value=list(original_stat(observed_path(path),*args,**kwargs));value[4]=0;value[5]=0;return os.stat_result(value)
os.stat=observed_stat
builtins.open=lambda path,*args,**kwargs:original_open(observed_path(path),*args,**kwargs)
os.listdir=lambda path:original_listdir(observed_path(path))
os.listxattr=lambda path,*args,**kwargs:original_listxattr(observed_path(path),*args,**kwargs)
"#;
        render_codex_runtime_install_script_v1(
            super::super::inventory::codex_runtime_install_script_template_v1(),
            spec,
        )
        .replace(
            "world_deps_root=\"/var/lib/substrate/world-deps\"",
            &format!("world_deps_root=\"{}\"", root.display()),
        )
        .replace(
            "python3 - \"${artifact_source_root}\" \"${installed_binary}\"",
            &format!(
                "python3 - \"${{artifact_source_root}}\" \"{}\"",
                config_projection::CODEX_MAIN_PATH_V1
            ),
        )
        .replace(
            "<<'PY'\nimport datetime",
            &format!("<<'PY'\n{prelude}\nimport datetime"),
        )
        .replace("grp.getgrnam(\"substrate\").gr_gid", "os.getgid()")
    }
    #[cfg(unix)]
    #[test]
    #[ignore = "non-executing connected installer proof; requires held archives and a new SUBSTRATE_CODEX_COMPONENT_DIR"]
    fn world_deps_codex_release_installer_handoff() {
        use sha2::{Digest, Sha256};
        use std::os::unix::fs::MetadataExt;
        let directory = PathBuf::from(
            std::env::var("SUBSTRATE_CODEX_COMPONENT_DIR").expect("component directory required"),
        );
        let archives = PathBuf::from(
            std::env::var("SUBSTRATE_CODEX_0156_ARCHIVE_DIR")
                .expect("held official archives required"),
        );
        let filesystem = directory.join("filesystem");
        assert!(
            !filesystem.exists(),
            "use a fresh test-owned component directory"
        );
        let root = filesystem.join("var/lib/substrate/world-deps");
        let downloads = root.join("codex-runtime/downloads");
        fs::create_dir_all(&downloads).unwrap();
        let spec = resolve_codex_runtime_install_spec_for_target_v1(
            config_projection::CODEX_RELEASE_TARGET_V1,
        )
        .unwrap();
        for pin in [
            &spec.release_bundle.main,
            &spec.release_bundle.code_mode_host,
        ] {
            let source = archives.join(&pin.archive_name);
            assert_eq!(
                fs::metadata(&source).unwrap().len(),
                pin.archive_byte_length
            );
            assert_eq!(
                format!("{:x}", Sha256::digest(fs::read(&source).unwrap())),
                pin.archive_sha256
            );
            fs::copy(
                source,
                downloads.join(format!("{}-{}", spec.version, pin.archive_name)),
            )
            .unwrap();
        }
        let system = filesystem.join("etc/codex");
        fs::create_dir_all(&system).unwrap();
        fs::set_permissions(&system, fs::Permissions::from_mode(0o755)).unwrap();
        for logical in [
            "/etc/hosts",
            "/etc/nsswitch.conf",
            "/etc/passwd",
            "/etc/group",
            "/etc/resolv.conf",
            "/etc/ssl/certs/ca-certificates.crt",
        ] {
            let path = filesystem.join(logical.trim_start_matches('/'));
            fs::create_dir_all(path.parent().unwrap()).unwrap();
            fs::write(&path, format!("test-owned support {logical}\n")).unwrap();
            fs::set_permissions(path, fs::Permissions::from_mode(0o644)).unwrap();
        }
        let stubs = directory.join("deny-acquisition");
        fs::create_dir_all(&stubs).unwrap();
        let denied = directory.join("acquisition-attempts");
        for name in ["wget", "curl"] {
            let path = stubs.join(name);
            fs::write(
                &path,
                format!(
                    "#!/bin/sh\nprintf '%s\\n' denied >> '{}'\nexit 99\n",
                    denied.display()
                ),
            )
            .unwrap();
            fs::set_permissions(path, fs::Permissions::from_mode(0o755)).unwrap();
        }
        // Test-owned filesystem/UID observations only. Logical paths, record bytes,
        // archive identities and the actual publisher/importer code remain exact.
        let script = codex_component_installer_script_v1(&spec, &filesystem);
        assert!(!script.contains("__SUBSTRATE_CODEX_"));
        let path = directory.join("installer.sh");
        fs::write(&path, &script).unwrap();
        let run = || {
            Command::new("bash")
                .arg(&path)
                .env(
                    "PATH",
                    format!("{}:{}", stubs.display(), std::env::var("PATH").unwrap()),
                )
                .env("SUBSTRATE_ARTIFACT_FIXTURE_ROOT", &filesystem)
                .output()
                .unwrap()
        };
        let first = run();
        assert!(
            first.status.success(),
            "{}",
            String::from_utf8_lossy(&first.stderr)
        );
        assert!(!denied.exists(), "unexpected acquisition attempt");
        let bin = root.join("codex-runtime/bin");
        let inodes = [
            fs::metadata(bin.join("codex")).unwrap().ino(),
            fs::metadata(bin.join("codex-code-mode-host"))
                .unwrap()
                .ino(),
        ];
        let retry = run();
        assert!(
            retry.status.success(),
            "{}",
            String::from_utf8_lossy(&retry.stderr)
        );
        assert_eq!(
            inodes,
            [
                fs::metadata(bin.join("codex")).unwrap().ino(),
                fs::metadata(bin.join("codex-code-mode-host"))
                    .unwrap()
                    .ino()
            ]
        );
        let source = root.join("runtime-artifacts-v1");
        let head: serde_json::Value =
            config_projection::ConfigProjectionCodecV1::decode_canonical_json(
                &fs::read(source.join("heads/codex-official-release.json")).unwrap(),
            )
            .unwrap();
        assert_eq!(head["head_revision"], 1);
        assert_eq!(head["source_stream"], "CodexOfficialRelease");
        let reference = &head["head_ref"];
        let record_path = source.join(format!(
            "records/codex-official-release/{:020}-{}.json",
            reference["revision"].as_u64().unwrap(),
            reference["source_record_id"].as_str().unwrap()
        ));
        let record: config_projection::InstallerArtifactSourceRecordV1 =
            config_projection::ConfigProjectionCodecV1::decode_canonical_json(
                &fs::read(&record_path).unwrap(),
            )
            .unwrap();
        assert_eq!(
            record.build_input,
            config_projection::InstallerArtifactBuildInputV1::CodexOfficialRelease(
                spec.release_input.clone()
            )
        );
        assert_eq!(record.entries[0].inode, inodes[0]);
        assert_eq!(record.entries[1].inode, inodes[1]);
        assert_eq!(record.entries[0].runtime_support.support_policy_version, 3);
        fs::write(
            directory.join("release-input.json"),
            config_projection::ConfigProjectionCodecV1::encode_canonical_json(&spec.release_input)
                .unwrap(),
        )
        .unwrap();
        fs::write(
            directory.join("installer-record.json"),
            fs::read(record_path).unwrap(),
        )
        .unwrap();
    }
}

struct WorldCommandOutputV1 {
    exit: i32,
    stdout: String,
    stderr: String,
}

const CODEX_RUNTIME_PACKAGE_NAME: &str = "codex-runtime";

#[derive(Debug, Clone, PartialEq, Eq)]
struct CodexRuntimeInstallSpecV1 {
    target_triple: String,
    version: String,
    archive_name: String,
    archive_url: String,
    archive_sha256: String,
    archive_entry_path: String,
    extracted_executable_sha256: String,
    release_bundle: CodexInstallerReleaseV1,
    release_input: config_projection::CodexReleaseInputV1,
    release_catalog: config_projection::CodexReleaseCatalogV1,
}

// Structural installer arguments derive from the authenticated release; no independent pins.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
struct CodexInstallerReleaseV1 {
    version: String,
    target_triple: String,
    main: config_projection::Codex0156OfficialArchiveV1,
    code_mode_host: config_projection::Codex0156OfficialArchiveV1,
}
#[allow(dead_code)]
fn run_world_command_output_for_deps(
    cmd: &str,
    cwd_override: Option<&str>,
    #[cfg(target_os = "linux")] context: &AuthenticatedWorldDepsContextV1,
) -> Result<WorldCommandOutputV1> {
    run_world_command_output_for_deps_with_profile(
        cmd,
        cwd_override,
        None,
        #[cfg(target_os = "linux")]
        context,
    )
}

fn run_world_command_output_for_deps_with_profile(
    cmd: &str,
    cwd_override: Option<&str>,
    profile_override: Option<&str>,
    #[cfg(target_os = "linux")] context: &AuthenticatedWorldDepsContextV1,
) -> Result<WorldCommandOutputV1> {
    let response = run_world_command_for_deps_at(
        cmd,
        cwd_override,
        profile_override,
        #[cfg(target_os = "linux")]
        context,
    )
    .map_err(classify_world_backend_error)?;
    let stdout = BASE64
        .decode(response.stdout_b64.as_bytes())
        .unwrap_or_default();
    let stderr = BASE64
        .decode(response.stderr_b64.as_bytes())
        .unwrap_or_default();
    Ok(WorldCommandOutputV1 {
        exit: response.exit,
        stdout: String::from_utf8_lossy(&stdout).to_string(),
        stderr: String::from_utf8_lossy(&stderr).to_string(),
    })
}

fn output_snippet_for_error(out: &WorldCommandOutputV1) -> String {
    let stderr = out.stderr.trim();
    if !stderr.is_empty() {
        return stderr.to_string();
    }
    out.stdout.trim().to_string()
}

fn apply_script_package_v1(
    pkg: &super::inventory::PackageDefV1,
    #[cfg(target_os = "linux")] context: &AuthenticatedWorldDepsContextV1,
) -> Result<()> {
    let script = resolve_script_body_for_package_v1(
        pkg,
        #[cfg(target_os = "linux")]
        context,
    )?;
    let wrappers = pkg
        .wrappers
        .iter()
        .map(render_wrapper_file_v1)
        .collect::<Result<Vec<_>>>()?;

    let cmd = build_world_script_install_command_v1(&script, &wrappers);
    run_world_command_checked_for_deps(
        &cmd,
        Some("/tmp"),
        #[cfg(target_os = "linux")]
        context,
    )
    .with_context(|| format!("world install script failed for '{}'", pkg.name))?;
    Ok(())
}

fn resolve_script_body_for_package_v1(
    pkg: &super::inventory::PackageDefV1,
    #[cfg(target_os = "linux")] context: &AuthenticatedWorldDepsContextV1,
) -> Result<String> {
    if pkg.name == CODEX_RUNTIME_PACKAGE_NAME {
        let target_triple = current_codex_runtime_target_triple_v1(
            #[cfg(target_os = "linux")]
            context,
        )?;
        let spec = resolve_codex_runtime_install_spec_for_target_v1(target_triple)?;
        let template = pkg.install.script.as_deref().ok_or_else(|| {
            config_model::user_error(format!(
                "invalid deps inventory: package '{}' must declare an install.script template",
                pkg.name
            ))
        })?;
        return Ok(render_codex_runtime_install_script_v1(template, &spec));
    }

    if let Some(path_raw) = &pkg.install.script_path {
        let path = PathBuf::from(path_raw);
        let resolved = if path.is_absolute() {
            path
        } else if let Some(def_path) = &pkg.definition_path {
            def_path
                .parent()
                .ok_or_else(|| anyhow!("invalid package definition path for '{}'", pkg.name))?
                .join(path)
        } else {
            return Err(config_model::user_error(format!(
                "invalid deps inventory: package '{}' declares a relative install.script_path but has no source path (built-ins should use install.script)",
                pkg.name
            )));
        };

        let raw = fs::read_to_string(&resolved).with_context(|| {
            format!(
                "failed to read install.script_path for '{}': {}",
                pkg.name,
                resolved.display()
            )
        })?;
        return Ok(raw);
    }

    if let Some(script) = &pkg.install.script {
        return Ok(script.clone());
    }

    Err(config_model::user_error(format!(
        "invalid deps inventory: package '{}' declares install.method=script but has no script content",
        pkg.name
    )))
}

fn current_codex_runtime_target_triple_v1(
    #[cfg(target_os = "linux")] context: &AuthenticatedWorldDepsContextV1,
) -> Result<&'static str> {
    let guest_arch = run_world_command_output_for_deps(
        "uname -m",
        Some("/tmp"),
        #[cfg(target_os = "linux")]
        context,
    )
    .context("failed to inspect guest architecture for codex runtime selection")?;
    if guest_arch.exit != 0 {
        return Err(anyhow!(
            "failed to inspect guest architecture for codex runtime selection: {}",
            output_snippet_for_error(&guest_arch)
        ));
    }
    let guest_arch = guest_arch.stdout.trim();
    map_codex_runtime_target_triple_for_guest_arch_v1(guest_arch)
}

fn map_codex_runtime_target_triple_for_guest_arch_v1(guest_arch: &str) -> Result<&'static str> {
    match guest_arch {
        "x86_64" | "amd64" => Ok("x86_64-unknown-linux-musl"),
        "aarch64" | "arm64" => Ok("aarch64-unknown-linux-musl"),
        arch => Err(config_model::user_error(format!(
            "world deps package '{}' is unsupported for guest arch '{}' because Substrate cannot derive a Linux Codex guest target triple from the actual guest posture",
            CODEX_RUNTIME_PACKAGE_NAME, arch
        ))),
    }
}

fn resolve_codex_runtime_install_spec_for_target_v1(
    target_triple: &str,
) -> Result<CodexRuntimeInstallSpecV1> {
    let release =
        config_projection::TrustedCodexReleaseV1::selected(target_triple).map_err(|error| {
            config_model::user_error(format!(
                "world deps package '{CODEX_RUNTIME_PACKAGE_NAME}' has no pinned official Codex release mapping for guest target '{target_triple}': {error:?}"
            ))
        })?;
    let archive = |artifact: &config_projection::CodexReleaseArtifactV1| {
        config_projection::Codex0156OfficialArchiveV1 {
            archive_name: artifact.archive.name.clone(),
            archive_url: artifact.archive.url.clone(),
            archive_sha256: artifact.archive.sha256.clone(),
            archive_byte_length: artifact.archive.byte_length,
            archive_entry_path: artifact.archive.member.clone(),
            extracted_executable_sha256: artifact.extracted.sha256.clone(),
            extracted_byte_length: artifact.extracted.byte_length,
        }
    };
    let bundle = CodexInstallerReleaseV1 {
        version: release.record().version.clone(),
        target_triple: release.record().target.clone(),
        main: archive(release.main()),
        code_mode_host: archive(release.host()),
    };
    Ok(CodexRuntimeInstallSpecV1 {
        target_triple: bundle.target_triple.clone(),
        version: bundle.version.clone(),
        archive_name: bundle.main.archive_name.clone(),
        archive_url: bundle.main.archive_url.clone(),
        archive_sha256: bundle.main.archive_sha256.clone(),
        archive_entry_path: bundle.main.archive_entry_path.clone(),
        extracted_executable_sha256: bundle.main.extracted_executable_sha256.clone(),
        release_bundle: bundle,
        release_input: release.input().clone(),
        release_catalog: config_projection::CodexReleaseCatalogV1::embedded()
            .map_err(|e| anyhow::anyhow!("trusted catalog: {e:?}"))?,
    })
}

fn validate_codex_runtime_install_spec_v1(spec: &CodexRuntimeInstallSpecV1) -> Result<()> {
    let manifest = &spec.release_input.manifest;
    spec.release_catalog
        .validate()
        .map_err(|e| anyhow::anyhow!("invalid catalog: {e:?}"))?;
    if manifest
        .binding()
        .map_err(|e| anyhow::anyhow!("invalid release: {e:?}"))?
        != spec.release_input.binding
        || !spec.release_catalog.releases.contains(manifest)
        || spec.release_catalog.selected.get(&manifest.target) != Some(&manifest.version)
        || (&spec.version, &spec.target_triple) != (&manifest.version, &manifest.target)
    {
        anyhow::bail!("installer selection differs from authenticated manifest");
    }
    let archive = |a: &config_projection::CodexReleaseArtifactV1| {
        config_projection::Codex0156OfficialArchiveV1 {
            archive_name: a.archive.name.clone(),
            archive_url: a.archive.url.clone(),
            archive_sha256: a.archive.sha256.clone(),
            archive_byte_length: a.archive.byte_length,
            archive_entry_path: a.archive.member.clone(),
            extracted_executable_sha256: a.extracted.sha256.clone(),
            extracted_byte_length: a.extracted.byte_length,
        }
    };
    if spec.release_bundle
        != (CodexInstallerReleaseV1 {
            version: manifest.version.clone(),
            target_triple: manifest.target.clone(),
            main: archive(&manifest.artifacts[0]),
            code_mode_host: archive(&manifest.artifacts[1]),
        })
        || (
            &spec.archive_name,
            &spec.archive_url,
            &spec.archive_sha256,
            &spec.archive_entry_path,
            &spec.extracted_executable_sha256,
        ) != (
            &spec.release_bundle.main.archive_name,
            &spec.release_bundle.main.archive_url,
            &spec.release_bundle.main.archive_sha256,
            &spec.release_bundle.main.archive_entry_path,
            &spec.release_bundle.main.extracted_executable_sha256,
        )
    {
        anyhow::bail!("installer archive view differs from authenticated manifest");
    }
    Ok(())
}

fn render_codex_runtime_install_script_v1(
    template: &str,
    spec: &CodexRuntimeInstallSpecV1,
) -> String {
    validate_codex_runtime_install_spec_v1(spec).expect("trusted installer release data");
    let publisher =
        crate::builtins::world_deps::inventory::render_codex_release_installer_artifact_source_v1();
    let (source_lock, publication) = publisher
        .split_once("python3 - ")
        .expect("fixed source publisher");
    template
        .replace("__SUBSTRATE_CODEX_SOURCE_LOCK_V1__", source_lock)
        .replace(
            "__SUBSTRATE_CODEX_RELEASE_INPUT_V1__",
            &sh_quote(
                &String::from_utf8(
                    config_projection::ConfigProjectionCodecV1::encode_canonical_json(
                        &spec.release_input,
                    )
                    .expect("trusted release"),
                )
                .expect("UTF8"),
            ),
        )
        .replace(
            "__SUBSTRATE_CODEX_RELEASE_CATALOG_V1__",
            &sh_quote(
                &String::from_utf8(
                    config_projection::ConfigProjectionCodecV1::encode_canonical_json(
                        &spec.release_catalog,
                    )
                    .expect("trusted catalog"),
                )
                .expect("UTF8"),
            ),
        )
        .replace(
            "__SUBSTRATE_CODEX_RELEASE_BUNDLE_V1__",
            &sh_quote(&serde_json::to_string(&spec.release_bundle).expect("release data")),
        )
        .replace(
            "__SUBSTRATE_CODEX_HOST_ARCHIVE_NAME__",
            &spec.release_bundle.code_mode_host.archive_name,
        )
        .replace(
            "__SUBSTRATE_CODEX_HOST_ARCHIVE_URL__",
            &spec.release_bundle.code_mode_host.archive_url,
        )
        .replace(
            "__SUBSTRATE_CODEX_HOST_ARCHIVE_SHA256__",
            &spec.release_bundle.code_mode_host.archive_sha256,
        )
        .replace(
            "__SUBSTRATE_CODEX_HOST_ARCHIVE_ENTRY_PATH__",
            &spec.release_bundle.code_mode_host.archive_entry_path,
        )
        .replace(
            "__SUBSTRATE_CODEX_HOST_EXECUTABLE_SHA256__",
            &spec
                .release_bundle
                .code_mode_host
                .extracted_executable_sha256,
        )
        .replace(
            "__SUBSTRATE_CODEX_MAIN_ARCHIVE_LENGTH__",
            &spec.release_bundle.main.archive_byte_length.to_string(),
        )
        .replace(
            "__SUBSTRATE_CODEX_MAIN_EXTRACTED_LENGTH__",
            &spec.release_bundle.main.extracted_byte_length.to_string(),
        )
        .replace(
            "__SUBSTRATE_CODEX_HOST_ARCHIVE_LENGTH__",
            &spec
                .release_bundle
                .code_mode_host
                .archive_byte_length
                .to_string(),
        )
        .replace(
            "__SUBSTRATE_CODEX_HOST_EXTRACTED_LENGTH__",
            &spec
                .release_bundle
                .code_mode_host
                .extracted_byte_length
                .to_string(),
        )
        .replace(
            "__SUBSTRATE_CODEX_PACKAGE_NAME__",
            CODEX_RUNTIME_PACKAGE_NAME,
        )
        .replace("__SUBSTRATE_CODEX_ARCHIVE_NAME__", &spec.archive_name)
        .replace("__SUBSTRATE_CODEX_ARCHIVE_URL__", &spec.archive_url)
        .replace("__SUBSTRATE_CODEX_ARCHIVE_SHA256__", &spec.archive_sha256)
        .replace(
            "__SUBSTRATE_CODEX_ARCHIVE_ENTRY_PATH__",
            &spec.archive_entry_path,
        )
        .replace(
            "__SUBSTRATE_CODEX_EXECUTABLE_SHA256__",
            &spec.extracted_executable_sha256,
        )
        .replace("__SUBSTRATE_CODEX_VERSION__", &spec.version)
        .replace("__SUBSTRATE_CODEX_TARGET_TRIPLE__", &spec.target_triple)
        .replace(
            "__SUBSTRATE_CODEX_INSTALLER_ARTIFACT_SOURCE_V1__",
            &format!("python3 - {publication}"),
        )
}

struct WrapperFileV1 {
    name: String,
    body: String,
}

fn render_wrapper_file_v1(wrapper: &WrapperDefV1) -> Result<WrapperFileV1> {
    validate_world_deps_bin_filename(&wrapper.name)?;
    let body = match &wrapper.kind {
        WrapperKindV1::BashFunction(def) => render_bash_wrapper_v1(
            "bash_function",
            &wrapper.name,
            Some(&def.bash_source),
            "source \"$SUBSTRATE_WDP_BASH_SOURCE\"; \"$SUBSTRATE_WDP_FUNCTION\" \"$@\"",
            Some(&def.function),
            None,
            &[],
        ),
        WrapperKindV1::BashSourceExec(def) => render_bash_wrapper_v1(
            "bash_source_exec",
            &wrapper.name,
            Some(&def.bash_source),
            "source \"$SUBSTRATE_WDP_BASH_SOURCE\"; exec \"$SUBSTRATE_WDP_EXEC\" \"$@\"",
            None,
            Some(&def.exec),
            &[],
        ),
        WrapperKindV1::ShEnvExec(def) => {
            render_sh_env_exec_wrapper_v1(&wrapper.name, &def.exec, &def.env)
        }
    }?;

    Ok(WrapperFileV1 {
        name: wrapper.name.clone(),
        body,
    })
}

fn render_bash_wrapper_v1(
    kind: &str,
    name: &str,
    bash_source: Option<&str>,
    bash_body: &str,
    function: Option<&str>,
    exec: Option<&str>,
    extra_env: &[(&str, &str)],
) -> Result<String> {
    let bash_source = bash_source.unwrap_or("");
    let function = function.unwrap_or("");
    let exec = exec.unwrap_or("");

    let mut out = String::new();
    out.push_str("#!/bin/sh\n");
    out.push_str("set -eu\n");
    out.push_str("SUBSTRATE_WDP_KIND=");
    out.push_str(&sh_quote(kind));
    out.push('\n');
    out.push_str("SUBSTRATE_WDP_NAME=");
    out.push_str(&sh_quote(name));
    out.push('\n');
    out.push_str("SUBSTRATE_WDP_BASH_SOURCE=");
    out.push_str(&sh_quote(bash_source));
    out.push('\n');
    out.push_str("SUBSTRATE_WDP_FUNCTION=");
    out.push_str(&sh_quote(function));
    out.push('\n');
    out.push_str("SUBSTRATE_WDP_EXEC=");
    out.push_str(&sh_quote(exec));
    out.push('\n');

    for (key, value) in extra_env {
        out.push_str(key);
        out.push('=');
        out.push_str(&sh_quote(value));
        out.push('\n');
    }

    out.push_str("export SUBSTRATE_WDP_KIND SUBSTRATE_WDP_NAME SUBSTRATE_WDP_BASH_SOURCE SUBSTRATE_WDP_FUNCTION SUBSTRATE_WDP_EXEC\n");
    out.push_str("if command -v bash >/dev/null 2>&1; then\n");
    out.push_str("  if [ -n \"$SUBSTRATE_WDP_BASH_SOURCE\" ] && [ ! -f \"$SUBSTRATE_WDP_BASH_SOURCE\" ]; then\n");
    out.push_str("    echo \"substrate: world deps wrapper failure: kind=$SUBSTRATE_WDP_KIND name=$SUBSTRATE_WDP_NAME bash_source=$SUBSTRATE_WDP_BASH_SOURCE bash_found=true\" >&2\n");
    out.push_str("    echo \"substrate: next: fix the wrapper source/fields or run 'substrate world deps current show ");
    out.push_str(name);
    out.push_str(" --explain'\" >&2\n");
    out.push_str("    exit 127\n");
    out.push_str("  fi\n");
    out.push_str("  if bash -lc ");
    out.push_str(&sh_quote(&format!("set -euo pipefail; {bash_body}")));
    out.push_str(" bash \"$@\"; then\n");
    out.push_str("    exit 0\n");
    out.push_str("  fi\n");
    out.push_str("  rc=$?\n");
    out.push_str("  echo \"substrate: world deps wrapper failure: kind=$SUBSTRATE_WDP_KIND name=$SUBSTRATE_WDP_NAME bash_source=$SUBSTRATE_WDP_BASH_SOURCE bash_found=true exit=$rc\" >&2\n");
    out.push_str("  echo \"substrate: next: fix the wrapper source/fields or run 'substrate world deps current show ");
    out.push_str(name);
    out.push_str(" --explain'\" >&2\n");
    out.push_str("  exit $rc\n");
    out.push_str("fi\n");
    out.push_str("echo \"substrate: world deps wrapper failure: kind=$SUBSTRATE_WDP_KIND name=$SUBSTRATE_WDP_NAME bash_source=$SUBSTRATE_WDP_BASH_SOURCE bash_found=false\" >&2\n");
    out.push_str("echo \"substrate: next: install bash in the world and retry (or run 'substrate world deps current show ");
    out.push_str(name);
    out.push_str(" --explain')\" >&2\n");
    out.push_str("exit 127\n");
    Ok(out)
}

fn render_sh_env_exec_wrapper_v1(
    name: &str,
    exec_cmd: &str,
    env_map: &HashMap<String, String>,
) -> Result<String> {
    let mut keys = env_map.keys().cloned().collect::<Vec<_>>();
    keys.sort();

    let mut out = String::new();
    out.push_str("#!/bin/sh\n");
    out.push_str("set -eu\n");
    out.push_str("SUBSTRATE_WDP_KIND='sh_env_exec'\n");
    out.push_str("SUBSTRATE_WDP_NAME=");
    out.push_str(&sh_quote(name));
    out.push('\n');
    out.push_str("SUBSTRATE_WDP_EXEC=");
    out.push_str(&sh_quote(exec_cmd));
    out.push('\n');

    for key in &keys {
        let value = env_map.get(key).map(String::as_str).unwrap_or("");
        out.push_str("export ");
        out.push_str(key);
        out.push('=');
        out.push_str(&sh_quote_if_needed(value));
        out.push('\n');
    }

    out.push_str("set -- $SUBSTRATE_WDP_EXEC\n");
    out.push_str("cmd=\"$1\"\n");
    out.push_str("if ! command -v \"$cmd\" >/dev/null 2>&1; then\n");
    out.push_str("  echo \"substrate: world deps wrapper failure: kind=$SUBSTRATE_WDP_KIND name=$SUBSTRATE_WDP_NAME exec=$SUBSTRATE_WDP_EXEC\" >&2\n");
    out.push_str("  echo \"substrate: next: ensure the exec target is installed in the world and retry (or run 'substrate world deps current show ");
    out.push_str(name);
    out.push_str(" --explain')\" >&2\n");
    out.push_str("  exit 127\n");
    out.push_str("fi\n");
    if exec_cmd.split_whitespace().count() == 1 {
        out.push_str("exec ");
        out.push_str(exec_cmd);
        out.push_str(" \"$@\"\n");
    } else {
        out.push_str("exec $SUBSTRATE_WDP_EXEC \"$@\"\n");
    }
    Ok(out)
}

fn sh_quote_if_needed(value: &str) -> String {
    if value.is_empty() {
        return "''".to_string();
    }
    let safe = value.chars().all(|c| {
        c.is_ascii_alphanumeric()
            || matches!(c, '_' | '-' | '.' | '/' | ':' | '@' | '%' | '+' | '=' | ',')
    });
    if safe {
        value.to_string()
    } else {
        sh_quote(value)
    }
}

fn validate_world_deps_bin_filename(name: &str) -> Result<()> {
    let trimmed = name.trim();
    if trimmed.is_empty()
        || trimmed == "."
        || trimmed == ".."
        || trimmed.contains('/')
        || trimmed.contains('\\')
    {
        return Err(config_model::user_error(format!(
            "invalid deps inventory: wrapper name '{name}' is not a valid filename"
        )));
    }
    Ok(())
}

fn validate_world_deps_entrypoint_filename(name: &str) -> Result<()> {
    let trimmed = name.trim();
    if trimmed.is_empty()
        || trimmed == "."
        || trimmed == ".."
        || trimmed.contains('/')
        || trimmed.contains('\\')
    {
        return Err(config_model::user_error(format!(
            "invalid deps inventory: entrypoint name '{name}' is not a valid filename"
        )));
    }
    Ok(())
}

fn build_world_script_install_command_v1(
    installer_script: &str,
    wrappers: &[WrapperFileV1],
) -> String {
    let mut cmd = String::new();
    cmd.push_str("set -e\n");
    cmd.push_str("world_deps_bin=\"${SUBSTRATE_WORLD_DEPS_GUEST_BIN_DIR:-/var/lib/substrate/world-deps/bin}\"\n");
    cmd.push_str("export SUBSTRATE_WORLD_DEPS_GUEST_BIN_DIR=\"$world_deps_bin\"\n");
    cmd.push_str("mkdir -p \"$world_deps_bin\"\n");
    cmd.push_str("if command -v bash >/dev/null 2>&1; then\n");
    cmd.push_str("  set -- bash -lc\n");
    cmd.push_str("else\n");
    cmd.push_str("  set -- sh -c\n");
    cmd.push_str("fi\n");
    cmd.push_str("\"$@\" \"$(cat <<'__SUBSTRATE_WDP4_INSTALL__'\n");
    cmd.push_str(installer_script);
    if !installer_script.ends_with('\n') {
        cmd.push('\n');
    }
    cmd.push_str("__SUBSTRATE_WDP4_INSTALL__\n");
    cmd.push_str(")\"\n");

    for (idx, wrapper) in wrappers.iter().enumerate() {
        let delimiter = format!("__SUBSTRATE_WDP4_WRAPPER_{idx}__");

        cmd.push_str("cat > \"$world_deps_bin/");
        cmd.push_str(&wrapper.name);
        cmd.push_str("\" <<'");
        cmd.push_str(&delimiter);
        cmd.push_str("'\n");
        cmd.push_str(&wrapper.body);
        if !wrapper.body.ends_with('\n') {
            cmd.push('\n');
        }
        cmd.push_str(&delimiter);
        cmd.push('\n');
        cmd.push_str("chmod 0755 ");
        cmd.push_str("\"$world_deps_bin/");
        cmd.push_str(&wrapper.name);
        cmd.push('"');
        cmd.push('\n');
    }

    cmd
}

fn run_world_command_checked_for_deps(
    cmd: &str,
    cwd_override: Option<&str>,
    #[cfg(target_os = "linux")] context: &AuthenticatedWorldDepsContextV1,
) -> Result<()> {
    let response = run_world_command_for_deps_at(
        cmd,
        cwd_override,
        None,
        #[cfg(target_os = "linux")]
        context,
    )
    .map_err(classify_world_backend_error)?;

    if response.exit == 0 {
        return Ok(());
    }

    let stdout = BASE64
        .decode(response.stdout_b64.as_bytes())
        .unwrap_or_default();
    let stderr = BASE64
        .decode(response.stderr_b64.as_bytes())
        .unwrap_or_default();
    let stderr_text = String::from_utf8_lossy(&stderr);
    let stdout_text = String::from_utf8_lossy(&stdout);
    let snippet = if !stderr_text.trim().is_empty() {
        stderr_text.trim().to_string()
    } else {
        stdout_text.trim().to_string()
    };
    if response.exit == 5 {
        return Err(anyhow!(WorldDepsSafetyViolationError::new(format!(
            "world command blocked (exit=5): {snippet}"
        ))));
    }
    Err(anyhow!(
        "world command failed (exit={}): {}",
        response.exit,
        snippet
    ))
}

fn expand_items_to_packages_v1(
    view: &InventoryViewV1,
    item_names: &[String],
) -> Result<Vec<String>> {
    let mut unknown: Vec<String> = Vec::new();
    let mut out: Vec<String> = Vec::new();
    let mut seen: HashSet<String> = HashSet::new();

    for item_name in item_names {
        let item = match view.get(item_name) {
            Some(item) => item,
            None => {
                unknown.push(item_name.clone());
                continue;
            }
        };
        match item {
            InventoryItemDefV1::Package(_) => {
                if seen.insert(item_name.clone()) {
                    out.push(item_name.clone());
                }
            }
            InventoryItemDefV1::Bundle(bundle) => {
                for pkg in &bundle.packages {
                    if !view.packages.contains_key(pkg) {
                        return Err(config_model::user_error(format!(
                            "invalid deps inventory: bundle '{}' references unknown package '{}'",
                            bundle.name, pkg
                        )));
                    }
                    if seen.insert(pkg.clone()) {
                        out.push(pkg.clone());
                    }
                }
            }
        }
    }

    if !unknown.is_empty() {
        unknown.sort();
        unknown.dedup();
        return Err(config_model::user_error(format!(
            "unknown deps item(s): {}",
            unknown.join(",")
        )));
    }

    Ok(out)
}

fn print_install_plan_v1(plan: &InstallPlanV1) {
    println!("World deps plan (dry-run)");

    let apt_items: Vec<String> = plan
        .apt
        .iter()
        .map(|spec| match &spec.version {
            Some(v) => format!("{}={}", spec.name, v),
            None => spec.name.clone(),
        })
        .collect();
    print_plan_list_v1("APT", &apt_items);
    print_plan_list_v1("PACMAN", &plan.pacman_packages);
    print_plan_list_v1("SCRIPT", &plan.script_packages);

    if plan.manual_packages.is_empty() {
        return;
    }
    println!("MANUAL (blocked):");
    for pkg in &plan.manual_packages {
        println!("  - {}", pkg.name);
        let instructions = pkg.manual_instructions.trim();
        if instructions.is_empty() {
            continue;
        }
        for line in instructions.lines() {
            println!("      {line}");
        }
    }
}

fn print_plan_list_v1(label: &str, items: &[String]) {
    if items.is_empty() {
        println!("{label}: -");
        return;
    }
    println!("{label}:");
    for item in items {
        println!("  - {item}");
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum WorldStatusV1 {
    Present,
    Missing,
    Blocked,
}

impl WorldStatusV1 {
    fn as_str(&self) -> &'static str {
        match self {
            Self::Present => "present",
            Self::Missing => "missing",
            Self::Blocked => "blocked",
        }
    }
}

#[derive(Debug, Serialize)]
struct CurrentShowExplainV1 {
    schema_version: u32,
    item_name: String,
    enabled: bool,
    enabled_via_global_patch: bool,
    enabled_via_workspace_patch: bool,
    world: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    remediation: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    manual_instructions: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    wrappers: Vec<WrapperExplainV1>,
    #[serde(skip_serializing_if = "Option::is_none")]
    why: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    next_command: Option<String>,
}

#[derive(Debug, Serialize)]
struct WrapperExplainV1 {
    name: String,
    kind: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    bash_source: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    function: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    exec: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    env_keys: Vec<String>,
    invocation: String,
}

fn build_current_show_explain_v1(
    cwd: &Path,
    cfg: &config_model::SubstrateConfig,
    view: &InventoryViewV1,
    item_name: &str,
    item: &InventoryItemDefV1,
    #[cfg(unix)] context: &AuthenticatedWorldDepsContextV1,
) -> Result<CurrentShowExplainV1> {
    let enabled = cfg.world.deps.enabled.iter().any(|name| name == item_name);
    let enabled_via_global_patch = {
        #[cfg(unix)]
        let path = context.global_config_path();
        #[cfg(not(unix))]
        let path = &config_model::global_config_path()?;
        let patch = match fs::read_to_string(path) {
            Ok(raw) => config_model::parse_config_patch_yaml(path, &raw)?,
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
                config_model::SubstrateConfigPatch::default()
            }
            Err(err) => {
                return Err(anyhow!("failed to read {}: {err}", path.display()));
            }
        };
        patch
            .world
            .deps
            .enabled
            .unwrap_or_default()
            .iter()
            .any(|name| name == item_name)
    };
    let enabled_via_workspace_patch = {
        if let Some(workspace_root) = crate::execution::find_workspace_root(cwd) {
            let path = workspace_marker_path(&workspace_root);
            match fs::read_to_string(&path) {
                Ok(raw) => {
                    let patch = config_model::parse_config_patch_yaml(&path, &raw)?;
                    patch
                        .world
                        .deps
                        .enabled
                        .unwrap_or_default()
                        .iter()
                        .any(|name| name == item_name)
                }
                Err(err) if err.kind() == std::io::ErrorKind::NotFound => false,
                Err(err) => {
                    return Err(anyhow!(
                        "failed to read workspace config patch at {}: {err}",
                        path.display()
                    ))
                }
            }
        } else {
            false
        }
    };

    let mut packages_to_check: Vec<String> = Vec::new();
    collect_required_package_names(item_name, item, view, &mut packages_to_check);
    let package_statuses = query_world_package_presence(
        view,
        &packages_to_check,
        #[cfg(target_os = "linux")]
        context,
    )?;
    let (world, remediation) =
        compute_world_status_and_remediation(item_name, item, view, &package_statuses, enabled)?;

    let wrappers = wrappers_explain(item);
    let manual_instructions = match item {
        InventoryItemDefV1::Package(pkg) if pkg.install.method == InstallMethodV1::Manual => pkg
            .install
            .manual_instructions
            .as_deref()
            .map(|t| t.trim().to_string())
            .filter(|t| !t.is_empty()),
        _ => None,
    };

    let (why, next_command) = if world != WorldStatusV1::Present {
        let why = format!("world status is '{}'", world.as_str());
        let next_command = if world == WorldStatusV1::Blocked {
            Some(format!(
                "substrate world deps current install {}",
                shell_escape_item_name(item_name)
            ))
        } else if enabled {
            Some("substrate world deps current sync".to_string())
        } else {
            Some(format!(
                "substrate world deps current install {}",
                shell_escape_item_name(item_name)
            ))
        };
        (Some(why), next_command)
    } else {
        // Even when the item is present, a consistent "next step" keeps `--explain` output
        // actionable and aligns with the behavior smoke harness expectations.
        (
            None,
            Some("substrate world deps current list applied".to_string()),
        )
    };

    Ok(CurrentShowExplainV1 {
        schema_version: 1,
        item_name: item_name.to_string(),
        enabled,
        enabled_via_global_patch,
        enabled_via_workspace_patch,
        world: world.as_str().to_string(),
        remediation,
        manual_instructions,
        wrappers,
        why,
        next_command,
    })
}

fn wrappers_explain(item: &InventoryItemDefV1) -> Vec<WrapperExplainV1> {
    let InventoryItemDefV1::Package(pkg) = item else {
        return Vec::new();
    };
    let mut out = Vec::with_capacity(pkg.wrappers.len());
    for wrapper in &pkg.wrappers {
        out.push(wrapper_explain(wrapper));
    }
    out
}

fn wrapper_explain(wrapper: &WrapperDefV1) -> WrapperExplainV1 {
    match &wrapper.kind {
        WrapperKindV1::BashFunction(def) => WrapperExplainV1 {
            name: wrapper.name.clone(),
            kind: "bash_function".to_string(),
            bash_source: Some(def.bash_source.clone()),
            function: Some(def.function.clone()),
            exec: None,
            env_keys: Vec::new(),
            invocation: format!(
                "bash -lc {}",
                sh_quote(&format!(
                    "source {}; {} \"$@\"",
                    def.bash_source, def.function
                ))
            ),
        },
        WrapperKindV1::BashSourceExec(def) => WrapperExplainV1 {
            name: wrapper.name.clone(),
            kind: "bash_source_exec".to_string(),
            bash_source: Some(def.bash_source.clone()),
            function: None,
            exec: Some(def.exec.clone()),
            env_keys: Vec::new(),
            invocation: format!(
                "bash -lc {}",
                sh_quote(&format!(
                    "source {}; exec {} \"$@\"",
                    def.bash_source, def.exec
                ))
            ),
        },
        WrapperKindV1::ShEnvExec(def) => {
            let mut keys = def.env.keys().cloned().collect::<Vec<_>>();
            keys.sort();
            let mut prelude = String::new();
            for key in &keys {
                prelude.push_str("export ");
                prelude.push_str(key);
                prelude.push_str("=...; ");
            }
            prelude.push_str("exec ");
            prelude.push_str(&def.exec);
            prelude.push_str(" \"$@\"");
            WrapperExplainV1 {
                name: wrapper.name.clone(),
                kind: "sh_env_exec".to_string(),
                bash_source: None,
                function: None,
                exec: Some(def.exec.clone()),
                env_keys: keys,
                invocation: format!("sh -c {}", sh_quote(&prelude)),
            }
        }
    }
}

fn run_global_list(
    args: &WorldDepsScopedListArgs,
    #[cfg(unix)] context: &AuthenticatedWorldDepsContextV1,
) -> Result<()> {
    match args.view {
        WorldDepsScopedListViewArg::Available => {
            let platform = HostPlatform::current();
            #[cfg(unix)]
            let deps_dir = context.global_deps_dir().to_path_buf();
            #[cfg(not(unix))]
            let deps_dir = substrate_paths::substrate_home()?.join("deps");
            let view = load_inventory_dir_v1(&deps_dir, platform)?;
            let items = summarize_inventory_v1(&view);
            if args.json {
                let out = ListOutputV1 {
                    schema_version: 1,
                    scope: "global".to_string(),
                    view: "available".to_string(),
                    items,
                };
                println!("{}", serde_json::to_string_pretty(&out)?);
            } else {
                print_inventory_table(&items);
            }
            Ok(())
        }
        WorldDepsScopedListViewArg::Enabled => {
            #[cfg(unix)]
            let path = context.global_config_path();
            #[cfg(not(unix))]
            let path = &config_model::global_config_path()?;
            let patch = match fs::read_to_string(path) {
                Ok(raw) => config_model::parse_config_patch_yaml(path, &raw)?,
                Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
                    config_model::SubstrateConfigPatch::default()
                }
                Err(err) => {
                    return Err(anyhow!("failed to read {}: {err}", path.display()));
                }
            };
            print_config_patch(&patch, args.json)
        }
    }
}

fn run_workspace_list(
    args: &WorldDepsScopedListArgs,
    #[cfg(unix)] context: &AuthenticatedWorldDepsContextV1,
) -> Result<()> {
    #[cfg(unix)]
    let workspace_root = context.workspace_root().map(Path::to_path_buf);
    #[cfg(not(unix))]
    let workspace_root =
        crate::execution::find_workspace_root(&env::current_dir().unwrap_or_else(|_| ".".into()));
    let workspace_root = workspace_root
        .ok_or_else(|| config_model::user_error("no workspace root detected for this directory"))?;

    match args.view {
        WorldDepsScopedListViewArg::Available => {
            let platform = HostPlatform::current();
            let deps_dir = workspace_root.join(".substrate").join("deps");
            let view = load_inventory_dir_v1(&deps_dir, platform)?;
            let items = summarize_inventory_v1(&view);
            if args.json {
                let out = ListOutputV1 {
                    schema_version: 1,
                    scope: "workspace".to_string(),
                    view: "available".to_string(),
                    items,
                };
                println!("{}", serde_json::to_string_pretty(&out)?);
            } else {
                print_inventory_table(&items);
            }
            Ok(())
        }
        WorldDepsScopedListViewArg::Enabled => {
            let path = workspace_marker_path(&workspace_root);
            let raw = fs::read_to_string(&path)
                .with_context(|| format!("failed to read {}", path.display()))?;
            let patch = config_model::parse_config_patch_yaml(&path, &raw)?;
            print_config_patch(&patch, args.json)
        }
    }
}

#[derive(Debug, Serialize)]
struct MutateOutputV1 {
    schema_version: u32,
    scope: String,
    action: String,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    added: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    removed: Vec<String>,
}

const DEFAULT_GLOBAL_WORLD_DEPS_PATCH_HEADER: &str = r#"# Substrate world deps enabled patch (global scope).
# - Update via:
#   - `substrate world deps global add ...`
#   - `substrate world deps global remove ...`
#   - `substrate world deps global reset`
# - Or edit this file directly (YAML).
# - Changes do not affect the world until you run:
#   - `substrate world deps current sync`
# - Inspect the effective view for your current directory:
#   - `substrate world deps current list enabled`
#   - `substrate world deps current list applied`
"#;

fn run_global_add(
    args: &WorldDepsScopedMutateArgs,
    #[cfg(unix)] context: &AuthenticatedWorldDepsContextV1,
) -> Result<()> {
    let items = dedupe_ordered(&args.item_names);
    let view = resolve_global_available_inventory_view(
        #[cfg(unix)]
        context,
    )?;

    let unknown = items
        .iter()
        .filter(|name| view.get(name).is_none())
        .cloned()
        .collect::<Vec<_>>();
    if !unknown.is_empty() {
        return Err(config_model::user_error(format!(
            "unknown deps item(s): {}",
            unknown.join(",")
        )));
    }

    #[cfg(unix)]
    let path = context.global_config_path().to_path_buf();
    #[cfg(not(unix))]
    let path = config_model::global_config_path()?;
    let (mut patch, existed) = match fs::read_to_string(&path) {
        Ok(raw) => (config_model::parse_config_patch_yaml(&path, &raw)?, true),
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
            (config_model::SubstrateConfigPatch::default(), false)
        }
        Err(err) => return Err(anyhow!("failed to read {}: {err}", path.display())),
    };

    let before = patch.world.deps.enabled.clone().unwrap_or_default();
    let added = items
        .iter()
        .filter(|name| !before.iter().any(|existing| existing == *name))
        .cloned()
        .collect::<Vec<_>>();

    let updates = items
        .into_iter()
        .map(|name| config_model::ConfigUpdate {
            key: "world.deps.enabled".to_string(),
            op: config_model::UpdateOp::Append,
            value: name,
        })
        .collect::<Vec<_>>();
    let changed = config_model::apply_updates_to_patch(&mut patch, &updates)?;

    if changed || (!existed && !patch.is_empty()) {
        let header = if existed {
            Some(read_comment_header_prefix(&path)?)
        } else {
            None
        };
        write_atomic_patch_yaml(
            &path,
            DEFAULT_GLOBAL_WORLD_DEPS_PATCH_HEADER,
            header.as_deref(),
            &patch,
        )
        .with_context(|| format!("failed to write {}", path.display()))?;
        config_model::invalidate_config_cache();
    }

    if args.json {
        let out = MutateOutputV1 {
            schema_version: 1,
            scope: "global".to_string(),
            action: "add".to_string(),
            added,
            removed: Vec::new(),
        };
        println!("{}", serde_json::to_string_pretty(&out)?);
    } else {
        println!("Enabled deps updated (global): added: {}", csv(&added));
    }
    println!(
        "substrate: note: enabled deps changes apply to the world only after 'substrate world deps current sync'"
    );
    Ok(())
}

fn run_global_remove(
    args: &WorldDepsScopedMutateArgs,
    #[cfg(unix)] context: &AuthenticatedWorldDepsContextV1,
) -> Result<()> {
    let items = dedupe_ordered(&args.item_names);

    #[cfg(unix)]
    let path = context.global_config_path().to_path_buf();
    #[cfg(not(unix))]
    let path = config_model::global_config_path()?;
    let (mut patch, existed) = match fs::read_to_string(&path) {
        Ok(raw) => (config_model::parse_config_patch_yaml(&path, &raw)?, true),
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
            (config_model::SubstrateConfigPatch::default(), false)
        }
        Err(err) => return Err(anyhow!("failed to read {}: {err}", path.display())),
    };

    let before = patch.world.deps.enabled.clone().unwrap_or_default();
    let removed = items
        .iter()
        .filter(|name| before.iter().any(|existing| existing == *name))
        .cloned()
        .collect::<Vec<_>>();

    let updates = items
        .into_iter()
        .map(|name| config_model::ConfigUpdate {
            key: "world.deps.enabled".to_string(),
            op: config_model::UpdateOp::Remove,
            value: name,
        })
        .collect::<Vec<_>>();
    let changed = config_model::apply_updates_to_patch(&mut patch, &updates)?;

    if changed {
        let header = if existed {
            Some(read_comment_header_prefix(&path)?)
        } else {
            None
        };
        write_atomic_patch_yaml(
            &path,
            DEFAULT_GLOBAL_WORLD_DEPS_PATCH_HEADER,
            header.as_deref(),
            &patch,
        )
        .with_context(|| format!("failed to write {}", path.display()))?;
        config_model::invalidate_config_cache();
    }

    if args.json {
        let out = MutateOutputV1 {
            schema_version: 1,
            scope: "global".to_string(),
            action: "remove".to_string(),
            added: Vec::new(),
            removed: removed.clone(),
        };
        println!("{}", serde_json::to_string_pretty(&out)?);
    } else {
        println!("Enabled deps updated (global): removed: {}", csv(&removed));
    }
    println!("substrate: note: 'remove' only updates enabled deps; it does not uninstall. Run 'substrate world deps current sync' to apply");

    if !removed.is_empty() {
        #[cfg(unix)]
        let workspace_root = context.workspace_root().map(Path::to_path_buf);
        #[cfg(not(unix))]
        let workspace_root = crate::execution::find_workspace_root(
            &env::current_dir().unwrap_or_else(|_| ".".into()),
        );
        if let Some(workspace_root) = workspace_root {
            let ws_patch_path = workspace_marker_path(&workspace_root);
            let raw = fs::read_to_string(&ws_patch_path)
                .with_context(|| format!("failed to read {}", ws_patch_path.display()))?;
            let ws_patch = config_model::parse_config_patch_yaml(&ws_patch_path, &raw)?;
            let ws_enabled = ws_patch.world.deps.enabled.unwrap_or_default();
            for item in removed {
                if ws_enabled.iter().any(|name| name == &item) {
                    println!("substrate: note: '{item}' was removed from global enabled deps but is still enabled via workspace; run 'substrate world deps workspace remove {item}' to fully disable it for this workspace");
                }
            }
        }
    }

    Ok(())
}

fn run_global_reset(
    args: &WorldDepsScopedResetArgs,
    #[cfg(unix)] context: &AuthenticatedWorldDepsContextV1,
) -> Result<()> {
    #[cfg(unix)]
    let path = context.global_config_path().to_path_buf();
    #[cfg(not(unix))]
    let path = config_model::global_config_path()?;
    let (mut patch, existed) = match fs::read_to_string(&path) {
        Ok(raw) => (config_model::parse_config_patch_yaml(&path, &raw)?, true),
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
            (config_model::SubstrateConfigPatch::default(), false)
        }
        Err(err) => return Err(anyhow!("failed to read {}: {err}", path.display())),
    };

    let changed = config_model::reset_patch_keys(&mut patch, &["world.deps.enabled".to_string()])?;
    if changed {
        let header = if existed {
            Some(read_comment_header_prefix(&path)?)
        } else {
            None
        };
        write_atomic_patch_yaml(
            &path,
            DEFAULT_GLOBAL_WORLD_DEPS_PATCH_HEADER,
            header.as_deref(),
            &patch,
        )
        .with_context(|| format!("failed to write {}", path.display()))?;
        config_model::invalidate_config_cache();
    }

    if args.json {
        let out = MutateOutputV1 {
            schema_version: 1,
            scope: "global".to_string(),
            action: "reset".to_string(),
            added: Vec::new(),
            removed: Vec::new(),
        };
        println!("{}", serde_json::to_string_pretty(&out)?);
    } else {
        println!("Enabled deps reset (global)");
    }
    println!(
        "substrate: note: run 'substrate world deps current sync' to apply enabled deps changes"
    );
    Ok(())
}

fn run_workspace_add(
    args: &WorldDepsScopedMutateArgs,
    #[cfg(unix)] context: &AuthenticatedWorldDepsContextV1,
) -> Result<()> {
    #[cfg(unix)]
    let (cwd, cfg, workspace_root) = (
        context.launch_cwd(),
        context.effective_config(),
        context.workspace_root().map(Path::to_path_buf),
    );
    #[cfg(not(unix))]
    let cwd = env::current_dir().unwrap_or_else(|_| ".".into());
    #[cfg(not(unix))]
    let cfg = config_model::resolve_effective_config(&cwd, &Default::default())
        .context("failed to resolve effective config")?;
    #[cfg(not(unix))]
    let workspace_root = crate::execution::find_workspace_root(&cwd);
    let workspace_root = workspace_root
        .ok_or_else(|| config_model::user_error("no workspace root detected for this directory"))?;

    let items = dedupe_ordered(&args.item_names);
    let global_deps_dir =
        if cfg.world.deps.inventory_mode == config_model::WorldDepsInventoryMode::Merged {
            #[cfg(unix)]
            {
                Some(context.global_deps_dir().to_path_buf())
            }
            #[cfg(not(unix))]
            {
                Some(substrate_paths::substrate_home()?.join("deps"))
            }
        } else {
            None
        };
    let view = resolve_current_inventory_view(cwd, cfg, global_deps_dir.as_deref())?;

    let unknown = items
        .iter()
        .filter(|name| view.get(name).is_none())
        .cloned()
        .collect::<Vec<_>>();
    if !unknown.is_empty() {
        return Err(config_model::user_error(format!(
            "unknown deps item(s): {}",
            unknown.join(",")
        )));
    }

    let path = workspace_marker_path(&workspace_root);
    let raw =
        fs::read_to_string(&path).with_context(|| format!("failed to read {}", path.display()))?;
    let header = read_comment_header_prefix_from_raw(&raw);
    let mut patch = config_model::parse_config_patch_yaml(&path, &raw)?;

    let before = patch.world.deps.enabled.clone().unwrap_or_default();
    let added = items
        .iter()
        .filter(|name| !before.iter().any(|existing| existing == *name))
        .cloned()
        .collect::<Vec<_>>();

    let updates = items
        .into_iter()
        .map(|name| config_model::ConfigUpdate {
            key: "world.deps.enabled".to_string(),
            op: config_model::UpdateOp::Append,
            value: name,
        })
        .collect::<Vec<_>>();
    let changed = config_model::apply_updates_to_patch(&mut patch, &updates)?;

    if changed {
        write_atomic_patch_yaml(&path, "", Some(&header), &patch)
            .with_context(|| format!("failed to write {}", path.display()))?;
        config_model::invalidate_config_cache();
    }

    if args.json {
        let out = MutateOutputV1 {
            schema_version: 1,
            scope: "workspace".to_string(),
            action: "add".to_string(),
            added,
            removed: Vec::new(),
        };
        println!("{}", serde_json::to_string_pretty(&out)?);
    } else {
        println!("Enabled deps updated (workspace): added: {}", csv(&added));
    }
    println!(
        "substrate: note: enabled deps changes apply to the world only after 'substrate world deps current sync'"
    );
    Ok(())
}

fn run_workspace_remove(
    args: &WorldDepsScopedMutateArgs,
    #[cfg(unix)] context: &AuthenticatedWorldDepsContextV1,
) -> Result<()> {
    #[cfg(unix)]
    let workspace_root = context.workspace_root().map(Path::to_path_buf);
    #[cfg(not(unix))]
    let workspace_root =
        crate::execution::find_workspace_root(&env::current_dir().unwrap_or_else(|_| ".".into()));
    let workspace_root = workspace_root
        .ok_or_else(|| config_model::user_error("no workspace root detected for this directory"))?;

    let items = dedupe_ordered(&args.item_names);

    let path = workspace_marker_path(&workspace_root);
    let raw =
        fs::read_to_string(&path).with_context(|| format!("failed to read {}", path.display()))?;
    let header = read_comment_header_prefix_from_raw(&raw);
    let mut patch = config_model::parse_config_patch_yaml(&path, &raw)?;

    let before = patch.world.deps.enabled.clone().unwrap_or_default();
    let removed = items
        .iter()
        .filter(|name| before.iter().any(|existing| existing == *name))
        .cloned()
        .collect::<Vec<_>>();

    let updates = items
        .into_iter()
        .map(|name| config_model::ConfigUpdate {
            key: "world.deps.enabled".to_string(),
            op: config_model::UpdateOp::Remove,
            value: name,
        })
        .collect::<Vec<_>>();
    let changed = config_model::apply_updates_to_patch(&mut patch, &updates)?;

    if changed {
        write_atomic_patch_yaml(&path, "", Some(&header), &patch)
            .with_context(|| format!("failed to write {}", path.display()))?;
        config_model::invalidate_config_cache();
    }

    if args.json {
        let out = MutateOutputV1 {
            schema_version: 1,
            scope: "workspace".to_string(),
            action: "remove".to_string(),
            added: Vec::new(),
            removed: removed.clone(),
        };
        println!("{}", serde_json::to_string_pretty(&out)?);
    } else {
        println!(
            "Enabled deps updated (workspace): removed: {}",
            csv(&removed)
        );
    }
    println!("substrate: note: 'remove' only updates enabled deps; it does not uninstall. Run 'substrate world deps current sync' to apply");

    if !removed.is_empty() {
        #[cfg(unix)]
        let global_path = context.global_config_path();
        #[cfg(not(unix))]
        let global_path = &config_model::global_config_path()?;
        let global_patch = match fs::read_to_string(global_path) {
            Ok(raw) => config_model::parse_config_patch_yaml(global_path, &raw)?,
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
                config_model::SubstrateConfigPatch::default()
            }
            Err(err) => {
                return Err(anyhow!("failed to read {}: {err}", global_path.display()));
            }
        };
        let global_enabled = global_patch.world.deps.enabled.unwrap_or_default();
        for item in removed {
            if global_enabled.iter().any(|name| name == &item) {
                println!("substrate: note: '{item}' was removed from workspace enabled deps but is still enabled via global; run 'substrate world deps global remove {item}' to fully disable it");
            }
        }
    }

    Ok(())
}

fn run_workspace_reset(
    args: &WorldDepsScopedResetArgs,
    #[cfg(unix)] context: &AuthenticatedWorldDepsContextV1,
) -> Result<()> {
    #[cfg(unix)]
    let workspace_root = context.workspace_root().map(Path::to_path_buf);
    #[cfg(not(unix))]
    let workspace_root =
        crate::execution::find_workspace_root(&env::current_dir().unwrap_or_else(|_| ".".into()));
    let workspace_root = workspace_root
        .ok_or_else(|| config_model::user_error("no workspace root detected for this directory"))?;

    let path = workspace_marker_path(&workspace_root);
    let raw =
        fs::read_to_string(&path).with_context(|| format!("failed to read {}", path.display()))?;
    let header = read_comment_header_prefix_from_raw(&raw);
    let mut patch = config_model::parse_config_patch_yaml(&path, &raw)?;

    let changed = config_model::reset_patch_keys(&mut patch, &["world.deps.enabled".to_string()])?;
    if changed {
        write_atomic_patch_yaml(&path, "", Some(&header), &patch)
            .with_context(|| format!("failed to write {}", path.display()))?;
        config_model::invalidate_config_cache();
    }

    if args.json {
        let out = MutateOutputV1 {
            schema_version: 1,
            scope: "workspace".to_string(),
            action: "reset".to_string(),
            added: Vec::new(),
            removed: Vec::new(),
        };
        println!("{}", serde_json::to_string_pretty(&out)?);
    } else {
        println!("Enabled deps reset (workspace)");
    }
    println!(
        "substrate: note: run 'substrate world deps current sync' to apply enabled deps changes"
    );
    Ok(())
}

fn run_current_list_enabled(
    cwd: &std::path::Path,
    cfg: &config_model::SubstrateConfig,
    global_deps_dir: Option<&Path>,
    json: bool,
) -> Result<()> {
    eprintln!("substrate: note: showing current effective enabled deps list for this directory");

    let enabled = &cfg.world.deps.enabled;
    if enabled.is_empty() {
        eprintln!("substrate: hint: add deps with 'substrate world deps workspace add ...' (or '... global add ...') then apply with 'substrate world deps current sync'");
    }

    let view = resolve_current_inventory_view(cwd, cfg, global_deps_dir)?;
    let mut unknown: Vec<String> = Vec::new();
    let mut items: Vec<InventoryListItemSummaryV1> = Vec::with_capacity(enabled.len());
    for name in enabled {
        match view.get(name) {
            Some(item) => items.push(enabled_item_summary(&item, name)),
            None => unknown.push(name.clone()),
        }
    }
    if !unknown.is_empty() {
        return Err(config_model::user_error(format!(
            "unknown deps item(s): {}",
            unknown.join(",")
        )));
    }

    if json {
        let out = ListOutputV1 {
            schema_version: 1,
            scope: "current".to_string(),
            view: "enabled".to_string(),
            items,
        };
        println!("{}", serde_json::to_string_pretty(&out)?);
    } else {
        print_enabled_table(&items);
    }
    Ok(())
}

fn enabled_item_summary(item: &InventoryItemDefV1, name: &str) -> InventoryListItemSummaryV1 {
    let kind = match item {
        InventoryItemDefV1::Package(_) => "package",
        InventoryItemDefV1::Bundle(_) => "bundle",
    };
    InventoryListItemSummaryV1 {
        kind: kind.to_string(),
        name: name.to_string(),
        enabled: None,
        world: None,
        remediation: None,
        runnable: None,
        method: None,
        entrypoints: Vec::new(),
        platforms: Vec::new(),
        description: None,
    }
}

fn run_current_list_applied(
    cwd: &Path,
    cfg: &config_model::SubstrateConfig,
    global_deps_dir: Option<&Path>,
    all: bool,
    json: bool,
    #[cfg(target_os = "linux")] context: &AuthenticatedWorldDepsContextV1,
) -> Result<()> {
    eprintln!("substrate: note: showing current world deps status for this directory");
    let view = resolve_current_inventory_view(cwd, cfg, global_deps_dir)?;
    let items = compute_current_applied_items_v1(
        &view,
        &cfg.world.deps.enabled,
        all,
        #[cfg(target_os = "linux")]
        context,
    )?;

    if json {
        let out = ListOutputV1 {
            schema_version: 1,
            scope: "current".to_string(),
            view: "applied".to_string(),
            items,
        };
        println!("{}", serde_json::to_string_pretty(&out)?);
    } else {
        print_applied_table(&items);
    }
    Ok(())
}

pub(super) fn compute_current_applied_items_v1(
    view: &InventoryViewV1,
    enabled: &[String],
    all: bool,
    #[cfg(target_os = "linux")] context: &AuthenticatedWorldDepsContextV1,
) -> Result<Vec<InventoryListItemSummaryV1>> {
    // For `applied`, expand enabled bundles to their packages so users can see the status of
    // concrete installables even when they only enabled a bundle.
    let effective_enabled = expand_enabled_items_for_applied(view, enabled);
    let enabled_set: HashSet<&str> = effective_enabled.iter().map(|s| s.as_str()).collect();

    let mut unknown: Vec<String> = Vec::new();
    if !all {
        for name in enabled {
            if view.get(name).is_none() {
                unknown.push(name.clone());
            }
        }
        if !unknown.is_empty() {
            return Err(config_model::user_error(format!(
                "unknown deps item(s): {}",
                unknown.join(",")
            )));
        }
    }

    let names_to_display: Vec<String> = if all {
        let mut all_names: Vec<String> = Vec::new();
        for name in view.packages.keys() {
            all_names.push(name.clone());
        }
        for name in view.bundles.keys() {
            all_names.push(name.clone());
        }
        all_names.sort();
        all_names
    } else {
        effective_enabled.clone()
    };

    let mut packages_to_check: Vec<String> = Vec::new();
    if all {
        packages_to_check.extend(view.packages.keys().cloned());
    } else {
        for name in enabled {
            if let Some(item) = view.get(name) {
                collect_required_package_names(name, &item, view, &mut packages_to_check);
            }
        }
    }
    packages_to_check.sort();
    packages_to_check.dedup();

    let package_presence = query_world_package_presence(
        view,
        &packages_to_check,
        #[cfg(target_os = "linux")]
        context,
    )?;

    let mut items: Vec<InventoryListItemSummaryV1> = Vec::with_capacity(names_to_display.len());
    for name in &names_to_display {
        let Some(item) = view.get(name) else {
            // Should only happen for `--all` where inventories mutated between resolve and render.
            continue;
        };
        let enabled_here = enabled_set.contains(name.as_str());
        let (world, remediation) = compute_world_status_and_remediation(
            name,
            &item,
            view,
            &package_presence,
            enabled_here,
        )?;

        let kind = match item {
            InventoryItemDefV1::Package(_) => "package",
            InventoryItemDefV1::Bundle(_) => "bundle",
        };

        items.push(InventoryListItemSummaryV1 {
            kind: kind.to_string(),
            name: name.to_string(),
            enabled: Some(enabled_here),
            world: Some(world.as_str().to_string()),
            remediation,
            runnable: None,
            method: None,
            entrypoints: Vec::new(),
            platforms: Vec::new(),
            description: None,
        });
    }

    Ok(items)
}

fn expand_enabled_items_for_applied(view: &InventoryViewV1, enabled: &[String]) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for name in enabled {
        out.push(name.clone());
        let Some(InventoryItemDefV1::Bundle(bundle)) = view.get(name) else {
            continue;
        };
        for pkg in &bundle.packages {
            if view.packages.contains_key(pkg) {
                out.push(pkg.clone());
            }
        }
    }
    dedupe_ordered(&out)
}

fn print_applied_table(items: &[InventoryListItemSummaryV1]) {
    let mut kind_width = "Kind".len();
    let mut name_width = "Name".len();
    let mut enabled_width = "Enabled".len();
    let mut world_width = "World".len();
    for item in items {
        kind_width = kind_width.max(item.kind.len());
        name_width = name_width.max(item.name.len());
        let enabled = item
            .enabled
            .map(|v| if v { "enabled=true" } else { "enabled=false" })
            .unwrap_or("-");
        enabled_width = enabled_width.max(enabled.len());
        let world = item
            .world
            .as_deref()
            .map(|v| format!("world={v}"))
            .unwrap_or_else(|| "-".to_string());
        world_width = world_width.max(world.len());
    }

    println!(
        "{:<kind_width$} {:<name_width$} {:<enabled_width$} {:<world_width$} Remediation",
        "Kind",
        "Name",
        "Enabled",
        "World",
        kind_width = kind_width,
        name_width = name_width,
        enabled_width = enabled_width,
        world_width = world_width
    );
    println!(
        "{:-<kind_width$} {:-<name_width$} {:-<enabled_width$} {:-<world_width$} {:-<11}",
        "",
        "",
        "",
        "",
        "",
        kind_width = kind_width,
        name_width = name_width,
        enabled_width = enabled_width,
        world_width = world_width
    );
    for item in items {
        let enabled = item
            .enabled
            .map(|v| if v { "enabled=true" } else { "enabled=false" })
            .unwrap_or("-");
        let world = item
            .world
            .as_deref()
            .map(|v| format!("world={v}"))
            .unwrap_or_else(|| "-".to_string());
        let remediation = item.remediation.as_deref().unwrap_or("-");
        println!(
            "{:<kind_width$} {:<name_width$} {:<enabled_width$} {:<world_width$} {}",
            item.kind,
            item.name,
            enabled,
            world,
            remediation,
            kind_width = kind_width,
            name_width = name_width,
            enabled_width = enabled_width,
            world_width = world_width
        );
    }
}

fn collect_required_package_names(
    item_name: &str,
    item: &InventoryItemDefV1,
    view: &InventoryViewV1,
    out: &mut Vec<String>,
) {
    match item {
        InventoryItemDefV1::Package(_) => out.push(item_name.to_string()),
        InventoryItemDefV1::Bundle(bundle) => {
            for pkg in &bundle.packages {
                if view.packages.contains_key(pkg) {
                    out.push(pkg.clone());
                }
            }
        }
    }
}

fn compute_world_status_and_remediation(
    item_name: &str,
    item: &InventoryItemDefV1,
    view: &InventoryViewV1,
    package_presence: &HashMap<String, bool>,
    enabled: bool,
) -> Result<(WorldStatusV1, Option<String>)> {
    match item {
        InventoryItemDefV1::Package(pkg) => {
            let present = package_presence.get(item_name).copied().unwrap_or(false);
            let status = if present {
                WorldStatusV1::Present
            } else if pkg.install.method == InstallMethodV1::Manual {
                WorldStatusV1::Blocked
            } else {
                WorldStatusV1::Missing
            };
            let remediation = if status == WorldStatusV1::Blocked {
                Some(format!(
                    "manual install required; run 'substrate world deps current show {} --explain'",
                    item_name
                ))
            } else if status != WorldStatusV1::Present && enabled {
                Some("run 'substrate world deps current sync'".to_string())
            } else {
                None
            };
            Ok((status, remediation))
        }
        InventoryItemDefV1::Bundle(bundle) => {
            let mut any_blocked = false;
            let mut any_missing = false;
            for pkg_name in &bundle.packages {
                if let Some(pkg) = view.packages.get(pkg_name) {
                    let present = package_presence.get(pkg_name).copied().unwrap_or(false);
                    if !present {
                        if pkg.install.method == InstallMethodV1::Manual {
                            any_blocked = true;
                        } else {
                            any_missing = true;
                        }
                    }
                } else {
                    any_missing = true;
                }
            }
            let status = if !any_blocked && !any_missing {
                WorldStatusV1::Present
            } else if any_blocked {
                WorldStatusV1::Blocked
            } else {
                WorldStatusV1::Missing
            };
            let remediation = if status != WorldStatusV1::Present && enabled {
                Some("run 'substrate world deps current sync'".to_string())
            } else {
                None
            };
            Ok((status, remediation))
        }
    }
}

#[derive(Debug, Clone)]
enum PackageCheckKind {
    Probe { command: String },
    Entrypoints { entrypoints: Vec<String> },
}

#[derive(Debug, Clone)]
struct PackageWorldCheck {
    name: String,
    check: PackageCheckKind,
}

fn query_world_package_presence(
    view: &InventoryViewV1,
    package_names: &[String],
    #[cfg(target_os = "linux")] context: &AuthenticatedWorldDepsContextV1,
) -> Result<HashMap<String, bool>> {
    let mut checks: Vec<PackageWorldCheck> = Vec::new();
    for name in package_names {
        let Some(pkg) = view.packages.get(name) else {
            continue;
        };
        if let Some(probe) = &pkg.probe {
            checks.push(PackageWorldCheck {
                name: name.clone(),
                check: PackageCheckKind::Probe {
                    command: probe.command.clone(),
                },
            });
        } else if pkg.runnable && !pkg.entrypoints.is_empty() {
            checks.push(PackageWorldCheck {
                name: name.clone(),
                check: PackageCheckKind::Entrypoints {
                    entrypoints: pkg.entrypoints.clone(),
                },
            });
        } else {
            // No probe method; treat as missing (do not call world).
        }
    }

    if checks.is_empty() {
        ensure_world_backend_available(
            #[cfg(target_os = "linux")]
            context,
        )?;
        return Ok(HashMap::new());
    }

    let script = build_world_probe_script(&checks);
    let response = run_world_command_for_deps(
        &script,
        #[cfg(target_os = "linux")]
        context,
    )
    .map_err(classify_world_backend_error)?;

    let stdout = BASE64
        .decode(response.stdout_b64.as_bytes())
        .unwrap_or_default();
    let stderr = BASE64
        .decode(response.stderr_b64.as_bytes())
        .unwrap_or_default();
    if response.exit != 0 {
        let stderr_text = String::from_utf8_lossy(&stderr);
        let stdout_text = String::from_utf8_lossy(&stdout);
        let snippet = if !stderr_text.trim().is_empty() {
            stderr_text.trim().to_string()
        } else {
            stdout_text.trim().to_string()
        };
        return Err(anyhow!(WorldDepsBackendUnavailableError::new(format!(
            "world probe script failed (exit={}): {}",
            response.exit, snippet
        ))));
    }

    let stdout_text = String::from_utf8_lossy(&stdout);
    let mut out: HashMap<String, bool> = parse_world_probe_output(&stdout_text);

    // Some test stubs (and some degraded backends) may drop stdout entirely. When we can't recover
    // per-package statuses from the bulk probe script, fall back to individual exit-code checks.
    let missing_checks: Vec<PackageWorldCheck> = checks
        .iter()
        .filter(|check| !out.contains_key(&check.name))
        .cloned()
        .collect();
    if !missing_checks.is_empty() {
        for check in missing_checks {
            let present = run_world_presence_check_v1(
                &check,
                #[cfg(target_os = "linux")]
                context,
            )?;
            out.insert(check.name.clone(), present);
        }
    }

    Ok(out)
}

fn query_world_package_entrypoint_presence(
    view: &InventoryViewV1,
    package_names: &[String],
    #[cfg(target_os = "linux")] context: &AuthenticatedWorldDepsContextV1,
) -> Result<HashMap<String, bool>> {
    let mut checks: Vec<PackageWorldCheck> = Vec::new();
    for name in package_names {
        let Some(pkg) = view.packages.get(name) else {
            continue;
        };
        if pkg.runnable && !pkg.entrypoints.is_empty() {
            checks.push(PackageWorldCheck {
                name: name.clone(),
                check: PackageCheckKind::Entrypoints {
                    entrypoints: pkg.entrypoints.clone(),
                },
            });
        }
    }

    if checks.is_empty() {
        ensure_world_backend_available(
            #[cfg(target_os = "linux")]
            context,
        )?;
        return Ok(HashMap::new());
    }

    let script = build_world_probe_script(&checks);
    let response = run_world_command_for_deps(
        &script,
        #[cfg(target_os = "linux")]
        context,
    )
    .map_err(classify_world_backend_error)?;

    let stdout = BASE64
        .decode(response.stdout_b64.as_bytes())
        .unwrap_or_default();
    let stderr = BASE64
        .decode(response.stderr_b64.as_bytes())
        .unwrap_or_default();
    if response.exit != 0 {
        let stderr_text = String::from_utf8_lossy(&stderr);
        let stdout_text = String::from_utf8_lossy(&stdout);
        let snippet = if !stderr_text.trim().is_empty() {
            stderr_text.trim().to_string()
        } else {
            stdout_text.trim().to_string()
        };
        return Err(anyhow!(WorldDepsBackendUnavailableError::new(format!(
            "world probe script failed (exit={}): {}",
            response.exit, snippet
        ))));
    }

    let stdout_text = String::from_utf8_lossy(&stdout);
    let mut out: HashMap<String, bool> = parse_world_probe_output(&stdout_text);

    let missing_checks: Vec<PackageWorldCheck> = checks
        .iter()
        .filter(|check| !out.contains_key(&check.name))
        .cloned()
        .collect();
    if !missing_checks.is_empty() {
        for check in missing_checks {
            let present = run_world_presence_check_v1(
                &check,
                #[cfg(target_os = "linux")]
                context,
            )?;
            out.insert(check.name.clone(), present);
        }
    }

    Ok(out)
}

fn parse_world_probe_output(stdout: &str) -> HashMap<String, bool> {
    let mut out: HashMap<String, bool> = HashMap::new();
    for line in stdout.lines() {
        let Some(rest) = line.strip_prefix("__SUBSTRATE_WDP2__ ") else {
            continue;
        };
        let mut parts = rest.split_whitespace();
        let Some(name) = parts.next() else {
            continue;
        };
        let val = parts.next();
        let present = matches!(val, Some("1"));
        out.insert(name.to_string(), present);
    }
    out
}

fn run_world_presence_check_v1(
    check: &PackageWorldCheck,
    #[cfg(target_os = "linux")] context: &AuthenticatedWorldDepsContextV1,
) -> Result<bool> {
    let cmd = match &check.check {
        PackageCheckKind::Probe { command } => {
            // Match the bulk probe script semantics: treat the probe string as a shell snippet.
            // Redirect output so the caller can rely on the exit code only.
            //
            // Defensive: force the sanitized PATH contract so "present" does not depend on any
            // host-inherited PATH (even when the backend executes under a login shell).
            let mut snippet = String::new();
            snippet.push_str(
                "world_deps_bin=\"${SUBSTRATE_WORLD_DEPS_GUEST_BIN_DIR:-/var/lib/substrate/world-deps/bin}\"\n",
            );
            snippet.push_str("world_deps_bin=\"${world_deps_bin%/}\"\n");
            snippet.push_str("export SUBSTRATE_WORLD_DEPS_GUEST_BIN_DIR=\"$world_deps_bin\"\n");
            snippet.push_str(
                "export PATH=\"$world_deps_bin:/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin\"\n",
            );
            snippet.push_str("sh -c ");
            snippet.push_str(&sh_quote(command));
            snippet.push_str(" >/dev/null 2>&1\n");
            snippet.push_str("exit $?\n");
            format!("sh -c {} >/dev/null 2>&1", sh_quote(&snippet))
        }
        PackageCheckKind::Entrypoints { entrypoints } => {
            let mut snippet = String::new();
            snippet.push_str(
                "world_deps_bin=\"${SUBSTRATE_WORLD_DEPS_GUEST_BIN_DIR:-/var/lib/substrate/world-deps/bin}\"\n",
            );
            snippet.push_str("world_deps_bin=\"${world_deps_bin%/}\"\n");
            snippet.push_str("export SUBSTRATE_WORLD_DEPS_GUEST_BIN_DIR=\"$world_deps_bin\"\n");
            snippet.push_str(
                "export PATH=\"$world_deps_bin:/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin\"\n",
            );
            snippet.push_str("for ep in");
            for ep in entrypoints {
                snippet.push(' ');
                snippet.push_str(&sh_quote(ep));
            }
            snippet.push_str("; do\n");
            snippet.push_str("  resolved=\"$(command -v \"$ep\" 2>/dev/null || true)\"\n");
            snippet.push_str("  [ \"$resolved\" = \"$world_deps_bin/$ep\" ] || exit 1\n");
            snippet.push_str("done\n");
            snippet.push_str("exit 0\n");
            format!("sh -c {} >/dev/null 2>&1", sh_quote(&snippet))
        }
    };

    let response = run_world_command_for_deps(
        &cmd,
        #[cfg(target_os = "linux")]
        context,
    )
    .map_err(classify_world_backend_error)?;
    Ok(response.exit == 0)
}

fn ensure_world_backend_available(
    #[cfg(target_os = "linux")] context: &AuthenticatedWorldDepsContextV1,
) -> Result<()> {
    let response = run_world_command_for_deps(
        ":",
        #[cfg(target_os = "linux")]
        context,
    )
    .map_err(classify_world_backend_error)?;
    if response.exit == 0 {
        return Ok(());
    }
    let stderr = BASE64
        .decode(response.stderr_b64.as_bytes())
        .unwrap_or_default();
    let stderr_text = String::from_utf8_lossy(&stderr);
    Err(anyhow!(WorldDepsBackendUnavailableError::new(format!(
        "world backend probe failed (exit={}): {}",
        response.exit,
        stderr_text.trim()
    ))))
}

fn build_world_probe_script(checks: &[PackageWorldCheck]) -> String {
    let mut script = String::new();
    script.push_str("set +e\n");
    script.push_str(
        "world_deps_bin=\"${SUBSTRATE_WORLD_DEPS_GUEST_BIN_DIR:-/var/lib/substrate/world-deps/bin}\"\n",
    );
    script.push_str("world_deps_bin=\"${world_deps_bin%/}\"\n");
    script.push_str("export SUBSTRATE_WORLD_DEPS_GUEST_BIN_DIR=\"$world_deps_bin\"\n");
    script.push_str(
        "export PATH=\"$world_deps_bin:/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin\"\n",
    );
    script.push_str("check_probe() {\n");
    script.push_str("  name=\"$1\"; kind=\"$2\"; shift 2\n");
    script.push_str("  rc=1\n");
    script.push_str("  if [ \"$kind\" = \"probe\" ]; then\n");
    script.push_str("    cmd=\"$1\"\n");
    script.push_str("    sh -c \"$cmd\" >/dev/null 2>&1\n");
    script.push_str("    rc=$?\n");
    script.push_str("  else\n");
    script.push_str("    rc=0\n");
    script.push_str("    for ep in \"$@\"; do\n");
    script.push_str("      resolved=\"$(command -v \"$ep\" 2>/dev/null || true)\"\n");
    script.push_str("      if [ \"$resolved\" != \"$world_deps_bin/$ep\" ]; then\n");
    script.push_str("        rc=1\n");
    script.push_str("      fi\n");
    script.push_str("    done\n");
    script.push_str("  fi\n");
    script.push_str("  if [ \"$rc\" -eq 0 ]; then\n");
    script.push_str("    printf '__SUBSTRATE_WDP2__ %s 1\\n' \"$name\"\n");
    script.push_str("  else\n");
    script.push_str("    printf '__SUBSTRATE_WDP2__ %s 0\\n' \"$name\"\n");
    script.push_str("  fi\n");
    script.push_str("}\n");

    for check in checks {
        match &check.check {
            PackageCheckKind::Probe { command } => {
                script.push_str("check_probe ");
                script.push_str(&sh_quote(&check.name));
                script.push_str(" probe ");
                script.push_str(&sh_quote(command));
                script.push('\n');
            }
            PackageCheckKind::Entrypoints { entrypoints } => {
                script.push_str("check_probe ");
                script.push_str(&sh_quote(&check.name));
                script.push_str(" entrypoints");
                for ep in entrypoints {
                    script.push(' ');
                    script.push_str(&sh_quote(ep));
                }
                script.push('\n');
            }
        }
    }
    script.push_str("exit 0\n");
    script
}

fn run_world_command_for_deps(
    cmd: &str,
    #[cfg(target_os = "linux")] context: &AuthenticatedWorldDepsContextV1,
) -> Result<transport_api_types::ExecuteResponse> {
    run_world_command_for_deps_at(
        cmd,
        None,
        None,
        #[cfg(target_os = "linux")]
        context,
    )
}

fn run_world_command_for_deps_at(
    cmd: &str,
    cwd_override: Option<&str>,
    profile_override: Option<&str>,
    #[cfg(target_os = "linux")] context: &AuthenticatedWorldDepsContextV1,
) -> Result<transport_api_types::ExecuteResponse> {
    #[cfg(target_os = "linux")]
    let (client, request, _) = build_authenticated_world_deps_client_and_request(
        context,
        cmd,
        cwd_override.map(Path::new),
        profile_override.unwrap_or("world-deps-probe"),
    )?;

    #[cfg(not(target_os = "linux"))]
    let (client, request, _) = {
        let (client, mut request, fs_enabled) = build_agent_client_and_request(cmd)?;
        // Runtime world-deps execution uses a dedicated internal profile so read-only probes and
        // world-deps wrapper installs can run on runners that block unprivileged user namespaces,
        // without borrowing the operator-facing provisioning profile.
        request.profile = Some("world-deps-probe".to_string());
        if let Some(profile) = profile_override {
            request.profile = Some(profile.to_string());
        }

        if let Some(cwd) = cwd_override {
            request.cwd = Some(cwd.to_string());
        } else if cfg!(target_os = "macos") {
            request.cwd = Some("/tmp".to_string());
        }

        if let Some(env) = request.env.as_mut() {
            ensure_world_deps_bin_on_path(env);
        }
        (client, request, fs_enabled)
    };

    let rt = Runtime::new()?;
    let response = rt.block_on(async move {
        client
            .execute(request)
            .await
            .context("world-service /v1/execute request failed")
    })?;
    Ok(response)
}

#[cfg(not(target_os = "linux"))]
fn ensure_world_deps_bin_on_path(env: &mut HashMap<String, String>) {
    const DEFAULT_WORLD_DEPS_BIN: &str = "/var/lib/substrate/world-deps/bin";
    let bin = env
        .get("SUBSTRATE_WORLD_DEPS_GUEST_BIN_DIR")
        .map(|v| v.trim().to_string())
        .filter(|v| !v.is_empty())
        .unwrap_or_else(|| DEFAULT_WORLD_DEPS_BIN.to_string());

    env.insert(
        "SUBSTRATE_WORLD_DEPS_GUEST_BIN_DIR".to_string(),
        bin.clone(),
    );

    let current = env.get("PATH").map(String::as_str).unwrap_or("");
    let bin_norm = bin.trim_end_matches('/');
    let has = current
        .split(':')
        .any(|segment| segment.trim_end_matches('/') == bin_norm);
    if has {
        return;
    }
    if current.trim().is_empty() {
        env.insert("PATH".to_string(), bin);
    } else {
        env.insert("PATH".to_string(), format!("{bin}:{current}"));
    }
}

fn classify_world_backend_error(err: anyhow::Error) -> anyhow::Error {
    if looks_like_world_backend_unavailable(&err) {
        return anyhow!(WorldDepsBackendUnavailableError::new(format!("{:#}", err)));
    }
    err
}

fn looks_like_world_backend_unavailable(err: &anyhow::Error) -> bool {
    let mut current: Option<&(dyn StdError + 'static)> = Some(err.as_ref());
    while let Some(err) = current {
        let message = err.to_string();
        if message.contains("world-service")
            || message.contains("platform world context")
            || message.contains("world backend")
            || message.contains("connect UDS")
            || message.contains("unix socket")
            || message.contains("Connection refused")
            || message.contains("connection refused")
            || message.contains("timed out")
            || message.contains("No such file or directory")
            || message.contains("SUN_LEN")
        {
            return true;
        }
        current = err.source();
    }
    false
}

fn sh_quote(value: &str) -> String {
    let escaped = value.replace('\'', "'\"'\"'");
    format!("'{escaped}'")
}

fn shell_escape_item_name(name: &str) -> String {
    let simple = !name.is_empty()
        && name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '-' | '.' | '+'));
    if simple {
        name.to_string()
    } else {
        sh_quote(name)
    }
}

fn resolve_global_available_inventory_view(
    #[cfg(unix)] context: &AuthenticatedWorldDepsContextV1,
) -> Result<InventoryViewV1> {
    let platform = HostPlatform::current();
    let mut view = builtin_inventory_v1(platform);
    #[cfg(unix)]
    let global_deps_dir = context.global_deps_dir().to_path_buf();
    #[cfg(not(unix))]
    let global_deps_dir = substrate_paths::substrate_home()?.join("deps");
    merge_inventory_layer_v1(
        &mut view,
        load_inventory_dir_v1(&global_deps_dir, platform)?,
    );
    view.validate_no_collisions()?;
    Ok(view)
}

fn workspace_marker_path(workspace_root: &Path) -> PathBuf {
    workspace_root.join(".substrate").join("workspace.yaml")
}

pub(crate) fn resolve_current_inventory_view(
    cwd: &std::path::Path,
    cfg: &config_model::SubstrateConfig,
    global_deps_dir: Option<&Path>,
) -> Result<InventoryViewV1> {
    let platform = HostPlatform::current();
    let mut view = InventoryViewV1::default();

    let include_builtins = cfg.world.deps.builtins == config_model::WorldDepsBuiltinsMode::Enabled;
    let inventory_mode = cfg.world.deps.inventory_mode;

    if inventory_mode == config_model::WorldDepsInventoryMode::Merged && include_builtins {
        merge_inventory_layer_v1(&mut view, builtin_inventory_v1(platform));
    }

    if inventory_mode == config_model::WorldDepsInventoryMode::Merged {
        let global_deps_dir = global_deps_dir.ok_or_else(|| {
            anyhow!("explicit global dependency inventory path is required in merged mode")
        })?;
        merge_inventory_layer_v1(&mut view, load_inventory_dir_v1(global_deps_dir, platform)?);
    }

    let workspace_root = crate::execution::find_workspace_root(cwd);
    for dir in find_workspace_inventory_chain(cwd, workspace_root.as_deref()) {
        merge_inventory_layer_v1(&mut view, load_inventory_dir_v1(&dir, platform)?);
    }

    view.validate_no_collisions()?;
    Ok(view)
}

fn print_config_patch(patch: &config_model::SubstrateConfigPatch, json: bool) -> Result<()> {
    if json {
        println!("{}", serde_json::to_string_pretty(patch)?);
    } else {
        println!("{}", serde_yaml::to_string(patch)?);
    }
    Ok(())
}

fn dedupe_ordered(items: &[String]) -> Vec<String> {
    let mut out: Vec<String> = Vec::with_capacity(items.len());
    for item in items {
        if !out.iter().any(|existing| existing == item) {
            out.push(item.clone());
        }
    }
    out
}

fn csv(items: &[String]) -> String {
    items.join(",")
}

fn read_comment_header_prefix(path: &Path) -> Result<String> {
    let raw =
        fs::read_to_string(path).with_context(|| format!("failed to read {}", path.display()))?;
    Ok(read_comment_header_prefix_from_raw(&raw))
}

fn read_comment_header_prefix_from_raw(raw: &str) -> String {
    let mut out = String::new();
    for line in raw.split_inclusive('\n') {
        let check = line.trim_end_matches('\n');
        let check = check.trim_start();
        if check.is_empty() || check.starts_with('#') {
            out.push_str(line);
            continue;
        }
        break;
    }
    out
}

fn write_atomic_patch_yaml(
    path: &Path,
    default_header: &str,
    existing_header: Option<&str>,
    patch: &config_model::SubstrateConfigPatch,
) -> Result<()> {
    let header = existing_header.unwrap_or(default_header);
    let mut body = serde_yaml::to_string(patch)
        .with_context(|| format!("failed to serialize {}", path.display()))?;
    if let Some(rest) = body.strip_prefix("---\n") {
        body = rest.to_string();
    }

    let mut out = String::new();
    out.push_str(header);
    if !out.is_empty() && !out.ends_with('\n') {
        out.push('\n');
    }
    out.push_str(&body);
    if !out.ends_with('\n') {
        out.push('\n');
    }

    let parent = path
        .parent()
        .ok_or_else(|| anyhow::anyhow!("path {} has no parent", path.display()))?;
    fs::create_dir_all(parent).with_context(|| format!("failed to create {}", parent.display()))?;
    let mut tmp = NamedTempFile::new_in(parent)
        .with_context(|| format!("failed to create temp file near {}", path.display()))?;
    tmp.write_all(out.as_bytes())
        .with_context(|| format!("failed to write {}", path.display()))?;
    tmp.flush()?;
    tmp.persist(path)
        .map_err(|err| anyhow::anyhow!("failed to persist {}: {}", path.display(), err.error))?;
    Ok(())
}

fn print_enabled_table(items: &[InventoryListItemSummaryV1]) {
    let mut kind_width = "Kind".len();
    for item in items {
        kind_width = kind_width.max(item.kind.len());
    }

    println!("{:<kind_width$} Name", "Kind", kind_width = kind_width);
    let kind_sep = "-".repeat(kind_width);
    println!("{kind_sep} ----");
    for item in items {
        println!(
            "{:<kind_width$} {}",
            item.kind,
            item.name,
            kind_width = kind_width
        );
    }
}

fn print_inventory_table(items: &[InventoryListItemSummaryV1]) {
    let mut kind_width = "Kind".len();
    let mut name_width = "Name".len();
    for item in items {
        kind_width = kind_width.max(item.kind.len());
        name_width = name_width.max(item.name.len());
    }

    println!(
        "{:<kind_width$} {:<name_width$} {:<8} {:<6} {:<12} {:<10} Description",
        "Kind",
        "Name",
        "Runnable",
        "Method",
        "Entrypoints",
        "Platforms",
        kind_width = kind_width,
        name_width = name_width
    );
    println!(
        "{:-<kind_width$} {:-<name_width$} {:-<8} {:-<6} {:-<12} {:-<10} {:-<11}",
        "",
        "",
        "",
        "",
        "",
        "",
        "",
        kind_width = kind_width,
        name_width = name_width
    );
    for item in items {
        let runnable = item
            .runnable
            .map(|v| if v { "true" } else { "false" })
            .unwrap_or("-");
        let method = item
            .method
            .as_ref()
            .map(|m| match m {
                super::inventory::InstallMethodV1::Apt => "apt",
                super::inventory::InstallMethodV1::Pacman => "pacman",
                super::inventory::InstallMethodV1::Script => "script",
                super::inventory::InstallMethodV1::Manual => "manual",
            })
            .unwrap_or("-");
        let entrypoints = if item.entrypoints.is_empty() {
            "-".to_string()
        } else {
            item.entrypoints.join(",")
        };
        let platforms = if item.platforms.is_empty() {
            "-".to_string()
        } else {
            item.platforms.join(",")
        };
        let desc = item.description.as_deref().unwrap_or("-");
        println!(
            "{:<kind_width$} {:<name_width$} {:<8} {:<6} {:<12} {:<10} {}",
            item.kind,
            item.name,
            runnable,
            method,
            entrypoints,
            platforms,
            desc,
            kind_width = kind_width,
            name_width = name_width
        );
    }
}
