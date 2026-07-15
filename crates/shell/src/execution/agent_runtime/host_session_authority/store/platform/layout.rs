use super::*;

pub(super) struct StoreLayout<'a> {
    pub(super) bootstrap: &'a TrustedDirectory,
    pub(super) authority: TrustedDirectory,
    pub(super) lock: TrustedDirectory,
    pub(super) tmp: TrustedDirectory,
    pub(super) objects: TrustedDirectory,
    pub(super) keys: TrustedDirectory,
    pub(super) root_lock: TrustedFile,
}

fn reconcile_temp_directory(tmp: &TrustedDirectory) -> Result<(), StoreError> {
    validate_temp_directory(tmp)?;
    for entry in tmp
        .entries()
        .map_err(|_| StoreError("enumerate authority temps"))?
    {
        tmp.unlink_file(&entry.name)
            .map_err(|_| StoreError("remove authority temp"))?;
    }
    Ok(())
}

fn validate_temp_directory(tmp: &TrustedDirectory) -> Result<(), StoreError> {
    for entry in tmp
        .entries()
        .map_err(|_| StoreError("enumerate authority temps"))?
    {
        if entry.kind != EntryKind::RegularFile
            || TempNameV1::parse(&entry.name).is_err()
            || tmp.revalidate_entry(&entry).is_err()
        {
            return Err(StoreError("authority temp state is invalid"));
        }
    }
    Ok(())
}

pub(super) struct StoreLayoutLockScope<'a> {
    pub(super) bootstrap: &'a TrustedDirectory,
    pub(super) authority: TrustedDirectory,
    pub(super) lock: TrustedDirectory,
    pub(super) tmp: TrustedDirectory,
    pub(super) root_lock: TrustedFile,
    strict: bool,
}

impl<'a> StoreLayoutLockScope<'a> {
    pub(super) fn open(root: &'a TrustedDirectory) -> Result<Self, StoreError> {
        let authority = match root
            .entry_kind(AUTHORITY_DIRECTORY)
            .map_err(|_| StoreError("inspect authority layout"))?
        {
            None => root
                .create_directory(AUTHORITY_DIRECTORY)
                .map_err(|_| StoreError("create authority layout"))?,
            Some(EntryKind::Directory) => root
                .open_directory(AUTHORITY_DIRECTORY)
                .map_err(|_| StoreError("open authority layout"))?,
            Some(_) => return Err(StoreError("authority layout is unsafe")),
        };
        let strict = authority
            .entry_kind(ROOT_FILE)
            .map_err(|_| StoreError("inspect authority root presence"))?
            .is_some()
            || authority
                .entry_kind(INIT_FILE)
                .map_err(|_| StoreError("inspect initialization marker presence"))?
                .is_some();
        let open_component = |name: &str, error| {
            if strict {
                authority
                    .open_directory(name)
                    .map_err(|_| StoreError(error))
            } else {
                authority
                    .create_directory(name)
                    .map_err(|_| StoreError(error))
            }
        };
        let lock = open_component("lock", "open authority lock directory")?;
        let tmp = open_component("tmp", "open authority temp directory")?;
        let root_lock = create_or_open_lock(&lock, strict)?;
        Ok(Self {
            bootstrap: root,
            authority,
            lock,
            tmp,
            root_lock,
            strict,
        })
    }

    pub(super) fn open_existing_activated(root: &'a TrustedDirectory) -> Result<Self, StoreError> {
        let authority = match root
            .entry_kind(AUTHORITY_DIRECTORY)
            .map_err(|_| StoreError("inspect existing authority layout"))?
        {
            Some(EntryKind::Directory) => root
                .open_directory(AUTHORITY_DIRECTORY)
                .map_err(|_| StoreError("open existing authority layout"))?,
            None | Some(_) => return Err(StoreError("activated authority layout is absent")),
        };
        if authority
            .entry_kind(ROOT_FILE)
            .map_err(|_| StoreError("inspect activated authority root"))?
            != Some(EntryKind::RegularFile)
        {
            return Err(StoreError("activated authority root is absent or unsafe"));
        }
        let lock = authority
            .open_directory("lock")
            .map_err(|_| StoreError("open existing authority lock directory"))?;
        let tmp = authority
            .open_directory("tmp")
            .map_err(|_| StoreError("open existing authority temp directory"))?;
        let root_lock = create_or_open_lock(&lock, true)?;
        Ok(Self {
            bootstrap: root,
            authority,
            lock,
            tmp,
            root_lock,
            strict: true,
        })
    }

    pub(super) fn authority_activated(&self) -> Result<bool, StoreError> {
        Ok(self
            .authority
            .entry_kind(ROOT_FILE)
            .map_err(|_| StoreError("inspect authority root activation"))?
            .is_some()
            || self
                .authority
                .entry_kind(INIT_FILE)
                .map_err(|_| StoreError("inspect authority marker activation"))?
                .is_some())
    }

    pub(super) fn validate_temps(&self) -> Result<(), StoreError> {
        validate_temp_directory(&self.tmp)
    }

    pub(super) fn reconcile_temps(&self) -> Result<(), StoreError> {
        reconcile_temp_directory(&self.tmp)
    }

    pub(super) fn finish(self) -> Result<StoreLayout<'a>, StoreError> {
        let open_component = |name: &str, error| {
            if self.strict {
                self.authority
                    .open_directory(name)
                    .map_err(|_| StoreError(error))
            } else {
                self.authority
                    .create_directory(name)
                    .map_err(|_| StoreError(error))
            }
        };
        let objects = open_component("objects", "open authority objects directory")?;
        let keys = open_component("keys", "open authority keys directory")?;
        Ok(StoreLayout {
            bootstrap: self.bootstrap,
            authority: self.authority,
            lock: self.lock,
            tmp: self.tmp,
            objects,
            keys,
            root_lock: self.root_lock,
        })
    }
}

impl<'a> StoreLayout<'a> {
    pub(super) fn open(root: &'a TrustedDirectory) -> Result<Self, StoreError> {
        StoreLayoutLockScope::open(root)?.finish()
    }

    pub(super) fn semantic_preflight(
        &self,
        bootstrap_home: &crate::execution::agent_runtime::host_session_authority::schema::CanonicalDirectoryV1,
    ) -> Result<LockedClassification, StoreError> {
        self.validate_closed_layout()?;
        self.validate_temps()?;
        let legacy = LegacyObservation::capture(self.bootstrap)?;
        let root_kind = self
            .authority
            .entry_kind(ROOT_FILE)
            .map_err(|_| StoreError("inspect authority root"))?;
        let init_kind = self
            .authority
            .entry_kind(INIT_FILE)
            .map_err(|_| StoreError("inspect initialization marker"))?;

        let (authority_classification, root) = match (root_kind, init_kind) {
            (None, None) if self.keys_empty()? && self.objects_empty()? => {
                (BootstrapClassificationV1::FreshAbsent, None)
            }
            (None, Some(EntryKind::RegularFile)) if self.objects_empty()? => {
                self.validate_pending(bootstrap_home)?;
                (BootstrapClassificationV1::InitializationPending, None)
            }
            (Some(EntryKind::RegularFile), None | Some(EntryKind::RegularFile)) => {
                let root = self.read_existing_without_reconciliation(bootstrap_home)?;
                self.validate_matching_marker_if_present(&root)?;
                (BootstrapClassificationV1::ValidExisting, Some(root))
            }
            _ => return Err(StoreError("authority root and marker state is invalid")),
        };
        let classification = if legacy.has_artifact {
            BootstrapClassificationV1::UnsupportedLegacyState
        } else {
            authority_classification
        };
        Ok(LockedClassification {
            classification,
            authority_classification,
            legacy,
            root,
        })
    }

    pub(super) fn reconcile_after_preflight(&self, root: &StateRootV1) -> Result<(), StoreError> {
        self.reconcile_temps()?;
        self.remove_matching_marker(root)?;
        self.reconcile_key_files(root)?;
        self.reconcile_released_objects(root)?;
        self.validate_existing_objects(root, false)
    }

    pub(super) fn reconcile_after_preflight_v2(
        &self,
        root: &StateRootV2,
    ) -> Result<(), StoreError> {
        self.reconcile_temps()?;
        self.remove_matching_marker_v2(root)?;
        self.reconcile_key_files_v2(root)?;
        self.reconcile_released_objects_v2(root)?;
        self.validate_existing_objects_v2(root, false)
    }

