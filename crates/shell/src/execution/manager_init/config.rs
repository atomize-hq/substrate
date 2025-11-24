//! Manager init configuration builders and validation helpers.

use std::collections::HashSet;
use std::env;
use std::path::PathBuf;

use substrate_common::manager_manifest::Platform;

#[derive(Debug, Clone)]
pub struct ManifestPaths {
    pub base: PathBuf,
    pub overlay: Option<PathBuf>,
}

#[derive(Debug, Clone)]
pub struct ManagerInitConfig {
    pub skip_all: bool,
    pub skip_list: HashSet<String>,
    pub platform: Platform,
    pub debug: bool,
}

impl ManagerInitConfig {
    pub fn from_env(platform: Platform) -> Self {
        let skip_all = env::var("SUBSTRATE_SKIP_MANAGER_INIT")
            .map(|value| is_truthy(&value))
            .unwrap_or(false);
        let skip_list = env::var("SUBSTRATE_SKIP_MANAGER_INIT_LIST")
            .ok()
            .map(|value| parse_skip_list(&value))
            .unwrap_or_default();
        let debug = env::var("SUBSTRATE_MANAGER_INIT_DEBUG")
            .map(|value| is_truthy(&value))
            .unwrap_or(false);

        Self {
            skip_all,
            skip_list,
            platform,
            debug,
        }
    }
}

pub(super) fn is_truthy(value: &str) -> bool {
    matches!(
        value.to_ascii_lowercase().as_str(),
        "1" | "true" | "yes" | "on"
    )
}

pub(super) fn parse_skip_list(raw: &str) -> HashSet<String> {
    raw.split(|c: char| c == ',' || c.is_whitespace())
        .map(|item| item.trim().to_lowercase())
        .filter(|item| !item.is_empty())
        .collect()
}
