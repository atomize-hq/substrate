use super::super::LegacyStateStoreDirectoryEntryV1;
use super::legacy::ObservedLegacyDirectory;
use super::*;
#[cfg(target_os = "linux")]
use crate::execution::agent_runtime::dispatch_policy_commitment::ReadOnlyAuthoritySnapshotErrorV1;
#[cfg(target_os = "linux")]
use std::cell::RefCell;
use std::collections::BTreeMap;
#[cfg(target_os = "linux")]
use std::path::Path;
use std::sync::Arc;

#[cfg(all(test, target_os = "linux"))]
std::thread_local! {
    static SEMANTIC_PREFLIGHT_COUNT: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}

#[cfg(all(test, target_os = "linux"))]
pub(in crate::execution::agent_runtime::host_session_authority) fn semantic_preflight_count_for_test(
) -> usize {
    SEMANTIC_PREFLIGHT_COUNT.with(std::cell::Cell::get)
}

const WORLD_WORK_RECEIPT_REGISTRY_FILE: &str = "world-work-receipt-registry-v1.json";
const WORLD_WORK_RECEIPT_REGISTRY_PREFIX: &str = "world-work-receipt-registry-v1";
const WORLD_WORK_RECEIPT_REGISTRY_TEMP_PREFIX: &str = "world-work-receipt-registry-v1--";
const WORLD_WORK_RECEIPT_REGISTRY_TEMP_SUFFIX: &str = ".tmp";
const WORLD_WORK_EXECUTION_SUPERVISOR_FILE: &str = "world-work-execution-supervisor-v1.json";
const WORLD_WORK_EXECUTION_SUPERVISOR_PREFIX: &str = "world-work-execution-supervisor-v1";
const WORLD_WORK_EXECUTION_SUPERVISOR_TEMP_PREFIX: &str = "world-work-execution-supervisor-v1--";
const WORLD_WORK_EXECUTION_SUPERVISOR_TEMP_SUFFIX: &str = ".tmp";
const POST_HSA_OBLIGATION_LEDGER_DIRECTORY: &str = "obligation-ledger";
const POST_HSA_OBLIGATION_LEDGER_TEMP_PREFIX: &str = ".auto-attach-ledger--";
const POST_HSA_OBLIGATION_LEDGER_TEMP_SUFFIX: &str = ".tmp";

#[cfg(all(test, target_os = "linux"))]
std::thread_local! {
    static E2_RM_BEFORE_FINAL_VERIFY_HOOK: RefCell<Option<Box<dyn FnOnce()>>> =
        RefCell::new(None);
    static RECONCILING_CURRENT_BEFORE_FINAL_VERIFY_HOOK: RefCell<Option<Box<dyn FnOnce()>>> =
        RefCell::new(None);
    static ORDINARY_SUPERVISOR_BEFORE_PUBLICATION_HOOK: RefCell<Option<Box<dyn FnOnce()>>> =
        RefCell::new(None);
}

#[cfg(all(test, target_os = "linux"))]
pub(in crate::execution::agent_runtime::host_session_authority) fn set_reconciling_current_before_final_verify_hook(
    hook: impl FnOnce() + 'static,
) {
    RECONCILING_CURRENT_BEFORE_FINAL_VERIFY_HOOK.with(|slot| {
        assert!(slot.borrow_mut().replace(Box::new(hook)).is_none());
    });
}

#[cfg(all(test, target_os = "linux"))]
fn run_reconciling_current_before_final_verify_hook() {
    let hook = RECONCILING_CURRENT_BEFORE_FINAL_VERIFY_HOOK.with(|slot| slot.borrow_mut().take());
    if let Some(hook) = hook {
        hook();
    }
}

#[cfg(all(test, target_os = "linux"))]
pub(crate) fn set_ordinary_supervisor_before_publication_hook(hook: impl FnOnce() + 'static) {
    ORDINARY_SUPERVISOR_BEFORE_PUBLICATION_HOOK.with(|slot| {
        assert!(slot.borrow_mut().replace(Box::new(hook)).is_none());
    });
}

#[cfg(all(test, target_os = "linux"))]
fn run_ordinary_supervisor_before_publication_hook() {
    let hook = ORDINARY_SUPERVISOR_BEFORE_PUBLICATION_HOOK.with(|slot| slot.borrow_mut().take());
    if let Some(hook) = hook {
        hook();
    }
}

#[cfg(all(test, target_os = "linux"))]
pub(crate) fn set_e2_rm_before_final_verify_hook(hook: impl FnOnce() + 'static) {
    E2_RM_BEFORE_FINAL_VERIFY_HOOK.with(|slot| {
        assert!(slot.borrow_mut().replace(Box::new(hook)).is_none());
    });
}

#[cfg(all(test, target_os = "linux"))]
fn run_e2_rm_before_final_verify_hook() {
    E2_RM_BEFORE_FINAL_VERIFY_HOOK.with(|slot| {
        if let Some(hook) = slot.borrow_mut().take() {
            hook();
        }
    });
}

#[cfg(target_os = "linux")]
const E2_RM_AUTHORITY_NAMESPACE: &str = "authority-v1";
#[cfg(target_os = "linux")]
const E2_RM_AUTHORITY_LOCK_NAMESPACE: &str = "authority-v1/lock";
#[cfg(target_os = "linux")]
const E2_RM_AUTHORITY_TEMP_NAMESPACE: &str = "authority-v1/tmp";
#[cfg(target_os = "linux")]
const E2_RM_B1_NAMESPACE: &str = "run/agent-hub/world-work-receipt-registry-v1";

#[cfg(target_os = "linux")]
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum WorldWorkReceiptRegistryPhysicalReadV1 {
    Absent,
    Present(WorldWorkReceiptRegistryPhysicalSnapshotV1),
}

#[cfg(target_os = "linux")]
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct WorldWorkReceiptRegistryPhysicalSnapshotV1 {
    registry_bytes: Vec<u8>,
}

#[cfg(target_os = "linux")]
impl WorldWorkReceiptRegistryPhysicalSnapshotV1 {
    pub(crate) fn registry_bytes(&self) -> &[u8] {
        &self.registry_bytes
    }
}

#[cfg(target_os = "linux")]
pub(super) struct ReadOnlyFileGuardV1 {
    pub(super) entry: DirectoryEntry,
    pub(super) metadata: TrustedEntryMetadataV1,
    pub(super) bytes: Vec<u8>,
    pub(super) namespace: &'static str,
}

#[cfg(target_os = "linux")]
impl ReadOnlyFileGuardV1 {
    pub(super) fn capture(
        parent: &TrustedDirectory,
        entry: &DirectoryEntry,
        namespace: &'static str,
    ) -> Result<Self, ReadOnlyAuthoritySnapshotErrorV1> {
        if entry.kind != EntryKind::RegularFile {
            return Err(ReadOnlyAuthoritySnapshotErrorV1::UnsafeNamespaceEntry {
                namespace,
                name: entry.name.clone(),
            });
        }
        let (bytes, metadata) = parent
            .read_regular_file_entry_stable_single_link(entry)
            .map_err(|_| ReadOnlyAuthoritySnapshotErrorV1::UnsafeFileMetadata {
                namespace,
                name: entry.name.clone(),
            })?;
        Ok(Self {
            entry: entry.clone(),
            metadata,
            bytes,
            namespace,
        })
    }

    fn verify(&self, parent: &TrustedDirectory) -> Result<(), ReadOnlyAuthoritySnapshotErrorV1> {
        let (bytes, metadata) = parent
            .read_regular_file_entry_stable_single_link(&self.entry)
            .map_err(|_| ReadOnlyAuthoritySnapshotErrorV1::SnapshotEntryChanged {
                namespace: self.namespace,
                name: self.entry.name.clone(),
            })?;
        if bytes != self.bytes || metadata != self.metadata {
            return Err(ReadOnlyAuthoritySnapshotErrorV1::SnapshotEntryChanged {
                namespace: self.namespace,
                name: self.entry.name.clone(),
            });
        }
        Ok(())
    }
}

#[cfg(target_os = "linux")]
pub(super) struct ReadOnlyDirectoryGuardV1 {
    pub(super) entry: DirectoryEntry,
    pub(super) directory: TrustedDirectory,
    pub(super) metadata: TrustedEntryMetadataV1,
    pub(super) manifest: Vec<DirectoryEntry>,
    pub(super) namespace: &'static str,
}

#[cfg(target_os = "linux")]
impl ReadOnlyDirectoryGuardV1 {
    pub(super) fn capture(
        parent: &TrustedDirectory,
        entry: &DirectoryEntry,
        namespace: &'static str,
    ) -> Result<Self, ReadOnlyAuthoritySnapshotErrorV1> {
        if entry.kind != EntryKind::Directory {
            return Err(ReadOnlyAuthoritySnapshotErrorV1::UnsafeNamespaceEntry {
                namespace,
                name: entry.name.clone(),
            });
        }
        parent.revalidate_entry(entry).map_err(|_| {
            ReadOnlyAuthoritySnapshotErrorV1::SnapshotEntryChanged {
                namespace,
                name: entry.name.clone(),
            }
        })?;
        let directory = parent.open_directory(&entry.name).map_err(|_| {
            ReadOnlyAuthoritySnapshotErrorV1::UnsafeNamespaceEntry {
                namespace,
                name: entry.name.clone(),
            }
        })?;
        let metadata = directory.metadata().map_err(|_| {
            ReadOnlyAuthoritySnapshotErrorV1::UnsafeNamespaceEntry {
                namespace,
                name: entry.name.clone(),
            }
        })?;
        let manifest = directory
            .entries()
            .map_err(|_| ReadOnlyAuthoritySnapshotErrorV1::Io {
                operation: "enumerate read-only authority namespace",
            })?;
        parent.revalidate_entry(entry).map_err(|_| {
            ReadOnlyAuthoritySnapshotErrorV1::SnapshotEntryChanged {
                namespace,
                name: entry.name.clone(),
            }
        })?;
        if parent.entry_metadata(entry).map_err(|_| {
            ReadOnlyAuthoritySnapshotErrorV1::SnapshotEntryChanged {
                namespace,
                name: entry.name.clone(),
            }
        })? != metadata
        {
            return Err(ReadOnlyAuthoritySnapshotErrorV1::SnapshotEntryChanged {
                namespace,
                name: entry.name.clone(),
            });
        }
        Ok(Self {
            entry: entry.clone(),
            directory,
            metadata,
            manifest,
            namespace,
        })
    }

    fn verify(&self, parent: &TrustedDirectory) -> Result<(), ReadOnlyAuthoritySnapshotErrorV1> {
        parent.revalidate_entry(&self.entry).map_err(|_| {
            ReadOnlyAuthoritySnapshotErrorV1::SnapshotEntryChanged {
                namespace: self.namespace,
                name: self.entry.name.clone(),
            }
        })?;
        if parent.entry_metadata(&self.entry).map_err(|_| {
            ReadOnlyAuthoritySnapshotErrorV1::SnapshotEntryChanged {
                namespace: self.namespace,
                name: self.entry.name.clone(),
            }
        })? != self.metadata
            || self.directory.metadata().map_err(|_| {
                ReadOnlyAuthoritySnapshotErrorV1::NamespaceChanged {
                    namespace: self.namespace,
                }
            })? != self.metadata
            || self.directory.entries().map_err(|_| {
                ReadOnlyAuthoritySnapshotErrorV1::NamespaceChanged {
                    namespace: self.namespace,
                }
            })? != self.manifest
        {
            return Err(ReadOnlyAuthoritySnapshotErrorV1::NamespaceChanged {
                namespace: self.namespace,
            });
        }
        Ok(())
    }
}

#[cfg(target_os = "linux")]
pub(super) struct ReadOnlyE2NamespaceGuardV1 {
    pub(super) root: ReadOnlyDirectoryGuardV1,
    pub(super) keys: Option<ReadOnlyDirectoryGuardV1>,
    pub(super) temporary: Option<ReadOnlyDirectoryGuardV1>,
    pub(super) registry: Option<ReadOnlyFileGuardV1>,
    pub(super) key_files: Vec<ReadOnlyFileGuardV1>,
    pub(super) key_temporary_files: Vec<ReadOnlyFileGuardV1>,
    pub(super) registry_temporary_files: Vec<ReadOnlyFileGuardV1>,
}

#[cfg(target_os = "linux")]
pub(super) fn verify_e2_population_source_guard(
    guard: &ReadOnlyE2NamespaceGuardV1,
    authority: &TrustedDirectory,
) -> Result<(), ReadOnlyAuthoritySnapshotErrorV1> {
    guard.verify(authority)
}

#[cfg(target_os = "linux")]
impl ReadOnlyE2NamespaceGuardV1 {
    fn verify(&self, authority: &TrustedDirectory) -> Result<(), ReadOnlyAuthoritySnapshotErrorV1> {
        self.root.verify(authority)?;
        if let Some(keys) = &self.keys {
            keys.verify(&self.root.directory)?;
            for key in self.key_files.iter().chain(&self.key_temporary_files) {
                key.verify(&keys.directory)?;
            }
        }
        if let Some(temporary) = &self.temporary {
            temporary.verify(&self.root.directory)?;
            for file in &self.registry_temporary_files {
                file.verify(&temporary.directory)?;
            }
        }
        if let Some(registry) = &self.registry {
            registry.verify(&self.root.directory)?;
        }
        self.root.verify(authority)
    }
}

#[cfg(target_os = "linux")]
struct ReadOnlyB1NamespaceGuardV1 {
    run: ReadOnlyDirectoryGuardV1,
    agent_hub: Option<ReadOnlyDirectoryGuardV1>,
    registry: Option<ReadOnlyFileGuardV1>,
    temporary_files: Vec<ReadOnlyFileGuardV1>,
}

#[cfg(target_os = "linux")]
impl ReadOnlyB1NamespaceGuardV1 {
    fn verify(&self, bootstrap: &TrustedDirectory) -> Result<(), ReadOnlyAuthoritySnapshotErrorV1> {
        self.run.verify(bootstrap)?;
        if let Some(agent_hub) = &self.agent_hub {
            agent_hub.verify(&self.run.directory)?;
            if let Some(registry) = &self.registry {
                registry.verify(&agent_hub.directory)?;
            }
            for temporary in &self.temporary_files {
                temporary.verify(&agent_hub.directory)?;
            }
            agent_hub.verify(&self.run.directory)?;
        }
        self.run.verify(bootstrap)
    }

    fn verify_for_supervisor_owner(
        &self,
        bootstrap: &TrustedDirectory,
    ) -> Result<(), ReadOnlyAuthoritySnapshotErrorV1> {
        self.run.verify(bootstrap)?;
        let hub =
            self.agent_hub
                .as_ref()
                .ok_or(ReadOnlyAuthoritySnapshotErrorV1::NamespaceChanged {
                    namespace: E2_RM_B1_NAMESPACE,
                })?;
        self.run
            .directory
            .revalidate_entry(&hub.entry)
            .map_err(|_| ReadOnlyAuthoritySnapshotErrorV1::NamespaceChanged {
                namespace: E2_RM_B1_NAMESPACE,
            })?;
        let metadata = hub.directory.metadata().map_err(|_| {
            ReadOnlyAuthoritySnapshotErrorV1::NamespaceChanged {
                namespace: E2_RM_B1_NAMESPACE,
            }
        })?;
        if !metadata.same_identity_and_security(hub.metadata) {
            return Err(ReadOnlyAuthoritySnapshotErrorV1::NamespaceChanged {
                namespace: E2_RM_B1_NAMESPACE,
            });
        }
        let receipt_and_other_sources = |entries: &[DirectoryEntry]| {
            entries
                .iter()
                .filter(|entry| {
                    !entry
                        .name
                        .starts_with(WORLD_WORK_EXECUTION_SUPERVISOR_PREFIX)
                })
                .cloned()
                .collect::<Vec<_>>()
        };
        let current = hub.directory.entries().map_err(|_| {
            ReadOnlyAuthoritySnapshotErrorV1::NamespaceChanged {
                namespace: E2_RM_B1_NAMESPACE,
            }
        })?;
        if receipt_and_other_sources(&current) != receipt_and_other_sources(&hub.manifest) {
            return Err(ReadOnlyAuthoritySnapshotErrorV1::NamespaceChanged {
                namespace: E2_RM_B1_NAMESPACE,
            });
        }
        let registry =
            self.registry
                .as_ref()
                .ok_or(ReadOnlyAuthoritySnapshotErrorV1::NamespaceChanged {
                    namespace: E2_RM_B1_NAMESPACE,
                })?;
        registry.verify(&hub.directory)?;
        for temporary in &self.temporary_files {
            temporary.verify(&hub.directory)?;
        }
        self.run
            .directory
            .revalidate_entry(&hub.entry)
            .map_err(|_| ReadOnlyAuthoritySnapshotErrorV1::NamespaceChanged {
                namespace: E2_RM_B1_NAMESPACE,
            })?;
        self.run.verify(bootstrap)
    }
}

#[cfg(target_os = "linux")]
pub(in crate::execution::agent_runtime::host_session_authority) struct ReadOnlyVersionedAuthorityTransactionV1<
    'root,
> {
    supplied_root: &'root TrustedAuthorityRoot,
    reopened_root: TrustedAuthorityRoot,
    bootstrap_metadata: TrustedEntryMetadataV1,
    bootstrap_manifest: Vec<DirectoryEntry>,
    authority: ReadOnlyDirectoryGuardV1,
    authority_children_metadata: BTreeMap<String, TrustedEntryMetadataV1>,
    root_file: ReadOnlyFileGuardV1,
    init_file: Option<ReadOnlyFileGuardV1>,
    lock_directory: ReadOnlyDirectoryGuardV1,
    temporary_directory: ReadOnlyDirectoryGuardV1,
    authority_temporary_files: Vec<ReadOnlyFileGuardV1>,
    root_lock: ReadOnlyFileGuardV1,
    decoded_root: VersionedStateRoot,
    e2_guard: RefCell<Option<ReadOnlyE2NamespaceGuardV1>>,
    b1_guard: RefCell<Option<ReadOnlyB1NamespaceGuardV1>>,
    typed_object_guards: RefCell<Vec<ReadOnlyTypedObjectGuardV1>>,
    _root_lock_file: TrustedFile,
    _lock: TrustedOwnedFileLock,
}

#[cfg(target_os = "linux")]
#[derive(Default)]
struct ReadOnlyTypedObjectGuardV1 {
    objects: Option<ReadOnlyDirectoryGuardV1>,
    kind: Option<ReadOnlyDirectoryGuardV1>,
    version: Option<ReadOnlyDirectoryGuardV1>,
    object: Option<ReadOnlyFileGuardV1>,
    keys: Option<ReadOnlyDirectoryGuardV1>,
    key: Option<ReadOnlyFileGuardV1>,
}

#[cfg(target_os = "linux")]
impl ReadOnlyTypedObjectGuardV1 {
    // Completion stages new HSA objects before its final exact-current commit.
    // Preserve each original path/leaf; the HSA publisher owns staged additions.
    // Generic snapshot verification below retains its exact namespace contract.
    fn verify_original_history(
        &self,
        authority: &TrustedDirectory,
    ) -> Result<(), ReadOnlyAuthoritySnapshotErrorV1> {
        let spine = |guard: &ReadOnlyDirectoryGuardV1, parent: &TrustedDirectory| {
            parent.revalidate_entry(&guard.entry).map_err(|_| {
                ReadOnlyAuthoritySnapshotErrorV1::SnapshotEntryChanged {
                    namespace: guard.namespace,
                    name: guard.entry.name.clone(),
                }
            })?;
            guard.directory.metadata().map_err(|_| {
                ReadOnlyAuthoritySnapshotErrorV1::NamespaceChanged {
                    namespace: guard.namespace,
                }
            })?;
            Ok::<(), ReadOnlyAuthoritySnapshotErrorV1>(())
        };
        if let Some(objects) = &self.objects {
            spine(objects, authority)?;
            if let Some(kind) = &self.kind {
                spine(kind, &objects.directory)?;
                if let Some(version) = &self.version {
                    spine(version, &kind.directory)?;
                    if let Some(object) = &self.object {
                        object.verify(&version.directory)?;
                    }
                    spine(version, &kind.directory)?;
                }
                spine(kind, &objects.directory)?;
            }
            spine(objects, authority)?;
        }
        if let Some(keys) = &self.keys {
            spine(keys, authority)?;
            if let Some(key) = &self.key {
                key.verify(&keys.directory)?;
            }
            spine(keys, authority)?;
        }
        Ok(())
    }

    fn verify(&self, authority: &TrustedDirectory) -> Result<(), ReadOnlyAuthoritySnapshotErrorV1> {
        if let Some(objects) = &self.objects {
            objects.verify(authority)?;
            if let Some(kind) = &self.kind {
                kind.verify(&objects.directory)?;
                if let Some(version) = &self.version {
                    version.verify(&kind.directory)?;
                    if let Some(object) = &self.object {
                        object.verify(&version.directory)?;
                    }
                    version.verify(&kind.directory)?;
                }
                kind.verify(&objects.directory)?;
            }
            objects.verify(authority)?;
        }
        if let Some(keys) = &self.keys {
            keys.verify(authority)?;
            if let Some(key) = &self.key {
                key.verify(&keys.directory)?;
            }
            keys.verify(authority)?;
        }
        Ok(())
    }
}

#[cfg(target_os = "linux")]
fn typed_object_read_error(operation: &'static str) -> ReadOnlyAuthoritySnapshotErrorV1 {
    ReadOnlyAuthoritySnapshotErrorV1::Io { operation }
}

#[cfg(target_os = "linux")]
fn capture_typed_object_directory(
    parent: &TrustedDirectory,
    manifest: &[DirectoryEntry],
    name: &str,
    namespace: &'static str,
) -> Result<ReadOnlyDirectoryGuardV1, ReadOnlyAuthoritySnapshotErrorV1> {
    let entry = manifest
        .iter()
        .find(|entry| entry.name == name)
        .ok_or_else(|| typed_object_read_error("typed object namespace is absent"))?;
    ReadOnlyDirectoryGuardV1::capture(parent, entry, namespace)
}

#[cfg(target_os = "linux")]
impl ReadOnlyVersionedAuthorityTransactionV1<'_> {
    fn validate_captured_exact_history(&self) -> Result<(), ReadOnlyAuthoritySnapshotErrorV1> {
        let VersionedStateRoot::V3(root) = &self.decoded_root else {
            return Ok(());
        };
        super::super::super::facade::validate_v3_schema_and_exact_history_with_reader(
            root,
            |reference| {
                self.read_typed_object(
                    &root.authority_store_id,
                    root.root_revision,
                    reference,
                    None,
                )
                .map_err(|error| format!("{error:?}"))
            },
        )
        .map_err(|_| ReadOnlyAuthoritySnapshotErrorV1::AuthorityRootEncodingInvalid)
    }

    /// Root captured and strictly decoded under this transaction's retained guards.
    pub(in crate::execution::agent_runtime::host_session_authority) fn captured_root(
        &self,
    ) -> &VersionedStateRoot {
        &self.decoded_root
    }

    /// Read an indexed object against this locked snapshot's frozen store and revision.
    ///
    /// Bytes remain inside HSA. The supplied reference's commitment authenticates
    /// the captured bytes; the index binds its ID, kind, schema and present state.
    /// Selecting that reference from the current authority graph and establishing
    /// the provenance of its parent context remain the caller's responsibility.
    pub(in crate::execution::agent_runtime::host_session_authority) fn read_typed_object(
        &self,
        expected_authority_store_id: &str,
        expected_root_revision: u64,
        reference: &AuthorityObjectRefV1,
        context: Option<&ObjectVerificationContextV1>,
    ) -> Result<Vec<u8>, ReadOnlyAuthoritySnapshotErrorV1> {
        if self.decoded_root.authority_store_id() != expected_authority_store_id {
            return Err(typed_object_read_error(
                "frozen HSA authority store mismatch",
            ));
        }
        if self.decoded_root.root_revision() != expected_root_revision {
            return Err(typed_object_read_error("frozen HSA root revision mismatch"));
        }
        match &self.decoded_root {
            VersionedStateRoot::V2(root) => self.capture_typed_object(root, reference, context),
            VersionedStateRoot::V3(root) => self.capture_typed_object(root, reference, context),
            VersionedStateRoot::V1(_) => Err(typed_object_read_error(
                "typed object snapshot requires StateRootV2 or StateRootV3",
            )),
        }
    }

    fn capture_typed_object<R: object_persistence::ObjectVerificationRootV1>(
        &self,
        root: &R,
        reference: &AuthorityObjectRefV1,
        context: Option<&ObjectVerificationContextV1>,
    ) -> Result<Vec<u8>, ReadOnlyAuthoritySnapshotErrorV1> {
        let mut guard = ReadOnlyTypedObjectGuardV1::default();
        let result = (|| {
            validate_ref_id(&reference.ref_id)
                .map_err(|_| typed_object_read_error("typed object ref ID is invalid"))?;
            let index = root
                .object_index()
                .get(&reference.ref_id)
                .filter(|index| {
                    index.schema_version == 1
                        && index.ref_id == reference.ref_id
                        && index.object_kind == reference.object_kind
                        && index.object_schema_version == reference.schema_version
                        && index.storage_state == AuthorityObjectStorageStateV1::Present
                })
                .ok_or_else(|| {
                    typed_object_read_error("typed object has no exact present index entry")
                })?;
            guard.objects = Some(capture_typed_object_directory(
                &self.authority.directory,
                &self.authority.manifest,
                "objects",
                "authority-v1/objects",
            )?);
            let objects = guard
                .objects
                .as_ref()
                .ok_or_else(|| typed_object_read_error("retain captured objects directory"))?;
            guard.kind = Some(capture_typed_object_directory(
                &objects.directory,
                &objects.manifest,
                kind_slug(reference.object_kind),
                "authority-v1/objects/kind",
            )?);
            let kind = guard
                .kind
                .as_ref()
                .ok_or_else(|| typed_object_read_error("retain captured object kind directory"))?;
            guard.version = Some(capture_typed_object_directory(
                &kind.directory,
                &kind.manifest,
                &format!("v{}", reference.schema_version),
                "authority-v1/objects/kind/version",
            )?);
            let version = guard.version.as_ref().ok_or_else(|| {
                typed_object_read_error("retain captured object version directory")
            })?;
            let object_name = format!("{}.obj", reference.ref_id);
            let entry = version
                .manifest
                .iter()
                .find(|entry| entry.name == object_name)
                .ok_or_else(|| typed_object_read_error("typed object file is absent"))?;
            guard.object = Some(ReadOnlyFileGuardV1::capture(
                &version.directory,
                entry,
                "authority-v1/objects/kind/version",
            )?);
            let object = guard
                .object
                .as_ref()
                .ok_or_else(|| typed_object_read_error("retain captured object file"))?;
            if object.bytes.len() as u64 != index.byte_length {
                return Err(typed_object_read_error("typed object byte length mismatch"));
            }
            if matches!(
                reference.commitment,
                AuthorityObjectCommitmentV1::CanonicalSha256 { .. }
            ) {
                let schema_version = object_persistence::canonical_schema_version(&object.bytes)
                    .map_err(|error| typed_object_read_error(error.0))?;
                if schema_version != reference.schema_version {
                    return Err(typed_object_read_error(
                        "canonical object schema version mismatch",
                    ));
                }
            }
            if let AuthorityObjectCommitmentV1::StoreHmacSha256 { key_id, .. } =
                &reference.commitment
            {
                let record = root
                    .commitment_key_registry()
                    .get(key_id)
                    .filter(|record| record.state != AuthorityStoreCommitmentKeyStateV1::Retired)
                    .ok_or_else(|| {
                        typed_object_read_error("sensitive object key is unavailable")
                    })?;
                guard.keys = Some(capture_typed_object_directory(
                    &self.authority.directory,
                    &self.authority.manifest,
                    "keys",
                    "authority-v1/keys",
                )?);
                let keys = guard
                    .keys
                    .as_ref()
                    .ok_or_else(|| typed_object_read_error("retain captured keys directory"))?;
                let key_name = format!("{}.key", record.key_id);
                let entry = keys
                    .manifest
                    .iter()
                    .find(|entry| entry.name == key_name)
                    .ok_or_else(|| {
                        typed_object_read_error("sensitive object key file is absent")
                    })?;
                guard.key = Some(ReadOnlyFileGuardV1::capture(
                    &keys.directory,
                    entry,
                    "authority-v1/keys",
                )?);
                let key = guard
                    .key
                    .as_ref()
                    .ok_or_else(|| typed_object_read_error("retain captured key file"))?;
                let envelope = AuthorityStoreCommitmentKeyFileV1::decode(&key.bytes)
                    .map_err(|_| typed_object_read_error("decode sensitive object key"))?;
                if envelope.authority_store_id != root.authority_store_id()
                    || envelope.authority_store_id != record.authority_store_id
                    || envelope.key_id != *key_id
                    || envelope.key_id != record.key_id
                    || envelope.created_at != record.created_at
                {
                    return Err(typed_object_read_error(
                        "sensitive object key identity mismatch",
                    ));
                }
            }
            object_persistence::verify_object_bytes_with_key_loader(
                root,
                reference,
                &object.bytes,
                context,
                true,
                |_| {
                    guard
                        .key
                        .as_ref()
                        .map(|key| key.bytes.clone())
                        .ok_or(BootstrapError("captured sensitive object key is absent"))
                },
            )
            .map_err(|error| typed_object_read_error(error.0))?;
            Ok(object.bytes.clone())
        })();
        // Retain partial traversals and captured bytes even when validation fails.
        self.typed_object_guards.borrow_mut().push(guard);
        result
    }

    pub(super) fn authority_directory(&self) -> &TrustedDirectory {
        &self.authority.directory
    }

    pub(super) fn authority_manifest(&self) -> &[DirectoryEntry] {
        &self.authority.manifest
    }

    pub(super) fn retain_e2_guard(
        &self,
        guard: ReadOnlyE2NamespaceGuardV1,
    ) -> Result<(), ReadOnlyAuthoritySnapshotErrorV1> {
        let mut retained = self.e2_guard.borrow_mut();
        if retained.is_some() {
            return Err(ReadOnlyAuthoritySnapshotErrorV1::Io {
                operation: "capture E2 namespace more than once",
            });
        }
        *retained = Some(guard);
        Ok(())
    }

    pub(super) fn authority_root_identity(&self) -> &CanonicalDirectoryV1 {
        self.decoded_root.bootstrap_home()
    }

    pub(super) fn authority_store_id(&self) -> &str {
        self.decoded_root.authority_store_id()
    }

    pub(super) fn root_bytes(&self) -> &[u8] {
        &self.root_file.bytes
    }

    pub(super) fn root_revision(&self) -> u64 {
        self.decoded_root.root_revision()
    }

    pub(super) fn read_world_work_receipt_registry_snapshot(
        &self,
    ) -> Result<WorldWorkReceiptRegistryPhysicalReadV1, ReadOnlyAuthoritySnapshotErrorV1> {
        if self.b1_guard.borrow().is_some() {
            return Err(ReadOnlyAuthoritySnapshotErrorV1::Io {
                operation: "capture B1 namespace more than once",
            });
        }
        let Some(run_entry) = self
            .bootstrap_manifest
            .iter()
            .find(|entry| entry.name == "run")
        else {
            return Ok(WorldWorkReceiptRegistryPhysicalReadV1::Absent);
        };
        let run =
            ReadOnlyDirectoryGuardV1::capture(self.reopened_root.directory(), run_entry, "run")?;
        let mut guard = ReadOnlyB1NamespaceGuardV1 {
            run,
            agent_hub: None,
            registry: None,
            temporary_files: Vec::new(),
        };
        let result = (|| {
            let Some(agent_hub_entry) = guard
                .run
                .manifest
                .iter()
                .find(|entry| entry.name == "agent-hub")
            else {
                return Ok(WorldWorkReceiptRegistryPhysicalReadV1::Absent);
            };
            guard.agent_hub = Some(ReadOnlyDirectoryGuardV1::capture(
                &guard.run.directory,
                agent_hub_entry,
                "run/agent-hub",
            )?);
            let agent_hub =
                guard
                    .agent_hub
                    .as_ref()
                    .ok_or(ReadOnlyAuthoritySnapshotErrorV1::Io {
                        operation: "retain B1 agent-hub guard",
                    })?;
            let receipt_entries = agent_hub
                .manifest
                .iter()
                .filter(|entry| entry.name.starts_with(WORLD_WORK_RECEIPT_REGISTRY_PREFIX))
                .collect::<Vec<_>>();
            if receipt_entries.is_empty() {
                return Ok(WorldWorkReceiptRegistryPhysicalReadV1::Absent);
            }

            let mut registry_entry = None;
            let mut failure = None;
            for entry in receipt_entries {
                if entry.name == WORLD_WORK_RECEIPT_REGISTRY_FILE {
                    if registry_entry.replace(entry).is_some() {
                        failure.get_or_insert(
                            ReadOnlyAuthoritySnapshotErrorV1::UnsafeNamespaceEntry {
                                namespace: E2_RM_B1_NAMESPACE,
                                name: entry.name.clone(),
                            },
                        );
                    }
                } else if entry.kind == EntryKind::RegularFile
                    && is_world_work_receipt_registry_temp_name(&entry.name)
                {
                    match ReadOnlyFileGuardV1::capture(
                        &agent_hub.directory,
                        entry,
                        E2_RM_B1_NAMESPACE,
                    ) {
                        Ok(temporary) => {
                            failure.get_or_insert(
                                ReadOnlyAuthoritySnapshotErrorV1::UnsafeTemporaryMaterial {
                                    namespace: E2_RM_B1_NAMESPACE,
                                    name: temporary.entry.name.clone(),
                                },
                            );
                            guard.temporary_files.push(temporary);
                        }
                        Err(error) => {
                            failure.get_or_insert(error);
                        }
                    }
                } else {
                    failure.get_or_insert(ReadOnlyAuthoritySnapshotErrorV1::UnsafeNamespaceEntry {
                        namespace: E2_RM_B1_NAMESPACE,
                        name: entry.name.clone(),
                    });
                }
            }
            if let Some(error) = failure {
                return Err(error);
            }
            let registry_entry =
                registry_entry.ok_or(ReadOnlyAuthoritySnapshotErrorV1::PartialNamespace {
                    namespace: E2_RM_B1_NAMESPACE,
                    component: WORLD_WORK_RECEIPT_REGISTRY_FILE,
                })?;
            guard.registry = Some(ReadOnlyFileGuardV1::capture(
                &agent_hub.directory,
                registry_entry,
                E2_RM_B1_NAMESPACE,
            )?);
            let registry = guard
                .registry
                .as_ref()
                .ok_or(ReadOnlyAuthoritySnapshotErrorV1::Io {
                    operation: "retain B1 registry guard",
                })?;
            Ok(WorldWorkReceiptRegistryPhysicalReadV1::Present(
                WorldWorkReceiptRegistryPhysicalSnapshotV1 {
                    registry_bytes: registry.bytes.clone(),
                },
            ))
        })();
        *self.b1_guard.borrow_mut() = Some(guard);
        result
    }

    fn verify_scope(&self) -> Result<(), ReadOnlyAuthoritySnapshotErrorV1> {
        self.supplied_root
            .revalidate()
            .map_err(|_| ReadOnlyAuthoritySnapshotErrorV1::AuthorityRootReplaced)?;
        self.reopened_root
            .revalidate()
            .map_err(|_| ReadOnlyAuthoritySnapshotErrorV1::AuthorityRootReplaced)?;
        if self.supplied_root.identity() != self.reopened_root.identity()
            || self
                .reopened_root
                .directory()
                .metadata()
                .map_err(|_| ReadOnlyAuthoritySnapshotErrorV1::AuthorityRootReplaced)?
                != self.bootstrap_metadata
            || self
                .reopened_root
                .directory()
                .entries()
                .map_err(|_| ReadOnlyAuthoritySnapshotErrorV1::AuthorityRootReplaced)?
                != self.bootstrap_manifest
        {
            return Err(ReadOnlyAuthoritySnapshotErrorV1::AuthorityRootReplaced);
        }
        self.authority.verify(self.reopened_root.directory())?;
        for entry in &self.authority.manifest {
            let current = self
                .authority
                .directory
                .entry_metadata(entry)
                .map_err(|_| ReadOnlyAuthoritySnapshotErrorV1::SnapshotEntryChanged {
                    namespace: E2_RM_AUTHORITY_NAMESPACE,
                    name: entry.name.clone(),
                })?;
            if self.authority_children_metadata.get(&entry.name) != Some(&current) {
                return Err(ReadOnlyAuthoritySnapshotErrorV1::SnapshotEntryChanged {
                    namespace: E2_RM_AUTHORITY_NAMESPACE,
                    name: entry.name.clone(),
                });
            }
        }
        self.lock_directory.verify(&self.authority.directory)?;
        self.temporary_directory.verify(&self.authority.directory)?;
        for temporary in &self.authority_temporary_files {
            temporary.verify(&self.temporary_directory.directory)?;
        }
        self.root_lock.verify(&self.lock_directory.directory)?;
        if let Some(init_file) = &self.init_file {
            init_file.verify(&self.authority.directory)?;
        }
        self.root_file.verify(&self.authority.directory)?;
        let root = decode_read_only_versioned_root(&self.root_file.bytes)?;
        if root.root_revision() != self.decoded_root.root_revision() || root != self.decoded_root {
            return Err(ReadOnlyAuthoritySnapshotErrorV1::AuthorityRootChangedWhileLocked);
        }
        if let Some(guard) = self.e2_guard.borrow().as_ref() {
            guard.verify(&self.authority.directory)?;
        }
        if let Some(guard) = self.b1_guard.borrow().as_ref() {
            guard.verify(self.reopened_root.directory())?;
        }
        for guard in self.typed_object_guards.borrow().iter() {
            guard.verify(&self.authority.directory)?;
        }
        self.root_file.verify(&self.authority.directory)?;
        self.authority.verify(self.reopened_root.directory())?;
        self.supplied_root
            .revalidate()
            .map_err(|_| ReadOnlyAuthoritySnapshotErrorV1::AuthorityRootReplaced)?;
        self.reopened_root
            .revalidate()
            .map_err(|_| ReadOnlyAuthoritySnapshotErrorV1::AuthorityRootReplaced)
    }
}

