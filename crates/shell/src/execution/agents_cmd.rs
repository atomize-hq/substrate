use crate::execution::cli::{AgentsAction, AgentsCmd, Cli};
use crate::execution::config_model::{self, AgentCliMode, AgentExecutionScope};
use crate::execution::{policy_model, workspace};
use anyhow::Result;
use serde::Deserialize;
use std::env;
use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};
use substrate_broker::Policy;
use substrate_common::paths as substrate_paths;
use url::Url;

pub(crate) fn handle_agents_command(cmd: &AgentsCmd, _cli: &Cli) -> i32 {
    let result = match &cmd.action {
        AgentsAction::Validate => run_validate(),
    };

    match result {
        Ok(()) => 0,
        Err(err) if config_model::is_user_error(&err) => {
            eprintln!("{err}");
            2
        }
        Err(err) => {
            eprintln!("{:#}", err);
            1
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct AgentFileV1 {
    version: u32,
    id: String,
    config: AgentConfigV1,
    #[serde(default)]
    policy_overlay: Option<policy_model::PolicyPatch>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct AgentConfigV1 {
    #[serde(default = "default_true")]
    enabled: bool,
    kind: AgentConfigKind,
    #[serde(default)]
    execution: AgentExecutionConfigV1,
    #[serde(default)]
    cli: Option<AgentCliConfigV1>,
    #[serde(default)]
    api: Option<AgentApiConfigV1>,
    #[serde(default)]
    capabilities: AgentCapabilitiesV1,
}

#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
enum AgentConfigKind {
    Cli,
    Api,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
struct AgentExecutionConfigV1 {
    #[serde(default)]
    scope: Option<AgentExecutionScope>,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
struct AgentCliConfigV1 {
    #[serde(default)]
    binary: String,
    #[serde(default)]
    mode: Option<AgentCliMode>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct AgentApiConfigV1 {
    base_url: String,
    auth: AgentApiAuthConfigV1,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct AgentApiAuthConfigV1 {
    env: Vec<String>,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
struct AgentCapabilitiesV1 {
    llm: bool,
    mcp_client: bool,
}

fn default_true() -> bool {
    true
}

fn run_validate() -> Result<()> {
    let cwd = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let (base_policy, _) = substrate_broker::resolve_effective_policy_with_explain(&cwd, false)
        .map_err(|err| config_model::user_error(err.to_string()))?;
    let agent_files = discover_agent_files(&cwd)?;

    for path in agent_files {
        validate_agent_file(&path, &base_policy)?;
    }

    Ok(())
}

fn discover_agent_files(cwd: &Path) -> Result<Vec<PathBuf>> {
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

fn collect_agent_files(dir: &Path, out: &mut Vec<PathBuf>) -> Result<()> {
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
    out.extend(entries);
    Ok(())
}

fn is_yaml_file(path: &Path) -> bool {
    matches!(path.extension().and_then(OsStr::to_str), Some("yaml"))
}

fn validate_agent_file(path: &Path, base_policy: &Policy) -> Result<()> {
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

    validate_agent_schema(path, &parsed, base_policy)
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

    let _ = config.execution.scope;
    let _ = config.cli.as_ref().and_then(|cli| cli.mode);
    let _ = config.enabled;
    let _ = config.capabilities.llm;
    let _ = config.capabilities.mcp_client;

    Ok(())
}

fn validate_https_base_url(path: &Path, raw: &str) -> Result<()> {
    let url = Url::parse(raw).map_err(|err| {
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
    overlay: &policy_model::PolicyPatch,
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
    overlay: &policy_model::PolicyPatch,
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

fn world_fs_deny_enforcement_rank(value: Option<substrate_broker::WorldFsDenyEnforcement>) -> u8 {
    match value {
        None => 0,
        Some(substrate_broker::WorldFsDenyEnforcement::Weak) => 1,
        Some(substrate_broker::WorldFsDenyEnforcement::PreferStrict) => 2,
        Some(substrate_broker::WorldFsDenyEnforcement::Strict) => 3,
    }
}

fn validate_overlay_subset(
    path: &Path,
    key: &str,
    overlay: Option<&[String]>,
    base: Option<&[String]>,
) -> Result<()> {
    let Some(overlay) = overlay else {
        return Ok(());
    };
    if overlay.is_empty() {
        return Ok(());
    }
    let Some(base) = base else {
        return Ok(());
    };

    for value in overlay {
        if !base.iter().any(|candidate| candidate == value) {
            return Err(config_model::user_error(format!(
                "invalid agent file in {}: {} entry '{}' broadens beyond the effective base policy",
                path.display(),
                key,
                value
            )));
        }
    }

    Ok(())
}
