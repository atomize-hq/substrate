#![cfg(unix)]
//! Integration tests for substrate shim
//!
//! These tests verify the complete shim execution flow, including the exact
//! scenarios that were proven to work with Claude Code through manual testing.

use anyhow::Result;
use nix::libc::ETXTBSY;
use nix::unistd::{geteuid, User};
use serde_json::Value;
use serial_test::serial;
use std::{fs, io, process::Command, thread, time::Duration};
use tempfile::TempDir;
use transport_api_types::{InstallBootstrapContextCarrierV1, InstallBootstrapContextV1};

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

fn run_with_retry(mut command: Command) -> Result<std::process::Output> {
    let mut last_err = None;

    for attempt in 0..3 {
        match command.output() {
            Ok(output) => return Ok(output),
            Err(err) if err.raw_os_error() == Some(ETXTBSY) => {
                last_err = Some(err);
                if attempt < 2 {
                    thread::sleep(Duration::from_millis(25));
                    continue;
                }
            }
            Err(err) => return Err(err.into()),
        }
    }

    Err(last_err
        .unwrap_or_else(|| io::Error::from_raw_os_error(ETXTBSY))
        .into())
}

fn initialize_test_git_repository(path: &std::path::Path, policy_yaml: &str) -> String {
    fs::create_dir_all(path).unwrap();
    let run = |args: &[&str]| {
        let output = Command::new("git")
            .arg("-C")
            .arg(path)
            .args(args)
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "git {:?} failed: {}",
            args,
            String::from_utf8_lossy(&output.stderr)
        );
        output
    };
    run(&["init", "--quiet"]);
    run(&["config", "user.name", "Substrate Shim Integration Test"]);
    run(&["config", "user.email", "shim-integration@substrate.invalid"]);
    fs::write(path.join("policy.yaml"), policy_yaml).unwrap();
    run(&["add", "policy.yaml"]);
    run(&["commit", "--quiet", "-m", "test policy"]);
    String::from_utf8(run(&["rev-parse", "HEAD"]).stdout)
        .unwrap()
        .trim()
        .to_string()
}

fn allow_policy_yaml(policy_id: &str, cmd: &str) -> String {
    format!("id: \"{policy_id}\"\nname: \"{policy_id}\"\ncmd_allowed:\n  - \"{cmd}\"\n")
}

fn deny_policy_yaml(policy_id: &str, cmd: &str) -> String {
    format!("id: \"{policy_id}\"\nname: \"{policy_id}\"\ncmd_denied:\n  - \"{cmd}\"\n")
}

fn workspace_deny_policy_patch(cmd: &str) -> String {
    format!("cmd_denied:\n  - \"{cmd}\"\n")
}

fn write_workspace_policy(root: &std::path::Path, policy_yaml: &str) {
    let substrate_dir = root.join(".substrate");
    fs::create_dir_all(&substrate_dir).unwrap();
    fs::write(substrate_dir.join("workspace.yaml"), "version: 1\n").unwrap();
    fs::write(substrate_dir.join("policy.yaml"), policy_yaml).unwrap();
}

fn current_unix_account() -> (String, u32) {
    let uid = geteuid();
    let user = User::from_uid(uid)
        .unwrap()
        .expect("current user should exist in passwd database");
    (user.name, uid.as_raw())
}

fn encoded_unix_host_carrier(prefix: &std::path::Path) -> InstallBootstrapContextCarrierV1 {
    let (account, uid) = current_unix_account();
    InstallBootstrapContextCarrierV1::from_context(
        InstallBootstrapContextV1::new_unix(prefix.to_str().unwrap(), &account, uid).unwrap(),
    )
    .unwrap()
}

/// Test the complete shim execution flow with real binary resolution
#[test]
#[serial]
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

    let mut cmd = Command::new(&shim_binary);
    cmd.args(["test", "message"])
        .env("SHIM_ORIGINAL_PATH", bin_dir.to_string_lossy().as_ref())
        .env("SHIM_TRACE_LOG", &log_file)
        .env("SHIM_SESSION_ID", &session_id)
        .env("PATH", &shimmed_path)
        .env_remove("SHIM_DEPTH") // Ensure deterministic test environment
        .env_remove("SHIM_ACTIVE");
    let output = run_with_retry(cmd)?;

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

/// Test the Claude Code hash pinning scenario through an actual pinned dispatch.
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

    // Test the exact hash-pinning sequence against a real shim dispatch.
    let shimmed_path = format!("{}:{}", shim_dir.display(), bin_dir.display());

    // Test 1: Basic PATH resolution - Test that shim is found first
    // We need bash and which, but want our shim to come first
    let full_path = format!("{shimmed_path}:/usr/bin:/bin");

    let output = Command::new("/bin/bash")
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

    // Test 2: Hash pinning - make ordinary PATH lookup prefer the real binary, with the shim only later in PATH.
    let hashed_path = format!("{}:{}:/usr/bin:/bin", bin_dir.display(), shim_dir.display());
    let hash_log = temp.path().join("hash-trace.jsonl");
    let trace_path = temp.path().join("trace.jsonl");
    let hash_command = format!(
        "set -e; hash -r; hash -p \"{}\" testcmd; hash -t testcmd >/dev/null; testcmd pinned-arg",
        shim_binary.display()
    );

    let output = Command::new("/bin/bash")
        .args(["-c", &hash_command])
        .env("PATH", &hashed_path)
        .env("SHIM_ORIGINAL_PATH", bin_dir.to_string_lossy().as_ref())
        .env("SHIM_TRACE_LOG", &hash_log)
        .output()?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        output.status.success(),
        "Hash pinning dispatch failed. stdout: {stdout}\nstderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        stdout.contains("testcmd: pinned-arg"),
        "Hash-pinned shim did not dispatch the expected command. Output: {stdout}"
    );
    let log_content = fs::read_to_string(&trace_path)?;
    assert!(
        log_content.contains("\"command\":\"testcmd\""),
        "Hash-pinned shim did not emit the expected trace log. Log: {log_content}"
    );
    assert!(!hash_log.exists());

    Ok(())
}

