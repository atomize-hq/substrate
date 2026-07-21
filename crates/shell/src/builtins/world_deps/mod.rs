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
use crate::execution::{policy_model, policy_snapshot};
#[cfg(unix)]
use anyhow::anyhow;
use anyhow::Result;
use std::path::{Path, PathBuf};
#[cfg(not(unix))]
use substrate_common::paths as substrate_paths;
#[cfg(unix)]
use transport_api_types::InstallBootstrapContextCarrierV1;

#[cfg(unix)]
pub(crate) struct AuthenticatedWorldDepsContextV1 {
    authority: HostSessionAuthority,
    selected_host_prefix: String,
    host_context_commitment: String,
    launch_cwd: PathBuf,
    workspace_root: Option<PathBuf>,
    global_config_path: PathBuf,
    global_deps_dir: PathBuf,
    effective_config: config_model::SubstrateConfig,
    config_explain: Option<config_model::ConfigExplainV1>,
    effective_policy: substrate_broker::Policy,
    runtime_network_policy: policy_snapshot::ResolvedWorldNetworkPolicy,
}

#[cfg(unix)]
impl std::fmt::Debug for AuthenticatedWorldDepsContextV1 {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("AuthenticatedWorldDepsContextV1")
            .field("selected_host_prefix", &self.selected_host_prefix)
            .field("host_context_commitment", &self.host_context_commitment)
            .field("launch_cwd", &self.launch_cwd)
            .finish_non_exhaustive()
    }
}

#[cfg(unix)]
impl AuthenticatedWorldDepsContextV1 {
    pub(crate) fn selected_host_prefix(&self) -> &str {
        &self.selected_host_prefix
    }

    pub(crate) fn host_context_commitment(&self) -> &str {
        &self.host_context_commitment
    }

    pub(crate) fn launch_cwd(&self) -> &Path {
        &self.launch_cwd
    }

    pub(crate) fn workspace_root(&self) -> Option<&Path> {
        self.workspace_root.as_deref()
    }

    pub(crate) fn global_config_path(&self) -> &Path {
        &self.global_config_path
    }

    pub(crate) fn global_deps_dir(&self) -> &Path {
        &self.global_deps_dir
    }

    pub(crate) fn effective_config(&self) -> &config_model::SubstrateConfig {
        &self.effective_config
    }

    pub(crate) fn config_explain(&self) -> Option<&config_model::ConfigExplainV1> {
        self.config_explain.as_ref()
    }

    pub(crate) fn effective_policy(&self) -> &substrate_broker::Policy {
        &self.effective_policy
    }

    pub(crate) fn runtime_network_policy(&self) -> &policy_snapshot::ResolvedWorldNetworkPolicy {
        &self.runtime_network_policy
    }

    pub(crate) fn world_fs_policy(&self) -> substrate_broker::WorldFsPolicy {
        self.effective_policy.world_fs_policy()
    }

    pub(crate) fn revalidate_authority(&self) -> Result<()> {
        self.authority
            .bootstrap_home()
            .revalidate()
            .map_err(|error| anyhow!("authenticated dependency root changed: {error}"))
    }
}