    pub(super) fn validate_closed_layout(&self) -> Result<(), StoreError> {
        for entry in self
            .authority
            .entries()
            .map_err(|_| StoreError("enumerate authority layout"))?
        {
            let valid = matches!(
                (entry.name.as_str(), entry.kind),
                ("lock" | "tmp" | "objects" | "keys", EntryKind::Directory)
                    | (ROOT_FILE | INIT_FILE, EntryKind::RegularFile)
            );
            if !valid {
                return Err(StoreError("authority layout contains an unknown entry"));
            }
            self.authority
                .revalidate_entry(&entry)
                .map_err(|_| StoreError("authority layout entry changed"))?;
        }
        let lock_entries = self
            .lock
            .entries()
            .map_err(|_| StoreError("enumerate authority lock directory"))?;
        if lock_entries.len() != 1
            || lock_entries[0].name != ROOT_LOCK_FILE
            || lock_entries[0].kind != EntryKind::RegularFile
        {
            return Err(StoreError("authority lock directory is invalid"));
        }
        self.lock
            .revalidate_entry(&lock_entries[0])
            .map_err(|_| StoreError("authority root lock changed"))
    }

    pub(super) fn reconcile_temps(&self) -> Result<(), StoreError> {
        reconcile_temp_directory(&self.tmp)
    }

    pub(super) fn validate_temps(&self) -> Result<(), StoreError> {
        validate_temp_directory(&self.tmp)
    }

    pub(super) fn validate_pending(
        &self,
        bootstrap_home: &crate::execution::agent_runtime::host_session_authority::schema::CanonicalDirectoryV1,
    ) -> Result<(), StoreError> {
        let marker = self
            .authority
            .open_file(INIT_FILE)
            .map_err(|_| StoreError("open initialization marker"))?;
        let marker: AuthorityStoreInitializationV1 = canonical_json::from_slice(
            &marker
                .read_all()
                .map_err(|_| StoreError("read initialization marker"))?,
        )
        .map_err(|_| StoreError("decode initialization marker"))?;
        marker
            .validate()
            .map_err(|_| StoreError("validate initialization marker"))?;
        if &marker.bootstrap_home != bootstrap_home {
            return Err(StoreError("initialization marker home mismatch"));
        }
        let keys = self
            .keys
            .entries()
            .map_err(|_| StoreError("enumerate initialization keys"))?;
        if keys.len() > 1
            || keys
                .first()
                .is_some_and(|entry| entry.name != format!("{}.key", marker.initial_key_id))
        {
            return Err(StoreError("pending initialization key state is invalid"));
        }
        if let Some(entry) = keys.first() {
            if entry.kind != EntryKind::RegularFile {
                return Err(StoreError("pending initialization key is unsafe"));
            }
            self.keys
                .revalidate_entry(entry)
                .map_err(|_| StoreError("pending initialization key changed"))?;
            let envelope = AuthorityStoreCommitmentKeyFileV1::decode(
                &self
                    .keys
                    .open_file(&entry.name)
                    .map_err(|_| StoreError("open pending initialization key"))?
                    .read_all()
                    .map_err(|_| StoreError("read pending initialization key"))?,
            )
            .map_err(|_| StoreError("decode pending initialization key"))?;
            if envelope.authority_store_id != marker.authority_store_id
                || envelope.key_id != marker.initial_key_id
                || envelope.created_at != marker.created_at
            {
                return Err(StoreError("pending initialization key identity mismatch"));
            }
        }
        Ok(())
    }

    pub(super) fn read_marker(&self) -> Result<AuthorityStoreInitializationV1, StoreError> {
        let marker = self
            .authority
            .open_file(INIT_FILE)
            .map_err(|_| StoreError("open initialization marker"))?;
        let marker: AuthorityStoreInitializationV1 = canonical_json::from_slice(
            &marker
                .read_all()
                .map_err(|_| StoreError("read initialization marker"))?,
        )
        .map_err(|_| StoreError("decode initialization marker"))?;
        marker
            .validate()
            .map_err(|_| StoreError("validate initialization marker"))?;
        Ok(marker)
    }

    pub(super) fn ensure_initial_key(
        &self,
        marker: &AuthorityStoreInitializationV1,
        nonce_bytes: [u8; 16],
        secret_key: [u8; 32],
    ) -> Result<(), BootstrapError> {
        let final_name = format!("{}.key", marker.initial_key_id);
        match self
            .keys
            .entry_kind(&final_name)
            .map_err(|_| BootstrapError("inspect initial commitment key"))?
        {
            None => {
                let envelope = AuthorityStoreCommitmentKeyFileV1 {
                    authority_store_id: marker.authority_store_id.clone(),
                    key_id: marker.initial_key_id.clone(),
                    created_at: marker.created_at.clone(),
                    secret_key,
                };
                let temp_name = TempNameV1::Key {
                    key_id: marker.initial_key_id.clone(),
                    nonce: nonce(nonce_bytes),
                }
                .file_name();
                let mut temp = self
                    .tmp
                    .create_file(&temp_name)
                    .map_err(|_| BootstrapError("create commitment key temp"))?;
                temp.write_all(
                    &envelope
                        .encode()
                        .map_err(|_| BootstrapError("encode commitment key"))?,
                )
                .map_err(|_| BootstrapError("write commitment key temp"))?;
                temp.sync()
                    .map_err(|_| BootstrapError("sync commitment key temp"))?;
                self.tmp
                    .rename_no_replace(&temp_name, temp, &self.keys, &final_name)
                    .map_err(|_| BootstrapError("publish commitment key"))?;
            }
            Some(EntryKind::RegularFile) => {}
            Some(_) => return Err(BootstrapError("initial commitment key is unsafe")),
        }
        let key = self
            .keys
            .open_file(&final_name)
            .map_err(|_| BootstrapError("open initial commitment key"))?;
        let envelope = AuthorityStoreCommitmentKeyFileV1::decode(
            &key.read_all()
                .map_err(|_| BootstrapError("read initial commitment key"))?,
        )
        .map_err(|_| BootstrapError("decode initial commitment key"))?;
        if envelope.authority_store_id != marker.authority_store_id
            || envelope.key_id != marker.initial_key_id
            || envelope.created_at != marker.created_at
        {
            return Err(BootstrapError("initial commitment key identity mismatch"));
        }
        key.sync()
            .map_err(|_| BootstrapError("sync initial commitment key"))?;
        self.keys
            .sync()
            .map_err(|_| BootstrapError("sync commitment key directory"))
    }

    pub(super) fn publish_key_envelope(
        &self,
        envelope: &AuthorityStoreCommitmentKeyFileV1,
        nonce_bytes: [u8; 16],
    ) -> Result<(), BootstrapError> {
        let final_name = format!("{}.key", envelope.key_id);
        if self
            .keys
            .entry_kind(&final_name)
            .map_err(|_| BootstrapError("inspect commitment key target"))?
            .is_some()
        {
            return Err(BootstrapError("commitment key target already exists"));
        }
        let temp_name = TempNameV1::Key {
            key_id: envelope.key_id.clone(),
            nonce: nonce(nonce_bytes),
        }
        .file_name();
        let mut temp = self
            .tmp
            .create_file(&temp_name)
            .map_err(|_| BootstrapError("create commitment key temp"))?;
        temp.write_all(
            &envelope
                .encode()
                .map_err(|_| BootstrapError("encode commitment key"))?,
        )
        .map_err(|_| BootstrapError("write commitment key temp"))?;
        temp.sync()
            .map_err(|_| BootstrapError("sync commitment key temp"))?;
        self.tmp
            .rename_no_replace(&temp_name, temp, &self.keys, &final_name)
            .map_err(|_| BootstrapError("publish commitment key"))?;
        let published = self
            .keys
            .open_file(&final_name)
            .map_err(|_| BootstrapError("open published commitment key"))?;
        let decoded = AuthorityStoreCommitmentKeyFileV1::decode(
            &published
                .read_all()
                .map_err(|_| BootstrapError("read published commitment key"))?,
        )
        .map_err(|_| BootstrapError("decode published commitment key"))?;
        if decoded != *envelope {
            return Err(BootstrapError("published commitment key changed"));
        }
        published
            .sync()
            .map_err(|_| BootstrapError("sync published commitment key"))?;
        self.keys
            .sync()
            .map_err(|_| BootstrapError("sync commitment key directory"))
    }

