//! Core broker state and policy evaluation logic.

use crate::approval::{self, ApprovalCache, ApprovalContext, ApprovalStatus};
use crate::mode::PolicyMode;
use crate::policy::{Decision, Policy, Restriction, RestrictionType, WorldFsPolicy};
use crate::policy_loader::{load_effective_policy_for_cwd, load_policy_from_path};
use anyhow::Result;
use std::path::Path;
use std::sync::atomic::{AtomicU8, Ordering};
use std::sync::{Arc, RwLock};
use substrate_common::WorldFsMode;
use tracing::{debug, info, warn};

pub struct Broker {
    pub(crate) policy: Arc<RwLock<Policy>>,
    pub(crate) approvals: Arc<RwLock<ApprovalCache>>,
    policy_mode: AtomicU8,
}

const MODE_DISABLED: u8 = 0;
const MODE_OBSERVE: u8 = 1;
const MODE_ENFORCE: u8 = 2;

fn mode_to_u8(mode: PolicyMode) -> u8 {
    match mode {
        PolicyMode::Disabled => MODE_DISABLED,
        PolicyMode::Observe => MODE_OBSERVE,
        PolicyMode::Enforce => MODE_ENFORCE,
    }
}

fn mode_from_u8(raw: u8) -> PolicyMode {
    match raw {
        MODE_DISABLED => PolicyMode::Disabled,
        MODE_ENFORCE => PolicyMode::Enforce,
        _ => PolicyMode::Observe,
    }
}

impl Broker {
    pub fn new() -> Self {
        let default_policy = Policy::default();
        Self {
            policy: Arc::new(RwLock::new(default_policy)),
            approvals: Arc::new(RwLock::new(ApprovalCache::new())),
            policy_mode: AtomicU8::new(mode_to_u8(PolicyMode::from_env())),
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

    pub fn detect_and_load_policy(&self, cwd: &Path) -> Result<()> {
        let (policy, source) = load_effective_policy_for_cwd(cwd)?;
        let mut guard = self
            .policy
            .write()
            .map_err(|e| anyhow::anyhow!("Failed to acquire policy write lock: {}", e))?;
        *guard = policy;

        if let Some(path) = source {
            info!("Loaded policy from {:?}", path);
        } else {
            info!("Loaded built-in default policy");
        }
        Ok(())
    }

    pub fn evaluate(&self, cmd: &str, cwd: &str, _world_id: Option<&str>) -> Result<Decision> {
        if self.policy_mode() == PolicyMode::Disabled {
            return Ok(Decision::Allow);
        }

        let policy = self
            .policy
            .read()
            .map_err(|e| anyhow::anyhow!("Failed to acquire policy read lock: {}", e))?;

        // Check denied commands first
        for pattern in &policy.cmd_denied {
            if matches_pattern(cmd, pattern) {
                log_violation(cmd, "Command explicitly denied");
                return Ok(Decision::Deny("Command explicitly denied".into()));
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
            log_violation(cmd, "Command not in allowlist");
            return Ok(Decision::Deny("Command not explicitly allowed".into()));
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
        if policy.require_approval && self.policy_mode() == PolicyMode::Enforce {
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
        if self.policy_mode() == PolicyMode::Disabled {
            return Ok(Decision::Allow);
        }

        // Fast path for shims - just check deny list
        let cmd = argv.join(" ");
        let policy = self
            .policy
            .read()
            .map_err(|e| anyhow::anyhow!("Failed to acquire policy read lock: {}", e))?;

        for pattern in &policy.cmd_denied {
            if matches_pattern(&cmd, pattern) {
                return Ok(Decision::Deny("Command denied by policy".into()));
            }
        }

        if !policy.cmd_allowed.is_empty()
            && !policy
                .cmd_allowed
                .iter()
                .any(|pattern| matches_pattern(&cmd, pattern))
        {
            return Ok(Decision::Deny("Command not explicitly allowed".into()));
        }

        for pattern in &policy.cmd_isolated {
            if matches_pattern(&cmd, pattern) {
                return Ok(Decision::AllowWithRestrictions(vec![Restriction {
                    type_: RestrictionType::IsolatedWorld,
                    value: "ephemeral".into(),
                }]));
            }
        }

        Ok(Decision::Allow)
    }

    pub fn set_observe_only(&self, observe: bool) {
        self.set_policy_mode(if observe {
            PolicyMode::Observe
        } else {
            PolicyMode::Enforce
        });
    }

    pub fn is_observe_only(&self) -> bool {
        self.policy_mode() != PolicyMode::Enforce
    }

    pub fn set_policy_mode(&self, mode: PolicyMode) {
        self.policy_mode.store(mode_to_u8(mode), Ordering::Relaxed);
        info!("Policy mode: {}", mode.as_str());
    }

    pub fn policy_mode(&self) -> PolicyMode {
        mode_from_u8(self.policy_mode.load(Ordering::Relaxed))
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

    pub fn world_fs_policy(&self) -> WorldFsPolicy {
        let Ok(policy) = self.policy.read() else {
            return Policy::default().world_fs_policy();
        };
        policy.world_fs_policy()
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
