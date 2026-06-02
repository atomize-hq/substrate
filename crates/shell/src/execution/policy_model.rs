use crate::execution::config_model::{self, ConfigUpdate, UpdateOp};
use crate::execution::value_parse::parse_bool_flag;
use crate::execution::workspace;
use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use substrate_broker::{
    validate_backend_id, validate_dotted_id, validate_snake_case_id,
    validate_world_dispatch_action_id, validate_world_dispatch_mode_id, Policy, PolicyExplainV1,
    WorldFsDenyEnforcement, WorldFsDimensionPolicy,
};

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default, deny_unknown_fields)]
pub(crate) struct PolicyPatch {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "WorldFsPatch::is_empty")]
    pub world_fs: WorldFsPatch,
    #[serde(skip_serializing_if = "LlmPatch::is_empty")]
    pub llm: LlmPatch,
    #[serde(skip_serializing_if = "AgentsPatch::is_empty")]
    pub agents: AgentsPatch,
    #[serde(skip_serializing_if = "WorkflowPatch::is_empty")]
    pub workflow: WorkflowPatch,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub net_allowed: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cmd_allowed: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cmd_denied: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cmd_isolated: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub require_approval: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allow_shell_operators: Option<bool>,
    #[serde(skip_serializing_if = "ResourceLimitsPatch::is_empty")]
    pub limits: ResourceLimitsPatch,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<BTreeMap<String, String>>,
}

impl PolicyPatch {
    pub(crate) fn is_empty(&self) -> bool {
        self.id.is_none()
            && self.name.is_none()
            && self.world_fs.is_empty()
            && self.llm.is_empty()
            && self.agents.is_empty()
            && self.workflow.is_empty()
            && self.net_allowed.is_none()
            && self.cmd_allowed.is_none()
            && self.cmd_denied.is_none()
            && self.cmd_isolated.is_none()
            && self.require_approval.is_none()
            && self.allow_shell_operators.is_none()
            && self.limits.is_empty()
            && self.metadata.is_none()
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default, deny_unknown_fields)]
pub(crate) struct LlmPatch {
    #[serde(skip_serializing_if = "LlmFailClosedPatch::is_empty")]
    pub fail_closed: LlmFailClosedPatch,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub require_approval: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allowed_backends: Option<Vec<String>>,
    #[serde(skip_serializing_if = "LlmConstraintsPatch::is_empty")]
    pub constraints: LlmConstraintsPatch,
    #[serde(skip_serializing_if = "LlmSecretsPatch::is_empty")]
    pub secrets: LlmSecretsPatch,
}

impl LlmPatch {
    fn is_empty(&self) -> bool {
        self.fail_closed.is_empty()
            && self.require_approval.is_none()
            && self.allowed_backends.is_none()
            && self.constraints.is_empty()
            && self.secrets.is_empty()
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default, deny_unknown_fields)]
pub(crate) struct LlmFailClosedPatch {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub routing: Option<bool>,
}

impl LlmFailClosedPatch {
    fn is_empty(&self) -> bool {
        self.routing.is_none()
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default, deny_unknown_fields)]
pub(crate) struct LlmSecretsPatch {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub env_allowed: Option<Vec<String>>,
}

impl LlmSecretsPatch {
    fn is_empty(&self) -> bool {
        self.env_allowed.is_none()
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default, deny_unknown_fields)]
pub(crate) struct LlmConstraintsPatch {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub routers: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub providers: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub protocols: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auth_authorities: Option<Vec<String>>,
}

impl LlmConstraintsPatch {
    fn is_empty(&self) -> bool {
        self.routers.is_none()
            && self.providers.is_none()
            && self.protocols.is_none()
            && self.auth_authorities.is_none()
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default, deny_unknown_fields)]
pub(crate) struct AgentsPatch {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allowed_backends: Option<Vec<String>>,
    #[serde(skip_serializing_if = "AgentsFailClosedPatch::is_empty")]
    pub fail_closed: AgentsFailClosedPatch,
    #[serde(skip_serializing_if = "AgentsHostCredentialsPatch::is_empty")]
    pub host_credentials: AgentsHostCredentialsPatch,
    #[serde(skip_serializing_if = "AgentsWorldDispatchPatch::is_empty")]
    pub world_dispatch: AgentsWorldDispatchPatch,
}

impl AgentsPatch {
    fn is_empty(&self) -> bool {
        self.allowed_backends.is_none()
            && self.fail_closed.is_empty()
            && self.host_credentials.is_empty()
            && self.world_dispatch.is_empty()
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default, deny_unknown_fields)]
pub(crate) struct AgentsFailClosedPatch {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub routing: Option<bool>,
}

impl AgentsFailClosedPatch {
    fn is_empty(&self) -> bool {
        self.routing.is_none()
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default, deny_unknown_fields)]
pub(crate) struct AgentsHostCredentialsPatch {
    #[serde(skip_serializing_if = "AgentsHostCredentialsReadPatch::is_empty")]
    pub read: AgentsHostCredentialsReadPatch,
}

impl AgentsHostCredentialsPatch {
    fn is_empty(&self) -> bool {
        self.read.is_empty()
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default, deny_unknown_fields)]
pub(crate) struct AgentsHostCredentialsReadPatch {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allowed_backends: Option<Vec<String>>,
}

impl AgentsHostCredentialsReadPatch {
    fn is_empty(&self) -> bool {
        self.allowed_backends.is_none()
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default, deny_unknown_fields)]
pub(crate) struct AgentsWorldDispatchPatch {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enabled: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allowed_backends: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allowed_actions: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allowed_modes: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub same_session_only: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub same_world_binding_only: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allow_capability_narrowing: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_live_retained_workers: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_concurrent_ephemeral: Option<u32>,
}

impl AgentsWorldDispatchPatch {
    fn is_empty(&self) -> bool {
        self.enabled.is_none()
            && self.allowed_backends.is_none()
            && self.allowed_actions.is_none()
            && self.allowed_modes.is_none()
            && self.same_session_only.is_none()
            && self.same_world_binding_only.is_none()
            && self.allow_capability_narrowing.is_none()
            && self.max_live_retained_workers.is_none()
            && self.max_concurrent_ephemeral.is_none()
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default, deny_unknown_fields)]
pub(crate) struct WorkflowPatch {
    #[serde(skip_serializing_if = "WorkflowRouterPatch::is_empty")]
    pub router: WorkflowRouterPatch,
}

impl WorkflowPatch {
    fn is_empty(&self) -> bool {
        self.router.is_empty()
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default, deny_unknown_fields)]
pub(crate) struct WorkflowRouterPatch {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enabled: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allow_cross_workspace: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allowed_rule_ids: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allowed_workflow_ids: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allowed_target_workspace_ids: Option<Vec<String>>,
}

impl WorkflowRouterPatch {
    fn is_empty(&self) -> bool {
        self.enabled.is_none()
            && self.allow_cross_workspace.is_none()
            && self.allowed_rule_ids.is_none()
            && self.allowed_workflow_ids.is_none()
            && self.allowed_target_workspace_ids.is_none()
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default, deny_unknown_fields)]
pub(crate) struct WorldFsPatch {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub host_visible: Option<bool>,
    #[serde(skip_serializing_if = "WorldFsFailClosedPatch::is_empty")]
    pub fail_closed: WorldFsFailClosedPatch,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deny_enforcement: Option<WorldFsDenyEnforcement>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub caged_required: Option<bool>,
    #[serde(skip_serializing_if = "WorldFsDimensionPatch::is_empty")]
    pub discover: WorldFsDimensionPatch,
    #[serde(skip_serializing_if = "WorldFsDimensionPatch::is_empty")]
    pub read: WorldFsDimensionPatch,
    #[serde(skip_serializing_if = "WorldFsWritePatch::is_empty")]
    pub write: WorldFsWritePatch,
}

impl WorldFsPatch {
    fn is_empty(&self) -> bool {
        self.host_visible.is_none()
            && self.fail_closed.is_empty()
            && self.deny_enforcement.is_none()
            && self.caged_required.is_none()
            && self.discover.is_empty()
            && self.read.is_empty()
            && self.write.is_empty()
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default, deny_unknown_fields)]
pub(crate) struct WorldFsFailClosedPatch {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub routing: Option<bool>,
}

impl WorldFsFailClosedPatch {
    fn is_empty(&self) -> bool {
        self.routing.is_none()
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default, deny_unknown_fields)]
pub(crate) struct WorldFsDimensionPatch {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allow_list: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deny_list: Option<Vec<String>>,
}

impl WorldFsDimensionPatch {
    fn is_empty(&self) -> bool {
        self.allow_list.is_none() && self.deny_list.is_none()
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default, deny_unknown_fields)]
pub(crate) struct WorldFsWritePatch {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enabled: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allow_list: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deny_list: Option<Vec<String>>,
}

impl WorldFsWritePatch {
    fn is_empty(&self) -> bool {
        self.enabled.is_none() && self.allow_list.is_none() && self.deny_list.is_none()
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default, deny_unknown_fields)]
pub(crate) struct ResourceLimitsPatch {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_memory_mb: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_cpu_percent: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_runtime_ms: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_egress_bytes: Option<u64>,
}

