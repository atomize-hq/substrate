use super::*;

pub(super) trait ObjectVerificationRootV1 {
    fn authority_store_id(&self) -> &str;
    fn active_commitment_key_id(&self) -> &str;
    fn commitment_key_registry(
        &self,
    ) -> &std::collections::BTreeMap<String, AuthorityStoreCommitmentKeyV1>;
    fn object_index(&self) -> &std::collections::BTreeMap<String, AuthorityObjectIndexEntryV1>;
}

impl ObjectVerificationRootV1 for StateRootV1 {
    fn authority_store_id(&self) -> &str {
        &self.authority_store_id
    }

    fn active_commitment_key_id(&self) -> &str {
        &self.active_commitment_key_id
    }

    fn commitment_key_registry(
        &self,
    ) -> &std::collections::BTreeMap<String, AuthorityStoreCommitmentKeyV1> {
        &self.commitment_key_registry
    }

    fn object_index(&self) -> &std::collections::BTreeMap<String, AuthorityObjectIndexEntryV1> {
        &self.object_index
    }
}

impl ObjectVerificationRootV1 for StateRootV2 {
    fn authority_store_id(&self) -> &str {
        &self.authority_store_id
    }

    fn active_commitment_key_id(&self) -> &str {
        &self.active_commitment_key_id
    }

    fn commitment_key_registry(
        &self,
    ) -> &std::collections::BTreeMap<String, AuthorityStoreCommitmentKeyV1> {
        &self.commitment_key_registry
    }

    fn object_index(&self) -> &std::collections::BTreeMap<String, AuthorityObjectIndexEntryV1> {
        &self.object_index
    }
}

pub(super) fn publish_or_join_orphan<R: ObjectVerificationRootV1>(
    layout: &StoreLayout<'_>,
    root: &R,
    reference: &AuthorityObjectRefV1,
    bytes: &[u8],
    context: Option<&ObjectVerificationContextV1>,
    nonce_bytes: [u8; 16],
) -> Result<ObjectPublicationOutcomeV1, BootstrapError> {
    validate_orphan_candidate(layout, root, reference, bytes, context)?;
    let existing = existing_orphan_bytes(layout, reference)?;
    let kind_name = kind_slug(reference.object_kind);
    let version_name = format!("v{}", reference.schema_version);
    let final_name = format!("{}.obj", reference.ref_id);
    match existing {
        None => {
            let kind_directory = layout
                .objects
                .create_directory(kind_name)
                .map_err(|_| BootstrapError("create typed object kind directory"))?;
            let version_directory = kind_directory
                .create_directory(&version_name)
                .map_err(|_| BootstrapError("create typed object version directory"))?;
            let temp_name = TempNameV1::Object {
                ref_id: reference.ref_id.clone(),
                nonce: nonce(nonce_bytes),
            }
            .file_name();
            let mut temp = layout
                .tmp
                .create_file(&temp_name)
                .map_err(|_| BootstrapError("create typed object temp"))?;
            temp.write_all(bytes)
                .map_err(|_| BootstrapError("write typed object temp"))?;
            temp.sync()
                .map_err(|_| BootstrapError("sync typed object temp"))?;
            layout
                .tmp
                .rename_no_replace(&temp_name, temp, &version_directory, &final_name)
                .map_err(|_| BootstrapError("publish typed object"))?;
            let published = version_directory
                .open_file(&final_name)
                .map_err(|_| BootstrapError("open published typed object"))?
                .read_all()
                .map_err(|_| BootstrapError("read published typed object"))?;
            if published != bytes {
                return Err(BootstrapError("published typed object bytes changed"));
            }
            verify_object_bytes(layout, root, reference, &published, context, false)?;
            Ok(ObjectPublicationOutcomeV1::PublishedOrphan)
        }
        Some(_) => Ok(ObjectPublicationOutcomeV1::JoinedExactOrphan),
    }
}

pub(super) fn validate_orphan_candidate<R: ObjectVerificationRootV1>(
    layout: &StoreLayout<'_>,
    root: &R,
    reference: &AuthorityObjectRefV1,
    bytes: &[u8],
    context: Option<&ObjectVerificationContextV1>,
) -> Result<(), BootstrapError> {
    if root.object_index().contains_key(&reference.ref_id) {
        return Err(BootstrapError("typed object is already authoritative"));
    }
    match existing_orphan_bytes(layout, reference)? {
        None => verify_object_bytes(layout, root, reference, bytes, context, false),
        Some(existing) => {
            if existing != bytes {
                return Err(BootstrapError("existing typed object bytes differ"));
            }
            verify_object_bytes(layout, root, reference, &existing, context, true)
        }
    }
}

