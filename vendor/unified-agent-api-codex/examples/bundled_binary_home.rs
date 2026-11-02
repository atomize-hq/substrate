//! Resolve a pinned bundled Codex binary, pick a per-project `CODEX_HOME`, and
//! (optionally) seed credentials from an app-owned home. The helper never
//! consults `CODEX_BINARY` or `PATH`, keeping the flow isolated from user
//! installs.
//!
//! Bundle layout: `<bundle_root>/<platform>/<version>/<codex|codex.exe>`.
//! `CODEX_BUNDLE_PLATFORM` is optional and defaults to the current target
//! (`darwin-arm64`, `linux-x64`, `windows-x64`).
//!
//! Example run (env vars are only used by this example):
//! ```bash
//! CODEX_BUNDLE_ROOT="$HOME/.myapp/codex-bin" \
//! CODEX_BUNDLE_VERSION="1.2.3" \
//! CODEX_PROJECT_HOME="$HOME/.myapp/codex-homes/demo-workspace" \
//! CODEX_AUTH_SEED_HOME="$HOME/.myapp/codex-auth-seed" \
//! cargo run -p unified-agent-api-codex --example bundled_binary_home -- "Health check prompt"
//! ```
//! `CODEX_AUTH_SEED_HOME` is optional; when set, only `auth.json` and
//! `.credentials.json` are copied. Avoid copying history/logs between project
//! homes.

use codex::{
    resolve_bundled_binary, AuthSeedOptions, AuthSessionHelper, BundledBinarySpec, CodexClient,
    CodexHomeLayout,
};
use std::{env, error::Error, path::PathBuf};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let prompt = collect_prompt();
    let bundle_root = require_env_path("CODEX_BUNDLE_ROOT")?;
    let version = require_env_string("CODEX_BUNDLE_VERSION")?;
    let platform = env::var("CODEX_BUNDLE_PLATFORM").ok();
    let codex_home = require_env_path("CODEX_PROJECT_HOME")?;
    let auth_seed = env::var_os("CODEX_AUTH_SEED_HOME").map(PathBuf::from);

    let bundled = resolve_bundled_binary(BundledBinarySpec {
        bundle_root: bundle_root.as_path(),
        version: &version,
        platform: platform.as_deref(),
    })?;
    let layout = CodexHomeLayout::new(&codex_home);
    layout.materialize(true)?;
    if let Some(seed_home) = auth_seed.as_deref() {
        let seeded = layout.seed_auth_from(
            seed_home,
            AuthSeedOptions {
                require_auth: true,
                ..Default::default()
            },
        )?;
        println!(
            "Seeded auth.json? {} | .credentials.json? {}",
            seeded.copied_auth, seeded.copied_credentials
        );
    }

    println!("Bundled binary: {}", bundled.binary_path.display());
    println!("CODEX_HOME: {}", layout.root().display());
    println!("Conversations: {}", layout.conversations_dir().display());
    println!("Logs: {}", layout.logs_dir().display());
    println!("Auth file: {}", layout.auth_path().display());
    println!("Credentials file: {}", layout.credentials_path().display());

    let client = CodexClient::builder()
        .binary(&bundled.binary_path)
        .codex_home(layout.root())
        .create_home_dirs(true)
        .build();
    let auth = AuthSessionHelper::with_client(client.clone());
    let status = auth.status().await?;
    println!("Auth status under CODEX_HOME: {status:?}");

    if let Ok(api_key) = env::var("CODEX_API_KEY") {
        let updated = auth.ensure_api_key_login(api_key).await?;
        println!("Auth status after ensure_api_key_login: {updated:?}");
    } else {
        println!(
            "Set CODEX_API_KEY to refresh login under {}",
            layout.root().display()
        );
    }

    let response = client.send_prompt(&prompt).await?;
    println!("{response}");
    Ok(())
}

fn collect_prompt() -> String {
    let args: Vec<String> = env::args().skip(1).collect();
    if args.is_empty() {
        "Health check prompt".to_string()
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
        _ => Err(format!("Set {key} to configure the bundled binary/home paths").into()),
    }
}
