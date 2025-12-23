//! Execution and command launch helpers.

use super::registry::{is_force_pty_command, is_pty_disabled, needs_pty};
use super::shim_ops::wrap_with_anchor_guard;
#[cfg(target_os = "linux")]
use super::world_ops::execute_world_pty_over_ws;
#[cfg(target_os = "macos")]
use super::world_ops::execute_world_pty_over_ws_macos;
use super::world_ops::{
    collect_world_telemetry, emit_stream_chunk, stream_non_pty_via_agent, AgentStreamOutcome,
};
use crate::execution::pty;
#[cfg(any(target_os = "macos", target_os = "windows"))]
use crate::execution::pw;
use crate::execution::routing::builtin::handle_builtin;
use crate::execution::routing::telemetry::{log_command_event, SHELL_AGENT_ID};
#[cfg(any(target_os = "macos", target_os = "windows"))]
use crate::execution::routing::world_transport_to_meta;
#[cfg(target_os = "linux")]
use crate::execution::socket_activation;
use crate::execution::{configure_child_shell_env, needs_shell, ShellConfig, ShellMode};
use anyhow::{Context, Result};
use serde_json::json;
use std::env;
#[cfg(unix)]
use std::os::unix::process::ExitStatusExt;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitStatus, Stdio};
use std::sync::atomic::{AtomicI32, Ordering};
use std::sync::Arc;
use substrate_broker::{detect_profile, evaluate, world_fs_mode, Decision};
use substrate_common::{log_schema, redact_sensitive, WorldRootMode};
#[cfg(target_os = "linux")]
use substrate_trace::TransportMeta;
use substrate_trace::{create_span_builder, ExecutionOrigin, PolicyDecision};

