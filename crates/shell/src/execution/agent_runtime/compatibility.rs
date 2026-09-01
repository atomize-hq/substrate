use std::collections::{BTreeMap, BTreeSet};
#[cfg(unix)]
use std::io;

use anyhow::Result;

use crate::execution::config_model::AgentExecutionScope;

use super::{
    mapping::ORCHESTRATOR_ROLE,
    obligation_ledger::OrchestrationObligationRecord,
    orchestration_session::{
        OrchestrationSessionPosture, OrchestrationSessionRecord, OrchestrationSessionState,
    },
    session::AgentRuntimeParticipantRecord,
    state_store::AgentRuntimeSessionRecord,
};

/// The only capability exposed to compatibility projection code.
///
/// Implementations decode and validate existing StateStore formats. This interface intentionally
/// exposes no authority transition, publication, removal, root, path, or transaction capability.
pub(super) trait CompatibilityReadSource {
    fn canonical_participants(&mut self) -> Result<Vec<AgentRuntimeParticipantRecord>>;
    fn flat_participants(&mut self) -> Result<Vec<AgentRuntimeParticipantRecord>>;
    fn legacy_handle_participants(&mut self) -> Result<Vec<AgentRuntimeParticipantRecord>>;

    fn canonical_session(
        &mut self,
        orchestration_session_id: &str,
    ) -> Result<Option<OrchestrationSessionRecord>>;
    fn flat_session(
        &mut self,
        orchestration_session_id: &str,
    ) -> Result<Option<OrchestrationSessionRecord>>;
    fn canonical_session_ids(&mut self) -> Result<Vec<String>>;
    fn flat_session_ids(&mut self) -> Result<Vec<String>>;

    fn legacy_obligation(
        &mut self,
        orchestration_session_id: &str,
        obligation_id: &str,
    ) -> Result<Option<OrchestrationObligationRecord>>;
    fn materialized_obligation(
        &mut self,
        orchestration_session_id: &str,
        obligation_id: &str,
    ) -> Result<Option<OrchestrationObligationRecord>>;
    fn legacy_obligations(
        &mut self,
        orchestration_session_id: &str,
    ) -> Result<Vec<OrchestrationObligationRecord>>;
    fn materialized_obligations(
        &mut self,
        orchestration_session_id: &str,
    ) -> Result<Vec<OrchestrationObligationRecord>>;
}

pub(super) struct CompatibilityReadModel<'a, Source: ?Sized> {
    source: &'a mut Source,
}

