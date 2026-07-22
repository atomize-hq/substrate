use crate::builtins::world_deps::{self, WorldDepsDoctorSnapshotV1};
#[cfg(unix)]
use crate::execution::agent_runtime::host_session_authority::trusted_fs::TrustedAuthorityRoot;
#[cfg(unix)]
use crate::execution::agent_runtime::HostSessionAuthority;
#[cfg(unix)]
use crate::execution::install_bootstrap::{
    bind_unix_install_bootstrap_context, checked_install_bootstrap_context_from_projections,
    expected_install_bootstrap_projections, unix_account_home_for_principal,
};
use crate::execution::{
    config_model::{self, CliConfigOverrides},
    current_platform,
    manager_init::{self, ManagerInitConfig, ManifestPaths},
    manager_manifest_base_path,
};
use anyhow::{anyhow, Context, Result};
use chrono::{DateTime, Utc};
use serde::Serialize;
use serde_json::Value;
use std::{
    collections::HashMap,
    env,
    fs::{self, File},
    io::{self, BufRead, BufReader},
    path::{Path, PathBuf},
    process::Command,
};
use substrate_common::manager_manifest::{ManagerManifest, ManagerSpec};
#[cfg(not(unix))]
use substrate_common::paths as substrate_paths;
#[cfg(unix)]
use transport_api_types::{InstallBootstrapContextCarrierV1, WorldDoctorReportV1};

#[derive(Debug, Serialize, Clone)]
pub struct ShimDoctorReport {
    #[cfg(unix)]
    pub selected_host_prefix: PathBuf,
    #[cfg(unix)]
    pub host_context_commitment: String,
    #[cfg(unix)]
    pub install_context_source: &'static str,
    pub manifest: ManifestInfo,
    pub path: PathDoctorStatus,
    pub trace_log: PathBuf,
    pub skip_all_requested: bool,
    pub states: Vec<ManagerDoctorState>,
    pub hints: Vec<HintRecord>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub world: Option<WorldDoctorSnapshot>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub world_deps: Option<WorldDepsDoctorSection>,
}

#[derive(Debug, Serialize, Clone)]
pub struct ManifestInfo {
    pub base: PathBuf,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub overlay: Option<PathBuf>,
    pub overlay_exists: bool,
}

#[derive(Debug, Serialize, Clone)]
pub struct PathDoctorStatus {
    pub shim_dir: PathBuf,
    pub shim_dir_exists: bool,
    pub path_first_entry: Option<String>,
    pub host_contains_shims: bool,
    pub shim_first_in_path: bool,
    pub bashenv_path: PathBuf,
    pub bashenv_exists: bool,
}

#[derive(Debug, Serialize, Clone)]
pub struct ManagerDoctorState {
    pub name: String,
    pub detected: bool,
    pub reason: Option<String>,
    pub init_sourced: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub snippet: Option<String>,
    pub repair_available: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_hint: Option<HintRecord>,
}

#[derive(Debug, Serialize, Clone)]
pub struct HintRecord {
    pub name: String,
    pub hint: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pattern: Option<String>,
    pub last_seen: DateTime<Utc>,
}

#[derive(Debug, Serialize, Clone)]
pub struct WorldDoctorSnapshot {
    pub status: WorldDoctorStatus,
    pub ok: bool,
    pub platform: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub world_disable_reason: Option<&'static str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub world_disable_source: Option<config_model::DoctorDisableSource>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exit_code: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stderr: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<Value>,
}

#[derive(Debug, Serialize, Clone)]
pub struct WorldDepsDoctorSection {
    pub status: WorldDepsDoctorStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub report: Option<WorldDepsDoctorSnapshotV1>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
}

#[allow(dead_code)]
#[derive(Debug, Serialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum WorldDoctorStatus {
    Healthy,
    NeedsAttention,
    Disabled,
    Unknown,
}

#[allow(dead_code)]
#[derive(Debug, Serialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum WorldDepsDoctorStatus {
    Ok,
    Error,
    SkippedDisabled,
    Unknown,
}

#[cfg_attr(
    unix,
    allow(
        dead_code,
        reason = "Unix checked-projection compatibility is retained until the R2-3 migration"
    )
)]
pub(crate) fn collect_report(
    cli_no_world: bool,
    cli_force_world: bool,
) -> Result<ShimDoctorReport> {
    #[cfg(unix)]
    {
        let install_context = checked_install_bootstrap_context_from_projections()?;
        collect_report_for_context(cli_no_world, cli_force_world, &install_context)
    }
    #[cfg(not(unix))]
    {
        build_report(cli_no_world, cli_force_world)
    }
}

#[cfg(unix)]
pub(crate) fn collect_report_for_context(
    cli_no_world: bool,
    cli_force_world: bool,
    install_context: &InstallBootstrapContextCarrierV1,
) -> Result<ShimDoctorReport> {
    bind_unix_install_bootstrap_context(install_context)?;
    build_report(cli_no_world, cli_force_world, install_context)
}

pub(crate) fn build_manifest_paths(
    #[cfg(unix)] install_context: &InstallBootstrapContextCarrierV1,
) -> Result<(ManifestInfo, ManifestPaths)> {
    #[cfg(unix)]
    {
        bind_unix_install_bootstrap_context(install_context)?;
        let base = manager_manifest_base_path(install_context);
        let substrate_home = PathBuf::from(&install_context.context.selected_host_prefix);
        build_manifest_paths_at(base, substrate_home)
    }
    #[cfg(not(unix))]
    {
        build_manifest_paths_at(
            manager_manifest_base_path(),
            substrate_paths::substrate_home()?,
        )
    }
}

fn build_manifest_paths_at(
    base: PathBuf,
    substrate_home: PathBuf,
) -> Result<(ManifestInfo, ManifestPaths)> {
    let overlay_path = substrate_home.join("manager_hooks.local.yaml");
    let overlay_exists = overlay_path.exists();
    let manifest_info = ManifestInfo {
        base: base.clone(),
        overlay: Some(overlay_path.clone()),
        overlay_exists,
    };
    let manifest_paths = ManifestPaths {
        base,
        overlay: Some(overlay_path),
    };
    Ok((manifest_info, manifest_paths))
}

pub(crate) fn manifest_spec_map(manifest: ManagerManifest) -> HashMap<String, ManagerSpec> {
    manifest
        .resolve_for_platform(current_platform())
        .into_iter()
        .map(|spec| (spec.name.clone(), spec))
        .collect()
}

pub(crate) fn legacy_bashenv_path(
    #[cfg(unix)] install_context: &InstallBootstrapContextCarrierV1,
) -> Result<PathBuf> {
    #[cfg(unix)]
    {
        bind_unix_install_bootstrap_context(install_context)?;
        Ok(
            unix_account_home_for_principal(&install_context.context.intended_host_principal)?
                .join(".substrate_bashenv"),
        )
    }
    #[cfg(not(unix))]
    {
        dirs::home_dir()
            .map(|home| home.join(".substrate_bashenv"))
            .ok_or_else(|| anyhow!("unable to determine home directory for ~/.substrate_bashenv"))
    }
}

pub(crate) fn path_separator() -> char {
    if cfg!(windows) {
        ';'
    } else {
        ':'
    }
}

pub(crate) fn same_path(lhs: &str, rhs: &str) -> bool {
    let left = normalize_path(lhs);
    let right = normalize_path(rhs);
    if cfg!(windows) {
        left.eq_ignore_ascii_case(&right)
    } else {
        left == right
    }
}

