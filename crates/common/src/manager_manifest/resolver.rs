use crate::manager_manifest::schema::{
    expand_path, ManagerManifest, ManagerSpec, RawManagerSpec, RawManifest,
    MANAGER_MANIFEST_VERSION,
};
use crate::manager_manifest::validator::{insert_entries, parse_manager_entries};
use anyhow::{anyhow, bail, Context, Result};
use serde_yaml::Value;
use std::{
    collections::HashMap,
    fs, io,
    path::{Path, PathBuf},
};

impl ManagerManifest {
    /// Load and merge the base manifest plus an optional overlay.
    pub fn load(base: &Path, overlay: Option<&Path>) -> Result<Self> {
        let overlays = overlay
            .map(|path| vec![path.to_path_buf()])
            .unwrap_or_default();
        Self::load_layered(base, &overlays)
    }

    /// Load and merge the base manifest plus zero or more overlay manifests.
    ///
    /// Overlay manifests are applied in the order provided, so later overlays
    /// override earlier ones.
    pub fn load_layered(base: &Path, overlays: &[PathBuf]) -> Result<Self> {
        let base_value = read_yaml_value(expand_path(base)?)
            .with_context(|| format!("failed to load manager manifest from {}", base.display()))?;
        let base_manifest: RawManifest =
            serde_yaml::from_value(base_value).context("manager manifest schema is invalid")?;

        if base_manifest.version != MANAGER_MANIFEST_VERSION {
            bail!(
                "manager manifest version must be {} (got {})",
                MANAGER_MANIFEST_VERSION,
                base_manifest.version
            );
        }

        let mut merged: HashMap<String, RawManagerSpec> = HashMap::new();
        insert_entries(
            &mut merged,
            parse_manager_entries(base_manifest.managers)?,
            "base manifest",
        )?;

        for overlay_path in overlays {
            let overlay_path = expand_path(overlay_path)?;
            let Some(value) = read_yaml_value_optional(overlay_path.clone())? else {
                continue;
            };
            let overlay_manifest: RawManifest =
                serde_yaml::from_value(value).with_context(|| {
                    format!(
                        "overlay manager manifest schema is invalid: {}",
                        overlay_path.display()
                    )
                })?;

            if overlay_manifest.version != base_manifest.version {
                bail!(
                    "overlay manifest version {} does not match base {}",
                    overlay_manifest.version,
                    base_manifest.version
                );
            }

            insert_entries(
                &mut merged,
                parse_manager_entries(overlay_manifest.managers)?,
                "overlay manifest",
            )?;
        }

        let mut managers = Vec::with_capacity(merged.len());
        for (name, spec) in merged {
            managers.push(ManagerSpec::from_raw(name, spec, base_manifest.version)?);
        }

        managers.sort_by(|a, b| {
            a.priority
                .cmp(&b.priority)
                .then_with(|| a.name.cmp(&b.name))
        });

        Ok(Self {
            version: base_manifest.version,
            managers,
        })
    }
}

fn read_yaml_value(path: PathBuf) -> Result<Value> {
    let data = fs::read_to_string(&path)
        .with_context(|| format!("failed to read manifest at {}", path.display()))?;
    serde_yaml::from_str(&data)
        .with_context(|| format!("failed to parse manifest at {}", path.display()))
}

fn read_yaml_value_optional(path: PathBuf) -> Result<Option<Value>> {
    match fs::read_to_string(&path) {
        Ok(contents) => {
            let value = serde_yaml::from_str(&contents)
                .with_context(|| format!("failed to parse overlay at {}", path.display()))?;
            Ok(Some(value))
        }
        Err(err) if err.kind() == io::ErrorKind::NotFound => Ok(None),
        Err(err) => Err(anyhow!(err))
            .with_context(|| format!("failed to read overlay at {}", path.display())),
    }
}
