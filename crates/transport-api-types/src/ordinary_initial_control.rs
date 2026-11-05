//! References to original retained-bootstrap control intent. These data grant
//! no control without the matching authenticated source and actual service owner.
use crate::{
    selection_sha256, validate_narrowing_identity, validate_prefixed_uuid_v7,
    E2MemberLaunchActivationCarrierV1, OrdinaryStartSelectionCarrierV1,
};
use serde::{Deserialize, Serialize};

/// Initial native cancellation and logical Stop have separate policy grants.
/// Neither variant acknowledges native settlement or member retirement.
/// ```
/// use transport_api_types::OrdinaryInitialBootstrapControlOperationV1;
/// assert!(serde_json::from_str::<OrdinaryInitialBootstrapControlOperationV1>(r#"{"operation":"cancel_bootstrap","terminal":true}"#).is_err());
/// ```
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(tag = "operation", rename_all = "snake_case", deny_unknown_fields)]
pub enum OrdinaryInitialBootstrapControlOperationV1 {
    CancelBootstrap {},
    StopBootstrap {},
}

/// Exact initial receiver and separately published operation intent. The service
/// authenticates all fields against its original admitted Execute and current
/// source/peer/policy before touching its reservation or SDK cancellation handle.
/// ```
/// use transport_api_types::OrdinaryInitialBootstrapControlRequestV1;
/// assert!(serde_json::from_str::<OrdinaryInitialBootstrapControlRequestV1>("{}").is_err());
/// ```
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(try_from = "ControlDef")]
pub struct OrdinaryInitialBootstrapControlRequestV1 {
    pub schema_version: u32,
    pub source_start: OrdinaryStartSelectionCarrierV1,
    pub activation: E2MemberLaunchActivationCarrierV1,
    pub original_sender_claim_id: String,
    pub execute_sha256: String,
    pub receiver_claim_id: String,
    pub service_span_id: String,
    pub control_request_id: String,
    pub control: OrdinaryInitialBootstrapControlOperationV1,
    pub signal: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ControlDef {
    schema_version: u32,
    source_start: OrdinaryStartSelectionCarrierV1,
    activation: E2MemberLaunchActivationCarrierV1,
    original_sender_claim_id: String,
    execute_sha256: String,
    receiver_claim_id: String,
    service_span_id: String,
    control_request_id: String,
    control: OrdinaryInitialBootstrapControlOperationV1,
    signal: String,
}

impl OrdinaryInitialBootstrapControlRequestV1 {
    /// Validate representation only. A valid request is not a control grant,
    /// native terminal acknowledgment, process-absence proof or permission to send.
    /// ```no_run
    /// # fn check(value: &transport_api_types::OrdinaryInitialBootstrapControlRequestV1) -> Result<(), String> {
    /// value.validate()
    /// # }
    /// ```
    pub fn validate(&self) -> Result<(), String> {
        self.source_start.validate()?;
        self.activation.validate()?;
        if self.schema_version != 1
            || !matches!(self.signal.as_str(), "INT" | "TERM")
            || self.source_start.authority_store_id
                != self.activation.commitment_ref.authority_store_id
            || self.source_start.original_session_id != self.activation.orchestration_session_id
        {
            return Err(
                "ordinary initial control changed schema, signal or original Start scope".into(),
            );
        }
        selection_sha256(&self.execute_sha256, "original Execute digest")?;
        validate_prefixed_uuid_v7("initial receiver claim", &self.receiver_claim_id, "obr_")?;
        for (name, value) in [
            ("original sender", &self.original_sender_claim_id),
            ("service span", &self.service_span_id),
            ("control request", &self.control_request_id),
        ] {
            validate_narrowing_identity(name, value)?;
        }
        Ok(())
    }
}

impl TryFrom<ControlDef> for OrdinaryInitialBootstrapControlRequestV1 {
    type Error = String;
    fn try_from(value: ControlDef) -> Result<Self, Self::Error> {
        let request = Self {
            schema_version: value.schema_version,
            source_start: value.source_start,
            activation: value.activation,
            original_sender_claim_id: value.original_sender_claim_id,
            execute_sha256: value.execute_sha256,
            receiver_claim_id: value.receiver_claim_id,
            service_span_id: value.service_span_id,
            control_request_id: value.control_request_id,
            control: value.control,
            signal: value.signal,
        };
        request.validate()?;
        Ok(request)
    }
}

/// Delivery facts only. A latch precedes SDK handle installation; delivery
/// means the existing SDK cancel handle was invoked. Unknown owner proves no
/// process absence. Retired carries the separate actual-owner report, whose
/// source and exact receiver must still be authenticated independently.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
pub enum OrdinaryInitialBootstrapControlDeliveryV1 {
    Latched,
    Delivered,
    UnknownOwner,
    Retired {
        report: Box<OrdinaryInitialBootstrapRetirementV1>,
    },
}

/// Closed control response. Delivery alone supplies no cleanup acknowledgment;
/// Retired carries the separately authenticated original-owner closeout report.
/// ```
/// use transport_api_types::OrdinaryInitialBootstrapControlResponseV1;
/// assert!(serde_json::from_str::<OrdinaryInitialBootstrapControlResponseV1>(r#"{"schema_version":1,"delivery":"latched","terminal":true}"#).is_err());
/// ```
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(try_from = "ControlResponseDef")]
pub struct OrdinaryInitialBootstrapControlResponseV1 {
    pub schema_version: u32,
    pub delivery: OrdinaryInitialBootstrapControlDeliveryV1,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ControlResponseDef {
    schema_version: u32,
    delivery: OrdinaryInitialBootstrapControlDeliveryV1,
}
impl TryFrom<ControlResponseDef> for OrdinaryInitialBootstrapControlResponseV1 {
    type Error = String;
    fn try_from(value: ControlResponseDef) -> Result<Self, Self::Error> {
        if value.schema_version != 1 {
            return Err("unsupported initial control response schema".into());
        }
        Ok(Self {
            schema_version: 1,
            delivery: value.delivery,
        })
    }
}

/// Copied SDK admission or process-wait data. The no-spawn variant requires a
/// positive sealed SDK admission observation; an empty wait list cannot supply it.
/// Neither variant acknowledges gateway, member, World or exclusion cleanup.
/// ```
/// use transport_api_types::OrdinaryInitialBootstrapProcessCloseoutV1;
/// assert!(serde_json::from_str::<OrdinaryInitialBootstrapProcessCloseoutV1>(r#"{"kind":"processes_waited","processes":[]}"#).is_err());
/// ```
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(
    try_from = "InitialProcessCloseoutDef",
    tag = "kind",
    rename_all = "snake_case"
)]
pub enum OrdinaryInitialBootstrapProcessCloseoutV1 {
    ClosedBeforeAnySpawn {},
    ProcessesWaited {
        processes: Vec<crate::NativeProcessWaitObservationV1>,
    },
}