/// Owned physical E2/B1 guards from one existing-root read. This has no HSA
/// mutation, execution, population classification, or settlement authority.
/// The root lock is released before return; a committing owner must exact-join
/// the captured HSA root and verify these same guards inside its commit guard.
#[cfg(target_os = "linux")]
pub(crate) struct HeldAcceptedWorkAuthorityPhysicalSnapshotV1 {
    snapshot: AcceptedWorkAuthorityPhysicalSnapshotV1,
    control_spine: HeldAuthorityControlSpineV1,
    typed_object_guards: Vec<ReadOnlyTypedObjectGuardV1>,
    root: TrustedAuthorityRoot,
    authority: ReadOnlyDirectoryGuardV1,
    e2: Option<ReadOnlyE2NamespaceGuardV1>,
    b1: Option<ReadOnlyB1NamespaceGuardV1>,
}

// Ordinary control changes only the supervisor namespace. Unlike an HSA
// completion publisher, that owner must preserve the original Root preimage.
#[cfg(target_os = "linux")]
struct HeldAuthorityControlSpineV1 {
    bootstrap_metadata: TrustedEntryMetadataV1,
    bootstrap_manifest: Vec<DirectoryEntry>,
    authority_children_metadata: BTreeMap<String, TrustedEntryMetadataV1>,
    root_file: ReadOnlyFileGuardV1,
    init_file: Option<ReadOnlyFileGuardV1>,
    lock_directory: ReadOnlyDirectoryGuardV1,
    temporary_directory: ReadOnlyDirectoryGuardV1,
    authority_temporary_files: Vec<ReadOnlyFileGuardV1>,
    root_lock: ReadOnlyFileGuardV1,
}

#[cfg(target_os = "linux")]
impl HeldAcceptedWorkAuthorityPhysicalSnapshotV1 {
    // The admission publisher owns only its existing admission namespace. Every
    // other original source, including B1 and supervisor data, remains strict.
    #[cfg(target_os = "linux")]
    fn verify_for_retained_admission_owner(&self) -> Result<(), ReadOnlyAuthoritySnapshotErrorV1> {
        let spine = &self.control_spine;
        self.root
            .revalidate()
            .map_err(|_| ReadOnlyAuthoritySnapshotErrorV1::AuthorityRootReplaced)?;
        if self
            .root
            .directory()
            .metadata()
            .map_err(|_| ReadOnlyAuthoritySnapshotErrorV1::AuthorityRootReplaced)?
            != spine.bootstrap_metadata
            || self
                .root
                .directory()
                .entries()
                .map_err(|_| ReadOnlyAuthoritySnapshotErrorV1::AuthorityRootReplaced)?
                != spine.bootstrap_manifest
        {
            return Err(ReadOnlyAuthoritySnapshotErrorV1::AuthorityRootReplaced);
        }
        self.authority.verify(self.root.directory())?;
        for entry in &self.authority.manifest {
            let current = self
                .authority
                .directory
                .entry_metadata(entry)
                .map_err(|_| ReadOnlyAuthoritySnapshotErrorV1::SnapshotEntryChanged {
                    namespace: E2_RM_AUTHORITY_NAMESPACE,
                    name: entry.name.clone(),
                })?;
            let expected = spine
                .authority_children_metadata
                .get(&entry.name)
                .ok_or_else(|| ReadOnlyAuthoritySnapshotErrorV1::SnapshotEntryChanged {
                    namespace: E2_RM_AUTHORITY_NAMESPACE,
                    name: entry.name.clone(),
                })?;
            let matches = if entry.name == RETAINED_ADMISSION_DIRECTORY {
                current.same_identity_and_security(*expected)
            } else {
                &current == expected
            };
            if !matches {
                return Err(ReadOnlyAuthoritySnapshotErrorV1::SnapshotEntryChanged {
                    namespace: E2_RM_AUTHORITY_NAMESPACE,
                    name: entry.name.clone(),
                });
            }
        }
        spine.lock_directory.verify(&self.authority.directory)?;
        spine
            .temporary_directory
            .verify(&self.authority.directory)?;
        for file in &spine.authority_temporary_files {
            file.verify(&spine.temporary_directory.directory)?;
        }
        spine.root_lock.verify(&spine.lock_directory.directory)?;
        if let Some(file) = &spine.init_file {
            file.verify(&self.authority.directory)?;
        }
        spine.root_file.verify(&self.authority.directory)?;
        match &self.e2 {
            Some(guard) => guard.verify(&self.authority.directory)?,
            None if self
                .authority
                .directory
                .entry_kind(super::dispatch_policy_commitment::held_snapshot_namespace_name())
                .map_err(|_| ReadOnlyAuthoritySnapshotErrorV1::Io {
                    operation: "verify admission control E2 absence",
                })?
                .is_some() =>
            {
                return Err(ReadOnlyAuthoritySnapshotErrorV1::NamespaceChanged {
                    namespace: "authority-v1/dispatch-policy-commitment-v1",
                });
            }
            None => {}
        }
        self.b1
            .as_ref()
            .ok_or(ReadOnlyAuthoritySnapshotErrorV1::Io {
                operation: "admission control B1 capture absent",
            })?
            .verify(self.root.directory())?;
        for guard in &self.typed_object_guards {
            guard.verify(&self.authority.directory)?;
        }
        spine.root_file.verify(&self.authority.directory)?;
        self.authority.verify(self.root.directory())?;
        self.root
            .revalidate()
            .map_err(|_| ReadOnlyAuthoritySnapshotErrorV1::AuthorityRootReplaced)
    }

    pub(crate) fn snapshot(&self) -> &AcceptedWorkAuthorityPhysicalSnapshotV1 {
        &self.snapshot
    }

    /// Verify the retained source descriptors and exact present/absent namespace
    /// entries. HSA's own typed-object/root publication is outside this scope;
    /// its separate exact-current transaction remains responsible for that join.
    pub(crate) fn verify_sources(&self) -> Result<(), ReadOnlyAuthoritySnapshotErrorV1> {
        self.root
            .revalidate()
            .map_err(|_| ReadOnlyAuthoritySnapshotErrorV1::AuthorityRootReplaced)?;
        self.root
            .directory()
            .revalidate_entry(&self.authority.entry)
            .map_err(|_| ReadOnlyAuthoritySnapshotErrorV1::AuthorityRootReplaced)?;
        match &self.e2 {
            Some(guard) => guard.verify(&self.authority.directory)?,
            None => {
                if self
                    .authority
                    .directory
                    .entry_kind(super::dispatch_policy_commitment::held_snapshot_namespace_name())
                    .map_err(|_| ReadOnlyAuthoritySnapshotErrorV1::Io {
                        operation: "verify held E2 source absence",
                    })?
                    .is_some()
                {
                    return Err(ReadOnlyAuthoritySnapshotErrorV1::NamespaceChanged {
                        namespace: "authority-v1/dispatch-policy-commitment-v1",
                    });
                }
            }
        }
        match &self.b1 {
            Some(guard) => guard.verify(self.root.directory())?,
            None => {
                if self
                    .root
                    .directory()
                    .entry_kind("run")
                    .map_err(|_| ReadOnlyAuthoritySnapshotErrorV1::Io {
                        operation: "verify held B1 source absence",
                    })?
                    .is_some()
                {
                    return Err(ReadOnlyAuthoritySnapshotErrorV1::NamespaceChanged {
                        namespace: "run",
                    });
                }
            }
        }
        for guard in &self.typed_object_guards {
            guard.verify_original_history(&self.authority.directory)?;
        }
        self.root
            .revalidate()
            .map_err(|_| ReadOnlyAuthoritySnapshotErrorV1::AuthorityRootReplaced)
    }

    /// Source verification for the supervisor's own publication. This does not
    /// permit HSA, E2, B1 receipt, key or admission changes. The supervisor
    /// publisher separately owns and validates its exact mutable namespace.
    pub(in crate::execution::agent_runtime::host_session_authority) fn verify_for_supervisor_owner(
        &self,
    ) -> Result<(), ReadOnlyAuthoritySnapshotErrorV1> {
        let spine = &self.control_spine;
        self.root
            .revalidate()
            .map_err(|_| ReadOnlyAuthoritySnapshotErrorV1::AuthorityRootReplaced)?;
        if self
            .root
            .directory()
            .metadata()
            .map_err(|_| ReadOnlyAuthoritySnapshotErrorV1::AuthorityRootReplaced)?
            != spine.bootstrap_metadata
            || self
                .root
                .directory()
                .entries()
                .map_err(|_| ReadOnlyAuthoritySnapshotErrorV1::AuthorityRootReplaced)?
                != spine.bootstrap_manifest
        {
            return Err(ReadOnlyAuthoritySnapshotErrorV1::AuthorityRootReplaced);
        }
        self.authority.verify(self.root.directory())?;
        for entry in &self.authority.manifest {
            let current = self
                .authority
                .directory
                .entry_metadata(entry)
                .map_err(|_| ReadOnlyAuthoritySnapshotErrorV1::SnapshotEntryChanged {
                    namespace: E2_RM_AUTHORITY_NAMESPACE,
                    name: entry.name.clone(),
                })?;
            if spine.authority_children_metadata.get(&entry.name) != Some(&current) {
                return Err(ReadOnlyAuthoritySnapshotErrorV1::SnapshotEntryChanged {
                    namespace: E2_RM_AUTHORITY_NAMESPACE,
                    name: entry.name.clone(),
                });
            }
        }
        spine.lock_directory.verify(&self.authority.directory)?;
        spine
            .temporary_directory
            .verify(&self.authority.directory)?;
        for file in &spine.authority_temporary_files {
            file.verify(&spine.temporary_directory.directory)?;
        }
        spine.root_lock.verify(&spine.lock_directory.directory)?;
        if let Some(file) = &spine.init_file {
            file.verify(&self.authority.directory)?;
        }
        spine.root_file.verify(&self.authority.directory)?;
        match &self.e2 {
            Some(guard) => guard.verify(&self.authority.directory)?,
            None if self
                .authority
                .directory
                .entry_kind(super::dispatch_policy_commitment::held_snapshot_namespace_name())
                .map_err(|_| ReadOnlyAuthoritySnapshotErrorV1::Io {
                    operation: "verify control E2 absence",
                })?
                .is_some() =>
            {
                return Err(ReadOnlyAuthoritySnapshotErrorV1::NamespaceChanged {
                    namespace: "authority-v1/dispatch-policy-commitment-v1",
                });
            }
            None => {}
        }
        match &self.b1 {
            Some(guard) => guard.verify_for_supervisor_owner(self.root.directory())?,
            None => {
                return Err(ReadOnlyAuthoritySnapshotErrorV1::NamespaceChanged {
                    namespace: "run/agent-hub/B1 control source absent",
                })
            }
        }
        for guard in &self.typed_object_guards {
            guard.verify(&self.authority.directory)?;
        }
        spine.root_file.verify(&self.authority.directory)?;
        self.authority.verify(self.root.directory())?;
        self.root
            .revalidate()
            .map_err(|_| ReadOnlyAuthoritySnapshotErrorV1::AuthorityRootReplaced)
    }
}

/// The exact supervisor and session-ledger files used by a World controller
/// completion. This borrows the original E2/B1 capture and retains every opened
/// directory and file identity. It neither reconciles nor publishes a ledger.
#[cfg(target_os = "linux")]
pub(crate) struct HeldWorldControllerCompletionPhysicalSourcesV1<'source> {
    source: &'source HeldAcceptedWorkAuthorityPhysicalSnapshotV1,
    admission: Option<ReadOnlyWorldControllerAdmissionSourcesV1>,
    supervisor: Option<ReadOnlyFileGuardV1>,
    legacy_sessions: Option<ReadOnlyDirectoryGuardV1>,
    ledger_parent: Option<ReadOnlyDirectoryGuardV1>,
    ledger: Option<ReadOnlyWorldCompletionTreeV1>,
    session_id: String,
    ledger_files: BTreeMap<String, Vec<u8>>,
}

/// Admission's existing owner namespace, captured without its creating or
/// reconciling acquisition API. Partial and uncommitted material refuses.
#[cfg(target_os = "linux")]
struct ReadOnlyWorldControllerAdmissionSourcesV1 {
    directory: ReadOnlyDirectoryGuardV1,
    registry: ReadOnlyFileGuardV1,
    keys: ReadOnlyDirectoryGuardV1,
    key_files: Vec<ReadOnlyFileGuardV1>,
    temporary: ReadOnlyDirectoryGuardV1,
}

/// Retained physical facts for the supervisor's mutation boundary. Possessing
/// these read sources grants no cancellation, delivery or publication permit.
#[cfg(target_os = "linux")]
pub(crate) struct HeldSupervisorAuthorityControlSourcesV1 {
    source: HeldAcceptedWorkAuthorityPhysicalSnapshotV1,
    admission: Option<ReadOnlyWorldControllerAdmissionSourcesV1>,
}

/// Original facts across the admission owner's own publication. Possession
/// grants no Stop, cancellation, mutation, delivery or native-send permission.
#[cfg(target_os = "linux")]
pub(crate) struct HeldRetainedAdmissionAuthorityControlSourcesV1 {
    source: HeldAcceptedWorkAuthorityPhysicalSnapshotV1,
    admission: ReadOnlyWorldControllerAdmissionSourcesV1,
    supervisor: Option<ReadOnlyFileGuardV1>,
}

#[cfg(target_os = "linux")]
impl HeldRetainedAdmissionAuthorityControlSourcesV1 {
    pub(crate) fn snapshot(&self) -> &AcceptedWorkAuthorityPhysicalSnapshotV1 {
        self.source.snapshot()
    }
    pub(crate) fn admission_registry_bytes(&self) -> &[u8] {
        &self.admission.registry.bytes
    }
    pub(crate) fn admission_key_files(&self) -> Vec<(String, Vec<u8>)> {
        self.admission
            .key_files
            .iter()
            .map(|file| (file.entry.name.clone(), file.bytes.clone()))
            .collect()
    }
    pub(crate) fn supervisor_bytes(&self) -> Option<&[u8]> {
        self.supervisor.as_ref().map(|file| file.bytes.as_slice())
    }

    fn verify_original_sources(
        &self,
        owned_temp: Option<&str>,
    ) -> Result<(), ReadOnlyAuthoritySnapshotErrorV1> {
        self.source.verify_for_retained_admission_owner()?;
        self.admission
            .verify_admission_owner_sources(&self.source.authority.directory, owned_temp)?;
        let hub = self
            .source
            .b1
            .as_ref()
            .and_then(|b1| b1.agent_hub.as_ref())
            .ok_or(ReadOnlyAuthoritySnapshotErrorV1::Io {
                operation: "admission control original B1 hub absent",
            })?;
        match &self.supervisor {
            Some(file) => file.verify(&hub.directory)?,
            None if hub
                .directory
                .entry_kind(WORLD_WORK_EXECUTION_SUPERVISOR_FILE)
                .map_err(|_| ReadOnlyAuthoritySnapshotErrorV1::Io {
                    operation: "verify original supervisor absence",
                })?
                .is_some() =>
            {
                return Err(ReadOnlyAuthoritySnapshotErrorV1::NamespaceChanged {
                    namespace: "run/agent-hub/supervisor",
                });
            }
            None => {}
        }
        self.source.verify_for_retained_admission_owner()
    }

    pub(crate) fn verify_before_publication(
        &self,
        owned_temp: Option<&str>,
    ) -> Result<(), ReadOnlyAuthoritySnapshotErrorV1> {
        self.verify_original_sources(owned_temp)?;
        self.admission
            .registry
            .verify(&self.admission.directory.directory)?;
        self.verify_original_sources(owned_temp)
    }

    // The original registry was consumed before the owner's known replacement.
    // This verifies its outcome, never substitutes a reread for source custody.
    pub(crate) fn verify_after_publication(
        &self,
        expected_registry: &[u8],
    ) -> Result<(), ReadOnlyAuthoritySnapshotErrorV1> {
        self.verify_original_sources(None)?;
        let actual = self
            .admission
            .directory
            .directory
            .open_file(RETAINED_ADMISSION_REGISTRY_FILE)
            .and_then(|file| file.read_all())
            .map_err(|_| ReadOnlyAuthoritySnapshotErrorV1::Io {
                operation: "verify admission owner's published outcome",
            })?;
        if actual != expected_registry {
            return Err(ReadOnlyAuthoritySnapshotErrorV1::NamespaceChanged {
                namespace: "authority-v1/retained-worker-admission-v1",
            });
        }
        self.verify_original_sources(None)
    }
}

#[cfg(target_os = "linux")]
pub(in crate::execution::agent_runtime::host_session_authority) fn hold_existing_retained_admission_control_sources_opened<
    R,
>(
    opened: &TrustedAuthorityRoot,
    operation: impl FnOnce(&ReadOnlyVersionedAuthorityTransactionV1<'_>) -> R,
) -> Result<(HeldRetainedAdmissionAuthorityControlSourcesV1, R), ReadOnlyAuthoritySnapshotErrorV1> {
    let (source, result) =
        hold_existing_accepted_work_authority_snapshot_opened_with_source(opened, |transaction| {
            let entry = transaction
                .authority
                .manifest
                .iter()
                .find(|entry| entry.name == RETAINED_ADMISSION_DIRECTORY)
                .ok_or(ReadOnlyAuthoritySnapshotErrorV1::Io {
                    operation: "ordinary retained control requires existing admission",
                })?;
            let admission = ReadOnlyWorldControllerAdmissionSourcesV1::capture_admission_sources(
                &transaction.authority.directory,
                entry,
            )?;
            let supervisor_result = (|| {
                let b1 = transaction.b1_guard.borrow();
                let hub = b1
                    .as_ref()
                    .and_then(|guard| guard.agent_hub.as_ref())
                    .ok_or(ReadOnlyAuthoritySnapshotErrorV1::Io {
                        operation: "ordinary retained control requires existing B1 hub",
                    })?;
                hub.manifest
                    .iter()
                    .find(|entry| entry.name == WORLD_WORK_EXECUTION_SUPERVISOR_FILE)
                    .map(|entry| {
                        ReadOnlyFileGuardV1::capture(
                            &hub.directory,
                            entry,
                            "run/agent-hub/supervisor",
                        )
                    })
                    .transpose()
            })();
            // Even refusal while capturing a later source finalizes the original
            // admission traversal; the enclosing owner also finalizes HSA/E2/B1.
            admission.verify_admission_sources(&transaction.authority.directory)?;
            let supervisor = supervisor_result?;
            let output = operation(transaction);
            admission.verify_admission_sources(&transaction.authority.directory)?;
            if let Some(file) = &supervisor {
                let b1 = transaction.b1_guard.borrow();
                let hub = b1
                    .as_ref()
                    .and_then(|guard| guard.agent_hub.as_ref())
                    .ok_or(ReadOnlyAuthoritySnapshotErrorV1::Io {
                        operation: "finalize original supervisor source",
                    })?;
                file.verify(&hub.directory)?;
            }
            Ok::<_, ReadOnlyAuthoritySnapshotErrorV1>((admission, supervisor, output))
        })?;
    let (admission, supervisor, output) = result?;
    let held = HeldRetainedAdmissionAuthorityControlSourcesV1 {
        source,
        admission,
        supervisor,
    };
    held.verify_before_publication(None)?;
    Ok((held, output))
}

#[cfg(target_os = "linux")]
impl HeldSupervisorAuthorityControlSourcesV1 {
    pub(crate) fn snapshot(&self) -> &AcceptedWorkAuthorityPhysicalSnapshotV1 {
        self.source.snapshot()
    }

    pub(crate) fn admission_registry_bytes(&self) -> Option<&[u8]> {
        self.admission
            .as_ref()
            .map(|held| held.registry.bytes.as_slice())
    }

    pub(crate) fn admission_key_files(&self) -> Vec<(String, Vec<u8>)> {
        self.admission
            .as_ref()
            .map(|held| {
                held.key_files
                    .iter()
                    .map(|file| (file.entry.name.clone(), file.bytes.clone()))
                    .collect()
            })
            .unwrap_or_default()
    }

    pub(crate) fn verify_sources(&self) -> Result<(), ReadOnlyAuthoritySnapshotErrorV1> {
        self.source.verify_for_supervisor_owner()?;
        match &self.admission {
            Some(held) => held.verify_admission_sources(&self.source.authority.directory)?,
            None if self
                .source
                .authority
                .directory
                .entry_kind(RETAINED_ADMISSION_DIRECTORY)
                .map_err(|_| ReadOnlyAuthoritySnapshotErrorV1::Io {
                    operation: "verify control admission absence",
                })?
                .is_some() =>
            {
                return Err(ReadOnlyAuthoritySnapshotErrorV1::NamespaceChanged {
                    namespace: "authority-v1/retained-worker-admission-v1",
                });
            }
            None => {}
        }
        self.source.verify_for_supervisor_owner()
    }
}

#[cfg(target_os = "linux")]
pub(in crate::execution::agent_runtime::host_session_authority) fn hold_existing_supervisor_control_sources_opened<
    R,
>(
    opened: &TrustedAuthorityRoot,
    operation: impl FnOnce(&ReadOnlyVersionedAuthorityTransactionV1<'_>) -> R,
) -> Result<(HeldSupervisorAuthorityControlSourcesV1, R), ReadOnlyAuthoritySnapshotErrorV1> {
    let (source, result) =
        hold_existing_accepted_work_authority_snapshot_opened_with_source(opened, |transaction| {
            let admission = transaction
                .authority
                .manifest
                .iter()
                .find(|entry| entry.name == RETAINED_ADMISSION_DIRECTORY)
                .map(|entry| {
                    ReadOnlyWorldControllerAdmissionSourcesV1::capture_admission_sources(
                        &transaction.authority.directory,
                        entry,
                    )
                })
                .transpose()?;
            let output = operation(transaction);
            if let Some(held) = &admission {
                held.verify_admission_sources(&transaction.authority.directory)?;
            }
            Ok::<_, ReadOnlyAuthoritySnapshotErrorV1>((admission, output))
        })?;
    let (admission, output) = result?;
    let held = HeldSupervisorAuthorityControlSourcesV1 { source, admission };
    held.verify_sources()?;
    Ok((held, output))
}

#[cfg(target_os = "linux")]
impl ReadOnlyWorldControllerAdmissionSourcesV1 {
    // The owner's temporary is the sole allowed staged entry. Original registry
    // bytes are checked separately before publication; original keys stay strict.
    fn verify_admission_owner_sources(
        &self,
        parent: &TrustedDirectory,
        owned_temp: Option<&str>,
    ) -> Result<(), ReadOnlyAuthoritySnapshotErrorV1> {
        let fail = || ReadOnlyAuthoritySnapshotErrorV1::NamespaceChanged {
            namespace: "authority-v1/retained-worker-admission-v1",
        };
        parent
            .revalidate_entry(&self.directory.entry)
            .map_err(|_| fail())?;
        if !self
            .directory
            .directory
            .metadata()
            .map_err(|_| fail())?
            .same_identity_and_security(self.directory.metadata)
            || !parent
                .entry_metadata(&self.directory.entry)
                .map_err(|_| fail())?
                .same_identity_and_security(self.directory.metadata)
        {
            return Err(fail());
        }
        let current = self.directory.directory.entries().map_err(|_| fail())?;
        if current.len() != 3
            || current.iter().any(|entry| {
                !matches!(
                    (entry.name.as_str(), entry.kind),
                    (RETAINED_ADMISSION_REGISTRY_FILE, EntryKind::RegularFile)
                        | (
                            RETAINED_ADMISSION_KEYS_DIRECTORY | RETAINED_ADMISSION_TEMP_DIRECTORY,
                            EntryKind::Directory
                        )
                )
            })
        {
            return Err(fail());
        }
        self.keys.verify(&self.directory.directory)?;
        for key in &self.key_files {
            key.verify(&self.keys.directory)?;
        }
        self.directory
            .directory
            .revalidate_entry(&self.temporary.entry)
            .map_err(|_| fail())?;
        if !self
            .temporary
            .directory
            .metadata()
            .map_err(|_| fail())?
            .same_identity_and_security(self.temporary.metadata)
            || !self
                .directory
                .directory
                .entry_metadata(&self.temporary.entry)
                .map_err(|_| fail())?
                .same_identity_and_security(self.temporary.metadata)
        {
            return Err(fail());
        }
        if owned_temp.is_some_and(|name| !super::admission_temp_file_name(name)) {
            return Err(fail());
        }
        let entries = self.temporary.directory.entries().map_err(|_| fail())?;
        if entries.len() > 1
            || entries.iter().any(|entry| {
                entry.kind != EntryKind::RegularFile || Some(entry.name.as_str()) != owned_temp
            })
        {
            return Err(fail());
        }
        Ok(())
    }

    fn capture_admission_sources(
        parent: &TrustedDirectory,
        entry: &DirectoryEntry,
    ) -> Result<Self, ReadOnlyAuthoritySnapshotErrorV1> {
        const NS: &str = "authority-v1/retained-worker-admission-v1";
        let directory = ReadOnlyDirectoryGuardV1::capture(parent, entry, NS)?;
        if directory.manifest.iter().any(|entry| {
            !matches!(
                (entry.name.as_str(), entry.kind),
                (RETAINED_ADMISSION_REGISTRY_FILE, EntryKind::RegularFile)
                    | (
                        RETAINED_ADMISSION_KEYS_DIRECTORY | RETAINED_ADMISSION_TEMP_DIRECTORY,
                        EntryKind::Directory
                    )
            )
        }) {
            return Err(ReadOnlyAuthoritySnapshotErrorV1::UnsafeNamespaceEntry {
                namespace: NS,
                name: "uncovered admission material".into(),
            });
        }
        let required = |name: &'static str| {
            directory
                .manifest
                .iter()
                .find(|entry| entry.name == name)
                .ok_or(ReadOnlyAuthoritySnapshotErrorV1::PartialNamespace {
                    namespace: NS,
                    component: name,
                })
        };
        let registry = ReadOnlyFileGuardV1::capture(
            &directory.directory,
            required(RETAINED_ADMISSION_REGISTRY_FILE)?,
            NS,
        )?;
        let keys = ReadOnlyDirectoryGuardV1::capture(
            &directory.directory,
            required(RETAINED_ADMISSION_KEYS_DIRECTORY)?,
            NS,
        )?;
        let temporary = ReadOnlyDirectoryGuardV1::capture(
            &directory.directory,
            required(RETAINED_ADMISSION_TEMP_DIRECTORY)?,
            NS,
        )?;
        if !temporary.manifest.is_empty()
            || keys.manifest.iter().any(|entry| {
                entry.kind != EntryKind::RegularFile || !admission_key_file_name(&entry.name)
            })
        {
            return Err(ReadOnlyAuthoritySnapshotErrorV1::UnsafeNamespaceEntry {
                namespace: NS,
                name: "uncommitted admission material".into(),
            });
        }
        let key_files = keys
            .manifest
            .iter()
            .map(|entry| ReadOnlyFileGuardV1::capture(&keys.directory, entry, NS))
            .collect::<Result<_, _>>()?;
        let held = Self {
            directory,
            registry,
            keys,
            key_files,
            temporary,
        };
        held.verify_admission_sources(parent)?;
        Ok(held)
    }

    fn verify_admission_sources(
        &self,
        parent: &TrustedDirectory,
    ) -> Result<(), ReadOnlyAuthoritySnapshotErrorV1> {
        self.directory.verify(parent)?;
        self.registry.verify(&self.directory.directory)?;
        self.keys.verify(&self.directory.directory)?;
        for file in &self.key_files {
            file.verify(&self.keys.directory)?;
        }
        self.temporary.verify(&self.directory.directory)?;
        self.directory.verify(parent)
    }
}

#[cfg(target_os = "linux")]
struct ReadOnlyWorldCompletionTreeV1 {
    guard: ReadOnlyDirectoryGuardV1,
    directories: Vec<ReadOnlyWorldCompletionTreeV1>,
    files: Vec<ReadOnlyFileGuardV1>,
}

#[cfg(target_os = "linux")]
impl ReadOnlyWorldCompletionTreeV1 {
    fn capture_world_completion_tree(
        parent: &TrustedDirectory,
        entry: &DirectoryEntry,
        relative: &str,
        depth: usize,
        files: &mut BTreeMap<String, Vec<u8>>,
    ) -> Result<Self, ReadOnlyAuthoritySnapshotErrorV1> {
        const NAMESPACE: &str = "obligation-ledger/World-controller-session";
        if depth > 5 {
            return Err(ReadOnlyAuthoritySnapshotErrorV1::UnsafeNamespaceEntry {
                namespace: NAMESPACE,
                name: entry.name.clone(),
            });
        }
        let guard = ReadOnlyDirectoryGuardV1::capture(parent, entry, NAMESPACE)?;
        let mut directories = Vec::new();
        let mut retained_files = Vec::new();
        for child in &guard.manifest {
            let path = if relative.is_empty() {
                child.name.clone()
            } else {
                format!("{relative}/{}", child.name)
            };
            let components: Vec<_> = path.split('/').collect();
            let covered = match child.kind {
                EntryKind::Directory => matches!(
                    components.as_slice(),
                    ["acceptances"]
                        | ["acceptances", _]
                        | ["acceptances", _, "events" | "obligations"]
                ),
                EntryKind::RegularFile => matches!(
                    components.as_slice(),
                    ["revision-cursor.json"]
                        | ["acceptances", _, "state.json"]
                        | ["acceptances", _, "events" | "obligations", _]
                ),
                _ => false,
            };
            if !covered {
                return Err(ReadOnlyAuthoritySnapshotErrorV1::UnsafeNamespaceEntry {
                    namespace: NAMESPACE,
                    name: path,
                });
            }
            match child.kind {
                EntryKind::Directory => directories.push(Self::capture_world_completion_tree(
                    &guard.directory,
                    child,
                    &path,
                    depth + 1,
                    files,
                )?),
                EntryKind::RegularFile => {
                    let file = ReadOnlyFileGuardV1::capture(&guard.directory, child, NAMESPACE)?;
                    files.insert(path, file.bytes.clone());
                    retained_files.push(file);
                }
                _ => {
                    return Err(ReadOnlyAuthoritySnapshotErrorV1::UnsafeNamespaceEntry {
                        namespace: NAMESPACE,
                        name: child.name.clone(),
                    })
                }
            }
        }
        let tree = Self {
            guard,
            directories,
            files: retained_files,
        };
        tree.verify_world_completion_tree(parent)?;
        Ok(tree)
    }

    fn verify_world_completion_tree(
        &self,
        parent: &TrustedDirectory,
    ) -> Result<(), ReadOnlyAuthoritySnapshotErrorV1> {
        self.guard.verify(parent)?;
        for file in &self.files {
            file.verify(&self.guard.directory)?;
        }
        for directory in &self.directories {
            directory.verify_world_completion_tree(&self.guard.directory)?;
        }
        self.guard.verify(parent)
    }
}

#[cfg(target_os = "linux")]
impl HeldAcceptedWorkAuthorityPhysicalSnapshotV1 {
    pub(crate) fn capture_world_controller_completion_sources(
        &self,
        session_id: &str,
    ) -> Result<HeldWorldControllerCompletionPhysicalSourcesV1<'_>, ReadOnlyAuthoritySnapshotErrorV1>
    {
        const NAMESPACE: &str = "obligation-ledger/World-controller-session";
        if session_id.is_empty()
            || session_id == "."
            || session_id == ".."
            || session_id.contains('/')
            || session_id.contains('\\')
        {
            return Err(ReadOnlyAuthoritySnapshotErrorV1::UnsafeNamespaceEntry {
                namespace: NAMESPACE,
                name: session_id.into(),
            });
        }
        self.verify_sources()?;
        let admission = self
            .authority
            .directory
            .entries()
            .map_err(|_| ReadOnlyAuthoritySnapshotErrorV1::Io {
                operation: "enumerate held admission namespace",
            })?
            .iter()
            .find(|entry| entry.name == RETAINED_ADMISSION_DIRECTORY)
            .map(|entry| {
                ReadOnlyWorldControllerAdmissionSourcesV1::capture_admission_sources(
                    &self.authority.directory,
                    entry,
                )
            })
            .transpose()?;
        let supervisor = self
            .b1
            .as_ref()
            .and_then(|b1| b1.agent_hub.as_ref())
            .map(|hub| {
                if let Some(entry) = hub.manifest.iter().find(|entry| {
                    entry
                        .name
                        .starts_with(WORLD_WORK_EXECUTION_SUPERVISOR_PREFIX)
                        && entry.name != WORLD_WORK_EXECUTION_SUPERVISOR_FILE
                }) {
                    return Err(ReadOnlyAuthoritySnapshotErrorV1::UnsafeNamespaceEntry {
                        namespace: NAMESPACE,
                        name: entry.name.clone(),
                    });
                }
                hub.manifest
                    .iter()
                    .find(|entry| entry.name == WORLD_WORK_EXECUTION_SUPERVISOR_FILE)
                    .map(|entry| ReadOnlyFileGuardV1::capture(&hub.directory, entry, NAMESPACE))
                    .transpose()
            })
            .transpose()?
            .flatten();
        let legacy_sessions = self
            .b1
            .as_ref()
            .and_then(|b1| b1.agent_hub.as_ref())
            .and_then(|hub| {
                hub.manifest
                    .iter()
                    .find(|entry| entry.name == "sessions")
                    .map(|entry| (hub, entry))
            })
            .map(|(hub, entry)| ReadOnlyDirectoryGuardV1::capture(&hub.directory, entry, NAMESPACE))
            .transpose()?;
        if legacy_sessions.as_ref().is_some_and(|sessions| {
            sessions
                .manifest
                .iter()
                .any(|entry| entry.name == session_id)
        }) {
            return Err(ReadOnlyAuthoritySnapshotErrorV1::UnsafeNamespaceEntry {
                namespace: NAMESPACE,
                name: "uncovered legacy session source".into(),
            });
        }
        let ledger_parent = self
            .root
            .directory()
            .entries()
            .map_err(|_| ReadOnlyAuthoritySnapshotErrorV1::Io {
                operation: "enumerate World controller ledger parent",
            })?
            .into_iter()
            .find(|entry| entry.name == POST_HSA_OBLIGATION_LEDGER_DIRECTORY)
            .map(|entry| {
                ReadOnlyDirectoryGuardV1::capture(self.root.directory(), &entry, NAMESPACE)
            })
            .transpose()?;
        let mut ledger_files = BTreeMap::new();
        let ledger = ledger_parent
            .as_ref()
            .and_then(|parent| {
                parent
                    .manifest
                    .iter()
                    .find(|entry| entry.name == session_id)
                    .map(|entry| (parent, entry))
            })
            .map(|(parent, entry)| {
                ReadOnlyWorldCompletionTreeV1::capture_world_completion_tree(
                    &parent.directory,
                    entry,
                    "",
                    0,
                    &mut ledger_files,
                )
            })
            .transpose()?;
        let held = HeldWorldControllerCompletionPhysicalSourcesV1 {
            source: self,
            admission,
            supervisor,
            legacy_sessions,
            ledger_parent,
            ledger,
            session_id: session_id.into(),
            ledger_files,
        };
        held.verify_sources()?;
        Ok(held)
    }
}

#[cfg(target_os = "linux")]
impl HeldWorldControllerCompletionPhysicalSourcesV1<'_> {
    pub(crate) fn admission_registry_bytes(&self) -> Option<&[u8]> {
        self.admission
            .as_ref()
            .map(|admission| admission.registry.bytes.as_slice())
    }

    pub(crate) fn admission_key_files(&self) -> Vec<(String, Vec<u8>)> {
        self.admission
            .as_ref()
            .map(|admission| {
                admission
                    .key_files
                    .iter()
                    .map(|file| (file.entry.name.clone(), file.bytes.clone()))
                    .collect()
            })
            .unwrap_or_default()
    }
    pub(crate) fn session_id(&self) -> &str {
        &self.session_id
    }

    pub(crate) fn supervisor_bytes(&self) -> Option<&[u8]> {
        self.supervisor.as_ref().map(|file| file.bytes.as_slice())
    }

    pub(crate) fn ledger_files(&self) -> &BTreeMap<String, Vec<u8>> {
        &self.ledger_files
    }

    pub(crate) fn accepted_work_source(&self) -> &HeldAcceptedWorkAuthorityPhysicalSnapshotV1 {
        self.source
    }

    pub(crate) fn verify_sources(&self) -> Result<(), ReadOnlyAuthoritySnapshotErrorV1> {
        self.source.verify_sources()?;
        match &self.admission {
            Some(admission) => {
                admission.verify_admission_sources(&self.source.authority.directory)?
            }
            None if self
                .source
                .authority
                .directory
                .entry_kind(RETAINED_ADMISSION_DIRECTORY)
                .map_err(|_| ReadOnlyAuthoritySnapshotErrorV1::Io {
                    operation: "verify absent admission namespace",
                })?
                .is_some() =>
            {
                return Err(ReadOnlyAuthoritySnapshotErrorV1::NamespaceChanged {
                    namespace: "authority-v1/retained-worker-admission-v1",
                });
            }
            None => {}
        }
        if let Some(hub) = self.source.b1.as_ref().and_then(|b1| b1.agent_hub.as_ref()) {
            match &self.legacy_sessions {
                Some(sessions) => sessions.verify(&hub.directory)?,
                None if hub
                    .directory
                    .entry_kind("sessions")
                    .map_err(|_| ReadOnlyAuthoritySnapshotErrorV1::Io {
                        operation: "verify absent World legacy attention namespace",
                    })?
                    .is_some() =>
                {
                    return Err(ReadOnlyAuthoritySnapshotErrorV1::NamespaceChanged {
                        namespace: "run/agent-hub/sessions",
                    })
                }
                None => {}
            }
            match &self.supervisor {
                Some(file) => file.verify(&hub.directory)?,
                None if hub
                    .directory
                    .entry_kind(WORLD_WORK_EXECUTION_SUPERVISOR_FILE)
                    .map_err(|_| ReadOnlyAuthoritySnapshotErrorV1::Io {
                        operation: "verify absent World supervisor",
                    })?
                    .is_some() =>
                {
                    return Err(ReadOnlyAuthoritySnapshotErrorV1::NamespaceChanged {
                        namespace: "run/agent-hub/World-supervisor",
                    });
                }
                None => {}
            }
        }
        match &self.ledger_parent {
            Some(parent) => {
                parent.verify(self.source.root.directory())?;
                if let Some(tree) = &self.ledger {
                    tree.verify_world_completion_tree(&parent.directory)?;
                }
            }
            None if self
                .source
                .root
                .directory()
                .entry_kind(POST_HSA_OBLIGATION_LEDGER_DIRECTORY)
                .map_err(|_| ReadOnlyAuthoritySnapshotErrorV1::Io {
                    operation: "verify absent World ledger",
                })?
                .is_some() =>
            {
                return Err(ReadOnlyAuthoritySnapshotErrorV1::NamespaceChanged {
                    namespace: "obligation-ledger",
                });
            }
            None => {}
        }
        self.source.verify_sources()
    }
}

