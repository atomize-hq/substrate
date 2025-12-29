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

fn yaml_escape_path(path: &Path) -> String {
    path.display().to_string().replace('\\', "\\\\")
}

fn write_config_yaml(
    path: &Path,
    world_enabled: bool,
    anchor_mode: &str,
    anchor_path: Option<&Path>,
    caged: bool,
) {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).expect("create config parent");
    }

    let anchor_path = anchor_path.map(yaml_escape_path).unwrap_or_default();
    let enabled = if world_enabled { "true" } else { "false" };
    let caged = if caged { "true" } else { "false" };
    let body = format!(
        "world:\n  enabled: {enabled}\n  anchor_mode: {anchor_mode}\n  anchor_path: \"{anchor_path}\"\n  caged: {caged}\n\npolicy:\n  mode: observe\n\nsync:\n  auto_sync: false\n  direction: from_world\n  conflict_policy: prefer_host\n  exclude: []\n"
    );
    std::fs::write(path, body).expect("write config yaml");
}

fn setup_substrate_home(temp: &TempDir) -> PathBuf {
    let home = temp.path().join("home").join(".substrate");
    std::fs::create_dir_all(&home).expect("create substrate home");
    home
}

fn write_global_config(
    substrate_home: &Path,
    world_enabled: bool,
    anchor_mode: &str,
    anchor_path: Option<&Path>,
    caged: bool,
) -> PathBuf {
    let config_path = substrate_home.join("config.yaml");
    write_config_yaml(&config_path, world_enabled, anchor_mode, anchor_path, caged);
    config_path
}

fn write_workspace_config(
    workspace_root: &Path,
    world_enabled: bool,
    anchor_mode: &str,
    anchor_path: Option<&Path>,
    caged: bool,
) -> PathBuf {
    let workspace_dir = workspace_root.join(".substrate");
    std::fs::create_dir_all(&workspace_dir).expect("create workspace .substrate");
    let marker_path = workspace_dir.join("workspace.yaml");
    write_config_yaml(&marker_path, world_enabled, anchor_mode, anchor_path, caged);
    marker_path
}

fn normalize_windows_verbatim_prefix(value: &str) -> String {
    #[cfg(windows)]
    {
        value.replace("\\\\?\\", "")
    }
    #[cfg(not(windows))]
    {
        value.to_string()
    }
}

fn normalize_path_display(path: &Path) -> String {
    normalize_windows_verbatim_prefix(&path.display().to_string())
}

#[test]
#[serial]
fn resolve_world_root_defaults_to_launch_dir_project() {
    let temp = TempDir::new().unwrap();
    let substrate_home = setup_substrate_home(&temp);
    let _env = EnvGuard::new(vec![
        ("SUBSTRATE_HOME", Some(substrate_home.display().to_string())),
        ("SUBSTRATE_ANCHOR_MODE", None),
        ("SUBSTRATE_ANCHOR_PATH", None),
        ("SUBSTRATE_CAGED", None),
    ]);

    let launch_dir = temp.path().join("workspace");
    std::fs::create_dir_all(&launch_dir).unwrap();

    let env_settings = world_root_from_env();
    assert_eq!(env_settings.mode, WorldRootMode::Project);

    let settings = resolve_world_root(None, None, None, &launch_dir).expect("default settings");
    assert_eq!(settings.mode, WorldRootMode::Project);
    assert_eq!(settings.path, launch_dir);
    assert!(settings.caged);
}

