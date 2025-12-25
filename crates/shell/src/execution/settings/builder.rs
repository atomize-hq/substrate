//! Resolve world root configuration from CLI, env, and settings files.

use super::{ANCHOR_MODE_ENV, ANCHOR_PATH_ENV, LEGACY_ROOT_MODE_ENV, LEGACY_ROOT_PATH_ENV};
use anyhow::{anyhow, bail, Context, Result};
use serde_yaml::Value as YamlValue;
use std::env;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use substrate_common::paths as substrate_paths;
use substrate_common::WorldRootMode;

#[derive(Debug, Clone)]
pub struct WorldRootSettings {
    pub mode: WorldRootMode,
    pub path: PathBuf,
    pub caged: bool,
}

impl WorldRootSettings {
    pub fn effective_root(&self) -> PathBuf {
        match self.mode {
            WorldRootMode::FollowCwd => env::current_dir().unwrap_or_else(|_| self.path.clone()),
            _ => self.path.clone(),
        }
    }

    pub fn anchor_root(&self, current_dir: &Path) -> PathBuf {
        match self.mode {
            WorldRootMode::FollowCwd => current_dir.to_path_buf(),
            _ => self.path.clone(),
        }
    }
}

#[derive(Default)]
struct PartialWorldRoot {
    mode: Option<WorldRootMode>,
    path: Option<PathBuf>,
    caged: Option<bool>,
}

pub(crate) fn resolve_world_root(
    cli_mode: Option<WorldRootMode>,
    cli_path: Option<PathBuf>,
    cli_caged: Option<bool>,
    launch_dir: &Path,
) -> Result<WorldRootSettings> {
    let cli = PartialWorldRoot {
        mode: cli_mode,
        path: cli_path,
        caged: cli_caged,
    };
    let dir_settings = load_directory_settings(launch_dir)?;
    let global_settings = load_global_settings()?;
    let env_settings = load_env_settings()?;

    let mode = cli
        .mode
        .or(dir_settings.mode)
        .or(global_settings.mode)
        .or(env_settings.mode)
        .unwrap_or(WorldRootMode::Project);

    let mut path = cli
        .path
        .or(dir_settings.path)
        .or(global_settings.path)
        .or(env_settings.path);

    if mode == WorldRootMode::FollowCwd {
        path = Some(launch_dir.to_path_buf());
    } else if path.is_none() {
        if mode == WorldRootMode::Custom {
            bail!(
                "world root mode 'custom' requires a path (use --anchor-path/--world-root-path, a config file, or SUBSTRATE_ANCHOR_PATH/SUBSTRATE_WORLD_ROOT_PATH)"
            );
        }
        path = Some(launch_dir.to_path_buf());
    }

    let resolved_path = path.unwrap_or_else(|| launch_dir.to_path_buf());
    let caged = cli
        .caged
        .or(dir_settings.caged)
        .or(global_settings.caged)
        .or(env_settings.caged)
        .unwrap_or(true);

    Ok(WorldRootSettings {
        mode,
        path: resolved_path,
        caged,
    })
}

fn load_directory_settings(base_dir: &Path) -> Result<PartialWorldRoot> {
    let legacy_path = base_dir.join(".substrate/settings.toml");
    let settings_path = base_dir.join(".substrate/settings.yaml");
    ensure_no_legacy_toml_files(&[legacy_path], &[settings_path.clone()])?;
    load_world_settings_file(&settings_path)
}

fn load_global_settings() -> Result<PartialWorldRoot> {
    let path = substrate_paths::config_file()?;
    let legacy = substrate_paths::substrate_home()?.join("config.toml");
    ensure_no_legacy_toml_files(&[legacy], &[path.clone()])?;
    load_world_settings_file(&path)
}

fn load_world_settings_file(path: &Path) -> Result<PartialWorldRoot> {
    match fs::read_to_string(path) {
        Ok(contents) => parse_world_settings(path, &contents),
        Err(err) if err.kind() == io::ErrorKind::NotFound => Ok(PartialWorldRoot::default()),
        Err(err) => Err(anyhow!("failed to read {}: {err}", path.display())),
    }
}

