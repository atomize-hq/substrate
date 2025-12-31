//! Resolve world root configuration from CLI, env, and ADR-0003 config files.

use crate::execution::config_model::{resolve_effective_config, user_error, CliConfigOverrides};
use crate::execution::workspace;
use anyhow::{Context, Result};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use substrate_common::WorldRootMode;

#[derive(Debug, Clone)]
pub struct WorldRootSettings {
    pub mode: WorldRootMode,
    pub path: PathBuf,
    pub caged: bool,
}

impl WorldRootSettings {
    pub fn effective_root(&self) -> PathBuf {
        match self.mode {
            WorldRootMode::FollowCwd => env::current_dir().unwrap_or_else(|_| self.path.clone()),
            _ => self.path.clone(),
        }
    }

    pub fn anchor_root(&self, current_dir: &Path) -> PathBuf {
        match self.mode {
            WorldRootMode::FollowCwd => current_dir.to_path_buf(),
            _ => self.path.clone(),
        }
    }
}

pub(crate) fn resolve_world_root(
    cli_mode: Option<WorldRootMode>,
    cli_path: Option<PathBuf>,
    cli_caged: Option<bool>,
    launch_dir: &Path,
) -> Result<WorldRootSettings> {
    let cli = CliConfigOverrides {
        anchor_mode: cli_mode,
        anchor_path: cli_path.map(|p| p.to_string_lossy().to_string()),
        caged: cli_caged,
        world_enabled: None,
    };
    let cfg = resolve_effective_config(launch_dir, &cli)?;

    let mode = cfg.world.anchor_mode;
    let caged = cfg.world.caged;

    let path = match mode {
        WorldRootMode::FollowCwd => launch_dir.to_path_buf(),
        WorldRootMode::Custom => resolve_custom_anchor_path(&cfg.world.anchor_path, launch_dir)?,
        WorldRootMode::Project => {
            workspace::find_workspace_root(launch_dir).unwrap_or_else(|| launch_dir.to_path_buf())
        }
    };

    Ok(WorldRootSettings { mode, path, caged })
}

fn resolve_custom_anchor_path(raw: &str, launch_dir: &Path) -> Result<PathBuf> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Err(user_error(
            "anchor_mode=custom requires world.anchor_path to be non-empty (use --anchor-path, SUBSTRATE_ANCHOR_PATH, or config)",
        ));
    }

    let mut path = PathBuf::from(trimmed);
    if path.is_relative() {
        path = launch_dir.join(path);
    }

    let meta = fs::metadata(&path)
        .with_context(|| format!("anchor_path does not exist: {}", path.display()))?;
    if !meta.is_dir() {
        return Err(user_error(format!(
            "anchor_path must be a directory (found {})",
            path.display()
        )));
    }

    Ok(path.canonicalize().unwrap_or(path))
}
