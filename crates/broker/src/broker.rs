//! Core broker state and policy evaluation logic.

use crate::approval::{self, ApprovalCache, ApprovalContext, ApprovalStatus};
use crate::policy::{Decision, Policy, Restriction, RestrictionType};
use crate::policy_loader::load_policy_from_path;
use crate::profile::ProfileDetector;
use anyhow::Result;
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, RwLock};
use substrate_common::WorldFsMode;
use tracing::{debug, info, warn};

pub struct Broker {
    pub(crate) policy: Arc<RwLock<Policy>>,
    pub(crate) approvals: Arc<RwLock<ApprovalCache>>,
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
        let new_policy = load_policy_from_path(path)?;
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

    pub fn allowed_domains(&self) -> Vec<String> {
        let Ok(policy) = self.policy.read() else {
            return Vec::new();
        };
        policy.net_allowed.clone()
    }

    pub fn world_fs_mode(&self) -> WorldFsMode {
        let Ok(policy) = self.policy.read() else {
            return WorldFsMode::Writable;
        };
        policy.world_fs_mode
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

pub(crate) fn matches_pattern(cmd: &str, pattern: &str) -> bool {
    // Simple glob matching for now, can be enhanced
    if pattern.contains('*') {
        match glob::Pattern::new(pattern) {
            Ok(pattern) => pattern.matches(cmd),
            Err(err) => {
                warn!("Invalid glob pattern '{}': {}", pattern, err);
                false
            }
        }
    } else {
        cmd.contains(pattern)
    }
}

fn log_violation(cmd: &str, reason: &str) {
    warn!("Policy violation: {} - Command: {}", reason, cmd);
    // In future, this could write to audit log or send telemetry
}
