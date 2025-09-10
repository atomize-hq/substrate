use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Policy {
    pub id: String,
    pub name: String,

    // Filesystem
    pub fs_read: Vec<String>,  // Paths that can be read
    pub fs_write: Vec<String>, // Paths that can be written

    // Network
    pub net_allowed: Vec<String>, // Allowed hosts/domains

    // Commands
    pub cmd_allowed: Vec<String>,  // Allowed command patterns
    pub cmd_denied: Vec<String>,   // Denied command patterns
    pub cmd_isolated: Vec<String>, // Commands to run in isolated world

    // Behavior
    pub require_approval: bool,
    pub allow_shell_operators: bool,

    // Resource limits (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limits: Option<ResourceLimits>,

    // Metadata
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<HashMap<String, String>>,
}

impl Default for Policy {
    fn default() -> Self {
        Self {
            id: "default".to_string(),
            name: "Default Policy".to_string(),
            fs_read: vec!["*".to_string()], // Allow all reads by default
            fs_write: vec![],               // No writes by default
            net_allowed: vec![],            // No network by default
            cmd_allowed: vec![],            // Empty = all allowed
            cmd_denied: vec![
                "rm -rf /*".to_string(),
                "curl * | bash".to_string(),
                "wget * | bash".to_string(),
            ],
            cmd_isolated: vec![
                "npm install".to_string(),
                "pip install".to_string(),
                "cargo install".to_string(),
            ],
            require_approval: false,
            allow_shell_operators: true,
            limits: None,
            metadata: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceLimits {
    pub max_memory_mb: Option<u64>,
    pub max_cpu_percent: Option<u32>,
    pub max_runtime_ms: Option<u64>,
    pub max_egress_bytes: Option<u64>,
}

#[derive(Debug, Clone)]
pub enum Decision {
    Allow,
    AllowWithRestrictions(Vec<Restriction>),
    Deny(String),
}

#[derive(Debug, Clone)]
pub struct Restriction {
    pub type_: RestrictionType,
    pub value: String,
}

#[derive(Debug, Clone)]
pub enum RestrictionType {
    IsolatedWorld,
    OverlayFS,
    NetworkFilter,
    ResourceLimit,
    Capability,
}

// Helper functions for policy management
impl Policy {
    pub fn from_yaml(content: &str) -> Result<Self, serde_yaml::Error> {
        serde_yaml::from_str(content)
    }

    pub fn to_yaml(&self) -> Result<String, serde_yaml::Error> {
        serde_yaml::to_string(self)
    }

    pub fn merge(&mut self, other: &Policy) {
        // Merge allow lists (union)
        for item in &other.fs_read {
            if !self.fs_read.contains(item) {
                self.fs_read.push(item.clone());
            }
        }

        for item in &other.fs_write {
            if !self.fs_write.contains(item) {
                self.fs_write.push(item.clone());
            }
        }

        for item in &other.net_allowed {
            if !self.net_allowed.contains(item) {
                self.net_allowed.push(item.clone());
            }
        }

        for item in &other.cmd_allowed {
            if !self.cmd_allowed.contains(item) {
                self.cmd_allowed.push(item.clone());
            }
        }

        // Merge deny lists (union)
        for item in &other.cmd_denied {
            if !self.cmd_denied.contains(item) {
                self.cmd_denied.push(item.clone());
            }
        }

        for item in &other.cmd_isolated {
            if !self.cmd_isolated.contains(item) {
                self.cmd_isolated.push(item.clone());
            }
        }

        // Take the more restrictive settings
        self.require_approval = self.require_approval || other.require_approval;
        self.allow_shell_operators = self.allow_shell_operators && other.allow_shell_operators;

        // Merge resource limits (take the more restrictive)
        if let Some(other_limits) = &other.limits {
            if let Some(limits) = &mut self.limits {
                if let Some(other_mem) = other_limits.max_memory_mb {
                    limits.max_memory_mb = Some(
                        limits
                            .max_memory_mb
                            .map(|m| m.min(other_mem))
                            .unwrap_or(other_mem),
                    );
                }
                if let Some(other_cpu) = other_limits.max_cpu_percent {
                    limits.max_cpu_percent = Some(
                        limits
                            .max_cpu_percent
                            .map(|c| c.min(other_cpu))
                            .unwrap_or(other_cpu),
                    );
                }
                if let Some(other_runtime) = other_limits.max_runtime_ms {
                    limits.max_runtime_ms = Some(
                        limits
                            .max_runtime_ms
                            .map(|r| r.min(other_runtime))
                            .unwrap_or(other_runtime),
                    );
                }
                if let Some(other_egress) = other_limits.max_egress_bytes {
                    limits.max_egress_bytes = Some(
                        limits
                            .max_egress_bytes
                            .map(|e| e.min(other_egress))
                            .unwrap_or(other_egress),
                    );
                }
            } else {
                self.limits = other.limits.clone();
            }
        }
    }

    pub fn is_path_readable(&self, path: &str) -> bool {
        for pattern in &self.fs_read {
            if pattern == "*" || path.starts_with(pattern.trim_end_matches('*')) {
                return true;
            }
        }
        false
    }

    pub fn is_path_writable(&self, path: &str) -> bool {
        for pattern in &self.fs_write {
            if pattern == "*" || path.starts_with(pattern.trim_end_matches('*')) {
                return true;
            }
        }
        false
    }

    pub fn is_host_allowed(&self, host: &str) -> bool {
        for pattern in &self.net_allowed {
            if pattern == "*" || host.ends_with(pattern.trim_start_matches('*')) {
                return true;
            }
        }
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_policy() {
        let policy = Policy::default();
        assert_eq!(policy.id, "default");
        assert!(!policy.cmd_allowed.is_empty() || !policy.cmd_denied.is_empty());
    }

    #[test]
    fn test_policy_merge() {
        let mut base = Policy {
            fs_write: vec!["/tmp/*".to_string()],
            ..Default::default()
        };

        let addon = Policy {
            id: "addon".to_string(),
            name: "Addon Policy".to_string(),
            fs_write: vec!["/var/tmp/*".to_string()],
            cmd_denied: vec!["sudo rm -rf /".to_string()],
            ..Default::default()
        };

        base.merge(&addon);

        assert_eq!(base.fs_write.len(), 2);
        assert!(base.fs_write.contains(&"/tmp/*".to_string()));
        assert!(base.fs_write.contains(&"/var/tmp/*".to_string()));
        assert!(base.cmd_denied.contains(&"sudo rm -rf /".to_string()));
    }

    #[test]
    fn test_path_checks() {
        let policy = Policy {
            fs_read: vec!["/home/*".to_string(), "/tmp/*".to_string()],
            fs_write: vec!["/tmp/*".to_string()],
            ..Default::default()
        };

        assert!(policy.is_path_readable("/home/user/file.txt"));
        assert!(policy.is_path_readable("/tmp/test.txt"));
        assert!(!policy.is_path_readable("/etc/passwd"));

        assert!(policy.is_path_writable("/tmp/test.txt"));
        assert!(!policy.is_path_writable("/home/user/file.txt"));
    }

    #[test]
    fn test_host_allowed() {
        let policy = Policy {
            net_allowed: vec!["github.com".to_string(), "*.example.com".to_string()],
            ..Default::default()
        };

        assert!(policy.is_host_allowed("github.com"));
        assert!(policy.is_host_allowed("api.github.com"));
        assert!(policy.is_host_allowed("test.example.com"));
        assert!(!policy.is_host_allowed("evil.com"));
    }
}
