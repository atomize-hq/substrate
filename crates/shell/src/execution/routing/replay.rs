//! Trace and replay helpers for routing.

use anyhow::Result;
use std::collections::HashMap;
use std::env;
use std::path::PathBuf;
use std::process::Stdio;
use uuid::Uuid;

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
pub(crate) fn handle_replay_command(span_id: &str) -> Result<()> {
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

    // Load the trace for the span
    let file = File::open(&trace_file)?;
    let reader = BufReader::new(file);
    let mut trace_entry = None;

    for line in reader.lines() {
        let line = line?;
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(&line) {
            if let Some(id) = json.get("span_id").and_then(|v| v.as_str()) {
                if id == span_id {
                    trace_entry = Some(json);
                    break;
                }
            }
        }
    }

    let entry = trace_entry.ok_or_else(|| anyhow::anyhow!("Span ID not found: {}", span_id))?;

    // Extract command information from the trace
    let command = entry
        .get("cmd")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow::anyhow!("No command found in trace"))?;

    let cwd = entry
        .get("cwd")
        .and_then(|v| v.as_str())
        .map(PathBuf::from)
        .unwrap_or_else(|| env::current_dir().unwrap());

    let session_id = entry
        .get("session_id")
        .and_then(|v| v.as_str())
        .map(String::from)
        .unwrap_or_else(|| Uuid::now_v7().to_string());

    // Verbose header
    if env::var("SUBSTRATE_REPLAY_VERBOSE").unwrap_or_default() == "1"
        || env::args().any(|a| a == "--replay-verbose")
    {
        eprintln!("[replay] span_id: {}", span_id);
        eprintln!("[replay] command: {}", command);
        eprintln!("[replay] cwd: {}", cwd.display());
        eprintln!("[replay] mode: bash -lc");
    }

    // Parse command into command and args
    let parts: Vec<&str> = command.split_whitespace().collect();
    let (cmd, args) = if !parts.is_empty() {
        (
            parts[0].to_string(),
            parts[1..].iter().map(|s| s.to_string()).collect(),
        )
    } else {
        (command.to_string(), Vec::new())
    };

    // Create execution state
    let state = substrate_replay::replay::ExecutionState {
        raw_cmd: command.to_string(),
        command: cmd,
        args,
        cwd,
        env: HashMap::new(), // Could extract from trace if captured
        stdin: None,
        session_id,
        span_id: span_id.to_string(),
    };

    // Execute with replay (choose world isolation; default enabled, allow opt-out)
    let runtime = tokio::runtime::Runtime::new()?;
    // Respect --no-world flag, then environment variable override, else default enabled
    let no_world_flag = env::args().any(|a| a == "--no-world");
    let use_world = if no_world_flag {
        false
    } else {
        match env::var("SUBSTRATE_REPLAY_USE_WORLD") {
            Ok(val) => val != "0" && val != "disabled",
            Err(_) => true,
        }
    };
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
