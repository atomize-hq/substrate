use crate::execution::cli::{
    ConfigAction, ConfigCmd, ConfigInitArgs, ConfigSetArgs, ConfigShowArgs,
};
use crate::execution::settings::parse_bool_flag;
use anyhow::{anyhow, bail, Context, Result};
use serde::Serialize;
use serde_json::Value as JsonValue;
use serde_yaml::{Mapping as YamlMapping, Value as YamlValue};
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use substrate_common::{paths as substrate_paths, WorldRootMode};
use tempfile::NamedTempFile;

pub(crate) fn handle_config_command(cmd: &ConfigCmd) -> Result<()> {
    match &cmd.action {
        ConfigAction::Init(args) => run_config_init(args),
        ConfigAction::Show(args) => run_config_show(args),
        ConfigAction::Set(args) => run_config_set(args),
    }
}

fn run_config_init(args: &ConfigInitArgs) -> Result<()> {
    let config_path = substrate_paths::config_file()?;
    ensure_no_legacy_toml_config(&config_path)?;

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
    let mut value: YamlValue = serde_yaml::from_str(&contents)
        .with_context(|| format!("invalid YAML in {}", config_path.display()))?;

    redact_config_value("", &mut value);

    if args.json {
        let payload = serde_json::to_string_pretty(&yaml_to_json_value(&value))
            .context("failed to serialize config as JSON")?;
        println!("{payload}");
    } else {
        let formatted =
            serde_yaml::to_string(&value).context("failed to serialize config as YAML")?;
        println!("{formatted}");
    }

    Ok(())
}

fn run_config_set(args: &ConfigSetArgs) -> Result<()> {
    let config_path = substrate_paths::config_file()?;
    let contents = read_config_contents(&config_path)?;
    let mut root: YamlValue = serde_yaml::from_str(&contents)
        .with_context(|| format!("invalid YAML in {}", config_path.display()))?;
    let document = root
        .as_mapping_mut()
        .ok_or_else(|| anyhow!("config in {} must be a mapping", config_path.display()))?;

    let updates = parse_config_updates(&args.updates)?;
    let mut all_changes = Vec::new();
    for update in updates {
        all_changes.extend(apply_config_update(document, &update)?);
    }

    let applied_changes: Vec<ConfigChange> = all_changes
        .into_iter()
        .filter(|change| change.changed())
        .collect();

    if applied_changes.is_empty() {
        if args.json {
            let summary = ConfigSetSummary::from_changes(&config_path, &[]);
            let payload = serde_json::to_string_pretty(&summary)
                .context("failed to serialize config set summary")?;
            println!("{payload}");
        } else {
            println!(
                "substrate: config already up to date at {}",
                config_path.display()
            );
        }
        return Ok(());
    }

    write_config_value(&config_path, &root)?;

    if args.json {
        let summary = ConfigSetSummary::from_changes(&config_path, &applied_changes);
        let payload = serde_json::to_string_pretty(&summary)
            .context("failed to serialize config set summary")?;
        println!("{payload}");
    } else {
        println!("substrate: updated config at {}", config_path.display());
        for change in &applied_changes {
            let alias_note = if change.is_alias { " (alias)" } else { "" };
            println!(
                "  - {}{}: {} -> {}",
                change.key,
                alias_note,
                format_optional_value(change.old_value.as_ref()),
                format_value(&change.new_value)
            );
        }
    }

    Ok(())
}

fn write_default_config(path: &Path) -> Result<()> {
    let defaults = default_config_value();
    write_config_value(path, &defaults).context("failed to write default config")
}

