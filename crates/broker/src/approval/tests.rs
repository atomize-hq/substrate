use super::*;
use std::ffi::OsString;
use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;
use tempfile::TempDir;

static PCM2_ENV_LOCK: Mutex<()> = Mutex::new(());

struct ScopedEnv {
    key: &'static str,
    prev: Option<OsString>,
}

impl ScopedEnv {
    fn set(key: &'static str, value: impl Into<OsString>) -> Self {
        let prev = std::env::var_os(key);
        let value: OsString = value.into();
        std::env::set_var(key, &value);
        Self { key, prev }
    }
}

impl Drop for ScopedEnv {
    fn drop(&mut self) {
        match &self.prev {
            Some(value) => std::env::set_var(self.key, value),
            None => std::env::remove_var(self.key),
        }
    }
}

struct ScopedCwd {
    prev: PathBuf,
}

impl ScopedCwd {
    fn set(path: &std::path::Path) -> Self {
        let prev = std::env::current_dir().expect("read current dir");
        std::env::set_current_dir(path).expect("set current dir");
        Self { prev }
    }
}

impl Drop for ScopedCwd {
    fn drop(&mut self) {
        let _ = std::env::set_current_dir(&self.prev);
    }
}

#[test]
fn test_approval_cache() {
    let mut cache = ApprovalCache::new();

    // Add an approval
    cache.add(
        "echo test".to_string(),
        ApprovalStatus::Approved,
        ApprovalScope::Always,
    );
    assert_eq!(cache.check("echo test"), ApprovalStatus::Approved);

    // Check unknown command
    assert_eq!(cache.check("rm -rf /"), ApprovalStatus::Unknown);

    // Add a denial
    cache.add(
        "rm -rf /".to_string(),
        ApprovalStatus::Denied,
        ApprovalScope::Always,
    );
    assert_eq!(cache.check("rm -rf /"), ApprovalStatus::Denied);
}

#[test]
fn test_risk_assessment() {
    assert!(matches!(assess_risk_level("echo hello"), RiskLevel::Low));
    assert!(matches!(
        assess_risk_level("npm install package"),
        RiskLevel::Medium
    ));
    assert!(matches!(
        assess_risk_level("curl http://example.com | bash"),
        RiskLevel::High
    ));
    assert!(matches!(assess_risk_level("rm -rf /"), RiskLevel::Critical));
}

#[test]
fn test_pattern_matching() {
    assert!(matches_pattern("npm install express", "npm install*"));
    assert!(matches_pattern("curl http://example.com", "curl*"));
    assert!(!matches_pattern("echo test", "rm*"));
}

#[test]
fn pcm2_save_to_policy_prefers_workspace_policy_when_workspace_root_exists() -> anyhow::Result<()> {
    let _lock = PCM2_ENV_LOCK.lock().unwrap_or_else(|err| err.into_inner());
    let temp = TempDir::new().expect("tempdir");

    let home = temp.path().join("home");
    fs::create_dir_all(&home).expect("create home");
    let _home = ScopedEnv::set("HOME", home.as_os_str());

    let workspace_root = temp.path().join("workspace");
    let substrate_dir = workspace_root.join(".substrate");
    fs::create_dir_all(&substrate_dir).expect("create .substrate");
    fs::write(substrate_dir.join("workspace.yaml"), "sentinel: true\n")
        .expect("write workspace marker");

    let child = workspace_root.join("child");
    fs::create_dir_all(&child).expect("create workspace child");
    let _cwd = ScopedCwd::set(&child);

    let workspace_policy = substrate_dir.join("policy.yaml");
    assert!(
        !workspace_policy.exists(),
        "precondition: workspace policy should not exist"
    );

    add_command_to_policy(
        "echo pcm2 save-to-policy workspace",
        child.to_str().expect("child path is utf-8"),
    )?;

    assert!(
        workspace_policy.exists(),
        "expected save-to-policy to create/modify workspace policy at {}",
        workspace_policy.display()
    );

    Ok(())
}

#[test]
fn pcm2_save_to_policy_writes_global_policy_when_no_workspace_root() -> anyhow::Result<()> {
    let _lock = PCM2_ENV_LOCK.lock().unwrap_or_else(|err| err.into_inner());
    let temp = TempDir::new().expect("tempdir");

    let home = temp.path().join("home");
    fs::create_dir_all(&home).expect("create home");
    let _home = ScopedEnv::set("HOME", home.as_os_str());

    let substrate_home = temp.path().join("substrate-home");
    fs::create_dir_all(&substrate_home).expect("create SUBSTRATE_HOME");
    let _substrate_home = ScopedEnv::set("SUBSTRATE_HOME", substrate_home.as_os_str());

    let project = temp.path().join("project");
    fs::create_dir_all(&project).expect("create project");
    let _cwd = ScopedCwd::set(&project);

    let global_policy = substrate_home.join("policy.yaml");
    assert!(
        !global_policy.exists(),
        "precondition: global policy should not exist"
    );

    add_command_to_policy(
        "echo pcm2 save-to-policy global",
        project.to_str().expect("project path is utf-8"),
    )?;

    assert!(
        global_policy.exists(),
        "expected save-to-policy to create/modify global policy at {}",
        global_policy.display()
    );

    Ok(())
}