#[cfg(target_os = "linux")]
pub(in crate::execution::agent_runtime::host_session_authority) fn hold_existing_accepted_work_authority_snapshot_opened(
    opened: &TrustedAuthorityRoot,
) -> Result<HeldAcceptedWorkAuthorityPhysicalSnapshotV1, ReadOnlyAuthoritySnapshotErrorV1> {
    hold_existing_accepted_work_authority_snapshot_opened_with_source(opened, |_| ())
        .map(|(held, ())| held)
}

#[cfg(target_os = "linux")]
pub(in crate::execution::agent_runtime::host_session_authority) fn hold_existing_accepted_work_authority_snapshot_opened_with_source<
    R,
>(
    opened: &TrustedAuthorityRoot,
    operation: impl FnOnce(&ReadOnlyVersionedAuthorityTransactionV1<'_>) -> R,
) -> Result<(HeldAcceptedWorkAuthorityPhysicalSnapshotV1, R), ReadOnlyAuthoritySnapshotErrorV1> {
    let (transaction, temporary_failure) = open_existing_versioned_read_only_transaction(opened)?;
    let result = (|| {
        if let Some(error) = temporary_failure {
            return Err(error);
        }
        let e2 = DispatchPolicyCommitmentReadCapabilityV1::from_transaction(&transaction)
            .read_existing_snapshot()?;
        let b1 = transaction.read_world_work_receipt_registry_snapshot()?;
        let snapshot = AcceptedWorkAuthorityPhysicalSnapshotV1 {
            authority_root_identity: transaction.authority_root_identity().clone(),
            authority_store_id: transaction.authority_store_id().to_owned(),
            hsa_state_root_bytes: transaction.root_bytes().to_vec(),
            hsa_root_revision_at_read: transaction.root_revision(),
            e2,
            b1,
        };
        Ok((snapshot, operation(&transaction)))
    })();
    #[cfg(test)]
    run_e2_rm_before_final_verify_hook();
    // Failure does not bypass the original full read finalizer.
    transaction.verify_scope()?;
    let (snapshot, output) = result?;
    let ReadOnlyVersionedAuthorityTransactionV1 {
        reopened_root,
        authority,
        e2_guard,
        b1_guard,
        typed_object_guards,
        _lock,
        bootstrap_metadata,
        bootstrap_manifest,
        authority_children_metadata,
        root_file,
        init_file,
        lock_directory,
        temporary_directory,
        authority_temporary_files,
        root_lock,
        ..
    } = transaction;
    let held = HeldAcceptedWorkAuthorityPhysicalSnapshotV1 {
        snapshot,
        control_spine: HeldAuthorityControlSpineV1 {
            bootstrap_metadata,
            bootstrap_manifest,
            authority_children_metadata,
            root_file,
            init_file,
            lock_directory,
            temporary_directory,
            authority_temporary_files,
            root_lock,
        },
        typed_object_guards: typed_object_guards.into_inner(),
        root: reopened_root,
        authority,
        e2: e2_guard.into_inner(),
        b1: b1_guard.into_inner(),
    };
    held.verify_sources()?;
    drop(_lock);
    Ok((held, output))
}

#[cfg(target_os = "linux")]
pub(super) fn with_opened_existing_versioned_read_only_snapshot<T>(
    opened: &TrustedAuthorityRoot,
    operation: impl FnOnce(
        &ReadOnlyVersionedAuthorityTransactionV1<'_>,
    ) -> Result<T, ReadOnlyAuthoritySnapshotErrorV1>,
) -> Result<T, ReadOnlyAuthoritySnapshotErrorV1> {
    let (transaction, authority_temporary_failure) =
        open_existing_versioned_read_only_transaction(opened)?;
    let result = match authority_temporary_failure {
        Some(error) => Err(error),
        None => operation(&transaction),
    };
    #[cfg(test)]
    run_e2_rm_before_final_verify_hook();
    let verification = transaction.verify_scope();
    match result {
        Ok(value) => verification.map(|()| value),
        Err(error) => {
            verification?;
            Err(error)
        }
    }
}

#[cfg(target_os = "linux")]
pub(super) fn with_opened_existing_versioned_reconciling_current_snapshot<T>(
    opened: &TrustedAuthorityRoot,
    operation: impl FnOnce(
        &ReadOnlyVersionedAuthorityTransactionV1<'_>,
    ) -> Result<T, ReadOnlyAuthoritySnapshotErrorV1>,
) -> Result<T, CurrentAuthoritySnapshotErrorV1> {
    let (transaction, temporary_failure) = open_existing_versioned_captured_transaction(
        opened,
        CurrentSnapshotPreflightV1::ReconcilingCurrent,
    )?;
    let result = match temporary_failure {
        Some(error) => Err(error),
        None => operation(&transaction),
    };
    #[cfg(test)]
    run_reconciling_current_before_final_verify_hook();
    let verification = transaction.verify_scope();
    match result {
        Ok(value) => verification.map(|()| value).map_err(Into::into),
        Err(error) => {
            verification?;
            Err(error.into())
        }
    }
}

#[cfg(target_os = "linux")]
pub(in crate::execution::agent_runtime::host_session_authority) fn with_existing_retained_worker_admission_read_opened<
    T,
>(
    opened: &TrustedAuthorityRoot,
    operation: impl FnOnce(&VersionedStateRoot, Option<&[u8]>, Vec<(String, Vec<u8>)>) -> T,
) -> Result<T, ReadOnlyAuthoritySnapshotErrorV1> {
    with_opened_existing_versioned_read_only_snapshot(opened, |transaction| {
        with_existing_retained_worker_admission_read_snapshot(transaction, operation)
    })
}

#[cfg(target_os = "linux")]
fn with_existing_retained_worker_admission_read_snapshot<T>(
    transaction: &ReadOnlyVersionedAuthorityTransactionV1<'_>,
    operation: impl FnOnce(&VersionedStateRoot, Option<&[u8]>, Vec<(String, Vec<u8>)>) -> T,
) -> Result<T, ReadOnlyAuthoritySnapshotErrorV1> {
    let admission = transaction
        .authority
        .manifest
        .iter()
        .find(|entry| entry.name == RETAINED_ADMISSION_DIRECTORY)
        .map(|entry| {
            ReadOnlyWorldControllerAdmissionSourcesV1::capture_admission_sources(
                &transaction.authority.directory,
                entry,
            )
        })
        .transpose()?;
    let bytes = admission
        .as_ref()
        .map(|held| held.registry.bytes.as_slice());
    let keys = admission
        .as_ref()
        .map(|held| {
            held.key_files
                .iter()
                .map(|file| (file.entry.name.clone(), file.bytes.clone()))
                .collect()
        })
        .unwrap_or_default();
    // T may itself be a semantic Result. Both its success and refusal still
    // pass the same physical finalizers; integrity errors take precedence.
    let result = operation(&transaction.decoded_root, bytes, keys);
    if let Some(held) = admission.as_ref() {
        held.verify_admission_sources(&transaction.authority.directory)?;
    }
    Ok(result)
}

/// Compose the existing E2/B1/admission owners under this same original read.
#[cfg(target_os = "linux")]
pub(in crate::execution::agent_runtime::host_session_authority) fn with_existing_ordinary_retained_inspection_snapshot<
    T,
>(
    transaction: &ReadOnlyVersionedAuthorityTransactionV1<'_>,
    operation: impl FnOnce(
        &AcceptedWorkAuthorityPhysicalSnapshotV1,
        Option<&[u8]>,
        Vec<(String, Vec<u8>)>,
        Option<&[u8]>,
    ) -> T,
) -> Result<T, ReadOnlyAuthoritySnapshotErrorV1> {
    let e2 = DispatchPolicyCommitmentReadCapabilityV1::from_transaction(transaction)
        .read_existing_snapshot()?;
    let b1 = transaction.read_world_work_receipt_registry_snapshot()?;
    let physical = AcceptedWorkAuthorityPhysicalSnapshotV1 {
        authority_root_identity: transaction.authority_root_identity().clone(),
        authority_store_id: transaction.authority_store_id().to_owned(),
        hsa_state_root_bytes: transaction.root_bytes().to_vec(),
        hsa_root_revision_at_read: transaction.root_revision(),
        e2,
        b1,
    };
    let b1_guard = transaction.b1_guard.borrow();
    let hub = b1_guard.as_ref().and_then(|guard| guard.agent_hub.as_ref());
    const NAMESPACE: &str = "run/agent-hub/ordinary-inspection-supervisor";
    let supervisor = hub
        .map(|hub| {
            if let Some(entry) = hub.manifest.iter().find(|entry| {
                entry
                    .name
                    .starts_with(WORLD_WORK_EXECUTION_SUPERVISOR_PREFIX)
                    && entry.name != WORLD_WORK_EXECUTION_SUPERVISOR_FILE
            }) {
                return Err(ReadOnlyAuthoritySnapshotErrorV1::UnsafeNamespaceEntry {
                    namespace: NAMESPACE,
                    name: entry.name.clone(),
                });
            }
            hub.manifest
                .iter()
                .find(|entry| entry.name == WORLD_WORK_EXECUTION_SUPERVISOR_FILE)
                .map(|entry| ReadOnlyFileGuardV1::capture(&hub.directory, entry, NAMESPACE))
                .transpose()
        })
        .transpose()?
        .flatten();
    let result =
        with_existing_retained_worker_admission_read_snapshot(transaction, |_, bytes, keys| {
            operation(
                &physical,
                bytes,
                keys,
                supervisor.as_ref().map(|file| file.bytes.as_slice()),
            )
        });
    // Verify even when the admission capture or semantic consumer refused.
    if let Some(hub) = hub {
        if let Some(file) = &supervisor {
            file.verify(&hub.directory)?;
        }
        hub.verify(
            &b1_guard
                .as_ref()
                .ok_or(ReadOnlyAuthoritySnapshotErrorV1::Io {
                    operation: "retain original inspection B1 guard",
                })?
                .run
                .directory,
        )?;
    }
    result
}

/// Existing admission source custody inside an already-held HSA writer guard.
/// It acquires no second root lock and never creates or reconciles admission.
/// E2 may publish only its own namespace while these admission guards remain
/// held; semantic refusal still passes every physical finalizer.
#[cfg(target_os = "linux")]
pub(super) fn with_existing_retained_admission_source_in_locked_authority<T>(
    authority: &TrustedDirectory,
    operation: impl FnOnce(
        &[u8],
        Vec<(String, Vec<u8>)>,
        &dyn Fn() -> Result<(), BootstrapError>,
    ) -> Result<T, BootstrapError>,
) -> Result<T, BootstrapError> {
    let entry = authority
        .entries()
        .map_err(|_| BootstrapError("enumerate locked admission source"))?
        .into_iter()
        .find(|entry| entry.name == RETAINED_ADMISSION_DIRECTORY)
        .ok_or(BootstrapError("required existing admission source absent"))?;
    let held =
        ReadOnlyWorldControllerAdmissionSourcesV1::capture_admission_sources(authority, &entry)
            .map_err(|_| BootstrapError("capture locked admission source refused"))?;
    let keys = held
        .key_files
        .iter()
        .map(|file| (file.entry.name.clone(), file.bytes.clone()))
        .collect();
    let verify = || {
        held.verify_admission_sources(authority)
            .map_err(|_| BootstrapError("locked admission source changed"))
    };
    let result = operation(&held.registry.bytes, keys, &verify);
    verify()?;
    result
}

#[cfg(target_os = "linux")]
fn open_existing_versioned_read_only_transaction(
    opened: &TrustedAuthorityRoot,
) -> Result<
    (
        ReadOnlyVersionedAuthorityTransactionV1<'_>,
        Option<ReadOnlyAuthoritySnapshotErrorV1>,
    ),
    ReadOnlyAuthoritySnapshotErrorV1,
> {
    open_existing_versioned_captured_transaction(opened, CurrentSnapshotPreflightV1::Nonreconciling)
        .map_err(|error| match error {
            CurrentAuthoritySnapshotErrorV1::Custody(error) => error,
            // This branch is unavailable to the nonreconciling mode, but keep
            // error handling total without converting a refusal into a panic.
            CurrentAuthoritySnapshotErrorV1::Semantic(error) => {
                ReadOnlyAuthoritySnapshotErrorV1::Io { operation: error.0 }
            }
        })
}

#[cfg(target_os = "linux")]
#[derive(Clone, Copy)]
enum CurrentSnapshotPreflightV1 {
    Nonreconciling,
    ReconcilingCurrent,
}

#[cfg(target_os = "linux")]
fn open_existing_versioned_captured_transaction(
    opened: &TrustedAuthorityRoot,
    preflight: CurrentSnapshotPreflightV1,
) -> Result<
    (
        ReadOnlyVersionedAuthorityTransactionV1<'_>,
        Option<ReadOnlyAuthoritySnapshotErrorV1>,
    ),
    CurrentAuthoritySnapshotErrorV1,
> {
    opened
        .revalidate()
        .map_err(|_| ReadOnlyAuthoritySnapshotErrorV1::AuthorityRootAbsentOrUnsafe)?;
    // Preserve the already-authenticated owner when a privileged service reads
    // an accepted developer home. This duplicates custody, not writer authority.
    let reopened = opened
        .reopen_for_dispatch_policy_commitment_storage()
        .map_err(|_| ReadOnlyAuthoritySnapshotErrorV1::AuthorityRootAbsentOrUnsafe)?;
    if reopened.identity() != opened.identity() {
        return Err(ReadOnlyAuthoritySnapshotErrorV1::AuthorityRootReplaced.into());
    }
    let initial_bootstrap_manifest = reopened
        .directory()
        .entries()
        .map_err(|_| ReadOnlyAuthoritySnapshotErrorV1::AuthorityRootAbsentOrUnsafe)?;
    let initial_authority_entry = initial_bootstrap_manifest
        .iter()
        .find(|entry| entry.name == AUTHORITY_DIRECTORY)
        .cloned()
        .ok_or(ReadOnlyAuthoritySnapshotErrorV1::AuthorityLayoutAbsentOrUnsafe)?;
    let initial_authority = reopened
        .directory()
        .open_directory(&initial_authority_entry.name)
        .map_err(|_| ReadOnlyAuthoritySnapshotErrorV1::AuthorityLayoutAbsentOrUnsafe)?;
    let initial_authority_manifest = initial_authority
        .entries()
        .map_err(|_| ReadOnlyAuthoritySnapshotErrorV1::AuthorityLayoutAbsentOrUnsafe)?;
    let initial_lock_entry = initial_authority_manifest
        .iter()
        .find(|entry| entry.name == "lock")
        .cloned()
        .ok_or(ReadOnlyAuthoritySnapshotErrorV1::AuthorityRootLockAbsentOrUnsafe)?;
    let initial_lock_directory = initial_authority
        .open_directory(&initial_lock_entry.name)
        .map_err(|_| ReadOnlyAuthoritySnapshotErrorV1::AuthorityRootLockAbsentOrUnsafe)?;
    let initial_lock_manifest = initial_lock_directory
        .entries()
        .map_err(|_| ReadOnlyAuthoritySnapshotErrorV1::AuthorityRootLockAbsentOrUnsafe)?;
    if initial_lock_manifest.len() != 1
        || initial_lock_manifest[0].name != ROOT_LOCK_FILE
        || initial_lock_manifest[0].kind != EntryKind::RegularFile
    {
        return Err(ReadOnlyAuthoritySnapshotErrorV1::AuthorityRootLockAbsentOrUnsafe.into());
    }
    let initial_root_lock_entry = initial_lock_manifest[0].clone();
    let root_lock_file = initial_lock_directory
        .open_file_entry(&initial_root_lock_entry)
        .map_err(|_| ReadOnlyAuthoritySnapshotErrorV1::AuthorityRootLockAbsentOrUnsafe)?;
    let lock = root_lock_file
        .lock_exclusive_owned()
        .map_err(|_| ReadOnlyAuthoritySnapshotErrorV1::AuthorityLockFailed)?;

    let semantic_root = match preflight {
        CurrentSnapshotPreflightV1::Nonreconciling => None,
        CurrentSnapshotPreflightV1::ReconcilingCurrent => {
            // Join every reopened anchor to the actual inode whose flock we
            // retain before the existing owner performs any reconciliation.
            reopened
                .directory()
                .revalidate_entry(&initial_authority_entry)
                .map_err(|_| ReadOnlyAuthoritySnapshotErrorV1::AuthorityRootReplaced)?;
            initial_authority
                .revalidate_entry(&initial_lock_entry)
                .map_err(|_| ReadOnlyAuthoritySnapshotErrorV1::AuthorityRootLockAbsentOrUnsafe)?;
            initial_lock_directory
                .revalidate_entry(&initial_root_lock_entry)
                .map_err(|_| ReadOnlyAuthoritySnapshotErrorV1::AuthorityRootLockAbsentOrUnsafe)?;
            let scope = StoreLayoutLockScope::open_existing_activated(opened.directory())
                .map_err(|_| ReadOnlyAuthoritySnapshotErrorV1::AuthorityLayoutAbsentOrUnsafe)?;
            if scope
                .authority
                .metadata()
                .map_err(|_| ReadOnlyAuthoritySnapshotErrorV1::AuthorityRootReplaced)?
                != initial_authority
                    .metadata()
                    .map_err(|_| ReadOnlyAuthoritySnapshotErrorV1::AuthorityRootReplaced)?
                || scope.lock.metadata().map_err(|_| {
                    ReadOnlyAuthoritySnapshotErrorV1::AuthorityRootLockAbsentOrUnsafe
                })? != initial_lock_directory.metadata().map_err(|_| {
                    ReadOnlyAuthoritySnapshotErrorV1::AuthorityRootLockAbsentOrUnsafe
                })?
            {
                return Err(ReadOnlyAuthoritySnapshotErrorV1::AuthorityRootReplaced.into());
            }
            scope
                .lock
                .revalidate_entry(&initial_root_lock_entry)
                .map_err(|_| ReadOnlyAuthoritySnapshotErrorV1::AuthorityRootLockAbsentOrUnsafe)?;
            Some(
                with_existing_versioned_semantic_scope(opened, scope, &lock, |transaction| {
                    Ok(transaction.root.clone())
                })
                .map_err(CurrentAuthoritySnapshotErrorV1::Semantic)?,
            )
        }
    };

    opened
        .revalidate()
        .map_err(|_| ReadOnlyAuthoritySnapshotErrorV1::AuthorityRootReplaced)?;
    reopened
        .revalidate()
        .map_err(|_| ReadOnlyAuthoritySnapshotErrorV1::AuthorityRootReplaced)?;
    let bootstrap_metadata = reopened
        .directory()
        .metadata()
        .map_err(|_| ReadOnlyAuthoritySnapshotErrorV1::AuthorityRootAbsentOrUnsafe)?;
    let bootstrap_manifest = reopened
        .directory()
        .entries()
        .map_err(|_| ReadOnlyAuthoritySnapshotErrorV1::AuthorityRootAbsentOrUnsafe)?;
    let authority_entry = bootstrap_manifest
        .iter()
        .find(|entry| entry.name == AUTHORITY_DIRECTORY)
        .ok_or(ReadOnlyAuthoritySnapshotErrorV1::AuthorityLayoutAbsentOrUnsafe)?;
    if authority_entry != &initial_authority_entry {
        return Err(ReadOnlyAuthoritySnapshotErrorV1::AuthorityRootReplaced.into());
    }
    let authority = ReadOnlyDirectoryGuardV1::capture(
        reopened.directory(),
        authority_entry,
        E2_RM_AUTHORITY_NAMESPACE,
    )
    .map_err(|_| ReadOnlyAuthoritySnapshotErrorV1::AuthorityLayoutAbsentOrUnsafe)?;
    validate_e2_rm_authority_manifest(&authority.manifest)?;

    let lock_entry = authority
        .manifest
        .iter()
        .find(|entry| entry.name == "lock")
        .ok_or(ReadOnlyAuthoritySnapshotErrorV1::AuthorityRootLockAbsentOrUnsafe)?;
    if lock_entry != &initial_lock_entry {
        return Err(ReadOnlyAuthoritySnapshotErrorV1::AuthorityRootReplaced.into());
    }
    let lock_directory = ReadOnlyDirectoryGuardV1::capture(
        &authority.directory,
        lock_entry,
        E2_RM_AUTHORITY_LOCK_NAMESPACE,
    )
    .map_err(|_| ReadOnlyAuthoritySnapshotErrorV1::AuthorityRootLockAbsentOrUnsafe)?;
    if lock_directory.manifest.len() != 1 || lock_directory.manifest[0] != initial_root_lock_entry {
        return Err(ReadOnlyAuthoritySnapshotErrorV1::AuthorityRootLockAbsentOrUnsafe.into());
    }
    let root_lock = ReadOnlyFileGuardV1::capture(
        &lock_directory.directory,
        &lock_directory.manifest[0],
        E2_RM_AUTHORITY_LOCK_NAMESPACE,
    )
    .map_err(|_| ReadOnlyAuthoritySnapshotErrorV1::AuthorityRootLockAbsentOrUnsafe)?;

    let mut authority_children_metadata = BTreeMap::new();
    for entry in &authority.manifest {
        let metadata = authority.directory.entry_metadata(entry).map_err(|_| {
            ReadOnlyAuthoritySnapshotErrorV1::UnsafeNamespaceEntry {
                namespace: E2_RM_AUTHORITY_NAMESPACE,
                name: entry.name.clone(),
            }
        })?;
        authority_children_metadata.insert(entry.name.clone(), metadata);
    }
    let root_entry = authority
        .manifest
        .iter()
        .find(|entry| entry.name == ROOT_FILE)
        .ok_or(ReadOnlyAuthoritySnapshotErrorV1::AuthorityLayoutAbsentOrUnsafe)?;
    let root_file =
        ReadOnlyFileGuardV1::capture(&authority.directory, root_entry, E2_RM_AUTHORITY_NAMESPACE)?;
    let init_file = authority
        .manifest
        .iter()
        .find(|entry| entry.name == INIT_FILE)
        .map(|entry| {
            ReadOnlyFileGuardV1::capture(&authority.directory, entry, E2_RM_AUTHORITY_NAMESPACE)
        })
        .transpose()?;

    let temporary_entry = authority
        .manifest
        .iter()
        .find(|entry| entry.name == "tmp")
        .ok_or(ReadOnlyAuthoritySnapshotErrorV1::AuthorityLayoutAbsentOrUnsafe)?;
    let temporary_directory = ReadOnlyDirectoryGuardV1::capture(
        &authority.directory,
        temporary_entry,
        E2_RM_AUTHORITY_TEMP_NAMESPACE,
    )?;
    let mut authority_temporary_files = Vec::new();
    let mut authority_temporary_failure = None;
    for entry in &temporary_directory.manifest {
        if entry.kind != EntryKind::RegularFile || TempNameV1::parse(&entry.name).is_err() {
            authority_temporary_failure.get_or_insert(
                ReadOnlyAuthoritySnapshotErrorV1::UnsafeTemporaryMaterial {
                    namespace: E2_RM_AUTHORITY_TEMP_NAMESPACE,
                    name: entry.name.clone(),
                },
            );
        }
        if entry.kind != EntryKind::RegularFile {
            continue;
        }
        match ReadOnlyFileGuardV1::capture(
            &temporary_directory.directory,
            entry,
            E2_RM_AUTHORITY_TEMP_NAMESPACE,
        ) {
            Ok(temporary) => {
                authority_temporary_failure.get_or_insert(
                    ReadOnlyAuthoritySnapshotErrorV1::AuthorityTemporaryMaterialPresent {
                        name: temporary.entry.name.clone(),
                    },
                );
                authority_temporary_files.push(temporary);
            }
            Err(error) => {
                authority_temporary_failure.get_or_insert(error);
            }
        }
    }
    let decoded_root = decode_read_only_versioned_root(&root_file.bytes)?;
    if decoded_root.bootstrap_home() != opened.identity() {
        return Err(ReadOnlyAuthoritySnapshotErrorV1::AuthorityRootReplaced.into());
    }
    if decoded_root
        .to_canonical_bytes()
        .map_err(|_| ReadOnlyAuthoritySnapshotErrorV1::AuthorityRootEncodingInvalid)?
        != root_file.bytes
    {
        return Err(ReadOnlyAuthoritySnapshotErrorV1::AuthorityRootEncodingInvalid.into());
    }

    let transaction = ReadOnlyVersionedAuthorityTransactionV1 {
        supplied_root: opened,
        reopened_root: reopened,
        bootstrap_metadata,
        bootstrap_manifest,
        authority,
        authority_children_metadata,
        root_file,
        init_file,
        lock_directory,
        temporary_directory,
        authority_temporary_files,
        root_lock,
        decoded_root,
        e2_guard: RefCell::new(None),
        b1_guard: RefCell::new(None),
        typed_object_guards: RefCell::new(Vec::new()),
        _root_lock_file: root_lock_file,
        _lock: lock,
    };
    // V3's structural view cannot authenticate interleaved Start/retained-worker
    // ancestry. Complete that validation using objects held by this transaction,
    // without reopening the Root or taking a nested lock. Partial object/key
    // captures must pass the same physical finalizer even on semantic refusal.
    let semantics = if semantic_root
        .as_ref()
        .is_some_and(|root| root != &transaction.decoded_root)
    {
        Err(ReadOnlyAuthoritySnapshotErrorV1::AuthorityRootReplaced)
    } else {
        transaction.validate_captured_exact_history()
    };
    transaction.verify_scope()?;
    semantics?;
    Ok((transaction, authority_temporary_failure))
}

#[cfg(target_os = "linux")]
struct ConfigProjectionHsaParentTransactionV1<'root> {
    trusted_root: &'root TrustedAuthorityRoot,
    layout: StoreLayout<'root>,
    versioned_root: VersionedStateRoot,
    legacy: LegacyObservation,
    non_e3_entries: BTreeMap<String, (DirectoryEntry, TrustedEntryMetadataV1)>,
    e3_entry: Option<DirectoryEntry>,
    _lock: TrustedOwnedFileLock,
}

#[cfg(target_os = "linux")]
impl<'root> ConfigProjectionHsaParentTransactionV1<'root> {
    fn begin(
        trusted_root: &'root TrustedAuthorityRoot,
    ) -> Result<Self, config_projection::ConfigProjectionFailureV1> {
        trusted_root
            .revalidate()
            .map_err(config_projection_hsa_error)?;
        let lock_scope = StoreLayoutLockScope::open_existing_activated(trusted_root.directory())
            .map_err(config_projection_hsa_error)?;
        let lock = lock_scope
            .root_lock
            .lock_exclusive_owned()
            .map_err(config_projection_hsa_error)?;
        trusted_root
            .revalidate()
            .map_err(config_projection_hsa_error)?;
        lock_scope
            .validate_temps()
            .map_err(config_projection_hsa_error)?;
        let layout = lock_scope.finish().map_err(config_projection_hsa_error)?;
        Self::validate_locked_layout(&layout)?;
        let legacy =
            LegacyObservation::capture(layout.bootstrap).map_err(config_projection_hsa_error)?;
        if legacy.has_artifact {
            return Err(config_projection::ConfigProjectionFailureV1::UnsupportedSecurityPosture);
        }
        let versioned_root = layout
            .read_existing_versioned_without_reconciliation(trusted_root.identity())
            .map_err(config_projection_hsa_error)?;
        Self::validate_marker(&layout, &versioned_root)?;
        let (non_e3_entries, e3_entry) = Self::capture_children(&layout)?;
        let transaction = Self {
            trusted_root,
            layout,
            versioned_root,
            legacy,
            non_e3_entries,
            e3_entry,
            _lock: lock,
        };
        transaction.verify_scope()?;
        Ok(transaction)
    }

    fn authority_fd(&self) -> std::os::fd::BorrowedFd<'_> {
        self.layout.authority.borrow_fd()
    }

    fn finish(self) -> Result<(), config_projection::ConfigProjectionFailureV1> {
        self.verify_scope()
    }

    fn verify_scope(&self) -> Result<(), config_projection::ConfigProjectionFailureV1> {
        self.trusted_root
            .revalidate()
            .map_err(config_projection_hsa_error)?;
        Self::validate_locked_layout(&self.layout)?;
        self.legacy
            .revalidate(self.layout.bootstrap)
            .map_err(config_projection_hsa_error)?;
        let current_root = self
            .layout
            .read_existing_versioned_without_reconciliation(self.trusted_root.identity())
            .map_err(config_projection_hsa_error)?;
        if current_root != self.versioned_root {
            return Err(config_projection::ConfigProjectionFailureV1::Conflict);
        }
        Self::validate_marker(&self.layout, &current_root)?;
        let (current_non_e3, current_e3) = Self::capture_children(&self.layout)?;
        if current_non_e3 != self.non_e3_entries {
            return Err(config_projection::ConfigProjectionFailureV1::Conflict);
        }
        match (&self.e3_entry, current_e3) {
            (None, Some(entry)) if entry.kind == EntryKind::Directory => {}
            (None, None) => {}
            (Some(expected), Some(current)) if expected == &current => {}
            _ => return Err(config_projection::ConfigProjectionFailureV1::Conflict),
        }
        self.trusted_root
            .revalidate()
            .map_err(config_projection_hsa_error)
    }

    fn validate_locked_layout(
        layout: &StoreLayout<'_>,
    ) -> Result<(), config_projection::ConfigProjectionFailureV1> {
        layout
            .validate_closed_layout()
            .map_err(config_projection_hsa_error)?;
        let manifest = layout
            .authority
            .entries()
            .map_err(config_projection_hsa_error)?;
        validate_e2_rm_authority_manifest(&manifest).map_err(config_projection_hsa_error)?;
        if !layout
            .tmp
            .entries()
            .map_err(config_projection_hsa_error)?
            .is_empty()
        {
            return Err(config_projection::ConfigProjectionFailureV1::PartialPublication);
        }
        Ok(())
    }

    fn validate_marker(
        layout: &StoreLayout<'_>,
        root: &VersionedStateRoot,
    ) -> Result<(), config_projection::ConfigProjectionFailureV1> {
        match root {
            VersionedStateRoot::V1(root) => layout
                .validate_matching_marker_if_present(root)
                .map_err(config_projection_hsa_error),
            VersionedStateRoot::V2(root) => layout
                .validate_matching_marker_if_present_v2(root)
                .map_err(config_projection_hsa_error),
            VersionedStateRoot::V3(root) => layout
                .validate_matching_marker_if_present_v3(root)
                .map_err(config_projection_hsa_error),
        }
    }

    #[allow(
        clippy::type_complexity,
        reason = "the snapshot deliberately retains each non-E3 entry and its exact metadata"
    )]
    fn capture_children(
        layout: &StoreLayout<'_>,
    ) -> Result<
        (
            BTreeMap<String, (DirectoryEntry, TrustedEntryMetadataV1)>,
            Option<DirectoryEntry>,
        ),
        config_projection::ConfigProjectionFailureV1,
    > {
        let mut non_e3 = BTreeMap::new();
        let mut e3 = None;
        for entry in layout
            .authority
            .entries()
            .map_err(config_projection_hsa_error)?
        {
            layout
                .authority
                .revalidate_entry(&entry)
                .map_err(config_projection_hsa_error)?;
            if entry.name == "agent-config-projection-v1" {
                if entry.kind != EntryKind::Directory || e3.replace(entry).is_some() {
                    return Err(
                        config_projection::ConfigProjectionFailureV1::UnsupportedSecurityPosture,
                    );
                }
                continue;
            }
            let metadata = layout
                .authority
                .entry_metadata(&entry)
                .map_err(config_projection_hsa_error)?;
            non_e3.insert(entry.name.clone(), (entry, metadata));
        }
        Ok((non_e3, e3))
    }
}

