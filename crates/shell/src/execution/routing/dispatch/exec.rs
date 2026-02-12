//! Execution and command launch helpers.

use super::registry::{is_force_pty_command, is_pty_disabled, needs_pty};
use super::shim_ops::wrap_with_anchor_guard;
#[cfg(target_os = "linux")]
use super::world_ops::execute_world_pty_over_ws;
#[cfg(target_os = "macos")]
use super::world_ops::execute_world_pty_over_ws_macos;
#[cfg(target_os = "linux")]
use super::world_ops::WorldFsStrategyUnavailableError;
use super::world_ops::{
    collect_world_telemetry, emit_stream_chunk, stream_non_pty_via_agent, AgentStreamOutcome,
};
use crate::execution::config_model;
use crate::execution::config_model::CliConfigOverrides;
use crate::execution::config_model::PolicyMode;
use crate::execution::pty;
#[cfg(any(target_os = "macos", target_os = "windows"))]
use crate::execution::pw;
use crate::execution::routing::builtin::handle_builtin;
use crate::execution::routing::telemetry::{log_command_event, SHELL_AGENT_ID};
#[cfg(any(target_os = "macos", target_os = "windows"))]
use crate::execution::routing::world_transport_to_meta;
#[cfg(target_os = "linux")]
use crate::execution::socket_activation;
use crate::execution::{
    configure_child_shell_env, needs_shell, update_world_env, ShellConfig, ShellMode,
};
use anyhow::{Context, Result};
use serde_json::json;
use std::env;
use std::ffi::OsString;
#[cfg(unix)]
use std::os::unix::process::ExitStatusExt;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitStatus, Stdio};
use std::sync::atomic::{AtomicI32, Ordering};
use std::sync::{Arc, Once};
use substrate_broker::{detect_profile, evaluate, world_fs_policy, Decision};
use substrate_common::{
    log_schema, redact_sensitive, WorldFsStrategy, WorldFsStrategyFallbackReason, WorldRootMode,
};
#[cfg(target_os = "linux")]
use substrate_trace::TransportMeta;
use substrate_trace::{create_span_builder, ExecutionOrigin, PolicyDecision};

#[cfg(target_os = "linux")]
const WORLD_BACKEND_UNAVAILABLE_HINT: &str =
    "hint: run 'substrate world doctor --json' and check 'systemctl status substrate-world-agent.socket'";
#[cfg(not(target_os = "linux"))]
const WORLD_BACKEND_UNAVAILABLE_HINT: &str = "hint: run 'substrate world doctor --json'";

static WORLD_BACKEND_UNAVAILABLE_WARN_ONCE: Once = Once::new();
static WORLD_ROUTING_FALLBACK_WARN_ONCE: Once = Once::new();

struct ParentSpanGuard {
    previous: Option<OsString>,
}

impl ParentSpanGuard {
    fn set_current(span_id: &str) -> Self {
        let previous = env::var_os("SHIM_PARENT_SPAN");
        env::set_var("SHIM_PARENT_SPAN", span_id);
        Self { previous }
    }
}

impl Drop for ParentSpanGuard {
    fn drop(&mut self) {
        match self.previous.take() {
            Some(value) => env::set_var("SHIM_PARENT_SPAN", value),
            None => env::remove_var("SHIM_PARENT_SPAN"),
        }
    }
}

#[cfg(target_os = "linux")]
const DEFAULT_WORLD_SOCKET_PATH: &str = "/run/substrate.sock";

#[cfg(target_os = "linux")]
fn world_socket_path() -> PathBuf {
    env::var_os("SUBSTRATE_WORLD_SOCKET")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(DEFAULT_WORLD_SOCKET_PATH))
}

#[cfg(target_os = "linux")]
fn world_socket_note() -> Option<String> {
    let socket_path = world_socket_path();

    let socket_override_active = env::var_os("SUBSTRATE_WORLD_SOCKET")
        .map(|p| p != std::ffi::OsStr::new(DEFAULT_WORLD_SOCKET_PATH))
        .unwrap_or(false);

    if socket_override_active {
        Some(format!(
            "SUBSTRATE_WORLD_SOCKET override: {}",
            socket_path.display()
        ))
    } else {
        Some(format!("socket: {}", socket_path.display()))
    }
}

#[cfg(not(target_os = "linux"))]
fn world_socket_note() -> Option<String> {
    None
}