/// Test SHIM_BYPASS functionality
#[test]
#[serial]
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
    let mut cmd = Command::new(&shim_echo);
    cmd.env("SHIM_BYPASS", "1")
        .env("SHIM_ORIGINAL_PATH", bin_dir.to_string_lossy().as_ref())
        .env("PATH", &shimmed_path)
        .arg("bypass works");
    let output = run_with_retry(cmd)?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("bypass works"));
    assert_eq!(output.status.code().unwrap_or(1), 0);

    Ok(())
}

#[test]
#[serial]
fn test_physical_shim_binds_trace_and_policy_to_selected_prefix_under_conflicting_ambient_roots(
) -> Result<()> {
    let temp = TempDir::new()?;
    let prefix_a = temp.path().join("selected-a");
    let prefix_b = temp.path().join("ambient-b");
    let shim_dir = prefix_a.join("shims");
    let bin_dir = temp.path().join("bin");
    fs::create_dir_all(&shim_dir)?;
    fs::create_dir_all(&bin_dir)?;
    let commit_a = initialize_test_git_repository(
        &prefix_a,
        &allow_policy_yaml("selected-a", "echo policy bound"),
    );
    let commit_b = initialize_test_git_repository(
        &prefix_b,
        &deny_policy_yaml("ambient-b", "echo policy bound"),
    );
    write_workspace_policy(
        &prefix_b,
        &deny_policy_yaml("ambient-workspace", "echo policy bound"),
    );

    let test_script = bin_dir.join("echo");
    fs::write(&test_script, "#!/bin/bash\necho \"trace-bound: $*\"")?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&test_script)?.permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&test_script, perms)?;
    }

    let shim_binary = shim_dir.join("echo");
    fs::copy(get_shim_binary_path(), &shim_binary)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&shim_binary)?.permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&shim_binary, perms)?;
    }

    let mut cmd = Command::new(&shim_binary);
    cmd.args(["policy", "bound"])
        .env("SHIM_ORIGINAL_PATH", bin_dir.to_string_lossy().as_ref())
        .env("SUBSTRATE_POLICY_MODE", "enforce")
        .env("SHIM_TRACE_LOG", prefix_b.join("trace.jsonl"))
        .env("HOME", &prefix_b)
        .env("USERPROFILE", &prefix_b)
        .env("SUBSTRATE_HOME", &prefix_b)
        .env("SUBSTRATE_ROOT", &prefix_b)
        .env(
            "PATH",
            format!("{}:{}", shim_dir.display(), bin_dir.display()),
        )
        .current_dir(&prefix_b)
        .env_remove("SHIM_DEPTH")
        .env_remove("SHIM_ACTIVE");
    let output = run_with_retry(cmd)?;

    assert!(output.status.success());
    assert_eq!(
        String::from_utf8_lossy(&output.stdout).trim(),
        "trace-bound: policy bound"
    );

    let trace_path = prefix_a.join("trace.jsonl");
    assert!(trace_path.is_file());
    assert!(!prefix_b.join("trace.jsonl").exists());

    let entries = fs::read_to_string(&trace_path)?
        .lines()
        .map(serde_json::from_str::<Value>)
        .collect::<std::result::Result<Vec<_>, _>>()?;
    let span = entries
        .iter()
        .find(|value| value["event_type"] == "command_complete")
        .expect("command_complete span");
    assert_eq!(span["policy_id"].as_str(), Some("selected-a"));
    assert_eq!(
        span["replay_context"]["policy_id"].as_str(),
        Some("selected-a")
    );
    assert_eq!(
        span["replay_context"]["policy_commit"].as_str(),
        Some(commit_a.as_str())
    );
    assert_ne!(
        span["replay_context"]["policy_commit"].as_str(),
        Some(commit_b.as_str())
    );
    assert!(
        entries
            .iter()
            .any(|value| value["command"] == "echo" && value["resolved_path"].is_string()),
        "expected execution log entry in bound trace output"
    );

    Ok(())
}