fn write_config_value(path: &Path, value: &YamlValue) -> Result<()> {
    ensure_no_legacy_toml_config(path)?;
    let parent = path
        .parent()
        .ok_or_else(|| anyhow!("config path {} has no parent", path.display()))?;
    fs::create_dir_all(parent).with_context(|| format!("failed to create {}", parent.display()))?;

    let mut tmp = NamedTempFile::new_in(parent)
        .with_context(|| format!("failed to create temp file near {}", path.display()))?;
    let body = serde_yaml::to_string(value)
        .with_context(|| format!("failed to serialize config at {}", path.display()))?;
    tmp.write_all(body.as_bytes())
        .with_context(|| format!("failed to write {}", path.display()))?;
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

fn default_config_value() -> YamlValue {
    let mut world = YamlMapping::new();
    world.insert(
        YamlValue::String("anchor_mode".to_string()),
        YamlValue::String("project".to_string()),
    );
    world.insert(
        YamlValue::String("anchor_path".to_string()),
        YamlValue::String(String::new()),
    );
    world.insert(
        YamlValue::String("root_mode".to_string()),
        YamlValue::String("project".to_string()),
    );
    world.insert(
        YamlValue::String("root_path".to_string()),
        YamlValue::String(String::new()),
    );
    world.insert(
        YamlValue::String("caged".to_string()),
        YamlValue::Bool(true),
    );

    let mut install = YamlMapping::new();
    install.insert(
        YamlValue::String("world_enabled".to_string()),
        YamlValue::Bool(true),
    );

    let mut doc = YamlMapping::new();
    doc.insert(
        YamlValue::String("install".to_string()),
        YamlValue::Mapping(install),
    );
    doc.insert(
        YamlValue::String("world".to_string()),
        YamlValue::Mapping(world),
    );
    YamlValue::Mapping(doc)
}

fn parse_config_updates(inputs: &[String]) -> Result<Vec<ConfigUpdate>> {
    let mut updates = Vec::with_capacity(inputs.len());
    for raw in inputs {
        let (raw_key, raw_value) = raw.split_once('=').ok_or_else(|| {
            anyhow!(
                "invalid assignment '{}'; expected key=value (e.g., world.anchor_mode=follow-cwd)",
                raw
            )
        })?;
        let key = raw_key.trim();
        if key.is_empty() {
            bail!("invalid assignment '{}'; missing key before '='", raw);
        }

        let spec = lookup_field_spec(key).ok_or_else(|| {
            anyhow!(
                "unsupported config key '{}'; supported keys: {}",
                key,
                SUPPORTED_CONFIG_KEYS.join(", ")
            )
        })?;

        let value = raw_value.trim();
        let parsed_value = parse_config_value(spec.kind, value, key)?;
        updates.push(ConfigUpdate {
            spec,
            requested_key: key.to_string(),
            value: parsed_value,
        });
    }
    Ok(updates)
}

fn parse_config_value(kind: ConfigValueKind, raw: &str, key: &str) -> Result<ConfigValue> {
    match kind {
        ConfigValueKind::Boolean => parse_bool_flag(raw)
            .map(ConfigValue::Boolean)
            .ok_or_else(|| anyhow!("{} must be true/false/1/0/yes/no (found {})", key, raw)),
        ConfigValueKind::Mode => {
            if raw.is_empty() {
                bail!("{} requires a value (project, follow-cwd, or custom)", key);
            }
            WorldRootMode::parse(raw)
                .map(ConfigValue::Mode)
                .ok_or_else(|| {
                    anyhow!(
                        "{} must be one of project, follow-cwd, or custom (found {})",
                        key,
                        raw
                    )
                })
        }
        ConfigValueKind::String => Ok(ConfigValue::String(raw.to_string())),
    }
}

fn apply_config_update(
    document: &mut YamlMapping,
    update: &ConfigUpdate,
) -> Result<Vec<ConfigChange>> {
    let mut paths: Vec<&'static str> = update.spec.paths.to_vec();
    if let Some(index) = paths
        .iter()
        .position(|candidate| *candidate == update.requested_key)
    {
        paths.swap(0, index);
    }

    let mut changes = Vec::with_capacity(paths.len());
    for path in paths {
        let new_value = update.value.to_yaml();
        let previous = set_yaml_path(document, path, new_value.clone())?;
        changes.push(ConfigChange {
            key: path.to_string(),
            is_alias: path != update.requested_key,
            old_value: previous,
            new_value,
        });
    }

    Ok(changes)
}

fn set_yaml_path(
    document: &mut YamlMapping,
    dotted: &str,
    value: YamlValue,
) -> Result<Option<YamlValue>> {
    let segments: Vec<&str> = dotted.split('.').collect();
    if segments.is_empty() {
        bail!("invalid config key '{}'", dotted);
    }
    if segments.iter().any(|segment| segment.is_empty()) {
        bail!("invalid config key '{}': empty path segment", dotted);
    }
    set_yaml_path_segments(document, &segments, dotted, "", value)
}

fn set_yaml_path_segments(
    document: &mut YamlMapping,
    segments: &[&str],
    dotted: &str,
    prefix: &str,
    value: YamlValue,
) -> Result<Option<YamlValue>> {
    let Some((segment, rest)) = segments.split_first() else {
        bail!("invalid config key '{}'", dotted);
    };

    let key = YamlValue::String((*segment).to_string());
    if rest.is_empty() {
        return Ok(document.insert(key, value));
    }

    let next_prefix = if prefix.is_empty() {
        (*segment).to_string()
    } else {
        format!("{prefix}.{segment}")
    };

    if !document.contains_key(&key) {
        document.insert(key.clone(), YamlValue::Mapping(YamlMapping::new()));
    }
    let entry = document
        .get_mut(&key)
        .ok_or_else(|| anyhow!("failed to access {} in {}", next_prefix, dotted))?;
    match entry {
        YamlValue::Mapping(map) => set_yaml_path_segments(map, rest, dotted, &next_prefix, value),
        other => bail!(
            "{} must be a mapping to set {} (found {})",
            next_prefix,
            dotted,
            yaml_type_name(other)
        ),
    }
}

fn lookup_field_spec(key: &str) -> Option<&'static ConfigFieldSpec> {
    match key {
        "install.world_enabled" => Some(&INSTALL_WORLD_ENABLED_SPEC),
        "world.anchor_mode" | "world.root_mode" => Some(&WORLD_ANCHOR_MODE_SPEC),
        "world.anchor_path" | "world.root_path" => Some(&WORLD_ANCHOR_PATH_SPEC),
        "world.caged" => Some(&WORLD_CAGED_SPEC),
        _ => None,
    }
}

