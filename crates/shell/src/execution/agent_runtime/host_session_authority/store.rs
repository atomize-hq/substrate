use std::fmt;
use std::path::Path;

use super::schema::TimestampV1;
use super::store_schema::StateRootV1;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum BootstrapClassificationV1 {
    FreshAbsent,
    InitializationPending,
    ValidExisting,
    UnsupportedLegacyState,
    CorruptOrUnsupported,
}

pub(crate) fn classify(path: &Path) -> BootstrapClassificationV1 {
    platform::classify(path)
}

pub(crate) fn bootstrap(path: &Path) -> Result<StateRootV1, BootstrapError> {
    platform::bootstrap(path)
}

pub(crate) fn rotate_commitment_key(path: &Path) -> Result<StateRootV1, BootstrapError> {
    platform::rotate_commitment_key(path)
}

pub(crate) fn retire_commitment_key(
    path: &Path,
    key_id: &str,
) -> Result<StateRootV1, BootstrapError> {
    platform::retire_commitment_key(path, key_id)
}

pub(crate) fn prepare_typed_object(
    path: &Path,
    reference: &crate::execution::agent_runtime::host_session_authority::schema::AuthorityObjectRefV1,
    bytes: &[u8],
    context: Option<&ObjectVerificationContextV1>,
) -> Result<ObjectPublicationOutcomeV1, BootstrapError> {
    platform::prepare_typed_object(path, reference, bytes, context)
}

