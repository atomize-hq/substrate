use chrono::{DateTime, Utc};
use serde::{Deserialize, Deserializer, Serialize};

use crate::execution::config_model::AgentExecutionScope;

use super::mapping::{
    protocol_validation_error, MEMBER_ROLE, ORCHESTRATOR_ROLE, PURE_AGENT_PROTOCOL,
};
use super::orchestration_session::OrchestrationSessionRecord;
use super::validator::RuntimeSelectionDescriptor;

#[allow(dead_code)]
const CANCELLED_TERMINATION_REASON: &str = "cancelled";

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct AgentRuntimeParticipantExecution {
    pub scope: AgentExecutionScope,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum AgentRuntimeSessionState {
    Allocating,
    Ready,
    Running,
    Restarting,
    Stopping,
    Stopped,
    Failed,
    Invalidated,
}

impl AgentRuntimeSessionState {
    pub(crate) fn is_live(&self) -> bool {
        matches!(
            self,
            Self::Allocating | Self::Ready | Self::Running | Self::Restarting | Self::Stopping
        )
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum AgentRuntimeOwnershipMode {
    AttachedControl,
    MemberRuntime,
    Replaced,
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
pub(crate) struct AgentRuntimeParticipantHandle {
    // Canonical runtime lineage identifier for this participant row.
    pub participant_id: String,
    // Legacy compatibility alias kept in-memory for existing callers; do not use as canonical
    // public terminology in new code.
    #[serde(skip)]
    pub session_handle_id: String,
    pub orchestration_session_id: String,
    pub agent_id: String,
    pub backend_id: String,
    pub role: String,
    pub protocol: String,
    pub execution: AgentRuntimeParticipantExecution,
    pub state: AgentRuntimeSessionState,
    pub opened_at: DateTime<Utc>,
    pub last_transition_at: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub world_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub world_generation: Option<u64>,
    // Lineage links use participant_id canonically. The *_session_handle_id fields remain as
    // compatibility mirrors for legacy reads only.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_participant_id: Option<String>,
    #[serde(skip)]
    pub parent_session_handle_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resumed_from_participant_id: Option<String>,
    #[serde(skip)]
    pub resumed_from_session_handle_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub orchestrator_participant_id: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct AgentRuntimeSessionInternal {
    pub resolved_agent_kind: String,
    pub resolved_binary_path: String,
    pub shell_owner_pid: u32,
    pub lease_token: String,
    // Backend-native upstream runtime handle. Internal only; never the default operator target.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uaa_session_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub latest_run_id: Option<String>,
    pub cancel_supported: bool,
    // A participant record is only authoritative-live while the REPL still retains the attached
    // UAA control boundary: the cancel handle remains owned, the event stream task is
    // still active, and the completion observer is still retained.
    #[serde(default)]
    pub control_owner_retained: bool,
    #[serde(default)]
    pub event_stream_active: bool,
    #[serde(default)]
    pub completion_observer_retained: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ownership_mode: Option<AgentRuntimeOwnershipMode>,
    #[serde(default)]
    pub ownership_valid: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ownership_verified_at: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_heartbeat_at: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_event_at: Option<DateTime<Utc>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub terminal_observed_at: Option<DateTime<Utc>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub termination_reason: Option<String>,
    #[serde(default)]
    pub attached_client_present: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_attached_at: Option<DateTime<Utc>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_detached_at: Option<DateTime<Utc>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub detach_reason: Option<String>,
    #[serde(default)]
    pub resume_eligible: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_error_bucket: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_error_message: Option<String>,
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
pub(crate) struct AgentRuntimeParticipantRecord {
    #[serde(flatten)]
    pub handle: AgentRuntimeParticipantHandle,
    pub internal: AgentRuntimeSessionInternal,
}

impl<'de> Deserialize<'de> for AgentRuntimeParticipantRecord {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let wire = AgentRuntimeParticipantRecordWire::deserialize(deserializer)?;
        Self::try_from(wire).map_err(serde::de::Error::custom)
    }
}

impl TryFrom<AgentRuntimeParticipantRecordWire> for AgentRuntimeParticipantRecord {
    type Error = String;

    fn try_from(wire: AgentRuntimeParticipantRecordWire) -> Result<Self, Self::Error> {
        let handle = AgentRuntimeParticipantHandle {
            session_handle_id: wire.handle.participant_id.clone(),
            participant_id: wire.handle.participant_id,
            orchestration_session_id: wire.handle.orchestration_session_id,
            agent_id: wire.handle.agent_id,
            backend_id: wire.handle.backend_id,
            role: wire.handle.role,
            protocol: wire.handle.protocol,
            execution: AgentRuntimeParticipantExecution {
                scope: wire.handle.execution.scope,
            },
            state: wire.handle.state,
            opened_at: wire.handle.opened_at,
            last_transition_at: wire.handle.last_transition_at,
            world_id: wire.handle.world_id,
            world_generation: wire.handle.world_generation,
            parent_session_handle_id: wire.handle.parent_participant_id.clone(),
            parent_participant_id: wire.handle.parent_participant_id,
            resumed_from_session_handle_id: wire.handle.resumed_from_participant_id.clone(),
            resumed_from_participant_id: wire.handle.resumed_from_participant_id,
            orchestrator_participant_id: wire.handle.orchestrator_participant_id,
        };
        let record = Self {
            handle,
            internal: wire.internal,
        };
        record.validate().map_err(|err| err.to_string())?;
        Ok(record)
    }
}

struct AgentRuntimeParticipantLineage {
    parent_participant_id: Option<String>,
    resumed_from_participant_id: Option<String>,
    orchestrator_participant_id: Option<String>,
}

struct AgentRuntimeParticipantInit {
    orchestration_session_id: String,
    participant_id: String,
    role: String,
    lease_token: String,
    lineage: AgentRuntimeParticipantLineage,
    world: Option<AgentRuntimeParticipantWorldBinding>,
    ownership_mode: AgentRuntimeOwnershipMode,
}

#[allow(dead_code)]
pub(crate) struct AgentRuntimeReplacementParticipantInit {
    pub orchestration_session_id: String,
    pub participant_id: String,
    pub role: String,
    pub orchestrator_participant_id: Option<String>,
    pub parent_participant_id: Option<String>,
    pub resumed_from_participant_id: String,
    pub world: Option<AgentRuntimeParticipantWorldBinding>,
    pub lease_token: String,
}

impl AgentRuntimeParticipantRecord {
    pub(crate) fn new(
        descriptor: &RuntimeSelectionDescriptor,
        orchestration_session_id: String,
        participant_id: String,
        lease_token: String,
    ) -> Self {
        Self::new_orchestrator_participant(
            descriptor,
            orchestration_session_id,
            participant_id,
            lease_token,
        )
        .expect("orchestrator participants must satisfy model invariants")
    }

    pub(crate) fn new_orchestrator_participant(
        descriptor: &RuntimeSelectionDescriptor,
        orchestration_session_id: String,
        participant_id: String,
        lease_token: String,
    ) -> anyhow::Result<Self> {
        Self::build_participant(
            descriptor,
            AgentRuntimeParticipantInit {
                orchestration_session_id,
                participant_id,
                role: ORCHESTRATOR_ROLE.to_string(),
                lease_token,
                lineage: AgentRuntimeParticipantLineage {
                    parent_participant_id: None,
                    resumed_from_participant_id: None,
                    orchestrator_participant_id: None,
                },
                world: None,
                ownership_mode: AgentRuntimeOwnershipMode::AttachedControl,
            },
        )
    }

    #[allow(dead_code)]
    pub(crate) fn new_member_participant(
        descriptor: &RuntimeSelectionDescriptor,
        orchestration_session_id: String,
        participant_id: String,
        orchestrator_participant_id: String,
        parent_participant_id: Option<String>,
        world: Option<AgentRuntimeParticipantWorldBinding>,
        lease_token: String,
    ) -> anyhow::Result<Self> {
        Self::build_participant(
            descriptor,
            AgentRuntimeParticipantInit {
                orchestration_session_id,
                participant_id,
                role: MEMBER_ROLE.to_string(),
                lease_token,
                lineage: AgentRuntimeParticipantLineage {
                    parent_participant_id,
                    resumed_from_participant_id: None,
                    orchestrator_participant_id: Some(orchestrator_participant_id),
                },
                world,
                ownership_mode: AgentRuntimeOwnershipMode::MemberRuntime,
            },
        )
    }

    #[allow(dead_code)]
    pub(crate) fn new_replacement_participant(
        descriptor: &RuntimeSelectionDescriptor,
        init: AgentRuntimeReplacementParticipantInit,
    ) -> anyhow::Result<Self> {
        let ownership_mode = if init.role == ORCHESTRATOR_ROLE {
            AgentRuntimeOwnershipMode::AttachedControl
        } else {
            AgentRuntimeOwnershipMode::MemberRuntime
        };

        Self::build_participant(
            descriptor,
            AgentRuntimeParticipantInit {
                orchestration_session_id: init.orchestration_session_id,
                participant_id: init.participant_id,
                role: init.role,
                lease_token: init.lease_token,
                lineage: AgentRuntimeParticipantLineage {
                    parent_participant_id: init.parent_participant_id,
                    resumed_from_participant_id: Some(init.resumed_from_participant_id),
                    orchestrator_participant_id: init.orchestrator_participant_id,
                },
                world: init.world,
                ownership_mode,
            },
        )
    }

    fn build_participant(
        descriptor: &RuntimeSelectionDescriptor,
        init: AgentRuntimeParticipantInit,
    ) -> anyhow::Result<Self> {
        let now = Utc::now();
        let world_id = init.world.as_ref().map(|binding| binding.world_id.clone());
        let world_generation = init.world.map(|binding| binding.world_generation);
        let host_attachment_contract = init.role == ORCHESTRATOR_ROLE
            && descriptor.execution_scope == AgentExecutionScope::Host;
        let record = Self {
            handle: AgentRuntimeParticipantHandle {
                session_handle_id: init.participant_id.clone(),
                participant_id: init.participant_id,
                orchestration_session_id: init.orchestration_session_id,
                agent_id: descriptor.agent_id.clone(),
                backend_id: descriptor.backend_id.clone(),
                role: init.role,
                protocol: descriptor.protocol.clone(),
                execution: AgentRuntimeParticipantExecution {
                    scope: descriptor.execution_scope,
                },
                state: AgentRuntimeSessionState::Allocating,
                opened_at: now,
                last_transition_at: now,
                world_id,
                world_generation,
                parent_session_handle_id: init.lineage.parent_participant_id.clone(),
                parent_participant_id: init.lineage.parent_participant_id,
                resumed_from_session_handle_id: init.lineage.resumed_from_participant_id.clone(),
                resumed_from_participant_id: init.lineage.resumed_from_participant_id,
                orchestrator_participant_id: init.lineage.orchestrator_participant_id,
            },
            internal: AgentRuntimeSessionInternal {
                resolved_agent_kind: descriptor.backend_kind.as_agent_kind_str().to_string(),
                resolved_binary_path: descriptor.binary_path.display().to_string(),
                shell_owner_pid: std::process::id(),
                lease_token: init.lease_token,
                uaa_session_id: None,
                latest_run_id: None,
                cancel_supported: true,
                control_owner_retained: false,
                event_stream_active: false,
                completion_observer_retained: false,
                ownership_mode: Some(init.ownership_mode),
                ownership_valid: false,
                ownership_verified_at: None,
                last_heartbeat_at: Some(now),
                last_event_at: None,
                terminal_observed_at: None,
                termination_reason: None,
                attached_client_present: host_attachment_contract,
                last_attached_at: host_attachment_contract.then_some(now),
                last_detached_at: None,
                detach_reason: None,
                resume_eligible: host_attachment_contract,
                last_error_bucket: None,
                last_error_message: None,
            },
        };
        record.validate()?;
        Ok(record)
    }

    pub(crate) fn validate(&self) -> anyhow::Result<()> {
        if self.handle.participant_id.trim().is_empty() {
            anyhow::bail!("participant_id must not be empty");
        }
        if self.handle.orchestration_session_id.trim().is_empty() {
            anyhow::bail!("orchestration_session_id must not be empty");
        }
        if self.handle.protocol != PURE_AGENT_PROTOCOL {
            anyhow::bail!(
                "participant record {}",
                protocol_validation_error("it", Some(self.handle.protocol.as_str()))
            );
        }

        let world_fields_present =
            self.handle.world_id.is_some() || self.handle.world_generation.is_some();
        let world_fields_complete =
            self.handle.world_id.is_some() && self.handle.world_generation.is_some();
        if world_fields_present && !world_fields_complete {
            anyhow::bail!(
                "world-scoped participant metadata must include both world_id and world_generation"
            );
        }

        match self.handle.execution.scope {
            AgentExecutionScope::Host if world_fields_present => {
                anyhow::bail!("host-scoped participants must omit world_id and world_generation");
            }
            AgentExecutionScope::World if !world_fields_complete => {
                anyhow::bail!(
                    "world-scoped participants must include world_id and world_generation"
                );
            }
            _ => {}
        }

        match self.handle.role.as_str() {
            ORCHESTRATOR_ROLE => {
                if self.handle.execution.scope != AgentExecutionScope::Host {
                    anyhow::bail!("orchestrator participants must use execution.scope=host");
                }
                if self.handle.orchestrator_participant_id.is_some() {
                    anyhow::bail!(
                        "orchestrator participants must omit orchestrator_participant_id"
                    );
                }
            }
            MEMBER_ROLE => {
                let orchestrator_participant_id = self
                    .handle
                    .orchestrator_participant_id
                    .as_deref()
                    .filter(|value| !value.trim().is_empty())
                    .ok_or_else(|| {
                        anyhow::anyhow!(
                            "member participants must include orchestrator_participant_id"
                        )
                    })?;
                if orchestrator_participant_id == self.handle.participant_id {
                    anyhow::bail!(
                        "member participants must not reuse participant_id as orchestrator_participant_id"
                    );
                }
            }
            other => anyhow::bail!("unsupported participant role '{other}'"),
        }

        let uses_host_attachment_contract = self.is_host_orchestrator();
        let host_attachment_fields_set = self.internal.attached_client_present
            || self.internal.last_attached_at.is_some()
            || self.internal.last_detached_at.is_some()
            || self.internal.detach_reason.is_some()
            || self.internal.resume_eligible;
        if !uses_host_attachment_contract && host_attachment_fields_set {
            anyhow::bail!(
                "non-host participants must not set attached_client_present, resume_eligible, or detach timestamps"
            );
        }
        if uses_host_attachment_contract {
            if self.internal.attached_client_present && self.internal.last_attached_at.is_none() {
                anyhow::bail!("attached host participants must include internal.last_attached_at");
            }
            if self.internal.attached_client_present && !self.internal.resume_eligible {
                anyhow::bail!("attached host participants must remain resume_eligible");
            }
            if self.internal.resume_eligible && self.internal.last_attached_at.is_none() {
                anyhow::bail!(
                    "resume-eligible host participants must record internal.last_attached_at"
                );
            }
            if self.internal.terminal_observed_at.is_some() && self.internal.resume_eligible {
                anyhow::bail!("terminal host participants must clear resume_eligible");
            }
        }

        if self.handle.parent_participant_id.as_deref() == Some(self.handle.participant_id.as_str())
        {
            anyhow::bail!("parent_participant_id must not point to the participant itself");
        }
        if self.handle.resumed_from_participant_id.as_deref()
            == Some(self.handle.participant_id.as_str())
        {
            anyhow::bail!("resumed_from_participant_id must not point to the participant itself");
        }

        Ok(())
    }

    pub(crate) fn transition_state(&mut self, next: AgentRuntimeSessionState) {
        self.handle.state = next;
        self.handle.last_transition_at = Utc::now();
    }

    pub(crate) fn invalidate_for_world_generation_rollover(&mut self) -> bool {
        if self.handle.state == AgentRuntimeSessionState::Invalidated {
            return false;
        }

        self.mark_terminal_state("world generation invalidated by replacement binding");
        self.transition_state(AgentRuntimeSessionState::Invalidated);
        true
    }

    pub(crate) fn touch_heartbeat(&mut self) {
        self.internal.last_heartbeat_at = Some(Utc::now());
    }

    pub(crate) fn touch_event(&mut self, ts: DateTime<Utc>) {
        self.internal.last_event_at = Some(ts);
    }

    pub(crate) fn participant_id(&self) -> &str {
        &self.handle.participant_id
    }

    pub(crate) fn internal_uaa_session_id(&self) -> Option<&str> {
        self.internal.uaa_session_id.as_deref()
    }

    pub(crate) fn is_host_orchestrator(&self) -> bool {
        self.handle.role == ORCHESTRATOR_ROLE
            && self.handle.execution.scope == AgentExecutionScope::Host
    }

    pub(crate) fn matches_public_parent_linkage(
        &self,
        session: &OrchestrationSessionRecord,
    ) -> bool {
        self.handle.orchestration_session_id == session.orchestration_session_id
            && self.handle.agent_id == session.orchestrator_agent_id
            && self.is_host_orchestrator()
    }

    pub(crate) fn matches_authoritative_parent_world_binding(
        &self,
        session: &OrchestrationSessionRecord,
    ) -> bool {
        let Some(world_id) = session.world_id.as_deref() else {
            return false;
        };
        let Some(world_generation) = session.world_generation else {
            return false;
        };

        self.handle.orchestration_session_id == session.orchestration_session_id
            && self.handle.role == MEMBER_ROLE
            && self.handle.execution.scope == AgentExecutionScope::World
            && self.handle.world_id.as_deref() == Some(world_id)
            && self.handle.world_generation == Some(world_generation)
    }

    pub(crate) fn set_uaa_session_id(&mut self, backend_session_id: impl Into<String>) {
        self.internal.uaa_session_id = Some(backend_session_id.into());
        if self.is_host_orchestrator() && self.handle.state.is_live() {
            self.internal.resume_eligible = true;
            self.internal.last_attached_at.get_or_insert_with(Utc::now);
        }
        self.refresh_ownership_validity();
    }

    pub(crate) fn set_event_stream_active(&mut self, active: bool) {
        self.internal.event_stream_active = active;
        self.refresh_ownership_validity();
    }

    pub(crate) fn can_advertise_live(&self) -> bool {
        self.internal.uaa_session_id.is_some()
            && self.internal.control_owner_retained
            && self.internal.event_stream_active
            && self.internal.completion_observer_retained
            && self.internal.terminal_observed_at.is_none()
    }

    fn refresh_ownership_validity(&mut self) {
        let now_live = self.can_advertise_live();
        let was_live = self.internal.ownership_valid;
        self.internal.ownership_valid = now_live;
        if now_live {
            if !was_live {
                self.internal.ownership_verified_at = Some(Utc::now());
            }
            self.internal.terminal_observed_at = None;
            self.internal.termination_reason = None;
        }
    }

    pub(crate) fn mark_runtime_ownership_retained(&mut self) {
        self.internal.control_owner_retained = true;
        self.internal.event_stream_active = true;
        self.internal.completion_observer_retained = true;
        self.mark_client_attached();
        self.refresh_ownership_validity();
    }

    pub(crate) fn release_runtime_ownership(&mut self) {
        self.internal.control_owner_retained = false;
        self.internal.event_stream_active = false;
        self.internal.completion_observer_retained = false;
        self.internal.ownership_valid = false;
    }

    pub(crate) fn mark_terminal_state(&mut self, reason: impl Into<String>) {
        let reason = reason.into();
        let now = Utc::now();
        self.release_runtime_ownership();
        self.internal.terminal_observed_at = Some(now);
        self.internal.termination_reason = Some(reason.clone());
        if self.is_host_orchestrator() {
            self.internal.attached_client_present = false;
            self.internal.resume_eligible = false;
            self.internal.last_detached_at = Some(now);
            self.internal.detach_reason = Some(reason);
        }
        if self.internal.ownership_verified_at.is_none() {
            self.internal.ownership_verified_at = Some(now);
        }
    }

    #[allow(dead_code)]
    pub(crate) fn mark_cancelled_terminal_state(&mut self) {
        self.mark_terminal_state(CANCELLED_TERMINATION_REASON);
        self.transition_state(AgentRuntimeSessionState::Invalidated);
        self.internal.latest_run_id = None;
    }

    #[allow(dead_code)]
    pub(crate) fn has_cancelled_terminal_truth(&self) -> bool {
        self.internal.terminal_observed_at.is_some()
            && self.internal.termination_reason.as_deref() == Some(CANCELLED_TERMINATION_REASON)
    }

    #[allow(dead_code)]
    pub(crate) fn reviewable_terminal_state_label(&self) -> &'static str {
        if self.has_cancelled_terminal_truth() {
            return CANCELLED_TERMINATION_REASON;
        }

        match self.handle.state {
            AgentRuntimeSessionState::Stopped => "stopped",
            AgentRuntimeSessionState::Failed => "failed",
            AgentRuntimeSessionState::Invalidated => "invalidated",
            AgentRuntimeSessionState::Allocating
            | AgentRuntimeSessionState::Ready
            | AgentRuntimeSessionState::Running
            | AgentRuntimeSessionState::Restarting
            | AgentRuntimeSessionState::Stopping => "terminal",
        }
    }

    pub(crate) fn has_valid_ownership(&self) -> bool {
        self.internal.ownership_valid && self.can_advertise_live()
    }

    pub(crate) fn is_authoritative_live(&self) -> bool {
        self.handle.state.is_live() && self.has_valid_ownership()
    }

    pub(crate) fn attached_client_present(&self) -> bool {
        self.internal.attached_client_present
    }

    pub(crate) fn is_resume_eligible(&self) -> bool {
        self.internal.resume_eligible
            && self.is_host_orchestrator()
            && self.handle.state.is_live()
            && self.internal.terminal_observed_at.is_none()
    }

    #[allow(dead_code)]
    pub(crate) fn mark_client_detached(&mut self, reason: impl Into<String>) {
        if !self.is_host_orchestrator() {
            return;
        }
        self.internal.attached_client_present = false;
        self.internal.last_detached_at = Some(Utc::now());
        self.internal.detach_reason = Some(reason.into());
        self.internal.resume_eligible = self.handle.state.is_live()
            && self.internal.terminal_observed_at.is_none()
            && self.internal.uaa_session_id.is_some();
    }

    pub(crate) fn last_status_at(&self) -> DateTime<Utc> {
        self.internal
            .last_event_at
            .or(self.internal.last_heartbeat_at)
            .unwrap_or(self.handle.last_transition_at)
    }

    fn mark_client_attached(&mut self) {
        if !self.is_host_orchestrator() {
            return;
        }
        self.internal.attached_client_present = true;
        self.internal.resume_eligible = true;
        self.internal.last_attached_at = Some(Utc::now());
        self.internal.detach_reason = None;
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct AgentRuntimeParticipantWorldBinding {
    pub world_id: String,
    pub world_generation: u64,
}

#[derive(Deserialize)]
struct AgentRuntimeParticipantRecordWire {
    #[serde(flatten)]
    handle: AgentRuntimeParticipantHandleWire,
    internal: AgentRuntimeSessionInternal,
}

#[derive(Deserialize)]
struct AgentRuntimeParticipantHandleWire {
    // Legacy alias reads remain supported for compatibility; participant_id is canonical.
    #[serde(alias = "session_handle_id")]
    participant_id: String,
    orchestration_session_id: String,
    agent_id: String,
    backend_id: String,
    role: String,
    protocol: String,
    execution: AgentRuntimeParticipantExecutionWire,
    state: AgentRuntimeSessionState,
    opened_at: DateTime<Utc>,
    last_transition_at: DateTime<Utc>,
    #[serde(default)]
    world_id: Option<String>,
    #[serde(default)]
    world_generation: Option<u64>,
    #[serde(default, alias = "parent_session_handle_id")]
    parent_participant_id: Option<String>,
    #[serde(default, alias = "resumed_from_session_handle_id")]
    resumed_from_participant_id: Option<String>,
    #[serde(default)]
    orchestrator_participant_id: Option<String>,
}

#[derive(Deserialize)]
struct AgentRuntimeParticipantExecutionWire {
    scope: AgentExecutionScope,
}

#[allow(dead_code)]
pub(crate) type AgentRuntimeSessionExecution = AgentRuntimeParticipantExecution;
#[allow(dead_code)]
pub(crate) type AgentRuntimeSessionHandle = AgentRuntimeParticipantHandle;
pub(crate) type AgentRuntimeSessionManifest = AgentRuntimeParticipantRecord;

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use serde_json::json;

    use super::*;
    use crate::execution::agent_runtime::mapping::{
        AgentRuntimeBackendKind, LEGACY_PURE_AGENT_PROTOCOL,
    };

    fn descriptor(scope: AgentExecutionScope) -> RuntimeSelectionDescriptor {
        RuntimeSelectionDescriptor {
            agent_id: "codex".to_string(),
            backend_id: "cli:codex".to_string(),
            backend_kind: AgentRuntimeBackendKind::Codex,
            protocol: "substrate.agent.session".to_string(),
            execution_scope: scope,
            binary_path: PathBuf::from("/usr/bin/codex"),
        }
    }

    #[test]
    fn orchestrator_participant_constructor() {
        let participant = AgentRuntimeParticipantRecord::new_orchestrator_participant(
            &descriptor(AgentExecutionScope::Host),
            "sess_001".to_string(),
            "ash_001".to_string(),
            "lease_001".to_string(),
        )
        .expect("orchestrator constructor should succeed");

        assert_eq!(participant.handle.participant_id, "ash_001");
        assert_eq!(participant.handle.session_handle_id, "ash_001");
        assert_eq!(participant.handle.role, ORCHESTRATOR_ROLE);
        assert_eq!(
            participant.handle.execution.scope,
            AgentExecutionScope::Host
        );
        assert!(participant.handle.world_id.is_none());
        assert!(participant.handle.world_generation.is_none());
        assert!(participant.handle.orchestrator_participant_id.is_none());
        assert_eq!(
            participant.internal.ownership_mode,
            Some(AgentRuntimeOwnershipMode::AttachedControl)
        );
    }

    #[test]
    fn member_participant_constructor() {
        let participant = AgentRuntimeParticipantRecord::new_member_participant(
            &descriptor(AgentExecutionScope::World),
            "sess_001".to_string(),
            "ash_002".to_string(),
            "ash_orchestrator".to_string(),
            Some("ash_parent".to_string()),
            Some(AgentRuntimeParticipantWorldBinding {
                world_id: "world-17".to_string(),
                world_generation: 3,
            }),
            "lease_002".to_string(),
        )
        .expect("member constructor should succeed");

        assert_eq!(participant.handle.participant_id, "ash_002");
        assert_eq!(participant.handle.role, MEMBER_ROLE);
        assert_eq!(
            participant.handle.orchestrator_participant_id.as_deref(),
            Some("ash_orchestrator")
        );
        assert_eq!(
            participant.handle.parent_participant_id.as_deref(),
            Some("ash_parent")
        );
        assert_eq!(participant.handle.world_id.as_deref(), Some("world-17"));
        assert_eq!(participant.handle.world_generation, Some(3));
        assert_eq!(
            participant.internal.ownership_mode,
            Some(AgentRuntimeOwnershipMode::MemberRuntime)
        );
    }

    #[test]
    fn replacement_participant_constructor() {
        let participant = AgentRuntimeParticipantRecord::new_replacement_participant(
            &descriptor(AgentExecutionScope::World),
            AgentRuntimeReplacementParticipantInit {
                orchestration_session_id: "sess_001".to_string(),
                participant_id: "ash_003".to_string(),
                role: MEMBER_ROLE.to_string(),
                orchestrator_participant_id: Some("ash_orchestrator".to_string()),
                parent_participant_id: Some("ash_parent".to_string()),
                resumed_from_participant_id: "ash_002".to_string(),
                world: Some(AgentRuntimeParticipantWorldBinding {
                    world_id: "world-18".to_string(),
                    world_generation: 4,
                }),
                lease_token: "lease_003".to_string(),
            },
        )
        .expect("replacement constructor should succeed");

        assert_eq!(
            participant.handle.resumed_from_participant_id.as_deref(),
            Some("ash_002")
        );
        assert_eq!(
            participant.handle.resumed_from_session_handle_id.as_deref(),
            Some("ash_002")
        );
        assert_eq!(
            participant.internal.ownership_mode,
            Some(AgentRuntimeOwnershipMode::MemberRuntime)
        );
    }

    #[test]
    fn invalidation_helper_marks_world_generation_rollover_tombstone() {
        let mut participant = AgentRuntimeParticipantRecord::new_member_participant(
            &descriptor(AgentExecutionScope::World),
            "sess_001".to_string(),
            "ash_002".to_string(),
            "ash_orchestrator".to_string(),
            None,
            Some(AgentRuntimeParticipantWorldBinding {
                world_id: "world-17".to_string(),
                world_generation: 3,
            }),
            "lease_002".to_string(),
        )
        .expect("member constructor should succeed");
        participant.mark_runtime_ownership_retained();
        participant.set_uaa_session_id("uaa_session");
        participant.transition_state(AgentRuntimeSessionState::Running);

        let changed = participant.invalidate_for_world_generation_rollover();

        assert!(changed, "invalidation helper must report first transition");
        assert_eq!(
            participant.handle.state,
            AgentRuntimeSessionState::Invalidated
        );
        assert!(!participant.internal.ownership_valid);
        assert!(!participant.can_advertise_live());
        assert_eq!(
            participant.internal.termination_reason.as_deref(),
            Some("world generation invalidated by replacement binding")
        );
        assert!(
            participant.internal.terminal_observed_at.is_some(),
            "invalidation should stamp terminal metadata"
        );
    }

    #[test]
    fn invalidation_helper_is_idempotent() {
        let mut participant = AgentRuntimeParticipantRecord::new_member_participant(
            &descriptor(AgentExecutionScope::World),
            "sess_001".to_string(),
            "ash_002".to_string(),
            "ash_orchestrator".to_string(),
            None,
            Some(AgentRuntimeParticipantWorldBinding {
                world_id: "world-17".to_string(),
                world_generation: 3,
            }),
            "lease_002".to_string(),
        )
        .expect("member constructor should succeed");

        assert!(participant.invalidate_for_world_generation_rollover());
        let first_transition_at = participant.handle.last_transition_at;
        let first_terminal_observed_at = participant.internal.terminal_observed_at;

        assert!(
            !participant.invalidate_for_world_generation_rollover(),
            "already invalidated participants must remain untouched"
        );
        assert_eq!(participant.handle.last_transition_at, first_transition_at);
        assert_eq!(
            participant.internal.terminal_observed_at,
            first_terminal_observed_at
        );
    }

    #[test]
    fn host_participant_constructor_tracks_attachment_fields() {
        let participant = AgentRuntimeParticipantRecord::new_orchestrator_participant(
            &descriptor(AgentExecutionScope::Host),
            "sess_001".to_string(),
            "ash_001".to_string(),
            "lease_001".to_string(),
        )
        .expect("host participant");

        assert!(participant.attached_client_present());
        assert!(participant.is_resume_eligible());
        assert!(participant.internal.last_attached_at.is_some());
    }

    #[test]
    fn mark_cancelled_terminal_state_surfaces_explicit_cancelled_truth() {
        let mut participant = AgentRuntimeParticipantRecord::new_member_participant(
            &descriptor(AgentExecutionScope::World),
            "sess_001".to_string(),
            "ash_002".to_string(),
            "ash_orchestrator".to_string(),
            None,
            Some(AgentRuntimeParticipantWorldBinding {
                world_id: "world-17".to_string(),
                world_generation: 3,
            }),
            "lease_002".to_string(),
        )
        .expect("member constructor should succeed");
        participant.mark_runtime_ownership_retained();
        participant.set_uaa_session_id("uaa_session");
        participant.transition_state(AgentRuntimeSessionState::Running);
        participant.internal.latest_run_id = Some("run-cancel".to_string());

        participant.mark_cancelled_terminal_state();

        assert_eq!(
            participant.handle.state,
            AgentRuntimeSessionState::Invalidated
        );
        assert!(participant.has_cancelled_terminal_truth());
        assert_eq!(participant.reviewable_terminal_state_label(), "cancelled");
        assert_eq!(participant.internal.termination_reason.as_deref(), Some("cancelled"));
        assert!(participant.internal.terminal_observed_at.is_some());
        assert_eq!(participant.internal.latest_run_id, None);
    }

    #[test]
    fn member_participant_rejects_host_attachment_fields() {
        let mut participant = AgentRuntimeParticipantRecord::new_member_participant(
            &descriptor(AgentExecutionScope::World),
            "sess_001".to_string(),
            "member_001".to_string(),
            "ash_001".to_string(),
            None,
            Some(AgentRuntimeParticipantWorldBinding {
                world_id: "world-17".to_string(),
                world_generation: 2,
            }),
            "lease_001".to_string(),
        )
        .expect("member participant");

        participant.internal.resume_eligible = true;
        let err = participant
            .validate()
            .expect_err("attachment contract error");
        assert!(err
            .to_string()
            .contains("non-host participants must not set attached_client_present"));
    }

    #[test]
    fn legacy_handle_upgrade() {
        let participant: AgentRuntimeParticipantRecord = serde_json::from_value(json!({
            "session_handle_id": "ash_legacy",
            "orchestration_session_id": "sess_legacy",
            "agent_id": "codex",
            "backend_id": "cli:codex",
            "role": "orchestrator",
            "protocol": "substrate.agent.session",
            "execution": { "scope": "host" },
            "state": "allocating",
            "opened_at": "2026-04-24T18:30:00Z",
            "last_transition_at": "2026-04-24T18:30:00Z",
            "parent_session_handle_id": "ash_parent",
            "resumed_from_session_handle_id": "ash_prev",
            "internal": {
                "resolved_agent_kind": "codex",
                "resolved_binary_path": "/usr/bin/codex",
                "shell_owner_pid": 42,
                "lease_token": "lease_legacy",
                "cancel_supported": true,
                "control_owner_retained": false,
                "event_stream_active": false,
                "completion_observer_retained": false,
                "ownership_mode": "attached_control",
                "ownership_valid": false
            }
        }))
        .expect("legacy handle JSON should deserialize");

        assert_eq!(participant.handle.participant_id, "ash_legacy");
        assert_eq!(participant.handle.session_handle_id, "ash_legacy");
        assert_eq!(
            participant.handle.parent_participant_id.as_deref(),
            Some("ash_parent")
        );
        assert_eq!(
            participant.handle.resumed_from_participant_id.as_deref(),
            Some("ash_prev")
        );
        assert!(participant.handle.orchestrator_participant_id.is_none());
    }

    #[test]
    fn centralized_role_scope_world_validation() {
        let err = AgentRuntimeParticipantRecord::new_member_participant(
            &descriptor(AgentExecutionScope::World),
            "sess_001".to_string(),
            "ash_invalid".to_string(),
            "ash_orchestrator".to_string(),
            None,
            None,
            "lease_invalid".to_string(),
        )
        .expect_err("world-scoped members without world binding must be rejected");
        assert!(err
            .to_string()
            .contains("world-scoped participants must include world_id and world_generation"));

        let err = serde_json::from_value::<AgentRuntimeParticipantRecord>(json!({
            "participant_id": "ash_invalid",
            "orchestration_session_id": "sess_001",
            "agent_id": "codex",
            "backend_id": "cli:codex",
            "role": "orchestrator",
            "protocol": "substrate.agent.session",
            "execution": { "scope": "world" },
            "state": "allocating",
            "opened_at": "2026-04-24T18:30:00Z",
            "last_transition_at": "2026-04-24T18:30:00Z",
            "world_id": "world-17",
            "world_generation": 3,
            "internal": {
                "resolved_agent_kind": "codex",
                "resolved_binary_path": "/usr/bin/codex",
                "shell_owner_pid": 42,
                "lease_token": "lease_invalid",
                "cancel_supported": true,
                "control_owner_retained": false,
                "event_stream_active": false,
                "completion_observer_retained": false,
                "ownership_mode": "attached_control",
                "ownership_valid": false
            }
        }))
        .expect_err("orchestrator world rows must be rejected");
        assert!(err
            .to_string()
            .contains("orchestrator participants must use execution.scope=host"));
    }

    #[test]
    fn deserialize_rejects_legacy_protocol_row() {
        let payload = json!({
            "participant_id": "ash_001",
            "orchestration_session_id": "sess_001",
            "agent_id": "codex",
            "backend_id": "cli:codex",
            "role": ORCHESTRATOR_ROLE,
            "protocol": LEGACY_PURE_AGENT_PROTOCOL,
            "execution": { "scope": "host" },
            "state": "ready",
            "opened_at": "2026-05-21T00:00:00Z",
            "last_transition_at": "2026-05-21T00:00:00Z",
            "internal": {
                "resolved_agent_kind": "codex",
                "resolved_binary_path": "/usr/bin/codex",
                "shell_owner_pid": 1,
                "lease_token": "lease_001",
                "cancel_supported": true,
                "control_owner_retained": false,
                "event_stream_active": false,
                "completion_observer_retained": false,
                "ownership_valid": false,
                "attached_client_present": false,
                "resume_eligible": false
            }
        });

        let error = serde_json::from_value::<AgentRuntimeParticipantRecord>(payload)
            .expect_err("legacy protocol rows must fail closed");
        assert!(
            error.to_string().contains(LEGACY_PURE_AGENT_PROTOCOL)
                && error.to_string().contains(PURE_AGENT_PROTOCOL),
            "unexpected error: {error}"
        );
    }
}
