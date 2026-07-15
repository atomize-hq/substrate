use std::fmt;
use std::path::Path;

use super::schema::{CanonicalDirectoryV1, TimestampV1};
use super::store_schema::{StateRootV1, StateRootV2};
use super::trusted_fs::TrustedAuthorityRoot;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum BootstrapClassificationV1 {
    FreshAbsent,
    InitializationPending,
    ValidExisting,
    UnsupportedLegacyState,
    CorruptOrUnsupported,
}

#[cfg(test)]
pub(crate) fn classify(path: &Path) -> BootstrapClassificationV1 {
    platform::classify(path)
}

pub(super) fn classify_opened(root: &TrustedAuthorityRoot) -> BootstrapClassificationV1 {
    platform::classify_opened(root)
}

pub(crate) use platform::RetainedWorkerAdmissionStorageV1;

pub(crate) fn retained_worker_admission_storage_for_authority(
    authority: &super::facade::HostSessionAuthority,
) -> Result<RetainedWorkerAdmissionStorageV1, BootstrapError> {
    platform::retained_worker_admission_storage_opened(authority.trusted_root())
}

#[cfg(test)]
pub(crate) fn bootstrap(path: &Path) -> Result<StateRootV1, BootstrapError> {
    platform::bootstrap(path)
}

pub(super) fn bootstrap_opened(root: &TrustedAuthorityRoot) -> Result<StateRootV1, BootstrapError> {
    platform::bootstrap_opened(root)
}

pub(super) fn upgrade_greenfield_root_opened(
    root: &TrustedAuthorityRoot,
) -> Result<RootUpgradeOutcomeV1, BootstrapError> {
    platform::upgrade_greenfield_root_opened(root)
}

pub(super) fn read_opened_root_v2(
    root: &TrustedAuthorityRoot,
) -> Result<StateRootV2, BootstrapError> {
    platform::read_opened_root_v2(root)
}

pub(super) fn prepare_generated_object_v2_opened(
    root: &TrustedAuthorityRoot,
    expected_root_revision: u64,
    object_kind: crate::execution::agent_runtime::host_session_authority::schema::AuthorityObjectKindV1,
    bytes: &[u8],
    context: Option<&ObjectVerificationContextV1>,
) -> Result<GeneratedObjectV1, BootstrapError> {
    platform::prepare_generated_object_v2_opened(
        root,
        expected_root_revision,
        object_kind,
        bytes,
        context,
    )
}

pub(super) fn allocate_sensitive_object_ref_v2_opened(
    root: &TrustedAuthorityRoot,
    expected_root_revision: u64,
    object_kind: crate::execution::agent_runtime::host_session_authority::schema::AuthorityObjectKindV1,
    bytes: &[u8],
    context: &ObjectVerificationContextV1,
) -> Result<
    crate::execution::agent_runtime::host_session_authority::schema::AuthorityObjectRefV1,
    BootstrapError,
> {
    platform::allocate_sensitive_object_ref_v2_opened(
        root,
        expected_root_revision,
        object_kind,
        bytes,
        context,
    )
}

pub(super) fn prepare_typed_object_v2_opened(
    root: &TrustedAuthorityRoot,
    expected_root_revision: u64,
    reference: &crate::execution::agent_runtime::host_session_authority::schema::AuthorityObjectRefV1,
    bytes: &[u8],
    context: Option<&ObjectVerificationContextV1>,
) -> Result<ObjectPublicationOutcomeV1, BootstrapError> {
    platform::prepare_typed_object_v2_opened(
        root,
        expected_root_revision,
        reference,
        bytes,
        context,
    )
}

pub(super) fn read_typed_object_v2_opened(
    root: &TrustedAuthorityRoot,
    expected_root_revision: u64,
    reference: &crate::execution::agent_runtime::host_session_authority::schema::AuthorityObjectRefV1,
    context: Option<&ObjectVerificationContextV1>,
) -> Result<Vec<u8>, BootstrapError> {
    platform::read_typed_object_v2_opened(root, expected_root_revision, reference, context)
}

pub(super) fn commit_v2_root_exact_current_opened(
    root: &TrustedAuthorityRoot,
    exact_current: &StateRootV2,
    proposed: &StateRootV2,
    publication_guard: impl FnMut() -> Result<(), BootstrapError>,
) -> Result<StateRootV2, BootstrapError> {
    platform::commit_v2_root_exact_current_opened(root, exact_current, proposed, publication_guard)
}

pub(super) fn reserve_retained_worker_registration_opened(
    root: &TrustedAuthorityRoot,
    input: &RetainedWorkerReservationInputV1,
    build_worker: impl Fn(
        &crate::execution::agent_runtime::host_session_authority::schema::AuthorityObjectRefV1,
        &crate::execution::agent_runtime::host_session_authority::schema::AuthorityObjectRefV1,
        &crate::execution::agent_runtime::host_session_authority::schema::AuthorityObjectRefV1,
        &crate::execution::agent_runtime::host_session_authority::schema::WorldBindingV1,
    ) -> Result<Vec<u8>, &'static str>,
) -> Result<RetainedWorkerReservationV1, BootstrapError> {
    platform::reserve_retained_worker_registration_opened(root, input, build_worker)
}

pub(super) fn publish_reserved_retained_object_opened(
    root: &TrustedAuthorityRoot,
    reserved: &RetainedWorkerReservationV1,
    reference: &crate::execution::agent_runtime::host_session_authority::schema::AuthorityObjectRefV1,
    bytes: &[u8],
) -> Result<ObjectPublicationOutcomeV1, BootstrapError> {
    platform::publish_reserved_retained_object_opened(root, reserved, reference, bytes)
}

pub(super) fn apply_reserved_retained_worker_registration_opened(
    root: &TrustedAuthorityRoot,
    reserved: &RetainedWorkerReservationV1,
) -> Result<RetainedWorkerApplicationV1, BootstrapError> {
    platform::apply_reserved_retained_worker_registration_opened(root, reserved)
}

#[cfg(test)]
pub(super) fn apply_reserved_retained_worker_registration_with_crash_point_opened(
    root: &TrustedAuthorityRoot,
    reserved: &RetainedWorkerReservationV1,
    crash_point: RetainedApplicationCrashPointV1,
) -> Result<RetainedWorkerApplicationV1, BootstrapError> {
    platform::apply_reserved_retained_worker_registration_with_crash_point_opened(
        root,
        reserved,
        crash_point,
    )
}

#[cfg(test)]
pub(super) fn reserve_retained_worker_registration_at_opened(
    root: &TrustedAuthorityRoot,
    input: &RetainedWorkerReservationInputV1,
    registered_at: crate::execution::agent_runtime::host_session_authority::schema::TimestampV1,
    crash_point: Option<RetainedReservationCrashPointV1>,
    build_worker: impl Fn(
        &crate::execution::agent_runtime::host_session_authority::schema::AuthorityObjectRefV1,
        &crate::execution::agent_runtime::host_session_authority::schema::AuthorityObjectRefV1,
        &crate::execution::agent_runtime::host_session_authority::schema::AuthorityObjectRefV1,
        &crate::execution::agent_runtime::host_session_authority::schema::WorldBindingV1,
    ) -> Result<Vec<u8>, &'static str>,
) -> Result<RetainedWorkerReservationV1, BootstrapError> {
    platform::reserve_retained_worker_registration_at_opened(
        root,
        input,
        registered_at,
        crash_point,
        build_worker,
    )
}

#[cfg(test)]
pub(crate) fn rotate_commitment_key(
    path: &Path,
    expected_root_revision: u64,
) -> Result<StateRootV1, BootstrapError> {
    platform::rotate_commitment_key(path, expected_root_revision)
}

#[cfg(test)]
pub(crate) fn retire_commitment_key(
    path: &Path,
    key_id: &str,
    expected_root_revision: u64,
) -> Result<StateRootV1, BootstrapError> {
    platform::retire_commitment_key(path, key_id, expected_root_revision)
}

#[cfg(test)]
pub(crate) fn prepare_typed_object(
    path: &Path,
    expected_root_revision: u64,
    reference: &crate::execution::agent_runtime::host_session_authority::schema::AuthorityObjectRefV1,
    bytes: &[u8],
    context: Option<&ObjectVerificationContextV1>,
) -> Result<ObjectPublicationOutcomeV1, BootstrapError> {
    platform::prepare_typed_object(path, expected_root_revision, reference, bytes, context)
}

#[cfg(test)]
pub(crate) fn reserved_object_ref_id_is_globally_absent_test(
    path: &Path,
    ref_id: &str,
) -> Result<bool, BootstrapError> {
    platform::reserved_object_ref_id_is_globally_absent_test(path, ref_id)
}

#[cfg(test)]
pub(crate) fn prepare_typed_object_v2_test(
    path: &Path,
    expected_root_revision: u64,
    reference: &crate::execution::agent_runtime::host_session_authority::schema::AuthorityObjectRefV1,
    bytes: &[u8],
) -> Result<ObjectPublicationOutcomeV1, BootstrapError> {
    let root = TrustedAuthorityRoot::open(path)
        .map_err(|_| BootstrapError("open strict V2 object test root"))?;
    platform::prepare_typed_object_v2_opened(&root, expected_root_revision, reference, bytes, None)
}

#[cfg(test)]
pub(crate) fn retained_reservation_authority_matches_test(
    root: &StateRootV2,
    orchestration_session_id: &str,
    expected_authority_store_id: &str,
    expected_authority_revision: u64,
    expected_authority_commitment: &crate::execution::agent_runtime::host_session_authority::schema::AuthorityObjectCommitmentV1,
) -> Result<(), BootstrapError> {
    platform::retained_reservation_authority_matches_test(
        root,
        orchestration_session_id,
        expected_authority_store_id,
        expected_authority_revision,
        expected_authority_commitment,
    )
}

pub(super) fn prepare_typed_object_opened(
    root: &TrustedAuthorityRoot,
    expected_root_revision: u64,
    reference: &crate::execution::agent_runtime::host_session_authority::schema::AuthorityObjectRefV1,
    bytes: &[u8],
    context: Option<&ObjectVerificationContextV1>,
) -> Result<ObjectPublicationOutcomeV1, BootstrapError> {
    platform::prepare_typed_object_opened(root, expected_root_revision, reference, bytes, context)
}

#[cfg(test)]
pub(crate) fn read_root(path: &Path) -> Result<StateRootV1, BootstrapError> {
    platform::read_root(path)
}

pub(super) fn read_opened_root(root: &TrustedAuthorityRoot) -> Result<StateRootV1, BootstrapError> {
    platform::read_opened_root(root)
}

#[cfg(test)]
pub(crate) fn compare_and_swap_root(
    path: &Path,
    expected: &ExpectedRevisionsV1,
    proposed: &StateRootV1,
) -> Result<TransactionCommitOutcomeV1, BootstrapError> {
    platform::compare_and_swap_root(path, expected, proposed)
}

pub(super) fn compare_and_swap_opened_root(
    root: &TrustedAuthorityRoot,
    expected: &ExpectedRevisionsV1,
    proposed: &StateRootV1,
) -> Result<TransactionCommitOutcomeV1, BootstrapError> {
    platform::compare_and_swap_opened_root(root, expected, proposed)
}

pub(super) fn compare_and_swap_opened_root_exact_current(
    root: &TrustedAuthorityRoot,
    exact_current: &StateRootV1,
    expected: &ExpectedRevisionsV1,
    proposed: &StateRootV1,
) -> Result<TransactionCommitOutcomeV1, BootstrapError> {
    platform::compare_and_swap_opened_root_exact_current(root, exact_current, expected, proposed)
}

pub(crate) fn legacy_writer_guard(path: &Path) -> Result<LegacyWriterGuard, BootstrapError> {
    begin_legacy_state_store_transaction(path)
}

