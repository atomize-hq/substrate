//! Manager environment export updates for world enable.

use crate::execution::config_model;
use crate::execution::write_env_sh_at;
use anyhow::{Context, Result};
#[cfg(test)]
use std::fs;
use std::path::Path;

pub(super) fn update_manager_env_exports(
    path: &Path,
    substrate_home: &Path,
    enabled: bool,
) -> Result<()> {
    let (mut cfg, _) = config_model::read_global_config_or_defaults()
        .with_context(|| "failed to load config for env.sh")?;
    cfg.world.enabled = enabled;
    write_env_sh_at(path, substrate_home, &cfg)
        .with_context(|| format!("failed to write env.sh at {}", path.display()))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;
    use tempfile::tempdir;

    #[test]
    #[serial]
    fn update_manager_env_exports_writes_env_sh_format() {
        let temp = tempdir().unwrap();
        let home = temp.path().join("substrate-home");
        let env_sh = home.join("env.sh");
        fs::create_dir_all(env_sh.parent().unwrap()).unwrap();

        update_manager_env_exports(&env_sh, &home, true).unwrap();

        let contents = fs::read_to_string(&env_sh).unwrap();
        assert!(contents.starts_with("#!/usr/bin/env bash\n"));
        assert!(contents.contains("export SUBSTRATE_HOME="));
        assert!(contents.contains("export SUBSTRATE_WORLD='enabled'\n"));
        assert!(contents.contains("export SUBSTRATE_CAGED="));
        assert!(contents.contains("export SUBSTRATE_ANCHOR_MODE="));
        assert!(contents.contains("export SUBSTRATE_ANCHOR_PATH="));
        assert!(contents.contains("export SUBSTRATE_POLICY_MODE="));
    }
}