#[test]
#[serial]
fn resolve_world_root_refuses_legacy_workspace_settings_yaml() {
    let temp = TempDir::new().unwrap();
    let substrate_home = setup_substrate_home(&temp);
    let _env = EnvGuard::new(vec![(
        "SUBSTRATE_HOME",
        Some(substrate_home.display().to_string()),
    )]);

    let workspace_root = temp.path().join("workspace");
    std::fs::create_dir_all(&workspace_root).unwrap();

    write_workspace_config(&workspace_root, true, "workspace", None, true);
    let legacy_settings = workspace_root.join(".substrate").join("settings.yaml");
    std::fs::write(&legacy_settings, "world:\n  enabled: true\n").unwrap();

    let err = resolve_world_root(None, None, None, &workspace_root)
        .expect_err("legacy settings.yaml should be rejected");
    let message = err.to_string();
    let normalized_message = normalize_windows_verbatim_prefix(&message);
    assert!(
        message.contains("unsupported legacy workspace config detected"),
        "unexpected error message: {message}"
    );
    assert!(
        normalized_message.contains(&normalize_path_display(&legacy_settings)),
        "error message missing legacy path: {message}"
    );
    let workspace_yaml = workspace_root.join(".substrate").join("workspace.yaml");
    assert!(
        normalized_message.contains(&normalize_path_display(&workspace_yaml)),
        "error message missing workspace.yaml path: {message}"
    );
}

#[test]
#[serial]
fn resolve_world_root_respects_env_when_no_configs() {
    let temp = TempDir::new().unwrap();
    let substrate_home = setup_substrate_home(&temp);
    let launch_dir = temp.path().join("project");
    std::fs::create_dir_all(&launch_dir).unwrap();
    let anchor_dir = temp.path().join("env-anchor");
    std::fs::create_dir_all(&anchor_dir).unwrap();

    let _env = EnvGuard::new(vec![
        ("SUBSTRATE_HOME", Some(substrate_home.display().to_string())),
        ("SUBSTRATE_ANCHOR_MODE", Some("custom".into())),
        (
            "SUBSTRATE_ANCHOR_PATH",
            Some(anchor_dir.display().to_string()),
        ),
        ("SUBSTRATE_CAGED", Some("false".into())),
    ]);

    let settings = resolve_world_root(None, None, None, &launch_dir).expect("env settings");
    assert_eq!(settings.mode, WorldRootMode::Custom);
    assert_eq!(
        settings.path,
        anchor_dir.canonicalize().unwrap_or(anchor_dir)
    );
    assert!(!settings.caged);
}

#[test]
#[serial]
fn resolve_world_root_env_overrides_global_config() {
    let temp = TempDir::new().unwrap();
    let substrate_home = setup_substrate_home(&temp);
    let launch_dir = temp.path().join("project");
    std::fs::create_dir_all(&launch_dir).unwrap();
    let anchor_dir = temp.path().join("env-anchor");
    std::fs::create_dir_all(&anchor_dir).unwrap();

    write_global_config(&substrate_home, true, "follow-cwd", None, true);

    let _env = EnvGuard::new(vec![
        ("SUBSTRATE_HOME", Some(substrate_home.display().to_string())),
        ("SUBSTRATE_ANCHOR_MODE", Some("custom".into())),
        (
            "SUBSTRATE_ANCHOR_PATH",
            Some(anchor_dir.display().to_string()),
        ),
        ("SUBSTRATE_CAGED", Some("0".into())),
    ]);

    let settings = resolve_world_root(None, None, None, &launch_dir).expect("resolved settings");
    assert_eq!(settings.mode, WorldRootMode::Custom);
    assert_eq!(
        settings.path,
        anchor_dir.canonicalize().unwrap_or(anchor_dir)
    );
    assert!(!settings.caged);
}

