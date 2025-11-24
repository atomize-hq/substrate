use anyhow::{anyhow, bail, Context, Result};
use serde_json::{Map, Value as JsonValue};
use std::fs;
use std::io;
use std::io::Write;
use std::path::{Path, PathBuf};
use tempfile::NamedTempFile;
use toml::value::{Table as TomlTable, Value as TomlValue};

#[derive(Debug, Clone)]
pub struct InstallConfig {
    pub world_enabled: bool,
    existed: bool,
    extras: TomlTable,
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
            extras: TomlTable::new(),
        }
    }
}

pub fn load_install_config(path: &Path) -> Result<InstallConfig> {
    match fs::read_to_string(path) {
        Ok(contents) => parse_toml_config(path, &contents),
        Err(err) if err.kind() == io::ErrorKind::NotFound => load_legacy_install_config(path),
        Err(err) => Err(anyhow!("failed to read {}: {err}", path.display())),
    }
}

pub fn save_install_config(path: &Path, cfg: &InstallConfig) -> Result<()> {
    let parent = path
        .parent()
        .ok_or_else(|| anyhow!("config path {} has no parent", path.display()))?;
    fs::create_dir_all(parent)
        .with_context(|| format!("failed to create directory for {}", path.display()))?;

    let mut data = cfg.extras.clone();
    let mut install_table = match data.remove("install") {
        Some(TomlValue::Table(table)) => table,
        Some(other) => {
            bail!(
                "install section in {} must be a table (found {})",
                path.display(),
                toml_type_name(&other)
            );
        }
        None => TomlTable::new(),
    };
    install_table.insert(
        "world_enabled".to_string(),
        TomlValue::Boolean(cfg.world_enabled),
    );
    data.insert("install".to_string(), TomlValue::Table(install_table));

    let mut tmp = NamedTempFile::new_in(parent)
        .with_context(|| format!("failed to create temp file near {}", path.display()))?;
    let body = toml::to_string_pretty(&TomlValue::Table(data))
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

fn parse_toml_config(path: &Path, contents: &str) -> Result<InstallConfig> {
    let mut raw: TomlTable =
        toml::from_str(contents).with_context(|| format!("invalid TOML in {}", path.display()))?;
    let mut install = match raw.remove("install") {
        Some(TomlValue::Table(table)) => table,
        Some(other) => {
            bail!(
                "install section in {} must be a table (found {})",
                path.display(),
                toml_type_name(&other)
            );
        }
        None => TomlTable::new(),
    };

    let world_enabled = match install.remove("world_enabled") {
        Some(TomlValue::Boolean(value)) => value,
        Some(other) => bail!(
            "install.world_enabled in {} must be a boolean (found {})",
            path.display(),
            toml_type_name(&other)
        ),
        None => true,
    };

    if !install.is_empty() {
        raw.insert("install".to_string(), TomlValue::Table(install));
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
        extras: json_map_to_toml(raw),
    })
}

fn legacy_config_path(new_path: &Path) -> PathBuf {
    new_path
        .parent()
        .map(|parent| parent.join("config.json"))
        .unwrap_or_else(|| PathBuf::from("config.json"))
}

fn json_map_to_toml(raw: Map<String, JsonValue>) -> TomlTable {
    let mut table = TomlTable::new();
    for (key, value) in raw {
        if let Some(converted) = json_to_toml(value) {
            table.insert(key, converted);
        }
    }
    table
}

fn json_to_toml(value: JsonValue) -> Option<TomlValue> {
    match value {
        JsonValue::Null => None,
        JsonValue::Bool(value) => Some(TomlValue::Boolean(value)),
        JsonValue::Number(num) => {
            if let Some(int) = num.as_i64() {
                Some(TomlValue::Integer(int))
            } else {
                num.as_f64().map(TomlValue::Float)
            }
        }
        JsonValue::String(value) => Some(TomlValue::String(value)),
        JsonValue::Array(values) => {
            let mut items = Vec::with_capacity(values.len());
            for value in values {
                if let Some(converted) = json_to_toml(value) {
                    items.push(converted);
                }
            }
            Some(TomlValue::Array(items))
        }
        JsonValue::Object(map) => {
            let mut table = TomlTable::new();
            for (key, value) in map {
                if let Some(converted) = json_to_toml(value) {
                    table.insert(key, converted);
                }
            }
            Some(TomlValue::Table(table))
        }
    }
}