#[cfg(target_os = "linux")]
pub(in super::super) fn with_opened_config_projection_hsa_parent(
    trusted_root: &TrustedAuthorityRoot,
    operation: &mut dyn for<'fd> FnMut(
        std::os::fd::BorrowedFd<'fd>,
    )
        -> Result<(), config_projection::ConfigProjectionFailureV1>,
) -> Result<(), config_projection::ConfigProjectionFailureV1> {
    let transaction = ConfigProjectionHsaParentTransactionV1::begin(trusted_root)?;
    let result = operation(transaction.authority_fd());
    let verification = transaction.finish();
    match result {
        Ok(()) => verification,
        Err(error) => {
            verification?;
            Err(error)
        }
    }
}

#[cfg(target_os = "linux")]
fn config_projection_hsa_error<T>(_error: T) -> config_projection::ConfigProjectionFailureV1 {
    config_projection::ConfigProjectionFailureV1::UnsupportedSecurityPosture
}

#[cfg(target_os = "linux")]
fn decode_read_only_versioned_root(
    bytes: &[u8],
) -> Result<VersionedStateRoot, ReadOnlyAuthoritySnapshotErrorV1> {
    let syntax: serde_json::Value =
        crate::execution::agent_runtime::host_session_authority::canonical_json::from_slice(bytes)
            .map_err(|_| ReadOnlyAuthoritySnapshotErrorV1::AuthorityRootEncodingInvalid)?;
    let version = syntax
        .as_object()
        .and_then(|object| object.get("schema_version"))
        .and_then(serde_json::Value::as_u64)
        .ok_or(ReadOnlyAuthoritySnapshotErrorV1::AuthorityRootEncodingInvalid)?;
    if !matches!(version, 1..=3) {
        return Err(ReadOnlyAuthoritySnapshotErrorV1::UnsupportedAuthorityRootSchema);
    }
    let root = VersionedStateRoot::decode(bytes)
        .map_err(|_| ReadOnlyAuthoritySnapshotErrorV1::AuthorityRootEncodingInvalid)?;
    // V3 semantics are authenticated after the transaction owns the object/key
    // reader. No captured Root is exposed until that exact-history check succeeds.
    if !matches!(root, VersionedStateRoot::V3(_)) {
        root.validate()
            .map_err(|_| ReadOnlyAuthoritySnapshotErrorV1::AuthorityRootEncodingInvalid)?;
    }
    Ok(root)
}

#[cfg(target_os = "linux")]
fn validate_e2_rm_authority_manifest(
    manifest: &[DirectoryEntry],
) -> Result<(), ReadOnlyAuthoritySnapshotErrorV1> {
    for entry in manifest {
        let valid = matches!(
            (entry.name.as_str(), entry.kind),
            (
                "lock"
                    | "tmp"
                    | "objects"
                    | "keys"
                    | "retained-worker-admission-v1"
                    | "dispatch-policy-commitment-v1"
                    | "agent-config-projection-v1",
                EntryKind::Directory
            ) | (ROOT_FILE | INIT_FILE, EntryKind::RegularFile)
        );
        if !valid {
            return Err(ReadOnlyAuthoritySnapshotErrorV1::UnsafeNamespaceEntry {
                namespace: E2_RM_AUTHORITY_NAMESPACE,
                name: entry.name.clone(),
            });
        }
    }
    Ok(())
}

struct WorldWorkReceiptRegistryStorageInnerV1 {
    root: TrustedAuthorityRoot,
    authority_store_id: String,
}

#[derive(Clone)]
pub(crate) struct WorldWorkReceiptRegistryStorageV1 {
    inner: Arc<WorldWorkReceiptRegistryStorageInnerV1>,
}

impl std::fmt::Debug for WorldWorkReceiptRegistryStorageV1 {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("WorldWorkReceiptRegistryStorageV1")
            .field("bootstrap_home", self.inner.root.identity())
            .field("authority_store_id", &self.inner.authority_store_id)
            .finish()
    }
}

pub(crate) struct WorldWorkReceiptRegistryTransactionV1<'storage> {
    root: &'storage TrustedAuthorityRoot,
    authority_store_id: &'storage str,
    locked_root: VersionedStateRoot,
    authority_entry: DirectoryEntry,
    authority: TrustedDirectory,
    lock_entry: DirectoryEntry,
    lock_directory: TrustedDirectory,
    root_lock_entry: DirectoryEntry,
    _root_lock_file: TrustedFile,
    run_entry: DirectoryEntry,
    run: TrustedDirectory,
    agent_hub_entry: DirectoryEntry,
    agent_hub: TrustedDirectory,
    retained_post_hsa_ledger_directories: RetainedPostHsaLedgerDirectories,
    _lock: TrustedOwnedFileLock,
}

impl WorldWorkReceiptRegistryStorageV1 {
    #[cfg(target_os = "linux")]
    pub(crate) fn read_existing_receipt_registry_snapshot(
        &self,
    ) -> Result<WorldWorkReceiptRegistryPhysicalReadV1, ReadOnlyAuthoritySnapshotErrorV1> {
        with_opened_existing_versioned_read_only_snapshot(&self.inner.root, |transaction| {
            if transaction.authority_store_id() != self.inner.authority_store_id {
                return Err(ReadOnlyAuthoritySnapshotErrorV1::AuthorityRootEncodingInvalid);
            }
            transaction.read_world_work_receipt_registry_snapshot()
        })
    }
    pub(crate) fn bind(
        path: &std::path::Path,
        expected_root: &CanonicalDirectoryV1,
        expected_authority_store_id: &str,
    ) -> Result<Self, BootstrapError> {
        let root = TrustedAuthorityRoot::open(path)
            .map_err(|_| BootstrapError("open trusted B1 receipt authority root"))?;
        if root.identity() != expected_root {
            return Err(BootstrapError(
                "B1 receipt authority root differs from expected identity",
            ));
        }
        with_opened_existing_versioned_semantic_preflight(&root, |transaction| {
            if transaction.root.authority_store_id() != expected_authority_store_id
                || transaction.root.bootstrap_home() != expected_root
            {
                return Err(BootstrapError(
                    "B1 receipt authority store identity mismatch",
                ));
            }
            Ok(())
        })?;
        root.revalidate()
            .map_err(|_| BootstrapError("revalidate bound B1 receipt authority root"))?;
        Ok(Self {
            inner: Arc::new(WorldWorkReceiptRegistryStorageInnerV1 {
                root,
                authority_store_id: expected_authority_store_id.to_string(),
            }),
        })
    }

    pub(crate) fn begin_transaction(
        &self,
    ) -> Result<WorldWorkReceiptRegistryTransactionV1<'_>, BootstrapError> {
        begin_world_work_receipt_registry_transaction(
            &self.inner.root,
            &self.inner.authority_store_id,
        )
    }

    #[cfg(target_os = "linux")]
    pub(crate) fn begin_transaction_with_ordinary_publication(
        &self,
        publication: &crate::execution::agent_runtime::host_session_authority::world_bound_controller::AuthenticatedOrdinaryWorldPublicationV1,
    ) -> Result<WorldWorkReceiptRegistryTransactionV1<'_>, BootstrapError> {
        begin_world_work_receipt_registry_transaction_with_publication(
            &self.inner.root,
            &self.inner.authority_store_id,
            Some(publication),
        )
    }
}

struct WorldWorkExecutionSupervisorStorageInnerV1 {
    root: TrustedAuthorityRoot,
    authority_store_id: String,
}

#[derive(Clone)]
pub(crate) struct WorldWorkExecutionSupervisorStorageV1 {
    inner: Arc<WorldWorkExecutionSupervisorStorageInnerV1>,
}

impl std::fmt::Debug for WorldWorkExecutionSupervisorStorageV1 {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("WorldWorkExecutionSupervisorStorageV1")
            .field("bootstrap_home", self.inner.root.identity())
            .field("authority_store_id", &self.inner.authority_store_id)
            .finish()
    }
}

pub(crate) struct WorldWorkExecutionSupervisorTransactionV1<'storage> {
    root: &'storage TrustedAuthorityRoot,
    authority_store_id: &'storage str,
    locked_root: VersionedStateRoot,
    authority_entry: DirectoryEntry,
    authority: TrustedDirectory,
    lock_entry: DirectoryEntry,
    lock_directory: TrustedDirectory,
    root_lock_entry: DirectoryEntry,
    _root_lock_file: TrustedFile,
    run_entry: DirectoryEntry,
    run: TrustedDirectory,
    agent_hub_entry: DirectoryEntry,
    agent_hub: TrustedDirectory,
    _lock: TrustedOwnedFileLock,
}

impl WorldWorkExecutionSupervisorStorageV1 {
    pub(crate) fn bind(
        path: &std::path::Path,
        expected_root: &CanonicalDirectoryV1,
        expected_authority_store_id: &str,
    ) -> Result<Self, BootstrapError> {
        let root = TrustedAuthorityRoot::open(path)
            .map_err(|_| BootstrapError("open trusted B2.1 supervisor authority root"))?;
        if root.identity() != expected_root {
            return Err(BootstrapError(
                "B2.1 supervisor authority root differs from expected identity",
            ));
        }
        with_opened_existing_versioned_semantic_preflight(&root, |transaction| {
            if transaction.root.authority_store_id() != expected_authority_store_id
                || transaction.root.bootstrap_home() != expected_root
            {
                return Err(BootstrapError(
                    "B2.1 supervisor authority store identity mismatch",
                ));
            }
            Ok(())
        })?;
        root.revalidate()
            .map_err(|_| BootstrapError("revalidate bound B2.1 supervisor authority root"))?;
        Ok(Self {
            inner: Arc::new(WorldWorkExecutionSupervisorStorageInnerV1 {
                root,
                authority_store_id: expected_authority_store_id.to_string(),
            }),
        })
    }

    pub(crate) fn begin_transaction(
        &self,
    ) -> Result<WorldWorkExecutionSupervisorTransactionV1<'_>, BootstrapError> {
        begin_world_work_execution_supervisor_transaction(
            &self.inner.root,
            &self.inner.authority_store_id,
        )
    }
}
pub(crate) struct LegacyStateStoreTransactionV1 {
    root: TrustedAuthorityRoot,
    _legacy_observation: LegacyObservation,
    retained_directories: RetainedLegacyDirectories,
    _lock: TrustedOwnedFileLock,
}

#[derive(Default)]
struct RetainedLegacyDirectories {
    children: RetainedDirectoryChildren,
}

#[derive(Default)]
struct RetainedDirectoryChildren {
    opened: BTreeMap<String, RetainedDirectory>,
    absent: BTreeMap<String, RetainedDirectoryChildren>,
}

struct RetainedDirectory {
    entry: DirectoryEntry,
    directory: TrustedDirectory,
    children: RetainedDirectoryChildren,
}

#[derive(Default)]
struct RetainedPostHsaLedgerDirectories {
    children: RetainedDirectoryChildren,
}

impl RetainedPostHsaLedgerDirectories {
    fn with_directory<T>(
        &mut self,
        root: &TrustedDirectory,
        components: &[&str],
        create_missing: bool,
        operation: impl FnOnce(
            &TrustedDirectory,
            &mut RetainedDirectoryChildren,
        ) -> Result<T, BootstrapError>,
    ) -> Result<Option<T>, BootstrapError> {
        Self::with_child_directory(
            root,
            &mut self.children,
            components,
            create_missing,
            operation,
        )
    }

    fn with_child_directory<T>(
        parent: &TrustedDirectory,
        retained: &mut RetainedDirectoryChildren,
        components: &[&str],
        create_missing: bool,
        operation: impl FnOnce(
            &TrustedDirectory,
            &mut RetainedDirectoryChildren,
        ) -> Result<T, BootstrapError>,
    ) -> Result<Option<T>, BootstrapError> {
        let Some((component, remaining)) = components.split_first() else {
            return operation(parent, retained).map(Some);
        };
        validate_post_hsa_obligation_ledger_component(component)?;

        if retained.absent.contains_key(*component) {
            if parent
                .entry_kind(component)
                .map_err(|_| BootstrapError("revalidate absent post-HSA ledger directory"))?
                .is_some()
            {
                return Err(BootstrapError(
                    "post-HSA ledger directory appeared after transaction admission",
                ));
            }
            if !create_missing {
                return Ok(None);
            }
            retained.absent.remove(*component);
        }

        if !retained.opened.contains_key(*component) {
            match parent
                .entry_kind(component)
                .map_err(|_| BootstrapError("inspect post-HSA ledger directory"))?
            {
                None if !create_missing => {
                    retained.absent.insert(
                        (*component).to_string(),
                        RetainedDirectoryChildren::default(),
                    );
                    return Ok(None);
                }
                None => {
                    parent
                        .create_directory(component)
                        .map(drop)
                        .map_err(|_| BootstrapError("create post-HSA ledger directory"))?;
                }
                Some(EntryKind::Directory) => {}
                Some(EntryKind::RegularFile | EntryKind::Symlink | EntryKind::Other) => {
                    return Err(BootstrapError("post-HSA ledger directory is unsafe"));
                }
            }
            let entry = parent
                .entries()
                .map_err(|_| BootstrapError("enumerate post-HSA ledger parent"))?
                .into_iter()
                .find(|entry| entry.name == *component && entry.kind == EntryKind::Directory)
                .ok_or(BootstrapError(
                    "post-HSA ledger directory was not safely enumerated",
                ))?;
            let directory = parent
                .open_controlled_directory_entry(&entry)
                .map_err(|_| BootstrapError("retain post-HSA ledger directory"))?;
            retained.opened.insert(
                (*component).to_string(),
                RetainedDirectory {
                    entry,
                    directory,
                    children: RetainedDirectoryChildren::default(),
                },
            );
        }

        let child = retained
            .opened
            .get_mut(*component)
            .ok_or(BootstrapError("post-HSA ledger directory was not retained"))?;
        parent
            .open_controlled_directory_entry(&child.entry)
            .map(drop)
            .map_err(|_| BootstrapError("post-HSA ledger directory changed identity"))?;
        Self::with_child_directory(
            &child.directory,
            &mut child.children,
            remaining,
            create_missing,
            operation,
        )
    }

    fn revalidate(&self, root: &TrustedDirectory) -> Result<(), BootstrapError> {
        Self::revalidate_children(root, &self.children)
    }

    fn retain_enumerated_child(
        parent: &TrustedDirectory,
        retained: &mut RetainedDirectoryChildren,
        entry: DirectoryEntry,
    ) -> Result<(), BootstrapError> {
        if retained.absent.contains_key(&entry.name) {
            return Err(BootstrapError(
                "enumerated post-HSA ledger directory conflicts with retained absence",
            ));
        }
        if let Some(existing) = retained.opened.get(&entry.name) {
            if existing.entry != entry {
                return Err(BootstrapError(
                    "enumerated post-HSA ledger directory changed identity",
                ));
            }
            return Ok(());
        }
        let directory = parent
            .open_controlled_directory_entry(&entry)
            .map_err(|_| BootstrapError("retain enumerated post-HSA ledger directory"))?;
        retained.opened.insert(
            entry.name.clone(),
            RetainedDirectory {
                entry,
                directory,
                children: RetainedDirectoryChildren::default(),
            },
        );
        Ok(())
    }

    fn revalidate_children(
        parent: &TrustedDirectory,
        retained: &RetainedDirectoryChildren,
    ) -> Result<(), BootstrapError> {
        for absent in retained.absent.keys() {
            if parent
                .entry_kind(absent)
                .map_err(|_| BootstrapError("revalidate absent post-HSA ledger directory"))?
                .is_some()
            {
                return Err(BootstrapError(
                    "absent post-HSA ledger directory appeared during transaction",
                ));
            }
        }
        for child in retained.opened.values() {
            parent
                .open_controlled_directory_entry(&child.entry)
                .map(drop)
                .map_err(|_| BootstrapError("post-HSA ledger directory changed identity"))?;
            Self::revalidate_children(&child.directory, &child.children)?;
        }
        Ok(())
    }
}

impl RetainedLegacyDirectories {
    fn from_observation(
        root: &TrustedDirectory,
        observation: &LegacyObservation,
    ) -> Result<Self, BootstrapError> {
        let mut retained = Self::default();
        Self::retain_observed_collection(
            root,
            &mut retained.children,
            &observation.sessions.components,
            &observation.sessions.missing_suffix,
        )?;
        Self::retain_observed_collection(
            root,
            &mut retained.children,
            &observation.participants.components,
            &observation.participants.missing_suffix,
        )?;
        Ok(retained)
    }

    fn retain_observed_collection(
        parent: &TrustedDirectory,
        retained: &mut RetainedDirectoryChildren,
        observed: &[ObservedLegacyDirectory],
        missing_suffix: &[String],
    ) -> Result<(), BootstrapError> {
        let Some((expected, remaining)) = observed.split_first() else {
            if let Some(missing) = missing_suffix.first() {
                if retained.opened.contains_key(missing)
                    || parent
                        .entry_kind(missing)
                        .map_err(|_| {
                            BootstrapError("revalidate classified legacy StateStore absence")
                        })?
                        .is_some()
                {
                    return Err(BootstrapError(
                        "classified legacy StateStore directory changed during admission",
                    ));
                }
                Self::retain_classified_absence(retained, missing_suffix)?;
            }
            return Ok(());
        };

        if retained.absent.contains_key(&expected.entry.name) {
            return Err(BootstrapError(
                "classified legacy StateStore directory conflicts during admission",
            ));
        }
        if let Some(existing) = retained.opened.get(&expected.entry.name) {
            if existing.entry != expected.entry {
                return Err(BootstrapError(
                    "classified legacy StateStore directory identity is inconsistent",
                ));
            }
        } else {
            parent
                .revalidate_entry(&expected.entry)
                .map_err(|_| BootstrapError("revalidate classified legacy StateStore route"))?;
            let directory = parent
                .open_controlled_directory_entry(&expected.entry)
                .map_err(|_| BootstrapError("retain classified legacy StateStore route"))?;
            retained.opened.insert(
                expected.entry.name.clone(),
                RetainedDirectory {
                    entry: expected.entry.clone(),
                    directory,
                    children: RetainedDirectoryChildren::default(),
                },
            );
        }
        let child = retained
            .opened
            .get_mut(&expected.entry.name)
            .ok_or(BootstrapError(
                "classified legacy StateStore route was not retained",
            ))?;
        Self::retain_observed_collection(
            &child.directory,
            &mut child.children,
            remaining,
            missing_suffix,
        )
    }

    fn retain_classified_absence(
        retained: &mut RetainedDirectoryChildren,
        missing_suffix: &[String],
    ) -> Result<(), BootstrapError> {
        let Some((missing, remaining)) = missing_suffix.split_first() else {
            return Ok(());
        };
        if retained.opened.contains_key(missing) {
            return Err(BootstrapError(
                "classified legacy StateStore absence conflicts with opened route",
            ));
        }
        let child = retained.absent.entry(missing.clone()).or_default();
        Self::retain_classified_absence(child, remaining)
    }

    fn revalidate(&self, root: &TrustedDirectory) -> Result<(), BootstrapError> {
        Self::revalidate_children(root, &self.children)
    }

    fn revalidate_children(
        parent: &TrustedDirectory,
        retained: &RetainedDirectoryChildren,
    ) -> Result<(), BootstrapError> {
        for absent in retained.absent.keys() {
            if parent
                .entry_kind(absent)
                .map_err(|_| BootstrapError("revalidate retained legacy StateStore absence"))?
                .is_some()
            {
                return Err(BootstrapError(
                    "retained legacy StateStore directory appeared",
                ));
            }
        }
        for child in retained.opened.values() {
            parent
                .revalidate_entry(&child.entry)
                .and_then(|()| {
                    parent
                        .open_controlled_directory_entry(&child.entry)
                        .map(drop)
                })
                .map_err(|_| {
                    BootstrapError("retained legacy StateStore directory changed identity")
                })?;
            Self::revalidate_children(&child.directory, &child.children)?;
        }
        Ok(())
    }

    fn with_directory<T>(
        &mut self,
        root: &TrustedDirectory,
        components: &[&str],
        create_missing: bool,
        operation: impl FnOnce(
            &TrustedDirectory,
            &mut RetainedDirectoryChildren,
        ) -> Result<T, BootstrapError>,
    ) -> Result<Option<T>, BootstrapError> {
        Self::with_child_directory(
            root,
            &mut self.children,
            components,
            create_missing,
            operation,
        )
    }

    fn with_child_directory<T>(
        parent: &TrustedDirectory,
        retained: &mut RetainedDirectoryChildren,
        components: &[&str],
        create_missing: bool,
        operation: impl FnOnce(
            &TrustedDirectory,
            &mut RetainedDirectoryChildren,
        ) -> Result<T, BootstrapError>,
    ) -> Result<Option<T>, BootstrapError> {
        let Some((component, remaining)) = components.split_first() else {
            return operation(parent, retained).map(Some);
        };

        if retained.absent.contains_key(*component) {
            if parent
                .entry_kind(component)
                .map_err(|_| BootstrapError("revalidate absent legacy StateStore directory"))?
                .is_some()
            {
                return Err(BootstrapError(
                    "legacy StateStore directory appeared after transaction admission",
                ));
            }
            if !create_missing {
                return Ok(None);
            }
            let (entry, directory) = parent
                .create_directory_entry_exclusive(component)
                .map_err(|_| BootstrapError("create retained legacy StateStore directory"))?;
            let classified_children = retained.absent.remove(*component).ok_or(BootstrapError(
                "classified legacy StateStore absence was not retained",
            ))?;
            retained.opened.insert(
                (*component).to_owned(),
                RetainedDirectory {
                    entry,
                    directory,
                    children: classified_children,
                },
            );
        } else if !retained.opened.contains_key(*component) {
            match parent
                .entry_kind(component)
                .map_err(|_| BootstrapError("inspect legacy StateStore directory route"))?
            {
                None => {
                    retained.absent.insert(
                        (*component).to_owned(),
                        RetainedDirectoryChildren::default(),
                    );
                    if !create_missing {
                        return Ok(None);
                    }
                    let (entry, directory) = parent
                        .create_directory_entry_exclusive(component)
                        .map_err(|_| {
                            BootstrapError("create retained legacy StateStore directory")
                        })?;
                    let dynamic_children = retained.absent.remove(*component).ok_or(
                        BootstrapError("dynamic legacy StateStore absence was not retained"),
                    )?;
                    retained.opened.insert(
                        (*component).to_owned(),
                        RetainedDirectory {
                            entry,
                            directory,
                            children: dynamic_children,
                        },
                    );
                }
                Some(EntryKind::Directory) => {
                    let entry = Self::directory_entry(parent, component)?;
                    let directory = parent
                        .open_controlled_directory_entry(&entry)
                        .map_err(|_| BootstrapError("open retained legacy StateStore directory"))?;
                    retained.opened.insert(
                        (*component).to_owned(),
                        RetainedDirectory {
                            entry,
                            directory,
                            children: RetainedDirectoryChildren::default(),
                        },
                    );
                }
                Some(_) => {
                    return Err(BootstrapError(
                        "legacy StateStore directory route is unsafe",
                    ))
                }
            }
        }

        let child = retained.opened.get_mut(*component).ok_or(BootstrapError(
            "retained legacy StateStore directory is unavailable",
        ))?;
        parent
            .revalidate_entry(&child.entry)
            .and_then(|()| {
                parent
                    .open_controlled_directory_entry(&child.entry)
                    .map(drop)
            })
            .map_err(|_| BootstrapError("retained legacy StateStore directory changed identity"))?;
        let result = Self::with_child_directory(
            &child.directory,
            &mut child.children,
            remaining,
            create_missing,
            operation,
        )?;
        parent
            .revalidate_entry(&child.entry)
            .and_then(|()| {
                parent
                    .open_controlled_directory_entry(&child.entry)
                    .map(drop)
            })
            .map_err(|_| {
                BootstrapError("retained legacy StateStore directory changed during operation")
            })?;
        Ok(result)
    }

    fn directory_entry(
        parent: &TrustedDirectory,
        name: &str,
    ) -> Result<DirectoryEntry, BootstrapError> {
        parent
            .entries()
            .map_err(|_| BootstrapError("enumerate retained legacy StateStore directory"))?
            .into_iter()
            .find(|entry| entry.name == name && entry.kind == EntryKind::Directory)
            .ok_or(BootstrapError(
                "retained legacy StateStore directory was not safely enumerated",
            ))
    }

    fn retain_enumerated_directory(
        parent: &TrustedDirectory,
        retained: &mut RetainedDirectoryChildren,
        entry: &DirectoryEntry,
    ) -> Result<(), BootstrapError> {
        if retained.absent.contains_key(&entry.name) {
            return Err(BootstrapError(
                "enumerated legacy StateStore directory conflicts with retained absence",
            ));
        }
        if let Some(existing) = retained.opened.get(&entry.name) {
            return if existing.entry == *entry {
                Ok(())
            } else {
                Err(BootstrapError(
                    "enumerated legacy StateStore directory changed identity",
                ))
            };
        }
        let directory = parent
            .open_controlled_directory_entry(entry)
            .map_err(|_| BootstrapError("retain enumerated legacy StateStore directory"))?;
        retained.opened.insert(
            entry.name.clone(),
            RetainedDirectory {
                entry: entry.clone(),
                directory,
                children: RetainedDirectoryChildren::default(),
            },
        );
        Ok(())
    }
}

#[cfg(test)]
pub(super) fn retain_classified_legacy_directories_test(
    root: &TrustedDirectory,
    observation: &LegacyObservation,
) -> Result<(), BootstrapError> {
    RetainedLegacyDirectories::from_observation(root, observation).map(drop)
}

pub(super) struct SemanticTransaction<'layout, 'root> {
    pub(super) layout: &'layout StoreLayout<'root>,
    pub(super) trusted_root: &'root TrustedAuthorityRoot,
    pub(super) root: StateRootV1,
    pub(super) legacy: LegacyObservation,
}

pub(super) struct VersionedSemanticTransaction<'layout, 'root> {
    pub(super) layout: &'layout StoreLayout<'root>,
    pub(super) trusted_root: &'root TrustedAuthorityRoot,
    pub(super) root: VersionedStateRoot,
    pub(super) legacy: LegacyObservation,
}

#[derive(Clone, Copy)]
pub(super) enum SemanticPreflightMode {
    AuthorityOperation,
    LegacyWriter,
}

impl SemanticTransaction<'_, '_> {
    pub(super) fn require_expected_root(
        &self,
        expected_root_revision: u64,
    ) -> Result<(), BootstrapError> {
        if self.root.root_revision == expected_root_revision {
            Ok(())
        } else {
            Err(BootstrapError("stale authority root revision"))
        }
    }

    pub(super) fn reconcile(&self) -> Result<(), BootstrapError> {
        self.layout
            .reconcile_after_preflight(&self.root)
            .map_err(|_| BootstrapError("reconcile authority store after semantic preflight"))
    }

    pub(super) fn validate_publication_candidate(
        &self,
        expected_root_revision: u64,
        candidate: &StateRootV1,
    ) -> Result<(), BootstrapError> {
        self.trusted_root
            .revalidate()
            .map_err(|_| BootstrapError("revalidate trusted root before candidate validation"))?;
        self.require_expected_root(expected_root_revision)?;
        let current = self
            .layout
            .read_existing_without_reconciliation(&self.root.bootstrap_home)
            .map_err(|_| BootstrapError("reread locked root before candidate validation"))?;
        if current != self.root {
            return Err(BootstrapError(
                "locked authority root changed before candidate validation",
            ));
        }
        let next_revision = expected_root_revision
            .checked_add(1)
            .ok_or(BootstrapError("expected root revision overflow"))?;
        if candidate.root_revision != next_revision
            || candidate.authority_store_id != current.authority_store_id
            || candidate.bootstrap_home != current.bootstrap_home
            || candidate.greenfield_namespace_certificate
                != current.greenfield_namespace_certificate
        {
            return Err(BootstrapError(
                "publication candidate identity or revision is invalid",
            ));
        }
        self.layout
            .validate_root_candidate(candidate)
            .map_err(|_| BootstrapError("validate reconciled publication candidate"))?;
        self.legacy
            .revalidate(self.layout.bootstrap)
            .map_err(|_| BootstrapError("revalidate legacy state for publication candidate"))?;
        self.trusted_root
            .revalidate()
            .map_err(|_| BootstrapError("revalidate trusted root after candidate validation"))?;
        let current = self
            .layout
            .read_existing_without_reconciliation(&self.root.bootstrap_home)
            .map_err(|_| BootstrapError("reread locked root after candidate validation"))?;
        if current != self.root {
            return Err(BootstrapError(
                "locked authority root changed during candidate validation",
            ));
        }
        self.layout
            .validate_root_candidate(candidate)
            .map_err(|_| BootstrapError("revalidate reconciled publication candidate"))
    }

    fn validate_exact_committed_candidate(
        &self,
        expected_root_revision: u64,
        candidate: &StateRootV1,
    ) -> Result<(), BootstrapError> {
        self.trusted_root
            .revalidate()
            .map_err(|_| BootstrapError("revalidate trusted root for exact retry"))?;
        let next_revision = expected_root_revision
            .checked_add(1)
            .ok_or(BootstrapError("expected root revision overflow"))?;
        if candidate.root_revision != next_revision || self.root != *candidate {
            return Err(BootstrapError("exact retry candidate is not current root"));
        }
        let current = self
            .layout
            .read_existing_without_reconciliation(&self.root.bootstrap_home)
            .map_err(|_| BootstrapError("reread locked root for exact retry"))?;
        if current != self.root {
            return Err(BootstrapError(
                "locked authority root changed during exact retry",
            ));
        }
        self.layout
            .validate_root_candidate(candidate)
            .map_err(|_| BootstrapError("validate exact committed authority root"))?;
        self.legacy
            .revalidate(self.layout.bootstrap)
            .map_err(|_| BootstrapError("revalidate legacy state for exact retry"))?;
        self.trusted_root
            .revalidate()
            .map_err(|_| BootstrapError("revalidate trusted root after exact retry"))?;
        let current = self
            .layout
            .read_existing_without_reconciliation(&self.root.bootstrap_home)
            .map_err(|_| BootstrapError("reread locked root after exact retry"))?;
        if current != self.root {
            return Err(BootstrapError(
                "locked authority root changed after exact retry",
            ));
        }
        self.layout
            .validate_root_candidate(candidate)
            .map_err(|_| BootstrapError("revalidate exact committed authority root"))
    }
}

impl VersionedSemanticTransaction<'_, '_> {
    pub(super) fn require_expected_root(
        &self,
        expected_root_revision: u64,
    ) -> Result<(), BootstrapError> {
        if self.root.root_revision() == expected_root_revision {
            Ok(())
        } else {
            Err(BootstrapError("stale authority root revision"))
        }
    }

    pub(super) fn reconcile(&self) -> Result<(), BootstrapError> {
        match &self.root {
            VersionedStateRoot::V1(root) => self.layout.reconcile_after_preflight(root),
            VersionedStateRoot::V2(root) => self.layout.reconcile_after_preflight_v2(root),
            VersionedStateRoot::V3(root) => self.layout.reconcile_after_preflight_v3(root),
        }
        .map_err(|_| BootstrapError("reconcile versioned authority store after preflight"))
    }

    pub(super) fn validate_publication_candidate(
        &self,
        expected_root_revision: u64,
        candidate: &VersionedStateRoot,
    ) -> Result<(), BootstrapError> {
        self.trusted_root
            .revalidate()
            .map_err(|_| BootstrapError("revalidate trusted root for versioned candidate"))?;
        self.require_expected_root(expected_root_revision)?;
        let current = self
            .layout
            .read_existing_versioned_without_reconciliation(self.root.bootstrap_home())
            .map_err(|_| BootstrapError("reread locked versioned root"))?;
        if current != self.root {
            return Err(BootstrapError("locked versioned authority root changed"));
        }
        let next_revision = expected_root_revision
            .checked_add(1)
            .ok_or(BootstrapError("expected versioned root revision overflow"))?;
        if candidate.root_revision() != next_revision
            || candidate.authority_store_id() != current.authority_store_id()
            || candidate.bootstrap_home() != current.bootstrap_home()
            || candidate.greenfield_namespace_certificate()
                != current.greenfield_namespace_certificate()
            || !matches!(
                (&current, candidate),
                (VersionedStateRoot::V1(_), VersionedStateRoot::V1(_))
                    | (VersionedStateRoot::V2(_), VersionedStateRoot::V2(_))
                    | (VersionedStateRoot::V2(_), VersionedStateRoot::V3(_))
                    | (VersionedStateRoot::V3(_), VersionedStateRoot::V3(_))
            )
        {
            return Err(BootstrapError(
                "versioned publication candidate identity or revision is invalid",
            ));
        }
        validate_versioned_candidate(self.layout, candidate)?;
        self.legacy
            .revalidate(self.layout.bootstrap)
            .map_err(|_| BootstrapError("revalidate legacy state for versioned candidate"))?;
        self.trusted_root
            .revalidate()
            .map_err(|_| BootstrapError("revalidate trusted root after versioned candidate"))?;
        let current = self
            .layout
            .read_existing_versioned_without_reconciliation(self.root.bootstrap_home())
            .map_err(|_| BootstrapError("reread locked versioned root after validation"))?;
        if current != self.root {
            return Err(BootstrapError(
                "locked versioned authority root changed during validation",
            ));
        }
        validate_versioned_candidate(self.layout, candidate)
    }
}

fn validate_versioned_candidate(
    layout: &StoreLayout<'_>,
    candidate: &VersionedStateRoot,
) -> Result<(), BootstrapError> {
    match candidate {
        VersionedStateRoot::V1(root) => layout.validate_root_candidate(root),
        VersionedStateRoot::V2(root) => layout.validate_root_candidate_v2(root),
        VersionedStateRoot::V3(root) => layout.validate_root_candidate_v3(root),
    }
    .map_err(|_| BootstrapError("validate versioned publication candidate"))
}

impl LegacyStateStoreTransactionV1 {
    #[cfg(test)]
    pub(crate) fn create_classified_run_directory_test(&mut self) -> Result<(), BootstrapError> {
        self.retained_directories
            .with_directory(self.root.directory(), &["run"], true, |_, _| Ok(()))?
            .ok_or(BootstrapError(
                "classified legacy StateStore run directory was not created",
            ))
    }

    pub(crate) fn read_file(
        &mut self,
        collection: LegacyStateStoreCollectionV1,
        descendants: &[&str],
    ) -> Result<Option<Vec<u8>>, BootstrapError> {
        self.verify_retained_root()?;
        let bytes = Self::with_parent(
            &self.root,
            &mut self.retained_directories,
            collection,
            descendants,
            false,
            |parent, target| match parent
                .entry_kind(target)
                .map_err(|_| BootstrapError("inspect legacy StateStore file"))?
            {
                None => Ok(None),
                Some(EntryKind::RegularFile) => parent
                    .open_file(target)
                    .and_then(|file| file.read_all())
                    .map(Some)
                    .map_err(|_| BootstrapError("read legacy StateStore file")),
                Some(_) => Err(BootstrapError("legacy StateStore file is unsafe")),
            },
        )?
        .flatten();
        self.verify_retained_root()?;
        Ok(bytes)
    }