#[test]
#[serial]
fn test_physical_shim_merges_bound_global_and_workspace_policy_layers() -> Result<()> {
    let temp = TempDir::new()?;
    let prefix_a = temp.path().join("selected-a");
    let prefix_b = temp.path().join("ambient-b");
    let shim_dir = prefix_a.join("shims");
    let bin_dir = temp.path().join("bin");
    fs::create_dir_all(&shim_dir)?;
    fs::create_dir_all(&bin_dir)?;
    fs::create_dir_all(&prefix_b)?;
    let commit_a =
        initialize_test_git_repository(&prefix_a, &allow_policy_yaml("selected-a", "echo merged"));
    write_workspace_policy(&prefix_a, &workspace_deny_policy_patch("echo merged"));

    let test_script = bin_dir.join("echo");
    fs::write(&test_script, "#!/bin/bash\necho \"merged: $*\"")?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&test_script)?.permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&test_script, perms)?;
    }

    let shim_binary = shim_dir.join("echo");
    fs::copy(get_shim_binary_path(), &shim_binary)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&shim_binary)?.permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&shim_binary, perms)?;
    }

    let mut cmd = Command::new(&shim_binary);
    cmd.arg("merged")
        .env("SUBSTRATE_POLICY_MODE", "enforce")
        .env("SHIM_ORIGINAL_PATH", bin_dir.to_string_lossy().as_ref())
        .env("SHIM_TRACE_LOG", prefix_b.join("trace.jsonl"))
        .env("HOME", &prefix_b)
        .env("USERPROFILE", &prefix_b)
        .env("SUBSTRATE_HOME", &prefix_b)
        .env("SUBSTRATE_ROOT", &prefix_b)
        .env(
            "PATH",
            format!("{}:{}", shim_dir.display(), bin_dir.display()),
        )
        .current_dir(&prefix_a)
        .env_remove("SHIM_DEPTH")
        .env_remove("SHIM_ACTIVE");
    let output = run_with_retry(cmd)?;

    assert_eq!(output.status.code(), Some(126));
    assert!(String::from_utf8_lossy(&output.stderr).contains("command denied by policy"));

    let trace_path = prefix_a.join("trace.jsonl");
    assert!(trace_path.is_file());
    assert!(!prefix_b.join("trace.jsonl").exists());

    let entries = fs::read_to_string(&trace_path)?
        .lines()
        .map(serde_json::from_str::<Value>)
        .collect::<std::result::Result<Vec<_>, _>>()?;
    let span = entries
        .iter()
        .find(|value| value["event_type"] == "command_complete")
        .expect("command_complete span");
    assert_eq!(span["policy_id"].as_str(), Some("selected-a"));
    assert_eq!(
        span["replay_context"]["policy_id"].as_str(),
        Some("selected-a")
    );
    assert_eq!(
        span["replay_context"]["policy_commit"].as_str(),
        Some(commit_a.as_str())
    );
    assert_eq!(span["policy_decision"]["action"].as_str(), Some("deny"));

    Ok(())
}

#[test]
#[serial]
fn test_physical_shim_honors_descendant_workspace_policy_within_selected_prefix() -> Result<()> {
    let temp = TempDir::new()?;
    let prefix_a = temp.path().join("selected-a");
    let prefix_b = temp.path().join("ambient-b");
    let child_dir = prefix_a.join("nested").join("child");
    let shim_dir = prefix_a.join("shims");
    let bin_dir = temp.path().join("bin");
    fs::create_dir_all(&shim_dir)?;
    fs::create_dir_all(&bin_dir)?;
    fs::create_dir_all(&prefix_b)?;
    fs::create_dir_all(&child_dir)?;
    let commit_a = initialize_test_git_repository(
        &prefix_a,
        &allow_policy_yaml("selected-a", "echo descendant bound"),
    );
    write_workspace_policy(
        &child_dir,
        &workspace_deny_policy_patch("echo descendant bound"),
    );

    let test_script = bin_dir.join("echo");
    fs::write(&test_script, "#!/bin/bash\necho \"descendant: $*\"")?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&test_script)?.permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&test_script, perms)?;
    }

    let shim_binary = shim_dir.join("echo");
    fs::copy(get_shim_binary_path(), &shim_binary)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&shim_binary)?.permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&shim_binary, perms)?;
    }

    let mut cmd = Command::new(&shim_binary);
    cmd.arg("descendant")
        .env("SUBSTRATE_POLICY_MODE", "enforce")
        .env("SHIM_ORIGINAL_PATH", bin_dir.to_string_lossy().as_ref())
        .env("SHIM_TRACE_LOG", prefix_b.join("trace.jsonl"))
        .env("HOME", &prefix_b)
        .env("USERPROFILE", &prefix_b)
        .env("SUBSTRATE_HOME", &prefix_b)
        .env("SUBSTRATE_ROOT", &prefix_b)
        .env(
            "PATH",
            format!("{}:{}", shim_dir.display(), bin_dir.display()),
        )
        .current_dir(&child_dir)
        .env_remove("SHIM_DEPTH")
        .env_remove("SHIM_ACTIVE");
    let output = run_with_retry(cmd)?;

    assert_eq!(output.status.code(), Some(126));
    assert!(String::from_utf8_lossy(&output.stderr).contains("command denied by policy"));

    let trace_path = prefix_a.join("trace.jsonl");
    assert!(trace_path.is_file());
    assert!(!prefix_b.join("trace.jsonl").exists());

    let entries = fs::read_to_string(&trace_path)?
        .lines()
        .map(serde_json::from_str::<Value>)
        .collect::<std::result::Result<Vec<_>, _>>()?;
    let span = entries
        .iter()
        .find(|value| value["event_type"] == "command_complete")
        .expect("command_complete span");
    assert_eq!(span["policy_id"].as_str(), Some("selected-a"));
    assert_eq!(
        span["replay_context"]["policy_id"].as_str(),
        Some("selected-a")
    );
    assert_eq!(
        span["replay_context"]["policy_commit"].as_str(),
        Some(commit_a.as_str())
    );
    assert_eq!(span["policy_decision"]["action"].as_str(), Some("deny"));

    Ok(())
}

