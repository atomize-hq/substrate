use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::str::FromStr;
use substrate_common::WorldFsMode;

fn default_allow_shell_operators() -> bool {
    true
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct StrictWorldFsMode(WorldFsMode);

impl<'de> Deserialize<'de> for StrictWorldFsMode {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let raw = String::deserialize(deserializer)?;
        let normalized = raw.trim().to_ascii_lowercase();
        let mode = match normalized.as_str() {
            "writable" => WorldFsMode::Writable,
            "read_only" => WorldFsMode::ReadOnly,
            other => {
                return Err(serde::de::Error::custom(format!(
                    "invalid world_fs.mode: {} (expected writable or read_only)",
                    other
                )));
            }
        };
        Ok(Self(mode))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorldFsCage {
    Project,
    Full,
}

impl WorldFsCage {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Project => "project",
            Self::Full => "full",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        value.parse().ok()
    }
}

impl FromStr for WorldFsCage {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value.trim().to_ascii_lowercase().as_str() {
            "project" => Ok(Self::Project),
            "full" => Ok(Self::Full),
            other => Err(format!(
                "invalid world_fs.cage: {} (expected project or full)",
                other
            )),
        }
    }
}

impl<'de> Deserialize<'de> for WorldFsCage {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let raw = String::deserialize(deserializer)?;
        raw.parse().map_err(serde::de::Error::custom)
    }
}

