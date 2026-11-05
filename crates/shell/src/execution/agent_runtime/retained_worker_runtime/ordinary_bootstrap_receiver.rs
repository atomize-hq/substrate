//! Durable one-time reception of an original retained bootstrap. A consumed
//! receiver is not successful execution or retirement, and is never retried.
use super::*;
use transport_api_types::{
    E2MemberLaunchActivationCarrierV1, E2MemberLaunchKindV1,
    OrdinaryInitialBootstrapControlOperationV1, OrdinaryInitialBootstrapControlRequestV1,
    OrdinaryStartSelectionCarrierV1,
};

const DOMAIN: &str = "substrate.retained-worker.ordinary-bootstrap-receiver.v1";

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct OrdinaryBootstrapReceiverRegistryV1 {
    body: ReceiverBodyV1,
    mac_hex: String,
}
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
struct ReceiverBodyV1 {
    schema_version: u32,
    domain: String,
    authority_store_id: String,
    key_id: String,
    #[serde(with = "records_codec")]
    by_activation: BTreeMap<String, ReceiverRecordV1>,
    #[serde(
        default,
        skip_serializing_if = "BTreeMap::is_empty",
        with = "pending_controls_codec"
    )]
    pending_controls_v1: BTreeMap<String, Vec<PendingInitialControlV1>>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
struct PendingInitialControlV1 {
    original_start: OrdinaryStartSelectionCarrierV1,
    activation: E2MemberLaunchActivationCarrierV1,
    original_sender_claim_id: String,
    control_request_id: String,
    control: OrdinaryInitialBootstrapControlOperationV1,
    signal: String,
}

/// Admitted Source intent only; it cannot identify a Root receiver or attest
/// delivery, SDK admission, process absence, retirement or capacity release.
pub(crate) struct InitialBootstrapPendingControlReceiptV1 {
    pub(crate) control_request_id: String,
    pub(crate) target_participant_id: String,
}

mod pending_controls_codec {
    use super::*;
    type Records = BTreeMap<String, Vec<PendingInitialControlV1>>;
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
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
struct ReceiverRecordV1 {
    original_start: OrdinaryStartSelectionCarrierV1,
    activation: E2MemberLaunchActivationCarrierV1,
    original_sender_claim_id: String,
    execute_sha256: String,
    receiver_claim_id: String,
    service_span_id: String,
    consumed_at: TimestampV1,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    controls_v1: Vec<OrdinaryInitialBootstrapControlRequestV1>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    retirement_v1: Option<transport_api_types::OrdinaryInitialBootstrapRetirementV1>,
}

/// Copied original source identity only. No Execute input, native runtime,
/// gateway, backend, history-use or exclusion owner is retained here.
#[derive(Clone)]
pub(crate) struct OrdinaryInitialBootstrapReceiverScopeV1 {
    original_start: OrdinaryStartSelectionCarrierV1,
    activation: E2MemberLaunchActivationCarrierV1,
    original_sender_claim_id: String,
    execute_sha256: String,
    receiver_claim_id: String,
    service_span_id: String,
}

impl OrdinaryInitialBootstrapReceiverScopeV1 {
    pub(crate) fn build_report(
        &self,
        retirement_id: String,
        native_processes: transport_api_types::OrdinaryInitialBootstrapProcessCloseoutV1,
        gateway_disposal: transport_api_types::OrdinaryGatewayDisposalV1,
        frame_identity: transport_api_types::RuntimeFrameIdentityV1,
        event_identity: transport_api_types::RuntimeEventIdentityV1,
    ) -> anyhow::Result<transport_api_types::OrdinaryInitialBootstrapRetirementV1> {
        transport_api_types::OrdinaryInitialBootstrapRetirementV1::from_observation(
            transport_api_types::OrdinaryInitialBootstrapRetirementObservationV1 {
                source_start: self.original_start.clone(),
                activation: self.activation.clone(),
                original_sender_claim_id: self.original_sender_claim_id.clone(),
                execute_sha256: self.execute_sha256.clone(),
                receiver_claim_id: self.receiver_claim_id.clone(),
                service_span_id: self.service_span_id.clone(),
                retirement_id,
                native_processes,
                gateway_disposal,
                frame_identity,
                event_identity,
            },
        )
        .map_err(anyhow::Error::msg)
    }
}

fn matches_receiver(
    control: &OrdinaryInitialBootstrapControlRequestV1,
    receiver: &ReceiverRecordV1,
) -> bool {
    control.source_start == receiver.original_start
        && control.activation == receiver.activation
        && control.original_sender_claim_id == receiver.original_sender_claim_id
        && control.execute_sha256 == receiver.execute_sha256
        && control.receiver_claim_id == receiver.receiver_claim_id
        && control.service_span_id == receiver.service_span_id
}
mod records_codec {
    use super::*;
    type Records = BTreeMap<String, ReceiverRecordV1>;
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
    RetainedWorkerRuntimeError(
        "ordinary bootstrap receiver already consumed or source conflicts".into(),
    )
}
impl OrdinaryBootstrapReceiverRegistryV1 {
    pub(super) fn initial_stop_grant_for(
        &self,
        session: &str,
        participant: &str,
        start: &OrdinaryStartSelectionCarrierV1,
    ) -> Result<Option<&OrdinaryInitialBootstrapControlRequestV1>, RetainedWorkerRuntimeError> {
        let mut receivers = self.body.by_activation.values().filter(|receiver| {
            receiver.activation.orchestration_session_id == session
                && receiver.activation.retained_participant_id == participant
        });
        let Some(receiver) = receivers.next() else {
            return Ok(None);
        };
        if receivers.next().is_some() || &receiver.original_start != start {
            return Err(conflict());
        }
        Ok(receiver.controls_v1.iter().find(|control| {
            matches!(
                control.control,
                OrdinaryInitialBootstrapControlOperationV1::StopBootstrap {}
            )
        }))
    }

    pub(super) fn retirement_for(
        &self,
        session: &str,
        participant: &str,
    ) -> Option<&transport_api_types::OrdinaryInitialBootstrapRetirementV1> {
        self.body
            .by_activation
            .values()
            .find(|record| {
                record.activation.orchestration_session_id == session
                    && record.activation.retained_participant_id == participant
            })
            .and_then(|record| record.retirement_v1.as_ref())
    }

    pub(super) fn retirement_counts(&self) -> (u64, u64) {
        self.body
            .by_activation
            .values()
            .filter(|record| record.retirement_v1.is_some())
            .fold((0, 0), |(spawn, fork), record| {
                match record.activation.launch_kind {
                    E2MemberLaunchKindV1::FreshSpawn => (spawn + 1, fork),
                    E2MemberLaunchKindV1::Fork => (spawn, fork + 1),
                }
            })
    }

    pub(super) fn retired_activations(
        &self,
        session: &str,
    ) -> Vec<E2MemberLaunchActivationCarrierV1> {
        self.body
            .by_activation
            .values()
            .filter(|record| {
                record.activation.orchestration_session_id == session
                    && record.retirement_v1.is_some()
            })
            .map(|record| record.activation.clone())
            .collect()
    }

    pub(super) fn has_requested_initial_stop(&self, session: &str, participant: &str) -> bool {
        self.initial_stop_targets(session)
            .into_iter()
            .any(|target| target == participant)
    }

    pub(super) fn initial_stop_targets(&self, session: &str) -> Vec<&str> {
        let received = self
            .body
            .by_activation
            .values()
            .filter(|record| {
                record.activation.orchestration_session_id == session
                    && record.controls_v1.iter().any(|control| {
                        matches!(
                            control.control,
                            OrdinaryInitialBootstrapControlOperationV1::StopBootstrap {}
                        )
                    })
            })
            .map(|record| record.activation.retained_participant_id.as_str());
        let pending = self
            .body
            .pending_controls_v1
            .values()
            .flatten()
            .filter(|control| {
                control.activation.orchestration_session_id == session
                    && matches!(
                        control.control,
                        OrdinaryInitialBootstrapControlOperationV1::StopBootstrap {}
                    )
            })
            .map(|control| control.activation.retained_participant_id.as_str());
        received
            .chain(pending)
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect()
    }

    #[cfg(target_os = "linux")]
    pub(super) fn validate_population(
        &self,
        population: &super::super::dispatch_policy_commitment::OrdinaryBootstrapPopulationV1,
    ) -> Result<(), RetainedWorkerRuntimeError> {
        if self
            .body
            .by_activation
            .values()
            .map(|record| &record.activation)
            .chain(
                self.body
                    .pending_controls_v1
                    .values()
                    .flatten()
                    .map(|control| &control.activation),
            )
            .any(|activation| !population.contains_activation(activation))
        {
            return Err(conflict());
        }
        Ok(())
    }

