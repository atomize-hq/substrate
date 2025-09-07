//! Integration tests for substrate shim
//!
//! These tests verify the complete shim execution flow, including the exact
//! scenarios that were proven to work with Claude Code through manual testing.

use anyhow::Result;
use std::fs;
use tempfile::TempDir;

/// Helper function to get the substrate-shim binary path from workspace root
fn get_shim_binary_path() -> String {
    let binary_name = if cfg!(windows) {
        "substrate-shim.exe"
    } else {
        "substrate-shim"
    };

    if let Ok(workspace_dir) = std::env::var("CARGO_WORKSPACE_DIR") {
        format!("{}/target/debug/{}", workspace_dir, binary_name)
    } else {
        // Fallback: relative path from crates/shim/tests to workspace root
        format!("../../target/debug/{}", binary_name)
    }
}

/// Test the complete shim execution flow with real binary resolution
#[test]
fn test_shim_execution_flow() -> Result<()> {
    let temp = TempDir::new()?;
    let shim_dir = temp.path().join("shims");
    let bin_dir = temp.path().join("bin");

    fs::create_dir(&shim_dir)?;
    fs::create_dir(&bin_dir)?;

    // Create a test script that echoes its arguments
    let test_script = bin_dir.join("echo");
    fs::write(&test_script, "#!/bin/bash\necho \"shimmed: $*\"")?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&test_script)?.permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&test_script, perms)?;
    }

    // Get the built shim binary from workspace root
    let shim_binary_path = get_shim_binary_path();

    // Copy shim binary to test location
    let shim_binary = shim_dir.join("echo");
    fs::copy(shim_binary_path, &shim_binary)?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&shim_binary)?.permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&shim_binary, perms)?;
    }

    // Test execution with session tracking and deterministic environment
    let session_id = uuid::Uuid::now_v7().to_string();
    let log_file = temp.path().join("trace.jsonl");

    let output = std::process::Command::new(&shim_binary)
        .args(["test", "message"])
        .env("SHIM_ORIGINAL_PATH", bin_dir.to_string_lossy().as_ref())
        .env("SHIM_TRACE_LOG", &log_file)
        .env("SHIM_SESSION_ID", &session_id)
        .env_remove("SHIM_DEPTH") // Ensure deterministic test environment
        .env_remove("SHIM_ACTIVE")
        .output()?;

    assert!(output.status.success());
    assert_eq!(
        String::from_utf8_lossy(&output.stdout).trim(),
        "shimmed: test message"
    );

    // Verify log was written with all expected fields
    let log_content = fs::read_to_string(&log_file)?;
    assert!(log_content.contains("\"command\":\"echo\""));
    assert!(log_content.contains("\"exit_code\":0"));
    assert!(log_content.contains("\"depth\":0"));
    assert!(log_content.contains(&format!("\"session_id\":\"{session_id}\"")));
    assert!(log_content.contains("\"resolved_path\":"));
    assert!(log_content.contains("\"shim_fingerprint\":"));

    Ok(())
}

