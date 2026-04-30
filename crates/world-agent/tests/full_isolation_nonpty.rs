#![cfg(all(unix, target_os = "linux"))]

use agent_api_types::{
    ExecuteRequest, PolicySnapshotV3, PolicySnapshotWorldFsDimensionV3,
    PolicySnapshotWorldFsFailClosedV3, PolicySnapshotWorldFsV3, PolicySnapshotWorldFsWriteV3,
    WorldFsMode,
};
use base64::engine::general_purpose::STANDARD as BASE64;
use base64::Engine;
use std::collections::HashMap;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::Command;
use substrate_broker::{set_global_broker, BrokerHandle};
use tempfile::tempdir;
use tokio::runtime::Runtime;
use world_agent::WorldAgentService;

fn decode(b64: &str) -> String {
    String::from_utf8_lossy(
        &BASE64
            .decode(b64.as_bytes())
            .unwrap_or_else(|_| Vec::from(b"<invalid base64>")),
    )
    .into_owned()
}

fn overlay_available() -> bool {
    (unsafe { libc::geteuid() == 0 })
        && fs::read_to_string("/proc/filesystems")
            .map(|data| data.contains("overlay"))
            .unwrap_or(false)
}

fn landlock_supported() -> bool {
    world::landlock::detect_support().supported
}

#[test]
fn landlock_exec_subprocess_entry() {
    if std::env::var("SUBSTRATE_TEST_RUN_LANDLOCK_EXEC")
        .ok()
        .as_deref()
        != Some("1")
    {
        return;
    }

    match world_agent::internal_exec::run_landlock_exec() {
        Ok(()) => {
            eprintln!("unexpected success: landlock exec wrapper returned Ok(())");
            std::process::exit(1);
        }
        Err(err) => {
            eprintln!("unexpected error return (expected process exit): {err:#}");
            std::process::exit(1);
        }
    }
}

fn make_service() -> Option<WorldAgentService> {
    let _ = set_global_broker(BrokerHandle::new());
    match WorldAgentService::new() {
        Ok(svc) => Some(svc),
        Err(err) => {
            eprintln!("skipping full-isolation test: service init failed: {err}");
            None
        }
    }
}

fn execute_non_pty(
    service: &WorldAgentService,
    cwd: &Path,
    cmd: &str,
    env: HashMap<String, String>,
    world_fs_mode: WorldFsMode,
    write_allow_list: Vec<String>,
) -> Option<agent_api_types::ExecuteResponse> {
    let write_enabled = matches!(world_fs_mode, WorldFsMode::Writable);
    let policy_snapshot = PolicySnapshotV3 {
        schema_version: 3,
        net_allowed: Vec::new(),
        world_fs: PolicySnapshotWorldFsV3 {
            host_visible: false,
            fail_closed: PolicySnapshotWorldFsFailClosedV3 {
                routing: !write_enabled,
            },
            deny_enforcement: None,
            caged_required: false,
            discover: Some(PolicySnapshotWorldFsDimensionV3 {
                allow_list: vec![".".to_string()],
                deny_list: Vec::new(),
            }),
            read: Some(PolicySnapshotWorldFsDimensionV3 {
                allow_list: vec![".".to_string()],
                deny_list: Vec::new(),
            }),
            write: PolicySnapshotWorldFsWriteV3 {
                enabled: write_enabled,
                allow_list: write_allow_list,
                deny_list: Vec::new(),
            },
        },
    };

    let req = ExecuteRequest {
        profile: None,
        cmd: cmd.to_string(),
        cwd: Some(cwd.display().to_string()),
        env: Some(env),
        pty: false,
        agent_id: "full-isolation-nonpty-test".to_string(),
        budget: None,
        policy_snapshot,
        shared_world: None,
        world_network: None,
        world_fs_mode: Some(world_fs_mode),
    };

    let rt = Runtime::new().expect("runtime");
    match rt.block_on(service.execute(req)) {
        Ok(resp) => Some(resp),
        Err(err) => {
            eprintln!("skipping full-isolation test: execute failed: {err}");
            None
        }
    }
}

fn base_cage_env() -> HashMap<String, String> {
    let mut env = HashMap::new();
    env.insert("SUBSTRATE_WORLD_REQUIRE_WORLD".to_string(), "1".to_string());
    env
}