#[derive(Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
enum InitialProcessCloseoutDef {
    ClosedBeforeAnySpawn {},
    ProcessesWaited {
        processes: Vec<crate::NativeProcessWaitObservationV1>,
    },
}

impl OrdinaryInitialBootstrapProcessCloseoutV1 {
    /// Check representation only; authentication and actual custody are separate.
    /// ```no_run
    /// # fn check(value: &transport_api_types::OrdinaryInitialBootstrapProcessCloseoutV1) -> Result<(), String> {
    /// value.validate()
    /// # }
    /// ```
    pub fn validate(&self) -> Result<(), String> {
        if let Self::ProcessesWaited { processes } = self {
            if processes.is_empty() || processes.len() > 1024 {
                return Err("initial closeout requires a bounded nonempty actual wait list".into());
            }
            let mut pids = std::collections::BTreeSet::new();
            for process in processes {
                process.validate()?;
                if !pids.insert(process.pid) {
                    return Err("initial closeout repeats an actual process wait".into());
                }
            }
        }
        Ok(())
    }
}

impl TryFrom<InitialProcessCloseoutDef> for OrdinaryInitialBootstrapProcessCloseoutV1 {
    type Error = String;
    fn try_from(value: InitialProcessCloseoutDef) -> Result<Self, Self::Error> {
        let result = match value {
            InitialProcessCloseoutDef::ClosedBeforeAnySpawn {} => Self::ClosedBeforeAnySpawn {},
            InitialProcessCloseoutDef::ProcessesWaited { processes } => {
                Self::ProcessesWaited { processes }
            }
        };
        result.validate()?;
        Ok(result)
    }
}

/// Exact unsuccessful initial-bootstrap owner closeout. These data carry no
/// native session or Registered readiness. The service must independently join
/// SDK custody, home handoff, gateway disposal and member retirement first.
/// ```
/// use transport_api_types::OrdinaryInitialBootstrapRetirementObservationV1;
/// assert!(serde_json::from_str::<OrdinaryInitialBootstrapRetirementObservationV1>("{}").is_err());
/// ```
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OrdinaryInitialBootstrapRetirementObservationV1 {
    pub source_start: OrdinaryStartSelectionCarrierV1,
    pub activation: E2MemberLaunchActivationCarrierV1,
    pub original_sender_claim_id: String,
    pub execute_sha256: String,
    pub receiver_claim_id: String,
    pub service_span_id: String,
    pub retirement_id: String,
    pub native_processes: OrdinaryInitialBootstrapProcessCloseoutV1,
    pub gateway_disposal: crate::OrdinaryGatewayDisposalV1,
    pub frame_identity: crate::RuntimeFrameIdentityV1,
    pub event_identity: crate::RuntimeEventIdentityV1,
}