impl ResourceLimitsPatch {
    fn is_empty(&self) -> bool {
        self.max_memory_mb.is_none()
            && self.max_cpu_percent.is_none()
            && self.max_runtime_ms.is_none()
            && self.max_egress_bytes.is_none()
    }
}

pub(crate) fn global_policy_path() -> Result<PathBuf> {
    substrate_common::paths::policy_file()
}

pub(crate) fn workspace_policy_path(workspace_root: &Path) -> PathBuf {
    workspace::workspace_policy_path(workspace_root)
}

pub(crate) fn read_global_policy_patch_or_empty() -> Result<(PolicyPatch, bool)> {
    let path = global_policy_path()?;
    match fs::read_to_string(&path) {
        Ok(raw) => Ok((parse_policy_patch_yaml(&path, &raw)?, true)),
        Err(err) if err.kind() == io::ErrorKind::NotFound => Ok((PolicyPatch::default(), false)),
        Err(err) => Err(anyhow!("failed to read {}: {err}", path.display())),
    }
}

pub(crate) fn parse_policy_patch_yaml(path: &Path, raw: &str) -> Result<PolicyPatch> {
    let value: serde_yaml::Value = serde_yaml::from_str(raw).map_err(|err| {
        config_model::user_error(format!(
            "invalid YAML in {}: {}",
            path.display(),
            err.to_string().trim()
        ))
    })?;

    match &value {
        serde_yaml::Value::Null => return Ok(PolicyPatch::default()),
        serde_yaml::Value::Mapping(map) if map.is_empty() => return Ok(PolicyPatch::default()),
        _ => {}
    }

    let parsed: PolicyPatch = serde_yaml::from_value(value).map_err(|err| {
        config_model::user_error(format!(
            "invalid YAML in {}: {}",
            path.display(),
            err.to_string().trim()
        ))
    })?;
    validate_policy_patch(&parsed)?;
    Ok(parsed)
}

fn validate_policy_patch(patch: &PolicyPatch) -> Result<()> {
    validate_backend_id_list_opt(&patch.llm.allowed_backends, "llm.allowed_backends")?;
    validate_snake_case_id_list_opt(&patch.llm.constraints.routers, "llm.constraints.routers")?;
    validate_snake_case_id_list_opt(
        &patch.llm.constraints.providers,
        "llm.constraints.providers",
    )?;
    validate_dotted_id_list_opt(
        &patch.llm.constraints.protocols,
        "llm.constraints.protocols",
    )?;
    validate_snake_case_id_list_opt(
        &patch.llm.constraints.auth_authorities,
        "llm.constraints.auth_authorities",
    )?;
    validate_backend_id_list_opt(&patch.agents.allowed_backends, "agents.allowed_backends")?;
    validate_backend_id_list_opt(
        &patch.agents.host_credentials.read.allowed_backends,
        "agents.host_credentials.read.allowed_backends",
    )?;
    validate_backend_id_list_opt(
        &patch.agents.world_dispatch.allowed_backends,
        "agents.world_dispatch.allowed_backends",
    )?;
    validate_world_dispatch_action_list_opt(
        &patch.agents.world_dispatch.allowed_actions,
        "agents.world_dispatch.allowed_actions",
    )?;
    validate_world_dispatch_mode_list_opt(
        &patch.agents.world_dispatch.allowed_modes,
        "agents.world_dispatch.allowed_modes",
    )?;
    Ok(())
}

fn validate_backend_id_list_opt(values: &Option<Vec<String>>, key: &str) -> Result<()> {
    let Some(values) = values else {
        return Ok(());
    };
    for value in values {
        validate_backend_id(value).map_err(|_| {
            config_model::user_error(format!(
                "invalid {} entry '{}'; expected <kind>:<name> with kind [a-z0-9_]+ and name [a-z0-9_-]+",
                key,
                value.trim()
            ))
        })?;
    }
    Ok(())
}

fn validate_snake_case_id_list_opt(values: &Option<Vec<String>>, key: &str) -> Result<()> {
    let Some(values) = values else {
        return Ok(());
    };
    for value in values {
        validate_snake_case_id(value).map_err(|_| {
            config_model::user_error(format!(
                "invalid {} entry '{}'; expected lowercase snake_case id",
                key,
                value.trim()
            ))
        })?;
    }
    Ok(())
}

fn validate_dotted_id_list_opt(values: &Option<Vec<String>>, key: &str) -> Result<()> {
    let Some(values) = values else {
        return Ok(());
    };
    for value in values {
        validate_dotted_id(value).map_err(|_| {
            config_model::user_error(format!(
                "invalid {} entry '{}'; expected lowercase dotted id",
                key,
                value.trim()
            ))
        })?;
    }
    Ok(())
}

fn validate_world_dispatch_action_list_opt(values: &Option<Vec<String>>, key: &str) -> Result<()> {
    let Some(values) = values else {
        return Ok(());
    };
    for value in values {
        validate_world_dispatch_action_id(value).map_err(|_| {
            config_model::user_error(format!(
                "invalid {} entry '{}'; expected one of run_world_task, spawn_world_worker, continue_world_worker, inspect_world_worker, cancel_world_work, stop_world_worker",
                key,
                value.trim()
            ))
        })?;
    }
    Ok(())
}

fn validate_world_dispatch_mode_list_opt(values: &Option<Vec<String>>, key: &str) -> Result<()> {
    let Some(values) = values else {
        return Ok(());
    };
    for value in values {
        validate_world_dispatch_mode_id(value).map_err(|_| {
            config_model::user_error(format!(
                "invalid {} entry '{}'; expected one of ephemeral, retained",
                key,
                value.trim()
            ))
        })?;
    }
    Ok(())
}

pub(crate) fn apply_updates_to_policy_patch(
    patch: &mut PolicyPatch,
    updates: &[ConfigUpdate],
) -> Result<bool> {
    let before = patch.clone();
    let mut changed = false;
    for update in updates {
        changed |= apply_update_to_patch(patch, update)?;
    }

    // Validate the patch by applying it on defaults and enforcing policy invariants.
    let mut effective = Policy::default();
    apply_policy_patch_over(&mut effective, patch);
    validate_policy(&effective)?;

    if !changed && before != *patch {
        changed = true;
    }

    Ok(changed)
}

pub(crate) fn reset_policy_patch_keys(patch: &mut PolicyPatch, keys: &[String]) -> Result<bool> {
    if keys.is_empty() {
        let was_empty = patch.is_empty();
        *patch = PolicyPatch::default();
        return Ok(!was_empty);
    }

    let mut changed = false;
    for key in keys {
        changed |= reset_policy_patch_key(patch, key)?;
    }
    Ok(changed)
}

