use anyhow::Result;
use world_agent::{internal_exec, run_world_agent};

#[tokio::main(flavor = "multi_thread")]
async fn main() -> Result<()> {
    if std::env::args()
        .nth(1)
        .is_some_and(|arg| arg == internal_exec::LANDLOCK_EXEC_ARG)
    {
        return internal_exec::run_landlock_exec();
    }

    run_world_agent().await
}
