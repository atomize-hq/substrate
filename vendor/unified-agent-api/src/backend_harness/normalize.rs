use std::{
    collections::{BTreeMap, BTreeSet},
    ffi::OsString,
    path::{Component, Path, PathBuf},
    time::Duration,
};

use serde_json::Value;
#[cfg(windows)]
use std::path::Prefix;

use super::{BackendDefaults, BackendHarnessAdapter, NormalizedRequest};
use crate::backends::spawn_path::resolve_relative_path_from_base;
use crate::{AgentWrapperError, AgentWrapperRunRequest, EXT_AGENT_API_CONFIG_MODEL_V1};

const ADD_DIRS_KEY: &str = "dirs";
const ADD_DIRS_ROOT_INVALID: &str = "invalid agent_api.exec.add_dirs.v1";
const ADD_DIRS_CONTAINER_INVALID: &str = "invalid agent_api.exec.add_dirs.v1.dirs";
const ADD_DIRS_MAX_COUNT: usize = 16;
const ADD_DIRS_MAX_ENTRY_BYTES: usize = 1024;
const MODEL_ID_INVALID: &str = "invalid agent_api.config.model.v1";
const MODEL_ID_MAX_BYTES: usize = 128;

#[cfg(windows)]
type AddDirDedupeKey = String;
#[cfg(not(windows))]
type AddDirDedupeKey = PathBuf;

fn validate_extension_keys_fail_closed<A: BackendHarnessAdapter>(
    adapter: &A,
    request: &AgentWrapperRunRequest,
) -> Result<(), AgentWrapperError> {
    let supported: &[&str] = adapter.supported_extension_keys();
    for key in request.extensions.keys() {
        if !supported.contains(&key.as_str()) {
            return Err(AgentWrapperError::UnsupportedCapability {
                agent_kind: adapter.kind().as_str().to_string(),
                capability: key.clone(),
            });
        }
    }
    Ok(())
}

fn merge_env_backend_defaults_then_request(
    defaults: &BTreeMap<String, String>,
    request: &BTreeMap<String, String>,
) -> BTreeMap<String, String> {
    let mut merged = defaults.clone();
    merged.extend(request.clone());
    merged
}

fn derive_effective_timeout(
    request_timeout: Option<Duration>,
    default_timeout: Option<Duration>,
) -> Option<Duration> {
    request_timeout.or(default_timeout)
}

fn normalize_model_id_v1(raw: Option<&Value>) -> Result<Option<String>, AgentWrapperError> {
    let Some(raw) = raw else {
        return Ok(None);
    };

    let raw = raw.as_str().ok_or_else(invalid_model_id)?;
    let trimmed = raw.trim();
    if trimmed.is_empty() || trimmed.len() > MODEL_ID_MAX_BYTES {
        return Err(invalid_model_id());
    }

    Ok(Some(trimmed.to_string()))
}

pub(crate) fn accepted_model_override_v1(
    request: &AgentWrapperRunRequest,
) -> Result<bool, AgentWrapperError> {
    Ok(normalize_model_id_v1(request.extensions.get(EXT_AGENT_API_CONFIG_MODEL_V1))?.is_some())
}

pub(crate) fn normalize_add_dirs_v1(
    raw: Option<&Value>,
    effective_working_dir: Option<&Path>,
) -> Result<Vec<PathBuf>, AgentWrapperError> {
    let Some(raw) = raw else {
        return Ok(Vec::new());
    };

    let object = raw.as_object().ok_or_else(invalid_add_dirs_root)?;

    if object.len() != 1 || !object.contains_key(ADD_DIRS_KEY) {
        return Err(invalid_add_dirs_root());
    }

    let dirs = object
        .get(ADD_DIRS_KEY)
        .ok_or_else(invalid_add_dirs_root)?
        .as_array()
        .ok_or_else(invalid_add_dirs_container)?;

    if dirs.is_empty() || dirs.len() > ADD_DIRS_MAX_COUNT {
        return Err(invalid_add_dirs_container());
    }

    let mut normalized_dirs = Vec::with_capacity(dirs.len());
    let mut seen = BTreeSet::new();
    for (index, entry) in dirs.iter().enumerate() {
        let normalized = normalize_add_dir_entry(entry, index, effective_working_dir)?;
        if seen.insert(add_dir_dedupe_key(&normalized)) {
            normalized_dirs.push(normalized);
        }
    }

    Ok(normalized_dirs)
}

pub(crate) fn normalize_request<A: BackendHarnessAdapter>(
    adapter: &A,
    defaults: &BackendDefaults,
    request: AgentWrapperRunRequest,
) -> Result<NormalizedRequest<A::Policy>, AgentWrapperError> {
    if request.prompt.trim().is_empty() {
        return Err(AgentWrapperError::InvalidRequest {
            message: "prompt must not be empty".to_string(),
        });
    }

    validate_extension_keys_fail_closed(adapter, &request)?;
    let model_id = normalize_model_id_v1(request.extensions.get(EXT_AGENT_API_CONFIG_MODEL_V1))?;
    let policy = adapter.validate_and_extract_policy(&request)?;

    let env = merge_env_backend_defaults_then_request(&defaults.env, &request.env);
    let effective_timeout = derive_effective_timeout(request.timeout, defaults.default_timeout);

    let agent_kind = adapter.kind();
    let prompt = request.prompt;
    let working_dir = request.working_dir;

    Ok(NormalizedRequest {
        agent_kind,
        prompt,
        model_id,
        working_dir,
        effective_timeout,
        env,
        policy,
    })
}

#[allow(dead_code)]
fn parse_ext_bool(value: &Value, key: &str) -> Result<bool, AgentWrapperError> {
    value
        .as_bool()
        .ok_or_else(|| AgentWrapperError::InvalidRequest {
            message: format!("{key} must be a boolean"),
        })
}

