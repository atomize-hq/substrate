use crate::execution::config_model;
use crate::execution::workspace;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};
use substrate_broker::{validate_backend_id, validate_dotted_id, Policy};
use substrate_common::derive_agent_backend_id;
use substrate_common::paths as substrate_paths;

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct AgentFileV1 {
    pub version: u32,
    pub id: String,
    pub config: AgentConfigV1,
    #[serde(default)]
    pub policy_overlay: Option<crate::execution::policy_model::PolicyPatch>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
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

#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq, Serialize)]
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

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(default, deny_unknown_fields)]
pub(crate) struct AgentExecutionConfigV1 {
    #[serde(default)]
    pub scope: Option<crate::execution::config_model::AgentExecutionScope>,
}

#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum AgentCliRuntimeFamily {
    Codex,
    ClaudeCode,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(default, deny_unknown_fields)]
pub(crate) struct AgentCliConfigV1 {
    #[serde(default)]
    pub binary: String,
    #[serde(default)]
    pub mode: Option<crate::execution::config_model::AgentCliMode>,
    #[serde(default)]
    pub runtime_family: Option<AgentCliRuntimeFamily>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct AgentApiConfigV1 {
    pub base_url: String,
    pub auth: AgentApiAuthConfigV1,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
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
pub(crate) struct AgentFileV3 {
    #[serde(skip)]
    pub source: Option<transport_api_types::AgentSelectionSourceV1>,
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
    pub selection: Option<transport_api_types::AgentSelectionTupleV1>,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(crate) struct AgentRuntimeProjectionInputV1 {
    pub mcp_servers: Vec<config_projection::LogicalMcpServerV1>,
    pub features: Vec<config_projection::LogicalFeatureV1>,
}

pub(crate) type AgentInventorySourceMaterialV1 = config_projection::AgentInventorySourceMaterialV1;

#[derive(Debug, Clone, Default, Deserialize, PartialEq, Eq, Serialize)]
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

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct AgentInventoryEntryV1 {
    pub path: PathBuf,
    pub file: AgentFileV1,
}

/// The V1-named entry is an internal flattened runtime descriptor. Inventory
/// wire decoding is V3-only; complete selection lives in this captured snapshot.
#[derive(Clone, Debug)]
pub(crate) struct AgentInventorySnapshotV3 {
    pub entries: BTreeMap<String, AgentInventoryEntryV1>,
    pub definitions: Vec<crate::execution::agent_selection::ConfiguredAgentDefinitionV3>,
}

impl AgentInventorySnapshotV3 {
    pub(crate) fn resolve_selection(
        &self,
        config: &config_model::SubstrateConfig,
        agent: Option<&str>,
        backend: Option<&str>,
        scope: config_model::AgentExecutionScope,
    ) -> Result<transport_api_types::ResolvedAgentSelectionV1> {
        use crate::execution::agent_selection::{
            resolve_agent_selection_v1, AgentSelectionConfigV1, AgentSelectionRequestV1,
        };
        resolve_agent_selection_v1(
            &self.definitions,
            &AgentSelectionConfigV1 {
                preference: Some(&config.agents.defaults.selection),
                preference_source: config.selection_preference_source.as_ref(),
                profiles: &config.selection_profiles,
                sources: &config.selection_sources,
            },
            &AgentSelectionRequestV1 {
                agent_id: agent,
                backend_id: backend,
                placement: match scope {
                    config_model::AgentExecutionScope::Host => {
                        transport_api_types::AgentSelectionPlacementV1::Host
                    }
                    config_model::AgentExecutionScope::World => {
                        transport_api_types::AgentSelectionPlacementV1::World
                    }
                },
            },
        )
        .map_err(config_model::user_error)
    }
}

fn ordinary_inventory_source(
    path: &Path,
    raw: &str,
    scope: &str,
    root: &Path,
) -> Result<transport_api_types::AgentSelectionSourceV1> {
    let relative = path
        .strip_prefix(root)
        .map_err(|_| config_model::user_error("inventory source is outside its declared root"))?
        .to_str()
        .ok_or_else(|| config_model::user_error("inventory source path requires UTF-8"))?;
    config_model::selection_source_from_bytes(
        root,
        relative,
        match scope {
            "global" => transport_api_types::AgentSelectionSourceScopeV1::Global,
            "workspace" => transport_api_types::AgentSelectionSourceScopeV1::Workspace,
            _ => {
                return Err(config_model::user_error(
                    "unsupported inventory source scope",
                ))
            }
        },
        raw.as_bytes(),
    )
}

fn assemble_inventory_snapshot_v3(
    files: Vec<(PathBuf, ParsedAgentInventoryFile)>,
) -> Result<AgentInventorySnapshotV3> {
    use crate::execution::agent_selection::{
        visible_agent_definitions_v3, ConfiguredAgentDefinitionV3, ConfiguredAgentPlacementV3,
    };
    use transport_api_types::{AgentHarnessFamilyV1, AgentSelectionPlacementV1};
    let mut definitions = Vec::new();
    for (_, parsed) in &files {
        let ParsedAgentInventoryFile::V3(file, _) = parsed;
        let mut placements = BTreeMap::new();
        for (placement, config) in [
            (
                AgentSelectionPlacementV1::Host,
                file.config.placements.host.as_ref(),
            ),
            (
                AgentSelectionPlacementV1::World,
                file.config.placements.world.as_ref(),
            ),
        ] {
            let Some(config) = config else { continue };
            let Some(selection) = &config.selection else {
                continue;
            };
            let family = config
                .cli
                .as_ref()
                .and_then(|cli| cli.runtime_family)
                .ok_or_else(|| {
                    config_model::user_error(
                        "selected CLI placement has no explicit runtime_family",
                    )
                })?;
            placements.insert(
                placement,
                ConfiguredAgentPlacementV3 {
                    enabled: config.enabled,
                    runtime_family: match family {
                        AgentCliRuntimeFamily::Codex => AgentHarnessFamilyV1::Codex,
                        AgentCliRuntimeFamily::ClaudeCode => AgentHarnessFamilyV1::ClaudeCode,
                    },
                    selection: selection.clone(),
                },
            );
        }
        definitions.push(ConfiguredAgentDefinitionV3 {
            logical_agent_id: file.id.clone(),
            enabled: file.config.enabled,
            source: file
                .source
                .clone()
                .ok_or_else(|| config_model::user_error("inventory source provenance missing"))?,
            placements,
        });
    }
    let visible = visible_agent_definitions_v3(&definitions).map_err(config_model::user_error)?;
    let mut entries = BTreeMap::new();
    for (path, parsed) in &files {
        let ParsedAgentInventoryFile::V3(file, _) = parsed;
        let selected = visible
            .get(file.id.as_str())
            .ok_or_else(|| config_model::user_error("visible logical agent missing"))?;
        if !file.config.enabled || file.source.as_ref() != Some(&selected.source) {
            continue;
        }
        for (placement, config) in [
            (AgentPlacement::Host, file.config.placements.host.as_ref()),
            (AgentPlacement::World, file.config.placements.world.as_ref()),
        ] {
            let Some(config) = config.filter(|placement| placement.enabled) else {
                continue;
            };
            let mut cli = config.cli.clone();
            if let Some(cli) = &mut cli {
                if cli.binary.trim().is_empty() {
                    cli.binary = file.id.clone();
                }
            }
            let id = format!("{}-{}", file.id, placement.as_str());
            entries.insert(
                id.clone(),
                AgentInventoryEntryV1 {
                    path: path.clone(),
                    file: AgentFileV1 {
                        version: 3,
                        id,
                        config: AgentConfigV1 {
                            enabled: true,
                            kind: file.config.kind,
                            protocol: file.config.protocol.clone(),
                            execution: AgentExecutionConfigV1 {
                                scope: Some(placement.execution_scope()),
                            },
                            cli,
                            api: config.api.clone(),
                            capabilities: config.capabilities.clone(),
                        },
                        policy_overlay: file.policy_overlay.clone(),
                    },
                },
            );
        }
    }
    Ok(AgentInventorySnapshotV3 {
        entries,
        definitions,
    })
}

pub(crate) fn load_agent_inventory_snapshot_v3(
    cwd: &Path,
    policy: &Policy,
) -> Result<AgentInventorySnapshotV3> {
    let world = authoritative_inventory_world_root(cwd);
    let mut files = Vec::new();
    for root in discover_agent_inventory_roots(cwd)? {
        if !root.exists() {
            continue;
        }
        if !root.is_dir() {
            return Err(config_model::user_error(format!(
                "invalid agent inventory directory {}",
                root.display()
            )));
        }
        let workspace =
            inventory_path_origin(cwd, &root) == AgentInventoryBaselineOrigin::WorkspaceInventory;
        let accepted = if workspace {
            root.parent().and_then(Path::parent)
        } else {
            root.parent()
        }
        .ok_or_else(|| config_model::user_error("inventory root has no accepted parent"))?;
        for path in collect_agent_files_in_root(&root)? {
            let parsed = parse_and_validate_agent_file(
                &path,
                policy,
                Some(&world),
                if workspace { "workspace" } else { "global" },
                accepted,
            )?;
            files.push((path, parsed));
        }
    }
    assemble_inventory_snapshot_v3(files)
}

pub(crate) fn load_agent_inventory_snapshot_for_bootstrap_home_v3(
    cwd: &Path,
    policy: &Policy,
    bootstrap: &crate::execution::agent_runtime::OpenedBootstrapHomeV1<'_>,
) -> Result<AgentInventorySnapshotV3> {
    bootstrap
        .revalidate()
        .map_err(|error| config_model::user_error(error.to_string()))?;
    let identity = bootstrap
        .identity()
        .map_err(|error| config_model::user_error(error.to_string()))?;
    let accepted = Path::new(&identity.physical_path);
    let world = authoritative_inventory_world_root(cwd);
    let mut files = Vec::new();
    for (name, bytes) in bootstrap
        .read_agent_inventory_yaml()
        .map_err(|error| config_model::user_error(error.to_string()))?
    {
        let path = accepted.join("agents").join(name);
        let raw = std::str::from_utf8(&bytes).map_err(|_| {
            config_model::user_error(format!("invalid UTF-8 in {}", path.display()))
        })?;
        let parsed = parse_and_validate_agent_file_raw(
            &path,
            raw,
            policy,
            Some(&world),
            "global",
            accepted,
            None,
        )?;
        files.push((path, parsed));
    }
    if let Some(workspace) = workspace::find_workspace_root(cwd) {
        let workspace_agents = workspace.join(".substrate/agents");
        for path in if workspace_agents.exists() {
            collect_agent_files_in_root(&workspace_agents)?
        } else {
            Vec::new()
        } {
            let parsed = parse_and_validate_agent_file(
                &path,
                policy,
                Some(&world),
                "workspace",
                &workspace,
            )?;
            files.push((path, parsed));
        }
    }
    bootstrap
        .revalidate()
        .map_err(|error| config_model::user_error(error.to_string()))?;
    assemble_inventory_snapshot_v3(files)
}

#[derive(Debug, Clone)]
#[allow(
    dead_code,
    clippy::large_enum_variant,
    reason = "E3-C validates and retains complete V3 source material before E3-D consumes it"
)]
enum ParsedAgentInventoryFile {
    // Only the held-descriptor path can attach authenticated source material.
    V3(AgentFileV3, Option<AgentInventorySourceMaterialV1>),
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
    pub selection: Option<transport_api_types::AgentSelectionTupleV1>,
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
pub(crate) fn project_inventory_v3_entry(
    cwd: &Path,
    path: &Path,
    file: &AgentFileV3,
    effective_default_cli_mode: crate::execution::config_model::AgentCliMode,
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
            .unwrap_or(effective_default_cli_mode);
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
            selection: placement_config.selection.clone(),
            source: source.clone(),
        });
    }
    projected
}

