//! World agent binary - runs inside worlds/VMs to provide execution API.

use anyhow::{Context, Result};
use axum::routing::{get, post};
use axum::Router;
use hyperlocal::UnixServerExt;
use std::path::Path;
use tracing::{info, warn};
use tracing_subscriber::{fmt, EnvFilter};

mod gc;
mod handlers;
mod pty;
mod service;

use service::WorldAgentService;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    fmt().with_env_filter(EnvFilter::from_default_env()).init();

    info!("Starting Substrate World Agent");

    // Ensure socket directory exists
    let socket_path = Path::new("/run/substrate.sock");
    if let Some(parent) = socket_path.parent() {
        std::fs::create_dir_all(parent).context("Failed to create socket directory")?;
    }

    // Remove existing socket if present
    if socket_path.exists() {
        std::fs::remove_file(socket_path).context("Failed to remove existing socket")?;
    }

    // Run initial GC sweep on startup
    info!("Running initial netns GC sweep");
    let ttl = get_env_u64("SUBSTRATE_NETNS_GC_TTL_SECS", 0)
        .filter(|&t| t > 0)
        .map(std::time::Duration::from_secs);

    match gc::sweep(ttl).await {
        Ok(report) => {
            info!(
                "Initial GC sweep complete: removed={}, kept={}, errors={}",
                report.removed.len(),
                report.kept.len(),
                report.errors.len()
            );
        }
        Err(e) => {
            warn!("Initial GC sweep failed: {}", e);
        }
    }

    // Create service instance
    let service = WorldAgentService::new()?;

    // Build router with agent-api-core
    let app = Router::new()
        .route("/v1/capabilities", get(handlers::capabilities))
        .route("/v1/execute", post(handlers::execute))
        .route("/v1/stream", get(handlers::stream))
        .route("/v1/trace/:span_id", get(handlers::get_trace))
        .route("/v1/request_scopes", post(handlers::request_scopes))
        .route("/v1/gc", post(handlers::gc))
        .with_state(service);

    info!("Routes registered, starting server...");
    info!("World agent listening on {}", socket_path.display());

    // Start periodic GC if enabled
    let gc_interval_secs = get_env_u64("SUBSTRATE_NETNS_GC_INTERVAL_SECS", 600).unwrap_or(600);
    if gc_interval_secs > 0 {
        info!("Starting periodic GC sweep every {} seconds", gc_interval_secs);
        spawn_periodic_gc(gc_interval_secs);
    } else {
        info!("Periodic GC sweep disabled");
    }

    // Serve the application over Unix socket using hyperlocal
    hyper::Server::bind_unix(socket_path)?
        .serve(app.into_make_service())
        .await
        .context("Server failed")?;

    Ok(())
}

fn get_env_u64(key: &str, default: u64) -> Option<u64> {
    std::env::var(key)
        .ok()
        .and_then(|s| s.parse::<u64>().ok())
        .or(Some(default))
}

fn spawn_periodic_gc(interval_secs: u64) {
    use tokio::time::{interval, Duration};

    tokio::spawn(async move {
        let base_interval = Duration::from_secs(interval_secs);
        let jitter_range = (interval_secs as f64 * 0.1) as u64;

        let mut interval = interval(base_interval);
        interval.tick().await; // Skip first immediate tick

        loop {
            interval.tick().await;

            // Add jitter - create RNG inside the async block
            let jitter = if jitter_range > 0 {
                use rand::Rng;
                let mut rng = rand::thread_rng();
                Duration::from_secs(rng.gen_range(0..jitter_range))
            } else {
                Duration::ZERO
            };

            tokio::time::sleep(jitter).await;

            // Read TTL for each sweep
            let ttl = get_env_u64("SUBSTRATE_NETNS_GC_TTL_SECS", 0)
                .filter(|&t| t > 0)
                .map(Duration::from_secs);

            info!("Starting periodic GC sweep");
            match gc::sweep(ttl).await {
                Ok(report) => {
                    info!(
                        "Periodic GC sweep complete: removed={}, kept={}, errors={}",
                        report.removed.len(),
                        report.kept.len(),
                        report.errors.len()
                    );
                }
                Err(e) => {
                    warn!("Periodic GC sweep failed: {}", e);
                }
            }
        }
    });
}
