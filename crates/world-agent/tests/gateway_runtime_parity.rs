#![cfg(target_os = "linux")]

use agent_api_types::{
    GatewayCliCodexIntegratedAuthV1, GatewayIntegratedAuthPayloadV1, GatewayLifecycleRequestV1,
    GatewayStatusV1, PolicySnapshotV3, PolicySnapshotWorldFsFailClosedV3, PolicySnapshotWorldFsV3,
    PolicySnapshotWorldFsWriteV3, WorldNetworkRoutingV1,
};
use once_cell::sync::Lazy;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};
use tempfile::TempDir;
use tokio::sync::Mutex;
use world_agent::WorldAgentService;

// These tests mutate process-global env and spawn async work that reads it, so
// the guard must stay alive across awaits to serialize the whole test body.
static ENV_LOCK: Lazy<Mutex<()>> = Lazy::new(|| Mutex::new(()));

fn minimal_policy_snapshot() -> PolicySnapshotV3 {
    PolicySnapshotV3 {
        schema_version: 3,
        net_allowed: Vec::new(),
        world_fs: PolicySnapshotWorldFsV3 {
            host_visible: true,
            fail_closed: PolicySnapshotWorldFsFailClosedV3 { routing: false },
            deny_enforcement: None,
            caged_required: false,
            discover: None,
            read: None,
            write: PolicySnapshotWorldFsWriteV3::default(),
        },
    }
}

fn gateway_request(cwd: &Path) -> GatewayLifecycleRequestV1 {
    GatewayLifecycleRequestV1 {
        profile: None,
        cwd: Some(cwd.display().to_string()),
        env: None,
        agent_id: "gateway-test".to_string(),
        policy_snapshot: minimal_policy_snapshot(),
        world_network: Some(WorldNetworkRoutingV1 {
            isolate_network: false,
            allowed_domains: Vec::new(),
        }),
        integrated_auth: Some(GatewayIntegratedAuthPayloadV1 {
            cli_codex: Some(GatewayCliCodexIntegratedAuthV1 {
                account_id: Some("acct_test".to_string()),
                access_token: "header.payload.signature".to_string(),
            }),
        }),
    }
}

fn gateway_request_with_backend(cwd: &Path, backend_id: &str) -> GatewayLifecycleRequestV1 {
    let mut request = gateway_request(cwd);
    let mut env = request.env.take().unwrap_or_default();
    env.insert(
        "SUBSTRATE_LLM_DEFAULT_BACKEND".to_string(),
        backend_id.to_string(),
    );
    request.env = Some(env);
    request
}

fn service_or_skip() -> Option<WorldAgentService> {
    match WorldAgentService::new() {
        Ok(service) => Some(service),
        Err(err) => {
            eprintln!("skipping gateway runtime parity test: service init failed: {err}");
            None
        }
    }
}

fn fake_gateway_binary(temp_dir: &TempDir) -> PathBuf {
    let path = temp_dir.path().join("fake-gateway.sh");
    fs::write(
        &path,
        r#"#!/bin/sh
set -eu
config=""
while [ "$#" -gt 0 ]; do
  case "$1" in
    start)
      shift
      ;;
    --config)
      config="$2"
      shift 2
      ;;
    *)
      shift
      ;;
  esac
done

if [ -z "$config" ]; then
  echo "missing --config" >&2
  exit 64
fi

if [ -z "${SUBSTRATE_LLM_BACKEND_AUTH_CLI_CODEX_ACCESS_TOKEN:-}" ]; then
  echo "missing Codex access token env" >&2
  exit 65
fi

port="$(python3 - "$config" <<'PY'
import re
import sys
text = open(sys.argv[1], 'r', encoding='utf-8').read()
match = re.search(r'^port\s*=\s*(\d+)\s*$', text, re.M)
if not match:
    raise SystemExit(64)
print(match.group(1))
PY
)"

