use crate::execution::config_model::SubstrateConfig;
use anyhow::{anyhow, Context, Result};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use substrate_common::paths as substrate_paths;
use tempfile::NamedTempFile;

pub(crate) fn env_sh_path() -> Result<PathBuf> {
    Ok(substrate_paths::substrate_home()?.join("env.sh"))
}

pub(crate) fn write_env_sh(cfg: &SubstrateConfig) -> Result<()> {
    let substrate_home = substrate_paths::substrate_home()?;
    write_env_sh_at(&substrate_home.join("env.sh"), &substrate_home, cfg)
}

pub(crate) fn write_env_sh_at(
    path: &Path,
    substrate_home: &Path,
    cfg: &SubstrateConfig,
) -> Result<()> {
    let parent = path
        .parent()
        .ok_or_else(|| anyhow!("path {} has no parent", path.display()))?;
    fs::create_dir_all(parent).with_context(|| format!("failed to create {}", parent.display()))?;

    let mut tmp = NamedTempFile::new_in(parent)
        .with_context(|| format!("failed to create temp file near {}", path.display()))?;
    tmp.write_all(render_env_sh(substrate_home, cfg).as_bytes())
        .with_context(|| format!("failed to write {}", path.display()))?;
    tmp.flush()?;
    tmp.persist(path)
        .map_err(|err| anyhow!("failed to persist {}: {}", path.display(), err.error))?;
    Ok(())
}

fn render_env_sh(substrate_home: &Path, cfg: &SubstrateConfig) -> String {
    let world_state = if cfg.world.enabled {
        "enabled"
    } else {
        "disabled"
    };
    let caged = if cfg.world.caged { "1" } else { "0" };

    let mut out = String::new();
    out.push_str("#!/usr/bin/env bash\n");
    out.push_str(&format!(
        "export SUBSTRATE_HOME={}\n",
        bash_quote(&substrate_home.to_string_lossy())
    ));
    out.push_str(&format!(
        "export SUBSTRATE_WORLD={}\n",
        bash_quote(world_state)
    ));
    out.push_str(&format!("export SUBSTRATE_CAGED={}\n", bash_quote(caged)));
    out.push_str(&format!(
        "export SUBSTRATE_ANCHOR_MODE={}\n",
        bash_quote(cfg.world.anchor_mode.as_str())
    ));
    out.push_str(&format!(
        "export SUBSTRATE_ANCHOR_PATH={}\n",
        bash_quote(&cfg.world.anchor_path)
    ));
    out.push_str(&format!(
        "export SUBSTRATE_POLICY_MODE={}\n",
        bash_quote(cfg.policy.mode.as_str())
    ));
    out
}

fn bash_quote(raw: &str) -> String {
    let mut out = String::with_capacity(raw.len() + 2);
    out.push('\'');
    for ch in raw.chars() {
        if ch == '\'' {
            out.push_str("'\"'\"'");
        } else {
            out.push(ch);
        }
    }
    out.push('\'');
    out
}