#[allow(dead_code)]
fn parse_ext_string<'a>(value: &'a Value, key: &str) -> Result<&'a str, AgentWrapperError> {
    value
        .as_str()
        .ok_or_else(|| AgentWrapperError::InvalidRequest {
            message: format!("{key} must be a string"),
        })
}

#[allow(dead_code)]
fn parse_ext_string_enum<'a>(
    value: &'a Value,
    key: &str,
    allowed: &[&str],
) -> Result<&'a str, AgentWrapperError> {
    let raw = parse_ext_string(value, key)?;
    if allowed.contains(&raw) {
        return Ok(raw);
    }

    let allowed = allowed.join(" | ");
    Err(AgentWrapperError::InvalidRequest {
        message: format!("{key} must be one of: {allowed}"),
    })
}

fn normalize_add_dir_entry(
    value: &Value,
    index: usize,
    effective_working_dir: Option<&Path>,
) -> Result<PathBuf, AgentWrapperError> {
    let raw = value
        .as_str()
        .ok_or_else(|| invalid_add_dirs_entry(index))?;
    let trimmed = raw.trim();
    if trimmed.is_empty() || trimmed.len() > ADD_DIRS_MAX_ENTRY_BYTES {
        return Err(invalid_add_dirs_entry(index));
    }

    let candidate = Path::new(trimmed);
    let resolved = if candidate.is_absolute() {
        candidate.to_path_buf()
    } else {
        let effective_working_dir =
            effective_working_dir.ok_or_else(|| invalid_add_dirs_entry(index))?;
        #[cfg(windows)]
        reject_cross_drive_windows_add_dir(candidate, effective_working_dir, index)?;
        resolve_relative_path_from_base(effective_working_dir, candidate)
    };
    let normalized = lexical_normalize_path(&resolved);

    if !normalized.exists() || !normalized.is_dir() {
        return Err(invalid_add_dirs_entry(index));
    }

    Ok(normalized)
}

fn add_dir_dedupe_key(path: &Path) -> AddDirDedupeKey {
    #[cfg(windows)]
    {
        return path.as_os_str().to_string_lossy().to_lowercase();
    }

    #[cfg(not(windows))]
    {
        path.to_path_buf()
    }
}

fn invalid_model_id() -> AgentWrapperError {
    AgentWrapperError::InvalidRequest {
        message: MODEL_ID_INVALID.to_string(),
    }
}

#[cfg(windows)]
fn reject_cross_drive_windows_add_dir(
    candidate: &Path,
    effective_working_dir: &Path,
    index: usize,
) -> Result<(), AgentWrapperError> {
    let Some(candidate_drive) = windows_drive_relative_prefix(candidate) else {
        return Ok(());
    };
    let Some(effective_drive) = windows_disk_prefix(effective_working_dir) else {
        return Ok(());
    };

    if candidate_drive == effective_drive {
        return Ok(());
    }

    Err(invalid_add_dirs_entry(index))
}

#[cfg(windows)]
fn windows_drive_relative_prefix(path: &Path) -> Option<u8> {
    let mut components = path.components();
    let Component::Prefix(prefix) = components.next()? else {
        return None;
    };

    let drive = match prefix.kind() {
        Prefix::Disk(drive) | Prefix::VerbatimDisk(drive) => drive.to_ascii_lowercase(),
        _ => return None,
    };

    for component in components {
        if matches!(component, Component::RootDir) {
            return None;
        }
    }

    Some(drive)
}

#[cfg(windows)]
fn windows_disk_prefix(path: &Path) -> Option<u8> {
    path.components().find_map(|component| match component {
        Component::Prefix(prefix) => match prefix.kind() {
            Prefix::Disk(drive) | Prefix::VerbatimDisk(drive) => Some(drive.to_ascii_lowercase()),
            _ => None,
        },
        _ => None,
    })
}

fn lexical_normalize_path(path: &Path) -> PathBuf {
    let mut prefix: Option<OsString> = None;
    let mut has_root = false;
    let mut parts: Vec<OsString> = Vec::new();

    for component in path.components() {
        match component {
            Component::Prefix(value) => {
                prefix = Some(value.as_os_str().to_os_string());
            }
            Component::RootDir => {
                has_root = true;
            }
            Component::CurDir => {}
            Component::ParentDir => {
                if let Some(last) = parts.last() {
                    if last != ".." {
                        parts.pop();
                        continue;
                    }
                }

                if !has_root {
                    parts.push(component.as_os_str().to_os_string());
                }
            }
            Component::Normal(value) => parts.push(value.to_os_string()),
        }
    }

    let mut normalized = PathBuf::new();
    if let Some(prefix) = prefix {
        normalized.push(PathBuf::from(prefix));
    }
    if has_root {
        normalized.push(std::path::MAIN_SEPARATOR_STR);
    }
    for part in parts {
        normalized.push(part);
    }
    if normalized.as_os_str().is_empty() {
        normalized.push(".");
    }

    normalized
}

fn invalid_add_dirs_root() -> AgentWrapperError {
    AgentWrapperError::InvalidRequest {
        message: ADD_DIRS_ROOT_INVALID.to_string(),
    }
}

fn invalid_add_dirs_container() -> AgentWrapperError {
    AgentWrapperError::InvalidRequest {
        message: ADD_DIRS_CONTAINER_INVALID.to_string(),
    }
}

fn invalid_add_dirs_entry(index: usize) -> AgentWrapperError {
    AgentWrapperError::InvalidRequest {
        message: format!("{ADD_DIRS_CONTAINER_INVALID}[{index}]"),
    }
}

#[cfg(test)]
mod tests;
