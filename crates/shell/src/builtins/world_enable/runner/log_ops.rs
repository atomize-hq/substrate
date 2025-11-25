//! Logging utilities for world enable flows.

use crate::WorldEnableArgs;
use anyhow::{Context, Result};
use chrono::Utc;
use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::path::Path;

pub(super) fn initialize_log_file(path: &Path) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create {}", parent.display()))?;
    }
    let mut file = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(path)?;
    writeln!(
        file,
        "# Substrate world enable log\ntimestamp: {}",
        Utc::now().to_rfc3339()
    )?;
    Ok(())
}

pub(super) fn print_dry_run_plan(
    script: &Path,
    args: &WorldEnableArgs,
    prefix: &Path,
    log_path: &Path,
) -> Result<()> {
    let mut command_line = vec![
        script.display().to_string(),
        "--prefix".to_string(),
        prefix.display().to_string(),
        "--profile".to_string(),
        args.profile.clone(),
    ];
    if args.verbose {
        command_line.push("--verbose".into());
    }
    if args.force {
        command_line.push("--force".into());
    }
    command_line.push("--dry-run".into());
    println!("Dry run: {}", command_line.join(" "));
    if let Some(parent) = log_path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create {}", parent.display()))?;
    }
    println!("Helper log would be written to {}", log_path.display());
    Ok(())
}

pub(super) fn append_log_line(log_path: &Path, message: &str) -> Result<()> {
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(log_path)
        .with_context(|| format!("failed to open {}", log_path.display()))?;
    writeln!(file, "{}", message)?;
    Ok(())
}

pub(super) fn write_world_doctor_output(
    log_path: &Path,
    label: &str,
    data: &[u8],
    verbose: bool,
) -> Result<()> {
    if data.is_empty() {
        return Ok(());
    }
    let mut file = OpenOptions::new()
        .append(true)
        .open(log_path)
        .with_context(|| format!("failed to open {}", log_path.display()))?;
    writeln!(file, "--- world doctor {} ---", label)?;
    file.write_all(data)?;
    if !data.ends_with(b"\n") {
        writeln!(file)?;
    }
    if verbose {
        io::stdout().write_all(data)?;
        if !data.ends_with(b"\n") {
            println!();
        }
    }
    Ok(())
}
