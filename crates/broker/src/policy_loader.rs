//! Helpers for reading and parsing policy definitions from disk.

use crate::Policy;
use anyhow::{Context, Result};
use std::path::Path;
use std::path::PathBuf;
use substrate_common::paths as substrate_paths;

pub fn load_policy_from_path(path: &Path) -> Result<Policy> {
    let content = std::fs::read_to_string(path)
        .with_context(|| format!("Failed to read policy from {:?}", path))?;

    serde_yaml::from_str(&content)
        .map_err(|err| anyhow::anyhow!("Failed to parse policy from {:?}: {}", path, err))
}

pub fn load_effective_policy_for_cwd(cwd: &Path) -> Result<(Policy, Option<PathBuf>)> {
    let home = substrate_paths::substrate_home()?;

    if let Some(workspace_root) = find_workspace_root(cwd) {
        let workspace_policy = workspace_root
            .join(substrate_paths::SUBSTRATE_DIR_NAME)
            .join("policy.yaml");
        if workspace_policy.exists() {
            let policy = load_policy_from_path(&workspace_policy)?;
            return Ok((policy, Some(workspace_policy)));
        }
    }

    let global_policy = home.join("policy.yaml");
    if global_policy.exists() {
        let policy = load_policy_from_path(&global_policy)?;
        return Ok((policy, Some(global_policy)));
    }

    Ok((Policy::default(), None))
}

fn find_workspace_root(cwd: &Path) -> Option<PathBuf> {
    let mut current = cwd;
    if current.is_file() {
        current = current.parent()?;
    }

    loop {
        let marker = current
            .join(substrate_paths::SUBSTRATE_DIR_NAME)
            .join("workspace.yaml");
        if marker.is_file() {
            return Some(current.to_path_buf());
        }

        current = current.parent()?;
    }
}
