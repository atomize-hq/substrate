use clap::{Args, Parser, Subcommand};

use crate::error::LiftError;

#[derive(Debug, Parser)]
#[command(
    name = "lift",
    version,
    about = "Deterministic lift scoring engine and CLI scaffold"
)]
pub(crate) struct Cli {
    #[command(subcommand)]
    pub(crate) command: Option<Command>,
}

#[derive(Debug, Subcommand)]
pub(crate) enum Command {
    #[command(subcommand)]
    Score(ScoreCommand),
    #[command(subcommand)]
    Estimate(EstimateCommand),
    #[command(subcommand)]
    Analyze(AnalyzeCommand),
    Explain(ExplainArgs),
    #[command(subcommand)]
    Validate(ValidateCommand),
    PrintSchema(PrintSchemaArgs),
    PrintModel(PrintModelArgs),
}

#[derive(Debug, Subcommand)]
pub(crate) enum ScoreCommand {
    Vector(ScoreVectorArgs),
    Diff(ScoreDiffArgs),
}

#[derive(Debug, Subcommand)]
pub(crate) enum EstimateCommand {
    Path(EstimatePathArgs),
    Symbol(EstimateSymbolArgs),
}

#[derive(Debug, Subcommand)]
pub(crate) enum AnalyzeCommand {
    Path(AnalyzePathArgs),
    Symbol(AnalyzeSymbolArgs),
}

#[derive(Debug, Subcommand)]
pub(crate) enum ValidateCommand {
    Vector(ValidateVectorArgs),
    Config(ValidateConfigArgs),
}

#[derive(Debug, Args)]
pub(crate) struct ScoreVectorArgs {}

#[derive(Debug, Args)]
pub(crate) struct ScoreDiffArgs {}

#[derive(Debug, Args)]
pub(crate) struct EstimatePathArgs {}

#[derive(Debug, Args)]
pub(crate) struct EstimateSymbolArgs {}

#[derive(Debug, Args)]
pub(crate) struct AnalyzePathArgs {}

#[derive(Debug, Args)]
pub(crate) struct AnalyzeSymbolArgs {}

#[derive(Debug, Args)]
pub(crate) struct ExplainArgs {}

#[derive(Debug, Args)]
pub(crate) struct ValidateVectorArgs {}

#[derive(Debug, Args)]
pub(crate) struct ValidateConfigArgs {}

#[derive(Debug, Args)]
pub(crate) struct PrintSchemaArgs {}

#[derive(Debug, Args)]
pub(crate) struct PrintModelArgs {}

pub(crate) fn dispatch(cli: Cli) -> Result<(), LiftError> {
    match cli.command {
        Some(Command::Score(ScoreCommand::Vector(_))) => {
            Err(LiftError::NotImplemented("lift score vector"))
        }
        Some(Command::Score(ScoreCommand::Diff(_))) => {
            Err(LiftError::NotImplemented("lift score diff"))
        }
        Some(Command::Estimate(EstimateCommand::Path(_))) => {
            Err(LiftError::NotImplemented("lift estimate path"))
        }
        Some(Command::Estimate(EstimateCommand::Symbol(_))) => {
            Err(LiftError::NotImplemented("lift estimate symbol"))
        }
        Some(Command::Analyze(AnalyzeCommand::Path(_))) => {
            Err(LiftError::NotImplemented("lift analyze path"))
        }
        Some(Command::Analyze(AnalyzeCommand::Symbol(_))) => {
            Err(LiftError::NotImplemented("lift analyze symbol"))
        }
        Some(Command::Explain(_)) => Err(LiftError::NotImplemented("lift explain")),
        Some(Command::Validate(ValidateCommand::Vector(_))) => {
            Err(LiftError::NotImplemented("lift validate vector"))
        }
        Some(Command::Validate(ValidateCommand::Config(_))) => {
            Err(LiftError::NotImplemented("lift validate config"))
        }
        Some(Command::PrintSchema(_)) => Err(LiftError::NotImplemented("lift print-schema")),
        Some(Command::PrintModel(_)) => Err(LiftError::NotImplemented("lift print-model")),
        None => Err(LiftError::NotImplemented("lift")),
    }
}
