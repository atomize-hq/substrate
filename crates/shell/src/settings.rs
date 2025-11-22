use anyhow::{anyhow, bail, Context, Result};
use std::env;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use substrate_common::paths as substrate_paths;
use substrate_common::WorldRootMode;
use toml::value::{Table as TomlTable, Value as TomlValue};

const ANCHOR_MODE_ENV: &str = "SUBSTRATE_ANCHOR_MODE";
const ANCHOR_PATH_ENV: &str = "SUBSTRATE_ANCHOR_PATH";
const LEGACY_ROOT_MODE_ENV: &str = "SUBSTRATE_WORLD_ROOT_MODE";
const LEGACY_ROOT_PATH_ENV: &str = "SUBSTRATE_WORLD_ROOT_PATH";

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

pub(crate) fn apply_world_root_env(settings: &WorldRootSettings) {
    let mode = settings.mode.as_str();
    let path = settings.path.to_string_lossy().to_string();
    env::set_var(ANCHOR_MODE_ENV, mode);
    env::set_var(LEGACY_ROOT_MODE_ENV, mode);
    env::set_var(ANCHOR_PATH_ENV, &path);
    env::set_var(LEGACY_ROOT_PATH_ENV, &path);
    env::set_var("SUBSTRATE_CAGED", if settings.caged { "1" } else { "0" });
}