root="$(dirname "$config")/serve"
mkdir -p "$root"
printf 'ok' >"$root/health"
exec python3 -m http.server "$port" --bind 127.0.0.1 --directory "$root"
"#,
    )
    .unwrap();
    let mut perms = fs::metadata(&path).unwrap().permissions();
    perms.set_mode(0o755);
    fs::set_permissions(&path, perms).unwrap();
    path
}

fn strict_gateway_binary(temp_dir: &TempDir, name: &str) -> PathBuf {
    let server_path = temp_dir.path().join(format!("{name}-strict-health.py"));
    fs::write(
        &server_path,
        r#"#!/usr/bin/env python3
import socket
import sys

port = int(sys.argv[1])
listener = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
listener.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
listener.bind(("127.0.0.1", port))
listener.listen(32)

while True:
    conn, _ = listener.accept()
    with conn:
        conn.settimeout(0.15)
        request = b""
        while b"\r\n\r\n" not in request:
            chunk = conn.recv(4096)
            if not chunk:
                break
            request += chunk
        if b"\r\n\r\n" not in request:
            continue
        try:
            probe = conn.recv(1)
            if probe == b"":
                continue
        except socket.timeout:
            pass
        response = (
            b"HTTP/1.1 200 OK\r\n"
            b"Content-Type: application/json\r\n"
            b"Content-Length: 45\r\n"
            b"Connection: close\r\n"
            b"\r\n"
            b"{\"status\":\"ok\",\"service\":\"substrate-gateway\"}"
        )
        conn.sendall(response)
"#,
    )
    .unwrap();

    let path = temp_dir.path().join(format!("{name}-strict-gateway.sh"));
    fs::write(
        &path,
        format!(
            r#"#!/bin/sh
set -eu
config=""
while [ "$#" -gt 0 ]; do
  case "$1" in
    start)
      shift
      ;;
    --config)
      config="$2"
      shift 2
      ;;
    *)
      shift
      ;;
  esac
done

if [ -z "$config" ]; then
  echo "missing --config" >&2
  exit 64
fi

if [ -z "${{SUBSTRATE_LLM_BACKEND_AUTH_CLI_CODEX_ACCESS_TOKEN:-}}" ]; then
  echo "missing Codex access token env" >&2
  exit 65
fi

port="$(python3 - "$config" <<'PY'
import re
import sys
text = open(sys.argv[1], 'r', encoding='utf-8').read()
match = re.search(r'^port\s*=\s*(\d+)\s*$', text, re.M)
if not match:
    raise SystemExit(64)
print(match.group(1))
PY
)"

exec python3 "{server_path}" "$port"
"#,
            server_path = server_path.display(),
        ),
    )
    .unwrap();
    let mut perms = fs::metadata(&path).unwrap().permissions();
    perms.set_mode(0o755);
    fs::set_permissions(&path, perms).unwrap();
    path
}

fn delayed_gateway_binary(temp_dir: &TempDir, name: &str, delay_ms: u64) -> (PathBuf, PathBuf) {
    let path = temp_dir.path().join(format!("{name}-gateway.sh"));
    let pid_path = temp_dir.path().join(format!("{name}.pid"));
    fs::write(
        &path,
        format!(
            r#"#!/bin/sh
set -eu
config=""
while [ "$#" -gt 0 ]; do
  case "$1" in
    start)
      shift
      ;;
    --config)
      config="$2"
      shift 2
      ;;
    *)
      shift
      ;;
  esac
done

if [ -z "$config" ]; then
  echo "missing --config" >&2
  exit 64
fi

if [ -z "${{SUBSTRATE_LLM_BACKEND_AUTH_CLI_CODEX_ACCESS_TOKEN:-}}" ]; then
  echo "missing Codex access token env" >&2
  exit 65
fi

printf '%s\n' "$$" >"{pid_path}"
port="$(python3 - "$config" <<'PY'
import re
import sys
text = open(sys.argv[1], 'r', encoding='utf-8').read()
match = re.search(r'^port\s*=\s*(\d+)\s*$', text, re.M)
if not match:
    raise SystemExit(64)