    pub(super) fn read_existing_without_reconciliation(
        &self,
        bootstrap_home: &crate::execution::agent_runtime::host_session_authority::schema::CanonicalDirectoryV1,
    ) -> Result<StateRootV1, StoreError> {
        let root = self
            .authority
            .open_file(ROOT_FILE)
            .map_err(|_| StoreError("open state root"))?;
        let root: StateRootV1 = canonical_json::from_slice(
            &root.read_all().map_err(|_| StoreError("read state root"))?,
        )
        .map_err(|_| StoreError("decode state root"))?;
        root.validate()
            .map_err(|_| StoreError("validate state root"))?;
        if &root.bootstrap_home != bootstrap_home {
            return Err(StoreError("state root home mismatch"));
        }
        self.validate_existing_keys(&root)?;
        self.validate_existing_objects(&root, true)?;
        self.validate_reachable_objects(&root)?;
        Ok(root)
    }

    pub(super) fn read_existing_versioned_without_reconciliation(
        &self,
        bootstrap_home: &crate::execution::agent_runtime::host_session_authority::schema::CanonicalDirectoryV1,
    ) -> Result<VersionedStateRoot, StoreError> {
        let file = self
            .authority
            .open_file(ROOT_FILE)
            .map_err(|_| StoreError("open versioned state root"))?;
        let root = VersionedStateRoot::decode(
            &file
                .read_all()
                .map_err(|_| StoreError("read versioned state root"))?,
        )
        .map_err(|_| StoreError("decode versioned state root"))?;
        root.validate()
            .map_err(|_| StoreError("validate versioned state root"))?;
        if root.bootstrap_home() != bootstrap_home {
            return Err(StoreError("versioned state root home mismatch"));
        }
        match &root {
            VersionedStateRoot::V1(root) => {
                self.validate_existing_keys(root)?;
                self.validate_existing_objects(root, true)?;
                self.validate_reachable_objects(root)?;
            }
            VersionedStateRoot::V2(root) => {
                self.validate_existing_keys_v2(root)?;
                self.validate_existing_objects_v2(root, true)?;
                self.validate_reachable_objects_v2(root)?;
            }
        }
        Ok(root)
    }

    pub(super) fn read_greenfield_upgrade_root(
        &self,
        bootstrap_home: &crate::execution::agent_runtime::host_session_authority::schema::CanonicalDirectoryV1,
    ) -> Result<VersionedStateRoot, StoreError> {
        let file = self
            .authority
            .open_file(ROOT_FILE)
            .map_err(|_| StoreError("open greenfield upgrade root"))?;
        let bytes = file
            .read_all()
            .map_err(|_| StoreError("read greenfield upgrade root"))?;
        let root = VersionedStateRoot::decode(&bytes)
            .map_err(|_| StoreError("decode strict greenfield upgrade root"))?;
        match &root {
            VersionedStateRoot::V1(root) => {
                root.validate()
                    .map_err(|_| StoreError("validate strict greenfield StateRootV1"))?;
                if &root.bootstrap_home != bootstrap_home {
                    return Err(StoreError("greenfield StateRootV1 home mismatch"));
                }
                self.validate_existing_keys(root)?;
                self.validate_existing_objects(root, true)?;
                self.validate_reachable_objects(root)?;
                self.validate_greenfield_upgrade_occupancy(
                    &root.authority_store_id,
                    &root.commitment_key_registry,
                )?;
                StateRootV2::try_from_greenfield_v1(root)
                    .map_err(|_| StoreError("UnsupportedNonGreenfieldRootV1"))?;
                self.validate_matching_marker_if_present(root)?;
            }
            VersionedStateRoot::V2(root) => {
                root.validate_greenfield()
                    .map_err(|_| StoreError("validate strict greenfield StateRootV2"))?;
                if &root.bootstrap_home != bootstrap_home {
                    return Err(StoreError("greenfield StateRootV2 home mismatch"));
                }
                self.validate_existing_keys_v2(root)?;
                self.validate_greenfield_upgrade_occupancy(
                    &root.authority_store_id,
                    &root.commitment_key_registry,
                )?;
                self.validate_matching_marker_if_present_v2(root)?;
            }
        }
        Ok(root)
    }

    fn validate_greenfield_upgrade_occupancy(
        &self,
        authority_store_id: &str,
        registry: &std::collections::BTreeMap<String, AuthorityStoreCommitmentKeyV1>,
    ) -> Result<(), StoreError> {
        let keys = self
            .keys
            .entries()
            .map_err(|_| StoreError("enumerate greenfield upgrade keys"))?;
        for entry in keys {
            let key_id = entry
                .name
                .strip_suffix(".key")
                .ok_or(StoreError("UnsupportedNonGreenfieldRootV1"))?;
            let record = registry
                .get(key_id)
                .ok_or(StoreError("UnsupportedNonGreenfieldRootV1"))?;
            let envelope = AuthorityStoreCommitmentKeyFileV1::decode(
                &self
                    .keys
                    .open_file(&entry.name)
                    .map_err(|_| StoreError("open greenfield upgrade key"))?
                    .read_all()
                    .map_err(|_| StoreError("read greenfield upgrade key"))?,
            )
            .map_err(|_| StoreError("decode greenfield upgrade key"))?;
            if envelope.authority_store_id != authority_store_id
                || envelope.key_id != record.key_id
                || envelope.created_at != record.created_at
            {
                return Err(StoreError("greenfield upgrade key identity mismatch"));
            }
        }
        for kind_entry in self
            .objects
            .entries()
            .map_err(|_| StoreError("enumerate greenfield upgrade object kinds"))?
        {
            if kind_entry.kind != EntryKind::Directory || kind_from_slug(&kind_entry.name).is_none()
            {
                return Err(StoreError("greenfield upgrade object kind is invalid"));
            }
            self.objects
                .revalidate_entry(&kind_entry)
                .map_err(|_| StoreError("greenfield upgrade object kind changed"))?;
            let kind = self
                .objects
                .open_directory(&kind_entry.name)
                .map_err(|_| StoreError("open greenfield upgrade object kind"))?;
            for version_entry in kind
                .entries()
                .map_err(|_| StoreError("enumerate greenfield upgrade object versions"))?
            {
                if version_entry.kind != EntryKind::Directory || version_entry.name != "v1" {
                    return Err(StoreError("greenfield upgrade object version is invalid"));
                }
                kind.revalidate_entry(&version_entry)
                    .map_err(|_| StoreError("greenfield upgrade object version changed"))?;
                let version = kind
                    .open_directory(&version_entry.name)
                    .map_err(|_| StoreError("open greenfield upgrade object version"))?;
                if !version
                    .entries()
                    .map_err(|_| StoreError("enumerate greenfield upgrade objects"))?
                    .is_empty()
                {
                    return Err(StoreError("UnsupportedNonGreenfieldRootV1"));
                }
            }
        }
        Ok(())
    }

    pub(super) fn validate_existing_keys_v2(&self, root: &StateRootV2) -> Result<(), StoreError> {
        let entries = self
            .keys
            .entries()
            .map_err(|_| StoreError("enumerate V2 commitment keys"))?;
        for entry in &entries {
            let key_id = entry
                .name
                .strip_suffix(".key")
                .ok_or(StoreError("commitment key filename is invalid"))?;
            validate_key_id(key_id)
                .map_err(|_| StoreError("commitment key filename is invalid"))?;
            if entry.kind != EntryKind::RegularFile {
                return Err(StoreError("commitment key entry is unsafe"));
            }
            self.keys
                .revalidate_entry(entry)
                .map_err(|_| StoreError("commitment key changed during validation"))?;
            let envelope = AuthorityStoreCommitmentKeyFileV1::decode(
                &self
                    .keys
                    .open_file(&entry.name)
                    .map_err(|_| StoreError("open registered V2 commitment key"))?
                    .read_all()
                    .map_err(|_| StoreError("read registered V2 commitment key"))?,
            )
            .map_err(|_| StoreError("decode registered V2 commitment key"))?;
            if let Some(record) = root.commitment_key_registry.get(key_id) {
                if record.algorithm != AuthorityStoreCommitmentAlgorithmV1::HmacSha256
                    || envelope.authority_store_id != root.authority_store_id
                    || envelope.authority_store_id != record.authority_store_id
                    || envelope.key_id != record.key_id
                    || envelope.created_at != record.created_at
                {
                    return Err(StoreError("registered V2 commitment key identity mismatch"));
                }
            } else if envelope.authority_store_id != root.authority_store_id
                || envelope.key_id != key_id
            {
                return Err(StoreError(
                    "unregistered V2 commitment key identity mismatch",
                ));
            }
        }
        for record in root.commitment_key_registry.values() {
            let present = entries
                .iter()
                .any(|entry| entry.name == format!("{}.key", record.key_id));
            if record.state != AuthorityStoreCommitmentKeyStateV1::Retired && !present {
                return Err(StoreError("required V2 commitment key file is missing"));
            }
        }
        Ok(())
    }

