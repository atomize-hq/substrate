#![cfg(target_os = "linux")]

mod support;

use support::{substrate_command_for_home, AgentSocket, ShellEnvFixture, SocketResponse};

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use tempfile::Builder;

fn write_executable(path: &Path, contents: &str) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("create executable parent dir");
    }
    fs::write(path, contents).expect("write executable");
    let mut perms = fs::metadata(path).expect("stat executable").permissions();
    perms.set_mode(0o755);
    fs::set_permissions(path, perms).expect("chmod executable");
}

#[test]
fn explicit_host_toolchain_path_is_denied_with_exit_5_when_guard_enabled() {
    let fixture = ShellEnvFixture::new();

    let blocked_bin = fixture.home().join(".cargo/bin/wdh2-blocked");
    write_executable(&blocked_bin, "#!/bin/sh\necho wdh2-should-not-run\n");
    let blocked_cmd = blocked_bin.display().to_string();

    // Keep the Unix socket path short to avoid `SUN_LEN` failures.
    let sock_tmp = Builder::new()
        .prefix("substrate-wdh2-exec-guard-")
        .tempdir_in("/tmp")
        .expect("socket tempdir");
    let socket_path = sock_tmp.path().join("world.sock");
    let socket = AgentSocket::start(
        &socket_path,
        SocketResponse::CapabilitiesAndExecute {
            stdout: "".to_string(),
            stderr: "".to_string(),
            exit: 0,
            scopes: vec![],
        },
    );

    substrate_command_for_home(&fixture)
        .env("SUBSTRATE_WORLD_SOCKET", &socket_path)
        .env("SUBSTRATE_SOCKET_ACTIVATION_OVERRIDE", "manual")
        .env("SUBSTRATE_OVERRIDE_POLICY_MODE", "enforce")
        .env("SUBSTRATE_OVERRIDE_WORLD_EXEC_GUARD", "1")
        .args(["--world", "-c"])
        .arg(&blocked_cmd)
        .assert()
        .code(5);

    assert_eq!(
        socket.execute_request_count(),
        0,
        "expected exec guard to deny before issuing /v1/execute; got {} execute requests",
        socket.execute_request_count()
    );
}