    pub(super) fn initial_control_observation_for(
        &self,
        session: &str,
        participant: &str,
        start: &OrdinaryStartSelectionCarrierV1,
    ) -> Result<
        Option<super::super::dispatch_contract::RetainedInitialControlInspectSnapshotV1>,
        RetainedWorkerRuntimeError,
    > {
        let mut records = self.body.by_activation.values().filter(|record| {
            record.activation.orchestration_session_id == session
                && record.activation.retained_participant_id == participant
        });
        let Some(record) = records.next() else {
            let pending = self
                .body
                .pending_controls_v1
                .values()
                .flatten()
                .filter(|control| {
                    control.activation.orchestration_session_id == session
                        && control.activation.retained_participant_id == participant
                })
                .collect::<Vec<_>>();
            if pending.is_empty() {
                return Ok(None);
            }
            if pending
                .iter()
                .any(|control| &control.original_start != start)
            {
                return Err(conflict());
            }
            let id_for = |operation| {
                pending
                    .iter()
                    .find(|control| control.control == operation)
                    .map(|control| control.control_request_id.clone())
            };
            return Ok(Some(
                super::super::dispatch_contract::RetainedInitialControlInspectSnapshotV1 {
                    cancel_request_id: id_for(
                        OrdinaryInitialBootstrapControlOperationV1::CancelBootstrap {},
                    ),
                    stop_request_id: id_for(
                        OrdinaryInitialBootstrapControlOperationV1::StopBootstrap {},
                    ),
                    retirement_id: None,
                },
            ));
        };
        if records.next().is_some() || &record.original_start != start {
            return Err(conflict());
        }
        if record.controls_v1.is_empty() && record.retirement_v1.is_none() {
            return Ok(None);
        }
        let id_for = |operation| {
            record
                .controls_v1
                .iter()
                .find(|control| control.control == operation)
                .map(|control| control.control_request_id.clone())
        };
        Ok(Some(
            super::super::dispatch_contract::RetainedInitialControlInspectSnapshotV1 {
                cancel_request_id: id_for(
                    OrdinaryInitialBootstrapControlOperationV1::CancelBootstrap {},
                ),
                stop_request_id: id_for(
                    OrdinaryInitialBootstrapControlOperationV1::StopBootstrap {},
                ),
                retirement_id: record
                    .retirement_v1
                    .as_ref()
                    .map(|report| report.observation().retirement_id.clone()),
            },
        ))
    }

    pub(super) fn has_consumed_receiver_for(&self, session: &str, participant: &str) -> bool {
        self.body.by_activation.values().any(|record| {
            record.activation.orchestration_session_id == session
                && record.activation.retained_participant_id == participant
        })
    }

    fn new(key: &RetainedWorkerAdmissionKeyEnvelopeV1) -> Self {
        Self {
            body: ReceiverBodyV1 {
                schema_version: 1,
                domain: DOMAIN.into(),
                authority_store_id: key.authority_store_id.clone(),
                key_id: key.key_id.clone(),
                by_activation: BTreeMap::new(),
                pending_controls_v1: BTreeMap::new(),
            },
            mac_hex: String::new(),
        }
    }
    fn expected_mac(
        &self,
        key: &RetainedWorkerAdmissionKeyEnvelopeV1,
    ) -> Result<String, RetainedWorkerRuntimeError> {
        let bytes = encode_canonical(&self.body, "encode bootstrap receiver MAC input")?;
        let mut input =
            b"substrate.retained-worker.ordinary-bootstrap-receiver.mac-input.v1\0".to_vec();
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
            || (self.body.by_activation.is_empty() && self.body.pending_controls_v1.is_empty())
            || !is_lower_hex(&self.mac_hex, 64)
        {
            return Err(conflict());
        }
        let expected = self.expected_mac(key)?;
        if expected
            .bytes()
            .zip(self.mac_hex.bytes())
            .fold(0u8, |difference, (a, b)| difference | (a ^ b))
            != 0
        {
            return Err(conflict());
        }
        Ok(())
    }
    pub(super) fn validate_registry_joins(
        &self,
        registry: &RetainedWorkerAdmissionRegistryV1,
    ) -> Result<(), RetainedWorkerRuntimeError> {
        if self.body.authority_store_id != registry.authority_store_id
            || self.body.key_id != registry.commitment_key.key_id
            || (self.body.by_activation.is_empty() && self.body.pending_controls_v1.is_empty())
        {
            return Err(conflict());
        }
        let mut receivers = BTreeSet::new();
        let mut spans = BTreeSet::new();
        for (id, controls) in &self.body.pending_controls_v1 {
            if controls.is_empty() || controls.len() > 2 {
                return Err(conflict());
            }
            let mut operations = BTreeSet::new();
            for control in controls {
                control.original_start.validate().map_err(|_| conflict())?;
                control.activation.validate().map_err(|_| conflict())?;
                let a = &control.activation;
                if id != &a.activation_id
                    || control.original_start.authority_store_id != registry.authority_store_id
                    || control.original_start.original_session_id != a.orchestration_session_id
                    || a.commitment_ref.authority_store_id != registry.authority_store_id
                    || control.control_request_id.is_empty()
                    || control.control_request_id.trim() != control.control_request_id
                    || !matches!(control.signal.as_str(), "INT" | "TERM")
                    || !operations.insert(&control.control)
                    || sender_claim(registry, &control.original_start, a)?.as_ref()
                        != Some(&control.original_sender_claim_id)
                {
                    return Err(conflict());
                }
                if let Some(receiver) = self.body.by_activation.get(id) {
                    if receiver.original_start != control.original_start
                        || receiver.activation != *a
                        || receiver.original_sender_claim_id != control.original_sender_claim_id
                        || !receiver.controls_v1.iter().any(|received| {
                            received.control == control.control
                                && received.control_request_id == control.control_request_id
                                && received.signal == control.signal
                        })
                    {
                        return Err(conflict());
                    }
                }
            }
        }
        for (id, record) in &self.body.by_activation {
            record.original_start.validate().map_err(|_| conflict())?;
            record.activation.validate().map_err(|_| conflict())?;
            let a = &record.activation;
            if let Some(report) = &record.retirement_v1 {
                let observed = report.observation();
                transport_api_types::OrdinaryInitialBootstrapRetirementV1::from_observation(
                    observed.clone(),
                )
                .map_err(|_| conflict())?;
                if observed.source_start != record.original_start
                    || observed.activation != *a
                    || observed.original_sender_claim_id != record.original_sender_claim_id
                    || observed.execute_sha256 != record.execute_sha256
                    || observed.receiver_claim_id != record.receiver_claim_id
                    || observed.service_span_id != record.service_span_id
                    || registry
                        .native_lifecycle_v1
                        .as_ref()
                        .and_then(|lifecycle| {
                            lifecycle.native_completion_for(
                                &a.orchestration_session_id,
                                &a.retained_participant_id,
                            )
                        })
                        .is_some()
                    || registry
                        .ordinary_forks_v1
                        .as_ref()
                        .and_then(|forks| {
                            forks.inspection_record(
                                &a.orchestration_session_id,
                                &a.retained_participant_id,
                            )
                        })
                        .is_some_and(|(_, completed)| completed)
                {
                    return Err(conflict());
                }
            }
            let mut operations = BTreeSet::new();
            if record.controls_v1.len() > 2 {
                return Err(conflict());
            }
            for control in &record.controls_v1 {
                control.validate().map_err(|_| conflict())?;
                if !matches_receiver(control, record) || !operations.insert(&control.control) {
                    return Err(conflict());
                }
            }
            let parsed = record
                .receiver_claim_id
                .strip_prefix("obr_")
                .and_then(|value| uuid::Uuid::parse_str(value).ok());
            if id != &a.activation_id
                || a.commitment_ref.authority_store_id != registry.authority_store_id
                || record.original_start.authority_store_id != registry.authority_store_id
                || record.original_start.original_session_id != a.orchestration_session_id
                || !is_lower_hex(&record.execute_sha256, 64)
                || record.service_span_id.is_empty()
                || record.service_span_id.trim() != record.service_span_id
                || record.original_sender_claim_id.is_empty()
                || parsed.is_none_or(|value| {
                    value.get_version_num() != 7
                        || record.receiver_claim_id != format!("obr_{value}")
                })
                || !receivers.insert(&record.receiver_claim_id)
                || !spans.insert(&record.service_span_id)
                || sender_claim(registry, &record.original_start, a)?
                    .as_ref()
                    .is_some_and(|id| id != &record.original_sender_claim_id)
            {
                return Err(conflict());
            }
        }
        Ok(())
    }
}

fn sender_claim(
    registry: &RetainedWorkerAdmissionRegistryV1,
    start: &OrdinaryStartSelectionCarrierV1,
    activation: &E2MemberLaunchActivationCarrierV1,
) -> Result<Option<String>, RetainedWorkerRuntimeError> {
    match activation.launch_kind {
        E2MemberLaunchKindV1::FreshSpawn => {
            let record = registry
                .records_by_session
                .get(&activation.orchestration_session_id)
                .and_then(|records| records.get(&activation.retained_participant_id))
                .ok_or_else(conflict)?;
            if record.bootstrap_run_id != activation.bootstrap_run_id
                || record.backend_id != activation.target_backend_id
                || record.world_binding.world_id != activation.target_world.world_id
                || record.world_binding.world_generation != activation.target_world.world_generation
            {
                return Err(conflict());
            }
            if record.bootstrap_receiver_protocol != Some(1) {
                return Err(conflict());
            }
            // Terminal source records omit the old sender claim. Its already
            // authenticated original value stays in the receiver MAC; do not
            // infer a new claim from that omission.
            match &record.state {
                RetainedWorkerAdmissionStateV1::TransportClaimedNonterminal {
                    transport_claim_id,
                    ..
                } => Ok(Some(transport_claim_id.clone())),
                RetainedWorkerAdmissionStateV1::Routable {
                    transport_claim_id: Some(id),
                    ..
                }
                | RetainedWorkerAdmissionStateV1::InterruptedNonterminal {
                    transport_claim_id: Some(id),
                    ..
                }
                | RetainedWorkerAdmissionStateV1::CancellationAcceptedTransportCloseoutPending {
                    transport_claim_id: Some(id),
                    ..
                } => Ok(Some(id.clone())),
                RetainedWorkerAdmissionStateV1::Terminal { .. } => Ok(None),
                _ => Err(conflict()),
            }
        }
        E2MemberLaunchKindV1::Fork => {
            let (record, _) = registry
                .ordinary_forks_v1
                .as_ref()
                .and_then(|forks| {
                    forks.inspection_record(
                        &activation.orchestration_session_id,
                        &activation.retained_participant_id,
                    )
                })
                .ok_or_else(conflict)?;
            if record.activation != *activation || record.start != *start {
                return Err(conflict());
            }
            Ok(Some(record.native_claim_id))
        }
    }
}

#[cfg(target_os = "linux")]
fn initial_control_policy_request(
    selection: &super::super::native_launch_context::AuthenticatedOrdinarySelectionV1,
    context: &super::super::native_launch_context::NativeLaunchContextV1,
    source: &super::super::host_session_authority::facade::OrdinaryRetainedLifecycleControlReadSourceV1,
    wire: &OrdinaryInitialBootstrapControlRequestV1,
    require_current_actor: bool,
) -> anyhow::Result<super::super::dispatch_contract::ValidatedWorldDispatchRequestV1> {
    initial_control_policy_request_from_source(
        selection,
        context,
        source,
        InitialControlPolicySourceV1 {
            start: &wire.source_start,
            activation: &wire.activation,
            control: &wire.control,
            request_id: &wire.control_request_id,
        },
        require_current_actor,
    )
}

struct InitialControlPolicySourceV1<'a> {
    start: &'a OrdinaryStartSelectionCarrierV1,
    activation: &'a E2MemberLaunchActivationCarrierV1,
    control: &'a OrdinaryInitialBootstrapControlOperationV1,
    request_id: &'a str,
}

