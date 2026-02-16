#![cfg(target_os = "linux")]

mod support;

use support::{substrate_command_for_home, AgentSocket, ReplWorldAgentStub, ShellEnvFixture};
use support::{SocketResponse, StreamBehavior};

use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tempfile::Builder;

const BASELINE_PATH: &str =
    "/var/lib/substrate/world-deps/bin:/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin";

fn hostile_host_path() -> String {
    "/home/test/.config/nvm/versions/node/v20.0.0/bin:/home/test/.pyenv/shims:/home/test/.cargo/bin:/home/test/.local/bin:/usr/bin:/bin".to_string()
}

fn assert_path_is_sanitized(path: &str) {
    assert_eq!(path, BASELINE_PATH, "PATH mismatch (expected baseline)");
    for banned in ["/.config/nvm/", "/.pyenv/", "/.cargo/bin", "/.local/bin"] {
        assert!(
            !path.contains(banned),
            "PATH must not include host toolchain segment {banned:?}; got PATH={path:?}"
        );
    }
    assert!(
        path.starts_with("/var/lib/substrate/world-deps/bin:"),
        "PATH must be prefixed with world-deps bin; got PATH={path:?}"
    );
}

fn start_world_socket_execute_record(
    prefix: &str,
) -> (
    tempfile::TempDir,
    PathBuf,
    AgentSocket,
    Arc<Mutex<Vec<serde_json::Value>>>,
) {
    // Keep the Unix socket path short to avoid `SUN_LEN` failures.
    let sock_tmp = Builder::new()
        .prefix(prefix)
        .tempdir_in("/tmp")
        .expect("socket tempdir");
    let socket_path = sock_tmp.path().join("world.sock");

    let records: Arc<Mutex<Vec<serde_json::Value>>> = Arc::new(Mutex::new(Vec::new()));
    let socket = AgentSocket::start(
        &socket_path,
        SocketResponse::CapabilitiesAndExecuteRecord {
            stdout: "".to_string(),
            stderr: "".to_string(),
            exit: 0,
            scopes: vec![],
            records: records.clone(),
        },
    );

    (sock_tmp, socket_path, socket, records)
}

fn last_env(
    records: &Arc<Mutex<Vec<serde_json::Value>>>,
) -> serde_json::Map<String, serde_json::Value> {
    let guard = records.lock().expect("lock records");
    let last = guard
        .last()
        .cloned()
        .expect("expected at least one execute request recorded");
    last.get("env")
        .and_then(|v| v.as_object())
        .cloned()
        .expect("expected recorded execute request to include env map")
}

#[test]
fn test_non_pty_request_builder_sanitizes_world_path() {
    let fixture = ShellEnvFixture::new();
    let (_sock_tmp, socket_path, _socket, records) =
        start_world_socket_execute_record("substrate-wdh0-nonpty-path-");

    substrate_command_for_home(&fixture)
        .env("SUBSTRATE_WORLD_SOCKET", &socket_path)
        .env("SUBSTRATE_SOCKET_ACTIVATION_OVERRIDE", "manual")
        .env("PATH", hostile_host_path())
        .args(["--world", "-c", ":"])
        .assert()
        .success();

    let env = last_env(&records);
    assert_eq!(
        env.get("SUBSTRATE_WORLD_DEPS_GUEST_BIN_DIR")
            .and_then(|v| v.as_str()),
        Some("/var/lib/substrate/world-deps/bin"),
        "missing or unexpected SUBSTRATE_WORLD_DEPS_GUEST_BIN_DIR"
    );
    let path = env.get("PATH").and_then(|v| v.as_str()).unwrap_or_default();
    assert_path_is_sanitized(path);
}

#[test]
fn test_pty_request_builder_sanitizes_world_path() {
    let fixture = ShellEnvFixture::new();

    // Keep the Unix socket path short to avoid `SUN_LEN` failures.
    let sock_tmp = Builder::new()
        .prefix("substrate-wdh0-pty-path-")
        .tempdir_in("/tmp")
        .expect("socket tempdir");
    let socket_path = sock_tmp.path().join("world.sock");

    let stub = ReplWorldAgentStub::start(&socket_path, StreamBehavior::Normal);

    substrate_command_for_home(&fixture)
        .env("SUBSTRATE_WORLD_SOCKET", stub.path())
        .env("SUBSTRATE_SOCKET_ACTIVATION_OVERRIDE", "manual")
        .env("PATH", hostile_host_path())
        .args(["--world", "-c", ":pty :"])
        .assert()
        .success();

    let records = stub.records();
    let guard = records.lock().expect("lock records");
    assert!(
        !guard.legacy_pty_starts.is_empty(),
        "expected at least one legacy PTY start frame recorded; got {guard:#?}"
    );
    let env = &guard.legacy_pty_starts[0].env;
    assert_eq!(
        env.get("SUBSTRATE_WORLD_DEPS_GUEST_BIN_DIR")
            .map(String::as_str),
        Some("/var/lib/substrate/world-deps/bin"),
        "missing or unexpected SUBSTRATE_WORLD_DEPS_GUEST_BIN_DIR"
    );
    let path = env.get("PATH").map(String::as_str).unwrap_or_default();
    assert_path_is_sanitized(path);
}
