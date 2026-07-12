use super::*;

pub(super) fn rotate_commitment_key_with(
    path: &std::path::Path,
    expected_root_revision: u64,
    material: Option<InitializationMaterialV1>,
    stop: Option<KeyLifecycleCrashPointV1>,
) -> Result<StateRootV1, BootstrapError> {
    with_existing_semantic_preflight(path, |transaction| {
        transaction.require_expected_root(expected_root_revision)?;
        let material = match material {
            Some(material) => material,
            None => system_material()?,
        };
        let mut root = transaction.root.clone();
        let new_key_id = key_id(material.key_entropy);
        if root.commitment_key_registry.contains_key(&new_key_id) {
            return Err(BootstrapError("commitment key ID already exists"));
        }
        let envelope = AuthorityStoreCommitmentKeyFileV1 {
            authority_store_id: root.authority_store_id.clone(),
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
        let former_active = root.active_commitment_key_id.clone();
        root.commitment_key_registry
            .get_mut(&former_active)
            .ok_or(BootstrapError("active commitment key is missing"))?
            .state = AuthorityStoreCommitmentKeyStateV1::VerificationOnly;
        root.commitment_key_registry.insert(
            new_key_id.clone(),
            AuthorityStoreCommitmentKeyV1 {
                schema_version: 1,
                authority_store_id: root.authority_store_id.clone(),
                key_id: new_key_id.clone(),
                algorithm: AuthorityStoreCommitmentAlgorithmV1::HmacSha256,
                created_at: material.created_at,
                state: AuthorityStoreCommitmentKeyStateV1::Active,
            },
        );
        root.active_commitment_key_id = new_key_id;
        root.root_revision = root
            .root_revision
            .checked_add(1)
            .ok_or(BootstrapError("root revision overflow"))?;
        publish_replacement_root(
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
    validate_key_id(retiring_key_id)
        .map_err(|_| BootstrapError("retiring commitment key ID is invalid"))?;
    with_existing_semantic_preflight(path, |transaction| {
        transaction.require_expected_root(expected_root_revision)?;
        let mut root = transaction.root.clone();
        let record = root
            .commitment_key_registry
            .get(retiring_key_id)
            .ok_or(BootstrapError("retiring commitment key is unregistered"))?;
        if record.state == AuthorityStoreCommitmentKeyStateV1::Retired {
            transaction
                .layout
                .reconcile_key_files(&root)
                .map_err(|_| BootstrapError("finish retired key cleanup"))?;
            return Ok(root);
        }
        if record.state != AuthorityStoreCommitmentKeyStateV1::VerificationOnly
            || root_references_key(&root, retiring_key_id)?
        {
            return Err(BootstrapError("commitment key is not retirement eligible"));
        }
        root.commitment_key_registry
            .get_mut(retiring_key_id)
            .ok_or(BootstrapError("retiring commitment key disappeared"))?
            .state = AuthorityStoreCommitmentKeyStateV1::Retired;
        root.root_revision = root
            .root_revision
            .checked_add(1)
            .ok_or(BootstrapError("root revision overflow"))?;
        transaction.reconcile()?;
        if stop == Some(KeyLifecycleCrashPointV1::Reconciled) {
            return Err(BootstrapError("injected key retirement interruption"));
        }
        publish_replacement_root(
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
        transaction
            .layout
            .reconcile_key_files(&root)
            .map_err(|_| BootstrapError("remove retired commitment key"))?;
        Ok(root)
    })
}

fn root_references_key(root: &StateRootV1, key_id: &str) -> Result<bool, BootstrapError> {
    if root.active_commitment_key_id == key_id {
        return Ok(true);
    }
    let mut semantic = root.clone();
    semantic.commitment_key_registry.remove(key_id);
    let bytes = canonical_json::to_vec(&semantic)
        .map_err(|_| BootstrapError("encode key reachability view"))?;
    Ok(bytes
        .windows(key_id.len())
        .any(|window| window == key_id.as_bytes()))
}
