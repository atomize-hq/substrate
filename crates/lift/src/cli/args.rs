use clap::{Args, Parser, Subcommand};

use crate::error::LiftError;

#[derive(Debug, Parser)]
#[command(
    name = "lift",
    bin_name = "lift",
    version,
    about = "Deterministic code-intelligence engine CLI scaffold"
)]
pub(crate) struct Cli {
    #[command(subcommand)]
    pub(crate) command: Option<Command>,
}

#[derive(Debug, Subcommand)]
pub(crate) enum Command {
    Score(ScoreArgs),
    Impact(ImpactArgs),
    Policy(PolicyArgs),
    Contract(ContractArgs),
    Context(ContextArgs),
    Index(IndexArgs),
    Query(QueryArgs),
    Rewrite(RewriteArgs),
}

#[derive(Debug, Args)]
pub(crate) struct ScoreArgs {}

#[derive(Debug, Args)]
pub(crate) struct ImpactArgs {}

#[derive(Debug, Args)]
pub(crate) struct PolicyArgs {}

#[derive(Debug, Args)]
pub(crate) struct ContractArgs {}

#[derive(Debug, Args)]
pub(crate) struct ContextArgs {}

#[derive(Debug, Args)]
pub(crate) struct IndexArgs {}

#[derive(Debug, Args)]
pub(crate) struct QueryArgs {}

#[derive(Debug, Args)]
pub(crate) struct RewriteArgs {}

pub(crate) fn dispatch(cli: Cli) -> Result<(), LiftError> {
    match cli.command {
        Some(Command::Score(_)) => Err(LiftError::NotImplemented("lift score")),
        Some(Command::Impact(_)) => Err(LiftError::NotImplemented("lift impact")),
        Some(Command::Policy(_)) => Err(LiftError::NotImplemented("lift policy")),
        Some(Command::Contract(_)) => Err(LiftError::NotImplemented("lift contract")),
        Some(Command::Context(_)) => Err(LiftError::NotImplemented("lift context")),
        Some(Command::Index(_)) => Err(LiftError::NotImplemented("lift index")),
        Some(Command::Query(_)) => Err(LiftError::NotImplemented("lift query")),
        Some(Command::Rewrite(_)) => Err(LiftError::NotImplemented("lift rewrite")),
        None => Err(LiftError::NotImplemented("lift")),
    }
}