print(match.group(1))
PY
)"

sleep {delay_s:.3}
root="$(dirname "$config")/serve"
mkdir -p "$root"
printf 'ok' >"$root/health"
exec python3 -m http.server "$port" --bind 127.0.0.1 --directory "$root"
"#,
            pid_path = pid_path.display(),
            delay_s = delay_ms as f64 / 1000.0,
        ),
    )
    .unwrap();
    let mut perms = fs::metadata(&path).unwrap().permissions();
    perms.set_mode(0o755);
    fs::set_permissions(&path, perms).unwrap();
    (path, pid_path)
}

fn tracking_gateway_binary(
    temp_dir: &TempDir,
    name: &str,
    delay_ms: u64,
) -> (PathBuf, PathBuf, PathBuf) {
    let path = temp_dir.path().join(format!("{name}-tracking-gateway.sh"));
    let pid_dir = temp_dir.path().join(format!("{name}-pids"));
    let launch_count_path = temp_dir.path().join(format!("{name}-launch-count.txt"));
    fs::write(
        &path,
        format!(
            r#"#!/bin/sh
set -eu
config=""
while [ "$#" -gt 0 ]; do
  case "$1" in
    start)
      shift
      ;;
    --config)
      config="$2"
      shift 2
      ;;
    *)
      shift
      ;;
  esac
done

if [ -z "$config" ]; then
  echo "missing --config" >&2
  exit 64
fi

if [ -z "${{SUBSTRATE_LLM_BACKEND_AUTH_CLI_CODEX_ACCESS_TOKEN:-}}" ]; then
  echo "missing Codex access token env" >&2
  exit 65
fi

launch="$(python3 - "{launch_count_path}" <<'PY'
import pathlib
import sys
path = pathlib.Path(sys.argv[1])
count = int(path.read_text(encoding='utf-8').strip()) if path.exists() else 0
count += 1
path.write_text(str(count), encoding='utf-8')
print(count)
PY
)"
mkdir -p "{pid_dir}"
printf '%s\n' "$$" >"{pid_dir}/$launch.pid"

port="$(python3 - "$config" <<'PY'
import re
import sys
text = open(sys.argv[1], 'r', encoding='utf-8').read()
match = re.search(r'^port\s*=\s*(\d+)\s*$', text, re.M)
if not match:
    raise SystemExit(64)
print(match.group(1))
PY
)"

sleep {delay_s:.3}
root="$(dirname "$config")/serve"
mkdir -p "$root"
printf 'ok' >"$root/health"
exec python3 -m http.server "$port" --bind 127.0.0.1 --directory "$root"
"#,
            launch_count_path = launch_count_path.display(),
            pid_dir = pid_dir.display(),
            delay_s = delay_ms as f64 / 1000.0,
        ),
    )
    .unwrap();
    let mut perms = fs::metadata(&path).unwrap().permissions();
    perms.set_mode(0o755);
    fs::set_permissions(&path, perms).unwrap();
    (path, pid_dir, launch_count_path)
}

fn crashing_gateway_binary(temp_dir: &TempDir) -> PathBuf {
    let path = temp_dir.path().join("crash-gateway.sh");
    fs::write(&path, "#!/bin/sh\nexit 17\n").unwrap();
    let mut perms = fs::metadata(&path).unwrap().permissions();
    perms.set_mode(0o755);
    fs::set_permissions(&path, perms).unwrap();
    path
}

fn hanging_gateway_binary(temp_dir: &TempDir) -> (PathBuf, PathBuf) {
    let path = temp_dir.path().join("hang-gateway.sh");
    let pid_path = temp_dir.path().join("hang-gateway.pid");
    fs::write(
        &path,
        format!(
            r#"#!/bin/sh
set -eu
while [ "$#" -gt 0 ]; do
  case "$1" in
    start)
      shift
      ;;
    --config)
      shift 2
      ;;
    *)
      shift
      ;;
  esac
