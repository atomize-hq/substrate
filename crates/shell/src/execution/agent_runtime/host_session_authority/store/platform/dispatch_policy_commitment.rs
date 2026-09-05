use super::*;
#[cfg(target_os = "linux")]
use crate::execution::agent_runtime::dispatch_policy_commitment::ReadOnlyAuthoritySnapshotErrorV1;
#[cfg(target_os = "linux")]
use std::collections::BTreeMap;

#[cfg(target_os = "linux")]
use super::transaction::{
    ReadOnlyDirectoryGuardV1, ReadOnlyE2NamespaceGuardV1, ReadOnlyFileGuardV1,
    ReadOnlyVersionedAuthorityTransactionV1,
};

const DIRECTORY: &str = "dispatch-policy-commitment-v1";
const REGISTRY_FILE: &str = "registry-v1.json";
const KEYS_DIRECTORY: &str = "keys";
const TEMP_DIRECTORY: &str = "tmp";
#[cfg(target_os = "linux")]
const E2_RM_NAMESPACE: &str = "authority-v1/dispatch-policy-commitment-v1";
#[cfg(target_os = "linux")]
const E2_RM_KEYS_NAMESPACE: &str = "authority-v1/dispatch-policy-commitment-v1/keys";
#[cfg(target_os = "linux")]
const E2_RM_TEMP_NAMESPACE: &str = "authority-v1/dispatch-policy-commitment-v1/tmp";

#[cfg(target_os = "linux")]
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum DispatchPolicyCommitmentPhysicalReadV1 {
    Absent,
    Present(DispatchPolicyCommitmentPhysicalSnapshotV1),
}

#[cfg(target_os = "linux")]
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct DispatchPolicyCommitmentPhysicalSnapshotV1 {
    registry_bytes: Vec<u8>,
    key_files: BTreeMap<String, Vec<u8>>,
}

#[cfg(target_os = "linux")]
impl DispatchPolicyCommitmentPhysicalSnapshotV1 {
    pub(crate) fn registry_bytes(&self) -> &[u8] {
        &self.registry_bytes
    }

    pub(crate) fn key_files(&self) -> &BTreeMap<String, Vec<u8>> {
        &self.key_files
    }
}

#[cfg(target_os = "linux")]
pub(super) struct DispatchPolicyCommitmentReadCapabilityV1<'scope, 'root> {
    transaction: &'scope ReadOnlyVersionedAuthorityTransactionV1<'root>,
}

#[cfg(target_os = "linux")]
impl<'scope, 'root> DispatchPolicyCommitmentReadCapabilityV1<'scope, 'root> {
    pub(super) fn from_transaction(
        transaction: &'scope ReadOnlyVersionedAuthorityTransactionV1<'root>,
    ) -> Self {
        Self { transaction }
    }

