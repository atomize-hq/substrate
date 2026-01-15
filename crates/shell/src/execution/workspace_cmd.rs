use crate::execution::cli::{WorkspaceAction, WorkspaceCmd, WorkspaceInitArgs, WorkspacePathArgs};
use crate::execution::workspace;
use anyhow::{anyhow, Context, Result};
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use tempfile::NamedTempFile;

const DEFAULT_WORKSPACE_PATCH_YAML: &str = r#"# Substrate workspace config patch (sparse overrides).
# - This file is a YAML mapping of workspace-scoped overrides.
# - Omitted keys inherit from global config + defaults.
{}
"#;

const DEFAULT_POLICY_PATCH_YAML: &str = r#"# Substrate workspace policy patch (sparse overrides).
# - This file is a YAML mapping of workspace-scoped overrides.
# - Omitted keys inherit from global policy + defaults.
{}
"#;

pub(crate) fn handle_workspace_command(cmd: &WorkspaceCmd) -> i32 {
    let result = match &cmd.action {
        WorkspaceAction::Init(args) => run_workspace_init(args),
        WorkspaceAction::Disable(args) => run_workspace_disable(args),
        WorkspaceAction::Enable(args) => run_workspace_enable(args),
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
        write_atomic_bytes(&workspace_yaml, DEFAULT_WORKSPACE_PATCH_YAML.as_bytes())
            .with_context(|| format!("failed to write {}", workspace_yaml.display()))?;
    }

    let policy_yaml = workspace::workspace_policy_path(&target);
    if !policy_yaml.exists() {
        write_atomic_bytes(&policy_yaml, DEFAULT_POLICY_PATCH_YAML.as_bytes())
            .with_context(|| format!("failed to write {}", policy_yaml.display()))?;
    }

    if args.examples {
        ensure_example_files(&target)?;
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

fn run_workspace_disable(args: &WorkspacePathArgs) -> Result<()> {
    let start = resolve_search_root(args, "workspace disable")?;
    let workspace_root = require_workspace_root_any(&start, "workspace disable")?;
    let marker = workspace::workspace_disabled_marker_path(&workspace_root);

    if !marker.exists() {
        write_atomic_bytes(&marker, b"")
            .with_context(|| format!("failed to write {}", marker.display()))?;
    }

    println!(
        "substrate: workspace disabled at {}",
        workspace_root.display()
    );
    Ok(())
}

fn run_workspace_enable(args: &WorkspacePathArgs) -> Result<()> {
    let start = resolve_search_root(args, "workspace enable")?;
    let workspace_root = require_workspace_root_any(&start, "workspace enable")?;
    let marker = workspace::workspace_disabled_marker_path(&workspace_root);

    match fs::remove_file(&marker) {
        Ok(()) => {}
        Err(err) if err.kind() == io::ErrorKind::NotFound => {}
        Err(err) => return Err(anyhow!("failed to remove {}: {err}", marker.display())),
    }

    println!(
        "substrate: workspace enabled at {}",
        workspace_root.display()
    );
    Ok(())
}

fn resolve_search_root(args: &WorkspacePathArgs, verb: &str) -> Result<PathBuf> {
    let target = args
        .path
        .clone()
        .unwrap_or_else(|| PathBuf::from("."))
        .canonicalize()
        .with_context(|| format!("invalid PATH for {verb}"))
        .map_err(|err| actionable(err.to_string()))?;

    if target.is_dir() {
        return Ok(target);
    }
    if target.is_file() {
        if let Some(parent) = target.parent() {
            return Ok(parent.to_path_buf());
        }
    }

    Err(actionable(format!(
        "substrate: invalid PATH for {verb}: {}",
        target.display()
    )))
}

fn require_workspace_root_any(start: &Path, verb: &str) -> Result<PathBuf> {
    workspace::find_workspace_root_any(start).ok_or_else(|| {
        actionable(format!(
            "substrate: not in a workspace for {verb}; run `substrate workspace init`"
        ))
    })
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

    let substrate_ignore = ".substrate/";
    let workspace_allow = "!.substrate/workspace.yaml";
    let policy_allow = "!.substrate/policy.yaml";

    if !existing
        .iter()
        .any(|line| line.trim_end() == substrate_ignore)
    {
        existing.push(substrate_ignore.to_string());
    }

    let last_substrate_ignore_idx = existing
        .iter()
        .rposition(|line| line.trim_end() == substrate_ignore)
        .expect("substrate_ignore must exist");

    let has_workspace_allow_after = existing[last_substrate_ignore_idx + 1..]
        .iter()
        .any(|line| line.trim_end() == workspace_allow);
    let has_policy_allow_after = existing[last_substrate_ignore_idx + 1..]
        .iter()
        .any(|line| line.trim_end() == policy_allow);

    if !has_workspace_allow_after {
        existing.push(workspace_allow.to_string());
    }
    if !has_policy_allow_after {
        existing.push(policy_allow.to_string());
    }

    let mut body = existing.join("\n");
    if !body.ends_with('\n') {
        body.push('\n');
    }
    write_atomic_bytes(&gitignore, body.as_bytes())
}

fn ensure_example_files(workspace_root: &Path) -> Result<()> {
    let substrate_dir = workspace_root.join(workspace::SUBSTRATE_DIR_NAME);
    let workspace_example = substrate_dir.join("workspace.example.yaml");
    let policy_example = substrate_dir.join("policy.example.yaml");

    if !workspace_example.exists() {
        write_atomic_bytes(&workspace_example, DEFAULT_WORKSPACE_PATCH_YAML.as_bytes())
            .with_context(|| {
                format!(
                    "failed to write workspace example file {}",
                    workspace_example.display()
                )
            })?;
    }

    if !policy_example.exists() {
        write_atomic_bytes(&policy_example, DEFAULT_POLICY_PATCH_YAML.as_bytes()).with_context(
            || {
                format!(
                    "failed to write policy example file {}",
                    policy_example.display()
                )
            },
        )?;
    }

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
