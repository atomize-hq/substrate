use std::{
    collections::BTreeMap,
    ffi::OsString,
    path::{Path, PathBuf},
    time::Duration,
};

use crate::{
    bounds::enforce_mcp_output_bound,
    mcp::{AgentWrapperMcpAddTransport, AgentWrapperMcpCommandContext},
    AgentWrapperError,
};

use super::super::{
    argv::{codex_mcp_add_argv, codex_mcp_get_argv, codex_mcp_list_argv, codex_mcp_remove_argv},
    run_codex_mcp,
    runner::{
        capture_bounded, classify_manifest_runtime_conflict_text, finalize_codex_mcp_output,
        CapturedCodexMcpCommandOutput,
    },
    PINNED_MCP_RUNTIME_CONFLICT,
};
use super::support::{
    exit_status_with_code, success_exit_status, test_env_lock, write_all_and_close, EnvGuard,
};

#[cfg(unix)]
use std::fs;

#[cfg(unix)]
use super::support::{temp_test_dir, write_fake_codex};

#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

const PATH_ENV: &str = "PATH";
const UNIX_TEST_PATH: &str = "/usr/bin:/bin";

fn assert_timeout_error(err: AgentWrapperError) {
    match err {
        AgentWrapperError::Backend { message } => {
            assert_eq!(message, super::super::super::PINNED_TIMEOUT);
        }
        other => panic!("expected Backend error, got {other:?}"),
    }
}

#[cfg(unix)]
fn write_test_executable(path: &Path, script: &str) {
    fs::write(path, script).expect("script should be written");
    let mut permissions = fs::metadata(path)
        .expect("script metadata should exist")
        .permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(path, permissions).expect("script should be executable");
}