    pub(crate) fn read_existing_snapshot(
        &self,
    ) -> Result<DispatchPolicyCommitmentPhysicalReadV1, ReadOnlyAuthoritySnapshotErrorV1> {
        let Some(root_entry) = self
            .transaction
            .authority_manifest()
            .iter()
            .find(|entry| entry.name == DIRECTORY)
        else {
            return Ok(DispatchPolicyCommitmentPhysicalReadV1::Absent);
        };
        let root = ReadOnlyDirectoryGuardV1::capture(
            self.transaction.authority_directory(),
            root_entry,
            E2_RM_NAMESPACE,
        )?;
        let mut guard = ReadOnlyE2NamespaceGuardV1 {
            root,
            keys: None,
            temporary: None,
            registry: None,
            key_files: Vec::new(),
            key_temporary_files: Vec::new(),
            registry_temporary_files: Vec::new(),
        };
        let result = (|| {
            for entry in &guard.root.manifest {
                if !matches!(
                    (entry.name.as_str(), entry.kind),
                    (KEYS_DIRECTORY | TEMP_DIRECTORY, EntryKind::Directory)
                        | (REGISTRY_FILE, EntryKind::RegularFile)
                ) {
                    return Err(ReadOnlyAuthoritySnapshotErrorV1::UnsafeNamespaceEntry {
                        namespace: E2_RM_NAMESPACE,
                        name: entry.name.clone(),
                    });
                }
            }
            let registry_entry = required_entry(
                &guard.root.manifest,
                REGISTRY_FILE,
                EntryKind::RegularFile,
                E2_RM_NAMESPACE,
            )?;
            let keys_entry = required_entry(
                &guard.root.manifest,
                KEYS_DIRECTORY,
                EntryKind::Directory,
                E2_RM_NAMESPACE,
            )?;
            let temporary_entry = required_entry(
                &guard.root.manifest,
                TEMP_DIRECTORY,
                EntryKind::Directory,
                E2_RM_NAMESPACE,
            )?;
            guard.registry = Some(ReadOnlyFileGuardV1::capture(
                &guard.root.directory,
                registry_entry,
                E2_RM_NAMESPACE,
            )?);
            guard.keys = Some(ReadOnlyDirectoryGuardV1::capture(
                &guard.root.directory,
                keys_entry,
                E2_RM_KEYS_NAMESPACE,
            )?);
            guard.temporary = Some(ReadOnlyDirectoryGuardV1::capture(
                &guard.root.directory,
                temporary_entry,
                E2_RM_TEMP_NAMESPACE,
            )?);

            let keys = guard
                .keys
                .as_ref()
                .ok_or(ReadOnlyAuthoritySnapshotErrorV1::Io {
                    operation: "retain E2-RM keys guard",
                })?;
            let mut key_files = BTreeMap::new();
            let mut failure = None;
            for entry in &keys.manifest {
                if entry.kind == EntryKind::RegularFile && key_temp_name(&entry.name) {
                    match ReadOnlyFileGuardV1::capture(&keys.directory, entry, E2_RM_KEYS_NAMESPACE)
                    {
                        Ok(temporary_key) => {
                            failure.get_or_insert(
                                ReadOnlyAuthoritySnapshotErrorV1::UnsafeTemporaryMaterial {
                                    namespace: E2_RM_KEYS_NAMESPACE,
                                    name: temporary_key.entry.name.clone(),
                                },
                            );
                            guard.key_temporary_files.push(temporary_key);
                        }
                        Err(error) => {
                            failure.get_or_insert(error);
                        }
                    }
                    continue;
                }
                if entry.kind != EntryKind::RegularFile || !key_file_name(&entry.name) {
                    failure.get_or_insert(ReadOnlyAuthoritySnapshotErrorV1::UnsafeNamespaceEntry {
                        namespace: E2_RM_KEYS_NAMESPACE,
                        name: entry.name.clone(),
                    });
                    continue;
                }
                match ReadOnlyFileGuardV1::capture(&keys.directory, entry, E2_RM_KEYS_NAMESPACE) {
                    Ok(key) => {
                        key_files.insert(entry.name.clone(), key.bytes.clone());
                        guard.key_files.push(key);
                    }
                    Err(error) => {
                        failure.get_or_insert(error);
                    }
                }
            }

            let temporary =
                guard
                    .temporary
                    .as_ref()
                    .ok_or(ReadOnlyAuthoritySnapshotErrorV1::Io {
                        operation: "retain E2-RM temporary guard",
                    })?;
            for entry in &temporary.manifest {
                if entry.kind != EntryKind::RegularFile || !registry_temp_name(&entry.name) {
                    failure.get_or_insert(
                        ReadOnlyAuthoritySnapshotErrorV1::UnsafeTemporaryMaterial {
                            namespace: E2_RM_TEMP_NAMESPACE,
                            name: entry.name.clone(),
                        },
                    );
                    continue;
                }
                match ReadOnlyFileGuardV1::capture(
                    &temporary.directory,
                    entry,
                    E2_RM_TEMP_NAMESPACE,
                ) {
                    Ok(temporary_registry) => {
                        failure.get_or_insert(
                            ReadOnlyAuthoritySnapshotErrorV1::UnsafeTemporaryMaterial {
                                namespace: E2_RM_TEMP_NAMESPACE,
                                name: temporary_registry.entry.name.clone(),
                            },
                        );
                        guard.registry_temporary_files.push(temporary_registry);
                    }
                    Err(error) => {
                        failure.get_or_insert(error);
                    }
                }
            }
            if let Some(error) = failure {
                return Err(error);
            }

            Ok(DispatchPolicyCommitmentPhysicalReadV1::Present(
                DispatchPolicyCommitmentPhysicalSnapshotV1 {
                    registry_bytes: guard
                        .registry
                        .as_ref()
                        .ok_or(ReadOnlyAuthoritySnapshotErrorV1::Io {
                            operation: "retain E2-RM registry guard",
                        })?
                        .bytes
                        .clone(),
                    key_files,
                },
            ))
        })();
        self.transaction.retain_e2_guard(guard)?;
        result
    }
}