#[cfg(target_os = "linux")]
fn initial_control_policy_request_from_source(
    selection: &super::super::native_launch_context::AuthenticatedOrdinarySelectionV1,
    context: &super::super::native_launch_context::NativeLaunchContextV1,
    source: &super::super::host_session_authority::facade::OrdinaryRetainedLifecycleControlReadSourceV1,
    control: InitialControlPolicySourceV1<'_>,
    require_current_actor: bool,
) -> anyhow::Result<super::super::dispatch_contract::ValidatedWorldDispatchRequestV1> {
    use super::super::dispatch_contract::*;
    let activation = control.activation;
    let current = &source.facts.current;
    anyhow::ensure!(
        control.start
            == &crate::execution::orchestrator_world_dispatch::ordinary_start_selection_carrier(
                selection
            )?
            && context.selections.native.placement
                == transport_api_types::AgentSelectionPlacementV1::Host
            && context.selections.requested.placement
                == transport_api_types::AgentSelectionPlacementV1::World
            && activation.orchestration_session_id == context.orchestration_session_id
            && activation.caller_backend_id == context.selections.native.backend_id
            && activation.target_backend_id == context.selections.requested.backend_id,
        "initial control changed original Start or configured role scope"
    );
    if require_current_actor {
        anyhow::ensure!(
            current.authority.lifecycle_posture == HostSessionPostureV1::ActiveAttached
                && current.caller.participant_id == activation.caller_participant_id
                && current.caller.participant_id == selection.current_participant_id()
                && current.caller.descriptor.execution_scope == AgentExecutionScopeV1::Host
                && current.caller.descriptor.backend_id == context.selections.native.backend_id
                && current.caller.descriptor.agent_id
                    == context.selections.native.realized_agent_id
                && current.caller.descriptor.protocol == context.descriptor.protocol
                && current.caller.descriptor.backend_kind == context.descriptor.backend_kind
                && current
                    .authority
                    .world_binding
                    .as_ref()
                    .is_some_and(|world| world.world_id == activation.target_world.world_id
                        && world.world_generation == activation.target_world.world_generation),
            "initial control current actor/World differs; history grants no transfer"
        );
    }
    let (action, payload) = match control.control {
        OrdinaryInitialBootstrapControlOperationV1::CancelBootstrap {} => (
            WorldDispatchActionV1::CancelWorldWork,
            WorldDispatchPayloadV1::WorkerCancel(WorkerCancelPayloadV1::default()),
        ),
        OrdinaryInitialBootstrapControlOperationV1::StopBootstrap {} => (
            WorldDispatchActionV1::StopWorldWorker,
            WorldDispatchPayloadV1::WorkerStop(WorkerStopPayloadV1::default()),
        ),
    };
    WorldDispatchRequestV1 {
        request_id: Some(control.request_id.to_owned()),
        idempotency_key: Some(control.request_id.to_owned()),
        orchestration_session_id: Some(activation.orchestration_session_id.clone()),
        caller_participant_id: Some(activation.caller_participant_id.clone()),
        action,
        mode: WorldDispatchModeV1::Retained,
        target_backend_id: Some(activation.target_backend_id.clone()),
        task_run_id: None,
        target_participant_id: Some(activation.retained_participant_id.clone()),
        world_id: Some(activation.target_world.world_id.clone()),
        world_generation: Some(activation.target_world.world_generation),
        dispatch_policy_narrowing: None,
        payload,
    }
    .validate()
}

#[cfg(target_os = "linux")]
impl RetainedWorkerRuntime {
    pub(crate) fn read_initial_bootstrap_controls_for_owner_v1(
        &self,
        selection: &super::super::native_launch_context::AuthenticatedOrdinarySelectionV1,
        actual_request: &transport_api_types::ExecuteRequest,
        span_id: &str,
    ) -> anyhow::Result<Vec<OrdinaryInitialBootstrapControlRequestV1>> {
        let scope =
            self.read_initial_bootstrap_receiver_scope_v1(selection, actual_request, span_id)?;
        selection.with_ordinary_lifecycle_control_sources_v1(
            &scope.activation.retained_participant_id,
            |_, source, verify, _| {
                let world = WorldBindingV1 {
                    world_id: scope.activation.target_world.world_id.clone(),
                    world_generation: scope.activation.target_world.world_generation,
                };
                let (_, registry) = decode_held_retained_admission_source(
                    source.snapshot(),
                    Some(source.admission_registry_bytes()),
                    &source.admission_key_files(),
                    &scope.activation.orchestration_session_id,
                    &world,
                )
                .map_err(|error| anyhow::anyhow!("{error}"))?;
                let registry = registry.ok_or_else(|| {
                    anyhow::anyhow!("stored initial control original registry absent")
                })?;
                let receiver = registry
                    .ordinary_bootstrap_receivers_v1
                    .as_ref()
                    .and_then(|receivers| {
                        receivers
                            .body
                            .by_activation
                            .get(&scope.activation.activation_id)
                    })
                    .ok_or_else(|| {
                        anyhow::anyhow!("stored initial control actual receiver absent")
                    })?;
                anyhow::ensure!(
                    receiver.original_start == scope.original_start
                        && receiver.activation == scope.activation
                        && receiver.original_sender_claim_id == scope.original_sender_claim_id
                        && receiver.execute_sha256 == scope.execute_sha256
                        && receiver.receiver_claim_id == scope.receiver_claim_id
                        && receiver.service_span_id == scope.service_span_id,
                    "stored initial control original receiver changed"
                );
                verify(None)?;
                Ok(receiver.controls_v1.clone())
            },
        )
    }

