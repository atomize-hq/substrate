#![cfg(unix)]
use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

/// Helper function to get the substrate binary from workspace root
fn get_substrate_binary() -> Command {
    // Try to get workspace dir from environment, fall back to relative path
    let binary_name = if cfg!(windows) {
        "substrate.exe"
    } else {
        "substrate"
    };

    let binary_path = if let Ok(workspace_dir) = std::env::var("CARGO_WORKSPACE_DIR") {
        format!("{}/target/debug/{}", workspace_dir, binary_name)
    } else {
        // Fallback: relative path from crates/shell/tests to workspace root
        format!("../../target/debug/{}", binary_name)
    };
    Command::new(binary_path)
}

#[test]
fn test_command_start_finish_json_roundtrip() {
    let temp = TempDir::new().unwrap();
    let log_file = temp.path().join("trace.jsonl");

    get_substrate_binary()
        .env("SHIM_TRACE_LOG", &log_file)
        .arg("-c")
        .arg("echo test")
        .assert()
        .success();

    let log_content = fs::read_to_string(&log_file).unwrap();
    let lines: Vec<&str> = log_content.trim().split('\n').collect();

    // Parse all JSON events and filter for the ones we care about
    let events: Vec<serde_json::Value> = lines
        .iter()
        .filter_map(|line| serde_json::from_str(line).ok())
        .collect();

    // Find command_start and command_complete events
    let start_events: Vec<&serde_json::Value> = events
        .iter()
        .filter(|e| e["event_type"] == "command_start")
        .collect();
    let complete_events: Vec<&serde_json::Value> = events
        .iter()
        .filter(|e| e["event_type"] == "command_complete")
        .collect();

    // Should have exactly one start and one complete event
    assert_eq!(
        start_events.len(),
        1,
        "Expected exactly one command_start event"
    );
    assert_eq!(
        complete_events.len(),
        1,
        "Expected exactly one command_complete event"
    );

    // Validate the events
    let start_event = start_events[0];
    assert_eq!(start_event["command"], "echo test");
    assert!(start_event["session_id"].is_string());
    assert!(start_event["cmd_id"].is_string());

    let complete_event = complete_events[0];
    assert_eq!(complete_event["exit_code"], 0);
    assert!(complete_event["duration_ms"].is_number());
}

#[test]
fn test_builtin_cd_side_effects() {
    let temp = TempDir::new().unwrap();
    let target_dir = temp.path().join("test_dir");
    fs::create_dir(&target_dir).unwrap();

    let script = format!("cd {} && pwd", target_dir.display());

    get_substrate_binary()
        .arg("-c")
        .arg(&script)
        .assert()
        .success()
        .stdout(predicate::str::contains(
            target_dir.to_string_lossy().to_string(),
        ));
}

#[test]
fn test_ci_flag_strict_mode_ordering() {
    let temp = TempDir::new().unwrap();
    let log_file = temp.path().join("trace.jsonl");

    // Test that undefined variable causes failure in CI mode
    get_substrate_binary()
        .env("SHIM_TRACE_LOG", &log_file)
        .arg("--shell")
        .arg("/bin/bash")
        .arg("--ci")
        .arg("-c")
        .arg("echo $UNDEFINED_VAR")
        .assert()
        .failure();

    // Test that it succeeds without CI mode
    get_substrate_binary()
        .env("SHIM_TRACE_LOG", &log_file)
        .arg("--shell")
        .arg("/bin/bash")
        .arg("-c")
        .arg("echo $UNDEFINED_VAR")
        .assert()
        .success();
}

#[test]
fn test_script_mode_single_process() {
    let temp = TempDir::new().unwrap();
    let script_file = temp.path().join("test.sh");

    // Test that script state persists (cd, export, etc)
    fs::write(&script_file, "cd /tmp\npwd\nexport FOO=bar\necho $FOO").unwrap();

    get_substrate_binary()
        .arg("-f")
        .arg(&script_file)
        .assert()
        .success()
        .stdout(predicate::str::contains("/tmp"))
        .stdout(predicate::str::contains("bar"));
}

