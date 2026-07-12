use super::legacy::ObservedLegacyDirectory;
use super::*;
use std::collections::BTreeMap;

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
        operation: impl FnOnce(&TrustedDirectory) -> Result<T, BootstrapError>,
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
        operation: impl FnOnce(&TrustedDirectory) -> Result<T, BootstrapError>,
    ) -> Result<Option<T>, BootstrapError> {
        let Some((component, remaining)) = components.split_first() else {
            return operation(parent).map(Some);
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
    pub(super) root: StateRootV1,
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
}

impl LegacyStateStoreTransactionV1 {
    #[cfg(test)]
    pub(crate) fn create_classified_run_directory_test(&mut self) -> Result<(), BootstrapError> {
        self.retained_directories
            .with_directory(self.root.directory(), &["run"], true, |_| Ok(()))?
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
        let prefix = match collection {
            LegacyStateStoreCollectionV1::Sessions => ["run", "agent-hub", "sessions"],
            LegacyStateStoreCollectionV1::Participants => ["run", "agent-hub", "participants"],
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
            |parent| operation(parent, target),
        )
    }
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

fn with_opened_semantic_preflight<T>(
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
    with_semantic_preflight(
        path,
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
    with_existing_semantic_preflight(path, |transaction| {
        validate_positive_expectations(expected)?;
        validate_proposed_root(&transaction.root, expected, proposed)?;
        if transaction.root == *proposed {
            validate_exact_retry_expectation(expected, proposed)?;
            transaction.reconcile()?;
            transaction
                .layout
                .validate_root_candidate(proposed)
                .map_err(|_| BootstrapError("validate exact committed authority root"))?;
            return Ok(TransactionCommitOutcomeV1::JoinedExact(proposed.clone()));
        }
        if transaction.root.root_revision != expected.root_revision {
            return Err(BootstrapError("stale authority root revision"));
        }
        validate_current_authority_expectation(&transaction.root, expected)?;
        validate_authority_changes(&transaction.root, proposed, expected)?;
        transaction.reconcile()?;
        transaction
            .layout
            .validate_root_candidate(proposed)
            .map_err(|_| BootstrapError("validate proposed authority root"))?;
        transaction
            .legacy
            .revalidate(transaction.layout.bootstrap)
            .map_err(|_| BootstrapError("revalidate legacy state before root CAS"))?;
        transaction
            .layout
            .validate_root_candidate(proposed)
            .map_err(|_| BootstrapError("revalidate proposed authority root"))?;
        publish_replacement_root(
            transaction.layout,
            &transaction.legacy,
            proposed,
            nonce_bytes,
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
    let authorities = proposed
        .session_namespace_map
        .iter()
        .filter_map(|(session_id, record)| match record {
            SessionNamespaceRecordV1::Authority(authority) => Some((session_id, authority)),
            SessionNamespaceRecordV1::StartReservation(_)
            | SessionNamespaceRecordV1::StartTombstone(_) => None,
        })
        .collect::<Vec<_>>();
    match expected.authority.as_ref() {
        None if authorities.is_empty() => Ok(()),
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
