use crate::execution::config_model::SubstrateConfig;
use anyhow::{anyhow, Context, Result};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use substrate_common::paths as substrate_paths;
use tempfile::NamedTempFile;
#[cfg(unix)]
use transport_api_types::{InstallBootstrapContextCarrierV1, PlatformPrincipalV1};

pub(crate) fn env_sh_path() -> Result<PathBuf> {
    Ok(substrate_paths::substrate_home()?.join("env.sh"))
}

#[cfg(unix)]
pub(crate) fn write_env_sh(cfg: &SubstrateConfig) -> Result<()> {
    let substrate_home = substrate_paths::substrate_home()?;
    let install_context =
        crate::execution::install_bootstrap::checked_install_bootstrap_context_from_projections()?;
    if Path::new(&install_context.context.selected_host_prefix) != substrate_home {
        return Err(anyhow!(
            "env.sh target does not match checked install bootstrap context"
        ));
    }
    let path = substrate_home.join("env.sh");
    let parent = path
        .parent()
        .ok_or_else(|| anyhow!("path {} has no parent", path.display()))?;
    fs::create_dir_all(parent).with_context(|| format!("failed to create {}", parent.display()))?;

    let mut tmp = NamedTempFile::new_in(parent)
        .with_context(|| format!("failed to create temp file near {}", path.display()))?;
    tmp.write_all(render_env_sh_for_context(&substrate_home, cfg, &install_context)?.as_bytes())
        .with_context(|| format!("failed to write {}", path.display()))?;
    tmp.flush()?;
    tmp.persist(&path)
        .map_err(|err| anyhow!("failed to persist {}: {}", path.display(), err.error))?;
    Ok(())
}

#[cfg(not(unix))]
pub(crate) fn write_env_sh(cfg: &SubstrateConfig) -> Result<()> {
    let substrate_home = substrate_paths::substrate_home()?;
    write_env_sh_at(&substrate_home.join("env.sh"), &substrate_home, cfg)
}

pub(crate) fn export_runtime_config_env(cfg: &SubstrateConfig) {
    std::env::set_var(
        "SUBSTRATE_WORLD_NET_FILTER",
        if cfg.world.net.filter { "1" } else { "0" },
    );
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
    let world_net_filter = if cfg.world.net.filter { "1" } else { "0" };

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
    out.push_str(&format!(
        "export SUBSTRATE_WORLD_NET_FILTER={}\n",
        bash_quote(world_net_filter)
    ));
    out
}

