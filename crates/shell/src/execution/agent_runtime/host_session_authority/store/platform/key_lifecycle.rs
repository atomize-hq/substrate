use super::*;

pub(super) fn rotate_commitment_key_with(
    path: &std::path::Path,
    expected_root_revision: u64,
    material: Option<InitializationMaterialV1>,
    stop: Option<KeyLifecycleCrashPointV1>,
) -> Result<StateRootV1, BootstrapError> {
    match rotate_commitment_key_impl(path, expected_root_revision, material, stop, true)? {
        VersionedStateRoot::V1(root) => Ok(root),
        VersionedStateRoot::V2(_) | VersionedStateRoot::V3(_) => Err(BootstrapError(
            "V1 key rotation caller encountered StateRootV2",
        )),
    }
}

pub(super) fn rotate_commitment_key_versioned_with(
    path: &std::path::Path,
    expected_root_revision: u64,
    material: Option<InitializationMaterialV1>,
    stop: Option<KeyLifecycleCrashPointV1>,
) -> Result<VersionedStateRoot, BootstrapError> {
    rotate_commitment_key_impl(path, expected_root_revision, material, stop, false)
}

fn rotate_commitment_key_impl(
    path: &std::path::Path,
    expected_root_revision: u64,
    material: Option<InitializationMaterialV1>,
    stop: Option<KeyLifecycleCrashPointV1>,
    require_v1: bool,
) -> Result<VersionedStateRoot, BootstrapError> {
    with_existing_versioned_semantic_preflight(path, |transaction| {
        transaction.require_expected_root(expected_root_revision)?;
        if require_v1 && !matches!(transaction.root, VersionedStateRoot::V1(_)) {
            return Err(BootstrapError(
                "V1 key rotation caller encountered StateRootV2",
            ));
        }
        let material = match material {
            Some(material) => material,
            None => system_material()?,
        };
        let mut root = transaction.root.clone();
        let new_key_id = key_id(material.key_entropy);
        if root.commitment_key_registry().contains_key(&new_key_id) {
            return Err(BootstrapError("commitment key ID already exists"));
        }
        let envelope = AuthorityStoreCommitmentKeyFileV1 {
            authority_store_id: root.authority_store_id().to_string(),
            key_id: new_key_id.clone(),
            created_at: material.created_at.clone(),
            secret_key: material.secret_key,
        };
        transaction.reconcile()?;
        if stop == Some(KeyLifecycleCrashPointV1::Reconciled) {
            return Err(BootstrapError("injected key rotation interruption"));
        }
        transaction
            .layout
            .publish_key_envelope(&envelope, material.key_nonce)?;
        if stop == Some(KeyLifecycleCrashPointV1::KeyPublished) {
            return Err(BootstrapError("injected key rotation interruption"));
        }
        let former_active = root.active_commitment_key_id().to_string();
        root.commitment_key_registry_mut()
            .get_mut(&former_active)
            .ok_or(BootstrapError("active commitment key is missing"))?
            .state = AuthorityStoreCommitmentKeyStateV1::VerificationOnly;
        let authority_store_id = root.authority_store_id().to_string();
        root.commitment_key_registry_mut().insert(
            new_key_id.clone(),
            AuthorityStoreCommitmentKeyV1 {
                schema_version: 1,
                authority_store_id,
                key_id: new_key_id.clone(),
                algorithm: AuthorityStoreCommitmentAlgorithmV1::HmacSha256,
                created_at: material.created_at,
                state: AuthorityStoreCommitmentKeyStateV1::Active,
            },
        );
        *root.active_commitment_key_id_mut() = new_key_id;
        let next_revision = root
            .root_revision()
            .checked_add(1)
            .ok_or(BootstrapError("root revision overflow"))?;
        root.set_root_revision(next_revision);
        publish_versioned_replacement_root(
            transaction.layout,
            transaction.trusted_root,
            &transaction.legacy,
            &root,
            material.root_nonce,
            || transaction.validate_publication_candidate(expected_root_revision, &root),
        )?;
        Ok(root)
    })
}

pub(super) fn retire_commitment_key_with(
    path: &std::path::Path,
    retiring_key_id: &str,
    expected_root_revision: u64,
    root_nonce: [u8; 16],
    stop: Option<KeyLifecycleCrashPointV1>,
) -> Result<StateRootV1, BootstrapError> {
    match retire_commitment_key_impl(
        path,
        retiring_key_id,
        expected_root_revision,
        root_nonce,
        stop,
        true,
    )? {
        VersionedStateRoot::V1(root) => Ok(root),
        VersionedStateRoot::V2(_) | VersionedStateRoot::V3(_) => Err(BootstrapError(
            "V1 key retirement caller encountered StateRootV2",
        )),
    }
}