#[allow(dead_code)]
pub(crate) fn resolve_effective_policy_with_explain(
    cwd: &Path,
    explain: bool,
) -> Result<(Policy, Option<PolicyExplainV1>)> {
    substrate_broker::resolve_effective_policy_with_explain(cwd, explain)
        .map_err(|err| config_model::user_error(err.to_string()))
    /*
    let (global_patch, _) = read_global_policy_patch_or_empty()?;
    let global_path = global_policy_path()?;

    let workspace_root = workspace::find_workspace_root(cwd);
    let workspace_layer = if let Some(root) = &workspace_root {
        let path = workspace_policy_path(root);
        match fs::read_to_string(&path) {
            Ok(raw) => Some((parse_policy_patch_yaml(&path, &raw)?, path)),
            Err(err) if err.kind() == io::ErrorKind::NotFound => None,
            Err(err) => return Err(anyhow!("failed to read {}: {err}", path.display())),
        }
    } else {
        None
    };

    let workspace_enabled = workspace_layer.is_some();
    let workspace_path = workspace_layer.as_ref().map(|(_, p)| p.as_path());

    #[derive(Clone, Copy)]
    enum ReplaceSource {
        WorkspacePatch,
        GlobalPatch,
        Default,
    }

    fn explain_source(
        source: ReplaceSource,
        global_path: &Path,
        workspace_path: Option<&Path>,
    ) -> PolicyExplainSource {
        match source {
            ReplaceSource::WorkspacePatch => PolicyExplainSource {
                layer: "workspace_patch".to_string(),
                path: workspace_path.map(|p| p.display().to_string()),
            },
            ReplaceSource::GlobalPatch => PolicyExplainSource {
                layer: "global_patch".to_string(),
                path: Some(global_path.display().to_string()),
            },
            ReplaceSource::Default => PolicyExplainSource {
                layer: "default".to_string(),
                path: None,
            },
        }
    }

    fn resolve_replace<T: Clone>(
        default: T,
        global: Option<T>,
        workspace: Option<T>,
        workspace_enabled: bool,
    ) -> (T, ReplaceSource) {
        if workspace_enabled {
            if let Some(v) = workspace {
                return (v, ReplaceSource::WorkspacePatch);
            }
        }
        if let Some(v) = global {
            return (v, ReplaceSource::GlobalPatch);
        }
        (default, ReplaceSource::Default)
    }

    fn resolve_replace_opt<T: Clone>(
        default: Option<T>,
        global: Option<T>,
        workspace: Option<T>,
        workspace_enabled: bool,
    ) -> (Option<T>, ReplaceSource) {
        if workspace_enabled && workspace.is_some() {
            return (workspace, ReplaceSource::WorkspacePatch);
        }
        if global.is_some() {
            return (global, ReplaceSource::GlobalPatch);
        }
        (default, ReplaceSource::Default)
    }

    let workspace_patch = workspace_layer.as_ref().map(|(p, _)| p);

    let mut effective = Policy::default();
    let mut explain_keys = if explain {
        Some(OrderedExplainKeys::default())
    } else {
        None
    };

    // id
    let (id, id_src) = resolve_replace(
        effective.id.clone(),
        global_patch.id.clone(),
        workspace_patch.and_then(|p| p.id.clone()),
        workspace_enabled,
    );
    effective.id = id;
    if let Some(keys) = &mut explain_keys {
        keys.insert(
            "id".to_string(),
            PolicyExplainKey {
                merge_strategy: "replace".to_string(),
                sources: vec![explain_source(id_src, &global_path, workspace_path)],
            },
        );
    }

    // name
    let (name, name_src) = resolve_replace(
        effective.name.clone(),
        global_patch.name.clone(),
        workspace_patch.and_then(|p| p.name.clone()),
        workspace_enabled,
    );
    effective.name = name;
    if let Some(keys) = &mut explain_keys {
        keys.insert(
            "name".to_string(),
            PolicyExplainKey {
                merge_strategy: "replace".to_string(),
                sources: vec![explain_source(name_src, &global_path, workspace_path)],
            },
        );
    }

    // world_fs.mode
    let (fs_mode, fs_mode_src) = resolve_replace(
        effective.world_fs_mode,
        global_patch.world_fs.mode,
        workspace_patch.and_then(|p| p.world_fs.mode),
        workspace_enabled,
    );
    effective.world_fs_mode = fs_mode;
    if let Some(keys) = &mut explain_keys {
        keys.insert(
            "world_fs.mode".to_string(),
            PolicyExplainKey {
                merge_strategy: "replace".to_string(),
                sources: vec![explain_source(fs_mode_src, &global_path, workspace_path)],
            },
        );
    }

    // world_fs.isolation
    let (fs_iso, fs_iso_src) = resolve_replace(
        effective.world_fs_isolation,
        global_patch.world_fs.isolation,
        workspace_patch.and_then(|p| p.world_fs.isolation),
        workspace_enabled,
    );
    effective.world_fs_isolation = fs_iso;
    if let Some(keys) = &mut explain_keys {
        keys.insert(
            "world_fs.isolation".to_string(),
            PolicyExplainKey {
                merge_strategy: "replace".to_string(),
                sources: vec![explain_source(fs_iso_src, &global_path, workspace_path)],
            },
        );
    }

    // world_fs.require_world
    let (require_world, require_world_src) = resolve_replace(
        effective.world_fs_require_world,
        global_patch.world_fs.require_world,
        workspace_patch.and_then(|p| p.world_fs.require_world),
        workspace_enabled,
    );
    effective.world_fs_require_world = require_world;
    if let Some(keys) = &mut explain_keys {
        keys.insert(
            "world_fs.require_world".to_string(),
            PolicyExplainKey {
                merge_strategy: "replace".to_string(),
                sources: vec![explain_source(
                    require_world_src,
                    &global_path,
                    workspace_path,
                )],
            },
        );
    }

    // world_fs.read_allowlist
    let (read_allow, read_allow_src) = resolve_replace(
        effective.fs_read.clone(),
        global_patch.world_fs.read_allowlist.clone(),
        workspace_patch.and_then(|p| p.world_fs.read_allowlist.clone()),
        workspace_enabled,
    );
    effective.fs_read = read_allow;
    if let Some(keys) = &mut explain_keys {
        keys.insert(
            "world_fs.read_allowlist".to_string(),
            PolicyExplainKey {
                merge_strategy: "replace".to_string(),
                sources: vec![explain_source(read_allow_src, &global_path, workspace_path)],
            },
        );
    }

    // world_fs.write_allowlist
    let (write_allow, write_allow_src) = resolve_replace(
        effective.fs_write.clone(),
        global_patch.world_fs.write_allowlist.clone(),
        workspace_patch.and_then(|p| p.world_fs.write_allowlist.clone()),
        workspace_enabled,
    );
    effective.fs_write = write_allow;
    if let Some(keys) = &mut explain_keys {
        keys.insert(
            "world_fs.write_allowlist".to_string(),
            PolicyExplainKey {
                merge_strategy: "replace".to_string(),
                sources: vec![explain_source(
                    write_allow_src,
                    &global_path,
                    workspace_path,
                )],
            },
        );
    }

    // net_allowed
    let (net_allowed, net_allowed_src) = resolve_replace(
        effective.net_allowed.clone(),
        global_patch.net_allowed.clone(),
        workspace_patch.and_then(|p| p.net_allowed.clone()),
        workspace_enabled,
    );
    effective.net_allowed = net_allowed;
    if let Some(keys) = &mut explain_keys {
        keys.insert(
            "net_allowed".to_string(),
            PolicyExplainKey {
                merge_strategy: "replace".to_string(),
                sources: vec![explain_source(
                    net_allowed_src,
                    &global_path,
                    workspace_path,
                )],
            },
        );
    }

    // cmd_allowed
    let (cmd_allowed, cmd_allowed_src) = resolve_replace(
        effective.cmd_allowed.clone(),
        global_patch.cmd_allowed.clone(),
        workspace_patch.and_then(|p| p.cmd_allowed.clone()),
        workspace_enabled,
    );
    effective.cmd_allowed = cmd_allowed;
    if let Some(keys) = &mut explain_keys {
        keys.insert(
            "cmd_allowed".to_string(),
            PolicyExplainKey {
                merge_strategy: "replace".to_string(),
                sources: vec![explain_source(
                    cmd_allowed_src,
                    &global_path,
                    workspace_path,
                )],
            },
        );
    }

    // cmd_denied
    let (cmd_denied, cmd_denied_src) = resolve_replace(
        effective.cmd_denied.clone(),
        global_patch.cmd_denied.clone(),
        workspace_patch.and_then(|p| p.cmd_denied.clone()),
        workspace_enabled,
    );
    effective.cmd_denied = cmd_denied;
    if let Some(keys) = &mut explain_keys {
        keys.insert(
            "cmd_denied".to_string(),
            PolicyExplainKey {
                merge_strategy: "replace".to_string(),
                sources: vec![explain_source(cmd_denied_src, &global_path, workspace_path)],
            },
        );
    }

    // cmd_isolated
    let (cmd_isolated, cmd_isolated_src) = resolve_replace(
        effective.cmd_isolated.clone(),
        global_patch.cmd_isolated.clone(),
        workspace_patch.and_then(|p| p.cmd_isolated.clone()),
        workspace_enabled,
    );
    effective.cmd_isolated = cmd_isolated;
    if let Some(keys) = &mut explain_keys {
        keys.insert(
            "cmd_isolated".to_string(),
            PolicyExplainKey {
                merge_strategy: "replace".to_string(),
                sources: vec![explain_source(
                    cmd_isolated_src,
                    &global_path,
                    workspace_path,
                )],
            },
        );
    }

    // require_approval
    let (require_approval, require_approval_src) = resolve_replace(
        effective.require_approval,
        global_patch.require_approval,
        workspace_patch.and_then(|p| p.require_approval),
        workspace_enabled,
    );
    effective.require_approval = require_approval;
    if let Some(keys) = &mut explain_keys {
        keys.insert(
            "require_approval".to_string(),
            PolicyExplainKey {
                merge_strategy: "replace".to_string(),
                sources: vec![explain_source(
                    require_approval_src,
                    &global_path,
                    workspace_path,
                )],
            },
        );
    }

    // allow_shell_operators
    let (allow_shell_operators, allow_shell_operators_src) = resolve_replace(
        effective.allow_shell_operators,
        global_patch.allow_shell_operators,
        workspace_patch.and_then(|p| p.allow_shell_operators),
        workspace_enabled,
    );
    effective.allow_shell_operators = allow_shell_operators;
    if let Some(keys) = &mut explain_keys {
        keys.insert(
            "allow_shell_operators".to_string(),
            PolicyExplainKey {
                merge_strategy: "replace".to_string(),
                sources: vec![explain_source(
                    allow_shell_operators_src,
                    &global_path,
                    workspace_path,
                )],
            },
        );
    }

    // limits.*
    let (max_memory_mb, max_memory_mb_src) = resolve_replace_opt(
        effective.limits.max_memory_mb,
        global_patch.limits.max_memory_mb,
        workspace_patch.and_then(|p| p.limits.max_memory_mb),
        workspace_enabled,
    );
    effective.limits.max_memory_mb = max_memory_mb;
    if let Some(keys) = &mut explain_keys {
        keys.insert(
            "limits.max_memory_mb".to_string(),
            PolicyExplainKey {
                merge_strategy: "replace".to_string(),
                sources: vec![explain_source(
                    max_memory_mb_src,
                    &global_path,
                    workspace_path,
                )],
            },
        );
    }

    let (max_cpu_percent, max_cpu_percent_src) = resolve_replace_opt(
        effective.limits.max_cpu_percent,
        global_patch.limits.max_cpu_percent,
        workspace_patch.and_then(|p| p.limits.max_cpu_percent),
        workspace_enabled,
    );
    effective.limits.max_cpu_percent = max_cpu_percent;
    if let Some(keys) = &mut explain_keys {
        keys.insert(
            "limits.max_cpu_percent".to_string(),
            PolicyExplainKey {
                merge_strategy: "replace".to_string(),
                sources: vec![explain_source(
                    max_cpu_percent_src,
                    &global_path,
                    workspace_path,
                )],
            },
        );
    }

    let (max_runtime_ms, max_runtime_ms_src) = resolve_replace_opt(
        effective.limits.max_runtime_ms,
        global_patch.limits.max_runtime_ms,
        workspace_patch.and_then(|p| p.limits.max_runtime_ms),
        workspace_enabled,
    );
    effective.limits.max_runtime_ms = max_runtime_ms;
    if let Some(keys) = &mut explain_keys {
        keys.insert(
            "limits.max_runtime_ms".to_string(),
            PolicyExplainKey {
                merge_strategy: "replace".to_string(),
                sources: vec![explain_source(
                    max_runtime_ms_src,
                    &global_path,
                    workspace_path,
                )],
            },
        );
    }

    let (max_egress_bytes, max_egress_bytes_src) = resolve_replace_opt(
        effective.limits.max_egress_bytes,
        global_patch.limits.max_egress_bytes,
        workspace_patch.and_then(|p| p.limits.max_egress_bytes),
        workspace_enabled,
    );
    effective.limits.max_egress_bytes = max_egress_bytes;
    if let Some(keys) = &mut explain_keys {
        keys.insert(
            "limits.max_egress_bytes".to_string(),
            PolicyExplainKey {
                merge_strategy: "replace".to_string(),
                sources: vec![explain_source(
                    max_egress_bytes_src,
                    &global_path,
                    workspace_path,
                )],
            },
        );
    }

    // metadata
    let default_meta = effective.metadata.clone();
    let global_meta = global_patch.metadata.as_ref().map(btree_to_hashmap);
    let workspace_meta = workspace_patch
        .and_then(|p| p.metadata.as_ref())
        .map(btree_to_hashmap);
    let (metadata, metadata_src) =
        resolve_replace(default_meta, global_meta, workspace_meta, workspace_enabled);
    effective.metadata = metadata;
    if let Some(keys) = &mut explain_keys {
        keys.insert(
            "metadata".to_string(),
            PolicyExplainKey {
                merge_strategy: "replace".to_string(),
                sources: vec![explain_source(metadata_src, &global_path, workspace_path)],
            },
        );
    }

    validate_policy(&effective)?;

    let explain = explain_keys.map(|keys| PolicyExplainV1 {
        kind: "substrate.policy.explain.v1".to_string(),
        keys,
    });

    Ok((effective, explain))
    */
}

