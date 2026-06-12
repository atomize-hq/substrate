// Mac backend smoke test example

#[cfg(not(target_os = "macos"))]
fn main() {
    eprintln!("mac_backend_smoke example runs only on macOS; skipping.");
}

#[cfg(target_os = "macos")]
use anyhow::Result;
#[cfg(target_os = "macos")]
use std::path::PathBuf;
#[cfg(target_os = "macos")]
use world_api::{
    BackendPolicyInputV1, BackendPolicySnapshotV3, BackendPolicySnapshotWorldFsDimensionV3,
    BackendPolicySnapshotWorldFsFailClosedV3, BackendPolicySnapshotWorldFsV3,
    BackendPolicySnapshotWorldFsWriteV3, BackendWorldFsDenyEnforcementV3,
    BackendWorldNetworkRoutingV1, ExecRequest, WorldBackend, WorldFsMode, WorldSpec,
};
#[cfg(target_os = "macos")]
use world_mac_lima::MacLimaBackend;

#[cfg(target_os = "macos")]
fn smoke_world_spec(project_dir: PathBuf) -> WorldSpec {
    WorldSpec {
        project_dir,
        fs_mode: WorldFsMode::ReadOnly,
        backend_policy: Some(BackendPolicyInputV1 {
            schema_version: 1,
            policy_snapshot: BackendPolicySnapshotV3 {
                schema_version: 3,
                net_allowed: vec!["https://api.example.com".to_string()],
                world_fs: BackendPolicySnapshotWorldFsV3 {
                    host_visible: false,
                    fail_closed: BackendPolicySnapshotWorldFsFailClosedV3 { routing: true },
                    deny_enforcement: Some(BackendWorldFsDenyEnforcementV3::Strict),
                    caged_required: true,
                    discover: Some(BackendPolicySnapshotWorldFsDimensionV3 {
                        allow_list: vec![".".to_string()],
                        deny_list: vec!["tmp".to_string()],
                    }),
                    read: Some(BackendPolicySnapshotWorldFsDimensionV3 {
                        allow_list: vec![".".to_string()],
                        deny_list: vec!["private".to_string()],
                    }),
                    write: BackendPolicySnapshotWorldFsWriteV3 {
                        enabled: false,
                        allow_list: vec!["out".to_string()],
                        deny_list: vec!["out/blocked".to_string()],
                    },
                },
            },
            world_network: BackendWorldNetworkRoutingV1 {
                isolate_network: true,
                allowed_domains: vec!["api.example.com".to_string()],
            },
        }),
        ..WorldSpec::default()
    }
}

#[cfg(target_os = "macos")]
fn main() -> Result<()> {
    println!("Creating MacLimaBackend...");
    let backend = MacLimaBackend::new()?;

    println!("Creating world session with authoritative backend policy...");
    let spec = smoke_world_spec(std::env::current_dir()?);
    let handle = backend.ensure_session(&spec)?;
    println!("World session created: {}", handle.id);

    println!("Executing test command...");
    let req = ExecRequest {
        cmd: "bash -lc 'echo from-mac-backend'".to_string(),
        cwd: std::env::current_dir()?,
        env: std::env::vars().collect(),
        pty: false,
        span_id: None,
        shared_world: None,
        member_dispatch: None,
    };

    let res = backend.exec(&handle, req)?;
    println!(
        "Command executed:\n  exit={}\n  stdout={}\n  stderr={}",
        res.exit,
        String::from_utf8_lossy(&res.stdout),
        String::from_utf8_lossy(&res.stderr)
    );

    if res.exit == 0 {
        println!("✓ Smoke test passed!");
    } else {
        println!("✗ Command failed with exit code {}", res.exit);
    }

    Ok(())
}
