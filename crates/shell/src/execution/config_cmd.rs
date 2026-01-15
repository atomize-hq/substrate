use crate::execution::cli::{
    AnchorModeArg, Cli, ConfigAction, ConfigCmd, ConfigCurrentAction, ConfigGlobalAction,
    ConfigGlobalCmd, ConfigInitArgs, ConfigResetArgs, ConfigSetArgs, ConfigShowArgs,
    ConfigWorkspaceAction,
};
use crate::execution::config_model::{self, CliConfigOverrides, ConfigExplainV1, SubstrateConfig};
use crate::execution::write_env_sh;
use anyhow::{anyhow, Context, Result};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use tempfile::NamedTempFile;

pub(crate) fn handle_config_command(cmd: &ConfigCmd, cli: &Cli) -> i32 {
    let result = match &cmd.action {
        ConfigAction::Current(cmd) => match &cmd.action {
            ConfigCurrentAction::Show(args) => run_current_show(args, cli),
        },
        ConfigAction::Show(args) => run_current_show(args, cli),
        ConfigAction::Set(args) => run_workspace_set(args, cli),
        ConfigAction::Global(cmd) => run_global(cmd),
        ConfigAction::Workspace(cmd) => match &cmd.action {
            ConfigWorkspaceAction::Set(args) => run_workspace_set(args, cli),
            ConfigWorkspaceAction::Reset(args) => run_workspace_reset(args, cli),
        },
    };

    match result {
        Ok(()) => 0,
        Err(err) if err.is::<ActionableError>() => {
            eprintln!("{:#}", err);
            2
        }
        Err(err) if config_model::is_user_error(&err) => {
            eprintln!("{err}");
            2
        }
        Err(err) => {
            eprintln!("{:#}", err);
            1
        }
    }
}

#[derive(Debug)]
struct ActionableError(String);

impl std::fmt::Display for ActionableError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for ActionableError {}

fn actionable(message: impl Into<String>) -> anyhow::Error {
    anyhow::Error::new(ActionableError(message.into()))
}

fn run_global(cmd: &ConfigGlobalCmd) -> Result<()> {
    match &cmd.action {
        ConfigGlobalAction::Init(args) => run_global_init(args),
        ConfigGlobalAction::Show(args) => run_global_show(args),
        ConfigGlobalAction::Set(args) => run_global_set(args),
    }
}

fn run_global_init(args: &ConfigInitArgs) -> Result<()> {
    let path = config_model::global_config_path()?;
    let existed = path.exists();

    if existed && !args.force {
        println!(
            "substrate: config already exists at {}; use --force to overwrite",
            path.display()
        );
        return Ok(());
    }

    let patch_yaml = config_model::default_config_patch_yaml();
    write_atomic_bytes(&path, patch_yaml.as_bytes())
        .with_context(|| format!("failed to write {}", path.display()))?;
    let cfg = config_model::resolve_global_effective_config()?;
    write_env_sh(&cfg).context("failed to write env.sh")?;
    if existed {
        println!(
            "substrate: overwrote config at {} (--force)",
            path.display()
        );
    } else {
        println!("substrate: wrote default config to {}", path.display());
    }
    Ok(())
}

fn run_global_show(args: &ConfigShowArgs) -> Result<()> {
    let (cfg, explain) = config_model::resolve_global_effective_config_with_explain(args.explain)?;
    if let Some(explain) = explain {
        print_explain(&explain)?;
    }
    print_config(&cfg, args.json)?;
    Ok(())
}

fn run_global_set(args: &ConfigSetArgs) -> Result<()> {
    let path = config_model::global_config_path()?;
    let (mut patch, existed) = config_model::read_global_config_patch_or_empty()
        .with_context(|| format!("failed to load global config patch at {}", path.display()))?;

    let updates = config_model::parse_updates(&args.updates)?;
    let changed = config_model::apply_updates_to_patch(&mut patch, &updates)?;

    if changed || !existed {
        write_atomic_yaml(&path, &patch)
            .with_context(|| format!("failed to write {}", path.display()))?;
    }

    let cfg = config_model::resolve_global_effective_config()?;
    write_env_sh(&cfg).context("failed to write env.sh")?;
    print_config(&cfg, args.json)?;
    Ok(())
}

fn run_current_show(args: &ConfigShowArgs, cli: &Cli) -> Result<()> {
    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let overrides = cli_overrides(cli);
    let (cfg, explain) =
        config_model::resolve_effective_config_with_explain(&cwd, &overrides, args.explain)?;
    if let Some(explain) = explain {
        print_explain(&explain)?;
    }
    print_config(&cfg, args.json)?;
    Ok(())
}

fn run_workspace_set(args: &ConfigSetArgs, cli: &Cli) -> Result<()> {
    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let workspace_root = require_workspace(&cwd)?;

    let path = crate::execution::workspace::workspace_marker_path(&workspace_root);
    let raw =
        fs::read_to_string(&path).with_context(|| format!("failed to read {}", path.display()))?;
    let mut patch = config_model::parse_config_patch_yaml(&path, &raw)?;

    let updates = config_model::parse_updates(&args.updates)?;
    let changed = config_model::apply_updates_to_patch(&mut patch, &updates)?;
    if changed {
        write_atomic_yaml(&path, &patch)
            .with_context(|| format!("failed to write {}", path.display()))?;
    }

    let overrides = cli_overrides(cli);
    let (effective, _) =
        config_model::resolve_effective_config_with_explain(&cwd, &overrides, false)?;
    print_config(&effective, args.json)?;
    Ok(())
}