fn btree_to_hashmap(map: &BTreeMap<String, String>) -> HashMap<String, String> {
    map.iter().map(|(k, v)| (k.clone(), v.clone())).collect()
}

fn validate_policy(policy: &Policy) -> Result<()> {
    if !policy.world_fs_write_enabled && !policy.world_fs_fail_closed_routing {
        return Err(config_model::user_error(
            "world_fs.write.enabled=false requires world_fs.fail_closed.routing=true",
        ));
    }

    if policy.world_fs_host_visible {
        if policy.world_fs_read.is_some() {
            return Err(config_model::user_error(
                "world_fs.read must be omitted when world_fs.host_visible=true",
            ));
        }
        if policy.world_fs_discover.is_some() {
            return Err(config_model::user_error(
                "world_fs.discover must be omitted when world_fs.host_visible=true",
            ));
        }
        if policy.world_fs_write.is_some() {
            return Err(config_model::user_error(
                "world_fs.write.allow_list and world_fs.write.deny_list must be omitted when world_fs.host_visible=true",
            ));
        }
    }

    let any_deny = policy
        .world_fs_read
        .as_ref()
        .is_some_and(|d| !d.deny_list.is_empty())
        || policy
            .world_fs_discover
            .as_ref()
            .is_some_and(|d| !d.deny_list.is_empty())
        || (policy.world_fs_write_enabled
            && policy
                .world_fs_write
                .as_ref()
                .is_some_and(|d| !d.deny_list.is_empty()));

    if any_deny && policy.world_fs_deny_enforcement.is_none() {
        return Err(config_model::user_error(
            "world_fs.deny_enforcement must be present when any deny_list is non-empty",
        ));
    }
    validate_backend_id_list(&policy.llm_allowed_backends, "llm.allowed_backends")?;
    validate_snake_case_id_list(&policy.llm_constraints_routers, "llm.constraints.routers")?;
    validate_snake_case_id_list(
        &policy.llm_constraints_providers,
        "llm.constraints.providers",
    )?;
    validate_dotted_id_list(
        &policy.llm_constraints_protocols,
        "llm.constraints.protocols",
    )?;
    validate_snake_case_id_list(
        &policy.llm_constraints_auth_authorities,
        "llm.constraints.auth_authorities",
    )?;
    validate_backend_id_list(&policy.agents_allowed_backends, "agents.allowed_backends")?;
    validate_backend_id_list(
        &policy.agents_host_credentials_read_allowed_backends,
        "agents.host_credentials.read.allowed_backends",
    )?;
    validate_backend_id_list(
        &policy.agents_world_dispatch_allowed_backends,
        "agents.world_dispatch.allowed_backends",
    )?;
    validate_world_dispatch_action_list(
        &policy.agents_world_dispatch_allowed_actions,
        "agents.world_dispatch.allowed_actions",
    )?;
    validate_world_dispatch_mode_list(
        &policy.agents_world_dispatch_allowed_modes,
        "agents.world_dispatch.allowed_modes",
    )?;
    Ok(())
}

fn validate_backend_id_list(values: &[String], key: &str) -> Result<()> {
    for value in values {
        validate_backend_id(value).map_err(|_| {
            config_model::user_error(format!(
                "invalid {} entry '{}'; expected <kind>:<name> with kind [a-z0-9_]+ and name [a-z0-9_-]+",
                key,
                value.trim()
            ))
        })?;
    }
    Ok(())
}

fn validate_snake_case_id_list(values: &[String], key: &str) -> Result<()> {
    for value in values {
        validate_snake_case_id(value).map_err(|_| {
            config_model::user_error(format!(
                "invalid {} entry '{}'; expected lowercase snake_case id",
                key,
                value.trim()
            ))
        })?;
    }
    Ok(())
}

fn validate_dotted_id_list(values: &[String], key: &str) -> Result<()> {
    for value in values {
        validate_dotted_id(value).map_err(|_| {
            config_model::user_error(format!(
                "invalid {} entry '{}'; expected lowercase dotted id",
                key,
                value.trim()
            ))
        })?;
    }
    Ok(())
}

fn validate_world_dispatch_action_list(values: &[String], key: &str) -> Result<()> {
    for value in values {
        validate_world_dispatch_action_id(value).map_err(|_| {
            config_model::user_error(format!(
                "invalid {} entry '{}'; expected one of run_world_task, spawn_world_worker, continue_world_worker, inspect_world_worker, cancel_world_work, stop_world_worker",
                key,
                value.trim()
            ))
        })?;
    }
    Ok(())
}

fn validate_world_dispatch_mode_list(values: &[String], key: &str) -> Result<()> {
    for value in values {
        validate_world_dispatch_mode_id(value).map_err(|_| {
            config_model::user_error(format!(
                "invalid {} entry '{}'; expected one of ephemeral, retained",
                key,
                value.trim()
            ))
        })?;
    }
    Ok(())
}

pub(crate) fn apply_policy_patch(base: &Policy, patch: &PolicyPatch) -> Policy {
    let mut effective = base.clone();
    apply_policy_patch_over(&mut effective, patch);
    effective
}

