use crate::manager_manifest::schema::{RawManagerEntry, RawManagerSpec};
use anyhow::{anyhow, bail, Context, Result};
use serde_yaml::Value;
use std::collections::{HashMap, HashSet};

pub(crate) fn parse_manager_entries(value: Value) -> Result<Vec<(String, RawManagerSpec)>> {
    match value {
        Value::Null => Ok(Vec::new()),
        Value::Sequence(entries) => entries
            .into_iter()
            .map(|entry| {
                let raw: RawManagerEntry =
                    serde_yaml::from_value(entry).context("manager entry must include a name")?;
                Ok((raw.name, raw.spec))
            })
            .collect(),
        Value::Mapping(map) => map
            .into_iter()
            .map(|(key, value)| {
                let key = key
                    .as_str()
                    .ok_or_else(|| anyhow!("manager names must be strings"))?
                    .to_string();
                let spec: RawManagerSpec = serde_yaml::from_value(value)
                    .with_context(|| format!("manager `{}` is invalid", key))?;
                Ok((key, spec))
            })
            .collect(),
        other => bail!(
            "`managers` must be a mapping or sequence, found {:?}",
            other
        ),
    }
}

pub(crate) fn insert_entries(
    target: &mut HashMap<String, RawManagerSpec>,
    entries: Vec<(String, RawManagerSpec)>,
    origin: &str,
) -> Result<()> {
    let mut seen = HashSet::new();
    for (name, spec) in entries {
        if !seen.insert(name.clone()) {
            bail!("duplicate manager entry `{}` in {}", name, origin);
        }
        if let Some(existing) = target.remove(&name) {
            target.insert(name, existing.merge(spec));
        } else {
            target.insert(name, spec);
        }
    }
    Ok(())
}