#[derive(Clone)]
struct InitializationMaterialV1 {
    store_entropy: [u8; 16],
    key_entropy: [u8; 16],
    marker_nonce: [u8; 16],
    key_nonce: [u8; 16],
    root_nonce: [u8; 16],
    secret_key: [u8; 32],
    created_at: TimestampV1,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum InitializationCrashPointV1 {
    Marker,
    Key,
    Root,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum KeyLifecycleCrashPointV1 {
    KeyPublished,
    RootPublished,
}

#[derive(Clone, Copy)]
enum LegacyMutationV1 {
    ReplaceSessions,
    CreateMissingSessions,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct BootstrapError(&'static str);

impl fmt::Display for BootstrapError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.0)
    }
}

impl std::error::Error for BootstrapError {}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ObjectVerificationContextV1 {
    pub(crate) intent_id: String,
    pub(crate) run_id: String,
    pub(crate) parent_intent: Option<
        Box<
            crate::execution::agent_runtime::host_session_authority::store_schema::HostSessionTransitionIntentV1,
        >,
    >,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ObjectPublicationOutcomeV1 {
    PublishedOrphan,
    JoinedExactOrphan,
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
mod platform {
    use std::fmt;

    use rand::RngCore;

    #[cfg(test)]
    use super::LegacyMutationV1;
    use super::{
        BootstrapClassificationV1, BootstrapError, InitializationCrashPointV1,
        InitializationMaterialV1, KeyLifecycleCrashPointV1, ObjectPublicationOutcomeV1,
        ObjectVerificationContextV1,
    };
    use crate::execution::agent_runtime::host_session_authority::canonical_json;
    use crate::execution::agent_runtime::host_session_authority::hash::{
        canonical_sha256, store_hmac_sha256, validate_object_commitment_rule, SensitiveDomainV1,
    };
    use crate::execution::agent_runtime::host_session_authority::schema::{
        AgentDescriptorHashInputV1, ApplicationResultHashInputV1, AuthorityObjectCommitmentV1,
        AuthorityObjectKindV1, AuthorityObjectRefV1, HostAttachContractHashInputV1,
        InputAcceptanceHashInputV1, PolicyObjectHashInputV1, PostTurnCompletionHashInputV1,
        ResumeHandleHashInputV1, RetainedWorkerObjectHashInputV1, TerminalHandoffHashInputV1,
        TerminalHandoffStateV1,
    };
    use crate::execution::agent_runtime::host_session_authority::store_format::{
        key_id, nonce, store_id, validate_key_id, validate_ref_id,
        AuthorityStoreCommitmentKeyFileV1, TempNameV1,
    };
    use crate::execution::agent_runtime::host_session_authority::store_schema::{
        AuthorityObjectStorageStateV1, AuthorityStoreCommitmentAlgorithmV1,
        AuthorityStoreCommitmentKeyStateV1, AuthorityStoreCommitmentKeyV1,
        AuthorityStoreInitializationV1, GreenfieldNamespaceCertificateV1,
        HostSessionPostTurnApplicationV1, HostSessionTransitionInputHandoffV1,
        HostSessionTransitionIntentStateV1, HostSessionTransitionIntentV1,
        HostSessionTransitionTransportPayloadStateV1, SessionNamespaceRecordV1, StateRootV1,
    };
    use crate::execution::agent_runtime::host_session_authority::trusted_fs::{
        DirectoryEntry, EntryKind, TrustedAuthorityRoot, TrustedDirectory, TrustedFile,
    };
    use crate::execution::agent_runtime::host_session_authority::validation::CanonicalHashInputV1;

    const AUTHORITY_DIRECTORY: &str = "authority-v1";
    const ROOT_FILE: &str = "state-root-v1.json";
    const INIT_FILE: &str = "init-v1.json";
    const ROOT_LOCK_FILE: &str = "root.lock";

    #[path = "legacy.rs"]
    mod legacy;
    use legacy::LegacyObservation;
    #[path = "layout.rs"]
    mod layout;
    #[cfg(test)]
    use layout::validate_resume_identity;
    use layout::StoreLayout;
    #[path = "key_lifecycle.rs"]
    mod key_lifecycle;
    use key_lifecycle::{retire_commitment_key_with, rotate_commitment_key_with};
    #[path = "object_persistence.rs"]
    mod object_persistence;
    use object_persistence::publish_or_join_orphan;
    use object_persistence::verify_object_bytes;
    #[path = "reachability.rs"]
    mod reachability;
    use reachability::{add_expected_ref, collect_reachable_objects};

    pub(super) fn classify(path: &std::path::Path) -> BootstrapClassificationV1 {
        classify_checked(path).unwrap_or(BootstrapClassificationV1::CorruptOrUnsupported)
    }

    pub(super) fn bootstrap(path: &std::path::Path) -> Result<StateRootV1, BootstrapError> {
        let root = TrustedAuthorityRoot::open(path)
            .map_err(|_| BootstrapError("open trusted authority root"))?;
        root.revalidate()
            .map_err(|_| BootstrapError("revalidate trusted authority root"))?;
        let layout = StoreLayout::open(root.directory())
            .map_err(|_| BootstrapError("open authority store layout"))?;
        let _lock = layout
            .root_lock
            .lock_exclusive()
            .map_err(|_| BootstrapError("lock authority store root"))?;
        let material = system_material()?;
        bootstrap_locked(&layout, root.identity(), material, None)
    }

    pub(super) fn rotate_commitment_key(
        path: &std::path::Path,
    ) -> Result<StateRootV1, BootstrapError> {
        rotate_commitment_key_with(path, None, None)
    }

    pub(super) fn retire_commitment_key(
        path: &std::path::Path,
        retiring_key_id: &str,
    ) -> Result<StateRootV1, BootstrapError> {
        let material = system_material()?;
        retire_commitment_key_with(path, retiring_key_id, material.root_nonce, None)
    }

    pub(super) fn prepare_typed_object(
        path: &std::path::Path,
        reference: &AuthorityObjectRefV1,
        bytes: &[u8],
        context: Option<&ObjectVerificationContextV1>,
    ) -> Result<ObjectPublicationOutcomeV1, BootstrapError> {
        let material = system_material()?;
        prepare_typed_object_with(path, reference, bytes, context, material.key_nonce)
    }

    #[cfg(test)]
    pub(super) fn rotate_commitment_key_test(
        path: &std::path::Path,
        material: InitializationMaterialV1,
        stop: Option<KeyLifecycleCrashPointV1>,
    ) -> Result<StateRootV1, BootstrapError> {
        rotate_commitment_key_with(path, Some(material), stop)
    }

    #[cfg(test)]
    pub(super) fn retire_commitment_key_test(
        path: &std::path::Path,
        key_id: &str,
        root_nonce: [u8; 16],
        stop: Option<KeyLifecycleCrashPointV1>,
    ) -> Result<StateRootV1, BootstrapError> {
        retire_commitment_key_with(path, key_id, root_nonce, stop)
    }

    #[cfg(test)]
    pub(super) fn validate_resume_identity_test(
        resume: &ResumeHandleHashInputV1,
        session_id: &str,
        participant_id: &str,
        backend_id: &str,
        protocol: &str,
    ) -> Result<(), &'static str> {
        validate_resume_identity(resume, session_id, participant_id, backend_id, protocol)
            .map_err(|error| error.0)
    }

    #[cfg(test)]
    pub(super) fn publish_object_test(
        path: &std::path::Path,
        reference: &AuthorityObjectRefV1,
        bytes: &[u8],
        context: Option<&ObjectVerificationContextV1>,
        nonce_bytes: [u8; 16],
    ) -> Result<ObjectPublicationOutcomeV1, BootstrapError> {
        prepare_typed_object_with(path, reference, bytes, context, nonce_bytes)
    }

    fn prepare_typed_object_with(
        path: &std::path::Path,
        reference: &AuthorityObjectRefV1,
        bytes: &[u8],
        context: Option<&ObjectVerificationContextV1>,
        nonce_bytes: [u8; 16],
    ) -> Result<ObjectPublicationOutcomeV1, BootstrapError> {
        let root_handle = TrustedAuthorityRoot::open(path)
            .map_err(|_| BootstrapError("open trusted authority root"))?;
        root_handle
            .revalidate()
            .map_err(|_| BootstrapError("revalidate trusted authority root"))?;
        let layout = StoreLayout::open(root_handle.directory())
            .map_err(|_| BootstrapError("open authority store layout"))?;
        let _lock = layout
            .root_lock
            .lock_exclusive()
            .map_err(|_| BootstrapError("lock authority store root"))?;
        let (root, _) = layout
            .prepare_existing(root_handle.identity())
            .map_err(|_| BootstrapError("prepare authority store for object publication"))?;
        publish_or_join_orphan(&layout, &root, reference, bytes, context, nonce_bytes)
    }

    #[cfg(test)]
    pub(super) fn verify_orphan_test(
        path: &std::path::Path,
        reference: &AuthorityObjectRefV1,
        bytes: &[u8],
        context: Option<&ObjectVerificationContextV1>,
    ) -> Result<(), BootstrapError> {
        let root_handle = TrustedAuthorityRoot::open(path)
            .map_err(|_| BootstrapError("open trusted authority root"))?;
        let layout = StoreLayout::open(root_handle.directory())
            .map_err(|_| BootstrapError("open authority store layout"))?;
        let _lock = layout
            .root_lock
            .lock_exclusive()
            .map_err(|_| BootstrapError("lock authority store root"))?;
        let (root, _) = layout
            .prepare_existing(root_handle.identity())
            .map_err(|_| BootstrapError("prepare authority store for orphan verification"))?;
        if root.object_index.contains_key(&reference.ref_id) {
            return Err(BootstrapError("object is already authoritative"));
        }
        let kind_directory = layout
            .objects
            .open_directory(kind_slug(reference.object_kind))
            .map_err(|_| BootstrapError("open orphan kind directory"))?;
        let version_directory = kind_directory
            .open_directory(&format!("v{}", reference.schema_version))
            .map_err(|_| BootstrapError("open orphan version directory"))?;
        let existing = version_directory
            .open_file(&format!("{}.obj", reference.ref_id))
            .map_err(|_| BootstrapError("open retained orphan"))?
            .read_all()
            .map_err(|_| BootstrapError("read retained orphan"))?;
        if existing != bytes {
            return Err(BootstrapError("retained orphan bytes differ from retry"));
        }
        verify_object_bytes(&layout, &root, reference, &existing, context, true)
    }

    #[cfg(test)]
    pub(super) fn bootstrap_test(
        path: &std::path::Path,
        material: InitializationMaterialV1,
        stop: Option<InitializationCrashPointV1>,
    ) -> Result<StateRootV1, BootstrapError> {
        let root = TrustedAuthorityRoot::open(path)
            .map_err(|_| BootstrapError("open trusted authority root"))?;
        root.revalidate()
            .map_err(|_| BootstrapError("revalidate trusted authority root"))?;
        let layout = StoreLayout::open(root.directory())
            .map_err(|_| BootstrapError("open authority store layout"))?;
        let _lock = layout
            .root_lock
            .lock_exclusive()
            .map_err(|_| BootstrapError("lock authority store root"))?;
        bootstrap_locked(&layout, root.identity(), material, stop)
    }

    fn system_material() -> Result<InitializationMaterialV1, BootstrapError> {
        let mut store_entropy = [0_u8; 16];
        let mut key_entropy = [0_u8; 16];
        let mut marker_nonce = [0_u8; 16];
        let mut key_nonce = [0_u8; 16];
        let mut root_nonce = [0_u8; 16];
        let mut secret_key = [0_u8; 32];
        let mut random = rand::rngs::OsRng;
        random.fill_bytes(&mut store_entropy);
        random.fill_bytes(&mut key_entropy);
        random.fill_bytes(&mut marker_nonce);
        random.fill_bytes(&mut key_nonce);
        random.fill_bytes(&mut root_nonce);
        random.fill_bytes(&mut secret_key);
        let created_at =
            crate::execution::agent_runtime::host_session_authority::schema::TimestampV1::parse(
                chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Nanos, true),
            )
            .map_err(|_| BootstrapError("sample canonical initialization timestamp"))?;
        Ok(InitializationMaterialV1 {
            store_entropy,
            key_entropy,
            marker_nonce,
            key_nonce,
            root_nonce,
            secret_key,
            created_at,
        })
    }

