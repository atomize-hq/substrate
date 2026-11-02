//! Resolve a pinned Codex binary from an app-owned bundle without ever
//! consulting `CODEX_BINARY` or `PATH`. Hosts own the bundle root and version,
//! and this example bails fast when the pinned binary is missing.
//!
//! Example layout: `<bundle_root>/<platform>/<version>/<codex|codex.exe>`.
//! Platform defaults to the current target (e.g. `darwin-arm64`, `linux-x64`,
//! `windows-x64`) but can be overridden.
//!
//! Example run (env vars are only used by this example):
//! ```bash
//! CODEX_BUNDLE_ROOT="$HOME/.myapp/codex-bin" \
//! CODEX_BUNDLE_VERSION="1.2.3" \
//! cargo run -p unified-agent-api-codex --example bundled_binary -- "Quick health check"
//! ```

use std::{env, error::Error, path::PathBuf, time::Duration};

use codex::{resolve_bundled_binary, BundledBinarySpec, CodexClient};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let prompt = collect_prompt();
    let bundle_root = require_env_path("CODEX_BUNDLE_ROOT")?;
    let version = require_env_string("CODEX_BUNDLE_VERSION")?;
    let platform = env::var("CODEX_BUNDLE_PLATFORM").ok();
    let bundled = resolve_bundled_binary(BundledBinarySpec {
        bundle_root: bundle_root.as_path(),
        version: &version,
        platform: platform.as_deref(),
    })?;

    println!("Resolved bundled Codex:");
    println!("  binary:   {}", bundled.binary_path.display());
    println!("  platform: {}", bundled.platform);
    println!("  version:  {}", bundled.version);

    let client = CodexClient::builder()
        .binary(&bundled.binary_path)
        .timeout(Duration::from_secs(45))
        .build();

    match client.send_prompt(&prompt).await {
        Ok(response) => println!("Codex replied:\n{response}"),
        Err(error) => {
            eprintln!("Codex invocation failed: {error}");
            eprintln!("Double-check the bundle root/version and ensure the binary is executable.");
        }
    }

    Ok(())
}

fn collect_prompt() -> String {
    let args: Vec<String> = env::args().skip(1).collect();
    if args.is_empty() {
        "Say hello from the bundled binary".to_string()
    } else {
        args.join(" ")
    }
}

fn require_env_path(key: &str) -> Result<PathBuf, Box<dyn Error>> {
    Ok(PathBuf::from(require_env_string(key)?))
}

fn require_env_string(key: &str) -> Result<String, Box<dyn Error>> {
    match env::var(key) {
        Ok(value) if !value.trim().is_empty() => Ok(value),
        _ => Err(format!("Set {key} to point at your app-owned Codex bundle").into()),
    }
}