#[cfg(target_os = "linux")]
fn required_entry<'a>(
    manifest: &'a [DirectoryEntry],
    name: &'static str,
    kind: EntryKind,
    namespace: &'static str,
) -> Result<&'a DirectoryEntry, ReadOnlyAuthoritySnapshotErrorV1> {
    let entry = manifest.iter().find(|entry| entry.name == name).ok_or(
        ReadOnlyAuthoritySnapshotErrorV1::PartialNamespace {
            namespace,
            component: name,
        },
    )?;
    if entry.kind != kind {
        return Err(ReadOnlyAuthoritySnapshotErrorV1::UnsafeNamespaceEntry {
            namespace,
            name: entry.name.clone(),
        });
    }
    Ok(entry)
}

pub(crate) struct DispatchPolicyCommitmentStorageV1 {
    root: TrustedAuthorityRoot,
    authority_store_id: String,
}

pub(crate) struct DispatchPolicyCommitmentStorageTransactionV1 {
    authority: TrustedDirectory,
    directory: TrustedDirectory,
    keys: TrustedDirectory,
    tmp: TrustedDirectory,
}

impl DispatchPolicyCommitmentStorageV1 {
    pub(crate) fn authority_store_id(&self) -> &str {
        &self.authority_store_id
    }

    pub(crate) fn transaction<T>(
        &self,
        operation: impl FnOnce(
            &mut DispatchPolicyCommitmentStorageTransactionV1,
        ) -> Result<T, BootstrapError>,
    ) -> Result<T, BootstrapError> {
        with_opened_existing_versioned_semantic_preflight(&self.root, |transaction| {
            transaction.reconcile()?;
            if transaction.root.authority_store_id() != self.authority_store_id {
                return Err(BootstrapError(
                    "dispatch policy commitment capability authority store changed",
                ));
            }
            let mut storage = open_transaction(transaction.layout)?;
            operation(&mut storage)
        })
    }
}

impl DispatchPolicyCommitmentStorageTransactionV1 {
    pub(crate) fn read_registry(&self) -> Result<Option<Vec<u8>>, BootstrapError> {
        match self
            .directory
            .entry_kind(REGISTRY_FILE)
            .map_err(|_| BootstrapError("inspect dispatch policy commitment registry"))?
        {
            None => Ok(None),
            Some(EntryKind::RegularFile) => self
                .directory
                .open_file(REGISTRY_FILE)
                .and_then(|file| file.read_all())
                .map(Some)
                .map_err(|_| BootstrapError("read dispatch policy commitment registry")),
            Some(_) => Err(BootstrapError(
                "dispatch policy commitment registry is unsafe",
            )),
        }
    }

    pub(crate) fn read_keys(&self) -> Result<Vec<(String, Vec<u8>)>, BootstrapError> {
        let mut result = Vec::new();
        for entry in self
            .keys
            .entries()
            .map_err(|_| BootstrapError("enumerate dispatch policy commitment keys"))?
        {
            if entry.kind != EntryKind::RegularFile || !key_file_name(&entry.name) {
                return Err(BootstrapError(
                    "dispatch policy commitment key layout is invalid",
                ));
            }
            self.keys
                .revalidate_entry(&entry)
                .map_err(|_| BootstrapError("dispatch policy commitment key changed"))?;
            result.push((
                entry.name.clone(),
                self.keys
                    .open_file_entry(&entry)
                    .and_then(|file| file.read_all())
                    .map_err(|_| BootstrapError("read dispatch policy commitment key"))?,
            ));
        }
        Ok(result)
    }