#[cfg(target_os = "linux")]
pub(crate) fn resolve_e3_selected_inventory_projection_v1(
    bootstrap_home: &crate::execution::agent_runtime::OpenedBootstrapHomeV1<'_>,
    global_inventory_root: &crate::execution::agent_runtime::host_session_authority::trusted_fs::HeldE3AgentInventoryRootV1,
    workspace_inventory_root: &crate::execution::agent_runtime::host_session_authority::trusted_fs::HeldE3AgentInventoryRootV1,
    effective_config: &config_projection::EffectiveSubstrateConfigSourceV1,
    expected_source: &AgentInventorySourceMaterialV1,
) -> Result<
    config_projection::LogicalAgentConfigProjectionV1,
    config_projection::ConfigProjectionFailureV1,
> {
    with_e3_selected_inventory_projection_v1(
        bootstrap_home,
        global_inventory_root,
        workspace_inventory_root,
        effective_config,
        Some(expected_source),
        |_, logical, _, _| Ok(logical.clone()),
    )
}

#[cfg(target_os = "linux")]
pub(crate) struct HeldE3SelectedInventoryProjectionV1<'r> {
    policy: Policy,
    logical: config_projection::LogicalAgentConfigProjectionV1,
    selected: PlacementProjectedInventoryEntryV3,
    sources: Vec<crate::execution::agent_runtime::host_session_authority::trusted_fs::HeldE3AgentInventorySourceV1<'r>>,
    global: &'r crate::execution::agent_runtime::host_session_authority::trusted_fs::HeldE3AgentInventoryRootV1,
    workspace: &'r crate::execution::agent_runtime::host_session_authority::trusted_fs::HeldE3AgentInventoryRootV1,
}