done
if [ -z "${{SUBSTRATE_LLM_BACKEND_AUTH_CLI_CODEX_ACCESS_TOKEN:-}}" ]; then
  echo "missing Codex access token env" >&2
  exit 65
fi
printf '%s\n' "$$" >"{pid_path}"
sleep 30
"#,
            pid_path = pid_path.display(),
        ),
    )
    .unwrap();
    let mut perms = fs::metadata(&path).unwrap().permissions();
    perms.set_mode(0o755);
    fs::set_permissions(&path, perms).unwrap();
    (path, pid_path)
}

fn wait_for_pid_file(pid_path: &Path) -> u32 {
    let deadline = Instant::now() + Duration::from_secs(2);
    loop {
        if let Ok(raw) = fs::read_to_string(pid_path) {
            let pid = raw.trim().parse::<u32>().expect("parse pid");
            if pid > 0 {
                return pid;
            }
        }
        assert!(
            Instant::now() < deadline,
            "timed out waiting for pid file {}",
            pid_path.display()
        );
        std::thread::sleep(Duration::from_millis(25));
    }
}

async fn wait_for_pid_file_async(pid_path: &Path) -> u32 {
    let deadline = Instant::now() + Duration::from_secs(2);
    loop {
        if let Ok(raw) = fs::read_to_string(pid_path) {
            let pid = raw.trim().parse::<u32>().expect("parse pid");
            if pid > 0 {
                return pid;
            }
        }
        assert!(
            Instant::now() < deadline,
            "timed out waiting for pid file {}",
            pid_path.display()
        );
        tokio::time::sleep(Duration::from_millis(25)).await;
    }
}

fn wait_for_pid(pid_dir: &Path, launch: u32) -> u32 {
    wait_for_pid_file(&pid_dir.join(format!("{launch}.pid")))
}

fn read_launch_count(path: &Path) -> u32 {
    fs::read_to_string(path)
        .ok()
        .and_then(|raw| raw.trim().parse::<u32>().ok())
        .unwrap_or(0)
}

fn assert_process_exited(pid: u32) {
    let rc = unsafe { libc::kill(pid as i32, 0) };
    assert_eq!(rc, -1, "expected pid {pid} to be gone");
    assert_eq!(
        std::io::Error::last_os_error().raw_os_error(),
        Some(libc::ESRCH),
        "pid {pid} should be gone",
    );
}

struct EnvGuard {
    key: &'static str,
    previous: Option<std::ffi::OsString>,
}

impl EnvGuard {
    fn set(key: &'static str, value: impl Into<std::ffi::OsString>) -> Self {
        let previous = std::env::var_os(key);
        std::env::set_var(key, value.into());
        Self { key, previous }
    }
}

impl Drop for EnvGuard {
    fn drop(&mut self) {
        if let Some(previous) = &self.previous {
            std::env::set_var(self.key, previous);
        } else {
            std::env::remove_var(self.key);
        }
    }
}

#[tokio::test]
async fn gateway_status_returns_unavailable_before_sync() {
    let _env_lock = ENV_LOCK.lock().await;
    let temp_dir = TempDir::new().unwrap();
    let binary = fake_gateway_binary(&temp_dir);
    let _binary_guard = EnvGuard::set("SUBSTRATE_GATEWAY_BINARY", binary);
    let Some(service) = service_or_skip() else {
        return;
    };
    let request = gateway_request(temp_dir.path());

    let response = service
        .gateway_status(request)
        .await
        .expect("gateway status");

    assert_eq!(response.status, GatewayStatusV1::Unavailable);
    assert!(response.client_wiring.is_none());
}

