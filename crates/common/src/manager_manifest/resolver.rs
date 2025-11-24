use crate::manager_manifest::schema::{
    expand_path, ManagerManifest, ManagerSpec, RawManagerSpec, RawManifest,
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
        let base_value = read_yaml_value(expand_path(base)?)
            .with_context(|| format!("failed to load manager manifest from {}", base.display()))?;
        let base_manifest: RawManifest =
            serde_yaml::from_value(base_value).context("manager manifest schema is invalid")?;

        let overlay_manifest = if let Some(overlay_path) = overlay {
            let overlay_path = expand_path(overlay_path)?;
            match read_yaml_value_optional(overlay_path.clone())? {
                Some(value) => Some(
                    serde_yaml::from_value(value)
                        .context("overlay manager manifest schema is invalid")?,
                ),
                None => None,
            }
        } else {
            None
        };

        Self::from_raw(base_manifest, overlay_manifest)
    }

    fn from_raw(base: RawManifest, overlay: Option<RawManifest>) -> Result<Self> {
        let mut merged: HashMap<String, RawManagerSpec> = HashMap::new();
        insert_entries(
            &mut merged,
            parse_manager_entries(base.managers)?,
            "base manifest",
        )?;

        if let Some(overlay_manifest) = overlay {
            if overlay_manifest.version != base.version {
                bail!(
                    "overlay manifest version {} does not match base {}",
                    overlay_manifest.version,
                    base.version
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
            managers.push(ManagerSpec::from_raw(name, spec)?);
        }

        managers.sort_by(|a, b| {
            a.priority
                .cmp(&b.priority)
                .then_with(|| a.name.cmp(&b.name))
        });

        Ok(Self {
            version: base.version,
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