pub(crate) fn execute_command(
    config: &ShellConfig,
    command: &str,
    cmd_id: &str,
    running_child_pid: Arc<AtomicI32>,
) -> Result<ExitStatus> {
    let trimmed = command.trim();

    // Always refresh policy/profile for this cwd before we read fs_mode.
    let cwd_for_profile = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    detect_profile(&cwd_for_profile).with_context(|| {
        format!(
            "failed to load Substrate profile for cwd {}",
            cwd_for_profile.display()
        )
    })?;

    let fs_mode = world_fs_mode();
    std::env::set_var("SUBSTRATE_WORLD_FS_MODE", fs_mode.as_str());

    // Prepare redacted command once (used for span + logging)
    let redacted_for_logging = if std::env::var("SHIM_LOG_OPTS").as_deref() == Ok("raw") {
        trimmed.to_string()
    } else {
        let toks = shell_words::split(trimmed)
            .unwrap_or_else(|_| trimmed.split_whitespace().map(|s| s.to_string()).collect());
        let mut out = Vec::new();
        let mut i = 0;

        while i < toks.len() {
            let t = &toks[i];
            let lt = t.to_lowercase();

            if lt == "-u" || lt == "--user" || lt == "--password" || lt == "--token" || lt == "-p" {
                out.push("***".into());
                if i + 1 < toks.len() {
                    out.push("***".into());
                    i += 2;
                } else {
                    i += 1;
                }
                continue;
            }

            if t == "-H" || lt == "--header" {
                out.push(t.clone());
                if i + 1 < toks.len() {
                    let hv = &toks[i + 1];
                    let lower = hv.to_lowercase();
                    let redacted = if lower.starts_with("authorization:")
                        || lower.starts_with("x-api-key:")
                        || lower.starts_with("x-auth-token:")
                        || lower.starts_with("cookie:")
                    {
                        format!(
                            "{}: ***",
                            hv.split(':').next().unwrap_or("").trim_end_matches(':')
                        )
                    } else {
                        hv.clone()
                    };
                    out.push(redacted);
                    i += 2;
                } else {
                    i += 1;
                }
                continue;
            }

            if t.contains('=') {
                let (k, _) = t.split_once('=').unwrap();
                let kl = k.to_lowercase();
                if kl.contains("token")
                    || kl.contains("password")
                    || kl.contains("secret")
                    || kl.contains("apikey")
                    || kl.contains("api_key")
                {
                    out.push(format!("{k}=***"));
                    i += 1;
                    continue;
                }
            }

            out.push(redact_sensitive(t));
            i += 1;
        }
        out.join(" ")
    };

    let world_env = std::env::var("SUBSTRATE_WORLD").unwrap_or_default();
    let world_enabled = world_env == "enabled";
    let world_disabled = world_env == "disabled" || config.no_world;
    let world_required = fs_mode != substrate_common::WorldFsMode::Writable && !world_disabled;
    if world_required && world_disabled {
        anyhow::bail!(
            "world execution required (fs_mode={}) but world is disabled (SUBSTRATE_WORLD=disabled or --no-world)",
            fs_mode.as_str()
        );
    }

    // Start span for command execution
    let policy_decision;
    let mut span = if world_enabled {
        // Policy evaluation (Phase 4)
        let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));

        // Detect and load .substrate-profile if present
        detect_profile(&cwd).with_context(|| {
            format!("failed to load Substrate profile for cwd {}", cwd.display())
        })?;

        let decision = evaluate(trimmed, cwd.to_str().unwrap_or("."), None)
            .with_context(|| format!("policy check failed for command: {trimmed}"))?;

        // Convert broker Decision to trace PolicyDecision
        policy_decision = match &decision {
            Decision::Allow => Some(PolicyDecision {
                action: "allow".to_string(),
                reason: None,
                restrictions: None,
            }),
            Decision::AllowWithRestrictions(restrictions) => {
                eprintln!(
                    "substrate: command requires restrictions: {:?}",
                    restrictions
                );
                // Convert Restriction objects to strings for trace
                let restriction_strings: Vec<String> = restrictions
                    .iter()
                    .map(|r| format!("{:?}:{}", r.type_, r.value))
                    .collect();
                Some(PolicyDecision {
                    action: "allow_with_restrictions".to_string(),
                    reason: None,
                    restrictions: Some(restriction_strings),
                })
            }
            Decision::Deny(reason) => {
                eprintln!("substrate: command denied by policy: {}", reason);
                Some(PolicyDecision {
                    action: "deny".to_string(),
                    reason: Some(reason.clone()),
                    restrictions: None,
                })
            }
        };

        // Handle denial
        if let Decision::Deny(_) = decision {
            // Return failure status
            #[cfg(unix)]
            {
                use std::os::unix::process::ExitStatusExt;
                return Ok(ExitStatus::from_raw(126 << 8)); // Cannot execute
            }
            #[cfg(windows)]
            {
                use std::os::windows::process::ExitStatusExt;
                return Ok(ExitStatus::from_raw(126));
            }
        }

        // Create span with policy decision
        if let Ok(mut builder) = create_span_builder() {
            builder = builder
                .with_command(&redacted_for_logging)
                .with_cwd(cwd.to_str().unwrap_or("."));

            if let Some(pd) = policy_decision.clone() {
                builder = builder.with_policy_decision(pd);
            }

            // Set parent span ID in environment for child processes
            match builder.start() {
                Ok(span) => {
                    std::env::set_var("SHIM_PARENT_SPAN", span.get_span_id());
                    Some(span)
                }
                Err(e) => {
                    eprintln!("substrate: failed to create span: {}", e);
                    None
                }
            }
        } else {
            eprintln!("substrate: failed to create span builder");
            None
        }
    } else if let Ok(mut builder) = create_span_builder() {
        let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        builder = builder
            .with_command(&redacted_for_logging)
            .with_cwd(cwd.to_str().unwrap_or("."));

        match builder.start() {
            Ok(span) => {
                std::env::set_var("SHIM_PARENT_SPAN", span.get_span_id());
                Some(span)
            }
            Err(e) => {
                eprintln!("substrate: failed to create span: {}", e);
                None
            }
        }
    } else {
        eprintln!("substrate: failed to create span builder");
        None
    };

    // Check if PTY should be used (force overrides disable)
    let disabled = is_pty_disabled();
    let forced = is_force_pty_command(trimmed);
    let should_use_pty = forced || (!disabled && needs_pty(trimmed));

    if should_use_pty {
        // Attempt world-agent PTY WS route on Linux when world is enabled or agent socket exists
        #[cfg(target_os = "linux")]
        {
            let uds_exists = std::path::Path::new("/run/substrate.sock").exists();
            let world_available = !world_disabled && (world_enabled || uds_exists);
            if world_required && !world_available {
                anyhow::bail!(
                    "world execution required (fs_mode={}) but world-agent is unavailable (/run/substrate.sock missing)",
                    fs_mode.as_str()
                );
            }
            if world_available {
                let transport_meta = TransportMeta {
                    mode: "unix".to_string(),
                    endpoint: Some("/run/substrate.sock".to_string()),
                    socket_activation: Some(
                        socket_activation::socket_activation_report().is_socket_activated(),
                    ),
                };
                // Use span id if we have a span, otherwise fall back to cmd_id as a correlation hint
                if let Some(active_span) = span.as_mut() {
                    active_span.set_transport(transport_meta.clone());
                }
                let span_id_for_ws = span
                    .as_ref()
                    .map(|s| s.get_span_id().to_string())
                    .unwrap_or_else(|| cmd_id.to_string());
                match execute_world_pty_over_ws(trimmed, &span_id_for_ws) {
                    Ok(code) => {
                        if let Some(active_span) = span.take() {
                            let mut active_span = active_span;
                            active_span.set_execution_origin(ExecutionOrigin::World);
                            active_span.set_transport(transport_meta);
                            let (scopes_used, fs_diff) =
                                collect_world_telemetry(active_span.get_span_id());
                            let _ = active_span.finish(code, scopes_used, fs_diff);
                        }
                        #[cfg(unix)]
                        {
                            use std::os::unix::process::ExitStatusExt;
                            return Ok(ExitStatus::from_raw((code & 0xff) << 8));
                        }
                        #[cfg(windows)]
                        {
                            use std::os::windows::process::ExitStatusExt;
                            return Ok(ExitStatus::from_raw(code as u32));
                        }
                    }
                    Err(e) => {
                        if world_required {
                            anyhow::bail!(
                                "world execution required (fs_mode={}); PTY world path failed: {}",
                                fs_mode.as_str(),
                                e
                            );
                        }
                        eprintln!(
                            "substrate: warn: world PTY over WS failed, falling back to host PTY: {}",
                            e
                        );
                        // fall through to host PTY
                    }
                }
            }
        }

        // Attempt world-agent PTY WS route on mac when world is enabled
        #[cfg(target_os = "macos")]
        {
            let context = pw::get_context();
            let uds_exists = context
                .as_ref()
                .map(|c| match &c.transport {
                    pw::WorldTransport::Unix(p) => p.exists(),
                    _ => false,
                })
                .unwrap_or(false);
            let world_available = !world_disabled && (world_enabled || uds_exists);
            if world_required && !world_available {
                anyhow::bail!(
                    "world execution required (fs_mode={}) but world-agent is unavailable",
                    fs_mode.as_str()
                );
            }
            if world_available {
                let transport_meta = context
                    .as_ref()
                    .map(|ctx| world_transport_to_meta(&ctx.transport));
                let span_id_for_ws = span
                    .as_ref()
                    .map(|s| s.get_span_id().to_string())
                    .unwrap_or_else(|| cmd_id.to_string());
                match execute_world_pty_over_ws_macos(trimmed, &span_id_for_ws) {
                    Ok(code) => {
                        if let Some(active_span) = span.take() {
                            let mut active_span = active_span;
                            if let Some(meta) = transport_meta {
                                active_span.set_transport(meta);
                            }
                            active_span.set_execution_origin(ExecutionOrigin::World);
                            let (scopes_used, fs_diff) =
                                collect_world_telemetry(active_span.get_span_id());
                            let _ = active_span.finish(code, scopes_used, fs_diff);
                        }
                        #[cfg(unix)]
                        {
                            use std::os::unix::process::ExitStatusExt;
                            return Ok(ExitStatus::from_raw((code & 0xff) << 8));
                        }
                        #[cfg(windows)]
                        {
                            use std::os::windows::process::ExitStatusExt;
                            return Ok(ExitStatus::from_raw(code as u32));
                        }
                    }
                    Err(e) => {
                        if world_required {
                            anyhow::bail!(
                                "world execution required (fs_mode={}); PTY world path failed: {}",
                                fs_mode.as_str(),
                                e
                            );
                        }
                        static WARNED: std::sync::Once = std::sync::Once::new();
                        WARNED.call_once(|| {
                            eprintln!("substrate: warn: world PTY over WS failed on mac, falling back to host PTY: {}", e);
                        });
                        // fall through to host PTY
                    }
                }
            }
        }

        // Use host PTY execution path as fallback
        let pty_status = pty::execute_with_pty(config, trimmed, cmd_id, running_child_pid)?;

        // Finish span if we created one (PTY path)
        if let Some(active_span) = span {
            let exit_code = pty_status
                .code()
                .or_else(|| pty_status.success().then_some(0))
                .unwrap_or(-1);
            // Collect scopes and fs_diff from world backend if enabled
            let origin_is_world = active_span.execution_origin() == ExecutionOrigin::World;
            let (scopes_used, fs_diff) = if origin_is_world {
                collect_world_telemetry(active_span.get_span_id())
            } else {
                (vec![], None)
            };
            let _ = active_span.finish(exit_code, scopes_used, fs_diff);
        }

        // Convert PtyExitStatus to std::process::ExitStatus for compatibility
        // NOTE: This is a documented compatibility shim using from_raw
        // The actual exit information is preserved in logs
        #[cfg(unix)]
        {
            use std::os::unix::process::ExitStatusExt;
            if let Some(signal) = pty_status.signal() {
                // Terminated by signal: set low 7 bits to the signal number
                // This makes status.signal() work correctly
                return Ok(ExitStatus::from_raw(signal & 0x7f));
            } else if let Some(code) = pty_status.code() {
                // Normal exit: code in bits 8-15
                return Ok(ExitStatus::from_raw((code & 0xff) << 8));
            } else {
                return Ok(ExitStatus::from_raw(0));
            }
        }

        #[cfg(windows)]
        {
            // Ensure Windows builds exercise signal() accessor even though it always returns None.
            let _ = pty_status.signal();
            // 🔥 EXPERT FIX: Don't shift bits on Windows - use raw code directly
            use std::os::windows::process::ExitStatusExt;
            let code = pty_status.code().unwrap_or(0) as u32;
            return Ok(ExitStatus::from_raw(code));
        }
    }

    // Continue with existing implementation for non-PTY commands...
    // Compute resolved path from raw command before redaction
    let resolved = first_command_path(trimmed);

    // Redact sensitive information before logging (token-aware)
    let redacted_command = redacted_for_logging.clone();

    // Log command start with redacted command and resolved path
    let start_extra = resolved.map(|p| json!({ "resolved_path": p }));
    log_command_event(
        config,
        "command_start",
        &redacted_command,
        cmd_id,
        start_extra,
    )?;
    let start_time = std::time::Instant::now();

    // Attempt to route non-PTY commands through the world agent when available
    let mut agent_result: Option<AgentStreamOutcome> = None;

    #[cfg(target_os = "macos")]
    {
        let context = pw::get_context();
        let uds_exists = context
            .as_ref()
            .map(|c| matches!(&c.transport, pw::WorldTransport::Unix(path) if path.exists()))
            .unwrap_or(false);
        let world_available = !world_disabled && (world_enabled || uds_exists);
        if world_required && !world_available {
            anyhow::bail!(
                "world execution required (fs_mode={}) but world-agent is unavailable",
                fs_mode.as_str()
            );
        }
        if world_available {
            let transport_meta = context
                .as_ref()
                .map(|ctx| world_transport_to_meta(&ctx.transport));
            let mut agent_command = trimmed.to_string();
            if config.ci_mode && !config.no_exit_on_error {
                let shell_name = Path::new(&config.shell_path)
                    .file_name()
                    .and_then(|s| s.to_str())
                    .unwrap_or("")
                    .to_ascii_lowercase();
                if shell_name == "bash" || shell_name == "bash.exe" {
                    agent_command = format!("set -euo pipefail; {trimmed}");
                }
            }
            match stream_non_pty_via_agent(&agent_command) {
                Ok(outcome) => {
                    if let Some(active_span) = span.as_mut() {
                        active_span.set_execution_origin(ExecutionOrigin::World);
                        if let Some(meta) = transport_meta.clone() {
                            active_span.set_transport(meta);
                        }
                    }
                    agent_result = Some(outcome);
                }
                Err(e) => {
                    if world_required {
                        anyhow::bail!(
                            "world execution required (fs_mode={}); world-agent exec failed: {}",
                            fs_mode.as_str(),
                            e
                        );
                    }
                    static WARN_ONCE: std::sync::Once = std::sync::Once::new();
                    let path_hint = context
                        .as_ref()
                        .map(|ctx| ctx.transport.to_string())
                        .unwrap_or_else(|| "mac context unavailable".to_string());
                    let err_msg = e.to_string();
                    WARN_ONCE.call_once(move || {
                        eprintln!(
                            "substrate: warn: shell world-agent path ({}) exec failed, running direct: {}",
                            path_hint,
                            err_msg
                        );
                    });
                }
            }
        }
    }

    #[cfg(target_os = "windows")]
    {
        let context = pw::get_context();
        let world_available = !world_disabled && (world_enabled || context.is_some());
        if world_required && !world_available {
            anyhow::bail!(
                "world execution required (fs_mode={}) but world-agent is unavailable",
                fs_mode.as_str()
            );
        }
        if world_available {
            let transport_meta = context
                .as_ref()
                .map(|ctx| world_transport_to_meta(&ctx.transport));
            let mut agent_command = trimmed.to_string();
            if config.ci_mode && !config.no_exit_on_error {
                let shell_name = Path::new(&config.shell_path)
                    .file_name()
                    .and_then(|s| s.to_str())
                    .unwrap_or("")
                    .to_ascii_lowercase();
                if shell_name == "bash" || shell_name == "bash.exe" {
                    agent_command = format!("set -euo pipefail; {trimmed}");
                }
            }
            match stream_non_pty_via_agent(&agent_command) {
                Ok(outcome) => {
                    if let Some(active_span) = span.as_mut() {
                        active_span.set_execution_origin(ExecutionOrigin::World);
                        if let Some(meta) = transport_meta.clone() {
                            active_span.set_transport(meta);
                        }
                    }
                    agent_result = Some(outcome);
                }
                Err(e) => {
                    if world_required {
                        anyhow::bail!(
                            "world execution required (fs_mode={}); world-agent exec failed: {}",
                            fs_mode.as_str(),
                            e
                        );
                    }
                    static WARN_ONCE: std::sync::Once = std::sync::Once::new();
                    let path_hint = context
                        .as_ref()
                        .map(|ctx| ctx.transport.to_string())
                        .unwrap_or_else(|| "windows context unavailable".to_string());
                    let err_msg = e.to_string();
                    WARN_ONCE.call_once(move || {
                        eprintln!(
                            "substrate: warn: shell world-agent path ({}) exec failed, running direct: {}",
                            path_hint,
                            err_msg
                        );
                    });
                }
            }
        }
    }

    // Handle lightweight builtins first (cd/pwd/export/unset) so stateful changes
    // like cwd take effect before we hand off to the agent path.
    if !needs_shell(trimmed) {
        if let Some(status) = handle_builtin(config, trimmed, cmd_id)? {
            if let Some(active_span) = span {
                let _ = active_span.finish(status.code().unwrap_or(-1), vec![], None);
            }
            let completion_extra = json!({
                log_schema::EXIT_CODE: status.code().unwrap_or(-1),
                log_schema::DURATION_MS: start_time.elapsed().as_millis()
            });
            log_command_event(
                config,
                "command_complete",
                &redacted_command,
                cmd_id,
                Some(completion_extra),
            )?;
            return Ok(status);
        }
    }

    #[cfg(target_os = "linux")]
    {
        let uds_exists = std::path::Path::new("/run/substrate.sock").exists();
        let world_available = !world_disabled && (world_enabled || uds_exists);
        if world_required && !world_available {
            anyhow::bail!(
                "world execution required (fs_mode={}) but world-agent is unavailable (/run/substrate.sock missing)",
                fs_mode.as_str()
            );
        }
        if world_available {
            let transport_meta = TransportMeta {
                mode: "unix".to_string(),
                endpoint: Some("/run/substrate.sock".to_string()),
                socket_activation: Some(
                    socket_activation::socket_activation_report().is_socket_activated(),
                ),
            };
            match stream_non_pty_via_agent(trimmed) {
                Ok(outcome) => {
                    if let Some(active_span) = span.as_mut() {
                        active_span.set_execution_origin(ExecutionOrigin::World);
                        active_span.set_transport(transport_meta);
                    }
                    agent_result = Some(outcome);
                }
                Err(e) => {
                    if world_required {
                        anyhow::bail!(
                            "world execution required (fs_mode={}); world-agent exec failed: {}",
                            fs_mode.as_str(),
                            e
                        );
                    }
                    eprintln!(
                        "substrate: warn: shell world-agent path (/run/substrate.sock) exec failed, running direct: {}",
                        e
                    );
                }
            }
        }
    }

    if let Some(outcome) = agent_result {
        if let Some(active_span) = span {
            let _ = active_span.finish(
                outcome.exit_code,
                outcome.scopes_used.clone(),
                outcome.fs_diff.clone(),
            );
        }
        let completion_extra = json!({
            log_schema::EXIT_CODE: outcome.exit_code,
            log_schema::DURATION_MS: start_time.elapsed().as_millis()
        });
        log_command_event(
            config,
            "command_complete",
            &redacted_command,
            cmd_id,
            Some(completion_extra),
        )?;
        #[cfg(unix)]
        {
            use std::os::unix::process::ExitStatusExt;
            return Ok(ExitStatus::from_raw((outcome.exit_code & 0xff) << 8));
        }
        #[cfg(windows)]
        {
            use std::os::windows::process::ExitStatusExt;
            return Ok(ExitStatus::from_raw(outcome.exit_code as u32));
        }
    }

    // Execute external command through shell for complex commands or when no builtin matched.
    let status = execute_external(config, trimmed, running_child_pid, cmd_id)?;

    // Log command completion with redacted command
    let duration = start_time.elapsed();
    #[cfg(unix)]
    let mut extra = json!({
        log_schema::EXIT_CODE: status.code().unwrap_or(-1),
        log_schema::DURATION_MS: duration.as_millis()
    });

    #[cfg(not(unix))]
    let extra = json!({
        log_schema::EXIT_CODE: status.code().unwrap_or(-1),
        log_schema::DURATION_MS: duration.as_millis()
    });

    #[cfg(unix)]
    {
        if let Some(sig) = status.signal() {
            extra["term_signal"] = json!(sig);
        }
    }

    log_command_event(
        config,
        "command_complete",
        &redacted_command,
        cmd_id,
        Some(extra),
    )?;

    // Finish span if we created one
    if let Some(active_span) = span {
        let exit_code = status.code().unwrap_or(-1);
        let origin_is_world = active_span.execution_origin() == ExecutionOrigin::World;
        let (scopes_used, fs_diff) = if origin_is_world {
            collect_world_telemetry(active_span.get_span_id())
        } else {
            (vec![], None)
        };
        let _ = active_span.finish(exit_code, scopes_used, fs_diff);
    }

    Ok(status)
}