impl<'a, Source> CompatibilityReadModel<'a, Source>
where
    Source: CompatibilityReadSource + ?Sized,
{
    pub(super) fn new(source: &'a mut Source) -> Self {
        Self { source }
    }

    pub(super) fn load_participant(
        &mut self,
        participant_id: &str,
    ) -> Result<Option<AgentRuntimeParticipantRecord>> {
        Ok(self
            .list_participants()?
            .into_iter()
            .find(|participant| participant.participant_id() == participant_id))
    }

    pub(super) fn list_participants(&mut self) -> Result<Vec<AgentRuntimeParticipantRecord>> {
        let mut participants = BTreeMap::new();
        for participant in self.source.canonical_participants()? {
            participants.insert(participant.participant_id().to_string(), participant);
        }
        for participant in self.source.flat_participants()? {
            participants
                .entry(participant.participant_id().to_string())
                .or_insert(participant);
        }
        for participant in self.source.legacy_handle_participants()? {
            participants
                .entry(participant.participant_id().to_string())
                .or_insert(participant);
        }

        let mut participants = participants.into_values().collect::<Vec<_>>();
        participants.sort_by(|left, right| {
            left.handle
                .last_transition_at
                .cmp(&right.handle.last_transition_at)
                .then(left.handle.participant_id.cmp(&right.handle.participant_id))
        });
        Ok(participants)
    }

    pub(super) fn list_live_participants(&mut self) -> Result<Vec<AgentRuntimeParticipantRecord>> {
        Ok(self
            .list_participants()?
            .into_iter()
            .filter(|participant| {
                participant.is_authoritative_live() && owner_process_is_alive(participant)
            })
            .collect())
    }

    pub(super) fn load_authoritative_session(
        &mut self,
        orchestration_session_id: &str,
    ) -> Result<Option<OrchestrationSessionRecord>> {
        if let Some(session) = self.source.canonical_session(orchestration_session_id)? {
            return Ok(Some(session));
        }
        self.source.flat_session(orchestration_session_id)
    }

    pub(super) fn load_obligation(
        &mut self,
        orchestration_session_id: &str,
        obligation_id: &str,
    ) -> Result<Option<OrchestrationObligationRecord>> {
        let legacy = self
            .source
            .legacy_obligation(orchestration_session_id, obligation_id)?;
        let materialized = self
            .source
            .materialized_obligation(orchestration_session_id, obligation_id)?;
        match (legacy, materialized) {
            (None, None) => Ok(None),
            (Some(obligation), None) | (None, Some(obligation)) => Ok(Some(obligation)),
            (Some(_), Some(_)) => {
                anyhow::bail!(
                    "duplicate orchestration obligation identity across compatibility surfaces"
                )
            }
        }
    }

    pub(super) fn list_obligations(
        &mut self,
        orchestration_session_id: &str,
    ) -> Result<Vec<OrchestrationObligationRecord>> {
        merge_compatibility_obligations(
            self.source.legacy_obligations(orchestration_session_id)?,
            self.source
                .materialized_obligations(orchestration_session_id)?,
        )
    }

    pub(super) fn load_session(
        &mut self,
        orchestration_session_id: &str,
    ) -> Result<Option<AgentRuntimeSessionRecord>> {
        let session = self.load_authoritative_session(orchestration_session_id)?;
        let participants = self
            .list_participants()?
            .into_iter()
            .filter(|participant| {
                participant.handle.orchestration_session_id == orchestration_session_id
            })
            .collect::<Vec<_>>();
        if session.is_none() && participants.is_empty() {
            return Ok(None);
        }

        let mut record = build_session_record(orchestration_session_id, session, participants);
        let obligations = self.list_obligations(orchestration_session_id)?;
        project_session_attention_compatibility(&mut record.session, &obligations)?;
        Ok(Some(record))
    }

    pub(super) fn list_sessions(&mut self) -> Result<Vec<AgentRuntimeSessionRecord>> {
        let mut session_ids = BTreeSet::new();
        session_ids.extend(self.source.canonical_session_ids()?);
        session_ids.extend(self.source.flat_session_ids()?);
        for participant in self.list_participants()? {
            session_ids.insert(participant.handle.orchestration_session_id);
        }

        let mut sessions = Vec::new();
        for session_id in session_ids {
            if let Some(record) = self.load_session(&session_id)? {
                sessions.push(record);
            }
        }
        sessions.sort_by(|left, right| {
            left.last_updated_at().cmp(&right.last_updated_at()).then(
                left.orchestration_session_id()
                    .cmp(right.orchestration_session_id()),
            )
        });
        Ok(sessions)
    }

    pub(super) fn list_live_sessions(&mut self) -> Result<Vec<AgentRuntimeSessionRecord>> {
        Ok(self
            .list_sessions()?
            .into_iter()
            .filter(|record| {
                record.is_complete()
                    && record.session.state == OrchestrationSessionState::Active
                    && owner_pid_is_alive(record.session.shell_owner_pid)
            })
            .collect())
    }

    pub(super) fn load_orchestration_session(
        &mut self,
        orchestration_session_id: &str,
    ) -> Result<Option<OrchestrationSessionRecord>> {
        let Some(mut session) = self.load_authoritative_session(orchestration_session_id)? else {
            return Ok(None);
        };
        let obligations = self.list_obligations(orchestration_session_id)?;
        project_session_attention_compatibility(&mut session, &obligations)?;
        Ok(Some(session))
    }

    pub(super) fn list_orchestration_sessions(
        &mut self,
    ) -> Result<Vec<OrchestrationSessionRecord>> {
        let mut session_ids = BTreeSet::new();
        session_ids.extend(self.source.canonical_session_ids()?);
        session_ids.extend(self.source.flat_session_ids()?);

        let mut sessions = Vec::new();
        for session_id in session_ids {
            if let Some(session) = self.load_orchestration_session(&session_id)? {
                sessions.push(session);
            }
        }
        sessions.sort_by_key(|session| session.last_active_at);
        Ok(sessions)
    }
}