    pub(crate) fn write_file(
        &mut self,
        collection: LegacyStateStoreCollectionV1,
        descendants: &[&str],
        bytes: &[u8],
        nonce_bytes: [u8; 16],
    ) -> Result<(), BootstrapError> {
        self.verify_retained_root()?;
        let root = &self.root;
        Self::with_parent(
            root,
            &mut self.retained_directories,
            collection,
            descendants,
            true,
            |parent, target| {
                match parent
                    .entry_kind(target)
                    .map_err(|_| BootstrapError("inspect legacy StateStore publication target"))?
                {
                    None | Some(EntryKind::RegularFile) => {}
                    Some(_) => {
                        return Err(BootstrapError(
                            "legacy StateStore publication target is unsafe",
                        ))
                    }
                }
                let temp_name = format!("legacy--{}.tmp", nonce(nonce_bytes));
                let mut temp = parent
                    .create_file(&temp_name)
                    .map_err(|_| BootstrapError("create legacy StateStore temp"))?;
                if temp.write_all(bytes).and_then(|()| temp.sync()).is_err() {
                    let _ = parent.unlink_file(&temp_name);
                    return Err(BootstrapError("write legacy StateStore temp"));
                }
                if root.revalidate().is_err() {
                    let _ = parent.unlink_file(&temp_name);
                    return Err(BootstrapError(
                        "trusted authority root changed before legacy publication",
                    ));
                }
                if parent
                    .rename_replace(&temp_name, temp, parent, target)
                    .is_err()
                {
                    let _ = parent.unlink_file(&temp_name);
                    return Err(BootstrapError("publish legacy StateStore file"));
                }
                Ok(())
            },
        )?
        .ok_or(BootstrapError("legacy StateStore parent was not opened"))?;
        self.verify_retained_root()
    }

    pub(crate) fn remove_file(
        &mut self,
        collection: LegacyStateStoreCollectionV1,
        descendants: &[&str],
    ) -> Result<bool, BootstrapError> {
        self.verify_retained_root()?;
        let removed = Self::with_parent(
            &self.root,
            &mut self.retained_directories,
            collection,
            descendants,
            false,
            |parent, target| match parent
                .entry_kind(target)
                .map_err(|_| BootstrapError("inspect legacy StateStore removal target"))?
            {
                None => Ok(false),
                Some(EntryKind::RegularFile) => parent
                    .unlink_file(target)
                    .map(|()| true)
                    .map_err(|_| BootstrapError("remove legacy StateStore file")),
                Some(_) => Err(BootstrapError("legacy StateStore removal target is unsafe")),
            },
        )?
        .unwrap_or(false);
        self.verify_retained_root()?;
        Ok(removed)
    }

    pub(crate) fn read_directory(
        &mut self,
        collection: LegacyStateStoreCollectionV1,
        descendants: &[&str],
    ) -> Result<Vec<LegacyStateStoreDirectoryEntryV1>, BootstrapError> {
        self.verify_retained_root()?;
        let prefix: &[&str] = match collection {
            LegacyStateStoreCollectionV1::Sessions => &["run", "agent-hub", "sessions"],
            LegacyStateStoreCollectionV1::Participants => &["run", "agent-hub", "participants"],
            LegacyStateStoreCollectionV1::Handles => &["run", "agent-hub", "handles"],
            LegacyStateStoreCollectionV1::HostInbox => &["host_inbox"],
        };
        let components = prefix
            .iter()
            .chain(descendants.iter())
            .copied()
            .collect::<Vec<_>>();
        let entries = self
            .retained_directories
            .with_directory(
                self.root.directory(),
                &components,
                false,
                |directory, retained| {
                    let scanned = directory
                        .entries()
                        .map_err(|_| BootstrapError("enumerate legacy StateStore directory"))?;
                    let mut entries = Vec::with_capacity(scanned.len());
                    for entry in scanned {
                        let bytes = match entry.kind {
                            EntryKind::Directory => {
                                RetainedLegacyDirectories::retain_enumerated_directory(
                                    directory, retained, &entry,
                                )?;
                                None
                            }
                            EntryKind::RegularFile => Some(
                                directory
                                    .open_file_entry(&entry)
                                    .and_then(|file| file.read_all())
                                    .map_err(|_| {
                                        BootstrapError("read legacy StateStore file entry")
                                    })?,
                            ),
                            EntryKind::Symlink | EntryKind::Other => {
                                return Err(BootstrapError(
                                    "legacy StateStore directory entry is unsafe",
                                ))
                            }
                        };
                        entries.push(LegacyStateStoreDirectoryEntryV1 {
                            name: entry.name,
                            is_directory: entry.kind == EntryKind::Directory,
                            bytes,
                        });
                    }
                    Ok(entries)
                },
            )?
            .unwrap_or_default();
        self.verify_retained_root()?;
        Ok(entries)
    }

    pub(crate) fn verify_physical_root(
        &self,
        expected: &crate::execution::agent_runtime::host_session_authority::schema::CanonicalDirectoryV1,
    ) -> Result<(), BootstrapError> {
        if self.root.identity() != expected {
            return Err(BootstrapError(
                "legacy transaction physical root identity mismatch",
            ));
        }
        self.verify_retained_root()
    }

    pub(crate) fn finish(self) -> Result<(), BootstrapError> {
        self.verify_retained_root()?;
        self.retained_directories
            .revalidate(self.root.directory())?;
        self.root
            .directory()
            .sync()
            .map_err(|_| BootstrapError("sync retained legacy StateStore root"))?;
        self.retained_directories
            .revalidate(self.root.directory())?;
        self.verify_retained_root()
    }

    fn verify_retained_root(&self) -> Result<(), BootstrapError> {
        self.root
            .revalidate()
            .map_err(|_| BootstrapError("trusted authority root was rebound or replaced"))
    }

    fn with_parent<T>(
        root: &TrustedAuthorityRoot,
        retained_directories: &mut RetainedLegacyDirectories,
        collection: LegacyStateStoreCollectionV1,
        descendants: &[&str],
        create_missing: bool,
        operation: impl FnOnce(&TrustedDirectory, &str) -> Result<T, BootstrapError>,
    ) -> Result<Option<T>, BootstrapError> {
        let (target, relative_directories) = descendants
            .split_last()
            .ok_or(BootstrapError("legacy StateStore path is empty"))?;
        let prefix: &[&str] = match collection {
            LegacyStateStoreCollectionV1::Sessions => &["run", "agent-hub", "sessions"],
            LegacyStateStoreCollectionV1::Participants => &["run", "agent-hub", "participants"],
            LegacyStateStoreCollectionV1::Handles => &["run", "agent-hub", "handles"],
            LegacyStateStoreCollectionV1::HostInbox => &["host_inbox"],
        };
        let components = prefix
            .iter()
            .chain(relative_directories.iter())
            .copied()
            .collect::<Vec<_>>();
        retained_directories.with_directory(
            root.directory(),
            &components,
            create_missing,
            |parent, _| operation(parent, target),
        )
    }
}

impl WorldWorkReceiptRegistryTransactionV1<'_> {
    #[cfg(target_os = "linux")]
    pub(crate) fn verify_ordinary_world_publication(
        &self,
        publication: &crate::execution::agent_runtime::host_session_authority::world_bound_controller::AuthenticatedOrdinaryWorldPublicationV1,
    ) -> Result<(), BootstrapError> {
        self.verify_scope()?;
        publication.verify_locked_ordinary_world_publication(&self.locked_root)?;
        self.verify_scope()
    }
    pub(crate) fn verify_post_hsa_authority_binding(
        &mut self,
        expected_observation: &crate::execution::agent_runtime::host_session_authority::facade::AuthorityObservationV1,
        expected_router_auto_attach_intent: Option<
            &crate::execution::agent_runtime::host_session_authority::store_schema::HostSessionTransitionIntentV3,
        >,
    ) -> Result<(), BootstrapError> {
        self.verify_scope()?;
        crate::execution::agent_runtime::host_session_authority::facade::validate_locked_post_hsa_authority_binding(
            &self.locked_root,
            expected_observation,
            expected_router_auto_attach_intent,
        )
        .map_err(|_| BootstrapError("validate locked post-HSA authority binding"))?;
        self.verify_scope()
    }

    pub(crate) fn read_post_hsa_obligation_ledger_file(
        &mut self,
        expected_root_revision: u64,
        orchestration_session_id: &str,
        relative_path: &[&str],
    ) -> Result<Option<Vec<u8>>, BootstrapError> {
        self.verify_post_hsa_obligation_ledger_scope(expected_root_revision)?;
        let (target, directories) = relative_path
            .split_last()
            .ok_or(BootstrapError("post-HSA obligation ledger path is empty"))?;
        validate_post_hsa_obligation_ledger_component(target)?;
        let bytes = self
            .with_post_hsa_obligation_ledger_directory(
                orchestration_session_id,
                directories,
                false,
                |parent, _| read_exact_regular_file(parent, target),
            )?
            .flatten();
        self.verify_post_hsa_obligation_ledger_scope(expected_root_revision)?;
        Ok(bytes)
    }

    pub(crate) fn list_post_hsa_obligation_ledger_files(
        &mut self,
        expected_root_revision: u64,
        orchestration_session_id: &str,
        relative_directory: &[&str],
    ) -> Result<Vec<(String, Vec<u8>)>, BootstrapError> {
        self.verify_post_hsa_obligation_ledger_scope(expected_root_revision)?;
        let files = self
            .with_post_hsa_obligation_ledger_directory(
                orchestration_session_id,
                relative_directory,
                false,
                |directory, _| {
                    reconcile_post_hsa_obligation_ledger_temps(directory)?;
                    let entries = directory.entries().map_err(|_| {
                        BootstrapError("enumerate post-HSA obligation ledger directory")
                    })?;
                    let mut files = Vec::new();
                    for entry in entries {
                        if entry.kind != EntryKind::RegularFile || !entry.name.ends_with(".json") {
                            return Err(BootstrapError(
                                "post-HSA obligation ledger directory contains an unsafe entry",
                            ));
                        }
                        let bytes = directory
                            .open_file_entry(&entry)
                            .and_then(|file| file.read_all())
                            .and_then(|bytes| directory.revalidate_entry(&entry).map(|()| bytes))
                            .map_err(|_| {
                                BootstrapError("read post-HSA obligation ledger artifact")
                            })?;
                        files.push((entry.name, bytes));
                    }
                    files.sort_by(|left, right| left.0.cmp(&right.0));
                    Ok(files)
                },
            )?
            .unwrap_or_default();
        self.verify_post_hsa_obligation_ledger_scope(expected_root_revision)?;
        Ok(files)
    }

    pub(crate) fn list_post_hsa_obligation_ledger_directories(
        &mut self,
        expected_root_revision: u64,
        orchestration_session_id: &str,
        relative_directory: &[&str],
    ) -> Result<Vec<String>, BootstrapError> {
        self.verify_post_hsa_obligation_ledger_scope(expected_root_revision)?;
        let names = self
            .with_post_hsa_obligation_ledger_directory(
                orchestration_session_id,
                relative_directory,
                false,
                |directory, retained_children| {
                    let mut names = Vec::new();
                    for entry in directory.entries().map_err(|_| {
                        BootstrapError("enumerate post-HSA obligation ledger directories")
                    })? {
                        if entry.kind != EntryKind::Directory {
                            return Err(BootstrapError(
                                "post-HSA obligation ledger collection contains a non-directory entry",
                            ));
                        }
                        let name = entry.name.clone();
                        RetainedPostHsaLedgerDirectories::retain_enumerated_child(
                            directory,
                            retained_children,
                            entry,
                        )?;
                        names.push(name);
                    }
                    names.sort();
                    Ok(names)
                },
            )?
            .unwrap_or_default();
        self.verify_post_hsa_obligation_ledger_scope(expected_root_revision)?;
        Ok(names)
    }

    pub(crate) fn replace_post_hsa_obligation_ledger_file(
        &mut self,
        expected_root_revision: u64,
        orchestration_session_id: &str,
        relative_path: &[&str],
        expected_bytes: &[u8],
        replacement_bytes: &[u8],
    ) -> Result<(), BootstrapError> {
        self.verify_post_hsa_obligation_ledger_scope(expected_root_revision)?;
        let (target, directories) = relative_path
            .split_last()
            .ok_or(BootstrapError("post-HSA obligation ledger path is empty"))?;
        validate_post_hsa_obligation_ledger_component(target)?;
        self.with_post_hsa_obligation_ledger_directory(
            orchestration_session_id,
            directories,
            false,
            |parent, _| {
                reconcile_post_hsa_obligation_ledger_temps(parent)?;
                let current = read_exact_regular_file(parent, target)?.ok_or(BootstrapError(
                    "expected post-HSA obligation ledger artifact is absent",
                ))?;
                if current != expected_bytes {
                    return Err(BootstrapError(
                        "stale or conflicting post-HSA obligation ledger artifact",
                    ));
                }
                publish_post_hsa_obligation_ledger_file(parent, target, replacement_bytes, true)
            },
        )?
        .ok_or(BootstrapError(
            "expected post-HSA obligation ledger parent is absent",
        ))?;
        self.root
            .directory()
            .sync()
            .map_err(|_| BootstrapError("sync post-HSA obligation ledger root"))?;
        self.verify_post_hsa_obligation_ledger_scope(expected_root_revision)
    }

    pub(crate) fn publish_post_hsa_obligation_ledger_file(
        &mut self,
        expected_root_revision: u64,
        orchestration_session_id: &str,
        relative_path: &[&str],
        bytes: &[u8],
    ) -> Result<(), BootstrapError> {
        self.verify_post_hsa_obligation_ledger_scope(expected_root_revision)?;
        let (target, directories) = relative_path
            .split_last()
            .ok_or(BootstrapError("post-HSA obligation ledger path is empty"))?;
        validate_post_hsa_obligation_ledger_component(target)?;
        self.with_post_hsa_obligation_ledger_directory(
            orchestration_session_id,
            directories,
            true,
            |parent, _| {
                reconcile_post_hsa_obligation_ledger_temps(parent)?;
                if read_exact_regular_file(parent, target)?.is_some() {
                    return Err(BootstrapError(
                        "post-HSA obligation ledger publication target already exists",
                    ));
                }
                publish_post_hsa_obligation_ledger_file(parent, target, bytes, false)
            },
        )?
        .ok_or(BootstrapError("create post-HSA obligation ledger parent"))?;
        self.root
            .directory()
            .sync()
            .map_err(|_| BootstrapError("sync post-HSA obligation ledger root"))?;
        self.verify_post_hsa_obligation_ledger_scope(expected_root_revision)
    }

    fn with_post_hsa_obligation_ledger_directory<T>(
        &mut self,
        orchestration_session_id: &str,
        relative_directory: &[&str],
        create_missing: bool,
        operation: impl FnOnce(
            &TrustedDirectory,
            &mut RetainedDirectoryChildren,
        ) -> Result<T, BootstrapError>,
    ) -> Result<Option<T>, BootstrapError> {
        validate_post_hsa_obligation_ledger_component(orchestration_session_id)?;
        let base = [
            POST_HSA_OBLIGATION_LEDGER_DIRECTORY,
            orchestration_session_id,
        ];
        if self
            .retained_post_hsa_ledger_directories
            .with_directory(self.root.directory(), &base, false, |_, _| Ok(()))?
            .is_none()
        {
            return Ok(None);
        }
        let mut components = base.to_vec();
        components.extend_from_slice(relative_directory);
        self.retained_post_hsa_ledger_directories.with_directory(
            self.root.directory(),
            &components,
            create_missing,
            operation,
        )
    }

    fn verify_post_hsa_obligation_ledger_scope(
        &self,
        expected_root_revision: u64,
    ) -> Result<(), BootstrapError> {
        self.verify_scope()?;
        match &self.locked_root {
            VersionedStateRoot::V2(root) if root.root_revision == expected_root_revision => Ok(()),
            VersionedStateRoot::V3(root) if root.root_revision == expected_root_revision => Ok(()),
            VersionedStateRoot::V1(_) => Err(BootstrapError(
                "post-HSA obligation ledger mutation requires strict StateRootV2 or StateRootV3",
            )),
            VersionedStateRoot::V2(_) | VersionedStateRoot::V3(_) => Err(BootstrapError(
                "stale post-HSA obligation ledger authority observation",
            )),
        }
    }

    /// Storage-version classification under the retained root lock. This does
    /// not grant materialization or execution permission. V1's existing producer
    /// can remain serialized without entering the later post-HSA file protocol.
    #[cfg(target_os = "linux")]
    pub(crate) fn post_hsa_materialization_root_revision(
        &self,
        expected_home: &Path,
    ) -> Result<Option<u64>, BootstrapError> {
        self.verify_scope()?;
        if expected_home != Path::new(&self.root.identity().physical_path) {
            return Err(BootstrapError(
                "canonical materializer store differs from retained B1 home",
            ));
        }
        Ok(match &self.locked_root {
            VersionedStateRoot::V1(_) => None,
            VersionedStateRoot::V2(root) => Some(root.root_revision),
            VersionedStateRoot::V3(root) => Some(root.root_revision),
        })
    }

    /// Explicit write-side creation for the accepted-work materializer. Readers,
    /// claims and settlements still require an existing canonical ledger. The
    /// producer must authenticate its exact B1 acceptance before calling this.
    #[cfg(target_os = "linux")]
    pub(crate) fn initialize_post_hsa_materialization_session(
        &mut self,
        expected_root_revision: u64,
        orchestration_session_id: &str,
    ) -> Result<(), BootstrapError> {
        self.verify_post_hsa_obligation_ledger_scope(expected_root_revision)?;
        validate_post_hsa_obligation_ledger_component(orchestration_session_id)?;
        self.retained_post_hsa_ledger_directories
            .with_directory(
                self.root.directory(),
                &[
                    POST_HSA_OBLIGATION_LEDGER_DIRECTORY,
                    orchestration_session_id,
                ],
                true,
                |_, _| Ok(()),
            )?
            .ok_or(BootstrapError(
                "create accepted-work materialization session",
            ))?;
        self.verify_post_hsa_obligation_ledger_scope(expected_root_revision)
    }

    pub(crate) fn read_registry(&mut self) -> Result<Option<Vec<u8>>, BootstrapError> {
        self.verify_scope()?;
        validate_world_work_receipt_registry_namespace(&self.agent_hub)?;
        let bytes = read_world_work_receipt_registry(&self.agent_hub)?;
        self.verify_scope()?;
        Ok(bytes)
    }

    #[cfg(target_os = "linux")]
    pub(crate) fn replace_registry_for_ordinary_retained_targets(
        &self,
        bytes: &[u8],
        targets: &[(String, String)],
    ) -> Result<(), BootstrapError> {
        with_existing_retained_admission_source_in_locked_authority(
            &self.authority,
            |admission, keys, verify| {
                for (session, target) in targets {
                    crate::execution::agent_runtime::retained_worker_runtime::require_ordinary_retained_target_open_from_sources(
                    &self.locked_root, admission, &keys, session, target)
                    .map_err(|_| BootstrapError("ordinary_retained_stop_pending_or_closed: new retained work refused"))?;
                }
                self.replace_registry_with_source_guard(bytes, verify)
            },
        )
    }

    pub(crate) fn replace_registry(&mut self, bytes: &[u8]) -> Result<(), BootstrapError> {
        self.replace_registry_with_source_guard(bytes, &|| Ok(()))
    }

    fn replace_registry_with_source_guard(
        &self,
        bytes: &[u8],
        verify_sources: &dyn Fn() -> Result<(), BootstrapError>,
    ) -> Result<(), BootstrapError> {
        use rand::RngCore as _;

        self.verify_scope()?;
        verify_sources()?;
        validate_world_work_receipt_registry_namespace(&self.agent_hub)?;
        let mut nonce_bytes = [0_u8; 16];
        rand::rngs::OsRng.fill_bytes(&mut nonce_bytes);
        let temp_name = format!(
            "{WORLD_WORK_RECEIPT_REGISTRY_TEMP_PREFIX}{}{WORLD_WORK_RECEIPT_REGISTRY_TEMP_SUFFIX}",
            nonce(nonce_bytes)
        );
        let mut temp = self
            .agent_hub
            .create_file(&temp_name)
            .map_err(|_| BootstrapError("create B1 receipt-registry temp"))?;
        let staged = (|| {
            temp.write_all(bytes)
                .map_err(|_| BootstrapError("write B1 receipt-registry temp"))?;
            temp.sync()
                .map_err(|_| BootstrapError("sync B1 receipt-registry temp"))?;
            self.verify_scope()?;
            verify_sources()
        })();
        if let Err(error) = staged {
            self.agent_hub
                .unlink_owned_file(&temp_name, temp)
                .map_err(|_| BootstrapError("B1 owned candidate cleanup unresolved"))?;
            return Err(error);
        }
        self.agent_hub
            .rename_replace(
                &temp_name,
                temp,
                &self.agent_hub,
                WORLD_WORK_RECEIPT_REGISTRY_FILE,
            )
            .map_err(|_| BootstrapError("publish B1 receipt registry"))?;
        self.agent_hub
            .sync()
            .and_then(|()| self.run.sync())
            .and_then(|()| self.root.directory().sync())
            .map_err(|_| BootstrapError("sync B1 receipt-registry publication path"))?;
        let published = read_world_work_receipt_registry(&self.agent_hub)?
            .ok_or(BootstrapError("published B1 receipt registry is absent"))?;
        if published != bytes {
            return Err(BootstrapError(
                "published B1 receipt-registry bytes changed",
            ));
        }
        self.verify_scope()
    }

    pub(crate) fn finish(self) -> Result<(), BootstrapError> {
        self.verify_scope()?;
        validate_world_work_receipt_registry_namespace(&self.agent_hub)?;
        self.agent_hub
            .sync()
            .and_then(|()| self.run.sync())
            .and_then(|()| self.root.directory().sync())
            .map_err(|_| BootstrapError("sync B1 receipt-registry transaction"))?;
        self.verify_scope()
    }

    fn verify_scope(&self) -> Result<(), BootstrapError> {
        self.verify_named_root_lock_scope()?;
        let layout = StoreLayout::open(self.root.directory())
            .map_err(|_| BootstrapError("open B1 receipt authority layout"))?;
        layout
            .validate_closed_layout()
            .map_err(|_| BootstrapError("revalidate B1 receipt authority layout"))?;
        layout
            .validate_temps()
            .map_err(|_| BootstrapError("revalidate B1 receipt authority temps"))?;
        let legacy = LegacyObservation::capture(layout.bootstrap)
            .map_err(|_| BootstrapError("recapture B1 receipt authority legacy state"))?;
        if legacy.has_artifact {
            return Err(BootstrapError(
                "legacy state appeared during B1 receipt transaction",
            ));
        }
        let current = layout
            .read_existing_versioned_without_reconciliation(self.root.identity())
            .map_err(|_| BootstrapError("preflight B1 receipt authority store"))?;
        match &current {
            VersionedStateRoot::V1(value) => layout
                .validate_matching_marker_if_present(value)
                .map_err(|_| BootstrapError("revalidate B1 receipt V1 initialization marker"))?,
            VersionedStateRoot::V2(value) => {
                layout
                    .validate_matching_marker_if_present_v2(value)
                    .map_err(|_| BootstrapError("revalidate B1 receipt V2 initialization marker"))?
            }
            VersionedStateRoot::V3(value) => {
                layout
                    .validate_matching_marker_if_present_v3(value)
                    .map_err(|_| BootstrapError("revalidate B1 receipt V3 initialization marker"))?
            }
        }
        if current != self.locked_root
            || current.authority_store_id() != self.authority_store_id
            || current.bootstrap_home() != self.root.identity()
        {
            return Err(BootstrapError(
                "B1 receipt authority scope changed during transaction",
            ));
        }
        self.root
            .directory()
            .revalidate_entry(&self.run_entry)
            .and_then(|()| {
                self.root
                    .directory()
                    .open_controlled_directory_entry(&self.run_entry)
                    .map(drop)
            })
            .map_err(|_| BootstrapError("B1 receipt run directory changed identity"))?;
        self.run
            .revalidate_entry(&self.agent_hub_entry)
            .and_then(|()| {
                self.run
                    .open_controlled_directory_entry(&self.agent_hub_entry)
                    .map(drop)
            })
            .map_err(|_| BootstrapError("B1 receipt agent-hub directory changed identity"))?;
        self.retained_post_hsa_ledger_directories
            .revalidate(self.root.directory())?;
        self.verify_named_root_lock_scope()
    }

    fn verify_named_root_lock_scope(&self) -> Result<(), BootstrapError> {
        revalidate_world_work_receipt_root_lock_scope(
            self.root,
            &self.authority_entry,
            &self.authority,
            &self.lock_entry,
            &self.lock_directory,
            &self.root_lock_entry,
        )
    }
}

fn validate_post_hsa_obligation_ledger_component(value: &str) -> Result<(), BootstrapError> {
    if value.is_empty()
        || value == "."
        || value == ".."
        || value.contains('/')
        || value.contains('\\')
        || value.contains('\0')
    {
        return Err(BootstrapError(
            "post-HSA obligation ledger path component is unsafe",
        ));
    }
    Ok(())
}

fn read_exact_regular_file(
    parent: &TrustedDirectory,
    target: &str,
) -> Result<Option<Vec<u8>>, BootstrapError> {
    validate_post_hsa_obligation_ledger_component(target)?;
    let entry = parent
        .entries()
        .map_err(|_| BootstrapError("enumerate post-HSA obligation ledger artifact"))?
        .into_iter()
        .find(|entry| entry.name == target);
    let Some(entry) = entry else {
        return Ok(None);
    };
    if entry.kind != EntryKind::RegularFile {
        return Err(BootstrapError(
            "post-HSA obligation ledger artifact is not a regular file",
        ));
    }
    let bytes = parent
        .open_file_entry(&entry)
        .and_then(|file| file.read_all())
        .and_then(|bytes| parent.revalidate_entry(&entry).map(|()| bytes))
        .map_err(|_| BootstrapError("read exact post-HSA obligation ledger artifact"))?;
    Ok(Some(bytes))
}

fn reconcile_post_hsa_obligation_ledger_temps(
    directory: &TrustedDirectory,
) -> Result<(), BootstrapError> {
    let entries = directory
        .entries()
        .map_err(|_| BootstrapError("enumerate post-HSA obligation ledger temps"))?;
    for entry in entries.into_iter().filter(|entry| {
        entry
            .name
            .starts_with(POST_HSA_OBLIGATION_LEDGER_TEMP_PREFIX)
    }) {
        let suffix = entry
            .name
            .strip_prefix(POST_HSA_OBLIGATION_LEDGER_TEMP_PREFIX)
            .and_then(|value| value.strip_suffix(POST_HSA_OBLIGATION_LEDGER_TEMP_SUFFIX));
        if entry.kind != EntryKind::RegularFile
            || !suffix.is_some_and(|hex| {
                hex.len() == 32
                    && hex
                        .bytes()
                        .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
            })
        {
            return Err(BootstrapError(
                "post-HSA obligation ledger namespace contains an unsafe temp",
            ));
        }
        directory
            .open_file_entry(&entry)
            .and_then(|file| file.sync())
            .and_then(|()| directory.revalidate_entry(&entry))
            .map_err(|_| BootstrapError("revalidate post-HSA obligation ledger temp"))?;
        directory
            .unlink_file(&entry.name)
            .map_err(|_| BootstrapError("remove post-HSA obligation ledger temp"))?;
    }
    Ok(())
}

fn publish_post_hsa_obligation_ledger_file(
    parent: &TrustedDirectory,
    target: &str,
    bytes: &[u8],
    replace: bool,
) -> Result<(), BootstrapError> {
    use rand::RngCore as _;

    validate_post_hsa_obligation_ledger_component(target)?;
    let mut nonce_bytes = [0_u8; 16];
    rand::rngs::OsRng.fill_bytes(&mut nonce_bytes);
    let temp_name = format!(
        "{POST_HSA_OBLIGATION_LEDGER_TEMP_PREFIX}{}{POST_HSA_OBLIGATION_LEDGER_TEMP_SUFFIX}",
        nonce(nonce_bytes)
    );
    let mut temp = parent
        .create_file(&temp_name)
        .map_err(|_| BootstrapError("create post-HSA obligation ledger temp"))?;
    temp.write_all(bytes)
        .map_err(|_| BootstrapError("write post-HSA obligation ledger temp"))?;
    temp.sync()
        .map_err(|_| BootstrapError("sync post-HSA obligation ledger temp"))?;
    if !replace
        && parent
            .entry_kind(target)
            .map_err(|_| BootstrapError("revalidate post-HSA obligation publication target"))?
            .is_some()
    {
        return Err(BootstrapError(
            "post-HSA obligation ledger publication target appeared",
        ));
    }
    parent
        .rename_replace(&temp_name, temp, parent, target)
        .map_err(|_| BootstrapError("publish post-HSA obligation ledger artifact"))?;
    parent
        .sync()
        .map_err(|_| BootstrapError("sync post-HSA obligation ledger publication"))?;
    let published = read_exact_regular_file(parent, target)?.ok_or(BootstrapError(
        "published post-HSA obligation ledger artifact is absent",
    ))?;
    if published != bytes {
        return Err(BootstrapError(
            "published post-HSA obligation ledger artifact changed",
        ));
    }
    Ok(())
}

impl WorldWorkExecutionSupervisorTransactionV1<'_> {
    pub(crate) fn read_supervisor(&mut self) -> Result<Option<Vec<u8>>, BootstrapError> {
        self.verify_scope()?;
        validate_world_work_execution_supervisor_namespace(&self.agent_hub)?;
        let bytes = read_world_work_execution_supervisor(&self.agent_hub)?;
        self.verify_scope()?;
        Ok(bytes)
    }

    pub(crate) fn replace_supervisor(&mut self, bytes: &[u8]) -> Result<(), BootstrapError> {
        self.replace_supervisor_with_source_guard(bytes, &|| Ok(()))
    }

    pub(crate) fn replace_supervisor_with_source_guard(
        &mut self,
        bytes: &[u8],
        verify_source: &dyn Fn() -> Result<(), BootstrapError>,
    ) -> Result<(), BootstrapError> {
        use rand::RngCore as _;

        verify_source()?;

        self.verify_scope()?;
        validate_world_work_execution_supervisor_namespace(&self.agent_hub)?;
        let mut nonce_bytes = [0_u8; 16];
        rand::rngs::OsRng.fill_bytes(&mut nonce_bytes);
        let temp_name = format!(
            "{WORLD_WORK_EXECUTION_SUPERVISOR_TEMP_PREFIX}{}{WORLD_WORK_EXECUTION_SUPERVISOR_TEMP_SUFFIX}",
            nonce(nonce_bytes)
        );
        let mut temp = self
            .agent_hub
            .create_file(&temp_name)
            .map_err(|_| BootstrapError("create B2.1 supervisor temp"))?;
        temp.write_all(bytes)
            .map_err(|_| BootstrapError("write B2.1 supervisor temp"))?;
        temp.sync()
            .map_err(|_| BootstrapError("sync B2.1 supervisor temp"))?;
        #[cfg(all(test, target_os = "linux"))]
        run_ordinary_supervisor_before_publication_hook();
        self.verify_scope()?;
        verify_source()?;
        self.agent_hub
            .rename_replace(
                &temp_name,
                temp,
                &self.agent_hub,
                WORLD_WORK_EXECUTION_SUPERVISOR_FILE,
            )
            .map_err(|_| BootstrapError("publish B2.1 supervisor state"))?;
        self.agent_hub
            .sync()
            .and_then(|()| self.run.sync())
            .and_then(|()| self.root.directory().sync())
            .map_err(|_| BootstrapError("sync B2.1 supervisor publication path"))?;
        let published = read_world_work_execution_supervisor(&self.agent_hub)?
            .ok_or(BootstrapError("published B2.1 supervisor state is absent"))?;
        if published != bytes {
            return Err(BootstrapError("published B2.1 supervisor bytes changed"));
        }
        self.verify_scope()
    }

    pub(crate) fn finish(self) -> Result<(), BootstrapError> {
        self.verify_scope()?;
        validate_world_work_execution_supervisor_namespace(&self.agent_hub)?;
        self.agent_hub
            .sync()
            .and_then(|()| self.run.sync())
            .and_then(|()| self.root.directory().sync())
            .map_err(|_| BootstrapError("sync B2.1 supervisor transaction"))?;
        self.verify_scope()
    }

    fn verify_scope(&self) -> Result<(), BootstrapError> {
        self.verify_named_root_lock_scope()?;
        let layout = StoreLayout::open(self.root.directory())
            .map_err(|_| BootstrapError("reopen B2.1 supervisor authority layout"))?;
        layout
            .validate_closed_layout()
            .map_err(|_| BootstrapError("revalidate B2.1 supervisor authority layout"))?;
        layout
            .validate_temps()
            .map_err(|_| BootstrapError("revalidate B2.1 supervisor authority temps"))?;
        let legacy = LegacyObservation::capture(layout.bootstrap)
            .map_err(|_| BootstrapError("recapture B2.1 supervisor authority legacy state"))?;
        if legacy.has_artifact {
            return Err(BootstrapError(
                "legacy state appeared during B2.1 supervisor transaction",
            ));
        }
        let observed = layout
            .read_existing_versioned_without_reconciliation(self.root.identity())
            .map_err(|_| BootstrapError("revalidate B2.1 supervisor authority store"))?;
        match &observed {
            VersionedStateRoot::V1(value) => layout
                .validate_matching_marker_if_present(value)
                .map_err(|_| {
                    BootstrapError("revalidate B2.1 supervisor V1 initialization marker")
                })?,
            VersionedStateRoot::V2(value) => layout
                .validate_matching_marker_if_present_v2(value)
                .map_err(|_| {
                BootstrapError("revalidate B2.1 supervisor V2 initialization marker")
            })?,
            VersionedStateRoot::V3(value) => layout
                .validate_matching_marker_if_present_v3(value)
                .map_err(|_| {
                BootstrapError("revalidate B2.1 supervisor V3 initialization marker")
            })?,
        }
        if observed != self.locked_root
            || self.locked_root.bootstrap_home() != self.root.identity()
            || self.locked_root.authority_store_id() != self.authority_store_id
        {
            return Err(BootstrapError("B2.1 supervisor authority scope changed"));
        }
        self.root
            .directory()
            .revalidate_entry(&self.run_entry)
            .and_then(|()| {
                self.root
                    .directory()
                    .open_controlled_directory_entry(&self.run_entry)
                    .map(drop)
            })
            .map_err(|_| BootstrapError("B2.1 supervisor run directory changed identity"))?;
        self.run
            .revalidate_entry(&self.agent_hub_entry)
            .and_then(|()| {
                self.run
                    .open_controlled_directory_entry(&self.agent_hub_entry)
                    .map(drop)
            })
            .map_err(|_| BootstrapError("B2.1 supervisor agent-hub directory changed identity"))?;
        self.verify_named_root_lock_scope()
    }

    fn verify_named_root_lock_scope(&self) -> Result<(), BootstrapError> {
        revalidate_world_work_execution_supervisor_root_lock_scope(
            self.root,
            &self.authority_entry,
            &self.authority,
            &self.lock_entry,
            &self.lock_directory,
            &self.root_lock_entry,
        )
    }
}
fn begin_world_work_receipt_registry_transaction<'storage>(
    root: &'storage TrustedAuthorityRoot,
    expected_authority_store_id: &'storage str,
) -> Result<WorldWorkReceiptRegistryTransactionV1<'storage>, BootstrapError> {
    begin_world_work_receipt_registry_transaction_with_publication(
        root,
        expected_authority_store_id,
        #[cfg(target_os = "linux")]
        None,
    )
}