pub(super) fn retire_commitment_key_versioned_with(
    path: &std::path::Path,
    retiring_key_id: &str,
    expected_root_revision: u64,
    root_nonce: [u8; 16],
    stop: Option<KeyLifecycleCrashPointV1>,
) -> Result<VersionedStateRoot, BootstrapError> {
    retire_commitment_key_impl(
        path,
        retiring_key_id,
        expected_root_revision,
        root_nonce,
        stop,
        false,
    )
}

fn retire_commitment_key_impl(
    path: &std::path::Path,
    retiring_key_id: &str,
    expected_root_revision: u64,
    root_nonce: [u8; 16],
    stop: Option<KeyLifecycleCrashPointV1>,
    require_v1: bool,
) -> Result<VersionedStateRoot, BootstrapError> {
    validate_key_id(retiring_key_id)
        .map_err(|_| BootstrapError("retiring commitment key ID is invalid"))?;
    with_existing_versioned_semantic_preflight(path, |transaction| {
        transaction.require_expected_root(expected_root_revision)?;
        if require_v1 && !matches!(transaction.root, VersionedStateRoot::V1(_)) {
            return Err(BootstrapError(
                "V1 key retirement caller encountered StateRootV2",
            ));
        }
        let mut root = transaction.root.clone();
        let record = root
            .commitment_key_registry()
            .get(retiring_key_id)
            .ok_or(BootstrapError("retiring commitment key is unregistered"))?;
        if record.state == AuthorityStoreCommitmentKeyStateV1::Retired {
            match &root {
                VersionedStateRoot::V1(root) => transaction.layout.reconcile_key_files(root),
                VersionedStateRoot::V2(root) => transaction.layout.reconcile_key_files_v2(root),
                VersionedStateRoot::V3(root) => transaction.layout.reconcile_key_files_v3(root),
            }
            .map_err(|_| BootstrapError("finish retired key cleanup"))?;
            return Ok(root);
        }
        if record.state != AuthorityStoreCommitmentKeyStateV1::VerificationOnly
            || root_references_key_versioned(&root, retiring_key_id)?
        {
            return Err(BootstrapError("commitment key is not retirement eligible"));
        }
        root.commitment_key_registry_mut()
            .get_mut(retiring_key_id)
            .ok_or(BootstrapError("retiring commitment key disappeared"))?
            .state = AuthorityStoreCommitmentKeyStateV1::Retired;
        let next_revision = root
            .root_revision()
            .checked_add(1)
            .ok_or(BootstrapError("root revision overflow"))?;
        root.set_root_revision(next_revision);
        transaction.reconcile()?;
        if stop == Some(KeyLifecycleCrashPointV1::Reconciled) {
            return Err(BootstrapError("injected key retirement interruption"));
        }
        publish_versioned_replacement_root(
            transaction.layout,
            transaction.trusted_root,
            &transaction.legacy,
            &root,
            root_nonce,
            || transaction.validate_publication_candidate(expected_root_revision, &root),
        )?;
        if stop == Some(KeyLifecycleCrashPointV1::RootPublished) {
            return Err(BootstrapError("injected key retirement interruption"));
        }
        match &root {
            VersionedStateRoot::V1(root) => transaction.layout.reconcile_key_files(root),
            VersionedStateRoot::V2(root) => transaction.layout.reconcile_key_files_v2(root),
            VersionedStateRoot::V3(root) => transaction.layout.reconcile_key_files_v3(root),
        }
        .map_err(|_| BootstrapError("remove retired commitment key"))?;
        Ok(root)
    })
}

fn root_references_key_versioned(
    root: &VersionedStateRoot,
    key_id: &str,
) -> Result<bool, BootstrapError> {
    if root.active_commitment_key_id() == key_id {
        return Ok(true);
    }
    let mut semantic = root.clone();
    semantic.commitment_key_registry_mut().remove(key_id);
    let bytes = semantic
        .to_canonical_bytes()
        .map_err(|_| BootstrapError("encode versioned key reachability view"))?;
    Ok(bytes
        .windows(key_id.len())
        .any(|window| window == key_id.as_bytes()))
}
