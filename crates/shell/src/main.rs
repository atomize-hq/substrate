use anyhow::Result;
use substrate_shell::run_shell;

fn main() -> Result<()> {
    // Initialize logger early for debugging (non-fatal to avoid panics)
    let _ = env_logger::try_init();
    
    let exit_code = run_shell()?;
    std::process::exit(exit_code);
}