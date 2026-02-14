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
    WorldDepsScopedMutateArgs, WorldDepsScopedResetArgs, WorldDepsWorkspaceAction,
    WorldDepsWorkspaceCmd,
};
use anyhow::{Context, Result};
use serde::Serialize;
use std::env;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use substrate_common::paths as substrate_paths;
use tempfile::NamedTempFile;

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
        WorldDepsGlobalAction::Add(args) => run_global_add(args),
        WorldDepsGlobalAction::Remove(args) => run_global_remove(args),
        WorldDepsGlobalAction::Reset(args) => run_global_reset(args),
    }
}

pub(crate) fn run_workspace(cmd: &WorldDepsWorkspaceCmd) -> Result<()> {
    match &cmd.action {
        WorldDepsWorkspaceAction::List(args) => run_workspace_list(args),
        WorldDepsWorkspaceAction::Add(args) => run_workspace_add(args),
        WorldDepsWorkspaceAction::Remove(args) => run_workspace_remove(args),
        WorldDepsWorkspaceAction::Reset(args) => run_workspace_reset(args),
    }
}

fn run_current_list(args: &WorldDepsCurrentListArgs) -> Result<()> {
    if args.all && args.view != WorldDepsCurrentListViewArg::Applied {
        return Err(config_model::user_error(
            "--all is only valid for `substrate world deps current list applied`",
        ));
    }
    let cwd = env::current_dir().unwrap_or_else(|_| ".".into());
    let cfg = config_model::resolve_effective_config(&cwd, &Default::default())
        .context("failed to resolve effective config")?;

    match args.view {
        WorldDepsCurrentListViewArg::Available => {
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
        WorldDepsCurrentListViewArg::Enabled => run_current_list_enabled(&cwd, &cfg, args.json),
        WorldDepsCurrentListViewArg::Applied => Err(config_model::user_error(format!(
            "`substrate world deps current list {}` is not implemented in this slice",
            format!("{:?}", args.view).to_ascii_lowercase()
        ))),
    }
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
    match args.view {
        WorldDepsScopedListViewArg::Available => {
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
        WorldDepsScopedListViewArg::Enabled => {
            let (patch, _) = config_model::read_global_config_patch_or_empty()?;
            print_config_patch(&patch, args.json)
        }
    }
}

fn run_workspace_list(args: &WorldDepsScopedListArgs) -> Result<()> {
    let cwd = env::current_dir().unwrap_or_else(|_| ".".into());
    let workspace_root = crate::execution::find_workspace_root(&cwd)
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

fn run_global_add(args: &WorldDepsScopedMutateArgs) -> Result<()> {
    let items = dedupe_ordered(&args.item_names);
    let view = resolve_global_available_inventory_view()?;

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

    let path = config_model::global_config_path()?;
    let (mut patch, existed) = config_model::read_global_config_patch_or_empty()
        .with_context(|| format!("failed to load global config patch at {}", path.display()))?;

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
    eprintln!(
        "substrate: note: enabled deps changes apply to the world only after 'substrate world deps current sync'"
    );
    Ok(())
}

fn run_global_remove(args: &WorldDepsScopedMutateArgs) -> Result<()> {
    let items = dedupe_ordered(&args.item_names);
    let cwd = env::current_dir().unwrap_or_else(|_| ".".into());

    let path = config_model::global_config_path()?;
    let (mut patch, existed) = config_model::read_global_config_patch_or_empty()
        .with_context(|| format!("failed to load global config patch at {}", path.display()))?;

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
    eprintln!("substrate: note: 'remove' only updates enabled deps; it does not uninstall. Run 'substrate world deps current sync' to apply");

    if !removed.is_empty() {
        if let Some(workspace_root) = crate::execution::find_workspace_root(&cwd) {
            let ws_patch_path = workspace_marker_path(&workspace_root);
            let raw = fs::read_to_string(&ws_patch_path)
                .with_context(|| format!("failed to read {}", ws_patch_path.display()))?;
            let ws_patch = config_model::parse_config_patch_yaml(&ws_patch_path, &raw)?;
            let ws_enabled = ws_patch.world.deps.enabled.unwrap_or_default();
            for item in removed {
                if ws_enabled.iter().any(|name| name == &item) {
                    eprintln!("substrate: note: '{item}' was removed from global enabled deps but is still enabled via workspace; run 'substrate world deps workspace remove {item}' to fully disable it for this workspace");
                }
            }
        }
    }

    Ok(())
}

fn run_global_reset(args: &WorldDepsScopedResetArgs) -> Result<()> {
    let path = config_model::global_config_path()?;
    let (mut patch, existed) = config_model::read_global_config_patch_or_empty()
        .with_context(|| format!("failed to load global config patch at {}", path.display()))?;

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
    eprintln!(
        "substrate: note: run 'substrate world deps current sync' to apply enabled deps changes"
    );
    Ok(())
}

fn run_workspace_add(args: &WorldDepsScopedMutateArgs) -> Result<()> {
    let cwd = env::current_dir().unwrap_or_else(|_| ".".into());
    let workspace_root = crate::execution::find_workspace_root(&cwd)
        .ok_or_else(|| config_model::user_error("no workspace root detected for this directory"))?;

    let items = dedupe_ordered(&args.item_names);
    let cfg = config_model::resolve_effective_config(&cwd, &Default::default())
        .context("failed to resolve effective config")?;
    let view = resolve_current_inventory_view(&cwd, &cfg)?;

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
    eprintln!(
        "substrate: note: enabled deps changes apply to the world only after 'substrate world deps current sync'"
    );
    Ok(())
}

fn run_workspace_remove(args: &WorldDepsScopedMutateArgs) -> Result<()> {
    let cwd = env::current_dir().unwrap_or_else(|_| ".".into());
    let workspace_root = crate::execution::find_workspace_root(&cwd)
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
    eprintln!("substrate: note: 'remove' only updates enabled deps; it does not uninstall. Run 'substrate world deps current sync' to apply");

    if !removed.is_empty() {
        let (global_patch, _) = config_model::read_global_config_patch_or_empty()?;
        let global_enabled = global_patch.world.deps.enabled.unwrap_or_default();
        for item in removed {
            if global_enabled.iter().any(|name| name == &item) {
                eprintln!("substrate: note: '{item}' was removed from workspace enabled deps but is still enabled via global; run 'substrate world deps global remove {item}' to fully disable it");
            }
        }
    }

    Ok(())
}

fn run_workspace_reset(args: &WorldDepsScopedResetArgs) -> Result<()> {
    let cwd = env::current_dir().unwrap_or_else(|_| ".".into());
    let workspace_root = crate::execution::find_workspace_root(&cwd)
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
    eprintln!(
        "substrate: note: run 'substrate world deps current sync' to apply enabled deps changes"
    );
    Ok(())
}

fn run_current_list_enabled(
    cwd: &std::path::Path,
    cfg: &config_model::SubstrateConfig,
    json: bool,
) -> Result<()> {
    eprintln!("substrate: note: showing current effective enabled deps list for this directory");

    let enabled = &cfg.world.deps.enabled;
    if enabled.is_empty() {
        eprintln!("substrate: hint: add deps with 'substrate world deps workspace add ...' (or '... global add ...') then apply with 'substrate world deps current sync'");
    }

    let view = resolve_current_inventory_view(cwd, cfg)?;
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
        runnable: None,
        method: None,
        entrypoints: Vec::new(),
        platforms: Vec::new(),
        description: None,
    }
}

fn resolve_global_available_inventory_view() -> Result<InventoryViewV1> {
    let platform = HostPlatform::current();
    let mut view = builtin_inventory_v1(platform);
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
    let mut name_width = "Name".len();
    for item in items {
        kind_width = kind_width.max(item.kind.len());
        name_width = name_width.max(item.name.len());
    }

    println!(
        "{:<kind_width$} {:<name_width$}",
        "Kind",
        "Name",
        kind_width = kind_width,
        name_width = name_width
    );
    println!(
        "{:-<kind_width$} {:-<name_width$}",
        "",
        "",
        kind_width = kind_width,
        name_width = name_width
    );
    for item in items {
        println!(
            "{:<kind_width$} {:<name_width$}",
            item.kind,
            item.name,
            kind_width = kind_width,
            name_width = name_width
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
