use super::models::WorldDepsSelectionScope;
use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use substrate_common::paths as substrate_paths;
use tempfile::NamedTempFile;

pub(crate) const SELECTION_FILENAME: &str = "world-deps.selection.yaml";
const SELECTION_VERSION: u32 = 1;
const SUBSTRATE_DIR_NAME: &str = ".substrate";

#[derive(Debug)]
pub(crate) struct SelectionConfigError {
    pub(crate) message: String,
}

impl std::fmt::Display for SelectionConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for SelectionConfigError {}

pub(crate) fn selection_schema_example() -> &'static str {
    "version: 1\nselected:\n  - <tool_name>\n"
}

#[derive(Debug, Clone)]
pub(crate) struct ActiveSelection {
    pub(crate) configured: bool,
    pub(crate) active_path: Option<PathBuf>,
    pub(crate) active_scope: Option<WorldDepsSelectionScope>,
    pub(crate) shadowed_paths: Vec<PathBuf>,
    pub(crate) selected: Vec<String>,
}

pub(crate) fn resolve_active_selection(
    cwd: &Path,
    inventory_tool_names: &HashSet<String>,
) -> Result<ActiveSelection> {
    let workspace = find_workspace_selection_file(cwd);
    let global = canonicalize_if_exists(global_selection_path()?);

    let (active_path, active_scope) = if let Some(workspace) = &workspace {
        (
            Some(workspace.clone()),
            Some(WorldDepsSelectionScope::Workspace),
        )
    } else if global.is_file() {
        (Some(global.clone()), Some(WorldDepsSelectionScope::Global))
    } else {
        (None, None)
    };

    let mut shadowed = Vec::new();
    if workspace.is_some() && global.is_file() {
        shadowed.push(global.clone());
    }

    let Some(active_path) = active_path else {
        return Ok(ActiveSelection {
            configured: false,
            active_path: None,
            active_scope: None,
            shadowed_paths: shadowed,
            selected: Vec::new(),
        });
    };

    let raw = read_selection_file(&active_path)?;
    let normalized = normalize_selected_list(&raw.selected, &active_path)?;
    let mut unknown = normalized
        .iter()
        .filter(|name| !inventory_tool_names.contains(*name))
        .cloned()
        .collect::<Vec<_>>();
    unknown.sort();
    unknown.dedup();
    if !unknown.is_empty() {
        return Err(anyhow!(SelectionConfigError {
            message: format!(
                "substrate: invalid world-deps selection at {}: unknown tool(s): {}\nHint: run `substrate world deps status --all` after initializing selection to discover available tool names.",
                active_path.display(),
                unknown.join(", ")
            ),
        }));
    }

    Ok(ActiveSelection {
        configured: true,
        active_path: Some(active_path),
        active_scope,
        shadowed_paths: shadowed,
        selected: normalized,
    })
}

pub(crate) fn resolve_selection_target(
    cwd: &Path,
    workspace: bool,
    global: bool,
) -> Result<(WorldDepsSelectionScope, PathBuf)> {
    if workspace {
        let root = find_workspace_root_for_write(cwd);
        return Ok((
            WorldDepsSelectionScope::Workspace,
            workspace_selection_path_at(&root),
        ));
    }
    if global {
        let path = global_selection_path()?;
        return Ok((WorldDepsSelectionScope::Global, path));
    }

    if find_workspace_root_if_substrate_dir_exists(cwd).is_some() {
        let root = find_workspace_root_for_write(cwd);
        Ok((
            WorldDepsSelectionScope::Workspace,
            workspace_selection_path_at(&root),
        ))
    } else {
        let path = global_selection_path()?;
        Ok((WorldDepsSelectionScope::Global, path))
    }
}

pub(crate) fn write_empty_selection_file(path: &Path, force: bool) -> Result<()> {
    if path.exists() && !force {
        return Err(anyhow!(SelectionConfigError {
            message: format!(
                "substrate: selection file already exists at {}\nRe-run with --force to overwrite.",
                path.display()
            ),
        }));
    }
    write_selection_file(path, &[])
}

