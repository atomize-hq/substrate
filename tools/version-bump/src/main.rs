use std::fs;
use std::path::Path;

use anyhow::{anyhow, Context, Result};
use camino::Utf8PathBuf;
use cargo_metadata::{MetadataCommand, Package};
use clap::Parser;
use toml_edit::{DocumentMut, Item, Value};

#[derive(Parser, Debug)]
#[command(
    author,
    version,
    about = "Bump versions for workspace crates and their path dependencies"
)]
struct Args {
    /// Next version to set (e.g. 0.3.0)
    #[arg(long, required = true)]
    next_version: String,

    /// Current version to replace; inferred from the `substrate` crate when omitted.
    #[arg(long)]
    current_version: Option<String>,

    /// Preview changes without writing files
    #[arg(long, default_value_t = false)]
    dry_run: bool,
}

fn main() -> Result<()> {
    let args = Args::parse();

    let metadata = MetadataCommand::new().no_deps().exec()?;
    let workspace_root: Utf8PathBuf = metadata.workspace_root.clone();

    let current_version = match args.current_version {
        Some(v) => v,
        None => infer_current_version(&metadata.packages)?,
    };

    let mut updated_manifests = Vec::new();

    for package in &metadata.packages {
        if package.version.to_string() != current_version {
            continue;
        }

        let manifest_path = package.manifest_path.clone().into_std_path_buf();

        let changed = process_manifest(
            &manifest_path,
            &current_version,
            &args.next_version,
            args.dry_run,
        )?;

        if changed {
            let relative = manifest_path
                .strip_prefix(workspace_root.as_std_path())
                .unwrap_or(&manifest_path);
            updated_manifests.push(relative.display().to_string());
        }
    }

    if updated_manifests.is_empty() {
        println!(
            "No manifests matched current version {}; nothing to update.",
            current_version
        );
    } else {
        println!("Updated {} manifest(s):", updated_manifests.len());
        for path in updated_manifests {
            println!("  - {}", path);
        }
    }

    Ok(())
}

fn infer_current_version(packages: &[Package]) -> Result<String> {
    let substrate = packages
        .iter()
        .find(|pkg| pkg.name == "substrate")
        .ok_or_else(|| anyhow!("could not locate `substrate` package in workspace"))?;
    Ok(substrate.version.to_string())
}

fn process_manifest(
    manifest_path: &Path,
    current_version: &str,
    next_version: &str,
    dry_run: bool,
) -> Result<bool> {
    let content = fs::read_to_string(manifest_path)
        .with_context(|| format!("failed to read {}", manifest_path.display()))?;
    let mut document = content
        .parse::<DocumentMut>()
        .with_context(|| format!("failed to parse {}", manifest_path.display()))?;

    let mut changed = false;

    // package.version
    if let Some(pkg) = document.get_mut("package") {
        if let Some(version_item) = pkg.get_mut("version") {
            if version_item.as_str() == Some(current_version) {
                *version_item = Item::Value(Value::from(next_version));
                changed = true;
            }
        }
    }

    // dependencies tables
    if update_tables(document.as_table_mut(), current_version, next_version) {
        changed = true;
    }

    if changed && !dry_run {
        fs::write(manifest_path, document.to_string())
            .with_context(|| format!("failed to write {}", manifest_path.display()))?;
    }

    Ok(changed)
}

fn update_tables(table: &mut toml_edit::Table, current: &str, next: &str) -> bool {
    let mut modified = false;

    for key in ["dependencies", "dev-dependencies", "build-dependencies"] {
        if let Some(Item::Table(dep_table)) = table.get_mut(key) {
            if update_dependency_table(dep_table, current, next) {
                modified = true;
            }
        }
    }

    // Recurse into nested tables (e.g. target.'cfg(...)'.dependencies)
    let keys: Vec<String> = table.iter().map(|(k, _)| k.to_string()).collect();
    for key in keys {
        if let Some(Item::Table(sub_table)) = table.get_mut(&key) {
            if update_tables(sub_table, current, next) {
                modified = true;
            }
        }
    }

    modified
}

fn update_dependency_table(table: &mut toml_edit::Table, current: &str, next: &str) -> bool {
    let mut modified = false;

    let dep_keys: Vec<String> = table.iter().map(|(k, _)| k.to_string()).collect();

    for key in dep_keys {
        if let Some(item) = table.get_mut(&key) {
            match item {
                Item::Value(value) => {
                    let mut updated = false;

                    if let Some(inline) = value.as_inline_table_mut() {
                        updated = maybe_update_inline_version(inline, current, next);
                    } else if let Some(str_value) = value.as_str() {
                        if str_value == current {
                            *value = Value::from(next);
                            updated = true;
                        }
                    }

                    if updated {
                        modified = true;
                    }
                }
                Item::Table(dep_table) => {
                    if let Some(item_version) = dep_table.get_mut("version") {
                        if item_version.as_str() == Some(current) {
                            *item_version = Item::Value(Value::from(next));
                            modified = true;
                        }
                    }
                }
                _ => {}
            }
        }
    }

    modified
}

fn maybe_update_inline_version(
    inline: &mut toml_edit::InlineTable,
    current: &str,
    next: &str,
) -> bool {
    if let Some(version_item) = inline.get_mut("version") {
        if version_item.as_str() == Some(current) {
            inline.insert("version", Value::from(next));
            return true;
        }
    }
    false
}