    pub(crate) fn prepare_initial_bootstrap_control_before_receiver_v1(
        &self,
        selection: &super::super::native_launch_context::AuthenticatedOrdinarySelectionV1,
        request: &super::super::dispatch_contract::ValidatedWorldDispatchRequestV1,
    ) -> anyhow::Result<Option<InitialBootstrapPendingControlReceiptV1>> {
        use super::super::dispatch_contract::*;
        if request.mode != WorldDispatchModeV1::Retained
            || !matches!(
                request.action,
                WorldDispatchActionV1::CancelWorldWork | WorldDispatchActionV1::StopWorldWorker
            )
        {
            return Ok(None);
        }
        anyhow::ensure!(
            request.dispatch_policy_narrowing.is_none(),
            "initial control cannot narrow original admission"
        );
        let exact = crate::execution::orchestrator_world_dispatch::resolve_orchestrator_world_dispatch_exact_target_v1(request)?;
        let target = match &exact {
            Some(WorldDispatchControlTargetV1::AcceptedRetainedTurn { .. }) => return Ok(None),
            Some(WorldDispatchControlTargetV1::PendingRetainedAdmission {
                target_participant_id,
                ..
            }) => target_participant_id.as_str(),
            None => request
                .target_participant_id
                .as_deref()
                .ok_or_else(|| anyhow::anyhow!("initial pending control target absent"))?,
        };
        selection.with_ordinary_lifecycle_control_sources_v1(target, |context, source, verify_sources, record_publication| {
            // The common control path reports a request naming another World.
            // This branch grants no initial-owner control or absence observation.
            if source.facts.current.authority.world_binding.as_ref().is_some_and(|world|
                world.world_id != request.world_id || world.world_generation != request.world_generation) {
                verify_sources(None)?;
                return Ok(None);
            }
            let world = WorldBindingV1 { world_id: request.world_id.clone(), world_generation: request.world_generation };
            let keys = source.admission_key_files();
            let (bootstraps, registry) = decode_held_retained_admission_source(source.snapshot(), Some(source.admission_registry_bytes()),
                &keys, &request.orchestration_session_id, &world).map_err(|error| anyhow::anyhow!("{error}"))?;
            let mut registry = registry.ok_or_else(|| anyhow::anyhow!("initial pending control original registry absent"))?;
            if bootstraps.completed_bootstraps.iter().any(|a| a.retained_participant_id == target)
                || registry.ordinary_bootstrap_receivers_v1.as_ref().is_some_and(|receivers|
                    receivers.body.by_activation.values().any(|receiver| receiver.activation.orchestration_session_id == request.orchestration_session_id
                        && receiver.activation.retained_participant_id == target)) { return Ok(None); }
            let population = super::super::dispatch_policy_commitment::decode_ordinary_bootstrap_population(
                source.snapshot().e2(), selection.authority_store_id())?;
            let Some(activation) = population.member_for(&request.orchestration_session_id, target).cloned() else { return Ok(None); };
            let start = crate::execution::orchestrator_world_dispatch::ordinary_start_selection_carrier(selection)?;
            if activation.launch_kind == E2MemberLaunchKindV1::Fork {
                anyhow::ensure!(registry.ordinary_forks_v1.as_ref().and_then(|forks|
                    forks.inspection_record(&activation.orchestration_session_id, &activation.retained_participant_id))
                    .is_some_and(|(record, _)| record.accepts_initial_receiver_v1()),
                    "initial pending control requires a fresh marked Fork sender claim");
            }
            // Unclaimed/incompatible input does not supply native absence or
            // justify fabricating a sender for a pending operation.
            let Some(sender) = sender_claim(&registry, &start, &activation).map_err(|error| anyhow::anyhow!("{error}"))? else { return Ok(None); };
            anyhow::ensure!(request.caller_participant_id == activation.caller_participant_id
                && request.target_backend_id == activation.target_backend_id && request.world_id == activation.target_world.world_id
                && request.world_generation == activation.target_world.world_generation, "initial pending control original role or World differs");
            if let Some(WorldDispatchControlTargetV1::PendingRetainedAdmission { issuer_request_id, .. }) = &exact {
                let issuer = match activation.launch_kind {
                    E2MemberLaunchKindV1::FreshSpawn => registry.records_by_session.get(&request.orchestration_session_id)
                        .and_then(|records| records.get(target)).map(|record| record.issuer_request_id.as_str()).ok_or_else(|| anyhow::anyhow!("initial pending issuer absent"))?,
                    E2MemberLaunchKindV1::Fork => activation.request_id.as_str(),
                };
                anyhow::ensure!(issuer_request_id == issuer, "initial pending exact issuer differs");
            }
            let operation = match request.action {
                WorldDispatchActionV1::CancelWorldWork => OrdinaryInitialBootstrapControlOperationV1::CancelBootstrap {},
                WorldDispatchActionV1::StopWorldWorker => OrdinaryInitialBootstrapControlOperationV1::StopBootstrap {},
                _ => unreachable!(),
            };
            let existing = registry.ordinary_bootstrap_receivers_v1.as_ref()
                .and_then(|receivers| receivers.body.pending_controls_v1.get(&activation.activation_id))
                .and_then(|controls| controls.iter().find(|control| control.control == operation)).cloned();
            let intent = existing.clone().unwrap_or_else(|| PendingInitialControlV1 {
                original_start: start.clone(), activation: activation.clone(), original_sender_claim_id: sender.clone(),
                control_request_id: request.request_id.clone(), control: operation, signal: "INT".into(),
            });
            anyhow::ensure!(intent.original_start == start && intent.activation == activation && intent.original_sender_claim_id == sender,
                "initial pending intent original source changed");
            let cleanup = existing.is_some() && matches!(intent.control, OrdinaryInitialBootstrapControlOperationV1::StopBootstrap {});
            let policy_request = initial_control_policy_request_from_source(selection, context, source,
                InitialControlPolicySourceV1 { start: &intent.original_start, activation: &intent.activation,
                    control: &intent.control, request_id: &intent.control_request_id }, !cleanup)?;
            let verify = |temp| -> anyhow::Result<()> {
                verify_sources(temp)?;
                let policy = crate::execution::policy_model::resolve_effective_policy_for_bootstrap_home(
                    std::path::Path::new(selection.workspace_root()), &selection.held_authority().bootstrap_home())?;
                crate::execution::orchestrator_world_dispatch::enforce_world_dispatch_read_or_effect_policy(&policy_request, &policy)
            };
            verify(None)?;
            let receipt = InitialBootstrapPendingControlReceiptV1 { control_request_id: intent.control_request_id.clone(),
                target_participant_id: target.to_owned() };
            if existing.is_some() { return Ok(Some(receipt)); }
            let (_, key) = load_committed_admission_key(selection.authority_store_id(), &registry, &keys).map_err(|error| anyhow::anyhow!("{error}"))?;
            let mut receivers = registry.ordinary_bootstrap_receivers_v1.take().unwrap_or_else(|| OrdinaryBootstrapReceiverRegistryV1::new(&key));
            let controls = receivers.body.pending_controls_v1.entry(activation.activation_id.clone()).or_default();
            controls.push(intent); controls.sort_by(|a, b| a.control.cmp(&b.control));
            receivers.mac_hex = receivers.expected_mac(&key).map_err(|error| anyhow::anyhow!("{error}"))?;
            registry.ordinary_bootstrap_receivers_v1 = Some(receivers);
            let bytes = encode_canonical(&registry, "encode initial pending control intent").map_err(|error| anyhow::anyhow!("{error}"))?;
            let storage = super::super::host_session_authority::store::retained_worker_admission_storage_existing_for_authority(selection.held_authority())
                .map_err(|error| anyhow::anyhow!("{error:?}"))?;
            let temp = format!("admission-registry--{}.tmp", uuid::Uuid::now_v7().simple());
            let guard_error = || super::super::host_session_authority::store::BootstrapError::ordinary_control_source_guard();
            storage.transaction_existing_control(&|| verify(None).map_err(|_| guard_error()), |transaction| {
                if transaction.read_registry()?.as_deref() != Some(source.admission_registry_bytes()) { return Err(guard_error()); }
                transaction.with_existing_e2_population_source(|physical, verify_e2| {
                    let population = super::super::dispatch_policy_commitment::decode_ordinary_bootstrap_population(physical, selection.authority_store_id()).map_err(|_| guard_error())?;
                    if !population.contains_activation(&activation) { return Err(guard_error()); }
                    validate_admission_e2_population(&registry, &population).map_err(|_| guard_error())?;
                    decode_held_retained_admission_source(source.snapshot(), Some(&bytes), &keys, &request.orchestration_session_id, &world).map_err(|_| guard_error())?;
                    transaction.replace_registry_with_source_guard(&temp, &bytes, &|| { verify_e2()?; verify(Some(&temp)).map_err(|_| guard_error()) })?;
                    record_publication(&bytes).map_err(|_| guard_error())
                })
            }).map_err(|error| anyhow::anyhow!("initial pending control publication refused: {error:?}"))?;
            Ok(Some(receipt))
        })
    }