    fn bootstrap_locked(
        layout: &StoreLayout<'_>,
        bootstrap_home: &crate::execution::agent_runtime::host_session_authority::schema::CanonicalDirectoryV1,
        material: InitializationMaterialV1,
        stop: Option<InitializationCrashPointV1>,
    ) -> Result<StateRootV1, BootstrapError> {
        let classified = layout
            .classify_locked_observed(bootstrap_home)
            .map_err(|_| BootstrapError("classify authority store"))?;
        match classified.classification {
            BootstrapClassificationV1::FreshAbsent => {
                initialize_fresh(layout, bootstrap_home, &classified.legacy, material, stop)
            }
            BootstrapClassificationV1::InitializationPending => {
                recover_pending(layout, bootstrap_home, &classified.legacy, material, stop)
            }
            BootstrapClassificationV1::ValidExisting => {
                let root = layout
                    .read_existing(bootstrap_home)
                    .map_err(|_| BootstrapError("read existing authority store"))?;
                layout
                    .reconcile_key_files(&root)
                    .map_err(|_| BootstrapError("reconcile commitment key files"))?;
                layout
                    .remove_matching_marker(&root)
                    .map_err(|_| BootstrapError("remove committed initialization marker"))?;
                Ok(root)
            }
            BootstrapClassificationV1::UnsupportedLegacyState => {
                Err(BootstrapError("pre-A1 authority state is unsupported"))
            }
            BootstrapClassificationV1::CorruptOrUnsupported => {
                Err(BootstrapError("authority store is corrupt or unsupported"))
            }
        }
    }