#[test]
#[serial]
fn test_physical_shim_ignores_ancestor_workspace_policy_outside_selected_prefix() -> Result<()> {
    let temp = TempDir::new()?;
    let ambient_root = temp.path().join("ambient-root");
    let prefix_a = ambient_root.join("selected-a");
    let prefix_b = temp.path().join("ambient-home");
    let shim_dir = prefix_a.join("shims");
    let bin_dir = temp.path().join("bin");
    fs::create_dir_all(&shim_dir)?;
    fs::create_dir_all(&bin_dir)?;
    fs::create_dir_all(&prefix_b)?;
    let commit_a = initialize_test_git_repository(
        &prefix_a,
        &allow_policy_yaml("selected-a", "echo ancestor bound"),
    );
    write_workspace_policy(
        &ambient_root,
        &deny_policy_yaml("ambient-ancestor", "echo ancestor bound"),
    );

    let test_script = bin_dir.join("echo");
    fs::write(&test_script, "#!/bin/bash\necho \"ancestor-bound: $*\"")?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&test_script)?.permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&test_script, perms)?;
    }

    let shim_binary = shim_dir.join("echo");
    fs::copy(get_shim_binary_path(), &shim_binary)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&shim_binary)?.permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&shim_binary, perms)?;
    }

    let mut cmd = Command::new(&shim_binary);
    cmd.args(["ancestor", "bound"])
        .env("SUBSTRATE_POLICY_MODE", "enforce")
        .env("SHIM_ORIGINAL_PATH", bin_dir.to_string_lossy().as_ref())
        .env("SHIM_TRACE_LOG", prefix_b.join("trace.jsonl"))
        .env("HOME", &prefix_b)
        .env("USERPROFILE", &prefix_b)
        .env("SUBSTRATE_HOME", &prefix_b)
        .env("SUBSTRATE_ROOT", &prefix_b)
        .env(
            "PATH",
            format!("{}:{}", shim_dir.display(), bin_dir.display()),
        )
        .current_dir(&prefix_a)
        .env_remove("SHIM_DEPTH")
        .env_remove("SHIM_ACTIVE");
    let output = run_with_retry(cmd)?;

    assert!(output.status.success());
    assert_eq!(
        String::from_utf8_lossy(&output.stdout).trim(),
        "ancestor-bound: ancestor bound"
    );

    let trace_path = prefix_a.join("trace.jsonl");
    assert!(trace_path.is_file());
    assert!(!prefix_b.join("trace.jsonl").exists());

    let entries = fs::read_to_string(&trace_path)?
        .lines()
        .map(serde_json::from_str::<Value>)
        .collect::<std::result::Result<Vec<_>, _>>()?;
    let span = entries
        .iter()
        .find(|value| value["event_type"] == "command_complete")
        .expect("command_complete span");
    assert_eq!(span["policy_id"].as_str(), Some("selected-a"));
    assert_eq!(
        span["replay_context"]["policy_id"].as_str(),
        Some("selected-a")
    );
    assert_eq!(
        span["replay_context"]["policy_commit"].as_str(),
        Some(commit_a.as_str())
    );

    Ok(())
}

#[test]
#[serial]
fn test_physical_shim_bypass_rejects_conflicting_inherited_bootstrap_context() -> Result<()> {
    let temp = TempDir::new()?;
    let prefix_a = temp.path().join("selected-a");
    let prefix_b = temp.path().join("ambient-b");
    let shim_dir = prefix_a.join("shims");
    let bin_dir = temp.path().join("bin");
    fs::create_dir_all(&shim_dir)?;
    fs::create_dir_all(&bin_dir)?;
    fs::create_dir_all(&prefix_a)?;
    fs::create_dir_all(&prefix_b)?;

    let test_script = bin_dir.join("echo");
    fs::write(&test_script, "#!/bin/bash\necho \"$@\"")?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&test_script)?.permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&test_script, perms)?;
    }

    let shim_binary = shim_dir.join("echo");
    fs::copy(get_shim_binary_path(), &shim_binary)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&shim_binary)?.permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&shim_binary, perms)?;
    }

    let conflicting = encoded_unix_host_carrier(&prefix_b);
    let mut cmd = Command::new(&shim_binary);
    cmd.arg("blocked")
        .env("SHIM_BYPASS", "1")
        .env("SHIM_ORIGINAL_PATH", bin_dir.to_string_lossy().as_ref())
        .env("SHIM_TRACE_LOG", prefix_b.join("trace.jsonl"))
        .env("HOME", &prefix_b)
        .env("USERPROFILE", &prefix_b)
        .env("SUBSTRATE_HOME", &prefix_b)
        .env("SUBSTRATE_ROOT", &prefix_b)
        .env(
            "SUBSTRATE_INSTALL_BOOTSTRAP_CONTEXT_V1",
            conflicting.encode().unwrap(),
        )
        .env(
            "SUBSTRATE_INSTALL_HOST_CONTEXT_COMMITMENT",
            &conflicting.host_context_commitment,
        )
        .env(
            "SUBSTRATE_INSTALL_PRIMARY_USER",
            match &conflicting.context.intended_host_principal {
                transport_api_types::PlatformPrincipalV1::Unix { account, .. } => account,
                transport_api_types::PlatformPrincipalV1::Windows { .. } => unreachable!(),
            },
        )
        .env(
            "SUBSTRATE_INSTALL_PRIMARY_UID",
            match &conflicting.context.intended_host_principal {
                transport_api_types::PlatformPrincipalV1::Unix { uid, .. } => uid.to_string(),
                transport_api_types::PlatformPrincipalV1::Windows { .. } => unreachable!(),
            },
        )
        .current_dir(&prefix_b)
        .env_remove("SHIM_DEPTH")
        .env_remove("SHIM_ACTIVE");
    let output = run_with_retry(cmd)?;

    assert_eq!(output.status.code(), Some(126));
    assert!(
        output.stdout.is_empty(),
        "conflicting inherited bootstrap context should fail before dispatch"
    );
    assert!(String::from_utf8_lossy(&output.stderr)
        .contains("install bootstrap environment projection is missing or conflicting"));
    assert!(!prefix_a.join("trace.jsonl").exists());
    assert!(!prefix_b.join("trace.jsonl").exists());

    Ok(())
}

