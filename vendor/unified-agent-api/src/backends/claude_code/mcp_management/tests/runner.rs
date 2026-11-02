use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
    time::Duration,
};

use crate::{mcp::AgentWrapperMcpAddTransport, AgentWrapperError};

use super::super::{
    claude_mcp_add_argv, claude_mcp_get_argv, claude_mcp_list_argv, claude_mcp_remove_argv,
    run_claude_mcp,
    runner::{
        capture_bounded, classify_manifest_runtime_conflict_text, finalize_claude_mcp_output,
        CapturedClaudeMcpCommandOutput,
    },
    PINNED_MCP_RUNTIME_CONFLICT,
};
use super::support::{
    exit_status_with_code, success_exit_status, test_env_lock, write_all_and_close, EnvGuard,
};

#[cfg(unix)]
use std::{fs, os::unix::fs::PermissionsExt};

#[cfg(unix)]
use super::support::{temp_test_dir, write_fake_claude};

const PATH_ENV: &str = "PATH";
const UNIX_TEST_PATH: &str = "/usr/bin:/bin";

#[cfg(unix)]
fn write_test_executable(path: &Path, script: &str) {
    fs::write(path, script).expect("script should be written");
    let mut permissions = fs::metadata(path)
        .expect("script metadata should exist")
        .permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(path, permissions).expect("script should be executable");
}

fn sample_stdio_transport() -> AgentWrapperMcpAddTransport {
    AgentWrapperMcpAddTransport::Stdio {
        command: vec!["node".to_string()],
        args: vec!["server.js".to_string()],
        env: BTreeMap::new(),
    }
}

fn assert_runtime_conflict(err: AgentWrapperError) {
    match err {
        AgentWrapperError::Backend { message } => {
            assert_eq!(message, PINNED_MCP_RUNTIME_CONFLICT);
        }
        other => panic!("expected Backend error, got {other:?}"),
    }
}

fn assert_timeout_error(err: AgentWrapperError) {
    match err {
        AgentWrapperError::Backend { message } => {
            assert_eq!(message, super::super::PINNED_TIMEOUT_FAILURE);
        }
        other => panic!("expected Backend error, got {other:?}"),
    }
}

#[cfg(unix)]
#[tokio::test]
async fn run_claude_mcp_zero_timeout_fails_before_materializing_or_spawning() {
    let temp_dir = temp_test_dir("zero-timeout");
    let script_dir = temp_dir.join("bin");
    let script_path = write_fake_claude(
        &script_dir,
        r#"#!/usr/bin/env bash
touch "$MARKER_PATH"
"#,
    );
    let marker_path = temp_dir.join("spawned.marker");
    let isolated_home = temp_dir.join("isolated-claude-home");

    let err = run_claude_mcp(
        super::super::super::ClaudeCodeBackendConfig {
            binary: Some(script_path),
            claude_home: Some(isolated_home.clone()),
            ..Default::default()
        },
        claude_mcp_list_argv(),
        crate::mcp::AgentWrapperMcpCommandContext {
            timeout: Some(Duration::ZERO),
            env: BTreeMap::from([
                (PATH_ENV.to_string(), UNIX_TEST_PATH.to_string()),
                (
                    "MARKER_PATH".to_string(),
                    marker_path.to_string_lossy().into_owned(),
                ),
            ]),
            ..Default::default()
        },
    )
    .await
    .expect_err("zero timeout should fail before spawn");

    assert_timeout_error(err);
    assert!(
        !marker_path.exists(),
        "zero-timeout runner path must not spawn the claude process"
    );
    assert!(
        !isolated_home.exists(),
        "zero-timeout runner path must not materialize CLAUDE_HOME"
    );

    fs::remove_dir_all(temp_dir).expect("temp dir should be removed");
}

#[cfg(unix)]
#[tokio::test]
#[allow(clippy::await_holding_lock)]
async fn run_claude_mcp_clears_ambient_env_before_spawn() {
    let _env_lock = test_env_lock();
    let temp_dir = temp_test_dir("ambient-env");
    let script_path = temp_dir.join("claude");
    write_test_executable(
        &script_path,
        r#"#!/usr/bin/env bash
printf "AMBIENT_ONLY=%s\n" "${AMBIENT_ONLY-unset}" 1>&2
printf "CLAUDE_HOME=%s\n" "${CLAUDE_HOME-unset}" 1>&2
printf "HOME=%s\n" "${HOME-unset}" 1>&2
printf "PATH=%s\n" "${PATH-unset}" 1>&2
"#,
    );
    let _ambient_only = EnvGuard::set("AMBIENT_ONLY", "ambient-value");
    let _ambient_home = EnvGuard::set("CLAUDE_HOME", "/tmp/ambient-claude-home");

    let result = run_claude_mcp(
        super::super::super::ClaudeCodeBackendConfig {
            binary: Some(script_path),
            claude_home: Some(PathBuf::from("/tmp/resolved-claude-home")),
            ..Default::default()
        },
        claude_mcp_list_argv(),
        crate::mcp::AgentWrapperMcpCommandContext {
            env: BTreeMap::from([(PATH_ENV.to_string(), UNIX_TEST_PATH.to_string())]),
            ..Default::default()
        },
    )
    .await
    .expect("runner should succeed");

    assert_eq!(
        result.stderr.lines().collect::<Vec<_>>(),
        vec![
            "AMBIENT_ONLY=unset".to_string(),
            "CLAUDE_HOME=/tmp/resolved-claude-home".to_string(),
            "HOME=/tmp/resolved-claude-home".to_string(),
            format!("PATH={UNIX_TEST_PATH}"),
        ]
    );

    fs::remove_dir_all(temp_dir).expect("temp dir should be removed");
}

