//! Trace and replay helpers for routing.

use crate::execution::cli::Cli;
use crate::execution::config_model::{self, CliConfigOverrides, WorldDisableAttribution};
use crate::execution::value_parse::parse_bool_flag;
#[cfg(test)]
use crate::execution::world_env_guard;
use anyhow::{anyhow, Context, Result};
use serde_json::json;
use std::collections::HashMap;
use std::env;
use std::path::Path;
use std::path::PathBuf;
use std::process::Stdio;
use substrate_common::WorldRootMode;
use substrate_replay::replay::record_replay_strategy;
use substrate_replay::state::{load_span_from_trace, reconstruct_state};
use substrate_replay::ReplayPlatformBootstrapInputV1;
use substrate_trace::ExecutionOrigin;
use transport_api_types::{InstallBootstrapContextCarrierV1, PlatformBootstrapMappingV1};

#[derive(Debug)]
enum ReplayWorldSource {
    Recorded,
    ForceWorldFlag,
    NoWorldFlag,
    EnvDisabled { raw: String },
}

impl ReplayWorldSource {
    fn reason_code(&self, flipped: bool) -> &'static str {
        match self {
            ReplayWorldSource::ForceWorldFlag => "flag_world",
            ReplayWorldSource::NoWorldFlag => "flag_no_world",
            ReplayWorldSource::EnvDisabled { .. } => "env_disabled",
            ReplayWorldSource::Recorded => {
                if flipped {
                    "flip_world"
                } else {
                    "recorded_origin"
                }
            }
        }
    }
}

#[derive(Debug, Clone, Copy)]
enum RecordedOriginSource {
    Span,
    ReplayContext,
    DefaultWorld,
}

#[derive(Debug)]
struct ReplayWorldMode {
    recorded: ExecutionOrigin,
    recorded_source: RecordedOriginSource,
    selected: ExecutionOrigin,
    source: ReplayWorldSource,
    flipped: bool,
    effective_disable_attribution: Option<WorldDisableAttribution>,
}

impl ReplayWorldMode {
    fn from_inputs(
        recorded: ExecutionOrigin,
        recorded_source: RecordedOriginSource,
        flip_requested: bool,
        cli: &Cli,
    ) -> Self {
        let env_override = env::var("SUBSTRATE_REPLAY_USE_WORLD").ok();
        if cli.world {
            return Self {
                recorded,
                recorded_source,
                selected: ExecutionOrigin::World,
                source: ReplayWorldSource::ForceWorldFlag,
                flipped: flip_requested,
                effective_disable_attribution: None,
            };
        }
        if cli.no_world {
            return Self {
                recorded,
                recorded_source,
                selected: ExecutionOrigin::Host,
                source: ReplayWorldSource::NoWorldFlag,
                flipped: flip_requested,
                effective_disable_attribution: None,
            };
        }

        let mut selected = recorded;
        let mut flipped = false;
        if flip_requested {
            selected = selected.flipped();
            flipped = true;
        }

        let mut source = ReplayWorldSource::Recorded;
        if let Some(raw) = env_override.clone() {
            let lowered = raw.to_ascii_lowercase();
            if lowered == "0" || lowered == "disabled" || lowered == "false" {
                selected = ExecutionOrigin::Host;
                source = ReplayWorldSource::EnvDisabled { raw };
            }
        }

        Self {
            recorded,
            recorded_source,
            selected,
            source,
            flipped,
            effective_disable_attribution: None,
        }
    }

    fn with_effective_disable_attribution(
        mut self,
        effective_disable_attribution: Option<WorldDisableAttribution>,
    ) -> Self {
        self.effective_disable_attribution = effective_disable_attribution;
        self
    }

    fn selected_origin(&self) -> ExecutionOrigin {
        self.selected
    }

    fn apply_env(&self) {
        #[cfg(test)]
        let _env_guard = world_env_guard();

        if self.selected == ExecutionOrigin::World {
            env::set_var("SUBSTRATE_WORLD", "enabled");
            env::set_var("SUBSTRATE_WORLD_ENABLED", "1");
            env::remove_var("SUBSTRATE_REPLAY_USE_WORLD");
        } else {
            env::set_var("SUBSTRATE_WORLD", "disabled");
            env::set_var("SUBSTRATE_WORLD_ENABLED", "0");
            env::set_var("SUBSTRATE_REPLAY_USE_WORLD", "disabled");
        }
    }