#[tokio::test]
async fn missing_backend_binding_returns_unavailable_for_lifecycle_actions() {
    let _env_lock = ENV_LOCK.lock().await;
    let temp_dir = TempDir::new().unwrap();
    let binary = fake_gateway_binary(&temp_dir);
    let _binary_guard = EnvGuard::set("SUBSTRATE_GATEWAY_BINARY", binary);
    let Some(service) = service_or_skip() else {
        return;
    };
    let request = gateway_request_with_backend(temp_dir.path(), "api:openai");

    let status = service
        .gateway_status(request.clone())
        .await
        .expect("gateway status");
    assert_eq!(status.status, GatewayStatusV1::Unavailable);

    let sync = service
        .gateway_sync(request.clone())
        .await
        .expect("gateway sync");
    assert_eq!(sync.status, GatewayStatusV1::Unavailable);

    let restart = service
        .gateway_restart(request)
        .await
        .expect("gateway restart");
    assert_eq!(restart.status, GatewayStatusV1::Unavailable);
}

#[tokio::test]
async fn gateway_sync_makes_status_available_and_is_idempotent() {
    let _env_lock = ENV_LOCK.lock().await;
    let temp_dir = TempDir::new().unwrap();
    let (binary, pid_dir, launch_count_path) = tracking_gateway_binary(&temp_dir, "idempotent", 0);
    let _binary_guard = EnvGuard::set("SUBSTRATE_GATEWAY_BINARY", binary);
    let Some(service) = service_or_skip() else {
        return;
    };
    let request = gateway_request(temp_dir.path());

    let sync_response = service
        .gateway_sync(request.clone())
        .await
        .expect("gateway sync");
    assert_eq!(sync_response.status, GatewayStatusV1::Available);
    let wiring = sync_response
        .client_wiring
        .expect("available sync should publish client wiring");
    assert!(wiring.openai_base_url.starts_with("http://127.0.0.1:"));
    assert_eq!(wiring.openai_base_url, wiring.anthropic_base_url);
    let first_pid = wait_for_pid(&pid_dir, 1);

    let status_response = service
        .gateway_status(request.clone())
        .await
        .expect("gateway status after sync");
    assert_eq!(status_response.status, GatewayStatusV1::Available);
    assert!(status_response.client_wiring.is_some());

    let second_sync = service
        .gateway_sync(request.clone())
        .await
        .expect("idempotent gateway sync");
    assert_eq!(second_sync.status, GatewayStatusV1::Available);
    let second_wiring = second_sync
        .client_wiring
        .expect("available sync should publish client wiring");
    assert_eq!(
        wiring.openai_base_url, second_wiring.openai_base_url,
        "sync should reuse the running gateway"
    );
    assert_eq!(read_launch_count(&launch_count_path), 1);
    assert_eq!(first_pid, wait_for_pid(&pid_dir, 1));
}

#[tokio::test]
async fn gateway_sync_succeeds_against_strict_health_server() {
    let _env_lock = ENV_LOCK.lock().await;
    let temp_dir = TempDir::new().unwrap();
    let binary = strict_gateway_binary(&temp_dir, "strict-ready");
    let _binary_guard = EnvGuard::set("SUBSTRATE_GATEWAY_BINARY", binary);
    let Some(service) = service_or_skip() else {
        return;
    };
    let request = gateway_request(temp_dir.path());

    let sync_response = service
        .gateway_sync(request.clone())
        .await
        .expect("gateway sync against strict health server");
    assert_eq!(sync_response.status, GatewayStatusV1::Available);
    assert!(
        sync_response.client_wiring.is_some(),
        "available sync should publish client wiring"
    );

    let status_response = service
        .gateway_status(request)
        .await
        .expect("gateway status after strict health sync");
    assert_eq!(status_response.status, GatewayStatusV1::Available);
}

#[tokio::test]
async fn gateway_restart_recycles_the_runtime() {
    let _env_lock = ENV_LOCK.lock().await;
    let temp_dir = TempDir::new().unwrap();
    let (binary, pid_dir, launch_count_path) = tracking_gateway_binary(&temp_dir, "restart", 0);
    let _binary_guard = EnvGuard::set("SUBSTRATE_GATEWAY_BINARY", binary);
    let Some(service) = service_or_skip() else {
        return;
    };
    let request = gateway_request(temp_dir.path());

    service
        .gateway_sync(request.clone())
        .await
        .expect("initial gateway sync");
    let initial_pid = wait_for_pid(&pid_dir, 1);

    let restart_response = service
        .gateway_restart(request.clone())
        .await
        .expect("gateway restart");
    assert_eq!(restart_response.status, GatewayStatusV1::Available);

    let restarted_pid = wait_for_pid(&pid_dir, 2);
    assert_ne!(
        initial_pid, restarted_pid,
        "restart should spawn a new process"
    );
    assert_eq!(read_launch_count(&launch_count_path), 2);
    assert_process_exited(initial_pid);
}

