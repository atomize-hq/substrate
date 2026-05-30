use anyhow::Context;
use camino::Utf8PathBuf;
use clap::Parser;

use crate::{analyze_bundle, AnalyzeRequest};

#[derive(Debug, Parser)]
#[command(name = "agent-drift-analyzer")]
#[command(about = "Analyze compactor bundles and emit deterministic drift checkpoints")]
pub struct Cli {
    #[arg(long)]
    input_dir: Utf8PathBuf,
    #[arg(long)]
    output_dir: Utf8PathBuf,
}

pub fn run() -> anyhow::Result<()> {
    let args = Cli::parse();
    analyze_bundle(&AnalyzeRequest {
        input_dir: args.input_dir,
        output_dir: args.output_dir,
    })
    .context("agent drift analysis failed")?;
    Ok(())
}