    fn reason_text(&self) -> String {
        match &self.source {
            ReplayWorldSource::ForceWorldFlag => "--world flag".to_string(),
            ReplayWorldSource::NoWorldFlag => "--no-world flag".to_string(),
            ReplayWorldSource::EnvDisabled { raw } => {
                format!("SUBSTRATE_REPLAY_USE_WORLD={raw}")
            }
            ReplayWorldSource::Recorded => {
                if self.flipped {
                    "--flip-world".to_string()
                } else {
                    match self.recorded_source {
                        RecordedOriginSource::Span => "recorded origin (span)".to_string(),
                        RecordedOriginSource::ReplayContext => {
                            "recorded origin (replay_context)".to_string()
                        }
                        RecordedOriginSource::DefaultWorld => "default origin".to_string(),
                    }
                }
            }
        }
    }

    fn reason_code(&self) -> &'static str {
        if self.uses_effective_disable_attribution() {
            return self.effective_disable_reason_code();
        }
        self.source.reason_code(self.flipped)
    }

    fn effective_disable_reason_code(&self) -> &'static str {
        let attribution = self
            .effective_disable_attribution
            .as_ref()
            .expect("effective-disable attribution should be present");
        match attribution.source.layer {
            "override_env" => "world_disabled_override_env",
            "workspace_patch" => "world_disabled_workspace_patch",
            "global_patch" => "world_disabled_global_patch",
            "source_unknown" => "world_disabled_unknown",
            "default" => panic!(
                "replay effective-disable attribution reached non-publishable helper layer `default`; block S3"
            ),
            other => panic!(
                "replay effective-disable attribution reached unexpected helper layer `{other}`; block S3"
            ),
        }
    }

    fn uses_effective_disable_attribution(&self) -> bool {
        self.selected == ExecutionOrigin::Host
            && self.recorded == ExecutionOrigin::Host
            && !self.flipped
            && matches!(self.source, ReplayWorldSource::Recorded)
            && self.effective_disable_attribution.is_some()
    }

    fn display_reason_text(&self) -> String {
        if self.uses_effective_disable_attribution() {
            return self
                .effective_disable_attribution
                .as_ref()
                .expect("effective disable attribution should be present")
                .reason
                .to_string();
        }
        self.reason_text()
    }

    fn effective_disable_source(&self) -> Option<serde_json::Value> {
        if self.uses_effective_disable_attribution() {
            let source = &self
                .effective_disable_attribution
                .as_ref()
                .expect("effective disable attribution should be present")
                .source;
            let mut value =
                serde_json::to_value(source).expect("world disable source should serialize");
            if let Some(layer) = value.get_mut("layer") {
                *layer = json!(match source.layer {
                    "override_env" => "override_env",
                    "workspace_patch" => "workspace_patch",
                    "global_patch" => "global_patch",
                    "source_unknown" => "unknown",
                    "default" => panic!(
                        "replay effective-disable attribution reached non-publishable helper layer `default`; block S3"
                    ),
                    other => panic!(
                        "replay effective-disable attribution reached unexpected helper layer `{other}`; block S3"
                    ),
                });
            }
            return Some(value);
        }
        None
    }

    fn summary(&self) -> String {
        let recorded_label = match self.recorded_source {
            RecordedOriginSource::DefaultWorld => format!("{} (default)", self.recorded.as_str()),
            _ => format!("{} (recorded)", self.recorded.as_str()),
        };

        if self.recorded == self.selected
            && !self.flipped
            && !matches!(self.source, ReplayWorldSource::EnvDisabled { .. })
        {
            if self.selected == ExecutionOrigin::Host {
                return format!(
                    "[replay] origin: host (recorded; {})",
                    self.display_reason_text()
                );
            }
            return format!("[replay] origin: {}", recorded_label);
        }

        let direction = format!("{} -> {}", self.recorded.as_str(), self.selected.as_str());
        let mut reason = self.display_reason_text();
        if self.flipped && matches!(self.source, ReplayWorldSource::EnvDisabled { .. }) {
            reason.push_str("; flip requested");
        }
        format!("[replay] origin: {direction} ({reason})")
    }

    fn warn_reason(&self) -> Option<String> {
        if self.selected == ExecutionOrigin::World {
            return match &self.source {
                ReplayWorldSource::ForceWorldFlag => Some("--world flag".to_string()),
                ReplayWorldSource::Recorded
                    if self.flipped && self.recorded == ExecutionOrigin::Host =>
                {
                    Some("--flip-world".to_string())
                }
                _ => None,
            };
        }
        match &self.source {
            ReplayWorldSource::Recorded if self.uses_effective_disable_attribution() => {
                Some(self.display_reason_text())
            }
            ReplayWorldSource::NoWorldFlag => Some("--no-world flag".to_string()),
            ReplayWorldSource::EnvDisabled { raw } => {
                Some(format!("SUBSTRATE_REPLAY_USE_WORLD={raw}"))
            }
            ReplayWorldSource::Recorded if self.flipped => Some("--flip-world".to_string()),
            ReplayWorldSource::Recorded if self.recorded == ExecutionOrigin::World => {
                Some("recorded origin=world".to_string())
            }
            _ => None,
        }
    }
}