fn merge_compatibility_obligations(
    legacy: Vec<OrchestrationObligationRecord>,
    materialized: Vec<OrchestrationObligationRecord>,
) -> Result<Vec<OrchestrationObligationRecord>> {
    let mut obligations = BTreeMap::new();
    for obligation in legacy.into_iter().chain(materialized) {
        match obligations.entry(obligation.obligation_id.clone()) {
            std::collections::btree_map::Entry::Vacant(entry) => {
                entry.insert(obligation);
            }
            std::collections::btree_map::Entry::Occupied(_) => {
                anyhow::bail!(
                    "duplicate orchestration obligation identity across compatibility surfaces"
                );
            }
        }
    }
    let mut obligations = obligations.into_values().collect::<Vec<_>>();
    obligations.sort_by(|left, right| {
        left.created_at
            .cmp(&right.created_at)
            .then(left.obligation_id.cmp(&right.obligation_id))
    });
    Ok(obligations)
}

fn project_session_attention_compatibility(
    session: &mut OrchestrationSessionRecord,
    obligations: &[OrchestrationObligationRecord],
) -> Result<()> {
    let projected_pending_count: u64 = obligations
        .iter()
        .filter(|obligation| obligation.projects_detached_attention())
        .count()
        .try_into()
        .map_err(|_| anyhow::anyhow!("pending_inbox_count overflow"))?;
    if projected_pending_count <= session.pending_inbox_count {
        return Ok(());
    }
    session.pending_inbox_count = projected_pending_count;
    if session.state.is_terminal() || session.attached_participant_id().is_some() {
        return Ok(());
    }
    session.posture = if projected_pending_count > 0 {
        OrchestrationSessionPosture::AwaitingAttention
    } else if session.active_participant_id().is_none() {
        OrchestrationSessionPosture::BornUnattached
    } else {
        OrchestrationSessionPosture::ParkedResumable
    };
    Ok(())
}

fn build_session_record(
    orchestration_session_id: &str,
    session: Option<OrchestrationSessionRecord>,
    mut participants: Vec<AgentRuntimeParticipantRecord>,
) -> AgentRuntimeSessionRecord {
    participants.sort_by(|left, right| {
        left.handle
            .last_transition_at
            .cmp(&right.handle.last_transition_at)
            .then(left.handle.participant_id.cmp(&right.handle.participant_id))
    });

    let has_authoritative_parent = session.is_some();
    let mut warnings = Vec::new();
    if !has_authoritative_parent {
        warnings.push(format!(
            "orchestration session {orchestration_session_id} is missing authoritative parent session metadata"
        ));
    }

    let session = session
        .unwrap_or_else(|| synthesize_session_record(orchestration_session_id, &participants));
    let contract_valid = match validate_runtime_contract(&session, &participants) {
        Ok(()) => true,
        Err(err) => {
            warnings.push(format!(
                "orchestration session {} violates persisted runtime contract: {err}",
                session.orchestration_session_id
            ));
            false
        }
    };

    let complete = if !has_authoritative_parent || !contract_valid {
        false
    } else if session.state != OrchestrationSessionState::Active {
        true
    } else {
        match session_authoritative_participant_id(&session) {
            Some(active_participant_id) => match participants
                .iter()
                .find(|participant| participant.handle.participant_id == active_participant_id)
            {
                Some(participant) if participant.matches_public_parent_linkage(&session) => {
                    if session_attached_to_participant(&session, participant) {
                        participant.is_authoritative_live() && owner_process_is_alive(participant)
                    } else {
                        valid_detached_host_continuity_posture(&session, participant, true)
                            .is_some()
                    }
                }
                Some(participant) => {
                    warnings.push(format!(
                        "active orchestration session {} references incomplete live orchestrator participant {}",
                        session.orchestration_session_id, participant.handle.participant_id
                    ));
                    false
                }
                None => {
                    warnings.push(format!(
                        "active orchestration session {} references missing participant {}",
                        session.orchestration_session_id, active_participant_id
                    ));
                    false
                }
            },
            None => {
                if session.posture == OrchestrationSessionPosture::BornUnattached
                    && born_unattached_status_anchor_parts(&session, &participants).is_some()
                {
                    true
                } else {
                    warnings.push(format!(
                        "active orchestration session {} is missing authoritative orchestrator participant linkage",
                        session.orchestration_session_id
                    ));
                    false
                }
            }
        }
    };

    AgentRuntimeSessionRecord::from_compatibility_projection(
        session,
        participants,
        warnings,
        has_authoritative_parent,
        complete,
    )
}

