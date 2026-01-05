use crate::execution::cli::{
    AnchorModeArg, Cli, ConfigAction, ConfigCmd, ConfigGlobalAction, ConfigGlobalCmd,
    ConfigInitArgs, ConfigSetArgs, ConfigShowArgs,
};
use crate::execution::config_model::{self, CliConfigOverrides, SubstrateConfig};
use crate::execution::write_env_sh;
use anyhow::{anyhow, Context, Result};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use tempfile::NamedTempFile;

pub(crate) fn handle_config_command(cmd: &ConfigCmd, cli: &Cli) -> i32 {
    let result = match &cmd.action {
        ConfigAction::Show(args) => run_workspace_show(args, cli),
        ConfigAction::Set(args) => run_workspace_set(args, cli),
        ConfigAction::Global(cmd) => run_global(cmd),
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

    let cfg = SubstrateConfig::default();
    write_atomic_yaml(&path, &cfg)
        .with_context(|| format!("failed to write {}", path.display()))?;
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
    let path = config_model::global_config_path()?;
    let (cfg, _) = config_model::read_global_config_or_defaults()
        .with_context(|| format!("failed to load global config at {}", path.display()))?;
    print_config(&cfg, args.json)?;
    Ok(())
}

fn run_global_set(args: &ConfigSetArgs) -> Result<()> {
    let path = config_model::global_config_path()?;
    let (mut cfg, existed) = config_model::read_global_config_or_defaults()
        .with_context(|| format!("failed to load global config at {}", path.display()))?;

    let updates = config_model::parse_updates(&args.updates)?;
    let changed = config_model::apply_updates(&mut cfg, &updates)?;

    if changed || !existed {
        write_atomic_yaml(&path, &cfg)
            .with_context(|| format!("failed to write {}", path.display()))?;
    }

    write_env_sh(&cfg).context("failed to write env.sh")?;
    print_config(&cfg, args.json)?;
    Ok(())
}

fn run_workspace_show(args: &ConfigShowArgs, cli: &Cli) -> Result<()> {
    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let overrides = cli_overrides(cli);
    let cfg = config_model::resolve_effective_config(&cwd, &overrides)?;
    print_config(&cfg, args.json)?;
    Ok(())
}

fn run_workspace_set(args: &ConfigSetArgs, cli: &Cli) -> Result<()> {
    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let workspace_root = require_workspace(&cwd)?;

    let path = crate::execution::workspace::workspace_marker_path(&workspace_root);
    let raw =
        fs::read_to_string(&path).with_context(|| format!("failed to read {}", path.display()))?;
    let mut cfg = config_model::parse_config_yaml(&path, &raw)?;

    let updates = config_model::parse_updates(&args.updates)?;
    let changed = config_model::apply_updates(&mut cfg, &updates)?;
    if changed {
        write_atomic_yaml(&path, &cfg)
            .with_context(|| format!("failed to write {}", path.display()))?;
    }

    let overrides = cli_overrides(cli);
    let effective = config_model::resolve_effective_config(&cwd, &overrides)?;
    print_config(&effective, args.json)?;
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
