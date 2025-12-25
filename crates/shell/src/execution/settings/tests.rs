use super::*;
use serial_test::serial;
use std::path::{Path, PathBuf};
use substrate_common::WorldRootMode;
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

fn write_world_settings(path: &Path, mode: &str, root_path: Option<&Path>, caged: Option<bool>) {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).expect("create settings parent");
    }
    let mut body = format!("world:\n  root_mode: {mode}\n");
    if let Some(root) = root_path {
        body.push_str(&format!("  root_path: \"{}\"\n", root.display()));
    }
    if let Some(flag) = caged {
        body.push_str(&format!(
            "  caged: {}\n",
            if flag { "true" } else { "false" }
        ));
    }
    std::fs::write(path, body).expect("write settings file");
}

fn write_anchor_world_settings(
    path: &Path,
    anchor_mode: &str,
    anchor_path: Option<&Path>,
    legacy_mode: Option<&str>,
    legacy_path: Option<&Path>,
    caged: Option<bool>,
) {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).expect("create anchor settings parent");
    }
    let mut body = format!("world:\n  anchor_mode: {anchor_mode}\n");
    if let Some(anchor) = anchor_path {
        body.push_str(&format!("  anchor_path: \"{}\"\n", anchor.display()));
    }
    if let Some(mode) = legacy_mode {
        body.push_str(&format!("  root_mode: {mode}\n"));
    }
    if let Some(root) = legacy_path {
        body.push_str(&format!("  root_path: \"{}\"\n", root.display()));
    }
    if let Some(flag) = caged {
        body.push_str(&format!(
            "  caged: {}\n",
            if flag { "true" } else { "false" }
        ));
    }
    std::fs::write(path, body).expect("write anchor settings file");
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
        ("SUBSTRATE_ANCHOR_MODE", None),
        ("SUBSTRATE_ANCHOR_PATH", None),
        ("SUBSTRATE_WORLD_ROOT_MODE", None),
        ("SUBSTRATE_WORLD_ROOT_PATH", None),
        ("SUBSTRATE_CAGED", None),
    ]);
    assert!(std::env::var("SUBSTRATE_ANCHOR_MODE").is_err());
    assert!(std::env::var("SUBSTRATE_ANCHOR_PATH").is_err());
    assert!(std::env::var("SUBSTRATE_WORLD_ROOT_MODE").is_err());
    assert!(std::env::var("SUBSTRATE_WORLD_ROOT_PATH").is_err());
    let launch_dir = temp.path().join("workspace");
    std::fs::create_dir_all(&launch_dir).unwrap();
    let dir_settings = launch_dir.join(".substrate/settings.yaml");
    let global_settings = home.join("config.yaml");
    assert!(
        !dir_settings.exists(),
        "unexpected directory settings file at {}",
        dir_settings.display()
    );
    assert!(
        !global_settings.exists(),
        "unexpected global config at {}",
        global_settings.display()
    );

    let cwd = std::env::current_dir().unwrap();
    let env_settings = world_root_from_env();
    assert_eq!(env_settings.mode, WorldRootMode::Project);
    assert_eq!(env_settings.path, cwd);

    let settings = resolve_world_root(None, None, None, &launch_dir).expect("default settings");

    assert_eq!(settings.mode, WorldRootMode::Project);
    assert_eq!(settings.path, launch_dir);
    assert!(settings.caged);
}

#[test]
#[serial]
fn resolve_world_root_refuses_legacy_settings_toml() {
    let temp = TempDir::new().unwrap();
    let home = setup_substrate_home(&temp);
    let _env = EnvGuard::new(vec![("SUBSTRATE_HOME", Some(home.display().to_string()))]);

    let launch_dir = temp.path().join("workspace");
    std::fs::create_dir_all(launch_dir.join(".substrate")).unwrap();
    let legacy_settings = launch_dir.join(".substrate/settings.toml");
    std::fs::write(&legacy_settings, "[world]\nroot_mode = \"project\"\n").unwrap();

    let err = resolve_world_root(None, None, None, &launch_dir)
        .expect_err("legacy settings.toml should be rejected");
    let message = err.to_string();

    assert!(
        message.contains("unsupported legacy TOML settings detected"),
        "unexpected error message: {message}"
    );
    assert!(
        message.contains(&legacy_settings.display().to_string()),
        "error message missing legacy path: {message}"
    );
    assert!(
        message.contains(
            &launch_dir
                .join(".substrate/settings.yaml")
                .display()
                .to_string()
        ),
        "error message missing yaml path: {message}"
    );
}