#[test]
#[serial]
fn test_physical_shim_fails_closed_on_malformed_bound_policy() -> Result<()> {
    let temp = TempDir::new()?;
    let prefix_a = temp.path().join("selected-a");
    let prefix_b = temp.path().join("ambient-b");
    let shim_dir = prefix_a.join("shims");
    let bin_dir = temp.path().join("bin");
    fs::create_dir_all(&shim_dir)?;
    fs::create_dir_all(&bin_dir)?;
    fs::create_dir_all(&prefix_b)?;
    fs::write(prefix_a.join("policy.yaml"), ":\n  - invalid").unwrap();

    let test_script = bin_dir.join("echo");
    fs::write(&test_script, "#!/bin/bash\necho \"malformed policy\"")?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&test_script)?.permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&test_script, perms)?;
    }

    let shim_binary = shim_dir.join("echo");
    fs::copy(get_shim_binary_path(), &shim_binary)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&shim_binary)?.permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&shim_binary, perms)?;
    }

    let mut cmd = Command::new(&shim_binary);
    cmd.arg("blocked")
        .env("SUBSTRATE_POLICY_MODE", "enforce")
        .env("SHIM_ORIGINAL_PATH", bin_dir.to_string_lossy().as_ref())
        .env("SHIM_TRACE_LOG", prefix_b.join("trace.jsonl"))
        .env("HOME", &prefix_b)
        .env("USERPROFILE", &prefix_b)
        .env("SUBSTRATE_HOME", &prefix_b)
        .env("SUBSTRATE_ROOT", &prefix_b)
        .env(
            "PATH",
            format!("{}:{}", shim_dir.display(), bin_dir.display()),
        )
        .current_dir(&prefix_b)
        .env_remove("SHIM_DEPTH")
        .env_remove("SHIM_ACTIVE");
    let output = run_with_retry(cmd)?;

    assert_eq!(output.status.code(), Some(126));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("failed to resolve bound effective policy")
            || stderr.contains("failed to load bound effective policy"),
        "unexpected stderr: {stderr}"
    );
    assert!(!prefix_a.join("trace.jsonl").exists());
    assert!(!prefix_b.join("trace.jsonl").exists());

    Ok(())
}

#[test]
#[serial]
fn test_physical_shim_rejects_conflicting_inherited_bootstrap_context_without_ambient_fallback(
) -> Result<()> {
    let temp = TempDir::new()?;
    let prefix_a = temp.path().join("selected-a");
    let prefix_b = temp.path().join("ambient-b");
    let shim_dir = prefix_a.join("shims");
    let bin_dir = temp.path().join("bin");
    fs::create_dir_all(&shim_dir)?;
    fs::create_dir_all(&bin_dir)?;
    fs::create_dir_all(&prefix_a)?;
    fs::create_dir_all(&prefix_b)?;

    let test_script = bin_dir.join("echo");
    fs::write(&test_script, "#!/bin/bash\necho \"$@\"")?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&test_script)?.permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&test_script, perms)?;
    }

    let shim_binary = shim_dir.join("echo");
    fs::copy(get_shim_binary_path(), &shim_binary)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&shim_binary)?.permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&shim_binary, perms)?;
    }

    let conflicting = encoded_unix_host_carrier(&prefix_b);
    let mut cmd = Command::new(&shim_binary);
    cmd.arg("blocked")
        .env("SHIM_ORIGINAL_PATH", bin_dir.to_string_lossy().as_ref())
        .env("SHIM_TRACE_LOG", prefix_b.join("trace.jsonl"))
        .env("HOME", &prefix_b)
        .env("USERPROFILE", &prefix_b)
        .env("SUBSTRATE_HOME", &prefix_b)
        .env("SUBSTRATE_ROOT", &prefix_b)
        .env(
            "SUBSTRATE_INSTALL_BOOTSTRAP_CONTEXT_V1",
            conflicting.encode().unwrap(),
        )
        .env(
            "SUBSTRATE_INSTALL_HOST_CONTEXT_COMMITMENT",
            &conflicting.host_context_commitment,
        )
        .env(
            "SUBSTRATE_INSTALL_PRIMARY_USER",
            match &conflicting.context.intended_host_principal {
                transport_api_types::PlatformPrincipalV1::Unix { account, .. } => account,
                transport_api_types::PlatformPrincipalV1::Windows { .. } => unreachable!(),
            },
        )
        .env(
            "SUBSTRATE_INSTALL_PRIMARY_UID",
            match &conflicting.context.intended_host_principal {
                transport_api_types::PlatformPrincipalV1::Unix { uid, .. } => uid.to_string(),
                transport_api_types::PlatformPrincipalV1::Windows { .. } => unreachable!(),
            },
        )
        .current_dir(&prefix_b)
        .env_remove("SHIM_DEPTH")
        .env_remove("SHIM_ACTIVE");
    let output = run_with_retry(cmd)?;

    assert_eq!(output.status.code(), Some(126));
    assert!(
        output.stdout.is_empty(),
        "conflicting inherited bootstrap context should fail before dispatch"
    );
    assert!(String::from_utf8_lossy(&output.stderr)
        .contains("install bootstrap environment projection is missing or conflicting"));
    assert!(!prefix_a.join("trace.jsonl").exists());
    assert!(!prefix_b.join("trace.jsonl").exists());

    Ok(())
}