#[cfg(unix)]
#[tokio::test]
#[allow(clippy::await_holding_lock)]
async fn run_claude_mcp_preserves_path_for_launcher_script_helpers() {
    let _env_lock = test_env_lock();
    let temp_dir = temp_test_dir("path-helper");
    let helper_path = temp_dir.join("helper-bin");
    let script_path = temp_dir.join("claude");
    let effective_path = format!("{}:{UNIX_TEST_PATH}", temp_dir.to_string_lossy());

    write_test_executable(&helper_path, "#!/usr/bin/env bash\nprintf 'helper-ran\\n'");
    write_test_executable(&script_path, "#!/usr/bin/env bash\nhelper-bin\n");

    let result = run_claude_mcp(
        super::super::super::ClaudeCodeBackendConfig {
            binary: Some(script_path),
            ..Default::default()
        },
        claude_mcp_list_argv(),
        crate::mcp::AgentWrapperMcpCommandContext {
            env: BTreeMap::from([(PATH_ENV.to_string(), effective_path.clone())]),
            ..Default::default()
        },
    )
    .await
    .expect("runner should succeed");

    assert!(
        result.status.success(),
        "launcher helper should resolve via PATH"
    );
    assert_eq!(result.stdout, "helper-ran\n");
    assert!(result.stderr.is_empty(), "stderr should remain empty");

    fs::remove_dir_all(temp_dir).expect("temp dir should be removed");
}

#[tokio::test]
async fn capture_bounded_preserves_small_streams() {
    let (writer, reader) = tokio::io::duplex(64);
    let writer_task = tokio::spawn(write_all_and_close(writer, b"hello".to_vec()));

    let (captured, saw_more) = capture_bounded(reader, 8).await.expect("capture succeeds");
    writer_task.await.expect("writer completes");

    assert_eq!(captured, b"hello");
    assert!(!saw_more);
}

#[tokio::test]
async fn capture_bounded_retains_only_bound_and_marks_overflow() {
    let (writer, reader) = tokio::io::duplex(64);
    let writer_task = tokio::spawn(write_all_and_close(
        writer,
        b"abcdefghijklmnopqrstuvwxyz".to_vec(),
    ));

    let (captured, saw_more) = capture_bounded(reader, 8).await.expect("capture succeeds");
    writer_task.await.expect("writer completes");

    assert_eq!(captured, b"abcdefgh");
    assert!(saw_more);
}

#[tokio::test]
async fn capture_bounded_with_zero_bound_drains_input_and_reports_overflow() {
    let (writer, reader) = tokio::io::duplex(64);
    let writer_task = tokio::spawn(write_all_and_close(writer, b"abcdef".to_vec()));

    let (captured, saw_more) = capture_bounded(reader, 0).await.expect("capture succeeds");
    writer_task.await.expect("writer completes");

    assert!(captured.is_empty());
    assert!(saw_more);
}

#[test]
fn classify_manifest_runtime_conflict_detects_unknown_mcp_command() {
    assert!(classify_manifest_runtime_conflict_text(
        &claude_mcp_list_argv(),
        "error: unrecognized subcommand 'mcp'"
    ));
}

#[test]
fn classify_manifest_runtime_conflict_detects_unknown_get_subcommand() {
    assert!(classify_manifest_runtime_conflict_text(
        &claude_mcp_get_argv("demo"),
        "error: no such subcommand 'get'"
    ));
}

#[test]
fn classify_manifest_runtime_conflict_detects_unknown_add_subcommand() {
    let argv =
        claude_mcp_add_argv("demo", &sample_stdio_transport()).expect("stdio transport should map");

    assert!(classify_manifest_runtime_conflict_text(
        &argv,
        "error: unknown subcommand 'add'"
    ));
}

#[test]
fn classify_manifest_runtime_conflict_detects_add_transport_flag_drift_without_echoed_add() {
    let argv =
        claude_mcp_add_argv("demo", &sample_stdio_transport()).expect("stdio transport should map");

    assert!(classify_manifest_runtime_conflict_text(
        &argv,
        "error: unexpected argument '--transport' found"
    ));
}

