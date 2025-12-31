use crate::execution::cli::{WorkspaceAction, WorkspaceCmd, WorkspaceInitArgs};
use crate::execution::config_model::{default_policy_yaml, SubstrateConfig};
use crate::execution::workspace;
use anyhow::{anyhow, Context, Result};
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use tempfile::NamedTempFile;

pub(crate) fn handle_workspace_command(cmd: &WorkspaceCmd) -> i32 {
    let result = match &cmd.action {
        WorkspaceAction::Init(args) => run_workspace_init(args),
    };

    match result {
        Ok(()) => 0,
        Err(err) if err.is::<ActionableError>() => {
            eprintln!("{:#}", err);
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

fn run_workspace_init(args: &WorkspaceInitArgs) -> Result<()> {
    let target = args
        .path
        .clone()
        .unwrap_or_else(|| PathBuf::from("."))
        .canonicalize()
        .with_context(|| "invalid PATH for workspace init")
        .map_err(|err| actionable(err.to_string()))?;

    if !target.is_dir() {
        return Err(actionable(format!(
            "substrate: invalid PATH for workspace init: {} (not a directory)",
            target.display()
        )));
    }

    ensure_not_nested_workspace(&target)?;

    fs::create_dir_all(target.join(workspace::SUBSTRATE_DIR_NAME))
        .with_context(|| format!("failed to create {}", target.display()))?;
    fs::create_dir_all(workspace::internal_git_dir(&target)).with_context(|| {
        format!(
            "failed to create internal git dir under {}",
            target.display()
        )
    })?;

    let workspace_yaml = workspace::workspace_marker_path(&target);
    if !workspace_yaml.exists() {
        write_atomic_yaml(&workspace_yaml, &SubstrateConfig::default())
            .with_context(|| format!("failed to write {}", workspace_yaml.display()))?;
    }

    let policy_yaml = workspace::workspace_policy_path(&target);
    if !policy_yaml.exists() {
        write_atomic_bytes(&policy_yaml, default_policy_yaml().as_bytes())
            .with_context(|| format!("failed to write {}", policy_yaml.display()))?;
    }

    ensure_gitignore_rules(&target).context("failed to update .gitignore")?;

    if args.force {
        println!(
            "substrate: workspace initialized at {} (--force; repaired missing entries only)",
            target.display()
        );
    } else {
        println!("substrate: workspace initialized at {}", target.display());
    }

    Ok(())
}

fn ensure_not_nested_workspace(target: &Path) -> Result<()> {
    let mut ancestors = target.ancestors();
    let _self_dir = ancestors.next();
    for parent in ancestors {
        let marker = workspace::workspace_marker_path(parent);
        if marker.is_file() {
            return Err(actionable(format!(
                "substrate: refusing to create a nested workspace at {}\nFound parent workspace marker at {}\n",
                target.display(),
                marker.display()
            )));
        }
    }
    Ok(())
}

fn ensure_gitignore_rules(root: &Path) -> Result<()> {
    let gitignore = root.join(".gitignore");
    let mut existing = match fs::read_to_string(&gitignore) {
        Ok(raw) => raw.lines().map(|l| l.to_string()).collect::<Vec<_>>(),
        Err(err) if err.kind() == io::ErrorKind::NotFound => Vec::new(),
        Err(err) => return Err(anyhow!("failed to read {}: {err}", gitignore.display())),
    };

    let required = [
        ".substrate-git/",
        ".substrate/*",
        "!.substrate/workspace.yaml",
        "!.substrate/policy.yaml",
    ];

    for rule in required {
        if !existing.iter().any(|line| line.trim_end() == rule) {
            existing.push(rule.to_string());
        }
    }

    let mut body = existing.join("\n");
    if !body.ends_with('\n') {
        body.push('\n');
    }
    write_atomic_bytes(&gitignore, body.as_bytes())
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
