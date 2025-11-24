use super::report::{build_manifest_paths, legacy_bashenv_path, manifest_spec_map};
use anyhow::{anyhow, Context, Result};
use chrono::Utc;
use serde_json::json;
use std::{
    fs,
    io::{self, Write},
    path::Path,
};
use substrate_common::{log_schema, manager_manifest::ManagerManifest};
use substrate_trace::{append_to_trace, init_trace, set_global_trace_context, TraceContext};
use tempfile::NamedTempFile;
use tracing::warn;
use uuid::Uuid;

const BLOCK_START_PREFIX: &str = "# >>> substrate repair:";
const BLOCK_END_PREFIX: &str = "# <<< substrate repair:";

#[derive(Debug)]
pub enum RepairOutcome {
    Applied {
        manager: String,
        bashenv_path: std::path::PathBuf,
        backup_path: Option<std::path::PathBuf>,
    },
    Skipped {
        manager: String,
        reason: String,
    },
}

pub(crate) fn run_repair(manager: &str, auto_confirm: bool) -> Result<RepairOutcome> {
    let (manifest_info, _) = build_manifest_paths()?;
    let manifest = ManagerManifest::load(&manifest_info.base, manifest_info.overlay.as_deref())?;
    let spec_map = manifest_spec_map(manifest);
    let Some(spec) = spec_map
        .values()
        .find(|spec| spec.name.eq_ignore_ascii_case(manager))
    else {
        return Err(anyhow!(
            "manager `{}` not found in manifest {}",
            manager,
            manifest_info.base.display()
        ));
    };

    let snippet = spec
        .repair_hint
        .as_ref()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .ok_or_else(|| {
            anyhow!(
                "manager `{}` does not define a repair_hint in {}",
                spec.name,
                manifest_info.base.display()
            )
        })?;

    let bashenv_path = legacy_bashenv_path()?;
    if !prompt_for_repair(auto_confirm, &spec.name, &bashenv_path, &snippet)? {
        return Ok(RepairOutcome::Skipped {
            manager: spec.name.clone(),
            reason: "user declined confirmation".to_string(),
        });
    }

    let existing = fs::read_to_string(&bashenv_path).ok();
    let block = build_manager_block(&spec.name, &snippet);
    let merged = upsert_block(existing.as_deref().unwrap_or(""), &spec.name, &block);

    if let Some(parent) = bashenv_path.parent() {
        fs::create_dir_all(parent).with_context(|| {
            format!("failed to create directory for {}", bashenv_path.display())
        })?;
    }

    let backup_path = if bashenv_path.exists() {
        create_backup(&bashenv_path)?
    } else {
        None
    };

    write_atomic(&bashenv_path, &merged)?;
    log_repair_event(&spec.name, &bashenv_path, backup_path.as_deref(), &block);

    Ok(RepairOutcome::Applied {
        manager: spec.name.clone(),
        bashenv_path,
        backup_path,
    })
}

fn build_manager_block(name: &str, snippet: &str) -> String {
    let mut block = String::new();
    block.push_str(&format!("{BLOCK_START_PREFIX} {name}\n"));
    block.push_str(snippet.trim_end());
    block.push('\n');
    block.push_str(&format!("{BLOCK_END_PREFIX} {name}\n"));
    block
}

fn upsert_block(contents: &str, name: &str, block: &str) -> String {
    let start_marker = format!("{BLOCK_START_PREFIX} {name}");
    let end_marker = format!("{BLOCK_END_PREFIX} {name}");

    if let Some(start_idx) = contents.find(&start_marker) {
        if let Some(end_rel) = contents[start_idx..].find(&end_marker) {
            let mut removal_end = start_idx + end_rel + end_marker.len();
            if contents[removal_end..].starts_with("\r\n") {
                removal_end += 2;
            } else if contents[removal_end..].starts_with('\n') {
                removal_end += 1;
            }
            let mut result = String::new();
            result.push_str(&contents[..start_idx]);
            if !result.ends_with('\n') && !result.is_empty() {
                result.push('\n');
            }
            result.push_str(block);
            if !block.ends_with('\n') {
                result.push('\n');
            }
            result.push_str(&contents[removal_end..]);
            return result;
        }
    }

    let mut result = String::from(contents);
    if !result.is_empty() && !result.ends_with('\n') {
        result.push('\n');
    }
    result.push_str(block);
    if !block.ends_with('\n') {
        result.push('\n');
    }
    result
}

fn create_backup(path: &Path) -> Result<Option<std::path::PathBuf>> {
    if !path.exists() {
        return Ok(None);
    }
    let backup = path.with_extension("bak");
    fs::copy(path, &backup)
        .with_context(|| format!("failed to create backup at {}", backup.display()))?;
    Ok(Some(backup))
}

fn write_atomic(path: &Path, contents: &str) -> Result<()> {
    let parent = path
        .parent()
        .ok_or_else(|| anyhow!("{} has no parent directory", path.display()))?;
    let mut temp = NamedTempFile::new_in(parent)?;
    temp.write_all(contents.as_bytes())?;
    temp.flush()?;
    temp.persist(path)?;
    Ok(())
}

fn prompt_for_repair(
    auto_confirm: bool,
    manager: &str,
    bashenv_path: &Path,
    snippet: &str,
) -> Result<bool> {
    if auto_confirm {
        return Ok(true);
    }
    println!(
        "About to update {} with repair snippet for `{}`:",
        bashenv_path.display(),
        manager
    );
    println!("{}", snippet.trim_end());
    print!("Proceed? [y/N]: ");
    io::stdout().flush().ok();
    let mut answer = String::new();
    io::stdin().read_line(&mut answer)?;
    let normalized = answer.trim().to_ascii_lowercase();
    Ok(matches!(normalized.as_str(), "y" | "yes"))
}

fn log_repair_event(manager: &str, bashenv_path: &Path, backup_path: Option<&Path>, block: &str) {
    let entry = json!({
        log_schema::TIMESTAMP: Utc::now().to_rfc3339(),
        log_schema::EVENT_TYPE: "shim_repair",
        log_schema::COMPONENT: "shell",
        log_schema::SESSION_ID: Uuid::now_v7().to_string(),
        "manager": manager,
        "bashenv_path": bashenv_path.display().to_string(),
        "backup_created": backup_path.is_some(),
        "backup_path": backup_path.map(|p| p.display().to_string()),
        "snippet_length": block.lines().count()
    });
    let _ = set_global_trace_context(TraceContext::default());
    if let Err(err) = init_trace(None).and_then(|_| append_to_trace(&entry)) {
        warn!(
            target = "substrate::shell",
            manager = manager,
            error = %err,
            "failed to log shim_repair event"
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn upsert_block_replaces_existing() {
        let original = "# >>> substrate repair: nvm\nold\n# <<< substrate repair: nvm\n";
        let block = build_manager_block("nvm", "new");
        let updated = upsert_block(original, "nvm", &block);
        assert!(updated.contains("new"));
        assert!(!updated.contains("old"));
    }

    #[test]
    fn upsert_block_appends_when_missing() {
        let original = "PATH=foo\n";
        let block = build_manager_block("nvm", "new");
        let updated = upsert_block(original, "nvm", &block);
        assert!(updated.ends_with(&block));
        assert!(updated.contains("PATH=foo"));
    }
}