fn existing_orphan_bytes(
    layout: &StoreLayout<'_>,
    reference: &AuthorityObjectRefV1,
) -> Result<Option<Vec<u8>>, BootstrapError> {
    let kind_name = kind_slug(reference.object_kind);
    let version_name = format!("v{}", reference.schema_version);
    let final_name = format!("{}.obj", reference.ref_id);
    let existing = match layout
        .objects
        .entry_kind(kind_name)
        .map_err(|_| BootstrapError("inspect typed object kind directory"))?
    {
        None => None,
        Some(EntryKind::Directory) => {
            let kind_directory = layout
                .objects
                .open_directory(kind_name)
                .map_err(|_| BootstrapError("open typed object kind directory"))?;
            match kind_directory
                .entry_kind(&version_name)
                .map_err(|_| BootstrapError("inspect typed object version directory"))?
            {
                None => None,
                Some(EntryKind::Directory) => {
                    let version_directory = kind_directory
                        .open_directory(&version_name)
                        .map_err(|_| BootstrapError("open typed object version directory"))?;
                    match version_directory
                        .entry_kind(&final_name)
                        .map_err(|_| BootstrapError("inspect typed object location"))?
                    {
                        None => None,
                        Some(EntryKind::RegularFile) => Some(
                            version_directory
                                .open_file(&final_name)
                                .map_err(|_| BootstrapError("open existing typed object"))?
                                .read_all()
                                .map_err(|_| BootstrapError("read existing typed object"))?,
                        ),
                        Some(_) => return Err(BootstrapError("typed object location is unsafe")),
                    }
                }
                Some(_) => return Err(BootstrapError("typed object version route is unsafe")),
            }
        }
        Some(_) => return Err(BootstrapError("typed object kind route is unsafe")),
    };
    Ok(existing)
}

pub(super) fn reserved_object_location_is_absent(
    layout: &StoreLayout<'_>,
    reference: &AuthorityObjectRefV1,
) -> Result<bool, BootstrapError> {
    existing_orphan_bytes(layout, reference).map(|bytes| bytes.is_none())
}

