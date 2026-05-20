#![cfg(all(unix, target_os = "linux"))]

use agent_api_types::{
    ExecuteRequest, PolicySnapshotV3, PolicySnapshotWorldFsDimensionV3,
    PolicySnapshotWorldFsFailClosedV3, PolicySnapshotWorldFsV3, PolicySnapshotWorldFsWriteV3,
    WorldFsDenyEnforcementV3,
};
use base64::engine::general_purpose::STANDARD as BASE64;
use base64::Engine;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use substrate_broker::{set_global_broker, BrokerHandle};
use tempfile::tempdir;
use tokio::runtime::Runtime;
use world_agent::WorldAgentService;

#[cfg(unix)]
use std::os::unix::fs::symlink;

fn decode(b64: &str) -> String {
    String::from_utf8_lossy(
        &BASE64
            .decode(b64.as_bytes())
            .unwrap_or_else(|_| Vec::from(b"<invalid base64>")),
    )
    .into_owned()
}

fn combined_output(resp: &agent_api_types::ExecuteResponse) -> String {
    format!("{}{}", decode(&resp.stdout_b64), decode(&resp.stderr_b64))
}

fn overlay_available() -> bool {
    (unsafe { libc::geteuid() == 0 })
        && fs::read_to_string("/proc/filesystems")
            .map(|data| data.contains("overlay"))
            .unwrap_or(false)
}

fn snapshot_read_only_with_read_deny(deny_list: Vec<String>) -> PolicySnapshotV3 {
    PolicySnapshotV3 {
        schema_version: 3,
        net_allowed: Vec::new(),
        world_fs: PolicySnapshotWorldFsV3 {
            host_visible: false,
            fail_closed: PolicySnapshotWorldFsFailClosedV3 { routing: true },
            deny_enforcement: Some(WorldFsDenyEnforcementV3::Weak),
            caged_required: false,
            discover: None,
            read: Some(PolicySnapshotWorldFsDimensionV3 {
                allow_list: vec![".".to_string()],
                deny_list,
            }),
            write: PolicySnapshotWorldFsWriteV3 {
                enabled: false,
                allow_list: vec![".".to_string()],
                deny_list: Vec::new(),
            },
        },
    }
}

fn snapshot_writable_with_write_deny(deny_list: Vec<String>) -> PolicySnapshotV3 {
    PolicySnapshotV3 {
        schema_version: 3,
        net_allowed: Vec::new(),
        world_fs: PolicySnapshotWorldFsV3 {
            host_visible: false,
            fail_closed: PolicySnapshotWorldFsFailClosedV3 { routing: false },
            deny_enforcement: Some(WorldFsDenyEnforcementV3::Weak),
            caged_required: false,
            discover: None,
            read: Some(PolicySnapshotWorldFsDimensionV3 {
                allow_list: vec![".".to_string()],
                deny_list: Vec::new(),
            }),
            write: PolicySnapshotWorldFsWriteV3 {
                enabled: true,
                allow_list: vec![".".to_string()],
                deny_list,
            },
        },
    }
}

fn execute_non_pty(
    service: &WorldAgentService,
    cwd: &Path,
    cmd: &str,
    env: HashMap<String, String>,
    policy_snapshot: PolicySnapshotV3,
) -> Option<agent_api_types::ExecuteResponse> {
    let req = ExecuteRequest {
        profile: None,
        cmd: cmd.to_string(),
        cwd: Some(cwd.display().to_string()),
        env: Some(env),
        pty: false,
        agent_id: "wfgad3-test".to_string(),
        budget: None,
        policy_snapshot,
        shared_world: None,
        world_network: None,
        world_fs_mode: None,
        member_dispatch: None,
    };

    let rt = Runtime::new().expect("runtime");
    match rt.block_on(service.execute(req)) {
        Ok(resp) => Some(resp),
        Err(err) => {
            eprintln!("skipping WFGAD3 test: execute failed: {err}");
            None
        }
    }
}

