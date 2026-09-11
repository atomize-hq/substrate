use crate::execution::config_model;
use crate::execution::workspace;
use anyhow::Result;
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};
use substrate_broker::{validate_backend_id, validate_dotted_id, Policy};
use substrate_common::derive_agent_backend_id;
use substrate_common::paths as substrate_paths;

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct AgentFileV1 {
    pub version: u32,
    pub id: String,
    pub config: AgentConfigV1,
    #[serde(default)]
    pub policy_overlay: Option<crate::execution::policy_model::PolicyPatch>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct AgentConfigV1 {
    #[serde(default = "default_true")]
    pub enabled: bool,
    pub kind: AgentConfigKind,
    #[serde(default)]
    pub protocol: Option<String>,
    #[serde(default)]
    pub execution: AgentExecutionConfigV1,
    #[serde(default)]
    pub cli: Option<AgentCliConfigV1>,
    #[serde(default)]
    pub api: Option<AgentApiConfigV1>,
    #[serde(default)]
    pub capabilities: AgentCapabilitiesV1,
}

#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum AgentConfigKind {
    Cli,
    Api,
}

impl AgentConfigKind {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::Cli => "cli",
            Self::Api => "api",
        }
    }
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub(crate) struct AgentExecutionConfigV1 {
    #[serde(default)]
    pub scope: Option<crate::execution::config_model::AgentExecutionScope>,
}

