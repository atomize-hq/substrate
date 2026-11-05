use crate::execution::config_model::{
    is_protected_selection_config_error_v1, parse_config_yaml, ProtectedSelectionConfigErrorV1,
    SubstrateConfig,
};
use anyhow::{anyhow, bail, Context, Result};
use std::fs;
use std::io;
use std::io::Write;
use std::path::{Path, PathBuf};
use tempfile::NamedTempFile;

#[derive(Debug, Clone)]
pub struct InstallConfig {
    pub world_enabled: bool,
    existed: bool,
}

impl InstallConfig {
    pub fn exists(&self) -> bool {
        self.existed
    }

    pub fn set_world_enabled(&mut self, enabled: bool) {
        self.world_enabled = enabled;
    }

    pub fn set_existed(&mut self, existed: bool) {
        self.existed = existed;
    }
}

impl Default for InstallConfig {
    fn default() -> Self {
        Self {
            world_enabled: true,
            existed: false,
        }
    }
}

pub fn load_install_config(path: &Path) -> Result<InstallConfig> {
    ensure_no_legacy_toml_config(path)?;
    match fs::read_to_string(path) {
        Ok(contents) => {
            let cfg = parse_config_yaml(path, &contents)?;
            Ok(InstallConfig {
                world_enabled: cfg.world.enabled,
                existed: true,
            })
        }
        Err(err) if err.kind() == io::ErrorKind::NotFound => Ok(InstallConfig::default()),
        Err(err) => Err(anyhow!("failed to read {}: {err}", path.display()))
            .context(ProtectedSelectionConfigErrorV1),
    }
}

/// Read-only refusal before installer discovery, provisioning or default recovery.
pub(super) fn refuse_protected_install_config(path: &Path) -> Result<()> {
    match fs::read_to_string(path) {
        Ok(raw) => match parse_config_yaml(path, &raw) {
            Err(err) if is_protected_selection_config_error_v1(&err) => Err(err),
            _ => Ok(()),
        },
        Err(err) if err.kind() == io::ErrorKind::NotFound => Ok(()),
        Err(err) => Err(anyhow!("failed to read {}: {err}", path.display()))
            .context(ProtectedSelectionConfigErrorV1),
    }
}

pub fn save_install_config(path: &Path, cfg: &InstallConfig) -> Result<()> {
    ensure_no_legacy_toml_config(path)?;
    let parent = path
        .parent()
        .ok_or_else(|| anyhow!("config path {} has no parent", path.display()))?;
    let mut current = match fs::read_to_string(path) {
        Ok(raw) => match parse_config_yaml(path, &raw) {
            Ok(current) => current,
            Err(err) if is_protected_selection_config_error_v1(&err) => return Err(err),
            Err(_) => SubstrateConfig::default(),
        },
        Err(err) if err.kind() == io::ErrorKind::NotFound => SubstrateConfig::default(),
        Err(err) => {
            return Err(anyhow!("failed to read {}: {err}", path.display()))
                .context(ProtectedSelectionConfigErrorV1)
        }
    };
    current.world.enabled = cfg.world_enabled;
    fs::create_dir_all(parent)
        .with_context(|| format!("failed to create directory for {}", path.display()))?;

    let mut tmp = NamedTempFile::new_in(parent)
        .with_context(|| format!("failed to create temp file near {}", path.display()))?;
    let body = serde_yaml::to_string(&current)
        .with_context(|| format!("failed to serialize config at {}", path.display()))?;
    tmp.write_all(body.as_bytes())?;
    tmp.flush()?;
    tmp.persist(path)
        .map_err(|e| anyhow!("failed to persist {}: {}", path.display(), e.error))?;
    crate::execution::config_model::invalidate_config_cache();
    Ok(())
}

