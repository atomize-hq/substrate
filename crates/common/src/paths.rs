use anyhow::Result;
use std::path::PathBuf;

pub const SUBSTRATE_DIR_NAME: &str = ".substrate";
pub const SHIMS_SUBDIR: &str = "shims";
pub const OLD_SHIM_DIR: &str = ".cmdshim_rust";

pub fn substrate_home() -> Result<PathBuf> {
    if let Ok(override_home) = std::env::var("SUBSTRATE_HOME") {
        let trimmed = override_home.trim();
        if !trimmed.is_empty() {
            return Ok(PathBuf::from(trimmed));
        }
    }
    Ok(dirs::home_dir()
        .ok_or_else(|| anyhow::anyhow!("No home directory found"))?
        .join(SUBSTRATE_DIR_NAME))
}

pub fn shims_dir() -> Result<PathBuf> {
    Ok(substrate_home()?.join(SHIMS_SUBDIR))
}

pub fn old_shims_dir() -> Result<PathBuf> {
    // If SUBSTRATE_HOME is set (e.g., in tests), use its parent as the home base for the legacy dir
    if let Ok(override_home) = std::env::var("SUBSTRATE_HOME") {
        let base = PathBuf::from(override_home);
        let home_base = base
            .parent()
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("."));
        return Ok(home_base.join(OLD_SHIM_DIR));
    }
    Ok(dirs::home_dir()
        .ok_or_else(|| anyhow::anyhow!("No home directory found"))?
        .join(OLD_SHIM_DIR))
}

pub fn version_file() -> Result<PathBuf> {
    Ok(shims_dir()?.join(".version"))
}

pub fn lock_file() -> Result<PathBuf> {
    Ok(substrate_home()?.join(".substrate.lock"))
}

pub fn config_file() -> Result<PathBuf> {
    Ok(substrate_home()?.join("config.yaml"))
}

pub fn policy_file() -> Result<PathBuf> {
    Ok(substrate_home()?.join("policy.yaml"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_substrate_home() {
        let path = substrate_home().unwrap();
        assert!(path.ends_with(SUBSTRATE_DIR_NAME));
        assert!(path.is_absolute());
    }

    #[test]
    fn test_shims_dir() {
        let path = shims_dir().unwrap();
        assert!(
            path.ends_with(format!("{SUBSTRATE_DIR_NAME}/{SHIMS_SUBDIR}").as_str())
                || path.ends_with(format!("{SUBSTRATE_DIR_NAME}\\{SHIMS_SUBDIR}").as_str())
        );
        assert!(path.is_absolute());
    }

    #[test]
    fn test_old_shims_dir() {
        let path = old_shims_dir().unwrap();
        assert!(path.ends_with(OLD_SHIM_DIR));
        assert!(path.is_absolute());
    }

    #[test]
    fn test_version_file() {
        let path = version_file().unwrap();
        assert!(path.ends_with(".version"));
        assert!(path.parent().unwrap().ends_with(SHIMS_SUBDIR));
    }

    #[test]
    fn test_lock_file() {
        let path = lock_file().unwrap();
        assert!(path.ends_with(".substrate.lock"));
        assert!(path.parent().unwrap().ends_with(SUBSTRATE_DIR_NAME));
    }

    #[test]
    fn test_config_file() {
        let path = config_file().unwrap();
        assert!(path.ends_with("config.yaml"));
        assert!(path.parent().unwrap().ends_with(SUBSTRATE_DIR_NAME));
    }

    #[test]
    fn test_paths_are_consistent() {
        let substrate = substrate_home().unwrap();
        let shims = shims_dir().unwrap();
        let version = version_file().unwrap();
        let lock = lock_file().unwrap();

        assert!(shims.starts_with(&substrate));
        assert!(version.starts_with(&shims));
        assert!(lock.starts_with(&substrate));
    }
}
