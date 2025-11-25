//! World root settings resolution and runtime helpers.

mod builder;
mod runtime;

#[cfg(test)]
mod tests;

pub(crate) use builder::{parse_bool_flag, resolve_world_root, WorldRootSettings};
pub(crate) use runtime::{apply_world_root_env, world_root_from_env};

const ANCHOR_MODE_ENV: &str = "SUBSTRATE_ANCHOR_MODE";
const ANCHOR_PATH_ENV: &str = "SUBSTRATE_ANCHOR_PATH";
const LEGACY_ROOT_MODE_ENV: &str = "SUBSTRATE_WORLD_ROOT_MODE";
const LEGACY_ROOT_PATH_ENV: &str = "SUBSTRATE_WORLD_ROOT_PATH";
