#![cfg(all(unix, target_os = "linux"))]

use agent_api_types::{ExecuteRequest, WorldFsMode};
use base64::engine::general_purpose::STANDARD as BASE64;
use base64::Engine;
use std::collections::HashMap;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
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
    match WorldAgentService::new() {
        Ok(svc) => Some(svc),
        Err(err) => {
            eprintln!("skipping full-cage test: service init failed: {err}");
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
        agent_id: "full-cage-nonpty-test".to_string(),
        budget: None,
        world_fs_mode: Some(world_fs_mode),
    };

    let rt = Runtime::new().expect("runtime");
    match rt.block_on(service.execute(req)) {
        Ok(resp) => Some(resp),
        Err(err) => {
            eprintln!("skipping full-cage test: execute failed: {err}");
            None
        }
    }
}

fn base_cage_env() -> HashMap<String, String> {
    let mut env = HashMap::new();
    env.insert("SUBSTRATE_WORLD_FS_CAGE".to_string(), "full".to_string());
    env.insert("SUBSTRATE_WORLD_REQUIRE_WORLD".to_string(), "1".to_string());
    env
}

#[test]
fn non_pty_full_cage_prevents_host_tmp_writes() {
    if !overlay_available() {
        eprintln!("skipping full-cage non-PTY test: overlay support or privileges missing");
        return;
    }
    let service = match make_service() {
        Some(svc) => svc,
        None => return,
    };

    let tmp = tempdir().expect("tempdir");
    let cwd = tmp.path().to_path_buf();

    let host_path = PathBuf::from("/tmp").join(format!(
        "substrate-full-cage-host-marker-{}",
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

    if host_path.exists() {
        let stderr = decode(&resp.stderr_b64);
        panic!(
            "full-cage execution wrote to host /tmp (unexpected file: {}), stderr: {}",
            host_path.display(),
            stderr
        );
    }
}

#[test]
fn non_pty_full_cage_prevents_host_tmp_reads() {
    if !overlay_available() {
        eprintln!("skipping full-cage non-PTY test: overlay support or privileges missing");
        return;
    }
    let service = match make_service() {
        Some(svc) => svc,
        None => return,
    };

    let tmp = tempdir().expect("tempdir");
    let cwd = tmp.path().to_path_buf();

    let host_path = PathBuf::from("/tmp").join(format!(
        "substrate-full-cage-host-secret-{}",
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
        "full-cage execution was able to read host /tmp secret (path: {})",
        host_path.display()
    );

    let _ = fs::remove_file(&host_path);
}