#[test]
#[serial]
fn resolve_world_root_prefers_workspace_config_over_global_when_env_unset() {
    let temp = TempDir::new().unwrap();
    let substrate_home = setup_substrate_home(&temp);

    let workspace_root = temp.path().join("workspace");
    let cwd = workspace_root.join("nested");
    std::fs::create_dir_all(&cwd).unwrap();

    let global_anchor = temp.path().join("global-anchor");
    let workspace_anchor = temp.path().join("workspace-anchor");
    std::fs::create_dir_all(&global_anchor).unwrap();
    std::fs::create_dir_all(&workspace_anchor).unwrap();

    write_global_config(&substrate_home, true, "custom", Some(&global_anchor), true);
    write_workspace_config(
        &workspace_root,
        true,
        "custom",
        Some(&workspace_anchor),
        false,
    );

    let _env = EnvGuard::new(vec![
        ("SUBSTRATE_HOME", Some(substrate_home.display().to_string())),
        ("SUBSTRATE_ANCHOR_MODE", None),
        ("SUBSTRATE_ANCHOR_PATH", None),
        ("SUBSTRATE_CAGED", None),
    ]);

    let settings = resolve_world_root(None, None, None, &cwd).expect("workspace settings");
    assert_eq!(settings.mode, WorldRootMode::Custom);
    assert_eq!(
        settings.path,
        workspace_anchor.canonicalize().unwrap_or(workspace_anchor)
    );
    assert!(!settings.caged);
}

#[test]
#[serial]
fn resolve_world_root_prefers_cli_over_all_other_sources() {
    let temp = TempDir::new().unwrap();
    let substrate_home = setup_substrate_home(&temp);

    let workspace_root = temp.path().join("workspace");
    let cwd = workspace_root.join("nested");
    std::fs::create_dir_all(&cwd).unwrap();

    let global_anchor = temp.path().join("global-anchor");
    let workspace_anchor = temp.path().join("workspace-anchor");
    let cli_anchor = temp.path().join("cli-anchor");
    std::fs::create_dir_all(&global_anchor).unwrap();
    std::fs::create_dir_all(&workspace_anchor).unwrap();
    std::fs::create_dir_all(&cli_anchor).unwrap();

    write_global_config(&substrate_home, true, "custom", Some(&global_anchor), false);
    write_workspace_config(
        &workspace_root,
        true,
        "custom",
        Some(&workspace_anchor),
        false,
    );

    let _env = EnvGuard::new(vec![
        ("SUBSTRATE_HOME", Some(substrate_home.display().to_string())),
        ("SUBSTRATE_ANCHOR_MODE", Some("custom".into())),
        (
            "SUBSTRATE_ANCHOR_PATH",
            Some(workspace_anchor.display().to_string()),
        ),
        ("SUBSTRATE_CAGED", Some("0".into())),
    ]);

    let settings = resolve_world_root(
        Some(WorldRootMode::Custom),
        Some(cli_anchor.clone()),
        Some(true),
        &cwd,
    )
    .expect("cli settings");

    assert_eq!(settings.mode, WorldRootMode::Custom);
    assert_eq!(
        settings.path,
        cli_anchor.canonicalize().unwrap_or(cli_anchor)
    );
    assert!(settings.caged);
}

#[test]
#[serial]
fn resolve_world_root_requires_anchor_path_for_custom_mode() {
    let temp = TempDir::new().unwrap();
    let substrate_home = setup_substrate_home(&temp);
    let _env = EnvGuard::new(vec![
        ("SUBSTRATE_HOME", Some(substrate_home.display().to_string())),
        ("SUBSTRATE_ANCHOR_MODE", None),
        ("SUBSTRATE_ANCHOR_PATH", None),
        ("SUBSTRATE_CAGED", None),
    ]);
    let launch_dir = temp.path().join("project");
    std::fs::create_dir_all(&launch_dir).unwrap();

    let err = resolve_world_root(Some(WorldRootMode::Custom), None, None, &launch_dir)
        .expect_err("custom without path should error");
    let message = err.to_string();
    assert!(
        message.contains("anchor_mode=custom requires world.anchor_path to be non-empty"),
        "unexpected error message: {message}"
    );
}

#[test]
#[serial]
fn effective_root_uses_current_directory_for_follow_mode() {
    let temp = TempDir::new().unwrap();
    let substrate_home = setup_substrate_home(&temp);
    let _env = EnvGuard::new(vec![
        ("SUBSTRATE_HOME", Some(substrate_home.display().to_string())),
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
