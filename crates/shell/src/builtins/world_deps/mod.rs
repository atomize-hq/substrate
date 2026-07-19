mod errors;
mod inventory;
mod surfaces;

pub use surfaces::run;

pub(crate) use inventory::AptSpecV1;
pub(crate) use inventory::InventoryListItemSummaryV1;

#[cfg(unix)]
use crate::execution::agent_runtime::host_session_authority::trusted_fs::TrustedAuthorityRoot;
#[cfg(unix)]
use crate::execution::agent_runtime::HostSessionAuthority;
use crate::execution::config_model;
#[cfg(unix)]
use crate::execution::install_bootstrap::bind_unix_install_bootstrap_context;
#[cfg(unix)]
use anyhow::anyhow;
use anyhow::Result;
use std::path::{Path, PathBuf};
use substrate_common::paths as substrate_paths;
#[cfg(unix)]
use transport_api_types::InstallBootstrapContextCarrierV1;

#[derive(Debug, Clone)]
pub(crate) struct WorldDepsProvisioningRequirementsV1 {
    pub apt: Vec<AptSpecV1>,
    pub pacman: Vec<String>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone)]
pub(crate) struct WorldDepsDoctorSnapshotV1 {
    pub schema_version: u32,
    pub cwd: PathBuf,
    pub inventory_packages: usize,
    pub inventory_bundles: usize,
    pub inventory_mode: String,
    pub builtins: String,
    pub enabled: Vec<String>,
    pub applied: Vec<InventoryListItemSummaryV1>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub applied_error: Option<String>,
}

pub(crate) fn collect_doctor_snapshot_v1(
    cwd: &Path,
    all: bool,
    #[cfg(unix)] install_context: &InstallBootstrapContextCarrierV1,
) -> Result<WorldDepsDoctorSnapshotV1> {
    #[cfg(unix)]
    let (cfg, global_deps_dir) = {
        bind_unix_install_bootstrap_context(install_context)?;
        let root =
            TrustedAuthorityRoot::open(Path::new(&install_context.context.selected_host_prefix))
                .map_err(|error| anyhow!("failed to open selected dependency root: {error}"))?;
        let authority = HostSessionAuthority::from_trusted_root(root)
            .map_err(|error| anyhow!("failed to bind selected dependency root: {error}"))?;
        let bootstrap_home = authority.bootstrap_home();
        let selected_root = PathBuf::from(
            &bootstrap_home
                .identity()
                .map_err(|error| anyhow!("failed to identify selected dependency root: {error}"))?
                .physical_path,
        );
        let (cfg, _) = config_model::resolve_effective_config_with_explain_for_bootstrap_home(
            cwd,
            &Default::default(),
            &bootstrap_home,
            false,
        )?;
        (cfg, Some(selected_root.join("deps")))
    };
    #[cfg(not(unix))]
    let (cfg, global_deps_dir) = {
        let cfg = config_model::resolve_effective_config(cwd, &Default::default())?;
        let global_deps_dir =
            if cfg.world.deps.inventory_mode == config_model::WorldDepsInventoryMode::Merged {
                Some(substrate_paths::substrate_home()?.join("deps"))
            } else {
                None
            };
        (cfg, global_deps_dir)
    };
    let view = surfaces::resolve_current_inventory_view(cwd, &cfg, global_deps_dir.as_deref())?;
    let enabled = cfg.world.deps.enabled.clone();
    let inventory_mode = match cfg.world.deps.inventory_mode {
        config_model::WorldDepsInventoryMode::Merged => "merged",
        config_model::WorldDepsInventoryMode::WorkspaceOnly => "workspace_only",
    }
    .to_string();
    let builtins = match cfg.world.deps.builtins {
        config_model::WorldDepsBuiltinsMode::Enabled => "enabled",
        config_model::WorldDepsBuiltinsMode::Disabled => "disabled",
    }
    .to_string();

    let applied = match surfaces::compute_current_applied_items_v1(&view, &enabled, all) {
        Ok(items) => WorldDepsDoctorSnapshotV1 {
            schema_version: 1,
            cwd: cwd.to_path_buf(),
            inventory_packages: view.packages.len(),
            inventory_bundles: view.bundles.len(),
            inventory_mode: inventory_mode.clone(),
            builtins: builtins.clone(),
            enabled,
            applied: items,
            applied_error: None,
        },
        Err(err) => WorldDepsDoctorSnapshotV1 {
            schema_version: 1,
            cwd: cwd.to_path_buf(),
            inventory_packages: view.packages.len(),
            inventory_bundles: view.bundles.len(),
            inventory_mode,
            builtins,
            enabled,
            applied: Vec::new(),
            applied_error: Some(format!("{:#}", err)),
        },
    };

    Ok(applied)
}

