use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::execution::config_model::AgentExecutionScope;

use super::mapping::ORCHESTRATOR_ROLE;
use super::validator::RuntimeSelectionDescriptor;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct AgentRuntimeSessionExecution {
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
pub(crate) struct AgentRuntimeSessionHandle {
    pub session_handle_id: String,
    pub orchestration_session_id: String,
    pub agent_id: String,
    pub backend_id: String,
    pub role: String,
    pub protocol: String,
    pub execution: AgentRuntimeSessionExecution,
    pub state: AgentRuntimeSessionState,
    pub opened_at: DateTime<Utc>,
    pub last_transition_at: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub world_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub world_generation: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_session_handle_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resumed_from_session_handle_id: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct AgentRuntimeSessionInternal {
    pub resolved_agent_kind: String,
    pub resolved_binary_path: String,
    pub shell_owner_pid: u32,
    pub lease_token: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uaa_session_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub latest_run_id: Option<String>,
    pub cancel_supported: bool,
    // A manifest is only authoritative-live while the REPL still retains the attached
    // UAA control boundary: the cancel handle remains owned, the event stream task is
    // still active, and the completion observer is still retained.
    #[serde(default)]
    pub control_owner_retained: bool,
    #[serde(default)]
    pub event_stream_active: bool,
    #[serde(default)]
    pub completion_observer_retained: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ownership_mode: Option<String>,
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_error_bucket: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_error_message: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct AgentRuntimeSessionManifest {
    #[serde(flatten)]
    pub handle: AgentRuntimeSessionHandle,
    pub internal: AgentRuntimeSessionInternal,
}

impl AgentRuntimeSessionManifest {
    pub(crate) fn new(
        descriptor: &RuntimeSelectionDescriptor,
        orchestration_session_id: String,
        session_handle_id: String,
        lease_token: String,
    ) -> Self {
        let now = Utc::now();
        Self {
            handle: AgentRuntimeSessionHandle {
                session_handle_id,
                orchestration_session_id,
                agent_id: descriptor.agent_id.clone(),
                backend_id: descriptor.backend_id.clone(),
                role: ORCHESTRATOR_ROLE.to_string(),
                protocol: descriptor.protocol.clone(),
                execution: AgentRuntimeSessionExecution {
                    scope: descriptor.execution_scope,
                },
                state: AgentRuntimeSessionState::Allocating,
                opened_at: now,
                last_transition_at: now,
                world_id: None,
                world_generation: None,
                parent_session_handle_id: None,
                resumed_from_session_handle_id: None,
            },
            internal: AgentRuntimeSessionInternal {
                resolved_agent_kind: descriptor.backend_kind.as_agent_kind_str().to_string(),
                resolved_binary_path: descriptor.binary_path.display().to_string(),
                shell_owner_pid: std::process::id(),
                lease_token,
                uaa_session_id: None,
                latest_run_id: None,
                cancel_supported: true,
                control_owner_retained: false,
                event_stream_active: false,
                completion_observer_retained: false,
                ownership_mode: Some("attached_control".to_string()),
                ownership_valid: false,
                ownership_verified_at: None,
                last_heartbeat_at: Some(now),
                last_event_at: None,
                terminal_observed_at: None,
                termination_reason: None,
                last_error_bucket: None,
                last_error_message: None,
            },
        }
    }

    pub(crate) fn transition_state(&mut self, next: AgentRuntimeSessionState) {
        self.handle.state = next;
        self.handle.last_transition_at = Utc::now();
    }

    pub(crate) fn touch_heartbeat(&mut self) {
        self.internal.last_heartbeat_at = Some(Utc::now());
    }

    pub(crate) fn touch_event(&mut self, ts: DateTime<Utc>) {
        self.internal.last_event_at = Some(ts);
    }

    pub(crate) fn set_uaa_session_id(&mut self, session_id: impl Into<String>) {
        self.internal.uaa_session_id = Some(session_id.into());
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
        self.refresh_ownership_validity();
    }

    pub(crate) fn release_runtime_ownership(&mut self) {
        self.internal.control_owner_retained = false;
        self.internal.event_stream_active = false;
        self.internal.completion_observer_retained = false;
        self.internal.ownership_valid = false;
    }

    pub(crate) fn mark_terminal_state(&mut self, reason: impl Into<String>) {
        let now = Utc::now();
        self.release_runtime_ownership();
        self.internal.terminal_observed_at = Some(Utc::now());
        self.internal.termination_reason = Some(reason.into());
        if self.internal.ownership_verified_at.is_none() {
            self.internal.ownership_verified_at = Some(now);
        }
    }

    pub(crate) fn has_valid_ownership(&self) -> bool {
        self.internal.ownership_valid && self.can_advertise_live()
    }

    pub(crate) fn is_authoritative_live(&self) -> bool {
        self.handle.state.is_live() && self.has_valid_ownership()
    }

    pub(crate) fn last_status_at(&self) -> DateTime<Utc> {
        self.internal
            .last_event_at
            .or(self.internal.last_heartbeat_at)
            .unwrap_or(self.handle.last_transition_at)
    }
}