#[doc(hidden)]
pub fn replay_unknown_effective_disable_attribution_fixture() -> serde_json::Value {
    let mode = ReplayWorldMode {
        recorded: ExecutionOrigin::Host,
        recorded_source: RecordedOriginSource::Span,
        selected: ExecutionOrigin::Host,
        source: ReplayWorldSource::Recorded,
        flipped: false,
        effective_disable_attribution: Some(WorldDisableAttribution {
            reason: "world isolation disabled by effective config (source unknown)",
            source: crate::execution::config_model::WorldDisableSource {
                key: "world.enabled",
                layer: "source_unknown",
                value_display: false,
                flag: None,
                env: None,
                path_display: None,
            },
        }),
    };

    json!({
        "summary": mode.summary(),
        "reason_code": mode.reason_code(),
        "world_disable_source": mode
            .effective_disable_source()
            .expect("expected unknown-source effective-disable attribution"),
    })
}

fn resolve_replay_effective_disable_attribution(cwd: &Path) -> Option<WorldDisableAttribution> {
    let cli = CliConfigOverrides {
        world_enabled: None,
        anchor_mode: None,
        anchor_path: None,
        caged: None,
    };
    let (effective, explain) =
        config_model::resolve_effective_config_with_explain(cwd, &cli, true).ok()?;
    config_model::world_disable_attribution(
        effective.world.enabled,
        explain
            .as_ref()
            .and_then(|value| value.world_enabled_explain()),
    )
}

fn apply_replay_world_mode_env(env: &mut HashMap<String, String>, mode: &ReplayWorldMode) {
    if mode.selected_origin() == ExecutionOrigin::World {
        env.insert("SUBSTRATE_WORLD".to_string(), "enabled".to_string());
        env.insert("SUBSTRATE_WORLD_ENABLED".to_string(), "1".to_string());
        env.remove("SUBSTRATE_REPLAY_USE_WORLD");
    } else {
        env.insert("SUBSTRATE_WORLD".to_string(), "disabled".to_string());
        env.insert("SUBSTRATE_WORLD_ENABLED".to_string(), "0".to_string());
        env.insert(
            "SUBSTRATE_REPLAY_USE_WORLD".to_string(),
            "disabled".to_string(),
        );
    }
}

fn decode_replay_platform_bootstrap_input(raw: &str) -> Result<ReplayPlatformBootstrapInputV1> {
    let parsed: ReplayPlatformBootstrapInputV1 =
        serde_json::from_str(raw).context("invalid replay platform bootstrap input JSON")?;
    let encoded_host = parsed
        .host_carrier
        .encode()
        .context("replay platform host carrier is invalid")?;
    let host_carrier = InstallBootstrapContextCarrierV1::decode(&encoded_host)
        .context("replay platform host carrier is not canonical")?;
    let encoded_mapping = parsed
        .platform_bootstrap_mapping
        .encode(&host_carrier)
        .context("replay platform bootstrap mapping is invalid")?;
    let platform_bootstrap_mapping =
        PlatformBootstrapMappingV1::decode(&encoded_mapping, &host_carrier)
            .context("replay platform bootstrap mapping is not canonical")?;
    Ok(ReplayPlatformBootstrapInputV1 {
        host_carrier,
        platform_bootstrap_mapping,
    })
}

