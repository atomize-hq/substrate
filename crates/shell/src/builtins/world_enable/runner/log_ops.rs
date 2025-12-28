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
    substrate_home: &Path,
    log_path: &Path,
) -> Result<()> {
    let mut command_line = vec![
        script.display().to_string(),
        "--home".to_string(),
        substrate_home.display().to_string(),
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::WorldEnableArgs;
    use tempfile::tempdir;

    fn sample_args() -> WorldEnableArgs {
        WorldEnableArgs {
            prefix: None,
            profile: "release".to_string(),
            dry_run: true,
            verbose: true,
            force: false,
            timeout: 5,
        }
    }

    #[test]
    fn initialize_log_file_writes_header_and_timestamp() {
        let temp = tempdir().unwrap();
        let log_path = temp.path().join("logs/world-enable.log");
        initialize_log_file(&log_path).unwrap();

        let body = std::fs::read_to_string(&log_path).unwrap();
        assert!(body.contains("Substrate world enable log"));
        assert!(body.contains("timestamp:"));
    }

    #[test]
    fn append_log_line_extends_existing_log() {
        let temp = tempdir().unwrap();
        let log_path = temp.path().join("logs/world-enable.log");
        initialize_log_file(&log_path).unwrap();

        append_log_line(&log_path, "first entry").unwrap();
        append_log_line(&log_path, "second entry").unwrap();

        let lines: Vec<_> = std::fs::read_to_string(&log_path)
            .unwrap()
            .lines()
            .map(|line| line.to_string())
            .collect();
        assert!(lines.iter().any(|line| line.contains("first entry")));
        assert!(lines.iter().any(|line| line.contains("second entry")));
        assert!(lines.last().map(|line| line.contains("second entry")) == Some(true));
    }

    #[test]
    fn write_world_doctor_output_appends_labeled_section() {
        let temp = tempdir().unwrap();
        let log_path = temp.path().join("logs/world-enable.log");
        initialize_log_file(&log_path).unwrap();

        write_world_doctor_output(&log_path, "diagnostic", b"hello", false).unwrap();

        let body = std::fs::read_to_string(&log_path).unwrap();
        assert!(body.contains("--- world doctor diagnostic ---"));
        assert!(body.contains("hello"));
    }

    #[test]
    fn print_dry_run_plan_creates_log_directory() {
        let temp = tempdir().unwrap();
        let script = temp.path().join("scripts/world-enable.sh");
        std::fs::create_dir_all(script.parent().unwrap()).unwrap();
        std::fs::write(&script, "#!/bin/sh\n").unwrap();
        let log_path = temp.path().join("logs/subdir/log.txt");

        let args = sample_args();
        print_dry_run_plan(&script, &args, temp.path(), &log_path).unwrap();

        assert!(log_path.parent().unwrap().exists());
    }
}
