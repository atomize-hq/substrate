//! Telemetry helpers for routing, including REPL metrics and command logging.

use super::super::{ShellConfig, ShellMode};
#[cfg(target_os = "linux")]
use crate::execution::socket_activation;
use anyhow::Result;
use chrono::Utc;
use serde_json::json;
use std::env;
use std::io::{self, IsTerminal};
use std::sync::Arc;
use substrate_common::{
    agent_events::{AgentEvent, AgentEventKind},
    log_schema, ProcessTelemetry,
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

    pub(crate) fn persist_agent_event(&self, event: &AgentEvent) {
        if let Err(err) = append_agent_event_to_trace(&self.config, event) {
            warn!(target = "substrate::shell", error = %err, "failed to append agent_event trace record");
        }
    }

    pub(crate) fn persist_warning_pty_structured_event_drops(
        &self,
        dropped_structured_event_lines: u64,
        max_pty_buffered_lines: usize,
        cmd_id: Option<&str>,
    ) {
        let dropped_i64 = i64::try_from(dropped_structured_event_lines).unwrap_or(i64::MAX);
        let max_i64 = i64::try_from(max_pty_buffered_lines).unwrap_or(i64::MAX);

        let mut entry = json!({
            log_schema::TIMESTAMP: Utc::now().to_rfc3339(),
            log_schema::EVENT_TYPE: "warning",
            log_schema::SESSION_ID: self.config.session_id,
            log_schema::COMPONENT: "shell",
            "code": "pty_structured_event_drops",
            "dropped_structured_event_lines": dropped_i64,
            "max_pty_buffered_lines": max_i64,
        });

        if let Some(cmd_id) = cmd_id {
            entry[log_schema::COMMAND_ID] = json!(cmd_id);
        }

        let _ = init_trace(None);
        if let Err(err) = append_to_trace(&entry) {
            warn!(target = "substrate::shell", error = %err, "failed to append pty_structured_event_drops warning record");
        }
    }

    pub(crate) fn persist_warning_config_value_clamped(
        &self,
        key: &str,
        provided: i64,
        effective: i64,
        min: i64,
        max: i64,
    ) {
        let entry = json!({
            log_schema::TIMESTAMP: Utc::now().to_rfc3339(),
            log_schema::EVENT_TYPE: "warning",
            log_schema::SESSION_ID: self.config.session_id,
            log_schema::COMPONENT: "shell",
            "code": "config_value_clamped",
            "key": key,
            "provided": provided,
            "effective": effective,
            "min": min,
            "max": max,
        });

        let _ = init_trace(None);
        if let Err(err) = append_to_trace(&entry) {
            warn!(target = "substrate::shell", error = %err, "failed to append config_value_clamped warning record");
        }
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

fn append_agent_event_to_trace(config: &ShellConfig, event: &AgentEvent) -> Result<()> {
    let mut sanitized = event.clone();
    let channel = sanitized.channel.take();
    sanitized.set_channel(channel);

    let mut entry = sanitized.to_trace_record()?;
    let obj = entry
        .as_object_mut()
        .ok_or_else(|| anyhow::anyhow!("agent event must serialize as a JSON object"))?;

    obj.insert(log_schema::EVENT_TYPE.to_string(), json!("agent_event"));
    obj.insert(log_schema::SESSION_ID.to_string(), json!(config.session_id));
    obj.insert(log_schema::COMPONENT.to_string(), json!("agent-hub"));

    let _ = init_trace(None);
    append_to_trace(&entry)?;
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

    if event_type == "builtin_command" {
        log_entry["command_omitted"] = json!(true);
    } else {
        log_entry["command"] = json!(command);
    }

    if matches!(&config.mode, ShellMode::Interactive { .. }) {
        log_entry["repl_mode"] = json!(if config.async_repl { "async" } else { "sync" });
    }

    #[cfg(target_os = "linux")]
    {
        log_entry["socket_activation"] =
            json!(socket_activation::socket_activation_report().is_socket_activated());
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

pub(crate) fn append_process_events_to_trace(process_telemetry: &ProcessTelemetry) -> Result<()> {
    if process_telemetry.process_events.is_empty() {
        return Ok(());
    }

    let _ = init_trace(None);
    for event in &process_telemetry.process_events {
        let mut entry = serde_json::to_value(event)?;
        if let Some(obj) = entry.as_object_mut() {
            obj.insert(log_schema::COMPONENT.to_string(), json!("world-service"));
        }
        append_to_trace(&entry)?;
    }

    Ok(())
}

pub(crate) fn add_process_telemetry_summary(
    log_entry: &mut serde_json::Value,
    process_telemetry: &ProcessTelemetry,
) {
    log_entry[log_schema::PROCESS_EVENTS_STATUS] =
        json!(process_telemetry.process_events_status.as_str());

    if process_telemetry.process_events_status != substrate_common::ProcessEventsStatus::Ok {
        if let Some(reason) = process_telemetry.process_events_reason.as_ref() {
            log_entry[log_schema::PROCESS_EVENTS_REASON] = json!(reason);
        }
    }

    if process_telemetry.process_events_status == substrate_common::ProcessEventsStatus::Truncated {
        if let Some(dropped) = process_telemetry.process_events_dropped {
            log_entry[log_schema::PROCESS_EVENTS_DROPPED] = json!(dropped);
        }
    }
}
