use anyhow::{anyhow, bail, Context, Result};
use serde_json::{Map, Value as JsonValue};
use serde_yaml::{Mapping as YamlMapping, Value as YamlValue};
use std::fs;
use std::io;
use std::io::Write;
use std::path::{Path, PathBuf};
use tempfile::NamedTempFile;

#[derive(Debug, Clone)]
pub struct InstallConfig {
    pub world_enabled: bool,
    existed: bool,
    extras: YamlMapping,
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
            extras: YamlMapping::new(),
        }
    }
}

pub fn load_install_config(path: &Path) -> Result<InstallConfig> {
    ensure_no_legacy_toml_config(path)?;
    match fs::read_to_string(path) {
        Ok(contents) => parse_yaml_config(path, &contents),
        Err(err) if err.kind() == io::ErrorKind::NotFound => load_legacy_install_config(path),
        Err(err) => Err(anyhow!("failed to read {}: {err}", path.display())),
    }
}

pub fn save_install_config(path: &Path, cfg: &InstallConfig) -> Result<()> {
    ensure_no_legacy_toml_config(path)?;
    let parent = path
        .parent()
        .ok_or_else(|| anyhow!("config path {} has no parent", path.display()))?;
    fs::create_dir_all(parent)
        .with_context(|| format!("failed to create directory for {}", path.display()))?;

    let mut data = cfg.extras.clone();
    let install_key = YamlValue::String("install".to_string());
    let mut install_table = match data.remove(&install_key) {
        Some(YamlValue::Mapping(table)) => table,
        Some(other) => {
            bail!(
                "install section in {} must be a mapping (found {})",
                path.display(),
                yaml_type_name(&other)
            );
        }
        None => YamlMapping::new(),
    };
    install_table.insert(
        YamlValue::String("world_enabled".to_string()),
        YamlValue::Bool(cfg.world_enabled),
    );
    data.insert(install_key, YamlValue::Mapping(install_table));

    let mut tmp = NamedTempFile::new_in(parent)
        .with_context(|| format!("failed to create temp file near {}", path.display()))?;
    let body = serde_yaml::to_string(&YamlValue::Mapping(data))
        .with_context(|| format!("failed to serialize install config at {}", path.display()))?;
    tmp.write_all(body.as_bytes())?;
    tmp.flush()?;
    tmp.persist(path)
        .map_err(|e| anyhow!("failed to persist {}: {}", path.display(), e.error))?;
    Ok(())
}

fn load_legacy_install_config(new_path: &Path) -> Result<InstallConfig> {
    let legacy_path = legacy_config_path(new_path);
    match fs::read_to_string(&legacy_path) {
        Ok(contents) => parse_legacy_json(&legacy_path, &contents),
        Err(err) if err.kind() == io::ErrorKind::NotFound => Ok(InstallConfig::default()),
        Err(err) => Err(anyhow!(
            "failed to read legacy {}: {err}",
            legacy_path.display()
        )),
    }
}

fn parse_yaml_config(path: &Path, contents: &str) -> Result<InstallConfig> {
    let root: YamlValue = serde_yaml::from_str(contents)
        .with_context(|| format!("invalid YAML in {}", path.display()))?;
    let mut raw = match root {
        YamlValue::Mapping(map) => map,
        other => bail!(
            "config in {} must be a mapping (found {})",
            path.display(),
            yaml_type_name(&other)
        ),
    };

    let install_key = YamlValue::String("install".to_string());
    let mut install = match raw.remove(&install_key) {
        Some(YamlValue::Mapping(table)) => table,
        Some(other) => bail!(
            "install section in {} must be a mapping (found {})",
            path.display(),
            yaml_type_name(&other)
        ),
        None => YamlMapping::new(),
    };

    let world_enabled_key = YamlValue::String("world_enabled".to_string());
    let world_enabled = match install.remove(&world_enabled_key) {
        Some(YamlValue::Bool(value)) => value,
        Some(other) => bail!(
            "install.world_enabled in {} must be a boolean (found {})",
            path.display(),
            yaml_type_name(&other)
        ),
        None => true,
    };

    if !install.is_empty() {
        raw.insert(install_key, YamlValue::Mapping(install));
    }

    Ok(InstallConfig {
        world_enabled,
        existed: true,
        extras: raw,
    })
}

fn parse_legacy_json(path: &Path, contents: &str) -> Result<InstallConfig> {
    let mut raw: Map<String, JsonValue> = serde_json::from_str(contents)
        .with_context(|| format!("invalid JSON in {}", path.display()))?;
    let world_enabled = match raw.remove("world_enabled") {
        Some(JsonValue::Bool(value)) => value,
        Some(other) => bail!(
            "world_enabled in {} must be a boolean (found {other})",
            path.display()
        ),
        None => true,
    };

    Ok(InstallConfig {
        world_enabled,
        existed: true,
        extras: json_map_to_yaml(raw),
    })
}

fn legacy_config_path(new_path: &Path) -> PathBuf {
    new_path
        .parent()
        .map(|parent| parent.join("config.json"))
        .unwrap_or_else(|| PathBuf::from("config.json"))
}