fn execute_external(
    config: &ShellConfig,
    command: &str,
    running_child_pid: Arc<AtomicI32>,
    cmd_id: &str,
) -> Result<ExitStatus> {
    let shell = &config.shell_path;
    let mut command = command.to_string();

    // Verify shell exists
    if which::which(shell).is_err() && !Path::new(shell).exists() {
        return Err(anyhow::anyhow!("Shell not found: {}", shell));
    }

    let mut cmd = Command::new(shell);

    // Shell-specific command execution
    let shell_name = Path::new(shell)
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();

    let is_pwsh = shell_name == "pwsh.exe" || shell_name == "pwsh";
    let is_powershell = shell_name == "powershell.exe" || shell_name == "powershell";
    let is_cmd = shell_name == "cmd.exe" || shell_name == "cmd";
    let is_bash = shell_name == "bash" || shell_name == "bash.exe";
    let should_guard_anchor = config.world_root.caged
        && config.world_root.mode != WorldRootMode::FollowCwd
        && !cfg!(windows)
        && needs_shell(&command);

    if should_guard_anchor {
        command = wrap_with_anchor_guard(&command, config);
    }
    let mut command_for_shell = command.clone();

    if is_pwsh || is_powershell {
        // PowerShell
        if config.ci_mode && !config.no_exit_on_error {
            command_for_shell = format!("$ErrorActionPreference='Stop'; {command_for_shell}");
            cmd.arg("-NoProfile")
                .arg("-NonInteractive")
                .arg("-Command")
                .arg(command_for_shell);
        } else {
            cmd.arg("-NoProfile").arg("-Command").arg(command_for_shell);
        }
    } else if is_cmd {
        // Windows CMD
        cmd.arg("/C").arg(command_for_shell);
    } else {
        // Unix shells (bash, sh, zsh, etc.)
        if config.ci_mode && !config.no_exit_on_error && is_bash {
            command_for_shell = format!("set -euo pipefail; {command_for_shell}");
        }
        cmd.arg("-c").arg(command_for_shell);
    }

    // Propagate environment
    cmd.env("SHIM_SESSION_ID", &config.session_id);
    cmd.env("SHIM_TRACE_LOG", &config.trace_log_file);
    cmd.env("SHIM_PARENT_CMD_ID", cmd_id); // Pass cmd_id for shim correlation
    cmd.env_remove("SHIM_ACTIVE"); // Clear to allow shims to work
    cmd.env_remove("SHIM_CALLER"); // Clear caller chain for fresh command
    cmd.env_remove("SHIM_CALL_STACK"); // Clear call stack for fresh command
                                       // Keep PATH as-is with shims - the env_remove("SHIM_ACTIVE") should be sufficient

    configure_child_shell_env(
        &mut cmd,
        config,
        is_bash,
        matches!(config.mode, ShellMode::Script(_)),
    );

    // Handle I/O based on mode - always inherit stdin for better compatibility and stream output ourselves
    cmd.stdin(Stdio::inherit());
    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::piped());

    // Make child process a group leader on Unix
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        unsafe {
            cmd.pre_exec(|| {
                // Safety: setpgid is safe when called before exec
                libc::setpgid(0, 0);
                Ok(())
            });
        }
    }

    // Spawn and track child PID for signal handling
    let mut child = cmd
        .spawn()
        .with_context(|| format!("Failed to execute: {command}"))?;

    let stdout_thread = child
        .stdout
        .take()
        .map(|pipe| spawn_host_stream_thread(pipe, false, SHELL_AGENT_ID.to_string()));
    let stderr_thread = child
        .stderr
        .take()
        .map(|pipe| spawn_host_stream_thread(pipe, true, SHELL_AGENT_ID.to_string()));

    let child_pid = child.id() as i32;
    running_child_pid.store(child_pid, Ordering::SeqCst);

    let status = child
        .wait()
        .with_context(|| format!("Failed to wait for command: {command}"))?;

    running_child_pid.store(0, Ordering::SeqCst);

    if let Some(handle) = stdout_thread {
        match handle.join() {
            Ok(Ok(())) => {}
            Ok(Err(e)) => eprintln!("substrate: warn: stdout stream error: {}", e),
            Err(e) => eprintln!("substrate: warn: stdout stream thread panicked: {:?}", e),
        }
    }
    if let Some(handle) = stderr_thread {
        match handle.join() {
            Ok(Ok(())) => {}
            Ok(Err(e)) => eprintln!("substrate: warn: stderr stream error: {}", e),
            Err(e) => eprintln!("substrate: warn: stderr stream thread panicked: {:?}", e),
        }
    }

    Ok(status)
}