pub(crate) fn normalize_path(segment: &str) -> String {
    let trimmed = segment.trim();
    let without_sep = trimmed.trim_end_matches(['/', '\\']).to_string();
    if without_sep.is_empty() {
        trimmed.to_string()
    } else {
        without_sep
    }
}

fn build_report(
    cli_no_world: bool,
    cli_force_world: bool,
    #[cfg(unix)] install_context: &InstallBootstrapContextCarrierV1,
) -> Result<ShimDoctorReport> {
    let cwd = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let cli_world_enabled = if cli_force_world {
        Some(true)
    } else if cli_no_world {
        Some(false)
    } else {
        None
    };
    let config_overrides = CliConfigOverrides {
        world_enabled: cli_world_enabled,
        ..Default::default()
    };
    #[cfg(unix)]
    let (effective, explain) = {
        bind_unix_install_bootstrap_context(install_context)?;
        let root =
            TrustedAuthorityRoot::open(Path::new(&install_context.context.selected_host_prefix))
                .map_err(|error| anyhow!("failed to open selected config root: {error}"))?;
        let authority = HostSessionAuthority::from_trusted_root(root)
            .map_err(|error| anyhow!("failed to bind selected config root: {error}"))?;
        config_model::resolve_effective_config_with_explain_for_bootstrap_home(
            &cwd,
            &config_overrides,
            &authority.bootstrap_home(),
            true,
        )?
    };
    #[cfg(not(unix))]
    let (effective, explain) =
        config_model::resolve_effective_config_with_explain(&cwd, &config_overrides, true)?;
    let world_enabled = effective.world.enabled;
    let disable_attribution = if world_enabled {
        None
    } else {
        config_model::world_disable_attribution(
            world_enabled,
            explain
                .as_ref()
                .and_then(|explain| explain.world_enabled_explain()),
        )
    };

    #[cfg(unix)]
    let (manifest_info, manifest_paths) = build_manifest_paths(install_context)?;
    #[cfg(not(unix))]
    let (manifest_info, manifest_paths) = build_manifest_paths()?;
    let manifest = ManagerManifest::load(&manifest_info.base, manifest_info.overlay.as_deref())?;
    let spec_map = manifest_spec_map(manifest);
    let (mut states, skip_requested) = collect_states(&manifest_paths, &spec_map)?;
    #[cfg(unix)]
    let trace_log = trace_log_path(install_context)?;
    #[cfg(not(unix))]
    let trace_log = trace_log_path()?;
    let mut hints = read_hint_records(&trace_log)?;
    hints.sort_by(|a, b| a.name.cmp(&b.name));
    let mut hint_lookup = HashMap::new();
    for hint in &hints {
        hint_lookup.insert(hint.name.to_ascii_lowercase(), hint.clone());
    }

    for state in &mut states {
        if let Some(hint) = hint_lookup.get(&state.name.to_ascii_lowercase()) {
            state.last_hint = Some(hint.clone());
        }
    }

    Ok(ShimDoctorReport {
        #[cfg(unix)]
        selected_host_prefix: PathBuf::from(&install_context.context.selected_host_prefix),
        #[cfg(unix)]
        host_context_commitment: install_context.host_context_commitment.clone(),
        #[cfg(unix)]
        install_context_source: "install_context",
        manifest: manifest_info,
        #[cfg(unix)]
        path: build_path_status(install_context)?,
        #[cfg(not(unix))]
        path: build_path_status()?,
        trace_log,
        skip_all_requested: skip_requested,
        states,
        hints,
        world: Some(if world_enabled {
            gather_world_doctor_snapshot(
                #[cfg(unix)]
                install_context,
            )
        } else {
            disabled_world_doctor_snapshot(disable_attribution.as_ref())
        }),
        world_deps: Some(if world_enabled {
            gather_world_deps_section(
                cli_no_world,
                cli_force_world,
                #[cfg(unix)]
                install_context,
            )
        } else {
            disabled_world_deps_section()
        }),
    })
}

fn collect_states(
    manifest_paths: &ManifestPaths,
    spec_map: &HashMap<String, ManagerSpec>,
) -> Result<(Vec<ManagerDoctorState>, bool)> {
    let mut init_cfg = ManagerInitConfig::from_env(current_platform());
    let skip_all_requested = init_cfg.skip_all;
    init_cfg.skip_all = false;
    let result = manager_init::detect_and_generate(manifest_paths.clone(), init_cfg)?;

    let mut states = Vec::with_capacity(result.states.len());
    for state in result.states {
        let snippet_present = state
            .snippet
            .as_ref()
            .map(|snippet| !snippet.trim().is_empty())
            .unwrap_or(false);
        let spec = spec_map.get(&state.name);
        let repair_available = spec
            .and_then(|spec| spec.repair_hint.as_ref())
            .map(|hint| !hint.trim().is_empty())
            .unwrap_or(false);
        states.push(ManagerDoctorState {
            name: state.name,
            detected: state.detected,
            reason: state.reason,
            init_sourced: snippet_present && state.detected,
            snippet: state.snippet,
            repair_available,
            last_hint: None,
        });
    }

    Ok((states, skip_all_requested))
}

fn build_path_status(
    #[cfg(unix)] install_context: &InstallBootstrapContextCarrierV1,
) -> Result<PathDoctorStatus> {
    #[cfg(unix)]
    let (shim_dir, bashenv_path) = {
        bind_unix_install_bootstrap_context(install_context)?;
        let shim_dir = PathBuf::from(&install_context.context.selected_host_prefix).join("shims");
        let bashenv_path = legacy_bashenv_path(install_context)?;
        (shim_dir, bashenv_path)
    };
    #[cfg(not(unix))]
    let (shim_dir, bashenv_path) = (substrate_paths::shims_dir()?, legacy_bashenv_path()?);
    let path_value = env::var("PATH").unwrap_or_default();
    let separator = path_separator();
    let shim_dir_str = shim_dir.display().to_string();
    let path_segments: Vec<String> = path_value
        .split(separator)
        .map(|segment| segment.to_string())
        .collect();
    let host_contains_shims = path_segments
        .iter()
        .any(|segment| same_path(segment, &shim_dir_str));
    let path_first_entry = path_segments.first().cloned().filter(|s| !s.is_empty());
    let shim_first_in_path = path_first_entry
        .as_deref()
        .map(|entry| same_path(entry, &shim_dir_str))
        .unwrap_or(false);

    let shim_dir_exists = shim_dir.exists();
    let bashenv_exists = bashenv_path.exists();
    Ok(PathDoctorStatus {
        shim_dir,
        shim_dir_exists,
        path_first_entry,
        host_contains_shims,
        shim_first_in_path,
        bashenv_path,
        bashenv_exists,
    })
}

fn trace_log_path(
    #[cfg(unix)] install_context: &InstallBootstrapContextCarrierV1,
) -> Result<PathBuf> {
    #[cfg(unix)]
    {
        bind_unix_install_bootstrap_context(install_context)?;
        Ok(PathBuf::from(&install_context.context.selected_host_prefix).join("trace.jsonl"))
    }
    #[cfg(not(unix))]
    {
        if let Ok(path) = env::var("SHIM_TRACE_LOG") {
            return Ok(PathBuf::from(path));
        }
        dirs::home_dir()
            .map(|home| home.join(".substrate/trace.jsonl"))
            .ok_or_else(|| anyhow!("unable to determine home directory for trace log"))
    }
}

