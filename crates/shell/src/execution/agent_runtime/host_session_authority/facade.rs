//! Exact A1 host-session authority facade.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
#[cfg(any(test, target_os = "linux"))]
use std::path::Path;

use super::canonical_json;
use super::hash::canonical_sha256;
#[cfg(test)]
use super::schema::TimestampV1;
use super::schema::{
    AgentDescriptorHashInputV1, AgentDescriptorV1, AuthoritativeLineageHashInputV1,
    AuthorityObjectCommitmentV1, AuthorityObjectRefV1, CanonicalDirectoryV1,
    DurableSessionAuthorityHashInputV1, DurableSessionAuthorityOriginV1,
    HostAttachContractHashInputV1, HostAttachContractV1, HostSessionAuthorityPreconditionV1,
    HostSessionPostureV1, HostSessionTransitionModeV1, PolicyObjectHashInputV1,
    StartContinuationHandleHashInputV2, StartContinuationHandleStateV2, WorldBindingV1,
};
use super::store::{
    self, BootstrapClassificationV1, ExpectedRevisionsV1, ObjectPublicationOutcomeV1,
    ObjectVerificationContextV1, TransactionCommitOutcomeV1,
};
use super::store_schema::{
    DurableSessionAuthorityV1, HostSessionPostTurnApplicationV1, HostSessionPostTurnApplicationV2,
    HostSessionStartupOwnershipApplicationV1, HostSessionStopIntentStateV1,
    HostSessionTransitionIntentStateV2, HostSessionTransitionIntentStateV3,
    HostSessionTransitionIntentV3, RetainedWorkerAuthorityRegistrationRequestStateV1,
    RetainedWorkerAuthorityRegistrationRequestV1, RetainedWorkerAuthorityRegistrationV1,
    SessionNamespaceRecordV1, StartTransactionStateV1, StateRootV1, StateRootV2, StateRootV3,
    VersionedStateRoot,
};
use super::transition::{verify_applied_start, ApplyHostSessionTransitionRequestV1};
#[cfg(any(target_os = "linux", target_os = "macos"))]
use super::trusted_fs::EntryKind;
use super::trusted_fs::TrustedAuthorityRoot;
use super::validation::validate_fork_successor_attach_semantics;

#[derive(Debug)]
pub(crate) struct HostSessionAuthority {
    root: TrustedAuthorityRoot,
}

/// Shell-owned, sealed implementation of the E3 projection HSA bridge.
#[cfg(target_os = "linux")]
#[derive(Debug)]
pub struct OpenedConfigProjectionHsaAuthorityV1 {
    root: TrustedAuthorityRoot,
}

#[cfg(target_os = "linux")]
impl OpenedConfigProjectionHsaAuthorityV1 {
    pub fn from_configured_accepted_home(
        configured: &config_projection::ConfiguredAcceptedHomeAuthorityV1,
    ) -> Result<Self, config_projection::ConfigProjectionFailureV1> {
        configured.revalidate()?;
        let owner_uid = libc::uid_t::try_from(configured.intended_uid())
            .map_err(|_| config_projection::ConfigProjectionFailureV1::Malformed)?;
        let accepted = configured.accepted_home();
        let root =
            TrustedAuthorityRoot::open_for_owner(Path::new(&accepted.physical_path), owner_uid)
                .map_err(|_| {
                    config_projection::ConfigProjectionFailureV1::UnsupportedSecurityPosture
                })?;
        let shell_identity = root.identity();
        let identities_match = match (
            &shell_identity.physical_identity,
            &accepted.physical_identity,
        ) {
            (
                super::schema::DirectoryPhysicalIdentityV1::Linux {
                    device_id: shell_device,
                    inode: shell_inode,
                },
                config_projection::DirectoryPhysicalIdentityV1::Linux {
                    device_id: accepted_device,
                    inode: accepted_inode,
                },
            ) => shell_device == accepted_device && shell_inode == accepted_inode,
            _ => false,
        };
        if shell_identity.physical_path != accepted.physical_path || !identities_match {
            return Err(config_projection::ConfigProjectionFailureV1::WrongBinding);
        }
        configured.revalidate()?;
        root.revalidate().map_err(|_| {
            config_projection::ConfigProjectionFailureV1::UnsupportedSecurityPosture
        })?;
        Ok(Self { root })
    }
}