fn ensure_no_legacy_toml_config(config_yaml_path: &Path) -> Result<()> {
    let legacy = config_yaml_path
        .parent()
        .map(|parent| parent.join("config.toml"))
        .unwrap_or_else(|| PathBuf::from("config.toml"));

    if !legacy.exists() {
        return Ok(());
    }

    let message = format!(
        "substrate: unsupported legacy TOML config detected:\n  - {}\nYAML config is now required:\n  - {}\nNext steps:\n  - Delete the TOML file and run `substrate config global init --force`\n  - Re-apply changes via `substrate config global set ...`\n",
        legacy.display(),
        config_yaml_path.display()
    );
    bail!("{message}");
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_yaml::Value as YamlValue;
    use std::fs;
    use tempfile::tempdir;

    fn write_config(path: &Path, contents: &str) {
        let parent = path.parent().expect("config path missing parent");
        fs::create_dir_all(parent).expect("create parent for config");
        fs::write(path, contents).expect("write config");
    }

    fn read_yaml(path: &Path) -> YamlValue {
        let raw = fs::read_to_string(path).expect("read config contents");
        serde_yaml::from_str(&raw).expect("parse config.yaml")
    }

    #[test]
    fn load_install_config_defaults_when_file_missing() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("config.yaml");

        let cfg = load_install_config(&path).expect("load default config");

        assert!(cfg.world_enabled);
        assert!(!cfg.exists());
    }

    #[test]
    fn load_install_config_errors_on_invalid_types() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("config.yaml");
        write_config(
            &path,
            "world:\n  enabled: \"yes\"\n  anchor_mode: workspace\n  anchor_path: \"\"\n  caged: true\npolicy:\n  mode: observe\nsync:\n  auto_sync: false\n  direction: from_world\n  conflict_policy: prefer_host\n  exclude: []\n",
        );

        let err = load_install_config(&path).expect_err("invalid type should error");
        let message = err.to_string();
        assert!(
            message.contains("enabled") || message.contains("boolean"),
            "unexpected error message: {message}"
        );
    }

    #[test]
    fn save_install_config_creates_or_updates_world_enabled() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("config.yaml");

        let mut cfg = InstallConfig::default();
        cfg.set_world_enabled(false);
        save_install_config(&path, &cfg).expect("save new config");

        let saved = read_yaml(&path);
        let root = saved.as_mapping().expect("root mapping missing after save");
        let world = root
            .get("world")
            .and_then(|value| value.as_mapping())
            .expect("world mapping missing after save");
        assert_eq!(
            world.get("enabled").and_then(|value| value.as_bool()),
            Some(false)
        );

        cfg.set_world_enabled(true);
        save_install_config(&path, &cfg).expect("update existing config");
        let saved = read_yaml(&path);
        let root = saved.as_mapping().expect("root mapping missing after save");
        let world = root
            .get("world")
            .and_then(|value| value.as_mapping())
            .expect("world mapping missing after save");
        assert_eq!(
            world.get("enabled").and_then(|value| value.as_bool()),
            Some(true)
        );
    }

    #[test]
    fn test_install_config_rejects_protected_input_without_materialization() {
        for raw in crate::execution::config_model::PROTECTED_SELECTION_CONFIG_FIXTURES_V1 {
            let dir = tempdir().unwrap();
            let path = dir.path().join("config.yaml");
            fs::write(&path, raw).unwrap();
            let before: Vec<_> = fs::read_dir(dir.path())
                .unwrap()
                .map(|e| e.unwrap().file_name())
                .collect();
            let error = load_install_config(&path).unwrap_err();
            assert!(
                is_protected_selection_config_error_v1(&error),
                "{error:#}; {raw}"
            );
            let error = save_install_config(&path, &InstallConfig::default()).unwrap_err();
            assert!(
                is_protected_selection_config_error_v1(&error),
                "{error:#}; {raw}"
            );
            assert_eq!(fs::read_to_string(&path).unwrap(), *raw);
            let after: Vec<_> = fs::read_dir(dir.path())
                .unwrap()
                .map(|e| e.unwrap().file_name())
                .collect();
            assert_eq!(before, after, "no materialization on protected refusal");
        }
    }

    #[test]
    fn test_install_config_writer_rechecks_changed_input() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("config.yaml");
        fs::write(&path, "world: {enabled: false}\n").unwrap();
        let mut config = load_install_config(&path).unwrap();
        config.set_world_enabled(true);
        let changed = crate::execution::config_model::PROTECTED_SELECTION_CONFIG_FIXTURES_V1[0];
        fs::write(&path, changed).unwrap();
        let error = save_install_config(&path, &config).unwrap_err();
        assert!(is_protected_selection_config_error_v1(&error));
        assert_eq!(fs::read_to_string(&path).unwrap(), changed);
        assert_eq!(fs::read_dir(dir.path()).unwrap().count(), 1);

        // Proven-absence legacy corruption keeps its existing recovery behavior.
        fs::write(&path, "world: {enabled: invalid}\n").unwrap();
        let error = load_install_config(&path).unwrap_err();
        assert!(!is_protected_selection_config_error_v1(&error));
        save_install_config(&path, &config).unwrap();
        assert!(load_install_config(&path).unwrap().world_enabled);

        // The current read must stand alone if bytes change after an early precheck.
        refuse_protected_install_config(&path).unwrap();
        let unreadable_yaml = b"\xffagents: {defaults: {selection: {harness: codex}}}\n";
        fs::write(&path, unreadable_yaml).unwrap();
        let error = load_install_config(&path).unwrap_err();
        assert!(is_protected_selection_config_error_v1(&error));
        let error = save_install_config(&path, &config).unwrap_err();
        assert!(is_protected_selection_config_error_v1(&error));
        assert_eq!(fs::read(&path).unwrap(), unreadable_yaml);
        assert_eq!(fs::read_dir(dir.path()).unwrap().count(), 1);

        let mixed = "world: {enabled: &flag false, caged: *flag}\nagents: {defaults: {selection: {harness: codex, provider: openai, connection_profile: 'built_in:codex-subscription', model: pinned-model}}}\n";
        fs::write(&path, mixed).unwrap();
        save_install_config(&path, &config).unwrap();
        let saved = parse_config_yaml(&path, &fs::read_to_string(&path).unwrap()).unwrap();
        assert!(saved.world.enabled);
        assert!(!saved.world.caged);
        assert_eq!(saved.agents.defaults.selection.model, "pinned-model");
    }
}
