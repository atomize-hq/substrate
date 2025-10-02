#[cfg(test)]
mod tests {
    use serial_test::serial;
    use std::fs;
    use std::path::{Path, PathBuf};
    use substrate_shell::shim_deploy::{DeploymentStatus, ShimDeployer};
    use tempfile::TempDir;

    /// Helper to create a mock substrate-shim binary for testing
    fn create_mock_shim_binary(dir: &Path) -> PathBuf {
        let shim_name = if cfg!(windows) {
            "substrate-shim.exe"
        } else {
            "substrate-shim"
        };

        let shim_path = dir.join(shim_name);

        // Create a dummy file
        fs::write(&shim_path, b"#!/bin/sh\necho mock shim\n").expect("Failed to create mock shim");

        // Make it executable on Unix
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(&shim_path)
                .expect("Failed to get metadata")
                .permissions();
            perms.set_mode(0o755);
            fs::set_permissions(&shim_path, perms).expect("Failed to set permissions");
        }

        shim_path
    }

    /// Helper to set up test environment with mock binary
    fn setup_test_env() -> (TempDir, PathBuf, Option<String>, Option<String>) {
        let temp = TempDir::new().unwrap();
        let original_home = std::env::var("HOME").ok();
        let original_path = std::env::var("PATH").ok();
        #[cfg(windows)]
        let original_userprofile = std::env::var("USERPROFILE").ok();

        // Point Substrate home to the temp directory (portable across OSes)
        let sub_home = temp.path().join(".substrate");
        std::env::set_var("SUBSTRATE_HOME", &sub_home);
        // Also set HOME/USERPROFILE for any code that still consults them indirectly
        std::env::set_var("HOME", temp.path());
        #[cfg(windows)]
        std::env::set_var("USERPROFILE", temp.path());

        // Create a bin directory for our mock shim
        let bin_dir = temp.path().join("bin");
        fs::create_dir(&bin_dir).unwrap();

        // Create mock substrate-shim
        let mock_shim = create_mock_shim_binary(&bin_dir);

        // Add bin directory to PATH
        #[cfg(windows)]
        let new_path = format!(
            "{};{}",
            bin_dir.display(),
            original_path
                .as_deref()
                .unwrap_or(r"C:\\Windows\\System32;C:\\Windows")
        );
        #[cfg(not(windows))]
        let new_path = format!(
            "{}:{}",
            bin_dir.display(),
            original_path.as_deref().unwrap_or("/usr/bin:/bin")
        );
        std::env::set_var("PATH", &new_path);

        (temp, mock_shim, original_home, original_path)
    }

    /// Helper to restore environment after test
    fn restore_env(original_home: Option<String>, original_path: Option<String>) {
        std::env::remove_var("SUBSTRATE_HOME");
        if let Some(home) = original_home {
            std::env::set_var("HOME", home);
        } else {
            std::env::remove_var("HOME");
        }

        #[cfg(windows)]
        {
            if let Some(up) = std::env::var("USERPROFILE").ok() {
                let _ = up; // suppress unused warning if not needed
            }
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
        let shims_dir = substrate_common::paths::shims_dir().unwrap();
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
        let (_temp, _mock_shim, original_home, original_path) = setup_test_env();

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

    #[test]
    #[serial]
    fn test_version_file_content() {
        let (_temp, _mock_shim, original_home, original_path) = setup_test_env();

        let deployer = ShimDeployer::with_skip(false).unwrap();
        deployer.ensure_deployed().unwrap();

        // Check version file content
        let version_file = substrate_common::paths::version_file().unwrap();
        let content = fs::read_to_string(&version_file).unwrap();
        let version_info: serde_json::Value = serde_json::from_str(&content).unwrap();

        // Verify all required fields
        assert!(
            version_info.get("version").is_some(),
            "Version field should exist"
        );
        assert!(
            version_info.get("deployed_at").is_some(),
            "Deployed_at field should exist"
        );
        assert!(
            version_info.get("commands").is_some(),
            "Commands field should exist"
        );

        // Check version matches
        let current_version = env!("CARGO_PKG_VERSION");
        assert_eq!(
            version_info.get("version").unwrap().as_str().unwrap(),
            current_version
        );

        restore_env(original_home, original_path);
    }

    #[test]
    #[serial]
    fn test_corrupted_version_file_recovery() {
        let (_temp, _mock_shim, original_home, original_path) = setup_test_env();

        // First deployment
        let deployer = ShimDeployer::with_skip(false).unwrap();
        deployer.ensure_deployed().unwrap();

        // Corrupt the version file
        let version_file = substrate_common::paths::version_file().unwrap();
        fs::write(&version_file, "invalid json").unwrap();

        // Second deployment should fail due to corrupted file but not panic
        let deployer2 = ShimDeployer::with_skip(false).unwrap();
        let result = deployer2.ensure_deployed();

        // The deployment will fail with an error (not DeploymentStatus::Failed)
        // because is_deployment_needed() returns an error
        assert!(
            result.is_err(),
            "Should return error when version file is corrupted"
        );

        // Manually remove the corrupted file to test recovery
        fs::remove_file(&version_file).unwrap();

        // Third deployment should succeed after removing corrupted file
        let deployer3 = ShimDeployer::with_skip(false).unwrap();
        let status = deployer3.ensure_deployed().unwrap();
        assert_eq!(status, DeploymentStatus::Deployed);

        // Version file should now be valid
        let content = fs::read_to_string(&version_file).unwrap();
        assert!(serde_json::from_str::<serde_json::Value>(&content).is_ok());

        restore_env(original_home, original_path);
    }

    #[test]
    #[serial]
    fn test_deployment_with_existing_substrate_dir() {
        let (temp, _mock_shim, original_home, original_path) = setup_test_env();

        // Pre-create substrate directory with existing content
        let substrate_dir = substrate_common::paths::substrate_home().unwrap();
        fs::create_dir_all(&substrate_dir).unwrap();
        fs::write(substrate_dir.join("existing.txt"), "content").unwrap();

        let deployer = ShimDeployer::with_skip(false).unwrap();
        deployer.ensure_deployed().unwrap();

        // Existing files should be preserved
        assert!(substrate_dir.join("existing.txt").exists());
        assert!(substrate_dir.join("shims").exists());

        restore_env(original_home, original_path);
    }

    #[test]
    #[serial]
    fn test_multi_instance_locking() {
        use std::sync::{Arc, Barrier};
        use std::thread;
        use std::time::Duration;

        let (_temp, _mock_shim, original_home, original_path) = setup_test_env();

        // Barrier to synchronize thread starts
        let barrier = Arc::new(Barrier::new(2));
        let completed = Arc::new(std::sync::Mutex::new(Vec::new()));

        let barrier1 = barrier.clone();
        let completed1 = completed.clone();
        let handle1 = thread::spawn(move || {
            barrier1.wait();
            let deployer = ShimDeployer::with_skip(false).unwrap();
            if let Ok(status) = deployer.ensure_deployed() {
                completed1.lock().unwrap().push((1, status));
            }
        });

        let barrier2 = barrier.clone();
        let completed2 = completed.clone();
        let handle2 = thread::spawn(move || {
            barrier2.wait();
            thread::sleep(Duration::from_millis(10));
            let deployer = ShimDeployer::with_skip(false).unwrap();
            if let Ok(status) = deployer.ensure_deployed() {
                completed2.lock().unwrap().push((2, status));
            }
        });

        handle1.join().unwrap();
        handle2.join().unwrap();

        // At least one should have succeeded
        let results = completed.lock().unwrap();
        assert!(
            !results.is_empty(),
            "At least one deployment should succeed"
        );

        // Check that shims exist
        let shims_dir = substrate_common::paths::shims_dir().unwrap();
        assert!(shims_dir.exists());

        restore_env(original_home, original_path);
    }

    #[test]
    #[serial]
    #[cfg(unix)]
    fn test_shim_permissions() {
        use std::os::unix::fs::PermissionsExt;

        let (_temp, _mock_shim, original_home, original_path) = setup_test_env();

        let deployer = ShimDeployer::with_skip(false).unwrap();
        deployer.ensure_deployed().unwrap();

        // Check substrate-shim binary permissions if it exists
        let shim_binary = substrate_common::paths::substrate_home()
            .unwrap()
            .join("shims")
            .join("substrate-shim");

        if shim_binary.exists() {
            let metadata = fs::metadata(&shim_binary).unwrap();
            let mode = metadata.permissions().mode();

            // Should be executable
            assert!(mode & 0o100 != 0, "Binary should be executable");
        }

        restore_env(original_home, original_path);
    }
}
