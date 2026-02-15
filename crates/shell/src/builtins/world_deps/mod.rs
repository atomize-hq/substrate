mod errors;
mod inventory;
mod surfaces;

pub use surfaces::run;

pub(crate) use inventory::InventoryListItemSummaryV1;

use crate::execution::config_model;
use anyhow::Result;
use std::path::{Path, PathBuf};

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone)]
pub(crate) struct WorldDepsDoctorSnapshotV1 {
    pub schema_version: u32,
    pub cwd: PathBuf,
    pub inventory_packages: usize,
    pub inventory_bundles: usize,
    pub inventory_mode: String,
    pub builtins: String,
    pub enabled: Vec<String>,
    pub applied: Vec<InventoryListItemSummaryV1>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub applied_error: Option<String>,
}

pub(crate) fn collect_doctor_snapshot_v1(
    cwd: &Path,
    all: bool,
) -> Result<WorldDepsDoctorSnapshotV1> {
    let cfg = config_model::resolve_effective_config(cwd, &Default::default())?;
    let view = surfaces::resolve_current_inventory_view(cwd, &cfg)?;
    let enabled = cfg.world.deps.enabled.clone();
    let inventory_mode = match cfg.world.deps.inventory_mode {
        config_model::WorldDepsInventoryMode::Merged => "merged",
        config_model::WorldDepsInventoryMode::WorkspaceOnly => "workspace_only",
    }
    .to_string();
    let builtins = match cfg.world.deps.builtins {
        config_model::WorldDepsBuiltinsMode::Enabled => "enabled",
        config_model::WorldDepsBuiltinsMode::Disabled => "disabled",
    }
    .to_string();

    let applied = match surfaces::compute_current_applied_items_v1(&view, &enabled, all) {
        Ok(items) => WorldDepsDoctorSnapshotV1 {
            schema_version: 1,
            cwd: cwd.to_path_buf(),
            inventory_packages: view.packages.len(),
            inventory_bundles: view.bundles.len(),
            inventory_mode: inventory_mode.clone(),
            builtins: builtins.clone(),
            enabled,
            applied: items,
            applied_error: None,
        },
        Err(err) => WorldDepsDoctorSnapshotV1 {
            schema_version: 1,
            cwd: cwd.to_path_buf(),
            inventory_packages: view.packages.len(),
            inventory_bundles: view.bundles.len(),
            inventory_mode,
            builtins,
            enabled,
            applied: Vec::new(),
            applied_error: Some(format!("{:#}", err)),
        },
    };

    Ok(applied)
}