fn run_workspace_reset(args: &ConfigResetArgs, cli: &Cli) -> Result<()> {
    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let workspace_root = require_workspace(&cwd)?;
    let path = crate::execution::workspace::workspace_marker_path(&workspace_root);
    let raw =
        fs::read_to_string(&path).with_context(|| format!("failed to read {}", path.display()))?;
    let mut patch = config_model::parse_config_patch_yaml(&path, &raw)?;

    let changed = config_model::reset_patch_keys(&mut patch, &args.keys)?;
    if changed {
        write_atomic_yaml(&path, &patch)
            .with_context(|| format!("failed to write {}", path.display()))?;
    }

    let overrides = cli_overrides(cli);
    let (effective, _) =
        config_model::resolve_effective_config_with_explain(&cwd, &overrides, false)?;
    print_config(&effective, false)?;
    Ok(())
}

fn require_workspace(cwd: &Path) -> Result<PathBuf> {
    crate::execution::workspace::find_workspace_root(cwd).ok_or_else(|| {
        actionable("substrate: not in a workspace; run `substrate workspace init`".to_string())
    })
}

fn print_config(cfg: &SubstrateConfig, json: bool) -> Result<()> {
    if json {
        println!(
            "{}",
            serde_json::to_string_pretty(cfg).context("failed to serialize JSON")?
        );
        return Ok(());
    }
    println!(
        "{}",
        serde_yaml::to_string(cfg).context("failed to serialize YAML")?
    );
    Ok(())
}

fn print_explain(explain: &ConfigExplainV1) -> Result<()> {
    // Emit a stable layer-order hint before the JSON so simple substring checks can
    // validate ordering without depending on map key ordering.
    let mut has_global_patch = false;
    let mut has_workspace_patch = false;
    for v in explain.keys.values() {
        for s in &v.sources {
            match s.layer.as_str() {
                "global_patch" => has_global_patch = true,
                "workspace_patch" => has_workspace_patch = true,
                _ => {}
            }
        }
    }
    if has_global_patch {
        eprintln!("global_patch");
    }
    if has_workspace_patch {
        eprintln!("workspace_patch");
    }

    eprintln!(
        "{}",
        serde_json::to_string_pretty(explain).context("failed to serialize explain JSON")?
    );
    Ok(())
}

fn write_atomic_yaml<T: serde::Serialize>(path: &Path, value: &T) -> Result<()> {
    let parent = path
        .parent()
        .ok_or_else(|| anyhow!("path {} has no parent", path.display()))?;
    fs::create_dir_all(parent).with_context(|| format!("failed to create {}", parent.display()))?;
    let mut tmp = NamedTempFile::new_in(parent)
        .with_context(|| format!("failed to create temp file near {}", path.display()))?;
    let body = serde_yaml::to_string(value)
        .with_context(|| format!("failed to serialize {}", path.display()))?;
    tmp.write_all(body.as_bytes())
        .with_context(|| format!("failed to write {}", path.display()))?;
    tmp.flush()?;
    tmp.persist(path)
        .map_err(|err| anyhow!("failed to persist {}: {}", path.display(), err.error))?;
    Ok(())
}

fn write_atomic_bytes(path: &Path, bytes: &[u8]) -> Result<()> {
    let parent = path
        .parent()
        .ok_or_else(|| anyhow!("path {} has no parent", path.display()))?;
    fs::create_dir_all(parent).with_context(|| format!("failed to create {}", parent.display()))?;
    let mut tmp = NamedTempFile::new_in(parent)
        .with_context(|| format!("failed to create temp file near {}", path.display()))?;
    tmp.write_all(bytes)
        .with_context(|| format!("failed to write {}", path.display()))?;
    tmp.flush()?;
    tmp.persist(path)
        .map_err(|err| anyhow!("failed to persist {}: {}", path.display(), err.error))?;
    Ok(())
}

fn cli_overrides(cli: &Cli) -> CliConfigOverrides {
    let mut overrides = CliConfigOverrides::default();

    if cli.world {
        overrides.world_enabled = Some(true);
    } else if cli.no_world {
        overrides.world_enabled = Some(false);
    }

    if let Some(mode) = cli.anchor_mode {
        overrides.anchor_mode = Some(match mode {
            AnchorModeArg::Workspace => substrate_common::WorldRootMode::Project,
            AnchorModeArg::FollowCwd => substrate_common::WorldRootMode::FollowCwd,
            AnchorModeArg::Custom => substrate_common::WorldRootMode::Custom,
        });
    }

    if let Some(path) = &cli.anchor_path {
        overrides.anchor_path = Some(path.to_string_lossy().to_string());
    }

    if cli.caged {
        overrides.caged = Some(true);
    } else if cli.uncaged {
        overrides.caged = Some(false);
    }

    overrides
}
