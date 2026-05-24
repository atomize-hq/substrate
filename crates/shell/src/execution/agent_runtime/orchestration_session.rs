use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::execution::agent_runtime::control::{
    ResolvedRuntimeBackendKind, ResolvedRuntimeDescriptor,
};
use crate::execution::config_model::AgentExecutionScope;

use super::mapping::{protocol_validation_error, PURE_AGENT_PROTOCOL};
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

    pub(crate) fn is_terminal(&self) -> bool {
        matches!(self, Self::Invalidated | Self::Stopped | Self::Failed)
    }
}

#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum StartupPromptStreamState {
    #[default]
    PendingAcceptance,
    Accepted,
    Completed,
    Failed,
}

impl StartupPromptStreamState {
    pub(crate) fn accepted(self) -> bool {
        !matches!(self, Self::PendingAcceptance)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct StartupPromptRecord {
    pub participant_id: String,
    #[serde(default)]
    pub state: StartupPromptStreamState,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub accepted_at: Option<DateTime<Utc>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub terminal_at: Option<DateTime<Utc>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub turn_outcome: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error_message: Option<String>,
}

#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum OrchestrationSessionPosture {
    ActiveAttached,
    #[default]
    ParkedResumable,
    AwaitingAttention,
    Terminal,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct HostAttachContract {
    pub backend_id: String,
    pub execution_scope: AgentExecutionScope,
    pub protocol: String,
    pub launch_descriptor: ResolvedRuntimeDescriptor,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub continuity_uaa_session_id: Option<String>,
}

impl HostAttachContract {
    fn from_manifest(manifest: &AgentRuntimeSessionManifest) -> Option<Self> {
        if manifest.handle.role != super::mapping::ORCHESTRATOR_ROLE
            || manifest.handle.execution.scope != AgentExecutionScope::Host
        {
            return None;
        }

        let backend_kind = match manifest.internal.resolved_agent_kind.as_str() {
            "codex" => ResolvedRuntimeBackendKind::Codex,
            "claude_code" => ResolvedRuntimeBackendKind::ClaudeCode,
            _ => return None,
        };

        Some(Self {
            backend_id: manifest.handle.backend_id.clone(),
            execution_scope: manifest.handle.execution.scope,
            protocol: manifest.handle.protocol.clone(),
            launch_descriptor: ResolvedRuntimeDescriptor {
                agent_id: manifest.handle.agent_id.clone(),
                backend_id: manifest.handle.backend_id.clone(),
                backend_kind,
                protocol: manifest.handle.protocol.clone(),
                execution_scope: manifest.handle.execution.scope,
                binary_path: manifest.internal.resolved_binary_path.clone(),
            },
            continuity_uaa_session_id: manifest.internal_uaa_session_id().map(ToOwned::to_owned),
        })
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct OrchestrationSessionRecord {
    // This is the public/operator-facing selector for the parent orchestration row.
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
    // Compatibility storage name retained; the value is the active orchestrator participant_id.
    pub active_session_handle_id: Option<String>,
    pub latest_run_id: Option<String>,
    pub world_id: Option<String>,
    pub world_generation: Option<u64>,
    pub invalidation_reason: Option<String>,
    pub closed_at: Option<DateTime<Utc>>,
    #[serde(default)]
    pub posture: OrchestrationSessionPosture,
    #[serde(default = "Utc::now")]
    pub posture_changed_at: DateTime<Utc>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub attached_participant_id: Option<String>,
    #[serde(default)]
    pub pending_inbox_count: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_parked_at: Option<DateTime<Utc>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_attention_at: Option<DateTime<Utc>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parked_reason: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub startup_prompt: Option<StartupPromptRecord>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub host_attach_contract: Option<HostAttachContract>,
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
            posture: OrchestrationSessionPosture::ActiveAttached,
            posture_changed_at: now,
            attached_participant_id: Some(child_manifest.handle.participant_id.clone()),
            pending_inbox_count: 0,
            last_parked_at: None,
            last_attention_at: None,
            parked_reason: None,
            startup_prompt: None,
            host_attach_contract: HostAttachContract::from_manifest(child_manifest),
        }
    }

    pub(crate) fn transition_state(&mut self, next: OrchestrationSessionState) {
        self.state = next;
        self.last_active_at = Utc::now();
        if self.state.is_terminal() {
            self.apply_posture(OrchestrationSessionPosture::Terminal);
            self.attached_participant_id = None;
        } else if self.state.is_active() {
            self.closed_at = None;
            self.invalidation_reason = None;
            if self.posture == OrchestrationSessionPosture::Terminal {
                self.apply_posture(self.detached_posture_without_attachment());
            }
        }
    }

    pub(crate) fn touch_active(&mut self) {
        self.last_active_at = Utc::now();
    }

    pub(crate) fn active_participant_id(&self) -> Option<&str> {
        self.active_session_handle_id.as_deref()
    }

    pub(crate) fn attached_participant_id(&self) -> Option<&str> {
        self.attached_participant_id.as_deref()
    }

    pub(crate) fn has_world_binding(&self) -> bool {
        self.world_id.is_some() && self.world_generation.is_some()
    }

    pub(crate) fn host_attach_contract(&self) -> Option<&HostAttachContract> {
        self.host_attach_contract.as_ref()
    }

    pub(crate) fn sync_host_attach_contract(&mut self, manifest: &AgentRuntimeSessionManifest) {
        let Some(contract) = self.host_attach_contract.as_mut() else {
            self.host_attach_contract = HostAttachContract::from_manifest(manifest);
            return;
        };
        if manifest.handle.role != super::mapping::ORCHESTRATOR_ROLE
            || manifest.handle.execution.scope != AgentExecutionScope::Host
            || manifest.handle.orchestration_session_id != self.orchestration_session_id
            || manifest.handle.agent_id != self.orchestrator_agent_id
        {
            return;
        }

        if let Some(session_id) = manifest.internal_uaa_session_id() {
            contract.continuity_uaa_session_id = Some(session_id.to_string());
        }
    }

    pub(crate) fn fork_successor_attach_contract(&self) -> Option<HostAttachContract> {
        let mut contract = self.host_attach_contract.clone()?;
        contract.continuity_uaa_session_id = None;
        Some(contract)
    }

    pub(crate) fn bind_active_session_handle(&mut self, participant_id: impl Into<String>) {
        let participant_id = participant_id.into();
        self.active_session_handle_id = Some(participant_id.clone());
        self.attached_participant_id = Some(participant_id);
        self.apply_posture(OrchestrationSessionPosture::ActiveAttached);
        self.parked_reason = None;
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

    #[allow(dead_code)]
    pub(crate) fn set_pending_inbox_count(&mut self, pending_inbox_count: u64) {
        self.pending_inbox_count = pending_inbox_count;
        if pending_inbox_count > 0 {
            self.last_attention_at = Some(Utc::now());
        }
        self.touch_active();
    }

    #[allow(dead_code)]
    pub(crate) fn mark_active_attached(&mut self, participant_id: impl Into<String>) {
        self.attached_participant_id = Some(participant_id.into());
        self.apply_posture(OrchestrationSessionPosture::ActiveAttached);
        self.parked_reason = None;
        self.touch_active();
    }

    #[allow(dead_code)]
    pub(crate) fn mark_parked_resumable(&mut self, reason: impl Into<String>) {
        self.attached_participant_id = None;
        self.last_parked_at = Some(Utc::now());
        self.parked_reason = Some(reason.into());
        self.apply_posture(OrchestrationSessionPosture::ParkedResumable);
        self.touch_active();
    }

    #[allow(dead_code)]
    pub(crate) fn mark_awaiting_attention(&mut self) {
        self.attached_participant_id = None;
        self.last_attention_at = Some(Utc::now());
        self.apply_posture(OrchestrationSessionPosture::AwaitingAttention);
        self.touch_active();
    }

    pub(crate) fn validate_persisted_invariants(&self) -> anyhow::Result<()> {
        if self.orchestrator_protocol != PURE_AGENT_PROTOCOL {
            anyhow::bail!(
                "orchestration session {}",
                protocol_validation_error("it", Some(self.orchestrator_protocol.as_str()))
            );
        }

        if let Some(contract) = self.host_attach_contract.as_ref() {
            if contract.execution_scope != AgentExecutionScope::Host {
                anyhow::bail!("host_attach_contract must remain host scoped");
            }
            if contract.protocol != PURE_AGENT_PROTOCOL {
                anyhow::bail!("host_attach_contract must use the pure agent protocol");
            }
            if contract.backend_id != self.orchestrator_backend_id {
                anyhow::bail!("host_attach_contract backend_id must match the session backend");
            }
            if contract.launch_descriptor.backend_id != contract.backend_id {
                anyhow::bail!("host_attach_contract launch_descriptor backend drifted");
            }
            if contract.launch_descriptor.execution_scope != contract.execution_scope {
                anyhow::bail!("host_attach_contract launch_descriptor scope drifted");
            }
            if contract.launch_descriptor.protocol != contract.protocol {
                anyhow::bail!("host_attach_contract launch_descriptor protocol drifted");
            }
        }

        match self.posture {
            OrchestrationSessionPosture::ActiveAttached => {
                if self.attached_participant_id.is_none() {
                    anyhow::bail!("active_attached posture requires attached_participant_id");
                }
            }
            OrchestrationSessionPosture::ParkedResumable => {
                if self.attached_participant_id.is_some() {
                    anyhow::bail!("parked_resumable posture must clear attached_participant_id");
                }
                if self.pending_inbox_count > 0 {
                    anyhow::bail!("parked_resumable posture cannot retain pending inbox items");
                }
            }
            OrchestrationSessionPosture::AwaitingAttention => {
                if self.attached_participant_id.is_some() {
                    anyhow::bail!("awaiting_attention posture must clear attached_participant_id");
                }
                if self.pending_inbox_count == 0 {
                    anyhow::bail!("awaiting_attention posture requires pending_inbox_count > 0");
                }
            }
            OrchestrationSessionPosture::Terminal => {
                if self.attached_participant_id.is_some() {
                    anyhow::bail!("terminal posture must clear attached_participant_id");
                }
            }
        }

        if self.state.is_terminal() && self.posture != OrchestrationSessionPosture::Terminal {
            anyhow::bail!("terminal session state requires terminal posture");
        }
        if !self.state.is_terminal() && self.posture == OrchestrationSessionPosture::Terminal {
            anyhow::bail!("non-terminal session state cannot advertise terminal posture");
        }

        Ok(())
    }

    pub(crate) fn mark_terminal(&mut self, reason: impl Into<String>) {
        let reason = reason.into();
        self.last_active_at = Utc::now();
        self.closed_at = Some(self.last_active_at);
        self.invalidation_reason = Some(reason.clone());
        self.attached_participant_id = None;
        self.parked_reason = Some(reason);
        self.apply_posture(OrchestrationSessionPosture::Terminal);
    }

    pub(crate) fn initialize_startup_prompt(&mut self, participant_id: impl Into<String>) {
        self.startup_prompt = Some(StartupPromptRecord {
            participant_id: participant_id.into(),
            state: StartupPromptStreamState::PendingAcceptance,
            accepted_at: None,
            terminal_at: None,
            turn_outcome: None,
            error_message: None,
        });
        self.touch_active();
    }

    pub(crate) fn startup_prompt_state(&self) -> Option<StartupPromptStreamState> {
        self.startup_prompt.as_ref().map(|record| record.state)
    }

    pub(crate) fn mark_startup_prompt_accepted(&mut self, participant_id: &str) {
        let Some(record) = self.startup_prompt.as_mut() else {
            return;
        };
        if record.participant_id != participant_id || record.state.accepted() {
            return;
        }
        record.state = StartupPromptStreamState::Accepted;
        record.accepted_at = Some(Utc::now());
        record.error_message = None;
        self.touch_active();
    }

    pub(crate) fn mark_startup_prompt_completed(
        &mut self,
        participant_id: &str,
        turn_outcome: impl Into<String>,
    ) {
        let Some(record) = self.startup_prompt.as_mut() else {
            return;
        };
        if record.participant_id != participant_id {
            return;
        }
        record.state = StartupPromptStreamState::Completed;
        record.accepted_at.get_or_insert_with(Utc::now);
        record.terminal_at = Some(Utc::now());
        record.turn_outcome = Some(turn_outcome.into());
        record.error_message = None;
        self.touch_active();
    }

    pub(crate) fn mark_startup_prompt_failed(
        &mut self,
        participant_id: &str,
        error_message: impl Into<String>,
    ) {
        let Some(record) = self.startup_prompt.as_mut() else {
            return;
        };
        if record.participant_id != participant_id {
            return;
        }
        record.state = StartupPromptStreamState::Failed;
        record.accepted_at.get_or_insert_with(Utc::now);
        record.terminal_at = Some(Utc::now());
        record.error_message = Some(error_message.into());
        self.touch_active();
    }

    fn apply_posture(&mut self, posture: OrchestrationSessionPosture) {
        if self.posture != posture {
            self.posture = posture;
            self.posture_changed_at = Utc::now();
        }
    }

    fn detached_posture_without_attachment(&self) -> OrchestrationSessionPosture {
        if self.pending_inbox_count > 0 {
            OrchestrationSessionPosture::AwaitingAttention
        } else {
            OrchestrationSessionPosture::ParkedResumable
        }
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;
    use crate::execution::agent_runtime::{
        mapping::AgentRuntimeBackendKind, validator::RuntimeSelectionDescriptor,
    };
    use crate::execution::config_model::AgentExecutionScope;

    fn manifest() -> AgentRuntimeSessionManifest {
        crate::execution::agent_runtime::session::AgentRuntimeParticipantRecord::new_orchestrator_participant(
            &RuntimeSelectionDescriptor {
                agent_id: "codex".to_string(),
                backend_id: "cli:codex".to_string(),
                backend_kind: AgentRuntimeBackendKind::Codex,
                protocol: "substrate.agent.session".to_string(),
                execution_scope: AgentExecutionScope::Host,
                binary_path: PathBuf::from("/usr/bin/codex"),
            },
            "sess_001".to_string(),
            "ash_001".to_string(),
            "lease_001".to_string(),
        )
        .expect("manifest")
    }

    #[test]
    fn new_session_starts_active_attached() {
        let manifest = manifest();
        let session = OrchestrationSessionRecord::new(
            "sess_001".to_string(),
            "trace_001".to_string(),
            "/workspace".to_string(),
            &manifest,
        );

        assert_eq!(session.posture, OrchestrationSessionPosture::ActiveAttached);
        assert_eq!(session.attached_participant_id(), Some("ash_001"));
        assert_eq!(session.pending_inbox_count, 0);
        session
            .validate_persisted_invariants()
            .expect("new session invariants");
    }

    #[test]
    fn detached_postures_enforce_pending_inbox_truth() {
        let manifest = manifest();
        let mut session = OrchestrationSessionRecord::new(
            "sess_001".to_string(),
            "trace_001".to_string(),
            "/workspace".to_string(),
            &manifest,
        );

        session.mark_parked_resumable("owner detached cleanly");
        session
            .validate_persisted_invariants()
            .expect("parked invariants");

        session.set_pending_inbox_count(1);
        assert!(session.validate_persisted_invariants().is_err());

        session.mark_awaiting_attention();
        session
            .validate_persisted_invariants()
            .expect("attention invariants");
    }
}