fn begin_world_work_receipt_registry_transaction_with_publication<'storage>(
    root: &'storage TrustedAuthorityRoot,
    expected_authority_store_id: &'storage str,
    #[cfg(target_os = "linux")]
    publication: Option<&crate::execution::agent_runtime::host_session_authority::world_bound_controller::AuthenticatedOrdinaryWorldPublicationV1>,
) -> Result<WorldWorkReceiptRegistryTransactionV1<'storage>, BootstrapError> {
    root.revalidate()
        .map_err(|_| BootstrapError("revalidate B1 receipt authority root"))?;
    let (authority_entry, authority) =
        open_existing_world_work_receipt_directory(root.directory(), AUTHORITY_DIRECTORY)?;
    let (lock_entry, lock_directory) =
        open_existing_world_work_receipt_directory(&authority, "lock")?;
    let root_lock_entry = lock_directory
        .entries()
        .map_err(|_| BootstrapError("enumerate B1 receipt root-lock directory"))?
        .into_iter()
        .find(|entry| entry.name == ROOT_LOCK_FILE && entry.kind == EntryKind::RegularFile)
        .ok_or(BootstrapError(
            "B1 receipt root lock was not safely enumerated",
        ))?;
    let root_lock_file = lock_directory
        .open_file_entry(&root_lock_entry)
        .map_err(|_| BootstrapError("retain exact B1 receipt root lock"))?;
    let lock = root_lock_file
        .lock_exclusive_owned()
        .map_err(|_| BootstrapError("lock exact B1 receipt root lock"))?;
    revalidate_world_work_receipt_root_lock_scope(
        root,
        &authority_entry,
        &authority,
        &lock_entry,
        &lock_directory,
        &root_lock_entry,
    )?;

    let layout = StoreLayout::open(root.directory())
        .map_err(|_| BootstrapError("open locked B1 receipt authority layout"))?;
    #[cfg(target_os = "linux")]
    if let Some(publication) = publication {
        let locked = layout
            .read_existing_versioned_without_reconciliation(root.identity())
            .map_err(|_| BootstrapError("read existing B1 publication authority"))?;
        publication.verify_locked_ordinary_world_publication(&locked)?;
    }
    layout
        .validate_temps()
        .map_err(|_| BootstrapError("validate B1 receipt authority temps"))?;
    layout
        .reconcile_temps()
        .map_err(|_| BootstrapError("reconcile B1 receipt authority temps"))?;
    layout
        .validate_closed_layout()
        .map_err(|_| BootstrapError("validate B1 receipt authority layout"))?;
    let legacy = LegacyObservation::capture(layout.bootstrap)
        .map_err(|_| BootstrapError("capture B1 receipt authority legacy state"))?;
    if legacy.has_artifact {
        return Err(BootstrapError(
            "legacy state is incompatible with B1 receipt authority",
        ));
    }
    let locked_root = layout
        .read_existing_versioned_without_reconciliation(root.identity())
        .map_err(|_| BootstrapError("preflight B1 receipt authority store"))?;
    match &locked_root {
        VersionedStateRoot::V1(value) => layout
            .validate_matching_marker_if_present(value)
            .map_err(|_| BootstrapError("validate B1 receipt V1 initialization marker"))?,
        VersionedStateRoot::V2(value) => layout
            .validate_matching_marker_if_present_v2(value)
            .map_err(|_| BootstrapError("validate B1 receipt V2 initialization marker"))?,
        VersionedStateRoot::V3(value) => layout
            .validate_matching_marker_if_present_v3(value)
            .map_err(|_| BootstrapError("validate B1 receipt V3 initialization marker"))?,
    }
    if locked_root.bootstrap_home() != root.identity()
        || locked_root.authority_store_id() != expected_authority_store_id
    {
        return Err(BootstrapError("B1 receipt authority scope mismatch"));
    }
    match &locked_root {
        VersionedStateRoot::V1(value) => layout.reconcile_after_preflight(value),
        VersionedStateRoot::V2(value) => layout.reconcile_after_preflight_v2(value),
        VersionedStateRoot::V3(value) => layout.reconcile_after_preflight_v3(value),
    }
    .map_err(|_| BootstrapError("reconcile B1 receipt authority store"))?;
    let reconciled = layout
        .read_existing_versioned_without_reconciliation(root.identity())
        .map_err(|_| BootstrapError("revalidate reconciled B1 authority store"))?;
    if reconciled != locked_root {
        return Err(BootstrapError(
            "B1 receipt authority changed during canonical preflight",
        ));
    }
    revalidate_world_work_receipt_root_lock_scope(
        root,
        &authority_entry,
        &authority,
        &lock_entry,
        &lock_directory,
        &root_lock_entry,
    )?;
    let (run_entry, run) = open_or_create_world_work_receipt_directory(root.directory(), "run")?;
    let (agent_hub_entry, agent_hub) =
        open_or_create_world_work_receipt_directory(&run, "agent-hub")?;
    reconcile_world_work_receipt_registry_namespace(&agent_hub)?;
    let transaction = WorldWorkReceiptRegistryTransactionV1 {
        root,
        authority_store_id: expected_authority_store_id,
        locked_root,
        authority_entry,
        authority,
        lock_entry,
        lock_directory,
        root_lock_entry,
        _root_lock_file: root_lock_file,
        run_entry,
        run,
        agent_hub_entry,
        agent_hub,
        retained_post_hsa_ledger_directories: RetainedPostHsaLedgerDirectories::default(),
        _lock: lock,
    };
    transaction.verify_scope()?;
    Ok(transaction)
}

fn begin_world_work_execution_supervisor_transaction<'storage>(
    root: &'storage TrustedAuthorityRoot,
    expected_authority_store_id: &'storage str,
) -> Result<WorldWorkExecutionSupervisorTransactionV1<'storage>, BootstrapError> {
    root.revalidate()
        .map_err(|_| BootstrapError("revalidate B2.1 supervisor authority root"))?;
    let (authority_entry, authority) = open_existing_world_work_execution_supervisor_directory(
        root.directory(),
        AUTHORITY_DIRECTORY,
    )?;
    let (lock_entry, lock_directory) =
        open_existing_world_work_execution_supervisor_directory(&authority, "lock")?;
    let root_lock_entry = lock_directory
        .entries()
        .map_err(|_| BootstrapError("enumerate B2.1 supervisor root-lock directory"))?
        .into_iter()
        .find(|entry| entry.name == ROOT_LOCK_FILE && entry.kind == EntryKind::RegularFile)
        .ok_or(BootstrapError(
            "B2.1 supervisor root lock was not safely enumerated",
        ))?;
    let root_lock_file = lock_directory
        .open_file_entry(&root_lock_entry)
        .map_err(|_| BootstrapError("retain exact B2.1 supervisor root lock"))?;
    let lock = root_lock_file
        .lock_exclusive_owned()
        .map_err(|_| BootstrapError("lock exact B2.1 supervisor root lock"))?;
    revalidate_world_work_execution_supervisor_root_lock_scope(
        root,
        &authority_entry,
        &authority,
        &lock_entry,
        &lock_directory,
        &root_lock_entry,
    )?;

    let layout = StoreLayout::open(root.directory())
        .map_err(|_| BootstrapError("open locked B2.1 supervisor authority layout"))?;
    layout
        .validate_temps()
        .map_err(|_| BootstrapError("validate B2.1 supervisor authority temps"))?;
    layout
        .reconcile_temps()
        .map_err(|_| BootstrapError("reconcile B2.1 supervisor authority temps"))?;
    layout
        .validate_closed_layout()
        .map_err(|_| BootstrapError("validate B2.1 supervisor authority layout"))?;
    let legacy = LegacyObservation::capture(layout.bootstrap)
        .map_err(|_| BootstrapError("capture B2.1 supervisor authority legacy state"))?;
    if legacy.has_artifact {
        return Err(BootstrapError(
            "legacy state is incompatible with B2.1 supervisor authority",
        ));
    }
    let locked_root = layout
        .read_existing_versioned_without_reconciliation(root.identity())
        .map_err(|_| BootstrapError("preflight B2.1 supervisor authority store"))?;
    match &locked_root {
        VersionedStateRoot::V1(value) => layout
            .validate_matching_marker_if_present(value)
            .map_err(|_| BootstrapError("validate B2.1 supervisor V1 initialization marker"))?,
        VersionedStateRoot::V2(value) => layout
            .validate_matching_marker_if_present_v2(value)
            .map_err(|_| BootstrapError("validate B2.1 supervisor V2 initialization marker"))?,
        VersionedStateRoot::V3(value) => layout
            .validate_matching_marker_if_present_v3(value)
            .map_err(|_| BootstrapError("validate B2.1 supervisor V3 initialization marker"))?,
    }
    if locked_root.bootstrap_home() != root.identity()
        || locked_root.authority_store_id() != expected_authority_store_id
    {
        return Err(BootstrapError("B2.1 supervisor authority scope mismatch"));
    }
    match &locked_root {
        VersionedStateRoot::V1(value) => layout.reconcile_after_preflight(value),
        VersionedStateRoot::V2(value) => layout.reconcile_after_preflight_v2(value),
        VersionedStateRoot::V3(value) => layout.reconcile_after_preflight_v3(value),
    }
    .map_err(|_| BootstrapError("reconcile B2.1 supervisor authority store"))?;
    let reconciled = layout
        .read_existing_versioned_without_reconciliation(root.identity())
        .map_err(|_| BootstrapError("revalidate reconciled B2.1 supervisor authority store"))?;
    if reconciled != locked_root {
        return Err(BootstrapError(
            "B2.1 supervisor authority changed during canonical preflight",
        ));
    }
    revalidate_world_work_execution_supervisor_root_lock_scope(
        root,
        &authority_entry,
        &authority,
        &lock_entry,
        &lock_directory,
        &root_lock_entry,
    )?;
    let (run_entry, run) =
        open_or_create_world_work_execution_supervisor_directory(root.directory(), "run")?;
    let (agent_hub_entry, agent_hub) =
        open_or_create_world_work_execution_supervisor_directory(&run, "agent-hub")?;
    reconcile_world_work_execution_supervisor_namespace(&agent_hub)?;
    let transaction = WorldWorkExecutionSupervisorTransactionV1 {
        root,
        authority_store_id: expected_authority_store_id,
        locked_root,
        authority_entry,
        authority,
        lock_entry,
        lock_directory,
        root_lock_entry,
        _root_lock_file: root_lock_file,
        run_entry,
        run,
        agent_hub_entry,
        agent_hub,
        _lock: lock,
    };
    transaction.verify_scope()?;
    Ok(transaction)
}

fn open_existing_world_work_execution_supervisor_directory(
    parent: &TrustedDirectory,
    name: &str,
) -> Result<(DirectoryEntry, TrustedDirectory), BootstrapError> {
    let entry = parent
        .entries()
        .map_err(|_| BootstrapError("enumerate B2.1 supervisor authority directory"))?
        .into_iter()
        .find(|entry| entry.name == name && entry.kind == EntryKind::Directory)
        .ok_or(BootstrapError(
            "B2.1 supervisor authority directory was not safely enumerated",
        ))?;
    parent
        .revalidate_entry(&entry)
        .map_err(|_| BootstrapError("revalidate B2.1 supervisor authority directory"))?;
    let directory = parent
        .open_controlled_directory_entry(&entry)
        .map_err(|_| BootstrapError("retain B2.1 supervisor authority directory"))?;
    Ok((entry, directory))
}

fn revalidate_world_work_execution_supervisor_root_lock_scope(
    root: &TrustedAuthorityRoot,
    authority_entry: &DirectoryEntry,
    authority: &TrustedDirectory,
    lock_entry: &DirectoryEntry,
    lock_directory: &TrustedDirectory,
    root_lock_entry: &DirectoryEntry,
) -> Result<(), BootstrapError> {
    root.revalidate()
        .map_err(|_| BootstrapError("B2.1 supervisor authority root was rebound or replaced"))?;
    root.directory()
        .revalidate_entry(authority_entry)
        .and_then(|()| {
            root.directory()
                .open_controlled_directory_entry(authority_entry)
                .map(drop)
        })
        .map_err(|_| BootstrapError("B2.1 supervisor authority directory changed identity"))?;
    authority
        .revalidate_entry(lock_entry)
        .and_then(|()| {
            authority
                .open_controlled_directory_entry(lock_entry)
                .map(drop)
        })
        .map_err(|_| BootstrapError("B2.1 supervisor lock directory changed identity"))?;
    lock_directory
        .revalidate_entry(root_lock_entry)
        .and_then(|()| lock_directory.open_file_entry(root_lock_entry).map(drop))
        .map_err(|_| BootstrapError("B2.1 supervisor root lock changed identity"))
}

fn open_or_create_world_work_execution_supervisor_directory(
    parent: &TrustedDirectory,
    name: &str,
) -> Result<(DirectoryEntry, TrustedDirectory), BootstrapError> {
    match parent
        .entry_kind(name)
        .map_err(|_| BootstrapError("inspect B2.1 supervisor directory"))?
    {
        None => {
            parent
                .create_directory(name)
                .map_err(|_| BootstrapError("create B2.1 supervisor directory"))?;
        }
        Some(EntryKind::Directory) => {}
        Some(EntryKind::RegularFile | EntryKind::Symlink | EntryKind::Other) => {
            return Err(BootstrapError("B2.1 supervisor directory is unsafe"));
        }
    }
    let entry = parent
        .entries()
        .map_err(|_| BootstrapError("enumerate B2.1 supervisor parent"))?
        .into_iter()
        .find(|entry| entry.name == name && entry.kind == EntryKind::Directory)
        .ok_or(BootstrapError(
            "B2.1 supervisor directory was not safely enumerated",
        ))?;
    parent
        .revalidate_entry(&entry)
        .map_err(|_| BootstrapError("revalidate B2.1 supervisor directory"))?;
    let directory = parent
        .open_controlled_directory_entry(&entry)
        .map_err(|_| BootstrapError("retain B2.1 supervisor directory"))?;
    Ok((entry, directory))
}
fn open_existing_world_work_receipt_directory(
    parent: &TrustedDirectory,
    name: &str,
) -> Result<(DirectoryEntry, TrustedDirectory), BootstrapError> {
    let entry = parent
        .entries()
        .map_err(|_| BootstrapError("enumerate B1 receipt authority directory"))?
        .into_iter()
        .find(|entry| entry.name == name && entry.kind == EntryKind::Directory)
        .ok_or(BootstrapError(
            "B1 receipt authority directory was not safely enumerated",
        ))?;
    parent
        .revalidate_entry(&entry)
        .map_err(|_| BootstrapError("revalidate B1 receipt authority directory"))?;
    let directory = parent
        .open_controlled_directory_entry(&entry)
        .map_err(|_| BootstrapError("retain B1 receipt authority directory"))?;
    Ok((entry, directory))
}

fn revalidate_world_work_receipt_root_lock_scope(
    root: &TrustedAuthorityRoot,
    authority_entry: &DirectoryEntry,
    authority: &TrustedDirectory,
    lock_entry: &DirectoryEntry,
    lock_directory: &TrustedDirectory,
    root_lock_entry: &DirectoryEntry,
) -> Result<(), BootstrapError> {
    root.revalidate()
        .map_err(|_| BootstrapError("B1 receipt authority root was rebound or replaced"))?;
    root.directory()
        .revalidate_entry(authority_entry)
        .and_then(|()| {
            root.directory()
                .open_controlled_directory_entry(authority_entry)
                .map(drop)
        })
        .map_err(|_| BootstrapError("B1 receipt authority directory changed identity"))?;
    authority
        .revalidate_entry(lock_entry)
        .and_then(|()| {
            authority
                .open_controlled_directory_entry(lock_entry)
                .map(drop)
        })
        .map_err(|_| BootstrapError("B1 receipt lock directory changed identity"))?;
    lock_directory
        .revalidate_entry(root_lock_entry)
        .and_then(|()| lock_directory.open_file_entry(root_lock_entry).map(drop))
        .map_err(|_| BootstrapError("B1 receipt root lock changed identity"))
}

fn open_or_create_world_work_receipt_directory(
    parent: &TrustedDirectory,
    name: &str,
) -> Result<(DirectoryEntry, TrustedDirectory), BootstrapError> {
    match parent
        .entry_kind(name)
        .map_err(|_| BootstrapError("inspect B1 receipt-registry directory"))?
    {
        None => {
            parent
                .create_directory(name)
                .map_err(|_| BootstrapError("create B1 receipt-registry directory"))?;
        }
        Some(EntryKind::Directory) => {}
        Some(EntryKind::RegularFile | EntryKind::Symlink | EntryKind::Other) => {
            return Err(BootstrapError("B1 receipt-registry directory is unsafe"));
        }
    }
    let entry = parent
        .entries()
        .map_err(|_| BootstrapError("enumerate B1 receipt-registry parent"))?
        .into_iter()
        .find(|entry| entry.name == name && entry.kind == EntryKind::Directory)
        .ok_or(BootstrapError(
            "B1 receipt-registry directory was not safely enumerated",
        ))?;
    parent
        .revalidate_entry(&entry)
        .map_err(|_| BootstrapError("revalidate B1 receipt-registry directory"))?;
    let directory = parent
        .open_controlled_directory_entry(&entry)
        .map_err(|_| BootstrapError("retain B1 receipt-registry directory"))?;
    Ok((entry, directory))
}

fn reconcile_world_work_receipt_registry_namespace(
    agent_hub: &TrustedDirectory,
) -> Result<(), BootstrapError> {
    let entries = agent_hub
        .entries()
        .map_err(|_| BootstrapError("enumerate B1 receipt-registry namespace"))?;
    let receipt_entries = entries
        .iter()
        .filter(|entry| entry.name.starts_with(WORLD_WORK_RECEIPT_REGISTRY_PREFIX))
        .cloned()
        .collect::<Vec<_>>();
    let mut temps = Vec::new();
    for entry in &receipt_entries {
        if entry.name == WORLD_WORK_RECEIPT_REGISTRY_FILE {
            validate_world_work_receipt_registry_entry(agent_hub, entry)?;
            continue;
        }
        if entry.kind != EntryKind::RegularFile
            || !is_world_work_receipt_registry_temp_name(&entry.name)
        {
            return Err(BootstrapError(
                "B1 receipt-registry namespace contains an unsafe entry",
            ));
        }
        agent_hub
            .open_file_entry(entry)
            .and_then(|file| file.sync())
            .and_then(|()| agent_hub.revalidate_entry(entry))
            .map_err(|_| BootstrapError("revalidate B1 receipt-registry temp"))?;
        temps.push(entry.clone());
    }
    let revalidated_entries = agent_hub
        .entries()
        .map_err(|_| BootstrapError("re-enumerate B1 receipt-registry namespace"))?
        .into_iter()
        .filter(|entry| entry.name.starts_with(WORLD_WORK_RECEIPT_REGISTRY_PREFIX))
        .collect::<Vec<_>>();
    if revalidated_entries != receipt_entries {
        return Err(BootstrapError(
            "B1 receipt-registry namespace changed before reconciliation",
        ));
    }
    for entry in temps {
        agent_hub
            .revalidate_entry(&entry)
            .and_then(|()| agent_hub.open_file_entry(&entry).map(drop))
            .map_err(|_| BootstrapError("revalidate B1 receipt-registry temp for removal"))?;
        agent_hub
            .unlink_file(&entry.name)
            .map_err(|_| BootstrapError("remove interrupted B1 receipt-registry temp"))?;
        if agent_hub
            .entry_kind(&entry.name)
            .map_err(|_| BootstrapError("revalidate removed B1 receipt-registry temp"))?
            .is_some()
        {
            return Err(BootstrapError(
                "B1 receipt-registry temp reappeared during reconciliation",
            ));
        }
    }
    validate_world_work_receipt_registry_namespace(agent_hub)
}

fn validate_world_work_receipt_registry_namespace(
    agent_hub: &TrustedDirectory,
) -> Result<(), BootstrapError> {
    for entry in agent_hub
        .entries()
        .map_err(|_| BootstrapError("validate B1 receipt-registry namespace"))?
    {
        if entry.name == WORLD_WORK_RECEIPT_REGISTRY_FILE {
            validate_world_work_receipt_registry_entry(agent_hub, &entry)?;
        } else if entry.name.starts_with(WORLD_WORK_RECEIPT_REGISTRY_PREFIX) {
            return Err(BootstrapError(
                "B1 receipt-registry namespace contains an uncommitted entry",
            ));
        }
    }
    Ok(())
}

fn validate_world_work_receipt_registry_entry(
    agent_hub: &TrustedDirectory,
    entry: &DirectoryEntry,
) -> Result<(), BootstrapError> {
    if entry.kind != EntryKind::RegularFile {
        return Err(BootstrapError("B1 receipt registry is not a regular file"));
    }
    agent_hub
        .open_file_entry(entry)
        .and_then(|file| file.sync())
        .and_then(|()| agent_hub.revalidate_entry(entry))
        .map_err(|_| BootstrapError("B1 receipt registry changed identity"))
}

fn read_world_work_receipt_registry(
    agent_hub: &TrustedDirectory,
) -> Result<Option<Vec<u8>>, BootstrapError> {
    let entry = agent_hub
        .entries()
        .map_err(|_| BootstrapError("enumerate B1 receipt registry"))?
        .into_iter()
        .find(|entry| entry.name == WORLD_WORK_RECEIPT_REGISTRY_FILE);
    let Some(entry) = entry else {
        return Ok(None);
    };
    validate_world_work_receipt_registry_entry(agent_hub, &entry)?;
    let bytes = agent_hub
        .open_file_entry(&entry)
        .and_then(|file| file.read_all())
        .map_err(|_| BootstrapError("read B1 receipt registry"))?;
    agent_hub
        .revalidate_entry(&entry)
        .map_err(|_| BootstrapError("revalidate read B1 receipt registry"))?;
    Ok(Some(bytes))
}

fn is_world_work_receipt_registry_temp_name(name: &str) -> bool {
    let Some(hex) = name
        .strip_prefix(WORLD_WORK_RECEIPT_REGISTRY_TEMP_PREFIX)
        .and_then(|value| value.strip_suffix(WORLD_WORK_RECEIPT_REGISTRY_TEMP_SUFFIX))
    else {
        return false;
    };
    hex.len() == 32
        && hex
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn reconcile_world_work_execution_supervisor_namespace(
    agent_hub: &TrustedDirectory,
) -> Result<(), BootstrapError> {
    let entries = agent_hub
        .entries()
        .map_err(|_| BootstrapError("enumerate B2.1 supervisor namespace"))?;
    let supervisor_entries = entries
        .iter()
        .filter(|entry| {
            entry
                .name
                .starts_with(WORLD_WORK_EXECUTION_SUPERVISOR_PREFIX)
        })
        .cloned()
        .collect::<Vec<_>>();
    let mut temps = Vec::new();
    for entry in &supervisor_entries {
        if entry.name == WORLD_WORK_EXECUTION_SUPERVISOR_FILE {
            validate_world_work_execution_supervisor_entry(agent_hub, entry)?;
            continue;
        }
        if entry.kind != EntryKind::RegularFile
            || !is_world_work_execution_supervisor_temp_name(&entry.name)
        {
            return Err(BootstrapError(
                "B2.1 supervisor namespace contains an unsafe entry",
            ));
        }
        agent_hub
            .open_file_entry(entry)
            .and_then(|file| file.sync())
            .and_then(|()| agent_hub.revalidate_entry(entry))
            .map_err(|_| BootstrapError("revalidate B2.1 supervisor temp"))?;
        temps.push(entry.clone());
    }
    let revalidated_entries = agent_hub
        .entries()
        .map_err(|_| BootstrapError("re-enumerate B2.1 supervisor namespace"))?
        .into_iter()
        .filter(|entry| {
            entry
                .name
                .starts_with(WORLD_WORK_EXECUTION_SUPERVISOR_PREFIX)
        })
        .collect::<Vec<_>>();
    if revalidated_entries != supervisor_entries {
        return Err(BootstrapError(
            "B2.1 supervisor namespace changed before reconciliation",
        ));
    }
    for entry in temps {
        agent_hub
            .revalidate_entry(&entry)
            .and_then(|()| agent_hub.open_file_entry(&entry).map(drop))
            .map_err(|_| BootstrapError("revalidate B2.1 supervisor temp for removal"))?;
        agent_hub
            .unlink_file(&entry.name)
            .map_err(|_| BootstrapError("remove interrupted B2.1 supervisor temp"))?;
        if agent_hub
            .entry_kind(&entry.name)
            .map_err(|_| BootstrapError("revalidate removed B2.1 supervisor temp"))?
            .is_some()
        {
            return Err(BootstrapError(
                "B2.1 supervisor temp reappeared during reconciliation",
            ));
        }
    }
    validate_world_work_execution_supervisor_namespace(agent_hub)
}

fn validate_world_work_execution_supervisor_namespace(
    agent_hub: &TrustedDirectory,
) -> Result<(), BootstrapError> {
    for entry in agent_hub
        .entries()
        .map_err(|_| BootstrapError("validate B2.1 supervisor namespace"))?
    {
        if entry.name == WORLD_WORK_EXECUTION_SUPERVISOR_FILE {
            validate_world_work_execution_supervisor_entry(agent_hub, &entry)?;
        } else if entry
            .name
            .starts_with(WORLD_WORK_EXECUTION_SUPERVISOR_PREFIX)
        {
            return Err(BootstrapError(
                "B2.1 supervisor namespace contains an uncommitted entry",
            ));
        }
    }
    Ok(())
}

fn validate_world_work_execution_supervisor_entry(
    agent_hub: &TrustedDirectory,
    entry: &DirectoryEntry,
) -> Result<(), BootstrapError> {
    if entry.kind != EntryKind::RegularFile {
        return Err(BootstrapError(
            "B2.1 supervisor state is not a regular file",
        ));
    }
    agent_hub
        .open_file_entry(entry)
        .and_then(|file| file.sync())
        .and_then(|()| agent_hub.revalidate_entry(entry))
        .map_err(|_| BootstrapError("B2.1 supervisor state changed identity"))
}

fn read_world_work_execution_supervisor(
    agent_hub: &TrustedDirectory,
) -> Result<Option<Vec<u8>>, BootstrapError> {
    let entry = agent_hub
        .entries()
        .map_err(|_| BootstrapError("enumerate B2.1 supervisor state"))?
        .into_iter()
        .find(|entry| entry.name == WORLD_WORK_EXECUTION_SUPERVISOR_FILE);
    let Some(entry) = entry else {
        return Ok(None);
    };
    validate_world_work_execution_supervisor_entry(agent_hub, &entry)?;
    let bytes = agent_hub
        .open_file_entry(&entry)
        .and_then(|file| file.read_all())
        .map_err(|_| BootstrapError("read B2.1 supervisor state"))?;
    agent_hub
        .revalidate_entry(&entry)
        .map_err(|_| BootstrapError("revalidate read B2.1 supervisor state"))?;
    Ok(Some(bytes))
}

fn is_world_work_execution_supervisor_temp_name(name: &str) -> bool {
    let Some(hex) = name
        .strip_prefix(WORLD_WORK_EXECUTION_SUPERVISOR_TEMP_PREFIX)
        .and_then(|value| value.strip_suffix(WORLD_WORK_EXECUTION_SUPERVISOR_TEMP_SUFFIX))
    else {
        return false;
    };
    hex.len() == 32
        && hex
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}
pub(super) fn with_semantic_preflight<T>(
    path: &std::path::Path,
    mode: SemanticPreflightMode,
    operation: impl FnOnce(
        &StoreLayout<'_>,
        &crate::execution::agent_runtime::host_session_authority::schema::CanonicalDirectoryV1,
        LockedClassification,
        TrustedOwnedFileLock,
    ) -> Result<T, BootstrapError>,
) -> Result<T, BootstrapError> {
    let root_handle = TrustedAuthorityRoot::open(path)
        .map_err(|_| BootstrapError("open trusted authority root"))?;
    with_opened_semantic_preflight(&root_handle, mode, operation)
}

pub(super) fn with_opened_semantic_preflight<T>(
    root_handle: &TrustedAuthorityRoot,
    mode: SemanticPreflightMode,
    operation: impl FnOnce(
        &StoreLayout<'_>,
        &crate::execution::agent_runtime::host_session_authority::schema::CanonicalDirectoryV1,
        LockedClassification,
        TrustedOwnedFileLock,
    ) -> Result<T, BootstrapError>,
) -> Result<T, BootstrapError> {
    root_handle
        .revalidate()
        .map_err(|_| BootstrapError("revalidate trusted authority root"))?;
    let lock_scope = StoreLayoutLockScope::open(root_handle.directory())
        .map_err(|_| BootstrapError("open authority store lock scope"))?;
    let lock = lock_scope
        .root_lock
        .lock_exclusive_owned()
        .map_err(|_| BootstrapError("lock authority store root"))?;
    lock_scope
        .validate_temps()
        .map_err(|_| BootstrapError("validate authority-store temps before preflight"))?;
    if matches!(mode, SemanticPreflightMode::LegacyWriter)
        && lock_scope
            .authority_activated()
            .map_err(|_| BootstrapError("inspect authority activation"))?
    {
        return Err(BootstrapError(
            "legacy authority writer is disabled after A1 activation",
        ));
    }
    lock_scope
        .reconcile_temps()
        .map_err(|_| BootstrapError("reconcile validated authority-store temps"))?;
    let layout = lock_scope
        .finish()
        .map_err(|_| BootstrapError("open locked authority store layout"))?;
    layout
        .validate_closed_layout()
        .map_err(|_| BootstrapError("validate authority-store layout before preflight"))?;
    let observed = layout
        .semantic_preflight(root_handle.identity())
        .map_err(|_| BootstrapError("semantic authority-store preflight"))?;
    operation(&layout, root_handle.identity(), observed, lock)
}

pub(super) fn begin_legacy_state_store_transaction(
    path: &std::path::Path,
) -> Result<LegacyStateStoreTransactionV1, BootstrapError> {
    let root = TrustedAuthorityRoot::open(path)
        .map_err(|_| BootstrapError("open retained legacy StateStore root"))?;
    begin_opened_legacy_state_store_transaction(root)
}

pub(super) fn begin_legacy_state_store_transaction_for_identity(
    path: &std::path::Path,
    expected: &CanonicalDirectoryV1,
) -> Result<LegacyStateStoreTransactionV1, BootstrapError> {
    let root = TrustedAuthorityRoot::open(path)
        .map_err(|_| BootstrapError("open retained legacy StateStore root"))?;
    if root.identity() != expected {
        return Err(BootstrapError(
            "legacy StateStore root differs from bootstrap-home identity",
        ));
    }
    begin_opened_legacy_state_store_transaction(root)
}

fn begin_opened_legacy_state_store_transaction(
    root: TrustedAuthorityRoot,
) -> Result<LegacyStateStoreTransactionV1, BootstrapError> {
    let (legacy_observation, retained_directories, lock) = with_opened_semantic_preflight(
        &root,
        SemanticPreflightMode::LegacyWriter,
        |layout, _, observed, lock| {
            if observed.authority_classification != BootstrapClassificationV1::FreshAbsent
                || !layout
                    .tmp
                    .entries()
                    .map_err(|_| BootstrapError("enumerate authority temps for legacy writer"))?
                    .is_empty()
            {
                return Err(BootstrapError(
                    "legacy authority writer is disabled after A1 activation",
                ));
            }
            let retained_directories =
                RetainedLegacyDirectories::from_observation(layout.bootstrap, &observed.legacy)?;
            Ok((observed.legacy, retained_directories, lock))
        },
    )?;
    root.revalidate()
        .map_err(|_| BootstrapError("revalidate retained legacy StateStore root"))?;
    Ok(LegacyStateStoreTransactionV1 {
        root,
        _legacy_observation: legacy_observation,
        retained_directories,
        _lock: lock,
    })
}

pub(super) fn with_existing_semantic_preflight<T>(
    path: &std::path::Path,
    operation: impl FnOnce(&SemanticTransaction<'_, '_>) -> Result<T, BootstrapError>,
) -> Result<T, BootstrapError> {
    let root_handle = TrustedAuthorityRoot::open(path)
        .map_err(|_| BootstrapError("open trusted authority root"))?;
    with_opened_existing_semantic_preflight(&root_handle, operation)
}

pub(super) fn with_existing_versioned_semantic_preflight<T>(
    path: &std::path::Path,
    operation: impl FnOnce(&VersionedSemanticTransaction<'_, '_>) -> Result<T, BootstrapError>,
) -> Result<T, BootstrapError> {
    let root_handle = TrustedAuthorityRoot::open(path)
        .map_err(|_| BootstrapError("open trusted versioned authority root"))?;
    with_opened_existing_versioned_semantic_preflight(&root_handle, operation)
}

pub(super) fn with_opened_existing_versioned_semantic_preflight<T>(
    root_handle: &TrustedAuthorityRoot,
    operation: impl FnOnce(&VersionedSemanticTransaction<'_, '_>) -> Result<T, BootstrapError>,
) -> Result<T, BootstrapError> {
    root_handle
        .revalidate()
        .map_err(|_| BootstrapError("revalidate trusted versioned authority root"))?;
    let lock_scope = StoreLayoutLockScope::open_existing_activated(root_handle.directory())
        .map_err(|_| BootstrapError("open activated versioned authority layout"))?;
    let _lock = lock_scope
        .root_lock
        .lock_exclusive_owned()
        .map_err(|_| BootstrapError("lock versioned authority root"))?;
    with_existing_versioned_semantic_scope(root_handle, lock_scope, &_lock, operation)
}

fn with_existing_versioned_semantic_scope<'root, T>(
    root_handle: &'root TrustedAuthorityRoot,
    lock_scope: StoreLayoutLockScope<'root>,
    _held_lock: &TrustedOwnedFileLock,
    operation: impl FnOnce(&VersionedSemanticTransaction<'_, '_>) -> Result<T, BootstrapError>,
) -> Result<T, BootstrapError> {
    #[cfg(all(test, target_os = "linux"))]
    SEMANTIC_PREFLIGHT_COUNT.with(|count| count.set(count.get() + 1));
    lock_scope
        .validate_temps()
        .map_err(|_| BootstrapError("validate versioned authority temps"))?;
    lock_scope
        .reconcile_temps()
        .map_err(|_| BootstrapError("reconcile versioned authority temps"))?;
    let layout = lock_scope
        .finish()
        .map_err(|_| BootstrapError("open versioned authority layout"))?;
    layout
        .validate_closed_layout()
        .map_err(|_| BootstrapError("validate versioned authority layout"))?;
    let legacy = LegacyObservation::capture(layout.bootstrap)
        .map_err(|_| BootstrapError("capture versioned authority legacy state"))?;
    if legacy.has_artifact {
        return Err(BootstrapError(
            "legacy state is incompatible with versioned authority",
        ));
    }
    let root = layout
        .read_existing_versioned_without_reconciliation(root_handle.identity())
        .map_err(|_| BootstrapError("read existing versioned authority root"))?;
    match &root {
        VersionedStateRoot::V1(root) => layout
            .validate_matching_marker_if_present(root)
            .map_err(|_| BootstrapError("validate V1 initialization marker"))?,
        VersionedStateRoot::V2(root) => layout
            .validate_matching_marker_if_present_v2(root)
            .map_err(|_| BootstrapError("validate V2 initialization marker"))?,
        VersionedStateRoot::V3(root) => layout
            .validate_matching_marker_if_present_v3(root)
            .map_err(|_| BootstrapError("validate V3 initialization marker"))?,
    }
    operation(&VersionedSemanticTransaction {
        layout: &layout,
        trusted_root: root_handle,
        root,
        legacy,
    })
}

