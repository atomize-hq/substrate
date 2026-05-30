use anyhow::Context;
use camino::Utf8PathBuf;
use clap::Parser;
use time::OffsetDateTime;

use crate::RunConfig;

#[derive(Debug, Clone, Parser)]
#[command(
    name = "agent-session-compactor",
    bin_name = "agent-session-compactor",
    version,
    about = "Discover and compact historical Codex session artifacts"
)]
pub struct Cli {
    #[arg(long)]
    pub codex_home: Option<Utf8PathBuf>,
    #[arg(long)]
    pub session_id: Option<String>,
    #[arg(long)]
    pub output_dir: Utf8PathBuf,
}

impl Cli {
    pub fn run_config(&self) -> RunConfig {
        RunConfig {
            codex_home: self.codex_home.clone(),
            session_id: self.session_id.clone(),
            output_dir: self.output_dir.clone(),
            generated_at: Some(OffsetDateTime::now_utc()),
        }
    }
}

pub fn run() -> anyhow::Result<()> {
    let cli = Cli::parse();
    run_with_cli(cli)
}

pub fn run_with_cli(cli: Cli) -> anyhow::Result<()> {
    crate::compact_codex_sessions(&cli.run_config())
        .context("failed to compact Codex session artifacts")?;
    Ok(())
}
