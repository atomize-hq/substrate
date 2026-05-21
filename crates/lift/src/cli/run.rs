use clap::Parser;

use crate::error::LiftError;

pub fn run() -> Result<(), LiftError> {
    let cli = crate::cli::args::Cli::parse();
    crate::cli::args::dispatch(cli)
}
