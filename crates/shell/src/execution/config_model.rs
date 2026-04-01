use crate::execution::value_parse::parse_bool_flag;
use crate::execution::workspace;
use anyhow::{anyhow, Context, Result};
use serde::ser::SerializeMap;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::env;
use std::error::Error as StdError;
use std::fmt;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};
use std::time::SystemTime;
use substrate_common::paths as substrate_paths;
use substrate_common::WorldRootMode;

pub(crate) const PROTECTED_EXCLUDES: [&str; 2] = [".git/**", ".substrate/**"];

#[derive(Debug, Clone, PartialEq, Eq)]
struct FileStatKey {
    exists: bool,
    mtime: Option<SystemTime>,
    size: Option<u64>,
}

impl FileStatKey {
    fn for_path(path: &Path) -> Result<Self> {
        match fs::metadata(path) {
            Ok(meta) => Ok(Self {
                exists: true,
                mtime: meta.modified().ok(),
                size: Some(meta.len()),
            }),
            Err(err) if err.kind() == io::ErrorKind::NotFound => Ok(Self {
                exists: false,
                mtime: None,
                size: None,
            }),
            Err(err) => Err(err).with_context(|| format!("failed to stat {}", path.display())),
        }
    }
}

#[derive(Debug, Clone)]
struct ConfigPatchCacheEntry {
    workspace_root: Option<PathBuf>,
    global_path: PathBuf,
    workspace_path: Option<PathBuf>,
    global_stat: FileStatKey,
    workspace_stat: Option<FileStatKey>,
    global_patch: SubstrateConfigPatch,
    workspace_patch: Option<SubstrateConfigPatch>,
}

static CONFIG_PATCH_CACHE: OnceLock<Mutex<Option<ConfigPatchCacheEntry>>> = OnceLock::new();

pub(crate) fn invalidate_config_cache() {
    let cache = CONFIG_PATCH_CACHE.get_or_init(|| Mutex::new(None));
    if let Ok(mut guard) = cache.lock() {
        *guard = None;
    }
}

#[derive(Debug)]
pub(crate) struct UserError {
    message: String,
}

impl UserError {
    fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl fmt::Display for UserError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

impl StdError for UserError {}

pub(crate) fn user_error(message: impl Into<String>) -> anyhow::Error {
    anyhow::Error::new(UserError::new(message))
}

pub(crate) fn is_user_error(err: &anyhow::Error) -> bool {
    err.chain().any(|cause| cause.is::<UserError>())
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default, deny_unknown_fields)]
pub(crate) struct SubstrateConfig {
    pub world: WorldConfig,
    pub policy: PolicyConfig,
    pub sync: SyncConfig,
    pub repl: ReplConfig,
}

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum ReplExitCwdMode {
    Entered,
    LastWorld,
}

impl Default for ReplExitCwdMode {
    fn default() -> Self {
        Self::Entered
    }
}

impl ReplExitCwdMode {
    pub(crate) fn parse_insensitive(raw: &str) -> Option<Self> {
        match raw.trim().to_ascii_lowercase().as_str() {
            "entered" => Some(Self::Entered),
            "last_world" => Some(Self::LastWorld),
            _ => None,
        }
    }
}

impl<'de> Deserialize<'de> for ReplExitCwdMode {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let raw = String::deserialize(deserializer)?;
        Self::parse_insensitive(&raw).ok_or_else(|| {
            serde::de::Error::custom(format!(
                "invalid repl.exit_cwd '{}'; expected entered or last_world",
                raw
            ))
        })
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default, deny_unknown_fields)]
pub(crate) struct ReplConfig {
    pub exit_cwd: ReplExitCwdMode,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default, deny_unknown_fields)]
pub(crate) struct WorldConfig {
    pub enabled: bool,
    pub anchor_mode: WorldRootMode,
    pub anchor_path: String,
    pub caged: bool,
    pub net: WorldNetConfig,
    pub env: WorldEnvConfig,
    pub deps: WorldDepsConfig,
}

impl Default for WorldConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            anchor_mode: WorldRootMode::Project,
            anchor_path: String::new(),
            caged: true,
            net: WorldNetConfig::default(),
            env: WorldEnvConfig::default(),
            deps: WorldDepsConfig::default(),
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default, deny_unknown_fields)]
pub(crate) struct WorldNetConfig {
    pub filter: bool,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default, deny_unknown_fields)]
pub(crate) struct WorldEnvConfig {
    pub inherit_from_host: bool,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default, deny_unknown_fields)]
