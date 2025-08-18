#[cfg(test)]
mod tests {
    use substrate::shim_deploy::{DeploymentStatus, ShimDeployer};
    use tempfile::TempDir;
    use std::fs;
    use std::path::{Path, PathBuf};
    use serial_test::serial;

    /// Helper to create a mock substrate-shim binary for testing
    fn create_mock_shim_binary(dir: &Path) -> PathBuf {
        let shim_name = if cfg!(windows) {
            "substrate-shim.exe"
        } else {
            "substrate-shim"
        };
        
        let shim_path = dir.join(shim_name);
        
        // Create a dummy file
        fs::write(&shim_path, b"#!/bin/sh\necho mock shim\n")
            .expect("Failed to create mock shim");
        
        // Make it executable on Unix
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(&shim_path)
                .expect("Failed to get metadata")
                .permissions();
            perms.set_mode(0o755);
            fs::set_permissions(&shim_path, perms)
                .expect("Failed to set permissions");
        }
        
        shim_path
    }

    /// Helper to set up test environment with mock binary
    fn setup_test_env() -> (TempDir, PathBuf, Option<String>, Option<String>) {
        let temp = TempDir::new().unwrap();
        let original_home = std::env::var("HOME").ok();
        let original_path = std::env::var("PATH").ok();
        
        // Set HOME to temp directory
        std::env::set_var("HOME", temp.path());
        
        // Create a bin directory for our mock shim
        let bin_dir = temp.path().join("bin");
        fs::create_dir(&bin_dir).unwrap();
        
        // Create mock substrate-shim
        let mock_shim = create_mock_shim_binary(&bin_dir);
        
        // Add bin directory to PATH
        let new_path = format!("{}:{}", 
            bin_dir.display(),
            original_path.as_deref().unwrap_or("/usr/bin:/bin")
        );
        std::env::set_var("PATH", &new_path);
        
        (temp, mock_shim, original_home, original_path)
    }

    /// Helper to restore environment after test
    fn restore_env(original_home: Option<String>, original_path: Option<String>) {
        if let Some(home) = original_home {
            std::env::set_var("HOME", home);
        } else {
            std::env::remove_var("HOME");
        }
        
        if let Some(path) = original_path {
            std::env::set_var("PATH", path);
        } else {
            std::env::remove_var("PATH");
        }
    }

    #[test]
    #[serial]
    fn test_skip_deployment() {
        // Test that skip flag works
        let deployer = ShimDeployer::with_skip(true).unwrap();
        let status = deployer.ensure_deployed().unwrap();
        assert_eq!(status, DeploymentStatus::Skipped);
    }

    #[test]
    #[serial]
    fn test_deployment_with_env_var() {
        // Test SUBSTRATE_NO_SHIMS environment variable
        std::env::set_var("SUBSTRATE_NO_SHIMS", "1");
        let deployer = ShimDeployer::with_skip(false).unwrap();
        let status = deployer.ensure_deployed().unwrap();
        assert_eq!(status, DeploymentStatus::Skipped);
        std::env::remove_var("SUBSTRATE_NO_SHIMS");
    }

    #[test]
    #[serial]
    fn test_clean_deployment() {
        // Clear any lingering environment variables
        std::env::remove_var("SUBSTRATE_NO_SHIMS");
        
        let (temp, _mock_shim, original_home, original_path) = setup_test_env();
        
        let deployer = ShimDeployer::with_skip(false).unwrap();
        let status = deployer.ensure_deployed().unwrap();
        
        // Should deploy successfully with mock binary
        assert_eq!(status, DeploymentStatus::Deployed);
        
        // Verify shims directory exists
        let shims_dir = temp.path().join(".substrate/shims");
        assert!(shims_dir.exists(), "Shims directory should exist");
        
        // Verify at least one shim exists
        let git_shim = shims_dir.join("git");
        assert!(git_shim.exists(), "Git shim should exist");
        
        // Verify version file exists
        let version_file = shims_dir.join(".version");
        assert!(version_file.exists(), "Version file should exist");
        
        restore_env(original_home, original_path);
    }

    #[test]
    #[serial]
    fn test_no_redeployment_when_current() {
        let (temp, _mock_shim, original_home, original_path) = setup_test_env();
        
        let deployer = ShimDeployer::with_skip(false).unwrap();
        
        // First deployment
        let status1 = deployer.ensure_deployed().unwrap();
        assert_eq!(status1, DeploymentStatus::Deployed);
        
        // Second deployment should detect current version
        let status2 = deployer.ensure_deployed().unwrap();
        assert_eq!(status2, DeploymentStatus::Current);
        
        restore_env(original_home, original_path);
    }

    #[test]
    #[serial]
    #[cfg(unix)]
    fn test_symlink_creation() {
        let (temp, mock_shim, original_home, original_path) = setup_test_env();
        
        let deployer = ShimDeployer::with_skip(false).unwrap();
        let _ = deployer.ensure_deployed();
        
        let git_shim = temp.path().join(".substrate/shims/git");
        if git_shim.exists() {
            let metadata = std::fs::symlink_metadata(&git_shim).unwrap();
            assert!(metadata.file_type().is_symlink(), "Should be a symlink");
            
            // Verify symlink points to mock shim
            let target = std::fs::read_link(&git_shim).unwrap();
            assert_eq!(target, mock_shim, "Symlink should point to mock shim");
        }
        
        restore_env(original_home, original_path);
    }

    #[test]
    #[serial]
    fn test_migration_from_old_directory() {
        let (temp, _mock_shim, original_home, original_path) = setup_test_env();
        
        // Create old shims directory with a file
        let old_dir = temp.path().join(".cmdshim_rust");
        fs::create_dir(&old_dir).unwrap();
        fs::write(old_dir.join("test_file"), b"test content").unwrap();
        
        let deployer = ShimDeployer::with_skip(false).unwrap();
        let status = deployer.ensure_deployed().unwrap();
        
        // Should successfully migrate and deploy
        assert_eq!(status, DeploymentStatus::Deployed);
        
        // Old directory should not exist (it was moved)
        assert!(!old_dir.exists(), "Old directory should be moved");
        
        // New directory should exist
        let new_dir = temp.path().join(".substrate/shims");
        assert!(new_dir.exists(), "New directory should exist");
        
        restore_env(original_home, original_path);
    }
}