fn synthesize_session_record(
    orchestration_session_id: &str,
    participants: &[AgentRuntimeParticipantRecord],
) -> OrchestrationSessionRecord {
    let template = participants
        .iter()
        .find(|participant| {
            participant.handle.role == ORCHESTRATOR_ROLE
                && participant.handle.execution.scope == AgentExecutionScope::Host
        })
        .or_else(|| participants.first())
        .expect("synthetic session record requires at least one participant");

    let mut session = OrchestrationSessionRecord::new(
        orchestration_session_id.to_string(),
        "<unknown-trace-session>".to_string(),
        "<unknown-workspace-root>".to_string(),
        template,
        None,
    );
    session.opened_at = participants
        .iter()
        .map(|participant| participant.handle.opened_at)
        .min()
        .unwrap_or(session.opened_at);
    session.last_active_at = participants
        .iter()
        .map(AgentRuntimeParticipantRecord::last_status_at)
        .max()
        .unwrap_or(session.last_active_at);

    if let Some(orchestrator) = participants.iter().find(|participant| {
        participant.handle.role == ORCHESTRATOR_ROLE
            && participant.handle.execution.scope == AgentExecutionScope::Host
    }) {
        session.orchestrator_agent_id = orchestrator.handle.agent_id.clone();
        session.orchestrator_backend_id = orchestrator.handle.backend_id.clone();
        session.orchestrator_protocol = orchestrator.handle.protocol.clone();
    }
    session.active_session_handle_id = participants
        .iter()
        .find(|participant| {
            participant.handle.role == ORCHESTRATOR_ROLE
                && participant.handle.execution.scope == AgentExecutionScope::Host
                && participant.is_authoritative_live()
                && owner_process_is_alive(participant)
        })
        .map(|participant| participant.handle.participant_id.clone());
    session.latest_run_id = participants
        .iter()
        .filter_map(|participant| {
            participant
                .internal
                .latest_run_id
                .as_ref()
                .map(|run_id| (participant.last_status_at(), run_id.clone()))
        })
        .max_by(|left, right| left.0.cmp(&right.0))
        .map(|(_, run_id)| run_id);
    session.state = if session.active_session_handle_id.is_some() {
        OrchestrationSessionState::Active
    } else {
        OrchestrationSessionState::Allocating
    };
    session
}

pub(super) fn valid_detached_host_continuity_posture(
    session: &OrchestrationSessionRecord,
    participant: &AgentRuntimeParticipantRecord,
    require_internal_session_id: bool,
) -> Option<OrchestrationSessionPosture> {
    let contract = session.host_attach_contract()?;
    if session.state.is_terminal() || !participant.handle.state.is_live() {
        return None;
    }
    if session.active_participant_id() != Some(participant.participant_id()) {
        return None;
    }
    if !participant.matches_public_parent_linkage(session) {
        return None;
    }
    if session.attached_participant_id().is_some() || participant.attached_client_present() {
        return None;
    }
    if !participant.is_public_attach_continuity_source() {
        return None;
    }
    if !contract.supports_public_attach_continuity() {
        return None;
    }
    if require_internal_session_id && contract.public_attach_continuity_session_id().is_none() {
        return None;
    }

    match session.posture {
        OrchestrationSessionPosture::ParkedResumable if session.pending_inbox_count == 0 => {
            Some(OrchestrationSessionPosture::ParkedResumable)
        }
        OrchestrationSessionPosture::AwaitingAttention if session.pending_inbox_count > 0 => {
            Some(OrchestrationSessionPosture::AwaitingAttention)
        }
        _ => None,
    }
}

pub(super) fn session_authoritative_participant_id(
    session: &OrchestrationSessionRecord,
) -> Option<&str> {
    session
        .active_participant_id()
        .or(session.attached_participant_id())
}

pub(super) fn session_attached_to_participant(
    session: &OrchestrationSessionRecord,
    participant: &AgentRuntimeParticipantRecord,
) -> bool {
    session.attached_participant_id() == Some(participant.participant_id())
}