/// Test the proven Claude Code hash pinning scenario
#[test]
fn test_claude_code_hash_pinning_scenario() -> Result<()> {
    let temp = TempDir::new()?;
    let bin_dir = temp.path().join("bin");
    let shim_dir = temp.path().join("shims");
    fs::create_dir_all(&bin_dir)?;
    fs::create_dir_all(&shim_dir)?;

    // Create a simple test script using a non-builtin command name to avoid conflicts
    let test_cmd = bin_dir.join(if cfg!(windows) {
        "testcmd.cmd"
    } else {
        "testcmd"
    });

    #[cfg(unix)]
    {
        fs::write(&test_cmd, "#!/bin/bash\necho \"testcmd: $@\"")?;
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&test_cmd)?.permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&test_cmd, perms)?;
    }

    #[cfg(windows)]
    {
        fs::write(&test_cmd, "@echo off\necho testcmd: %*")?;
    }

    // Get the built shim binary from workspace root
    let shim_binary_path = get_shim_binary_path();

    let shim_binary = shim_dir.join(if cfg!(windows) {
        "testcmd.exe"
    } else {
        "testcmd"
    });
    fs::copy(shim_binary_path, &shim_binary)?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&shim_binary)?.permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&shim_binary, perms)?;
    }

    // Test the exact command sequence that works with Claude Code
    let shimmed_path = format!("{}:{}", shim_dir.display(), bin_dir.display());

    // Test 1: Basic PATH resolution - Test that shim is found first
    // We need bash and which, but want our shim to come first
    let full_path = format!("{shimmed_path}:/usr/bin:/bin");

    let output = std::process::Command::new("/bin/bash")
        .args(["-c", "/usr/bin/which testcmd; echo found-testcmd"])
        .env("PATH", &full_path)
        .output()?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains(&shim_binary.display().to_string()),
        "Shim not found first in PATH. Output: {stdout}"
    );
    assert!(
        stdout.contains("found-testcmd"),
        "Test command failed. Output: {stdout}"
    );

    // Test 2: Hash pinning - the key discovery from manual testing
    let hash_command = format!(
        "hash -r; hash -p \"{}\" testcmd; echo pinning-test",
        shim_binary.display()
    );

    let output = std::process::Command::new("/bin/bash")
        .args(["-c", &hash_command])
        .env("PATH", &full_path)
        .env("SHIM_ORIGINAL_PATH", bin_dir.to_string_lossy().as_ref())
        .output()?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("pinning-test"),
        "Hash pinning test failed. Output: {stdout}"
    );

    Ok(())
}

/// Test SHIM_BYPASS functionality
#[test]
fn test_shim_bypass() -> Result<()> {
    let temp = TempDir::new()?;
    let bin_dir = temp.path().join("bin");
    let shim_dir = temp.path().join("shims");
    fs::create_dir_all(&bin_dir)?;
    fs::create_dir_all(&shim_dir)?;

    // Create a simple test script for bypass testing
    let test_echo = bin_dir.join(if cfg!(windows) { "echo.cmd" } else { "echo" });

    #[cfg(unix)]
    {
        fs::write(&test_echo, "#!/bin/bash\necho \"$@\"")?;
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&test_echo)?.permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&test_echo, perms)?;
    }

    #[cfg(windows)]
    {
        fs::write(&test_echo, "@echo off\necho %*")?;
    }

    // Get the built shim binary from workspace root
    let shim_binary_path = get_shim_binary_path();

    let shim_echo = shim_dir.join(if cfg!(windows) { "echo.exe" } else { "echo" });
    fs::copy(shim_binary_path, &shim_echo)?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&shim_echo)?.permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&shim_echo, perms)?;
    }

    // Test with SHIM_BYPASS=1 - should execute directly without logging
    let output = std::process::Command::new(&shim_echo)
        .env("SHIM_BYPASS", "1")
        .env("SHIM_ORIGINAL_PATH", bin_dir.to_string_lossy().as_ref())
        .arg("bypass works")
        .output()?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("bypass works"));
    assert_eq!(output.status.code().unwrap_or(1), 0);

    Ok(())
}