#[test]
fn wildcard_deny_snapshot_scan_does_not_follow_symlinked_directories() {
    if !overlay_available() {
        eprintln!("skipping WFGAD3 test: overlay support or privileges missing");
        return;
    }

    let _ = set_global_broker(BrokerHandle::new());
    let service = match WorldAgentService::new() {
        Ok(svc) => svc,
        Err(err) => {
            eprintln!("skipping WFGAD3 test: service init failed: {err}");
            return;
        }
    };

    // Host-visible directory that is guaranteed to be bind-mounted in full isolation.
    let world_deps_root = Path::new("/var/lib/substrate/world-deps");
    if let Err(err) = fs::create_dir_all(world_deps_root) {
        eprintln!(
            "skipping WFGAD3 test: cannot create {}: {err}",
            world_deps_root.display()
        );
        return;
    }
    let world_deps = match tempfile::Builder::new()
        .prefix("wfgad3_symlink_scan_")
        .tempdir_in(world_deps_root)
    {
        Ok(d) => d,
        Err(err) => {
            eprintln!(
                "skipping WFGAD3 test: cannot create tempdir under {}: {err}",
                world_deps_root.display()
            );
            return;
        }
    };
    let external_pem = world_deps.path().join("external.pem");
    fs::write(&external_pem, "EXTERNAL\n").expect("write external.pem");

    let tmp = tempdir().expect("tempdir");
    let cwd = tmp.path().to_path_buf();

    fs::create_dir_all(cwd.join("certs")).expect("create ./certs");
    fs::write(cwd.join("certs/a.pem"), "DIRECT\n").expect("write ./certs/a.pem");

    symlink(world_deps.path(), cwd.join("certs_link")).expect("symlink ./certs_link -> world-deps");

    let policy_snapshot = snapshot_read_only_with_read_deny(vec!["**/*.pem".to_string()]);

    let mut env = HashMap::new();
    env.insert("SUBSTRATE_WORLD_REQUIRE_WORLD".to_string(), "1".to_string());

    let resp = match execute_non_pty(
        &service,
        &cwd,
        "sh -lc 'cat ./certs/a.pem'",
        env.clone(),
        policy_snapshot.clone(),
    ) {
        Some(resp) => resp,
        None => return,
    };
    assert_ne!(
        resp.exit,
        0,
        "expected wildcard deny to block direct .pem read; output={}",
        combined_output(&resp)
    );
    let out = combined_output(&resp).to_ascii_lowercase();
    assert!(
        out.contains("permission denied"),
        "expected EACCES/permission denied for read deny; output={}",
        combined_output(&resp)
    );

    let resp = match execute_non_pty(
        &service,
        &cwd,
        "sh -lc 'cat \"$SUBSTRATE_MOUNT_PROJECT_DIR/certs/a.pem\"'",
        env.clone(),
        policy_snapshot.clone(),
    ) {
        Some(resp) => resp,
        None => return,
    };
    assert_ne!(
        resp.exit,
        0,
        "expected deny masks to apply to $SUBSTRATE_MOUNT_PROJECT_DIR view; output={}",
        combined_output(&resp)
    );
    let out = combined_output(&resp).to_ascii_lowercase();
    assert!(
        out.contains("permission denied"),
        "expected EACCES/permission denied for read deny; output={}",
        combined_output(&resp)
    );

    let resp = match execute_non_pty(
        &service,
        &cwd,
        "sh -lc 'cat ./certs_link/external.pem'",
        env.clone(),
        policy_snapshot.clone(),
    ) {
        Some(resp) => resp,
        None => return,
    };
    assert_eq!(
        resp.exit, 0,
        "expected symlinked external .pem to remain readable (scan must not follow symlinks); output={}",
        combined_output(&resp)
    );
    assert!(
        decode(&resp.stdout_b64).contains("EXTERNAL"),
        "expected external content; stdout={}",
        decode(&resp.stdout_b64)
    );

    let resp = match execute_non_pty(
        &service,
        &cwd,
        "sh -lc 'cat \"$SUBSTRATE_MOUNT_PROJECT_DIR/certs_link/external.pem\"'",
        env,
        policy_snapshot,
    ) {
        Some(resp) => resp,
        None => return,
    };
    assert_eq!(
        resp.exit, 0,
        "expected symlinked external .pem to remain readable via $SUBSTRATE_MOUNT_PROJECT_DIR; output={}",
        combined_output(&resp)
    );
    assert!(
        decode(&resp.stdout_b64).contains("EXTERNAL"),
        "expected external content; stdout={}",
        decode(&resp.stdout_b64)
    );
}

#[test]
fn write_deny_returns_read_only_file_system() {
    if !overlay_available() {
        eprintln!("skipping WFGAD3 test: overlay support or privileges missing");
        return;
    }

    let _ = set_global_broker(BrokerHandle::new());
    let service = match WorldAgentService::new() {
        Ok(svc) => svc,
        Err(err) => {
            eprintln!("skipping WFGAD3 test: service init failed: {err}");
            return;
        }
    };

    let tmp = tempdir().expect("tempdir");
    let cwd = tmp.path().to_path_buf();

    let policy_snapshot =
        snapshot_writable_with_write_deny(vec!["./outputs/private/**".to_string()]);

    let mut env = HashMap::new();
    env.insert("SUBSTRATE_WORLD_REQUIRE_WORLD".to_string(), "1".to_string());

    let ok = match execute_non_pty(
        &service,
        &cwd,
        "sh -lc 'mkdir -p ./outputs/public/x'",
        env.clone(),
        policy_snapshot.clone(),
    ) {
        Some(resp) => resp,
        None => return,
    };
    assert_eq!(
        ok.exit,
        0,
        "expected non-denied writes to succeed; output={}",
        combined_output(&ok)
    );

    let denied = match execute_non_pty(
        &service,
        &cwd,
        "sh -lc 'mkdir -p ./outputs/private/x'",
        env,
        policy_snapshot,
    ) {
        Some(resp) => resp,
        None => return,
    };
    assert_ne!(
        denied.exit,
        0,
        "expected mkdir under write deny prefix to fail; output={}",
        combined_output(&denied)
    );
    let out = combined_output(&denied).to_ascii_lowercase();
    assert!(
        out.contains("read-only file system") || out.contains("readonly file system"),
        "expected EROFS/read-only diagnostic for write deny; output={}",
        combined_output(&denied)
    );
}