fn read_hint_records(trace_path: &Path) -> Result<Vec<HintRecord>> {
    let file = match File::open(trace_path) {
        Ok(file) => file,
        Err(err) if err.kind() == io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(err) => {
            return Err(err)
                .with_context(|| format!("failed to read trace log at {}", trace_path.display()))
        }
    };

    let reader = BufReader::new(file);
    let mut latest: HashMap<String, HintRecord> = HashMap::new();
    for line in reader.lines() {
        let line = match line {
            Ok(line) => line,
            Err(_) => continue,
        };
        if line.trim().is_empty() {
            continue;
        }
        let value: serde_json::Value = match serde_json::from_str(&line) {
            Ok(value) => value,
            Err(_) => continue,
        };
        let Some(hint_obj) = value.get("manager_hint") else {
            continue;
        };
        let Some(obj) = hint_obj.as_object() else {
            continue;
        };
        let Some(name) = obj.get("name").and_then(|v| v.as_str()) else {
            continue;
        };
        let Some(hint_text) = obj.get("hint").and_then(|v| v.as_str()) else {
            continue;
        };
        if name.is_empty() || hint_text.is_empty() {
            continue;
        }
        let pattern = obj
            .get("pattern")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());
        let ts_raw = obj
            .get("ts")
            .and_then(|v| v.as_str())
            .or_else(|| value.get("ts").and_then(|v| v.as_str()));
        let Some(last_seen) = ts_raw.and_then(parse_ts) else {
            continue;
        };
        let record = HintRecord {
            name: name.to_string(),
            hint: hint_text.to_string(),
            pattern,
            last_seen,
        };
        let key = name.to_ascii_lowercase();
        let should_insert = match latest.get(&key) {
            Some(existing) => record.last_seen >= existing.last_seen,
            None => true,
        };
        if should_insert {
            latest.insert(key, record);
        }
    }

    Ok(latest.into_values().collect())
}

fn parse_ts(raw: &str) -> Option<DateTime<Utc>> {
    DateTime::parse_from_rfc3339(raw)
        .map(|dt| dt.with_timezone(&Utc))
        .ok()
}

fn disabled_world_doctor_snapshot(
    disable_attribution: Option<&config_model::DoctorDisableAttribution>,
) -> WorldDoctorSnapshot {
    WorldDoctorSnapshot {
        status: WorldDoctorStatus::Disabled,
        ok: false,
        platform: env::consts::OS.to_string(),
        world_disable_reason: disable_attribution.map(|attribution| attribution.reason),
        world_disable_source: disable_attribution.map(|attribution| attribution.source.clone()),
        source: None,
        exit_code: None,
        stderr: None,
        error: None,
        details: None,
    }
}

fn disabled_world_deps_section() -> WorldDepsDoctorSection {
    WorldDepsDoctorSection {
        status: WorldDepsDoctorStatus::SkippedDisabled,
        report: None,
        error: None,
        source: Some("disabled".to_string()),
    }
}

#[cfg(unix)]
fn validate_non_secret_identity(
    value: &Value,
    pointer: &str,
    install_context: &InstallBootstrapContextCarrierV1,
) -> std::result::Result<(), &'static str> {
    fn contains_sensitive_field(value: &Value) -> bool {
        match value {
            Value::Object(fields) => fields.iter().any(|(key, value)| {
                let key: String = key
                    .chars()
                    .filter(|character| character.is_ascii_alphanumeric())
                    .flat_map(char::to_lowercase)
                    .collect();
                let forbidden = key.contains("credential")
                    || key == "token"
                    || key.ends_with("token")
                    || key.contains("apikey")
                    || key.contains("privatekey")
                    || matches!(key.as_str(), "authorization" | "authorizationheader")
                    || key == "prompt"
                    || key.starts_with("prompt")
                    || matches!(
                        key.as_str(),
                        "request" | "requestbody" | "requestbytes" | "requestinput"
                    )
                    || key.contains("preimage")
                    || key.contains("bootstrapcarrier")
                    || key.contains("shimcarrier")
                    || key.contains("authbundle")
                    || matches!(
                        key.as_str(),
                        "parentenv" | "parentenvironment" | "fullenvironment"
                    )
                    || key == "password"
                    || key.contains("secret");
                forbidden || contains_sensitive_field(value)
            }),
            Value::Array(values) => values.iter().any(contains_sensitive_field),
            _ => false,
        }
    }

    if contains_sensitive_field(value) {
        return Err("rejected: sensitive diagnostic field");
    }
    let identity = if pointer.is_empty() {
        value
    } else {
        value
            .pointer(pointer)
            .ok_or("unavailable: missing authenticated identity")?
    };
    let selected_host_prefix = identity
        .get("selected_host_prefix")
        .and_then(Value::as_str)
        .ok_or("unavailable: missing authenticated identity")?;
    let host_context_commitment = identity
        .get("host_context_commitment")
        .and_then(Value::as_str)
        .ok_or("unavailable: missing authenticated identity")?;
    if selected_host_prefix != install_context.context.selected_host_prefix
        || host_context_commitment != install_context.host_context_commitment
    {
        return Err("incoherent: authenticated identity mismatch");
    }
    Ok(())
}

fn gather_world_doctor_snapshot(
    #[cfg(unix)] install_context: &InstallBootstrapContextCarrierV1,
) -> WorldDoctorSnapshot {
    match try_load_health_fixture(
        "world_doctor.json",
        #[cfg(unix)]
        install_context,
    ) {
        Ok(Some(value)) => {
            let snapshot = snapshot_from_value(
                value,
                "fixture",
                #[cfg(unix)]
                install_context,
            );
            return snapshot;
        }
        Err(_err) => {
            return WorldDoctorSnapshot {
                status: WorldDoctorStatus::NeedsAttention,
                ok: false,
                platform: env::consts::OS.to_string(),
                world_disable_reason: None,
                world_disable_source: None,
                source: Some("fixture".to_string()),
                exit_code: None,
                stderr: None,
                error: Some("world doctor fixture unavailable".to_string()),
                details: None,
            };
        }
        Ok(None) => {}
    }

    match run_json_subcommand(
        &["world", "doctor", "--json"],
        #[cfg(unix)]
        install_context,
    ) {
        Ok(output) => snapshot_from_command(
            output,
            #[cfg(unix)]
            install_context,
        ),
        Err(_err) => WorldDoctorSnapshot {
            status: WorldDoctorStatus::NeedsAttention,
            ok: false,
            platform: env::consts::OS.to_string(),
            world_disable_reason: None,
            world_disable_source: None,
            source: Some("command".to_string()),
            exit_code: None,
            stderr: None,
            error: Some("world doctor command unavailable".to_string()),
            details: None,
        },
    }
}

