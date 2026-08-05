use super::super::LegacyStateStoreDirectoryEntryV1;
use super::legacy::ObservedLegacyDirectory;
use super::*;
use std::collections::BTreeMap;
use std::sync::Arc;

const WORLD_WORK_RECEIPT_REGISTRY_FILE: &str = "world-work-receipt-registry-v1.json";
const WORLD_WORK_RECEIPT_REGISTRY_PREFIX: &str = "world-work-receipt-registry-v1";
const WORLD_WORK_RECEIPT_REGISTRY_TEMP_PREFIX: &str = "world-work-receipt-registry-v1--";
const WORLD_WORK_RECEIPT_REGISTRY_TEMP_SUFFIX: &str = ".tmp";
const WORLD_WORK_EXECUTION_SUPERVISOR_FILE: &str = "world-work-execution-supervisor-v1.json";
const WORLD_WORK_EXECUTION_SUPERVISOR_PREFIX: &str = "world-work-execution-supervisor-v1";
const WORLD_WORK_EXECUTION_SUPERVISOR_TEMP_PREFIX: &str = "world-work-execution-supervisor-v1--";
const WORLD_WORK_EXECUTION_SUPERVISOR_TEMP_SUFFIX: &str = ".tmp";

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
    _lock: TrustedOwnedFileLock,
}

impl WorldWorkReceiptRegistryStorageV1 {
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
    pub(crate) fn read_registry(&mut self) -> Result<Option<Vec<u8>>, BootstrapError> {
        self.verify_scope()?;
        validate_world_work_receipt_registry_namespace(&self.agent_hub)?;
        let bytes = read_world_work_receipt_registry(&self.agent_hub)?;
        self.verify_scope()?;
        Ok(bytes)
    }

    pub(crate) fn replace_registry(&mut self, bytes: &[u8]) -> Result<(), BootstrapError> {
        use rand::RngCore as _;

        self.verify_scope()?;
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
        temp.write_all(bytes)
            .map_err(|_| BootstrapError("write B1 receipt-registry temp"))?;
        temp.sync()
            .map_err(|_| BootstrapError("sync B1 receipt-registry temp"))?;
        self.verify_scope()?;
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

impl WorldWorkExecutionSupervisorTransactionV1<'_> {
    pub(crate) fn read_supervisor(&mut self) -> Result<Option<Vec<u8>>, BootstrapError> {
        self.verify_scope()?;
        validate_world_work_execution_supervisor_namespace(&self.agent_hub)?;
        let bytes = read_world_work_execution_supervisor(&self.agent_hub)?;
        self.verify_scope()?;
        Ok(bytes)
    }

    pub(crate) fn replace_supervisor(&mut self, bytes: &[u8]) -> Result<(), BootstrapError> {
        use rand::RngCore as _;

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
        self.verify_scope()?;
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