    pub(crate) fn stage_key_temp(&self, name: &str, bytes: &[u8]) -> Result<(), BootstrapError> {
        if !key_temp_name(name) {
            return Err(BootstrapError(
                "dispatch policy commitment key temp name is invalid",
            ));
        }
        let mut file = self
            .keys
            .create_file(name)
            .map_err(|_| BootstrapError("create dispatch policy commitment key temp"))?;
        file.write_all(bytes)
            .map_err(|_| BootstrapError("write dispatch policy commitment key temp"))?;
        file.sync()
            .map_err(|_| BootstrapError("sync dispatch policy commitment key temp"))?;
        self.keys
            .sync()
            .map_err(|_| BootstrapError("sync dispatch policy commitment key directory"))
    }

    pub(crate) fn publish_staged_key_no_replace(
        &self,
        temp_name: &str,
        key_name: &str,
    ) -> Result<(), BootstrapError> {
        if !key_temp_name(temp_name) || !key_file_name(key_name) {
            return Err(BootstrapError(
                "dispatch policy commitment key publication name is invalid",
            ));
        }
        let file = self
            .keys
            .open_file(temp_name)
            .map_err(|_| BootstrapError("open dispatch policy commitment key temp"))?;
        self.keys
            .rename_no_replace(temp_name, file, &self.keys, key_name)
            .map_err(|_| BootstrapError("publish dispatch policy commitment key"))?;
        sync_publication(&self.keys, &self.directory, &self.authority)
    }

