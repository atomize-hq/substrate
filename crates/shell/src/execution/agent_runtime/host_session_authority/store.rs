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

pub(crate) fn rotate_commitment_key(
    path: &Path,
    expected_root_revision: u64,
) -> Result<StateRootV1, BootstrapError> {
    platform::rotate_commitment_key(path, expected_root_revision)
}

pub(crate) fn retire_commitment_key(
    path: &Path,
    key_id: &str,
    expected_root_revision: u64,
) -> Result<StateRootV1, BootstrapError> {
    platform::retire_commitment_key(path, key_id, expected_root_revision)
}

pub(crate) fn prepare_typed_object(
    path: &Path,
    expected_root_revision: u64,
    reference: &crate::execution::agent_runtime::host_session_authority::schema::AuthorityObjectRefV1,
    bytes: &[u8],
    context: Option<&ObjectVerificationContextV1>,
) -> Result<ObjectPublicationOutcomeV1, BootstrapError> {
    platform::prepare_typed_object(path, expected_root_revision, reference, bytes, context)
}

pub(crate) fn read_root(path: &Path) -> Result<StateRootV1, BootstrapError> {
    platform::read_root(path)
}

pub(crate) fn compare_and_swap_root(
    path: &Path,
    expected: &ExpectedRevisionsV1,
    proposed: &StateRootV1,
) -> Result<TransactionCommitOutcomeV1, BootstrapError> {
    platform::compare_and_swap_root(path, expected, proposed)
}

pub(crate) fn legacy_writer_guard(path: &Path) -> Result<LegacyWriterGuard, BootstrapError> {
    begin_legacy_state_store_transaction(path)
}

pub(crate) fn begin_legacy_state_store_transaction(
    path: &Path,
) -> Result<LegacyStateStoreTransactionV1, BootstrapError> {
    platform::begin_legacy_state_store_transaction(path)
}

#[cfg(all(test, any(target_os = "linux", target_os = "macos")))]
fn legacy_transaction_admission_handoff_test(
    path: &Path,
    after_classification: impl FnOnce(),
) -> Result<(), BootstrapError> {
    platform::legacy_transaction_admission_handoff_test(path, after_classification)
}