pub(crate) fn legacy_writer_guard_for_identity(
    path: &Path,
    expected: &CanonicalDirectoryV1,
) -> Result<LegacyWriterGuard, BootstrapError> {
    platform::begin_legacy_state_store_transaction_for_identity(path, expected)
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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum GreenfieldUpgradeCrashPointV1 {
    BeforeRootPublication,
    FinalRevalidationMismatch,
    AfterRootPublication,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum RootUpgradeOutcomeV1 {
    Upgraded(super::store_schema::StateRootV2),
    JoinedExact(super::store_schema::StateRootV2),
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

impl BootstrapError {
    pub(super) fn transition_guard() -> Self {
        Self("transition publication guard failed")
    }

    pub(crate) fn retained_admission_semantic() -> Self {
        Self("retained admission semantic validation failed")
    }

    pub(crate) fn retained_admission_crash() -> Self {
        Self("injected retained admission initialization crash")
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ObjectVerificationContextV1 {
    pub(crate) intent_id: String,
    pub(crate) run_id: String,
    pub(crate) parent_intent: Option<VersionedObjectVerificationParentIntentV1>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum VersionedObjectVerificationParentIntentV1 {
    V1(
        Box<
            crate::execution::agent_runtime::host_session_authority::store_schema::HostSessionTransitionIntentV1,
        >,
    ),
    V2(
        Box<
            crate::execution::agent_runtime::host_session_authority::store_schema::HostSessionTransitionIntentV2,
        >,
    ),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ObjectPublicationOutcomeV1 {
    PublishedOrphan,
    JoinedExactOrphan,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct GeneratedObjectV1 {
    pub(super) reference:
        crate::execution::agent_runtime::host_session_authority::schema::AuthorityObjectRefV1,
    pub(super) byte_length: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct RetainedWorkerReservationInputV1 {
    pub(super) issuer_request_id: String,
    pub(super) orchestration_session_id: String,
    pub(super) expected_authority_store_id: String,
    pub(super) expected_authority_revision: u64,
    pub(super) expected_authority_commitment:
        crate::execution::agent_runtime::host_session_authority::schema::AuthorityObjectCommitmentV1,
    pub(super) retained_participant_id: String,
    pub(super) descriptor_bytes: Vec<u8>,
    pub(super) resume_handle_bytes: Vec<u8>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum RetainedReservationCrashPointV1 {
    BeforeRootPublication,
    AfterRootPublication,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(not(test), allow(dead_code))]
pub(super) enum RetainedApplicationCrashPointV1 {
    BeforeRootPublication,
    AfterRootPublication,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct RetainedWorkerReservationV1 {
    pub(super) request: crate::execution::agent_runtime::host_session_authority::store_schema::RetainedWorkerAuthorityRegistrationRequestV1,
    pub(super) descriptor_ref:
        crate::execution::agent_runtime::host_session_authority::schema::AuthorityObjectRefV1,
    pub(super) resume_handle_ref:
        crate::execution::agent_runtime::host_session_authority::schema::AuthorityObjectRefV1,
    pub(super) retained_worker_ref:
        crate::execution::agent_runtime::host_session_authority::schema::AuthorityObjectRefV1,
    pub(super) descriptor_bytes: Vec<u8>,
    pub(super) resume_handle_bytes: Vec<u8>,
    pub(super) retained_worker_bytes: Vec<u8>,
    pub(super) joined: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct RetainedWorkerApplicationV1 {
    pub(super) registration: crate::execution::agent_runtime::host_session_authority::store_schema::RetainedWorkerAuthorityRegistrationV1,
    pub(super) joined: bool,
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
mod platform {
    use std::fmt;

    use rand::RngCore;

    #[cfg(test)]
    use super::LegacyMutationV1;
    use super::{
        BootstrapClassificationV1, BootstrapError, ExpectedRevisionsV1, GeneratedObjectV1,
        GreenfieldUpgradeCrashPointV1, InitializationCrashPointV1, InitializationMaterialV1,
        KeyLifecycleCrashPointV1, LegacyStateStoreCollectionV1, ObjectPublicationOutcomeV1,
        ObjectVerificationContextV1, RetainedApplicationCrashPointV1,
        RetainedReservationCrashPointV1, RetainedWorkerApplicationV1,
        RetainedWorkerReservationInputV1, RetainedWorkerReservationV1, RootUpgradeOutcomeV1,
        TransactionCommitOutcomeV1, VersionedObjectVerificationParentIntentV1,
    };
    use crate::execution::agent_runtime::host_session_authority::canonical_json;
    use crate::execution::agent_runtime::host_session_authority::hash::{
        canonical_sha256, store_hmac_sha256, validate_object_commitment_rule, SensitiveDomainV1,
    };
    use crate::execution::agent_runtime::host_session_authority::schema::{
        AgentDescriptorHashInputV1, AgentExecutionScopeV1, ApplicationResultHashInputV1,
        AuthoritativeLineageHashInputV1, AuthorityObjectCommitmentV1, AuthorityObjectKindV1,
        AuthorityObjectRefV1, CanonicalDirectoryV1, DurableSessionAuthorityHashInputV1,
        HostAttachContractHashInputV1, InputAcceptanceHashInputV1, PolicyObjectHashInputV1,
        PostTurnCompletionHashInputV1, ResumeHandleHashInputV1, RetainedWorkerObjectHashInputV1,
        TerminalHandoffHashInputV1, TerminalHandoffStateV1, TimestampV1,
    };
    use crate::execution::agent_runtime::host_session_authority::store_format::{
        key_id, nonce, object_ref_id, store_id, validate_key_id, validate_ref_id,
        AuthorityStoreCommitmentKeyFileV1, TempNameV1,
    };
    use crate::execution::agent_runtime::host_session_authority::store_schema::{
        AuthorityObjectIndexEntryV1, AuthorityObjectStorageStateV1,
        AuthorityStoreCommitmentAlgorithmV1, AuthorityStoreCommitmentKeyStateV1,
        AuthorityStoreCommitmentKeyV1, AuthorityStoreInitializationV1,
        GreenfieldNamespaceCertificateV1, HostSessionPostTurnApplicationV1,
        HostSessionStartupOwnershipApplicationV1, HostSessionTransitionInputHandoffV1,
        HostSessionTransitionIntentStateV1, HostSessionTransitionIntentStateV2,
        HostSessionTransitionIntentV1, HostSessionTransitionIntentV2,
        HostSessionTransitionTransportPayloadStateV1,
        RetainedWorkerAuthorityRegistrationRequestStateV1,
        RetainedWorkerAuthorityRegistrationRequestV1, RetainedWorkerAuthorityRegistrationV1,
        SessionNamespaceRecordV1, StateRootV1, StateRootV2, VersionedStateRoot,
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
    const RETAINED_ADMISSION_DIRECTORY: &str = "retained-worker-admission-v1";
    const RETAINED_ADMISSION_REGISTRY_FILE: &str = "registry-v1.json";
    const RETAINED_ADMISSION_KEYS_DIRECTORY: &str = "keys";
    const RETAINED_ADMISSION_TEMP_DIRECTORY: &str = "tmp";

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
    #[cfg(test)]
    use key_lifecycle::{
        retire_commitment_key_versioned_with, rotate_commitment_key_versioned_with,
    };
    use key_lifecycle::{retire_commitment_key_with, rotate_commitment_key_with};
    #[path = "object_persistence.rs"]
    mod object_persistence;
    use object_persistence::validate_orphan_candidate;
    use object_persistence::verify_object_bytes;
    use object_persistence::{
        canonical_digest, publish_or_join_orphan, reserved_object_ref_id_is_globally_absent,
        sensitive_domain,
    };
    #[path = "reachability.rs"]
    mod reachability;
    use reachability::{add_expected_ref, collect_reachable_objects, collect_reachable_objects_v2};
    #[path = "transaction.rs"]
    mod transaction;
    #[cfg(test)]
    use transaction::retain_classified_legacy_directories_test;
    #[cfg(test)]
    use transaction::with_semantic_preflight;
    pub(crate) use transaction::LegacyStateStoreTransactionV1;
    use transaction::{
        begin_legacy_state_store_transaction as begin_legacy_transaction,
        begin_legacy_state_store_transaction_for_identity as begin_legacy_transaction_for_identity,
        compare_and_swap_opened_root_with, compare_and_swap_opened_root_with_exact_current,
        compare_and_swap_root_with, with_existing_semantic_preflight,
        with_existing_versioned_semantic_preflight, with_opened_existing_semantic_preflight,
        with_opened_existing_versioned_semantic_preflight, with_opened_semantic_preflight,
        SemanticPreflightMode,
    };

    pub(crate) struct RetainedWorkerAdmissionStorageV1 {
        root: TrustedAuthorityRoot,
        authority_store_id: String,
    }

    pub(crate) struct RetainedWorkerAdmissionStorageTransactionV1 {
        admission: TrustedDirectory,
        keys: TrustedDirectory,
        tmp: TrustedDirectory,
    }

    impl RetainedWorkerAdmissionStorageV1 {
        pub(crate) fn authority_store_id(&self) -> &str {
            &self.authority_store_id
        }

        pub(crate) fn transaction<T>(
            &self,
            operation: impl FnOnce(
                &mut RetainedWorkerAdmissionStorageTransactionV1,
            ) -> Result<T, BootstrapError>,
        ) -> Result<T, BootstrapError> {
            with_opened_existing_versioned_semantic_preflight(&self.root, |transaction| {
                transaction.reconcile()?;
                if transaction.root.authority_store_id() != self.authority_store_id {
                    return Err(BootstrapError(
                        "retained admission capability authority store changed",
                    ));
                }
                let mut storage = open_retained_admission_transaction(transaction.layout)?;
                operation(&mut storage)
            })
        }
    }

    impl RetainedWorkerAdmissionStorageTransactionV1 {
        pub(crate) fn read_registry(&self) -> Result<Option<Vec<u8>>, BootstrapError> {
            match self
                .admission
                .entry_kind(RETAINED_ADMISSION_REGISTRY_FILE)
                .map_err(|_| BootstrapError("inspect retained admission registry"))?
            {
                None => Ok(None),
                Some(EntryKind::RegularFile) => self
                    .admission
                    .open_file(RETAINED_ADMISSION_REGISTRY_FILE)
                    .and_then(|file| file.read_all())
                    .map(Some)
                    .map_err(|_| BootstrapError("read retained admission registry")),
                Some(_) => Err(BootstrapError("retained admission registry is unsafe")),
            }
        }

        pub(crate) fn read_keys(&self) -> Result<Vec<(String, Vec<u8>)>, BootstrapError> {
            let mut keys = Vec::new();
            for entry in self
                .keys
                .entries()
                .map_err(|_| BootstrapError("enumerate retained admission keys"))?
            {
                if entry.kind != EntryKind::RegularFile || !admission_key_file_name(&entry.name) {
                    return Err(BootstrapError("retained admission key layout is invalid"));
                }
                self.keys
                    .revalidate_entry(&entry)
                    .map_err(|_| BootstrapError("retained admission key changed"))?;
                let bytes = self
                    .keys
                    .open_file_entry(&entry)
                    .and_then(|file| file.read_all())
                    .map_err(|_| BootstrapError("read retained admission key"))?;
                keys.push((entry.name, bytes));
            }
            Ok(keys)
        }

        pub(crate) fn stage_key_temp(
            &self,
            temp_name: &str,
            bytes: &[u8],
        ) -> Result<(), BootstrapError> {
            if !admission_temp_file_name(temp_name) {
                return Err(BootstrapError(
                    "retained admission key temp name is invalid",
                ));
            }
            let mut temp = self
                .tmp
                .create_file(temp_name)
                .map_err(|_| BootstrapError("create retained admission key temp"))?;
            temp.write_all(bytes)
                .map_err(|_| BootstrapError("write retained admission key temp"))?;
            temp.sync()
                .map_err(|_| BootstrapError("sync retained admission key temp"))?;
            self.tmp
                .sync()
                .map_err(|_| BootstrapError("sync retained admission temp directory"))
        }

        pub(crate) fn publish_staged_key_no_replace(
            &self,
            temp_name: &str,
            key_name: &str,
        ) -> Result<(), BootstrapError> {
            if !admission_temp_file_name(temp_name) || !admission_key_file_name(key_name) {
                return Err(BootstrapError(
                    "retained admission key publication name is invalid",
                ));
            }
            let temp = self
                .tmp
                .open_file(temp_name)
                .map_err(|_| BootstrapError("open retained admission key temp"))?;
            self.tmp
                .rename_no_replace(temp_name, temp, &self.keys, key_name)
                .map_err(|_| {
                    BootstrapError("publish retained admission key without replacement")
                })?;
            self.keys
                .sync()
                .map_err(|_| BootstrapError("sync retained admission key directory"))
        }

        pub(crate) fn remove_key(&self, key_name: &str) -> Result<(), BootstrapError> {
            if !admission_key_file_name(key_name) {
                return Err(BootstrapError(
                    "retained admission orphan key name is invalid",
                ));
            }
            self.keys
                .unlink_file(key_name)
                .map_err(|_| BootstrapError("remove retained admission orphan key"))
        }

        pub(crate) fn publish_registry_no_replace(
            &self,
            temp_name: &str,
            bytes: &[u8],
        ) -> Result<(), BootstrapError> {
            self.publish_registry(temp_name, bytes, false)
        }

        pub(crate) fn replace_registry(
            &self,
            temp_name: &str,
            bytes: &[u8],
        ) -> Result<(), BootstrapError> {
            self.publish_registry(temp_name, bytes, true)
        }

        fn publish_registry(
            &self,
            temp_name: &str,
            bytes: &[u8],
            replace: bool,
        ) -> Result<(), BootstrapError> {
            if !admission_temp_file_name(temp_name) {
                return Err(BootstrapError(
                    "retained admission registry temp name is invalid",
                ));
            }
            let mut temp = self
                .tmp
                .create_file(temp_name)
                .map_err(|_| BootstrapError("create retained admission registry temp"))?;
            temp.write_all(bytes)
                .map_err(|_| BootstrapError("write retained admission registry temp"))?;
            temp.sync()
                .map_err(|_| BootstrapError("sync retained admission registry temp"))?;
            if replace {
                self.tmp
                    .rename_replace(
                        temp_name,
                        temp,
                        &self.admission,
                        RETAINED_ADMISSION_REGISTRY_FILE,
                    )
                    .map_err(|_| BootstrapError("replace retained admission registry"))?;
            } else {
                self.tmp
                    .rename_no_replace(
                        temp_name,
                        temp,
                        &self.admission,
                        RETAINED_ADMISSION_REGISTRY_FILE,
                    )
                    .map_err(|_| BootstrapError("publish retained admission registry"))?;
            }
            self.admission
                .sync()
                .map_err(|_| BootstrapError("sync retained admission directory"))
        }
    }

    pub(super) fn retained_worker_admission_storage_opened(
        opened: &TrustedAuthorityRoot,
    ) -> Result<RetainedWorkerAdmissionStorageV1, BootstrapError> {
        opened
            .revalidate()
            .map_err(|_| BootstrapError("revalidate retained admission authority root"))?;
        let rebound =
            TrustedAuthorityRoot::open(std::path::Path::new(&opened.identity().physical_path))
                .map_err(|_| BootstrapError("open retained admission authority root"))?;
        if rebound.identity() != opened.identity() {
            return Err(BootstrapError(
                "retained admission authority root identity mismatch",
            ));
        }
        let authority_store_id =
            with_opened_existing_versioned_semantic_preflight(&rebound, |transaction| {
                transaction.reconcile()?;
                Ok(transaction.root.authority_store_id().to_owned())
            })?;
        Ok(RetainedWorkerAdmissionStorageV1 {
            root: rebound,
            authority_store_id,
        })
    }

    fn open_retained_admission_transaction(
        layout: &StoreLayout<'_>,
    ) -> Result<RetainedWorkerAdmissionStorageTransactionV1, BootstrapError> {
        let admission = match layout
            .authority
            .entry_kind(RETAINED_ADMISSION_DIRECTORY)
            .map_err(|_| BootstrapError("inspect retained admission directory"))?
        {
            None => layout
                .authority
                .create_directory(RETAINED_ADMISSION_DIRECTORY)
                .map_err(|_| BootstrapError("create retained admission directory"))?,
            Some(EntryKind::Directory) => layout
                .authority
                .open_directory(RETAINED_ADMISSION_DIRECTORY)
                .map_err(|_| BootstrapError("open retained admission directory"))?,
            Some(_) => return Err(BootstrapError("retained admission directory is unsafe")),
        };
        let entries = admission
            .entries()
            .map_err(|_| BootstrapError("enumerate retained admission directory"))?;
        let registry_exists = entries.iter().any(|entry| {
            entry.name == RETAINED_ADMISSION_REGISTRY_FILE && entry.kind == EntryKind::RegularFile
        });
        for entry in &entries {
            let valid = matches!(
                (entry.name.as_str(), entry.kind),
                (
                    RETAINED_ADMISSION_KEYS_DIRECTORY | RETAINED_ADMISSION_TEMP_DIRECTORY,
                    EntryKind::Directory
                ) | (RETAINED_ADMISSION_REGISTRY_FILE, EntryKind::RegularFile)
            );
            if !valid {
                return Err(BootstrapError(
                    "retained admission directory layout is invalid",
                ));
            }
            admission
                .revalidate_entry(entry)
                .map_err(|_| BootstrapError("retained admission directory entry changed"))?;
        }
        let open_component = |name: &str| match admission
            .entry_kind(name)
            .map_err(|_| BootstrapError("inspect retained admission component"))?
        {
            Some(EntryKind::Directory) => admission
                .open_directory(name)
                .map_err(|_| BootstrapError("open retained admission component")),
            None if !registry_exists => admission
                .create_directory(name)
                .map_err(|_| BootstrapError("create retained admission component")),
            None | Some(_) => Err(BootstrapError("retained admission component is invalid")),
        };
        let keys = open_component(RETAINED_ADMISSION_KEYS_DIRECTORY)?;
        let tmp = open_component(RETAINED_ADMISSION_TEMP_DIRECTORY)?;
        reconcile_retained_admission_temps(&tmp)?;
        admission
            .sync()
            .map_err(|_| BootstrapError("sync retained admission layout"))?;
        Ok(RetainedWorkerAdmissionStorageTransactionV1 {
            admission,
            keys,
            tmp,
        })
    }

    fn reconcile_retained_admission_temps(tmp: &TrustedDirectory) -> Result<(), BootstrapError> {
        for entry in tmp
            .entries()
            .map_err(|_| BootstrapError("enumerate retained admission temps"))?
        {
            if entry.kind != EntryKind::RegularFile || !admission_temp_file_name(&entry.name) {
                return Err(BootstrapError("retained admission temp layout is invalid"));
            }
            tmp.revalidate_entry(&entry)
                .map_err(|_| BootstrapError("retained admission temp changed"))?;
            tmp.unlink_file(&entry.name)
                .map_err(|_| BootstrapError("remove retained admission temp"))?;
        }
        Ok(())
    }

    fn admission_key_file_name(name: &str) -> bool {
        name.strip_suffix(".key").is_some_and(|key_id| {
            key_id.strip_prefix("adk_").is_some_and(|hex| {
                hex.len() == 32
                    && hex
                        .bytes()
                        .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
            })
        })
    }

    fn admission_temp_file_name(name: &str) -> bool {
        ["admission-key--", "admission-registry--"]
            .into_iter()
            .any(|prefix| {
                name.strip_prefix(prefix)
                    .and_then(|rest| rest.strip_suffix(".tmp"))
                    .is_some_and(|hex| {
                        hex.len() == 32
                            && hex
                                .bytes()
                                .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
                    })
            })
    }

    pub(super) fn classify(path: &std::path::Path) -> BootstrapClassificationV1 {
        classify_checked(path).unwrap_or(BootstrapClassificationV1::CorruptOrUnsupported)
    }

    pub(super) fn classify_opened(root: &TrustedAuthorityRoot) -> BootstrapClassificationV1 {
        classify_opened_checked(root).unwrap_or(BootstrapClassificationV1::CorruptOrUnsupported)
    }

    pub(super) fn bootstrap(path: &std::path::Path) -> Result<StateRootV1, BootstrapError> {
        let root = TrustedAuthorityRoot::open(path)
            .map_err(|_| BootstrapError("open trusted authority root"))?;
        bootstrap_opened(&root)
    }

    pub(super) fn bootstrap_opened(
        root: &TrustedAuthorityRoot,
    ) -> Result<StateRootV1, BootstrapError> {
        with_opened_semantic_preflight(
            root,
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

    pub(super) fn upgrade_greenfield_root_opened(
        root: &TrustedAuthorityRoot,
    ) -> Result<RootUpgradeOutcomeV1, BootstrapError> {
        upgrade_greenfield_root_opened_with(root, system_material()?.root_nonce, None)
    }

    #[cfg(test)]
    pub(super) fn upgrade_greenfield_root_test(
        path: &std::path::Path,
        nonce_bytes: [u8; 16],
        stop: Option<GreenfieldUpgradeCrashPointV1>,
    ) -> Result<RootUpgradeOutcomeV1, BootstrapError> {
        let root = TrustedAuthorityRoot::open(path)
            .map_err(|_| BootstrapError("open trusted authority root for greenfield upgrade"))?;
        upgrade_greenfield_root_opened_with(&root, nonce_bytes, stop)
    }

    #[cfg(test)]
    pub(super) fn reachable_v2_ref_ids_test(
        root: &StateRootV2,
    ) -> Result<Vec<String>, BootstrapError> {
        collect_reachable_objects_v2(root)
            .map(|reachable| reachable.into_keys().collect())
            .map_err(|_| BootstrapError("collect strict V2 reachable objects"))
    }

    #[cfg(test)]
    pub(super) fn reserved_object_ref_id_is_globally_absent_test(
        path: &std::path::Path,
        ref_id: &str,
    ) -> Result<bool, BootstrapError> {
        let root = TrustedAuthorityRoot::open(path)
            .map_err(|_| BootstrapError("open retained collision test root"))?;
        with_opened_existing_versioned_semantic_preflight(&root, |transaction| {
            if !matches!(transaction.root, VersionedStateRoot::V2(_)) {
                return Err(BootstrapError(
                    "retained collision test requires strict StateRootV2",
                ));
            }
            reserved_object_ref_id_is_globally_absent(transaction.layout, ref_id)
        })
    }

    #[cfg(test)]
    pub(super) fn retained_reservation_authority_matches_test(
        root: &StateRootV2,
        orchestration_session_id: &str,
        expected_authority_store_id: &str,
        expected_authority_revision: u64,
        expected_authority_commitment: &AuthorityObjectCommitmentV1,
    ) -> Result<(), BootstrapError> {
        validate_reservation_authority(
            root,
            &RetainedWorkerReservationInputV1 {
                issuer_request_id: "retained-worker-registration:test".into(),
                orchestration_session_id: orchestration_session_id.into(),
                expected_authority_store_id: expected_authority_store_id.into(),
                expected_authority_revision,
                expected_authority_commitment: expected_authority_commitment.clone(),
                retained_participant_id: "test-retained-participant".into(),
                descriptor_bytes: Vec::new(),
                resume_handle_bytes: Vec::new(),
            },
        )
    }

    fn upgrade_greenfield_root_opened_with(
        root_handle: &TrustedAuthorityRoot,
        nonce_bytes: [u8; 16],
        stop: Option<GreenfieldUpgradeCrashPointV1>,
    ) -> Result<RootUpgradeOutcomeV1, BootstrapError> {
        root_handle
            .revalidate()
            .map_err(|_| BootstrapError("UnsupportedNonGreenfieldRootV1"))?;
        let lock_scope = StoreLayoutLockScope::open_existing_activated(root_handle.directory())
            .map_err(|_| BootstrapError("UnsupportedNonGreenfieldRootV1"))?;
        let _lock = lock_scope
            .root_lock
            .lock_exclusive_owned()
            .map_err(|_| BootstrapError("lock greenfield upgrade root"))?;
        lock_scope
            .validate_temps()
            .map_err(|_| BootstrapError("UnsupportedNonGreenfieldRootV1"))?;
        let layout = lock_scope
            .finish()
            .map_err(|_| BootstrapError("UnsupportedNonGreenfieldRootV1"))?;
        layout
            .validate_closed_layout()
            .map_err(|_| BootstrapError("UnsupportedNonGreenfieldRootV1"))?;
        let legacy = LegacyObservation::capture(layout.bootstrap)
            .map_err(|_| BootstrapError("UnsupportedNonGreenfieldRootV1"))?;
        if legacy.has_artifact {
            return Err(BootstrapError("UnsupportedNonGreenfieldRootV1"));
        }
        let current = layout
            .read_greenfield_upgrade_root(root_handle.identity())
            .map_err(greenfield_upgrade_store_error)?;
        layout
            .reconcile_temps()
            .map_err(|_| BootstrapError("reconcile eligible greenfield upgrade temps"))?;
        let VersionedStateRoot::V1(v1) = current else {
            let VersionedStateRoot::V2(v2) = current else {
                unreachable!("closed root version")
            };
            legacy
                .revalidate(layout.bootstrap)
                .map_err(|_| BootstrapError("revalidate legacy state for V2 exact join"))?;
            root_handle
                .revalidate()
                .map_err(|_| BootstrapError("revalidate trusted root for V2 exact join"))?;
            if layout
                .read_greenfield_upgrade_root(root_handle.identity())
                .map_err(greenfield_upgrade_store_error)?
                != VersionedStateRoot::V2(v2.clone())
            {
                return Err(BootstrapError(
                    "greenfield V2 exact retry changed under lock",
                ));
            }
            layout
                .remove_matching_marker_v2(&v2)
                .map_err(|_| BootstrapError("reconcile V2 initialization marker"))?;
            layout
                .authority
                .sync()
                .map_err(|_| BootstrapError("sync exact-joined V2 authority directory"))?;
            return Ok(RootUpgradeOutcomeV1::JoinedExact(v2));
        };
        let v2 = StateRootV2::try_from_greenfield_v1(&v1)
            .map_err(|_| BootstrapError("UnsupportedNonGreenfieldRootV1"))?;
        let temp_name = TempNameV1::Root {
            root_revision: v2.root_revision,
            nonce: nonce(nonce_bytes),
        }
        .file_name();
        let bytes = canonical_json::to_vec(&v2)
            .map_err(|_| BootstrapError("encode strict StateRootV2 upgrade"))?;
        let mut temp = layout
            .tmp
            .create_file(&temp_name)
            .map_err(|_| BootstrapError("create StateRootV2 upgrade temp"))?;
        temp.write_all(&bytes)
            .map_err(|_| BootstrapError("write StateRootV2 upgrade temp"))?;
        temp.sync()
            .map_err(|_| BootstrapError("sync StateRootV2 upgrade temp"))?;
        if stop == Some(GreenfieldUpgradeCrashPointV1::BeforeRootPublication) {
            return Err(BootstrapError(
                "injected crash before StateRootV2 publication",
            ));
        }
        let final_state_is_exact = stop
            != Some(GreenfieldUpgradeCrashPointV1::FinalRevalidationMismatch)
            && legacy.revalidate(layout.bootstrap).is_ok()
            && root_handle.revalidate().is_ok()
            && matches!(
                layout.read_greenfield_upgrade_root(root_handle.identity()),
                Ok(VersionedStateRoot::V1(ref current)) if current == &v1
            );
        if !final_state_is_exact {
            remove_rejected_upgrade_temp(&layout, &temp_name)?;
            return Err(BootstrapError("UnsupportedNonGreenfieldRootV1"));
        }
        layout
            .tmp
            .rename_replace(&temp_name, temp, &layout.authority, ROOT_FILE)
            .map_err(|_| BootstrapError("publish strict StateRootV2 upgrade"))?;
        layout
            .authority
            .sync()
            .map_err(|_| BootstrapError("sync StateRootV2 authority directory"))?;
        if stop == Some(GreenfieldUpgradeCrashPointV1::AfterRootPublication) {
            return Err(BootstrapError(
                "injected crash after StateRootV2 publication",
            ));
        }
        if layout
            .read_greenfield_upgrade_root(root_handle.identity())
            .map_err(greenfield_upgrade_store_error)?
            != VersionedStateRoot::V2(v2.clone())
        {
            return Err(BootstrapError(
                "published StateRootV2 does not exactly match upgrade candidate",
            ));
        }
        layout
            .remove_matching_marker_v2(&v2)
            .map_err(|_| BootstrapError("remove matching V2 initialization marker"))?;
        layout
            .authority
            .sync()
            .map_err(|_| BootstrapError("sync reconciled V2 authority directory"))?;
        Ok(RootUpgradeOutcomeV1::Upgraded(v2))
    }

    fn greenfield_upgrade_store_error(_error: StoreError) -> BootstrapError {
        BootstrapError("UnsupportedNonGreenfieldRootV1")
    }

    fn remove_rejected_upgrade_temp(
        layout: &StoreLayout<'_>,
        temp_name: &str,
    ) -> Result<(), BootstrapError> {
        layout
            .tmp
            .unlink_file(temp_name)
            .map_err(|_| BootstrapError("remove rejected StateRootV2 upgrade temp"))?;
        layout
            .tmp
            .sync()
            .map_err(|_| BootstrapError("sync rejected StateRootV2 upgrade temp removal"))
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
        let root = TrustedAuthorityRoot::open(path)
            .map_err(|_| BootstrapError("open trusted authority root"))?;
        prepare_typed_object_opened(&root, expected_root_revision, reference, bytes, context)
    }

    pub(super) fn prepare_typed_object_opened(
        root: &TrustedAuthorityRoot,
        expected_root_revision: u64,
        reference: &AuthorityObjectRefV1,
        bytes: &[u8],
        context: Option<&ObjectVerificationContextV1>,
    ) -> Result<ObjectPublicationOutcomeV1, BootstrapError> {
        prepare_typed_object_opened_with(
            root,
            expected_root_revision,
            reference,
            bytes,
            context,
            system_material()?.key_nonce,
        )
    }

    pub(super) fn read_opened_root_v2(
        root: &TrustedAuthorityRoot,
    ) -> Result<StateRootV2, BootstrapError> {
        with_opened_existing_versioned_semantic_preflight(root, |transaction| {
            let VersionedStateRoot::V2(root) = &transaction.root else {
                return Err(BootstrapError("A1.2a requires strict StateRootV2"));
            };
            Ok(root.clone())
        })
    }

    pub(super) fn prepare_generated_object_v2_opened(
        root: &TrustedAuthorityRoot,
        expected_root_revision: u64,
        object_kind: AuthorityObjectKindV1,
        bytes: &[u8],
        context: Option<&ObjectVerificationContextV1>,
    ) -> Result<GeneratedObjectV1, BootstrapError> {
        with_opened_existing_versioned_semantic_preflight(root, |transaction| {
            transaction.require_expected_root(expected_root_revision)?;
            let VersionedStateRoot::V2(root) = &transaction.root else {
                return Err(BootstrapError("A1.2a objects require strict StateRootV2"));
            };
            let reference =
                generated_object_ref_v2(transaction.layout, root, object_kind, bytes, context)?;
            validate_orphan_candidate(transaction.layout, root, &reference, bytes, context)?;
            transaction.reconcile()?;
            publish_or_join_orphan(
                transaction.layout,
                root,
                &reference,
                bytes,
                context,
                system_material()?.key_nonce,
            )?;
            Ok(GeneratedObjectV1 {
                reference,
                byte_length: bytes.len() as u64,
            })
        })
    }

    pub(super) fn allocate_sensitive_object_ref_v2_opened(
        root: &TrustedAuthorityRoot,
        expected_root_revision: u64,
        object_kind: AuthorityObjectKindV1,
        bytes: &[u8],
        context: &ObjectVerificationContextV1,
    ) -> Result<AuthorityObjectRefV1, BootstrapError> {
        with_opened_existing_versioned_semantic_preflight(root, |transaction| {
            transaction.require_expected_root(expected_root_revision)?;
            let VersionedStateRoot::V2(root) = &transaction.root else {
                return Err(BootstrapError("A1.2a objects require strict StateRootV2"));
            };
            if sensitive_domain(object_kind).is_none() {
                return Err(BootstrapError("generated object kind is not sensitive"));
            }
            generated_object_ref_v2(transaction.layout, root, object_kind, bytes, Some(context))
        })
    }

    pub(super) fn prepare_typed_object_v2_opened(
        root: &TrustedAuthorityRoot,
        expected_root_revision: u64,
        reference: &AuthorityObjectRefV1,
        bytes: &[u8],
        context: Option<&ObjectVerificationContextV1>,
    ) -> Result<ObjectPublicationOutcomeV1, BootstrapError> {
        with_opened_existing_versioned_semantic_preflight(root, |transaction| {
            transaction.require_expected_root(expected_root_revision)?;
            let VersionedStateRoot::V2(root) = &transaction.root else {
                return Err(BootstrapError("A1.2a objects require strict StateRootV2"));
            };
            validate_orphan_candidate(transaction.layout, root, reference, bytes, context)?;
            transaction.reconcile()?;
            publish_or_join_orphan(
                transaction.layout,
                root,
                reference,
                bytes,
                context,
                system_material()?.key_nonce,
            )
        })
    }

    pub(super) fn read_typed_object_v2_opened(
        root: &TrustedAuthorityRoot,
        expected_root_revision: u64,
        reference: &AuthorityObjectRefV1,
        context: Option<&ObjectVerificationContextV1>,
    ) -> Result<Vec<u8>, BootstrapError> {
        with_opened_existing_versioned_semantic_preflight(root, |transaction| {
            transaction.require_expected_root(expected_root_revision)?;
            let VersionedStateRoot::V2(root) = &transaction.root else {
                return Err(BootstrapError("A1.2a objects require strict StateRootV2"));
            };
            let index = root
                .object_index
                .get(&reference.ref_id)
                .filter(|index| {
                    index.object_kind == reference.object_kind
                        && index.object_schema_version == reference.schema_version
                        && index.storage_state == AuthorityObjectStorageStateV1::Present
                })
                .ok_or(BootstrapError("typed object has no present index entry"))?;
            let kind = transaction
                .layout
                .objects
                .open_directory(kind_slug(reference.object_kind))
                .map_err(|_| BootstrapError("open typed object kind directory"))?;
            let version = kind
                .open_directory(&format!("v{}", reference.schema_version))
                .map_err(|_| BootstrapError("open typed object version directory"))?;
            let bytes = version
                .open_file(&format!("{}.obj", reference.ref_id))
                .and_then(|file| file.read_all())
                .map_err(|_| BootstrapError("read typed object"))?;
            if bytes.len() as u64 != index.byte_length {
                return Err(BootstrapError("typed object byte length mismatch"));
            }
            verify_object_bytes(transaction.layout, root, reference, &bytes, context, true)?;
            Ok(bytes)
        })
    }

    pub(super) fn commit_v2_root_exact_current_opened(
        root: &TrustedAuthorityRoot,
        exact_current: &StateRootV2,
        proposed: &StateRootV2,
        mut publication_guard: impl FnMut() -> Result<(), BootstrapError>,
    ) -> Result<StateRootV2, BootstrapError> {
        with_opened_existing_versioned_semantic_preflight(root, |transaction| {
            let VersionedStateRoot::V2(locked) = &transaction.root else {
                return Err(BootstrapError("A1.2a mutation requires strict StateRootV2"));
            };
            if locked != exact_current {
                return Err(BootstrapError(
                    "locked V2 authority root differs from exact observed root",
                ));
            }
            let candidate = VersionedStateRoot::V2(proposed.clone());
            transaction.validate_publication_candidate(exact_current.root_revision, &candidate)?;
            transaction.reconcile()?;
            publish_versioned_replacement_root(
                transaction.layout,
                transaction.trusted_root,
                &transaction.legacy,
                &candidate,
                system_material()?.root_nonce,
                || {
                    transaction
                        .validate_publication_candidate(exact_current.root_revision, &candidate)?;
                    publication_guard()
                },
            )?;
            Ok(proposed.clone())
        })
    }

    pub(super) fn reserve_retained_worker_registration_opened(
        root_handle: &TrustedAuthorityRoot,
        input: &RetainedWorkerReservationInputV1,
        build_worker: impl Fn(
            &AuthorityObjectRefV1,
            &AuthorityObjectRefV1,
            &AuthorityObjectRefV1,
            &crate::execution::agent_runtime::host_session_authority::schema::WorldBindingV1,
        ) -> Result<Vec<u8>, &'static str>,
    ) -> Result<RetainedWorkerReservationV1, BootstrapError> {
        reserve_retained_worker_registration_opened_with(
            root_handle,
            input,
            None,
            None,
            build_worker,
        )
    }

    #[cfg(test)]
    pub(super) fn reserve_retained_worker_registration_at_opened(
        root_handle: &TrustedAuthorityRoot,
        input: &RetainedWorkerReservationInputV1,
        registered_at: TimestampV1,
        crash_point: Option<RetainedReservationCrashPointV1>,
        build_worker: impl Fn(
            &AuthorityObjectRefV1,
            &AuthorityObjectRefV1,
            &AuthorityObjectRefV1,
            &crate::execution::agent_runtime::host_session_authority::schema::WorldBindingV1,
        ) -> Result<Vec<u8>, &'static str>,
    ) -> Result<RetainedWorkerReservationV1, BootstrapError> {
        reserve_retained_worker_registration_opened_with(
            root_handle,
            input,
            Some(registered_at),
            crash_point,
            build_worker,
        )
    }

    fn reserve_retained_worker_registration_opened_with(
        root_handle: &TrustedAuthorityRoot,
        input: &RetainedWorkerReservationInputV1,
        registered_at_override: Option<TimestampV1>,
        crash_point: Option<RetainedReservationCrashPointV1>,
        build_worker: impl Fn(
            &AuthorityObjectRefV1,
            &AuthorityObjectRefV1,
            &AuthorityObjectRefV1,
            &crate::execution::agent_runtime::host_session_authority::schema::WorldBindingV1,
        ) -> Result<Vec<u8>, &'static str>,
    ) -> Result<RetainedWorkerReservationV1, BootstrapError> {
        with_opened_existing_versioned_semantic_preflight(root_handle, |transaction| {
            let VersionedStateRoot::V2(root) = &transaction.root else {
                return Err(BootstrapError(
                    "retained registration requires strict StateRootV2",
                ));
            };
            if input.issuer_request_id.is_empty()
                || !input
                    .issuer_request_id
                    .starts_with("retained-worker-registration:")
                || input.issuer_request_id == "retained-worker-registration:"
                || input.orchestration_session_id.is_empty()
                || input.retained_participant_id.is_empty()
                || input.expected_authority_revision == 0
            {
                return Err(BootstrapError(
                    "retained registration input identity is invalid",
                ));
            }
            let descriptor: AgentDescriptorHashInputV1 =
                canonical_json::from_slice(&input.descriptor_bytes)
                    .map_err(|_| BootstrapError("decode retained descriptor bytes"))?;
            let resume: ResumeHandleHashInputV1 =
                canonical_json::from_slice(&input.resume_handle_bytes)
                    .map_err(|_| BootstrapError("decode retained resume bytes"))?;
            if descriptor.schema_version != 1
                || descriptor.descriptor.schema_version != 1
                || descriptor.descriptor.execution_scope != AgentExecutionScopeV1::World
                || resume.schema_version != 1
                || resume.orchestration_session_id != input.orchestration_session_id
                || resume.participant_id != input.retained_participant_id
                || resume.backend_id != descriptor.descriptor.backend_id
                || resume.protocol != descriptor.descriptor.protocol
            {
                return Err(BootstrapError(
                    "retained descriptor and resume plan disagree",
                ));
            }
            let descriptor_commitment = canonical_commitment_for_kind(
                AuthorityObjectKindV1::AgentDescriptor,
                &input.descriptor_bytes,
            )?;
            let resume_handle_commitment = canonical_commitment_for_kind(
                AuthorityObjectKindV1::ResumeHandle,
                &input.resume_handle_bytes,
            )?;

            if let Some(existing) = root
                .retained_worker_registration_request_index
                .get(&input.issuer_request_id)
            {
                let descriptor_ref = reserved_ref(
                    &existing.descriptor_ref_id,
                    AuthorityObjectKindV1::AgentDescriptor,
                    &existing.descriptor_commitment,
                );
                let resume_handle_ref = reserved_ref(
                    &existing.resume_handle_ref_id,
                    AuthorityObjectKindV1::ResumeHandle,
                    &existing.resume_handle_commitment,
                );
                let retained_worker_ref = reserved_ref(
                    &existing.retained_worker_ref_id,
                    AuthorityObjectKindV1::RetainedWorker,
                    &existing.retained_worker_commitment,
                );
                let worker_bytes = build_worker(
                    &descriptor_ref,
                    &resume_handle_ref,
                    &existing.current_policy_ref,
                    &existing.world_binding,
                )
                .map_err(BootstrapError)?;
                let worker_commitment = canonical_commitment_for_kind(
                    AuthorityObjectKindV1::RetainedWorker,
                    &worker_bytes,
                )?;
                if root.authority_store_id != input.expected_authority_store_id
                    || existing.orchestration_session_id != input.orchestration_session_id
                    || existing.authority_revision_before != input.expected_authority_revision
                    || existing.authority_record_commitment_before
                        != input.expected_authority_commitment
                    || existing.retained_participant_id != input.retained_participant_id
                    || existing.descriptor_commitment != descriptor_commitment
                    || existing.resume_handle_commitment != resume_handle_commitment
                    || existing.retained_worker_commitment != worker_commitment
                    || registered_at_override
                        .as_ref()
                        .is_some_and(|registered_at| registered_at != &existing.registered_at)
                {
                    return Err(BootstrapError(
                        "retained registration retry changed reserved bytes or scope",
                    ));
                }
                validate_reserved_graph(
                    transaction.layout,
                    root,
                    existing,
                    &descriptor_ref,
                    &input.descriptor_bytes,
                    &resume_handle_ref,
                    &input.resume_handle_bytes,
                    &retained_worker_ref,
                    &worker_bytes,
                )?;
                if matches!(
                    existing.state,
                    RetainedWorkerAuthorityRegistrationRequestStateV1::Reserved
                ) {
                    validate_reservation_authority(root, input)?;
                }
                transaction.reconcile()?;
                return Ok(RetainedWorkerReservationV1 {
                    request: existing.clone(),
                    descriptor_ref,
                    resume_handle_ref,
                    retained_worker_ref,
                    descriptor_bytes: input.descriptor_bytes.clone(),
                    resume_handle_bytes: input.resume_handle_bytes.clone(),
                    retained_worker_bytes: worker_bytes,
                    joined: true,
                });
            }

            validate_reservation_authority(root, input)?;
            if root
                .issuer_request_index
                .contains_key(&input.issuer_request_id)
                || root
                    .retained_worker_registration_request_index
                    .values()
                    .any(|request| request.retained_participant_id == input.retained_participant_id)
            {
                return Err(BootstrapError(
                    "retained registration issuer or participant is already reserved",
                ));
            }
            let SessionNamespaceRecordV1::Authority(authority) = root
                .session_namespace_map
                .get(&input.orchestration_session_id)
                .ok_or(BootstrapError("retained registration session is absent"))?
            else {
                return Err(BootstrapError(
                    "retained registration session has no durable authority",
                ));
            };
            if authority
                .authoritative_participant_lineage
                .contains(&input.retained_participant_id)
            {
                return Err(BootstrapError(
                    "retained participant already belongs to authority lineage",
                ));
            }
            let current_policy_ref = authority.current_policy_ref.clone().ok_or(BootstrapError(
                "retained registration requires current policy",
            ))?;
            let world_binding = authority.world_binding.clone().ok_or(BootstrapError(
                "retained registration requires exact world binding",
            ))?;
            let mut allocated_ref_ids = std::collections::BTreeSet::new();
            let descriptor_ref = allocate_reserved_canonical_ref(
                transaction.layout,
                root,
                AuthorityObjectKindV1::AgentDescriptor,
                &input.descriptor_bytes,
                &allocated_ref_ids,
            )?;
            allocated_ref_ids.insert(descriptor_ref.ref_id.clone());
            let resume_handle_ref = allocate_reserved_canonical_ref(
                transaction.layout,
                root,
                AuthorityObjectKindV1::ResumeHandle,
                &input.resume_handle_bytes,
                &allocated_ref_ids,
            )?;
            allocated_ref_ids.insert(resume_handle_ref.ref_id.clone());
            let worker_bytes = build_worker(
                &descriptor_ref,
                &resume_handle_ref,
                &current_policy_ref,
                &world_binding,
            )
            .map_err(BootstrapError)?;
            let retained_worker_ref = allocate_reserved_canonical_ref(
                transaction.layout,
                root,
                AuthorityObjectKindV1::RetainedWorker,
                &worker_bytes,
                &allocated_ref_ids,
            )?;
            let registration_id = allocate_registration_id(root)?;
            let registered_at = match &registered_at_override {
                Some(value) => value.clone(),
                None => system_timestamp()?,
            };
            let request = RetainedWorkerAuthorityRegistrationRequestV1 {
                schema_version: 1,
                issuer_request_id: input.issuer_request_id.clone(),
                registration_id,
                orchestration_session_id: input.orchestration_session_id.clone(),
                authority_revision_before: input.expected_authority_revision,
                authority_record_commitment_before: input.expected_authority_commitment.clone(),
                retained_participant_id: input.retained_participant_id.clone(),
                descriptor_ref_id: descriptor_ref.ref_id.clone(),
                descriptor_commitment: descriptor_ref.commitment.clone(),
                resume_handle_ref_id: resume_handle_ref.ref_id.clone(),
                resume_handle_commitment: resume_handle_ref.commitment.clone(),
                retained_worker_ref_id: retained_worker_ref.ref_id.clone(),
                retained_worker_commitment: retained_worker_ref.commitment.clone(),
                current_policy_ref,
                world_binding,
                registered_at,
                state: RetainedWorkerAuthorityRegistrationRequestStateV1::Reserved,
            };
            validate_reserved_graph(
                transaction.layout,
                root,
                &request,
                &descriptor_ref,
                &input.descriptor_bytes,
                &resume_handle_ref,
                &input.resume_handle_bytes,
                &retained_worker_ref,
                &worker_bytes,
            )?;
            let mut proposed = root.clone();
            proposed.root_revision = proposed.root_revision.checked_add(1).ok_or(
                BootstrapError("retained reservation root revision overflow"),
            )?;
            proposed
                .retained_worker_registration_request_index
                .insert(input.issuer_request_id.clone(), request.clone());
            let candidate = VersionedStateRoot::V2(proposed);
            transaction.validate_publication_candidate(root.root_revision, &candidate)?;
            transaction.reconcile()?;
            publish_versioned_replacement_root(
                transaction.layout,
                transaction.trusted_root,
                &transaction.legacy,
                &candidate,
                system_material()?.root_nonce,
                || {
                    transaction.validate_publication_candidate(root.root_revision, &candidate)?;
                    if crash_point == Some(RetainedReservationCrashPointV1::BeforeRootPublication) {
                        return Err(BootstrapError(
                            "injected crash before retained reservation publication",
                        ));
                    }
                    Ok(())
                },
            )?;
            if crash_point == Some(RetainedReservationCrashPointV1::AfterRootPublication) {
                return Err(BootstrapError(
                    "injected crash after retained reservation publication",
                ));
            }
            Ok(RetainedWorkerReservationV1 {
                request,
                descriptor_ref,
                resume_handle_ref,
                retained_worker_ref,
                descriptor_bytes: input.descriptor_bytes.clone(),
                resume_handle_bytes: input.resume_handle_bytes.clone(),
                retained_worker_bytes: worker_bytes,
                joined: false,
            })
        })
    }

    pub(super) fn publish_reserved_retained_object_opened(
        root_handle: &TrustedAuthorityRoot,
        reserved: &RetainedWorkerReservationV1,
        reference: &AuthorityObjectRefV1,
        bytes: &[u8],
    ) -> Result<ObjectPublicationOutcomeV1, BootstrapError> {
        with_opened_existing_versioned_semantic_preflight(root_handle, |transaction| {
            let VersionedStateRoot::V2(root) = &transaction.root else {
                return Err(BootstrapError(
                    "retained object publication requires strict StateRootV2",
                ));
            };
            let persisted = root
                .retained_worker_registration_request_index
                .get(&reserved.request.issuer_request_id)
                .filter(|persisted| *persisted == &reserved.request)
                .ok_or(BootstrapError(
                    "retained object publication has no exact reservation",
                ))?;
            if !matches!(
                persisted.state,
                RetainedWorkerAuthorityRegistrationRequestStateV1::Reserved
            ) {
                return Err(BootstrapError(
                    "retained object publication reservation is not Reserved",
                ));
            }
            validate_reservation_authority(
                root,
                &RetainedWorkerReservationInputV1 {
                    issuer_request_id: persisted.issuer_request_id.clone(),
                    orchestration_session_id: persisted.orchestration_session_id.clone(),
                    expected_authority_store_id: root.authority_store_id.clone(),
                    expected_authority_revision: persisted.authority_revision_before,
                    expected_authority_commitment: persisted
                        .authority_record_commitment_before
                        .clone(),
                    retained_participant_id: persisted.retained_participant_id.clone(),
                    descriptor_bytes: reserved.descriptor_bytes.clone(),
                    resume_handle_bytes: reserved.resume_handle_bytes.clone(),
                },
            )?;
            validate_reserved_graph(
                transaction.layout,
                root,
                persisted,
                &reserved.descriptor_ref,
                &reserved.descriptor_bytes,
                &reserved.resume_handle_ref,
                &reserved.resume_handle_bytes,
                &reserved.retained_worker_ref,
                &reserved.retained_worker_bytes,
            )?;
            let expected_bytes = if reference == &reserved.descriptor_ref {
                &reserved.descriptor_bytes
            } else if reference == &reserved.resume_handle_ref {
                &reserved.resume_handle_bytes
            } else if reference == &reserved.retained_worker_ref {
                &reserved.retained_worker_bytes
            } else {
                return Err(BootstrapError(
                    "retained object is outside the exact reserved graph",
                ));
            };
            if bytes != expected_bytes {
                return Err(BootstrapError(
                    "retained object publication bytes differ from reservation",
                ));
            }
            validate_orphan_candidate(transaction.layout, root, reference, bytes, None)?;
            transaction.reconcile()?;
            publish_or_join_orphan(
                transaction.layout,
                root,
                reference,
                bytes,
                None,
                system_material()?.key_nonce,
            )
        })
    }

    pub(super) fn apply_reserved_retained_worker_registration_opened(
        root_handle: &TrustedAuthorityRoot,
        reserved: &RetainedWorkerReservationV1,
    ) -> Result<RetainedWorkerApplicationV1, BootstrapError> {
        apply_reserved_retained_worker_registration_opened_with(root_handle, reserved, None)
    }

    #[cfg(test)]
    pub(super) fn apply_reserved_retained_worker_registration_with_crash_point_opened(
        root_handle: &TrustedAuthorityRoot,
        reserved: &RetainedWorkerReservationV1,
        crash_point: RetainedApplicationCrashPointV1,
    ) -> Result<RetainedWorkerApplicationV1, BootstrapError> {
        apply_reserved_retained_worker_registration_opened_with(
            root_handle,
            reserved,
            Some(crash_point),
        )
    }

    fn apply_reserved_retained_worker_registration_opened_with(
        root_handle: &TrustedAuthorityRoot,
        reserved: &RetainedWorkerReservationV1,
        crash_point: Option<RetainedApplicationCrashPointV1>,
    ) -> Result<RetainedWorkerApplicationV1, BootstrapError> {
        with_opened_existing_versioned_semantic_preflight(root_handle, |transaction| {
            let VersionedStateRoot::V2(root) = &transaction.root else {
                return Err(BootstrapError(
                    "retained authority application requires strict StateRootV2",
                ));
            };
            let persisted = root
                .retained_worker_registration_request_index
                .get(&reserved.request.issuer_request_id)
                .ok_or(BootstrapError(
                    "retained authority application has no reservation",
                ))?;
            let mut expected_request = reserved.request.clone();
            expected_request.state = persisted.state.clone();
            if &expected_request != persisted {
                return Err(BootstrapError(
                    "retained authority application reservation is inexact",
                ));
            }
            validate_reserved_graph(
                transaction.layout,
                root,
                persisted,
                &reserved.descriptor_ref,
                &reserved.descriptor_bytes,
                &reserved.resume_handle_ref,
                &reserved.resume_handle_bytes,
                &reserved.retained_worker_ref,
                &reserved.retained_worker_bytes,
            )?;
            for (reference, expected_bytes) in [
                (
                    &reserved.descriptor_ref,
                    reserved.descriptor_bytes.as_slice(),
                ),
                (
                    &reserved.resume_handle_ref,
                    reserved.resume_handle_bytes.as_slice(),
                ),
                (
                    &reserved.retained_worker_ref,
                    reserved.retained_worker_bytes.as_slice(),
                ),
            ] {
                let actual = transaction
                    .layout
                    .read_object_bytes(reference)
                    .map_err(|_| BootstrapError("read reserved retained object"))?;
                if actual != expected_bytes {
                    return Err(BootstrapError(
                        "reserved retained object bytes changed before application",
                    ));
                }
                verify_object_bytes(transaction.layout, root, reference, &actual, None, false)?;
            }
            if matches!(
                persisted.state,
                RetainedWorkerAuthorityRegistrationRequestStateV1::Applied { .. }
            ) {
                let registration = root
                    .retained_worker_registration_journal
                    .get(&persisted.registration_id)
                    .ok_or(BootstrapError(
                        "applied retained registration has no exact journal",
                    ))?;
                return Ok(RetainedWorkerApplicationV1 {
                    registration: registration.clone(),
                    joined: true,
                });
            }
            validate_reservation_authority(
                root,
                &RetainedWorkerReservationInputV1 {
                    issuer_request_id: persisted.issuer_request_id.clone(),
                    orchestration_session_id: persisted.orchestration_session_id.clone(),
                    expected_authority_store_id: root.authority_store_id.clone(),
                    expected_authority_revision: persisted.authority_revision_before,
                    expected_authority_commitment: persisted
                        .authority_record_commitment_before
                        .clone(),
                    retained_participant_id: persisted.retained_participant_id.clone(),
                    descriptor_bytes: reserved.descriptor_bytes.clone(),
                    resume_handle_bytes: reserved.resume_handle_bytes.clone(),
                },
            )?;
            let SessionNamespaceRecordV1::Authority(current_authority) = root
                .session_namespace_map
                .get(&persisted.orchestration_session_id)
                .ok_or(BootstrapError("retained application authority is absent"))?
            else {
                return Err(BootstrapError(
                    "retained application authority record is not durable",
                ));
            };
            if current_authority.current_policy_ref.as_ref() != Some(&persisted.current_policy_ref)
                || current_authority.world_binding.as_ref() != Some(&persisted.world_binding)
                || current_authority
                    .authoritative_participant_lineage
                    .contains(&persisted.retained_participant_id)
                || current_authority
                    .retained_worker_refs
                    .contains(&reserved.retained_worker_ref)
                || current_authority.updated_at.as_str() > persisted.registered_at.as_str()
            {
                return Err(BootstrapError(
                    "retained application authority inputs are no longer exact",
                ));
            }

            let mut next_authority = current_authority.as_ref().clone();
            next_authority.authority_revision = next_authority
                .authority_revision
                .checked_add(1)
                .ok_or(BootstrapError("retained authority revision overflow"))?;
            next_authority
                .authoritative_participant_lineage
                .push(persisted.retained_participant_id.clone());
            next_authority
                .retained_worker_refs
                .push(reserved.retained_worker_ref.clone());
            next_authority.updated_at = persisted.registered_at.clone();
            let authoritative_lineage_commitment_after =
                AuthorityObjectCommitmentV1::CanonicalSha256 {
                    digest_hex: canonical_sha256(&AuthoritativeLineageHashInputV1 {
                        schema_version: 1,
                        orchestration_session_id: persisted.orchestration_session_id.clone(),
                        participant_ids: next_authority.authoritative_participant_lineage.clone(),
                    })
                    .map_err(|_| BootstrapError("commit retained authority lineage"))?,
                };
            let authority_record_commitment_after =
                canonical_authority_commitment(&next_authority)?;
            let registration = RetainedWorkerAuthorityRegistrationV1 {
                schema_version: 1,
                issuer_request_id: persisted.issuer_request_id.clone(),
                registration_id: persisted.registration_id.clone(),
                orchestration_session_id: persisted.orchestration_session_id.clone(),
                authority_revision_before: persisted.authority_revision_before,
                authority_record_commitment_before: persisted
                    .authority_record_commitment_before
                    .clone(),
                authority_revision_after: next_authority.authority_revision,
                authority_record_commitment_after: authority_record_commitment_after.clone(),
                retained_participant_id: persisted.retained_participant_id.clone(),
                authoritative_lineage_commitment_after,
                descriptor_ref: reserved.descriptor_ref.clone(),
                resume_handle_ref: reserved.resume_handle_ref.clone(),
                retained_worker_ref: reserved.retained_worker_ref.clone(),
                current_policy_ref: persisted.current_policy_ref.clone(),
                world_binding: persisted.world_binding.clone(),
                registered_at: persisted.registered_at.clone(),
            };

            let mut proposed = root.clone();
            proposed.root_revision = proposed.root_revision.checked_add(1).ok_or(
                BootstrapError("retained application root revision overflow"),
            )?;
            proposed.session_namespace_map.insert(
                persisted.orchestration_session_id.clone(),
                SessionNamespaceRecordV1::Authority(Box::new(next_authority)),
            );
            for (reference, bytes) in [
                (
                    &reserved.descriptor_ref,
                    reserved.descriptor_bytes.as_slice(),
                ),
                (
                    &reserved.resume_handle_ref,
                    reserved.resume_handle_bytes.as_slice(),
                ),
                (
                    &reserved.retained_worker_ref,
                    reserved.retained_worker_bytes.as_slice(),
                ),
            ] {
                insert_retained_present_index(&mut proposed, reference, bytes.len() as u64)?;
            }
            let next_request = proposed
                .retained_worker_registration_request_index
                .get_mut(&persisted.issuer_request_id)
                .ok_or(BootstrapError(
                    "retained reservation disappeared during application",
                ))?;
            next_request.state = RetainedWorkerAuthorityRegistrationRequestStateV1::Applied {
                authority_revision_after: registration.authority_revision_after,
                authority_record_commitment_after,
            };
            if proposed
                .retained_worker_registration_journal
                .insert(registration.registration_id.clone(), registration.clone())
                .is_some()
            {
                return Err(BootstrapError(
                    "retained registration journal identity already exists",
                ));
            }
            let candidate = VersionedStateRoot::V2(proposed);
            transaction.validate_publication_candidate(root.root_revision, &candidate)?;
            transaction.reconcile()?;
            publish_versioned_replacement_root(
                transaction.layout,
                transaction.trusted_root,
                &transaction.legacy,
                &candidate,
                system_material()?.root_nonce,
                || {
                    transaction.validate_publication_candidate(root.root_revision, &candidate)?;
                    if crash_point == Some(RetainedApplicationCrashPointV1::BeforeRootPublication) {
                        return Err(BootstrapError(
                            "injected crash before retained authority root publication",
                        ));
                    }
                    Ok(())
                },
            )?;
            if crash_point == Some(RetainedApplicationCrashPointV1::AfterRootPublication) {
                return Err(BootstrapError(
                    "injected crash after retained authority root publication",
                ));
            }
            Ok(RetainedWorkerApplicationV1 {
                registration,
                joined: false,
            })
        })
    }

    fn insert_retained_present_index(
        root: &mut StateRootV2,
        reference: &AuthorityObjectRefV1,
        byte_length: u64,
    ) -> Result<(), BootstrapError> {
        let entry = AuthorityObjectIndexEntryV1 {
            schema_version: 1,
            ref_id: reference.ref_id.clone(),
            object_kind: reference.object_kind,
            object_schema_version: reference.schema_version,
            byte_length,
            storage_state: AuthorityObjectStorageStateV1::Present,
        };
        match root
            .object_index
            .insert(reference.ref_id.clone(), entry.clone())
        {
            None => Ok(()),
            Some(existing) if existing == entry => Ok(()),
            Some(_) => Err(BootstrapError(
                "retained object index conflicts with existing authority",
            )),
        }
    }

    fn validate_reservation_authority(
        root: &StateRootV2,
        input: &RetainedWorkerReservationInputV1,
    ) -> Result<(), BootstrapError> {
        if root.authority_store_id != input.expected_authority_store_id {
            return Err(BootstrapError(
                "retained registration authority store is inexact",
            ));
        }
        let SessionNamespaceRecordV1::Authority(authority) = root
            .session_namespace_map
            .get(&input.orchestration_session_id)
            .ok_or(BootstrapError("retained registration authority is absent"))?
        else {
            return Err(BootstrapError(
                "retained registration authority record is not durable",
            ));
        };
        let commitment = canonical_authority_commitment(authority)?;
        if authority.authority_revision != input.expected_authority_revision
            || commitment != input.expected_authority_commitment
        {
            return Err(BootstrapError(
                "retained registration expected authority is stale",
            ));
        }
        Ok(())
    }

    fn canonical_authority_commitment(
        authority: &crate::execution::agent_runtime::host_session_authority::store_schema::DurableSessionAuthorityV1,
    ) -> Result<AuthorityObjectCommitmentV1, BootstrapError> {
        canonical_sha256(&DurableSessionAuthorityHashInputV1 {
            schema_version: authority.schema_version,
            orchestration_session_id: authority.orchestration_session_id.clone(),
            shell_trace_session_id: authority.shell_trace_session_id.clone(),
            authority_revision: authority.authority_revision,
            origin: authority.origin.clone(),
            authoritative_participant_lineage: authority.authoritative_participant_lineage.clone(),
            active_authoritative_participant_id: authority
                .active_authoritative_participant_id
                .clone(),
            workspace_binding: authority.workspace_binding.clone(),
            world_binding: authority.world_binding.clone(),
            host_attach_contract_ref: authority.host_attach_contract_ref.clone(),
            retained_worker_refs: authority.retained_worker_refs.clone(),
            internal_resume_handle_refs: authority.internal_resume_handle_refs.clone(),
            lifecycle_posture: authority.lifecycle_posture,
            current_policy_ref: authority.current_policy_ref.clone(),
            current_policy_revision: authority.current_policy_revision.clone(),
        })
        .map(|digest_hex| AuthorityObjectCommitmentV1::CanonicalSha256 { digest_hex })
        .map_err(|_| BootstrapError("commit retained registration authority"))
    }

    fn canonical_commitment_for_kind(
        kind: AuthorityObjectKindV1,
        bytes: &[u8],
    ) -> Result<AuthorityObjectCommitmentV1, BootstrapError> {
        canonical_digest(kind, bytes)
            .map(|digest_hex| AuthorityObjectCommitmentV1::CanonicalSha256 { digest_hex })
    }

    fn reserved_ref(
        ref_id: &str,
        object_kind: AuthorityObjectKindV1,
        commitment: &AuthorityObjectCommitmentV1,
    ) -> AuthorityObjectRefV1 {
        AuthorityObjectRefV1 {
            ref_id: ref_id.to_owned(),
            object_kind,
            schema_version: 1,
            commitment: commitment.clone(),
        }
    }

    fn allocate_reserved_canonical_ref(
        layout: &StoreLayout<'_>,
        root: &StateRootV2,
        object_kind: AuthorityObjectKindV1,
        bytes: &[u8],
        allocated_ref_ids: &std::collections::BTreeSet<String>,
    ) -> Result<AuthorityObjectRefV1, BootstrapError> {
        let commitment = canonical_commitment_for_kind(object_kind, bytes)?;
        let mut random = rand::rngs::OsRng;
        for _ in 0..32 {
            let mut entropy = [0_u8; 16];
            random.fill_bytes(&mut entropy);
            let reference = reserved_ref(&object_ref_id(entropy), object_kind, &commitment);
            let reserved = root
                .retained_worker_registration_request_index
                .values()
                .any(|request| {
                    request.descriptor_ref_id == reference.ref_id
                        || request.resume_handle_ref_id == reference.ref_id
                        || request.retained_worker_ref_id == reference.ref_id
                });
            if !root.object_index.contains_key(&reference.ref_id)
                && !reserved
                && !allocated_ref_ids.contains(&reference.ref_id)
                && reserved_object_ref_id_is_globally_absent(layout, &reference.ref_id)?
            {
                return Ok(reference);
            }
        }
        Err(BootstrapError(
            "unable to allocate collision-free retained object ID",
        ))
    }

    fn allocate_registration_id(root: &StateRootV2) -> Result<String, BootstrapError> {
        let mut random = rand::rngs::OsRng;
        for _ in 0..32 {
            let mut entropy = [0_u8; 16];
            random.fill_bytes(&mut entropy);
            let candidate = format!("rr_{}", nonce(entropy));
            if !root
                .retained_worker_registration_request_index
                .values()
                .any(|request| request.registration_id == candidate)
                && !root
                    .retained_worker_registration_journal
                    .contains_key(&candidate)
            {
                return Ok(candidate);
            }
        }
        Err(BootstrapError(
            "unable to allocate collision-free retained registration ID",
        ))
    }

    #[allow(clippy::too_many_arguments)]
    fn validate_reserved_graph(
        layout: &StoreLayout<'_>,
        root: &StateRootV2,
        request: &RetainedWorkerAuthorityRegistrationRequestV1,
        descriptor_ref: &AuthorityObjectRefV1,
        descriptor_bytes: &[u8],
        resume_handle_ref: &AuthorityObjectRefV1,
        resume_handle_bytes: &[u8],
        retained_worker_ref: &AuthorityObjectRefV1,
        retained_worker_bytes: &[u8],
    ) -> Result<(), BootstrapError> {
        verify_object_bytes(layout, root, descriptor_ref, descriptor_bytes, None, false)?;
        verify_object_bytes(
            layout,
            root,
            resume_handle_ref,
            resume_handle_bytes,
            None,
            false,
        )?;
        verify_object_bytes(
            layout,
            root,
            retained_worker_ref,
            retained_worker_bytes,
            None,
            false,
        )?;
        let descriptor: AgentDescriptorHashInputV1 =
            canonical_json::from_slice(descriptor_bytes)
                .map_err(|_| BootstrapError("decode reserved retained descriptor"))?;
        let resume: ResumeHandleHashInputV1 = canonical_json::from_slice(resume_handle_bytes)
            .map_err(|_| BootstrapError("decode reserved retained resume handle"))?;
        let worker: RetainedWorkerObjectHashInputV1 =
            canonical_json::from_slice(retained_worker_bytes)
                .map_err(|_| BootstrapError("decode reserved retained worker"))?;
        if descriptor_ref.ref_id != request.descriptor_ref_id
            || descriptor_ref.commitment != request.descriptor_commitment
            || resume_handle_ref.ref_id != request.resume_handle_ref_id
            || resume_handle_ref.commitment != request.resume_handle_commitment
            || retained_worker_ref.ref_id != request.retained_worker_ref_id
            || retained_worker_ref.commitment != request.retained_worker_commitment
            || descriptor.descriptor.execution_scope != AgentExecutionScopeV1::World
            || resume.orchestration_session_id != request.orchestration_session_id
            || resume.participant_id != request.retained_participant_id
            || resume.backend_id != descriptor.descriptor.backend_id
            || resume.protocol != descriptor.descriptor.protocol
            || worker.orchestration_session_id != request.orchestration_session_id
            || worker.participant_id != request.retained_participant_id
            || worker.world_binding != request.world_binding
            || worker.descriptor_ref != *descriptor_ref
            || worker.resume_handle_ref != *resume_handle_ref
            || worker.policy_ref != request.current_policy_ref
        {
            return Err(BootstrapError("reserved retained object graph is inexact"));
        }
        Ok(())
    }

    fn system_timestamp() -> Result<TimestampV1, BootstrapError> {
        TimestampV1::parse(chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Nanos, true))
            .map_err(|_| BootstrapError("sample retained registration timestamp"))
    }

    fn generated_object_ref_v2(
        layout: &StoreLayout<'_>,
        root: &StateRootV2,
        object_kind: AuthorityObjectKindV1,
        bytes: &[u8],
        context: Option<&ObjectVerificationContextV1>,
    ) -> Result<AuthorityObjectRefV1, BootstrapError> {
        let commitment = if let Some(domain) = sensitive_domain(object_kind) {
            let context =
                context.ok_or(BootstrapError("sensitive object parent context is missing"))?;
            let record = root
                .commitment_key_registry
                .get(&root.active_commitment_key_id)
                .filter(|record| record.state == AuthorityStoreCommitmentKeyStateV1::Active)
                .ok_or(BootstrapError("active commitment key is unavailable"))?;
            let key = layout
                .keys
                .open_file(&format!("{}.key", record.key_id))
                .map_err(|_| BootstrapError("open active commitment key"))?;
            let envelope = AuthorityStoreCommitmentKeyFileV1::decode(
                &key.read_all()
                    .map_err(|_| BootstrapError("read active commitment key"))?,
            )
            .map_err(|_| BootstrapError("decode active commitment key"))?;
            AuthorityObjectCommitmentV1::StoreHmacSha256 {
                key_id: record.key_id.clone(),
                domain: String::from_utf8(domain.as_bytes().to_vec())
                    .map_err(|_| BootstrapError("sensitive object domain is invalid"))?,
                digest_hex: store_hmac_sha256(
                    &envelope.secret_key,
                    domain,
                    &root.authority_store_id,
                    &context.intent_id,
                    Some(&context.run_id),
                    bytes,
                )
                .map_err(|_| BootstrapError("commit sensitive object"))?,
            }
        } else {
            AuthorityObjectCommitmentV1::CanonicalSha256 {
                digest_hex: canonical_digest(object_kind, bytes)?,
            }
        };
        let mut random = rand::thread_rng();
        for _ in 0..32 {
            let mut entropy = [0_u8; 16];
            random.fill_bytes(&mut entropy);
            let reference = AuthorityObjectRefV1 {
                ref_id: object_ref_id(entropy),
                object_kind,
                schema_version: 1,
                commitment: commitment.clone(),
            };
            if !root.object_index.contains_key(&reference.ref_id) {
                return Ok(reference);
            }
        }
        Err(BootstrapError(
            "unable to allocate collision-free object ID",
        ))
    }

    pub(super) fn read_root(path: &std::path::Path) -> Result<StateRootV1, BootstrapError> {
        with_existing_semantic_preflight(path, |transaction| Ok(transaction.root.clone()))
    }

    pub(super) fn read_opened_root(
        root: &TrustedAuthorityRoot,
    ) -> Result<StateRootV1, BootstrapError> {
        with_opened_existing_semantic_preflight(root, |transaction| Ok(transaction.root.clone()))
    }

    pub(super) fn compare_and_swap_root(
        path: &std::path::Path,
        expected: &ExpectedRevisionsV1,
        proposed: &StateRootV1,
    ) -> Result<TransactionCommitOutcomeV1, BootstrapError> {
        compare_and_swap_root_with(path, expected, proposed, system_material()?.root_nonce)
    }

    pub(super) fn compare_and_swap_opened_root(
        root: &TrustedAuthorityRoot,
        expected: &ExpectedRevisionsV1,
        proposed: &StateRootV1,
    ) -> Result<TransactionCommitOutcomeV1, BootstrapError> {
        compare_and_swap_opened_root_with(root, expected, proposed, system_material()?.root_nonce)
    }

    pub(super) fn compare_and_swap_opened_root_exact_current(
        root: &TrustedAuthorityRoot,
        exact_current: &StateRootV1,
        expected: &ExpectedRevisionsV1,
        proposed: &StateRootV1,
    ) -> Result<TransactionCommitOutcomeV1, BootstrapError> {
        compare_and_swap_opened_root_with_exact_current(
            root,
            Some(exact_current),
            expected,
            proposed,
            system_material()?.root_nonce,
        )
    }

    pub(super) fn begin_legacy_state_store_transaction(
        path: &std::path::Path,
    ) -> Result<LegacyStateStoreTransactionV1, BootstrapError> {
        begin_legacy_transaction(path)
    }

    pub(super) fn begin_legacy_state_store_transaction_for_identity(
        path: &std::path::Path,
        expected: &CanonicalDirectoryV1,
    ) -> Result<LegacyStateStoreTransactionV1, BootstrapError> {
        begin_legacy_transaction_for_identity(path, expected)
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
    pub(super) fn rotate_commitment_key_versioned_test(
        path: &std::path::Path,
        material: InitializationMaterialV1,
        stop: Option<KeyLifecycleCrashPointV1>,
    ) -> Result<VersionedStateRoot, BootstrapError> {
        let root = with_existing_versioned_semantic_preflight(path, |transaction| {
            Ok(transaction.root.clone())
        })?;
        rotate_commitment_key_versioned_with(path, root.root_revision(), Some(material), stop)
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
    pub(super) fn retire_commitment_key_versioned_test(
        path: &std::path::Path,
        key_id: &str,
        root_nonce: [u8; 16],
        stop: Option<KeyLifecycleCrashPointV1>,
    ) -> Result<VersionedStateRoot, BootstrapError> {
        let root = with_existing_versioned_semantic_preflight(path, |transaction| {
            Ok(transaction.root.clone())
        })?;
        retire_commitment_key_versioned_with(path, key_id, root.root_revision(), root_nonce, stop)
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
        let root = TrustedAuthorityRoot::open(path)
            .map_err(|_| BootstrapError("open trusted authority root"))?;
        prepare_typed_object_opened_with(
            &root,
            expected_root_revision,
            reference,
            bytes,
            context,
            nonce_bytes,
        )
    }

    fn prepare_typed_object_opened_with(
        root: &TrustedAuthorityRoot,
        expected_root_revision: u64,
        reference: &AuthorityObjectRefV1,
        bytes: &[u8],
        context: Option<&ObjectVerificationContextV1>,
        nonce_bytes: [u8; 16],
    ) -> Result<ObjectPublicationOutcomeV1, BootstrapError> {
        with_opened_existing_semantic_preflight(root, |transaction| {
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
        mut validate_candidate: impl FnMut() -> Result<(), BootstrapError>,
    ) -> Result<(), BootstrapError> {
        legacy
            .revalidate(layout.bootstrap)
            .map_err(|_| BootstrapError("revalidate legacy state before root temp"))?;
        validate_candidate()?;
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

    fn publish_versioned_replacement_root(
        layout: &StoreLayout<'_>,
        trusted_root: &TrustedAuthorityRoot,
        legacy: &LegacyObservation,
        root: &VersionedStateRoot,
        nonce_bytes: [u8; 16],
        mut validate_candidate: impl FnMut() -> Result<(), BootstrapError>,
    ) -> Result<(), BootstrapError> {
        legacy
            .revalidate(layout.bootstrap)
            .map_err(|_| BootstrapError("revalidate legacy state before versioned root temp"))?;
        validate_candidate()?;
        let temp_name = TempNameV1::Root {
            root_revision: root.root_revision(),
            nonce: nonce(nonce_bytes),
        }
        .file_name();
        let bytes = root
            .to_canonical_bytes()
            .map_err(|_| BootstrapError("encode versioned replacement state root"))?;
        let mut temp = layout
            .tmp
            .create_file(&temp_name)
            .map_err(|_| BootstrapError("create versioned replacement state root temp"))?;
        temp.write_all(&bytes)
            .map_err(|_| BootstrapError("write versioned replacement state root temp"))?;
        temp.sync()
            .map_err(|_| BootstrapError("sync versioned replacement state root temp"))?;
        legacy
            .revalidate(layout.bootstrap)
            .map_err(|_| BootstrapError("revalidate legacy state before versioned root replace"))?;
        validate_candidate()?;
        trusted_root
            .revalidate()
            .map_err(|_| BootstrapError("revalidate trusted root before versioned replace"))?;
        layout
            .tmp
            .rename_replace(&temp_name, temp, &layout.authority, ROOT_FILE)
            .map_err(|_| BootstrapError("replace versioned authority state root"))
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
        let root = TrustedAuthorityRoot::open(path)
            .map_err(|_| BootstrapError("open trusted authority root"))?;
        classify_opened_checked(&root)
    }

    fn classify_opened_checked(
        root: &TrustedAuthorityRoot,
    ) -> Result<BootstrapClassificationV1, BootstrapError> {
        with_opened_semantic_preflight(
            root,
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
    #[cfg(test)]
    use super::RetainedReservationCrashPointV1;
    use super::{
        BootstrapClassificationV1, BootstrapError, GeneratedObjectV1, ObjectPublicationOutcomeV1,
        ObjectVerificationContextV1, RetainedApplicationCrashPointV1, RetainedWorkerApplicationV1,
        RetainedWorkerReservationInputV1, RetainedWorkerReservationV1, RootUpgradeOutcomeV1,
        TrustedAuthorityRoot,
    };
    #[cfg(test)]
    use crate::execution::agent_runtime::host_session_authority::schema::{
        AuthorityObjectCommitmentV1, TimestampV1,
    };
    use crate::execution::agent_runtime::host_session_authority::schema::{
        AuthorityObjectKindV1, AuthorityObjectRefV1, CanonicalDirectoryV1, WorldBindingV1,
    };
    use crate::execution::agent_runtime::host_session_authority::store_schema::StateRootV1;
    #[cfg(test)]
    use crate::execution::agent_runtime::host_session_authority::store_schema::StateRootV2;

    pub(crate) struct LegacyStateStoreTransactionV1;

    pub(crate) struct RetainedWorkerAdmissionStorageV1;

    pub(crate) struct RetainedWorkerAdmissionStorageTransactionV1;

    impl RetainedWorkerAdmissionStorageV1 {
        pub(crate) fn authority_store_id(&self) -> &str {
            ""
        }

        pub(crate) fn transaction<T>(
            &self,
            _operation: impl FnOnce(
                &mut RetainedWorkerAdmissionStorageTransactionV1,
            ) -> Result<T, BootstrapError>,
        ) -> Result<T, BootstrapError> {
            Err(BootstrapError(
                "retained admission storage is unsupported on this platform",
            ))
        }
    }

    impl RetainedWorkerAdmissionStorageTransactionV1 {
        pub(crate) fn read_registry(&self) -> Result<Option<Vec<u8>>, BootstrapError> {
            Err(BootstrapError(
                "retained admission storage is unsupported on this platform",
            ))
        }

        pub(crate) fn read_keys(&self) -> Result<Vec<(String, Vec<u8>)>, BootstrapError> {
            Err(BootstrapError(
                "retained admission storage is unsupported on this platform",
            ))
        }

        pub(crate) fn stage_key_temp(
            &self,
            _temp_name: &str,
            _bytes: &[u8],
        ) -> Result<(), BootstrapError> {
            Err(BootstrapError(
                "retained admission storage is unsupported on this platform",
            ))
        }

        pub(crate) fn publish_staged_key_no_replace(
            &self,
            _temp_name: &str,
            _key_name: &str,
        ) -> Result<(), BootstrapError> {
            Err(BootstrapError(
                "retained admission storage is unsupported on this platform",
            ))
        }

        pub(crate) fn remove_key(&self, _key_name: &str) -> Result<(), BootstrapError> {
            Err(BootstrapError(
                "retained admission storage is unsupported on this platform",
            ))
        }

        pub(crate) fn publish_registry_no_replace(
            &self,
            _temp_name: &str,
            _bytes: &[u8],
        ) -> Result<(), BootstrapError> {
            Err(BootstrapError(
                "retained admission storage is unsupported on this platform",
            ))
        }
    }

    pub(super) fn retained_worker_admission_storage_opened(
        _root: &TrustedAuthorityRoot,
    ) -> Result<RetainedWorkerAdmissionStorageV1, BootstrapError> {
        Err(BootstrapError(
            "retained admission storage is unsupported on this platform",
        ))
    }

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

    pub(super) fn classify_opened(_root: &TrustedAuthorityRoot) -> BootstrapClassificationV1 {
        BootstrapClassificationV1::CorruptOrUnsupported
    }

    pub(super) fn bootstrap(_path: &std::path::Path) -> Result<StateRootV1, BootstrapError> {
        Err(BootstrapError(
            "authority store is unsupported on this platform",
        ))
    }

    pub(super) fn bootstrap_opened(
        _root: &TrustedAuthorityRoot,
    ) -> Result<StateRootV1, BootstrapError> {
        Err(BootstrapError(
            "authority store is unsupported on this platform",
        ))
    }

    pub(super) fn upgrade_greenfield_root_opened(
        _root: &TrustedAuthorityRoot,
    ) -> Result<RootUpgradeOutcomeV1, BootstrapError> {
        Err(BootstrapError(
            "authority store is unsupported on this platform",
        ))
    }

    pub(super) fn read_opened_root_v2(
        _root: &TrustedAuthorityRoot,
    ) -> Result<super::StateRootV2, BootstrapError> {
        Err(BootstrapError(
            "authority store is unsupported on this platform",
        ))
    }

    pub(super) fn reserve_retained_worker_registration_opened(
        _root: &TrustedAuthorityRoot,
        _input: &RetainedWorkerReservationInputV1,
        _build_worker: impl Fn(
            &AuthorityObjectRefV1,
            &AuthorityObjectRefV1,
            &AuthorityObjectRefV1,
            &WorldBindingV1,
        ) -> Result<Vec<u8>, &'static str>,
    ) -> Result<RetainedWorkerReservationV1, BootstrapError> {
        Err(BootstrapError(
            "retained registration is unsupported on this platform",
        ))
    }

    pub(super) fn publish_reserved_retained_object_opened(
        _root: &TrustedAuthorityRoot,
        _reserved: &RetainedWorkerReservationV1,
        _reference: &AuthorityObjectRefV1,
        _bytes: &[u8],
    ) -> Result<ObjectPublicationOutcomeV1, BootstrapError> {
        Err(BootstrapError(
            "retained object publication is unsupported on this platform",
        ))
    }

    pub(super) fn apply_reserved_retained_worker_registration_opened(
        _root: &TrustedAuthorityRoot,
        _reserved: &RetainedWorkerReservationV1,
    ) -> Result<RetainedWorkerApplicationV1, BootstrapError> {
        Err(BootstrapError(
            "retained authority application is unsupported on this platform",
        ))
    }

    #[cfg(test)]
    pub(super) fn apply_reserved_retained_worker_registration_with_crash_point_opened(
        _root: &TrustedAuthorityRoot,
        _reserved: &RetainedWorkerReservationV1,
        _crash_point: RetainedApplicationCrashPointV1,
    ) -> Result<RetainedWorkerApplicationV1, BootstrapError> {
        Err(BootstrapError(
            "retained authority application is unsupported on this platform",
        ))
    }

    #[cfg(test)]
    pub(super) fn reserve_retained_worker_registration_at_opened(
        _root: &TrustedAuthorityRoot,
        _input: &RetainedWorkerReservationInputV1,
        _registered_at: TimestampV1,
        _crash_point: Option<RetainedReservationCrashPointV1>,
        _build_worker: impl Fn(
            &AuthorityObjectRefV1,
            &AuthorityObjectRefV1,
            &AuthorityObjectRefV1,
            &WorldBindingV1,
        ) -> Result<Vec<u8>, &'static str>,
    ) -> Result<RetainedWorkerReservationV1, BootstrapError> {
        Err(BootstrapError(
            "retained registration is unsupported on this platform",
        ))
    }

    #[cfg(test)]
    pub(super) fn reserved_object_ref_id_is_globally_absent_test(
        _path: &std::path::Path,
        _ref_id: &str,
    ) -> Result<bool, BootstrapError> {
        Err(BootstrapError(
            "retained collision checks are unsupported on this platform",
        ))
    }

    #[cfg(test)]
    pub(super) fn retained_reservation_authority_matches_test(
        _root: &StateRootV2,
        _orchestration_session_id: &str,
        _expected_authority_store_id: &str,
        _expected_authority_revision: u64,
        _expected_authority_commitment: &AuthorityObjectCommitmentV1,
    ) -> Result<(), BootstrapError> {
        Err(BootstrapError(
            "retained authority checks are unsupported on this platform",
        ))
    }

    pub(super) fn prepare_generated_object_v2_opened(
        _root: &TrustedAuthorityRoot,
        _expected_root_revision: u64,
        _object_kind: AuthorityObjectKindV1,
        _bytes: &[u8],
        _context: Option<&ObjectVerificationContextV1>,
    ) -> Result<GeneratedObjectV1, BootstrapError> {
        Err(BootstrapError(
            "authority store is unsupported on this platform",
        ))
    }

    pub(super) fn allocate_sensitive_object_ref_v2_opened(
        _root: &TrustedAuthorityRoot,
        _expected_root_revision: u64,
        _object_kind: AuthorityObjectKindV1,
        _bytes: &[u8],
        _context: &ObjectVerificationContextV1,
    ) -> Result<AuthorityObjectRefV1, BootstrapError> {
        Err(BootstrapError(
            "authority store is unsupported on this platform",
        ))
    }

    pub(super) fn prepare_typed_object_v2_opened(
        _root: &TrustedAuthorityRoot,
        _expected_root_revision: u64,
        _reference: &AuthorityObjectRefV1,
        _bytes: &[u8],
        _context: Option<&ObjectVerificationContextV1>,
    ) -> Result<ObjectPublicationOutcomeV1, BootstrapError> {
        Err(BootstrapError(
            "authority store is unsupported on this platform",
        ))
    }

    pub(super) fn read_typed_object_v2_opened(
        _root: &TrustedAuthorityRoot,
        _expected_root_revision: u64,
        _reference: &AuthorityObjectRefV1,
        _context: Option<&ObjectVerificationContextV1>,
    ) -> Result<Vec<u8>, BootstrapError> {
        Err(BootstrapError(
            "authority store is unsupported on this platform",
        ))
    }

    pub(super) fn commit_v2_root_exact_current_opened(
        _root: &TrustedAuthorityRoot,
        _exact_current: &super::StateRootV2,
        _proposed: &super::StateRootV2,
        _publication_guard: impl FnMut() -> Result<(), BootstrapError>,
    ) -> Result<super::StateRootV2, BootstrapError> {
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

    pub(super) fn prepare_typed_object_opened(
        _root: &TrustedAuthorityRoot,
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

    pub(super) fn read_opened_root(
        _root: &TrustedAuthorityRoot,
    ) -> Result<StateRootV1, BootstrapError> {
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

    pub(super) fn compare_and_swap_opened_root(
        _root: &TrustedAuthorityRoot,
        _expected: &super::ExpectedRevisionsV1,
        _proposed: &StateRootV1,
    ) -> Result<super::TransactionCommitOutcomeV1, BootstrapError> {
        Err(BootstrapError(
            "authority store is unsupported on this platform",
        ))
    }

    pub(super) fn compare_and_swap_opened_root_exact_current(
        _root: &TrustedAuthorityRoot,
        _exact_current: &StateRootV1,
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

    pub(super) fn begin_legacy_state_store_transaction_for_identity(
        _path: &std::path::Path,
        _expected: &CanonicalDirectoryV1,
    ) -> Result<LegacyStateStoreTransactionV1, BootstrapError> {
        Err(BootstrapError(
            "authority store is unsupported on this platform",
        ))
    }
}

#[cfg(all(test, any(target_os = "linux", target_os = "macos")))]
#[path = "store_tests.rs"]
mod tests;
