use crate::execution::cli::{ConfigAction, ConfigCmd, ConfigInitArgs, ConfigShowArgs};
use anyhow::{anyhow, bail, Context, Result};
use std::fs;
use std::io::{self, Write};
use std::path::Path;
use substrate_common::paths as substrate_paths;
use tempfile::NamedTempFile;
use toml::value::{Table as TomlTable, Value as TomlValue};

pub(crate) fn handle_config_command(cmd: &ConfigCmd) -> Result<()> {
    match &cmd.action {
        ConfigAction::Init(args) => run_config_init(args),
        ConfigAction::Show(args) => run_config_show(args),
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

fn run_config_show(args: &ConfigShowArgs) -> Result<()> {
    let config_path = substrate_paths::config_file()?;
    let contents = read_config_contents(&config_path)?;
    let mut value: TomlValue = toml::from_str(&contents)
        .with_context(|| format!("invalid TOML in {}", config_path.display()))?;

    redact_config_value("", &mut value);

    if args.json {
        let json =
            serde_json::to_string_pretty(&value).context("failed to serialize config as JSON")?;
        println!("{json}");
    } else {
        let formatted =
            toml::to_string_pretty(&value).context("failed to serialize config as TOML")?;
        println!("{formatted}");
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

fn read_config_contents(path: &Path) -> Result<String> {
    match fs::read_to_string(path) {
        Ok(contents) => Ok(contents),
        Err(err) if err.kind() == io::ErrorKind::NotFound => bail!(
            "substrate: config file missing at {}; run `substrate config init` to create defaults",
            path.display()
        ),
        Err(err) => Err(anyhow!("failed to read {}: {err}", path.display())),
    }
}

fn redact_config_value(current_path: &str, value: &mut TomlValue) {
    match value {
        TomlValue::Table(table) => {
            for (key, entry) in table.iter_mut() {
                let next_path = if current_path.is_empty() {
                    key.clone()
                } else {
                    format!("{current_path}.{key}")
                };

                if is_sensitive_path(&next_path) {
                    *entry = TomlValue::String(REDACTED_PLACEHOLDER.to_string());
                } else {
                    redact_config_value(&next_path, entry);
                }
            }
        }
        TomlValue::Array(items) => {
            for (index, item) in items.iter_mut().enumerate() {
                let next_path = if current_path.is_empty() {
                    format!("[{index}]")
                } else {
                    format!("{current_path}[{index}]")
                };
                redact_config_value(&next_path, item);
            }
        }
        _ => {}
    }
}

fn is_sensitive_path(path: &str) -> bool {
    const SENSITIVE_PATHS: &[&str] = &[
        "install.api_token",
        "install.auth_token",
        "install.access_token",
        "world.api_token",
    ];
    SENSITIVE_PATHS
        .iter()
        .any(|candidate| candidate.eq_ignore_ascii_case(path))
}

const REDACTED_PLACEHOLDER: &str = "*** redacted ***";
