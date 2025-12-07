//! Trace and replay helpers for routing.

use crate::execution::cli::Cli;
use crate::execution::settings::world_root_from_env;
use anyhow::Result;
use std::collections::HashMap;
use std::env;
use std::path::PathBuf;
use std::process::Stdio;
use substrate_replay::state::{load_span_from_trace, reconstruct_state};

#[derive(Debug)]
enum ReplayWorldSource {
    Default,
    ForceWorldFlag,
    NoWorldFlag,
    EnvDisabled { raw: String },
}

impl ReplayWorldSource {
    fn summary(&self, use_world: bool) -> String {
        let state = if use_world { "enabled" } else { "disabled" };
        let reason = match self {
            ReplayWorldSource::Default => "default",
            ReplayWorldSource::ForceWorldFlag => "--world flag",
            ReplayWorldSource::NoWorldFlag => "--no-world flag",
            ReplayWorldSource::EnvDisabled { .. } => "SUBSTRATE_REPLAY_USE_WORLD override",
        };
        format!("[replay] world toggle: {state} ({reason})")
    }

    fn warn_reason(&self) -> Option<String> {
        match self {
            ReplayWorldSource::NoWorldFlag => Some("--no-world flag".to_string()),
            ReplayWorldSource::EnvDisabled { raw } => {
                Some(format!("SUBSTRATE_REPLAY_USE_WORLD={raw}"))
            }
            _ => None,
        }
    }
}

#[derive(Debug)]
struct ReplayWorldMode {
    use_world: bool,
    source: ReplayWorldSource,
}

impl ReplayWorldMode {
    fn from_cli(cli: &Cli) -> Self {
        if cli.world {
            return Self {
                use_world: true,
                source: ReplayWorldSource::ForceWorldFlag,
            };
        }
        if cli.no_world {
            return Self {
                use_world: false,
                source: ReplayWorldSource::NoWorldFlag,
            };
        }
        if let Ok(raw) = env::var("SUBSTRATE_REPLAY_USE_WORLD") {
            let lowered = raw.to_ascii_lowercase();
            if lowered == "0" || lowered == "disabled" || lowered == "false" {
                return Self {
                    use_world: false,
                    source: ReplayWorldSource::EnvDisabled { raw },
                };
            }
        }
        Self {
            use_world: true,
            source: ReplayWorldSource::Default,
        }
    }

    fn apply_env(&self) {
        if self.use_world {
            env::set_var("SUBSTRATE_WORLD", "enabled");
            env::set_var("SUBSTRATE_WORLD_ENABLED", "1");
            env::remove_var("SUBSTRATE_REPLAY_USE_WORLD");
        } else {
            env::set_var("SUBSTRATE_WORLD", "disabled");
            env::set_var("SUBSTRATE_WORLD_ENABLED", "0");
            env::set_var("SUBSTRATE_REPLAY_USE_WORLD", "disabled");
        }
    }
}

fn apply_replay_world_mode_env(env: &mut HashMap<String, String>, mode: &ReplayWorldMode) {
    if mode.use_world {
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

fn inject_world_root_env(env: &mut HashMap<String, String>) {
    let world_root = world_root_from_env();
    let mode = world_root.mode.as_str().to_string();
    let path = world_root.path.to_string_lossy().to_string();
    env.insert("SUBSTRATE_ANCHOR_MODE".to_string(), mode.clone());
    env.insert("SUBSTRATE_WORLD_ROOT_MODE".to_string(), mode);
    env.insert("SUBSTRATE_ANCHOR_PATH".to_string(), path.clone());
    env.insert("SUBSTRATE_WORLD_ROOT_PATH".to_string(), path);
    env.insert(
        "SUBSTRATE_CAGED".to_string(),
        if world_root.caged { "1" } else { "0" }.to_string(),
    );
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
        eprintln!("Make sure tracing is enabled with SUBSTRATE_WORLD=enabled");
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
        eprintln!("Make sure tracing is enabled with SUBSTRATE_WORLD=enabled");
        std::process::exit(1);
    }

    let runtime = tokio::runtime::Runtime::new()?;

    // Reconstruct state from the trace entry (includes PATH/user/env metadata when captured)
    let span = runtime.block_on(async { load_span_from_trace(&trace_file, span_id).await })?;
    let mut state = reconstruct_state(&span, &HashMap::new())?;

    // Verbose header
    if verbose_requested {
        eprintln!("[replay] span_id: {}", span_id);
        eprintln!("[replay] command: {}", state.raw_cmd);
        eprintln!("[replay] cwd: {}", state.cwd.display());
        eprintln!("[replay] mode: bash -lc");
    }

    // Respect replay toggle precedence: --world > --no-world > SUBSTRATE_REPLAY_USE_WORLD
    let replay_world_mode = ReplayWorldMode::from_cli(cli);
    apply_replay_world_mode_env(&mut state.env, &replay_world_mode);
    inject_world_root_env(&mut state.env);
    replay_world_mode.apply_env();

    if verbose_requested {
        eprintln!(
            "{}",
            replay_world_mode
                .source
                .summary(replay_world_mode.use_world)
        );
    }
    if !replay_world_mode.use_world && verbose_requested {
        if let Some(reason) = replay_world_mode.source.warn_reason() {
            eprintln!("[replay] warn: running without world isolation ({reason})");
        } else {
            eprintln!("[replay] warn: running without world isolation");
        }
    }

    let use_world = replay_world_mode.use_world;
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