fn gather_world_deps_section(
    cli_no_world: bool,
    cli_force_world: bool,
    #[cfg(unix)] install_context: &InstallBootstrapContextCarrierV1,
) -> WorldDepsDoctorSection {
    if cli_no_world && !cli_force_world {
        return disabled_world_deps_section();
    }

    let cwd = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
    match try_load_health_fixture(
        "world_deps.json",
        #[cfg(unix)]
        install_context,
    ) {
        Ok(Some(value)) => {
            #[cfg(unix)]
            if let Err(reason) = validate_non_secret_identity(&value, "", install_context) {
                return WorldDepsDoctorSection {
                    status: WorldDepsDoctorStatus::Error,
                    report: None,
                    error: Some(format!("world deps evidence {reason}")),
                    source: Some("fixture".to_string()),
                };
            }
            match serde_json::from_value::<WorldDepsDoctorSnapshotV1>(value) {
                Ok(mut report) => match status_for_world_deps_report(
                    &mut report,
                    &cwd,
                    #[cfg(unix)]
                    install_context,
                ) {
                    Ok(status) => {
                        return WorldDepsDoctorSection {
                            status,
                            report: Some(report),
                            error: None,
                            source: Some("fixture".to_string()),
                        };
                    }
                    Err(reason) => {
                        return WorldDepsDoctorSection {
                            status: WorldDepsDoctorStatus::Error,
                            report: None,
                            error: Some(format!("world deps evidence {reason}")),
                            source: Some("fixture".to_string()),
                        };
                    }
                },
                Err(_err) => {
                    return WorldDepsDoctorSection {
                        status: WorldDepsDoctorStatus::Error,
                        report: None,
                        error: Some("invalid world deps fixture".to_string()),
                        source: Some("fixture".to_string()),
                    };
                }
            }
        }
        Err(_err) => {
            return WorldDepsDoctorSection {
                status: WorldDepsDoctorStatus::Error,
                report: None,
                error: Some("world deps fixture unavailable".to_string()),
                source: Some("fixture".to_string()),
            };
        }
        Ok(None) => {}
    }

    match world_deps::collect_doctor_snapshot_v1(
        &cwd,
        false,
        #[cfg(unix)]
        install_context,
    ) {
        Ok(mut report) => match status_for_world_deps_report(
            &mut report,
            &cwd,
            #[cfg(unix)]
            install_context,
        ) {
            Ok(status) => WorldDepsDoctorSection {
                status,
                report: Some(report),
                error: None,
                source: Some("command".to_string()),
            },
            Err(reason) => WorldDepsDoctorSection {
                status: WorldDepsDoctorStatus::Error,
                report: None,
                error: Some(format!("world deps evidence {reason}")),
                source: Some("command".to_string()),
            },
        },
        Err(_err) => WorldDepsDoctorSection {
            status: WorldDepsDoctorStatus::Error,
            report: None,
            error: Some("world deps snapshot unavailable".to_string()),
            source: Some("command".to_string()),
        },
    }
}

fn status_for_world_deps_report(
    report: &mut WorldDepsDoctorSnapshotV1,
    expected_cwd: &Path,
    #[cfg(unix)] install_context: &InstallBootstrapContextCarrierV1,
) -> std::result::Result<WorldDepsDoctorStatus, &'static str> {
    #[cfg(unix)]
    {
        let value = serde_json::to_value(&*report)
            .map_err(|_| "unavailable: failed to encode dependency evidence")?;
        validate_non_secret_identity(&value, "", install_context)?;
        if report.schema_version != 1 {
            return Err("incoherent: unsupported dependency evidence schema");
        }
        if report.cwd != expected_cwd {
            return Err("incoherent: runtime scope mismatch");
        }
        if !matches!(report.inventory_mode.as_str(), "merged" | "workspace_only")
            || !matches!(report.builtins.as_str(), "enabled" | "disabled")
            || report.applied.iter().any(|item| {
                !matches!(item.kind.as_str(), "package" | "bundle")
                    || item.name.is_empty()
                    || item.enabled.is_none()
                    || item
                        .world
                        .as_deref()
                        .is_some_and(|world| !matches!(world, "present" | "missing" | "blocked"))
            })
        {
            return Err("incoherent: malformed dependency evidence");
        }
    }
    #[cfg(not(unix))]
    let _ = expected_cwd;
    Ok(if report.applied_error.is_some() {
        report.applied_error = Some("dependency application evidence unavailable".to_string());
        WorldDepsDoctorStatus::Error
    } else {
        WorldDepsDoctorStatus::Ok
    })
}

fn snapshot_from_value(
    mut value: Value,
    source: &str,
    #[cfg(unix)] install_context: &InstallBootstrapContextCarrierV1,
) -> WorldDoctorSnapshot {
    let platform = env::consts::OS.to_string();
    value["platform"] = Value::String(platform.clone());
    let explicit_ok = value.get("ok").and_then(Value::as_bool);
    #[cfg(unix)]
    let identity_result = (|| {
        let mut identity_present = false;
        if value.get("selected_host_prefix").is_some()
            || value.get("host_context_commitment").is_some()
        {
            validate_non_secret_identity(&value, "", install_context)?;
            identity_present = true;
        }
        if value.get("host").is_some() {
            validate_non_secret_identity(&value, "/host", install_context)?;
            identity_present = true;
        }
        if value.get("world").is_some() {
            validate_non_secret_identity(&value, "/world", install_context)?;
            identity_present = true;
        }
        if !identity_present {
            return Err("unavailable: missing authenticated identity");
        }
        if explicit_ok == Some(true) {
            if value.get("schema_version").and_then(Value::as_u64) != Some(1)
                || value.pointer("/host/ok").and_then(Value::as_bool) != Some(true)
                || value.pointer("/world/ok").and_then(Value::as_bool) != Some(true)
            {
                return Err("incoherent: incomplete constituent set");
            }
            let world = value
                .get("world")
                .cloned()
                .ok_or("incoherent: incomplete constituent set")?;
            let world: WorldDoctorReportV1 = serde_json::from_value(world)
                .map_err(|_| "incoherent: malformed world evidence")?;
            if world.schema_version != 2 || !world.ok {
                return Err("incoherent: malformed world evidence");
            }
        }
        Ok::<(), &'static str>(())
    })();
    #[cfg(unix)]
    if let Err(reason) = identity_result {
        return WorldDoctorSnapshot {
            status: WorldDoctorStatus::NeedsAttention,
            ok: false,
            platform,
            world_disable_reason: None,
            world_disable_source: None,
            source: Some(source.to_string()),
            exit_code: None,
            stderr: None,
            error: Some(format!("world doctor evidence {reason}")),
            details: None,
        };
    }
    let Some(ok) = explicit_ok else {
        return WorldDoctorSnapshot {
            status: WorldDoctorStatus::NeedsAttention,
            ok: false,
            platform,
            world_disable_reason: None,
            world_disable_source: None,
            source: Some(source.to_string()),
            exit_code: None,
            stderr: None,
            error: Some("world doctor evidence unavailable: explicit ok field is required".into()),
            details: None,
        };
    };
    WorldDoctorSnapshot {
        status: if ok {
            WorldDoctorStatus::Healthy
        } else {
            WorldDoctorStatus::NeedsAttention
        },
        ok,
        platform,
        world_disable_reason: None,
        world_disable_source: None,
        source: Some(source.to_string()),
        exit_code: None,
        stderr: None,
        error: None,
        details: Some(value),
    }
}

