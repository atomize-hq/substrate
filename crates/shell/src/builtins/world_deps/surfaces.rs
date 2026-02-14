use super::inventory::{
    builtin_inventory_v1, find_workspace_inventory_chain, load_inventory_dir_v1,
    merge_inventory_layer_v1, summarize_inventory_v1, HostPlatform, InventoryItemDefV1,
    InventoryListItemSummaryV1, InventoryViewV1,
};
use crate::execution::config_model;
use crate::execution::{
    WorldDepsCurrentAction, WorldDepsCurrentCmd, WorldDepsCurrentListArgs,
    WorldDepsCurrentListViewArg, WorldDepsCurrentShowArgs, WorldDepsGlobalAction,
    WorldDepsGlobalCmd, WorldDepsScopedListArgs, WorldDepsScopedListViewArg,
    WorldDepsWorkspaceAction, WorldDepsWorkspaceCmd,
};
use anyhow::{Context, Result};
use serde::Serialize;
use std::env;
use substrate_common::paths as substrate_paths;

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
    item: InventoryItemDefV1,
}

pub(crate) fn run_current(cmd: &WorldDepsCurrentCmd) -> Result<()> {
    match &cmd.action {
        WorldDepsCurrentAction::List(args) => run_current_list(args),
        WorldDepsCurrentAction::Show(args) => run_current_show(args),
    }
}

pub(crate) fn run_global(cmd: &WorldDepsGlobalCmd) -> Result<()> {
    match &cmd.action {
        WorldDepsGlobalAction::List(args) => run_global_list(args),
    }
}

pub(crate) fn run_workspace(cmd: &WorldDepsWorkspaceCmd) -> Result<()> {
    match &cmd.action {
        WorldDepsWorkspaceAction::List(args) => run_workspace_list(args),
    }
}

fn run_current_list(args: &WorldDepsCurrentListArgs) -> Result<()> {
    if args.all && args.view != WorldDepsCurrentListViewArg::Applied {
        return Err(config_model::user_error(
            "--all is only valid for `substrate world deps current list applied`",
        ));
    }
    if args.view != WorldDepsCurrentListViewArg::Available {
        return Err(config_model::user_error(format!(
            "`substrate world deps current list {}` is not implemented in this slice",
            format!("{:?}", args.view).to_ascii_lowercase()
        )));
    }

    let cwd = env::current_dir().unwrap_or_else(|_| ".".into());
    let cfg = config_model::resolve_effective_config(&cwd, &Default::default())
        .context("failed to resolve effective config")?;

    let view = resolve_current_inventory_view(&cwd, &cfg)?;
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

fn run_current_show(args: &WorldDepsCurrentShowArgs) -> Result<()> {
    if args.explain {
        return Err(config_model::user_error(
            "`--explain` is not implemented for `current show` in this slice",
        ));
    }
    let cwd = env::current_dir().unwrap_or_else(|_| ".".into());
    let cfg = config_model::resolve_effective_config(&cwd, &Default::default())
        .context("failed to resolve effective config")?;
    let view = resolve_current_inventory_view(&cwd, &cfg)?;
    let item = view.get(&args.item_name).ok_or_else(|| {
        config_model::user_error(format!("unknown deps item '{}'", args.item_name))
    })?;

    if args.json {
        let out = ShowOutputV1 {
            schema_version: 1,
            scope: "current".to_string(),
            item,
        };
        println!("{}", serde_json::to_string_pretty(&out)?);
    } else {
        println!("{}", serde_yaml::to_string(&item)?);
    }
    Ok(())
}

fn run_global_list(args: &WorldDepsScopedListArgs) -> Result<()> {
    if args.view != WorldDepsScopedListViewArg::Available {
        return Err(config_model::user_error(format!(
            "`substrate world deps global list {}` is not implemented in this slice",
            format!("{:?}", args.view).to_ascii_lowercase()
        )));
    }
    let platform = HostPlatform::current();
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

fn run_workspace_list(args: &WorldDepsScopedListArgs) -> Result<()> {
    if args.view != WorldDepsScopedListViewArg::Available {
        return Err(config_model::user_error(format!(
            "`substrate world deps workspace list {}` is not implemented in this slice",
            format!("{:?}", args.view).to_ascii_lowercase()
        )));
    }
    let cwd = env::current_dir().unwrap_or_else(|_| ".".into());
    let workspace_root = crate::execution::find_workspace_root(&cwd)
        .ok_or_else(|| config_model::user_error("no workspace root detected for this directory"))?;
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

fn resolve_current_inventory_view(
    cwd: &std::path::Path,
    cfg: &config_model::SubstrateConfig,
) -> Result<InventoryViewV1> {
    let platform = HostPlatform::current();
    let mut view = InventoryViewV1::default();

    let include_builtins = cfg.world.deps.builtins == config_model::WorldDepsBuiltinsMode::Enabled;
    let inventory_mode = cfg.world.deps.inventory_mode;

    if inventory_mode == config_model::WorldDepsInventoryMode::Merged && include_builtins {
        merge_inventory_layer_v1(&mut view, builtin_inventory_v1(platform));
    }

    if inventory_mode == config_model::WorldDepsInventoryMode::Merged {
        let global_deps_dir = substrate_paths::substrate_home()?.join("deps");
        merge_inventory_layer_v1(
            &mut view,
            load_inventory_dir_v1(&global_deps_dir, platform)?,
        );
    }

    let workspace_root = crate::execution::find_workspace_root(cwd);
    for dir in find_workspace_inventory_chain(cwd, workspace_root.as_deref()) {
        merge_inventory_layer_v1(&mut view, load_inventory_dir_v1(&dir, platform)?);
    }

    view.validate_no_collisions()?;
    Ok(view)
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