    /// Read a previously authenticated closeout under its exact operation grant
    /// and fresh operation policy. The Host bit is a separate current fact, not
    /// an inference from one member's retirement or global World settlement.
    pub(crate) fn read_initial_bootstrap_retirement_for_control_v1(
        &self,
        selection: &super::super::native_launch_context::AuthenticatedOrdinarySelectionV1,
        request: &OrdinaryInitialBootstrapControlRequestV1,
        current_caller: &str,
    ) -> anyhow::Result<(
        Option<transport_api_types::OrdinaryInitialBootstrapRetirementV1>,
        bool,
    )> {
        request.validate().map_err(anyhow::Error::msg)?;
        anyhow::ensure!(
            crate::execution::orchestrator_world_dispatch::ordinary_start_selection_carrier(
                selection
            )? == request.source_start,
            "initial retirement read changed original Start"
        );
        selection.with_ordinary_lifecycle_control_sources_v1(&request.activation.retained_participant_id, |context, source, verify_source, _| {
            let world = WorldBindingV1 { world_id: request.activation.target_world.world_id.clone(), world_generation: request.activation.target_world.world_generation };
            let (_, registry) = decode_held_retained_admission_source(source.snapshot(), Some(source.admission_registry_bytes()),
                &source.admission_key_files(), &request.activation.orchestration_session_id, &world).map_err(|error| anyhow::anyhow!("{error}"))?;
            let registry = registry.ok_or_else(|| anyhow::anyhow!("initial retirement original registry absent"))?;
            let receiver = registry.ordinary_bootstrap_receivers_v1.as_ref().and_then(|receivers| receivers.body.by_activation.get(&request.activation.activation_id))
                .ok_or_else(|| anyhow::anyhow!("initial retirement receiver absent"))?;
            anyhow::ensure!(matches_receiver(request, receiver) && receiver.controls_v1.contains(request), "initial retirement read has no exact operation grant");
            let policy_request = initial_control_policy_request(selection, context, source, request,
                matches!(request.control, OrdinaryInitialBootstrapControlOperationV1::CancelBootstrap {}))?;
            let verify = || -> anyhow::Result<()> {
                verify_source(None)?;
                let policy = crate::execution::policy_model::resolve_effective_policy_for_bootstrap_home(
                    std::path::Path::new(selection.workspace_root()), &selection.held_authority().bootstrap_home())?;
                crate::execution::orchestrator_world_dispatch::enforce_world_dispatch_read_or_effect_policy(&policy_request, &policy)
            };
            verify()?;
            let report = receiver.retirement_v1.clone();
            let active_host = source.facts.current.authority.lifecycle_posture == HostSessionPostureV1::ActiveAttached
                && source.facts.current.caller.participant_id == current_caller;
            verify()?;
            Ok((report, active_host))
        })
    }

    pub(crate) fn read_initial_bootstrap_receiver_scope_v1(
        &self,
        selection: &super::super::native_launch_context::AuthenticatedOrdinarySelectionV1,
        actual_request: &transport_api_types::ExecuteRequest,
        span_id: &str,
    ) -> anyhow::Result<OrdinaryInitialBootstrapReceiverScopeV1> {
        actual_request.validate().map_err(anyhow::Error::msg)?;
        anyhow::ensure!(
            actual_request.acceptance_context.is_none(),
            "initial retained closeout cannot target ephemeral acceptance"
        );
        let activation = actual_request
            .member_dispatch
            .as_ref()
            .and_then(transport_api_types::MemberDispatchRequest::as_v1)
            .and_then(|member| member.e2_launch_activation.as_ref())
            .ok_or_else(|| anyhow::anyhow!("initial closeout original E2 missing"))?;
        let start =
            crate::execution::orchestrator_world_dispatch::ordinary_start_selection_carrier(
                selection,
            )?;
        let digest = lower_hex(&Sha256::digest(serde_json::to_vec(actual_request)?));
        anyhow::ensure!(
            actual_request.ordinary_start_selection.as_ref() == Some(&start),
            "initial closeout original configured Start differs"
        );
        selection.with_ordinary_lifecycle_control_sources_v1(
            &activation.retained_participant_id,
            |_, source, verify, _| {
                let world = WorldBindingV1 {
                    world_id: activation.target_world.world_id.clone(),
                    world_generation: activation.target_world.world_generation,
                };
                let (_, registry) = decode_held_retained_admission_source(
                    source.snapshot(),
                    Some(source.admission_registry_bytes()),
                    &source.admission_key_files(),
                    &activation.orchestration_session_id,
                    &world,
                )
                .map_err(|error| anyhow::anyhow!("{error}"))?;
                let registry = registry
                    .ok_or_else(|| anyhow::anyhow!("initial closeout original registry absent"))?;
                let receiver = registry
                    .ordinary_bootstrap_receivers_v1
                    .as_ref()
                    .and_then(|receivers| {
                        receivers.body.by_activation.get(&activation.activation_id)
                    })
                    .ok_or_else(|| {
                        anyhow::anyhow!("initial closeout actual consumed receiver absent")
                    })?;
                anyhow::ensure!(
                    receiver.original_start == start
                        && receiver.activation == *activation
                        && receiver.execute_sha256 == digest
                        && receiver.service_span_id == span_id,
                    "initial closeout differs from actual original Execute or receiver span"
                );
                verify(None)?;
                Ok(OrdinaryInitialBootstrapReceiverScopeV1 {
                    original_start: receiver.original_start.clone(),
                    activation: receiver.activation.clone(),
                    original_sender_claim_id: receiver.original_sender_claim_id.clone(),
                    execute_sha256: receiver.execute_sha256.clone(),
                    receiver_claim_id: receiver.receiver_claim_id.clone(),
                    service_span_id: receiver.service_span_id.clone(),
                })
            },
        )
    }

    /// Record data received from the authenticated original Root stream only
    /// after its actual owner reports checked initial retirement. This cleanup
    /// writer cannot issue a receiver, send input or reconstruct Registered/R0.
    pub(crate) fn record_initial_bootstrap_retirement_v1(
        &self,
        selection: &super::super::native_launch_context::AuthenticatedOrdinarySelectionV1,
        report: &transport_api_types::OrdinaryInitialBootstrapRetirementV1,
    ) -> anyhow::Result<()> {
        let observed = report.observation();
        let activation = &observed.activation;
        let start =
            crate::execution::orchestrator_world_dispatch::ordinary_start_selection_carrier(
                selection,
            )?;
        anyhow::ensure!(
            observed.source_start == start,
            "initial retirement changed original configured Start"
        );
        selection.with_ordinary_lifecycle_control_sources_v1(&activation.retained_participant_id, |_, source, verify, record_publication| {
            let world = WorldBindingV1 { world_id: activation.target_world.world_id.clone(), world_generation: activation.target_world.world_generation };
            let keys = source.admission_key_files();
            let (_, registry) = decode_held_retained_admission_source(source.snapshot(), Some(source.admission_registry_bytes()),
                &keys, &activation.orchestration_session_id, &world).map_err(|error| anyhow::anyhow!("{error}"))?;
            let mut registry = registry.ok_or_else(|| anyhow::anyhow!("initial retirement original registry absent"))?;
            let (_, key) = load_committed_admission_key(selection.authority_store_id(), &registry, &keys).map_err(|error| anyhow::anyhow!("{error}"))?;
            let receivers = registry.ordinary_bootstrap_receivers_v1.as_mut().ok_or_else(|| anyhow::anyhow!("initial retirement receiver absent"))?;
            let receiver = receivers.body.by_activation.get_mut(&activation.activation_id).ok_or_else(|| anyhow::anyhow!("initial retirement receiver absent"))?;
            anyhow::ensure!(observed.source_start == receiver.original_start && observed.activation == receiver.activation
                && observed.original_sender_claim_id == receiver.original_sender_claim_id
                && observed.execute_sha256 == receiver.execute_sha256 && observed.receiver_claim_id == receiver.receiver_claim_id
                && observed.service_span_id == receiver.service_span_id, "initial retirement differs from original consumed receiver");
            verify(None)?;
            if let Some(existing) = &receiver.retirement_v1 {
                anyhow::ensure!(existing == report, "initial retirement receipt conflicts");
                return Ok(());
            }
            receiver.retirement_v1 = Some(report.clone());
            receivers.mac_hex = receivers.expected_mac(&key).map_err(|error| anyhow::anyhow!("{error}"))?;
            let bytes = encode_canonical(&registry, "encode initial retirement").map_err(|error| anyhow::anyhow!("{error}"))?;
            let storage = super::super::host_session_authority::store::retained_worker_admission_storage_existing_for_authority(selection.held_authority())
                .map_err(|error| anyhow::anyhow!("{error:?}"))?;
            let temp = format!("admission-registry--{}.tmp", uuid::Uuid::now_v7().simple());
            let guard_error = || super::super::host_session_authority::store::BootstrapError::ordinary_control_source_guard();
            storage.transaction_existing_control(&|| verify(None).map_err(|_| guard_error()), |transaction| {
                if transaction.read_registry()?.as_deref() != Some(source.admission_registry_bytes()) { return Err(guard_error()); }
                transaction.with_existing_e2_population_source(|physical, verify_e2| {
                    let population = super::super::dispatch_policy_commitment::decode_ordinary_bootstrap_population(physical, selection.authority_store_id()).map_err(|_| guard_error())?;
                    if !population.contains_activation(activation) { return Err(guard_error()); }
                    validate_admission_e2_population(&registry, &population).map_err(|_| guard_error())?;
                    decode_held_retained_admission_source(source.snapshot(), Some(&bytes), &keys, &activation.orchestration_session_id, &world).map_err(|_| guard_error())?;
                    transaction.replace_registry_with_source_guard(&temp, &bytes, &|| {
                        verify_e2()?; verify(Some(&temp)).map_err(|_| guard_error())
                    })?;
                    record_publication(&bytes).map_err(|_| guard_error())
                })
            }).map_err(|error| anyhow::anyhow!("initial retirement publication refused: {error:?}"))?;
            Ok(())
        })
    }

