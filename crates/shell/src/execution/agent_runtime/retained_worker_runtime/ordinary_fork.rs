//! Durable ordinary Fork ownership. A claimed native send is never reissued.
//! Checked native completion and original Spawn admission have distinct meaning.
use super::*;
use transport_api_types::{
    E2MemberLaunchActivationCarrierV1, E2MemberLaunchKindV1, OrdinaryStartSelectionCarrierV1,
    RetainedForkBootstrapCompletionV1,
};

#[cfg(test)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum OrdinaryForkR0CrashPointV1 {
    AfterReservation,
    AfterApplication,
}

#[cfg(test)]
thread_local! {
    static FORK_R0_CRASH_POINT: std::cell::Cell<Option<OrdinaryForkR0CrashPointV1>> = const { std::cell::Cell::new(None) };
}

#[cfg(test)]
pub(crate) fn set_ordinary_fork_r0_crash_point_for_test(point: OrdinaryForkR0CrashPointV1) {
    FORK_R0_CRASH_POINT.with(|slot| assert!(slot.replace(Some(point)).is_none()));
}

#[cfg(test)]
fn fork_r0_crash_for_test(
    point: OrdinaryForkR0CrashPointV1,
) -> Result<(), RetainedWorkerRuntimeError> {
    FORK_R0_CRASH_POINT.with(|slot| {
        if slot.get() == Some(point) {
            slot.set(None);
            return Err(RetainedWorkerRuntimeError(format!(
                "injected ordinary Fork R0 crash at {point:?}"
            )));
        }
        Ok(())
    })
}