#[cfg(target_os = "linux")]
impl HeldE3SelectedInventoryProjectionV1<'_> {
    pub(crate) fn policy(&self) -> &Policy {
        &self.policy
    }

    pub(crate) fn logical(&self) -> &config_projection::LogicalAgentConfigProjectionV1 {
        &self.logical
    }

    pub(crate) fn selected(&self) -> &PlacementProjectedInventoryEntryV3 {
        &self.selected
    }

    pub(crate) fn selected_source(&self) -> Result<
        &crate::execution::agent_runtime::host_session_authority::trusted_fs::HeldE3AgentInventorySourceV1<'_>,
        config_projection::ConfigProjectionFailureV1,
    >{
        self.sources
            .iter()
            .find(|source| source.source_material() == &self.selected.source)
            .ok_or(config_projection::ConfigProjectionFailureV1::WrongBinding)
    }

    pub(crate) fn revalidate(&self) -> Result<(), config_projection::ConfigProjectionFailureV1> {
        for source in &self.sources {
            source.revalidate()?;
        }
        self.global.revalidate()?;
        self.workspace.revalidate()
    }
}

#[cfg(target_os = "linux")]
pub(crate) fn with_e3_selected_inventory_projection_v1<
    T,
    E: From<config_projection::ConfigProjectionFailureV1>,
>(
    bootstrap_home: &crate::execution::agent_runtime::OpenedBootstrapHomeV1<'_>,
    global: &crate::execution::agent_runtime::host_session_authority::trusted_fs::HeldE3AgentInventoryRootV1,
    workspace: &crate::execution::agent_runtime::host_session_authority::trusted_fs::HeldE3AgentInventoryRootV1,
    effective: &config_projection::EffectiveSubstrateConfigSourceV1,
    expected: Option<&AgentInventorySourceMaterialV1>,
    consume: impl FnOnce(
        &Policy,
        &config_projection::LogicalAgentConfigProjectionV1,
        &PlacementProjectedInventoryEntryV3,
        &crate::execution::agent_runtime::host_session_authority::trusted_fs::HeldE3AgentInventorySourceV1<'_>,
    ) -> Result<T, E>,
) -> Result<T, E> {
    let held = hold_e3_selected_inventory_projection_v1(
        bootstrap_home,
        global,
        workspace,
        effective,
        expected,
    )?;
    let result = consume(
        held.policy(),
        held.logical(),
        held.selected(),
        held.selected_source()?,
    );
    held.revalidate()?;
    bootstrap_home
        .revalidate()
        .map_err(|_| config_projection::ConfigProjectionFailureV1::UnsupportedSecurityPosture)?;
    result
}