    /// Classify only under the genuine complete source cut, then publish any
    /// initial intent in its own freshly guarded transaction. No read lock or
    /// observation becomes a control grant or crosses IPC.
    pub(crate) fn prepare_initial_bootstrap_control_if_pending_v1(
        &self,
        selection: &super::super::native_launch_context::AuthenticatedOrdinarySelectionV1,
        request: &super::super::dispatch_contract::ValidatedWorldDispatchRequestV1,
    ) -> anyhow::Result<Option<OrdinaryInitialBootstrapControlRequestV1>> {
        use super::super::dispatch_contract::*;
        if request.mode != WorldDispatchModeV1::Retained
            || !matches!(
                request.action,
                WorldDispatchActionV1::CancelWorldWork | WorldDispatchActionV1::StopWorldWorker
            )
        {
            return Ok(None);
        }
        let exact = crate::execution::orchestrator_world_dispatch::resolve_orchestrator_world_dispatch_exact_target_v1(request)?;
        let target = match &exact {
            Some(WorldDispatchControlTargetV1::AcceptedRetainedTurn { .. }) => return Ok(None),
            Some(WorldDispatchControlTargetV1::PendingRetainedAdmission {
                target_participant_id,
                ..
            }) => target_participant_id.as_str(),
            None => request
                .target_participant_id
                .as_deref()
                .ok_or_else(|| anyhow::anyhow!("initial control target missing"))?,
        };
        let initial = selection.with_ordinary_lifecycle_control_sources_v1(
            target,
            |_, source, verify, _| {
                // The common control path reports a request naming another World.
                // This branch grants no initial-owner control or absence observation.
                if source
                    .facts
                    .current
                    .authority
                    .world_binding
                    .as_ref()
                    .is_some_and(|world| {
                        world.world_id != request.world_id
                            || world.world_generation != request.world_generation
                    })
                {
                    verify(None)?;
                    return Ok(false);
                }
                let world = WorldBindingV1 {
                    world_id: request.world_id.clone(),
                    world_generation: request.world_generation,
                };
                let (sources, _registry) = decode_held_retained_admission_source(
                    source.snapshot(),
                    Some(source.admission_registry_bytes()),
                    &source.admission_key_files(),
                    &request.orchestration_session_id,
                    &world,
                )
                .map_err(|error| anyhow::anyhow!("{error}"))?;
                let pending = sources
                    .unresolved_admission_participant_ids
                    .iter()
                    .any(|id| id == target);
                let completed = sources
                    .completed_bootstraps
                    .iter()
                    .any(|a| a.retained_participant_id == target);
                let retired = sources
                    .retired_bootstraps
                    .iter()
                    .any(|a| a.retained_participant_id == target);
                anyhow::ensure!(
                    pending || completed || retired,
                    "initial control target has no authenticated bootstrap source"
                );
                verify(None)?;
                // A completed bootstrap has no initial SDK owner. Its stored
                // Stop grant continues through the actual parked-member path.
                Ok(!completed)
            },
        )?;
        if initial {
            self.prepare_initial_bootstrap_control_v1(selection, request)
                .map(Some)
        } else {
            Ok(None)
        }
    }