#[test]
fn classify_manifest_runtime_conflict_detects_add_env_flag_usage_drift() {
    let transport = AgentWrapperMcpAddTransport::Stdio {
        command: vec!["node".to_string()],
        args: vec!["server.js".to_string()],
        env: BTreeMap::from([("ALPHA_ENV".to_string(), "1".to_string())]),
    };
    let argv = claude_mcp_add_argv("demo", &transport).expect("stdio transport should map");

    assert!(classify_manifest_runtime_conflict_text(
        &argv,
        "error: unexpected argument '--env' found\n\nusage: claude mcp add [options]"
    ));
}

#[test]
fn classify_manifest_runtime_conflict_detects_unknown_list_subcommand() {
    assert!(classify_manifest_runtime_conflict_text(
        &claude_mcp_list_argv(),
        "error: unknown subcommand 'list'"
    ));
}

#[test]
fn classify_manifest_runtime_conflict_detects_unknown_remove_subcommand() {
    assert!(classify_manifest_runtime_conflict_text(
        &claude_mcp_remove_argv("demo"),
        "error: unrecognized subcommand 'remove'"
    ));
}

#[test]
fn classify_manifest_runtime_conflict_ignores_domain_failures() {
    assert!(!classify_manifest_runtime_conflict_text(
        &claude_mcp_get_argv("demo"),
        "server demo not found"
    ));
    assert!(!classify_manifest_runtime_conflict_text(
        &claude_mcp_get_argv("demo"),
        "unknown server demo"
    ));
    assert!(!classify_manifest_runtime_conflict_text(
        &claude_mcp_get_argv("demo"),
        "permission denied while contacting remote service"
    ));
    assert!(!classify_manifest_runtime_conflict_text(
        &claude_mcp_get_argv("demo"),
        "network error: failed to connect"
    ));

    let argv =
        claude_mcp_add_argv("demo", &sample_stdio_transport()).expect("stdio transport should map");
    assert!(!classify_manifest_runtime_conflict_text(
        &argv,
        "error: unexpected argument '--foo' found"
    ));
}

#[test]
fn finalize_claude_mcp_output_returns_backend_error_for_drift() {
    let err = finalize_claude_mcp_output(
        &claude_mcp_get_argv("demo"),
        CapturedClaudeMcpCommandOutput {
            status: exit_status_with_code(2),
            stdout_bytes: b"raw stdout should not leak".to_vec(),
            stdout_saw_more: false,
            stderr_bytes: b"error: no such subcommand 'get'".to_vec(),
            stderr_saw_more: false,
        },
    )
    .expect_err("drift should fail closed");

    assert_runtime_conflict(err);
}

#[test]
fn finalize_claude_mcp_output_returns_backend_error_for_add_flag_drift() {
    let argv =
        claude_mcp_add_argv("demo", &sample_stdio_transport()).expect("stdio transport should map");

    let err = finalize_claude_mcp_output(
        &argv,
        CapturedClaudeMcpCommandOutput {
            status: exit_status_with_code(2),
            stdout_bytes: b"raw stdout should not leak".to_vec(),
            stdout_saw_more: false,
            stderr_bytes: b"error: unexpected argument '--transport' found".to_vec(),
            stderr_saw_more: false,
        },
    )
    .expect_err("add flag drift should fail closed");

    assert_runtime_conflict(err);
}

#[test]
fn finalize_claude_mcp_output_keeps_normal_non_zero_exits_as_ok() {
    let output = finalize_claude_mcp_output(
        &claude_mcp_get_argv("demo"),
        CapturedClaudeMcpCommandOutput {
            status: exit_status_with_code(3),
            stdout_bytes: b"listed output".to_vec(),
            stdout_saw_more: false,
            stderr_bytes: b"server demo not found".to_vec(),
            stderr_saw_more: false,
        },
    )
    .expect("normal failures should remain Ok(output)");

    assert_eq!(output.status, exit_status_with_code(3));
    assert_eq!(output.stdout, "listed output");
    assert_eq!(output.stderr, "server demo not found");
    assert!(!output.stdout_truncated);
    assert!(!output.stderr_truncated);
}

#[test]
fn finalize_claude_mcp_output_detects_drift_in_stdout_too() {
    let err = finalize_claude_mcp_output(
        &claude_mcp_list_argv(),
        CapturedClaudeMcpCommandOutput {
            status: exit_status_with_code(4),
            stdout_bytes: b"error: unknown subcommand 'list'".to_vec(),
            stdout_saw_more: false,
            stderr_bytes: Vec::new(),
            stderr_saw_more: false,
        },
    )
    .expect_err("stdout drift should fail closed");

    assert_runtime_conflict(err);
}

#[test]
fn success_exit_skips_drift_classification() {
    let output = finalize_claude_mcp_output(
        &claude_mcp_get_argv("demo"),
        CapturedClaudeMcpCommandOutput {
            status: success_exit_status(),
            stdout_bytes: b"error: no such subcommand 'get'".to_vec(),
            stdout_saw_more: false,
            stderr_bytes: Vec::new(),
            stderr_saw_more: false,
        },
    )
    .expect("successful exits should remain Ok(output)");

    assert_eq!(output.status, success_exit_status());
    assert_eq!(output.stdout, "error: no such subcommand 'get'");
    assert!(output.stderr.is_empty());
}