fn validate_runtime_contract(
    session: &OrchestrationSessionRecord,
    participants: &[AgentRuntimeParticipantRecord],
) -> Result<()> {
    session.validate_persisted_invariants()?;

    let Some(authoritative_participant_id) = session_authoritative_participant_id(session) else {
        if session.state == OrchestrationSessionState::Active
            && session.posture == OrchestrationSessionPosture::BornUnattached
        {
            if born_unattached_status_anchor_parts(session, participants).is_some() {
                return Ok(());
            }
            anyhow::bail!(
                "born_unattached session requires authoritative world member launch proof"
            );
        }
        if session.state == OrchestrationSessionState::Active
            && session.posture == OrchestrationSessionPosture::ActiveAttached
        {
            anyhow::bail!("active_attached session is missing authoritative participant linkage");
        }
        return Ok(());
    };

    let participant = participants
        .iter()
        .find(|participant| participant.participant_id() == authoritative_participant_id)
        .ok_or_else(|| {
            anyhow::anyhow!(
                "authoritative participant {} is missing from the session snapshot",
                authoritative_participant_id
            )
        })?;
    if !participant.matches_public_parent_linkage(session) {
        anyhow::bail!(
            "authoritative participant {} no longer matches the session linkage",
            authoritative_participant_id
        );
    }

    match session.posture {
        OrchestrationSessionPosture::ActiveAttached => {
            if !participant.attached_client_present() {
                anyhow::bail!("active_attached session requires attached host participant truth");
            }
        }
        OrchestrationSessionPosture::BornUnattached => {
            anyhow::bail!("born_unattached sessions must not retain an authoritative participant");
        }
        OrchestrationSessionPosture::ParkedResumable => {
            if !participant.is_resume_eligible() {
                anyhow::bail!("parked_resumable session requires resume-eligible host participant");
            }
        }
        OrchestrationSessionPosture::AwaitingAttention => {
            if !participant.is_resume_eligible() {
                anyhow::bail!(
                    "awaiting_attention session requires resume-eligible host participant"
                );
            }
        }
        OrchestrationSessionPosture::Terminal => {}
    }
    Ok(())
}

pub(super) fn born_unattached_status_anchor(
    record: &AgentRuntimeSessionRecord,
) -> Option<AgentRuntimeParticipantRecord> {
    born_unattached_status_anchor_parts(&record.session, &record.participants)
}

fn born_unattached_status_anchor_parts(
    session: &OrchestrationSessionRecord,
    participants: &[AgentRuntimeParticipantRecord],
) -> Option<AgentRuntimeParticipantRecord> {
    if session.posture != OrchestrationSessionPosture::BornUnattached
        || session.state != OrchestrationSessionState::Active
    {
        return None;
    }
    participants
        .iter()
        .filter(|participant| participant.matches_authoritative_parent_world_binding(session))
        .max_by(|left, right| left.last_status_at().cmp(&right.last_status_at()))
        .cloned()
}

pub(super) fn owner_process_is_alive(participant: &AgentRuntimeParticipantRecord) -> bool {
    owner_pid_is_alive(participant.internal.shell_owner_pid)
}

#[cfg(unix)]
pub(super) fn owner_pid_is_alive(pid: u32) -> bool {
    let pid = pid as libc::pid_t;
    if pid <= 0 {
        return false;
    }
    let rc = unsafe { libc::kill(pid, 0) };
    if rc == 0 {
        return true;
    }
    matches!(io::Error::last_os_error().raw_os_error(), Some(libc::EPERM))
}

#[cfg(not(unix))]
pub(super) fn owner_pid_is_alive(pid: u32) -> bool {
    pid == std::process::id()
}

#[cfg(test)]
mod tests {
    #[test]
    fn compatibility_boundary_has_no_authority_or_publication_capability() {
        let source = include_str!("compatibility.rs");
        let forbidden = [
            concat!("Legacy", "WriterGuard"),
            concat!("HostSession", "Authority"),
            concat!("with_legacy_snapshot_", "transaction"),
            concat!("write_", "file("),
            concat!("remove_", "file("),
            concat!("write_atomic_", "json"),
            concat!("state-root-", "v1"),
            concat!("std::", "fs"),
            concat!("Path", "Buf"),
        ];
        for capability in forbidden {
            assert!(
                !source.contains(capability),
                "read-only compatibility boundary unexpectedly contains {capability}"
            );
        }
    }
}
