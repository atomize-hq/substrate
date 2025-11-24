//! Runtime helpers for applying resolved world root settings.

use super::builder::{first_env_value, parse_bool_flag, WorldRootSettings};
use super::{ANCHOR_MODE_ENV, ANCHOR_PATH_ENV, LEGACY_ROOT_MODE_ENV, LEGACY_ROOT_PATH_ENV};
use std::env;
use std::path::PathBuf;
use substrate_common::WorldRootMode;

pub(crate) fn apply_world_root_env(settings: &WorldRootSettings) {
    let mode = settings.mode.as_str();
    let path = settings.path.to_string_lossy().to_string();
    env::set_var(ANCHOR_MODE_ENV, mode);
    env::set_var(LEGACY_ROOT_MODE_ENV, mode);
    env::set_var(ANCHOR_PATH_ENV, &path);
    env::set_var(LEGACY_ROOT_PATH_ENV, &path);
    env::set_var("SUBSTRATE_CAGED", if settings.caged { "1" } else { "0" });
}

pub(crate) fn world_root_from_env() -> WorldRootSettings {
    let cwd = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let mode = first_env_value(&[ANCHOR_MODE_ENV, LEGACY_ROOT_MODE_ENV])
        .and_then(|(_, value)| WorldRootMode::parse(&value))
        .unwrap_or(WorldRootMode::Project);
    let base_path = first_env_value(&[ANCHOR_PATH_ENV, LEGACY_ROOT_PATH_ENV])
        .and_then(|(_, value)| {
            let trimmed = value.trim();
            (!trimmed.is_empty()).then(|| PathBuf::from(trimmed))
        })
        .unwrap_or_else(|| cwd.clone());
    let path = match mode {
        WorldRootMode::FollowCwd => cwd,
        _ => base_path,
    };
    let caged = env::var("SUBSTRATE_CAGED")
        .ok()
        .and_then(|value| parse_bool_flag(&value))
        .unwrap_or(true);
    WorldRootSettings { mode, path, caged }
}