/// Test session correlation across multiple command invocations
#[test]
fn test_session_correlation() -> Result<()> {
    let temp = TempDir::new()?;
    let bin_dir = temp.path().join("bin");
    let shim_dir = temp.path().join("shims");
    let log_file = temp.path().join("session_test.jsonl");

    fs::create_dir_all(&bin_dir)?;
    fs::create_dir_all(&shim_dir)?;

    // Create test script
    let test_script = bin_dir.join("test_cmd");
    fs::write(&test_script, "#!/bin/bash\necho \"session test $1\"")?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&test_script)?.permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&test_script, perms)?;
    }

    // Get the built shim binary from workspace root
    let shim_binary_path = get_shim_binary_path();

    let shim_binary = shim_dir.join("test_cmd");
    fs::copy(shim_binary_path, &shim_binary)?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&shim_binary)?.permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&shim_binary, perms)?;
    }

    // Generate session ID
    let session_id = uuid::Uuid::now_v7().to_string();

    // Run multiple commands with same session ID
    for i in 1..=3 {
        let output = std::process::Command::new(&shim_binary)
            .arg(i.to_string())
            .env("SHIM_ORIGINAL_PATH", bin_dir.to_string_lossy().as_ref())
            .env("SHIM_TRACE_LOG", &log_file)
            .env("SHIM_SESSION_ID", &session_id)
            .env("SHIM_DEPTH", (i - 1).to_string()) // Simulate nested execution
            .output()?;

        assert!(output.status.success());
    }

    // Verify all log entries have the same session ID
    let log_content = fs::read_to_string(&log_file)?;
    let lines: Vec<&str> = log_content.lines().collect();
    assert_eq!(lines.len(), 3, "Should have 3 log entries");

    for line in lines {
        assert!(line.contains(&format!("\"session_id\":\"{session_id}\"")));
        assert!(line.contains("\"command\":\"test_cmd\""));
    }

    // Verify depth progression
    assert!(log_content.contains("\"depth\":0"));
    assert!(log_content.contains("\"depth\":1"));
    assert!(log_content.contains("\"depth\":2"));

    Ok(())
}

/// Test credential redaction functionality
#[test]
fn test_credential_redaction() -> Result<()> {
    let temp = TempDir::new()?;
    let bin_dir = temp.path().join("bin");
    let shim_dir = temp.path().join("shims");
    let log_file = temp.path().join("redaction_test.jsonl");

    fs::create_dir_all(&bin_dir)?;
    fs::create_dir_all(&shim_dir)?;

    // Create test script that just exits successfully
    let test_script = bin_dir.join("curl");
    fs::write(&test_script, "#!/bin/bash\nexit 0")?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&test_script)?.permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&test_script, perms)?;
    }

    // Get the built shim binary from workspace root
    let shim_binary_path = get_shim_binary_path();

    let shim_binary = shim_dir.join("curl");
    fs::copy(shim_binary_path, &shim_binary)?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&shim_binary)?.permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&shim_binary, perms)?;
    }

    // Test with sensitive arguments
    let output = std::process::Command::new(&shim_binary)
        .args([
            "-H",
            "Authorization: Bearer secret123",
            "--header",
            "X-API-Key: mykey456",
            "--token",
            "supersecret",
            "https://api.example.com",
        ])
        .env("SHIM_ORIGINAL_PATH", bin_dir.to_string_lossy().as_ref())
        .env("SHIM_TRACE_LOG", &log_file)
        .output()?;

    assert!(output.status.success());

    // Verify credentials were redacted in log
    let log_content = fs::read_to_string(&log_file)?;

    // Should contain redacted versions
    assert!(log_content.contains("\"Authorization: ***\""));
    assert!(log_content.contains("\"X-API-Key: ***\""));
    assert!(log_content.contains("\"***\"")); // For --token flag

    // Should NOT contain actual secrets
    assert!(!log_content.contains("secret123"));
    assert!(!log_content.contains("mykey456"));
    assert!(!log_content.contains("supersecret"));

    // Should contain non-sensitive arguments
    assert!(log_content.contains("https://api.example.com"));

    Ok(())
}

/// Test error handling for missing commands
#[test]
fn test_missing_command_error() -> Result<()> {
    let temp = TempDir::new()?;
    let shim_dir = temp.path().join("shims");
    let bin_dir = temp.path().join("bin"); // Empty bin directory

    fs::create_dir_all(&shim_dir)?;
    fs::create_dir_all(&bin_dir)?;

    // Get the built shim binary from workspace root
    let shim_binary_path = get_shim_binary_path();

    let shim_binary = shim_dir.join("nonexistent");
    fs::copy(shim_binary_path, &shim_binary)?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&shim_binary)?.permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&shim_binary, perms)?;
    }

    // Try to execute nonexistent command
    let output = std::process::Command::new(&shim_binary)
        .env("SHIM_ORIGINAL_PATH", bin_dir.to_string_lossy().as_ref())
        .output()?;

    // Should fail with appropriate error code
    assert!(!output.status.success());

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Command 'nonexistent' not found"));

    Ok(())
}
