use anyhow::Result;
use world_agent::run_world_agent;

#[tokio::main]
async fn main() -> Result<()> {
    run_world_agent().await
}