/// Closed service report, independent of raw SDK cancellation or a stream Exit.
/// Consumers authenticate the original receiver and the service connection;
/// parsing this report never supplies either authority.
/// ```
/// use transport_api_types::OrdinaryInitialBootstrapRetirementV1;
/// assert!(serde_json::from_str::<OrdinaryInitialBootstrapRetirementV1>("{}").is_err());
/// ```
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(try_from = "InitialRetirementDef")]
pub struct OrdinaryInitialBootstrapRetirementV1 {
    schema_version: u32,
    observation: OrdinaryInitialBootstrapRetirementObservationV1,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct InitialRetirementDef {
    schema_version: u32,
    observation: OrdinaryInitialBootstrapRetirementObservationV1,
}
impl TryFrom<InitialRetirementDef> for OrdinaryInitialBootstrapRetirementV1 {
    type Error = String;
    fn try_from(value: InitialRetirementDef) -> Result<Self, Self::Error> {
        if value.schema_version != 1 {
            return Err("unsupported initial retirement report schema".into());
        }
        Self::from_observation(value.observation)
    }
}
impl OrdinaryInitialBootstrapRetirementV1 {
    /// Service-owned marker. Backend-supplied events cannot publish this key.
    pub const EVENT_DATA_KEY: &'static str = "ordinary_initial_bootstrap_retirement_v1";

    /// Compare copied receiver identity only. This grants no control and does
    /// not authenticate either the report or operation intent.
    /// ```no_run
    /// # fn check(report: &transport_api_types::OrdinaryInitialBootstrapRetirementV1,
    /// # control: &transport_api_types::OrdinaryInitialBootstrapControlRequestV1) {
    /// let _ = report.matches_control(control);
    /// # }
    /// ```
    pub fn matches_control(&self, control: &OrdinaryInitialBootstrapControlRequestV1) -> bool {
        let observed = self.observation();
        observed.source_start == control.source_start
            && observed.activation == control.activation
            && observed.original_sender_claim_id == control.original_sender_claim_id
            && observed.execute_sha256 == control.execute_sha256
            && observed.receiver_claim_id == control.receiver_claim_id
            && observed.service_span_id == control.service_span_id
    }

    /// Validate copied facts only, without authenticating their producer.
    /// ```no_run
    /// # fn encode(value: transport_api_types::OrdinaryInitialBootstrapRetirementObservationV1) -> Result<transport_api_types::OrdinaryInitialBootstrapRetirementV1, String> {
    /// transport_api_types::OrdinaryInitialBootstrapRetirementV1::from_observation(value)
    /// # }
    /// ```
    pub fn from_observation(
        observation: OrdinaryInitialBootstrapRetirementObservationV1,
    ) -> Result<Self, String> {
        let value = &observation;
        value.source_start.validate()?;
        value.activation.validate()?;
        value.frame_identity.validate()?;
        value.event_identity.validate()?;
        value.native_processes.validate()?;
        selection_sha256(&value.execute_sha256, "original Execute digest")?;
        validate_prefixed_uuid_v7("initial receiver claim", &value.receiver_claim_id, "obr_")?;
        validate_prefixed_uuid_v7("initial retirement", &value.retirement_id, "obrt_")?;
        validate_narrowing_identity("original sender", &value.original_sender_claim_id)?;
        validate_narrowing_identity("service span", &value.service_span_id)?;
        if value.source_start.authority_store_id
            != value.activation.commitment_ref.authority_store_id
            || value.source_start.original_session_id != value.activation.orchestration_session_id
        {
            return Err("initial retirement changed original Start scope".into());
        }
        let (runtime_id, process) = match &value.gateway_disposal {
            crate::OrdinaryGatewayDisposalV1::ClosedBeforeIssue { runtime_id } => {
                (runtime_id, None)
            }
            crate::OrdinaryGatewayDisposalV1::ProcessWaited {
                runtime_id,
                pid,
                termination,
            } => (runtime_id, Some((*pid, termination))),
        };
        validate_narrowing_identity("gateway runtime", runtime_id)?;
        if process.is_some_and(|(pid, status)| {
            pid == 0
                || match status {
                    crate::NativeProcessTerminationV1::Exited { exit_code } => *exit_code < 0,
                    crate::NativeProcessTerminationV1::Signaled { signal } => *signal <= 0,
                }
        }) {
            return Err("initial retirement omitted actual gateway wait".into());
        }
        Ok(Self {
            schema_version: 1,
            observation,
        })
    }

    /// Read copied closeout facts; this exposes no control or send handle.
    /// ```no_run
    /// # fn read(value: &transport_api_types::OrdinaryInitialBootstrapRetirementV1) {
    /// let _ = value.observation();
    /// # }
    /// ```
    pub fn observation(&self) -> &OrdinaryInitialBootstrapRetirementObservationV1 {
        &self.observation
    }