#[test]
fn non_pty_full_isolation_prevents_host_tmp_writes() {
    if !overlay_available() {
        eprintln!("skipping full-isolation non-PTY test: overlay support or privileges missing");
        return;
    }
    let service = match make_service() {
        Some(svc) => svc,
        None => return,
    };

    let tmp = tempdir().expect("tempdir");
    let cwd = tmp.path().to_path_buf();

    let host_path = PathBuf::from("/tmp").join(format!(
        "substrate-full-isolation-host-marker-{}",
        uuid::Uuid::now_v7()
    ));
    let _ = fs::remove_file(&host_path);

    let mut env = base_cage_env();
    env.insert(
        "SUBSTRATE_TEST_HOST_MARKER".to_string(),
        host_path.display().to_string(),
    );

    let resp = match execute_non_pty(
        &service,
        &cwd,
        r#"sh -lc 'echo cage > "$SUBSTRATE_TEST_HOST_MARKER"'"#,
        env,
        WorldFsMode::Writable,
        vec![".".to_string()],
    ) {
        Some(resp) => resp,
        None => return,
    };

    assert_eq!(
        resp.exit,
        0,
        "full-isolation execution failed unexpectedly: exit={} stderr={}",
        resp.exit,
        decode(&resp.stderr_b64)
    );

    if host_path.exists() {
        let stderr = decode(&resp.stderr_b64);
        panic!(
            "full-isolation execution wrote to host /tmp (unexpected file: {}), stderr: {}",
            host_path.display(),
            stderr
        );
    }
}

#[test]
fn non_pty_full_isolation_prevents_host_tmp_reads() {
    if !overlay_available() {
        eprintln!("skipping full-isolation non-PTY test: overlay support or privileges missing");
        return;
    }
    let service = match make_service() {
        Some(svc) => svc,
        None => return,
    };

    let tmp = tempdir().expect("tempdir");
    let cwd = tmp.path().to_path_buf();

    let host_path = PathBuf::from("/tmp").join(format!(
        "substrate-full-isolation-host-secret-{}",
        uuid::Uuid::now_v7()
    ));
    let secret = format!("host-secret-{}\n", uuid::Uuid::now_v7());
    fs::write(&host_path, secret.as_bytes()).expect("write host secret");
    fs::set_permissions(&host_path, fs::Permissions::from_mode(0o644)).expect("chmod host secret");

    let mut env = base_cage_env();
    env.insert(
        "SUBSTRATE_TEST_HOST_SECRET".to_string(),
        host_path.display().to_string(),
    );

    let resp = match execute_non_pty(
        &service,
        &cwd,
        r#"sh -lc 'cat "$SUBSTRATE_TEST_HOST_SECRET"'"#,
        env,
        WorldFsMode::Writable,
        vec![".".to_string()],
    ) {
        Some(resp) => resp,
        None => return,
    };

    let stdout = decode(&resp.stdout_b64);
    assert!(
        !stdout.contains(&secret),
        "full-isolation execution was able to read host /tmp secret (path: {})",
        host_path.display()
    );
    assert_ne!(
        resp.exit, 0,
        "expected host /tmp read attempt to fail inside full isolation, but exit=0 (stdout={stdout:?} stderr={})",
        decode(&resp.stderr_b64)
    );

    let _ = fs::remove_file(&host_path);
}

#[test]
fn non_pty_full_isolation_runs_from_tmp_rooted_project() {
    if !overlay_available() {
        eprintln!("skipping full-isolation non-PTY test: overlay support or privileges missing");
        return;
    }
    let service = match make_service() {
        Some(svc) => svc,
        None => return,
    };

    let tmp = tempdir().expect("tempdir");
    let cwd = tmp.path().to_path_buf();

    let resp = match execute_non_pty(
        &service,
        &cwd,
        "sh -lc 'pwd'",
        base_cage_env(),
        WorldFsMode::Writable,
        vec![".".to_string()],
    ) {
        Some(resp) => resp,
        None => return,
    };

    assert_eq!(
        resp.exit,
        0,
        "full-isolation execution failed unexpectedly: exit={} stderr={}",
        resp.exit,
        decode(&resp.stderr_b64)
    );

    let pwd = decode(&resp.stdout_b64);
    assert!(
        pwd.trim_start().starts_with("/project"),
        "expected full-isolation cwd to use stable /project mount for /tmp-rooted projects, got: {pwd:?}"
    );
}

