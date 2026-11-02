//! Small helper for examples that need a real Codex CLI binary + CODEX_HOME.
//!
//! Conventions:
//! - The Codex crate already honors `CODEX_BINARY` / `CODEX_HOME` via the builder defaults, but
//!   examples can opt into an isolated home under `target/` to avoid mutating a user's real home.
//! - When isolated, you may optionally seed auth (`auth.json` + `.credentials.json`) from an
//!   existing home so cloud commands can run without re-login.

#![allow(dead_code)]

use std::{
    env,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

use codex::{AuthSeedOptions, CodexClient, CodexHomeLayout};

pub const ENV_BINARY: &str = "CODEX_BINARY";
pub const ENV_HOME: &str = "CODEX_HOME";

pub const ENV_EXAMPLE_ISOLATED_HOME: &str = "CODEX_EXAMPLE_ISOLATED_HOME";
pub const ENV_EXAMPLE_SEED_AUTH: &str = "CODEX_EXAMPLE_SEED_AUTH";

pub fn resolve_binary() -> PathBuf {
    // Prefer explicit env override.
    if let Some(binary) = env::var_os(ENV_BINARY) {
        return PathBuf::from(binary);
    }

    // Prefer a pinned parity binary when present.
    let repo_root = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let pinned = repo_root.join(".codex-bins/0.92.0/codex-x86_64-unknown-linux-musl");
    if pinned.is_file() {
        return pinned;
    }

    PathBuf::from("codex")
}

pub fn default_client() -> CodexClient {
    CodexClient::builder()
        .binary(resolve_binary())
        .mirror_stdout(false)
        .quiet(true)
        .build()
}

pub fn isolated_home_root(example_name: &str) -> PathBuf {
    let repo_root = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let target = repo_root.join("target");
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    target.join(format!(
        "codex-example-home-{}-{}-{}",
        example_name,
        std::process::id(),
        now
    ))
}

pub fn build_client_with_home(home: &Path) -> CodexClient {
    CodexClient::builder()
        .binary(resolve_binary())
        .codex_home(home)
        .create_home_dirs(true)
        .mirror_stdout(false)
        .quiet(true)
        .build()
}

pub fn maybe_seed_auth(
    target_home: &Path,
    seed_home: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    let layout = CodexHomeLayout::new(target_home.to_path_buf());
    layout.materialize(true)?;
    layout
        .seed_auth_from(
            seed_home,
            AuthSeedOptions {
                require_auth: false,
                require_credentials: false,
                ..AuthSeedOptions::default()
            },
        )
        .map(|_| ())
        .map_err(|err| err.into())
}

pub fn wants_isolated_home() -> bool {
    matches!(
        env::var(ENV_EXAMPLE_ISOLATED_HOME).ok().as_deref(),
        Some("1") | Some("true") | Some("yes")
    )
}

pub fn wants_seed_auth() -> bool {
    matches!(
        env::var(ENV_EXAMPLE_SEED_AUTH).ok().as_deref(),
        Some("1") | Some("true") | Some("yes")
    )
}