pub(super) fn verify_object_bytes<R: ObjectVerificationRootV1>(
    layout: &StoreLayout<'_>,
    root: &R,
    reference: &AuthorityObjectRefV1,
    bytes: &[u8],
    context: Option<&ObjectVerificationContextV1>,
    allow_verification_only: bool,
) -> Result<(), BootstrapError> {
    validate_ref_id(&reference.ref_id)
        .map_err(|_| BootstrapError("typed object ref ID is invalid"))?;
    validate_object_commitment_rule(
        reference.object_kind,
        reference.schema_version,
        &reference.commitment,
    )
    .map_err(|_| BootstrapError("typed object commitment rule is invalid"))?;
    match &reference.commitment {
        AuthorityObjectCommitmentV1::CanonicalSha256 { digest_hex } => {
            if context.is_some() {
                return Err(BootstrapError(
                    "canonical object has unexpected parent context",
                ));
            }
            let actual = canonical_digest(reference.object_kind, bytes)?;
            if &actual != digest_hex {
                return Err(BootstrapError("canonical object commitment mismatch"));
            }
        }
        AuthorityObjectCommitmentV1::StoreHmacSha256 {
            key_id,
            domain,
            digest_hex,
        } => {
            let context =
                context.ok_or(BootstrapError("sensitive object parent context is missing"))?;
            let expected_domain = sensitive_domain(reference.object_kind)
                .ok_or(BootstrapError("sensitive object kind is invalid"))?;
            if domain.as_bytes() != expected_domain.as_bytes() {
                return Err(BootstrapError("sensitive object domain mismatch"));
            }
            if reference.object_kind == AuthorityObjectKindV1::TransitionTransportPayload {
                let parent = context
                    .parent_intent
                    .as_ref()
                    .ok_or(BootstrapError("transport payload parent is missing"))?;
                match parent {
                    VersionedObjectVerificationParentIntentV1::V1(parent) => {
                        if context.intent_id != parent.intent_id
                            || context.run_id != parent.run_id
                            || reference != &parent.transport_payload_ref
                        {
                            return Err(BootstrapError(
                                "transport ref or HMAC context disagrees with V1 parent",
                            ));
                        }
                        validate_transport_parent_v1(bytes, parent)?;
                    }
                    VersionedObjectVerificationParentIntentV1::V2(parent) => {
                        if context.intent_id != parent.intent_id
                            || context.run_id != parent.run_id
                            || reference != &parent.transport_payload_ref
                        {
                            return Err(BootstrapError(
                                "transport ref or HMAC context disagrees with V2 parent",
                            ));
                        }
                        validate_transport_parent_v2(bytes, parent)?;
                    }
                }
            } else if context.parent_intent.is_some() {
                return Err(BootstrapError(
                    "sensitive object has an unexpected full parent",
                ));
            }
            let record = root
                .commitment_key_registry()
                .get(key_id)
                .filter(|record| {
                    if allow_verification_only {
                        record.state != AuthorityStoreCommitmentKeyStateV1::Retired
                    } else {
                        record.state == AuthorityStoreCommitmentKeyStateV1::Active
                            && record.key_id == root.active_commitment_key_id()
                    }
                })
                .ok_or(BootstrapError("sensitive object key is unavailable"))?;
            let key_file = layout
                .keys
                .open_file(&format!("{}.key", record.key_id))
                .map_err(|_| BootstrapError("open sensitive object key"))?;
            let envelope = AuthorityStoreCommitmentKeyFileV1::decode(
                &key_file
                    .read_all()
                    .map_err(|_| BootstrapError("read sensitive object key"))?,
            )
            .map_err(|_| BootstrapError("decode sensitive object key"))?;
            let actual = store_hmac_sha256(
                &envelope.secret_key,
                expected_domain,
                root.authority_store_id(),
                &context.intent_id,
                Some(&context.run_id),
                bytes,
            )
            .map_err(|_| BootstrapError("verify sensitive object commitment"))?;
            if &actual != digest_hex {
                return Err(BootstrapError("sensitive object commitment mismatch"));
            }
        }
    }
    Ok(())
}

fn validate_transport_parent_v1(
    bytes: &[u8],
    intent: &HostSessionTransitionIntentV1,
) -> Result<(), BootstrapError> {
    use crate::execution::agent_runtime::host_session_authority::schema::TransitionTransportPayloadObjectV1;
    use crate::execution::agent_runtime::host_session_authority::validation::ValidatedCanonicalV1;

    let payload: TransitionTransportPayloadObjectV1 = canonical_json::from_slice(bytes)
        .map_err(|_| BootstrapError("transport payload bytes are invalid"))?;
    payload
        .validate()
        .map_err(|_| BootstrapError("transport payload schema is invalid"))?;
    if payload.intent_id != intent.intent_id
        || payload.mode != intent.mode
        || payload.orchestration_session_id != intent.orchestration_session_id
        || payload.shell_trace_session_id != intent.shell_trace_session_id
        || payload.caller != intent.caller
        || payload.source_authoritative_participant_id != intent.source_authoritative_participant_id
        || payload.target_authoritative_participant_id != intent.target_authoritative_participant_id
        || payload.target_participant_lease_token_ref != intent.target_participant_lease_token_ref
        || payload.run_id != intent.run_id
        || payload.resulting_authoritative_lineage != intent.resulting_authoritative_lineage
        || payload.workspace_binding != intent.workspace_binding
        || payload.world_binding != intent.world_binding
        || payload.descriptor_ref != intent.descriptor_ref
        || payload.host_attach_contract_ref != intent.host_attach_contract_ref
        || payload.resume_handle_ref != intent.resume_handle_ref
        || payload.transition_input_ref != intent.transition_input_ref
        || payload.post_turn_disposition != intent.post_turn_disposition
    {
        return Err(BootstrapError(
            "transport payload and parent intent disagree",
        ));
    }
    Ok(())
}

