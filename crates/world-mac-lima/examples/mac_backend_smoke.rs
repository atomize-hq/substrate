//! Mac backend smoke test example
//!
//! This example exercises the MacLimaBackend to ensure it can:
//! - Start/connect to Lima VM
//! - Establish forwarding
//! - Execute a command via agent

use anyhow::Result;
use world_api::{ExecRequest, WorldBackend, WorldSpec};
use world_mac_lima::MacLimaBackend;

fn main() -> Result<()> {
    // RUST_LOG env var affects library's internal logging

    println!("Creating MacLimaBackend...");
    let backend = MacLimaBackend::new()?;

    println!("Creating world session...");
    let spec = WorldSpec::default();
    let handle = backend.ensure_session(&spec)?;
    println!("World session created: {}", handle.id);

    println!("Executing test command...");
    let req = ExecRequest {
        cmd: "bash -lc 'echo from-mac-backend'".to_string(),
        cwd: std::env::current_dir()?,
        env: std::env::vars().collect(),
        pty: false,
        span_id: None,
    };

    let res = backend.exec(&handle, req)?;
    println!(
        "Command executed:\n  exit={}\n  stdout={}\n  stderr={}",
        res.exit,
        String::from_utf8_lossy(&res.stdout),
        String::from_utf8_lossy(&res.stderr)
    );

    if res.exit == 0 {
        println!("✅ Smoke test passed!");
    } else {
        println!("❌ Command failed with exit code {}", res.exit);
    }

    Ok(())
}
