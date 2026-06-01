use serde::ser::SerializeMap;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::str::FromStr;
use substrate_common::WorldFsMode;

fn default_allow_shell_operators() -> bool {
    true
}

fn matches_backend_kind(value: &str) -> bool {
    !value.is_empty()
        && value
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_')
}

fn matches_backend_name(value: &str) -> bool {
    !value.is_empty()
        && value.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_' || byte == b'-'
        })
}

fn matches_identity_snake_case_id(value: &str) -> bool {
    let mut bytes = value.bytes();
    match bytes.next() {
        Some(byte) if byte.is_ascii_lowercase() => {}
        _ => return false,
    }

    let mut prev_underscore = false;
    for byte in bytes {
        match byte {
            b'a'..=b'z' | b'0'..=b'9' => prev_underscore = false,
            b'_' if !prev_underscore => prev_underscore = true,
            _ => return false,
        }
    }

    !prev_underscore
}

fn matches_identity_dotted_id(value: &str) -> bool {
    let mut saw_dot = false;
    for segment in value.split('.') {
        if !matches_identity_snake_case_id(segment) {
            return false;
        }
        saw_dot = true;
    }
    saw_dot && value.contains('.')
}

pub fn validate_backend_id(value: &str) -> Result<(), String> {
    let trimmed = value.trim();
    let Some((kind, name)) = trimmed.split_once(':') else {
        return Err(format!(
            "invalid backend id '{}'; expected <kind>:<name>",
            trimmed
        ));
    };
    if !matches_backend_kind(kind) || !matches_backend_name(name) || name.contains(':') {
        return Err(format!(
            "invalid backend id '{}'; expected <kind>:<name> with kind [a-z0-9_]+ and name [a-z0-9_-]+",
            trimmed
        ));
    }
    Ok(())
}

pub fn validate_snake_case_id(value: &str) -> Result<(), String> {
    let trimmed = value.trim();
    if matches_identity_snake_case_id(trimmed) {
        Ok(())
    } else {
        Err(format!(
            "invalid snake_case id '{}'; expected lowercase snake_case id",
            trimmed
        ))
    }
}

pub fn validate_dotted_id(value: &str) -> Result<(), String> {
    let trimmed = value.trim();
    if matches_identity_dotted_id(trimmed) {
        Ok(())
    } else {
        Err(format!(
            "invalid dotted id '{}'; expected lowercase dotted id",
            trimmed
        ))
    }
}

pub fn validate_world_dispatch_action_id(value: &str) -> Result<(), String> {
    let trimmed = value.trim();
    match trimmed {
        "run_world_task"
        | "spawn_world_worker"
        | "continue_world_worker"
        | "inspect_world_worker" => Ok(()),
        _ => Err(format!(
            "invalid world dispatch action '{}'; expected one of run_world_task, spawn_world_worker, continue_world_worker, inspect_world_worker",
            trimmed
        )),
    }
}

pub fn validate_world_dispatch_mode_id(value: &str) -> Result<(), String> {
    let trimmed = value.trim();
    match trimmed {
        "ephemeral" | "retained" => Ok(()),
        _ => Err(format!(
            "invalid world dispatch mode '{}'; expected one of ephemeral, retained",
            trimmed
        )),
    }
}

fn validate_backend_ids(values: &[String], key: &str) -> Result<(), String> {
    for value in values {
        validate_backend_id(value).map_err(|_| {
            format!(
                "invalid {} entry '{}'; expected <kind>:<name> with kind [a-z0-9_]+ and name [a-z0-9_-]+",
                key,
                value.trim()
            )
        })?;
    }
    Ok(())
}

fn validate_snake_case_ids(values: &[String], key: &str) -> Result<(), String> {
    for value in values {
        validate_snake_case_id(value).map_err(|_| {
            format!(
                "invalid {} entry '{}'; expected lowercase snake_case id",
                key,
                value.trim()
            )
        })?;
    }
    Ok(())
}

fn validate_dotted_ids(values: &[String], key: &str) -> Result<(), String> {
    for value in values {
        validate_dotted_id(value).map_err(|_| {
            format!(
                "invalid {} entry '{}'; expected lowercase dotted id",
                key,
                value.trim()
            )
        })?;
    }
    Ok(())
}

fn validate_world_dispatch_action_ids(values: &[String], key: &str) -> Result<(), String> {
    for value in values {
        validate_world_dispatch_action_id(value).map_err(|_| {
            format!(
                "invalid {} entry '{}'; expected one of run_world_task, spawn_world_worker, continue_world_worker",
                key,
                value.trim()
            )
        })?;
    }
    Ok(())
}

