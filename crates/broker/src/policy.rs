use serde::ser::SerializeMap;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::str::FromStr;
use substrate_common::WorldFsMode;

fn default_allow_shell_operators() -> bool {
    true
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WorldFsDenyEnforcement {
    Strict,
    PreferStrict,
    Weak,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WorldFsEnforcement {
    Strict,
    BestEffort,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WorldFsDimensionPolicy {
    pub allow_list: Vec<String>,
    #[serde(default)]
    pub deny_list: Vec<String>,
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
pub enum WorldFsIsolation {
    Workspace,
    Full,
}

impl WorldFsIsolation {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Workspace => "workspace",
            Self::Full => "full",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        value.parse().ok()
    }
}

impl FromStr for WorldFsIsolation {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value.trim().to_ascii_lowercase().as_str() {
            "workspace" => Ok(Self::Workspace),
            "full" => Ok(Self::Full),
            other => Err(format!(
                "invalid world_fs.isolation: {} (expected workspace or full)",
                other
            )),
        }
    }
}

impl<'de> Deserialize<'de> for WorldFsIsolation {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let raw = String::deserialize(deserializer)?;
        raw.parse().map_err(serde::de::Error::custom)
    }
}

impl Serialize for WorldFsIsolation {
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
    pub isolation: WorldFsIsolation,
    pub require_world: bool,
    pub host_visible: bool,
    pub fail_closed_routing: bool,
    pub read_allowlist: Vec<String>,
    pub write_allowlist: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct Policy {
    pub id: String,
    pub name: String,

    // Filesystem (policy schema: world_fs.*)
    pub fs_read: Vec<String>,                 // world_fs.read_allowlist
    pub fs_write: Vec<String>,                // world_fs.write_allowlist
    pub world_fs_mode: WorldFsMode,           // world_fs.mode
    pub world_fs_isolation: WorldFsIsolation, // world_fs.isolation
    pub world_fs_require_world: bool,         // world_fs.require_world
    pub world_fs_enforcement: Option<WorldFsEnforcement>, // world_fs.enforcement (V2; full isolation only)
    pub world_fs_host_visible: bool,                      // world_fs.host_visible (V3)
    pub world_fs_fail_closed_routing: bool,               // world_fs.fail_closed.routing (V3)
    pub world_fs_deny_enforcement: Option<WorldFsDenyEnforcement>, // world_fs.deny_enforcement (V3)
    pub world_fs_caged_required: bool,                    // world_fs.caged_required (V3)
    pub world_fs_write_enabled: bool,                     // world_fs.write.enabled (V3)
    pub world_fs_discover: Option<WorldFsDimensionPolicy>, // world_fs.discover (V2; optional)
    pub world_fs_read: Option<WorldFsDimensionPolicy>,    // world_fs.read (V2)
    pub world_fs_write: Option<WorldFsDimensionPolicy>, // world_fs.write (V2; required iff mode=writable)

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
            world_fs_isolation: WorldFsIsolation::Workspace,
            world_fs_require_world: false,
            world_fs_enforcement: None,
            world_fs_host_visible: true,
            world_fs_fail_closed_routing: false,
            world_fs_deny_enforcement: None,
            world_fs_caged_required: false,
            world_fs_write_enabled: true,
            world_fs_discover: None,
            world_fs_read: None,
            world_fs_write: None,
            net_allowed: vec![],
            cmd_allowed: vec![],
            cmd_denied: vec![
                "rm -rf *".to_string(),
                "curl * | bash".to_string(),
                "wget * | bash".to_string(),
            ],
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
struct RawWorldFsV2 {
    mode: StrictWorldFsMode,
    #[serde(rename = "isolation")]
    isolation: WorldFsIsolation,
    require_world: bool,
    #[serde(default)]
    enforcement: Option<WorldFsEnforcement>,
    #[serde(default)]
    discover: Option<WorldFsDimensionPolicy>,
    #[serde(default)]
    read: Option<WorldFsDimensionPolicy>,
    #[serde(default)]
    write: Option<WorldFsDimensionPolicy>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawPolicyV2 {
    id: String,
    name: String,
    world_fs: RawWorldFsV2,

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
struct WorldFsDimensionFileV2<'a> {
    allow_list: &'a [String],
    #[serde(skip_serializing_if = "slice_is_empty")]
    deny_list: &'a [String],
}

fn slice_is_empty<T>(value: &[T]) -> bool {
    value.is_empty()
}

#[derive(Debug, Serialize)]
#[serde(deny_unknown_fields)]
struct WorldFsFileV2<'a> {
    mode: WorldFsMode,
    isolation: WorldFsIsolation,
    require_world: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    enforcement: Option<WorldFsEnforcement>,
    #[serde(skip_serializing_if = "Option::is_none")]
    discover: Option<WorldFsDimensionFileV2<'a>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    read: Option<WorldFsDimensionFileV2<'a>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    write: Option<WorldFsDimensionFileV2<'a>>,
}

#[derive(Debug, Serialize)]
#[serde(deny_unknown_fields)]
struct PolicyFileV2<'a> {
    id: &'a str,
    name: &'a str,
    world_fs: WorldFsFileV2<'a>,

    net_allowed: &'a [String],

    cmd_allowed: &'a [String],
    cmd_denied: &'a [String],
    cmd_isolated: &'a [String],

    require_approval: bool,
    allow_shell_operators: bool,

    limits: &'a ResourceLimits,

    metadata: SortedMap<'a>,
}

#[derive(Debug, Clone, Copy)]
struct SortedMap<'a>(&'a HashMap<String, String>);

impl Serialize for SortedMap<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut entries: Vec<_> = self.0.iter().collect();
        entries.sort_by(|(a_key, _), (b_key, _)| a_key.cmp(b_key));

        let mut map = serializer.serialize_map(Some(entries.len()))?;
        for (key, value) in entries {
            map.serialize_entry(key, value)?;
        }
        map.end()
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

impl Policy {
    pub fn world_fs_policy(&self) -> WorldFsPolicy {
        WorldFsPolicy {
            mode: self.world_fs_mode,
            isolation: self.world_fs_isolation,
            require_world: self.world_fs_require_world,
            host_visible: self.world_fs_host_visible,
            fail_closed_routing: self.world_fs_fail_closed_routing,
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

    fn validate_world_fs(_world_fs: &RawWorldFsV2) -> Result<(), String> {
        // Full V2 validation happens when resolving effective policy patches (see effective_policy).
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
        self.world_fs_isolation = match (self.world_fs_isolation, other.world_fs_isolation) {
            (WorldFsIsolation::Full, _) | (_, WorldFsIsolation::Full) => WorldFsIsolation::Full,
            _ => WorldFsIsolation::Workspace,
        };
        self.world_fs_require_world = self.world_fs_require_world || other.world_fs_require_world;
        self.world_fs_host_visible = self.world_fs_host_visible && other.world_fs_host_visible;
        self.world_fs_fail_closed_routing =
            self.world_fs_fail_closed_routing || other.world_fs_fail_closed_routing;
        self.world_fs_caged_required =
            self.world_fs_caged_required || other.world_fs_caged_required;
        self.world_fs_write_enabled = self.world_fs_write_enabled && other.world_fs_write_enabled;
        self.world_fs_deny_enforcement = merge_deny_enforcement(
            self.world_fs_deny_enforcement,
            other.world_fs_deny_enforcement,
        );

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

fn merge_deny_enforcement(
    a: Option<WorldFsDenyEnforcement>,
    b: Option<WorldFsDenyEnforcement>,
) -> Option<WorldFsDenyEnforcement> {
    use WorldFsDenyEnforcement as D;
    match (a, b) {
        (Some(D::Strict), _) | (_, Some(D::Strict)) => Some(D::Strict),
        (Some(D::PreferStrict), _) | (_, Some(D::PreferStrict)) => Some(D::PreferStrict),
        (Some(D::Weak), _) | (_, Some(D::Weak)) => Some(D::Weak),
        _ => None,
    }
}

impl<'de> Deserialize<'de> for Policy {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = serde_yaml::Value::deserialize(deserializer)?;
        let raw: RawPolicyV2 = serde_yaml::from_value(value).map_err(serde::de::Error::custom)?;
        Policy::validate_world_fs(&raw.world_fs).map_err(serde::de::Error::custom)?;

        Ok(Self {
            id: raw.id,
            name: raw.name,
            fs_read: raw
                .world_fs
                .read
                .as_ref()
                .map(|d| d.allow_list.clone())
                .unwrap_or_default(),
            fs_write: raw
                .world_fs
                .write
                .as_ref()
                .map(|d| d.allow_list.clone())
                .unwrap_or_default(),
            world_fs_mode: raw.world_fs.mode.0,
            world_fs_isolation: raw.world_fs.isolation,
            world_fs_require_world: raw.world_fs.require_world,
            world_fs_enforcement: raw.world_fs.enforcement,
            world_fs_host_visible: matches!(raw.world_fs.isolation, WorldFsIsolation::Workspace),
            world_fs_fail_closed_routing: raw.world_fs.require_world,
            world_fs_deny_enforcement: raw.world_fs.enforcement.map(|e| match e {
                WorldFsEnforcement::Strict => WorldFsDenyEnforcement::Strict,
                WorldFsEnforcement::BestEffort => WorldFsDenyEnforcement::Weak,
            }),
            world_fs_caged_required: false,
            world_fs_write_enabled: matches!(raw.world_fs.mode.0, WorldFsMode::Writable),
            world_fs_discover: raw.world_fs.discover,
            world_fs_read: raw.world_fs.read,
            world_fs_write: raw.world_fs.write,
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
        let read = self.world_fs_read.as_ref().map(|d| WorldFsDimensionFileV2 {
            allow_list: &d.allow_list,
            deny_list: &d.deny_list,
        });
        let discover = self
            .world_fs_discover
            .as_ref()
            .map(|d| WorldFsDimensionFileV2 {
                allow_list: &d.allow_list,
                deny_list: &d.deny_list,
            });
        let write = self
            .world_fs_write
            .as_ref()
            .map(|d| WorldFsDimensionFileV2 {
                allow_list: &d.allow_list,
                deny_list: &d.deny_list,
            });

        let file = PolicyFileV2 {
            id: &self.id,
            name: &self.name,
            world_fs: WorldFsFileV2 {
                mode: self.world_fs_mode,
                isolation: self.world_fs_isolation,
                require_world: self.world_fs_require_world,
                enforcement: self.world_fs_enforcement,
                discover,
                read,
                write,
            },
            net_allowed: &self.net_allowed,
            cmd_allowed: &self.cmd_allowed,
            cmd_denied: &self.cmd_denied,
            cmd_isolated: &self.cmd_isolated,
            require_approval: self.require_approval,
            allow_shell_operators: self.allow_shell_operators,
            limits: &self.limits,
            metadata: SortedMap(&self.metadata),
        };
        file.serialize(serializer)
    }
}

#[cfg(test)]
mod tests;
