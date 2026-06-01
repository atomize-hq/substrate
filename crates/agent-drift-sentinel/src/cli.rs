use std::thread;
use std::time::Duration;

use anyhow::{bail, Context};
use camino::Utf8PathBuf;
use clap::{Parser, ValueEnum};

use crate::{
    execute, AdjudicationConfig, CheckpointCursor, LiveSessionCoordinator, LiveSessionRequest,
    ReasoningEffort, SchedulerPolicy, SentinelMode, SentinelRequest, WarningPolicy,
};

const LIVE_POLL_INTERVAL: Duration = Duration::from_millis(250);

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
    #[arg(long)]
    codex_home: Option<Utf8PathBuf>,
    #[arg(long)]
    session_id: Option<String>,
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
    match args.into_command()? {
        Command::Replay(request) => {
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
        }
        Command::Live(request, scheduler_policy, warning_policy) => {
            run_live(request, scheduler_policy, warning_policy)?;
        }
    }
    Ok(())
}

impl Cli {
    fn into_command(self) -> anyhow::Result<Command> {
        let cursor =
            self.cursor_session_id
                .zip(self.cursor_ordinal)
                .map(|(session_id, ordinal)| CheckpointCursor {
                    session_id,
                    ordinal,
                });

        let scheduler_policy = SchedulerPolicy {
            checkpoint_cooldown: self.checkpoint_cooldown,
            heartbeat_interval: self.heartbeat_interval,
            warning_debounce: self.warning_debounce,
            repeated_failure_threshold: self.repeated_failure_threshold,
        };
        let warning_policy = WarningPolicy {
            minimum_visible_score: self.minimum_visible_score,
            max_evidence_lines: self.max_evidence_lines,
            max_objective_chars: WarningPolicy::default().max_objective_chars,
        };

        match self.mode {
            ModeArg::Replay => {
                if self.codex_home.is_some() || self.session_id.is_some() {
                    bail!("--codex-home and --session-id are only valid with --mode live");
                }

                Ok(Command::Replay(SentinelRequest {
                    checkpoint_dir: self.checkpoint_dir,
                    mode: SentinelMode::Replay,
                    cursor,
                    scheduler_policy,
                    warning_policy,
                    adjudication: AdjudicationConfig {
                        enabled: self.enable_model_adjudication,
                        model: self.model,
                        reasoning_effort: self.reasoning_effort.into(),
                        ..AdjudicationConfig::default()
                    },
                }))
            }
            ModeArg::Live => {
                if cursor.is_some() {
                    bail!("--cursor-session-id and --cursor-ordinal are only valid with --mode replay");
                }
                let session_id = self
                    .session_id
                    .filter(|value| !value.trim().is_empty())
                    .ok_or_else(|| anyhow::anyhow!("--session-id is required with --mode live"))?;
                Ok(Command::Live(
                    LiveSessionRequest {
                        codex_home: self.codex_home,
                        session_id,
                        state_dir: self.checkpoint_dir,
                    },
                    scheduler_policy,
                    warning_policy,
                ))
            }
        }
    }
}

#[derive(Debug)]
enum Command {
    Replay(SentinelRequest),
    Live(LiveSessionRequest, SchedulerPolicy, WarningPolicy),
}

fn run_live(
    request: LiveSessionRequest,
    scheduler_policy: SchedulerPolicy,
    warning_policy: WarningPolicy,
) -> anyhow::Result<()> {
    let session_id = request.session_id.clone();
    let state_dir = request.state_dir.clone();
    let mut coordinator = LiveSessionCoordinator::new(request, scheduler_policy, warning_policy)
        .context("agent drift sentinel live setup failed")?;

    println!("# Agent Drift Sentinel Live");
    println!();
    println!("Session: `{session_id}`");
    println!(
        "Rollout artifact: `{}`",
        coordinator.rollout_path().as_str()
    );
    println!("State dir: `{}`", state_dir.as_str());

    loop {
        let poll = coordinator
            .poll_once()
            .context("agent drift sentinel live poll failed")?;
        if poll.reran_pipeline {
            println!();
            println!(
                "Observed rollout update at `{}` ({} bytes); emitted `{}` new checkpoint(s).",
                poll.rollout_path.as_str(),
                poll.observed_size_bytes,
                poll.emitted_checkpoints
            );
            for observation in &poll.observations {
                println!();
                println!("{}", observation.presentation.render_console_block(None));
            }
        }
        thread::sleep(LIVE_POLL_INTERVAL);
    }
}

#[cfg(test)]
mod tests {
    use clap::Parser;

    use super::{Cli, Command, ModeArg};

    #[test]
    fn live_mode_accepts_session_id() {
        let cli = Cli::parse_from(["agent-drift-sentinel", "--checkpoint-dir", "/tmp/live"]);
        assert!(matches!(cli.mode, ModeArg::Replay));

        let cli = Cli::parse_from([
            "agent-drift-sentinel",
            "--mode",
            "live",
            "--checkpoint-dir",
            "/tmp/live",
            "--session-id",
            "session-123",
        ]);
        let command = cli.into_command().expect("live command");
        assert!(matches!(command, Command::Live(..)));
    }

    #[test]
    fn live_mode_requires_session_id() {
        let cli = Cli::parse_from([
            "agent-drift-sentinel",
            "--mode",
            "live",
            "--checkpoint-dir",
            "/tmp/live",
        ]);
        let error = cli
            .into_command()
            .expect_err("missing session id should fail");
        assert!(error
            .to_string()
            .contains("--session-id is required with --mode live"));
    }

    #[test]
    fn replay_mode_rejects_live_only_flags() {
        let cli = Cli::parse_from([
            "agent-drift-sentinel",
            "--checkpoint-dir",
            "/tmp/replay",
            "--session-id",
            "session-123",
        ]);
        let error = cli
            .into_command()
            .expect_err("replay should reject live flags");
        assert!(error
            .to_string()
            .contains("--codex-home and --session-id are only valid with --mode live"));
    }
}
