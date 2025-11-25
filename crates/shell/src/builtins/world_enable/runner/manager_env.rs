//! Manager environment export updates for world enable.

use anyhow::{Context, Result};
use std::fs;
use std::path::Path;

pub(super) fn update_manager_env_exports(path: &Path, enabled: bool) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).with_context(|| {
            format!(
                "failed to create directory for manager env at {}",
                parent.display()
            )
        })?;
    }

    let existing = fs::read_to_string(path).unwrap_or_else(|_| String::new());
    let mut lines: Vec<String> = existing.lines().map(|line| line.to_string()).collect();
    let mut shebang = None;
    if let Some(first) = lines.first() {
        if first.starts_with("#!") {
            shebang = Some(lines.remove(0));
        }
    }
    lines.retain(|line| {
        let trimmed = line.trim_start();
        !trimmed.starts_with("export SUBSTRATE_WORLD=")
            && !trimmed.starts_with("export SUBSTRATE_WORLD_ENABLED=")
    });

    let mut output = Vec::new();
    if let Some(sb) = shebang {
        output.push(sb);
    }
    output.push("# Managed by `substrate world enable`".to_string());
    output.push(format!(
        "export SUBSTRATE_WORLD={}",
        if enabled { "enabled" } else { "disabled" }
    ));
    output.push(format!(
        "export SUBSTRATE_WORLD_ENABLED={}",
        if enabled { "1" } else { "0" }
    ));
    if !lines.is_empty() {
        output.push(String::new());
        output.extend(lines);
    }

    let mut body = output.join("\n");
    body.push('\n');
    fs::write(path, body)
        .with_context(|| format!("failed to update manager env at {}", path.display()))?;
    Ok(())
}