fn apply_policy_patch_over(target: &mut Policy, patch: &PolicyPatch) {
    if let Some(v) = &patch.id {
        target.id = v.clone();
    }
    if let Some(v) = &patch.name {
        target.name = v.clone();
    }
    if let Some(v) = patch.world_fs.host_visible {
        target.world_fs_host_visible = v;
    }
    if let Some(v) = patch.world_fs.fail_closed.routing {
        target.world_fs_fail_closed_routing = v;
    }
    if let Some(v) = patch.world_fs.deny_enforcement {
        target.world_fs_deny_enforcement = Some(v);
    }
    if let Some(v) = patch.world_fs.caged_required {
        target.world_fs_caged_required = v;
    }
    if let Some(v) = patch.world_fs.write.enabled {
        target.world_fs_write_enabled = v;
    }

    if !patch.world_fs.discover.is_empty() {
        let mut dim = target
            .world_fs_discover
            .take()
            .unwrap_or(WorldFsDimensionPolicy {
                allow_list: Vec::new(),
                deny_list: Vec::new(),
            });
        if let Some(v) = &patch.world_fs.discover.allow_list {
            dim.allow_list = v.clone();
        }
        if let Some(v) = &patch.world_fs.discover.deny_list {
            dim.deny_list = v.clone();
        }
        target.world_fs_discover = Some(dim);
    }

    if !patch.world_fs.read.is_empty() {
        let mut dim = target
            .world_fs_read
            .take()
            .unwrap_or(WorldFsDimensionPolicy {
                allow_list: Vec::new(),
                deny_list: Vec::new(),
            });
        if let Some(v) = &patch.world_fs.read.allow_list {
            dim.allow_list = v.clone();
        }
        if let Some(v) = &patch.world_fs.read.deny_list {
            dim.deny_list = v.clone();
        }
        target.world_fs_read = Some(dim);
    }

    if !patch.world_fs.write.is_empty() {
        let mut dim = target
            .world_fs_write
            .take()
            .unwrap_or(WorldFsDimensionPolicy {
                allow_list: Vec::new(),
                deny_list: Vec::new(),
            });
        if let Some(v) = &patch.world_fs.write.allow_list {
            dim.allow_list = v.clone();
        }
        if let Some(v) = &patch.world_fs.write.deny_list {
            dim.deny_list = v.clone();
        }
        target.world_fs_write = Some(dim);
    }
    if let Some(v) = patch.llm.fail_closed.routing {
        target.llm_fail_closed_routing = v;
    }
    if let Some(v) = patch.llm.require_approval {
        target.llm_require_approval = v;
    }
    if let Some(v) = &patch.llm.allowed_backends {
        target.llm_allowed_backends = v.clone();
    }
    if let Some(v) = &patch.llm.constraints.routers {
        target.llm_constraints_routers = v.clone();
    }
    if let Some(v) = &patch.llm.constraints.providers {
        target.llm_constraints_providers = v.clone();
    }
    if let Some(v) = &patch.llm.constraints.protocols {
        target.llm_constraints_protocols = v.clone();
    }
    if let Some(v) = &patch.llm.constraints.auth_authorities {
        target.llm_constraints_auth_authorities = v.clone();
    }
    if let Some(v) = &patch.llm.secrets.env_allowed {
        target.llm_secrets_env_allowed = v.clone();
    }
    if let Some(v) = &patch.agents.allowed_backends {
        target.agents_allowed_backends = v.clone();
    }
    if let Some(v) = patch.agents.fail_closed.routing {
        target.agents_fail_closed_routing = v;
    }
    if let Some(v) = &patch.agents.host_credentials.read.allowed_backends {
        target.agents_host_credentials_read_allowed_backends = v.clone();
    }
    if let Some(v) = patch.agents.world_dispatch.enabled {
        target.agents_world_dispatch_enabled = v;
    }
    if let Some(v) = &patch.agents.world_dispatch.allowed_backends {
        target.agents_world_dispatch_allowed_backends = v.clone();
    }
    if let Some(v) = &patch.agents.world_dispatch.allowed_actions {
        target.agents_world_dispatch_allowed_actions = v.clone();
    }
    if let Some(v) = &patch.agents.world_dispatch.allowed_modes {
        target.agents_world_dispatch_allowed_modes = v.clone();
    }
    if let Some(v) = patch.agents.world_dispatch.same_session_only {
        target.agents_world_dispatch_same_session_only = v;
    }
    if let Some(v) = patch.agents.world_dispatch.same_world_binding_only {
        target.agents_world_dispatch_same_world_binding_only = v;
    }
    if let Some(v) = patch.agents.world_dispatch.allow_capability_narrowing {
        target.agents_world_dispatch_allow_capability_narrowing = v;
    }
    if let Some(v) = patch.agents.world_dispatch.max_live_retained_workers {
        target.agents_world_dispatch_max_live_retained_workers = v;
    }
    if let Some(v) = patch.agents.world_dispatch.max_concurrent_ephemeral {
        target.agents_world_dispatch_max_concurrent_ephemeral = v;
    }
    if let Some(v) = patch.workflow.router.enabled {
        target.workflow_router_enabled = v;
    }
    if let Some(v) = patch.workflow.router.allow_cross_workspace {
        target.workflow_router_allow_cross_workspace = v;
    }
    if let Some(v) = &patch.workflow.router.allowed_rule_ids {
        target.workflow_router_allowed_rule_ids = v.clone();
    }
    if let Some(v) = &patch.workflow.router.allowed_workflow_ids {
        target.workflow_router_allowed_workflow_ids = v.clone();
    }
    if let Some(v) = &patch.workflow.router.allowed_target_workspace_ids {
        target.workflow_router_allowed_target_workspace_ids = v.clone();
    }
    if let Some(v) = &patch.net_allowed {
        target.net_allowed = v.clone();
    }
    if let Some(v) = &patch.cmd_allowed {
        target.cmd_allowed = v.clone();
    }
    if let Some(v) = &patch.cmd_denied {
        target.cmd_denied = v.clone();
    }
    if let Some(v) = &patch.cmd_isolated {
        target.cmd_isolated = v.clone();
    }
    if let Some(v) = patch.require_approval {
        target.require_approval = v;
    }
    if let Some(v) = patch.allow_shell_operators {
        target.allow_shell_operators = v;
    }
    if let Some(v) = patch.limits.max_memory_mb {
        target.limits.max_memory_mb = Some(v);
    }
    if let Some(v) = patch.limits.max_cpu_percent {
        target.limits.max_cpu_percent = Some(v);
    }
    if let Some(v) = patch.limits.max_runtime_ms {
        target.limits.max_runtime_ms = Some(v);
    }
    if let Some(v) = patch.limits.max_egress_bytes {
        target.limits.max_egress_bytes = Some(v);
    }
    if let Some(v) = &patch.metadata {
        target.metadata = btree_to_hashmap(v);
    }
}

fn reset_policy_patch_key(patch: &mut PolicyPatch, key: &str) -> Result<bool> {
    match key {
        "id" => Ok(patch.id.take().is_some()),
        "name" => Ok(patch.name.take().is_some()),

        "world_fs.host_visible" => Ok(patch.world_fs.host_visible.take().is_some()),
        "world_fs.fail_closed.routing" => Ok(patch.world_fs.fail_closed.routing.take().is_some()),
        "world_fs.deny_enforcement" => Ok(patch.world_fs.deny_enforcement.take().is_some()),
        "world_fs.caged_required" => Ok(patch.world_fs.caged_required.take().is_some()),

        "world_fs.discover.allow_list" => Ok(patch.world_fs.discover.allow_list.take().is_some()),
        "world_fs.discover.deny_list" => Ok(patch.world_fs.discover.deny_list.take().is_some()),
        "world_fs.discover" => {
            let was = !patch.world_fs.discover.is_empty();
            patch.world_fs.discover = WorldFsDimensionPatch::default();
            Ok(was)
        }
        "world_fs.read.allow_list" => Ok(patch.world_fs.read.allow_list.take().is_some()),
        "world_fs.read.deny_list" => Ok(patch.world_fs.read.deny_list.take().is_some()),
        "world_fs.read" => {
            let was = !patch.world_fs.read.is_empty();
            patch.world_fs.read = WorldFsDimensionPatch::default();
            Ok(was)
        }
        "world_fs.write.enabled" => Ok(patch.world_fs.write.enabled.take().is_some()),
        "world_fs.write.allow_list" => Ok(patch.world_fs.write.allow_list.take().is_some()),
        "world_fs.write.deny_list" => Ok(patch.world_fs.write.deny_list.take().is_some()),
        "world_fs.write" => {
            let was = !patch.world_fs.write.is_empty();
            patch.world_fs.write = WorldFsWritePatch::default();
            Ok(was)
        }

        "llm.fail_closed.routing" => Ok(patch.llm.fail_closed.routing.take().is_some()),
        "llm.require_approval" => Ok(patch.llm.require_approval.take().is_some()),
        "llm.allowed_backends" => Ok(patch.llm.allowed_backends.take().is_some()),
        "llm.constraints.routers" => Ok(patch.llm.constraints.routers.take().is_some()),
        "llm.constraints.providers" => Ok(patch.llm.constraints.providers.take().is_some()),
        "llm.constraints.protocols" => Ok(patch.llm.constraints.protocols.take().is_some()),
        "llm.constraints.auth_authorities" => {
            Ok(patch.llm.constraints.auth_authorities.take().is_some())
        }
        "llm.secrets.env_allowed" => Ok(patch.llm.secrets.env_allowed.take().is_some()),

        "agents.allowed_backends" => Ok(patch.agents.allowed_backends.take().is_some()),
        "agents.fail_closed.routing" => Ok(patch.agents.fail_closed.routing.take().is_some()),
        "agents.host_credentials.read.allowed_backends" => Ok(patch
            .agents
            .host_credentials
            .read
            .allowed_backends
            .take()
            .is_some()),
        "agents.world_dispatch.enabled" => Ok(patch.agents.world_dispatch.enabled.take().is_some()),
        "agents.world_dispatch.allowed_backends" => Ok(patch
            .agents
            .world_dispatch
            .allowed_backends
            .take()
            .is_some()),
        "agents.world_dispatch.allowed_actions" => {
            Ok(patch.agents.world_dispatch.allowed_actions.take().is_some())
        }
        "agents.world_dispatch.allowed_modes" => {
            Ok(patch.agents.world_dispatch.allowed_modes.take().is_some())
        }
        "agents.world_dispatch.same_session_only" => Ok(patch
            .agents
            .world_dispatch
            .same_session_only
            .take()
            .is_some()),
        "agents.world_dispatch.same_world_binding_only" => Ok(patch
            .agents
            .world_dispatch
            .same_world_binding_only
            .take()
            .is_some()),
        "agents.world_dispatch.allow_capability_narrowing" => Ok(patch
            .agents
            .world_dispatch
            .allow_capability_narrowing
            .take()
            .is_some()),
        "agents.world_dispatch.max_live_retained_workers" => Ok(patch
            .agents
            .world_dispatch
            .max_live_retained_workers
            .take()
            .is_some()),
        "agents.world_dispatch.max_concurrent_ephemeral" => Ok(patch
            .agents
            .world_dispatch
            .max_concurrent_ephemeral
            .take()
            .is_some()),

        "workflow.router.enabled" => Ok(patch.workflow.router.enabled.take().is_some()),
        "workflow.router.allow_cross_workspace" => {
            Ok(patch.workflow.router.allow_cross_workspace.take().is_some())
        }
        "workflow.router.allowed_rule_ids" => {
            Ok(patch.workflow.router.allowed_rule_ids.take().is_some())
        }
        "workflow.router.allowed_workflow_ids" => {
            Ok(patch.workflow.router.allowed_workflow_ids.take().is_some())
        }
        "workflow.router.allowed_target_workspace_ids" => Ok(patch
            .workflow
            .router
            .allowed_target_workspace_ids
            .take()
            .is_some()),

        "net_allowed" => Ok(patch.net_allowed.take().is_some()),
        "cmd_allowed" => Ok(patch.cmd_allowed.take().is_some()),
        "cmd_denied" => Ok(patch.cmd_denied.take().is_some()),
        "cmd_isolated" => Ok(patch.cmd_isolated.take().is_some()),

        "require_approval" => Ok(patch.require_approval.take().is_some()),
        "allow_shell_operators" => Ok(patch.allow_shell_operators.take().is_some()),

        "limits.max_memory_mb" => Ok(patch.limits.max_memory_mb.take().is_some()),
        "limits.max_cpu_percent" => Ok(patch.limits.max_cpu_percent.take().is_some()),
        "limits.max_runtime_ms" => Ok(patch.limits.max_runtime_ms.take().is_some()),
        "limits.max_egress_bytes" => Ok(patch.limits.max_egress_bytes.take().is_some()),

        "metadata" => Ok(patch.metadata.take().is_some()),

        _ => Err(config_model::user_error(format!(
            "unknown policy key '{}'",
            key
        ))),
    }
}