    fn initialize_fresh(
        layout: &StoreLayout<'_>,
        bootstrap_home: &crate::execution::agent_runtime::host_session_authority::schema::CanonicalDirectoryV1,
        legacy: &LegacyObservation,
        material: InitializationMaterialV1,
        stop: Option<InitializationCrashPointV1>,
    ) -> Result<StateRootV1, BootstrapError> {
        let marker = AuthorityStoreInitializationV1 {
            schema_version: 1,
            authority_store_id: store_id(material.store_entropy),
            bootstrap_home: bootstrap_home.clone(),
            initial_key_id: key_id(material.key_entropy),
            created_at: material.created_at.clone(),
        };
        marker
            .validate()
            .map_err(|_| BootstrapError("validate fresh initialization marker"))?;
        publish_marker(layout, &marker, material.marker_nonce)?;
        if stop == Some(InitializationCrashPointV1::Marker) {
            return Err(BootstrapError("injected initialization interruption"));
        }
        finish_pending(layout, legacy, marker, material, stop)
    }

    fn recover_pending(
        layout: &StoreLayout<'_>,
        bootstrap_home: &crate::execution::agent_runtime::host_session_authority::schema::CanonicalDirectoryV1,
        legacy: &LegacyObservation,
        material: InitializationMaterialV1,
        stop: Option<InitializationCrashPointV1>,
    ) -> Result<StateRootV1, BootstrapError> {
        let marker = layout
            .read_marker()
            .map_err(|_| BootstrapError("read pending initialization marker"))?;
        if &marker.bootstrap_home != bootstrap_home {
            return Err(BootstrapError("pending initialization home mismatch"));
        }
        finish_pending(layout, legacy, marker, material, stop)
    }