#[cfg(target_os = "linux")]
pub(crate) fn hold_e3_selected_inventory_projection_v1<'r>(
    bootstrap_home: &crate::execution::agent_runtime::OpenedBootstrapHomeV1<'_>,
    global_inventory_root: &'r crate::execution::agent_runtime::host_session_authority::trusted_fs::HeldE3AgentInventoryRootV1,
    workspace_inventory_root: &'r crate::execution::agent_runtime::host_session_authority::trusted_fs::HeldE3AgentInventoryRootV1,
    effective_config: &config_projection::EffectiveSubstrateConfigSourceV1,
    expected_source: Option<&AgentInventorySourceMaterialV1>,
) -> Result<HeldE3SelectedInventoryProjectionV1<'r>, config_projection::ConfigProjectionFailureV1> {
    use crate::execution::agent_runtime::host_session_authority::trusted_fs::HeldE3AgentInventorySourceV1;
    use crate::execution::config_model::AgentCliMode;
    use config_projection::{ConfigProjectionFailureV1::*, LogicalConfigSourceRefV1};
    bootstrap_home
        .revalidate()
        .map_err(|_| UnsupportedSecurityPosture)?;
    global_inventory_root.revalidate()?;
    workspace_inventory_root.revalidate()?;
    let cwd = Path::new(&effective_config.workspace_root.physical_path);
    let policy = crate::execution::policy_model::resolve_effective_policy_for_bootstrap_home(
        cwd,
        bootstrap_home,
    )
    .map_err(|_| UnsupportedSecurityPosture)?;
    let mode = match effective_config.values.default_cli_mode.as_str() {
        "persistent" => AgentCliMode::Persistent,
        "per_request" => AgentCliMode::PerRequest,
        _ => return Err(WrongBinding),
    };
    let mut effective = BTreeMap::new();
    let mut held_sources = Vec::new();
    for root in [global_inventory_root, workspace_inventory_root] {
        let mut ids = BTreeSet::new();
        for relative in root.source_relative_paths()? {
            let held = HeldE3AgentInventorySourceV1::open(root, &relative)?;
            let raw = std::str::from_utf8(held.source_bytes()).map_err(|_| Malformed)?;
            let value: serde_yaml::Value = serde_yaml::from_str(raw).map_err(|_| Malformed)?;
            let id = value
                .get("id")
                .and_then(serde_yaml::Value::as_str)
                .ok_or(Malformed)?;
            if !ids.insert(id.to_string()) {
                return Err(Conflict);
            }
            let source = held.source_material();
            let path = Path::new(&source.accepted_root.physical_path).join(&relative);
            let parsed = parse_and_validate_agent_file_raw(
                &path,
                raw,
                &policy,
                Some(cwd),
                &source.inventory_scope,
                Path::new(&source.accepted_root.physical_path),
                Some((held.source_bytes(), source)),
            )
            .map_err(|_| Malformed)?;
            if matches!(parsed, ParsedAgentInventoryFile::V3(..)) {
                // R05 introduces complete ordinary selection. Legacy protected
                // records cannot authenticate that tuple; R11-R13 own adoption.
                return Err(UnsupportedConfiguration);
            }
            effective.insert(id.to_string(), (path, parsed));
            held_sources.push(held);
        }
    }
    let mut selected = None;
    let mut source_selected_but_ineligible = false;
    for (path, parsed) in effective.values() {
        let ParsedAgentInventoryFile::V3(file, Some(source)) = parsed else {
            continue;
        };
        let same_source = expected_source.is_some_and(|expected| {
            source.inventory_scope == expected.inventory_scope
                && source.relative_path == expected.relative_path
        });
        if same_source
            && file.config.enabled
            && file
                .config
                .placements
                .world
                .as_ref()
                .is_some_and(|world| world.enabled)
        {
            source_selected_but_ineligible = true;
        }
        for entry in project_inventory_v3_entry(cwd, path, file, mode, source) {
            if entry.placement == AgentPlacement::World
                && entry.backend_id == effective_config.values.default_backend_id
                && selected.replace(entry).is_some()
            {
                return Err(Conflict);
            }
        }
    }
    let selected = selected.ok_or(if source_selected_but_ineligible {
        WrongBinding
    } else {
        UnsupportedConfiguration
    })?;
    let source = &selected.source;
    if let Some(expected_source) = expected_source {
        if source.inventory_scope != expected_source.inventory_scope
            || source.accepted_root != expected_source.accepted_root
            || source.relative_path != expected_source.relative_path
            || source.file_device_id != expected_source.file_device_id
            || source.file_inode != expected_source.file_inode
            || source.byte_length != expected_source.byte_length
        {
            return Err(WrongBinding);
        }
        if source.raw_bytes_sha256 != expected_source.raw_bytes_sha256
            || source.source_revision != expected_source.source_revision
            || source.source_hash != expected_source.source_hash
        {
            return Err(HashInvalid);
        }
    }
    let runtime = selected
        .runtime_projection
        .as_ref()
        .ok_or(UnsupportedConfiguration)?;
    let runtime_family = match selected
        .cli_runtime_family
        .ok_or(UnsupportedConfiguration)?
    {
        AgentCliRuntimeFamily::Codex => "codex",
        AgentCliRuntimeFamily::ClaudeCode => "claude_code",
    };
    let caps = &selected.capabilities;
    let mut capabilities = [
        ("session_start", caps.session_start),
        ("session_resume", caps.session_resume),
        ("session_fork", caps.session_fork),
        ("session_stop", caps.session_stop),
        ("status_snapshot", caps.status_snapshot),
        ("event_stream", caps.event_stream),
        ("llm", caps.llm),
        ("mcp_client", caps.mcp_client),
    ]
    .into_iter()
    .filter(|(_, enabled)| *enabled)
    .map(|(name, _)| name.to_string())
    .collect::<Vec<_>>();
    capabilities.sort_by(|a, b| a.as_bytes().cmp(b.as_bytes()));
    let mut logical = config_projection::LogicalAgentConfigProjectionV1 {
        projection_hash: String::new(),
        sources: vec![
            LogicalConfigSourceRefV1::EffectiveSubstrateConfig {
                authority_store_id: effective_config.authority_store_id.clone(),
                source_revision: effective_config.source_revision.clone(),
                source_hash: effective_config.source_hash.clone(),
            },
            LogicalConfigSourceRefV1::AgentInventory {
                authority_store_id: effective_config.authority_store_id.clone(),
                inventory_scope: source.inventory_scope.clone(),
                source_revision: source.source_revision.clone(),
                source_hash: source.source_hash.clone(),
            },
        ],
        logical_agent_id: selected.logical_agent_id.clone(),
        placement: selected.placement.as_str().into(),
        realized_agent_id: selected.realized_agent_id.clone(),
        backend_id: selected.backend_id.clone(),
        kind: selected.kind.as_str().into(),
        protocol: selected.protocol.clone().ok_or(WrongBinding)?,
        execution_scope: match selected.execution_scope {
            config_model::AgentExecutionScope::Host => "host",
            config_model::AgentExecutionScope::World => "world",
        }
        .into(),
        cli_mode: match selected.cli_mode {
            AgentCliMode::Persistent => "persistent",
            AgentCliMode::PerRequest => "per_request",
        }
        .into(),
        runtime_family: runtime_family.into(),
        capabilities,
        requested_model: selected
            .selection
            .as_ref()
            .ok_or(UnsupportedConfiguration)?
            .model
            .clone(),
        requested_mcp_servers: runtime.mcp_servers.clone(),
        requested_features: runtime.features.clone(),
    };
    let mut value = serde_json::to_value(&logical).map_err(|_| HashInvalid)?;
    value
        .as_object_mut()
        .ok_or(HashInvalid)?
        .remove("projection_hash");
    logical.projection_hash = config_projection::ConfigProjectionCodecV1::domain_sha256(
        "substrate.e3.logical-config-projection.v1",
        &serde_json::json!({"projection": value}),
    )
    .map_err(|_| HashInvalid)?;
    for held in &held_sources {
        held.revalidate()?;
    }
    global_inventory_root.revalidate()?;
    workspace_inventory_root.revalidate()?;
    bootstrap_home
        .revalidate()
        .map_err(|_| UnsupportedSecurityPosture)?;
    let held = HeldE3SelectedInventoryProjectionV1 {
        policy,
        logical,
        selected,
        sources: held_sources,
        global: global_inventory_root,
        workspace: workspace_inventory_root,
    };
    held.selected_source()?;
    Ok(held)
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
    Ok(load_agent_inventory_snapshot_v3(cwd, base_policy)?.entries)
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
    Ok(
        load_agent_inventory_snapshot_for_bootstrap_home_v3(cwd, base_policy, bootstrap_home)?
            .entries,
    )
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