fn apply_update_to_patch(patch: &mut PolicyPatch, update: &ConfigUpdate) -> Result<bool> {
    match update.key.as_str() {
        "id" => apply_string_opt(&mut patch.id, &update.op, &update.value),
        "name" => apply_string_opt(&mut patch.name, &update.op, &update.value),

        "world_fs.host_visible" => {
            apply_bool_opt(&mut patch.world_fs.host_visible, &update.op, &update.value)
        }
        "world_fs.fail_closed.routing" => apply_bool_opt(
            &mut patch.world_fs.fail_closed.routing,
            &update.op,
            &update.value,
        ),
        "world_fs.deny_enforcement" => {
            apply_enum_world_fs_deny_enforcement_opt(&mut patch.world_fs.deny_enforcement, update)
        }
        "world_fs.caged_required" => apply_bool_opt(
            &mut patch.world_fs.caged_required,
            &update.op,
            &update.value,
        ),
        "world_fs.discover.allow_list" => {
            apply_string_list_opt(&mut patch.world_fs.discover.allow_list, update)
        }
        "world_fs.discover.deny_list" => {
            apply_string_list_opt(&mut patch.world_fs.discover.deny_list, update)
        }
        "world_fs.read.allow_list" => {
            apply_string_list_opt(&mut patch.world_fs.read.allow_list, update)
        }
        "world_fs.read.deny_list" => {
            apply_string_list_opt(&mut patch.world_fs.read.deny_list, update)
        }
        "world_fs.write.enabled" => {
            apply_bool_opt(&mut patch.world_fs.write.enabled, &update.op, &update.value)
        }
        "world_fs.write.allow_list" => {
            apply_string_list_opt(&mut patch.world_fs.write.allow_list, update)
        }
        "world_fs.write.deny_list" => {
            apply_string_list_opt(&mut patch.world_fs.write.deny_list, update)
        }

        "llm.fail_closed.routing" => apply_bool_opt(
            &mut patch.llm.fail_closed.routing,
            &update.op,
            &update.value,
        ),
        "llm.require_approval" => {
            apply_bool_opt(&mut patch.llm.require_approval, &update.op, &update.value)
        }
        "llm.allowed_backends" => {
            apply_backend_id_list_opt(&mut patch.llm.allowed_backends, update)
        }
        "llm.constraints.routers" => {
            apply_string_list_opt(&mut patch.llm.constraints.routers, update)
        }
        "llm.constraints.providers" => {
            apply_string_list_opt(&mut patch.llm.constraints.providers, update)
        }
        "llm.constraints.protocols" => {
            apply_string_list_opt(&mut patch.llm.constraints.protocols, update)
        }
        "llm.constraints.auth_authorities" => {
            apply_string_list_opt(&mut patch.llm.constraints.auth_authorities, update)
        }
        "llm.secrets.env_allowed" => {
            apply_string_list_opt(&mut patch.llm.secrets.env_allowed, update)
        }

        "agents.allowed_backends" => {
            apply_backend_id_list_opt(&mut patch.agents.allowed_backends, update)
        }
        "agents.fail_closed.routing" => apply_bool_opt(
            &mut patch.agents.fail_closed.routing,
            &update.op,
            &update.value,
        ),
        "agents.host_credentials.read.allowed_backends" => apply_backend_id_list_opt(
            &mut patch.agents.host_credentials.read.allowed_backends,
            update,
        ),
        "agents.world_dispatch.enabled" => apply_bool_opt(
            &mut patch.agents.world_dispatch.enabled,
            &update.op,
            &update.value,
        ),
        "agents.world_dispatch.allowed_backends" => {
            apply_backend_id_list_opt(&mut patch.agents.world_dispatch.allowed_backends, update)
        }
        "agents.world_dispatch.allowed_actions" => {
            apply_string_list_opt(&mut patch.agents.world_dispatch.allowed_actions, update)
        }
        "agents.world_dispatch.allowed_modes" => {
            apply_string_list_opt(&mut patch.agents.world_dispatch.allowed_modes, update)
        }
        "agents.world_dispatch.same_session_only" => apply_bool_opt(
            &mut patch.agents.world_dispatch.same_session_only,
            &update.op,
            &update.value,
        ),
        "agents.world_dispatch.same_world_binding_only" => apply_bool_opt(
            &mut patch.agents.world_dispatch.same_world_binding_only,
            &update.op,
            &update.value,
        ),
        "agents.world_dispatch.allow_capability_narrowing" => apply_bool_opt(
            &mut patch.agents.world_dispatch.allow_capability_narrowing,
            &update.op,
            &update.value,
        ),
        "agents.world_dispatch.max_live_retained_workers" => apply_u32_opt(
            &mut patch.agents.world_dispatch.max_live_retained_workers,
            &update.op,
            &update.value,
        ),
        "agents.world_dispatch.max_concurrent_ephemeral" => apply_u32_opt(
            &mut patch.agents.world_dispatch.max_concurrent_ephemeral,
            &update.op,
            &update.value,
        ),

        "workflow.router.enabled" => apply_bool_opt(
            &mut patch.workflow.router.enabled,
            &update.op,
            &update.value,
        ),
        "workflow.router.allow_cross_workspace" => apply_bool_opt(
            &mut patch.workflow.router.allow_cross_workspace,
            &update.op,
            &update.value,
        ),
        "workflow.router.allowed_rule_ids" => {
            apply_string_list_opt(&mut patch.workflow.router.allowed_rule_ids, update)
        }
        "workflow.router.allowed_workflow_ids" => {
            apply_string_list_opt(&mut patch.workflow.router.allowed_workflow_ids, update)
        }
        "workflow.router.allowed_target_workspace_ids" => apply_string_list_opt(
            &mut patch.workflow.router.allowed_target_workspace_ids,
            update,
        ),

        "net_allowed" => apply_string_list_opt(&mut patch.net_allowed, update),
        "cmd_allowed" => apply_string_list_opt(&mut patch.cmd_allowed, update),
        "cmd_denied" => apply_string_list_opt(&mut patch.cmd_denied, update),
        "cmd_isolated" => apply_string_list_opt(&mut patch.cmd_isolated, update),

        "require_approval" => {
            apply_bool_opt(&mut patch.require_approval, &update.op, &update.value)
        }
        "allow_shell_operators" => {
            apply_bool_opt(&mut patch.allow_shell_operators, &update.op, &update.value)
        }

        "limits.max_memory_mb" => {
            apply_u64_opt(&mut patch.limits.max_memory_mb, &update.op, &update.value)
        }
        "limits.max_cpu_percent" => {
            apply_u32_opt(&mut patch.limits.max_cpu_percent, &update.op, &update.value)
        }
        "limits.max_runtime_ms" => {
            apply_u64_opt(&mut patch.limits.max_runtime_ms, &update.op, &update.value)
        }
        "limits.max_egress_bytes" => apply_u64_opt(
            &mut patch.limits.max_egress_bytes,
            &update.op,
            &update.value,
        ),

        "metadata" => apply_metadata_opt(&mut patch.metadata, &update.op, &update.value),

        _ => Err(config_model::user_error(format!(
            "unknown policy key '{}'",
            update.key
        ))),
    }
}

