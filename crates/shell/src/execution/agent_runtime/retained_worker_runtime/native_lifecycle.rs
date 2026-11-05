//! Native bootstrap facts, distinct from logical retained-worker retirement.
//!
//! This separately versioned envelope lives in the existing mutable admission
//! registry. Its MAC key is selected exclusively by that registry's existing key
//! header. Neither a parsed report nor a valid MAC grants later permission.
use super::*;
use transport_api_types::{
    E2MemberLaunchKindV1, OrdinaryGatewayRuntimeIdentityV1, RetainedBootstrapCompletionV1,
};

const DOMAIN: &str = "substrate.retained-worker.native-lifecycle.v1";

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct RetainedNativeBootstrapLifecycleV1 {
    body: NativeBootstrapLifecycleBodyV1,
    mac_hex: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
struct NativeBootstrapLifecycleBodyV1 {
    schema_version: u32,
    domain: String,
    authority_store_id: String,
    key_id: String,
    revision: u64,
    completions_by_session: BTreeMap<String, BTreeMap<String, NativeBootstrapCompletionRecordV1>>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
struct NativeBootstrapCompletionRecordV1 {
    issuer_request_id: String,
    canonical_spawn_fingerprint: RetainedWorkerAdmissionCommitmentV1,
    registration: RetainedWorkerAdmissionRegistrationV1,
    admission_record_revision: u64,
    registered_frame: RuntimeFrameIdentityV1,
    registered_event: RuntimeEventIdentityV1,
    registered_event_sha256: String,
    #[serde(with = "transport_completion_codec")]
    completion: RetainedBootstrapCompletionV1,
}

// Transport enum strings and HSA A1 enum tags are different contracts. Encode
// the closed, validated transport report as JSON data inside the A1 container;
// decoding always runs the original strict report decoder and commitment check.
// The surrounding A1 canonical round trip and lifecycle MAC still cover every
// byte. No generic HSA enum or object decoder is broadened.
mod transport_completion_codec {
    use super::RetainedBootstrapCompletionV1;
    use serde::{Deserialize, Serialize};

    pub fn serialize<S: serde::Serializer>(
        value: &RetainedBootstrapCompletionV1,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        value.validate().map_err(serde::ser::Error::custom)?;
        serde_json::to_value(value)
            .map_err(serde::ser::Error::custom)?
            .serialize(serializer)
    }

    pub fn deserialize<'de, D: serde::Deserializer<'de>>(
        deserializer: D,
    ) -> Result<RetainedBootstrapCompletionV1, D::Error> {
        let value = serde_json::Value::deserialize(deserializer)?;
        serde_json::from_value(value).map_err(serde::de::Error::custom)
    }
}

impl RetainedNativeBootstrapLifecycleV1 {
    #[cfg(target_os = "linux")]
    pub(super) fn validate_population(
        &self,
        population: &super::super::dispatch_policy_commitment::OrdinaryBootstrapPopulationV1,
    ) -> Result<(), RetainedWorkerRuntimeError> {
        if self
            .body
            .completions_by_session
            .values()
            .flat_map(BTreeMap::values)
            .any(|record| {
                !population.contains_activation(&record.completion.observation().activation)
            })
        {
            return Err(RetainedWorkerRuntimeError(
                "native bootstrap E2 population coverage differs".into(),
            ));
        }
        Ok(())
    }

    pub(super) fn native_completion_for(
        &self,
        session: &str,
        participant: &str,
    ) -> Option<&RetainedBootstrapCompletionV1> {
        self.body
            .completions_by_session
            .get(session)?
            .get(participant)
            .map(|record| &record.completion)
    }

    pub(super) fn native_session_locators(&self) -> BTreeSet<(String, String)> {
        self.body
            .completions_by_session
            .iter()
            .flat_map(|(session, records)| {
                records.values().map(move |record| {
                    (
                        session.clone(),
                        record.completion.observation().native_session_id.clone(),
                    )
                })
            })
            .collect()
    }

    fn new(key: &RetainedWorkerAdmissionKeyEnvelopeV1) -> Self {
        Self {
            body: NativeBootstrapLifecycleBodyV1 {
                schema_version: 1,
                domain: DOMAIN.into(),
                authority_store_id: key.authority_store_id.clone(),
                key_id: key.key_id.clone(),
                revision: 0,
                completions_by_session: BTreeMap::new(),
            },
            mac_hex: String::new(),
        }
    }

    fn expected_mac(
        &self,
        key: &RetainedWorkerAdmissionKeyEnvelopeV1,
    ) -> Result<String, RetainedWorkerRuntimeError> {
        let bytes = encode_canonical(&self.body, "encode native lifecycle MAC input")?;
        let mut input = b"substrate.retained-worker.native-lifecycle.mac-input.v1\0".to_vec();
        append_len64(&mut input, &bytes)?;
        Ok(lower_hex(&hmac_sha256(&key.secret_key, &input)))
    }

    pub(super) fn verify_mac(
        &self,
        key: &RetainedWorkerAdmissionKeyEnvelopeV1,
    ) -> Result<(), RetainedWorkerRuntimeError> {
        if self.body.schema_version != 1
            || self.body.domain != DOMAIN
            || self.body.authority_store_id != key.authority_store_id
            || self.body.key_id != key.key_id
            || !is_lower_hex(&self.mac_hex, 64)
            || self.body.completions_by_session.is_empty()
        {
            return Err(RetainedWorkerRuntimeError(
                "invalid native lifecycle header".into(),
            ));
        }
        let expected = self.expected_mac(key)?;
        let difference = expected
            .bytes()
            .zip(self.mac_hex.bytes())
            .fold(0u8, |value, (a, b)| value | (a ^ b));
        if difference != 0 {
            return Err(RetainedWorkerRuntimeError(
                "native lifecycle MAC mismatch".into(),
            ));
        }
        Ok(())
    }

    pub(super) fn validate_registry_joins(
        &self,
        registry: &RetainedWorkerAdmissionRegistryV1,
    ) -> Result<(), RetainedWorkerRuntimeError> {
        let mut count = 0u64;
        let mut native_sessions = BTreeSet::new();
        for (session, completions) in &self.body.completions_by_session {
            if completions.is_empty() {
                return Err(RetainedWorkerRuntimeError(
                    "empty native lifecycle session".into(),
                ));
            }
            for (participant, completed) in completions {
                count = count.checked_add(1).ok_or_else(|| {
                    RetainedWorkerRuntimeError("native lifecycle count overflow".into())
                })?;
                let record = registry
                    .records_by_session
                    .get(session)
                    .and_then(|records| records.get(participant))
                    .ok_or_else(|| {
                        RetainedWorkerRuntimeError(
                            "native lifecycle has no issued admission".into(),
                        )
                    })?;
                completed.validate_join(record)?;
                if !native_sessions.insert((
                    record.orchestration_session_id.clone(),
                    completed.completion.observation().native_session_id.clone(),
                )) {
                    return Err(RetainedWorkerRuntimeError(
                        "native lifecycle reused a native session".into(),
                    ));
                }
            }
        }
        if self.body.authority_store_id != registry.authority_store_id
            || self.body.key_id != registry.commitment_key.key_id
            || count == 0
            || self.body.revision != count
        {
            return Err(RetainedWorkerRuntimeError(
                "native lifecycle registry history is inexact".into(),
            ));
        }
        Ok(())
    }
}

impl NativeBootstrapCompletionRecordV1 {
    fn validate_join(
        &self,
        record: &RetainedWorkerAdmissionRecordV1,
    ) -> Result<(), RetainedWorkerRuntimeError> {
        let fail = || {
            RetainedWorkerRuntimeError(
                "native bootstrap changed issued R0/Registered ancestry".into(),
            )
        };
        self.completion.validate().map_err(|_| fail())?;
        self.registered_frame.validate().map_err(|_| fail())?;
        self.registered_event.validate().map_err(|_| fail())?;
        let observed = self.completion.observation();
        let OrdinaryGatewayRuntimeIdentityV1::WorldWorker {
            orchestration_session_id,
            participant_id,
            backend_id,
            run_id,
            world_id,
            world_generation,
            ..
        } = &observed.runtime
        else {
            return Err(fail());
        };
        if self.issuer_request_id != record.issuer_request_id
            || self.canonical_spawn_fingerprint != record.canonical_spawn_fingerprint
            || admission_registration(&record.state) != Some(&self.registration)
            || !is_lower_hex(&self.registered_event_sha256, 64)
            || self.admission_record_revision == 0
            || self.admission_record_revision > record.record_revision
            || observed.source_start.authority_store_id != record.authority_store_id
            || orchestration_session_id != &record.orchestration_session_id
            || participant_id != &record.retained_participant_id
            || backend_id != &record.backend_id
            || run_id != &record.bootstrap_run_id
            || world_id != &record.world_binding.world_id
            || *world_generation != record.world_binding.world_generation
            || observed.activation.launch_kind != E2MemberLaunchKindV1::FreshSpawn
            || observed.activation.source_participant_id.is_some()
            || self.registered_frame.stream_id != observed.frame_identity.stream_id
            || self.registered_frame.frame_sequence >= observed.frame_identity.frame_sequence
            || self.registered_event.event_sequence >= observed.event_identity.event_sequence
        {
            return Err(fail());
        }
        if let RetainedWorkerAdmissionStateV1::Routable {
            stream_id,
            registered_frame_sequence,
            registered_event_id,
            registered_event_sequence,
            transport_span_id,
            ..
        } = &record.state
        {
            if stream_id != &self.registered_frame.stream_id
                || *registered_frame_sequence != self.registered_frame.frame_sequence
                || registered_event_id != &self.registered_event.event_id
                || *registered_event_sequence != self.registered_event.event_sequence
                || transport_span_id.as_deref() != Some(observed.span_id.as_str())
            {
                return Err(fail());
            }
        }
        Ok(())
    }
}

impl RetainedWorkerRuntime {
    /// Record a newly witnessed service-owned bootstrap completion. This leaves
    /// the genuine R0 admission Routable; it never revives old Terminal truth.
    #[cfg(target_os = "linux")]
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn record_native_bootstrap_completion(
        &self,
        authority: &HostSessionAuthority,
        selection: &super::super::native_launch_context::AuthenticatedOrdinarySelectionV1,
        registered_frame: &RuntimeFrameIdentityV1,
        registered: &AgentEvent,
        completion: &RetainedBootstrapCompletionV1,
    ) -> Result<RetainedWorkerAdmissionRecordV1, RetainedWorkerRuntimeError> {
        use super::super::dispatch_policy_commitment::{
            authenticate_dispatch_policy_commitment, DispatchPolicyCommitmentRefV1,
        };
        let fail = || {
            RetainedWorkerRuntimeError("native bootstrap source/Registered join conflicts".into())
        };
        if !std::ptr::eq(authority, selection.held_authority().as_ref()) {
            return Err(fail());
        }
        let observed = completion.observation();
        let expected_start =
            crate::execution::orchestrator_world_dispatch::ordinary_start_selection_carrier(
                selection,
            )
            .map_err(|_| fail())?;
        let cap = authenticate_dispatch_policy_commitment(
            authority,
            &DispatchPolicyCommitmentRefV1 {
                authority_store_id: observed
                    .activation
                    .commitment_ref
                    .authority_store_id
                    .clone(),
                commitment_id: observed.activation.commitment_ref.commitment_id.clone(),
                exact_linkage_hash: observed
                    .activation
                    .commitment_ref
                    .exact_linkage_hash
                    .clone(),
            },
        )
        .map_err(|_| fail())?;
        let activation = cap.member_launch_activation_carrier().map_err(|_| fail())?;
        completion
            .validate_for_source(&expected_start, &observed.runtime, &activation)
            .map_err(|_| fail())?;
        let OrdinaryGatewayRuntimeIdentityV1::WorldWorker {
            orchestration_session_id,
            caller_participant_id: _,
            participant_id,
            backend_id,
            run_id,
            world_id,
            world_generation,
        } = &observed.runtime
        else {
            return Err(fail());
        };
        registered
            .validate_identity_contract()
            .map_err(|_| fail())?;
        let event_identity = registered.event_identity.as_ref().ok_or_else(fail)?;
        if registered.kind != AgentEventKind::Registered
            || registered.orchestration_session_id != *orchestration_session_id
            || registered.run_id != *run_id
            || registered.participant_id.as_deref() != Some(participant_id)
            || registered.backend_id.as_deref() != Some(backend_id)
            || registered.world_id.as_deref() != Some(world_id)
            || registered.world_generation != Some(*world_generation)
            || registered.role.as_deref() != Some("member")
            || registered.parent_participant_id.is_some()
            || registered.resumed_from_participant_id.is_some()
            || registered.span_id.as_deref() != Some(observed.span_id.as_str())
            || registered
                .data
                .get("schema")
                .and_then(serde_json::Value::as_str)
                != Some(super::super::mapping::SESSION_HANDLE_SCHEMA_V1)
            || ["/internal/uaa_session_id", "/session/id"]
                .into_iter()
                .find_map(|pointer| {
                    registered
                        .data
                        .pointer(pointer)
                        .and_then(serde_json::Value::as_str)
                        .map(str::trim)
                        .filter(|identity| !identity.is_empty())
                })
                != Some(observed.native_session_id.as_str())
        {
            return Err(fail());
        }
        // Preserve every received Registered field across exact retries, including
        // its original native/logical locator distinction and source timestamp.
        let mut event_hash = Sha256::new();
        event_hash.update(b"substrate.retained-worker.registered-event.v1\0");
        event_hash.update(serde_json::to_vec(registered).map_err(|_| fail())?);
        let registered_event_sha256 = format!("{:x}", event_hash.finalize());
        let issued = self
            .read_existing_admission_record(authority, orchestration_session_id, participant_id)?
            .ok_or_else(fail)?;
        self.resolve_admitted_record_graph(authority, &issued)?;
        self.validate_admission_authority_ancestry(authority, &issued)?;
        let storage = super::super::host_session_authority::store::retained_worker_admission_storage_for_authority(authority)
            .map_err(|error| RetainedWorkerRuntimeError(error.to_string()))?;
        let mut nonce = [0u8; 16];
        rand::rngs::OsRng.fill_bytes(&mut nonce);
        storage
            .transaction(|transaction| {
                let bytes = transaction.read_registry()?;
                let keys = transaction.read_keys()?;
                Ok((|| {
                    let mut registry: RetainedWorkerAdmissionRegistryV1 = decode_canonical(
                        bytes.as_deref().ok_or_else(fail)?,
                        "decode existing native lifecycle registry",
                    )?;
                    let (_, key) = load_committed_admission_key(
                        transaction.authority_root().authority_store_id(),
                        &registry,
                        &keys,
                    )?;
                    validate_admission_registry(&registry, transaction.authority_root())?;
                    let record = registry
                        .records_by_session
                        .get(orchestration_session_id)
                        .and_then(|records| records.get(participant_id))
                        .cloned()
                        .ok_or_else(fail)?;
                    if record != issued {
                        return Err(fail());
                    }
                    let registered_record = NativeBootstrapCompletionRecordV1 {
                        issuer_request_id: record.issuer_request_id.clone(),
                        canonical_spawn_fingerprint: record.canonical_spawn_fingerprint.clone(),
                        registration: admission_registration(&record.state)
                            .cloned()
                            .ok_or_else(fail)?,
                        admission_record_revision: record.record_revision,
                        registered_frame: registered_frame.clone(),
                        registered_event: event_identity.clone(),
                        registered_event_sha256: registered_event_sha256.clone(),
                        completion: completion.clone(),
                    };
                    let mut lifecycle = registry
                        .native_lifecycle_v1
                        .clone()
                        .unwrap_or_else(|| RetainedNativeBootstrapLifecycleV1::new(&key));
                    let records = lifecycle
                        .body
                        .completions_by_session
                        .entry(orchestration_session_id.clone())
                        .or_default();
                    if let Some(existing) = records.get(participant_id) {
                        // A later logical terminal remains authoritative. Exact replay
                        // joins the original immutable completion without any rewrite.
                        let mut candidate = registered_record;
                        candidate.admission_record_revision = existing.admission_record_revision;
                        if existing != &candidate {
                            return Err(fail());
                        }
                        return Ok(record);
                    }
                    if !matches!(
                        record.state,
                        RetainedWorkerAdmissionStateV1::Routable { .. }
                    ) {
                        return Err(fail());
                    }
                    registered_record.validate_join(&record)?;
                    records.insert(participant_id.clone(), registered_record);
                    lifecycle.body.revision =
                        lifecycle.body.revision.checked_add(1).ok_or_else(fail)?;
                    lifecycle.mac_hex = lifecycle.expected_mac(&key)?;
                    registry.native_lifecycle_v1 = Some(lifecycle);
                    validate_admission_registry(&registry, transaction.authority_root())?;
                    load_committed_admission_key(
                        transaction.authority_root().authority_store_id(),
                        &registry,
                        &keys,
                    )?;
                    let bytes = encode_canonical(&registry, "encode native bootstrap lifecycle")?;
                    transaction
                        .replace_registry(
                            &format!("admission-registry--{}.tmp", lower_hex(&nonce)),
                            &bytes,
                        )
                        .map_err(|error| RetainedWorkerRuntimeError(error.to_string()))?;
                    Ok(record)
                })())
            })
            .map_err(|error| RetainedWorkerRuntimeError(error.to_string()))?
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_native_lifecycle_mac_and_closed_codec() {
        // MAC codec data only: even this MAC-valid empty session is refused by
        // registry/history validation and cannot establish native ownership.
        let key = RetainedWorkerAdmissionKeyEnvelopeV1 {
            schema_version: 1,
            authority_store_id: "store-test".into(),
            key_id: format!("adk_{}", "12".repeat(16)),
            created_at: TimestampV1::parse("2026-11-03T12:00:00.000000000Z").unwrap(),
            algorithm: RetainedWorkerAdmissionCommitmentAlgorithmV1::HmacSha256,
            secret_key: [7; 32],
        };
        let mut lifecycle = RetainedNativeBootstrapLifecycleV1::new(&key);
        lifecycle
            .body
            .completions_by_session
            .insert("session-test".into(), BTreeMap::new());
        lifecycle.body.revision = 1;
        lifecycle.mac_hex = lifecycle.expected_mac(&key).unwrap();
        lifecycle.verify_mac(&key).unwrap();
        let encoded = encode_canonical(&lifecycle, "encode test lifecycle").unwrap();
        let decoded: RetainedNativeBootstrapLifecycleV1 =
            decode_canonical(&encoded, "decode test lifecycle").unwrap();
        assert_eq!(decoded, lifecycle);
        for defect in [
            "domain", "version", "store", "key", "revision", "payload", "mac",
        ] {
            let mut wrong = lifecycle.clone();
            match defect {
                "domain" => wrong.body.domain.push('x'),
                "version" => wrong.body.schema_version += 1,
                "store" => wrong.body.authority_store_id.push('x'),
                "key" => wrong.body.key_id = format!("adk_{}", "34".repeat(16)),
                "revision" => wrong.body.revision += 1,
                "payload" => {
                    wrong
                        .body
                        .completions_by_session
                        .insert("other-session".into(), BTreeMap::new());
                }
                "mac" => wrong.mac_hex = "0".repeat(64),
                _ => unreachable!(),
            }
            assert!(wrong.verify_mac(&key).is_err(), "{defect}");
        }
        let mut wrong_key = key.clone();
        wrong_key.secret_key[0] ^= 1;
        assert!(lifecycle.verify_mac(&wrong_key).is_err());
        let mut unknown = serde_json::to_value(&lifecycle).unwrap();
        unknown["extra"] = true.into();
        assert!(serde_json::from_value::<RetainedNativeBootstrapLifecycleV1>(unknown).is_err());
    }
}
