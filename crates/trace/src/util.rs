use anyhow::Result;
use sha2::{Digest, Sha256};
use std::env;

pub fn hash_env_vars() -> Result<String> {
    let mut hasher = Sha256::new();

    for (key, value) in env::vars() {
        if !key.starts_with("SHIM_") && !key.starts_with("SUBSTRATE_") {
            hasher.update(format!("{}={}\n", key, value));
        }
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

pub fn get_policy_git_hash() -> Result<String> {
    use std::path::PathBuf;
    use std::process::Command;

    let output = Command::new("git")
        .args(["rev-parse", "HEAD"])
        .current_dir(
            dirs::home_dir()
                .unwrap_or_else(|| PathBuf::from("/tmp"))
                .join(".substrate"),
        )
        .output()?;

    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}