fn apply_string_opt(target: &mut Option<String>, op: &UpdateOp, raw: &str) -> Result<bool> {
    let UpdateOp::Set = op else {
        return Err(config_model::user_error(
            "operator += and -= are only valid for list keys",
        ));
    };
    let next = raw.trim().to_string();
    let next = Some(next);
    if *target == next {
        return Ok(false);
    }
    *target = next;
    Ok(true)
}

fn apply_bool_opt(target: &mut Option<bool>, op: &UpdateOp, raw: &str) -> Result<bool> {
    let UpdateOp::Set = op else {
        return Err(config_model::user_error(
            "operator += and -= are only valid for list keys",
        ));
    };
    let next = parse_bool_flag(raw).ok_or_else(|| {
        config_model::user_error(format!("invalid boolean value '{}'", raw.trim()))
    })?;
    let next = Some(next);
    if *target == next {
        return Ok(false);
    }
    *target = next;
    Ok(true)
}

fn apply_u64_opt(target: &mut Option<u64>, op: &UpdateOp, raw: &str) -> Result<bool> {
    let UpdateOp::Set = op else {
        return Err(config_model::user_error(
            "operator += and -= are only valid for list keys",
        ));
    };
    let trimmed = raw.trim();
    let next = trimmed.parse::<u64>().map_err(|_| {
        config_model::user_error(format!(
            "invalid integer value '{}' (expected base-10)",
            trimmed
        ))
    })?;
    let next = Some(next);
    if *target == next {
        return Ok(false);
    }
    *target = next;
    Ok(true)
}

fn apply_u32_opt(target: &mut Option<u32>, op: &UpdateOp, raw: &str) -> Result<bool> {
    let UpdateOp::Set = op else {
        return Err(config_model::user_error(
            "operator += and -= are only valid for list keys",
        ));
    };
    let trimmed = raw.trim();
    let next = trimmed.parse::<u32>().map_err(|_| {
        config_model::user_error(format!(
            "invalid integer value '{}' (expected base-10)",
            trimmed
        ))
    })?;
    let next = Some(next);
    if *target == next {
        return Ok(false);
    }
    *target = next;
    Ok(true)
}

fn apply_enum_world_fs_deny_enforcement_opt(
    target: &mut Option<WorldFsDenyEnforcement>,
    update: &ConfigUpdate,
) -> Result<bool> {
    let UpdateOp::Set = update.op else {
        return Err(config_model::user_error(
            "operator += and -= are only valid for list keys",
        ));
    };
    let next = match update.value.trim().to_ascii_lowercase().as_str() {
        "strict" => WorldFsDenyEnforcement::Strict,
        "prefer_strict" => WorldFsDenyEnforcement::PreferStrict,
        "weak" => WorldFsDenyEnforcement::Weak,
        _ => {
            return Err(config_model::user_error(format!(
                "invalid world_fs.deny_enforcement '{}' (expected strict, prefer_strict, or weak)",
                update.value.trim()
            )));
        }
    };
    let next = Some(next);
    if *target == next {
        return Ok(false);
    }
    *target = next;
    Ok(true)
}

fn apply_string_list_opt(target: &mut Option<Vec<String>>, update: &ConfigUpdate) -> Result<bool> {
    match update.op {
        UpdateOp::Set => {
            let parsed = parse_yaml_string_list(&update.value)?;
            let next = Some(parsed);
            if *target == next {
                return Ok(false);
            }
            *target = next;
            Ok(true)
        }
        UpdateOp::Append => {
            if target.is_none() {
                *target = Some(Vec::new());
            }
            let Some(list) = target.as_mut() else {
                return Ok(false);
            };
            if list.iter().any(|item| item == &update.value) {
                return Ok(false);
            }
            list.push(update.value.clone());
            Ok(true)
        }
        UpdateOp::Remove => {
            let Some(list) = target.as_mut() else {
                return Ok(false);
            };
            let before = list.len();
            list.retain(|item| item != &update.value);
            Ok(before != list.len())
        }
    }
}

fn apply_backend_id_list_opt(
    target: &mut Option<Vec<String>>,
    update: &ConfigUpdate,
) -> Result<bool> {
    match update.op {
        UpdateOp::Set => {
            let parsed = parse_yaml_string_list(&update.value)?;
            validate_backend_id_list(&parsed, update.key.as_str())?;
            let next = Some(parsed);
            if *target == next {
                return Ok(false);
            }
            *target = next;
            Ok(true)
        }
        UpdateOp::Append => {
            validate_backend_id_list(std::slice::from_ref(&update.value), update.key.as_str())?;
            let list = target.get_or_insert_with(Vec::new);
            if list.iter().any(|item| item == &update.value) {
                return Ok(false);
            }
            list.push(update.value.clone());
            Ok(true)
        }
        UpdateOp::Remove => {
            let Some(list) = target.as_mut() else {
                return Ok(false);
            };
            let before = list.len();
            list.retain(|item| item != &update.value);
            Ok(before != list.len())
        }
    }
}

fn parse_yaml_string_list(raw: &str) -> Result<Vec<String>> {
    let parsed: Vec<String> = serde_yaml::from_str(raw).map_err(|err| {
        config_model::user_error(format!(
            "invalid YAML list literal '{}': {}",
            raw.trim(),
            err.to_string().trim()
        ))
    })?;
    Ok(parsed)
}