fn recorded_origin_source(
    state: &substrate_replay::replay::ExecutionState,
) -> RecordedOriginSource {
    match state
        .recorded_origin_source
        .as_deref()
        .unwrap_or("default_world")
    {
        "span" => RecordedOriginSource::Span,
        "replay_context" => RecordedOriginSource::ReplayContext,
        _ => RecordedOriginSource::DefaultWorld,
    }
}

fn inject_world_root_env(env: &mut HashMap<String, String>, cwd: &Path) {
    let mode = env
        .get("SUBSTRATE_ANCHOR_MODE")
        .and_then(|value| WorldRootMode::parse(value))
        .unwrap_or(WorldRootMode::Project);

    let root_path = env
        .get("SUBSTRATE_ANCHOR_PATH")
        .map(|value| value.trim())
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
        .unwrap_or_else(|| cwd.to_path_buf());

    let path = match mode {
        WorldRootMode::FollowCwd => cwd.to_path_buf(),
        _ => root_path,
    };

    let caged = env
        .get("SUBSTRATE_CAGED")
        .and_then(|value| parse_bool_flag(value))
        .unwrap_or(true);

    env.entry("SUBSTRATE_ANCHOR_MODE".to_string())
        .or_insert_with(|| mode.as_str().to_string());
    env.entry("SUBSTRATE_ANCHOR_PATH".to_string())
        .or_insert_with(|| path.to_string_lossy().to_string());
    env.entry("SUBSTRATE_CAGED".to_string())
        .or_insert_with(|| if caged { "1" } else { "0" }.to_string());
}

fn request_carried_project_dir(
    span: &substrate_replay::state::TraceSpan,
) -> Result<(WorldRootMode, PathBuf)> {
    let request_mode = span
        .replay_context
        .as_ref()
        .and_then(|ctx| ctx.anchor_mode.as_deref())
        .and_then(WorldRootMode::parse)
        .unwrap_or(WorldRootMode::Project);
    let request_anchor_path = span
        .replay_context
        .as_ref()
        .and_then(|ctx| ctx.anchor_path.as_deref())
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(PathBuf::from);
    let request_cwd = span
        .cwd
        .as_ref()
        .filter(|path| !path.as_os_str().is_empty())
        .cloned();

    let project_dir = match request_mode {
        WorldRootMode::Project => request_cwd.ok_or_else(|| {
            anyhow!(
                "Windows world replay requires an explicit project path from the replay request"
            )
        })?,
        WorldRootMode::FollowCwd => request_cwd.ok_or_else(|| {
            anyhow!(
                "Windows world replay requires an explicit project path from the replay request"
            )
        })?,
        WorldRootMode::Custom => request_anchor_path.ok_or_else(|| {
            anyhow!(
                "Windows world replay requires an explicit project path from the replay request"
            )
        })?,
    };

    if project_dir.as_os_str().is_empty() {
        return Err(anyhow!(
            "Windows world replay requires an explicit project path"
        ));
    }

    Ok((request_mode, project_dir))
}

fn apply_request_carried_windows_project_selector(
    state: &mut substrate_replay::replay::ExecutionState,
    span: &substrate_replay::state::TraceSpan,
) -> Result<()> {
    if !cfg!(target_os = "windows")
        || state.platform_bootstrap_mapping.is_none()
        || state.target_origin != ExecutionOrigin::World
    {
        return Ok(());
    }

    let (mode, project_dir) = request_carried_project_dir(span)?;
    state.env.insert(
        "SUBSTRATE_ANCHOR_MODE".to_string(),
        mode.as_str().to_string(),
    );
    match mode {
        WorldRootMode::Project | WorldRootMode::Custom => {
            state.env.insert(
                "SUBSTRATE_ANCHOR_PATH".to_string(),
                project_dir.to_string_lossy().to_string(),
            );
        }
        WorldRootMode::FollowCwd => {
            state.env.remove("SUBSTRATE_ANCHOR_PATH");
        }
    }
    Ok(())
}