fn format_optional_value(value: Option<&YamlValue>) -> String {
    value
        .map(format_value)
        .unwrap_or_else(|| "(unset)".to_string())
}

fn format_value(value: &YamlValue) -> String {
    match value {
        YamlValue::Null => "null".to_string(),
        YamlValue::Bool(flag) => flag.to_string(),
        YamlValue::Number(num) => num.to_string(),
        YamlValue::String(s) => format!("{:?}", s),
        YamlValue::Sequence(_) | YamlValue::Mapping(_) => format_yaml_compact(value),
        YamlValue::Tagged(tagged) => format_value(&tagged.value),
    }
}

fn yaml_to_json_value(value: &YamlValue) -> JsonValue {
    serde_json::to_value(value).unwrap_or_else(|_| JsonValue::String(format_value(value)))
}

struct ConfigUpdate {
    spec: &'static ConfigFieldSpec,
    requested_key: String,
    value: ConfigValue,
}

#[derive(Clone, Copy)]
struct ConfigFieldSpec {
    kind: ConfigValueKind,
    paths: &'static [&'static str],
}

#[derive(Clone, Copy)]
enum ConfigValueKind {
    Boolean,
    Mode,
    String,
}

enum ConfigValue {
    Boolean(bool),
    Mode(WorldRootMode),
    String(String),
}

impl ConfigValue {
    fn to_yaml(&self) -> YamlValue {
        match self {
            ConfigValue::Boolean(flag) => YamlValue::Bool(*flag),
            ConfigValue::Mode(mode) => YamlValue::String(mode.to_string()),
            ConfigValue::String(value) => YamlValue::String(value.clone()),
        }
    }
}

struct ConfigChange {
    key: String,
    is_alias: bool,
    old_value: Option<YamlValue>,
    new_value: YamlValue,
}

impl ConfigChange {
    fn changed(&self) -> bool {
        match &self.old_value {
            Some(old) => old != &self.new_value,
            None => true,
        }
    }
}

#[derive(Serialize)]
struct ConfigSetSummary {
    config_path: String,
    changed: bool,
    changes: Vec<ConfigChangeSummary>,
}

impl ConfigSetSummary {
    fn from_changes(path: &Path, changes: &[ConfigChange]) -> Self {
        let converted = changes
            .iter()
            .map(|change| ConfigChangeSummary {
                key: change.key.clone(),
                alias: change.is_alias,
                old_value: change.old_value.as_ref().map(yaml_to_json_value),
                new_value: yaml_to_json_value(&change.new_value),
            })
            .collect();
        Self {
            config_path: path.display().to_string(),
            changed: !changes.is_empty(),
            changes: converted,
        }
    }
}

