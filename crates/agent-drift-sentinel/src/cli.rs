use anyhow::{bail, Context};
use camino::Utf8PathBuf;
use clap::{Parser, ValueEnum};

use crate::{
    execute, AdjudicationConfig, CheckpointCursor, ReasoningEffort, SchedulerPolicy, SentinelMode,
    SentinelRequest, WarningPolicy,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
enum ModeArg {
    Replay,
    Live,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
enum ReasoningEffortArg {
    Minimal,
    Low,
    Medium,
    High,
}

impl From<ReasoningEffortArg> for ReasoningEffort {
    fn from(value: ReasoningEffortArg) -> Self {
        match value {
            ReasoningEffortArg::Minimal => Self::Minimal,
            ReasoningEffortArg::Low => Self::Low,
            ReasoningEffortArg::Medium => Self::Medium,
            ReasoningEffortArg::High => Self::High,
        }
    }
}

#[derive(Debug, Parser)]
#[command(name = "agent-drift-sentinel")]
#[command(about = "Replay-first drift sentinel over analyzer checkpoints")]
pub struct Cli {
    #[arg(long)]
    checkpoint_dir: Utf8PathBuf,
    #[arg(long, value_enum, default_value_t = ModeArg::Replay)]
    mode: ModeArg,
    #[arg(long, requires = "cursor_ordinal")]
    cursor_session_id: Option<String>,
    #[arg(long, requires = "cursor_session_id")]
    cursor_ordinal: Option<usize>,
    #[arg(long, default_value_t = SchedulerPolicy::default().checkpoint_cooldown)]
    checkpoint_cooldown: usize,
    #[arg(long, default_value_t = SchedulerPolicy::default().heartbeat_interval)]
    heartbeat_interval: usize,
    #[arg(long, default_value_t = SchedulerPolicy::default().warning_debounce)]
    warning_debounce: usize,
    #[arg(long, default_value_t = SchedulerPolicy::default().repeated_failure_threshold)]
    repeated_failure_threshold: usize,
    #[arg(long, default_value_t = WarningPolicy::default().minimum_visible_score)]
    minimum_visible_score: u8,
    #[arg(long, default_value_t = WarningPolicy::default().max_evidence_lines)]
    max_evidence_lines: usize,
    #[arg(long, default_value_t = false)]
    enable_model_adjudication: bool,
    #[arg(long, default_value = "gpt-5.4-mini")]
    model: String,
    #[arg(long, value_enum, default_value_t = ReasoningEffortArg::Medium)]
    reasoning_effort: ReasoningEffortArg,
}

pub fn run() -> anyhow::Result<()> {
    let args = Cli::parse();
    let request = args.into_request()?;
    let result = execute(&request).context("agent drift sentinel execution failed")?;
    println!("{}", result.report.to_console_text());
    if !result.adjudication_requests.is_empty() {
        println!();
        println!("Prepared model adjudication requests:");
        for request in &result.adjudication_requests {
            println!(
                "- {} [{} / {}]",
                request.checkpoint_id, request.model, request.reasoning_effort
            );
        }
    }
    Ok(())
}

impl Cli {
    fn into_request(self) -> anyhow::Result<SentinelRequest> {
        if self.mode == ModeArg::Live {
            bail!("live mode remains gated by S10; replay usefulness review must pass first");
        }

        let cursor =
            self.cursor_session_id
                .zip(self.cursor_ordinal)
                .map(|(session_id, ordinal)| CheckpointCursor {
                    session_id,
                    ordinal,
                });

        Ok(SentinelRequest {
            checkpoint_dir: self.checkpoint_dir,
            mode: match self.mode {
                ModeArg::Replay => SentinelMode::Replay,
                ModeArg::Live => SentinelMode::Live,
            },
            cursor,
            scheduler_policy: SchedulerPolicy {
                checkpoint_cooldown: self.checkpoint_cooldown,
                heartbeat_interval: self.heartbeat_interval,
                warning_debounce: self.warning_debounce,
                repeated_failure_threshold: self.repeated_failure_threshold,
            },
            warning_policy: WarningPolicy {
                minimum_visible_score: self.minimum_visible_score,
                max_evidence_lines: self.max_evidence_lines,
                max_objective_chars: WarningPolicy::default().max_objective_chars,
            },
            adjudication: AdjudicationConfig {
                enabled: self.enable_model_adjudication,
                model: self.model,
                reasoning_effort: self.reasoning_effort.into(),
                ..AdjudicationConfig::default()
            },
        })
    }
}