#[tokio::test]
async fn gateway_manifest_recovery_restores_status_sync_and_restart() {
    let _env_lock = ENV_LOCK.lock().await;
    let temp_dir = TempDir::new().unwrap();
    let (binary, pid_dir, launch_count_path) = tracking_gateway_binary(&temp_dir, "recovery", 0);
    let _binary_guard = EnvGuard::set("SUBSTRATE_GATEWAY_BINARY", binary);
    let Some(service) = service_or_skip() else {
        return;
    };
    let request = gateway_request(temp_dir.path());

    service
        .gateway_sync(request.clone())
        .await
        .expect("initial gateway sync");
    let initial_pid = wait_for_pid(&pid_dir, 1);
    drop(service);

    let Some(service) = service_or_skip() else {
        return;
    };

    let status_response = service
        .gateway_status(request.clone())
        .await
        .expect("status via recovered manifest");
    assert_eq!(status_response.status, GatewayStatusV1::Available);
    assert_eq!(read_launch_count(&launch_count_path), 1);

    let sync_response = service
        .gateway_sync(request.clone())
        .await
        .expect("sync via recovered manifest");
    assert_eq!(sync_response.status, GatewayStatusV1::Available);
    assert_eq!(read_launch_count(&launch_count_path), 1);

    let restart_response = service
        .gateway_restart(request.clone())
        .await
        .expect("restart via recovered manifest");
    assert_eq!(restart_response.status, GatewayStatusV1::Available);
    let restarted_pid = wait_for_pid(&pid_dir, 2);
    assert_ne!(
        restarted_pid, initial_pid,
        "restart should stop the recovered pid and replace it"
    );
    assert_eq!(read_launch_count(&launch_count_path), 2);
    assert_process_exited(initial_pid);
}

#[tokio::test]
async fn gateway_status_turns_unavailable_after_child_exit() {
    let _env_lock = ENV_LOCK.lock().await;
    let temp_dir = TempDir::new().unwrap();
    let (binary, pid_dir, _) = tracking_gateway_binary(&temp_dir, "child-exit", 0);
    let _binary_guard = EnvGuard::set("SUBSTRATE_GATEWAY_BINARY", binary);
    let Some(service) = service_or_skip() else {
        return;
    };
    let request = gateway_request(temp_dir.path());

    service
        .gateway_sync(request.clone())
        .await
        .expect("initial gateway sync");
    let pid = wait_for_pid(&pid_dir, 1);

    let kill_status = unsafe { libc::kill(pid as i32, libc::SIGKILL) };
    assert_eq!(kill_status, 0, "failed to kill fake gateway child");
    tokio::time::sleep(std::time::Duration::from_millis(250)).await;

    let status_response = service
        .gateway_status(request)
        .await
        .expect("status after child exit");
    assert_eq!(status_response.status, GatewayStatusV1::Unavailable);
    assert!(status_response.client_wiring.is_none());
}

#[tokio::test]
async fn gateway_sync_reports_transient_failure_when_startup_crashes() {
    let _env_lock = ENV_LOCK.lock().await;
    let temp_dir = TempDir::new().unwrap();
    let binary = crashing_gateway_binary(&temp_dir);
    let _binary_guard = EnvGuard::set("SUBSTRATE_GATEWAY_BINARY", binary);
    let Some(service) = service_or_skip() else {
        return;
    };
    let request = gateway_request(temp_dir.path());

    let err = service
        .gateway_sync(request)
        .await
        .expect_err("crashing gateway should surface a transient failure");
    assert!(
        err.to_string().contains("gateway_transient_failure:"),
        "unexpected error: {err:#}"
    );
}

