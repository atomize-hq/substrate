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

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn update_manager_env_exports_preserves_shebang_and_existing_lines() {
        let temp = tempdir().unwrap();
        let manager_env = temp.path().join("env/manager_env.sh");
        fs::create_dir_all(manager_env.parent().unwrap()).unwrap();
        fs::write(&manager_env, "#!/usr/bin/env bash\nexport OTHER_VAR=1\n").unwrap();

        update_manager_env_exports(&manager_env, true).unwrap();
        let contents = fs::read_to_string(&manager_env).unwrap();
        assert!(contents.starts_with(
            "#!/usr/bin/env bash\n# Managed by `substrate world enable`\nexport SUBSTRATE_WORLD=enabled\nexport SUBSTRATE_WORLD_ENABLED=1\n"
        ));
        assert!(contents.contains("export OTHER_VAR=1"));

        update_manager_env_exports(&manager_env, false).unwrap();
        let updated = fs::read_to_string(&manager_env).unwrap();
        assert!(updated.contains("export SUBSTRATE_WORLD=disabled"));
        assert!(updated.contains("export SUBSTRATE_WORLD_ENABLED=0"));
        assert!(updated.contains("export OTHER_VAR=1"));
    }
}
