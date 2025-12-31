//! Manager environment export updates for world enable.

use crate::execution::config_model;
use crate::execution::write_env_sh_at;
use anyhow::{Context, Result};
#[cfg(test)]
use std::fs;
use std::path::Path;
use substrate_common::paths as substrate_paths;

pub(super) fn update_manager_env_exports(path: &Path, enabled: bool) -> Result<()> {
    let substrate_home = substrate_paths::substrate_home()
        .with_context(|| "failed to resolve Substrate home for env.sh")?;
    let (mut cfg, _) = config_model::read_global_config_or_defaults()
        .with_context(|| "failed to load config for env.sh")?;
    cfg.world.enabled = enabled;
    write_env_sh_at(path, &substrate_home, &cfg)
        .with_context(|| format!("failed to write env.sh at {}", path.display()))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn update_manager_env_exports_writes_env_sh_format() {
        let temp = tempdir().unwrap();
        let home = temp.path().join("substrate-home");
        let env_sh = home.join("env.sh");
        fs::create_dir_all(env_sh.parent().unwrap()).unwrap();

        let prev_home = std::env::var_os("SUBSTRATE_HOME");
        std::env::set_var("SUBSTRATE_HOME", &home);
        let result = update_manager_env_exports(&env_sh, true);
        if let Some(val) = prev_home {
            std::env::set_var("SUBSTRATE_HOME", val);
        } else {
            std::env::remove_var("SUBSTRATE_HOME");
        }
        result.unwrap();

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
