use super::guest::WorldBackendUnavailable;
use super::inventory::{
    builtin_inventory_v1, find_workspace_inventory_chain, load_inventory_dir_v1,
    merge_inventory_layer_v1, summarize_inventory_v1, HostPlatform, InstallMethodV1,
    InventoryItemDefV1, InventoryListItemSummaryV1, InventoryViewV1, WrapperDefV1, WrapperKindV1,
};
use crate::execution::build_agent_client_and_request;
use crate::execution::config_model;
use crate::execution::{
    WorldDepsCurrentAction, WorldDepsCurrentCmd, WorldDepsCurrentListArgs,
    WorldDepsCurrentListViewArg, WorldDepsCurrentShowArgs, WorldDepsGlobalAction,
    WorldDepsGlobalCmd, WorldDepsScopedListArgs, WorldDepsScopedListViewArg,
    WorldDepsScopedMutateArgs, WorldDepsScopedResetArgs, WorldDepsWorkspaceAction,
    WorldDepsWorkspaceCmd,
};
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
        WorldDepsCurrentListViewArg::Applied => {
            run_current_list_applied(&cwd, &cfg, args.all, args.json)
        }
    }
}

fn run_current_show(args: &WorldDepsCurrentShowArgs) -> Result<()> {
    let cwd = env::current_dir().unwrap_or_else(|_| ".".into());
    let cfg = config_model::resolve_effective_config(&cwd, &Default::default())
        .context("failed to resolve effective config")?;
    let view = resolve_current_inventory_view(&cwd, &cfg)?;
    let item = view.get(&args.item_name).ok_or_else(|| {
        config_model::user_error(format!("unknown deps item '{}'", args.item_name))
    })?;

    if args.explain {
        let explain = build_current_show_explain_v1(&cwd, &cfg, &view, &args.item_name, &item)?;
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
) -> Result<CurrentShowExplainV1> {
    let enabled = cfg.world.deps.enabled.iter().any(|name| name == item_name);
    let enabled_via_global_patch = {
        let (patch, _) = config_model::read_global_config_patch_or_empty()?;
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
    let package_statuses = query_world_package_presence(view, &packages_to_check)?;
    let (world, _remediation) =
        compute_world_status_and_remediation(item_name, item, view, &package_statuses, enabled)?;

    let wrappers = wrappers_explain(item);

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
        (None, None)
    };

    Ok(CurrentShowExplainV1 {
        schema_version: 1,
        item_name: item_name.to_string(),
        enabled,
        enabled_via_global_patch,
        enabled_via_workspace_patch,
        world: world.as_str().to_string(),
        remediation: None,
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
    println!(
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
    println!("substrate: note: 'remove' only updates enabled deps; it does not uninstall. Run 'substrate world deps current sync' to apply");

    if !removed.is_empty() {
        if let Some(workspace_root) = crate::execution::find_workspace_root(&cwd) {
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
    println!(
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
    println!(
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
    println!("substrate: note: 'remove' only updates enabled deps; it does not uninstall. Run 'substrate world deps current sync' to apply");

    if !removed.is_empty() {
        let (global_patch, _) = config_model::read_global_config_patch_or_empty()?;
        let global_enabled = global_patch.world.deps.enabled.unwrap_or_default();
        for item in removed {
            if global_enabled.iter().any(|name| name == &item) {
                println!("substrate: note: '{item}' was removed from workspace enabled deps but is still enabled via global; run 'substrate world deps global remove {item}' to fully disable it");
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
    println!(
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
    all: bool,
    json: bool,
) -> Result<()> {
    eprintln!("substrate: note: showing current world deps status for this directory");

    let view = resolve_current_inventory_view(cwd, cfg)?;
    let enabled = &cfg.world.deps.enabled;

    let enabled_set: HashSet<&str> = enabled.iter().map(|s| s.as_str()).collect();

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
        enabled.clone()
    };

    let mut packages_to_check: Vec<String> = Vec::new();
    if all {
        packages_to_check.extend(view.packages.keys().cloned());
    } else {
        for name in enabled {
            if let Some(item) = view.get(name) {
                collect_required_package_names(name, &item, &view, &mut packages_to_check);
            }
        }
    }
    packages_to_check.sort();
    packages_to_check.dedup();

    let package_presence = query_world_package_presence(&view, &packages_to_check)?;

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
            &view,
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

fn print_applied_table(items: &[InventoryListItemSummaryV1]) {
    let mut kind_width = "Kind".len();
    let mut name_width = "Name".len();
    for item in items {
        kind_width = kind_width.max(item.kind.len());
        name_width = name_width.max(item.name.len());
    }

    println!(
        "{:<kind_width$} {:<name_width$} {:<7} {:<7} Remediation",
        "Kind",
        "Name",
        "Enabled",
        "World",
        kind_width = kind_width,
        name_width = name_width
    );
    println!(
        "{:-<kind_width$} {:-<name_width$} {:-<7} {:-<7} {:-<11}",
        "",
        "",
        "",
        "",
        "",
        kind_width = kind_width,
        name_width = name_width
    );
    for item in items {
        let enabled = item
            .enabled
            .map(|v| if v { "true" } else { "false" })
            .unwrap_or("-");
        let world = item.world.as_deref().unwrap_or("-");
        let remediation = item.remediation.as_deref().unwrap_or("-");
        println!(
            "{:<kind_width$} {:<name_width$} {:<7} {:<7} {}",
            item.kind,
            item.name,
            enabled,
            world,
            remediation,
            kind_width = kind_width,
            name_width = name_width
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
        ensure_world_backend_available()?;
        return Ok(HashMap::new());
    }

    let script = build_world_probe_script(&checks);
    let response = run_world_command_for_deps(&script).map_err(classify_world_backend_error)?;

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
        return Err(anyhow!(WorldBackendUnavailable::new(format!(
            "world probe script failed (exit={}): {}",
            response.exit, snippet
        ))));
    }

    let stdout_text = String::from_utf8_lossy(&stdout);
    let mut out: HashMap<String, bool> = HashMap::new();
    for line in stdout_text.lines() {
        let Some(rest) = line.strip_prefix("__SUBSTRATE_WDP2__ ") else {
            continue;
        };
        let mut parts = rest.split_whitespace();
        let Some(name) = parts.next() else { continue };
        let val = parts.next();
        let present = matches!(val, Some("1"));
        out.insert(name.to_string(), present);
    }
    Ok(out)
}

fn ensure_world_backend_available() -> Result<()> {
    let response = run_world_command_for_deps(":").map_err(classify_world_backend_error)?;
    if response.exit == 0 {
        return Ok(());
    }
    let stderr = BASE64
        .decode(response.stderr_b64.as_bytes())
        .unwrap_or_default();
    let stderr_text = String::from_utf8_lossy(&stderr);
    Err(anyhow!(WorldBackendUnavailable::new(format!(
        "world backend probe failed (exit={}): {}",
        response.exit,
        stderr_text.trim()
    ))))
}

fn build_world_probe_script(checks: &[PackageWorldCheck]) -> String {
    let mut script = String::new();
    script.push_str("set +e\n");
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
    script.push_str("      command -v \"$ep\" >/dev/null 2>&1 || rc=1\n");
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

fn run_world_command_for_deps(cmd: &str) -> Result<agent_api_types::ExecuteResponse> {
    let (client, mut request, _) = build_agent_client_and_request(cmd)?;
    if cfg!(target_os = "macos") {
        request.cwd = Some("/tmp".to_string());
    }
    let rt = Runtime::new()?;
    let response = rt.block_on(async move {
        client
            .execute(request)
            .await
            .context("world-agent /v1/execute request failed")
    })?;
    Ok(response)
}

fn classify_world_backend_error(err: anyhow::Error) -> anyhow::Error {
    if looks_like_world_backend_unavailable(&err) {
        return anyhow!(WorldBackendUnavailable::new(format!("{:#}", err)));
    }
    err
}

fn looks_like_world_backend_unavailable(err: &anyhow::Error) -> bool {
    let mut current: Option<&(dyn StdError + 'static)> = Some(err.as_ref());
    while let Some(err) = current {
        let message = err.to_string();
        if message.contains("world-agent")
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
