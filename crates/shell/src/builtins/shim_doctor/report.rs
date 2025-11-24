use crate::builtins::world_deps::{self, WorldDepsStatusReport};
use crate::execution::{
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
use substrate_common::{
    manager_manifest::{ManagerManifest, ManagerSpec},
    paths as substrate_paths,
};

#[derive(Debug, Serialize, Clone)]
pub struct ShimDoctorReport {
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
    pub ok: bool,
    pub platform: String,
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub report: Option<WorldDepsStatusReport>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
}

pub(crate) fn collect_report(
    cli_no_world: bool,
    cli_force_world: bool,
) -> Result<ShimDoctorReport> {
    build_report(cli_no_world, cli_force_world)
}

pub(crate) fn build_manifest_paths() -> Result<(ManifestInfo, ManifestPaths)> {
    let base = manager_manifest_base_path();
    let substrate_home = substrate_paths::substrate_home()?;
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

pub(crate) fn legacy_bashenv_path() -> Result<PathBuf> {
    dirs::home_dir()
        .map(|home| home.join(".substrate_bashenv"))
        .ok_or_else(|| anyhow!("unable to determine home directory for ~/.substrate_bashenv"))
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

fn build_report(cli_no_world: bool, cli_force_world: bool) -> Result<ShimDoctorReport> {
    let (manifest_info, manifest_paths) = build_manifest_paths()?;
    let manifest = ManagerManifest::load(&manifest_info.base, manifest_info.overlay.as_deref())?;
    let spec_map = manifest_spec_map(manifest);
    let (mut states, skip_requested) = collect_states(&manifest_paths, &spec_map)?;
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
        manifest: manifest_info,
        path: build_path_status()?,
        trace_log,
        skip_all_requested: skip_requested,
        states,
        hints,
        world: Some(gather_world_doctor_snapshot()),
        world_deps: Some(gather_world_deps_section(cli_no_world, cli_force_world)),
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

fn build_path_status() -> Result<PathDoctorStatus> {
    let shim_dir = substrate_paths::shims_dir()?;
    let bashenv_path = legacy_bashenv_path()?;
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

fn trace_log_path() -> Result<PathBuf> {
    if let Ok(path) = env::var("SHIM_TRACE_LOG") {
        return Ok(PathBuf::from(path));
    }
    dirs::home_dir()
        .map(|home| home.join(".substrate/trace.jsonl"))
        .ok_or_else(|| anyhow!("unable to determine home directory for trace log"))
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

fn gather_world_doctor_snapshot() -> WorldDoctorSnapshot {
    match try_load_health_fixture("world_doctor.json") {
        Ok(Some(value)) => {
            let snapshot = snapshot_from_value(value, "fixture");
            return snapshot;
        }
        Err(err) => {
            return WorldDoctorSnapshot {
                ok: false,
                platform: env::consts::OS.to_string(),
                source: Some("fixture".to_string()),
                exit_code: None,
                stderr: None,
                error: Some(format!("failed to read world doctor fixture: {err}")),
                details: None,
            };
        }
        Ok(None) => {}
    }

    match run_json_subcommand(&["world", "doctor", "--json"]) {
        Ok(output) => snapshot_from_command(output),
        Err(err) => WorldDoctorSnapshot {
            ok: false,
            platform: env::consts::OS.to_string(),
            source: Some("command".to_string()),
            exit_code: None,
            stderr: None,
            error: Some(format!("failed to gather world doctor output: {err}")),
            details: None,
        },
    }
}

fn gather_world_deps_section(cli_no_world: bool, cli_force_world: bool) -> WorldDepsDoctorSection {
    match try_load_health_fixture("world_deps.json") {
        Ok(Some(value)) => match serde_json::from_value::<WorldDepsStatusReport>(value.clone()) {
            Ok(report) => {
                return WorldDepsDoctorSection {
                    report: Some(report),
                    error: None,
                    source: Some("fixture".to_string()),
                };
            }
            Err(err) => {
                return WorldDepsDoctorSection {
                    report: None,
                    error: Some(format!("invalid world deps fixture: {err}")),
                    source: Some("fixture".to_string()),
                };
            }
        },
        Err(err) => {
            return WorldDepsDoctorSection {
                report: None,
                error: Some(format!("failed to read world deps fixture: {err}")),
                source: Some("fixture".to_string()),
            };
        }
        Ok(None) => {}
    }

    let requested: Vec<String> = Vec::new();
    match world_deps::status_report_for_health(cli_no_world, cli_force_world, &requested) {
        Ok(report) => WorldDepsDoctorSection {
            report: Some(report),
            error: None,
            source: Some("command".to_string()),
        },
        Err(err) => WorldDepsDoctorSection {
            report: None,
            error: Some(format!("failed to collect world deps status: {:#}", err)),
            source: Some("command".to_string()),
        },
    }
}

fn snapshot_from_value(value: Value, source: &str) -> WorldDoctorSnapshot {
    let ok = value.get("ok").and_then(Value::as_bool).unwrap_or(true);
    let platform = value
        .get("platform")
        .and_then(Value::as_str)
        .unwrap_or(env::consts::OS)
        .to_string();
    WorldDoctorSnapshot {
        ok,
        platform,
        source: Some(source.to_string()),
        exit_code: None,
        stderr: None,
        error: None,
        details: Some(value),
    }
}

fn snapshot_from_command(output: JsonCommandOutput) -> WorldDoctorSnapshot {
    let mut ok = output
        .value
        .get("ok")
        .and_then(Value::as_bool)
        .unwrap_or_else(|| output.exit_code.unwrap_or(0) == 0);
    if let Some(code) = output.exit_code {
        if code != 0 {
            ok = false;
        }
    }
    let platform = output
        .value
        .get("platform")
        .and_then(Value::as_str)
        .unwrap_or(env::consts::OS)
        .to_string();
    let stderr = if output.stderr.is_empty() {
        None
    } else {
        Some(output.stderr)
    };
    WorldDoctorSnapshot {
        ok,
        platform,
        source: Some("command".to_string()),
        exit_code: output.exit_code,
        stderr,
        error: None,
        details: Some(output.value),
    }
}

fn try_load_health_fixture(name: &str) -> Result<Option<Value>> {
    let Some(path) = health_fixture_path(name) else {
        return Ok(None);
    };
    let raw = fs::read_to_string(&path)
        .with_context(|| format!("failed to read health fixture {}", path.display()))?;
    let value = serde_json::from_str(&raw)
        .with_context(|| format!("failed to parse health fixture {}", path.display()))?;
    Ok(Some(value))
}

fn health_fixture_path(name: &str) -> Option<PathBuf> {
    let base = substrate_paths::substrate_home().ok()?;
    let path = base.join("health").join(name);
    if path.exists() {
        Some(path)
    } else {
        None
    }
}

struct JsonCommandOutput {
    value: Value,
    exit_code: Option<i32>,
    stderr: String,
}

fn run_json_subcommand(args: &[&str]) -> Result<JsonCommandOutput> {
    let exe = env::current_exe().with_context(|| "failed to locate substrate binary")?;
    let output = Command::new(&exe)
        .args(args)
        .output()
        .with_context(|| format!("failed to execute `{}`", args.join(" ")))?;
    if output.stdout.is_empty() {
        return Err(anyhow!("`{}` produced no JSON output", args.join(" ")));
    }
    let value: Value = serde_json::from_slice(&output.stdout).with_context(|| {
        format!(
            "failed to parse JSON output from `{}`: {}",
            args.join(" "),
            String::from_utf8_lossy(&output.stdout)
        )
    })?;
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

    #[test]
    fn normalize_path_trims_trailing_separators() {
        assert_eq!(normalize_path("/tmp/"), "/tmp");
        assert_eq!(normalize_path(r"C:\\bin\\"), r"C:\\bin");
    }
}
