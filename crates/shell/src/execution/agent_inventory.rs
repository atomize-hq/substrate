use crate::execution::config_model;
use crate::execution::workspace;
use anyhow::Result;
use serde::Deserialize;
use std::collections::BTreeMap;
use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};
use substrate_broker::{validate_backend_id, validate_dotted_id, Policy, WorldFsDenyEnforcement};
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

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub(crate) struct AgentCliConfigV1 {
    #[serde(default)]
    pub binary: String,
    #[serde(default)]
    pub mode: Option<crate::execution::config_model::AgentCliMode>,
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
    pub capabilities: AgentCapabilitiesV1,
    #[allow(dead_code)]
    pub policy_overlay: Option<crate::execution::policy_model::PolicyPatch>,
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
}

pub(crate) fn inventory_entry_origin(
    cwd: &Path,
    entry: &AgentInventoryEntryV1,
) -> AgentInventoryBaselineOrigin {
    let workspace_root = workspace::find_workspace_root(cwd);
    let workspace_agents_root = workspace_root.map(|root| {
        root.join(workspace::SUBSTRATE_DIR_NAME)
            .join("agents")
            .to_string_lossy()
            .to_string()
    });
    let entry_path = entry.path.to_string_lossy();

    if workspace_agents_root
        .as_deref()
        .is_some_and(|root| entry_path.starts_with(root))
    {
        AgentInventoryBaselineOrigin::WorkspaceInventory
    } else {
        AgentInventoryBaselineOrigin::GlobalInventory
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
        capabilities: entry.file.config.capabilities.clone(),
        policy_overlay: entry.file.policy_overlay.clone(),
    }
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
        for path in collect_agent_files_in_root(&root)? {
            let file = validate_agent_file(&path, base_policy)?;
            effective.insert(file.id.clone(), AgentInventoryEntryV1 { path, file });
        }
    }

    Ok(effective)
}

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

pub(crate) fn validate_agent_file(path: &Path, base_policy: &Policy) -> Result<AgentFileV1> {
    let raw = fs::read_to_string(path).map_err(|err| {
        config_model::user_error(format!("failed to read {}: {err}", path.display()))
    })?;
    let parsed: AgentFileV1 = serde_yaml::from_str(&raw).map_err(|err| {
        config_model::user_error(format!(
            "invalid YAML in {}: {}",
            path.display(),
            err.to_string().trim()
        ))
    })?;

    validate_agent_schema(path, &parsed, base_policy)?;
    Ok(parsed)
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

fn validate_agent_schema(path: &Path, parsed: &AgentFileV1, base_policy: &Policy) -> Result<()> {
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
        validate_policy_overlay(path, overlay, base_policy)?;
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
    validate_world_fs_overlay(path, overlay, base_policy)?;
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
) -> Result<()> {
    if let Some(overlay_host_visible) = overlay.world_fs.host_visible {
        if overlay_host_visible && !base_policy.world_fs_host_visible {
            return Err(config_model::user_error(format!(
                "invalid agent file in {}: policy_overlay.world_fs.host_visible=true broadens beyond the effective base policy (base is host_visible=false)",
                path.display()
            )));
        }
    }

    if let Some(overlay_fail_closed) = overlay.world_fs.fail_closed.routing {
        if !overlay_fail_closed && base_policy.world_fs_fail_closed_routing {
            return Err(config_model::user_error(format!(
                "invalid agent file in {}: policy_overlay.world_fs.fail_closed.routing=false broadens beyond the effective base policy",
                path.display()
            )));
        }
    }

    if let Some(overlay_caged_required) = overlay.world_fs.caged_required {
        if !overlay_caged_required && base_policy.world_fs_caged_required {
            return Err(config_model::user_error(format!(
                "invalid agent file in {}: policy_overlay.world_fs.caged_required=false broadens beyond the effective base policy",
                path.display()
            )));
        }
    }

    if let Some(overlay_write_enabled) = overlay.world_fs.write.enabled {
        if overlay_write_enabled && !base_policy.world_fs_write_enabled {
            return Err(config_model::user_error(format!(
                "invalid agent file in {}: policy_overlay.world_fs.write.enabled=true broadens beyond the effective base policy",
                path.display()
            )));
        }
    }

    if let Some(overlay_deny_enforcement) = overlay.world_fs.deny_enforcement {
        let base_rank = world_fs_deny_enforcement_rank(base_policy.world_fs_deny_enforcement);
        let overlay_rank = world_fs_deny_enforcement_rank(Some(overlay_deny_enforcement));
        if overlay_rank < base_rank {
            return Err(config_model::user_error(format!(
                "invalid agent file in {}: policy_overlay.world_fs.deny_enforcement={:?} broadens beyond the effective base policy",
                path.display(),
                overlay_deny_enforcement
            )));
        }
    }

    validate_overlay_subset(
        path,
        "policy_overlay.world_fs.read.allow_list",
        overlay.world_fs.read.allow_list.as_deref(),
        base_policy
            .world_fs_read
            .as_ref()
            .map(|dim| dim.allow_list.as_slice())
            .or_else(|| {
                base_policy
                    .world_fs_discover
                    .as_ref()
                    .map(|dim| dim.allow_list.as_slice())
            }),
    )?;
    validate_overlay_subset(
        path,
        "policy_overlay.world_fs.discover.allow_list",
        overlay.world_fs.discover.allow_list.as_deref(),
        base_policy
            .world_fs_discover
            .as_ref()
            .map(|dim| dim.allow_list.as_slice())
            .or_else(|| {
                base_policy
                    .world_fs_read
                    .as_ref()
                    .map(|dim| dim.allow_list.as_slice())
            }),
    )?;
    validate_overlay_subset(
        path,
        "policy_overlay.world_fs.write.allow_list",
        overlay.world_fs.write.allow_list.as_deref(),
        base_policy
            .world_fs_write
            .as_ref()
            .map(|dim| dim.allow_list.as_slice()),
    )?;

    Ok(())
}

fn world_fs_deny_enforcement_rank(value: Option<WorldFsDenyEnforcement>) -> u8 {
    match value {
        Some(WorldFsDenyEnforcement::Strict) => 3,
        Some(WorldFsDenyEnforcement::PreferStrict) => 2,
        Some(WorldFsDenyEnforcement::Weak) => 1,
        None => 0,
    }
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