#[cfg(unix)]
#[tokio::test]
async fn run_codex_mcp_zero_timeout_fails_before_materializing_or_spawning() {
    let temp_dir = temp_test_dir("zero-timeout");
    let script_dir = temp_dir.join("bin");
    let script_path = write_fake_codex(
        &script_dir,
        r#"#!/bin/sh
touch "$MARKER_PATH"
"#,
    );
    let marker_path = temp_dir.join("spawned.marker");
    let isolated_home = temp_dir.join("isolated-codex-home");

    let err = run_codex_mcp(
        super::super::super::CodexBackendConfig {
            binary: Some(script_path),
            codex_home: Some(isolated_home.clone()),
            ..Default::default()
        },
        codex_mcp_list_argv(),
        AgentWrapperMcpCommandContext {
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
        "zero-timeout runner path must not spawn the codex process"
    );
    assert!(
        !isolated_home.exists(),
        "zero-timeout runner path must not materialize CODEX_HOME"
    );

    fs::remove_dir_all(temp_dir).expect("temp dir should be removed");
}

#[cfg(unix)]
#[tokio::test]
async fn run_codex_mcp_uses_context_env_without_leaking_stdio_transport_env() {
    let transport = AgentWrapperMcpAddTransport::Stdio {
        command: vec!["node".to_string()],
        args: vec!["server.js".to_string()],
        env: BTreeMap::from([("SERVER_ONLY".to_string(), "server-value".to_string())]),
    };
    let mut argv = vec![
        OsString::from("-c"),
        OsString::from(
            r#"printf "%s\n" "$@"
printf "CLI_ONLY=%s\n" "${CLI_ONLY-unset}" 1>&2
printf "SERVER_ONLY=%s\n" "${SERVER_ONLY-unset}" 1>&2
"#,
        ),
        OsString::from("codex-shim"),
    ];
    argv.extend(codex_mcp_add_argv("demo", &transport));
    let context = AgentWrapperMcpCommandContext {
        env: BTreeMap::from([
            ("CLI_ONLY".to_string(), "cli-value".to_string()),
            (PATH_ENV.to_string(), UNIX_TEST_PATH.to_string()),
        ]),
        ..Default::default()
    };

    let result = run_codex_mcp(
        super::super::super::CodexBackendConfig {
            binary: Some(PathBuf::from("/bin/sh")),
            ..Default::default()
        },
        argv,
        context,
    )
    .await
    .expect("runner should succeed");

    assert_eq!(
        result.stdout.lines().collect::<Vec<_>>(),
        vec![
            "mcp",
            "add",
            "demo",
            "--env",
            "SERVER_ONLY=server-value",
            "--",
            "node",
            "server.js",
        ]
    );
    assert_eq!(
        result.stderr.lines().collect::<Vec<_>>(),
        vec!["CLI_ONLY=cli-value", "SERVER_ONLY=unset"]
    );
}

#[cfg(unix)]
#[tokio::test]
#[allow(clippy::await_holding_lock)]
async fn run_codex_mcp_clears_ambient_env_before_spawn() {
    let _env_lock = test_env_lock();
    let temp_dir = temp_test_dir("ambient-env");
    let script_path = write_fake_codex(
        &temp_dir,
        r#"#!/bin/sh
printf "AMBIENT_ONLY=%s\n" "${AMBIENT_ONLY-unset}" 1>&2
printf "CODEX_HOME=%s\n" "${CODEX_HOME-unset}" 1>&2
printf "PATH=%s\n" "${PATH-unset}" 1>&2
"#,
    );
    let _ambient_only = EnvGuard::set("AMBIENT_ONLY", "ambient-value");
    let _ambient_home = EnvGuard::set("CODEX_HOME", "/tmp/ambient-codex-home");

    let result = run_codex_mcp(
        super::super::super::CodexBackendConfig {
            binary: Some(script_path),
            codex_home: Some(PathBuf::from("/tmp/resolved-codex-home")),
            ..Default::default()
        },
        codex_mcp_list_argv(),
        AgentWrapperMcpCommandContext {
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
            "CODEX_HOME=/tmp/resolved-codex-home".to_string(),
            format!("PATH={UNIX_TEST_PATH}"),
        ]
    );

    fs::remove_dir_all(temp_dir).expect("temp dir should be removed");
}

#[cfg(unix)]
#[tokio::test]
#[allow(clippy::await_holding_lock)]
async fn run_codex_mcp_preserves_path_for_launcher_script_helpers() {
    let _env_lock = test_env_lock();
    let temp_dir = temp_test_dir("path-helper");
    let helper_path = temp_dir.join("helper-bin");
    let script_path = temp_dir.join("codex");
    let effective_path = format!("{}:{UNIX_TEST_PATH}", temp_dir.to_string_lossy());

    write_test_executable(&helper_path, "#!/bin/sh\nprintf 'helper-ran\\n'");
    write_test_executable(&script_path, "#!/bin/sh\nhelper-bin\n");

    let result = run_codex_mcp(
        super::super::super::CodexBackendConfig {
            binary: Some(script_path),
            ..Default::default()
        },
        codex_mcp_list_argv(),
        AgentWrapperMcpCommandContext {
            working_dir: Some(temp_dir.clone()),
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
async fn capture_bounded_retains_only_bound_and_marks_overflow() {
    let (writer, reader) = tokio::io::duplex(64);
    let payload = b"abcdefghijklmnopqrstuvwxyz".to_vec();
    let writer_task = tokio::spawn(write_all_and_close(writer, payload));

    let (captured, saw_more) = capture_bounded(reader, 8).await.expect("capture succeeds");
    writer_task.await.expect("writer completes");

    assert_eq!(captured, b"abcdefgh");
    assert!(saw_more);
}

#[tokio::test]
async fn capture_bounded_preserves_small_streams() {
    let (writer, reader) = tokio::io::duplex(64);
    let payload = b"hello".to_vec();
    let writer_task = tokio::spawn(write_all_and_close(writer, payload));

    let (captured, saw_more) = capture_bounded(reader, 8).await.expect("capture succeeds");
    writer_task.await.expect("writer completes");

    assert_eq!(captured, b"hello");
    assert!(!saw_more);
}

#[test]
fn enforce_mcp_output_bound_stays_utf8_safe_after_lossy_decode() {
    let bytes = vec![0xf0, 0x9f, 0x92, 0x61, 0x62, 0x63];
    let (bounded, truncated) = enforce_mcp_output_bound(&bytes, true, 8);

    assert!(truncated);
    assert!(bounded.len() <= 8);
    assert!(std::str::from_utf8(bounded.as_bytes()).is_ok());
}

#[test]
fn classify_manifest_runtime_conflict_detects_unknown_mcp_subcommand() {
    assert!(classify_manifest_runtime_conflict_text(
        &codex_mcp_list_argv(),
        "error: unrecognized subcommand 'mcp'"
    ));
}

#[test]
fn classify_manifest_runtime_conflict_detects_unknown_list_subcommand() {
    assert!(classify_manifest_runtime_conflict_text(
        &codex_mcp_list_argv(),
        "error: unknown subcommand 'list'"
    ));
}

#[test]
fn classify_manifest_runtime_conflict_detects_unknown_get_subcommand() {
    assert!(classify_manifest_runtime_conflict_text(
        &codex_mcp_get_argv("demo"),
        "error: no such subcommand 'get'"
    ));
}

#[test]
fn classify_manifest_runtime_conflict_detects_unknown_remove_subcommand() {
    assert!(classify_manifest_runtime_conflict_text(
        &codex_mcp_remove_argv("demo"),
        "error: unrecognized subcommand 'remove'"
    ));
}

#[test]
fn classify_manifest_runtime_conflict_detects_unknown_add_subcommand() {
    let transport = AgentWrapperMcpAddTransport::Stdio {
        command: vec!["node".to_string()],
        args: vec!["server.js".to_string()],
        env: BTreeMap::new(),
    };

    assert!(classify_manifest_runtime_conflict_text(
        &codex_mcp_add_argv("demo", &transport),
        "error: unknown subcommand 'add'"
    ));
}

#[test]
fn classify_manifest_runtime_conflict_detects_unknown_json_flag() {
    assert!(classify_manifest_runtime_conflict_text(
        &codex_mcp_list_argv(),
        "error: unexpected argument '--json' found"
    ));
}

#[test]
fn classify_manifest_runtime_conflict_detects_legacy_add_usage_error() {
    let transport = AgentWrapperMcpAddTransport::Stdio {
        command: vec!["node".to_string()],
        args: vec!["server.js".to_string()],
        env: BTreeMap::new(),
    };

    assert!(classify_manifest_runtime_conflict_text(
        &codex_mcp_add_argv("demo", &transport),
        "error: unexpected argument '--env' found\n\nusage: codex mcp add <name> --url <url>"
    ));
}

#[test]
fn classify_manifest_runtime_conflict_detects_url_add_flag_drift_without_usage() {
    let transport = AgentWrapperMcpAddTransport::Url {
        url: "https://example.test/mcp".to_string(),
        bearer_token_env_var: None,
    };

    assert!(classify_manifest_runtime_conflict_text(
        &codex_mcp_add_argv("demo", &transport),
        "error: unexpected argument '--url' found"
    ));
}

#[test]
fn classify_manifest_runtime_conflict_detects_bearer_env_add_flag_drift_without_usage() {
    let transport = AgentWrapperMcpAddTransport::Url {
        url: "https://example.test/mcp".to_string(),
        bearer_token_env_var: Some("TOKEN_ENV".to_string()),
    };

    assert!(classify_manifest_runtime_conflict_text(
        &codex_mcp_add_argv("demo", &transport),
        "error: unexpected argument '--bearer-token-env-var' found"
    ));
}

#[test]
fn classify_manifest_runtime_conflict_ignores_normal_domain_failures() {
    assert!(!classify_manifest_runtime_conflict_text(
        &codex_mcp_get_argv("demo"),
        "server demo not found"
    ));
    assert!(!classify_manifest_runtime_conflict_text(
        &codex_mcp_get_argv("demo"),
        "unknown server demo"
    ));

    let transport = AgentWrapperMcpAddTransport::Stdio {
        command: vec!["node".to_string()],
        args: vec!["server.js".to_string()],
        env: BTreeMap::new(),
    };

    assert!(!classify_manifest_runtime_conflict_text(
        &codex_mcp_add_argv("demo", &transport),
        "error: unexpected argument '--foo' found"
    ));
}

fn assert_runtime_conflict(err: AgentWrapperError) {
    match err {
        AgentWrapperError::Backend { message } => {
            assert_eq!(message, PINNED_MCP_RUNTIME_CONFLICT);
        }
        other => panic!("expected Backend error, got {other:?}"),
    }
}

#[test]
fn codex_unknown_get_subcommand_drift_maps_to_pinned_backend_error() {
    let err = finalize_codex_mcp_output(
        &codex_mcp_get_argv("demo"),
        CapturedCodexMcpCommandOutput {
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
fn codex_add_flag_drift_maps_to_pinned_backend_error() {
    let transport = AgentWrapperMcpAddTransport::Url {
        url: "https://example.test/mcp".to_string(),
        bearer_token_env_var: Some("TOKEN_ENV".to_string()),
    };

    let err = finalize_codex_mcp_output(
        &codex_mcp_add_argv("demo", &transport),
        CapturedCodexMcpCommandOutput {
            status: exit_status_with_code(2),
            stdout_bytes: b"raw stdout should not leak".to_vec(),
            stdout_saw_more: false,
            stderr_bytes: b"error: unexpected argument '--url' found".to_vec(),
            stderr_saw_more: false,
        },
    )
    .expect_err("drift should fail closed");

    assert_runtime_conflict(err);
}

#[test]
fn codex_success_exit_skips_drift_classification() {
    let output = finalize_codex_mcp_output(
        &codex_mcp_get_argv("demo"),
        CapturedCodexMcpCommandOutput {
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