pub(crate) fn resolve_effective_enabled_provisioning_requirements_v1(
    cwd: &Path,
) -> Result<WorldDepsProvisioningRequirementsV1> {
    let cfg = config_model::resolve_effective_config(cwd, &Default::default())?;
    let global_deps_dir =
        if cfg.world.deps.inventory_mode == config_model::WorldDepsInventoryMode::Merged {
            Some(substrate_paths::substrate_home()?.join("deps"))
        } else {
            None
        };
    let view = surfaces::resolve_current_inventory_view(cwd, &cfg, global_deps_dir.as_deref())?;
    let apt = surfaces::resolve_enabled_apt_requirements_v1(&view, &cfg.world.deps.enabled)?;
    let pacman = surfaces::resolve_enabled_pacman_packages_v1(&view, &cfg.world.deps.enabled)?;

    Ok(WorldDepsProvisioningRequirementsV1 { apt, pacman })
}

#[cfg(all(test, unix))]
mod tests {
    use super::*;
    use crate::execution::install_bootstrap::current_unix_principal_and_home;
    use serial_test::serial;
    use std::ffi::{OsStr, OsString};
    use std::fs;
    use std::os::unix::fs::{MetadataExt, PermissionsExt};
    use tempfile::Builder;
    use transport_api_types::{InstallBootstrapContextCarrierV1, InstallBootstrapContextV1};