const DOMAIN: &str = "substrate.retained-worker.ordinary-fork.v1";

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct OrdinaryForkRegistryV1 {
    body: OrdinaryForkRegistryBodyV1,
    mac_hex: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
struct OrdinaryForkRegistryBodyV1 {
    schema_version: u32,
    domain: String,
    authority_store_id: String,
    key_id: String,
    revision: u64,
    #[serde(with = "records_codec")]
    records_by_session: BTreeMap<String, BTreeMap<String, OrdinaryForkRecordV1>>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct OrdinaryForkRecordV1 {
    pub(crate) start: OrdinaryStartSelectionCarrierV1,
    pub(crate) activation: E2MemberLaunchActivationCarrierV1,
    pub(crate) descriptor: AgentDescriptorV1,
    pub(crate) native_claim_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    bootstrap_receiver_protocol: Option<u32>,
    source_cap_ref: transport_api_types::DispatchPolicyCommitmentRefCarrierV1,
    claimed_at: TimestampV1,
    max_live_retained_workers: u64,
    record_revision: u64,
    transport_start: Option<OrdinaryForkTransportStartV1>,
    completion: Option<OrdinaryForkCompletedV1>,
    registration_head: Option<OrdinaryForkRegistrationHeadV1>,
    pub(crate) registration: Option<RetainedWorkerAdmissionRegistrationV1>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
struct OrdinaryForkTransportStartV1 {
    frame: RuntimeFrameIdentityV1,
    span_id: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
struct OrdinaryForkCompletedV1 {
    registered_frame: RuntimeFrameIdentityV1,
    registered_event: RuntimeEventIdentityV1,
    registered_event_sha256: String,
    report: RetainedForkBootstrapCompletionV1,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
struct OrdinaryForkRegistrationHeadV1 {
    revision: u64,
    authority_revision: u64,
    authority_record_commitment: AuthorityObjectCommitmentV1,
}

// The enclosing store uses A1 tags; transport records retain their closed JSON
// codecs. This typed bridge preserves both codecs rather than widening either.
mod records_codec {
    use super::*;
    type Records = BTreeMap<String, BTreeMap<String, OrdinaryForkRecordV1>>;
    pub fn serialize<S: Serializer>(records: &Records, serializer: S) -> Result<S::Ok, S::Error> {
        serde_json::to_value(records)
            .map_err(serde::ser::Error::custom)?
            .serialize(serializer)
    }
    pub fn deserialize<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Records, D::Error> {
        let value = serde_json::Value::deserialize(deserializer)?;
        serde_json::from_value(value).map_err(serde::de::Error::custom)
    }
}

fn conflict() -> RetainedWorkerRuntimeError {
    RetainedWorkerRuntimeError("ordinary Fork claim/completion/R0 source conflicts".into())
}

impl OrdinaryForkRecordV1 {
    pub(super) fn accepts_initial_receiver_v1(&self) -> bool {
        self.bootstrap_receiver_protocol == Some(1)
            && self.transport_start.is_none()
            && self.completion.is_none()
            && self.registration_head.is_none()
            && self.registration.is_none()
    }
    pub(super) fn checked_native_completion(&self) -> Option<&RetainedForkBootstrapCompletionV1> {
        self.completion.as_ref().map(|completed| &completed.report)
    }

    pub(super) fn inspection_applied_registration(
        &self,
        root: &VersionedStateRoot,
    ) -> Result<Option<RetainedWorkerAdmissionRegistrationV1>, RetainedWorkerRuntimeError> {
        let preserved = preserved_start_root_view(root)?;
        let actual = fork_registration(self, preserved.as_ref())?;
        if self
            .registration
            .as_ref()
            .is_some_and(|stored| actual.as_ref() != Some(stored))
        {
            return Err(conflict());
        }
        Ok(actual)
    }

    fn registration_request_id(&self) -> String {
        format!("ordinary-fork-registration:{}", self.native_claim_id)
    }

    fn authority_registration_issuer(&self) -> String {
        format!(
            "retained-worker-registration:{}",
            self.registration_request_id()
        )
    }

    fn validate(&self) -> Result<(), RetainedWorkerRuntimeError> {
        if self
            .bootstrap_receiver_protocol
            .is_some_and(|version| version != 1)
        {
            return Err(conflict());
        }
        self.start.validate().map_err(|_| conflict())?;
        self.activation.validate().map_err(|_| conflict())?;
        self.source_cap_ref.validate().map_err(|_| conflict())?;
        AgentDescriptorHashInputV1 {
            schema_version: 1,
            descriptor: self.descriptor.clone(),
        }
        .validate()
        .map_err(|_| conflict())?;
        let activation = &self.activation;
        let expected_revision = 1u64
            .checked_add(u64::from(self.transport_start.is_some()))
            .and_then(|r| r.checked_add(u64::from(self.completion.is_some())))
            .and_then(|r| r.checked_add(self.registration_head.as_ref().map_or(0, |h| h.revision)))
            .and_then(|r| r.checked_add(u64::from(self.registration.is_some())))
            .ok_or_else(conflict)?;
        if activation.launch_kind != E2MemberLaunchKindV1::Fork
            || self.start.authority_store_id != activation.commitment_ref.authority_store_id
            || self.start.original_session_id != activation.orchestration_session_id
            || self.source_cap_ref.authority_store_id != self.start.authority_store_id
            || self.source_cap_ref == activation.immutable_worker_cap_ref
            || self.descriptor.backend_id != activation.target_backend_id
            || self.descriptor.execution_scope != AgentExecutionScopeV1::World
            || !valid_generated_id(&self.native_claim_id, "ofc_")
            || self.max_live_retained_workers == 0
            || self.record_revision != expected_revision
            || (self.registration_head.is_some() && self.completion.is_none())
            || (self.registration.is_some() && self.completion.is_none())
            || (self.registration.is_some() && self.registration_head.is_none())
        {
            return Err(conflict());
        }
        if let Some(start) = &self.transport_start {
            start.frame.validate().map_err(|_| conflict())?;
            if start.span_id.trim().is_empty() {
                return Err(conflict());
            }
        }
        if let Some(completed) = &self.completion {
            let start = self.transport_start.as_ref().ok_or_else(conflict)?;
            completed.report.validate().map_err(|_| conflict())?;
            completed
                .registered_frame
                .validate()
                .map_err(|_| conflict())?;
            completed
                .registered_event
                .validate()
                .map_err(|_| conflict())?;
            let observed = completed.report.completion().observation();
            if observed.source_start != self.start
                || observed.activation != self.activation
                || start.frame.stream_id != completed.registered_frame.stream_id
                || start.frame.frame_sequence >= completed.registered_frame.frame_sequence
                || start.span_id != observed.span_id
                || completed.registered_frame.stream_id != observed.frame_identity.stream_id
                || completed.registered_frame.frame_sequence
                    >= observed.frame_identity.frame_sequence
                || completed.registered_event.event_sequence
                    >= observed.event_identity.event_sequence
                || !is_lower_hex(&completed.registered_event_sha256, 64)
                || self
                    .registration_head
                    .as_ref()
                    .is_some_and(|head| head.authority_revision == 0 || head.revision == 0)
            {
                return Err(conflict());
            }
        }
        Ok(())
    }
}

impl OrdinaryForkRegistryV1 {
    pub(super) fn inspection_record(
        &self,
        session: &str,
        participant: &str,
    ) -> Option<(OrdinaryForkRecordV1, bool)> {
        let record = self
            .body
            .records_by_session
            .get(session)?
            .get(participant)?;
        Some((record.clone(), record.completion.is_some()))
    }

    #[cfg(target_os = "linux")]
    pub(super) fn validate_population(
        &self,
        population: &super::super::dispatch_policy_commitment::OrdinaryBootstrapPopulationV1,
    ) -> Result<(), RetainedWorkerRuntimeError> {
        if self
            .body
            .records_by_session
            .values()
            .flat_map(BTreeMap::values)
            .any(|record| !population.contains_activation(&record.activation))
        {
            return Err(conflict());
        }
        Ok(())
    }

    pub(super) fn post_r0_registrations(
        &self,
        root: &VersionedStateRoot,
    ) -> Result<
        Vec<(OrdinaryForkRecordV1, RetainedWorkerAdmissionRegistrationV1)>,
        RetainedWorkerRuntimeError,
    > {
        let root = preserved_start_root_view(root)?;
        self.body
            .records_by_session
            .values()
            .flat_map(BTreeMap::values)
            .filter_map(|record| match fork_registration(record, &root) {
                Ok(Some(registration)) => Some(Ok((record.clone(), registration))),
                Ok(None) => None,
                Err(error) => Some(Err(error)),
            })
            .collect()
    }

    pub(super) fn validate_complete_graphs(
        &self,
        root: &VersionedStateRoot,
        graphs: &BTreeMap<(String, String), ResolvedPostR0RegistryGraphV1>,
    ) -> Result<usize, RetainedWorkerRuntimeError> {
        let preserved = preserved_start_root_view(root)?;
        let mut count = 0usize;
        for (record, registration) in self.post_r0_registrations(root)? {
            let a = &record.activation;
            let graph = graphs
                .get(&(
                    a.orchestration_session_id.clone(),
                    a.retained_participant_id.clone(),
                ))
                .ok_or_else(conflict)?;
            let SessionNamespaceRecordV1::Authority(current) = preserved
                .session_namespace_map
                .get(&a.orchestration_session_id)
                .ok_or_else(conflict)?
            else {
                return Err(conflict());
            };
            if &graph.current_exact_authority.authority != current.as_ref()
                || graph.current_exact_authority.authority_store_id != preserved.authority_store_id
                || graph.registered_graph.result.registration_id != registration.registration_id
                || graph.registered_graph.result.retained_worker_ref
                    != registration.retained_worker_ref
                || preserved
                    .retained_worker_registration_journal
                    .get(&registration.registration_id)
                    != Some(&graph.registered_graph.resolved_target.registration)
            {
                return Err(conflict());
            }
            validate_fork_target(&record, &graph.registered_graph.resolved_target)?;
            count = count.checked_add(1).ok_or_else(conflict)?;
        }
        Ok(count)
    }

    fn new(key: &RetainedWorkerAdmissionKeyEnvelopeV1) -> Self {
        Self {
            body: OrdinaryForkRegistryBodyV1 {
                schema_version: 1,
                domain: DOMAIN.into(),
                authority_store_id: key.authority_store_id.clone(),
                key_id: key.key_id.clone(),
                revision: 0,
                records_by_session: BTreeMap::new(),
            },
            mac_hex: String::new(),
        }
    }

    fn expected_mac(
        &self,
        key: &RetainedWorkerAdmissionKeyEnvelopeV1,
    ) -> Result<String, RetainedWorkerRuntimeError> {
        let mut input = b"substrate.retained-worker.ordinary-fork.mac-input.v1\0".to_vec();
        append_len64(
            &mut input,
            &encode_canonical(&self.body, "encode ordinary Fork MAC input")?,
        )?;
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
            || self.body.records_by_session.is_empty()
            || !is_lower_hex(&self.mac_hex, 64)
        {
            return Err(conflict());
        }
        let expected = self.expected_mac(key)?;
        if expected
            .bytes()
            .zip(self.mac_hex.bytes())
            .fold(0u8, |v, (a, b)| v | (a ^ b))
            != 0
        {
            return Err(conflict());
        }
        Ok(())
    }

    fn source_completion<'a>(
        &'a self,
        registry: &'a RetainedWorkerAdmissionRegistryV1,
        session: &str,
        participant: &str,
    ) -> Result<&'a transport_api_types::RetainedBootstrapObservationV1, RetainedWorkerRuntimeError>
    {
        if let Some(report) = registry
            .native_lifecycle_v1
            .as_ref()
            .and_then(|lifecycle| lifecycle.native_completion_for(session, participant))
        {
            return Ok(report.observation());
        }
        self.body
            .records_by_session
            .get(session)
            .and_then(|records| records.get(participant))
            .and_then(|record| record.completion.as_ref())
            .map(|completed| completed.report.observation())
            .ok_or_else(conflict)
    }

    #[cfg(target_os = "linux")]
    pub(super) fn world_controller_bootstraps(
        &self,
        session: &str,
        world: &WorldBindingV1,
    ) -> Result<(Vec<E2MemberLaunchActivationCarrierV1>, Vec<String>), RetainedWorkerRuntimeError>
    {
        let mut completed = Vec::new();
        let mut unresolved = Vec::new();
        for record in self
            .body
            .records_by_session
            .get(session)
            .into_iter()
            .flat_map(BTreeMap::values)
        {
            let a = &record.activation;
            if a.target_world.world_id != world.world_id
                || a.target_world.world_generation != world.world_generation
            {
                return Err(conflict());
            }
            if record.completion.is_some() && record.registration.is_some() {
                completed.push(a.clone());
            } else {
                unresolved.push(a.retained_participant_id.clone());
            }
        }
        Ok((completed, unresolved))
    }

    pub(super) fn live_count(&self) -> usize {
        // Neither a native bootstrap Exit nor observer loss retires a worker.
        self.body
            .records_by_session
            .values()
            .map(BTreeMap::len)
            .sum()
    }

    pub(super) fn validate_registry_joins(
        &self,
        registry: &RetainedWorkerAdmissionRegistryV1,
        authority_root: &VersionedStateRoot,
    ) -> Result<BTreeSet<String>, RetainedWorkerRuntimeError> {
        let root = preserved_start_root_view(authority_root)?;
        let mut requests = BTreeSet::new();
        let mut claims = BTreeSet::new();
        let mut issuers = BTreeSet::new();
        let mut participants = registry
            .records_by_session
            .values()
            .flat_map(BTreeMap::values)
            .map(|r| r.retained_participant_id.clone())
            .collect::<BTreeSet<_>>();
        let mut runs = registry
            .records_by_session
            .values()
            .flat_map(BTreeMap::values)
            .map(|r| r.bootstrap_run_id.clone())
            .collect::<BTreeSet<_>>();
        let mut native_sessions = registry
            .native_lifecycle_v1
            .as_ref()
            .map(|lifecycle| lifecycle.native_session_locators())
            .unwrap_or_default();
        let mut revision = 0u64;
        for (session, records) in &self.body.records_by_session {
            if session.is_empty() || records.is_empty() {
                return Err(conflict());
            }
            for (participant, record) in records {
                record.validate()?;
                let a = &record.activation;
                if &a.orchestration_session_id != session
                    || &a.retained_participant_id != participant
                    || a.commitment_ref.authority_store_id != registry.authority_store_id
                    || !participants.insert(participant.clone())
                    || !runs.insert(a.bootstrap_run_id.clone())
                    || !claims.insert(record.native_claim_id.clone())
                    || !issuers.insert((session.clone(), a.request_id.clone()))
                    || registry.issuer_request_index.contains_key(&a.request_id)
                {
                    return Err(conflict());
                }
                revision = revision
                    .checked_add(record.record_revision)
                    .ok_or_else(conflict)?;
                let source = self.source_completion(
                    registry,
                    session,
                    a.source_participant_id.as_deref().ok_or_else(conflict)?,
                )?;
                if source.source_start != record.start
                    || source.activation.immutable_worker_cap_ref != record.source_cap_ref
                {
                    return Err(conflict());
                }
                if let Some(completed) = &record.completion {
                    let provenance = completed.report.provenance();
                    if provenance.source_runtime != source.runtime
                        || provenance.source_activation != source.activation
                        || provenance.source_native_session_id != source.native_session_id
                        || !native_sessions.insert((
                            session.clone(),
                            completed
                                .report
                                .completion()
                                .observation()
                                .native_session_id
                                .clone(),
                        ))
                    {
                        return Err(conflict());
                    }
                }
                let request_id = record.authority_registration_issuer();
                if let Some(request) = root
                    .retained_worker_registration_request_index
                    .get(&request_id)
                {
                    let head = record.registration_head.as_ref().ok_or_else(conflict)?;
                    if request.issuer_request_id != request_id
                        || request.orchestration_session_id != *session
                        || request.retained_participant_id != *participant
                        || request.authority_revision_before != head.authority_revision
                        || request.authority_record_commitment_before
                            != head.authority_record_commitment
                        || super::super::dispatch_policy_commitment::transport_policy_ref(
                            &request.current_policy_ref,
                        ) != a.parent_policy_ref
                        || request.world_binding.world_id != a.target_world.world_id
                        || request.world_binding.world_generation != a.target_world.world_generation
                    {
                        return Err(conflict());
                    }
                    if let Some(registration) = &record.registration {
                        if registration.registration_id != request.registration_id {
                            return Err(conflict());
                        }
                    }
                    match &request.state {
                        RetainedWorkerAuthorityRegistrationRequestStateV1::Reserved => {
                            if record.registration.is_some() {
                                return Err(conflict());
                            }
                        }
                        RetainedWorkerAuthorityRegistrationRequestStateV1::Applied {
                            authority_revision_after,
                            authority_record_commitment_after,
                        } => {
                            let journal = root
                                .retained_worker_registration_journal
                                .get(&request.registration_id)
                                .ok_or_else(conflict)?;
                            if journal.issuer_request_id != request_id
                                || journal.registration_id != request.registration_id
                                || journal.orchestration_session_id != *session
                                || journal.retained_participant_id != *participant
                                || journal.authority_revision_before != head.authority_revision
                                || journal.authority_record_commitment_before
                                    != head.authority_record_commitment
                                || journal.authority_revision_after != *authority_revision_after
                                || journal.authority_record_commitment_after
                                    != *authority_record_commitment_after
                                || journal.current_policy_ref != request.current_policy_ref
                                || journal.world_binding != request.world_binding
                                || journal.descriptor_ref.ref_id != request.descriptor_ref_id
                                || journal.descriptor_ref.commitment
                                    != request.descriptor_commitment
                                || journal.resume_handle_ref.ref_id != request.resume_handle_ref_id
                                || journal.resume_handle_ref.commitment
                                    != request.resume_handle_commitment
                                || journal.retained_worker_ref.ref_id
                                    != request.retained_worker_ref_id
                                || journal.retained_worker_ref.commitment
                                    != request.retained_worker_commitment
                                || record.registration.as_ref().is_some_and(|r| {
                                    r.retained_worker_ref != journal.retained_worker_ref
                                })
                            {
                                return Err(conflict());
                            }
                        }
                    }
                    if !requests.insert(request_id) {
                        return Err(conflict());
                    }
                } else if record.registration.is_some() {
                    return Err(conflict());
                }
            }
        }
        if self.body.authority_store_id != registry.authority_store_id
            || self.body.key_id != registry.commitment_key.key_id
            || revision == 0
            || self.body.revision != revision
        {
            return Err(conflict());
        }
        Ok(requests)
    }
}

#[derive(Debug)]
pub(crate) struct OrdinaryForkNativeClaimV1 {
    pub(crate) record: OrdinaryForkRecordV1,
    pub(crate) new_native_claim: bool,
    native_send: Option<OrdinaryForkNativeSendPermitV1>,
}

/// Affine local send ownership backed by the one durable claim. Neither this
/// value nor an exact retry can be cloned into another native-send permission.
#[derive(Debug)]
pub(crate) struct OrdinaryForkNativeSendPermitV1 {
    record: OrdinaryForkRecordV1,
}

impl OrdinaryForkNativeClaimV1 {
    pub(crate) fn into_parts(
        self,
    ) -> (OrdinaryForkRecordV1, Option<OrdinaryForkNativeSendPermitV1>) {
        (self.record, self.native_send)
    }
}

impl OrdinaryForkNativeSendPermitV1 {
    #[cfg(target_os = "linux")]
    pub(crate) fn validate_for_native_launch(
        self,
        selection: &super::super::native_launch_context::AuthenticatedOrdinarySelectionV1,
        activation: &E2MemberLaunchActivationCarrierV1,
        descriptor: &super::super::validator::RuntimeSelectionDescriptor,
    ) -> Result<(), RetainedWorkerRuntimeError> {
        selection.revalidate().map_err(|_| conflict())?;
        let expected_kind = match descriptor.backend_kind {
            super::super::mapping::AgentRuntimeBackendKind::Codex => {
                super::super::host_session_authority::schema::RuntimeBackendKindV1::Codex
            }
            super::super::mapping::AgentRuntimeBackendKind::ClaudeCode => {
                super::super::host_session_authority::schema::RuntimeBackendKindV1::ClaudeCode
            }
        };
        if self.record.activation != *activation
            || self.record.registration.is_some()
            || self.record.completion.is_some()
            || descriptor.execution_scope
                != crate::execution::config_model::AgentExecutionScope::World
            || self.record.descriptor.agent_id != descriptor.agent_id
            || self.record.descriptor.backend_id != descriptor.backend_id
            || self.record.descriptor.protocol != descriptor.protocol
            || self.record.descriptor.backend_kind != expected_kind
            || self.record.descriptor.binary_path
                != descriptor.binary_path.to_str().ok_or_else(conflict)?
            || self.record.start
                != crate::execution::orchestrator_world_dispatch::ordinary_start_selection_carrier(
                    selection,
                )
                .map_err(|_| conflict())?
        {
            return Err(conflict());
        }
        let current = RetainedWorkerRuntime
            .read_existing_ordinary_fork_record(
                selection.held_authority(),
                &activation.orchestration_session_id,
                &activation.retained_participant_id,
            )?
            .ok_or_else(conflict)?;
        if current != self.record {
            return Err(conflict());
        }
        Ok(())
    }
}

#[cfg(target_os = "linux")]
impl RetainedWorkerRuntime {
    fn mutate_ordinary_forks<T>(
        &self,
        authority: &HostSessionAuthority,
        operation: impl FnOnce(
            &mut OrdinaryForkRegistryV1,
            &RetainedWorkerAdmissionRegistryV1,
            &VersionedStateRoot,
            &super::super::dispatch_policy_commitment::OrdinaryBootstrapPopulationV1,
        ) -> Result<T, RetainedWorkerRuntimeError>,
    ) -> Result<T, RetainedWorkerRuntimeError> {
        let graphs = self.resolve_all_post_r0_registry_graphs(authority)?;
        let storage = super::super::host_session_authority::store::retained_worker_admission_storage_for_authority(authority)
            .map_err(|error| RetainedWorkerRuntimeError(error.to_string()))?;
        storage.transaction(|transaction| {
            transaction.with_existing_e2_population_source(|physical, verify_population| {
                Ok((|| {
                    let population = super::super::dispatch_policy_commitment::decode_ordinary_bootstrap_population(
                        physical, transaction.authority_root().authority_store_id()).map_err(|error| RetainedWorkerRuntimeError(error.to_string()))?;
                    let bytes = transaction.read_registry().map_err(|error| RetainedWorkerRuntimeError(error.to_string()))?;
                    let keys = transaction.read_keys().map_err(|error| RetainedWorkerRuntimeError(error.to_string()))?;
                    let original = bytes.as_deref().ok_or_else(conflict)?;
                    let mut registry: RetainedWorkerAdmissionRegistryV1 = decode_canonical(original, "decode Fork admission source")?;
                    let (_, key) = load_committed_admission_key(transaction.authority_root().authority_store_id(), &registry, &keys)?;
                    validate_admission_registry(&registry, transaction.authority_root())?;
                    validate_complete_post_r0_registry_graphs(&registry, transaction.authority_root(), &graphs)?;
                    validate_admission_e2_population(&registry, &population)?;
                    let mut forks = registry.ordinary_forks_v1.clone().unwrap_or_else(|| OrdinaryForkRegistryV1::new(&key));
                    let result = operation(&mut forks, &registry, transaction.authority_root(), &population)?;
                    if registry.ordinary_forks_v1.is_none() && forks.body.records_by_session.is_empty() { return Ok(result); }
                    forks.body.revision = forks.body.records_by_session.values().flat_map(BTreeMap::values)
                        .try_fold(0u64, |sum, record| sum.checked_add(record.record_revision).ok_or_else(conflict))?;
                    forks.mac_hex = forks.expected_mac(&key)?;
                    registry.ordinary_forks_v1 = Some(forks);
                    validate_admission_registry(&registry, transaction.authority_root())?;
                    validate_admission_e2_population(&registry, &population)?;
                    load_committed_admission_key(transaction.authority_root().authority_store_id(), &registry, &keys)?;
                    let published = encode_canonical(&registry, "encode ordinary Fork registry")?;
                    if published != original {
                        #[cfg(test)]
                        run_ordinary_population_before_publication_hook();
                        let mut nonce = [0u8; 16]; rand::rngs::OsRng.fill_bytes(&mut nonce);
                        transaction.replace_registry_with_source_guard(&format!("admission-registry--{}.tmp", lower_hex(&nonce)), &published, verify_population)
                            .map_err(|error| RetainedWorkerRuntimeError(error.to_string()))?;
                    }
                    Ok(result)
                })())
            })
        }).map_err(|error| RetainedWorkerRuntimeError(error.to_string()))?
    }

    /// Acquire the one durable native-send claim for an authenticated E2 Fork.
    /// An exact join returns data only, never another native-send permit.
    pub(crate) fn claim_ordinary_fork_native(
        &self,
        selection: &super::super::native_launch_context::AuthenticatedOrdinarySelectionV1,
        activation: &E2MemberLaunchActivationCarrierV1,
        descriptor: &AgentDescriptorV1,
        issuance: Option<
            super::super::dispatch_policy_commitment::AuthenticatedOrdinaryForkIssuanceV1,
        >,
    ) -> Result<OrdinaryForkNativeClaimV1, RetainedWorkerRuntimeError> {
        use super::super::dispatch_policy_commitment::{
            authenticate_dispatch_policy_commitment, DispatchPolicyCommitmentRefV1,
        };
        selection
            .require_ordinary_configured_authority()
            .map_err(|_| conflict())?;
        selection.revalidate().map_err(|_| conflict())?;
        activation.validate().map_err(|_| conflict())?;
        let authority = selection.held_authority();
        let current = authority
            .resolve_current_exact(selection.original_session_id(), None)
            .map_err(|_| conflict())?;
        let cap = authenticate_dispatch_policy_commitment(
            authority,
            &DispatchPolicyCommitmentRefV1 {
                authority_store_id: activation.commitment_ref.authority_store_id.clone(),
                commitment_id: activation.commitment_ref.commitment_id.clone(),
                exact_linkage_hash: activation.commitment_ref.exact_linkage_hash.clone(),
            },
        )
        .map_err(|_| conflict())?;
        if cap
            .member_launch_activation_carrier()
            .map_err(|_| conflict())?
            != *activation
            || activation.launch_kind != E2MemberLaunchKindV1::Fork
            || activation.orchestration_session_id != selection.original_session_id()
            || activation.caller_participant_id != selection.current_participant_id()
            || activation.target_backend_id != selection.requested_selection().backend_id
        {
            return Err(conflict());
        }
        let source = activation
            .source_participant_id
            .as_deref()
            .ok_or_else(conflict)?;

        let source_cap = cap.fork_source_cap_ref().ok_or_else(conflict)?;
        let source_cap_ref = transport_api_types::DispatchPolicyCommitmentRefCarrierV1 {
            authority_store_id: source_cap.authority_store_id.clone(),
            commitment_id: source_cap.commitment_id.clone(),
            exact_linkage_hash: source_cap.exact_linkage_hash.clone(),
        };
        let policy = crate::execution::policy_model::resolve_effective_policy_for_bootstrap_home(
            std::path::Path::new(selection.workspace_root()),
            &authority.bootstrap_home(),
        )
        .map_err(|_| conflict())?;
        let steering = policy.world_dispatch_policy();
        if !steering.enabled
            || !steering
                .allowed_actions
                .iter()
                .any(|a| a == "fork_world_worker")
            || !steering.allowed_modes.iter().any(|m| m == "retained")
            || !steering
                .allowed_backends
                .iter()
                .any(|b| b == &activation.target_backend_id)
        {
            return Err(conflict());
        }
        selection
            .revalidate_requested_account(&policy)
            .map_err(|_| conflict())?;
        let start =
            crate::execution::orchestrator_world_dispatch::ordinary_start_selection_carrier(
                selection,
            )
            .map_err(|_| conflict())?;
        let max_live_retained_workers = u64::from(steering.max_live_retained_workers);
        let claimed_at = TimestampV1::parse(
            chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Nanos, true),
        )
        .map_err(|_| conflict())?;
        let mut entropy = [0u8; 16];
        rand::rngs::OsRng.fill_bytes(&mut entropy);
        let record = OrdinaryForkRecordV1 {
            start,
            activation: activation.clone(),
            descriptor: descriptor.clone(),
            source_cap_ref,
            native_claim_id: format!("ofc_{}", lower_hex(&entropy)),
            bootstrap_receiver_protocol: Some(1),
            claimed_at,
            max_live_retained_workers,
            record_revision: 1,
            transport_start: None,
            completion: None,
            registration_head: None,
            registration: None,
        };
        record.validate()?;
        // Native-writer recovery can reconcile its own partial publication. A
        // checked child no longer needs its historical source to remain live.
        let freshly_issued = issuance.is_some();
        if let Some(issuance) = issuance {
            issuance
                .validate_for_activation(activation)
                .map_err(|_| conflict())?;
        }
        let had_existing_claim = self.mutate_ordinary_forks(authority, |forks, _, _, _| {
            Ok(forks
                .body
                .records_by_session
                .get(&activation.orchestration_session_id)
                .is_some_and(|records| records.contains_key(&activation.retained_participant_id)))
        })?;
        if !had_existing_claim {
            if !freshly_issued {
                return Err(RetainedWorkerRuntimeError("ordinary Fork E2 issuance has missing native ownership; no native input may be reissued".into()));
            }
            super::super::state_store::resolve_canonical_retained_world_dispatch_target(
                authority,
                &current,
                source,
                &activation.target_backend_id,
            )
            .map_err(|_| conflict())?;
        }
        selection.revalidate().map_err(|_| conflict())?;
        self.mutate_ordinary_forks(authority, |forks, registry, root, population| {
            let stored_root = preserved_start_root_view(root)?;
            let SessionNamespaceRecordV1::Authority(now) = stored_root.session_namespace_map
                .get(&activation.orchestration_session_id).ok_or_else(conflict)? else { return Err(conflict()); };
            if now.as_ref() != &current.authority { return Err(conflict()); }
            if let Some(existing) = forks.body.records_by_session.values().flat_map(BTreeMap::values)
                .find(|r| r.activation.orchestration_session_id == activation.orchestration_session_id
                    && (r.activation.request_id == activation.request_id || r.activation.idempotency_key == activation.idempotency_key)) {
                if existing.start != record.start || existing.activation != *activation
                    || existing.descriptor != *descriptor || existing.source_cap_ref != record.source_cap_ref
                { return Err(conflict()); }
                return Ok(OrdinaryForkNativeClaimV1 { record: existing.clone(), new_native_claim: false, native_send: None });
            }
            if had_existing_claim { return Err(conflict()); }
            let live_spawn = registry.records_by_session.values().flat_map(BTreeMap::values)
                .filter(|r| admission_state_is_live(&r.state)).count();
            // The current newly published E2 child is already in this set.
            let (closed_spawn, closed_fork) = registry.ordinary_stops_v1.as_ref()
                .map(|stops| stops.acknowledged_counts(registry)).transpose()?.unwrap_or_default();
            let (retired_spawn, retired_fork) = registry.ordinary_bootstrap_receivers_v1.as_ref()
                .map_or((0, 0), |receivers| receivers.retirement_counts());
            let live = (live_spawn as u64).checked_add(population.fork_count())
                .and_then(|count| count.checked_sub(closed_spawn)).and_then(|count| count.checked_sub(closed_fork))
                .and_then(|count| count.checked_sub(retired_spawn)).and_then(|count| count.checked_sub(retired_fork)).ok_or_else(conflict)?;
            if live > max_live_retained_workers {
                return Err(RetainedWorkerRuntimeError(format!("retained_worker_cap_reached: live retained worker count {live} reaches configured maximum {max_live_retained_workers}")));
            }
            let source_report = forks.source_completion(registry, &activation.orchestration_session_id, source)?;
            if source_report.source_start != record.start
                || source_report.activation.immutable_worker_cap_ref != record.source_cap_ref
            { return Err(conflict()); }
            let session = forks.body.records_by_session.entry(activation.orchestration_session_id.clone()).or_default();
            if session.insert(activation.retained_participant_id.clone(), record.clone()).is_some() { return Err(conflict()); }
            Ok(OrdinaryForkNativeClaimV1 { record: record.clone(), new_native_claim: true, native_send: Some(OrdinaryForkNativeSendPermitV1 { record }) })
        })
    }
}

#[cfg(target_os = "linux")]
impl RetainedWorkerRuntime {
    /// Existing-source read only; partial writer publications are not recovered here.
    pub(crate) fn read_existing_ordinary_fork_record(
        &self,
        authority: &HostSessionAuthority,
        session: &str,
        participant: &str,
    ) -> Result<Option<OrdinaryForkRecordV1>, RetainedWorkerRuntimeError> {
        super::super::host_session_authority::store::with_existing_retained_worker_admission_read(
            authority,
            |root, bytes, keys| {
                let Some(bytes) = bytes else {
                    return Ok(None);
                };
                let registry: RetainedWorkerAdmissionRegistryV1 =
                    decode_canonical(bytes, "decode existing Fork source")?;
                load_committed_admission_key(root.authority_store_id(), &registry, &keys)?;
                validate_admission_registry(&registry, root)?;
                Ok(registry.ordinary_forks_v1.and_then(|forks| {
                    forks
                        .body
                        .records_by_session
                        .get(session)
                        .and_then(|records| records.get(participant))
                        .cloned()
                }))
            },
        )
        .map_err(|error| {
            RetainedWorkerRuntimeError(format!("read existing Fork source: {error:?}"))
        })?
    }

    /// Preserve actual stream custody without granting another native send.
    pub(crate) fn observe_ordinary_fork_transport_start(
        &self,
        selection: &super::super::native_launch_context::AuthenticatedOrdinarySelectionV1,
        activation: &E2MemberLaunchActivationCarrierV1,
        frame: &RuntimeFrameIdentityV1,
        span_id: &str,
    ) -> Result<(), RetainedWorkerRuntimeError> {
        frame.validate().map_err(|_| conflict())?;
        if span_id.trim().is_empty() {
            return Err(conflict());
        }
        let start =
            crate::execution::orchestrator_world_dispatch::ordinary_start_selection_carrier(
                selection,
            )
            .map_err(|_| conflict())?;
        self.mutate_ordinary_forks(selection.held_authority(), |forks, _, _, _| {
            let record = forks
                .body
                .records_by_session
                .get_mut(&activation.orchestration_session_id)
                .and_then(|records| records.get_mut(&activation.retained_participant_id))
                .ok_or_else(conflict)?;
            if record.start != start || record.activation != *activation {
                return Err(conflict());
            }
            let observed = OrdinaryForkTransportStartV1 {
                frame: frame.clone(),
                span_id: span_id.into(),
            };
            match &record.transport_start {
                Some(existing) if existing == &observed => {}
                Some(_) => return Err(conflict()),
                None => {
                    record.transport_start = Some(observed);
                    record.record_revision =
                        record.record_revision.checked_add(1).ok_or_else(conflict)?;
                }
            }
            Ok(())
        })
    }

    /// Check readiness against the held E2 child and original transport Start.
    /// It does not make the retained child R0-routable.
    pub(crate) fn validate_ordinary_fork_registered(
        &self,
        selection: &super::super::native_launch_context::AuthenticatedOrdinarySelectionV1,
        activation: &E2MemberLaunchActivationCarrierV1,
        frame: &RuntimeFrameIdentityV1,
        registered: &AgentEvent,
    ) -> Result<(), RetainedWorkerRuntimeError> {
        let start =
            crate::execution::orchestrator_world_dispatch::ordinary_start_selection_carrier(
                selection,
            )
            .map_err(|_| conflict())?;
        self.mutate_ordinary_forks(selection.held_authority(), |forks, _, _, _| {
            let record = forks
                .body
                .records_by_session
                .get(&activation.orchestration_session_id)
                .and_then(|records| records.get(&activation.retained_participant_id))
                .ok_or_else(conflict)?;
            if record.start != start || record.activation != *activation {
                return Err(conflict());
            }
            validate_fork_registered(record, frame, registered)
        })
    }

    /// Record checked original native completion before attempting genuine R0.
    /// Exact recovery may complete registration, but never resends native input.
    pub(crate) fn record_native_fork_bootstrap_completion(
        &self,
        selection: &super::super::native_launch_context::AuthenticatedOrdinarySelectionV1,
        registered_frame: &RuntimeFrameIdentityV1,
        registered: &AgentEvent,
        report: &RetainedForkBootstrapCompletionV1,
    ) -> Result<OrdinaryForkRecordV1, RetainedWorkerRuntimeError> {
        use super::super::dispatch_policy_commitment::{
            authenticate_dispatch_policy_commitment, DispatchPolicyCommitmentRefV1,
        };
        report.validate().map_err(|_| conflict())?;
        registered_frame.validate().map_err(|_| conflict())?;
        registered
            .validate_identity_contract()
            .map_err(|_| conflict())?;
        let observed = report.completion().observation();
        let activation = &observed.activation;
        let authority = selection.held_authority();
        let start =
            crate::execution::orchestrator_world_dispatch::ordinary_start_selection_carrier(
                selection,
            )
            .map_err(|_| conflict())?;
        let cap = authenticate_dispatch_policy_commitment(
            authority,
            &DispatchPolicyCommitmentRefV1 {
                authority_store_id: activation.commitment_ref.authority_store_id.clone(),
                commitment_id: activation.commitment_ref.commitment_id.clone(),
                exact_linkage_hash: activation.commitment_ref.exact_linkage_hash.clone(),
            },
        )
        .map_err(|_| conflict())?;
        let source_cap = cap.fork_source_cap_ref().ok_or_else(conflict)?;
        if observed.source_start != start
            || cap
                .member_launch_activation_carrier()
                .map_err(|_| conflict())?
                != *activation
            || source_cap.authority_store_id
                != report
                    .provenance()
                    .source_activation
                    .immutable_worker_cap_ref
                    .authority_store_id
            || source_cap.commitment_id
                != report
                    .provenance()
                    .source_activation
                    .immutable_worker_cap_ref
                    .commitment_id
            || source_cap.exact_linkage_hash
                != report
                    .provenance()
                    .source_activation
                    .immutable_worker_cap_ref
                    .exact_linkage_hash
            || registered.kind != AgentEventKind::Registered
            || registered.orchestration_session_id != activation.orchestration_session_id
            || registered.participant_id.as_deref()
                != Some(activation.retained_participant_id.as_str())
            || registered.run_id != activation.bootstrap_run_id
            || registered.backend_id.as_deref() != Some(activation.target_backend_id.as_str())
            || registered.world_id.as_deref() != Some(activation.target_world.world_id.as_str())
            || registered.world_generation != Some(activation.target_world.world_generation)
            || registered.role.as_deref() != Some("member")
            || registered.parent_participant_id != activation.source_participant_id
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
                        .filter(|id| !id.is_empty())
                })
                != Some(observed.native_session_id.as_str())
        {
            return Err(conflict());
        }
        let mut hash = Sha256::new();
        hash.update(b"substrate.retained-worker.fork-registered-event.v1\0");
        hash.update(serde_json::to_vec(registered).map_err(|_| conflict())?);
        let completed = OrdinaryForkCompletedV1 {
            registered_frame: registered_frame.clone(),
            registered_event: registered.event_identity.clone().ok_or_else(conflict)?,
            registered_event_sha256: format!("{:x}", hash.finalize()),
            report: report.clone(),
        };
        self.mutate_ordinary_forks(authority, |forks, _, _, _| {
            let record = forks
                .body
                .records_by_session
                .get_mut(&activation.orchestration_session_id)
                .and_then(|records| records.get_mut(&activation.retained_participant_id))
                .ok_or_else(conflict)?;
            if record.start != start || record.activation != *activation {
                return Err(conflict());
            }
            validate_fork_registered(record, registered_frame, registered)?;
            if let Some(existing) = &record.completion {
                if existing != &completed {
                    return Err(conflict());
                }
            } else {
                record.completion = Some(completed);
                record.record_revision =
                    record.record_revision.checked_add(1).ok_or_else(conflict)?;
            }
            Ok(())
        })?;
        self.adopt_ordinary_fork_r0(
            authority,
            &activation.orchestration_session_id,
            &activation.retained_participant_id,
        )
    }

    pub(crate) fn recover_ordinary_fork_registration(
        &self,
        selection: &super::super::native_launch_context::AuthenticatedOrdinarySelectionV1,
        record: &OrdinaryForkRecordV1,
    ) -> Result<OrdinaryForkRecordV1, RetainedWorkerRuntimeError> {
        let joined = self.claim_ordinary_fork_native(
            selection,
            &record.activation,
            &record.descriptor,
            None,
        )?;
        if joined.new_native_claim || joined.record.native_claim_id != record.native_claim_id {
            return Err(conflict());
        }
        if joined.record.completion.is_none() {
            return Err(RetainedWorkerRuntimeError("ordinary Fork bootstrap remains pending or ambiguous; native input will not be redelivered".into()));
        }
        self.adopt_ordinary_fork_r0(
            selection.held_authority(),
            &record.activation.orchestration_session_id,
            &record.activation.retained_participant_id,
        )
    }

    fn adopt_ordinary_fork_r0(
        &self,
        authority: &HostSessionAuthority,
        session: &str,
        participant: &str,
    ) -> Result<OrdinaryForkRecordV1, RetainedWorkerRuntimeError> {
        use super::super::dispatch_policy_commitment::transport_policy_ref;
        let current = authority
            .resolve_current_exact(session, None)
            .map_err(|_| conflict())?;
        let record = self.mutate_ordinary_forks(authority, |forks, _, root, _| {
            let stored = preserved_start_root_view(root)?;
            let record = forks
                .body
                .records_by_session
                .get_mut(session)
                .and_then(|records| records.get_mut(participant))
                .ok_or_else(conflict)?;
            record.completion.as_ref().ok_or_else(conflict)?;
            if record.registration.is_some() {
                return Ok(record.clone());
            }
            if stored
                .retained_worker_registration_request_index
                .contains_key(&record.authority_registration_issuer())
            {
                record.registration_head.as_ref().ok_or_else(conflict)?;
                return Ok(record.clone());
            }
            let SessionNamespaceRecordV1::Authority(now) = stored
                .session_namespace_map
                .get(session)
                .ok_or_else(conflict)?
            else {
                return Err(conflict());
            };
            if now.as_ref() != &current.authority
                || current
                    .authority
                    .current_policy_ref
                    .as_ref()
                    .map(transport_policy_ref)
                    .as_ref()
                    != Some(&record.activation.parent_policy_ref)
                || current.current_policy.policy_revision
                    != record.activation.parent_policy_revision
                || current
                    .authority
                    .world_binding
                    .as_ref()
                    .map(|w| (&w.world_id, w.world_generation))
                    != Some((
                        &record.activation.target_world.world_id,
                        record.activation.target_world.world_generation,
                    ))
            {
                return Err(conflict());
            }
            let head = OrdinaryForkRegistrationHeadV1 {
                revision: record
                    .registration_head
                    .as_ref()
                    .map_or(Ok(1), |h| h.revision.checked_add(1).ok_or_else(conflict))?,
                authority_revision: current.observation.authority_revision,
                authority_record_commitment: current
                    .observation
                    .authority_record_commitment
                    .clone(),
            };
            if record.registration_head.as_ref().is_none_or(|old| {
                old.authority_revision != head.authority_revision
                    || old.authority_record_commitment != head.authority_record_commitment
            }) {
                record.registration_head = Some(head);
                record.record_revision =
                    record.record_revision.checked_add(1).ok_or_else(conflict)?;
            }
            Ok(record.clone())
        })?;
        if record.registration.is_some() {
            self.resolve_ordinary_fork_registered_target(authority, &record)?;
            return Ok(record);
        }
        let head = record.registration_head.as_ref().ok_or_else(conflict)?;
        let observed = record
            .completion
            .as_ref()
            .ok_or_else(conflict)?
            .report
            .completion()
            .observation();
        let plan = RetainedWorkerRegistrationPlanV1 {
            registration_request_id: record.registration_request_id(),
            orchestration_session_id: session.into(),
            expected_authority: RetainedWorkerAuthorityPreconditionV1 {
                authority_store_id: record.start.authority_store_id.clone(),
                authority_revision: head.authority_revision,
                authority_record_commitment: head.authority_record_commitment.clone(),
            },
            retained_participant_id: participant.into(),
            descriptor: record.descriptor.clone(),
            internal_uaa_session_id: observed.native_session_id.clone(),
        };
        #[cfg(not(test))]
        let result = self.register_retained_target(authority, &plan)?;
        #[cfg(test)]
        let result = self.register_retained_target_with(authority, &plan, |_| {
            fork_r0_crash_for_test(OrdinaryForkR0CrashPointV1::AfterReservation)
        })?;
        #[cfg(test)]
        fork_r0_crash_for_test(OrdinaryForkR0CrashPointV1::AfterApplication)?;
        let resolved = self.resolve_retained_target(authority, &result)?;
        validate_fork_target(&record, &resolved)?;
        self.mutate_ordinary_forks(authority, |forks, _, _, _| {
            let current = forks
                .body
                .records_by_session
                .get_mut(session)
                .and_then(|records| records.get_mut(participant))
                .ok_or_else(conflict)?;
            if current != &record {
                return Err(conflict());
            }
            current.registration = Some(RetainedWorkerAdmissionRegistrationV1 {
                registration_id: result.registration_id,
                retained_worker_ref: result.retained_worker_ref,
            });
            current.record_revision = current
                .record_revision
                .checked_add(1)
                .ok_or_else(conflict)?;
            Ok(current.clone())
        })
    }

    pub(crate) fn resolve_ordinary_fork_registered_target(
        &self,
        authority: &HostSessionAuthority,
        record: &OrdinaryForkRecordV1,
    ) -> Result<ResolvedRetainedTargetV1, RetainedWorkerRuntimeError> {
        let root = authority
            .read_preserved_start_root_v2()
            .map_err(|_| conflict())?;
        let registration = fork_registration(record, &root)?.ok_or_else(conflict)?;
        let result = self.resolve_admission_registration_graph(authority, &registration)?;
        validate_fork_target(record, &result.resolved_target)?;
        Ok(result.resolved_target)
    }
}

fn fork_registration(
    record: &OrdinaryForkRecordV1,
    root: &StateRootV2,
) -> Result<Option<RetainedWorkerAdmissionRegistrationV1>, RetainedWorkerRuntimeError> {
    let Some(request) = root
        .retained_worker_registration_request_index
        .get(&record.authority_registration_issuer())
    else {
        return Ok(None);
    };
    if !matches!(
        request.state,
        RetainedWorkerAuthorityRegistrationRequestStateV1::Applied { .. }
    ) {
        return Ok(None);
    }
    let journal = root
        .retained_worker_registration_journal
        .get(&request.registration_id)
        .ok_or_else(conflict)?;
    Ok(Some(RetainedWorkerAdmissionRegistrationV1 {
        registration_id: journal.registration_id.clone(),
        retained_worker_ref: journal.retained_worker_ref.clone(),
    }))
}

pub(super) fn validate_fork_target(
    record: &OrdinaryForkRecordV1,
    target: &ResolvedRetainedTargetV1,
) -> Result<(), RetainedWorkerRuntimeError> {
    use super::super::dispatch_policy_commitment::transport_policy_ref;
    let observed = record
        .completion
        .as_ref()
        .ok_or_else(conflict)?
        .report
        .completion()
        .observation();
    if target.descriptor != record.descriptor
        || target.resume_handle.internal_uaa_session_id != observed.native_session_id
        || target.resume_handle.orchestration_session_id
            != record.activation.orchestration_session_id
        || target.resume_handle.participant_id != record.activation.retained_participant_id
        || target.retained_worker.participant_id != record.activation.retained_participant_id
        || transport_policy_ref(&target.retained_worker.policy_ref)
            != record.activation.parent_policy_ref
        || target.current_policy.policy_revision != record.activation.parent_policy_revision
        || target.retained_worker.world_binding.world_id != record.activation.target_world.world_id
        || target.retained_worker.world_binding.world_generation
            != record.activation.target_world.world_generation
        || target.registration.issuer_request_id != record.authority_registration_issuer()
        || target.registration.authority_revision_before
            != record
                .registration_head
                .as_ref()
                .ok_or_else(conflict)?
                .authority_revision
        || target.registration.authority_record_commitment_before
            != record
                .registration_head
                .as_ref()
                .ok_or_else(conflict)?
                .authority_record_commitment
    {
        return Err(conflict());
    }
    Ok(())
}

fn validate_fork_registered(
    record: &OrdinaryForkRecordV1,
    frame: &RuntimeFrameIdentityV1,
    registered: &AgentEvent,
) -> Result<(), RetainedWorkerRuntimeError> {
    frame.validate().map_err(|_| conflict())?;
    registered
        .validate_identity_contract()
        .map_err(|_| conflict())?;
    let start = record.transport_start.as_ref().ok_or_else(conflict)?;
    let a = &record.activation;
    if frame.stream_id != start.frame.stream_id
        || frame.frame_sequence <= start.frame.frame_sequence
        || registered.kind != AgentEventKind::Registered
        || registered.orchestration_session_id != a.orchestration_session_id
        || registered.participant_id.as_deref() != Some(a.retained_participant_id.as_str())
        || registered.run_id != a.bootstrap_run_id
        || registered.backend_id.as_deref() != Some(a.target_backend_id.as_str())
        || registered.world_id.as_deref() != Some(a.target_world.world_id.as_str())
        || registered.world_generation != Some(a.target_world.world_generation)
        || registered.role.as_deref() != Some("member")
        || registered.parent_participant_id != a.source_participant_id
        || registered.resumed_from_participant_id.is_some()
        || registered.span_id.as_deref() != Some(start.span_id.as_str())
        || registered.event_identity.is_none()
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
                    .filter(|id| !id.is_empty())
            })
            .is_none()
    {
        return Err(conflict());
    }
    Ok(())
}
