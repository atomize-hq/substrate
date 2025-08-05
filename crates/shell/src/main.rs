use anyhow::Result;
use substrate_shell::run_shell;

fn main() -> Result<()> {
    let exit_code = run_shell()?;
    std::process::exit(exit_code);
}