#[derive(Serialize)]
struct ConfigChangeSummary {
    key: String,
    alias: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    old_value: Option<JsonValue>,
    new_value: JsonValue,
}

const INSTALL_WORLD_ENABLED_SPEC: ConfigFieldSpec = ConfigFieldSpec {
    kind: ConfigValueKind::Boolean,
    paths: &["install.world_enabled"],
};

const WORLD_ANCHOR_MODE_SPEC: ConfigFieldSpec = ConfigFieldSpec {
    kind: ConfigValueKind::Mode,
    paths: &["world.anchor_mode", "world.root_mode"],
};

const WORLD_ANCHOR_PATH_SPEC: ConfigFieldSpec = ConfigFieldSpec {
    kind: ConfigValueKind::String,
    paths: &["world.anchor_path", "world.root_path"],
};

const WORLD_CAGED_SPEC: ConfigFieldSpec = ConfigFieldSpec {
    kind: ConfigValueKind::Boolean,
    paths: &["world.caged"],
};

const SUPPORTED_CONFIG_KEYS: &[&str] = &[
    "install.world_enabled",
    "world.anchor_mode",
    "world.root_mode",
    "world.anchor_path",
    "world.root_path",
    "world.caged",
];

fn read_config_contents(path: &Path) -> Result<String> {
    ensure_no_legacy_toml_config(path)?;
    match fs::read_to_string(path) {
        Ok(contents) => Ok(contents),
        Err(err) if err.kind() == io::ErrorKind::NotFound => bail!(
            "substrate: config file missing at {}; run `substrate config init` to create defaults",
            path.display()
        ),
        Err(err) => Err(anyhow!("failed to read {}: {err}", path.display())),
    }
}

fn redact_config_value(current_path: &str, value: &mut YamlValue) {
    match value {
        YamlValue::Mapping(map) => {
            for (key, entry) in map.iter_mut() {
                let Some(key_str) = key.as_str() else {
                    redact_config_value(current_path, entry);
                    continue;
                };
                let next_path = if current_path.is_empty() {
                    key_str.to_string()
                } else {
                    format!("{current_path}.{key_str}")
                };

                if is_sensitive_path(&next_path) {
                    *entry = YamlValue::String(REDACTED_PLACEHOLDER.to_string());
                } else {
                    redact_config_value(&next_path, entry);
                }
            }
        }
        YamlValue::Sequence(items) => {
            for (index, item) in items.iter_mut().enumerate() {
                let next_path = if current_path.is_empty() {
                    format!("[{index}]")
                } else {
                    format!("{current_path}[{index}]")
                };
                redact_config_value(&next_path, item);
            }
        }
        YamlValue::Tagged(tagged) => redact_config_value(current_path, &mut tagged.value),
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

fn ensure_no_legacy_toml_config(config_yaml_path: &Path) -> Result<()> {
    let legacy = config_yaml_path
        .parent()
        .map(|parent| parent.join("config.toml"))
        .unwrap_or_else(|| PathBuf::from("config.toml"));

    if !legacy.exists() {
        return Ok(());
    }

    let message = format!(
        "substrate: unsupported legacy TOML config detected:\n  - {}\nYAML config is now required:\n  - {}\nNext steps:\n  - Delete the TOML file and run `substrate config init --force`\n  - Re-apply changes via `substrate config set ...`\n",
        legacy.display(),
        config_yaml_path.display()
    );
    bail!("{message}");
}

fn yaml_type_name(value: &YamlValue) -> &'static str {
    match value {
        YamlValue::Null => "null",
        YamlValue::Bool(_) => "boolean",
        YamlValue::Number(_) => "number",
        YamlValue::String(_) => "string",
        YamlValue::Sequence(_) => "sequence",
        YamlValue::Mapping(_) => "mapping",
        YamlValue::Tagged(_) => "tagged",
    }
}

fn format_yaml_compact(value: &YamlValue) -> String {
    let rendered = serde_yaml::to_string(value)
        .unwrap_or_else(|_| "<unprintable yaml>".to_string())
        .trim()
        .to_string();
    rendered
        .strip_prefix("---\n")
        .unwrap_or(&rendered)
        .trim()
        .to_string()
}