    pub(crate) fn prepare_initial_bootstrap_control_v1(
        &self,
        selection: &super::super::native_launch_context::AuthenticatedOrdinarySelectionV1,
        request: &super::super::dispatch_contract::ValidatedWorldDispatchRequestV1,
    ) -> anyhow::Result<OrdinaryInitialBootstrapControlRequestV1> {
        use super::super::dispatch_contract::*;
        let control = match request.action {
            WorldDispatchActionV1::CancelWorldWork => {
                OrdinaryInitialBootstrapControlOperationV1::CancelBootstrap {}
            }
            WorldDispatchActionV1::StopWorldWorker => {
                OrdinaryInitialBootstrapControlOperationV1::StopBootstrap {}
            }
            _ => anyhow::bail!("initial control requires Cancel or Stop"),
        };
        anyhow::ensure!(
            request.mode == WorldDispatchModeV1::Retained
                && request.dispatch_policy_narrowing.is_none(),
            "initial control requires exact retained operation"
        );
        let exact = crate::execution::orchestrator_world_dispatch::resolve_orchestrator_world_dispatch_exact_target_v1(request)?;
        let target = match &exact {
            Some(WorldDispatchControlTargetV1::AcceptedRetainedTurn { .. }) => {
                anyhow::bail!("initial control cannot target accepted work")
            }
            Some(WorldDispatchControlTargetV1::PendingRetainedAdmission {
                target_participant_id,
                ..
            }) => target_participant_id.as_str(),
            None => request
                .target_participant_id
                .as_deref()
                .ok_or_else(|| anyhow::anyhow!("initial control target missing"))?,
        };
        selection.with_ordinary_lifecycle_control_sources_v1(target, |context, source, verify_sources, record_publication| {
            let world = WorldBindingV1 { world_id: request.world_id.clone(), world_generation: request.world_generation };
            let keys = source.admission_key_files();
            let (bootstraps, registry) = decode_held_retained_admission_source(source.snapshot(), Some(source.admission_registry_bytes()),
                &keys, &request.orchestration_session_id, &world).map_err(|error| anyhow::anyhow!("{error}"))?;
            let completed = bootstraps.completed_bootstraps.iter().any(|a| a.retained_participant_id == target);
            let mut registry = registry.ok_or_else(|| anyhow::anyhow!("initial control admission missing"))?;
            let receivers = registry.ordinary_bootstrap_receivers_v1.as_ref().ok_or_else(|| anyhow::anyhow!("initial receiver absent"))?;
            let mut matches = receivers.body.by_activation.values().filter(|r|
                r.activation.orchestration_session_id == request.orchestration_session_id && r.activation.retained_participant_id == target);
            let receiver = matches.next().cloned().ok_or_else(|| anyhow::anyhow!("initial receiver not consumed"))?;
            anyhow::ensure!(matches.next().is_none(), "initial receiver target ambiguous");
            let wire = OrdinaryInitialBootstrapControlRequestV1 { schema_version: 1, source_start: receiver.original_start.clone(),
                activation: receiver.activation.clone(), original_sender_claim_id: receiver.original_sender_claim_id.clone(),
                execute_sha256: receiver.execute_sha256.clone(), receiver_claim_id: receiver.receiver_claim_id.clone(),
                service_span_id: receiver.service_span_id.clone(), control_request_id: request.request_id.clone(),
                control: control.clone(), signal: "INT".into() };
            wire.validate().map_err(anyhow::Error::msg)?;
            anyhow::ensure!(request.caller_participant_id == wire.activation.caller_participant_id
                && request.target_backend_id == wire.activation.target_backend_id
                && request.world_id == wire.activation.target_world.world_id && request.world_generation == wire.activation.target_world.world_generation,
                "initial control request changed original receiver scope");
            if let Some(exact) = &exact {
                let issuer = match wire.activation.launch_kind {
                    E2MemberLaunchKindV1::FreshSpawn => registry.records_by_session.get(&wire.activation.orchestration_session_id)
                        .and_then(|records| records.get(target)).map(|record| record.issuer_request_id.as_str())
                        .ok_or_else(|| anyhow::anyhow!("initial control original issuer absent"))?,
                    E2MemberLaunchKindV1::Fork => wire.activation.request_id.as_str(),
                };
                anyhow::ensure!(matches!(exact, WorldDispatchControlTargetV1::PendingRetainedAdmission {
                    issuer_request_id, target_participant_id } if issuer_request_id == issuer && target_participant_id == target),
                    "initial control exact selector is not its original bootstrap");
            }
            let existing = receiver.controls_v1.iter().find(|stored| stored.control == wire.control);
            let cleanup = existing.is_some() && matches!(wire.control, OrdinaryInitialBootstrapControlOperationV1::StopBootstrap {});
            anyhow::ensure!(!completed || cleanup, "initial bootstrap already completed; use the actual current control owner");
            let policy_request = initial_control_policy_request(selection, context, source, &wire, !cleanup)?;
            let verify = |temp| -> anyhow::Result<()> {
                verify_sources(temp)?;
                let policy = crate::execution::policy_model::resolve_effective_policy_for_bootstrap_home(
                    std::path::Path::new(selection.workspace_root()), &selection.held_authority().bootstrap_home())?;
                crate::execution::orchestrator_world_dispatch::enforce_world_dispatch_read_or_effect_policy(&policy_request, &policy)
            };
            verify(None)?;
            if let Some(existing) = existing {
                return Ok(existing.clone());
            }
            let (_, key) = load_committed_admission_key(selection.authority_store_id(), &registry, &keys).map_err(|error| anyhow::anyhow!("{error}"))?;
            let receivers = registry.ordinary_bootstrap_receivers_v1.as_mut().ok_or_else(|| anyhow::anyhow!("initial receiver absent"))?;
            let record = receivers.body.by_activation.get_mut(&wire.activation.activation_id).ok_or_else(|| anyhow::anyhow!("initial receiver changed"))?;
            record.controls_v1.push(wire.clone());
            record.controls_v1.sort_by(|a,b| a.control.cmp(&b.control));
            receivers.mac_hex = receivers.expected_mac(&key).map_err(|error| anyhow::anyhow!("{error}"))?;
            let bytes = encode_canonical(&registry, "encode initial control grant").map_err(|error| anyhow::anyhow!("{error}"))?;
            let storage = super::super::host_session_authority::store::retained_worker_admission_storage_existing_for_authority(selection.held_authority())
                .map_err(|error| anyhow::anyhow!("{error:?}"))?;
            let temp = format!("admission-registry--{}.tmp", uuid::Uuid::now_v7().simple());
            let guard_error = || super::super::host_session_authority::store::BootstrapError::ordinary_control_source_guard();
            storage.transaction_existing_control(&|| verify(None).map_err(|_| guard_error()), |transaction| {
                if transaction.read_registry()?.as_deref() != Some(source.admission_registry_bytes()) { return Err(guard_error()); }
                transaction.with_existing_e2_population_source(|physical, verify_e2| {
                    let population = super::super::dispatch_policy_commitment::decode_ordinary_bootstrap_population(physical, selection.authority_store_id()).map_err(|_| guard_error())?;
                    if !population.contains_activation(&wire.activation) { return Err(guard_error()); }
                    validate_admission_e2_population(&registry, &population).map_err(|_| guard_error())?;
                    decode_held_retained_admission_source(source.snapshot(), Some(&bytes), &keys, &request.orchestration_session_id, &world).map_err(|_| guard_error())?;
                    transaction.replace_registry_with_source_guard(&temp, &bytes, &|| {
                        verify_e2()?; verify(Some(&temp)).map_err(|_| guard_error())
                    })?;
                    record_publication(&bytes).map_err(|_| guard_error())
                })
            }).map_err(|error| anyhow::anyhow!("initial control publication refused: {error:?}"))?;
            Ok(wire)
        })
    }

    pub(crate) fn with_initial_bootstrap_control_v1(
        &self,
        selection: &super::super::native_launch_context::AuthenticatedOrdinarySelectionV1,
        actual_request: &transport_api_types::ExecuteRequest,
        request: &OrdinaryInitialBootstrapControlRequestV1,
        peer_uid: u32,
        cancel: impl FnOnce() -> anyhow::Result<bool>,
    ) -> anyhow::Result<bool> {
        request.validate().map_err(anyhow::Error::msg)?;
        anyhow::ensure!(
            peer_uid != 0,
            "initial control peer differs from original role principal"
        );
        let scope = self.read_initial_bootstrap_receiver_scope_v1(
            selection,
            actual_request,
            &request.service_span_id,
        )?;
        self.with_initial_bootstrap_control_scope_v1(selection, &scope, request, peer_uid, cancel)
    }

    pub(crate) fn with_initial_bootstrap_control_scope_v1(
        &self,
        selection: &super::super::native_launch_context::AuthenticatedOrdinarySelectionV1,
        scope: &OrdinaryInitialBootstrapReceiverScopeV1,
        request: &OrdinaryInitialBootstrapControlRequestV1,
        peer_uid: u32,
        cancel: impl FnOnce() -> anyhow::Result<bool>,
    ) -> anyhow::Result<bool> {
        request.validate().map_err(anyhow::Error::msg)?;
        anyhow::ensure!(
            peer_uid != 0
                && scope.original_start == request.source_start
                && scope.activation == request.activation
                && scope.original_sender_claim_id == request.original_sender_claim_id
                && scope.execute_sha256 == request.execute_sha256
                && scope.receiver_claim_id == request.receiver_claim_id
                && scope.service_span_id == request.service_span_id,
            "initial control differs from actual admitted Execute or UDS principal"
        );
        selection.with_ordinary_lifecycle_control_sources_v1(&request.activation.retained_participant_id,
            |context, source, verify_sources, _| {
                for principal in [selection.native_principal(), selection.requested_principal()] {
                    anyhow::ensure!(matches!(principal, transport_api_types::PlatformPrincipalV1::Unix { uid, .. } if *uid == peer_uid),
                        "initial control peer differs from original role principal");
                }
                let world = WorldBindingV1 { world_id: request.activation.target_world.world_id.clone(), world_generation: request.activation.target_world.world_generation };
                let keys = source.admission_key_files();
                let (_, registry) = decode_held_retained_admission_source(source.snapshot(), Some(source.admission_registry_bytes()), &keys,
                    &request.activation.orchestration_session_id, &world).map_err(|error| anyhow::anyhow!("{error}"))?;
                let registry = registry.ok_or_else(|| anyhow::anyhow!("initial control admission missing"))?;
                let receiver = registry.ordinary_bootstrap_receivers_v1.as_ref().and_then(|r| r.body.by_activation.get(&request.activation.activation_id))
                    .ok_or_else(|| anyhow::anyhow!("initial receiver absent"))?;
                anyhow::ensure!(matches_receiver(request, receiver) && receiver.controls_v1.contains(request),
                    "initial control has no exact authenticated operation grant");
                // A stored Stop remains cleanup authority after Host settlement;
                // fresh Stop issuance and user Cancel still require today's actor.
                let policy_request = initial_control_policy_request(selection, context, source, request,
                    matches!(request.control, OrdinaryInitialBootstrapControlOperationV1::CancelBootstrap {}))?;
                let verify = || -> anyhow::Result<()> {
                    verify_sources(None)?;
                    let policy = crate::execution::policy_model::resolve_effective_policy_for_bootstrap_home(
                        std::path::Path::new(selection.workspace_root()), &selection.held_authority().bootstrap_home())?;
                    crate::execution::orchestrator_world_dispatch::enforce_world_dispatch_read_or_effect_policy(&policy_request, &policy)
                };
                verify()?;
                let original_root = source.snapshot().hsa_state_root_bytes();
                super::super::host_session_authority::store::with_existing_retained_worker_admission_read(selection.held_authority(),
                    |root, bytes, locked_keys| -> anyhow::Result<bool> {
                        anyhow::ensure!(root.to_canonical_bytes()? == original_root && bytes == Some(source.admission_registry_bytes())
                            && locked_keys.into_iter().collect::<BTreeMap<_,_>>() == keys.iter().cloned().collect::<BTreeMap<_,_>>(),
                            "initial control locked original sources changed");
                        verify()?;
                        let result = cancel();
                        verify()?;
                        result
                    }).map_err(|error| anyhow::anyhow!("initial control source lock refused: {error:?}"))?
            })
    }
}