#[test]
fn non_pty_full_isolation_honors_write_allowlist_prefix_globs() {
    if !overlay_available() || !landlock_supported() {
        eprintln!(
            "skipping full-isolation non-PTY allowlist test: overlay or Landlock support missing"
        );
        return;
    }
    let service = match make_service() {
        Some(svc) => svc,
        None => return,
    };

    let tmp = tempdir().expect("tempdir");
    let cwd = tmp.path().to_path_buf();
    let resp = match execute_non_pty(
        &service,
        &cwd,
        r#"sh -lc 'set -eu
mp="${SUBSTRATE_MOUNT_PROJECT_DIR:-}"
echo "MOUNT_PROJECT_DIR=${mp:-<missing>}"
if [ -z "$mp" ]; then
  echo MISSING_SUBSTRATE_MOUNT_PROJECT_DIR
  exit 44
fi
line="$(awk -v mp="$mp" '\''$5==mp {line=$0} END { if (line=="") exit 1; print line }'\'' /proc/self/mountinfo 2>/dev/null)" || {
  echo MOUNTINFO_NO_MATCH
  exit 45
}
echo "MOUNTINFO_LINE=$line"
echo "$line" | grep -q " - overlay " || {
  echo MOUNTINFO_NOT_OVERLAY
  exit 46
}
echo "$line" | grep -q "upperdir=" || {
  echo MOUNTINFO_MISSING_UPPERDIR
  exit 47
}
echo "$line" | grep -q "workdir=" || {
  echo MOUNTINFO_MISSING_WORKDIR
  exit 48
}
if [ -z "${SUBSTRATE_WORLD_FS_LANDLOCK_WRITE_ALLOWLIST:-}" ]; then
  echo MISSING_LANDLOCK_WRITE_ALLOWLIST_ENV
  exit 49
fi
expected="${mp%/}/writable"
echo "$SUBSTRATE_WORLD_FS_LANDLOCK_WRITE_ALLOWLIST" | grep -F "$expected" >/dev/null || {
  echo "MISSING_LANDLOCK_WRITE_ENTRY=$expected"
  echo "LANDLOCK_WRITE_ALLOWLIST=$SUBSTRATE_WORLD_FS_LANDLOCK_WRITE_ALLOWLIST"
  exit 50
}
echo "PWD=$(pwd)"
mkdir -p writable/sub
echo ok > writable/sub/ok.txt
cat writable/sub/ok.txt
if deny_out="$(sh -c '\''echo nope > denied.txt'\'' 2>&1)"; then
  echo UNEXPECTED_WRITE
  exit 41
else
  echo DENIED_WRITE
  echo "DENIED_WRITE_ERR=$deny_out"
  case "$deny_out" in
    *"Permission denied"*|*"Read-only file system"*|*"Operation not permitted"*) ;;
    *)
      echo "UNEXPECTED_DENIED_WRITE_ERR=$deny_out"
      exit 51
      ;;
  esac
fi
        '"#,
        base_cage_env(),
        WorldFsMode::Writable,
        vec!["writable".to_string()],
    ) {
        Some(resp) => resp,
        None => return,
    };

    let stdout = decode(&resp.stdout_b64);
    let stderr = decode(&resp.stderr_b64);
    assert_eq!(
        resp.exit, 0,
        "full-isolation allowlist test failed unexpectedly: exit={} stdout={stdout:?} stderr={stderr:?}",
        resp.exit
    );
    assert!(
        stdout.contains("DENIED_WRITE"),
        "expected denied write marker in stdout, got: {stdout:?} stderr={stderr:?}"
    );
    assert!(
        !stdout.contains("UNEXPECTED_WRITE"),
        "expected denied write, but write succeeded: stdout={stdout:?} stderr={stderr:?}"
    );

    assert!(
        !cwd.join("writable/sub/ok.txt").exists(),
        "allowlisted write should not mutate host project directory"
    );
    assert!(
        !cwd.join("denied.txt").exists(),
        "denied write should not mutate host project directory"
    );
}

