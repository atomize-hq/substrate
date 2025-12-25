use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::str::FromStr;
use substrate_common::WorldFsMode;

fn default_fs_read() -> Vec<String> {
    vec!["*".to_string()]
}

fn default_allow_shell_operators() -> bool {
    true
}

fn is_false(value: &bool) -> bool {
    !*value
}

fn is_default_allow_shell_operators(value: &bool) -> bool {
    *value == default_allow_shell_operators()
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

    // Resource limits (optional)
    pub limits: Option<ResourceLimits>,

    // Metadata
    pub metadata: Option<HashMap<String, String>>,
}

impl Default for Policy {
    fn default() -> Self {
        Self {
            id: "default".to_string(),
            name: "Default Policy".to_string(),
            fs_read: default_fs_read(),
            fs_write: vec![],
            world_fs_mode: WorldFsMode::Writable,
            world_fs_cage: WorldFsCage::Project,
            world_fs_require_world: false,
            net_allowed: vec![],
            cmd_allowed: vec![],
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
            allow_shell_operators: default_allow_shell_operators(),
            limits: None,
            metadata: None,
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawWorldFsV1 {
    mode: WorldFsMode,
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

    #[serde(default)]
    net_allowed: Vec<String>,

    #[serde(default)]
    cmd_allowed: Vec<String>,
    #[serde(default)]
    cmd_denied: Vec<String>,
    #[serde(default)]
    cmd_isolated: Vec<String>,

    #[serde(default)]
    require_approval: bool,
    #[serde(default = "default_allow_shell_operators")]
    allow_shell_operators: bool,

    #[serde(default)]
    limits: Option<ResourceLimits>,

    #[serde(default)]
    metadata: Option<HashMap<String, String>>,
}

#[derive(Debug, Serialize)]
#[serde(deny_unknown_fields)]
struct WorldFsFileV1<'a> {
    mode: WorldFsMode,
    cage: WorldFsCage,
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

    #[serde(skip_serializing_if = "<[String]>::is_empty")]
    net_allowed: &'a [String],

    #[serde(skip_serializing_if = "<[String]>::is_empty")]
    cmd_allowed: &'a [String],
    #[serde(skip_serializing_if = "<[String]>::is_empty")]
    cmd_denied: &'a [String],
    #[serde(skip_serializing_if = "<[String]>::is_empty")]
    cmd_isolated: &'a [String],

    #[serde(skip_serializing_if = "is_false")]
    require_approval: bool,
    #[serde(skip_serializing_if = "is_default_allow_shell_operators")]
    allow_shell_operators: bool,

    #[serde(skip_serializing_if = "Option::is_none")]
    limits: &'a Option<ResourceLimits>,

    #[serde(skip_serializing_if = "Option::is_none")]
    metadata: &'a Option<HashMap<String, String>>,
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
            require_world: self.requires_world(),
            read_allowlist: self.fs_read.clone(),
            write_allowlist: self.fs_write.clone(),
        }
    }

    pub fn requires_world(&self) -> bool {
        self.world_fs_require_world
            || self.world_fs_mode == WorldFsMode::ReadOnly
            || self.world_fs_cage == WorldFsCage::Full
    }

    pub fn from_yaml(content: &str) -> Result<Self, serde_yaml::Error> {
        serde_yaml::from_str(content)
    }

    pub fn to_yaml(&self) -> Result<String, serde_yaml::Error> {
        serde_yaml::to_string(self)
    }

    fn validate_world_fs(world_fs: &RawWorldFsV1) -> Result<(), String> {
        if world_fs.read_allowlist.is_empty() {
            return Err(
                "world_fs.read_allowlist must be non-empty (provide at least one glob, e.g. \"*\")"
                    .to_string(),
            );
        }
        for (idx, value) in world_fs.read_allowlist.iter().enumerate() {
            if value.trim().is_empty() {
                return Err(format!(
                    "world_fs.read_allowlist[{idx}] must be a non-empty string"
                ));
            }
        }
        for (idx, value) in world_fs.write_allowlist.iter().enumerate() {
            if value.trim().is_empty() {
                return Err(format!(
                    "world_fs.write_allowlist[{idx}] must be a non-empty string"
                ));
            }
        }

        if world_fs.mode == WorldFsMode::ReadOnly && !world_fs.require_world {
            return Err("world_fs.mode=read_only requires world_fs.require_world=true".to_string());
        }

        if world_fs.cage == WorldFsCage::Full && !world_fs.require_world {
            return Err("world_fs.cage=full requires world_fs.require_world=true".to_string());
        }

        Ok(())
    }

    fn missing_world_fs_message() -> String {
        [
            "missing required policy block: world_fs",
            "",
            "required fields:",
            "  world_fs.mode: writable | read_only",
            "  world_fs.cage: project | full",
            "  world_fs.require_world: true | false",
            "  world_fs.read_allowlist: [ ... ]   (must be non-empty)",
            "  world_fs.write_allowlist: [ ... ]  (can be empty)",
            "",
            "example:",
            "  world_fs:",
            "    mode: writable",
            "    cage: project",
            "    require_world: false",
            "    read_allowlist:",
            "      - \"*\"",
            "    write_allowlist: []",
        ]
        .join("\n")
    }

    fn legacy_key_message(key: &str, replacement: &str) -> String {
        format!("legacy policy key '{key}' is not supported; use '{replacement}' instead")
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

impl<'de> Deserialize<'de> for Policy {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = serde_yaml::Value::deserialize(deserializer)?;
        let Some(map) = value.as_mapping() else {
            return Err(serde::de::Error::custom(
                "policy must be a YAML mapping/object",
            ));
        };

        for (legacy, replacement) in [
            ("world_fs_mode", "world_fs.mode"),
            ("fs_read", "world_fs.read_allowlist"),
            ("fs_write", "world_fs.write_allowlist"),
        ] {
            if map.contains_key(serde_yaml::Value::String(legacy.to_string())) {
                return Err(serde::de::Error::custom(Policy::legacy_key_message(
                    legacy,
                    replacement,
                )));
            }
        }

        if !map.contains_key(serde_yaml::Value::String("world_fs".to_string())) {
            return Err(serde::de::Error::custom(Policy::missing_world_fs_message()));
        }

        let raw: RawPolicyV1 = serde_yaml::from_value(value).map_err(serde::de::Error::custom)?;
        Policy::validate_world_fs(&raw.world_fs).map_err(serde::de::Error::custom)?;

        Ok(Self {
            id: raw.id,
            name: raw.name,
            fs_read: raw.world_fs.read_allowlist,
            fs_write: raw.world_fs.write_allowlist,
            world_fs_mode: raw.world_fs.mode,
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
                cage: self.world_fs_cage,
                require_world: self.requires_world(),
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
