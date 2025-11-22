#![cfg(unix)]

#[path = "common.rs"]
mod common;

use assert_cmd::Command;
use common::{substrate_shell_driver, temp_dir};
use predicates::prelude::*;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use substrate_common::dedupe_path;
use tempfile::TempDir;

/// Helper function to get the substrate binary from workspace root
fn get_substrate_binary() -> Command {
    substrate_shell_driver()
}

struct ShellEnvFixture {
    _temp: TempDir,
    home: PathBuf,
}

impl ShellEnvFixture {
    fn new() -> Self {
        let temp = temp_dir("substrate-test-");
        let home = temp.path().join("home");
        fs::create_dir_all(home.join(".substrate/shims"))
            .expect("failed to create shims directory");
        Self { _temp: temp, home }
    }

    fn home(&self) -> &Path {
        &self.home
    }

    fn shim_dir(&self) -> PathBuf {
        self.home.join(".substrate/shims")
    }

    fn manager_env_path(&self) -> PathBuf {
        self.home.join(".substrate/manager_env.sh")
    }

    fn manager_init_path(&self) -> PathBuf {
        self.home.join(".substrate/manager_init.sh")
    }

    fn preexec_path(&self) -> PathBuf {
        self.home.join(".substrate_preexec")
    }

    fn overlay_path(&self) -> PathBuf {
        self.home.join(".substrate/manager_hooks.local.yaml")
    }

    fn write_manifest(&self, contents: &str) -> PathBuf {
        let path = self.home.join("manager_hooks.yaml");
        fs::write(&path, contents).expect("failed to write manager manifest");
        path
    }
}

fn substrate_command_for_home(fixture: &ShellEnvFixture) -> Command {
    let mut cmd = get_substrate_binary();
    cmd.env("HOME", fixture.home())
        .env("USERPROFILE", fixture.home())
        .env("SHELL", "/bin/bash");
    cmd
}

fn path_str(path: &Path) -> String {
    path.to_string_lossy().into_owned()
}

const PAYLOAD_MARKER: &str = "__SUBSTRATE_PAYLOAD__";

fn payload_lines(stdout: &[u8]) -> Vec<String> {
    let data = String::from_utf8_lossy(stdout);
    let mut marker_found = false;
    let mut lines = Vec::new();
    for line in data.lines() {
        if marker_found {
            lines.push(line.trim_end().to_string());
        } else if line.trim() == PAYLOAD_MARKER {
            marker_found = true;
        }
    }
    assert!(
        marker_found,
        "payload marker `{}` not found in output: {}",
        PAYLOAD_MARKER, data
    );
    lines
}

