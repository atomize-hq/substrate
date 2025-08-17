use std::{env, process::Command};

fn main() {
    // Re-run if version changes
    println!("cargo:rerun-if-changed=Cargo.toml");
    
    // Get version from Cargo.toml
    let version = env::var("CARGO_PKG_VERSION").unwrap();
    
    // Try to get git hash, but handle non-git builds (e.g., from crates.io)
    let git_hash = if std::path::Path::new("../../.git").exists() {
        Command::new("git")
            .args(["rev-parse", "--short", "HEAD"])
            .output()
            .ok()
            .and_then(|o| String::from_utf8(o.stdout).ok())
            .map(|h| h.trim().to_string())
    } else {
        None
    };
    
    // Create version string with or without git hash
    let version_string = match git_hash {
        Some(hash) => format!("{version}-{hash}"),
        None => version, // Fallback for non-git builds
    };
    
    // Set environment variables for compilation
    println!("cargo:rustc-env=SHIM_VERSION={version_string}");
    
    // Note: Build time moved to runtime to avoid chrono in build deps if not needed
    // Can be added back if timestamp is critical
}