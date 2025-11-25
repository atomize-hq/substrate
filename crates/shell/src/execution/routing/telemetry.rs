//! Telemetry helpers for routing, including REPL metrics and command logging.

use super::super::{ShellConfig, ShellMode};
use anyhow::Result;
use chrono::Utc;
use serde_json::json;
use std::env;
use std::io::{self, IsTerminal};
use std::sync::Arc;
use substrate_common::{
    agent_events::{AgentEvent, AgentEventKind},
    log_schema,
};
use substrate_trace::{append_to_trace, init_trace};
use tracing::{info, warn};

pub(crate) const SHELL_AGENT_ID: &str = "shell";

pub(crate) fn is_shell_stream_event(event: &AgentEvent) -> bool {
    event.agent_id == SHELL_AGENT_ID && matches!(event.kind, AgentEventKind::PtyData)
}

#[derive(Default, Debug, Clone)]
pub(crate) struct ReplMetrics {
    input_events: u64,
    agent_events: u64,
    commands_executed: u64,
}

pub(crate) struct ReplSessionTelemetry {
    config: Arc<ShellConfig>,
    mode: &'static str,
    metrics: ReplMetrics,
}

impl ReplSessionTelemetry {
    pub(crate) fn new(config: Arc<ShellConfig>, mode: &'static str) -> Self {
        let telemetry = Self {
            config,
            mode,
            metrics: ReplMetrics::default(),
        };

        if let Err(err) = log_repl_event(&telemetry.config, mode, "start", None) {
            warn!(target = "substrate::shell", error = %err, "failed to append REPL start event");
        }

        telemetry
    }

    pub(crate) fn record_input_event(&mut self) {
        self.metrics.input_events = self.metrics.input_events.saturating_add(1);
    }

    pub(crate) fn record_agent_event(&mut self) {
        self.metrics.agent_events = self.metrics.agent_events.saturating_add(1);
    }

    pub(crate) fn record_command(&mut self) {
        self.metrics.commands_executed = self.metrics.commands_executed.saturating_add(1);
    }
}

impl Drop for ReplSessionTelemetry {
    fn drop(&mut self) {
        if let Err(err) = log_repl_event(&self.config, self.mode, "stop", Some(&self.metrics)) {
            warn!(target = "substrate::shell", error = %err, "failed to append REPL stop event");
        }
    }
}

fn log_repl_event(
    config: &ShellConfig,
    mode: &str,
    stage: &str,
    metrics: Option<&ReplMetrics>,
) -> Result<()> {
    let mut entry = json!({
        log_schema::TIMESTAMP: Utc::now().to_rfc3339(),
        log_schema::EVENT_TYPE: "repl_status",
        log_schema::SESSION_ID: config.session_id,
        log_schema::COMPONENT: "shell",
        "stage": stage,
        "repl_mode": mode,
        "ci_mode": config.ci_mode,
        "async_enabled": config.async_repl,
        "no_world": config.no_world,
        "shell": config.shell_path,
    });

    if let ShellMode::Interactive { use_pty } = &config.mode {
        entry["interactive_use_pty"] = json!(*use_pty);
    }

    let (input_events, agent_events, commands_executed) = metrics
        .map(|m| (m.input_events, m.agent_events, m.commands_executed))
        .unwrap_or((0, 0, 0));

    if metrics.is_some() {
        entry["metrics"] = json!({
            "input_events": input_events,
            "agent_events": agent_events,
            "commands_executed": commands_executed,
        });
    }

    let _ = init_trace(None);
    append_to_trace(&entry)?;

    info!(
        target = "substrate::shell",
        repl_mode = mode,
        stage,
        input_events,
        agent_events,
        commands_executed,
        "repl_status"
    );

    Ok(())
}

pub(crate) fn log_command_event(
    config: &ShellConfig,
    event_type: &str,
    command: &str,
    cmd_id: &str,
    extra: Option<serde_json::Value>,
) -> Result<()> {
    let stdin_is_tty = io::stdin().is_terminal();
    let stdout_is_tty = io::stdout().is_terminal();
    let stderr_is_tty = io::stderr().is_terminal();

    let mut log_entry = json!({
        log_schema::TIMESTAMP: Utc::now().to_rfc3339(),
        log_schema::EVENT_TYPE: event_type,
        log_schema::SESSION_ID: config.session_id,
        log_schema::COMMAND_ID: cmd_id,
        "command": command,
        log_schema::COMPONENT: "shell",
        "mode": match &config.mode {
            ShellMode::Interactive { .. } => "interactive",
            ShellMode::Wrap(_) => "wrap",
            ShellMode::Script(_) => "script",
            ShellMode::Pipe => "pipe",
        },
        "cwd": env::current_dir()?.display().to_string(),
        "host": gethostname::gethostname().to_string_lossy().to_string(),
        "shell": config.shell_path,
        "isatty_stdin": stdin_is_tty,
        "isatty_stdout": stdout_is_tty,
        "isatty_stderr": stderr_is_tty,
        "pty": matches!(&config.mode, ShellMode::Interactive { use_pty: true }),
    });

    if matches!(&config.mode, ShellMode::Interactive { .. }) {
        log_entry["repl_mode"] = json!(if config.async_repl { "async" } else { "sync" });
    }

    // Add build version if available
    if let Ok(build) = env::var("SHIM_BUILD") {
        log_entry["build"] = json!(build);
    }

    // Add ppid on Unix
    #[cfg(unix)]
    {
        log_entry["ppid"] = json!(nix::unistd::getppid().as_raw());
    }

    // Merge extra data
    if let Some(extra_data) = extra {
        if let Some(obj) = log_entry.as_object_mut() {
            if let Some(extra_obj) = extra_data.as_object() {
                for (k, v) in extra_obj {
                    obj.insert(k.clone(), v.clone());
                }
            }
        }
    }

    // Route all event writes through unified trace; writer handles rotation
    // Ensure trace is initialized first when world is enabled
    let _ = init_trace(None);
    append_to_trace(&log_entry)?;

    Ok(())
}