fn snapshot_from_command(
    mut output: JsonCommandOutput,
    #[cfg(unix)] install_context: &InstallBootstrapContextCarrierV1,
) -> WorldDoctorSnapshot {
    let _stderr_was_present = !output.stderr.is_empty();
    let explicit_ok = output.value.get("ok").and_then(Value::as_bool);
    let platform = env::consts::OS.to_string();
    output.value["platform"] = Value::String(platform.clone());
    #[cfg(unix)]
    let identity_result = (|| {
        let mut identity_present = false;
        if output.value.get("selected_host_prefix").is_some()
            || output.value.get("host_context_commitment").is_some()
        {
            validate_non_secret_identity(&output.value, "", install_context)?;
            identity_present = true;
        }
        if output.value.get("host").is_some() {
            validate_non_secret_identity(&output.value, "/host", install_context)?;
            identity_present = true;
        }
        if output.value.get("world").is_some() {
            validate_non_secret_identity(&output.value, "/world", install_context)?;
            identity_present = true;
        }
        if !identity_present {
            return Err("unavailable: missing authenticated identity");
        }
        if explicit_ok == Some(true) {
            if output.value.get("schema_version").and_then(Value::as_u64) != Some(1)
                || output.value.pointer("/host/ok").and_then(Value::as_bool) != Some(true)
                || output.value.pointer("/world/ok").and_then(Value::as_bool) != Some(true)
            {
                return Err("incoherent: incomplete constituent set");
            }
            let world = output
                .value
                .get("world")
                .cloned()
                .ok_or("incoherent: incomplete constituent set")?;
            let world: WorldDoctorReportV1 = serde_json::from_value(world)
                .map_err(|_| "incoherent: malformed world evidence")?;
            if world.schema_version != 2 || !world.ok {
                return Err("incoherent: malformed world evidence");
            }
        }
        Ok::<(), &'static str>(())
    })();
    #[cfg(unix)]
    if let Err(reason) = identity_result {
        return WorldDoctorSnapshot {
            status: WorldDoctorStatus::NeedsAttention,
            ok: false,
            platform,
            world_disable_reason: None,
            world_disable_source: None,
            source: Some("command".to_string()),
            exit_code: output.exit_code,
            stderr: None,
            error: Some(format!("world doctor evidence {reason}")),
            details: None,
        };
    }
    let Some(mut ok) = explicit_ok else {
        return WorldDoctorSnapshot {
            status: WorldDoctorStatus::NeedsAttention,
            ok: false,
            platform,
            world_disable_reason: None,
            world_disable_source: None,
            source: Some("command".to_string()),
            exit_code: output.exit_code,
            stderr: None,
            error: Some("world doctor evidence unavailable: explicit ok field is required".into()),
            details: None,
        };
    };
    if let Some(code) = output.exit_code {
        if code != 0 {
            ok = false;
        }
    }
    let command_succeeded = output.exit_code.is_none_or(|code| code == 0);
    WorldDoctorSnapshot {
        status: if ok {
            WorldDoctorStatus::Healthy
        } else {
            WorldDoctorStatus::NeedsAttention
        },
        ok,
        platform,
        world_disable_reason: None,
        world_disable_source: None,
        source: Some("command".to_string()),
        exit_code: output.exit_code,
        stderr: None,
        error: (!command_succeeded).then(|| "world doctor command did not succeed".to_string()),
        details: command_succeeded.then_some(output.value),
    }
}

fn try_load_health_fixture(
    name: &str,
    #[cfg(unix)] install_context: &InstallBootstrapContextCarrierV1,
) -> Result<Option<Value>> {
    let Some(path) = health_fixture_path(
        name,
        #[cfg(unix)]
        install_context,
    )?
    else {
        return Ok(None);
    };
    let raw = fs::read_to_string(&path)
        .with_context(|| format!("failed to read health fixture {}", path.display()))?;
    let value = serde_json::from_str(&raw)
        .with_context(|| format!("failed to parse health fixture {}", path.display()))?;
    Ok(Some(value))
}

fn health_fixture_path(
    name: &str,
    #[cfg(unix)] install_context: &InstallBootstrapContextCarrierV1,
) -> Result<Option<PathBuf>> {
    #[cfg(unix)]
    let base = {
        bind_unix_install_bootstrap_context(install_context)?;
        PathBuf::from(&install_context.context.selected_host_prefix)
    };
    #[cfg(not(unix))]
    let Some(base) = substrate_paths::substrate_home().ok() else {
        return Ok(None);
    };
    let path = base.join("health").join(name);
    if path.exists() {
        Ok(Some(path))
    } else {
        Ok(None)
    }
}

struct JsonCommandOutput {
    value: Value,
    exit_code: Option<i32>,
    stderr: String,
}

