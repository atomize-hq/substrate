use crate::builtins::world_enable;
use anyhow::{bail, Result};
use std::env;
use substrate_common::paths as substrate_paths;

pub(crate) struct WorldState {
    pub(crate) cli_force_world: bool,
    cli_disabled: bool,
    env_disabled: bool,
    config_disabled: bool,
}

impl WorldState {
    pub(crate) fn detect(cli_no_world: bool, cli_force_world: bool) -> Result<Self> {
        let env_disabled = env::var("SUBSTRATE_WORLD")
            .map(|value| value.eq_ignore_ascii_case("disabled"))
            .unwrap_or(false)
            || env::var("SUBSTRATE_WORLD_ENABLED")
                .map(|value| value == "0")
                .unwrap_or(false);

        let install_config = world_enable::load_install_config(&substrate_paths::config_file()?)?;
        Ok(Self {
            cli_force_world,
            cli_disabled: cli_no_world,
            env_disabled,
            config_disabled: !install_config.world_enabled,
        })
    }

    pub(crate) fn cli_disabled(&self) -> bool {
        self.cli_disabled
    }

    pub(crate) fn is_disabled(&self) -> bool {
        if self.cli_force_world {
            return false;
        }
        self.cli_disabled || self.env_disabled || self.config_disabled
    }

    pub(crate) fn ensure_enabled(&self) -> Result<()> {
        if self.is_disabled() {
            let reason = self
                .reason()
                .unwrap_or_else(|| "unknown reason".to_string());
            bail!(
                "world backend disabled ({}). Re-run `substrate world enable` or drop --no-world.",
                reason
            );
        }
        Ok(())
    }

