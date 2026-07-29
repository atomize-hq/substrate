use anyhow::Result;
use sha2::{Digest, Sha256};
use std::env;
use std::fs;
use std::path::{Component, Path};
use std::sync::{Mutex, OnceLock};
use std::time::SystemTime;

static ENV_HASH_CACHE: OnceLock<String> = OnceLock::new();

pub fn hash_env_vars() -> Result<String> {
    if let Some(cached) = ENV_HASH_CACHE.get() {
        return Ok(cached.clone());
    }

    let mut hasher = Sha256::new();

    let mut vars = env::vars()
        .filter(|(key, _)| !key.starts_with("SHIM_") && !key.starts_with("SUBSTRATE_"))
        .collect::<Vec<_>>();
    vars.sort_by(|(key_a, value_a), (key_b, value_b)| {
        key_a.cmp(key_b).then_with(|| value_a.cmp(value_b))
    });

    for (key, value) in vars {
        hasher.update(key.as_bytes());
        hasher.update(b"=");
        hasher.update(value.as_bytes());
        hasher.update(b"\n");
    }

    let digest = format!("{:x}", hasher.finalize());
    let _ = ENV_HASH_CACHE.set(digest.clone());
    Ok(digest)
}

pub fn get_umask() -> Result<u32> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let temp = tempfile::tempfile()?;
        let metadata = temp.metadata()?;
        let mode = metadata.permissions().mode();
        Ok(0o777 - (mode & 0o777))
    }

    #[cfg(not(unix))]
    {
        Ok(0o022)
    }
}

#[derive(Clone, Debug, Default)]
struct CachedPolicyHash {
    mtime: Option<SystemTime>,
    hash: Option<String>,
}

static POLICY_GIT_HASH: OnceLock<Mutex<CachedPolicyHash>> = OnceLock::new();

pub(crate) fn get_policy_git_hash_at(policy_dir: &Path) -> Result<Option<String>> {
    let normalized = policy_dir.components().collect::<std::path::PathBuf>();
    if !policy_dir.is_absolute()
        || normalized.as_os_str() != policy_dir.as_os_str()
        || policy_dir
            .components()
            .any(|component| matches!(component, Component::CurDir | Component::ParentDir))
    {
        return Ok(None);
    }

    let git_dir = policy_dir.join(".git");
    let Ok(git_metadata) = fs::symlink_metadata(&git_dir) else {
        return Ok(None);
    };
    if !git_metadata.file_type().is_dir() || fs::symlink_metadata(git_dir.join("commondir")).is_ok()
    {
        return Ok(None);
    }

    let Some(head) = read_regular_git_metadata_file(&git_dir, Path::new("HEAD")) else {
        return Ok(None);
    };
    let head = head.trim();
    if is_git_object_id(head) {
        return Ok(Some(head.to_ascii_lowercase()));
    }
    let Some(reference) = head.strip_prefix("ref: ") else {
        return Ok(None);
    };
    let reference_path = Path::new(reference);
    if !reference.starts_with("refs/")
        || reference_path.is_absolute()
        || reference_path
            .components()
            .any(|component| !matches!(component, Component::Normal(_)))
    {
        return Ok(None);
    }

    if let Some(value) = read_regular_git_metadata_file(&git_dir, reference_path) {
        let value = value.trim();
        return Ok(is_git_object_id(value).then(|| value.to_ascii_lowercase()));
    }

    let Some(packed_refs) = read_regular_git_metadata_file(&git_dir, Path::new("packed-refs"))
    else {
        return Ok(None);
    };
    for line in packed_refs.lines() {
        if line.starts_with('#') || line.starts_with('^') {
            continue;
        }
        let Some((object_id, name)) = line.split_once(' ') else {
            continue;
        };
        if name == reference && is_git_object_id(object_id) {
            return Ok(Some(object_id.to_ascii_lowercase()));
        }
    }
    Ok(None)
}

fn read_regular_git_metadata_file(git_dir: &Path, relative: &Path) -> Option<String> {
    let components = relative.components().collect::<Vec<_>>();
    if components.is_empty()
        || components
            .iter()
            .any(|component| !matches!(component, Component::Normal(_)))
    {
        return None;
    }

    let mut path = git_dir.to_path_buf();
    for (index, component) in components.iter().enumerate() {
        path.push(component.as_os_str());
        let metadata = fs::symlink_metadata(&path).ok()?;
        let is_last = index + 1 == components.len();
        if (is_last && !metadata.file_type().is_file())
            || (!is_last && !metadata.file_type().is_dir())
        {
            return None;
        }
    }
    fs::read_to_string(path).ok()
}

fn is_git_object_id(value: &str) -> bool {
    matches!(value.len(), 40 | 64) && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}

pub fn get_policy_git_hash() -> Result<Option<String>> {
    let cache_disabled = env::var("SUBSTRATE_POLICY_GIT_CACHE")
        .map(|v| v == "0" || v.eq_ignore_ascii_case("false"))
        .unwrap_or(false);

    let cache_mutex = POLICY_GIT_HASH.get_or_init(|| Mutex::new(CachedPolicyHash::default()));
    let mut cache = cache_mutex.lock().expect("policy git hash cache poisoned");

    if !cache_disabled && cache.mtime.is_some() {
        return Ok(cache.hash.clone());
    }

    cache.mtime = Some(SystemTime::now());
    cache.hash = None;

    Ok(None)
}