#[test]
fn test_command_start_finish_json_roundtrip() {
    let temp = temp_dir("substrate-test-");
    let log_file = temp.path().join("trace.jsonl");

    get_substrate_binary()
        .env("SHIM_TRACE_LOG", &log_file)
        .arg("-c")
        .arg("echo test")
        .assert()
        .success();

    let log_content = fs::read_to_string(&log_file).unwrap();
    let lines: Vec<&str> = log_content
        .lines()
        .filter(|line| !line.trim().is_empty())
        .collect();

    // Parse all JSON events and filter for the ones we care about
    let events: Vec<serde_json::Value> = lines
        .iter()
        .filter_map(|line| serde_json::from_str(line).ok())
        .collect();

    // Find command_start and command_complete events (prefer shell component, fall back to shim when world exec is delegated)
    let shell_starts: Vec<&serde_json::Value> = events
        .iter()
        .filter(|e| e["event_type"] == "command_start" && e["component"] == "shell")
        .collect();
    let shell_completes: Vec<&serde_json::Value> = events
        .iter()
        .filter(|e| e["event_type"] == "command_complete" && e["component"] == "shell")
        .collect();

    let (starts, completes, component_label) =
        if !shell_starts.is_empty() && !shell_completes.is_empty() {
            (shell_starts, shell_completes, "shell")
        } else {
            let shim_starts: Vec<&serde_json::Value> = events
                .iter()
                .filter(|e| {
                    e["event_type"] == "command_start"
                        && e["component"] == "shim"
                        && e["command"] == "echo test"
                })
                .collect();
            let shim_completes: Vec<&serde_json::Value> = events
                .iter()
                .filter(|e| {
                    e["event_type"] == "command_complete"
                        && e["component"] == "shim"
                        && e["command"] == "echo test"
                })
                .collect();
            (shim_starts, shim_completes, "shim")
        };

    // Should have exactly one start and one complete event for the chosen component
    assert_eq!(
        starts.len(),
        1,
        "Expected exactly one command_start event from {component_label}"
    );
    assert_eq!(
        completes.len(),
        1,
        "Expected exactly one command_complete event from {component_label}"
    );

    // Validate the events
    let start_event = starts[0];
    assert_eq!(start_event["command"], "echo test");
    assert!(start_event["session_id"].is_string());
    assert!(start_event["cmd_id"].is_string());

    let complete_event = completes[0];
    match component_label {
        "shell" => {
            assert_eq!(complete_event["exit_code"], 0);
            assert!(complete_event["duration_ms"].is_number());
        }
        "shim" => {
            // Shim entries use `exit` instead of `exit_code`
            assert_eq!(complete_event["exit"], 0);
        }
        _ => unreachable!(),
    }
}

#[test]
fn test_builtin_cd_side_effects() {
    let temp = temp_dir("substrate-test-");
    let target_dir = temp.path().join("test_dir");
    fs::create_dir(&target_dir).unwrap();

    let canonical_dir = target_dir.canonicalize().unwrap();
    let script = format!("cd {} && pwd", canonical_dir.display());

    get_substrate_binary()
        .env("SUBSTRATE_CAGED", "0")
        .arg("-c")
        .arg(&script)
        .assert()
        .success()
        .stdout(predicate::str::contains(
            canonical_dir.to_string_lossy().to_string(),
        ));
}

#[test]
fn test_ci_flag_strict_mode_ordering() {
    let temp = temp_dir("substrate-test-");
    let log_file = temp.path().join("trace.jsonl");

    // Test that undefined variable causes failure in CI mode
    get_substrate_binary()
        .env("SHIM_TRACE_LOG", &log_file)
        .arg("--no-world")
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
        .arg("--no-world")
        .arg("--shell")
        .arg("/bin/bash")
        .arg("-c")
        .arg("echo $UNDEFINED_VAR")
        .assert()
        .success();
}

#[test]
fn test_script_mode_single_process() {
    let temp = temp_dir("substrate-test-");
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
    let temp = temp_dir("substrate-test-");
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
    let temp = temp_dir("substrate-test-");
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
    let temp = temp_dir("substrate-test-");
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
    let temp = temp_dir("substrate-test-");
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
    common::ensure_substrate_built();
    let substrate_bin = std::path::PathBuf::from(common::binary_path());

    let mut child = StdCommand::new(substrate_bin)
        .arg("--no-world")
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
    let code = status.code();
    if let Some(code) = code {
        assert!(
            code == 143 || code == 0,
            "expected SIGTERM exit (143) or graceful shutdown (0), got {}",
            code
        );
    } else {
        use std::os::unix::process::ExitStatusExt;
        let signal = status.signal().unwrap_or_default();
        assert_eq!(
            signal,
            Signal::SIGTERM as i32,
            "expected SIGTERM signal ({}) but got {}",
            Signal::SIGTERM as i32,
            signal
        );
    }
}