#[cfg(target_os = "linux")]
impl config_projection::ConfigProjectionHsaAuthorityV1 for OpenedConfigProjectionHsaAuthorityV1 {
    fn with_locked_parent(
        &self,
        operation: &mut dyn for<'fd> FnMut(
            std::os::fd::BorrowedFd<'fd>,
        ) -> Result<
            (),
            config_projection::ConfigProjectionFailureV1,
        >,
    ) -> Result<(), config_projection::ConfigProjectionFailureV1> {
        store::with_config_projection_hsa_parent(&self.root, operation)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct AuthorityObservationV1 {
    pub(crate) authority_store_id: String,
    pub(crate) bootstrap_home: CanonicalDirectoryV1,
    pub(crate) orchestration_session_id: String,
    pub(crate) root_revision: u64,
    pub(crate) authority_revision: u64,
    pub(crate) authority_record_commitment: AuthorityObjectCommitmentV1,
    pub(crate) authoritative_lineage_commitment: AuthorityObjectCommitmentV1,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ResolvedSessionAuthorityV1 {
    pub(crate) root_revision: u64,
    pub(crate) authority: DurableSessionAuthorityV1,
    pub(crate) authority_record_commitment: AuthorityObjectCommitmentV1,
    pub(crate) authoritative_lineage_commitment: AuthorityObjectCommitmentV1,
    authority_store_id: String,
    bootstrap_home: CanonicalDirectoryV1,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum AuthorityParticipantRoleV1 {
    Orchestrator,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ResolvedAuthorityCallerV1 {
    pub(crate) participant_id: String,
    pub(crate) role: AuthorityParticipantRoleV1,
    pub(crate) descriptor_ref: AuthorityObjectRefV1,
    pub(crate) descriptor: AgentDescriptorV1,
}

#[derive(Clone, Debug)]
pub(crate) struct ResolvedCurrentAuthorityV1 {
    pub(crate) observation: AuthorityObservationV1,
    pub(crate) authority: DurableSessionAuthorityV1,
    pub(crate) caller: ResolvedAuthorityCallerV1,
    pub(crate) host_attach_contract: HostAttachContractV1,
    pub(crate) current_policy: PolicyObjectHashInputV1,
    pub(crate) bound_state_store: super::super::state_store::BoundAgentRuntimeStateStore,
}

#[cfg(test)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum RetainedReservationCrashPointV1 {
    BeforeRootPublication,
    AfterRootPublication,
}

#[cfg(test)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum RetainedApplicationCrashPointV1 {
    BeforeRootPublication,
    AfterRootPublication,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct RetainedWorkerAuthorityPreconditionV1 {
    pub(crate) authority_store_id: String,
    pub(crate) authority_revision: u64,
    pub(crate) authority_record_commitment: AuthorityObjectCommitmentV1,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ReservedRetainedWorkerRegistrationV1 {
    pub(crate) request: RetainedWorkerAuthorityRegistrationRequestV1,
    pub(crate) descriptor_ref: AuthorityObjectRefV1,
    pub(crate) resume_handle_ref: AuthorityObjectRefV1,
    pub(crate) retained_worker_ref: AuthorityObjectRefV1,
    pub(crate) descriptor_bytes: Vec<u8>,
    pub(crate) resume_handle_bytes: Vec<u8>,
    pub(crate) retained_worker_bytes: Vec<u8>,
    pub(crate) joined: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct AppliedRetainedWorkerRegistrationV1 {
    pub(crate) registration: RetainedWorkerAuthorityRegistrationV1,
    pub(crate) joined: bool,
}

impl ResolvedSessionAuthorityV1 {
    pub(crate) fn observation(&self) -> AuthorityObservationV1 {
        AuthorityObservationV1 {
            authority_store_id: self.authority_store_id.clone(),
            bootstrap_home: self.bootstrap_home.clone(),
            orchestration_session_id: self.authority.orchestration_session_id.clone(),
            root_revision: self.root_revision,
            authority_revision: self.authority.authority_revision,
            authority_record_commitment: self.authority_record_commitment.clone(),
            authoritative_lineage_commitment: self.authoritative_lineage_commitment.clone(),
        }
    }
}

pub(crate) fn validate_locked_post_hsa_authority_binding(
    root: &VersionedStateRoot,
    expected_observation: &AuthorityObservationV1,
    expected_router_auto_attach_intent: Option<&HostSessionTransitionIntentV3>,
) -> Result<(), AuthorityFacadeError> {
    let (authority_store_id, bootstrap_home, root_revision, session_namespace_map) = match root {
        VersionedStateRoot::V1(_) => {
            return Err(AuthorityFacadeError(
                "post-HSA authority binding requires strict StateRootV2 or StateRootV3".into(),
            ));
        }
        VersionedStateRoot::V2(root) => (
            &root.authority_store_id,
            &root.bootstrap_home,
            root.root_revision,
            &root.session_namespace_map,
        ),
        VersionedStateRoot::V3(root) => (
            &root.authority_store_id,
            &root.bootstrap_home,
            root.root_revision,
            &root.session_namespace_map,
        ),
    };
    let Some(SessionNamespaceRecordV1::Authority(authority)) =
        session_namespace_map.get(&expected_observation.orchestration_session_id)
    else {
        return Err(AuthorityFacadeError(
            "locked post-HSA root does not contain the exact durable session authority".into(),
        ));
    };
    let authority_record_commitment = canonical_commitment(&authority_hash_input(authority))?;
    let authoritative_lineage_commitment =
        canonical_commitment(&AuthoritativeLineageHashInputV1 {
            schema_version: 1,
            orchestration_session_id: authority.orchestration_session_id.clone(),
            participant_ids: authority.authoritative_participant_lineage.clone(),
        })?;
    let locked_observation = AuthorityObservationV1 {
        authority_store_id: authority_store_id.clone(),
        bootstrap_home: bootstrap_home.clone(),
        orchestration_session_id: authority.orchestration_session_id.clone(),
        root_revision,
        authority_revision: authority.authority_revision,
        authority_record_commitment,
        authoritative_lineage_commitment,
    };
    if &locked_observation != expected_observation {
        return Err(AuthorityFacadeError(
            "locked post-HSA authority observation is stale or mismatched".into(),
        ));
    }

    if let Some(expected_intent) = expected_router_auto_attach_intent {
        let VersionedStateRoot::V3(root) = root else {
            return Err(AuthorityFacadeError(
                "RouterAutoAttach settlement requires strict StateRootV3".into(),
            ));
        };
        if root
            .successor_transition_intent_map
            .get(&expected_intent.intent_id)
            != Some(expected_intent)
            || !root
                .successor_application_journal
                .contains_key(&expected_intent.intent_id)
        {
            return Err(AuthorityFacadeError(
                "locked post-HSA root does not contain the exact applied RouterAutoAttach intent"
                    .into(),
            ));
        }
    }
    Ok(())
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct AuthorityFacadeError(String);

impl fmt::Display for AuthorityFacadeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl std::error::Error for AuthorityFacadeError {}

#[cfg(all(test, target_os = "linux"))]
mod e3_b_hsa_bridge_tests {
    use std::collections::BTreeMap;
    use std::fs::File;
    use std::os::fd::AsFd;
    use std::os::unix::fs::PermissionsExt;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::{mpsc, Arc, Barrier};
    use std::time::Duration;

    use config_projection::{
        CanonicalDirectoryV1 as ProjectionDirectoryV1, ConfigProjectionCodecV1,
        ConfigProjectionFailureV1, ConfigProjectionHsaAuthorityV1, ConfigProjectionRegistryV1,
        ConfiguredAcceptedHomeAuthorityV1, InstalledAcceptedHomeBootstrapRecordV1, Timestamp,
    };
    use transport_api_types::{InstallBootstrapContextCarrierV1, InstallBootstrapContextV1};

    use super::{HostSessionAuthority, OpenedConfigProjectionHsaAuthorityV1};
    use crate::execution::agent_runtime::host_session_authority::store;

    fn current_account() -> String {
        let uid = unsafe { libc::geteuid() };
        let mut pwd = std::mem::MaybeUninit::<libc::passwd>::uninit();
        let mut result = std::ptr::null_mut();
        let mut buffer = vec![0_u8; 16 * 1024];
        assert_eq!(
            unsafe {
                libc::getpwuid_r(
                    uid,
                    pwd.as_mut_ptr(),
                    buffer.as_mut_ptr().cast(),
                    buffer.len(),
                    &mut result,
                )
            },
            0
        );
        assert!(!result.is_null());
        unsafe { std::ffi::CStr::from_ptr((*result).pw_name) }
            .to_str()
            .unwrap()
            .to_string()
    }

    fn configured(home: &std::path::Path) -> ConfiguredAcceptedHomeAuthorityV1 {
        let descriptor = File::open(home).unwrap();
        let accepted_home =
            ProjectionDirectoryV1::capture_linux_from_fd(descriptor.as_fd()).unwrap();
        let uid = unsafe { libc::geteuid() };
        let gid = unsafe { libc::getegid() };
        let carrier = InstallBootstrapContextCarrierV1::from_context(
            InstallBootstrapContextV1::new_unix(
                accepted_home.physical_path.as_str(),
                &current_account(),
                uid,
            )
            .unwrap(),
        )
        .unwrap();
        let mut record = InstalledAcceptedHomeBootstrapRecordV1 {
            schema_version: 1,
            install_bootstrap_carrier: carrier.encode().unwrap(),
            host_context_commitment: carrier.host_context_commitment,
            intended_account: current_account(),
            intended_uid: u64::from(uid),
            intended_gid: u64::from(gid),
            accepted_home,
            installed_at: Timestamp("2026-09-10T00:00:00.000000Z".to_string()),
            record_hash: String::new(),
        };
        let mut value = serde_json::to_value(&record).unwrap();
        value.as_object_mut().unwrap().remove("record_hash");
        let mut payload = BTreeMap::new();
        payload.insert("record", value);
        record.record_hash = ConfigProjectionCodecV1::domain_sha256(
            "substrate.e3.installed-accepted-home-bootstrap.v1",
            &payload,
        )
        .unwrap();
        ConfiguredAcceptedHomeAuthorityV1::from_record_for_test(record).unwrap()
    }

    struct TestHome {
        _parent: tempfile::TempDir,
        path: std::path::PathBuf,
    }

    impl TestHome {
        fn path(&self) -> &std::path::Path {
            &self.path
        }
    }

    fn activated_home() -> (TestHome, HostSessionAuthority) {
        let safe_parent = std::env::var_os("XDG_RUNTIME_DIR")
            .map(std::path::PathBuf::from)
            .unwrap_or_else(|| {
                std::path::PathBuf::from(std::env::var_os("HOME").expect("tests require HOME"))
                    .join(".cache")
            });
        std::fs::create_dir_all(&safe_parent).unwrap();
        let parent = tempfile::tempdir_in(safe_parent).unwrap();
        std::fs::set_permissions(parent.path(), std::fs::Permissions::from_mode(0o700)).unwrap();
        let path = parent.path().join("home");
        std::fs::create_dir(&path).unwrap();
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o700)).unwrap();
        let authority = HostSessionAuthority::open(&path).unwrap();
        authority.bootstrap().unwrap();
        (
            TestHome {
                _parent: parent,
                path,
            },
            authority,
        )
    }

    #[test]
    fn e3_b_hsa_bridge_callback_is_exactly_once_and_e2_rm_reads_after_creation() {
        let (home, authority) = activated_home();
        let configured = configured(home.path());
        let bridge = Arc::new(
            OpenedConfigProjectionHsaAuthorityV1::from_configured_accepted_home(&configured)
                .unwrap(),
        );
        let calls = AtomicUsize::new(0);
        bridge
            .with_locked_parent(&mut |_| {
                calls.fetch_add(1, Ordering::SeqCst);
                Ok(())
            })
            .unwrap();
        assert_eq!(calls.load(Ordering::SeqCst), 1);

        let registry = ConfigProjectionRegistryV1::open(bridge).unwrap();
        registry.recover().unwrap();
        let snapshot = store::read_existing_accepted_work_authority_snapshot(&authority).unwrap();
        assert!(!snapshot.hsa_state_root_bytes().is_empty());
    }

    #[test]
    fn e3_b_hsa_bridge_rejected_substitution_never_invokes_callback() {
        let (home, _authority) = activated_home();
        let configured = configured(home.path());
        let bridge =
            OpenedConfigProjectionHsaAuthorityV1::from_configured_accepted_home(&configured)
                .unwrap();
        let moved = home.path().with_extension("held-e3-original");
        std::fs::rename(home.path(), &moved).unwrap();
        std::fs::create_dir(home.path()).unwrap();
        std::fs::set_permissions(home.path(), std::fs::Permissions::from_mode(0o700)).unwrap();
        assert!(
            OpenedConfigProjectionHsaAuthorityV1::from_configured_accepted_home(&configured)
                .is_err()
        );
        let calls = AtomicUsize::new(0);
        assert!(bridge
            .with_locked_parent(&mut |_| {
                calls.fetch_add(1, Ordering::SeqCst);
                Ok(())
            })
            .is_err());
        assert_eq!(calls.load(Ordering::SeqCst), 0);
        std::fs::remove_dir(home.path()).unwrap();
        std::fs::rename(moved, home.path()).unwrap();
    }

    #[test]
    fn e3_b_hsa_bridge_parent_lock_releases_on_error_unwind_and_process_death() {
        let (home, _authority) = activated_home();
        let configured = configured(home.path());
        let bridge = Arc::new(
            OpenedConfigProjectionHsaAuthorityV1::from_configured_accepted_home(&configured)
                .unwrap(),
        );
        assert_eq!(
            bridge.with_locked_parent(&mut |_| Err(ConfigProjectionFailureV1::Conflict)),
            Err(ConfigProjectionFailureV1::Conflict)
        );
        let unwind = std::panic::catch_unwind(std::panic::AssertUnwindSafe({
            let bridge = Arc::clone(&bridge);
            move || {
                let _ = bridge.with_locked_parent(&mut |_| panic!("intentional E3-B unwind"));
            }
        }));
        assert!(unwind.is_err());
        bridge.with_locked_parent(&mut |_| Ok(())).unwrap();

        let child = unsafe { libc::fork() };
        assert!(child >= 0);
        if child == 0 {
            let _ = bridge.with_locked_parent(&mut |_| unsafe { libc::_exit(0) });
            unsafe { libc::_exit(127) };
        }
        let mut status = 0;
        assert_eq!(unsafe { libc::waitpid(child, &mut status, 0) }, child);
        assert_eq!(libc::WEXITSTATUS(status), 0);
        bridge.with_locked_parent(&mut |_| Ok(())).unwrap();
    }

    #[test]
    fn e3_b_hsa_bridge_serializes_concurrent_hsa_and_e3_access() {
        let (home, _authority) = activated_home();
        let configured = configured(home.path());
        let bridge = Arc::new(
            OpenedConfigProjectionHsaAuthorityV1::from_configured_accepted_home(&configured)
                .unwrap(),
        );
        let barrier = Arc::new(Barrier::new(2));
        let (entered_tx, entered_rx) = mpsc::channel();
        let (release_tx, release_rx) = mpsc::channel();
        let holder = {
            let bridge = Arc::clone(&bridge);
            let barrier = Arc::clone(&barrier);
            std::thread::spawn(move || {
                barrier.wait();
                bridge
                    .with_locked_parent(&mut |_| {
                        entered_tx.send(()).unwrap();
                        release_rx.recv().unwrap();
                        Ok(())
                    })
                    .unwrap();
            })
        };
        barrier.wait();
        entered_rx.recv_timeout(Duration::from_secs(5)).unwrap();
        let (done_tx, done_rx) = mpsc::channel();
        let ordinary = {
            let path = home.path().to_path_buf();
            let done_tx = done_tx.clone();
            std::thread::spawn(move || {
                let authority = HostSessionAuthority::open(&path).unwrap();
                authority.read_root().unwrap();
                done_tx.send("ordinary").unwrap();
            })
        };
        let e2_rm = {
            let path = home.path().to_path_buf();
            std::thread::spawn(move || {
                let authority = HostSessionAuthority::open(&path).unwrap();
                store::read_existing_accepted_work_authority_snapshot(&authority).unwrap();
                done_tx.send("e2-rm").unwrap();
            })
        };
        assert!(done_rx.recv_timeout(Duration::from_millis(100)).is_err());
        release_tx.send(()).unwrap();
        let mut completed = [
            done_rx.recv_timeout(Duration::from_secs(5)).unwrap(),
            done_rx.recv_timeout(Duration::from_secs(5)).unwrap(),
        ];
        completed.sort_unstable();
        assert_eq!(completed, ["e2-rm", "ordinary"]);
        holder.join().unwrap();
        ordinary.join().unwrap();
        e2_rm.join().unwrap();
    }

    #[test]
    fn e3_b_hsa_bridge_unknown_authority_entry_is_rejected_and_preserved() {
        let (home, _authority) = activated_home();
        let configured = configured(home.path());
        let bridge =
            OpenedConfigProjectionHsaAuthorityV1::from_configured_accepted_home(&configured)
                .unwrap();
        let unknown = home.path().join("authority-v1").join("unknown-e3-neighbor");
        std::fs::create_dir(&unknown).unwrap();
        let calls = AtomicUsize::new(0);
        assert!(bridge
            .with_locked_parent(&mut |_| {
                calls.fetch_add(1, Ordering::SeqCst);
                Ok(())
            })
            .is_err());
        assert_eq!(calls.load(Ordering::SeqCst), 0);
        assert!(unknown.is_dir());
    }
}

pub(crate) struct OpenedBootstrapHomeV1<'authority> {
    root: &'authority TrustedAuthorityRoot,
}

impl OpenedBootstrapHomeV1<'_> {
    pub(crate) fn identity(&self) -> Result<&CanonicalDirectoryV1, AuthorityFacadeError> {
        self.revalidate()?;
        #[cfg(any(target_os = "linux", target_os = "macos"))]
        {
            Ok(self.root.identity())
        }
        #[cfg(not(any(target_os = "linux", target_os = "macos")))]
        {
            Err(unsupported_platform())
        }
    }

    pub(crate) fn revalidate(&self) -> Result<(), AuthorityFacadeError> {
        #[cfg(any(target_os = "linux", target_os = "macos"))]
        {
            self.root
                .revalidate()
                .map_err(|error| AuthorityFacadeError(error.to_string()))
        }
        #[cfg(not(any(target_os = "linux", target_os = "macos")))]
        {
            Err(unsupported_platform())
        }
    }

    pub(crate) fn read_config_yaml(&self) -> Result<Option<Vec<u8>>, AuthorityFacadeError> {
        self.read_optional_file("config.yaml")
    }

    pub(crate) fn read_policy_yaml(&self) -> Result<Option<Vec<u8>>, AuthorityFacadeError> {
        self.read_optional_file("policy.yaml")
    }

    pub(crate) fn read_agent_inventory_yaml(
        &self,
    ) -> Result<Vec<(String, Vec<u8>)>, AuthorityFacadeError> {
        self.read_agent_inventory_yaml_after_revalidation(|| {})
    }

    fn read_agent_inventory_yaml_after_revalidation(
        &self,
        after_revalidation: impl FnOnce(),
    ) -> Result<Vec<(String, Vec<u8>)>, AuthorityFacadeError> {
        #[cfg(any(target_os = "linux", target_os = "macos"))]
        {
            self.revalidate()?;
            after_revalidation();
            let root = self.root.directory();
            let Some(kind) = root.entry_kind("agents").map_err(trusted_fs_error)? else {
                self.revalidate()?;
                return Ok(Vec::new());
            };
            if kind != EntryKind::Directory {
                return Err(AuthorityFacadeError(
                    "bootstrap-home agents entry is not a directory".into(),
                ));
            }
            let directory = root.open_directory("agents").map_err(trusted_fs_error)?;
            let mut files = Vec::new();
            for entry in directory.entries().map_err(trusted_fs_error)? {
                if !entry.name.ends_with(".yaml") {
                    continue;
                }
                if entry.kind != EntryKind::RegularFile {
                    return Err(AuthorityFacadeError(
                        "bootstrap-home YAML inventory entry is not a regular file".into(),
                    ));
                }
                let bytes = directory
                    .open_file_entry(&entry)
                    .and_then(|file| file.read_all())
                    .map_err(trusted_fs_error)?;
                directory
                    .revalidate_entry(&entry)
                    .map_err(trusted_fs_error)?;
                files.push((entry.name, bytes));
            }
            files.sort_by(|left, right| left.0.cmp(&right.0));
            self.revalidate()?;
            Ok(files)
        }
        #[cfg(not(any(target_os = "linux", target_os = "macos")))]
        {
            let _ = after_revalidation;
            Err(unsupported_platform())
        }
    }

    fn read_optional_file(&self, name: &str) -> Result<Option<Vec<u8>>, AuthorityFacadeError> {
        self.read_optional_file_after_revalidation(name, || {})
    }

    fn read_optional_file_after_revalidation(
        &self,
        name: &str,
        after_revalidation: impl FnOnce(),
    ) -> Result<Option<Vec<u8>>, AuthorityFacadeError> {
        #[cfg(any(target_os = "linux", target_os = "macos"))]
        {
            self.revalidate()?;
            after_revalidation();
            let root = self.root.directory();
            let Some(kind) = root.entry_kind(name).map_err(trusted_fs_error)? else {
                self.revalidate()?;
                return Ok(None);
            };
            if kind != EntryKind::RegularFile {
                return Err(AuthorityFacadeError(
                    "bootstrap-home configuration entry is not a regular file".into(),
                ));
            }
            let bytes = root
                .open_file(name)
                .and_then(|file| file.read_all())
                .map_err(trusted_fs_error)?;
            self.revalidate()?;
            Ok(Some(bytes))
        }
        #[cfg(not(any(target_os = "linux", target_os = "macos")))]
        {
            let _ = (name, after_revalidation);
            Err(unsupported_platform())
        }
    }
}

impl HostSessionAuthority {
    pub(super) fn trusted_root(&self) -> &TrustedAuthorityRoot {
        &self.root
    }

    pub(crate) fn from_trusted_root(
        root: TrustedAuthorityRoot,
    ) -> Result<Self, AuthorityFacadeError> {
        #[cfg(any(target_os = "linux", target_os = "macos"))]
        {
            root.revalidate()
                .map_err(|error| AuthorityFacadeError(error.to_string()))?;
            Ok(Self { root })
        }
        #[cfg(not(any(target_os = "linux", target_os = "macos")))]
        {
            let _ = root;
            Err(unsupported_platform())
        }
    }

    #[cfg(test)]
    pub(crate) fn open(path: &Path) -> Result<Self, AuthorityFacadeError> {
        let root = TrustedAuthorityRoot::open(path)
            .map_err(|error| AuthorityFacadeError(error.to_string()))?;
        Self::from_trusted_root(root)
    }

    pub(crate) fn bootstrap_home(&self) -> OpenedBootstrapHomeV1<'_> {
        OpenedBootstrapHomeV1 { root: &self.root }
    }

    pub(crate) fn classify(&self) -> BootstrapClassificationV1 {
        store::classify_opened(&self.root)
    }

    pub(crate) fn bootstrap(&self) -> Result<StateRootV1, AuthorityFacadeError> {
        store::bootstrap_opened(&self.root).map_err(store_error)
    }

    pub(crate) fn read_root(&self) -> Result<StateRootV1, AuthorityFacadeError> {
        store::read_opened_root(&self.root).map_err(store_error)
    }

    pub(crate) fn read_preserved_start_root_v2(&self) -> Result<StateRootV2, AuthorityFacadeError> {
        let root = store::read_opened_root_v2_or_v3(&self.root).map_err(store_error)?;
        match root {
            VersionedStateRoot::V2(root) => Ok(root),
            VersionedStateRoot::V3(root) => Ok(root.preserved_v2_view()),
            VersionedStateRoot::V1(_) => Err(AuthorityFacadeError(
                "current authority requires strict StateRootV2 or StateRootV3".into(),
            )),
        }
    }

    pub(crate) fn prepare_typed_object(
        &self,
        expected_root_revision: u64,
        reference: &AuthorityObjectRefV1,
        bytes: &[u8],
        context: Option<&ObjectVerificationContextV1>,
    ) -> Result<ObjectPublicationOutcomeV1, AuthorityFacadeError> {
        store::prepare_typed_object_opened(
            &self.root,
            expected_root_revision,
            reference,
            bytes,
            context,
        )
        .map_err(store_error)
    }

    pub(crate) fn compare_and_swap_root(
        &self,
        expected: &ExpectedRevisionsV1,
        proposed: &StateRootV1,
    ) -> Result<TransactionCommitOutcomeV1, AuthorityFacadeError> {
        let current = self.read_root()?;
        if current.session_namespace_map != proposed.session_namespace_map
            || current.transition_intent_map != proposed.transition_intent_map
            || current.issuer_request_index != proposed.issuer_request_index
            || current.application_journal != proposed.application_journal
        {
            return Err(AuthorityFacadeError(
                "generic facade CAS cannot mutate authority or application history".into(),
            ));
        }
        if expected.authority.is_some() {
            return Err(AuthorityFacadeError(
                "generic facade CAS cannot consume an authority transition precondition".into(),
            ));
        }
        validate_current_authority_proofs(&current)?;
        validate_current_authority_proofs(proposed)?;
        store::compare_and_swap_opened_root_exact_current(&self.root, &current, expected, proposed)
            .map_err(store_error)
    }

    pub(crate) fn resolve_exact(
        &self,
        orchestration_session_id: &str,
        expected: Option<&AuthorityObservationV1>,
    ) -> Result<ResolvedSessionAuthorityV1, AuthorityFacadeError> {
        if orchestration_session_id.is_empty() {
            return Err(AuthorityFacadeError(
                "orchestration session ID must be exact and non-empty".into(),
            ));
        }
        let root = self.read_root()?;
        let record = root
            .session_namespace_map
            .get(orchestration_session_id)
            .ok_or_else(|| AuthorityFacadeError("exact authority session was not found".into()))?;
        let SessionNamespaceRecordV1::Authority(authority) = record else {
            return Err(AuthorityFacadeError(
                "exact session namespace record is not durable authority".into(),
            ));
        };
        let authority = authority.as_ref().clone();
        let authority_record_commitment = canonical_commitment(&authority_hash_input(&authority))?;
        let persisted_commitment = exact_current_authority_proof(
            &root,
            &authority.orchestration_session_id,
            authority.authority_revision,
        )?;
        if persisted_commitment != &authority_record_commitment {
            return Err(AuthorityFacadeError(
                "durable application proof does not commit to current authority".into(),
            ));
        }
        let authoritative_lineage_commitment =
            canonical_commitment(&AuthoritativeLineageHashInputV1 {
                schema_version: 1,
                orchestration_session_id: authority.orchestration_session_id.clone(),
                participant_ids: authority.authoritative_participant_lineage.clone(),
            })?;
        let resolved = ResolvedSessionAuthorityV1 {
            root_revision: root.root_revision,
            authority,
            authority_record_commitment,
            authoritative_lineage_commitment,
            authority_store_id: root.authority_store_id,
            bootstrap_home: root.bootstrap_home,
        };
        if expected.is_some_and(|value| value != &resolved.observation()) {
            return Err(AuthorityFacadeError(
                "stale or mismatched exact authority observation".into(),
            ));
        }
        Ok(resolved)
    }

    pub(crate) fn resolve_current_exact(
        &self,
        orchestration_session_id: &str,
        expected: Option<&AuthorityObservationV1>,
    ) -> Result<ResolvedCurrentAuthorityV1, AuthorityFacadeError> {
        if orchestration_session_id.is_empty() {
            return Err(AuthorityFacadeError(
                "orchestration session ID must be exact and non-empty".into(),
            ));
        }
        let root = store::read_opened_root_v2_or_v3(&self.root).map_err(store_error)?;
        match root {
            VersionedStateRoot::V2(root) => self.resolve_current_exact_from_preserved_start_root(
                &root,
                orchestration_session_id,
                expected,
            ),
            VersionedStateRoot::V3(root) => {
                let preserved = root.preserved_v2_view();
                self.resolve_current_exact_from_preserved_start_root(
                    &preserved,
                    orchestration_session_id,
                    expected,
                )
                .or_else(|_| {
                    self.resolve_current_exact_from_v3_root(
                        &root,
                        orchestration_session_id,
                        expected,
                    )
                })
            }
            VersionedStateRoot::V1(_) => Err(AuthorityFacadeError(
                "current authority requires strict StateRootV2 or StateRootV3".into(),
            )),
        }
    }

    pub(crate) fn resolve_exact_at_revision(
        &self,
        orchestration_session_id: &str,
        authority_revision: u64,
    ) -> Result<ResolvedSessionAuthorityV1, AuthorityFacadeError> {
        if orchestration_session_id.is_empty() || authority_revision == 0 {
            return Err(AuthorityFacadeError(
                "historical authority identity must be exact and non-empty".into(),
            ));
        }
        let root = store::read_opened_root_v2_or_v3(&self.root).map_err(store_error)?;
        match root {
            VersionedStateRoot::V3(root) => exact_v3_authority_history_from_root(
                self.trusted_root(),
                &root,
                orchestration_session_id,
            )?
            .remove(&authority_revision)
            .ok_or_else(|| {
                AuthorityFacadeError(
                    "historical authority revision is absent from the exact typed chain".into(),
                )
            }),
            VersionedStateRoot::V2(root) => {
                let Some(SessionNamespaceRecordV1::Authority(current)) =
                    root.session_namespace_map.get(orchestration_session_id)
                else {
                    return Err(AuthorityFacadeError(
                        "historical authority session is absent".into(),
                    ));
                };
                if current.authority_revision != authority_revision {
                    return Err(AuthorityFacadeError(
                        "historical V2 authority requires the retained ancestry resolver".into(),
                    ));
                }
                resolved_session_authority(
                    root.root_revision,
                    &root.authority_store_id,
                    &root.bootstrap_home,
                    current.as_ref().clone(),
                )
            }
            VersionedStateRoot::V1(_) => Err(AuthorityFacadeError(
                "historical authority requires strict StateRootV2 or StateRootV3".into(),
            )),
        }
    }

    pub(crate) fn resolve_exact_typed_history(
        &self,
        orchestration_session_id: &str,
    ) -> Result<Option<BTreeMap<u64, ResolvedSessionAuthorityV1>>, AuthorityFacadeError> {
        if orchestration_session_id.is_empty() {
            return Err(AuthorityFacadeError(
                "historical authority session identity is empty".into(),
            ));
        }
        match store::read_opened_root_v2_or_v3(&self.root).map_err(store_error)? {
            VersionedStateRoot::V3(root) => exact_v3_authority_history_from_root(
                self.trusted_root(),
                &root,
                orchestration_session_id,
            )
            .map(Some),
            VersionedStateRoot::V2(_) => Ok(None),
            VersionedStateRoot::V1(_) => Err(AuthorityFacadeError(
                "historical authority requires strict StateRootV2 or StateRootV3".into(),
            )),
        }
    }

    fn resolve_current_exact_from_preserved_start_root(
        &self,
        root: &StateRootV2,
        orchestration_session_id: &str,
        expected: Option<&AuthorityObservationV1>,
    ) -> Result<ResolvedCurrentAuthorityV1, AuthorityFacadeError> {
        let Some(SessionNamespaceRecordV1::Authority(authority)) =
            root.session_namespace_map.get(orchestration_session_id)
        else {
            return Err(AuthorityFacadeError(
                "exact session namespace record is not current durable authority".into(),
            ));
        };
        let authority = authority.as_ref().clone();
        let DurableSessionAuthorityOriginV1::StartIntent {
            intent_id,
            issuer_request_id,
            payload_commitment,
        } = &authority.origin;
        let intent = root
            .transition_intent_map
            .get(intent_id)
            .ok_or_else(|| AuthorityFacadeError("current authority has no Start intent".into()))?;
        if intent.issuer_request_id != *issuer_request_id
            || intent.payload_commitment != *payload_commitment
            || intent.orchestration_session_id != orchestration_session_id
        {
            return Err(AuthorityFacadeError(
                "current authority origin conflicts with its Start intent".into(),
            ));
        }
        let (
            claim_id,
            authority_revision_after,
            active_authoritative_participant_id,
            applied_at,
            authority_record_commitment,
        ) = match &intent.state {
            HostSessionTransitionIntentStateV2::Applied {
                claim_id,
                authority_revision_before: None,
                authority_revision_after,
                active_authoritative_participant_id,
                resulting_posture: HostSessionPostureV1::ActiveAttached,
                authority_record_commitment,
                startup_ownership,
                post_turn,
                applied_at,
                ..
            } if matches!(
                startup_ownership.as_ref(),
                HostSessionStartupOwnershipApplicationV1::Pending {
                    expected_run_id,
                    expected_authority_revision,
                    expected_active_authoritative_participant_id,
                } if expected_run_id == &intent.run_id
                    && expected_authority_revision == authority_revision_after
                    && expected_active_authoritative_participant_id
                        == active_authoritative_participant_id
            ) && post_turn.as_ref() == &HostSessionPostTurnApplicationV1::NotApplicable =>
            {
                (
                    claim_id,
                    *authority_revision_after,
                    active_authoritative_participant_id,
                    applied_at,
                    authority_record_commitment,
                )
            }
            _ => {
                return Err(AuthorityFacadeError(
                    "current authority is not a complete applied A1.2a Start".into(),
                ))
            }
        };
        if authority_revision_after != 1
            || active_authoritative_participant_id != &intent.target_authoritative_participant_id
        {
            return Err(AuthorityFacadeError(
                "applied Start authority identity is inconsistent".into(),
            ));
        }
        let expected_claim_revision = intent
            .intent_revision
            .checked_sub(1)
            .ok_or_else(|| AuthorityFacadeError("applied Start revision underflow".into()))?;
        verify_applied_start(
            self,
            root,
            intent,
            &ApplyHostSessionTransitionRequestV1 {
                intent_id: intent.intent_id.clone(),
                issuer_request_id: intent.issuer_request_id.clone(),
                payload_commitment: intent.payload_commitment.clone(),
                expected_intent_revision: expected_claim_revision,
                claim_id: claim_id.clone(),
                expected_claim_revision,
            },
        )
        .map_err(|error| AuthorityFacadeError(error.to_string()))?;
        let initial_authority = DurableSessionAuthorityV1 {
            schema_version: authority.schema_version,
            orchestration_session_id: authority.orchestration_session_id.clone(),
            shell_trace_session_id: authority.shell_trace_session_id.clone(),
            authority_revision: authority_revision_after,
            origin: authority.origin.clone(),
            authoritative_participant_lineage: intent.resulting_authoritative_lineage.clone(),
            active_authoritative_participant_id: Some(
                intent.target_authoritative_participant_id.clone(),
            ),
            workspace_binding: authority.workspace_binding.clone(),
            world_binding: authority.world_binding.clone(),
            host_attach_contract_ref: authority.host_attach_contract_ref.clone(),
            retained_worker_refs: Vec::new(),
            internal_resume_handle_refs: Vec::new(),
            lifecycle_posture: HostSessionPostureV1::ActiveAttached,
            current_policy_ref: authority.current_policy_ref.clone(),
            current_policy_revision: authority.current_policy_revision.clone(),
            updated_at: applied_at.clone(),
        };
        let initial_commitment = canonical_commitment(&authority_hash_input(&initial_authority))?;
        if initial_commitment != *authority_record_commitment {
            return Err(AuthorityFacadeError(
                "applied Start authority proof is inconsistent".into(),
            ));
        }
        verify_retained_registration_descendant_v2(
            root,
            &initial_authority,
            &authority,
            &initial_commitment,
        )?;

        let descriptor_bytes = store::read_typed_object_v2_or_v3_opened(
            &self.root,
            root.root_revision,
            &intent.descriptor_ref,
            None,
        )
        .map_err(store_error)?;
        let descriptor: AgentDescriptorHashInputV1 = canonical_json::from_slice(&descriptor_bytes)
            .map_err(|error| AuthorityFacadeError(error.to_string()))?;
        let attach_bytes = store::read_typed_object_v2_or_v3_opened(
            &self.root,
            root.root_revision,
            &intent.host_attach_contract_ref,
            None,
        )
        .map_err(store_error)?;
        let attach: HostAttachContractHashInputV1 = canonical_json::from_slice(&attach_bytes)
            .map_err(|error| AuthorityFacadeError(error.to_string()))?;
        let policy_bytes = store::read_typed_object_v2_or_v3_opened(
            &self.root,
            root.root_revision,
            &attach.contract.policy_ref,
            None,
        )
        .map_err(store_error)?;
        let policy: PolicyObjectHashInputV1 = canonical_json::from_slice(&policy_bytes)
            .map_err(|error| AuthorityFacadeError(error.to_string()))?;
        if descriptor.schema_version != 1
            || descriptor.descriptor.schema_version != 1
            || attach.schema_version != 1
            || attach.contract.schema_version != 1
            || policy.schema_version != 1
            || attach.contract.descriptor_ref != intent.descriptor_ref
            || attach.contract.backend_id != descriptor.descriptor.backend_id
            || attach.contract.execution_scope != descriptor.descriptor.execution_scope
            || attach.contract.protocol != descriptor.descriptor.protocol
            || (descriptor.descriptor.execution_scope
                == super::schema::AgentExecutionScopeV1::World
                && authority.world_binding.is_none())
            || authority.host_attach_contract_ref.as_ref() != Some(&intent.host_attach_contract_ref)
            || authority.current_policy_ref.as_ref() != Some(&attach.contract.policy_ref)
            || authority.current_policy_revision.as_ref() != Some(&policy.policy_revision)
        {
            return Err(AuthorityFacadeError(
                "applied Start descriptor, attach contract, or policy truth is inconsistent".into(),
            ));
        }

        let authority_record_commitment = canonical_commitment(&authority_hash_input(&authority))?;
        let authoritative_lineage_commitment =
            canonical_commitment(&AuthoritativeLineageHashInputV1 {
                schema_version: 1,
                orchestration_session_id: authority.orchestration_session_id.clone(),
                participant_ids: authority.authoritative_participant_lineage.clone(),
            })?;
        let observation = AuthorityObservationV1 {
            authority_store_id: root.authority_store_id.clone(),
            bootstrap_home: root.bootstrap_home.clone(),
            orchestration_session_id: authority.orchestration_session_id.clone(),
            root_revision: root.root_revision,
            authority_revision: authority.authority_revision,
            authority_record_commitment,
            authoritative_lineage_commitment,
        };
        if expected.is_some_and(|value| value != &observation) {
            return Err(AuthorityFacadeError(
                "stale or mismatched exact current-authority observation".into(),
            ));
        }
        let active_participant_id = authority
            .active_authoritative_participant_id
            .clone()
            .ok_or_else(|| AuthorityFacadeError("current authority has no active caller".into()))?;
        if active_participant_id != intent.target_authoritative_participant_id
            || !authority
                .authoritative_participant_lineage
                .contains(&active_participant_id)
        {
            return Err(AuthorityFacadeError(
                "current authority caller is not the applied Start orchestrator".into(),
            ));
        }
        let bound_state_store =
            super::super::state_store::AgentRuntimeStateStore::for_bootstrap_home(
                &self.bootstrap_home(),
            )
            .map_err(|error| AuthorityFacadeError(error.to_string()))?;
        if bound_state_store.bootstrap_home_identity() != &root.bootstrap_home {
            return Err(AuthorityFacadeError(
                "bound StateStore does not match current authority bootstrap home".into(),
            ));
        }
        Ok(ResolvedCurrentAuthorityV1 {
            observation,
            authority,
            caller: ResolvedAuthorityCallerV1 {
                participant_id: active_participant_id,
                role: AuthorityParticipantRoleV1::Orchestrator,
                descriptor_ref: intent.descriptor_ref.clone(),
                descriptor: descriptor.descriptor,
            },
            host_attach_contract: attach.contract,
            current_policy: policy,
            bound_state_store,
        })
    }

    fn resolve_current_exact_from_v3_root(
        &self,
        root: &StateRootV3,
        orchestration_session_id: &str,
        expected: Option<&AuthorityObservationV1>,
    ) -> Result<ResolvedCurrentAuthorityV1, AuthorityFacadeError> {
        let Some(SessionNamespaceRecordV1::Authority(authority)) =
            root.session_namespace_map.get(orchestration_session_id)
        else {
            return Err(AuthorityFacadeError(
                "exact session namespace record is not current durable authority".into(),
            ));
        };
        let authority = authority.as_ref().clone();
        let authority_record_commitment = canonical_commitment(&authority_hash_input(&authority))?;
        let persisted_commitment = exact_current_authority_proof_v3(
            self,
            root,
            &authority.orchestration_session_id,
            authority.authority_revision,
        )?;
        if persisted_commitment != authority_record_commitment {
            return Err(AuthorityFacadeError(
                "durable V3 application proof does not commit to current authority".into(),
            ));
        }
        let authoritative_lineage_commitment =
            canonical_commitment(&AuthoritativeLineageHashInputV1 {
                schema_version: 1,
                orchestration_session_id: authority.orchestration_session_id.clone(),
                participant_ids: authority.authoritative_participant_lineage.clone(),
            })?;
        let attach_ref = authority.host_attach_contract_ref.as_ref().ok_or_else(|| {
            AuthorityFacadeError("current authority has no attach contract".into())
        })?;
        let attach_bytes = store::read_typed_object_v2_or_v3_opened(
            &self.root,
            root.root_revision,
            attach_ref,
            None,
        )
        .map_err(store_error)?;
        let attach: HostAttachContractHashInputV1 = canonical_json::from_slice(&attach_bytes)
            .map_err(|error| AuthorityFacadeError(error.to_string()))?;
        let descriptor_bytes = store::read_typed_object_v2_or_v3_opened(
            &self.root,
            root.root_revision,
            &attach.contract.descriptor_ref,
            None,
        )
        .map_err(store_error)?;
        let descriptor: AgentDescriptorHashInputV1 = canonical_json::from_slice(&descriptor_bytes)
            .map_err(|error| AuthorityFacadeError(error.to_string()))?;
        let current_policy_ref = authority
            .current_policy_ref
            .as_ref()
            .ok_or_else(|| AuthorityFacadeError("current authority has no policy ref".into()))?;
        if current_policy_ref != &attach.contract.policy_ref {
            return Err(AuthorityFacadeError(
                "current authority policy ref conflicts with its attach contract".into(),
            ));
        }
        let policy_bytes = store::read_typed_object_v2_or_v3_opened(
            &self.root,
            root.root_revision,
            current_policy_ref,
            None,
        )
        .map_err(store_error)?;
        let policy: PolicyObjectHashInputV1 = canonical_json::from_slice(&policy_bytes)
            .map_err(|error| AuthorityFacadeError(error.to_string()))?;
        if descriptor.schema_version != 1
            || descriptor.descriptor.schema_version != 1
            || attach.schema_version != 1
            || attach.contract.schema_version != 1
            || policy.schema_version != 1
            || authority.host_attach_contract_ref.as_ref() != Some(attach_ref)
            || authority.current_policy_ref.as_ref() != Some(&attach.contract.policy_ref)
            || authority.current_policy_revision.as_ref() != Some(&policy.policy_revision)
            || attach.contract.backend_id != descriptor.descriptor.backend_id
            || attach.contract.execution_scope != descriptor.descriptor.execution_scope
            || attach.contract.protocol != descriptor.descriptor.protocol
            || (descriptor.descriptor.execution_scope
                == super::schema::AgentExecutionScopeV1::World
                && authority.world_binding.is_none())
            || attach
                .contract
                .continuity_resume_handle_ref
                .as_ref()
                .is_some_and(|reference| {
                    !authority
                        .internal_resume_handle_refs
                        .iter()
                        .any(|current| current == reference)
                })
        {
            return Err(AuthorityFacadeError(
                "applied V3 descriptor, attach contract, or policy truth is inconsistent".into(),
            ));
        }
        let observation = AuthorityObservationV1 {
            authority_store_id: root.authority_store_id.clone(),
            bootstrap_home: root.bootstrap_home.clone(),
            orchestration_session_id: authority.orchestration_session_id.clone(),
            root_revision: root.root_revision,
            authority_revision: authority.authority_revision,
            authority_record_commitment,
            authoritative_lineage_commitment,
        };
        if expected.is_some_and(|value| value != &observation) {
            return Err(AuthorityFacadeError(
                "stale or mismatched exact current-authority observation".into(),
            ));
        }
        let active_participant_id = authority
            .active_authoritative_participant_id
            .clone()
            .ok_or_else(|| AuthorityFacadeError("current authority has no active caller".into()))?;
        if !authority
            .authoritative_participant_lineage
            .contains(&active_participant_id)
        {
            return Err(AuthorityFacadeError(
                "current authority caller is not part of its authoritative lineage".into(),
            ));
        }
        let bound_state_store =
            super::super::state_store::AgentRuntimeStateStore::for_bootstrap_home(
                &self.bootstrap_home(),
            )
            .map_err(|error| AuthorityFacadeError(error.to_string()))?;
        if bound_state_store.bootstrap_home_identity() != &root.bootstrap_home {
            return Err(AuthorityFacadeError(
                "bound StateStore does not match current authority bootstrap home".into(),
            ));
        }
        let caller_descriptor_ref = attach.contract.descriptor_ref.clone();
        let host_attach_contract = attach.contract;
        Ok(ResolvedCurrentAuthorityV1 {
            observation,
            authority,
            caller: ResolvedAuthorityCallerV1 {
                participant_id: active_participant_id,
                role: AuthorityParticipantRoleV1::Orchestrator,
                descriptor_ref: caller_descriptor_ref,
                descriptor: descriptor.descriptor,
            },
            host_attach_contract,
            current_policy: policy,
            bound_state_store,
        })
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn reserve_retained_worker_registration(
        &self,
        registration_request_id: &str,
        orchestration_session_id: &str,
        expected_authority: &RetainedWorkerAuthorityPreconditionV1,
        retained_participant_id: &str,
        descriptor_bytes: Vec<u8>,
        resume_handle_bytes: Vec<u8>,
        build_worker: impl Fn(
            &AuthorityObjectRefV1,
            &AuthorityObjectRefV1,
            &AuthorityObjectRefV1,
            &WorldBindingV1,
        ) -> Result<Vec<u8>, &'static str>,
    ) -> Result<ReservedRetainedWorkerRegistrationV1, AuthorityFacadeError> {
        let input = retained_reservation_input(
            registration_request_id,
            orchestration_session_id,
            expected_authority,
            retained_participant_id,
            descriptor_bytes,
            resume_handle_bytes,
        )?;
        store::reserve_retained_worker_registration_opened(&self.root, &input, build_worker)
            .map(reserved_registration)
            .map_err(store_error)
    }

    pub(crate) fn publish_reserved_retained_object(
        &self,
        reserved: &ReservedRetainedWorkerRegistrationV1,
        reference: &AuthorityObjectRefV1,
        bytes: &[u8],
    ) -> Result<ObjectPublicationOutcomeV1, AuthorityFacadeError> {
        store::publish_reserved_retained_object_opened(
            &self.root,
            &store_reservation(reserved),
            reference,
            bytes,
        )
        .map_err(store_error)
    }

    pub(crate) fn apply_reserved_retained_worker_registration(
        &self,
        reserved: &ReservedRetainedWorkerRegistrationV1,
    ) -> Result<AppliedRetainedWorkerRegistrationV1, AuthorityFacadeError> {
        store::apply_reserved_retained_worker_registration_opened(
            &self.root,
            &store_reservation(reserved),
        )
        .map(|applied| AppliedRetainedWorkerRegistrationV1 {
            registration: applied.registration,
            joined: applied.joined,
        })
        .map_err(store_error)
    }

    pub(crate) fn read_authority_object_v2_at(
        &self,
        expected_root_revision: u64,
        reference: &AuthorityObjectRefV1,
    ) -> Result<Vec<u8>, AuthorityFacadeError> {
        store::read_typed_object_v2_or_v3_opened(
            &self.root,
            expected_root_revision,
            reference,
            None,
        )
        .map_err(store_error)
    }

    pub(crate) fn read_successor_sensitive_object_v3_at(
        &self,
        expected_root_revision: u64,
        intent: &HostSessionTransitionIntentV3,
        reference: &AuthorityObjectRefV1,
    ) -> Result<Vec<u8>, AuthorityFacadeError> {
        let context = ObjectVerificationContextV1 {
            intent_id: intent.intent_id.clone(),
            run_id: intent.run_id.clone(),
            parent_intent: None,
        };
        store::read_typed_object_v2_or_v3_opened(
            &self.root,
            expected_root_revision,
            reference,
            Some(&context),
        )
        .map_err(store_error)
    }

    #[cfg(test)]
    pub(crate) fn apply_reserved_retained_worker_registration_with_crash_point(
        &self,
        reserved: &ReservedRetainedWorkerRegistrationV1,
        crash_point: RetainedApplicationCrashPointV1,
    ) -> Result<AppliedRetainedWorkerRegistrationV1, AuthorityFacadeError> {
        store::apply_reserved_retained_worker_registration_with_crash_point_opened(
            &self.root,
            &store_reservation(reserved),
            match crash_point {
                RetainedApplicationCrashPointV1::BeforeRootPublication => {
                    store::RetainedApplicationCrashPointV1::BeforeRootPublication
                }
                RetainedApplicationCrashPointV1::AfterRootPublication => {
                    store::RetainedApplicationCrashPointV1::AfterRootPublication
                }
            },
        )
        .map(|applied| AppliedRetainedWorkerRegistrationV1 {
            registration: applied.registration,
            joined: applied.joined,
        })
        .map_err(store_error)
    }

    #[cfg(test)]
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn reserve_retained_worker_registration_at(
        &self,
        registration_request_id: &str,
        orchestration_session_id: &str,
        expected_authority: &RetainedWorkerAuthorityPreconditionV1,
        retained_participant_id: &str,
        descriptor_bytes: Vec<u8>,
        resume_handle_bytes: Vec<u8>,
        registered_at: TimestampV1,
        crash_point: Option<RetainedReservationCrashPointV1>,
        build_worker: impl Fn(
            &AuthorityObjectRefV1,
            &AuthorityObjectRefV1,
            &AuthorityObjectRefV1,
            &WorldBindingV1,
        ) -> Result<Vec<u8>, &'static str>,
    ) -> Result<ReservedRetainedWorkerRegistrationV1, AuthorityFacadeError> {
        let input = retained_reservation_input(
            registration_request_id,
            orchestration_session_id,
            expected_authority,
            retained_participant_id,
            descriptor_bytes,
            resume_handle_bytes,
        )?;
        store::reserve_retained_worker_registration_at_opened(
            &self.root,
            &input,
            registered_at,
            crash_point.map(|point| match point {
                RetainedReservationCrashPointV1::BeforeRootPublication => {
                    store::RetainedReservationCrashPointV1::BeforeRootPublication
                }
                RetainedReservationCrashPointV1::AfterRootPublication => {
                    store::RetainedReservationCrashPointV1::AfterRootPublication
                }
            }),
            build_worker,
        )
        .map(reserved_registration)
        .map_err(store_error)
    }
}

fn resolved_session_authority(
    root_revision: u64,
    authority_store_id: &str,
    bootstrap_home: &CanonicalDirectoryV1,
    authority: DurableSessionAuthorityV1,
) -> Result<ResolvedSessionAuthorityV1, AuthorityFacadeError> {
    let authority_record_commitment = canonical_commitment(&authority_hash_input(&authority))?;
    let authoritative_lineage_commitment =
        canonical_commitment(&AuthoritativeLineageHashInputV1 {
            schema_version: 1,
            orchestration_session_id: authority.orchestration_session_id.clone(),
            participant_ids: authority.authoritative_participant_lineage.clone(),
        })?;
    Ok(ResolvedSessionAuthorityV1 {
        root_revision,
        authority,
        authority_record_commitment,
        authoritative_lineage_commitment,
        authority_store_id: authority_store_id.to_string(),
        bootstrap_home: bootstrap_home.clone(),
    })
}

fn offer_exact_authority_edge(
    candidate: &mut Option<(String, ResolvedSessionAuthorityV1)>,
    key: String,
    state: ResolvedSessionAuthorityV1,
    expected_commitment: Option<&AuthorityObjectCommitmentV1>,
) -> Result<(), AuthorityFacadeError> {
    if expected_commitment.is_some_and(|expected| expected != &state.authority_record_commitment) {
        return Err(AuthorityFacadeError(
            "typed authority edge produced a mismatched authority commitment".into(),
        ));
    }
    if candidate.replace((key, state)).is_some() {
        return Err(AuthorityFacadeError(
            "authority ancestry has multiple authenticated edges at one revision".into(),
        ));
    }
    Ok(())
}

fn exact_start_continuation_handles<F>(
    root: &StateRootV3,
    current: &DurableSessionAuthorityV1,
    read_object: &mut F,
) -> Result<Vec<(AuthorityObjectRefV1, StartContinuationHandleHashInputV2)>, AuthorityFacadeError>
where
    F: FnMut(&AuthorityObjectRefV1) -> Result<Vec<u8>, String>,
{
    let transactions = root
        .start_transaction_map
        .values()
        .filter(|transaction| {
            transaction.orchestration_session_id == current.orchestration_session_id
        })
        .collect::<Vec<_>>();
    let transaction = match transactions.as_slice() {
        [] => None,
        [transaction] => Some(*transaction),
        _ => {
            return Err(AuthorityFacadeError(
                "authority ancestry has multiple Start transactions for one session".into(),
            ))
        }
    };
    let mut handles = Vec::new();
    for reference in current
        .internal_resume_handle_refs
        .iter()
        .filter(|reference| {
            reference.object_kind == super::schema::AuthorityObjectKindV1::ResumeHandle
                && reference.schema_version == 2
        })
    {
        let bytes = read_object(reference).map_err(AuthorityFacadeError)?;
        let handle: StartContinuationHandleHashInputV2 = canonical_json::from_slice(&bytes)
            .map_err(|error| AuthorityFacadeError(error.to_string()))?;
        if handle.schema_version != 2
            || handle.authority_store_id != root.authority_store_id
            || handle.orchestration_session_id != current.orchestration_session_id
            || handle.authority_revision_before == 0
            || handle.authority_revision_after
                != handle
                    .authority_revision_before
                    .checked_add(1)
                    .ok_or_else(|| {
                        AuthorityFacadeError(
                            "Start continuation authority revision overflow".into(),
                        )
                    })?
        {
            return Err(AuthorityFacadeError(
                "Start continuation handle has mismatched authority identity".into(),
            ));
        }
        let transaction = transaction.ok_or_else(|| {
            AuthorityFacadeError("Start continuation handle has no exact Start transaction".into())
        })?;
        if handle.participant_id != transaction.authoritative_participant_id
            || handle.backend_id != transaction.backend_id
            || handle.protocol != transaction.protocol
            || handle.start_intent_id != transaction.start_intent_id
            || handle.start_issuer_request_id != transaction.start_issuer_request_id
            || handle.start_payload_commitment != transaction.start_payload_commitment
            || handle.start_application_result_ref != transaction.start_application_result_ref
            || handle.start_run_id != transaction.start_run_id
        {
            return Err(AuthorityFacadeError(
                "Start continuation handle conflicts with its exact transaction".into(),
            ));
        }
        handles.push((reference.clone(), handle));
    }
    handles.sort_by(|left, right| {
        left.1
            .authority_revision_before
            .cmp(&right.1.authority_revision_before)
            .then_with(|| left.0.ref_id.cmp(&right.0.ref_id))
    });

    match (
        transaction.map(|transaction| &transaction.state),
        handles.as_slice(),
    ) {
        (
            None
            | Some(
                StartTransactionStateV1::PromptNotSubmitted
                | StartTransactionStateV1::PromptSubmissionNoReplayBarrier { .. }
                | StartTransactionStateV1::PromptSubmissionIndeterminate { .. },
            ),
            [],
        ) => {}
        (
            Some(StartTransactionStateV1::ContinuationRegistered {
                registration_ref,
                authority_revision_after,
                registered_at,
            }),
            [(reference, handle)],
        ) if reference == registration_ref
            && handle.authority_revision_after == *authority_revision_after
            && matches!(
                &handle.state,
                StartContinuationHandleStateV2::Registered { observed_at, .. }
                    if observed_at == registered_at
            ) => {}
        (
            Some(
                StartTransactionStateV1::TurnSettledAwaitingResponse {
                    settlement_ref,
                    authority_revision_after,
                    resulting_posture,
                    completed_at,
                }
                | StartTransactionStateV1::PublicResponseDelivered {
                    settlement_ref,
                    authority_revision_after,
                    resulting_posture,
                    completed_at,
                    ..
                },
            ),
            [(registration_ref, registration), (settlement_ref_actual, settlement)],
        ) if settlement_ref_actual == settlement_ref
            && settlement.authority_revision_after == *authority_revision_after
            && matches!(
                &registration.state,
                StartContinuationHandleStateV2::Registered { .. }
            )
            && matches!(
                &settlement.state,
                StartContinuationHandleStateV2::Settled {
                    registered_resume_handle_ref,
                    completion_kind,
                    obligation_snapshot,
                    completed_at: handle_completed_at,
                    ..
                } if registered_resume_handle_ref.as_ref() == registration_ref
                    && handle_completed_at == completed_at
                    && super::start_continuity::committed_settlement_posture(
                        completion_kind,
                        obligation_snapshot,
                    )
                    .is_ok_and(|posture| posture == *resulting_posture)
            ) => {}
        _ => {
            return Err(AuthorityFacadeError(
                "Start transaction and continuation handle ancestry disagree".into(),
            ))
        }
    }
    Ok(handles)
}

pub(super) fn exact_v3_authority_history_from_root(
    trusted_root: &TrustedAuthorityRoot,
    root: &StateRootV3,
    orchestration_session_id: &str,
) -> Result<BTreeMap<u64, ResolvedSessionAuthorityV1>, AuthorityFacadeError> {
    exact_v3_authority_history_with_reader(root, orchestration_session_id, |reference| {
        store::read_typed_object_v2_or_v3_opened(trusted_root, root.root_revision, reference, None)
            .map_err(|error| error.to_string())
    })
}

pub(super) fn validate_v3_schema_with_external_exact_authority_history(
    root: &StateRootV3,
) -> Result<(), AuthorityFacadeError> {
    match root.validate() {
        Ok(()) => Ok(()),
        Err(error)
            if error.to_string()
                == "V3 preserved V2 retained authority ancestry is not uniquely contiguous" =>
        {
            Ok(())
        }
        Err(error)
            if error.to_string() == "Start transaction state conflicts with current authority" =>
        {
            // A registered Start continuation may legitimately acquire an interleaved
            // retained-worker edge before Start settles. The ordinary V3 schema view
            // predates that composition and compares the unfinished transaction only
            // with the current authority. Validate every other schema invariant on a
            // read-only projection; the caller must authenticate the original Start
            // transaction and every authority edge with the exact-history verifier.
            let mut projection = root.clone();
            projection.start_transaction_map.clear();
            match projection.validate() {
                Ok(()) => Ok(()),
                Err(projected_error)
                    if projected_error.to_string()
                        == "V3 preserved V2 retained authority ancestry is not uniquely contiguous" =>
                {
                    Ok(())
                }
                Err(_) => Err(AuthorityFacadeError(
                    "V3 semantic projection is invalid outside exact authority ancestry".into(),
                )),
            }
        }
        Err(_) => Err(AuthorityFacadeError(
            "V3 state-root semantics are invalid outside exact authority ancestry".into(),
        )),
    }
}

pub(super) fn exact_v3_authority_history_with_reader<F>(
    root: &StateRootV3,
    orchestration_session_id: &str,
    mut read_object: F,
) -> Result<BTreeMap<u64, ResolvedSessionAuthorityV1>, AuthorityFacadeError>
where
    F: FnMut(&AuthorityObjectRefV1) -> Result<Vec<u8>, String>,
{
    let Some(SessionNamespaceRecordV1::Authority(current)) =
        root.session_namespace_map.get(orchestration_session_id)
    else {
        return Err(AuthorityFacadeError(
            "exact V3 authority session is absent".into(),
        ));
    };
    let current = current.as_ref();
    let fork_allocations = root
        .fork_successor_allocation_map
        .values()
        .filter(|allocation| {
            allocation.request.target_orchestration_session_id == orchestration_session_id
        })
        .collect::<Vec<_>>();
    let (initial, origin_startup_terminal, origin_intent_id, continuation_handles) =
        match fork_allocations.as_slice() {
            [allocation] => {
                let source_attach_ref = allocation
                    .source_authority_before
                    .host_attach_contract_ref
                    .as_ref()
                    .ok_or_else(|| {
                        AuthorityFacadeError(
                            "fork successor source attach contract is absent".into(),
                        )
                    })?;
                let target_attach_ref = allocation
                    .target_authority
                    .host_attach_contract_ref
                    .as_ref()
                    .ok_or_else(|| {
                        AuthorityFacadeError(
                            "fork successor target attach contract is absent".into(),
                        )
                    })?;
                let source_attach: HostAttachContractHashInputV1 = canonical_json::from_slice(
                    &read_object(source_attach_ref).map_err(AuthorityFacadeError)?,
                )
                .map_err(|error| AuthorityFacadeError(error.to_string()))?;
                let target_attach: HostAttachContractHashInputV1 = canonical_json::from_slice(
                    &read_object(target_attach_ref).map_err(AuthorityFacadeError)?,
                )
                .map_err(|error| AuthorityFacadeError(error.to_string()))?;
                validate_fork_successor_attach_semantics(&source_attach, &target_attach).map_err(
                    |_| {
                        AuthorityFacadeError(
                            "fork successor attach capability or transformation is invalid".into(),
                        )
                    },
                )?;
                let initial = resolved_session_authority(
                    root.root_revision,
                    &root.authority_store_id,
                    &root.bootstrap_home,
                    allocation.target_authority.as_ref().clone(),
                )?;
                if initial.authority_record_commitment
                    != allocation.target_authority_record_commitment
                    || initial.authoritative_lineage_commitment
                        != allocation.target_authoritative_lineage_commitment
                {
                    return Err(AuthorityFacadeError(
                        "fork successor initial authority edge is inconsistent".into(),
                    ));
                }
                (initial, None, String::new(), Vec::new())
            }
            [] => {
                let DurableSessionAuthorityOriginV1::StartIntent {
                    intent_id,
                    issuer_request_id,
                    payload_commitment,
                } = &current.origin;
                let intent = root.transition_intent_map.get(intent_id).ok_or_else(|| {
                    AuthorityFacadeError(
                        "exact authority ancestry has no origin Start intent".into(),
                    )
                })?;
                let HostSessionTransitionIntentStateV2::Applied {
                    authority_revision_before: None,
                    authority_revision_after,
                    active_authoritative_participant_id,
                    resulting_posture,
                    authority_record_commitment,
                    startup_ownership,
                    applied_at,
                    ..
                } = &intent.state
                else {
                    return Err(AuthorityFacadeError(
                        "exact authority ancestry requires an applied origin Start".into(),
                    ));
                };
                let journal = root.application_journal.get(intent_id).ok_or_else(|| {
                    AuthorityFacadeError(
                        "exact authority ancestry has no origin Start journal".into(),
                    )
                })?;
                if intent.issuer_request_id != *issuer_request_id
                    || intent.payload_commitment != *payload_commitment
                    || intent.orchestration_session_id != orchestration_session_id
                    || *authority_revision_after != 1
                    || active_authoritative_participant_id
                        != &intent.target_authoritative_participant_id
                    || *resulting_posture != HostSessionPostureV1::ActiveAttached
                    || journal
                        .initial_application
                        .authority_revision_before
                        .is_some()
                    || journal.initial_application.authority_revision_after != 1
                    || journal.initial_application.authority_record_commitment
                        != *authority_record_commitment
                    || journal.initial_application.applied_at != *applied_at
                {
                    return Err(AuthorityFacadeError(
                        "origin Start authority edge is inconsistent".into(),
                    ));
                }
                let origin_startup_terminal = match (
                    startup_ownership.as_ref(),
                    journal.startup_terminal_application.as_ref(),
                ) {
                    (
                        HostSessionStartupOwnershipApplicationV1::Pending { .. }
                        | HostSessionStartupOwnershipApplicationV1::Accepted { .. },
                        None,
                    ) => None,
                    (
                        HostSessionStartupOwnershipApplicationV1::TerminalReconciled {
                            evidence_id,
                            result_ref,
                            authority_revision_before,
                            authority_revision_after,
                            resulting_posture,
                            reconciled_at,
                        },
                        Some(terminal),
                    ) if terminal.schema_version == 1
                        && terminal.evidence_id == *evidence_id
                        && terminal.startup_ownership_result_ref == *result_ref
                        && terminal.authority_revision_before == *authority_revision_before
                        && terminal.authority_revision_after == *authority_revision_after
                        && terminal.resulting_posture == *resulting_posture
                        && terminal.applied_at == *reconciled_at =>
                    {
                        Some(terminal)
                    }
                    _ => {
                        return Err(AuthorityFacadeError(
                            "origin Start startup-terminal edge is inconsistent".into(),
                        ))
                    }
                };
                let attach_bytes =
                    read_object(&intent.host_attach_contract_ref).map_err(AuthorityFacadeError)?;
                let attach: HostAttachContractHashInputV1 =
                    canonical_json::from_slice(&attach_bytes)
                        .map_err(|error| AuthorityFacadeError(error.to_string()))?;
                let policy_bytes =
                    read_object(&attach.contract.policy_ref).map_err(AuthorityFacadeError)?;
                let policy: PolicyObjectHashInputV1 = canonical_json::from_slice(&policy_bytes)
                    .map_err(|error| AuthorityFacadeError(error.to_string()))?;
                let initial = resolved_session_authority(
                    root.root_revision,
                    &root.authority_store_id,
                    &root.bootstrap_home,
                    DurableSessionAuthorityV1 {
                        schema_version: 1,
                        orchestration_session_id: orchestration_session_id.to_string(),
                        shell_trace_session_id: intent.shell_trace_session_id.clone(),
                        authority_revision: 1,
                        origin: current.origin.clone(),
                        authoritative_participant_lineage: intent
                            .resulting_authoritative_lineage
                            .clone(),
                        active_authoritative_participant_id: Some(
                            intent.target_authoritative_participant_id.clone(),
                        ),
                        workspace_binding: intent.workspace_binding.clone(),
                        world_binding: intent.world_binding.clone(),
                        host_attach_contract_ref: Some(intent.host_attach_contract_ref.clone()),
                        retained_worker_refs: Vec::new(),
                        internal_resume_handle_refs: Vec::new(),
                        lifecycle_posture: HostSessionPostureV1::ActiveAttached,
                        current_policy_ref: Some(attach.contract.policy_ref),
                        current_policy_revision: Some(policy.policy_revision),
                        updated_at: applied_at.clone(),
                    },
                )?;
                if initial.authority_record_commitment != *authority_record_commitment {
                    return Err(AuthorityFacadeError(
                        "origin Start authority commitment is inconsistent".into(),
                    ));
                }
                let continuation_handles =
                    exact_start_continuation_handles(root, current, &mut read_object)?;
                (
                    initial,
                    origin_startup_terminal,
                    intent_id.clone(),
                    continuation_handles,
                )
            }
            _ => {
                return Err(AuthorityFacadeError(
                    "multiple fork successor allocations claim one target session".into(),
                ))
            }
        };
    let retained_count = root
        .retained_worker_registration_journal
        .values()
        .filter(|registration| registration.orchestration_session_id == orchestration_session_id)
        .count();
    let successor_phase_count = root
        .successor_transition_intent_map
        .values()
        .filter(|successor| successor.orchestration_session_id == orchestration_session_id)
        .map(|successor| {
            if matches!(
                successor.state,
                HostSessionTransitionIntentStateV3::Applied { .. }
            ) {
                let journal = root.successor_application_journal.get(&successor.intent_id);
                1 + usize::from(
                    journal.is_some_and(|journal| journal.startup_terminal_application.is_some()),
                ) + usize::from(
                    journal.is_some_and(|journal| journal.post_turn_application.is_some()),
                )
            } else {
                0
            }
        })
        .sum::<usize>();
    let stop_phase_count = root
        .stop_transaction_map
        .values()
        .filter(|stop| {
            stop.orchestration_session_id == orchestration_session_id
                && matches!(stop.state, HostSessionStopIntentStateV1::Completed { .. })
        })
        .count();
    let expected_edge_count = usize::from(origin_startup_terminal.is_some())
        + continuation_handles.len()
        + retained_count
        + successor_phase_count
        + stop_phase_count;
    let mut consumed = BTreeSet::new();
    let mut history = BTreeMap::new();
    history.insert(1, initial);

    while consumed.len() < expected_edge_count {
        let latest = history
            .last_key_value()
            .map(|(_, state)| state.clone())
            .ok_or_else(|| AuthorityFacadeError("exact authority history is empty".into()))?;
        let next_revision = latest
            .authority
            .authority_revision
            .checked_add(1)
            .ok_or_else(|| AuthorityFacadeError("authority revision overflow".into()))?;
        let mut candidate = None;

        if let Some(terminal) = origin_startup_terminal {
            let key = format!("start-startup-terminal:{origin_intent_id}");
            if !consumed.contains(&key)
                && terminal.authority_revision_before == latest.authority.authority_revision
                && terminal.authority_record_commitment_before == latest.authority_record_commitment
            {
                if terminal.authority_revision_after != next_revision
                    || terminal.applied_at.as_str() < latest.authority.updated_at.as_str()
                {
                    return Err(AuthorityFacadeError(
                        "origin Start startup-terminal authority edge is stale or reordered".into(),
                    ));
                }
                let mut next = latest.authority.clone();
                next.authority_revision = terminal.authority_revision_after;
                next.lifecycle_posture = terminal.resulting_posture;
                next.updated_at = terminal.applied_at.clone();
                let state = resolved_session_authority(
                    root.root_revision,
                    &root.authority_store_id,
                    &root.bootstrap_home,
                    next,
                )?;
                offer_exact_authority_edge(
                    &mut candidate,
                    key,
                    state,
                    Some(&terminal.authority_record_commitment_after),
                )?;
            }
        }

        for (reference, handle) in &continuation_handles {
            let key = format!("start-continuation:{}", reference.ref_id);
            if consumed.contains(&key)
                || handle.authority_revision_before != latest.authority.authority_revision
                || handle.authority_record_commitment_before != latest.authority_record_commitment
            {
                continue;
            }
            let mut next = latest.authority.clone();
            next.authority_revision = handle.authority_revision_after;
            if next.authority_revision != next_revision
                || next.internal_resume_handle_refs.contains(reference)
            {
                return Err(AuthorityFacadeError(
                    "Start continuation authority edge is stale or duplicated".into(),
                ));
            }
            match &handle.state {
                StartContinuationHandleStateV2::Registered { observed_at, .. } => {
                    if observed_at.as_str() < latest.authority.updated_at.as_str()
                        || latest.authority.lifecycle_posture
                            != HostSessionPostureV1::ActiveAttached
                    {
                        return Err(AuthorityFacadeError(
                            "Start continuation registration ordering is invalid".into(),
                        ));
                    }
                    next.updated_at = observed_at.clone();
                }
                StartContinuationHandleStateV2::Settled {
                    registered_resume_handle_ref,
                    completion_kind,
                    obligation_snapshot,
                    completed_at,
                    ..
                } => {
                    if !latest
                        .authority
                        .internal_resume_handle_refs
                        .contains(registered_resume_handle_ref)
                        || completed_at.as_str() < latest.authority.updated_at.as_str()
                    {
                        return Err(AuthorityFacadeError(
                            "Start continuation settlement ordering is invalid".into(),
                        ));
                    }
                    next.lifecycle_posture = super::start_continuity::committed_settlement_posture(
                        completion_kind,
                        obligation_snapshot,
                    )
                    .map_err(|_| {
                        AuthorityFacadeError(
                            "Start continuation settlement posture is invalid".into(),
                        )
                    })?;
                    next.updated_at = completed_at.clone();
                }
            }
            next.internal_resume_handle_refs.push(reference.clone());
            let state = resolved_session_authority(
                root.root_revision,
                &root.authority_store_id,
                &root.bootstrap_home,
                next,
            )?;
            offer_exact_authority_edge(&mut candidate, key, state, None)?;
        }

        for registration in root.retained_worker_registration_journal.values() {
            let key = format!("retained:{}", registration.registration_id);
            if consumed.contains(&key)
                || registration.orchestration_session_id != orchestration_session_id
                || registration.authority_revision_before != latest.authority.authority_revision
                || registration.authority_record_commitment_before
                    != latest.authority_record_commitment
            {
                continue;
            }
            let request = root
                .retained_worker_registration_request_index
                .get(&registration.issuer_request_id)
                .ok_or_else(|| {
                    AuthorityFacadeError("retained authority edge has no exact request".into())
                })?;
            let RetainedWorkerAuthorityRegistrationRequestStateV1::Applied {
                authority_revision_after,
                authority_record_commitment_after,
            } = &request.state
            else {
                return Err(AuthorityFacadeError(
                    "retained authority journal request is not applied".into(),
                ));
            };
            if request.registration_id != registration.registration_id
                || request.orchestration_session_id != registration.orchestration_session_id
                || request.authority_revision_before != registration.authority_revision_before
                || request.authority_record_commitment_before
                    != registration.authority_record_commitment_before
                || *authority_revision_after != registration.authority_revision_after
                || *authority_record_commitment_after
                    != registration.authority_record_commitment_after
                || request.retained_participant_id != registration.retained_participant_id
                || request.retained_worker_ref_id != registration.retained_worker_ref.ref_id
                || request.retained_worker_commitment != registration.retained_worker_ref.commitment
                || request.current_policy_ref != registration.current_policy_ref
                || request.world_binding != registration.world_binding
                || request.registered_at != registration.registered_at
                || registration.authority_revision_after != next_revision
                || latest.authority.current_policy_ref.as_ref()
                    != Some(&registration.current_policy_ref)
                || latest.authority.world_binding.as_ref() != Some(&registration.world_binding)
                || latest
                    .authority
                    .authoritative_participant_lineage
                    .contains(&registration.retained_participant_id)
                || latest
                    .authority
                    .retained_worker_refs
                    .contains(&registration.retained_worker_ref)
                || registration.registered_at.as_str() < latest.authority.updated_at.as_str()
            {
                return Err(AuthorityFacadeError(
                    "retained authority edge is stale, substituted, or inconsistent".into(),
                ));
            }
            let mut next = latest.authority.clone();
            next.authority_revision = registration.authority_revision_after;
            next.authoritative_participant_lineage
                .push(registration.retained_participant_id.clone());
            next.retained_worker_refs
                .push(registration.retained_worker_ref.clone());
            next.updated_at = registration.registered_at.clone();
            let state = resolved_session_authority(
                root.root_revision,
                &root.authority_store_id,
                &root.bootstrap_home,
                next,
            )?;
            if state.authoritative_lineage_commitment
                != registration.authoritative_lineage_commitment_after
            {
                return Err(AuthorityFacadeError(
                    "retained authority lineage commitment is inconsistent".into(),
                ));
            }
            offer_exact_authority_edge(
                &mut candidate,
                key,
                state,
                Some(&registration.authority_record_commitment_after),
            )?;
        }

        for successor in root.successor_transition_intent_map.values() {
            if successor.orchestration_session_id != orchestration_session_id {
                continue;
            }
            let HostSessionTransitionIntentStateV3::Applied {
                authority_revision_before,
                authority_revision_after,
                authority_record_commitment,
                startup_ownership,
                post_turn,
                applied_at,
                ..
            } = &successor.state
            else {
                continue;
            };
            let journal = root
                .successor_application_journal
                .get(&successor.intent_id)
                .ok_or_else(|| {
                    AuthorityFacadeError("applied successor authority edge has no journal".into())
                })?;
            let initial_key = format!("successor-initial:{}", successor.intent_id);
            if !consumed.contains(&initial_key)
                && *authority_revision_before == Some(latest.authority.authority_revision)
            {
                let HostSessionAuthorityPreconditionV1::ExpectedRevision {
                    authority_revision,
                    authority_record_commitment: before_commitment,
                    active_authoritative_participant_id,
                    authoritative_lineage_commitment,
                    lifecycle_posture,
                } = &successor.authority_precondition
                else {
                    return Err(AuthorityFacadeError(
                        "successor authority edge has no exact precondition".into(),
                    ));
                };
                if *authority_revision != latest.authority.authority_revision
                    || before_commitment != &latest.authority_record_commitment
                    || active_authoritative_participant_id
                        != latest
                            .authority
                            .active_authoritative_participant_id
                            .as_ref()
                            .ok_or_else(|| {
                                AuthorityFacadeError(
                                    "successor predecessor has no active participant".into(),
                                )
                            })?
                    || authoritative_lineage_commitment != &latest.authoritative_lineage_commitment
                    || *lifecycle_posture != latest.authority.lifecycle_posture
                    || *authority_revision_after != next_revision
                    || journal.initial_application.authority_revision_before
                        != *authority_revision_before
                    || journal.initial_application.authority_revision_after
                        != *authority_revision_after
                    || journal.initial_application.authority_record_commitment
                        != *authority_record_commitment
                    || journal.initial_application.applied_at != *applied_at
                    || applied_at.as_str() < latest.authority.updated_at.as_str()
                    || successor.mode == HostSessionTransitionModeV1::Start
                {
                    return Err(AuthorityFacadeError(
                        "successor initial authority edge is inconsistent".into(),
                    ));
                }
                let mut next = latest.authority.clone();
                next.authority_revision = *authority_revision_after;
                next.active_authoritative_participant_id =
                    Some(successor.target_authoritative_participant_id.clone());
                next.authoritative_participant_lineage
                    .push(successor.target_authoritative_participant_id.clone());
                next.host_attach_contract_ref = Some(successor.host_attach_contract_ref.clone());
                next.world_binding = successor.world_binding.clone();
                if let Some(reference) = successor.resume_handle_ref.as_ref() {
                    if !next.internal_resume_handle_refs.contains(reference) {
                        next.internal_resume_handle_refs.push(reference.clone());
                    }
                }
                next.lifecycle_posture = HostSessionPostureV1::ActiveAttached;
                next.updated_at = applied_at.clone();
                let state = resolved_session_authority(
                    root.root_revision,
                    &root.authority_store_id,
                    &root.bootstrap_home,
                    next,
                )?;
                offer_exact_authority_edge(
                    &mut candidate,
                    initial_key.clone(),
                    state,
                    Some(authority_record_commitment),
                )?;
            }

            if let Some(terminal) = journal.startup_terminal_application.as_ref() {
                let key = format!("successor-startup-terminal:{}", successor.intent_id);
                if !consumed.contains(&key)
                    && terminal.authority_revision_before == latest.authority.authority_revision
                    && terminal.authority_record_commitment_before
                        == latest.authority_record_commitment
                {
                    if !consumed.contains(&initial_key)
                        || terminal.authority_revision_after != next_revision
                        || terminal.applied_at.as_str() < latest.authority.updated_at.as_str()
                        || !matches!(
                            startup_ownership.as_ref(),
                            HostSessionStartupOwnershipApplicationV1::TerminalReconciled {
                                authority_revision_before,
                                authority_revision_after,
                                resulting_posture,
                                ..
                            } if *authority_revision_before == terminal.authority_revision_before
                                && *authority_revision_after == terminal.authority_revision_after
                                && *resulting_posture == terminal.resulting_posture
                        )
                    {
                        return Err(AuthorityFacadeError(
                            "successor startup terminal authority edge is inconsistent".into(),
                        ));
                    }
                    let mut next = latest.authority.clone();
                    next.authority_revision = terminal.authority_revision_after;
                    next.lifecycle_posture = terminal.resulting_posture;
                    next.updated_at = terminal.applied_at.clone();
                    let state = resolved_session_authority(
                        root.root_revision,
                        &root.authority_store_id,
                        &root.bootstrap_home,
                        next,
                    )?;
                    offer_exact_authority_edge(
                        &mut candidate,
                        key,
                        state,
                        Some(&terminal.authority_record_commitment_after),
                    )?;
                }
            }

            if let Some(post_turn_journal) = journal.post_turn_application.as_ref() {
                let key = format!("successor-post-turn:{}", successor.intent_id);
                if !consumed.contains(&key)
                    && post_turn_journal.authority_revision_before
                        == latest.authority.authority_revision
                {
                    let HostSessionPostTurnApplicationV2::Applied {
                        authority_revision_before,
                        authority_revision_after,
                        resulting_posture,
                        applied_at,
                        ..
                    } = post_turn.as_ref()
                    else {
                        return Err(AuthorityFacadeError(
                            "successor post-turn journal has no applied state".into(),
                        ));
                    };
                    if !consumed.contains(&initial_key)
                        || *authority_revision_before != post_turn_journal.authority_revision_before
                        || *authority_revision_after != post_turn_journal.authority_revision_after
                        || *authority_revision_after != next_revision
                        || applied_at != &post_turn_journal.applied_at
                        || applied_at.as_str() < latest.authority.updated_at.as_str()
                    {
                        return Err(AuthorityFacadeError(
                            "successor post-turn authority edge is inconsistent".into(),
                        ));
                    }
                    let mut next = latest.authority.clone();
                    next.authority_revision = *authority_revision_after;
                    next.lifecycle_posture = *resulting_posture;
                    next.updated_at = applied_at.clone();
                    let state = resolved_session_authority(
                        root.root_revision,
                        &root.authority_store_id,
                        &root.bootstrap_home,
                        next,
                    )?;
                    offer_exact_authority_edge(
                        &mut candidate,
                        key,
                        state,
                        Some(&post_turn_journal.authority_record_commitment),
                    )?;
                }
            }
        }

        for stop in root.stop_transaction_map.values() {
            let HostSessionStopIntentStateV1::Completed {
                authority_revision_after,
                authority_record_commitment_after,
                completed_at,
                ..
            } = &stop.state
            else {
                continue;
            };
            let key = format!("stop:{}", stop.intent_id);
            if consumed.contains(&key)
                || stop.orchestration_session_id != orchestration_session_id
                || stop.authority_before.authority_revision != latest.authority.authority_revision
                || stop.authority_record_commitment_before != latest.authority_record_commitment
            {
                continue;
            }
            if stop.authority_before.as_ref() != &latest.authority
                || *authority_revision_after != next_revision
                || completed_at.as_str() < latest.authority.updated_at.as_str()
            {
                return Err(AuthorityFacadeError(
                    "HSA Stop authority edge is stale, substituted, or reordered".into(),
                ));
            }
            let mut next = latest.authority.clone();
            next.authority_revision = *authority_revision_after;
            next.lifecycle_posture = HostSessionPostureV1::Terminal;
            next.updated_at = completed_at.clone();
            let state = resolved_session_authority(
                root.root_revision,
                &root.authority_store_id,
                &root.bootstrap_home,
                next,
            )?;
            offer_exact_authority_edge(
                &mut candidate,
                key,
                state,
                Some(authority_record_commitment_after),
            )?;
        }

        let Some((key, state)) = candidate else {
            return Err(AuthorityFacadeError(
                "authority ancestry is not uniquely contiguous across typed HSA edges".into(),
            ));
        };
        if history.insert(next_revision, state).is_some() || !consumed.insert(key) {
            return Err(AuthorityFacadeError(
                "authority ancestry duplicated a revision or typed edge".into(),
            ));
        }
    }

    let reconstructed = history
        .last_key_value()
        .map(|(_, state)| state)
        .ok_or_else(|| AuthorityFacadeError("exact authority history is empty".into()))?;
    if &reconstructed.authority != current {
        return Err(AuthorityFacadeError(
            "current authority is not the exact terminal state of its typed ancestry".into(),
        ));
    }
    Ok(history)
}

fn retained_reservation_input(
    registration_request_id: &str,
    orchestration_session_id: &str,
    expected_authority: &RetainedWorkerAuthorityPreconditionV1,
    retained_participant_id: &str,
    descriptor_bytes: Vec<u8>,
    resume_handle_bytes: Vec<u8>,
) -> Result<store::RetainedWorkerReservationInputV1, AuthorityFacadeError> {
    if registration_request_id.is_empty()
        || orchestration_session_id.is_empty()
        || retained_participant_id.is_empty()
        || expected_authority.authority_store_id.is_empty()
        || expected_authority.authority_revision == 0
    {
        return Err(AuthorityFacadeError(
            "retained registration plan identity is invalid".into(),
        ));
    }
    Ok(store::RetainedWorkerReservationInputV1 {
        issuer_request_id: format!("retained-worker-registration:{registration_request_id}"),
        orchestration_session_id: orchestration_session_id.to_owned(),
        expected_authority_store_id: expected_authority.authority_store_id.clone(),
        expected_authority_revision: expected_authority.authority_revision,
        expected_authority_commitment: expected_authority.authority_record_commitment.clone(),
        retained_participant_id: retained_participant_id.to_owned(),
        descriptor_bytes,
        resume_handle_bytes,
    })
}

fn reserved_registration(
    reserved: store::RetainedWorkerReservationV1,
) -> ReservedRetainedWorkerRegistrationV1 {
    ReservedRetainedWorkerRegistrationV1 {
        request: reserved.request,
        descriptor_ref: reserved.descriptor_ref,
        resume_handle_ref: reserved.resume_handle_ref,
        retained_worker_ref: reserved.retained_worker_ref,
        descriptor_bytes: reserved.descriptor_bytes,
        resume_handle_bytes: reserved.resume_handle_bytes,
        retained_worker_bytes: reserved.retained_worker_bytes,
        joined: reserved.joined,
    }
}

fn store_reservation(
    reserved: &ReservedRetainedWorkerRegistrationV1,
) -> store::RetainedWorkerReservationV1 {
    store::RetainedWorkerReservationV1 {
        request: reserved.request.clone(),
        descriptor_ref: reserved.descriptor_ref.clone(),
        resume_handle_ref: reserved.resume_handle_ref.clone(),
        retained_worker_ref: reserved.retained_worker_ref.clone(),
        descriptor_bytes: reserved.descriptor_bytes.clone(),
        resume_handle_bytes: reserved.resume_handle_bytes.clone(),
        retained_worker_bytes: reserved.retained_worker_bytes.clone(),
        joined: reserved.joined,
    }
}

fn store_error(error: store::BootstrapError) -> AuthorityFacadeError {
    AuthorityFacadeError(error.to_string())
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
fn trusted_fs_error(error: super::trusted_fs::TrustedFsError) -> AuthorityFacadeError {
    AuthorityFacadeError(error.to_string())
}

fn unsupported_platform() -> AuthorityFacadeError {
    AuthorityFacadeError("A1 host-session authority is unsupported on this platform".into())
}

pub(super) fn exact_current_authority_proof<'root>(
    root: &'root StateRootV1,
    orchestration_session_id: &str,
    authority_revision: u64,
) -> Result<&'root AuthorityObjectCommitmentV1, AuthorityFacadeError> {
    let mut commitment = None;
    let mut highest_revision = None;
    for journal in root.application_journal.values() {
        let Some(intent) = root.transition_intent_map.get(&journal.intent_id) else {
            continue;
        };
        if intent.orchestration_session_id != orchestration_session_id {
            continue;
        }
        let phases = std::iter::once(&journal.initial_application).map(|initial| {
            (
                initial.authority_revision_after,
                &initial.authority_record_commitment,
            )
        });
        let phases = phases.chain(journal.post_turn_application.as_ref().map(|post_turn| {
            (
                post_turn.authority_revision_after,
                &post_turn.authority_record_commitment,
            )
        }));
        for (revision, candidate) in phases {
            highest_revision =
                Some(highest_revision.map_or(revision, |highest: u64| highest.max(revision)));
            if revision != authority_revision {
                continue;
            }
            if commitment.replace(candidate).is_some() {
                return Err(AuthorityFacadeError(
                    "current authority revision has ambiguous application proof".into(),
                ));
            }
        }
    }
    if highest_revision.is_some_and(|revision| revision > authority_revision) {
        return Err(AuthorityFacadeError(
            "current authority revision is behind durable application proof".into(),
        ));
    }
    let Some(commitment) = commitment else {
        return Err(AuthorityFacadeError(
            "current authority revision has no durable application proof".into(),
        ));
    };
    Ok(commitment)
}

fn exact_current_authority_proof_v3(
    authority_facade: &HostSessionAuthority,
    root: &StateRootV3,
    orchestration_session_id: &str,
    authority_revision: u64,
) -> Result<AuthorityObjectCommitmentV1, AuthorityFacadeError> {
    let mut commitment = None;
    let mut highest_revision = None;
    for allocation in root.fork_successor_allocation_map.values() {
        if allocation.request.target_orchestration_session_id != orchestration_session_id {
            continue;
        }
        let revision = allocation.target_authority.authority_revision;
        highest_revision =
            Some(highest_revision.map_or(revision, |highest: u64| highest.max(revision)));
        if revision != authority_revision {
            continue;
        }
        if commitment
            .replace(&allocation.target_authority_record_commitment)
            .is_some()
        {
            return Err(AuthorityFacadeError(
                "current authority revision has ambiguous fork allocation proof".into(),
            ));
        }
    }
    for journal in root.application_journal.values() {
        let Some(intent) = root.transition_intent_map.get(&journal.intent_id) else {
            continue;
        };
        if intent.orchestration_session_id != orchestration_session_id {
            continue;
        }
        let phases = std::iter::once((
            journal.initial_application.authority_revision_after,
            &journal.initial_application.authority_record_commitment,
        ))
        .chain(
            journal
                .startup_terminal_application
                .as_ref()
                .map(|startup| {
                    (
                        startup.authority_revision_after,
                        &startup.authority_record_commitment_after,
                    )
                }),
        )
        .chain(journal.post_turn_application.as_ref().map(|post_turn| {
            (
                post_turn.authority_revision_after,
                &post_turn.authority_record_commitment,
            )
        }));
        for (revision, candidate) in phases {
            highest_revision =
                Some(highest_revision.map_or(revision, |highest: u64| highest.max(revision)));
            if revision != authority_revision {
                continue;
            }
            if commitment.replace(candidate).is_some() {
                return Err(AuthorityFacadeError(
                    "current authority revision has ambiguous durable V3 application proof".into(),
                ));
            }
        }
    }
    for journal in root.successor_application_journal.values() {
        let Some(intent) = root.successor_transition_intent_map.get(&journal.intent_id) else {
            continue;
        };
        if intent.orchestration_session_id != orchestration_session_id {
            continue;
        }
        let phases = std::iter::once((
            journal.initial_application.authority_revision_after,
            &journal.initial_application.authority_record_commitment,
        ))
        .chain(
            journal
                .startup_terminal_application
                .as_ref()
                .map(|startup| {
                    (
                        startup.authority_revision_after,
                        &startup.authority_record_commitment_after,
                    )
                }),
        )
        .chain(journal.post_turn_application.as_ref().map(|post_turn| {
            (
                post_turn.authority_revision_after,
                &post_turn.authority_record_commitment,
            )
        }));
        for (revision, candidate) in phases {
            highest_revision =
                Some(highest_revision.map_or(revision, |highest: u64| highest.max(revision)));
            if revision != authority_revision {
                continue;
            }
            if commitment.replace(candidate).is_some() {
                return Err(AuthorityFacadeError(
                    "current authority revision has ambiguous durable V3 application proof".into(),
                ));
            }
        }
    }
    for registration in root.retained_worker_registration_journal.values() {
        if registration.orchestration_session_id != orchestration_session_id {
            continue;
        }
        let revision = registration.authority_revision_after;
        let candidate = &registration.authority_record_commitment_after;
        highest_revision =
            Some(highest_revision.map_or(revision, |highest: u64| highest.max(revision)));
        if revision != authority_revision {
            continue;
        }
        if commitment.replace(candidate).is_some() {
            return Err(AuthorityFacadeError(
                "current authority revision has ambiguous durable V3 application proof".into(),
            ));
        }
    }
    for stop in root.stop_transaction_map.values() {
        if stop.orchestration_session_id != orchestration_session_id {
            continue;
        }
        let HostSessionStopIntentStateV1::Completed {
            authority_revision_after,
            authority_record_commitment_after,
            ..
        } = &stop.state
        else {
            continue;
        };
        highest_revision = Some(
            highest_revision.map_or(*authority_revision_after, |highest: u64| {
                highest.max(*authority_revision_after)
            }),
        );
        if *authority_revision_after != authority_revision {
            continue;
        }
        if commitment
            .replace(authority_record_commitment_after)
            .is_some()
        {
            return Err(AuthorityFacadeError(
                "current authority revision has ambiguous durable V3 application proof".into(),
            ));
        }
    }
    if highest_revision.is_some_and(|revision| revision > authority_revision) {
        return Err(AuthorityFacadeError(
            "current authority revision is behind durable V3 application proof".into(),
        ));
    }
    if let Some(commitment) = commitment {
        return Ok(commitment.clone());
    }
    let authority = root
        .session_namespace_map
        .get(orchestration_session_id)
        .and_then(|record| match record {
            SessionNamespaceRecordV1::Authority(authority) => Some(authority.as_ref()),
            _ => None,
        })
        .ok_or_else(|| {
            AuthorityFacadeError("current V3 authority proof has no authority record".into())
        })?;
    let reference = authority
        .internal_resume_handle_refs
        .last()
        .filter(|reference| reference.schema_version == 2)
        .ok_or_else(|| {
            AuthorityFacadeError(
                "current authority revision has no durable V3 application proof".into(),
            )
        })?;
    let bytes = store::read_typed_object_v2_or_v3_opened(
        authority_facade.trusted_root(),
        root.root_revision,
        reference,
        None,
    )
    .map_err(store_error)?;
    let handle: StartContinuationHandleHashInputV2 =
        canonical_json::from_slice(&bytes).map_err(|error| {
            AuthorityFacadeError(format!(
                "decode Start continuation authority proof: {error}"
            ))
        })?;
    if handle.authority_store_id != root.authority_store_id
        || handle.orchestration_session_id != orchestration_session_id
        || handle.authority_revision_after != authority_revision
    {
        return Err(AuthorityFacadeError(
            "Start continuation authority proof does not authenticate current revision".into(),
        ));
    }
    canonical_commitment(&authority_hash_input(authority))
}

fn verify_retained_registration_descendant_v2(
    root: &StateRootV2,
    initial_authority: &DurableSessionAuthorityV1,
    current_authority: &DurableSessionAuthorityV1,
    initial_commitment: &AuthorityObjectCommitmentV1,
) -> Result<(), AuthorityFacadeError> {
    let mut expected = initial_authority.clone();
    let mut expected_commitment = canonical_commitment(&authority_hash_input(&expected))?;
    if &expected_commitment != initial_commitment
        || current_authority.authority_revision < expected.authority_revision
    {
        return Err(AuthorityFacadeError(
            "Start application authority origin is inconsistent".into(),
        ));
    }
    let mut consumed = Vec::new();
    while expected.authority_revision < current_authority.authority_revision {
        let candidates = root
            .retained_worker_registration_journal
            .iter()
            .filter(|(_, registration)| {
                registration.orchestration_session_id == expected.orchestration_session_id
                    && registration.authority_revision_before == expected.authority_revision
                    && registration.authority_record_commitment_before == expected_commitment
            })
            .collect::<Vec<_>>();
        let [(registration_key, registration)] = candidates.as_slice() else {
            return Err(AuthorityFacadeError(
                "current authority has no unique contiguous R0 registration ancestry".into(),
            ));
        };
        if *registration_key != &registration.registration_id
            || registration.authority_revision_after != expected.authority_revision + 1
            || expected.current_policy_ref.as_ref() != Some(&registration.current_policy_ref)
            || expected.world_binding.as_ref() != Some(&registration.world_binding)
            || expected
                .authoritative_participant_lineage
                .contains(&registration.retained_participant_id)
            || expected
                .retained_worker_refs
                .contains(&registration.retained_worker_ref)
        {
            return Err(AuthorityFacadeError(
                "R0 registration ancestry link is inconsistent".into(),
            ));
        }
        let request = root
            .retained_worker_registration_request_index
            .get(&registration.issuer_request_id)
            .ok_or_else(|| {
                AuthorityFacadeError("R0 registration ancestry has no request record".into())
            })?;
        if request.issuer_request_id != registration.issuer_request_id
            || request.registration_id != registration.registration_id
            || request.orchestration_session_id != registration.orchestration_session_id
            || request.authority_revision_before != registration.authority_revision_before
            || request.authority_record_commitment_before
                != registration.authority_record_commitment_before
            || request.retained_participant_id != registration.retained_participant_id
            || request.descriptor_ref_id != registration.descriptor_ref.ref_id
            || request.descriptor_commitment != registration.descriptor_ref.commitment
            || request.resume_handle_ref_id != registration.resume_handle_ref.ref_id
            || request.resume_handle_commitment != registration.resume_handle_ref.commitment
            || request.retained_worker_ref_id != registration.retained_worker_ref.ref_id
            || request.retained_worker_commitment != registration.retained_worker_ref.commitment
            || request.current_policy_ref != registration.current_policy_ref
            || request.world_binding != registration.world_binding
            || request.registered_at != registration.registered_at
            || !matches!(
                &request.state,
                super::store_schema::RetainedWorkerAuthorityRegistrationRequestStateV1::Applied {
                    authority_revision_after,
                    authority_record_commitment_after,
                } if *authority_revision_after == registration.authority_revision_after
                    && authority_record_commitment_after
                        == &registration.authority_record_commitment_after
            )
        {
            return Err(AuthorityFacadeError(
                "R0 registration request and ancestry link disagree".into(),
            ));
        }

        expected.authority_revision = registration.authority_revision_after;
        expected
            .authoritative_participant_lineage
            .push(registration.retained_participant_id.clone());
        expected
            .retained_worker_refs
            .push(registration.retained_worker_ref.clone());
        expected.updated_at = registration.registered_at.clone();
        let lineage_commitment = canonical_commitment(&AuthoritativeLineageHashInputV1 {
            schema_version: 1,
            orchestration_session_id: expected.orchestration_session_id.clone(),
            participant_ids: expected.authoritative_participant_lineage.clone(),
        })?;
        if lineage_commitment != registration.authoritative_lineage_commitment_after {
            return Err(AuthorityFacadeError(
                "R0 registration lineage commitment is inconsistent".into(),
            ));
        }
        expected_commitment = canonical_commitment(&authority_hash_input(&expected))?;
        if expected_commitment != registration.authority_record_commitment_after {
            return Err(AuthorityFacadeError(
                "R0 registration authority commitment is inconsistent".into(),
            ));
        }
        consumed.push(registration.registration_id.clone());
    }
    let session_registration_count = root
        .retained_worker_registration_journal
        .values()
        .filter(|registration| {
            registration.orchestration_session_id == expected.orchestration_session_id
        })
        .count();
    if consumed.len() != session_registration_count || expected != *current_authority {
        return Err(AuthorityFacadeError(
            "current authority is not the exact contiguous R0 registration descendant".into(),
        ));
    }
    Ok(())
}

fn validate_current_authority_proofs(root: &StateRootV1) -> Result<(), AuthorityFacadeError> {
    for record in root.session_namespace_map.values() {
        let SessionNamespaceRecordV1::Authority(authority) = record else {
            continue;
        };
        let computed = canonical_commitment(&authority_hash_input(authority))?;
        let persisted = exact_current_authority_proof(
            root,
            &authority.orchestration_session_id,
            authority.authority_revision,
        )?;
        if persisted != &computed {
            return Err(AuthorityFacadeError(
                "proposed authority has no matching durable application proof".into(),
            ));
        }
    }
    Ok(())
}

fn canonical_commitment<T>(value: &T) -> Result<AuthorityObjectCommitmentV1, AuthorityFacadeError>
where
    T: super::validation::CanonicalHashInputV1,
{
    Ok(AuthorityObjectCommitmentV1::CanonicalSha256 {
        digest_hex: canonical_sha256(value)
            .map_err(|error| AuthorityFacadeError(error.to_string()))?,
    })
}

fn authority_hash_input(
    authority: &DurableSessionAuthorityV1,
) -> DurableSessionAuthorityHashInputV1 {
    DurableSessionAuthorityHashInputV1 {
        schema_version: authority.schema_version,
        orchestration_session_id: authority.orchestration_session_id.clone(),
        shell_trace_session_id: authority.shell_trace_session_id.clone(),
        authority_revision: authority.authority_revision,
        origin: authority.origin.clone(),
        authoritative_participant_lineage: authority.authoritative_participant_lineage.clone(),
        active_authoritative_participant_id: authority.active_authoritative_participant_id.clone(),
        workspace_binding: authority.workspace_binding.clone(),
        world_binding: authority.world_binding.clone(),
        host_attach_contract_ref: authority.host_attach_contract_ref.clone(),
        retained_worker_refs: authority.retained_worker_refs.clone(),
        internal_resume_handle_refs: authority.internal_resume_handle_refs.clone(),
        lifecycle_posture: authority.lifecycle_posture,
        current_policy_ref: authority.current_policy_ref.clone(),
        current_policy_revision: authority.current_policy_revision.clone(),
    }
}

#[cfg(all(test, any(target_os = "linux", target_os = "macos")))]
mod tests {
    use std::fs;
    use std::os::unix::fs::PermissionsExt;

    use super::HostSessionAuthority;
    use crate::execution::agent_runtime::host_session_authority::trusted_fs::{
        ensure_private_substrate_home, TrustedAuthorityRoot,
    };

    fn write_private(path: &std::path::Path, bytes: &[u8]) {
        fs::write(path, bytes).unwrap();
        fs::set_permissions(path, fs::Permissions::from_mode(0o600)).unwrap();
    }

    #[test]
    fn opened_facade_rejects_lexical_home_replacement_without_touching_replacement() {
        let safe_parent = std::env::var_os("XDG_RUNTIME_DIR")
            .map(std::path::PathBuf::from)
            .unwrap_or_else(|| {
                std::path::PathBuf::from(std::env::var_os("HOME").expect("tests require HOME"))
                    .join(".cache")
            });
        fs::create_dir_all(&safe_parent).unwrap();
        let parent = tempfile::tempdir_in(safe_parent).unwrap();
        fs::set_permissions(parent.path(), fs::Permissions::from_mode(0o700)).unwrap();
        let home = parent.path().join("home");
        fs::create_dir(&home).unwrap();
        fs::set_permissions(&home, fs::Permissions::from_mode(0o700)).unwrap();

        let authority = HostSessionAuthority::open(&home).unwrap();
        authority.bootstrap().unwrap();
        assert_eq!(
            authority.classify(),
            super::BootstrapClassificationV1::ValidExisting
        );
        let retained = parent.path().join("retained");
        fs::rename(&home, &retained).unwrap();
        fs::create_dir(&home).unwrap();
        fs::set_permissions(&home, fs::Permissions::from_mode(0o700)).unwrap();

        assert!(authority.read_root().is_err());
        assert!(!home.join("authority-v1").exists());
        assert!(retained.join("authority-v1/state-root-v1.json").exists());
    }

    #[test]
    fn accepted_home_replacement_before_facade_handoff_fails_closed() {
        let safe_parent = std::env::var_os("XDG_RUNTIME_DIR")
            .map(std::path::PathBuf::from)
            .unwrap_or_else(|| {
                std::path::PathBuf::from(std::env::var_os("HOME").expect("tests require HOME"))
                    .join(".cache")
            });
        fs::create_dir_all(&safe_parent).unwrap();
        let parent = tempfile::tempdir_in(safe_parent).unwrap();
        fs::set_permissions(parent.path(), fs::Permissions::from_mode(0o700)).unwrap();
        let home = parent.path().join("home");
        let accepted = ensure_private_substrate_home(&home, unsafe { libc::geteuid() }).unwrap();
        let retained = parent.path().join("retained");
        fs::rename(&home, &retained).unwrap();
        fs::create_dir(&home).unwrap();
        fs::set_permissions(&home, fs::Permissions::from_mode(0o700)).unwrap();

        assert!(HostSessionAuthority::from_trusted_root(accepted).is_err());
        assert!(!home.join("authority-v1").exists());
        assert!(!retained.join("authority-v1").exists());

        let replacement = TrustedAuthorityRoot::open(&home).unwrap();
        assert_eq!(
            replacement.identity().physical_path,
            home.display().to_string()
        );
    }

    #[test]
    fn absent_bootstrap_entries_revalidate_identity_before_success() {
        for entry in ["config", "policy", "agents"] {
            let safe_parent = std::env::var_os("XDG_RUNTIME_DIR")
                .map(std::path::PathBuf::from)
                .unwrap_or_else(|| {
                    std::path::PathBuf::from(std::env::var_os("HOME").expect("tests require HOME"))
                        .join(".cache")
                });
            fs::create_dir_all(&safe_parent).unwrap();
            let parent = tempfile::tempdir_in(safe_parent).unwrap();
            fs::set_permissions(parent.path(), fs::Permissions::from_mode(0o700)).unwrap();
            let home = parent.path().join("home");
            fs::create_dir(&home).unwrap();
            fs::set_permissions(&home, fs::Permissions::from_mode(0o700)).unwrap();
            let authority = HostSessionAuthority::open(&home).unwrap();
            let bootstrap_home = authority.bootstrap_home();
            match entry {
                "config" => assert_eq!(bootstrap_home.read_config_yaml().unwrap(), None),
                "policy" => assert_eq!(bootstrap_home.read_policy_yaml().unwrap(), None),
                "agents" => assert!(bootstrap_home
                    .read_agent_inventory_yaml()
                    .unwrap()
                    .is_empty()),
                _ => unreachable!(),
            }
            let retained = parent.path().join("retained");
            let replacement = || {
                fs::rename(&home, &retained).unwrap();
                fs::create_dir(&home).unwrap();
                fs::set_permissions(&home, fs::Permissions::from_mode(0o700)).unwrap();
            };

            let result = match entry {
                "config" => bootstrap_home
                    .read_optional_file_after_revalidation("config.yaml", replacement)
                    .map(|_| ()),
                "policy" => bootstrap_home
                    .read_optional_file_after_revalidation("policy.yaml", replacement)
                    .map(|_| ()),
                "agents" => bootstrap_home
                    .read_agent_inventory_yaml_after_revalidation(replacement)
                    .map(|_| ()),
                _ => unreachable!(),
            };
            assert!(
                result.is_err(),
                "absent {entry} must close with revalidation"
            );
            assert!(fs::read_dir(&home).unwrap().next().is_none());
            assert!(fs::read_dir(&retained).unwrap().next().is_none());
        }
    }

    #[cfg(unix)]
    #[test]
    fn explicit_bootstrap_descriptors_reject_symlinks_and_non_files() {
        let safe_parent = std::env::var_os("XDG_RUNTIME_DIR")
            .map(std::path::PathBuf::from)
            .unwrap_or_else(|| {
                std::path::PathBuf::from(std::env::var_os("HOME").expect("tests require HOME"))
                    .join(".cache")
            });
        fs::create_dir_all(&safe_parent).unwrap();
        let parent = tempfile::tempdir_in(safe_parent).unwrap();
        fs::set_permissions(parent.path(), fs::Permissions::from_mode(0o700)).unwrap();
        let home = parent.path().join("home");
        fs::create_dir(&home).unwrap();
        fs::set_permissions(&home, fs::Permissions::from_mode(0o700)).unwrap();

        let external_config = parent.path().join("external-config.yaml");
        write_private(&external_config, b"world:\n  enabled: true\n");
        std::os::unix::fs::symlink(&external_config, home.join("config.yaml")).unwrap();
        fs::create_dir(home.join("policy.yaml")).unwrap();
        fs::set_permissions(home.join("policy.yaml"), fs::Permissions::from_mode(0o700)).unwrap();
        fs::create_dir(home.join("agents")).unwrap();
        fs::set_permissions(home.join("agents"), fs::Permissions::from_mode(0o700)).unwrap();
        let external_agent = parent.path().join("external-agent.yaml");
        write_private(
            &external_agent,
            b"version: 1\nid: escaped\nconfig:\n  kind: cli\n",
        );
        std::os::unix::fs::symlink(&external_agent, home.join("agents/escaped.yaml")).unwrap();

        let authority = HostSessionAuthority::open(&home).unwrap();
        let bootstrap_home = authority.bootstrap_home();
        assert!(bootstrap_home.read_config_yaml().is_err());
        assert!(bootstrap_home.read_policy_yaml().is_err());
        assert!(bootstrap_home.read_agent_inventory_yaml().is_err());
    }

    #[test]
    fn explicit_config_preserves_conditional_policy_parsing() {
        let safe_parent = std::env::var_os("XDG_RUNTIME_DIR")
            .map(std::path::PathBuf::from)
            .unwrap_or_else(|| {
                std::path::PathBuf::from(std::env::var_os("HOME").expect("tests require HOME"))
                    .join(".cache")
            });
        fs::create_dir_all(&safe_parent).unwrap();
        let parent = tempfile::tempdir_in(safe_parent).unwrap();
        fs::set_permissions(parent.path(), fs::Permissions::from_mode(0o700)).unwrap();
        let home = parent.path().join("home");
        fs::create_dir(&home).unwrap();
        fs::set_permissions(&home, fs::Permissions::from_mode(0o700)).unwrap();
        write_private(&home.join("config.yaml"), b"{}\n");
        write_private(&home.join("policy.yaml"), b"world_fs: [\n");
        let authority = HostSessionAuthority::open(&home).unwrap();
        let bootstrap_home = authority.bootstrap_home();

        crate::execution::config_model::resolve_effective_config_for_bootstrap_home(
            parent.path(),
            &crate::execution::config_model::CliConfigOverrides::default(),
            &bootstrap_home,
        )
        .expect("in_world config must not parse malformed policy");

        write_private(
            &home.join("config.yaml"),
            b"llm:\n  gateway:\n    mode: host_only\n",
        );
        assert!(
            crate::execution::config_model::resolve_effective_config_for_bootstrap_home(
                parent.path(),
                &crate::execution::config_model::CliConfigOverrides::default(),
                &bootstrap_home,
            )
            .is_err()
        );
    }

    #[test]
    fn explicit_bootstrap_home_drives_config_policy_snapshot_inventory_and_state_store() {
        let safe_parent = std::env::var_os("XDG_RUNTIME_DIR")
            .map(std::path::PathBuf::from)
            .unwrap_or_else(|| {
                std::path::PathBuf::from(std::env::var_os("HOME").expect("tests require HOME"))
                    .join(".cache")
            });
        fs::create_dir_all(&safe_parent).unwrap();
        let parent = tempfile::tempdir_in(safe_parent).unwrap();
        fs::set_permissions(parent.path(), fs::Permissions::from_mode(0o700)).unwrap();
        let accepted_home = parent.path().join("accepted");
        let ambient_home = parent.path().join("ambient");
        for home in [&accepted_home, &ambient_home] {
            fs::create_dir(home).unwrap();
            fs::set_permissions(home, fs::Permissions::from_mode(0o700)).unwrap();
            fs::create_dir(home.join("agents")).unwrap();
            fs::set_permissions(home.join("agents"), fs::Permissions::from_mode(0o700)).unwrap();
        }
        write_private(
            &accepted_home.join("config.yaml"),
            b"world:\n  enabled: false\n",
        );
        write_private(
            &ambient_home.join("config.yaml"),
            b"world:\n  enabled: true\n",
        );
        write_private(&accepted_home.join("policy.yaml"), b"id: policy-a\n");
        write_private(&ambient_home.join("policy.yaml"), b"id: policy-b\n");
        write_private(
            &accepted_home.join("agents/accepted.yaml"),
            br#"version: 1
id: accepted
config:
  kind: cli
  enabled: true
  protocol: substrate.agent.session
  execution:
    scope: host
  cli:
    binary: accepted
  capabilities:
    llm: true
"#,
        );
        write_private(
            &ambient_home.join("agents/ambient.yaml"),
            br#"version: 1
id: ambient
config:
  kind: cli
  enabled: true
  protocol: substrate.agent.session
  execution:
    scope: host
  cli:
    binary: ambient
  capabilities:
    llm: true
"#,
        );

        let authority = HostSessionAuthority::open(&accepted_home).unwrap();
        let bootstrap_home = authority.bootstrap_home();
        let config = crate::execution::config_model::resolve_effective_config_for_bootstrap_home(
            &ambient_home,
            &crate::execution::config_model::CliConfigOverrides::default(),
            &bootstrap_home,
        )
        .unwrap();
        assert!(!config.world.enabled);
        let policy = crate::execution::policy_model::resolve_effective_policy_for_bootstrap_home(
            &ambient_home,
            &bootstrap_home,
        )
        .unwrap();
        assert_eq!(policy.id, "policy-a");
        let snapshot =
            crate::execution::policy_snapshot::resolve_policy_snapshot_for_bootstrap_home(
                &ambient_home,
                &bootstrap_home,
            )
            .unwrap();
        assert!(!snapshot.snapshot_hash.is_empty());
        let post_acceptance_cwd = parent.path().join("post-acceptance-cwd");
        fs::create_dir(&post_acceptance_cwd).unwrap();
        fs::set_permissions(&post_acceptance_cwd, fs::Permissions::from_mode(0o700)).unwrap();
        let inventory =
            crate::execution::agent_inventory::load_effective_agent_inventory_for_bootstrap_home(
                &post_acceptance_cwd,
                &policy,
                &bootstrap_home,
            )
            .unwrap();
        assert!(inventory.contains_key("accepted"));
        assert!(!inventory.contains_key("ambient"));
        assert_eq!(
            inventory.get("accepted").unwrap().path,
            accepted_home.join("agents/accepted.yaml")
        );
        let state_store =
            crate::execution::agent_runtime::AgentRuntimeStateStore::for_bootstrap_home(
                &bootstrap_home,
            )
            .unwrap();
        assert_eq!(
            state_store.bootstrap_home_identity(),
            bootstrap_home.identity().unwrap()
        );
        assert_eq!(
            state_store.bootstrap_home_identity().physical_path,
            accepted_home.display().to_string()
        );

        let retained = parent.path().join("retained");
        fs::rename(&accepted_home, &retained).unwrap();
        fs::create_dir(&accepted_home).unwrap();
        fs::set_permissions(&accepted_home, fs::Permissions::from_mode(0o700)).unwrap();
        write_private(
            &accepted_home.join("config.yaml"),
            b"world:\n  enabled: true\n",
        );
        assert!(
            crate::execution::config_model::resolve_effective_config_for_bootstrap_home(
                &ambient_home,
                &crate::execution::config_model::CliConfigOverrides::default(),
                &bootstrap_home,
            )
            .is_err()
        );
        assert!(!accepted_home.join("agents").exists());
    }
}