#[test]
#[serial]
fn resolve_world_root_respects_env_when_no_configs() {
    let temp = TempDir::new().unwrap();
    let home = setup_substrate_home(&temp);
    let _env = EnvGuard::new(vec![
        ("SUBSTRATE_HOME", Some(home.display().to_string())),
        ("SUBSTRATE_ANCHOR_MODE", None),
        ("SUBSTRATE_ANCHOR_PATH", None),
        ("SUBSTRATE_WORLD_ROOT_MODE", Some("custom".into())),
        ("SUBSTRATE_WORLD_ROOT_PATH", Some("/env/root".into())),
        ("SUBSTRATE_CAGED", Some("false".into())),
    ]);
    assert!(std::env::var("SUBSTRATE_ANCHOR_MODE").is_err());
    assert!(std::env::var("SUBSTRATE_ANCHOR_PATH").is_err());
    assert_eq!(
        std::env::var("SUBSTRATE_WORLD_ROOT_MODE").as_deref(),
        Ok("custom")
    );
    assert_eq!(
        std::env::var("SUBSTRATE_WORLD_ROOT_PATH").as_deref(),
        Ok("/env/root")
    );
    let launch_dir = temp.path().join("project");
    std::fs::create_dir_all(&launch_dir).unwrap();
    let dir_settings = launch_dir.join(".substrate/settings.yaml");
    let global_settings = home.join("config.yaml");
    assert!(
        !dir_settings.exists(),
        "unexpected directory settings file at {}",
        dir_settings.display()
    );
    assert!(
        !global_settings.exists(),
        "unexpected global config at {}",
        global_settings.display()
    );

    let env_settings = world_root_from_env();
    assert_eq!(env_settings.mode, WorldRootMode::Custom);
    assert_eq!(env_settings.path, PathBuf::from("/env/root"));
    assert!(!env_settings.caged);

    let settings = resolve_world_root(None, None, None, &launch_dir).expect("env settings");

    assert_eq!(settings.mode, WorldRootMode::Custom);
    assert_eq!(settings.path, PathBuf::from("/env/root"));
    assert!(!settings.caged);
}

#[test]
#[serial]
fn world_root_from_env_prefers_anchor_over_legacy_keys() {
    let temp = TempDir::new().unwrap();
    let home = setup_substrate_home(&temp);
    let _env = EnvGuard::new(vec![
        ("SUBSTRATE_HOME", Some(home.display().to_string())),
        ("SUBSTRATE_ANCHOR_MODE", Some("custom".into())),
        ("SUBSTRATE_WORLD_ROOT_MODE", Some("follow-cwd".into())),
        ("SUBSTRATE_ANCHOR_PATH", Some("/env/anchor".into())),
        ("SUBSTRATE_WORLD_ROOT_PATH", Some("/env/legacy".into())),
        ("SUBSTRATE_CAGED", Some("0".into())),
    ]);
    let launch_dir = temp.path().join("project");
    std::fs::create_dir_all(&launch_dir).unwrap();

    let settings = resolve_world_root(None, None, None, &launch_dir).expect("anchor env settings");

    assert_eq!(settings.mode, WorldRootMode::Custom);
    assert_eq!(settings.path, PathBuf::from("/env/anchor"));
    assert!(!settings.caged);
}