pub(crate) struct WorldDepsConfig {
    pub enabled: Vec<String>,
    pub inventory_mode: WorldDepsInventoryMode,
    pub builtins: WorldDepsBuiltinsMode,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum WorldDepsInventoryMode {
    Merged,
    WorkspaceOnly,
}

impl Default for WorldDepsInventoryMode {
    fn default() -> Self {
        Self::Merged
    }
}

impl WorldDepsInventoryMode {
    pub(crate) fn parse_insensitive(raw: &str) -> Option<Self> {
        match raw.trim().to_ascii_lowercase().as_str() {
            "merged" => Some(Self::Merged),
            "workspace_only" => Some(Self::WorkspaceOnly),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum WorldDepsBuiltinsMode {
    Enabled,
    Disabled,
}

impl Default for WorldDepsBuiltinsMode {
    fn default() -> Self {
        Self::Enabled
    }
}

impl WorldDepsBuiltinsMode {
    pub(crate) fn parse_insensitive(raw: &str) -> Option<Self> {
        match raw.trim().to_ascii_lowercase().as_str() {
            "enabled" => Some(Self::Enabled),
            "disabled" => Some(Self::Disabled),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum PolicyMode {
    Disabled,
    Observe,
    Enforce,
}

impl Default for PolicyMode {
    fn default() -> Self {
        Self::Observe
    }
}

impl PolicyMode {
    pub(crate) fn parse_insensitive(raw: &str) -> Option<Self> {
        match raw.trim().to_ascii_lowercase().as_str() {
            "disabled" => Some(Self::Disabled),
            "observe" => Some(Self::Observe),
            "enforce" => Some(Self::Enforce),
            _ => None,
        }
    }

    pub(crate) fn as_str(&self) -> &'static str {
        match self {
            Self::Disabled => "disabled",
            Self::Observe => "observe",
            Self::Enforce => "enforce",
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default, deny_unknown_fields)]
pub(crate) struct PolicyConfig {
    pub mode: PolicyMode,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum SyncDirection {
    FromWorld,
    FromHost,
    Both,
}

impl Default for SyncDirection {
    fn default() -> Self {
        Self::FromWorld
    }
}

impl SyncDirection {
    pub(crate) fn parse_insensitive(raw: &str) -> Option<Self> {
        match raw.trim().to_ascii_lowercase().as_str() {
            "from_world" => Some(Self::FromWorld),
            "from_host" => Some(Self::FromHost),
            "both" => Some(Self::Both),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum SyncConflictPolicy {
    PreferHost,
    PreferWorld,
    Abort,
}

impl Default for SyncConflictPolicy {
    fn default() -> Self {
        Self::PreferHost
    }
}

impl SyncConflictPolicy {
    pub(crate) fn parse_insensitive(raw: &str) -> Option<Self> {
        match raw.trim().to_ascii_lowercase().as_str() {
            "prefer_host" => Some(Self::PreferHost),
            "prefer_world" => Some(Self::PreferWorld),
            "abort" => Some(Self::Abort),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default, deny_unknown_fields)]
pub(crate) struct SyncConfig {
    pub auto_sync: bool,
    pub direction: SyncDirection,
    pub conflict_policy: SyncConflictPolicy,
    pub exclude: Vec<String>,
}

#[derive(Debug, Default, Clone)]
pub(crate) struct CliConfigOverrides {
    pub world_enabled: Option<bool>,
    pub anchor_mode: Option<WorldRootMode>,
    pub anchor_path: Option<String>,
    pub caged: Option<bool>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default, deny_unknown_fields)]
pub(crate) struct SubstrateConfigPatch {
    #[serde(skip_serializing_if = "WorldConfigPatch::is_empty")]
    pub world: WorldConfigPatch,
    #[serde(skip_serializing_if = "PolicyConfigPatch::is_empty")]
    pub policy: PolicyConfigPatch,
    #[serde(skip_serializing_if = "SyncConfigPatch::is_empty")]
    pub sync: SyncConfigPatch,
    #[serde(skip_serializing_if = "ReplConfigPatch::is_empty")]
    pub repl: ReplConfigPatch,
}

impl SubstrateConfigPatch {
    pub(crate) fn is_empty(&self) -> bool {
        self.world.is_empty()
            && self.policy.is_empty()
            && self.sync.is_empty()
            && self.repl.is_empty()
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default, deny_unknown_fields)]
pub(crate) struct WorldConfigPatch {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enabled: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub anchor_mode: Option<WorldRootMode>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub anchor_path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub caged: Option<bool>,
    #[serde(skip_serializing_if = "WorldNetConfigPatch::is_empty")]
    pub net: WorldNetConfigPatch,
    #[serde(skip_serializing_if = "WorldEnvConfigPatch::is_empty")]
    pub env: WorldEnvConfigPatch,
    #[serde(skip_serializing_if = "WorldDepsConfigPatch::is_empty")]
    pub deps: WorldDepsConfigPatch,
}

impl WorldConfigPatch {
    fn is_empty(&self) -> bool {
        self.enabled.is_none()
            && self.anchor_mode.is_none()
            && self.anchor_path.is_none()
            && self.caged.is_none()
            && self.net.is_empty()
            && self.env.is_empty()
            && self.deps.is_empty()
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default, deny_unknown_fields)]
pub(crate) struct WorldNetConfigPatch {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filter: Option<bool>,
}

impl WorldNetConfigPatch {
    fn is_empty(&self) -> bool {
        self.filter.is_none()
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default, deny_unknown_fields)]
pub(crate) struct WorldEnvConfigPatch {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub inherit_from_host: Option<bool>,
}

impl WorldEnvConfigPatch {
    fn is_empty(&self) -> bool {
        self.inherit_from_host.is_none()
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default, deny_unknown_fields)]
pub(crate) struct WorldDepsConfigPatch {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enabled: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub inventory_mode: Option<WorldDepsInventoryMode>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub builtins: Option<WorldDepsBuiltinsMode>,
}

impl WorldDepsConfigPatch {
    fn is_empty(&self) -> bool {
        self.enabled.is_none() && self.inventory_mode.is_none() && self.builtins.is_none()
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default, deny_unknown_fields)]
pub(crate) struct PolicyConfigPatch {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mode: Option<PolicyMode>,
}

impl PolicyConfigPatch {
    fn is_empty(&self) -> bool {
        self.mode.is_none()
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default, deny_unknown_fields)]
pub(crate) struct SyncConfigPatch {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auto_sync: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub direction: Option<SyncDirection>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub conflict_policy: Option<SyncConflictPolicy>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exclude: Option<Vec<String>>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default, deny_unknown_fields)]
pub(crate) struct ReplConfigPatch {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exit_cwd: Option<ReplExitCwdMode>,
}

impl ReplConfigPatch {
    fn is_empty(&self) -> bool {
        self.exit_cwd.is_none()
    }
}

impl SyncConfigPatch {
    fn is_empty(&self) -> bool {
        self.auto_sync.is_none()
            && self.direction.is_none()
            && self.conflict_policy.is_none()
            && self.exclude.is_none()
    }
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub(crate) struct ConfigExplainV1 {
    pub kind: String,
    pub keys: OrderedExplainKeys,
}

impl ConfigExplainV1 {
    pub(crate) fn world_enabled_explain(&self) -> Option<&ConfigExplainKey> {
        self.keys.0.get("world.enabled")
    }
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub(crate) struct ConfigExplainKey {
    pub merge_strategy: String,
    pub sources: Vec<ConfigExplainSource>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub(crate) struct ConfigExplainSource {
    pub layer: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub(crate) struct WorldDisableSource {
    pub key: &'static str,
    pub layer: &'static str,
    pub value_display: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub flag: Option<&'static str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub env: Option<&'static str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path_display: Option<&'static str>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub(crate) struct WorldDisableAttribution {
    pub reason: &'static str,
    pub source: WorldDisableSource,
}

pub(crate) type DoctorDisableSource = WorldDisableSource;
pub(crate) type DoctorDisableAttribution = WorldDisableAttribution;

#[allow(dead_code)]
pub(crate) fn world_disable_attribution_message(
    world_enabled: bool,
    world_enabled_explain: Option<&ConfigExplainKey>,
) -> Option<&'static str> {
    world_disable_attribution(world_enabled, world_enabled_explain)
        .map(|attribution| attribution.reason)
}

pub(crate) fn world_disable_attribution(
    world_enabled: bool,
    world_enabled_explain: Option<&ConfigExplainKey>,
) -> Option<WorldDisableAttribution> {
    if world_enabled {
        return None;
    }

    let Some(explain) = world_enabled_explain else {
        return Some(WorldDisableAttribution {
            reason: "world isolation disabled by effective config (source unknown)",
            source: WorldDisableSource {
                key: "world.enabled",
                layer: "source_unknown",
                value_display: false,
                flag: None,
                env: None,
                path_display: None,
            },
        });
    };

    let Some(source) = explain.sources.as_slice().first() else {
        return Some(WorldDisableAttribution {
            reason: "world isolation disabled by effective config (source unknown)",
            source: WorldDisableSource {
                key: "world.enabled",
                layer: "source_unknown",
                value_display: false,
                flag: None,
                env: None,
                path_display: None,
            },
        });
    };

    if explain.sources.len() != 1 {
        return Some(WorldDisableAttribution {
            reason: "world isolation disabled by effective config (source unknown)",
            source: WorldDisableSource {
                key: "world.enabled",
                layer: "source_unknown",
                value_display: false,
                flag: None,
                env: None,
                path_display: None,
            },
        });
    }

    match source.layer.as_str() {
        "cli_flag" => Some(WorldDisableAttribution {
            reason: "world isolation disabled by CLI flag --no-world",
            source: WorldDisableSource {
                key: "world.enabled",
                layer: "cli_flag",
                value_display: false,
                flag: Some("--no-world"),
                env: None,
                path_display: None,
            },
        }),
        "override_env" => Some(WorldDisableAttribution {
            reason: "world isolation disabled by env override SUBSTRATE_OVERRIDE_WORLD=disabled",
            source: WorldDisableSource {
                key: "world.enabled",
                layer: "override_env",
                value_display: false,
                flag: None,
                env: Some("SUBSTRATE_OVERRIDE_WORLD"),
                path_display: None,
            },
        }),
        "workspace_patch" => Some(WorldDisableAttribution {
            reason: "world isolation disabled by workspace config <workspace>/.substrate/workspace.yaml (world.enabled: false)",
            source: WorldDisableSource {
                key: "world.enabled",
                layer: "workspace_patch",
                value_display: false,
                flag: None,
                env: None,
                path_display: Some("<workspace>/.substrate/workspace.yaml"),
            },
        }),
        "global_patch" => Some(WorldDisableAttribution {
            reason: "world isolation disabled by global config $SUBSTRATE_HOME/config.yaml (world.enabled: false)",
            source: WorldDisableSource {
                key: "world.enabled",
                layer: "global_patch",
                value_display: false,
                flag: None,
                env: None,
                path_display: Some("$SUBSTRATE_HOME/config.yaml"),
            },
        }),
        "default" => Some(WorldDisableAttribution {
            reason: "world isolation disabled by default config (world.enabled: false)",
            source: WorldDisableSource {
                key: "world.enabled",
                layer: "default",
                value_display: false,
                flag: None,
                env: None,
                path_display: None,
            },
        }),
        _ => Some(WorldDisableAttribution {
            reason: "world isolation disabled by effective config (source unknown)",
            source: WorldDisableSource {
                key: "world.enabled",
                layer: "source_unknown",
                value_display: false,
                flag: None,
                env: None,
                path_display: None,
            },
        }),
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct OrderedExplainKeys(BTreeMap<String, ConfigExplainKey>);

impl OrderedExplainKeys {
    fn insert(&mut self, key: String, value: ConfigExplainKey) {
        self.0.insert(key, value);
    }

    #[cfg(test)]
    fn get(&self, key: &str) -> Option<&ConfigExplainKey> {
        self.0.get(key)
    }
}

impl Serialize for OrderedExplainKeys {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        // Deterministic bytes are required, but we also keep the output stable for simple
        // string-scanning consumers by ensuring global-layer entries serialize before
        // workspace-layer entries.
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

fn explain_key_rank(key: &ConfigExplainKey) -> u8 {
    key.sources
        .iter()
        .map(|source| match source.layer.as_str() {
            "global_patch" => 0,
            "default" => 1,
            "workspace_patch" => 2,
            "override_env" => 3,
            "cli_flag" => 4,
            _ => 5,
        })
        .min()
        .unwrap_or(5)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum UpdateOp {
    Set,
    Append,
    Remove,
}

#[derive(Debug, Clone)]
pub(crate) struct ConfigUpdate {
    pub key: String,
    pub op: UpdateOp,
    pub value: String,
}

pub(crate) fn global_config_path() -> Result<PathBuf> {
    substrate_paths::config_file()
}

pub(crate) fn read_global_config_patch_or_empty() -> Result<(SubstrateConfigPatch, bool)> {
    let path = global_config_path()?;
    match fs::read_to_string(&path) {
        Ok(raw) => Ok((parse_config_patch_yaml(&path, &raw)?, true)),
        Err(err) if err.kind() == io::ErrorKind::NotFound => {
            Ok((SubstrateConfigPatch::default(), false))
        }
        Err(err) => Err(anyhow!("failed to read {}: {err}", path.display())),
    }
}

pub(crate) fn read_global_config_or_defaults() -> Result<(SubstrateConfig, bool)> {
    let (patch, existed) = read_global_config_patch_or_empty()?;
    let cfg = resolve_effective_from_layers(
        &patch,
        &global_config_path()?,
        None,
        &EnvOverrides::default(),
        &CliConfigOverrides::default(),
        false,
        false,
    )?
    .0;
    Ok((cfg, existed))
}

pub(crate) fn parse_config_yaml(path: &Path, raw: &str) -> Result<SubstrateConfig> {
    let value: serde_yaml::Value = serde_yaml::from_str(raw).map_err(|err| {
        user_error(format!(
            "invalid YAML in {}: {}",
            path.display(),
            err.to_string().trim()
        ))
    })?;

    match &value {
        serde_yaml::Value::Null => return Ok(SubstrateConfig::default()),
        serde_yaml::Value::Mapping(map) if map.is_empty() => return Ok(SubstrateConfig::default()),
        _ => {}
    }

    let parsed: SubstrateConfig = serde_yaml::from_value(value).map_err(|err| {
        user_error(format!(
            "invalid YAML in {}: {}",
            path.display(),
            err.to_string().trim()
        ))
    })?;
    validate_config(&parsed)?;
    Ok(parsed)
}

pub(crate) fn parse_config_patch_yaml(path: &Path, raw: &str) -> Result<SubstrateConfigPatch> {
    let value: serde_yaml::Value = serde_yaml::from_str(raw).map_err(|err| {
        user_error(format!(
            "invalid YAML in {}: {}",
            path.display(),
            err.to_string().trim()
        ))
    })?;

    match &value {
        serde_yaml::Value::Null => return Ok(SubstrateConfigPatch::default()),
        serde_yaml::Value::Mapping(map) if map.is_empty() => {
            return Ok(SubstrateConfigPatch::default())
        }
        _ => {}
    }

    let parsed: SubstrateConfigPatch = serde_yaml::from_value(value).map_err(|err| {
        user_error(format!(
            "invalid YAML in {}: {}",
            path.display(),
            err.to_string().trim()
        ))
    })?;
    Ok(parsed)
}

pub(crate) fn resolve_effective_config(
    cwd: &Path,
    cli: &CliConfigOverrides,
) -> Result<SubstrateConfig> {
    Ok(resolve_effective_config_with_explain(cwd, cli, false)?.0)
}

pub(crate) fn resolve_effective_config_with_explain(
    cwd: &Path,
    cli: &CliConfigOverrides,
    explain: bool,
) -> Result<(SubstrateConfig, Option<ConfigExplainV1>)> {
    let env_overrides = parse_env_overrides()?;
    let (global_patch, global_path, workspace_layer) = load_config_patch_layers_cached(cwd)?;
    let workspace_ref = workspace_layer
        .as_ref()
        .map(|(p, path)| (p, path.as_path()));

    resolve_effective_from_layers(
        &global_patch,
        &global_path,
        workspace_ref,
        &env_overrides,
        cli,
        explain,
        true,
    )
}

fn load_config_patch_layers_cached(cwd: &Path) -> Result<LoadedConfigPatchLayers> {
    let global_path = global_config_path()?;
    let workspace_root = workspace::find_workspace_root(cwd);
    let workspace_path = workspace_root
        .as_ref()
        .map(|root| workspace::workspace_marker_path(root));

    if let Some(root) = &workspace_root {
        let legacy = workspace::workspace_legacy_settings_path(root);
        if legacy.exists() {
            return Err(user_error(format!(
                "substrate: unsupported legacy workspace config detected:\n  - {}\nConfig is now read from:\n  - {}\nNext steps:\n  - Delete the legacy file and use `substrate config workspace set ...`\n",
                legacy.display(),
                workspace::workspace_marker_path(root).display()
            )));
        }
    }

    let global_stat = FileStatKey::for_path(&global_path)?;
    let workspace_stat = workspace_path
        .as_ref()
        .map(|path| FileStatKey::for_path(path))
        .transpose()?;

    let cache = CONFIG_PATCH_CACHE.get_or_init(|| Mutex::new(None));
    if let Ok(guard) = cache.lock() {
        if let Some(entry) = guard.as_ref() {
            if entry.workspace_root == workspace_root
                && entry.global_path == global_path
                && entry.workspace_path == workspace_path
                && entry.global_stat == global_stat
                && entry.workspace_stat == workspace_stat
            {
                let workspace_layer = entry
                    .workspace_patch
                    .clone()
                    .zip(entry.workspace_path.clone());
                return Ok((entry.global_patch.clone(), global_path, workspace_layer));
            }
        }
    }

    let global_patch = match fs::read_to_string(&global_path) {
        Ok(raw) => parse_config_patch_yaml(&global_path, &raw)?,
        Err(err) if err.kind() == io::ErrorKind::NotFound => SubstrateConfigPatch::default(),
        Err(err) => return Err(anyhow!("failed to read {}: {err}", global_path.display())),
    };

    let workspace_layer = if let Some(workspace_path) = &workspace_path {
        let raw = fs::read_to_string(workspace_path)
            .with_context(|| format!("failed to read {}", workspace_path.display()))?;
        let patch = parse_config_patch_yaml(workspace_path, &raw)?;
        Some((patch, workspace_path.clone()))
    } else {
        None
    };

    if let Ok(mut guard) = cache.lock() {
        *guard = Some(ConfigPatchCacheEntry {
            workspace_root,
            global_path: global_path.clone(),
            workspace_path,
            global_stat,
            workspace_stat,
            global_patch: global_patch.clone(),
            workspace_patch: workspace_layer.as_ref().map(|(p, _)| p.clone()),
        });
    }

    Ok((global_patch, global_path, workspace_layer))
}

type ConfigPatchLayer = (SubstrateConfigPatch, PathBuf);
type LoadedConfigPatchLayers = (SubstrateConfigPatch, PathBuf, Option<ConfigPatchLayer>);

fn resolve_effective_from_layers(
    global_patch: &SubstrateConfigPatch,
    global_path: &Path,
    workspace_patch: Option<(&SubstrateConfigPatch, &Path)>,
    env_overrides: &EnvOverrides,
    cli_overrides: &CliConfigOverrides,
    explain: bool,
    inject_protected_excludes: bool,
) -> Result<(SubstrateConfig, Option<ConfigExplainV1>)> {
    let mut effective = SubstrateConfig::default();
    let mut explain_keys = if explain {
        Some(OrderedExplainKeys::default())
    } else {
        None
    };

    let workspace_enabled = workspace_patch.is_some();
    let workspace_path = workspace_patch.map(|(_, p)| p);

    // Helper for replace keys.
    #[derive(Clone, Copy)]
    enum ReplaceSource {
        CliFlag,
        OverrideEnv,
        WorkspacePatch,
        GlobalPatch,
        Default,
    }

    fn explain_source(
        source: ReplaceSource,
        global_path: &Path,
        workspace_path: Option<&Path>,
    ) -> ConfigExplainSource {
        match source {
            ReplaceSource::CliFlag => ConfigExplainSource {
                layer: "cli_flag".to_string(),
                path: None,
            },
            ReplaceSource::OverrideEnv => ConfigExplainSource {
                layer: "override_env".to_string(),
                path: None,
            },
            ReplaceSource::WorkspacePatch => ConfigExplainSource {
                layer: "workspace_patch".to_string(),
                path: workspace_path.map(|p| p.display().to_string()),
            },
            ReplaceSource::GlobalPatch => ConfigExplainSource {
                layer: "global_patch".to_string(),
                path: Some(global_path.display().to_string()),
            },
            ReplaceSource::Default => ConfigExplainSource {
                layer: "default".to_string(),
                path: None,
            },
        }
    }

    fn resolve_replace<T: Clone>(
        default: T,
        global: Option<T>,
        workspace: Option<T>,
        env: Option<T>,
        cli: Option<T>,
        workspace_enabled: bool,
    ) -> (T, ReplaceSource) {
        if let Some(v) = cli {
            return (v, ReplaceSource::CliFlag);
        }
        if workspace_enabled {
            if let Some(v) = workspace {
                return (v, ReplaceSource::WorkspacePatch);
            }
        } else if let Some(v) = env {
            // Preserve historical behavior: env overrides apply only when no workspace exists.
            return (v, ReplaceSource::OverrideEnv);
        }
        if let Some(v) = global {
            return (v, ReplaceSource::GlobalPatch);
        }
        (default, ReplaceSource::Default)
    }

    // world.enabled
    let (world_enabled, world_enabled_src) = resolve_replace(
        effective.world.enabled,
        global_patch.world.enabled,
        workspace_patch
            .map(|(p, _)| p.world.enabled)
            .unwrap_or(None),
        env_overrides.world_enabled,
        cli_overrides.world_enabled,
        workspace_enabled,
    );
    effective.world.enabled = world_enabled;
    if let Some(keys) = &mut explain_keys {
        keys.insert(
            "world.enabled".to_string(),
            ConfigExplainKey {
                merge_strategy: "replace".to_string(),
                sources: vec![explain_source(
                    world_enabled_src,
                    global_path,
                    workspace_path,
                )],
            },
        );
    }

    // world.anchor_mode
    let (anchor_mode, anchor_mode_src) = resolve_replace(
        effective.world.anchor_mode,
        global_patch.world.anchor_mode,
        workspace_patch
            .map(|(p, _)| p.world.anchor_mode)
            .unwrap_or(None),
        env_overrides.anchor_mode,
        cli_overrides.anchor_mode,
        workspace_enabled,
    );
    effective.world.anchor_mode = anchor_mode;
    if let Some(keys) = &mut explain_keys {
        keys.insert(
            "world.anchor_mode".to_string(),
            ConfigExplainKey {
                merge_strategy: "replace".to_string(),
                sources: vec![explain_source(anchor_mode_src, global_path, workspace_path)],
            },
        );
    }

    // world.anchor_path
    let (anchor_path, anchor_path_src) = resolve_replace(
        effective.world.anchor_path.clone(),
        global_patch.world.anchor_path.clone(),
        workspace_patch
            .map(|(p, _)| p.world.anchor_path.clone())
            .unwrap_or(None),
        env_overrides.anchor_path.clone(),
        cli_overrides.anchor_path.clone(),
        workspace_enabled,
    );
    effective.world.anchor_path = anchor_path;
    if let Some(keys) = &mut explain_keys {
        keys.insert(
            "world.anchor_path".to_string(),
            ConfigExplainKey {
                merge_strategy: "replace".to_string(),
                sources: vec![explain_source(anchor_path_src, global_path, workspace_path)],
            },
        );
    }

    // world.caged
    let (caged, caged_src) = resolve_replace(
        effective.world.caged,
        global_patch.world.caged,
        workspace_patch.map(|(p, _)| p.world.caged).unwrap_or(None),
        env_overrides.caged,
        cli_overrides.caged,
        workspace_enabled,
    );
    effective.world.caged = caged;
    if let Some(keys) = &mut explain_keys {
        keys.insert(
            "world.caged".to_string(),
            ConfigExplainKey {
                merge_strategy: "replace".to_string(),
                sources: vec![explain_source(caged_src, global_path, workspace_path)],
            },
        );
    }

    // world.net.filter
    let (net_filter, net_filter_src) = resolve_replace(
        effective.world.net.filter,
        global_patch.world.net.filter,
        workspace_patch
            .map(|(p, _)| p.world.net.filter)
            .unwrap_or(None),
        env_overrides.world_net_filter,
        None,
        workspace_enabled,
    );
    effective.world.net.filter = net_filter;
    if let Some(keys) = &mut explain_keys {
        keys.insert(
            "world.net.filter".to_string(),
            ConfigExplainKey {
                merge_strategy: "replace".to_string(),
                sources: vec![explain_source(net_filter_src, global_path, workspace_path)],
            },
        );
    }

    // world.env.inherit_from_host
    let (inherit_from_host, inherit_from_host_src) = resolve_replace(
        effective.world.env.inherit_from_host,
        global_patch.world.env.inherit_from_host,
        workspace_patch
            .map(|(p, _)| p.world.env.inherit_from_host)
            .unwrap_or(None),
        None,
        None,
        workspace_enabled,
    );
    effective.world.env.inherit_from_host = inherit_from_host;
    if let Some(keys) = &mut explain_keys {
        keys.insert(
            "world.env.inherit_from_host".to_string(),
            ConfigExplainKey {
                merge_strategy: "replace".to_string(),
                sources: vec![explain_source(
                    inherit_from_host_src,
                    global_path,
                    workspace_path,
                )],
            },
        );
    }

    // repl.exit_cwd
    let (exit_cwd, exit_cwd_src) = resolve_replace(
        effective.repl.exit_cwd,
        global_patch.repl.exit_cwd,
        workspace_patch
            .map(|(p, _)| p.repl.exit_cwd)
            .unwrap_or(None),
        None,
        None,
        workspace_enabled,
    );
    effective.repl.exit_cwd = exit_cwd;
    if let Some(keys) = &mut explain_keys {
        keys.insert(
            "repl.exit_cwd".to_string(),
            ConfigExplainKey {
                merge_strategy: "replace".to_string(),
                sources: vec![explain_source(exit_cwd_src, global_path, workspace_path)],
            },
        );
    }

    // policy.mode (from config.yaml/workspace.yaml, not policy.yaml)
    let (policy_mode, policy_mode_src) = resolve_replace(
        effective.policy.mode,
        global_patch.policy.mode,
        workspace_patch.map(|(p, _)| p.policy.mode).unwrap_or(None),
        env_overrides.policy_mode,
        None,
        workspace_enabled,
    );
    effective.policy.mode = policy_mode;
    if let Some(keys) = &mut explain_keys {
        keys.insert(
            "policy.mode".to_string(),
            ConfigExplainKey {
                merge_strategy: "replace".to_string(),
                sources: vec![explain_source(policy_mode_src, global_path, workspace_path)],
            },
        );
    }

    // sync.auto_sync
    let (auto_sync, auto_sync_src) = resolve_replace(
        effective.sync.auto_sync,
        global_patch.sync.auto_sync,
        workspace_patch
            .map(|(p, _)| p.sync.auto_sync)
            .unwrap_or(None),
        env_overrides.sync_auto_sync,
        None,
        workspace_enabled,
    );
    effective.sync.auto_sync = auto_sync;
    if let Some(keys) = &mut explain_keys {
        keys.insert(
            "sync.auto_sync".to_string(),
            ConfigExplainKey {
                merge_strategy: "replace".to_string(),
                sources: vec![explain_source(auto_sync_src, global_path, workspace_path)],
            },
        );
    }

    // sync.direction
    let (sync_direction, sync_direction_src) = resolve_replace(
        effective.sync.direction,
        global_patch.sync.direction,
        workspace_patch
            .map(|(p, _)| p.sync.direction)
            .unwrap_or(None),
        env_overrides.sync_direction,
        None,
        workspace_enabled,
    );
    effective.sync.direction = sync_direction;
    if let Some(keys) = &mut explain_keys {
        keys.insert(
            "sync.direction".to_string(),
            ConfigExplainKey {
                merge_strategy: "replace".to_string(),
                sources: vec![explain_source(
                    sync_direction_src,
                    global_path,
                    workspace_path,
                )],
            },
        );
    }

    // sync.conflict_policy
    let (conflict_policy, conflict_policy_src) = resolve_replace(
        effective.sync.conflict_policy,
        global_patch.sync.conflict_policy,
        workspace_patch
            .map(|(p, _)| p.sync.conflict_policy)
            .unwrap_or(None),
        env_overrides.sync_conflict_policy,
        None,
        workspace_enabled,
    );
    effective.sync.conflict_policy = conflict_policy;
    if let Some(keys) = &mut explain_keys {
        keys.insert(
            "sync.conflict_policy".to_string(),
            ConfigExplainKey {
                merge_strategy: "replace".to_string(),
                sources: vec![explain_source(
                    conflict_policy_src,
                    global_path,
                    workspace_path,
                )],
            },
        );
    }

    // sync.exclude (replace semantics; protected excludes are injected later)
    let (exclude, exclude_src) = resolve_replace(
        effective.sync.exclude.clone(),
        global_patch.sync.exclude.clone(),
        workspace_patch
            .map(|(p, _)| p.sync.exclude.clone())
            .unwrap_or(None),
        env_overrides.sync_exclude.clone(),
        None,
        workspace_enabled,
    );
    effective.sync.exclude = exclude;
    if let Some(keys) = &mut explain_keys {
        keys.insert(
            "sync.exclude".to_string(),
            ConfigExplainKey {
                merge_strategy: "replace".to_string(),
                sources: vec![explain_source(exclude_src, global_path, workspace_path)],
            },
        );
    }

    // world.deps.enabled (concat_dedupe_ordered_set)
    let default_list = effective.world.deps.enabled.clone();
    let mut enabled_sources = Vec::new();
    let mut layers = Vec::new();
    let has_any_patch = global_patch.world.deps.enabled.is_some()
        || (workspace_enabled
            && workspace_patch
                .and_then(|(p, _)| p.world.deps.enabled.as_ref())
                .is_some());
    if !default_list.is_empty() || !has_any_patch {
        enabled_sources.push(ConfigExplainSource {
            layer: "default".to_string(),
            path: None,
        });
        layers.push(default_list);
    }
    if let Some(list) = &global_patch.world.deps.enabled {
        enabled_sources.push(ConfigExplainSource {
            layer: "global_patch".to_string(),
            path: Some(global_path.display().to_string()),
        });
        layers.push(list.clone());
    }
    if workspace_enabled {
        if let Some(list) = workspace_patch.and_then(|(p, _)| p.world.deps.enabled.as_ref()) {
            enabled_sources.push(ConfigExplainSource {
                layer: "workspace_patch".to_string(),
                path: workspace_path.map(|p| p.display().to_string()),
            });
            layers.push(list.clone());
        }
    }
    effective.world.deps.enabled = concat_dedupe_ordered_set(&layers);
    if let Some(keys) = &mut explain_keys {
        keys.insert(
            "world.deps.enabled".to_string(),
            ConfigExplainKey {
                merge_strategy: "concat_dedupe_ordered_set".to_string(),
                sources: enabled_sources,
            },
        );
    }

    // world.deps.inventory_mode (replace)
    let (inv_mode, inv_mode_src) = resolve_replace(
        effective.world.deps.inventory_mode,
        global_patch.world.deps.inventory_mode,
        workspace_patch
            .map(|(p, _)| p.world.deps.inventory_mode)
            .unwrap_or(None),
        None,
        None,
        workspace_enabled,
    );
    effective.world.deps.inventory_mode = inv_mode;
    if let Some(keys) = &mut explain_keys {
        keys.insert(
            "world.deps.inventory_mode".to_string(),
            ConfigExplainKey {
                merge_strategy: "replace".to_string(),
                sources: vec![explain_source(inv_mode_src, global_path, workspace_path)],
            },
        );
    }

    // world.deps.builtins (replace)
    let (builtins, builtins_src) = resolve_replace(
        effective.world.deps.builtins,
        global_patch.world.deps.builtins,
        workspace_patch
            .map(|(p, _)| p.world.deps.builtins)
            .unwrap_or(None),
        None,
        None,
        workspace_enabled,
    );
    effective.world.deps.builtins = builtins;
    if let Some(keys) = &mut explain_keys {
        keys.insert(
            "world.deps.builtins".to_string(),
            ConfigExplainKey {
                merge_strategy: "replace".to_string(),
                sources: vec![explain_source(builtins_src, global_path, workspace_path)],
            },
        );
    }

    if inject_protected_excludes {
        apply_protected_excludes(&mut effective.sync.exclude);
    }

    validate_config(&effective)?;

    let explain = explain_keys.map(|keys| ConfigExplainV1 {
        kind: "substrate.config.explain.v1".to_string(),
        keys,
    });
    Ok((effective, explain))
}

fn concat_dedupe_ordered_set(layers: &[Vec<String>]) -> Vec<String> {
    let mut out = Vec::new();
    for layer in layers {
        for item in layer {
            if !out.iter().any(|existing| existing == item) {
                out.push(item.clone());
            }
        }
    }
    out
}

#[derive(Debug, Default)]
struct EnvOverrides {
    world_enabled: Option<bool>,
    anchor_mode: Option<WorldRootMode>,
    anchor_path: Option<String>,
    caged: Option<bool>,
    world_net_filter: Option<bool>,
    policy_mode: Option<PolicyMode>,
    sync_auto_sync: Option<bool>,
    sync_direction: Option<SyncDirection>,
    sync_conflict_policy: Option<SyncConflictPolicy>,
    sync_exclude: Option<Vec<String>>,
}

fn parse_env_overrides() -> Result<EnvOverrides> {
    let mut overrides = EnvOverrides::default();

    if let Ok(world) = env::var("SUBSTRATE_OVERRIDE_WORLD") {
        let normalized = world.trim().to_ascii_lowercase();
        if !normalized.is_empty() {
            overrides.world_enabled = Some(match normalized.as_str() {
                "enabled" => true,
                "disabled" => false,
                _ => {
                    return Err(user_error(format!(
                        "SUBSTRATE_OVERRIDE_WORLD must be 'enabled' or 'disabled' (found '{}')",
                        world
                    )));
                }
            });
        }
    }

    if let Ok(mode) = env::var("SUBSTRATE_OVERRIDE_ANCHOR_MODE") {
        let trimmed = mode.trim();
        if !trimmed.is_empty() {
            overrides.anchor_mode = Some(WorldRootMode::parse(trimmed).ok_or_else(|| {
                user_error(format!(
                    "SUBSTRATE_OVERRIDE_ANCHOR_MODE must be one of workspace, follow-cwd, or custom (found '{}')",
                    mode
                ))
            })?);
        }
    }

    if let Ok(path) = env::var("SUBSTRATE_OVERRIDE_ANCHOR_PATH") {
        overrides.anchor_path = Some(path);
    }

    if let Ok(raw) = env::var("SUBSTRATE_OVERRIDE_CAGED") {
        let trimmed = raw.trim();
        if !trimmed.is_empty() {
            overrides.caged = Some(parse_bool_flag(trimmed).ok_or_else(|| {
                user_error(format!(
                    "SUBSTRATE_OVERRIDE_CAGED must be a boolean (true|false|1|0|yes|no|on|off) (found '{}')",
                    raw
                ))
            })?);
        }
    }

    if let Ok(raw) = env::var("SUBSTRATE_OVERRIDE_WORLD_NET_FILTER") {
        let trimmed = raw.trim();
        if !trimmed.is_empty() {
            overrides.world_net_filter = Some(parse_bool_flag(trimmed).ok_or_else(|| {
                user_error(format!(
                    "SUBSTRATE_OVERRIDE_WORLD_NET_FILTER must be a boolean (true|false|1|0|yes|no|on|off) (found '{}')",
                    raw
                ))
            })?);
        }
    }

    if let Ok(raw) = env::var("SUBSTRATE_OVERRIDE_POLICY_MODE") {
        let trimmed = raw.trim();
        if !trimmed.is_empty() {
            overrides.policy_mode = Some(PolicyMode::parse_insensitive(trimmed).ok_or_else(|| {
                user_error(format!(
                    "SUBSTRATE_OVERRIDE_POLICY_MODE must be one of disabled, observe, or enforce (found '{}')",
                    raw
                ))
            })?);
        }
    }

    if let Ok(raw) = env::var("SUBSTRATE_OVERRIDE_SYNC_AUTO_SYNC") {
        let trimmed = raw.trim();
        if !trimmed.is_empty() {
            overrides.sync_auto_sync = Some(parse_bool_flag(trimmed).ok_or_else(|| {
                user_error(format!(
                    "SUBSTRATE_OVERRIDE_SYNC_AUTO_SYNC must be a boolean (true|false|1|0|yes|no|on|off) (found '{}')",
                    raw
                ))
            })?);
        }
    }

    if let Ok(raw) = env::var("SUBSTRATE_OVERRIDE_SYNC_DIRECTION") {
        let trimmed = raw.trim();
        if !trimmed.is_empty() {
            overrides.sync_direction = Some(SyncDirection::parse_insensitive(trimmed).ok_or_else(|| {
                user_error(format!(
                    "SUBSTRATE_OVERRIDE_SYNC_DIRECTION must be one of from_world, from_host, or both (found '{}')",
                    raw
                ))
            })?);
        }
    }

    if let Ok(raw) = env::var("SUBSTRATE_OVERRIDE_SYNC_CONFLICT_POLICY") {
        let trimmed = raw.trim();
        if !trimmed.is_empty() {
            overrides.sync_conflict_policy =
                Some(SyncConflictPolicy::parse_insensitive(trimmed).ok_or_else(|| {
                    user_error(format!(
                        "SUBSTRATE_OVERRIDE_SYNC_CONFLICT_POLICY must be one of prefer_host, prefer_world, or abort (found '{}')",
                        raw
                    ))
                })?);
        }
    }

    if let Ok(raw) = env::var("SUBSTRATE_OVERRIDE_SYNC_EXCLUDE") {
        let items = raw
            .split(',')
            .map(|item| item.trim())
            .filter(|item| !item.is_empty())
            .map(|item| item.to_string())
            .collect::<Vec<_>>();
        overrides.sync_exclude = Some(items);
    }

    Ok(overrides)
}

pub(crate) fn apply_protected_excludes(excludes: &mut Vec<String>) {
    excludes.retain(|item| !PROTECTED_EXCLUDES.contains(&item.as_str()));
    for (idx, item) in PROTECTED_EXCLUDES.iter().enumerate() {
        excludes.insert(idx, item.to_string());
    }
}

pub(crate) fn parse_updates(raw_updates: &[String]) -> Result<Vec<ConfigUpdate>> {
    raw_updates.iter().map(|raw| parse_update(raw)).collect()
}

fn parse_update(raw: &str) -> Result<ConfigUpdate> {
    let (key, op, value) = if let Some((k, v)) = raw.split_once("+=") {
        (k, UpdateOp::Append, v)
    } else if let Some((k, v)) = raw.split_once("-=") {
        (k, UpdateOp::Remove, v)
    } else if let Some((k, v)) = raw.split_once('=') {
        (k, UpdateOp::Set, v)
    } else {
        return Err(user_error(format!(
            "invalid update '{}'; expected key=value, key+=value, or key-=value",
            raw
        )));
    };

    let key = key.trim();
    if key.is_empty() {
        return Err(user_error(format!("invalid update '{}'; missing key", raw)));
    }

    Ok(ConfigUpdate {
        key: key.to_string(),
        op,
        value: value.to_string(),
    })
}

pub(crate) fn apply_updates_to_patch(
    patch: &mut SubstrateConfigPatch,
    updates: &[ConfigUpdate],
) -> Result<bool> {
    let mut changed = false;
    for update in updates {
        changed |= apply_update_to_patch(patch, update)?;
    }
    Ok(changed)
}

pub(crate) fn reset_patch_keys(patch: &mut SubstrateConfigPatch, keys: &[String]) -> Result<bool> {
    let mut changed = false;
    for key in keys {
        changed |= reset_patch_key(patch, key)?;
    }
    Ok(changed)
}

fn validate_config(cfg: &SubstrateConfig) -> Result<()> {
    if cfg.world.anchor_mode == WorldRootMode::Custom && cfg.world.anchor_path.trim().is_empty() {
        return Err(user_error(
            "anchor_mode=custom requires world.anchor_path to be non-empty",
        ));
    }
    Ok(())
}

fn apply_update_to_patch(patch: &mut SubstrateConfigPatch, update: &ConfigUpdate) -> Result<bool> {
    match update.key.as_str() {
        "world.enabled" => apply_bool_opt(&mut patch.world.enabled, &update.op, &update.value),
        "world.anchor_mode" => {
            apply_enum_anchor_mode_opt(&mut patch.world.anchor_mode, &update.op, &update.value)
        }
        "world.anchor_path" => {
            apply_string_opt(&mut patch.world.anchor_path, &update.op, &update.value)
        }
        "world.caged" => apply_bool_opt(&mut patch.world.caged, &update.op, &update.value),
        "world.net.filter" => {
            apply_bool_opt(&mut patch.world.net.filter, &update.op, &update.value)
        }
        "world.env.inherit_from_host" => apply_bool_opt(
            &mut patch.world.env.inherit_from_host,
            &update.op,
            &update.value,
        ),

        "world.deps.enabled" => apply_string_list_opt(&mut patch.world.deps.enabled, update),
        "world.deps.inventory_mode" => apply_enum_inventory_mode_opt(
            &mut patch.world.deps.inventory_mode,
            &update.op,
            &update.value,
        ),
        "world.deps.builtins" => {
            apply_enum_builtins_opt(&mut patch.world.deps.builtins, &update.op, &update.value)
        }

        "policy.mode" => {
            apply_enum_policy_mode_opt(&mut patch.policy.mode, &update.op, &update.value)
        }

        "sync.auto_sync" => apply_bool_opt(&mut patch.sync.auto_sync, &update.op, &update.value),
        "sync.direction" => {
            apply_enum_sync_direction_opt(&mut patch.sync.direction, &update.op, &update.value)
        }
        "sync.conflict_policy" => apply_enum_sync_conflict_policy_opt(
            &mut patch.sync.conflict_policy,
            &update.op,
            &update.value,
        ),
        "sync.exclude" => apply_string_list_opt(&mut patch.sync.exclude, update),

        "repl.exit_cwd" => {
            apply_enum_repl_exit_cwd_opt(&mut patch.repl.exit_cwd, &update.op, &update.value)
        }

        _ => Err(user_error(format!(
            "unsupported config key '{}'",
            update.key
        ))),
    }
}

fn reset_patch_key(patch: &mut SubstrateConfigPatch, key: &str) -> Result<bool> {
    match key {
        "world.enabled" => reset_opt(&mut patch.world.enabled),
        "world.anchor_mode" => reset_opt(&mut patch.world.anchor_mode),
        "world.anchor_path" => reset_opt(&mut patch.world.anchor_path),
        "world.caged" => reset_opt(&mut patch.world.caged),
        "world.net.filter" => reset_opt(&mut patch.world.net.filter),
        "world.env.inherit_from_host" => reset_opt(&mut patch.world.env.inherit_from_host),

        "world.deps.enabled" => reset_opt(&mut patch.world.deps.enabled),
        "world.deps.inventory_mode" => reset_opt(&mut patch.world.deps.inventory_mode),
        "world.deps.builtins" => reset_opt(&mut patch.world.deps.builtins),

        "policy.mode" => reset_opt(&mut patch.policy.mode),

        "sync.auto_sync" => reset_opt(&mut patch.sync.auto_sync),
        "sync.direction" => reset_opt(&mut patch.sync.direction),
        "sync.conflict_policy" => reset_opt(&mut patch.sync.conflict_policy),
        "sync.exclude" => reset_opt(&mut patch.sync.exclude),

        "repl.exit_cwd" => reset_opt(&mut patch.repl.exit_cwd),

        _ => Err(user_error(format!("unsupported config key '{}'", key))),
    }
}

fn reset_opt<T>(target: &mut Option<T>) -> Result<bool> {
    let changed = target.is_some();
    *target = None;
    Ok(changed)
}

fn apply_bool_opt(target: &mut Option<bool>, op: &UpdateOp, raw: &str) -> Result<bool> {
    if *op != UpdateOp::Set {
        return Err(user_error("unsupported operator for boolean key"));
    }
    let next = parse_bool_flag(raw).ok_or_else(|| {
        user_error(format!(
            "invalid boolean '{}'; expected true|false|1|0|yes|no|on|off",
            raw
        ))
    })?;
    let changed = *target != Some(next);
    *target = Some(next);
    Ok(changed)
}

fn apply_enum_anchor_mode_opt(
    target: &mut Option<WorldRootMode>,
    op: &UpdateOp,
    raw: &str,
) -> Result<bool> {
    if *op != UpdateOp::Set {
        return Err(user_error("unsupported operator for enum key"));
    }
    let next = WorldRootMode::parse(raw).ok_or_else(|| {
        user_error(format!(
            "invalid world.anchor_mode '{}'; expected workspace, follow-cwd, or custom",
            raw
        ))
    })?;
    let changed = *target != Some(next);
    *target = Some(next);
    Ok(changed)
}

fn apply_enum_policy_mode_opt(
    target: &mut Option<PolicyMode>,
    op: &UpdateOp,
    raw: &str,
) -> Result<bool> {
    if *op != UpdateOp::Set {
        return Err(user_error("unsupported operator for enum key"));
    }
    let next = PolicyMode::parse_insensitive(raw).ok_or_else(|| {
        user_error(format!(
            "invalid policy.mode '{}'; expected disabled, observe, or enforce",
            raw
        ))
    })?;
    let changed = *target != Some(next);
    *target = Some(next);
    Ok(changed)
}

fn apply_enum_repl_exit_cwd_opt(
    target: &mut Option<ReplExitCwdMode>,
    op: &UpdateOp,
    raw: &str,
) -> Result<bool> {
    if *op != UpdateOp::Set {
        return Err(user_error("unsupported operator for enum key"));
    }
    let next = ReplExitCwdMode::parse_insensitive(raw).ok_or_else(|| {
        user_error(format!(
            "invalid repl.exit_cwd '{}'; expected entered or last_world",
            raw
        ))
    })?;
    let changed = *target != Some(next);
    *target = Some(next);
    Ok(changed)
}

fn apply_enum_sync_direction_opt(
    target: &mut Option<SyncDirection>,
    op: &UpdateOp,
    raw: &str,
) -> Result<bool> {
    if *op != UpdateOp::Set {
        return Err(user_error("unsupported operator for enum key"));
    }
    let next = SyncDirection::parse_insensitive(raw).ok_or_else(|| {
        user_error(format!(
            "invalid sync.direction '{}'; expected from_world, from_host, or both",
            raw
        ))
    })?;
    let changed = *target != Some(next);
    *target = Some(next);
    Ok(changed)
}

fn apply_enum_sync_conflict_policy_opt(
    target: &mut Option<SyncConflictPolicy>,
    op: &UpdateOp,
    raw: &str,
) -> Result<bool> {
    if *op != UpdateOp::Set {
        return Err(user_error("unsupported operator for enum key"));
    }
    let next = SyncConflictPolicy::parse_insensitive(raw).ok_or_else(|| {
        user_error(format!(
            "invalid sync.conflict_policy '{}'; expected prefer_host, prefer_world, or abort",
            raw
        ))
    })?;
    let changed = *target != Some(next);
    *target = Some(next);
    Ok(changed)
}

fn apply_enum_inventory_mode_opt(
    target: &mut Option<WorldDepsInventoryMode>,
    op: &UpdateOp,
    raw: &str,
) -> Result<bool> {
    if *op != UpdateOp::Set {
        return Err(user_error("unsupported operator for enum key"));
    }
    let next = WorldDepsInventoryMode::parse_insensitive(raw).ok_or_else(|| {
        user_error(format!(
            "invalid world.deps.inventory_mode '{}'; expected merged or workspace_only",
            raw
        ))
    })?;
    let changed = *target != Some(next);
    *target = Some(next);
    Ok(changed)
}

fn apply_enum_builtins_opt(
    target: &mut Option<WorldDepsBuiltinsMode>,
    op: &UpdateOp,
    raw: &str,
) -> Result<bool> {
    if *op != UpdateOp::Set {
        return Err(user_error("unsupported operator for enum key"));
    }
    let next = WorldDepsBuiltinsMode::parse_insensitive(raw).ok_or_else(|| {
        user_error(format!(
            "invalid world.deps.builtins '{}'; expected enabled or disabled",
            raw
        ))
    })?;
    let changed = *target != Some(next);
    *target = Some(next);
    Ok(changed)
}

fn apply_string_opt(target: &mut Option<String>, op: &UpdateOp, raw: &str) -> Result<bool> {
    if *op != UpdateOp::Set {
        return Err(user_error("unsupported operator for string key"));
    }
    let changed = target.as_deref() != Some(raw);
    *target = Some(raw.to_string());
    Ok(changed)
}

fn apply_string_list_opt(target: &mut Option<Vec<String>>, update: &ConfigUpdate) -> Result<bool> {
    match update.op {
        UpdateOp::Set => {
            let mut parsed: Vec<String> = serde_yaml::from_str(&update.value).map_err(|_| {
                user_error(format!(
                    "{} with '=' must be a YAML list literal (e.g., [] or [\"a\",\"b\"]); got '{}'",
                    update.key, update.value
                ))
            })?;
            if update.key == "world.deps.enabled" {
                dedupe_ordered_set_in_place(&mut parsed);
            }
            let changed = target.as_ref() != Some(&parsed);
            *target = Some(parsed);
            Ok(changed)
        }
        UpdateOp::Append => {
            let list = target.get_or_insert_with(Vec::new);
            if list.iter().any(|item| item == &update.value) {
                return Ok(false);
            }
            list.push(update.value.clone());
            if update.key == "world.deps.enabled" {
                dedupe_ordered_set_in_place(list);
            }
            Ok(true)
        }
        UpdateOp::Remove => {
            let Some(list) = target.as_mut() else {
                return Ok(false);
            };
            let before = list.len();
            list.retain(|item| item != &update.value);
            let changed = before != list.len();
            if changed && update.key == "world.deps.enabled" {
                dedupe_ordered_set_in_place(list);
            }
            Ok(changed)
        }
    }
}

fn dedupe_ordered_set_in_place(items: &mut Vec<String>) {
    let mut out: Vec<String> = Vec::with_capacity(items.len());
    for item in items.drain(..) {
        if !out.iter().any(|existing| existing == &item) {
            out.push(item);
        }
    }
    *items = out;
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;
    use std::ffi::OsString;
    use std::fs;
    use tempfile::TempDir;

    struct EnvGuard {
        key: &'static str,
        prev: Option<OsString>,
    }

    impl EnvGuard {
        fn set(key: &'static str, value: &std::path::Path) -> Self {
            let prev = std::env::var_os(key);
            std::env::set_var(key, value);
            Self { key, prev }
        }

        fn set_str(key: &'static str, value: &str) -> Self {
            let prev = std::env::var_os(key);
            std::env::set_var(key, value);
            Self { key, prev }
        }
    }

    impl Drop for EnvGuard {
        fn drop(&mut self) {
            match self.prev.take() {
                Some(v) => std::env::set_var(self.key, v),
                None => std::env::remove_var(self.key),
            }
        }
    }

    fn write_file(path: &Path, body: &str) {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        fs::write(path, body).unwrap();
    }

    fn canonicalize_for_compare(path: &Path) -> PathBuf {
        path.canonicalize().unwrap_or_else(|_| path.to_path_buf())
    }

    fn resolve_world_disable_message(
        cwd: &Path,
        overrides: CliConfigOverrides,
    ) -> Option<&'static str> {
        let (effective, explain) =
            resolve_effective_config_with_explain(cwd, &overrides, true).unwrap();
        world_disable_attribution_message(
            effective.world.enabled,
            explain
                .as_ref()
                .and_then(|explain| explain.keys.get("world.enabled")),
        )
    }

    fn explain_key(layer: &str, path: Option<&str>) -> ConfigExplainKey {
        ConfigExplainKey {
            merge_strategy: "replace".to_string(),
            sources: vec![ConfigExplainSource {
                layer: layer.to_string(),
                path: path.map(|value| value.to_string()),
            }],
        }
    }

    fn assert_same_path(actual: Option<&String>, expected: &Path) {
        let actual = actual
            .map(PathBuf::from)
            .unwrap_or_else(|| panic!("expected path, got None"));
        let actual = canonicalize_for_compare(&actual);
        let expected = canonicalize_for_compare(expected);
        assert_eq!(actual, expected);
    }

    #[test]
    #[serial]
    fn test_phase_a_concat_dedupe_and_replace_provenance() {
        let tmp = TempDir::new().unwrap();
        let substrate_home = tmp.path().join(".substrate");
        fs::create_dir_all(&substrate_home).unwrap();
        let _guard = EnvGuard::set("SUBSTRATE_HOME", &substrate_home);

        let global_path = global_config_path().unwrap();
        write_file(
            &global_path,
            r#"
world:
  deps:
    enabled: ["a", "b"]
    inventory_mode: merged
"#,
        );

        let workspace_root = tmp.path().join("ws");
        let workspace_yaml = crate::execution::workspace::workspace_marker_path(&workspace_root);
        write_file(
            &workspace_yaml,
            r#"
world:
  deps:
    enabled: ["b", "c"]
    inventory_mode: workspace_only
"#,
        );

        let (effective, explain) = resolve_effective_config_with_explain(
            &workspace_root,
            &CliConfigOverrides::default(),
            true,
        )
        .unwrap();

        assert_eq!(effective.world.deps.enabled, vec!["a", "b", "c"]);
        assert_eq!(
            effective.world.deps.inventory_mode,
            WorldDepsInventoryMode::WorkspaceOnly
        );

        let explain = explain.unwrap();
        assert_eq!(explain.kind, "substrate.config.explain.v1");

        let enabled = explain.keys.get("world.deps.enabled").unwrap();
        assert_eq!(enabled.merge_strategy, "concat_dedupe_ordered_set");
        assert_eq!(enabled.sources.len(), 2);
        let expected_global_path = global_path.display().to_string();
        assert_eq!(enabled.sources[0].layer, "global_patch");
        assert_eq!(
            enabled.sources[0].path.as_deref(),
            Some(expected_global_path.as_str())
        );
        assert_eq!(enabled.sources[1].layer, "workspace_patch");
        assert_same_path(enabled.sources[1].path.as_ref(), &workspace_yaml);

        let inv = explain.keys.get("world.deps.inventory_mode").unwrap();
        assert_eq!(inv.merge_strategy, "replace");
        assert_eq!(inv.sources.len(), 1);
        assert_eq!(inv.sources[0].layer, "workspace_patch");
        assert_same_path(inv.sources[0].path.as_ref(), &workspace_yaml);
    }

    #[test]
    #[serial]
    fn test_phase_a_explicit_empty_list_counts_as_source() {
        let tmp = TempDir::new().unwrap();
        let substrate_home = tmp.path().join(".substrate");
        fs::create_dir_all(&substrate_home).unwrap();
        let _guard = EnvGuard::set("SUBSTRATE_HOME", &substrate_home);

        let global_path = global_config_path().unwrap();
        write_file(
            &global_path,
            r#"
world:
  deps:
    enabled: ["a"]
"#,
        );

        let workspace_root = tmp.path().join("ws");
        let workspace_yaml = crate::execution::workspace::workspace_marker_path(&workspace_root);
        write_file(
            &workspace_yaml,
            r#"
world:
  deps:
    enabled: []
"#,
        );

        let (effective, explain) = resolve_effective_config_with_explain(
            &workspace_root,
            &CliConfigOverrides::default(),
            true,
        )
        .unwrap();

        assert_eq!(effective.world.deps.enabled, vec!["a"]);

        let explain = explain.unwrap();
        let enabled = explain.keys.get("world.deps.enabled").unwrap();
        assert_eq!(enabled.sources.len(), 2);
        assert_eq!(enabled.sources[0].layer, "global_patch");
        assert_eq!(enabled.sources[1].layer, "workspace_patch");
    }

    #[test]
    #[serial]
    fn test_phase_a_workspace_disabled_ignores_workspace_patch() {
        let tmp = TempDir::new().unwrap();
        let substrate_home = tmp.path().join(".substrate");
        fs::create_dir_all(&substrate_home).unwrap();
        let _guard = EnvGuard::set("SUBSTRATE_HOME", &substrate_home);

        let global_path = global_config_path().unwrap();
        write_file(
            &global_path,
            r#"
world:
  deps:
    enabled: ["a"]
    inventory_mode: merged
"#,
        );

        let workspace_root = tmp.path().join("ws");
        let workspace_yaml = crate::execution::workspace::workspace_marker_path(&workspace_root);
        write_file(
            &workspace_yaml,
            r#"
world:
  deps:
    enabled: ["b"]
    inventory_mode: workspace_only
"#,
        );
        let disabled_marker =
            crate::execution::workspace::workspace_disabled_marker_path(&workspace_root);
        write_file(&disabled_marker, "disabled\n");

        let (effective, explain) = resolve_effective_config_with_explain(
            &workspace_root,
            &CliConfigOverrides::default(),
            true,
        )
        .unwrap();

        assert_eq!(effective.world.deps.enabled, vec!["a"]);
        assert_eq!(
            effective.world.deps.inventory_mode,
            WorldDepsInventoryMode::Merged
        );

        let explain = explain.unwrap();
        let enabled = explain.keys.get("world.deps.enabled").unwrap();
        assert_eq!(enabled.sources.len(), 1);
        assert_eq!(enabled.sources[0].layer, "global_patch");

        let inv = explain.keys.get("world.deps.inventory_mode").unwrap();
        assert_eq!(inv.sources.len(), 1);
        assert_eq!(inv.sources[0].layer, "global_patch");
        let expected_global_path = global_path.display().to_string();
        assert_eq!(
            inv.sources[0].path.as_deref(),
            Some(expected_global_path.as_str())
        );
    }

    #[test]
    #[serial]
    fn test_diagnostics_world_enabled_prefers_cli_over_workspace_and_env() {
        let tmp = TempDir::new().unwrap();
        let substrate_home = tmp.path().join(".substrate");
        fs::create_dir_all(&substrate_home).unwrap();
        let _guard = EnvGuard::set("SUBSTRATE_HOME", &substrate_home);

        let global_path = global_config_path().unwrap();
        write_file(
            &global_path,
            r#"
world:
  enabled: false
"#,
        );

        let workspace_root = tmp.path().join("ws");
        let workspace_yaml = crate::execution::workspace::workspace_marker_path(&workspace_root);
        write_file(
            &workspace_yaml,
            r#"
world:
  enabled: true
"#,
        );

        let _env_guard = EnvGuard::set_str("SUBSTRATE_OVERRIDE_WORLD", "disabled");

        let enabled = resolve_effective_config(
            &workspace_root,
            &CliConfigOverrides {
                world_enabled: Some(true),
                ..Default::default()
            },
        )
        .unwrap()
        .world
        .enabled;
        assert!(enabled, "CLI override should win over workspace and env");

        let disabled = resolve_effective_config(
            &workspace_root,
            &CliConfigOverrides {
                world_enabled: Some(false),
                ..Default::default()
            },
        )
        .unwrap()
        .world
        .enabled;
        assert!(
            !disabled,
            "CLI disable override should win over workspace and env"
        );

        let resolved = resolve_effective_config(&workspace_root, &CliConfigOverrides::default())
            .unwrap()
            .world
            .enabled;
        assert!(
            resolved,
            "workspace config should win over SUBSTRATE_OVERRIDE_WORLD when a workspace is enabled"
        );
    }

    #[test]
    #[serial]
    fn test_diagnostics_world_enabled_uses_env_override_without_workspace() {
        let tmp = TempDir::new().unwrap();
        let substrate_home = tmp.path().join(".substrate");
        fs::create_dir_all(&substrate_home).unwrap();
        let _guard = EnvGuard::set("SUBSTRATE_HOME", &substrate_home);

        let global_path = global_config_path().unwrap();
        write_file(
            &global_path,
            r#"
world:
  enabled: true
"#,
        );

        let workspace_root = tmp.path().join("ws");
        fs::create_dir_all(&workspace_root).unwrap();
        let _env_guard = EnvGuard::set_str("SUBSTRATE_OVERRIDE_WORLD", "disabled");

        let resolved = resolve_effective_config(&workspace_root, &CliConfigOverrides::default())
            .unwrap()
            .world
            .enabled;
        assert!(
            !resolved,
            "SUBSTRATE_OVERRIDE_WORLD should apply when no enabled workspace exists"
        );
    }

    #[test]
    #[serial]
    fn test_world_disable_attribution_maps_cli_flag() {
        let tmp = TempDir::new().unwrap();
        let substrate_home = tmp.path().join(".substrate");
        fs::create_dir_all(&substrate_home).unwrap();
        let _guard = EnvGuard::set("SUBSTRATE_HOME", &substrate_home);

        let global_path = global_config_path().unwrap();
        write_file(
            &global_path,
            r#"
world:
  enabled: true
"#,
        );

        let workspace_root = tmp.path().join("ws");
        let workspace_yaml = crate::execution::workspace::workspace_marker_path(&workspace_root);
        write_file(
            &workspace_yaml,
            r#"
world:
  enabled: true
"#,
        );

        let _env_guard = EnvGuard::set_str("SUBSTRATE_OVERRIDE_WORLD", "disabled");
        let message = resolve_world_disable_message(
            &workspace_root,
            CliConfigOverrides {
                world_enabled: Some(false),
                ..Default::default()
            },
        );
        assert_eq!(
            message,
            Some("world isolation disabled by CLI flag --no-world")
        );
    }

    #[test]
    #[serial]
    fn test_world_disable_attribution_maps_override_env() {
        let tmp = TempDir::new().unwrap();
        let substrate_home = tmp.path().join(".substrate");
        fs::create_dir_all(&substrate_home).unwrap();
        let _guard = EnvGuard::set("SUBSTRATE_HOME", &substrate_home);

        let global_path = global_config_path().unwrap();
        write_file(
            &global_path,
            r#"
world:
  enabled: true
"#,
        );

        let workspace_root = tmp.path().join("ws");
        fs::create_dir_all(&workspace_root).unwrap();
        let _env_guard = EnvGuard::set_str("SUBSTRATE_OVERRIDE_WORLD", "disabled");

        let message = resolve_world_disable_message(&workspace_root, CliConfigOverrides::default());
        assert_eq!(
            message,
            Some("world isolation disabled by env override SUBSTRATE_OVERRIDE_WORLD=disabled")
        );
    }

    #[test]
    #[serial]
    fn test_world_disable_attribution_maps_workspace_patch() {
        let tmp = TempDir::new().unwrap();
        let substrate_home = tmp.path().join(".substrate");
        fs::create_dir_all(&substrate_home).unwrap();
        let _guard = EnvGuard::set("SUBSTRATE_HOME", &substrate_home);

        let global_path = global_config_path().unwrap();
        write_file(
            &global_path,
            r#"
world:
  enabled: true
"#,
        );

        let workspace_root = tmp.path().join("ws");
        let workspace_yaml = crate::execution::workspace::workspace_marker_path(&workspace_root);
        write_file(
            &workspace_yaml,
            r#"
world:
  enabled: false
"#,
        );

        let message = resolve_world_disable_message(&workspace_root, CliConfigOverrides::default());
        assert_eq!(
            message,
            Some("world isolation disabled by workspace config <workspace>/.substrate/workspace.yaml (world.enabled: false)")
        );
    }

    #[test]
    #[serial]
    fn test_world_disable_attribution_maps_global_patch() {
        let tmp = TempDir::new().unwrap();
        let substrate_home = tmp.path().join(".substrate");
        fs::create_dir_all(&substrate_home).unwrap();
        let _guard = EnvGuard::set("SUBSTRATE_HOME", &substrate_home);

        let global_path = global_config_path().unwrap();
        write_file(
            &global_path,
            r#"
world:
  enabled: false
"#,
        );

        let workspace_root = tmp.path().join("ws");
        fs::create_dir_all(&workspace_root).unwrap();

        let message = resolve_world_disable_message(&workspace_root, CliConfigOverrides::default());
        assert_eq!(
            message,
            Some("world isolation disabled by global config $SUBSTRATE_HOME/config.yaml (world.enabled: false)")
        );
    }

    #[test]
    fn test_world_disable_attribution_maps_default_layer() {
        let message = world_disable_attribution_message(false, Some(&explain_key("default", None)));
        assert_eq!(
            message,
            Some("world isolation disabled by default config (world.enabled: false)")
        );
    }

    #[test]
    #[serial]
    fn test_world_disable_attribution_omits_enabled_cases() {
        let tmp = TempDir::new().unwrap();
        let substrate_home = tmp.path().join(".substrate");
        fs::create_dir_all(&substrate_home).unwrap();
        let _guard = EnvGuard::set("SUBSTRATE_HOME", &substrate_home);

        let workspace_root = tmp.path().join("ws");
        let workspace_yaml = crate::execution::workspace::workspace_marker_path(&workspace_root);
        write_file(
            &workspace_yaml,
            r#"
world:
  enabled: true
"#,
        );

        let message = resolve_world_disable_message(
            &workspace_root,
            CliConfigOverrides {
                world_enabled: Some(true),
                ..Default::default()
            },
        );
        assert_eq!(message, None);
    }

    #[test]
    fn test_world_disable_attribution_falls_back_when_provenance_missing() {
        assert_eq!(
            world_disable_attribution_message(false, None),
            Some("world isolation disabled by effective config (source unknown)")
        );
        assert_eq!(
            world_disable_attribution_message(
                false,
                Some(&ConfigExplainKey {
                    merge_strategy: "replace".to_string(),
                    sources: vec![],
                }),
            ),
            Some("world isolation disabled by effective config (source unknown)")
        );
        assert_eq!(
            world_disable_attribution_message(
                false,
                Some(&ConfigExplainKey {
                    merge_strategy: "replace".to_string(),
                    sources: vec![ConfigExplainSource {
                        layer: "not-a-real-layer".to_string(),
                        path: Some("/tmp/secret/path".to_string()),
                    }],
                })
            ),
            Some("world isolation disabled by effective config (source unknown)")
        );
    }

    #[test]
    fn test_world_disable_attribution_ignores_source_path() {
        let message = world_disable_attribution_message(
            false,
            Some(&ConfigExplainKey {
                merge_strategy: "replace".to_string(),
                sources: vec![ConfigExplainSource {
                    layer: "global_patch".to_string(),
                    path: Some("/tmp/secret/substrate/config.yaml".to_string()),
                }],
            }),
        );
        assert_eq!(
            message,
            Some("world isolation disabled by global config $SUBSTRATE_HOME/config.yaml (world.enabled: false)")
        );
    }

    #[test]
    #[serial]
    fn test_world_disable_attribution_builder_maps_sources() {
        let cases = [
            (
                "cli_flag",
                Some("world isolation disabled by CLI flag --no-world"),
                WorldDisableSource {
                    key: "world.enabled",
                    layer: "cli_flag",
                    value_display: false,
                    flag: Some("--no-world"),
                    env: None,
                    path_display: None,
                },
            ),
            (
                "override_env",
                Some("world isolation disabled by env override SUBSTRATE_OVERRIDE_WORLD=disabled"),
                WorldDisableSource {
                    key: "world.enabled",
                    layer: "override_env",
                    value_display: false,
                    flag: None,
                    env: Some("SUBSTRATE_OVERRIDE_WORLD"),
                    path_display: None,
                },
            ),
            (
                "workspace_patch",
                Some("world isolation disabled by workspace config <workspace>/.substrate/workspace.yaml (world.enabled: false)"),
                WorldDisableSource {
                    key: "world.enabled",
                    layer: "workspace_patch",
                    value_display: false,
                    flag: None,
                    env: None,
                    path_display: Some("<workspace>/.substrate/workspace.yaml"),
                },
            ),
            (
                "global_patch",
                Some("world isolation disabled by global config $SUBSTRATE_HOME/config.yaml (world.enabled: false)"),
                WorldDisableSource {
                    key: "world.enabled",
                    layer: "global_patch",
                    value_display: false,
                    flag: None,
                    env: None,
                    path_display: Some("$SUBSTRATE_HOME/config.yaml"),
                },
            ),
            (
                "default",
                Some("world isolation disabled by default config (world.enabled: false)"),
                WorldDisableSource {
                    key: "world.enabled",
                    layer: "default",
                    value_display: false,
                    flag: None,
                    env: None,
                    path_display: None,
                },
            ),
            (
                "source_unknown",
                Some("world isolation disabled by effective config (source unknown)"),
                WorldDisableSource {
                    key: "world.enabled",
                    layer: "source_unknown",
                    value_display: false,
                    flag: None,
                    env: None,
                    path_display: None,
                },
            ),
        ];

        for (layer, expected_reason, expected_source) in cases {
            let attribution = world_disable_attribution(
                false,
                Some(&ConfigExplainKey {
                    merge_strategy: "replace".to_string(),
                    sources: vec![ConfigExplainSource {
                        layer: layer.to_string(),
                        path: None,
                    }],
                }),
            )
            .expect("expected attribution");
            assert_eq!(Some(attribution.reason), expected_reason);
            assert_eq!(attribution.source, expected_source);
        }
    }

    #[test]
    fn test_world_disable_attribution_builder_falls_back_to_source_unknown() {
        let attribution = world_disable_attribution(
            false,
            Some(&ConfigExplainKey {
                merge_strategy: "replace".to_string(),
                sources: vec![],
            }),
        )
        .expect("expected attribution");

        assert_eq!(
            attribution.reason,
            "world isolation disabled by effective config (source unknown)"
        );
        assert_eq!(attribution.source.key, "world.enabled");
        assert_eq!(attribution.source.layer, "source_unknown");
        assert!(!attribution.source.value_display);
        assert_eq!(attribution.source.flag, None);
        assert_eq!(attribution.source.env, None);
        assert_eq!(attribution.source.path_display, None);
    }

    #[test]
    fn test_world_disable_attribution_builder_ignores_raw_source_path() {
        let workspace = world_disable_attribution(
            false,
            Some(&ConfigExplainKey {
                merge_strategy: "replace".to_string(),
                sources: vec![ConfigExplainSource {
                    layer: "workspace_patch".to_string(),
                    path: Some("/tmp/secret/workspace.yaml".to_string()),
                }],
            }),
        )
        .expect("expected attribution");
        assert_eq!(
            workspace.source.path_display,
            Some("<workspace>/.substrate/workspace.yaml")
        );

        let global = world_disable_attribution(
            false,
            Some(&ConfigExplainKey {
                merge_strategy: "replace".to_string(),
                sources: vec![ConfigExplainSource {
                    layer: "global_patch".to_string(),
                    path: Some("/tmp/secret/config.yaml".to_string()),
                }],
            }),
        )
        .expect("expected attribution");
        assert_eq!(
            global.source.path_display,
            Some("$SUBSTRATE_HOME/config.yaml")
        );
    }

    #[test]
    fn test_world_disable_attribution_builder_omits_enabled_cases() {
        assert_eq!(world_disable_attribution(true, None), None);
        assert_eq!(
            world_disable_attribution(
                true,
                Some(&ConfigExplainKey {
                    merge_strategy: "replace".to_string(),
                    sources: vec![ConfigExplainSource {
                        layer: "cli_flag".to_string(),
                        path: None,
                    }],
                }),
            ),
            None
        );
    }

    #[test]
    fn test_enum_validation_rejects_invalid_values_without_mutation() {
        let before = SubstrateConfigPatch::default();

        let mut patch = before.clone();
        let updates = parse_updates(&["world.deps.inventory_mode=bogus".to_string()]).unwrap();
        let err = apply_updates_to_patch(&mut patch, &updates).unwrap_err();
        assert!(is_user_error(&err));
        assert_eq!(patch, before);

        let mut patch = before.clone();
        let updates = parse_updates(&["world.deps.builtins=bogus".to_string()]).unwrap();
        let err = apply_updates_to_patch(&mut patch, &updates).unwrap_err();
        assert!(is_user_error(&err));
        assert_eq!(patch, before);
    }

    #[test]
    #[serial]
    fn test_explain_json_bytes_are_deterministic_for_identical_inputs() {
        let tmp = TempDir::new().unwrap();
        let substrate_home = tmp.path().join(".substrate");
        fs::create_dir_all(&substrate_home).unwrap();
        let _guard = EnvGuard::set("SUBSTRATE_HOME", &substrate_home);

        let global_path = global_config_path().unwrap();
        write_file(
            &global_path,
            r#"
world:
  deps:
    enabled: ["a", "b"]
"#,
        );

        let workspace_root = tmp.path().join("ws");
        let workspace_yaml = crate::execution::workspace::workspace_marker_path(&workspace_root);
        write_file(
            &workspace_yaml,
            r#"
world:
  deps:
    enabled: ["b", "c"]
"#,
        );

        let (_, explain_1) = resolve_effective_config_with_explain(
            &workspace_root,
            &CliConfigOverrides::default(),
            true,
        )
        .unwrap();
        let (_, explain_2) = resolve_effective_config_with_explain(
            &workspace_root,
            &CliConfigOverrides::default(),
            true,
        )
        .unwrap();

        let bytes_1 = serde_json::to_vec_pretty(&explain_1.unwrap()).unwrap();
        let bytes_2 = serde_json::to_vec_pretty(&explain_2.unwrap()).unwrap();
        assert_eq!(bytes_1, bytes_2);
    }
}
