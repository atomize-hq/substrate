use crate::policy::{Policy, WorldFsDimensionPolicy, WorldFsEnforcement, WorldFsIsolation};
use anyhow::{anyhow, Context, Result};
use serde::ser::SerializeMap;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use substrate_common::paths as substrate_paths;
use substrate_common::WorldFsMode;

const WORKSPACE_MARKER_FILENAME: &str = "workspace.yaml";
const WORKSPACE_DISABLED_FILENAME: &str = "workspace.disabled";
const WORKSPACE_POLICY_FILENAME: &str = "policy.yaml";

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default, deny_unknown_fields)]
pub struct PolicyPatch {
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
    #[allow(dead_code)]
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
pub struct WorldFsPatch {
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
pub struct WorldFsDimensionPatch {
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
pub struct ResourceLimitsPatch {
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

#[derive(Debug, Clone)]
pub struct EffectivePolicySources {
    pub global_patch_path: Option<PathBuf>,
    pub workspace_patch_path: Option<PathBuf>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct PolicyExplainV1 {
    pub kind: String,
    pub keys: OrderedExplainKeys,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct PolicyExplainKey {
    pub merge_strategy: String,
    pub sources: Vec<PolicyExplainSource>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct PolicyExplainSource {
    pub layer: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct OrderedExplainKeys(BTreeMap<String, PolicyExplainKey>);

impl OrderedExplainKeys {
    fn insert(&mut self, key: String, value: PolicyExplainKey) {
        self.0.insert(key, value);
    }
}

impl Serialize for OrderedExplainKeys {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut entries: Vec<_> = self.0.iter().collect();
        entries.sort_by(|(a_key, a_val), (b_key, b_val)| {
            explain_key_rank(a_val)
                .cmp(&explain_key_rank(b_val))
                .then_with(|| a_key.cmp(b_key))
        });

        let mut map = serializer.serialize_map(Some(entries.len()))?;
        for (key, value) in entries {
            map.serialize_entry(key, value)?;
        }
        map.end()
    }
}

fn explain_key_rank(key: &PolicyExplainKey) -> u8 {
    key.sources
        .iter()
        .map(|source| match source.layer.as_str() {
            "global_patch" => 0,
            "default" => 1,
            "workspace_patch" => 2,
            _ => 5,
        })
        .min()
        .unwrap_or(5)
}

pub(crate) fn find_workspace_root(start: &Path) -> Option<PathBuf> {
    let mut start = start;
    if start.is_file() {
        start = start.parent()?;
    }
    let start = start.canonicalize().unwrap_or_else(|_| start.to_path_buf());

    for dir in start.ancestors() {
        let substrate_dir = dir.join(substrate_paths::SUBSTRATE_DIR_NAME);
        let marker = substrate_dir.join(WORKSPACE_MARKER_FILENAME);
        let disabled = substrate_dir.join(WORKSPACE_DISABLED_FILENAME);
        if marker.is_file() && !disabled.exists() {
            return Some(dir.to_path_buf());
        }
    }
    None
}

pub fn parse_policy_patch_yaml(path: &Path, raw: &str) -> Result<PolicyPatch> {
    let value: serde_yaml::Value = serde_yaml::from_str(raw).map_err(|err| {
        anyhow!(
            "invalid YAML in {}: {}",
            path.display(),
            err.to_string().trim()
        )
    })?;

    match &value {
        serde_yaml::Value::Null => return Ok(PolicyPatch::default()),
        serde_yaml::Value::Mapping(map) if map.is_empty() => return Ok(PolicyPatch::default()),
        _ => {}
    }

    let parsed: PolicyPatch = serde_yaml::from_value(value).map_err(|err| {
        anyhow!(
            "invalid YAML in {}: {}",
            path.display(),
            err.to_string().trim()
        )
    })?;

    Ok(parsed)
}

pub fn read_policy_patch_or_empty(path: &Path) -> Result<(PolicyPatch, bool)> {
    match fs::read_to_string(path) {
        Ok(raw) => Ok((parse_policy_patch_yaml(path, &raw)?, true)),
        Err(err) if err.kind() == io::ErrorKind::NotFound => Ok((PolicyPatch::default(), false)),
        Err(err) => Err(anyhow!("failed to read {}: {err}", path.display())),
    }
}

pub fn load_policy_from_path(path: &Path) -> Result<Policy> {
    let raw =
        fs::read_to_string(path).with_context(|| format!("failed to read {}", path.display()))?;
    let patch = parse_policy_patch_yaml(path, &raw)?;

    let mut policy = Policy::default();
    apply_policy_patch_over(&mut policy, &patch);
    validate_and_finalize_effective_policy(&mut policy)?;
    Ok(policy)
}

pub fn load_effective_policy_for_cwd(cwd: &Path) -> Result<(Policy, EffectivePolicySources)> {
    let global_path = substrate_paths::policy_file()?;
    let (global_patch, global_exists) = read_policy_patch_or_empty(&global_path)?;

    let workspace_root = find_workspace_root(cwd);
    let workspace_layer = if let Some(root) = &workspace_root {
        let path = root
            .join(substrate_paths::SUBSTRATE_DIR_NAME)
            .join(WORKSPACE_POLICY_FILENAME);
        match fs::read_to_string(&path) {
            Ok(raw) => Some((parse_policy_patch_yaml(&path, &raw)?, path)),
            Err(err) if err.kind() == io::ErrorKind::NotFound => None,
            Err(err) => return Err(anyhow!("failed to read {}: {err}", path.display())),
        }
    } else {
        None
    };

    let mut policy = Policy::default();
    apply_policy_patch_over(&mut policy, &global_patch);
    if let Some((workspace_patch, _path)) = &workspace_layer {
        apply_policy_patch_over(&mut policy, workspace_patch);
    }
    validate_and_finalize_effective_policy(&mut policy)?;

    Ok((
        policy,
        EffectivePolicySources {
            global_patch_path: global_exists.then_some(global_path),
            workspace_patch_path: workspace_layer.as_ref().map(|(_, p)| p.clone()),
        },
    ))
}

pub fn resolve_effective_policy_with_explain(
    cwd: &Path,
    explain: bool,
) -> Result<(Policy, Option<PolicyExplainV1>)> {
    let global_path = substrate_paths::policy_file()?;
    let (global_patch, _global_exists) = read_policy_patch_or_empty(&global_path)?;

    let workspace_root = find_workspace_root(cwd);
    let workspace_layer = if let Some(root) = &workspace_root {
        let path = root
            .join(substrate_paths::SUBSTRATE_DIR_NAME)
            .join(WORKSPACE_POLICY_FILENAME);
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
    let workspace_patch = workspace_layer.as_ref().map(|(p, _)| p);

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

    let mut effective = Policy::default();
    let mut explain_keys = if explain {
        Some(OrderedExplainKeys::default())
    } else {
        None
    };

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

    let (enforcement, enforcement_src) = resolve_replace_opt(
        effective.world_fs_enforcement,
        global_patch.world_fs.enforcement,
        workspace_patch.and_then(|p| p.world_fs.enforcement),
        workspace_enabled,
    );
    effective.world_fs_enforcement = enforcement;
    if let Some(keys) = &mut explain_keys {
        keys.insert(
            "world_fs.enforcement".to_string(),
            PolicyExplainKey {
                merge_strategy: "replace".to_string(),
                sources: vec![explain_source(
                    enforcement_src,
                    &global_path,
                    workspace_path,
                )],
            },
        );
    }

    let mut resolve_string_list =
        |key: &str,
         default: Option<Vec<String>>,
         global: Option<Vec<String>>,
         workspace: Option<Vec<String>>| {
            let (value, src) = resolve_replace_opt(default, global, workspace, workspace_enabled);
            if let Some(keys) = &mut explain_keys {
                keys.insert(
                    key.to_string(),
                    PolicyExplainKey {
                        merge_strategy: "replace".to_string(),
                        sources: vec![explain_source(src, &global_path, workspace_path)],
                    },
                );
            }
            value
        };

    let read_default_allow = effective
        .world_fs_read
        .as_ref()
        .map(|d| d.allow_list.clone());
    let read_default_deny = effective
        .world_fs_read
        .as_ref()
        .map(|d| d.deny_list.clone());
    let read_allow = resolve_string_list(
        "world_fs.read.allow_list",
        read_default_allow,
        global_patch.world_fs.read.allow_list.clone(),
        workspace_patch.and_then(|p| p.world_fs.read.allow_list.clone()),
    );
    let read_deny = resolve_string_list(
        "world_fs.read.deny_list",
        read_default_deny,
        global_patch.world_fs.read.deny_list.clone(),
        workspace_patch.and_then(|p| p.world_fs.read.deny_list.clone()),
    );
    effective.world_fs_read = if read_allow.is_none() && read_deny.is_none() {
        None
    } else {
        Some(WorldFsDimensionPolicy {
            allow_list: read_allow.unwrap_or_default(),
            deny_list: read_deny.unwrap_or_default(),
        })
    };

    let discover_default_allow = effective
        .world_fs_discover
        .as_ref()
        .map(|d| d.allow_list.clone());
    let discover_default_deny = effective
        .world_fs_discover
        .as_ref()
        .map(|d| d.deny_list.clone());
    let discover_allow = resolve_string_list(
        "world_fs.discover.allow_list",
        discover_default_allow,
        global_patch.world_fs.discover.allow_list.clone(),
        workspace_patch.and_then(|p| p.world_fs.discover.allow_list.clone()),
    );
    let discover_deny = resolve_string_list(
        "world_fs.discover.deny_list",
        discover_default_deny,
        global_patch.world_fs.discover.deny_list.clone(),
        workspace_patch.and_then(|p| p.world_fs.discover.deny_list.clone()),
    );
    effective.world_fs_discover = if discover_allow.is_none() && discover_deny.is_none() {
        None
    } else {
        Some(WorldFsDimensionPolicy {
            allow_list: discover_allow.unwrap_or_default(),
            deny_list: discover_deny.unwrap_or_default(),
        })
    };

    let write_default_allow = effective
        .world_fs_write
        .as_ref()
        .map(|d| d.allow_list.clone());
    let write_default_deny = effective
        .world_fs_write
        .as_ref()
        .map(|d| d.deny_list.clone());
    let write_allow = resolve_string_list(
        "world_fs.write.allow_list",
        write_default_allow,
        global_patch.world_fs.write.allow_list.clone(),
        workspace_patch.and_then(|p| p.world_fs.write.allow_list.clone()),
    );
    let write_deny = resolve_string_list(
        "world_fs.write.deny_list",
        write_default_deny,
        global_patch.world_fs.write.deny_list.clone(),
        workspace_patch.and_then(|p| p.world_fs.write.deny_list.clone()),
    );
    effective.world_fs_write = if write_allow.is_none() && write_deny.is_none() {
        None
    } else {
        Some(WorldFsDimensionPolicy {
            allow_list: write_allow.unwrap_or_default(),
            deny_list: write_deny.unwrap_or_default(),
        })
    };

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

    let (metadata, metadata_src) = resolve_replace_opt(
        if effective.metadata.is_empty() {
            None
        } else {
            Some(hashmap_to_btree(&effective.metadata))
        },
        global_patch.metadata.clone(),
        workspace_patch.and_then(|p| p.metadata.clone()),
        workspace_enabled,
    );
    effective.metadata = metadata.as_ref().map(btree_to_hashmap).unwrap_or_default();
    if let Some(keys) = &mut explain_keys {
        keys.insert(
            "metadata".to_string(),
            PolicyExplainKey {
                merge_strategy: "replace".to_string(),
                sources: vec![explain_source(metadata_src, &global_path, workspace_path)],
            },
        );
    }

    validate_and_finalize_effective_policy(&mut effective)?;

    let explain = explain_keys.map(|keys| PolicyExplainV1 {
        kind: "substrate.policy.explain.v1".to_string(),
        keys,
    });

    Ok((effective, explain))
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

    apply_world_fs_dimension_patch(&mut target.world_fs_discover, &patch.world_fs.discover);
    apply_world_fs_dimension_patch(&mut target.world_fs_read, &patch.world_fs.read);
    apply_world_fs_dimension_patch(&mut target.world_fs_write, &patch.world_fs.write);

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

fn apply_world_fs_dimension_patch(
    target: &mut Option<WorldFsDimensionPolicy>,
    patch: &WorldFsDimensionPatch,
) {
    if patch.is_empty() {
        return;
    }

    if target.is_none() {
        *target = Some(WorldFsDimensionPolicy {
            allow_list: Vec::new(),
            deny_list: Vec::new(),
        });
    }

    let Some(target) = target.as_mut() else {
        return;
    };

    if let Some(v) = &patch.allow_list {
        target.allow_list = v.clone();
    }
    if let Some(v) = &patch.deny_list {
        target.deny_list = v.clone();
    }
}

fn validate_and_finalize_effective_policy(policy: &mut Policy) -> Result<()> {
    match policy.world_fs_isolation {
        WorldFsIsolation::Workspace => {
            if policy.world_fs_enforcement.is_some() {
                return Err(anyhow!(
                    "world_fs.enforcement is only supported when world_fs.isolation=full"
                ));
            }
            if policy.world_fs_read.is_some()
                || policy.world_fs_discover.is_some()
                || policy.world_fs_write.is_some()
            {
                return Err(anyhow!(
                    "world_fs.read/discover/write are only supported when world_fs.isolation=full"
                ));
            }
        }
        WorldFsIsolation::Full => {
            if policy.world_fs_read.is_none() {
                return Err(anyhow!(
                    "world_fs.read.allow_list is required when world_fs.isolation=full"
                ));
            }

            match policy.world_fs_mode {
                WorldFsMode::ReadOnly => {
                    if policy.world_fs_write.is_some() {
                        return Err(anyhow!(
                            "world_fs.write must be omitted when world_fs.mode=read_only"
                        ));
                    }
                }
                WorldFsMode::Writable => {
                    if policy.world_fs_write.is_none() {
                        return Err(anyhow!(
                            "world_fs.write.allow_list is required when world_fs.mode=writable"
                        ));
                    }
                }
            }
        }
    }

    if let Some(dimension) = policy.world_fs_read.as_mut() {
        normalize_and_validate_dimension("world_fs.read", dimension)?;
    }
    if let Some(dimension) = policy.world_fs_discover.as_mut() {
        normalize_and_validate_dimension("world_fs.discover", dimension)?;
    }
    if let Some(dimension) = policy.world_fs_write.as_mut() {
        normalize_and_validate_dimension("world_fs.write", dimension)?;
    }

    let any_deny = policy
        .world_fs_read
        .as_ref()
        .is_some_and(|d| !d.deny_list.is_empty())
        || policy
            .world_fs_discover
            .as_ref()
            .is_some_and(|d| !d.deny_list.is_empty())
        || policy
            .world_fs_write
            .as_ref()
            .is_some_and(|d| !d.deny_list.is_empty());

    if any_deny {
        if policy.world_fs_isolation != WorldFsIsolation::Full {
            return Err(anyhow!(
                "deny_list is only supported when world_fs.isolation=full"
            ));
        }
        if policy.world_fs_enforcement.is_none() {
            return Err(anyhow!(
                "world_fs.enforcement must be present when any deny_list is non-empty"
            ));
        }
        if !policy.world_fs_require_world {
            return Err(anyhow!("deny_list requires world_fs.require_world=true"));
        }
    } else if policy.world_fs_enforcement.is_some() {
        return Err(anyhow!(
            "world_fs.enforcement is only valid when at least one deny_list is non-empty"
        ));
    }

    // Compatibility: callers currently consume fs_read/fs_write as the effective read/write allow
    // lists. Under V2, those are sourced from read/write.allow_list.
    if let Some(dimension) = policy.world_fs_read.as_ref() {
        policy.fs_read = dimension.allow_list.clone();
    }
    if let Some(dimension) = policy.world_fs_write.as_ref() {
        policy.fs_write = dimension.allow_list.clone();
    }

    Ok(())
}

fn normalize_and_validate_dimension(
    prefix: &str,
    dimension: &mut WorldFsDimensionPolicy,
) -> Result<()> {
    if dimension.allow_list.is_empty() {
        return Err(anyhow!("{prefix}.allow_list must be non-empty"));
    }

    let mut allow_out = Vec::with_capacity(dimension.allow_list.len());
    for raw in &dimension.allow_list {
        let normalized =
            normalize_project_pattern(raw).map_err(|e| anyhow!("{prefix}.allow_list: {e}"))?;
        if normalized.contains(['*', '?', '[', ']']) {
            return Err(anyhow!(
                "{prefix}.allow_list contains glob metacharacters; wildcards are not supported in allow_list"
            ));
        }
        allow_out.push(normalized);
    }
    dimension.allow_list = allow_out;

    let mut deny_out = Vec::with_capacity(dimension.deny_list.len());
    for raw in &dimension.deny_list {
        let normalized =
            normalize_project_pattern(raw).map_err(|e| anyhow!("{prefix}.deny_list: {e}"))?;
        if normalized.contains(['?', '[', ']']) {
            return Err(anyhow!(
                "{prefix}.deny_list contains unsupported glob metacharacters ('?' or character classes)"
            ));
        }
        validate_deny_wildcards(&normalized).map_err(|e| anyhow!("{prefix}.deny_list: {e}"))?;
        deny_out.push(normalized);
    }
    dimension.deny_list = deny_out;
    Ok(())
}

fn validate_deny_wildcards(pattern: &str) -> Result<(), String> {
    let mut run = 0usize;
    for ch in pattern.chars() {
        if ch == '*' {
            run += 1;
            if run > 2 {
                return Err(
                    "deny_list wildcard runs must be '*' or '**' (no '***' or longer)".to_string(),
                );
            }
        } else {
            run = 0;
        }
    }
    Ok(())
}

fn normalize_project_pattern(raw: &str) -> Result<String, String> {
    let raw = raw.trim();
    if raw.is_empty() {
        return Err("pattern must not be empty".to_string());
    }

    if raw.starts_with('/') {
        return Err("absolute patterns are invalid".to_string());
    }

    // Split on '/', drop empty segments and '.' segments, and reject '..'.
    let mut segments: Vec<&str> = Vec::new();
    for seg in raw.split('/') {
        let seg = seg.trim();
        if seg.is_empty() || seg == "." {
            continue;
        }
        if seg == ".." {
            return Err("pattern must not contain '..' segments".to_string());
        }
        segments.push(seg);
    }

    let normalized = if segments.is_empty() {
        ".".to_string()
    } else {
        segments.join("/")
    };
    Ok(normalized)
}

fn btree_to_hashmap(map: &BTreeMap<String, String>) -> HashMap<String, String> {
    map.iter().map(|(k, v)| (k.clone(), v.clone())).collect()
}

fn hashmap_to_btree(map: &HashMap<String, String>) -> BTreeMap<String, String> {
    map.iter().map(|(k, v)| (k.clone(), v.clone())).collect()
}
