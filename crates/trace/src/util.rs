use anyhow::Result;
use sha2::{Digest, Sha256};
use std::env;
use std::fs;
use std::sync::{Mutex, OnceLock};
use std::time::SystemTime;

pub fn hash_env_vars() -> Result<String> {
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

    Ok(format!("{:x}", hasher.finalize()))
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

pub fn get_policy_git_hash() -> Result<Option<String>> {
    use std::path::PathBuf;
    use std::process::Command;

    let policy_dir = dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("/tmp"))
        .join(".substrate");

    let cache_disabled = env::var("SUBSTRATE_POLICY_GIT_CACHE")
        .map(|v| v == "0" || v.eq_ignore_ascii_case("false"))
        .unwrap_or(false);

    let head_path = policy_dir.join(".git/HEAD");
    let head_mtime = fs::metadata(&head_path).and_then(|m| m.modified()).ok();

    let cache_mutex = POLICY_GIT_HASH.get_or_init(|| Mutex::new(CachedPolicyHash::default()));
    let mut cache = cache_mutex.lock().expect("policy git hash cache poisoned");

    if !cache_disabled && head_mtime == cache.mtime {
        return Ok(cache.hash.clone());
    }

    let result = if policy_dir.join(".git").exists() {
        let output = Command::new("git")
            .args(["rev-parse", "HEAD"])
            .current_dir(&policy_dir)
            .output()?;

        if output.status.success() {
            let hash = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if hash.is_empty() {
                None
            } else {
                Some(hash)
            }
        } else {
            None
        }
    } else {
        None
    };

    cache.mtime = head_mtime;
    cache.hash = result.clone();

    Ok(result)
}