fn apply_metadata_opt(
    target: &mut Option<BTreeMap<String, String>>,
    op: &UpdateOp,
    raw: &str,
) -> Result<bool> {
    match op {
        UpdateOp::Set => {
            let parsed: BTreeMap<String, String> = serde_yaml::from_str(raw).map_err(|err| {
                config_model::user_error(format!(
                    "invalid YAML mapping literal for metadata '{}': {}",
                    raw.trim(),
                    err.to_string().trim()
                ))
            })?;
            let next = Some(parsed);
            if *target == next {
                return Ok(false);
            }
            *target = next;
            Ok(true)
        }
        UpdateOp::Append | UpdateOp::Remove => Err(config_model::user_error(
            "metadata+= and metadata-= are not allowed (use metadata=... to replace the full mapping)",
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::execution::config_model::{ConfigUpdate, UpdateOp};

    #[test]
    fn policy_patch_accepts_world_fs_v3_keys_and_dimension_lists() {
        let mut patch = PolicyPatch::default();
        let updates = vec![
            ConfigUpdate {
                key: "world_fs.host_visible".to_string(),
                op: UpdateOp::Set,
                value: "false".to_string(),
            },
            ConfigUpdate {
                key: "world_fs.deny_enforcement".to_string(),
                op: UpdateOp::Set,
                value: "weak".to_string(),
            },
            ConfigUpdate {
                key: "world_fs.read.allow_list".to_string(),
                op: UpdateOp::Append,
                value: ".".to_string(),
            },
            ConfigUpdate {
                key: "world_fs.read.deny_list".to_string(),
                op: UpdateOp::Append,
                value: "./secrets/**".to_string(),
            },
            ConfigUpdate {
                key: "world_fs.write.allow_list".to_string(),
                op: UpdateOp::Append,
                value: ".".to_string(),
            },
            ConfigUpdate {
                key: "world_fs.write.deny_list".to_string(),
                op: UpdateOp::Append,
                value: "./outputs/private/**".to_string(),
            },
        ];

        let changed = apply_updates_to_policy_patch(&mut patch, &updates).unwrap();
        assert!(changed);

        assert_eq!(patch.world_fs.host_visible, Some(false));
        assert_eq!(
            patch.world_fs.deny_enforcement,
            Some(WorldFsDenyEnforcement::Weak)
        );
        assert_eq!(
            patch.world_fs.read.allow_list.as_deref(),
            Some(&[".".to_string()][..])
        );
        assert_eq!(
            patch.world_fs.read.deny_list.as_deref(),
            Some(&["./secrets/**".to_string()][..])
        );
        assert_eq!(
            patch.world_fs.write.allow_list.as_deref(),
            Some(&[".".to_string()][..])
        );
        assert_eq!(
            patch.world_fs.write.deny_list.as_deref(),
            Some(&["./outputs/private/**".to_string()][..])
        );
    }

    #[test]
    fn policy_patch_rejects_legacy_world_fs_allowlist_keys() {
        let mut patch = PolicyPatch::default();
        let updates = vec![ConfigUpdate {
            key: "world_fs.read_allowlist".to_string(),
            op: UpdateOp::Append,
            value: ".".to_string(),
        }];
        let err = apply_updates_to_policy_patch(&mut patch, &updates).unwrap_err();
        assert!(err
            .to_string()
            .contains("unknown policy key 'world_fs.read_allowlist'"));
    }

    #[test]
    fn policy_patch_accepts_agents_world_dispatch_keys() {
        let mut patch = PolicyPatch::default();
        let updates = vec![
            ConfigUpdate {
                key: "agents.world_dispatch.enabled".to_string(),
                op: UpdateOp::Set,
                value: "true".to_string(),
            },
            ConfigUpdate {
                key: "agents.world_dispatch.allowed_backends".to_string(),
                op: UpdateOp::Append,
                value: "cli:codex_world".to_string(),
            },
            ConfigUpdate {
                key: "agents.world_dispatch.allowed_actions".to_string(),
                op: UpdateOp::Append,
                value: "stop_world_worker".to_string(),
            },
            ConfigUpdate {
                key: "agents.world_dispatch.allowed_modes".to_string(),
                op: UpdateOp::Append,
                value: "ephemeral".to_string(),
            },
            ConfigUpdate {
                key: "agents.world_dispatch.same_session_only".to_string(),
                op: UpdateOp::Set,
                value: "true".to_string(),
            },
            ConfigUpdate {
                key: "agents.world_dispatch.same_world_binding_only".to_string(),
                op: UpdateOp::Set,
                value: "true".to_string(),
            },
            ConfigUpdate {
                key: "agents.world_dispatch.allow_capability_narrowing".to_string(),
                op: UpdateOp::Set,
                value: "false".to_string(),
            },
            ConfigUpdate {
                key: "agents.world_dispatch.max_live_retained_workers".to_string(),
                op: UpdateOp::Set,
                value: "2".to_string(),
            },
            ConfigUpdate {
                key: "agents.world_dispatch.max_concurrent_ephemeral".to_string(),
                op: UpdateOp::Set,
                value: "1".to_string(),
            },
        ];

        let changed = apply_updates_to_policy_patch(&mut patch, &updates).unwrap();
        assert!(changed);
        assert_eq!(patch.agents.world_dispatch.enabled, Some(true));
        assert_eq!(
            patch.agents.world_dispatch.allowed_backends.as_deref(),
            Some(&["cli:codex_world".to_string()][..])
        );
        assert_eq!(
            patch.agents.world_dispatch.allowed_actions.as_deref(),
            Some(&["stop_world_worker".to_string()][..])
        );
        assert_eq!(
            patch.agents.world_dispatch.allowed_modes.as_deref(),
            Some(&["ephemeral".to_string()][..])
        );
        assert_eq!(patch.agents.world_dispatch.same_session_only, Some(true));
        assert_eq!(
            patch.agents.world_dispatch.same_world_binding_only,
            Some(true)
        );
        assert_eq!(
            patch.agents.world_dispatch.allow_capability_narrowing,
            Some(false)
        );
        assert_eq!(
            patch.agents.world_dispatch.max_live_retained_workers,
            Some(2)
        );
        assert_eq!(
            patch.agents.world_dispatch.max_concurrent_ephemeral,
            Some(1)
        );
    }

    #[test]
    fn policy_patch_parses_agents_world_dispatch_yaml_with_deny_by_default_shape() {
        let path = Path::new("policy.yaml");
        let patch = parse_policy_patch_yaml(
            path,
            r#"
agents:
  world_dispatch:
    enabled: true
    allowed_backends:
      - "cli:codex_world"
    allowed_actions:
      - "run_world_task"
      - "spawn_world_worker"
      - "continue_world_worker"
      - "inspect_world_worker"
      - "cancel_world_work"
      - "stop_world_worker"
    allowed_modes:
      - "ephemeral"
      - "retained"
    same_session_only: true
    same_world_binding_only: true
    allow_capability_narrowing: false
    max_live_retained_workers: 2
    max_concurrent_ephemeral: 1
"#,
        )
        .expect("world dispatch keys should parse under agents");

        assert_eq!(patch.agents.world_dispatch.enabled, Some(true));
        assert_eq!(
            patch.agents.world_dispatch.allowed_actions.as_deref(),
            Some(
                &[
                    "run_world_task".to_string(),
                    "spawn_world_worker".to_string(),
                    "continue_world_worker".to_string(),
                    "inspect_world_worker".to_string(),
                    "cancel_world_work".to_string(),
                    "stop_world_worker".to_string()
                ][..]
            )
        );
        assert_eq!(
            patch.agents.world_dispatch.allowed_modes.as_deref(),
            Some(&["ephemeral".to_string(), "retained".to_string()][..])
        );
    }

    #[test]
    fn policy_patch_accepts_stop_agents_world_dispatch_action() {
        let path = Path::new("policy.yaml");
        let patch = parse_policy_patch_yaml(
            path,
            r#"
agents:
  world_dispatch:
    allowed_actions:
      - "stop_world_worker"
"#,
        )
        .expect("stop_world_worker should be allowlistable in packet 1");

        assert_eq!(
            patch.agents.world_dispatch.allowed_actions.as_deref(),
            Some(&["stop_world_worker".to_string()][..])
        );
    }

    #[test]
    fn policy_patch_accepts_cancel_agents_world_dispatch_action() {
        let path = Path::new("policy.yaml");
        let patch = parse_policy_patch_yaml(
            path,
            r#"
agents:
  world_dispatch:
    allowed_actions:
      - "cancel_world_work"
"#,
        )
        .expect("cancel_world_work should be allowlistable in packet 1");

        assert_eq!(
            patch.agents.world_dispatch.allowed_actions.as_deref(),
            Some(&["cancel_world_work".to_string()][..])
        );
    }

    #[test]
    fn policy_patch_rejects_unknown_agents_world_dispatch_action() {
        let path = Path::new("policy.yaml");
        let err = parse_policy_patch_yaml(
            path,
            r#"
agents:
  world_dispatch:
    allowed_actions:
      - "unknown_world_dispatch_action"
"#,
        )
        .expect_err("unknown world-dispatch action must fail closed");

        let msg = err.to_string();
        assert!(
            msg.contains("agents.world_dispatch.allowed_actions"),
            "expected world dispatch action diagnostic, got: {msg}"
        );
        assert!(
            msg.contains("unknown_world_dispatch_action"),
            "expected invalid action value in diagnostic, got: {msg}"
        );
    }

    #[test]
    fn policy_patch_keeps_world_dispatch_action_allowlist_empty_when_absent() {
        let path = Path::new("policy.yaml");
        let patch = parse_policy_patch_yaml(
            path,
            r#"
agents:
  world_dispatch:
    enabled: true
"#,
        )
        .expect("omitting allowed_actions must keep deny-by-default defaults intact");

        assert_eq!(patch.agents.world_dispatch.allowed_actions, None);
    }

    #[test]
    fn policy_patch_parses_tuple_axes_under_existing_llm_constraints_root() {
        let path = Path::new("policy.yaml");
        let patch = parse_policy_patch_yaml(
            path,
            r#"
llm:
  allowed_backends:
    - "api:openai"
  constraints:
    routers:
      - "substrate_gateway"
    providers:
      - "openai"
    protocols:
      - "openai.responses"
    auth_authorities:
      - "openai_api_key"
"#,
        )
        .expect("tuple-axis keys should parse under llm.constraints");

        assert_eq!(
            patch.llm.allowed_backends.as_deref(),
            Some(&["api:openai".to_string()][..])
        );
        assert_eq!(
            patch.llm.constraints.routers.as_deref(),
            Some(&["substrate_gateway".to_string()][..])
        );
        assert_eq!(
            patch.llm.constraints.providers.as_deref(),
            Some(&["openai".to_string()][..])
        );
        assert_eq!(
            patch.llm.constraints.protocols.as_deref(),
            Some(&["openai.responses".to_string()][..])
        );
        assert_eq!(
            patch.llm.constraints.auth_authorities.as_deref(),
            Some(&["openai_api_key".to_string()][..])
        );
    }

    #[test]
    fn policy_patch_rejects_second_tuple_policy_roots_and_unknown_tuple_axes() {
        let path = Path::new("policy.yaml");
        let invalid_cases = [
            (
                r#"
llm:
  identity_tuple:
    providers:
      - "openai"
"#,
                "unknown field `identity_tuple`",
            ),
            (
                r#"
trace:
  identity_tuple:
    provider: "openai"
"#,
                "unknown field `trace`",
            ),
            (
                r#"
llm:
  constraints:
    clients:
      - "codex"
"#,
                "unknown field `clients`",
            ),
        ];

        for (raw, expected_fragment) in invalid_cases {
            let err = parse_policy_patch_yaml(path, raw)
                .expect_err("invalid tuple-policy roots should fail validation");
            assert!(
                err.to_string().contains(expected_fragment),
                "expected `{expected_fragment}` in error, got: {err}"
            );
        }
    }
}