    pub(super) fn validate_root_candidate(&self, root: &StateRootV1) -> Result<(), StoreError> {
        root.validate()
            .map_err(|_| StoreError("validate proposed state root"))?;
        self.validate_existing_keys(root)?;
        self.validate_existing_objects(root, false)?;
        self.validate_reachable_objects(root)
    }

    pub(super) fn validate_root_candidate_v2(&self, root: &StateRootV2) -> Result<(), StoreError> {
        root.validate()
            .map_err(|_| StoreError("validate proposed V2 state root"))?;
        self.validate_existing_keys_v2(root)?;
        self.validate_existing_objects_v2(root, false)?;
        self.validate_reachable_objects_v2(root)
    }

    pub(super) fn validate_existing_keys(&self, root: &StateRootV1) -> Result<(), StoreError> {
        let entries = self
            .keys
            .entries()
            .map_err(|_| StoreError("enumerate commitment keys"))?;
        for entry in &entries {
            let key_id = entry
                .name
                .strip_suffix(".key")
                .ok_or(StoreError("commitment key filename is invalid"))?;
            validate_key_id(key_id)
                .map_err(|_| StoreError("commitment key filename is invalid"))?;
            if entry.kind != EntryKind::RegularFile {
                return Err(StoreError("commitment key entry is unsafe"));
            }
            self.keys
                .revalidate_entry(entry)
                .map_err(|_| StoreError("commitment key changed during validation"))?;
            if let Some(record) = root.commitment_key_registry.get(key_id) {
                self.validate_key_envelope(root, record)?;
            } else {
                let envelope = AuthorityStoreCommitmentKeyFileV1::decode(
                    &self
                        .keys
                        .open_file(&entry.name)
                        .map_err(|_| StoreError("open unregistered commitment key"))?
                        .read_all()
                        .map_err(|_| StoreError("read unregistered commitment key"))?,
                )
                .map_err(|_| StoreError("decode unregistered commitment key"))?;
                if envelope.authority_store_id != root.authority_store_id
                    || envelope.key_id != key_id
                {
                    return Err(StoreError("unregistered commitment key identity mismatch"));
                }
            }
        }
        for record in root.commitment_key_registry.values() {
            let present = entries
                .iter()
                .any(|entry| entry.name == format!("{}.key", record.key_id));
            if record.state != AuthorityStoreCommitmentKeyStateV1::Retired && !present {
                return Err(StoreError("required commitment key file is missing"));
            }
        }
        Ok(())
    }

    pub(super) fn reconcile_key_files(&self, root: &StateRootV1) -> Result<(), StoreError> {
        self.reconcile_key_registry(&root.commitment_key_registry)
    }

    pub(super) fn reconcile_key_files_v2(&self, root: &StateRootV2) -> Result<(), StoreError> {
        self.reconcile_key_registry(&root.commitment_key_registry)
    }

    fn reconcile_key_registry(
        &self,
        registry: &std::collections::BTreeMap<String, AuthorityStoreCommitmentKeyV1>,
    ) -> Result<(), StoreError> {
        for entry in self
            .keys
            .entries()
            .map_err(|_| StoreError("enumerate commitment keys for reconciliation"))?
        {
            let key_id = entry
                .name
                .strip_suffix(".key")
                .ok_or(StoreError("commitment key filename is invalid"))?;
            let remove = match registry.get(key_id) {
                None => true,
                Some(record) => record.state == AuthorityStoreCommitmentKeyStateV1::Retired,
            };
            if remove {
                self.keys
                    .unlink_file(&entry.name)
                    .map_err(|_| StoreError("remove non-authoritative commitment key"))?;
            }
        }
        Ok(())
    }

    pub(super) fn validate_key_envelope(
        &self,
        root: &StateRootV1,
        record: &AuthorityStoreCommitmentKeyV1,
    ) -> Result<(), StoreError> {
        let file = self
            .keys
            .open_file(&format!("{}.key", record.key_id))
            .map_err(|_| StoreError("open registered commitment key"))?;
        let envelope = AuthorityStoreCommitmentKeyFileV1::decode(
            &file
                .read_all()
                .map_err(|_| StoreError("read registered commitment key"))?,
        )
        .map_err(|_| StoreError("decode registered commitment key"))?;
        if record.algorithm != AuthorityStoreCommitmentAlgorithmV1::HmacSha256
            || envelope.authority_store_id != root.authority_store_id
            || envelope.authority_store_id != record.authority_store_id
            || envelope.key_id != record.key_id
            || envelope.created_at != record.created_at
        {
            return Err(StoreError("registered commitment key identity mismatch"));
        }
        Ok(())
    }

    pub(super) fn validate_existing_objects(
        &self,
        root: &StateRootV1,
        allow_released_copy: bool,
    ) -> Result<(), StoreError> {
        self.validate_existing_object_index(&root.object_index, allow_released_copy)
    }

    pub(super) fn validate_existing_objects_v2(
        &self,
        root: &StateRootV2,
        allow_released_copy: bool,
    ) -> Result<(), StoreError> {
        self.validate_existing_object_index(&root.object_index, allow_released_copy)
    }

    fn validate_existing_object_index(
        &self,
        object_index: &std::collections::BTreeMap<String, AuthorityObjectIndexEntryV1>,
        allow_released_copy: bool,
    ) -> Result<(), StoreError> {
        let mut present = std::collections::BTreeSet::new();
        for kind_entry in self
            .objects
            .entries()
            .map_err(|_| StoreError("enumerate object kinds"))?
        {
            if kind_entry.kind != EntryKind::Directory {
                return Err(StoreError("object kind entry is unsafe"));
            }
            let kind = kind_from_slug(&kind_entry.name)
                .ok_or(StoreError("object kind directory is unknown"))?;
            let kind_directory = self
                .objects
                .open_directory(&kind_entry.name)
                .map_err(|_| StoreError("open object kind directory"))?;
            self.objects
                .revalidate_entry(&kind_entry)
                .map_err(|_| StoreError("object kind directory changed"))?;
            for version_entry in kind_directory
                .entries()
                .map_err(|_| StoreError("enumerate object schema versions"))?
            {
                if version_entry.kind != EntryKind::Directory || version_entry.name != "v1" {
                    return Err(StoreError("object schema version directory is invalid"));
                }
                let version_directory = kind_directory
                    .open_directory(&version_entry.name)
                    .map_err(|_| StoreError("open object schema version directory"))?;
                kind_directory
                    .revalidate_entry(&version_entry)
                    .map_err(|_| StoreError("object schema version directory changed"))?;
                for object_entry in version_directory
                    .entries()
                    .map_err(|_| StoreError("enumerate typed objects"))?
                {
                    if object_entry.kind != EntryKind::RegularFile {
                        return Err(StoreError("typed object entry is unsafe"));
                    }
                    let ref_id = object_entry
                        .name
                        .strip_suffix(".obj")
                        .ok_or(StoreError("typed object filename is invalid"))?;
                    validate_ref_id(ref_id)
                        .map_err(|_| StoreError("typed object ref ID is invalid"))?;
                    version_directory
                        .revalidate_entry(&object_entry)
                        .map_err(|_| StoreError("typed object changed during validation"))?;
                    let bytes = version_directory
                        .open_file(&object_entry.name)
                        .map_err(|_| StoreError("open typed object"))?
                        .read_all()
                        .map_err(|_| StoreError("read typed object"))?;
                    if !present.insert(ref_id.to_string()) {
                        return Err(StoreError("typed object ref is duplicated"));
                    }
                    if let Some(index) = object_index.get(ref_id) {
                        let released = matches!(
                            index.storage_state,
                            AuthorityObjectStorageStateV1::Released { .. }
                        );
                        if index.object_kind != kind
                            || index.object_schema_version != 1
                            || (!released && index.byte_length != bytes.len() as u64)
                            || (released && !allow_released_copy)
                        {
                            return Err(StoreError("typed object index does not match file"));
                        }
                    }
                }
            }
        }
        for (ref_id, index) in object_index {
            let must_exist = !matches!(
                index.storage_state,
                AuthorityObjectStorageStateV1::Released { .. }
            );
            if present.contains(ref_id) != must_exist
                && !(allow_released_copy && !must_exist && present.contains(ref_id))
            {
                return Err(StoreError("typed object presence does not match root"));
            }
        }
        Ok(())
    }