    fn finish_pending(
        layout: &StoreLayout<'_>,
        legacy: &LegacyObservation,
        marker: AuthorityStoreInitializationV1,
        material: InitializationMaterialV1,
        stop: Option<InitializationCrashPointV1>,
    ) -> Result<StateRootV1, BootstrapError> {
        let key_record = AuthorityStoreCommitmentKeyV1 {
            schema_version: 1,
            authority_store_id: marker.authority_store_id.clone(),
            key_id: marker.initial_key_id.clone(),
            algorithm: AuthorityStoreCommitmentAlgorithmV1::HmacSha256,
            created_at: marker.created_at.clone(),
            state: AuthorityStoreCommitmentKeyStateV1::Active,
        };
        layout.ensure_initial_key(&marker, material.key_nonce, material.secret_key)?;
        if stop == Some(InitializationCrashPointV1::Key) {
            return Err(BootstrapError("injected initialization interruption"));
        }
        legacy
            .revalidate(layout.bootstrap)
            .map_err(|_| BootstrapError("revalidate legacy authority state"))?;
        let root = StateRootV1 {
            schema_version: 1,
            authority_store_id: marker.authority_store_id.clone(),
            bootstrap_home: marker.bootstrap_home.clone(),
            root_revision: 1,
            active_commitment_key_id: marker.initial_key_id.clone(),
            commitment_key_registry: std::collections::BTreeMap::from([(
                key_record.key_id.clone(),
                key_record,
            )]),
            greenfield_namespace_certificate: GreenfieldNamespaceCertificateV1 {
                schema_version: 1,
                authority_store_id: marker.authority_store_id.clone(),
                bootstrap_home: marker.bootstrap_home.clone(),
                certified_at: marker.created_at.clone(),
            },
            session_namespace_map: std::collections::BTreeMap::new(),
            transition_intent_map: std::collections::BTreeMap::new(),
            issuer_request_index: std::collections::BTreeMap::new(),
            application_journal: std::collections::BTreeMap::new(),
            object_index: std::collections::BTreeMap::new(),
        };
        root.validate()
            .map_err(|_| BootstrapError("validate initial state root"))?;
        publish_root(layout, legacy, &root, material.root_nonce)?;
        if stop == Some(InitializationCrashPointV1::Root) {
            return Err(BootstrapError("injected initialization interruption"));
        }
        layout
            .authority
            .unlink_file(INIT_FILE)
            .map_err(|_| BootstrapError("remove initialization marker"))?;
        Ok(root)
    }

    fn publish_marker(
        layout: &StoreLayout<'_>,
        marker: &AuthorityStoreInitializationV1,
        nonce_bytes: [u8; 16],
    ) -> Result<(), BootstrapError> {
        let temp_name = TempNameV1::Init {
            authority_store_id: marker.authority_store_id.clone(),
            nonce: nonce(nonce_bytes),
        }
        .file_name();
        let bytes = canonical_json::to_vec(marker)
            .map_err(|_| BootstrapError("encode initialization marker"))?;
        let mut temp = layout
            .tmp
            .create_file(&temp_name)
            .map_err(|_| BootstrapError("create initialization marker temp"))?;
        temp.write_all(&bytes)
            .map_err(|_| BootstrapError("write initialization marker temp"))?;
        temp.sync()
            .map_err(|_| BootstrapError("sync initialization marker temp"))?;
        layout
            .tmp
            .rename_no_replace(&temp_name, temp, &layout.authority, INIT_FILE)
            .map_err(|_| BootstrapError("publish initialization marker"))
    }