pub(crate) fn validate_agent_file(path: &Path, base_policy: &Policy) -> Result<AgentFileV3> {
    let world_root = inventory_workspace_root_from_path(path);
    let accepted_root = world_root.as_deref().unwrap_or_else(|| {
        path.parent()
            .and_then(Path::parent)
            .unwrap_or_else(|| Path::new("/"))
    });
    match parse_and_validate_agent_file(
        path,
        base_policy,
        world_root.as_deref(),
        if world_root.is_some() {
            "workspace"
        } else {
            "global"
        },
        accepted_root,
    )? {
        ParsedAgentInventoryFile::V3(file, _) => Ok(file),
    }
}

fn parse_and_validate_agent_file(
    path: &Path,
    base_policy: &Policy,
    world_root: Option<&Path>,
    source_scope: &str,
    accepted_root: &Path,
) -> Result<ParsedAgentInventoryFile> {
    let raw = read_inventory_source(path, accepted_root)?;
    parse_and_validate_agent_file_raw(
        path,
        &raw,
        base_policy,
        world_root,
        source_scope,
        accepted_root,
        None,
    )
}

/// Read from the named source with stable directory/file identity. Source hashes
/// describe these bytes; they do not confer authority to launch an agent.
fn read_inventory_source(path: &Path, accepted_root: &Path) -> Result<String> {
    use std::io::Read;
    let relative = path.strip_prefix(accepted_root).map_err(|_| {
        config_model::user_error(format!(
            "inventory source {} escapes its root",
            path.display()
        ))
    })?;
    let mut names = relative.components().peekable();
    let mut parent_path = accepted_root.to_path_buf();
    let mut directories = Vec::new();
    let mut options = fs::OpenOptions::new();
    options.read(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.custom_flags(libc::O_NOFOLLOW | libc::O_DIRECTORY);
    }
    let mut directory = options.open(accepted_root).map_err(|error| {
        config_model::user_error(format!(
            "invalid inventory root {}: {error}",
            accepted_root.display()
        ))
    })?;
    directories.push((parent_path.clone(), directory.metadata()?));
    let mut source = None;
    while let Some(component) = names.next() {
        let std::path::Component::Normal(name) = component else {
            return Err(config_model::user_error(
                "inventory source requires a confined relative path",
            ));
        };
        parent_path.push(name);
        let is_directory = names.peek().is_some();
        #[cfg(unix)]
        let opened = {
            use std::os::{
                fd::{AsRawFd, FromRawFd},
                unix::ffi::OsStrExt,
            };
            let name = std::ffi::CString::new(name.as_bytes())?;
            let flags = libc::O_RDONLY
                | libc::O_CLOEXEC
                | libc::O_NOFOLLOW
                | if is_directory { libc::O_DIRECTORY } else { 0 };
            // SAFETY: directory is held, name is NUL-terminated, and a successful
            // new fd is transferred exactly once to File.
            let fd = unsafe { libc::openat(directory.as_raw_fd(), name.as_ptr(), flags) };
            if fd < 0 {
                return Err(config_model::user_error(format!(
                    "failed to read inventory source {}: {}",
                    path.display(),
                    std::io::Error::last_os_error()
                )));
            }
            unsafe { fs::File::from_raw_fd(fd) }
        };
        #[cfg(not(unix))]
        let opened = {
            if fs::symlink_metadata(&parent_path)?.file_type().is_symlink() {
                return Err(config_model::user_error("inventory symlink is unsupported"));
            }
            fs::File::open(&parent_path)?
        };
        if is_directory {
            directories.push((parent_path.clone(), opened.metadata()?));
            directory = opened;
        } else {
            source = Some(opened);
        }
    }
    let file =
        source.ok_or_else(|| config_model::user_error("inventory source filename missing"))?;
    let before = file.metadata()?;
    if !before.is_file() || before.len() > 1024 * 1024 {
        return Err(config_model::user_error(format!(
            "inventory source {} must be a regular file at most 1MiB",
            path.display()
        )));
    }
    let mut bytes = Vec::new();
    (&file).take(1024 * 1024 + 1).read_to_end(&mut bytes)?;
    let after = file.metadata()?;
    let stable = |left: &fs::Metadata, right: &fs::Metadata| {
        #[cfg(unix)]
        {
            use std::os::unix::fs::MetadataExt;
            left.dev() == right.dev()
                && left.ino() == right.ino()
                && left.mode() == right.mode()
                && left.len() == right.len()
                && left.mtime() == right.mtime()
                && left.mtime_nsec() == right.mtime_nsec()
                && left.ctime() == right.ctime()
                && left.ctime_nsec() == right.ctime_nsec()
        }
        #[cfg(not(unix))]
        {
            left.len() == right.len()
                && left.modified().ok() == right.modified().ok()
                && left.file_type() == right.file_type()
        }
    };
    if bytes.len() > 1024 * 1024
        || !stable(&before, &after)
        || !stable(&after, &fs::symlink_metadata(path)?)
    {
        return Err(config_model::user_error(format!(
            "inventory source changed while reading {}",
            path.display()
        )));
    }
    for (name, accepted) in directories {
        if !stable(&accepted, &fs::symlink_metadata(&name)?) {
            return Err(config_model::user_error(format!(
                "inventory source directory changed: {}",
                name.display()
            )));
        }
    }
    String::from_utf8(bytes).map_err(|error| {
        config_model::user_error(format!("invalid UTF-8 in {}: {error}", path.display()))
    })
}