    pub(super) fn reconcile_released_objects(&self, root: &StateRootV1) -> Result<(), StoreError> {
        self.reconcile_released_object_index(&root.object_index)
    }

    pub(super) fn reconcile_released_objects_v2(
        &self,
        root: &StateRootV2,
    ) -> Result<(), StoreError> {
        self.reconcile_released_object_index(&root.object_index)
    }

    fn reconcile_released_object_index(
        &self,
        object_index: &std::collections::BTreeMap<String, AuthorityObjectIndexEntryV1>,
    ) -> Result<(), StoreError> {
        for (ref_id, index) in object_index {
            if !matches!(
                index.storage_state,
                AuthorityObjectStorageStateV1::Released { .. }
            ) {
                continue;
            }
            let slug = kind_slug(index.object_kind);
            let Some(EntryKind::Directory) = self
                .objects
                .entry_kind(slug)
                .map_err(|_| StoreError("inspect released object kind"))?
            else {
                continue;
            };
            let kind = self
                .objects
                .open_directory(slug)
                .map_err(|_| StoreError("open released object kind"))?;
            let Some(EntryKind::Directory) = kind
                .entry_kind("v1")
                .map_err(|_| StoreError("inspect released object version"))?
            else {
                continue;
            };
            let version = kind
                .open_directory("v1")
                .map_err(|_| StoreError("open released object version"))?;
            let name = format!("{ref_id}.obj");
            if version
                .entry_kind(&name)
                .map_err(|_| StoreError("inspect released object copy"))?
                .is_some()
            {
                version
                    .unlink_file(&name)
                    .map_err(|_| StoreError("remove released object copy"))?;
            }
        }
        Ok(())
    }

    pub(super) fn validate_reachable_objects(&self, root: &StateRootV1) -> Result<(), StoreError> {
        let mut reachable = collect_reachable_objects(root)?;
        let mut pending = reachable.keys().cloned().collect::<Vec<_>>();
        let mut processed = std::collections::BTreeSet::new();
        let mut attach_contracts = std::collections::BTreeMap::new();
        let mut descriptors = std::collections::BTreeMap::new();
        let mut resume_handles = std::collections::BTreeMap::new();
        let mut retained_workers = std::collections::BTreeMap::new();
        let mut terminal_handoffs = std::collections::BTreeMap::new();
        while let Some(ref_id) = pending.pop() {
            if !processed.insert(ref_id.clone()) {
                continue;
            }
            let object = reachable
                .get(&ref_id)
                .cloned()
                .ok_or(StoreError("reachable object disappeared"))?;
            let index = root
                .object_index
                .get(&ref_id)
                .ok_or(StoreError("parent ref is absent from object index"))?;
            if index.object_kind != object.reference.object_kind
                || index.object_schema_version != object.reference.schema_version
            {
                return Err(StoreError("parent ref and object index disagree"));
            }
            if matches!(
                index.storage_state,
                AuthorityObjectStorageStateV1::Released { .. }
            ) {
                continue;
            }
            let bytes = self.read_object_bytes(&object.reference)?;
            verify_object_bytes(
                self,
                root,
                &object.reference,
                &bytes,
                object.context.as_ref(),
                true,
            )
            .map_err(|_| StoreError("parent-owned object commitment mismatch"))?;
            let before = reachable.len();
            match object.reference.object_kind {
                AuthorityObjectKindV1::HostAttachContract => {
                    let value: HostAttachContractHashInputV1 =
                        canonical_json::from_slice(&bytes)
                            .map_err(|_| StoreError("decode host attach contract graph"))?;
                    add_expected_ref(
                        &mut reachable,
                        &value.contract.descriptor_ref,
                        AuthorityObjectKindV1::AgentDescriptor,
                        None,
                    )?;
                    add_expected_ref(
                        &mut reachable,
                        &value.contract.policy_ref,
                        AuthorityObjectKindV1::Policy,
                        None,
                    )?;
                    if let Some(reference) = &value.contract.continuity_resume_handle_ref {
                        add_expected_ref(
                            &mut reachable,
                            reference,
                            AuthorityObjectKindV1::ResumeHandle,
                            None,
                        )?;
                    }
                    attach_contracts.insert(ref_id.clone(), value);
                }
                AuthorityObjectKindV1::RetainedWorker => {
                    let value: RetainedWorkerObjectHashInputV1 = canonical_json::from_slice(&bytes)
                        .map_err(|_| StoreError("decode retained worker graph"))?;
                    for (reference, kind) in [
                        (
                            &value.descriptor_ref,
                            AuthorityObjectKindV1::AgentDescriptor,
                        ),
                        (
                            &value.resume_handle_ref,
                            AuthorityObjectKindV1::ResumeHandle,
                        ),
                        (&value.policy_ref, AuthorityObjectKindV1::Policy),
                    ] {
                        add_expected_ref(&mut reachable, reference, kind, None)?;
                    }
                    retained_workers.insert(ref_id.clone(), value);
                }
                AuthorityObjectKindV1::AgentDescriptor => {
                    let value: AgentDescriptorHashInputV1 = canonical_json::from_slice(&bytes)
                        .map_err(|_| StoreError("decode agent descriptor graph"))?;
                    descriptors.insert(ref_id.clone(), value);
                }
                AuthorityObjectKindV1::ResumeHandle => {
                    let value: ResumeHandleHashInputV1 = canonical_json::from_slice(&bytes)
                        .map_err(|_| StoreError("decode resume handle graph"))?;
                    resume_handles.insert(ref_id.clone(), value);
                }
                AuthorityObjectKindV1::TerminalHandoff => {
                    let value: TerminalHandoffHashInputV1 = canonical_json::from_slice(&bytes)
                        .map_err(|_| StoreError("decode terminal handoff graph"))?;
                    terminal_handoffs.insert(ref_id.clone(), value);
                }
                _ => {}
            }
            if reachable.len() > before {
                pending.extend(
                    reachable
                        .keys()
                        .filter(|key| !processed.contains(*key))
                        .cloned(),
                );
            }
        }
        if reachable.len() != root.object_index.len() {
            return Err(StoreError("object index and parent reachability differ"));
        }
        self.validate_decoded_object_graphs(
            root,
            &attach_contracts,
            &descriptors,
            &resume_handles,
            &retained_workers,
            &terminal_handoffs,
        )?;
        Ok(())
    }