#[cfg(unix)]
fn render_env_sh_for_context(
    substrate_home: &Path,
    cfg: &SubstrateConfig,
    install_context: &InstallBootstrapContextCarrierV1,
) -> Result<String> {
    install_context
        .validate()
        .context("invalid install bootstrap context for env.sh")?;
    if Path::new(&install_context.context.selected_host_prefix) != substrate_home {
        return Err(anyhow!(
            "env.sh projection does not match install bootstrap context"
        ));
    }
    let encoded = install_context
        .encode()
        .context("failed to encode env.sh install bootstrap context")?;
    let PlatformPrincipalV1::Unix { account, uid } =
        &install_context.context.intended_host_principal
    else {
        return Err(anyhow!("env.sh requires a Unix install principal"));
    };
    let mut rendered = render_env_sh(substrate_home, cfg);
    rendered.push_str(&format!(
        "export SUBSTRATE_ROOT={}\n",
        bash_quote(&install_context.context.host_substrate_root)
    ));
    rendered.push_str(&format!(
        "export SUBSTRATE_INSTALL_HOST_CONTEXT_COMMITMENT={}\n",
        bash_quote(&install_context.host_context_commitment)
    ));
    rendered.push_str(&format!(
        "export SUBSTRATE_INSTALL_PRIMARY_USER={}\n",
        bash_quote(account)
    ));
    rendered.push_str(&format!(
        "export SUBSTRATE_INSTALL_PRIMARY_UID={}\n",
        bash_quote(&uid.to_string())
    ));
    rendered.push_str(&format!(
        "export SUBSTRATE_INSTALL_BOOTSTRAP_CONTEXT_V1={}\n",
        bash_quote(&encoded)
    ));
    Ok(rendered)
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

#[cfg(test)]
mod tests {
    use super::render_env_sh;
    #[cfg(unix)]
    use super::write_env_sh;
    use crate::execution::config_model::SubstrateConfig;
    #[cfg(unix)]
    use crate::execution::install_bootstrap::{
        current_unix_principal_and_home, install_bootstrap_projections,
        INSTALL_BOOTSTRAP_ACCOUNT_ENV, INSTALL_BOOTSTRAP_COMMITMENT_ENV,
        INSTALL_BOOTSTRAP_CONTEXT_ENV, INSTALL_BOOTSTRAP_UID_ENV,
    };
    #[cfg(unix)]
    use std::ffi::OsString;
    use std::path::Path;
    #[cfg(unix)]
    use transport_api_types::{
        InstallBootstrapContextCarrierV1, InstallBootstrapContextV1, PlatformPrincipalV1,
    };

    #[cfg(unix)]
    struct ProjectionEnvGuard(Vec<(&'static str, Option<OsString>)>);

    #[cfg(unix)]
    impl ProjectionEnvGuard {
        fn capture() -> Self {
            Self(
                [
                    "SUBSTRATE_HOME",
                    "SUBSTRATE_ROOT",
                    INSTALL_BOOTSTRAP_COMMITMENT_ENV,
                    INSTALL_BOOTSTRAP_ACCOUNT_ENV,
                    INSTALL_BOOTSTRAP_UID_ENV,
                    INSTALL_BOOTSTRAP_CONTEXT_ENV,
                ]
                .into_iter()
                .map(|key| (key, std::env::var_os(key)))
                .collect(),
            )
        }
    }

    #[cfg(unix)]
    impl Drop for ProjectionEnvGuard {
        fn drop(&mut self) {
            for (key, value) in self.0.drain(..) {
                match value {
                    Some(value) => std::env::set_var(key, value),
                    None => std::env::remove_var(key),
                }
            }
        }
    }

    #[test]
    fn render_env_sh_exports_world_net_filter_disabled_by_default() {
        let cfg = SubstrateConfig::default();
        let rendered = render_env_sh(Path::new("/tmp/substrate-home"), &cfg);
        assert!(rendered.contains("export SUBSTRATE_WORLD_NET_FILTER='0'\n"));
    }

    #[test]
    fn render_env_sh_exports_world_net_filter_when_enabled() {
        let mut cfg = SubstrateConfig::default();
        cfg.world.net.filter = true;

        let rendered = render_env_sh(Path::new("/tmp/substrate-home"), &cfg);
        assert!(rendered.contains("export SUBSTRATE_WORLD_NET_FILTER='1'\n"));
    }

    #[test]
    #[cfg(unix)]
    #[serial_test::serial]
    fn write_env_sh_preserves_authenticated_install_projection() {
        let _guard = ProjectionEnvGuard::capture();
        let home = tempfile::tempdir().unwrap();
        let (principal, _) = current_unix_principal_and_home().unwrap();
        let PlatformPrincipalV1::Unix { account, uid } = principal else {
            unreachable!();
        };
        let context =
            InstallBootstrapContextV1::new_unix(home.path().to_str().unwrap(), &account, uid)
                .unwrap();
        let carrier = InstallBootstrapContextCarrierV1::from_context(context).unwrap();
        let encoded = install_bootstrap_projections(&carrier).unwrap();

        write_env_sh(&SubstrateConfig::default()).unwrap();

        let rendered = std::fs::read_to_string(home.path().join("env.sh")).unwrap();
        assert!(rendered.contains(&format!(
            "export SUBSTRATE_ROOT='{}'\n",
            home.path().display()
        )));
        assert!(rendered.contains(&format!(
            "export SUBSTRATE_INSTALL_HOST_CONTEXT_COMMITMENT='{}'\n",
            carrier.host_context_commitment
        )));
        assert!(rendered.contains(&format!(
            "export SUBSTRATE_INSTALL_PRIMARY_USER='{}'\n",
            account
        )));
        assert!(rendered.contains(&format!("export SUBSTRATE_INSTALL_PRIMARY_UID='{uid}'\n")));
        assert!(rendered.contains(&format!(
            "export SUBSTRATE_INSTALL_BOOTSTRAP_CONTEXT_V1='{encoded}'\n"
        )));
    }
}