fn run_json_subcommand(
    args: &[&str],
    #[cfg(unix)] install_context: &InstallBootstrapContextCarrierV1,
) -> Result<JsonCommandOutput> {
    let exe = env::current_exe().with_context(|| "failed to locate substrate binary")?;
    let mut command = Command::new(&exe);
    #[cfg(unix)]
    {
        bind_unix_install_bootstrap_context(install_context)?;
        let encoded = install_context
            .encode()
            .context("failed to encode authenticated install context")?;
        command.arg("--install-bootstrap-context-v1").arg(&encoded);
        for (key, value) in expected_install_bootstrap_projections(install_context, &encoded)? {
            command.env(key, value);
        }
    }
    let output = command
        .args(args)
        .output()
        .with_context(|| format!("failed to execute `{}`", args.join(" ")))?;
    if output.stdout.is_empty() {
        return Err(anyhow!("`{}` produced no JSON output", args.join(" ")));
    }
    #[cfg(unix)]
    let mut value: Value = serde_json::from_slice(&output.stdout)
        .with_context(|| format!("failed to parse JSON output from `{}`", args.join(" ")))?;
    #[cfg(not(unix))]
    let value: Value = serde_json::from_slice(&output.stdout).with_context(|| {
        format!(
            "failed to parse JSON output from `{}`: {}",
            args.join(" "),
            String::from_utf8_lossy(&output.stdout)
        )
    })?;
    #[cfg(unix)]
    if let Some(access) = value
        .pointer_mut("/host/world_socket/access")
        .and_then(Value::as_object_mut)
    {
        access.remove("current_user");
        access.remove("current_uid");
    }
    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
    Ok(JsonCommandOutput {
        value,
        exit_code: output.status.code(),
        stderr,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[cfg(unix)]
    use crate::execution::install_bootstrap::current_unix_principal_and_home;
    #[cfg(unix)]
    use serial_test::serial;
    #[cfg(unix)]
    use std::os::unix::fs::PermissionsExt;
    #[cfg(unix)]
    use tempfile::Builder;
    #[cfg(unix)]
    use transport_api_types::InstallBootstrapContextV1;

    #[cfg(unix)]
    struct ProcessStateGuard {
        _process_cwd: crate::execution::ProcessCwdTestGuard,
    }

    #[cfg(unix)]
    impl ProcessStateGuard {
        fn set(cwd: &Path, values: &[(&'static str, &Path)]) -> Self {
            let process_cwd = crate::execution::ProcessCwdTestGuard::change_to(cwd);
            for (key, value) in values {
                std::env::set_var(key, value);
            }
            Self {
                _process_cwd: process_cwd,
            }
        }
    }

    #[cfg(unix)]
    fn prepare_route_d_prefix(root: &Path, name: &str, packages: &[&str]) -> PathBuf {
        let prefix = root.join(name);
        fs::create_dir(&prefix).expect("create Route D report prefix");
        fs::set_permissions(&prefix, fs::Permissions::from_mode(0o700))
            .expect("secure Route D report prefix");
        fs::write(
            prefix.join("config.yaml"),
            "world:\n  enabled: true\n  deps:\n    builtins: disabled\n    inventory_mode: merged\n    enabled: []\n",
        )
        .expect("write Route D report config");
        let package_dir = prefix.join("deps/packages");
        fs::create_dir_all(&package_dir).expect("create Route D report inventory");
        for package in packages {
            fs::write(
                package_dir.join(format!("{package}.yaml")),
                format!(
                    "version: 1\nname: {package}\nrunnable: false\ninstall:\n  method: manual\n  manual_instructions: test only\n"
                ),
            )
            .expect("write Route D report package");
        }
        prefix
    }

    #[cfg(unix)]
    fn route_d_carrier(prefix: &Path) -> InstallBootstrapContextCarrierV1 {
        let (principal, _) = current_unix_principal_and_home().expect("current Unix principal");
        let transport_api_types::PlatformPrincipalV1::Unix { account, uid } = principal else {
            panic!("expected Unix principal");
        };
        let context = InstallBootstrapContextV1::new_unix(
            prefix.to_str().expect("UTF-8 Route D prefix"),
            &account,
            uid,
        )
        .expect("valid Route D report context");
        InstallBootstrapContextCarrierV1::from_context(context)
            .expect("committed Route D report context")
    }

    #[cfg(unix)]
    fn coherent_world_doctor_constituent(carrier: &InstallBootstrapContextCarrierV1) -> Value {
        serde_json::json!({
            "schema_version": 2,
            "ok": true,
            "collected_at_utc": "2026-07-22T00:00:00Z",
            "selected_host_prefix": carrier.context.selected_host_prefix,
            "host_context_commitment": carrier.host_context_commitment,
            "policy_snapshot_v1_supported": true,
            "policy_resolution_mode": "snapshot_v3",
            "landlock": {
                "supported": true,
                "abi": 3,
                "reason": null
            },
            "world_fs_strategy": {
                "primary": "overlay",
                "fallback": "fuse",
                "probe": {
                    "id": "enumeration_v1",
                    "probe_file": ".substrate_enum_probe",
                    "result": "pass",
                    "failure_reason": null
                }
            }
        })
    }

    #[test]
    fn normalize_path_trims_trailing_separators() {
        assert_eq!(normalize_path("/tmp/"), "/tmp");
        assert_eq!(normalize_path(r"C:\\bin\\"), r"C:\\bin");
    }

    #[cfg(unix)]
    #[test]
    #[serial]
    fn world_deps_section_forwards_authenticated_a_under_conflicting_ambient_b() {
        let _authority_env = crate::execution::AuthorityEnvTestGuard::preserve();
        let (_, account_home) = current_unix_principal_and_home().expect("current Unix home");
        let temp = Builder::new()
            .prefix("substrate-route-d-report-")
            .tempdir_in(account_home)
            .expect("secure Route D report fixture");
        let selected_a = prepare_route_d_prefix(temp.path(), "selected-a", &["selected-only"]);
        let ambient_b =
            prepare_route_d_prefix(temp.path(), "ambient-b", &["ambient-one", "ambient-two"]);
        let ambient_health = ambient_b.join("health");
        fs::create_dir(&ambient_health).expect("create conflicting ambient health fixtures");
        fs::write(
            ambient_health.join("world_deps.json"),
            serde_json::json!({
                "schema_version": 1,
                "cwd": ambient_b,
                "inventory_packages": 99,
                "inventory_bundles": 0,
                "inventory_mode": "merged",
                "builtins": "disabled",
                "enabled": [],
                "applied": []
            })
            .to_string(),
        )
        .expect("write conflicting ambient world deps fixture");
        let carrier = route_d_carrier(&selected_a);
        let _state = ProcessStateGuard::set(
            temp.path(),
            &[
                ("SUBSTRATE_HOME", &ambient_b),
                ("SUBSTRATE_ROOT", &ambient_b),
            ],
        );

        let section = gather_world_deps_section(false, true, &carrier);
        let snapshot = section.report.expect("typed Route D report snapshot");

        assert_eq!(section.source.as_deref(), Some("command"));
        assert_eq!(snapshot.inventory_packages, 1);
        assert_eq!(snapshot.inventory_bundles, 0);
        assert_eq!(snapshot.builtins, "disabled");
        assert_eq!(
            snapshot.selected_host_prefix,
            carrier.context.selected_host_prefix
        );
        assert_eq!(
            snapshot.host_context_commitment,
            carrier.host_context_commitment
        );
    }

    #[cfg(unix)]
    #[test]
    #[serial]
    fn world_doctor_fixture_decoder_requires_explicit_ok() {
        let (_, account_home) = current_unix_principal_and_home().expect("current Unix home");
        let temp = Builder::new()
            .prefix("substrate-f5-fixture-decoder-")
            .tempdir_in(account_home)
            .expect("secure fixture-decoder root");
        let selected_a = prepare_route_d_prefix(temp.path(), "selected-a", &[]);
        let carrier = route_d_carrier(&selected_a);
        let snapshot = snapshot_from_value(
            serde_json::json!({
                "platform": std::env::consts::OS,
                "selected_host_prefix": carrier.context.selected_host_prefix,
                "host_context_commitment": carrier.host_context_commitment,
            }),
            "fixture",
            &carrier,
        );

        assert!(!snapshot.ok);
        assert_eq!(snapshot.status, WorldDoctorStatus::NeedsAttention);
        assert!(snapshot.details.is_none());
    }

    #[cfg(unix)]
    #[test]
    #[serial]
    fn world_doctor_command_decoder_requires_explicit_ok_even_on_zero_exit() {
        let (_, account_home) = current_unix_principal_and_home().expect("current Unix home");
        let temp = Builder::new()
            .prefix("substrate-f5-command-decoder-")
            .tempdir_in(account_home)
            .expect("secure command-decoder root");
        let selected_a = prepare_route_d_prefix(temp.path(), "selected-a", &[]);
        let carrier = route_d_carrier(&selected_a);
        let snapshot = snapshot_from_command(
            JsonCommandOutput {
                value: serde_json::json!({
                    "platform": std::env::consts::OS,
                    "host": {
                        "selected_host_prefix": carrier.context.selected_host_prefix,
                        "host_context_commitment": carrier.host_context_commitment,
                    },
                }),
                exit_code: Some(0),
                stderr: String::new(),
            },
            &carrier,
        );

        assert!(!snapshot.ok);
        assert_eq!(snapshot.status, WorldDoctorStatus::NeedsAttention);
        assert!(snapshot.details.is_none());
    }

    #[cfg(unix)]
    #[test]
    #[serial]
    fn world_doctor_fixture_accepts_only_complete_joined_a_constituents() {
        let (_, account_home) = current_unix_principal_and_home().expect("current Unix home");
        let temp = Builder::new()
            .prefix("substrate-f5-joined-doctor-")
            .tempdir_in(account_home)
            .expect("secure joined-doctor root");
        let selected_a = prepare_route_d_prefix(temp.path(), "selected-a", &[]);
        let carrier = route_d_carrier(&selected_a);
        let snapshot = snapshot_from_value(
            serde_json::json!({
                "schema_version": 1,
                "platform": std::env::consts::OS,
                "ok": true,
                "host": {
                    "ok": true,
                    "selected_host_prefix": carrier.context.selected_host_prefix,
                    "host_context_commitment": carrier.host_context_commitment,
                },
                "world": coherent_world_doctor_constituent(&carrier),
            }),
            "fixture",
            &carrier,
        );

        assert!(snapshot.ok);
        assert_eq!(snapshot.status, WorldDoctorStatus::Healthy);
        assert!(snapshot.error.is_none());
        assert!(snapshot.details.is_some());
    }

    #[cfg(unix)]
    #[test]
    #[serial]
    fn world_doctor_coherent_success_rejects_each_missing_constituent() {
        let (_, account_home) = current_unix_principal_and_home().expect("current Unix home");
        let temp = Builder::new()
            .prefix("substrate-f5-partial-doctor-")
            .tempdir_in(account_home)
            .expect("secure partial-doctor root");
        let selected_a = prepare_route_d_prefix(temp.path(), "selected-a", &[]);
        let carrier = route_d_carrier(&selected_a);
        let coherent = serde_json::json!({
            "schema_version": 1,
            "platform": std::env::consts::OS,
            "ok": true,
            "host": {
                "ok": true,
                "selected_host_prefix": carrier.context.selected_host_prefix,
                "host_context_commitment": carrier.host_context_commitment,
            },
            "world": coherent_world_doctor_constituent(&carrier),
        });

        for pointer in [
            "/host",
            "/world",
            "/host/selected_host_prefix",
            "/host/host_context_commitment",
            "/world/selected_host_prefix",
            "/world/host_context_commitment",
        ] {
            let mut partial = coherent.clone();
            partial
                .pointer_mut(pointer.rsplit_once('/').map_or("", |(parent, _)| parent))
                .and_then(Value::as_object_mut)
                .expect("partial constituent parent")
                .remove(pointer.rsplit_once('/').expect("constituent pointer").1);
            let snapshot = snapshot_from_value(partial, "fixture", &carrier);
            assert!(!snapshot.ok, "missing {pointer} must fail closed");
            assert_eq!(snapshot.status, WorldDoctorStatus::NeedsAttention);
            assert!(snapshot.details.is_none());
            assert!(snapshot.error.is_some());
        }
    }

    #[cfg(unix)]
    #[test]
    #[serial]
    fn world_doctor_command_exit_failure_cannot_become_success() {
        let (_, account_home) = current_unix_principal_and_home().expect("current Unix home");
        let temp = Builder::new()
            .prefix("substrate-f5-command-exit-")
            .tempdir_in(account_home)
            .expect("secure command-exit root");
        let selected_a = prepare_route_d_prefix(temp.path(), "selected-a", &[]);
        let carrier = route_d_carrier(&selected_a);
        let snapshot = snapshot_from_command(
            JsonCommandOutput {
                value: serde_json::json!({
                    "schema_version": 1,
                    "platform": std::env::consts::OS,
                    "ok": true,
                    "host": {
                        "ok": true,
                        "selected_host_prefix": carrier.context.selected_host_prefix,
                        "host_context_commitment": carrier.host_context_commitment,
                    },
                    "world": coherent_world_doctor_constituent(&carrier),
                }),
                exit_code: Some(7),
                stderr: "bounded command failure".to_string(),
            },
            &carrier,
        );

        assert!(!snapshot.ok);
        assert_eq!(snapshot.status, WorldDoctorStatus::NeedsAttention);
        assert_eq!(snapshot.exit_code, Some(7));
    }

    #[cfg(unix)]
    #[test]
    #[serial]
    fn world_deps_report_requires_exact_identity_scope_and_schema() {
        let (_, account_home) = current_unix_principal_and_home().expect("current Unix home");
        let temp = Builder::new()
            .prefix("substrate-f5-deps-coherence-")
            .tempdir_in(account_home)
            .expect("secure dependency-coherence root");
        let selected_a = prepare_route_d_prefix(temp.path(), "selected-a", &[]);
        let carrier = route_d_carrier(&selected_a);
        let mut report = WorldDepsDoctorSnapshotV1 {
            schema_version: 1,
            selected_host_prefix: carrier.context.selected_host_prefix.clone(),
            host_context_commitment: carrier.host_context_commitment.clone(),
            cwd: temp.path().to_path_buf(),
            inventory_packages: 0,
            inventory_bundles: 0,
            inventory_mode: "merged".to_string(),
            builtins: "disabled".to_string(),
            enabled: Vec::new(),
            applied: Vec::new(),
            applied_error: None,
        };

        assert_eq!(
            status_for_world_deps_report(&mut report, temp.path(), &carrier),
            Ok(WorldDepsDoctorStatus::Ok)
        );

        let mut mismatched_identity = report.clone();
        mismatched_identity.host_context_commitment = "0".repeat(64);
        assert!(
            status_for_world_deps_report(&mut mismatched_identity, temp.path(), &carrier)
                .is_err_and(|reason| reason.contains("incoherent"))
        );

        let mismatched_scope = temp.path().join("other-workspace");
        assert!(
            status_for_world_deps_report(&mut report, &mismatched_scope, &carrier)
                .is_err_and(|reason| reason.contains("incoherent"))
        );

        let mut unsupported_schema = report;
        unsupported_schema.schema_version = 2;
        assert!(
            status_for_world_deps_report(&mut unsupported_schema, temp.path(), &carrier)
                .is_err_and(|reason| reason.contains("incoherent"))
        );
    }

    #[cfg(unix)]
    #[test]
    #[serial]
    fn world_deps_fixture_without_authenticated_identity_is_unavailable() {
        let _authority_env = crate::execution::AuthorityEnvTestGuard::preserve();
        let (_, account_home) = current_unix_principal_and_home().expect("current Unix home");
        let temp = Builder::new()
            .prefix("substrate-f5-world-deps-missing-identity-")
            .tempdir_in(account_home)
            .expect("secure F5 missing-identity fixture");
        let selected_a = prepare_route_d_prefix(temp.path(), "selected-a", &[]);
        let health = selected_a.join("health");
        fs::create_dir(&health).expect("create selected health fixtures");
        let carrier = route_d_carrier(&selected_a);
        let _state = ProcessStateGuard::set(temp.path(), &[]);

        for (label, identity) in [
            ("both", serde_json::json!({})),
            (
                "commitment",
                serde_json::json!({
                    "selected_host_prefix": carrier.context.selected_host_prefix,
                }),
            ),
            (
                "prefix",
                serde_json::json!({
                    "host_context_commitment": carrier.host_context_commitment,
                }),
            ),
        ] {
            let mut fixture = serde_json::json!({
                "schema_version": 1,
                "cwd": temp.path(),
                "inventory_packages": 0,
                "inventory_bundles": 0,
                "inventory_mode": "merged",
                "builtins": "disabled",
                "enabled": [],
                "applied": []
            });
            fixture
                .as_object_mut()
                .expect("world-deps fixture object")
                .extend(identity.as_object().expect("identity object").clone());
            fs::write(health.join("world_deps.json"), fixture.to_string())
                .unwrap_or_else(|error| panic!("write {label} missing-identity fixture: {error}"));

            let section = gather_world_deps_section(false, true, &carrier);

            assert_eq!(section.status, WorldDepsDoctorStatus::Error, "{label}");
            assert!(section.report.is_none(), "{label}");
            assert!(
                section
                    .error
                    .as_deref()
                    .is_some_and(|error| error.contains("unavailable")),
                "{label}"
            );
        }
    }

    #[cfg(unix)]
    #[test]
    #[serial]
    fn world_doctor_fixture_rejects_mixed_identity_without_disclosure() {
        let _authority_env = crate::execution::AuthorityEnvTestGuard::preserve();
        let (_, account_home) = current_unix_principal_and_home().expect("current Unix home");
        let temp = Builder::new()
            .prefix("substrate-f5-world-mixed-identity-")
            .tempdir_in(account_home)
            .expect("secure F5 mixed-identity fixture");
        let selected_a = prepare_route_d_prefix(temp.path(), "selected-a", &[]);
        let health = selected_a.join("health");
        fs::create_dir(&health).expect("create selected health fixtures");
        let carrier = route_d_carrier(&selected_a);
        let mut conflicting_commitment = carrier.host_context_commitment.clone();
        let replacement = if conflicting_commitment.starts_with('0') {
            "1"
        } else {
            "0"
        };
        conflicting_commitment.replace_range(..1, replacement);
        fs::write(
            health.join("world_doctor.json"),
            serde_json::json!({
                "schema_version": 1,
                "platform": std::env::consts::OS,
                "ok": true,
                "host": {
                    "ok": true,
                    "selected_host_prefix": selected_a,
                    "host_context_commitment": carrier.host_context_commitment,
                },
                "world": {
                    "ok": true,
                    "selected_host_prefix": selected_a,
                    "host_context_commitment": conflicting_commitment,
                },
            })
            .to_string(),
        )
        .expect("write mixed-identity world fixture");

        let snapshot = gather_world_doctor_snapshot(&carrier);
        let rendered = serde_json::to_string(&snapshot).expect("serialize rejected snapshot");

        assert!(!snapshot.ok);
        assert_eq!(snapshot.status, WorldDoctorStatus::NeedsAttention);
        assert!(snapshot.details.is_none());
        assert!(!rendered.contains(&conflicting_commitment));
        assert!(snapshot
            .error
            .as_deref()
            .is_some_and(|error| error.contains("incoherent")));
    }

    #[cfg(unix)]
    #[test]
    #[serial]
    fn world_doctor_unavailable_fixture_validates_every_present_constituent() {
        let (_, account_home) = current_unix_principal_and_home().expect("current Unix home");
        let temp = Builder::new()
            .prefix("substrate-f5-unavailable-constituents-")
            .tempdir_in(account_home)
            .expect("secure unavailable-constituent root");
        let selected_a = prepare_route_d_prefix(temp.path(), "selected-a", &[]);
        let carrier = route_d_carrier(&selected_a);
        let root = serde_json::json!({
            "schema_version": 1,
            "platform": std::env::consts::OS,
            "ok": false,
            "selected_host_prefix": carrier.context.selected_host_prefix,
            "host_context_commitment": carrier.host_context_commitment,
        });

        let mut missing_identity = root.clone();
        missing_identity["host"] = serde_json::json!({"ok": false});
        let missing = snapshot_from_value(missing_identity, "fixture", &carrier);
        assert!(!missing.ok);
        assert_eq!(missing.status, WorldDoctorStatus::NeedsAttention);
        assert!(missing.details.is_none());
        assert!(missing
            .error
            .as_deref()
            .is_some_and(|error| error.contains("unavailable")));

        let mut mixed_identity = root;
        mixed_identity["host"] = serde_json::json!({
            "ok": false,
            "selected_host_prefix": selected_a.join("ambient-b"),
            "host_context_commitment": "0".repeat(64),
        });
        let mixed = snapshot_from_value(mixed_identity, "fixture", &carrier);
        assert!(!mixed.ok);
        assert_eq!(mixed.status, WorldDoctorStatus::NeedsAttention);
        assert!(mixed.details.is_none());
        assert!(mixed
            .error
            .as_deref()
            .is_some_and(|error| error.contains("incoherent")));
    }

    #[cfg(unix)]
    #[test]
    #[serial]
    fn world_doctor_rejection_bounds_platform_stderr_and_key_spelling_markers() {
        const PLATFORM_MARKER: &str = "f5-platform-private-key-marker";
        const STDERR_MARKER: &str = "f5-command-provider-token-marker";
        const FIELD_MARKER: &str = "f5-camel-secret-marker";
        let (_, account_home) = current_unix_principal_and_home().expect("current Unix home");
        let temp = Builder::new()
            .prefix("substrate-f5-bounded-rejection-")
            .tempdir_in(account_home)
            .expect("secure bounded-rejection root");
        let selected_a = prepare_route_d_prefix(temp.path(), "selected-a", &[]);
        let carrier = route_d_carrier(&selected_a);

        for key in [
            "providerToken",
            "api-key",
            "private-key",
            "authorization-header",
            "commitment-pre-image",
            "bootstrapCarrier",
            "authBundle",
            "parentEnvironment",
            "client_secret",
        ] {
            let mut value = serde_json::json!({
                "schema_version": 1,
                "platform": PLATFORM_MARKER,
                "ok": false,
                "selected_host_prefix": carrier.context.selected_host_prefix,
                "host_context_commitment": carrier.host_context_commitment,
            });
            value[key] = serde_json::json!(FIELD_MARKER);
            let snapshot = snapshot_from_value(value, "fixture", &carrier);
            let rendered = serde_json::to_string(&snapshot).expect("serialize bounded rejection");
            assert!(!snapshot.ok, "{key}");
            assert_eq!(snapshot.platform, std::env::consts::OS, "{key}");
            assert!(snapshot.details.is_none(), "{key}");
            assert!(!rendered.contains(PLATFORM_MARKER), "{key}");
            assert!(!rendered.contains(FIELD_MARKER), "{key}");
        }

        let command = snapshot_from_command(
            JsonCommandOutput {
                value: serde_json::json!({
                    "schema_version": 1,
                    "platform": PLATFORM_MARKER,
                    "ok": true,
                    "host": {
                        "ok": true,
                        "selected_host_prefix": carrier.context.selected_host_prefix,
                        "host_context_commitment": carrier.host_context_commitment,
                    },
                    "world": coherent_world_doctor_constituent(&carrier),
                }),
                exit_code: Some(9),
                stderr: STDERR_MARKER.to_string(),
            },
            &carrier,
        );
        let rendered = serde_json::to_string(&command).expect("serialize bounded command result");
        assert!(!command.ok);
        assert_eq!(command.platform, std::env::consts::OS);
        assert!(command.stderr.is_none());
        assert!(command.details.is_none());
        assert!(!rendered.contains(PLATFORM_MARKER));
        assert!(!rendered.contains(STDERR_MARKER));
    }

    #[cfg(unix)]
    #[test]
    #[serial]
    fn world_deps_errors_are_bounded_and_application_error_is_sanitized() {
        const TYPE_MARKER: &str = "f5-malformed-api-key-marker";
        const APPLY_MARKER: &str = "f5-applied-private-key-marker";
        let _authority_env = crate::execution::AuthorityEnvTestGuard::preserve();
        let (_, account_home) = current_unix_principal_and_home().expect("current Unix home");
        let temp = Builder::new()
            .prefix("substrate-f5-bounded-deps-")
            .tempdir_in(account_home)
            .expect("secure bounded-deps root");
        let selected_a = prepare_route_d_prefix(temp.path(), "selected-a", &[]);
        let health = selected_a.join("health");
        fs::create_dir(&health).expect("create bounded dependency fixtures");
        let carrier = route_d_carrier(&selected_a);
        let _state = ProcessStateGuard::set(temp.path(), &[]);

        let base = serde_json::json!({
            "schema_version": 1,
            "selected_host_prefix": carrier.context.selected_host_prefix,
            "host_context_commitment": carrier.host_context_commitment,
            "cwd": temp.path(),
            "inventory_packages": 0,
            "inventory_bundles": 0,
            "inventory_mode": "merged",
            "builtins": "disabled",
            "enabled": [],
            "applied": [],
        });
        let mut malformed = base.clone();
        malformed["inventory_packages"] = serde_json::json!(TYPE_MARKER);
        fs::write(health.join("world_deps.json"), malformed.to_string())
            .expect("write malformed dependency fixture");
        let malformed_section = gather_world_deps_section(false, true, &carrier);
        let malformed_rendered =
            serde_json::to_string(&malformed_section).expect("serialize malformed section");
        assert_eq!(malformed_section.status, WorldDepsDoctorStatus::Error);
        assert!(malformed_section.report.is_none());
        assert!(!malformed_rendered.contains(TYPE_MARKER));

        let mut unavailable = base;
        unavailable["applied_error"] = serde_json::json!(APPLY_MARKER);
        fs::write(health.join("world_deps.json"), unavailable.to_string())
            .expect("write unavailable dependency fixture");
        let unavailable_section = gather_world_deps_section(false, true, &carrier);
        let unavailable_rendered =
            serde_json::to_string(&unavailable_section).expect("serialize unavailable section");
        assert_eq!(unavailable_section.status, WorldDepsDoctorStatus::Error);
        assert!(unavailable_section.report.is_some());
        assert!(!unavailable_rendered.contains(APPLY_MARKER));
    }
}
