use anyhow::Result;

#[cfg(unix)]
use host_proxy::run_host_proxy;

#[cfg(unix)]
#[tokio::main]
async fn main() -> Result<()> {
    run_host_proxy().await
}

#[cfg(not(unix))]
fn main() -> Result<()> {
    Err(anyhow::anyhow!(
        "host-proxy binary is not supported on this platform"
    ))
}