#[cfg(unix)]
pub(crate) fn bind_authenticated_world_deps_context_v1(
    carrier: &InstallBootstrapContextCarrierV1,
    launch_cwd: &Path,
    cli: &config_model::CliConfigOverrides,
) -> Result<AuthenticatedWorldDepsContextV1> {
    bind_unix_install_bootstrap_context(carrier)?;
    if !launch_cwd.is_absolute() {
        return Err(anyhow!(
            "authenticated world-deps launch cwd must be absolute"
        ));
    }

    let root = TrustedAuthorityRoot::open(Path::new(&carrier.context.selected_host_prefix))
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
    let (effective_config, config_explain) =
        config_model::resolve_effective_config_with_explain_for_bootstrap_home(
            launch_cwd,
            cli,
            &bootstrap_home,
            true,
        )?;
    let effective_policy =
        policy_model::resolve_effective_policy_for_bootstrap_home(launch_cwd, &bootstrap_home)?;
    let runtime_network_policy = policy_snapshot::resolve_world_network_policy_for_bootstrap_home(
        launch_cwd,
        &bootstrap_home,
        &effective_config,
    )?;
    Ok(AuthenticatedWorldDepsContextV1 {
        authority,
        selected_host_prefix: carrier.context.selected_host_prefix.clone(),
        host_context_commitment: carrier.host_context_commitment.clone(),
        launch_cwd: launch_cwd.to_path_buf(),
        workspace_root: crate::execution::find_workspace_root(launch_cwd),
        global_config_path: selected_root.join("config.yaml"),
        global_deps_dir: selected_root.join("deps"),
        effective_config,
        config_explain,
        effective_policy,
        runtime_network_policy,
    })
}

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
    #[cfg(not(unix))] cwd: &Path,
    #[cfg(unix)] context: &AuthenticatedWorldDepsContextV1,
) -> Result<WorldDepsProvisioningRequirementsV1> {
    #[cfg(unix)]
    let (cwd, cfg) = (context.launch_cwd(), context.effective_config());
    #[cfg(not(unix))]
    let cfg = config_model::resolve_effective_config(cwd, &Default::default())?;
    let global_deps_dir =
        if cfg.world.deps.inventory_mode == config_model::WorldDepsInventoryMode::Merged {
            #[cfg(unix)]
            {
                Some(context.global_deps_dir().to_path_buf())
            }
            #[cfg(not(unix))]
            {
                Some(substrate_paths::substrate_home()?.join("deps"))
            }
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

    struct EnvGuard {
        previous: Vec<(&'static str, Option<OsString>)>,
        _authority_env: Option<crate::execution::AuthorityEnvTestGuard>,
    }

    impl EnvGuard {
        fn apply(values: &[(&'static str, Option<&OsStr>)]) -> Self {
            let authority_env = values
                .iter()
                .any(|(key, _)| matches!(*key, "SUBSTRATE_HOME" | "SUBSTRATE_WORLD_SOCKET"))
                .then(crate::execution::AuthorityEnvTestGuard::preserve);
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
            Self {
                previous,
                _authority_env: authority_env,
            }
        }
    }

    impl Drop for EnvGuard {
        fn drop(&mut self) {
            for (key, value) in self.previous.drain(..) {
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
    fn authenticated_context_binds_a_projection_under_conflicting_ambient_b() {
        let _authority_env = crate::execution::AuthorityEnvTestGuard::preserve();
        let (_, account_home) = current_unix_principal_and_home().expect("current Unix home");
        let temp = Builder::new()
            .prefix("substrate-world-deps-context-")
            .tempdir_in(account_home)
            .expect("secure context fixture");
        fs::create_dir(temp.path().join(".substrate")).expect("create workspace metadata");
        fs::write(temp.path().join(".substrate/workspace.yaml"), "")
            .expect("write workspace marker");
        let selected_a = prepare_prefix(temp.path(), "selected-a");
        let ambient_b = prepare_prefix(temp.path(), "ambient-b");
        fs::write(
            selected_a.join("policy.yaml"),
            "id: selected-policy\nnet_allowed: [selected.example]\nworld_fs:\n  host_visible: false\n  write:\n    enabled: false\n  fail_closed:\n    routing: true\n",
        )
        .expect("write selected policy");
        fs::write(
            ambient_b.join("config.yaml"),
            "world:\n  enabled: true\n  deps:\n    builtins: enabled\n    inventory_mode: workspace_only\n    enabled: [ambient-only]\n",
        )
        .expect("write ambient config");
        fs::write(
            ambient_b.join("policy.yaml"),
            "id: ambient-policy\nnet_allowed: [ambient.example]\n",
        )
        .expect("write ambient policy");
        let carrier = carrier_for(&selected_a);
        let _env = EnvGuard::apply(&[
            ("SUBSTRATE_HOME", Some(ambient_b.as_os_str())),
            ("SUBSTRATE_ROOT", Some(ambient_b.as_os_str())),
        ]);

        let context = bind_authenticated_world_deps_context_v1(
            &carrier,
            temp.path(),
            &config_model::CliConfigOverrides::default(),
        )
        .expect("bind authenticated world-deps context");

        context
            .revalidate_authority()
            .expect("retained authority remains valid");
        assert_eq!(context.selected_host_prefix(), selected_a.to_str().unwrap());
        assert_eq!(
            context.host_context_commitment(),
            carrier.host_context_commitment
        );
        assert_eq!(context.launch_cwd(), temp.path());
        assert_eq!(context.workspace_root(), Some(temp.path()));
        assert_eq!(context.global_config_path(), selected_a.join("config.yaml"));
        assert_eq!(context.global_deps_dir(), selected_a.join("deps"));
        assert_eq!(
            context.effective_config().world.deps.builtins,
            config_model::WorldDepsBuiltinsMode::Disabled
        );
        assert_eq!(context.effective_policy().id, "selected-policy");
        assert_eq!(
            context.runtime_network_policy().snapshot.net_allowed,
            vec!["selected.example".to_string()]
        );
        assert_eq!(
            context.world_fs_policy(),
            context.effective_policy().world_fs_policy()
        );
    }

    #[test]
    #[serial]
    fn authenticated_context_rejects_tampered_commitment_before_mutation() {
        let _authority_env = crate::execution::AuthorityEnvTestGuard::preserve();
        let (_, account_home) = current_unix_principal_and_home().expect("current Unix home");
        let temp = Builder::new()
            .prefix("substrate-world-deps-context-tamper-")
            .tempdir_in(account_home)
            .expect("secure tamper fixture");
        let selected_a = prepare_prefix(temp.path(), "selected-a");
        let ambient_b = prepare_prefix(temp.path(), "ambient-b");
        fs::write(selected_a.join("config.yaml"), "world: [")
            .expect("poison selected config ordering sentinel");
        fs::write(selected_a.join("policy.yaml"), "id: [")
            .expect("poison selected policy ordering sentinel");
        let mut carrier = carrier_for(&selected_a);
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

        let error = bind_authenticated_world_deps_context_v1(
            &carrier,
            temp.path(),
            &config_model::CliConfigOverrides::default(),
        )
        .expect_err("tampered authenticated context must fail closed");

        assert!(format!("{error:#}").contains("invalid install bootstrap carrier"));
        assert_eq!(snapshot_tree(&selected_a), selected_before);
        assert_eq!(snapshot_tree(&ambient_b), ambient_before);
    }

    #[test]
    #[serial]
    fn doctor_snapshot_uses_authenticated_a_under_conflicting_ambient_b_without_mutation() {
        let _authority_env = crate::execution::AuthorityEnvTestGuard::preserve();
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
        let _authority_env = crate::execution::AuthorityEnvTestGuard::preserve();
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