fn validate_transport_parent_v2(
    bytes: &[u8],
    intent: &HostSessionTransitionIntentV2,
) -> Result<(), BootstrapError> {
    use crate::execution::agent_runtime::host_session_authority::schema::TransitionTransportPayloadObjectV1;
    use crate::execution::agent_runtime::host_session_authority::validation::ValidatedCanonicalV1;

    let payload: TransitionTransportPayloadObjectV1 = canonical_json::from_slice(bytes)
        .map_err(|_| BootstrapError("transport payload bytes are invalid"))?;
    payload
        .validate()
        .map_err(|_| BootstrapError("transport payload schema is invalid"))?;
    if payload.intent_id != intent.intent_id
        || payload.mode != intent.mode
        || payload.orchestration_session_id != intent.orchestration_session_id
        || payload.shell_trace_session_id != intent.shell_trace_session_id
        || payload.caller != intent.caller
        || payload.source_authoritative_participant_id != intent.source_authoritative_participant_id
        || payload.target_authoritative_participant_id != intent.target_authoritative_participant_id
        || payload.target_participant_lease_token_ref != intent.target_participant_lease_token_ref
        || payload.run_id != intent.run_id
        || payload.resulting_authoritative_lineage != intent.resulting_authoritative_lineage
        || payload.workspace_binding != intent.workspace_binding
        || payload.world_binding != intent.world_binding
        || payload.descriptor_ref != intent.descriptor_ref
        || payload.host_attach_contract_ref != intent.host_attach_contract_ref
        || payload.resume_handle_ref != intent.resume_handle_ref
        || payload.transition_input_ref != intent.transition_input_ref
        || payload.post_turn_disposition != intent.post_turn_disposition
    {
        return Err(BootstrapError(
            "transport payload and V2 parent intent disagree",
        ));
    }
    Ok(())
}

pub(super) fn canonical_digest(
    kind: AuthorityObjectKindV1,
    bytes: &[u8],
) -> Result<String, BootstrapError> {
    match kind {
        AuthorityObjectKindV1::AgentDescriptor => {
            canonical_digest_as::<AgentDescriptorHashInputV1>(bytes)
        }
        AuthorityObjectKindV1::RetainedWorker => {
            canonical_digest_as::<RetainedWorkerObjectHashInputV1>(bytes)
        }
        AuthorityObjectKindV1::ResumeHandle => {
            canonical_digest_as::<ResumeHandleHashInputV1>(bytes)
        }
        AuthorityObjectKindV1::Policy => canonical_digest_as::<PolicyObjectHashInputV1>(bytes),
        AuthorityObjectKindV1::HostAttachContract => {
            canonical_digest_as::<HostAttachContractHashInputV1>(bytes)
        }
        AuthorityObjectKindV1::ApplicationResult => {
            canonical_digest_as::<ApplicationResultHashInputV1>(bytes)
        }
        AuthorityObjectKindV1::InputAcceptance => {
            canonical_digest_as::<InputAcceptanceHashInputV1>(bytes)
        }
        AuthorityObjectKindV1::PostTurnCompletion => {
            canonical_digest_as::<PostTurnCompletionHashInputV1>(bytes)
        }
        AuthorityObjectKindV1::TerminalHandoff => {
            canonical_digest_as::<TerminalHandoffHashInputV1>(bytes)
        }
        AuthorityObjectKindV1::TransitionTransportPayload
        | AuthorityObjectKindV1::TransitionInput
        | AuthorityObjectKindV1::LeaseToken => Err(BootstrapError(
            "sensitive object cannot use a canonical commitment",
        )),
    }
}

fn canonical_digest_as<T>(bytes: &[u8]) -> Result<String, BootstrapError>
where
    T: serde::de::DeserializeOwned + serde::Serialize + CanonicalHashInputV1,
{
    let value: T = canonical_json::from_slice(bytes)
        .map_err(|_| BootstrapError("canonical object bytes are invalid"))?;
    canonical_sha256(&value).map_err(|_| BootstrapError("canonical object is invalid"))
}

pub(super) fn sensitive_domain(kind: AuthorityObjectKindV1) -> Option<SensitiveDomainV1> {
    match kind {
        AuthorityObjectKindV1::TransitionInput => Some(SensitiveDomainV1::TransitionInput),
        AuthorityObjectKindV1::LeaseToken => Some(SensitiveDomainV1::ParticipantLeaseToken),
        AuthorityObjectKindV1::TransitionTransportPayload => {
            Some(SensitiveDomainV1::RawTransportPayload)
        }
        _ => None,
    }
}