    pub(super) fn validate_reachable_objects_v2(
        &self,
        root: &StateRootV2,
    ) -> Result<(), StoreError> {
        let mut reachable = collect_reachable_objects_v2(root)?;
        let mut pending = reachable.keys().cloned().collect::<Vec<_>>();
        let mut processed = std::collections::BTreeSet::new();
        let mut attach_contracts = std::collections::BTreeMap::new();
        let mut descriptors = std::collections::BTreeMap::new();
        let mut resume_handles = std::collections::BTreeMap::new();
        let mut retained_workers = std::collections::BTreeMap::new();
        let mut terminal_handoffs = std::collections::BTreeMap::new();
        while let Some(ref_id) = pending.pop() {
            if !processed.insert(ref_id.clone()) {
                continue;
            }
            let object = reachable
                .get(&ref_id)
                .cloned()
                .ok_or(StoreError("V2 reachable object disappeared"))?;
            let index = root
                .object_index
                .get(&ref_id)
                .ok_or(StoreError("V2 parent ref is absent from object index"))?;
            if index.object_kind != object.reference.object_kind
                || index.object_schema_version != object.reference.schema_version
            {
                return Err(StoreError("V2 parent ref and object index disagree"));
            }
            if matches!(
                index.storage_state,
                AuthorityObjectStorageStateV1::Released { .. }
            ) {
                continue;
            }
            let bytes = self.read_object_bytes(&object.reference)?;
            verify_object_bytes(
                self,
                root,
                &object.reference,
                &bytes,
                object.context.as_ref(),
                true,
            )
            .map_err(|_| StoreError("V2 parent-owned object commitment mismatch"))?;
            let before = reachable.len();
            match object.reference.object_kind {
                AuthorityObjectKindV1::HostAttachContract => {
                    let value: HostAttachContractHashInputV1 =
                        canonical_json::from_slice(&bytes)
                            .map_err(|_| StoreError("decode V2 host attach contract graph"))?;
                    add_expected_ref(
                        &mut reachable,
                        &value.contract.descriptor_ref,
                        AuthorityObjectKindV1::AgentDescriptor,
                        None,
                    )?;
                    add_expected_ref(
                        &mut reachable,
                        &value.contract.policy_ref,
                        AuthorityObjectKindV1::Policy,
                        None,
                    )?;
                    if value.contract.continuity_resume_handle_ref.is_some() {
                        return Err(StoreError("A1.2a Start attach contract has a resume ref"));
                    }
                    attach_contracts.insert(ref_id.clone(), value);
                }
                AuthorityObjectKindV1::AgentDescriptor => {
                    let value: AgentDescriptorHashInputV1 = canonical_json::from_slice(&bytes)
                        .map_err(|_| StoreError("decode V2 agent descriptor graph"))?;
                    descriptors.insert(ref_id.clone(), value);
                }
                AuthorityObjectKindV1::RetainedWorker => {
                    let value: RetainedWorkerObjectHashInputV1 = canonical_json::from_slice(&bytes)
                        .map_err(|_| StoreError("decode V2 retained worker graph"))?;
                    for (reference, kind) in [
                        (
                            &value.descriptor_ref,
                            AuthorityObjectKindV1::AgentDescriptor,
                        ),
                        (
                            &value.resume_handle_ref,
                            AuthorityObjectKindV1::ResumeHandle,
                        ),
                        (&value.policy_ref, AuthorityObjectKindV1::Policy),
                    ] {
                        add_expected_ref(&mut reachable, reference, kind, None)?;
                    }
                    retained_workers.insert(ref_id.clone(), value);
                }
                AuthorityObjectKindV1::ResumeHandle => {
                    let value: ResumeHandleHashInputV1 = canonical_json::from_slice(&bytes)
                        .map_err(|_| StoreError("decode V2 resume handle graph"))?;
                    resume_handles.insert(ref_id.clone(), value);
                }
                AuthorityObjectKindV1::TerminalHandoff => {
                    let value: TerminalHandoffHashInputV1 = canonical_json::from_slice(&bytes)
                        .map_err(|_| StoreError("decode V2 terminal handoff graph"))?;
                    terminal_handoffs.insert(ref_id.clone(), value);
                }
                _ => {}
            }
            if reachable.len() > before {
                pending.extend(
                    reachable
                        .keys()
                        .filter(|key| !processed.contains(*key))
                        .cloned(),
                );
            }
        }
        if reachable.len() != root.object_index.len() {
            return Err(StoreError("V2 object index and parent reachability differ"));
        }
        for (ref_id, attach) in &attach_contracts {
            let descriptor = descriptors
                .get(&attach.contract.descriptor_ref.ref_id)
                .ok_or(StoreError("V2 attach descriptor is unreachable"))?;
            if descriptor.descriptor.backend_id != attach.contract.backend_id
                || descriptor.descriptor.protocol != attach.contract.protocol
                || descriptor.descriptor.execution_scope != attach.contract.execution_scope
            {
                return Err(StoreError("V2 attach contract and descriptor disagree"));
            }
            for intent in root
                .transition_intent_map
                .values()
                .filter(|intent| intent.host_attach_contract_ref.ref_id == *ref_id)
            {
                if attach.contract.descriptor_ref != intent.descriptor_ref
                    || attach.contract.continuity_resume_handle_ref.is_some()
                {
                    return Err(StoreError("V2 attach contract and Start intent disagree"));
                }
            }
        }
        for (ref_id, worker) in &retained_workers {
            let descriptor = descriptors
                .get(&worker.descriptor_ref.ref_id)
                .ok_or(StoreError("V2 retained descriptor is unreachable"))?;
            let resume = resume_handles
                .get(&worker.resume_handle_ref.ref_id)
                .ok_or(StoreError("V2 retained resume handle is unreachable"))?;
            if descriptor.descriptor.execution_scope != AgentExecutionScopeV1::World {
                return Err(StoreError("V2 retained descriptor is not world-scoped"));
            }
            validate_resume_identity(
                resume,
                &worker.orchestration_session_id,
                &worker.participant_id,
                &descriptor.descriptor.backend_id,
                &descriptor.descriptor.protocol,
            )?;
            let parents = root
                .session_namespace_map
                .values()
                .filter_map(|record| match record {
                    SessionNamespaceRecordV1::Authority(authority)
                        if authority
                            .retained_worker_refs
                            .iter()
                            .any(|reference| reference.ref_id == *ref_id) =>
                    {
                        Some(authority.as_ref())
                    }
                    _ => None,
                })
                .collect::<Vec<_>>();
            let [authority] = parents.as_slice() else {
                return Err(StoreError(
                    "V2 retained worker has no unique authority parent",
                ));
            };
            let registrations = root
                .retained_worker_registration_journal
                .values()
                .filter(|registration| registration.retained_worker_ref.ref_id == *ref_id)
                .collect::<Vec<_>>();
            let [registration] = registrations.as_slice() else {
                return Err(StoreError(
                    "V2 retained worker has no unique registration parent",
                ));
            };
            if worker.orchestration_session_id != authority.orchestration_session_id
                || worker.orchestration_session_id != registration.orchestration_session_id
                || worker.participant_id != registration.retained_participant_id
                || !authority
                    .authoritative_participant_lineage
                    .contains(&worker.participant_id)
                || authority.world_binding.as_ref() != Some(&worker.world_binding)
                || authority.current_policy_ref.as_ref() != Some(&worker.policy_ref)
                || worker.world_binding != registration.world_binding
                || worker.descriptor_ref != registration.descriptor_ref
                || worker.resume_handle_ref != registration.resume_handle_ref
                || worker.policy_ref != registration.current_policy_ref
            {
                return Err(StoreError(
                    "V2 retained object graph and authority disagree",
                ));
            }
        }
        for intent in root.transition_intent_map.values() {
            let terminal_ref = match &intent.state {
                HostSessionTransitionIntentStateV2::Rejected {
                    terminal_handoff_ref,
                    ..
                }
                | HostSessionTransitionIntentStateV2::Expired {
                    terminal_handoff_ref,
                    ..
                } => Some(terminal_handoff_ref),
                _ => match &intent.transport_payload_state {
                    HostSessionTransitionTransportPayloadStateV1::ReleaseEligible {
                        terminal_handoff_ref,
                    }
                    | HostSessionTransitionTransportPayloadStateV1::Released {
                        terminal_handoff_ref,
                        ..
                    } => Some(terminal_handoff_ref),
                    HostSessionTransitionTransportPayloadStateV1::Retained => None,
                },
            };
            if let Some(reference) = terminal_ref {
                let terminal = terminal_handoffs
                    .get(&reference.ref_id)
                    .ok_or(StoreError("V2 terminal handoff is unreachable"))?;
                validate_terminal_handoff_v2(intent, terminal)?;
            }
        }
        Ok(())
    }