#[test]
#[serial]
fn test_bypass_mode_does_not_fallback_to_ambient_trace_when_bound_bypass_log_target_is_unwritable(
) -> Result<()> {
    let temp = TempDir::new()?;
    let prefix_a = temp.path().join("selected-a");
    let prefix_b = temp.path().join("ambient-b");
    let shim_dir = prefix_a.join("shims");
    let bin_dir = temp.path().join("bin");
    fs::create_dir_all(&shim_dir)?;
    fs::create_dir_all(&bin_dir)?;
    fs::create_dir_all(&prefix_b)?;

    let test_script = bin_dir.join("echo");
    fs::write(&test_script, "#!/bin/bash\necho \"bypass bound\"")?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&test_script)?.permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&test_script, perms)?;
    }

    let shim_binary = shim_dir.join("echo");
    fs::copy(get_shim_binary_path(), &shim_binary)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&shim_binary)?.permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&shim_binary, perms)?;
        let mut root_perms = fs::metadata(&prefix_a)?.permissions();
        root_perms.set_mode(0o555);
        fs::set_permissions(&prefix_a, root_perms)?;
    }

    let mut cmd = Command::new(&shim_binary);
    cmd.arg("bound")
        .env("SHIM_BYPASS", "1")
        .env("SHIM_ORIGINAL_PATH", bin_dir.to_string_lossy().as_ref())
        .env("SHIM_TRACE_LOG", prefix_b.join("trace.jsonl"))
        .env("HOME", &prefix_b)
        .env("USERPROFILE", &prefix_b)
        .env("SUBSTRATE_HOME", &prefix_b)
        .env("SUBSTRATE_ROOT", &prefix_b)
        .env(
            "PATH",
            format!("{}:{}", shim_dir.display(), bin_dir.display()),
        )
        .current_dir(&prefix_b)
        .env_remove("SHIM_DEPTH")
        .env_remove("SHIM_ACTIVE");
    let output = run_with_retry(cmd)?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut root_perms = fs::metadata(&prefix_a)?.permissions();
        root_perms.set_mode(0o755);
        fs::set_permissions(&prefix_a, root_perms)?;
    }

    assert!(output.status.success());
    assert_eq!(
        String::from_utf8_lossy(&output.stdout).trim(),
        "bypass bound"
    );
    assert!(!prefix_a.join("trace.jsonl").exists());
    assert!(!prefix_b.join("trace.jsonl").exists());

    Ok(())
}

