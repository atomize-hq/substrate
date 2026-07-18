//! Manager environment export updates for world enable.

use crate::execution::config_model;
use crate::execution::write_env_sh_at;
use anyhow::{anyhow, Context, Result};
use std::fs;
use std::io;
use std::path::Path;

pub(super) fn update_manager_env_exports(
    path: &Path,
    substrate_home: &Path,
    enabled: bool,
) -> Result<()> {
    let mut cfg = if cfg!(unix) {
        let config_path = substrate_home.join("config.yaml");
        match fs::read_to_string(&config_path) {
            Ok(raw) => config_model::parse_config_yaml(&config_path, &raw),
            Err(err) if err.kind() == io::ErrorKind::NotFound => {
                Ok(config_model::SubstrateConfig::default())
            }
            Err(err) => Err(anyhow!("failed to read {}: {err}", config_path.display())),
        }
        .with_context(|| "failed to load config for env.sh")?
    } else {
        config_model::read_global_config_or_defaults()
            .with_context(|| "failed to load config for env.sh")?
            .0
    };
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
        let ambient_home = temp.path().join("ambient-home");
        let env_sh = home.join("env.sh");
        fs::create_dir_all(env_sh.parent().unwrap()).unwrap();
        fs::create_dir_all(&ambient_home).unwrap();
        fs::write(
            home.join("config.yaml"),
            "world:\n  enabled: false\n  anchor_mode: follow-cwd\n  caged: false\n  net:\n    filter: false\npolicy:\n  mode: disabled\n",
        )
        .unwrap();
        fs::write(
            ambient_home.join("config.yaml"),
            "world:\n  enabled: false\n  anchor_mode: workspace\n  caged: true\n  net:\n    filter: true\npolicy:\n  mode: observe\n",
        )
        .unwrap();

        let previous_home = std::env::var_os("SUBSTRATE_HOME");
        std::env::set_var("SUBSTRATE_HOME", &ambient_home);
        let result = update_manager_env_exports(&env_sh, &home, true);
        match previous_home {
            Some(value) => std::env::set_var("SUBSTRATE_HOME", value),
            None => std::env::remove_var("SUBSTRATE_HOME"),
        }
        result.unwrap();

        let contents = fs::read_to_string(&env_sh).unwrap();
        assert!(contents.starts_with("#!/usr/bin/env bash\n"));
        assert!(contents.contains(&format!("export SUBSTRATE_HOME='{}'", home.display())));
        assert!(!contents.contains(&ambient_home.display().to_string()));
        assert!(contents.contains("export SUBSTRATE_WORLD='enabled'\n"));
        if cfg!(unix) {
            assert!(contents.contains("export SUBSTRATE_CAGED='0'\n"));
            assert!(contents.contains("export SUBSTRATE_ANCHOR_MODE='follow-cwd'\n"));
            assert!(contents.contains("export SUBSTRATE_POLICY_MODE='disabled'\n"));
            assert!(contents.contains("export SUBSTRATE_WORLD_NET_FILTER='0'\n"));
        } else {
            assert!(contents.contains("export SUBSTRATE_CAGED='1'\n"));
            assert!(contents.contains("export SUBSTRATE_ANCHOR_MODE='workspace'\n"));
            assert!(contents.contains("export SUBSTRATE_POLICY_MODE='observe'\n"));
            assert!(contents.contains("export SUBSTRATE_WORLD_NET_FILTER='1'\n"));
        }
        assert!(contents.contains("export SUBSTRATE_ANCHOR_PATH=''\n"));

        #[cfg(unix)]
        {
            let config_path = home.join("config.yaml");
            fs::remove_file(&config_path).unwrap();
            fs::create_dir(&config_path).unwrap();
            let err = update_manager_env_exports(&env_sh, &home, true).unwrap_err();
            let message = format!("{err:#}");
            assert!(message.contains("failed to load config for env.sh"));
            assert!(message.contains(&format!("failed to read {}", config_path.display())));
        }
    }
}
