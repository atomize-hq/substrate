use anyhow::{Context, Result};
use world_agent::{internal_exec, run_world_agent};

fn main() -> Result<()> {
    if std::env::args()
        .nth(1)
        .is_some_and(|arg| arg == internal_exec::LANDLOCK_EXEC_ARG)
    {
        return internal_exec::run_landlock_exec();
    }

    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .context("failed to build tokio runtime")?;

    runtime.block_on(run_world_agent())
}