pub(crate) fn add_tools_to_selection_file(
    path: &Path,
    tools: &[String],
    inventory_tool_names: &HashSet<String>,
) -> Result<Vec<String>> {
    let existing = if path.exists() {
        let raw = read_selection_file(path)?;
        normalize_selected_list(&raw.selected, path)?
    } else {
        Vec::new()
    };

    let mut merged: HashSet<String> = existing.into_iter().collect();
    let mut added = Vec::new();
    for tool in tools {
        let normalized = normalize_tool_name(tool);
        if normalized.is_empty() {
            return Err(anyhow!(SelectionConfigError {
                message: format!(
                    "substrate: invalid tool name in selection update for {} (empty string)",
                    path.display()
                ),
            }));
        }
        if !inventory_tool_names.contains(&normalized) {
            return Err(anyhow!(SelectionConfigError {
                message: format!(
                    "substrate: unknown tool `{}` (not in inventory); run `substrate world deps status --all` after initializing selection to discover available tool names.",
                    tool
                ),
            }));
        }
        if merged.insert(normalized.clone()) {
            added.push(normalized);
        }
    }

    let mut final_list = merged.into_iter().collect::<Vec<_>>();
    final_list.sort();
    final_list.dedup();
    write_selection_file(path, &final_list)?;

    added.sort();
    added.dedup();
    Ok(added)
}

fn read_selection_file(path: &Path) -> Result<SelectionFile> {
    let raw = fs::read_to_string(path)
        .with_context(|| format!("failed to read selection file {}", path.display()))?;
    let parsed: SelectionFile = serde_yaml::from_str(&raw).map_err(|err| {
        anyhow!(SelectionConfigError {
            message: format!(
                "substrate: invalid world-deps selection YAML at {}: {err}\nExpected schema:\n{}",
                path.display(),
                selection_schema_example()
            ),
        })
    })?;
    if parsed.version != SELECTION_VERSION {
        return Err(anyhow!(SelectionConfigError {
            message: format!(
                "substrate: invalid world-deps selection at {}: version must be {} (got {})",
                path.display(),
                SELECTION_VERSION,
                parsed.version
            ),
        }));
    }
    Ok(parsed)
}

fn write_selection_file(path: &Path, selected: &[String]) -> Result<()> {
    let parent = path
        .parent()
        .ok_or_else(|| anyhow!("path {} has no parent", path.display()))?;
    fs::create_dir_all(parent).with_context(|| format!("failed to create {}", parent.display()))?;

    let file = SelectionFile {
        version: SELECTION_VERSION,
        selected: selected.to_vec(),
    };
    let body = serde_yaml::to_string(&file)
        .with_context(|| format!("failed to serialize {}", path.display()))?;

    let mut tmp = NamedTempFile::new_in(parent)
        .with_context(|| format!("failed to create temp file near {}", path.display()))?;
    tmp.write_all(body.as_bytes())
        .with_context(|| format!("failed to write {}", path.display()))?;
    tmp.flush()?;
    tmp.persist(path)
        .map_err(|err| anyhow!("failed to persist {}: {}", path.display(), err.error))?;
    Ok(())
}

fn global_selection_path() -> Result<PathBuf> {
    Ok(substrate_paths::substrate_home()?.join(SELECTION_FILENAME))
}

fn canonicalize_if_exists(path: PathBuf) -> PathBuf {
    if path.exists() {
        path.canonicalize().unwrap_or(path)
    } else {
        path
    }
}

fn workspace_selection_path_at(workspace_root: &Path) -> PathBuf {
    workspace_root
        .join(SUBSTRATE_DIR_NAME)
        .join(SELECTION_FILENAME)
}

fn find_workspace_selection_file(cwd: &Path) -> Option<PathBuf> {
    let start = cwd.canonicalize().unwrap_or_else(|_| cwd.to_path_buf());
    for dir in start.ancestors() {
        let candidate = workspace_selection_path_at(dir);
        if candidate.is_file() {
            return Some(candidate);
        }
    }
    None
}

fn find_workspace_root_if_substrate_dir_exists(cwd: &Path) -> Option<PathBuf> {
    let start = cwd.canonicalize().unwrap_or_else(|_| cwd.to_path_buf());
    for dir in start.ancestors() {
        if dir.join(SUBSTRATE_DIR_NAME).is_dir() {
            return Some(dir.to_path_buf());
        }
    }
    None
}

fn find_workspace_root_for_write(cwd: &Path) -> PathBuf {
    find_workspace_root_if_substrate_dir_exists(cwd).unwrap_or_else(|| cwd.to_path_buf())
}

fn normalize_tool_name(value: &str) -> String {
    value.trim().to_ascii_lowercase()
}

fn normalize_selected_list(raw: &[String], path: &Path) -> Result<Vec<String>> {
    let mut out = Vec::new();
    for entry in raw {
        let normalized = normalize_tool_name(entry);
        if normalized.is_empty() {
            return Err(anyhow!(SelectionConfigError {
                message: format!(
                    "substrate: invalid world-deps selection at {}: selected entries must be non-empty strings",
                    path.display()
                ),
            }));
        }
        out.push(normalized);
    }
    out.sort();
    out.dedup();
    Ok(out)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SelectionFile {
    version: u32,
    selected: Vec<String>,
}