    /// Validate the exact typed event on an independently authenticated stream.
    /// A missing marker is an unrelated event; malformed markers fail closed.
    /// ```no_run
    /// # fn read(frame: &transport_api_types::RuntimeFrameIdentityV1, event: &substrate_common::agent_events::AgentEvent) {
    /// let _ = transport_api_types::OrdinaryInitialBootstrapRetirementV1::from_service_event(frame, event);
    /// # }
    /// ```
    pub fn from_service_event(
        frame: &crate::RuntimeFrameIdentityV1,
        event: &substrate_common::agent_events::AgentEvent,
    ) -> Result<Option<Self>, String> {
        let Some(value) = event.data.get(Self::EVENT_DATA_KEY) else {
            return Ok(None);
        };
        let fail = || "invalid initial retirement service-event boundary".to_string();
        let report: Self = serde_json::from_value(value.clone()).map_err(|_| fail())?;
        frame.validate()?;
        event.validate_identity_contract()?;
        let observed = report.observation();
        let activation = &observed.activation;
        if event.data.as_object().map(|object| object.len()) != Some(1)
            || frame != &observed.frame_identity
            || event.event_identity.as_ref() != Some(&observed.event_identity)
            || event.kind != substrate_common::agent_events::AgentEventKind::Status
            || event.worker_event.is_some()
            || event.orchestration_session_id != activation.orchestration_session_id
            || event.run_id != activation.bootstrap_run_id
            || event.participant_id.as_deref() != Some(activation.retained_participant_id.as_str())
            || event.backend_id.as_deref() != Some(activation.target_backend_id.as_str())
            || event.role.as_deref() != Some("member")
            || event.world_id.as_deref() != Some(activation.target_world.world_id.as_str())
            || event.world_generation != Some(activation.target_world.world_generation)
            || event.parent_participant_id != activation.source_participant_id
            || event.resumed_from_participant_id.is_some()
            || event.span_id.as_deref() != Some(observed.service_span_id.as_str())
        {
            return Err(fail());
        }
        Ok(Some(report))
    }
}

#[cfg(test)]
mod initial_closeout_tests {
    use super::*;

    #[test]
    fn test_initial_process_closeout_requires_positive_admission_or_actual_waits() {
        // An empty successful wait list cannot be upgraded to no-ever-spawned.
        for invalid in [
            r#"{"kind":"processes_waited","processes":[]}"#,
            r#"{"kind":"closed_before_any_spawn","processes":[]}"#,
            r#"{"kind":"processes_waited","processes":[{"pid":0,"termination":{"kind":"exited","exit_code":0},"recovery_termination":null}]}"#,
            r#"{"kind":"processes_waited","processes":[{"pid":41,"termination":{"kind":"signaled","signal":0},"recovery_termination":null}]}"#,
            r#"{"kind":"processes_waited","processes":[{"pid":41,"termination":{"kind":"exited","exit_code":0},"recovery_termination":null},{"pid":41,"termination":{"kind":"exited","exit_code":0},"recovery_termination":null}]}"#,
        ] {
            assert!(
                serde_json::from_str::<OrdinaryInitialBootstrapProcessCloseoutV1>(invalid).is_err()
            );
        }
        let no_ever: OrdinaryInitialBootstrapProcessCloseoutV1 =
            serde_json::from_str(r#"{"kind":"closed_before_any_spawn"}"#).unwrap();
        assert_eq!(
            serde_json::to_value(&no_ever).unwrap(),
            serde_json::json!({"kind":"closed_before_any_spawn"})
        );
        assert!(matches!(
            no_ever,
            OrdinaryInitialBootstrapProcessCloseoutV1::ClosedBeforeAnySpawn {}
        ));
        let waited: OrdinaryInitialBootstrapProcessCloseoutV1 = serde_json::from_str(
            r#"{"kind":"processes_waited","processes":[{"pid":41,"termination":{"kind":"signaled","signal":9},"recovery_termination":{"kind":"exited","exit_code":0}}]}"#,
        ).unwrap();
        assert_eq!(
            serde_json::from_value::<OrdinaryInitialBootstrapProcessCloseoutV1>(
                serde_json::to_value(&waited).unwrap()
            )
            .unwrap(),
            waited
        );
        let OrdinaryInitialBootstrapProcessCloseoutV1::ProcessesWaited { processes } = waited
        else {
            panic!("physical wait was changed");
        };
        assert_eq!(
            processes[0].termination,
            crate::NativeProcessTerminationV1::Signaled { signal: 9 }
        );
        assert_eq!(
            processes[0].recovery_termination,
            Some(crate::NativeProcessTerminationV1::Exited { exit_code: 0 })
        );
    }
}