    pub(crate) fn remove_key(&self, name: &str) -> Result<(), BootstrapError> {
        if !key_file_name(name) {
            return Err(BootstrapError(
                "dispatch policy commitment orphan key name is invalid",
            ));
        }
        self.keys
            .unlink_file(name)
            .map_err(|_| BootstrapError("remove dispatch policy commitment orphan key"))?;
        self.keys
            .sync()
            .map_err(|_| BootstrapError("sync dispatch policy commitment key directory"))
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

    #[cfg(test)]
    pub(crate) fn stage_registry_replacement_for_test(
        &self,
        temp_name: &str,
        bytes: &[u8],
    ) -> Result<(), BootstrapError> {
        self.stage_registry_temp(temp_name, bytes)
    }

    fn publish_registry(
        &self,
        temp_name: &str,
        bytes: &[u8],
        replace: bool,
    ) -> Result<(), BootstrapError> {
        self.stage_registry_temp(temp_name, bytes)?;
        let file = self
            .tmp
            .open_file(temp_name)
            .map_err(|_| BootstrapError("open dispatch policy commitment registry temp"))?;
        if replace {
            self.tmp
                .rename_replace(temp_name, file, &self.directory, REGISTRY_FILE)
                .map_err(|_| BootstrapError("replace dispatch policy commitment registry"))?;
        } else {
            self.tmp
                .rename_no_replace(temp_name, file, &self.directory, REGISTRY_FILE)
                .map_err(|_| BootstrapError("publish dispatch policy commitment registry"))?;
        }
        sync_publication(&self.tmp, &self.directory, &self.authority)
    }

    fn stage_registry_temp(&self, name: &str, bytes: &[u8]) -> Result<(), BootstrapError> {
        if !registry_temp_name(name) {
            return Err(BootstrapError(
                "dispatch policy commitment registry temp name is invalid",
            ));
        }
        let mut file = self
            .tmp
            .create_file(name)
            .map_err(|_| BootstrapError("create dispatch policy commitment registry temp"))?;
        file.write_all(bytes)
            .map_err(|_| BootstrapError("write dispatch policy commitment registry temp"))?;
        file.sync()
            .map_err(|_| BootstrapError("sync dispatch policy commitment registry temp"))?;
        self.tmp
            .sync()
            .map_err(|_| BootstrapError("sync dispatch policy commitment temp directory"))
    }
}

pub(crate) fn dispatch_policy_commitment_storage_opened(
    opened: &TrustedAuthorityRoot,
) -> Result<DispatchPolicyCommitmentStorageV1, BootstrapError> {
    opened
        .revalidate()
        .map_err(|_| BootstrapError("revalidate dispatch policy commitment authority root"))?;
    let rebound =
        TrustedAuthorityRoot::open(std::path::Path::new(&opened.identity().physical_path))
            .map_err(|_| BootstrapError("open dispatch policy commitment authority root"))?;
    if rebound.identity() != opened.identity() {
        return Err(BootstrapError(
            "dispatch policy commitment authority root identity mismatch",
        ));
    }
    let authority_store_id =
        with_opened_existing_versioned_semantic_preflight(&rebound, |transaction| {
            transaction.reconcile()?;
            Ok(transaction.root.authority_store_id().to_owned())
        })?;
    Ok(DispatchPolicyCommitmentStorageV1 {
        root: rebound,
        authority_store_id,
    })
}

pub(crate) fn dispatch_policy_commitment_registry_exists_opened(
    opened: &TrustedAuthorityRoot,
) -> Result<bool, BootstrapError> {
    opened
        .revalidate()
        .map_err(|_| BootstrapError("revalidate dispatch policy commitment authority root"))?;
    let authority = match opened
        .directory()
        .entry_kind(AUTHORITY_DIRECTORY)
        .map_err(|_| BootstrapError("inspect dispatch policy authority directory"))?
    {
        None => return Ok(false),
        Some(EntryKind::Directory) => opened
            .directory()
            .open_directory(AUTHORITY_DIRECTORY)
            .map_err(|_| BootstrapError("open dispatch policy authority directory"))?,
        Some(_) => {
            return Err(BootstrapError(
                "dispatch policy authority directory is unsafe",
            ))
        }
    };
    let directory = match authority
        .entry_kind(DIRECTORY)
        .map_err(|_| BootstrapError("inspect dispatch policy commitment directory"))?
    {
        None => return Ok(false),
        Some(EntryKind::Directory) => authority
            .open_directory(DIRECTORY)
            .map_err(|_| BootstrapError("open dispatch policy commitment directory"))?,
        Some(_) => {
            return Err(BootstrapError(
                "dispatch policy commitment directory is unsafe",
            ))
        }
    };
    let exists = match directory
        .entry_kind(REGISTRY_FILE)
        .map_err(|_| BootstrapError("inspect dispatch policy commitment registry"))?
    {
        None => false,
        Some(EntryKind::RegularFile) => {
            directory
                .open_file(REGISTRY_FILE)
                .map_err(|_| BootstrapError("open dispatch policy commitment registry"))?;
            true
        }
        Some(_) => {
            return Err(BootstrapError(
                "dispatch policy commitment registry is unsafe",
            ))
        }
    };
    opened
        .revalidate()
        .map_err(|_| BootstrapError("revalidate dispatch policy commitment authority root"))?;
    Ok(exists)
}

fn open_transaction(
    layout: &StoreLayout<'_>,
) -> Result<DispatchPolicyCommitmentStorageTransactionV1, BootstrapError> {
    let directory = match layout
        .authority
        .entry_kind(DIRECTORY)
        .map_err(|_| BootstrapError("inspect dispatch policy commitment directory"))?
    {
        None => layout
            .authority
            .create_directory(DIRECTORY)
            .map_err(|_| BootstrapError("create dispatch policy commitment directory"))?,
        Some(EntryKind::Directory) => layout
            .authority
            .open_directory(DIRECTORY)
            .map_err(|_| BootstrapError("open dispatch policy commitment directory"))?,
        Some(_) => {
            return Err(BootstrapError(
                "dispatch policy commitment directory is unsafe",
            ))
        }
    };
    let entries = directory
        .entries()
        .map_err(|_| BootstrapError("enumerate dispatch policy commitment directory"))?;
    let registry_exists = entries
        .iter()
        .any(|entry| entry.name == REGISTRY_FILE && entry.kind == EntryKind::RegularFile);
    for entry in &entries {
        if !matches!(
            (entry.name.as_str(), entry.kind),
            (KEYS_DIRECTORY | TEMP_DIRECTORY, EntryKind::Directory)
                | (REGISTRY_FILE, EntryKind::RegularFile)
        ) {
            return Err(BootstrapError(
                "dispatch policy commitment directory layout is invalid",
            ));
        }
        directory
            .revalidate_entry(entry)
            .map_err(|_| BootstrapError("dispatch policy commitment entry changed"))?;
    }
    let open_component = |name: &str| match directory
        .entry_kind(name)
        .map_err(|_| BootstrapError("inspect dispatch policy commitment component"))?
    {
        Some(EntryKind::Directory) => directory
            .open_directory(name)
            .map_err(|_| BootstrapError("open dispatch policy commitment component")),
        None if !registry_exists => directory
            .create_directory(name)
            .map_err(|_| BootstrapError("create dispatch policy commitment component")),
        None | Some(_) => Err(BootstrapError(
            "dispatch policy commitment component is invalid",
        )),
    };
    let keys = open_component(KEYS_DIRECTORY)?;
    let tmp = open_component(TEMP_DIRECTORY)?;
    reconcile_keys(&keys)?;
    reconcile_temps(&tmp)?;
    directory
        .sync()
        .map_err(|_| BootstrapError("sync dispatch policy commitment layout"))?;
    let authority = layout
        .bootstrap
        .open_directory(AUTHORITY_DIRECTORY)
        .map_err(|_| BootstrapError("reopen dispatch policy authority directory"))?;
    authority
        .sync()
        .map_err(|_| BootstrapError("sync dispatch policy authority layout"))?;
    Ok(DispatchPolicyCommitmentStorageTransactionV1 {
        authority,
        directory,
        keys,
        tmp,
    })
}

fn reconcile_keys(keys: &TrustedDirectory) -> Result<(), BootstrapError> {
    let mut removed = false;
    for entry in keys
        .entries()
        .map_err(|_| BootstrapError("enumerate dispatch policy commitment keys"))?
    {
        if entry.kind != EntryKind::RegularFile {
            return Err(BootstrapError(
                "dispatch policy commitment key layout is invalid",
            ));
        }
        keys.revalidate_entry(&entry)
            .map_err(|_| BootstrapError("dispatch policy commitment key changed"))?;
        if key_file_name(&entry.name) {
            continue;
        }
        if !key_temp_name(&entry.name) {
            return Err(BootstrapError(
                "dispatch policy commitment key layout is invalid",
            ));
        }
        keys.unlink_file(&entry.name)
            .map_err(|_| BootstrapError("remove dispatch policy commitment key temp"))?;
        removed = true;
    }
    if removed {
        keys.sync()
            .map_err(|_| BootstrapError("sync dispatch policy commitment key directory"))?;
    }
    Ok(())
}

fn reconcile_temps(tmp: &TrustedDirectory) -> Result<(), BootstrapError> {
    let mut removed = false;
    for entry in tmp
        .entries()
        .map_err(|_| BootstrapError("enumerate dispatch policy commitment temps"))?
    {
        if entry.kind != EntryKind::RegularFile || !registry_temp_name(&entry.name) {
            return Err(BootstrapError(
                "dispatch policy commitment temp layout is invalid",
            ));
        }
        tmp.revalidate_entry(&entry)
            .map_err(|_| BootstrapError("dispatch policy commitment temp changed"))?;
        tmp.unlink_file(&entry.name)
            .map_err(|_| BootstrapError("remove dispatch policy commitment temp"))?;
        removed = true;
    }
    if removed {
        tmp.sync()
            .map_err(|_| BootstrapError("sync dispatch policy commitment temp directory"))?;
    }
    Ok(())
}

fn sync_publication(
    run_directory: &TrustedDirectory,
    registry_directory: &TrustedDirectory,
    authority: &TrustedDirectory,
) -> Result<(), BootstrapError> {
    run_directory
        .sync()
        .map_err(|_| BootstrapError("sync dispatch policy run directory"))?;
    registry_directory
        .sync()
        .map_err(|_| BootstrapError("sync dispatch policy registry directory"))?;
    authority
        .sync()
        .map_err(|_| BootstrapError("sync dispatch policy authority root"))
}

fn key_file_name(name: &str) -> bool {
    name.strip_suffix(".key")
        .and_then(|value| value.strip_prefix("dpk_"))
        .is_some_and(lower_hex_nonce)
}

fn key_temp_name(name: &str) -> bool {
    name.strip_prefix("dispatch-policy-key--")
        .and_then(|value| value.strip_suffix(".tmp"))
        .is_some_and(lower_hex_nonce)
}

fn registry_temp_name(name: &str) -> bool {
    name.strip_prefix("dispatch-policy-registry--")
        .and_then(|value| value.strip_suffix(".tmp"))
        .is_some_and(lower_hex_nonce)
}

fn lower_hex_nonce(value: &str) -> bool {
    value.len() == 32
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}