pub(crate) fn world_root_from_env() -> WorldRootSettings {
    let cwd = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let mode = first_env_value(&[ANCHOR_MODE_ENV, LEGACY_ROOT_MODE_ENV])
        .and_then(|(_, value)| WorldRootMode::parse(&value))
        .unwrap_or(WorldRootMode::Project);
    let base_path = first_env_value(&[ANCHOR_PATH_ENV, LEGACY_ROOT_PATH_ENV])
        .and_then(|(_, value)| {
            let trimmed = value.trim();
            (!trimmed.is_empty()).then(|| PathBuf::from(trimmed))
        })
        .unwrap_or_else(|| cwd.clone());
    let path = match mode {
        WorldRootMode::FollowCwd => cwd,
        _ => base_path,
    };
    let caged = env::var("SUBSTRATE_CAGED")
        .ok()
        .and_then(|value| parse_bool_flag(&value))
        .unwrap_or(true);
    WorldRootSettings { mode, path, caged }
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

    let (mode_value, mode_key) = match table.get("anchor_mode") {
        Some(value) => (Some(value), "world.anchor_mode"),
        None => match table.get("root_mode") {
            Some(value) => (Some(value), "world.root_mode"),
            None => (None, ""),
        },
    };

    let mode = match mode_value {
        Some(TomlValue::String(value)) => Some(parse_mode(value, path, mode_key)?),
        Some(other) => {
            bail!(
                "{} in {} must be a string (found {})",
                mode_key,
                path.display(),
                toml_type_name(other)
            );
        }
        None => None,
    };

    let (path_value, path_key) = match table.get("anchor_path") {
        Some(value) => (Some(value), "world.anchor_path"),
        None => match table.get("root_path") {
            Some(value) => (Some(value), "world.root_path"),
            None => (None, ""),
        },
    };

    let path_value = match path_value {
        Some(TomlValue::String(value)) => {
            let trimmed = value.trim();
            (!trimmed.is_empty()).then(|| PathBuf::from(trimmed))
        }
        Some(other) => {
            bail!(
                "{} in {} must be a string (found {})",
                path_key,
                path.display(),
                toml_type_name(other)
            );
        }
        None => None,
    };

    let caged = match table.get("caged") {
        Some(TomlValue::Boolean(value)) => Some(*value),
        Some(other) => {
            bail!(
                "world.caged in {} must be a boolean (found {})",
                path.display(),
                toml_type_name(other)
            );
        }
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

fn parse_bool_flag(raw: &str) -> Option<bool> {
    match raw.trim().to_ascii_lowercase().as_str() {
        "1" | "true" | "yes" | "on" => Some(true),
        "0" | "false" | "no" | "off" => Some(false),
        _ => None,
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

fn first_env_value(keys: &[&'static str]) -> Option<(&'static str, String)> {
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

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;
    use std::path::{Path, PathBuf};
    use tempfile::TempDir;

    struct EnvGuard {
        saved: Vec<(&'static str, Option<String>)>,
    }

    impl EnvGuard {
        fn new(vars: Vec<(&'static str, Option<String>)>) -> Self {
            let mut saved = Vec::new();
            for (key, value) in vars {
                saved.push((key, std::env::var(key).ok()));
                match value {
                    Some(val) => std::env::set_var(key, val),
                    None => std::env::remove_var(key),
                }
            }
            Self { saved }
        }
    }

    impl Drop for EnvGuard {
        fn drop(&mut self) {
            for (key, value) in self.saved.iter().rev() {
                match value {
                    Some(val) => std::env::set_var(key, val),
                    None => std::env::remove_var(key),
                }
            }
        }
    }

    struct CwdGuard {
        original: PathBuf,
    }

    impl CwdGuard {
        fn new() -> Self {
            let original = std::env::current_dir().expect("capture cwd");
            Self { original }
        }
    }

    impl Drop for CwdGuard {
        fn drop(&mut self) {
            let _ = std::env::set_current_dir(&self.original);
        }
    }

    fn write_world_settings(
        path: &Path,
        mode: &str,
        root_path: Option<&Path>,
        caged: Option<bool>,
    ) {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).expect("create settings parent");
        }
        let mut body = format!("[world]\nroot_mode = \"{mode}\"\n");
        if let Some(root) = root_path {
            body.push_str(&format!("root_path = \"{}\"\n", root.display()));
        }
        if let Some(flag) = caged {
            body.push_str(&format!(
                "caged = {}\n",
                if flag { "true" } else { "false" }
            ));
        }
        std::fs::write(path, body).expect("write settings file");
    }

    fn setup_substrate_home(temp: &TempDir) -> PathBuf {
        let home = temp.path().join("home").join(".substrate");
        std::fs::create_dir_all(&home).expect("create substrate home");
        home
    }

    #[test]
    #[serial]
    fn resolve_world_root_defaults_to_launch_dir_project() {
        let temp = TempDir::new().unwrap();
        let home = setup_substrate_home(&temp);
        let _env = EnvGuard::new(vec![
            ("SUBSTRATE_HOME", Some(home.display().to_string())),
            ("SUBSTRATE_WORLD_ROOT_MODE", None),
            ("SUBSTRATE_WORLD_ROOT_PATH", None),
            ("SUBSTRATE_CAGED", None),
        ]);
        let launch_dir = temp.path().join("workspace");
        std::fs::create_dir_all(&launch_dir).unwrap();

        let settings = resolve_world_root(None, None, None, &launch_dir).expect("default settings");

        assert_eq!(settings.mode, WorldRootMode::Project);
        assert_eq!(settings.path, launch_dir);
        assert!(settings.caged);
    }

    #[test]
    #[serial]
    fn resolve_world_root_respects_env_when_no_configs() {
        let temp = TempDir::new().unwrap();
        let home = setup_substrate_home(&temp);
        let _env = EnvGuard::new(vec![
            ("SUBSTRATE_HOME", Some(home.display().to_string())),
            ("SUBSTRATE_WORLD_ROOT_MODE", Some("custom".into())),
            ("SUBSTRATE_WORLD_ROOT_PATH", Some("/env/root".into())),
            ("SUBSTRATE_CAGED", Some("false".into())),
        ]);
        let launch_dir = temp.path().join("project");
        std::fs::create_dir_all(&launch_dir).unwrap();

        let settings = resolve_world_root(None, None, None, &launch_dir).expect("env settings");

        assert_eq!(settings.mode, WorldRootMode::Custom);
        assert_eq!(settings.path, PathBuf::from("/env/root"));
        assert!(!settings.caged);
    }

    #[test]
    #[serial]
    fn resolve_world_root_prefers_global_config_over_env() {
        let temp = TempDir::new().unwrap();
        let home = setup_substrate_home(&temp);
        let config_path = home.join("config.toml");
        write_world_settings(&config_path, "follow-cwd", None, Some(true));
        let _env = EnvGuard::new(vec![
            ("SUBSTRATE_HOME", Some(home.display().to_string())),
            ("SUBSTRATE_WORLD_ROOT_MODE", Some("custom".into())),
            ("SUBSTRATE_WORLD_ROOT_PATH", Some("/env/root".into())),
            ("SUBSTRATE_CAGED", Some("false".into())),
        ]);
        let launch_dir = temp.path().join("project");
        std::fs::create_dir_all(&launch_dir).unwrap();

        let settings = resolve_world_root(None, None, None, &launch_dir).expect("global settings");

        assert_eq!(settings.mode, WorldRootMode::FollowCwd);
        assert_eq!(settings.path, launch_dir);
        assert!(settings.caged);
    }

    #[test]
    #[serial]
    fn resolve_world_root_prefers_directory_config_over_global_and_env() {
        let temp = TempDir::new().unwrap();
        let home = setup_substrate_home(&temp);
        let config_path = home.join("config.toml");
        write_world_settings(
            &config_path,
            "project",
            Some(Path::new("/global/root")),
            Some(true),
        );
        let _env = EnvGuard::new(vec![
            ("SUBSTRATE_HOME", Some(home.display().to_string())),
            ("SUBSTRATE_WORLD_ROOT_MODE", Some("custom".into())),
            ("SUBSTRATE_WORLD_ROOT_PATH", Some("/env/root".into())),
            ("SUBSTRATE_CAGED", Some("false".into())),
        ]);
        let launch_dir = temp.path().join("project");
        std::fs::create_dir_all(launch_dir.join(".substrate")).unwrap();
        let dir_settings = launch_dir.join(".substrate/settings.toml");
        write_world_settings(
            &dir_settings,
            "custom",
            Some(Path::new("/dir/root")),
            Some(false),
        );

        let settings =
            resolve_world_root(None, None, None, &launch_dir).expect("dir config settings");

        assert_eq!(settings.mode, WorldRootMode::Custom);
        assert_eq!(settings.path, PathBuf::from("/dir/root"));
        assert!(!settings.caged);
    }

    #[test]
    #[serial]
    fn resolve_world_root_prefers_cli_over_all_other_sources() {
        let temp = TempDir::new().unwrap();
        let home = setup_substrate_home(&temp);
        let config_path = home.join("config.toml");
        write_world_settings(
            &config_path,
            "project",
            Some(Path::new("/global/root")),
            Some(true),
        );
        let _env = EnvGuard::new(vec![
            ("SUBSTRATE_HOME", Some(home.display().to_string())),
            ("SUBSTRATE_WORLD_ROOT_MODE", Some("project".into())),
            ("SUBSTRATE_WORLD_ROOT_PATH", Some("/env/root".into())),
            ("SUBSTRATE_CAGED", Some("false".into())),
        ]);
        let launch_dir = temp.path().join("project");
        std::fs::create_dir_all(launch_dir.join(".substrate")).unwrap();
        let dir_settings = launch_dir.join(".substrate/settings.toml");
        write_world_settings(
            &dir_settings,
            "custom",
            Some(Path::new("/dir/root")),
            Some(false),
        );
        let cli_path = PathBuf::from("/cli/root");

        let settings = resolve_world_root(
            Some(WorldRootMode::Custom),
            Some(cli_path.clone()),
            Some(true),
            &launch_dir,
        )
        .expect("cli settings");

        assert_eq!(settings.mode, WorldRootMode::Custom);
        assert_eq!(settings.path, cli_path);
        assert!(settings.caged);
    }

    #[test]
    #[serial]
    fn resolve_world_root_requires_path_for_custom_mode() {
        let temp = TempDir::new().unwrap();
        let home = setup_substrate_home(&temp);
        let _env = EnvGuard::new(vec![
            ("SUBSTRATE_HOME", Some(home.display().to_string())),
            ("SUBSTRATE_WORLD_ROOT_MODE", None),
            ("SUBSTRATE_WORLD_ROOT_PATH", None),
            ("SUBSTRATE_CAGED", None),
        ]);
        let launch_dir = temp.path().join("project");
        std::fs::create_dir_all(&launch_dir).unwrap();

        let err = resolve_world_root(Some(WorldRootMode::Custom), None, None, &launch_dir)
            .expect_err("custom without path should error");
        let message = err.to_string();
        assert!(
            message.contains("requires a path"),
            "unexpected error message: {message}"
        );
    }

    #[test]
    #[serial]
    fn effective_root_uses_current_directory_for_follow_mode() {
        let temp = TempDir::new().unwrap();
        let home = setup_substrate_home(&temp);
        let _env = EnvGuard::new(vec![("SUBSTRATE_HOME", Some(home.display().to_string()))]);
        let target_cwd = temp.path().join("changing");
        std::fs::create_dir_all(&target_cwd).unwrap();
        let _cwd_guard = CwdGuard::new();
        std::env::set_current_dir(&target_cwd).unwrap();

        let settings = WorldRootSettings {
            mode: WorldRootMode::FollowCwd,
            path: PathBuf::from("/should/be/ignored"),
            caged: true,
        };

        assert_eq!(settings.effective_root(), target_cwd);
    }
}