fn warn_world_backend_unavailable_once() {
    WORLD_BACKEND_UNAVAILABLE_WARN_ONCE.call_once(|| {
        let socket_note = world_socket_note();
        let socket_note = socket_note
            .as_deref()
            .map(|note| format!(" ({note})"))
            .unwrap_or_default();
        eprintln!(
            "substrate: warn: world backend unavailable{}; running on host ({})",
            socket_note, WORLD_BACKEND_UNAVAILABLE_HINT
        );
    });
}

fn warn_world_routing_failed_falling_back_to_host_once() {
    WORLD_ROUTING_FALLBACK_WARN_ONCE.call_once(|| {
        let socket_note = world_socket_note();
        let socket_note = socket_note
            .as_deref()
            .map(|note| format!(" ({note})"))
            .unwrap_or_default();
        eprintln!(
            "substrate: warn: world routing failed; falling back to host{}; world_fs.host_visible=false was requested; world_fs.fail_closed.routing=false allows fallback ({})",
            socket_note, WORLD_BACKEND_UNAVAILABLE_HINT
        );
    });
}

fn required_world_backend_unavailable_error(reason: &str) -> anyhow::Error {
    let socket_note = world_socket_note();
    let socket_note = socket_note
        .as_deref()
        .map(|note| format!(" ({note})"))
        .unwrap_or_default();
    anyhow::anyhow!(
        "world execution required ({}) but world backend is unavailable{} ({})",
        reason,
        socket_note,
        WORLD_BACKEND_UNAVAILABLE_HINT
    )
}

fn exit_status_from_code(code: i32) -> ExitStatus {
    #[cfg(unix)]
    {
        ExitStatus::from_raw((code & 0xff) << 8)
    }

    #[cfg(windows)]
    {
        use std::os::windows::process::ExitStatusExt;
        ExitStatus::from_raw(code as u32)
    }
}

