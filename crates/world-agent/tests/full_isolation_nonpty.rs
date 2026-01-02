#![cfg(all(unix, target_os = "linux"))]

use agent_api_types::{ExecuteRequest, WorldFsMode};
use base64::engine::general_purpose::STANDARD as BASE64;
use base64::Engine;
use std::collections::HashMap;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
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
) -> Option<agent_api_types::ExecuteResponse> {
    let req = ExecuteRequest {
        profile: None,
        cmd: cmd.to_string(),
        cwd: Some(cwd.display().to_string()),
        env: Some(env),
        pty: false,
        agent_id: "full-isolation-nonpty-test".to_string(),
        budget: None,
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

fn write_profile_policy(project_dir: &Path, write_allowlist: &[&str]) {
    let profile_dir = project_dir.join(".substrate-profile.d");
    fs::create_dir_all(&profile_dir).expect("create profile directory");

    let allowlist_yaml = if write_allowlist.is_empty() {
        "  write_allowlist: []\n".to_string()
    } else {
        let mut out = String::from("  write_allowlist:\n");
        for pattern in write_allowlist {
            out.push_str(&format!("    - {pattern:?}\n"));
        }
        out
    };

    let policy = format!(
        r#"id: full-isolation-test
name: Full Cage Test Policy
world_fs:
  mode: writable
  isolation: full
  require_world: true
  read_allowlist: ["*"]
{allowlist_yaml}net_allowed: []
cmd_allowed: []
cmd_denied: []
cmd_isolated: []
require_approval: false
allow_shell_operators: true
limits:
  max_memory_mb: null
  max_cpu_percent: null
  max_runtime_ms: null
  max_egress_bytes: null
metadata: {{}}
"#
    );

    fs::write(profile_dir.join("policy.yaml"), policy.as_bytes()).expect("write profile policy");
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
    write_profile_policy(&cwd, &["./writable/*"]);

    let resp = match execute_non_pty(
        &service,
        &cwd,
        r#"sh -lc 'set -eu
echo "PWD=$(pwd)"
mkdir -p writable/sub
echo ok > writable/sub/ok.txt
cat writable/sub/ok.txt
if echo nope > denied.txt 2>/dev/null; then
  echo UNEXPECTED_WRITE
  exit 41
else
  echo DENIED_WRITE
fi
'"#,
        base_cage_env(),
        WorldFsMode::Writable,
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