    struct EnvGuard(Vec<(&'static str, Option<OsString>)>);

    impl EnvGuard {
        fn apply(values: &[(&'static str, Option<&OsStr>)]) -> Self {
            let previous = values
                .iter()
                .map(|(key, _)| (*key, std::env::var_os(key)))
                .collect();
            for (key, value) in values {
                match value {
                    Some(value) => std::env::set_var(key, value),
                    None => std::env::remove_var(key),
                }
            }
            Self(previous)
        }
    }

    impl Drop for EnvGuard {
        fn drop(&mut self) {
            for (key, value) in self.0.drain(..) {
                match value {
                    Some(value) => std::env::set_var(key, value),
                    None => std::env::remove_var(key),
                }
            }
        }
    }

    fn snapshot_tree(root: &Path) -> Vec<(PathBuf, u32, u32, u32, Vec<u8>)> {
        fn walk(root: &Path, path: &Path, entries: &mut Vec<(PathBuf, u32, u32, u32, Vec<u8>)>) {
            let metadata = fs::symlink_metadata(path).expect("snapshot metadata");
            let relative = path.strip_prefix(root).expect("snapshot relative path");
            let bytes = if metadata.is_file() {
                fs::read(path).expect("snapshot file")
            } else {
                Vec::new()
            };
            entries.push((
                relative.to_path_buf(),
                metadata.mode(),
                metadata.uid(),
                metadata.gid(),
                bytes,
            ));
            if metadata.is_dir() {
                let mut children = fs::read_dir(path)
                    .expect("snapshot directory")
                    .map(|entry| entry.expect("snapshot entry").path())
                    .collect::<Vec<_>>();
                children.sort();
                for child in children {
                    walk(root, &child, entries);
                }
            }
        }

        let mut entries = Vec::new();
        walk(root, root, &mut entries);
        entries
    }

    fn carrier_for(prefix: &Path) -> InstallBootstrapContextCarrierV1 {
        let (principal, _) = current_unix_principal_and_home().expect("current Unix principal");
        let transport_api_types::PlatformPrincipalV1::Unix { account, uid } = principal else {
            panic!("expected Unix principal");
        };
        let context = InstallBootstrapContextV1::new_unix(
            prefix.to_str().expect("UTF-8 test prefix"),
            &account,
            uid,
        )
        .expect("valid test install context");
        InstallBootstrapContextCarrierV1::from_context(context).expect("committed test context")
    }

    fn write_inventory_package(prefix: &Path, name: &str) {
        let path = prefix.join("deps/packages").join(format!("{name}.yaml"));
        fs::create_dir_all(path.parent().expect("package parent")).expect("create deps inventory");
        fs::write(
            path,
            format!(
                "version: 1\nname: {name}\nrunnable: false\ninstall:\n  method: manual\n  manual_instructions: test only\n"
            ),
        )
        .expect("write deps inventory");
    }

    fn prepare_prefix(root: &Path, name: &str) -> PathBuf {
        let prefix = root.join(name);
        fs::create_dir(&prefix).expect("create selected prefix");
        fs::set_permissions(&prefix, fs::Permissions::from_mode(0o700))
            .expect("secure selected prefix");
        fs::write(
            prefix.join("config.yaml"),
            "world:\n  enabled: true\n  deps:\n    builtins: disabled\n    inventory_mode: merged\n    enabled: []\n",
        )
        .expect("write selected config");
        prefix
    }

    #[test]
    #[serial]
    fn doctor_snapshot_uses_authenticated_a_under_conflicting_ambient_b_without_mutation() {
        let (_, account_home) = current_unix_principal_and_home().expect("current Unix home");
        let temp = Builder::new()
            .prefix("substrate-route-d-snapshot-")
            .tempdir_in(account_home)
            .expect("secure snapshot fixture");
        let selected_a = prepare_prefix(temp.path(), "selected-a");
        let ambient_b = prepare_prefix(temp.path(), "ambient-b");
        write_inventory_package(&selected_a, "selected-only");
        write_inventory_package(&ambient_b, "ambient-one");
        write_inventory_package(&ambient_b, "ambient-two");
        let carrier = carrier_for(&selected_a);
        let selected_before = snapshot_tree(&selected_a);
        let ambient_before = snapshot_tree(&ambient_b);
        let _env = EnvGuard::apply(&[
            ("SUBSTRATE_HOME", Some(ambient_b.as_os_str())),
            ("SUBSTRATE_ROOT", Some(ambient_b.as_os_str())),
            ("SUBSTRATE_OVERRIDE_WORLD", None),
            ("SUBSTRATE_WORLD", None),
            ("SUBSTRATE_WORLD_ENABLED", None),
        ]);

        let snapshot = collect_doctor_snapshot_v1(temp.path(), false, &carrier)
            .expect("typed Route D snapshot");

        assert_eq!(snapshot.inventory_packages, 1);
        assert_eq!(snapshot.inventory_bundles, 0);
        assert_eq!(snapshot.inventory_mode, "merged");
        assert_eq!(snapshot.builtins, "disabled");
        assert_eq!(snapshot_tree(&selected_a), selected_before);
        assert_eq!(snapshot_tree(&ambient_b), ambient_before);
    }

    #[test]
    #[serial]
    fn doctor_snapshot_rejects_tampered_context_without_mutation_or_disclosure() {
        let (_, account_home) = current_unix_principal_and_home().expect("current Unix home");
        let temp = Builder::new()
            .prefix("substrate-route-d-tamper-")
            .tempdir_in(account_home)
            .expect("secure tamper fixture");
        let selected_a = prepare_prefix(temp.path(), "selected-a");
        let ambient_b = prepare_prefix(temp.path(), "ambient-b");
        let mut carrier = carrier_for(&selected_a);
        let encoded = carrier.encode().expect("encode valid carrier");
        let principal = match &carrier.context.intended_host_principal {
            transport_api_types::PlatformPrincipalV1::Unix { account, .. } => account.clone(),
            transport_api_types::PlatformPrincipalV1::Windows { .. } => unreachable!(),
        };
        let replacement = if carrier.host_context_commitment.starts_with('0') {
            "1"
        } else {
            "0"
        };
        carrier
            .host_context_commitment
            .replace_range(..1, replacement);
        let selected_before = snapshot_tree(&selected_a);
        let ambient_before = snapshot_tree(&ambient_b);
        let _env = EnvGuard::apply(&[
            ("SUBSTRATE_HOME", Some(ambient_b.as_os_str())),
            ("SUBSTRATE_ROOT", Some(ambient_b.as_os_str())),
        ]);

        let error = collect_doctor_snapshot_v1(temp.path(), false, &carrier)
            .expect_err("tampered Route D context must fail closed");
        let rendered = format!("{error:#}");

        assert!(!rendered.contains(&encoded));
        assert!(!rendered.contains(&principal));
        assert_eq!(snapshot_tree(&selected_a), selected_before);
        assert_eq!(snapshot_tree(&ambient_b), ambient_before);
    }
}