fn parse_world_settings(path: &Path, contents: &str) -> Result<PartialWorldRoot> {
    let raw: YamlValue = serde_yaml::from_str(contents)
        .with_context(|| format!("invalid YAML in {}", path.display()))?;

    let root = match raw {
        YamlValue::Mapping(map) => map,
        other => bail!(
            "settings in {} must be a mapping (found {})",
            path.display(),
            yaml_type_name(&other)
        ),
    };

    let Some(world) = root.get(&YamlValue::String("world".to_string())) else {
        return Ok(PartialWorldRoot::default());
    };

    let table = match world {
        YamlValue::Mapping(map) => map,
        other => bail!(
            "world section in {} must be a mapping (found {})",
            path.display(),
            yaml_type_name(other)
        ),
    };

    let (mode_value, mode_key) = match table.get(&YamlValue::String("anchor_mode".to_string())) {
        Some(value) => (Some(value), "world.anchor_mode"),
        None => match table.get(&YamlValue::String("root_mode".to_string())) {
            Some(value) => (Some(value), "world.root_mode"),
            None => (None, ""),
        },
    };

    let mode = match mode_value {
        Some(value) => match value.as_str() {
            Some(raw) => Some(parse_mode(raw, path, mode_key)?),
            None => bail!(
                "{} in {} must be a string (found {})",
                mode_key,
                path.display(),
                yaml_type_name(value)
            ),
        },
        None => None,
    };

    let (path_value, path_key) = match table.get(&YamlValue::String("anchor_path".to_string())) {
        Some(value) => (Some(value), "world.anchor_path"),
        None => match table.get(&YamlValue::String("root_path".to_string())) {
            Some(value) => (Some(value), "world.root_path"),
            None => (None, ""),
        },
    };

    let path_value = match path_value {
        Some(value) => match value.as_str() {
            Some(raw) => {
                let trimmed = raw.trim();
                (!trimmed.is_empty()).then(|| PathBuf::from(trimmed))
            }
            None => bail!(
                "{} in {} must be a string (found {})",
                path_key,
                path.display(),
                yaml_type_name(value)
            ),
        },
        None => None,
    };

    let caged = match table.get(&YamlValue::String("caged".to_string())) {
        Some(value) => match value.as_bool() {
            Some(flag) => Some(flag),
            None => bail!(
                "world.caged in {} must be a boolean (found {})",
                path.display(),
                yaml_type_name(value)
            ),
        },
        None => None,
    };

    Ok(PartialWorldRoot {
        mode,
        path: path_value,
        caged,
    })
}

fn load_env_settings() -> Result<PartialWorldRoot> {
    let mut partial = PartialWorldRoot::default();

    if let Some((key, mode)) = first_env_value(&[ANCHOR_MODE_ENV, LEGACY_ROOT_MODE_ENV]) {
        partial.mode = Some(parse_env_mode(key, &mode)?);
    }

    if let Some((_, path)) = first_env_value(&[ANCHOR_PATH_ENV, LEGACY_ROOT_PATH_ENV]) {
        let trimmed = path.trim();
        if !trimmed.is_empty() {
            partial.path = Some(PathBuf::from(trimmed));
        }
    }

    if let Ok(raw) = env::var("SUBSTRATE_CAGED") {
        let parsed = parse_bool_env("SUBSTRATE_CAGED", &raw)?;
        partial.caged = Some(parsed);
    }

    Ok(partial)
}

fn parse_mode(value: &str, path: &Path, key: &str) -> Result<WorldRootMode> {
    WorldRootMode::parse(value).ok_or_else(|| {
        anyhow!(
            "{} in {} must be one of project, follow-cwd, or custom (found {})",
            key,
            path.display(),
            value
        )
    })
}

fn parse_bool_env(key: &str, raw: &str) -> Result<bool> {
    parse_bool_flag(raw).ok_or_else(|| {
        anyhow!(
            "{} must be true/false/1/0/yes/no (found {})",
            key,
            raw.trim()
        )
    })
}

pub(crate) fn parse_bool_flag(raw: &str) -> Option<bool> {
    match raw.trim().to_ascii_lowercase().as_str() {
        "1" | "true" | "yes" | "on" => Some(true),
        "0" | "false" | "no" | "off" => Some(false),
        _ => None,
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

fn ensure_no_legacy_toml_files(legacy_paths: &[PathBuf], yaml_paths: &[PathBuf]) -> Result<()> {
    let present: Vec<&PathBuf> = legacy_paths.iter().filter(|path| path.exists()).collect();
    if present.is_empty() {
        return Ok(());
    }

    let mut message = "substrate: unsupported legacy TOML settings detected:\n".to_string();
    for path in present {
        message.push_str(&format!("  - {}\n", path.display()));
    }
    message.push_str("YAML settings are now required:\n");
    for path in yaml_paths {
        message.push_str(&format!("  - {}\n", path.display()));
    }
    message.push_str("Next steps:\n");
    message.push_str("  - Delete or rename the TOML file(s)\n");
    message
        .push_str("  - Create the YAML file(s) (global config: `substrate config init --force`)\n");
    bail!("{message}");
}

pub(super) fn first_env_value(keys: &[&'static str]) -> Option<(&'static str, String)> {
    keys.iter()
        .find_map(|&key| env::var(key).ok().map(|value| (key, value)))
}

fn parse_env_mode(key: &str, raw: &str) -> Result<WorldRootMode> {
    WorldRootMode::parse(raw).ok_or_else(|| {
        anyhow!(
            "{} must be one of project, follow-cwd, or custom (found {})",
            key,
            raw.trim()
        )
    })
}
