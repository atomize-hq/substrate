//! Helpers for reading and parsing policy definitions from disk.

use crate::Policy;
use anyhow::{Context, Result};
use std::path::Path;

pub fn load_policy_from_path(path: &Path) -> Result<Policy> {
    let content = std::fs::read_to_string(path)
        .with_context(|| format!("Failed to read policy from {:?}", path))?;

    serde_yaml::from_str(&content)
        .map_err(|err| anyhow::anyhow!("Failed to parse policy from {:?}: {}", path, err))
}