/// Existing-only storage boundary for an independently authorized control
/// publisher. No reconciliation or creation happens before its source guard.
#[cfg(target_os = "linux")]
pub(super) fn with_opened_existing_versioned_control_preflight<T>(
    root_handle: &TrustedAuthorityRoot,
    operation: impl FnOnce(&VersionedSemanticTransaction<'_, '_>) -> Result<T, BootstrapError>,
) -> Result<T, BootstrapError> {
    root_handle
        .revalidate()
        .map_err(|_| BootstrapError("revalidate existing control authority root"))?;
    let lock_scope = StoreLayoutLockScope::open_existing_activated(root_handle.directory())
        .map_err(|_| BootstrapError("open existing activated control authority layout"))?;
    let _lock = lock_scope
        .root_lock
        .lock_exclusive_owned()
        .map_err(|_| BootstrapError("lock existing control authority root"))?;
    lock_scope
        .validate_temps()
        .map_err(|_| BootstrapError("validate existing control authority temps"))?;
    let layout = lock_scope
        .finish()
        .map_err(|_| BootstrapError("open existing control authority layout"))?;
    layout
        .validate_closed_layout()
        .map_err(|_| BootstrapError("validate existing control closed layout"))?;
    let legacy = LegacyObservation::capture(layout.bootstrap)
        .map_err(|_| BootstrapError("capture existing control legacy exclusion"))?;
    if legacy.has_artifact {
        return Err(BootstrapError(
            "legacy state is incompatible with existing control authority",
        ));
    }
    let root = layout
        .read_existing_versioned_without_reconciliation(root_handle.identity())
        .map_err(|_| BootstrapError("read existing control authority root"))?;
    match &root {
        VersionedStateRoot::V1(root) => layout.validate_matching_marker_if_present(root),
        VersionedStateRoot::V2(root) => layout.validate_matching_marker_if_present_v2(root),
        VersionedStateRoot::V3(root) => layout.validate_matching_marker_if_present_v3(root),
    }
    .map_err(|_| BootstrapError("validate existing control initialization marker"))?;
    let result = operation(&VersionedSemanticTransaction {
        layout: &layout,
        trusted_root: root_handle,
        root,
        legacy,
    });
    root_handle
        .revalidate()
        .map_err(|_| BootstrapError("finalize existing control authority root"))?;
    result
}

pub(super) fn with_opened_existing_semantic_preflight<T>(
    root_handle: &TrustedAuthorityRoot,
    operation: impl FnOnce(&SemanticTransaction<'_, '_>) -> Result<T, BootstrapError>,
) -> Result<T, BootstrapError> {
    with_opened_semantic_preflight(
        root_handle,
        SemanticPreflightMode::AuthorityOperation,
        |layout, _, observed, _lock| {
            if observed.classification != BootstrapClassificationV1::ValidExisting {
                return Err(BootstrapError(
                    "authority store is not valid existing state",
                ));
            }
            let root = observed
                .root
                .ok_or(BootstrapError("semantic preflight omitted existing root"))?;
            operation(&SemanticTransaction {
                layout,
                trusted_root: root_handle,
                root,
                legacy: observed.legacy,
            })
        },
    )
}

pub(super) fn compare_and_swap_root_with(
    path: &std::path::Path,
    expected: &ExpectedRevisionsV1,
    proposed: &StateRootV1,
    nonce_bytes: [u8; 16],
) -> Result<TransactionCommitOutcomeV1, BootstrapError> {
    let root_handle = TrustedAuthorityRoot::open(path)
        .map_err(|_| BootstrapError("open trusted authority root"))?;
    compare_and_swap_opened_root_with(&root_handle, expected, proposed, nonce_bytes)
}

pub(super) fn compare_and_swap_opened_root_with(
    root_handle: &TrustedAuthorityRoot,
    expected: &ExpectedRevisionsV1,
    proposed: &StateRootV1,
    nonce_bytes: [u8; 16],
) -> Result<TransactionCommitOutcomeV1, BootstrapError> {
    compare_and_swap_opened_root_with_exact_current(
        root_handle,
        None,
        expected,
        proposed,
        nonce_bytes,
    )
}

pub(super) fn compare_and_swap_opened_root_with_exact_current(
    root_handle: &TrustedAuthorityRoot,
    exact_current: Option<&StateRootV1>,
    expected: &ExpectedRevisionsV1,
    proposed: &StateRootV1,
    nonce_bytes: [u8; 16],
) -> Result<TransactionCommitOutcomeV1, BootstrapError> {
    with_opened_existing_semantic_preflight(root_handle, |transaction| {
        validate_positive_expectations(expected)?;
        validate_proposed_root(&transaction.root, expected, proposed)?;
        if exact_current.is_some_and(|current| transaction.root != *current) {
            return Err(BootstrapError(
                "locked authority root differs from exact observed root",
            ));
        }
        if transaction.root == *proposed {
            validate_exact_retry_expectation(expected, proposed)?;
            transaction.reconcile()?;
            transaction.validate_exact_committed_candidate(expected.root_revision, proposed)?;
            return Ok(TransactionCommitOutcomeV1::JoinedExact(proposed.clone()));
        }
        if transaction.root.root_revision != expected.root_revision {
            return Err(BootstrapError("stale authority root revision"));
        }
        validate_current_authority_expectation(&transaction.root, expected)?;
        validate_authority_changes(&transaction.root, proposed, expected)?;
        transaction.reconcile()?;
        publish_replacement_root(
            transaction.layout,
            transaction.trusted_root,
            &transaction.legacy,
            proposed,
            nonce_bytes,
            || transaction.validate_publication_candidate(expected.root_revision, proposed),
        )?;
        Ok(TransactionCommitOutcomeV1::Committed(proposed.clone()))
    })
}

fn validate_positive_expectations(expected: &ExpectedRevisionsV1) -> Result<(), BootstrapError> {
    if expected.root_revision == 0
        || expected
            .authority
            .as_ref()
            .is_some_and(|authority| authority.authority_revision == 0)
    {
        Err(BootstrapError(
            "CAS expected revisions must be strictly positive",
        ))
    } else {
        Ok(())
    }
}

fn validate_exact_retry_expectation(
    expected: &ExpectedRevisionsV1,
    proposed: &StateRootV1,
) -> Result<(), BootstrapError> {
    match expected.authority.as_ref() {
        None if proposed.session_namespace_map.is_empty()
            && proposed.transition_intent_map.is_empty()
            && proposed.issuer_request_index.is_empty()
            && proposed.application_journal.is_empty()
            && proposed.object_index.is_empty() =>
        {
            Ok(())
        }
        _ => Err(BootstrapError(
            "exact retry authority provenance is ambiguous",
        )),
    }
}

#[cfg(test)]
pub(super) fn validate_exact_retry_expectation_test(
    expected: &ExpectedRevisionsV1,
    proposed: &StateRootV1,
) -> Result<(), BootstrapError> {
    validate_exact_retry_expectation(expected, proposed)
}

fn validate_proposed_root(
    current: &StateRootV1,
    expected: &ExpectedRevisionsV1,
    proposed: &StateRootV1,
) -> Result<(), BootstrapError> {
    let next_revision = expected
        .root_revision
        .checked_add(1)
        .ok_or(BootstrapError("expected root revision overflow"))?;
    if proposed.root_revision != next_revision
        || proposed.authority_store_id != current.authority_store_id
        || proposed.bootstrap_home != current.bootstrap_home
        || proposed.greenfield_namespace_certificate != current.greenfield_namespace_certificate
    {
        return Err(BootstrapError(
            "proposed authority root identity or revision is invalid",
        ));
    }
    proposed
        .validate()
        .map_err(|_| BootstrapError("proposed authority root is invalid"))
}

fn validate_current_authority_expectation(
    current: &StateRootV1,
    expected: &ExpectedRevisionsV1,
) -> Result<(), BootstrapError> {
    let Some(authority) = expected.authority.as_ref() else {
        return Ok(());
    };
    match current
        .session_namespace_map
        .get(&authority.orchestration_session_id)
    {
        Some(SessionNamespaceRecordV1::Authority(value))
            if value.authority_revision == authority.authority_revision =>
        {
            Ok(())
        }
        _ => Err(BootstrapError("stale or missing authority revision")),
    }
}

fn validate_authority_changes(
    current: &StateRootV1,
    proposed: &StateRootV1,
    expected: &ExpectedRevisionsV1,
) -> Result<(), BootstrapError> {
    let target = expected
        .authority
        .as_ref()
        .map(|authority| authority.orchestration_session_id.as_str());
    let keys = current
        .session_namespace_map
        .keys()
        .chain(proposed.session_namespace_map.keys())
        .collect::<std::collections::BTreeSet<_>>();
    for key in keys {
        let before = current.session_namespace_map.get(key);
        let after = proposed.session_namespace_map.get(key);
        let authority_changed = match (before, after) {
            (
                Some(SessionNamespaceRecordV1::Authority(before)),
                Some(SessionNamespaceRecordV1::Authority(after)),
            ) => before != after,
            (Some(SessionNamespaceRecordV1::Authority(_)), _)
            | (_, Some(SessionNamespaceRecordV1::Authority(_))) => true,
            _ => false,
        };
        if authority_changed && target != Some(key.as_str()) {
            return Err(BootstrapError(
                "authority changed without its expected revision",
            ));
        }
    }
    if let Some(authority) = expected.authority.as_ref() {
        let next = authority
            .authority_revision
            .checked_add(1)
            .ok_or(BootstrapError("expected authority revision overflow"))?;
        match proposed
            .session_namespace_map
            .get(&authority.orchestration_session_id)
        {
            Some(SessionNamespaceRecordV1::Authority(value))
                if value.authority_revision == next => {}
            _ => return Err(BootstrapError("proposed authority revision is invalid")),
        }
    }
    Ok(())
}

/// Storage authority is confined to raw inbox ingress and this admitted source.
/// Semantic enrollment, publication and completeness belong to ObligationLedger.
#[cfg(target_os = "linux")]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum DirectHostInboxStorageAreaV1 {
    RawInbox,
    Source,
}

#[cfg(target_os = "linux")]
pub(crate) struct DirectHostInboxTransactionV1<'layout, 'root> {
    read_only: bool,
    trusted_root: &'root TrustedAuthorityRoot,
    layout: &'layout StoreLayout<'root>,
    locked_root: Option<VersionedStateRoot>,
    legacy: LegacyObservation,
    authority_entry: DirectoryEntry,
    lock_entry: DirectoryEntry,
    root_lock_entry: DirectoryEntry,
    directories: RetainedPostHsaLedgerDirectories,
    files: BTreeMap<Vec<String>, Option<Vec<u8>>>,
    manifests: BTreeMap<Vec<String>, Vec<DirectoryEntry>>,
}

#[cfg(target_os = "linux")]
impl DirectHostInboxTransactionV1<'_, '_> {
    pub(crate) fn bootstrap_home(&self) -> &CanonicalDirectoryV1 {
        self.trusted_root.identity()
    }

    pub(crate) fn authority_root(&self) -> Option<&VersionedStateRoot> {
        self.locked_root.as_ref()
    }

    fn components(
        area: DirectHostInboxStorageAreaV1,
        relative: &[&str],
    ) -> Result<Vec<String>, BootstrapError> {
        let prefix: &[&str] = match area {
            DirectHostInboxStorageAreaV1::RawInbox => &["host_inbox"],
            DirectHostInboxStorageAreaV1::Source => &["direct-host-inbox-ledger-v1"],
        };
        for component in relative {
            validate_post_hsa_obligation_ledger_component(component)?;
        }
        Ok(prefix
            .iter()
            .chain(relative)
            .map(|s| (*s).to_string())
            .collect())
    }

    fn read_path(&mut self, path: &[String]) -> Result<Option<Vec<u8>>, BootstrapError> {
        let (target, parents) = path
            .split_last()
            .ok_or(BootstrapError("direct-host inbox path is empty"))?;
        self.directories
            .with_directory(
                self.trusted_root.directory(),
                &parents.iter().map(String::as_str).collect::<Vec<_>>(),
                false,
                |parent, _| direct_host_inbox_read_bounded_file(parent, target),
            )
            .map(Option::flatten)
    }

    pub(crate) fn read_file(
        &mut self,
        area: DirectHostInboxStorageAreaV1,
        relative: &[&str],
    ) -> Result<Option<Vec<u8>>, BootstrapError> {
        self.verify()?;
        let path = Self::components(area, relative)?;
        let bytes = self.read_path(&path)?;
        self.files.insert(path, bytes.clone());
        Ok(bytes)
    }

    pub(crate) fn list_files(
        &mut self,
        area: DirectHostInboxStorageAreaV1,
        relative: &[&str],
    ) -> Result<Vec<(String, Vec<u8>)>, BootstrapError> {
        self.verify()?;
        let path = Self::components(area, relative)?;
        let entries = self
            .directories
            .with_directory(
                self.trusted_root.directory(),
                &path.iter().map(String::as_str).collect::<Vec<_>>(),
                false,
                |directory, _| {
                    directory
                        .entries()
                        .map_err(|_| BootstrapError("enumerate direct-host inbox source"))
                },
            )?
            .unwrap_or_default();
        if entries.len() > 16384 || entries.iter().any(|e| e.kind != EntryKind::RegularFile) {
            return Err(BootstrapError(
                "direct-host inbox file collection is unsafe or oversized",
            ));
        }
        self.manifests.insert(path.clone(), entries.clone());
        let mut result = Vec::with_capacity(entries.len());
        let mut total = 0_usize;
        for entry in entries {
            let mut file = path.clone();
            file.push(entry.name.clone());
            let bytes = self.read_path(&file)?.ok_or(BootstrapError(
                "enumerated direct-host inbox artifact disappeared",
            ))?;
            total = total
                .checked_add(bytes.len())
                .ok_or(BootstrapError("direct-host inbox source size overflow"))?;
            if total > 32 * 1024 * 1024 {
                return Err(BootstrapError(
                    "direct-host inbox source exceeds its byte bound",
                ));
            }
            self.files.insert(file, Some(bytes.clone()));
            result.push((entry.name, bytes));
        }
        result.sort_by(|a, b| a.0.cmp(&b.0));
        self.verify()?;
        Ok(result)
    }

    /// Compare and replace one scoped artifact. Passing None requires absence;
    /// immutable publications use that mode and join exact retries in the ledger.
    pub(crate) fn write_file(
        &mut self,
        area: DirectHostInboxStorageAreaV1,
        relative: &[&str],
        expected: Option<&[u8]>,
        bytes: &[u8],
    ) -> Result<(), BootstrapError> {
        if self.read_only {
            return Err(BootstrapError(
                "direct-host snapshot has no mutation authority",
            ));
        }
        if bytes.len() > 32 * 1024 * 1024 {
            return Err(BootstrapError(
                "direct-host inbox artifact exceeds its byte bound",
            ));
        }
        self.verify()?;
        let path = Self::components(area, relative)?;
        if self.read_path(&path)?.as_deref() != expected {
            return Err(BootstrapError(
                "stale or conflicting direct-host inbox artifact",
            ));
        }
        let current_bytes = self.source_tree_bytes()?;
        // The complete replacement temp coexists with the old artifact until
        // rename. Bound that publication peak before allocating the temp.
        let publication_bytes = current_bytes
            .checked_add(bytes.len())
            .ok_or(BootstrapError("direct-host inbox source size overflow"))?;
        if publication_bytes > 32 * 1024 * 1024 {
            return Err(BootstrapError(
                "direct-host inbox source exceeds its byte bound",
            ));
        }
        let (target, parents) = path
            .split_last()
            .ok_or(BootstrapError("direct-host inbox path is empty"))?;
        let new_entries = self
            .directories
            .with_directory(
                self.trusted_root.directory(),
                &parents.iter().map(String::as_str).collect::<Vec<_>>(),
                true,
                |parent, _| {
                    let mut entropy = [0_u8; 16];
                    rand::rngs::OsRng.fill_bytes(&mut entropy);
                    let temp_name = format!(".direct-host-inbox--{}.tmp", nonce(entropy));
                    let mut temp = parent
                        .create_file(&temp_name)
                        .map_err(|_| BootstrapError("create direct-host inbox publication temp"))?;
                    temp.write_all(bytes)
                        .and_then(|()| temp.sync())
                        .map_err(|_| {
                            BootstrapError("write and sync direct-host inbox publication")
                        })?;
                    parent
                        .rename_replace(&temp_name, temp, parent, target)
                        .map_err(|_| BootstrapError("publish direct-host inbox artifact"))?;
                    parent.entries().map_err(|_| {
                        BootstrapError("re-enumerate direct-host inbox publication parent")
                    })
                },
            )?
            .ok_or(BootstrapError(
                "direct-host inbox publication parent is absent",
            ))?;
        if let Some(old_entries) = self.manifests.get(parents) {
            let old_other = old_entries
                .iter()
                .filter(|e| &e.name != target)
                .collect::<Vec<_>>();
            let new_other = new_entries
                .iter()
                .filter(|e| &e.name != target)
                .collect::<Vec<_>>();
            if old_other != new_other {
                return Err(BootstrapError(
                    "direct-host inbox namespace changed during publication",
                ));
            }
            self.manifests.insert(parents.to_vec(), new_entries);
        }
        self.files.insert(path, Some(bytes.to_vec()));
        self.trusted_root
            .directory()
            .sync()
            .map_err(|_| BootstrapError("sync direct-host inbox source root"))?;
        self.verify()
    }

    fn source_tree_bytes(&mut self) -> Result<usize, BootstrapError> {
        let mut total = 0_usize;
        for area in [
            DirectHostInboxStorageAreaV1::RawInbox,
            DirectHostInboxStorageAreaV1::Source,
        ] {
            let components = Self::components(area, &[])?;
            let bytes = self
                .directories
                .with_directory(
                    self.trusted_root.directory(),
                    &components.iter().map(String::as_str).collect::<Vec<_>>(),
                    false,
                    |directory, _| direct_host_inbox_tree_bytes(directory, &mut 0, 0),
                )?
                .unwrap_or(0);
            total = total
                .checked_add(bytes)
                .ok_or(BootstrapError("direct-host inbox source size overflow"))?;
            if total > 32 * 1024 * 1024 {
                return Err(BootstrapError(
                    "direct-host inbox source exceeds its byte bound",
                ));
            }
        }
        Ok(total)
    }

    pub(crate) fn verify(&mut self) -> Result<(), BootstrapError> {
        revalidate_world_work_receipt_root_lock_scope(
            self.trusted_root,
            &self.authority_entry,
            &self.layout.authority,
            &self.lock_entry,
            &self.layout.lock,
            &self.root_lock_entry,
        )?;
        self.directories.revalidate(self.trusted_root.directory())?;
        if let Some(expected) = &self.locked_root {
            let current = self
                .layout
                .read_existing_versioned_without_reconciliation(self.trusted_root.identity())
                .map_err(|_| BootstrapError("reread direct-host inbox authority root"))?;
            if &current != expected {
                return Err(BootstrapError("direct-host inbox authority root changed"));
            }
            self.legacy
                .revalidate(self.layout.bootstrap)
                .map_err(|_| BootstrapError("direct-host inbox legacy exclusions changed"))?;
        } else if self
            .layout
            .authority
            .entry_kind(ROOT_FILE)
            .map_err(|_| BootstrapError("revalidate preactivation inbox authority absence"))?
            .is_some()
            || self
                .layout
                .authority
                .entry_kind(INIT_FILE)
                .map_err(|_| BootstrapError("revalidate preactivation inbox marker absence"))?
                .is_some()
        {
            return Err(BootstrapError(
                "authority activation raced preactivation inbox ingress",
            ));
        }
        for (path, expected) in self.files.clone() {
            if self.read_path(&path)? != expected {
                return Err(BootstrapError(
                    "retained direct-host inbox artifact changed",
                ));
            }
        }
        for (path, expected) in self.manifests.clone() {
            let actual = self
                .directories
                .with_directory(
                    self.trusted_root.directory(),
                    &path.iter().map(String::as_str).collect::<Vec<_>>(),
                    false,
                    |parent, _| {
                        parent
                            .entries()
                            .map_err(|_| BootstrapError("revalidate direct-host inbox namespace"))
                    },
                )?
                .unwrap_or_default();
            if actual != expected {
                return Err(BootstrapError(
                    "retained direct-host inbox namespace changed",
                ));
            }
        }
        Ok(())
    }

    pub(in crate::execution::agent_runtime::host_session_authority) fn read_authority_object(
        &mut self,
        reference: &AuthorityObjectRefV1,
    ) -> Result<Vec<u8>, BootstrapError> {
        self.verify()?;
        let VersionedStateRoot::V3(root) = self
            .locked_root
            .as_ref()
            .ok_or(BootstrapError("direct-host Resume has no authority root"))?
        else {
            return Err(BootstrapError(
                "direct-host Resume requires strict StateRootV3",
            ));
        };
        let bytes = self
            .layout
            .read_object_bytes(reference)
            .map_err(|_| BootstrapError("read direct-host Resume typed object"))?;
        verify_object_bytes(self.layout, root, reference, &bytes, None, false)?;
        Ok(bytes)
    }

    pub(in crate::execution::agent_runtime::host_session_authority) fn prepare_authority_object(
        &mut self,
        kind: AuthorityObjectKindV1,
        bytes: &[u8],
    ) -> Result<GeneratedObjectV1, BootstrapError> {
        if self.read_only {
            return Err(BootstrapError(
                "direct-host snapshot has no mutation authority",
            ));
        }
        self.verify()?;
        let VersionedStateRoot::V3(root) = self
            .locked_root
            .as_ref()
            .ok_or(BootstrapError("direct-host Resume has no authority root"))?
        else {
            return Err(BootstrapError(
                "direct-host Resume requires strict StateRootV3",
            ));
        };
        if !matches!(
            kind,
            AuthorityObjectKindV1::PostTurnProtocolEvent
                | AuthorityObjectKindV1::PostTurnCompletion
                | AuthorityObjectKindV1::ApplicationResult
                | AuthorityObjectKindV1::TerminalHandoff
                | AuthorityObjectKindV1::ResumeHandle
        ) {
            return Err(BootstrapError(
                "direct-host settlement bridge refuses unrelated object kind",
            ));
        }
        if kind == AuthorityObjectKindV1::ResumeHandle
            && super::object_persistence::canonical_schema_version(bytes)? != 1
        {
            return Err(BootstrapError(
                "direct-host settlement produces only ResumeHandle schema1",
            ));
        }
        let mut reference = generated_object_ref_v3(self.layout, root, kind, bytes, None)?;
        reference.schema_version = super::object_persistence::canonical_schema_version(bytes)?;
        publish_or_join_orphan(
            self.layout,
            root,
            &reference,
            bytes,
            None,
            system_material()?.key_nonce,
        )?;
        self.verify()?;
        Ok(GeneratedObjectV1 {
            reference,
            byte_length: bytes.len() as u64,
        })
    }

    /// Physical source applicability observation only. Existing canonical
    /// protocols retain their bytes and meanings; ledger decides unknown/refusal.
    pub(crate) fn direct_host_prior_canonical_source_present(
        &mut self,
        session_id: &str,
    ) -> Result<bool, BootstrapError> {
        validate_post_hsa_obligation_ledger_component(session_id)?;
        self.verify()?;
        let mut present = false;
        for components in [
            vec!["run", "agent-hub", "sessions", session_id, "obligations"],
            vec!["obligation-ledger", session_id],
        ] {
            present |= self
                .directories
                .with_directory(
                    self.trusted_root.directory(),
                    &components,
                    false,
                    |directory, _| {
                        directory.metadata().map_err(|_| {
                            BootstrapError("validate prior canonical source directory")
                        })?;
                        Ok(())
                    },
                )?
                .is_some();
        }
        self.verify()?;
        Ok(present)
    }

    /// Verify the actual helper token against this exact Resume's store HMAC.
    /// Token and prompt bytes stay inside HSA and never reach ledger callers.
    pub(in crate::execution::agent_runtime::host_session_authority) fn verify_direct_host_resume_input_and_lease(
        &mut self,
        intent: &crate::execution::agent_runtime::host_session_authority::store_schema::HostSessionTransitionIntentV3,
        expected_token: &[u8],
        expected_input: &[u8],
    ) -> Result<(), BootstrapError> {
        self.verify()?;
        let Some(VersionedStateRoot::V3(root)) = self.locked_root.as_ref() else {
            return Err(BootstrapError(
                "direct-host helper token requires V3 authority",
            ));
        };
        if root.successor_transition_intent_map.get(&intent.intent_id) != Some(intent) {
            return Err(BootstrapError("direct-host helper token intent changed"));
        }
        let reference = &intent.target_participant_lease_token_ref;
        if reference.object_kind != AuthorityObjectKindV1::LeaseToken {
            return Err(BootstrapError(
                "direct-host helper token has the wrong kind",
            ));
        }
        let context = ObjectVerificationContextV1 {
            intent_id: intent.intent_id.clone(),
            run_id: intent.run_id.clone(),
            parent_intent: None,
        };
        let bytes = self
            .layout
            .read_object_bytes(reference)
            .map_err(|_| BootstrapError("read exact direct-host helper token"))?;
        verify_object_bytes(self.layout, root, reference, &bytes, Some(&context), false)?;
        if bytes != expected_token {
            return Err(BootstrapError(
                "direct-host helper token differs from applied intent",
            ));
        }
        let input_ref = intent
            .transition_input_ref
            .as_ref()
            .filter(|r| r.object_kind == AuthorityObjectKindV1::TransitionInput)
            .ok_or(BootstrapError(
                "direct-host helper input reference is absent",
            ))?;
        let input = self
            .layout
            .read_object_bytes(input_ref)
            .map_err(|_| BootstrapError("read exact direct-host helper input"))?;
        verify_object_bytes(self.layout, root, input_ref, &input, Some(&context), false)?;
        if input != expected_input {
            return Err(BootstrapError(
                "direct-host helper prompt differs from accepted input",
            ));
        }
        self.verify()
    }

    /// The source cut remains retained through the same physical-root commit.
    /// Ledger callers cannot use this authority-only operation.
    pub(in crate::execution::agent_runtime::host_session_authority) fn commit_authority_root(
        &mut self,
        exact_current: &StateRootV3,
        proposed: &StateRootV3,
        mut authorization_guard: impl FnMut() -> Result<(), BootstrapError>,
    ) -> Result<StateRootV3, BootstrapError> {
        authorization_guard()?;
        if self.read_only {
            return Err(BootstrapError(
                "direct-host snapshot has no mutation authority",
            ));
        }
        self.verify()?;
        if self.locked_root.as_ref() != Some(&VersionedStateRoot::V3(exact_current.clone())) {
            return Err(BootstrapError(
                "direct-host Resume exact authority root changed",
            ));
        }
        let semantic = VersionedSemanticTransaction {
            layout: self.layout,
            trusted_root: self.trusted_root,
            root: VersionedStateRoot::V3(exact_current.clone()),
            legacy: LegacyObservation::capture(self.layout.bootstrap)
                .map_err(|_| BootstrapError("capture direct-host Resume publication exclusions"))?,
        };
        let candidate = VersionedStateRoot::V3(proposed.clone());
        semantic.validate_publication_candidate(exact_current.root_revision, &candidate)?;
        semantic.reconcile()?;
        publish_versioned_replacement_root(
            self.layout,
            self.trusted_root,
            &semantic.legacy,
            &candidate,
            system_material()?.root_nonce,
            || {
                authorization_guard()?;
                self.verify()?;
                semantic.validate_publication_candidate(exact_current.root_revision, &candidate)
            },
        )?;
        self.locked_root = Some(candidate);
        self.verify()?;
        Ok(proposed.clone())
    }
}

#[cfg(target_os = "linux")]
fn direct_host_inbox_read_bounded_file(
    parent: &TrustedDirectory,
    target: &str,
) -> Result<Option<Vec<u8>>, BootstrapError> {
    use std::io::Read as _;
    use std::os::fd::{AsRawFd as _, FromRawFd as _};
    use std::os::unix::fs::MetadataExt as _;
    validate_post_hsa_obligation_ledger_component(target)?;
    let entry = parent
        .entries()
        .map_err(|_| BootstrapError("enumerate bounded direct-host inbox artifact"))?
        .into_iter()
        .find(|e| e.name == target);
    let Some(entry) = entry else {
        return Ok(None);
    };
    if entry.kind != EntryKind::RegularFile {
        return Err(BootstrapError(
            "direct-host inbox artifact is not a regular file",
        ));
    }
    // Validate the trusted entry before opening the descriptor used for the
    // bounded read. Named and opened metadata are compared again afterward.
    parent
        .open_file_entry(&entry)
        .map_err(|_| BootstrapError("validate direct-host inbox artifact"))?;
    let name = std::ffi::CString::new(target)
        .map_err(|_| BootstrapError("invalid direct-host inbox artifact name"))?;
    // SAFETY: parent owns a live descriptor; name is a validated single component.
    let fd = unsafe {
        libc::openat(
            parent.borrow_fd().as_raw_fd(),
            name.as_ptr(),
            libc::O_RDONLY | libc::O_CLOEXEC | libc::O_NOFOLLOW | libc::O_NONBLOCK,
        )
    };
    if fd < 0 {
        return Err(BootstrapError("open bounded direct-host inbox artifact"));
    }
    // SAFETY: openat returned a new owned descriptor exactly once.
    let file = unsafe { std::fs::File::from_raw_fd(fd) };
    let before = file
        .metadata()
        .map_err(|_| BootstrapError("inspect bounded direct-host inbox artifact"))?;
    if !before.is_file()
        || before.nlink() != 1
        || before.len() > 32 * 1024 * 1024
        || before.mode() & 0o077 != 0
        || before.uid() != unsafe { libc::geteuid() }
    {
        return Err(BootstrapError(
            "direct-host inbox artifact is unsafe or oversized",
        ));
    }
    let mut bytes = Vec::new();
    (&file)
        .take(32 * 1024 * 1024 + 1)
        .read_to_end(&mut bytes)
        .map_err(|_| BootstrapError("read bounded direct-host inbox artifact"))?;
    let after = file
        .metadata()
        .map_err(|_| BootstrapError("reinspect bounded direct-host inbox artifact"))?;
    let identity = |m: &std::fs::Metadata| {
        (
            m.dev(),
            m.ino(),
            m.uid(),
            m.mode(),
            m.nlink(),
            m.len(),
            m.mtime(),
            m.mtime_nsec(),
            m.ctime(),
            m.ctime_nsec(),
        )
    };
    let mut named = std::mem::MaybeUninit::<libc::stat>::uninit();
    // SAFETY: descriptor/name are live, and fstatat initializes stat on success.
    if unsafe {
        libc::fstatat(
            parent.borrow_fd().as_raw_fd(),
            name.as_ptr(),
            named.as_mut_ptr(),
            libc::AT_SYMLINK_NOFOLLOW,
        )
    } != 0
    {
        return Err(BootstrapError("reinspect named direct-host inbox artifact"));
    }
    // SAFETY: successful fstatat initialized this value.
    let named = unsafe { named.assume_init() };
    if identity(&before) != identity(&after)
        || bytes.len() > 32 * 1024 * 1024
        || after.dev() != named.st_dev
        || after.ino() != named.st_ino
        || after.len() != named.st_size as u64
        || after.nlink() != named.st_nlink
        || after.uid() != named.st_uid
        || after.mode() != named.st_mode
        || after.mtime() != named.st_mtime
        || after.mtime_nsec() != named.st_mtime_nsec
        || after.ctime() != named.st_ctime
        || after.ctime_nsec() != named.st_ctime_nsec
    {
        return Err(BootstrapError(
            "direct-host inbox artifact changed during bounded read",
        ));
    }
    parent
        .revalidate_entry(&entry)
        .map_err(|_| BootstrapError("direct-host inbox artifact identity changed"))?;
    Ok(Some(bytes))
}

#[cfg(target_os = "linux")]
pub(in crate::execution::agent_runtime::host_session_authority::store) fn with_locked_direct_host_inbox_transaction_opened<
    T,
>(
    root: &TrustedAuthorityRoot,
    operation: impl FnOnce(&mut DirectHostInboxTransactionV1<'_, '_>) -> Result<T, BootstrapError>,
) -> Result<T, BootstrapError> {
    with_direct_host_inbox_scope(root, false, None, operation)
}

#[cfg(target_os = "linux")]
pub(in crate::execution::agent_runtime::host_session_authority::store) fn with_existing_direct_host_inbox_snapshot_opened<
    T,
>(
    root: &TrustedAuthorityRoot,
    operation: impl FnOnce(&mut DirectHostInboxTransactionV1<'_, '_>) -> Result<T, BootstrapError>,
) -> Result<T, BootstrapError> {
    with_direct_host_inbox_scope(root, true, None, operation)
}

#[cfg(target_os = "linux")]
fn with_direct_host_inbox_scope<T>(
    root: &TrustedAuthorityRoot,
    read_only: bool,
    recovery_lock: Option<&DirectHostResumeRecoveryReadV1<'_>>,
    operation: impl FnOnce(&mut DirectHostInboxTransactionV1<'_, '_>) -> Result<T, BootstrapError>,
) -> Result<T, BootstrapError> {
    root.revalidate()
        .map_err(|_| BootstrapError("revalidate direct-host inbox physical root"))?;
    let scope = if read_only {
        StoreLayoutLockScope::open_existing_activated(root.directory())
    } else {
        StoreLayoutLockScope::open(root.directory())
    }
    .map_err(|_| BootstrapError("open direct-host inbox physical-root lock"))?;
    let authority_entry =
        RetainedLegacyDirectories::directory_entry(root.directory(), AUTHORITY_DIRECTORY)?;
    let lock_entry = RetainedLegacyDirectories::directory_entry(&scope.authority, "lock")?;
    let root_lock_entry = scope
        .lock
        .entries()
        .map_err(|_| BootstrapError("enumerate direct-host inbox root lock"))?
        .into_iter()
        .find(|e| e.name == ROOT_LOCK_FILE && e.kind == EntryKind::RegularFile)
        .ok_or(BootstrapError(
            "direct-host inbox root lock identity is absent",
        ))?;
    direct_host_inbox_verify_open_directory(root.directory(), &authority_entry, &scope.authority)?;
    direct_host_inbox_verify_open_directory(&scope.authority, &lock_entry, &scope.lock)?;
    let root_lock_file = scope
        .lock
        .open_file_entry(&root_lock_entry)
        .map_err(|_| BootstrapError("retain exact direct-host inbox root lock file"))?;
    let _lock = if let Some(held) = recovery_lock {
        held.verify()?;
        if held.root.identity() != root.identity()
            || held.authority_entry != authority_entry
            || held.lock_entry != lock_entry
            || held.root_lock_entry != root_lock_entry
        {
            return Err(BootstrapError(
                "direct-host recovery lock does not match active scope",
            ));
        }
        // The sealed recovery scope owns the exact open-file-description lock
        // until this operation returns. Re-locking a separately opened fd would
        // deadlock; callers cannot construct or release this private permit.
        None
    } else {
        Some(
            root_lock_file
                .lock_exclusive_owned()
                .map_err(|_| BootstrapError("lock exact direct-host inbox physical root"))?,
        )
    };
    revalidate_world_work_receipt_root_lock_scope(
        root,
        &authority_entry,
        &scope.authority,
        &lock_entry,
        &scope.lock,
        &root_lock_entry,
    )?;
    let activated = scope
        .authority_activated()
        .map_err(|_| BootstrapError("inspect direct-host inbox authority activation"))?;
    if read_only && !activated {
        return Err(BootstrapError(
            "existing direct-host snapshot authority disappeared",
        ));
    }
    scope
        .validate_temps()
        .map_err(|_| BootstrapError("validate direct-host inbox authority temps"))?;
    let layout = scope
        .finish()
        .map_err(|_| BootstrapError("open direct-host inbox authority layout"))?;
    layout
        .validate_closed_layout()
        .map_err(|_| BootstrapError("validate direct-host inbox authority layout"))?;
    let legacy = LegacyObservation::capture(layout.bootstrap)
        .map_err(|_| BootstrapError("capture direct-host inbox legacy exclusions"))?;
    let locked_root = if activated {
        if legacy.has_artifact {
            return Err(BootstrapError(
                "legacy state is incompatible with activated direct-host inbox ingress",
            ));
        }
        let current = layout
            .read_existing_versioned_without_reconciliation(root.identity())
            .map_err(|_| BootstrapError("read direct-host inbox authority root"))?;
        match &current {
            VersionedStateRoot::V1(value) => layout.validate_matching_marker_if_present(value),
            VersionedStateRoot::V2(value) => layout.validate_matching_marker_if_present_v2(value),
            VersionedStateRoot::V3(value) => layout.validate_matching_marker_if_present_v3(value),
        }
        .map_err(|_| BootstrapError("validate direct-host inbox authority marker"))?;
        Some(current)
    } else {
        let observed = layout
            .semantic_preflight(root.identity())
            .map_err(|_| BootstrapError("classify preactivation direct-host inbox ingress"))?;
        if observed.authority_classification != BootstrapClassificationV1::FreshAbsent {
            return Err(BootstrapError(
                "direct-host inbox authority initialization is incomplete",
            ));
        }
        None
    };
    let mut transaction = DirectHostInboxTransactionV1 {
        read_only,
        trusted_root: root,
        layout: &layout,
        locked_root,
        legacy,
        authority_entry,
        lock_entry,
        root_lock_entry,
        directories: RetainedPostHsaLedgerDirectories::default(),
        files: BTreeMap::new(),
        manifests: BTreeMap::new(),
    };
    transaction.verify()?;
    let result = operation(&mut transaction);
    transaction.verify()?;
    result
}

