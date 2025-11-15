#![cfg(unix)]
//! Integration tests for substrate shim
//!
//! These tests verify the complete shim execution flow, including the exact
//! scenarios that were proven to work with Claude Code through manual testing.

use anyhow::Result;
use std::fs;
use tempfile::TempDir;

/// Helper function to get the substrate-shim binary path from workspace root
fn get_shim_binary_path() -> String {
    if let Some(bin) = option_env!("CARGO_BIN_EXE_substrate_shim_test_bin") {
        return bin.to_string();
    }
    if let Ok(bin) = std::env::var("CARGO_BIN_EXE_substrate_shim_test_bin") {
        return bin;
    }

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

    let shimmed_path = format!("{}:{}", shim_dir.display(), bin_dir.display());

    let output = std::process::Command::new(&shim_binary)
        .args(["test", "message"])
        .env("SHIM_ORIGINAL_PATH", bin_dir.to_string_lossy().as_ref())
        .env("SHIM_TRACE_LOG", &log_file)
        .env("SHIM_SESSION_ID", &session_id)
        .env("PATH", &shimmed_path)
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

    let shimmed_path = format!("{}:{}", shim_dir.display(), bin_dir.display());

    // Test with SHIM_BYPASS=1 - should execute directly without logging
    let output = std::process::Command::new(&shim_echo)
        .env("SHIM_BYPASS", "1")
        .env("SHIM_ORIGINAL_PATH", bin_dir.to_string_lossy().as_ref())
        .env("PATH", &shimmed_path)
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

    let shimmed_path = format!("{}:{}", shim_dir.display(), bin_dir.display());

    // Run multiple commands with same session ID
    for i in 1..=3 {
        let mut cmd = std::process::Command::new(&shim_binary);
        #[cfg(unix)]
        {
            use std::os::unix::process::CommandExt;
            cmd.arg0("test_cmd");
        }
        let output = cmd
            .arg(i.to_string())
            .env("SHIM_ORIGINAL_PATH", bin_dir.to_string_lossy().as_ref())
            .env("SHIM_TRACE_LOG", &log_file)
            .env("SHIM_SESSION_ID", &session_id)
            .env("PATH", &shimmed_path)
            .env("SHIM_DEPTH", (i - 1).to_string()) // Simulate nested execution
            .output()?;

        assert!(output.status.success());
    }

    // Verify all log entries have the same session ID
    let log_content = fs::read_to_string(&log_file)?;
    let lines: Vec<&str> = log_content.lines().collect();
    assert_eq!(lines.len(), 3, "Should have 3 log entries");

    let mut depths = Vec::new();

    for line in &lines {
        assert!(line.contains(&format!("\"session_id\":\"{session_id}\"")));
        assert!(line.contains("\"command\":\"test_cmd\""));

        if let Ok(value) = serde_json::from_str::<serde_json::Value>(line) {
            if let Some(depth) = value.get("depth").and_then(|d| d.as_i64()).or_else(|| {
                value
                    .get("depth")
                    .and_then(|d| d.as_u64().map(|v| v as i64))
            }) {
                depths.push(depth as i32);
            }
        }
    }

    depths.sort();
    assert_eq!(
        depths.len(),
        3,
        "Expected three depth entries, got {:?}",
        depths
    );
    let start = depths[0];
    assert!(
        start == 0 || start == 1,
        "Unexpected starting depth {}, expected 0 or 1",
        start
    );
    for (idx, depth) in depths.iter().enumerate() {
        assert_eq!(
            *depth,
            start + idx as i32,
            "Depth sequence should progress by 1 starting at {}",
            start
        );
    }

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

    let shimmed_path = format!("{}:{}", shim_dir.display(), bin_dir.display());

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
        .env("PATH", &shimmed_path)
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

    let shimmed_path = format!("{}:{}", shim_dir.display(), bin_dir.display());

    // Try to execute nonexistent command
    let output = std::process::Command::new(&shim_binary)
        .env("SHIM_ORIGINAL_PATH", bin_dir.to_string_lossy().as_ref())
        .env("PATH", &shimmed_path)
        .output()?;

    // Should fail with appropriate error code
    assert!(!output.status.success());

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Command 'nonexistent' not found"));

    Ok(())
}