#[tokio::test]
async fn gateway_sync_kills_child_after_ready_timeout() {
    let _env_lock = ENV_LOCK.lock().await;
    let temp_dir = TempDir::new().unwrap();
    let (binary, pid_path) = hanging_gateway_binary(&temp_dir);
    let _binary_guard = EnvGuard::set("SUBSTRATE_GATEWAY_BINARY", binary);
    let Some(service) = service_or_skip() else {
        return;
    };
    let request = gateway_request(temp_dir.path());

    let err = service
        .gateway_sync(request.clone())
        .await
        .expect_err("hung gateway should time out");
    assert!(
        err.to_string()
            .contains("gateway did not become ready before timeout"),
        "unexpected error: {err:#}"
    );

    let pid = wait_for_pid_file(&pid_path);
    tokio::time::sleep(Duration::from_millis(200)).await;
    assert_process_exited(pid);

    let status_response = service
        .gateway_status(request)
        .await
        .expect("status after ready-timeout cleanup");
    assert_eq!(status_response.status, GatewayStatusV1::Unavailable);
    assert!(status_response.client_wiring.is_none());
}

#[tokio::test]
async fn gateway_status_reports_transient_failure_while_starting() {
    let _env_lock = ENV_LOCK.lock().await;
    let temp_dir = TempDir::new().unwrap();
    let (binary, pid_path) = delayed_gateway_binary(&temp_dir, "starting", 1000);
    let _binary_guard = EnvGuard::set("SUBSTRATE_GATEWAY_BINARY", binary);
    let Some(service) = service_or_skip() else {
        return;
    };
    let request = gateway_request(temp_dir.path());

    let sync_task = tokio::spawn({
        let service = service.clone();
        let request = request.clone();
        async move { service.gateway_sync(request).await }
    });
    wait_for_pid_file_async(&pid_path).await;

    let err = service
        .gateway_status(request.clone())
        .await
        .expect_err("starting status should be transient");
    assert!(
        err.to_string().contains("gateway_transient_failure:"),
        "unexpected error: {err:#}"
    );
    assert!(
        err.to_string().contains("starting"),
        "unexpected error: {err:#}"
    );

    let sync_response = sync_task.await.unwrap().expect("sync should finish");
    assert_eq!(sync_response.status, GatewayStatusV1::Available);
}

#[tokio::test]
async fn gateway_status_reports_transient_failure_while_restarting() {
    let _env_lock = ENV_LOCK.lock().await;
    let temp_dir = TempDir::new().unwrap();
    let binary = fake_gateway_binary(&temp_dir);
    let _binary_guard = EnvGuard::set("SUBSTRATE_GATEWAY_BINARY", binary);
    let Some(service) = service_or_skip() else {
        return;
    };
    let request = gateway_request(temp_dir.path());

    service
        .gateway_sync(request.clone())
        .await
        .expect("initial gateway sync");

    let (restart_binary, pid_path) = delayed_gateway_binary(&temp_dir, "restarting", 1000);
    let _restart_binary_guard = EnvGuard::set("SUBSTRATE_GATEWAY_BINARY", restart_binary);
    let restart_task = tokio::spawn({
        let service = service.clone();
        let request = request.clone();
        async move { service.gateway_restart(request).await }
    });
    wait_for_pid_file_async(&pid_path).await;

    let err = service
        .gateway_status(request.clone())
        .await
        .expect_err("restart status should be transient");
    assert!(
        err.to_string().contains("gateway_transient_failure:"),
        "unexpected error: {err:#}"
    );
    assert!(
        err.to_string().contains("restarting"),
        "unexpected error: {err:#}"
    );

    let restart_response = restart_task.await.unwrap().expect("restart should finish");
    assert_eq!(restart_response.status, GatewayStatusV1::Available);
}