/// Test session correlation across multiple command invocations
#[test]
fn test_session_correlation() -> Result<()> {
    let temp = TempDir::new()?;
    let bin_dir = temp.path().join("bin");
    let shim_dir = temp.path().join("shims");
    let log_file = temp.path().join("session_test.jsonl");
    let trace_path = temp.path().join("trace.jsonl");

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
        let mut cmd = Command::new(&shim_binary);
        #[cfg(unix)]
        {
            use std::os::unix::process::CommandExt;
            cmd.arg0("test_cmd");
        }
        let output = {
            cmd.arg(i.to_string())
                .env("SHIM_ORIGINAL_PATH", bin_dir.to_string_lossy().as_ref())
                .env("SHIM_TRACE_LOG", &log_file)
                .env("SHIM_SESSION_ID", &session_id)
                .env("PATH", &shimmed_path)
                .env("SHIM_DEPTH", (i - 1).to_string()); // Simulate nested execution
            run_with_retry(cmd)?
        };

        assert!(output.status.success());
    }

    // Verify all log entries have the same session ID
    let log_content = fs::read_to_string(&trace_path)?;
    let lines: Vec<&str> = log_content.lines().collect();
    let cmd_lines: Vec<&str> = lines
        .iter()
        .copied()
        .filter(|line| line.contains("\"command\":\"test_cmd\""))
        .collect();
    assert_eq!(cmd_lines.len(), 3, "Should have 3 test_cmd log entries");

    let mut depths = Vec::new();

    for line in &cmd_lines {
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
#[serial]
fn test_credential_redaction() -> Result<()> {
    if std::env::var("SHIM_LOG_OPTS").as_deref() == Ok("raw") {
        eprintln!("skipping credential redaction assertions: SHIM_LOG_OPTS=raw disables redaction");
        return Ok(());
    }

    let temp = TempDir::new()?;
    let bin_dir = temp.path().join("bin");
    let shim_dir = temp.path().join("shims");
    let log_file = temp.path().join("redaction_test.jsonl");
    let trace_path = temp.path().join("trace.jsonl");

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
    let mut cmd = Command::new(&shim_binary);
    cmd.args([
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
    .env("PATH", &shimmed_path);
    cmd.env_remove("SHIM_LOG_OPTS");
    let output = run_with_retry(cmd)?;

    assert!(output.status.success());

    // Verify credentials were redacted in log payloads that include argv
    let log_content = fs::read_to_string(&trace_path)?;
    let mut redacted_entry = None;
    for line in log_content.lines() {
        if let Ok(value) = serde_json::from_str::<serde_json::Value>(line) {
            if value.get("argv").is_some() {
                redacted_entry = Some(value);
                break;
            }
        }
    }
    let Some(entry) = redacted_entry else {
        panic!(
            "expected argv-bearing log entry in {}",
            trace_path.display()
        );
    };
    let argv = entry
        .get("argv")
        .and_then(|v| v.as_array())
        .expect("argv should be array");
    let argv_strs: Vec<String> = argv
        .iter()
        .filter_map(|v| v.as_str().map(|s| s.to_string()))
        .collect();

    // Should contain redacted versions
    assert!(argv_strs.iter().any(|a| a.contains("Authorization: ***")));
    assert!(argv_strs.iter().any(|a| a.contains("X-API-Key: ***")));
    assert!(argv_strs.iter().any(|a| a == "***")); // For --token flag

    // Should NOT contain actual secrets
    for secret in ["secret123", "mykey456", "supersecret"] {
        assert!(
            argv_strs.iter().all(|a| !a.contains(secret)),
            "argv leaked secret {secret}: {argv_strs:?}"
        );
    }

    // Should contain non-sensitive arguments
    assert!(argv_strs
        .iter()
        .any(|a| a.contains("https://api.example.com")));

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
    let mut cmd = Command::new(&shim_binary);
    cmd.env("SHIM_ORIGINAL_PATH", bin_dir.to_string_lossy().as_ref())
        .env("PATH", &shimmed_path);
    let output = run_with_retry(cmd)?;

    // Should fail with appropriate error code
    assert!(!output.status.success());

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Command 'nonexistent' not found"));

    Ok(())
}

/// Ensure runtime PATH mutations from managers like pyenv/nvm are honored
#[test]
#[serial]
fn test_runtime_path_overrides_original_var() -> Result<()> {
    let temp = TempDir::new()?;
    let shim_dir = temp.path().join("shims");
    let original_dir = temp.path().join("original");
    let override_dir = temp.path().join("override");
    let log_file = temp.path().join("runtime_path.jsonl");
    let trace_path = temp.path().join("trace.jsonl");

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

    let mut cmd = Command::new(&shim_python);
    cmd.env("PATH", &path_with_manager)
        .env(
            "SHIM_ORIGINAL_PATH",
            original_dir.to_string_lossy().as_ref(),
        )
        .env("SHIM_TRACE_LOG", &log_file)
        .env("SHIM_CACHE_BUST", "1")
        .env_remove("SHIM_ACTIVE")
        .env_remove("SHIM_DEPTH");
    let output = run_with_retry(cmd)?;

    assert!(
        output.status.success(),
        "shim execution failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let log_content = fs::read_to_string(&trace_path)?;

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

#[test]
#[serial]
fn manager_hint_logging_records_entry() -> Result<()> {
    let temp = TempDir::new()?;
    let shim_dir = temp.path().join("shims");
    let bin_dir = temp.path().join("bin");
    let log_file = temp.path().join("hint_log.jsonl");
    let trace_path = temp.path().join("trace.jsonl");
    let manifest_path = temp.path().join("manager_hooks.yaml");

    fs::create_dir_all(&shim_dir)?;
    fs::create_dir_all(&bin_dir)?;

    fs::write(
        &manifest_path,
        r#"version: 2
managers:
  - name: nvm
    priority: 10
    detect: {}
    init: {}
    errors:
      - "nvm: command not found"
    repair_hint: "initialize nvm inside Substrate"
"#,
    )?;

    let failing = bin_dir.join("nvm");
    fs::write(
        &failing,
        "#!/usr/bin/env bash\necho 'shim-outer'\necho 'nvm: command not found' >&2\nexit 127\n",
    )?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&failing)?.permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&failing, perms)?;
    }

    let shim_binary_path = get_shim_binary_path();
    let shim_copy = shim_dir.join("nvm");
    fs::copy(shim_binary_path, &shim_copy)?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&shim_copy)?.permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&shim_copy, perms)?;
    }

    let host_path = std::env::var("PATH").unwrap_or_default();
    let shimmed_path = if host_path.is_empty() {
        format!("{}:{}:/usr/bin:/bin", shim_dir.display(), bin_dir.display())
    } else {
        format!(
            "{}:{}:{}:/usr/bin:/bin",
            shim_dir.display(),
            bin_dir.display(),
            host_path
        )
    };

    let mut cmd = Command::new(&shim_copy);
    cmd.env("SHIM_ORIGINAL_PATH", bin_dir.to_string_lossy().as_ref())
        .env("PATH", &shimmed_path)
        .env("SHIM_TRACE_LOG", &log_file)
        .env(
            "SUBSTRATE_MANAGER_MANIFEST",
            temp.path()
                .join("ambient-b")
                .join("manager_hooks.yaml")
                .to_string_lossy()
                .as_ref(),
        )
        .env("SUBSTRATE_WORLD", "enabled")
        .env("SUBSTRATE_SHIM_HINTS", "1")
        .env_remove("SHIM_ACTIVE")
        .env_remove("SHIM_DEPTH");
    let output = run_with_retry(cmd)?;

    assert!(!output.status.success(), "shim should propagate failure");

    let log_content = fs::read_to_string(&trace_path)?;
    let mut hint_found = false;
    for line in log_content.lines() {
        if let Ok(Value::Object(obj)) = serde_json::from_str::<Value>(line) {
            if let Some(hint) = obj.get("manager_hint") {
                hint_found = true;
                assert_eq!(hint.get("name").and_then(|v| v.as_str()), Some("nvm"));
                assert!(hint.get("hint").is_some());
                break;
            }
        }
    }

    assert!(hint_found, "manager_hint entry missing in log");
    Ok(())
}