    fn validate_decoded_object_graphs(
        &self,
        root: &StateRootV1,
        attach_contracts: &std::collections::BTreeMap<String, HostAttachContractHashInputV1>,
        descriptors: &std::collections::BTreeMap<String, AgentDescriptorHashInputV1>,
        resume_handles: &std::collections::BTreeMap<String, ResumeHandleHashInputV1>,
        retained_workers: &std::collections::BTreeMap<String, RetainedWorkerObjectHashInputV1>,
        terminal_handoffs: &std::collections::BTreeMap<String, TerminalHandoffHashInputV1>,
    ) -> Result<(), StoreError> {
        for (ref_id, attach) in attach_contracts {
            let descriptor = descriptors
                .get(&attach.contract.descriptor_ref.ref_id)
                .ok_or(StoreError("attach contract descriptor is unreachable"))?;
            if descriptor.descriptor.backend_id != attach.contract.backend_id
                || descriptor.descriptor.protocol != attach.contract.protocol
                || descriptor.descriptor.execution_scope != attach.contract.execution_scope
            {
                return Err(StoreError("attach contract and descriptor disagree"));
            }
            for intent in root
                .transition_intent_map
                .values()
                .filter(|intent| intent.host_attach_contract_ref.ref_id == *ref_id)
            {
                if attach.contract.descriptor_ref != intent.descriptor_ref
                    || attach.contract.continuity_resume_handle_ref != intent.resume_handle_ref
                {
                    return Err(StoreError("attach contract and intent graph disagree"));
                }
                if let Some(reference) = &attach.contract.continuity_resume_handle_ref {
                    let resume = resume_handles
                        .get(&reference.ref_id)
                        .ok_or(StoreError("attach resume handle is unreachable"))?;
                    validate_resume_identity(
                        resume,
                        &intent.orchestration_session_id,
                        &intent.target_authoritative_participant_id,
                        &attach.contract.backend_id,
                        &attach.contract.protocol,
                    )?;
                }
            }
        }
        for (ref_id, worker) in retained_workers {
            let descriptor = descriptors
                .get(&worker.descriptor_ref.ref_id)
                .ok_or(StoreError("retained worker descriptor is unreachable"))?;
            let resume = resume_handles
                .get(&worker.resume_handle_ref.ref_id)
                .ok_or(StoreError("retained worker resume handle is unreachable"))?;
            validate_resume_identity(
                resume,
                &worker.orchestration_session_id,
                &worker.participant_id,
                &descriptor.descriptor.backend_id,
                &descriptor.descriptor.protocol,
            )?;
            let mut parent_count = 0_usize;
            for authority in root
                .session_namespace_map
                .values()
                .filter_map(|record| match record {
                    SessionNamespaceRecordV1::Authority(authority)
                        if authority
                            .retained_worker_refs
                            .iter()
                            .any(|reference| reference.ref_id == *ref_id) =>
                    {
                        Some(authority.as_ref())
                    }
                    _ => None,
                })
            {
                parent_count += 1;
                if worker.orchestration_session_id != authority.orchestration_session_id
                    || !authority
                        .authoritative_participant_lineage
                        .contains(&worker.participant_id)
                    || authority.world_binding.as_ref() != Some(&worker.world_binding)
                {
                    return Err(StoreError("retained worker and authority disagree"));
                }
            }
            if parent_count == 0 {
                return Err(StoreError("retained worker has no authority parent"));
            }
        }
        for authority in root
            .session_namespace_map
            .values()
            .filter_map(|record| match record {
                SessionNamespaceRecordV1::Authority(authority) => Some(authority.as_ref()),
                _ => None,
            })
        {
            for reference in &authority.internal_resume_handle_refs {
                let resume = resume_handles
                    .get(&reference.ref_id)
                    .ok_or(StoreError("authority resume handle is unreachable"))?;
                if resume.orchestration_session_id != authority.orchestration_session_id
                    || !authority
                        .authoritative_participant_lineage
                        .contains(&resume.participant_id)
                {
                    return Err(StoreError("authority resume identity disagrees"));
                }
                let attach = authority
                    .host_attach_contract_ref
                    .as_ref()
                    .and_then(|attach_ref| attach_contracts.get(&attach_ref.ref_id))
                    .filter(|attach| {
                        attach.contract.continuity_resume_handle_ref.as_ref() == Some(reference)
                    });
                let worker = retained_workers
                    .values()
                    .find(|worker| worker.resume_handle_ref == *reference);
                match (attach, worker) {
                    (Some(attach), _) => validate_resume_identity(
                        resume,
                        &authority.orchestration_session_id,
                        &resume.participant_id,
                        &attach.contract.backend_id,
                        &attach.contract.protocol,
                    )?,
                    (None, Some(worker)) => {
                        let descriptor = descriptors
                            .get(&worker.descriptor_ref.ref_id)
                            .ok_or(StoreError("resume owner descriptor is unreachable"))?;
                        validate_resume_identity(
                            resume,
                            &authority.orchestration_session_id,
                            &worker.participant_id,
                            &descriptor.descriptor.backend_id,
                            &descriptor.descriptor.protocol,
                        )?;
                    }
                    (None, None) => {
                        return Err(StoreError("authority resume handle has no semantic owner"))
                    }
                }
            }
        }
        for intent in root.transition_intent_map.values() {
            let terminal_ref = match &intent.state {
                HostSessionTransitionIntentStateV1::Rejected {
                    terminal_handoff_ref,
                    ..
                }
                | HostSessionTransitionIntentStateV1::Expired {
                    terminal_handoff_ref,
                    ..
                } => Some(terminal_handoff_ref),
                HostSessionTransitionIntentStateV1::Issued
                | HostSessionTransitionIntentStateV1::Claimed { .. }
                | HostSessionTransitionIntentStateV1::Applied { .. } => {
                    match &intent.transport_payload_state {
                        HostSessionTransitionTransportPayloadStateV1::ReleaseEligible {
                            terminal_handoff_ref,
                        }
                        | HostSessionTransitionTransportPayloadStateV1::Released {
                            terminal_handoff_ref,
                            ..
                        } => Some(terminal_handoff_ref),
                        HostSessionTransitionTransportPayloadStateV1::Retained => None,
                    }
                }
            };
            if let Some(reference) = terminal_ref {
                let terminal = terminal_handoffs
                    .get(&reference.ref_id)
                    .ok_or(StoreError("terminal handoff is unreachable"))?;
                validate_terminal_handoff(intent, terminal)?;
            }
        }
        Ok(())
    }

    pub(super) fn read_object_bytes(
        &self,
        reference: &AuthorityObjectRefV1,
    ) -> Result<Vec<u8>, StoreError> {
        let kind = self
            .objects
            .open_directory(kind_slug(reference.object_kind))
            .map_err(|_| StoreError("open parent-owned object kind"))?;
        let version = kind
            .open_directory(&format!("v{}", reference.schema_version))
            .map_err(|_| StoreError("open parent-owned object version"))?;
        version
            .open_file(&format!("{}.obj", reference.ref_id))
            .map_err(|_| StoreError("open parent-owned object"))?
            .read_all()
            .map_err(|_| StoreError("read parent-owned object"))
    }

    pub(super) fn remove_matching_marker(&self, root: &StateRootV1) -> Result<(), StoreError> {
        match self
            .authority
            .entry_kind(INIT_FILE)
            .map_err(|_| StoreError("inspect committed initialization marker"))?
        {
            None => Ok(()),
            Some(EntryKind::RegularFile) => {
                self.validate_matching_marker_if_present(root)?;
                self.authority
                    .unlink_file(INIT_FILE)
                    .map_err(|_| StoreError("remove committed initialization marker"))
            }
            Some(_) => Err(StoreError("committed initialization marker is unsafe")),
        }
    }

    pub(super) fn validate_matching_marker_if_present(
        &self,
        root: &StateRootV1,
    ) -> Result<(), StoreError> {
        match self
            .authority
            .entry_kind(INIT_FILE)
            .map_err(|_| StoreError("inspect committed initialization marker"))?
        {
            None => Ok(()),
            Some(EntryKind::RegularFile) => {
                let marker = self.read_marker()?;
                let key = root
                    .commitment_key_registry
                    .get(&marker.initial_key_id)
                    .ok_or(StoreError("initial commitment key is absent from root"))?;
                if marker.authority_store_id != root.authority_store_id
                    || marker.bootstrap_home != root.bootstrap_home
                    || marker.created_at != key.created_at
                    || marker.created_at != root.greenfield_namespace_certificate.certified_at
                {
                    return Err(StoreError("committed initialization marker mismatch"));
                }
                Ok(())
            }
            Some(_) => Err(StoreError("committed initialization marker is unsafe")),
        }
    }