pub(crate) fn validate_strict_v3_yaml(path: &Path, raw: &str) -> Result<()> {
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

fn parse_and_validate_agent_file_raw(
    path: &Path,
    raw: &str,
    base_policy: &Policy,
    world_root: Option<&Path>,
    _source_scope: &str,
    _accepted_root: &Path,
    held_source: Option<(&[u8], &AgentInventorySourceMaterialV1)>,
) -> Result<ParsedAgentInventoryFile> {
    match detect_agent_inventory_version(path, raw)? {
        3 => {
            validate_strict_v3_yaml(path, raw)?;
            let mut parsed: AgentFileV3 = serde_yaml::from_str(raw).map_err(|err| {
                config_model::user_error(format!(
                    "invalid YAML in {}: {}",
                    path.display(),
                    err.to_string().trim()
                ))
            })?;
            parsed.source = Some(ordinary_inventory_source(
                path,
                raw,
                _source_scope,
                _accepted_root,
            )?);
            validate_agent_schema_v3(path, &parsed, base_policy, world_root)?;
            Ok(ParsedAgentInventoryFile::V3(
                parsed,
                held_source
                    .map(|(bytes, material)| {
                        if bytes != raw.as_bytes()
                            || material.raw_bytes_sha256 != format!("{:x}", Sha256::digest(bytes))
                        {
                            return Err(config_model::user_error(
                                "invalid held inventory source digest",
                            ));
                        }
                        Ok(material.clone())
                    })
                    .transpose()?,
            ))
        }
        version => Err(config_model::user_error(format!(
            "invalid agent file in {}: version must be 3 (got {})",
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
        if is_yaml_file(&path) {
            if !entry.file_type()?.is_file() {
                return Err(config_model::user_error(format!(
                    "inventory source {} must be a regular file",
                    path.display()
                )));
            }
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
        if parsed.config.enabled
            && placement_config.enabled
            && parsed.config.kind == AgentConfigKind::Cli
        {
            let selection = placement_config.selection.as_ref().ok_or_else(|| {
                config_model::user_error(format!(
                    "invalid agent file in {}: enabled model placement requires complete selection",
                    path.display()
                ))
            })?;
            selection.validate().map_err(|error| {
                config_model::user_error(format!(
                    "invalid agent file in {}: {error}",
                    path.display()
                ))
            })?;
            let family = placement_config
                .cli
                .as_ref()
                .and_then(|cli| cli.runtime_family);
            let expected = match selection.harness {
                transport_api_types::AgentHarnessFamilyV1::Codex => AgentCliRuntimeFamily::Codex,
                transport_api_types::AgentHarnessFamilyV1::ClaudeCode => {
                    AgentCliRuntimeFamily::ClaudeCode
                }
            };
            if family != Some(expected) {
                return Err(config_model::user_error(format!("invalid agent file in {}: selection.harness must equal explicit cli.runtime_family", path.display())));
            }
        }
        if let Some(runtime) = &placement_config.runtime_projection {
            validate_runtime_projection(path, runtime)?;
        }
    }
    if let Some(overlay) = &parsed.policy_overlay {
        validate_policy_overlay(path, overlay, base_policy, world_root)?;
    }
    Ok(())
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
    use super::*;
    use crate::execution::config_model::{AgentCliMode, SubstrateConfig};
    use crate::execution::workspace::{workspace_marker_path, SUBSTRATE_DIR_NAME};
    use std::path::{Path, PathBuf};

    #[cfg(target_os = "linux")]
    mod e3f_held_selection_tests {
        use super::*;
        use crate::execution::agent_inventory::{
            hold_e3_selected_inventory_projection_v1, with_e3_selected_inventory_projection_v1,
        };
        use crate::execution::agent_runtime::host_session_authority::{
            facade::HostSessionAuthority,
            trusted_fs::{HeldE3AgentInventoryRootV1, TrustedAuthorityRoot, TrustedWorkspaceRoot},
        };
        use crate::execution::config_model::{
            extract_e3_effective_config_source_v1, resolve_e3_effective_config_source_v1,
        };
        use config_projection::ConfigProjectionFailureV1;
        use std::fs;
        use std::os::{fd::AsFd, unix::fs::PermissionsExt};

        fn with_inventory(
            workspace_yaml: Option<&str>,
            consume: impl FnOnce(
                &crate::execution::agent_runtime::OpenedBootstrapHomeV1<'_>,
                &HeldE3AgentInventoryRootV1,
                &HeldE3AgentInventoryRootV1,
                &config_projection::EffectiveSubstrateConfigSourceV1,
                &Path,
                &Path,
            ),
        ) {
            let _env = crate::execution::AuthorityEnvTestGuard::preserve();
            let home = tempfile::tempdir_in(std::env::var_os("HOME").unwrap()).unwrap();
            let workspace = tempfile::tempdir().unwrap();
            for root in [home.path(), workspace.path()] {
                fs::set_permissions(root, fs::Permissions::from_mode(0o700)).unwrap();
            }
            fs::create_dir(home.path().join("agents")).unwrap();
            fs::set_permissions(
                home.path().join("agents"),
                fs::Permissions::from_mode(0o700),
            )
            .unwrap();
            fs::create_dir_all(workspace.path().join(".substrate/agents")).unwrap();
            for (path, bytes) in [
                (home.path().join("config.yaml"), "llm: {enabled: true, gateway: {enabled: true, mode: in_world}, routing: {default_backend: 'cli:codex-world'}}\nagents: {enabled: true, defaults: {execution: {scope: world}, cli: {mode: persistent}}}\nworld: {enabled: true}\n"),
                (home.path().join("policy.yaml"), "{}\n"),
                (home.path().join("agents/codex.yaml"), e3c_v3_codex_yaml()),
                (workspace.path().join(".substrate/workspace.yaml"), "{}\n"),
            ] {
                fs::write(&path, bytes).unwrap();
                fs::set_permissions(path, fs::Permissions::from_mode(0o600)).unwrap();
            }
            if let Some(yaml) = workspace_yaml {
                fs::write(workspace.path().join(".substrate/agents/codex.yaml"), yaml).unwrap();
            }
            let trusted = TrustedAuthorityRoot::open(home.path()).unwrap();
            let authority = HostSessionAuthority::open(home.path()).unwrap();
            let bootstrap = authority.bootstrap_home();
            let identity = config_projection::CanonicalDirectoryV1::capture_linux_from_fd(
                fs::File::open(workspace.path()).unwrap().as_fd(),
            )
            .unwrap();
            let config_projection::DirectoryPhysicalIdentityV1::Linux { device_id, inode } =
                identity.physical_identity;
            let trusted_identity = crate::execution::agent_runtime::host_session_authority::schema::CanonicalDirectoryV1 {
                physical_path: identity.physical_path.clone(),
                physical_identity: crate::execution::agent_runtime::host_session_authority::schema::DirectoryPhysicalIdentityV1::Linux { device_id, inode },
            };
            let trusted_workspace = TrustedWorkspaceRoot::open_exact(&trusted_identity).unwrap();
            let global = HeldE3AgentInventoryRootV1::from_global(&trusted).unwrap();
            let local = HeldE3AgentInventoryRootV1::from_workspace(&trusted_workspace).unwrap();
            let snapshot =
                resolve_e3_effective_config_source_v1(workspace.path(), &bootstrap, &identity)
                    .unwrap();
            let effective = extract_e3_effective_config_source_v1(
                "cpa_018f0892-cc48-7a56-b711-9f17a2a81586",
                &snapshot,
            )
            .unwrap();
            consume(
                &bootstrap,
                &global,
                &local,
                &effective,
                home.path(),
                workspace.path(),
            );
        }

        #[test]
        #[serial_test::serial]
        fn test_v3_ordinary_world_passes_and_held_ingress_refuses_before_callback() {
            for workspace_yaml in [None, Some(e3c_v3_codex_yaml())] {
                with_inventory(
                    workspace_yaml,
                    |bootstrap, global, local, effective, _, workspace| {
                        let ordinary = load_agent_inventory_snapshot_for_bootstrap_home_v3(
                            workspace,
                            &Policy::default(),
                            bootstrap,
                        )
                        .unwrap();
                        assert!(ordinary.entries.contains_key("codex-world"));
                        let config = crate::execution::config_model::resolve_effective_config_for_bootstrap_home(workspace, &Default::default(), bootstrap).unwrap();
                        let selected = ordinary
                            .resolve_selection(
                                &config,
                                None,
                                Some("cli:codex-world"),
                                crate::execution::config_model::AgentExecutionScope::World,
                            )
                            .unwrap();
                        assert_eq!(selected.upstream_model_id, "codex-mini-latest");
                        assert_eq!(
                            selected.inventory_source.scope,
                            if workspace_yaml.is_some() {
                                transport_api_types::AgentSelectionSourceScopeV1::Workspace
                            } else {
                                transport_api_types::AgentSelectionSourceScopeV1::Global
                            }
                        );
                        assert!(matches!(
                            hold_e3_selected_inventory_projection_v1(
                                bootstrap, global, local, effective, None
                            ),
                            Err(ConfigProjectionFailureV1::UnsupportedConfiguration)
                        ));
                        let mut callback_entered = false;
                        let result: Result<(), ConfigProjectionFailureV1> =
                            with_e3_selected_inventory_projection_v1(
                                bootstrap,
                                global,
                                local,
                                effective,
                                None,
                                |_, _, _, _| {
                                    callback_entered = true;
                                    Ok(())
                                },
                            );
                        assert_eq!(
                            result,
                            Err(ConfigProjectionFailureV1::UnsupportedConfiguration)
                        );
                        assert!(
                            !callback_entered,
                            "refusal must precede projection/preparation/transfer callback"
                        );
                    },
                );
            }
        }
    }

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
      selection: {harness: codex, provider: openai, connection_profile: "built_in:codex-subscription", model: codex-mini-latest}
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
      selection: {harness: codex, provider: openai, connection_profile: "built_in:codex-subscription", model: codex-mini-latest}
      runtime_projection:
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
            format!(
                "{}\npolicy_overlay:\n  world_fs:\n    read:\n      allow_list: [src/lib.rs]\n",
                e3c_v3_codex_yaml().replace("id: codex", "id: e1")
            ),
        )
        .unwrap();

        let parent = e1_inventory_parent(&project);
        let original_cwd = std::env::current_dir().unwrap();
        let _cwd_guard = CurrentDirGuard(original_cwd);
        std::env::set_current_dir(&process_cwd).unwrap();
        _authority_env.install_home(&substrate_home);

        let inventory = load_effective_agent_inventory(&project, &parent)
            .expect("global inventory must bind containment to the requested project root");
        assert!(inventory.contains_key("e1-world"));
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
            e3c_v3_codex_yaml().replace("id: codex", "id: selected"),
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
                "cli:selected-world",
                &Policy::default(),
                &authority.bootstrap_home(),
            )
        };

        let entry = result.expect("resolve selected inventory");
        assert_eq!(entry.path, selected_agent);
        assert_eq!(entry.derived_backend_id(), "cli:selected-world");
    }

    #[cfg(target_os = "linux")]
    fn e3_v3_parse_held_test_source(
        path: &Path,
        raw: &str,
        policy: &Policy,
        world: Option<&Path>,
        scope: &str,
        root: &Path,
        _: Option<()>,
    ) -> anyhow::Result<ParsedAgentInventoryFile> {
        use crate::execution::agent_runtime::host_session_authority::{
            schema::{CanonicalDirectoryV1, DirectoryPhysicalIdentityV1},
            trusted_fs::{
                HeldE3AgentInventoryRootV1, HeldE3AgentInventorySourceV1, TrustedAuthorityRoot,
                TrustedWorkspaceRoot,
            },
        };
        use std::os::unix::fs::{MetadataExt, PermissionsExt};
        let held_root = if scope == "global" {
            fs::set_permissions(root, fs::Permissions::from_mode(0o700))?;
            HeldE3AgentInventoryRootV1::from_global(&TrustedAuthorityRoot::open(root).unwrap())?
        } else {
            let metadata = fs::metadata(root)?;
            let identity = CanonicalDirectoryV1 {
                physical_path: root.to_str().unwrap().into(),
                physical_identity: DirectoryPhysicalIdentityV1::Linux {
                    device_id: metadata.dev(),
                    inode: metadata.ino(),
                },
            };
            HeldE3AgentInventoryRootV1::from_workspace(
                &TrustedWorkspaceRoot::open_exact(&identity).unwrap(),
            )?
        };
        let held = HeldE3AgentInventorySourceV1::open(
            &held_root,
            path.strip_prefix(root)?.to_str().unwrap(),
        )?;
        assert_eq!(held.source_bytes(), raw.as_bytes());
        let parsed = parse_and_validate_agent_file_raw(
            path,
            std::str::from_utf8(held.source_bytes())?,
            policy,
            world,
            scope,
            root,
            Some((held.source_bytes(), held.source_material())),
        );
        held.revalidate()?;
        parsed
    }

    #[test]
    fn e3c_v3_strict_yaml_rejects_ambiguous_yaml_features() {
        let temp =
            tempfile::tempdir_in(Path::new(&std::env::var_os("HOME").unwrap()).join(".cache"))
                .unwrap();
        let root = temp.path().join("home");
        let agents = root.join("agents");
        fs::create_dir_all(&agents).unwrap();
        let path = agents.join("codex.yaml");
        let valid = e3c_v3_codex_yaml();
        fs::write(&path, valid).unwrap();
        e3_v3_parse_held_test_source(
            &path,
            valid,
            &Policy::default(),
            Some(temp.path()),
            "global",
            &root,
            None,
        )
        .expect("strict V3 fixture");

        for invalid in [
            format!("{valid}version: 3\n"),
            valid.replace("id: codex", "id: &agent codex\nalias: *agent"),
            valid.replace("  enabled: true", "  <<: { enabled: true }"),
            valid.replace(
                "model: codex-mini-latest",
                "model: !custom codex-mini-latest",
            ),
            format!("{valid}---\nversion: 3\n"),
            valid.replace("  protocol:", "  7: rejected\n  protocol:"),
        ] {
            fs::write(&path, &invalid).unwrap();
            assert!(e3_v3_parse_held_test_source(
                &path,
                &invalid,
                &Policy::default(),
                Some(temp.path()),
                "global",
                &root,
                None,
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
        validate_agent_file(&path, &Policy::default()).expect("ordinary V3 validator");
        for version in [1, 2, 4] {
            fs::write(
                &path,
                e3c_v3_codex_yaml().replacen("version: 3", &format!("version: {version}"), 1),
            )
            .unwrap();
            assert!(validate_agent_file(&path, &Policy::default())
                .unwrap_err()
                .to_string()
                .contains("version must be 3"));
        }

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
        let temp =
            tempfile::tempdir_in(Path::new(&std::env::var_os("HOME").unwrap()).join(".cache"))
                .unwrap();
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
        let global = e3_v3_parse_held_test_source(
            &global_path,
            global,
            &Policy::default(),
            Some(&workspace_root),
            "global",
            &global_root,
            None,
        )
        .unwrap();
        let workspace = e3_v3_parse_held_test_source(
            &workspace_path,
            &workspace,
            &Policy::default(),
            Some(&workspace_root),
            "workspace",
            &workspace_root,
            None,
        )
        .unwrap();
        let (workspace_file, workspace_source) = match &workspace {
            ParsedAgentInventoryFile::V3(file, Some(source)) => (file, source),
            _ => panic!("expected V3"),
        };
        let projected = project_inventory_v3_entry(
            &workspace_root,
            &workspace_path,
            workspace_file,
            SubstrateConfig::default().agents.defaults.cli.mode,
            workspace_source,
        );
        assert_eq!(projected.len(), 2);
        assert!(projected
            .iter()
            .all(|entry| entry.source.source_hash == workspace_source.source_hash));
        let global_source_hash = match global {
            ParsedAgentInventoryFile::V3(_, Some(source)) => source.source_hash,
            _ => unreachable!(),
        };
        assert_ne!(global_source_hash, workspace_source.source_hash);

        let error = assemble_inventory_snapshot_v3(vec![
            (workspace_path.clone(), workspace.clone()),
            (workspace_path, workspace),
        ])
        .expect_err("duplicate logical IDs must fail before selection");
        assert!(error.to_string().contains("duplicate agent"));
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
        let inventory = load_effective_agent_inventory_for_bootstrap_home(
            parent.path(),
            &Policy::default(),
            &authority.bootstrap_home(),
        )
        .expect("authenticated ordinary V3");
        assert_eq!(inventory["codex-world"].path, inventory_path);
        assert_eq!(
            inventory["codex-world"].cli_runtime_family(),
            Some(AgentCliRuntimeFamily::Codex)
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
