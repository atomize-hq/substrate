use crate::execution::cli::{
    Cli, PolicyAction, PolicyCmd, PolicyGlobalAction, PolicyGlobalCmd, PolicyInitArgs,
    PolicySetArgs, PolicyShowArgs,
};
use crate::execution::config_model;
use crate::execution::config_model::default_policy_yaml;
use crate::execution::policy_model;
use anyhow::{anyhow, Context, Result};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use tempfile::NamedTempFile;

pub(crate) fn handle_policy_command(cmd: &PolicyCmd, _cli: &Cli) -> i32 {
    let result = match &cmd.action {
        PolicyAction::Init(args) => run_workspace_init(args),
        PolicyAction::Show(args) => run_workspace_show(args),
        PolicyAction::Set(args) => run_workspace_set(args),
        PolicyAction::Global(cmd) => run_global(cmd),
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

fn run_global(cmd: &PolicyGlobalCmd) -> Result<()> {
    match &cmd.action {
        PolicyGlobalAction::Init(args) => run_global_init(args),
        PolicyGlobalAction::Show(args) => run_global_show(args),
        PolicyGlobalAction::Set(args) => run_global_set(args),
    }
}

fn run_global_init(args: &PolicyInitArgs) -> Result<()> {
    let path = policy_model::global_policy_path()?;
    let existed = path.exists();

    if existed && !args.force {
        println!(
            "substrate: policy already exists at {}; use --force to overwrite",
            path.display()
        );
        return Ok(());
    }

    write_atomic_bytes(&path, default_policy_yaml().as_bytes())
        .with_context(|| format!("failed to write {}", path.display()))?;
    if existed {
        println!(
            "substrate: overwrote policy at {} (--force)",
            path.display()
        );
    } else {
        println!("substrate: wrote default policy to {}", path.display());
    }
    Ok(())
}

fn run_global_show(args: &PolicyShowArgs) -> Result<()> {
    let (policy, _) = policy_model::load_global_policy_or_defaults()?;
    print_policy(&policy, args.json)?;
    Ok(())
}

fn run_global_set(args: &PolicySetArgs) -> Result<()> {
    let path = policy_model::global_policy_path()?;
    let (mut policy, existed) = policy_model::load_global_policy_or_defaults()
        .with_context(|| format!("failed to load global policy at {}", path.display()))?;

    let updates = config_model::parse_updates(&args.updates)?;
    let changed = policy_model::apply_updates(&mut policy, &updates)?;

    if changed || !existed {
        write_atomic_yaml(&path, &policy)
            .with_context(|| format!("failed to write {}", path.display()))?;
    }

    print_policy(&policy, args.json)?;
    Ok(())
}

fn run_workspace_init(args: &PolicyInitArgs) -> Result<()> {
    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let workspace_root = require_workspace(&cwd)?;
    let path = policy_model::workspace_policy_path(&workspace_root);
    let existed = path.exists();

    if existed && !args.force {
        println!(
            "substrate: policy already exists at {}; use --force to overwrite",
            path.display()
        );
        return Ok(());
    }

    write_atomic_bytes(&path, default_policy_yaml().as_bytes())
        .with_context(|| format!("failed to write {}", path.display()))?;
    if existed {
        println!(
            "substrate: overwrote policy at {} (--force)",
            path.display()
        );
    } else {
        println!("substrate: wrote default policy to {}", path.display());
    }
    Ok(())
}

fn run_workspace_show(args: &PolicyShowArgs) -> Result<()> {
    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    require_workspace(&cwd)?;
    let (policy, _) = policy_model::load_effective_policy(&cwd)?;

    print_policy(&policy, args.json)?;
    Ok(())
}

fn run_workspace_set(args: &PolicySetArgs) -> Result<()> {
    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let workspace_root = require_workspace(&cwd)?;
    let path = policy_model::workspace_policy_path(&workspace_root);

    let (mut policy, existed) = match fs::read_to_string(&path) {
        Ok(raw) => (policy_model::parse_policy_yaml(&path, &raw)?, true),
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
            (substrate_broker::Policy::default(), false)
        }
        Err(err) => return Err(anyhow!("failed to read {}: {err}", path.display())),
    };

    let updates = config_model::parse_updates(&args.updates)?;
    let changed = policy_model::apply_updates(&mut policy, &updates)?;

    if changed || !existed {
        write_atomic_yaml(&path, &policy)
            .with_context(|| format!("failed to write {}", path.display()))?;
    }

    print_policy(&policy, args.json)?;
    Ok(())
}

fn require_workspace(cwd: &Path) -> Result<PathBuf> {
    crate::execution::workspace::find_workspace_root(cwd).ok_or_else(|| {
        actionable("substrate: not in a workspace; run `substrate workspace init`".to_string())
    })
}

fn print_policy(policy: &substrate_broker::Policy, json: bool) -> Result<()> {
    if json {
        println!(
            "{}",
            serde_json::to_string_pretty(policy).context("failed to serialize JSON")?
        );
        return Ok(());
    }
    println!(
        "{}",
        serde_yaml::to_string(policy).context("failed to serialize YAML")?
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
