#![cfg(target_os = "linux")]

use agent_api_types::{
    GatewayLifecycleRequestV1, GatewayStatusV1, PolicySnapshotV3,
    PolicySnapshotWorldFsFailClosedV3, PolicySnapshotWorldFsV3, PolicySnapshotWorldFsWriteV3,
    WorldNetworkRoutingV1,
};
use once_cell::sync::Lazy;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use tempfile::TempDir;
use world_agent::WorldAgentService;

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
    }
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

fn crashing_gateway_binary(temp_dir: &TempDir) -> PathBuf {
    let path = temp_dir.path().join("crash-gateway.sh");
    fs::write(&path, "#!/bin/sh\nexit 17\n").unwrap();
    let mut perms = fs::metadata(&path).unwrap().permissions();
    perms.set_mode(0o755);
    fs::set_permissions(&path, perms).unwrap();
    path
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
    let _env_lock = ENV_LOCK.lock().unwrap();
    let temp_dir = TempDir::new().unwrap();
    let binary = fake_gateway_binary(&temp_dir);
    let _binary_guard = EnvGuard::set("SUBSTRATE_GATEWAY_BINARY", binary);
    let _account_guard = EnvGuard::set(
        "SUBSTRATE_LLM_BACKEND_AUTH_CLI_CODEX_ACCOUNT_ID",
        "acct_test",
    );
    let _token_guard = EnvGuard::set(
        "SUBSTRATE_LLM_BACKEND_AUTH_CLI_CODEX_ACCESS_TOKEN",
        "header.payload.signature",
    );
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
async fn gateway_sync_makes_status_available_and_is_idempotent() {
    let _env_lock = ENV_LOCK.lock().unwrap();
    let temp_dir = TempDir::new().unwrap();
    let binary = fake_gateway_binary(&temp_dir);
    let _binary_guard = EnvGuard::set("SUBSTRATE_GATEWAY_BINARY", binary);
    let _account_guard = EnvGuard::set(
        "SUBSTRATE_LLM_BACKEND_AUTH_CLI_CODEX_ACCOUNT_ID",
        "acct_test",
    );
    let _token_guard = EnvGuard::set(
        "SUBSTRATE_LLM_BACKEND_AUTH_CLI_CODEX_ACCESS_TOKEN",
        "header.payload.signature",
    );
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
    let first_pid = service
        .gateway_runtime_pid_for_test(&request)
        .expect("inspect gateway pid")
        .expect("gateway pid after sync");

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
    let second_pid = service
        .gateway_runtime_pid_for_test(&request)
        .expect("inspect gateway pid after second sync")
        .expect("gateway pid after second sync");
    assert_eq!(
        first_pid, second_pid,
        "sync should reuse the running gateway"
    );
}

#[tokio::test]
async fn gateway_restart_recycles_the_runtime() {
    let _env_lock = ENV_LOCK.lock().unwrap();
    let temp_dir = TempDir::new().unwrap();
    let binary = fake_gateway_binary(&temp_dir);
    let _binary_guard = EnvGuard::set("SUBSTRATE_GATEWAY_BINARY", binary);
    let _account_guard = EnvGuard::set(
        "SUBSTRATE_LLM_BACKEND_AUTH_CLI_CODEX_ACCOUNT_ID",
        "acct_test",
    );
    let _token_guard = EnvGuard::set(
        "SUBSTRATE_LLM_BACKEND_AUTH_CLI_CODEX_ACCESS_TOKEN",
        "header.payload.signature",
    );
    let Some(service) = service_or_skip() else {
        return;
    };
    let request = gateway_request(temp_dir.path());

    service
        .gateway_sync(request.clone())
        .await
        .expect("initial gateway sync");
    let initial_pid = service
        .gateway_runtime_pid_for_test(&request)
        .expect("inspect initial pid")
        .expect("initial pid");

    let restart_response = service
        .gateway_restart(request.clone())
        .await
        .expect("gateway restart");
    assert_eq!(restart_response.status, GatewayStatusV1::Available);

    let restarted_pid = service
        .gateway_runtime_pid_for_test(&request)
        .expect("inspect restarted pid")
        .expect("restarted pid");
    assert_ne!(
        initial_pid, restarted_pid,
        "restart should spawn a new process"
    );
}

#[tokio::test]
async fn gateway_status_turns_unavailable_after_child_exit() {
    let _env_lock = ENV_LOCK.lock().unwrap();
    let temp_dir = TempDir::new().unwrap();
    let binary = fake_gateway_binary(&temp_dir);
    let _binary_guard = EnvGuard::set("SUBSTRATE_GATEWAY_BINARY", binary);
    let _account_guard = EnvGuard::set(
        "SUBSTRATE_LLM_BACKEND_AUTH_CLI_CODEX_ACCOUNT_ID",
        "acct_test",
    );
    let _token_guard = EnvGuard::set(
        "SUBSTRATE_LLM_BACKEND_AUTH_CLI_CODEX_ACCESS_TOKEN",
        "header.payload.signature",
    );
    let Some(service) = service_or_skip() else {
        return;
    };
    let request = gateway_request(temp_dir.path());

    service
        .gateway_sync(request.clone())
        .await
        .expect("initial gateway sync");
    let pid = service
        .gateway_runtime_pid_for_test(&request)
        .expect("inspect pid")
        .expect("gateway pid");

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
    let _env_lock = ENV_LOCK.lock().unwrap();
    let temp_dir = TempDir::new().unwrap();
    let binary = crashing_gateway_binary(&temp_dir);
    let _binary_guard = EnvGuard::set("SUBSTRATE_GATEWAY_BINARY", binary);
    let _account_guard = EnvGuard::set(
        "SUBSTRATE_LLM_BACKEND_AUTH_CLI_CODEX_ACCOUNT_ID",
        "acct_test",
    );
    let _token_guard = EnvGuard::set(
        "SUBSTRATE_LLM_BACKEND_AUTH_CLI_CODEX_ACCESS_TOKEN",
        "header.payload.signature",
    );
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