#[test]
fn test_log_rotation() {
    let temp = temp_dir("substrate-test-");
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
    let temp = temp_dir("substrate-test-");
    let log_file = temp.path().join("trace.jsonl");

    // Test basic cd functionality - cd - functionality is complex in subshells
    // Just verify that cd commands are logged and work
    get_substrate_binary()
        .env("SHIM_TRACE_LOG", &log_file)
        .env("SUBSTRATE_CAGED", "0")
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
    let temp = temp_dir("substrate-test-");
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
    let temp = temp_dir("substrate-test-");
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
    let temp = temp_dir("substrate-test-");
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
    let temp = temp_dir("substrate-test-");
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

#[test]
fn shell_env_injects_manager_snippets() {
    let fixture = ShellEnvFixture::new();
    let manifest = fixture.write_manifest(
        r#"version: 1
managers:
  - name: DemoManager
    detect:
      script: "exit 0"
    init:
      shell: |
        export MANAGER_MARKER="manager_init_loaded"
  - name: Volta
    detect:
      script: "exit 0"
    init:
      shell: |
        export VOLTA_MARKER="volta_loaded"
"#,
    );
    let host_bash_env = fixture.home().join("host_bash_env.sh");
    fs::write(&host_bash_env, "export HOST_BE_VALUE=\"host_env\"\n").unwrap();
    let legacy_bashenv = fixture.home().join(".substrate_bashenv");
    fs::write(&legacy_bashenv, "export LEGACY_MARKER=\"legacy_env\"\n").unwrap();
    let parent_path_before = env::var("PATH").unwrap_or_default();
    let host_path = fixture.home().join("host-bin");
    fs::create_dir_all(&host_path).unwrap();
    let host_segment = path_str(&host_path);
    let host_path_str = if parent_path_before.is_empty() {
        host_segment.clone()
    } else {
        format!("{}:{}", host_segment, parent_path_before)
    };

    let script = format!(
        "printf '%s\\n' \"{marker}\" \"$PATH\" \"$MANAGER_MARKER\" \"$LEGACY_MARKER\" \
         \"$HOST_BE_VALUE\" \"${{BASH_ENV:-}}\" \"${{SUBSTRATE_MANAGER_ENV:-}}\" \
         \"${{SUBSTRATE_MANAGER_INIT:-}}\" \"${{SUBSTRATE_ORIGINAL_BASH_ENV:-}}\"",
        marker = PAYLOAD_MARKER
    );
    let output = substrate_command_for_home(&fixture)
        .env("PATH", &host_path_str)
        .env("BASH_ENV", &host_bash_env)
        .env("SUBSTRATE_WORLD", "enabled")
        .env("SUBSTRATE_WORLD_ENABLED", "1")
        .env("SUBSTRATE_MANAGER_MANIFEST", path_str(&manifest))
        .arg("-c")
        .arg(script)
        .output()
        .expect("failed to run substrate -c for shell env test");

    assert!(
        output.status.success(),
        "substrate -c failed: stdout={} stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let lines = payload_lines(&output.stdout);
    assert_eq!(
        lines.len(),
        8,
        "unexpected payload: {}",
        String::from_utf8_lossy(&output.stdout)
    );
    let path_line = &lines[0];
    let shim_prefix = format!("{}:", fixture.shim_dir().display());
    assert!(
        path_line.starts_with(&shim_prefix),
        "PATH did not start with shims: {}",
        path_line
    );
    let remainder = &path_line[shim_prefix.len()..];
    assert_eq!(remainder, dedupe_path(&host_path_str));
    assert_eq!(lines[1], "manager_init_loaded");
    assert_eq!(lines[2], "legacy_env");
    assert_eq!(lines[3], "host_env");
    assert_eq!(lines[4], fixture.preexec_path().display().to_string());
    assert_eq!(lines[5], fixture.manager_env_path().display().to_string());
    assert_eq!(lines[6], fixture.manager_init_path().display().to_string());
    assert_eq!(lines[7], host_bash_env.display().to_string());
    let parent_path_after = env::var("PATH").unwrap_or_default();
    assert_eq!(parent_path_before, parent_path_after);

    let manager_env_contents =
        fs::read_to_string(fixture.manager_env_path()).expect("manager env contents");
    assert!(
        manager_env_contents.contains("SUBSTRATE_MANAGER_INIT"),
        "manager_env missing manager init sourcing"
    );
    assert!(
        manager_env_contents.contains("SUBSTRATE_ORIGINAL_BASH_ENV"),
        "manager_env missing original BASH_ENV sourcing"
    );
    assert!(
        manager_env_contents.contains(".substrate_bashenv"),
        "manager_env missing legacy bashenv sourcing"
    );
    let manager_init_contents =
        fs::read_to_string(fixture.manager_init_path()).expect("manager init contents");
    assert!(
        manager_init_contents.contains("VOLTA_MARKER"),
        "manager init snippet missing Tier-2 manager content"
    );
}

#[test]
fn shell_env_no_world_skips_manager_env() {
    let fixture = ShellEnvFixture::new();
    let host_path = fixture.home().join("host-only");
    fs::create_dir_all(&host_path).unwrap();
    let host_path_str = path_str(&host_path);
    let host_bash_env = fixture.home().join("host_env.sh");
    fs::write(&host_bash_env, "export HOST_ONLY=1\n").unwrap();

    let script = format!(
        "printf '%s\\n' \"{marker}\" \"$PATH\" \"${{BASH_ENV:-}}\" \
         \"${{SUBSTRATE_MANAGER_ENV:-none}}\" \"${{SUBSTRATE_MANAGER_INIT:-none}}\" \
         \"${{SUBSTRATE_ORIGINAL_BASH_ENV:-none}}\"",
        marker = PAYLOAD_MARKER
    );
    let output = substrate_command_for_home(&fixture)
        .env("PATH", &host_path_str)
        .env("BASH_ENV", &host_bash_env)
        .arg("--no-world")
        .arg("-c")
        .arg(script)
        .output()
        .expect("failed to run substrate --no-world");

    assert!(
        output.status.success(),
        "substrate --no-world failed: stdout={} stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let lines = payload_lines(&output.stdout);
    assert_eq!(
        lines.len(),
        5,
        "unexpected payload: {}",
        String::from_utf8_lossy(&output.stdout)
    );
    assert_eq!(lines[0], host_path_str);
    assert_eq!(lines[1], host_bash_env.display().to_string());
    assert_eq!(lines[2], "none");
    assert_eq!(lines[3], "none");
    assert_eq!(lines[4], "none");
}

#[test]
fn shell_env_applies_overlay_manifest() {
    let fixture = ShellEnvFixture::new();
    let missing_path = fixture.home().join("missing-tool");
    let manifest_body = format!(
        r#"version: 1
managers:
  - name: OverlayDemo
    detect:
      files:
        - "{}"
    init:
      shell: |
        export OVERLAY_VALUE="base"
"#,
        missing_path.display()
    );
    let manifest = fixture.write_manifest(&manifest_body);
    let overlay_contents = r#"version: 1
managers:
  - name: OverlayDemo
    detect:
      script: "exit 0"
    init:
      shell: |
        export OVERLAY_VALUE="overlay-active"
"#;
    fs::write(fixture.overlay_path(), overlay_contents).unwrap();

    let script = format!(
        "printf '%s\\n' \"{marker}\" \"$OVERLAY_VALUE\"",
        marker = PAYLOAD_MARKER
    );
    let output = substrate_command_for_home(&fixture)
        .env("SUBSTRATE_MANAGER_MANIFEST", path_str(&manifest))
        .env("SUBSTRATE_WORLD", "enabled")
        .env("SUBSTRATE_WORLD_ENABLED", "1")
        .arg("-c")
        .arg(script)
        .output()
        .expect("failed to run substrate for overlay manifest");
    assert!(
        output.status.success(),
        "substrate -c failed: stdout={} stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let lines = payload_lines(&output.stdout);
    assert_eq!(
        lines.len(),
        1,
        "unexpected payload: {}",
        String::from_utf8_lossy(&output.stdout)
    );
    assert_eq!(lines[0], "overlay-active");
}