fn validate_world_dispatch_mode_ids(values: &[String], key: &str) -> Result<(), String> {
    for value in values {
        validate_world_dispatch_mode_id(value).map_err(|_| {
            format!(
                "invalid {} entry '{}'; expected one of ephemeral, retained",
                key,
                value.trim()
            )
        })?;
    }
    Ok(())
}

fn intersect_ordered(lhs: &[String], rhs: &[String]) -> Vec<String> {
    lhs.iter()
        .filter(|value| rhs.iter().any(|other| other == *value))
        .cloned()
        .collect()
}

fn narrow_constraints(lhs: &[String], rhs: &[String]) -> Vec<String> {
    if lhs.is_empty() {
        return rhs.to_vec();
    }
    if rhs.is_empty() {
        return lhs.to_vec();
    }
    intersect_ordered(lhs, rhs)
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
    pub caged_required: bool,
    pub read_allowlist: Vec<String>,
    pub write_allowlist: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorldDispatchPolicy {
    pub enabled: bool,
    pub allowed_backends: Vec<String>,
    pub allowed_actions: Vec<String>,
    pub allowed_modes: Vec<String>,
    pub same_session_only: bool,
    pub same_world_binding_only: bool,
    pub allow_capability_narrowing: bool,
    pub max_live_retained_workers: u32,
    pub max_concurrent_ephemeral: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct LlmPolicyFileV1 {
    fail_closed: LlmFailClosedPolicyFileV1,
    require_approval: bool,
    allowed_backends: Vec<String>,
    constraints: LlmConstraintsPolicyFileV1,
    secrets: LlmSecretsPolicyFileV1,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct LlmFailClosedPolicyFileV1 {
    routing: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct LlmSecretsPolicyFileV1 {
    env_allowed: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct LlmConstraintsPolicyFileV1 {
    routers: Vec<String>,
    providers: Vec<String>,
    protocols: Vec<String>,
    auth_authorities: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct AgentsPolicyFileV1 {
    allowed_backends: Vec<String>,
    fail_closed: AgentsFailClosedPolicyFileV1,
    host_credentials: AgentsHostCredentialsPolicyFileV1,
    world_dispatch: AgentsWorldDispatchPolicyFileV1,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct AgentsFailClosedPolicyFileV1 {
    routing: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct AgentsHostCredentialsPolicyFileV1 {
    read: AgentsHostCredentialsReadPolicyFileV1,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct AgentsHostCredentialsReadPolicyFileV1 {
    allowed_backends: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct AgentsWorldDispatchPolicyFileV1 {
    enabled: bool,
    allowed_backends: Vec<String>,
    allowed_actions: Vec<String>,
    allowed_modes: Vec<String>,
    same_session_only: bool,
    same_world_binding_only: bool,
    allow_capability_narrowing: bool,
    max_live_retained_workers: u32,
    max_concurrent_ephemeral: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct WorkflowPolicyFileV1 {
    router: WorkflowRouterPolicyFileV1,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct WorkflowRouterPolicyFileV1 {
    enabled: bool,
    allow_cross_workspace: bool,
    allowed_rule_ids: Vec<String>,
    allowed_workflow_ids: Vec<String>,
    allowed_target_workspace_ids: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
struct RawLlmPolicyV1 {
    fail_closed: RawLlmFailClosedPolicyV1,
    require_approval: bool,
    allowed_backends: Vec<String>,
    constraints: RawLlmConstraintsPolicyV1,
    secrets: RawLlmSecretsPolicyV1,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(default, deny_unknown_fields)]
struct RawLlmFailClosedPolicyV1 {
    routing: bool,
}

impl Default for RawLlmFailClosedPolicyV1 {
    fn default() -> Self {
        Self { routing: true }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
struct RawLlmSecretsPolicyV1 {
    env_allowed: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
struct RawLlmConstraintsPolicyV1 {
    routers: Vec<String>,
    providers: Vec<String>,
    protocols: Vec<String>,
    auth_authorities: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
struct RawAgentsPolicyV1 {
    allowed_backends: Vec<String>,
    fail_closed: RawAgentsFailClosedPolicyV1,
    host_credentials: RawAgentsHostCredentialsPolicyV1,
    world_dispatch: RawAgentsWorldDispatchPolicyV1,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(default, deny_unknown_fields)]
struct RawAgentsFailClosedPolicyV1 {
    routing: bool,
}

impl Default for RawAgentsFailClosedPolicyV1 {
    fn default() -> Self {
        Self { routing: true }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
struct RawAgentsHostCredentialsPolicyV1 {
    read: RawAgentsHostCredentialsReadPolicyV1,
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
struct RawAgentsHostCredentialsReadPolicyV1 {
    allowed_backends: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(default, deny_unknown_fields)]
struct RawAgentsWorldDispatchPolicyV1 {
    enabled: bool,
    allowed_backends: Vec<String>,
    allowed_actions: Vec<String>,
    allowed_modes: Vec<String>,
    same_session_only: bool,
    same_world_binding_only: bool,
    allow_capability_narrowing: bool,
    max_live_retained_workers: u32,
    max_concurrent_ephemeral: u32,
}

impl Default for RawAgentsWorldDispatchPolicyV1 {
    fn default() -> Self {
        Self {
            enabled: false,
            allowed_backends: Vec::new(),
            allowed_actions: Vec::new(),
            allowed_modes: Vec::new(),
            same_session_only: true,
            same_world_binding_only: true,
            allow_capability_narrowing: false,
            max_live_retained_workers: 0,
            max_concurrent_ephemeral: 0,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
struct RawWorkflowPolicyV1 {
    router: RawWorkflowRouterPolicyV1,
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
struct RawWorkflowRouterPolicyV1 {
    enabled: bool,
    allow_cross_workspace: bool,
    allowed_rule_ids: Vec<String>,
    allowed_workflow_ids: Vec<String>,
    allowed_target_workspace_ids: Vec<String>,
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

    // LLM
    pub llm_fail_closed_routing: bool, // llm.fail_closed.routing
    pub llm_require_approval: bool,    // llm.require_approval
    pub llm_allowed_backends: Vec<String>, // llm.allowed_backends
    pub llm_constraints_routers: Vec<String>, // llm.constraints.routers
    pub llm_constraints_providers: Vec<String>, // llm.constraints.providers
    pub llm_constraints_protocols: Vec<String>, // llm.constraints.protocols
    pub llm_constraints_auth_authorities: Vec<String>, // llm.constraints.auth_authorities
    pub llm_secrets_env_allowed: Vec<String>, // llm.secrets.env_allowed

    // Agents
    pub agents_allowed_backends: Vec<String>, // agents.allowed_backends
    pub agents_fail_closed_routing: bool,     // agents.fail_closed.routing
    pub agents_host_credentials_read_allowed_backends: Vec<String>, // agents.host_credentials.read.allowed_backends
    pub agents_world_dispatch_enabled: bool,                        // agents.world_dispatch.enabled
    pub agents_world_dispatch_allowed_backends: Vec<String>, // agents.world_dispatch.allowed_backends
    pub agents_world_dispatch_allowed_actions: Vec<String>, // agents.world_dispatch.allowed_actions
    pub agents_world_dispatch_allowed_modes: Vec<String>,   // agents.world_dispatch.allowed_modes
    pub agents_world_dispatch_same_session_only: bool, // agents.world_dispatch.same_session_only
    pub agents_world_dispatch_same_world_binding_only: bool, // agents.world_dispatch.same_world_binding_only
    pub agents_world_dispatch_allow_capability_narrowing: bool, // agents.world_dispatch.allow_capability_narrowing
    pub agents_world_dispatch_max_live_retained_workers: u32, // agents.world_dispatch.max_live_retained_workers
    pub agents_world_dispatch_max_concurrent_ephemeral: u32, // agents.world_dispatch.max_concurrent_ephemeral

    // Workflow router
    pub workflow_router_enabled: bool, // workflow.router.enabled
    pub workflow_router_allow_cross_workspace: bool, // workflow.router.allow_cross_workspace
    pub workflow_router_allowed_rule_ids: Vec<String>, // workflow.router.allowed_rule_ids
    pub workflow_router_allowed_workflow_ids: Vec<String>, // workflow.router.allowed_workflow_ids
    pub workflow_router_allowed_target_workspace_ids: Vec<String>, // workflow.router.allowed_target_workspace_ids

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
            llm_fail_closed_routing: true,
            llm_require_approval: false,
            llm_allowed_backends: Vec::new(),
            llm_constraints_routers: Vec::new(),
            llm_constraints_providers: Vec::new(),
            llm_constraints_protocols: Vec::new(),
            llm_constraints_auth_authorities: Vec::new(),
            llm_secrets_env_allowed: Vec::new(),
            agents_allowed_backends: Vec::new(),
            agents_fail_closed_routing: true,
            agents_host_credentials_read_allowed_backends: Vec::new(),
            agents_world_dispatch_enabled: false,
            agents_world_dispatch_allowed_backends: Vec::new(),
            agents_world_dispatch_allowed_actions: Vec::new(),
            agents_world_dispatch_allowed_modes: Vec::new(),
            agents_world_dispatch_same_session_only: true,
            agents_world_dispatch_same_world_binding_only: true,
            agents_world_dispatch_allow_capability_narrowing: false,
            agents_world_dispatch_max_live_retained_workers: 0,
            agents_world_dispatch_max_concurrent_ephemeral: 0,
            workflow_router_enabled: false,
            workflow_router_allow_cross_workspace: false,
            workflow_router_allowed_rule_ids: Vec::new(),
            workflow_router_allowed_workflow_ids: Vec::new(),
            workflow_router_allowed_target_workspace_ids: Vec::new(),
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
    #[serde(default)]
    llm: RawLlmPolicyV1,
    #[serde(default)]
    agents: RawAgentsPolicyV1,
    #[serde(default)]
    workflow: RawWorkflowPolicyV1,

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
    llm: LlmPolicyFileV1,
    agents: AgentsPolicyFileV1,
    workflow: WorkflowPolicyFileV1,

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
        entries.sort_by_key(|(key, _)| *key);

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
            caged_required: self.world_fs_caged_required,
            read_allowlist: self.fs_read.clone(),
            write_allowlist: self.fs_write.clone(),
        }
    }

    pub fn world_dispatch_policy(&self) -> WorldDispatchPolicy {
        WorldDispatchPolicy {
            enabled: self.agents_world_dispatch_enabled,
            allowed_backends: self.agents_world_dispatch_allowed_backends.clone(),
            allowed_actions: self.agents_world_dispatch_allowed_actions.clone(),
            allowed_modes: self.agents_world_dispatch_allowed_modes.clone(),
            same_session_only: self.agents_world_dispatch_same_session_only,
            same_world_binding_only: self.agents_world_dispatch_same_world_binding_only,
            allow_capability_narrowing: self.agents_world_dispatch_allow_capability_narrowing,
            max_live_retained_workers: self.agents_world_dispatch_max_live_retained_workers,
            max_concurrent_ephemeral: self.agents_world_dispatch_max_concurrent_ephemeral,
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
        self.llm_fail_closed_routing =
            self.llm_fail_closed_routing || other.llm_fail_closed_routing;
        self.llm_require_approval = self.llm_require_approval || other.llm_require_approval;
        self.llm_allowed_backends =
            intersect_ordered(&self.llm_allowed_backends, &other.llm_allowed_backends);
        self.llm_constraints_routers = narrow_constraints(
            &self.llm_constraints_routers,
            &other.llm_constraints_routers,
        );
        self.llm_constraints_providers = narrow_constraints(
            &self.llm_constraints_providers,
            &other.llm_constraints_providers,
        );
        self.llm_constraints_protocols = narrow_constraints(
            &self.llm_constraints_protocols,
            &other.llm_constraints_protocols,
        );
        self.llm_constraints_auth_authorities = narrow_constraints(
            &self.llm_constraints_auth_authorities,
            &other.llm_constraints_auth_authorities,
        );
        self.llm_secrets_env_allowed = intersect_ordered(
            &self.llm_secrets_env_allowed,
            &other.llm_secrets_env_allowed,
        );
        self.agents_allowed_backends = intersect_ordered(
            &self.agents_allowed_backends,
            &other.agents_allowed_backends,
        );
        self.agents_fail_closed_routing =
            self.agents_fail_closed_routing || other.agents_fail_closed_routing;
        self.agents_host_credentials_read_allowed_backends = intersect_ordered(
            &self.agents_host_credentials_read_allowed_backends,
            &other.agents_host_credentials_read_allowed_backends,
        );
        self.agents_world_dispatch_enabled =
            self.agents_world_dispatch_enabled && other.agents_world_dispatch_enabled;
        self.agents_world_dispatch_allowed_backends = intersect_ordered(
            &self.agents_world_dispatch_allowed_backends,
            &other.agents_world_dispatch_allowed_backends,
        );
        self.agents_world_dispatch_allowed_actions = intersect_ordered(
            &self.agents_world_dispatch_allowed_actions,
            &other.agents_world_dispatch_allowed_actions,
        );
        self.agents_world_dispatch_allowed_modes = intersect_ordered(
            &self.agents_world_dispatch_allowed_modes,
            &other.agents_world_dispatch_allowed_modes,
        );
        self.agents_world_dispatch_same_session_only = self.agents_world_dispatch_same_session_only
            || other.agents_world_dispatch_same_session_only;
        self.agents_world_dispatch_same_world_binding_only = self
            .agents_world_dispatch_same_world_binding_only
            || other.agents_world_dispatch_same_world_binding_only;
        self.agents_world_dispatch_allow_capability_narrowing = self
            .agents_world_dispatch_allow_capability_narrowing
            && other.agents_world_dispatch_allow_capability_narrowing;
        self.agents_world_dispatch_max_live_retained_workers = self
            .agents_world_dispatch_max_live_retained_workers
            .min(other.agents_world_dispatch_max_live_retained_workers);
        self.agents_world_dispatch_max_concurrent_ephemeral = self
            .agents_world_dispatch_max_concurrent_ephemeral
            .min(other.agents_world_dispatch_max_concurrent_ephemeral);
        self.workflow_router_enabled =
            self.workflow_router_enabled && other.workflow_router_enabled;
        self.workflow_router_allow_cross_workspace = self.workflow_router_allow_cross_workspace
            && other.workflow_router_allow_cross_workspace;
        self.workflow_router_allowed_rule_ids = intersect_ordered(
            &self.workflow_router_allowed_rule_ids,
            &other.workflow_router_allowed_rule_ids,
        );
        self.workflow_router_allowed_workflow_ids = intersect_ordered(
            &self.workflow_router_allowed_workflow_ids,
            &other.workflow_router_allowed_workflow_ids,
        );
        self.workflow_router_allowed_target_workspace_ids = intersect_ordered(
            &self.workflow_router_allowed_target_workspace_ids,
            &other.workflow_router_allowed_target_workspace_ids,
        );
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
        let policy = Self {
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
            llm_fail_closed_routing: raw.llm.fail_closed.routing,
            llm_require_approval: raw.llm.require_approval,
            llm_allowed_backends: raw.llm.allowed_backends,
            llm_constraints_routers: raw.llm.constraints.routers,
            llm_constraints_providers: raw.llm.constraints.providers,
            llm_constraints_protocols: raw.llm.constraints.protocols,
            llm_constraints_auth_authorities: raw.llm.constraints.auth_authorities,
            llm_secrets_env_allowed: raw.llm.secrets.env_allowed,
            agents_allowed_backends: raw.agents.allowed_backends,
            agents_fail_closed_routing: raw.agents.fail_closed.routing,
            agents_host_credentials_read_allowed_backends: raw
                .agents
                .host_credentials
                .read
                .allowed_backends,
            agents_world_dispatch_enabled: raw.agents.world_dispatch.enabled,
            agents_world_dispatch_allowed_backends: raw.agents.world_dispatch.allowed_backends,
            agents_world_dispatch_allowed_actions: raw.agents.world_dispatch.allowed_actions,
            agents_world_dispatch_allowed_modes: raw.agents.world_dispatch.allowed_modes,
            agents_world_dispatch_same_session_only: raw.agents.world_dispatch.same_session_only,
            agents_world_dispatch_same_world_binding_only: raw
                .agents
                .world_dispatch
                .same_world_binding_only,
            agents_world_dispatch_allow_capability_narrowing: raw
                .agents
                .world_dispatch
                .allow_capability_narrowing,
            agents_world_dispatch_max_live_retained_workers: raw
                .agents
                .world_dispatch
                .max_live_retained_workers,
            agents_world_dispatch_max_concurrent_ephemeral: raw
                .agents
                .world_dispatch
                .max_concurrent_ephemeral,
            workflow_router_enabled: raw.workflow.router.enabled,
            workflow_router_allow_cross_workspace: raw.workflow.router.allow_cross_workspace,
            workflow_router_allowed_rule_ids: raw.workflow.router.allowed_rule_ids,
            workflow_router_allowed_workflow_ids: raw.workflow.router.allowed_workflow_ids,
            workflow_router_allowed_target_workspace_ids: raw
                .workflow
                .router
                .allowed_target_workspace_ids,
            net_allowed: raw.net_allowed,
            cmd_allowed: raw.cmd_allowed,
            cmd_denied: raw.cmd_denied,
            cmd_isolated: raw.cmd_isolated,
            require_approval: raw.require_approval,
            allow_shell_operators: raw.allow_shell_operators,
            limits: raw.limits,
            metadata: raw.metadata,
        };
        validate_backend_ids(&policy.llm_allowed_backends, "llm.allowed_backends")
            .map_err(serde::de::Error::custom)?;
        validate_snake_case_ids(&policy.llm_constraints_routers, "llm.constraints.routers")
            .map_err(serde::de::Error::custom)?;
        validate_snake_case_ids(
            &policy.llm_constraints_providers,
            "llm.constraints.providers",
        )
        .map_err(serde::de::Error::custom)?;
        validate_dotted_ids(
            &policy.llm_constraints_protocols,
            "llm.constraints.protocols",
        )
        .map_err(serde::de::Error::custom)?;
        validate_snake_case_ids(
            &policy.llm_constraints_auth_authorities,
            "llm.constraints.auth_authorities",
        )
        .map_err(serde::de::Error::custom)?;
        validate_backend_ids(&policy.agents_allowed_backends, "agents.allowed_backends")
            .map_err(serde::de::Error::custom)?;
        validate_backend_ids(
            &policy.agents_host_credentials_read_allowed_backends,
            "agents.host_credentials.read.allowed_backends",
        )
        .map_err(serde::de::Error::custom)?;
        validate_backend_ids(
            &policy.agents_world_dispatch_allowed_backends,
            "agents.world_dispatch.allowed_backends",
        )
        .map_err(serde::de::Error::custom)?;
        validate_world_dispatch_action_ids(
            &policy.agents_world_dispatch_allowed_actions,
            "agents.world_dispatch.allowed_actions",
        )
        .map_err(serde::de::Error::custom)?;
        validate_world_dispatch_mode_ids(
            &policy.agents_world_dispatch_allowed_modes,
            "agents.world_dispatch.allowed_modes",
        )
        .map_err(serde::de::Error::custom)?;
        Ok(policy)
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
            llm: LlmPolicyFileV1 {
                fail_closed: LlmFailClosedPolicyFileV1 {
                    routing: self.llm_fail_closed_routing,
                },
                require_approval: self.llm_require_approval,
                allowed_backends: self.llm_allowed_backends.clone(),
                constraints: LlmConstraintsPolicyFileV1 {
                    routers: self.llm_constraints_routers.clone(),
                    providers: self.llm_constraints_providers.clone(),
                    protocols: self.llm_constraints_protocols.clone(),
                    auth_authorities: self.llm_constraints_auth_authorities.clone(),
                },
                secrets: LlmSecretsPolicyFileV1 {
                    env_allowed: self.llm_secrets_env_allowed.clone(),
                },
            },
            agents: AgentsPolicyFileV1 {
                allowed_backends: self.agents_allowed_backends.clone(),
                fail_closed: AgentsFailClosedPolicyFileV1 {
                    routing: self.agents_fail_closed_routing,
                },
                host_credentials: AgentsHostCredentialsPolicyFileV1 {
                    read: AgentsHostCredentialsReadPolicyFileV1 {
                        allowed_backends: self
                            .agents_host_credentials_read_allowed_backends
                            .clone(),
                    },
                },
                world_dispatch: AgentsWorldDispatchPolicyFileV1 {
                    enabled: self.agents_world_dispatch_enabled,
                    allowed_backends: self.agents_world_dispatch_allowed_backends.clone(),
                    allowed_actions: self.agents_world_dispatch_allowed_actions.clone(),
                    allowed_modes: self.agents_world_dispatch_allowed_modes.clone(),
                    same_session_only: self.agents_world_dispatch_same_session_only,
                    same_world_binding_only: self.agents_world_dispatch_same_world_binding_only,
                    allow_capability_narrowing: self
                        .agents_world_dispatch_allow_capability_narrowing,
                    max_live_retained_workers: self.agents_world_dispatch_max_live_retained_workers,
                    max_concurrent_ephemeral: self.agents_world_dispatch_max_concurrent_ephemeral,
                },
            },
            workflow: WorkflowPolicyFileV1 {
                router: WorkflowRouterPolicyFileV1 {
                    enabled: self.workflow_router_enabled,
                    allow_cross_workspace: self.workflow_router_allow_cross_workspace,
                    allowed_rule_ids: self.workflow_router_allowed_rule_ids.clone(),
                    allowed_workflow_ids: self.workflow_router_allowed_workflow_ids.clone(),
                    allowed_target_workspace_ids: self
                        .workflow_router_allowed_target_workspace_ids
                        .clone(),
                },
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