    fn publish_root(
        layout: &StoreLayout<'_>,
        legacy: &LegacyObservation,
        root: &StateRootV1,
        nonce_bytes: [u8; 16],
    ) -> Result<(), BootstrapError> {
        let temp_name = TempNameV1::Root {
            root_revision: root.root_revision,
            nonce: nonce(nonce_bytes),
        }
        .file_name();
        let bytes = canonical_json::to_vec(root)
            .map_err(|_| BootstrapError("encode initial state root"))?;
        let mut temp = layout
            .tmp
            .create_file(&temp_name)
            .map_err(|_| BootstrapError("create state root temp"))?;
        temp.write_all(&bytes)
            .map_err(|_| BootstrapError("write state root temp"))?;
        temp.sync()
            .map_err(|_| BootstrapError("sync state root temp"))?;
        legacy
            .revalidate(layout.bootstrap)
            .map_err(|_| BootstrapError("revalidate legacy state before root publish"))?;
        layout
            .tmp
            .rename_no_replace(&temp_name, temp, &layout.authority, ROOT_FILE)
            .map_err(|_| BootstrapError("publish initial state root"))
    }

    fn publish_replacement_root(
        layout: &StoreLayout<'_>,
        legacy: &LegacyObservation,
        root: &StateRootV1,
        nonce_bytes: [u8; 16],
    ) -> Result<(), BootstrapError> {
        legacy
            .revalidate(layout.bootstrap)
            .map_err(|_| BootstrapError("revalidate legacy state before root temp"))?;
        let temp_name = TempNameV1::Root {
            root_revision: root.root_revision,
            nonce: nonce(nonce_bytes),
        }
        .file_name();
        let bytes = canonical_json::to_vec(root)
            .map_err(|_| BootstrapError("encode replacement state root"))?;
        let mut temp = layout
            .tmp
            .create_file(&temp_name)
            .map_err(|_| BootstrapError("create replacement state root temp"))?;
        temp.write_all(&bytes)
            .map_err(|_| BootstrapError("write replacement state root temp"))?;
        temp.sync()
            .map_err(|_| BootstrapError("sync replacement state root temp"))?;
        legacy
            .revalidate(layout.bootstrap)
            .map_err(|_| BootstrapError("revalidate legacy state before root replacement"))?;
        layout
            .tmp
            .rename_replace(&temp_name, temp, &layout.authority, ROOT_FILE)
            .map_err(|_| BootstrapError("replace authority state root"))
    }

    #[cfg(test)]
    pub(super) fn classify_diagnostic(
        path: &std::path::Path,
    ) -> Result<BootstrapClassificationV1, &'static str> {
        classify_checked(path).map_err(|error| error.0)
    }