impl Serialize for WorldFsCage {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorldFsPolicy {
    pub mode: WorldFsMode,
    pub cage: WorldFsCage,
    pub require_world: bool,
    pub read_allowlist: Vec<String>,
    pub write_allowlist: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct Policy {
    pub id: String,
    pub name: String,

    // Filesystem (policy schema: world_fs.*)
    pub fs_read: Vec<String>,         // world_fs.read_allowlist
    pub fs_write: Vec<String>,        // world_fs.write_allowlist
    pub world_fs_mode: WorldFsMode,   // world_fs.mode
    pub world_fs_cage: WorldFsCage,   // world_fs.cage
    pub world_fs_require_world: bool, // world_fs.require_world

    // Network
    pub net_allowed: Vec<String>, // Allowed hosts/domains

    // Commands
    pub cmd_allowed: Vec<String>,  // Allowed command patterns
    pub cmd_denied: Vec<String>,   // Denied command patterns
    pub cmd_isolated: Vec<String>, // Commands to run in isolated world

    // Behavior
    pub require_approval: bool,
    pub allow_shell_operators: bool,

    // Resource limits
    pub limits: ResourceLimits,

    // Metadata
    pub metadata: HashMap<String, String>,
}

impl Default for Policy {
    fn default() -> Self {
        Self {
            id: "default".to_string(),
            name: "Default Policy".to_string(),
            fs_read: vec!["*".to_string()],
            fs_write: vec![],
            world_fs_mode: WorldFsMode::Writable,
            world_fs_cage: WorldFsCage::Project,
            world_fs_require_world: false,
            net_allowed: vec![],
            cmd_allowed: vec![],
            cmd_denied: vec![],
            cmd_isolated: vec![],
            require_approval: false,
            allow_shell_operators: default_allow_shell_operators(),
            limits: ResourceLimits {
                max_memory_mb: None,
                max_cpu_percent: None,
                max_runtime_ms: None,
                max_egress_bytes: None,
            },
            metadata: HashMap::new(),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawWorldFsV1 {
    mode: StrictWorldFsMode,
    #[serde(rename = "isolation")]
    cage: WorldFsCage,
    require_world: bool,
    read_allowlist: Vec<String>,
    write_allowlist: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawPolicyV1 {
    id: String,
    name: String,
    world_fs: RawWorldFsV1,

    net_allowed: Vec<String>,

    cmd_allowed: Vec<String>,
    cmd_denied: Vec<String>,
    cmd_isolated: Vec<String>,

    require_approval: bool,
    allow_shell_operators: bool,

    limits: ResourceLimits,

    metadata: HashMap<String, String>,
}

#[derive(Debug, Serialize)]
#[serde(deny_unknown_fields)]
struct WorldFsFileV1<'a> {
    mode: WorldFsMode,
    isolation: WorldFsCage,
    require_world: bool,
    read_allowlist: &'a [String],
    write_allowlist: &'a [String],
}

#[derive(Debug, Serialize)]
#[serde(deny_unknown_fields)]
struct PolicyFileV1<'a> {
    id: &'a str,
    name: &'a str,
    world_fs: WorldFsFileV1<'a>,

    net_allowed: &'a [String],

    cmd_allowed: &'a [String],
    cmd_denied: &'a [String],
    cmd_isolated: &'a [String],

    require_approval: bool,
    allow_shell_operators: bool,

    limits: &'a ResourceLimits,

    metadata: &'a HashMap<String, String>,
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

impl Policy {
    pub fn world_fs_policy(&self) -> WorldFsPolicy {
        WorldFsPolicy {
            mode: self.world_fs_mode,
            cage: self.world_fs_cage,
            require_world: self.world_fs_require_world,
            read_allowlist: self.fs_read.clone(),
            write_allowlist: self.fs_write.clone(),
        }
    }

    pub fn requires_world(&self) -> bool {
        self.world_fs_require_world
    }

    pub fn from_yaml(content: &str) -> Result<Self, serde_yaml::Error> {
        serde_yaml::from_str(content)
    }

    pub fn to_yaml(&self) -> Result<String, serde_yaml::Error> {
        serde_yaml::to_string(self)
    }

    fn validate_world_fs(world_fs: &RawWorldFsV1) -> Result<(), String> {
        if world_fs.mode.0 == WorldFsMode::ReadOnly && !world_fs.require_world {
            return Err("world_fs.mode=read_only requires world_fs.require_world=true".to_string());
        }

        if world_fs.cage == WorldFsCage::Full && !world_fs.require_world {
            return Err("world_fs.isolation=full requires world_fs.require_world=true".to_string());
        }

        Ok(())
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
        self.world_fs_mode = match (self.world_fs_mode, other.world_fs_mode) {
            (WorldFsMode::ReadOnly, _) | (_, WorldFsMode::ReadOnly) => WorldFsMode::ReadOnly,
            _ => WorldFsMode::Writable,
        };
        self.world_fs_cage = match (self.world_fs_cage, other.world_fs_cage) {
            (WorldFsCage::Full, _) | (_, WorldFsCage::Full) => WorldFsCage::Full,
            _ => WorldFsCage::Project,
        };
        self.world_fs_require_world = self.world_fs_require_world || other.world_fs_require_world;

        // Merge resource limits (take the more restrictive)
        if let Some(other_mem) = other.limits.max_memory_mb {
            self.limits.max_memory_mb = Some(
                self.limits
                    .max_memory_mb
                    .map(|m| m.min(other_mem))
                    .unwrap_or(other_mem),
            );
        }
        if let Some(other_cpu) = other.limits.max_cpu_percent {
            self.limits.max_cpu_percent = Some(
                self.limits
                    .max_cpu_percent
                    .map(|c| c.min(other_cpu))
                    .unwrap_or(other_cpu),
            );
        }
        if let Some(other_runtime) = other.limits.max_runtime_ms {
            self.limits.max_runtime_ms = Some(
                self.limits
                    .max_runtime_ms
                    .map(|r| r.min(other_runtime))
                    .unwrap_or(other_runtime),
            );
        }
        if let Some(other_egress) = other.limits.max_egress_bytes {
            self.limits.max_egress_bytes = Some(
                self.limits
                    .max_egress_bytes
                    .map(|e| e.min(other_egress))
                    .unwrap_or(other_egress),
            );
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

impl<'de> Deserialize<'de> for Policy {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = serde_yaml::Value::deserialize(deserializer)?;
        let raw: RawPolicyV1 = serde_yaml::from_value(value).map_err(serde::de::Error::custom)?;
        Policy::validate_world_fs(&raw.world_fs).map_err(serde::de::Error::custom)?;

        Ok(Self {
            id: raw.id,
            name: raw.name,
            fs_read: raw.world_fs.read_allowlist,
            fs_write: raw.world_fs.write_allowlist,
            world_fs_mode: raw.world_fs.mode.0,
            world_fs_cage: raw.world_fs.cage,
            world_fs_require_world: raw.world_fs.require_world,
            net_allowed: raw.net_allowed,
            cmd_allowed: raw.cmd_allowed,
            cmd_denied: raw.cmd_denied,
            cmd_isolated: raw.cmd_isolated,
            require_approval: raw.require_approval,
            allow_shell_operators: raw.allow_shell_operators,
            limits: raw.limits,
            metadata: raw.metadata,
        })
    }
}

impl Serialize for Policy {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let file = PolicyFileV1 {
            id: &self.id,
            name: &self.name,
            world_fs: WorldFsFileV1 {
                mode: self.world_fs_mode,
                isolation: self.world_fs_cage,
                require_world: self.world_fs_require_world,
                read_allowlist: &self.fs_read,
                write_allowlist: &self.fs_write,
            },
            net_allowed: &self.net_allowed,
            cmd_allowed: &self.cmd_allowed,
            cmd_denied: &self.cmd_denied,
            cmd_isolated: &self.cmd_isolated,
            require_approval: self.require_approval,
            allow_shell_operators: self.allow_shell_operators,
            limits: &self.limits,
            metadata: &self.metadata,
        };
        file.serialize(serializer)
    }
}

#[cfg(test)]
mod tests;
