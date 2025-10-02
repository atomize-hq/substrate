// Mac backend smoke test example

#[cfg(not(target_os = "macos"))]
fn main() {
    eprintln!("mac_backend_smoke example runs only on macOS; skipping.");
}

#[cfg(target_os = "macos")]
use anyhow::Result;
#[cfg(target_os = "macos")]
use world_api::{ExecRequest, WorldBackend, WorldSpec};
#[cfg(target_os = "macos")]
use world_mac_lima::MacLimaBackend;

#[cfg(target_os = "macos")]
fn main() -> Result<()> {
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
        println!("? Smoke test passed!");
    } else {
        println!("? Command failed with exit code {}", res.exit);
    }

    Ok(())
}
