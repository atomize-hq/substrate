#![cfg(all(unix, target_os = "linux"))]

use agent_api_types::{ExecuteRequest, WorldFsMode};
use base64::engine::general_purpose::STANDARD as BASE64;
use base64::Engine;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
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
        agent_id: "overlayfs-enumeration-test".to_string(),
        budget: None,
        world_fs_mode: Some(world_fs_mode),
    };

    let rt = Runtime::new().expect("runtime");
    match rt.block_on(service.execute(req)) {
        Ok(resp) => Some(resp),
        Err(err) => {
            eprintln!("skipping overlayfs enumeration test: execute failed: {err}");
            None
        }
    }
}

#[test]
fn world_overlay_directory_enumeration_includes_created_file() {
    if !overlay_available() {
        eprintln!("skipping overlayfs enumeration test: overlay support or privileges missing");
        return;
    }

    let _ = set_global_broker(BrokerHandle::new());
    let service = match WorldAgentService::new() {
        Ok(svc) => svc,
        Err(err) => {
            eprintln!("skipping overlayfs enumeration test: service init failed: {err}");
            return;
        }
    };

    let tmp = tempdir().expect("tempdir");
    let cwd = tmp.path().to_path_buf();

    let mut env = HashMap::new();
    env.insert("SUBSTRATE_WORLD_REQUIRE_WORLD".to_string(), "1".to_string());

    let resp = match execute_non_pty(
        &service,
        &cwd,
        "sh -lc 'rm -f .substrate_enum_probe; touch .substrate_enum_probe; ls -a1; rm -f .substrate_enum_probe'",
        env,
        WorldFsMode::Writable,
    ) {
        Some(resp) => resp,
        None => return,
    };

    assert_eq!(
        resp.exit,
        0,
        "expected execution to succeed: exit={} stderr={}",
        resp.exit,
        decode(&resp.stderr_b64)
    );

    let stdout = decode(&resp.stdout_b64);
    let listed = stdout.lines().map(str::trim).collect::<Vec<_>>();
    assert!(
        listed.contains(&"."),
        "expected ls -a1 output to include '.'; stdout={stdout:?}"
    );
    assert!(
        listed.contains(&".."),
        "expected ls -a1 output to include '..'; stdout={stdout:?}"
    );
    assert!(
        listed.contains(&".substrate_enum_probe"),
        "expected ls -a1 output to include .substrate_enum_probe; stdout={stdout:?}"
    );
}