#[cfg(target_os = "linux")]
fn direct_host_inbox_tree_bytes(
    directory: &TrustedDirectory,
    count: &mut usize,
    depth: usize,
) -> Result<usize, BootstrapError> {
    use std::os::fd::AsRawFd as _;
    if depth > 16 {
        return Err(BootstrapError(
            "direct-host inbox inventory exceeds its depth bound",
        ));
    }
    let entries = directory
        .entries()
        .map_err(|_| BootstrapError("enumerate direct-host inbox size inventory"))?;
    let mut total = 0_usize;
    for entry in &entries {
        *count = count
            .checked_add(1)
            .ok_or(BootstrapError("direct-host inbox inventory size overflow"))?;
        if *count > 16384 {
            return Err(BootstrapError(
                "direct-host inbox inventory exceeds its entry bound",
            ));
        }
        let bytes = match entry.kind {
            EntryKind::Directory => {
                let child = directory
                    .open_controlled_directory_entry(entry)
                    .map_err(|_| BootstrapError("open direct-host inbox size inventory"))?;
                direct_host_inbox_tree_bytes(&child, count, depth + 1)?
            }
            EntryKind::RegularFile => {
                directory
                    .open_file_entry(entry)
                    .map_err(|_| BootstrapError("validate direct-host inbox size artifact"))?;
                let name = std::ffi::CString::new(entry.name.as_str())
                    .map_err(|_| BootstrapError("invalid direct-host inbox size artifact name"))?;
                let mut metadata = std::mem::MaybeUninit::<libc::stat>::uninit();
                // SAFETY: directory/name are live and fstatat initializes metadata on success.
                if unsafe {
                    libc::fstatat(
                        directory.borrow_fd().as_raw_fd(),
                        name.as_ptr(),
                        metadata.as_mut_ptr(),
                        libc::AT_SYMLINK_NOFOLLOW,
                    )
                } != 0
                {
                    return Err(BootstrapError("inspect direct-host inbox source size"));
                }
                // SAFETY: successful fstatat initialized this value.
                let metadata = unsafe { metadata.assume_init() };
                if metadata.st_mode & libc::S_IFMT != libc::S_IFREG
                    || metadata.st_nlink != 1
                    || metadata.st_mode & 0o077 != 0
                    || metadata.st_uid != unsafe { libc::geteuid() }
                    || metadata.st_size < 0
                {
                    return Err(BootstrapError("direct-host inbox size artifact is unsafe"));
                }
                usize::try_from(metadata.st_size)
                    .map_err(|_| BootstrapError("direct-host inbox artifact size overflow"))?
            }
            _ => {
                return Err(BootstrapError(
                    "direct-host inbox size inventory contains an unsafe entry",
                ))
            }
        };
        total = total
            .checked_add(bytes)
            .ok_or(BootstrapError("direct-host inbox source size overflow"))?;
        if total > 32 * 1024 * 1024 {
            return Err(BootstrapError(
                "direct-host inbox source exceeds its byte bound",
            ));
        }
        directory
            .revalidate_entry(entry)
            .map_err(|_| BootstrapError("direct-host inbox size entry changed identity"))?;
    }
    if directory
        .entries()
        .map_err(|_| BootstrapError("re-enumerate direct-host inbox size inventory"))?
        != entries
    {
        return Err(BootstrapError("direct-host inbox size inventory changed"));
    }
    Ok(total)
}

#[cfg(target_os = "linux")]
fn direct_host_inbox_verify_open_directory(
    parent: &TrustedDirectory,
    expected: &DirectoryEntry,
    opened: &TrustedDirectory,
) -> Result<(), BootstrapError> {
    use std::os::fd::AsRawFd as _;
    parent
        .revalidate_entry(expected)
        .map_err(|_| BootstrapError("direct-host inbox named directory changed"))?;
    let name = std::ffi::CString::new(expected.name.as_str())
        .map_err(|_| BootstrapError("invalid direct-host inbox directory name"))?;
    let mut named = std::mem::MaybeUninit::<libc::stat>::uninit();
    let mut held = std::mem::MaybeUninit::<libc::stat>::uninit();
    // SAFETY: both descriptors and the single-component name remain live;
    // successful stat calls initialize the respective output values.
    if unsafe {
        libc::fstatat(
            parent.borrow_fd().as_raw_fd(),
            name.as_ptr(),
            named.as_mut_ptr(),
            libc::AT_SYMLINK_NOFOLLOW,
        )
    } != 0
        || unsafe { libc::fstat(opened.borrow_fd().as_raw_fd(), held.as_mut_ptr()) } != 0
    {
        return Err(BootstrapError(
            "inspect retained direct-host inbox directory",
        ));
    }
    // SAFETY: both successful stat calls initialized these values.
    let (named, held) = unsafe { (named.assume_init(), held.assume_init()) };
    if named.st_dev != held.st_dev
        || named.st_ino != held.st_ino
        || named.st_mode & libc::S_IFMT != libc::S_IFDIR
        || held.st_mode & libc::S_IFMT != libc::S_IFDIR
    {
        return Err(BootstrapError(
            "direct-host inbox directory descriptor differs from its name",
        ));
    }
    Ok(())
}

/// Minimal existing-root read authority for an exact committed Resume retry.
/// The root is strictly decoded but intentionally NOT generally preflighted.
/// HSA must verify the selected committed intent/result before returning Some;
/// this type supplies no writer and no key/input/ledger access.
#[cfg(target_os = "linux")]
pub(in crate::execution::agent_runtime::host_session_authority) struct DirectHostResumeRecoveryReadV1<
    'root,
> {
    root: &'root TrustedAuthorityRoot,
    authority: TrustedDirectory,
    lock: TrustedDirectory,
    authority_entry: DirectoryEntry,
    lock_entry: DirectoryEntry,
    root_lock_entry: DirectoryEntry,
    root_entry: DirectoryEntry,
    _held_lock: TrustedOwnedFileLock,
    root_bytes: Vec<u8>,
    state: StateRootV3,
    directories: RetainedPostHsaLedgerDirectories,
    files: BTreeMap<Vec<String>, Vec<u8>>,
}

#[cfg(target_os = "linux")]
impl<'root> DirectHostResumeRecoveryReadV1<'root> {
    fn open(root: &'root TrustedAuthorityRoot) -> Result<Self, BootstrapError> {
        root.revalidate()
            .map_err(|_| BootstrapError("revalidate committed Resume physical root"))?;
        let authority_entry =
            RetainedLegacyDirectories::directory_entry(root.directory(), AUTHORITY_DIRECTORY)?;
        let authority = root
            .directory()
            .open_directory(AUTHORITY_DIRECTORY)
            .map_err(|_| BootstrapError("open existing committed Resume authority"))?;
        let lock_entry = RetainedLegacyDirectories::directory_entry(&authority, "lock")?;
        let lock = authority
            .open_directory("lock")
            .map_err(|_| BootstrapError("open existing committed Resume lock directory"))?;
        direct_host_inbox_verify_open_directory(root.directory(), &authority_entry, &authority)?;
        direct_host_inbox_verify_open_directory(&authority, &lock_entry, &lock)?;
        let root_lock_entry = lock
            .entries()
            .map_err(|_| BootstrapError("inspect committed Resume root lock"))?
            .into_iter()
            .find(|e| e.name == ROOT_LOCK_FILE && e.kind == EntryKind::RegularFile)
            .ok_or(BootstrapError("committed Resume root lock is absent"))?;
        let lock_file = lock
            .open_file_entry(&root_lock_entry)
            .map_err(|_| BootstrapError("retain committed Resume root lock"))?;
        let held = lock_file
            .lock_exclusive_owned()
            .map_err(|_| BootstrapError("lock committed Resume root"))?;
        revalidate_world_work_receipt_root_lock_scope(
            root,
            &authority_entry,
            &authority,
            &lock_entry,
            &lock,
            &root_lock_entry,
        )?;
        let root_entry = authority
            .entries()
            .map_err(|_| BootstrapError("inspect committed Resume state root"))?
            .into_iter()
            .find(|e| e.name == ROOT_FILE && e.kind == EntryKind::RegularFile)
            .ok_or(BootstrapError("committed Resume state root is absent"))?;
        let root_bytes = direct_host_inbox_read_bounded_file(&authority, ROOT_FILE)?
            .ok_or(BootstrapError("committed Resume state root disappeared"))?;
        let state: StateRootV3 = canonical_json::from_slice(&root_bytes)
            .map_err(|_| BootstrapError("strict committed Resume V3 root decoding failed"))?;
        if state.schema_version!=3 || state.root_revision==0 || state.bootstrap_home!=*root.identity()
            || state.greenfield_namespace_certificate.schema_version!=1
            || state.greenfield_namespace_certificate.authority_store_id!=state.authority_store_id
            || state.greenfield_namespace_certificate.bootstrap_home!=state.bootstrap_home
            || crate::execution::agent_runtime::host_session_authority::store_format::validate_store_id(&state.authority_store_id).is_err() {
            return Err(BootstrapError("committed Resume root physical/store header disagrees"));
        }
        let read = Self {
            root,
            authority,
            lock,
            authority_entry,
            lock_entry,
            root_lock_entry,
            root_entry,
            _held_lock: held,
            root_bytes,
            state,
            directories: RetainedPostHsaLedgerDirectories::default(),
            files: BTreeMap::new(),
        };
        read.verify()?;
        Ok(read)
    }

    pub(in crate::execution::agent_runtime::host_session_authority) fn state(
        &self,
    ) -> &StateRootV3 {
        &self.state
    }

    /// Canonical public protocol objects only. HMAC/secret objects, raw inputs
    /// and all current external source namespaces are unreachable from here.
    pub(in crate::execution::agent_runtime::host_session_authority) fn read_canonical_object(
        &mut self,
        reference: &AuthorityObjectRefV1,
    ) -> Result<Vec<u8>, BootstrapError> {
        self.verify()?;
        let allowed = matches!(
            (reference.object_kind, reference.schema_version),
            (AuthorityObjectKindV1::PostTurnCompletion, 3)
                | (
                    AuthorityObjectKindV1::PostTurnCompletion
                        | AuthorityObjectKindV1::PostTurnProtocolEvent,
                    2,
                )
                | (
                    AuthorityObjectKindV1::AgentDescriptor
                        | AuthorityObjectKindV1::ApplicationResult
                        | AuthorityObjectKindV1::InputAcceptance
                        | AuthorityObjectKindV1::HostAttachContract
                        | AuthorityObjectKindV1::Policy
                        | AuthorityObjectKindV1::TerminalHandoff,
                    1,
                )
                | (AuthorityObjectKindV1::ResumeHandle, 1 | 2)
        );
        if !allowed || validate_ref_id(&reference.ref_id).is_err() {
            return Err(BootstrapError(
                "committed Resume reader refuses unrelated or secret object",
            ));
        }
        let AuthorityObjectCommitmentV1::CanonicalSha256 { digest_hex } = &reference.commitment
        else {
            return Err(BootstrapError(
                "committed Resume reader cannot verify HMAC objects",
            ));
        };
        let index = self
            .state
            .object_index
            .get(&reference.ref_id)
            .ok_or(BootstrapError(
                "committed Resume canonical object has no index",
            ))?;
        if index.schema_version != 1
            || index.ref_id != reference.ref_id
            || index.object_kind != reference.object_kind
            || index.object_schema_version != reference.schema_version
            || index.storage_state != AuthorityObjectStorageStateV1::Present
            || index.byte_length > 32 * 1024 * 1024
        {
            return Err(BootstrapError(
                "committed Resume canonical object index disagrees",
            ));
        }
        let expected_length = index.byte_length;
        let version = format!("v{}", reference.schema_version);
        let name = format!("{}.obj", reference.ref_id);
        let parents = [
            AUTHORITY_DIRECTORY,
            "objects",
            kind_slug(reference.object_kind),
            version.as_str(),
        ];
        let retained_bytes = self
            .files
            .values()
            .try_fold(self.root_bytes.len() as u64, |n, b| {
                n.checked_add(b.len() as u64)
            })
            .ok_or(BootstrapError(
                "committed Resume recovery byte bound overflow",
            ))?;
        let already_retained = self.files.keys().any(|p| {
            p.iter().map(String::as_str).eq(parents
                .iter()
                .copied()
                .chain(std::iter::once(name.as_str())))
        });
        if retained_bytes
            .checked_add(if already_retained { 0 } else { expected_length })
            .is_none_or(|n| n > 32 * 1024 * 1024)
        {
            return Err(BootstrapError(
                "committed Resume recovery exceeds its byte bound",
            ));
        }
        let bytes = self
            .directories
            .with_directory(self.root.directory(), &parents, false, |directory, _| {
                direct_host_inbox_read_bounded_file(directory, &name)
            })?
            .flatten()
            .ok_or(BootstrapError(
                "committed Resume canonical object is absent",
            ))?;
        if bytes.len() as u64 != expected_length
            || super::object_persistence::canonical_schema_version(&bytes)?
                != reference.schema_version
            || canonical_digest(reference.object_kind, &bytes)? != *digest_hex
        {
            return Err(BootstrapError(
                "committed Resume canonical object bytes disagree",
            ));
        }
        self.files.insert(
            parents
                .iter()
                .map(|s| (*s).to_owned())
                .chain(std::iter::once(name))
                .collect(),
            bytes.clone(),
        );
        self.verify()?;
        Ok(bytes)
    }

    fn verify_lock(&self) -> Result<(), BootstrapError> {
        revalidate_world_work_receipt_root_lock_scope(
            self.root,
            &self.authority_entry,
            &self.authority,
            &self.lock_entry,
            &self.lock,
            &self.root_lock_entry,
        )
    }

    pub(in crate::execution::agent_runtime::host_session_authority) fn verify(
        &self,
    ) -> Result<(), BootstrapError> {
        self.verify_lock()?;
        self.authority
            .revalidate_entry(&self.root_entry)
            .map_err(|_| BootstrapError("committed Resume named root changed"))?;
        if direct_host_inbox_read_bounded_file(&self.authority, ROOT_FILE)?.as_deref()
            != Some(self.root_bytes.as_slice())
        {
            return Err(BootstrapError(
                "committed Resume retained root bytes changed",
            ));
        }
        self.directories.revalidate(self.root.directory())?;
        // Reopen through retained descriptors; no keys or unrelated object
        // collections are enumerated or read during this exact verification.
        for (path, expected) in &self.files {
            let (name, parents) = path
                .split_last()
                .ok_or(BootstrapError("empty committed Resume object path"))?;
            let mut directory = self
                .root
                .directory()
                .open_directory(&parents[0])
                .map_err(|_| BootstrapError("reopen committed Resume authority"))?;
            for component in &parents[1..] {
                directory = directory
                    .open_directory(component)
                    .map_err(|_| BootstrapError("reopen committed Resume object directory"))?;
            }
            if direct_host_inbox_read_bounded_file(&directory, name)?.as_ref() != Some(expected) {
                return Err(BootstrapError("retained committed Resume object changed"));
            }
        }
        self.verify_lock()
    }
}

/// One physical lock covers both the minimal committed shortcut and the strict
/// active branch. A response-loss retry therefore cannot race payload release
/// between an unlocked early probe and the original semantic preflight.
#[cfg(target_os = "linux")]
pub(in crate::execution::agent_runtime::host_session_authority::store) fn with_direct_host_resume_recovery_opened<
    T,
>(
    root: &TrustedAuthorityRoot,
    committed: impl FnOnce(&mut DirectHostResumeRecoveryReadV1<'_>) -> Result<Option<T>, BootstrapError>,
    active: impl FnOnce(&mut DirectHostInboxTransactionV1<'_, '_>) -> Result<T, BootstrapError>,
) -> Result<T, BootstrapError> {
    let mut recovery = DirectHostResumeRecoveryReadV1::open(root)?;
    let result = committed(&mut recovery);
    recovery.verify()?;
    match result? {
        Some(value) => Ok(value),
        None => {
            let result = with_direct_host_inbox_scope(root, false, Some(&recovery), active);
            recovery.verify_lock()?;
            result
        }
    }
}

// This capability is intentionally independent of the strict generic snapshot.
// It adds contextual custody only for the current Host run verifier, while the
// existing activated read-only inbox scope retains layout/exact-history checks.
#[cfg(target_os = "linux")]
pub(in crate::execution::agent_runtime::host_session_authority) struct CurrentHostRunReadViewV1<
    'root,
    'scope,
> {
    decoded_root: &'scope VersionedStateRoot,
    custody: &'scope CurrentHostRunReadCustodyV1<'root>,
}

#[cfg(target_os = "linux")]
struct CurrentHostRunReadCustodyV1<'root> {
    supplied_root: &'root TrustedAuthorityRoot,
    bootstrap_metadata: TrustedEntryMetadataV1,
    bootstrap_manifest: Vec<DirectoryEntry>,
    authority: ReadOnlyDirectoryGuardV1,
    root_file: ReadOnlyFileGuardV1,
    typed_object_guards: RefCell<Vec<ReadOnlyTypedObjectGuardV1>>,
}

#[cfg(target_os = "linux")]
impl<'root> CurrentHostRunReadCustodyV1<'root> {
    fn capture(
        root: &'root TrustedAuthorityRoot,
    ) -> Result<Self, ReadOnlyAuthoritySnapshotErrorV1> {
        root.revalidate()
            .map_err(|_| ReadOnlyAuthoritySnapshotErrorV1::AuthorityRootReplaced)?;
        let bootstrap_metadata = root
            .directory()
            .metadata()
            .map_err(|_| ReadOnlyAuthoritySnapshotErrorV1::AuthorityRootReplaced)?;
        let bootstrap_manifest = root
            .directory()
            .entries()
            .map_err(|_| ReadOnlyAuthoritySnapshotErrorV1::AuthorityRootReplaced)?;
        let authority = capture_typed_object_directory(
            root.directory(),
            &bootstrap_manifest,
            AUTHORITY_DIRECTORY,
            "authority-v1",
        )?;
        let entry = authority
            .manifest
            .iter()
            .find(|entry| entry.name == ROOT_FILE)
            .ok_or_else(|| typed_object_read_error("current Host run entry root is absent"))?;
        let root_file = ReadOnlyFileGuardV1::capture(&authority.directory, entry, "authority-v1")?;
        Ok(Self {
            supplied_root: root,
            bootstrap_metadata,
            bootstrap_manifest,
            authority,
            root_file,
            typed_object_guards: RefCell::new(Vec::new()),
        })
    }

    fn verify(&self) -> Result<(), ReadOnlyAuthoritySnapshotErrorV1> {
        self.supplied_root
            .revalidate()
            .map_err(|_| ReadOnlyAuthoritySnapshotErrorV1::AuthorityRootReplaced)?;
        if self
            .supplied_root
            .directory()
            .metadata()
            .map_err(|_| ReadOnlyAuthoritySnapshotErrorV1::AuthorityRootReplaced)?
            != self.bootstrap_metadata
            || self
                .supplied_root
                .directory()
                .entries()
                .map_err(|_| ReadOnlyAuthoritySnapshotErrorV1::AuthorityRootReplaced)?
                != self.bootstrap_manifest
        {
            return Err(ReadOnlyAuthoritySnapshotErrorV1::AuthorityRootReplaced);
        }
        self.authority.verify(self.supplied_root.directory())?;
        self.root_file.verify(&self.authority.directory)?;
        for guard in self.typed_object_guards.borrow().iter() {
            guard.verify(&self.authority.directory)?;
        }
        self.root_file.verify(&self.authority.directory)?;
        self.authority.verify(self.supplied_root.directory())?;
        self.supplied_root
            .revalidate()
            .map_err(|_| ReadOnlyAuthoritySnapshotErrorV1::AuthorityRootReplaced)
    }
}

#[cfg(target_os = "linux")]
impl CurrentHostRunReadViewV1<'_, '_> {
    pub(in crate::execution::agent_runtime::host_session_authority) fn captured_root(
        &self,
    ) -> &VersionedStateRoot {
        self.decoded_root
    }

    pub(in crate::execution::agent_runtime::host_session_authority) fn read_typed_object(
        &self,
        expected_authority_store_id: &str,
        expected_root_revision: u64,
        reference: &AuthorityObjectRefV1,
        context: Option<&ObjectVerificationContextV1>,
    ) -> Result<Vec<u8>, ReadOnlyAuthoritySnapshotErrorV1> {
        if self.decoded_root.authority_store_id() != expected_authority_store_id {
            return Err(typed_object_read_error(
                "frozen HSA authority store mismatch",
            ));
        }
        if self.decoded_root.root_revision() != expected_root_revision {
            return Err(typed_object_read_error("frozen HSA root revision mismatch"));
        }
        match self.decoded_root {
            VersionedStateRoot::V2(root) => self.capture_typed_object(root, reference, context),
            VersionedStateRoot::V3(root) => self.capture_typed_object(root, reference, context),
            VersionedStateRoot::V1(_) => Err(typed_object_read_error(
                "current Host run requires V2 or V3",
            )),
        }
    }
    /// Internal physical capture commitment; never returned to a consumer.
    pub(in crate::execution::agent_runtime::host_session_authority) fn current_cut_identity_v1(
        &self,
    ) -> String {
        use sha2::{Digest, Sha256};
        let mut hash = Sha256::new();
        hash.update(format!(
            "{:?}{:?}{:?}",
            self.custody.bootstrap_metadata,
            self.custody.authority.metadata,
            self.custody.root_file.metadata
        ));
        for guard in self.custody.typed_object_guards.borrow().iter() {
            for file in [&guard.object, &guard.key].into_iter().flatten() {
                hash.update(format!("{:?}{:?}", file.entry, file.metadata));
            }
            for directory in [&guard.objects, &guard.kind, &guard.version, &guard.keys]
                .into_iter()
                .flatten()
            {
                hash.update(format!("{:?}{:?}", directory.metadata, directory.manifest));
            }
        }
        format!("{:x}", hash.finalize())
    }

    fn capture_typed_object<R: object_persistence::ObjectVerificationRootV1>(
        &self,
        root: &R,
        reference: &AuthorityObjectRefV1,
        context: Option<&ObjectVerificationContextV1>,
    ) -> Result<Vec<u8>, ReadOnlyAuthoritySnapshotErrorV1> {
        // Register before any fallible traversal or authentication. Partial
        // file/key captures remain visible to finalization on every refusal.
        let mut guards = self.custody.typed_object_guards.borrow_mut();
        guards.push(ReadOnlyTypedObjectGuardV1::default());
        let guard = guards
            .last_mut()
            .ok_or_else(|| typed_object_read_error("retain current Host run capture guard"))?;
        (|| {
            validate_ref_id(&reference.ref_id)
                .map_err(|_| typed_object_read_error("typed object ref ID is invalid"))?;
            let index = root
                .object_index()
                .get(&reference.ref_id)
                .filter(|index| {
                    index.schema_version == 1
                        && index.ref_id == reference.ref_id
                        && index.object_kind == reference.object_kind
                        && index.object_schema_version == reference.schema_version
                        && index.storage_state == AuthorityObjectStorageStateV1::Present
                })
                .ok_or_else(|| {
                    typed_object_read_error("typed object has no exact present index entry")
                })?;
            guard.objects = Some(capture_typed_object_directory(
                &self.custody.authority.directory,
                &self.custody.authority.manifest,
                "objects",
                "authority-v1/objects",
            )?);
            let objects = guard
                .objects
                .as_ref()
                .ok_or_else(|| typed_object_read_error("retain captured objects directory"))?;
            guard.kind = Some(capture_typed_object_directory(
                &objects.directory,
                &objects.manifest,
                kind_slug(reference.object_kind),
                "authority-v1/objects/kind",
            )?);
            let kind = guard
                .kind
                .as_ref()
                .ok_or_else(|| typed_object_read_error("retain captured object kind directory"))?;
            guard.version = Some(capture_typed_object_directory(
                &kind.directory,
                &kind.manifest,
                &format!("v{}", reference.schema_version),
                "authority-v1/objects/kind/version",
            )?);
            let version = guard.version.as_ref().ok_or_else(|| {
                typed_object_read_error("retain captured object version directory")
            })?;
            let object_name = format!("{}.obj", reference.ref_id);
            let entry = version
                .manifest
                .iter()
                .find(|entry| entry.name == object_name)
                .ok_or_else(|| typed_object_read_error("typed object file is absent"))?;
            guard.object = Some(ReadOnlyFileGuardV1::capture(
                &version.directory,
                entry,
                "authority-v1/objects/kind/version",
            )?);
            let object = guard
                .object
                .as_ref()
                .ok_or_else(|| typed_object_read_error("retain captured object file"))?;
            if object.bytes.len() as u64 != index.byte_length {
                return Err(typed_object_read_error("typed object byte length mismatch"));
            }
            if matches!(
                reference.commitment,
                AuthorityObjectCommitmentV1::CanonicalSha256 { .. }
            ) {
                let schema_version = object_persistence::canonical_schema_version(&object.bytes)
                    .map_err(|error| typed_object_read_error(error.0))?;
                if schema_version != reference.schema_version {
                    return Err(typed_object_read_error(
                        "canonical object schema version mismatch",
                    ));
                }
            }
            if let AuthorityObjectCommitmentV1::StoreHmacSha256 { key_id, .. } =
                &reference.commitment
            {
                let record = root
                    .commitment_key_registry()
                    .get(key_id)
                    .filter(|record| record.state != AuthorityStoreCommitmentKeyStateV1::Retired)
                    .ok_or_else(|| {
                        typed_object_read_error("sensitive object key is unavailable")
                    })?;
                guard.keys = Some(capture_typed_object_directory(
                    &self.custody.authority.directory,
                    &self.custody.authority.manifest,
                    "keys",
                    "authority-v1/keys",
                )?);
                let keys = guard
                    .keys
                    .as_ref()
                    .ok_or_else(|| typed_object_read_error("retain captured keys directory"))?;
                let key_name = format!("{}.key", record.key_id);
                let entry = keys
                    .manifest
                    .iter()
                    .find(|entry| entry.name == key_name)
                    .ok_or_else(|| {
                        typed_object_read_error("sensitive object key file is absent")
                    })?;
                guard.key = Some(ReadOnlyFileGuardV1::capture(
                    &keys.directory,
                    entry,
                    "authority-v1/keys",
                )?);
                let key = guard
                    .key
                    .as_ref()
                    .ok_or_else(|| typed_object_read_error("retain captured key file"))?;
                let envelope = AuthorityStoreCommitmentKeyFileV1::decode(&key.bytes)
                    .map_err(|_| typed_object_read_error("decode sensitive object key"))?;
                if envelope.authority_store_id != root.authority_store_id()
                    || envelope.authority_store_id != record.authority_store_id
                    || envelope.key_id != *key_id
                    || envelope.key_id != record.key_id
                    || envelope.created_at != record.created_at
                {
                    return Err(typed_object_read_error(
                        "sensitive object key identity mismatch",
                    ));
                }
            }
            #[cfg(test)]
            run_current_host_run_read_hook(CurrentHostRunReadPhaseV1::BeforeObjectAuthentication);
            object_persistence::verify_object_bytes_with_key_loader(
                root,
                reference,
                &object.bytes,
                context,
                true,
                |_| {
                    guard
                        .key
                        .as_ref()
                        .map(|key| key.bytes.clone())
                        .ok_or(BootstrapError("captured sensitive object key is absent"))
                },
            )
            .map_err(|error| typed_object_read_error(error.0))?;
            Ok(object.bytes.clone())
        })()
    }
}

/// Root bytes are captured before entering the existing scope, then physically
/// revalidated under its lock and matched to the exact original retained root.
/// The view exposes neither that scope's mutation methods nor a key exporter.
#[cfg(target_os = "linux")]
pub(in crate::execution::agent_runtime::host_session_authority) fn with_existing_current_host_run_snapshot_opened<
    T,
>(
    opened: &TrustedAuthorityRoot,
    operation: impl FnOnce(
        &CurrentHostRunReadViewV1<'_, '_>,
    ) -> Result<T, ReadOnlyAuthoritySnapshotErrorV1>,
) -> Result<T, BootstrapError> {
    let custody = CurrentHostRunReadCustodyV1::capture(opened)
        .map_err(|_| BootstrapError("capture current Host run entry custody"))?;
    #[cfg(test)]
    run_current_host_run_read_hook(CurrentHostRunReadPhaseV1::BeforeScope);
    with_existing_direct_host_inbox_snapshot_opened(opened, |transaction| {
        // Keep setup, authentication and semantic refusal as a value. No early
        // return may bypass custody verification or the outer scope finalizer.
        let semantic = (|| {
            custody
                .verify()
                .map_err(|_| BootstrapError("current Host run entry custody changed"))?;
            let captured = VersionedStateRoot::decode(&custody.root_file.bytes)
                .map_err(|_| BootstrapError("decode captured current Host run entry root"))?;
            if transaction.locked_root.as_ref() != Some(&captured) {
                return Err(BootstrapError(
                    "current Host run captured entry differs from scope root",
                ));
            }
            if !matches!(
                captured,
                VersionedStateRoot::V2(_) | VersionedStateRoot::V3(_)
            ) {
                return Err(BootstrapError("current Host run requires V2 or V3"));
            }
            let view = CurrentHostRunReadViewV1 {
                decoded_root: &captured,
                custody: &custody,
            };
            operation(&view).map_err(|_| BootstrapError("current Host run contextual read refused"))
        })();
        #[cfg(test)]
        run_current_host_run_read_hook(CurrentHostRunReadPhaseV1::BeforeCustodyFinalize);
        let integrity = custody.verify();
        #[cfg(test)]
        run_current_host_run_read_hook(CurrentHostRunReadPhaseV1::AfterCustodyFinalize);
        // The unchanged outer inbox scope still runs transaction.verify() on
        // this callback's Ok OR Err, with its lock and our captures retained.
        match integrity {
            Ok(()) => semantic,
            Err(_) => Err(BootstrapError(
                "current Host run custody finalization failed",
            )),
        }
    })
}

#[cfg(all(test, target_os = "linux"))]
#[derive(Clone, Copy)]
pub(in crate::execution::agent_runtime::host_session_authority) enum CurrentHostRunReadPhaseV1 {
    BeforeScope = 0,
    BeforeCustodyFinalize = 1,
    AfterCustodyFinalize = 2,
    BeforeObjectAuthentication = 3,
}

#[cfg(all(test, target_os = "linux"))]
type CurrentHostRunReadHooksV1 = RefCell<[Option<Box<dyn FnOnce()>>; 4]>;

#[cfg(all(test, target_os = "linux"))]
std::thread_local! {
    static CURRENT_HOST_RUN_READ_HOOKS: CurrentHostRunReadHooksV1 =
        RefCell::new([None, None, None, None]);
}

#[cfg(all(test, target_os = "linux"))]
pub(in crate::execution::agent_runtime::host_session_authority) fn set_current_host_run_read_hook(
    phase: CurrentHostRunReadPhaseV1,
    hook: impl FnOnce() + 'static,
) {
    CURRENT_HOST_RUN_READ_HOOKS.with(|slots| {
        assert!(slots.borrow_mut()[phase as usize]
            .replace(Box::new(hook))
            .is_none());
    });
}

#[cfg(all(test, target_os = "linux"))]
fn run_current_host_run_read_hook(phase: CurrentHostRunReadPhaseV1) {
    let hook = CURRENT_HOST_RUN_READ_HOOKS.with(|slots| slots.borrow_mut()[phase as usize].take());
    if let Some(hook) = hook {
        hook();
    }
}

// Same existing Root/read owners as ordinary inspection, with their original
// verifier available at the synchronous accepted-control effect boundary.
#[cfg(target_os = "linux")]
pub(in crate::execution::agent_runtime::host_session_authority) fn with_existing_ordinary_accepted_control_snapshot<
    T,
>(
    transaction: &ReadOnlyVersionedAuthorityTransactionV1<'_>,
    operation: impl FnOnce(
        &AcceptedWorkAuthorityPhysicalSnapshotV1,
        Option<&[u8]>,
        &dyn Fn() -> Result<(), ReadOnlyAuthoritySnapshotErrorV1>,
    ) -> T,
) -> Result<T, ReadOnlyAuthoritySnapshotErrorV1> {
    let e2 = DispatchPolicyCommitmentReadCapabilityV1::from_transaction(transaction)
        .read_existing_snapshot()?;
    let b1 = transaction.read_world_work_receipt_registry_snapshot()?;
    let physical = AcceptedWorkAuthorityPhysicalSnapshotV1 {
        authority_root_identity: transaction.authority_root_identity().clone(),
        authority_store_id: transaction.authority_store_id().to_owned(),
        hsa_state_root_bytes: transaction.root_bytes().to_vec(),
        hsa_root_revision_at_read: transaction.root_revision(),
        e2,
        b1,
    };
    let b1_guard = transaction.b1_guard.borrow();
    let hub = b1_guard
        .as_ref()
        .and_then(|guard| guard.agent_hub.as_ref())
        .ok_or(ReadOnlyAuthoritySnapshotErrorV1::Io {
            operation: "ordinary accepted control B1 source absent",
        })?;
    const NAMESPACE: &str = "run/agent-hub/ordinary-accepted-control";
    if let Some(entry) = hub.manifest.iter().find(|entry| {
        entry
            .name
            .starts_with(WORLD_WORK_EXECUTION_SUPERVISOR_PREFIX)
            && entry.name != WORLD_WORK_EXECUTION_SUPERVISOR_FILE
    }) {
        return Err(ReadOnlyAuthoritySnapshotErrorV1::UnsafeNamespaceEntry {
            namespace: NAMESPACE,
            name: entry.name.clone(),
        });
    }
    let supervisor = hub
        .manifest
        .iter()
        .find(|entry| entry.name == WORLD_WORK_EXECUTION_SUPERVISOR_FILE)
        .map(|entry| ReadOnlyFileGuardV1::capture(&hub.directory, entry, NAMESPACE))
        .transpose()?;
    let verify = || -> Result<(), ReadOnlyAuthoritySnapshotErrorV1> {
        transaction.verify_scope()?;
        if let Some(file) = &supervisor {
            file.verify(&hub.directory)?;
        }
        hub.verify(
            &b1_guard
                .as_ref()
                .ok_or(ReadOnlyAuthoritySnapshotErrorV1::Io {
                    operation: "retain original accepted-control B1 guard",
                })?
                .run
                .directory,
        )?;
        transaction.verify_scope()
    };
    verify()?;
    let result = operation(
        &physical,
        supervisor.as_ref().map(|file| file.bytes.as_slice()),
        &verify,
    );
    verify()?;
    Ok(result)
}
