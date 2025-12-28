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
    fn update_manager_env_exports_preserves_shebang_and_existing_lines() {
        let temp = tempdir().unwrap();
        let manager_env = temp.path().join("env/manager_env.sh");
        fs::create_dir_all(manager_env.parent().unwrap()).unwrap();
        fs::write(&manager_env, "#!/usr/bin/env bash\nexport OTHER_VAR=1\n").unwrap();

        update_manager_env_exports(&manager_env, true).unwrap();
        let contents = fs::read_to_string(&manager_env).unwrap();
        assert!(contents.starts_with(
            "#!/usr/bin/env bash\n# Managed by `substrate world enable`\nexport SUBSTRATE_WORLD=enabled\nexport SUBSTRATE_WORLD_ENABLED=1\n"
        ));
        assert!(contents.contains("export OTHER_VAR=1"));

        update_manager_env_exports(&manager_env, false).unwrap();
        let updated = fs::read_to_string(&manager_env).unwrap();
        assert!(updated.contains("export SUBSTRATE_WORLD=disabled"));
        assert!(updated.contains("export SUBSTRATE_WORLD_ENABLED=0"));
        assert!(updated.contains("export OTHER_VAR=1"));
    }
}