#[test]
fn landlock_exec_fails_closed_with_actionable_error_when_overlay_backing_dirs_missing() {
    if !landlock_supported() {
        eprintln!("skipping landlock exec wrapper failure test: Landlock unsupported");
        return;
    }

    let exe = std::env::current_exe().expect("current_exe");

    let output = Command::new(exe)
        .arg("landlock_exec_subprocess_entry")
        .arg("--exact")
        .arg("--nocapture")
        .env("SUBSTRATE_TEST_RUN_LANDLOCK_EXEC", "1")
        .env("SUBSTRATE_WORLD_FS_ISOLATION", "full")
        .env("SUBSTRATE_MOUNT_FS_MODE", "writable")
        .env("SUBSTRATE_MOUNT_PROJECT_DIR", "/proc")
        .env(
            "SUBSTRATE_WORLD_FS_LANDLOCK_WRITE_ALLOWLIST",
            "/project/writable",
        )
        .env("SUBSTRATE_MOUNT_CWD", "/")
        .env("SUBSTRATE_INNER_CMD", "echo should-not-run")
        .env("SUBSTRATE_INNER_LOGIN_SHELL", "0")
        .output()
        .expect("run world-agent landlock exec wrapper");

    assert_eq!(
        output.status.code(),
        Some(4),
        "expected strict overlay backing dir derivation to fail closed with exit=4, got status={:?} stdout={} stderr={}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
    );

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("full isolation landlock prerequisites missing"),
        "expected high-signal prerequisites error, got stderr={stderr:?}"
    );
    assert!(
        stderr.contains("\"feature\":\"full-isolation-landlock-overlayfs-compat\""),
        "expected feature tag in stderr, got stderr={stderr:?}"
    );
    assert!(
        stderr.contains("\"mount_point\":\"/proc\""),
        "expected mount_point in stderr, got stderr={stderr:?}"
    );
    assert!(
        stderr.contains("deriving overlayfs backing dirs from /proc/self/mountinfo"),
        "expected remediation hint in stderr, got stderr={stderr:?}"
    );
}

#[test]
fn non_pty_full_isolation_blocks_outside_host_reads_and_writes() {
    if !overlay_available() {
        eprintln!("skipping full-isolation non-PTY test: overlay support or privileges missing");
        return;
    }
    let service = match make_service() {
        Some(svc) => svc,
        None => return,
    };

    let tmp = tempdir().expect("tempdir");
    let cwd = tmp.path().to_path_buf();

    let host_secret = PathBuf::from("/var/tmp").join(format!(
        "substrate-full-isolation-host-secret-{}",
        uuid::Uuid::now_v7()
    ));
    let host_marker = PathBuf::from("/var/tmp").join(format!(
        "substrate-full-isolation-host-marker-{}",
        uuid::Uuid::now_v7()
    ));
    let secret = format!("host-secret-{}\n", uuid::Uuid::now_v7());
    let _ = fs::remove_file(&host_marker);
    fs::write(&host_secret, secret.as_bytes()).expect("write host secret");
    fs::set_permissions(&host_secret, fs::Permissions::from_mode(0o644))
        .expect("chmod host secret");

    let mut env = base_cage_env();
    env.insert(
        "SUBSTRATE_TEST_HOST_SECRET".to_string(),
        host_secret.display().to_string(),
    );
    env.insert(
        "SUBSTRATE_TEST_HOST_MARKER".to_string(),
        host_marker.display().to_string(),
    );

    let resp = match execute_non_pty(
        &service,
        &cwd,
        r#"sh -lc 'set -eu
if cat "$SUBSTRATE_TEST_HOST_SECRET" >/dev/null 2>&1; then
  echo UNEXPECTED_READ
  exit 42
else
  echo READ_BLOCKED
fi
if echo cage > "$SUBSTRATE_TEST_HOST_MARKER" 2>/dev/null; then
  echo UNEXPECTED_WRITE
  exit 43
else
  echo WRITE_BLOCKED
fi
        '"#,
        env,
        WorldFsMode::Writable,
        vec![".".to_string()],
    ) {
        Some(resp) => resp,
        None => return,
    };

    let stdout = decode(&resp.stdout_b64);
    let stderr = decode(&resp.stderr_b64);
    assert_eq!(
        resp.exit, 0,
        "full-isolation outside-host test failed unexpectedly: exit={} stdout={stdout:?} stderr={stderr:?}",
        resp.exit
    );
    assert!(
        stdout.contains("READ_BLOCKED") && stdout.contains("WRITE_BLOCKED"),
        "expected READ_BLOCKED and WRITE_BLOCKED markers: stdout={stdout:?} stderr={stderr:?}"
    );
    assert!(
        !stdout.contains("UNEXPECTED_"),
        "expected outside-host access to be blocked: stdout={stdout:?} stderr={stderr:?}"
    );

    assert!(
        !host_marker.exists(),
        "full-isolation execution wrote to host path outside project (unexpected file: {})",
        host_marker.display()
    );

    let _ = fs::remove_file(&host_secret);
}
