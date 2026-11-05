//! Explicit logical Stop intent and acknowledged retirement, separate from
//! native bootstrap completion and accepted-turn terminal observations.
use super::*;
use transport_api_types::{
    OrdinaryRetainedWorkerStopAcknowledgmentV1, OrdinaryRetainedWorkerStopRequestV1,
    RetainedBootstrapCompletionV1, RetainedForkBootstrapCompletionV1,
};

const DOMAIN: &str = "substrate.retained-worker.ordinary-stop.v1";

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct OrdinaryStopRegistryV1 {
    body: OrdinaryStopRegistryBodyV1,
    mac_hex: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
struct OrdinaryStopRegistryBodyV1 {
    schema_version: u32,
    domain: String,
    authority_store_id: String,
    key_id: String,
    revision: u64,
    #[serde(with = "records_codec")]
    records_by_session: BTreeMap<String, BTreeMap<String, OrdinaryStopRecordV1>>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub(super) enum OrdinaryStopBootstrapV1 {
    Spawn {
        completion: Box<RetainedBootstrapCompletionV1>,
    },
    Fork {
        completion: Box<RetainedForkBootstrapCompletionV1>,
    },
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct OrdinaryStopRecordV1 {
    pub(super) request: OrdinaryRetainedWorkerStopRequestV1,
    pub(super) bootstrap: OrdinaryStopBootstrapV1,
    pub(super) idempotency_key: String,
    pub(super) accepted_work_ids_at_admission: Vec<String>,
    pub(super) admitted_at: TimestampV1,
    pub(super) acknowledgment: Option<OrdinaryRetainedWorkerStopAcknowledgmentV1>,
    pub(super) completed_at: Option<TimestampV1>,
    record_revision: u64,
}

mod records_codec {
    use super::*;
    type Records = BTreeMap<String, BTreeMap<String, OrdinaryStopRecordV1>>;
    pub fn serialize<S: Serializer>(records: &Records, serializer: S) -> Result<S::Ok, S::Error> {
        serde_json::to_value(records)
            .map_err(serde::ser::Error::custom)?
            .serialize(serializer)
    }
    pub fn deserialize<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Records, D::Error> {
        serde_json::from_value(serde_json::Value::deserialize(deserializer)?)
            .map_err(serde::de::Error::custom)
    }
}

fn conflict() -> RetainedWorkerRuntimeError {
    RetainedWorkerRuntimeError("ordinary Stop intent/retirement source conflicts".into())
}

impl OrdinaryStopBootstrapV1 {
    pub(super) fn completion(&self) -> &transport_api_types::RetainedBootstrapObservationV1 {
        match self {
            Self::Spawn { completion } => completion.observation(),
            Self::Fork { completion } => completion.observation(),
        }
    }

    fn validate_request(
        &self,
        request: &OrdinaryRetainedWorkerStopRequestV1,
    ) -> Result<(), RetainedWorkerRuntimeError> {
        request.validate().map_err(|_| conflict())?;
        match self {
            Self::Spawn { completion } => completion.validate(),
            Self::Fork { completion } => completion.validate(),
        }
        .map_err(|_| conflict())?;
        let observation = self.completion();
        let activation = &observation.activation;
        if request.authority_store_id != observation.source_start.authority_store_id
            || request.orchestration_session_id != observation.source_start.original_session_id
            || request.orchestration_session_id != activation.orchestration_session_id
            || request.target_participant_id != activation.retained_participant_id
            || request.target_backend_id != activation.target_backend_id
            || request.bootstrap_run_id != activation.bootstrap_run_id
            || request.world_id != activation.target_world.world_id
            || request.world_generation != activation.target_world.world_generation
            || request.worker_cap_ref != activation.immutable_worker_cap_ref
            || request.native_session_id != observation.native_session_id
            || matches!(
                (self, &activation.launch_kind),
                (
                    Self::Spawn { .. },
                    transport_api_types::E2MemberLaunchKindV1::Fork
                ) | (
                    Self::Fork { .. },
                    transport_api_types::E2MemberLaunchKindV1::FreshSpawn
                )
            )
        {
            return Err(conflict());
        }
        Ok(())
    }
}

impl OrdinaryStopRecordV1 {
    fn validate(&self) -> Result<(), RetainedWorkerRuntimeError> {
        self.bootstrap.validate_request(&self.request)?;
        if self.idempotency_key.is_empty()
            || self.idempotency_key.trim() != self.idempotency_key
            || self
                .accepted_work_ids_at_admission
                .windows(2)
                .any(|ids| ids[0] >= ids[1])
            || self.accepted_work_ids_at_admission.iter().any(|id| {
                id.strip_prefix("wwa_")
                    .and_then(|value| uuid::Uuid::parse_str(value).ok())
                    .is_none_or(|value| {
                        value.get_version_num() != 7 || format!("wwa_{value}") != *id
                    })
            })
        {
            return Err(conflict());
        }
        if self.record_revision != 1 + u64::from(self.acknowledgment.is_some())
            || self.acknowledgment.is_some() != self.completed_at.is_some()
        {
            return Err(conflict());
        }
        if let Some(ack) = &self.acknowledgment {
            ack.validate(&self.request).map_err(|_| conflict())?;
        }
        Ok(())
    }
}

impl OrdinaryStopRegistryV1 {
    pub(super) fn new(key: &RetainedWorkerAdmissionKeyEnvelopeV1) -> Self {
        Self {
            body: OrdinaryStopRegistryBodyV1 {
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
        let bytes = encode_canonical(&self.body, "encode ordinary Stop MAC input")?;
        let mut input = b"substrate.retained-worker.ordinary-stop.mac-input.v1\0".to_vec();
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
            || self.body.records_by_session.is_empty()
            || !is_lower_hex(&self.mac_hex, 64)
        {
            return Err(conflict());
        }
        let expected = self.expected_mac(key)?;
        if expected
            .bytes()
            .zip(self.mac_hex.bytes())
            .fold(0u8, |value, (a, b)| value | (a ^ b))
            != 0
        {
            return Err(conflict());
        }
        Ok(())
    }

    pub(super) fn validate_registry_joins(
        &self,
        registry: &RetainedWorkerAdmissionRegistryV1,
        root: &VersionedStateRoot,
    ) -> Result<(), RetainedWorkerRuntimeError> {
        let preserved = preserved_start_root_view(root)?;
        let mut revision = 0u64;
        let mut requests = BTreeSet::new();
        for (session, records) in &self.body.records_by_session {
            if session.is_empty() || records.is_empty() {
                return Err(conflict());
            }
            for (target, record) in records {
                record.validate()?;
                let request = &record.request;
                if &request.orchestration_session_id != session
                    || &request.target_participant_id != target
                    || request.authority_store_id != registry.authority_store_id
                    || !requests.insert((session.clone(), request.stop_request_id.clone()))
                {
                    return Err(conflict());
                }
                let registration = preserved
                    .retained_worker_registration_journal
                    .get(&request.registration_id)
                    .ok_or_else(conflict)?;
                if registration.orchestration_session_id != *session
                    || registration.retained_participant_id != *target
                    || registration.world_binding.world_id != request.world_id
                    || registration.world_binding.world_generation != request.world_generation
                {
                    return Err(conflict());
                }
                match &record.bootstrap {
                    OrdinaryStopBootstrapV1::Spawn { completion } => {
                        let actual = registry
                            .native_lifecycle_v1
                            .as_ref()
                            .and_then(|lifecycle| lifecycle.native_completion_for(session, target))
                            .ok_or_else(conflict)?;
                        if actual != completion.as_ref() {
                            return Err(conflict());
                        }
                        let admission = registry
                            .records_by_session
                            .get(session)
                            .and_then(|rows| rows.get(target))
                            .ok_or_else(conflict)?;
                        if admission_registration(&admission.state)
                            .map(|value| value.registration_id.as_str())
                            != Some(request.registration_id.as_str())
                        {
                            return Err(conflict());
                        }
                    }
                    OrdinaryStopBootstrapV1::Fork { completion } => {
                        let (actual, _) = registry
                            .ordinary_forks_v1
                            .as_ref()
                            .and_then(|forks| forks.inspection_record(session, target))
                            .ok_or_else(conflict)?;
                        if actual.checked_native_completion() != Some(completion.as_ref())
                            || actual
                                .inspection_applied_registration(root)?
                                .as_ref()
                                .map(|value| value.registration_id.as_str())
                                != Some(request.registration_id.as_str())
                        {
                            return Err(conflict());
                        }
                    }
                }
                revision = revision
                    .checked_add(record.record_revision)
                    .ok_or_else(conflict)?;
            }
        }
        if revision == 0
            || revision != self.body.revision
            || self.body.authority_store_id != registry.authority_store_id
            || self.body.key_id != registry.commitment_key.key_id
        {
            return Err(conflict());
        }
        Ok(())
    }

    pub(super) fn record(&self, session: &str, target: &str) -> Option<&OrdinaryStopRecordV1> {
        self.body.records_by_session.get(session)?.get(target)
    }

    pub(super) fn records_for_session(
        &self,
        session: &str,
    ) -> impl Iterator<Item = (&String, &OrdinaryStopRecordV1)> {
        self.body
            .records_by_session
            .get(session)
            .into_iter()
            .flat_map(BTreeMap::iter)
    }

    // Called only after the surrounding registry MAC and full R0/native joins
    // have been validated. Pending intents and bootstrap exits release nothing.
    pub(super) fn acknowledged_counts(
        &self,
        registry: &RetainedWorkerAdmissionRegistryV1,
    ) -> Result<(u64, u64), RetainedWorkerRuntimeError> {
        let mut spawns = 0u64;
        let mut forks = 0u64;
        for records in self.body.records_by_session.values() {
            for record in records
                .values()
                .filter(|record| record.acknowledgment.is_some())
            {
                match &record.bootstrap {
                    OrdinaryStopBootstrapV1::Spawn { .. } => {
                        let admission = registry
                            .records_by_session
                            .get(&record.request.orchestration_session_id)
                            .and_then(|rows| rows.get(&record.request.target_participant_id))
                            .ok_or_else(conflict)?;
                        // Existing legacy terminal accounting may already have
                        // removed this slot; never credit a second release.
                        if admission_state_is_live(&admission.state) {
                            spawns = spawns.checked_add(1).ok_or_else(conflict)?;
                        }
                    }
                    OrdinaryStopBootstrapV1::Fork { .. } => {
                        forks = forks.checked_add(1).ok_or_else(conflict)?;
                    }
                }
            }
        }
        Ok((spawns, forks))
    }

    // Data transition only. The runtime publisher must independently authenticate
    // the actual current Stop grant and complete original physical work sources.
    pub(super) fn insert_pending(
        &mut self,
        request: OrdinaryRetainedWorkerStopRequestV1,
        bootstrap: OrdinaryStopBootstrapV1,
        idempotency_key: String,
        accepted_work_ids_at_admission: Vec<String>,
        admitted_at: TimestampV1,
        key: &RetainedWorkerAdmissionKeyEnvelopeV1,
    ) -> Result<(OrdinaryStopRecordV1, bool), RetainedWorkerRuntimeError> {
        bootstrap.validate_request(&request)?;
        if let Some(existing) = self.record(
            &request.orchestration_session_id,
            &request.target_participant_id,
        ) {
            if existing.request != request
                || existing.bootstrap != bootstrap
                || existing.idempotency_key != idempotency_key
                || existing.accepted_work_ids_at_admission != accepted_work_ids_at_admission
            {
                return Err(conflict());
            }
            return Ok((existing.clone(), false));
        }
        if self
            .body
            .records_by_session
            .get(&request.orchestration_session_id)
            .is_some_and(|records| {
                records
                    .values()
                    .any(|record| record.request.stop_request_id == request.stop_request_id)
            })
        {
            return Err(conflict());
        }
        let record = OrdinaryStopRecordV1 {
            request,
            bootstrap,
            idempotency_key,
            accepted_work_ids_at_admission,
            admitted_at,
            acknowledgment: None,
            completed_at: None,
            record_revision: 1,
        };
        record.validate()?;
        self.body
            .records_by_session
            .entry(record.request.orchestration_session_id.clone())
            .or_default()
            .insert(record.request.target_participant_id.clone(), record.clone());
        self.body.revision = self.body.revision.checked_add(1).ok_or_else(conflict)?;
        self.mac_hex = self.expected_mac(key)?;
        Ok((record, true))
    }

    // Completion is cleanup of this original intent. The service report is data
    // until its actual producer, ownership and exact original sources are joined.
    pub(super) fn complete(
        &mut self,
        acknowledgment: OrdinaryRetainedWorkerStopAcknowledgmentV1,
        closed_accepted_work_ids: &[String],
        completed_at: TimestampV1,
        key: &RetainedWorkerAdmissionKeyEnvelopeV1,
    ) -> Result<(OrdinaryStopRecordV1, bool), RetainedWorkerRuntimeError> {
        let request = &acknowledgment.request;
        let record = self
            .body
            .records_by_session
            .get_mut(&request.orchestration_session_id)
            .and_then(|records| records.get_mut(&request.target_participant_id))
            .ok_or_else(conflict)?;
        acknowledgment
            .validate(&record.request)
            .map_err(|_| conflict())?;
        if closed_accepted_work_ids != record.accepted_work_ids_at_admission {
            return Err(conflict());
        }
        if let Some(existing) = &record.acknowledgment {
            if existing != &acknowledgment {
                return Err(conflict());
            }
            return Ok((record.clone(), false));
        }
        record.acknowledgment = Some(acknowledgment);
        record.completed_at = Some(completed_at);
        record.record_revision = record.record_revision.checked_add(1).ok_or_else(conflict)?;
        record.validate()?;
        let record = record.clone();
        self.body.revision = self.body.revision.checked_add(1).ok_or_else(conflict)?;
        self.mac_hex = self.expected_mac(key)?;
        Ok((record, true))
    }
}

/// Data prepared by the actual admission publisher. It is not a service
/// retirement acknowledgment, native-send permit or lifecycle closeout.
#[cfg(target_os = "linux")]
pub(crate) struct PreparedOrdinaryWorkerStopV1 {
    record: OrdinaryStopRecordV1,
    active_acceptance_record: Option<super::super::state_store::WorldWorkAcceptanceRecordV1>,
    observed_current_caller: String,
    observed_host_posture: super::super::host_session_authority::schema::HostSessionPostureV1,
}

#[cfg(target_os = "linux")]
impl PreparedOrdinaryWorkerStopV1 {
    pub(crate) fn policy_request(
        &self,
    ) -> anyhow::Result<super::super::dispatch_contract::ValidatedWorldDispatchRequestV1> {
        use super::super::dispatch_contract::*;
        let request = &self.record.request;
        WorldDispatchRequestV1 {
            request_id: Some(request.stop_request_id.clone()),
            idempotency_key: Some(self.record.idempotency_key.clone()),
            orchestration_session_id: Some(request.orchestration_session_id.clone()),
            caller_participant_id: Some(request.caller_participant_id.clone()),
            action: WorldDispatchActionV1::StopWorldWorker,
            mode: WorldDispatchModeV1::Retained,
            target_backend_id: Some(request.target_backend_id.clone()),
            task_run_id: None,
            target_participant_id: Some(request.target_participant_id.clone()),
            world_id: Some(request.world_id.clone()),
            world_generation: Some(request.world_generation),
            dispatch_policy_narrowing: None,
            payload: WorldDispatchPayloadV1::WorkerStop(WorkerStopPayloadV1::default()),
        }
        .validate()
    }
    pub(crate) fn source_start(&self) -> &transport_api_types::OrdinaryStartSelectionCarrierV1 {
        &self.record.bootstrap.completion().source_start
    }
    pub(crate) fn observes_active_controller(&self, caller: &str) -> bool {
        self.observed_current_caller == caller && self.observed_host_posture
            == super::super::host_session_authority::schema::HostSessionPostureV1::ActiveAttached
    }
    pub(crate) fn request(&self) -> &OrdinaryRetainedWorkerStopRequestV1 {
        &self.record.request
    }
    pub(crate) fn acknowledgment(&self) -> Option<&OrdinaryRetainedWorkerStopAcknowledgmentV1> {
        self.record.acknowledgment.as_ref()
    }
    pub(crate) fn active_acceptance_record(
        &self,
    ) -> Option<&super::super::state_store::WorldWorkAcceptanceRecordV1> {
        self.active_acceptance_record.as_ref()
    }
}

#[cfg(target_os = "linux")]
impl RetainedWorkerRuntime {
    pub(crate) fn prepare_ordinary_stop_intent_v1(
        &self,
        selection: &super::super::native_launch_context::AuthenticatedOrdinarySelectionV1,
        request: &super::super::dispatch_contract::ValidatedWorldDispatchRequestV1,
    ) -> Result<PreparedOrdinaryWorkerStopV1, RetainedWorkerRuntimeError> {
        use super::super::dispatch_contract::{
            WorldDispatchActionV1, WorldDispatchModeV1, WorldDispatchPayloadV1,
        };
        use crate::execution::orchestrator_world_dispatch::{
            enforce_world_dispatch_read_or_effect_policy, ordinary_start_selection_carrier,
        };
        if request.action != WorldDispatchActionV1::StopWorldWorker
            || request.mode != WorldDispatchModeV1::Retained
            || request.task_run_id.is_some()
            || request.dispatch_policy_narrowing.is_some()
            || !matches!(request.payload, WorldDispatchPayloadV1::WorkerStop(_))
        {
            return Err(RetainedWorkerRuntimeError(
                "ordinary Stop requires an exact plain retained lifecycle request".into(),
            ));
        }
        let target = request
            .target_participant_id
            .as_deref()
            .ok_or_else(conflict)?;
        selection.with_ordinary_lifecycle_control_sources_v1(target, |context, source, verify_native, record_publication| {
            let current = &source.facts.current;
            let world = current.authority.world_binding.as_ref().ok_or_else(|| anyhow::anyhow!("ordinary Stop current World absent"))?;
            let keys = source.admission_key_files();
            let (_, grant_registry) = decode_held_retained_admission_source(source.snapshot(), Some(source.admission_registry_bytes()),
                &keys, &request.orchestration_session_id, world).map_err(|error| anyhow::anyhow!("{error}"))?;
            let original_start = ordinary_start_selection_carrier(selection)?;
            let initial_grant = grant_registry.as_ref().and_then(|registry| registry.ordinary_bootstrap_receivers_v1.as_ref())
                .map(|receivers| receivers.initial_stop_grant_for(&request.orchestration_session_id, target, &original_start))
                .transpose().map_err(|error| anyhow::anyhow!("{error}"))?.flatten().cloned();
            if context.selections.native.placement != transport_api_types::AgentSelectionPlacementV1::Host
                || context.selections.requested.placement != transport_api_types::AgentSelectionPlacementV1::World
                || request.orchestration_session_id != context.orchestration_session_id
                || request.target_backend_id != context.selections.requested.backend_id
                || request.world_id != world.world_id || request.world_generation != world.world_generation {
                anyhow::bail!("ordinary Stop original role or World source differs");
            }
            if let Some(grant) = &initial_grant {
                anyhow::ensure!(request.caller_participant_id == grant.activation.caller_participant_id,
                    "ordinary Stop initial grant caller changed; history grants no transfer");
            } else if current.authority.lifecycle_posture != HostSessionPostureV1::ActiveAttached
                || current.caller.descriptor.execution_scope != AgentExecutionScopeV1::Host
                || current.caller.descriptor.agent_id != context.selections.native.realized_agent_id
                || current.caller.descriptor.backend_id != context.selections.native.backend_id
                || current.caller.descriptor.protocol != context.descriptor.protocol
                || current.caller.descriptor.backend_kind != context.descriptor.backend_kind
                || request.caller_participant_id != current.caller.participant_id {
                anyhow::bail!("ordinary Stop fresh current actor differs");
            }
            let verify = |owned_temp: Option<&str>| -> anyhow::Result<()> {
                verify_native(owned_temp)?;
                let home = selection.held_authority().bootstrap_home();
                let policy = crate::execution::policy_model::resolve_effective_policy_for_bootstrap_home(
                    std::path::Path::new(&context.workspace_binding.workspace_root.physical_path), &home)?;
                enforce_world_dispatch_read_or_effect_policy(request, &policy)
            };
            verify(None)?;
            let row = read_ordinary_retained_worker_from_sources_v1(source.snapshot(), Some(source.admission_registry_bytes()),
                &keys, &request.orchestration_session_id, world, target, &ordinary_start_selection_carrier(selection)?)
                .map_err(|error| anyhow::anyhow!("{error}"))?;
            let graph = source.target_graph.as_ref().ok_or_else(|| anyhow::anyhow!("ordinary Stop has no applied R0 source"))?;
            row.validate_inspected_graph(graph).map_err(|error| anyhow::anyhow!("{error}"))?;
            if row.inspection_registration().is_none() || graph.descriptor.agent_id != context.selections.requested.realized_agent_id {
                anyhow::bail!("ordinary Stop requires the actual requested-role R0 registration");
            }
            let (_, registry) = decode_held_retained_admission_source(source.snapshot(), Some(source.admission_registry_bytes()),
                &keys, &request.orchestration_session_id, world).map_err(|error| anyhow::anyhow!("{error}"))?;
            let registry = registry.ok_or_else(|| anyhow::anyhow!("ordinary Stop admission absent"))?;
            let bootstrap = match &row {
                OrdinaryRetainedWorkerReadV1::Spawn { native_bootstrap_completed: true, .. } => OrdinaryStopBootstrapV1::Spawn {
                    completion: Box::new(registry.native_lifecycle_v1.as_ref().and_then(|lifecycle|
                        lifecycle.native_completion_for(&request.orchestration_session_id, target)).ok_or_else(|| anyhow::anyhow!("ordinary Stop checked Spawn source absent"))?.clone()),
                },
                OrdinaryRetainedWorkerReadV1::Fork { record, native_bootstrap_completed: true, .. } => OrdinaryStopBootstrapV1::Fork {
                    completion: Box::new(record.checked_native_completion().ok_or_else(|| anyhow::anyhow!("ordinary Stop checked Fork source absent"))?.clone()),
                },
                _ => anyhow::bail!("ordinary Stop native bootstrap remains unresolved"),
            };
            let observed = bootstrap.completion();
            if observed.source_start != ordinary_start_selection_carrier(selection)? { anyhow::bail!("ordinary Stop original Start differs"); }
            if let Some(grant) = &initial_grant {
                anyhow::ensure!(grant.source_start == observed.source_start && grant.activation == observed.activation,
                    "ordinary Stop initial grant differs from actual completed bootstrap");
            }
            let stop_request_id = initial_grant.as_ref().map_or_else(|| request.request_id.clone(), |grant| grant.control_request_id.clone());
            let idempotency_key = initial_grant.as_ref().map_or_else(|| request.idempotency_key.clone(), |grant| grant.control_request_id.clone());
            let transport = OrdinaryRetainedWorkerStopRequestV1 {
                schema_version: 1, stop_request_id, authority_store_id: selection.authority_store_id().into(),
                orchestration_session_id: request.orchestration_session_id.clone(), caller_participant_id: request.caller_participant_id.clone(),
                target_participant_id: target.into(), target_backend_id: request.target_backend_id.clone(), world_id: request.world_id.clone(),
                world_generation: request.world_generation, registration_id: graph.registration.registration_id.clone(),
                bootstrap_run_id: observed.activation.bootstrap_run_id.clone(), native_session_id: observed.native_session_id.clone(),
                worker_cap_ref: observed.activation.immutable_worker_cap_ref.clone(),
            };
            bootstrap.validate_request(&transport).map_err(|error| anyhow::anyhow!("{error}"))?;
            let cut = checked_ordinary_stop_accepted_cut(source, &transport, &context.selections.native.backend_id)?;
            if cut.nonterminal_acceptances.len() > 1 { anyhow::bail!("ordinary Stop active target is ambiguous"); }
            let active_acceptance_record = cut.nonterminal_acceptances.into_iter().next();
            if active_acceptance_record.as_ref().is_some_and(|record| record.caller_participant_id != request.caller_participant_id) {
                anyhow::bail!("ordinary Stop active receipt belongs to another actor; history grants no cancellation transfer");
            }
            let accepted_ids = cut.accepted_work_ids;
            if let Some(existing) = registry.ordinary_stops_v1.as_ref().and_then(|stops| stops.record(&request.orchestration_session_id, target)) {
                let same_request_id = existing.request.stop_request_id == transport.stop_request_id;
                let mut expected = transport;
                expected.stop_request_id = existing.request.stop_request_id.clone();
                if existing.acknowledgment.is_some() { expected.caller_participant_id = existing.request.caller_participant_id.clone(); }
                if existing.request != expected || existing.bootstrap != bootstrap || existing.accepted_work_ids_at_admission != accepted_ids
                    || (same_request_id && existing.idempotency_key != idempotency_key)
                    || (existing.acknowledgment.is_some() && active_acceptance_record.is_some()) {
                    anyhow::bail!("ordinary Stop retry changed exact original intent or work coverage");
                }
                verify(None)?;
                return Ok(PreparedOrdinaryWorkerStopV1 { record: existing.clone(), active_acceptance_record,
                    observed_current_caller: current.caller.participant_id.clone(), observed_host_posture: current.authority.lifecycle_posture });
            }
            let (_, key) = load_committed_admission_key(selection.authority_store_id(), &registry, &keys)
                .map_err(|error| anyhow::anyhow!("{error}"))?;
            let admitted_at = TimestampV1::parse(chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Nanos, true))
                .map_err(|error| anyhow::anyhow!("{error}"))?;
            let mut nonce = [0u8; 16]; rand::rngs::OsRng.fill_bytes(&mut nonce);
            let temp_name = format!("admission-registry--{}.tmp", lower_hex(&nonce));
            let storage = super::super::host_session_authority::store::retained_worker_admission_storage_existing_for_authority(selection.held_authority())
                .map_err(|error| anyhow::anyhow!("{error:?}"))?;
            let mut semantic_failure = None;
            let result = storage.transaction_existing_control(&|| verify(None)
                .map_err(|_| super::super::host_session_authority::store::BootstrapError::ordinary_control_source_guard()), |transaction| {
                if transaction.read_registry()?.as_deref() != Some(source.admission_registry_bytes()) {
                    return Err(super::super::host_session_authority::store::BootstrapError::ordinary_control_source_guard());
                }
                let mut candidate = registry;
                let mut stops = candidate.ordinary_stops_v1.take().unwrap_or_else(|| OrdinaryStopRegistryV1::new(&key));
                let (record, changed) = retain_semantic_error(stops.insert_pending(transport, bootstrap,
                    idempotency_key.clone(), accepted_ids, admitted_at, &key), &mut semantic_failure)?;
                candidate.ordinary_stops_v1 = Some(stops);
                if !changed { return Ok(record); }
                let bytes = retain_semantic_error(encode_canonical(&candidate, "encode ordinary Stop intent"), &mut semantic_failure)?;
                retain_semantic_error(decode_held_retained_admission_source(source.snapshot(), Some(&bytes), &keys,
                    &request.orchestration_session_id, world), &mut semantic_failure)?;
                transaction.replace_registry_with_source_guard(&temp_name, &bytes, &|| verify(Some(&temp_name))
                    .map_err(|_| super::super::host_session_authority::store::BootstrapError::ordinary_control_source_guard()))?;
                retain_semantic_error(record_publication(&bytes).map_err(|error| RetainedWorkerRuntimeError(error.to_string())), &mut semantic_failure)?;
                Ok(record)
            });
            let record = match result {
                Ok(record) => record,
                Err(error) => return Err(anyhow::anyhow!("{}", semantic_failure.unwrap_or_else(|| RetainedWorkerRuntimeError(format!("ordinary Stop publication refused: {error:?}"))))),
            };
            Ok(PreparedOrdinaryWorkerStopV1 { record, active_acceptance_record,
                observed_current_caller: current.caller.participant_id.clone(), observed_host_posture: current.authority.lifecycle_posture })
        }).map_err(|error| RetainedWorkerRuntimeError(error.to_string()))
    }
}

/// Denial-only check over original owner bytes. Callers retain the original
/// physical guards through their publication boundary; this grants no work.
#[cfg(target_os = "linux")]
pub(crate) fn require_ordinary_retained_target_open_from_sources(
    root: &VersionedStateRoot,
    bytes: &[u8],
    keys: &[(String, Vec<u8>)],
    session: &str,
    target: &str,
) -> Result<(), RetainedWorkerRuntimeError> {
    let registry: RetainedWorkerAdmissionRegistryV1 =
        decode_canonical(bytes, "decode ordinary retained lifecycle gate")?;
    load_committed_admission_key(root.authority_store_id(), &registry, keys)?;
    validate_admission_registry(&registry, root)?;
    if registry
        .ordinary_stops_v1
        .as_ref()
        .and_then(|stops| stops.record(session, target))
        .is_some()
        || registry
            .ordinary_bootstrap_receivers_v1
            .as_ref()
            .is_some_and(|receivers| {
                receivers.has_requested_initial_stop(session, target)
                    || receivers.retirement_for(session, target).is_some()
            })
    {
        return Err(RetainedWorkerRuntimeError(
            "ordinary_retained_stop_pending_or_closed: new retained work refused".into(),
        ));
    }
    Ok(())
}

/// Original selection and bootstrap identity for checking an existing Stop.
/// This reader retains no World, cgroup, gateway or exclusion lease. It can be
/// retained with a service acknowledgment after the actual member is retired.
/// It grants no native launch or account/MaySend permission.
#[cfg(target_os = "linux")]
pub struct OrdinaryRetainedStopControlReaderV1 {
    selection:
        std::sync::Arc<super::super::native_launch_context::AuthenticatedOrdinarySelectionV1>,
    activation: transport_api_types::E2MemberLaunchActivationCarrierV1,
    original_start: transport_api_types::OrdinaryStartSelectionCarrierV1,
}

/// Scoped permission to finish one already-admitted Stop. Construction is
/// private and the original physical sources and Root lock outlive this value.
/// The consumer must separately authenticate its actual parked native member.
#[cfg(target_os = "linux")]
pub struct OrdinaryRetainedStopAdmissionV1<'a> {
    request: &'a OrdinaryRetainedWorkerStopRequestV1,
    existing_acknowledgment: Option<&'a OrdinaryRetainedWorkerStopAcknowledgmentV1>,
    verify: &'a dyn Fn() -> anyhow::Result<()>,
}

#[cfg(target_os = "linux")]
impl OrdinaryRetainedStopAdmissionV1<'_> {
    /// Exact identity data; this does not prove that a native member retired.
    /// ```no_run
    /// # fn check(value: &substrate_shell::OrdinaryRetainedStopAdmissionV1<'_>) {
    /// let request = value.request();
    /// # let _ = request;
    /// # }
    /// ```
    pub fn request(&self) -> &OrdinaryRetainedWorkerStopRequestV1 {
        self.request
    }

    /// Historical closeout data, never a grant to retire a new member. A service
    /// may join only its own original cached acknowledgment to this value.
    /// ```no_run
    /// # fn check(value: &substrate_shell::OrdinaryRetainedStopAdmissionV1<'_>) {
    /// let previous = value.existing_acknowledgment();
    /// # let _ = previous;
    /// # }
    /// ```
    pub fn existing_acknowledgment(&self) -> Option<&OrdinaryRetainedWorkerStopAcknowledgmentV1> {
        self.existing_acknowledgment
    }

    /// Check original source custody and fresh Stop policy immediately before
    /// the actual member owner removes its parked registration.
    /// ```no_run
    /// # fn check(value: &substrate_shell::OrdinaryRetainedStopAdmissionV1<'_>) -> anyhow::Result<()> {
    /// value.revalidate_before_retirement()
    /// # }
    /// ```
    pub fn revalidate_before_retirement(&self) -> anyhow::Result<()> {
        (self.verify)()
    }
}

#[cfg(target_os = "linux")]
impl OrdinaryRetainedStopControlReaderV1 {
    pub(crate) fn from_bootstrap(
        selection: std::sync::Arc<
            super::super::native_launch_context::AuthenticatedOrdinarySelectionV1,
        >,
        activation: transport_api_types::E2MemberLaunchActivationCarrierV1,
    ) -> anyhow::Result<Self> {
        activation.validate().map_err(anyhow::Error::msg)?;
        let original_start =
            crate::execution::orchestrator_world_dispatch::ordinary_start_selection_carrier(
                &selection,
            )?;
        anyhow::ensure!(
            activation.immutable_worker_cap_ref.authority_store_id
                == original_start.authority_store_id
                && activation.commitment_ref.authority_store_id
                    == original_start.authority_store_id
                && activation.orchestration_session_id == original_start.original_session_id
                && activation.target_backend_id == selection.requested_selection().backend_id,
            "ordinary Stop reader changed original bootstrap scope"
        );
        Ok(Self {
            selection,
            activation,
            original_start,
        })
    }

    /// Check the original admitted intent and its complete accepted-work cut,
    /// without current account acquisition, executing refresh or MaySend. The
    /// kernel-authenticated UDS peer must be the original native principal.
    ///
    /// This is cleanup of the original intent, not transfer of control to a
    /// later Host actor. Fresh Stop policy still applies. Active accepted work
    /// yields NotQuiescent; it never reaches the retirement callback.
    ///
    /// The synchronous callback holds the existing Root lock and original
    /// physical guards. It must perform only local member-owner retirement or
    /// acknowledgment observation; no IPC, native launch or authority write.
    /// ```no_run
    /// # #[cfg(target_os = "linux")]
    /// # fn checked(reader: &substrate_shell::OrdinaryRetainedStopControlReaderV1,
    /// # request: &transport_api_types::OrdinaryRetainedWorkerStopRequestV1, uid: u32) -> anyhow::Result<()> {
    /// let result = reader.with_locked_retirement(request, uid, |admission| {
    ///     admission.revalidate_before_retirement()?;
    ///     Ok(transport_api_types::OrdinaryRetainedWorkerStopResponseV1::UnknownOwner {})
    /// })?;
    /// # let _ = result; Ok(())
    /// # }
    /// ```
    pub fn with_locked_retirement(
        &self,
        request: &OrdinaryRetainedWorkerStopRequestV1,
        peer_uid: u32,
        retire: impl FnOnce(
            &OrdinaryRetainedStopAdmissionV1<'_>,
        )
            -> anyhow::Result<transport_api_types::OrdinaryRetainedWorkerStopResponseV1>,
    ) -> anyhow::Result<transport_api_types::OrdinaryRetainedWorkerStopResponseV1> {
        use super::super::dispatch_contract::*;
        use transport_api_types::{
            OrdinaryRetainedWorkerStopBusyReasonV1 as Busy,
            OrdinaryRetainedWorkerStopResponseV1 as Response,
        };
        request.validate().map_err(anyhow::Error::msg)?;
        anyhow::ensure!(
            peer_uid != 0,
            "ordinary Stop requires the actual native UDS principal"
        );
        self.selection.with_ordinary_lifecycle_control_sources_v1(&request.target_participant_id,
            |context, source, verify_native, _record_publication| {
                for principal in [self.selection.native_principal(), self.selection.requested_principal()] {
                    anyhow::ensure!(matches!(principal, transport_api_types::PlatformPrincipalV1::Unix { uid, .. } if *uid == peer_uid),
                        "ordinary Stop UDS peer differs from original native principal");
                }
                let world = WorldBindingV1 { world_id: request.world_id.clone(), world_generation: request.world_generation };
                let keys = source.admission_key_files();
                let (_, registry) = decode_held_retained_admission_source(source.snapshot(), Some(source.admission_registry_bytes()),
                    &keys, &request.orchestration_session_id, &world).map_err(|error| anyhow::anyhow!("{error}"))?;
                let registry = registry.ok_or_else(|| anyhow::anyhow!("ordinary Stop admission absent"))?;
                let intent = registry.ordinary_stops_v1.as_ref().and_then(|stops|
                    stops.record(&request.orchestration_session_id, &request.target_participant_id))
                    .ok_or_else(|| anyhow::anyhow!("ordinary Stop has no actual admitted intent"))?;
                let observation = intent.bootstrap.completion();
                anyhow::ensure!(&intent.request == request && observation.activation == self.activation
                    && observation.source_start == self.original_start
                    && self.original_start == crate::execution::orchestrator_world_dispatch::ordinary_start_selection_carrier(&self.selection)?
                    && context.selections.native.placement == transport_api_types::AgentSelectionPlacementV1::Host
                    && context.selections.requested.placement == transport_api_types::AgentSelectionPlacementV1::World,
                    "ordinary Stop request changed original admitted native target");
                let row = read_ordinary_retained_worker_from_sources_v1(source.snapshot(), Some(source.admission_registry_bytes()),
                    &keys, &request.orchestration_session_id, &world, &request.target_participant_id, &self.original_start)
                    .map_err(|error| anyhow::anyhow!("{error}"))?;
                let graph = source.target_graph.as_ref().ok_or_else(|| anyhow::anyhow!("ordinary Stop applied R0 absent"))?;
                row.validate_inspected_graph(graph).map_err(|error| anyhow::anyhow!("{error}"))?;
                anyhow::ensure!(graph.registration.registration_id == request.registration_id
                    && graph.descriptor.agent_id == context.selections.requested.realized_agent_id,
                    "ordinary Stop changed original R0/requested-role binding");
                let policy_request = WorldDispatchRequestV1 {
                    request_id: Some(request.stop_request_id.clone()), idempotency_key: Some(intent.idempotency_key.clone()),
                    orchestration_session_id: Some(request.orchestration_session_id.clone()), caller_participant_id: Some(request.caller_participant_id.clone()),
                    action: WorldDispatchActionV1::StopWorldWorker, mode: WorldDispatchModeV1::Retained,
                    target_backend_id: Some(request.target_backend_id.clone()), task_run_id: None,
                    target_participant_id: Some(request.target_participant_id.clone()), world_id: Some(request.world_id.clone()),
                    world_generation: Some(request.world_generation), dispatch_policy_narrowing: None,
                    payload: WorldDispatchPayloadV1::WorkerStop(WorkerStopPayloadV1::default()),
                }.validate()?;
                let verify = || -> anyhow::Result<()> {
                    verify_native(None)?;
                    let policy = crate::execution::policy_model::resolve_effective_policy_for_bootstrap_home(
                        std::path::Path::new(&context.workspace_binding.workspace_root.physical_path), &self.selection.held_authority().bootstrap_home())?;
                    crate::execution::orchestrator_world_dispatch::enforce_world_dispatch_read_or_effect_policy(&policy_request, &policy)
                };
                verify()?;
                let cut = checked_ordinary_stop_accepted_cut(source, request, &context.selections.native.backend_id)?;
                anyhow::ensure!(cut.accepted_work_ids == intent.accepted_work_ids_at_admission,
                    "ordinary Stop original accepted-work cut changed");
                let active = !cut.nonterminal_acceptances.is_empty();
                let original_root = VersionedStateRoot::decode(source.snapshot().hsa_state_root_bytes())?.to_canonical_bytes()?;
                // This locked read serializes against legitimate admission/B1
                // publishers. Equal bytes are only a join: the original held
                // capture, never this second read, remains the physical guard.
                super::super::host_session_authority::store::with_existing_retained_worker_admission_read(self.selection.held_authority(),
                    |root, bytes, locked_keys| -> anyhow::Result<Response> {
                        anyhow::ensure!(root.to_canonical_bytes()? == original_root
                            && bytes == Some(source.admission_registry_bytes())
                            && locked_keys.into_iter().collect::<BTreeMap<_, _>>() == keys.iter().cloned().collect::<BTreeMap<_, _>>(),
                            "ordinary Stop locked original source changed");
                        verify()?;
                        if active { return Ok(Response::NotQuiescent { reason: Busy::AcceptedTurn }); }
                        let admission = OrdinaryRetainedStopAdmissionV1 { request,
                            existing_acknowledgment: intent.acknowledgment.as_ref(), verify: &verify };
                        retire(&admission)
                    }).map_err(|error| anyhow::anyhow!("ordinary Stop locked source refused: {error:?}"))?
            })
    }
}

#[cfg(target_os = "linux")]
struct CheckedOrdinaryStopAcceptedCutV1 {
    accepted_work_ids: Vec<String>,
    nonterminal_acceptances: Vec<super::super::state_store::WorldWorkAcceptanceRecordV1>,
}

// Data-only interpretation under the caller's original full physical capture.
// The publisher separately checks its current actor; the service never grants
// cancellation transfer and reaches retirement only when this whole cut closed.
#[cfg(target_os = "linux")]
fn checked_ordinary_stop_accepted_cut(
    source: &super::super::host_session_authority::facade::OrdinaryRetainedLifecycleControlReadSourceV1,
    request: &OrdinaryRetainedWorkerStopRequestV1,
    native_backend: &str,
) -> anyhow::Result<CheckedOrdinaryStopAcceptedCutV1> {
    checked_ordinary_stop_accepted_cut_from_sources(
        source.snapshot(),
        source.supervisor_bytes(),
        request,
        native_backend,
    )
}

#[cfg(target_os = "linux")]
fn checked_ordinary_stop_accepted_cut_from_sources(
    snapshot: &super::super::host_session_authority::store::AcceptedWorkAuthorityPhysicalSnapshotV1,
    supervisor_bytes: Option<&[u8]>,
    request: &OrdinaryRetainedWorkerStopRequestV1,
    native_backend: &str,
) -> anyhow::Result<CheckedOrdinaryStopAcceptedCutV1> {
    use super::super::state_store::{
        authenticate_ordinary_world_work_session_cut_v1, AcceptedWorldWorkIdentityV1,
        ProposedWorldWorkIdentityV1,
    };
    use super::super::world_work_execution_supervisor::{
        decode_ordinary_accepted_observation, decode_world_controller_supervisor_observations,
    };
    let world = WorldBindingV1 {
        world_id: request.world_id.clone(),
        world_generation: request.world_generation,
    };
    let session = authenticate_ordinary_world_work_session_cut_v1(
        snapshot,
        &request.orchestration_session_id,
        &world,
    )?;
    let records = session.acceptances().iter().filter(|record| matches!(&record.work_identity,
        AcceptedWorldWorkIdentityV1::RetainedTurn { target_participant_id, .. } if target_participant_id == &request.target_participant_id)).collect::<Vec<_>>();
    if session.proposals().iter().any(|proposal| matches!(&proposal.proposed_work,
        ProposedWorldWorkIdentityV1::RetainedTurn { target_participant_id, .. } if target_participant_id == &request.target_participant_id)
        && !records.iter().any(|record| record.acceptance_record_id == proposal.acceptance_context.proposed_acceptance_record_id)) {
        anyhow::bail!("ordinary Stop target has unresolved unaccepted work coverage");
    }
    let mut accepted_work_ids = Vec::new();
    let mut nonterminal_acceptances = Vec::new();
    for record in &records {
        anyhow::ensure!(
            record.target_backend_id == request.target_backend_id
                && record.caller_backend_id == native_backend,
            "ordinary Stop accepted role changed"
        );
        let checked = decode_ordinary_accepted_observation(snapshot, supervisor_bytes, record)?;
        if checked.terminal.is_none() {
            nonterminal_acceptances.push((*record).clone());
        }
        accepted_work_ids.push(record.acceptance_record_id.clone());
    }
    accepted_work_ids.sort();
    for observation in decode_world_controller_supervisor_observations(
        supervisor_bytes,
        snapshot.authority_store_id(),
        &request.orchestration_session_id,
    )? {
        if matches!(&observation.claim.work_identity,
            AcceptedWorldWorkIdentityV1::RetainedTurn { target_participant_id, .. } if target_participant_id == &request.target_participant_id)
            && !records
                .iter()
                .any(|record| observation.claim.matches_acceptance(record))
        {
            anyhow::bail!("ordinary Stop supervisor target has unclassified receipt coverage");
        }
    }
    Ok(CheckedOrdinaryStopAcceptedCutV1 {
        accepted_work_ids,
        nonterminal_acceptances,
    })
}

#[cfg(target_os = "linux")]
impl RetainedWorkerRuntime {
    /// Cleanup publication requires non-deserializable connected service
    /// provenance, then independently rechecks the original intent and work.
    /// A raw decoded ACK is not accepted by this owner API.
    pub(crate) fn complete_ordinary_stop_v1(
        &self,
        selection: &super::super::native_launch_context::AuthenticatedOrdinarySelectionV1,
        prepared: &PreparedOrdinaryWorkerStopV1,
        service: transport_api_client::OrdinaryRetainedStopServiceResponseV1,
    ) -> Result<PreparedOrdinaryWorkerStopV1, RetainedWorkerRuntimeError> {
        let transport_api_types::OrdinaryRetainedWorkerStopResponseV1::Retired { acknowledgment } =
            service.response()
        else {
            return Err(RetainedWorkerRuntimeError(
                "ordinary Stop has no actual service retirement acknowledgment".into(),
            ));
        };
        acknowledgment
            .validate(prepared.request())
            .map_err(|_| conflict())?;
        selection.with_ordinary_lifecycle_control_sources_v1(&prepared.request().target_participant_id,
            |context, source, verify_native, record_publication| {
                let request = prepared.request();
                let world = WorldBindingV1 { world_id: request.world_id.clone(), world_generation: request.world_generation };
                let keys = source.admission_key_files();
                let (_, registry) = decode_held_retained_admission_source(source.snapshot(), Some(source.admission_registry_bytes()),
                    &keys, &request.orchestration_session_id, &world).map_err(|error| anyhow::anyhow!("{error}"))?;
                let registry = registry.ok_or_else(|| anyhow::anyhow!("ordinary Stop closeout original admission absent"))?;
                let intent = registry.ordinary_stops_v1.as_ref().and_then(|stops|
                    stops.record(&request.orchestration_session_id, &request.target_participant_id))
                    .ok_or_else(|| anyhow::anyhow!("ordinary Stop closeout original intent absent"))?;
                anyhow::ensure!(intent.request == prepared.record.request && intent.bootstrap == prepared.record.bootstrap
                    && intent.idempotency_key == prepared.record.idempotency_key && intent.admitted_at == prepared.record.admitted_at
                    && intent.accepted_work_ids_at_admission == prepared.record.accepted_work_ids_at_admission
                    && intent.bootstrap.completion().source_start
                        == crate::execution::orchestrator_world_dispatch::ordinary_start_selection_carrier(selection)?,
                    "ordinary Stop closeout changed immutable original intent");
                acknowledgment.validate(&intent.request).map_err(anyhow::Error::msg)?;
                let row = read_ordinary_retained_worker_from_sources_v1(source.snapshot(), Some(source.admission_registry_bytes()),
                    &keys, &request.orchestration_session_id, &world, &request.target_participant_id,
                    &crate::execution::orchestrator_world_dispatch::ordinary_start_selection_carrier(selection)?)
                    .map_err(|error| anyhow::anyhow!("{error}"))?;
                let graph = source.target_graph.as_ref().ok_or_else(|| anyhow::anyhow!("ordinary Stop closeout applied R0 absent"))?;
                row.validate_inspected_graph(graph).map_err(|error| anyhow::anyhow!("{error}"))?;
                anyhow::ensure!(graph.registration.registration_id == request.registration_id
                    && graph.descriptor.agent_id == context.selections.requested.realized_agent_id,
                    "ordinary Stop closeout original requested-role R0 changed");
                let cut = checked_ordinary_stop_accepted_cut(source, request, &context.selections.native.backend_id)?;
                anyhow::ensure!(cut.nonterminal_acceptances.is_empty() && cut.accepted_work_ids == intent.accepted_work_ids_at_admission,
                    "ordinary Stop closeout has unresolved original accepted work");
                // This records already-acknowledged cleanup. It does not ask
                // for a fresh actor/send/Stop grant after the actual retirement.
                verify_native(None)?;
                if let Some(existing) = intent.acknowledgment.as_ref() {
                    anyhow::ensure!(existing == acknowledgment.as_ref(), "ordinary Stop closeout acknowledgment differs");
                    return Ok(PreparedOrdinaryWorkerStopV1 { record: intent.clone(), active_acceptance_record: None,
                        observed_current_caller: source.facts.current.caller.participant_id.clone(),
                        observed_host_posture: source.facts.current.authority.lifecycle_posture });
                }
                let (_, key) = load_committed_admission_key(selection.authority_store_id(), &registry, &keys)
                    .map_err(|error| anyhow::anyhow!("{error}"))?;
                let completed_at = TimestampV1::parse(chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Nanos, true))?;
                let mut nonce = [0u8; 16]; rand::rngs::OsRng.fill_bytes(&mut nonce);
                let temp_name = format!("admission-registry--{}.tmp", lower_hex(&nonce));
                let storage = super::super::host_session_authority::store::retained_worker_admission_storage_existing_for_authority(selection.held_authority())
                    .map_err(|error| anyhow::anyhow!("{error:?}"))?;
                let mut semantic_failure = None;
                let result = storage.transaction_existing_control(&|| verify_native(None)
                    .map_err(|_| super::super::host_session_authority::store::BootstrapError::ordinary_control_source_guard()), |transaction| {
                    if transaction.read_registry()?.as_deref() != Some(source.admission_registry_bytes()) {
                        return Err(super::super::host_session_authority::store::BootstrapError::ordinary_control_source_guard());
                    }
                    let mut candidate = registry;
                    let mut stops = candidate.ordinary_stops_v1.take()
                        .ok_or_else(super::super::host_session_authority::store::BootstrapError::ordinary_control_source_guard)?;
                    let (record, changed) = retain_semantic_error(stops.complete(acknowledgment.as_ref().clone(),
                        &cut.accepted_work_ids, completed_at, &key), &mut semantic_failure)?;
                    candidate.ordinary_stops_v1 = Some(stops);
                    if !changed { return Ok(record); }
                    let bytes = retain_semantic_error(encode_canonical(&candidate, "encode ordinary Stop closed lifecycle"), &mut semantic_failure)?;
                    retain_semantic_error(decode_held_retained_admission_source(source.snapshot(), Some(&bytes), &keys,
                        &request.orchestration_session_id, &world), &mut semantic_failure)?;
                    transaction.replace_registry_with_source_guard(&temp_name, &bytes, &|| verify_native(Some(&temp_name))
                        .map_err(|_| super::super::host_session_authority::store::BootstrapError::ordinary_control_source_guard()))?;
                    retain_semantic_error(record_publication(&bytes).map_err(|error| RetainedWorkerRuntimeError(error.to_string())), &mut semantic_failure)?;
                    Ok(record)
                });
                let record = result.map_err(|error| anyhow::anyhow!("{}", semantic_failure.unwrap_or_else(||
                    RetainedWorkerRuntimeError(format!("ordinary Stop closeout publication unresolved: {error:?}")))))?;
                Ok(PreparedOrdinaryWorkerStopV1 { record, active_acceptance_record: None,
                    observed_current_caller: source.facts.current.caller.participant_id.clone(),
                    observed_host_posture: source.facts.current.authority.lifecycle_posture })
            }).map_err(|error| RetainedWorkerRuntimeError(error.to_string()))
    }
}

// Independent service-side operation grant. The surrounding owner retains the
// original lifecycle sources and joins its actual accepted native runtime.
#[cfg(target_os = "linux")]
pub(crate) fn authorize_ordinary_stop_cancel_from_source_v1(
    snapshot: &super::super::host_session_authority::store::AcceptedWorkAuthorityPhysicalSnapshotV1,
    admission_bytes: &[u8],
    keys: &[(String, Vec<u8>)],
    supervisor_bytes: Option<&[u8]>,
    request: &OrdinaryRetainedWorkerStopRequestV1,
    acceptance: &super::super::state_store::WorldWorkAcceptanceRecordV1,
) -> anyhow::Result<super::super::dispatch_contract::ValidatedWorldDispatchRequestV1> {
    use super::super::dispatch_contract::*;
    let world = WorldBindingV1 {
        world_id: request.world_id.clone(),
        world_generation: request.world_generation,
    };
    let (_, registry) = decode_held_retained_admission_source(
        snapshot,
        Some(admission_bytes),
        keys,
        &request.orchestration_session_id,
        &world,
    )
    .map_err(|error| anyhow::anyhow!("{error}"))?;
    let registry =
        registry.ok_or_else(|| anyhow::anyhow!("ordinary Stop cancellation admission absent"))?;
    let intent = registry
        .ordinary_stops_v1
        .as_ref()
        .and_then(|stops| {
            stops.record(
                &request.orchestration_session_id,
                &request.target_participant_id,
            )
        })
        .ok_or_else(|| {
            anyhow::anyhow!("ordinary Stop cancellation has no original admitted intent")
        })?;
    anyhow::ensure!(
        intent.request == *request
            && intent.acknowledgment.is_none()
            && intent
                .accepted_work_ids_at_admission
                .contains(&acceptance.acceptance_record_id),
        "ordinary Stop cancellation changed original admitted work"
    );
    let cut = checked_ordinary_stop_accepted_cut_from_sources(
        snapshot,
        supervisor_bytes,
        request,
        &acceptance.caller_backend_id,
    )?;
    anyhow::ensure!(
        cut.accepted_work_ids == intent.accepted_work_ids_at_admission,
        "ordinary Stop cancellation original work cut changed"
    );
    WorldDispatchRequestV1 {
        request_id: Some(request.stop_request_id.clone()),
        idempotency_key: Some(intent.idempotency_key.clone()),
        orchestration_session_id: Some(request.orchestration_session_id.clone()),
        caller_participant_id: Some(request.caller_participant_id.clone()),
        action: WorldDispatchActionV1::StopWorldWorker,
        mode: WorldDispatchModeV1::Retained,
        target_backend_id: Some(request.target_backend_id.clone()),
        task_run_id: None,
        target_participant_id: Some(request.target_participant_id.clone()),
        world_id: Some(request.world_id.clone()),
        world_generation: Some(request.world_generation),
        dispatch_policy_narrowing: None,
        payload: WorldDispatchPayloadV1::WorkerStop(WorkerStopPayloadV1::default()),
    }
    .validate()
}

#[cfg(target_os = "linux")]
impl RetainedWorkerRuntime {
    /// Continue cleanup of an already-published exact intent. Its MACed grant,
    /// rather than the current actor or historical B1 alone, authorizes this
    /// continuation. Fresh Stop policy and all original physical sources apply.
    pub(crate) fn resume_ordinary_stop_intent_v1(
        &self,
        selection: &super::super::native_launch_context::AuthenticatedOrdinarySelectionV1,
        prepared: &PreparedOrdinaryWorkerStopV1,
    ) -> Result<PreparedOrdinaryWorkerStopV1, RetainedWorkerRuntimeError> {
        selection.with_ordinary_lifecycle_control_sources_v1(&prepared.request().target_participant_id,
            |context, source, verify_native, _| {
                let request = prepared.request();
                let world = WorldBindingV1 { world_id: request.world_id.clone(), world_generation: request.world_generation };
                let keys = source.admission_key_files();
                let (_, registry) = decode_held_retained_admission_source(source.snapshot(), Some(source.admission_registry_bytes()),
                    &keys, &request.orchestration_session_id, &world).map_err(|error| anyhow::anyhow!("{error}"))?;
                let registry = registry.ok_or_else(|| anyhow::anyhow!("ordinary Stop continuation admission absent"))?;
                let intent = registry.ordinary_stops_v1.as_ref().and_then(|stops|
                    stops.record(&request.orchestration_session_id, &request.target_participant_id))
                    .ok_or_else(|| anyhow::anyhow!("ordinary Stop continuation original intent absent"))?;
                anyhow::ensure!(intent.request == prepared.record.request && intent.bootstrap == prepared.record.bootstrap
                    && intent.idempotency_key == prepared.record.idempotency_key && intent.admitted_at == prepared.record.admitted_at
                    && intent.accepted_work_ids_at_admission == prepared.record.accepted_work_ids_at_admission
                    && intent.bootstrap.completion().source_start
                        == crate::execution::orchestrator_world_dispatch::ordinary_start_selection_carrier(selection)?,
                    "ordinary Stop continuation changed immutable admitted grant");
                let row = read_ordinary_retained_worker_from_sources_v1(source.snapshot(), Some(source.admission_registry_bytes()),
                    &keys, &request.orchestration_session_id, &world, &request.target_participant_id, prepared.source_start())
                    .map_err(|error| anyhow::anyhow!("{error}"))?;
                let graph = source.target_graph.as_ref().ok_or_else(|| anyhow::anyhow!("ordinary Stop continuation original R0 absent"))?;
                row.validate_inspected_graph(graph).map_err(|error| anyhow::anyhow!("{error}"))?;
                anyhow::ensure!(graph.registration.registration_id == request.registration_id
                    && graph.descriptor.agent_id == context.selections.requested.realized_agent_id,
                    "ordinary Stop continuation requested-role R0 changed");
                let cut = checked_ordinary_stop_accepted_cut(source, request, &context.selections.native.backend_id)?;
                anyhow::ensure!(cut.accepted_work_ids == intent.accepted_work_ids_at_admission
                    && cut.nonterminal_acceptances.len() <= 1
                    && (intent.acknowledgment.is_none() || cut.nonterminal_acceptances.is_empty()),
                    "ordinary Stop continuation original work cut changed");
                verify_native(None)?;
                let policy = crate::execution::policy_model::resolve_effective_policy_for_bootstrap_home(
                    std::path::Path::new(&context.workspace_binding.workspace_root.physical_path), &selection.held_authority().bootstrap_home())?;
                crate::execution::orchestrator_world_dispatch::enforce_world_dispatch_read_or_effect_policy(&prepared.policy_request()?, &policy)?;
                verify_native(None)?;
                Ok(PreparedOrdinaryWorkerStopV1 { record: intent.clone(), active_acceptance_record: cut.nonterminal_acceptances.into_iter().next(),
                    observed_current_caller: source.facts.current.caller.participant_id.clone(), observed_host_posture: source.facts.current.authority.lifecycle_posture })
            }).map_err(|error| RetainedWorkerRuntimeError(error.to_string()))
    }
}

/// Read only the separately MACed logical intent/ACK under the surrounding
/// original held source capture. No lifecycle transition or native inference.
#[cfg(target_os = "linux")]
pub(crate) fn read_ordinary_retained_stop_from_sources_v1(
    snapshot: &super::super::host_session_authority::store::AcceptedWorkAuthorityPhysicalSnapshotV1,
    admission: Option<&[u8]>,
    keys: &[(String, Vec<u8>)],
    session: &str,
    world: &WorldBindingV1,
    target: &str,
    start: &transport_api_types::OrdinaryStartSelectionCarrierV1,
) -> Result<
    Option<Box<super::super::dispatch_contract::RetainedStopInspectSnapshotV1>>,
    RetainedWorkerRuntimeError,
> {
    let (_, registry) =
        decode_held_retained_admission_source(snapshot, admission, keys, session, world)?;
    let Some(intent) = registry
        .as_ref()
        .and_then(|registry| registry.ordinary_stops_v1.as_ref())
        .and_then(|stops| stops.record(session, target))
    else {
        return Ok(None);
    };
    if &intent.bootstrap.completion().source_start != start {
        return Err(conflict());
    }
    intent.validate()?;
    Ok(Some(Box::new(
        super::super::dispatch_contract::RetainedStopInspectSnapshotV1 {
            schema_version: 1,
            request: intent.request.clone(),
            accepted_work_ids_at_admission: intent.accepted_work_ids_at_admission.clone(),
            admitted_at: intent.admitted_at.clone(),
            acknowledgment: intent.acknowledgment.clone().map(Box::new),
            completed_at: intent.completed_at.clone(),
        },
    )))
}