    #[cfg(test)]
    pub(super) fn legacy_observation_mutation_test(
        path: &std::path::Path,
        mutation: LegacyMutationV1,
    ) -> Result<(), &'static str> {
        let root = TrustedAuthorityRoot::open(path).map_err(|_| "open trusted root")?;
        let layout = StoreLayout::open(root.directory()).map_err(|_| "open store layout")?;
        let _lock = layout
            .root_lock
            .lock_exclusive()
            .map_err(|_| "lock authority root")?;
        let observation = LegacyObservation::capture(layout.bootstrap)
            .map_err(|_| "capture legacy observation")?;
        let hub = path.join("run/agent-hub");
        match mutation {
            LegacyMutationV1::ReplaceSessions => {
                std::fs::rename(hub.join("sessions"), hub.join("sessions-replaced"))
                    .map_err(|_| "replace observed sessions directory")?;
                std::fs::create_dir(hub.join("sessions"))
                    .map_err(|_| "recreate sessions directory")?;
            }
            LegacyMutationV1::CreateMissingSessions => {
                std::fs::create_dir(hub.join("sessions"))
                    .map_err(|_| "create missing sessions directory")?;
            }
        }
        observation
            .revalidate(layout.bootstrap)
            .map_err(|error| error.0)
    }

    fn classify_checked(path: &std::path::Path) -> Result<BootstrapClassificationV1, StoreError> {
        let root = TrustedAuthorityRoot::open(path).map_err(|_| StoreError("open trusted root"))?;
        root.revalidate()
            .map_err(|_| StoreError("revalidate trusted root"))?;
        let layout = StoreLayout::open(root.directory())?;
        let _lock = layout
            .root_lock
            .lock_exclusive()
            .map_err(|_| StoreError("lock authority root"))?;
        layout.classify_locked(root.identity())
    }

    fn kind_from_slug(slug: &str) -> Option<AuthorityObjectKindV1> {
        Some(match slug {
            "agent-descriptor" => AuthorityObjectKindV1::AgentDescriptor,
            "retained-worker" => AuthorityObjectKindV1::RetainedWorker,
            "resume-handle" => AuthorityObjectKindV1::ResumeHandle,
            "policy" => AuthorityObjectKindV1::Policy,
            "host-attach-contract" => AuthorityObjectKindV1::HostAttachContract,
            "transition-transport-payload" => AuthorityObjectKindV1::TransitionTransportPayload,
            "transition-input" => AuthorityObjectKindV1::TransitionInput,
            "lease-token" => AuthorityObjectKindV1::LeaseToken,
            "application-result" => AuthorityObjectKindV1::ApplicationResult,
            "input-acceptance" => AuthorityObjectKindV1::InputAcceptance,
            "post-turn-completion" => AuthorityObjectKindV1::PostTurnCompletion,
            "terminal-handoff" => AuthorityObjectKindV1::TerminalHandoff,
            _ => return None,
        })
    }

    fn kind_slug(kind: AuthorityObjectKindV1) -> &'static str {
        match kind {
            AuthorityObjectKindV1::AgentDescriptor => "agent-descriptor",
            AuthorityObjectKindV1::RetainedWorker => "retained-worker",
            AuthorityObjectKindV1::ResumeHandle => "resume-handle",
            AuthorityObjectKindV1::Policy => "policy",
            AuthorityObjectKindV1::HostAttachContract => "host-attach-contract",
            AuthorityObjectKindV1::TransitionTransportPayload => "transition-transport-payload",
            AuthorityObjectKindV1::TransitionInput => "transition-input",
            AuthorityObjectKindV1::LeaseToken => "lease-token",
            AuthorityObjectKindV1::ApplicationResult => "application-result",
            AuthorityObjectKindV1::InputAcceptance => "input-acceptance",
            AuthorityObjectKindV1::PostTurnCompletion => "post-turn-completion",
            AuthorityObjectKindV1::TerminalHandoff => "terminal-handoff",
        }
    }

    #[derive(Clone, Copy, Debug)]
    struct StoreError(&'static str);

    impl fmt::Display for StoreError {
        fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
            formatter.write_str(self.0)
        }
    }

    impl std::error::Error for StoreError {}
}

#[cfg(not(any(target_os = "linux", target_os = "macos")))]
mod platform {
    use super::{
        BootstrapClassificationV1, BootstrapError, ObjectPublicationOutcomeV1,
        ObjectVerificationContextV1,
    };
    use crate::execution::agent_runtime::host_session_authority::schema::AuthorityObjectRefV1;
    use crate::execution::agent_runtime::host_session_authority::store_schema::StateRootV1;

    pub(super) fn classify(_path: &std::path::Path) -> BootstrapClassificationV1 {
        BootstrapClassificationV1::CorruptOrUnsupported
    }

    pub(super) fn bootstrap(_path: &std::path::Path) -> Result<StateRootV1, BootstrapError> {
        Err(BootstrapError(
            "authority store is unsupported on this platform",
        ))
    }

    pub(super) fn rotate_commitment_key(
        _path: &std::path::Path,
    ) -> Result<StateRootV1, BootstrapError> {
        Err(BootstrapError(
            "authority store is unsupported on this platform",
        ))
    }

    pub(super) fn retire_commitment_key(
        _path: &std::path::Path,
        _key_id: &str,
    ) -> Result<StateRootV1, BootstrapError> {
        Err(BootstrapError(
            "authority store is unsupported on this platform",
        ))
    }

    pub(super) fn prepare_typed_object(
        _path: &std::path::Path,
        _reference: &AuthorityObjectRefV1,
        _bytes: &[u8],
        _context: Option<&ObjectVerificationContextV1>,
    ) -> Result<ObjectPublicationOutcomeV1, BootstrapError> {
        Err(BootstrapError(
            "authority store is unsupported on this platform",
        ))
    }
}

#[cfg(all(test, any(target_os = "linux", target_os = "macos")))]
#[path = "store_tests.rs"]
mod tests;
