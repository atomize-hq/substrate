//! Substrate command shim - intercepts and logs command execution
//!
//! This binary is copied to multiple names in ~/.cmdshim_rust/ to intercept
//! different commands. It resolves the real binary from a clean PATH and
//! executes it while logging the invocation.

use anyhow::Result;
use std::process::ExitCode;
use substrate_shim::run_shim;

fn main() -> ExitCode {
    match run_main() {
        Ok(code) => ExitCode::from(code as u8),
        Err(e) => {
            eprintln!("shim error: {e:?}");
            ExitCode::from(126)
        }
    }
}

fn run_main() -> Result<i32> {
    let exit_code = run_shim()?;
    Ok(exit_code)
}