pub(crate) use platform::LegacyStateStoreTransactionV1;
pub(crate) type LegacyWriterGuard = LegacyStateStoreTransactionV1;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum LegacyStateStoreCollectionV1 {
    Sessions,
    Participants,
    Handles,
    HostInbox,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct LegacyStateStoreDirectoryEntryV1 {
    pub(crate) name: String,
    pub(crate) is_directory: bool,
    pub(crate) bytes: Option<Vec<u8>>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ExpectedAuthorityRevisionV1 {
    pub(crate) orchestration_session_id: String,
    pub(crate) authority_revision: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ExpectedRevisionsV1 {
    pub(crate) root_revision: u64,
    pub(crate) authority: Option<ExpectedAuthorityRevisionV1>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum TransactionCommitOutcomeV1 {
    Committed(StateRootV1),
    JoinedExact(StateRootV1),
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
    Reconciled,
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
        BootstrapClassificationV1, BootstrapError, ExpectedRevisionsV1, InitializationCrashPointV1,
        InitializationMaterialV1, KeyLifecycleCrashPointV1, LegacyStateStoreCollectionV1,
        ObjectPublicationOutcomeV1, ObjectVerificationContextV1, TransactionCommitOutcomeV1,
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
        TrustedOwnedFileLock,
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
    use layout::{create_or_join_lock, validate_resume_identity};
    use layout::{LockedClassification, StoreLayout, StoreLayoutLockScope};
    #[path = "key_lifecycle.rs"]
    mod key_lifecycle;
    use key_lifecycle::{retire_commitment_key_with, rotate_commitment_key_with};
    #[path = "object_persistence.rs"]
    mod object_persistence;
    use object_persistence::publish_or_join_orphan;
    use object_persistence::validate_orphan_candidate;
    use object_persistence::verify_object_bytes;
    #[path = "reachability.rs"]
    mod reachability;
    use reachability::{add_expected_ref, collect_reachable_objects};
    #[path = "transaction.rs"]
    mod transaction;
    #[cfg(test)]
    use transaction::retain_classified_legacy_directories_test;
    pub(crate) use transaction::LegacyStateStoreTransactionV1;
    use transaction::{
        begin_legacy_state_store_transaction as begin_legacy_transaction,
        compare_and_swap_root_with, with_existing_semantic_preflight, with_semantic_preflight,
        SemanticPreflightMode,
    };

    pub(super) fn classify(path: &std::path::Path) -> BootstrapClassificationV1 {
        classify_checked(path).unwrap_or(BootstrapClassificationV1::CorruptOrUnsupported)
    }

    pub(super) fn bootstrap(path: &std::path::Path) -> Result<StateRootV1, BootstrapError> {
        with_semantic_preflight(
            path,
            SemanticPreflightMode::AuthorityOperation,
            |layout, bootstrap_home, observed, _lock| {
                let material = match observed.classification {
                    BootstrapClassificationV1::FreshAbsent
                    | BootstrapClassificationV1::InitializationPending => Some(system_material()?),
                    BootstrapClassificationV1::ValidExisting
                    | BootstrapClassificationV1::UnsupportedLegacyState
                    | BootstrapClassificationV1::CorruptOrUnsupported => None,
                };
                bootstrap_locked(layout, bootstrap_home, observed, material, None)
            },
        )
    }

    pub(super) fn rotate_commitment_key(
        path: &std::path::Path,
        expected_root_revision: u64,
    ) -> Result<StateRootV1, BootstrapError> {
        rotate_commitment_key_with(path, expected_root_revision, None, None)
    }

    pub(super) fn retire_commitment_key(
        path: &std::path::Path,
        retiring_key_id: &str,
        expected_root_revision: u64,
    ) -> Result<StateRootV1, BootstrapError> {
        let material = system_material()?;
        retire_commitment_key_with(
            path,
            retiring_key_id,
            expected_root_revision,
            material.root_nonce,
            None,
        )
    }

    pub(super) fn prepare_typed_object(
        path: &std::path::Path,
        expected_root_revision: u64,
        reference: &AuthorityObjectRefV1,
        bytes: &[u8],
        context: Option<&ObjectVerificationContextV1>,
    ) -> Result<ObjectPublicationOutcomeV1, BootstrapError> {
        let material = system_material()?;
        prepare_typed_object_with(
            path,
            expected_root_revision,
            reference,
            bytes,
            context,
            material.key_nonce,
        )
    }

    pub(super) fn read_root(path: &std::path::Path) -> Result<StateRootV1, BootstrapError> {
        with_existing_semantic_preflight(path, |transaction| Ok(transaction.root.clone()))
    }

    pub(super) fn compare_and_swap_root(
        path: &std::path::Path,
        expected: &ExpectedRevisionsV1,
        proposed: &StateRootV1,
    ) -> Result<TransactionCommitOutcomeV1, BootstrapError> {
        compare_and_swap_root_with(path, expected, proposed, system_material()?.root_nonce)
    }

    pub(super) fn begin_legacy_state_store_transaction(
        path: &std::path::Path,
    ) -> Result<LegacyStateStoreTransactionV1, BootstrapError> {
        begin_legacy_transaction(path)
    }

    #[cfg(test)]
    pub(super) fn legacy_transaction_admission_handoff_test(
        path: &std::path::Path,
        after_classification: impl FnOnce(),
    ) -> Result<(), BootstrapError> {
        let root = TrustedAuthorityRoot::open(path)
            .map_err(|_| BootstrapError("open test legacy StateStore root"))?;
        let observation = LegacyObservation::capture(root.directory())
            .map_err(|_| BootstrapError("capture test legacy StateStore observation"))?;
        after_classification();
        retain_classified_legacy_directories_test(root.directory(), &observation)
    }

    #[cfg(test)]
    pub(super) fn rotate_commitment_key_test(
        path: &std::path::Path,
        material: InitializationMaterialV1,
        stop: Option<KeyLifecycleCrashPointV1>,
    ) -> Result<StateRootV1, BootstrapError> {
        let expected_root_revision = read_root(path)?.root_revision;
        rotate_commitment_key_with(path, expected_root_revision, Some(material), stop)
    }

    #[cfg(test)]
    pub(super) fn retire_commitment_key_test(
        path: &std::path::Path,
        key_id: &str,
        root_nonce: [u8; 16],
        stop: Option<KeyLifecycleCrashPointV1>,
    ) -> Result<StateRootV1, BootstrapError> {
        let expected_root_revision = read_root(path)?.root_revision;
        retire_commitment_key_with(path, key_id, expected_root_revision, root_nonce, stop)
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
    pub(super) fn validate_exact_retry_expectation_test(
        expected: &ExpectedRevisionsV1,
        proposed: &StateRootV1,
    ) -> Result<(), BootstrapError> {
        transaction::validate_exact_retry_expectation_test(expected, proposed)
    }

    #[cfg(test)]
    pub(super) fn publish_key_lifecycle_candidate_test(
        path: &std::path::Path,
        expected_root_revision: u64,
        candidate: &StateRootV1,
        nonce_bytes: [u8; 16],
    ) -> Result<(), BootstrapError> {
        with_existing_semantic_preflight(path, |transaction| {
            transaction.reconcile()?;
            publish_replacement_root(
                transaction.layout,
                transaction.trusted_root,
                &transaction.legacy,
                candidate,
                nonce_bytes,
                || transaction.validate_publication_candidate(expected_root_revision, candidate),
            )
        })
    }

    #[cfg(test)]
    pub(super) fn publish_object_test(
        path: &std::path::Path,
        reference: &AuthorityObjectRefV1,
        bytes: &[u8],
        context: Option<&ObjectVerificationContextV1>,
        nonce_bytes: [u8; 16],
    ) -> Result<ObjectPublicationOutcomeV1, BootstrapError> {
        let expected_root_revision = read_root(path)?.root_revision;
        prepare_typed_object_with(
            path,
            expected_root_revision,
            reference,
            bytes,
            context,
            nonce_bytes,
        )
    }

    fn prepare_typed_object_with(
        path: &std::path::Path,
        expected_root_revision: u64,
        reference: &AuthorityObjectRefV1,
        bytes: &[u8],
        context: Option<&ObjectVerificationContextV1>,
        nonce_bytes: [u8; 16],
    ) -> Result<ObjectPublicationOutcomeV1, BootstrapError> {
        with_existing_semantic_preflight(path, |transaction| {
            transaction.require_expected_root(expected_root_revision)?;
            validate_orphan_candidate(
                transaction.layout,
                &transaction.root,
                reference,
                bytes,
                context,
            )?;
            transaction.reconcile()?;
            publish_or_join_orphan(
                transaction.layout,
                &transaction.root,
                reference,
                bytes,
                context,
                nonce_bytes,
            )
        })
    }

    #[cfg(test)]
    pub(super) fn verify_orphan_test(
        path: &std::path::Path,
        reference: &AuthorityObjectRefV1,
        bytes: &[u8],
        context: Option<&ObjectVerificationContextV1>,
    ) -> Result<(), BootstrapError> {
        with_existing_semantic_preflight(path, |transaction| {
            if transaction
                .root
                .object_index
                .contains_key(&reference.ref_id)
            {
                return Err(BootstrapError("object is already authoritative"));
            }
            let kind_directory = transaction
                .layout
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
            verify_object_bytes(
                transaction.layout,
                &transaction.root,
                reference,
                &existing,
                context,
                true,
            )
        })
    }

    #[cfg(test)]
    pub(super) fn bootstrap_test(
        path: &std::path::Path,
        material: InitializationMaterialV1,
        stop: Option<InitializationCrashPointV1>,
    ) -> Result<StateRootV1, BootstrapError> {
        with_semantic_preflight(
            path,
            SemanticPreflightMode::AuthorityOperation,
            |layout, bootstrap_home, observed, _lock| {
                bootstrap_locked(layout, bootstrap_home, observed, Some(material), stop)
            },
        )
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
        observed: LockedClassification,
        material: Option<InitializationMaterialV1>,
        stop: Option<InitializationCrashPointV1>,
    ) -> Result<StateRootV1, BootstrapError> {
        match observed.classification {
            BootstrapClassificationV1::FreshAbsent => {
                let material = material.ok_or(BootstrapError(
                    "fresh initialization material was not generated under lock",
                ))?;
                layout
                    .reconcile_temps()
                    .map_err(|_| BootstrapError("reconcile validated bootstrap temps"))?;
                initialize_fresh(layout, bootstrap_home, &observed.legacy, material, stop)
            }
            BootstrapClassificationV1::InitializationPending => {
                let material = material.ok_or(BootstrapError(
                    "pending recovery material was not generated under lock",
                ))?;
                layout
                    .reconcile_temps()
                    .map_err(|_| BootstrapError("reconcile validated bootstrap temps"))?;
                recover_pending(layout, bootstrap_home, &observed.legacy, material, stop)
            }
            BootstrapClassificationV1::ValidExisting => {
                let root = observed
                    .root
                    .ok_or(BootstrapError("semantic preflight omitted existing root"))?;
                layout
                    .reconcile_after_preflight(&root)
                    .map_err(|_| BootstrapError("reconcile valid existing authority store"))?;
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
        trusted_root: &TrustedAuthorityRoot,
        legacy: &LegacyObservation,
        root: &StateRootV1,
        nonce_bytes: [u8; 16],
        validate_candidate: impl FnOnce() -> Result<(), BootstrapError>,
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
        validate_candidate()?;
        trusted_root
            .revalidate()
            .map_err(|_| BootstrapError("revalidate trusted root before root replacement"))?;
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

    #[cfg(test)]
    pub(super) fn first_lock_creation_join_test(
        path: &std::path::Path,
    ) -> Result<(), &'static str> {
        let root = TrustedAuthorityRoot::open(path).map_err(|_| "open trusted root")?;
        let authority = root
            .directory()
            .create_directory(AUTHORITY_DIRECTORY)
            .map_err(|_| "create authority directory")?;
        let lock = authority
            .create_directory("lock")
            .map_err(|_| "create lock directory")?;
        let first = lock
            .create_file(ROOT_LOCK_FILE)
            .map_err(|_| "create first lock file")?;
        first.sync().map_err(|_| "sync first lock file")?;
        lock.sync().map_err(|_| "sync first lock directory")?;

        let joined = create_or_join_lock(&lock).map_err(|_| "join first lock creation")?;
        drop(
            joined
                .lock_exclusive_owned()
                .map_err(|_| "lock joined file")?,
        );
        Ok(())
    }

    fn classify_checked(
        path: &std::path::Path,
    ) -> Result<BootstrapClassificationV1, BootstrapError> {
        with_semantic_preflight(
            path,
            SemanticPreflightMode::AuthorityOperation,
            |layout, bootstrap_home, observed, _lock| {
                if observed.classification == BootstrapClassificationV1::UnsupportedLegacyState {
                    return Ok(observed.classification);
                }
                if let Some(root) = observed.root.as_ref() {
                    layout
                        .reconcile_after_preflight(root)
                        .map_err(|_| BootstrapError("reconcile valid existing classification"))?;
                } else {
                    layout
                        .reconcile_temps()
                        .map_err(|_| BootstrapError("reconcile validated classification temps"))?;
                }
                layout
                    .semantic_preflight(bootstrap_home)
                    .map(|revalidated| revalidated.classification)
                    .map_err(|_| BootstrapError("revalidate classified authority store"))
            },
        )
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

    pub(crate) struct LegacyStateStoreTransactionV1;

    impl LegacyStateStoreTransactionV1 {
        pub(crate) fn read_file(
            &self,
            _collection: super::LegacyStateStoreCollectionV1,
            _descendants: &[&str],
        ) -> Result<Option<Vec<u8>>, BootstrapError> {
            Err(BootstrapError(
                "legacy StateStore transactions are unsupported on this platform",
            ))
        }

        pub(crate) fn write_file(
            &self,
            _collection: super::LegacyStateStoreCollectionV1,
            _descendants: &[&str],
            _bytes: &[u8],
            _nonce_bytes: [u8; 16],
        ) -> Result<(), BootstrapError> {
            Err(BootstrapError(
                "legacy StateStore transactions are unsupported on this platform",
            ))
        }

        pub(crate) fn remove_file(
            &self,
            _collection: super::LegacyStateStoreCollectionV1,
            _descendants: &[&str],
        ) -> Result<bool, BootstrapError> {
            Err(BootstrapError(
                "legacy StateStore transactions are unsupported on this platform",
            ))
        }

        pub(crate) fn read_directory(
            &mut self,
            _collection: super::LegacyStateStoreCollectionV1,
            _descendants: &[&str],
        ) -> Result<Vec<super::LegacyStateStoreDirectoryEntryV1>, BootstrapError> {
            Err(BootstrapError(
                "legacy StateStore transactions are unsupported on this platform",
            ))
        }

        pub(crate) fn verify_physical_root(
            &self,
            _expected: &crate::execution::agent_runtime::host_session_authority::schema::CanonicalDirectoryV1,
        ) -> Result<(), BootstrapError> {
            Err(BootstrapError(
                "legacy StateStore transactions are unsupported on this platform",
            ))
        }

        pub(crate) fn finish(self) -> Result<(), BootstrapError> {
            Err(BootstrapError(
                "legacy StateStore transactions are unsupported on this platform",
            ))
        }
    }

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
        _expected_root_revision: u64,
    ) -> Result<StateRootV1, BootstrapError> {
        Err(BootstrapError(
            "authority store is unsupported on this platform",
        ))
    }

    pub(super) fn retire_commitment_key(
        _path: &std::path::Path,
        _key_id: &str,
        _expected_root_revision: u64,
    ) -> Result<StateRootV1, BootstrapError> {
        Err(BootstrapError(
            "authority store is unsupported on this platform",
        ))
    }

    pub(super) fn prepare_typed_object(
        _path: &std::path::Path,
        _expected_root_revision: u64,
        _reference: &AuthorityObjectRefV1,
        _bytes: &[u8],
        _context: Option<&ObjectVerificationContextV1>,
    ) -> Result<ObjectPublicationOutcomeV1, BootstrapError> {
        Err(BootstrapError(
            "authority store is unsupported on this platform",
        ))
    }

    pub(super) fn read_root(_path: &std::path::Path) -> Result<StateRootV1, BootstrapError> {
        Err(BootstrapError(
            "authority store is unsupported on this platform",
        ))
    }

    pub(super) fn compare_and_swap_root(
        _path: &std::path::Path,
        _expected: &super::ExpectedRevisionsV1,
        _proposed: &StateRootV1,
    ) -> Result<super::TransactionCommitOutcomeV1, BootstrapError> {
        Err(BootstrapError(
            "authority store is unsupported on this platform",
        ))
    }

    pub(super) fn begin_legacy_state_store_transaction(
        _path: &std::path::Path,
    ) -> Result<LegacyStateStoreTransactionV1, BootstrapError> {
        Err(BootstrapError(
            "authority store is unsupported on this platform",
        ))
    }
}

#[cfg(all(test, any(target_os = "linux", target_os = "macos")))]
#[path = "store_tests.rs"]
mod tests;