pub(crate) fn execute_command(
    config: &ShellConfig,
    command: &str,
    cmd_id: &str,
    running_child_pid: Arc<AtomicI32>,
) -> Result<ExitStatus> {
    let trimmed = command.trim();

    let cwd_for_profile = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));

    // Always refresh config for this cwd so config patch edits are visible on the next command.
    let cli_world_enabled = if config.cli_world {
        Some(true)
    } else if config.cli_no_world {
        Some(false)
    } else {
        None
    };
    let effective_config = config_model::resolve_effective_config(
        &cwd_for_profile,
        &CliConfigOverrides {
            world_enabled: cli_world_enabled,
            anchor_mode: config.cli_anchor_mode,
            anchor_path: config
                .cli_anchor_path
                .as_ref()
                .map(|p| p.to_string_lossy().to_string()),
            caged: config.cli_caged,
        },
    )?;
    let policy_mode = effective_config.policy.mode;
    std::env::set_var("SUBSTRATE_POLICY_MODE", policy_mode.as_str());
    substrate_broker::set_policy_mode(match policy_mode {
        PolicyMode::Disabled => substrate_broker::PolicyMode::Disabled,
        PolicyMode::Observe => substrate_broker::PolicyMode::Observe,
        PolicyMode::Enforce => substrate_broker::PolicyMode::Enforce,
    });

    let world_disabled = !effective_config.world.enabled;
    let world_enabled = !world_disabled;

    // Always refresh policy/profile for this cwd before we read world_fs.
    let profile_result = detect_profile(&cwd_for_profile).with_context(|| {
        format!(
            "failed to load Substrate profile for cwd {}",
            cwd_for_profile.display()
        )
    });
    if let Err(err) = profile_result {
        return Err(config_model::user_error(format!("{:#}", err)));
    }

    let world_fs = world_fs_policy();
    let fs_mode = world_fs.mode;
    let fail_closed_routing = world_fs.fail_closed_routing;
    let host_visible = world_fs.host_visible;
    let caged_required = world_fs.caged_required;
    update_world_env(world_disabled);

    if fail_closed_routing && world_disabled {
        return Err(config_model::user_error(
            "world_fs.fail_closed.routing=true requires world.enabled=true (effective world disable is a hard error)",
        ));
    }

    if caged_required {
        if !effective_config.world.caged {
            return Err(config_model::user_error(
                "world_fs.caged_required=true requires world.caged=true (uncaged mode is a hard error)",
            ));
        }
        if effective_config.world.anchor_mode == WorldRootMode::FollowCwd {
            return Err(config_model::user_error(
                "world_fs.caged_required=true is incompatible with world.anchor_mode=follow-cwd (hard error)",
            ));
        }
    }

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

    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let decision = match policy_mode {
        PolicyMode::Disabled => None,
        PolicyMode::Observe | PolicyMode::Enforce => Some(
            evaluate(trimmed, cwd.to_str().unwrap_or("."), None)
                .with_context(|| format!("policy check failed for command: {trimmed}"))?,
        ),
    };

    let cmd_isolated_match = matches!(decision, Some(Decision::AllowWithRestrictions(_)));

    let requires_world_constraint =
        matches!(policy_mode, PolicyMode::Observe | PolicyMode::Enforce)
            && (world_fs.require_world
                || world_fs.mode == substrate_common::WorldFsMode::ReadOnly
                || world_fs.isolation == substrate_broker::WorldFsIsolation::Full
                || cmd_isolated_match);

    let world_required_by_cli = config.cli_world;
    let world_required_by_policy = policy_mode == PolicyMode::Enforce && requires_world_constraint;
    let world_required = world_required_by_cli || world_required_by_policy;

    let world_required_reason = {
        let mut reasons = Vec::new();
        if world_required_by_cli {
            reasons.push("--world".to_string());
        }
        if world_required_by_policy {
            let mut details = Vec::new();
            if world_fs.require_world {
                details.push("world_fs.require_world=true");
            }
            if world_fs.mode == substrate_common::WorldFsMode::ReadOnly {
                details.push("world_fs.mode=read_only");
            }
            if world_fs.isolation == substrate_broker::WorldFsIsolation::Full {
                details.push("world_fs.isolation=full");
            }
            if cmd_isolated_match {
                details.push("cmd_isolated match");
            }
            let detail = if details.is_empty() {
                "".to_string()
            } else {
                format!(" ({})", details.join(", "))
            };
            reasons.push(format!("policy requires world{detail}"));
        }
        if reasons.is_empty() {
            "unknown".to_string()
        } else {
            reasons.join(" + ")
        }
    };

    if world_required && world_disabled {
        return Err(config_model::user_error(format!(
            "world execution required ({}) but world is disabled (SUBSTRATE_OVERRIDE_WORLD=disabled, --no-world, or world.enabled=false)",
            world_required_reason
        )));
    }

    let mut policy_decision = None;
    if let Some(ref decision) = decision {
        let mut restriction_strings: Vec<String> = match decision {
            Decision::AllowWithRestrictions(restrictions) => restrictions
                .iter()
                .map(|r| format!("{:?}:{}", r.type_, r.value))
                .collect(),
            _ => Vec::new(),
        };

        if requires_world_constraint {
            restriction_strings.push("requires_world".to_string());
        }

        let (action, reason) = match decision {
            Decision::Allow => {
                if restriction_strings.is_empty() {
                    ("allow".to_string(), None)
                } else {
                    ("allow_with_restrictions".to_string(), None)
                }
            }
            Decision::AllowWithRestrictions(_) => ("allow_with_restrictions".to_string(), None),
            Decision::Deny(reason) => {
                let effective_reason = if policy_mode == PolicyMode::Observe {
                    format!("would deny (policy.mode=observe): {reason}")
                } else {
                    reason.clone()
                };
                ("deny".to_string(), Some(effective_reason))
            }
        };

        policy_decision = Some(PolicyDecision {
            action,
            reason,
            restrictions: (!restriction_strings.is_empty()).then_some(restriction_strings),
        });
    }

    let mut _parent_span_guard: Option<ParentSpanGuard> = None;
    let mut span = if let Ok(mut builder) = create_span_builder() {
        builder = builder
            .with_command(&redacted_for_logging)
            .with_cwd(cwd.to_str().unwrap_or("."));
        if let Some(pd) = policy_decision.clone() {
            builder = builder.with_policy_decision(pd);
        }
        match builder.start() {
            Ok(span) => {
                _parent_span_guard = Some(ParentSpanGuard::set_current(span.get_span_id()));
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

    if policy_mode == PolicyMode::Enforce {
        if let Some(Decision::Deny(reason)) = decision {
            eprintln!("substrate: command denied by policy: {}", reason);
            if let Some(mut active_span) = span.take() {
                active_span.set_outcome("denied");
                let _ = active_span.finish(126, vec![], None);
            }
            #[cfg(unix)]
            {
                use std::os::unix::process::ExitStatusExt;
                return Ok(ExitStatus::from_raw(126 << 8));
            }
            #[cfg(windows)]
            {
                use std::os::windows::process::ExitStatusExt;
                return Ok(ExitStatus::from_raw(126));
            }
        }
    }

    // Check if PTY should be used (force overrides disable)
    let disabled = is_pty_disabled();
    let forced = is_force_pty_command(trimmed);
    let should_use_pty = forced || (!disabled && needs_pty(trimmed));

    // WO0/ADR-0004: command_complete events written via `log_command_event` must always include
    // the world fs strategy contract fields. Host-only execution paths use a conservative default,
    // with specific fallback cases overriding this.
    #[cfg(target_os = "linux")]
    let mut world_fs_strategy_log_override: Option<(
        WorldFsStrategy,
        WorldFsStrategy,
        WorldFsStrategyFallbackReason,
    )> = None;

    #[cfg(not(target_os = "linux"))]
    let world_fs_strategy_log_override: Option<(
        WorldFsStrategy,
        WorldFsStrategy,
        WorldFsStrategyFallbackReason,
    )> = None;

    if should_use_pty {
        // Attempt world-agent PTY WS route on Linux when world is enabled or agent socket exists
        #[cfg(target_os = "linux")]
        {
            let socket_path = world_socket_path();
            let uds_exists = socket_path.exists();
            if world_required && world_enabled && !uds_exists {
                return Err(required_world_backend_unavailable_error(
                    &world_required_reason,
                ));
            }
            if world_enabled && !world_required && !uds_exists {
                if fail_closed_routing {
                    eprintln!(
                        "substrate: error: {}",
                        required_world_backend_unavailable_error(
                            "world_fs.fail_closed.routing=true"
                        )
                    );
                    return Ok(exit_status_from_code(3));
                }
                if host_visible {
                    warn_world_backend_unavailable_once();
                } else {
                    warn_world_routing_failed_falling_back_to_host_once();
                }
            }
            let world_available = world_enabled && uds_exists;
            if world_available {
                let transport_meta = TransportMeta {
                    mode: "unix".to_string(),
                    endpoint: Some(socket_path.display().to_string()),
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
                    Ok(outcome) => {
                        let code = outcome.exit_code;
                        if let Some(active_span) = span.take() {
                            let mut active_span = active_span;
                            active_span.set_execution_origin(ExecutionOrigin::World);
                            active_span.set_transport(transport_meta);
                            if let Ok(resolved) =
                                crate::execution::policy_snapshot::resolve_policy_snapshot_for_cwd(
                                    &cwd_for_profile,
                                )
                            {
                                active_span.set_policy_snapshot_meta(
                                    resolved.snapshot.schema_version,
                                    resolved.snapshot_hash,
                                );
                            }
                            if let Some(meta) = outcome.fs_strategy {
                                active_span.set_world_fs_strategy(
                                    meta.primary,
                                    meta.final_strategy,
                                    meta.fallback_reason,
                                );
                            }
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
                        if let Some(unavail) = e.downcast_ref::<WorldFsStrategyUnavailableError>() {
                            if world_required {
                                if let Some(mut active_span) = span.take() {
                                    active_span.set_execution_origin(ExecutionOrigin::World);
                                    active_span.set_transport(transport_meta);
                                    if let Ok(resolved) = crate::execution::policy_snapshot::resolve_policy_snapshot_for_cwd(&cwd_for_profile) {
                                        active_span.set_policy_snapshot_meta(
                                            resolved.snapshot.schema_version,
                                            resolved.snapshot_hash,
                                        );
                                    }
                                    active_span.set_world_fs_strategy(
                                        WorldFsStrategy::Overlay,
                                        WorldFsStrategy::Fuse,
                                        unavail.fallback_reason.unwrap_or(
                                            WorldFsStrategyFallbackReason::FallbackMountFailed,
                                        ),
                                    );
                                    let _ = active_span.finish(4, vec![], None);
                                }
                                return Ok(exit_status_from_code(4));
                            }
                            if fail_closed_routing {
                                if let Some(mut active_span) = span.take() {
                                    active_span.set_execution_origin(ExecutionOrigin::World);
                                    active_span.set_transport(transport_meta);
                                    if let Ok(resolved) = crate::execution::policy_snapshot::resolve_policy_snapshot_for_cwd(&cwd_for_profile) {
                                        active_span.set_policy_snapshot_meta(
                                            resolved.snapshot.schema_version,
                                            resolved.snapshot_hash,
                                        );
                                    }
                                    active_span.set_world_fs_strategy(
                                        WorldFsStrategy::Overlay,
                                        WorldFsStrategy::Fuse,
                                        unavail.fallback_reason.unwrap_or(
                                            WorldFsStrategyFallbackReason::FallbackMountFailed,
                                        ),
                                    );
                                    let _ = active_span.finish(4, vec![], None);
                                }
                                eprintln!("substrate: error: {unavail}");
                                return Ok(exit_status_from_code(4));
                            }
                            eprintln!("substrate: warn: world unavailable; falling back to host");
                            if let Some(active_span) = span.as_mut() {
                                active_span.set_world_fs_strategy(
                                    WorldFsStrategy::Overlay,
                                    WorldFsStrategy::Host,
                                    WorldFsStrategyFallbackReason::WorldOptionalFallbackToHost,
                                );
                            }
                            // fall through to host PTY
                        } else {
                            if world_required {
                                anyhow::bail!(
                                    "world execution required ({}); PTY world path failed: {}",
                                    world_required_reason,
                                    e
                                );
                            }
                            if fail_closed_routing {
                                if let Some(mut active_span) = span.take() {
                                    active_span.set_execution_origin(ExecutionOrigin::World);
                                    active_span.set_transport(transport_meta);
                                    let _ = active_span.finish(3, vec![], None);
                                }
                                eprintln!("substrate: error: world routing failed: {e}");
                                return Ok(exit_status_from_code(3));
                            }
                            let _ = e;
                            warn_world_backend_unavailable_once();
                            // fall through to host PTY
                        }
                    }
                }
            }
        }

        // Attempt world-agent PTY WS route on mac when world is enabled
        #[cfg(target_os = "macos")]
        {
            let context = pw::get_context();
            if world_enabled && fail_closed_routing && context.is_none() {
                eprintln!(
                    "substrate: error: world routing failed; world backend unavailable on this platform (world_fs.fail_closed.routing=true)"
                );
                return Ok(exit_status_from_code(4));
            }
            let uds_exists = context
                .as_ref()
                .map(|c| match &c.transport {
                    pw::WorldTransport::Unix(p) => p.exists(),
                    _ => false,
                })
                .unwrap_or(false);
            let world_available = world_enabled && uds_exists;
            if world_required && world_enabled && !uds_exists {
                return Err(required_world_backend_unavailable_error(
                    &world_required_reason,
                ));
            }
            if world_enabled && !world_required && !uds_exists {
                if fail_closed_routing {
                    eprintln!(
                        "substrate: error: {}",
                        required_world_backend_unavailable_error(
                            "world_fs.fail_closed.routing=true"
                        )
                    );
                    return Ok(exit_status_from_code(3));
                }
                if host_visible {
                    warn_world_backend_unavailable_once();
                } else {
                    warn_world_routing_failed_falling_back_to_host_once();
                }
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
                            if let Ok(resolved) =
                                crate::execution::policy_snapshot::resolve_policy_snapshot_for_cwd(
                                    &cwd_for_profile,
                                )
                            {
                                active_span.set_policy_snapshot_meta(
                                    resolved.snapshot.schema_version,
                                    resolved.snapshot_hash,
                                );
                            }
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
                                "world execution required ({}); PTY world path failed: {}",
                                world_required_reason,
                                e
                            );
                        }
                        if fail_closed_routing {
                            if let Some(mut active_span) = span.take() {
                                if let Some(meta) = transport_meta {
                                    active_span.set_transport(meta);
                                }
                                active_span.set_execution_origin(ExecutionOrigin::World);
                                let _ = active_span.finish(3, vec![], None);
                            }
                            eprintln!("substrate: error: world routing failed: {e}");
                            return Ok(exit_status_from_code(3));
                        }
                        let _ = e;
                        warn_world_backend_unavailable_once();
                        // fall through to host PTY
                    }
                }
            }
        }

        if world_required {
            anyhow::bail!(
	                "world execution required ({}) but no world PTY execution path is available on this platform",
	                world_required_reason
	            );
        }
        if world_enabled && fail_closed_routing {
            eprintln!(
                "substrate: error: world routing failed; no world PTY execution path is available on this platform (world_fs.fail_closed.routing=true)"
            );
            return Ok(exit_status_from_code(4));
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
    let span_id_for_cmd_events = span.as_ref().map(|s| s.get_span_id().to_string());
    let start_extra = {
        let mut obj = serde_json::Map::new();
        if let Some(p) = resolved {
            obj.insert("resolved_path".to_string(), json!(p));
        }
        if let Some(span_id) = span_id_for_cmd_events.as_ref() {
            obj.insert("span_id".to_string(), json!(span_id));
        }
        (!obj.is_empty()).then_some(serde_json::Value::Object(obj))
    };
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
        let socket_override_path = std::env::var_os("SUBSTRATE_WORLD_SOCKET")
            .map(std::path::PathBuf::from)
            .filter(|p| p.exists());
        if world_enabled
            && fail_closed_routing
            && context.is_none()
            && socket_override_path.is_none()
        {
            eprintln!(
                "substrate: error: world routing failed; world backend unavailable on this platform (world_fs.fail_closed.routing=true)"
            );
            return Ok(exit_status_from_code(4));
        }
        let uds_exists = socket_override_path.is_some()
            || context
                .as_ref()
                .map(|c| matches!(&c.transport, pw::WorldTransport::Unix(path) if path.exists()))
                .unwrap_or(false);
        let world_available = world_enabled && uds_exists;
        if world_required && world_enabled && !uds_exists {
            return Err(required_world_backend_unavailable_error(
                &world_required_reason,
            ));
        }
        if world_enabled && !world_required && !uds_exists {
            if fail_closed_routing {
                eprintln!(
                    "substrate: error: {}",
                    required_world_backend_unavailable_error("world_fs.fail_closed.routing=true")
                );
                return Ok(exit_status_from_code(3));
            }
            if host_visible {
                warn_world_backend_unavailable_once();
            } else {
                warn_world_routing_failed_falling_back_to_host_once();
            }
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
                            "world execution required ({}); world-agent exec failed: {}",
                            world_required_reason,
                            e
                        );
                    }
                    if fail_closed_routing {
                        if let Some(mut active_span) = span.take() {
                            active_span.set_execution_origin(ExecutionOrigin::World);
                            if let Some(meta) = transport_meta.clone() {
                                active_span.set_transport(meta);
                            }
                            let _ = active_span.finish(3, vec![], None);
                        }
                        eprintln!("substrate: error: world routing failed: {e}");
                        return Ok(exit_status_from_code(3));
                    }
                    let _ = e;
                    if host_visible {
                        warn_world_backend_unavailable_once();
                    } else {
                        warn_world_routing_failed_falling_back_to_host_once();
                    }
                }
            }
        }
    }

    #[cfg(target_os = "windows")]
    {
        let context = pw::get_context();
        if world_enabled && fail_closed_routing && context.is_none() {
            eprintln!(
                "substrate: error: world routing failed; world backend unavailable on this platform (world_fs.fail_closed.routing=true)"
            );
            return Ok(exit_status_from_code(4));
        }
        let world_available = world_enabled && context.is_some();
        if world_required && world_enabled && context.is_none() {
            return Err(required_world_backend_unavailable_error(
                &world_required_reason,
            ));
        }
        if world_enabled && !world_required && context.is_none() {
            if host_visible {
                warn_world_backend_unavailable_once();
            } else {
                warn_world_routing_failed_falling_back_to_host_once();
            }
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
                            "world execution required ({}); world-agent exec failed: {}",
                            world_required_reason,
                            e
                        );
                    }
                    if fail_closed_routing {
                        if let Some(mut active_span) = span.take() {
                            active_span.set_execution_origin(ExecutionOrigin::World);
                            if let Some(meta) = transport_meta.clone() {
                                active_span.set_transport(meta);
                            }
                            let _ = active_span.finish(3, vec![], None);
                        }
                        eprintln!("substrate: error: world routing failed: {e}");
                        return Ok(exit_status_from_code(3));
                    }
                    let _ = e;
                    if host_visible {
                        warn_world_backend_unavailable_once();
                    } else {
                        warn_world_routing_failed_falling_back_to_host_once();
                    }
                }
            }
        }
    }

    // Handle lightweight builtins (cd/pwd/export/unset) only when we're definitively
    // running on the host.
    //
    // C5: In non-interactive `-c/--command` and stdin pipe mode, when world execution
    // is enabled, these MUST be interpreted in-world (shell semantics), not as
    // host-only builtins.
    let allow_host_lightweight_builtins = !world_enabled
        || matches!(
            &config.mode,
            ShellMode::Interactive { .. } | ShellMode::Script(_)
        );

    if allow_host_lightweight_builtins && !needs_shell(trimmed) {
        if let Some(status) = handle_builtin(config, trimmed, cmd_id)? {
            if let Some(active_span) = span {
                let _ = active_span.finish(status.code().unwrap_or(-1), vec![], None);
            }
            let mut completion_extra = json!({
                log_schema::EXIT_CODE: status.code().unwrap_or(-1),
                log_schema::DURATION_MS: start_time.elapsed().as_millis()
            });
            if let Some(span_id) = span_id_for_cmd_events.as_ref() {
                completion_extra["span_id"] = json!(span_id);
            }
            completion_extra["world_fs_strategy_primary"] =
                json!(WorldFsStrategy::Overlay.as_str());
            completion_extra["world_fs_strategy_final"] = json!(WorldFsStrategy::Host.as_str());
            completion_extra["world_fs_strategy_fallback_reason"] =
                json!(WorldFsStrategyFallbackReason::None.as_str());
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
        let socket_path = world_socket_path();
        let uds_exists = socket_path.exists();
        if world_required && world_enabled && !uds_exists {
            return Err(required_world_backend_unavailable_error(
                &world_required_reason,
            ));
        }
        if world_enabled && !world_required && !uds_exists {
            if fail_closed_routing {
                eprintln!(
                    "substrate: error: {}",
                    required_world_backend_unavailable_error("world_fs.fail_closed.routing=true")
                );
                return Ok(exit_status_from_code(3));
            }
            if host_visible {
                warn_world_backend_unavailable_once();
            } else {
                warn_world_routing_failed_falling_back_to_host_once();
            }
        }
        let world_available = world_enabled && uds_exists;
        if world_available {
            let transport_meta = TransportMeta {
                mode: "unix".to_string(),
                endpoint: Some(socket_path.display().to_string()),
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
                    if let Some(unavail) = e.downcast_ref::<WorldFsStrategyUnavailableError>() {
                        if world_required {
                            if let Some(mut active_span) = span.take() {
                                active_span.set_execution_origin(ExecutionOrigin::World);
                                active_span.set_transport(transport_meta);
                                if let Ok(resolved) = crate::execution::policy_snapshot::resolve_policy_snapshot_for_cwd(&cwd_for_profile) {
                                    active_span.set_policy_snapshot_meta(
                                        resolved.snapshot.schema_version,
                                        resolved.snapshot_hash,
                                    );
                                }
                                active_span.set_world_fs_strategy(
                                    WorldFsStrategy::Overlay,
                                    WorldFsStrategy::Fuse,
                                    unavail.fallback_reason.unwrap_or(
                                        WorldFsStrategyFallbackReason::FallbackMountFailed,
                                    ),
                                );
                                let _ = active_span.finish(4, vec![], None);
                            }
                            let reason = unavail
                                .fallback_reason
                                .unwrap_or(WorldFsStrategyFallbackReason::FallbackMountFailed);

                            let mut completion_extra = json!({
                                log_schema::EXIT_CODE: 4,

                                log_schema::DURATION_MS: start_time.elapsed().as_millis(),
                                "world_fs_strategy_primary": WorldFsStrategy::Overlay.as_str(),
                                "world_fs_strategy_final": WorldFsStrategy::Fuse.as_str(),
                                "world_fs_strategy_fallback_reason": reason.as_str(),
                            });
                            if let Some(span_id) = span_id_for_cmd_events.as_ref() {
                                completion_extra["span_id"] = json!(span_id);
                            }
                            log_command_event(
                                config,
                                "command_complete",
                                &redacted_command,
                                cmd_id,
                                Some(completion_extra),
                            )?;
                            return Ok(exit_status_from_code(4));
                        }
                        if fail_closed_routing {
                            if let Some(mut active_span) = span.take() {
                                active_span.set_execution_origin(ExecutionOrigin::World);
                                active_span.set_transport(transport_meta);
                                active_span.set_world_fs_strategy(
                                    WorldFsStrategy::Overlay,
                                    WorldFsStrategy::Fuse,
                                    unavail.fallback_reason.unwrap_or(
                                        WorldFsStrategyFallbackReason::FallbackMountFailed,
                                    ),
                                );
                                let _ = active_span.finish(4, vec![], None);
                            }
                            eprintln!("substrate: error: {unavail}");
                            return Ok(exit_status_from_code(4));
                        }

                        if host_visible {
                            eprintln!("substrate: warn: world unavailable; falling back to host");
                        } else {
                            warn_world_routing_failed_falling_back_to_host_once();
                        }
                        if let Some(active_span) = span.as_mut() {
                            active_span.set_world_fs_strategy(
                                WorldFsStrategy::Overlay,
                                WorldFsStrategy::Host,
                                WorldFsStrategyFallbackReason::WorldOptionalFallbackToHost,
                            );
                        }
                        world_fs_strategy_log_override = Some((
                            WorldFsStrategy::Overlay,
                            WorldFsStrategy::Host,
                            WorldFsStrategyFallbackReason::WorldOptionalFallbackToHost,
                        ));
                    } else {
                        if world_required {
                            anyhow::bail!(
                                "world execution required ({}); world-agent exec failed: {}",
                                world_required_reason,
                                e
                            );
                        }
                        if fail_closed_routing {
                            if let Some(mut active_span) = span.take() {
                                active_span.set_execution_origin(ExecutionOrigin::World);
                                active_span.set_transport(transport_meta);
                                let _ = active_span.finish(3, vec![], None);
                            }
                            eprintln!("substrate: error: world routing failed: {e}");
                            return Ok(exit_status_from_code(3));
                        }
                        let _ = e;
                        if host_visible {
                            warn_world_backend_unavailable_once();
                        } else {
                            warn_world_routing_failed_falling_back_to_host_once();
                        }
                    }
                }
            }
        }
    }

    if let Some(outcome) = agent_result {
        let mut final_exit_code = outcome.exit_code;

        if outcome.exit_code == 0 {
            let auto_sync_exit_code =
                crate::execution::run_auto_sync_if_enabled(config, &effective_config)?;
            if auto_sync_exit_code != 0 {
                final_exit_code = auto_sync_exit_code;
            }
        }

        if let Some(mut active_span) = span {
            if let Ok(resolved) =
                crate::execution::policy_snapshot::resolve_policy_snapshot_for_cwd(&cwd_for_profile)
            {
                active_span.set_policy_snapshot_meta(
                    resolved.snapshot.schema_version,
                    resolved.snapshot_hash,
                );
            }
            if let Some(meta) = outcome.fs_strategy {
                active_span.set_world_fs_strategy(
                    meta.primary,
                    meta.final_strategy,
                    meta.fallback_reason,
                );
            }
            let _ = active_span.finish(
                final_exit_code,
                outcome.scopes_used.clone(),
                outcome.fs_diff.clone(),
            );
        }
        let mut completion_extra = json!({
            log_schema::EXIT_CODE: final_exit_code,
            log_schema::DURATION_MS: start_time.elapsed().as_millis()
        });
        if let Some(span_id) = span_id_for_cmd_events.as_ref() {
            completion_extra["span_id"] = json!(span_id);
        }
        if let Some(meta) = outcome.fs_strategy {
            completion_extra["world_fs_strategy_primary"] = json!(meta.primary.as_str());
            completion_extra["world_fs_strategy_final"] = json!(meta.final_strategy.as_str());
            completion_extra["world_fs_strategy_fallback_reason"] =
                json!(meta.fallback_reason.as_str());
        } else {
            completion_extra["world_fs_strategy_primary"] =
                json!(WorldFsStrategy::Overlay.as_str());
            completion_extra["world_fs_strategy_final"] = json!(WorldFsStrategy::Host.as_str());
            completion_extra["world_fs_strategy_fallback_reason"] =
                json!(WorldFsStrategyFallbackReason::None.as_str());
        }
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
            return Ok(ExitStatus::from_raw((final_exit_code & 0xff) << 8));
        }
        #[cfg(windows)]
        {
            use std::os::windows::process::ExitStatusExt;
            return Ok(ExitStatus::from_raw(final_exit_code as u32));
        }
    }

    if world_required {
        anyhow::bail!(
	            "world execution required (fs_mode={}) but no world execution path is available on this platform",
	            fs_mode.as_str()
	        );
    }
    if world_enabled && fail_closed_routing {
        eprintln!(
            "substrate: error: world routing failed; no world execution path is available on this platform (world_fs.fail_closed.routing=true)"
        );
        return Ok(exit_status_from_code(4));
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
    let mut extra = json!({
        log_schema::EXIT_CODE: status.code().unwrap_or(-1),
        log_schema::DURATION_MS: duration.as_millis()
    });

    if let Some(span_id) = span_id_for_cmd_events.as_ref() {
        extra["span_id"] = json!(span_id);
    }

    #[cfg(unix)]
    {
        if let Some(sig) = status.signal() {
            extra["term_signal"] = json!(sig);
        }
    }

    if let Some((primary, final_strategy, reason)) = world_fs_strategy_log_override {
        extra["world_fs_strategy_primary"] = json!(primary.as_str());
        extra["world_fs_strategy_final"] = json!(final_strategy.as_str());
        extra["world_fs_strategy_fallback_reason"] = json!(reason.as_str());
    } else {
        extra["world_fs_strategy_primary"] = json!(WorldFsStrategy::Overlay.as_str());
        extra["world_fs_strategy_final"] = json!(WorldFsStrategy::Host.as_str());
        extra["world_fs_strategy_fallback_reason"] =
            json!(WorldFsStrategyFallbackReason::None.as_str());
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
        let mut cmd_command = command_for_shell;
        if cmd_command.contains("%SUBSTRATE_") && cmd_command.contains('|') {
            cmd_command = cmd_command.replace('|', "^|");
        }
        cmd.arg("/C").arg(cmd_command);
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