    pub(super) fn validate_matching_marker_if_present_v2(
        &self,
        root: &StateRootV2,
    ) -> Result<(), StoreError> {
        match self
            .authority
            .entry_kind(INIT_FILE)
            .map_err(|_| StoreError("inspect committed V2 initialization marker"))?
        {
            None => Ok(()),
            Some(EntryKind::RegularFile) => {
                let marker = self.read_marker()?;
                let key = root
                    .commitment_key_registry
                    .get(&marker.initial_key_id)
                    .ok_or(StoreError("initial commitment key is absent from V2 root"))?;
                if marker.authority_store_id != root.authority_store_id
                    || marker.bootstrap_home != root.bootstrap_home
                    || marker.created_at != key.created_at
                    || marker.created_at != root.greenfield_namespace_certificate.certified_at
                {
                    return Err(StoreError("committed V2 initialization marker mismatch"));
                }
                Ok(())
            }
            Some(_) => Err(StoreError("committed V2 initialization marker is unsafe")),
        }
    }

    pub(super) fn remove_matching_marker_v2(&self, root: &StateRootV2) -> Result<(), StoreError> {
        self.validate_matching_marker_if_present_v2(root)?;
        if self
            .authority
            .entry_kind(INIT_FILE)
            .map_err(|_| StoreError("inspect V2 initialization marker for removal"))?
            .is_some()
        {
            self.authority
                .unlink_file(INIT_FILE)
                .map_err(|_| StoreError("remove committed V2 initialization marker"))?;
        }
        Ok(())
    }

    pub(super) fn keys_empty(&self) -> Result<bool, StoreError> {
        self.keys
            .entries()
            .map(|entries| entries.is_empty())
            .map_err(|_| StoreError("enumerate authority keys"))
    }

    pub(super) fn objects_empty(&self) -> Result<bool, StoreError> {
        self.objects
            .entries()
            .map(|entries| entries.is_empty())
            .map_err(|_| StoreError("enumerate authority objects"))
    }
}

pub(super) fn validate_resume_identity(
    resume: &ResumeHandleHashInputV1,
    session_id: &str,
    participant_id: &str,
    backend_id: &str,
    protocol: &str,
) -> Result<(), StoreError> {
    if resume.orchestration_session_id != session_id
        || resume.participant_id != participant_id
        || resume.backend_id != backend_id
        || resume.protocol != protocol
    {
        Err(StoreError(
            "resume handle identity disagrees with parent graph",
        ))
    } else {
        Ok(())
    }
}

fn validate_terminal_handoff(
    intent: &HostSessionTransitionIntentV1,
    terminal: &TerminalHandoffHashInputV1,
) -> Result<(), StoreError> {
    let (state, application_result_ref) = match &intent.state {
        HostSessionTransitionIntentStateV1::Applied {
            application_result_ref,
            ..
        } => (
            TerminalHandoffStateV1::Applied,
            Some(application_result_ref),
        ),
        HostSessionTransitionIntentStateV1::Rejected { reason, .. } => {
            (TerminalHandoffStateV1::Rejected { reason: *reason }, None)
        }
        HostSessionTransitionIntentStateV1::Expired { .. } => {
            (TerminalHandoffStateV1::Expired, None)
        }
        HostSessionTransitionIntentStateV1::Issued
        | HostSessionTransitionIntentStateV1::Claimed { .. } => {
            return Err(StoreError("nonterminal intent has a terminal handoff"))
        }
    };
    let input_acceptance_ref = match &intent.input_handoff {
        HostSessionTransitionInputHandoffV1::Accepted { acceptance_ref, .. } => {
            Some(acceptance_ref)
        }
        _ => None,
    };
    let (post_turn_completion_ref, post_turn_application_result_ref) = match &intent.state {
        HostSessionTransitionIntentStateV1::Applied { post_turn, .. } => match post_turn.as_ref() {
            HostSessionPostTurnApplicationV1::Applied {
                completion_ref,
                application_result_ref,
                ..
            } => (
                Some(completion_ref.as_ref()),
                Some(application_result_ref.as_ref()),
            ),
            _ => (None, None),
        },
        _ => (None, None),
    };
    if terminal.intent_id != intent.intent_id
        || terminal.run_id != intent.run_id
        || terminal.payload_commitment != intent.payload_commitment
        || terminal.terminal_state != state
        || terminal.application_result_ref.as_ref() != application_result_ref
        || terminal.input_acceptance_ref.as_ref() != input_acceptance_ref
        || terminal.post_turn_completion_ref.as_ref() != post_turn_completion_ref
        || terminal.post_turn_application_result_ref.as_ref() != post_turn_application_result_ref
    {
        Err(StoreError("terminal handoff and intent graph disagree"))
    } else {
        Ok(())
    }
}

fn validate_terminal_handoff_v2(
    intent: &HostSessionTransitionIntentV2,
    terminal: &TerminalHandoffHashInputV1,
) -> Result<(), StoreError> {
    let (state, application_result_ref) = match &intent.state {
        HostSessionTransitionIntentStateV2::Applied {
            application_result_ref,
            ..
        } => (
            TerminalHandoffStateV1::Applied,
            Some(application_result_ref),
        ),
        HostSessionTransitionIntentStateV2::Rejected { reason, .. } => {
            (TerminalHandoffStateV1::Rejected { reason: *reason }, None)
        }
        HostSessionTransitionIntentStateV2::Expired { .. } => {
            (TerminalHandoffStateV1::Expired, None)
        }
        HostSessionTransitionIntentStateV2::Issued
        | HostSessionTransitionIntentStateV2::Claimed { .. } => {
            return Err(StoreError("nonterminal V2 Start has a terminal handoff"))
        }
    };
    let input_acceptance_ref = match &intent.input_handoff {
        HostSessionTransitionInputHandoffV1::Accepted { acceptance_ref, .. } => {
            Some(acceptance_ref)
        }
        _ => None,
    };
    if terminal.intent_id != intent.intent_id
        || terminal.run_id != intent.run_id
        || terminal.payload_commitment != intent.payload_commitment
        || terminal.terminal_state != state
        || terminal.application_result_ref.as_ref() != application_result_ref
        || terminal.input_acceptance_ref.as_ref() != input_acceptance_ref
        || terminal.post_turn_completion_ref.is_some()
        || terminal.post_turn_application_result_ref.is_some()
    {
        Err(StoreError("V2 terminal handoff and Start intent disagree"))
    } else {
        Ok(())
    }
}

fn create_or_open_lock(
    directory: &TrustedDirectory,
    strict: bool,
) -> Result<TrustedFile, StoreError> {
    match directory
        .entry_kind(ROOT_LOCK_FILE)
        .map_err(|_| StoreError("inspect authority lock file"))?
    {
        None if strict => Err(StoreError("authority lock file is missing")),
        None => create_or_join_lock(directory),
        Some(EntryKind::RegularFile) => directory
            .open_file(ROOT_LOCK_FILE)
            .map_err(|_| StoreError("open authority lock file")),
        Some(_) => Err(StoreError("authority lock entry is unsafe")),
    }
}

pub(super) fn create_or_join_lock(directory: &TrustedDirectory) -> Result<TrustedFile, StoreError> {
    match directory.create_file(ROOT_LOCK_FILE) {
        Ok(file) => {
            file.sync()
                .map_err(|_| StoreError("sync authority lock file"))?;
            directory
                .sync()
                .map_err(|_| StoreError("sync authority lock directory"))?;
            Ok(file)
        }
        Err(error) if error.is_already_exists() => {
            let file = directory
                .open_file(ROOT_LOCK_FILE)
                .map_err(|_| StoreError("join authority lock creation"))?;
            file.sync()
                .map_err(|_| StoreError("sync joined authority lock file"))?;
            directory
                .sync()
                .map_err(|_| StoreError("sync joined authority lock directory"))?;
            Ok(file)
        }
        Err(_) => Err(StoreError("create authority lock file")),
    }
}

pub(super) struct LockedClassification {
    pub(super) classification: BootstrapClassificationV1,
    pub(super) authority_classification: BootstrapClassificationV1,
    pub(super) legacy: LegacyObservation,
    pub(super) root: Option<StateRootV1>,
}
