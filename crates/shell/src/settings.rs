use anyhow::{anyhow, bail, Context, Result};
use std::env;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use substrate_common::paths as substrate_paths;
use substrate_common::WorldRootMode;
use toml::value::{Table as TomlTable, Value as TomlValue};

#[derive(Debug, Clone)]
pub struct WorldRootSettings {
    pub mode: WorldRootMode,
    pub path: PathBuf,
}

impl WorldRootSettings {
    pub fn effective_root(&self) -> PathBuf {
        match self.mode {
            WorldRootMode::FollowCwd => env::current_dir().unwrap_or_else(|_| self.path.clone()),
            _ => self.path.clone(),
        }
    }
}

#[derive(Default)]
struct PartialWorldRoot {
    mode: Option<WorldRootMode>,
    path: Option<PathBuf>,
}

pub(crate) fn resolve_world_root(
    cli_mode: Option<WorldRootMode>,
    cli_path: Option<PathBuf>,
    launch_dir: &Path,
) -> Result<WorldRootSettings> {
    let cli = PartialWorldRoot {
        mode: cli_mode,
        path: cli_path,
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

    if path.is_none() && mode != WorldRootMode::FollowCwd {
        path = Some(launch_dir.to_path_buf());
    }

    if mode == WorldRootMode::Custom && path.is_none() {
        bail!(
            "world root mode 'custom' requires a path (use --world-root-path, a config file, or SUBSTRATE_WORLD_ROOT_PATH)"
        );
    }

    let resolved_path = path.unwrap_or_else(|| launch_dir.to_path_buf());

    Ok(WorldRootSettings {
        mode,
        path: resolved_path,
    })
}

pub(crate) fn apply_world_root_env(settings: &WorldRootSettings) {
    env::set_var("SUBSTRATE_WORLD_ROOT_MODE", settings.mode.as_str());
    env::set_var(
        "SUBSTRATE_WORLD_ROOT_PATH",
        settings.path.to_string_lossy().to_string(),
    );
}

pub(crate) fn world_root_from_env() -> WorldRootSettings {
    let cwd = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let mode = env::var("SUBSTRATE_WORLD_ROOT_MODE")
        .ok()
        .and_then(|value| WorldRootMode::parse(&value))
        .unwrap_or(WorldRootMode::Project);
    let base_path = env::var("SUBSTRATE_WORLD_ROOT_PATH")
        .ok()
        .and_then(|value| {
            let trimmed = value.trim();
            (!trimmed.is_empty()).then(|| PathBuf::from(trimmed))
        })
        .unwrap_or_else(|| cwd.clone());
    let path = match mode {
        WorldRootMode::FollowCwd => cwd,
        _ => base_path,
    };
    WorldRootSettings { mode, path }
}

fn load_directory_settings(base_dir: &Path) -> Result<PartialWorldRoot> {
    let settings_path = base_dir.join(".substrate/settings.toml");
    load_world_settings_file(&settings_path)
}

fn load_global_settings() -> Result<PartialWorldRoot> {
    let path = substrate_paths::config_file()?;
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
    let mut raw: TomlTable =
        toml::from_str(contents).with_context(|| format!("invalid TOML in {}", path.display()))?;
    let Some(world) = raw.remove("world") else {
        return Ok(PartialWorldRoot::default());
    };

    let table = match world {
        TomlValue::Table(table) => table,
        other => {
            bail!(
                "world section in {} must be a table (found {})",
                path.display(),
                toml_type_name(&other)
            );
        }
    };

    let mode = match table.get("root_mode") {
        Some(TomlValue::String(value)) => Some(parse_mode(value, path, "world.root_mode")?),
        Some(other) => {
            bail!(
                "world.root_mode in {} must be a string (found {})",
                path.display(),
                toml_type_name(other)
            );
        }
        None => None,
    };

    let path_value = match table.get("root_path") {
        Some(TomlValue::String(value)) => {
            let trimmed = value.trim();
            (!trimmed.is_empty()).then(|| PathBuf::from(trimmed))
        }
        Some(other) => {
            bail!(
                "world.root_path in {} must be a string (found {})",
                path.display(),
                toml_type_name(other)
            );
        }
        None => None,
    };

    Ok(PartialWorldRoot {
        mode,
        path: path_value,
    })
}

fn load_env_settings() -> Result<PartialWorldRoot> {
    let mut partial = PartialWorldRoot::default();

    if let Ok(mode) = env::var("SUBSTRATE_WORLD_ROOT_MODE") {
        partial.mode = Some(
            WorldRootMode::parse(&mode).ok_or_else(|| {
                anyhow!(
                    "SUBSTRATE_WORLD_ROOT_MODE must be one of project, follow-cwd, or custom (found {})",
                    mode
                )
            })?,
        );
    }

    if let Ok(path) = env::var("SUBSTRATE_WORLD_ROOT_PATH") {
        let trimmed = path.trim();
        if !trimmed.is_empty() {
            partial.path = Some(PathBuf::from(trimmed));
        }
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