#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum AgentCliRuntimeFamily {
    Codex,
    ClaudeCode,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub(crate) struct AgentCliConfigV1 {
    #[serde(default)]
    pub binary: String,
    #[serde(default)]
    pub mode: Option<crate::execution::config_model::AgentCliMode>,
    #[serde(default)]
    pub runtime_family: Option<AgentCliRuntimeFamily>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct AgentApiConfigV1 {
    pub base_url: String,
    pub auth: AgentApiAuthConfigV1,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct AgentApiAuthConfigV1 {
    pub env: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
struct AgentFileVersionProbe {
    version: u32,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct AgentFileV2 {
    pub version: u32,
    pub id: String,
    pub config: AgentConfigV2,
    #[serde(default)]
    pub policy_overlay: Option<crate::execution::policy_model::PolicyPatch>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct AgentConfigV2 {
    #[serde(default = "default_true")]
    pub enabled: bool,
    pub kind: AgentConfigKind,
    #[serde(default)]
    pub protocol: Option<String>,
    pub placements: AgentPlacementsV2,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub(crate) struct AgentPlacementsV2 {
    pub host: Option<AgentPlacementConfigV2>,
    pub world: Option<AgentPlacementConfigV2>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct AgentPlacementConfigV2 {
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default)]
    pub cli: Option<AgentCliConfigV1>,
    #[serde(default)]
    pub api: Option<AgentApiConfigV1>,
    #[serde(default)]
    pub capabilities: AgentCapabilitiesV1,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct AgentFileV3 {
    pub version: u32,
    pub id: String,
    pub config: AgentConfigV3,
    #[serde(default)]
    pub policy_overlay: Option<crate::execution::policy_model::PolicyPatch>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct AgentConfigV3 {
    pub enabled: bool,
    pub kind: AgentConfigKind,
    pub protocol: Option<String>,
    pub placements: AgentPlacementsV3,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct AgentPlacementsV3 {
    pub host: Option<AgentPlacementConfigV3>,
    pub world: Option<AgentPlacementConfigV3>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct AgentPlacementConfigV3 {
    pub enabled: bool,
    pub cli: Option<AgentCliConfigV1>,
    pub api: Option<AgentApiConfigV1>,
    pub capabilities: AgentCapabilitiesV1,
    pub runtime_projection: Option<AgentRuntimeProjectionInputV1>,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(crate) struct AgentRuntimeProjectionInputV1 {
    pub model: String,
    pub mcp_servers: Vec<config_projection::LogicalMcpServerV1>,
    pub features: Vec<config_projection::LogicalFeatureV1>,
}

pub(crate) type AgentInventorySourceMaterialV1 = config_projection::AgentInventorySourceMaterialV1;

#[derive(Debug, Clone, Default, Deserialize, PartialEq, Eq)]
#[serde(default, deny_unknown_fields)]
pub(crate) struct AgentCapabilitiesV1 {
    pub session_start: bool,
    pub session_resume: bool,
    pub session_fork: bool,
    pub session_stop: bool,
    pub status_snapshot: bool,
    pub event_stream: bool,
    pub llm: bool,
    pub mcp_client: bool,
}

#[derive(Debug, Clone)]
pub(crate) struct AgentInventoryEntryV1 {
    pub path: PathBuf,
    pub file: AgentFileV1,
}

#[derive(Debug, Clone)]
#[allow(
    dead_code,
    clippy::large_enum_variant,
    reason = "E3-C validates and retains complete V3 source material before E3-D consumes it"
)]
enum ParsedAgentInventoryFile {
    V1(AgentFileV1),
    V2(AgentFileV2),
    V3(AgentFileV3, AgentInventorySourceMaterialV1),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum AgentInventoryBaselineOrigin {
    GlobalInventory,
    WorkspaceInventory,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ProjectedInventoryValueOrigin {
    InventoryExplicit,
    EffectiveConfigDefault,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum AgentPlacement {
    Host,
    World,
}

impl AgentPlacement {
    #[allow(dead_code)]
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::Host => "host",
            Self::World => "world",
        }
    }

    pub(crate) fn execution_scope(self) -> crate::execution::config_model::AgentExecutionScope {
        match self {
            Self::Host => crate::execution::config_model::AgentExecutionScope::Host,
            Self::World => crate::execution::config_model::AgentExecutionScope::World,
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct ProjectedInventoryEntryV1 {
    pub origin: AgentInventoryBaselineOrigin,
    pub path: PathBuf,
    pub agent_id: String,
    pub backend_id: String,
    pub kind: AgentConfigKind,
    pub protocol: Option<String>,
    pub execution_scope: crate::execution::config_model::AgentExecutionScope,
    pub execution_scope_origin: ProjectedInventoryValueOrigin,
    pub cli_mode: crate::execution::config_model::AgentCliMode,
    pub cli_mode_origin: ProjectedInventoryValueOrigin,
    pub cli_binary: Option<String>,
    pub cli_runtime_family: Option<AgentCliRuntimeFamily>,
    pub capabilities: AgentCapabilitiesV1,
    #[allow(dead_code)]
    pub policy_overlay: Option<crate::execution::policy_model::PolicyPatch>,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub(crate) struct PlacementProjectedInventoryEntryV2 {
    pub origin: AgentInventoryBaselineOrigin,
    pub path: PathBuf,
    pub logical_agent_id: String,
    pub placement: AgentPlacement,
    pub realized_agent_id: String,
    pub backend_id: String,
    pub display_label: String,
    pub kind: AgentConfigKind,
    pub protocol: Option<String>,
    pub execution_scope: crate::execution::config_model::AgentExecutionScope,
    pub execution_scope_origin: ProjectedInventoryValueOrigin,
    pub cli_mode: crate::execution::config_model::AgentCliMode,
    pub cli_mode_origin: ProjectedInventoryValueOrigin,
    pub cli_binary: Option<String>,
    pub cli_runtime_family: Option<AgentCliRuntimeFamily>,
    pub capabilities: AgentCapabilitiesV1,
    pub policy_overlay: Option<crate::execution::policy_model::PolicyPatch>,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub(crate) struct PlacementProjectedInventoryEntryV3 {
    pub origin: AgentInventoryBaselineOrigin,
    pub path: PathBuf,
    pub logical_agent_id: String,
    pub placement: AgentPlacement,
    pub realized_agent_id: String,
    pub backend_id: String,
    pub display_label: String,
    pub kind: AgentConfigKind,
    pub protocol: Option<String>,
    pub execution_scope: crate::execution::config_model::AgentExecutionScope,
    pub execution_scope_origin: ProjectedInventoryValueOrigin,
    pub cli_mode: crate::execution::config_model::AgentCliMode,
    pub cli_mode_origin: ProjectedInventoryValueOrigin,
    pub cli_binary: Option<String>,
    pub cli_runtime_family: Option<AgentCliRuntimeFamily>,
    pub capabilities: AgentCapabilitiesV1,
    pub policy_overlay: Option<crate::execution::policy_model::PolicyPatch>,
    pub runtime_projection: Option<AgentRuntimeProjectionInputV1>,
    pub source: AgentInventorySourceMaterialV1,
}

impl AgentFileV1 {
    pub(crate) fn derived_backend_id(&self) -> String {
        derive_agent_backend_id(self.config.kind.as_str(), &self.id)
    }

    pub(crate) fn effective_scope(
        &self,
        effective_config: &crate::execution::config_model::SubstrateConfig,
    ) -> crate::execution::config_model::AgentExecutionScope {
        self.config
            .execution
            .scope
            .unwrap_or(effective_config.agents.defaults.execution.scope)
    }

    pub(crate) fn effective_cli_mode(
        &self,
        effective_config: &crate::execution::config_model::SubstrateConfig,
    ) -> crate::execution::config_model::AgentCliMode {
        self.config
            .cli
            .as_ref()
            .and_then(|cli| cli.mode)
            .unwrap_or(effective_config.agents.defaults.cli.mode)
    }

    pub(crate) fn effective_cli_binary(&self) -> Option<&str> {
        let cli = self.config.cli.as_ref()?;
        let trimmed = cli.binary.trim();
        if trimmed.is_empty() {
            Some(self.id.as_str())
        } else {
            Some(trimmed)
        }
    }

    pub(crate) fn cli_runtime_family(&self) -> Option<AgentCliRuntimeFamily> {
        self.config.cli.as_ref().and_then(|cli| cli.runtime_family)
    }
}

impl AgentInventoryEntryV1 {
    pub(crate) fn derived_backend_id(&self) -> String {
        self.file.derived_backend_id()
    }

    pub(crate) fn effective_scope(
        &self,
        effective_config: &crate::execution::config_model::SubstrateConfig,
    ) -> crate::execution::config_model::AgentExecutionScope {
        self.file.effective_scope(effective_config)
    }

    pub(crate) fn effective_cli_mode(
        &self,
        effective_config: &crate::execution::config_model::SubstrateConfig,
    ) -> crate::execution::config_model::AgentCliMode {
        self.file.effective_cli_mode(effective_config)
    }

    pub(crate) fn effective_cli_binary(&self) -> Option<&str> {
        self.file.effective_cli_binary()
    }

    pub(crate) fn cli_runtime_family(&self) -> Option<AgentCliRuntimeFamily> {
        self.file.cli_runtime_family()
    }
}

pub(crate) fn inventory_entry_origin(
    cwd: &Path,
    entry: &AgentInventoryEntryV1,
) -> AgentInventoryBaselineOrigin {
    inventory_path_origin(cwd, &entry.path)
}

fn inventory_path_origin(cwd: &Path, path: &Path) -> AgentInventoryBaselineOrigin {
    let Some(workspace_root) = workspace::find_workspace_root(cwd) else {
        return AgentInventoryBaselineOrigin::GlobalInventory;
    };
    let workspace_agents_root = normalize_inventory_origin_path(
        &workspace_root
            .join(workspace::SUBSTRATE_DIR_NAME)
            .join("agents"),
    );
    let entry_path = normalize_inventory_origin_path(path);

    if entry_path.starts_with(&workspace_agents_root) {
        AgentInventoryBaselineOrigin::WorkspaceInventory
    } else {
        AgentInventoryBaselineOrigin::GlobalInventory
    }
}

fn normalize_inventory_origin_path(path: &Path) -> PathBuf {
    let mut current = path;
    let mut suffix = Vec::new();

    loop {
        if let Ok(canonical) = current.canonicalize() {
            let mut normalized = canonical;
            for component in suffix.iter().rev() {
                normalized.push(component);
            }
            return normalized;
        }

        let Some(name) = current.file_name() else {
            return path.to_path_buf();
        };
        let Some(parent) = current.parent() else {
            return path.to_path_buf();
        };

        suffix.push(name.to_os_string());
        current = parent;
    }
}

pub(crate) fn project_inventory_entry(
    cwd: &Path,
    entry: &AgentInventoryEntryV1,
    effective_config: &crate::execution::config_model::SubstrateConfig,
) -> ProjectedInventoryEntryV1 {
    let execution_scope_origin = if entry.file.config.execution.scope.is_some() {
        ProjectedInventoryValueOrigin::InventoryExplicit
    } else {
        ProjectedInventoryValueOrigin::EffectiveConfigDefault
    };
    let cli_mode_origin = if entry
        .file
        .config
        .cli
        .as_ref()
        .and_then(|cli| cli.mode)
        .is_some()
    {
        ProjectedInventoryValueOrigin::InventoryExplicit
    } else {
        ProjectedInventoryValueOrigin::EffectiveConfigDefault
    };

    ProjectedInventoryEntryV1 {
        origin: inventory_entry_origin(cwd, entry),
        path: entry.path.clone(),
        agent_id: entry.file.id.clone(),
        backend_id: entry.derived_backend_id(),
        kind: entry.file.config.kind,
        protocol: entry.file.config.protocol.clone(),
        execution_scope: entry.effective_scope(effective_config),
        execution_scope_origin,
        cli_mode: entry.effective_cli_mode(effective_config),
        cli_mode_origin,
        cli_binary: entry.effective_cli_binary().map(ToOwned::to_owned),
        cli_runtime_family: entry.cli_runtime_family(),
        capabilities: entry.file.config.capabilities.clone(),
        policy_overlay: entry.file.policy_overlay.clone(),
    }
}

#[allow(dead_code)]
pub(crate) fn project_inventory_v2_entry(
    cwd: &Path,
    path: &Path,
    file: &AgentFileV2,
    effective_config: &crate::execution::config_model::SubstrateConfig,
) -> Vec<PlacementProjectedInventoryEntryV2> {
    if !file.config.enabled {
        return Vec::new();
    }

    let origin = inventory_path_origin(cwd, path);
    let mut projected = Vec::new();
    for (placement, placement_config) in [
        (AgentPlacement::Host, file.config.placements.host.as_ref()),
        (AgentPlacement::World, file.config.placements.world.as_ref()),
    ] {
        let Some(placement_config) = placement_config else {
            continue;
        };
        if !placement_config.enabled {
            continue;
        }

        let cli_mode_origin = if placement_config
            .cli
            .as_ref()
            .and_then(|cli| cli.mode)
            .is_some()
        {
            ProjectedInventoryValueOrigin::InventoryExplicit
        } else {
            ProjectedInventoryValueOrigin::EffectiveConfigDefault
        };
        let cli_mode = placement_config
            .cli
            .as_ref()
            .and_then(|cli| cli.mode)
            .unwrap_or(effective_config.agents.defaults.cli.mode);
        let cli_binary = placement_config.cli.as_ref().map(|cli| {
            let trimmed = cli.binary.trim();
            if trimmed.is_empty() {
                file.id.clone()
            } else {
                trimmed.to_string()
            }
        });
        let realized_agent_id = format!("{}-{}", file.id, placement.as_str());

        projected.push(PlacementProjectedInventoryEntryV2 {
            origin,
            path: path.to_path_buf(),
            logical_agent_id: file.id.clone(),
            placement,
            realized_agent_id: realized_agent_id.clone(),
            backend_id: derive_agent_backend_id(file.config.kind.as_str(), &realized_agent_id),
            display_label: format!("{} ({})", file.id, placement.as_str()),
            kind: file.config.kind,
            protocol: file.config.protocol.clone(),
            execution_scope: placement.execution_scope(),
            execution_scope_origin: ProjectedInventoryValueOrigin::InventoryExplicit,
            cli_mode,
            cli_mode_origin,
            cli_binary,
            cli_runtime_family: placement_config
                .cli
                .as_ref()
                .and_then(|cli| cli.runtime_family),
            capabilities: placement_config.capabilities.clone(),
            policy_overlay: file.policy_overlay.clone(),
        });
    }

    projected
}

#[allow(dead_code)]
pub(crate) fn project_inventory_v3_entry(
    cwd: &Path,
    path: &Path,
    file: &AgentFileV3,
    effective_config: &crate::execution::config_model::SubstrateConfig,
    source: &AgentInventorySourceMaterialV1,
) -> Vec<PlacementProjectedInventoryEntryV3> {
    if !file.config.enabled {
        return Vec::new();
    }
    let origin = inventory_path_origin(cwd, path);
    let mut projected = Vec::new();
    for (placement, placement_config) in [
        (AgentPlacement::Host, file.config.placements.host.as_ref()),
        (AgentPlacement::World, file.config.placements.world.as_ref()),
    ] {
        let Some(placement_config) = placement_config else {
            continue;
        };
        if !placement_config.enabled {
            continue;
        }
        let cli_mode_origin = if placement_config
            .cli
            .as_ref()
            .and_then(|cli| cli.mode)
            .is_some()
        {
            ProjectedInventoryValueOrigin::InventoryExplicit
        } else {
            ProjectedInventoryValueOrigin::EffectiveConfigDefault
        };
        let cli_mode = placement_config
            .cli
            .as_ref()
            .and_then(|cli| cli.mode)
            .unwrap_or(effective_config.agents.defaults.cli.mode);
        let cli_binary = placement_config.cli.as_ref().map(|cli| {
            let trimmed = cli.binary.trim();
            if trimmed.is_empty() {
                file.id.clone()
            } else {
                trimmed.to_string()
            }
        });
        let realized_agent_id = format!("{}-{}", file.id, placement.as_str());
        projected.push(PlacementProjectedInventoryEntryV3 {
            origin,
            path: path.to_path_buf(),
            logical_agent_id: file.id.clone(),
            placement,
            realized_agent_id: realized_agent_id.clone(),
            backend_id: derive_agent_backend_id(file.config.kind.as_str(), &realized_agent_id),
            display_label: format!("{} ({})", file.id, placement.as_str()),
            kind: file.config.kind,
            protocol: file.config.protocol.clone(),
            execution_scope: placement.execution_scope(),
            execution_scope_origin: ProjectedInventoryValueOrigin::InventoryExplicit,
            cli_mode,
            cli_mode_origin,
            cli_binary,
            cli_runtime_family: placement_config
                .cli
                .as_ref()
                .and_then(|cli| cli.runtime_family),
            capabilities: placement_config.capabilities.clone(),
            policy_overlay: file.policy_overlay.clone(),
            runtime_projection: placement_config.runtime_projection.clone(),
            source: source.clone(),
        });
    }
    projected
}

fn default_true() -> bool {
    true
}

pub(crate) fn discover_agent_files(cwd: &Path) -> Result<Vec<PathBuf>> {
    let mut roots = vec![substrate_paths::substrate_home()?.join("agents")];
    if let Some(workspace_root) = workspace::find_workspace_root(cwd) {
        roots.push(
            workspace_root
                .join(workspace::SUBSTRATE_DIR_NAME)
                .join("agents"),
        );
    }

    let mut files = Vec::new();
    for root in roots {
        if !root.exists() {
            continue;
        }
        if !root.is_dir() {
            return Err(config_model::user_error(format!(
                "invalid agent inventory directory {}: expected a directory",
                root.display()
            )));
        }
        collect_agent_files(&root, &mut files)?;
    }

    files.sort();
    files.dedup();
    Ok(files)
}

pub(crate) fn load_effective_agent_inventory(
    cwd: &Path,
    base_policy: &Policy,
) -> Result<BTreeMap<String, AgentInventoryEntryV1>> {
    let world_root = authoritative_inventory_world_root(cwd);
    let mut effective = BTreeMap::new();
    for root in discover_agent_inventory_roots(cwd)? {
        if !root.exists() {
            continue;
        }
        if !root.is_dir() {
            return Err(config_model::user_error(format!(
                "invalid agent inventory directory {}: expected a directory",
                root.display()
            )));
        }
        let workspace_inventory =
            inventory_path_origin(cwd, &root) == AgentInventoryBaselineOrigin::WorkspaceInventory;
        let (source_scope, accepted_root) = if workspace_inventory {
            (
                "workspace",
                root.parent()
                    .and_then(Path::parent)
                    .ok_or_else(|| config_model::user_error("invalid workspace inventory root"))?,
            )
        } else {
            (
                "global",
                root.parent()
                    .ok_or_else(|| config_model::user_error("invalid global inventory root"))?,
            )
        };
        let mut validated_files = Vec::new();
        let mut root_shadowed_agent_ids = BTreeSet::new();
        for path in collect_agent_files_in_root(&root)? {
            let file = parse_and_validate_agent_file(
                &path,
                base_policy,
                Some(&world_root),
                source_scope,
                accepted_root,
            )?;
            if let ParsedAgentInventoryFile::V2(parsed) = &file {
                root_shadowed_agent_ids.extend(legacy_shadowed_agent_ids_from_v2(parsed));
            }
            if let ParsedAgentInventoryFile::V3(parsed, _) = &file {
                root_shadowed_agent_ids.extend([
                    parsed.id.clone(),
                    format!("{}-host", parsed.id),
                    format!("{}-world", parsed.id),
                ]);
            }
            validated_files.push((path, file));
        }
        merge_inventory_root(
            &mut effective,
            workspace_inventory,
            root_shadowed_agent_ids,
            validated_files,
        )?;
    }

    Ok(effective)
}

#[allow(
    dead_code,
    reason = "A1.1e establishes the explicit-home entry point before A1.3 adopts it"
)]
pub(crate) fn load_effective_agent_inventory_for_bootstrap_home(
    cwd: &Path,
    base_policy: &Policy,
    bootstrap_home: &crate::execution::agent_runtime::OpenedBootstrapHomeV1<'_>,
) -> Result<BTreeMap<String, AgentInventoryEntryV1>> {
    let world_root = authoritative_inventory_world_root(cwd);
    let mut effective = BTreeMap::new();
    let mut global_files = Vec::new();
    let mut global_shadowed = BTreeSet::new();
    let bootstrap_identity = bootstrap_home
        .identity()
        .map_err(|error| config_model::user_error(error.to_string()))?
        .clone();
    let global_agents_root = PathBuf::from(&bootstrap_identity.physical_path).join("agents");
    for (name, bytes) in bootstrap_home
        .read_agent_inventory_yaml()
        .map_err(|error| config_model::user_error(error.to_string()))?
    {
        let path = global_agents_root.join(name);
        let raw = std::str::from_utf8(&bytes).map_err(|_| {
            config_model::user_error(format!("invalid UTF-8 in {}", path.display()))
        })?;
        let file = parse_and_validate_agent_file_raw(
            &path,
            raw,
            base_policy,
            Some(&world_root),
            "global",
            Path::new(&bootstrap_identity.physical_path),
        )?;
        if let ParsedAgentInventoryFile::V2(parsed) = &file {
            global_shadowed.extend(legacy_shadowed_agent_ids_from_v2(parsed));
        }
        if let ParsedAgentInventoryFile::V3(parsed, source) = &file {
            let root_matches = source.accepted_root.physical_path
                == bootstrap_identity.physical_path
                && matches!(
                    (
                        &source.accepted_root.physical_identity,
                        &bootstrap_identity.physical_identity,
                    ),
                    (
                        config_projection::DirectoryPhysicalIdentityV1::Linux {
                            device_id: source_device,
                            inode: source_inode,
                        },
                        crate::execution::agent_runtime::host_session_authority::schema::DirectoryPhysicalIdentityV1::Linux {
                            device_id: bootstrap_device,
                            inode: bootstrap_inode,
                        },
                    ) if source_device == bootstrap_device && source_inode == bootstrap_inode
                );
            if !root_matches {
                return Err(config_model::user_error(
                    "authenticated bootstrap inventory root identity changed",
                ));
            }
            global_shadowed.extend([
                parsed.id.clone(),
                format!("{}-host", parsed.id),
                format!("{}-world", parsed.id),
            ]);
        }
        global_files.push((path, file));
    }
    merge_inventory_root(&mut effective, false, global_shadowed, global_files)?;

    if let Some(workspace_root) = workspace::find_workspace_root(cwd) {
        let root = workspace_root
            .join(workspace::SUBSTRATE_DIR_NAME)
            .join("agents");
        if root.exists() {
            if !root.is_dir() {
                return Err(config_model::user_error(format!(
                    "invalid agent inventory directory {}: expected a directory",
                    root.display()
                )));
            }
            let mut workspace_files = Vec::new();
            let mut workspace_shadowed = BTreeSet::new();
            for path in collect_agent_files_in_root(&root)? {
                let file = parse_and_validate_agent_file(
                    &path,
                    base_policy,
                    Some(&world_root),
                    "workspace",
                    &workspace_root,
                )?;
                if let ParsedAgentInventoryFile::V2(parsed) = &file {
                    workspace_shadowed.extend(legacy_shadowed_agent_ids_from_v2(parsed));
                }
                if let ParsedAgentInventoryFile::V3(parsed, _) = &file {
                    workspace_shadowed.extend([
                        parsed.id.clone(),
                        format!("{}-host", parsed.id),
                        format!("{}-world", parsed.id),
                    ]);
                }
                workspace_files.push((path, file));
            }
            merge_inventory_root(&mut effective, true, workspace_shadowed, workspace_files)?;
        }
    }
    Ok(effective)
}

fn merge_inventory_root(
    effective: &mut BTreeMap<String, AgentInventoryEntryV1>,
    workspace: bool,
    root_shadowed_agent_ids: BTreeSet<String>,
    validated_files: Vec<(PathBuf, ParsedAgentInventoryFile)>,
) -> Result<()> {
    let mut v3_logical_ids = BTreeSet::new();
    for (_, file) in &validated_files {
        if let ParsedAgentInventoryFile::V3(file, _) = file {
            if !v3_logical_ids.insert(file.id.as_str()) {
                return Err(config_model::user_error(format!(
                    "ambiguous agent inventory id '{}' appears more than once in one inventory root",
                    file.id
                )));
            }
        }
    }
    if workspace {
        // Packet 1.5 keeps workspace-local version-2 compatibility rows from leaving
        // stale global split-entry truth live. A workspace v2 override replaces the
        // whole pre-cutover logical-agent view, so it must suppress both legacy
        // compatibility ids (`logical` and `logical_world`) regardless of which
        // single placement the bridge materialized.
        for shadowed_agent_id in &root_shadowed_agent_ids {
            effective.remove(shadowed_agent_id);
        }
    }
    for (path, file) in validated_files {
        match file {
            ParsedAgentInventoryFile::V1(file) => {
                if root_shadowed_agent_ids.contains(&file.id) {
                    continue;
                }
                effective.insert(file.id.clone(), AgentInventoryEntryV1 { path, file });
            }
            ParsedAgentInventoryFile::V2(file) => {
                for entry in materialize_effective_inventory_entries_from_v2(&path, &file) {
                    effective.insert(entry.file.id.clone(), entry);
                }
            }
            ParsedAgentInventoryFile::V3(_, _) => {
                return Err(config_model::user_error(
                    "unsupported agent schema_version 3 in legacy inventory loader",
                ));
            }
        }
    }
    Ok(())
}

fn materialize_effective_inventory_entries_from_v2(
    path: &Path,
    file: &AgentFileV2,
) -> Vec<AgentInventoryEntryV1> {
    if !file.config.enabled {
        return Vec::new();
    }

    let mut entries = Vec::new();
    for (placement, placement_config) in [
        (AgentPlacement::Host, file.config.placements.host.as_ref()),
        (AgentPlacement::World, file.config.placements.world.as_ref()),
    ] {
        let Some(placement_config) = placement_config else {
            continue;
        };
        if !placement_config.enabled {
            continue;
        }

        entries.push(AgentInventoryEntryV1 {
            path: path.to_path_buf(),
            file: AgentFileV1 {
                version: file.version,
                id: format!("{}-{}", file.id, placement.as_str()),
                config: AgentConfigV1 {
                    enabled: true,
                    kind: file.config.kind,
                    protocol: file.config.protocol.clone(),
                    execution: AgentExecutionConfigV1 {
                        scope: Some(placement.execution_scope()),
                    },
                    cli: materialized_cli_config_from_v2(&file.id, placement_config.cli.as_ref()),
                    api: placement_config.api.clone(),
                    capabilities: placement_config.capabilities.clone(),
                },
                policy_overlay: file.policy_overlay.clone(),
            },
        });
    }

    entries
}

fn materialized_cli_config_from_v2(
    logical_agent_id: &str,
    cli: Option<&AgentCliConfigV1>,
) -> Option<AgentCliConfigV1> {
    let mut cli = cli.cloned()?;
    if cli.binary.trim().is_empty() {
        cli.binary = logical_agent_id.to_string();
    }
    Some(cli)
}

fn legacy_shadowed_agent_ids_from_v2(file: &AgentFileV2) -> Vec<String> {
    vec![
        file.id.clone(),
        format!("{}_world", file.id),
        format!("{}-host", file.id),
        format!("{}-world", file.id),
    ]
}

#[allow(
    dead_code,
    reason = "ambient gateway inventory compatibility remains frozen until R2-3"
)]
pub(crate) fn resolve_gateway_backend_inventory_entry(
    cwd: &Path,
    backend_id: &str,
    base_policy: &Policy,
) -> Result<AgentInventoryEntryV1> {
    validate_backend_id(backend_id).map_err(|err| config_model::user_error(err.to_string()))?;
    let trimmed_backend_id = backend_id.trim();
    let (backend_kind, backend_name) = trimmed_backend_id
        .split_once(':')
        .expect("validated backend id should contain a colon");
    let effective_inventory = load_effective_agent_inventory(cwd, base_policy)?;
    let entry = effective_inventory.get(backend_name).ok_or_else(|| {
        config_model::user_error(format!(
            "gateway backend '{}' is not present in the effective agent inventory",
            trimmed_backend_id
        ))
    })?;

    let derived_backend_id = entry.derived_backend_id();
    if derived_backend_id != trimmed_backend_id || backend_kind != entry.file.config.kind.as_str() {
        return Err(config_model::user_error(format!(
            "gateway backend '{}' does not match effective inventory item '{}' in {}",
            trimmed_backend_id,
            derived_backend_id,
            entry.path.display()
        )));
    }

    if !entry.file.config.capabilities.llm {
        return Err(config_model::user_error(format!(
            "gateway backend '{}' is not llm-capable in {}",
            trimmed_backend_id,
            entry.path.display()
        )));
    }

    Ok(entry.clone())
}

#[cfg_attr(
    not(unix),
    allow(
        dead_code,
        reason = "authenticated non-Unix inventory remains R2-3-owned"
    )
)]
pub(crate) fn resolve_gateway_backend_inventory_entry_for_bootstrap_home(
    cwd: &Path,
    backend_id: &str,
    base_policy: &Policy,
    bootstrap_home: &crate::execution::agent_runtime::OpenedBootstrapHomeV1<'_>,
) -> Result<AgentInventoryEntryV1> {
    validate_backend_id(backend_id).map_err(|err| config_model::user_error(err.to_string()))?;
    let trimmed_backend_id = backend_id.trim();
    let (backend_kind, backend_name) = trimmed_backend_id
        .split_once(':')
        .expect("validated backend id should contain a colon");
    let effective_inventory =
        load_effective_agent_inventory_for_bootstrap_home(cwd, base_policy, bootstrap_home)?;
    let entry = effective_inventory.get(backend_name).ok_or_else(|| {
        config_model::user_error(format!(
            "gateway backend '{}' is not present in the effective agent inventory",
            trimmed_backend_id
        ))
    })?;

    let derived_backend_id = entry.derived_backend_id();
    if derived_backend_id != trimmed_backend_id || backend_kind != entry.file.config.kind.as_str() {
        return Err(config_model::user_error(format!(
            "gateway backend '{}' does not match effective inventory item '{}' in {}",
            trimmed_backend_id,
            derived_backend_id,
            entry.path.display()
        )));
    }

    if !entry.file.config.capabilities.llm {
        return Err(config_model::user_error(format!(
            "gateway backend '{}' is not llm-capable in {}",
            trimmed_backend_id,
            entry.path.display()
        )));
    }

    Ok(entry.clone())
}

pub(crate) fn validate_agent_file(path: &Path, base_policy: &Policy) -> Result<AgentFileV1> {
    let world_root = inventory_workspace_root_from_path(path);
    let accepted_root = world_root.as_deref().unwrap_or_else(|| {
        path.parent()
            .and_then(Path::parent)
            .unwrap_or_else(|| Path::new("/"))
    });
    let source_scope = if world_root.is_some() {
        "workspace"
    } else {
        "global"
    };
    match parse_and_validate_agent_file(
        path,
        base_policy,
        world_root.as_deref(),
        source_scope,
        accepted_root,
    )? {
        ParsedAgentInventoryFile::V1(parsed) => Ok(parsed),
        ParsedAgentInventoryFile::V2(parsed) => Ok(compatibility_inventory_file_from_v2(&parsed)),
        ParsedAgentInventoryFile::V3(_, _) => Err(config_model::user_error(
            "unsupported agent schema_version 3 in legacy validate_agent_file",
        )),
    }
}

fn parse_and_validate_agent_file(
    path: &Path,
    base_policy: &Policy,
    world_root: Option<&Path>,
    source_scope: &str,
    accepted_root: &Path,
) -> Result<ParsedAgentInventoryFile> {
    let raw = fs::read_to_string(path).map_err(|err| {
        config_model::user_error(format!("failed to read {}: {err}", path.display()))
    })?;
    parse_and_validate_agent_file_raw(
        path,
        &raw,
        base_policy,
        world_root,
        source_scope,
        accepted_root,
    )
}

fn parse_and_validate_agent_file_raw(
    path: &Path,
    raw: &str,
    base_policy: &Policy,
    world_root: Option<&Path>,
    source_scope: &str,
    accepted_root: &Path,
) -> Result<ParsedAgentInventoryFile> {
    match detect_agent_inventory_version(path, raw)? {
        1 => {
            let parsed: AgentFileV1 = serde_yaml::from_str(raw).map_err(|err| {
                config_model::user_error(format!(
                    "invalid YAML in {}: {}",
                    path.display(),
                    err.to_string().trim()
                ))
            })?;

            validate_agent_schema(path, &parsed, base_policy, world_root)?;
            Ok(ParsedAgentInventoryFile::V1(parsed))
        }
        2 => {
            let parsed: AgentFileV2 = serde_yaml::from_str(raw).map_err(|err| {
                config_model::user_error(format!(
                    "invalid YAML in {}: {}",
                    path.display(),
                    err.to_string().trim()
                ))
            })?;

            validate_agent_schema_v2(path, &parsed, base_policy, world_root)?;
            Ok(ParsedAgentInventoryFile::V2(parsed))
        }
        3 => {
            fn validate_strict_v3_yaml(path: &Path, raw: &str) -> Result<()> {
                let mut document_markers = 0usize;
                for line in raw.lines() {
                    let trimmed = line.trim_start();
                    if trimmed == "---" {
                        document_markers += 1;
                        if document_markers > 1 {
                            return Err(config_model::user_error(format!(
                                "invalid strict V3 YAML in {}: multiple documents are forbidden",
                                path.display()
                            )));
                        }
                    }
                    if trimmed == "..." {
                        return Err(config_model::user_error(format!(
                            "invalid strict V3 YAML in {}: document terminators are forbidden",
                            path.display()
                        )));
                    }
                    let mut single = false;
                    let mut double = false;
                    let mut escaped = false;
                    let bytes = line.as_bytes();
                    for (index, byte) in bytes.iter().copied().enumerate() {
                        if double && escaped {
                            escaped = false;
                            continue;
                        }
                        if double && byte == b'\\' {
                            escaped = true;
                            continue;
                        }
                        if !double && byte == b'\'' {
                            single = !single;
                            continue;
                        }
                        if !single && byte == b'"' {
                            double = !double;
                            continue;
                        }
                        if !single && !double {
                            if byte == b'#' {
                                break;
                            }
                            if byte == b'!' {
                                return Err(config_model::user_error(format!(
                                    "invalid strict V3 YAML in {}: tags are forbidden",
                                    path.display()
                                )));
                            }
                            if matches!(byte, b'&' | b'*')
                                && (index == 0
                                    || bytes[index - 1].is_ascii_whitespace()
                                    || matches!(bytes[index - 1], b':' | b'[' | b'{' | b','))
                            {
                                return Err(config_model::user_error(format!(
                                    "invalid strict V3 YAML in {}: anchors and aliases are forbidden",
                                    path.display()
                                )));
                            }
                        }
                    }
                    if !single && !double {
                        let key = trimmed.split_once(':').map(|(key, _)| key.trim());
                        if key == Some("<<") || trimmed.starts_with('?') {
                            return Err(config_model::user_error(format!(
                                "invalid strict V3 YAML in {}: merge and non-string keys are forbidden",
                                path.display()
                            )));
                        }
                    }
                }
                let value: serde_yaml::Value = serde_yaml::from_str(raw).map_err(|err| {
                    config_model::user_error(format!(
                        "invalid strict V3 YAML in {}: {}",
                        path.display(),
                        err.to_string().trim()
                    ))
                })?;
                fn validate_value(path: &Path, value: &serde_yaml::Value) -> Result<()> {
                    match value {
                        serde_yaml::Value::Mapping(mapping) => {
                            for (key, value) in mapping {
                                if !matches!(key, serde_yaml::Value::String(_)) {
                                    return Err(config_model::user_error(format!(
                                        "invalid strict V3 YAML in {}: mapping keys must be strings",
                                        path.display()
                                    )));
                                }
                                validate_value(path, value)?;
                            }
                        }
                        serde_yaml::Value::Sequence(values) => {
                            for value in values {
                                validate_value(path, value)?;
                            }
                        }
                        serde_yaml::Value::Tagged(_) => {
                            return Err(config_model::user_error(format!(
                                "invalid strict V3 YAML in {}: tags are forbidden",
                                path.display()
                            )))
                        }
                        _ => {}
                    }
                    Ok(())
                }
                validate_value(path, &value)
            }

            fn source_material(
                path: &Path,
                raw: &str,
                scope: &str,
                root: &Path,
            ) -> Result<AgentInventorySourceMaterialV1> {
                #[cfg(target_os = "linux")]
                {
                    use std::ffi::CString;
                    use std::fs::OpenOptions;
                    use std::io::Read;
                    use std::mem::MaybeUninit;
                    use std::os::fd::{AsFd, AsRawFd, FromRawFd};
                    use std::os::unix::ffi::OsStrExt;
                    use std::os::unix::fs::{MetadataExt, OpenOptionsExt};

                    if !matches!(scope, "global" | "workspace") || !root.is_absolute() {
                        return Err(config_model::user_error(
                            "invalid authenticated inventory source context",
                        ));
                    }
                    let relative_path = path
                        .strip_prefix(root)
                        .map_err(|_| {
                            config_model::user_error(
                                "inventory source is outside its authenticated root",
                            )
                        })?
                        .to_str()
                        .ok_or_else(|| {
                            config_model::user_error("invalid inventory source relative path")
                        })?
                        .to_string();
                    let root_file = OpenOptions::new()
                        .read(true)
                        .custom_flags(libc::O_DIRECTORY | libc::O_CLOEXEC | libc::O_NOFOLLOW)
                        .open(root)
                        .map_err(|err| {
                            config_model::user_error(format!(
                                "failed to open inventory root {}: {err}",
                                root.display()
                            ))
                        })?;
                    let accepted_root =
                        config_projection::CanonicalDirectoryV1::capture_linux_from_fd(
                            root_file.as_fd(),
                        )
                        .map_err(|error| config_model::user_error(error.to_string()))?;

                    let relative = Path::new(&relative_path);
                    let relative_components = relative.components().collect::<Vec<_>>();
                    if relative_components.len() < 2
                        || relative_components
                            .iter()
                            .any(|component| !matches!(component, std::path::Component::Normal(_)))
                    {
                        return Err(config_model::user_error(
                            "invalid inventory source relative path",
                        ));
                    }
                    let mut directory = root_file;
                    for component in &relative_components[..relative_components.len() - 1] {
                        let std::path::Component::Normal(name) = component else {
                            unreachable!("relative components were validated")
                        };
                        let name = CString::new(name.as_bytes()).map_err(|_| {
                            config_model::user_error("invalid inventory directory component")
                        })?;
                        // SAFETY: `directory` is live, `name` is NUL-terminated, and the
                        // returned descriptor is immediately placed under `File` ownership.
                        let fd = unsafe {
                            libc::openat(
                                directory.as_raw_fd(),
                                name.as_ptr(),
                                libc::O_RDONLY
                                    | libc::O_DIRECTORY
                                    | libc::O_CLOEXEC
                                    | libc::O_NOFOLLOW,
                            )
                        };
                        if fd < 0 {
                            return Err(config_model::user_error(format!(
                                "failed to open inventory directory component: {}",
                                std::io::Error::last_os_error()
                            )));
                        }
                        // SAFETY: `openat` returned this uniquely owned descriptor.
                        let opened = unsafe { std::fs::File::from_raw_fd(fd) };
                        let metadata = opened.metadata().map_err(|error| {
                            config_model::user_error(format!(
                                "failed to inspect inventory directory: {error}"
                            ))
                        })?;
                        if !metadata.is_dir() || metadata.dev() != directory.metadata()?.dev() {
                            return Err(config_model::user_error(
                                "inventory directory traversal changed filesystem",
                            ));
                        }
                        directory = opened;
                    }
                    let std::path::Component::Normal(file_name) =
                        relative_components[relative_components.len() - 1]
                    else {
                        unreachable!("relative components were validated")
                    };
                    let file_name = CString::new(file_name.as_bytes())
                        .map_err(|_| config_model::user_error("invalid inventory filename"))?;
                    // SAFETY: the retained parent descriptor and C string are valid for
                    // this call; ownership transfers to `source_file` on success.
                    let source_fd = unsafe {
                        libc::openat(
                            directory.as_raw_fd(),
                            file_name.as_ptr(),
                            libc::O_RDONLY | libc::O_CLOEXEC | libc::O_NOFOLLOW,
                        )
                    };
                    if source_fd < 0 {
                        return Err(config_model::user_error(format!(
                            "failed to open inventory source {}: {}",
                            path.display(),
                            std::io::Error::last_os_error()
                        )));
                    }
                    // SAFETY: `openat` returned this uniquely owned descriptor.
                    let mut source_file = unsafe { std::fs::File::from_raw_fd(source_fd) };
                    let before = source_file.metadata().map_err(|error| {
                        config_model::user_error(format!(
                            "failed to inspect inventory source {}: {error}",
                            path.display()
                        ))
                    })?;
                    if !before.is_file()
                        || before.nlink() != 1
                        || before.dev() != directory.metadata()?.dev()
                    {
                        return Err(config_model::user_error(format!(
                            "invalid inventory source {}: expected one regular same-filesystem file",
                            path.display()
                        )));
                    }
                    let mut descriptor_bytes = Vec::new();
                    source_file
                        .read_to_end(&mut descriptor_bytes)
                        .map_err(|error| {
                            config_model::user_error(format!(
                                "failed to read inventory source {}: {error}",
                                path.display()
                            ))
                        })?;
                    let after = source_file.metadata().map_err(|error| {
                        config_model::user_error(format!(
                            "failed to revalidate inventory source {}: {error}",
                            path.display()
                        ))
                    })?;
                    let stable_metadata = |metadata: &std::fs::Metadata| {
                        (
                            metadata.dev(),
                            metadata.ino(),
                            metadata.mode(),
                            metadata.nlink(),
                            metadata.len(),
                            metadata.uid(),
                            metadata.mtime(),
                            metadata.mtime_nsec(),
                            metadata.ctime(),
                            metadata.ctime_nsec(),
                        )
                    };
                    if stable_metadata(&before) != stable_metadata(&after)
                        || descriptor_bytes != raw.as_bytes()
                    {
                        return Err(config_model::user_error(format!(
                            "inventory source {} changed during authenticated read",
                            path.display()
                        )));
                    }
                    let mut named = MaybeUninit::<libc::stat>::uninit();
                    // SAFETY: `named` points to writable storage and `file_name` is valid;
                    // `AT_SYMLINK_NOFOLLOW` binds the named entry rather than its target.
                    let named_result = unsafe {
                        libc::fstatat(
                            directory.as_raw_fd(),
                            file_name.as_ptr(),
                            named.as_mut_ptr(),
                            libc::AT_SYMLINK_NOFOLLOW,
                        )
                    };
                    if named_result != 0 {
                        return Err(config_model::user_error(format!(
                            "failed to revalidate inventory source {}: {}",
                            path.display(),
                            std::io::Error::last_os_error()
                        )));
                    }
                    // SAFETY: successful `fstatat` initialized the structure.
                    let named = unsafe { named.assume_init() };
                    if named.st_dev != before.dev()
                        || named.st_ino != before.ino()
                        || named.st_mode != before.mode()
                        || named.st_nlink != before.nlink()
                        || named.st_size as u64 != before.len()
                        || named.st_uid != before.uid()
                        || named.st_mtime != before.mtime()
                        || named.st_mtime_nsec != before.mtime_nsec()
                        || named.st_ctime != before.ctime()
                        || named.st_ctime_nsec != before.ctime_nsec()
                    {
                        return Err(config_model::user_error(format!(
                            "inventory source {} was replaced after authenticated read",
                            path.display()
                        )));
                    }
                    let raw_bytes_sha256 = format!("{:x}", Sha256::digest(&descriptor_bytes));
                    let mut source = AgentInventorySourceMaterialV1 {
                        inventory_scope: scope.to_string(),
                        accepted_root,
                        relative_path,
                        file_device_id: before.dev(),
                        file_inode: before.ino(),
                        byte_length: descriptor_bytes.len() as u64,
                        raw_bytes_sha256: raw_bytes_sha256.clone(),
                        source_revision: format!("aisr1_{raw_bytes_sha256}"),
                        source_hash: String::new(),
                    };
                    let mut value = serde_json::to_value(&source)
                        .map_err(|_| config_model::user_error("invalid inventory source"))?;
                    value.as_object_mut().unwrap().remove("source_hash");
                    source.source_hash = config_projection::ConfigProjectionCodecV1::domain_sha256(
                        "substrate.e3.agent-inventory-source.v1",
                        &serde_json::json!({"source": value}),
                    )
                    .map_err(|error| config_model::user_error(error.to_string()))?;
                    Ok(source)
                }
                #[cfg(not(target_os = "linux"))]
                {
                    let _ = (path, raw);
                    Err(config_model::user_error(
                        "agent inventory schema_version 3 is unsupported on this platform",
                    ))
                }
            }

            validate_strict_v3_yaml(path, raw)?;
            let parsed: AgentFileV3 = serde_yaml::from_str(raw).map_err(|err| {
                config_model::user_error(format!(
                    "invalid YAML in {}: {}",
                    path.display(),
                    err.to_string().trim()
                ))
            })?;
            validate_agent_schema_v3(path, &parsed, base_policy, world_root)?;
            Ok(ParsedAgentInventoryFile::V3(
                parsed,
                source_material(path, raw, source_scope, accepted_root)?,
            ))
        }
        version => Err(config_model::user_error(format!(
            "invalid agent file in {}: version must be 1, 2, or 3 (got {})",
            path.display(),
            version
        ))),
    }
}

fn discover_agent_inventory_roots(cwd: &Path) -> Result<Vec<PathBuf>> {
    let mut roots = vec![substrate_paths::substrate_home()?.join("agents")];
    if let Some(workspace_root) = workspace::find_workspace_root(cwd) {
        roots.push(
            workspace_root
                .join(workspace::SUBSTRATE_DIR_NAME)
                .join("agents"),
        );
    }
    Ok(roots)
}

fn authoritative_inventory_world_root(cwd: &Path) -> PathBuf {
    workspace::find_workspace_root(cwd).unwrap_or_else(|| cwd.to_path_buf())
}

fn collect_agent_files_in_root(dir: &Path) -> Result<Vec<PathBuf>> {
    let mut entries = Vec::new();
    for entry in fs::read_dir(dir).map_err(|err| {
        config_model::user_error(format!("failed to read {}: {err}", dir.display()))
    })? {
        let entry = entry.map_err(|err| {
            config_model::user_error(format!("failed to read {}: {err}", dir.display()))
        })?;
        let path = entry.path();
        if is_yaml_file(&path) && path.is_file() {
            entries.push(path);
        }
    }

    entries.sort();
    Ok(entries)
}

fn collect_agent_files(dir: &Path, out: &mut Vec<PathBuf>) -> Result<()> {
    out.extend(collect_agent_files_in_root(dir)?);
    Ok(())
}

fn is_yaml_file(path: &Path) -> bool {
    matches!(path.extension().and_then(OsStr::to_str), Some("yaml"))
}

fn validate_agent_schema(
    path: &Path,
    parsed: &AgentFileV1,
    base_policy: &Policy,
    world_root: Option<&Path>,
) -> Result<()> {
    if parsed.version != 1 {
        return Err(config_model::user_error(format!(
            "invalid agent file in {}: version must be 1 (got {})",
            path.display(),
            parsed.version
        )));
    }

    let expected_id = path.file_stem().and_then(OsStr::to_str).ok_or_else(|| {
        config_model::user_error(format!(
            "invalid agent filename in {}: expected a .yaml filename",
            path.display()
        ))
    })?;
    if parsed.id != expected_id {
        return Err(config_model::user_error(format!(
            "invalid agent file in {}: id '{}' must match filename '{}.yaml'",
            path.display(),
            parsed.id,
            expected_id
        )));
    }

    validate_agent_config(path, &parsed.config)?;
    if let Some(overlay) = &parsed.policy_overlay {
        validate_policy_overlay(path, overlay, base_policy, world_root)?;
    }

    Ok(())
}

fn detect_agent_inventory_version(path: &Path, raw: &str) -> Result<u32> {
    let version_probe: AgentFileVersionProbe = serde_yaml::from_str(raw).map_err(|err| {
        config_model::user_error(format!(
            "invalid YAML in {}: {}",
            path.display(),
            err.to_string().trim()
        ))
    })?;
    Ok(version_probe.version)
}

fn validate_agent_schema_v2(
    path: &Path,
    parsed: &AgentFileV2,
    base_policy: &Policy,
    world_root: Option<&Path>,
) -> Result<()> {
    if parsed.version != 2 {
        return Err(config_model::user_error(format!(
            "invalid agent file in {}: version must be 2 (got {})",
            path.display(),
            parsed.version
        )));
    }

    let expected_id = path.file_stem().and_then(OsStr::to_str).ok_or_else(|| {
        config_model::user_error(format!(
            "invalid agent filename in {}: expected a .yaml filename",
            path.display()
        ))
    })?;
    if parsed.id != expected_id {
        return Err(config_model::user_error(format!(
            "invalid agent file in {}: id '{}' must match filename '{}.yaml'",
            path.display(),
            parsed.id,
            expected_id
        )));
    }

    if parsed.config.placements.host.is_none() && parsed.config.placements.world.is_none() {
        return Err(config_model::user_error(format!(
            "invalid agent file in {}: config.placements must define at least one placement",
            path.display()
        )));
    }
    let enabled_placements = count_enabled_v2_placements(parsed);
    if parsed.config.enabled && enabled_placements == 0 {
        return Err(config_model::user_error(format!(
            "invalid agent file in {}: config.placements must enable at least one placement",
            path.display(),
        )));
    }

    for (placement, placement_config) in [
        (AgentPlacement::Host, parsed.config.placements.host.as_ref()),
        (
            AgentPlacement::World,
            parsed.config.placements.world.as_ref(),
        ),
    ] {
        let Some(placement_config) = placement_config else {
            continue;
        };
        validate_agent_config(
            path,
            &AgentConfigV1 {
                enabled: placement_config.enabled,
                kind: parsed.config.kind,
                protocol: parsed.config.protocol.clone(),
                execution: AgentExecutionConfigV1 {
                    scope: Some(placement.execution_scope()),
                },
                cli: placement_config.cli.clone(),
                api: placement_config.api.clone(),
                capabilities: placement_config.capabilities.clone(),
            },
        )?;
    }

    if let Some(overlay) = &parsed.policy_overlay {
        validate_policy_overlay(path, overlay, base_policy, world_root)?;
    }

    Ok(())
}

fn validate_agent_schema_v3(
    path: &Path,
    parsed: &AgentFileV3,
    base_policy: &Policy,
    world_root: Option<&Path>,
) -> Result<()> {
    fn validate_runtime_projection(
        path: &Path,
        runtime: &AgentRuntimeProjectionInputV1,
    ) -> Result<()> {
        if runtime.model.is_empty() || runtime.model.trim() != runtime.model {
            return Err(config_model::user_error(format!(
                "invalid agent file in {}: runtime_projection.model must be nonempty and trimmed",
                path.display()
            )));
        }
        let mut previous_server: Option<&[u8]> = None;
        for server in &runtime.mcp_servers {
            if server.server_id.is_empty()
                || previous_server.is_some_and(|previous| server.server_id.as_bytes() <= previous)
            {
                return Err(config_model::user_error(format!(
                    "invalid agent file in {}: runtime_projection.mcp_servers must have unique bytewise-sorted ids",
                    path.display()
                )));
            }
            previous_server = Some(server.server_id.as_bytes());
            let named_values = match &server.transport {
                config_projection::LogicalMcpTransportV1::Stdio { nonsecret_env, .. } => {
                    nonsecret_env
                }
                config_projection::LogicalMcpTransportV1::StreamableHttp {
                    url,
                    nonsecret_headers,
                } => {
                    if url.contains('@') || url.contains('#') {
                        return Err(config_model::user_error(format!(
                            "invalid agent file in {}: MCP URLs may not contain userinfo or fragments",
                            path.display()
                        )));
                    }
                    nonsecret_headers
                }
            };
            let mut previous_name: Option<&[u8]> = None;
            for value in named_values {
                let upper = value.name.to_ascii_uppercase();
                if value.name.is_empty()
                    || previous_name.is_some_and(|previous| value.name.as_bytes() <= previous)
                    || upper.contains("AUTHORIZATION")
                    || upper.contains("COOKIE")
                    || upper.ends_with("_KEY")
                    || upper.ends_with("_TOKEN")
                    || upper.ends_with("_SECRET")
                {
                    return Err(config_model::user_error(format!(
                        "invalid agent file in {}: MCP named values must be sorted, unique, and nonsecret",
                        path.display()
                    )));
                }
                previous_name = Some(value.name.as_bytes());
            }
        }
        let mut previous_feature: Option<&[u8]> = None;
        for feature in &runtime.features {
            if feature.name.is_empty()
                || previous_feature.is_some_and(|previous| feature.name.as_bytes() <= previous)
            {
                return Err(config_model::user_error(format!(
                    "invalid agent file in {}: runtime_projection.features must have unique bytewise-sorted names",
                    path.display()
                )));
            }
            previous_feature = Some(feature.name.as_bytes());
        }
        Ok(())
    }

    if parsed.version != 3 {
        return Err(config_model::user_error(format!(
            "invalid agent file in {}: version must be 3 (got {})",
            path.display(),
            parsed.version
        )));
    }
    let expected_id = path.file_stem().and_then(OsStr::to_str).ok_or_else(|| {
        config_model::user_error(format!(
            "invalid agent filename in {}: expected a .yaml filename",
            path.display()
        ))
    })?;
    if parsed.id != expected_id {
        return Err(config_model::user_error(format!(
            "invalid agent file in {}: id '{}' must match filename '{}.yaml'",
            path.display(),
            parsed.id,
            expected_id
        )));
    }
    if parsed.config.placements.host.is_none() && parsed.config.placements.world.is_none() {
        return Err(config_model::user_error(format!(
            "invalid agent file in {}: config.placements must define at least one placement",
            path.display()
        )));
    }
    let enabled_count = [
        parsed.config.placements.host.as_ref(),
        parsed.config.placements.world.as_ref(),
    ]
    .into_iter()
    .flatten()
    .filter(|placement| placement.enabled)
    .count();
    if parsed.config.enabled && enabled_count == 0 {
        return Err(config_model::user_error(format!(
            "invalid agent file in {}: config.placements must enable at least one placement",
            path.display()
        )));
    }
    for (placement, placement_config) in [
        (AgentPlacement::Host, parsed.config.placements.host.as_ref()),
        (
            AgentPlacement::World,
            parsed.config.placements.world.as_ref(),
        ),
    ] {
        let Some(placement_config) = placement_config else {
            continue;
        };
        validate_agent_config(
            path,
            &AgentConfigV1 {
                enabled: placement_config.enabled,
                kind: parsed.config.kind,
                protocol: parsed.config.protocol.clone(),
                execution: AgentExecutionConfigV1 {
                    scope: Some(placement.execution_scope()),
                },
                cli: placement_config.cli.clone(),
                api: placement_config.api.clone(),
                capabilities: placement_config.capabilities.clone(),
            },
        )?;
        if let Some(runtime) = &placement_config.runtime_projection {
            validate_runtime_projection(path, runtime)?;
        }
    }
    if parsed.config.enabled
        && parsed.config.kind == AgentConfigKind::Cli
        && parsed
            .config
            .placements
            .world
            .as_ref()
            .is_some_and(|placement| {
                placement.enabled
                    && placement
                        .cli
                        .as_ref()
                        .is_some_and(|cli| cli.runtime_family == Some(AgentCliRuntimeFamily::Codex))
            })
    {
        let world = parsed.config.placements.world.as_ref().unwrap();
        let cli = world.cli.as_ref().ok_or_else(|| {
            config_model::user_error(format!(
                "invalid agent file in {}: E3 Codex world placement requires explicit cli",
                path.display()
            ))
        })?;
        let runtime = world.runtime_projection.as_ref().ok_or_else(|| {
            config_model::user_error(format!(
                "invalid agent file in {}: E3 Codex world placement requires runtime_projection",
                path.display()
            ))
        })?;
        if cli.binary != "/var/lib/substrate/world-deps/codex-runtime/bin/codex"
            || cli.mode != Some(config_model::AgentCliMode::Persistent)
            || runtime.model != "codex"
            || !runtime.mcp_servers.is_empty()
            || !runtime.features.is_empty()
        {
            return Err(config_model::user_error(format!(
                "invalid agent file in {}: E3 Codex world placement does not match the admitted V1 descriptor",
                path.display()
            )));
        }
    }
    if let Some(overlay) = &parsed.policy_overlay {
        validate_policy_overlay(path, overlay, base_policy, world_root)?;
    }
    Ok(())
}

fn count_enabled_v2_placements(parsed: &AgentFileV2) -> usize {
    [
        parsed.config.placements.host.as_ref(),
        parsed.config.placements.world.as_ref(),
    ]
    .into_iter()
    .flatten()
    .filter(|placement| placement.enabled)
    .count()
}

fn compatibility_source_from_v2(
    parsed: &AgentFileV2,
) -> Option<(AgentPlacement, &AgentPlacementConfigV2)> {
    let mut enabled = [
        (AgentPlacement::Host, parsed.config.placements.host.as_ref()),
        (
            AgentPlacement::World,
            parsed.config.placements.world.as_ref(),
        ),
    ]
    .into_iter()
    .filter_map(|(placement, config)| config.map(|config| (placement, config)))
    .filter(|(_, placement)| placement.enabled);
    let first = enabled.next()?;
    if enabled.next().is_some() {
        return None;
    }
    Some(first)
}

fn compatibility_inventory_file_from_v2(parsed: &AgentFileV2) -> AgentFileV1 {
    // Packet 1 validates the version-2 schema and exposes placement-aware projection helpers
    // without cutting legacy selector/runtime consumers over to multi-row inventory yet.
    let compatibility_source = compatibility_source_from_v2(parsed);
    let compatibility_id = compatibility_source
        .map(|(placement, _)| match placement {
            AgentPlacement::Host => parsed.id.clone(),
            AgentPlacement::World => format!("{}_world", parsed.id),
        })
        .unwrap_or_else(|| parsed.id.clone());

    AgentFileV1 {
        version: parsed.version,
        id: compatibility_id,
        config: AgentConfigV1 {
            enabled: parsed.config.enabled && compatibility_source.is_some(),
            kind: parsed.config.kind,
            protocol: parsed.config.protocol.clone(),
            execution: AgentExecutionConfigV1 {
                scope: compatibility_source.map(|(placement, _)| placement.execution_scope()),
            },
            cli: compatibility_source.and_then(|(_, placement)| placement.cli.clone()),
            api: compatibility_source.and_then(|(_, placement)| placement.api.clone()),
            capabilities: compatibility_source
                .map(|(_, placement)| placement.capabilities.clone())
                .unwrap_or_default(),
        },
        policy_overlay: parsed.policy_overlay.clone(),
    }
}

fn validate_agent_config(path: &Path, config: &AgentConfigV1) -> Result<()> {
    match config.kind {
        AgentConfigKind::Cli => {
            if config.api.is_some() {
                return Err(config_model::user_error(format!(
                    "invalid agent file in {}: config.api is not permitted when config.kind=cli",
                    path.display()
                )));
            }
            if let Some(cli) = &config.cli {
                if !cli.binary.is_empty()
                    && (cli.binary.trim().is_empty() || cli.binary.trim() != cli.binary)
                {
                    return Err(config_model::user_error(format!(
                        "invalid agent file in {}: config.cli.binary must not include leading or trailing whitespace",
                        path.display()
                    )));
                }
            }
        }
        AgentConfigKind::Api => {
            if config.cli.is_some() {
                return Err(config_model::user_error(format!(
                    "invalid agent file in {}: config.cli is not permitted when config.kind=api",
                    path.display()
                )));
            }
            let Some(api) = &config.api else {
                return Err(config_model::user_error(format!(
                    "invalid agent file in {}: config.api is required when config.kind=api",
                    path.display()
                )));
            };
            validate_https_base_url(path, &api.base_url)?;
            validate_env_name_list(path, &api.auth.env, "config.api.auth.env")?;
            if api.auth.env.is_empty() {
                return Err(config_model::user_error(format!(
                    "invalid agent file in {}: config.api.auth.env must be a non-empty list",
                    path.display()
                )));
            }
        }
    }

    if let Some(protocol) = &config.protocol {
        let trimmed = protocol.trim();
        if trimmed.is_empty() {
            return Err(config_model::user_error(format!(
                "invalid agent file in {}: config.protocol must not be empty",
                path.display()
            )));
        }
        if trimmed != protocol {
            return Err(config_model::user_error(format!(
                "invalid agent file in {}: config.protocol must not include leading or trailing whitespace",
                path.display()
            )));
        }
        validate_dotted_id(protocol).map_err(|_| {
            config_model::user_error(format!(
                "invalid agent file in {}: config.protocol '{}' must be a lowercase dotted id",
                path.display(),
                protocol
            ))
        })?;
    }

    let _ = &config.protocol;
    let _ = config.execution.scope;
    let _ = config.cli.as_ref().and_then(|cli| cli.mode);
    let _ = config.cli.as_ref().and_then(|cli| cli.runtime_family);
    let _ = config.enabled;
    let _ = config.capabilities.session_start;
    let _ = config.capabilities.session_resume;
    let _ = config.capabilities.session_fork;
    let _ = config.capabilities.session_stop;
    let _ = config.capabilities.status_snapshot;
    let _ = config.capabilities.event_stream;
    let _ = config.capabilities.llm;
    let _ = config.capabilities.mcp_client;

    Ok(())
}

fn validate_https_base_url(path: &Path, raw: &str) -> Result<()> {
    let url = url::Url::parse(raw).map_err(|err| {
        config_model::user_error(format!(
            "invalid agent file in {}: config.api.base_url '{}' is not a valid URL: {}",
            path.display(),
            raw.trim(),
            err
        ))
    })?;

    if url.scheme() != "https" {
        return Err(config_model::user_error(format!(
            "invalid agent file in {}: config.api.base_url '{}' must use https",
            path.display(),
            raw.trim()
        )));
    }
    if !url.username().is_empty() || url.password().is_some() {
        return Err(config_model::user_error(format!(
            "invalid agent file in {}: config.api.base_url '{}' must not include userinfo",
            path.display(),
            raw.trim()
        )));
    }
    if url.query().is_some() {
        return Err(config_model::user_error(format!(
            "invalid agent file in {}: config.api.base_url '{}' must not include a query string",
            path.display(),
            raw.trim()
        )));
    }
    if url.fragment().is_some() {
        return Err(config_model::user_error(format!(
            "invalid agent file in {}: config.api.base_url '{}' must not include a fragment",
            path.display(),
            raw.trim()
        )));
    }
    if url.host_str().is_none() {
        return Err(config_model::user_error(format!(
            "invalid agent file in {}: config.api.base_url '{}' must include a host",
            path.display(),
            raw.trim()
        )));
    }

    Ok(())
}

fn validate_env_name_list(path: &Path, values: &[String], key: &str) -> Result<()> {
    for value in values {
        let trimmed = value.trim();
        if trimmed.is_empty()
            || trimmed != value
            || trimmed.contains('=')
            || trimmed.chars().any(char::is_whitespace)
        {
            return Err(config_model::user_error(format!(
                "invalid agent file in {}: {} entry '{}' must be an environment variable name without values",
                path.display(),
                key,
                value
            )));
        }
    }
    Ok(())
}

fn validate_policy_overlay(
    path: &Path,
    overlay: &crate::execution::policy_model::PolicyPatch,
    base_policy: &Policy,
    world_root: Option<&Path>,
) -> Result<()> {
    if overlay.id.is_some() {
        return Err(config_model::user_error(format!(
            "invalid agent file in {}: policy_overlay.id is not permitted",
            path.display()
        )));
    }
    if overlay.name.is_some() {
        return Err(config_model::user_error(format!(
            "invalid agent file in {}: policy_overlay.name is not permitted",
            path.display()
        )));
    }
    if overlay.llm.require_approval.is_some() {
        return Err(config_model::user_error(format!(
            "invalid agent file in {}: policy_overlay.llm.require_approval is not permitted; use policy_overlay.require_approval instead",
            path.display()
        )));
    }
    if overlay.llm.allowed_backends.is_some() {
        return Err(config_model::user_error(format!(
            "invalid agent file in {}: policy_overlay.llm.allowed_backends is not permitted",
            path.display()
        )));
    }
    if overlay.agents.allowed_backends.is_some() {
        return Err(config_model::user_error(format!(
            "invalid agent file in {}: policy_overlay.agents.allowed_backends is not permitted",
            path.display()
        )));
    }
    if overlay.workflow.router.enabled.is_some()
        || overlay.workflow.router.allow_cross_workspace.is_some()
        || overlay.workflow.router.allowed_rule_ids.is_some()
        || overlay.workflow.router.allowed_workflow_ids.is_some()
        || overlay
            .workflow
            .router
            .allowed_target_workspace_ids
            .is_some()
    {
        return Err(config_model::user_error(format!(
            "invalid agent file in {}: policy_overlay.workflow is not permitted",
            path.display()
        )));
    }
    if overlay.allow_shell_operators.is_some() {
        return Err(config_model::user_error(format!(
            "invalid agent file in {}: policy_overlay.allow_shell_operators is not permitted",
            path.display()
        )));
    }
    if overlay.metadata.is_some() {
        return Err(config_model::user_error(format!(
            "invalid agent file in {}: policy_overlay.metadata is not permitted",
            path.display()
        )));
    }

    if let Some(values) = overlay.llm.secrets.env_allowed.as_deref() {
        validate_env_name_list(path, values, "policy_overlay.llm.secrets.env_allowed")?;
    }
    validate_world_fs_overlay(path, overlay, base_policy, world_root)?;
    validate_overlay_subset(
        path,
        "policy_overlay.agents.host_credentials.read.allowed_backends",
        overlay
            .agents
            .host_credentials
            .read
            .allowed_backends
            .as_deref(),
        Some(&base_policy.agents_host_credentials_read_allowed_backends),
    )?;
    validate_overlay_subset(
        path,
        "policy_overlay.llm.secrets.env_allowed",
        overlay.llm.secrets.env_allowed.as_deref(),
        Some(&base_policy.llm_secrets_env_allowed),
    )?;

    Ok(())
}

fn validate_world_fs_overlay(
    path: &Path,
    overlay: &crate::execution::policy_model::PolicyPatch,
    base_policy: &Policy,
    world_root: Option<&Path>,
) -> Result<()> {
    let dimension = |value: &crate::execution::policy_model::WorldFsDimensionPatch| {
        if value.allow_list.is_none() && value.deny_list.is_none() {
            None
        } else {
            Some(substrate_broker::RestrictedWorldFsDimensionPatchV1 {
                allow_list: value.allow_list.clone(),
                deny_list: value.deny_list.clone(),
            })
        }
    };
    let write = &overlay.world_fs.write;
    let patch = substrate_broker::RestrictedWorldFsPatchV1 {
        host_visible: overlay.world_fs.host_visible,
        fail_closed_routing: overlay.world_fs.fail_closed.routing,
        deny_enforcement: overlay.world_fs.deny_enforcement,
        caged_required: overlay.world_fs.caged_required,
        discover: dimension(&overlay.world_fs.discover),
        read: dimension(&overlay.world_fs.read),
        write: if write.enabled.is_none() && write.allow_list.is_none() && write.deny_list.is_none()
        {
            None
        } else {
            Some(substrate_broker::RestrictedWorldFsWritePatchV1 {
                enabled: write.enabled,
                allow_list: write.allow_list.clone(),
                deny_list: write.deny_list.clone(),
            })
        },
    };
    if patch.is_empty() {
        return Ok(());
    }
    let world_root = world_root.ok_or_else(|| {
        config_model::user_error(format!(
            "invalid agent file in {}: policy_overlay.world_fs requires an authoritative project or workspace root",
            path.display()
        ))
    })?;
    substrate_broker::resolve_restricted_world_fs_narrowing(base_policy, &patch, world_root)
        .map(|_| ())
        .map_err(|error| {
            config_model::user_error(format!(
                "invalid agent file in {}: policy_overlay.world_fs broadens beyond the effective base policy or cannot be proven: {error}",
                path.display()
            ))
        })
}

fn inventory_workspace_root_from_path(path: &Path) -> Option<PathBuf> {
    path.parent()
        .filter(|parent| parent.file_name().is_some_and(|name| name == "agents"))
        .and_then(Path::parent)
        .filter(|parent| {
            parent
                .file_name()
                .is_some_and(|name| name == workspace::SUBSTRATE_DIR_NAME)
                && parent.join(workspace::WORKSPACE_MARKER_FILENAME).is_file()
        })
        .and_then(Path::parent)
        .map(Path::to_path_buf)
}

fn validate_overlay_subset(
    path: &Path,
    key: &str,
    values: Option<&[String]>,
    allowed: Option<&[String]>,
) -> Result<()> {
    if let Some(values) = values {
        if let Some(allowed) = allowed {
            for value in values {
                if !allowed.iter().any(|candidate| candidate == value) {
                    return Err(config_model::user_error(format!(
                        "invalid agent file in {}: {} entry '{}' broadens beyond the effective base policy",
                        path.display(),
                        key,
                        value
                    )));
                }
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{
        compatibility_inventory_file_from_v2, inventory_entry_origin,
        load_effective_agent_inventory, load_effective_agent_inventory_for_bootstrap_home,
        materialize_effective_inventory_entries_from_v2, merge_inventory_root,
        normalize_inventory_origin_path, parse_and_validate_agent_file,
        parse_and_validate_agent_file_raw, project_inventory_entry, project_inventory_v2_entry,
        project_inventory_v3_entry, validate_agent_file, validate_agent_schema_v2,
        validate_world_fs_overlay, AgentCapabilitiesV1, AgentCliConfigV1, AgentCliRuntimeFamily,
        AgentConfigKind, AgentConfigV1, AgentExecutionConfigV1, AgentFileV1, AgentFileV2,
        AgentInventoryBaselineOrigin, AgentInventoryEntryV1, AgentPlacement,
        ParsedAgentInventoryFile,
    };
    use crate::execution::config_model::{AgentCliMode, SubstrateConfig};
    use crate::execution::workspace::{workspace_marker_path, SUBSTRATE_DIR_NAME};
    use std::collections::{BTreeMap, BTreeSet};
    use std::path::{Path, PathBuf};

    fn e3c_v3_codex_yaml() -> &'static str {
        r#"version: 3
id: codex
config:
  enabled: true
  kind: cli
  protocol: substrate.agent.session
  placements:
    host:
      enabled: true
      cli:
        binary: codex
        mode: persistent
        runtime_family: codex
      api: null
      capabilities:
        session_start: true
      runtime_projection: null
    world:
      enabled: true
      cli:
        binary: /var/lib/substrate/world-deps/codex-runtime/bin/codex
        mode: persistent
        runtime_family: codex
      api: null
      capabilities:
        session_start: true
        session_resume: true
        session_fork: true
        session_stop: true
        status_snapshot: true
        event_stream: true
        llm: true
        mcp_client: false
      runtime_projection:
        model: codex
        mcp_servers: []
        features: []
"#
    }

    fn e1_inventory_parent(root: &Path) -> Policy {
        fs::create_dir_all(root.join("src")).unwrap();
        fs::write(root.join("src/lib.rs"), "lib").unwrap();
        fs::write(root.join("src/sibling.rs"), "sibling").unwrap();
        let mut parent = Policy::default();
        parent.world_fs_host_visible = false;
        parent.world_fs_fail_closed_routing = true;
        parent.world_fs_write_enabled = false;
        parent.world_fs_read = Some(substrate_broker::WorldFsDimensionPolicy {
            allow_list: vec!["src".to_string()],
            deny_list: Vec::new(),
        });
        parent.world_fs_discover = parent.world_fs_read.clone();
        parent.world_fs_write = None;
        parent
    }

    fn e1_inventory_agent_path(root: &Path) -> PathBuf {
        let substrate_dir = root.join(SUBSTRATE_DIR_NAME);
        fs::create_dir_all(substrate_dir.join("agents")).unwrap();
        fs::write(workspace_marker_path(root), "{}\n").unwrap();
        substrate_dir.join("agents/e1.yaml")
    }

    #[test]
    fn e1_inventory_uses_broker_directory_to_child_containment() {
        let root = tempfile::tempdir().unwrap();
        let path = e1_inventory_agent_path(root.path());
        let parent = e1_inventory_parent(root.path());
        let mut overlay = crate::execution::policy_model::PolicyPatch::default();
        overlay.world_fs.read.allow_list = Some(vec!["src/lib.rs".to_string()]);
        validate_world_fs_overlay(&path, &overlay, &parent, Some(root.path()))
            .expect("directory authority narrows to child file");

        overlay.world_fs.read.allow_list = Some(vec!["outside.txt".to_string()]);
        assert!(validate_world_fs_overlay(&path, &overlay, &parent, Some(root.path())).is_err());
    }

    #[cfg(unix)]
    #[test]
    fn e1_inventory_reuses_broker_symlink_escape_rejection() {
        use std::os::unix::fs::symlink;
        let root = tempfile::tempdir().unwrap();
        let outside = tempfile::tempdir().unwrap();
        let path = e1_inventory_agent_path(root.path());
        let parent = e1_inventory_parent(root.path());
        symlink(outside.path(), root.path().join("src/escape")).unwrap();
        let mut overlay = crate::execution::policy_model::PolicyPatch::default();
        overlay.world_fs.read.allow_list = Some(vec!["src/escape/secret".to_string()]);
        assert!(validate_world_fs_overlay(&path, &overlay, &parent, Some(root.path())).is_err());
    }

    #[cfg(unix)]
    #[test]
    #[serial_test::serial]
    fn e1_global_inventory_uses_authoritative_project_root_not_process_cwd() {
        use std::os::unix::fs::symlink;

        struct CurrentDirGuard(PathBuf);
        impl Drop for CurrentDirGuard {
            fn drop(&mut self) {
                std::env::set_current_dir(&self.0).expect("restore process cwd");
            }
        }

        let _authority_env = crate::execution::AuthorityEnvTestGuard::preserve();
        let root = tempfile::tempdir().unwrap();
        let project = root.path().join("project");
        let process_cwd = root.path().join("ambient-cwd");
        let external = root.path().join("external");
        let substrate_home = root.path().join("substrate-home");
        fs::create_dir_all(project.join("src")).unwrap();
        fs::write(project.join("src/lib.rs"), "project").unwrap();
        fs::create_dir_all(process_cwd.join("src")).unwrap();
        fs::create_dir_all(&external).unwrap();
        fs::write(external.join("secret"), "secret").unwrap();
        symlink(external.join("secret"), process_cwd.join("src/lib.rs")).unwrap();
        fs::create_dir_all(substrate_home.join("agents")).unwrap();
        fs::write(
            substrate_home.join("agents/e1.yaml"),
            r#"
version: 1
id: e1
config:
  kind: cli
  enabled: true
  protocol: transport.iframe.v1
  cli:
    binary: e1
policy_overlay:
  world_fs:
    read:
      allow_list:
        - src/lib.rs
"#,
        )
        .unwrap();

        let parent = e1_inventory_parent(&project);
        let original_cwd = std::env::current_dir().unwrap();
        let _cwd_guard = CurrentDirGuard(original_cwd);
        std::env::set_current_dir(&process_cwd).unwrap();
        _authority_env.install_home(&substrate_home);

        let inventory = load_effective_agent_inventory(&project, &parent)
            .expect("global inventory must bind containment to the requested project root");
        assert!(inventory.contains_key("e1"));
    }
    use std::fs;
    use substrate_broker::Policy;
    use tempfile::tempdir;

    #[test]
    fn inventory_entry_origin_treats_noncanonical_workspace_paths_as_workspace_inventory() {
        let temp = tempdir().expect("tempdir");
        let workspace_root = temp.path().join("workspace");
        let workspace_agents = workspace_root.join(SUBSTRATE_DIR_NAME).join("agents");
        std::fs::create_dir_all(&workspace_agents).expect("workspace agents");
        std::fs::write(workspace_marker_path(&workspace_root), "version: 1\n")
            .expect("workspace marker");

        let cwd = workspace_root.join("src");
        std::fs::create_dir_all(&cwd).expect("cwd");
        let entry = AgentInventoryEntryV1 {
            path: workspace_agents.join("codex.yaml"),
            file: AgentFileV1 {
                version: 1,
                id: "codex".to_string(),
                config: AgentConfigV1 {
                    enabled: true,
                    kind: AgentConfigKind::Cli,
                    protocol: Some("transport.iframe.v1".to_string()),
                    execution: AgentExecutionConfigV1::default(),
                    cli: Some(AgentCliConfigV1 {
                        binary: "codex".to_string(),
                        mode: None,
                        runtime_family: None,
                    }),
                    api: None,
                    capabilities: AgentCapabilitiesV1::default(),
                },
                policy_overlay: None,
            },
        };

        assert_eq!(
            inventory_entry_origin(&cwd, &entry),
            AgentInventoryBaselineOrigin::WorkspaceInventory
        );
    }

    #[test]
    fn project_inventory_entry_carries_cli_runtime_family_truth() {
        let cwd = tempdir().expect("tempdir");
        let mut effective_config = SubstrateConfig::default();
        effective_config.agents.defaults.cli.mode = AgentCliMode::Persistent;

        let entry = AgentInventoryEntryV1 {
            path: cwd.path().join("codex_world.yaml"),
            file: AgentFileV1 {
                version: 1,
                id: "codex_world".to_string(),
                config: AgentConfigV1 {
                    enabled: true,
                    kind: AgentConfigKind::Cli,
                    protocol: Some("substrate.agent.session".to_string()),
                    execution: AgentExecutionConfigV1::default(),
                    cli: Some(AgentCliConfigV1 {
                        binary: "codex".to_string(),
                        mode: None,
                        runtime_family: Some(AgentCliRuntimeFamily::Codex),
                    }),
                    api: None,
                    capabilities: AgentCapabilitiesV1::default(),
                },
                policy_overlay: None,
            },
        };

        let projected = project_inventory_entry(cwd.path(), &entry, &effective_config);

        assert_eq!(
            projected.cli_runtime_family,
            Some(AgentCliRuntimeFamily::Codex)
        );
    }

    #[test]
    fn project_inventory_v2_entry_emits_realized_rows_for_enabled_placements() {
        let cwd = tempdir().expect("tempdir");
        let mut effective_config = SubstrateConfig::default();
        effective_config.agents.defaults.cli.mode = AgentCliMode::Persistent;

        let raw = r#"
version: 2
id: codex
config:
  kind: cli
  enabled: true
  protocol: substrate.agent.session
  placements:
    host:
      enabled: true
      cli:
        binary: codex
        mode: persistent
        runtime_family: codex
      capabilities:
        session_start: true
        llm: true
    world:
      enabled: true
      cli:
        binary: codex
        runtime_family: codex
      capabilities:
        session_resume: true
        llm: true
"#;
        let file: AgentFileV2 = serde_yaml::from_str(raw).expect("v2 inventory");

        let projected = project_inventory_v2_entry(
            cwd.path(),
            &cwd.path().join("codex.yaml"),
            &file,
            &effective_config,
        );

        assert_eq!(projected.len(), 2);

        let host = &projected[0];
        assert_eq!(host.logical_agent_id, "codex");
        assert_eq!(host.placement, AgentPlacement::Host);
        assert_eq!(host.realized_agent_id, "codex-host");
        assert_eq!(host.backend_id, "cli:codex-host");
        assert_eq!(host.display_label, "codex (host)");
        assert_eq!(
            host.execution_scope,
            crate::execution::config_model::AgentExecutionScope::Host
        );
        assert_eq!(host.cli_runtime_family, Some(AgentCliRuntimeFamily::Codex));
        assert_eq!(host.cli_binary.as_deref(), Some("codex"));
        assert!(host.capabilities.session_start);
        assert!(host.capabilities.llm);

        let world = &projected[1];
        assert_eq!(world.logical_agent_id, "codex");
        assert_eq!(world.placement, AgentPlacement::World);
        assert_eq!(world.realized_agent_id, "codex-world");
        assert_eq!(world.backend_id, "cli:codex-world");
        assert_eq!(world.display_label, "codex (world)");
        assert_eq!(
            world.execution_scope,
            crate::execution::config_model::AgentExecutionScope::World
        );
        assert_eq!(world.cli_runtime_family, Some(AgentCliRuntimeFamily::Codex));
        assert_eq!(world.cli_binary.as_deref(), Some("codex"));
        assert!(world.capabilities.session_resume);
        assert!(world.capabilities.llm);
    }

    #[test]
    fn project_inventory_v2_entry_skips_disabled_placements() {
        let cwd = tempdir().expect("tempdir");
        let effective_config = SubstrateConfig::default();
        let raw = r#"
version: 2
id: codex
config:
  kind: cli
  enabled: true
  protocol: substrate.agent.session
  placements:
    host:
      enabled: false
      cli:
        binary: codex
        runtime_family: codex
      capabilities:
        llm: true
    world:
      enabled: true
      cli:
        binary: codex
        runtime_family: codex
      capabilities:
        llm: true
"#;
        let file: AgentFileV2 = serde_yaml::from_str(raw).expect("v2 inventory");

        let projected = project_inventory_v2_entry(
            cwd.path(),
            &cwd.path().join("codex.yaml"),
            &file,
            &effective_config,
        );

        assert_eq!(projected.len(), 1);
        assert_eq!(projected[0].placement, AgentPlacement::World);
        assert_eq!(projected[0].realized_agent_id, "codex-world");
    }

    #[test]
    fn validate_agent_schema_v2_rejects_enabled_inventory_without_enabled_placements() {
        let temp = tempdir().expect("tempdir");
        let path = temp.path().join("codex.yaml");
        let base_policy = Policy::default();

        let raw = r#"
version: 2
id: codex
config:
  kind: cli
  enabled: true
  protocol: substrate.agent.session
  placements:
    host:
      enabled: false
      cli:
        binary: codex
        runtime_family: codex
    world:
      enabled: false
      cli:
        binary: codex
        runtime_family: codex
"#;
        let parsed: AgentFileV2 = serde_yaml::from_str(raw).expect("v2 inventory");
        let err = validate_agent_schema_v2(&path, &parsed, &base_policy, None)
            .expect_err("inventory without enabled placements should fail validation");
        assert!(
            err.to_string()
                .contains("config.placements must enable at least one placement"),
            "unexpected validation error: {err:?}"
        );
    }

    #[test]
    fn validate_agent_schema_v2_accepts_top_level_disabled_inventory_without_enabled_placements() {
        let temp = tempdir().expect("tempdir");
        let path = temp.path().join("codex.yaml");
        let base_policy = Policy::default();

        let raw = r#"
version: 2
id: codex
config:
  kind: cli
  enabled: false
  protocol: substrate.agent.session
  placements:
    host:
      enabled: false
      cli:
        binary: codex
        runtime_family: codex
    world:
      enabled: false
      cli:
        binary: codex
        runtime_family: codex
"#;
        let parsed: AgentFileV2 = serde_yaml::from_str(raw).expect("v2 inventory");
        validate_agent_schema_v2(&path, &parsed, &base_policy, None)
            .expect("top-level disabled inventory should preserve disabled semantics");
    }

    #[test]
    fn validate_agent_schema_v2_accepts_multi_enabled_inventory_after_packet_2_cutover() {
        let temp = tempdir().expect("tempdir");
        let path = temp.path().join("codex.yaml");
        let base_policy = Policy::default();
        let raw = r#"
version: 2
id: codex
config:
  kind: cli
  enabled: true
  protocol: substrate.agent.session
  placements:
    host:
      enabled: true
      cli:
        binary: codex
        runtime_family: codex
    world:
      enabled: true
      cli:
        binary: codex
        runtime_family: codex
        "#;
        let parsed: AgentFileV2 = serde_yaml::from_str(raw).expect("v2 inventory");
        validate_agent_schema_v2(&path, &parsed, &base_policy, None)
            .expect("Packet 2 should accept multi-enabled placement-aware inventory");
    }

    #[test]
    fn compatibility_inventory_file_from_v2_fails_closed_for_ambiguous_or_disabled_placements() {
        for raw in [
            r#"
version: 2
id: codex
config:
  kind: cli
  enabled: true
  protocol: substrate.agent.session
  placements:
    host:
      enabled: false
      cli:
        binary: codex-host
        runtime_family: codex
      capabilities:
        session_start: true
        llm: true
    world:
      enabled: false
      cli:
        binary: codex-world
        runtime_family: codex
      capabilities:
        session_resume: true
        llm: true
"#,
            r#"
version: 2
id: codex
config:
  kind: cli
  enabled: true
  protocol: substrate.agent.session
  placements:
    host:
      enabled: true
      cli:
        binary: codex-host
        runtime_family: codex
      capabilities:
        session_start: true
        llm: true
    world:
      enabled: true
      cli:
        binary: codex-world
        runtime_family: codex
      capabilities:
        session_resume: true
        llm: true
"#,
        ] {
            let parsed: AgentFileV2 = serde_yaml::from_str(raw).expect("v2 inventory");
            let compatibility = compatibility_inventory_file_from_v2(&parsed);

            assert!(
                !compatibility.config.enabled,
                "ambiguous compatibility projection must fail closed"
            );
            assert!(compatibility.config.cli.is_none());
            assert!(compatibility.config.api.is_none());
            assert_eq!(
                compatibility.config.capabilities,
                AgentCapabilitiesV1::default()
            );
        }
    }

    #[test]
    fn effective_inventory_materialization_accepts_single_placement_version_2_inventory() {
        let raw = r#"
version: 2
id: codex
config:
  kind: cli
  enabled: true
  protocol: substrate.agent.session
  placements:
    world:
      enabled: true
      cli:
        binary: codex
        runtime_family: codex
      capabilities:
        session_resume: true
        llm: true
"#;
        let parsed: AgentFileV2 = serde_yaml::from_str(raw).expect("v2 inventory");
        let path = tempdir().expect("tempdir").path().join("codex.yaml");

        let materialized = materialize_effective_inventory_entries_from_v2(&path, &parsed);

        assert_eq!(materialized.len(), 1);
        let materialized = &materialized[0].file;
        assert!(materialized.config.enabled);
        assert_eq!(materialized.id, "codex-world");
        assert_eq!(materialized.derived_backend_id(), "cli:codex-world");
        assert_eq!(
            materialized.config.execution.scope,
            Some(crate::execution::config_model::AgentExecutionScope::World)
        );
    }

    #[test]
    fn effective_inventory_materialization_realizes_multi_placement_version_2_inventory() {
        let raw = r#"
version: 2
id: codex
config:
  kind: cli
  enabled: true
  protocol: substrate.agent.session
  placements:
    host:
      enabled: true
      cli:
        binary: codex
        runtime_family: codex
      capabilities:
        llm: true
    world:
      enabled: true
      cli:
        binary: /var/lib/substrate/world-deps/bin/codex
        runtime_family: codex
      capabilities:
        llm: true
"#;
        let parsed: AgentFileV2 = serde_yaml::from_str(raw).expect("v2 inventory");
        let path = tempdir().expect("tempdir").path().join("codex.yaml");

        let materialized = materialize_effective_inventory_entries_from_v2(&path, &parsed);

        assert_eq!(materialized.len(), 2);
        assert_eq!(materialized[0].file.id, "codex-host");
        assert_eq!(materialized[0].file.derived_backend_id(), "cli:codex-host");
        assert_eq!(materialized[1].file.id, "codex-world");
        assert_eq!(materialized[1].file.derived_backend_id(), "cli:codex-world");
    }

    #[test]
    fn effective_inventory_materialization_preserves_logical_binary_shorthand_for_realized_v2_rows()
    {
        let raw = r#"
version: 2
id: codex
config:
  kind: cli
  enabled: true
  protocol: substrate.agent.session
  placements:
    host:
      enabled: true
      cli:
        runtime_family: codex
      capabilities:
        llm: true
    world:
      enabled: true
      cli:
        runtime_family: codex
      capabilities:
        llm: true
"#;
        let parsed: AgentFileV2 = serde_yaml::from_str(raw).expect("v2 inventory");
        let path = tempdir().expect("tempdir").path().join("codex.yaml");

        let materialized = materialize_effective_inventory_entries_from_v2(&path, &parsed);

        assert_eq!(materialized.len(), 2);
        assert_eq!(materialized[0].file.id, "codex-host");
        assert_eq!(materialized[0].file.effective_cli_binary(), Some("codex"));
        assert_eq!(materialized[1].file.id, "codex-world");
        assert_eq!(materialized[1].file.effective_cli_binary(), Some("codex"));
    }

    #[test]
    #[serial_test::serial]
    fn load_effective_agent_inventory_materializes_single_placement_version_2_files() {
        let _authority_env = crate::execution::AuthorityEnvTestGuard::preserve();
        let temp = tempdir().expect("tempdir");
        let substrate_home = temp.path().join("substrate-home");
        let agents_dir = substrate_home.join("agents");
        fs::create_dir_all(&agents_dir).expect("create agents directory");
        fs::write(
            agents_dir.join("claude_code.yaml"),
            r#"
version: 1
id: claude_code
config:
  kind: cli
  enabled: true
  protocol: substrate.agent.session
  execution:
    scope: host
  cli:
    binary: claude
    runtime_family: claude_code
  capabilities:
    llm: true
"#,
        )
        .expect("write v1 inventory");
        fs::write(
            agents_dir.join("codex.yaml"),
            r#"
version: 2
id: codex
config:
  kind: cli
  enabled: true
  protocol: substrate.agent.session
  placements:
    world:
      enabled: true
      cli:
        binary: codex
        runtime_family: codex
      capabilities:
        llm: true
"#,
        )
        .expect("write v2 inventory");

        let inventory_result = {
            _authority_env.install_home(&substrate_home);
            load_effective_agent_inventory(temp.path(), &Policy::default())
        };
        let inventory = inventory_result
            .expect("effective inventory should materialize single-placement version 2 files");

        assert_eq!(
            inventory.len(),
            2,
            "legacy v1 plus single-placement v2 entries should materialize"
        );
        assert!(inventory.contains_key("claude_code"));
        let codex = inventory
            .get("codex-world")
            .expect("single-placement version 2 entry should be present");
        assert_eq!(
            codex.file.config.execution.scope,
            Some(crate::execution::config_model::AgentExecutionScope::World),
            "materialized compatibility entry must preserve the selected placement scope"
        );
        assert_eq!(codex.file.derived_backend_id(), "cli:codex-world");
    }

    #[test]
    #[serial_test::serial]
    fn load_effective_agent_inventory_workspace_v2_shadow_materializes_single_placement_entry() {
        let _authority_env = crate::execution::AuthorityEnvTestGuard::preserve();
        let temp = tempdir().expect("tempdir");
        let substrate_home = temp.path().join("substrate-home");
        let global_agents_dir = substrate_home.join("agents");
        fs::create_dir_all(&global_agents_dir).expect("create global agents directory");
        fs::write(
            global_agents_dir.join("codex.yaml"),
            r#"
version: 1
id: codex
config:
  kind: cli
  enabled: true
  protocol: substrate.agent.session
  execution:
    scope: host
  cli:
    binary: codex
    runtime_family: codex
  capabilities:
    llm: true
"#,
        )
        .expect("write global v1 inventory");

        let workspace_root = temp.path().join("workspace");
        let workspace_agents_dir = workspace_root.join(SUBSTRATE_DIR_NAME).join("agents");
        fs::create_dir_all(&workspace_agents_dir).expect("create workspace agents directory");
        fs::write(workspace_marker_path(&workspace_root), "version: 1\n")
            .expect("write workspace marker");
        fs::write(
            workspace_agents_dir.join("codex.yaml"),
            r#"
version: 2
id: codex
config:
  kind: cli
  enabled: true
  protocol: substrate.agent.session
  placements:
    world:
      enabled: true
      cli:
        binary: codex
        runtime_family: codex
      capabilities:
        llm: true
"#,
        )
        .expect("write workspace v2 inventory");

        let inventory_result = {
            _authority_env.install_home(&substrate_home);
            load_effective_agent_inventory(&workspace_root, &Policy::default())
        };
        let inventory = inventory_result
            .expect("effective inventory should materialize the workspace version 2 shadow entry");

        assert!(
            !inventory.contains_key("codex"),
            "workspace version 2 world truth must suppress the stale global host row"
        );
        let codex = inventory.get("codex-world").expect(
            "workspace version 2 inventory should materialize through the realized world id",
        );
        assert_eq!(
            normalize_inventory_origin_path(&codex.path),
            normalize_inventory_origin_path(&workspace_agents_dir.join("codex.yaml")),
            "workspace version 2 inventory must win the shadowing boundary"
        );
        assert_eq!(
            codex.file.config.execution.scope,
            Some(crate::execution::config_model::AgentExecutionScope::World)
        );
        assert_eq!(codex.file.derived_backend_id(), "cli:codex-world");
    }

    #[test]
    #[serial_test::serial]
    fn load_effective_agent_inventory_workspace_host_v2_shadow_suppresses_stale_global_world_sibling(
    ) {
        let _authority_env = crate::execution::AuthorityEnvTestGuard::preserve();
        let temp = tempdir().expect("tempdir");
        let substrate_home = temp.path().join("substrate-home");
        let global_agents_dir = substrate_home.join("agents");
        fs::create_dir_all(&global_agents_dir).expect("create global agents directory");
        fs::write(
            global_agents_dir.join("codex.yaml"),
            r#"
version: 1
id: codex
config:
  kind: cli
  enabled: true
  protocol: substrate.agent.session
  execution:
    scope: host
  cli:
    binary: codex
    runtime_family: codex
  capabilities:
    llm: true
"#,
        )
        .expect("write global host v1 inventory");
        fs::write(
            global_agents_dir.join("codex_world.yaml"),
            r#"
version: 1
id: codex_world
config:
  kind: cli
  enabled: true
  protocol: substrate.agent.session
  execution:
    scope: world
  cli:
    binary: codex
    runtime_family: codex
  capabilities:
    llm: true
"#,
        )
        .expect("write global world v1 inventory");

        let workspace_root = temp.path().join("workspace");
        let workspace_agents_dir = workspace_root.join(SUBSTRATE_DIR_NAME).join("agents");
        fs::create_dir_all(&workspace_agents_dir).expect("create workspace agents directory");
        fs::write(workspace_marker_path(&workspace_root), "version: 1\n")
            .expect("write workspace marker");
        fs::write(
            workspace_agents_dir.join("codex.yaml"),
            r#"
version: 2
id: codex
config:
  kind: cli
  enabled: true
  protocol: substrate.agent.session
  placements:
    host:
      enabled: true
      cli:
        binary: codex
        runtime_family: codex
      capabilities:
        llm: true
"#,
        )
        .expect("write workspace host-only v2 inventory");

        let inventory_result = {
            _authority_env.install_home(&substrate_home);
            load_effective_agent_inventory(&workspace_root, &Policy::default())
        };
        let inventory = inventory_result.expect(
            "effective inventory should materialize the workspace host-only version 2 shadow entry",
        );

        assert!(
            !inventory.contains_key("codex"),
            "workspace version 2 host truth must replace the stale legacy host id"
        );
        let codex = inventory.get("codex-host").expect(
            "workspace version 2 inventory should materialize through the realized host id",
        );
        assert_eq!(
            normalize_inventory_origin_path(&codex.path),
            normalize_inventory_origin_path(&workspace_agents_dir.join("codex.yaml")),
            "workspace version 2 inventory must win the shadowing boundary"
        );
        assert_eq!(
            codex.file.config.execution.scope,
            Some(crate::execution::config_model::AgentExecutionScope::Host)
        );
        assert_eq!(codex.file.derived_backend_id(), "cli:codex-host");
        assert!(
            !inventory.contains_key("codex_world"),
            "workspace host-only version 2 truth must suppress the stale global world row"
        );
    }

    #[test]
    #[serial_test::serial]
    fn load_effective_agent_inventory_workspace_host_v2_shadow_suppresses_lower_root_v2_realized_world_row(
    ) {
        let _authority_env = crate::execution::AuthorityEnvTestGuard::preserve();
        let temp = tempdir().expect("tempdir");
        let substrate_home = temp.path().join("substrate-home");
        let global_agents_dir = substrate_home.join("agents");
        fs::create_dir_all(&global_agents_dir).expect("create global agents directory");
        fs::write(
            global_agents_dir.join("codex.yaml"),
            r#"
version: 2
id: codex
config:
  kind: cli
  enabled: true
  protocol: substrate.agent.session
  placements:
    host:
      enabled: true
      cli:
        binary: codex
        runtime_family: codex
      capabilities:
        llm: true
    world:
      enabled: true
      cli:
        binary: /var/lib/substrate/world-deps/bin/codex
        runtime_family: codex
      capabilities:
        llm: true
"#,
        )
        .expect("write global multi-placement v2 inventory");

        let workspace_root = temp.path().join("workspace");
        let workspace_agents_dir = workspace_root.join(SUBSTRATE_DIR_NAME).join("agents");
        fs::create_dir_all(&workspace_agents_dir).expect("create workspace agents directory");
        fs::write(workspace_marker_path(&workspace_root), "version: 1\n")
            .expect("write workspace marker");
        fs::write(
            workspace_agents_dir.join("codex.yaml"),
            r#"
version: 2
id: codex
config:
  kind: cli
  enabled: true
  protocol: substrate.agent.session
  placements:
    host:
      enabled: true
      cli:
        binary: codex
        runtime_family: codex
      capabilities:
        llm: true
"#,
        )
        .expect("write workspace host-only v2 inventory");

        let inventory_result = {
            _authority_env.install_home(&substrate_home);
            load_effective_agent_inventory(&workspace_root, &Policy::default())
        };
        let inventory = inventory_result.expect(
            "effective inventory should materialize the workspace host-only version 2 shadow",
        );

        assert_eq!(
            inventory.len(),
            1,
            "workspace host-only version 2 shadow should replace the full lower-root logical agent view"
        );
        let codex = inventory.get("codex-host").expect(
            "workspace version 2 inventory should materialize through the realized host id",
        );
        assert_eq!(
            normalize_inventory_origin_path(&codex.path),
            normalize_inventory_origin_path(&workspace_agents_dir.join("codex.yaml")),
            "workspace version 2 inventory must win the shadowing boundary"
        );
        assert!(
            !inventory.contains_key("codex-world"),
            "workspace host-only version 2 truth must suppress the stale lower-root realized world row"
        );
        assert!(
            !inventory.contains_key("codex"),
            "workspace host-only version 2 truth must suppress the stale lower-root legacy host row"
        );
        assert!(
            !inventory.contains_key("codex_world"),
            "workspace host-only version 2 truth must suppress the stale lower-root legacy world row"
        );
    }

    #[test]
    #[serial_test::serial]
    fn load_effective_agent_inventory_same_root_v2_suppresses_stale_legacy_world_sibling() {
        let _authority_env = crate::execution::AuthorityEnvTestGuard::preserve();
        let temp = tempdir().expect("tempdir");
        let substrate_home = temp.path().join("substrate-home");
        let agents_dir = substrate_home.join("agents");
        fs::create_dir_all(&agents_dir).expect("create agents directory");
        fs::write(
            agents_dir.join("codex.yaml"),
            r#"
version: 2
id: codex
config:
  kind: cli
  enabled: true
  protocol: substrate.agent.session
  placements:
    host:
      enabled: true
      cli:
        binary: codex
        runtime_family: codex
      capabilities:
        llm: true
"#,
        )
        .expect("write host-only v2 inventory");
        fs::write(
            agents_dir.join("codex_world.yaml"),
            r#"
version: 1
id: codex_world
config:
  kind: cli
  enabled: true
  protocol: substrate.agent.session
  execution:
    scope: world
  cli:
    binary: codex
    runtime_family: codex
  capabilities:
    llm: true
"#,
        )
        .expect("write stale legacy world inventory");

        let inventory_result = {
            _authority_env.install_home(&substrate_home);
            load_effective_agent_inventory(temp.path(), &Policy::default())
        };
        let inventory = inventory_result.expect(
            "effective inventory should keep the realized single-placement version 2 truth",
        );

        assert!(
            !inventory.contains_key("codex"),
            "effective inventory should no longer retain the legacy host compatibility id"
        );
        let codex = inventory.get("codex-host").expect(
            "host-only version 2 inventory should materialize through the realized host id",
        );
        assert_eq!(codex.path, agents_dir.join("codex.yaml"));
        assert_eq!(
            codex.file.config.execution.scope,
            Some(crate::execution::config_model::AgentExecutionScope::Host)
        );
        assert_eq!(codex.file.derived_backend_id(), "cli:codex-host");
        assert!(
            !inventory.contains_key("codex_world"),
            "single-placement version 2 truth must suppress the stale same-root legacy world sibling"
        );
    }

    #[cfg(any(target_os = "linux", target_os = "macos"))]
    #[test]
    #[serial_test::serial]
    fn gateway_backend_resolution_uses_explicit_bootstrap_home_under_conflicting_ambient_home() {
        use std::os::unix::fs::PermissionsExt;

        let _authority_env = crate::execution::AuthorityEnvTestGuard::preserve();
        let safe_parent = std::env::var_os("XDG_RUNTIME_DIR")
            .map(std::path::PathBuf::from)
            .unwrap_or_else(|| {
                std::path::PathBuf::from(std::env::var_os("HOME").expect("tests require HOME"))
                    .join(".cache")
            });
        fs::create_dir_all(&safe_parent).expect("create safe parent");
        let parent = tempfile::tempdir_in(safe_parent).expect("private fixture parent");
        fs::set_permissions(parent.path(), fs::Permissions::from_mode(0o700))
            .expect("secure fixture parent");
        let selected_home = parent.path().join("selected");
        let agents = selected_home.join("agents");
        fs::create_dir_all(&agents).expect("create selected agents");
        fs::set_permissions(&selected_home, fs::Permissions::from_mode(0o700))
            .expect("secure selected home");
        fs::set_permissions(&agents, fs::Permissions::from_mode(0o700))
            .expect("secure selected agents");
        let selected_agent = agents.join("selected.yaml");
        fs::write(
            &selected_agent,
            r#"version: 1
id: selected
config:
  kind: cli
  enabled: true
  protocol: substrate.agent.session
  execution:
    scope: world
  cli:
    binary: selected
  capabilities:
    llm: true
"#,
        )
        .expect("write selected inventory");
        fs::set_permissions(&selected_agent, fs::Permissions::from_mode(0o600))
            .expect("secure selected inventory");

        let conflicting_home = parent.path().join("conflicting");
        fs::create_dir(&conflicting_home).expect("create conflicting home");
        let result = {
            _authority_env.install_home(&conflicting_home);
            let authority =
                crate::execution::agent_runtime::HostSessionAuthority::open(&selected_home)
                    .expect("open selected authority");
            super::resolve_gateway_backend_inventory_entry_for_bootstrap_home(
                parent.path(),
                "cli:selected",
                &Policy::default(),
                &authority.bootstrap_home(),
            )
        };

        let entry = result.expect("resolve selected inventory");
        assert_eq!(entry.path, selected_agent);
        assert_eq!(entry.derived_backend_id(), "cli:selected");
    }

    #[test]
    fn e3c_v3_strict_yaml_rejects_ambiguous_yaml_features() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().join("home");
        let agents = root.join("agents");
        fs::create_dir_all(&agents).unwrap();
        let path = agents.join("codex.yaml");
        let valid = e3c_v3_codex_yaml();
        fs::write(&path, valid).unwrap();
        parse_and_validate_agent_file_raw(
            &path,
            valid,
            &Policy::default(),
            Some(temp.path()),
            "global",
            &root,
        )
        .expect("strict V3 fixture");

        for invalid in [
            format!("{valid}version: 3\n"),
            valid.replace("id: codex", "id: &agent codex\nalias: *agent"),
            valid.replace("  enabled: true", "  <<: { enabled: true }"),
            valid.replace("model: codex", "model: !custom codex"),
            format!("{valid}---\nversion: 3\n"),
            valid.replace("  protocol:", "  7: rejected\n  protocol:"),
        ] {
            fs::write(&path, &invalid).unwrap();
            assert!(parse_and_validate_agent_file_raw(
                &path,
                &invalid,
                &Policy::default(),
                Some(temp.path()),
                "global",
                &root,
            )
            .is_err());
        }
    }

    #[test]
    fn e3c_v3_non_utf8_and_legacy_downconversion_are_rejected() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().join("home");
        let agents = root.join("agents");
        fs::create_dir_all(&agents).unwrap();
        let path = agents.join("codex.yaml");
        fs::write(&path, e3c_v3_codex_yaml()).unwrap();
        let legacy = validate_agent_file(&path, &Policy::default())
            .expect_err("legacy entry point must reject V3");
        assert!(legacy
            .to_string()
            .contains("unsupported agent schema_version 3"));

        fs::write(&path, [0xff, 0xfe, 0xfd]).unwrap();
        assert!(parse_and_validate_agent_file(
            &path,
            &Policy::default(),
            Some(temp.path()),
            "global",
            &root,
        )
        .is_err());
    }

    #[test]
    fn e3c_v3_projection_retains_only_the_selected_source() {
        let temp = tempfile::tempdir().unwrap();
        let global_root = temp.path().join("global");
        let workspace_root = temp.path().join("workspace");
        fs::create_dir_all(global_root.join("agents")).unwrap();
        fs::create_dir_all(workspace_root.join(".substrate/agents")).unwrap();
        let global_path = global_root.join("agents/codex.yaml");
        let workspace_path = workspace_root.join(".substrate/agents/codex.yaml");
        let global = e3c_v3_codex_yaml();
        let workspace = global.replace("session_start: true", "session_start: false");
        fs::write(&global_path, global).unwrap();
        fs::write(&workspace_path, &workspace).unwrap();
        let global = parse_and_validate_agent_file_raw(
            &global_path,
            global,
            &Policy::default(),
            Some(&workspace_root),
            "global",
            &global_root,
        )
        .unwrap();
        let workspace = parse_and_validate_agent_file_raw(
            &workspace_path,
            &workspace,
            &Policy::default(),
            Some(&workspace_root),
            "workspace",
            &workspace_root,
        )
        .unwrap();
        let (workspace_file, workspace_source) = match &workspace {
            ParsedAgentInventoryFile::V3(file, source) => (file, source),
            _ => panic!("expected V3"),
        };
        let projected = project_inventory_v3_entry(
            &workspace_root,
            &workspace_path,
            workspace_file,
            &SubstrateConfig::default(),
            workspace_source,
        );
        assert_eq!(projected.len(), 2);
        assert!(projected
            .iter()
            .all(|entry| entry.source.source_hash == workspace_source.source_hash));
        let global_source_hash = match global {
            ParsedAgentInventoryFile::V3(_, source) => source.source_hash,
            _ => unreachable!(),
        };
        assert_ne!(global_source_hash, workspace_source.source_hash);

        let mut compatibility = BTreeMap::new();
        let error = merge_inventory_root(
            &mut compatibility,
            false,
            BTreeSet::new(),
            vec![
                (workspace_path.clone(), workspace.clone()),
                (workspace_path, workspace),
            ],
        )
        .expect_err("duplicate logical IDs must fail before selection");
        assert!(error.to_string().contains("ambiguous agent inventory id"));
    }

    #[cfg(target_os = "linux")]
    #[test]
    #[serial_test::serial]
    fn e3c_bootstrap_dot_substrate_global_v3_uses_authenticated_global_root() {
        use std::os::unix::fs::PermissionsExt;

        let _authority_env = crate::execution::AuthorityEnvTestGuard::preserve();
        let safe_parent = std::env::var_os("XDG_RUNTIME_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|| {
                PathBuf::from(std::env::var_os("HOME").expect("tests require HOME")).join(".cache")
            });
        fs::create_dir_all(&safe_parent).expect("create safe parent");
        let parent = tempfile::tempdir_in(safe_parent).expect("private fixture parent");
        fs::set_permissions(parent.path(), fs::Permissions::from_mode(0o700))
            .expect("secure fixture parent");
        let accepted_home = parent.path().join(".substrate");
        let agents = accepted_home.join("agents");
        fs::create_dir_all(&agents).expect("create authenticated agents root");
        fs::set_permissions(&accepted_home, fs::Permissions::from_mode(0o700))
            .expect("secure accepted home");
        fs::set_permissions(&agents, fs::Permissions::from_mode(0o700))
            .expect("secure agents root");
        let inventory_path = agents.join("codex.yaml");
        fs::write(&inventory_path, e3c_v3_codex_yaml()).expect("write V3 inventory");
        fs::set_permissions(&inventory_path, fs::Permissions::from_mode(0o600))
            .expect("secure V3 inventory");

        let authority = crate::execution::agent_runtime::HostSessionAuthority::open(&accepted_home)
            .expect("open authenticated accepted home");
        let error = load_effective_agent_inventory_for_bootstrap_home(
            parent.path(),
            &Policy::default(),
            &authority.bootstrap_home(),
        )
        .expect_err("the legacy loader must reject, not misclassify, authenticated V3");

        assert!(
            error
                .to_string()
                .contains("unsupported agent schema_version 3 in legacy inventory loader"),
            "unexpected authenticated global V3 failure: {error}"
        );
    }

    #[cfg(target_os = "linux")]
    #[test]
    #[serial_test::serial]
    fn e3c_bootstrap_inventory_rejects_accepted_root_substitution() {
        use std::os::unix::fs::PermissionsExt;

        let _authority_env = crate::execution::AuthorityEnvTestGuard::preserve();
        let safe_parent = std::env::var_os("XDG_RUNTIME_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|| {
                PathBuf::from(std::env::var_os("HOME").expect("tests require HOME")).join(".cache")
            });
        fs::create_dir_all(&safe_parent).expect("create safe parent");
        let parent = tempfile::tempdir_in(safe_parent).expect("private fixture parent");
        fs::set_permissions(parent.path(), fs::Permissions::from_mode(0o700))
            .expect("secure fixture parent");
        let accepted_home = parent.path().join("selected");
        fs::create_dir(&accepted_home).expect("create accepted home");
        fs::set_permissions(&accepted_home, fs::Permissions::from_mode(0o700))
            .expect("secure accepted home");
        let authority = crate::execution::agent_runtime::HostSessionAuthority::open(&accepted_home)
            .expect("open authenticated accepted home");

        let displaced = parent.path().join("displaced");
        fs::rename(&accepted_home, &displaced).expect("replace accepted-home name");
        fs::create_dir(&accepted_home).expect("create substituted accepted home");
        fs::set_permissions(&accepted_home, fs::Permissions::from_mode(0o700))
            .expect("secure substituted home");

        let error = load_effective_agent_inventory_for_bootstrap_home(
            parent.path(),
            &Policy::default(),
            &authority.bootstrap_home(),
        )
        .expect_err("accepted-root substitution must fail closed");
        let message = error.to_string();
        assert!(
            message.contains("identity")
                || message.contains("replaced")
                || message.contains("changed"),
            "unexpected accepted-root substitution failure: {message}"
        );
    }

    #[cfg(target_os = "linux")]
    #[test]
    #[serial_test::serial]
    fn e3c_bootstrap_inventory_rejects_accepted_root_ancestor_substitution() {
        use std::os::unix::fs::PermissionsExt;

        let _authority_env = crate::execution::AuthorityEnvTestGuard::preserve();
        let safe_parent = std::env::var_os("XDG_RUNTIME_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|| {
                PathBuf::from(std::env::var_os("HOME").expect("tests require HOME")).join(".cache")
            });
        fs::create_dir_all(&safe_parent).expect("create safe parent");
        let parent = tempfile::tempdir_in(safe_parent).expect("private fixture parent");
        fs::set_permissions(parent.path(), fs::Permissions::from_mode(0o700))
            .expect("secure fixture parent");
        let ancestor = parent.path().join("authority");
        let accepted_home = ancestor.join("selected");
        let agents = accepted_home.join("agents");
        fs::create_dir_all(&agents).expect("create accepted inventory root");
        fs::set_permissions(&ancestor, fs::Permissions::from_mode(0o700)).expect("secure ancestor");
        fs::set_permissions(&accepted_home, fs::Permissions::from_mode(0o700))
            .expect("secure accepted home");
        fs::set_permissions(&agents, fs::Permissions::from_mode(0o700))
            .expect("secure agents root");
        let original_inventory = agents.join("codex.yaml");
        fs::write(&original_inventory, e3c_v3_codex_yaml()).expect("V3 inventory");
        fs::set_permissions(&original_inventory, fs::Permissions::from_mode(0o600))
            .expect("secure V3 inventory");
        let authority = crate::execution::agent_runtime::HostSessionAuthority::open(&accepted_home)
            .expect("open authenticated accepted home");

        fs::rename(&ancestor, parent.path().join("displaced-authority"))
            .expect("replace accepted-home ancestor");
        let substituted_agents = accepted_home.join("agents");
        fs::create_dir_all(&substituted_agents).expect("create substituted inventory root");
        fs::set_permissions(&ancestor, fs::Permissions::from_mode(0o700))
            .expect("secure substituted ancestor");
        fs::set_permissions(&accepted_home, fs::Permissions::from_mode(0o700))
            .expect("secure substituted accepted home");
        fs::set_permissions(&substituted_agents, fs::Permissions::from_mode(0o700))
            .expect("secure substituted agents root");
        let substituted_inventory = substituted_agents.join("codex.yaml");
        fs::write(&substituted_inventory, e3c_v3_codex_yaml()).expect("substituted V3 inventory");
        fs::set_permissions(&substituted_inventory, fs::Permissions::from_mode(0o600))
            .expect("secure substituted V3 inventory");

        assert!(load_effective_agent_inventory_for_bootstrap_home(
            parent.path(),
            &Policy::default(),
            &authority.bootstrap_home(),
        )
        .is_err());
    }
}
