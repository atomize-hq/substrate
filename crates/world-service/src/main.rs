use anyhow::Result;
use world_service::{internal_exec, run_world_service};

#[tokio::main(flavor = "multi_thread")]
async fn main() -> Result<()> {
    if std::env::args()
        .nth(1)
        .is_some_and(|arg| arg == internal_exec::LANDLOCK_EXEC_ARG)
    {
        return internal_exec::run_landlock_exec();
    }

    run_world_service().await
}
