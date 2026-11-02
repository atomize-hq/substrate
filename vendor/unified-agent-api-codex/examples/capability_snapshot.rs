use std::{env, path::PathBuf, time::Duration};

use codex::{
    capability_snapshot_matches_binary, read_capabilities_snapshot, write_capabilities_snapshot,
    CapabilityCachePolicy, CodexClient,
};

/// Demonstrates disk snapshot reuse with fingerprint checks and cache policies.
///
/// Usage:
/// `cargo run -p unified-agent-api-codex --example capability_snapshot -- [binary_path] [snapshot_path] [auto|refresh|bypass]`
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = env::args().skip(1);
    let binary_path = args
        .next()
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("codex"));
    let snapshot_path = args
        .next()
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("codex-capabilities.json"));
    let mode = args.next().unwrap_or_else(|| "auto".to_string());

    let loaded_snapshot = read_capabilities_snapshot(&snapshot_path, None)
        .ok()
        .and_then(|snapshot| {
            if capability_snapshot_matches_binary(&snapshot, &binary_path) {
                println!(
                    "Reusing cached snapshot from {} collected at {:?}",
                    snapshot_path.display(),
                    snapshot.collected_at
                );
                Some(snapshot)
            } else {
                println!(
                    "Skipping cached snapshot at {} because the fingerprint does not match the binary.",
                    snapshot_path.display()
                );
                None
            }
        });

    let mut builder = CodexClient::builder().binary(binary_path.clone());
    if let Some(snapshot) = loaded_snapshot.clone() {
        builder = builder.capability_snapshot(snapshot);
    }

    let mut policy = match mode.as_str() {
        "refresh" => CapabilityCachePolicy::Refresh,
        "bypass" => CapabilityCachePolicy::Bypass,
        _ => CapabilityCachePolicy::PreferCache,
    };

    // In auto mode, refresh stale snapshots even when fingerprints match
    // and bypass caches when metadata is missing (common on FUSE/overlay filesystems).
    if matches!(policy, CapabilityCachePolicy::PreferCache) {
        const SNAPSHOT_TTL: Duration = Duration::from_secs(300);

        let is_stale = loaded_snapshot
            .as_ref()
            .and_then(|snapshot| snapshot.collected_at.elapsed().ok())
            .map(|age| age > SNAPSHOT_TTL)
            .unwrap_or(true);
        if is_stale {
            println!("Refreshing capabilities because the cached snapshot exceeded the TTL.");
            policy = CapabilityCachePolicy::Refresh;
        }

        if loaded_snapshot
            .as_ref()
            .map(|snapshot| snapshot.fingerprint.is_none())
            .unwrap_or(false)
        {
            println!(
                "Bypassing the cache because fingerprint metadata is missing (likely a FUSE/overlay path)."
            );
            policy = CapabilityCachePolicy::Bypass;
        }
    }

    let client = builder.capability_cache_policy(policy).build();
    let capabilities = client.probe_capabilities_with_policy(policy).await;

    if capabilities.fingerprint.is_none() {
        println!("Fingerprint metadata is missing; skipping disk persistence to avoid reusing stale data.");
    } else {
        write_capabilities_snapshot(&snapshot_path, &capabilities, None)?;
        println!("Wrote refreshed snapshot to {}", snapshot_path.display());
    }

    println!(
        "Cache policy: {:?} | Probed steps: {:?}",
        policy, capabilities.probe_plan.steps
    );

    Ok(())
}