fn spawn_host_stream_thread<R>(
    mut reader: R,
    is_stderr: bool,
    agent_label: String,
) -> std::thread::JoinHandle<anyhow::Result<()>>
where
    R: std::io::Read + Send + 'static,
{
    std::thread::spawn(move || {
        let mut buf = [0u8; 8192];
        loop {
            match std::io::Read::read(&mut reader, &mut buf) {
                Ok(0) => break,
                Ok(n) => {
                    emit_stream_chunk(&agent_label, &buf[..n], is_stderr);
                }
                Err(e) if e.kind() == std::io::ErrorKind::Interrupted => continue,
                Err(e) => return Err(anyhow::anyhow!("pipe read failed: {}", e)),
            }
        }
        Ok(())
    })
}

fn first_command_path(cmd: &str) -> Option<String> {
    // Skip resolution unless SHIM_LOG_OPTS=resolve is set (performance optimization)
    if env::var("SHIM_LOG_OPTS").as_deref() != Ok("resolve") {
        return None;
    }

    // Use shell_words for proper tokenization, fall back to whitespace split
    let tokens = shell_words::split(cmd)
        .unwrap_or_else(|_| cmd.split_whitespace().map(|s| s.to_string()).collect());

    let first = tokens.first()?;
    let p = std::path::Path::new(first);
    if p.is_absolute() {
        return Some(first.to_string());
    }
    // Best effort PATH lookup
    which::which(first).ok().map(|pb| pb.display().to_string())
}