/// Ensure runtime PATH mutations from managers like pyenv/nvm are honored
#[test]
fn test_runtime_path_overrides_original_var() -> Result<()> {
    let temp = TempDir::new()?;
    let shim_dir = temp.path().join("shims");
    let original_dir = temp.path().join("original");
    let override_dir = temp.path().join("override");
    let log_file = temp.path().join("runtime_path.jsonl");

    fs::create_dir_all(&shim_dir)?;
    fs::create_dir_all(&original_dir)?;
    fs::create_dir_all(&override_dir)?;

    let original_python = original_dir.join("python");
    let override_python = override_dir.join("python");

    fs::write(&original_python, "#!/bin/bash\necho original-python")?;
    fs::write(&override_python, "#!/bin/bash\necho override-python")?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        for script in [&original_python, &override_python] {
            let mut perms = fs::metadata(script)?.permissions();
            perms.set_mode(0o755);
            fs::set_permissions(script, perms)?;
        }
    }

    let shim_binary_path = get_shim_binary_path();
    let shim_python = shim_dir.join("python");
    fs::copy(shim_binary_path, &shim_python)?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&shim_python)?.permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&shim_python, perms)?;
    }

    let path_with_manager = format!(
        "{}:{}:{}",
        override_dir.display(),
        shim_dir.display(),
        original_dir.display()
    );

    let output = std::process::Command::new(&shim_python)
        .env("PATH", &path_with_manager)
        .env(
            "SHIM_ORIGINAL_PATH",
            original_dir.to_string_lossy().as_ref(),
        )
        .env("SHIM_TRACE_LOG", &log_file)
        .env("SHIM_CACHE_BUST", "1")
        .env_remove("SHIM_ACTIVE")
        .env_remove("SHIM_DEPTH")
        .output()?;

    assert!(
        output.status.success(),
        "shim execution failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let log_content = fs::read_to_string(&log_file)?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("override-python"),
        "expected override binary to run, got: {stdout}"
    );

    assert!(
        log_content.contains(&override_dir.display().to_string()),
        "log should record resolved override path: {log_content}"
    );

    Ok(())
}

/// Ensure bypass mode (nested shims) still respects runtime PATH changes
#[test]
fn test_bypass_mode_honors_runtime_path_changes() -> Result<()> {
    let temp = TempDir::new()?;
    let shim_dir = temp.path().join("shims");
    let original_dir = temp.path().join("original");
    let override_dir = temp.path().join("override");
    let log_file = temp.path().join("bypass_path.jsonl");

    fs::create_dir_all(&shim_dir)?;
    fs::create_dir_all(&original_dir)?;
    fs::create_dir_all(&override_dir)?;

    let original_node = original_dir.join("node");
    let override_node = override_dir.join("node");

    fs::write(&original_node, "#!/bin/bash\necho original-node")?;
    fs::write(&override_node, "#!/bin/bash\necho override-node")?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        for script in [&original_node, &override_node] {
            let mut perms = fs::metadata(script)?.permissions();
            perms.set_mode(0o755);
            fs::set_permissions(script, perms)?;
        }
    }

    let shim_binary_path = get_shim_binary_path();
    let shim_node = shim_dir.join("node");
    fs::copy(shim_binary_path, &shim_node)?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&shim_node)?.permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&shim_node, perms)?;
    }

    let path_with_manager = format!(
        "{}:{}:{}",
        override_dir.display(),
        shim_dir.display(),
        original_dir.display()
    );

    let output = std::process::Command::new(&shim_node)
        .env("PATH", &path_with_manager)
        .env(
            "SHIM_ORIGINAL_PATH",
            original_dir.to_string_lossy().as_ref(),
        )
        .env("SHIM_ACTIVE", "1") // simulate nested invocation
        .env("SHIM_TRACE_LOG", &log_file)
        .env("SHIM_CACHE_BUST", "1")
        .env_remove("SHIM_DEPTH")
        .output()?;

    assert!(
        output.status.success(),
        "bypass execution failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("override-node"),
        "expected override binary to run in bypass mode, got: {stdout}"
    );

    let log_content = fs::read_to_string(&log_file)?;
    assert!(
        log_content.contains(&override_dir.display().to_string()),
        "log should record resolved override path: {log_content}"
    );

    Ok(())
}