pub(crate) fn handle_trace_command(span_id: &str) -> Result<()> {
    use std::fs::File;
    use std::io::{BufRead, BufReader};

    // Get trace file location
    let trace_file = env::var("SHIM_TRACE_LOG")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            dirs::home_dir()
                .expect("Cannot determine home directory")
                .join(".substrate/trace.jsonl")
        });

    if !trace_file.exists() {
        eprintln!("Trace file not found: {}", trace_file.display());
        eprintln!("Make sure tracing is enabled (set SHIM_TRACE_LOG or avoid disabling tracing via SUBSTRATE_WORLD)");
        std::process::exit(1);
    }

    // Read trace file and find the span
    let file = File::open(&trace_file)?;
    let reader = BufReader::new(file);
    let mut found: Option<serde_json::Value> = None;

    for line in reader.lines() {
        let line = line?;
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(&line) {
            if let Some(id) = json.get("span_id").and_then(|v| v.as_str()) {
                if id == span_id {
                    // Prefer command_complete if multiple entries exist
                    let is_complete =
                        json.get("event_type").and_then(|v| v.as_str()) == Some("command_complete");
                    match &found {
                        None => found = Some(json),
                        Some(current) => {
                            let current_is_complete =
                                current.get("event_type").and_then(|v| v.as_str())
                                    == Some("command_complete");
                            if is_complete && !current_is_complete {
                                found = Some(json);
                            }
                        }
                    }
                }
            }
        }
    }

    if let Some(json) = found {
        println!("{}", serde_json::to_string_pretty(&json)?);
    } else {
        eprintln!("Span ID not found: {}", span_id);
        std::process::exit(1);
    }

    Ok(())
}