fn toml_type_name(value: &TomlValue) -> &'static str {
    match value {
        TomlValue::Array(_) => "array",
        TomlValue::Boolean(_) => "boolean",
        TomlValue::Datetime(_) => "datetime",
        TomlValue::Float(_) => "float",
        TomlValue::Integer(_) => "integer",
        TomlValue::String(_) => "string",
        TomlValue::Table(_) => "table",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;
    use toml::Value as TomlValue;

    fn write_config(path: &Path, contents: &str) {
        let parent = path.parent().expect("config path missing parent");
        fs::create_dir_all(parent).expect("create parent for config");
        fs::write(path, contents).expect("write config");
    }

    fn read_toml(path: &Path) -> TomlValue {
        let raw = fs::read_to_string(path).expect("read config contents");
        toml::from_str(&raw).expect("parse config.toml")
    }

    #[test]
    fn load_install_config_defaults_when_file_missing() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("config.toml");

        let cfg = load_install_config(&path).expect("load default config");

        assert!(cfg.world_enabled);
        assert!(!cfg.exists());
        assert!(cfg.extras.is_empty(), "extras should be empty by default");
    }

    #[test]
    fn load_install_config_errors_on_non_bool_flag() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("config.toml");
        write_config(
            &path,
            r#"
[install]
world_enabled = "yes"
"#,
        );

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
        let path = dir.path().join("config.toml");
        write_config(
            &path,
            r#"
[install]
world_enabled = false
mode = "keep-me"

[world]
root_mode = "follow-cwd"
root_path = "/tmp/custom"
"#,
        );

        let mut cfg = load_install_config(&path).expect("load config with extras");
        assert!(cfg.exists());
        assert!(!cfg.world_enabled);

        cfg.set_world_enabled(true);
        save_install_config(&path, &cfg).expect("save updated config");

        let saved = read_toml(&path);
        let install = saved
            .get("install")
            .and_then(|value| value.as_table())
            .expect("install table missing after save");
        assert_eq!(
            install
                .get("world_enabled")
                .and_then(|value| value.as_bool()),
            Some(true)
        );
        assert_eq!(
            install.get("mode").and_then(|value| value.as_str()),
            Some("keep-me")
        );

        let world = saved
            .get("world")
            .and_then(|value| value.as_table())
            .expect("world table missing after save");
        assert_eq!(
            world.get("root_mode").and_then(|value| value.as_str()),
            Some("follow-cwd")
        );
        assert_eq!(
            world.get("root_path").and_then(|value| value.as_str()),
            Some("/tmp/custom")
        );
    }

    #[test]
    fn load_install_config_defaults_missing_flag_and_retains_sections() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("config.toml");
        write_config(
            &path,
            r#"
[world]
root_mode = "project"
"#,
        );

        let mut cfg = load_install_config(&path).expect("load config without install flag");
        assert!(cfg.exists());
        assert!(cfg.world_enabled, "missing flag should default to enabled");

        cfg.set_world_enabled(false);
        save_install_config(&path, &cfg).expect("persist updated config");

        let saved = read_toml(&path);
        let install = saved
            .get("install")
            .and_then(|value| value.as_table())
            .expect("install table should be created during save");
        assert_eq!(
            install
                .get("world_enabled")
                .and_then(|value| value.as_bool()),
            Some(false)
        );
        let world = saved
            .get("world")
            .and_then(|value| value.as_table())
            .expect("world table should be preserved");
        assert_eq!(
            world.get("root_mode").and_then(|value| value.as_str()),
            Some("project")
        );
    }
}
