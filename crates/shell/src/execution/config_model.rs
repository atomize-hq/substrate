use crate::execution::value_parse::parse_bool_flag;
use crate::execution::workspace;
use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};
use std::env;
use std::error::Error as StdError;
use std::fmt;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use substrate_common::paths as substrate_paths;
use substrate_common::WorldRootMode;

pub(crate) const PROTECTED_EXCLUDES: [&str; 3] = [".git/**", ".substrate/**", ".substrate-git/**"];

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
#[serde(deny_unknown_fields)]
pub(crate) struct SubstrateConfig {
    pub world: WorldConfig,
    pub policy: PolicyConfig,
    pub sync: SyncConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(crate) struct WorldConfig {
    pub enabled: bool,
    pub anchor_mode: WorldRootMode,
    pub anchor_path: String,
    pub caged: bool,
}

impl Default for WorldConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            anchor_mode: WorldRootMode::Project,
            anchor_path: String::new(),
            caged: true,
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
#[serde(deny_unknown_fields)]
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
#[serde(deny_unknown_fields)]
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

pub(crate) fn read_global_config_or_defaults() -> Result<(SubstrateConfig, bool)> {
    let path = global_config_path()?;
    match fs::read_to_string(&path) {
        Ok(raw) => Ok((parse_config_yaml(&path, &raw)?, true)),
        Err(err) if err.kind() == io::ErrorKind::NotFound => {
            Ok((SubstrateConfig::default(), false))
        }
        Err(err) => Err(anyhow!("failed to read {}: {err}", path.display())),
    }
}

pub(crate) fn parse_config_yaml(path: &Path, raw: &str) -> Result<SubstrateConfig> {
    let parsed: SubstrateConfig = serde_yaml::from_str(raw).map_err(|err| {
        user_error(format!(
            "invalid YAML in {}: {}",
            path.display(),
            err.to_string().trim()
        ))
    })?;
    validate_config(&parsed)?;
    Ok(parsed)
}

pub(crate) fn resolve_effective_config(
    cwd: &Path,
    cli: &CliConfigOverrides,
) -> Result<SubstrateConfig> {
    let (mut effective, _) = read_global_config_or_defaults()?;

    let workspace_root = workspace::find_workspace_root(cwd);
    if let Some(root) = &workspace_root {
        let legacy = workspace::workspace_legacy_settings_path(root);
        if legacy.exists() {
            return Err(user_error(format!(
                "substrate: unsupported legacy workspace config detected:\n  - {}\nConfig is now read from:\n  - {}\nNext steps:\n  - Delete the legacy file and use `substrate config set ...`\n",
                legacy.display(),
                workspace::workspace_marker_path(root).display()
            )));
        }

        let workspace_path = workspace::workspace_marker_path(root);
        let raw = fs::read_to_string(&workspace_path)
            .with_context(|| format!("failed to read {}", workspace_path.display()))?;
        effective = parse_config_yaml(&workspace_path, &raw)?;
    }

    apply_env_overrides(&mut effective)?;
    apply_cli_overrides(&mut effective, cli);
    apply_protected_excludes(&mut effective.sync.exclude);

    Ok(effective)
}

fn apply_cli_overrides(cfg: &mut SubstrateConfig, cli: &CliConfigOverrides) {
    if let Some(enabled) = cli.world_enabled {
        cfg.world.enabled = enabled;
    }
    if let Some(mode) = cli.anchor_mode {
        cfg.world.anchor_mode = mode;
    }
    if let Some(path) = &cli.anchor_path {
        cfg.world.anchor_path = path.clone();
    }
    if let Some(caged) = cli.caged {
        cfg.world.caged = caged;
    }
}

fn apply_env_overrides(cfg: &mut SubstrateConfig) -> Result<()> {
    if let Ok(world) = env::var("SUBSTRATE_WORLD") {
        let normalized = world.trim().to_ascii_lowercase();
        if !normalized.is_empty() {
            cfg.world.enabled = match normalized.as_str() {
                "enabled" => true,
                "disabled" => false,
                _ => {
                    return Err(user_error(format!(
                        "SUBSTRATE_WORLD must be 'enabled' or 'disabled' (found '{}')",
                        world
                    )));
                }
            };
        }
    }

    if let Ok(mode) = env::var("SUBSTRATE_ANCHOR_MODE") {
        let trimmed = mode.trim();
        if !trimmed.is_empty() {
            cfg.world.anchor_mode = WorldRootMode::parse(trimmed).ok_or_else(|| {
                user_error(format!(
                    "SUBSTRATE_ANCHOR_MODE must be one of workspace, follow-cwd, or custom (found '{}')",
                    mode
                ))
            })?;
        }
    }

    if let Ok(path) = env::var("SUBSTRATE_ANCHOR_PATH") {
        cfg.world.anchor_path = path;
    }

    if let Ok(raw) = env::var("SUBSTRATE_CAGED") {
        let parsed = parse_bool_flag(&raw).ok_or_else(|| {
            user_error(format!(
                "SUBSTRATE_CAGED must be a boolean (true|false|1|0|yes|no|on|off) (found '{}')",
                raw
            ))
        })?;
        cfg.world.caged = parsed;
    }

    if let Ok(raw) = env::var("SUBSTRATE_POLICY_MODE") {
        let trimmed = raw.trim();
        if !trimmed.is_empty() {
            cfg.policy.mode = PolicyMode::parse_insensitive(trimmed).ok_or_else(|| {
                user_error(format!(
                    "SUBSTRATE_POLICY_MODE must be one of disabled, observe, or enforce (found '{}')",
                    raw
                ))
            })?;
        }
    }

    if let Ok(raw) = env::var("SUBSTRATE_SYNC_AUTO_SYNC") {
        let parsed = parse_bool_flag(&raw).ok_or_else(|| {
            user_error(format!(
                "SUBSTRATE_SYNC_AUTO_SYNC must be a boolean (true|false|1|0|yes|no|on|off) (found '{}')",
                raw
            ))
        })?;
        cfg.sync.auto_sync = parsed;
    }

    if let Ok(raw) = env::var("SUBSTRATE_SYNC_DIRECTION") {
        let trimmed = raw.trim();
        if !trimmed.is_empty() {
            cfg.sync.direction = SyncDirection::parse_insensitive(trimmed).ok_or_else(|| {
                user_error(format!(
                    "SUBSTRATE_SYNC_DIRECTION must be one of from_world, from_host, or both (found '{}')",
                    raw
                ))
            })?;
        }
    }

    if let Ok(raw) = env::var("SUBSTRATE_SYNC_CONFLICT_POLICY") {
        let trimmed = raw.trim();
        if !trimmed.is_empty() {
            cfg.sync.conflict_policy =
                SyncConflictPolicy::parse_insensitive(trimmed).ok_or_else(|| {
                    user_error(format!(
                        "SUBSTRATE_SYNC_CONFLICT_POLICY must be one of prefer_host, prefer_world, or abort (found '{}')",
                        raw
                    ))
                })?;
        }
    }

    if let Ok(raw) = env::var("SUBSTRATE_SYNC_EXCLUDE") {
        let items = raw
            .split(',')
            .map(|item| item.trim())
            .filter(|item| !item.is_empty())
            .map(|item| item.to_string())
            .collect::<Vec<_>>();
        cfg.sync.exclude = items;
    }

    Ok(())
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

pub(crate) fn apply_updates(cfg: &mut SubstrateConfig, updates: &[ConfigUpdate]) -> Result<bool> {
    let mut changed = false;
    for update in updates {
        changed |= apply_update(cfg, update)?;
    }
    validate_config(cfg)?;
    Ok(changed)
}

fn validate_config(cfg: &SubstrateConfig) -> Result<()> {
    if cfg.world.anchor_mode == WorldRootMode::Custom && cfg.world.anchor_path.trim().is_empty() {
        return Err(user_error(
            "world.anchor_path is required when world.anchor_mode=custom",
        ));
    }
    Ok(())
}

fn apply_update(cfg: &mut SubstrateConfig, update: &ConfigUpdate) -> Result<bool> {
    match update.key.as_str() {
        "world.enabled" => apply_bool(&mut cfg.world.enabled, &update.op, &update.value),
        "world.anchor_mode" => {
            apply_enum_anchor_mode(&mut cfg.world.anchor_mode, &update.op, &update.value)
        }
        "world.anchor_path" => apply_string(&mut cfg.world.anchor_path, &update.op, &update.value),
        "world.caged" => apply_bool(&mut cfg.world.caged, &update.op, &update.value),
        "policy.mode" => apply_enum_policy_mode(&mut cfg.policy.mode, &update.op, &update.value),
        "sync.auto_sync" => apply_bool(&mut cfg.sync.auto_sync, &update.op, &update.value),
        "sync.direction" => {
            apply_enum_sync_direction(&mut cfg.sync.direction, &update.op, &update.value)
        }
        "sync.conflict_policy" => apply_enum_sync_conflict_policy(
            &mut cfg.sync.conflict_policy,
            &update.op,
            &update.value,
        ),
        "sync.exclude" => apply_list_exclude(&mut cfg.sync.exclude, update),
        _ => Err(user_error(format!(
            "unsupported config key '{}'",
            update.key
        ))),
    }
}

fn apply_bool(target: &mut bool, op: &UpdateOp, raw: &str) -> Result<bool> {
    if *op != UpdateOp::Set {
        return Err(user_error("unsupported operator for boolean key"));
    }
    let next = parse_bool_flag(raw).ok_or_else(|| {
        user_error(format!(
            "invalid boolean '{}'; expected true|false|1|0|yes|no|on|off",
            raw
        ))
    })?;
    let changed = *target != next;
    *target = next;
    Ok(changed)
}

fn apply_enum_anchor_mode(target: &mut WorldRootMode, op: &UpdateOp, raw: &str) -> Result<bool> {
    if *op != UpdateOp::Set {
        return Err(user_error("unsupported operator for enum key"));
    }
    let next = WorldRootMode::parse(raw).ok_or_else(|| {
        user_error(format!(
            "invalid world.anchor_mode '{}'; expected workspace, follow-cwd, or custom",
            raw
        ))
    })?;
    let changed = *target != next;
    *target = next;
    Ok(changed)
}

fn apply_enum_policy_mode(target: &mut PolicyMode, op: &UpdateOp, raw: &str) -> Result<bool> {
    if *op != UpdateOp::Set {
        return Err(user_error("unsupported operator for enum key"));
    }
    let next = PolicyMode::parse_insensitive(raw).ok_or_else(|| {
        user_error(format!(
            "invalid policy.mode '{}'; expected disabled, observe, or enforce",
            raw
        ))
    })?;
    let changed = *target != next;
    *target = next;
    Ok(changed)
}

fn apply_enum_sync_direction(target: &mut SyncDirection, op: &UpdateOp, raw: &str) -> Result<bool> {
    if *op != UpdateOp::Set {
        return Err(user_error("unsupported operator for enum key"));
    }
    let next = SyncDirection::parse_insensitive(raw).ok_or_else(|| {
        user_error(format!(
            "invalid sync.direction '{}'; expected from_world, from_host, or both",
            raw
        ))
    })?;
    let changed = *target != next;
    *target = next;
    Ok(changed)
}

fn apply_enum_sync_conflict_policy(
    target: &mut SyncConflictPolicy,
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
    let changed = *target != next;
    *target = next;
    Ok(changed)
}

fn apply_string(target: &mut String, op: &UpdateOp, raw: &str) -> Result<bool> {
    if *op != UpdateOp::Set {
        return Err(user_error("unsupported operator for string key"));
    }
    let changed = target != raw;
    *target = raw.to_string();
    Ok(changed)
}

fn apply_list_exclude(target: &mut Vec<String>, update: &ConfigUpdate) -> Result<bool> {
    match update.op {
        UpdateOp::Set => {
            let parsed: Vec<String> = serde_yaml::from_str(&update.value).map_err(|_| {
                user_error(format!(
                    "sync.exclude with '=' must be a YAML list literal (e.g., [] or [\"a\",\"b\"]); got '{}'",
                    update.value
                ))
            })?;
            let changed = *target != parsed;
            *target = parsed;
            Ok(changed)
        }
        UpdateOp::Append => {
            target.push(update.value.clone());
            Ok(true)
        }
        UpdateOp::Remove => {
            let before = target.len();
            target.retain(|item| item != &update.value);
            Ok(before != target.len())
        }
    }
}

pub(crate) fn default_policy_yaml() -> &'static str {
    // PCM0 requires workspace init to write the built-in default policy file inventory.
    // The policy schema and CLI are implemented in PCM1.
    r#"id: "default"
name: "Default Policy"

world_fs:
  mode: writable
  isolation: project
  require_world: false
  read_allowlist: ["*"]
  write_allowlist: []

net_allowed: []

cmd_allowed: []
cmd_denied:
  - "rm -rf *"
  - "curl * | bash"
  - "wget * | bash"
cmd_isolated: []

require_approval: false
allow_shell_operators: true

limits:
  max_memory_mb: null
  max_cpu_percent: null
  max_runtime_ms: null
  max_egress_bytes: null

metadata: {}
"#
}