    pub(crate) fn reason(&self) -> Option<String> {
        if self.cli_force_world {
            return None;
        }
        if self.cli_disabled {
            Some("--no-world flag is active".to_string())
        } else if self.env_disabled {
            Some("SUBSTRATE_WORLD=disabled".to_string())
        } else if self.config_disabled {
            Some("global config reports world disabled".to_string())
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;
    use std::fs;
    use std::path::Path;
    use tempfile::tempdir;

    fn set_env(key: &str, value: &str) -> Option<String> {
        let previous = env::var(key).ok();
        env::set_var(key, value);
        previous
    }

    fn restore_env(key: &str, previous: Option<String>) {
        if let Some(value) = previous {
            env::set_var(key, value);
        } else {
            env::remove_var(key);
        }
    }

    fn write_install_config(substrate_home: &Path, enabled: bool) {
        fs::create_dir_all(substrate_home).expect("config parent");
        let flag = if enabled { "true" } else { "false" };
        fs::write(
            substrate_home.join("config.yaml"),
            format!(
                "world:\n  enabled: {flag}\n  anchor_mode: workspace\n  anchor_path: \"\"\n  caged: true\n\npolicy:\n  mode: observe\n\nsync:\n  auto_sync: false\n  direction: from_world\n  conflict_policy: prefer_host\n  exclude: []\n"
            ),
        )
        .expect("write config");
    }

    #[test]
    #[serial]
    fn force_world_flag_ignores_disabled_sources() {
        let temp = tempdir().unwrap();
        let home = temp.path().join("home");
        let substrate_home = home.join(".substrate");
        write_install_config(&substrate_home, false);

        let prev_home = set_env("HOME", &home.display().to_string());
        let prev_userprofile = set_env("USERPROFILE", &home.display().to_string());
        let prev_substrate_home = set_env("SUBSTRATE_HOME", &substrate_home.display().to_string());
        let prev_world = set_env("SUBSTRATE_WORLD", "disabled");
        let prev_world_enabled = set_env("SUBSTRATE_WORLD_ENABLED", "0");

        let state = WorldState::detect(false, true).expect("detect world state");
        assert!(!state.is_disabled());
        assert!(state.reason().is_none());

        restore_env("SUBSTRATE_WORLD", prev_world);
        restore_env("SUBSTRATE_WORLD_ENABLED", prev_world_enabled);
        restore_env("SUBSTRATE_HOME", prev_substrate_home);
        restore_env("USERPROFILE", prev_userprofile);
        restore_env("HOME", prev_home);
    }

    #[test]
    #[serial]
    fn no_world_flag_disables_even_with_enabled_metadata() {
        let temp = tempdir().unwrap();
        let home = temp.path().join("home");
        let substrate_home = home.join(".substrate");
        write_install_config(&substrate_home, true);

        let prev_home = set_env("HOME", &home.display().to_string());
        let prev_userprofile = set_env("USERPROFILE", &home.display().to_string());
        let prev_substrate_home = set_env("SUBSTRATE_HOME", &substrate_home.display().to_string());
        let prev_world = set_env("SUBSTRATE_WORLD", "enabled");
        let prev_world_enabled = set_env("SUBSTRATE_WORLD_ENABLED", "1");

        let state = WorldState::detect(true, false).expect("detect world state");
        assert!(state.is_disabled());
        assert_eq!(state.reason().as_deref(), Some("--no-world flag is active"));

        restore_env("SUBSTRATE_WORLD", prev_world);
        restore_env("SUBSTRATE_WORLD_ENABLED", prev_world_enabled);
        restore_env("SUBSTRATE_HOME", prev_substrate_home);
        restore_env("USERPROFILE", prev_userprofile);
        restore_env("HOME", prev_home);
    }

    #[test]
    #[serial]
    fn env_disabled_world_is_honored_without_flags() {
        let temp = tempdir().unwrap();
        let home = temp.path().join("home");
        let substrate_home = home.join(".substrate");
        write_install_config(&substrate_home, true);

        let prev_home = set_env("HOME", &home.display().to_string());
        let prev_userprofile = set_env("USERPROFILE", &home.display().to_string());
        let prev_substrate_home = set_env("SUBSTRATE_HOME", &substrate_home.display().to_string());
        let prev_world = set_env("SUBSTRATE_WORLD", "disabled");
        let prev_world_enabled = set_env("SUBSTRATE_WORLD_ENABLED", "0");

        let state = WorldState::detect(false, false).expect("detect world state");
        assert!(state.is_disabled());
        assert_eq!(state.reason().as_deref(), Some("SUBSTRATE_WORLD=disabled"));

        restore_env("SUBSTRATE_WORLD", prev_world);
        restore_env("SUBSTRATE_WORLD_ENABLED", prev_world_enabled);
        restore_env("SUBSTRATE_HOME", prev_substrate_home);
        restore_env("USERPROFILE", prev_userprofile);
        restore_env("HOME", prev_home);
    }

    #[test]
    #[serial]
    fn config_disabled_without_env_or_flags_is_reported() {
        let temp = tempdir().unwrap();
        let home = temp.path().join("home");
        let substrate_home = home.join(".substrate");
        write_install_config(&substrate_home, false);

        let prev_home = set_env("HOME", &home.display().to_string());
        let prev_userprofile = set_env("USERPROFILE", &home.display().to_string());
        let prev_substrate_home = set_env("SUBSTRATE_HOME", &substrate_home.display().to_string());
        let prev_world = env::var("SUBSTRATE_WORLD").ok();
        let prev_world_enabled = env::var("SUBSTRATE_WORLD_ENABLED").ok();
        env::remove_var("SUBSTRATE_WORLD");
        env::remove_var("SUBSTRATE_WORLD_ENABLED");

        let state = WorldState::detect(false, false).expect("detect world state");
        assert!(state.is_disabled());
        assert_eq!(
            state.reason().as_deref(),
            Some("global config reports world disabled")
        );

        restore_env("SUBSTRATE_WORLD_ENABLED", prev_world_enabled);
        restore_env("SUBSTRATE_WORLD", prev_world);
        restore_env("SUBSTRATE_HOME", prev_substrate_home);
        restore_env("USERPROFILE", prev_userprofile);
        restore_env("HOME", prev_home);
    }
}
