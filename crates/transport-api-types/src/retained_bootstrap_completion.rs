//! Versioned reports about a completed native bootstrap of a retained World worker.
//! These data never grant permission. Consumers authenticate the service source,
//! original issuance, R0/E2 ancestry and the applicable physical custody separately.
use crate::{
    E2MemberLaunchActivationCarrierV1, OrdinaryGatewayRuntimeIdentityV1,
    OrdinaryStartSelectionCarrierV1, RuntimeEventIdentityV1, RuntimeFrameIdentityV1,
    RuntimeTerminalIdentityV1,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

/// The actual wait status reported by the owned process supervisor.
/// ```
/// use transport_api_types::NativeProcessTerminationV1;
/// let status = NativeProcessTerminationV1::Exited { exit_code: 0 };
/// assert!(matches!(status, NativeProcessTerminationV1::Exited { exit_code: 0 }));
/// ```
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum NativeProcessTerminationV1 {
    Exited { exit_code: i32 },
    Signaled { signal: i32 },
}

/// A report that the existing SDK custody/stdio settlement returned success.
/// This representation is not itself an SDK acknowledgment or authority.
/// ```
/// use transport_api_types::NativeSettlementAcknowledgmentV1;
/// assert!(serde_json::from_str::<NativeSettlementAcknowledgmentV1>("\"unsupported\"").is_err());
/// ```
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NativeSettlementAcknowledgmentV1 {
    SdkChecked,
}

/// Disposal of the exact issued gateway lease, after its admitted operations join.
/// `ClosedBeforeIssue` requires a positive closed-owner observation; absence of
/// a file, process or registry entry cannot produce it.
/// ```
/// use transport_api_types::OrdinaryGatewayDisposalV1;
/// let data = OrdinaryGatewayDisposalV1::ClosedBeforeIssue { runtime_id: "lease-data".into() };
/// assert!(serde_json::to_string(&data).unwrap().contains("closed_before_issue"));
/// ```
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum OrdinaryGatewayDisposalV1 {
    ProcessWaited {
        runtime_id: String,
        pid: u32,
        termination: NativeProcessTerminationV1,
    },
    ClosedBeforeIssue {
        runtime_id: String,
    },
}

/// Closed codec data supplied by the actual retained bootstrap completion owner.
/// The native session locator and the logical bootstrap run have separate meanings.
/// ```
/// use transport_api_types::RetainedBootstrapObservationV1;
/// assert!(serde_json::from_str::<RetainedBootstrapObservationV1>("{}").is_err());
/// ```
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RetainedBootstrapObservationV1 {
    pub source_start: OrdinaryStartSelectionCarrierV1,
    pub runtime: OrdinaryGatewayRuntimeIdentityV1,
    pub activation: E2MemberLaunchActivationCarrierV1,
    pub native_session_id: String,
    pub span_id: String,
    pub frame_identity: RuntimeFrameIdentityV1,
    pub event_identity: RuntimeEventIdentityV1,
    pub terminal_identity: RuntimeTerminalIdentityV1,
    pub native_settlement: NativeSettlementAcknowledgmentV1,
    pub native_termination: NativeProcessTerminationV1,
    pub gateway_disposal: OrdinaryGatewayDisposalV1,
}

impl RetainedBootstrapObservationV1 {
    fn validate(&self) -> Result<(), String> {
        self.validate_identity_and_cleanup()?;
        if self.native_termination != (NativeProcessTerminationV1::Exited { exit_code: 0 }) {
            return Err("retained bootstrap omitted exact checked native completion".into());
        }
        Ok(())
    }

    fn validate_identity_and_cleanup(&self) -> Result<(), String> {
        self.source_start.validate()?;
        self.runtime.validate()?;
        self.activation.validate()?;
        self.frame_identity.validate()?;
        self.event_identity.validate()?;
        self.terminal_identity.validate()?;
        if self.native_session_id.is_empty()
            || self.native_session_id.trim() != self.native_session_id
            || self.span_id.is_empty()
            || self.span_id.trim() != self.span_id
            || !self.terminal_identity.matches_event(&self.event_identity)
            || self.frame_identity.frame_sequence <= self.event_identity.event_sequence
        {
            return Err("retained bootstrap omitted exact checked native completion".into());
        }
        let OrdinaryGatewayRuntimeIdentityV1::WorldWorker {
            orchestration_session_id,
            caller_participant_id,
            participant_id,
            backend_id,
            run_id,
            world_id,
            world_generation,
        } = &self.runtime
        else {
            return Err("retained bootstrap requires a World worker".into());
        };
        let activation = &self.activation;
        if self.source_start.authority_store_id != activation.commitment_ref.authority_store_id
            || self.source_start.authority_store_id
                != activation.immutable_worker_cap_ref.authority_store_id
            || self.source_start.original_session_id != *orchestration_session_id
            || activation.orchestration_session_id != *orchestration_session_id
            || activation.caller_participant_id != *caller_participant_id
            || activation.retained_participant_id != *participant_id
            || activation.target_backend_id != *backend_id
            || activation.bootstrap_run_id != *run_id
            || activation.target_world.world_id != *world_id
            || activation.target_world.world_generation != *world_generation
        {
            return Err("retained bootstrap changed its original worker/claim/World source".into());
        }
        match &self.gateway_disposal {
            OrdinaryGatewayDisposalV1::ProcessWaited {
                runtime_id,
                pid,
                termination,
            } => {
                if runtime_id.is_empty() || runtime_id.trim() != runtime_id || *pid == 0 {
                    return Err("retained bootstrap omitted its exact issued gateway".into());
                }
                if matches!(termination, NativeProcessTerminationV1::Signaled { signal } if *signal <= 0)
                {
                    return Err("retained bootstrap has an invalid gateway wait status".into());
                }
            }
            OrdinaryGatewayDisposalV1::ClosedBeforeIssue { runtime_id } => {
                if runtime_id.is_empty() || runtime_id.trim() != runtime_id {
                    return Err("retained bootstrap omitted its closed gateway owner".into());
                }
            }
        }
        Ok(())
    }
}

/// A separately versioned, domain-separated bootstrap report. It is not a
/// logical-worker terminal and does not make any retained target routable.
/// ```
/// use transport_api_types::RetainedBootstrapCompletionV1;
/// assert!(serde_json::from_str::<RetainedBootstrapCompletionV1>(r#"{"schema_version":2}"#).is_err());
/// ```
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct RetainedBootstrapCompletionV1 {
    schema_version: u32,
    domain: String,
    observation: RetainedBootstrapObservationV1,
    sha256: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RetainedBootstrapCompletionDef {
    schema_version: u32,
    domain: String,
    observation: RetainedBootstrapObservationV1,
    sha256: String,
}

impl RetainedBootstrapCompletionV1 {
    pub const DOMAIN: &'static str = "substrate.retained_worker.native_bootstrap_completion.v1";

    /// Reserved for reports emitted by the native completion owner. Backends
    /// cannot supply this marker through their untrusted event payloads.
    pub const EVENT_DATA_KEY: &'static str = "retained_bootstrap_completion_v1";

    /// Decode closed report data and join its stream attribution. This validates
    /// data only; the consumer still authenticates original issuance and custody.
    /// ```
    /// use transport_api_types::{RetainedBootstrapCompletionV1, RuntimeFrameIdentityV1};
    /// use substrate_common::agent_events::{AgentEvent, MessageEventKind};
    /// let frame = RuntimeFrameIdentityV1 { schema_version: 1, stream_id: "rts_example".into(), frame_sequence: 1 };
    /// let event = AgentEvent::message("worker", "session", "run", MessageEventKind::Status, "working");
    /// assert!(RetainedBootstrapCompletionV1::from_service_event(&frame, &event).unwrap().is_none());
    /// ```
    pub fn from_service_event(
        frame: &RuntimeFrameIdentityV1,
        event: &substrate_common::agent_events::AgentEvent,
    ) -> Result<Option<Self>, String> {
        let Some(value) = event.data.get(Self::EVENT_DATA_KEY) else {
            return Ok(None);
        };
        let fail = || "invalid retained bootstrap service-event boundary".to_string();
        frame.validate()?;
        event.validate_identity_contract()?;
        let report: Self = serde_json::from_value(value.clone()).map_err(|_| fail())?;
        if event.data.as_object().map(|object| object.len()) != Some(1) {
            return Err(fail());
        }
        report.validate_service_boundary(frame, event)?;
        Ok(Some(report))
    }

    fn validate_service_boundary(
        &self,
        frame: &RuntimeFrameIdentityV1,
        event: &substrate_common::agent_events::AgentEvent,
    ) -> Result<(), String> {
        Self::validate_observation_service_boundary(&self.observation, frame, event)
    }

    fn validate_observation_service_boundary(
        observed: &RetainedBootstrapObservationV1,
        frame: &RuntimeFrameIdentityV1,
        event: &substrate_common::agent_events::AgentEvent,
    ) -> Result<(), String> {
        use substrate_common::agent_events::AgentEventKind;
        let fail = || "invalid retained bootstrap service-event boundary".to_string();
        frame.validate()?;
        event.validate_identity_contract()?;
        let identity = event.event_identity.as_ref().ok_or_else(fail)?;
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
        if event.kind != AgentEventKind::Status
            || event.worker_event.is_some()
            || event.orchestration_session_id != *orchestration_session_id
            || event.run_id != *run_id
            || event.participant_id.as_deref() != Some(participant_id.as_str())
            || event.backend_id.as_deref() != Some(backend_id.as_str())
            || event.role.as_deref() != Some("member")
            || event.world_id.as_deref() != Some(world_id.as_str())
            || event.world_generation != Some(*world_generation)
            || event.parent_participant_id != observed.activation.source_participant_id
            || event.resumed_from_participant_id.is_some()
            || event.span_id.as_deref() != Some(observed.span_id.as_str())
            || frame.stream_id != observed.frame_identity.stream_id
            || frame.frame_sequence >= observed.frame_identity.frame_sequence
            || identity.event_sequence >= observed.event_identity.event_sequence
        {
            return Err(fail());
        }
        Ok(())
    }

    /// Validate and encode reported data; this does not authenticate its producer.
    pub fn from_observation(observation: RetainedBootstrapObservationV1) -> Result<Self, String> {
        observation.validate()?;
        let sha256 = Self::digest(&observation)?;
        Ok(Self {
            schema_version: 1,
            domain: Self::DOMAIN.into(),
            observation,
            sha256,
        })
    }

    pub fn observation(&self) -> &RetainedBootstrapObservationV1 {
        &self.observation
    }

    /// Validate the closed codec and its separate commitment domain.
    pub fn validate(&self) -> Result<(), String> {
        if self.schema_version != 1 || self.domain != Self::DOMAIN {
            return Err("unsupported retained bootstrap completion version/domain".into());
        }
        self.observation.validate()?;
        if self.sha256 != Self::digest(&self.observation)? {
            return Err("retained bootstrap completion commitment mismatch".into());
        }
        Ok(())
    }

    /// Join this report to the exact received stream boundary. No permission is granted.
    pub fn validate_for_exit(
        &self,
        frame: &RuntimeFrameIdentityV1,
        event: &RuntimeEventIdentityV1,
        terminal: &RuntimeTerminalIdentityV1,
        span_id: &str,
        exit: i32,
    ) -> Result<(), String> {
        self.validate()?;
        let value = &self.observation;
        if exit != 0
            || value.frame_identity != *frame
            || value.event_identity != *event
            || value.terminal_identity != *terminal
            || value.span_id != span_id
        {
            return Err("retained bootstrap completion changed its native boundary".into());
        }
        Ok(())
    }

    /// Compare data against independently authenticated original source material.
    pub fn validate_for_source(
        &self,
        start: &OrdinaryStartSelectionCarrierV1,
        runtime: &OrdinaryGatewayRuntimeIdentityV1,
        activation: &E2MemberLaunchActivationCarrierV1,
    ) -> Result<(), String> {
        self.validate()?;
        if self.observation.source_start != *start
            || self.observation.runtime != *runtime
            || self.observation.activation != *activation
        {
            return Err(
                "retained bootstrap completion changed authenticated source material".into(),
            );
        }
        Ok(())
    }

    fn digest(observation: &RetainedBootstrapObservationV1) -> Result<String, String> {
        let mut hash = Sha256::new();
        hash.update(Self::DOMAIN.as_bytes());
        hash.update([0]);
        hash.update(1u32.to_be_bytes());
        hash.update(
            serde_json::to_vec(observation).map_err(|_| "encode retained bootstrap completion")?,
        );
        Ok(format!("{:x}", hash.finalize()))
    }
}

impl<'de> Deserialize<'de> for RetainedBootstrapCompletionV1 {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = RetainedBootstrapCompletionDef::deserialize(deserializer)?;
        let value = Self {
            schema_version: value.schema_version,
            domain: value.domain,
            observation: value.observation,
            sha256: value.sha256,
        };
        value.validate().map_err(serde::de::Error::custom)?;
        Ok(value)
    }
}

/// Native source/transfer observations made by the initial Fork completion owner.
/// These data confer neither permission nor retained registration authority.
/// ```
/// use transport_api_types::OrdinaryForkBootstrapProvenanceV1;
/// assert!(serde_json::from_str::<OrdinaryForkBootstrapProvenanceV1>("{}").is_err());
/// ```
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OrdinaryForkBootstrapProvenanceV1 {
    pub source_runtime: OrdinaryGatewayRuntimeIdentityV1,
    pub source_activation: E2MemberLaunchActivationCarrierV1,
    pub source_native_session_id: String,
    pub transferred_child_native_session_id: String,
}

impl OrdinaryForkBootstrapProvenanceV1 {
    fn validate_for_child(&self, child: &RetainedBootstrapObservationV1) -> Result<(), String> {
        use crate::E2MemberLaunchKindV1;
        self.source_runtime.validate()?;
        self.source_activation.validate()?;
        let source = &self.source_activation;
        let activation = &child.activation;
        let expected_source = OrdinaryGatewayRuntimeIdentityV1::WorldWorker {
            orchestration_session_id: source.orchestration_session_id.clone(),
            caller_participant_id: source.caller_participant_id.clone(),
            participant_id: source.retained_participant_id.clone(),
            backend_id: source.target_backend_id.clone(),
            run_id: source.bootstrap_run_id.clone(),
            world_id: source.target_world.world_id.clone(),
            world_generation: source.target_world.world_generation,
        };
        if activation.launch_kind != E2MemberLaunchKindV1::Fork
            || activation.source_participant_id.as_deref()
                != Some(source.retained_participant_id.as_str())
            || self.source_runtime != expected_source
            || source.commitment_ref.authority_store_id != child.source_start.authority_store_id
            || source.immutable_worker_cap_ref.authority_store_id
                != child.source_start.authority_store_id
            || source.orchestration_session_id != activation.orchestration_session_id
            || source.target_world != activation.target_world
            || source.target_backend_id != activation.target_backend_id
            || source.retained_participant_id == activation.retained_participant_id
            || source.bootstrap_run_id == activation.bootstrap_run_id
            || source.activation_id == activation.activation_id
            || source.immutable_worker_cap_ref == activation.immutable_worker_cap_ref
            || self.source_native_session_id.is_empty()
            || self.source_native_session_id.trim() != self.source_native_session_id
            || self.source_native_session_id == child.native_session_id
            || self.transferred_child_native_session_id != child.native_session_id
        {
            return Err("Fork bootstrap changed its native source/child transfer".into());
        }
        Ok(())
    }
}

/// One actual owned child wait. Unknown PID/status cannot form this successful
/// report. A known recovery wait never replaces an unknown primary wait.
/// ```
/// use transport_api_types::{NativeProcessWaitObservationV1, NativeProcessTerminationV1};
/// let wait = NativeProcessWaitObservationV1 { pid: 1,
///     termination: NativeProcessTerminationV1::Signaled { signal: 9 }, recovery_termination: None };
/// assert_eq!(wait.pid, 1);
/// ```
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NativeProcessWaitObservationV1 {
    pub pid: u32,
    pub termination: NativeProcessTerminationV1,
    pub recovery_termination: Option<NativeProcessTerminationV1>,
}

impl NativeProcessWaitObservationV1 {
    pub(crate) fn validate(&self) -> Result<(), String> {
        let valid = |status: &NativeProcessTerminationV1| match status {
            NativeProcessTerminationV1::Exited { exit_code } => *exit_code >= 0,
            NativeProcessTerminationV1::Signaled { signal } => *signal > 0,
        };
        if self.pid == 0
            || !valid(&self.termination)
            || self
                .recovery_termination
                .as_ref()
                .is_some_and(|status| !valid(status))
        {
            return Err("Fork bootstrap omitted an actual child wait".into());
        }
        Ok(())
    }
}

/// Strict initial Fork protocol success, separate from the physical process
/// waits. The observation's native termination is the first actual child wait,
/// which may be signaled; logical Exit remains zero. Spawn's codec is unchanged.
/// Consumers still authenticate the source cap, producer and physical custody.
/// ```
/// use transport_api_types::RetainedForkBootstrapCompletionV1;
/// assert!(serde_json::from_str::<RetainedForkBootstrapCompletionV1>("{}").is_err());
/// ```
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct RetainedForkBootstrapCompletionV1 {
    schema_version: u32,
    domain: String,
    observation: RetainedBootstrapObservationV1,
    provenance: OrdinaryForkBootstrapProvenanceV1,
    native_processes: Vec<NativeProcessWaitObservationV1>,
    sha256: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RetainedForkBootstrapCompletionDef {
    schema_version: u32,
    domain: String,
    observation: RetainedBootstrapObservationV1,
    provenance: OrdinaryForkBootstrapProvenanceV1,
    native_processes: Vec<NativeProcessWaitObservationV1>,
    sha256: String,
}

impl RetainedForkBootstrapCompletionV1 {
    pub const DOMAIN: &'static str =
        "substrate.retained_worker.native_fork_bootstrap_completion.v1";
    pub const EVENT_DATA_KEY: &'static str = "retained_fork_bootstrap_completion_v1";

    /// Encode validated reported data; this does not authenticate its producer.
    pub fn from_observation(
        observation: RetainedBootstrapObservationV1,
        provenance: OrdinaryForkBootstrapProvenanceV1,
        native_processes: Vec<NativeProcessWaitObservationV1>,
    ) -> Result<Self, String> {
        let mut report = Self {
            schema_version: 1,
            domain: Self::DOMAIN.into(),
            observation,
            provenance,
            native_processes,
            sha256: String::new(),
        };
        report.sha256 = report.digest()?;
        report.validate()?;
        Ok(report)
    }

    /// Decode the Fork report and join the original service stream boundary.
    /// Source authentication and custody remain the consumer's responsibility.
    pub fn from_service_event(
        frame: &RuntimeFrameIdentityV1,
        event: &substrate_common::agent_events::AgentEvent,
    ) -> Result<Option<Self>, String> {
        let Some(value) = event.data.get(Self::EVENT_DATA_KEY) else {
            return Ok(None);
        };
        if event.data.as_object().map(|object| object.len()) != Some(1) {
            return Err("invalid Fork bootstrap service-event boundary".into());
        }
        let report: Self = serde_json::from_value(value.clone())
            .map_err(|_| "invalid Fork bootstrap service-event report")?;
        RetainedBootstrapCompletionV1::validate_observation_service_boundary(
            &report.observation,
            frame,
            event,
        )?;
        Ok(Some(report))
    }

    /// The Fork completion stays a Fork report; it cannot become Spawn proof.
    pub fn completion(&self) -> &Self {
        self
    }
    pub fn observation(&self) -> &RetainedBootstrapObservationV1 {
        &self.observation
    }
    pub fn native_processes(&self) -> &[NativeProcessWaitObservationV1] {
        &self.native_processes
    }

    /// Join logical protocol success to its exact original stream Exit.
    pub fn validate_for_exit(
        &self,
        frame: &RuntimeFrameIdentityV1,
        event: &RuntimeEventIdentityV1,
        terminal: &RuntimeTerminalIdentityV1,
        span_id: &str,
        exit: i32,
    ) -> Result<(), String> {
        self.validate()?;
        let value = &self.observation;
        if exit != 0
            || value.frame_identity != *frame
            || value.event_identity != *event
            || value.terminal_identity != *terminal
            || value.span_id != span_id
        {
            return Err("Fork bootstrap changed its protocol success boundary".into());
        }
        Ok(())
    }
    pub fn provenance(&self) -> &OrdinaryForkBootstrapProvenanceV1 {
        &self.provenance
    }

    /// Validate closed data and its independent commitment. No authority is granted.
    pub fn validate(&self) -> Result<(), String> {
        if self.schema_version != 1 || self.domain != Self::DOMAIN {
            return Err("unsupported Fork bootstrap version/domain".into());
        }
        self.observation.validate_identity_and_cleanup()?;
        if self.native_processes.first().map(|wait| &wait.termination)
            != Some(&self.observation.native_termination)
        {
            return Err("Fork bootstrap omitted its actual primary process wait".into());
        }
        for wait in &self.native_processes {
            wait.validate()?;
        }
        self.provenance.validate_for_child(&self.observation)?;
        if self.sha256 != self.digest()? {
            return Err("Fork bootstrap commitment mismatch".into());
        }
        Ok(())
    }

    fn digest(&self) -> Result<String, String> {
        let mut hash = Sha256::new();
        hash.update(Self::DOMAIN.as_bytes());
        hash.update([0]);
        hash.update(1u32.to_be_bytes());
        hash.update(
            serde_json::to_vec(&(&self.observation, &self.provenance, &self.native_processes))
                .map_err(|_| "encode Fork bootstrap completion")?,
        );
        Ok(format!("{:x}", hash.finalize()))
    }
}

impl<'de> Deserialize<'de> for RetainedForkBootstrapCompletionV1 {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = RetainedForkBootstrapCompletionDef::deserialize(deserializer)?;
        let report = Self {
            schema_version: value.schema_version,
            domain: value.domain,
            observation: value.observation,
            provenance: value.provenance,
            native_processes: value.native_processes,
            sha256: value.sha256,
        };
        report.validate().map_err(serde::de::Error::custom)?;
        Ok(report)
    }
}
