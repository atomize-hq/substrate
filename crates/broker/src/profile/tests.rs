use super::*;
use std::path::Path;
use std::sync::{Mutex, OnceLock};
use tempfile::tempdir;

fn env_lock() -> &'static Mutex<()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
}

fn write_workspace_marker(root: &Path) {
    let substrate_dir = root.join(substrate_paths::SUBSTRATE_DIR_NAME);
    std::fs::create_dir_all(&substrate_dir).unwrap();
    std::fs::write(substrate_dir.join("workspace.yaml"), "version: 1\n").unwrap();
}

fn write_workspace_policy(root: &Path, contents: &str) -> PathBuf {
    let substrate_dir = root.join(substrate_paths::SUBSTRATE_DIR_NAME);
    std::fs::create_dir_all(&substrate_dir).unwrap();
    let path = substrate_dir.join("policy.yaml");
    std::fs::write(&path, contents).unwrap();
    path
}

fn with_substrate_home<T>(home: &Path, f: impl FnOnce() -> T) -> T {
    let _lock = env_lock().lock().unwrap();
    let previous = std::env::var_os("SUBSTRATE_HOME");
    std::env::set_var("SUBSTRATE_HOME", home);
    let result = f();
    if let Some(value) = previous {
        std::env::set_var("SUBSTRATE_HOME", value);
    } else {
        std::env::remove_var("SUBSTRATE_HOME");
    }
    result
}

#[test]
fn test_workspace_policy_detection_from_subdirectory() {
    let temp = tempdir().unwrap();
    let substrate_home = temp.path().join("home").join(".substrate");
    with_substrate_home(&substrate_home, || {
        let project_dir = temp.path().join("project");
        let sub_dir = project_dir.join("src").join("lib");
        std::fs::create_dir_all(&sub_dir).unwrap();

        write_workspace_marker(&project_dir);
        let policy_path = write_workspace_policy(&project_dir, "test: workspace\n");

        let mut detector = ProfileDetector::new();
        let result = detector.find_profile(&sub_dir).unwrap();
        assert!(result.is_some());

        let found = result.unwrap().canonicalize().unwrap();
        let expected = policy_path.canonicalize().unwrap();
        assert_eq!(found, expected);

        let result2 = detector.find_profile(&sub_dir).unwrap();
        assert!(result2.is_some());
    });
}

#[test]
fn test_workspace_without_policy_returns_none() {
    let temp = tempdir().unwrap();
    let substrate_home = temp.path().join("home").join(".substrate");
    with_substrate_home(&substrate_home, || {
        let project_dir = temp.path().join("project");
        std::fs::create_dir_all(&project_dir).unwrap();

        write_workspace_marker(&project_dir);

        let mut detector = ProfileDetector::new();
        let result = detector.find_profile(&project_dir).unwrap();
        assert_eq!(result, None);
    });
}

#[test]
fn test_global_policy_used_when_no_workspace_found() {
    let temp = tempdir().unwrap();
    let substrate_home = temp.path().join("home").join(".substrate");
    with_substrate_home(&substrate_home, || {
        let global_policy = write_workspace_policy(substrate_home.parent().unwrap(), "id: test\n");

        let project_dir = temp.path().join("project");
        std::fs::create_dir_all(&project_dir).unwrap();

        let mut detector = ProfileDetector::new();
        let result = detector.find_profile(&project_dir).unwrap();
        assert!(result.is_some());
        assert_eq!(
            result.unwrap().canonicalize().unwrap(),
            global_policy.canonicalize().unwrap()
        );
    });
}

#[test]
fn test_no_policy_returns_none() {
    let temp = tempdir().unwrap();
    let substrate_home = temp.path().join("home").join(".substrate");
    with_substrate_home(&substrate_home, || {
        let project_dir = temp.path().join("project");
        std::fs::create_dir_all(&project_dir).unwrap();

        let mut detector = ProfileDetector::new();
        let result = detector.find_profile(&project_dir).unwrap();
        assert_eq!(result, None);
    });
}

#[test]
fn test_sample_policy_creation() {
    let temp = tempdir().unwrap();
    let substrate_home = temp.path().join("home").join(".substrate");
    with_substrate_home(&substrate_home, || {
        let project_dir = temp.path().join("project");
        let policy_path = project_dir
            .join(substrate_paths::SUBSTRATE_DIR_NAME)
            .join("policy.yaml");
        std::fs::create_dir_all(policy_path.parent().unwrap()).unwrap();

        create_sample_profile(&policy_path).unwrap();
        assert!(policy_path.exists());

        let content = std::fs::read_to_string(&policy_path).unwrap();
        let policy: crate::Policy = serde_yaml::from_str(&content).unwrap();
        assert_eq!(policy.id, "project-policy");
    });
}
