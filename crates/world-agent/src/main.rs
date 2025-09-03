//! World agent binary - runs inside worlds/VMs to provide execution API.

use anyhow::{Context, Result};
use axum::routing::{get, post};
use axum::Router;
use hyperlocal::UnixServerExt;
use std::path::Path;
use tracing::info;
use tracing_subscriber::{fmt, EnvFilter};

mod handlers;
mod pty;
mod service;

use service::WorldAgentService;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

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

    // Create service instance
    let service = WorldAgentService::new()?;

    // Build router with agent-api-core
    let app = Router::new()
        .route("/v1/capabilities", get(handlers::capabilities))
        .route("/v1/execute", post(handlers::execute))
        .route("/v1/stream", post(handlers::stream))
        .route("/v1/trace/:span_id", get(handlers::get_trace))
        .route("/v1/request_scopes", post(handlers::request_scopes))
        .with_state(service);

    info!("Routes registered, starting server...");
    info!("World agent listening on {}", socket_path.display());

    // Serve the application over Unix socket using hyperlocal
    hyper::Server::bind_unix(socket_path)?
        .serve(app.into_make_service())
        .await
        .context("Server failed")?;

    Ok(())
}