/// Handle replay command - replay a traced command by span ID
pub(crate) fn handle_replay_command(span_id: &str, cli: &Cli) -> Result<()> {
    let verbose_requested =
        std::env::var("SUBSTRATE_REPLAY_VERBOSE").unwrap_or_default() == "1" || cli.replay_verbose;

    let trace_file = std::env::var("SHIM_TRACE_LOG")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            dirs::home_dir()
                .expect("Cannot determine home directory")
                .join(".substrate/trace.jsonl")
        });

    if !trace_file.exists() {
        eprintln!("Trace file not found: {}", trace_file.display());
        eprintln!("Make sure tracing is enabled (set SHIM_TRACE_LOG or avoid disabling tracing via SUBSTRATE_WORLD)");
        std::process::exit(1);
    }

    let runtime = tokio::runtime::Runtime::new()?;

    // Reconstruct state from the trace entry (includes PATH/user/env metadata when captured)
    let span = runtime.block_on(async { load_span_from_trace(&trace_file, span_id).await })?;
    let mut state = reconstruct_state(&span, &HashMap::new(), None)?;

    // Verbose header
    if verbose_requested {
        eprintln!("[replay] span_id: {}", span_id);
        eprintln!("[replay] command: {}", state.raw_cmd);
        eprintln!("[replay] cwd: {}", state.cwd.display());
        eprintln!("[replay] mode: bash -lc");
    }

    // Respect replay toggle precedence: --world > --no-world > SUBSTRATE_REPLAY_USE_WORLD
    let recorded_source = recorded_origin_source(&state);
    let effective_disable_attribution = resolve_replay_effective_disable_attribution(&state.cwd);
    let replay_world_mode =
        ReplayWorldMode::from_inputs(state.recorded_origin, recorded_source, cli.flip_world, cli)
            .with_effective_disable_attribution(effective_disable_attribution);

    if let Some(attribution) = replay_world_mode.effective_disable_attribution.as_ref() {
        if attribution.source.layer == "default" {
            return Err(anyhow!(
                "replay effective-disable attribution reached non-publishable helper layer `default`; block S3"
            ));
        }
    }

    state.target_origin = replay_world_mode.selected_origin();
    state.origin_reason = Some(replay_world_mode.display_reason_text());
    state.origin_reason_code = Some(replay_world_mode.reason_code().to_string());
    state.world_disable_source = replay_world_mode.effective_disable_source();

    let use_world = replay_world_mode.selected_origin() == ExecutionOrigin::World;
    if use_world && cfg!(any(target_os = "macos", target_os = "windows")) {
        state.platform_bootstrap_mapping = cli
            .replay_platform_bootstrap_input_v1
            .as_deref()
            .map(decode_replay_platform_bootstrap_input)
            .transpose()?;
    }
    apply_request_carried_windows_project_selector(&mut state, &span)?;

    apply_replay_world_mode_env(&mut state.env, &replay_world_mode);
    inject_world_root_env(&mut state.env, &state.cwd);
    replay_world_mode.apply_env();

    if verbose_requested {
        eprintln!("{}", replay_world_mode.summary());
    }
    if replay_world_mode.selected_origin() == ExecutionOrigin::Host && verbose_requested {
        if let Some(reason) = replay_world_mode.warn_reason() {
            eprintln!("[replay] warn: running on host ({reason})");
        }
    }

    // Best-effort capability warnings when world isolation requested but not available
    if cfg!(target_os = "linux") && use_world {
        // cgroup v2
        if !PathBuf::from("/sys/fs/cgroup/cgroup.controllers").exists() {
            eprintln!("[replay] warn: cgroup v2 not mounted; world cgroups will not activate");
        }
        // overlayfs
        let overlay_ok = std::fs::read_to_string("/proc/filesystems")
            .ok()
            .map(|s| s.contains("overlay"))
            .unwrap_or(false);
        if !overlay_ok {
            eprintln!("[replay] warn: overlayfs not present; fs_diff will be unavailable");
        }
        // nftables
        let nft_ok = std::process::Command::new("nft")
            .arg("--version")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .ok()
            .map(|s| s.success())
            .unwrap_or(false);
        if !nft_ok {
            eprintln!("[replay] warn: nft not available; netfilter scoping/logging disabled");
        }
        // dmesg restrict
        if let Ok(out) = std::process::Command::new("sh")
            .arg("-lc")
            .arg("sysctl -n kernel.dmesg_restrict 2>/dev/null || echo n/a")
            .output()
        {
            if let Ok(s) = String::from_utf8(out.stdout) {
                if s.trim() == "1" {
                    eprintln!(
                        "[replay] warn: kernel.dmesg_restrict=1; LOG lines may not be visible"
                    );
                }
            }
        }
    }

    if !use_world {
        record_replay_strategy(
            &state,
            "host",
            None,
            replay_world_mode.warn_reason().as_deref(),
            json!({
                "origin_summary": replay_world_mode.summary(),
            }),
        );
    }

    let result = if use_world {
        runtime.block_on(async { substrate_replay::replay::execute_in_world(&state, 60).await })?
    } else {
        runtime.block_on(async { substrate_replay::replay::execute_direct(&state, 60).await })?
    };

    // Display results
    println!("Exit code: {}", result.exit_code);
    if !result.stdout.is_empty() {
        println!("\nStdout:");
        println!("{}", String::from_utf8_lossy(&result.stdout));
    }
    if !result.stderr.is_empty() {
        println!("\nStderr:");
        println!("{}", String::from_utf8_lossy(&result.stderr));
    }

    if let Some(fs_diff) = result.fs_diff {
        if !fs_diff.is_empty() {
            println!("\nFilesystem changes:");
            for write in &fs_diff.writes {
                println!("  + {}", write.display());
            }
            for modify in &fs_diff.mods {
                println!("  ~ {}", modify.display());
            }
            for delete in &fs_diff.deletes {
                println!("  - {}", delete.display());
            }
        }
    }

    std::process::exit(result.exit_code);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::execution::config_model::WorldDisableSource;
    use chrono::Utc;

    fn replay_platform_bootstrap_input_json() -> String {
        json!({
            "host_carrier": {
                "context": {
                    "selected_host_prefix": "/tmp/substrate",
                    "host_substrate_home": "/tmp/substrate",
                    "host_substrate_root": "/tmp/substrate",
                    "intended_host_principal": {
                        "kind": "unix",
                        "account": "alice",
                        "uid": 1000
                    }
                },
                "host_context_commitment": "544d04a3e88530f5c5bc5f2af6d139e9c5c1819c5ff0c954a71d104445f8dfdb"
            },
            "platform_bootstrap_mapping": {
                "host_context_commitment": "544d04a3e88530f5c5bc5f2af6d139e9c5c1819c5ff0c954a71d104445f8dfdb",
                "platform_instance": {
                    "kind": "lima",
                    "vm_name": "substrate",
                    "guest_machine_id": "0123456789abcdef0123456789abcdef"
                },
                "host_platform_control_root": "/Users/alice/.lima",
                "realized_substrate_home": "/home/substrate/.substrate",
                "realized_principal": {
                    "kind": "unix",
                    "account": "substrate",
                    "uid": 1000
                },
                "realized_transport": {
                    "kind": "lima",
                    "host_socket": "/tmp/substrate/sock/agent.sock",
                    "guest_socket": "/run/substrate.sock"
                }
            }
        })
        .to_string()
    }

    fn trace_span_with_request_path(
        cwd: Option<&str>,
        anchor_mode: Option<&str>,
        anchor_path: Option<&str>,
    ) -> substrate_replay::state::TraceSpan {
        substrate_replay::state::TraceSpan {
            ts: Utc::now(),
            event_type: "command_complete".to_string(),
            span_id: "span".to_string(),
            session_id: "session".to_string(),
            component: "shell".to_string(),
            cmd: "echo hi".to_string(),
            cwd: cwd.map(PathBuf::from),
            exit_code: Some(0),
            duration_ms: Some(1),
            policy_decision: None,
            fs_diff: None,
            scopes_used: None,
            replay_context: Some(substrate_trace::ReplayContext {
                path: None,
                env_hash: "hash".to_string(),
                umask: 0,
                locale: None,
                cwd: cwd.unwrap_or_default().to_string(),
                policy_id: "policy".to_string(),
                policy_commit: None,
                world_image_version: "test".to_string(),
                hostname: None,
                user: None,
                shell: None,
                term: None,
                world_image: None,
                execution_origin: Some(ExecutionOrigin::World),
                transport: None,
                anchor_mode: anchor_mode.map(str::to_string),
                anchor_path: anchor_path.map(str::to_string),
                world_root_mode: None,
                world_root_path: None,
                caged: Some(true),
                world_fs_mode: None,
            }),
            transport: None,
            execution_origin: Some(ExecutionOrigin::World),
            stdout: None,
            stderr: None,
            env_hash: None,
        }
    }

    #[test]
    fn recorded_host_summary_uses_effective_disable_source_unknown_reason() {
        let mode = ReplayWorldMode {
            recorded: ExecutionOrigin::Host,
            recorded_source: RecordedOriginSource::Span,
            selected: ExecutionOrigin::Host,
            source: ReplayWorldSource::Recorded,
            flipped: false,
            effective_disable_attribution: Some(WorldDisableAttribution {
                reason: "world isolation disabled by effective config (source unknown)",
                source: WorldDisableSource {
                    key: "world.enabled",
                    layer: "source_unknown",
                    value_display: false,
                    flag: None,
                    env: None,
                    path_display: None,
                },
            }),
        };

        assert_eq!(
            mode.summary(),
            "[replay] origin: host (recorded; world isolation disabled by effective config (source unknown))"
        );
        assert_eq!(
            mode.warn_reason().as_deref(),
            Some("world isolation disabled by effective config (source unknown)")
        );
        assert_eq!(mode.reason_code(), "world_disabled_unknown");
        assert_eq!(
            mode.effective_disable_source(),
            Some(json!({
                "key": "world.enabled",
                "layer": "unknown",
                "value_display": false,
            }))
        );
    }

    #[test]
    fn decode_replay_platform_bootstrap_input_rejects_malformed_json() {
        let err = decode_replay_platform_bootstrap_input("{not-json")
            .expect_err("malformed replay authority input must fail");

        assert!(err
            .to_string()
            .contains("invalid replay platform bootstrap input JSON"));
    }

    #[test]
    fn decode_replay_platform_bootstrap_input_rejects_noncanonical_commitment() {
        let mut value: serde_json::Value =
            serde_json::from_str(&replay_platform_bootstrap_input_json()).expect("valid JSON");
        value["host_carrier"]["host_context_commitment"] = json!("not-a-canonical-commitment");

        let err = decode_replay_platform_bootstrap_input(
            &serde_json::to_string(&value).expect("serialize invalid replay authority input"),
        )
        .expect_err("noncanonical replay authority input must fail");

        assert!(err
            .to_string()
            .contains("replay platform host carrier is invalid"));
    }

    #[test]
    fn request_carried_project_dir_defaults_to_span_cwd_without_anchor_selector() {
        let span = trace_span_with_request_path(Some("C:/request/repo"), None, None);

        let (mode, project_dir) =
            request_carried_project_dir(&span).expect("request-carried project path");

        assert_eq!(mode, WorldRootMode::Project);
        assert_eq!(project_dir, PathBuf::from("C:/request/repo"));
    }

    #[test]
    fn request_carried_project_dir_rejects_missing_request_path() {
        let span = trace_span_with_request_path(None, None, None);

        let err = request_carried_project_dir(&span)
            .expect_err("missing request-carried project path must fail");

        assert!(err
            .to_string()
            .contains("explicit project path from the replay request"));
    }
}