fn json_map_to_yaml(raw: Map<String, JsonValue>) -> YamlMapping {
    let mut table = YamlMapping::new();
    for (key, value) in raw {
        if let Some(converted) = json_to_yaml(value) {
            table.insert(YamlValue::String(key), converted);
        }
    }
    table
}

fn json_to_yaml(value: JsonValue) -> Option<YamlValue> {
    match value {
        JsonValue::Null => Some(YamlValue::Null),
        JsonValue::Bool(value) => Some(YamlValue::Bool(value)),
        JsonValue::Number(num) => serde_yaml::to_value(num).ok(),
        JsonValue::String(value) => Some(YamlValue::String(value)),
        JsonValue::Array(values) => {
            let mut items = Vec::with_capacity(values.len());
            for value in values {
                if let Some(converted) = json_to_yaml(value) {
                    items.push(converted);
                }
            }
            Some(YamlValue::Sequence(items))
        }
        JsonValue::Object(map) => {
            let mut table = YamlMapping::new();
            for (key, value) in map {
                if let Some(converted) = json_to_yaml(value) {
                    table.insert(YamlValue::String(key), converted);
                }
            }
            Some(YamlValue::Mapping(table))
        }
    }
}

fn yaml_type_name(value: &YamlValue) -> &'static str {
    match value {
        YamlValue::Null => "null",
        YamlValue::Bool(_) => "boolean",
        YamlValue::Number(_) => "number",
        YamlValue::String(_) => "string",
        YamlValue::Sequence(_) => "sequence",
        YamlValue::Mapping(_) => "mapping",
        YamlValue::Tagged(_) => "tagged",
    }
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
        "substrate: unsupported legacy TOML config detected:\n  - {}\nYAML config is now required:\n  - {}\nNext steps:\n  - Delete the TOML file and run `substrate config init --force`\n  - Re-apply changes via `substrate config set ...`\n",
        legacy.display(),
        config_yaml_path.display()
    );
    bail!("{message}");
}

#[cfg(test)]
mod tests {
    use super::*;
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
        assert!(cfg.extras.is_empty(), "extras should be empty by default");
    }

    #[test]
    fn load_install_config_errors_on_non_bool_flag() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("config.yaml");
        write_config(&path, "install:\n  world_enabled: \"yes\"\n");

        let err = load_install_config(&path).expect_err("invalid flag should error");
        let message = err.to_string();
        assert!(
            message.contains("world_enabled"),
            "unexpected error message: {message}"
        );
    }

    #[test]
    fn load_and_save_install_config_preserves_unknown_keys() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("config.yaml");
        write_config(
            &path,
            "install:\n  world_enabled: false\n  mode: keep-me\nworld:\n  root_mode: follow-cwd\n  root_path: /tmp/custom\n",
        );

        let mut cfg = load_install_config(&path).expect("load config with extras");
        assert!(cfg.exists());
        assert!(!cfg.world_enabled);

        cfg.set_world_enabled(true);
        save_install_config(&path, &cfg).expect("save updated config");

        let saved = read_yaml(&path);
        let root = saved.as_mapping().expect("root mapping missing after save");

        let install = root
            .get(&YamlValue::String("install".to_string()))
            .and_then(|value| value.as_mapping())
            .expect("install mapping missing after save");
        assert_eq!(
            install
                .get(&YamlValue::String("world_enabled".to_string()))
                .and_then(|value| value.as_bool()),
            Some(true)
        );
        assert_eq!(
            install
                .get(&YamlValue::String("mode".to_string()))
                .and_then(|value| value.as_str()),
            Some("keep-me")
        );

        let world = root
            .get(&YamlValue::String("world".to_string()))
            .and_then(|value| value.as_mapping())
            .expect("world mapping missing after save");
        assert_eq!(
            world
                .get(&YamlValue::String("root_mode".to_string()))
                .and_then(|value| value.as_str()),
            Some("follow-cwd")
        );
        assert_eq!(
            world
                .get(&YamlValue::String("root_path".to_string()))
                .and_then(|value| value.as_str()),
            Some("/tmp/custom")
        );
    }

    #[test]
    fn load_install_config_defaults_missing_flag_and_retains_sections() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("config.yaml");
        write_config(&path, "world:\n  root_mode: project\n");

        let mut cfg = load_install_config(&path).expect("load config without install flag");
        assert!(cfg.exists());
        assert!(cfg.world_enabled, "missing flag should default to enabled");

        cfg.set_world_enabled(false);
        save_install_config(&path, &cfg).expect("persist updated config");

        let saved = read_yaml(&path);
        let root = saved.as_mapping().expect("root mapping missing after save");

        let install = root
            .get(&YamlValue::String("install".to_string()))
            .and_then(|value| value.as_mapping())
            .expect("install mapping missing after save");
        assert_eq!(
            install
                .get(&YamlValue::String("world_enabled".to_string()))
                .and_then(|value| value.as_bool()),
            Some(false)
        );

        let world = root
            .get(&YamlValue::String("world".to_string()))
            .and_then(|value| value.as_mapping())
            .expect("world mapping missing after save");
        assert_eq!(
            world
                .get(&YamlValue::String("root_mode".to_string()))
                .and_then(|value| value.as_str()),
            Some("project")
        );
    }
}