#[cfg(target_os = "linux")]
impl RetainedWorkerRuntime {
    pub(crate) fn claim_initial_bootstrap_receiver(
        &self,
        selection: &super::super::native_launch_context::AuthenticatedOrdinarySelectionV1,
        request: &transport_api_types::ExecuteRequest,
        service_span_id: &str,
    ) -> Result<(), RetainedWorkerRuntimeError> {
        request.validate().map_err(|_| conflict())?;
        let member = request
            .member_dispatch
            .as_ref()
            .and_then(transport_api_types::MemberDispatchRequest::as_v1)
            .ok_or_else(conflict)?;
        let activation = member.e2_launch_activation.as_ref().ok_or_else(conflict)?;
        if request.acceptance_context.is_some()
            || service_span_id.is_empty()
            || service_span_id.trim() != service_span_id
            || member.participant_id != activation.retained_participant_id
            || member.run_id != activation.bootstrap_run_id
            || member.orchestrator_participant_id != activation.caller_participant_id
            || member.backend_id != activation.target_backend_id
            || member.world_id != activation.target_world.world_id
            || member.world_generation != activation.target_world.world_generation
        {
            return Err(conflict());
        }
        let execute_sha256 = lower_hex(&Sha256::digest(
            serde_json::to_vec(request).map_err(|_| conflict())?,
        ));
        selection.with_ordinary_lifecycle_control_sources_v1(&member.participant_id, |context, source, verify, record_publication| {
            let current = &source.facts.current;
            let start = crate::execution::orchestrator_world_dispatch::ordinary_start_selection_carrier(selection)?;
            let world = current.authority.world_binding.as_ref().ok_or_else(|| anyhow::anyhow!("bootstrap receiver current World absent"))?;
            anyhow::ensure!(request.ordinary_start_selection.as_ref() == Some(&start)
                && current.authority.lifecycle_posture == HostSessionPostureV1::ActiveAttached
                && current.caller.participant_id == activation.caller_participant_id
                && current.caller.participant_id == selection.current_participant_id()
                && current.caller.descriptor.execution_scope == AgentExecutionScopeV1::Host
                && current.caller.descriptor.backend_id == context.selections.native.backend_id
                && current.caller.descriptor.agent_id == context.selections.native.realized_agent_id
                && context.selections.requested.backend_id == activation.target_backend_id
                && world.world_id == activation.target_world.world_id && world.world_generation == activation.target_world.world_generation,
                "bootstrap receiver changed current Host/World/source roles");
            let policy = crate::execution::policy_model::resolve_effective_policy_for_bootstrap_home(
                std::path::Path::new(selection.workspace_root()), &selection.held_authority().bootstrap_home())?;
            selection.revalidate_requested_account(&policy)?;
            let steering = policy.world_dispatch_policy();
            let action = match activation.launch_kind { E2MemberLaunchKindV1::FreshSpawn => "spawn_world_worker", E2MemberLaunchKindV1::Fork => "fork_world_worker" };
            anyhow::ensure!(steering.enabled && steering.allowed_actions.iter().any(|value| value == action)
                && steering.allowed_modes.iter().any(|value| value == "retained")
                && steering.allowed_backends.iter().any(|value| value == &activation.target_backend_id)
                && steering.max_live_retained_workers != 0, "bootstrap receiver current policy refuses launch");
            let keys = source.admission_key_files();
            let (_, registry) = decode_held_retained_admission_source(source.snapshot(), Some(source.admission_registry_bytes()), &keys,
                &member.orchestration_session_id, world).map_err(|error| anyhow::anyhow!("{error}"))?;
            let mut registry = registry.ok_or_else(|| anyhow::anyhow!("bootstrap receiver original admission absent"))?;
            let (_, key) = load_committed_admission_key(selection.authority_store_id(), &registry, &keys).map_err(|error| anyhow::anyhow!("{error}"))?;
            let mut receivers = registry.ordinary_bootstrap_receivers_v1.take().unwrap_or_else(|| OrdinaryBootstrapReceiverRegistryV1::new(&key));
            anyhow::ensure!(!receivers.body.by_activation.contains_key(&activation.activation_id), "ordinary bootstrap receiver already consumed; native input cannot be redelivered");
            let original_sender_claim_id = sender_claim(&registry, &start, activation).map_err(|error| anyhow::anyhow!("{error}"))?
                .ok_or_else(|| anyhow::anyhow!("bootstrap receiver original sender claim unavailable"))?;
            if activation.launch_kind == E2MemberLaunchKindV1::FreshSpawn {
                anyhow::ensure!(registry.records_by_session.get(&activation.orchestration_session_id)
                    .and_then(|records| records.get(&activation.retained_participant_id)).is_some_and(|record|
                        matches!(record.state, RetainedWorkerAdmissionStateV1::TransportClaimedNonterminal {
                            transport_span_id: None, stream_id: None, last_frame_sequence: None, .. })), "bootstrap receiver requires fresh marked Spawn sender claim");
            } else {
                anyhow::ensure!(registry.ordinary_forks_v1.as_ref().and_then(|forks|
                    forks.inspection_record(&activation.orchestration_session_id, &activation.retained_participant_id))
                    .is_some_and(|(record, _)| record.accepts_initial_receiver_v1()),
                    "bootstrap receiver requires fresh marked Fork sender claim");
            }
            let mut receiver = ReceiverRecordV1 {
                original_start: start, activation: activation.clone(), original_sender_claim_id, execute_sha256,
                receiver_claim_id: format!("obr_{}", uuid::Uuid::now_v7()), service_span_id: service_span_id.into(),
                consumed_at: TimestampV1::parse(chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Nanos, true)).map_err(|error| anyhow::anyhow!("{error}"))?,
                controls_v1: Vec::new(), retirement_v1: None,
            };
            if let Some(pending) = receivers.body.pending_controls_v1.get(&activation.activation_id) {
                for intent in pending {
                    anyhow::ensure!(intent.original_start == receiver.original_start && intent.activation == receiver.activation
                        && intent.original_sender_claim_id == receiver.original_sender_claim_id, "pending initial control changed received original source");
                    let wire = OrdinaryInitialBootstrapControlRequestV1 { schema_version: 1,
                        source_start: receiver.original_start.clone(), activation: receiver.activation.clone(),
                        original_sender_claim_id: receiver.original_sender_claim_id.clone(), execute_sha256: receiver.execute_sha256.clone(),
                        receiver_claim_id: receiver.receiver_claim_id.clone(), service_span_id: receiver.service_span_id.clone(),
                        control_request_id: intent.control_request_id.clone(), control: intent.control.clone(), signal: intent.signal.clone(),
                    };
                    wire.validate().map_err(anyhow::Error::msg)?;
                    receiver.controls_v1.push(wire);
                }
            }
            receivers.body.by_activation.insert(activation.activation_id.clone(), receiver);
            receivers.mac_hex = receivers.expected_mac(&key).map_err(|error| anyhow::anyhow!("{error}"))?;
            registry.ordinary_bootstrap_receivers_v1 = Some(receivers);
            let bytes = encode_canonical(&registry, "encode bootstrap receiver consumption").map_err(|error| anyhow::anyhow!("{error}"))?;
            let storage = super::super::host_session_authority::store::retained_worker_admission_storage_existing_for_authority(selection.held_authority())
                .map_err(|error| anyhow::anyhow!("{error:?}"))?;
            let mut nonce = [0u8;16]; rand::rngs::OsRng.fill_bytes(&mut nonce);
            let temp_name = format!("admission-registry--{}.tmp", lower_hex(&nonce));
            let guard_error = || super::super::host_session_authority::store::BootstrapError::ordinary_control_source_guard();
            storage.transaction_existing_control(&|| verify(None).map_err(|_| guard_error()), |transaction| {
                if transaction.read_registry()?.as_deref() != Some(source.admission_registry_bytes()) { return Err(guard_error()); }
                transaction.with_existing_e2_population_source(|physical, verify_e2| {
                    let population = super::super::dispatch_policy_commitment::decode_ordinary_bootstrap_population(physical, selection.authority_store_id())
                        .map_err(|_| guard_error())?;
                    if !population.contains_activation(activation) { return Err(guard_error()); }
                    validate_admission_e2_population(&registry, &population).map_err(|_| guard_error())?;
                    decode_held_retained_admission_source(source.snapshot(), Some(&bytes), &keys, &member.orchestration_session_id, world)
                        .map_err(|_| guard_error())?;
                    transaction.replace_registry_with_source_guard(&temp_name, &bytes, &|| {
                        verify_e2()?;
                        verify(Some(&temp_name)).map_err(|_| guard_error())
                    })?;
                    record_publication(&bytes).map_err(|_| guard_error())
                })
            }).map_err(|error| anyhow::anyhow!("bootstrap receiver publication refused: {error:?}"))?;
            Ok(())
        }).map_err(|error| RetainedWorkerRuntimeError(error.to_string()))
    }
}
