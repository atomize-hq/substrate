#![cfg(unix)]

mod support;

use support::{substrate_command_for_home, AgentSocket, ShellEnvFixture, SocketResponse};

use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tempfile::Builder;

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
fn test_world_execute_injects_world_deps_bin_dir_and_prepends_path() {
    let fixture = ShellEnvFixture::new();
    let (_sock_tmp, socket_path, _socket, records) =
        start_world_socket_execute_record("substrate-world-deps-path-");

    substrate_command_for_home(&fixture)
        .env("SUBSTRATE_WORLD_SOCKET", &socket_path)
        .env("SUBSTRATE_SOCKET_ACTIVATION_OVERRIDE", "manual")
        .env("PATH", "/usr/bin:/bin")
        .args(["--world", "-c", ":"])
        .assert()
        .success();

    let env = last_env(&records);
    assert_eq!(
        env.get("SUBSTRATE_WORLD_DEPS_GUEST_BIN_DIR")
            .and_then(|v| v.as_str()),
        Some("/var/lib/substrate/world-deps/bin")
    );
    let path = env.get("PATH").and_then(|v| v.as_str()).unwrap_or_default();
    assert!(
        path.starts_with("/var/lib/substrate/world-deps/bin:"),
        "expected PATH to be prepended with world-deps bin; got PATH={path:?}"
    );
}

#[test]
fn test_world_execute_respects_world_deps_bin_override() {
    let fixture = ShellEnvFixture::new();
    let (_sock_tmp, socket_path, _socket, records) =
        start_world_socket_execute_record("substrate-world-deps-path-override-");

    substrate_command_for_home(&fixture)
        .env("SUBSTRATE_WORLD_SOCKET", &socket_path)
        .env("SUBSTRATE_SOCKET_ACTIVATION_OVERRIDE", "manual")
        .env("SUBSTRATE_WORLD_DEPS_GUEST_BIN_DIR", "/tmp/custom-wdp-bin")
        .env("PATH", "/usr/bin:/bin")
        .args(["--world", "-c", ":"])
        .assert()
        .success();

    let env = last_env(&records);
    assert_eq!(
        env.get("SUBSTRATE_WORLD_DEPS_GUEST_BIN_DIR")
            .and_then(|v| v.as_str()),
        Some("/tmp/custom-wdp-bin")
    );
    let path = env.get("PATH").and_then(|v| v.as_str()).unwrap_or_default();
    assert!(
        path.starts_with("/tmp/custom-wdp-bin:"),
        "expected PATH to be prepended with override world-deps bin; got PATH={path:?}"
    );
}
