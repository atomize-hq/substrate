use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::session::AgentRuntimeSessionManifest;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum OrchestrationSessionState {
    Allocating,
    Active,
    Invalidated,
    Stopping,
    Stopped,
    Failed,
}

impl OrchestrationSessionState {
    pub(crate) fn is_active(&self) -> bool {
        matches!(self, Self::Active)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct OrchestrationSessionRecord {
    pub orchestration_session_id: String,
    pub shell_trace_session_id: String,
    pub workspace_root: String,
    pub shell_owner_pid: u32,
    pub state: OrchestrationSessionState,
    pub opened_at: DateTime<Utc>,
    pub last_active_at: DateTime<Utc>,
    pub orchestrator_agent_id: String,
    pub orchestrator_backend_id: String,
    pub orchestrator_protocol: String,
    pub active_session_handle_id: Option<String>,
    pub latest_run_id: Option<String>,
    pub world_id: Option<String>,
    pub world_generation: Option<u64>,
    pub invalidation_reason: Option<String>,
    pub closed_at: Option<DateTime<Utc>>,
}

impl OrchestrationSessionRecord {
    pub(crate) fn new(
        orchestration_session_id: String,
        shell_trace_session_id: String,
        workspace_root: String,
        child_manifest: &AgentRuntimeSessionManifest,
    ) -> Self {
        let now = Utc::now();
        Self {
            orchestration_session_id,
            shell_trace_session_id,
            workspace_root,
            shell_owner_pid: child_manifest.internal.shell_owner_pid,
            state: OrchestrationSessionState::Allocating,
            opened_at: now,
            last_active_at: now,
            orchestrator_agent_id: child_manifest.handle.agent_id.clone(),
            orchestrator_backend_id: child_manifest.handle.backend_id.clone(),
            orchestrator_protocol: child_manifest.handle.protocol.clone(),
            active_session_handle_id: None,
            latest_run_id: child_manifest.internal.latest_run_id.clone(),
            world_id: None,
            world_generation: None,
            invalidation_reason: None,
            closed_at: None,
        }
    }

    pub(crate) fn transition_state(&mut self, next: OrchestrationSessionState) {
        self.state = next;
        self.last_active_at = Utc::now();
        if self.state.is_active() {
            self.closed_at = None;
            self.invalidation_reason = None;
        }
    }

    pub(crate) fn touch_active(&mut self) {
        self.last_active_at = Utc::now();
    }

    pub(crate) fn bind_active_session_handle(&mut self, session_handle_id: impl Into<String>) {
        self.active_session_handle_id = Some(session_handle_id.into());
        self.touch_active();
    }

    pub(crate) fn set_world_binding(&mut self, world_id: impl Into<String>, world_generation: u64) {
        self.world_id = Some(world_id.into());
        self.world_generation = Some(world_generation);
        self.touch_active();
    }

    pub(crate) fn clear_world_binding(&mut self) {
        self.world_id = None;
        self.world_generation = None;
        self.touch_active();
    }

    pub(crate) fn mark_terminal(&mut self, reason: impl Into<String>) {
        self.last_active_at = Utc::now();
        self.closed_at = Some(self.last_active_at);
        self.invalidation_reason = Some(reason.into());
    }
}
