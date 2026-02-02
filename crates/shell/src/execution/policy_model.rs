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
    Policy, PolicyExplainV1, WorldFsDimensionPolicy, WorldFsEnforcement, WorldFsIsolation,
};
use substrate_common::WorldFsMode;

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default, deny_unknown_fields)]
pub(crate) struct PolicyPatch {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "WorldFsPatch::is_empty")]
    pub world_fs: WorldFsPatch,
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
pub(crate) struct WorldFsPatch {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mode: Option<WorldFsMode>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub isolation: Option<WorldFsIsolation>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub require_world: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enforcement: Option<WorldFsEnforcement>,
    #[serde(skip_serializing_if = "WorldFsDimensionPatch::is_empty")]
    pub discover: WorldFsDimensionPatch,
    #[serde(skip_serializing_if = "WorldFsDimensionPatch::is_empty")]
    pub read: WorldFsDimensionPatch,
    #[serde(skip_serializing_if = "WorldFsDimensionPatch::is_empty")]
    pub write: WorldFsDimensionPatch,
}

impl WorldFsPatch {
    fn is_empty(&self) -> bool {
        self.mode.is_none()
            && self.isolation.is_none()
            && self.require_world.is_none()
            && self.enforcement.is_none()
            && self.discover.is_empty()
            && self.read.is_empty()
            && self.write.is_empty()
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
    Ok(parsed)
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
    if policy.world_fs_mode == WorldFsMode::ReadOnly && !policy.world_fs_require_world {
        return Err(config_model::user_error(
            "world_fs.mode=read_only requires world_fs.require_world=true",
        ));
    }
    if policy.world_fs_isolation == WorldFsIsolation::Full && !policy.world_fs_require_world {
        return Err(config_model::user_error(
            "world_fs.isolation=full requires world_fs.require_world=true",
        ));
    }
    Ok(())
}

fn apply_policy_patch_over(target: &mut Policy, patch: &PolicyPatch) {
    if let Some(v) = &patch.id {
        target.id = v.clone();
    }
    if let Some(v) = &patch.name {
        target.name = v.clone();
    }
    if let Some(v) = patch.world_fs.mode {
        target.world_fs_mode = v;
    }
    if let Some(v) = patch.world_fs.isolation {
        target.world_fs_isolation = v;
    }
    if let Some(v) = patch.world_fs.require_world {
        target.world_fs_require_world = v;
    }
    if let Some(v) = patch.world_fs.enforcement {
        target.world_fs_enforcement = Some(v);
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

        "world_fs.mode" => Ok(patch.world_fs.mode.take().is_some()),
        "world_fs.isolation" => Ok(patch.world_fs.isolation.take().is_some()),
        "world_fs.require_world" => Ok(patch.world_fs.require_world.take().is_some()),
        "world_fs.enforcement" => Ok(patch.world_fs.enforcement.take().is_some()),

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
        "world_fs.write.allow_list" => Ok(patch.world_fs.write.allow_list.take().is_some()),
        "world_fs.write.deny_list" => Ok(patch.world_fs.write.deny_list.take().is_some()),
        "world_fs.write" => {
            let was = !patch.world_fs.write.is_empty();
            patch.world_fs.write = WorldFsDimensionPatch::default();
            Ok(was)
        }

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

        "world_fs.mode" => apply_enum_world_fs_mode_opt(&mut patch.world_fs.mode, update),
        "world_fs.isolation" => {
            apply_enum_world_fs_isolation_opt(&mut patch.world_fs.isolation, update)
        }
        "world_fs.require_world" => {
            apply_bool_opt(&mut patch.world_fs.require_world, &update.op, &update.value)
        }
        "world_fs.enforcement" => {
            apply_enum_world_fs_enforcement_opt(&mut patch.world_fs.enforcement, update)
        }
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
        "world_fs.write.allow_list" => {
            apply_string_list_opt(&mut patch.world_fs.write.allow_list, update)
        }
        "world_fs.write.deny_list" => {
            apply_string_list_opt(&mut patch.world_fs.write.deny_list, update)
        }

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

fn apply_enum_world_fs_mode_opt(
    target: &mut Option<WorldFsMode>,
    update: &ConfigUpdate,
) -> Result<bool> {
    let UpdateOp::Set = update.op else {
        return Err(config_model::user_error(
            "operator += and -= are only valid for list keys",
        ));
    };
    let next = match update.value.trim().to_ascii_lowercase().as_str() {
        "writable" => WorldFsMode::Writable,
        "read_only" => WorldFsMode::ReadOnly,
        _ => {
            return Err(config_model::user_error(format!(
                "invalid world_fs.mode '{}' (expected writable or read_only)",
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

fn apply_enum_world_fs_isolation_opt(
    target: &mut Option<WorldFsIsolation>,
    update: &ConfigUpdate,
) -> Result<bool> {
    let UpdateOp::Set = update.op else {
        return Err(config_model::user_error(
            "operator += and -= are only valid for list keys",
        ));
    };
    let next = match update.value.trim().to_ascii_lowercase().as_str() {
        "workspace" | "project" => WorldFsIsolation::Workspace,
        "full" => WorldFsIsolation::Full,
        _ => {
            return Err(config_model::user_error(format!(
                "invalid world_fs.isolation '{}' (expected workspace or full)",
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

fn apply_enum_world_fs_enforcement_opt(
    target: &mut Option<WorldFsEnforcement>,
    update: &ConfigUpdate,
) -> Result<bool> {
    let UpdateOp::Set = update.op else {
        return Err(config_model::user_error(
            "operator += and -= are only valid for list keys",
        ));
    };
    let next = match update.value.trim().to_ascii_lowercase().as_str() {
        "strict" => WorldFsEnforcement::Strict,
        "best_effort" => WorldFsEnforcement::BestEffort,
        _ => {
            return Err(config_model::user_error(format!(
                "invalid world_fs.enforcement '{}' (expected strict or best_effort)",
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
    fn policy_patch_accepts_world_fs_enforcement_and_dimension_lists() {
        let mut patch = PolicyPatch::default();
        let updates = vec![
            ConfigUpdate {
                key: "world_fs.mode".to_string(),
                op: UpdateOp::Set,
                value: "read_only".to_string(),
            },
            ConfigUpdate {
                key: "world_fs.isolation".to_string(),
                op: UpdateOp::Set,
                value: "full".to_string(),
            },
            ConfigUpdate {
                key: "world_fs.require_world".to_string(),
                op: UpdateOp::Set,
                value: "true".to_string(),
            },
            ConfigUpdate {
                key: "world_fs.enforcement".to_string(),
                op: UpdateOp::Set,
                value: "best_effort".to_string(),
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

        assert_eq!(
            patch.world_fs.enforcement,
            Some(WorldFsEnforcement::BestEffort)
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
}