#[test]
#[serial]
fn tier2_manager_hint_logging_records_entry() -> Result<()> {
    let temp = TempDir::new()?;
    let shim_dir = temp.path().join("shims");
    let bin_dir = temp.path().join("bin");
    let log_file = temp.path().join("bun_hint_log.jsonl");
    let trace_path = temp.path().join("trace.jsonl");
    let manifest_path = temp.path().join("manager_hooks.yaml");

    fs::create_dir_all(&shim_dir)?;
    fs::create_dir_all(&bin_dir)?;

    fs::write(
        &manifest_path,
        r#"version: 2
managers:
  - name: bun
    priority: 15
    detect: {}
    init: {}
    errors:
      - "bun: command not found"
    repair_hint: "install bun inside Substrate"
"#,
    )?;

    let failing = bin_dir.join("bun");
    fs::write(
        &failing,
        "#!/usr/bin/env bash\necho 'bun: command not found' >&2\nexit 127\n",
    )?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&failing)?.permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&failing, perms)?;
    }

    let shim_binary_path = get_shim_binary_path();
    let shim_copy = shim_dir.join("bun");
    fs::copy(shim_binary_path, &shim_copy)?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&shim_copy)?.permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&shim_copy, perms)?;
    }

    let host_path = std::env::var("PATH").unwrap_or_default();
    let shimmed_path = if host_path.is_empty() {
        format!("{}:{}:/usr/bin:/bin", shim_dir.display(), bin_dir.display())
    } else {
        format!(
            "{}:{}:{}:/usr/bin:/bin",
            shim_dir.display(),
            bin_dir.display(),
            host_path
        )
    };

    let mut cmd = Command::new(&shim_copy);
    cmd.env("SHIM_ORIGINAL_PATH", bin_dir.to_string_lossy().as_ref())
        .env("PATH", &shimmed_path)
        .env("SHIM_TRACE_LOG", &log_file)
        .env(
            "SUBSTRATE_MANAGER_MANIFEST",
            temp.path()
                .join("ambient-b")
                .join("manager_hooks.yaml")
                .to_string_lossy()
                .as_ref(),
        )
        .env("SUBSTRATE_WORLD", "enabled")
        .env("SUBSTRATE_SHIM_HINTS", "1")
        .env_remove("SHIM_ACTIVE")
        .env_remove("SHIM_DEPTH");
    let output = run_with_retry(cmd)?;

    assert!(
        !output.status.success(),
        "bun shim should propagate failure so hints emit"
    );

    let log_content = fs::read_to_string(&trace_path)?;
    let mut bun_hint = None;
    for line in log_content.lines() {
        if let Ok(Value::Object(obj)) = serde_json::from_str::<Value>(line) {
            if let Some(hint) = obj.get("manager_hint") {
                bun_hint = Some(hint.clone());
                break;
            }
        }
    }

    let hint = bun_hint.expect("bun manager hint missing");
    assert_eq!(hint.get("name").and_then(|v| v.as_str()), Some("bun"));
    assert_eq!(
        hint.get("hint").and_then(|v| v.as_str()),
        Some("install bun inside Substrate")
    );
    Ok(())
}

#[test]
fn manager_hint_skipped_when_world_disabled() -> Result<()> {
    let temp = TempDir::new()?;
    let shim_dir = temp.path().join("shims");
    let bin_dir = temp.path().join("bin");
    let log_file = temp.path().join("hint_disabled.jsonl");
    let trace_path = temp.path().join("trace.jsonl");
    let manifest_path = temp.path().join("manager_hooks.yaml");

    fs::create_dir_all(&shim_dir)?;
    fs::create_dir_all(&bin_dir)?;

    fs::write(
        &manifest_path,
        r#"version: 2
managers:
  - name: direnv
    priority: 5
    detect: {}
    init: {}
    errors:
      - "direnv: command not found"
    repair_hint: "install direnv"
"#,
    )?;

    let failing = bin_dir.join("direnv");
    fs::write(
        &failing,
        "#!/usr/bin/env bash\necho 'direnv: command not found' >&2\nexit 127\n",
    )?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&failing)?.permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&failing, perms)?;
    }

    let shim_binary_path = get_shim_binary_path();
    let shim_copy = shim_dir.join("direnv");
    fs::copy(shim_binary_path, &shim_copy)?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&shim_copy)?.permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&shim_copy, perms)?;
    }

    let host_path = std::env::var("PATH").unwrap_or_default();
    let shimmed_path = if host_path.is_empty() {
        format!("{}:{}:/usr/bin:/bin", shim_dir.display(), bin_dir.display())
    } else {
        format!(
            "{}:{}:{}:/usr/bin:/bin",
            shim_dir.display(),
            bin_dir.display(),
            host_path
        )
    };

    let mut cmd = Command::new(&shim_copy);
    cmd.env("SHIM_ORIGINAL_PATH", bin_dir.to_string_lossy().as_ref())
        .env("PATH", &shimmed_path)
        .env("SHIM_TRACE_LOG", &log_file)
        .env(
            "SUBSTRATE_MANAGER_MANIFEST",
            temp.path()
                .join("ambient-b")
                .join("manager_hooks.yaml")
                .to_string_lossy()
                .as_ref(),
        )
        .env("SUBSTRATE_WORLD_ENABLED", "false")
        .env("SUBSTRATE_WORLD", "disabled");
    let output = run_with_retry(cmd)?;

    assert!(!output.status.success());

    let hint_entry = fs::read_to_string(&trace_path)?
        .lines()
        .filter_map(|line| serde_json::from_str::<Value>(line).ok())
        .find(|value| value.get("manager_hint").is_some());

    assert!(
        hint_entry.is_none(),
        "hint should be skipped when world disabled"
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
    let trace_path = temp.path().join("trace.jsonl");

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

    let mut cmd = Command::new(&shim_node);
    cmd.env("PATH", &path_with_manager)
        .env(
            "SHIM_ORIGINAL_PATH",
            original_dir.to_string_lossy().as_ref(),
        )
        .env("SHIM_ACTIVE", "1") // simulate nested invocation
        .env("SHIM_TRACE_LOG", &log_file)
        .env("SHIM_CACHE_BUST", "1")
        .env_remove("SHIM_DEPTH");
    let output = run_with_retry(cmd)?;

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

    let log_content = fs::read_to_string(&trace_path)?;
    assert!(
        log_content.contains(&override_dir.display().to_string()),
        "log should record resolved override path: {log_content}"
    );

    Ok(())
}