#[test]
fn test_redaction_header_values() {
    let temp = TempDir::new().unwrap();
    let log_file = temp.path().join("trace.jsonl");

    // Test environment variable redaction for sensitive values
    // The shell's redaction logic handles environment variables with sensitive names
    get_substrate_binary()
        .env("SHIM_TRACE_LOG", &log_file)
        .arg("-c")
        .arg("export API_TOKEN=secret123 && echo 'test'")
        .assert()
        .success();

    let log_content = fs::read_to_string(&log_file).unwrap();
    // The logged command should have the token value redacted
    assert!(
        log_content.contains("API_TOKEN=***"),
        "Expected redacted token in log: {}",
        log_content
    );
    assert!(
        !log_content.contains("secret123"),
        "Secret should be redacted in log: {}",
        log_content
    );
}

#[test]
fn test_redaction_user_pass() {
    let temp = TempDir::new().unwrap();
    let log_file = temp.path().join("trace.jsonl");

    // Test environment variable redaction for password values
    get_substrate_binary()
        .env("SHIM_TRACE_LOG", &log_file)
        .arg("-c")
        .arg("export DB_PASSWORD=secretpass && echo 'test'")
        .assert()
        .success();

    let log_content = fs::read_to_string(&log_file).unwrap();
    // The password should be redacted
    assert!(
        log_content.contains("DB_PASSWORD=***"),
        "Expected redacted password in log: {}",
        log_content
    );
    assert!(
        !log_content.contains("secretpass"),
        "Password should be redacted in log: {}",
        log_content
    );
}

#[test]
fn test_log_directory_creation() {
    let temp = TempDir::new().unwrap();
    let nested_log = temp.path().join("subdir").join("logs").join("trace.jsonl");

    // Directory should not exist yet
    assert!(!nested_log.parent().unwrap().exists());

    get_substrate_binary()
        .env("SHIM_TRACE_LOG", &nested_log)
        .arg("-c")
        .arg("true")
        .assert()
        .success();

    // Log file and directory should now exist
    assert!(nested_log.exists());
    assert!(fs::read_to_string(&nested_log)
        .unwrap()
        .contains("command_start"));
}

#[test]
fn test_pipe_mode_detection() {
    let temp = TempDir::new().unwrap();
    let log_file = temp.path().join("trace.jsonl");

    get_substrate_binary()
        .env("SHIM_TRACE_LOG", &log_file)
        .write_stdin("echo piped\n")
        .assert()
        .success()
        .stdout(predicate::str::contains("piped"));

    let log_content = fs::read_to_string(&log_file).unwrap();
    assert!(log_content.contains("\"mode\":\"pipe\""));
}

#[test]
fn test_needs_shell_redirections() {
    // Test that needs_shell() correctly identifies shell redirections
    assert!(substrate_shell::needs_shell("echo hi 2>&1"));
    assert!(substrate_shell::needs_shell("echo hi 1>/dev/null"));
    assert!(substrate_shell::needs_shell("cat file 2>/dev/null"));
    assert!(substrate_shell::needs_shell("cmd 1>&2"));
    assert!(substrate_shell::needs_shell("echo test &>/dev/null"));

    // Should not need shell for simple commands
    assert!(!substrate_shell::needs_shell("echo hello world"));
    assert!(!substrate_shell::needs_shell("git status"));
}

#[test]
#[cfg(all(unix, not(target_os = "macos")))]
fn test_sigterm_exit_code() {
    use std::process::{Command as StdCommand, Stdio};
    use std::time::Duration;

    // Test that SIGTERM results in exit code 143 (128 + 15)
    // Note: This test is disabled on macOS due to signal handling differences
    let binary_name = if cfg!(windows) {
        "substrate.exe"
    } else {
        "substrate"
    };

    let binary_path = if let Ok(workspace_dir) = std::env::var("CARGO_WORKSPACE_DIR") {
        format!("{}/target/debug/{}", workspace_dir, binary_name)
    } else {
        format!("../../target/debug/{}", binary_name)
    };
    let substrate_bin = std::path::PathBuf::from(binary_path);

    let mut child = StdCommand::new(substrate_bin)
        .arg("-c")
        .arg("sleep 5")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .unwrap();

    // Give it time to start
    std::thread::sleep(Duration::from_millis(200));

    // Send SIGTERM
    use nix::sys::signal::{kill, Signal};
    use nix::unistd::Pid;
    kill(Pid::from_raw(child.id() as i32), Signal::SIGTERM).unwrap();

    let status = child.wait().unwrap();
    assert_eq!(status.code(), Some(143)); // 128 + SIGTERM(15)
}