#[test]
#[serial]
fn resolve_world_root_prefers_global_config_over_env() {
    let temp = TempDir::new().unwrap();
    let home = setup_substrate_home(&temp);
    let config_path = home.join("config.yaml");
    write_world_settings(&config_path, "follow-cwd", None, Some(true));
    let _env = EnvGuard::new(vec![
        ("SUBSTRATE_HOME", Some(home.display().to_string())),
        ("SUBSTRATE_ANCHOR_MODE", None),
        ("SUBSTRATE_ANCHOR_PATH", None),
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
    let config_path = home.join("config.yaml");
    write_world_settings(
        &config_path,
        "project",
        Some(Path::new("/global/root")),
        Some(true),
    );
    let _env = EnvGuard::new(vec![
        ("SUBSTRATE_HOME", Some(home.display().to_string())),
        ("SUBSTRATE_ANCHOR_MODE", None),
        ("SUBSTRATE_ANCHOR_PATH", None),
        ("SUBSTRATE_WORLD_ROOT_MODE", Some("custom".into())),
        ("SUBSTRATE_WORLD_ROOT_PATH", Some("/env/root".into())),
        ("SUBSTRATE_CAGED", Some("false".into())),
    ]);
    let launch_dir = temp.path().join("project");
    std::fs::create_dir_all(launch_dir.join(".substrate")).unwrap();
    let dir_settings = launch_dir.join(".substrate/settings.yaml");
    write_world_settings(
        &dir_settings,
        "custom",
        Some(Path::new("/dir/root")),
        Some(false),
    );

    let settings = resolve_world_root(None, None, None, &launch_dir).expect("dir config settings");

    assert_eq!(settings.mode, WorldRootMode::Custom);
    assert_eq!(settings.path, PathBuf::from("/dir/root"));
    assert!(!settings.caged);
}

#[test]
#[serial]
fn resolve_world_root_prefers_cli_over_all_other_sources() {
    let temp = TempDir::new().unwrap();
    let home = setup_substrate_home(&temp);
    let config_path = home.join("config.yaml");
    write_world_settings(
        &config_path,
        "project",
        Some(Path::new("/global/root")),
        Some(true),
    );
    let _env = EnvGuard::new(vec![
        ("SUBSTRATE_HOME", Some(home.display().to_string())),
        ("SUBSTRATE_ANCHOR_MODE", None),
        ("SUBSTRATE_ANCHOR_PATH", None),
        ("SUBSTRATE_WORLD_ROOT_MODE", Some("project".into())),
        ("SUBSTRATE_WORLD_ROOT_PATH", Some("/env/root".into())),
        ("SUBSTRATE_CAGED", Some("false".into())),
    ]);
    let launch_dir = temp.path().join("project");
    std::fs::create_dir_all(launch_dir.join(".substrate")).unwrap();
    let dir_settings = launch_dir.join(".substrate/settings.yaml");
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
fn resolve_world_root_honors_anchor_keys_and_legacy_precedence() {
    let temp = TempDir::new().unwrap();
    let home = setup_substrate_home(&temp);
    let global_path = home.join("config.yaml");
    let launch_dir = temp.path().join("project");
    std::fs::create_dir_all(&launch_dir).unwrap();
    write_anchor_world_settings(
        &global_path,
        "custom",
        Some(Path::new("/global/anchor")),
        Some("project"),
        Some(Path::new("/global/legacy")),
        Some(true),
    );
    let _env = EnvGuard::new(vec![
        ("SUBSTRATE_HOME", Some(home.display().to_string())),
        ("SUBSTRATE_ANCHOR_MODE", Some("project".into())),
        ("SUBSTRATE_WORLD_ROOT_MODE", Some("custom".into())),
        ("SUBSTRATE_ANCHOR_PATH", Some("/env/anchor".into())),
        ("SUBSTRATE_WORLD_ROOT_PATH", Some("/env/legacy".into())),
        ("SUBSTRATE_CAGED", Some("false".into())),
    ]);

    let settings =
        resolve_world_root(None, None, None, &launch_dir).expect("global anchor settings");
    assert_eq!(settings.mode, WorldRootMode::Custom);
    assert_eq!(settings.path, PathBuf::from("/global/anchor"));
    assert!(settings.caged);

    let dir_settings = launch_dir.join(".substrate/settings.yaml");
    write_world_settings(
        &dir_settings,
        "custom",
        Some(Path::new("/dir/root")),
        Some(false),
    );

    let settings_with_dir =
        resolve_world_root(None, None, None, &launch_dir).expect("dir settings override");
    assert_eq!(settings_with_dir.mode, WorldRootMode::Custom);
    assert_eq!(settings_with_dir.path, PathBuf::from("/dir/root"));
    assert!(!settings_with_dir.caged);
}

#[test]
#[serial]
fn resolve_world_root_requires_path_for_custom_mode() {
    let temp = TempDir::new().unwrap();
    let home = setup_substrate_home(&temp);
    let _env = EnvGuard::new(vec![
        ("SUBSTRATE_HOME", Some(home.display().to_string())),
        ("SUBSTRATE_ANCHOR_MODE", None),
        ("SUBSTRATE_ANCHOR_PATH", None),
        ("SUBSTRATE_WORLD_ROOT_MODE", None),
        ("SUBSTRATE_WORLD_ROOT_PATH", None),
        ("SUBSTRATE_CAGED", None),
    ]);
    assert!(std::env::var("SUBSTRATE_ANCHOR_MODE").is_err());
    assert!(std::env::var("SUBSTRATE_ANCHOR_PATH").is_err());
    assert!(std::env::var("SUBSTRATE_WORLD_ROOT_MODE").is_err());
    assert!(std::env::var("SUBSTRATE_WORLD_ROOT_PATH").is_err());
    let launch_dir = temp.path().join("project");
    std::fs::create_dir_all(&launch_dir).unwrap();
    let dir_settings = launch_dir.join(".substrate/settings.yaml");
    let global_settings = home.join("config.yaml");
    assert!(
        !dir_settings.exists(),
        "unexpected directory settings file at {}",
        dir_settings.display()
    );
    assert!(
        !global_settings.exists(),
        "unexpected global config at {}",
        global_settings.display()
    );

    let cwd = std::env::current_dir().unwrap();
    let env_settings = world_root_from_env();
    assert_eq!(env_settings.mode, WorldRootMode::Project);
    assert_eq!(env_settings.path, cwd);
    assert!(env_settings.caged);

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
    let _env = EnvGuard::new(vec![
        ("SUBSTRATE_HOME", Some(home.display().to_string())),
        ("SUBSTRATE_ANCHOR_MODE", None),
        ("SUBSTRATE_ANCHOR_PATH", None),
    ]);
    let target_cwd = temp.path().join("changing");
    std::fs::create_dir_all(&target_cwd).unwrap();
    let _cwd_guard = CwdGuard::new();
    std::env::set_current_dir(&target_cwd).unwrap();

    let settings = WorldRootSettings {
        mode: WorldRootMode::FollowCwd,
        path: PathBuf::from("/should/be/ignored"),
        caged: true,
    };

    let expected = std::fs::canonicalize(&target_cwd).unwrap_or(target_cwd);
    let actual = std::fs::canonicalize(settings.effective_root())
        .unwrap_or_else(|_| settings.effective_root());
    assert_eq!(actual, expected);
}
