use crate::execution::cli::{ConfigAction, ConfigCmd, ConfigInitArgs};
use anyhow::{anyhow, Context, Result};
use std::fs;
use std::io::Write;
use std::path::Path;
use substrate_common::paths as substrate_paths;
use tempfile::NamedTempFile;
use toml::value::{Table as TomlTable, Value as TomlValue};

pub(crate) fn handle_config_command(cmd: &ConfigCmd) -> Result<()> {
    match &cmd.action {
        ConfigAction::Init(args) => run_config_init(args),
    }
}

fn run_config_init(args: &ConfigInitArgs) -> Result<()> {
    let config_path = substrate_paths::config_file()?;
    let parent = config_path
        .parent()
        .ok_or_else(|| anyhow!("config path {} has no parent", config_path.display()))?;
    fs::create_dir_all(parent)
        .with_context(|| format!("failed to create directory {}", parent.display()))?;

    let existed = config_path.exists();
    if existed && !args.force {
        println!(
            "substrate: config already exists at {}; use --force to regenerate",
            config_path.display()
        );
        return Ok(());
    }

    write_default_config(&config_path)?;
    if existed {
        println!(
            "substrate: overwrote config at {} (--force)",
            config_path.display()
        );
    } else {
        println!(
            "substrate: wrote default config to {}",
            config_path.display()
        );
    }
    Ok(())
}

fn write_default_config(path: &Path) -> Result<()> {
    let parent = path
        .parent()
        .ok_or_else(|| anyhow!("config path {} has no parent", path.display()))?;
    fs::create_dir_all(parent).with_context(|| format!("failed to create {}", parent.display()))?;

    let mut tmp = NamedTempFile::new_in(parent)
        .with_context(|| format!("failed to create temp file near {}", path.display()))?;
    let body = toml::to_string_pretty(&TomlValue::Table(default_config_tables()))
        .context("failed to serialize default config")?;
    tmp.write_all(body.as_bytes())
        .with_context(|| format!("failed to write defaults to {}", path.display()))?;
    tmp.flush()?;
    tmp.persist(path).map_err(|err| {
        anyhow!(
            "failed to persist config at {}: {}",
            path.display(),
            err.error
        )
    })?;
    Ok(())
}

fn default_config_tables() -> TomlTable {
    let mut root = TomlTable::new();
    root.insert(
        "anchor_mode".to_string(),
        TomlValue::String("project".to_string()),
    );
    root.insert("anchor_path".to_string(), TomlValue::String(String::new()));
    root.insert(
        "root_mode".to_string(),
        TomlValue::String("project".to_string()),
    );
    root.insert("root_path".to_string(), TomlValue::String(String::new()));
    root.insert("caged".to_string(), TomlValue::Boolean(true));

    let mut install = TomlTable::new();
    install.insert("world_enabled".to_string(), TomlValue::Boolean(true));

    let mut doc = TomlTable::new();
    doc.insert("install".to_string(), TomlValue::Table(install));
    doc.insert("world".to_string(), TomlValue::Table(root));
    doc
}