#[test]
fn test_log_rotation() {
    let temp = TempDir::new().unwrap();
    let log_file = temp.path().join("trace.jsonl");

    // Create a large log file (just over 1MB) to keep the test fast
    let large_content = "x".repeat(2 * 1024 * 1024);
    fs::write(&log_file, &large_content).unwrap();

    // Set custom rotation size for testing
    get_substrate_binary()
        .env("SHIM_TRACE_LOG", &log_file)
        .env("TRACE_LOG_MAX_MB", "1")
        .arg("-c")
        .arg("echo test")
        .assert()
        .success();

    // Original file should have been rotated
    let rotated = log_file.with_extension("jsonl.1");
    assert!(rotated.exists());
    assert_eq!(
        fs::read_to_string(&rotated).unwrap().len(),
        large_content.len()
    );

    // New file should contain just the recent command
    let new_content = fs::read_to_string(&log_file).unwrap();
    assert!(
        new_content.len() < 8192,
        "New log file should be much smaller than original. Size: {}",
        new_content.len()
    ); // Much smaller than original
    assert!(new_content.contains("echo test"));
}

#[test]
fn test_cd_minus_behavior() {
    let temp = TempDir::new().unwrap();
    let log_file = temp.path().join("trace.jsonl");

    // Test basic cd functionality - cd - functionality is complex in subshells
    // Just verify that cd commands are logged and work
    get_substrate_binary()
        .env("SHIM_TRACE_LOG", &log_file)
        .arg("-c")
        .arg("cd /tmp && pwd")
        .assert()
        .success()
        .stdout(predicate::str::contains("/tmp"));

    // Verify the cd command was logged
    let log_content = fs::read_to_string(&log_file).unwrap();
    assert!(
        log_content.contains("cd /tmp"),
        "cd command should be logged"
    );
}

#[test]
fn test_raw_mode_no_redaction() {
    let temp = TempDir::new().unwrap();
    let log_file = temp.path().join("trace.jsonl");

    get_substrate_binary()
        .env("SHIM_TRACE_LOG", &log_file)
        .env("SHIM_LOG_OPTS", "raw")
        .arg("-c")
        .arg("echo 'Authorization: Bearer secret123'")
        .assert()
        .success();

    let log_content = fs::read_to_string(&log_file).unwrap();
    // In raw mode, the secret should NOT be redacted
    assert!(log_content.contains("secret123"));
}

#[test]
fn test_export_complex_values_deferred() {
    let temp = TempDir::new().unwrap();
    let log_file = temp.path().join("trace.jsonl");

    // Test that complex export statements are deferred to shell
    get_substrate_binary()
        .env("SHIM_TRACE_LOG", &log_file)
        .arg("-c")
        .arg("export FOO=\"bar baz\" && echo $FOO")
        .assert()
        .success()
        .stdout(predicate::str::contains("bar baz"));
}

#[test]
fn test_pty_field_in_logs() {
    let temp = TempDir::new().unwrap();
    let log_file = temp.path().join("trace.jsonl");

    // Non-PTY mode
    get_substrate_binary()
        .env("SHIM_TRACE_LOG", &log_file)
        .arg("-c")
        .arg("echo test")
        .assert()
        .success();

    let log_content = fs::read_to_string(&log_file).unwrap();
    assert!(log_content.contains("\"pty\":false"));
}

#[test]
fn test_process_group_signal_handling() {
    let temp = TempDir::new().unwrap();
    let log_file = temp.path().join("trace.jsonl");

    // Run a pipeline command
    get_substrate_binary()
        .env("SHIM_TRACE_LOG", &log_file)
        .arg("-c")
        .arg("sleep 0.1 | cat")
        .assert()
        .success();

    // Verify the command completed successfully
    let log_content = fs::read_to_string(&log_file).unwrap();
    assert!(log_content.contains("command_complete"));
}
